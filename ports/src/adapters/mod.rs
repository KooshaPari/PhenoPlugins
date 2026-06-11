//! Adapter implementations of the [`crate::plugin_loader::PluginLoader`] port.
//!
//! - [`native`] — NativeLoader (libloading)
//! - [`wasm`]    — WasmLoader (wasmtime)

pub mod native;
pub mod wasm;
