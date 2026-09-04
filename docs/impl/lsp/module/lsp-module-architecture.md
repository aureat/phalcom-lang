# Phalcom LSP Module Architecture Specification

**Status:** Authoritative target architecture  
**Scope:** `phalcom-modules`, `phalcom-semantic`, `phalcom-lsp`, compiler/LSP parity, incremental module analysis  
**Primary audience:** compiler, semantic-analysis, module-system, and LSP implementers  
**Normative language:** **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are used in the RFC sense.  
**Document role:** This document is the authoritative specification for module ownership, topology, linking, semantic projection, source provenance, incremental invalidation, caching, and LSP module behavior.

---

## 1. Purpose

Phalcom requires one coherent module architecture shared by the compiler, semantic analyzer, LSP, and runtime-facing compilation pipeline.

The module system is not merely a filesystem resolver. It defines:

- source ownership;
- package identity;
- module identity;
- logical module topology;
- import path resolution;
- package exposure;
- module public interfaces;
- linked import/export identity;
- cross-module semantic identity;
- source provenance;
- incremental invalidation boundaries;
- editor navigation;
- module diagnostics.

The LSP MUST NOT implement a parallel module system. It consumes immutable compiler-owned semantic products and adapts them to LSP protocol types.

The target pipeline is:

```text
filesystem / overlays / workspace discovery
                │
                ▼
        Source Ownership Classifier
                │
                ▼
          Module Topology
                │
                ▼
      WorkspaceModuleSession
                │
       ┌────────┴────────┐
       ▼                 ▼
 Interface Products   Import Resolution
       │                 │
       └────────┬────────┘
                ▼
             Linker
                │
                ▼
        Linked Module World
                │
                ▼
     SemanticWorkspaceSession
                │
       ┌────────┴────────┐
       ▼                 ▼
 Semantic Products   SourceSemanticIndex
       │                 │
       └────────┬────────┘
                ▼
        SemanticSnapshot
                │
                ▼
       EditorSemanticQuery
                │
                ▼
           phalcom-lsp
```

The central architectural rule is:

> **Filesystem layout provides source candidates. It does not directly grant semantic accessibility. `package.ph` establishes package structure, `expose` establishes public child-path accessibility, `export` establishes public symbol accessibility, the linker establishes canonical cross-module identity, and the semantic layer projects that identity into language meaning and source provenance. The LSP only presents those products.**

---

## 2. Goals

This architecture MUST provide the following properties.

### 2.1 Single semantic world

For one workspace generation, every participating source has exactly one canonical module identity and exactly one ownership context.

Compiler analysis and LSP analysis of the same source universe MUST agree on:

- `ModuleId`;
- package/module kind;
- import targets;
- linked export targets;
- declaration identities;
- module-global identities;
- package exposure;
- source definitions.

### 2.2 Explicit packages

A directory becomes a Phalcom package only through `package.ph`.

A directory MUST NOT become a package because:

- it is an editor workspace folder;
- it contains multiple `.ph` files;
- it contains `main.ph`;
- it is the parent directory of a directly opened source file;
- it was used as a convenient synthetic resolution root.

### 2.3 Private-by-default symbols

A declaration existing in a module MUST NOT make it externally accessible.

Cross-module symbol accessibility is controlled through the linked public export interface.

### 2.4 Compiler-owned editor semantics

All editor-facing module meaning MUST be available through `SemanticSnapshot` / `EditorSemanticQuery`.

LSP request handlers MUST NOT perform module resolution, parsing, linking, filesystem discovery, or semantic inference.

### 2.5 Incremental module processing

Ordinary body edits SHOULD cost approximately:

```text
changed source
+
affected semantic queries
```

and MUST NOT require broad module re-resolution or relinking when the module interface has not changed.

### 2.6 Error-tolerant workspaces

Invalid source MUST produce current diagnostics and partial current semantic products.

A source-authored module error MUST NOT be represented as analysis cancellation.

---

## 3. Non-goals

This specification does not require:

- a new package manager;
- separate compilation artifacts;
- incremental parsing;
- a new persistent-map collection library;
- parallel module linking;
- custom arena allocation for module records;
- replacement of `BTreeMap` with hash maps;
- LSP-side semantic caches;
- runtime reflection redesign except where identity parity is required.

These may be addressed separately after the architecture in this document is implemented and measured.

---

# Part I — Ownership and Identity

## 4. Source ownership

### 4.1 `EntryOwnership`

`EntryOwnership` is the authoritative classification of a source execution/analysis context.

The conceptual model is:

```rust
pub enum EntryOwnership {
    ProjectOwned {
        project: ResolvedProjectId,
    },

    StandalonePackageOwned {
        package_root: CanonicalPath,
    },

    StandaloneModule {
        file: CanonicalPath,
    },

    Inline {
        synthetic: SyntheticProjectId,
    },
}
```

Universe-owned sources are provider-owned and do not require filesystem entry classification.

The exact Rust representation MAY vary, but these semantic categories MUST remain distinct.

### 4.2 Ownership is determined once

Ownership MUST be determined before module resolution.

No later subsystem may reinterpret a source as a different ownership category merely to reuse another resolution implementation.

In particular, this is forbidden:

```text
unowned source file
    ↓
take source.parent()
    ↓
manufacture synthetic project/package root
    ↓
allow sibling resolution
```

### 4.3 Ownership classification order

For a directly selected filesystem `.ph` file, classification is:

```text
1. enclosing persistent Project?
       yes → ProjectOwned

2. otherwise inside a valid standalone Package hierarchy?
       yes → StandalonePackageOwned

3. otherwise
       → StandaloneModule
```

An inline/REPL source is explicitly `Inline`.

### 4.4 Workspace roots are discovery roots only

An editor workspace root grants tooling permission to discover candidate source files.

It MUST NOT create language package identity.

Therefore:

```text
LSP WorkspaceScanState
    → discovers candidate .ph files
    → ownership classifier determines semantic ownership
```

The scanner MUST NOT assign package semantics.

---

## 5. Package identity

### 5.1 `package.ph` is authoritative

`package.ph` is the only filesystem marker that creates a package namespace node.

The following are not package markers:

- `main.ph`;
- directory existence;
- editor workspace membership;
- sibling source files;
- project source-root directory existence alone.

### 5.2 Executability and package identity are independent

A standalone package directory:

```text
pkg/
└── package.ph
```

is a valid package even if it is not directly executable.

A package is directly executable through the default package-entry convention only when the required executable entry exists, for example:

```text
pkg/
├── package.ph
└── main.ph
```

`main.ph` selects default execution. It does not create package identity.

### 5.3 Nested package continuity

Package ancestry is logical package ancestry, not arbitrary filesystem ancestry.

A directly selected source inside a standalone package hierarchy MUST retain that package ownership.

Example:

