use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "athena",
    version = "0.1.0",
    about = "A powerful CLI tool for DSL-based Docker Compose generation and boilerplate creation",
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

    /// Initialize new project with boilerplate code
    #[command(subcommand, alias = "i")]
    Init(InitCommands),

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

#[derive(Subcommand, Debug)]
pub enum InitCommands {
    /// Initialize FastAPI project with production-ready setup
    Fastapi {
        /// Project name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project directory (defaults to project name)
        #[arg(short, long, value_name = "DIR")]
        directory: Option<PathBuf>,

        /// Include MongoDB configuration
        #[arg(long)]
        with_mongodb: bool,

        /// Include PostgreSQL configuration instead of MongoDB
        #[arg(long)]
        with_postgresql: bool,

        /// Skip Docker files generation
        #[arg(long)]
        no_docker: bool,
    },

    /// Initialize Flask project with production-ready setup
    Flask {
        /// Project name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project directory (defaults to project name)
        #[arg(short, long, value_name = "DIR")]
        directory: Option<PathBuf>,

        /// Use MySQL database instead of PostgreSQL
        #[arg(long)]
        with_mysql: bool,

        /// Skip Docker files generation
        #[arg(long)]
        no_docker: bool,
    },

    /// Initialize Go project with production-ready setup  
    Go {
        /// Project name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project directory (defaults to project name)
        #[arg(short, long, value_name = "DIR")]
        directory: Option<PathBuf>,

        /// Web framework choice
        #[arg(long, value_enum, default_value = "gin")]
        framework: GoFramework,

        /// Include MongoDB configuration
        #[arg(long)]
        with_mongodb: bool,

        /// Include PostgreSQL configuration instead of MongoDB
        #[arg(long)]
        with_postgresql: bool,

        /// Skip Docker files generation
        #[arg(long)]
        no_docker: bool,
    },

    /// Generate a Laravel PHP project boilerplate with Clean Architecture
    Laravel {
        /// Project name
        name: String,

        /// Output directory (defaults to project name)
        directory: Option<std::path::PathBuf>,

        /// Include MySQL configuration instead of PostgreSQL
        #[arg(long)]
        with_mysql: bool,

        /// Skip Docker files generation
        #[arg(long)]
        no_docker: bool,
    },

    /// Generate a Symfony PHP project boilerplate with Hexagonal Architecture
    Symfony {
        /// Project name
        name: String,

        /// Output directory (defaults to project name)
        directory: Option<std::path::PathBuf>,

        /// Include MySQL configuration instead of PostgreSQL
        #[arg(long)]
        with_mysql: bool,

        /// Skip Docker files generation
        #[arg(long)]
        no_docker: bool,
    },
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum GoFramework {
    Gin,
    Echo,
    Fiber,
}

impl std::fmt::Display for GoFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoFramework::Gin => write!(f, "gin"),
            GoFramework::Echo => write!(f, "echo"),
            GoFramework::Fiber => write!(f, "fiber"),
        }
    }
}