# Phalcom LSP Incrementality, Snapshot Coherence, and Structural-Sharing Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Use `superpowers:test-driven-development` for each implementation slice, `superpowers:systematic-debugging` for failures, and `superpowers:verification-before-completion` before declaring any task finished. Steps use `- [ ]` checkboxes for tracking.

**Goal:** Complete the second-stage performance and correctness optimization of `phalcom-lsp` after the async-worker work: make semantic computation proportional to the actual semantic delta, make every editor request internally generation/revision coherent, eliminate workspace-scale deep cloning and query-time filesystem work, and preserve Phalcom's canonical semantic identities exactly.

**Architecture:** Retain the current dedicated semantic worker, latest-wins scheduling, immutable publications, progressive workspace discovery, and unified flow machinery. Refine the design so the worker owns the only mutable `SemanticEngine`; published generations structurally share unchanged state; current-document requests pin one `DocumentSnapshot` and one `SemanticSnapshot`; body-only edits seed exact changed callables instead of entire dependent modules; parameter evidence is updated contribution-by-contribution; filesystem identity and disk refresh happen only on the worker/source-ingestion side; and semantic queries borrow directly from immutable tables instead of reconstructing them.

**Tech Stack:** Rust, Cargo, `tower-lsp`, Tokio notifications, dedicated `std::thread` semantic worker, `Arc`, `RwLock` only for publication/cache handles, `DashMap` open-document store, Phalcom AST, existing `BTreeMap`/`BTreeSet` deterministic semantic tables, TypeScript, `vscode-languageclient` 8.x, VS Code Extension Host tests.

**Recommended plan location:** `docs/superpowers/plans/2026-08-14-phalcom-lsp-incrementality-and-query-coherence.md`

---

## 1. Authority, checkpoint, and relation to the previous plan

The attached plan remains the authority for the first async-performance completion phase: one semantic worker, immutable publication, progressive scanning, unified flow results, callable worklists, contribution storage, closed-source caching, performance instrumentation, and resilient extension restart behavior. 

The new starting point is remote `main` at:

```text
38e0996578259ab47a9a28d95f2f59a0d1c893ac
perf(lsp): finalize async worker scheduling
```

This is 29 commits beyond the attached plan's `195ef13` checkpoint. Do **not** reimplement Tasks 8–13 wholesale. Treat their landed architecture as prerequisite infrastructure and modify it only where this plan explicitly identifies remaining gaps.

The re-audit confirms several important unfinished seams. `SemanticDb` still owns a mutex-protected mutable `SemanticEngine`, while `SemanticEngine` itself clones its state transactionally; `SemanticDb::apply_mutations_with_cancel` then adds a second engine clone before publication. `SemanticEngine::snapshot()` deep-clones file/class/summary products into new `Arc`s on every publication.  

The immutable query layer also reconstructs complete class/summary maps for individual operations such as `receiver_member`, `return_for_callable`, `class_for_name`, `returns_for_callables`, and `infer_expression`; it additionally returns deep owned clones for many simple lookups. 

Semantic member identity is not fully preserved. `ClassSurface` simultaneously keeps a side-blind `members: BTreeMap<String, MemberSurface>` and a side-aware `members_by_side`, so an already-resolved `CallableId { owner, selector, side }` can later be degraded to `(owner, selector)` and select the wrong class-side/instance-side declaration. 

`ModuleId::from_uri` still performs `std::fs::canonicalize`, and it is called from semantic query paths. Filesystem identity therefore remains coupled to interactive lookup. 

`ParameterContributions::replace_source` removes one source by scanning every parameter slot and then recomputes the joined value for every slot, so its data model is contribution-aware but its update algorithm is still global. 

The incremental callable solver initially queues every callable belonging to every affected input module, then reconstructs the complete current parameter-fact map after each analyzed callable. 

The progressive scanner limits the number of directories per turn but consumes every entry in each selected directory before yielding, meaning a single extremely wide directory can still monopolize the worker. 

The open-document store already exposes the right primitive—an owned `DocumentSnapshot` that releases the `DashMap` guard immediately—but many backend paths still execute semantic/query work inside `with_document`. 

Semantic token refinement uses whichever semantic file snapshot exists without verifying that its source revision matches the text currently being lexed. 

These are the targets of this plan.

---

# 2. Required architectural invariants

These invariants are normative. A locally faster implementation that violates one of them is not acceptable.

### INV-1 — One deep-state writer

Only the background analysis worker owns and mutates `SemanticEngine`.

`SemanticDb` is a publication/query object only.

Production code must make it impossible to call:

```rust
db.update_file(...)
db.remove_file(...)
db.update_core(...)
```

from an arbitrary request thread.

Tests may have a dedicated synchronous harness, but that harness must own its own `SemanticEngine`.

---

### INV-2 — One semantic generation per request

Every semantic LSP request pins exactly one:

```rust
Arc<SemanticSnapshot>
```

at request entry.

It must not repeatedly call `SemanticDb::snapshot()` through forwarding methods during the same request.

This applies to:

```text
hover
completion
definition
references
inlay hints
semantic tokens
workspace-backed source resolution used by those requests
```

A request may be based on an older semantic generation than the newest publication, but it must be internally coherent.

---

### INV-3 — Current source owns current source identity

A file-local semantic identity is only valid against the source revision that produced it.

The following are source-revision-local and must never be consumed against a different live document revision:

```text
BindingId
ScopeId
OccurrenceIndex source ranges
Member/class source ranges used against the current document
LocalFacts
current-file parameter/local occurrence ranges
```

When:

```text
published_file.revision != live_document.revision
```

the request must not reinterpret stale byte ranges against current text.

It should fall back to current parse/shallow/index information.

No waiting for the worker is allowed.

---

### INV-4 — Stable global identity may survive revision lag

Module-qualified semantic identities such as:

```rust
ClassId
CallableId
FieldId
```

can be reused across generations when their declaration surface is known to remain compatible.

But a request must first obtain those identities from a source-compatible route.

A stale `BindingId` or stale occurrence range must never be used merely because the resulting `ClassId` or `CallableId` would have been stable.

---

### INV-5 — Semantic identity never loses dispatch side

Once a member is represented by:

```rust
CallableId {
    owner,
    selector,
    side,
}
```

all later semantic lookups use that complete identity.

Do not convert it back to:

```text
(owner, selector)
```

and infer the side again.

The same rule applies to fields:

```rust
FieldId {
    owner,
    name,
    side,
}
```

---

### INV-6 — No filesystem operations on editor query paths

These request paths must perform zero:

```text
read_to_string
canonicalize
metadata
is_file
read_dir
disk parse
```

operations:

```text
hover
completion
definition
references
inlay hints
semantic tokens
workspace/symbol
```

Disk-backed source discovery and canonicalization belong to source ingestion / the background worker.

---

### INV-7 — No synchronous closed-file disk refresh in Tokio LSP handlers

`didChangeWatchedFiles` and `didClose` may enqueue work, update immediate in-memory open-document state, and publish syntax diagnostics.

They must not synchronously read and parse closed files.

---

### INV-8 — Publication is structural sharing, not serialization-by-clone

Publishing a new generation must not deep-clone unchanged:

```text
ASTs
scope graphs
occurrence indexes
class surfaces
callable summaries
field evidence
parameter contributions
module graph
```

Unchanged semantic products must remain shared through `Arc`.

---

### INV-9 — Cancellation does not require a deep copy of the universe

Speculative analysis may use a candidate semantic state, but cloning that candidate must be O(1) or shallow with respect to deep semantic products.

Cancellation must abandon the candidate without rolling partial changes into the published generation.

---

### INV-10 — Body-only edits begin at the changed callable

For a body-only change:

```text
changed source
    ↓
changed callable(s)
    ↓
recompute those callable summaries
```

Do **not** immediately add the module's entire transitive importer closure.

Only a changed externally observable summary/parameter contribution/dependency surface may propagate beyond the changed callable/module.

---

### INV-11 — Parameter propagation is contribution-local

Replacing caller `A`'s contribution to parameter slot `B.x` must touch:

```text
A's previously-contributed slots
∪
A's newly-contributed slots
```

—not every parameter slot in the semantic universe.

Consumers are dirtied only when the joined value of one of their parameter slots actually changes.

---

### INV-12 — Progressive scanning has bounded work units

A scanner turn must be bounded by:

```text
directories started
directory entries consumed
files emitted
```

A single directory containing 100,000 children must not be processed in one uninterrupted step.

---

### INV-13 — Scan/disk results cannot overwrite newer live source

A disk scan beginning while a source is closed cannot overwrite:

```text
index
source cache
semantic source contribution
canonical source identity
```

if the file becomes open or receives a newer disk/source mutation before that scan result commits.

---

### INV-14 — Refresh requests correspond to changed editor products

Deep inference changes should refresh inlay hints when needed.

Semantic-token refresh should only be requested when token classification may have changed.

