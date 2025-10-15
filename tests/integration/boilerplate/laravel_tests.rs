use super::*;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn test_laravel_init_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_laravel_basic";
    
    let mut cmd = run_init_command("laravel", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Laravel project"))
        .stdout(predicate::str::contains(project_name));

    // Verify project structure was created
    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for Laravel files that we know exist
    let expected_files = &[
        "composer.json",
        "docker-compose.yml",
        ".env.docker.example",
        "README.md",
        "docker/php/Dockerfile",
        "docker/nginx/Dockerfile",
        "app/Domain/User/Entities/User.php",
        "app/Application/User/Commands/CreateUserCommand.php",
        "app/Infrastructure/Http/Controllers/Api/V1/AuthController.php",
    ];

    for file in expected_files {
        assert!(project_dir.join(file).exists(), "{} should exist", file);
    }
}

#[test]
#[serial]
fn test_laravel_docker_compose_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_laravel_docker";
    
    let mut cmd = run_init_command("laravel", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check docker-compose.yml contains production-ready configuration
    let docker_compose_path = project_dir.join("docker-compose.yml");
    assert!(docker_compose_path.exists(), "docker-compose.yml should exist");
    
    let docker_compose_content = fs::read_to_string(&docker_compose_path)
        .expect("Should be able to read docker-compose.yml");
    
    // Check for production-ready features
    assert!(docker_compose_content.contains("env_file:"), "Should use env_file for security");
    assert!(docker_compose_content.contains("expose:"), "Should use expose instead of ports for internal services");
    assert!(docker_compose_content.contains("healthcheck:"), "Should have health checks");
    assert!(docker_compose_content.contains("depends_on:"), "Should have service dependencies");
    assert!(docker_compose_content.contains("restart: unless-stopped"), "Should have restart policy");
}

#[test]
#[serial]
fn test_laravel_clean_architecture_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_laravel_architecture";
    
    let mut cmd = run_init_command("laravel", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check for Clean Architecture directories
    let architecture_dirs = &[
        "app/Domain",
        "app/Domain/User",
        "app/Domain/User/Entities",
        "app/Domain/User/Repositories",
        "app/Domain/User/Services",
        "app/Application",
        "app/Application/User/Commands",
        "app/Application/User/Queries",
        "app/Application/User/Handlers",
        "app/Infrastructure",
        "app/Infrastructure/Http/Controllers",
        "app/Infrastructure/Persistence/Eloquent",
    ];

    for dir in architecture_dirs {
        assert!(project_dir.join(dir).exists(), "{} directory should exist", dir);
    }
}

#[test]
#[serial]
fn test_laravel_jwt_authentication() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_laravel_jwt";
    
    let mut cmd = run_init_command("laravel", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check composer.json contains JWT dependency
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("tymon/jwt-auth"), "Should include JWT authentication package");
    
    // Check for JWT-related files
    assert!(project_dir.join("app/Infrastructure/Http/Controllers/Api/V1/AuthController.php").exists(),
            "AuthController should exist");
    
    let auth_controller_path = project_dir.join("app/Infrastructure/Http/Controllers/Api/V1/AuthController.php");
    let auth_content = fs::read_to_string(&auth_controller_path)
        .expect("Should be able to read AuthController");
    
    assert!(auth_content.contains("login"), "AuthController should have login method");
    assert!(auth_content.contains("register"), "AuthController should have register method");
    assert!(auth_content.contains("logout"), "AuthController should have logout method");
}

#[test]
#[serial]
fn test_laravel_environment_security() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_laravel_security";
    
    let mut cmd = run_init_command("laravel", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check .env.docker.example exists with secure variables
    let env_example_path = project_dir.join(".env.docker.example");
    assert!(env_example_path.exists(), ".env.docker.example should exist");
    
    let env_content = fs::read_to_string(&env_example_path)
        .expect("Should be able to read .env.docker.example");
    
    // Should contain variable templates, not actual secrets
    assert!(env_content.contains("APP_KEY="), "Should have APP_KEY template");
    assert!(env_content.contains("DB_PASSWORD="), "Should have DB_PASSWORD template");
    assert!(env_content.contains("JWT_SECRET="), "Should have JWT_SECRET template");
    assert!(env_content.contains("openssl rand"), "Should use openssl for secret generation");
}

#[test]
#[serial]
fn test_laravel_testing_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_laravel_testing";
    
    let mut cmd = run_init_command("laravel", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check for testing configuration
    assert!(project_dir.join("phpunit.xml").exists(), "phpunit.xml should exist");
    
    // Check for test directories
    let test_dirs = &[
        "tests/Unit",
        "tests/Feature", 
        "tests/Integration",
    ];

    for dir in test_dirs {
        assert!(project_dir.join(dir).exists(), "{} directory should exist", dir);
    }
    
    // Check composer.json contains testing dependencies
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("phpunit/phpunit"), "Should include PHPUnit");
}

#[test]
#[serial]
fn test_laravel_no_docker() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_laravel_no_docker";
    
    let mut cmd = run_init_command("laravel", project_name, &["--no-docker"]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Laravel project"));

    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");
    
    // Docker files should NOT exist
    assert!(!project_dir.join("docker-compose.yml").exists(), 
            "docker-compose.yml should not exist with --no-docker");
    assert!(!project_dir.join("docker/php/Dockerfile").exists(), 
            "Dockerfile should not exist with --no-docker");
    assert!(!project_dir.join(".env.docker.example").exists(), 
            ".env.docker.example should not exist with --no-docker");
            
    // But regular Laravel files should exist
    assert!(project_dir.join("composer.json").exists(), "composer.json should exist");
}

#[test]
fn test_laravel_init_help() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init").arg("laravel").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Laravel"))
        .stdout(predicate::str::contains("--no-docker"));
}