use super::{create_test_ath_file, run_athena_build_and_parse};
use tempfile::TempDir;

#[test]
fn test_basic_yaml_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "basic.ath",
        include_str!("../../fixtures/minimal_valid.ath"),
    );
    
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    // Verify basic Docker Compose structure (modern format - no version field needed)
    assert!(parsed["services"].is_mapping(), "Should have services section");
    
    // Networks section is optional, only check if it exists
    if let Some(networks) = parsed.get("networks") {
        assert!(networks.is_mapping(), "Networks should be a mapping if present");
    }
    
    // Verify services count
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    assert_eq!(services.len(), 1, "Should have exactly 1 service");
    
    // Verify specific service exists
    assert!(services.contains_key("minimal_service"), "Should contain minimal_service");
}

#[test]
fn test_multi_service_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "multi.ath",
        include_str!("../../fixtures/valid_simple.ath"),
    );
    
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    
    // Verify all expected services exist
    let expected_services = ["web", "app", "database"];
    assert_eq!(services.len(), expected_services.len(), 
        "Should have {} services", expected_services.len());
    
    for service_name in &expected_services {
        assert!(services.contains_key(*service_name), 
            "Should contain {} service", service_name);
    }
}