Do not trigger both refreshes merely because some semantic generation was published.

---

### INV-15 — Advisory inference remains distinct from formal typing

Do not reinterpret `ValueShape`, `InferredValue`, confidence, field inference, or callable summaries as formal Phalcom types.

This phase prepares reusable infrastructure for future typing but does not implement the type system.

---

# 3. Explicit non-goals

Do not expand this work into:

- incremental parsing;
- compiler IR reuse;
- VM execution in LSP analysis;
- a formal static type system;
- a new generic query engine;
- Salsa-style dependency tracking;
- persistent on-disk semantic caches;
- speculative parallel semantic solving;
- distributed analysis;
- semantic-token delta protocol;
- a new persistent-collection dependency;
- whole-repository AST interning;
- a rewrite of `tower-lsp`;
- a full completion product redesign;
- arbitrary refactoring of unrelated Phalcom crates.

Use standard-library `Arc` structural sharing before considering a persistent-map crate.

---

# 4. Target semantic data model

## 4.1 Side-safe member storage

Replace the ambiguous dual-map model with one selector table whose value preserves both dispatch lanes.

Target interface:

```rust
#[derive(Clone, Debug, Default)]
pub struct MemberSides {
    pub instance: Option<MemberSurface>,
    pub class: Option<MemberSurface>,
}

impl MemberSides {
    pub fn get(&self, side: DispatchSide) -> Option<&MemberSurface> {
        match side {
            DispatchSide::Instance => self.instance.as_ref(),
            DispatchSide::Class => self.class.as_ref(),
        }
    }

    pub fn get_mut(&mut self, side: DispatchSide) -> Option<&mut MemberSurface> {
        match side {
            DispatchSide::Instance => self.instance.as_mut(),
            DispatchSide::Class => self.class.as_mut(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClassSurface {
    pub id: ClassId,
    pub superclass: Option<ClassId>,
    pub members: BTreeMap<String, MemberSides>,
    pub fields: BTreeMap<String, FieldSides>,
    pub source_range: SourceRange,
    pub name_range: SourceRange,
}
```

Corresponding field storage:

```rust
#[derive(Clone, Debug, Default)]
pub struct FieldSides {
    pub instance: Option<FieldSurface>,
    pub class: Option<FieldSurface>,
}
```

Required helpers:

```rust
impl ClassSurface {
    pub fn member(
        &self,
        selector: &str,
        side: DispatchSide,
    ) -> Option<&MemberSurface>;

    pub fn member_by_id(
        &self,
        callable: &CallableId,
    ) -> Option<&MemberSurface>;

    pub fn field(
        &self,
        name: &str,
        side: DispatchSide,
    ) -> Option<&FieldSurface>;

    pub fn members_on(
        &self,
        side: DispatchSide,
    ) -> impl Iterator<Item = &MemberSurface>;

    pub fn all_members(
        &self,
    ) -> impl Iterator<Item = &MemberSurface>;
}
```

Do not retain a separate side-blind `BTreeMap<String, MemberSurface>` compatibility table.

---

## 4.2 Canonical field occurrence identity

Replace:

```rust
SemanticTarget::Field {
    owner: ClassId,
    name: String,
}
```

with:

```rust
SemanticTarget::Field(FieldId)
```

The occurrence builder must determine the lexical dispatch/storage side when creating field occurrences.

A class-side field and an instance-side field with the same name must remain distinguishable all the way through:

```text
occurrence
→ hover
→ definition
→ references
→ inferred field fact
```

---

## 4.3 Published semantic tables

Target semantic state:

```rust
#[derive(Clone)]
pub(crate) struct SemanticState {
    pub generation: SemanticGeneration,

    pub files:
        Arc<BTreeMap<ModuleId, Arc<FileSemanticSnapshot>>>,

    pub classes:
        Arc<BTreeMap<ClassId, Arc<ClassSurface>>>,

    pub summaries:
        Arc<BTreeMap<CallableId, Arc<CallableSummary>>>,

    pub field_facts:
        Arc<BTreeMap<FieldId, InferredValue>>,

    pub parameter_facts:
        Arc<BTreeMap<ParameterSlot, InferredValue>>,

    pub parameter_contributions:
        Arc<ParameterContributions>,

    pub callable_dependencies:
        Arc<BTreeMap<CallableId, BTreeSet<CallableId>>>,

    pub callable_dependents:
        Arc<BTreeMap<CallableId, BTreeSet<CallableId>>>,

    pub graph:
        Arc<ModuleGraph>,

    pub uri_aliases:
        Arc<BTreeMap<Url, ModuleId>>,
}
```

`SemanticSnapshot` should structurally share the relevant tables:

```rust
#[derive(Clone, Debug)]
pub struct SemanticSnapshot {
    generation: SemanticGeneration,
    files: Arc<BTreeMap<ModuleId, Arc<FileSemanticSnapshot>>>,
    classes: Arc<BTreeMap<ClassId, Arc<ClassSurface>>>,
    summaries: Arc<BTreeMap<CallableId, Arc<CallableSummary>>>,
    field_facts: Arc<BTreeMap<FieldId, InferredValue>>,
    parameter_facts: Arc<BTreeMap<ParameterSlot, InferredValue>>,
    graph: Arc<ModuleGraph>,
    uri_aliases: Arc<BTreeMap<Url, ModuleId>>,
}
```

`SemanticEngine::snapshot()` then becomes pointer sharing rather than workspace reconstruction.

For mutation use:

```rust
Arc::make_mut(&mut state.files)
Arc::make_mut(&mut state.classes)
Arc::make_mut(&mut state.summaries)
```

only for tables that actually change.

---

## 4.4 Publication ownership

Target production ownership:

```text
Backend
 ├── Arc<SemanticDb>
 │     └── latest Arc<SemanticSnapshot>
 │
 └── AnalysisService
       └── worker thread
             └── SemanticEngine
                   └── mutable SemanticState
```

`SemanticDb` must no longer contain:

```rust
engine: Mutex<SemanticEngine>
```

---

## 4.5 Request context

Create one request-local context:

```rust
pub(crate) struct RequestContext {
    pub uri: Url,
    pub document: DocumentSnapshot,
    pub semantic: Arc<SemanticSnapshot>,
    pub module: Option<ModuleId>,
}

impl RequestContext {
    pub fn exact_file(&self) -> Option<&FileSemanticSnapshot> {
        let module = self.module.as_ref()?;
        let file = self.semantic.file(module)?;

        (file.revision == self.document.revision)
            .then_some(file)
    }

    pub fn is_semantically_current(&self) -> bool {
        self.exact_file().is_some()
    }
}
```

The exact implementation may adjust lifetimes, but the semantics are fixed.

---

## 4.6 Source identity

`ModuleId` construction used by request code becomes pure.

Target:

```rust
impl ModuleId {
    pub fn from_normalized_uri(uri: &Url) -> Self {
        Self(uri.to_string())
    }

    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }
}
```

Filesystem canonicalization belongs to worker/source-ingestion code, not `ModuleId`.

Maintain aliases in the published semantic generation:

```text
LSP URI
canonical physical URI
→ one ModuleId
```

A request resolves:

```rust
snapshot.module_for_uri(&uri)
```

without touching the filesystem.

---

## 4.7 Source delta

Add an explicit source delta distinct from the broad `SourceChangeKind`:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SourceDelta {
    pub kind: SourceChangeKind,

    pub changed_callables: BTreeSet<CallableId>,
    pub added_callables: BTreeSet<CallableId>,
    pub removed_callables: BTreeSet<CallableId>,

    pub top_level_changed: bool,
    pub local_products_changed: bool,
}
```

`SourceChangeKind` answers:

> Which structural semantic layers may have changed?

`SourceDelta` answers:

> Which exact executable units must be recomputed first?

Do not make one enum carry both responsibilities.

---

## 4.8 Contribution-local parameter storage

Target:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterFactDelta {
    pub slot: ParameterSlot,
    pub before: Option<InferredValue>,
    pub after: Option<InferredValue>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParameterContributions {
    by_slot:
        BTreeMap<
            ParameterSlot,
            BTreeMap<ContributionSource, InferredValue>
        >,

    slots_by_source:
        BTreeMap<
            ContributionSource,
            BTreeSet<ParameterSlot>
        >,

    joined:
        BTreeMap<ParameterSlot, InferredValue>,
}
```

Required API:

```rust
impl ParameterContributions {
    pub fn replace_source(
        &mut self,
        source: ContributionSource,
        facts: impl IntoIterator<Item = (ParameterSlot, InferredValue)>,
    ) -> Vec<ParameterFactDelta>;

    pub fn remove_source(
        &mut self,
        source: &ContributionSource,
    ) -> Vec<ParameterFactDelta>;

    pub fn get(
        &self,
        slot: &ParameterSlot,
    ) -> Option<&InferredValue>;

    pub fn joined_iter(
        &self,
    ) -> impl Iterator<Item = (&ParameterSlot, &InferredValue)>;
}
```