```text
foo/
├── package.ph
├── util.ph
└── tools/
    ├── package.ph
    └── run.ph
```

`run.ph` is analyzed as a source inside the standalone package rooted at `foo`, with nested package `tools`.

A directory without `package.ph` does not become a nested package merely because it contains `.ph` files.

### 5.4 Standalone module isolation

Given:

```text
scratch/
├── main.ph
└── helper.ph
```

where no owning project or package exists, `main.ph` is a standalone module.

A relative sibling import from `main.ph` MUST be rejected.

The existence of `helper.ph` is not sufficient authority to import it.

---

## 6. Module identity

### 6.1 Canonical identity

`ModuleId` remains the canonical toolchain identity.

Conceptually:

```rust
pub struct ModuleId {
    pub project: ProjectIdentity,
    pub path: ModulePath,
}
```

`ProjectIdentity` MUST continue to distinguish at least:

```text
Universe
Resolved(...)
Synthetic(...)
```

These identity domains MUST NOT alias.

### 6.2 One physical source, one module identity

Within one module-session generation, a canonical physical source MUST map to at most one `ModuleId`.

The reverse identity relation MUST be maintained:

```text
ModuleId ↔ SourceId
```

Conflicting insertion MUST be diagnosed.

### 6.3 Stable identity under body edits

Changing ordinary source content MUST NOT change `ModuleId`.

`ModuleId` changes only when identity-defining inputs change, such as:

- ownership category;
- project identity;
- logical module path;
- package topology;
- source move/rename;
- project source-root configuration.

---

# Part II — Topology

## 7. Module topology

The module system MUST represent namespace topology explicitly.

Topology is not equivalent to the filesystem tree.

A canonical topology product contains the information required to answer:

- which modules/packages exist;
- which source owns each module;
- which package contains which child;
- which paths are internally resolvable;
- which child paths are externally exposed;
- which absolute import roots exist;
- which project/package context owns a source.

A conceptual representation is:

```rust
pub struct ModuleTopology {
    pub ownership: OwnershipIndex,
    pub nodes: BTreeMap<ModuleId, TopologyNode>,
    pub children: BTreeMap<ModuleId, BTreeMap<ModuleComponent, ModuleId>>,
    pub exposed_children: BTreeMap<ModuleId, BTreeMap<ModuleComponent, ModuleId>>,
    pub import_roots: BTreeMap<ImportRootName, ImportRootTarget>,
}
```

The exact shape MAY differ.

### 7.1 Topology nodes

A topology node includes at least:

```rust
pub struct TopologyNode {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
    pub ownership: OwnershipIdentity,
}
```

`ModuleKind` MUST distinguish ordinary modules from package modules.

### 7.2 Internal child edges

Package containment defines internal namespace structure.

A valid child module/package may be internally resolvable even when it is not externally exposed.

### 7.3 Exposure edges

`expose .child` creates a public path edge from a package to an immediate child.

It does not:

- import the child;
- initialize the child;
- export the child object;
- create a local variable.

Exposure is path visibility.

---

## 8. `TopologyFingerprint`

### 8.1 Definition

`TopologyFingerprint` is a semantic fingerprint of the resolvable namespace topology relevant to module-path resolution.

It is **not**:

- a hash of all source text;
- a hash of all files beneath a workspace root;
- the workspace semantic revision;
- an LSP document version;
- merely a filesystem mtime generation.

Conceptually:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TopologyFingerprint(...);
```

### 8.2 Inputs

The fingerprint MUST change when any semantic input capable of changing module-path resolution changes.

This includes:

- project/package ownership boundaries;
- project source roots;
- project namespace;
- dependency/import root mapping;
- package presence/removal;
- module presence/removal;
- module/package kind;
- logical module path;
- canonical physical/logical mapping where it affects resolution;
- package exposure edges;
- source move/rename that changes logical identity.

### 8.3 Excluded inputs

The fingerprint MUST remain stable under edits that cannot change module-path resolution, including:

- method bodies;
- local expressions;
- comments;
- formatting;
- declaration implementation bodies;
- ordinary symbol exports when path exposure is unchanged.

`export Foo` affects symbol linking, not namespace path topology.

`expose .foo` affects topology and therefore changes the fingerprint.

### 8.4 Global epoch vs semantic fingerprint

Implementations MAY maintain a monotonic `TopologyEpoch` for cheap invalidation signaling.

The two concepts serve different purposes:

```text
TopologyEpoch
    → “something topology-relevant changed”

TopologyFingerprint
    → “this topology product's semantic meaning is X”
```

A global epoch is not sufficient as the final cache reuse key because it invalidates unrelated resolution contexts too broadly.

### 8.5 Fingerprint stability

Fingerprints MUST be deterministic for semantically equivalent topology products.

They SHOULD be built from canonical semantic structures rather than raw filesystem paths or source text.

---

## 9. Ownership index

Ownership lookup is a module-infrastructure query, not an LSP query.

A persistent workspace SHOULD retain an `OwnershipIndex`.

Conceptually:

```rust
pub struct OwnershipIndex {
    pub sources: BTreeMap<SourceId, EntryOwnership>,
    pub directories: BTreeMap<CanonicalPath, OwnershipBoundary>,
}
```

Directory-level caching is encouraged because many sources share the same ownership boundary.

### 9.1 Ownership cache invalidation

Ownership cache entries are invalidated by topology changes such as:

- `project.toml` addition/removal/change;
- `package.ph` addition/removal;
- workspace root changes;
- source moves across ownership boundaries.

Ordinary source body edits MUST NOT invalidate ownership.

### 9.2 Live overlays

The authoritative workspace topology SHOULD operate over:

```text
disk state + active source overlays
```

This permits an unsaved `package.ph` to establish live package topology.

If live topology-changing marker overlays are temporarily unsupported, that limitation MUST be explicit and isolated at the source-provider boundary; LSP request code MUST NOT compensate through heuristic resolution.

---

# Part III — Source Providers and Caching

## 10. Source-provider responsibilities

The filesystem/source-provider layer owns:

- canonical source lookup;
- physical/logical naming validation;
- source-root confinement;
- source reading;
- generation-aware filesystem memoization;
- `SourceId ↔ ModuleId` physical identity safety.

It does not own:

- symbol exports;
- semantic declarations;
- editor navigation policy.

---

## 11. Filesystem caches

A persistent workspace SHOULD distinguish three cache classes.

### 11.1 Location/topology cache

```text
(Project/Package Context, ModulePath)
    → SourceUnit / resolution failure
