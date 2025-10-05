use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_single_line_comments() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let content = r#"
// This is a single-line comment
DEPLOYMENT-ID COMMENT_TEST

SERVICES SECTION

// Comment before service
SERVICE test_service
IMAGE-ID "nginx:alpine"  // Inline comment
PORT-MAPPING 80 TO 80    // Port comment
END SERVICE
// Comment after service
"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "test.ath", content);
    let yaml = run_athena_build_and_parse(&ath_file).expect("Failed to parse YAML");
    
    // Verify the structure is correct despite comments
    assert!(yaml["services"].is_mapping());
    assert!(yaml["services"]["test_service"].is_mapping());
    assert_eq!(yaml["services"]["test_service"]["image"].as_str().unwrap(), "nginx:alpine");
    assert_eq!(yaml["services"]["test_service"]["ports"][0].as_str().unwrap(), "80:80");
}

#[test]
fn test_multi_line_comments() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let content = r#"
/*
 * Multi-line comment at the top
 * describing the configuration
 */
DEPLOYMENT-ID MULTILINE_TEST

SERVICES SECTION

SERVICE api
/*
 * This is the API service
 * that handles all requests
 */
IMAGE-ID "python:3.11"
PORT-MAPPING 8000 TO 8000
/*
 * Environment variables for the API
 */
ENV-VARIABLE {{API_KEY}}
END SERVICE
"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "test.ath", content);
    let yaml = run_athena_build_and_parse(&ath_file).expect("Failed to parse YAML");
    
    // Verify the structure is correct despite multi-line comments
    assert!(yaml["services"]["api"].is_mapping());
    assert_eq!(yaml["services"]["api"]["image"].as_str().unwrap(), "python:3.11");
    assert_eq!(yaml["services"]["api"]["ports"][0].as_str().unwrap(), "8000:8000");
    assert_eq!(yaml["services"]["api"]["environment"][0].as_str().unwrap(), "API_KEY=${API_KEY}");
}

#[test]
fn test_mixed_comments() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let content = r#"
// Single line comment
DEPLOYMENT-ID MIXED_COMMENTS
/* Block comment */

ENVIRONMENT SECTION
NETWORK-NAME test_network  // Inline comment

SERVICES SECTION

SERVICE web
IMAGE-ID "nginx:alpine" /* inline block comment */
PORT-MAPPING 80 TO 80
/*
 * Dependencies section
 */
DEPENDS-ON api // dependency comment
END SERVICE

// API service with mixed comments
SERVICE api
IMAGE-ID "python:3.11"
/* Multi-line
   comment inside
   service block */
PORT-MAPPING 8000 TO 8000
ENV-VARIABLE {{DATABASE_URL}} // Database URL
RESTART-POLICY always /* always restart */
END SERVICE
"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "test.ath", content);
    let yaml = run_athena_build_and_parse(&ath_file).expect("Failed to parse YAML");
    
    // Verify both services are present and configured correctly
    assert!(yaml["services"]["web"].is_mapping());
    assert!(yaml["services"]["api"].is_mapping());
    assert_eq!(yaml["services"]["web"]["depends_on"][0].as_str().unwrap(), "api");
    assert_eq!(yaml["services"]["api"]["restart"].as_str().unwrap(), "always");
    assert_eq!(yaml["networks"]["test_network"]["driver"].as_str().unwrap(), "bridge");
}

#[test]
fn test_comments_with_complex_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Use the fixture file we created
    let fixture_content = fs::read_to_string("tests/fixtures/comments_test.ath")
        .expect("Failed to read comments test fixture");
    
    let ath_file = create_test_ath_file(&temp_dir, "test.ath", &fixture_content);
    let yaml = run_athena_build_and_parse(&ath_file).expect("Failed to parse YAML");
    
    // Verify complex structure with comments
    assert!(yaml["services"]["web"].is_mapping());
    assert!(yaml["services"]["api"].is_mapping());
    assert_eq!(yaml["services"]["web"]["depends_on"][0].as_str().unwrap(), "api");
    assert_eq!(yaml["services"]["web"]["restart"].as_str().unwrap(), "unless-stopped");
    assert_eq!(yaml["services"]["api"]["restart"].as_str().unwrap(), "always");
    assert_eq!(yaml["networks"]["comments_network"]["driver"].as_str().unwrap(), "bridge");
}

#[test]
fn test_comment_edge_cases() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let content = r#"
// Comment with special characters: !@#$%^&*()
DEPLOYMENT-ID EDGE_CASE_TEST

SERVICES SECTION

SERVICE test
// Comment with quotes: "hello" and 'world'
IMAGE-ID "nginx:alpine"
/* Comment with slashes and other symbols */
PORT-MAPPING 80 TO 80
END SERVICE

// Final comment with unicode: café, résumé, naïve
"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "test.ath", content);
    let yaml = run_athena_build_and_parse(&ath_file).expect("Failed to parse YAML");
    
    // Verify parsing works with edge case comments
    assert!(yaml["services"]["test"].is_mapping());
    assert_eq!(yaml["services"]["test"]["image"].as_str().unwrap(), "nginx:alpine");
}

#[test]
fn test_unclosed_comment_error() {
    use assert_cmd::Command;
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let content = r#"
DEPLOYMENT-ID BROKEN_TEST

SERVICES SECTION

SERVICE test
/* Unclosed comment
IMAGE-ID "nginx:alpine"
END SERVICE
"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "test.ath", content);
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    let result = cmd.arg("validate")
        .arg(&ath_file)
        .output()
        .expect("Failed to execute command");
    
    // Should fail with specific comment error
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("Unclosed multi-line comment"));
    assert!(stderr.contains("Multi-line comments must be closed with '*/'"));
}