`replace_source` must recalculate only touched slots.

---

## 4.9 Parameter contribution provenance

`SurfaceFlowAnalysis` must no longer erase which executable source contributed call-site evidence.

Target representation:

```rust
pub struct SurfaceFlowAnalysis {
    pub local_facts: LocalFacts,
    pub field_facts: FieldFacts,

    pub parameter_contributions:
        BTreeMap<ContributionSource, ParameterFacts>,

    pub summaries: Vec<(CallableSummary, bool)>,
}
```

When analyzing a callable:

```rust
ContributionSource::Callable(callable.clone())
```

When analyzing module-level executable code:

```rust
ContributionSource::TopLevel(module.clone())
```

Do not merge those two sources and try to reconstruct provenance later.

---

## 4.10 Publication effects

Worker publication should carry product-level refresh information:

```rust
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
)]
pub struct PublicationEffects {
    pub inlay_hints_changed: bool,
    pub semantic_tokens_changed: bool,
}
```

Event:

```rust
AnalysisEvent::Published {
    generation: SemanticGeneration,
    effects: PublicationEffects,
}
```

---

# 5. Parallel execution and review protocol

The highest-risk files are shared integration seams. Do not let parallel agents edit them concurrently.

## Wave 0 — baseline only

One worker establishes the exact current behavior and counters.

No code changes.

## Wave 1 — independent leaf refactors

These may run in parallel with strict ownership:

| Worker | Exclusive files |
|---|---|
| Semantic identity | `semantic/surface.rs`, `semantic/dispatch.rs`, `semantic/occurrence.rs` |
| Parameter contributions | `semantic/facts.rs` |
| Scanner fairness | `workspace_scan.rs` |
| Document/query context scaffolding | `documents.rs`, new `request_context.rs` |
| VS Code configuration hygiene | `tools/vsphalcom/package.json`, extension configuration tests |

Do not edit `engine.rs`, `infer.rs`, `analysis_service.rs`, `backend.rs`, `semantic/mod.rs`, or `snapshot.rs` during parallel Wave 1.

## Wave 2 — serialized semantic integration

Controller owns:

```text
semantic/snapshot.rs
semantic/mod.rs
semantic/engine.rs
semantic/infer.rs
semantic/flow.rs
semantic/invalidation.rs
semantic/source.rs
analysis_service.rs
```

Integrate Tasks 14–18 sequentially.

## Wave 3 — serialized backend/source integration

Controller owns:

```text
backend.rs
analysis_service.rs
semantic_tokens.rs
completion.rs
hover.rs
documents.rs
source identity/catalog module
integration test support
```

## Reviews

After each task:

1. fresh spec-compliance review;
2. fix Critical/Important findings;
3. fresh code-quality/Rust review;
4. fix Critical/Important findings;
5. rerun task-local gates;
6. commit.

A worker's own review does not satisfy the review requirement.

---

# Task 0: Establish the new baseline

**Files:**

- Read: current `main`
- Read: attached plan/spec
- Test: existing semantic and integration tests
- No implementation changes

### Interfaces

**Consumes:** current `38e0996` repository state.

**Produces:** written baseline results and exact counter behavior against which later tasks are compared.

- [ ] **Step 1: verify checkpoint**

Run:

```bash
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Expected:

```text
38e0996578259ab47a9a28d95f2f59a0d1c893ac
```

with a clean working tree.

If `main` has advanced, record the new SHA and review every changed `phalcom-lsp` file before implementing this plan. Do not blindly reset to `38e0996`.

- [ ] **Step 2: run focused semantic baseline**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic:: --quiet
```

Record pass/fail.

- [ ] **Step 3: run focused editor baseline**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage3_completion --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage4_hover --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage5_semantic_tokens --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage6_inlay_hints --quiet
```

- [ ] **Step 4: run existing ignored performance harness once**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration perf_ \
  -- --ignored --nocapture
```

Record:

```text
flow_passes
solver_rounds
callables_analyzed
semantic batches
coalesced updates
stale batches
workspace files parsed/discovered
```

Do not use wall-clock numbers as strict future thresholds.

- [ ] **Step 5: create isolated worktrees**

Use `superpowers:using-git-worktrees`.

Do not begin Wave 1 until file ownership is assigned.

---

# Task 14: Make semantic member and field identity side-safe

**Files:**

- Modify: `phalcom-lsp/src/semantic/surface.rs`
- Modify: `phalcom-lsp/src/semantic/dispatch.rs`
- Modify: `phalcom-lsp/src/semantic/occurrence.rs`
- Integrate later: `semantic/snapshot.rs`, `backend.rs`, `completion.rs`, `flow.rs`, `infer.rs`
- Test: `surface.rs`, `dispatch.rs`, semantic tests, Stage 4 hover/definition tests

### Interfaces

**Consumes:** `CallableId`, `FieldId`, `DispatchSide`.

**Produces:**

```rust
MemberSides
FieldSides
ClassSurface::member(...)
ClassSurface::member_by_id(...)
ClassSurface::field(...)
SemanticTarget::Field(FieldId)
```

---

- [ ] **Step 1: write failing same-selector side tests**

Add to `surface.rs`:

```rust
#[test]
fn instance_and_class_members_with_same_selector_remain_distinct() {
    let program = parse(
        r#"
class Widget {
  make() { 1 }

  @class
  make() { 2 }
}
"#,
        0,
    )
    .program;

    let module = ModuleId::new("file:///widget.ph");
    let surface = build_module_surface(module.clone(), &program);
    let class = &surface.classes[&ClassId::new(module, "Widget")];

    let instance = class
        .member("make()", DispatchSide::Instance)
        .expect("instance make");

    let class_side = class
        .member("make()", DispatchSide::Class)
        .expect("class make");

    assert_ne!(instance.callable, class_side.callable);
}
```

The test should initially fail because the new API does not exist.

- [ ] **Step 2: add field-side identity test**

Construct a fixture containing both supported storage lanes if the parser accepts the combination.

If duplicate same-name field declaration is syntactically forbidden, instead test that class-side field occurrences carry:

```rust
FieldId.side == DispatchSide::Class
```

and instance fields carry:

```rust
DispatchSide::Instance
```

The test must prove that field-side information is not discarded.

- [ ] **Step 3: replace `members` + `members_by_side` with `MemberSides`**

Implement the target data model.

Do not leave:

```rust
pub members_by_side: ...
```

as a second authoritative store.

One source of truth only.

- [ ] **Step 4: make dispatch lookup allocation-free by selector**

Current tuple-key lookup creates a fresh selector `String`.

After this change `DispatchResolver::resolve` should effectively perform:

```rust
let member = surface.member(selector, side)?;
```

No:

```rust
selector.to_string()
```

inside each inheritance step.

- [ ] **Step 5: canonicalize field occurrence identity**

Change:

```rust
SemanticTarget::Field { owner, name }
```

to:

```rust
SemanticTarget::Field(FieldId)
```

Ensure declaration and read/write occurrences preserve side.

- [ ] **Step 6: delete side-blind member lookup APIs**

The eventual semantic query API must use:

```rust
member_surface(&CallableId)
```

not:

```rust
member_surface(&ClassId, &str)
```

Do not add a convenience overload that silently chooses one side.

- [ ] **Step 7: add LSP regression**

Add a fixture where one class defines both instance/class `make()` implementations with different PhalDoc text.

Assert:

```text
instance call hover → instance declaration/doc
class call hover    → class declaration/doc
instance definition → instance source range
class definition    → class source range
```

- [ ] **Step 8: focused tests**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic::surface --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic::dispatch --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage4_hover --quiet
```

- [ ] **Step 9: commit**

```bash
git add \
  phalcom-lsp/src/semantic/surface.rs \
  phalcom-lsp/src/semantic/dispatch.rs \
  phalcom-lsp/src/semantic/occurrence.rs \
  phalcom-lsp/tests/stage4_hover.rs

git commit -m \
  "fix(lsp): preserve side-aware semantic member identity"
