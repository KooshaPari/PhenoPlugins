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
        // Container(NotFound("web")) → "Container error: web"
        let err = VesselError::Container(ContainerError::NotFound("web".to_string()));
        let s = format!("{}", err);
        assert!(s.contains("Container"), "expected 'Container' in '{}'", s);
        assert!(s.contains("web"), "expected inner 'web' in '{}'", s);

        // ImagePullFailed("nginx") → "Image error: failed to pull image"
        // (inner string intentionally NOT included in Display).
        let err = VesselError::ImagePullFailed("nginx".to_string());
        let s = format!("{}", err);
        assert!(!s.is_empty(), "ImagePullFailed display should be non-empty");
        assert!(s.contains("Image"), "expected 'Image' in '{}'", s);
        assert!(
            !s.contains("nginx"),
            "ImagePullFailed must NOT include inner string in Display: '{}'",
            s
        );

        // Runtime("daemon down") → "Runtime error: daemon down"
        let err = VesselError::Runtime("daemon down".to_string());
        let s = format!("{}", err);
        assert!(s.contains("Runtime"), "expected 'Runtime' in '{}'", s);
        assert!(s.contains("daemon down"), "expected inner text in '{}'", s);

        // Network("timeout") → "Network error: timeout"
        let err = VesselError::Network("timeout".to_string());
        let s = format!("{}", err);
        assert!(s.contains("Network"), "expected 'Network' in '{}'", s);
        assert!(s.contains("timeout"), "expected inner text in '{}'", s);

        // Io(io::Error) → "IO error: <io display>"
        let io_err = std::io::Error::other("disk gone");
        let err = VesselError::Io(io_err);
        let s = format!("{}", err);
        assert!(s.contains("IO") || s.contains("I/O"), "expected 'IO' or 'I/O' in '{}'", s);
        assert!(s.contains("disk gone"), "expected inner text in '{}'", s);
    }

    #[test]
    fn test_vessel_error_from() {
        use std::error::Error;

        // From<ContainerError> for VesselError::Container
        let c_err = ContainerError::NotFound("web".to_string());
        let v: VesselError = c_err.into();
        assert!(matches!(v, VesselError::Container(_)));
        let source = v.source().expect("VesselError::Container should expose a source");
        assert!(
            source.downcast_ref::<ContainerError>().is_some(),
            "source of VesselError::Container should be a ContainerError"
        );

        // From<std::io::Error> for VesselError::Io
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "nope");
        let v: VesselError = io_err.into();
        assert!(matches!(v, VesselError::Io(_)));
        let source = v.source().expect("VesselError::Io should expose a source");
        assert!(
            source.downcast_ref::<std::io::Error>().is_some(),
            "source of VesselError::Io should be a std::io::Error"
        );

        // Non-#[from] variants must NOT expose a source.
        let rt = VesselError::Runtime("x".to_string());
        assert!(rt.source().is_none(), "VesselError::Runtime should have no source");

        let net = VesselError::Network("y".to_string());
        assert!(net.source().is_none(), "VesselError::Network should have no source");

        let img = VesselError::ImagePullFailed("z".to_string());
        assert!(img.source().is_none(), "VesselError::ImagePullFailed should have no source");
    }

    #[test]
    fn test_re_exports() {
        // Compile-time check: all 12 re-exported types are accessible.
        #[allow(dead_code)]
        fn _takes_container_client(_: ContainerClient<DockerRuntime>) {}
        #[allow(dead_code)]
        fn _takes_container_error(_: ContainerError) {}
        #[allow(dead_code)]
        fn _takes_compose_file(_: ComposeFile) {}
        #[allow(dead_code)]
        fn _takes_compose_service(_: ComposeService) {}
        #[allow(dead_code)]
        fn _takes_container(_: Container) {}
        #[allow(dead_code)]
        fn _takes_container_config(_: ContainerConfig) {}
        #[allow(dead_code)]
        fn _takes_container_status(_: ContainerStatus) {}
        #[allow(dead_code)]
        fn _takes_image(_: Image) {}
        #[allow(dead_code)]
        fn _takes_image_pull_progress(_: ImagePullProgress) {}
        #[allow(dead_code)]
        fn _takes_container_runtime(_: &dyn ContainerRuntime) {}
        #[allow(dead_code)]
        fn _takes_docker_runtime(_: DockerRuntime) {}
        #[allow(dead_code)]
        fn _takes_podman_runtime(_: PodmanRuntime) {}

        // Each named type must be usable in a sized position.
        let _: Option<ContainerClient<DockerRuntime>> = None;
        let _: Option<ContainerError> = None;
        let _: Option<ComposeFile> = None;
        let _: Option<ComposeService> = None;
        let _: Option<Container> = None;
        let _: Option<ContainerConfig> = None;
        let _: Option<ContainerStatus> = None;
        let _: Option<Image> = None;
        let _: Option<ImagePullProgress> = None;
        let _: Option<Box<dyn ContainerRuntime>> = None;
        let _: Option<DockerRuntime> = None;
        let _: Option<PodmanRuntime> = None;
    }

    #[test]
    fn test_vessel_error_equality() {
        // Same variant + same payload → same discriminant.
        let a = VesselError::Runtime("x".to_string());
        let b = VesselError::Runtime("x".to_string());
        assert_eq!(std::mem::discriminant(&a), std::mem::discriminant(&b));

        // Same variant + different payload → same outer discriminant, different inner.
        let c = VesselError::Runtime("x".to_string());
        let d = VesselError::Runtime("y".to_string());
        assert_eq!(std::mem::discriminant(&c), std::mem::discriminant(&d));
        match (&c, &d) {
            (VesselError::Runtime(s_c), VesselError::Runtime(s_d)) => {
                assert_ne!(s_c, s_d, "inner payloads should differ");
            }
            _ => unreachable!("expected Runtime variants"),
        }

        // Different variant families → different discriminants.
        let rt = VesselError::Runtime("x".to_string());
        let net = VesselError::Network("x".to_string());
        let img = VesselError::ImagePullFailed("x".to_string());
        let io = VesselError::Io(std::io::Error::other("x"));
        let cont = VesselError::Container(ContainerError::NotFound("x".to_string()));
        assert_ne!(std::mem::discriminant(&rt), std::mem::discriminant(&net));
        assert_ne!(std::mem::discriminant(&rt), std::mem::discriminant(&img));
        assert_ne!(std::mem::discriminant(&rt), std::mem::discriminant(&io));
        assert_ne!(std::mem::discriminant(&rt), std::mem::discriminant(&cont));
        assert_ne!(std::mem::discriminant(&net), std::mem::discriminant(&img));
        assert_ne!(std::mem::discriminant(&net), std::mem::discriminant(&io));
        assert_ne!(std::mem::discriminant(&net), std::mem::discriminant(&cont));
        assert_ne!(std::mem::discriminant(&img), std::mem::discriminant(&io));
        assert_ne!(std::mem::discriminant(&img), std::mem::discriminant(&cont));
        assert_ne!(std::mem::discriminant(&io), std::mem::discriminant(&cont));

        // From<ContainerError>: different inner variants → same outer
        // VesselError::Container discriminant, different inner discriminants.
        let v1: VesselError = ContainerError::NotFound("a".to_string()).into();
        let v2: VesselError = ContainerError::AlreadyExists("b".to_string()).into();
        let v3: VesselError = ContainerError::OperationFailed("c".to_string()).into();
        assert_eq!(std::mem::discriminant(&v1), std::mem::discriminant(&v2));
        assert_eq!(std::mem::discriminant(&v2), std::mem::discriminant(&v3));
        assert_eq!(std::mem::discriminant(&v1), std::mem::discriminant(&v3));

        match (&v1, &v2, &v3) {
            (
                VesselError::Container(c1),
                VesselError::Container(c2),
                VesselError::Container(c3),
            ) => {
                assert_ne!(std::mem::discriminant(c1), std::mem::discriminant(c2));
                assert_ne!(std::mem::discriminant(c2), std::mem::discriminant(c3));
                assert_ne!(std::mem::discriminant(c1), std::mem::discriminant(c3));
            }
            _ => unreachable!("expected Container variants"),
        }
    }

    #[test]
    fn test_module_declarations_exist() {
        // Compile-time check: all 5 submodules are accessible and contain real types.
        let _: &str = "ok";

        // client module: ContainerClient is generic over ContainerRuntime.
        let _ = client::ContainerClient::<DockerRuntime>::new(DockerRuntime);

        // compose module: ComposeService derives Default (ComposeFile does not).
        let _ = compose::ComposeService::default();

        // container module: ContainerStatus is an enum with unit variants.
        let _ = container::ContainerStatus::Created;

        // image module: Image has a public constructor.
        let _ = image::Image::new("x");

        // runtime module: Protocol is a public enum.
        let _ = runtime::Protocol::Tcp;
    }

    #[test]
    fn test_vessel_error_debug_format() {
        // For each VesselError variant, format!("{:?}", err) must contain the
        // variant name and the inner payload string (or inner error message for Io).

        let c_err = ContainerError::NotFound("web".to_string());
        let err: VesselError = c_err.into();
        let s = format!("{:?}", err);
        assert!(s.contains("Container"), "expected 'Container' in Debug '{}'", s);
        assert!(s.contains("NotFound"), "expected 'NotFound' in Debug '{}'", s);
        assert!(s.contains("web"), "expected inner 'web' in Debug '{}'", s);

        let err = VesselError::ImagePullFailed("nginx".to_string());
        let s = format!("{:?}", err);
        assert!(
            s.contains("ImagePullFailed"),
            "expected 'ImagePullFailed' in Debug '{}'",
            s
        );
        assert!(s.contains("nginx"), "expected inner 'nginx' in Debug '{}'", s);

        let err = VesselError::Runtime("daemon down".to_string());
        let s = format!("{:?}", err);
        assert!(s.contains("Runtime"), "expected 'Runtime' in Debug '{}'", s);
        assert!(
            s.contains("daemon down"),
            "expected inner 'daemon down' in Debug '{}'",
            s
        );

        let err = VesselError::Network("timeout".to_string());
        let s = format!("{:?}", err);
        assert!(s.contains("Network"), "expected 'Network' in Debug '{}'", s);
        assert!(s.contains("timeout"), "expected inner 'timeout' in Debug '{}'", s);

        let io_err = std::io::Error::other("disk gone");
        let err = VesselError::Io(io_err);
        let s = format!("{:?}", err);
        assert!(s.contains("Io"), "expected 'Io' in Debug '{}'", s);
        assert!(
            s.contains("disk gone"),
            "expected inner 'disk gone' in Debug '{}'",
            s
        );
    }

    #[test]
    fn test_vessel_error_io_from_real_io_error() {
        // From<std::io::Error> should preserve the ErrorKind.
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let v: VesselError = io_err.into();
        match v {
            VesselError::Io(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
            other => panic!("expected VesselError::Io, got {:?}", other),
        }
    }

    #[test]
    fn test_vessel_error_source_chain_through_box() {
        // VesselError does NOT derive Clone (only Debug + Error), so we
        // cannot .clone() the error directly. This test replaces the
        // originally-proposed Clone check with a check that the error
        // source chain is preserved when the error is boxed as
        // Box<dyn std::error::Error> — a common real-world use case.
        use std::error::Error;

        // Io variant: source should be the original std::io::Error.
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io msg");
        let v: VesselError = io_err.into();
        let boxed: Box<dyn Error> = Box::new(v);
        let source = boxed
            .source()
            .expect("VesselError::Io should expose source through Box<dyn Error>");
        let downcast = source
            .downcast_ref::<std::io::Error>()
            .expect("source should be std::io::Error");
        assert_eq!(downcast.kind(), std::io::ErrorKind::Other);

        // Container variant: source should be the original ContainerError.
        let c_err = ContainerError::OperationFailed("op failed".to_string());
        let v: VesselError = c_err.into();
        let boxed: Box<dyn Error> = Box::new(v);
        let source = boxed
            .source()
            .expect("VesselError::Container should expose source through Box<dyn Error>");
        let downcast = source
            .downcast_ref::<ContainerError>()
            .expect("source should be ContainerError");
        assert!(matches!(downcast, ContainerError::OperationFailed(_)));
    }

    #[test]
    fn test_podman_runtime_re_exported() {
        // Verifies the `pub use runtime::PodmanRuntime` re-export works
        // and that PodmanRuntime::new() + name() behave as expected.
        let p = PodmanRuntime::new();
        assert_eq!(p.name(), "podman");
    }

    #[test]
    fn test_image_pull_progress_re_exported() {
        // Verifies the `pub use image::ImagePullProgress` re-export works
        // and that the struct fields are accessible.
        let p = ImagePullProgress {
            status: "x".into(),
            progress: None,
            speed: None,
        };
        assert_eq!(p.status, "x");
        assert!(p.progress.is_none());
        assert!(p.speed.is_none());
    }
}
