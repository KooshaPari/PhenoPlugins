//! T70: PhenoPlugins hexagonal port — PluginLoader.
//!
//! Adapters live under [`adapters`]. Domain code depends on the
//! [`plugin_loader::PluginLoader`] trait, not on any specific loader
//! implementation (libloading / wasmtime / dlopen).
//!
//! SOTA pattern: the port trait is declared upfront; adapters are
//! implemented against it. Dead-code warnings on the adapters are
//! expected until the application crate starts using them.

pub mod adapters;
pub mod plugin_loader;
