use assert_cmd::Command;
use serde_yaml::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use pretty_assertions::assert_eq;

/// Helper to load test fixtures
fn load_fixture(name: &str) -> String {
    let fixture_path = Path::new("tests/fixtures").join(name);
    std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to load fixture: {}", fixture_path.display()))
}

/// Helper to create temporary file in test directory
fn create_test_file(temp_dir: &TempDir, filename: &str, content: &str) -> String {
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, content).expect("Failed to create test file");
    file_path.to_string_lossy().to_string()
}

/// Helper to parse generated YAML
fn parse_yaml(yaml: &str) -> Value {
    serde_yaml::from_str(yaml)
        .unwrap_or_else(|e| panic!("Failed to parse YAML: {}", e))
}

#[test]
fn test_build_args_basic_cli() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create test file
    let fixture_content = load_fixture("build_args_basic.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    // Run athena build
    let mut cmd = Command::cargo_bin("athena").unwrap();
    let output = cmd
        .arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    
    // Should succeed
    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Command failed with status: {}", output.status);
    }
    
    // Check that docker-compose.yml was generated
    let compose_file = temp_dir.path().join("docker-compose.yml");
    assert!(compose_file.exists(), "docker-compose.yml should be generated");
    
    // Verify content
    let yaml_content = fs::read_to_string(&compose_file).unwrap();
    let parsed = parse_yaml(&yaml_content);
    
    // Check service has build configuration with args
    let services = parsed["services"].as_mapping().unwrap();
    let api_service = services.get(&Value::String("api".to_string())).unwrap();
    
    let build_config = api_service.get("build").expect("Should have build config");
    let args = build_config.get("args").expect("Should have args");
    let args_map = args.as_mapping().unwrap();
    
    assert_eq!(args_map.get(&Value::String("NODE_ENV".to_string())),
               Some(&Value::String("production".to_string())));
    assert_eq!(args_map.get(&Value::String("PORT".to_string())),
               Some(&Value::String("3000".to_string())));
}

#[test]
fn test_build_args_with_dockerfile_validation_success() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create a matching Dockerfile
    let dockerfile_content = r#"
FROM node:18-alpine

ARG NODE_ENV=production
ARG PORT=3000

WORKDIR /app
CMD ["npm", "start"]
    "#;
    create_test_file(&temp_dir, "Dockerfile", dockerfile_content);
    
    // Create athena file with matching build args
    let fixture_content = load_fixture("build_args_basic.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    // Should succeed with validation
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Verify docker-compose.yml was generated
    let compose_file = temp_dir.path().join("docker-compose.yml");
    assert!(compose_file.exists());
    
    let yaml_content = fs::read_to_string(&compose_file).unwrap();
    assert!(yaml_content.contains("NODE_ENV: production"));
    assert!(yaml_content.contains("PORT: '3000'"));
}

#[test]
fn test_build_args_with_dockerfile_validation_failure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create Dockerfile with specific ARGs
    let dockerfile_content = r#"
FROM node:18-alpine
ARG NODE_ENV=production
ARG PORT=3000
    "#;
    create_test_file(&temp_dir, "Dockerfile", dockerfile_content);
    
    // Create athena file with mismatched build args
    let fixture_content = load_fixture("build_args_invalid.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    // Should fail validation
    let mut cmd = Command::cargo_bin("athena").unwrap();
    let output = cmd
        .arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success(), "Should fail with invalid build args");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("BUILD-ARGS validation failed"));
    assert!(stderr.contains("INVALID_ARG"));
    assert!(stderr.contains("Available ARGs"));
}

#[test]
fn test_build_args_without_dockerfile_succeeds() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // No Dockerfile created - validation should be skipped
    let fixture_content = load_fixture("build_args_basic.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    // Should succeed (validation skipped)
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Should still generate docker-compose.yml
    let compose_file = temp_dir.path().join("docker-compose.yml");
    assert!(compose_file.exists());
}

#[test]
fn test_build_args_multiple_services() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    let fixture_content = load_fixture("build_args_multiple_services.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Verify docker-compose.yml content
    let compose_file = temp_dir.path().join("docker-compose.yml");
    let yaml_content = fs::read_to_string(&compose_file).unwrap();
    let parsed = parse_yaml(&yaml_content);
    
    let services = parsed["services"].as_mapping().unwrap();
    
    // Check frontend service
    let frontend = services.get(&Value::String("frontend".to_string())).unwrap();
    let frontend_build = frontend.get("build").expect("Frontend should have build");
    assert!(frontend_build.get("args").is_some());
    
    // Check api service
    let api = services.get(&Value::String("api".to_string())).unwrap(); 
    let api_build = api.get("build").expect("API should have build");
    assert!(api_build.get("args").is_some());
    
    // Check database service (should use image)
    let database = services.get(&Value::String("database".to_string())).unwrap();
    assert!(database.get("build").is_none());
    assert_eq!(database.get("image").unwrap(), "postgres:15");
}

