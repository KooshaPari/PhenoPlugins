# Product Requirements Document: PhenoPlugins

## Executive Summary

PhenoPlugins provides the foundational plugin architecture for the Phenotype ecosystem, enabling extensible, modular, and secure plugin systems across all Phenotype applications. It provides core plugin trait definitions, plugin registry and lifecycle management, reference implementations, and plugin discovery mechanisms.

The architecture prioritizes security through trait boundaries and Rust's ownership model, performance through zero-cost abstractions and async-first design, and compatibility through stable interfaces with versioning. PhenoPlugins enables applications to extend functionality without compromising core integrity.

---

## Problem Statement

### Current State Challenges

Extensible applications face significant architectural challenges:

1. **Security Risks**: Plugins often run with full application privileges, creating attack surfaces.

2. **API Instability**: Plugin interfaces change frequently, breaking existing plugins.

3. **Performance Overhead**: Plugin systems introduce measurable runtime costs.

4. **Complexity**: Plugin architectures are often over-engineered with unnecessary abstractions.

5. **Limited Discovery**: Finding and installing plugins is difficult without centralized mechanisms.

6. **Lifecycle Management**: Installing, updating, and removing plugins is error-prone.

7. **Testing Difficulty**: Plugins are hard to test in isolation from the host application.

### Impact Analysis

These challenges result in:
- Security vulnerabilities from plugin execution
- Maintenance burden from breaking API changes
- Performance degradation from plugin overhead
- Developer confusion from complex APIs
- Ecosystem fragmentation from poor discovery
- Operational issues from lifecycle problems

### Solution Vision

PhenoPlugins provides:
- Security through trait boundaries and no unsafe code
- Minimalist design with minimal trait surface area
- Performance through static dispatch and zero-copy patterns
- Compatibility through semantic versioning and deprecation paths
- Observability through health checks and error propagation
- Discovery through registry and metadata

---

## Target Users

### Primary Users

#### 1. Application Developers
- **Profile**: Building extensible applications
- **Goals**: Enable third-party extensions safely
- **Pain Points**:
  - Security concerns with plugins
  - Complex plugin APIs
  - Version compatibility issues
- **Success Criteria**: Secure, performant plugin system

#### 2. Plugin Authors
- **Profile**: Creating plugins for Phenotype applications
- **Goals**: Build and distribute plugins easily
- **Pain Points**:
  - Unclear APIs
  - Testing difficulties
  - Distribution challenges
- **Success Criteria**: Clear API, easy testing, simple distribution

#### 3. Platform Engineers
- **Profile**: Managing plugin infrastructure
- **Goals**: Control plugin lifecycle and security
- **Pain Points**:
  - Plugin governance
  - Version management
  - Security auditing
- **Success Criteria**: Controlled, observable plugin ecosystem

### Secondary Users

#### 4. Security Engineers
- **Profile**: Auditing plugin security
- **Needs**: Security scanning, policy enforcement
- **Usage**: Security reviews, compliance

#### 5. End Users
- **Profile**: Using plugin-enabled applications
- **Needs**: Reliable plugins, easy installation
- **Usage**: Plugin discovery, installation

### User Personas Summary

| Persona | Role | Primary Goal | Key Pain Point | Success Metric |
|---------|------|--------------|----------------|----------------|
| App Dev | Application Dev | Enable extensions | Security risks | Secure system |
| Plugin Author | Plugin Dev | Build plugins | Unclear APIs | Clear documentation |
| Platform Eng | Infrastructure | Control plugins | Governance gaps | Controlled ecosystem |
| Security | Security | Audit plugins | Unknown risks | Security scanning |
| End User | User | Use plugins | Installation difficulty | Easy installation |

---

## Functional Requirements

### FR-1: Plugin Core

#### FR-1.1: Plugin Trait
- The system SHALL define core Plugin trait
- The trait SHALL include metadata method
- The trait SHALL include initialization method
- The trait SHALL include shutdown method

#### FR-1.2: Type Safety
- The system SHALL use Rust type system for safety
- The system SHALL have no unsafe code in plugin interfaces
- The system SHALL enforce ownership and borrowing rules
- The system SHALL provide compile-time verification

