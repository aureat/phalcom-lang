# Current Phalcom Semantic Implementation Map

This file is a repository orientation map inspected on **2026-08-28** after the `phalcom-lsp` single-world retirement. It is not a normative language specification. Re-check current source/tests before repository work.

Use status labels rigorously:

```text
CURRENT      observed in current repository source/tests
NORMATIVE    established by current spec/ratified decision
PROPOSED     documented design not yet current behavior
EXPERIMENTAL repository experiment without normative guarantee
FUTURE       expected direction without ratified semantics
```

## 1. Ownership at a glance

**CURRENT:** semantic ownership is split by responsibility, not duplicated by consumer.

```text
phalcom-ast
  syntax / recovered Program / source ranges

phalcom-modules
  projects / packages / source identity / module identity
  source overlays / workspace lifecycle
  interfaces / import resolution / linking / module graph products

phalcom-semantic
  sole static semantic implementation
  canonical identities / TypeStore / SemanticDb
  checker / inference / dispatch / hierarchy
  source index / occurrences / editor queries
  formal + advisory products
  incremental SemanticWorkspaceSession
  immutable SemanticSnapshot

phalcom-lsp
  protocol adapter only
  live document store / workspace discovery / worker scheduling
  immutable snapshot publication / source-coherence checks
  protocol rendering and syntax-only cursor recovery
```

`phalcom-lsp` no longer contains `src/semantic/` or `src/index.rs` as an alternative semantic world.

## 2. Front end: `phalcom-ast`

**CURRENT:** `phalcom-ast` owns parsing/recovery and AST/source structures. Semantic layers consume `Program` plus source ranges; LSP does not maintain a second parser for semantics.

For syntax changes that introduce bindings, declarations, type forms, imports, call shapes, or control-flow constructs, inspect parser/AST changes together with the relevant semantic source-index/checker owners.

## 3. Module lifecycle: `phalcom-modules/src/session.rs`

**CURRENT:** `WorkspaceModuleSession` is the persistent compiler owner of:

- `ProjectUniverse`;
- project roots and synthetic standalone project identities;
- source overlays/disk snapshots;
- `SourceId` / `SourceLocation`;
- source-to-`ModuleId` mapping;
- linked workspace product;
- canonical resolved imports;
- module generation.

Primary mutation boundary:

```text
WorkspaceSourceMutation
WorkspaceSourceBatchMutation
        -> WorkspaceModuleSession
        -> WorkspaceModuleUpdate
```

`WorkspaceModuleUpdate` carries the linked program, parsed source map, changed modules, removed modules, and identity changes into `phalcom-semantic`.

## 4. Interfaces and linking: `phalcom-modules`

Inspect these owners for module/import semantics:

```text
src/interface.rs   declaration/import/export interface extraction
src/linker.rs      linked bindings and re-export resolution
src/resolver.rs    module path/source resolution
src/project.rs     project universe and project identity
src/graph.rs       canonical semantic/runtime module graph products
src/query.rs       immutable module query facade
```

Do not implement import semantics in `phalcom-lsp` request handlers.

A selective import must resolve through linked export identity. An unresolved import may retain a compiler-owned local binding identity until linking can establish the external target.

## 5. Semantic session: `phalcom-semantic/src/session.rs`

**CURRENT:** `SemanticWorkspaceSession` is the long-lived compiler semantic owner.

It owns:

```text
WorkspaceId
WorkspaceModuleSession
SemanticDb
TypeStore
base declaration table
base hierarchy
base dispatch resolver
base callable signatures
parsed sources + fingerprints
last snapshot
last-known-good snapshot
```

`apply_module_mutations` is the canonical workspace mutation path used by the LSP worker and compiler tests.

A successful update produces `SemanticWorkspacePublication`:

```text
Arc<SemanticSnapshot>
invalidated query keys
recomputed query keys
SemanticUpdateStats
SemanticPublicationEffects
```

## 6. Incremental query engine: `phalcom-semantic/src/db/`

**CURRENT:** `SemanticDb` is compiler-owned and lives in `phalcom-semantic`, not LSP.

The DB/query layer owns semantic query keys, dependencies, reuse validity, budgets/cancellation, cached products, and reverse invalidation.

Important rule: LSP scheduling/debounce is not semantic invalidation. Compiler query/product dependencies determine semantic reuse.

Before adding a new incremental cache, identify its key, product, dependencies, validity condition, retraction behavior, and publication semantics.

## 7. Type system and evidence

Inspect:

```text
src/types/
src/declarations.rs
src/signature.rs
src/hierarchy_product.rs
src/resolver.rs
src/export.rs
```