```

This cache is invalidated by topology changes.

It MUST NOT be flushed merely because a source body changed.

### 11.2 Content cache

```text
SourceId → source text
```

This is invalidated per changed source.

### 11.3 Reverse identity cache

```text
SourceId ↔ ModuleId
```

This is invalidated when ownership or logical identity changes.

### 11.4 Negative cache entries

Negative path results such as `ModuleNotFound` MAY be cached.

They MUST be tied to topology validity so that creating the missing source invalidates the result immediately.

---

## 12. Transactional workspace updates

Workspace updates MUST remain transactional:

> A failed internal update MUST NOT corrupt the previously committed module world.

However, transactions SHOULD NOT clone O(workspace-size) maps on every edit.

A recommended architecture is a delta transaction:

```rust
pub struct WorkspaceModuleTransaction<'a> {
    base: &'a WorkspaceModuleState,

    source_updates: BTreeMap<ModuleId, WorkspaceSourceState>,
    source_removals: BTreeSet<ModuleId>,

    interface_updates: BTreeMap<ModuleId, InterfaceProduct>,
    resolution_updates: BTreeMap<ImportSiteId, ImportResolutionProduct>,
    link_updates: BTreeMap<ModuleId, LinkedModuleProduct>,
}
```

Lookup follows:

```text
transaction delta
    → fallback to committed state
```

Commit applies changed keys only.

Equivalent copy-on-write implementations are acceptable.

### 12.1 Cache sharing across transactions

Transactional staging MUST NOT require a fresh independent filesystem cache universe.

Immutable/generation-safe filesystem caches SHOULD be shared across transactions.

Overlays remain transaction-specific.

---

# Part IV — Interfaces

## 13. Unlinked module interface

`InterfaceBuilder` produces the source-local static module surface.

The interface includes at least:

- module-scope declarations;
- module import declarations;
- selective import declarations;
- re-export declarations;
- local exports;
- package exposure declarations;
- module-global namespace information.

Interface building MUST remain independent of source statement order for declarative exports.

### 13.1 Unified namespace

All module-level binding producers participate in one namespace collision policy.

Examples:

- class vs class;
- enum vs class;
- type alias vs import;
- import alias vs declaration;
- top-level binding vs import alias.

Duplicate/ambiguous bindings MUST be diagnosed.

---

## 14. `InterfaceFingerprint`

Each module interface SHOULD have a deterministic semantic fingerprint.

```rust
pub struct InterfaceProduct {
    pub interface: Arc<UnlinkedModuleInterface>,
    pub fingerprint: InterfaceFingerprint,
}
```

### 14.1 Included semantics

The fingerprint MUST include anything that can affect module linking or topology, including:

- module kind;
- module-scope declaration identities/shapes needed by linking;
- import declarations and local import names;
- export declarations;
- re-export declarations;
- exposure declarations.

### 14.2 Excluded semantics

The fingerprint SHOULD exclude:

- ordinary callable bodies;
- local expressions;
- comments;
- formatting;
- implementation details that do not affect module interface identity.

### 14.3 Rebuild policy

When source `M` changes:

1. rebuild the interface for `M`;
2. compute its new fingerprint;
3. compare with the prior fingerprint.

If unchanged, module-level propagation MUST stop unless ownership/topology separately changed.

This is the first major incremental boundary.

---

# Part V — Module Resolution

## 15. Resolver authority

Only `phalcom-modules` may interpret `ImportPath`.

The canonical query is:

```text
(importing ModuleId, ImportPath, topology)
    → target ModuleId or structured diagnostic
```

No semantic or LSP subsystem may reconstruct a target from URI spelling or filesystem adjacency.

---

## 16. Relative imports

Relative imports operate on logical package ancestry.

A `StandaloneModule` has no sibling package namespace.

Therefore a relative sibling import from a standalone module fails with a package-context diagnostic.

Relative imports within a valid package context MAY access internal children without `expose`.

---

## 17. Absolute imports

Absolute imports resolve through the import-root table established by project/Universe/module infrastructure.

The root table is semantic configuration, not filesystem guessing.

---

## 18. External exposure

Cross-owner hierarchical traversal MUST obey exposure edges.

For:

```text
dep/
├── package.ph
└── a/
    ├── package.ph
    └── b.ph
```

external import of `dep.a.b` requires public path edges:

```phalcom
// dep/package.ph
expose .a
```

and:

```phalcom
// dep/a/package.ph
expose .b
```

Every externally traversed package boundary must authorize the next child.

---

## 19. Import resolution products

The workspace SHOULD retain per-import resolution products.

Conceptually:

```rust
pub struct ImportResolutionProduct {
    pub importer: ModuleId,
    pub written_path: ImportPath,
    pub target: Result<ModuleId, ModuleResolutionDiagnostic>,
    pub topology_fingerprint: TopologyFingerprint,
    pub fingerprint: ResolutionFingerprint,
}
```

The exact identifiers MAY differ.

### 19.1 Reuse conditions

A resolution product is reusable when:

- the written import path is unchanged;
- importer identity/ownership is unchanged;
- all topology/exposure dependencies observed by the result remain semantically unchanged.

A normal body edit in either importer or target MUST NOT force path re-resolution.

---

## 20. Resolution trace

The resolver SHOULD be able to retain a path-resolution trace sufficient for:

- precise diagnostics;
- package exposure diagnostics;
- segment-level editor navigation where desired;
- dependency recording.

Conceptually:

```text
root
→ package segment
→ package segment
→ final module/package
```

The trace is compiler data. LSP MUST NOT reproduce it.

---

# Part VI — Linking

## 21. Linker authority

The linker owns cross-module symbol identity.

Only linked module interfaces may answer:

```text
target ModuleId + public spelling
    → LinkedExportTarget
```

The semantic layer MUST NOT construct cross-module declaration identity from `(ModuleId, spelling)` and then test whether such a declaration exists.

---

## 22. Symbol visibility

Declarations are private by default.

For:

```phalcom
enum Either<L, R> {
    ...
}
```

cross-module import is invalid unless the name is made public, typically through:

```phalcom
export Either
```

Module-path resolution and symbol-link resolution are distinct products.

Therefore:

```phalcom
from .either import Either
```

may have:

```text
.either  → valid ModuleId
Either   → invalid because not exported
```

This distinction is intentional.

---

## 23. Export and `expose`

`export` and `expose` are orthogonal.

```text
export Foo
    → symbol visibility

expose .child
    → child path visibility
```

`expose` MUST NOT create a runtime binding or public symbol export.

---

## 24. Re-exports

A re-export is an interface edge, not a new declaration.

Given:

```phalcom
// either.ph
enum Either<L, R> { ... }
export Either
```

and:

```phalcom
// package.ph
export Either from .either
```

a consumer importing `Either` from the package MUST receive the same canonical origin identity owned by `either.ph`.

Re-export chains MUST preserve canonical origin.

---

## 25. Linked global target kinds

A linked exported binding is not always a nominal declaration.

The semantic target algebra MUST distinguish nominal declarations from module-global values.

This specification ratifies an explicit module-global semantic target.

Conceptually:

```rust
pub enum SemanticTargetId {
    Module(ModuleId),

