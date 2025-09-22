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

fn run_athena_build(ath_file: &str, output_file: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    let result = cmd.arg("build")
        .arg(ath_file)
        .arg("-o")
        .arg(output_file)
        .output()
        .expect("Failed to execute command");
    
    if result.status.success() {
        fs::read_to_string(output_file).map_err(|e| e.into())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("Command failed: {}", stderr).into())
    }
}

fn parse_yaml_safely(yaml_content: &str) -> Result<Value, serde_yaml::Error> {
    serde_yaml::from_str(yaml_content)
}

#[test]
fn test_simple_service_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "simple.ath",
        include_str!("../fixtures/valid_simple.ath"),
    );
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    // Parse YAML to ensure it's valid
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    // Verify basic structure (modern Docker Compose doesn't require version field)
    assert!(parsed["services"].is_mapping(), "Should have services section");
    assert!(parsed["networks"].is_mapping(), "Should have networks section");
    
    // Verify that version field does NOT exist (modern Docker Compose style)
    assert!(parsed["version"].is_null(), "Version field should not exist in modern Docker Compose");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    
    // Verify all expected services are present
    assert!(services.contains_key("web"), "Should contain web service");
    assert!(services.contains_key("app"), "Should contain app service");
    assert!(services.contains_key("database"), "Should contain database service");
    
    // Verify web service configuration
    let web_service = &services["web"];
    assert_eq!(web_service["image"], "nginx:alpine");
    assert!(web_service["ports"].is_sequence(), "Web service should have ports");
    assert!(web_service["environment"].is_sequence(), "Web service should have environment variables");
    assert!(web_service["healthcheck"].is_mapping(), "Web service should have healthcheck");
    assert_eq!(web_service["restart"], "unless-stopped");
    
    // Verify app service configuration
    let app_service = &services["app"];
    assert_eq!(app_service["image"], "python:3.11-slim");
    assert!(app_service["depends_on"].is_sequence(), "App service should have dependencies");
    assert!(app_service["command"].is_string(), "App service should have custom command");
    
    // Verify database service configuration
    let database_service = &services["database"];
    assert_eq!(database_service["image"], "postgres:15");
    assert!(database_service["volumes"].is_sequence(), "Database service should have volumes");
    assert_eq!(database_service["restart"], "always");
    
    // Verify network configuration
    let networks = parsed["networks"].as_mapping().expect("Networks should be a mapping");
    assert!(networks.contains_key("simple_test_network"), "Should contain the specified network");
}

#[test]
fn test_complex_microservices_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "complex.ath",
        include_str!("../fixtures/valid_complex_microservices.ath"),
    );
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    // Parse YAML to ensure it's valid
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    
    // Verify all microservices are present
    let expected_services = [
        "api_gateway", "auth_service", "user_service", "payment_service", 
        "notification_service", "auth_db", "user_db", "payment_db", 
        "redis", "elasticsearch", "rabbitmq", "monitoring"
    ];
    
    for service_name in &expected_services {
        assert!(services.contains_key(*service_name), "Should contain {} service", service_name);
    }
    
    // Verify resource limits are applied
    let auth_service = &services["auth_service"];
    assert!(auth_service["deploy"]["resources"]["limits"].is_mapping(), 
        "Auth service should have resource limits");
    
    let limits = &auth_service["deploy"]["resources"]["limits"];
    assert!(limits["cpus"].is_string() || limits["cpus"].is_number(), 
        "Should have CPU limits");
    assert!(limits["memory"].is_string(), "Should have memory limits");
    
    // Verify complex dependencies
    let api_gateway = &services["api_gateway"];
    let depends_on = api_gateway["depends_on"].as_sequence().expect("Should have dependencies");
    assert!(depends_on.len() >= 3, "API Gateway should depend on multiple services");
    
    // Verify multiple port mappings
    let api_gateway_ports = api_gateway["ports"].as_sequence().expect("Should have ports");
    assert!(api_gateway_ports.len() >= 2, "API Gateway should expose multiple ports");
    
    // Verify volume mappings with options
    let elasticsearch = &services["elasticsearch"];
    assert!(elasticsearch["volumes"].is_sequence(), "Elasticsearch should have volumes");
    
    // Verify complex network configuration
    let networks = parsed["networks"].as_mapping().expect("Networks should be a mapping");
    assert!(networks.contains_key("complex_microservices_network"), 
        "Should contain the specified network");
}