#### FR-1.3: Lifecycle Management
- The system SHALL support plugin loading
- The system SHALL support plugin initialization
- The system SHALL support plugin activation
- The system SHALL support plugin deactivation
- The system SHALL support plugin unloading

#### FR-1.4: Error Handling
- The system SHALL provide structured error types
- The system SHALL support error propagation
- The system SHALL provide error context
- The system SHALL support error recovery

### FR-2: Registry

#### FR-2.1: Plugin Registration
- The system SHALL support static registration
- The system SHALL support dynamic registration
- The system SHALL support plugin factories
- The system SHALL provide registration validation

#### FR-2.2: Plugin Discovery
- The system SHALL provide plugin enumeration
- The system SHALL support filtering by type
- The system SHALL support metadata queries
- The system SHALL provide plugin search

#### FR-2.3: Dependency Management
- The system SHALL track plugin dependencies
- The system SHALL detect circular dependencies
- The system SHALL validate dependency versions
- The system SHALL provide dependency resolution

### FR-3: Reference Implementations

#### FR-3.1: Git Plugin
- The system SHALL provide Git VCS adapter
- The plugin SHALL support common Git operations
- The plugin SHALL provide status information
- The plugin SHALL support configuration

#### FR-3.2: SQLite Plugin
- The system SHALL provide SQLite storage adapter
- The plugin SHALL support SQL operations
- The plugin SHALL provide connection pooling
- The plugin SHALL support migrations

#### FR-3.3: Example Plugins
- The system SHALL provide example plugin implementations
- The examples SHALL demonstrate best practices
- The system SHALL include test examples
- The system SHALL provide documentation examples

### FR-4: Tooling

#### FR-4.1: Plugin SDK
- The system SHALL provide plugin development kit
- The SDK SHALL include templates
- The SDK SHALL provide testing utilities
- The SDK SHALL include documentation tools

#### FR-4.2: CLI Tool
- The system SHALL provide plugin CLI
- The CLI SHALL support plugin scaffolding
- The CLI SHALL support plugin validation
- The CLI SHALL support plugin packaging

#### FR-4.3: Testing Framework
- The system SHALL provide plugin testing utilities
- The system SHALL provide mock host interfaces
- The system SHALL support integration testing
- The system SHALL provide test fixtures

---

## Non-Functional Requirements

### NFR-1: Security

#### NFR-1.1: Isolation
- Plugins SHALL be isolated through trait boundaries
- No unsafe code SHALL be allowed in plugin interfaces
- Input validation SHALL be enforced at boundaries
- Dynamic code execution SHALL require explicit opt-in

#### NFR-1.2: Access Control
- Plugins SHALL have explicit capability grants
- The system SHALL enforce least privilege
- The system SHALL support capability revocation

### NFR-2: Performance

#### NFR-2.1: Overhead
- Plugin overhead SHALL be minimal (zero-cost abstractions)
- Static dispatch SHALL be preferred
- Dynamic dispatch SHALL be used only where necessary
- Memory allocations SHALL be minimized

#### NFR-2.2: Async Support
- The system SHALL be async-first
- Plugins SHALL support async operations
- The system SHALL use tokio for async runtime

### NFR-3: Compatibility

#### NFR-3.1: Versioning
- The system SHALL follow semantic versioning
- Breaking changes SHALL require major version bumps
- Deprecation SHALL include clear paths
- The system SHALL support backward compatibility

#### NFR-3.2: Stability
- Plugin APIs SHALL be stable
- The system SHALL maintain compatibility guarantees
- The system SHALL provide migration guides

---

## User Stories

### US-1: Implementing Plugin System

**As an** application developer,  
**I want to** add plugin support to my application,  
**So that** users can extend functionality safely.

**Acceptance Criteria**:
- Given the core crate, when integrated, then plugin system is available
- Given a plugin trait, when implemented, then plugins can be loaded
- Given a plugin, when loaded, then it runs safely in isolation

### US-2: Creating a Plugin

**As a** plugin author,  
**I want to** create a plugin following clear patterns,  
**So that** it works reliably with host applications.

**Acceptance Criteria**:
- Given the SDK, when used, then plugin scaffold is generated
- Given documentation, when followed, then plugin compiles
- Given tests, when run, then plugin passes validation

