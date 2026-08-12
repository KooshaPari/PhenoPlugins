# Snapshot: PhenoPlugins Plugin Conformance Roster — 2026-08-11

## Summary

Anchors the canonical plugin roster inside PhenoPlugins' audit
trail. Each plugin listed below either implements the `Plugin`
trait from `pheno-plugin-core` or is in the conformance backlog
with a target promotion date.

## Roster

| Plugin crate | Status | Adapter for | Notes |
|---|---|---|---|
| `pheno-plugin-core` | core | n/a | Defines the `Plugin` trait, `Registry`, lifecycle hooks |
| `pheno-plugin-git` | v1 | Git repositories | Git source-of-truth; integrates with the rest of the fleet |
| `pheno-plugin-sqlite` | v1 | SQLite databases | Local storage adapter; parallel to agileplus-sqlite |
| `pheno-plugin-vessel` | v0 | Vessel runtime | Experimental; orbits pheno-vessel substrate |
| `pheno-plugin-examples` | n/a | n/a | Example plugins; not promoted to v1 |

## Conformance gates

Every plugin must clear the same 5-signal promotion gate as
Benchora's model-family conformance matrix:

1. **Real-world usage** — at least one org agent has run the plugin
   end-to-end.
2. **Lifecycle hooks** — `init / on_register / on_unregister / shutdown`
   all exercised.
3. **Schema declaration** — the plugin's `schema.json` is
   versioned and parseable.
4. **Stability** — across 3 reruns, no panics.
5. **CI green** — `cargo test -p <plugin>` passes on `main`.

## Cross-references

- `PhenoPlugins/PLAN.md` (phases)
- `pheno-plugin-core` (the trait this matrix measures against)
- `BACKLOG-CROSSREPO-001-cluster-2` (PhenoPlugins audits scaffold)
- `BACKLOG-OMLX-003` (Benchora model-family conformance matrix;
  same gate structure)

## Supersedes

None.
