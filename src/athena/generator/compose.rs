use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write;

use super::defaults::{DefaultsEngine, EnhancedDockerService};
use crate::athena::dockerfile::{analyze_dockerfile, validate_build_args_against_dockerfile};
use crate::athena::error::{
    AthenaError, AthenaResult, EnhancedValidationError,
};
use crate::athena::parser::ast::{AthenaFile, NetworkDriver, VolumeDefinition};

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerCompose {
    services: IndexMap<String, EnhancedDockerService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    networks: Option<BTreeMap<String, DockerNetwork>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volumes: Option<BTreeMap<String, DockerVolume>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerNetwork {
    driver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ingress: Option<bool>,
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
        name: Some(project_name.to_lowercase().replace('_', "-")),
        services: IndexMap::new(),
        networks: None,
        volumes: None,
    };

    // Create optimized network configuration with Swarm support
    compose.networks = Some(create_optimized_networks(athena_file));

    // Process volumes with enhanced configuration
    if let Some(env) = &athena_file.environment {
        if !env.volumes.is_empty() {
            compose.volumes = Some(create_optimized_volumes(&env.volumes));
        }
    }

    // Sort services in dependency order (no-deps first, then dependents)
    let sorted_services = topological_sort_services(&athena_file.services.services);

    // Convert services using intelligent defaults, inserting in topological order
    for service in &sorted_services {
        let enhanced_service =
            DefaultsEngine::create_enhanced_service(service, &network_name, &project_name);
        compose
            .services
            .insert(service.name.clone(), enhanced_service);
    }

    // Fast validation with enhanced error reporting
    validate_compose_enhanced(&compose, athena_file)?;

    // Generate optimized YAML
    let yaml = serde_yaml::to_string(&compose).map_err(AthenaError::YamlError)?;

    // Improve formatting for better readability
    let formatted_yaml = improve_yaml_formatting(yaml);

    Ok(add_enhanced_yaml_comments(formatted_yaml, athena_file))
}

