//! External smoke test for `pheno-plugin-sqlite`.
//!
//! Validates the public surface of the SQLite storage plugin against an
//! in-memory database. This is the first external (integration) test for
//! the crate — the only test coverage so far is internal `#[cfg(test)] mod
//! tests` blocks inside `src/lib.rs`. An external test exercises the
//! crate's compiled library the way a downstream consumer would.
//!
//! Traces to: FR-ORG-AUDIT-2026-04-001

use pheno_plugin_core::traits::{AdapterPlugin, PluginConfig};
use pheno_plugin_sqlite::SqliteStoragePlugin;

#[test]
fn sqlite_in_memory_plugin_smoke() {
    // Construct the plugin via the in-memory constructor exposed in lib.rs.
    let plugin = SqliteStoragePlugin::in_memory()
        .expect("in-memory SQLite plugin should construct");

    // `AdapterPlugin::initialize` should succeed against a freshly-migrated
    // in-memory database with an empty config.
    plugin
        .initialize(PluginConfig {
            name: "sqlite-storage".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            adapter_config: serde_json::json!({}),
        })
        .expect("initialize should succeed against an in-memory database");

    // AdapterPlugin metadata is correct.
    assert_eq!(plugin.name(), "sqlite-storage");
    assert_eq!(plugin.version(), env!("CARGO_PKG_VERSION"));
}
