# ADR-002: Async-First Plugin Interface

## Status

Accepted

## Context

PhenoPlugins defines interfaces for VCS (git) and storage (SQLite) operations. Both are primarily I/O bound. The design question was whether plugin interfaces should be:

1. **Synchronous (blocking)**: Simple, direct calls that block the thread
2. **Asynchronous (non-blocking)**: Using async/await for concurrent operations
3. **Callback-based**: Traditional async with callbacks
4. **Hybrid**: Sync by default, async variants available

### Requirements Analysis

**Git Operations:**
- Clone: 10ms - 60s (network dependent)
- Commit: 10-100ms (disk I/O)
- Worktree creation: 5-50ms (filesystem)

**SQLite Operations:**
- Query: 0.1-10ms (cache dependent)
- Write: 1-100ms (disk/fsync dependent)
- Migration: 10ms-1s (schema changes)

**Host Requirements:**
- AgilePlus may manage multiple features concurrently
- heliosCLI needs responsive UI during operations
- thegent may batch operations

### Option Analysis

### Option 1: Synchronous with Thread Pool

```rust
pub trait VcsPlugin: AdapterPlugin {
    fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
}

// Host uses thread pool
pool.spawn(move || {
    plugin.create_worktree(&slug, &wp_id)
});
```

**Pros:**
- Simple implementation
- No async complexity
- Easy error handling

**Cons:**
- Thread pool management complexity
- Context switching overhead
- Hard to compose operations
- No backpressure handling
- Thread limits constrain concurrency

### Option 2: Async with tokio

```rust
#[async_trait::async_trait]
pub trait VcsPlugin: AdapterPlugin {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
}

// Natural composition
let path = plugin.create_worktree(&slug, &wp_id).await?;
let result = plugin.checkout_branch(&branch).await?;
```

**Pros:**
- Non-blocking I/O
- Natural composition (async/await syntax)
- Efficient resource usage (fewer threads)
- Backpressure through tokio
- Consistent with Phenotype ecosystem (tokio standard)

**Cons:**
- `async_trait` adds boxing overhead
- Debug complexity (stack traces)
- Additional cognitive load for simple cases
- Infection (async spreads through codebase)

### Option 3: Callback-Based

```rust
pub trait VcsPlugin: AdapterPlugin {
    fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
        callback: Box<dyn FnOnce(PluginResult<PathBuf>) + Send>
    );
}
```

**Pros:**
- No runtime required
- Lower overhead than async
- Can use in sync contexts

**Cons:**
- Callback hell
- Error handling complexity
- Hard to compose
- No cancellation support
- Ownership/lifetime challenges

### Option 4: Native Async Traits (Unstable)

```rust
#![feature(async_fn_in_trait)]

pub trait VcsPlugin: AdapterPlugin {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
}
```

**Pros:**
- No `async_trait` overhead
- Native Rust feature
- Future-proof

**Cons:**
- Requires nightly Rust
- Feature not stabilized
- Limited ecosystem support

## Decision

**Adopt async-first plugin interfaces using `async_trait`, targeting stable Rust with tokio runtime.**

Specifically:
1. All plugin traits use `#[async_trait]` macro
2. Methods return `PluginResult<T>` (not Result with futures)
3. Hosts are expected to run tokio runtime
4. Accept `async_trait` boxing overhead as reasonable trade-off
5. Evaluate native async traits when stabilized

## Consequences

### Positive

1. **Non-Blocking I/O**: Host remains responsive during slow operations
2. **Composability**: Natural async/await composition
3. **Resource Efficiency**: Fewer threads needed vs thread pool
4. **Ecosystem Alignment**: Consistent with rest of Phenotype (tokio)
5. **Cancellation**: Operations can be cancelled via tokio
6. **Timeout Support**: Easy timeout wrapping

### Negative

1. **Boxing Overhead**: `async_trait` boxes futures (allocation per call)
2. **Debug Complexity**: Async stack traces are harder to read
3. **Infection**: Async spreads through codebase
4. **Testing**: Requires tokio test runtime
5. **Learning Curve**: Team must understand async Rust

### Mitigations

1. **Overhead**: Acceptable for I/O bound operations (network/disk dominates)
2. **Debug**: Use `tracing` for structured logging across await points
3. **Testing**: Standardize on `#[tokio::test]` attribute

## Implementation Details

### Trait Definition

```rust
use async_trait::async_trait;

#[async_trait]
pub trait VcsPlugin: AdapterPlugin + Send + Sync {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
    async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>>;
    async fn cleanup_worktree(&self, worktree_path: &Path) -> PluginResult<()>;
    // ...
}
```

### Implementation Pattern

```rust
pub struct GitAdapter {
    repo_path: PathBuf,
}

#[async_trait]
impl VcsPlugin for GitAdapter {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
        // git2 operations are blocking, so we use spawn_blocking
        let repo_path = self.repo_path.clone();
        let slug = feature_slug.to_string();
        let wp = wp_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            // Blocking git2 operations here
            Self::create_worktree_blocking(&repo_path, &slug, &wp)
        }).await.map_err(|e| PluginError::Execution(e.to_string()))?
    }
}
```

### Host Usage

```rust
use pheno_plugin_core::PluginRegistry;

async fn create_feature_worktree(registry: &PluginRegistry, slug: &str) -> PluginResult<()> {
    let vcs = registry.vcs("git").ok_or_else(|| {
        PluginError::NotFound("git".to_string())
    })?;
    
    let path = vcs.create_worktree(slug, "WP001").await?;
    println!("Created worktree at: {:?}", path);
    
    Ok(())
}
```

### Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_worktree() -> PluginResult<()> {
        let adapter = create_test_adapter().await?;
        let path = adapter.create_worktree("test-feature", "WP001").await?;
        assert!(path.exists());
        Ok(())
    }
}
```

## Migration Path to Native Async Traits

When Rust stabilizes `async_fn_in_trait`, migration is straightforward:

1. Remove `#[async_trait]` macro
2. Change trait definition to native async
3. Implementations naturally adapt
4. No host code changes required (await syntax identical)

```rust
// Future state (when stabilized)
pub trait VcsPlugin: AdapterPlugin {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
}
```

## Performance Analysis

### async_trait Overhead

The `#[async_trait]` macro transforms:

```rust
// Source
async fn method(&self) -> Result<T>;

// Expanded
fn method<'life0, 'async_trait>(
    &'life0 self
) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'async_trait>>
where
    'life0: 'async_trait,
    Self: 'async_trait;
```

**Cost per call:**
- Box allocation: ~50-100ns
- Virtual dispatch: ~1-3ns
- Total overhead: ~50-100ns per async call

**Context:** Git operations take milliseconds to seconds. 100ns overhead is negligible.

### Comparison

| Approach | Latency | Throughput | Resource Usage | Complexity |
|----------|---------|------------|----------------|--------------|
| Sync + ThreadPool | Low | Medium | High (threads) | Medium |
| async_trait | Low | High | Low (tasks) | Medium |
| Native async | Lowest | High | Low (tasks) | Low |
| Callbacks | Low | Medium | Low | High |

## References

- [async_trait crate](https://docs.rs/async-trait/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [Tokio documentation](https://tokio.rs/)
- [Rust Async Working Group](https://rust-lang.github.io/wg-async/)

## Notes

This decision prioritizes ecosystem consistency and developer ergonomics over theoretical performance optimization. The Phenotype ecosystem has already standardized on tokio; adding a different concurrency model would create friction.

*Decision Date: 2026-04-04*
*Decision Author: PhenoPlugins Team*
*Stakeholders: All Phenotype tool maintainers*