```

---

# Task 15: Pin request generations and remove query-time filesystem identity

**Files:**

- Modify: `phalcom-lsp/src/semantic/ids.rs`
- Modify: `phalcom-lsp/src/semantic/snapshot.rs`
- Modify: `phalcom-lsp/src/documents.rs`
- Create: `phalcom-lsp/src/request_context.rs`
- Create if necessary: `phalcom-lsp/src/source_catalog.rs`
- Modify: `phalcom-lsp/src/lib.rs` or module root
- Integrate: `backend.rs`, `completion.rs`, `semantic_tokens.rs`, `inlay_hints.rs`
- Test: semantic consistency, stage 3/4/5/6 integration

### Interfaces

**Produces:**

```rust
SemanticSnapshot::module_for_uri(...)
SemanticSnapshot::file(...)
RequestContext
RequestContext::exact_file()
```

and a pure `ModuleId` API.

---

- [ ] **Step 1: add a filesystem-call regression counter**

Under `cfg(test)`, instrument the canonicalization helper—not `std::fs` globally—with an atomic counter.

The existing `ModuleId::from_uri` should increment it initially.

Add a test:

```rust
#[test]
fn semantic_snapshot_queries_do_not_canonicalize_filesystem_paths() {
    reset_test_canonicalization_count();

    // Build/publish semantic fixture first.

    let snapshot = db.snapshot();

    // Query class/member/occurrence/inference.

    assert_eq!(test_canonicalization_count(), 0);
}
```

The test must fail before the refactor.

- [ ] **Step 2: make `ModuleId` pure**

Remove `std::fs::canonicalize` from `semantic/ids.rs`.

Filesystem canonicalization must not be an identity constructor.

- [ ] **Step 3: introduce source-ingestion canonicalization**

Canonicalize disk paths in one worker/source-catalog helper.

For example:

```rust
pub(crate) fn canonical_source_identity(
    uri: &Url,
) -> SourceIdentity;
```

This function may perform filesystem I/O because it is worker-side.

It must retain:

```text
incoming URI
canonical URI
ModuleId
```

as aliases.

- [ ] **Step 4: publish URI aliases**

Add:

```rust
SemanticSnapshot::module_for_uri(&Url) -> Option<&ModuleId>
```

Queries never reconstruct a `ModuleId` from the filesystem.

- [ ] **Step 5: extend `DocumentSnapshot` only with immutable request data**

Do not add a filesystem-derived `ModuleId` in `DocumentStore::snapshot()`.

The `DocumentSnapshot` should remain cheap and independent of disk state.

- [ ] **Step 6: add `RequestContext`**

Implement one helper in the backend layer:

```rust
fn request_context(
    &self,
    uri: &Url,
) -> Option<RequestContext>
```

It must:

1. clone `DocumentSnapshot`;
2. clone `Arc<SemanticSnapshot>`;
3. resolve module through `semantic.module_for_uri(uri)`;
4. release every shared-map guard before semantic processing.

- [ ] **Step 7: revision-gate file-local semantic products**

Only:

```rust
request.exact_file()
```

may supply:

```text
OccurrenceIndex
BindingId
ScopeGraph-backed current-file identity
LocalFacts
current-file source ranges
```

If revisions differ, do not wait.

- [ ] **Step 8: convert hover**

`hover` pins `RequestContext` once.

On semantic revision mismatch:

- use current parse/keyword/selector fallback;
- do not use stale semantic occurrence ranges;
- cross-file declaration metadata from the pinned global snapshot remains available if safely reached through current syntax/index identity.

- [ ] **Step 9: convert definition and references**

Same rule.

In particular, this must be impossible:

```text
new document byte offset
→ stale occurrence_at(offset)
→ old BindingId
→ incorrect new-text definition
```

- [ ] **Step 10: convert inlay hints**

Keep the existing stale-revision rejection behavior, but consume the already-pinned request snapshot rather than independently calling `db.file_snapshot()`.

- [ ] **Step 11: convert semantic tokens**

Change the semantic-token interface from:

```rust
tokens_for(
    db: &SemanticDb,
    uri: &Url,
    text: &str,
    line_index: &LineIndex,
)
```

to a request-snapshot-based form.

For example:

```rust
tokens_for(
    request: &RequestContext,
)
```

Use semantic occurrence overrides only when:

```rust
request.exact_file().is_some()
```

Otherwise use current lexer/current AST declaration refinement only.

- [ ] **Step 12: stop reparsing current text inside semantic-token fallback**

The backend already owns `DocumentSnapshot.parse`.

Pass that parse/program to the token refinement logic instead of:

```rust
phalcom_ast::parser::parse(text, 0)
```

again.

- [ ] **Step 13: convert completion**

Completion may still perform a small request-local recovery parse of the receiver/incomplete expression.

It must not perform filesystem identity lookup and must not call `SemanticDb` convenience methods that acquire new snapshots.

- [ ] **Step 14: add deterministic stale-source tests**

Required cases:

```text
published revision 1:
  class A { first() {} }

live revision 2:
  inserted text before first
```

Assert:

- hover never resolves revision-1 occurrence range against revision-2 text;
- definition never jumps using revision-1 `BindingId`;
- references never use stale local identity;
- semantic tokens do not apply revision-1 semantic classifications to revision-2 positions;
- inlay hints remain absent until compatible publication;
- request returns immediately.

- [ ] **Step 15: focused gates**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration semantic_consistency --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage3_completion --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage4_hover --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage5_semantic_tokens --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage6_inlay_hints --quiet
```

- [ ] **Step 16: commit**

```bash
git add phalcom-lsp/src \
        phalcom-lsp/tests

git commit -m \
  "fix(lsp): pin coherent request snapshots and source revisions"
```

---

# Task 16: Make publication structurally shared and enforce one-writer ownership

**Files:**

- Modify: `semantic/engine.rs`
- Modify: `semantic/snapshot.rs`
- Modify: `semantic/mod.rs`
- Modify: `analysis_service.rs`
- Modify tests using synchronous `SemanticDb` mutation
- Modify: `perf.rs`

### Interfaces

**Produces:**

```text
worker-owned SemanticEngine
query-only SemanticDb
Arc-shared SemanticState tables
O(1) SemanticEngine::snapshot()
```

---

- [ ] **Step 1: write structural-sharing tests before modifying state**

Construct a two-file engine.

Publish snapshot 1.

Edit one file body.

Publish snapshot 2.

Assert pointer identity for unrelated products:

```rust
assert!(Arc::ptr_eq(
    snapshot1.files.get(&unrelated).unwrap(),
    snapshot2.files.get(&unrelated).unwrap(),
));
```

Do the same for one unrelated class and callable summary.

- [ ] **Step 2: add publication-clone counters**

Add test/perf counters for:

```text
semantic_candidate_state_clones
published_file_products_reused
published_class_products_reused
published_summary_products_reused
```

Do not instrument every `Arc::clone`.

The purpose is to prove the architecture, not count reference increments.

- [ ] **Step 3: convert deep semantic products to `Arc` values**

Change `SemanticState` to the target tables described above.

A candidate state clone must clone table `Arc`s, not the contents.

- [ ] **Step 4: use copy-on-write only on changed tables**

For example:

```rust
let files = Arc::make_mut(&mut candidate.files);
files.insert(module, Arc::new(file_snapshot));
```

Body-only updates must not trigger:

```rust
Arc::make_mut(&mut candidate.classes)
```

unless the declaration surface actually changed.

Likewise do not copy the graph for a body-only edit.

- [ ] **Step 5: make `SemanticSnapshot` share table Arcs**

`SemanticEngine::snapshot()` should become approximately:

```rust
pub fn snapshot(&self) -> SemanticSnapshot {
    SemanticSnapshot {
        generation: self.state.generation,
        files: self.state.files.clone(),
        classes: self.state.classes.clone(),
        summaries: self.state.summaries.clone(),
        field_facts: self.state.field_facts.clone(),
        parameter_facts: self.state.parameter_facts.clone(),
        graph: self.state.graph.clone(),
        uri_aliases: self.state.uri_aliases.clone(),
    }
}
```

No iteration over every semantic entry.

- [ ] **Step 6: remove the double candidate clone**

There must be one candidate transaction boundary.

Do not retain both:

```text
SemanticDb clones SemanticEngine
SemanticEngine clones itself again
```

- [ ] **Step 7: move `SemanticEngine` into the worker**

`SemanticDb` becomes:

```rust
pub struct SemanticDb {
    current: RwLock<Arc<SemanticSnapshot>>,
    counters: PerfCountersHandle,
}
```

Worker loop owns:

```rust
let mut engine = SemanticEngine::new_with_counters(...);
```

- [ ] **Step 8: remove production synchronous mutators from `SemanticDb`**

Delete or `#[cfg(test)]`-gate:

```text
update_file
update_files_batch
update_core
remove_file
apply_mutations_with_cancel
```

Preferred solution: migrate tests to a dedicated `SemanticTestHarness`.

Example:

```rust
#[cfg(test)]
pub(crate) struct SemanticTestHarness {
    pub db: Arc<SemanticDb>,
    pub engine: SemanticEngine,
}

#[cfg(test)]
impl SemanticTestHarness {
    pub fn update_file(...) {
        self.engine.update_file(...);
        self.db.publish(Arc::new(self.engine.snapshot()));
    }
}
```

Production architecture must not depend on this harness.

- [ ] **Step 9: remove unnecessary state clones inside rebuild**

Replace patterns such as:

```rust
let classes = state.classes.clone();
let graph = state.graph.clone();
```

when they exist solely to obtain immutable read views.

Borrow:

```rust
state.classes.as_ref()
state.graph.as_ref()
```

instead.

- [ ] **Step 10: ensure solver receives borrowed immutable base state**

