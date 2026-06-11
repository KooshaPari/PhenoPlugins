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

    #[test]
    fn test_vessel_error_display() {
        // String-bearing variant: Runtime
        let runtime_err = VesselError::Runtime("docker daemon not responding".to_string());
        let runtime_display = format!("{}", runtime_err);
        assert!(
            runtime_display.contains("Runtime"),
            "Runtime display should mention 'Runtime', got: {:?}",
            runtime_display
        );
        assert!(
            runtime_display.contains("docker daemon not responding"),
            "Runtime display should include inner text, got: {:?}",
            runtime_display
        );

        // String-bearing variant: Network
        let network_err = VesselError::Network("connection refused".to_string());
        let network_display = format!("{}", network_err);
        assert!(
            network_display.contains("Network"),
            "Network display should mention 'Network', got: {:?}",
            network_display
        );
        assert!(
            network_display.contains("connection refused"),
            "Network display should include inner text, got: {:?}",
            network_display
        );

        // String-bearing variant: ImagePullFailed.
        // Note: its `#[error("Image error: failed to pull image")]` attribute
        // does not include `{0}`, so the inner String is intentionally omitted
        // from Display. We still verify the output is non-empty and contains a
        // recognizable keyword.
        let image_err = VesselError::ImagePullFailed("nginx:latest".to_string());
        let image_display = format!("{}", image_err);
        assert!(
            !image_display.is_empty(),
            "ImagePullFailed display should be non-empty, got: {:?}",
            image_display
        );
        assert!(
            image_display.contains("Image"),
            "ImagePullFailed display should mention 'Image', got: {:?}",
            image_display
        );

        // From-derived variant: Container (via From<ContainerError>).
        // Display should be non-empty and include the inner error text.
        let container_err: VesselError = ContainerError::NotFound("web".to_string()).into();
        let container_display = format!("{}", container_err);
        assert!(!container_display.is_empty(), "Container display should be non-empty");
        assert!(
            container_display.contains("Container"),
            "Container display should mention 'Container', got: {:?}",
            container_display
        );
        assert!(
            container_display.contains("web"),
            "Container display should include inner NotFound text, got: {:?}",
            container_display
        );

        // From-derived variant: Io (via From<std::io::Error>).
        let io_err: VesselError =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied").into();
        let io_display = format!("{}", io_err);
        assert!(!io_display.is_empty(), "Io display should be non-empty");
        assert!(
            io_display.contains("IO") || io_display.contains("I/O"),
            "Io display should mention 'IO' or 'I/O', got: {:?}",
            io_display
        );
        assert!(
            io_display.contains("access denied"),
            "Io display should include inner io error text, got: {:?}",
            io_display
        );
    }

    #[test]
    fn test_vessel_error_from() {
        // From<ContainerError> -> VesselError::Container
        let inner = ContainerError::OperationFailed("start timed out".to_string());
        let vessel: VesselError = inner.into();
        match &vessel {
            VesselError::Container(ce) => {
                assert!(
                    matches!(ce, ContainerError::OperationFailed(ref s) if s == "start timed out"),
                    "Inner ContainerError should round-trip"
                );
            }
            _ => panic!("Expected VesselError::Container variant, got: {:?}", vessel),
        }
        // Source chain should point to the inner ContainerError
        let source = std::error::Error::source(&vessel);
        assert!(source.is_some(), "VesselError::Container should have a source");
        let source_str = source.unwrap().to_string();
        assert!(
            source_str.contains("start timed out"),
            "Source should include inner error text, got: {:?}",
            source_str
        );

        // From<std::io::Error> -> VesselError::Io
        let io_inner = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let vessel_io: VesselError = io_inner.into();
        match &vessel_io {
            VesselError::Io(_) => {}
            _ => panic!("Expected VesselError::Io variant, got: {:?}", vessel_io),
        }
        let io_source = std::error::Error::source(&vessel_io);
        assert!(io_source.is_some(), "VesselError::Io should have a source");
        let io_source_str = io_source.unwrap().to_string();
        assert!(
            io_source_str.contains("file missing"),
            "Io source should include inner error text, got: {:?}",
            io_source_str
        );

        // Non-#[from] variants should not have a source
        assert!(
            std::error::Error::source(&VesselError::Runtime("plain".to_string())).is_none(),
            "VesselError::Runtime should not have a source"
        );
        assert!(
            std::error::Error::source(&VesselError::ImagePullFailed("nginx".to_string())).is_none(),
            "VesselError::ImagePullFailed should not have a source"
        );
        assert!(
            std::error::Error::source(&VesselError::Network("timeout".to_string())).is_none(),
            "VesselError::Network should not have a source"
        );
    }

    #[test]
    fn test_re_exports() {
        // Compile-time check: the following types must be accessible through
        // the crate root via the `pub use` re-exports in `lib.rs`. If any
        // re-export is removed or renamed, this test will fail to compile.
        fn _assert_reexports_compile() {
            // From `pub use client::{ContainerClient, ContainerError};`
            let _: Option<ContainerClient<DockerRuntime>> = None;
            fn _takes_container_error(_: ContainerError) {}

            // From `pub use compose::{ComposeFile, ComposeService};`
            fn _takes_compose_file(_: ComposeFile) {}
            fn _takes_compose_service(_: ComposeService) {}

            // From `pub use container::{Container, ContainerConfig, ContainerStatus};`
            fn _takes_container(_: Container) {}
            fn _takes_container_config(_: ContainerConfig) {}
            fn _takes_container_status(_: ContainerStatus) {}

            // From `pub use image::{Image, ImagePullProgress};`
            fn _takes_image(_: Image) {}
            fn _takes_image_pull_progress(_: ImagePullProgress) {}

            // From `pub use runtime::{ContainerRuntime, DockerRuntime, PodmanRuntime};`
            fn _takes_container_runtime<T: ContainerRuntime>(_: &T) {}
            fn _takes_docker_runtime(_: DockerRuntime) {}
            fn _takes_podman_runtime(_: PodmanRuntime) {}
        }
        // No runtime assertion needed — successful compilation is the test.
    }

    #[test]
    fn test_vessel_error_equality() {
        // VesselError does not derive PartialEq, so `==` is not available.
        // `std::mem::discriminant` is the canonical way to verify variant
        // identity on enums without PartialEq; we additionally check the
        // inner payload via `matches!` for content equality.

        // Identical: same variant, same string payload
        let a = VesselError::Runtime("boom".to_string());
        let b = VesselError::Runtime("boom".to_string());
        assert_eq!(
            std::mem::discriminant(&a),
            std::mem::discriminant(&b),
            "Identical Runtime variants should share a discriminant"
        );
        assert!(
            matches!(&a, VesselError::Runtime(s) if s == "boom"),
            "First variant should match Runtime(\"boom\")"
        );
        assert!(
            matches!(&b, VesselError::Runtime(s) if s == "boom"),
            "Second variant should match Runtime(\"boom\")"
        );

        // Different: same variant family, different payload
        let c = VesselError::Runtime("other".to_string());
        if let (VesselError::Runtime(s_a), VesselError::Runtime(s_c)) = (&a, &c) {
            assert_ne!(s_a, s_c, "Inner payload should differ");
        } else {
            panic!("Expected Runtime variants");
        }

        // Different: distinct variant families
        let network = VesselError::Network("x".to_string());
        let runtime = VesselError::Runtime("x".to_string());
        assert_ne!(
            std::mem::discriminant(&network),
            std::mem::discriminant(&runtime),
            "Network and Runtime should have different discriminants"
        );

        // From-derived: same inner -> same outer discriminant
        let c1: VesselError = ContainerError::NotFound("abc".to_string()).into();
        let c2: VesselError = ContainerError::NotFound("abc".to_string()).into();
        assert_eq!(
            std::mem::discriminant(&c1),
            std::mem::discriminant(&c2),
            "Container variants from identical ContainerErrors should share a discriminant"
        );

        // From-derived: different inner variant -> same outer discriminant
        // (both are `VesselError::Container(_)`), but inner discriminants differ.
        let c3: VesselError = ContainerError::AlreadyExists("abc".to_string()).into();
        assert_eq!(
            std::mem::discriminant(&c1),
            std::mem::discriminant(&c3),
            "Different ContainerError variants should produce the same outer discriminant"
        );
        if let (VesselError::Container(inner1), VesselError::Container(inner3)) = (&c1, &c3) {
            assert_ne!(
                std::mem::discriminant(inner1),
                std::mem::discriminant(inner3),
                "Inner ContainerError discriminants should differ"
            );
        } else {
            panic!("Expected Container variants");
        }

        // From-derived: Io variant distinct from Container variant
        let io_vessel: VesselError = std::io::Error::new(std::io::ErrorKind::Other, "boom").into();
        assert_ne!(
            std::mem::discriminant(&c1),
            std::mem::discriminant(&io_vessel),
            "Container and Io outer discriminants should differ"
        );
    }
}