**CURRENT:** `TypeStore` is retained by `SemanticWorkspaceSession` across revisions and is part of `SnapshotId` identity through its store domain.

Formal type knowledge carries epistemic status/provenance. A developer annotation can supply evidence when the compiler cannot establish a fact, but cannot override a contradictory established compiler proof.

Do not replace unknown/unavailable proof states with convenient nominal guesses.

## 8. Checker and formal analysis

Primary implementation lives under:

```text
src/checker/
```

This includes declaration checking, expression synthesis, call application, generic inference, flow state, causal diagnostics, callable publication, and formal checker products.

For correctness work, inspect the checker path that owns the proof rather than patching a downstream presentation projection.

Formal products are authoritative when established.

## 9. Canonical dispatch and hierarchy

Inspect:

```text
src/dispatch.rs
src/surface.rs
src/hierarchy_product.rs
```

Canonical callable identity includes owner, selector and dispatch side. Inherited dispatch preserves the defining callable identity while receiver-specific semantics such as constructor `Self` specialization are handled by the checker/type model.

Keep selector construction and instance/class-side semantics aligned with runtime/compiler behavior.

## 10. Source semantic index: `phalcom-semantic/src/source_index/`

**CURRENT:** this package owns compiler source structure for editor/refactoring semantics:

```text
SourceScopeIndex
SourceScope / SourceBindingInfo
SourceSite / SourceSiteId
OccurrenceIndex / SemanticOccurrence
canonical site -> SemanticTargetId mapping
callable/declaration/field source metadata
expression/source attachments
```

Bindings and occurrences are built centrally from parsed source plus linked semantic context.

This is where lexical source identity belongs. LSP should not rediscover semantic definitions/references by matching text.

## 11. Canonical target identity

Inspect `phalcom-semantic/src/identity.rs` and source-index registration.

Important semantic identities include:

```text
BindingId / SourceSiteId
DeclarationId
CallableId
FieldId
ModuleId
SemanticTargetId
```

Definition/reference semantics are target-based, not spelling-based.

A resolved import token may occur in the importing file while denoting a declaration owned by another module. The import occurrence is therefore a reference site, not the declaration's canonical definition site.

## 12. Editor facade: `phalcom-semantic/src/editor.rs`

**CURRENT:** `EditorSemanticQuery` is the compiler-owned, protocol-neutral editor query API.

Current responsibilities include:

- target lookup at a position;
- definition/reference site classification;
- lexical access context;
- receiver resolution from formal/advisory products;
- canonical member selection/visibility;
- visible lexical symbols;
- native callable presentation metadata.

This facade is the preferred place to add semantic editor capabilities that multiple protocol features need.

It intentionally fails closed for unsupported/unknown semantic states instead of guessing.

## 13. Semantic snapshot: `phalcom-semantic/src/snapshot.rs`

**CURRENT:** `SemanticSnapshot` is the immutable coherent semantic publication.

Major retained products include:

```text
SnapshotId + generation
Arc<TypeStore>
parsed source map
compiler-generated presentation sources
declaration surfaces
dispatch resolver
callable signatures
declaration type table
hierarchy
diagnostics
semantic graph
callable analyses + internal incidents
formal projection
source semantic index
advisory workspace
module query products
snapshot completeness status
```

Old snapshots remain valid immutable values when held by existing readers.

## 14. Module query products: `phalcom-semantic/src/snapshot.rs` + `phalcom-modules/src/query.rs`

`ModuleQueryProducts` retains immutable module information needed by editor/compiler queries:

```text
ProjectUniverse
unlinked interfaces
linked interfaces
resolved import targets
module -> SourceLocation
SourceId -> ModuleId
PathBuf -> ModuleId
```

`SemanticSnapshot::module_queries()` exposes a pure compiler/module facade over these retained products.

Module path navigation should consume these products rather than re-run filesystem/import resolution.

## 15. Formal presentation: `phalcom-semantic/src/presentation.rs`

**CURRENT:** compiler-owned presentation projections combine semantic facts with source-site identity without depending on LSP protocol types.

Use this layer for reusable semantic presentation facts. Markdown, LSP labels, protocol ranges and client capability handling remain in `phalcom-lsp`.

## 16. Advisory analysis: `phalcom-semantic/src/advisory/`

**CURRENT:** advisory runtime-shape analysis remains a compiler semantic product, not an LSP inference engine.

`ValueShape` and related advisory facts are explicitly not the language type representation. Advisory facts carry confidence/status/provenance and are attached to the same canonical source/target identities as formal products.

Formal evidence wins when it is established. Advisory products are useful where formal knowledge is unavailable, but they cannot overwrite formal proof.