Do not clone every class merely to construct `DispatchResolver`.

- [ ] **Step 11: structural acceptance**

For a body-only edit of module A in workspace `{A, B, C}`:

```text
unrelated B FileSemanticSnapshot Arc reused
unrelated C FileSemanticSnapshot Arc reused

unchanged class surfaces reused
unchanged callable summaries reused
module graph Arc reused
```

- [ ] **Step 12: focused gates**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic:: --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration workspace_semantics --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration semantic_consistency --quiet
```

- [ ] **Step 13: commit**

```bash
git add phalcom-lsp/src/semantic \
        phalcom-lsp/src/analysis_service.rs \
        phalcom-lsp/src/perf.rs \
        phalcom-lsp/tests

git commit -m \
  "perf(lsp): structurally share published semantic generations"
```

---

# Task 17: Seed body-only invalidation from exact changed callables

**Files:**

- Modify: `semantic/invalidation.rs`
- Modify: `semantic/surface.rs`
- Modify: `semantic/snapshot.rs`
- Modify: `semantic/engine.rs`
- Modify: `semantic/infer.rs`
- Modify: `semantic/flow.rs`
- Test: semantic invalidation/workspace tests

### Interfaces

**Produces:**

```rust
SourceDelta
classify_source_delta(...)
dirty callable seed
```

---

- [ ] **Step 1: replace string-based declaration fingerprinting**

Do not keep declaration semantic equality as a sorted `Vec<String>` built through `format!("{:?}")`.

Introduce typed comparable fingerprint structures.

For example:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
struct MemberDeclarationFingerprint {
    selector: String,
    side: DispatchSide,
    kind: MemberKind,
    visibility: MemberVisibility,
    constructor: bool,
    params: Vec<ParameterDeclarationFingerprint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParameterDeclarationFingerprint {
    label: Option<String>,
    name: String,
}
```

Only include fields that affect semantic surface.

Source ranges must not participate.

- [ ] **Step 2: retain source text in source snapshots**

Exact callable-body change detection requires comparing the source contribution from old/new snapshots.

Extend the source input/snapshot to retain:

```rust
Arc<str>
```

The text should already exist from open-document/disk ingestion.

Do not reread it.

- [ ] **Step 3: create direct callable source index**

Avoid scanning every class/member to locate one `CallableId`.

Add to `ModuleSurface` or `FileSourceSnapshot`:

```rust
callables:
    BTreeMap<CallableId, MemberAstRef>
```

and/or direct member-surface access.

The index must be built once with the source surface.

- [ ] **Step 4: write exact body-delta tests**

Fixture:

```phalcom
class A {
  untouched() { 1 }

  changed() { 2 }
}
```

Edit only:

```phalcom
changed() { 3 }
```

Assert:

```text
SourceChangeKind::BodyOnly
changed_callables == { A.changed() }
untouched() absent
```

Also test a source-range shift:

```text
insert blank/comment/text before untouched()
```

with identical `untouched()` source content.

Its callable must not be considered semantically changed merely because its byte range moved.

- [ ] **Step 5: compare callable content by source slice, not source range identity**

For callable existing in both surfaces:

```text
old text[old member declaration range]
new text[new member declaration range]
```

may be compared directly.

This phase accepts whitespace inside a callable as a callable body change.

Do not implement an AST-normalizing formatter/hash merely to ignore whitespace.

- [ ] **Step 6: detect top-level executable change independently**

A body edit outside a class callable must set:

```rust
top_level_changed = true;
```

Do not treat it as a changed method.

- [ ] **Step 7: core body-only edits become body-only**

Current core changes are always classified as broad `CoreSurface`.

Change the rule:

```text
core declaration/native surface changed → CoreSurface
core callable body only changed          → BodyOnly
```

A core body edit may propagate through callable dependencies, but it must not automatically rebuild every class/module.

- [ ] **Step 8: stop adding import dependents before summary comparison**

Delete the unconditional body-edit path that effectively does:

```rust
affected.extend(
    graph.dependent_closure(&module)
);
```

before callable output is known.

For `BodyOnly`:

```text
affected module for current-file local product refresh
dirty callables = SourceDelta.changed_callables
```

No importer closure yet.

- [ ] **Step 9: change incremental solver entry point**

Normal solver entry should accept explicit dirty callables:

```rust
solve_dirty_callables_with_cancel(
    ...,
    dirty_callables,
    ...
)
```

Do not initialize the worklist with every callable in every affected input module.

- [ ] **Step 10: propagate only on changed callable summary**

When callable A finishes:

```rust
if old_summary != new_summary {
    enqueue reverse callable dependents of A
}
```

If equal:

```text
stop that propagation branch
```

- [ ] **Step 11: declaration/import changes remain conservatively module-aware**

For:

```text
ImportSurface
DeclarationSurface
CoreSurface
FileAddedRemoved
```

module reverse dependencies may still establish the initial frontier.

Do not prematurely force callable-level precision where a declaration can alter dispatch/name resolution globally.

- [ ] **Step 12: improve test trace semantics**

Current test tracing should distinguish:

```rust
callables_visited
callables_changed
modules_source_refreshed
```

Do not call a callable “recomputed” only because its summary differed if the performance test is supposed to prove which bodies were visited.

- [ ] **Step 13: key acceptance test**

With:

```text
A.ph
  f() { 1 }
  g() { 100 }

B.ph
  imports A
  h() { A.f() }
```

change:

```text
A.f(): 1 → 1 + 0
```

if the resulting abstract summary is unchanged.

Required:

```text
solver visits A.f()
solver does not visit A.g()
solver does not visit B.h()
```

One allowed final source-local pass over `A.ph` does not count as interprocedural callable re-solving.

- [ ] **Step 14: changed-summary test**

Change:

```text
A.f(): Int → String
```

Required:

```text
A.f visited
B.h visited if dependency exists
A.g not visited
unrelated modules untouched
```

- [ ] **Step 15: focused gates**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic::invalidation --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic:: --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration workspace_semantics --quiet
```

- [ ] **Step 16: commit**

```bash
git add phalcom-lsp/src/semantic \
        phalcom-lsp/tests/workspace_semantics.rs

git commit -m \
  "perf(lsp): seed semantic invalidation from changed callables"
```

---

# Task 18: Make parameter propagation truly contribution-local

**Files:**

- Modify: `semantic/facts.rs`
- Modify: `semantic/flow.rs`
- Modify: `semantic/infer.rs`
- Modify: `semantic/engine.rs`
- Modify: `semantic/callable.rs`
- Test: semantic parameter/convergence fixtures

### Interfaces

**Consumes:** `SourceDelta` and exact callable worklist from Task 17.

**Produces:**

```rust
ParameterFactDelta
source-indexed ParameterContributions
parameter_contributions in SurfaceFlowAnalysis
```

---

- [ ] **Step 1: write `replace_source` complexity-behavior test**

Populate 1,000 unrelated slots from unrelated contribution sources.

Replace one source touching two slots.

Add a `cfg(test)` touched-slot counter.

Assert:

```text
touched slots <= old source slots ∪ new source slots
```

and not 1,000.

- [ ] **Step 2: add inverse source index**

Implement:

```rust
slots_by_source:
    BTreeMap<
        ContributionSource,
        BTreeSet<ParameterSlot>
    >
```

- [ ] **Step 3: add cached joined values**

Maintain:

```rust
joined:
    BTreeMap<ParameterSlot, InferredValue>
```

`replace_source` recomputes only touched slots.

- [ ] **Step 4: return explicit deltas**

Implement:

```rust
Vec<ParameterFactDelta>
```

where unchanged joined values are omitted.

- [ ] **Step 5: preserve caller provenance in flow**

Change `SurfaceFlowAnalysis` to retain contributions by:

```text
Callable(callable)
TopLevel(module)
```

Do not flatten all source call sites into one anonymous `ParameterFacts`.

- [ ] **Step 6: make solver replace only the analyzed caller**

After analyzing callable `A`:

```rust
let deltas =
    contributions.replace_source(
        ContributionSource::Callable(A),
        analysis contributions from A,
    );
```

Do not reconstruct:

```rust
parameter_facts = base_parameters.clone();

for facts in every_source_fact {
    parameter_facts.merge_from(facts);
}
```

- [ ] **Step 7: dirty only changed target slots**

For each `ParameterFactDelta`:

```rust
if before != after {
    worklist.push(delta.slot.callable.clone());
}
```

No other callable is dirtied merely because another parameter somewhere changed.

- [ ] **Step 8: caller removal removes its evidence locally**

Deleting/changing one caller must remove only that caller's contributions.

Regression:

```text
A contributes Int to B.x
C contributes String to B.x

