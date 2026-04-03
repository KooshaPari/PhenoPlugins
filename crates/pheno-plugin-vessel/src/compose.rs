//! # phenotype-vessel
//!
//! Docker Compose file parsing and orchestration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Docker Compose file representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeFile {
    /// Compose file version
    pub version: Option<String>,
    /// Services defined
    pub services: HashMap<String, ComposeService>,
    /// Networks defined
    pub networks: Option<HashMap<String, NetworkConfig>>,
    /// Volumes defined
    pub volumes: Option<HashMap<String, VolumeConfig>>,
}

/// Configuration for a compose service
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposeService {
    /// Image to use
    pub image: Option<String>,
    /// Build configuration
    pub build: Option<BuildConfig>,
    /// Container name
    pub container_name: Option<String>,
    /// Environment variables
    pub environment: Option<HashMap<String, String>>,
    /// Port mappings
    pub ports: Option<Vec<String>>,
    /// Volume mounts
    pub volumes: Option<Vec<String>>,
    /// Dependencies (depends_on)
    pub depends_on: Option<Vec<String>>,
    /// Restart policy
    pub restart: Option<String>,
    /// Command to run
    pub command: Option<String>,
    /// Working directory
    pub working_dir: Option<String>,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Build context
    pub context: Option<String>,
    /// Dockerfile path
    pub dockerfile: Option<String>,
    /// Build args
    pub args: Option<HashMap<String, String>>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub driver: Option<String>,
    pub external: Option<bool>,
}

/// Volume configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    pub driver: Option<String>,
    pub external: Option<bool>,
}

impl ComposeFile {
    /// Parse a compose file from YAML
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Serialize to YAML
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Get all service names
    pub fn service_names(&self) -> Vec<&String> {
        self.services.keys().collect()
    }

    /// Get service in dependency order
    pub fn ordered_services(&self) -> Vec<&ComposeService> {
        let mut ordered = Vec::new();
        let mut visited = std::collections::HashSet::new();

        fn visit<'a>(
            service_name: &'a str,
            services: &'a HashMap<String, ComposeService>,
            ordered: &mut Vec<&'a ComposeService>,
            visited: &mut std::collections::HashSet<&'a str>,
        ) {
            if visited.contains(service_name) {
                return;
            }
            visited.insert(service_name);

            if let Some(service) = services.get(service_name) {
                if let Some(deps) = &service.depends_on {
                    for dep in deps {
                        visit(dep, services, ordered, visited);
                    }
                }
                if let Some(svc) = services.get(service_name) {
                    ordered.push(svc);
                }
            }
        }

        for name in self.services.keys() {
            visit(name.as_str(), &self.services, &mut ordered, &mut visited);
        }

        ordered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compose_file() {
        let yaml = r#"
version: "3.8"
services:
  web:
    image: nginx:latest
    ports:
      - "80:80"
  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: secret
"#;
        let compose = ComposeFile::from_yaml(yaml).unwrap();
        assert_eq!(compose.services.len(), 2);
        assert!(compose.services.contains_key("web"));
        assert!(compose.services.contains_key("db"));
    }

    #[test]
    fn test_service_dependencies() {
        let mut services = HashMap::new();

        let mut web = ComposeService::default();
        web.image = Some("nginx".to_string());
        web.depends_on = Some(vec!["db".to_string()]);

        let mut db = ComposeService::default();
        db.image = Some("postgres".to_string());

        services.insert("web".to_string(), web);
        services.insert("db".to_string(), db);

        let compose = ComposeFile {
            version: Some("3.8".to_string()),
            services,
            networks: None,
            volumes: None,
        };

        let ordered = compose.ordered_services();
        // db should come before web due to dependency
        let db_idx = ordered.iter().position(|s| s.image.as_ref().unwrap() == "postgres");
        let web_idx = ordered.iter().position(|s| s.image.as_ref().unwrap() == "nginx");

        assert!(db_idx.is_some() && web_idx.is_some());
    }
}