### US-3: Managing Plugins

**As a** platform engineer,  
**I want to** control which plugins are installed and running,  
**So that** I can ensure security and stability.

**Acceptance Criteria**:
- Given the registry, when viewed, then all plugins are listed
- Given a plugin, when disabled, then it stops running
- Given logs, when reviewed, then plugin activity is visible

### US-4: Testing Plugins

**As a** plugin author,  
**I want to** test my plugin in isolation,  
**So that** I can ensure quality before distribution.

**Acceptance Criteria**:
- Given test utilities, when used, then plugin can be tested
- Given mocks, when configured, then host can be simulated
- Given test suite, when run, then coverage is reported

### US-5: Discovering Plugins

**As an** end user,  
**I want to** find and install plugins easily,  
**So that** I can extend application functionality.

**Acceptance Criteria**:
- Given the registry, when browsed, then plugins are discoverable
- Given a plugin, when selected, then installation is simple
- Given an installed plugin, when enabled, then it works immediately

---

## Features

### Feature 1: Plugin Core

**Description**: Core plugin trait definitions and lifecycle management.

**Components**:
- Plugin trait
- Lifecycle manager
- Error types
- Registry

**User Value**: Type-safe plugins; clear interfaces; lifecycle control.

**Dependencies**: None (foundational)

**Priority**: P0 (Critical)

### Feature 2: Security Framework

**Description**: Security through type system and capability management.

**Components**:
- Capability system
- Input validation
- Sandbox abstractions
- Security policy

**User Value**: Safe execution; reduced attack surface; confidence.

**Dependencies**: Plugin Core

**Priority**: P0 (Critical)

### Feature 3: Reference Plugins

**Description**: Git and SQLite adapter plugins as examples.

**Components**:
- Git plugin
- SQLite plugin
- Example plugins
- Documentation

**User Value**: Working examples; best practices; learning resource.

**Dependencies**: Plugin Core

**Priority**: P1 (High)

### Feature 4: Plugin SDK

**Description**: Development kit for plugin authors.

**Components**:
- Templates
- Testing utilities
- Documentation generator
- CLI tool

**User Value**: Easy development; faster iteration; quality.

**Dependencies**: Plugin Core

**Priority**: P1 (High)

### Feature 5: Registry System

**Description**: Plugin discovery and management.

**Components**:
- Registry service
- Metadata management
- Dependency resolver
- Version manager

**User Value**: Discovery; dependency management; consistency.

**Dependencies**: Plugin Core

**Priority**: P2 (Medium)

---

## Metrics & KPIs

### Technical Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Overhead | <5% | Benchmarks |
| Load Time | <100ms | Benchmarks |
| API Stability | 100% backward | Testing |

### Adoption Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Plugins | 20+ | Registry |
| Applications | 5+ | Usage |
| Downloads | 1000+ | Crates.io |

### Quality Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Unsafe Code | 0 | Audit |
| Test Coverage | >80% | Coverage |
| Satisfaction | >4.0/5 | Survey |

---

## Release Criteria

### MVP Release (Month 2)

**Must Have**:
- [ ] Core Plugin trait
- [ ] Basic registry
- [ ] Lifecycle management
- [ ] Error handling
- [ ] Documentation

**Exit Criteria**:
- 0 unsafe code
- Core compiles and passes tests
- Basic example works

### Beta Release (Month 4)

**Must Have**:
- [ ] Git plugin
- [ ] SQLite plugin
- [ ] Security framework
- [ ] Plugin SDK
- [ ] CLI tool

**Exit Criteria**:
- 2 reference plugins complete
- SDK supports plugin development
- 3+ applications using

### GA Release (Month 6)

**Must Have**:
- [ ] Registry system
- [ ] Dependency management
- [ ] Testing framework
- [ ] Complete documentation
- [ ] 5+ example plugins

**Exit Criteria**:
- 10+ plugins in ecosystem
- User satisfaction >4.0/5
- Security audit passed

---

## Appendix

### A. Glossary

- **Plugin**: Extension that adds functionality to host application
- **Trait**: Rust interface definition
- **Registry**: Repository of available plugins
- **Capability**: Permission granted to plugin
- **WASM**: WebAssembly (potential future sandbox)