    Declaration(DeclarationId),
    ModuleBinding(SymbolId),

    Binding(SourceSiteId),
    Callable(CallableId),
    Field(FieldId),

    Variant(VariantId),
    VariantFamily(VariantFamilyId),
    VariantField(VariantFieldId),

    // future canonical targets...
}
```

The exact enum location/name MAY vary, but the semantic distinction is mandatory.

### 25.1 Projection rules

Examples:

```text
class Foo
    → Declaration(Foo)

enum Either
    → Declaration(Either)

type Alias
    → Declaration(Alias)

const version = "1"
    → ModuleBinding(module::version)

let state = ...
    → ModuleBinding(module::state)

module object
    → Module(ModuleId)
```

The semantic session MUST NOT coerce every `LinkedExportTarget::Binding(SymbolId)` into `DeclarationId`.

---

## 26. `LinkedInterfaceFingerprint`

Linked module products SHOULD carry a semantic fingerprint.

```rust
pub struct LinkedModuleProduct {
    pub module: Arc<LinkedModule>,
    pub fingerprint: LinkedInterfaceFingerprint,
}
```

The fingerprint MUST represent the canonical externally meaningful linked result, including:

- public export names;
- canonical export targets;
- linked import target identities relevant to semantic consumers;
- module-object exports where present.

If a module is recomputed but produces the same `LinkedInterfaceFingerprint`, downstream linked dependents SHOULD be reusable.

---

## 27. Link connected components once

A workspace rebuild MUST NOT redundantly relink the same connected module component once per module entry.

At minimum, one update links each affected connected component once.

The preferred long-term behavior is per-module incremental linking with reverse dependency propagation.

---

## 28. Reverse linker dependency index

The module workspace SHOULD retain reverse dependencies sufficient to identify affected link products.

Conceptually:

```text
target module/export
    → importers/re-exporters depending on it
```

When a linked interface fingerprint changes, propagate only through that reverse closure.

Propagation stops when downstream fingerprints stabilize.

---

# Part VII — Strict and Tolerant Worlds

## 29. One link engine, two consumption policies

Phalcom needs two consumers:

```text
strict closed-program compilation
tolerant workspace analysis
```

They MUST share the same resolver, interface rules, exposure rules, and linker semantics.

There MUST NOT be a separate “LSP linker” with different language meaning.

---

## 30. Link report

The canonical module-linking pipeline SHOULD be capable of producing a report containing both successful products and structured failures.

Conceptually:

```rust
pub struct ModuleLinkReport {
    pub modules: BTreeMap<ModuleId, LinkedModuleProduct>,
    pub graphs: ModuleGraphs,
    pub diagnostics: Vec<ModuleDiagnostic>,
    pub unresolved: Vec<UnresolvedModuleEdge>,
}
```

### 30.1 Strict consumer

Compiler/runtime compilation rejects a report containing fatal module errors.

### 30.2 Workspace consumer

The LSP semantic workspace publishes all valid products plus diagnostics as a partial current snapshot.

The semantic rules are identical.

---

## 31. Source-authored failures are not cancellation

These conditions are user/source diagnostics:

- missing module;
- invalid relative import context;
- missing export;
- non-exported import;
- invalid `expose`;
- unknown export;
- re-export failure;
- package topology violation.

They MUST NOT cause a workspace batch to be treated as cancelled.

Cancellation is reserved for:

- explicit cancellation token;
- superseding revision/epoch;
- shutdown;
- equivalent infrastructure control flow.

Internal infrastructure failure is a third category and MUST be distinguishable from both.

---

## 32. Partial snapshot semantics

A current invalid workspace SHOULD publish:

```text
SemanticSnapshot generation N
status: Partial
diagnostics: current
valid products: retained/current
unresolved products: explicit
```

It MUST NOT silently serve generation `N-1` as if it represented current source.

Last-known-good semantic products MAY be retained internally for explicitly defined fallback behavior, but current diagnostics and source identity must correspond to current source.

---

# Part VIII — Semantic Projection

## 33. Cross-module type resolution

Cross-module semantic resolution MUST pass through linked public exports.

Forbidden pattern:

```rust
let decl = DeclarationId::new(target_module, leaf_name);
if known_declarations.contains(&decl) {
    return Some(decl);
}
```

Required pattern:

```text
module import binding
    ↓
LinkedReadSpec::Module(target)
    ↓
target LinkedModuleInterface.exports[leaf]
    ↓
LinkedExportTarget
    ↓
canonical SemanticTargetId
```

Thus:

```phalcom
import .models as models
```

does not make `models.Hidden` type-visible unless `Hidden` is publicly exported.

---

## 34. Unsupported deep qualification

Until true multi-component namespace traversal is implemented, unsupported qualified type forms MUST fail closed.

The semantic resolver MUST NOT ignore intermediate components or fall back to leaf-name lookup.

---

## 35. Semantic target projection

The semantic workspace maintains a canonical projection from linked module identities into semantic identities.

Conceptually:

```rust
pub struct SemanticTargetProjection {
    pub modules: BTreeMap<ModuleId, SemanticTargetId>,
    pub symbols: BTreeMap<SymbolId, SemanticTargetId>,
}
```

This projection is consumed by:

- source indexing;
- type resolution;
- editor queries;
- reference analysis;
- import completion;
- hover.

There MUST NOT be separate projection logic in LSP code.

---

# Part IX — Source Provenance

## 36. Source semantic index

Every source-authored canonical semantic entity that can be navigated MUST have source provenance.

The source index maps:

```text
source location/range
    ↔ SourceSiteId
    ↔ SemanticTargetId
```

and supports reverse target lookup.

---

## 37. Declaration source coverage

At minimum, the source index MUST cover:

| Source construct | Canonical target |
|---|---|
| class | `DeclarationId` |
| enum | `DeclarationId` |
| type alias | `DeclarationId` |
| top-level `let` / `const` | `ModuleBinding(SymbolId)` or equivalent |
| method | `CallableId` |
| getter | `CallableId` |
| setter | `CallableId` |
| index getter/setter | `CallableId` |
| field | `FieldId` |
| enum variant | `VariantId` |
| variant family | `VariantFamilyId` where source-addressable |
| variant payload field | `VariantFieldId` |

Enum declarations MUST NOT be omitted from top-level declaration indexing.

---

## 38. Dependency syntax occurrences

Dependency preamble syntax is part of the source semantic graph.

The index MUST cover:

- whole-module import paths;
- selective import remote names;
- import aliases;
- re-export paths;
- re-export names;
- `expose` child paths;
- export references.

---

## 39. Import occurrence model

For:

```phalcom
from .either import Either as E
```

the source model distinguishes:

```text
.either
    → Module target

Either
    → remote canonical export target

