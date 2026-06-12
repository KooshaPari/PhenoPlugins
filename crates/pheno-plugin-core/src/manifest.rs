//! Plugin manifest schema — the declarative interop contract for PhenoPlugins.
//!
//! A [`PluginManifest`] is the serializable, versioned description of a plugin.
//! It is the bridge that lets a PhenoPlugins adapter be discovered, cataloged,
//! and consumed by an external agent runtime (notably Agentora's `agentkit`),
//! without either side depending on the other's code.
//!
//! ## Agentora interop
//!
//! The `agentkit` runtime defines `Tool { name, description, parameters() -> JSON Schema }`
//! and `Skill { name, description }`. A [`PluginManifest`] of `kind = "skill"` or
//! `kind = "tool"` carries exactly those fields: [`PluginManifest::name`],
//! [`PluginManifest::description`], and [`PluginManifest::parameters`] (an embedded
//! JSON Schema). A runtime can therefore register a PhenoPlugin from its manifest
//! alone — see `docs/PLUGIN_MANIFEST_SPEC.md` for the field-by-field mapping.
//!
//! ## Relationship to [`crate::traits::PluginConfig`]
//!
//! `PluginConfig` is the *runtime* init payload (name + version + opaque config).
//! `PluginManifest` is the *design-time* contract (the full, validated descriptor
//! from which a `PluginConfig` can be derived via [`PluginManifest::to_config`]).

use serde::{Deserialize, Serialize};

use crate::error::{PluginError, PluginResult};
use crate::traits::PluginConfig;

/// Current manifest schema version. Bumped on breaking schema changes.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// The category of capability a plugin provides.
///
/// `Vcs` and `Storage` map to PhenoPlugins' own adapter traits; `Skill` and
/// `Tool` map to Agentora `agentkit`'s `Skill`/`Tool` runtime contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginKind {
    /// Version-control adapter (implements `VcsPlugin`).
    Vcs,
    /// Storage adapter (implements `StoragePlugin`).
    Storage,
    /// An agent skill (maps to `agentkit::Skill`).
    Skill,
    /// An agent tool (maps to `agentkit::Tool`).
    Tool,
}

/// Declarative, versioned description of a single plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Manifest schema version this document conforms to.
    pub schema_version: u32,
    /// Plugin name, unique within its kind (e.g. `"git"`, `"sqlite"`).
    pub name: String,
    /// Plugin version, semantic-versioning `MAJOR.MINOR.PATCH`.
    pub version: String,
    /// Human-readable description of what the plugin does.
    pub description: String,
    /// What category of capability the plugin provides.
    pub kind: PluginKind,
    /// Free-form capability tags (e.g. `["worktree", "branch"]`). Optional.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// JSON Schema describing the plugin's parameters.
    ///
    /// Shaped to match `agentkit`'s `Tool::parameters` default
    /// (`{"type":"object","properties":{},"required":[]}`) so a manifest is
    /// directly consumable by the Agentora runtime.
    #[serde(default = "default_parameters")]
    pub parameters: serde_json::Value,
    /// Entry point identifier the host uses to load the plugin
    /// (e.g. a crate path, dynamic-library name, or registry key).
    pub entrypoint: String,
}

fn default_parameters() -> serde_json::Value {
    serde_json::json!({ "type": "object", "properties": {}, "required": [] })
}

impl PluginManifest {
    /// Parse a manifest from a JSON string and validate it.
    pub fn from_json(json: &str) -> PluginResult<Self> {
        let manifest: PluginManifest = serde_json::from_str(json)
            .map_err(|e| PluginError::Config(format!("invalid manifest JSON: {e}")))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest against the schema rules.
    ///
    /// Checks: supported schema version, non-empty `name`/`description`/`entrypoint`,
    /// and a well-formed `MAJOR.MINOR.PATCH` semantic version.
    pub fn validate(&self) -> PluginResult<()> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(PluginError::Config(format!(
                "unsupported manifest schema_version {} (expected {MANIFEST_SCHEMA_VERSION})",
                self.schema_version
            )));
        }
        if self.name.trim().is_empty() {
            return Err(PluginError::Config("manifest name is empty".into()));
        }
        if self.description.trim().is_empty() {
            return Err(PluginError::Config("manifest description is empty".into()));
        }
        if self.entrypoint.trim().is_empty() {
            return Err(PluginError::Config("manifest entrypoint is empty".into()));
        }
        validate_semver(&self.version)?;
        Ok(())
    }

    /// Derive the runtime [`PluginConfig`] this manifest describes.
    pub fn to_config(&self) -> PluginConfig {
        PluginConfig {
            name: self.name.clone(),
            version: self.version.clone(),
            adapter_config: serde_json::Value::Null,
        }
    }
}

