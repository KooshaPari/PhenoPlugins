# ADR-003: In-Process Plugin Loading

## Status

Accepted

## Context

A critical architectural decision for PhenoPlugins was the isolation level between host and plugins. The options range from no isolation (shared process) to maximum isolation (separate containers).

### Threat Model

PhenoPlugins operates in a specific context:
- **All plugins are internal**: Part of Phenotype organization
- **All code is reviewed**: Standard Phenotype code review process
- **Same organization**: No third-party plugins
- **Same language**: All plugins written in Rust
- **Observability**: Full logging, metrics, tracing available

Given this context, the security requirements differ from systems accepting untrusted plugins.

### Option Analysis

### Option 1: Separate Process (Maximum Isolation)

Plugins run as separate OS processes, communicate via IPC.

```
┌─────────────────────────────────────────┐
│           Host Process                  │
│  ┌───────────────────────────────────┐ │
│  │         Plugin Manager              │ │
│  └───────────────────────────────────┘ │
│                    │                     │
│         IPC (sockets/pipes)              │
│                    │                     │
└────────────────────┼─────────────────────┘
                     │
┌────────────────────┼─────────────────────┐
│      Plugin Process│                    │
│  ┌─────────────────▼─────────────────┐   │
│  │           Git Adapter             │   │
│  │  • Own memory space               │   │
│  │  • Own process ID                 │   │
│  │  • OS resource limits             │   │
│  └───────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

**Pros:**
- Crash isolation (plugin crash doesn't kill host)
- Memory protection
- OS resource limits apply
- Can kill runaway plugins
- Different runtime versions possible

**Cons:**
- High latency (IPC serialization)
- High memory overhead (per-process)
- Complex debugging (multi-process)
- Complex deployment (multiple binaries)
- Serialization required for all communication

### Option 2: WebAssembly Sandbox

Plugins compiled to WASM, executed in runtime like wasmtime.

```
┌─────────────────────────────────────────┐
│           Host Process                  │
│  ┌───────────────────────────────────┐ │
│  │       WASM Runtime (wasmtime)       │ │
│  │  ┌───────────────────────────────┐ │ │
│  │  │      Linear Memory (2GB)      │ │ │
│  │  │  ┌─────────────────────────┐  │ │ │
│  │  │  │    Plugin (WASM)         │  │ │ │
│  │  │  │  • Sandboxed             │  │ │ │
│  │  │  │  • Bounds checked        │  │ │ │
│  │  │  │  • Capability limited    │  │ │ │
│  │  │  └─────────────────────────┘  │ │ │
│  │  └───────────────────────────────┘ │ │
│  └───────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

**Pros:**
- Memory safety (bounds checking)
- No direct system calls
- Capability-based security
- Language-agnostic
- Fine-grained resource limits

**Cons:**
- 2-10MB memory overhead per plugin
- WASI limitations
- Startup latency
- Debugging complexity
- Additional runtime dependency
- FFI complexity for host functions

### Option 3: In-Process with Language Safety

Plugins loaded in same process, relying on Rust's safety guarantees.

```
┌─────────────────────────────────────────┐
│           Host Process                  │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │         Plugin Registry             │ │
│  │  ┌─────────────┐ ┌───────────────┐│ │
│  │  │ GitAdapter  │ │ SQLiteAdapter ││ │
│  │  │ (dyn Trait) │ │  (dyn Trait)  ││ │
│  │  └─────────────┘ └───────────────┘│ │
│  └───────────────────────────────────┘ │
│                                         │
│  Shared memory space, same runtime      │
└─────────────────────────────────────────┘
```

**Pros:**
- Zero overhead (direct calls)
- Simple debugging (single process)
- Fastest possible communication
- Simple deployment (single binary)
- Full IDE support

**Cons:**
- No crash isolation (plugin panic = host panic)
- No memory limits enforced
- Shared fate (plugin memory leak affects host)
- Same runtime version required
- Trust-based security model

### Option 4: Container Isolation

Each plugin in separate Docker/container.

**Pros:**
- Maximum isolation
- Resource limits enforced
- Image-based distribution

**Cons:**
- Massive overhead (hundreds of MB per plugin)
- Slow startup (seconds)
- Complex orchestration
- Complete overkill for this use case

## Decision

**Adopt in-process plugin loading with language-level safety (Rust).**

