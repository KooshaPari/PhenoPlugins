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