## 17. Diagnostics and explanations

Inspect:

```text
src/diagnostic.rs
src/explain.rs
src/checker/causal.rs
src/checker/context.rs
```

Semantic diagnostics are compiler products with module/source provenance. LSP converts them to protocol diagnostics; it should not independently decide type correctness.

Internal semantic incidents are retained separately from user-facing source diagnostics.

## 18. Invalidation and product stability

Inspect:

```text
src/invalidation.rs
src/db/
src/session.rs
```

Current tests cover body-only reuse, signature propagation, field/superclass/import dependencies, contribution retraction, range-only edits, type-store revisions, and clean/incremental stability.

Source movement is provenance change, not automatically semantic identity change.

## 19. LSP analysis worker: `phalcom-lsp/src/analysis_service.rs`

**CURRENT:** the LSP worker is a scheduler around one compiler session.

`CompilerWorkspaceState` contains exactly:

```text
phalcom_semantic::SemanticWorkspaceSession
```

The worker may:

- coalesce live edits;
- track open/closed source epochs;
- perform bounded filesystem discovery/refresh;
- construct source mutations;
- call the semantic session;
- publish returned snapshots;
- emit status/log/refresh notifications.

It may not implement semantic resolution, typing, dispatch, or a parallel generation model.

## 20. LSP publication: `phalcom-lsp/src/publication.rs`

`SemanticPublication` is a small `RwLock<Option<Arc<SemanticSnapshot>>>` publication cell.

It deliberately exposes no semantic mutation or lookup engine. Its public read-only handle is only for source-coherence scheduling/tests.

Do not evolve this cell into a second semantic database.

## 21. Request coherence: `phalcom-lsp/src/request_context.rs`

Each request pins the live `DocumentSnapshot` and one compiler `SemanticSnapshot`.

`SourceMatch` is:

```text
Exact
Stale
Unmapped
```

Semantic queries requiring source identity/ranges should run only against exact canonical source. Stale/unmapped states fail closed rather than triggering request-time semantic reconstruction.

## 22. LSP protocol consumers

High-level protocol routing lives primarily in `phalcom-lsp/src/backend.rs`; focused presentation/context helpers live in files such as:

```text
completion.rs
hover.rs
inlay_hints.rs
semantic_tokens.rs
signature_help.rs
import_completion.rs
```

Allowed consumer work includes:

- protocol object construction;
- cursor/syntax context recovery;
- snippets/ranking/markdown;
- line/range conversion;
- syntax-only fallback where it does not claim semantic identity.

Forbidden consumer work includes:

- name-resolution engines;
- semantic definition/reference fabrication;
- type/value inference;
- dispatch resolution;
- import resolution from filesystem/URI spelling.

## 23. Architectural regression gate: `phalcom-lsp/tests/semantic_boundary.rs`

This test is not merely stylistic. It makes single-world ownership executable.

It currently checks, among other things:

- no `phalcom-lsp/src/semantic` package;
- no old `src/index.rs` semantic bridge;
- forbidden legacy semantic types/helpers are absent from LSP production source;
- request features do not read/canonicalize filesystem paths;
- worker code does not reimplement import resolution/generation publication;
- `phalcom-semantic` does not depend on LSP;
- every top-level LSP test is actually registered despite `autotests = false`.

When retiring another compatibility bridge, add its forbidden symbol/pattern here when a mechanical boundary can prevent regression.

## 24. Import/navigation regression anchors

Use these tests for cross-module editor semantics:

```text
phalcom-semantic/tests/semantic.rs
  imported_resolution::*
  module_query_provenance::*

phalcom-modules/tests/standalone_incremental_imports.rs

phalcom-lsp/tests/module_navigation.rs
phalcom-lsp/tests/imported_binding_resolution.rs
```

They collectively prove module lifecycle, import provenance, imported type participation, canonical definition identity, and LSP projection.

## 25. Typing status discipline

Detailed typing documents can be normative, proposed, partially implemented, or stale. For each claim, distinguish specification status from repository implementation status.

Never infer that a feature is current merely because a detailed typing spec exists. Conversely, do not preserve obsolete implementation architecture merely because an older implementation map describes it.

## 26. Repository-review questions

Before a semantic change, answer:

- Which crate owns this concept now?
- Which stable/canonical identity represents it?
- Which query/product proves the fact?
- What evidence/status/provenance is retained?
- What invalidates the product?
- Which snapshot/revision owns it?
- Which regression proves the compiler behavior?
- Which LSP test proves protocol projection, if relevant?
- Can an architecture boundary test prevent a parallel semantic implementation from returning?
