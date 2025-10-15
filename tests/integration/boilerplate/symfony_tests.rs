use super::*;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn test_symfony_init_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_basic";
    
    let mut cmd = run_init_command("symfony", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Symfony project"))
        .stdout(predicate::str::contains(project_name));

    // Verify project structure was created
    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for Symfony files that we know exist
    let expected_files = &[
        "composer.json",
        "docker-compose.yml",
        ".env.docker.example",
        "README.md",
        "docker/php/Dockerfile",
        "docker/nginx/Dockerfile",
        "src/Domain/User/Entity/User.php",
        "src/Application/User/Command/CreateUserCommand.php",
        "src/Infrastructure/Http/Controller/Api/V1/AuthController.php",
    ];

    for file in expected_files {
        assert!(project_dir.join(file).exists(), "{} should exist", file);
    }
}

#[test]
#[serial]
fn test_symfony_docker_compose_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_docker";
    
    let mut cmd = run_init_command("symfony", project_name, &[]);
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
fn test_symfony_hexagonal_architecture_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_architecture";
    
    let mut cmd = run_init_command("symfony", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check for Hexagonal Architecture directories
    let architecture_dirs = &[
        "src/Domain",
        "src/Domain/User",
        "src/Domain/User/Entity",
        "src/Domain/User/Repository",
        "src/Domain/User/Service",
        "src/Application",
        "src/Application/User/Command",
        "src/Application/User/Query",
        "src/Application/User/Handler",
        "src/Infrastructure",
        "src/Infrastructure/Http/Controller",
        "src/Infrastructure/Persistence/Doctrine",
    ];

    for dir in architecture_dirs {
        assert!(project_dir.join(dir).exists(), "{} directory should exist", dir);
    }
}

#[test]
#[serial]
fn test_symfony_jwt_authentication() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_jwt";
    
    let mut cmd = run_init_command("symfony", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check composer.json contains JWT dependencies
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("lexik/jwt-authentication-bundle"), 
            "Should include JWT authentication bundle");
    assert!(composer_content.contains("gesdinet/jwt-refresh-token-bundle"), 
            "Should include JWT refresh token bundle");
    
    // Check for JWT-related files
    assert!(project_dir.join("src/Infrastructure/Http/Controller/Api/V1/AuthController.php").exists(),
            "AuthController should exist");
}

#[test]
#[serial]
fn test_symfony_doctrine_configuration() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_doctrine";
    
    let mut cmd = run_init_command("symfony", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check composer.json contains Doctrine dependencies
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("doctrine/orm"), "Should include Doctrine ORM");
    assert!(composer_content.contains("doctrine/doctrine-bundle"), "Should include Doctrine bundle");
    assert!(composer_content.contains("doctrine/doctrine-migrations-bundle"), 
            "Should include Doctrine migrations");
    
    // Check for Doctrine configuration files
    assert!(project_dir.join("config/packages/doctrine.yaml").exists(),
            "Doctrine configuration should exist");
}

#[test]
#[serial]
fn test_symfony_environment_security() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_security";
    
    let mut cmd = run_init_command("symfony", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check .env.docker.example exists with secure variables
    let env_example_path = project_dir.join(".env.docker.example");
    assert!(env_example_path.exists(), ".env.docker.example should exist");
    
    let env_content = fs::read_to_string(&env_example_path)
        .expect("Should be able to read .env.docker.example");
    
    // Should contain variable templates, not actual secrets
    assert!(env_content.contains("APP_SECRET="), "Should have APP_SECRET template");
    assert!(env_content.contains("DB_PASSWORD="), "Should have DB_PASSWORD template");
    assert!(env_content.contains("JWT_SECRET="), "Should have JWT_SECRET template");
    assert!(env_content.contains("openssl rand"), "Should use openssl for secret generation");
}

#[test]
#[serial]
fn test_symfony_testing_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_testing";
    
    let mut cmd = run_init_command("symfony", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check for testing configuration
    assert!(project_dir.join("phpunit.xml.dist").exists(), "phpunit.xml.dist should exist");
    
    // Check composer.json contains testing dependencies
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("phpunit/phpunit"), "Should include PHPUnit");
    assert!(composer_content.contains("symfony/phpunit-bridge"), "Should include Symfony PHPUnit bridge");
    assert!(composer_content.contains("symfony/test-pack"), "Should include Symfony test pack");
    
    // Check for test directories
    let test_dirs = &[
        "tests/Unit",
        "tests/Functional",
    ];

    for dir in test_dirs {
        assert!(project_dir.join(dir).exists(), "{} directory should exist", dir);
    }
    
    // Should have testing scripts in composer.json
    assert!(composer_content.contains("\"test\""), "Should have test script");
}

#[test]
#[serial]
fn test_symfony_no_docker() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_no_docker";
    
    let mut cmd = run_init_command("symfony", project_name, &["--no-docker"]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Symfony project"));

    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");
    
    // Docker files should NOT exist
    assert!(!project_dir.join("docker-compose.yml").exists(), 
            "docker-compose.yml should not exist with --no-docker");
    assert!(!project_dir.join("docker/php/Dockerfile").exists(), 
            "Dockerfile should not exist with --no-docker");
    assert!(!project_dir.join(".env.docker.example").exists(), 
            ".env.docker.example should not exist with --no-docker");
            
    // But regular Symfony files should exist
    assert!(project_dir.join("composer.json").exists(), "composer.json should exist");
}

#[test]
#[serial]
fn test_symfony_api_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_symfony_api";
    
    let mut cmd = run_init_command("symfony", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert().success();

    let project_dir = temp_dir.path().join(project_name);
    
    // Check composer.json contains API-specific dependencies
    let composer_path = project_dir.join("composer.json");
    let composer_content = fs::read_to_string(&composer_path)
        .expect("Should be able to read composer.json");
    
    assert!(composer_content.contains("symfony/serializer"), "Should include Serializer component");
    assert!(composer_content.contains("symfony/validator"), "Should include Validator component");
    assert!(composer_content.contains("nelmio/cors-bundle"), "Should include CORS bundle");
}

#[test]
fn test_symfony_init_help() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init").arg("symfony").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Symfony"))
        .stdout(predicate::str::contains("--no-docker"));
}