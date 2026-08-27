# Phalcom Single Semantic World — `phalcom-lsp` Semantic Retirement Technical Specification

**Status:** Proposed architectural closure gate before semantic-correctness Part 4 implementation  
**Repository:** `aureat/phalcom-lang`  
**Grounded branch:** `main`  
**Grounded HEAD:** `24919cd26019c6b5ffa72b069fa4692255ab0108`  
**Grounded date:** 2026-08-27  
**Supersedes as implementation guidance:** the transitional dual-world assumptions that remain in Part 3 documentation and in `phalcom-lsp/src/semantic/`  
**Primary goal:** complete Parts 1–3 by removing the remaining duplicate semantic world from the LSP before further semantic work is added.

---

## 1. Executive decision

Phalcom must have one implementation of language semantics.

The architectural invariant is:

> `phalcom-semantic` is the only owner of Phalcom semantic identity, semantic inference, formal checking, advisory value knowledge, dispatch, hierarchy, scope meaning, module meaning, source semantic identity, semantic invalidation, semantic presentation products, and immutable semantic snapshots.

`phalcom-lsp` remains responsible for editor protocol concerns:

- `tower-lsp` request/notification handling;
- current open buffers;
- source text parsing needed for syntax recovery;
- byte/UTF-16 conversion;
- latest-wins scheduling and cancellation;
- progressive workspace discovery;
- immutable snapshot publication/pinning;
- stale-source policy;
- completion/snippet construction;
- Markdown and LSP object rendering;
- diagnostics conversion;
- lexical semantic-token fallback;
- virtual document transport.

It must not contain:

- an alternative semantic database;
- an alternative semantic snapshot;
- language-semantic IDs;
- a second scope graph;
- a second occurrence/target identity model;
- a second dispatch implementation;
- a second module-semantic graph;
- branch/value joins;
- callable fixed-point inference;
- semantic parameter propagation;
- semantic field inference;
- semantic import resolution;
- semantic invalidation;
- semantic receiver inference;
- source-to-semantic reconstruction from AST on request paths.

This is not a “prefer the compiler path” cleanup. It is a physical and conceptual deletion of the old LSP semantic implementation.

---

## 2. Why this is a Part 3 closure gate

Current `main` already contains Part 4 planning material under:

`docs/impl/semantic/semantic-correctness/part-4/`

However, the current Part 3 checklist still records the following as incomplete or transitional:

- routing the worker through one `SemanticWorkspaceSession`;
- deleting the duplicate LSP semantic engine/database;
- deleting duplicate semantic IDs;
- deleting duplicate scope/occurrence/dispatch/module graph/advisory machinery;
- removing remaining request-time AST semantic-surface reconstruction;
- completing compiler-owned hover/inlay/completion cutovers;
- proving exactly one owner for project/module identity;
- proving exactly one owner for formal semantics;
- proving exactly one owner for advisory semantics;
- proving one immutable semantic snapshot is consumed by semantic LSP requests.

Therefore this work is best treated as **Part 3 architectural closure**, or informally **Part 3.5: Single Semantic World Retirement**.

Part 4 implementation should be considered blocked until this specification’s definition of done passes. Otherwise every Part 4 change creates pressure either to update the legacy LSP engine again or to widen compatibility bridges.

---

## 3. Repository-grounded current state at `24919cd…`

The handoff was written against `e1c8764bb85f4e9d9dcab89e2da06da3a03881b9`. Current `main` has advanced to:

`24919cd26019c6b5ffa72b069fa4692255ab0108`

The relevant architecture has not yet been retired. In several places, Part 3 has improved canonical consumption while preserving the old ownership shell.

### 3.1 `phalcom-lsp/src/semantic/` remains a complete semantic subsystem

At current HEAD, the directory still contains the old implementation, including the following modules:

```text
phalcom-lsp/src/semantic/
├── analyzer.rs
├── callable.rs
├── core_source.rs
├── dispatch.rs
├── engine.rs
├── facts.rs
├── flow.rs
├── ids.rs
├── infer.rs
├── invalidation.rs
├── module_graph.rs
├── occurrence.rs
├── query.rs
├── scope.rs
├── snapshot.rs
├── source.rs
├── surface.rs
└── mod.rs
```

The exact deletion set must be re-enumerated immediately before implementation, but current HEAD still exposes the package through:

```rust
// phalcom-lsp/src/lib.rs
pub mod semantic;
```

`semantic/mod.rs` still describes itself as a VM-free live semantic database and re-exports or defines LSP-owned semantic concepts such as:

```text
SemanticDb
SemanticEngine
SemanticSnapshot
FileSemanticSnapshot
CallableId
ClassId
FieldId
ModuleId
SemanticTarget
ScopeGraph
InferredValue
ValueShape
Confidence
CallableSummary
FieldFact
ParameterFact
```

This is not presentation-only compatibility code.

### 3.2 The LSP semantic snapshot still wraps the compiler snapshot

`phalcom-lsp/src/semantic/snapshot.rs` currently owns an LSP `SemanticSnapshot` containing legacy files/classes/summaries/facts/module mappings and:

```rust
compiler_snapshot: Option<Arc<phalcom_semantic::SemanticSnapshot>>
```

It also retains:

```rust
canonical_callables: Arc<BTreeMap<legacy CallableId, canonical CallableId>>
```

and conversion routines between canonical and LSP target/occurrence identities.

The most important consequence is not memory overhead. It is that the published object still says the old semantic world is the outer authority and the compiler snapshot is an optional attachment.

The target must invert this completely: there is no outer semantic wrapper.

### 3.3 `SemanticDb` is still an LSP-owned semantic database

`phalcom-lsp/src/semantic/mod.rs` defines the current protocol-owned `SemanticDb` with two publication pointers:

```rust
pub struct SemanticDb {
    current: RwLock<Arc<SemanticSnapshot>>,
    compiler_current: RwLock<Option<Arc<CompilerSemanticSnapshot>>>,
    // ...
}
```

It also exposes many semantic query helpers and test-only update paths backed by the legacy `SemanticEngine`.

After this cutover there must be no LSP type named `SemanticDb`.

A publication cell is allowed. A semantic database is not.

### 3.4 `AnalysisService` still runs two semantic systems

Current `phalcom-lsp/src/analysis_service.rs` imports:

```rust
use crate::semantic::{
    CompilerSemanticSnapshot,
    FileRevision,
    SemanticDb,
    SemanticEngine,
    SemanticGeneration,
    SemanticSnapshot,
    SourceAnalysisDepth,
};
```

The worker owns both:

```rust
let mut engine = SemanticEngine::new_with_counters(...);
let mut compiler_workspace_state = CompilerWorkspaceState::default();
```

where `CompilerWorkspaceState` contains a persistent canonical:

```rust
phalcom_semantic::SemanticWorkspaceSession
```

Ordinary edit processing still performs, conceptually:

```text
coalesced LSP mutation batch
        │
        ▼
legacy SemanticEngine.apply_mutations...
        │
        ▼
legacy generation / facts / flow / invalidation
        │
        ▼
refresh_compiler_workspace(...)
        │
        ▼
SemanticWorkspaceSession / WorkspaceModuleSession
        │
        ▼
canonical SemanticSnapshot
        │
        ▼
engine.set_compiler_analysis(...)
        │
        ▼
publish_engine(&SemanticDb, &SemanticEngine)
```

So the canonical session is persistent, but it is still hosted behind the legacy engine.

### 3.5 Canonical workspace publication is rebuilt from an LSP catalog

`publish_persistent_compiler_workspace` currently reconstructs the compiler module-session input from `source_catalog`:

- computes current source IDs;
- removes missing sources;
- clears and rebuilds a URI↔module document map;
- builds a full list of overlays;
- calls `WorkspaceModuleSession::set_overlays_with_programs`;
- updates `SemanticWorkspaceSession`;
- adds builtin URI aliases.

This is already better than creating a fresh semantic session, but it does not use the canonical module session as the direct source-lifecycle owner. The worker still mirrors state into `source_catalog` and then replays the catalog into the compiler session.

The target should use canonical source mutations directly.

### 3.6 `WorkspaceModuleSession` already owns the correct lifecycle

Current `phalcom-modules/src/session.rs` already owns:

```rust
ProjectUniverse
OverlaySourceProvider<FilesystemSourceProvider>
modules_by_source
sources_by_module
standalone_projects
linked program
generation
```

and supports:

```rust
WorkspaceSourceMutation::SetOverlay
WorkspaceSourceMutation::RemoveOverlay
WorkspaceSourceMutation::RefreshDisk
WorkspaceSourceMutation::RemoveSource
```

It also:

- preserves project and module identity;
- parses or accepts recovered programs;
- handles overlay precedence;
- falls back to disk after close;
- removes sources;
- resolves project ownership;
- rolls back a failed `set_overlays_with_programs` batch.

This means the LSP should stop implementing source/module semantic lifecycle beside it.

One canonical gap remains: a **transactional heterogeneous mutation batch** is needed so one coalesced LSP batch can contain updates, removals, closes, and disk refreshes while rebuilding/linking once.

### 3.7 `RequestContext` still pins two worlds

Current `phalcom-lsp/src/request_context.rs` contains:

```rust
semantic: Arc<crate::semantic::SemanticSnapshot>,
compiler: Option<Arc<CompilerSemanticSnapshot>>,
module: Option<legacy ModuleId>,
source_match: SourceMatch,
```

and computes the canonical module through the legacy wrapper’s document map.

This structure is the clearest remaining expression of the dual-world architecture.

The target is:

```rust
pub struct RequestContext {
    pub uri: Url,
    pub document: DocumentSnapshot,
    pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
    pub module: Option<phalcom_modules::ModuleId>,
    pub source_match: SourceMatch,
}
```

A semantic request context should never contain two snapshots.

### 3.8 `FileRevision` is still a duplicate revision vocabulary

`phalcom-lsp/src/documents.rs` imports:

```rust
crate::semantic::FileRevision
```

and uses it in:

```rust
Document
DocumentSnapshot
DocumentStore
CachedSource
PendingWork
AnalysisService
```

Canonical source lifecycle already has:

```rust
phalcom_modules::SourceRevision
```

The cutover should replace `FileRevision` with `SourceRevision`. This is part of deleting the old semantic identity/revision world, not a cosmetic rename.

### 3.9 Completion is canonical in important paths, but receiver semantics are duplicated in LSP

`phalcom-lsp/src/completion.rs` still has both:

```rust
SemanticResolvedReceiver
CompilerResolvedReceiver
```

The compiler completion path directly consumes canonical declaration surfaces, hierarchy, signatures, and visibility.

However, `backend.rs::compiler_receiver_for_range` manually performs semantic receiver resolution by combining:

- source-scope name resolution;
- canonical declaration targets;
- canonical field surfaces;
- formal binding facts;
- advisory binding facts;
- expression source sites;
- source AST initializer recognition;
- hierarchy ownership.

This is canonical data, but the algorithm is still semantic logic owned by LSP.

It belongs in `phalcom-semantic`.

### 3.10 Hover is canonical-first but still converts back to legacy presentation surfaces

Current compiler-backed hover paths still construct or consult legacy identities/surfaces for presentation details.

The missing canonical facts are mostly source/presentation metadata, not semantic reasoning:

- callable source kind;
- declaration/member source ranges;
- Phaldoc attachment anchor;
- convenient source member presentation.

The correct repair is to publish those facts canonically, not to preserve a legacy semantic engine.

### 3.11 Signature help still has parallel legacy and compiler renderers

`phalcom-lsp/src/signature_help.rs` currently contains both:

```rust
render_signature_help(
    member: &legacy MemberSurface,
    formal: Option<&legacy FormalCallablePresentation>,
    advisory: Option<&legacy CallableSignature>,
    ...
)

render_compiler_signature_help(
    signature: &phalcom_semantic::CallableSemanticSignature,
    store: &phalcom_semantic::TypeStore,
    advisory: Option<&phalcom_semantic::AdvisoryCallableSummary>,
    ...
)
```

Once exact-source requests are canonical and stale requests are syntax-only, the legacy renderer should be deleted.

### 3.12 Inlay hints remain heavily coupled to `FileSemanticSnapshot`

`phalcom-lsp/src/inlay_hints.rs` still imports:

```text
CompilerSemanticSnapshot
FileSemanticSnapshot
InferredValue
SemanticBindingKind
SemanticDb
SemanticSnapshot
ValueShape
```

and walks old `file_snapshot.source.scopes` / local facts for substantial hint generation.

The current compiler cutover is partial. This is one of the largest feature migrations and must be completed before deleting the semantic package.

### 3.13 Semantic tokens are canonical-refined but still type-coupled to the old database

`phalcom-lsp/src/semantic_tokens.rs` currently imports:

```rust
crate::semantic::{SemanticDb, SemanticOccurrenceKind}
```

The lexical tokenization itself correctly belongs in LSP. Compiler-owned occurrence classification should be the only semantic refinement.

### 3.14 `WorkspaceIndex` is explicitly transitional

`phalcom-lsp/src/index.rs` now documents itself as a text-only compatibility index. That is progress, but it still owns:

- selector definitions;
- selector references;
- class membership;
- inheritance-ish class metadata;
- member kinds;
- workspace symbols.

Those are semantic-looking products.

For the completed architecture, `WorkspaceIndex` should be deleted, not renamed into another pseudo-semantic fallback.

Workspace discovery may stay. Workspace semantic indexing must be canonical.

### 3.15 Core source handling mixes transport and semantics

`phalcom-lsp/src/semantic/core_source.rs` currently does both:

1. protocol/source selection:
   - configured core path;
   - workspace conventional path;
   - bundled fallback;
   - physical/virtual URI handling;

2. language semantics:
   - `build_core_surface`;
   - native-surface ingestion;
   - class/member construction;
   - native return shape projection;
   - dispatch side and visibility synthesis.

