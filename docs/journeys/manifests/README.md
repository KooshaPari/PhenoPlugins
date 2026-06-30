# Journey Manifests

Each JSON file in this directory is a structured journey manifest conforming to
`journey-manifest/v1`. Manifests link documented user journeys to test
evidence and the crates they exercise.

| Manifest | Journey | Evidence |
|---|---|---|
| `plugin-registration.json` | Register and look up a VCS plugin | `registry.rs` unit tests |
| `error-codes.json` | Typed error codes and recovery hints | `error.rs` unit tests |

See `crates/pheno-plugin-vessel/docs/journeys.md` for prose descriptions of
each journey.