/// Sort services in topological order: services with no dependencies first,
/// then services that depend on them, etc. Falls back to original order on cycles.
fn topological_sort_services(services: &[crate::athena::parser::ast::Service]) -> Vec<&crate::athena::parser::ast::Service> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let name_to_service: HashMap<&str, &crate::athena::parser::ast::Service> =
        services.iter().map(|s| (s.name.as_str(), s)).collect();

    // Build in-degree map
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for service in services {
        in_degree.entry(service.name.as_str()).or_insert(0);
        dependents.entry(service.name.as_str()).or_default();
        for dep in &service.depends_on {
            if name_to_service.contains_key(dep.as_str()) {
                dependents.entry(dep.as_str()).or_default().push(&service.name);
                *in_degree.entry(service.name.as_str()).or_insert(0) += 1;
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&name, _)| name)
        .collect();

    // Sort the initial queue for determinism among peers
    let mut queue_vec: Vec<&str> = queue.drain(..).collect();
    queue_vec.sort();
    queue = queue_vec.into_iter().collect();

    let mut sorted: Vec<&crate::athena::parser::ast::Service> = Vec::with_capacity(services.len());
    let mut visited = HashSet::new();

    while let Some(current) = queue.pop_front() {
        if visited.contains(current) {
            continue;
        }
        visited.insert(current);

        if let Some(&service) = name_to_service.get(current) {
            sorted.push(service);
        }

        // Collect and sort neighbors for deterministic order among peers
        let mut neighbors: Vec<&str> = dependents
            .get(current)
            .map(|v| v.as_slice())
            .unwrap_or_default()
            .to_vec();
        neighbors.sort();

        for neighbor in neighbors {
            if let Some(deg) = in_degree.get_mut(neighbor) {
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    // Fallback: if cycle detected, append remaining services in original order
    if sorted.len() < services.len() {
        for service in services {
            if !visited.contains(service.name.as_str()) {
                sorted.push(service);
            }
        }
    }

    sorted
}

/// Create optimized network configuration with Docker Swarm support
fn create_optimized_networks(athena_file: &AthenaFile) -> BTreeMap<String, DockerNetwork> {
    let mut networks = BTreeMap::new();
    
    if let Some(env) = &athena_file.environment {
        // Use networks defined in environment section
        for network_def in &env.networks {
            let driver = match &network_def.driver {
                Some(NetworkDriver::Bridge) => "bridge".to_string(),
                Some(NetworkDriver::Overlay) => "overlay".to_string(),
                Some(NetworkDriver::Host) => "host".to_string(),
                Some(NetworkDriver::None) => "none".to_string(),
                None => "bridge".to_string(),
            };
            
            networks.insert(
                network_def.name.clone(),
                DockerNetwork {
                    driver,
                    attachable: network_def.attachable,
                    encrypted: network_def.encrypted,
                    ingress: network_def.ingress,
                },
            );
        }
    }
    
    // If no networks defined, create default network
    if networks.is_empty() {
        let default_name = athena_file.get_network_name();
        networks.insert(
            default_name,
            DockerNetwork {
                driver: "bridge".to_string(),
                attachable: None,
                encrypted: None,
                ingress: None,
            },
        );
    }
    
    networks
}

/// Create optimized volume configuration
fn create_optimized_volumes(volume_defs: &[VolumeDefinition]) -> BTreeMap<String, DockerVolume> {
    let mut volumes = BTreeMap::new();
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
fn validate_compose_enhanced(
    compose: &DockerCompose,
    athena_file: &AthenaFile,
) -> AthenaResult<()> {
    // Pre-allocate for better performance
    let service_names: std::collections::HashSet<String> =
        compose.services.keys().cloned().collect();

    // Parallel validation for better performance on large compositions
    for (service_name, service) in &compose.services {
        // Image or build validation - service must have at least one
        if service.image.is_none() && service.build.is_none() {
            return Err(AthenaError::validation_error_enhanced(
                EnhancedValidationError::new(
                    format!("Service '{service_name}' is missing both image and build configuration"),
                )
                .with_suggestion("Add IMAGE-ID \"image:tag\" or ensure a Dockerfile exists in the current directory".to_string())
                .with_services(vec![service_name.clone()])
            ));
        }

        // Enhanced dependency validation
        if let Some(deps) = &service.depends_on {
            for dep in deps {
                if !service_names.contains(dep) {
                    let available: Vec<String> = service_names.iter().cloned().collect();
                    return Err(AthenaError::validation_error_enhanced(
                        EnhancedValidationError::service_reference(service_name, dep, &available),
                    ));
                }
            }
        }

        // Validate port mappings
        if let Some(ports) = &service.ports {
            for port_mapping in ports {
                if !is_valid_port_mapping(port_mapping) {
                    return Err(AthenaError::validation_error_enhanced(
                        EnhancedValidationError::new(
                            format!("Service '{service_name}' has invalid port mapping: {port_mapping}"),
                        )
                        .with_suggestion("Use format: PORT-MAPPING <host_port> TO <container_port>, e.g., PORT-MAPPING 8080 TO 80".to_string())
                        .with_services(vec![service_name.clone()])
                    ));
                }
            }
        }
    }

    // Fast circular dependency detection
    detect_circular_dependencies_optimized(compose)?;

    // Detect port conflicts between services
    detect_port_conflicts(compose)?;

    // Advanced validation: BUILD-ARGS vs Dockerfile ARGs
    validate_dockerfile_build_args(athena_file)?;

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
        if !visited.contains(service_name)
            && has_cycle_iterative(service_name, compose, &mut visited, &mut temp_visited)? {
                return Err(AthenaError::validation_error_enhanced(
                    EnhancedValidationError::circular_dependency(service_name),
                ));
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
                        .or_default()
                        .push(service_name.clone());
                }
            }
        }
    }

    // Check for conflicts
    for (port, services) in port_to_services {
        if services.len() > 1 {
            let suggestion = format!(
                "Use different host ports, e.g., {}",
                generate_port_suggestions(&port, services.len())
            );

            let error = EnhancedValidationError::new(
                format!(
                    "Port conflict detected! Host port {} is used by multiple services: {}",
                    port,
                    services.join(", ")
                ),
            )
            .with_suggestion(suggestion)
            .with_services(services);

            return Err(AthenaError::validation_error_enhanced(error));
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

/// Validate BUILD-ARGS against Dockerfile ARGs (intelligent validation).
fn validate_dockerfile_build_args(athena_file: &AthenaFile) -> AthenaResult<()> {
    for service in &athena_file.services.services {
        if let Some(build_args) = &service.build_args {
            let dockerfile_path = "Dockerfile";

            let dockerfile_analysis = match analyze_dockerfile(dockerfile_path) {
                Ok(analysis) => analysis,
                Err(_) => continue,
            };

            let warnings = validate_build_args_against_dockerfile(build_args, &dockerfile_analysis);

            if !warnings.is_empty() {
                let combined_warning = warnings.join("\n\n");
                return Err(AthenaError::validation_error_enhanced(
                    EnhancedValidationError::new(format!(
                        "BUILD-ARGS validation failed for service '{}':\n\n{combined_warning}",
                        service.name
                    ))
                    .with_suggestion(
                        "Ensure all BUILD-ARGS correspond to ARG declarations in your Dockerfile"
                            .to_string(),
                    )
                    .with_services(vec![service.name.clone()]),
                ));
            }
        }
    }

    Ok(())
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
        if inside_services
            && line.starts_with("  ")
            && !line.starts_with("    ")
            && line.contains(':')
        {
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
    let mut result = String::with_capacity(yaml.len() + 500);

    let _ = writeln!(
        result,
        "# Generated by Athena v{} from {} deployment",
        env!("CARGO_PKG_VERSION"),
        athena_file.get_project_name()
    );
    let _ = writeln!(result, "# Developed by UNFAIR Team: https://github.com/Jeck0v/Athena");

    if let Some(deployment) = &athena_file.deployment {
        if let Some(version) = &deployment.version_id {
            let _ = writeln!(result, "# Project Version: {version}");
        }
    }

    let _ = writeln!(
        result,
        "# Generated: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    let _ = writeln!(
        result,
        "# Features: Intelligent defaults, optimized networking, enhanced health checks\n"
    );

    let service_count = athena_file.services.services.len();
    let _ = writeln!(
        result,
        "# Services: {service_count} configured with intelligent defaults\n"
    );

    result.push_str(&yaml);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::athena::parser::ast::{DeploymentSection, PortMapping, Protocol, Service};

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
        assert!(!yaml.contains("container_name:"));
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
