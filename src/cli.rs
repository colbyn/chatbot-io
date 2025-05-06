use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    Format(CliFormatCommand),
}

#[derive(Parser, Debug)]
pub struct CliFormatCommand {
    #[arg(long)]
    template: PathBuf,
    /// Array of file paths or unix style glob patterns.
    /// 
    /// The system will try to automatically resolve whether each respective input is a glob or a file path. To disable glob mode checking and treat each input as a file path see the `no_globs` flag.
    #[arg(long, num_args = 1..)]
    input: Vec<String>,
    /// Disable glob mode behavior; all inputs will be considered file paths. OFF by default.
    #[arg(long, default_value_t = false)]
    no_globs: bool,
    /// By default the system will attempt to trim the content string of leading and trailing whitespace. This flag will disable that behavior.
    #[arg(long, default_value_t = false)]
    no_trim: bool,
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
            .set_allow_globs(!self.no_globs)
            .set_trim_contents(!self.no_trim);
        let environment_result = crate::template::Environment::populate_from(
            &self.input,
            settings
        );
        let output_result = environment_result.and_then(|x| {
            x.run_preprocessor(&self.template)
        });
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