Only the first category is an LSP concern.

Canonical `phalcom-semantic::core_surface` already contains merged source/native core presentation machinery. The second category must be deleted from LSP.

### 3.16 The direct `phalcom-native-surface` dependency is now architectural debt

`phalcom-lsp/Cargo.toml` still depends directly on:

```toml
phalcom-native-surface = { path = "../phalcom-native-surface" }
```

Current LSP code uses it in the legacy core/surface and hover paths.

The target dependency is:

```text
phalcom-lsp
├── phalcom-ast
├── phalcom-common
├── phalcom-modules
├── phalcom-semantic
├── tower-lsp
└── protocol/runtime utility crates
```

`phalcom-native-surface` should be reached through canonical semantic products, not consumed directly by the LSP.

---

## 4. Architectural invariants

The following invariants are normative.

### 4.1 One semantic owner

Only `phalcom-semantic` may define language-semantic state or reasoning.

`phalcom-modules` remains the canonical owner of project/module/source identity and linking infrastructure used by semantic analysis.

### 4.2 One workspace semantic session

A running LSP backend has exactly one persistent:

```rust
phalcom_semantic::SemanticWorkspaceSession
```

owned by the analysis worker.

There is no LSP `SemanticEngine`.

### 4.3 One semantic publication per accepted worker batch

A successful coalesced mutation batch produces at most one new canonical semantic publication.

No legacy semantic generation is computed before or after it.

### 4.4 One immutable semantic snapshot per semantic request

Every semantic answer within one request comes from one:

```rust
Arc<phalcom_semantic::SemanticSnapshot>
```

The snapshot ID cannot change while the request executes.

### 4.5 No canonical-to-legacy identity conversion

Canonical IDs may be converted only to protocol coordinates/objects at the last boundary.

Allowed:

```text
CallableId
  → SourceSiteId
  → SourceLocation + SourceRange
  → LSP Location
```

Forbidden:

```text
canonical CallableId
  → LSP-local CallableId
  → LSP-local MemberSurface
  → Location
```

### 4.6 Staleness reduces completeness, never semantic truth

For an exact source:

```text
canonical semantic result + syntax/presentation
```

For stale source:

```text
syntax-only or source-insensitive canonical information
```

For unmapped source:

```text
syntax-only
```

No stale/unmapped request may run a second semantic engine.

### 4.7 No request-path semantic reconstruction

Request handlers may parse/lex current text to recover syntax.

They may not rebuild:

- class surfaces;
- inheritance;
- dispatch;
- semantic scope;
- type/value facts;
- module semantic identities.

### 4.8 No protocol dependency in semantic crates

`phalcom-semantic` and `phalcom-modules` must not depend on:

```text
tower-lsp
lsp-types
phalcom-lsp
```

### 4.9 Core semantic meaning is canonical

Native/source core merging, core declarations, core method signatures, and core identities are owned by compiler infrastructure.

LSP may select, expose, or render source documents, but not construct core meaning.

---

## 5. Non-goals

This cutover does not:

- redesign the Phalcom type system;
- add Part 4 generic inference;
- add new abrupt-exit semantics;
- add new field lifecycle semantics;
- change runtime VM semantics;
- move `tower-lsp` into compiler crates;
- make the parser incrementally patch AST nodes;
- replace current LSP full-text synchronization;
- require request handlers to block on fresh semantic analysis;
- make stale results semantically speculative;
- merge all editor presentation into `phalcom-semantic`.

The purpose is ownership cleanup and deletion.

---

## 6. Target dependency graph

```text
             phalcom-ast
                 ▲
                 │ parsed/recovered Program
                 │
         phalcom-modules
       project/module/source identity
       overlays/linking/import resolution
                 ▲
                 │ WorkspaceModuleUpdate
                 │
        phalcom-semantic
       SemanticWorkspaceSession
       SemanticDb/query products
       canonical identity
       formal semantics
       advisory semantics
       dispatch/hierarchy/surfaces
       source semantic index
       semantic presentation
       immutable SemanticSnapshot
                 ▲
                 │ read-only Arc
                 │
           phalcom-lsp
       scheduling / documents
       syntax recovery
       UTF-16 conversion
       stale-source policy
       LSP rendering / notifications
```

Forbidden dependency arrows:

```text
phalcom-semantic ─X→ phalcom-lsp
phalcom-semantic ─X→ tower-lsp
phalcom-modules  ─X→ phalcom-lsp
```

---

## 7. Target LSP runtime architecture

### 7.1 Publication cell

Create a small protocol-owned publication primitive, for example:

```rust
// phalcom-lsp/src/publication.rs

use std::sync::{Arc, RwLock};

#[derive(Default)]
pub(crate) struct SemanticPublication {
    current: RwLock<Option<Arc<phalcom_semantic::SemanticSnapshot>>>,
}

impl SemanticPublication {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn snapshot(
        &self,
    ) -> Option<Arc<phalcom_semantic::SemanticSnapshot>> {
        self.current
            .read()
            .expect("semantic publication lock poisoned")
            .clone()
    }

    pub(crate) fn publish(
        &self,
        snapshot: Arc<phalcom_semantic::SemanticSnapshot>,
    ) {
        *self
            .current
            .write()
            .expect("semantic publication lock poisoned") = Some(snapshot);
    }
}
```

This type owns no query API, identity, facts, invalidation, or inference.

It should deliberately not be called `SemanticDb`.

### 7.2 Worker state

The analysis worker should converge on:

```rust
struct AnalysisWorkerState {
    semantic: phalcom_semantic::SemanticWorkspaceSession,
}
```

Other scheduler-only state remains outside or alongside it:

```text
PendingWork
workspace scan cursor
open-document set
source epochs
closed-source text cache
performance counters
status tracker
```

There is no legacy `SemanticEngine`.

### 7.3 Update flow

Target:

```text
didOpen/didChange/didClose/filesystem scan
            │
            ▼
DocumentStore + PendingWork
            │
            ▼
AnalysisService coalesces latest revisions
            │
            ▼
canonical WorkspaceSourceMutation batch
            │
            ▼
SemanticWorkspaceSession
            │
            ├── WorkspaceModuleSession
            │     project/source/module identity
            │     overlay/disk lifecycle
            │     one link rebuild
            │
            └── semantic DB update
                  one invalidation/recompute pass
            │
            ▼
SemanticWorkspacePublication
            │
            ├── Arc<SemanticSnapshot>
            ├── SemanticPublicationEffects
            └── SemanticUpdateStats
            │
            ▼
SemanticPublication.publish(snapshot)
            │
            ▼
LSP refresh/diagnostic notifications
```

---

## 8. Canonical API gaps that must be closed

The correct migration is not to copy LSP algorithms into a `legacy` namespace. It is to add the smallest canonical query/source products needed by multiple IDE consumers.

### 8.1 Gap A — transactional heterogeneous module mutations

`WorkspaceModuleSession` already supports individual mutations and a batch of overlays. The worker needs a batch that can combine:

```text
SetOverlay
RemoveOverlay
RefreshDisk
RemoveSource
```

with one link rebuild and atomic rollback.

Add a canonical batch input in `phalcom-modules`, conceptually:

```rust
pub enum WorkspaceSourceBatchMutation {
    SetOverlay {
        source: SourceLocation,
        text: Arc<str>,
        revision: SourceRevision,
        recovered_program: Option<Arc<Program>>,
    },
    RemoveOverlay {
        source: SourceId,
    },
    RefreshDisk {
        source: SourceLocation,
        revision: SourceRevision,
    },
    RemoveSource {
        source: SourceId,
    },
}
```

and:

```rust
impl WorkspaceModuleSession {
    pub fn apply_batch<I>(
        &mut self,
        mutations: I,
    ) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError>
    where
        I: IntoIterator<Item = WorkspaceSourceBatchMutation>;
}
```

Requirements:

- stage every read/parse/identity resolution before committing where possible;
- snapshot mutable lifecycle maps necessary for rollback;
- apply overlay provider changes coherently;
- increment module-session generation once;
- call `rebuild` exactly once;
- on failure restore previous maps, linked program, generation, and overlays;
- preserve current recovered-program behavior for syntax-error editing.

Then expose from `SemanticWorkspaceSession`:

```rust
pub fn apply_module_mutations<I>(
    &mut self,
    mutations: I,
) -> Result<SemanticWorkspacePublication, WorkspaceModuleSessionError>
where
    I: IntoIterator<Item = WorkspaceSourceBatchMutation>;
```

which calls one module batch and one semantic update.

This removes the need for LSP `source_catalog` replay.

### 8.2 Gap B — canonical reverse source lookup

`ModuleQueryProducts` currently stores:

```rust
sources: Arc<BTreeMap<ModuleId, SourceLocation>>
```

and `ModuleQueryFacade` exposes module → source through `definition_source`.

RequestContext needs source → module without consulting a legacy document map and without filesystem resolution.

Extend canonical module query products with a reverse index:

```rust
pub struct ModuleQueryProducts {
    // existing...
    pub source_modules: Arc<BTreeMap<SourceId, ModuleId>>,
    pub path_modules: Arc<BTreeMap<PathBuf, ModuleId>>,
}
```

The reverse maps are derived products of `sources`, not new authority.

Expose:

```rust
impl ModuleQueryFacade<'_> {
    pub fn module_for_source(
        &self,
        source: &SourceId,
    ) -> Option<&ModuleId>;

    pub fn module_for_display_path(
        &self,
        path: &Path,
    ) -> Option<&ModuleId>;
}
```

`module_for_display_path` is a pure snapshot lookup. It must not canonicalize or read the filesystem.

This replaces `DocumentModuleMap` as semantic module authority.

### 8.3 Gap C — canonical source member metadata

Current `SourceSite` records canonical identity, name-range, and structural site kind. That is enough for basic navigation but insufficient for all hover/inlay presentation currently taken from legacy `MemberSurface`.

Add compiler-owned source metadata to `SourceScopeIndex` or a sibling source product.

Recommended shapes:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceCallableKind {
    Method,
    Getter,
    Setter,
    IndexGetter,
    IndexSetter,
    Constructor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationSourceInfo {
    pub declaration: DeclarationId,
    pub declaration_site: SourceSiteId,
    pub declaration_range: SourceRange,
    pub name_range: SourceRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableSourceInfo {
    pub callable: CallableId,
    pub declaration_site: SourceSiteId,
    pub declaration_range: SourceRange,
    pub name_range: SourceRange,
    pub kind: SourceCallableKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldSourceInfo {
    pub field: FieldId,
    pub declaration_site: SourceSiteId,
    pub declaration_range: SourceRange,
    pub name_range: SourceRange,
}
```

Populate these in `source_index/builder.rs`, where the AST distinction already exists.

Do not attach LSP Markdown, `Url`, or protocol ranges.

Benefits:

- hover can harvest Phaldoc from canonical source ranges;
- definition locations are direct;
- member kind does not require a legacy surface;
- return inlay placement can use current parsed source plus canonical member identity;
- semantic tokens can classify declarations without re-deriving identity.

### 8.4 Gap D — canonical editor semantic query facade

Do not make `backend.rs` coordinate the compiler’s individual maps.

Create:

```text
phalcom-semantic/src/editor.rs
```

This is a protocol-neutral read-only facade over one immutable `SemanticSnapshot`.

Recommended public types:

```rust
pub enum ReceiverMode {
    Instance,
    ClassObject,
}

pub struct ReceiverAlternative {
    pub declaration: DeclarationId,
    pub mode: ReceiverMode,
}

pub struct ResolvedReceiver {
    pub alternatives: Vec<ReceiverAlternative>,
}

pub struct AccessContext {
    pub lexical_owner: Option<DeclarationId>,
    pub privileged: bool,
}

pub enum EditorMemberTarget {
    Callable(CallableId),
    Field(FieldId),
}

pub struct EditorMember {
    pub target: EditorMemberTarget,
    pub owner: DeclarationId,
    pub side: DispatchSide,
    pub visibility: MemberVisibility,
}
```

Recommended facade:

```rust
pub struct EditorSemanticQuery<'a> {
    snapshot: &'a SemanticSnapshot,
}

impl SemanticSnapshot {
    pub fn editor(&self) -> EditorSemanticQuery<'_>;
}
```

Queries should include:

```rust
target_at(module, offset)

definition_sites(target)

reference_sites(target)

visible_symbols_at(module, offset)

resolve_receiver_at(module, receiver_range)

members_for_receiver(receiver, access)

resolve_member(receiver, selector, access)

callable_source(callable)

declaration_source(declaration)

field_source(field)

enclosing_callable_at(module, offset)

enclosing_declaration_at(module, offset)
```

The facade must:

- compose existing canonical source/formal/advisory products;
- use canonical hierarchy and surfaces;
- perform semantic visibility filtering;
- never parse strings as a semantic fallback;
- never depend on LSP types;
- return unknown/none when canonical evidence is insufficient.

### 8.5 Gap E — move receiver resolution out of `backend.rs`

The current algorithm in `compiler_receiver_for_range` is the primary semantic algorithm that still lives outside `phalcom-semantic`.

Move it into `EditorSemanticQuery::resolve_receiver_at`.

The canonical implementation should handle:

```text
self
super
lexical binding
import target
class declaration target
field target
formal binding fact
advisory binding fact
formal expression fact
advisory expression fact
class-object vs instance alternatives
union alternatives
initializer/source-site fallback only if it is canonical semantic reasoning
```

If an initializer expression is missing formal/advisory coverage and interpreting it would amount to inference, the canonical query may use compiler-owned source data to derive a bounded result. LSP must not do so.

The result must be deterministic and snapshot-local.

### 8.6 Gap F — canonical language-value presentation helper

`signature_help.rs` currently contains an LSP-local recursive renderer for canonical `phalcom_semantic::ValueShape`.

That is not protocol-specific. It is a language semantic presentation.

Add to `phalcom-semantic/src/presentation.rs`, conceptually:

```rust
pub struct ValueShapePresenter;

impl ValueShapePresenter {
    pub fn present(shape: &ValueShape) -> String;
}
```

or:

```rust
pub fn present_value_shape(shape: &ValueShape) -> String;
```

Then LSP renderers consume text.

This prevents completion, hover, signature help, and inlay hints from each inventing a slightly different textual model.

### 8.7 Gap G — canonical callable/member presentation

Add protocol-neutral projections that bind together information currently gathered in LSP:

```rust
pub struct CallablePresentation<'a> {
    pub id: CallableId,
    pub signature: &'a CallableSemanticSignature,
    pub source: Option<&'a CallableSourceInfo>,
    pub formal: FormalPresentation,
    pub advisory: Option<&'a AdvisoryCallableSummary>,
}

pub struct DeclarationPresentation<'a> {
    pub id: DeclarationId,
    pub source: Option<&'a DeclarationSourceInfo>,
    pub superclass: Option<&'a DeclarationId>,
}