#[test]
fn test_minimal_service_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "minimal.ath",
        include_str!("../fixtures/minimal_valid.ath"),
    );
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    // Parse YAML to ensure it's valid
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    
    // Verify minimal service
    assert!(services.contains_key("minimal_service"), "Should contain minimal_service");
    let minimal_service = &services["minimal_service"];
    assert_eq!(minimal_service["image"], "alpine:latest");
    assert!(minimal_service["command"].is_string(), "Should have custom command");
}

#[test]
fn test_environment_variable_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID ENV_VAR_TEST
VERSION-ID 1.0.0

ENVIRONMENT SECTION
NETWORK-NAME env_test_network

SERVICES SECTION

SERVICE test_service
IMAGE-ID alpine:latest
ENV-VARIABLE {{DATABASE_URL}}
ENV-VARIABLE {{API_KEY}}
ENV-VARIABLE {{SECRET_TOKEN}}
COMMAND "env"
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "env_test.ath", ath_content);
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let test_service = &services["test_service"];
    
    // Verify environment variables are correctly templated
    let env_vars = test_service["environment"].as_sequence().expect("Should have environment variables");
    assert!(env_vars.len() == 3, "Should have exactly 3 environment variables");
    
    // Check that environment variables contain template placeholders
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
fn test_port_mapping_generation() {
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
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let service = &services["multi_port_service"];
    
    let ports = service["ports"].as_sequence().expect("Should have ports");
    assert_eq!(ports.len(), 3, "Should have exactly 3 port mappings");
    
    // Convert ports to strings for easier checking
    let port_strings: Vec<String> = ports.iter()
        .map(|p| p.as_str().unwrap_or("").to_string())
        .collect();
    
    // Verify different port mapping formats
    assert!(port_strings.iter().any(|p| p.contains("8080:80")), 
        "Should contain HTTP port mapping");
    assert!(port_strings.iter().any(|p| p.contains("8443:443")), 
        "Should contain HTTPS port mapping");
}

#[test]
fn test_volume_mapping_generation() {
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
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let service = &services["volume_service"];
    
    let volumes = service["volumes"].as_sequence().expect("Should have volumes");
    assert!(volumes.len() >= 3, "Should have at least 3 volume mappings");
    
    let volume_strings: Vec<String> = volumes.iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();
    
    // Verify different volume mapping formats
    assert!(volume_strings.iter().any(|v| v.contains("./data:/var/lib/postgresql/data")), 
        "Should contain data volume mapping");
    assert!(volume_strings.iter().any(|v| v.contains("./config:/etc/postgresql:ro")), 
        "Should contain read-only config volume mapping");
    assert!(volume_strings.iter().any(|v| v.contains("logs:/var/log")), 
        "Should contain named volume mapping");
}

#[test]
fn test_health_check_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID HEALTH_TEST
SERVICES SECTION

SERVICE health_service
IMAGE-ID nginx:alpine
PORT-MAPPING 80 TO 80
HEALTH-CHECK "curl -f http://localhost:80/health || exit 1"
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "health_test.ath", ath_content);
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let service = &services["health_service"];
    
    assert!(service["healthcheck"].is_mapping(), "Should have healthcheck configuration");
    let healthcheck = &service["healthcheck"];
    
    assert!(healthcheck["test"].is_string() || healthcheck["test"].is_sequence(), 
        "Healthcheck should have test command");
    
    // Verify the health check command is properly formatted
    let test_cmd = if healthcheck["test"].is_string() {
        healthcheck["test"].as_str().unwrap().to_string()
    } else {
        // For sequence format like ["CMD-SHELL", "command"], check the actual command (second element)
        let sequence = healthcheck["test"].as_sequence().unwrap();
        if sequence.len() > 1 {
            sequence[1].as_str().unwrap().to_string()
        } else {
            sequence[0].as_str().unwrap().to_string()
        }
    };
    
    assert!(test_cmd.contains("curl") || test_cmd.contains("health"), 
        "Health check should contain the specified command");
}

