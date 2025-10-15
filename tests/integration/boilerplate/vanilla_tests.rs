use super::*;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn test_vanilla_init_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_basic";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PHP Vanilla project"))
        .stdout(predicate::str::contains(project_name));

    // Verify project structure was created
    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for PHP Vanilla files that we know exist
    let expected_files = &[
        "composer.json",
        "docker-compose.yml",
        ".env.docker.example",
        ".env.example",
        "README.md",
        "public/index.php",
        "public/.htaccess",
        "src/Domain/User/Entity/User.php",
        "src/Application/User/Command/CreateUserCommand.php",
        "src/Infrastructure/Http/Controller/Api/V1/AuthController.php",
        "src/Infrastructure/Http/Router.php",
        "src/Infrastructure/Database/PDOConnection.php",
        "src/Infrastructure/Security/JWTManager.php",
        "src/Infrastructure/Config/AppConfig.php",
    ];

    for file in expected_files {
        assert!(project_dir.join(file).exists(), "{} should exist", file);
    }
}

#[test]
#[serial]
fn test_vanilla_docker_compose_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_docker";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
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
fn test_vanilla_clean_architecture_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_architecture";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check for Clean Architecture directories
    let architecture_dirs = &[
        "src/Domain",
        "src/Domain/User",
        "src/Domain/User/Entity",
        "src/Domain/User/Repository",
        "src/Domain/User/Service",
        "src/Domain/User/ValueObject",
        "src/Application",
        "src/Application/User/Command",
        "src/Application/User/Handler",
        "src/Application/Auth",
        "src/Infrastructure",
        "src/Infrastructure/Http/Controller",
        "src/Infrastructure/Persistence/PDO",
        "src/Infrastructure/Security",
        "src/Infrastructure/Config",
    ];

    for dir in architecture_dirs {
        assert!(project_dir.join(dir).exists(), "{} directory should exist", dir);
    }
}

#[test]
#[serial]
fn test_vanilla_jwt_authentication() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_jwt";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check composer.json contains JWT dependency
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("firebase/php-jwt"), "Should include JWT library");
    
    // Check for JWT-related files
    assert!(project_dir.join("src/Infrastructure/Security/JWTManager.php").exists(),
            "JWTManager should exist");
    assert!(project_dir.join("src/Infrastructure/Http/Controller/Api/V1/AuthController.php").exists(),
            "AuthController should exist");
}

#[test]
#[serial]
fn test_vanilla_pdo_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_database";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check for database configuration
    assert!(project_dir.join("config/database.php").exists(), "Database config should exist");
    assert!(project_dir.join("src/Infrastructure/Database/PDOConnection.php").exists(), 
            "PDO connection should exist");
    assert!(project_dir.join("database/migrations/001_create_users_table.sql").exists(),
            "Database migration should exist");
    
    // Check database config content
    let db_config_path = project_dir.join("config/database.php");
    let db_config_content = fs::read_to_string(&db_config_path)
        .expect("Should be able to read database config");
    
    assert!(db_config_content.contains("pgsql"), "Should support PostgreSQL");
    assert!(db_config_content.contains("mysql"), "Should support MySQL");
}

#[test]
#[serial]
fn test_vanilla_environment_security() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_security";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check .env.example exists with secure variables
    let env_example_path = project_dir.join(".env.example");
    assert!(env_example_path.exists(), ".env.example should exist");
    
    let env_content = fs::read_to_string(&env_example_path)
        .expect("Should be able to read .env.example");
    
    // Should contain environment variable templates
    assert!(env_content.contains("APP_NAME="), "Should have APP_NAME template");
    assert!(env_content.contains("DB_PASSWORD="), "Should have DB_PASSWORD template");
    assert!(env_content.contains("JWT_SECRET="), "Should have JWT_SECRET template");
    assert!(env_content.contains("BCRYPT_ROUNDS="), "Should have BCRYPT_ROUNDS setting");
}

#[test]
#[serial]
fn test_vanilla_testing_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_testing";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check for testing configuration
    assert!(project_dir.join("phpunit.xml").exists(), "phpunit.xml should exist");
    
    // Check composer.json contains testing dependencies
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("phpunit/phpunit"), "Should include PHPUnit");
    assert!(composer_content.contains("phpstan/phpstan"), "Should include PHPStan");
    assert!(composer_content.contains("friendsofphp/php-cs-fixer"), "Should include PHP CS Fixer");
    
    // Check for test directories
    let test_dirs = &[
        "tests/Unit",
        "tests/Integration", 
        "tests/Functional",
    ];

    for dir in test_dirs {
        assert!(project_dir.join(dir).exists(), "{} directory should exist", dir);
    }
    
    // Check for test files
    assert!(project_dir.join("tests/Unit/UserTest.php").exists(), "Unit test should exist");
    assert!(project_dir.join("tests/Functional/AuthTest.php").exists(), "Functional test should exist");
}

#[test]
#[serial]
fn test_vanilla_no_docker() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_no_docker";
    
    let mut cmd = run_init_command("vanilla", project_name, &["--no-docker"]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PHP Vanilla project"));

    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");
    
    // Docker files should NOT exist
    assert!(!project_dir.join("docker-compose.yml").exists(), 
            "docker-compose.yml should not exist with --no-docker");
    assert!(!project_dir.join("docker/php/Dockerfile").exists(), 
            "Dockerfile should not exist with --no-docker");
    assert!(!project_dir.join(".env.docker.example").exists(), 
            ".env.docker.example should not exist with --no-docker");
            
    // But regular PHP files should exist
    assert!(project_dir.join("composer.json").exists(), "composer.json should exist");
    assert!(project_dir.join("public/index.php").exists(), "index.php should exist");
}

#[test]
#[serial]
fn test_vanilla_api_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_api";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check for API structure
    assert!(project_dir.join("src/Infrastructure/Http/Router.php").exists(), "Router should exist");
    assert!(project_dir.join("src/Infrastructure/Http/Request.php").exists(), "Request class should exist");
    assert!(project_dir.join("src/Infrastructure/Http/Response.php").exists(), "Response class should exist");
    
    // Check public/index.php has API routes defined
    let index_php_path = project_dir.join("public/index.php");
    let index_content = fs::read_to_string(&index_php_path)
        .expect("Should be able to read index.php");
    
    assert!(index_content.contains("/api/v1/health"), "Should have health endpoint");
    assert!(index_content.contains("/api/v1/auth/register"), "Should have register endpoint");
    assert!(index_content.contains("/api/v1/auth/login"), "Should have login endpoint");
    assert!(index_content.contains("/api/v1/users"), "Should have users endpoint");
}

#[test]
#[serial]
fn test_vanilla_psr4_autoloading() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_vanilla_psr4";
    
    let mut cmd = run_init_command("vanilla", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check composer.json contains PSR-4 autoloading
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("\"App\\\\\""), "Should have PSR-4 autoloading for App namespace");
    assert!(composer_content.contains("\"Tests\\\\\""), "Should have PSR-4 autoloading for Tests namespace");
}

#[test]
fn test_vanilla_init_help() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init").arg("vanilla").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PHP Vanilla"))
        .stdout(predicate::str::contains("--no-docker"));
}