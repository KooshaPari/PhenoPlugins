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

    // ============================================================================
    // Additional tests for untested surface: mixed registries, health checks,
    // scalability, name-space separation, and derived-trait behavior.
    // ============================================================================

    // Macro to implement VcsPlugin with no-op defaults — avoids duplicating
    // ~40 lines per mock type.
    macro_rules! impl_vcs_default {
        ($t:ty) => {
            #[async_trait::async_trait]
            impl VcsPlugin for $t {
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
                async fn detect_conflicts(
                    &self,
                    _: &str,
                    _: &str,
                ) -> PluginResult<Vec<ConflictInfo>> {
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
        };
    }

    // Macro to implement StoragePlugin with no-op defaults.
    macro_rules! impl_storage_default {
        ($t:ty) => {
            #[async_trait::async_trait]
            impl StoragePlugin for $t {
                async fn create_feature(
                    &self,
                    _feature: &serde_json::Value,
                ) -> PluginResult<i64> {
                    Ok(1)
                }
                async fn get_feature_by_slug(
                    &self,
                    _slug: &str,
                ) -> PluginResult<Option<serde_json::Value>> {
                    Ok(None)
                }
                async fn get_feature_by_id(
                    &self,
                    _id: i64,
                ) -> PluginResult<Option<serde_json::Value>> {
                    Ok(None)
                }
                async fn update_feature_state(
                    &self,
                    _id: i64,
                    _state: &str,
                ) -> PluginResult<()> {
                    Ok(())
                }
                async fn list_all_features(&self) -> PluginResult<Vec<serde_json::Value>> {
                    Ok(vec![])
                }
                async fn create_work_package(
                    &self,
                    _wp: &serde_json::Value,
                ) -> PluginResult<i64> {
                    Ok(1)
                }
                async fn get_work_package(
                    &self,
                    _id: i64,
                ) -> PluginResult<Option<serde_json::Value>> {
                    Ok(None)
                }
                async fn update_wp_state(&self, _id: i64, _state: &str) -> PluginResult<()> {
                    Ok(())
                }
                async fn append_audit_entry(
                    &self,
                    _entry: &serde_json::Value,
                ) -> PluginResult<i64> {
                    Ok(1)
                }
                async fn get_audit_trail(
                    &self,
                    _feature_id: i64,
                ) -> PluginResult<Vec<serde_json::Value>> {
                    Ok(vec![])
                }
            }
        };
    }

    /// A VCS plugin with a configurable name. Used in tests that need many
    /// distinct plugin names.
    struct NamedVcsPlugin {
        name: String,
    }

    impl NamedVcsPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl AdapterPlugin for NamedVcsPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn initialize(&self, _config: crate::traits::PluginConfig) -> PluginResult<()> {
            Ok(())
        }
    }

    impl_vcs_default!(NamedVcsPlugin);

    /// A Storage plugin with a configurable name.
    struct NamedStoragePlugin {
        name: String,
    }

    impl NamedStoragePlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl AdapterPlugin for NamedStoragePlugin {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn initialize(&self, _config: crate::traits::PluginConfig) -> PluginResult<()> {
            Ok(())
        }
    }

    impl_storage_default!(NamedStoragePlugin);

    #[tokio::test]
    async fn test_registry_with_both_storage_and_vcs() {
        let registry = PluginRegistry::new();
        // 2 storage plugins.
        registry
            .register_storage(Box::new(MockStoragePlugin))
            .unwrap();
        registry
            .register_storage(Box::new(NamedStoragePlugin::new("storage-two")))
            .unwrap();
        // 3 VCS plugins.
        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();
        registry
            .register_vcs(Box::new(NamedVcsPlugin::new("vcs-two")))
            .unwrap();
        registry
            .register_vcs(Box::new(NamedVcsPlugin::new("vcs-three")))
            .unwrap();

        let stats = registry.stats();
        assert_eq!(stats.storage_count, 2);
        assert_eq!(stats.vcs_count, 3);
        // health_check must succeed across both kinds.
        assert!(registry.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_registry_health_check_with_mixed_plugins() {
        let registry = PluginRegistry::new();
        registry
            .register_storage(Box::new(MockStoragePlugin))
            .unwrap();
        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();

        let result = registry.health_check().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_with_many_plugins() {
        let registry = PluginRegistry::new();
        // 20 storage plugins.
        for i in 0..20 {
            let name = format!("storage-{:02}", i);
            registry
                .register_storage(Box::new(NamedStoragePlugin::new(&name)))
                .unwrap();
        }
        // 15 VCS plugins.
        for i in 0..15 {
            let name = format!("vcs-{:02}", i);
            registry
                .register_vcs(Box::new(NamedVcsPlugin::new(&name)))
                .unwrap();
        }

        let stats = registry.stats();
        assert_eq!(stats.storage_count, 20);
        assert_eq!(stats.vcs_count, 15);

        // All 35 plugins must be findable by name.
        for i in 0..20 {
            let name = format!("storage-{:02}", i);
            assert!(
                registry.storage(&name).is_some(),
                "missing storage plugin: {}",
                name
            );
        }
        for i in 0..15 {
            let name = format!("vcs-{:02}", i);
            assert!(
                registry.vcs(&name).is_some(),
                "missing VCS plugin: {}",
                name
            );
        }
    }

    #[test]
    fn test_storage_and_vcs_with_same_name_both_allowed() {
        let registry = PluginRegistry::new();
        // Storage and VCS plugins live in separate maps, so a name collision
        // across the two kinds is allowed — both registrations should succeed.
        registry
            .register_storage(Box::new(NamedStoragePlugin::new("shared")))
            .unwrap();
        registry
            .register_vcs(Box::new(NamedVcsPlugin::new("shared")))
            .unwrap();

        assert!(registry.storage("shared").is_some());
        assert!(registry.vcs("shared").is_some());
        assert_eq!(registry.stats().storage_count, 1);
        assert_eq!(registry.stats().vcs_count, 1);
    }

    #[test]
    fn test_lookup_storage_after_unrelated_vcs_registered() {
        let registry = PluginRegistry::new();
        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();
        registry
            .register_vcs(Box::new(NamedVcsPlugin::new("another-vcs")))
            .unwrap();

        // Storage lookups must not see VCS plugins — the maps are separate.
        assert!(registry.storage("mock-vcs").is_none());
        assert!(registry.storage("another-vcs").is_none());
        assert!(registry.storage("never-existed").is_none());
    }

    #[test]
    fn test_lookup_vcs_after_unrelated_storage_registered() {
        let registry = PluginRegistry::new();
        registry
            .register_storage(Box::new(MockStoragePlugin))
            .unwrap();
        registry
            .register_storage(Box::new(NamedStoragePlugin::new("another-storage")))
            .unwrap();

        // VCS lookups must not see storage plugins — the maps are separate.
        assert!(registry.vcs("mock-storage").is_none());
        assert!(registry.vcs("another-storage").is_none());
        assert!(registry.vcs("never-existed").is_none());
    }

    #[test]
    fn test_registry_clone_or_not() {
        // PluginRegistry does NOT currently derive Clone. The struct owns
        // `Arc<dyn VcsPlugin>` / `Arc<dyn StoragePlugin>` plus `RwLock<bool>`,
        // and the design is single-ownership per process — sharing is done by
        // wrapping the registry in an Arc at the call site if needed.
        //
        // This test documents the design choice by exercising the parts of
        // the API that are cloneable (`RegistryStats` derives Clone) and
        // by verifying the registry remains usable after method calls.
        //
        // If a `#[derive(Clone)]` is ever added to PluginRegistry, replace
        // this test with a real `registry.clone()` assertion.
        let registry = PluginRegistry::new();
        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();
        let stats: RegistryStats = registry.stats();
        let _stats_clone = stats.clone();
        assert_eq!(stats.vcs_count, 1);
    }

    #[test]
    fn test_registry_debug_format() {
        // PluginRegistry itself does NOT implement Debug — it owns
        // `Arc<dyn VcsPlugin>` and `Arc<dyn StoragePlugin>`, neither of which
        // auto-derive Debug.
        //
        // This test documents that limitation by exercising Debug on the
        // parts of the API that DO derive Debug: `RegistryStats` and the
        // `Vec<String>` adapter-name lists returned by `vcs_adapters()` /
        // `storage_adapters()`. If a manual Debug impl is added to
        // PluginRegistry in the future, extend this test to format it.
        let registry = PluginRegistry::new();
        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();
        registry
            .register_vcs(Box::new(NamedVcsPlugin::new("vcs-two")))
            .unwrap();
        registry
            .register_storage(Box::new(MockStoragePlugin))
            .unwrap();

        // Debug on RegistryStats.
        let stats_debug = format!("{:?}", registry.stats());
        assert!(
            stats_debug.contains("RegistryStats"),
            "expected 'RegistryStats' in {:?}",
            stats_debug
        );
        assert!(
            stats_debug.contains("vcs_count"),
            "expected 'vcs_count' in {:?}",
            stats_debug
        );
        assert!(
            stats_debug.contains("storage_count"),
            "expected 'storage_count' in {:?}",
            stats_debug
        );

        // Debug on the adapter name lists (Vec<String> is Debug).
        let vcs_names = registry.vcs_adapters();
        let vcs_debug = format!("{:?}", vcs_names);
        assert!(vcs_debug.contains("mock-vcs"));
        assert!(vcs_debug.contains("vcs-two"));

        let storage_names = registry.storage_adapters();
        let storage_debug = format!("{:?}", storage_names);
        assert!(storage_debug.contains("mock-storage"));
    }

    #[test]
    fn test_registry_size_after_drops() {
        let registry = PluginRegistry::new();
        for i in 0..5 {
            let name = format!("vcs-{}", i);
            registry
                .register_vcs(Box::new(NamedVcsPlugin::new(&name)))
                .unwrap();
        }
        assert_eq!(registry.stats().vcs_count, 5);

        // Look up 3 plugins and immediately drop the returned Arcs.
        for i in 0..3 {
            let name = format!("vcs-{}", i);
            let arc = registry.vcs(&name);
            assert!(arc.is_some());
            drop(arc);
        }

        // The registry holds its own Arc references, so dropping the lookups
        // must not reduce its size.
        assert_eq!(registry.stats().vcs_count, 5);
    }
}
