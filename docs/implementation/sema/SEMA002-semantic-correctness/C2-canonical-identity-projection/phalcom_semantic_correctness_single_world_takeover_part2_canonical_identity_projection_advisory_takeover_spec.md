# Phalcom Semantic Correctness / Single-World Takeover — Part 2 of 3
# Canonical Semantic Identity, Projection, and Advisory Evidence Takeover

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this specification task-by-task. Every task below is independently reviewable and test-gated.
>
> **Implementation order:** Part 1 — Formal Semantic Epistemic Foundation, together with its corrections/amendments document, is a hard prerequisite. Part 3 must not begin until this Part 2 release gate passes.

**Goal:** Eliminate the architectural condition in which the compiler and LSP maintain different semantic identities, source-occurrence models, runtime-shape inference graphs, and presentation lookup paths. After Part 2, `phalcom-semantic` owns one canonical source-semantic index, one canonical attachment model for formal and advisory facts, and one compiler-owned advisory abstract-interpretation subsystem. `phalcom-lsp` may still own scheduling, URI/protocol adaptation, and the transitional publication wrapper needed for Part 3, but it no longer owns semantic truth.

**Architecture:** Formal type knowledge remains exactly the epistemic model established by Part 1. Advisory runtime-shape knowledge becomes a separate compiler-owned abstract domain under `phalcom-semantic::advisory`; it may enrich editor presentation but can never participate as a hard premise in formal subtype checking, formal diagnostics, proof discharge, or optimizer correctness. Source identity is split deliberately into cross-revision semantic identities (`ModuleId`, `DeclarationId`, `CallableId`, `FieldId`) and snapshot-scoped source-site identities for local bindings, expressions, and token occurrences. Immutable `SemanticSnapshot` publishes both formal products and advisory/source-index products, so presentation and LSP queries resolve against one snapshot rather than reconciling two semantic databases.

**Tech stack:** Rust workspace; `phalcom-modules` canonical module/project identity; `phalcom-semantic` formal checker, query DB, `SemanticWorkspaceSession`, immutable snapshots, type store, dispatch surfaces; compiler-owned advisory abstract interpretation; `phalcom-lsp` URI/protocol adapters and asynchronous worker; Cargo unit/integration tests; structural performance counters and fingerprint assertions.

**Normative predecessor specifications:**

- `phalcom_semantic_correctness_single_world_takeover_part1_formal_epistemic_foundation_spec.md`
- `phalcom_semantic_correctness_part1_corrections_and_amendments.md`

**Repository baseline inspected for this specification:** `aureat/phalcom-lang` `main` at commit `a3f932e01118053265378e678b0dbaef2b9ceab8` (`Fix semantic authority composition and tail let returns`, 2026-08-24). This baseline is newer than the Part 1 archaeology baseline. It already contains `SemanticWorkspaceSession`, immutable formal snapshots, an initial pure `SemanticPresentationIndex`, and a bridge from LSP to compiler formal snapshots. It still contains a complete independent advisory semantic engine in `phalcom-lsp/src/semantic/`.

---

# 1. Part 2 scope and hard boundary

Part 2 implements the architectural work previously labeled SC-10 through SC-13:

1. **SC-10 — Canonical compiler-owned source semantic identity.**
   Bindings, expressions, declarations, callables, fields, module references, and exact source occurrences must be attachable through compiler-owned identities. LSP-local semantic IDs cease to be semantic authority.

2. **SC-11 — Compiler-owned semantic projection/index publication.**
   Formal facts must be discoverable by source position and semantic target without re-analysis, string matching, or scanning every callable body. Presentation is a projection over machine-readable compiler facts, not a second semantic representation.

3. **SC-12 — Compiler-owned advisory evidence subsystem.**
   Preserve the useful runtime-shape, collection-shape, method-family, local-flow, field, parameter, and interprocedural advisory capabilities currently implemented in `phalcom-lsp/src/semantic`, but move their ownership into `phalcom-semantic` and rebuild them over canonical compiler identities/products.

4. **SC-13 — Advisory authority takeover and LSP semantic demotion.**
   Remove/demote the duplicate LSP scope graph, occurrence identity, class/member identity, dispatch authority, module dependency graph, advisory invalidation graph, and type/shape inference authority. LSP retains adapters and orchestration only.

Part 2 explicitly does **not** complete the final workspace lifecycle cutover. The following remain Part 3 responsibilities:

- deleting production `run_static_workspace_analysis(...)` and any cold-reconstruction route still used by LSP scheduling;
- making one compiler `SemanticWorkspaceSession` the sole long-lived production worker/session object;
- final migration of every hover/completion/definition/references/inlay/token/diagnostic handler to direct compiler snapshot APIs;
- deletion of the remaining transitional LSP semantic publication wrapper;
- project/open/close/remove/rename lifecycle finalization;
- final end-to-end single-world, cold-vs-incremental, LSP latency, and lifecycle release gates.

This division is intentional. Part 2 moves **semantic ownership** first. Part 3 moves **production lifecycle and all protocol consumers** onto that ownership.

---

# 2. Prerequisite contract from Part 1

An implementation agent must treat the following as already merged, even if the repository baseline inspected here predates the final Part 1 patch:

```text
TypeKnowledge
    formal only: Established / Assumed / Unknown / Dynamic

BindingState
    persistent contract + current knowledge + consistency + causal invalidity

ExpectedType
    contextual, non-evidentiary

relation layer
    explicit outcomes; no advisory authority gate

FlowState
    sole owner of current formal binding facts

TypedExpression / ExpressionAnalysis
    formal knowledge, analysis status and causal invalidity are orthogonal

call checker
    exact call-result facts are established only from sound derivation

UnknownReason
    distinguishes legitimate absence from blocked/implementation gaps

ExplanationArena
    preserves actual epistemic status/origin

SemanticDb fingerprints
    include semantically observable epistemic/contract state
```

The Part 1 amendments are normative where they refine generic-inference conflict representation, generic-result evidence support, causal suppression, and fingerprints.

## 2.1 Forbidden shortcut

Do **not** implement advisory takeover by introducing any of these:

```rust
TypeKnowledge::Advisory(...)
EvidenceStatus::Advisory
TypeKnowledge::Known(... advisory confidence ...)
TypeKnowledge::Assumed(... from ValueShape ...)
```

`Assumed` is a formal premise accepted from developer/runtime-facing contract semantics under Part 1 rules. It is not a synonym for “the editor guessed this”. Advisory evidence remains structurally outside `TypeKnowledge`.

---

# 3. Current repository state: what is already correct

The current repository has several foundations that Part 2 must **reuse rather than replace**.

## 3.1 Canonical declaration-level identities already exist

`phalcom-semantic/src/identity.rs` already defines/re-exports:

```rust
ModuleId
DeclarationId
CallableId {
    owner: DeclarationId,
    selector: Selector,
    side: DispatchSide,
}
FieldId {
    owner: DeclarationId,
    name: Box<str>,
    side: DispatchSide,
}
SnapshotId {
    workspace: WorkspaceId,
    revision: SemanticRevision,
    store: TypeStoreId,
}
```

These are the authoritative identities for modules, declarations, callables and fields. Part 2 must not create another compiler-level `ClassId`, string-selector callable ID, URI module ID, or independent dispatch-side enum.

## 3.2 Snapshot-local formal IDs already exist and should remain product-local

The same file currently contains:

```rust
BindingId(pub u32)
BodyId(pub u32)
LocalExpressionId(pub u32)
ExpressionId { owner: BodyId, local: LocalExpressionId }
```

These IDs are appropriate as dense IDs inside checker products. They are **not** cross-revision source identity and must not be advertised as such. Part 2 adds a source-site attachment layer instead of destabilizing checker internals by trying to make every local ordinal globally persistent.

## 3.3 `SemanticWorkspaceSession` is already compiler-owned

`phalcom-semantic/src/session.rs` now owns:

- `SemanticDb`;
- one `TypeStore` retained across revisions;
- base universe declarations/hierarchy/dispatch/signatures;
- source products and fingerprints;
- last published snapshot and last-known-good snapshot.

Part 2 extends this session's build/publication path with source-site and advisory products. It does not create a second session class.

## 3.4 `SemanticSnapshot` is already the correct publication boundary

`phalcom-semantic/src/snapshot.rs` already publishes immutable compiler products under one `SnapshotId`, including sources, surfaces, dispatch, signatures, declarations, hierarchy, diagnostics, semantic graph, and `CallableAnalysis` products.

Part 2 adds source/advisory indexes here. It does not introduce a parallel compiler `EditorSemanticSnapshot` with its own formal state.

## 3.5 A pure formal presenter already exists

`phalcom-semantic/src/presentation.rs` already provides `TypePresenter`, `FormalPresentation`, `FormalSiteId`, `FormalTypeSite`, and `SemanticPresentationIndex`. Tests already verify that it can project `CallableAnalysis` without running analysis.

This is a good seam, but the current index is incomplete and too presentation-oriented:

- it is constructed per callable rather than published in `SemanticSnapshot`;
- it stores already-rendered strings rather than machine-readable semantic fact attachment;
- binding lookup uses exact `(ModuleId, SourceRange)`;
- expression lookup scans a vector of sites for the module;
- it does not cover canonical source occurrences or advisory attachments.

Part 2 evolves this seam rather than deleting it.

---

# 4. Current repository state: the duplicate semantic world that must be dismantled

`phalcom-lsp/src/semantic/` is still a second semantic system. This is not merely a presentation cache.

## 4.1 Duplicate identity authority

`phalcom-lsp/src/semantic/ids.rs` currently defines:

```text
ModuleId(String URI-like key)
ClassId(ModuleId, name)
CallableId(ClassId, selector String, DispatchSide)
FieldId(ClassId, name, DispatchSide)
DispatchSide
```

`DocumentModuleMap` stores both canonical `phalcom_modules::ModuleId` and a second `lsp_by_uri`/`uri_by_lsp` key space used by the advisory tables.

This forces later bridges to compare names and selector strings to rediscover compiler identities.

## 4.2 Duplicate lexical binding authority

`phalcom-lsp/src/semantic/scope.rs` walks the AST and independently builds:

- `ScopeId`;
- `BindingId`;
- binding kinds/mutability;
- source-order name visibility;
- imports/classes/module bindings;
- nested method/block/for scopes;
- destructuring bindings;
- nearest-scope name resolution.

This overlaps formal checker/source semantics and assigns a different binding identity from `phalcom-semantic::BindingId`.

## 4.3 Duplicate occurrence authority

`phalcom-lsp/src/semantic/occurrence.rs` performs another AST traversal to build editor token occurrences and maps them to the LSP-local identity universe. Some targets are only unresolved spellings, and import module references can be manufactured from path strings rather than canonical linked identities.

The interval-selection implementation is useful and should be ported; its ownership is wrong.

## 4.4 Duplicate advisory fact and flow authority

`phalcom-lsp/src/semantic/facts.rs`, `analyzer.rs`, `flow.rs`, `callable.rs`, and `infer.rs` implement a substantial abstract interpreter:

- runtime `ValueShape` domain;
- exact/local/interprocedural/heuristic confidence;
- compact provenance;
- local binding flow;
- field observations;
- parameter contributions;
- collection and record shapes;
- callable/method/method-family shapes;
- bounded unions;
- callable summaries and interprocedural propagation.

This functionality is valuable. It should survive, but under compiler ownership and canonical identities.

## 4.5 Duplicate dispatch/surface/module/invalidation authority

The LSP semantic directory also owns its own:

- source `ClassSurface` / `MemberSurface` model;
- receiver dispatch resolver;
- module graph and reverse dependency closure;
- source-delta classification/invalidation;
- callable dependency/dependent graph;
- mutable `SemanticEngine` transaction state.

The compiler now already has canonical declaration surfaces, dispatch, semantic graph, linked module products, DB query dependencies, fingerprints, and workspace session invalidation. Keeping the LSP copies means two worlds can disagree even if formal/advisory epistemic categories are correctly separated.

## 4.6 Current bridge proves why Part 2 is necessary

`phalcom-lsp/src/semantic/snapshot.rs` currently nests a compiler `StaticSemanticSnapshot` inside an LSP advisory snapshot and then implements bridges such as:

- scan every compiler callable analysis for a module/body range to answer formal binding/expression presentation;
- compare owner module/name, encoded selector strings, and dispatch side to find a compiler callable signature corresponding to an LSP callable;
- query LSP class surfaces/dispatch for completion while separately querying formal compiler types for presentation.

That is reconciliation between two semantic worlds, not a single semantic model.

---

# 5. Part 2 architectural laws

These are mandatory implementation invariants.

## 5.1 One owner per semantic concept

After Part 2:

| Concept | Sole semantic owner |
| --- | --- |
| Project/module identity and linking | `phalcom-modules` |
| Declaration/callable/field identity | `phalcom-semantic` using `phalcom-modules` IDs |
| Formal type knowledge | `phalcom-semantic` formal checker |
| Formal diagnostics/proofs | `phalcom-semantic` |
| Lexical source semantic identity | `phalcom-semantic::source_index` |
| Exact semantic occurrences | `phalcom-semantic::source_index` |
| Canonical dispatch/surfaces | `phalcom-semantic` |
| Advisory runtime-shape facts | `phalcom-semantic::advisory` |
| Advisory propagation/invalidation dependencies | compiler `SemanticDb` / canonical products |
| Formatting into LSP prose/markup | `phalcom-lsp` presentation adapters |
| URI ↔ canonical module mapping | LSP document boundary using canonical module IDs |
| Worker/debounce/open-document orchestration | temporarily `phalcom-lsp` until Part 3 |

## 5.2 Formal and advisory are parallel channels, not a confidence ladder

There is no total ordering like:

```text
Heuristic < Advisory < Assumed < Established
```

Formal and advisory answer different questions.

- Formal: “What type relation can the checker justify or accept as a formal premise?”
- Advisory: “What runtime/value shape is useful to predict for tooling from current program evidence?”

An advisory `Instance(CellNum)` can coexist with formal `Unknown`. It does not turn `Unknown` into `Assumed(CellNum)`.

An advisory `Instance(Int)` can disagree with formal `Established(CellNum)`. The checker remains authoritative. Tooling may hide, label, or surface the advisory disagreement for debugging, but the advisory fact cannot alter compiler acceptance.

## 5.3 Source range is attachment, never semantic identity

`SourceRange` moves under edits and can collide for nested constructs. It is always metadata/indexing input, not the identity of a binding, expression, callable or occurrence.

## 5.4 Snapshot-local IDs must not escape without a snapshot guard

Dense site IDs are efficient and correct inside one immutable snapshot. They become unsafe if callers carry them into a later snapshot.

Every public handle that can survive outside a borrowed `&SemanticSnapshot` must include or validate `SnapshotId`.

## 5.5 Presentation cannot own inference

The presentation/index layer may:

- select a site;
- retrieve formal/advisory facts;
- format canonical type/advisory shape text;
- expose provenance/status.

It may not:

- run subtype checking;
- infer receiver type;
- dispatch a call to discover a new semantic target;
- create an advisory fact from syntax;
- fall back from formal unknown to a fake formal known type.

## 5.6 Advisory computation uses formal products but cannot feed them back in the same revision

The dependency direction is:

```text
parsed/module/surface/formal products
        ↓
source semantic index
        ↓
advisory derived products
        ↓
presentation / LSP
```

Never:

```text
advisory fact -> formal checker premise
```

This acyclic authority direction is the central safety property of Part 2.

---

# 6. Identity model: cross-revision targets versus snapshot-scoped sites

The specification deliberately uses two identity lifetimes.

## 6.1 Cross-revision semantic target identities

Retain existing canonical identities:

```rust
pub use phalcom_modules::ModuleId;

pub struct DeclarationId { /* existing */ }
pub struct CallableId {
    pub owner: DeclarationId,
    pub selector: Selector,
    pub side: DispatchSide,
}
pub struct FieldId {
    pub owner: DeclarationId,
    pub name: Box<str>,
    pub side: DispatchSide,
}
```

They identify language-level entities whose identity is meaningful across snapshots as long as the owning module/declaration continues to exist.

## 6.2 Snapshot-scoped source-site identity

