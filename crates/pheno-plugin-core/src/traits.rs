//! Core plugin traits for AgilePlus extensibility.
//!
//! These traits define the port interfaces that adapters must implement.
//! They follow the Hexagonal Architecture pattern where the core domain
//! defines the interfaces that adapters must satisfy.
//!
//! ## Dyn Compatibility
//!
//! These traits use `#[trait_variant]` to enable dynamic dispatch via `dyn Trait`.
//! This allows runtime plugin selection and swapping.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::PluginResult;

/// Configuration for a plugin adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Adapter-specific configuration (JSON)
    #[serde(default)]
    pub adapter_config: serde_json::Value,
}

/// Base trait for all AgilePlus plugins.
///
/// All adapters must implement this trait to be registered in the system.
/// It provides metadata and lifecycle management for plugins.
///
/// ## Example
///
/// ```rust,ignore
/// struct GitAdapter { /* ... */ }
///
/// impl AdapterPlugin for GitAdapter {
///     fn name(&self) -> &str { "git" }
///     fn version(&self) -> &str { "0.1.0" }
///     fn initialize(&self, config: PluginConfig) -> PluginResult<()> {
///         // Initialize adapter
///         Ok(())
///     }
/// }
/// ```
pub trait AdapterPlugin: Send + Sync {
    /// Returns the plugin name (e.g., "git", "sqlite", "ollama").
    fn name(&self) -> &str;

    /// Returns the plugin version.
    fn version(&self) -> &str;

    /// Initializes the plugin with configuration.
    ///
    /// This is called once when the plugin is registered.
    fn initialize(&self, config: PluginConfig) -> PluginResult<()>;

    /// Returns the plugin health status.
    ///
    /// Returns `Ok(())` if healthy, or an error describing the issue.
    fn health_check(&self) -> PluginResult<()> {
        Ok(())
    }
}

// ============================================================================
// VCS Plugin Trait
// ============================================================================

/// Metadata about an active git worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub feature_slug: String,
    pub wp_id: String,
}

/// Result of a merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<ConflictInfo>,
    pub merged_commit: Option<String>,
}

/// Description of a merge conflict in a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub path: String,
    pub ours: Option<String>,
    pub theirs: Option<String>,
}

/// Collected feature artifacts discovered in the repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureArtifacts {
    pub meta_json: Option<String>,
    pub audit_chain: Option<String>,
    pub evidence_paths: Vec<String>,
}

/// VCS (Version Control System) plugin trait.
///
/// Abstracts git operations so tests can use in-memory mocks.
///
/// ## Dyn Compatibility
///
/// This trait uses `#[async_trait]` to support dynamic dispatch.
///
/// ## Implementations
///
/// - `agileplus-plugin-git`: Production git adapter using gitoxide
/// - Mock adapter: For testing without filesystem
#[async_trait::async_trait]
pub trait VcsPlugin: AdapterPlugin {
    // -- Worktree operations --

    /// Create a worktree for a feature work package.
    async fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> PluginResult<PathBuf>;

    /// List all worktrees.
    async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>>;

    /// Clean up (remove) a worktree.
    async fn cleanup_worktree(&self, worktree_path: &Path) -> PluginResult<()>;

    // -- Branch operations --

    /// Create a new branch.
    async fn create_branch(&self, branch_name: &str, base: &str) -> PluginResult<()>;

    /// Checkout a branch.
    async fn checkout_branch(&self, branch_name: &str) -> PluginResult<()>;

    // -- Merge operations --

    /// Merge source branch into target.
    async fn merge_to_target(
        &self,
        source: &str,
        target: &str,
    ) -> PluginResult<MergeResult>;

    /// Detect conflicts between branches.
    async fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> PluginResult<Vec<ConflictInfo>>;

    // -- Artifact operations --

    /// Read an artifact file.
    async fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> PluginResult<String>;

    /// Write an artifact file.
    async fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> PluginResult<()>;

    /// Check if an artifact exists.
    async fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> PluginResult<bool>;

    /// Scan and collect all artifacts for a feature.
    async fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> PluginResult<FeatureArtifacts>;
}

// ============================================================================
// Storage Plugin Trait
// ============================================================================

/// Storage plugin trait.
///
/// Abstracts database operations for persistence.
///
/// ## Dyn Compatibility
///
/// This trait uses `#[async_trait]` to support dynamic dispatch.
///
/// ## Implementations
///
/// - `agileplus-plugin-sqlite`: SQLite adapter (rusqlite)
/// - `agileplus-plugin-postgres`: PostgreSQL adapter (sqlx) [future]
#[async_trait::async_trait]
pub trait StoragePlugin: AdapterPlugin {
    // -- Feature operations --

    /// Create a new feature.
    async fn create_feature(
        &self,
        feature: &serde_json::Value,
    ) -> PluginResult<i64>;

    /// Get a feature by slug.
    async fn get_feature_by_slug(
        &self,
        slug: &str,
    ) -> PluginResult<Option<serde_json::Value>>;

