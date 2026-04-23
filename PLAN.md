# PLAN.md - PhenoPlugins

Plugin system registry for the Phenotype ecosystem.

## Phases

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| 1. Plugin Core | 2 weeks | Plugin trait, Registry, Lifecycle management |
| 2. Git Adapter | 1 week | pheno-plugin-git implementing Plugin trait |
| 3. SQLite Adapter | 1 week | pheno-plugin-sqlite implementing Plugin trait |
| 4. Testing | 1 week | Unit tests for each crate |
| 5. Documentation | 1 week | API docs, usage examples |

## Key Deliverables

- `pheno-plugin-core` - Plugin trait, registry, error handling
- `pheno-plugin-git` - Git VCS adapter plugin
- `pheno-plugin-sqlite` - SQLite storage adapter plugin

## Resource Estimate

- **Dev time**: 6 person-weeks
- **Dependencies**: none (core crate)
- **Testing**: Unit tests per crate, integration tests with hosts

---

Generated: 2026-04-03
