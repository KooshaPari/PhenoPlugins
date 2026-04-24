# PhenoPlugins Charter

## Mission

PhenoPlugins provides the foundational plugin architecture for the Phenotype ecosystem, enabling extensible, modular, and secure plugin systems across all Phenotype applications.

## Tenets (unless you know better ones)

These tenets guide PhenoPlugins' development. They are guidelines and goals.

1. **Security**:

Security is paramount in plugin architecture. PhenoPlugins implements defense in depth:
- Plugin isolation through trait boundaries
- No unsafe code in plugin interfaces
- Clear ownership and borrowing rules enforced by Rust
- Input validation at all plugin boundaries
- No dynamic code execution without explicit opt-in

2. **Minimalist**:

KISS - Keep it Simple Stupid. Plugin systems often suffer from over-engineering. PhenoPlugins maintains:
- Minimal trait surface area
- No unnecessary abstractions
- Direct, clear APIs
- Zero-cost abstractions where possible
- No hidden complexity

3. **Performance**:

Plugins should not impose significant overhead:
- Static dispatch by default
- Dynamic dispatch only where necessary
- Zero-copy where possible
- Minimal allocations in hot paths
- Async-first design for I/O bound operations

4. **Compatibility**:

Plugins must work across the Phenotype ecosystem:
- Stable trait interfaces with versioning
- Backward compatibility guarantees
- Clear deprecation paths
- Cross-platform support
- No ecosystem fragmentation

5. **Observability**:

Plugin behavior must be transparent:
- Health check interfaces
- Error propagation without information loss
- Metrics and tracing hooks
- Clear error messages
- Debuggable at all levels

## Scope

PhenoPlugins provides:

- Core plugin trait definitions
- Plugin registry and lifecycle management
- Reference implementations (Git, SQLite)
- Plugin discovery mechanisms
- Error handling patterns

PhenoPlugins explicitly does NOT provide:

- Plugin distribution/marketplace
- UI components for plugin management
- Remote plugin loading
- Plugin sandboxing beyond Rust's guarantees
- Non-Rust plugin interfaces (FFI out of scope)

## Contributions & Project Roles

All contributions must align with this charter.

Changes to core traits require:
1. ADR documenting the decision
2. Backward compatibility analysis
3. Migration guide if breaking
4. Approval from maintainers

## Versioning Policy

- MAJOR: Breaking changes to core traits
- MINOR: New plugin types, non-breaking additions
- PATCH: Bug fixes, documentation improvements

## License

MIT - See LICENSE file
