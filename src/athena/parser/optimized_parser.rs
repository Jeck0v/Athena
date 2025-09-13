use std::collections::HashMap;
use crate::athena::error::{AthenaError, AthenaResult};
use super::ast::*;

/// Optimized parser with performance improvements and better error handling
pub struct OptimizedParser;

impl OptimizedParser {
    /// Parse Athena file with optimized performance and enhanced error reporting
    pub fn parse_with_performance_optimizations(input: &str) -> AthenaResult<AthenaFile> {
        // Pre-validate input for common issues
        Self::pre_validate_input(input)?;
        
        // Use the original parser but with optimizations
        let result = super::parser::parse_athena_file(input)?;
        
        // Post-process for optimization
        Self::optimize_ast(result)
    }
    
    /// Pre-validation for common syntax issues to fail fast
    fn pre_validate_input(input: &str) -> AthenaResult<()> {
        let trimmed = input.trim();
        
        if trimmed.is_empty() {
            return Err(AthenaError::ParseError(
                "Input file is empty. Please provide a valid .ath file with at least a SERVICES SECTION.".to_string()
            ));
        }
        
        // Check for required SERVICES SECTION
        if !trimmed.contains("SERVICES SECTION") {
            return Err(AthenaError::ParseError(
                "Missing required 'SERVICES SECTION'. Every .ath file must contain at least one service definition.".to_string()
            ));
        }
        
        // Check for balanced SERVICE/END SERVICE blocks
        let service_count = trimmed.matches("SERVICE ").count();
        let end_service_count = trimmed.matches("END SERVICE").count();
        
        if service_count != end_service_count {
            return Err(AthenaError::ParseError(
                format!(
                    "Unbalanced SERVICE blocks: found {} 'SERVICE' declarations but {} 'END SERVICE' statements. \
                     Each SERVICE block must be closed with 'END SERVICE'.",
                    service_count, end_service_count
                )
            ));
        }
        
        // Check for empty service names
        if trimmed.contains("SERVICE \n") || trimmed.contains("SERVICE\n") {
            return Err(AthenaError::ParseError(
                "Empty service name detected. Each SERVICE declaration must be followed by a service name.".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Optimize the AST after parsing for better runtime performance
    fn optimize_ast(mut athena_file: AthenaFile) -> AthenaResult<AthenaFile> {
        // Sort services by dependency order for faster dependency resolution
        athena_file.services.services = Self::topological_sort_services(athena_file.services.services)?;
        
        // Optimize service configurations
        for service in &mut athena_file.services.services {
            Self::optimize_service(service);
        }
        
        // Set intelligent defaults if missing
        Self::apply_intelligent_defaults(&mut athena_file);
        
        Ok(athena_file)
    }
    
    /// Topological sort services by dependencies for optimal processing order
    fn topological_sort_services(services: Vec<Service>) -> AthenaResult<Vec<Service>> {
        use std::collections::{HashMap, VecDeque};
        
        // Build dependency graph
        let mut service_map: HashMap<String, Service> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize structures
        for service in services {
            service_map.insert(service.name.clone(), service);
        }
        
        for (name, service) in &service_map {
            in_degree.insert(name.clone(), 0);
            graph.insert(name.clone(), Vec::new());
            
            for dep in &service.depends_on {
                if !service_map.contains_key(dep) {
                    return Err(AthenaError::ValidationError(
                        format!("Service '{}' depends on '{}' which doesn't exist", name, dep)
                    ));
                }
            }
        }
        
        // Build edges and calculate in-degrees
        for (name, service) in &service_map {
            for dep in &service.depends_on {
                graph.get_mut(dep).unwrap().push(name.clone());
                *in_degree.get_mut(name).unwrap() += 1;
            }
        }
        
        // Kahn's algorithm for topological sorting
        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();
        
        let mut sorted_services = Vec::new();
        
        while let Some(current) = queue.pop_front() {
            sorted_services.push(service_map.remove(&current).unwrap());
            
            for neighbor in &graph[&current] {
                let degree = in_degree.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
        
        if sorted_services.len() != service_map.len() + sorted_services.len() {
            return Err(AthenaError::ValidationError(
                "Circular dependency detected in services".to_string()
            ));
        }
        
        Ok(sorted_services)
    }
    
    /// Optimize individual service configuration
    fn optimize_service(service: &mut Service) {
        // Remove duplicate environment variables
        service.environment.dedup();
        
        // Remove duplicate dependencies
        service.depends_on.dedup();
        
        // Optimize port mappings (remove duplicates, sort for consistency)
        service.ports.dedup_by(|a, b| {
            a.host_port == b.host_port && a.container_port == b.container_port
        });
        service.ports.sort_by_key(|p| p.host_port);
        
        // Optimize volume mappings
        service.volumes.dedup_by(|a, b| {
            a.host_path == b.host_path && a.container_path == b.container_path
        });
    }
    
    /// Apply intelligent defaults based on service patterns
    fn apply_intelligent_defaults(athena_file: &mut AthenaFile) {
        // Set default deployment if missing
        if athena_file.deployment.is_none() {
            athena_file.deployment = Some(DeploymentSection {
                deployment_id: "athena-project".to_string(),
                version_id: Some("1.0.0".to_string()),
            });
        }
        
        // Set default environment if missing
        if athena_file.environment.is_none() {
            athena_file.environment = Some(EnvironmentSection {
                network_name: None, // Will use project name
                volumes: Vec::new(),
                secrets: HashMap::new(),
            });
        }
        
        // Apply service-specific defaults
        for service in &mut athena_file.services.services {
            // Set default restart policy if missing
            if service.restart.is_none() {
                service.restart = Some(match service.image.as_deref() {
                    Some(img) if img.contains("postgres") || img.contains("mysql") || img.contains("mongodb") => {
                        RestartPolicy::Always
                    },
                    Some(img) if img.contains("redis") || img.contains("memcached") => {
                        RestartPolicy::Always
                    },
                    _ => RestartPolicy::UnlessStopped
                });
            }
        }
    }
    
    /// Parse with caching for repeated parsing operations
    pub fn parse_with_cache(
        input: &str, 
        cache: &mut HashMap<u64, AthenaFile>
    ) -> AthenaResult<AthenaFile> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Create hash of input for caching
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let input_hash = hasher.finish();
        
        // Check cache first
        if let Some(cached_result) = cache.get(&input_hash) {
            return Ok(cached_result.clone());
        }
        
        // Parse and cache result
        let result = Self::parse_with_performance_optimizations(input)?;
        cache.insert(input_hash, result.clone());
        
        Ok(result)
    }
    
    /// Validate syntax without full parsing for quick feedback
    pub fn quick_syntax_check(input: &str) -> AthenaResult<()> {
        Self::pre_validate_input(input)?;
        
        // Quick regex-based checks for common syntax issues
        let lines: Vec<&str> = input.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Check for malformed directives
            if trimmed.starts_with("SERVICE") && !trimmed.contains(' ') && trimmed != "SERVICE" {
                return Err(AthenaError::ParseError(
                    format!("Line {}: SERVICE directive requires a service name", line_num)
                ));
            }
            
            if trimmed.contains("PORT-MAPPING") && !trimmed.contains("TO") {
                return Err(AthenaError::ParseError(
                    format!("Line {}: PORT-MAPPING requires 'TO' keyword (e.g., 'PORT-MAPPING 8080 TO 80')", line_num)
                ));
            }
            
            if trimmed.contains("VOLUME-MAPPING") && !trimmed.contains("TO") {
                return Err(AthenaError::ParseError(
                    format!("Line {}: VOLUME-MAPPING requires 'TO' keyword", line_num)
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_validation() {
        assert!(OptimizedParser::pre_validate_input("").is_err());
        assert!(OptimizedParser::pre_validate_input("DEPLOYMENT-ID test").is_err());
        
        let valid_input = r#"
            SERVICES SECTION
            SERVICE test
            IMAGE-ID alpine:latest
            END SERVICE
        "#;
        assert!(OptimizedParser::pre_validate_input(valid_input).is_ok());
    }
    
    #[test]
    fn test_unbalanced_service_blocks() {
        let invalid_input = r#"
            SERVICES SECTION
            SERVICE test1
            IMAGE-ID alpine:latest
            SERVICE test2
            IMAGE-ID nginx:latest
            END SERVICE
        "#;
        assert!(OptimizedParser::pre_validate_input(invalid_input).is_err());
    }
    
    #[test]
    fn test_quick_syntax_check() {
        let invalid_port = r#"
            SERVICES SECTION
            SERVICE test
            PORT-MAPPING 8080 WRONG 80
            END SERVICE
        "#;
        assert!(OptimizedParser::quick_syntax_check(invalid_port).is_err());
        
        let valid_port = r#"
            SERVICES SECTION
            SERVICE test
            PORT-MAPPING 8080 TO 80
            END SERVICE
        "#;
        assert!(OptimizedParser::quick_syntax_check(valid_port).is_ok());
    }
}