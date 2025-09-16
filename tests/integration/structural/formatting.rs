use super::{create_test_ath_file, run_athena_build_and_parse};
use tempfile::TempDir;
use assert_cmd::Command;
use serde_yaml::Value;

#[test]
fn test_yaml_validity() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "validity.ath",
        include_str!("../../fixtures/valid_complex_microservices.ath"),
    );
    
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    // Test that the YAML can be re-serialized (validates structure)
    let re_serialized = serde_yaml::to_string(&parsed)
        .expect("Should be able to re-serialize YAML");
    
    // Re-parse to ensure consistency
    let re_parsed: Value = serde_yaml::from_str(&re_serialized)
        .expect("Re-serialized YAML should be valid");
    
    // Basic structure should be preserved (modern Docker Compose doesn't need version)
    assert_eq!(parsed["services"].as_mapping().unwrap().len(), 
               re_parsed["services"].as_mapping().unwrap().len(), 
               "Service count should be preserved");
               
    // Verify essential fields are preserved
    assert!(re_parsed["services"].is_mapping(), "Services section should be preserved");
}

#[test]
fn test_yaml_formatting_with_blank_lines() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID FORMATTING_TEST
SERVICES SECTION

SERVICE web
IMAGE-ID nginx:alpine
PORT-MAPPING 80 TO 80
END SERVICE

SERVICE app
IMAGE-ID python:3.11-slim
PORT-MAPPING 5000 TO 5000
DEPENDS-ON database
END SERVICE

SERVICE database
IMAGE-ID postgres:15
PORT-MAPPING 5432 TO 5432
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "formatting_test.ath", ath_content);
    
    // Generate the full YAML output (not just parse it)
    let temp_output = temp_dir.path().join("formatted_output.yml");
    let output_file = temp_output.to_string_lossy().to_string();
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    let result = cmd.arg("build")
        .arg(&ath_file)
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to execute command");
    
    assert!(result.status.success(), "Command should succeed");
    
    // Read the generated YAML file
    let yaml_content = std::fs::read_to_string(&output_file)
        .expect("Failed to read generated YAML file");
    
    // Check that there are blank lines between services
    let lines: Vec<&str> = yaml_content.lines().collect();
    let mut service_lines = Vec::new();
    let mut inside_services = false;
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("services:") {
            inside_services = true;
            continue;
        }
        
        if inside_services && !line.starts_with(" ") && !line.trim().is_empty() {
            inside_services = false;
        }
        
        // Find service definitions (2 spaces + service name + colon)
        if inside_services && line.starts_with("  ") && !line.starts_with("    ") && line.contains(':') {
            service_lines.push(i);
        }
    }
    
    // Should have 3 services
    assert_eq!(service_lines.len(), 3, "Should have exactly 3 service definitions");
    
    // Check that there are blank lines between services (except before the first one)
    for i in 1..service_lines.len() {
        let current_service_line = service_lines[i];
        let previous_line = current_service_line - 1;
        
        // The line before each service (except the first) should be blank
        assert!(lines[previous_line].trim().is_empty(), 
                "Should have blank line before service at line {}", current_service_line + 1);
    }
    
    // Verify that the services are properly separated
    assert!(yaml_content.contains("services:"), "Should contain services section");
    assert!(yaml_content.contains("  web:"), "Should contain web service");
    assert!(yaml_content.contains("  app:"), "Should contain app service");
    assert!(yaml_content.contains("  database:"), "Should contain database service");
}