remove A
```

Expected:

```text
B.x becomes String
C contribution retained
```

- [ ] **Step 9: preserve widening semantics**

If bounded solver budget is exhausted, summaries and parameter facts must widen coherently.

Do not publish:

```text
new widened summary
old precise parameter fact
```

or the reverse.

- [ ] **Step 10: reconsider solver budget using actual frontier**

Do not base the only practical bound on:

```rust
callable_count * callable_count
```

for all possible dependency edges when the actual dirty graph is much smaller.

Use an explicit conservative bound based on:

```text
dirty/visited callables
affected parameter slots
actual dependency edges
finite ValueShape union bound
```

Keep it deterministic.

- [ ] **Step 11: convergence regressions**

Retain and run:

```text
three-step forwarding
recursive SCC with concrete evidence
shape widening > MAX_SHAPE_UNION
caller edit removes stale contribution
cross-module parameter propagation
```

- [ ] **Step 12: focused gates**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic:: --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration workspace_semantics --quiet
```

- [ ] **Step 13: commit**

```bash
git add \
  phalcom-lsp/src/semantic/facts.rs \
  phalcom-lsp/src/semantic/flow.rs \
  phalcom-lsp/src/semantic/infer.rs \
  phalcom-lsp/src/semantic/engine.rs \
  phalcom-lsp/src/semantic/callable.rs

git commit -m \
  "perf(lsp): propagate parameter facts by source contribution"
```

---

# Task 19: Move disk refresh fully to the worker and make scanning preemptible

**Files:**

- Modify: `analysis_service.rs`
- Modify: `workspace_scan.rs`
- Modify: `backend.rs`
- Modify source identity/catalog module
- Modify test support
- Test: backend watched-file/workspace-folder/performance tests

### Interfaces

**Produces:**

```rust
MutationBatch
DiskRefresh
per-URI SourceEpoch
bounded directory cursor
```

---

- [ ] **Step 1: introduce one mutation-batch entry point**

Target:

```rust
pub(crate) struct MutationBatch {
    pub open_updates: Vec<OpenSourceUpdate>,
    pub disk_refreshes: Vec<DiskRefresh>,
    pub removals: Vec<Url>,
    pub core_refresh: Option<CoreRefresh>,
}
```

All convenience enqueue methods must delegate to one scheduler mutation operation that increments the semantic epoch once per logical batch.

- [ ] **Step 2: fix watched-file batching**

One `didChangeWatchedFiles` notification must generate one worker batch.

Do not call:

```text
enqueue_file_removals(...)
then
enqueue_file_updates(...)
```

as two epochs.

- [ ] **Step 3: remove `read_to_string` and parse from `didChangeWatchedFiles`**

Handler should convert file events to:

```text
DiskRefresh
Remove
```

and return.

Worker performs:

```text
read
parse
line-index construction
canonical identity
index update
cache update
semantic mutation
```

- [ ] **Step 4: remove disk read/parse from `didClose`**

`didClose`:

1. removes live `Document`;
2. marks URI closed;
3. clears live diagnostics;
4. enqueues disk refresh if the file should persist;
5. does not read it synchronously.

- [ ] **Step 5: remove duplicate removal enqueue paths**

`remove_indexed_file` must not enqueue a removal and then have its caller enqueue the same removal again.

Choose one ownership point.

Add a counter assertion proving one logical delete generates one pending removal.

- [ ] **Step 6: move core filesystem probing off `initialize`**

`initialize` currently schedules the workspace and may probe configured/conventional core paths.

Move physical core selection to the background worker/source-discovery side.

Use `CoreSource::select` or one consolidated implementation.

Delete duplicate backend core-selection logic where possible.

`initialize` should only capture configuration/roots and enqueue discovery.

- [ ] **Step 7: add per-URI source epoch**

A global semantic epoch is too coarse for stale scan results.

Maintain:

```rust
SourceEpoch(u64)
```

per URI.

Increment when:

```text
didOpen
didChange
didClose transition
watched file change
delete
workspace-root removal
explicit source refresh
```

- [ ] **Step 8: guard scan result commit**

Before disk read:

```rust
let ticket = source_epoch(uri);
```

After read/parse and before any cache/index/semantic commit:

```rust
if source_epoch(uri) != ticket
    || is_open(uri)
{
    discard_result;
}
```

Do not let a stale background disk parse overwrite a newly-opened buffer.

- [ ] **Step 9: test stale scan/open race deterministically**

Use a test gate:

```text
scanner begins reading closed file
scanner blocks before commit
didOpen installs newer live contents
release scanner
```

Assert disk scan result does not overwrite:

```text
WorkspaceIndex
closed-source cache
semantic file revision
```

- [ ] **Step 10: make directory iteration resumable**

Current scanner consumes an entire `ReadDir`.

Replace it with persistent cursor state.

Extend budget:

```rust
pub struct ScanBudget {
    pub max_dirs_started: usize,
    pub max_entries: usize,
    pub max_files: usize,
}
```

Store the currently-open directory iterator across `step()` calls.

- [ ] **Step 11: wide-directory regression**

Create one temp directory containing several thousand entries.

Use:

```rust
ScanBudget {
    max_dirs_started: 1,
    max_entries: 16,
    max_files: 8,
}
```

Assert one `step()` does not consume the whole directory.

- [ ] **Step 12: check interactive work between scanned files**

Worker should re-check:

```text
shutdown
interactive pending work
source ticket freshness
```

between each disk file parse.

Do not parse a 32-file scan batch atomically before noticing an open edit.

- [ ] **Step 13: fix exclusion wording or behavior**

Current setting is described as glob-style while implementation is path-fragment/pseudo-glob matching.

For this performance phase, prefer documenting exact supported semantics rather than introducing a glob dependency.

Change configuration description to reflect reality unless a real glob grammar already exists elsewhere.

- [ ] **Step 14: focused gates**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib workspace_scan --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib analysis_service --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration workspace_semantics --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration performance -- --ignored --nocapture
```

If the final command's filter differs because performance is a module of `integration`, use:

```bash
cargo test -p phalcom-lsp --test integration perf_ -- --ignored --nocapture
```

- [ ] **Step 15: commit**

```bash
git add \
  phalcom-lsp/src/analysis_service.rs \
  phalcom-lsp/src/workspace_scan.rs \
  phalcom-lsp/src/backend.rs \
  phalcom-lsp/tests

git commit -m \
  "perf(lsp): isolate disk refresh and bound workspace scanning"
```

---

# Task 20: Make hot semantic queries zero-copy and reuse retained source products

**Files:**

- Modify: `semantic/snapshot.rs`
- Modify: `semantic/mod.rs`
- Modify: `semantic/dispatch.rs`
- Modify: `analysis_service.rs`
- Modify: `backend.rs`
- Modify: `semantic_tokens.rs`
- Modify: `completion.rs`
- Modify: `scope.rs`
- Test: Stage 2–6 and semantic tests

### Interfaces

**Produces:** reference-based immutable query APIs.

---

- [ ] **Step 1: write map-materialization regression counters**

Add `cfg(test)` counters around any compatibility helper that materializes a whole class/summary map.

Test one hover/completion/inference query.

Required:

```text
class map materializations   = 0
summary map materializations = 0
```

- [ ] **Step 2: make dispatch resolver consume published Arc tables directly**

Either:

```rust
DispatchResolver<'a> {
    classes:
        &'a BTreeMap<ClassId, Arc<ClassSurface>>,
}
```

or a tiny lookup trait.

Do not clone class surfaces to construct the resolver.

- [ ] **Step 3: make return resolution operate on Arc summary tables**

`return_for_callable` should borrow:

```rust
&BTreeMap<CallableId, Arc<CallableSummary>>
```

No complete summary map clone.

- [ ] **Step 4: make snapshot query methods return references where possible**

Examples:

```rust
pub fn file(
    &self,
    module: &ModuleId,
) -> Option<&FileSemanticSnapshot>;

pub fn class_surface(
    &self,
    id: &ClassId,
) -> Option<&ClassSurface>;

pub fn member_surface(
    &self,
    id: &CallableId,
) -> Option<&MemberSurface>;

pub fn callable_summary(
    &self,
    id: &CallableId,
) -> Option<&CallableSummary>;
```

Return owned values only where the caller genuinely needs ownership.

- [ ] **Step 5: shrink `SemanticDb` API**

After requests pin a snapshot, `SemanticDb` should mostly expose:

```rust
snapshot()
publish()
perf_counters()
```

Do not preserve dozens of forwarding methods that accidentally acquire a new generation per call.

- [ ] **Step 6: use `DocumentStore::snapshot()` for substantive handlers**

Convert:

```text
completion
hover
inlay_hint
semantic_tokens
definition
references
```

away from long-running `with_document` closures.

A `DashMap` guard must not be held while:

```text
parsing recovery expressions
semantic inference
member hierarchy traversal
token lexing
PhalDoc harvesting
```

- [ ] **Step 7: stop reparsing scanned text in `WorkspaceFileIndexed` event handling**

The worker already parsed the file.

Either:

- place the complete `CachedSource` in the cache before sending the event and make the event carry only URI/revision; or
- pass the retained `Arc<Program>` in the event.

Do not execute:

```rust
phalcom_ast::parser::parse(&text, 0)
```

in the event task as a fallback for an already-indexed source.

- [ ] **Step 8: remove `cached_definition_info` workspace reconstruction fallback**

Do not answer one hover by:

```text
for every cached source:
    build_module_surface(...)
