//! # phenotype-vessel
//!
//! Container client for managing containers.

use super::container::{Container, ContainerStatus};
use super::image::Image;
use super::{ContainerRuntime, VesselError};
use crate::runtime::{ContainerCreateConfig, ContainerInfo};

/// Unified container client for all runtimes
#[derive(Debug)]
pub struct ContainerClient<R: ContainerRuntime> {
    runtime: R,
}

impl<R: ContainerRuntime> ContainerClient<R> {
    /// Create a new container client
    pub fn new(runtime: R) -> Self {
        Self { runtime }
    }

    /// Get the runtime name
    pub fn runtime_name(&self) -> &str {
        self.runtime.name()
    }

    /// Check if the runtime is available
    pub async fn is_available(&self) -> bool {
        self.runtime.is_available().await
    }

    /// List all containers
    pub async fn list_containers(&self) -> Result<Vec<ContainerInfo>, VesselError> {
        self.runtime.list_containers().await.map_err(VesselError::Runtime)
    }

    /// Pull an image
    pub async fn pull_image(&self, image: &str) -> Result<Image, VesselError> {
        self.runtime.pull_image(image).await.map_err(VesselError::Runtime)?;

        Ok(Image {
            id: image.to_string(),
            name: image.to_string(),
            tag: "latest".to_string(),
            size: 0,
        })
    }

    /// Remove an image
    pub async fn remove_image(&self, image: &str) -> Result<(), VesselError> {
        self.runtime.remove_image(image).await.map_err(VesselError::Runtime)
    }

    /// Create and start a container
    pub async fn run(&self, image: &str, name: &str) -> Result<Container, VesselError> {
        let config = ContainerCreateConfig {
            image: image.to_string(),
            name: Some(name.to_string()),
            env: Default::default(),
            ports: vec![],
            volumes: vec![],
        };

        let container_id =
            self.runtime.create_container(&config).await.map_err(VesselError::Runtime)?;

        self.runtime.start_container(&container_id).await.map_err(VesselError::Runtime)?;

        Ok(Container {
            id: container_id,
            name: name.to_string(),
            image: image.to_string(),
            status: ContainerStatus::Running,
        })
    }

    /// Create a container without starting it
    pub async fn create(&self, image: &str, name: &str) -> Result<Container, VesselError> {
        let config = ContainerCreateConfig {
            image: image.to_string(),
            name: Some(name.to_string()),
            env: Default::default(),
            ports: vec![],
            volumes: vec![],
        };

        let container_id =
            self.runtime.create_container(&config).await.map_err(VesselError::Runtime)?;

        Ok(Container {
            id: container_id,
            name: name.to_string(),
            image: image.to_string(),
            status: ContainerStatus::Created,
        })
    }

    /// Start a container
    pub async fn start(&self, id: &str) -> Result<(), VesselError> {
        self.runtime.start_container(id).await.map_err(VesselError::Runtime)
    }

    /// Stop a container
    pub async fn stop(&self, id: &str) -> Result<(), VesselError> {
        self.runtime.stop_container(id).await.map_err(VesselError::Runtime)
    }

    /// Remove a container
    pub async fn rm(&self, id: &str) -> Result<(), VesselError> {
        self.runtime.remove_container(id).await.map_err(VesselError::Runtime)
    }

    /// Get container logs
    pub async fn logs(&self, id: &str) -> Result<String, VesselError> {
        self.runtime.logs(id).await.map_err(VesselError::Runtime)
    }
}

/// Container operation errors
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("Container not found: {0}")]
    NotFound(String),

    #[error("Container already exists: {0}")]
    AlreadyExists(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::DockerRuntime;

    #[test]
    fn test_container_client_new() {
        let client = ContainerClient::new(DockerRuntime::new());
        assert_eq!(client.runtime_name(), "docker");
    }

    #[test]
    fn test_container_client_runtime_name() {
        let client = ContainerClient::new(DockerRuntime::new());
        assert_eq!(client.runtime_name(), "docker");
        assert!(!client.runtime_name().is_empty());
    }

    #[test]
    fn test_container_error_display() {
        let not_found = ContainerError::NotFound("my-container".to_string());
        let not_found_str = format!("{}", not_found);
        assert!(not_found_str.contains("not found"));
        assert!(not_found_str.contains("my-container"));

        let already_exists = ContainerError::AlreadyExists("my-container".to_string());
        let already_exists_str = format!("{}", already_exists);
        assert!(already_exists_str.contains("already exists"));
        assert!(already_exists_str.contains("my-container"));

        let operation_failed = ContainerError::OperationFailed("something broke".to_string());
        let operation_failed_str = format!("{}", operation_failed);
        assert!(operation_failed_str.contains("failed"));
        assert!(operation_failed_str.contains("something broke"));
    }

    #[test]
    fn test_container_error_from() {
        let container_err = ContainerError::NotFound("abc".to_string());
        let v: VesselError = container_err.into();
        match v {
            VesselError::Container(ContainerError::NotFound(s)) => {
                assert_eq!(s, "abc");
            }
            other => panic!("Expected VesselError::Container(NotFound), got {:?}", other),
        }
    }

    #[test]
    fn test_container_client_default_runtime() {
        let client = ContainerClient::new(DockerRuntime);
        assert_eq!(client.runtime_name(), "docker");
    }

    #[tokio::test]
    async fn test_container_client_is_available_returns_bool() {
        let client = ContainerClient::new(DockerRuntime);
        let _: bool = client.is_available().await;
    }
}
