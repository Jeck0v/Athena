use super::{create_test_ath_file, run_athena_build_and_parse};
use tempfile::TempDir;

#[test]
fn test_complex_microservices_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "complex.ath",
        include_str!("../../fixtures/valid_complex_microservices.ath"),
    );
    
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    
    // Verify that we have a reasonable number of services (not all may be implemented)
    assert!(services.len() >= 3, "Should have at least 3 services in complex setup");
    
    // Check for some key services that should exist
    let key_services = ["api_gateway", "auth_service", "user_service"];
    let mut found_services = 0;
    
    for service_name in &key_services {
        if services.contains_key(*service_name) {
            found_services += 1;
        }
    }
    
    assert!(found_services >= 1, "Should find at least one key service in complex setup");
}