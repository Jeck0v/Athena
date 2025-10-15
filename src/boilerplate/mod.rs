//! Boilerplate generation module for FastAPI, Flask, Go, and PHP projects
//! 
//! This module provides production-ready project templates with:
//! - Authentication systems (JWT with refresh tokens)
//! - Security best practices (bcrypt/argon2, AES-256)
//! - Database integration (MongoDB/PostgreSQL/MySQL)
//! - Docker containerization
//! - Nginx reverse proxy configuration
//! - Clean Architecture and DDD patterns

pub mod fastapi;
pub mod flask;
pub mod go;
pub mod php;
pub mod templates;
pub mod utils;

use crate::athena::AthenaError;
use std::path::Path;

pub type BoilerplateResult<T> = Result<T, AthenaError>;

#[derive(Debug, Clone)]
pub enum DatabaseType {
    MongoDB,
    PostgreSQL,
    MySQL,
}

#[derive(Debug, Clone)]
pub enum GoFramework {
    Gin,
    Echo,
    Fiber,
}

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub directory: String,
    pub database: DatabaseType,
    pub include_docker: bool,
    #[allow(dead_code)]
    pub framework: Option<GoFramework>,
}

pub trait BoilerplateGenerator {
    fn generate_project(&self, config: &ProjectConfig) -> BoilerplateResult<()>;
    fn validate_config(&self, config: &ProjectConfig) -> BoilerplateResult<()>;
}

/// Generate a FastAPI boilerplate project
pub fn generate_fastapi_project(config: &ProjectConfig) -> BoilerplateResult<()> {
    let generator = fastapi::FastAPIGenerator::new();
    generator.validate_config(config)?;
    generator.generate_project(config)
}

/// Generate a Flask boilerplate project
pub fn generate_flask_project(config: &ProjectConfig) -> BoilerplateResult<()> {
    let generator = flask::FlaskGenerator::new();
    generator.validate_config(config)?;
    generator.generate_project(config)
}

/// Generate a Go boilerplate project
pub fn generate_go_project(config: &ProjectConfig) -> BoilerplateResult<()> {
    let generator = go::GoGenerator::new();
    generator.validate_config(config)?;
    generator.generate_project(config)
}

/// Generate a Laravel PHP boilerplate project
pub fn generate_laravel_project(config: &ProjectConfig) -> BoilerplateResult<()> {
    let generator = php::PhpGenerator::new();
    generator.validate_config(config)?;
    generator.generate_laravel_project(config)
}

/// Generate a Symfony PHP boilerplate project
pub fn generate_symfony_project(config: &ProjectConfig) -> BoilerplateResult<()> {
    let generator = php::PhpGenerator::new();
    generator.validate_config(config)?;
    generator.generate_symfony_project(config)
}

/// Validate project name
pub fn validate_project_name(name: &str) -> BoilerplateResult<()> {
    if name.is_empty() {
        return Err(AthenaError::validation_error_simple("Project name cannot be empty".to_string()));
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(AthenaError::validation_error_simple(
            "Project name can only contain alphanumeric characters, underscores, and hyphens".to_string()
        ));
    }
    
    if name.len() > 50 {
        return Err(AthenaError::validation_error_simple("Project name must be 50 characters or less".to_string()));
    }
    
    Ok(())
}

/// Check if directory already exists
pub fn check_directory_availability(path: &Path) -> BoilerplateResult<()> {
    if path.exists() {
        return Err(AthenaError::validation_error_simple(
            format!("Directory '{}' already exists", path.display())
        ));
    }
    Ok(())
}