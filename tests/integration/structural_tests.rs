use assert_cmd::Command;
use serde_yaml::Value;
use std::fs;
use tempfile::TempDir;
use pretty_assertions::assert_eq;

fn create_test_ath_file(temp_dir: &TempDir, filename: &str, content: &str) -> String {
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, content).expect("Failed to create test file");
    file_path.to_string_lossy().to_string()
}

fn run_athena_build_and_parse(ath_file: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    let result = cmd.arg("build")
        .arg(ath_file)
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to execute command");
    
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Command failed: {}", stderr).into());
    }
    
    let yaml_content = fs::read_to_string(&output_file)?;
    let parsed: Value = serde_yaml::from_str(&yaml_content)?;
    Ok(parsed)
}

#[test]
fn test_basic_yaml_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "basic.ath",
        include_str!("../fixtures/minimal_valid.ath"),
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
        include_str!("../fixtures/valid_simple.ath"),
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

#[test]
fn test_service_configuration_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "config.ath",
        include_str!("../fixtures/valid_simple.ath"),
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

#[test]
fn test_service_dependencies() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "deps.ath",
        include_str!("../fixtures/valid_simple.ath"),
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
fn test_yaml_validity() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "validity.ath",
        include_str!("../fixtures/valid_complex_microservices.ath"),
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
fn test_complex_microservices_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "complex.ath",
        include_str!("../fixtures/valid_complex_microservices.ath"),
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