pub mod error;
pub mod parser;
pub mod generator;

pub use error::{AthenaError, AthenaResult};
pub use parser::parse_athena_file;
pub use generator::generate_docker_compose;

/// Configuration for Athena operations
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AthenaConfig {
    pub output_file: Option<String>,
    pub validate_only: bool,
    pub verbose: bool,
}