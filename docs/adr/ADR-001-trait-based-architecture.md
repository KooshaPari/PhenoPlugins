# ADR-001: Trait-Based Plugin Architecture

## Status

Accepted

## Context

PhenoPlugins needed an architectural foundation for extensibility across the Phenotype ecosystem. The ecosystem consists of multiple tools (AgilePlus, thegent, heliosCLI) all written in Rust, requiring a consistent plugin mechanism.

The primary requirements were:
1. **Performance**: Minimal overhead for plugin operations
2. **Safety**: Memory safety and type safety guarantees
3. **Ergonomics**: Developer-friendly interfaces with good IDE support
4. **Integration**: Seamless integration with existing Rust code

Multiple implementation patterns were evaluated:

### Option 1: C ABI with Dynamic Loading
Traditional POSIX dynamic loading with C interface.

```c
// plugin.h
#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    const char* name;
    int (*initialize)(void* config);
    int (*execute)(const char* input, char** output);
} plugin_exports_t;

plugin_exports_t* plugin_init(void);

#ifdef __cplusplus
}
#endif
```

**Pros:**
- Language-agnostic (any language can implement)
- Well-understood pattern
- No toolchain lock-in

**Cons:**
- Memory unsafe (raw pointers)
- Manual memory management required
- No type safety across boundary
- Complex error handling (C-style error codes)
- Symbol versioning challenges

### Option 2: WebAssembly (WASM) Modules
Plugins compiled to WASM and executed in sandboxed runtime.

```rust
// Using wasmtime
use wasmtime::{Engine, Module, Instance, Store};

let engine = Engine::default();
let module = Module::from_file(&engine, "plugin.wasm")?;
let mut store = Store::new(&engine, ());
let instance = Instance::new(&mut store, &module, &[])?;
```

**Pros:**
- True sandboxing (memory-safe)
- Language-agnostic
- Near-native performance possible
- No host process risk from plugin crashes

**Cons:**
- 2-10MB memory overhead per plugin
- WASI still maturing
- Limited system access
- Debugging complexity
- Additional runtime dependency (wasmtime/wasmer)

### Option 3: gRPC-Based Separate Processes
Plugins run as separate processes, communicate via gRPC.

```protobuf
service VcsPlugin {
    rpc CreateWorktree(CreateWorktreeRequest) returns (CreateWorktreeResponse);
    rpc ListWorktrees(ListWorktreesRequest) returns (ListWorktreesResponse);
}
```

**Pros:**
- Process isolation (crash protection)
- Language-agnostic
- Can use different runtime versions
- OS-level resource limits

**Cons:**
- Serialization overhead (JSON/Protobuf)
- High latency (context switches, network stack)
- Complex debugging (multiple processes)
- Coordination challenges
- Significant memory overhead per plugin

### Option 4: Language-Native Traits (Rust)
Rust trait-based interfaces with dynamic dispatch where needed.

```rust
#[async_trait::async_trait]
pub trait VcsPlugin: AdapterPlugin {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
    async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>>;
}
```

**Pros:**
- Zero-cost abstractions
- Compile-time type safety
- Excellent IDE support (autocomplete, goto definition)
- No serialization overhead
- Memory safety by default
- Async/await support

**Cons:**
- Same-language limitation (Rust-only)
- No runtime sandboxing (relies on Rust safety)
- Requires recompilation for interface changes
- Dynamic loading limited (no stable Rust ABI)

## Decision

**Adopt Rust trait-based architecture as the primary plugin interface pattern.**

Specifically:
1. Define plugin interfaces as Rust traits in `pheno-plugin-core`
2. Use `async_trait` for async support
3. Use `Arc<dyn Trait>` for runtime plugin selection
4. Implement a registry pattern for plugin management
5. Accept the same-language limitation (all Phenotype tools use Rust)

## Consequences

### Positive

1. **Maximum Performance**: Direct method calls with no serialization or context switching
2. **Type Safety**: Compile-time verification of plugin interface compliance
3. **Ergonomics**: First-class IDE support with autocomplete, type hints, documentation
4. **Zero Overhead**: No runtime penalty for plugin abstraction
5. **Safety**: Rust's ownership model prevents memory errors, data races
6. **Composability**: Traits naturally compose (VcsPlugin + StoragePlugin)

### Negative

1. **Language Lock-in**: Only Rust can implement plugins (acceptable for Phenotype ecosystem)
2. **No Runtime Sandboxing**: Plugin crash crashes host (mitigated by Rust safety)
3. **Binary Coupling**: Plugins must be compiled with compatible dependency versions
4. **Dynamic Loading Limited**: Requires careful design around `abi_stable` if needed later

### Neutral

1. **Binary Size**: Plugins compiled as separate crates (reasonable for architecture)
2. **Build Complexity**: Workspace structure required (already in place)
3. **Testing**: Can test plugins in isolation or integrated

## Implementation

### Trait Design Principles

1. **Async-by-default**: All I/O operations are async
2. **Error propagation**: Use `PluginError` enum across all traits
3. **Object safety**: Traits must be object-safe for dynamic dispatch
4. **Send + Sync**: All plugin types must be thread-safe

### Registry Pattern

```rust
pub struct PluginRegistry {
    vcs: RwLock<HashMap<String, Arc<dyn VcsPlugin>>>,
    storage: RwLock<HashMap<String, Arc<dyn StoragePlugin>>>,
    initialized: RwLock<bool>,
}
```

### Example Plugin Implementation

```rust
pub struct GitAdapter {
    repo_path: PathBuf,
}

impl AdapterPlugin for GitAdapter {
    fn name(&self) -> &str { "git" }
    fn version(&self) -> &str { env!("CARGO_PKG_VERSION") }
    fn initialize(&self, _config: PluginConfig) -> PluginResult<()> { Ok(()) }
}

#[async_trait]
impl VcsPlugin for GitAdapter {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
        // Implementation using git2
    }
    // ... other methods
}
```

## Alternatives Retained for Future

**WASM**: Not completely rejected. Consider for:
- Third-party plugin distribution (untrusted code)
- User-contributed extensions
- Multi-language support requirement

The trait-based architecture can be wrapped with WASM bindings if needed later.

## References

- [SOTA.md - Plugin System Patterns](../SOTA.md#modern-plugin-architecture-patterns)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [ Designing Data-Intensive Applications](https://dataintensive.net/) - Chapter on extensibility

## Notes

This decision aligns with the Phenotype ecosystem's "aggressive adoption of native/compiler rewrites" principle. We choose the most idiomatic Rust solution rather than compromising for hypothetical future requirements.

*Decision Date: 2026-04-04*
*Decision Author: PhenoPlugins Team*
*Stakeholders: AgilePlus, thegent, heliosCLI maintainers*
