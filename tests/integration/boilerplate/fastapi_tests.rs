use super::*;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn test_fastapi_init_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_fastapi_basic";
    
    let mut cmd = run_init_command("fastapi", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"))
        .stdout(predicate::str::contains(project_name));

    // Verify project structure was created
    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for FastAPI files that we know exist
    assert!(project_dir.join("requirements.txt").exists(), "requirements.txt should exist");
    assert!(project_dir.join("Dockerfile").exists(), "Dockerfile should exist");
    assert!(project_dir.join("docker-compose.yml").exists(), "docker-compose.yml should exist");
    assert!(project_dir.join(".env.example").exists(), ".env.example should exist");
    
    // Check for app directory structure
    assert!(project_dir.join("app").exists(), "app directory should exist");
}

#[test]
#[serial]
fn test_fastapi_init_with_postgresql() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_fastapi_postgres";
    
    let mut cmd = run_init_command("fastapi", project_name, &["--with-postgresql"]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"));

    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for PostgreSQL-specific files/configuration
    let has_postgres_config = check_for_postgres_configuration(&project_dir);
    assert!(has_postgres_config, "PostgreSQL configuration should be present");
}

#[test]
#[serial]
fn test_fastapi_init_with_mongodb() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_fastapi_mongo";
    
    let mut cmd = run_init_command("fastapi", project_name, &["--with-mongodb"]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"));

    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for MongoDB-specific files/configuration
    let has_mongo_config = check_for_mongo_configuration(&project_dir);
    assert!(has_mongo_config, "MongoDB configuration should be present");
}

#[test]
#[serial]
fn test_fastapi_init_no_docker() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_fastapi_no_docker";
    
    let mut cmd = run_init_command("fastapi", project_name, &["--no-docker"]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"));

    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");
    
    // Dockerfile should NOT exist
    assert!(!project_dir.join("Dockerfile").exists(), 
            "Dockerfile should not exist with --no-docker");
    assert!(!project_dir.join("docker-compose.yml").exists(), 
            "docker-compose.yml should not exist with --no-docker");
}

#[test]
#[serial] 
fn test_fastapi_custom_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "custom_name";
    let custom_dir = "custom_directory_path";
    
    let mut cmd = run_init_command("fastapi", project_name, &["--directory", custom_dir]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("FastAPI project"));

    // Project should be created in custom directory, not project name
    let project_dir = temp_dir.path().join(custom_dir);
    assert!(project_dir.exists(), "Custom directory should be created");
    assert!(!temp_dir.path().join(project_name).exists(), 
            "Project name directory should not exist");
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