#[test]
fn test_yaml_validity_and_formatting() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_file = create_test_ath_file(
        &temp_dir,
        "format_test.ath",
        include_str!("../fixtures/valid_complex_microservices.ath"),
    );
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    // Test that the YAML can be parsed and re-serialized
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    let re_serialized = serde_yaml::to_string(&parsed)
        .expect("Should be able to re-serialize YAML");
    
    // Re-parse to ensure consistency
    let re_parsed: Value = parse_yaml_safely(&re_serialized)
        .expect("Re-serialized YAML should be valid");
    
    // Basic structure should be preserved
    // Version field should not exist in both original and re-parsed
    assert!(parsed["version"].is_null(), "Original YAML should not have version field");
    assert!(re_parsed["version"].is_null(), "Re-parsed YAML should not have version field");
    assert_eq!(parsed["services"].as_mapping().unwrap().len(), 
               re_parsed["services"].as_mapping().unwrap().len(), 
               "Service count should be preserved");
}

#[test]
fn test_generation_with_custom_network_names() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID NETWORK_TEST
VERSION-ID 1.0.0

ENVIRONMENT SECTION
NETWORK-NAME custom_network_name

SERVICES SECTION

SERVICE test_service
IMAGE-ID alpine:latest
COMMAND "echo 'test'"
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "network_test.ath", ath_content);
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Failed to generate docker-compose.yml");
    
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    // Verify custom network name is used
    let networks = parsed["networks"].as_mapping().expect("Should have networks");
    assert!(networks.contains_key("custom_network_name"), 
        "Should contain custom network name");
    
    // Verify service is connected to the custom network
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    let service = &services["test_service"];
    
    if let Some(service_networks) = service.get("networks") {
        let network_list = service_networks.as_sequence().expect("Networks should be a sequence");
        let has_custom_network = network_list.iter()
            .any(|n| n.as_str() == Some("custom_network_name"));
        assert!(has_custom_network, "Service should be connected to custom network");
    }
}

#[test]
fn test_port_conflict_prevention_in_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID PORT_CONFLICT_TEST

SERVICES SECTION

SERVICE service1
IMAGE-ID nginx:alpine
PORT-MAPPING 8080 TO 80
END SERVICE

SERVICE service2
IMAGE-ID apache:latest
PORT-MAPPING 8080 TO 8000
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "port_conflict.ath", ath_content);
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    // This should fail due to port conflict
    let result = run_athena_build(&ath_file, &output_file);
    assert!(result.is_err(), "Build should fail due to port conflict");
    
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Port conflict detected"), 
        "Error should mention port conflict detection");
    assert!(error_message.contains("8080"), 
        "Error should mention the conflicting port");
}

#[test]
fn test_successful_generation_with_different_ports() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ath_content = r#"DEPLOYMENT-ID NO_PORT_CONFLICT_TEST

SERVICES SECTION

SERVICE service1
IMAGE-ID nginx:alpine
PORT-MAPPING 8080 TO 80
END SERVICE

SERVICE service2
IMAGE-ID apache:latest
PORT-MAPPING 8081 TO 8000
END SERVICE"#;
    
    let ath_file = create_test_ath_file(&temp_dir, "no_conflict.ath", ath_content);
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    // This should succeed with different ports
    let yaml_content = run_athena_build(&ath_file, &output_file)
        .expect("Build should succeed with different ports");
    
    let parsed: Value = parse_yaml_safely(&yaml_content)
        .expect("Generated YAML should be valid");
    
    let services = parsed["services"].as_mapping().expect("Services should be a mapping");
    assert!(services.contains_key("service1"), "Should contain service1");
    assert!(services.contains_key("service2"), "Should contain service2");
    
    // Verify both services have their respective ports
    let service1 = &services["service1"];
    let service2 = &services["service2"];
    
    let service1_ports = service1["ports"].as_sequence().expect("Service1 should have ports");
    let service2_ports = service2["ports"].as_sequence().expect("Service2 should have ports");
    
    let port1_str = service1_ports[0].as_str().unwrap();
    let port2_str = service2_ports[0].as_str().unwrap();
    
    assert!(port1_str.contains("8080"), "Service1 should use port 8080");
    assert!(port2_str.contains("8081"), "Service2 should use port 8081");
}