pub struct FieldPresentation<'a> {
    pub id: FieldId,
    pub source: Option<&'a FieldSourceInfo>,
    pub formal: Option<FormalPresentation>,
    pub advisory: Option<&'a AdvisoryFact>,
}
```

The exact ownership/lifetime shape may vary. The invariant is that feature handlers should not reproduce the same semantic composition.

---

## 9. Request context model

### 9.1 Semantic request context

Replace the current dual-snapshot structure with:

```rust
pub struct RequestContext {
    pub uri: Url,
    pub document: DocumentSnapshot,
    pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
    pub module: Option<phalcom_modules::ModuleId>,
    pub source_match: SourceMatch,
}
```

`SourceMatch` remains:

```rust
pub enum SourceMatch {
    Exact,
    Stale,
    Unmapped,
}
```

### 9.2 Context construction

Backend should:

1. snapshot the current document;
2. pin `SemanticPublication::snapshot()`;
3. derive the file path from `Url` without filesystem I/O;
4. look up canonical module through `snapshot.module_queries().module_for_display_path(path)`;
5. compare `document.text` against `snapshot.sources[module].text`.

Conceptually:

```rust
let semantic = self.publication.snapshot()?;
let module = uri
    .to_file_path()
    .ok()
    .and_then(|path| {
        semantic
            .module_queries()
            .module_for_display_path(&path)
            .cloned()
    });

let source_match = match module
    .as_ref()
    .and_then(|module| semantic.sources().get(module))
{
    Some(source) if source.text.as_ref() == document.text.as_ref() => {
        SourceMatch::Exact
    }
    Some(_) => SourceMatch::Stale,
    None => SourceMatch::Unmapped,
};
```

### 9.3 Startup before first semantic publication

Do not reintroduce an empty legacy semantic snapshot just to make this type non-optional.

Use one of these two acceptable patterns:

**Preferred:** bootstrap the canonical session before declaring initial analysis ready, so normal semantic requests have a publication.

**Fallback:** have `Backend::request_context` return `Option<RequestContext>` and let handlers execute syntax-only behavior when no canonical publication exists.

Never create a fake LSP semantic snapshot.

---

## 10. Source revision model

Replace LSP-owned `FileRevision` with canonical:

```rust
phalcom_modules::SourceRevision
```

Affected protocol state includes:

```text
Document.revision
DocumentSnapshot.revision
DocumentStore::open_or_update*
DocumentStore::bump_revision
CachedSource.revision
PendingWork.file_updates
PendingWork.core_update if that path survives
AnalysisService enqueue methods
worker freshness checks
```

This does not make `SourceRevision` a semantic identity. It makes the source lifecycle use the same revision vocabulary as the canonical module session.

Client LSP document version remains separate:

```rust
Option<i32>
```

---

## 11. Analysis service redesign

### 11.1 Keep

Keep:

- worker thread;
- condvar;
- pending-work coalescing;
- latest-wins epoch;
- source epochs;
- open-document priority;
- scan budgets;
- cancellation checks;
- status events;
- log events;
- immutable publication;
- refresh notifications.

### 11.2 Remove

Remove from `AnalysisService`:

```text
Arc<crate::semantic::SemanticDb>
SemanticEngine
SourceAnalysisDepth
CompilerWorkspaceState
refresh_compiler_workspace
publish_persistent_compiler_workspace
publish_engine
legacy publication_effects
merge_publication_effects
document/module alias bridge
legacy core surface update
legacy import-closure semantic reconstruction
```

### 11.3 Constructor target

Current constructors take an LSP `SemanticDb`.

Target:

```rust
pub fn new(
    publication: Arc<SemanticPublication>,
) -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>)
```

and variants add cache/config as necessary.

`Backend::new` creates:

```rust
let publication = Arc::new(SemanticPublication::new());
let analysis = AnalysisService::new_with_cache(
    publication.clone(),
    closed_sources.clone(),
    ...
);
```

### 11.4 Worker batch translation

Convert `PendingWork` to canonical batch mutations:

```rust
Vec<phalcom_modules::WorkspaceSourceBatchMutation>
```

Examples:

Open/change:

```rust
SetOverlay {
    source: source_location,
    text,
    revision,
    recovered_program: Some(program),
}
```

Close:

```rust
RemoveOverlay {
    source: source_id,
}
```

Deleted file:

```rust
RemoveSource {
    source: source_id,
}
```

Closed-file refresh:

```rust
RefreshDisk {
    source: source_location,
    revision,
}
```

Then exactly once:

```rust
let result = state.semantic.apply_module_mutations(mutations);
```

### 11.5 Publication effects

Delete LSP semantic-diff synthesis.

Map directly from:

```rust
phalcom_semantic::SemanticPublicationEffects
```

to editor refresh decisions.

Recommended mapping:

```text
diagnostics_changed
    → republish affected diagnostics

formal_changed
advisory_changed
source_index_changed
    → inlay hint refresh

source_index_changed
declaration_index_changed
    → semantic token refresh

module_graph_changed
declaration_index_changed
source_index_changed
    → completion/navigation data changed
