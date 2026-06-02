//! Round-trips every manifest in the published catalog through the parser/validator.
//!
//! Traces to: FR-PHENOPLUGINS-007

use std::path::PathBuf;

use pheno_plugin_core::PluginManifest;

fn catalog_dir() -> PathBuf {
    // crate is at <repo>/crates/pheno-plugin-core; catalog at <repo>/docs/journeys/manifests
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/journeys/manifests")
}

#[test]
fn every_cataloged_manifest_parses_and_validates() {
    let dir = catalog_dir();
    let index = std::fs::read_to_string(dir.join("catalog.json")).expect("read catalog.json");
    let index: serde_json::Value = serde_json::from_str(&index).expect("parse catalog.json");

    let plugins = index["plugins"].as_array().expect("plugins array");
    assert!(!plugins.is_empty(), "catalog must list at least one plugin");

    for entry in plugins {
        let file = entry["manifest"].as_str().expect("manifest filename");
        let json =
            std::fs::read_to_string(dir.join(file)).unwrap_or_else(|e| panic!("read {file}: {e}"));
        let manifest = PluginManifest::from_json(&json)
            .unwrap_or_else(|e| panic!("{file} failed validation: {e}"));

        // Catalog index and manifest must agree on the name.
        assert_eq!(
            manifest.name,
            entry["name"].as_str().unwrap(),
            "name mismatch in {file}"
        );
    }
}
