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

**Description:** Plugin lifecycle state-machine contract (5×5 transition matrix).

`pheno_plugin_core::lifecycle::PluginState` defines exactly five states:
`Registered`, `Initialized`, `Running`, `Stopped`, `Failed`. The full
contract — including which transitions are allowed, the natural-language
rationale for each cell, and the error variant returned for forbidden
transitions — lives in `crates/pheno-plugin-core/tests/fixtures/lifecycle_state_machine.json`,
which is the single source of truth consumed by the fixture-driven oracle
at `crates/pheno-plugin-core/tests/lifecycle_fixture_oracle.rs`.

The matrix covers 5 × 5 = 25 cells. Each row declares `allowed: true|false`
plus a `rationale` string used as living documentation. The in-code
predicate `PluginState::can_transition_to(a, b)` and the method
`PluginState::transition(a, b) -> PluginResult<PluginState>` MUST agree
with the fixture for every cell; forbidden transitions MUST surface as
`PluginError::Validation(String)` carrying `ErrorCode::Validation`, with
a `Display` message that names both `from` and `to` states.

When a new variant is added, the fixture MUST be updated to declare the
new state (one variant, 5 new transition rows) and the
`test_fixture_schema_is_well_formed` oracle will surface a missing-row
panic at build time if the update is incomplete.

**Status:** SCAFFOLD

**Test Traces:**
- `crates/pheno-plugin-core/tests/lifecycle_fixture_oracle.rs::test_fixture_schema_is_well_formed` — pins 5 states, 5×5 = 25 rows, no duplicates or missing cells.
- `crates/pheno-plugin-core/tests/lifecycle_fixture_oracle.rs::test_fixture_allowed_transitions_pass_in_code` — positive oracle: every `allowed: true` cell succeeds in `can_transition_to` and `transition()`.
- `crates/pheno-plugin-core/tests/lifecycle_fixture_oracle.rs::test_fixture_forbidden_transitions_rejected_in_code` — negative oracle: every `allowed: false` cell returns `PluginError::Validation(_)` carrying `ErrorCode::Validation`, with a display message naming both endpoints.
- `crates/pheno-plugin-core/tests/lifecycle_fixture_oracle.rs::test_fixture_predicate_and_method_agree_for_every_cell` — coherence oracle: predicate ↔ method ↔ fixture agree for every cell.

---

## Traceability

All tests MUST reference at least one FR using this marker:

```rust
// Traces to: FR-<REPOID>-NNN
#[test]
fn test_feature_name() { }
```

Every FR must have at least one corresponding test. Use the pattern above to link test to requirement.
