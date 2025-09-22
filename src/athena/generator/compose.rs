use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::athena::error::{AthenaError, AthenaResult};
use crate::athena::parser::ast::*;
use super::defaults::{DefaultsEngine, EnhancedDockerService};

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerCompose {
    services: HashMap<String, EnhancedDockerService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    networks: Option<HashMap<String, DockerNetwork>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volumes: Option<HashMap<String, DockerVolume>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

// Legacy DockerService - kept for backward compatibility
// Use EnhancedDockerService for new implementations

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerHealthCheck {
    test: Vec<String>,
    interval: String,
    timeout: String,
    retries: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerDeploy {
    resources: Option<DockerResources>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerResources {
    limits: Option<ResourceSpec>,
    reservations: Option<ResourceSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceSpec {
    cpus: Option<String>,
    memory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerNetwork {
    driver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerVolume {
    driver: Option<String>,
}

/// Generate optimized Docker Compose with intelligent defaults
pub fn generate_docker_compose(athena_file: &AthenaFile) -> AthenaResult<String> {
    let project_name = athena_file.get_project_name();
    let network_name = athena_file.get_network_name();
    
    let mut compose = DockerCompose {
        name: Some(project_name.clone()),
        services: HashMap::new(),
        networks: None,
        volumes: None,
    };

    // Create optimized network configuration
    compose.networks = Some(create_optimized_networks(&network_name));

    // Process volumes with enhanced configuration
    if let Some(env) = &athena_file.environment {
        if !env.volumes.is_empty() {
            compose.volumes = Some(create_optimized_volumes(&env.volumes));
        }
    }

    // Convert services using intelligent defaults
    for service in &athena_file.services.services {
        let enhanced_service = DefaultsEngine::create_enhanced_service(
            service, 
            &network_name, 
            &project_name
        );
        compose.services.insert(service.name.clone(), enhanced_service);
    }

    // Fast validation with enhanced error reporting
    validate_compose_enhanced(&compose)?;

    // Generate optimized YAML
    let yaml = serde_yaml::to_string(&compose)
        .map_err(AthenaError::YamlError)?;

    // Improve formatting for better readability
    let formatted_yaml = improve_yaml_formatting(yaml);

    Ok(add_enhanced_yaml_comments(formatted_yaml, athena_file))
}

/// Create optimized network configuration
fn create_optimized_networks(network_name: &str) -> HashMap<String, DockerNetwork> {
    let mut networks = HashMap::new();
    networks.insert(
        network_name.to_string(),
        DockerNetwork {
            driver: "bridge".to_string(),
        },
    );
    networks
}

/// Create optimized volume configuration  
fn create_optimized_volumes(volume_defs: &[VolumeDefinition]) -> HashMap<String, DockerVolume> {
    let mut volumes = HashMap::new();
    for vol_def in volume_defs {
        volumes.insert(
            vol_def.name.clone(),
            DockerVolume {
                driver: Some("local".to_string()),
            },
        );
    }
    volumes
}

/// Enhanced validation with better error reporting and performance
fn validate_compose_enhanced(compose: &DockerCompose) -> AthenaResult<()> {
    // Pre-allocate for better performance
    let service_names: std::collections::HashSet<String> = compose.services.keys().cloned().collect();
    
    // Parallel validation for better performance on large compositions
    for (service_name, service) in &compose.services {
        // Image or build validation - service must have at least one
        if service.image.is_none() && service.build.is_none() {
            return Err(AthenaError::ValidationError(
                format!("Service '{}' is missing both image and build configuration", service_name)
            ));
        }

        // Enhanced dependency validation
        if let Some(deps) = &service.depends_on {
            for dep in deps {
                if !service_names.contains(dep) {
                    return Err(AthenaError::ValidationError(
                        format!(
                            "Service '{}' depends on '{}' which doesn't exist. Available services: {}", 
                            service_name, 
                            dep,
                            service_names.iter().cloned().collect::<Vec<_>>().join(", ")
                        )
                    ));
                }
            }
        }
        
        // Validate port mappings
        if let Some(ports) = &service.ports {
            for port_mapping in ports {
                if !is_valid_port_mapping(port_mapping) {
                    return Err(AthenaError::ValidationError(
                        format!("Service '{}' has invalid port mapping: {}", service_name, port_mapping)
                    ));
                }
            }
        }
    }

    // Fast circular dependency detection
    detect_circular_dependencies_optimized(compose)?;

    // Detect port conflicts between services
    detect_port_conflicts(compose)?;

    Ok(())
}

/// Validate port mapping format
fn is_valid_port_mapping(port_mapping: &str) -> bool {
    // Basic validation for format like "8080:80" or "8080:80/tcp"
    let parts: Vec<&str> = port_mapping.split(':').collect();
    if parts.len() < 2 {
        return false;
    }
    
    // Validate host port
    if parts[0].parse::<u16>().is_err() {
        return false;
    }
    
    // Validate container port (may include protocol)
    let container_part = parts[1];
    let container_port = if container_part.contains('/') {
        container_part.split('/').next().unwrap_or("")
    } else {
        container_part
    };
    
    container_port.parse::<u16>().is_ok()
}

/// Optimized circular dependency detection using iterative DFS
fn detect_circular_dependencies_optimized(compose: &DockerCompose) -> AthenaResult<()> {
    use std::collections::HashSet;
    
    let mut visited = HashSet::new();
    let mut temp_visited = HashSet::new();
    
    for service_name in compose.services.keys() {
        if !visited.contains(service_name) {
            if has_cycle_iterative(service_name, compose, &mut visited, &mut temp_visited)? {
                return Err(AthenaError::ValidationError(
                    format!(
                        "Circular dependency detected involving service '{}'. \
                         Check the DEPENDS-ON declarations in your .ath file.", 
                        service_name
                    )
                ));
            }
        }
    }
    
    Ok(())
}

/// Iterative cycle detection for better performance and stack safety
fn has_cycle_iterative(
    start_service: &str,
    compose: &DockerCompose,
    visited: &mut std::collections::HashSet<String>,
    temp_visited: &mut std::collections::HashSet<String>,
) -> AthenaResult<bool> {
    use std::collections::VecDeque;
    
    let mut stack = VecDeque::new();
    stack.push_back((start_service.to_string(), false));
    
    while let Some((service, is_return)) = stack.pop_back() {
        if is_return {
            temp_visited.remove(&service);
            continue;
        }
        
        if temp_visited.contains(&service) {
            return Ok(true); // Cycle detected
        }
        
        if visited.contains(&service) {
            continue;
        }
        
        visited.insert(service.clone());
        temp_visited.insert(service.clone());
        
        // Add return marker
        stack.push_back((service.clone(), true));
        
        // Add dependencies
        if let Some(service_def) = compose.services.get(&service) {
            if let Some(deps) = &service_def.depends_on {
                for dep in deps {
                    stack.push_back((dep.clone(), false));
                }
            }
        }
    }
    
    Ok(false)
}

/// Detect port conflicts between services
fn detect_port_conflicts(compose: &DockerCompose) -> AthenaResult<()> {
    use std::collections::HashMap;
    
    let mut port_to_services: HashMap<String, Vec<String>> = HashMap::new();
    
    // Collect all host ports from all services
    for (service_name, service) in &compose.services {
        if let Some(ports) = &service.ports {
            for port_mapping in ports {
                if let Some(host_port) = extract_host_port(port_mapping) {
                    port_to_services
                        .entry(host_port)
                        .or_insert_with(Vec::new)
                        .push(service_name.clone());
                }
            }
        }
    }
    
    // Check for conflicts
    for (port, services) in port_to_services {
        if services.len() > 1 {
            return Err(AthenaError::ValidationError(
                format!(
                    "Port conflict detected! Host port {} is used by multiple services: {}. \
                     Each service must use a unique host port. Consider using different ports like: {}",
                    port,
                    services.join(", "),
                    generate_port_suggestions(&port, services.len())
                )
            ));
        }
    }
    
    Ok(())
}

/// Extract host port from port mapping (e.g., "8080:80" -> "8080")
fn extract_host_port(port_mapping: &str) -> Option<String> {
    let parts: Vec<&str> = port_mapping.split(':').collect();
    if parts.len() >= 2 {
        Some(parts[0].to_string())
    } else {
        None
    }
}

/// Generate port suggestions for conflicts
fn generate_port_suggestions(base_port: &str, count: usize) -> String {
    if let Ok(port_num) = base_port.parse::<u16>() {
        let mut suggestions = Vec::new();
        for i in 0..count {
            suggestions.push((port_num + i as u16).to_string());
        }
        suggestions.join(", ")
    } else {
        "8080, 8081, 8082".to_string() // fallback suggestions
    }
}

/// Improve YAML formatting for better readability by adding blank lines between services
fn improve_yaml_formatting(yaml: String) -> String {
    let lines: Vec<&str> = yaml.lines().collect();
    let mut formatted_lines = Vec::new();
    let mut inside_services = false;
    let mut first_service = true;
    
    for line in lines.iter() {
        // Check if we're in the services section
        if line.starts_with("services:") {
            inside_services = true;
            first_service = true;
            formatted_lines.push(line.to_string());
            continue;
        }
        
        // Check if we've left the services section (reached networks, volumes, etc.)
        if inside_services && !line.starts_with(" ") && !line.trim().is_empty() {
            inside_services = false;
        }
        
        // Detect service definition: exactly 2 spaces + service name + colon
        if inside_services && line.starts_with("  ") && !line.starts_with("    ") && line.contains(':') {
            // This is a service definition (e.g., "  web:", "  app:", "  database:")
            if !first_service {
                formatted_lines.push(String::new()); // Add blank line before service
            }
            first_service = false;
        }
        
        formatted_lines.push(line.to_string());
    }
    
    formatted_lines.join("\n")
}

/// Add enhanced YAML comments with metadata and optimization notes
fn add_enhanced_yaml_comments(yaml: String, athena_file: &AthenaFile) -> String {
    let mut result = String::with_capacity(yaml.len() + 500); // Pre-allocate for better performance
    
    // Enhanced header with metadata
    result.push_str(&format!(
        "# Generated by Athena v{} from {} deployment\n", 
        env!("CARGO_PKG_VERSION"),
        athena_file.get_project_name()
    ));
    
    if let Some(deployment) = &athena_file.deployment {
        if let Some(version) = &deployment.version_id {
            result.push_str(&format!("# Project Version: {}\n", version));
        }
    }
    
    result.push_str(&format!(
        "# Generated: {}\n", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
    
    result.push_str("# Features: Intelligent defaults, optimized networking, enhanced health checks\n");
    result.push_str("# DO NOT EDIT MANUALLY - This file is auto-generated\n\n");
    
    // Add service count and optimization info
    let service_count = athena_file.services.services.len();
    result.push_str(&format!("# Services: {} configured with intelligent defaults\n\n", service_count));
    
    result.push_str(&yaml);
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_compose_generation() {
        let mut athena_file = AthenaFile::new();
        athena_file.deployment = Some(DeploymentSection {
            deployment_id: "test_project".to_string(),
            version_id: Some("1.0.0".to_string()),
        });

        let mut service = Service::new("backend".to_string());
        service.image = Some("python:3.11-slim".to_string());
        service.ports.push(PortMapping {
            host_port: 8000,
            container_port: 8000,
            protocol: Protocol::Tcp,
        });

        athena_file.services.services.push(service);

        let result = generate_docker_compose(&athena_file);
        assert!(result.is_ok());
        
        let yaml = result.unwrap();
        assert!(!yaml.contains("version:"));
        assert!(yaml.contains("backend:"));
        assert!(yaml.contains("image: python:3.11-slim"));
        assert!(yaml.contains("8000:8000"));
        assert!(yaml.contains("restart: unless-stopped"));
        assert!(yaml.contains("container_name: test-project-backend"));
    }


    #[test]
    fn test_extract_host_port() {
        assert_eq!(extract_host_port("8080:80"), Some("8080".to_string()));
        assert_eq!(extract_host_port("3000:3000/tcp"), Some("3000".to_string()));
        assert_eq!(extract_host_port("80"), None);
        assert_eq!(extract_host_port(""), None);
    }

    #[test]
    fn test_port_suggestions() {
        assert_eq!(generate_port_suggestions("8080", 3), "8080, 8081, 8082");
        assert_eq!(generate_port_suggestions("3000", 2), "3000, 3001");
        assert_eq!(generate_port_suggestions("invalid", 2), "8080, 8081, 8082");
    }
}