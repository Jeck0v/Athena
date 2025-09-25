use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::athena::parser::ast::*;

/// Default Docker Compose configurations based on service patterns and Docker standards
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServiceDefaults {
    pub restart_policy: RestartPolicy,
    pub health_check_interval: String,
    pub health_check_timeout: String,
    pub health_check_retries: u32,
    pub health_check_start_period: String,
    #[allow(dead_code)]
    pub network_mode: NetworkMode,
    pub pull_policy: PullPolicy,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NetworkMode {
    Bridge,
    #[allow(dead_code)]
    Host,
    #[allow(dead_code)]
    None,
    #[allow(dead_code)]
    Custom(String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PullPolicy {
    #[allow(dead_code)]
    Always,
    Missing,
    #[allow(dead_code)]
    Never,
}

impl Default for ServiceDefaults {
    fn default() -> Self {
        Self {
            restart_policy: RestartPolicy::UnlessStopped,
            health_check_interval: "30s".to_string(),
            health_check_timeout: "10s".to_string(),
            health_check_retries: 3,
            health_check_start_period: "40s".to_string(),
            network_mode: NetworkMode::Bridge,
            pull_policy: PullPolicy::Missing,
        }
    }
}

/// Enhanced Docker service structure with intelligent defaults
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedDockerService {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<EnhancedHealthCheck>,
    pub restart: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy: Option<EnhancedDeploy>,
    pub networks: Vec<String>,
    pub pull_policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildConfig {
    pub context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedHealthCheck {
    pub test: Vec<String>,
    pub interval: String,
    pub timeout: String,
    pub retries: u32,
    pub start_period: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedDeploy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<EnhancedResources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_policy: Option<EnhancedRestartPolicy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedResources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<ResourceSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservations: Option<ResourceSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedRestartPolicy {
    pub condition: String,
    pub delay: String,
    pub max_attempts: u32,
    pub window: String,
}

/// Service type detection for intelligent defaults
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ServiceType {
    WebApp,
    Database,
    Cache,
    Proxy,
    Generic,
}

pub struct DefaultsEngine;

impl DefaultsEngine {
    /// Detect service type from image and configuration for intelligent defaults
    pub fn detect_service_type(service: &Service) -> ServiceType {
        if let Some(image) = &service.image {
            let image_lower = image.to_lowercase();
            
            // Database detection
            if image_lower.contains("postgres") || image_lower.contains("mysql") || 
               image_lower.contains("mongodb") || image_lower.contains("mariadb") {
                return ServiceType::Database;
            }
            
            // Cache detection
            if image_lower.contains("redis") || image_lower.contains("memcached") {
                return ServiceType::Cache;
            }
            
            // Proxy detection
            if image_lower.contains("nginx") || image_lower.contains("apache") || 
               image_lower.contains("traefik") || image_lower.contains("haproxy") {
                return ServiceType::Proxy;
            }
            
            // Web app detection (common patterns)
            if image_lower.contains("node") || image_lower.contains("python") || 
               image_lower.contains("php") || image_lower.contains("ruby") ||
               image_lower.contains("java") || image_lower.contains("go") {
                return ServiceType::WebApp;
            }
        }
        
        ServiceType::Generic
    }
    
    /// Get optimized defaults based on service type
    pub fn get_defaults_for_type(service_type: ServiceType) -> ServiceDefaults {
        match service_type {
            ServiceType::Database => ServiceDefaults {
                restart_policy: RestartPolicy::Always,
                health_check_interval: "10s".to_string(),
                health_check_timeout: "5s".to_string(),
                health_check_retries: 5,
                health_check_start_period: "60s".to_string(),
                network_mode: NetworkMode::Bridge,
                pull_policy: PullPolicy::Missing,
            },
            ServiceType::Cache => ServiceDefaults {
                restart_policy: RestartPolicy::Always,
                health_check_interval: "15s".to_string(),
                health_check_timeout: "3s".to_string(),
                health_check_retries: 3,
                health_check_start_period: "20s".to_string(),
                network_mode: NetworkMode::Bridge,
                pull_policy: PullPolicy::Missing,
            },
            ServiceType::Proxy => ServiceDefaults {
                restart_policy: RestartPolicy::Always,
                health_check_interval: "20s".to_string(),
                health_check_timeout: "5s".to_string(),
                health_check_retries: 3,
                health_check_start_period: "30s".to_string(),
                network_mode: NetworkMode::Bridge,
                pull_policy: PullPolicy::Missing,
            },
            ServiceType::WebApp => ServiceDefaults {
                restart_policy: RestartPolicy::UnlessStopped,
                health_check_interval: "30s".to_string(),
                health_check_timeout: "10s".to_string(),
                health_check_retries: 3,
                health_check_start_period: "40s".to_string(),
                network_mode: NetworkMode::Bridge,
                pull_policy: PullPolicy::Missing,
            },
            ServiceType::Generic => ServiceDefaults::default(),
        }
    }
    
    /// Create enhanced Docker service with intelligent defaults
    pub fn create_enhanced_service(
        service: &Service, 
        network_name: &str,
        project_name: &str
    ) -> EnhancedDockerService {
        let service_type = Self::detect_service_type(service);
        let defaults = Self::get_defaults_for_type(service_type);
        
        let build_config = Self::create_build_config(service, project_name);
        let mut enhanced_service = EnhancedDockerService {
            // If we have build config with args, don't use image (build takes precedence)
            image: if build_config.is_some() && service.build_args.is_some() { 
                None 
            } else { 
                service.image.clone() 
            },
            build: build_config,
            container_name: Some(format!("{}_{}", project_name, service.name)),
            ports: Self::convert_ports(&service.ports),
            environment: Self::convert_environment(&service.environment),
            command: service.command.clone(),
            volumes: Self::convert_volumes(&service.volumes),
            depends_on: if service.depends_on.is_empty() { 
                None 
            } else { 
                Some(service.depends_on.clone()) 
            },
            healthcheck: Self::convert_healthcheck(&service.health_check, &defaults),
            restart: Self::convert_restart_policy(&service.restart, &defaults),
            deploy: Self::convert_deploy(&service.resources),
            networks: vec![network_name.to_string()],
            pull_policy: Self::convert_pull_policy(&defaults.pull_policy),
            labels: Some(Self::generate_labels(project_name, &service.name, service_type)),
        };
        
        // Optimize container name for readability
        enhanced_service.container_name = Some(
            format!("{}-{}", project_name.to_lowercase().replace("_", "-"), service.name)
        );
        
        enhanced_service
    }
    
    /// Create build configuration - prefer Dockerfile over image when no image is specified
    fn create_build_config(service: &Service, _project_name: &str) -> Option<BuildConfig> {
        // If no image is specified OR if build_args are provided, use build configuration
        if service.image.is_none() || service.build_args.is_some() {
            Some(BuildConfig {
                context: ".".to_string(), // Current directory
                dockerfile: Some("Dockerfile".to_string()), // Default Dockerfile name
                args: service.build_args.clone(), // Include build args from service
            })
        } else {
            None // Use image instead of build
        }
    }
    
    fn convert_ports(ports: &[PortMapping]) -> Option<Vec<String>> {
        if ports.is_empty() {
            return None;
        }
        
        let port_strings: Vec<String> = ports
            .iter()
            .map(|p| match p.protocol {
                Protocol::Tcp => format!("{}:{}", p.host_port, p.container_port),
                Protocol::Udp => format!("{}:{}/udp", p.host_port, p.container_port),
            })
            .collect();
        
        Some(port_strings)
    }
    
    fn convert_environment(env_vars: &[EnvironmentVariable]) -> Option<Vec<String>> {
        if env_vars.is_empty() {
            return None;
        }
        
        let mut env_list = Vec::new();
        for env_var in env_vars {
            match env_var {
                EnvironmentVariable::Template(var_name) => {
                    env_list.push(format!("{}=${{{}}}", var_name, var_name));
                }
                EnvironmentVariable::Literal(value) => {
                    // If it's already in KEY=VALUE format, use as-is
                    // Otherwise, treat as a standalone value
                    if value.contains('=') {
                        env_list.push(value.clone());
                    } else {
                        env_list.push(format!("VALUE={}", value));
                    }
                }
            }
        }
        
        Some(env_list)
    }
    
    fn convert_volumes(volumes: &[VolumeMapping]) -> Option<Vec<String>> {
        if volumes.is_empty() {
            return None;
        }
        
        let volume_strings: Vec<String> = volumes
            .iter()
            .map(|v| {
                let mut volume_str = format!("{}:{}", v.host_path, v.container_path);
                if !v.options.is_empty() {
                    volume_str.push(':');
                    volume_str.push_str(&v.options.join(","));
                }
                volume_str
            })
            .collect();
        
        Some(volume_strings)
    }
    
    fn convert_healthcheck(
        health_check: &Option<String>, 
        defaults: &ServiceDefaults
    ) -> Option<EnhancedHealthCheck> {
        health_check.as_ref().map(|cmd| EnhancedHealthCheck {
            test: vec!["CMD-SHELL".to_string(), cmd.clone()],
            interval: defaults.health_check_interval.clone(),
            timeout: defaults.health_check_timeout.clone(),
            retries: defaults.health_check_retries,
            start_period: defaults.health_check_start_period.clone(),
        })
    }
    
    fn convert_restart_policy(
        restart: &Option<RestartPolicy>, 
        defaults: &ServiceDefaults
    ) -> String {
        match restart.as_ref().unwrap_or(&defaults.restart_policy) {
            RestartPolicy::Always => "always".to_string(),
            RestartPolicy::UnlessStopped => "unless-stopped".to_string(),
            RestartPolicy::OnFailure => "on-failure".to_string(),
            RestartPolicy::No => "no".to_string(),
        }
    }
    
    fn convert_deploy(resources: &Option<ResourceLimits>) -> Option<EnhancedDeploy> {
        resources.as_ref().map(|res| EnhancedDeploy {
            resources: Some(EnhancedResources {
                limits: Some(ResourceSpec {
                    cpus: Some(res.cpu.clone()),
                    memory: Some(res.memory.clone()),
                }),
                reservations: None,
            }),
            restart_policy: Some(EnhancedRestartPolicy {
                condition: "on-failure".to_string(),
                delay: "5s".to_string(),
                max_attempts: 3,
                window: "120s".to_string(),
            }),
        })
    }
    
    fn convert_pull_policy(pull_policy: &PullPolicy) -> String {
        match pull_policy {
            PullPolicy::Always => "always".to_string(),
            PullPolicy::Missing => "missing".to_string(),
            PullPolicy::Never => "never".to_string(),
        }
    }
    
    fn generate_labels(project_name: &str, service_name: &str, service_type: ServiceType) -> HashMap<String, String> {
        let mut labels = HashMap::new();
        labels.insert("athena.project".to_string(), project_name.to_string());
        labels.insert("athena.service".to_string(), service_name.to_string());
        labels.insert("athena.type".to_string(), format!("{:?}", service_type).to_lowercase());
        labels.insert("athena.generated".to_string(), chrono::Utc::now().format("%Y-%m-%d").to_string());
        labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_type_detection() {
        let mut service = Service::new("test".to_string());
        
        service.image = Some("postgres:15".to_string());
        assert_eq!(DefaultsEngine::detect_service_type(&service), ServiceType::Database);
        
        service.image = Some("redis:7-alpine".to_string());
        assert_eq!(DefaultsEngine::detect_service_type(&service), ServiceType::Cache);
        
        service.image = Some("nginx:alpine".to_string());
        assert_eq!(DefaultsEngine::detect_service_type(&service), ServiceType::Proxy);
        
        service.image = Some("node:18-alpine".to_string());
        assert_eq!(DefaultsEngine::detect_service_type(&service), ServiceType::WebApp);
    }
    
    #[test]
    fn test_enhanced_service_creation() {
        let mut service = Service::new("backend".to_string());
        service.image = Some("python:3.11-slim".to_string());
        service.ports.push(PortMapping {
            host_port: 8000,
            container_port: 8000,
            protocol: Protocol::Tcp,
        });
        
        let enhanced = DefaultsEngine::create_enhanced_service(
            &service, 
            "test_network", 
            "test_project"
        );
        
        assert_eq!(enhanced.image, Some("python:3.11-slim".to_string()));
        assert_eq!(enhanced.restart, "unless-stopped");
        assert_eq!(enhanced.networks, vec!["test_network"]);
        assert!(enhanced.labels.is_some());
        assert!(enhanced.ports.is_some());
    }

    #[test]
    fn test_build_args_service_creation() {
        let mut service = Service::new("api".to_string());
        let mut build_args = HashMap::new();
        build_args.insert("NODE_VERSION".to_string(), "20".to_string());
        build_args.insert("BUILD_ENV".to_string(), "production".to_string());
        service.build_args = Some(build_args.clone());
        
        let enhanced = DefaultsEngine::create_enhanced_service(
            &service, 
            "test_network", 
            "test_project"
        );
        
        // Should use build instead of image when build_args are provided
        assert!(enhanced.image.is_none());
        assert!(enhanced.build.is_some());
        
        let build_config = enhanced.build.unwrap();
        assert_eq!(build_config.context, ".");
        assert_eq!(build_config.dockerfile, Some("Dockerfile".to_string()));
        assert_eq!(build_config.args, Some(build_args));
    }

    #[test]
    fn test_build_args_with_image_uses_build() {
        let mut service = Service::new("api".to_string());
        service.image = Some("node:18".to_string());
        
        let mut build_args = HashMap::new();
        build_args.insert("NODE_ENV".to_string(), "development".to_string());
        service.build_args = Some(build_args.clone());
        
        let enhanced = DefaultsEngine::create_enhanced_service(
            &service, 
            "test_network", 
            "test_project"
        );
        
        // Build args should take precedence over image
        assert!(enhanced.image.is_none());
        assert!(enhanced.build.is_some());
        
        let build_config = enhanced.build.unwrap();
        assert_eq!(build_config.args, Some(build_args));
    }
}