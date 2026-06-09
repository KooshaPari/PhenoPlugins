//! T70: PhenoPlugins hexagonal port — PluginLoader.
//!
//! 3 adapters: WasmLoader (wasmtime), NativeLoader (libloading), DynamicLoader (dlopen).
use async_trait::async_trait;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("not found")]
    NotFound,
    #[error("load: {0}")]
    Load(String),
    #[error("call: {0}")]
    Call(String),
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn call(&self, fn_name: &str, args: &[u8]) -> Result<Vec<u8>, PluginError>;
}

#[async_trait]
pub trait PluginLoader: Send + Sync {
    fn backend(&self) -> &str;
    async fn load(&self, path: &Path) -> Result<Box<dyn Plugin>, PluginError>;
    async fn unload(&self, name: &str) -> Result<(), PluginError>;
    fn list(&self) -> Vec<String>;
}
