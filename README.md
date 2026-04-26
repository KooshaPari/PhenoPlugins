# PhenoPlugins

Unified plugin system and extension architecture for the Phenotype ecosystem. Provides trait-based plugin interface, dynamic registry, lifecycle management, and battle-tested adapters for Git, SQLite, and container/storage abstractions.

## Overview

**PhenoPlugins** is the extensibility foundation for all Phenotype applications. It provides a modular, type-safe plugin system enabling applications like AgilePlus to seamlessly integrate custom adapters for VCS, storage, containers, and future extensions without tight coupling.

**Core Mission**: Enable plug-and-play extensibility across the Phenotype platform without coupling application logic to specific implementations.

## Technology Stack

- **Language**: Rust (Edition 2024)
- **Core Pattern**: Trait-based plugin abstraction with dynamic registry
- **Key Crates**:
  - `pheno-plugin-core` — Plugin trait, registry, lifecycle, error handling
  - `pheno-plugin-git` — Git VCS adapter plugin
  - `pheno-plugin-sqlite` — SQLite storage adapter plugin
  - `pheno-plugin-vessel` — Container/storage abstraction plugin
- **Async Runtime**: Tokio (for lifecycle hooks)
- **Error Handling**: thiserror with contextual errors

## Key Features

- **Trait-Based Design**: Plugin interface defined as Rust traits for type safety
- **Dynamic Registry**: Runtime plugin discovery and registration
- **Lifecycle Management**: Initialization, health checks, graceful shutdown hooks
- **Error Propagation**: Structured error types with full context
- **Adapter Pattern**: Clean separation between application and implementation
- **Testability**: Mock plugins for testing plugin hosts
- **Extensible**: Add new plugin types without modifying core

## Quick Start

```bash
# Clone and explore
git clone <repo-url>
cd PhenoPlugins

# Review governance and architecture
cat CLAUDE.md          # Project governance
cat SPEC.md            # Comprehensive specification
cat AGENTS.md          # Agent operating contract

# Build and test
cargo build --release
cargo test --workspace
cargo clippy --workspace -- -D warnings

# Review examples
ls examples/           # Plugin implementations
```

## Project Structure

```
PhenoPlugins/
├── crates/
│   ├── pheno-plugin-core/     # Core trait & registry
│   ├── pheno-plugin-git/      # Git VCS adapter
│   ├── pheno-plugin-sqlite/   # SQLite storage adapter
│   └── pheno-plugin-vessel/   # Container/storage abstraction
├── examples/                  # Example plugin implementations
├── docs/
│   └── SPEC.md               # Comprehensive specification
└── CLAUDE.md, AGENTS.md      # Governance & agent contract
```

## Architecture

```
Application Host (e.g., AgilePlus)
        ↓
┌─────────────────────────────────┐
│   pheno-plugin-core (Traits)    │
│  • Plugin trait                 │
│  • Registry                     │
│  • Lifecycle management         │
└─────────────────────────────────┘
        ↓  ↓  ↓
    [Git] [SQLite] [Vessel] [Future plugins...]
```

## Migration History

Consolidated from AgilePlus-specific crates:
- `agileplus-plugin-core` → `pheno-plugin-core`
- `agileplus-plugin-git` → `pheno-plugin-git`
- `agileplus-plugin-sqlite` → `pheno-plugin-sqlite`
- `phenoVessel` → `pheno-plugin-vessel`

## Related Phenotype Projects

- **[AgilePlus](../AgilePlus)** — Primary plugin host and consumer
- **[PhenoKit](../PhenoKit)** — Base kit with plugin-aware utilities
- **[AuthKit](../AuthKit)** — Auth-aware plugin implementations

## Plugin Development Guide

### Creating a Custom Plugin

Implement the `Plugin` trait:

```rust
use pheno_plugin_core::{Plugin, PluginMetadata, PluginError};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Custom plugin for Phenotype".to_string(),
        }
    }

    fn initialize(&mut self) -> Result<(), PluginError> {
        // Initialization logic
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup logic
        Ok(())
    }

    fn health_check(&self) -> Result<(), PluginError> {
        // Health probe
        Ok(())
    }
}
```

### Registering Plugins

```rust
use pheno_plugin_core::Registry;

let mut registry = Registry::new();
registry.register("my-plugin", Box::new(MyPlugin))?;
```

## Architecture Patterns

### Adapter Pattern

Each built-in plugin adapter follows the same pattern:

```
Interface (Trait)
      ↓
Adapter (Implements trait)
      ↓
Backend (Concrete implementation)
```

Example: Git adapter bridges `VcsPlugin` trait to git2/libgit2 backend.

### Lifecycle Hooks

All plugins support these lifecycle stages:

1. **Registration** — Plugin added to registry
2. **Initialization** — `initialize()` called; plugin allocates resources
3. **Active** — Plugin ready for use; `health_check()` called periodically
4. **Shutdown** — `shutdown()` called before removal; cleanup happens
5. **Deregistration** — Plugin removed from registry

### Error Handling

All plugin operations return `Result<T, PluginError>` with contextual information:

```rust
pub enum PluginError {
    NotFound(String),
    AlreadyExists(String),
    InitializationFailed(String),
    OperationFailed(String),
    HealthCheckFailed(String),
}
```

## Testing Plugins

Use `MockPlugin` for testing plugin hosts:

```rust
use pheno_plugin_core::mocks::MockPlugin;

#[test]
fn test_plugin_host() {
    let mock = MockPlugin::new("test-plugin");
    let mut registry = Registry::new();
    registry.register("test", Box::new(mock)).unwrap();
    // Test registry behavior
}
```

## Performance & Scalability

- **Registry lookup**: O(1) amortized
- **Plugin initialization**: Async via Tokio
- **Lifecycle overhead**: <1ms per operation
- **Concurrent plugins**: 100+ supported

## License

MIT — see [LICENSE](./LICENSE).

## Governance

Governance in `CLAUDE.md`. Functional requirements and FR-to-test mapping in `FUNCTIONAL_REQUIREMENTS.md`. Architecture decisions in `docs/SPEC.md` and `docs/adr/`.