```

The `WorkspaceIndex` is worker-updated before the indexing event is delivered.

Use it as the shallow declaration authority.

- [ ] **Step 9: keep closed-file PhalDoc cache-only**

Harvesting from cached:

```text
Arc<str>
Arc<Program>
Arc<LineIndex>
```

is acceptable.

Disk reads and whole-workspace surface reconstruction are not.

If repeated PhalDoc parsing appears material after counters, add a small precomputed per-source PhalDoc declaration index. Do not add it before measurement.

- [ ] **Step 10: add direct callable lookup**

`flow::analyze_callable` should not locate one callable by scanning:

```text
every class
    every member
```

Use the callable index created in Task 17.

- [ ] **Step 11: fix `visible_bindings_at` ordering contract**

The method says nearest scope first.

Do not accumulate output in a name-sorted `BTreeMap`.

Use:

```rust
let mut seen = BTreeSet::new();
let mut result = Vec::new();
```

walking inner scope outward.

- [ ] **Step 12: keep workspace reference indexing out unless measured**

`references_for_target` remains workspace-linear for non-binding targets.

Do **not** add a complex reverse workspace occurrence index in this task unless the new performance harness shows reference lookup to be material after eliminating the larger global clones.

This is the explicit YAGNI boundary for this phase.

- [ ] **Step 13: focused query gates**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage2_index --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage3_completion --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage4_hover --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage5_semantic_tokens --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage6_inlay_hints --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic:: --quiet
```

- [ ] **Step 14: commit**

```bash
git add phalcom-lsp/src \
        phalcom-lsp/tests

git commit -m \
  "perf(lsp): make immutable semantic queries zero-copy"
```

---

# Task 21: Make publication refreshes selective and remove configuration drift

**Files:**

- Modify: `analysis_service.rs`
- Modify: `backend.rs`
- Modify: `perf.rs`
- Modify: `tools/vsphalcom/package.json`
- Modify: `tools/vsphalcom/src/extension.ts` only if configuration transport changes
- Modify tests

### Interfaces

**Produces:**

```rust
PublicationEffects
```

---

- [ ] **Step 1: add publication-effect unit tests**

Test:

```text
inferred local/return shape changed
→ inlay_hints_changed = true
→ semantic_tokens_changed = false
```

Test:

```text
new exact occurrence/source classification published
→ semantic_tokens_changed = true
```

- [ ] **Step 2: emit product-specific worker event**

Change:

```rust
AnalysisEvent::Published {
    generation,
}
```

to:

```rust
AnalysisEvent::Published {
    generation,
    effects,
}
```

- [ ] **Step 3: retain refresh coalescing separately**

One in-flight inlay refresh should coalesce more inlay requests.

Semantic-token refreshes should coalesce independently.

Do not make a pending inlay refresh force an unnecessary semantic-token refresh.

- [ ] **Step 4: remove dead `phalcom.completion.unknownReceiver` setting**

Current repository exposes this configuration but does not implement the server-side behavior.

This optimization phase must not invent a new completion product policy.

Remove the setting from `package.json` unless a current accepted Phalcom LSP specification—not old patchwork design prose—already defines its exact semantics.

If such an accepted specification exists at implementation time, implement that specification instead and add end-to-end tests.

Do not leave a UI setting that does nothing.

- [ ] **Step 5: reconcile server/extension inlay defaults**

The extension declares `suppressObvious=true`, while `ServerConfig::default()` currently uses `false`.

Choose one canonical default and apply it for every LSP client.

Given the current extension contribution, use:

```rust
suppress_obvious: true
```

unless an accepted product specification states otherwise.

- [ ] **Step 6: preserve existing restart lifecycle**

Do not rewrite `createLspClientLifecycle`.

Task 12's serialized restart + failed-stop disposal behavior is already the desired architecture.

Only modify lifecycle code if a new failing regression demonstrates a real issue.

- [ ] **Step 7: focused Rust tests**

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib backend --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage6_inlay_hints --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage5_semantic_tokens --quiet
```

- [ ] **Step 8: focused extension tests**

From `tools/vsphalcom`:

```bash
npm run lint
npm run compile
npm test
```

- [ ] **Step 9: commit**

```bash
git add \
  phalcom-lsp/src \
  tools/vsphalcom/package.json \
  tools/vsphalcom/src

git commit -m \
  "perf(lsp): refresh only changed editor products"
```

---

# Task 22: Add structural acceptance instrumentation and final verification

**Files:**

- Modify: `perf.rs`
- Modify: `analysis_service.rs`
- Modify test support
- Modify/create tests in `phalcom-lsp/tests/performance.rs`
- Modify `integration.rs` only if a new test module is added
- Update plan/docs after verified behavior

---

## Required new counters

Add counters sufficient to prove—not infer—the new architecture:

```text
candidate_state_clones

published_files_reused
published_classes_reused
published_summaries_reused

query_class_table_materializations
query_summary_table_materializations

query_filesystem_canonicalizations

dirty_callables_seeded
solver_callables_visited
solver_callables_changed

parameter_sources_replaced
parameter_slots_touched
parameter_slots_changed

scan_directory_entries_consumed
scan_results_discarded_as_stale
scan_results_discarded_for_open_document

inlay_refresh_requests
semantic_token_refresh_requests
```

Counters may be test-only where production telemetry has no value.

---

## Required deterministic acceptance tests

### A. Request generation pinning

- [ ] Start request with semantic generation `N`.
- [ ] Publish generation `N+1` during the request.
- [ ] Assert all semantic objects observed by request belong to `N`.

Do not use timing to prove this. Use a test gate.

---

### B. Stale current-file occurrence rejection

- [ ] Publish file revision 1.
- [ ] Install live revision 2.
- [ ] Block semantic worker before revision 2 publication.
- [ ] Query hover/definition/references/tokens.
- [ ] Assert revision-1 occurrence ranges are never used against revision-2 text.

---

### C. Side-aware collision

- [ ] Same selector, instance + class side.
- [ ] Distinct hover.
- [ ] Distinct definition.
- [ ] Distinct PhalDoc.
- [ ] Distinct callable return summary lookup.

---

### D. Structural publication sharing

- [ ] Update module A.
- [ ] Assert unrelated file/class/summary `Arc`s survive publication via `Arc::ptr_eq`.
- [ ] Assert graph `Arc` survives body-only update.

---

### E. Exact callable body frontier

- [ ] Change one callable with unchanged abstract summary.
- [ ] Solver visits exactly that dirty callable.
- [ ] No importer callable visited.
- [ ] No sibling callable visited by the interprocedural worklist.

---

### F. Summary-change propagation

- [ ] Change return shape.
- [ ] Visit exact reverse callable dependents.
- [ ] Do not visit unrelated callables.

---

### G. Contribution-local parameter update

- [ ] Populate large unrelated contribution universe.
- [ ] Replace one caller source touching two slots.
- [ ] Assert unrelated slots are not visited/rejoined.

---

### H. Stale scan result rejection

- [ ] Begin disk scan.
- [ ] Open file with newer buffer before scan commit.
- [ ] Release scanner.
- [ ] Assert stale disk source never enters published cache/index/semantics.

---

### I. Wide-directory scanner fairness

- [ ] Thousands of entries in one directory.
- [ ] One scanner turn consumes no more than configured entry budget.

---

### J. No query filesystem activity

Run:

```text
hover
completion
definition
references
inlay
semantic tokens
workspace symbol
```

against a prepared workspace.

Assert:

```text
filesystem canonicalization query counter == 0
disk read query counter == 0
```

---

### K. No query global-table materialization

Assert:

```text
query_class_table_materializations   == 0
query_summary_table_materializations == 0
```

after representative hover/completion/inference.

---

### L. Selective refresh

Deep shape-only publication:

```text
inlay refresh      = yes
semantic refresh   = no
```

Occurrence/source-semantic publication:

```text
semantic refresh = yes
```

---

# 6. Final acceptance matrix

| Requirement | Acceptance evidence |
|---|---|
| Worker remains only deep writer | `SemanticDb` has no production engine/mutator API |
| One snapshot per request | deterministic publication-mid-request test |
| Stale file-local identity rejected | hover/definition/reference/token revision mismatch tests |
| Side-aware member identity preserved | same-selector class/instance regression |
| Side-aware field identity preserved | `SemanticTarget::Field(FieldId)` tests |
| Query path performs no canonicalization | query filesystem counter = 0 |
| Notification handlers perform no disk refresh parsing | watched/close test hooks |
| Snapshot publication reuses deep products | `Arc::ptr_eq` structural tests |
| Cancellation no longer deep-clones semantic universe | candidate clone counters/Arc state |
| Body edit starts from exact callable | dirty-callable test |
| Unchanged summary stops propagation | reverse dependent not visited |
| Changed summary reaches true dependents | reverse-edge test |
| Parameter updates touch only source slots | touched-slot counter |
| Stale caller contribution removal is exact | multi-caller parameter fixture |
| Scanner yields inside wide directory | max-entry test |
| Stale scan cannot override open buffer | scan/open race test |
| Query map materialization eliminated | table-materialization counters = 0 |
| Semantic token revisions are exact | stale semantic-token regression |
| DashMap guards not held during semantics | request-context API/code review + concurrency test |
| Refreshes are product-specific | PublicationEffects tests |
| Existing async responsiveness remains | blocked-worker hover/inlay tests |
| Existing VS Code restart remains resilient | extension lifecycle tests |
| Formal type-system boundary preserved | no changes making `ValueShape` a language type |

---

# 7. Verification order

Run narrow tests first. Do not jump directly to `cargo test --workspace`.

## Phase 1 — Rust format/lints relevant to changed code

```bash
cargo fmt --all -- --check
```

If project Clippy is currently clean and part of normal repository gates:

```bash
CARGO_TARGET_DIR=target \
cargo clippy -p phalcom-lsp --all-targets -- -D warnings
```

If existing unrelated Clippy baseline failures exist, record them separately.

---

## Phase 2 — semantic unit suite

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib semantic:: --quiet
```

