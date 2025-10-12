use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthenaFile {
    pub deployment: Option<DeploymentSection>,
    pub environment: Option<EnvironmentSection>,
    pub services: ServicesSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSection {
    pub deployment_id: String,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSection {
    pub networks: Vec<NetworkDefinition>,
    pub volumes: Vec<VolumeDefinition>,
    pub secrets: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDefinition {
    pub name: String,
    pub driver: Option<NetworkDriver>,
    pub attachable: Option<bool>,
    pub encrypted: Option<bool>,
    pub ingress: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkDriver {
    Bridge,
    Overlay,
    Host,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDefinition {
    pub name: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesSection {
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub image: Option<String>,
    pub ports: Vec<PortMapping>,
    pub environment: Vec<EnvironmentVariable>,
    pub command: Option<String>,
    pub volumes: Vec<VolumeMapping>,
    pub depends_on: Vec<String>,
    pub health_check: Option<String>,
    pub restart: Option<RestartPolicy>,
    pub resources: Option<ResourceLimits>,
    pub build_args: Option<HashMap<String, String>>,
    pub swarm_config: Option<SwarmConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Protocol {
    #[default]
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnvironmentVariable {
    Template(String),     // {{VAR_NAME}}
    Literal(String),      // "actual value"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    Always,
    UnlessStopped,
    OnFailure,
    No,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub replicas: Option<u32>,
    pub update_config: Option<UpdateConfig>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub parallelism: Option<u32>,
    pub delay: Option<String>,
    pub failure_action: Option<FailureAction>,
    pub monitor: Option<String>,
    pub max_failure_ratio: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureAction {
    Continue,
    Pause,
    Rollback,
}

impl Default for AthenaFile {
    fn default() -> Self {
        Self::new()
    }
}

impl AthenaFile {
    pub fn new() -> Self {
        Self {
            deployment: None,
            environment: None,
            services: ServicesSection {
                services: Vec::new(),
            },
        }
    }

    pub fn get_project_name(&self) -> String {
        self.deployment
            .as_ref()
            .map(|d| d.deployment_id.clone())
            .unwrap_or_else(|| "athena-project".to_string())
    }

    pub fn get_network_name(&self) -> String {
        self.environment
            .as_ref()
            .and_then(|e| {
                e.networks.first().map(|net| net.name.clone())
            })
            .unwrap_or_else(|| format!("{}_network", self.get_project_name().to_lowercase()))
    }

    #[allow(dead_code)]
    pub fn get_networks(&self) -> Vec<&NetworkDefinition> {
        self.environment
            .as_ref()
            .map(|e| e.networks.iter().collect())
            .unwrap_or_default()
    }
}

impl Service {
    pub fn new(name: String) -> Self {
        Self {
            name,
            image: None,
            ports: Vec::new(),
            environment: Vec::new(),
            command: None,
            volumes: Vec::new(),
            depends_on: Vec::new(),
            health_check: None,
            restart: None,
            resources: None,
            build_args: None,
            swarm_config: None,
        }
    }
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SwarmConfig {
    pub fn new() -> Self {
        Self {
            replicas: None,
            update_config: None,
            labels: None,
        }
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateConfig {
    pub fn new() -> Self {
        Self {
            parallelism: None,
            delay: None,
            failure_action: None,
            monitor: None,
            max_failure_ratio: None,
        }
    }
}

impl NetworkDefinition {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Self {
            name,
            driver: None,
            attachable: None,
            encrypted: None,
            ingress: None,
        }
    }
}