### B. References

- Rust Traits: https://doc.rust-lang.org/book/ch10-02-traits.html
- Plugin Architecture: https://en.wikipedia.org/wiki/Plugin_architecture
- WASMtime: https://wasmtime.dev/

### C. Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-05 | Maintainer | Initial PRD creation |

---

## Additional Sections

### Plugin Trait System

#### Core Plugin Interface

The Plugin trait defines the contract between host and extension:

```rust
/// The core trait that all plugins must implement
pub trait Plugin: Send + Sync + 'static {
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Initialize the plugin with configuration
    fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError>;
    
    /// Start the plugin
    fn start(&mut self) -> Result<(), PluginError>;
    
    /// Stop the plugin gracefully
    fn stop(&mut self) -> Result<(), PluginError>;
    
    /// Health check
    fn health(&self) -> HealthStatus;
    
    /// Handle events from the host
    fn on_event(&mut self, event: HostEvent) -> Result<(), PluginError>;
}

/// Plugin metadata
pub struct PluginMetadata {
    pub name: String,
    pub version: Version,
    pub description: String,
    pub author: String,
    pub license: String,
    pub capabilities: Vec<Capability>,
    pub api_versions: Vec<ApiVersion>,
}
```

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Plugin crashing host process | Low | Critical | Process isolation, panic catching, health monitoring |
| Plugin consuming excessive resources | Medium | High | Resource limits, cgroup constraints, monitoring |
| Incompatible plugin versions | Medium | Medium | Version checking, dependency resolution, rollback |
| Plugin API breaking changes | Medium | High | Semantic versioning, deprecation periods, migration guides |
| Malicious plugin behavior | Low | Critical | Code review, sandboxing, limited capabilities |
| Plugin loading latency | Medium | Medium | Lazy loading, caching, preloading |
| Cross-platform compatibility issues | Medium | Medium | CI testing on all platforms, abstraction layer |
| Plugin state corruption | Low | High | State validation, snapshots, recovery mechanisms |

### Capabilities System

#### Capability-Based Security

Plugins declare capabilities they require; host grants explicitly:

```rust
/// Capabilities that plugins can request
pub enum Capability {
    /// Read from filesystem
    FileRead { paths: Vec<PathPattern> },
    /// Write to filesystem
    FileWrite { paths: Vec<PathPattern> },
    /// Network access
    Network { hosts: Vec<HostPattern> },
    /// Execute subprocesses
    Exec { allowed_commands: Vec<String> },
    /// Access environment variables
    Env { allowed_vars: Vec<String> },
    /// Access plugin configuration
    Config { keys: Vec<String> },
    /// Emit custom events
    EventEmit { event_types: Vec<String> },
    /// Access to specific host APIs
    HostApi { endpoints: Vec<String> },
}
```

**Capability Granting**:
- Default-deny: No capabilities granted by default
- Explicit grant: Host must explicitly grant each capability
- Pattern matching: Paths, hosts, patterns with wildcards
- Audit logging: All capability checks logged

### Plugin Testing Strategy

#### Test Isolation

Plugins can be tested in isolation with mock host:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pheno_plugin::testing::*;
    
    #[test]
    fn test_plugin_initialization() {
        let mut plugin = MyPlugin::new();
        let mock_host = MockHost::new();
        
        let config = PluginConfig::new()
            .set("key", "value");
        
        plugin.initialize(config).unwrap();
        assert!(plugin.health().is_healthy());
    }
    
    #[test]
    fn test_plugin_handles_events() {
        let mut plugin = MyPlugin::new();
        let event = HostEvent::DataReceived {
            data: vec![1, 2, 3],
        };
        
        plugin.on_event(event).unwrap();
        // Assert expected behavior
    }
}
```

#### Integration Testing

Full integration tests with real host:
- Test plugin loading
- Test host-plugin communication
- Test resource limits enforcement
- Test failure recovery

### Performance Benchmarks

#### Target Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Plugin load time | <100ms | Cold start |
| Plugin call latency | <1ms | IPC overhead |
| Memory overhead | <10MB | Per plugin |
| Concurrent plugins | 100+ | Load testing |
| Zero-cost abstraction | Verified | Assembly analysis |

*This document is a living specification. Updates require Maintainer approval and version increment.*

### Plugin Development Workflow

#### Development Lifecycle

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│  Plan   │───▶│ Develop │───▶│  Test   │───▶│ Package │───▶│ Publish │
│         │    │         │    │         │    │         │    │         │
└─────────┘    └─────────┘    └─────────┘    └─────────┘    └─────────┘
     ▲                                                         │
     └─────────────────────────────────────────────────────────┘
                          Feedback/Updates
```

