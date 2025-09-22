use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn create_test_ath_file(temp_dir: &TempDir, filename: &str, content: &str) -> String {
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, content).expect("Failed to create test file");
    file_path.to_string_lossy().to_string()
}

#[test]
fn test_enhanced_port_conflict_detection() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let port_conflict_content = r#"DEPLOYMENT-ID enhanced_port_test

SERVICES SECTION

SERVICE app1
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
END SERVICE

SERVICE app2
IMAGE-ID "httpd:alpine"
PORT-MAPPING 8080 TO 8000
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "port_conflict.ath", port_conflict_content);
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build").arg(&ath_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Port conflict detected! Host port 8080 is used by multiple services"))
        .stderr(predicate::str::contains("app1"))
        .stderr(predicate::str::contains("app2"))
        .stderr(predicate::str::contains("Use different host ports"));
}

#[test]
fn test_enhanced_service_reference_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let invalid_reference_content = r#"DEPLOYMENT-ID enhanced_reference_test

SERVICES SECTION

SERVICE frontend
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
DEPENDS-ON nonexistent_backend
END SERVICE

SERVICE database
IMAGE-ID "postgres:15"
PORT-MAPPING 5432 TO 5432
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "invalid_reference.ath", invalid_reference_content);
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build").arg(&ath_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Service 'frontend' depends on 'nonexistent_backend' which doesn't exist"))
        .stderr(predicate::str::contains("Available services"))
        .stderr(predicate::str::contains("frontend"))
        .stderr(predicate::str::contains("database"));
}

#[test]
fn test_enhanced_missing_end_service_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let missing_end_service_content = r#"DEPLOYMENT-ID enhanced_missing_end_test

SERVICES SECTION

SERVICE incomplete_service
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
# Missing END SERVICE statement"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "missing_end_service.ath", missing_end_service_content);
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build").arg(&ath_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Parse error"));
}

#[test]
fn test_enhanced_invalid_port_mapping_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let invalid_port_content = r#"DEPLOYMENT-ID enhanced_port_format_test

SERVICES SECTION

SERVICE invalid_port_service
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 INVALID_FORMAT 80
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "invalid_port_format.ath", invalid_port_content);
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build").arg(&ath_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Parse error"));
}

#[test]
fn test_enhanced_multiple_port_conflicts() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let multiple_conflicts_content = r#"DEPLOYMENT-ID enhanced_multiple_conflicts_test

SERVICES SECTION

SERVICE service1
IMAGE-ID "nginx:alpine"
PORT-MAPPING 3000 TO 80
END SERVICE

SERVICE service2
IMAGE-ID "apache:latest"
PORT-MAPPING 3000 TO 8080
END SERVICE

SERVICE service3
IMAGE-ID "httpd:alpine"
PORT-MAPPING 3000 TO 8000
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "multiple_conflicts.ath", multiple_conflicts_content);
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build").arg(&ath_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Port conflict detected"))
        .stderr(predicate::str::contains("3000"))
        .stderr(predicate::str::contains("Use different host ports"));
}

#[test]
fn test_enhanced_no_conflicts_success() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let no_conflicts_content = r#"DEPLOYMENT-ID enhanced_no_conflicts_test

SERVICES SECTION

SERVICE app1
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
END SERVICE

SERVICE app2
IMAGE-ID "httpd:alpine"
PORT-MAPPING 8081 TO 8000
END SERVICE

SERVICE app3
IMAGE-ID "apache:latest"
PORT-MAPPING 9000 TO 80
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "no_conflicts.ath", no_conflicts_content);
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build").arg(&ath_file);

    // This should succeed without any port conflicts
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated docker-compose.yml"));
}

#[test]
fn test_enhanced_validation_error_suggestions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let content_with_validation_issues = r#"DEPLOYMENT-ID enhanced_validation_test

SERVICES SECTION

SERVICE problematic_service
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
DEPENDS-ON missing_dependency_1
DEPENDS-ON missing_dependency_2
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "validation_issues.ath", content_with_validation_issues);
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("build").arg(&ath_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Service 'problematic_service' depends on"))
        .stderr(predicate::str::contains("which doesn't exist"))
        .stderr(predicate::str::contains("Available services:"));
}