#[test]
fn test_build_args_with_image_precedence() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    let fixture_content = load_fixture("build_args_with_image.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Verify that build takes precedence over image when BUILD-ARGS present
    let compose_file = temp_dir.path().join("docker-compose.yml");
    let yaml_content = fs::read_to_string(&compose_file).unwrap();
    let parsed = parse_yaml(&yaml_content);
    
    let services = parsed["services"].as_mapping().unwrap();
    let api = services.get(&Value::String("api".to_string())).unwrap();
    
    // Should use build config, not image
    assert!(api.get("build").is_some());
    assert!(api.get("image").is_none());
    
    let build_config = api.get("build").unwrap();
    let args = build_config.get("args").unwrap();
    assert!(args.as_mapping().unwrap().contains_key(&Value::String("NODE_ENV".to_string())));
}

#[test]
fn test_build_args_complex_scenario() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    let fixture_content = load_fixture("build_args_complex.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Verify comprehensive configuration
    let compose_file = temp_dir.path().join("docker-compose.yml");
    let yaml_content = fs::read_to_string(&compose_file).unwrap();
    let parsed = parse_yaml(&yaml_content);
    
    // Check project name and network
    assert_eq!(parsed["name"], "BUILD_ARGS_COMPLEX");
    
    let networks = parsed["networks"].as_mapping().unwrap();
    assert!(networks.contains_key(&Value::String("custom_network".to_string())));
    
    // Check all services are present
    let services = parsed["services"].as_mapping().unwrap();
    assert!(services.contains_key(&Value::String("web_server".to_string())));
    assert!(services.contains_key(&Value::String("app".to_string())));
    assert!(services.contains_key(&Value::String("redis".to_string())));
    assert!(services.contains_key(&Value::String("cache".to_string())));
    
    // Check web_server has multiple build args
    let web_server = services.get(&Value::String("web_server".to_string())).unwrap();
    let web_build = web_server.get("build").unwrap();
    let web_args = web_build.get("args").unwrap().as_mapping().unwrap();
    assert_eq!(web_args.len(), 3);
}

#[test]
fn test_validate_command_with_build_args() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    let fixture_content = load_fixture("build_args_basic.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    // Test validate command
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("validate")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();
}

#[test]
fn test_verbose_output_with_build_args() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    let fixture_content = load_fixture("build_args_basic.ath");
    let test_file = create_test_file(&temp_dir, "test.ath", &fixture_content);
    
    // Test with --verbose flag
    let mut cmd = Command::cargo_bin("athena").unwrap();
    let output = cmd
        .arg("--verbose")
        .arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reading Athena file"));
    assert!(stdout.contains("services"));
}

#[test]
fn test_build_args_intelligent_similarity_suggestions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create Dockerfile with ARGs that have common typo patterns
    let dockerfile_content = r#"
FROM node:18-alpine

# Common ARG names that people often mistype
ARG NODE_VERSION=18
ARG NODE_ENV=production
ARG API_PORT=3000
ARG DATABASE_URL
ARG BUILD_VERSION=1.0.0
ARG APP_NAME="MyApp"

WORKDIR /app
CMD ["npm", "start"]
    "#;
    create_test_file(&temp_dir, "Dockerfile", dockerfile_content);
    
    // Create .ath file with common typos that should trigger smart suggestions
    let ath_content = r#"
DEPLOYMENT-ID SIMILARITY_TEST
VERSION-ID 1.0.0

SERVICES SECTION

SERVICE api
BUILD-ARGS NODEJS_VERSION="20" NODE_ENVIRONMENT="dev" API_URL="http://test" DB_URL="postgres://test"
PORT-MAPPING 3000 TO 3000
END SERVICE
    "#;
    let test_file = create_test_file(&temp_dir, "test.ath", &ath_content);
    
    // Should fail with intelligent suggestions
    let mut cmd = Command::cargo_bin("athena").unwrap();
    let output = cmd
        .arg("build")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success(), "Should fail with invalid build args");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Verify intelligent error reporting
    assert!(stderr.contains("BUILD-ARGS validation failed"), "Should contain validation error");
    assert!(stderr.contains("NODEJS_VERSION"), "Should mention the typo");
    assert!(stderr.contains("NODE_ENVIRONMENT"), "Should mention the typo");
    assert!(stderr.contains("Available ARGs"), "Should list available ARGs");
    
    // Check that all the actual ARGs are listed
    assert!(stderr.contains("NODE_VERSION"), "Should list correct ARG");
    assert!(stderr.contains("NODE_ENV"), "Should list correct ARG");  
    assert!(stderr.contains("API_PORT"), "Should list correct ARG");
    assert!(stderr.contains("DATABASE_URL"), "Should list correct ARG");
    assert!(stderr.contains("BUILD_VERSION"), "Should list correct ARG");
    assert!(stderr.contains("APP_NAME"), "Should list correct ARG");
    
    // Verify specific services are mentioned
    assert!(stderr.contains("api"), "Should mention the failing service");
    
    // Check suggestions are provided
    assert!(stderr.contains("Suggestion:"), "Should provide helpful suggestions");
    assert!(stderr.contains("correspond to ARG declarations"), "Should explain the issue");
    
    // Verify that the error is comprehensive and actionable
    let error_lines: Vec<&str> = stderr.lines().collect();
    let error_content = error_lines.join(" ");
    
    // Should mention multiple invalid args
    assert!(error_content.contains("NODEJS_VERSION") && error_content.contains("NODE_ENVIRONMENT"),
            "Should mention multiple invalid args in the same error");
    
    // Verify the error format is user-friendly
    assert!(stderr.contains("Error:"), "Should start with Error:");
    
    println!("Full error output for verification:\n{}", stderr);
}