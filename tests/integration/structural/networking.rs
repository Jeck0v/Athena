use super::{create_test_ath_file, run_athena_build_and_parse};
use tempfile::TempDir;

#[test]
fn test_network_configuration() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID NETWORK_TEST
VERSION-ID 1.0.0

ENVIRONMENT SECTION
NETWORK-NAME custom_test_network

SERVICES SECTION

SERVICE test_service
IMAGE-ID alpine:latest
COMMAND "echo 'test'"
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "network_test.ath", ath_content);
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    // Verify custom network configuration
    assert!(parsed["networks"].is_mapping(), "Should have networks section");
    let networks = parsed["networks"].as_mapping().expect("Networks should be mapping");
    assert!(networks.contains_key("custom_test_network"), 
        "Should contain custom network name");
    
    // Verify network has correct configuration
    let custom_network = &networks["custom_test_network"];
    assert!(custom_network.is_mapping(), "Network should have configuration");
}

#[test]
fn test_service_dependencies() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "deps.ath",
        include_str!("../../fixtures/valid_simple.ath"),
    );
    
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    
    // Test that app service depends on database
    let app_service = &services["app"];
    assert!(app_service["depends_on"].is_sequence(), "App should have dependencies");
    
    let dependencies = app_service["depends_on"].as_sequence().expect("Dependencies should be sequence");
    let dep_strings: Vec<String> = dependencies.iter()
        .map(|d| d.as_str().unwrap().to_string())
        .collect();
    
    assert!(dep_strings.contains(&"database".to_string()), 
        "App service should depend on database service");
}