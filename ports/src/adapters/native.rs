//! NativePluginLoader (libloading).
use super::plugin_loader::{Plugin, PluginError, PluginLoader};
use async_trait::async_trait;
use std::path::Path;

pub struct NativeLoader;

pub struct NativePlugin {
    pub name: String,
}

#[async_trait]
impl Plugin for NativePlugin {
    fn name(&self) -> &str {
        &self.name
    }
    async fn call(&self, _fn_name: &str, _args: &[u8]) -> Result<Vec<u8>, PluginError> {
        Ok(vec![])
    }
}

#[async_trait]
impl PluginLoader for NativeLoader {
    fn backend(&self) -> &str {
        "native"
    }
    async fn load(&self, path: &Path) -> Result<Box<dyn Plugin>, PluginError> {
        Ok(Box::new(NativePlugin {
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string(),
        }))
    }
    async fn unload(&self, _name: &str) -> Result<(), PluginError> {
        Ok(())
    }
    fn list(&self) -> Vec<String> {
        vec![]
    }
}
