use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub fn preprocess(
    template_path: impl AsRef<std::path::Path>,
    environment: &Environment,
) -> Result<String, Box<dyn std::error::Error>> {
    let globals = environment.to_object();
    let source = std::fs::read_to_string(template_path)?;
    let output = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(&source)
        .unwrap()
        .render(&globals)?;
    Ok(output)
}

pub fn preprocess_with_template_str(
    template_str: &str,
    environment: &Environment,
) -> Result<String, Box<dyn std::error::Error>> {
    let globals = environment.to_object();
    let output = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(&template_str)
        .unwrap()
        .render(&globals)?;
    Ok(output)
}

#[derive(Serialize, Deserialize)]
pub struct Environment {
    pub files: Vec<File>,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub fence: String,
}

#[derive(Debug, Clone, Copy)]
pub struct EnvironmentPopulateSettings {
    pub allow_globs: bool,
}

impl EnvironmentPopulateSettings {
    pub fn set_allow_globs(mut self, allow_globs: bool) -> Self {
        self.allow_globs = allow_globs;
        self
    }
}

impl std::default::Default for EnvironmentPopulateSettings {
    fn default() -> Self {
        EnvironmentPopulateSettings {
            allow_globs: true,
        }
    }
}

// EXTERNAL

impl Environment {
    pub fn populate_from(
        input: &[String],
        settings: EnvironmentPopulateSettings,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut stdin_count = 0usize;
        let mut path_inputs = Vec::<String>::new();

        for item in input {
            if item == "-" {
                stdin_count += 1;
            } else {
                path_inputs.push(item.clone());
            }
        }

        let mut environment = if settings.allow_globs {
            Self::populate_from_dynamic(&path_inputs, settings)?
        } else {
            let paths = path_inputs.iter().map(PathBuf::from).collect::<Vec<_>>();
            Self::populate_from_files(&paths, settings)?
        };

        if stdin_count > 0 {
            if stdin_count > 1 {
                return Err("stdin input `-` may only be used once".into());
            }

            let mut stdin_content = String::new();

            use std::io::Read;
            std::io::stdin().read_to_string(&mut stdin_content)?;

            environment.files.push(File::from_stdin(stdin_content, settings));
        }

        Ok(environment)
    }
    
    pub fn populate_from_dynamic(
        patterns: &[String],
        settings: EnvironmentPopulateSettings,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut files = Vec::<File>::new();
        for path in resolve_file_path_patterns(patterns)? {
            match File::from_file(path, settings) {
                Ok(file) => {
                    files.push(file);
                }
                Err(error) => return Err(error),
            }
        }
        let environment = Self { files };
        Ok(environment)
    }
    pub fn populate_from_files(
        files: &[PathBuf],
        settings: EnvironmentPopulateSettings,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let files = files
            .iter()
            .map(|path| File::from_file(path, settings).unwrap())
            .collect::<Vec<_>>();
        let environment = Self { files };
        Ok(environment)
    }
    pub fn populate_from_glob(
        glob_pattern: &str,
        settings: EnvironmentPopulateSettings,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let files = glob::glob(glob_pattern)?
            .map(|x| x.ok().unwrap())
            .map(|path| File::from_file(path, settings).unwrap())
            .collect::<Vec<_>>();
        let environment = Self { files };
        Ok(environment)
    }
    pub fn run_preprocessor(
        &self,
        template_path: impl AsRef<std::path::Path>
    ) -> Result<String, Box<dyn std::error::Error>> {
        preprocess(template_path, self)
    }
    pub fn run_preprocessor_with_template_str(
        &self,
        template_str: impl AsRef<str>
    ) -> Result<String, Box<dyn std::error::Error>> {
        preprocess_with_template_str(template_str.as_ref(), self)
    }
}

impl File {
    pub fn from_file(
        file_path: impl AsRef<std::path::Path>,
        settings: EnvironmentPopulateSettings,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = settings;
        let file_path = file_path.as_ref();
        let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

        let file_content = std::fs::read_to_string(file_path)?;

        // if settings.trim_contents {
        //     file_content = file_content.trim().to_owned();
        // }

        Ok(File {
            name: file_name,
            path: file_path.to_path_buf(),
            fence: compute_fence(&file_content),
            content: file_content,
        })
    }
}

// INTERNAL

impl Environment {
    fn to_object(&self) -> liquid::Object {
        let mut object = liquid::Object::default();
        let files = self.files
            .iter()
            .map(|x| x.to_object().into())
            .collect::<Vec<_>>();
        let files = liquid::model::Value::Array(files);
        object.insert("files".into(), files);
        object
    }
}

impl File {
    fn to_object(&self) -> liquid::Object {
        let name: liquid::model::Value = liquid::model::Value::scalar(self.name.clone());
        let content: liquid::model::Value = liquid::model::Value::scalar(self.content.clone());
        let fence: liquid::model::Value = liquid::model::Value::scalar(self.fence.clone());
        let mut object = liquid::Object::default();
        let path = self.path
            .to_str()
            .map(|x| x.to_string())
            .unwrap_or(self.name.clone());
        let path = liquid::model::Value::scalar(path);

        // File metadata
        object.insert("name".into(), name);
        object.insert("path".into(), path);

        // File content
        object.insert("content".into(), content);
        
        // Markdown fenced code block helper
        object.insert("fence".into(), fence);

        object
    }
}

fn resolve_file_path_patterns(patterns: &[String]) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    fn resolve_entry_as_glob(pattern: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut results = Vec::<PathBuf>::new();
        for pattern in glob::glob(pattern)? {
            match pattern {
                Ok(path) => {
                    results.push(path);
                    continue;
                }
                Err(error) => return Err(Box::new(error)),
            }
        }
        Ok(results)
    }
    fn resolve_entry(pattern: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        if let Ok(results) = resolve_entry_as_glob(pattern) {
            return Ok(results)
        }
        let path = PathBuf::from(pattern);
        return Ok(vec![path])
    }
    let mut results = Vec::<PathBuf>::new();
    for pattern in patterns {
        match resolve_entry(&pattern) {
            Ok(paths) => {
                results.extend(paths);
            }
            Err(error) => {
                return Err(error)
            }
        }
    }
    Ok(results)
}

impl File {
    pub fn from_stdin(
        content: impl Into<String>,
        settings: EnvironmentPopulateSettings,
    ) -> Self {
        let _ = settings;
        let content = content.into();

        // if settings.trim_contents {
        //     content = content.trim().to_owned();
        // }
        let content = content.trim().to_owned();

        File {
            name: "<stdin>".to_string(),
            path: PathBuf::from("<stdin>"),
            fence: compute_fence(&content),
            content,
        }
    }
}

fn compute_fence(content: &str) -> String {
    let mut fence = String::from("```");

    loop {
        if !content.contains(&fence) {
            return fence;
        }

        fence = format!("{fence}`");
    }
}
