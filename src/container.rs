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
}