```

If the existing LSP `PublicationEffects` struct remains useful, make it a pure mapping of canonical effects, not a second fingerprint computation.

### 11.6 Failure policy

There are three classes:

1. **syntax recovery available**
   - feed recovered `Program`;
   - canonical session may publish partial semantic products.

2. **canonical module/session error**
   - keep last published snapshot;
   - emit structured analysis status/log;
   - do not substitute legacy semantics.

3. **superseded worker batch**
   - discard candidate;
   - do not publish;
   - keep current snapshot.

---

## 12. Workspace discovery and `WorkspaceIndex`

### 12.1 Progressive scanning stays

`workspace_scan.rs` remains an LSP scheduling/discovery component.

It may discover:

```text
physical path
URI
source text
parse result
revision/freshness metadata
```

It must feed discovered files into `SemanticWorkspaceSession`.

### 12.2 Remove manual semantic import closure

Current worker helper code follows relative imports itself to extend the catalog.

After canonical module lifecycle becomes direct, import meaning belongs to `WorkspaceModuleSession` / canonical module resolver.

Any scan helper that exists solely to reproduce semantic import closure should be deleted.

Workspace scanning can still choose which physical files to discover for performance, but it may not define module resolution semantics.

### 12.3 Delete `WorkspaceIndex`

Once:

- every discovered closed file is fed into the canonical module session;
- navigation/workspace symbols consume canonical source indexes;
- stale/unmapped behavior is syntax-only;

delete:

```text
phalcom-lsp/src/index.rs
```

and remove `dashmap` if no other code uses it.

Do not replace it with a renamed semantic compatibility index.

If a lightweight source discovery catalog remains useful, it must contain only source-transport data and should live with workspace scanning, not semantic query code.

---

## 13. Feature cutover contracts

### 13.1 Diagnostics

Exact source:

```text
DocumentStore syntax diagnostics
+
SemanticSnapshot diagnostics_for(canonical module)
```

Stale/unmapped:

```text
DocumentStore syntax diagnostics only
```

Delete dependence on:

```text
legacy SemanticSnapshot
DocumentModuleMap
legacy FileRevision matching
```

### 13.2 Definition

Exact source:

```text
offset
→ snapshot.editor().target_at(module, offset)
→ snapshot.editor().definition_sites(target)
→ canonical SourceLocation + SourceRange
→ LSP Location
```

Stale/unmapped:

Return no semantic definition unless a purely syntactic definition can be resolved without claiming semantic identity.

Do not query `WorkspaceIndex`.

### 13.3 References

Exact source:

```text
target_at
→ reference_sites(target)
→ source locations
→ LSP Location[]
```

No selector-text fallback.

### 13.4 Rename

Exact source only.

Resolve canonical target first. Build edits from canonical occurrence sites.

If source is stale/unmapped, refuse/return no rename rather than using selector spelling as semantic identity.

### 13.5 Workspace symbols

Use canonical declaration/callable/field source sites across the published snapshot.

`WorkspaceIndex::symbols_matching` disappears.

### 13.6 Completion

Keep syntax recovery in `completion.rs`:

```text
identifier prefix
receiver range
call shape
import path fragments
snippets
CompletionItem rendering
```

Move semantic work to canonical facade:

```text
visible symbols
receiver resolution
member enumeration
visibility
hierarchy walk
field/callable target identity
```

Delete:

```rust
SemanticResolvedReceiver
```

Rename:

```rust
CompilerResolvedReceiver
```

to:

```rust
ResolvedReceiver
```

or use the canonical type directly.

Exact source member completion is canonical.

Stale/unmapped member completion is absent. Syntax-visible non-member completion may remain.

### 13.7 Hover

Keep in LSP:

```text
keyword hover
Phaldoc lexical harvest
Markdown layout
LSP Hover conversion
syntax-only stale fallback
```

Canonical query provides:

```text
target
declaration/callable/field identity
signature
source member kind
source ranges
formal presentation
advisory presentation
hierarchy/source provenance
```

Delete old member/class surface conversion and canonical→legacy ID mapping.

### 13.8 Signature help

Keep syntax-only call-site recovery.

Exact source:

```text
CallSite
→ canonical receiver resolution
→ canonical callable resolution
→ CallableSemanticSignature
→ advisory summary if useful
→ render_compiler_signature_help
```

Then rename the compiler renderer to simply:

```rust
render_signature_help
```

Delete the old renderer and all old surface/signature types.

Stale/unmapped: return no semantic signature.

### 13.9 Inlay hints

Keep:

- hint policy;
- explicit annotation suppression;
- source placement calculation;
- LSP `InlayHint` construction.

Canonical source enumeration/query must supply:

- bindings;
- parameters;
- fields;
- callables/returns;
- closure parameters;
- formal facts;
- advisory facts.

Remove `FileSemanticSnapshot` and old local facts.

A useful canonical query is:

```rust
snapshot.editor().hint_sites(module, visible_range)
```

but it is also acceptable for LSP to enumerate canonical source sites and ask `semantic_site_at` per site, provided no semantic reasoning is rebuilt.

### 13.10 Semantic tokens

Keep lexer-driven base classification.

Exact source semantic refinement uses:

```text
canonical occurrences
canonical source sites
canonical target kinds
```

Stale/unmapped: lexical/AST syntactic refinement only.

Delete `SemanticDb` and legacy occurrence-kind imports.

---

## 14. Core source and virtual document cutover

### 14.1 Split transport from semantics

Delete semantic construction from:

```text
phalcom-lsp/src/semantic/core_source.rs
```

If source selection is still needed, create a protocol-focused module, for example:

```text
phalcom-lsp/src/core_documents.rs
```

Allowed responsibilities:

```text
configured physical source path selection
workspace source path preference for opening a document
builtin/canonical virtual URI mapping
serving read-only source text
URI ↔ builtin ModuleId adaptation
```

Forbidden responsibilities:

```text
building ClassSurface
building MemberSurface
merging native metadata
assigning semantic return shapes
creating semantic IDs
resolving dispatch
```

### 14.2 Canonical core presentation

Use:

```text
phalcom-semantic::core_surface
phalcom-semantic::ClassPresentation
phalcom-semantic::MethodPresentation
canonical snapshot module/source provenance
```

for core hovers and virtual source.

### 14.3 Configured sysroot semantics

If a configured core path is intended to alter semantic core meaning, the configuration must be modeled as a canonical compiler/session input before legacy core deletion.

It must not survive as an LSP-only semantic override.

If configured core is presentation/provenance-only, make that explicit and do not feed it into semantic construction.

The implementation phase should add a regression test locking the intended behavior before deleting `CoreSource::build_core_surface`.

---

## 15. Performance instrumentation cleanup

`phalcom-lsp/src/perf.rs` currently includes counters for the legacy semantic implementation:

```text
flow_passes
solver_rounds
callables_analyzed
dirty_callables_seeded
solver_callables_visited
solver_callables_changed
semantic_candidate_state_clones
published_file_products_reused
published_class_products_reused
published_summary_products_reused
parameter_sources_replaced
parameter_slots_touched
parameter_slots_changed
```

These counters should not remain as fake LSP ownership after the engine is deleted.

Split counters into:

### LSP-owned scheduler/protocol counters

Keep:

```text
source_updates_enqueued
source_updates_coalesced
source_updates_discarded
semantic_batches_started
semantic_batches_published
scan_batches_published
stale_batches_discarded
workspace_files_discovered
workspace_files_parsed
query_filesystem_canonicalizations
query_disk_reads
inlay_refresh_requests
semantic_token_refresh_requests
scan_directory_entries_consumed
scan_results_discarded_as_stale
scan_results_discarded_for_open_document
```

### Canonical semantic update statistics

Read from:

```rust
phalcom_semantic::SemanticUpdateStats
```

and log/report them without duplicating counter ownership.

The LSP should not increment a semantic solver counter itself.

---

## 16. Test ownership

### 16.1 Move/replace semantic behavior tests

Any LSP test whose subject is actually:

```text
type inference
flow join
dispatch correctness
subtyping
field facts
parameter propagation
callable fixed point
module semantic invalidation
semantic import identity
semantic source identity
```

must be migrated to `phalcom-semantic/tests` or `phalcom-modules/tests`.

### 16.2 LSP tests should cover boundaries

LSP tests should verify:

```text
document/position → canonical query
canonical result → LSP object
exact/stale/unmapped policy
UTF-16 conversion
snapshot pinning
latest-wins publication
scan lifecycle
completion snippets
Markdown
diagnostic conversion
semantic token encoding
refresh notification routing
```

### 16.3 Rewrite `single_world_cutover.rs`

Current test `worker_reuses_compiler_snapshot_store_across_edits` constructs:

```rust
phalcom_lsp::semantic::SemanticDb
```

just to read the embedded compiler snapshot.

Target:

```rust
let publication = Arc::new(SemanticPublication::new());
let (service, _) = AnalysisService::new(publication.clone());