/// Validate a `MAJOR.MINOR.PATCH` semantic version without pulling a new dependency.
fn validate_semver(version: &str) -> PluginResult<()> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(PluginError::Config(format!(
            "version '{version}' must be MAJOR.MINOR.PATCH"
        )));
    }
    for part in parts {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(PluginError::Config(format!(
                "version '{version}' has a non-numeric component"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest_json() -> &'static str {
        r#"{
            "schema_version": 1,
            "name": "git",
            "version": "0.1.0",
            "description": "Git worktree VCS adapter",
            "kind": "vcs",
            "capabilities": ["worktree", "branch"],
            "parameters": {"type": "object", "properties": {}, "required": []},
            "entrypoint": "pheno-plugin-git"
        }"#
    }

    // Traces to: FR-PHENOPLUGINS-007
    #[test]
    fn parses_and_validates_a_well_formed_manifest() {
        let m = PluginManifest::from_json(valid_manifest_json()).expect("should parse");
        assert_eq!(m.name, "git");
        assert_eq!(m.kind, PluginKind::Vcs);
        assert_eq!(m.capabilities, vec!["worktree", "branch"]);
    }

    // Traces to: FR-PHENOPLUGINS-007
    #[test]
    fn rejects_unsupported_schema_version() {
        let json = valid_manifest_json().replace("\"schema_version\": 1", "\"schema_version\": 99");
        assert!(PluginManifest::from_json(&json).is_err());
    }

    // Traces to: FR-PHENOPLUGINS-007
    #[test]
    fn rejects_empty_name() {
        let json = valid_manifest_json().replace("\"name\": \"git\"", "\"name\": \"\"");
        assert!(PluginManifest::from_json(&json).is_err());
    }

    // Traces to: FR-PHENOPLUGINS-007
    #[test]
    fn rejects_malformed_version() {
        let json = valid_manifest_json().replace("\"version\": \"0.1.0\"", "\"version\": \"1.0\"");
        assert!(PluginManifest::from_json(&json).is_err());
        let json2 =
            valid_manifest_json().replace("\"version\": \"0.1.0\"", "\"version\": \"a.b.c\"");
        assert!(PluginManifest::from_json(&json2).is_err());
    }

    // Traces to: FR-PHENOPLUGINS-007
    #[test]
    fn rejects_empty_entrypoint() {
        let json = valid_manifest_json().replace(
            "\"entrypoint\": \"pheno-plugin-git\"",
            "\"entrypoint\": \"\"",
        );
        assert!(PluginManifest::from_json(&json).is_err());
    }

    // Traces to: FR-PHENOPLUGINS-007
    #[test]
    fn parameters_default_when_omitted() {
        let json = r#"{
            "schema_version": 1,
            "name": "sqlite",
            "version": "0.1.0",
            "description": "SQLite storage adapter",
            "kind": "storage",
            "entrypoint": "pheno-plugin-sqlite"
        }"#;
        let m = PluginManifest::from_json(json).expect("should parse with defaults");
        assert_eq!(m.parameters, default_parameters());
        assert!(m.capabilities.is_empty());
    }

    // Traces to: FR-PHENOPLUGINS-007
    #[test]
    fn derives_runtime_config_from_manifest() {
        let m = PluginManifest::from_json(valid_manifest_json()).unwrap();
        let cfg = m.to_config();
        assert_eq!(cfg.name, "git");
        assert_eq!(cfg.version, "0.1.0");
    }
}