---

## Phase 3 — analysis/backend units

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib analysis_service --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib workspace_scan --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --lib backend --quiet
```

Use actual test-path filters if Rust's generated names differ.

---

## Phase 4 — editor integration slices

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage2_index --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage3_completion --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage4_hover --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage5_semantic_tokens --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration stage6_inlay_hints --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration semantic_consistency --quiet

CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp --test integration workspace_semantics --quiet
```

The repository currently aggregates these modules through `phalcom-lsp/tests/integration.rs`. 

---

## Phase 5 — full LSP crate

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp
```

---

## Phase 6 — dependency crates touched by shared surfaces

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-ast

CARGO_TARGET_DIR=target \
cargo test -p phalcom-native-surface
```

---

## Phase 7 — workspace

```bash
CARGO_TARGET_DIR=target \
cargo test --workspace
```

---

## Phase 8 — VS Code extension

```bash
cd tools/vsphalcom
npm ci
npm run lint
npm run compile
npm test
npm run test:lsp:e2e
```

---

## Phase 9 — documentation

```bash
cargo doc --workspace --no-deps
```

New rustdoc warnings in changed LSP APIs are defects.

---

## Phase 10 — performance harness

Debug:

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp \
  --test integration perf_ \
  -- --ignored --nocapture
```

Release:

```bash
CARGO_TARGET_DIR=target \
cargo test -p phalcom-lsp \
  --release \
  --test integration perf_ \
  -- --ignored --nocapture
```

Record:

```text
commit SHA
build profile
machine
workspace fixture size
wall-clock observations
counter snapshots
```

Do not hard-code historical machine-specific timings as CI thresholds.

The structural counters are the CI contract.

---

# 8. Required performance interpretation

The final implementation is not accepted merely because editor handlers remain asynchronous.

The first async phase established:

```text
editor latency
    ≠
semantic convergence latency
```

This phase must establish the stronger property:

```text
semantic update cost
    ≈
changed semantic frontier
```

For the normal body-edit path the expected architecture is:

```text
live full-document parse
        │
        ▼
worker source snapshot
        │
        ▼
SourceDelta
        │
        ├── current source-local refresh
        │
        └── exact dirty callable(s)
                 │
                 ▼
          callable worklist
                 │
                 ├── summary unchanged
                 │       └── stop
                 │
                 └── summary changed
                         │
                         ▼
                exact reverse dependents
```

Parameter propagation becomes:

```text
caller A analyzed
      │
      ▼
replace ContributionSource::Callable(A)
      │
      ▼
rejoin only A's old/new slots
      │
      ├── joined value unchanged
      │       └── stop
      │
      └── joined value changed
              │
              ▼
       enqueue target callable
```

Publication becomes:

```text
candidate SemanticState
      │
      ├── unchanged tables: shared Arc
      ├── changed tables: Arc::make_mut once
      └── changed products: new Arc
      │
      ▼
SemanticSnapshot
      │
      └── O(1) top-level Arc sharing
```

A request becomes:

```text
DocumentSnapshot
       +
Arc<SemanticSnapshot>
       │
       ▼
 RequestContext
       │
       ├── exact file revision
       │       → semantic occurrence/local facts allowed
       │
       └── stale/missing file revision
               → current syntax/shallow fallback
               → never wait
```

That is the target system.

---

# 9. Rust code-quality requirements

The implementation must also remove the smells that enabled the current global work.

Do not retain broad helper APIs whose convenience hides allocation. In particular, a function such as:

```rust
fn class_surface(...) -> Option<ClassSurface>
```

should not deep-clone merely to make borrowing easier.

Do not use `Clone` as a transaction abstraction for a deeply-owned semantic database.

Do not use `Debug` string formatting as semantic equality.

Do not hide request-time filesystem operations inside innocent identity constructors.

Do not maintain two authoritative representations of the same member set.

Do not broaden `#![allow(clippy::too_many_arguments)]` to cover new APIs. If the final solver still needs many repeated parameters, introduce a focused immutable solver context/world object.

A suitable shape is:

```rust
struct SolverWorld<'a> {
    classes: &'a ClassTable,
    graph: &'a ModuleGraph,
    summaries: &'a SummaryTable,
    parameter_facts: &'a ParameterFactTable,
}
```

or an equivalent design.

Do not create a generic “everything context” that owns unrelated backend state.

---

# 10. Review questions every task reviewer must answer

Each spec-compliance reviewer should explicitly answer these questions for the changed slice:

1. Can this change make a request wait for semantic convergence?
2. Can this change use a source range from a different document revision?
3. Does any resolved `CallableId` or `FieldId` lose its dispatch side?
4. Does the request perform filesystem I/O?
5. Does one leaf edit clone or traverse unrelated semantic products?
6. Does one callable edit dirty unrelated callables before its summary changes?
7. Does one parameter contribution cause global parameter recomputation?
8. Can a stale scan/disk result overwrite a newer open buffer?
9. Are unpublished partial semantic facts externally observable?
10. Does the change accidentally turn advisory `ValueShape` inference into language typing?
11. Does it add an unnecessary new dependency?
12. Is the behavior proven structurally by a deterministic test rather than inferred from timing?

The code-quality reviewer additionally checks:

```text
ownership
Arc boundaries
allocation
borrow lifetime
map lookup complexity
duplicate representations
naming
error handling
poisoned-lock strategy consistency
documentation
test determinism
```

---

# 11. Final completion criteria

This optimization phase is complete only when all of the following are true:

- `SemanticDb` no longer owns mutable `SemanticEngine`.
- `SemanticEngine::snapshot()` does not iterate through the workspace to clone deep semantic products.
- a request pins one `Arc<SemanticSnapshot>`;
- current-file occurrence/binding semantics require exact file revision compatibility;
- `ModuleId` query construction performs no filesystem canonicalization;
- side-blind `ClassSurface.members` ambiguity is gone;
- canonical field targets contain `FieldId`;
- `receiver_member`, `return_for_callable`, `class_for_name`, and `infer_expression` do not materialize whole copied semantic maps;
- body-only edits do not seed importer closure before summary change;
- callable worklist begins with exact changed callables;
- parameter contribution replacement is indexed by source;
- parameter propagation happens only for joined-slot deltas;
- watched-file and close handlers do not synchronously read/parse files;
- one logical watched mutation batch corresponds to one scheduling epoch;
- wide-directory scans yield inside the directory;
- scan results are protected against open-document/source races;
- worker indexing does not reparse source again in the Tokio event consumer;
- semantic tokens reject stale occurrence ranges;
- semantic/inlay refresh requests are separately coalesced and product-specific;
- existing restart resilience remains intact;
- focused tests pass;
- full `phalcom-lsp` tests pass;
- workspace tests are either green or any unrelated baseline failures are explicitly documented;
- VS Code lint/compile/tests/E2E pass;
- rustdoc has no new warnings;
- performance counters demonstrate the structural invariants above.

The important endpoint is not merely a faster version of the current code. It is a cleaner semantic architecture in which **source identity, semantic identity, dependency propagation, publication, and editor querying all have explicit ownership and compatibility rules**. That architecture also gives Phalcom a substantially better foundation for the future formal typing/checker work, because the eventual type layer can consume stable `ModuleId`/`ClassId`/`CallableId` identities and incremental dependency machinery without inheriting the current batch-oriented cloning behavior.