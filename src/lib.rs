//! # phenotype-vessel
//!
//! @trace VES-001: Agent Runtime
//! @trace VES-002: Sandbox Isolation
//! @trace VES-004: Monitoring
//!
//! Rust container utilities library providing abstractions over Docker, Podman, and containerd.
//!
//! ## Features
//!
//! - **Multi-runtime**: Unified API for Docker, Podman, and containerd
//! - **Async-first**: All operations are async using tokio
//! - **Image management**: Build, pull, and manage container images
//! - **Container lifecycle**: Run, stop, and manage containers
//! - **Compose support**: Multi-container orchestration
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # async fn quickstart() -> Result<(), Box<dyn std::error::Error>> {
//! use phenotype_vessel::{ContainerClient, DockerRuntime};
//!
//! let client = ContainerClient::new(DockerRuntime);
//! let image = client.pull_image("nginx:latest").await?;
//! let container = client.run("nginx:latest", "my-container").await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod compose;
pub mod container;
pub mod image;
pub mod runtime;

pub use client::{ContainerClient, ContainerError};
pub use compose::{ComposeFile, ComposeService};
pub use container::{Container, ContainerConfig, ContainerStatus};
pub use image::{Image, ImagePullProgress};
pub use runtime::{ContainerRuntime, DockerRuntime, PodmanRuntime};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VesselError {
    #[error("Container error: {0}")]
    Container(#[from] ContainerError),

    #[error("Image error: failed to pull image")]
    ImagePullFailed(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_runtime_creation() {
        let runtime = DockerRuntime;
        assert_eq!(runtime.name(), "docker");
    }
}