Add to `phalcom-semantic/src/identity.rs`:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceOwner {
    Module(ModuleId),
    Callable(CallableId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceSiteLocalId(pub u32);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceSiteId {
    pub owner: SourceOwner,
    pub local: SourceSiteLocalId,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceSiteRef {
    pub snapshot: SnapshotId,
    pub site: SourceSiteId,
}
```

`SourceSiteId` is valid only inside the snapshot that owns the index. `SourceSiteRef` is the externally carryable handle.

The owner-qualified shape is deliberate. A site local ordinal is allocated inside a module-level or callable-level source owner, so an unrelated edit in an earlier callable does not mechanically renumber every later callable's site namespace. This improves reuse without promising cross-revision local identity.

Do **not** put `SnapshotId` into every internal map key. The immutable `SemanticSnapshot` already provides the ownership boundary; dense internal keys keep memory and comparisons cheap.

## 6.3 Site kinds

Create `phalcom-semantic/src/source_index/site.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceSiteKind {
    Module,
    Declaration(DeclarationId),
    Callable(CallableId),
    Field(FieldId),
    BindingDeclaration,
    Expression,
    Occurrence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceSite {
    pub id: SourceSiteId,
    pub range: SourceRange,
    pub kind: SourceSiteKind,
}

```

`SourceOwner` lives in `identity.rs` with `SourceSiteId`; `source_index::site` consumes it. This keeps the central identity module independent of the source-index implementation module.

A site is a location identity. It is not automatically a semantic target.

## 6.4 Semantic target model

Create:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticTargetId {
    Binding(SourceSiteId),
    Declaration(DeclarationId),
    Callable(CallableId),
    Field(FieldId),
    Module(ModuleId),
}
```

For local and top-level source bindings, the declaration site itself is the target. This avoids inventing a second binding-ID universe.

For a callable-local formal binding, `SourceSemanticIndex` stores the bridge:

```rust
BTreeMap<(CallableId, BindingId), SourceSiteId>
```

For formal expressions:

```rust
BTreeMap<(CallableId, ExpressionId), SourceSiteId>
```

These maps are snapshot-local and never string-based.

## 6.5 Why not make `BindingId` globally stable?

Do not attempt content-addressed local binding IDs in Part 2. Local identity under arbitrary edits, recovery syntax, duplicated declarations, and moved blocks is a separate incremental-identity problem. The current compiler already has correct immutable snapshots and query fingerprints.

The contract is instead:

- `BindingId`/`ExpressionId`: dense checker-product identity;
- `SourceSiteId`: dense source-index identity in the same snapshot;
- `SourceSiteRef`: snapshot-guarded external reference;
- semantic target lookup is re-issued against a new snapshot after edits.

This is honest, deterministic and safe.

---

# 7. Compiler-owned source semantic index

Create a focused module tree:

```text
phalcom-semantic/src/source_index/
    mod.rs
    site.rs
    scope.rs
    occurrence.rs
    interval.rs
    builder.rs
```

Do not put this implementation back into `presentation.rs`; source semantics are useful to navigation, references, rename, completion, diagnostics and advisory inference independently of formatting.

## 7.1 `SourceSemanticIndex`

Target structure:

```rust
#[derive(Clone, Debug, Default)]
pub struct SourceSemanticIndex {
    modules: BTreeMap<ModuleId, Arc<ModuleSourceIndex>>,
    target_occurrences: BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
}

#[derive(Clone, Debug)]
pub struct ModuleSourceIndex {
    pub structure: Arc<ModuleSourceStructure>,
    pub attachments: BTreeMap<CallableId, Arc<CallableSourceAttachment>>,
}

#[derive(Clone, Debug)]
pub struct ModuleSourceStructure {
    pub module: ModuleId,
    pub syntax_sites: BTreeMap<SourceOwner, Arc<[SourceSite]>>,
    pub scopes: SourceScopeIndex,
    pub occurrences: OccurrenceIndex,
    pub exact_targets: BTreeMap<SourceSiteId, SemanticTargetId>,
}

#[derive(Clone, Debug)]
pub struct CallableSourceAttachment {
    pub callable: CallableId,
    pub expression_sites: Arc<[SourceSite]>,
    pub formal_bindings: BTreeMap<BindingId, SourceSiteId>,
    pub formal_expressions: BTreeMap<ExpressionId, SourceSiteId>,
    pub exact_targets: BTreeMap<SourceSiteId, SemanticTargetId>,
}
```

`ModuleSourceStructure` is syntax/link owned and can be reused without type checking. `CallableSourceAttachment` is formal-product owned. `SourceSemanticIndex` exposes one effective exact-target view over both maps and builds the target → occurrence reverse index from that effective view. This is what permits a type-only reanalysis to reuse lexical/occurrence structure while changing a call target or expression fact.

If `SemanticTargetId` containing `SourceSiteId` makes one workspace-level reverse map awkward for binding targets, shard binding-target reverse occurrences with their module and keep a thin workspace facade. The semantic requirement is deterministic indexed target lookup, not this exact physical container.

## 7.2 Deterministic site allocation

For each module revision, allocate independently inside each `SourceOwner` namespace:

1. Traverse syntax in stable source order.
2. Create syntax-owned declaration/token/occurrence sites in `(range.start, range.end, site-kind priority)` order.
3. Assign those syntax site local IDs densely from zero.
4. Record the syntax-site count for that owner.
5. After formal checker products are available, append formal-expression sites after the syntax-owned range, in deterministic `(range.start, range.end, ExpressionId)` order.
6. Binding attachments reuse the already allocated binding declaration site; they do not allocate another binding site.

This allows the source-structure product to be cached independently from formal attachments: a type-only reanalysis can reuse all syntax site IDs and only rebuild the callable attachment tail. Never allocate externally observable site numbers from `HashMap` iteration.

A source edit may change ranges and therefore site local IDs inside the edited owner. This is allowed. A snapshot local ID has no cross-revision stability promise. Unrelated callable owners do not renumber merely because an earlier callable changed.

Declaration occurrences reuse their declaration site rather than allocating a redundant occurrence-only site at the same token. Reads/writes/calls/references allocate occurrence sites; their exact target is stored in the effective exact-target map rather than inside the syntax occurrence record. This keeps “where is the declaration?” and “this declaration occurrence” identical without conflating reference occurrences with their targets.

## 7.3 Site-table memory rule

Store `ModuleId` once in `ModuleSourceIndex` if profiling shows repeated module clones dominate memory. `SourceSiteId` carries a canonical `SourceOwner` for identity safety; hot per-owner interval vectors may store only `SourceSiteLocalId` and recover the owner from the containing shard.

Part 2 is not permission to create a heavyweight object graph per token.

---

# 8. Compiler-owned lexical scope index

The existing LSP `scope.rs` contains useful behavior but wrong ownership. Port the semantics into `phalcom-semantic/src/source_index/scope.rs` using canonical IDs.

## 8.1 Target model

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceScopeId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceBindingKind {
    TopLevelLet,
    TopLevelConst,
    LocalLet,
    LocalConst,
    MethodParameter,
    SetterParameter,
    IndexParameter,
    ClosureParameter,
    ForBinding,
    Destructure,
    Import,
}

#[derive(Clone, Debug)]
pub struct SourceBindingInfo {
    pub declaration_site: SourceSiteId,
    pub scope: SourceScopeId,
    pub name: Box<str>,
    pub kind: SourceBindingKind,
    pub declaration_range: SourceRange,
    pub mutable: bool,
    pub redeclaration_of: Option<SourceSiteId>,
}
```

Formal checker binding IDs are intentionally **not stored here**. `SourceBindingInfo` is part of the reusable syntax-owned structure; `(BindingId -> SourceSiteId)` lives in `CallableSourceAttachment` after formal analysis.

## 8.2 Same-scope redeclaration semantics

Part 1 requires same-scope redeclaration to diagnose and preserve the first binding as the lexical target.

Source indexing must preserve both facts:

- the duplicate declaration gets its own occurrence/site so diagnostics and editor selection can point at it;
- name resolution after that point still resolves to the first binding target under the language recovery rule;
- `redeclaration_of` records the primary declaration site.

Do not silently omit the duplicate site simply because the binding map keeps the first entry.

## 8.3 Source-order visibility

Resolution must retain the existing source-order rule: a binding is visible only at/after its declaration under the language's current lexical semantics.

Store declaration starts and scope parent relationships so `visible_bindings_at(offset)` and `resolve_name(scope, name, offset)` are pure snapshot reads.

## 8.4 Imports and classes

Do not construct module targets using raw spelling such as `ModuleId::new(path.to_string())`.

The builder receives compiler-owned linked/module query products and resolves import targets through `ModuleQueryProducts` / `ModuleQueryFacade` / `LinkedTypeResolver` as appropriate.

Class/type declaration references use canonical `DeclarationId`.

## 8.5 No third scope graph

The formal checker's internal lexical environment and the source query scope index serve different representations but must share target identity attachments. Do not add a third “presentation scope” graph.

---

# 9. Exact compiler-owned occurrence index

Port the good interval-indexing concept from `phalcom-lsp/src/semantic/occurrence.rs` into `phalcom-semantic/src/source_index/occurrence.rs`.

## 9.1 Occurrence model

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OccurrenceKind {
    Binding,
    Parameter,
    Declaration,
    Module,
    Member,
    Field,
    Operator,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OccurrenceRole {
    Declaration,
    Read,
    Write,
    Call,
    Reference,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticOccurrence {
    pub site: SourceSiteId,
    pub range: SourceRange,
    pub kind: OccurrenceKind,
    pub role: OccurrenceRole,
    pub owner: SourceOwner,
    pub hint: Option<OccurrenceHint>,
}

pub struct OccurrenceView<'a> {
    pub occurrence: &'a SemanticOccurrence,
    pub target: Option<&'a SemanticTargetId>,
}
```

The occurrence record itself is syntax-owned. Exact/canonical target attachment lives in `ModuleSourceStructure::exact_targets` for lexical/module/declaration facts and in `CallableSourceAttachment::exact_targets` for checker-resolved call/member facts. `SourceSemanticIndex::occurrence_at` returns the merged `OccurrenceView`.

A missing exact target means the compiler cannot resolve it from lexical/module/formal semantics. This is preferable to fabricating a string-identity target.

Advisory resolution must **not mutate either exact-target map** and advisory predictions **must not be inserted into `exact_targets`**. If advisory analysis can predict a member/callable when exact/formal resolution is unavailable, publish that prediction separately in `AdvisoryWorkspace::targets` as described in §21. This preserves the distinction between exact source identity and advisory navigation fallback.

If unresolved spelling is useful for completion/diagnostics, store it only as non-authoritative hint data:

```rust
pub enum OccurrenceHint {
    MemberSelector(Selector),
    Operator(Box<str>),
    Name(Box<str>),
}
```

Do not put unresolved spellings in `SemanticTargetId`.

## 9.2 Interval selection algorithm

Retain the current efficient shape:

```text
occurrences sorted by range.start, then range length, then kind priority
max_end_prefix[i] = max(end of occurrences[0..=i])
```

`occurrence_at(offset)`:

1. binary-search the first item with `start > offset`;
2. walk backward only while `max_end_prefix[index] > offset`;
3. among containing occurrences choose the shortest range;
4. break equal-length ties by documented semantic priority and source start.

Do not replace this with a full linear scan.

## 9.3 Expression selection uses the same interval primitive

The current compiler `SemanticPresentationIndex::find_expression_at` linearly scans expression sites. Replace it with the same bounded interval index primitive.

`interval.rs` should be generic enough to index site local IDs by `SourceRange`, but not generic for its own sake. A small internal `RangeIndex<T: Copy>` is sufficient.

## 9.4 References index

At source-index finalization time, merge syntax-owned and callable-formal exact-target overlays, then construct target → occurrence lists in deterministic source order. A references query should not scan every source occurrence in every file.

For bindings, the target is the declaration `SourceSiteId`. For cross-module declaration/callable/field/module targets, use canonical IDs. Advisory target predictions are excluded from this **reverse exact-reference index** unless a future query explicitly asks for advisory references.

---

# 10. Attaching formal checker products to source sites

The source index is built from both syntax and formal products. It must not infer formal meaning itself.

## 10.1 Binding attachment

For every `CallableAnalysis.bindings[(BindingId)]`:

1. identify the corresponding source declaration site by callable owner + binding declaration range + binding name;
2. require a unique match among lexical declaration sites inside the callable;
3. record `(CallableId, BindingId) -> SourceSiteId`;
4. attach the formal `BindingState` by reference/key, not by copying its type into a new semantic fact.

If a unique attachment cannot be made, publish an explicit source-index incident/blocked attachment record. Do not choose the first same-name range heuristically.

A same-scope redeclaration must remain attachable because §8.2 records both primary and duplicate sites.

## 10.2 Expression attachment

For each `ExpressionAnalysis`:

1. create or identify the expression source site from its exact range and callable owner;
2. record `(CallableId, ExpressionId) -> SourceSiteId`;
3. add to the module expression interval index;
4. retain the formal product key so queries can retrieve `knowledge`, status, causal invalidity, denotation and explanation from the owning `CallableAnalysis`.

Do not copy only `TypeId`/string and discard Part 1's orthogonal states.

## 10.3 Callable attachment

`CallableId` is already canonical. Source callable declaration ranges should map directly to it during source-surface collection. No later owner-name + selector-string reconciliation is permitted.

## 10.4 Field attachment

Use `DeclarationSurface::get_field_id(side, name)` / canonical `FieldId` generated by compiler surfaces. Do not reconstruct a field ID in LSP.

## 10.5 Call-site target attachment

Part 1 requires the call checker to retain real resolved `CallableId`. Part 2 must expose that resolved target through source attachment.

If current `CallResolutionId` lacks a published resolution table, extend `CallableAnalysis` with a compact call-resolution product:

```rust
pub struct CallResolution {
    pub id: CallResolutionId,
    pub expression: ExpressionId,
    pub callable: Option<CallableId>,
    pub status: CallResolutionStatus,
}

pub enum CallResolutionStatus {
    Resolved,
    Dynamic,
    Missing,
    Ambiguous(Arc<[CallableId]>),
    Blocked,
}
```

Then `CallableSourceAttachment::exact_targets` can attach canonical `CallableId` to the member/call occurrence site when formal resolution is `Resolved`; otherwise the occurrence remains exact-targetless.

If advisory dispatch later predicts a callable for an exact-targetless occurrence, store that prediction only in the advisory target attachment map; never promote it into an exact-target overlay.

Do not rerun dispatch in the presentation/source-index builder just to recover the exact target.

---

# 11. Machine-readable formal site records

The current `FormalTypeSite` stores `FormalPresentation`, which is already rendered. Part 2 separates semantic attachment from textual projection.

## 11.1 Replace “presentation is the index” with “facts are indexed, presentation is projected”

Introduce:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FormalFactRef {
    Expression {
        callable: CallableId,
        expression: ExpressionId,
    },
    Binding {
        callable: CallableId,
        binding: BindingId,
    },
    Callable(CallableId),
    Field(FieldId),
}

#[derive(Clone, Debug)]
pub struct FormalSiteAttachment {
    pub site: SourceSiteId,
    pub fact: FormalFactRef,
}
```

`SourceSemanticIndex` owns the source-site lookup. `SemanticSnapshot` owns the actual formal products.

## 11.2 Evolve `SemanticPresentationIndex`

Keep the public concept but redefine it as a pure snapshot projection facade, or rename internally to `SemanticProjectionIndex` while retaining a compatibility alias during migration.

Recommended API:

```rust
impl SemanticSnapshot {
    pub fn site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite>;
    pub fn occurrence_at(&self, module: &ModuleId, offset: usize) -> Option<OccurrenceView<'_>>;
    pub fn formal_fact_at(&self, module: &ModuleId, offset: usize) -> Option<FormalFactView<'_>>;
    pub fn advisory_fact_at(&self, module: &ModuleId, offset: usize) -> Option<&AdvisoryFact>;
}

pub struct SemanticPresenter<'a> {
    snapshot: &'a SemanticSnapshot,
}
```

Formatting happens only when requested:

```rust
presenter.present_formal(formal_view)
presenter.present_advisory(advisory_fact)
```

## 11.3 Preserve epistemic status in presentation

Part 1's `Established` versus `Assumed` distinction is semantically meaningful. A machine-readable formal view must retain it.

Do not reduce a formal fact to `FormalPresentation::Known("Int")` before the LSP has a chance to distinguish:

```text
Established(Int)
Assumed(Int)
```

Recommended presentation payload:

```rust
pub enum FormalPresentation {
    Known {
        text: String,
        status: EvidenceStatus,
    },
    Dynamic,
    Unknown,
    Invalid,
    Blocked,
    Cancelled,
    BudgetExceeded,
    InternalFailure,
    Partial,
}
```

If Part 1 already lands a different equivalent representation, preserve it. Do not reintroduce the old collapsed variant merely for compatibility.

---

# 12. Snapshot publication changes

Modify `phalcom-semantic/src/snapshot.rs` to publish source and advisory products.

Target fields:

```rust
pub struct SemanticSnapshot {
    // existing fields ...
    pub source_index: Arc<SourceSemanticIndex>,
    pub advisory: Arc<AdvisoryWorkspace>,
}
```

The exact name `AdvisoryWorkspace` may be `AdvisoryIndex` if implementation is predominantly immutable maps. Pick one name and use it consistently.

## 12.1 Snapshot atomicity

Formal products, source attachments and advisory products published in one `SemanticSnapshot` must all correspond to the same:

```text
WorkspaceId
SemanticRevision
TypeStoreId
source generation/input set
```

Never publish a new source index with the previous advisory maps or vice versa unless the reused `Arc` is proven semantically unchanged by fingerprint/input identity.

## 12.2 Reuse is encouraged

If a module's source semantic product and relevant formal dependencies are unchanged, reuse its `Arc<ModuleSourceIndex>` across snapshots.

If advisory input fingerprints are unchanged, reuse advisory product `Arc`s.

Immutability plus `Arc` identity is a useful test oracle for incrementality.

## 12.3 No duplicate rendered string cache by default

Do not store rendered type strings for every site in the snapshot. `TypeId` + epistemic state is already canonical and string rendering is cheap compared with semantic analysis. Add a presentation-string cache only if profiling later shows it matters.

---

# 13. Advisory domain: rename and relocate the LSP runtime-shape model

Create:

```text
phalcom-semantic/src/advisory/
    mod.rs
    fact.rs
    shape.rs
    provenance.rs
    analyzer.rs
    flow.rs
    summary.rs
    solver.rs
    query.rs
```

The migration is a **semantic port**, not a filesystem copy. Every identity and dependency must be recast in compiler terms.

## 13.1 Rename `InferredValue`

The old name invites confusion with formal inference. Use:

```rust
pub struct AdvisoryFact
```

or `AdvisoryValueFact`. This spec uses `AdvisoryFact`.

## 13.2 Target shape domain

Preserve the useful domain, replacing LSP-local IDs with canonical IDs:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValueShape {
    Unknown,
    Instance(DeclarationId),
    ClassObject(DeclarationId),
    Module(ModuleId),
    Tuple(Arc<[ValueShape]>),
    ExactList(Arc<[ValueShape]>),
    Record(Arc<[(Box<str>, ValueShape)]>),
    List(Box<ValueShape>),
    Set(Box<ValueShape>),
    Map {
        key: Box<ValueShape>,
        value: Box<ValueShape>,
    },
    Range(Box<ValueShape>),
    Callable(CallableId),
    Selector(Selector),
    SelectorPattern(SelectorPattern),
    Family {
        receiver: Box<ValueShape>,
        spec: NormalizedSelectorSpec,
    },
    Method(CallableId),
    MethodFamily(Arc<CapturedMethodFamilyShape>),
    BoundMethod {
        receiver: Box<ValueShape>,
        method: CallableId,
    },
    BoundMethodFamily {
        receiver: Box<ValueShape>,
        family: Arc<CapturedMethodFamilyShape>,
    },
    Union(Arc<[ValueShape]>),
}
```

Use `DeclarationId` rather than inventing compiler `ClassId`; a class declaration's canonical identity is already `DeclarationId`.

`CapturedMethodFamilyShape` must also lose its dependency on LSP `RestSurface`. Target shape:

```rust
pub struct CapturedMethodFamilyShape {
    pub source_behavior: DeclarationId,
    pub pattern: SelectorPattern,
    pub exact: Box<[(Selector, CallableId)]>,
    pub rest_candidates: Box<[CallableId]>,
}
```

Resolve a rest candidate by consulting the compiler-owned `CallableSignatureTable` / `CallableParameterSemantic.rest: RestMode` for that callable. If repeated signature inspection is proven hot, add a small compiler-owned derived `RestAcceptance` value constructed from the canonical semantic signature; do not port LSP `RestSurface`/`ClassSurface` as a second signature authority.

## 13.3 Bounded union rule

Retain `MAX_SHAPE_UNION = 8` unless benchmarks justify a later change.

Rules:

- equal shapes join to themselves;
- `Unknown` joined with anything is `Unknown`;
- compatible collection structures join recursively;
- exact lists with different lengths widen to list element shape;
- unions flatten/deduplicate deterministically;
- more than 8 incompatible alternatives widens to `Unknown`;
- union order must be canonical/deterministic before hashing/publishing, using an explicit `shape_sort_key`/canonical encoder rather than relying on incidental enum/hash-map ordering.

Do not let a hash-set's iteration order determine a product fingerprint.

Canonicalize advisory record fields by label before equality/join/fingerprinting unless Phalcom's record runtime semantics explicitly make source field order meaningful. Two record literals with the same labels in different source order must not become incompatible advisory shapes merely because the old LSP representation stored a `Vec` in insertion order.

Likewise, captured method-family `exact` tables must be sorted by structural `Selector`, and rest candidates must retain the compiler's documented subclass-to-superclass dispatch order.

## 13.4 Advisory confidence is not formal status

Rename for clarity:

```rust
pub enum AdvisoryConfidence {
    Exact,
    Flow,
    Interprocedural,
    Heuristic,
}
```

The current conservative join (`min`) is acceptable if ordering remains documented:

```text
Exact > Flow > Interprocedural > Heuristic
```

This ordering only governs how much tooling should trust/display an advisory prediction. It has no relation to `EvidenceStatus::{Established, Assumed}`.

---

# 14. Advisory provenance uses canonical sites

Replace raw range-centric `FactOrigin` with compiler-owned site/target provenance.

Target:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AdvisoryOrigin {
    Syntax(SourceSiteId),
    Binding(SourceSiteId),
    Callable(CallableId),
    CallSite(SourceSiteId),
    Constraint(SourceSiteId),
    Field(FieldId),
    FormalFact(FormalFactRef),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryFact {
    pub shape: ValueShape,
    pub literal: Option<AdvisoryLiteral>,
    pub confidence: AdvisoryConfidence,
    pub provenance: SmallVec<[AdvisoryOrigin; 4]>,
}
```

If the workspace avoids `smallvec`, retain `Vec` capped at four; do not add a dependency solely for this representation.

## 14.1 Literal knowledge

The current `known_boolean: Option<bool>` is useful but too special-case in name. Prefer:

```rust
pub enum AdvisoryLiteral {
    Bool(bool),
}
```

Additional literals can be added later if proven useful. Part 2 does not need to become a constant evaluator.

Literal shape construction must obtain builtin declaration identities from the compiler's canonical core/declaration products. Do not repeat the old LSP pattern of constructing “Bool” or another builtin by `(core URI string, class name)` inside advisory code. If the canonical builtin declaration cannot be resolved, the advisory literal fact is blocked/unknown according to advisory status rules; no fake declaration identity is manufactured.

## 14.2 Provenance cap

Retain a strict small cap (current behavior: four unique origins) to prevent interprocedural provenance explosion.

When cap is exceeded, keep deterministic representative origins using an explicit stable provenance sort key (source owner/site order, callable canonical ordering, then origin-kind discriminant). Do not require every nested AST/selector payload to implement `Ord`, and do not make product identity depend on traversal/hash-map order.

---

# 15. Formal-to-advisory projection rules

Advisory analysis may seed itself from formal products, but projection must be explicit and one-way.

## 15.1 Safe projection table

A formal known nominal type can seed a broad advisory instance shape:

```text
Established(Nominal C) -> advisory Instance(C), confidence Exact/Flow depending site
Assumed(Nominal C)     -> advisory Instance(C), but mark origin FormalFact and confidence no stronger than Flow
Unknown                -> no advisory seed from formal channel
Dynamic                -> no concrete advisory seed solely from formal channel
Invalid known fact     -> may seed shape if type knowledge is independently retained,
                          while causal/status metadata stays formal and advisory never repairs validity
```

Applied collection types may seed broad shape containers only when the mapping is canonical and semantically valid:

```text
List<T> -> List(project(T))
Set<T>  -> Set(project(T))
Map<K,V> -> Map(project(K), project(V))
```

If a type argument cannot be projected, use `ValueShape::Unknown` for that component or skip the seed according to the domain rule. Never fabricate an advisory class from a type-form spelling.

## 15.2 Advisory can be more structurally precise than formal type

Example:

```phalcom
let xs: List<Int> = [1, 2, 3]
```

Formal current fact may be `Established(List<Int>)` while advisory can retain `ExactList([Int, Int, Int])` for tooling. This is legitimate because advisory shape is a different abstraction.

## 15.3 Advisory disagreement never emits hard formal diagnostics

No call in `advisory/` may invoke the compiler's hard mismatch diagnostic policy as a consequence of advisory disagreement.

The subsystem may expose an `AdvisoryConflict` record for debugging/telemetry, but it must not enter `SemanticDiagnostic` error output unless a separate future language rule explicitly makes that observation formal evidence.

---

# 16. Advisory flow ownership

Port the useful structured traversal from LSP `flow.rs`, but feed it canonical source/formal products.

## 16.1 One advisory flow traversal per callable product

Preserve the existing architectural strength that one traversal can collect:

- local advisory binding facts;
- normal return advisory fact;
- field-write observations;
- call-site parameter contributions;
- callable dependencies/effects needed by advisory solving.

Do not split these into several independent AST walkers.

## 16.2 Advisory binding keys

Use canonical source binding targets:

```rust
pub type AdvisoryBindingKey = SourceSiteId; // declaration site target
```

Do not introduce `advisory::BindingId`.

## 16.3 Formal flow facts are inputs, not duplicated control authority

Where `CallableAnalysis`/formal flow already provides reachability or an exact type fact, advisory traversal should consume it when practical.

Do not attempt to rewrite Part 1's formal flow engine in Part 2. Advisory may still need a lightweight runtime-shape environment because exact list/record/method-family shapes are not formal types, but its control decisions must not contradict known formal reachability.

## 16.4 Recovery behavior

If an expression is formally blocked/unresolved but the advisory analyzer can derive a harmless shape from syntax, it may retain the advisory result. It must preserve origin/confidence and cannot mark the formal expression Ready.

If syntax/recovery is insufficient, advisory widens to `Unknown` rather than borrowing `Unit`, `Object`, `Never`, or another language type as a sentinel.

---

# 17. Advisory expression analysis and dispatch

Port `phalcom-lsp/src/semantic/analyzer.rs` semantics into `phalcom-semantic/src/advisory/analyzer.rs`, but remove independent semantic authority.

## 17.1 Canonical dispatch only

Advisory receiver/member lookup must use the snapshot's existing compiler `SurfaceDispatchResolver`, `DeclarationSurface`, `MapTypeHierarchy`, and canonical selectors.

Do not port the LSP `DispatchResolver` as a second receiver lookup engine.

For `ValueShape::Instance(declaration)`:

```rust
snapshot.dispatch.resolve_dispatch_with_trace(
    snapshot.hierarchy.as_ref(),
    &declaration,
    DispatchSide::Instance,
    &selector,
)
```

For class objects, use `DispatchSide::Class`.

For formal `Dynamic` or advisory `Unknown`, fail advisory dispatch conservatively.

## 17.2 Resolved call target reuse

If the formal `ExpressionAnalysis` already records a resolved `CallableId`, advisory uses it directly. It should only perform canonical advisory dispatch when formal dispatch is unavailable because the formal expression is outside checker coverage or genuinely dynamic/advisory-only.

## 17.3 Method family behavior

Preserve useful method/method-family shapes, but `CapturedMethodFamilyShape` must store canonical `CallableId` and canonical declaration identity. Exact selector matching uses `phalcom_common::selector` representations; rest acceptance is derived from compiler `CallableSemanticSignature`/`RestMode`, never from an LSP member surface.

## 17.4 No “formal fallback from advisory dispatch”

Advisory resolution of a method cannot populate formal `ExpressionAnalysis.call`. Formal call resolution is compiler checker output only.

---

# 18. Advisory field facts

Port `FieldFacts` and `FieldEvidence`, replacing local field IDs with canonical `FieldId`.

Target:

```rust
pub enum FieldEvidenceKind {
    DeclarationInitializer,
    ConstructorInitialization,
    GeneralWrite,
}

pub struct FieldEvidence {
    pub value: AdvisoryFact,
    pub kind: FieldEvidenceKind,
    pub site: SourceSiteId,
}

pub struct AdvisoryFieldFacts {
    pub joined: BTreeMap<FieldId, AdvisoryFact>,
    pub evidence: BTreeMap<FieldId, Arc<[FieldEvidence]>>,
}
```

## 18.1 Constructor knowledge does not redefine formal field contracts

An advisory constructor write can improve predicted runtime shape, but the formal field contract/type remains owned by `DeclarationSurface` / field signature products.

## 18.2 Deterministic evidence ordering

Sort field evidence by module/site source order before fingerprinting/publication.

---

# 19. Advisory parameter contributions and interprocedural solver

The LSP implementation's contribution-indexed parameter facts are worth preserving because they allow precise removal/replacement of caller contributions.

## 19.1 Canonical parameter slot identity

Use callable identity plus parameter index, not source parameter name, as the primary slot identity:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdvisoryParameterSlot {
    pub callable: CallableId,
    pub index: u32,
}
```

Names are presentation metadata and can change without changing selector slot position in ways that should not silently alias old contributions.

Validate the index against the compiler callable signature.

## 19.2 Contribution source

```rust
pub enum AdvisoryContributionSource {
    Callable(CallableId),
    Module(ModuleId),
}
```

The caller/source identity is canonical. Do not use URI module keys.

## 19.3 Replace-source algorithm

Retain the efficient contribution replacement model:

1. remove all prior `(source -> slots)` contributions;
2. insert the new source's contributions;
3. recompute only touched joined slots;
4. emit a `ParameterFactDelta` only when the joined advisory fact changes;
5. schedule dependent advisory callable products through compiler query dependencies/product fingerprints.

## 19.4 Fixpoint discipline

The advisory solver is allowed to widen. It is not allowed to loop arbitrarily.

Use one of these existing-compatible termination strategies:

- worklist until no product fingerprint changes, with a query budget; or
- SCC-local fixpoint with bounded shape joins.

Do not use an unexplained fixed pass count as correctness behavior.

Because `MAX_SHAPE_UNION` widens large incompatible sets to `Unknown`, the shape domain is finite enough for practical monotone convergence if transfer functions are monotone.

The solver may construct a **transient** adjacency/SCC/worklist from canonical `CallableAnalysis.dependencies` and resolved-call products for the affected computation. That scratch graph dies with the query evaluation. There is **no second graph** of persistent advisory dependencies: it is not published as a second semantic dependency authority. Persistent invalidation/reuse edges remain `SemanticDb` query dependencies.

For recursive SCCs, solve members to a local fixpoint first and publish their advisory products only after the SCC result is coherent. Do not try to express an in-progress recursive cycle as mutually recursive cached `QueryKey::AdvisoryCallable` products if the DB query engine does not support cycle publication; the SCC solver is the cycle boundary.

## 19.5 Monotonicity requirement

Within one fixed source/formal input revision, advisory join/transfer must not oscillate between incomparable shapes. Add tests for recursive mutually calling functions to prove convergence and stable product fingerprints.

---

# 20. Advisory summary products

Port/reshape `CallableSummary` into compiler ownership.

Recommended model:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryCallableSummary {
    pub callable: CallableId,
    pub parameters: Arc<[AdvisoryFact]>,
    pub return_fact: AdvisoryFact,
    pub dependencies: Arc<[CallableId]>,
    pub effects: AdvisorySummaryEffects,
    pub fingerprint: ProductFingerprint,
}
```

Do not include a second advisory “generation” if `SemanticSnapshot.id`/revision already identifies publication. A product may retain its input/product fingerprint for reuse, but a separate semantic generation counter is not authority.

## 20.1 Exact formal return wins as a seed, not as advisory authority

If formal call-result knowledge is `Established(T)`, advisory can seed/project a shape from `T`. A more precise advisory return from syntax/flow may coexist.

If formal return is `Assumed(T)`, advisory may seed a broad shape but must not label it as exact runtime observation.

---

# 21. Advisory workspace/index product

Target immutable product:

```rust
#[derive(Clone, Debug, Default)]
pub struct AdvisoryWorkspace {
    pub expressions: BTreeMap<SourceSiteId, AdvisoryFact>,
    pub bindings: BTreeMap<SourceSiteId, AdvisoryFact>,
    pub fields: BTreeMap<FieldId, AdvisoryFact>,
    pub parameters: BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,
    pub callables: BTreeMap<CallableId, Arc<AdvisoryCallableSummary>>,
    pub targets: BTreeMap<SourceSiteId, AdvisoryTargetResolution>,
}

pub struct AdvisoryTargetResolution {
    pub target: SemanticTargetId,
    pub confidence: AdvisoryConfidence,
    pub provenance: Vec<AdvisoryOrigin>,
}
```

If memory/performance favors per-module shards, use:

```rust
BTreeMap<ModuleId, Arc<AdvisoryModuleProduct>>
```

with a thin workspace facade. Prefer shard reuse if current incremental source architecture already operates per module/callable.

## 21.1 Query semantics

Missing entry means “no advisory product published for this site”, not `Unknown` unless the caller explicitly asks for an `AdvisoryFact` and the API chooses to synthesize an `unknown()` view.

This distinction matters for coverage metrics:

```text
no advisory coverage != analyzed and widened to Unknown
```

Expose that distinction internally.

---

# 22. Reuse the compiler query/invalidation system; do not port the LSP graph

`phalcom-semantic/src/db/key.rs` already defines typed query keys and the DB already owns dependency/reverse dependency state.

Add query products as necessary. Prefer separating syntax-owned source structure from formal callable attachment:

```rust
QueryKey::SourceStructure(ModuleId)
QueryKey::SourceFormalAttachment(CallableId)
QueryKey::AdvisoryCallable(CallableId)
QueryKey::AdvisoryModule(ModuleId) // only when top-level/module aggregation requires it
```

Do not mechanically add all four if the existing product API supports equivalent typed subproducts under fewer keys. The semantic granularity requirement is:

- source structure/scopes/lexical occurrences are module/source keyed and must not depend on every type-checking result;
- formal binding/expression/call attachment is callable keyed and may depend on `CallableBody(callable)`;
- advisory callable summary/body shape is callable keyed;
- module advisory aggregation exists only for top-level statements/field aggregation where required.

This split prevents a type-only change in one callable from rebuilding the module's entire lexical/occurrence structure.

## 22.1 Required dependencies

A source-structure query depends on the products it actually consumes, such as:

```text
ParsedModule(module)
LinkedInterface(module) / resolved import product
DeclarationSurface(declarations in module)
```

A source-formal-attachment query for one callable additionally depends on:

```text
SourceStructure(callable.owner.module)
CallableBody(callable)
CallableSignature(callable) when signature/parameter sites are attached
```

An advisory callable query depends on:

```text
CallableBody(callable)            when consuming formal expression/flow facts
CallableSignature(callable)
DeclarationSurface(receiver owners actually dispatched through)
HierarchyEdge(visited owners)
SourceIndex(module)
AdvisoryCallable(callee)           for consumed advisory return summaries
```

The exact edge set must reflect actual consumption, not “depend on whole workspace for safety”.

## 22.2 Do not port these LSP invalidation structures

Do not migrate as parallel authority:

```text
phalcom-lsp semantic ModuleGraph dependent closure
LSP callable_dependencies / callable_dependents
LSP SourceChangeKind-driven semantic invalidation
LSP advisory generation as cache identity
```

Any remaining LSP source-change classifier may remain purely as a worker scheduling optimization until Part 3, but it cannot decide semantic validity/reuse of compiler products.

## 22.3 Product fingerprint requirements

Add deterministic fingerprints for:

- module source index semantic product;
- advisory callable summary;
- advisory module/workspace shard if stored.

Do not hash:

- `SnapshotId.revision` merely because it changed;
- allocation addresses;
- hash-map iteration order;
- raw site local numbers when renumbering does not change semantic attachment and a canonical source ordering can be hashed instead.

Do hash semantically observable:

- target identity;
- occurrence kind/role/target and ranges where source-position product changes matter;
- advisory shape/confidence and canonicalized provenance where exposed;
- callable dependency identities;
- field/parameter contribution semantics.

---

# 23. Source-index/product fingerprint distinction

Source indexes are unusual because source ranges are semantically observable to tooling even when language semantics are unchanged.

Use two fingerprints if necessary:

```rust
pub struct SourceIndexFingerprints {
    pub semantic: ProductFingerprint,
    pub presentation: ProductFingerprint,
}
```

- `semantic`: target/scope/attachment meaning; ignores pure trivia/range shifts when safe.
- `presentation`: includes ranges/occurrence positions and changes on editor-visible movement.

Do not force the formal semantic dependency graph to invalidate type products because whitespace moved every source range.

Advisory computation should normally depend on the **semantic** source-index fingerprint, while LSP token/occurrence publication may observe the **presentation** fingerprint.

If the existing DB product model cannot carry two fingerprints cleanly, represent source position as a separate lightweight product/query rather than contaminating formal semantic invalidation.

This is a critical incrementality design point.

---

# 24. Snapshot query facade

Add pure read APIs on `SemanticSnapshot`; request handlers should eventually need no knowledge of internal maps.

Minimum Part 2 query surface:

```rust
impl SemanticSnapshot {
    pub fn source_site(&self, id: &SourceSiteId) -> Option<&SourceSite>;
    pub fn source_site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite>;
    pub fn occurrence_at(&self, module: &ModuleId, offset: usize) -> Option<OccurrenceView<'_>>;
    pub fn occurrences_for_target(&self, target: &SemanticTargetId) -> &[SourceSiteId];

    pub fn formal_expression(&self, site: &SourceSiteId) -> Option<&ExpressionAnalysis>;
    pub fn formal_binding(&self, site: &SourceSiteId) -> Option<&BindingState>;
    pub fn formal_fact_at(&self, module: &ModuleId, offset: usize) -> Option<FormalFactView<'_>>;

    pub fn advisory_fact(&self, site: &SourceSiteId) -> Option<&AdvisoryFact>;
    pub fn advisory_binding(&self, binding_target: &SourceSiteId) -> Option<&AdvisoryFact>;
    pub fn advisory_callable(&self, callable: &CallableId) -> Option<&AdvisoryCallableSummary>;
}
```

Use borrowed views. Do not clone whole maps or `CallableAnalysis` just to answer hover.

## 24.1 Snapshot-guarded external handles

For APIs that accept a handle cached outside the snapshot:

```rust
impl SemanticSnapshot {
    pub fn resolve_site_ref(&self, reference: &SourceSiteRef) -> Option<&SourceSite> {
        if reference.snapshot != self.id { return None; }
        self.source_site(&reference.site)
    }
}
```

Never silently reinterpret a stale local site ID in the newest snapshot.

---

# 25. Formal/advisory composition for presentation

Part 2 defines composition policy but not final hover UI wording.

## 25.1 Composition record

Provide a machine-readable view:

```rust
pub struct SemanticSiteView<'a> {
    pub site: &'a SourceSite,
    pub occurrence: Option<OccurrenceView<'a>>,
    pub formal: Option<FormalFactView<'a>>,
    pub advisory: Option<&'a AdvisoryFact>,
}
```

## 25.2 Precedence laws

1. Formal established/assumed fact is the formal answer.
2. Formal invalid/blocked/dynamic/unknown status is preserved exactly.
3. Advisory may be displayed in a separate labeled field.
4. Advisory cannot replace an available formal result.
5. Formal `Unknown` does not become formal known just because advisory has a shape.
6. If no formal coverage exists, tooling may show advisory only, clearly labeled as advisory.
7. If formal and advisory disagree, formal remains authoritative; do not blend them into a union type.

## 25.3 No `≈` inside compiler type strings

The LSP may continue a UI convention such as `≈ List<Int>`, but the compiler advisory product stores structured `ValueShape`, not decorated strings.

---

# 26. LSP identity boundary after Part 2

Refactor `phalcom-lsp/src/semantic/ids.rs` aggressively.

## 26.1 `DocumentModuleMap` target

The long-term boundary map should be conceptually:

```rust
pub struct DocumentModuleMap {
    by_uri: BTreeMap<Url, phalcom_modules::ModuleId>,
    by_module: BTreeMap<phalcom_modules::ModuleId, Url>,
}
```

Remove semantic use of:

```text
legacy ModuleId(String)
lsp_by_uri
uri_by_lsp
ClassId
CallableId
FieldId
local DispatchSide
```

A temporary legacy key adapter may exist during an intermediate task solely to keep unported request handlers compiling. It must be private to the LSP boundary and must not key any semantic product after Part 2.

## 26.2 Standalone unsaved documents

Canonical synthetic module/project identities already exist in `phalcom-modules`. Use them for unsaved/standalone buffers. Do not fall back to “URI string is module identity” as semantic truth.

## 26.3 URI is presentation/source locator

URI mapping may change while canonical source/module identity remains stable. Therefore URI never appears inside `ValueShape`, callable identity, field identity, parameter slot identity, or compiler dependency keys.

---

# 27. LSP semantic snapshot transition

`phalcom-lsp/src/semantic/snapshot.rs` currently owns a full advisory snapshot plus `static_snapshot`.

By the end of Part 2 it must no longer be a semantic database.

A permissible transitional wrapper is:

```rust
pub struct SemanticSnapshot {
    pub generation: SemanticGeneration, // orchestration stamp only
    pub compiler: Arc<phalcom_semantic::SemanticSnapshot>,
    pub documents: Arc<DocumentModuleMap>,
}
```

If source cache metadata is needed, keep it clearly non-semantic or reference compiler parsed-source products.

Remove fields representing independent semantic truth:

```text
files with LocalFacts/FieldFacts/ParameterFacts
classes
summaries
field_facts
parameter_facts
LSP ModuleGraph
static_snapshot nested beside advisory snapshot
```

## 27.1 Transitional facade methods

To avoid forcing all Part 3 handler migrations into Part 2, existing LSP methods may survive as adapters:

```rust
occurrence_at(...)
references_for_target(...)
visible_bindings_at(...)
formal_*_presentation(...)
binding_at(...)
return_for_callable(...)
```

But their implementations must delegate to `compiler.source_index`, formal products, or `compiler.advisory`. They cannot run local inference or scan duplicate tables.

Mark these adapters with a Part 3 deletion note in code comments referencing the Part 3 spec/task, not a generic placeholder comment.

---

# 28. LSP `SemanticEngine` transition

`phalcom-lsp/src/semantic/engine.rs` currently owns the entire advisory world.

By the Part 2 release gate, it may remain only as an orchestration compatibility shell if Part 3 still needs it, but it must not own:

- class surfaces;
- callable summaries;
- field/parameter advisory facts;
- scope/occurrence products;
- advisory call dependency graph;
- semantic module graph;
- semantic invalidation decisions.

Preferred transitional shape:

```rust
pub struct SemanticEngine {
    compiler_snapshot: Option<Arc<phalcom_semantic::SemanticSnapshot>>,
    counters: PerfCountersHandle,
}
```

or delete it if `AnalysisService` can already publish the compiler snapshot without broad handler churn.

`set_static_analysis(...)` should disappear as a semantic composition operation. There is no longer “advisory state plus attached static state”; the compiler snapshot already contains both channels.

---

# 29. What to migrate, what to delete, what to keep as an adapter

Use this as the file-level takeover map.

## 29.1 `phalcom-semantic` — create

```text
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/source_index/site.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/source_index/interval.rs
phalcom-semantic/src/source_index/builder.rs

phalcom-semantic/src/advisory/mod.rs
phalcom-semantic/src/advisory/shape.rs
phalcom-semantic/src/advisory/fact.rs
phalcom-semantic/src/advisory/provenance.rs
phalcom-semantic/src/advisory/analyzer.rs
phalcom-semantic/src/advisory/flow.rs
phalcom-semantic/src/advisory/summary.rs
phalcom-semantic/src/advisory/solver.rs
phalcom-semantic/src/advisory/query.rs
```

Consolidate files if implementations are genuinely small; do not create empty layering files.

## 29.2 `phalcom-semantic` — modify

```text
phalcom-semantic/src/lib.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/scope.rs          # retire/redirect minimal old ScopeTable if superseded
phalcom-semantic/src/checker/analysis.rs  # publish call resolution table if needed
phalcom-semantic/src/checker/context.rs   # expose resolution product, not presentation logic
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/metrics.rs     # advisory/source reuse counters if warranted
```

## 29.3 `phalcom-lsp` — migrate/delete semantic authority

These modules should disappear or become thin compatibility re-exports/adapters by Part 2 completion:

```text
phalcom-lsp/src/semantic/facts.rs
phalcom-lsp/src/semantic/analyzer.rs
phalcom-lsp/src/semantic/flow.rs
phalcom-lsp/src/semantic/callable.rs
phalcom-lsp/src/semantic/infer.rs
phalcom-lsp/src/semantic/scope.rs
phalcom-lsp/src/semantic/occurrence.rs
phalcom-lsp/src/semantic/dispatch.rs
phalcom-lsp/src/semantic/surface.rs
phalcom-lsp/src/semantic/module_graph.rs
phalcom-lsp/src/semantic/invalidation.rs
```

Do not leave these files containing shadow implementations “for now” after their compiler replacements pass parity tests.

## 29.4 `phalcom-lsp` — keep transitional adapters

```text
phalcom-lsp/src/semantic/mod.rs
phalcom-lsp/src/semantic/ids.rs
phalcom-lsp/src/semantic/snapshot.rs
phalcom-lsp/src/semantic/engine.rs      # only if lifecycle wrapper still requires it
phalcom-lsp/src/analysis_service.rs
phalcom-lsp/src/backend.rs
```

These may remain because Part 3 owns final lifecycle/handler cutover, but they must no longer own semantic inference/identity.

---

# 30. Build pipeline inside `SemanticWorkspaceSession`

Extend the current compiler session update in this order:

```text
1. ingest/retain parsed module products
2. module/link/declaration/surface/signature products
3. build/reuse syntax-owned source structure shards (scopes, declaration/token occurrences)
4. formal callable analysis (Part 1 semantics)
5. build/reuse callable-scoped formal source attachments and expression sites
6. evaluate/reuse advisory callable/module products
7. assemble immutable SourceSemanticIndex + AdvisoryWorkspace
8. construct one SemanticSnapshot containing all channels
9. publish only after all required products for that revision are coherent
```

## 30.1 Advisory failure does not invalidate formal snapshot

If advisory analysis is cancelled, budget-exceeded or internally fails:

- formal semantic products remain valid/publishable;
- advisory product for affected site/callable is absent or explicitly non-ready in an advisory status channel;
- no formal type is weakened/strengthened;
- LSP may omit advisory enrichment.

Recommended advisory product status:

```rust
pub enum AdvisoryStatus {
    Ready,
    Blocked,
    Cancelled,
    BudgetExceeded,
    InternalFailure,
}
```

Do not overload `ValueShape::Unknown` to mean cancellation/internal failure.

## 30.2 Last-known-good policy

The existing session retains last-known-good formal snapshots. Advisory reuse must not splice facts from a different `SnapshotId` into a new formal snapshot merely to keep UI hints alive.

Reusing an `Arc<AdvisoryCallableSummary>` is valid only when its semantic input fingerprint proves it unchanged.

---

# 31. Advisory status is orthogonal to advisory shape

Mirror the lesson from Part 1: do not conflate “what shape do we have?” with “did analysis complete?”.

Use:

```rust
pub struct AdvisoryAnalysis<T> {
    pub value: Option<T>,
    pub status: AdvisoryStatus,
}
```

or equivalent per-product representation.

Examples:

```text
Ready + Unknown shape
    analysis completed and concluded no useful shape

Blocked + no fact
    required input unavailable

BudgetExceeded + partial fact
    permissible only if explicitly represented as partial; never publish as Ready
```

This prevents the old `Unknown` bucket from becoming another hidden failure sentinel.

---

# 32. Canonical type/shape conversion helper

Create one explicit helper module/function for formal → advisory projection; do not duplicate class-name mappings in LSP.

Example interface:

```rust
pub fn advisory_shape_from_type(
    store: &TypeStore,
    declarations: &DeclarationTypeTable,
    ty: TypeId,
) -> ValueShape
```

Requirements:

- inspect canonical `TypeData`, never formatted type strings;
- map nominal declaration directly to `DeclarationId`;
- recurse through supported applied/tuple/record forms;
- apply a recursion/depth budget to recursive type structure;
- unsupported forms return `Unknown` conservatively;
- `Never` is not “unknown shape”; it represents no runtime value and should normally produce no advisory value fact rather than `Unknown` instance;
- `Unit` maps only according to actual Phalcom runtime/unit semantics, not as a fallback.

Add tests for generic applied types and unions.

---

# 33. Interaction with invalid formal expressions

A major Part 1 rule is that an invalid expression can retain independently established type knowledge.

Part 2 must preserve that in site composition.

Example:

```phalcom
let x: Int = CellNum.new(1) // formal mismatch; current fact CellNum retained
x.foo
```

At the declaration site:

```text
formal current: Established(CellNum)
formal status/causal state: invalid due to annotation mismatch
advisory shape: may be Instance(CellNum)
```

At downstream `x`:

- formal lookup uses retained binding fact under Part 1 recovery semantics;
- advisory can also propagate CellNum shape;
- neither channel erases the invalid cause;
- presentation can explain “known CellNum, declaration contract Int was refuted” rather than displaying a fake Int or hiding all type information.

Part 2 must add an integration regression around this exact composition because it is the point of the single-world architecture.

---

# 34. Advisory disagreement policy

Add a pure diagnostic/debug helper, not a compiler error:

```rust
pub enum AdvisoryAgreement {
    Compatible,
    MoreSpecific,
    Incomparable,
    Unknown,
}
```

This can be used for telemetry/tests to verify advisory quality against formal established types where a projection exists.

It must not call hard diagnostic emission.

Useful invariant test:

```text
for every Established(T) + advisory shape S with a defined shape→formal approximation,
if S is incompatible with T, the checker result is unchanged and only advisory telemetry records it.
```

Do not implement a general shape→formal type converter merely to support this gate. Only compare forms with a sound existing projection relation.

---

# 35. Performance and complexity requirements

Part 2 touches hot LSP query paths, so correctness must include complexity constraints.

## 35.1 Position lookup

Target:

```text
occurrence_at: O(log n + k) where k is bounded overlapping candidates
expression/formal-site-at: O(log n + k)
local lexical scope lookup: O(log scopes + lexical depth + map lookup)
target references lookup: O(log targets) + output size
callable/formal/advisory lookup by canonical ID: O(log n) or expected O(1)
```

Forbidden after Part 2:

```text
scan every callable analysis to answer one hover
scan every expression in a module to answer one position query
scan every file occurrence to answer references for a canonical target
linear-search callable signatures by owner-name + selector-string reconciliation
```

## 35.2 Allocation discipline

Snapshot query methods should return references/slices/views. No per-hover clone of:

- `CallableAnalysis`;
- all occurrences;
- all visible classes;
- entire advisory workspace;
- type store.

Formatting can allocate final protocol strings.

## 35.3 Structural counters

Add test-visible compiler counters if no existing counter can prove:

```text
source_index_modules_rebuilt
source_index_modules_reused
advisory_callables_recomputed
advisory_callables_reused
advisory_fixpoint_steps
```

Do not add counters that will never be asserted or operationally useful.

---

# 36. Required focused tests — compiler source identity/index

Create `phalcom-semantic/tests/source_semantic_index.rs`.

At minimum include these cases.

## 36.1 Canonical callable occurrence

Source declares a class method and calls it. Assert declaration and call occurrence target the exact same compiler `CallableId`; no string comparison in test helper.

## 36.2 Local binding declaration/read/write target

Assert:

```text
declaration site target == read target == write target
```

using `SemanticTargetId::Binding(declaration_site)`.

## 36.3 Shadowed bindings

Nested `x` shadows outer `x`. References for inner target exclude outer occurrences and vice versa.

## 36.4 Same-scope redeclaration recovery

Two same-scope declarations of `x`:

- duplicate has its own source site;
- `redeclaration_of` points at first site;
- subsequent resolution targets first binding under Part 1 recovery rule;
- formal redeclaration diagnostic remains compiler-owned.

## 36.5 Canonical import target

Import occurrence target equals the exact `phalcom_modules::ModuleId` from linked module products; no URI/path-string fake module ID.

## 36.6 Field identity

Field declaration and resolved field reference use same compiler `FieldId` including dispatch side.

## 36.7 Expression attachment

Every expression in a representative `CallableAnalysis` maps from `(CallableId, ExpressionId)` to one source site and back to the same formal product.

## 36.8 Snapshot guard

A `SourceSiteRef` from snapshot N is rejected by snapshot N+1 even if local numeric ID happens to exist.

## 36.9 Interval nested selection

For nested expression/token ranges, position query selects documented shortest/highest-priority occurrence.

## 36.10 Large index bounded lookup

Generate thousands of occurrences and assert structural instrumentation proves lookup does not scan all entries.

---

# 37. Required focused tests — formal projection

Extend/replace `phalcom-semantic/tests/presentation.rs` with machine-readable index coverage.

## 37.1 No reanalysis

Build snapshot, capture checker/query counters, perform repeated formal site/presentation lookups, assert no callable query recomputes.

## 37.2 Established vs Assumed survives projection

Two sites with same `TypeId`, one established and one assumed, remain distinguishable before rendering and in structured presentation output.

## 37.3 Non-ready status survives projection

Blocked/cancelled/budget/internal/invalid states are not converted to Unknown.

## 37.4 Invalid-but-known fact

Expression with invalid status and independently known type presents both properties; type is not discarded.

## 37.5 No exact-range binding lookup dependency

Lookup by position/site succeeds when cursor is within reference occurrence, not only when it equals declaration range.

---

# 38. Required focused tests — advisory domain

Create `phalcom-semantic/tests/advisory_shapes.rs`.

## 38.1 Unknown dominates shape join

`Unknown.join(Instance(Int)) == Unknown`.

## 38.2 Bounded union canonicalization

Joining alternatives in different traversal orders yields equal shape and equal fingerprint.

## 38.3 Union cap

Ninth incompatible alternative widens to Unknown at the configured cap of eight.

## 38.4 Exact-list widening

Equal-length exact lists join positionally; differing lengths widen to list element shape.

## 38.5 Record join

Same labels join per field; incompatible label shape does not fabricate fields.

## 38.6 Canonical IDs inside shapes

Instance/callable/field-related shapes use compiler IDs; compile-time type system should make LSP-local IDs impossible to pass.

## 38.7 Provenance cap/determinism

More than four origins produces deterministic capped provenance independent of insertion order.

---

# 39. Required focused tests — advisory analysis and formal separation

Create `phalcom-semantic/tests/advisory_analysis.rs`.

## 39.1 Literal and collection shape inference

Verify exact literals/list/tuple/record shapes without affecting formal `TypeKnowledge`.

## 39.2 Formal Established seed

Established nominal/applied type can seed compatible broad advisory shape.

## 39.3 Formal Assumed seed remains advisory

Assumed formal type can guide advisory shape but does not become advisory “Exact” solely because it is a formal premise.

## 39.4 Formal Unknown + advisory known

Tooling may have advisory `Instance(C)` while formal remains `Unknown`; assert the formal product is byte/structurally unchanged.

## 39.5 Advisory disagreement cannot reject

Inject/derive advisory shape incompatible with formal established type and assert compiler diagnostics/formal fingerprint remain unchanged.

## 39.6 Invalid-but-known composition

Use the refuted annotation scenario from §33 and assert formal known type, invalid cause, advisory shape and downstream target coexist.

## 39.7 Canonical dispatch

Advisory member call resolves through compiler `SurfaceDispatchResolver`; no LSP surface/dispatch object participates.

## 39.8 Method-family capture

Method family shape stores canonical callable IDs and resolves exact/rest selector behavior deterministically.

---

# 40. Required focused tests — interprocedural advisory incrementality

Create `phalcom-semantic/tests/advisory_incrementality.rs`.

## 40.1 Caller contribution replacement

Changing one caller argument recomputes only touched parameter slots and dependent advisory callable products.

## 40.2 Caller removal

Removing a caller removes its contribution rather than monotonically accumulating stale shape evidence.

## 40.3 Unchanged caller reuse

Whitespace-only/source-position change that leaves advisory semantic inputs unchanged reuses advisory callable `Arc`/fingerprint.

## 40.4 Callee body change propagation

Change advisory return shape of callee; assert dependent caller advisory summary recomputes.

## 40.5 Formal-only semantic change propagation

Change a formal callable signature/dispatch dependency without changing raw caller text; advisory dependent invalidates because it consumed the formal product.

## 40.6 Recursive SCC convergence

Mutually recursive callables reach deterministic fixpoint under bounded unions; assert no arbitrary fixed-pass truncation.

## 40.7 Budget/cancellation separation

Advisory cancellation/budget failure leaves formal snapshot/product valid and exposes advisory non-ready status rather than `Unknown` masquerading as success.

---

# 41. Required focused tests — LSP takeover boundary

Add/update LSP tests without making Part 2 own final protocol behavior.

Recommended files:

```text
phalcom-lsp/tests/semantic_identity_takeover.rs
phalcom-lsp/tests/advisory_takeover.rs
phalcom-lsp/tests/composition1.rs
phalcom-lsp/tests/performance.rs
```

## 41.1 No duplicate semantic identity types in public query path

Compile-time/runtime assertions should show LSP query targets are compiler `SemanticTargetId` / canonical `CallableId`, not LSP-local `ClassId`/`CallableId`.

## 41.2 Existing hover/inlay composition parity

Current formal-first/advisory-`≈` behavior remains user-visible, but data comes from one compiler snapshot.

## 41.3 Formal lookup no full callable scan

Instrument current LSP facade and assert repeated hover/formal lookup does not scan all compiler callable analyses.

## 41.4 References use target reverse index

References query results stay correct after takeover and no longer iterate all occurrences across all files.

## 41.5 Canonical callable signature lookup

Signature-help bridge receives canonical `CallableId`; delete test helper/path that reconciles owner-name + selector-string.

## 41.6 Advisory engine cannot publish without compiler snapshot

Any transitional LSP wrapper must consume one compiler snapshot; it cannot independently rebuild semantic classes/facts from `Program`.

---

# 42. Repository-wide forbidden-pattern audit

Before Part 2 completion, run searches and manually classify every hit.

Forbidden semantic-authority patterns under `phalcom-lsp/src/semantic`:

```text
struct ClassId
struct CallableId
struct FieldId
enum DispatchSide
struct ScopeGraph
struct OccurrenceIndex
struct SemanticEngine state containing classes/summaries/field_facts/parameter_facts
ValueShape definitions owned by LSP
ClassSurface / MemberSurface as semantic authority
callable_dependencies / callable_dependents semantic graph
ModuleGraph semantic dependency authority
formal callable lookup by selector.encode() string comparison
formal binding/expression lookup by scanning all callable analyses
```

Allowed hits must be one of:

- temporary private adapter/re-export explicitly scheduled for Part 3 deletion;
- test fixture referring to historical behavior;
- unrelated protocol type with the same word.

Do not satisfy the audit by renaming duplicate structures.

---

# 43. Implementation task 1 — Lock identity lifetimes and source-site primitives

**Files:**

- Modify: `phalcom-semantic/src/identity.rs`
- Create: `phalcom-semantic/src/source_index/site.rs`
- Create: `phalcom-semantic/tests/source_semantic_index.rs`

**Produces:** `SourceSiteLocalId`, `SourceSiteId`, `SourceSiteRef`, `SourceSite`, `SourceOwner`, `SemanticTargetId`.

**Steps:**

- [ ] Write tests proving canonical target IDs remain cross-revision while `SourceSiteRef` is snapshot-guarded.
- [ ] Run the focused test and verify it fails because source-site types/API do not exist.
- [ ] Implement dense snapshot-scoped site identities and guarded external reference.
- [ ] Add rustdoc stating exact lifetime/stability promises.
- [ ] Re-run focused tests.
- [ ] Run `cargo check -p phalcom-semantic`.
- [ ] Commit as one identity-foundation change.

Review gate: reject any design that encodes source range or URI as target identity.

---

# 44. Implementation task 2 — Move lexical source identity into compiler ownership

**Files:**

- Create: `phalcom-semantic/src/source_index/scope.rs`
- Create: `phalcom-semantic/src/source_index/builder.rs`
- Modify: `phalcom-semantic/src/source_index/mod.rs`
- Tests: `phalcom-semantic/tests/source_semantic_index.rs`

**Consumes:** parsed `Program`, canonical module/link products, source-site allocator.

**Produces:** `SourceScopeIndex`, `SourceBindingInfo`, deterministic binding declaration sites.

**Steps:**

- [ ] Port focused behavior tests from LSP scope code first: nested scopes, source order, imports, destructure, method/block/for parameters, mutability.
- [ ] Add same-scope redeclaration test with separate duplicate site + `redeclaration_of`.
- [ ] Implement scope builder under compiler ownership.
- [ ] Resolve imports/classes through canonical compiler/module products.
- [ ] Ensure no LSP ID type is imported into `phalcom-semantic`.
- [ ] Run focused tests and `cargo check -p phalcom-semantic`.

Do not delete LSP scope code yet; deletion waits for Task 9 after parity.

---

# 45. Implementation task 3 — Move exact occurrence indexing into compiler ownership

**Files:**

- Create: `phalcom-semantic/src/source_index/occurrence.rs`
- Create: `phalcom-semantic/src/source_index/interval.rs`
- Modify: `phalcom-semantic/src/source_index/builder.rs`
- Tests: `phalcom-semantic/tests/source_semantic_index.rs`

**Produces:** canonical occurrence/target index and O(log n + k) source-position selection.

**Steps:**

- [ ] Port occurrence-selection regressions before implementation.
- [ ] Implement deterministic interval index using sorted starts + prefix max-end.
- [ ] Build occurrences against compiler scope targets/canonical declaration IDs.
- [ ] Build target reverse occurrence index.
- [ ] Add unresolved occurrence hint separate from semantic target.
- [ ] Add large-index structural lookup test.
- [ ] Run source-index tests.

Review gate: no full-workspace references scan remains in compiler query API.

---

# 46. Implementation task 4 — Attach formal binding/expression/call products

**Files:**

- Modify: `phalcom-semantic/src/checker/analysis.rs`
- Modify: checker call-resolution publication files if required
- Modify: `phalcom-semantic/src/source_index/builder.rs`
- Tests: `phalcom-semantic/tests/source_semantic_index.rs`
- Extend: `phalcom-semantic/tests/presentation.rs`

**Produces:** `(CallableId, BindingId)` and `(CallableId, ExpressionId)` source attachments; canonical resolved call target attachment.

**Steps:**

- [ ] Write expression/binding round-trip attachment tests.
- [ ] Write resolved call occurrence → exact `CallableId` test.
- [ ] Publish a compact call-resolution table if the current `CallResolutionId` cannot be dereferenced from `CallableAnalysis`.
- [ ] Attach formal products by exact checker IDs, using range/name only during unique construction matching—not as final identity.
- [ ] Fail closed on ambiguous attachment; never choose arbitrary first match.
- [ ] Re-run checker composition/dependency tests from Part 1.

---

# 47. Implementation task 5 — Publish source index and machine-readable formal projection

**Files:**

- Modify: `phalcom-semantic/src/presentation.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Tests: `phalcom-semantic/tests/presentation.rs`
- Tests: `phalcom-semantic/tests/source_semantic_index.rs`

**Produces:** snapshot-owned `SourceSemanticIndex`, `FormalFactRef/View`, position query facade, pure presenter.

**Steps:**

- [ ] Add tests that fail on current per-callable/on-demand presentation index.
- [ ] Add `source_index` to immutable snapshot construction.
- [ ] Replace rendered-string-as-index ownership with machine-readable formal references.
- [ ] Preserve Part 1 Established/Assumed/status/causal distinctions.
- [ ] Replace linear expression scan with interval lookup.
- [ ] Assert repeated presentation performs zero semantic recomputation.

---

# 48. Implementation task 6 — Port advisory domain with canonical identities

**Files:**

- Create: `phalcom-semantic/src/advisory/{mod.rs,shape.rs,fact.rs,provenance.rs}`
- Tests: `phalcom-semantic/tests/advisory_shapes.rs`

**Produces:** `ValueShape`, `AdvisoryFact`, `AdvisoryConfidence`, `AdvisoryOrigin`, bounded deterministic joins.

**Steps:**

- [ ] Copy/translate LSP shape-domain tests into compiler crate first.
- [ ] Replace local `ClassId` with canonical `DeclarationId`; local callable/field/module IDs with compiler IDs.
- [ ] Rename `InferredValue` and `Confidence` to advisory-explicit names.
- [ ] Preserve union cap and compact provenance with deterministic ordering.
- [ ] Add formal/advisory type-level separation tests.
- [ ] Run `cargo test -p phalcom-semantic --test advisory_shapes`.

No LSP deletion until parity is proven.

---

# 49. Implementation task 7 — Port advisory expression/flow analysis over compiler products

**Files:**

- Create: `phalcom-semantic/src/advisory/analyzer.rs`
- Create: `phalcom-semantic/src/advisory/flow.rs`
- Modify: `phalcom-semantic/src/advisory/mod.rs`
- Tests: `phalcom-semantic/tests/advisory_analysis.rs`

**Consumes:** compiler parsed source, source index, formal analyses, canonical dispatch/surfaces/hierarchy.

**Produces:** per-callable local/binding/expression/field/return advisory facts.

**Steps:**

- [ ] Port representative LSP advisory flow tests first.
- [ ] Implement formal→advisory projection helper over `TypeData`, not strings.
- [ ] Implement advisory analyzer using compiler dispatch only.
- [ ] Port method/method-family behavior with canonical callable IDs.
- [ ] Preserve one shared flow traversal for all advisory effects.
- [ ] Add invalid-but-known and formal-Unknown/advisory-known composition tests.
- [ ] Assert formal products/fingerprints do not change when advisory result changes.

---

# 50. Implementation task 8 — Port contribution-indexed interprocedural advisory solver

**Files:**

- Create: `phalcom-semantic/src/advisory/summary.rs`
- Create: `phalcom-semantic/src/advisory/solver.rs`
- Create: `phalcom-semantic/src/advisory/query.rs`
- Modify: `phalcom-semantic/src/db/key.rs`
- Modify: `phalcom-semantic/src/db/product.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Tests: `phalcom-semantic/tests/advisory_incrementality.rs`

**Produces:** canonical parameter slots/contributions, callable advisory summaries, compiler-DB dependency/reuse.

**Steps:**

- [ ] Write caller replacement/removal/reuse tests.
- [ ] Use `(CallableId, parameter index)` slots.
- [ ] Port contribution replacement algorithm.
- [ ] Implement worklist/SCC convergence under query budget; no arbitrary pass-count success.
- [ ] Record dependencies through compiler DB query keys.
- [ ] Add deterministic advisory product fingerprints.
- [ ] Add recursive convergence and cancellation tests.
- [ ] Run existing Step 5.5/product-stability tests to ensure no broad invalidation regression.

---

# 51. Implementation task 9 — Publish advisory workspace in the compiler snapshot

**Files:**

- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/presentation.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Tests: `phalcom-semantic/tests/advisory_analysis.rs`
- Tests: `phalcom-semantic/tests/advisory_incrementality.rs`

**Produces:** coherent single snapshot containing formal + source + advisory channels.

**Steps:**

- [ ] Add snapshot coherence test verifying same `SnapshotId` owns formal/source/advisory products.
- [ ] Add advisory non-ready status independent of `ValueShape::Unknown`.
- [ ] Reuse unchanged advisory shards/summary `Arc`s.
- [ ] Add `SemanticSiteView` composition API.
- [ ] Ensure advisory failure cannot prevent publication of otherwise valid formal snapshot.
- [ ] Run all compiler semantic tests.

---

# 52. Implementation task 10 — Replace LSP identity and snapshot bridges with compiler adapters

**Files:**

- Modify: `phalcom-lsp/src/semantic/ids.rs`
- Modify: `phalcom-lsp/src/semantic/snapshot.rs`
- Modify: `phalcom-lsp/src/semantic/mod.rs`
- Modify: `phalcom-lsp/src/backend.rs` only where needed for canonical handles
- Tests: `phalcom-lsp/tests/semantic_identity_takeover.rs`
- Extend: `phalcom-lsp/tests/composition1.rs`

**Produces:** URI ↔ canonical module boundary and compiler-snapshot-backed LSP facade.

**Steps:**

- [ ] Add tests rejecting old selector-string/name reconciliation.
- [ ] Reduce `DocumentModuleMap` to canonical module identity mappings.
- [ ] Use synthetic canonical module IDs for standalone documents.
- [ ] Replace `formal_*` full-callable scans with compiler source-index queries.
- [ ] Replace local target IDs in references/navigation adapters with compiler `SemanticTargetId`.
- [ ] Keep protocol rendering behavior stable.

---

# 53. Implementation task 11 — Delete/demote LSP semantic authority modules

**Files:**

Delete or reduce to trivial re-export only after parity tests pass:

```text
phalcom-lsp/src/semantic/facts.rs
phalcom-lsp/src/semantic/analyzer.rs
phalcom-lsp/src/semantic/flow.rs
phalcom-lsp/src/semantic/callable.rs
phalcom-lsp/src/semantic/infer.rs
phalcom-lsp/src/semantic/scope.rs
phalcom-lsp/src/semantic/occurrence.rs
phalcom-lsp/src/semantic/dispatch.rs
phalcom-lsp/src/semantic/surface.rs
phalcom-lsp/src/semantic/module_graph.rs
phalcom-lsp/src/semantic/invalidation.rs
```

Modify:

```text
phalcom-lsp/src/semantic/engine.rs
phalcom-lsp/src/semantic/mod.rs
phalcom-lsp/src/analysis_service.rs
```

**Steps:**

- [ ] Run compiler-vs-old-advisory parity fixtures before deletion and record intentional differences.
- [ ] Switch all transitional semantic facade methods to compiler products.
- [ ] Delete old implementations rather than leave dormant copies.
- [ ] Shrink `SemanticEngine` to orchestration compatibility shell or remove it if safe.
- [ ] Ensure `AnalysisService` still owns debounce/cancellation/workspace scanning only, not semantic facts.
- [ ] Run full `phalcom-lsp` test suite.

Part 3 may still remove the remaining wrapper/session plumbing; Part 2 must remove semantic authority.

---

# 54. Implementation task 12 — Incrementality, performance, and takeover audit

**Files:**

- Modify compiler/LSP performance tests and metrics where required.
- Add repository-wide audit script/test if existing test infrastructure supports it.

**Steps:**

- [ ] Assert source index module reuse across semantic-no-op edits where ranges/product split permits it.
- [ ] Assert advisory callable reuse and narrow invalidation.
- [ ] Assert position queries do not run semantic analysis or full scans.
- [ ] Run forbidden-pattern audit from §42.
- [ ] Run `cargo check --workspace`.
- [ ] Run `cargo test -p phalcom-semantic`.
- [ ] Run `cargo test -p phalcom-lsp`.
- [ ] Run existing semantic product stability/dependency/incrementality suites.
- [ ] Manually review that no LSP-owned semantic graph/ID system survives under a renamed type.

---

# 55. Migration/parity policy for existing LSP advisory tests

Do not discard the mature advisory test surface when deleting the old engine.

Classify every existing test under `phalcom-lsp/src/semantic` and `phalcom-lsp/tests` into:

1. **Domain test** — move to `phalcom-semantic/tests/advisory_shapes.rs`.
2. **Flow/solver semantic test** — move to `advisory_analysis.rs` or `advisory_incrementality.rs`.
3. **Source identity/occurrence test** — move to `source_semantic_index.rs`.
4. **Protocol presentation test** — keep in LSP, but feed compiler snapshot products.
5. **Worker lifecycle/performance test** — keep in LSP/Part 3 boundary.
6. **Legacy implementation detail** — delete only after equivalent semantic invariant is covered elsewhere.

The goal is not numerical test preservation. The goal is semantic coverage preservation at the correct owner.

---

# 56. Required negative architecture tests

Part 2 must contain tests that make regression toward two semantic worlds difficult.

## 56.1 Advisory cannot be a formal evidence constructor

If possible, enforce by module visibility/type API: `advisory` code has no constructor path that returns `TypeKnowledge::Known/Assumed` from `AdvisoryFact`.

A compile-fail test is ideal if the workspace already uses one; otherwise a repository audit plus module-level API test is acceptable.

## 56.2 LSP cannot construct semantic callable identity from strings

No public LSP helper like:

```rust
CallableId { owner_name, selector: String, ... }
```

should exist after takeover.

## 56.3 Snapshot mismatch is explicit

Stale `SourceSiteRef` returns `None`/`SnapshotMismatch`; it is never remapped by numeric coincidence.

## 56.4 No duplicate dispatch in advisory

A test with inheritance/override must show advisory resolved target equals compiler formal `ResolvedDispatch.callable` when both are available.

## 56.5 No advisory semantic graph

Changing module dependency must invalidate advisory through compiler DB/module products; test should not instantiate an LSP `ModuleGraph`.

---

# 57. Error handling and fail-closed rules

## 57.1 Source attachment ambiguity

If two source sites could map to one formal `(CallableId, BindingId/ExpressionId)`, this is an internal source-index attachment failure. Record an internal incident/non-ready attachment; do not pick by map order.

## 57.2 Unresolved semantic target

Occurrence remains targetless with optional hint. References/definition return no semantic target rather than guessing by spelling.

## 57.3 Advisory missing dependency

Return `AdvisoryStatus::Blocked`/no fact as appropriate. Do not make `ValueShape::Unknown` hide dependency failure.

## 57.4 Advisory internal failure

Formal snapshot remains valid. Record compiler internal advisory incident/metric/log product without turning it into a type mismatch.

## 57.5 Cancellation

No partially mutated shared advisory map reaches published snapshot. Build candidate immutable products and publish atomically with the owning compiler snapshot.

---

# 58. Concurrency and immutability requirements

Part 2 must preserve the existing “worker mutates, request path reads immutable Arc snapshot” architecture.

- `SourceSemanticIndex` is immutable after publication.
- `AdvisoryWorkspace` is immutable after publication.
- query methods require no mutable engine lock.
- source/advisory builders may use mutable scratch structures only before publication.
- LSP request handlers never trigger advisory fixpoint computation synchronously on the protocol thread.

Part 3 may change who owns the worker/session, but not this immutability principle.

---

# 59. Compatibility with demand-driven core analysis

The current repository intentionally avoids eagerly deep-analyzing all core callable bodies.

Part 2 must not regress this by making source-index/advisory publication demand every core body.

Rules:

- core declaration/member occurrences and surfaces can be indexed from source/surface products without callable body analysis;
- formal expression/binding site attachment exists only for analyzed callable bodies;
- advisory body summaries are demand-driven or limited to already analyzed/required callables;
- an absent core advisory summary is “not analyzed”, not `Unknown` formal type;
- opening/editing a core body may request deep analysis under existing demand policy.

Add a regression to existing core-startup tests: source/advisory takeover must not increase eager callable-body analysis count at startup.

---

# 60. Compatibility with native/core provenance

Canonical core/native declarations may point to physical or virtual sources.

Source sites exist only where a source artifact/range exists. Native semantic targets without a body source are still valid `DeclarationId`/`CallableId`/`FieldId` targets and can resolve through metadata provenance.

Do not invent fake `SourceSiteId` for native declarations that have no source site. Navigation can fall back to existing canonical native/source provenance products.

Advisory `Method(CallableId)` is valid even when the callable has no source site.

---

# 61. Documentation and public API naming

Update rustdoc and architecture docs so terminology is unambiguous:

- **formal inference**: checker/solver derivation of type facts;
- **advisory analysis**: non-authoritative abstract runtime-shape prediction;
- **source site**: snapshot-scoped source location identity;
- **semantic target**: entity an occurrence denotes;
- **presentation**: rendering/projection of already computed facts.

Remove comments that call LSP `ValueShape` “semantic type inference” without the advisory qualifier.

Do not call both compiler and LSP structures `SemanticDb` after takeover. The Part 3 final naming may simplify further; Part 2 should at least make ownership obvious.

---

# 62. Part 2 release gate

Part 2 is complete only when **all** of the following are true. The gate groups correspond to the four takeover requirements: **SC-10** canonical source identity, **SC-11** compiler-owned projection/index publication, **SC-12** compiler-owned advisory evidence, and **SC-13** LSP semantic demotion.

1. Part 1 WIP + amendments have been implemented and their release gates pass.
2. Canonical compiler `ModuleId`/`DeclarationId`/`CallableId`/`FieldId` are the only semantic identities for those entities.
3. Source binding targets are compiler-owned declaration sites; no LSP binding-ID semantic universe remains.
4. `SourceSiteRef` guards snapshot-scoped site IDs against stale reuse.
5. Compiler owns lexical source scope/index semantics required by navigation/advisory analysis.
6. Compiler owns exact source occurrence index and target reverse index.
7. Position queries use bounded interval lookup, not whole-module/full-workspace scans.
8. Formal bindings map by `(CallableId, BindingId)` to canonical source sites.
9. Formal expressions map by `(CallableId, ExpressionId)` to canonical source sites.
10. Formally resolved calls expose real canonical `CallableId` to exact source occurrences without presentation-time redispatch.
11. Advisory-only resolved targets remain in advisory target attachments and never masquerade as exact occurrence targets.
12. `SemanticSnapshot` publishes the source semantic index.
13. Formal presentation is projected from machine-readable formal products; rendered strings are not semantic truth.
14. Established vs Assumed remains observable through the projection layer.
15. Invalid/blocked/dynamic/unknown/cancelled/budget/internal formal states are preserved exactly.
16. `phalcom-semantic::advisory` owns the runtime `ValueShape` domain.
17. Advisory shape identities are canonical compiler IDs, not LSP URI/name/string IDs.
18. Advisory confidence is structurally separate from formal `EvidenceStatus`.
19. Advisory facts cannot be inserted into formal `TypeKnowledge`.
20. Compiler owns advisory local flow, field facts, parameter contributions and callable summaries.
21. Advisory dispatch uses compiler canonical dispatch/surfaces/hierarchy.
22. Advisory interprocedural propagation uses compiler DB dependencies/products, not a second graph.
23. Caller contribution replacement/removal does not retain stale facts.
24. Recursive advisory solving converges under explicit worklist/SCC + budget rules, not arbitrary successful pass count.
25. Advisory `Unknown` is distinct from advisory blocked/cancelled/budget/internal status.
26. `SemanticSnapshot` publishes advisory products coherent with the same snapshot revision.
27. Advisory failure cannot invalidate or strengthen formal results.
28. Invalid-but-known formal facts compose with advisory facts without losing invalidity or exact type knowledge.
29. Source semantic and advisory fingerprints are deterministic.
30. Pure source-position changes do not unnecessarily invalidate formal/advisory semantic products where the semantic/presentation fingerprint split applies.
31. Unchanged advisory products are reused across revisions where inputs are unchanged.
32. LSP `DocumentModuleMap` semantic mapping uses canonical module IDs; URI-string module identity is not semantic truth.
33. LSP formal lookup no longer scans every compiler callable analysis.
34. LSP callable mapping no longer reconciles owner names + selector strings.
35. LSP reference lookup no longer scans every occurrence in every file for canonical targets.
36. LSP-local duplicate class/callable/field/dispatch semantic ID definitions are removed or private non-semantic adapters scheduled for Part 3 deletion.
37. LSP-local scope/occurrence semantic authority is removed.
38. LSP-local class surface/dispatch semantic authority is removed.
39. LSP-local module dependency/invalidation semantic authority is removed.
40. LSP-local advisory analyzer/flow/solver semantic authority is removed.
41. The remaining LSP semantic wrapper, if any, is a read-only facade over one compiler snapshot plus boundary metadata.
42. Demand-driven core startup remains demand-driven; no eager universe body explosion occurs.
43. Existing formal composition, product-stability, dependency and type-checker tests remain green.
44. Migrated advisory/source tests retain the substantive semantics of the old mature LSP test suite.
45. `cargo check --workspace` passes.
46. `cargo test -p phalcom-semantic` passes.
47. `cargo test -p phalcom-lsp` passes.
48. The forbidden-pattern audit in §42 is manually reviewed, not merely executed.
49. A reviewer can point to exactly one owner for every row in §5.1.

Only after all 49 are true may implementation proceed to Part 3.
---

# 63. Handoff contract to Part 3

Part 3 may assume this architecture:

```text
phalcom-modules
    sole module/project/link identity authority

phalcom-semantic::SemanticWorkspaceSession
    existing compiler session and semantic DB owner

phalcom-semantic::SemanticSnapshot
    one immutable snapshot containing:
        formal products
        source semantic index
        canonical occurrences/targets
        advisory products
        module query products
        diagnostics/surfaces/signatures/hierarchy

formal channel
    Part 1 epistemic semantics only

advisory channel
    compiler-owned ValueShape abstract interpretation
    never formal authority

phalcom-lsp
    scheduling + URI/protocol adapters + transitional read facade only
    no independent semantic IDs, scope graph, occurrence graph,
    dispatch engine, module graph, advisory solver, or formal cache
```

Part 3's job is then lifecycle and consumer cutover, not another semantic migration.

---

# 64. Specification verification record

This Part 2 specification was grounded against repository `main` commit `a3f932e01118053265378e678b0dbaef2b9ceab8` and the saved Part 1 WIP + amendments.

## 64.1 Repository facts explicitly accounted for

The design accounts for the following inspected current implementation facts:

- `phalcom-semantic/src/identity.rs` already has canonical declaration/callable/field IDs plus snapshot-local binding/expression IDs.
- `phalcom-semantic/src/session.rs` already has `SemanticWorkspaceSession`; this spec extends it instead of inventing a replacement.
- `phalcom-semantic/src/snapshot.rs` already publishes immutable formal products and callable analyses.
- `phalcom-semantic/src/presentation.rs` already has a pure initial presentation index; this spec evolves it to machine-readable snapshot publication.
- `phalcom-semantic/src/surface.rs` and `dispatch.rs` already own canonical compiler member surfaces and dispatch.
- `phalcom-semantic/src/scope.rs` is currently only a minimal `ScopeTable`; it is insufficient for source semantic queries.
- `phalcom-semantic/src/db/key.rs` already owns typed formal query keys and is the correct place to extend source/advisory query ownership.
- `phalcom-lsp/src/semantic/ids.rs` still has duplicate URI-backed/module/class/callable/field identities.
- `phalcom-lsp/src/semantic/scope.rs` still independently builds lexical scope/binding identity.
- `phalcom-lsp/src/semantic/occurrence.rs` still independently builds exact semantic occurrences.
- `phalcom-lsp/src/semantic/facts.rs` contains the valuable non-type `ValueShape` abstract domain, confidence, provenance and contribution tables.
- `phalcom-lsp/src/semantic/engine.rs` still owns a full independent mutable semantic state, module/callable dependency logic and advisory solving.
- `phalcom-lsp/src/semantic/snapshot.rs` currently reconciles LSP and compiler worlds by scanning formal callable analyses and comparing string selector/name identity.
- `phalcom-lsp/src/analysis_service.rs` already has mature worker/debounce/cancellation/workspace-scan orchestration; final lifecycle replacement is intentionally deferred to Part 3.

## 64.2 Scope verification

This specification does not require:

- redesigning Part 1 formal evidence/status semantics;
- turning advisory evidence into formal assumptions;
- final deletion of the LSP async worker or all protocol facade methods;
- final removal of `run_static_workspace_analysis(...)` production reconstruction;
- full project/open-close/rename lifecycle redesign;
- broad new formal AST coverage;
- arbitrary persistent local binding identity across edits.

Those exclusions prevent Part 2 from swallowing Part 3 or destabilizing the formal checker.

## 64.3 Representation verification

The central representations were checked for the following failure modes:

- no source range as semantic identity;
- no URI string as canonical module identity;
- no selector string as canonical callable identity;
- no unguarded local site ID crossing snapshots;
- no advisory status hidden inside `Unknown` shape;
- no advisory confidence conflated with formal evidence status;
- no advisory→formal feedback edge;
- no presentation-time semantic inference;
- no second module/dispatch/invalidation graph;
- no nondeterministic hash iteration in site/fingerprint identity;
- no whole-workspace position/references scan as the target query algorithm.

## 64.4 Self-review / no-placeholder gate

The implementation tasks define concrete files, interfaces, algorithms, negative constraints and focused tests. Every Part 2 requirement has an explicit representation, algorithm, ownership rule, or test gate; no task delegates a required semantic decision to unspecified future work.

## 64.5 Final interpretation rule

Where current `main` still exposes pre-Part-1 types such as `EvidenceAuthority`, implement Part 2 against the **post-Part-1 API**, not by preserving obsolete compatibility semantics. Repository paths and ownership observations remain relevant; Part 1's final representation wins for formal epistemic types.

---

# 65. Recommended commit sequence

Keep commits reviewable and bisectable:

```text
1. semantic: add snapshot-scoped source site identities
2. semantic: own lexical source scope index
3. semantic: own canonical occurrence index
4. semantic: attach formal products to source sites
5. semantic: publish source index and formal projection
6. semantic: add canonical advisory shape domain
7. semantic: port advisory flow and expression analysis
8. semantic: port advisory interprocedural solver to semantic db
9. semantic: publish advisory products in snapshots
10. lsp: switch semantic identities and snapshot facade to compiler products
11. lsp: remove duplicate advisory semantic authority
12. test: harden single-world identity/advisory incrementality gates
```

Do not combine Tasks 1–9 and LSP deletion into one unreviewable migration commit. The compiler-owned replacements must have parity tests before old code is removed.

---

# 66. Final target dataflow

After Part 2, the intended dataflow is:

```text
                        phalcom-modules
                             │
                 canonical module/link products
                             │
                             ▼
                  phalcom-semantic session
                             │
           ┌─────────────────┼─────────────────┐
           │                 │                 │
           ▼                 ▼                 ▼
      formal checker    source index       surfaces/dispatch
   TypeKnowledge/etc.   sites/scopes/      canonical IDs
           │            occurrences            │
           │                 │                 │
           └────────────┬────┴─────────────────┘
                        ▼
              compiler advisory analysis
              ValueShape / summaries
                        │
                        ▼
                one SemanticSnapshot
          formal + source + advisory products
                        │
                        ▼
                pure presentation views
                        │
                        ▼
                 phalcom-lsp adapters
               URI / LSP protocol only
```

There is no semantic arrow from LSP back into formal truth, and no advisory arrow back into the formal checker.

That is the Part 2 definition of **single-world semantic ownership**.
