use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use std::path::Path;

fn create_test_ath_file(temp_dir: &TempDir, filename: &str, content: &str) -> String {
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, content).expect("Failed to create test file");
    file_path.to_string_lossy().to_string()
}

#[test]
fn test_cli_build_command_with_valid_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "test.ath",
        r#"DEPLOYMENT-ID TEST_APP
VERSION-ID 1.0.0

ENVIRONMENT SECTION
NETWORK-NAME test_network

SERVICES SECTION

SERVICE web
IMAGE-ID nginx:alpine
PORT-MAPPING 8080 TO 80
ENV-VARIABLE {{NGINX_HOST}}
END SERVICE"#,
    );
    
    let output_file = temp_dir.path().join("docker-compose.yml");
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build")
        .arg(&ath_file)
        .arg("-o")
        .arg(&output_file)
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated docker-compose.yml"));
        
    // Verify output file was created
    assert!(output_file.exists(), "Output file should be created");
    
    // Verify output contains expected content
    let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert!(output_content.contains("version:"), "Should contain Docker Compose version");
    assert!(output_content.contains("services:"), "Should contain services section");
    assert!(output_content.contains("web:"), "Should contain web service");
    assert!(output_content.contains("nginx:alpine"), "Should contain correct image");
}

#[test]
fn test_cli_build_command_validate_only() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "test.ath",
        r#"DEPLOYMENT-ID VALIDATE_TEST
SERVICES SECTION

SERVICE test_service
IMAGE-ID alpine:latest
COMMAND "echo 'test'"
END SERVICE"#,
    );
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build")
        .arg(&ath_file)
        .arg("--validate-only")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Athena file is valid"))
        .stdout(predicate::str::contains("Generated docker-compose.yml").not());
}

#[test]
fn test_cli_validate_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "test.ath",
        include_str!("../fixtures/valid_simple.ath"),
    );
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("validate")
        .arg(&ath_file)
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Athena file is valid"))
        .stdout(predicate::str::contains("Project name:"))
        .stdout(predicate::str::contains("Services found:"));
}

#[test]
fn test_cli_validate_command_with_invalid_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "invalid.ath",
        include_str!("../fixtures/invalid_syntax.ath"),
    );
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("validate").arg(&ath_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error:"));
}

#[test]
fn test_cli_info_command() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("info");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Athena DSL - Docker Compose Generator"))
        .stdout(predicate::str::contains("Basic structure:"));
}

#[test]
fn test_cli_info_examples() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("info").arg("--examples");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Athena DSL Examples"))
        .stdout(predicate::str::contains("Simple web application"))
        .stdout(predicate::str::contains("DEPLOYMENT-ID"));
}

#[test]
fn test_cli_info_directives() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("info").arg("--directives");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Athena DSL Directives Reference"))
        .stdout(predicate::str::contains("FILE STRUCTURE"))
        .stdout(predicate::str::contains("SERVICE DIRECTIVES"));
}

#[test]
fn test_cli_build_with_missing_file() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build").arg("nonexistent.ath");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error:"));
}

#[test]
fn test_cli_magic_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    
    // Change to temp directory and create a test .ath file
    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");
    let ath_file = temp_dir.path().join("app.ath");
    fs::write(&ath_file, include_str!("../fixtures/minimal_valid.ath"))
        .expect("Failed to create test file");
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated docker-compose.yml"));
        
    // Restore current directory
    std::env::set_current_dir(current_dir).expect("Failed to restore directory");
}

#[test]
fn test_cli_build_quiet_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "test.ath",
        include_str!("../fixtures/minimal_valid.ath"),
    );
    
    let output_file = temp_dir.path().join("docker-compose.yml");
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build")
        .arg(&ath_file)
        .arg("-o")
        .arg(&output_file)
        .arg("--quiet");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated docker-compose.yml"))
        // In quiet mode, should not contain verbose output
        .stdout(predicate::str::contains("Reading Athena file:").not())
        .stdout(predicate::str::contains("Validating syntax...").not());
}

#[test]
fn test_cli_build_with_custom_output_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "test.ath",
        include_str!("../fixtures/minimal_valid.ath"),
    );
    
    let custom_output = temp_dir.path().join("custom-compose.yml");
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build")
        .arg(&ath_file)
        .arg("-o")
        .arg(&custom_output);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("custom-compose.yml"));
        
    // Verify custom output file was created
    assert!(custom_output.exists(), "Custom output file should be created");
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A powerful CLI tool for DSL-based Docker Compose generation"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("info"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("athena"))
        .stdout(predicate::str::contains("0.1.0"));
}