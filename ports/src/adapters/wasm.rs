//! WasmPluginLoader (wasmtime).
use super::plugin_loader::{Plugin, PluginError, PluginLoader};
use async_trait::async_trait;
use std::path::Path;

pub struct WasmLoader;

pub struct WasmPlugin {
    pub name: String,
}

#[async_trait]
impl Plugin for WasmPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    async fn call(&self, _fn_name: &str, _args: &[u8]) -> Result<Vec<u8>, PluginError> {
        Ok(vec![])
    }
}

#[async_trait]
impl PluginLoader for WasmLoader {
    fn backend(&self) -> &str {
        "wasm"
    }
    async fn load(&self, path: &Path) -> Result<Box<dyn Plugin>, PluginError> {
        Ok(Box::new(WasmPlugin {
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
