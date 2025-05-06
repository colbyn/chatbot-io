use clap::Parser;

pub mod cli;
pub mod template;

fn main() {
    cli::Cli::parse().execute();
}

