# phenoVessel archive history

This directory preserves AgilePlus specs and traceability artifacts from the archived
[KooshaPari/phenoVessel](https://github.com/KooshaPari/phenoVessel) repository.

## Absorption

| Field | Value |
|-------|-------|
| Source repo | `KooshaPari/phenoVessel` (archived) |
| Target | `PhenoPlugins/crates/pheno-plugin-vessel` |
| Migration date | 2026-06-16 |

The runtime crate modules (`client`, `compose`, `container`, `image`, `runtime`) were
already present in `pheno-plugin-vessel` prior to this migration. This directory retains
specs and traceability for audit purposes only.

## Contents

- `001-vessel-core/` — core vessel specification and tasks
- `architecture-decisions/` — ADR YAML artifacts
- `functional-requirements/` — FR YAML artifacts
- `user-stories/` — user story YAML artifacts
- `index.yaml`, `traceability-matrix.yaml` — spec index and traceability matrix
