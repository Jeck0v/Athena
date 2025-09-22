use thiserror::Error;

pub type AthenaResult<T> = Result<T, AthenaError>;

#[derive(Error, Debug)]
pub enum AthenaError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML serialization error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Template error: {0}")]
    #[allow(dead_code)]
    TemplateError(String),
}

impl AthenaError {
    #[allow(dead_code)]
    pub fn parse_error<T: Into<String>>(msg: T) -> Self {
        AthenaError::ParseError(msg.into())
    }

    #[allow(dead_code)]
    pub fn config_error<T: Into<String>>(msg: T) -> Self {
        AthenaError::ConfigError(msg.into())
    }

    #[allow(dead_code)]
    pub fn validation_error<T: Into<String>>(msg: T) -> Self {
        AthenaError::ValidationError(msg.into())
    }

    #[allow(dead_code)]
    pub fn template_error<T: Into<String>>(msg: T) -> Self {
        AthenaError::TemplateError(msg.into())
    }
}