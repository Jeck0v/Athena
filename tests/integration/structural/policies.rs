use super::{create_test_ath_file, run_athena_build_and_parse};
use tempfile::TempDir;

#[test]
fn test_restart_policies() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID RESTART_TEST
SERVICES SECTION

SERVICE always_service
IMAGE-ID postgres:15
RESTART-POLICY always
END SERVICE

SERVICE unless_stopped_service
IMAGE-ID nginx:alpine
RESTART-POLICY unless-stopped
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "restart_test.ath", ath_content);
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    
    // Test restart policies
    assert_eq!(services["always_service"]["restart"], "always", 
        "Always service should have 'always' restart policy");
    assert_eq!(services["unless_stopped_service"]["restart"], "unless-stopped", 
        "Unless stopped service should have 'unless-stopped' restart policy");
}

#[test]
fn test_health_checks() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID HEALTH_TEST
SERVICES SECTION

SERVICE health_service
IMAGE-ID nginx:alpine
PORT-MAPPING 80 TO 80
HEALTH-CHECK "curl -f http://localhost:80/health || exit 1"
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "health_test.ath", ath_content);
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let service = &services["health_service"];
    
    // Verify healthcheck structure
    assert!(service["healthcheck"].is_mapping(), "Should have healthcheck configuration");
    let healthcheck = &service["healthcheck"];
    
    assert!(healthcheck["test"].is_string() || healthcheck["test"].is_sequence(), 
        "Healthcheck should have test command");
    assert!(healthcheck["interval"].is_string(), "Healthcheck should have interval");
    assert!(healthcheck["timeout"].is_string(), "Healthcheck should have timeout");
    assert!(healthcheck["retries"].is_number(), "Healthcheck should have retries");
}