E
    → local import Binding target
       + origin edge to remote canonical target
```

This distinction is required for correct navigation, references, and rename.

---

## 40. Import origin relation

An import alias/local binding SHOULD retain an explicit origin relation.

Conceptually:

```rust
pub struct ImportBindingOrigin {
    pub local_binding: SourceSiteId,
    pub remote_target: SemanticTargetId,
}
```

or equivalent.

This permits:

```text
Go to Definition
    → follow import origin to upstream declaration/value

Find References on local alias
    → local binding/reference set

Rename local alias
    → local alias/reference set

Rename upstream declaration
    → declaration/export/re-export/remote-import references
```

The LSP MUST NOT infer this distinction from syntax.

---

## 41. Export occurrences

For:

```phalcom
export Either
```

the `Either` token is a source reference to its actual namespace target.

For a module-global value:

```phalcom
const version = "1"
export version
```

the export occurrence targets the module-binding identity, not a fabricated nominal declaration.

---

## 42. Re-export occurrences

For:

```phalcom
export Either from .either
```

the source index records at least:

```text
.either
    → target ModuleId

Either
    → canonical upstream export target
```

The re-export statement does not create a new declaration identity.

---

## 43. `expose` occurrences

For:

```phalcom
expose .parser
```

the child path occurrence targets the child `ModuleId`.

The compiler/module workspace MUST validate that the exposed child exists and is an immediate child of the package.

---

## 44. Module definition locations

`SemanticTargetId::Module(ModuleId)` MUST be navigable to the canonical source backing that module/package.

A protocol-neutral editor definition location SHOULD represent both ranged declaration definitions and whole-module source definitions.

Conceptually:

```rust
pub enum SemanticDefinitionLocation {
    Source {
        module: ModuleId,
        range: SourceRange,
    },

    Module {
        module: ModuleId,
        source: SourceLocation,
    },
}
```

---

# Part X — Editor Query Architecture

## 45. `EditorSemanticQuery` is the protocol-neutral editor API

Editor features MUST consume immutable semantic snapshots through compiler-owned query methods.

The query layer SHOULD provide or subsume:

```rust
target_at(module, offset)
definition_locations(...)
reference_sites(...)
visible_symbols(...)
public_exports(...)
module_at_source(...)
source_site_at(...)
```

The exact names MAY differ.

---

## 46. Go-to-definition

The end-state LSP path is:

```text
URI
 ↓
snapshot source mapping
 ↓
ModuleId
 ↓
offset
 ↓
EditorSemanticQuery
 ↓
source occurrence / target
 ↓
definition location(s)
 ↓
LSP Location conversion
```

LSP go-to-definition MUST NOT:

- inspect import syntax to derive semantic meaning;
- call the filesystem;
- call the module resolver;
- fabricate declaration identity;
- parse source;
- scan sibling files.

Any current import-specific go-to-definition fallback becomes obsolete once the source index is complete and SHOULD be deleted.

---

## 47. Hover

Hover on module/import/export syntax consumes the same canonical targets.

A module hover MAY present:

- logical module name;
- package/module kind;
- canonical source;
- public exports;
- package ownership.

LSP hover MUST NOT independently derive module identity from an import path.

---

## 48. Completion

### 48.1 Selective imports

For:

```phalcom
from .either import |
```

completion candidates come from the target module's linked public export interface.

Private declarations MUST NOT appear.

### 48.2 External module paths

External path completion observes package exposure.

### 48.3 Internal relative paths

Same-owner package-internal completion may include internally resolvable children even when they are not externally exposed.

---

## 49. References and rename

References follow canonical target identity.

Import aliases remain independent local bindings with origin relations.

This supports:

```text
rename alias
    → local uses only

rename upstream declaration
    → declaration/export/re-export/remote-import references
```

---

## 50. LSP performance invariant

Interactive request handlers MUST perform zero:

- filesystem I/O;
- parsing;
- interface construction;
- module resolution;
- linking;
- semantic type checking.

Requests are read-only queries over the current immutable snapshot.

The semantic snapshot is the cache. LSP MUST NOT add `(URI, position) → semantic answer` caches.

---

# Part XI — Incrementality and Product Stability

## 51. Product ladder

The module workspace SHOULD expose stable incremental products at approximately these boundaries:

```text
SourceRevision / ParsedInputFingerprint
                 │
                 ▼
        InterfaceFingerprint
                 │
                 ▼
        TopologyFingerprint
                 │
                 ▼
        ResolutionFingerprint
                 │
                 ▼
     LinkedInterfaceFingerprint
                 │
                 ▼
    RuntimeDependencyFingerprint
                 │
                 ▼
           SemanticDb
```

Not every implementation requires a public Rust type for each fingerprint, but the semantic invalidation boundaries MUST exist.

---

## 52. Body-only edit

For a change confined to an implementation body:

```text
parse/recover changed source
    ↓
rebuild changed module interface
    ↓
InterfaceFingerprint unchanged
    ↓
stop module propagation
    ↓
SemanticDb recomputes affected source/callable queries
```

Expected module work:

- one changed source;
- one interface rebuild;
- no import re-resolution;
- no relink;
- no ownership rediscovery except cheap cached validation.

---

## 53. Import surface edit

When imports change:

1. rebuild changed interface;
2. re-resolve changed import sites;
3. update reference/runtime dependency edges;
4. relink affected module;
5. propagate only if linked fingerprint changes.

Unchanged imports SHOULD retain their previous resolution products.

---

## 54. Export surface edit

When exports change:

1. interface fingerprint changes;
2. topology remains unchanged unless `expose` also changed;
3. import path resolution remains reusable;
4. linked interface is recomputed;
5. reverse import/re-export dependents are reconsidered;
6. propagation stops when linked fingerprints stabilize.

---

## 55. Exposure edit

Changing:

```phalcom
expose .foo
```

changes topology.

Affected work includes:

- changed package interface;
- topology/exposure product;
- external import paths whose traversal depends on that package edge;
- relevant completion/navigation products;
- downstream link results.

Internal same-owner relative imports that do not depend on exposure SHOULD remain reusable.

---

## 56. `package.ph` added/removed

Adding/removing a package marker is an ownership/topology identity change.

The workspace MUST:

- reclassify affected sources;
- retire old module identities where necessary;
- allocate canonical new identities;
- invalidate topology-dependent resolution products;
- rebuild source provenance;
- purge obsolete cache entries;
- publish a new current snapshot.

An IDE restart MUST NOT be required.

---

## 57. Project configuration change

Changes to `project.toml` or equivalent project configuration may affect:

- project identity context;
- namespace;
- source root;
- dependency aliases;
- absolute import roots;
- module ownership.

Such changes invalidate the relevant project topology domain.

---

## 58. Source add/remove/rename

Source lifecycle changes are topology events when they alter module existence or identity.

A rename that changes physical spelling but not canonical logical identity is governed by canonical naming rules.

A rename that changes logical identity invalidates corresponding module IDs and dependent products.

---

# Part XII — Semantic DB Interaction

## 59. Preserve existing semantic query caching

`SemanticDb` is the authoritative fine-grained semantic cache.

Its reuse principle is:

```text
input fingerprint unchanged
+
dependencies validated for current revision
+
observed dependency product fingerprints unchanged
=
reuse semantic product
```

The module repair MUST NOT introduce a parallel semantic cache above or below this mechanism.

---

## 60. Module layer adopts product stability

The module workspace SHOULD apply the same conceptual rule:

```text
changed immediate input
    ↓
