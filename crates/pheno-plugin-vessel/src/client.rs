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
        // Construct a ContainerClient backed by a concrete DockerRuntime.
        // Verifies that ContainerClient::new accepts a runtime that implements
        // the ContainerRuntime trait and that the resulting client is usable
        // (no panic during construction, runtime_name() is reachable).
        let client = ContainerClient::new(DockerRuntime::new());
        assert_eq!(client.runtime_name(), "docker");
    }

    #[test]
    fn test_container_client_runtime_name() {
        // Verify that runtime_name() correctly delegates to the underlying
        // runtime and returns "docker" for a DockerRuntime-backed client.
        let client = ContainerClient::new(DockerRuntime::new());
        assert_eq!(client.runtime_name(), "docker");
    }

    #[test]
    fn test_container_error_display() {
        // Verify Display formatting for every ContainerError variant. The
        // generated messages must contain a recognizable keyword plus the
        // stringified inner value.
        let not_found = ContainerError::NotFound("my-container".to_string());
        let rendered = format!("{}", not_found);
        assert!(rendered.contains("not found"), "expected 'not found' substring in: {}", rendered);
        assert!(
            rendered.contains("my-container"),
            "expected 'my-container' substring in: {}",
            rendered
        );

        let already_exists = ContainerError::AlreadyExists("dup-name".to_string());
        let rendered = format!("{}", already_exists);
        assert!(
            rendered.contains("already exists"),
            "expected 'already exists' substring in: {}",
            rendered
        );
        assert!(rendered.contains("dup-name"), "expected 'dup-name' substring in: {}", rendered);

        let op_failed = ContainerError::OperationFailed("boom".to_string());
        let rendered = format!("{}", op_failed);
        assert!(rendered.contains("failed"), "expected 'failed' substring in: {}", rendered);
        assert!(rendered.contains("boom"), "expected 'boom' substring in: {}", rendered);
    }

    #[test]
    fn test_container_error_from() {
        // ContainerError itself has no `#[from]` impls, but VesselError does:
        //   #[from] ContainerError
        // Verify the conversion is wired up and the inner payload is preserved.
        let inner = ContainerError::NotFound("abc".to_string());
        let outer: VesselError = inner.into();
        match outer {
            VesselError::Container(ContainerError::NotFound(s)) => {
                assert_eq!(s, "abc");
            }
            other => panic!("expected VesselError::Container(NotFound), got {:?}", other),
        }

        // Also verify the std::io::Error -> VesselError #[from] path that is
        // declared on VesselError, since it is the only other #[from] impl
        // exposed by this crate's error module.
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let vessel: VesselError = io_err.into();
        match vessel {
            VesselError::Io(_) => {}
            other => panic!("expected VesselError::Io, got {:?}", other),
        }
    }

    #[test]
    fn test_container_client_default_runtime() {
        // ContainerClient does not expose a with_default_runtime() constructor,
        // but DockerRuntime is a unit struct that can be constructed with
        // no arguments. Verify a client can be built from such a default
        // runtime and still reports the correct name.
        let client = ContainerClient::new(DockerRuntime);
        assert_eq!(client.runtime_name(), "docker");
    }

    #[tokio::test]
    async fn test_container_client_is_available_returns_bool() {
        // is_available() does not require docker to be installed; it returns
        // false in that case rather than erroring. We only assert that the
        // call resolves to a bool value without panicking.
        let client = ContainerClient::new(DockerRuntime::new());
        let available: bool = client.is_available().await;
        // The result is environment-dependent (true if docker is on PATH,
        // false otherwise); we just want to ensure the future completed.
        let _ = available;
    }
}
