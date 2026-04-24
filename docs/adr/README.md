# Architecture Decision Records

## Overview

This directory contains Architecture Decision Records (ADRs) for PhenoPlugins. Each ADR documents a significant architectural decision, its context, consequences, and alternatives considered.

ADRs follow the [Nygard format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) with Phenotype ecosystem conventions.

---

## Active Decisions

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| 001 | Trait-Based Plugin Architecture | Accepted | 2026-04-04 |
| 002 | Async-First Plugin Interface | Accepted | 2026-04-04 |
| 003 | In-Process Plugin Loading | Accepted | 2026-04-04 |

---

## ADR Index

### ADR-001: Trait-Based Plugin Architecture

**Status:** Accepted

**Context:** PhenoPlugins needed an architectural foundation for extensibility across the Phenotype ecosystem. Multiple implementation patterns were considered including:
- C ABI with dynamic loading
- WebAssembly (WASM) modules
- gRPC-based separate processes
- Language-native traits/interfaces

**Decision:** Adopt Rust trait-based architecture as the primary plugin interface pattern.

**Consequences:**
- (+) Maximum performance with zero-cost abstractions
- (+) Compile-time type safety
- (+) Clear, IDE-friendly interfaces
- (+) No serialization overhead
- (-) Same-language limitation (Rust-only plugins)
- (-) No runtime sandboxing (relies on Rust safety)
- (-) Requires recompilation for interface changes

**Alternatives Considered:**
1. **WASM**: Rejected due to performance overhead for internal ecosystem
2. **gRPC**: Rejected due to serialization and latency overhead
3. **C ABI**: Rejected due to safety concerns and complexity

### ADR-002: Async-First Plugin Interface

**Status:** Accepted

**Context:** Plugin operations (git, SQLite) are primarily I/O bound. Synchronous interfaces would block the host. Options considered:
- Synchronous with thread pools
- Async with tokio
- Async with async-std
- Callback-based

**Decision:** All plugin traits use async methods with `async_trait` macro, assuming tokio runtime.

**Consequences:**
- (+) Non-blocking I/O operations
- (+) Composable with host async code
- (+) Backpressure handling through tokio
- (-) `async_trait` introduces boxing overhead
- (-) Additional complexity for simple operations
- (-) Debug complexity with nested async

**Alternatives Considered:**
1. **Sync with thread pool**: Rejected - less efficient for many concurrent operations
2. **Callback-based**: Rejected - callback hell, error handling complexity
3. **Native async traits**: Considered - waiting for stabilization in Rust

### ADR-003: In-Process Plugin Loading

**Status:** Accepted

**Context:** Plugin isolation level needed decision. Options:
- Separate process (maximum isolation)
- WASM sandbox (memory-safe isolation)
- In-process with language safety (Rust)
- Container isolation

**Decision:** Load plugins in-process, relying on Rust's memory safety and type system for isolation.

**Consequences:**
- (+) Minimal overhead
- (+) Simple debugging
- (+) Shared memory possible
- (+) Fast communication (direct calls)
- (-) Plugin crash crashes host
- (-) No resource limits enforced
- (-) Plugins share host memory space

**Mitigations:**
- All plugins are internal to Phenotype ecosystem
- Code review required for all plugins
- Test coverage requirements
- Health check mechanisms

**Alternatives Considered:**
1. **Separate process**: Rejected - too high overhead for internal plugins
2. **WASM**: Rejected - performance overhead, not needed for trusted plugins
3. **Containers**: Rejected - massive overhead, not applicable

---

## ADR Template

When proposing new ADRs, use this template:

```markdown
# ADR-XXX: Title

## Status

- Proposed / Accepted / Deprecated / Superseded by ADR-YYY

## Context

What is the issue that we're seeing that is motivating this decision or change?

## Decision

What is the change that we're proposing or have agreed to implement?

## Consequences

What becomes easier or more difficult to do because of this change?

### Positive
- Point 1
- Point 2

### Negative
- Point 1
- Point 2

### Neutral
- Point 1

## Alternatives Considered

### Alternative 1: Name
- Description
- Pros
- Cons
- Why rejected

### Alternative 2: Name
- Description
- Pros
- Cons
- Why rejected

## References

- Links to related specs, issues, PRs
- External resources
```

---

## Decision Log

| Date | ADR | Event |
|------|-----|-------|
| 2026-04-04 | 001 | Initial acceptance |
| 2026-04-04 | 002 | Initial acceptance |
| 2026-04-04 | 003 | Initial acceptance |

---

*See individual ADR files for full details:*
- [ADR-001](./ADR-001-trait-based-architecture.md)
- [ADR-002](./ADR-002-async-first-interface.md)
- [ADR-003](./ADR-003-in-process-loading.md)
