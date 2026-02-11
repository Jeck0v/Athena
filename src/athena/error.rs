use std::fmt::{self, Write};
use thiserror::Error;

pub type AthenaResult<T> = Result<T, AthenaError>;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum AthenaError {
    #[error("{0}")]
    ParseError(EnhancedParseError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML serialization error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("{0}")]
    ValidationError(EnhancedValidationError),
}

#[derive(Debug, Clone)]
pub struct ErrorLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct EnhancedParseError {
    pub message: String,
    pub location: Option<ErrorLocation>,
    pub suggestion: Option<String>,
    pub file_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnhancedValidationError {
    pub message: String,
    pub suggestion: Option<String>,
    pub related_services: Vec<String>,
}

impl fmt::Display for EnhancedParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = &self.location {
            write!(
                f,
                "Parse error at line {}, column {}: {}",
                location.line, location.column, self.message
            )?;

            if let Some(content) = &self.file_content {
                if let Some(context_display) = self.format_context(content) {
                    write!(f, "\n{context_display}")?;
                }
            }
        } else {
            write!(f, "Parse error: {}", self.message)?;
        }

        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n\nSuggestion: {suggestion}")?;
        }

        Ok(())
    }
}

impl fmt::Display for EnhancedValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)?;

        if !self.related_services.is_empty() {
            write!(f, "\nAffected services: {}", self.related_services.join(", "))?;
        }

        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n\nSuggestion: {suggestion}")?;
        }

        Ok(())
    }
}

impl EnhancedParseError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            location: None,
            suggestion: None,
            file_content: None,
        }
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.location = Some(ErrorLocation { line, column });
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn with_file_content(mut self, content: String) -> Self {
        self.file_content = Some(content);
        self
    }

    fn format_context(&self, content: &str) -> Option<String> {
        let location = self.location.as_ref()?;
        let lines: Vec<&str> = content.lines().collect();

        if location.line == 0 || location.line > lines.len() {
            return None;
        }

        let line_idx = location.line - 1;
        let line = lines[line_idx];

        let mut result = String::new();
        let _ = writeln!(result, "   |");
        let _ = writeln!(result, "{:2} | {line}", location.line);
        let _ = write!(result, "   | ");

        for _ in 0..location.column.saturating_sub(1) {
            result.push(' ');
        }
        result.push_str("^ Error here");

        Some(result)
    }
}

impl EnhancedValidationError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            suggestion: None,
            related_services: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn with_services(mut self, services: Vec<String>) -> Self {
        self.related_services = services;
        self
    }

    pub fn service_reference(service: &str, dependency: &str, available: &[String]) -> Self {
        let message = format!(
            "Service '{service}' depends on '{dependency}' which doesn't exist"
        );

        let suggestion = format!(
            "Available services: {}. Check the service name in your DEPENDS-ON declaration",
            available.join(", ")
        );

        Self::new(message)
            .with_suggestion(suggestion)
            .with_services(vec![service.to_string(), dependency.to_string()])
    }

    pub fn circular_dependency(service: &str) -> Self {
        let message = format!(
            "Circular dependency detected involving service '{service}'"
        );

        let suggestion =
            "Check the DEPENDS-ON declarations in your .ath file and remove circular dependencies"
                .to_string();

        Self::new(message)
            .with_suggestion(suggestion)
            .with_services(vec![service.to_string()])
    }
}

impl AthenaError {
    pub fn parse_error_enhanced(error: EnhancedParseError) -> Self {
        AthenaError::ParseError(error)
    }

    pub fn config_error<T: Into<String>>(msg: T) -> Self {
        AthenaError::ConfigError(msg.into())
    }

    pub fn validation_error_enhanced(error: EnhancedValidationError) -> Self {
        AthenaError::ValidationError(error)
    }
}