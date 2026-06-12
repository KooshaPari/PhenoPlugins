# PhenoPlugin Manifests — Catalog & Authoring Guide

This directory is the **maintained catalog** of published PhenoPlugin manifests.
A manifest is the declarative, versioned contract that lets a plugin be
discovered, cataloged, and consumed by an agent runtime (including Agentora's
`agentkit`) without code coupling.

See [`docs/PLUGIN_MANIFEST_SPEC.md`](../../PLUGIN_MANIFEST_SPEC.md) for the full
schema and the field-by-field Agentora interop mapping.

## Catalog

[`catalog.json`](catalog.json) is the index. Each entry points at a manifest file:

| Plugin | Kind    | Manifest |
|--------|---------|----------|
| git    | vcs     | [`git.manifest.json`](git.manifest.json) |
| sqlite | storage | [`sqlite.manifest.json`](sqlite.manifest.json) |

## How to author a new PhenoPlugin manifest

1. **Copy an existing manifest** of the closest `kind` (`git.manifest.json` for
   an adapter, or follow the spec for `skill`/`tool`).
2. **Fill the required fields**: `schema_version` (currently `1`), `name`,
   `version` (`MAJOR.MINOR.PATCH`), `description`, `kind`
   (`vcs` | `storage` | `skill` | `tool`), and `entrypoint`.
3. **Describe parameters as JSON Schema** under `parameters`. Use the same shape
   an agent tool expects (`{"type":"object","properties":{...},"required":[...]}`)
   so the manifest is directly consumable by `agentkit`'s `Tool`/`Skill` registry.
4. **Validate** it:
   ```rust
   use pheno_plugin_core::PluginManifest;
   let m = PluginManifest::from_json(&std::fs::read_to_string("my.manifest.json")?)?;
   ```
   or run `cargo test -p pheno-plugin-core` (the catalog round-trip test validates
   every cataloged manifest).
5. **Register it in the catalog**: add an entry to `catalog.json` and the table
   above.

Every manifest in this directory is validated in CI via
`crates/pheno-plugin-core/tests/catalog_test.rs`.
