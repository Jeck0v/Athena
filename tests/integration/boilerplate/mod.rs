use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

pub mod fastapi_tests;
pub mod flask_tests;
pub mod go_tests;
pub mod common_tests;
pub mod laravel_tests;
pub mod symfony_tests;

// Common test utilities for boilerplate generation tests

pub fn cleanup_project_directory(dir_name: &str) {
    if Path::new(dir_name).exists() {
        fs::remove_dir_all(dir_name).ok();
    }
}

pub fn run_init_command(framework: &str, project_name: &str, args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init").arg(framework).arg(project_name);
    
    for arg in args {
        cmd.arg(arg);
    }
    
    cmd
}

pub fn check_basic_project_structure(project_dir: &Path, expected_files: &[&str]) -> bool {
    for file in expected_files {
        if !project_dir.join(file).exists() {
            return false;
        }
    }
    true
}

// Helper functions to check for specific configurations

pub fn check_file_contains_any(project_dir: &Path, file_names: &[&str], patterns: &[&str]) -> bool {
    for file_name in file_names {
        let file_path = project_dir.join(file_name);
        if file_path.exists() {
            if let Ok(content) = fs::read_to_string(&file_path) {
                for pattern in patterns {
                    if content.contains(pattern) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn check_directory_contains_any(project_dir: &Path, dir_names: &[&str], patterns: &[&str]) -> bool {
    for dir_name in dir_names {
        let dir_path = project_dir.join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".go") {
                                if let Ok(content) = fs::read_to_string(entry.path()) {
                                    for pattern in patterns {
                                        if content.contains(pattern) {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn check_for_postgres_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "requirements.txt", "pyproject.toml", "docker-compose.yml", 
        ".env.example", "config.py", "settings.py"
    ], &["postgres", "psycopg", "postgresql", "POSTGRES_"])
}

pub fn check_for_mongo_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "requirements.txt", "pyproject.toml", "docker-compose.yml",
        ".env.example", "config.py", "settings.py"
    ], &["mongo", "pymongo", "mongodb", "MONGO_"])
}

pub fn check_for_mysql_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "requirements.txt", "Pipfile", "docker-compose.yml",
        ".env.example", "config.py", "app.py"
    ], &["mysql", "pymysql", "MySQL", "MYSQL_"])
}

pub fn check_for_gin_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "main.go", "go.mod", "go.sum"
    ], &["gin-gonic", "gin.Default", "gin.Engine"]) ||
    check_directory_contains_any(project_dir, &["cmd", "internal", "pkg"], &[
        "gin-gonic", "gin.Default", "gin.Engine"
    ])
}

pub fn check_for_echo_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "main.go", "go.mod", "go.sum"
    ], &["labstack/echo", "echo.New", "echo.Echo"]) ||
    check_directory_contains_any(project_dir, &["cmd", "internal", "pkg"], &[
        "labstack/echo", "echo.New", "echo.Echo"
    ])
}

// PHP-specific helper functions

pub fn check_for_laravel_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "composer.json", "config/app.php", "artisan"
    ], &["laravel/framework", "Laravel", "Illuminate"])
}

pub fn check_for_symfony_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "composer.json", "config/services.yaml", "bin/console"
    ], &["symfony/framework-bundle", "Symfony", "symfony/console"])
}

pub fn check_for_jwt_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "composer.json", "config/jwt.php", "config/packages/lexik_jwt_authentication.yaml"
    ], &["tymon/jwt-auth", "lexik/jwt-authentication-bundle", "JWT_SECRET", "jwt"])
}

pub fn check_for_doctrine_configuration(project_dir: &Path) -> bool {
    check_file_contains_any(project_dir, &[
        "composer.json", "config/packages/doctrine.yaml"
    ], &["doctrine/orm", "doctrine/doctrine-bundle", "doctrine/migrations"])
}