Rationale:
1. **Trust model**: All plugins are internal, reviewed, trusted
2. **Performance**: Direct calls required for git/SQLite efficiency
3. **Ergonomics**: Simple debugging and deployment
4. **Rust safety**: Memory safety, type safety, no data races
5. **Phenotype context**: Ecosystem values performance over sandboxing

### Security Mitigations

While accepting in-process loading, we implement these mitigations:

1. **Code Review**: All plugin code undergoes standard review
2. **Test Coverage**: Comprehensive test requirements
3. **Health Checks**: Plugin health monitoring via `health_check()` method
4. **Input Validation**: All plugin inputs validated at boundaries
5. **Audit Logging**: All plugin operations logged
6. **Feature Flags**: Gradual rollout capability

### Future Escape Hatch

WASM remains an option for future scenarios:
```rust
// Future: WASM wrapper for untrusted plugins
pub struct WasmPlugin {
    engine: wasmtime::Engine,
    instance: wasmtime::Instance,
}

impl VcsPlugin for WasmPlugin {
    async fn create_worktree(&self, slug: &str, wp: &str) -> PluginResult<PathBuf> {
        // Call WASM exported function
    }
}
```

## Consequences

### Positive

1. **Maximum Performance**: No serialization, no context switching
2. **Simple Debugging**: Standard debugger, stack traces
3. **Simple Deployment**: Single binary, no coordination
4. **Low Memory**: No per-plugin overhead
5. **IDE Support**: Full autocomplete, goto definition
6. **Fast Startup**: No process creation overhead

### Negative

1. **No Crash Isolation**: Plugin panic crashes host
2. **Shared Memory**: Plugin memory leak affects host
3. **Same Runtime**: All plugins must use compatible Rust versions
4. **Trust Required**: Cannot load untrusted plugins

### Mitigations

| Risk | Mitigation |
|------|------------|
| Plugin crash | Rust panic handling, `catch_unwind` at boundaries |
| Memory leak | Regular health checks, resource monitoring |
| Infinite loop | Operation timeouts via tokio |
| Data corruption | Input validation, immutable boundaries |

## Implementation

### Registry Design

```rust
pub struct PluginRegistry {
    vcs: RwLock<HashMap<String, Arc<dyn VcsPlugin>>>,
    storage: RwLock<HashMap<String, Arc<dyn StoragePlugin>>>,
}

impl PluginRegistry {
    pub fn register_vcs(&self, plugin: Box<dyn VcsPlugin>) -> PluginResult<()> {
        // Direct in-process registration
        let name = plugin.name().to_string();
        let mut vcs = self.vcs.write().map_err(|_| {
            PluginError::Initialization("Lock poisoned".to_string())
        })?;
        vcs.insert(name, Arc::from(plugin));
        Ok(())
    }
}
```

### Panic Handling

```rust
use std::panic::catch_unwind;

pub fn call_plugin_safe<F, R>(f: F) -> PluginResult<R>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    match catch_unwind(f) {
        Ok(result) => Ok(result),
        Err(_) => Err(PluginError::Execution("Plugin panicked".to_string())),
    }
}
```

### Health Monitoring

```rust
#[async_trait]
pub trait AdapterPlugin: Send + Sync {
    fn health_check(&self) -> PluginResult<()> {
        Ok(())
    }
}

// Registry periodically checks all plugins
pub async fn health_check_all(&self) -> Vec<(String, PluginResult<()>)> {
    let mut results = Vec::new();
    for name in self.vcs_adapters() {
        if let Some(vcs) = self.vcs(&name) {
            results.push((name.clone(), vcs.health_check()));
        }
    }
    results
}
```

## Monitoring Requirements

In-process loading requires observability:

1. **Metrics**: Plugin operation counts, durations, errors
2. **Tracing**: Distributed tracing across plugin calls
3. **Resource**: Memory usage, CPU time per plugin
4. **Health**: Periodic health checks with alerting

## References

- [Rust Safety Guarantees](https://doc.rust-lang.org/nomicon/)
- [SOTA.md - Security Models](../SOTA.md#security-models)
- [Rust Panic Handling](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)

## Notes

This decision reflects the Phenotype ecosystem's values:
- Performance over sandboxing for trusted code
- Aggressive adoption of Rust's safety features
- Pragmatic engineering over theoretical purity

If future requirements include untrusted plugins, WASM wrapping can be added without breaking the trait-based interface.

*Decision Date: 2026-04-04*
*Decision Author: PhenoPlugins Team*
*Stakeholders: Phenotype Security Review, All Tool Teams*
