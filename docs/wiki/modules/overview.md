# Modules overview

The `modules` crate (`phalcom-modules/src/lib.rs`) is the project and module-graph layer in Phalcom. It doesn't implement the VM or parser; instead, it models how projects are discovered, modules are named, imports resolve, and interfaces and dependencies are tracked.

## Main responsibilities

- **Project discovery** — find and load `project.toml` manifests, resolve transitive dependencies
- **Module identity** — assign canonical identities to projects, modules, and source units
- **Source loading** — abstract source retrieval via `SourceProvider`, track module kinds
- **Manifest parsing** — validate project metadata, dependency specs, and entry points
- **Interface extraction** — surface declarations, exports, imports from parsed source
- **Import resolution** — resolve absolute/relative imports, validate package exposure boundaries
- **Linking** — bind module symbols, assign global binding IDs, track module-level layouts
- **Dependency graphs** — maintain reference, semantic, and runtime dependency graphs

The crate root re-exports the major types (see [lib.rs](../../../phalcom-modules/src/lib.rs)), making it the central coordination layer for module semantics.

## Core submodules

| Module | Responsibility | Key Types |
|--------|-----------------|----------|
| [identity.rs](../../../phalcom-modules/src/identity.rs) | Semantic identity for projects, modules, source units | `ResolvedProjectId`, `ModuleId`, `ModulePath`, `ModuleComponent` |
| [project.rs](../../../phalcom-modules/src/project.rs) | Project resolution and universe | `ResolvedProject`, `ProjectUniverse` |
| [manifest.rs](../../../phalcom-modules/src/manifest.rs) | Manifest parsing and validation | `ProjectManifest`, `ValidatedProjectManifest`, `DependencySpec` |
| [source.rs](../../../phalcom-modules/src/source.rs) | Source provider abstraction | `SourceProvider`, `SourceUnit`, `ModuleKind` |
| [resolver.rs](../../../phalcom-modules/src/resolver.rs) | Import resolution and validation | `ModuleResolver`, `ImportResolutionTrace` |
| [interface.rs](../../../phalcom-modules/src/interface.rs) | Interface extraction and linking targets | `UnlinkedModuleInterface`, `InterfaceBuilder` |
| [linker.rs](../../../phalcom-modules/src/linker.rs) | Symbol binding and module linking | `ModuleLinker`, `LinkedModule`, `SymbolId`, `GlobalBindingId` |
| [graph.rs](../../../phalcom-modules/src/graph.rs) | Dependency graph structures | `ReferenceGraph`, `SemanticGraph`, `RuntimeDependencyGraph` |
| [session.rs](../../../phalcom-modules/src/session.rs) | Workspace sessions and incremental updates | `WorkspaceModuleSession`, `WorkspaceSourceMutation` |

## Design philosophy

1. **Separation of concerns**: Reference edges, semantic edges, and runtime dependencies are kept distinct. A declaration cycle is valid input to a semantic fixed point; a runtime initialization cycle is not.

2. **Identity-driven**: Every module, project, and source unit has a canonical stable identity (`ModuleId`, `ResolvedProjectId`, `SourceId`), enabling caching and incremental analysis.

3. **Manifest-as-law**: Project structure comes from `project.toml`. Namespace, dependencies, entry points, and source root are determined by validated manifest, not file system convention.

4. **Package exposure**: External imports are validated hierarchically against `exposed_children` sets. Not every module path is legal just because a project exists.

5. **Traceability**: Resolution and linking operations produce traces (e.g., `ImportResolutionTrace`) for debugging and semantic tooling.

## Mental model

Think of the crate as the **namespace and dependency backbone** for Phalcom: it knows which project owns a module, which import roots are legal, how a module path maps to a file or package surface, and which dependencies must be tracked for interface checks versus runtime initialization.
