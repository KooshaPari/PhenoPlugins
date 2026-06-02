# PhenoPlugin Manifest Specification

**Status:** v1 · **Schema version:** `1` · Traces to: `FR-PHENOPLUGINS-007`

The plugin manifest is the **interop contract** for PhenoPlugins. It is a
serializable, versioned descriptor of a single plugin that allows the plugin to
be cataloged and consumed by an external agent runtime — in particular
Agentora's `agentkit` — without either side depending on the other's code.

## Format

A manifest is a JSON document. JSON (not TOML) was chosen because the
`parameters` field is an embedded JSON Schema and `agentkit`'s tool/skill
parameter contract is already JSON-valued (`serde_json::Value`), so a manifest's
`parameters` can be handed to the runtime verbatim with no transcoding.

## Fields

| Field            | Type        | Required | Description |
|------------------|-------------|----------|-------------|
| `schema_version` | integer     | yes      | Manifest schema version. Must equal `1`. |
| `name`           | string      | yes      | Plugin name, unique within its `kind`. Non-empty. |
| `version`        | string      | yes      | `MAJOR.MINOR.PATCH` semantic version. |
| `description`    | string      | yes      | Human-readable summary. Non-empty. |
| `kind`           | enum        | yes      | One of `vcs`, `storage`, `skill`, `tool`. |
| `capabilities`   | string[]    | no       | Free-form capability tags. Defaults to `[]`. |
| `parameters`     | JSON Schema | no       | Parameter schema. Defaults to `{"type":"object","properties":{},"required":[]}`. |
| `entrypoint`     | string      | yes      | Identifier the host uses to load the plugin (crate path / dylib / registry key). Non-empty. |

Validation is implemented in
[`crates/pheno-plugin-core/src/manifest.rs`](../crates/pheno-plugin-core/src/manifest.rs)
(`PluginManifest::from_json` / `validate`).

## Agentora (`agentkit`) interop mapping

`agentkit` defines (verified in `Agentora/src/domain/`):

- `Tool { fn name() -> &str; fn description() -> String; fn parameters() -> serde_json::Value /* JSON Schema */; }`
- `Skill { fn name() -> &str; fn description() -> String; }`

Mapping from a manifest of `kind = "tool"` (or `"skill"`):

| `agentkit` member     | Manifest field           |
|-----------------------|--------------------------|
| `Tool::name()`        | `name`                   |
| `Tool::description()` | `description`            |
| `Tool::parameters()`  | `parameters` (verbatim)  |
| `Skill::name()`       | `name`                   |
| `Skill::description()`| `description`            |

Because `parameters` is already a JSON-Schema `serde_json::Value` matching
`agentkit`'s default tool-parameter shape, a runtime can register a PhenoPlugin
from its manifest alone — closing the previously-missing interop gap between the
PhenoPlugins plugin model and the Agentora skill/tool system.

## Catalog

Published manifests live in
[`docs/journeys/manifests/`](journeys/manifests/) and are indexed by
`catalog.json`. Every cataloged manifest is validated in CI by
`crates/pheno-plugin-core/tests/catalog_test.rs`.
