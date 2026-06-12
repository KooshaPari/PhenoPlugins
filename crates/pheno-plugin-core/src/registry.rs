//! Plugin registry for managing adapter registrations.
//!
//! The registry is the central component that holds all plugin instances.
//! It provides lookup by type and name.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{PluginError, PluginResult};
use crate::traits::{StoragePlugin, VcsPlugin};

/// Thread-safe plugin registry.
/// Thread-safe plugin registry.
///
/// Manages registration and lookup of all adapter plugins.
/// Uses interior mutability for concurrent access.
pub struct PluginRegistry {
    vcs: RwLock<HashMap<String, Arc<dyn VcsPlugin>>>,
    storage: RwLock<HashMap<String, Arc<dyn StoragePlugin>>>,
    initialized: RwLock<bool>,
}
impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            vcs: RwLock::new(HashMap::new()),
            storage: RwLock::new(HashMap::new()),
            initialized: RwLock::new(false),
        }
    }

    /// Mark registry as initialized.
    ///
    /// After initialization, no new plugins can be registered.
    pub fn finalize(&self) -> PluginResult<()> {
        let mut initialized = self
            .initialized
            .write()
            .map_err(|_| PluginError::Initialization("Poisoned lock".to_string()))?;
        *initialized = true;
        Ok(())
    }

    /// Check if registry is finalized.
    pub fn is_finalized(&self) -> bool {
        self.initialized.read().map(|g| *g).unwrap_or(false)
    }

    // -- VCS plugin management --

    /// Register a VCS adapter plugin.
    pub fn register_vcs(&self, plugin: Box<dyn VcsPlugin>) -> PluginResult<()> {
        if self.is_finalized() {
            return Err(PluginError::Initialization(
                "Registry is finalized, cannot register new plugins".to_string(),
            ));
        }

        let name = plugin.name().to_string();
        let mut vcs = self
            .vcs
            .write()
            .map_err(|_| PluginError::Initialization("Poisoned lock".to_string()))?;

        if vcs.contains_key(&name) {
            return Err(PluginError::AlreadyRegistered(format!(
                "VCS plugin '{}' already registered",
                name
            )));
        }

        vcs.insert(name, Arc::from(plugin));
        Ok(())
    }

    /// Get a VCS adapter by name.
    pub fn vcs(&self, name: &str) -> Option<Arc<dyn VcsPlugin>> {
        self.vcs.read().ok().and_then(|g| g.get(name).cloned())
    }

    /// Get all registered VCS adapter names.
    pub fn vcs_adapters(&self) -> Vec<String> {
        self.vcs
            .read()
            .map(|g| g.keys().cloned().collect())
            .unwrap_or_default()
    }

    // -- Storage plugin management --

    /// Register a storage adapter plugin.
    pub fn register_storage(&self, plugin: Box<dyn StoragePlugin>) -> PluginResult<()> {
        if self.is_finalized() {
            return Err(PluginError::Initialization(
                "Registry is finalized, cannot register new plugins".to_string(),
            ));
        }

        let name = plugin.name().to_string();
        let mut storage = self
            .storage
            .write()
            .map_err(|_| PluginError::Initialization("Poisoned lock".to_string()))?;

        if storage.contains_key(&name) {
            return Err(PluginError::AlreadyRegistered(format!(
                "Storage plugin '{}' already registered",
                name
            )));
        }

        storage.insert(name, Arc::from(plugin));
        Ok(())
    }

    /// Get a storage adapter by name.
    pub fn storage(&self, name: &str) -> Option<Arc<dyn StoragePlugin>> {
        self.storage.read().ok().and_then(|g| g.get(name).cloned())
    }

    /// Get all registered storage adapter names.
    pub fn storage_adapters(&self) -> Vec<String> {
        self.storage
            .read()
            .map(|g| g.keys().cloned().collect())
            .unwrap_or_default()
    }

    // -- Health checks --

    /// Check health of all registered plugins.
    pub async fn health_check(&self) -> PluginResult<()> {
        // Check VCS plugins
        for name in self.vcs_adapters() {
            if let Some(vcs) = self.vcs(&name) {
                vcs.health_check()?;
            }
        }

        // Check storage plugins
        for name in self.storage_adapters() {
            if let Some(storage) = self.storage(&name) {
                storage.health_check()?;
            }
        }

        Ok(())
    }

    /// Get registry statistics.
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            vcs_count: self.vcs_adapters().len(),
            storage_count: self.storage_adapters().len(),
            finalized: self.is_finalized(),
        }
    }
}

