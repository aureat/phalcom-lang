# Current Semantic Architecture

This is a repository-orientation map, not a frozen API or a language specification. It describes the post-retirement single-world architecture as observed on 2026-08-28. Re-check current source before editing.

## Architectural invariant

`phalcom-semantic` is the only implementation of Phalcom static semantics.

`phalcom-lsp` is a protocol, source-buffer, scheduling, workspace-discovery, and presentation adapter. It must not own an alternative semantic database, scope graph, module graph, dispatch engine, type/value inference engine, or import resolver.

The canonical direction is:

```text
source / recovered AST
        |
        v
phalcom-modules
  project + source + module identity
  overlays + module lifecycle
  import/link products
        |
        v
phalcom-semantic
  SemanticWorkspaceSession
  SemanticDb + TypeStore
  declarations + surfaces + hierarchy + dispatch
  formal checker products
  source/occurrence index
  advisory products
  immutable SemanticSnapshot
        |
        v
phalcom-lsp
  worker scheduling + publication
  RequestContext pins document + snapshot
  editor query facade -> protocol rendering
```

A change that introduces a second path around this flow needs explicit architectural justification.

## Front-end boundary

Semantic analysis consumes `phalcom_ast` programs/source ranges. LSP may retain recovered syntax for source-buffer operations and cursor recovery, but it does not independently interpret that syntax into semantic identity or type facts.

Important source concepts remain distinct:

```text
live document revision
module/source lifecycle revision
semantic workspace revision
semantic publication generation
snapshot identity
```

Do not collapse these into one counter or use a text revision as proof that a semantic product is current.

## Module and source ownership: `phalcom-modules`

`phalcom-modules::WorkspaceModuleSession` is the persistent owner of project/source/module lifecycle and linking.

It retains:

- project universe and project roots;
- source overlays and disk snapshots;
- `SourceId` / `SourceLocation` ownership;
- stable `ModuleId` mapping;
- linked program/interface products;
- canonical import resolutions keyed by importer and written logical import path;
- module generation.

Source clients submit `WorkspaceSourceMutation` / `WorkspaceSourceBatchMutation` values. The module layer contains no LSP protocol types.

Do not re-resolve imports from request URIs or filesystem spelling in an editor feature.

## Canonical semantic session: `phalcom-semantic/src/session.rs`

`SemanticWorkspaceSession` owns the stateful compiler semantic world across revisions.

Current owned state includes:

```text
WorkspaceId
WorkspaceModuleSession
SemanticDb
TypeStore
base/native declarations
base hierarchy
base dispatch
base callable signatures
parsed module sources
source fingerprints
last published snapshot
last-known-good snapshot
```

The session bootstraps universe/native semantics and applies module/source mutations through the canonical module session. One accepted workspace update produces one `SemanticWorkspacePublication` containing an immutable snapshot, invalidation/recomputation information, statistics, and product-level publication effects.

The LSP worker must call this owner rather than reproducing its update transaction.

## Canonical identities

Compiler identities live in `phalcom-semantic::identity` and module identities in `phalcom-modules`.

Important identity families include:

```text
ModuleId
DeclarationId
CallableId
FieldId
BindingId / SourceSiteId
SemanticTargetId
WorkspaceId
SemanticRevision
SnapshotId
TypeStoreId
```

Identity is semantic, not spelling-based. A source range is provenance/location evidence; it is not declaration identity.

Imported bindings are an important example:

- an unresolved selective import may have a compiler-owned local binding identity;
- once linking establishes an exported declaration, the import occurrence targets that external canonical declaration;
- the local import token is a reference to the external target, not the target's definition site.

## Immutable publication: `phalcom-semantic/src/snapshot.rs`

`SemanticSnapshot` is the coherent read boundary. It retains compiler-owned products such as:

```text
TypeStore
parsed sources
presentation-only generated sources
declaration surfaces
dispatch
callable signatures
declaration type table
hierarchy
diagnostics
semantic graph
callable analyses
formal projection
source semantic index
advisory workspace
module query products
snapshot completeness status
```

A request must observe one snapshot, never a mixture of generations.

`ModuleQueryProducts` retains canonical linked/unlinked interfaces, source/module provenance, display-path mapping, and resolved imports for pure immutable queries.

## Source index and occurrences

`phalcom-semantic::source_index` owns lexical source structure, bindings, source sites, occurrences, and canonical semantic targets.

This is the semantic substrate for navigation/refactoring. Do not replace it with text matching in LSP code.

The source index deliberately distinguishes:

```text
where syntax occurs
what semantic target it denotes
whether that source site is the canonical definition of that target
```

That distinction is essential for imports, aliases, inherited callables, generated/native source, and future re-exports.

## Editor query boundary: `phalcom-semantic/src/editor.rs`

`EditorSemanticQuery` is the protocol-neutral compiler-owned facade for editor semantics.

It owns queries such as:

- target at source position;
- definition sites;
- reference sites;
- receiver resolution;
- member visibility/candidates;
- lexical symbols;
- native callable presentation metadata.

Unknown or unsupported semantic states fail closed rather than inventing a target. If a new editor feature needs semantic reasoning, extend a compiler-owned query/fact first and adapt it in LSP second.

## Formal and advisory products

Formal checker/type products and advisory runtime-shape products coexist in the same semantic snapshot but have different authority.

Formal type knowledge is authoritative when established. Advisory analysis can help editor presentation when formal knowledge is unavailable, but it cannot contradict or replace a proved formal fact.

`ValueShape` and related advisory products are not the language type system.

Keep uncertainty, provenance, and product status explicit. Do not translate "not currently proved" into a fabricated dynamic or nominal type merely to produce an editor answer.

## Incrementality and invalidation

Incrementality is owned by the compiler semantic/module layers. The semantic session retains dependencies, query products, contribution ownership, and the last accepted publication.

The correctness criterion is semantic equivalence with clean recomputation for observable queries. Edits must retract stale evidence, not merely append new facts.

LSP scheduling may debounce/coalesce work, but scheduling policy must not become semantic invalidation logic.

## LSP worker boundary: `phalcom-lsp/src/analysis_service.rs`

`AnalysisService` owns protocol-side scheduling only.

Its worker owns one `CompilerWorkspaceState`, whose semantic state is exactly one `phalcom_semantic::SemanticWorkspaceSession`. The worker:

1. coalesces source updates/removals/disk refreshes;
2. performs bounded workspace discovery;
3. converts source events into module-layer mutations;
4. calls `SemanticWorkspaceSession::apply_module_mutations`;
5. publishes the returned immutable snapshot;
6. emits protocol-facing status/log/refresh events.

It must not implement import resolution, semantic generations, dispatch, typing, or source-target identity itself.

Filesystem access belongs to worker-side source discovery/refresh, not request-time semantic queries.

## LSP publication boundary: `phalcom-lsp/src/publication.rs`

`SemanticPublication` is intentionally a tiny cell containing only:

```text
Option<Arc<phalcom_semantic::SemanticSnapshot>>
```

It has no semantic lookup, mutation, invalidation, or identity-translation behavior. Existing requests can retain an older `Arc` while later requests observe a newer publication.

This is publication plumbing, not an LSP semantic database.

## Request coherence: `phalcom-lsp/src/request_context.rs`

Every feature request pins:

```text
DocumentSnapshot
Option<Arc<SemanticSnapshot>>
canonical ModuleId, if mapped
SourceMatch::{Exact, Stale, Unmapped}
```

Semantic editor requests should consume canonical products only when `SourceMatch::Exact` permits their source ranges/targets to be trusted.

`Stale` and `Unmapped` states must not trigger a request-local reimplementation of semantics.

## What syntax may still do in LSP

Syntax remains appropriate for protocol/presentation work that does not establish semantic truth, for example:

- recovering the import path under the cursor before asking compiler module queries for its target;
- detecting completion/signature syntactic context;
- lexer-driven semantic token presentation where explicitly designed;
- rendering source labels, snippets, ranges, and markdown;
- providing syntax-only completion when no canonical semantic source is available, provided it does not claim compiler semantic identity.

The dividing line is authority: syntax can identify *what the user is pointing at*; compiler products decide *what it means*.

## Forbidden LSP architecture

Do not reintroduce any of the following under new names:

```text
phalcom-lsp/src/semantic/
phalcom-lsp/src/index.rs as semantic truth
LSP SemanticDb / SemanticEngine
LSP ScopeGraph / ModuleGraph
LSP dispatch or type/value inference
request-time import resolution from URI/path spelling
request-time filesystem reads for semantic lookup
syntax-fabricated definition/reference identity
parallel compiler-to-LSP canonical ID translation tables
```

The `semantic_boundary` test exists specifically to make these regressions mechanical failures.

## Verification anchors

Use focused tests while iterating, then the retirement gate before declaring architectural work complete.

Current high-value anchors include:

```sh
cargo fmt --all -- --check
cargo check -p phalcom-semantic
cargo check -p phalcom-modules
cargo check -p phalcom-lsp --lib
cargo test -p phalcom-semantic
cargo test -p phalcom-modules
cargo test -p phalcom-lsp
cargo test -p phalcom-lsp --test semantic_boundary
```

For module/import editor work, also run the focused `module_navigation` and `imported_binding_resolution` tests.

## Review questions

Before adding semantic behavior, answer:

- Which compiler-owned product is authoritative?
- What is the canonical semantic identity?
- Which revision/generation owns the evidence?
- Is this source range a definition, a reference, or only syntax provenance?
- What happens when the snapshot is stale or unmapped?
- Does this change preserve clean/incremental equivalence?
- Can the LSP implementation be reduced to query + protocol projection?
- Is there a boundary test preventing a second semantic world from returning?
