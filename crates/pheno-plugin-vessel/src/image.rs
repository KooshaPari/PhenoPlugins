//! # phenotype-vessel
//!
//! Container image management.

use serde::{Deserialize, Serialize};

/// Container image representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Image ID (digest or hash)
    pub id: String,
    /// Image name
    pub name: String,
    /// Image tag
    pub tag: String,
    /// Image size in bytes
    pub size: u64,
}

/// Image pull progress information
#[derive(Debug, Clone)]
pub struct ImagePullProgress {
    /// Status message
    pub status: String,
    /// Progress percentage (if available)
    pub progress: Option<f32>,
    /// Download speed (bytes/sec)
    pub speed: Option<u64>,
}

impl Image {
    /// Create a new image reference
    pub fn new(name: &str) -> Self {
        let parts: Vec<&str> = name.split(':').collect();
        Self {
            id: name.to_string(),
            name: parts.first().unwrap_or(&name).to_string(),
            tag: parts.get(1).unwrap_or(&"latest").to_string(),
            size: 0,
        }
    }

    /// Full image reference (name:tag)
    pub fn reference(&self) -> String {
        format!("{}:{}", self.name, self.tag)
    }

    /// Check if this is a digest reference
    pub fn is_digest(&self) -> bool {
        self.tag.starts_with("sha256:")
    }
}

impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_creation() {
        let image = Image::new("nginx:latest");
        assert_eq!(image.name, "nginx");
        assert_eq!(image.tag, "latest");
    }

    #[test]
    fn test_image_reference() {
        let image = Image::new("postgres:15");
        assert_eq!(image.reference(), "postgres:15");
    }

    #[test]
    fn test_digest_image() {
        let image = Image {
            id: "sha256:abc123".to_string(),
            name: "nginx".to_string(),
            tag: "sha256:abc123".to_string(),
            size: 100,
        };
        assert!(image.is_digest());
    }

    #[test]
    fn test_image_is_digest_false() {
        let image = Image::new("alpine");
        assert_eq!(image.name, "alpine");
        assert_eq!(image.tag, "latest");
        assert!(!image.is_digest());

        let image = Image::new("ubuntu:20.04");
        assert_eq!(image.name, "ubuntu");
        assert_eq!(image.tag, "20.04");
        assert!(!image.is_digest());
    }

    #[test]
    fn test_image_display() {
        let image = Image {
            id: "alpine:3.18".to_string(),
            name: "alpine".to_string(),
            tag: "3.18".to_string(),
            size: 0,
        };
        assert_eq!(format!("{}", image), "alpine:3.18");

        let digest_image = Image {
            id: "nginx:sha256:abcdef".to_string(),
            name: "nginx".to_string(),
            tag: "sha256:abcdef".to_string(),
            size: 0,
        };
        assert_eq!(format!("{}", digest_image), "nginx:sha256:abcdef");
    }

    #[test]
    fn test_image_serde_roundtrip() {
        let image = Image {
            id: "nginx:1.25-alpine".to_string(),
            name: "nginx".to_string(),
            tag: "1.25-alpine".to_string(),
            size: 42,
        };
        // Use serde_yaml (already a dev-dep) to validate the Serialize/Deserialize
        // derives on Image round-trip cleanly. The on-the-wire format is irrelevant
        // for verifying derive correctness on this all-primitive field set.
        let serialized = serde_yaml::to_string(&image).expect("serialize Image");
        let parsed: Image = serde_yaml::from_str(&serialized).expect("deserialize Image");
        assert_eq!(parsed.id, "nginx:1.25-alpine");
        assert_eq!(parsed.name, "nginx");
        assert_eq!(parsed.tag, "1.25-alpine");
        assert_eq!(parsed.size, 42);
    }

    #[test]
    fn test_image_new_defaults() {
        let alpine = Image::new("alpine");
        assert_eq!(alpine.name, "alpine");
        assert_eq!(alpine.tag, "latest");
        assert_eq!(alpine.id, "alpine");
        assert_eq!(alpine.size, 0);

        let redis = Image::new("redis");
        assert_eq!(redis.name, "redis");
        assert_eq!(redis.tag, "latest");
        assert_eq!(redis.id, "redis");
        assert_eq!(redis.size, 0);

        let postgres = Image::new("postgres:15");
        assert_eq!(postgres.name, "postgres");
        assert_eq!(postgres.tag, "15");
        assert_eq!(postgres.id, "postgres:15");
        assert_eq!(postgres.size, 0);
    }

    #[test]
    fn test_image_pull_progress() {
        let mut progress = ImagePullProgress {
            status: "Downloading".to_string(),
            progress: Some(0.5),
            speed: Some(1_000_000),
        };
        assert_eq!(progress.status, "Downloading");
        assert_eq!(progress.progress, Some(0.5));
        assert_eq!(progress.speed, Some(1_000_000));

        progress.status = "Extracting".to_string();
        progress.progress = Some(1.0);
        progress.speed = None;
        assert_eq!(progress.status, "Extracting");
        assert_eq!(progress.progress, Some(1.0));
        assert_eq!(progress.speed, None);
    }

    #[test]
    fn test_image_equality_via_fields() {
        // Image does not derive PartialEq, so compare field-by-field.
        let a = Image {
            id: "nginx:1.25".to_string(),
            name: "nginx".to_string(),
            tag: "1.25".to_string(),
            size: 100,
        };
        let b = Image {
            id: "nginx:1.25".to_string(),
            name: "nginx".to_string(),
            tag: "1.25".to_string(),
            size: 100,
        };
        assert_eq!(a.id, b.id);
        assert_eq!(a.name, b.name);
        assert_eq!(a.tag, b.tag);
        assert_eq!(a.size, b.size);

        // Differing Images diverge in at least one field.
        let different_size = Image {
            id: "nginx:1.25".to_string(),
            name: "nginx".to_string(),
            tag: "1.25".to_string(),
            size: 200,
        };
        assert_ne!(a.size, different_size.size);

        let different_tag = Image {
            id: "nginx:1.26".to_string(),
            name: "nginx".to_string(),
            tag: "1.26".to_string(),
            size: 100,
        };
        assert_ne!(a.tag, different_tag.tag);
    }
}
