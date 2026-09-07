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

## Type System

Documentation for Phalcom's syntax-level type notation and serialized semantic type metadata.

- [Phalcom Type Syntax](type-system/phalcom-type-syntax.md) — VM-free parser and AST for type expressions, callable signatures, parameters, and generic constraints
- [Phalcom Type Metadata](type-system/phalcom-type-meta.md) — versioned semantic metadata graphs, stable identities, JSON transport, fingerprints, and validation invariants

## Benchmarks

Measurement corpora, VM baselines, mathematical correctness programs, Wren ports, and result schemas.

- [Phalcom Benchmarks](benchmarks/benchmarks.md) — benchmark families, measurement boundaries, result storage, and promotion discipline

## Common Utilities

Compiler-stage-agnostic source ranges and selector identity shared across the toolchain.

- [Phalcom Common](common/phalcom-common.md) — copyable source spans, exact/pattern selectors, and total runtime selector decoding

## Diagnostics

Structured diagnostic rendering, snippets, terminal styles, and report assembly.

- [Phalcom Diagnostics](diagnostics/phalcom-diagnostics.md) — severity/report models, Unicode-aware snippets, glyphs, color, and width policy

## Native Metadata and Declarations

Declarative native primitive contracts and the shared parsing/validation boundary.

- [Phalcom Native Metadata](native-meta/phalcom-native-meta.md) — primitive contracts, symbolic type specs, effects, lifecycle, and universe catalog
- [Phalcom Native Declarations](native-decl/phalcom-native-decl.md) — normalized primitive attributes, documentation capture, and declaration validation
- [Phalcom Native Macros](native-macros/phalcom-native-macros.md) — the `#[primitive]` procedural attribute and compile-time contract expansion

## Native Surface

The generated canonical native-member catalog and its deterministic generation/drift checks.

- [Phalcom Native Surface](native-surface/phalcom-native-surface.md) — catalog records, lookup indexes, structural fingerprints, and invariants
- [Phalcom Native Surface Generator](native-surface-gen/phalcom-native-surface-gen.md) — primitive census, metadata lowering, generated Rust output, and `--check` drift gate

## Scope

The `modules` crate is Phalcom's project and module-system layer. It defines logical project identity, module interfaces, symbol resolution, dependency graphs, and the manifest/runtime boundaries used by compiler and semantic stages. See [lib.rs](../../../phalcom-modules/src/lib.rs) for the full public API.

## Cross-links

- **Phalcom implementation**: See [phalcom-core](../../../phalcom-core/src) for the compiler, bytecode VM, and runtime
- **AST and parsing**: See [phalcom-ast](../../../phalcom-ast/src) for lexer, parser, and AST types
- **Diagnostics**: See [phalcom-diagnostics](../../../phalcom-diagnostics/src) for error rendering
- **Semantic analysis**: See [phalcom-semantic](../../../phalcom-semantic/src) for type inference and scope resolution
- **LSP support**: See [phalcom-lsp](../../../phalcom-lsp/src) for language server integration
