use std::path::PathBuf;
use clap::{Parser, Subcommand};

static DEFAULT_TEMPLATE_SOURCE: &'static str = include_str!("../default.liquid");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    #[clap(alias = "f")]
    Format(CliFormatCommand),
}

#[derive(Parser, Debug)]
pub struct CliFormatCommand {
    /// The liquid template file; none uses the default template.
    #[arg(long)]
    template: Option<PathBuf>,
    /// Array of file paths or unix style glob patterns.
    /// 
    /// The system will try to automatically resolve whether each respective input is a glob or a file path. To disable glob mode checking and treat each input as a file path see the `no_globs` flag.
    #[arg(short, long, num_args = 1..)]
    input: Vec<String>,
    /// Disable glob mode behavior; all inputs will be considered file paths. OFF by default.
    #[arg(long, default_value_t = false)]
    no_globs: bool,
}

impl Cli {
    pub fn execute(self) {
        match self.command {
            CliCommand::Format(format_cmd) => format_cmd.execute(),
        }
    }
}

impl CliFormatCommand {
    pub fn execute(self) {
        let settings = crate::template::EnvironmentPopulateSettings::default()
            .set_allow_globs(!self.no_globs);
        let environment_result = crate::template::Environment::populate_from(
            &self.input,
            settings
        );
        let output_result = {
            if let Some(template_path) = self.template.as_ref() {
                environment_result.and_then(|x| {
                    x.run_preprocessor(&template_path)
                })
            } else {
                environment_result.and_then(|x| {
                    x.run_preprocessor_with_template_str(DEFAULT_TEMPLATE_SOURCE)
                })
            }
        };
        match output_result {
            Ok(output) => {
                println!("{output}")
            }
            Err(error) => {
                if let Some(error) = error.downcast_ref::<std::io::Error>() {
                    panic!("Failed to read input(s): {error}")
                }
                if let Some(error) = error.downcast_ref::<liquid::Error>() {
                    panic!("Failed to process liquid template: {error}")
                }
                panic!("Failed: {error}")
            }
        }
        // let output = environment
    }
}

