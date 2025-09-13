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
    pub network_name: Option<String>,
    pub volumes: Vec<VolumeDefinition>,
    pub secrets: HashMap<String, String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Tcp
    }
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
            .and_then(|e| e.network_name.as_ref())
            .map(|n| n.clone())
            .unwrap_or_else(|| format!("{}_network", self.get_project_name().to_lowercase()))
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
        }
    }
}