...
let first = publication.snapshot().expect("canonical publication");
...
let second = publication.snapshot().expect("canonical publication");
```

The test should additionally prove:

- only one canonical snapshot pointer is published;
- TypeStore is reused across body edits;
- module identity remains stable;
- snapshot ID changes;
- no LSP semantic wrapper exists.

### 16.4 Add stale-source negative tests

Tests must explicitly prove:

```text
stale completion does not invoke semantic receiver fallback
stale hover does not invent class/member identity
stale definition does not use selector text
stale signature help does not resolve through old surfaces
stale rename does not run
```

Temporary incompleteness is the correct result.

---

## 17. Architecture regression gate

Add a dedicated test target, for example:

```text
phalcom-lsp/tests/semantic_boundary.rs
```

Because `phalcom-lsp` uses `autotests = false`, add it explicitly to `Cargo.toml`.

The test should verify:

1. `phalcom-lsp/src/semantic` does not exist.
2. `phalcom-lsp/src/index.rs` does not exist after WorkspaceIndex retirement.
3. no LSP source file declares:
   - `SemanticDb`;
   - `SemanticEngine`;
   - `ClassId`;
   - `CallableId`;
   - `FieldId`;
   - `SemanticTarget`;
   - `ScopeGraph`;
   - `InferredValue`;
   - an LSP-local `ValueShape`;
   - a semantic `DispatchResolver`.
4. `phalcom-lsp/Cargo.toml` has no direct `phalcom-native-surface` dependency.
5. `phalcom-semantic/Cargo.toml` contains no `tower-lsp`, `lsp-types`, or `phalcom-lsp`.
6. request-path source files contain no filesystem source read/canonicalization in semantic handlers.
7. no semantic fallback branch imports or invokes legacy inference/surface builders.

This gate is intentionally strict. Once the duplicate subsystem is deleted, allowing it to regrow should require an explicit architecture decision.

---

## 18. Filesystem and source-path rules

The request path must not perform:

```rust
std::fs::read_to_string(...)
Path::canonicalize(...)
```

to resolve semantic meaning.

Filesystem operations are allowed in:

```text
analysis worker
workspace scanner
configured source selection
closed-file source cache population
```

Semantic requests use immutable snapshot provenance.

URI conversion is protocol adaptation, not semantic resolution.

---

## 19. Detailed ownership matrix

| Concept | Owner after cutover | LSP role |
|---|---|---|
| Project identity | `phalcom-modules` | none |
| Module identity | `phalcom-modules` | carry canonical ID |
| Source identity | `phalcom-modules` | URI/path adaptation |
| Source revision | `phalcom-modules::SourceRevision` | generate/track editor revisions |
| Parser AST | `phalcom-ast` | cache current live parse |
| TypeStore | `phalcom-semantic` | read only |
| Declaration ID | `phalcom-semantic` | carry/read only |
| Callable ID | `phalcom-semantic` | carry/read only |
| Field ID | `phalcom-semantic` | carry/read only |
| Binding/source-site ID | `phalcom-semantic` | carry/read only |
| Scope semantics | `phalcom-semantic` | query only |
| Dispatch | `phalcom-semantic` | query only |
| Hierarchy | `phalcom-semantic` | query only |
| Formal proof/type facts | `phalcom-semantic` | present only |
| Advisory value facts | `phalcom-semantic` | present only |
| Module semantic graph | `phalcom-semantic` / modules infrastructure | query only |
| Semantic invalidation | `phalcom-semantic` | schedule update only |
| Occurrence identity | `phalcom-semantic` | convert to LSP ranges |
| Semantic diagnostics | `phalcom-semantic` | convert to LSP diagnostics |
| Phaldoc raw-text harvest | LSP presentation layer | yes |
| Keyword hover | LSP | yes |
| Syntax recovery | LSP + parser | yes |
| UTF-16 position mapping | LSP | yes |
| Completion snippets | LSP | yes |
| Stale-source policy | LSP | yes |
| Worker cancellation/latest-wins | LSP | yes |
| Workspace filesystem scan | LSP | yes |
| Native/core semantic merge | `phalcom-semantic` | consume presentation |
| Virtual core URI | LSP | yes |

---

## 20. Migration sequence

The implementation must use small commits, but the dependency order is strict.

### Phase A — canonical primitives first

1. Add transactional module mutation batch.
2. Add reverse source lookup to canonical module query products.
3. Add canonical source member metadata.
4. Add canonical editor semantic query facade.
5. Add canonical value/member presentation helpers.

No LSP deletion before these canonical gaps have tests.

### Phase B — publication spine

6. Add LSP publication cell.
7. migrate `FileRevision` → `SourceRevision`.
8. rewrite `AnalysisService` to own one `SemanticWorkspaceSession`.
9. delete compiler-workspace attachment/replay bridge.
10. derive refreshes from canonical publication effects.

At the end of Phase B, edits execute one semantic architecture.

### Phase C — request spine

11. rewrite `RequestContext` to one canonical snapshot.
12. canonicalize source→module lookup through snapshot module products.
13. standardize `Exact/Stale/Unmapped`.

At the end of Phase C, every semantic request pins one world.

### Phase D — feature cutover

14. diagnostics;
15. definition/references/rename/workspace symbols;
16. completion;
17. hover;
18. signature help;
19. inlay hints;
20. semantic tokens.

Each feature loses its legacy fallback in the same commit that canonical parity tests pass.

### Phase E — compatibility deletion

21. delete `WorkspaceIndex`;
22. split core document transport from semantic core construction;
23. remove direct native-surface use/dependency;
24. delete `phalcom-lsp/src/semantic/`;
25. remove `pub mod semantic`;
26. clean imports/docs/perf counters/tests.

### Phase F — mechanical proof

27. add architecture gate;
28. run full workspace verification;
29. measure one semantic update per edit;
30. mark Part 3 ownership items complete and unblock Part 4.

---

## 21. Compatibility strategy

There should be no long-lived compatibility layer.

Allowed temporary bridges during implementation:

- a temporary type alias for `SourceRevision` while call sites are migrated;
- a temporary adapter from canonical presentation to existing LSP renderer inputs;
- a temporary feature flag only if needed to bisect behavior.

Forbidden temporary bridges:

- aliasing old semantic IDs to canonical IDs while retaining old algorithms;
- retaining `SemanticEngine` after the worker uses the canonical session;
- retaining old fallback inference for stale requests;
- moving old semantic files into `phalcom-semantic`;
- wrapping the canonical snapshot in another semantic snapshot.

Every temporary adapter must have a deletion task in the same implementation plan.

---

## 22. Behavioral expectations during staleness

This cutover intentionally changes some transient IDE behavior.

Example:

```phalcom
foo.|
```

If the live document is newer than the semantic snapshot:

- identifier/prefix syntax recovery still works;
- keyword and lexical features still work;
- general syntax-visible completion may work;
- receiver-member semantic completion may temporarily return no items;
- definition/references/rename may temporarily return no semantic result;
- signature help may temporarily return no result;
- old semantic inference must not be run.

This is preferable to giving a fast but semantically inconsistent answer from a second engine.

Latest-wins analysis is responsible for making the stale interval short.

---

## 23. Performance acceptance

The cutover is expected to improve ordinary editing by removing duplicate work.

Measure at least:

```text
semantic update count per accepted edit batch
wall-clock worker batch time
peak retained semantic snapshot memory
number of AST semantic walks per request
module link rebuild count
callable recomputation count
source-index recomputation count
```

Hard expectations:

1. one canonical semantic update per accepted batch;
2. one module link rebuild per canonical mutation batch;
3. zero legacy flow/solver passes;
4. zero canonical→legacy identity index construction;
5. no request-time semantic AST surface reconstruction;
6. TypeStore remains reused across eligible edits;
7. unrelated canonical products remain reusable according to existing incrementality.

Do not require a specific percentage speedup as a correctness gate. The structural absence of duplicate work is the first requirement.

---

## 24. Release and rollback behavior

This should land as a sequence of small commits, but the repository must not end a release branch in a half-dual state.

If a canonical feature gap blocks migration:

1. keep that feature on the previous branch/commit;
2. implement the missing canonical product;
3. add canonical tests;
4. migrate the feature;
5. remove fallback immediately.

Do not restore legacy semantic inference as a “temporary reliability fix.”

The last known good semantic snapshot is the fallback publication for infrastructure failures.

---

## 25. Documentation changes

Update:

```text
phalcom-lsp/src/lib.rs
docs/impl/semantic/semantic-correctness/part-3/*
.agent skill/reference documents that describe current semantic ownership
ADR-0056 commentary if it claims LSP owns a live semantic database
```

The new architecture description should say:

```text
AnalysisService = scheduler
WorkspaceModuleSession = module/source lifecycle
SemanticWorkspaceSession = semantic analyzer
SemanticSnapshot = immutable semantic publication
RequestContext = one pinned semantic world
LSP feature modules = syntax recovery + protocol presentation
```

Part 4 documents should state that their implementation assumes this invariant.

---

## 26. Definition of done

The retirement is complete only when all of the following are true.

### Physical structure

```text
phalcom-lsp/src/semantic/ does not exist.
phalcom-lsp/src/index.rs does not exist.
phalcom-lsp/src/lib.rs does not export semantic.
phalcom-lsp has no direct phalcom-native-surface dependency.
```

### Runtime architecture

```text
AnalysisService owns/drives one SemanticWorkspaceSession.
No SemanticEngine exists in phalcom-lsp.
An accepted edit batch executes one module-link/semantic update architecture.
Canonical SemanticSnapshot is published directly.
RequestContext contains one Arc<phalcom_semantic::SemanticSnapshot>.
```

### Identity

```text
No LSP ClassId.
No LSP CallableId.
No LSP FieldId.
No LSP SemanticTarget.
No canonical_callables bridge.
No canonical_target_to_lsp semantic conversion.
```

### Semantic behavior

```text
No LSP flow engine.
No LSP inference engine.
No LSP dispatch engine.
No LSP scope graph.
No LSP module semantic graph.
No LSP semantic invalidation.
No LSP callable fixed-point solving.
No request-time AST semantic surface reconstruction.
```

### Feature behavior

```text
Diagnostics use canonical semantic diagnostics.
Definition/references/rename use canonical targets and source sites.
Completion uses canonical receiver/member queries.
Hover uses canonical presentation/source metadata.
Signature help uses canonical signatures.
Inlay hints use canonical source/formal/advisory sites.
Semantic-token semantic refinement uses canonical occurrences.
Stale/unmapped requests degrade to syntax-only behavior.
```

### Core

```text
Core semantic/native merge is outside phalcom-lsp.
Virtual/source document behavior remains protocol-owned.
```

### Tests

```text
canonical semantic tests own semantic correctness
LSP tests own protocol projection/staleness/snapshot consistency
single_world_cutover reads the direct publication cell
architecture boundary test passes
full workspace checks/tests pass
```

### Ownership proof

A reviewer can answer each question with exactly one owner:

```text
Who owns project/module identity?       phalcom-modules
Who owns formal semantics?              phalcom-semantic
Who owns advisory semantics?            phalcom-semantic
Who owns dispatch/hierarchy?            phalcom-semantic
Who owns semantic source identity?      phalcom-semantic
Who owns semantic invalidation?         phalcom-semantic
Who schedules analysis?                 phalcom-lsp
Who renders LSP objects?                phalcom-lsp
What snapshot does a request read?      one canonical SemanticSnapshot
```

---

## 27. Final architectural statement

The repository has already built most of the canonical machinery required for this cutover:

- persistent `SemanticWorkspaceSession`;
- persistent `WorkspaceModuleSession`;
- canonical source/module identities;
- canonical source index;
- formal projection;
- advisory workspace;
- canonical dispatch/hierarchy/surfaces;
- module query products;
- canonical semantic presentations;
- compiler-backed completion/navigation/diagnostics paths;
- immutable snapshot publication semantics;
- incremental effects/stats.

The remaining problem is ownership.

The compiler world is still attached to an LSP semantic world that should no longer exist.

The required final shape is:

```text
BEFORE

AnalysisService
  ├─ SemanticEngine -------------------- legacy semantics
  ├─ CompilerWorkspaceState
  │    └─ SemanticWorkspaceSession ----- canonical semantics
  └─ SemanticDb
       └─ legacy SemanticSnapshot
            └─ Option<canonical SemanticSnapshot>


AFTER

AnalysisService
  └─ SemanticWorkspaceSession
       └─ canonical SemanticSnapshot
              │
              ▼
       SemanticPublication
              │
              ▼
        RequestContext
              │
              ▼
   protocol/syntax/presentation only
```

No new Part 4 semantic capability should be implemented until the “AFTER” diagram is true in code.