recompute immediate product
    ↓
compare semantic fingerprint
    ↓
propagate only if meaning changed
```

This is the core performance principle of the architecture.

---

## 61. Invalidation vs purge

Two lifecycle operations MUST be distinguished.

### 61.1 Invalidate for recomputation

The identity still exists and may produce a new product.

Preserve dependency metadata necessary for product-stability reuse where appropriate.

### 61.2 Hard purge

The identity can no longer become valid in the current ownership topology.

Examples:

- source deleted;
- module identity replaced;
- project removed;
- standalone module converted into package-owned identity.

Hard purge removes:

- current query products;
- last-known-good products for obsolete identities;
- dependency edges;
- reverse indexes;
- interface products;
- resolution products;
- linked products;
- source mappings.

Long-lived LSP sessions MUST NOT retain obsolete identity products indefinitely.

---

# Part XIII — Data Structures and Complexity

## 62. Deterministic maps

Use of `BTreeMap` / `BTreeSet` remains acceptable and is preferred where deterministic ordering is useful.

This architecture MUST first eliminate broad recomputation before considering container-level micro-optimization.

An O(log N) lookup is not the dominant problem when unnecessary O(N) rebuilds still occur.

---

## 63. Transaction complexity target

Ordinary single-source workspace edits SHOULD avoid O(workspace-size) state cloning.

Target transactional cost is approximately:

```text
O(changed entries × log workspace size)
```

plus genuinely affected graph work.

---

## 64. Interactive query complexity target

Typical navigation should approximate:

```text
SourceId/URI → ModuleId       indexed lookup
offset → occurrence           interval/indexed lookup
occurrence → target           indexed lookup
target → definition           reverse/indexed lookup
```

No graph reconstruction occurs on the request thread.

---

## 65. Path canonicalization

Filesystem canonicalization SHOULD occur at source/topology ingestion boundaries.

Known canonical sources SHOULD reuse stable `SourceId`/`SourceLocation` rather than repeatedly calling filesystem canonicalization in ordinary editor requests.

---

# Part XIV — Metrics

## 66. Module-layer metrics

The module workspace SHOULD expose counters at least equivalent to:

```text
module.interfaces_built
module.interfaces_reused

module.imports_resolved
module.import_resolutions_reused

module.linked_modules
module.linked_modules_reused
module.linked_components

module.ownership_lookups
module.ownership_cache_hits

module.filesystem_resolution_hits
module.filesystem_resolution_misses

module.topology_invalidations

module.update.changed_sources
module.update.affected_modules
module.update.identity_changes

module.cache.purged_products
```

These metrics are necessary to verify that incrementality is real.

---

## 67. Work-count tests

Performance regressions SHOULD be asserted using deterministic recomputation counts, not only wall-clock timing.

Example fixture:

```text
A imports B
B imports C
100 unrelated modules
```

Edit only a method body in `C`.

Expected:

```text
interfaces built: 1
unrelated interfaces reused
import paths re-resolved: 0
linked modules recomputed: 0
```

assuming `C`'s interface fingerprint is unchanged.

---

# Part XV — Diagnostics

## 68. Module diagnostics are first-class

The module layer MUST distinguish at least these semantic failure classes:

- relative import requires package context;
- module not found;
- relative import beyond root;
- imported name absent;
- imported name exists but is not exported;
- unknown local export;
- invalid expose outside package;
- invalid expose target;
- exposed child missing;
- external module path not exposed;
- import binding collision;
- re-export cycle;
- runtime initialization cycle;
- invalid package topology.

Exact diagnostic codes are defined by the diagnostic catalog, but the distinctions are normative.

---

## 69. Precise ranges

Source-authored module errors MUST retain precise item ranges.

Examples:

```phalcom
from .either import Missing
                    ^^^^^^^
```

```phalcom
export Missing
       ^^^^^^^
```

```phalcom
export Missing from .either
       ^^^^^^^
```

Default/fabricated ranges MUST NOT replace known AST ranges.

---

## 70. Cascade suppression

A failed import SHOULD create a blocked/unresolved local import binding product so later semantic analysis can avoid noisy cascades.

For:

```phalcom
from .either import Missing

Missing.foo()
```

the primary diagnostic is the import failure.

The checker SHOULD avoid emitting a cascade of unrelated unknown-name/receiver/inference diagnostics caused solely by that failure.

---

# Part XVI — Canonical API Responsibilities

## 71. `phalcom-modules`

`phalcom-modules` owns:

- `EntryOwnership`;
- ownership classification;
- project/package topology;
- `ModuleId`;
- `ModuleKind`;
- source providers;
- module path resolution;
- package exposure;
- `UnlinkedModuleInterface`;
- `InterfaceFingerprint`;
- resolved import products;
- linker;
- linked interfaces;
- linked fingerprints;
- module graphs;
- module diagnostics;
- incremental module-session products.

It does not own language type inference or LSP protocol types.

---

## 72. `phalcom-semantic`

`phalcom-semantic` owns:

- semantic declaration identity;
- module-global semantic target projection;
- type resolution over linked module products;
- semantic diagnostics derived from module products;
- `SourceSemanticIndex`;
- source occurrence/definition/reference identity;
- import origin relationships;
- semantic query DB;
- immutable `SemanticSnapshot`;
- protocol-neutral `EditorSemanticQuery`.

It MUST NOT perform filesystem module guessing.

---

## 73. `phalcom-lsp`

`phalcom-lsp` owns:

- URI/range/position conversion;
- workspace candidate-file discovery scheduling;
- request/notification handling;
- presentation adaptation;
- publication of compiler diagnostics;
- orchestration of compiler-owned workspace updates.

It MUST NOT own:

- module resolution;
- package identity;
- export visibility;
- declaration synthesis;
- semantic fallback heuristics.

---

## 74. `phalcom-core`

Strict compiler/runtime-facing module compilation consumes the same module topology/resolution/linking products.

It MAY impose strict “no fatal module diagnostics” gating before executable program construction.

It MUST NOT reinterpret standalone/package ownership differently from the semantic workspace.

---

# Part XVII — Required End-to-End Semantics

## 75. Valid selective enum import

Filesystem:

```text
pkg/
├── package.ph
├── main.ph
└── either.ph
```

`either.ph`:

```phalcom
enum Either<L, R> {
    ...
}

