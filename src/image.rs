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
}