#### Local Development

**Hot Reload**:
- File watcher for changes
- Automatic recompilation
- State preservation where possible
- Error reporting in real-time

**Debugging**:
- Attach debugger to plugin process
- Breakpoint support
- Variable inspection
- Stack trace analysis

#### Testing Strategy

**Unit Tests**:
```rust
#[test]
fn test_plugin_metadata() {
    let plugin = MyPlugin::new();
    let meta = plugin.metadata();
    assert_eq!(meta.name, "my-plugin");
}
```

**Integration Tests**:
```rust
#[tokio::test]
async fn test_plugin_with_host() {
    let host = TestHost::new();
    let plugin = load_plugin("my-plugin").await;
    let result = plugin.execute("test-action").await;
    assert!(result.is_ok());
}
```

### Plugin Security Scanning

#### Static Analysis

- Cargo audit for known vulnerabilities
- Clippy for code quality
- Custom security rules
- License compliance

#### Dynamic Analysis

- Fuzzing of plugin inputs
- Resource limit testing
- Crash recovery testing
- Memory leak detection


### Plugin Marketplace

#### Marketplace Features

**Discovery**:
- Category browsing
- Search with filters
- Featured plugins
- Trending list
- Personal recommendations

**Evaluation**:
- Star ratings
- Download counts
- Version history
- Dependency info
- Security score

**Installation**:
- One-click install
- Dependency resolution
- Conflict detection
- Rollback capability

#### Plugin Monetization

**Models**:
- Free and open source
- Freemium (basic free, pro paid)
- Subscription-based
- Enterprise licensing

**Payment Integration**:
- Stripe for payments
- Invoice for enterprise
- Revenue sharing (70/30 split)

### Plugin Analytics

#### Usage Metrics

**Collected Data**:
- Download counts
- Active installations
- Feature usage
- Error rates
- Performance metrics

**Privacy**:
- Opt-in only
- Anonymized
- No PII
- Clear data policy

**Insights for Authors**:
- Adoption trends
- Geographic distribution
- Version adoption
- User retention

#### Health Monitoring

- Crash reporting
- Performance regression detection
- Security vulnerability alerts
- Deprecation notices

---

## Additional Sections

### Plugin Architecture Deep Dive

#### Host-Plugin Communication

**Communication Patterns**:

```
Synchronous Call
┌─────────┐      Request       ┌─────────┐
│  Host   │ ─────────────────▶ │ Plugin  │
│         │                     │         │
│         │ ◀───────────────── │         │
│         │      Response       │         │
└─────────┘                     └─────────┘

Asynchronous Event
┌─────────┐      Event         ┌─────────┐
│  Host   │ ─────────────────▶ │ Plugin  │
│         │                     │         │
│         │ ◀───────────────── │         │
│         │    Ack/Nack         │         │
└─────────┘                     └─────────┘
```

**Serialization Formats**:
- JSON (default, human-readable)
- MessagePack (compact, fast)
- Protocol Buffers (schema-enforced)
- CBOR (IETF standard)

#### Resource Management

**Memory Limits**:
```rust
pub struct ResourceLimits {
    pub max_memory_mb: usize,
    pub max_cpu_percent: f32,
    pub max_file_descriptors: u32,
    pub max_network_connections: u32,
    pub request_timeout_ms: u64,
}
```

**Enforcement**:
- Soft limits (warnings)
- Hard limits (termination)
- Graceful degradation
- Resource request/response

### Plugin Distribution

#### Registry Architecture

**Centralized Registry**:
- Single source of truth
- Verified publishers
- Security scanning
- Performance metrics