export Either
```

`main.ph`:

```phalcom
from .either import Either
```

Required products:

```text
main.ph ownership
    → package-owned module

.either
    → ModuleId(pkg.either)

Either export lookup
    → LinkedExportTarget::Binding(SymbolId(pkg.either, Either))

semantic projection
    → DeclarationId(pkg.either::Either)

source occurrence `.either`
    → SemanticTargetId::Module(pkg.either)

source occurrence imported `Either`
    → SemanticTargetId::Declaration(pkg.either::Either)

enum declaration source
    → same DeclarationId
```

Go-to-definition on imported `Either` lands on the enum declaration.

---

## 76. Resolved module, private imported member

`either.ph` defines but does not export `Either`.

Required behavior:

```text
.either
    → valid ModuleId

Either
    → NonExportedImport diagnostic

workspace
    → current Partial snapshot

path navigation
    → may navigate to either.ph

member navigation
    → no false successful declaration target
```

Module-path and member-link status are intentionally independent.

---

## 77. Package-less sibling import

Filesystem:

```text
scratch/
├── main.ph
└── either.ph
```

No `package.ph`, no owning project.

`main.ph`:

```phalcom
from .either import Either
```

Required behavior:

```text
main.ph
    → StandaloneModule

relative import
    → RelativeImportRequiresPackageContext

no target ModuleId for `.either`
no imported member target
```

The sibling file's physical existence is irrelevant.

---

## 78. Qualified private type

```phalcom
import .models as models
```

`models.ph`:

```phalcom
class Hidden {}
class Public {}

export Public
```

Required:

```phalcom
let a: models.Public
```

resolves.

```phalcom
let b: models.Hidden
```

does not resolve cross-module.

The semantic resolver MUST query linked exports and MUST NOT synthesize `DeclarationId(models, Hidden)`.

---

## 79. Package façade

`either.ph`:

```phalcom
enum Either<L, R> { ... }
export Either
```

`package.ph`:

```phalcom
export Either from .either
```

Consumer:

```phalcom
from collection import Either
```

Required canonical semantic identity:

```text
DeclarationId(collection.either::Either)
```

The package façade creates no new declaration identity.

---

## 80. Exported module-global value

`constants.ph`:

```phalcom
const pi = 3.14159
export pi
```

Consumer:

```phalcom
from .constants import pi
```

Required target:

```text
SemanticTargetId::ModuleBinding(SymbolId(constants, pi))
```

not:

```text
DeclarationId(constants, pi)
```

Go-to-definition lands on the top-level binding declaration.

---

# Part XVIII — Incremental Scenarios

## 81. Remove export while importer is open

Initial:

```phalcom
export Either
```

Importer resolves.

Delete the export.

Required next generation:

- source mutation succeeds as a workspace update;
- package/module path remains resolved;
- imported member becomes non-exported;
- current partial snapshot is published;
- diagnostic appears at imported `Either`;
- previous target is not exposed as current truth;
- unrelated semantic products remain usable;
- restoring export recovers without restart.

---

## 82. Add `package.ph`

Initial:

```text
x/
├── a.ph
└── b.ph
```

Both are standalone modules.

Add live/disk:

```text
x/package.ph
```

Required:

- topology fingerprint changes;
- relevant ownership cache invalidates;
- `a.ph` and `b.ph` are reclassified into package context as appropriate;
- old standalone identities are retired;
- new canonical package identities are published;
- relative imports may now resolve;
- obsolete semantic/module cache products are purged.

---

## 83. Remove `package.ph`

The inverse transition MUST work without IDE restart.

---

## 84. Add/remove `expose`

An exposure change invalidates only the path-resolution consumers whose external traversal depends on the changed edge.

Internal same-package resolution SHOULD remain unaffected.

---

## 85. Body-only edit

Change only a method body in a module exported to many importers.

Required:

- changed source parsed/recovered;
- changed interface rebuilt;
- `InterfaceFingerprint` unchanged;
- import path products reused;
- linked interface reused;
- importer module products not relinked;
- semantic DB handles body-level invalidation.

---

# Part XIX — Conformance Test Matrix

## 86. Ownership tests

Required tests include:

```text
project_missing_root_package_is_rejected
workspace_folder_is_not_implicitly_package
standalone_sibling_files_do_not_form_package
standalone_module_cannot_relative_import_sibling
standalone_package_supports_relative_children
nested_standalone_package_ownership_is_preserved
direct_file_inside_standalone_package_uses_package_identity
package_entry_requires_package_ph
main_ph_does_not_create_package_identity
```

---

## 87. Topology tests

```text
intermediate_directory_without_package_ph_is_not_package
package_marker_addition_changes_topology_fingerprint
package_marker_removal_changes_topology_fingerprint
body_edit_does_not_change_topology_fingerprint
expose_edit_changes_topology_fingerprint
export_only_edit_does_not_change_topology_fingerprint
external_hierarchical_path_requires_each_expose_edge
internal_relative_path_does_not_require_expose
```

---

## 88. Export/link tests

```text
exported_class_selective_import_resolves
exported_enum_selective_import_resolves
exported_type_alias_selective_import_resolves
exported_global_value_uses_module_binding_target

private_class_selective_import_rejected
private_enum_selective_import_rejected
unknown_import_member_distinct_from_non_exported_member

qualified_public_type_through_module_alias_resolves
qualified_private_type_through_module_alias_rejected

