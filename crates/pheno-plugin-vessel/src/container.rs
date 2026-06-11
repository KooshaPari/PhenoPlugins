//! # phenotype-vessel
//!
//! Container lifecycle management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Container status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}

impl std::fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerStatus::Created => write!(f, "created"),
            ContainerStatus::Running => write!(f, "running"),
            ContainerStatus::Paused => write!(f, "paused"),
            ContainerStatus::Restarting => write!(f, "restarting"),
            ContainerStatus::Removing => write!(f, "removing"),
            ContainerStatus::Exited => write!(f, "exited"),
            ContainerStatus::Dead => write!(f, "dead"),
        }
    }
}

/// Container representation
#[derive(Debug, Clone)]
pub struct Container {
    /// Container ID
    pub id: String,
    /// Container name
    pub name: String,
    /// Image used
    pub image: String,
    /// Current status
    pub status: ContainerStatus,
}

/// Configuration for creating a container
#[derive(Debug, Clone, Default)]
pub struct ContainerConfig {
    /// Image to use
    pub image: String,
    /// Container name
    pub name: Option<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Command to run
    pub cmd: Option<Vec<String>>,
    /// Working directory
    pub workdir: Option<String>,
}

impl Container {
    /// Check if container is running
    pub fn is_running(&self) -> bool {
        self.status == ContainerStatus::Running
    }

    /// Check if container is stopped
    pub fn is_stopped(&self) -> bool {
        matches!(self.status, ContainerStatus::Exited | ContainerStatus::Dead)
    }

    /// Short ID (first 12 characters)
    pub fn short_id(&self) -> &str {
        &self.id[..12.min(self.id.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_status_display() {
        assert_eq!(ContainerStatus::Running.to_string(), "running");
        assert_eq!(ContainerStatus::Exited.to_string(), "exited");
    }

    #[test]
    fn test_container_short_id() {
        let container = Container {
            id: "abc123def456".to_string(),
            name: "test".to_string(),
            image: "nginx".to_string(),
            status: ContainerStatus::Running,
        };
        assert_eq!(container.short_id(), "abc123def456");
    }

    #[test]
    fn test_container_is_running() {
        let container = Container {
            id: "123".to_string(),
            name: "test".to_string(),
            image: "nginx".to_string(),
            status: ContainerStatus::Running,
        };
        assert!(container.is_running());
        assert!(!container.is_stopped());
    }

    #[test]
    fn test_container_status_all_displays() {
        assert_eq!(ContainerStatus::Created.to_string(), "created");
        assert_eq!(ContainerStatus::Running.to_string(), "running");
        assert_eq!(ContainerStatus::Paused.to_string(), "paused");
        assert_eq!(ContainerStatus::Restarting.to_string(), "restarting");
        assert_eq!(ContainerStatus::Removing.to_string(), "removing");
        assert_eq!(ContainerStatus::Exited.to_string(), "exited");
        assert_eq!(ContainerStatus::Dead.to_string(), "dead");
    }

    #[test]
    fn test_container_status_serde_roundtrip() {
        let variants = [
            ContainerStatus::Created,
            ContainerStatus::Running,
            ContainerStatus::Paused,
            ContainerStatus::Restarting,
            ContainerStatus::Removing,
            ContainerStatus::Exited,
            ContainerStatus::Dead,
        ];
        for v in variants {
            let serialized = serde_yaml::to_string(&v).expect("serialize");
            let back: ContainerStatus =
                serde_yaml::from_str(&serialized).expect("deserialize");
            assert_eq!(back, v);
        }
    }

    #[test]
    fn test_container_is_stopped() {
        let mk = |id: &str, status: ContainerStatus| Container {
            id: id.to_string(),
            name: "n".to_string(),
            image: "i".to_string(),
            status,
        };

        // Exited and Dead are stopped.
        assert!(mk("1", ContainerStatus::Exited).is_stopped());
        assert!(!mk("2", ContainerStatus::Exited).is_running());
        assert!(mk("3", ContainerStatus::Dead).is_stopped());
        assert!(!mk("4", ContainerStatus::Dead).is_running());

        // All other statuses are not stopped.
        assert!(!mk("5", ContainerStatus::Running).is_stopped());
        assert!(!mk("6", ContainerStatus::Created).is_stopped());
        assert!(!mk("7", ContainerStatus::Paused).is_stopped());
    }

    #[test]
    fn test_container_config_defaults() {
        let cfg = ContainerConfig::default();
        assert_eq!(cfg.image, "");
        assert!(cfg.name.is_none());
        assert!(cfg.env.is_empty());
        assert!(cfg.cmd.is_none());
        assert!(cfg.workdir.is_none());
    }

    #[test]
    fn test_container_config_fields() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());

        let cfg = ContainerConfig {
            image: "nginx:latest".to_string(),
            name: Some("web".to_string()),
            env,
            cmd: Some(vec![
                "nginx".to_string(),
                "-g".to_string(),
                "daemon off;".to_string(),
            ]),
            workdir: Some("/app".to_string()),
        };

        assert_eq!(cfg.image, "nginx:latest");
        assert_eq!(cfg.name.as_deref(), Some("web"));
        assert_eq!(cfg.env.get("FOO").map(String::as_str), Some("bar"));
        assert_eq!(cfg.env.get("BAZ").map(String::as_str), Some("qux"));
        assert_eq!(cfg.env.len(), 2);
        assert_eq!(
            cfg.cmd.as_deref(),
            Some(["nginx", "-g", "daemon off;"].map(String::from).as_slice())
        );
        assert_eq!(cfg.workdir.as_deref(), Some("/app"));
    }

    #[test]
    fn test_container_status_equality() {
        assert_eq!(ContainerStatus::Running, ContainerStatus::Running);
        assert_eq!(ContainerStatus::Exited, ContainerStatus::Exited);
        assert_eq!(ContainerStatus::Created, ContainerStatus::Created);
        assert_eq!(ContainerStatus::Paused, ContainerStatus::Paused);
        assert_eq!(ContainerStatus::Restarting, ContainerStatus::Restarting);
        assert_eq!(ContainerStatus::Removing, ContainerStatus::Removing);
        assert_eq!(ContainerStatus::Dead, ContainerStatus::Dead);

        assert_ne!(ContainerStatus::Running, ContainerStatus::Exited);
        assert_ne!(ContainerStatus::Created, ContainerStatus::Dead);
        assert_ne!(ContainerStatus::Paused, ContainerStatus::Restarting);
        assert_ne!(ContainerStatus::Removing, ContainerStatus::Exited);
    }
}
