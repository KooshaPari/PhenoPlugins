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
            let back: ContainerStatus = serde_yaml::from_str(&serialized).expect("deserialize");
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
            cmd: Some(vec!["nginx".to_string(), "-g".to_string(), "daemon off;".to_string()]),
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

    #[test]
    fn test_container_construction_all_fields() {
        let container = Container {
            id: "abc123def456ghi789".to_string(),
            name: "web".to_string(),
            image: "nginx:1.25".to_string(),
            status: ContainerStatus::Paused,
        };
        assert_eq!(container.id, "abc123def456ghi789");
        assert_eq!(container.name, "web");
        assert_eq!(container.image, "nginx:1.25");
        assert_eq!(container.status, ContainerStatus::Paused);
    }

    #[test]
    fn test_container_is_running_false_for_other_statuses() {
        let mk = |status: ContainerStatus| Container {
            id: "id".to_string(),
            name: "n".to_string(),
            image: "i".to_string(),
            status,
        };

        for status in [
            ContainerStatus::Created,
            ContainerStatus::Paused,
            ContainerStatus::Restarting,
            ContainerStatus::Removing,
            ContainerStatus::Exited,
            ContainerStatus::Dead,
        ] {
            let c = mk(status);
            assert!(
                !c.is_running(),
                "expected {status:?} to not be running, but is_running() returned true"
            );
        }
    }

    #[test]
    fn test_container_short_id_truncation() {
        // An id longer than 12 characters is truncated to the first 12.
        let container = Container {
            id: "abcdefghijklmnopqrstuvwxyz".to_string(),
            name: "n".to_string(),
            image: "i".to_string(),
            status: ContainerStatus::Running,
        };
        assert_eq!(container.short_id(), "abcdefghijkl");
        assert_eq!(container.short_id().len(), 12);
    }

    #[test]
    fn test_container_short_id_exact_length() {
        // An id of exactly 12 characters is returned verbatim, no truncation
        // needed.
        let container = Container {
            id: "abc123def456".to_string(),
            name: "n".to_string(),
            image: "i".to_string(),
            status: ContainerStatus::Running,
        };
        assert_eq!(container.short_id(), "abc123def456");
        assert_eq!(container.short_id().len(), 12);
    }

    #[test]
    fn test_container_short_id_short_id() {
        // An id shorter than 12 characters is returned in full.
        let container = Container {
            id: "abc".to_string(),
            name: "n".to_string(),
            image: "i".to_string(),
            status: ContainerStatus::Running,
        };
        assert_eq!(container.short_id(), "abc");
        assert_eq!(container.short_id().len(), 3);
    }

    #[test]
    fn test_container_status_copy() {
        // `ContainerStatus` derives `Copy`, so the original remains usable
        // after binding a copy to a new variable.
        let a = ContainerStatus::Running;
        let b = a;
        assert_eq!(a, b);
        assert_eq!(a, ContainerStatus::Running);
        assert_eq!(b, ContainerStatus::Running);
    }

    #[test]
    fn test_container_config_clone() {
        // `ContainerConfig` derives `Clone`, so cloning must yield a value
        // with equal fields.
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());

        let original = ContainerConfig {
            image: "redis:7".to_string(),
            name: Some("cache".to_string()),
            env: env.clone(),
            cmd: Some(vec!["redis-server".to_string()]),
            workdir: Some("/data".to_string()),
        };
        let cloned = original.clone();

        assert_eq!(original.image, cloned.image);
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.workdir, cloned.workdir);
        assert_eq!(original.cmd, cloned.cmd);
        assert_eq!(original.env, cloned.env);
        assert_eq!(original.env.len(), cloned.env.len());
        assert_eq!(
            original.env.get("FOO").map(String::as_str),
            cloned.env.get("FOO").map(String::as_str)
        );
    }

    #[test]
    fn test_container_status_paused_is_not_stopped() {
        // `Paused` is neither `Running` nor `Stopped` (which is defined as
        // `Exited | Dead`). It is a third distinct lifecycle state.
        let container = Container {
            id: "id".to_string(),
            name: "n".to_string(),
            image: "i".to_string(),
            status: ContainerStatus::Paused,
        };
        assert!(!container.is_running(), "Paused must not be running");
        assert!(!container.is_stopped(), "Paused must not be stopped");
        assert_eq!(container.status, ContainerStatus::Paused);
        assert_ne!(container.status, ContainerStatus::Running);
        assert_ne!(container.status, ContainerStatus::Exited);
        assert_ne!(container.status, ContainerStatus::Dead);
    }

    #[test]
    fn test_container_debug_format() {
        // `Container` derives `Debug`, so its `{:?}` representation must
        // include the struct name and the field values.
        let c = Container {
            id: "abc".to_string(),
            name: "web".to_string(),
            image: "nginx".to_string(),
            status: ContainerStatus::Running,
        };
        let dbg = format!("{:?}", c);
        assert!(dbg.contains("Container"), "debug must include type name: {dbg}");
        assert!(dbg.contains("abc"), "debug must include id: {dbg}");
        assert!(dbg.contains("web"), "debug must include name: {dbg}");
        assert!(
            dbg.contains("nginx"),
            "debug must include image: {dbg}"
        );
    }

    #[test]
    fn test_container_clone() {
        // `Container` derives `Clone`, so cloning must yield a value with
        // equal fields.
        let original = Container {
            id: "abc123def456".to_string(),
            name: "web".to_string(),
            image: "nginx:1.25".to_string(),
            status: ContainerStatus::Paused,
        };
        let cloned = original.clone();
        assert_eq!(cloned.id, original.id);
        assert_eq!(cloned.name, original.name);
        assert_eq!(cloned.image, original.image);
        assert_eq!(cloned.status, original.status);
    }

    #[test]
    fn test_container_is_running_true() {
        // A container with `Running` status must report `is_running() == true`.
        let c = Container {
            id: "id".to_string(),
            name: "n".to_string(),
            image: "i".to_string(),
            status: ContainerStatus::Running,
        };
        assert!(c.is_running());
    }

    #[test]
    fn test_container_is_stopped_true() {
        // A container with `Exited` status must report `is_stopped() == true`.
        let c = Container {
            id: "id".to_string(),
            name: "n".to_string(),
            image: "i".to_string(),
            status: ContainerStatus::Exited,
        };
        assert!(c.is_stopped());
    }

    #[test]
    fn test_container_config_debug() {
        // `ContainerConfig` derives `Debug`, so its `{:?}` representation
        // must include the struct name and field values.
        let cfg = ContainerConfig {
            image: "nginx:1.25".to_string(),
            name: Some("web".to_string()),
            env: HashMap::new(),
            cmd: None,
            workdir: None,
        };
        let dbg = format!("{:?}", cfg);
        assert!(
            dbg.contains("ContainerConfig"),
            "debug must include type name: {dbg}"
        );
        assert!(
            dbg.contains("nginx:1.25"),
            "debug must include image: {dbg}"
        );
        assert!(dbg.contains("web"), "debug must include name: {dbg}");
    }

    #[test]
    fn test_container_config_with_env() {
        // Environment variables passed into `ContainerConfig` must be
        // preserved verbatim.
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "VAL".to_string());
        env.insert("PATH".to_string(), "/usr/local/bin".to_string());

        let cfg = ContainerConfig {
            image: "redis:7".to_string(),
            name: Some("cache".to_string()),
            env: env.clone(),
            cmd: None,
            workdir: None,
        };

        assert_eq!(cfg.env.len(), 2);
        assert_eq!(cfg.env.get("KEY").map(String::as_str), Some("VAL"));
        assert_eq!(
            cfg.env.get("PATH").map(String::as_str),
            Some("/usr/local/bin")
        );
        // The `env` we passed in must be unchanged (no silent copy/move).
        assert_eq!(env.len(), 2);
    }

    #[test]
    fn test_container_config_with_command() {
        // The `cmd` field must hold the exact command list that was provided.
        let cfg = ContainerConfig {
            image: "alpine:3.20".to_string(),
            name: None,
            env: HashMap::new(),
            cmd: Some(vec!["sh".to_string(), "-c".to_string(), "echo hi".to_string()]),
            workdir: Some("/tmp".to_string()),
        };

        let cmd = cfg.cmd.as_ref().expect("cmd should be Some");
        assert_eq!(cmd.len(), 3);
        assert_eq!(cmd[0], "sh");
        assert_eq!(cmd[1], "-c");
        assert_eq!(cmd[2], "echo hi");
        assert_eq!(cfg.workdir.as_deref(), Some("/tmp"));
        assert_eq!(cfg.image, "alpine:3.20");
    }
}