/// Statistics about the plugin registry.
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub vcs_count: usize,
    pub storage_count: usize,
    pub finalized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{
        AdapterPlugin, ConflictInfo, FeatureArtifacts, MergeResult, VcsPlugin, WorktreeInfo,
    };
    use std::path::{Path, PathBuf};

    struct MockVcsPlugin;

    impl AdapterPlugin for MockVcsPlugin {
        fn name(&self) -> &str {
            "mock-vcs"
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn initialize(&self, _config: crate::traits::PluginConfig) -> PluginResult<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl VcsPlugin for MockVcsPlugin {
        async fn create_worktree(&self, _: &str, _: &str) -> PluginResult<PathBuf> {
            Ok(PathBuf::from("/tmp/test"))
        }
        async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>> {
            Ok(vec![])
        }
        async fn cleanup_worktree(&self, _: &Path) -> PluginResult<()> {
            Ok(())
        }
        async fn create_branch(&self, _: &str, _: &str) -> PluginResult<()> {
            Ok(())
        }
        async fn checkout_branch(&self, _: &str) -> PluginResult<()> {
            Ok(())
        }
        async fn merge_to_target(&self, _: &str, _: &str) -> PluginResult<MergeResult> {
            Ok(MergeResult {
                success: true,
                conflicts: vec![],
                merged_commit: None,
            })
        }
        async fn detect_conflicts(&self, _: &str, _: &str) -> PluginResult<Vec<ConflictInfo>> {
            Ok(vec![])
        }
        async fn read_artifact(&self, _: &str, _: &str) -> PluginResult<String> {
            Ok(String::new())
        }
        async fn write_artifact(&self, _: &str, _: &str, _: &str) -> PluginResult<()> {
            Ok(())
        }
        async fn artifact_exists(&self, _: &str, _: &str) -> PluginResult<bool> {
            Ok(false)
        }
        async fn scan_feature_artifacts(&self, _: &str) -> PluginResult<FeatureArtifacts> {
            Ok(FeatureArtifacts {
                meta_json: None,
                audit_chain: None,
                evidence_paths: vec![],
            })
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = PluginRegistry::new();
        assert!(!registry.is_finalized());
        assert_eq!(registry.stats().vcs_count, 0);
    }

    #[test]
    fn test_register_vcs_plugin() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(MockVcsPlugin);

        registry.register_vcs(plugin).unwrap();

        assert!(registry.vcs("mock-vcs").is_some());
        assert_eq!(registry.stats().vcs_count, 1);
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(MockVcsPlugin);

        registry.register_vcs(plugin).unwrap();
        let result = registry.register_vcs(Box::new(MockVcsPlugin));

        assert!(result.is_err());
    }

    #[test]
    fn test_finalize_registry() {
        let registry = PluginRegistry::new();
        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();

        registry.finalize().unwrap();
        assert!(registry.is_finalized());

        // Cannot register after finalization
        let result = registry.register_vcs(Box::new(MockVcsPlugin));
        assert!(result.is_err());
    }

    struct MockStoragePlugin;

    impl AdapterPlugin for MockStoragePlugin {
        fn name(&self) -> &str {
            "mock-storage"
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn initialize(&self, _config: crate::traits::PluginConfig) -> PluginResult<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl StoragePlugin for MockStoragePlugin {
        async fn create_feature(&self, _feature: &serde_json::Value) -> PluginResult<i64> {
            Ok(1)
        }
        async fn get_feature_by_slug(
            &self,
            _slug: &str,
        ) -> PluginResult<Option<serde_json::Value>> {
            Ok(None)
        }
        async fn get_feature_by_id(&self, _id: i64) -> PluginResult<Option<serde_json::Value>> {
            Ok(None)
        }
        async fn update_feature_state(&self, _id: i64, _state: &str) -> PluginResult<()> {
            Ok(())
        }
        async fn list_all_features(&self) -> PluginResult<Vec<serde_json::Value>> {
            Ok(vec![])
        }
        async fn create_work_package(&self, _wp: &serde_json::Value) -> PluginResult<i64> {
            Ok(1)
        }
        async fn get_work_package(&self, _id: i64) -> PluginResult<Option<serde_json::Value>> {
            Ok(None)
        }
        async fn update_wp_state(&self, _id: i64, _state: &str) -> PluginResult<()> {
            Ok(())
        }
        async fn append_audit_entry(&self, _entry: &serde_json::Value) -> PluginResult<i64> {
            Ok(1)
        }
        async fn get_audit_trail(&self, _feature_id: i64) -> PluginResult<Vec<serde_json::Value>> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_storage_registration_and_lookup() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(MockStoragePlugin);

        registry.register_storage(plugin).unwrap();

        assert!(registry.storage("mock-storage").is_some());
        assert!(registry
            .storage_adapters()
            .contains(&"mock-storage".to_string()));
        assert_eq!(registry.stats().storage_count, 1);
    }

    #[test]
    fn test_storage_duplicate_registration() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(MockStoragePlugin);

        registry.register_storage(plugin).unwrap();
        let result = registry.register_storage(Box::new(MockStoragePlugin));

        assert!(result.is_err());
    }

    #[test]
    fn test_register_storage_after_finalize() {
        let registry = PluginRegistry::new();
        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();

        registry.finalize().unwrap();

        let result = registry.register_storage(Box::new(MockStoragePlugin));
        assert!(result.is_err());
    }

    #[test]
    fn test_vcs_adapters_empty() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.vcs_adapters(), Vec::<String>::new());

        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();
        assert_eq!(registry.vcs_adapters(), vec!["mock-vcs".to_string()]);
    }

    #[tokio::test]
    async fn test_health_check_with_empty_registry() {
        let registry = PluginRegistry::new();
        let result = registry.health_check().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_registry_equals_new() {
        let default_registry = PluginRegistry::default();
        let new_registry = PluginRegistry::new();

        assert!(!default_registry.is_finalized());
        assert!(!new_registry.is_finalized());
        assert_eq!(default_registry.stats().vcs_count, 0);
        assert_eq!(new_registry.stats().vcs_count, 0);
    }
}
