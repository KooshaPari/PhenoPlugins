//! # phenotype-vessel
//!
//! Container runtime trait and implementations.

use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for container runtime implementations
#[async_trait]
pub trait ContainerRuntime: Send + Sync {
    /// Get the runtime name
    fn name(&self) -> &str;

    /// Check if the runtime is available
    async fn is_available(&self) -> bool;

    /// List all containers
    async fn list_containers(&self) -> Result<Vec<ContainerInfo>, String>;

    /// Pull an image
    async fn pull_image(&self, image: &str) -> Result<(), String>;

    /// Remove an image
    async fn remove_image(&self, image: &str) -> Result<(), String>;

    /// Create a container
    async fn create_container(&self, config: &ContainerCreateConfig) -> Result<String, String>;

    /// Start a container
    async fn start_container(&self, id: &str) -> Result<(), String>;

    /// Stop a container
    async fn stop_container(&self, id: &str) -> Result<(), String>;

    /// Remove a container
    async fn remove_container(&self, id: &str) -> Result<(), String>;

    /// Get container logs
    async fn logs(&self, id: &str) -> Result<String, String>;
}

/// Container information from list_containers
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created: String,
}

/// Configuration for creating a container
#[derive(Debug, Clone)]
pub struct ContainerCreateConfig {
    pub image: String,
    pub name: Option<String>,
    pub env: HashMap<String, String>,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMapping>,
}

/// Port mapping for container networking
#[derive(Debug, Clone)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Protocol,
}

/// Volume mapping for persistent storage
#[derive(Debug, Clone)]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

/// Network protocol
#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Tcp,
    Udp,
}

/// Docker runtime implementation
#[derive(Debug, Clone, Default)]
pub struct DockerRuntime;

impl DockerRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContainerRuntime for DockerRuntime {
    fn name(&self) -> &str {
        "docker"
    }

    async fn is_available(&self) -> bool {
        tokio::process::Command::new("docker")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn list_containers(&self) -> Result<Vec<ContainerInfo>, String> {
        let output = tokio::process::Command::new("docker")
            .args([
                "ps",
                "-a",
                "--format",
                "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.CreatedAt}}",
            ])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let containers = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 5 {
                    Some(ContainerInfo {
                        id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        image: parts[2].to_string(),
                        status: parts[3].to_string(),
                        created: parts[4].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(containers)
    }

    async fn pull_image(&self, image: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker")
            .args(["pull", image])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn remove_image(&self, image: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker")
            .args(["rmi", image])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn create_container(&self, config: &ContainerCreateConfig) -> Result<String, String> {
        let mut args: Vec<String> = vec!["create".to_string()];

        if let Some(name) = &config.name {
            args.push("--name".to_string());
            args.push(name.clone());
        }

        for env in &config.env {
            args.push("-e".to_string());
            args.push(format!("{}={}", env.0, env.1));
        }

        for port in &config.ports {
            args.push("-p".to_string());
            args.push(format!("{}:{}", port.host_port, port.container_port));
        }

        for volume in &config.volumes {
            let mode = if volume.read_only { ":ro" } else { "" };
            args.push("-v".to_string());
            args.push(format!("{}:{}{}", volume.host_path, volume.container_path, mode));
        }

        args.push(config.image.clone());

        let output = tokio::process::Command::new("docker")
            .args(&args)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn start_container(&self, id: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker")
            .args(["start", id])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn stop_container(&self, id: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker")
            .args(["stop", id])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn remove_container(&self, id: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker")
            .args(["rm", "-f", id])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn logs(&self, id: &str) -> Result<String, String> {
        let output = tokio::process::Command::new("docker")
            .args(["logs", id])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() || output.status.code() == Some(0) {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

/// Podman runtime implementation
#[derive(Debug, Clone, Default)]
pub struct PodmanRuntime;

impl PodmanRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContainerRuntime for PodmanRuntime {
    fn name(&self) -> &str {
        "podman"
    }

    async fn is_available(&self) -> bool {
        tokio::process::Command::new("podman")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn list_containers(&self) -> Result<Vec<ContainerInfo>, String> {
        // Similar to Docker but using podman commands
        let output = tokio::process::Command::new("podman")
            .args(["ps", "-a", "--format", "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}"])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let containers = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    Some(ContainerInfo {
                        id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        image: parts[2].to_string(),
                        status: parts[3].to_string(),
                        created: String::new(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(containers)
    }

    async fn pull_image(&self, image: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("podman")
            .args(["pull", image])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn remove_image(&self, image: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("podman")
            .args(["rmi", image])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn create_container(&self, config: &ContainerCreateConfig) -> Result<String, String> {
        let mut args: Vec<String> = vec!["create".to_string()];

        if let Some(name) = &config.name {
            args.push("--name".to_string());
            args.push(name.clone());
        }

        for env in &config.env {
            args.push("-e".to_string());
            args.push(format!("{}={}", env.0, env.1));
        }

        args.push(config.image.clone());

        let output = tokio::process::Command::new("podman")
            .args(&args)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn start_container(&self, id: &str) -> Result<(), String> {
        tokio::process::Command::new("podman")
            .args(["start", id])
            .output()
            .await
            .map_err(|e| e.to_string())
            .and_then(|o| {
                if o.status.success() {
                    Ok(())
                } else {
                    Err(String::from_utf8_lossy(&o.stderr).to_string())
                }
            })
    }

    async fn stop_container(&self, id: &str) -> Result<(), String> {
        tokio::process::Command::new("podman")
            .args(["stop", id])
            .output()
            .await
            .map_err(|e| e.to_string())
            .and_then(|o| {
                if o.status.success() {
                    Ok(())
                } else {
                    Err(String::from_utf8_lossy(&o.stderr).to_string())
                }
            })
    }

    async fn remove_container(&self, id: &str) -> Result<(), String> {
        tokio::process::Command::new("podman")
            .args(["rm", "-f", id])
            .output()
            .await
            .map_err(|e| e.to_string())
            .and_then(|o| {
                if o.status.success() {
                    Ok(())
                } else {
                    Err(String::from_utf8_lossy(&o.stderr).to_string())
                }
            })
    }

    async fn logs(&self, id: &str) -> Result<String, String> {
        tokio::process::Command::new("podman")
            .args(["logs", id])
            .output()
            .await
            .map_err(|e| e.to_string())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_runtime_name() {
        let runtime = DockerRuntime::new();
        assert_eq!(runtime.name(), "docker");
    }

    #[test]
    fn test_podman_runtime_name() {
        let runtime = PodmanRuntime::new();
        assert_eq!(runtime.name(), "podman");
    }

    #[test]
    fn test_container_create_config() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());

        let config = ContainerCreateConfig {
            image: "nginx:latest".to_string(),
            name: Some("test".to_string()),
            env,
            ports: vec![],
            volumes: vec![],
        };

        assert_eq!(config.image, "nginx:latest");
        assert_eq!(config.name, Some("test".to_string()));
    }
}
