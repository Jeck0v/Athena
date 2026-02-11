pub mod ast;
#[allow(clippy::module_inception)]
pub mod parser;

pub use parser::parse_athena_file;