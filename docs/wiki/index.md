# Modules Wiki

Documentation for the `modules` crate (`phalcom-modules`), the project and module-graph layer in Phalcom.

## Core Topics

### Foundation
- [Overview](modules/overview.md) — architecture and main responsibilities
- [Identity and module paths](modules/identity.md) — semantic identities for projects and modules

### Project and manifest
- [Project structure](modules/project-structure.md) — projects, universes, and project resolution
- [Project manifests](modules/project-manifest.md) — `project.toml` structure, validation, and dependency specs

### Module resolution and visibility
- [Module resolution](modules/module-resolution.md) — import roots, path resolution, and exposure boundaries
- [Source providers](modules/source-providers.md) — loading source units and module kinds

### Interfaces and linking
- [Interfaces](modules/interfaces.md) — declarations, exports, imports, and interface extraction
- [Linking and symbols](modules/linking-symbols.md) — module binding, symbol resolution, and global symbol layout

### Dependencies and relationships
- [Dependency graphs](modules/dependency-graphs.md) — reference, semantic, and runtime graphs; phases and edge kinds
- [Sessions and incremental updates](modules/sessions.md) — workspace sessions and source mutations

## Scope

The `modules` crate is Phalcom's project and module-system layer. It defines logical project identity, module interfaces, symbol resolution, dependency graphs, and the manifest/runtime boundaries used by compiler and semantic stages. See [lib.rs](../../../phalcom-modules/src/lib.rs) for the full public API.

## Cross-links

- **Phalcom implementation**: See [phalcom-core](../../../phalcom-core/src) for the compiler, bytecode VM, and runtime
- **AST and parsing**: See [phalcom-ast](../../../phalcom-ast/src) for lexer, parser, and AST types
- **Diagnostics**: See [phalcom-diagnostics](../../../phalcom-diagnostics/src) for error rendering
- **Semantic analysis**: See [phalcom-semantic](../../../phalcom-semantic/src) for type inference and scope resolution
- **LSP support**: See [phalcom-lsp](../../../phalcom-lsp/src) for language server integration
