use clap::Parser;
use std::process;

mod cli;
mod athena;

use cli::{Cli, execute_command};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = execute_command(cli.command, cli.verbose) {
        eprintln!("Error: {e}");

        // Print additional context for common errors
        match &e {
            athena::AthenaError::IoError(io_err) => {
                match io_err.kind() {
                    std::io::ErrorKind::NotFound => {
                        eprintln!("Make sure the file path is correct and the file exists.");
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("Check file permissions and try running with appropriate privileges.");
                    }
                    _ => {}
                }
            }
            athena::AthenaError::ParseError(msg) => {
                eprintln!("Check the syntax of your .ath file. Use 'athena info --examples' for syntax examples.");
                if msg.message.contains("Parse error") {
                    eprintln!("Common issues: missing END SERVICE, incorrect keywords, or malformed strings.");
                }
            }
            athena::AthenaError::ValidationError(msg) => {
                eprintln!("Fix the validation issues in your configuration.");
                if msg.message.contains("circular") {
                    eprintln!("Review your service dependencies to avoid circular references.");
                }
            }
            athena::AthenaError::YamlError(_) | athena::AthenaError::ConfigError(_) => {}
        }

        process::exit(1);
    }
}