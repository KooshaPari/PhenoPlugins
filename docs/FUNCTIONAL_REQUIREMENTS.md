# Functional Requirements

Specification document for PHENOPLUGINS module.

## Overview

This document enumerates the functional requirements that guide implementation, testing, and
quality validation for this project. Each FR has an assigned identifier for cross-reference
in tests, PRs, and architectural documentation.

## Functional Requirements

### FR-PHENOPLUGINS-006

**Description:** Persistent data storage

**Status:** SCAFFOLD

**Test Traces:** (pending implementation)

---

### FR-PHENOPLUGINS-007

**Description:** Versioned plugin manifest schema + maintained catalog providing the
interop contract between the PhenoPlugins plugin model and an external agent runtime
(Agentora `agentkit` `Skill`/`Tool`). A manifest declares name, version, description,
kind, capabilities, JSON-Schema parameters, and entrypoint; the catalog indexes
published manifests and is validated in CI. See `docs/PLUGIN_MANIFEST_SPEC.md`.

**Status:** IMPLEMENTED

**Test Traces:** `crates/pheno-plugin-core/src/manifest.rs` (unit tests),
`crates/pheno-plugin-core/tests/catalog_test.rs` (catalog round-trip)

---

## Traceability

All tests MUST reference at least one FR using this marker:

```rust
// Traces to: FR-<REPOID>-NNN
#[test]
fn test_feature_name() { }
```

Every FR must have at least one corresponding test. Use the pattern above to link test to requirement.
