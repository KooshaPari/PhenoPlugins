# PhenoPlugins Specification

## Table of Contents

1. [Overview](#1-overview)
2. [Goals and Non-Goals](#2-goals-and-non-goals)
3. [Architecture](#3-architecture)
4. [Core Components](#4-core-components)
5. [Plugin Interfaces](#5-plugin-interfaces)
6. [Registry System](#6-registry-system)
7. [Error Handling](#7-error-handling)
8. [Lifecycle Management](#8-lifecycle-management)
9. [Implementation Details](#9-implementation-details)
10. [Testing Strategy](#10-testing-strategy)
11. [Performance Characteristics](#11-performance-characteristics)
12. [Security Model](#12-security-model)
13. [Versioning and Evolution](#13-versioning-and-evolution)
14. [Integration Guide](#14-integration-guide)
15. [Deployment](#15-deployment)
16. [Monitoring and Observability](#16-monitoring-and-observability)
17. [Future Directions](#17-future-directions)
18. [References](#18-references)

---

## 1. Overview

PhenoPlugins is the foundational plugin architecture for the Phenotype ecosystem, providing extensible, modular, and secure plugin systems for all Phenotype applications. It enables the ecosystem to adapt to varying requirements while maintaining consistent interfaces and high performance.

### 1.1 Purpose

PhenoPlugins addresses the need for:
- **VCS Abstraction**: Different projects may use different version control strategies
- **Storage Flexibility**: Multiple storage backends (SQLite, PostgreSQL, etc.)
- **Feature Extensibility**: New capabilities added without core changes
- **Testing Isolation**: Mock implementations for testing

### 1.2 Scope

**In Scope:**
- Core plugin trait definitions
- Plugin registry and lifecycle management
- Reference implementations (Git, SQLite)
- Error handling patterns
- Integration patterns for host applications

**Out of Scope:**
- Plugin marketplace or distribution system
- UI components for plugin management
- Remote plugin loading over network
- Plugin sandboxing beyond Rust's guarantees
- Non-Rust plugin interfaces (FFI out of scope for V1)

### 1.3 Target Audience

- Phenotype tool developers (AgilePlus, thegent, heliosCLI)
- Plugin authors within the Phenotype ecosystem
- System integrators extending Phenotype tools

### 1.4 Document Conventions

- **MUST**: Absolute requirement
- **SHOULD**: Strong recommendation
- **MAY**: Optional
- **SHALL**: Mandatory (typically for specifications)

Code examples use Rust unless otherwise specified.

---

## 2. Goals and Non-Goals

### 2.1 Goals

| Priority | Goal | Rationale |
|----------|------|-----------|
| P0 | Zero-cost abstractions | Plugins should not impose runtime overhead |
| P0 | Type safety | Compile-time verification of plugin interfaces |
| P0 | Async-first | Non-blocking I/O for responsive applications |
| P1 | Simple integration | Hosts can integrate with minimal boilerplate |
| P1 | Testability | Easy mocking for unit tests |
| P1 | Observability | Health checks and metrics built-in |
| P2 | Extensibility | New plugin types can be added |
| P2 | Documentation | Clear patterns and examples |

### 2.2 Non-Goals

| Item | Rationale |
|------|-----------|
| Cross-language plugins | Phenotype ecosystem is Rust-native |
| Dynamic loading | Trait objects provide sufficient dynamism |
| Plugin sandboxing | Trusted environment, Rust safety sufficient |
| Plugin marketplace | Out of scope for core architecture |
| UI framework | Hosts provide their own UI |
| Self-updating plugins | Deployment managed externally |

### 2.3 Success Criteria

The PhenoPlugins implementation is successful when:

1. **Performance**: Plugin call overhead < 100ns (excluding operation cost)
2. **Ergonomics**: New plugin integration requires < 50 lines of host code
3. **Reliability**: Plugin crashes do not corrupt host state (Rust safety)
4. **Testability**: All plugins mockable for testing
5. **Documentation**: API documentation coverage > 90%

---

## 3. Architecture

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Host Application                               │
│  (AgilePlus / thegent / heliosCLI / etc.)                               │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Host Business Logic                          │   │
│  │                                                                  │   │
│  │  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────┐ │   │
│  │  │ Feature Mgmt  │  │  Work Package │  │  Audit System       │ │   │
│  │  │               │  │  Management   │  │                     │ │   │
│  │  └───────┬───────┘  └───────┬───────┘  └──────────┬──────────┘ │   │
│  │          │                  │                     │            │   │
│  └──────────┼──────────────────┼─────────────────────┼────────────┘   │
│             │                  │                     │                │
│             └──────────────────┼─────────────────────┘                │
│                                ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    pheno-plugin-core                             │   │
│  │  ┌─────────────────────────────────────────────────────────────┐  │   │
│  │  │                     PluginRegistry                          │  │   │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │  │   │
│  │  │  │ VCS Plugins │  │StoragePlugins│  │  Future: LLM, etc.  │   │  │   │
│  │  │  │  • git      │  │  • sqlite    │  │                     │   │  │   │
│  │  │  │  • mock     │  │  • postgres  │  │                     │   │  │   │
│  │  │  └─────────────┘  └─────────────┘  └─────────────────────┘   │  │   │
│  │  └─────────────────────────────────────────────────────────────┘  │   │
│  │                                                                  │   │
│  │  ┌─────────────────────────────────────────────────────────────┐  │   │
│  │  │                    Trait Definitions                        │  │   │
│  │  │  • AdapterPlugin (base)                                      │  │   │
│  │  │  • VcsPlugin (VCS operations)                                │  │   │
│  │  │  • StoragePlugin (persistence)                               │  │   │
│  │  └─────────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Design Principles

#### 3.2.1 Hexagonal Architecture (Ports and Adapters)

PhenoPlugins follows the Hexagonal Architecture pattern:

- **Ports**: Trait definitions (interfaces) in `pheno-plugin-core`
- **Adapters**: Concrete implementations in plugin crates
- **Domain Logic**: Host application logic independent of adapters

```
┌─────────────────────────────────────────────┐
│              Host Application               │
│         (Domain Logic - No I/O)            │
│                                             │
│  ┌─────────────────────────────────────┐   │
│  │         PluginRegistry               │   │
│  │    (Dependency Injection Hub)        │   │
│  └─────────────────────────────────────┘   │
│                    │                        │
│       ┌────────────┼────────────┐          │
│       ▼            ▼            ▼          │
│  ┌────────┐   ┌────────┐   ┌────────┐      │
│  │  Port  │   │  Port  │   │  Port  │      │
│  │(Traits)│   │(Traits)│   │(Traits)│      │
│  └───┬────┘   └───┬────┘   └───┬────┘      │
│      │            │            │            │
└──────┼────────────┼────────────┼────────────┘
       │            │            │
       ▼            ▼            ▼
  ┌─────────┐  ┌─────────┐  ┌─────────┐
  │ Adapter │  │ Adapter │  │ Adapter │
  │  (git)  │  │(sqlite) │  │  (mock) │
  └─────────┘  └─────────┘  └─────────┘
```

#### 3.2.2 Interface Segregation

Each trait is focused on a single responsibility:

- `AdapterPlugin`: Base lifecycle (all plugins)
- `VcsPlugin`: Version control operations
- `StoragePlugin`: Persistence operations

No plugin is forced to implement methods it doesn't need.

#### 3.2.3 Dependency Inversion

Core depends on abstractions, not implementations:

```rust
// Core defines the interface (trait)
pub trait VcsPlugin: AdapterPlugin {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
}

// Host depends on interface
pub struct FeatureManager {
    vcs: Arc<dyn VcsPlugin>,  // Not Arc<GitAdapter>
}
```

### 3.3 Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            Cargo Workspace                               │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    pheno-plugin-core (lib)                       │   │
│  │                                                                  │   │
│  │  pub mod error;    // PluginError, PluginResult                │   │
│  │  pub mod registry; // PluginRegistry                            │   │
│  │  pub mod traits;   // AdapterPlugin, VcsPlugin, StoragePlugin  │   │
│  │                                                                  │   │
│  │  Dependencies:                                                    │   │
│  │  • serde (serialization)                                        │   │
│  │  • thiserror (error handling)                                    │   │
│  │  • async-trait (async traits)                                    │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    pheno-plugin-git (lib)                        │   │
│  │                                                                  │   │
│  │  pub struct GitAdapter;                                          │   │
│  │                                                                  │   │
│  │  impl AdapterPlugin for GitAdapter                               │   │
│  │  impl VcsPlugin for GitAdapter                                   │   │
│  │                                                                  │   │
│  │  Dependencies:                                                    │   │
│  │  • pheno-plugin-core (traits)                                    │   │
│  │  • git2 (libgit2 bindings)                                      │   │
│  │  • async-trait                                                   │   │
│  │  • tokio (spawn_blocking)                                         │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                  pheno-plugin-sqlite (lib)                       │   │
│  │                                                                  │   │
│  │  pub struct SqliteStoragePlugin;                                 │   │
│  │                                                                  │   │
│  │  impl AdapterPlugin for SqliteStoragePlugin                      │   │
│  │  impl StoragePlugin for SqliteStoragePlugin                      │   │
│  │                                                                  │   │
│  │  Dependencies:                                                    │   │
│  │  • pheno-plugin-core (traits)                                    │   │
│  │  • rusqlite (SQLite bindings)                                   │   │
│  │  • async-trait                                                   │   │
│  │  • tokio (spawn_blocking)                                         │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Core Components

### 4.1 pheno-plugin-core

The core crate defines all interfaces and shared types.

#### 4.1.1 Module Structure

```rust
pub mod error;    // Error types and Result alias
pub mod registry; // Plugin registry implementation
pub mod traits;   // Plugin trait definitions

pub use error::{PluginError, PluginResult};
pub use registry::{PluginRegistry, RegistryStats};
pub use traits::{AdapterPlugin, VcsPlugin, StoragePlugin, PluginConfig};
```

#### 4.1.2 Feature Flags

| Flag | Description | Default |
|------|-------------|---------|
| `runtime-tokio` | Enable tokio integration | yes |
| `test-utils` | Mock implementations for testing | no |

### 4.2 pheno-plugin-git

Git VCS adapter using libgit2 via the `git2` crate.

#### 4.2.1 Capabilities

- Worktree creation and management
- Branch operations (create, checkout)
- Merge operations with conflict detection
- Artifact reading/writing
- Feature artifact scanning

#### 4.2.2 Git Adapter Configuration

```rust
pub struct GitAdapter {
    repo_path: PathBuf,
}

impl GitAdapter {
    pub fn new(repo_path: impl Into<PathBuf>) -> PluginResult<Self>;
    pub fn from_cwd() -> PluginResult<Self>;
}
```

### 4.3 pheno-plugin-sqlite

SQLite storage adapter using `rusqlite`.

#### 4.3.1 Capabilities

- Feature CRUD operations
- Work package management
- Audit trail persistence
- Schema migrations

#### 4.3.2 Database Schema

```sql
-- Features table
CREATE TABLE features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    state TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Work packages table
CREATE TABLE work_packages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    state TEXT NOT NULL DEFAULT 'backlog',
    priority TEXT NOT NULL DEFAULT 'medium',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (feature_id) REFERENCES features(id)
);

-- Audit trail
CREATE TABLE audit_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER NOT NULL,
    entry_type TEXT NOT NULL,
    actor TEXT NOT NULL,
    details TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (feature_id) REFERENCES features(id)
);
```

---

## 5. Plugin Interfaces

### 5.1 AdapterPlugin (Base Trait)

All plugins MUST implement `AdapterPlugin`:

```rust
/// Configuration for plugin initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Adapter-specific configuration (JSON)
    #[serde(default)]
    pub adapter_config: serde_json::Value,
}

/// Base trait for all plugins.
pub trait AdapterPlugin: Send + Sync {
    /// Returns the plugin name (e.g., "git", "sqlite", "ollama").
    fn name(&self) -> &str;

    /// Returns the plugin version.
    fn version(&self) -> &str;

    /// Initializes the plugin with configuration.
    fn initialize(&self, config: PluginConfig) -> PluginResult<()>;

    /// Returns the plugin health status.
    fn health_check(&self) -> PluginResult<()> {
        Ok(())
    }
}
```

#### 5.1.1 Object Safety

`AdapterPlugin` is object-safe, allowing:

```rust
Box<dyn AdapterPlugin>  // Static dispatch not required
Arc<dyn AdapterPlugin>  // Thread-safe shared ownership
```

#### 5.1.2 Send + Sync Requirements

All plugins must be thread-safe:

- `Send`: Can move between threads
- `Sync`: Can share between threads

This enables the registry to be thread-safe.

### 5.2 VcsPlugin Trait

Version Control System operations for feature worktrees.

```rust
/// Metadata about an active git worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub feature_slug: String,
    pub wp_id: String,
}

/// Result of a merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<ConflictInfo>,
    pub merged_commit: Option<String>,
}

/// Description of a merge conflict in a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub path: String,
    pub ours: Option<String>,
    pub theirs: Option<String>,
}

/// Collected feature artifacts discovered in the repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureArtifacts {
    pub meta_json: Option<String>,
    pub audit_chain: Option<String>,
    pub evidence_paths: Vec<String>,
}

#[async_trait::async_trait]
pub trait VcsPlugin: AdapterPlugin {
    // Worktree operations
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
    async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>>;
    async fn cleanup_worktree(&self, worktree_path: &Path) -> PluginResult<()>;

    // Branch operations
    async fn create_branch(&self, branch_name: &str, base: &str) -> PluginResult<()>;
    async fn checkout_branch(&self, branch_name: &str) -> PluginResult<()>;

    // Merge operations
    async fn merge_to_target(&self, source: &str, target: &str) -> PluginResult<MergeResult>;
    async fn detect_conflicts(&self, source: &str, target: &str) -> PluginResult<Vec<ConflictInfo>>;

    // Artifact operations
    async fn read_artifact(&self, feature_slug: &str, relative_path: &str) -> PluginResult<String>;
    async fn write_artifact(&self, feature_slug: &str, relative_path: &str, content: &str) -> PluginResult<()>;
    async fn artifact_exists(&self, feature_slug: &str, relative_path: &str) -> PluginResult<bool>;
    async fn scan_feature_artifacts(&self, feature_slug: &str) -> PluginResult<FeatureArtifacts>;
}
```

#### 5.2.1 Worktree Operations

**create_worktree**
- Creates a new worktree for a feature work package
- Creates branch `feature/{slug}` from main/master
- Returns path to new worktree

**list_worktrees**
- Lists all active worktrees in the repository
- Parses feature_slug and wp_id from worktree names

**cleanup_worktree**
- Removes a worktree and its associated branch
- Prunes git worktree metadata

#### 5.2.2 Branch Operations

**create_branch**
- Creates new branch from specified base
- Does not checkout the new branch

**checkout_branch**
- Switches to specified branch
- Force checkout (discards local changes)

#### 5.2.3 Merge Operations

**merge_to_target**
- Merges source branch into target
- Detects conflicts and aborts if present
- Creates merge commit on success

**detect_conflicts**
- Pre-flights merge to identify conflicting files
- Does not modify repository state

#### 5.2.4 Artifact Operations

Artifacts are files stored in `kitty-specs/{feature_slug}/`:

**read_artifact**
- Reads file content as string
- Returns `PluginError::NotFound` if missing

**write_artifact**
- Writes content to file (creates directories)
- Overwrites if exists

**artifact_exists**
- Checks file existence

**scan_feature_artifacts**
- Discovers all artifacts for a feature
- Returns paths to meta.json, audit files, evidence

### 5.3 StoragePlugin Trait

Persistence operations for features and work packages.

```rust
#[async_trait::async_trait]
pub trait StoragePlugin: AdapterPlugin {
    // Feature operations
    async fn create_feature(&self, feature: &serde_json::Value) -> PluginResult<i64>;
    async fn get_feature_by_slug(&self, slug: &str) -> PluginResult<Option<serde_json::Value>>;
    async fn get_feature_by_id(&self, id: i64) -> PluginResult<Option<serde_json::Value>>;
    async fn update_feature_state(&self, id: i64, state: &str) -> PluginResult<()>;
    async fn list_all_features(&self) -> PluginResult<Vec<serde_json::Value>>;

    // Work package operations
    async fn create_work_package(&self, wp: &serde_json::Value) -> PluginResult<i64>;
    async fn get_work_package(&self, id: i64) -> PluginResult<Option<serde_json::Value>>;
    async fn update_wp_state(&self, id: i64, state: &str) -> PluginResult<()>;

    // Audit operations
    async fn append_audit_entry(&self, entry: &serde_json::Value) -> PluginResult<i64>;
    async fn get_audit_trail(&self, feature_id: i64) -> PluginResult<Vec<serde_json::Value>>;
}
```

#### 5.3.1 Feature Operations

**create_feature**
- Creates new feature from JSON data
- Required fields: `slug`, `name`
- Optional fields: `description`, `state`
- Returns auto-generated ID

**get_feature_by_slug**
- Retrieves feature by unique slug
- Returns `None` if not found

**get_feature_by_id**
- Retrieves feature by ID
- Returns `None` if not found

**update_feature_state**
- Updates feature state (draft, active, completed, etc.)
- Automatically updates `updated_at` timestamp

**list_all_features**
- Returns all features ordered by creation date (newest first)

#### 5.3.2 Work Package Operations

**create_work_package**
- Creates work package associated with feature
- Required: `feature_id`, `title`
- Optional: `description`, `state`, `priority`

**get_work_package**
- Retrieves work package by ID

**update_wp_state**
- Updates work package state

#### 5.3.3 Audit Operations

**append_audit_entry**
- Appends entry to feature's audit trail
- Required: `feature_id`
- Recommended: `entry_type`, `actor`, `details`

**get_audit_trail**
- Returns all audit entries for feature
- Ordered by timestamp (newest first)

---

## 6. Registry System

### 6.1 Registry Architecture

The `PluginRegistry` manages plugin instances and provides lookup:

```rust
pub struct PluginRegistry {
    vcs: RwLock<HashMap<String, Arc<dyn VcsPlugin>>>,
    storage: RwLock<HashMap<String, Arc<dyn StoragePlugin>>>,
    initialized: RwLock<bool>,
}
```

### 6.2 Registry Lifecycle

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Empty   │ -> │ Register │ -> │Finalize  │ -> │ Running  │
│          │    │  Plugins │    │          │    │          │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
                                     │
                                     ▼
                              ┌──────────┐
                              │  Locked  │
                              │  (frozen)│
                              └──────────┘
```

### 6.3 Registration API

```rust
impl PluginRegistry {
    /// Creates new empty registry.
    pub fn new() -> Self;

    /// Register a VCS plugin.
    pub fn register_vcs(&self, plugin: Box<dyn VcsPlugin>) -> PluginResult<()>;

    /// Register a storage plugin.
    pub fn register_storage(&self, plugin: Box<dyn StoragePlugin>) -> PluginResult<()>;

    /// Finalize - prevent further registration.
    pub fn finalize(&self) -> PluginResult<()>;

    /// Check if registry is finalized.
    pub fn is_finalized(&self) -> bool;
}
```

### 6.4 Lookup API

```rust
impl PluginRegistry {
    /// Get VCS plugin by name.
    pub fn vcs(&self, name: &str) -> Option<Arc<dyn VcsPlugin>>;

    /// Get storage plugin by name.
    pub fn storage(&self, name: &str) -> Option<Arc<dyn StoragePlugin>>;

    /// List all VCS plugin names.
    pub fn vcs_adapters(&self) -> Vec<String>;

    /// List all storage plugin names.
    pub fn storage_adapters(&self) -> Vec<String>;
}
```

### 6.5 Health Checking

```rust
impl PluginRegistry {
    /// Check health of all plugins.
    pub async fn health_check(&self) -> PluginResult<()>;

    /// Get registry statistics.
    pub fn stats(&self) -> RegistryStats;
}

#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub vcs_count: usize,
    pub storage_count: usize,
    pub finalized: bool,
}
```

### 6.6 Thread Safety

The registry uses interior mutability:

- `RwLock` for plugin maps (many readers, few writers)
- `Arc` for plugin instances (shared ownership)
- All operations are Send + Sync safe

---

## 7. Error Handling

### 7.1 Error Types

```rust
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    Initialization(String),

    #[error("Plugin `{0}` not found in registry")]
    NotFound(String),

    #[error("Plugin `{0}` already registered")]
    AlreadyRegistered(String),

    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    #[error("Operation failed: {0}")]
    Operation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Plugin execution error: {0}")]
    Execution(String),

    #[error("Validation error: {0}")]
    Validation(String),
}
```

### 7.2 Result Type

```rust
pub type PluginResult<T> = Result<T, PluginError>;
```

### 7.3 Error Propagation

Plugins use `?` operator for error propagation:

```rust
async fn create_worktree(&self, slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
    let repo = self.open_repo()?;  // git2::Error -> PluginError::Operation
    let base = self.main_branch_name()?;  // PluginError
    
    // git2 operations
    repo.branch(&branch_name, &commit, false)
        .map_err(|e| PluginError::Operation(e.to_string()))?;
    
    Ok(worktree_path)
}
```

### 7.4 Error Mapping

Git adapter maps git2 errors:

| git2::ErrorCode | PluginError |
|-----------------|-------------|
| NotFound | NotFound |
| Exists | AlreadyExists |
| _ | Operation |

SQLite adapter maps rusqlite errors:

| rusqlite::Error | PluginError |
|-----------------|-------------|
| QueryReturnedNoRows | Ok(None) |
| _ | Operation |

---

## 8. Lifecycle Management

### 8.1 Plugin States

```
┌─────────┐    ┌──────────┐    ┌─────────┐    ┌─────────┐
│ Created │ -> │Registered│ -> │ Active  │ -> │Shutdown │
│         │    │          │    │         │    │         │
└─────────┘    └──────────┘    └─────────┘    └─────────┘
                    │
                    ▼
              ┌──────────┐
              │  Failed  │
              │  (Error) │
              └──────────┘
```

### 8.2 State Transitions

| Transition | Trigger | Action |
|------------|---------|--------|
| Created -> Registered | `register_*` called | Plugin added to registry |
| Registered -> Active | `finalize()` called | Registry locked |
| Active -> Shutdown | Host shutdown | Plugins dropped |
| Any -> Failed | Error in initialization | Error propagated |

### 8.3 Initialization

```rust
// Host registers plugins before finalization
let registry = PluginRegistry::new();
registry.register_vcs(Box::new(GitAdapter::new(".")?))?;
registry.register_storage(Box::new(SqliteStoragePlugin::new("data.db")?))?;

// Finalize prevents further registration
registry.finalize()?;
```

### 8.4 Health Checks

Plugins implement health check for runtime validation:

```rust
impl AdapterPlugin for GitAdapter {
    fn health_check(&self) -> PluginResult<()> {
        // Verify repository is accessible
        self.open_repo()?;
        Ok(())
    }
}
```

---

## 9. Implementation Details

### 9.1 Git Adapter Implementation

The GitAdapter uses `git2` crate (libgit2 bindings):

```rust
pub struct GitAdapter {
    repo_path: PathBuf,
}

impl GitAdapter {
    fn open_repo(&self) -> Result<Repository, PluginError> {
        Repository::open(&self.repo_path).map_err(git_err)
    }
    
    fn main_branch_name(&self) -> PluginResult<String> {
        // Try "main", then "master", fallback to HEAD
    }
}

#[async_trait]
impl VcsPlugin for GitAdapter {
    async fn create_worktree(&self, slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
        // Spawn blocking git2 operations
        let path = self.repo_path.clone();
        let slug = slug.to_string();
        let wp = wp_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            Self::create_worktree_blocking(&path, &slug, &wp)
        }).await.map_err(|e| PluginError::Execution(e.to_string()))?
    }
}
```

### 9.2 SQLite Adapter Implementation

Uses `rusqlite` with WAL mode:

```rust
pub struct SqliteStoragePlugin {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl SqliteStoragePlugin {
    pub fn new(db_path: impl AsRef<Path>) -> PluginResult<Self> {
        let conn = Connection::open(&db_path)?;
        
        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        
        Self::run_migrations(&conn)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: db_path.as_ref().to_path_buf(),
        })
    }
}
```

### 9.3 Async Strategy

Both adapters use `tokio::task::spawn_blocking` for blocking operations:

```rust
async fn database_operation(&self) -> PluginResult<T> {
    let conn = self.conn.clone();
    
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock()?;
        // Blocking database operation
    }).await.map_err(|e| PluginError::Execution(e.to_string()))?
}
```

This pattern:
- Keeps async interface
- Prevents blocking async runtime
- Maintains thread safety

---

## 10. Testing Strategy

### 10.1 Unit Testing

Plugins tested in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_repo() -> PluginResult<(TempDir, GitAdapter)> {
        let temp = TempDir::new()?;
        Repository::init(temp.path())?;
        let adapter = GitAdapter::new(temp.path())?;
        Ok((temp, adapter))
    }
    
    #[tokio::test]
    async fn test_create_worktree() -> PluginResult<()> {
        let (_temp, adapter) = create_test_repo()?;
        let path = adapter.create_worktree("test", "WP001").await?;
        assert!(path.exists());
        Ok(())
    }
}
```

### 10.2 Mock Plugins

Mock implementations for host testing:

```rust
struct MockVcsPlugin {
    worktrees: Arc<Mutex<Vec<WorktreeInfo>>>,
}

#[async_trait]
impl VcsPlugin for MockVcsPlugin {
    async fn create_worktree(&self, slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
        let path = PathBuf::from(format!("/tmp/{}-{}", slug, wp_id));
        self.worktrees.lock().unwrap().push(WorktreeInfo {
            path: path.clone(),
            branch: format!("feature/{}", slug),
            feature_slug: slug.to_string(),
            wp_id: wp_id.to_string(),
        });
        Ok(path)
    }
    // ...
}
```

### 10.3 Integration Testing

Tests with real git/SQLite:

```rust
#[tokio::test]
async fn test_git_sqlite_integration() -> PluginResult<()> {
    let temp = TempDir::new()?;
    Repository::init(temp.path())?;
    
    let git = GitAdapter::new(temp.path())?;
    let sqlite = SqliteStoragePlugin::in_memory()?;
    
    // Create feature in SQLite
    let feature = serde_json::json!({
        "slug": "test-feature",
        "name": "Test Feature"
    });
    let id = sqlite.create_feature(&feature).await?;
    
    // Create worktree via git
    let path = git.create_worktree("test-feature", "WP001").await?;
    
    // Verify
    assert!(path.exists());
    assert!(id > 0);
    
    Ok(())
}
```

---

## 11. Performance Characteristics

### 11.1 Overhead Analysis

| Operation | Overhead | Notes |
|-----------|----------|-------|
| Trait dispatch | 1-3ns | Virtual call through vtable |
| async_trait boxing | 50-100ns | Allocation per async call |
| Arc clone | ~10ns | Reference count increment |
| RwLock read | 20-50ns | Uncontested |
| Registry lookup | 50-100ns | HashMap lookup |

### 11.2 Baseline Comparisons

Operation latencies (excluding plugin overhead):

| Operation | Git | SQLite | Mock |
|-----------|-----|--------|------|
| create_worktree | 10-100ms | N/A | <1µs |
| read_artifact | 0.1-1ms | N/A | <1µs |
| create_feature | N/A | 1-10ms | <1µs |
| get_feature | N/A | 0.1-1ms | <1µs |

### 11.3 Throughput

With tokio runtime (default configuration):

- Concurrent plugin operations: 10,000+
- Registry operations/second: 1,000,000+

### 11.4 Memory Usage

Per-plugin overhead:

| Component | Size |
|-----------|------|
| GitAdapter | ~48 bytes (PathBuf) |
| SqliteStoragePlugin | ~40 bytes (Arc + PathBuf) |
| Registry entry | ~32 bytes (HashMap node) |

Negligible compared to underlying resources (git repo, database).

---

## 12. Security Model

### 12.1 Threat Model

PhenoPlugins operates in a trusted environment:

- All plugins internal to Phenotype
- All code reviewed before merge
- No third-party plugins accepted
- Rust memory safety guarantees

### 12.2 Security Properties

| Property | Mechanism |
|----------|-----------|
| Memory safety | Rust ownership + borrowing |
| Type safety | Rust type system |
| No data races | Rust Send/Sync traits |
| Input validation | PluginError::Validation |
| Error isolation | Result types (no panics) |

### 12.3 Limitations

- No crash isolation (plugin panic = host panic)
- No resource limits enforced
- Plugins share filesystem with host

### 12.4 Mitigations

1. Code review requirements
2. Comprehensive test coverage
3. Health check monitoring
4. Input validation at boundaries
5. Panic handling with `catch_unwind`

---

## 13. Versioning and Evolution

### 13.1 Semantic Versioning

PhenoPlugins follows SemVer:

- **MAJOR**: Breaking trait changes
- **MINOR**: New traits, non-breaking additions
- **PATCH**: Bug fixes, documentation

### 13.2 Interface Stability

Traits are stable within major versions:

| Version | Trait Stability |
|---------|----------------|
| 1.x | Stable |
| 2.x (future) | Breaking changes allowed |

### 13.3 Evolution Strategy

New capabilities added via:

1. **New methods**: Minor version bump
2. **New traits**: Minor version bump
3. **Breaking changes**: New trait name or major version

Example:
```rust
// V1: VcsPlugin
// V2: VcsPluginV2 (if breaking changes needed)
pub trait VcsPluginV2: VcsPlugin {
    async fn new_capability(&self) -> PluginResult<()>;
}
```

### 13.4 Deprecation

Deprecated items marked with `#[deprecated]`:

```rust
#[deprecated(since = "1.2.0", note = "Use new_method instead")]
async fn old_method(&self) -> PluginResult<()>;
```

---

## 14. Integration Guide

### 14.1 Adding PhenoPlugins to a Host

Cargo.toml:
```toml
[dependencies]
pheno-plugin-core = { path = "../pheno-plugin-core" }
pheno-plugin-git = { path = "../pheno-plugin-git" }
pheno-plugin-sqlite = { path = "../pheno-plugin-sqlite" }
tokio = { version = "1", features = ["full"] }
```

### 14.2 Basic Integration

```rust
use pheno_plugin_core::PluginRegistry;
use pheno_plugin_git::GitAdapter;
use pheno_plugin_sqlite::SqliteStoragePlugin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry
    let registry = PluginRegistry::new();
    
    // Register plugins
    registry.register_vcs(Box::new(GitAdapter::from_cwd()?))?;
    registry.register_storage(Box::new(SqliteStoragePlugin::new("data.db")?))?;
    
    // Finalize
    registry.finalize()?;
    
    // Use plugins
    if let Some(vcs) = registry.vcs("git") {
        let path = vcs.create_worktree("my-feature", "WP001").await?;
        println!("Created worktree at: {:?}", path);
    }
    
    Ok(())
}
```

### 14.3 Advanced Integration

With dependency injection:

```rust
pub struct FeatureManager {
    vcs: Arc<dyn VcsPlugin>,
    storage: Arc<dyn StoragePlugin>,
}

impl FeatureManager {
    pub fn new(registry: &PluginRegistry) -> PluginResult<Self> {
        let vcs = registry.vcs("git")
            .ok_or_else(|| PluginError::NotFound("git".to_string()))?;
        let storage = registry.storage("sqlite-storage")
            .ok_or_else(|| PluginError::NotFound("sqlite-storage".to_string()))?;
        
        Ok(Self { vcs, storage })
    }
    
    pub async fn create_feature(&self, slug: &str) -> PluginResult<i64> {
        // Create worktree
        let path = self.vcs.create_worktree(slug, "WP001").await?;
        
        // Persist feature
        let feature = serde_json::json!({
            "slug": slug,
            "name": slug,
            "state": "draft"
        });
        let id = self.storage.create_feature(&feature).await?;
        
        Ok(id)
    }
}
```

---

## 15. Deployment

### 15.1 Build Configuration

Release build with optimizations:

```bash
cargo build --release --workspace
```

### 15.2 Testing

```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --workspace --test integration

# Documentation tests
cargo test --workspace --doc
```

### 15.3 Documentation

```bash
# Generate API docs
cargo doc --workspace --no-deps --open
```

---

## 16. Monitoring and Observability

### 16.1 Health Checks

```rust
// Periodic health check
async fn check_plugins(registry: &PluginRegistry) {
    match registry.health_check().await {
        Ok(()) => println!("All plugins healthy"),
        Err(e) => eprintln!("Plugin health check failed: {}", e),
    }
    
    let stats = registry.stats();
    println!("Registry: {} VCS, {} storage", stats.vcs_count, stats.storage_count);
}
```

### 16.2 Metrics (Future)

Planned metrics integration:

```rust
pub trait ObservablePlugin: AdapterPlugin {
    fn metrics(&self) -> Vec<Metric>;
}

pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
}
```

### 16.3 Tracing

Plugins integrate with `tracing`:

```rust
use tracing::{info, error, instrument};

#[async_trait]
impl VcsPlugin for GitAdapter {
    #[instrument(skip(self))]
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
        info!("Creating worktree for {}-{}", feature_slug, wp_id);
        // ...
    }
}
```

---

## 17. Future Directions

### 17.1 Planned Features

| Feature | Priority | Description |
|---------|----------|-------------|
| PostgreSQL plugin | P1 | PostgreSQL storage adapter |
| Metrics API | P2 | Standardized metrics export |
| Plugin Discovery | P2 | Dynamic plugin loading |
| WASM Support | P3 | WebAssembly plugin sandbox |

### 17.2 Research Areas

- Native async traits (when stabilized)
- Plugin hot-reloading
- Distributed plugin coordination

### 17.3 Ecosystem Expansion

Potential new plugin types:

- LLM integration (Ollama, OpenAI)
- Cloud provider adapters (AWS, GCP, Azure)
- Notification services (Email, Slack, Discord)

---

## 18. References

### 18.1 Internal Documents

- [CHARTER.md](./CHARTER.md) - Project charter and tenets
- [SOTA.md](./SOTA.md) - State of the art research
- [docs/adr/](./docs/adr/) - Architecture Decision Records

### 18.2 External References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [async-trait crate](https://docs.rs/async-trait/)
- [git2 crate](https://docs.rs/git2/)
- [rusqlite crate](https://docs.rs/rusqlite/)

### 18.3 Related Specifications

- AgilePlus Spec-001 (Plugin System Completion)
- Phenotype Architecture Guidelines

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| Adapter | Concrete implementation of a plugin trait |
| ADR | Architecture Decision Record |
| Async | Asynchronous programming with futures |
| Port | Interface (trait) in hexagonal architecture |
| Plugin | Implementor of one or more plugin traits |
| Registry | Central plugin management component |
| Trait | Rust interface definition |
| VCS | Version Control System (git) |
| WAL | Write-Ahead Logging (SQLite) |
| Worktree | Git feature work directory |
| WP | Work Package |

## Appendix B: Changelog

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-04-04 | Initial specification |

## Appendix C: FAQ

**Q: Why not use WASM for all plugins?**
A: WASM adds overhead (2-10MB, compilation time) unnecessary for trusted internal plugins. Trait-based gives zero overhead.

**Q: Can I write a plugin in another language?**
A: Not directly. PhenoPlugins is Rust-native. Future WASM support may enable other languages.

**Q: How do I add a new plugin type?**
A: Define a new trait in `pheno-plugin-core`, implement in a new crate following the adapter pattern.

**Q: What happens if a plugin panics?**
A: Currently, plugin panic crashes host. Use `catch_unwind` at boundaries for critical applications.

**Q: Can plugins depend on each other?**
A: Indirectly through the host. Host can orchestrate multiple plugins.

## Appendix D: Complete API Reference

### D.1 pheno-plugin-core API

#### D.1.1 Error Types

Full error type specification:

```rust
/// The error type for all plugin operations.
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin failed to initialize.
    #[error("Plugin initialization failed: {0}")]
    Initialization(String),

    /// Plugin not found in registry.
    #[error("Plugin `{0}` not found in registry")]
    NotFound(String),

    /// Plugin already registered.
    #[error("Plugin `{0}` already registered")]
    AlreadyRegistered(String),

    /// Entity (feature, work package) already exists.
    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    /// Generic operation failure.
    #[error("Operation failed: {0}")]
    Operation(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Plugin execution error.
    #[error("Plugin execution error: {0}")]
    Execution(String),

    /// Input validation error.
    #[error("Validation error: {0}")]
    Validation(String),
}
```

#### D.1.2 PluginConfig

Configuration structure for plugin initialization:

```rust
/// Configuration for a plugin adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name (must be unique within type)
    pub name: String,
    
    /// Plugin version (semantic versioning)
    pub version: String,
    
    /// Adapter-specific configuration
    /// JSON object with plugin-specific settings
    #[serde(default)]
    pub adapter_config: serde_json::Value,
}

impl PluginConfig {
    /// Create minimal config with name and version.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            adapter_config: serde_json::Value::Object(Default::default()),
        }
    }
    
    /// Add adapter-specific configuration.
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.adapter_config = config;
        self
    }
}
```

#### D.1.3 RegistryStats

Statistics structure for registry inspection:

```rust
/// Statistics about the plugin registry.
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Number of registered VCS plugins.
    pub vcs_count: usize,
    
    /// Number of registered storage plugins.
    pub storage_count: usize,
    
    /// Whether the registry has been finalized.
    pub finalized: bool,
}

impl RegistryStats {
    /// Total number of registered plugins.
    pub fn total(&self) -> usize {
        self.vcs_count + self.storage_count
    }
    
    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}
```

### D.2 Trait Method Specifications

#### D.2.1 AdapterPlugin Methods

| Method | Input | Output | Description |
|--------|-------|--------|-------------|
| `name` | `&self` | `&str` | Returns plugin identifier |
| `version` | `&self` | `&str` | Returns semantic version |
| `initialize` | `&self`, `PluginConfig` | `PluginResult<()>` | One-time initialization |
| `health_check` | `&self` | `PluginResult<()>` | Runtime health validation |

#### D.2.2 VcsPlugin Methods - Worktree Operations

| Method | Input | Output | Latency |
|--------|-------|--------|---------|
| `create_worktree` | `feature_slug: &str`, `wp_id: &str` | `PluginResult<PathBuf>` | 10-100ms |
| `list_worktrees` | - | `PluginResult<Vec<WorktreeInfo>>` | 1-10ms |
| `cleanup_worktree` | `worktree_path: &Path` | `PluginResult<()>` | 10-50ms |

#### D.2.3 VcsPlugin Methods - Branch Operations

| Method | Input | Output | Latency |
|--------|-------|--------|---------|
| `create_branch` | `branch_name: &str`, `base: &str` | `PluginResult<()>` | 5-20ms |
| `checkout_branch` | `branch_name: &str` | `PluginResult<()>` | 5-50ms |

#### D.2.4 VcsPlugin Methods - Merge Operations

| Method | Input | Output | Latency |
|--------|-------|--------|---------|
| `merge_to_target` | `source: &str`, `target: &str` | `PluginResult<MergeResult>` | 10-100ms |
| `detect_conflicts` | `source: &str`, `target: &str` | `PluginResult<Vec<ConflictInfo>>` | 10-50ms |

#### D.2.5 VcsPlugin Methods - Artifact Operations

| Method | Input | Output | Latency |
|--------|-------|--------|---------|
| `read_artifact` | `feature_slug: &str`, `relative_path: &str` | `PluginResult<String>` | 0.1-1ms |
| `write_artifact` | `feature_slug: &str`, `relative_path: &str`, `content: &str` | `PluginResult<()>` | 1-10ms |
| `artifact_exists` | `feature_slug: &str`, `relative_path: &str` | `PluginResult<bool>` | 0.1-1ms |
| `scan_feature_artifacts` | `feature_slug: &str` | `PluginResult<FeatureArtifacts>` | 1-10ms |

#### D.2.6 StoragePlugin Methods - Feature Operations

| Method | Input | Output | Latency |
|--------|-------|--------|---------|
| `create_feature` | `feature: &serde_json::Value` | `PluginResult<i64>` | 1-10ms |
| `get_feature_by_slug` | `slug: &str` | `PluginResult<Option<serde_json::Value>>` | 0.1-1ms |
| `get_feature_by_id` | `id: i64` | `PluginResult<Option<serde_json::Value>>` | 0.1-1ms |
| `update_feature_state` | `id: i64`, `state: &str` | `PluginResult<()>` | 1-10ms |
| `list_all_features` | - | `PluginResult<Vec<serde_json::Value>>` | 1-100ms |

#### D.2.7 StoragePlugin Methods - Work Package Operations

| Method | Input | Output | Latency |
|--------|-------|--------|---------|
| `create_work_package` | `wp: &serde_json::Value` | `PluginResult<i64>` | 1-10ms |
| `get_work_package` | `id: i64` | `PluginResult<Option<serde_json::Value>>` | 0.1-1ms |
| `update_wp_state` | `id: i64`, `state: &str` | `PluginResult<()>` | 1-10ms |

#### D.2.8 StoragePlugin Methods - Audit Operations

| Method | Input | Output | Latency |
|--------|-------|--------|---------|
| `append_audit_entry` | `entry: &serde_json::Value` | `PluginResult<i64>` | 1-10ms |
| `get_audit_trail` | `feature_id: i64` | `PluginResult<Vec<serde_json::Value>>` | 1-10ms |

### D.3 Data Types Specification

#### D.3.1 WorktreeInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    /// Absolute path to worktree directory.
    pub path: PathBuf,
    
    /// Git branch name.
    pub branch: String,
    
    /// Feature slug extracted from branch name.
    pub feature_slug: String,
    
    /// Work package ID extracted from worktree name.
    pub wp_id: String,
}
```

#### D.3.2 MergeResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Whether merge completed successfully.
    pub success: bool,
    
    /// List of conflicts if merge failed.
    pub conflicts: Vec<ConflictInfo>,
    
    /// Merge commit hash if successful.
    pub merged_commit: Option<String>,
}
```

#### D.3.3 ConflictInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// File path with conflict.
    pub path: String,
    
    /// Our version of conflicting hunk (if available).
    pub ours: Option<String>,
    
    /// Their version of conflicting hunk (if available).
    pub theirs: Option<String>,
}
```

#### D.3.4 FeatureArtifacts

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureArtifacts {
    /// Path to meta.json if present.
    pub meta_json: Option<String>,
    
    /// Path to audit chain file if present.
    pub audit_chain: Option<String>,
    
    /// Paths to all evidence files.
    pub evidence_paths: Vec<String>,
}
```

### D.4 Registry API Reference

#### D.4.1 Constructor and Lifecycle

```rust
impl PluginRegistry {
    /// Create new empty registry.
    /// 
    /// # Example
    /// ```
    /// let registry = PluginRegistry::new();
    /// assert!(!registry.is_finalized());
    /// ```
    pub fn new() -> Self;
    
    /// Finalize registry, preventing further registration.
    ///
    /// # Errors
    /// Returns error if already finalized.
    ///
    /// # Example
    /// ```
    /// let registry = PluginRegistry::new();
    /// registry.finalize()?;
    /// assert!(registry.is_finalized());
    /// ```
    pub fn finalize(&self) -> PluginResult<()>;
    
    /// Check if registry is finalized.
    pub fn is_finalized(&self) -> bool;
}
```

#### D.4.2 VCS Plugin Registration

```rust
impl PluginRegistry {
    /// Register a VCS plugin.
    ///
    /// # Arguments
    /// * `plugin` - Boxed VCS plugin implementation
    ///
    /// # Errors
    /// * `AlreadyRegistered` - Plugin with same name exists
    /// * `Initialization` - Registry is finalized
    ///
    /// # Example
    /// ```
    /// let registry = PluginRegistry::new();
    /// registry.register_vcs(Box::new(GitAdapter::new(".")?))?;
    /// ```
    pub fn register_vcs(&self, plugin: Box<dyn VcsPlugin>) -> PluginResult<()>;
    
    /// Get VCS plugin by name.
    ///
    /// Returns `None` if plugin not found.
    pub fn vcs(&self, name: &str) -> Option<Arc<dyn VcsPlugin>>;
    
    /// List all registered VCS plugin names.
    pub fn vcs_adapters(&self) -> Vec<String>;
}
```

#### D.4.3 Storage Plugin Registration

```rust
impl PluginRegistry {
    /// Register a storage plugin.
    ///
    /// # Arguments
    /// * `plugin` - Boxed storage plugin implementation
    ///
    /// # Errors
    /// * `AlreadyRegistered` - Plugin with same name exists
    /// * `Initialization` - Registry is finalized
    pub fn register_storage(&self, plugin: Box<dyn StoragePlugin>) -> PluginResult<()>;
    
    /// Get storage plugin by name.
    pub fn storage(&self, name: &str) -> Option<Arc<dyn StoragePlugin>>;
    
    /// List all registered storage plugin names.
    pub fn storage_adapters(&self) -> Vec<String>;
}
```

#### D.4.4 Operations

```rust
impl PluginRegistry {
    /// Run health checks on all plugins.
    ///
    /// Returns `Ok(())` if all plugins healthy.
    /// Returns first error encountered.
    pub async fn health_check(&self) -> PluginResult<()>;
    
    /// Get registry statistics.
    pub fn stats(&self) -> RegistryStats;
}
```

## Appendix E: Implementation Examples

### E.1 Complete Host Implementation

```rust
use pheno_plugin_core::{PluginRegistry, PluginError, PluginResult};
use pheno_plugin_git::GitAdapter;
use pheno_plugin_sqlite::SqliteStoragePlugin;
use std::sync::Arc;

/// Application context holding plugin registry.
pub struct Application {
    registry: PluginRegistry,
}

impl Application {
    /// Initialize application with plugins.
    pub async fn initialize() -> PluginResult<Self> {
        let registry = PluginRegistry::new();
        
        // Register Git plugin
        let git = GitAdapter::from_cwd()?;
        registry.register_vcs(Box::new(git))?;
        
        // Register SQLite plugin
        let sqlite = SqliteStoragePlugin::new("agileplus.db")?;
        registry.register_storage(Box::new(sqlite))?;
        
        // Finalize before use
        registry.finalize()?;
        
        // Health check
        registry.health_check().await?;
        
        Ok(Self { registry })
    }
    
    /// Create a new feature with associated resources.
    pub async fn create_feature(&self, slug: &str, name: &str) -> PluginResult<Feature> {
        let vcs = self.registry.vcs("git")
            .ok_or_else(|| PluginError::NotFound("git".to_string()))?;
        let storage = self.registry.storage("sqlite-storage")
            .ok_or_else(|| PluginError::NotFound("sqlite-storage".to_string()))?;
        
        // Create worktree
        let worktree_path = vcs.create_worktree(slug, "WP001").await?;
        
        // Persist feature
        let feature_data = serde_json::json!({
            "slug": slug,
            "name": name,
            "state": "draft",
            "worktree_path": worktree_path.to_string_lossy(),
        });
        let id = storage.create_feature(&feature_data).await?;
        
        // Create initial artifact
        let meta = serde_json::json!({
            "id": id,
            "slug": slug,
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
        vcs.write_artifact(slug, "meta.json", &meta.to_string()).await?;
        
        Ok(Feature {
            id,
            slug: slug.to_string(),
            name: name.to_string(),
            worktree_path,
        })
    }
    
    /// Get plugin statistics.
    pub fn stats(&self) -> RegistryStats {
        self.registry.stats()
    }
}

/// Feature domain model.
pub struct Feature {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub worktree_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let app = Application::initialize().await?;
    
    let feature = app.create_feature("test-feature", "Test Feature").await?;
    println!("Created feature {} at {:?}", feature.id, feature.worktree_path);
    
    let stats = app.stats();
    println!("Registry stats: {:?}", stats);
    
    Ok(())
}
```

### E.2 Mock Plugin for Testing

```rust
#[cfg(test)]
mod mock {
    use pheno_plugin_core::traits::*;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    
    /// Mock VCS plugin for testing.
    pub struct MockVcsPlugin {
        worktrees: Arc<Mutex<Vec<WorktreeInfo>>>,
        artifacts: Arc<Mutex<HashMap<(String, String), String>>>,
    }
    
    impl MockVcsPlugin {
        pub fn new() -> Self {
            Self {
                worktrees: Arc::new(Mutex::new(Vec::new())),
                artifacts: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }
    
    impl AdapterPlugin for MockVcsPlugin {
        fn name(&self) -> &str { "mock-vcs" }
        fn version(&self) -> &str { "0.1.0" }
        fn initialize(&self, _config: PluginConfig) -> PluginResult<()> { Ok(()) }
    }
    
    #[async_trait::async_trait]
    impl VcsPlugin for MockVcsPlugin {
        async fn create_worktree(&self, slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
            let path = PathBuf::from(format!("/tmp/worktrees/{}-{}", slug, wp_id));
            
            let info = WorktreeInfo {
                path: path.clone(),
                branch: format!("feature/{}", slug),
                feature_slug: slug.to_string(),
                wp_id: wp_id.to_string(),
            };
            
            self.worktrees.lock().unwrap().push(info);
            Ok(path)
        }
        
        async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>> {
            Ok(self.worktrees.lock().unwrap().clone())
        }
        
        async fn cleanup_worktree(&self, path: &Path) -> PluginResult<()> {
            let mut worktrees = self.worktrees.lock().unwrap();
            worktrees.retain(|w| w.path != path);
            Ok(())
        }
        
        async fn create_branch(&self, _name: &str, _base: &str) -> PluginResult<()> {
            Ok(())
        }
        
        async fn checkout_branch(&self, _name: &str) -> PluginResult<()> {
            Ok(())
        }
        
        async fn merge_to_target(&self, _source: &str, _target: &str) -> PluginResult<MergeResult> {
            Ok(MergeResult {
                success: true,
                conflicts: vec![],
                merged_commit: Some("abc123".to_string()),
            })
        }
        
        async fn detect_conflicts(&self, _source: &str, _target: &str) -> PluginResult<Vec<ConflictInfo>> {
            Ok(vec![])
        }
        
        async fn read_artifact(&self, slug: &str, path: &str) -> PluginResult<String> {
            self.artifacts.lock().unwrap()
                .get(&(slug.to_string(), path.to_string()))
                .cloned()
                .ok_or_else(|| PluginError::NotFound(format!("{}/{}", slug, path)))
        }
        
        async fn write_artifact(&self, slug: &str, path: &str, content: &str) -> PluginResult<()> {
            self.artifacts.lock().unwrap()
                .insert((slug.to_string(), path.to_string()), content.to_string());
            Ok(())
        }
        
        async fn artifact_exists(&self, slug: &str, path: &str) -> PluginResult<bool> {
            Ok(self.artifacts.lock().unwrap()
                .contains_key(&(slug.to_string(), path.to_string())))
        }
        
        async fn scan_feature_artifacts(&self, slug: &str) -> PluginResult<FeatureArtifacts> {
            let artifacts = self.artifacts.lock().unwrap();
            let mut meta_json = None;
            let mut evidence_paths = Vec::new();
            
            for ((s, path), _) in artifacts.iter() {
                if s == slug {
                    if path == "meta.json" {
                        meta_json = Some(format!("kitty-specs/{}/{}", slug, path));
                    } else {
                        evidence_paths.push(format!("kitty-specs/{}/{}", slug, path));
                    }
                }
            }
            
            Ok(FeatureArtifacts {
                meta_json,
                audit_chain: None,
                evidence_paths,
            })
        }
    }
}
```

### E.3 Custom Plugin Implementation

```rust
/// Example: Custom Git hosting provider adapter.
use pheno_plugin_core::traits::*;
use std::path::PathBuf;

pub struct GitHubEnterpriseAdapter {
    base_url: String,
    token: String,
    local_path: PathBuf,
}

impl GitHubEnterpriseAdapter {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>, local_path: impl Into<PathBuf>) -> Self {
        Self {
            base_url: base_url.into(),
            token: token.into(),
            local_path: local_path.into(),
        }
    }
}

impl AdapterPlugin for GitHubEnterpriseAdapter {
    fn name(&self) -> &str { "github-enterprise" }
    fn version(&self) -> &str { env!("CARGO_PKG_VERSION") }
    fn initialize(&self, _config: PluginConfig) -> PluginResult<()> {
        // Validate credentials
        Ok(())
    }
}

#[async_trait::async_trait]
impl VcsPlugin for GitHubEnterpriseAdapter {
    async fn create_worktree(&self, slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
        // Implement GHE-specific worktree creation
        // Could use GHE API for remote operations
        // Fall back to standard git for local
        todo!("Implement GHE-specific worktree creation")
    }
    
    // ... implement other methods
}
```

## Appendix F: Best Practices

### F.1 Plugin Design

1. **Keep traits focused**: Each trait should have a single responsibility
2. **Use async for I/O**: Any network or disk operation should be async
3. **Implement health_check**: Always provide health check for monitoring
4. **Handle errors gracefully**: Map errors to appropriate PluginError variants
5. **Validate inputs**: Check all inputs and return Validation errors

### F.2 Host Integration

1. **Finalize early**: Call finalize() before starting operations
2. **Check health**: Run health checks on startup
3. **Handle None**: Always handle the case where plugin lookup returns None
4. **Clone Arc**: Clone the Arc before holding across await points
5. **Log operations**: Use tracing for observability

### F.3 Testing

1. **Use mocks**: Create mock implementations for unit tests
2. **Test error cases**: Verify error handling paths
3. **Integration tests**: Test with real plugins for integration
4. **Concurrent tests**: Verify thread safety under load
5. **Cleanup**: Always clean up test resources

## Appendix G: Troubleshooting

### G.1 Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `NotFound` | Plugin not registered | Check registration, verify name |
| `AlreadyRegistered` | Duplicate name | Use unique plugin names |
| `Initialization` | Registry finalized | Cannot register after finalize() |
| `Io` | File system error | Check permissions, paths |
| `Operation` | Underlying library error | Check library-specific logs |

### G.2 Debugging Tips

1. Enable trace logging:
   ```rust
   tracing_subscriber::fmt()
       .with_max_level(tracing::Level::TRACE)
       .init();
   ```

2. Check registry state:
   ```rust
   println!("Stats: {:?}", registry.stats());
   println!("VCS: {:?}", registry.vcs_adapters());
   ```

3. Verify plugin health:
   ```rust
   if let Some(plugin) = registry.vcs("git") {
       match plugin.health_check() {
           Ok(()) => println!("Healthy"),
           Err(e) => println!("Unhealthy: {}", e),
       }
   }
   ```

### G.3 Performance Tuning

1. **Reduce clones**: Reuse Arc instead of cloning inner data
2. **Batch operations**: Group multiple operations when possible
3. **Connection pooling**: Reuse connections in storage plugins
4. **Spawn blocking**: Always use spawn_blocking for sync I/O

## Appendix H: Migration Guide

### H.1 From V0.1 to Future Versions

When new major versions are released, follow these migration steps:

1. Check deprecation warnings
2. Review trait changes
3. Update plugin implementations
4. Test thoroughly
5. Update documentation

### H.2 Adding New Plugin Types

To add a new plugin type:

1. Define trait in `pheno-plugin-core`:
   ```rust
   #[async_trait]
   pub trait NotificationPlugin: AdapterPlugin {
       async fn send(&self, message: &str) -> PluginResult<()>;
   }
   ```

2. Add registry methods:
   ```rust
   pub fn register_notification(&self, plugin: Box<dyn NotificationPlugin>) -> PluginResult<()>;
   pub fn notification(&self, name: &str) -> Option<Arc<dyn NotificationPlugin>>;
   ```

3. Implement in new crate:
   ```rust
   pub struct SlackNotificationPlugin;
   impl NotificationPlugin for SlackNotificationPlugin { ... }
   ```

## Appendix I: Detailed Design Rationale

### I.1 Why Traits Over Other Approaches

The decision to use Rust traits as the plugin interface was made after extensive evaluation of alternatives:

#### I.1.1 Alternative: C FFI

A C-based FFI interface would enable cross-language plugins:

```rust
// Hypothetical C FFI approach
#[repr(C)]
pub struct PluginVTable {
    pub name: extern "C" fn() -> *const c_char,
    pub version: extern "C" fn() -> *const c_char,
    pub execute: extern "C" fn(*const c_char) -> *const c_char,
}

#[no_mangle]
pub extern "C" fn plugin_init() -> *const PluginVTable {
    // ...
}
```

**Why Rejected:**
- Memory management complexity (who frees strings?)
- No type safety (void* everywhere)
- Manual error code mapping
- Unsafe code required
- No IDE support for autocompletion

#### I.1.2 Alternative: gRPC/Prost

Protocol buffers with gRPC would provide language-agnostic interfaces:

```protobuf
service VcsPlugin {
    rpc CreateWorktree(CreateWorktreeRequest) returns (CreateWorktreeResponse);
    rpc ListWorktrees(ListWorktreesRequest) returns (ListworktreesResponse);
}
```

**Why Rejected:**
- Serialization overhead for every call
- Process boundary required (high latency)
- Complex deployment (multiple binaries)
- Debug complexity (network issues)
- No shared memory possible

#### I.1.3 Alternative: WebAssembly

WASM provides sandboxing and language support:

```rust
// Plugin compiled to WASM
#[no_mangle]
pub extern "C" fn create_worktree(slug: &str, wp_id: &str) -> i32 {
    // ...
}
```

**Why Deferred:**
- 2-10MB memory overhead per plugin
- WASI limitations for file system operations
- Additional runtime dependency
- Startup compilation latency
- Overkill for trusted internal plugins

**Decision Rationale:**
- Phenotype ecosystem is Rust-native
- Performance is critical (git operations frequent)
- Trust model doesn't require sandboxing
- Maximum IDE support desired

### I.2 Why async_trait Over Native Async

Rust's native async traits are still being stabilized:

```rust
// Native async trait (unstable)
#![feature(async_fn_in_trait)]

pub trait VcsPlugin {
    async fn create_worktree(&self, slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
}
```

**Current Approach (async_trait):**

```rust
#[async_trait]
pub trait VcsPlugin {
    async fn create_worktree(&self, slug: &str, wp_id: &str) -> PluginResult<PathBuf>;
}
```

**Why async_trait:**
- Works on stable Rust today
- Industry standard (widely used)
- Minimal overhead (~50-100ns per call)
- Easy migration path when native stabilizes

**Future Migration:**
When native async traits stabilize, migration requires only:
1. Remove `#[async_trait]` macro
2. No host code changes needed

### I.3 Why Registry Pattern Over Dependency Injection

Alternative: DI container like `shaku` or `archery`:

```rust
// Hypothetical DI approach
#[derive(Component)]
#[shaku(interface = dyn VcsPlugin)]
struct GitAdapter { /* ... */ }

module! {
    MyModule {
        components = [GitAdapter],
        providers = []
    }
}
```

**Why Simple Registry:**
- No additional dependencies
- Simpler mental model
- Explicit over implicit
- Easier to debug
- Sufficient for current needs

## Appendix J: Code Examples Library

### J.1 Feature Creation Workflow

```rust
use pheno_plugin_core::{PluginRegistry, PluginResult, PluginError};
use pheno_plugin_git::GitAdapter;
use pheno_plugin_sqlite::SqliteStoragePlugin;
use std::sync::Arc;

/// Complete workflow for creating a new feature.
pub async fn create_feature_workflow(
    registry: &PluginRegistry,
    slug: &str,
    name: &str,
    description: Option<&str>,
) -> PluginResult<FeatureCreated> {
    // Get plugins
    let vcs = registry
        .vcs("git")
        .ok_or_else(|| PluginError::NotFound("git plugin not found".to_string()))?;
    
    let storage = registry
        .storage("sqlite-storage")
        .ok_or_else(|| PluginError::NotFound("sqlite plugin not found".to_string()))?;
    
    // Create worktree
    let worktree_path = vcs.create_worktree(slug, "WP001").await?;
    
    // Create feature in database
    let feature_data = serde_json::json!({
        "slug": slug,
        "name": name,
        "description": description,
        "state": "draft",
        "created_at": chrono::Utc::now().to_rfc3339(),
    });
    
    let feature_id = storage.create_feature(&feature_data).await?;
    
    // Create work package
    let wp_data = serde_json::json!({
        "feature_id": feature_id,
        "title": format!("Initial work for {}", name),
        "state": "backlog",
        "priority": "high",
    });
    
    let wp_id = storage.create_work_package(&wp_data).await?;
    
    // Write meta.json to worktree
    let meta = serde_json::json!({
        "feature_id": feature_id,
        "slug": slug,
        "work_package_id": wp_id,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });
    
    vcs.write_artifact(slug, "meta.json", &meta.to_string()).await?;
    
    // Log audit entry
    let audit_entry = serde_json::json!({
        "feature_id": feature_id,
        "entry_type": "feature_created",
        "actor": "system",
        "details": serde_json::json!({
            "slug": slug,
            "worktree": worktree_path.to_string_lossy(),
        }),
    });
    
    storage.append_audit_entry(&audit_entry).await?;
    
    Ok(FeatureCreated {
        feature_id,
        work_package_id: wp_id,
        worktree_path,
    })
}

pub struct FeatureCreated {
    pub feature_id: i64,
    pub work_package_id: i64,
    pub worktree_path: std::path::PathBuf,
}
```

### J.2 Batch Operations

```rust
use pheno_plugin_core::{PluginRegistry, PluginResult};

/// Batch process multiple features.
pub async fn batch_create_features(
    registry: &PluginRegistry,
    features: Vec<NewFeatureRequest>,
) -> Vec<PluginResult<FeatureCreated>> {
    let vcs = registry.vcs("git").expect("git plugin");
    let storage = registry.storage("sqlite-storage").expect("sqlite plugin");
    
    // Process in parallel
    let futures: Vec<_> = features
        .into_iter()
        .map(|req| {
            let vcs = Arc::clone(&vcs);
            let storage = Arc::clone(&storage);
            
            tokio::spawn(async move {
                create_single_feature(&*vcs, &*storage, req).await
            })
        })
        .collect();
    
    let mut results = Vec::new();
    for fut in futures {
        match fut.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(Err(PluginError::Execution(e.to_string()))),
        }
    }
    
    results
}

async fn create_single_feature(
    vcs: &dyn VcsPlugin,
    storage: &dyn StoragePlugin,
    req: NewFeatureRequest,
) -> PluginResult<FeatureCreated> {
    // Implementation
    todo!()
}

pub struct NewFeatureRequest {
    pub slug: String,
    pub name: String,
}
```

### J.3 Error Recovery

```rust
use pheno_plugin_core::{PluginRegistry, PluginResult, PluginError};

/// Create feature with rollback on failure.
pub async fn create_feature_with_rollback(
    registry: &PluginRegistry,
    slug: &str,
    name: &str,
) -> PluginResult<FeatureCreated> {
    let vcs = registry.vcs("git").ok_or_else(|| {
        PluginError::NotFound("git".to_string())
    })?;
    let storage = registry.storage("sqlite-storage").ok_or_else(|| {
        PluginError::NotFound("sqlite-storage".to_string())
    })?;
    
    // Track for potential rollback
    let mut worktree_created = None;
    let mut feature_id = None;
    
    // Try to create worktree
    match vcs.create_worktree(slug, "WP001").await {
        Ok(path) => {
            worktree_created = Some(path.clone());
            
            // Try database operation
            let feature_data = serde_json::json!({
                "slug": slug,
                "name": name,
                "state": "draft",
            });
            
            match storage.create_feature(&feature_data).await {
                Ok(id) => {
                    feature_id = Some(id);
                    
                    Ok(FeatureCreated {
                        feature_id: id,
                        work_package_id: 0, // Would create WP
                        worktree_path: path,
                    })
                }
                Err(e) => {
                    // Rollback worktree creation
                    if let Some(ref path) = worktree_created {
                        let _ = vcs.cleanup_worktree(path).await;
                    }
                    Err(e)
                }
            }
        }
        Err(e) => Err(e),
    }
}
```

## Appendix K: Testing Patterns

### K.1 Integration Test Setup

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use pheno_plugin_core::PluginRegistry;
    use pheno_plugin_git::GitAdapter;
    use pheno_plugin_sqlite::SqliteStoragePlugin;
    use tempfile::TempDir;
    use git2::Repository;
    
    /// Test harness with real plugins.
    struct TestHarness {
        _temp_dir: TempDir,
        registry: PluginRegistry,
    }
    
    impl TestHarness {
        async fn new() -> PluginResult<Self> {
            let temp = TempDir::new().map_err(|e| {
                PluginError::Io(e)
            })?;
            
            // Initialize git repo
            Repository::init(temp.path()).map_err(|e| {
                PluginError::Initialization(e.to_string())
            })?;
            
            // Create registry with plugins
            let registry = PluginRegistry::new();
            
            let git = GitAdapter::new(temp.path())?;
            registry.register_vcs(Box::new(git))?;
            
            let sqlite = SqliteStoragePlugin::new(temp.path().join("test.db"))?;
            registry.register_storage(Box::new(sqlite))?;
            
            registry.finalize()?;
            
            Ok(Self {
                _temp_dir: temp,
                registry,
            })
        }
        
        fn registry(&self) -> &PluginRegistry {
            &self.registry
        }
    }
    
    #[tokio::test]
    async fn test_full_feature_lifecycle() -> PluginResult<()> {
        let harness = TestHarness::new().await?;
        
        // Create feature
        let vcs = harness.registry().vcs("git").unwrap();
        let storage = harness.registry().storage("sqlite-storage").unwrap();
        
        let path = vcs.create_worktree("test-feature", "WP001").await?;
        assert!(path.exists());
        
        let feature = serde_json::json!({
            "slug": "test-feature",
            "name": "Test Feature",
            "state": "draft",
        });
        let id = storage.create_feature(&feature).await?;
        assert!(id > 0);
        
        // Verify retrieval
        let retrieved = storage.get_feature_by_id(id).await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap()["slug"], "test-feature");
        
        // Cleanup
        vcs.cleanup_worktree(&path).await?;
        
        Ok(())
    }
}
```

### K.2 Property-Based Testing

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn feature_slug_roundtrip(slug in "[a-z0-9-]{1,50}") {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Create feature with random slug
                // Verify it can be retrieved
            });
        }
    }
}
```

## Appendix L: Performance Optimization Guide

### L.1 Profiling Plugin Operations

```rust
use tracing::instrument;
use std::time::Instant;

#[async_trait]
impl VcsPlugin for InstrumentedGitAdapter {
    #[instrument(skip(self))]
    async fn create_worktree(&self, slug: &str, wp_id: &str) -> PluginResult<PathBuf> {
        let start = Instant::now();
        
        let result = self.inner.create_worktree(slug, wp_id).await;
        
        let elapsed = start.elapsed();
        tracing::info!(
            operation = "create_worktree",
            slug = slug,
            wp_id = wp_id,
            duration_ms = elapsed.as_millis() as u64,
        );
        
        result
    }
}
```

### L.2 Connection Pooling for Storage

```rust
pub struct PooledSqlitePlugin {
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl PooledSqlitePlugin {
    pub fn new(db_path: impl AsRef<Path>) -> PluginResult<Self> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = r2d2::Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(|e| PluginError::Initialization(e.to_string()))?;
        
        Ok(Self { pool })
    }
}
```

---

*Specification Version: 1.0*
*Last Updated: 2026-04-04*
*Maintainers: PhenoPlugins Team*
