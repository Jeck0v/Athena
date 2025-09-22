use thiserror::Error;
use std::fmt;

pub type AthenaResult<T> = Result<T, AthenaError>;

#[derive(Error, Debug)]
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

    #[error("Template error: {0}")]
    #[allow(dead_code)]
    TemplateError(String),
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
    pub context: Option<String>,
    pub suggestion: Option<String>,
    pub file_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnhancedValidationError {
    pub message: String,
    pub error_type: ValidationErrorType,
    pub suggestion: Option<String>,
    pub related_services: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationErrorType {
    PortConflict,
    ServiceReference,
    CircularDependency,
    MissingConfiguration,
    InvalidFormat,
}

impl fmt::Display for EnhancedParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = &self.location {
            write!(f, "Parse error at line {}, column {}: {}", 
                   location.line, location.column, self.message)?;
            
            if let Some(content) = &self.file_content {
                if let Some(context_display) = self.format_context(content) {
                    write!(f, "\n{}", context_display)?;
                }
            }
        } else {
            write!(f, "Parse error: {}", self.message)?;
        }
        
        if let Some(context) = &self.context {
            write!(f, "\n{}", context)?;
        }
        
        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n\nSuggestion: {}", suggestion)?;
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
            write!(f, "\n\nSuggestion: {}", suggestion)?;
        }
        
        Ok(())
    }
}

impl EnhancedParseError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            location: None,
            context: None,
            suggestion: None,
            file_content: None,
        }
    }
    
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.location = Some(ErrorLocation { line, column });
        self
    }
    
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
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
        result.push_str("   |\n");
        result.push_str(&format!("{:2} | {}\n", location.line, line));
        result.push_str("   | ");
        
        for _ in 0..location.column.saturating_sub(1) {
            result.push(' ');
        }
        result.push_str("^ Error here");
        
        Some(result)
    }
}

impl EnhancedValidationError {
    pub fn new(message: String, error_type: ValidationErrorType) -> Self {
        Self {
            message,
            error_type,
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
    
    pub fn port_conflict(port: &str, services: Vec<String>) -> Self {
        let message = format!(
            "Port conflict detected! Host port {} is used by multiple services: {}",
            port, services.join(", ")
        );
        
        let suggestion = format!(
            "Use different host ports, e.g., {}",
            generate_port_suggestions(port, services.len())
        );
        
        Self::new(message, ValidationErrorType::PortConflict)
            .with_suggestion(suggestion)
            .with_services(services)
    }
    
    pub fn service_reference(service: &str, dependency: &str, available: &[String]) -> Self {
        let message = format!(
            "Service '{}' depends on '{}' which doesn't exist",
            service, dependency
        );
        
        let suggestion = format!(
            "Available services: {}. Check the service name in your DEPENDS-ON declaration",
            available.join(", ")
        );
        
        Self::new(message, ValidationErrorType::ServiceReference)
            .with_suggestion(suggestion)
            .with_services(vec![service.to_string(), dependency.to_string()])
    }
    
    pub fn circular_dependency(service: &str) -> Self {
        let message = format!(
            "Circular dependency detected involving service '{}'",
            service
        );
        
        let suggestion = "Check the DEPENDS-ON declarations in your .ath file and remove circular dependencies".to_string();
        
        Self::new(message, ValidationErrorType::CircularDependency)
            .with_suggestion(suggestion)
            .with_services(vec![service.to_string()])
    }
}

fn generate_port_suggestions(base_port: &str, count: usize) -> String {
    if let Ok(port_num) = base_port.parse::<u16>() {
        let mut suggestions = Vec::new();
        for i in 0..count {
            suggestions.push(format!("{}:{}", port_num + i as u16, port_num));
        }
        suggestions.join(", ")
    } else {
        "8080:80, 8081:80, 8082:80".to_string()
    }
}

impl AthenaError {
    #[allow(dead_code)]
    pub fn parse_error_simple<T: Into<String>>(msg: T) -> Self {
        AthenaError::ParseError(EnhancedParseError::new(msg.into()))
    }

    #[allow(dead_code)]
    pub fn parse_error_with_location<T: Into<String>>(msg: T, line: usize, column: usize) -> Self {
        AthenaError::ParseError(
            EnhancedParseError::new(msg.into())
                .with_location(line, column)
        )
    }

    #[allow(dead_code)]
    pub fn parse_error_enhanced(error: EnhancedParseError) -> Self {
        AthenaError::ParseError(error)
    }

    #[allow(dead_code)]
    pub fn config_error<T: Into<String>>(msg: T) -> Self {
        AthenaError::ConfigError(msg.into())
    }

    #[allow(dead_code)]
    pub fn validation_error_simple<T: Into<String>>(msg: T) -> Self {
        AthenaError::ValidationError(
            EnhancedValidationError::new(msg.into(), ValidationErrorType::InvalidFormat)
        )
    }

    #[allow(dead_code)]
    pub fn validation_error_enhanced(error: EnhancedValidationError) -> Self {
        AthenaError::ValidationError(error)
    }

    #[allow(dead_code)]
    pub fn template_error<T: Into<String>>(msg: T) -> Self {
        AthenaError::TemplateError(msg.into())
    }
}