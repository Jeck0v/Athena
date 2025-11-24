use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "athena",
    version = "0.1.0",
    about = "A powerful CLI tool for DSL-based Docker Compose generation",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build docker-compose.yml from Athena DSL file
    #[command(alias = "b")]
    Build {
        /// Input .ath file path (auto-detects if not specified)
        #[arg(value_name = "FILE")]
        input: Option<PathBuf>,

        /// Output file path (defaults to docker-compose.yml)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Validate syntax only, don't generate output
        #[arg(long)]
        validate_only: bool,

        /// Quiet mode (disable verbose output)
        #[arg(short, long)]
        quiet: bool,
    },

    /// Validate Athena DSL file syntax
    #[command(alias = "v")]
    Validate {
        /// Input .ath file path (auto-detects if not specified)
        #[arg(value_name = "FILE")]
        input: Option<PathBuf>,
    },

    /// Show information about Athena DSL syntax
    Info {
        /// Show examples
        #[arg(long)]
        examples: bool,

        /// Show supported directives
        #[arg(long)]
        directives: bool,
    },
}