direct_reexport_preserves_origin_identity
multi_hop_reexport_preserves_origin_identity
reexport_cycle_is_diagnostic
```

---

## 89. Source-index tests

Assert exact source target identities for:

```text
class declaration
enum declaration
type alias declaration
top-level const/let
variant declaration
variant payload field
module import path
selective remote import item
import alias
export item
re-export path
re-export item
expose child
```

These are semantic tests, not LSP-only tests.

---

## 90. LSP tests

Valid package fixture:

```text
workspace/
├── package.ph
├── main.ph
└── either.ph
```

Assert:

```text
go-to-definition on .either → either.ph
go-to-definition on imported Either → enum declaration
go-to-definition on Either type use → same enum declaration
hover uses canonical semantic target
completion from .either lists public exports only
```

Negative fixture without `package.ph` MUST reject sibling relative import.

The LSP test suite MUST NOT ratify package-less sibling namespace behavior.

---

## 91. Partial snapshot tests

Required scenarios:

```text
remove export → current partial snapshot + diagnostic
restore export → automatic recovery
break one import → unrelated modules remain semantically current
missing module created → negative resolution cache invalidates
delete module → obsolete identity products purged
```

---

## 92. Compiler/LSP parity tests

The same fixture MUST be processed through strict compiler analysis and semantic workspace analysis.

Assert equality of:

- module identities;
- resolved import targets;
- linked public export targets;
- declaration/module-binding identities.

Successful execution alone is insufficient.

---

## 93. Performance work-count tests

Required deterministic work tests include:

### Body-only edit

Expect:

- one interface rebuild;
- no import re-resolution if interface unchanged;
- no linked-interface recomputation;
- unrelated modules reused.

### Export change

Expect:

- changed interface recomputed;
- path resolutions reused;
- affected reverse linked closure reconsidered;
- unrelated modules reused.

### Exposure change

Expect:

- topology change;
- only external resolution closure affected where possible.

### Package marker change

Expect:

- ownership/topology invalidation limited to the affected domain;
- not an unconditional full-workspace semantic reset.

---

# Part XX — Migration Requirements

## 94. Ownership cutover

Replace arbitrary parent-directory synthetic-root behavior with authoritative `EntryOwnership`.

`WorkspaceModuleSession`, strict compiler entry selection, and LSP ingestion MUST share this ownership mechanism.

---

## 95. Interface retention

Retain `InterfaceProduct` per module with semantic fingerprints.

Do not rebuild interfaces for unchanged modules.

---

## 96. Resolution retention

Retain import resolution products with topology dependencies.

Do not re-resolve unchanged imports after body-only edits.

---

## 97. Linker incrementality

First ensure affected connected components are linked once.

Then add linked-product retention and reverse dependency propagation.

---

## 98. Semantic target correction

Introduce/ratify module-global semantic targets.

Remove the rule that all linked bindings become `DeclarationId`.

---

## 99. Cross-module visibility correction

Remove semantic paths that synthesize `(target module, spelling)` declarations for external access.

All cross-module symbol resolution must consume linked exports.

---

## 100. Source-index completion

Add missing enum and dependency/export/expose source products.

Editor navigation must become complete at the compiler semantic layer.

---

## 101. LSP fallback retirement

Once source semantic products are complete:

- remove import-path semantic fallback logic;
- remove request-time module resolution;
- route go-to-definition/hover/completion through `EditorSemanticQuery`.

---

## 102. Error-tolerant workspace publication

Refactor module/interface/link source errors into diagnostics retained by workspace reports.

Only cancellation/supersession/infrastructure failure aborts publication.

---

## 103. Cache lifecycle

Add hard purge for obsolete identities.

Ensure long-running IDE sessions do not retain deleted/reidentified module products indefinitely.

---

# Part XXI — Release Gate

## 104. Architecture gate

The architecture is conformant only when all of the following are true.

### Ownership

- one authoritative ownership classifier exists;
- `package.ph` is the only package marker;
- workspace roots do not create package identity;
- standalone modules cannot import arbitrary siblings;
- directly selected files inside standalone packages retain package ownership.

### Resolution and linking

- only module infrastructure resolves `ImportPath`;
- cross-module symbols are resolved only through linked public exports;
- `export` and `expose` remain orthogonal;
- re-exports preserve canonical origin;
- exported module-global values have a proper semantic target.

### Workspace behavior

- source-authored module errors publish current partial snapshots;
- cancellation is not conflated with source invalidity;
- module topology changes recover incrementally.

### Source provenance

- classes, enums, aliases, globals, callables, fields, variants, imports, exports, re-exports, and exposes have canonical source targets.

### LSP

- no request-time filesystem/module resolution exists;
- import path navigation uses compiler source occurrences;
- selective import navigation uses canonical linked semantic targets;
- package-less sibling navigation is rejected.

### Performance

- body-only edits do not broadly re-resolve/relink;
- workspace transactions avoid O(workspace-size) full state cloning;
- filesystem caches survive transaction boundaries safely;
- product fingerprints stop invalidation propagation;
- hard purge exists for removed identities;
- recomputation metrics and work-count tests exist.

### Parity

- compiler and LSP produce identical canonical module/link identities for the same source universe.

---

# 105. Final Architectural Invariants

The implementation MUST preserve these invariants.

### MOD-OWN-1

A source has one ownership context per workspace generation.

### MOD-PKG-1

A filesystem directory has package semantics only through `package.ph`.

### MOD-ID-1

A canonical physical source has at most one `ModuleId` per module-session generation.

### MOD-RES-1

Only module infrastructure resolves logical import paths.

### MOD-TOP-1

`TopologyFingerprint` changes exactly when namespace path resolution semantics change.

### MOD-IFACE-1

A source edit propagates into module consumers only when its module interface product changes.

### MOD-EXP-1

Cross-module symbol visibility is determined exclusively by linked public exports.

### MOD-XPS-1

`expose` grants child path reachability and does not export symbols or create bindings.

### MOD-LINK-1

Re-exports preserve canonical origin identity.

### MOD-SEM-1

Cross-module semantic resolution never synthesizes declaration identity from target module plus spelling.

### MOD-SEM-2

Nominal declarations and module-global values have distinct canonical semantic targets.

### MOD-SRC-1

Every navigable source-authored semantic identity has source provenance.

### MOD-WS-1

Source-authored module failures publish current diagnostics and partial products rather than masquerading as cancellation.

### MOD-INC-1

Invalidation propagates by semantic product change, not merely source revision change.

### MOD-CACHE-1

Caches are owned by the layer that owns the corresponding semantic product.

### MOD-CACHE-2

LSP protocol code owns no semantic cache.

### MOD-CACHE-3

Obsolete identities are hard-purged from long-lived caches.

### MOD-LSP-1

LSP request handlers perform no parsing, filesystem resolution, interface construction, linking, or semantic checking.

### MOD-PARITY-1

Strict compiler analysis and workspace/LSP analysis consume the same module ownership, topology, resolution, and linking semantics.

---

# 106. Canonical Summary

The Phalcom module architecture is a staged semantic pipeline:

```text
source candidates
    ↓
ownership
    ↓
topology
    ↓
source-local interfaces
    ↓
module-path resolution
    ↓
cross-module linking
    ↓
semantic target projection
    ↓
source provenance
    ↓
immutable semantic snapshot
    ↓
editor queries
```

Each stage owns one form of meaning and one corresponding cache.

The architecture does not cache answers at arbitrary call sites. It caches **canonical semantic products**.

The incremental rule at every stage is:

```text
input changed
    ↓
recompute immediate product
    ↓
compare semantic fingerprint
    ↓
propagate only if semantic meaning changed
```

This produces the desired combination of:

- explicit package semantics;
- private-by-default module APIs;
- correct cross-module identity;
- deterministic source navigation;
- strict compiler/LSP parity;
- error-tolerant editor behavior;
- bounded incremental recomputation;
- cache correctness;
- low interactive latency.

That is the authoritative module architecture for Phalcom's LSP and semantic workspace.