    /// Get a feature by ID.
    async fn get_feature_by_id(
        &self,
        id: i64,
    ) -> PluginResult<Option<serde_json::Value>>;

    /// Update feature state.
    async fn update_feature_state(&self, id: i64, state: &str) -> PluginResult<()>;

    /// List all features.
    async fn list_all_features(&self) -> PluginResult<Vec<serde_json::Value>>;

    // -- Work package operations --

    /// Create a work package.
    async fn create_work_package(
        &self,
        wp: &serde_json::Value,
    ) -> PluginResult<i64>;

    /// Get a work package by ID.
    async fn get_work_package(
        &self,
        id: i64,
    ) -> PluginResult<Option<serde_json::Value>>;

    /// Update work package state.
    async fn update_wp_state(&self, id: i64, state: &str) -> PluginResult<()>;

    // -- Audit operations --

    /// Append an audit entry.
    async fn append_audit_entry(
        &self,
        entry: &serde_json::Value,
    ) -> PluginResult<i64>;

    /// Get audit trail for a feature.
    async fn get_audit_trail(
        &self,
        feature_id: i64,
    ) -> PluginResult<Vec<serde_json::Value>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_plugin_config_construction_and_clone() {
        let cfg = PluginConfig {
            name: "x".to_string(),
            version: "1.0".to_string(),
            adapter_config: serde_json::json!({}),
        };
        assert_eq!(cfg.name, "x");
        assert_eq!(cfg.version, "1.0");
        assert_eq!(cfg.adapter_config, serde_json::json!({}));

        let cloned = cfg.clone();
        assert_eq!(cloned.name, cfg.name);
        assert_eq!(cloned.version, cfg.version);
        assert_eq!(cloned.adapter_config, cfg.adapter_config);

        let dbg = format!("{:?}", cfg);
        assert!(dbg.contains("x"), "debug output should contain name 'x': {}", dbg);
        assert!(
            dbg.contains("1.0"),
            "debug output should contain version '1.0': {}",
            dbg
        );
    }

    #[test]
    fn test_plugin_config_serde_roundtrip() {
        let original = PluginConfig {
            name: "git".to_string(),
            version: "0.1.0".to_string(),
            adapter_config: serde_json::json!({"key": "value"}),
        };

        // Sanity-check that the JSON is valid by successfully serializing.
        let json_str = serde_json::to_string(&original).expect("serialize should succeed");
        assert!(!json_str.is_empty(), "serialized JSON should not be empty");
        // Re-parse to ensure the produced JSON is itself valid JSON.
        let reparsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("serialized output should be valid JSON");
        assert_eq!(reparsed["name"], "git");
        assert_eq!(reparsed["version"], "0.1.0");
        assert_eq!(reparsed["adapter_config"]["key"], "value");

        let deserialized: PluginConfig =
            serde_json::from_str(&json_str).expect("deserialize should succeed");
        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.version, original.version);
        assert_eq!(deserialized.adapter_config, original.adapter_config);
    }

    #[test]
    fn test_plugin_config_default_adapter_config() {
        // JSON payload intentionally omits `adapter_config` to exercise `#[serde(default)]`.
        let json_str = r#"{"name":"x","version":"1.0"}"#;
        let cfg: PluginConfig =
            serde_json::from_str(json_str).expect("deserialize should succeed");

        assert_eq!(cfg.name, "x");
        assert_eq!(cfg.version, "1.0");
        assert!(
            cfg.adapter_config.is_null(),
            "adapter_config should default to Null when missing, got: {}",
            cfg.adapter_config
        );
    }

    #[test]
    fn test_worktree_info_construction() {
        let info = WorktreeInfo {
            path: PathBuf::from("/tmp/x"),
            branch: "main".to_string(),
            feature_slug: "f".to_string(),
            wp_id: "w".to_string(),
        };
        assert_eq!(info.path, PathBuf::from("/tmp/x"));
        assert_eq!(info.branch, "main");
        assert_eq!(info.feature_slug, "f");
        assert_eq!(info.wp_id, "w");
    }

    #[test]
    fn test_merge_result_construction() {
        // Empty conflicts case.
        let ok = MergeResult {
            success: true,
            conflicts: vec![],
            merged_commit: Some("abc123".to_string()),
        };
        assert!(ok.success);
        assert!(ok.conflicts.is_empty());
        assert_eq!(ok.merged_commit.as_deref(), Some("abc123"));

        // With a single conflict and no merged commit.
        let conflict = ConflictInfo {
            path: "x.rs".to_string(),
            ours: Some("a".to_string()),
            theirs: Some("b".to_string()),
        };
        let failed = MergeResult {
            success: false,
            conflicts: vec![conflict],
            merged_commit: None,
        };
        assert!(!failed.success);
        assert_eq!(failed.conflicts.len(), 1);
        assert_eq!(failed.conflicts[0].path, "x.rs");
        assert_eq!(failed.conflicts[0].ours.as_deref(), Some("a"));
        assert_eq!(failed.conflicts[0].theirs.as_deref(), Some("b"));
        assert!(failed.merged_commit.is_none());
    }
}
