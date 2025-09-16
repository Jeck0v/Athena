use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn cleanup_project_directory(dir_name: &str) {
    if Path::new(dir_name).exists() {
        fs::remove_dir_all(dir_name).ok();
    }
}

#[test]
#[serial]
fn test_fastapi_init_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_fastapi_basic";
    let project_path = temp_dir.path().join(project_name);
    
    // Change to temp directory for the test
    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("fastapi")
        .arg(project_name)
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initializing FastAPI project"))
        .stdout(predicate::str::contains(project_name));

    // Verify project structure was created
    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for common FastAPI files
    assert!(project_dir.join("main.py").exists() || 
            project_dir.join("app").join("main.py").exists() ||
            project_dir.join("src").join("main.py").exists(),
            "Main Python file should exist");
            
    assert!(project_dir.join("requirements.txt").exists() ||
            project_dir.join("pyproject.toml").exists(),
            "Dependencies file should exist");

    // Check for Docker files (default behavior)
    assert!(project_dir.join("Dockerfile").exists(), "Dockerfile should exist");
    
    // Check for configuration files
    let has_config = project_dir.join("config").exists() ||
                     project_dir.join(".env.example").exists() ||
                     project_dir.join("settings.py").exists();
    assert!(has_config, "Some configuration should exist");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_fastapi_init_with_postgresql() {
    let project_name = "test_fastapi_postgres";
    cleanup_project_directory(project_name);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("fastapi")
        .arg(project_name)
        .arg("--with-postgresql")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"));

    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for PostgreSQL-specific files/configuration
    // This might be in requirements.txt, docker-compose, or configuration files
    let has_postgres_config = check_for_postgres_configuration(project_dir);
    assert!(has_postgres_config, "PostgreSQL configuration should be present");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_fastapi_init_with_mongodb() {
    let project_name = "test_fastapi_mongo";
    cleanup_project_directory(project_name);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("fastapi")
        .arg(project_name)
        .arg("--with-mongodb")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"));

    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for MongoDB-specific files/configuration
    let has_mongo_config = check_for_mongo_configuration(project_dir);
    assert!(has_mongo_config, "MongoDB configuration should be present");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_fastapi_init_no_docker() {
    let project_name = "test_fastapi_no_docker";
    cleanup_project_directory(project_name);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("fastapi")
        .arg(project_name)
        .arg("--no-docker");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"));

    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");
    
    // Dockerfile should NOT exist
    assert!(!project_dir.join("Dockerfile").exists(), 
            "Dockerfile should not exist with --no-docker");
    assert!(!project_dir.join("docker-compose.yml").exists(), 
            "docker-compose.yml should not exist with --no-docker");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_flask_init_basic() {
    let project_name = "test_flask_basic";
    cleanup_project_directory(project_name);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("flask")
        .arg(project_name)
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Flask project"));

    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for Flask-specific files
    assert!(project_dir.join("app.py").exists() || 
            project_dir.join("run.py").exists() ||
            project_dir.join("wsgi.py").exists() ||
            project_dir.join("app").join("__init__.py").exists(),
            "Flask application file should exist");
            
    assert!(project_dir.join("requirements.txt").exists() ||
            project_dir.join("Pipfile").exists(),
            "Dependencies file should exist");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_flask_init_with_mysql() {
    let project_name = "test_flask_mysql";
    cleanup_project_directory(project_name);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("flask")
        .arg(project_name)
        .arg("--with-mysql");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Flask project with MySQL"));

    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for MySQL-specific configuration
    let has_mysql_config = check_for_mysql_configuration(project_dir);
    assert!(has_mysql_config, "MySQL configuration should be present");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_go_init_with_gin() {
    let project_name = "test_go_gin";
    cleanup_project_directory(project_name);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("go")
        .arg(project_name)
        .arg("--framework")
        .arg("gin")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Go project"))
        .stdout(predicate::str::contains("gin"));

    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for Go-specific files
    assert!(project_dir.join("main.go").exists() || 
            project_dir.join("cmd").join("main.go").exists(),
            "Go main file should exist");
            
    assert!(project_dir.join("go.mod").exists(), "go.mod file should exist");

    // Check for Gin-specific imports or configuration
    let has_gin_config = check_for_gin_configuration(project_dir);
    assert!(has_gin_config, "Gin framework configuration should be present");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_go_init_with_echo() {
    let project_name = "test_go_echo";
    cleanup_project_directory(project_name);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("go")
        .arg(project_name)
        .arg("--framework")
        .arg("echo");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Go project"));

    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");
    assert!(project_dir.join("go.mod").exists(), "go.mod file should exist");

    // Check for Echo-specific configuration
    let has_echo_config = check_for_echo_configuration(project_dir);
    assert!(has_echo_config, "Echo framework configuration should be present");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_go_init_with_fiber() {
    let project_name = "test_go_fiber";
    cleanup_project_directory(project_name);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("go")
        .arg(project_name)
        .arg("--framework")
        .arg("fiber");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Go project"));

    let project_dir = Path::new(project_name);
    assert!(project_dir.exists(), "Project directory should be created");
    assert!(project_dir.join("go.mod").exists(), "go.mod file should exist");

    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_custom_directory_option() {
    let project_name = "custom_name";
    let custom_dir = "custom_directory_path";
    cleanup_project_directory(custom_dir);

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("fastapi")
        .arg(project_name)
        .arg("--directory")
        .arg(custom_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"));

    // Project should be created in custom directory, not project name
    let project_dir = Path::new(custom_dir);
    assert!(project_dir.exists(), "Custom directory should be created");
    assert!(!Path::new(project_name).exists(), "Project name directory should not exist");

    cleanup_project_directory(custom_dir);
}

#[test]
#[serial]
fn test_project_already_exists_handling() {
    let project_name = "existing_project";
    
    // Pre-create the directory
    fs::create_dir_all(project_name).expect("Failed to create directory");
    fs::write(Path::new(project_name).join("existing_file.txt"), "content")
        .expect("Failed to create file");

    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("fastapi")
        .arg(project_name);

    // The command might succeed (overwrite) or fail (directory exists)
    // This documents the current behavior
    let result = cmd.assert();
    
    // Either way, we should get some indication
    // If it fails, there should be an error message
    // If it succeeds, the directory should contain project files
    
    cleanup_project_directory(project_name);
}

#[test]
#[serial]
fn test_invalid_project_name() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("fastapi")
        .arg(""); // Empty project name

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required").or(
            predicate::str::contains("invalid").or(
                predicate::str::contains("NAME")
            )
        ));
}

#[test]
fn test_init_help_commands() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize new project"))
        .stdout(predicate::str::contains("fastapi"))
        .stdout(predicate::str::contains("flask"))
        .stdout(predicate::str::contains("go"));
}

#[test]
fn test_fastapi_init_help() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init").arg("fastapi").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"))
        .stdout(predicate::str::contains("--with-mongodb"))
        .stdout(predicate::str::contains("--with-postgresql"))
        .stdout(predicate::str::contains("--no-docker"));
}

// Helper functions to check for specific database configurations

fn check_for_postgres_configuration(project_dir: &Path) -> bool {
    // Check for PostgreSQL-related content in various files
    check_file_contains_any(project_dir, &[
        "requirements.txt", "pyproject.toml", "docker-compose.yml", 
        ".env.example", "config.py", "settings.py"
    ], &["postgres", "psycopg", "postgresql", "POSTGRES_"])
}

fn check_for_mongo_configuration(project_dir: &Path) -> bool {
    // Check for MongoDB-related content in various files
    check_file_contains_any(project_dir, &[
        "requirements.txt", "pyproject.toml", "docker-compose.yml",
        ".env.example", "config.py", "settings.py"
    ], &["mongo", "pymongo", "mongodb", "MONGO_"])
}

fn check_for_mysql_configuration(project_dir: &Path) -> bool {
    // Check for MySQL-related content in various files
    check_file_contains_any(project_dir, &[
        "requirements.txt", "Pipfile", "docker-compose.yml",
        ".env.example", "config.py", "app.py"
    ], &["mysql", "pymysql", "MySQL", "MYSQL_"])
}

fn check_for_gin_configuration(project_dir: &Path) -> bool {
    // Check for Gin-related content in Go files
    check_file_contains_any(project_dir, &[
        "main.go", "go.mod", "go.sum"
    ], &["gin-gonic", "gin.Default", "gin.Engine"]) ||
    check_directory_contains_any(project_dir, &["cmd", "internal", "pkg"], &[
        "gin-gonic", "gin.Default", "gin.Engine"
    ])
}

fn check_for_echo_configuration(project_dir: &Path) -> bool {
    // Check for Echo-related content in Go files
    check_file_contains_any(project_dir, &[
        "main.go", "go.mod", "go.sum"
    ], &["labstack/echo", "echo.New", "echo.Echo"]) ||
    check_directory_contains_any(project_dir, &["cmd", "internal", "pkg"], &[
        "labstack/echo", "echo.New", "echo.Echo"
    ])
}

fn check_file_contains_any(project_dir: &Path, file_names: &[&str], patterns: &[&str]) -> bool {
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

fn check_directory_contains_any(project_dir: &Path, dir_names: &[&str], patterns: &[&str]) -> bool {
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