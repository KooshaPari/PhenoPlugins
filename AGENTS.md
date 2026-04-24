# AGENTS.md — PhenoPlugins

## Project Overview

- **Name**: PhenoPlugins (Plugin System)
- **Description**: Plugin architecture and registry for extensible Phenotype components
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/PhenoPlugins`
- **Language Stack**: Rust, WASM, Cargo
- **Published**: Private (Phenotype org)

## Quick Start

```bash
# Navigate to project
cd /Users/kooshapari/CodeProjects/Phenotype/repos/PhenoPlugins

# Build
cargo build --workspace

# Run tests
cargo test --workspace
```

## Architecture

### Plugin System

```
┌─────────────────────────────────────────────────────────────────┐
│                     Plugin Host                                    │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                    Plugin Manager                             │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │ │
│  │  │ Load     │  │ Validate │  │ Sandbox  │  │ Events   │  │ │
│  │  │ Plugin   │  │ Manifest │  │ (WASM)   │  │ Bridge   │  │ │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Registry      │  │   API Surface     │  │   Hooks           │ │
│  │   (Local/Remote)│  │   (Exports/Imports)│  │   (Callbacks)     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Quality Standards

### Rust Quality

- **Formatter**: rustfmt
- **Linter**: clippy
- **Tests**: cargo test
- **WASM**: wasm-bindgen test

## Git Workflow

### Branch Naming

Format: `<type>/<component>/<description>`

Examples:
- `feat/host/add-hot-reload`
- `fix/sandbox/memory-leak`
- `feat/registry/add-version-check`

## CLI Commands

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
```

## Resources

- [WASMtime](https://wasmtime.dev/)
- [Rust WASM](https://rustwasm.github.io/)
- [Phenotype Registry](https://github.com/KooshaPari/phenotype-registry)

## Agent Notes

**Critical Details:**
- WASM sandboxing
- Manifest validation
- Version compatibility
- Event isolation
