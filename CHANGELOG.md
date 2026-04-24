# Changelog

All notable changes to this project will be documented in this file.

## 📚 Documentation
- Docs(readme): expand README.md with purpose, stack, quick-start, related projects (`b63d597`)
- Docs: add PLAN.md (`a294e6d`)
- Docs: add README.md (`44c3d32`)
- Docs: add SPEC.md (`127224a`)
## 🔨 Other
- Chore(ci): adopt phenotype-tooling workflows (wave-2) (`f37a0a5`)
- Chore(governance): adopt standard CLAUDE.md + AGENTS.md + worklog (wave-2) (`6fa3a14`)
- Test(smoke): seed minimal smoke test — proves harness works (`6a2153e`)
- Merge remote-tracking branch 'origin/main' (`a1eccd4`)
- Chore(phenoplugins): annotate 3 dead_code suppressions with kept reasons (#1)

* docs: add SPEC.md

* ci: migrate to reusable workflows from template-commons

- Use reusable-rust-ci.yml, reusable-python-ci.yml, reusable-typescript-ci.yml
- Add security scanning with reusable-security-scan.yml
- Add governance validation with validate-governance.yml

* docs: add README.md

* docs: add PLAN.md

* ci(legacy-enforcement): add legacy tooling anti-pattern gate (WARN mode)

Adds legacy-tooling-gate.yml monitoring per CLAUDE.md Technology Adoption Philosophy.

Refs: phenotype/repos/tooling/legacy-enforcement/

* chore: add AgilePlus scaffolding

* chore(phenoplugins): annotate 3 dead_code suppressions with kept reasons

All 3 in crates/pheno-plugin-sqlite/src/lib.rs mark public methods on
the SQLite plugin (in_memory constructor, connection accessor, db_path
accessor). Added `// kept: ...` rationale above each.

cargo build: NOT VERIFIED — the pheno-plugin-core and pheno-plugin-sqlite
crates have no Cargo.toml checked in (pre-existing scaffold-only state,
not caused by these edits). Changes are doc-only and cannot regress
compilation.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>

---------

Co-authored-by: Forge <forge@phenotype.dev>
Co-authored-by: Claude Opus 4.7 (1M context) <noreply@anthropic.com> (`e9dce8a`)
- Add phenoVessel as pheno-plugin-vessel crate (`bf3a79b`)
- Chore: add AgilePlus scaffolding (`4d3df02`)
- Ci(legacy-enforcement): add legacy tooling anti-pattern gate (WARN mode)

Adds legacy-tooling-gate.yml monitoring per CLAUDE.md Technology Adoption Philosophy.

Refs: phenotype/repos/tooling/legacy-enforcement/ (`3333144`)
- Ci: migrate to reusable workflows from template-commons

- Use reusable-rust-ci.yml, reusable-python-ci.yml, reusable-typescript-ci.yml
- Add security scanning with reusable-security-scan.yml
- Add governance validation with validate-governance.yml (`93dc82d`)
- Initial: PhenoPlugins plugin system registry (`d18be90`)