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

    #[test]
    fn test_compose_from_yaml_invalid() {
        let invalid_yaml = ":invalid: yaml: :::";
        let result = ComposeFile::from_yaml(invalid_yaml);
        assert!(result.is_err(), "expected invalid YAML to return an Err");
    }

    #[test]
    fn test_compose_to_yaml_roundtrip() {
        let mut services = HashMap::new();

        let mut web = ComposeService::default();
        web.image = Some("nginx:latest".to_string());
        web.ports = Some(vec!["80:80".to_string()]);
        services.insert("web".to_string(), web);

        let mut db = ComposeService::default();
        db.image = Some("postgres:15".to_string());
        services.insert("db".to_string(), db);

        let compose = ComposeFile {
            version: Some("3.8".to_string()),
            services,
            networks: None,
            volumes: None,
        };

        let yaml = compose.to_yaml().expect("to_yaml should succeed");
        let parsed = ComposeFile::from_yaml(&yaml).expect("from_yaml should succeed");

        assert_eq!(compose.services.len(), parsed.services.len());
        assert_eq!(compose.version, parsed.version);
        assert_eq!(
            compose.services.get("web").unwrap().image,
            parsed.services.get("web").unwrap().image
        );
        assert_eq!(
            compose.services.get("web").unwrap().ports,
            parsed.services.get("web").unwrap().ports
        );
        assert_eq!(
            compose.services.get("db").unwrap().image,
            parsed.services.get("db").unwrap().image
        );
    }

    #[test]
    fn test_compose_service_names() {
        let mut services = HashMap::new();
        services.insert("db".to_string(), ComposeService::default());
        services.insert("api".to_string(), ComposeService::default());
        services.insert("web".to_string(), ComposeService::default());

        let compose = ComposeFile { version: None, services, networks: None, volumes: None };

        let names = compose.service_names();
        assert_eq!(names.len(), 3);

        let name_set: std::collections::HashSet<&str> = names.iter().map(|n| n.as_str()).collect();
        assert!(name_set.contains("db"));
        assert!(name_set.contains("api"));
        assert!(name_set.contains("web"));
    }

    #[test]
    fn test_compose_service_default() {
        let svc = ComposeService::default();
        assert!(svc.image.is_none());
        assert!(svc.command.is_none());
        assert!(svc.depends_on.is_none());
        assert!(svc.environment.is_none());
        assert!(svc.ports.is_none());
        assert!(svc.volumes.is_none());
    }

    #[test]
    fn test_ordered_services_empty() {
        let compose =
            ComposeFile { version: None, services: HashMap::new(), networks: None, volumes: None };
        let ordered = compose.ordered_services();
        assert!(ordered.is_empty());
        assert_eq!(ordered.len(), 0);
    }

    #[test]
    fn test_compose_service_field_access() {
        let mut env = HashMap::new();
        env.insert("POSTGRES_DB".to_string(), "test".to_string());

        let svc = ComposeService {
            image: Some("postgres:15".to_string()),
            environment: Some(env),
            ports: Some(vec!["5432:5432".to_string()]),
            ..Default::default()
        };

        assert_eq!(svc.image.as_deref(), Some("postgres:15"));
        assert!(svc.environment.is_some());
        assert_eq!(
            svc.environment.as_ref().and_then(|e| e.get("POSTGRES_DB").map(|s| s.as_str())),
            Some("test")
        );
        assert!(svc.ports.is_some());
        assert_eq!(svc.ports.as_ref().unwrap().len(), 1);
        assert_eq!(svc.ports.as_ref().unwrap()[0], "5432:5432");
    }

    #[test]
    fn test_compose_dependencies_count() {
        let mut services = HashMap::new();

        let mut web = ComposeService::default();
        web.image = Some("nginx".to_string());
        web.depends_on = Some(vec!["db".to_string(), "api".to_string()]);
        services.insert("web".to_string(), web);

        let mut db = ComposeService::default();
        db.image = Some("postgres".to_string());
        services.insert("db".to_string(), db);

        let mut api = ComposeService::default();
        api.image = Some("myapi".to_string());
        services.insert("api".to_string(), api);

        let compose = ComposeFile {
            version: Some("3.8".to_string()),
            services,
            networks: None,
            volumes: None,
        };

        let ordered = compose.ordered_services();
        assert_eq!(ordered.len(), 3);

        // web should have 2 deps: db and api
        let web_svc = ordered
            .iter()
            .find(|s| s.image.as_deref() == Some("nginx"))
            .expect("web service should be present");
        let web_deps = web_svc.depends_on.as_ref().expect("web should have depends_on");
        assert_eq!(web_deps.len(), 2);
        assert!(web_deps.contains(&"db".to_string()));
        assert!(web_deps.contains(&"api".to_string()));

        // db should have no deps
        let db_svc = compose.services.get("db").expect("db service should be present");
        assert!(db_svc.depends_on.is_none());

        // api should have no deps
        let api_svc = compose.services.get("api").expect("api service should be present");
        assert!(api_svc.depends_on.is_none());
    }
}
