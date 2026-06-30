# PhenoPlugins — Threat Model

_Last updated: 2026-06-30. Covers v0.1.x plugin SDK._

---

## 1. Scope

This document covers the PhenoPlugins workspace:

- `pheno-plugin-core` — registry, traits, error types, lifecycle.
- `pheno-plugin-git` — VCS adapter (git2 / gitoxide).
- `pheno-plugin-sqlite` — storage adapter (rusqlite, WAL mode).
- `pheno-plugin-vessel` — container utility scaffolding.
- `pheno-plugin-examples` — reference in-memory adapters (not production code).

Host applications (e.g. AgilePlus) embed this SDK. They own process isolation,
network exposure, and end-user authentication. This document calls out which
security properties PhenoPlugins enforces itself vs. which it delegates to the
host.

---

## 2. Trust Boundaries

```
┌───────────────────────────────────────────────────────────┐
│  Host Process (e.g. AgilePlus)                            │
│  ┌────────────────────────────────────────────────────┐   │
│  │  PluginRegistry  (pheno-plugin-core)               │   │
│  │  ┌──────────────┐  ┌───────────────┐               │   │
│  │  │ VcsPlugin    │  │ StoragePlugin │               │   │
│  │  │ (git adapter)│  │ (sqlite)      │               │   │
│  │  └──────┬───────┘  └──────┬────────┘              │   │
│  └─────────┼─────────────────┼───────────────────────┘   │
│            │ fs/git           │ SQLite file               │
└────────────┼─────────────────┼───────────────────────────┘
             ▼                 ▼
       Git repository     SQLite database file
       (local FS)         (local FS)
```

**Trust boundary T1 — Plugin SDK / Host boundary.**
The host controls what plugin implementations are registered. The SDK trusts
all registered adapters as host-provided code. Malicious adapter injection is
a host security concern (process isolation / code signing).

**Trust boundary T2 — Plugin SDK / filesystem.**
`pheno-plugin-git` calls `git2` to operate on local repositories.
`pheno-plugin-sqlite` opens SQLite files. Both operate on paths supplied by
the host at construction time. The SDK does not validate that paths are within
a safe prefix; this is the host's responsibility.

**Trust boundary T3 — Plugin SDK / data supplied to trait methods.**
Method arguments (feature slugs, artifact paths, JSON blobs) come from the
host. The SDK performs Rust type-level validation only; semantic/business
validation is the host's responsibility.

---

## 3. Attacker Model

| Actor | Goal | Capability |
|---|---|---|
| Malicious adapter | Escape sandbox, corrupt shared state | Can register an arbitrary `Box<dyn VcsPlugin>` if the host permits it |
| Malicious input | Path traversal, SQL injection, JSON bomb | Controls string arguments passed to trait methods |
| Local privilege escalation | Read other tenants' SQLite data | Access to the filesystem at the SQLite path |
| Dependency supply chain | Inject code via transitive crate update | Upstream crate compromise |

---

## 4. STRIDE Analysis

### S — Spoofing

- **Risk:** An attacker-controlled adapter registered under a trusted name
  (e.g. `"git"`) would spoof the legitimate VCS plugin.
- **SDK mitigation:** `PluginRegistry::register_vcs` returns
  `Err(AlreadyRegistered)` if the name is already taken, and
  `register_*` rejects all calls after `finalize()`.
- **Host obligation:** Call `registry.finalize()` immediately after registering
  all trusted adapters and before handling any external input.

### T — Tampering

- **Risk:** Concurrent writers could race-write the same plugin slot.
- **SDK mitigation:** `RwLock<HashMap<…>>` serialises all writes.
  Registration is idempotent-safe (duplicate name → error, not silent
  overwrite).
- **Risk:** SQLite WAL file could be modified on disk by another process.
- **SDK mitigation:** Foreign-key enforcement and WAL mode are enabled.
  Row-level integrity is not enforced by the SDK; use filesystem permissions.

### R — Repudiation

- **Risk:** Plugin lifecycle events are not audited.
- **SDK mitigation (partial):** `tracing` spans are emitted for register,
  lookup, and health-check events. The host must wire a `tracing` subscriber
  to capture and persist these events.
- **Gap:** No append-only audit log is embedded in the SDK. The SQLite adapter
  has an `audit_entries` table for feature-level events; plugin lifecycle
  events are tracing-only.

### I — Information Disclosure

- **Risk:** Error messages may expose filesystem paths.
- **SDK mitigation:** `PluginError` variants accept free-form strings from
  adapters. Host adapters (e.g. `pheno-plugin-git`) must scrub paths from
  error context before surfacing them to untrusted callers.
- **Risk:** SQLite file contains all feature + work-package data in plaintext.
- **SDK mitigation:** None. Encryption at rest is the host's responsibility
  (e.g. filesystem encryption, SQLCipher).

### D — Denial of Service

- **Risk:** Unbounded plugin registration (memory exhaustion).
- **SDK mitigation:** None currently. The host should impose a plugin count
  ceiling and call `finalize()` early.
- **Risk:** Health-check fan-out blocks indefinitely if an adapter hangs.
- **SDK mitigation:** `PluginRegistry::health_check` is async but has no
  timeout. The host must wrap the call with `tokio::time::timeout`.

### E — Elevation of Privilege

- **Risk:** A VCS adapter executing `git` operations could traverse outside
  the intended repository path.
- **SDK mitigation:** The SDK does not restrict paths; the host must supply
  a canonicalized, allowlisted `repo_path` at adapter construction.
- **Risk:** SQLite `ATTACH` or `pragma` injection through user-controlled data.
- **SDK mitigation:** `pheno-plugin-sqlite` uses parameterized queries
  (`rusqlite::params!`) for all data operations. Schema DDL is hardcoded and
  not user-controlled.

---

## 5. Tenancy and Authorization Position

**Current state:** PhenoPlugins is a **single-tenant library SDK**. There is no
runtime concept of user identity, session token, or tenant isolation.

The SQLite schema (`crates/pheno-plugin-sqlite/src/lib.rs`) does not include a
`tenant_id` column. All rows are visible to any caller with access to the
plugin instance.

**Host obligation for multi-tenant use:**

1. Instantiate a separate `SqliteStoragePlugin` per tenant, each backed by a
   distinct SQLite file path with appropriate filesystem permissions.
2. Never share a single `PluginRegistry` instance across tenant contexts unless
   the host enforces access control above the registry layer.
3. Authorization (who may call which trait method) is entirely the host's
   responsibility. The SDK does not verify caller identity.

**Future work (L21/L23):** If PhenoPlugins is ever promoted to a multi-tenant
service SDK, the storage trait must be extended with a mandatory `tenant_id`
argument and the SQLite schema must add a `tenant_id TEXT NOT NULL` column with
row-level indexing and check constraints.

---

## 6. Supply Chain

- `cargo deny` is configured (`deny.toml`) to block known-vulnerable and
  duplicate crates.
- Renovate is active for automated dependency updates.
- SLSA provenance is recorded (`docs/slsa.md`).
- SBOM files (`*.cdx.json`) are generated per crate.
- Gap: only `cargo-deny` + Renovate scan; no OSV or Grype multi-scanner
  coverage.

---

## 7. Out of Scope

- Network-level security (no network surface in this SDK).
- Authentication/authorization of API callers (host responsibility).
- Secrets management (no secrets stored by the SDK).
- Container/sandbox isolation for plugin execution (host responsibility).