**Decentralized Options**:
- Git-based distribution
- Private registries
- Air-gapped support
- Offline capability

#### Version Resolution

**Dependency Solving**:
```
Plugin A v1.0.0 requires:
- Plugin Core >=2.0.0, <3.0.0
- Git Plugin ^1.2.0

Plugin B v2.0.0 requires:
- Plugin Core ^2.1.0

Resolution:
- Plugin Core 2.5.0 (satisfies both)
- Git Plugin 1.3.0
```

**Conflict Resolution**:
- Semantic versioning precedence
- Manual override capability
- Dependency tree inspection
- Version pinning

### Security Architecture

#### Threat Model

**Attack Vectors**:
1. Malicious plugin code
2. Plugin privilege escalation
3. Resource exhaustion
4. Data exfiltration
5. Supply chain attacks

**Mitigations**:
- Code signing
- Capability-based sandbox
- Resource quotas
- Network policies
- Audit logging

#### Code Signing

**Signature Verification**:
```rust
pub struct SignedPlugin {
    pub plugin: Vec<u8>,
    pub signature: Vec<u8>,
    pub certificate_chain: Vec<Certificate>,
}

impl SignedPlugin {
    pub fn verify(&self, trust_store: &TrustStore) -> Result<(), VerifyError> {
        // Verify certificate chain
        // Check signature
        // Validate timestamp
    }
}
```

**Key Management**:
- HSM for signing keys
- Key rotation schedule
- Revocation lists
- Emergency key procedures

### Performance Engineering

#### Benchmarking Framework

**Metrics Collection**:
- Load time (cold/warm)
- Memory footprint
- CPU utilization
- Throughput (ops/sec)
- Latency percentiles

**Benchmark Types**:
- Microbenchmarks (individual operations)
- Macrobenchmarks (workflows)
- Load tests (concurrent plugins)
- Stress tests (resource exhaustion)

#### Optimization Strategies

**Lazy Loading**:
- On-demand initialization
- Background preloading
- Priority loading queue
- Memory-mapped files

**Caching**:
- Compiled plugin cache
- Metadata cache
- Configuration cache
- Result caching

### Developer Experience

#### Plugin Templates

**Template Types**:
- Minimal (hello world)
- Standard (full lifecycle)
- Web service (HTTP handlers)
- Database (storage adapter)
- ML/AI (model serving)

**Template Features**:
- Pre-configured build setup
- Testing scaffolding
- CI/CD workflows
- Documentation structure

#### Debugging Tools

**Runtime Inspection**:
```bash
# Attach to running plugin
pheno-plugin debug --plugin=my-plugin --attach

# Set breakpoints
(pheno) break my-plugin::process_data
(pheno) continue

# Inspect state
(pheno) inspect state
(pheno) trace calls
```

**Logging Integration**:
- Structured logging
- Log levels per plugin
- Log aggregation
- Correlation IDs

### Enterprise Features

#### Multi-Tenancy

**Tenant Isolation**:
- Separate plugin instances per tenant
- Resource quotas per tenant
- Configuration isolation
- Data segregation

**Shared Services**:
- Common plugin pool
- Warm plugin cache
- Tenant-aware routing
- Usage tracking

#### Compliance

**Audit Requirements**:
- All plugin operations logged
- Immutable audit trail
- Retention policies
- Compliance reporting

**Data Protection**:
- GDPR compliance
- Data residency
- Encryption at rest
- Encryption in transit

### Integration Patterns

#### Event-Driven Integration

**Event Bus**:
```rust
pub trait EventBus {
    fn publish(&self, event: Event) -> Result<(), EventError>;
    fn subscribe(&self, filter: EventFilter) -> EventStream;
    fn unsubscribe(&self, subscription_id: SubscriptionId);
}
```

**Event Types**:
- System events (startup, shutdown)
- Data events (created, updated, deleted)
- Custom events (domain-specific)
- Lifecycle events (plugin loaded, unloaded)

#### Service Mesh Integration

**Sidecar Pattern**:
- Plugin as sidecar container
- Service mesh proxy
- mTLS termination
- Traffic management

**API Gateway**:
- Plugin endpoints exposed
- Rate limiting
- Authentication
- Request transformation


