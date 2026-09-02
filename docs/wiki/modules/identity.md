# Identity and module paths

Semantic identity is the foundation of Phalcom's module system. Every project, module, and source unit has a canonical stable identity that enables caching, incremental analysis, and cross-tool communication. See [identity.rs](../../../phalcom-modules/src/identity.rs).

## Project identities

Projects come in three varieties:

```rust
pub enum ProjectIdentity {
    Universe,                    // The built-in universe scope
    Resolved(ResolvedProjectId), // A persistent project with project.toml
    Synthetic(SyntheticProjectId), // An inline/standalone execution context
}
```

### ResolvedProjectId

```rust
pub struct ResolvedProjectId(NonZeroU32);
```

Graph-node identity for a persistent user project. Non-zero ensures no collisions with unresolved or synthetic projects. Allocated sequentially as projects are resolved during project discovery.

### SyntheticProjectId

```rust
pub struct SyntheticProjectId(u64);
```

Process-local identity for inline/standalone code (e.g., REPL input, snippets). Allocated from a global monotonic counter per-process, never reset, so inline modules from different compiler instances never collide within a single process.

## Module paths and components

Modules are identified by `ModulePath`:

```rust
pub struct ModulePath {
    pub segments: Vec<ModuleComponent>,
}
```

Each segment is a `ModuleComponent`, a normalized snake_case identifier:

```rust
pub struct ModuleComponent {
    name: String,
}

impl ModuleComponent {
    pub fn from_identifier(ident: &str) -> Result<Self, InvalidModuleNameError>
    // Requires: lowercase alphanumeric + underscores, no leading digits
}
```

Example paths:
- `foo` — top-level module
- `foo::bar::baz` — nested module hierarchy
- `core::collections::map` — nested in the universe

## Module identities

A `ModuleId` ties together project and module path:

```rust
pub struct ModuleId {
    pub project: ProjectIdentity,
    pub path: ModulePath,
}
```

This is sufficient to identify any module unambiguously within a workspace or REPL session.

### SourceId

For file-based modules, `SourceId` represents a concrete source location:

```rust
pub struct SourceId {
    pub project: ProjectIdentity,
    pub path: PathBuf,
}
```

Multiple modules (e.g., concatenated snippets) can share a `SourceId`; multiple source locations can contribute to one `ModuleId`.

## Import roots

An import root is the first component of an absolute import path. Special roots are:

- `universe` — the built-in universe scope
- Project namespace (e.g., `my_project`) — self-import within a project
- Dependency aliases (e.g., `other_project`) — import from a resolved dependency

Absolute imports are resolved through an `ImportRootTarget`:

```rust
pub enum ImportRootTarget {
    Universe,
    Project(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}
```

Each resolved project precomputes its import root table (`BTreeMap<ModuleComponent, (ImportRootTarget, bool)>`) during project resolution, including self, dependencies, and the universe.

## Source location

For diagnostics and LSP:

```rust
pub struct SourceLocation {
    pub module: ModuleId,
    pub range: SourceRange,
}
```

This pairs a module identity with a source range from `phalcom-common`.

## Stable keys for persistence

For serialization and caching:

- `StableProjectKey` — persistent identity string for a resolved project
- `StableModuleKey` — persistent identity string for a module

These are used by incremental analysis to map identities across sessions.

## Cross-reference

- See [project-structure.md](project-structure.md) for how projects are discovered and loaded
- See [module-resolution.md](module-resolution.md) for how imports are resolved to module IDs
- See [linking-symbols.md](linking-symbols.md) for how module IDs map to global symbol bindings
