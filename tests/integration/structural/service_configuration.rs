use super::{create_test_ath_file, run_athena_build_and_parse};
use tempfile::TempDir;

#[test]
fn test_service_configuration_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "config.ath",
        include_str!("../../fixtures/valid_simple.ath"),
    );
    
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    
    // Test web service configuration
    let web_service = &services["web"];
    assert_eq!(web_service["image"], "nginx:alpine", "Web service should have correct image");
    assert!(web_service["ports"].is_sequence(), "Web service should have ports");
    
    // Environment variables are optional - only check if they exist
    if let Some(env) = web_service.get("environment") {
        assert!(env.is_sequence(), "Environment should be a sequence if present");
    }
    
    // Health checks are optional - only check if they exist  
    if let Some(healthcheck) = web_service.get("healthcheck") {
        assert!(healthcheck.is_mapping(), "Healthcheck should be a mapping if present");
    }
    
    assert_eq!(web_service["restart"], "unless-stopped", "Web service should have correct restart policy");
    
    // Test app service configuration
    let app_service = &services["app"];
    assert_eq!(app_service["image"], "python:3.11-slim", "App service should have correct image");
    assert!(app_service["depends_on"].is_sequence(), "App service should have dependencies");
    assert!(app_service["command"].is_string(), "App service should have custom command");
    
    // Test database service configuration
    let database_service = &services["database"];
    assert_eq!(database_service["image"], "postgres:15", "Database service should have correct image");
    assert!(database_service["volumes"].is_sequence(), "Database service should have volumes");
    assert_eq!(database_service["restart"], "always", "Database service should have always restart policy");
}

#[test]
fn test_environment_variables() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID ENV_TEST
SERVICES SECTION

SERVICE test_service
IMAGE-ID alpine:latest
ENV-VARIABLE {{DATABASE_URL}}
ENV-VARIABLE {{API_KEY}}
ENV-VARIABLE {{SECRET_TOKEN}}
COMMAND "env"
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "env_test.ath", ath_content);
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let test_service = &services["test_service"];
    
    // Verify environment variables structure - they should exist for this specific test
    let env_vars = if let Some(env) = test_service.get("environment") {
        assert!(env.is_sequence(), "Environment should be a sequence");
        env.as_sequence().expect("Environment should be sequence")
    } else {
        // If environment variables are not generated, this reveals a real issue
        panic!("Environment variables should be generated for this test service");
    };
    
    assert_eq!(env_vars.len(), 3, "Should have exactly 3 environment variables");
    
    // Check that environment variables contain expected patterns
    let env_strings: Vec<String> = env_vars.iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    
    assert!(env_strings.iter().any(|s| s.contains("DATABASE_URL")), 
        "Should contain DATABASE_URL environment variable");
    assert!(env_strings.iter().any(|s| s.contains("API_KEY")), 
        "Should contain API_KEY environment variable");
    assert!(env_strings.iter().any(|s| s.contains("SECRET_TOKEN")), 
        "Should contain SECRET_TOKEN environment variable");
}

#[test]
fn test_port_mappings() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID PORT_TEST
SERVICES SECTION

SERVICE multi_port_service
IMAGE-ID nginx:alpine
PORT-MAPPING 8080 TO 80
PORT-MAPPING 8443 TO 443
PORT-MAPPING 9090 TO 9090
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "port_test.ath", ath_content);
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let service = &services["multi_port_service"];
    
    // Verify port mappings structure
    assert!(service["ports"].is_sequence(), "Should have ports");
    let ports = service["ports"].as_sequence().expect("Ports should be sequence");
    assert!(ports.len() >= 3, "Should have at least 3 port mappings");
    
    // Convert ports to strings for easier checking
    let port_strings: Vec<String> = ports.iter()
        .map(|p| p.as_str().unwrap_or("").to_string())
        .collect();
    
    // Verify specific port mappings exist
    assert!(port_strings.iter().any(|p| p.contains("8080") && p.contains("80")), 
        "Should contain HTTP port mapping");
    assert!(port_strings.iter().any(|p| p.contains("8443") && p.contains("443")), 
        "Should contain HTTPS port mapping");
}

#[test]
fn test_volume_mappings() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID VOLUME_TEST
SERVICES SECTION

SERVICE volume_service
IMAGE-ID postgres:15
VOLUME-MAPPING "./data" TO "/var/lib/postgresql/data"
VOLUME-MAPPING "./config" TO "/etc/postgresql" (ro)
VOLUME-MAPPING "logs" TO "/var/log" (rw)
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "volume_test.ath", ath_content);
    let parsed = run_athena_build_and_parse(&ath_file)
        .expect("Failed to generate and parse YAML");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let service = &services["volume_service"];
    
    // Verify volume mappings structure
    assert!(service["volumes"].is_sequence(), "Should have volumes");
    let volumes = service["volumes"].as_sequence().expect("Volumes should be sequence");
    assert!(volumes.len() >= 3, "Should have at least 3 volume mappings");
    
    let volume_strings: Vec<String> = volumes.iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();
    
    // Verify specific volume mappings
    assert!(volume_strings.iter().any(|v| v.contains("./data") && v.contains("/var/lib/postgresql/data")), 
        "Should contain data volume mapping");
    assert!(volume_strings.iter().any(|v| v.contains("./config") && v.contains("/etc/postgresql")), 
        "Should contain config volume mapping");
}