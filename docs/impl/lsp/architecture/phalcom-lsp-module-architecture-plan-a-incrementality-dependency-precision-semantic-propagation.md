# Phalcom LSP Module Architecture — Implementation Plan A

**Plan type:** Repository-grounded, checkpoint-driven, patch-grade implementation program  
**Program:** Module Incrementality, Dependency Precision, and Semantic Propagation  
**Companion document:** Plan B — IDE Indexing, References, Rename, Retention, and Latency Architecture  
**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Exact remote HEAD:** `d60e4589352ac5f4167ba295e7e2a5f6c870ef4b`  
**HEAD commit:** `docs: record final module architecture gate`  
**HEAD commit date:** 2026-09-05  
**Production-code note:** `d60e458...` is one documentation-only commit after `fd7c7fd7feb9b354953ee8c443dc4b7faa0ced18`; production code is unchanged by the final-gate commit.  
**Repository-state limitation:** The available repository connector exposes the remote repository state. Local working-tree status and local uncommitted changes are not visible and are therefore unknown. The executing agent MUST run the drift protocol in §8 before editing.  
**Baseline test note:** C7 focused evidence and the full `phalcom-lsp` suite are green. The workspace-wide gate is currently red because of an existing `phalcom-core` baseline: 483 passed, 24 failed, 33 ignored at the recorded C7 final gate. This plan MUST preserve that baseline and MUST NOT classify pre-existing failures as new Plan-A regressions.

---

# 0. Checkpoint Map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| **A0 — Transactional current-state correctness** | 1–5 | A workspace mutation either publishes one self-consistent current module world or publishes nothing; source-authored failures become current partial products, never stale-current state. | `phalcom-modules` workspace-session hostile failure tests; partial transitive error tests; dependency-edge replacement; tolerant runtime-order tests. | Full LSP except one targeted publication smoke test; full workspace. |
| **A1 — Indexed topology is a production product** | 6–8 | Module topology and reverse imports are immutable snapshot products used directly by production queries; ordinary editor/module queries do not scan the workspace. | module query tests with topology/reverse indexes; zero-fallback-scan instrumentation; `phalcom-semantic` snapshot query tests. | Resolver selectivity and linking selectivity. |
| **A2 — Stable import-site resolution and validate-before-resolve** | 9–14 | Every import occurrence has stable compiler identity; positive and negative resolution products retain the topology facts that justify them; only invalid import sites invoke the resolver. | 1-of-20 import edit work-count test; negative-resolution reuse tests; prefix provenance tests; topology-delta tests; directory-cache tests. | Semantic consumer invalidation; full LSP. |
| **A3 — Affected-component incremental linking** | 15–19 | Linking is retained by component and recomputed only for components whose interface/resolution inputs changed; public linked surface and private linked dependency state have separate product identities. | disconnected-component work-count tests; runtime/reference/semantic graph delta tests; strict/tolerant parity tests; cycle-survivor initialization tests. | Fine-grained body invalidation. |
| **A4 — Exact module facts join the existing SemanticDb dependency graph** | 20–24 | Semantic consumers record exact local-name/import/public-export facts through existing `QueryKey` / `DependencyRecorder` / product fingerprints; no second module dependency engine exists. | DB dependency tests for exact export/name/absence reads; re-export retarget tests; missing-name tests; product-stability tests. | Removal of broad safety fingerprints until A5. |
| **A5 — Remove coarse whole-workspace semantic safety fingerprints** | 25–27 | Callable-body direct inputs contain only direct non-query inputs; cross-module/source-resolution meaning arrives exclusively through exact query dependencies, with explicit coarse fallbacks only for genuinely aggregate operations. | unused-export edit leaves consumer bodies at original product revision; missing-name-addition invalidates exact consumers; high-fanout propagation work-count tests; cold/incremental equivalence. | Global semantic aggregate sharding. |
| **A6 — Incremental semantic workspace aggregation** | 28–33 | `SemanticWorkspaceSession` no longer reconstructs declaration/hierarchy/alias/signature workspace state from every source after each change; unchanged module semantic shards are structurally reused. | per-module semantic-shard reuse counters; declaration/hierarchy/alias change isolation; callable reuse tests; cold-vs-incremental semantic parity. | Plan-B source/reference index work; retention/COW TypeStore work. |
| **A7 — Performance and release closure for Plan A** | 34–38 | Deterministic work counts prove update cost is proportional to changed products and actual semantic consumers; current LSP behavior remains green; no new workspace failures are introduced. | synthetic 1k/10k/50k module fixtures where practical; body/export/import/topology scenarios; full `phalcom-lsp`; affected crate suites; workspace baseline comparison. | Plan B editor-index/refactor/retention/scheduling work. |

**Implementation order is strict:**

```text
A0 → A1 → A2 → A3 → A4 → A5 → A6 → A7
```

Do not start A4 before A2 and A3 establish exact import/link products. Do not remove broad semantic safety fingerprints in A5 before A4 proves exact missing-name and retarget dependencies. Do not begin Plan B before A7 is complete.

---

# 1. Implementation Program

Plan A completes the compiler/module substrate required for a high-performance Phalcom IDE.

The existing C0–C7 program established the correct ownership model:

```text
EntryOwnership
    ↓
canonical ModuleId / package semantics
    ↓
UnlinkedModuleInterface
    ↓
canonical ModuleResolver
    ↓
LinkedProgram / LinkedModuleInterface
    ↓
SemanticWorkspaceSession / SemanticDb
    ↓
SemanticSnapshot
    ↓
EditorSemanticQuery
    ↓
LSP
```

The remaining problem is not that these authorities are absent. The remaining problem is that **update granularity is still too broad at several seams**, and a few current-generation correctness holes remain.

The target update pipeline after Plan A is:

```text
source/workspace mutation
        ↓
private transactional module delta
        ↓
changed source/interface products
        ↓
validate affected ImportSite products
        ↓
re-resolve invalid sites only
        ↓
relink affected components only
        ↓
publish exact import / local-name / public-export semantic products
        ↓
existing SemanticDb dependency-product validation
        ↓
recompute only consumers whose recorded facts changed
        ↓
replace only changed semantic module shards
        ↓
publish immutable current SemanticSnapshot
```

The core propagation law is:

```text
A thing changed physically
        ≠
a semantic consumer must recompute
```

Instead:

```text
physical/input change
    ↓
recompute or validate immediate product
    ↓
compare semantic product fingerprint
    ↓
consumer dependency observes same product fingerprint?
    yes → validate/reuse consumer
    no  → recompute consumer
```

Plan A MUST preserve the existing `SemanticDb` product-stability architecture. It MUST NOT introduce a second Pyrefly-inspired dependency engine beside it.

---

# 2. Repository-Grounded Baseline

## 2.1 Relevant ownership layers

| Crate | Current responsibility relevant to Plan A |
|---|---|
| `phalcom-ast` | Import/export/expose syntax and source ranges. No grammar change is planned. |
| `phalcom-modules` | Project/package ownership, canonical module identity, source providers, interfaces, import resolution, linking, module graphs, persistent workspace module lifecycle, topology model. |
| `phalcom-semantic` | Persistent semantic workspace, `SemanticDb`, dynamic query dependencies, declaration/type products, linked type resolution, immutable snapshots, source semantic projection. |
| `phalcom-lsp` | Source/workspace event scheduling and protocol adaptation over immutable compiler products. Plan A changes it only for regression evidence or minimal plumbing required by new snapshot products. |
| `phalcom-core` | Strict compiler/runtime module consumption. Used for parity and regression evidence; Plan A must not create LSP-only semantics. |

Plan A does **not** own:

- rename/refactor semantics;
- semantic-vs-textual reference indexes;
- retention tiers;
- editor overlay transactions;
- heavy-query scheduling lanes;
- TypeStore structural sharing/COW redesign;
- parser incrementality;
- VM representation;
- language grammar.

Those belong to Plan B or later work.

---

## 2.2 Existing primitives that MUST be reused

### Module layer

Existing authoritative types/products include:

```text
EntryOwnership
ModuleId
ModulePath
SourceId / SourceLocation
WorkspaceModuleSession
WorkspaceModuleStats
UnlinkedModuleInterface
ImportResolutionProduct
ResolutionTopologyDependencies
LinkedModule
LinkedProgram
LinkedModuleInterface
LinkedInterfaceFingerprint
ModuleTopology
ModuleQueryFacade
SymbolId
```

### Semantic incremental layer

The current repository already has:

```text
InputFingerprint
ProductFingerprint
QueryKey
QueryState
DependencyEdge
DependencyRecorder
DependencyIndex
SemanticDb::validate_reuse
SemanticDb::record_dependency
SemanticDb::discard_for_recompute
SemanticDb::invalidate_roots
SemanticDb::purge_module
last-known-good products
```

It also already has checker-side semantic read capture:

```text
SemanticDependency
TrackingTypeResolver
TrackingTypeHierarchy
CallableAnalysis.semantic_dependencies
```

These are the foundation of Plan A. Do not recreate them under module-specific names.

### Snapshot/editor layer

Existing immutable products include:

```text
SemanticSnapshot
ModuleQueryProducts
SourceSemanticIndex
EditorSemanticQuery
SemanticTargetId
SemanticDefinitionLocation
ImportBindingOrigin
```

Plan A wires more exact module products into snapshots. Plan B performs the larger source/reference-index and rename work.

---

# 3. Confirmed Remaining Defects at the Prepared Revision

The implementation agent should treat these as repository facts to re-verify during A0 drift checking.

| Finding | Current location / symbol | Planning consequence |
|---|---|---|
| `WorkspaceModuleSession::apply_batch` stages deltas but mutates provider/session/universe state before `rebuild()` succeeds | `phalcom-modules/src/session.rs` | Introduce a true private transaction/commit barrier. |
| Existing rollback test primarily proves an early parse-failure path | `phalcom-modules/tests/workspace_session.rs` | Add deterministic late-failure injection/evidence. |
| Interface build failure marks a module blocked but can leave the previous interface retained | `WorkspaceModuleSession::rebuild` | Current-generation invalidity must remove/replace current product; previous product may only survive as last-known-good. |
| Body-only shortcut can return retained linked/diagnostic state when no interface fingerprint changed | same | Shortcut must be generation-valid, not merely fingerprint-stable. |
| Newly discovered transitive `load_parsed` / `InterfaceBuilder::build` failures can propagate through `?` | same | Source-authored transitive errors must become current partial products. |
| Import products remain keyed by `(ModuleId, String)` | `WorkspaceModuleSession` | Introduce stable `ImportSiteId`. |
| Failed resolution products retain empty topology evidence | `phalcom-modules/src/resolver.rs` | Record negative/absence dependencies. |
| Provider topology invalidation is generation-wide | `FilesystemSourceProvider::invalidate_topology` | Add topology-delta validation above/beside cache generation; do not re-resolve every site merely because the provider generation changed. |
| Filesystem lookup repeatedly performs `is_file`, `is_dir`, and candidate checks | `FilesystemSourceProvider::{locate_internal,find_directory,find_final_candidates}` | Add directory topology snapshot/cache. |
| `ModuleTopology` exists but is not a published production snapshot product | `phalcom-modules/src/topology.rs`; `phalcom-semantic/src/snapshot.rs` | Wire existing topology into production queries. |
| `ModuleQueryFacade` can use topology/reverse maps but production `SemanticSnapshot::module_queries()` does not supply them | `phalcom-modules/src/query.rs`; `phalcom-semantic/src/snapshot.rs` | Remove ordinary scan fallback path after wiring. |
| Non-body updates still iterate/link all disconnected components | `WorkspaceModuleSession::rebuild` | Retain component products and compute an affected-component frontier. |
| Current linked public fingerprint excludes `linked_reads` and runtime dependencies | `phalcom-modules/src/fingerprint.rs` | Preserve this public-surface meaning; add a separate private dependency fingerprint/product. |
| Tolerant runtime-cycle path marks cycle modules blocked but does not recompute a real topological order for the surviving graph | `phalcom-modules/src/linker.rs` | Repair before retained component-graph merging. |
| `SemanticDb` is already fine-grained, but callable-body formal input hashes all source-resolution semantics and the whole `LinkedProgram` | `phalcom-semantic/src/db/fingerprint.rs` | Exact module facts must replace these conservative whole-workspace safety inputs. |
| `SemanticDependency::LinkedInterface(ModuleId)` is coarse for ordinary exact-name lookup | `checker/analysis.rs`, `checker/context.rs`, `db/query.rs` | Add exact linked-name/public-export query products and record them at lookup boundaries. |
| `SemanticWorkspaceSession::update_with_budget_and_cancel` still rebuilds declaration/hierarchy/alias and related aggregates across all sources | `phalcom-semantic/src/session.rs` | Introduce retained per-module semantic shards and delta composition. |
| Snapshot creation still clones `TypeStore` | `phalcom-semantic/src/session.rs` | Explicitly deferred to Plan B unless A7 profiling proves it blocks Plan-A acceptance. |

---

# 4. Source-of-Truth Matrix

| Fact | Authoritative owner after Plan A | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Source/project/package ownership | `phalcom-modules::EntryOwnership` + `WorkspaceModuleSession` project universe | resolver, topology, semantic snapshot, compiler/LSP | URI-parent heuristics, LSP workspace-folder semantics |
| Module identity | `phalcom_modules::ModuleId` | all layers | LSP-local module ID |
| Import occurrence identity | new compiler-owned `ImportSiteId` derived from interface/source structure | resolver products, reverse import-site index, prefix navigation products | `(ModuleId, String)` as identity |
| Import resolution meaning | `ImportResolutionProduct` for one `ImportSiteId` | linker, semantic query products, snapshot | ad hoc source-index/path lookup |
| Topology facts | `ModuleTopology` + explicit topology delta/dependency evidence | resolver validation, module queries | repeated filesystem scans in editor queries |
| Public linked surface | `LinkedModuleInterface` + existing public `LinkedInterfaceFingerprint` | export/name semantic products, completion, semantic resolver | private runtime-dependency hash pretending to be public API |
| Private linked dependency state | new retained linked-dependency product/fingerprint | affected-component invalidation, runtime/reference/semantic graph merge | overloading `LinkedInterfaceFingerprint` |
| Semantic dependency graph | existing `SemanticDb` / `DependencyIndex` | all formal semantic queries | second module dependency graph with parallel invalidation semantics |
| Exact local linked name meaning | new query-addressable linked-name fact `(module, local name)` | `TrackingTypeResolver`, declaration/type lookup | full `LinkedProgram` fingerprint |
| Exact public export meaning | new query-addressable public-export fact `(exporting module, public name)` | imported/member type lookup, downstream re-export/name lookup | entire `LinkedInterface` as dependency for every exact-name consumer |
| Missing name meaning | explicit `Absent` product for exact linked/public name fact | unresolved-name consumers | whole-workspace source-resolution hash |
| Declaration/hierarchy/alias workspace state | persistent module semantic shards composed by `SemanticWorkspaceSession` | query inputs/snapshot | rebuilding all sources on every update |

---

# 5. Non-Negotiable Plan-A Invariants

Use these identifiers in tests, comments, and implementation-state notes where useful.

### Transaction/currentness

**INC-TXN-1 — Atomic module publication**  
A failed workspace module transaction MUST NOT mutate committed ownership, overlay, interface, import, reverse-dependency, linked, diagnostic, or generation state.

**INC-CURRENT-1 — No stale-current products**  
If a product cannot be derived for generation N+1, a generation-N product MUST NOT be represented as current N+1 truth. Last-known-good storage is separate.

**INC-PARTIAL-1 — Source failure is current data**  
Parse/interface/import/link errors caused by current source/topology publish a current partial workspace whenever unaffected canonical products can still be produced.

### Resolution

**INC-SITE-1 — Stable import identity**  
Every authored import/re-export path consumed by resolution has a stable compiler-owned `ImportSiteId` independent of path-string cache keying.

**INC-RES-1 — Resolution invalidation follows resolution facts**  
An import site is re-resolved only when its syntax/context changes or a topology/ownership/exposure fact it actually depended on may have changed.

**INC-NEG-1 — Negative products retain evidence**  
A failed import resolution records enough absence/topology evidence to determine whether an unrelated topology event can be ignored.

**INC-PREFIX-1 — Import prefixes retain canonical targets**  
A compound import path retains the canonical module reached at each meaningful prefix, not only the final target.

### Linking

**INC-LINK-1 — Affected components only**  
A linked component is recomputed only when one of its interface/resolution/link dependency inputs changed.

**INC-LINK-2 — Public and private linked identity are distinct**  
Public export-surface stability is not invalidated by private linked-read/runtime-edge changes that do not alter public meaning.

**INC-GRAPH-1 — Retained graph merge is exact**  
Reference, semantic, and runtime graph contributions from an unchanged component are structurally reused; changed component contributions atomically replace old contributions.

### Semantic propagation

**INC-DEP-1 — One dependency engine**  
`SemanticDb` remains the sole formal semantic invalidation/product-stability engine. Plan A may add query products and dependency edges but MUST NOT add a parallel semantic invalidation graph.

**INC-NAME-1 — Exact name facts are query products**  
Ordinary exact name lookup depends on exact linked-name/public-export products, including explicit `Absent` products for failed lookup.

**INC-PROP-1 — Product stability stops propagation**  
A recomputed dependency whose semantic `ProductFingerprint` is unchanged must allow downstream cached products to validate without recomputation.

**INC-INPUT-1 — Direct inputs are truly direct**  
After A5, callable-body `InputFingerprint` MUST NOT hash the entire workspace source-resolution state or entire linked program to compensate for missing dependency edges.

### Semantic workspace aggregation

**INC-SHARD-1 — Module contribution replacement**  
A source/module change replaces only that module's declaration/hierarchy/alias/etc. contribution plus consumers selected by query dependencies.

**PERF-WORK-1 — Work proportionality**  
Warm update work scales with changed products and actual semantic consumers, not total workspace size except where a deliberately aggregate operation is requested.

---

# 6. Target Architecture

## 6.1 Module transaction shape

STRUCTURAL target:

```rust
pub struct WorkspaceModuleTransaction<'a> {
    base: &'a WorkspaceModuleSession,

    source_delta: ...,
    ownership_delta: ...,
    overlay_delta: ...,
    universe_delta: ...,

    interface_delta: ...,
    import_delta: ...,
    reverse_import_delta: ...,
    component_delta: ...,
    linked_delta: ...,
    graph_delta: ...,
    diagnostic_delta: ...,

    next_generation: u64,
}
```

The exact internal map/container types are implementation details. The semantic rule is exact:

```text
read unchanged state from base
write changed products into transaction-local delta
perform all fallible derivation privately
validate cross-index consistency
commit once
```

No rollback procedure should be necessary for normal failures because committed state is not mutated before commit.

---

## 6.2 Import-site product shape

STRUCTURAL target:

```rust
pub struct ImportSiteId {
    pub importer: ModuleId,
    pub local: ImportSiteLocalId,
}

pub struct ImportResolutionProduct {
    pub site: ImportSiteId,
    pub written_path: ImportPathIdentity,
    pub prefixes: Arc<[ResolvedImportPrefix]>,
    pub target: Result<ModuleId, ModuleResolutionError>,
    pub dependencies: ResolutionTopologyDependencies,
    pub fingerprint: ResolutionFingerprint,
}
```

`ImportSiteLocalId` MUST be derived from stable compiler/source structure, not LSP offsets or a global counter that changes when unrelated modules change. Exact representation is an A2 task.

The current `(ModuleId, String)` maps may exist temporarily as compatibility projections during migration, but they MUST cease being the retained product identity by A2 completion.

---

## 6.3 Topology dependency model

The final exact fields should be driven by current resolver behavior, but the product must be capable of representing at least:

```text
project/import-root selection consulted
package exposure interfaces consulted
ancestor package existence consulted
child/file/package candidate existence consulted
negative child/file/package absence consulted
nested project-boundary fact consulted
target project/module if resolved
```

Do not encode arbitrary filesystem paths into semantic identity when a canonical module/project/topology key exists.

---

## 6.4 Linked product split

Preserve the meaning of the current public fingerprint:

```text
LinkedPublicSurfaceFingerprint
    ≈ current LinkedInterfaceFingerprint
```

Add a private product/fingerprint representing linkage dependencies, such as:

```text
LinkedDependencyFingerprint
    linked import targets
    linked reads
    runtime dependencies
    relevant graph edge contribution
```

Exact naming is STRUCTURAL. Do not overload the existing public fingerprint.

---

## 6.5 Exact semantic module facts

Plan A should add the minimum exact product vocabulary needed to remove global safety fingerprints.

Recommended STRUCTURAL query/product families:

```text
ResolvedImport(ImportSiteId)
LinkedName(ModuleId, local name)
PublicExport(ModuleId, public name)
```

Possible concrete representation:

```rust
enum LinkedNameFact {
    Absent,
    Local(SymbolId),
    ImportedModule(ModuleId),
    ImportedBinding(SymbolId),
}

enum PublicExportFact {
    Absent,
    Present(LinkedExport),
}
```

Names/types may be adjusted to current repository conventions during A4, but the semantics are fixed:

- exact local name reads have exact product identity;
- exact public export reads have exact product identity;
- failed lookup is a first-class stable product;
- re-export retargeting changes the relevant exact product even if the final declaration surface is unchanged;
- bodies depend on these products through the existing `DependencyRecorder`.

`LinkedInterface(ModuleId)` remains valid for genuinely aggregate operations such as enumerating all exports. It must stop being the mandatory dependency for every exact-name read.

---

# 7. Pyrefly Adaptation Contract

Plan A intentionally adapts specific production-proven Pyrefly ideas, but only where they match Phalcom ownership.

## 7.1 Adapt closely

### Validate dirty resolution products before recomputing

Pyrefly's useful rule is:

```text
dirty event
    ≠
rebuild
```

Instead:

```text
dirty event
    ↓
validate recorded assumptions
    ↓
reuse if still valid
```

Phalcom applies this to `ImportResolutionProduct` and directory/package topology facts.

### Fine-grained cross-module semantic dependency precision

Pyrefly records the semantic fact read and intersects it with provider changes. Phalcom already has a more general product-fingerprint query DB, so the adaptation is:

```text
exact export/local-name/import fact
    → SemanticDb query product
    → ProductFingerprint
    → existing DependencyEdge
```

Do NOT add `ModuleDeps`/`ModuleChanges` beside `SemanticDb` unless an exact repository limitation is proven and escalated.

### Copy-on-write transaction philosophy

Unchanged committed module state should be read from the base session; only changed/touched products should be owned by the active transaction.

### Directory entry caching

Adapt the directory-snapshot principle to Phalcom's canonical package/file rules. Do not copy Python import heuristics.

---

## 7.2 Do not copy literally

Do not copy:

- Python module-name resolution semantics;
- Python wildcard/import conventions;
- Pyrefly's exact dependency enum;
- source-position identities where Phalcom already has canonical IDs;
- a module-only semantic engine that bypasses Phalcom `SemanticDb`;
- Python-specific AST reference scanning.

Phalcom already has stronger canonical identities and a formal semantic query graph. Plan A must exploit them.

---

# 8. Drift and Baseline Protocol

Before any implementation edit:

```bash
git status --short
git rev-parse HEAD
git log -n 12 --oneline
```

Prepared baseline:

```text
d60e4589352ac5f4167ba295e7e2a5f6c870ef4b
```

If HEAD differs:

1. inspect commits since `d60e458...`;
2. re-open all primary A0–A5 files listed below;
3. search for any already-landed `ImportSiteId`, topology snapshot publication, linked dependency fingerprint, exact export/name query, or transaction abstraction;
4. update the implementation-state note before editing;
5. preserve semantic invariants even if symbol names moved.

At minimum inspect current versions of:

```text
phalcom-modules/src/session.rs
phalcom-modules/src/source.rs
phalcom-modules/src/resolver.rs
phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/fingerprint.rs
phalcom-modules/src/topology.rs
phalcom-modules/src/query.rs
phalcom-modules/src/graph.rs

phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/resolver.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/db/dependency.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs

phalcom-lsp/src/analysis_service.rs
```

Baseline evidence before A0 implementation:

```bash
RUSTFLAGS='' cargo test -p phalcom-modules
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-lsp
```

For the workspace-wide baseline, record the exact failing test names rather than only counts:

```bash
RUSTFLAGS='' cargo test --workspace --all-targets
```

Expected historical classification from C7 final gate:

```text
phalcom-lsp: green
workspace: red due to existing phalcom-core baseline
```

If the failure set differs materially before Plan A edits, mark `BASELINE-DRIFT` and reconcile before implementation.

---

# 9. Checkpoint A0 — Transactional Current-State Correctness

Tasks:
- Task 1 — re-ground the current failure and test baseline.
- Task 2 — introduce a true private workspace-module transaction and one commit barrier.
- Task 3 — make current-generation interface/product validity explicit.
- Task 4 — make transitive source/interface/import discovery fully tolerant.
- Task 5 — make forward/reverse dependency replacement and tolerant runtime graph publication exact.

## Why this is a checkpoint

Every later optimization retains more state for longer. Retaining state before transaction/currentness rules are airtight would make stale-product and half-commit bugs significantly harder to diagnose.

A0 therefore establishes the only safe base for incremental reuse:

> After A0, the session may aggressively retain products because it can prove that every committed generation is internally consistent and current-source failures cannot leak stale prior meaning into that generation.

## Entry conditions

- C7 is committed.
- Current `phalcom-lsp` baseline is green.
- Existing persistent `WorkspaceModuleSession`, `WorkspaceModuleStats`, partial diagnostics, and `SemanticDb::purge_module` remain intact.

## Working set

Primary:

- `phalcom-modules/src/session.rs` — `WorkspaceModuleSession`, `apply_batch`, `rebuild`, reclassification/removal logic.
- `phalcom-modules/src/linker.rs` — tolerant runtime-cycle behavior and component products.
- `phalcom-modules/src/graph.rs` — runtime initialization order and graph operations.
- `phalcom-modules/tests/workspace_session.rs` — persistent lifecycle and rollback regressions.

Secondary — inspect only if required:

- `phalcom-modules/src/source.rs` — overlay/provider mutation API needed for transaction staging.
- `phalcom-semantic/src/session.rs` — publication behavior when module update returns current partial state.
- `phalcom-lsp/src/analysis_service.rs` — one smoke test proving partial module publication reaches LSP.

Out of scope:

- `ImportSiteId`.
- resolver selectivity.
- affected-component retention.
- semantic dependency precision.

## Semantic contract established

- `INC-TXN-1`.
- `INC-CURRENT-1`.
- `INC-PARTIAL-1`.
- dependency edge maps are forward/reverse consistent at commit.
- tolerant runtime publication has a valid initialization order for the surviving unblocked graph.

## Semantic risks

- accidentally cloning the whole session and calling that “transactional” while preserving O(workspace) edit cost;
- mutating shared `FilesystemSourceProvider` cache state before commit in ways that cannot be reverted;
- preserving stale interfaces for editor convenience and thereby violating current truth;
- treating parse/interface errors as infrastructure errors;
- dropping blocked cycle nodes without rebuilding surviving runtime order;
- deleting reverse edges too early and missing importer revalidation.

## Hostile cases

- A mutation succeeds through parsing/ownership but an injected late rebuild/link stage fails.
- A previously valid module changes to an interface-invalid module.
- A newly discovered transitive dependency is syntactically/interface invalid.
- Importer A retargets from B to C in one transaction.
- A runtime cycle affects X/Y while unrelated Z→W remains valid and orderable.

---

## Task 1 — Lock the current baseline and add late-failure test seams

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: test/support only

**Classification:** EXACT for baseline capture; STRUCTURAL for fault-injection seam.

### Edit operations

1. Add/update a Plan-A implementation-state document under the repository's existing module-work state area. Record:
   - exact local HEAD;
   - local dirty state;
   - C7 full-LSP result;
   - workspace failing test names/counts;
   - A0 start time/revision.
2. Add a test-only failure injection seam at a point **after** live-state mutation currently begins but **before** successful final publication. Prefer a `#[cfg(test)]` hook or a private injected strategy rather than production environment variables.
3. Extend the existing rollback fixture so it proves a late-stage failure rather than an early parse rejection.

### Required invariant

The failing test must demonstrate why the current implementation is insufficient before Task 2. Do not weaken it to match current behavior.

### Testing classification

Run only the exact new regression before Task 2 and expect it to fail on the old implementation.

---

## Task 2 — Introduce the private module transaction and one commit barrier

**Risk:**  
Semantic: HIGH  
Implementation fanout: multi-file, central module lifecycle

**Classification:** STRUCTURAL.

### Primary edits

`phalcom-modules/src/session.rs`

Introduce an internal transaction object or equivalent delta world with these properties:

```text
base session is immutable for transaction duration
transaction owns changed overlay/source/ownership/interface/import/link/diagnostic deltas
fallible rebuild reads base + deltas
commit applies all deltas exactly once
```

### Required implementation rules

1. `self.generation` does not advance until commit.
2. `modules_by_source`, `sources_by_module`, project roots, standalone-project identity, universe replacement, interface maps, import maps, reverse edges, linked maps, diagnostics, and blocked sets are not mutated in committed state during fallible derivation.
3. Overlay/provider behavior must be transaction-safe:
   - preferred: stage overlay mutations in transaction-visible provider overlay state and commit them later;
   - acceptable: construct a private overlay view over the existing base provider;
   - forbidden: mutate the shared provider and write rollback code for every possible failure.
4. Existing caches may be read during derivation. Cache entries that are pure memoization may be populated if they cannot change semantic committed state; identity/overlay cache mutations that affect future resolution must obey the commit barrier.
5. The transaction commit validates cross-index consistency before installing deltas.

### Do not do

Do not implement:

```rust
let mut cloned_session = self.clone();
...
*self = cloned_session;
```

as the final architecture. That is atomic but defeats the performance objective.

### Caller updates

Update `apply`, `apply_batch`, project-marker/source mutation helpers, and any reclassification helper to operate through the transaction or transaction-owned delta view.

---

## Task 3 — Make current-generation module product validity explicit

**Risk:**  
Semantic: HIGH  
Implementation fanout: session/interface publication

**Classification:** STRUCTURAL.

### Problem to remove

Current interface storage can preserve an older `UnlinkedModuleInterface` when the new source cannot build one, allowing a shortcut to treat the retained product as current.

### Required architecture

Use either:

```rust
enum CurrentInterfaceState {
    Valid(Arc<UnlinkedModuleInterface>, InterfaceFingerprint),
    Invalid,
}
```

or equivalent map/removal semantics that make generation validity explicit.

The exact representation is flexible; behavior is not.

### Required rules

- old valid product may remain in last-known-good storage if useful;
- current interface map supplied to linker/semantic session contains only products valid for the current transaction generation;
- a module that fails interface construction is blocked/current-diagnostic and cannot contribute stale exports/imports;
- body-only shortcut requires proof that every retained current product is valid for the proposed generation.

### Regression

Valid `provider.ph` → edit to interface-invalid provider while consumer remains open:

```text
new snapshot is Partial/current
provider old export is not current
consumer cannot navigate/typecheck through stale provider meaning
unrelated modules remain available
```

---

## Task 4 — Make transitive discovery failures current partial products

**Risk:**  
Semantic: HIGH  
Implementation fanout: resolver/session diagnostics

**Classification:** EXACT in behavior, STRUCTURAL in helper factoring.

### Primary edit

Replace source-authored `?` propagation in the transitive import closure with classification into:

```text
current module diagnostic
blocked module
retained unaffected products
```

Infrastructure failures remain `Err`.

### Distinguish

Source/current-topology failures include, at minimum:

- syntax/parse error in discovered source;
- interface construction error;
- unresolved import/module path;
- missing/non-exported name;
- source-authored runtime cycle.

Infrastructure failures include unexpected I/O/internal invariant failures that prevent a trustworthy current world from being constructed.

Use existing diagnostic enums; do not invent an LSP diagnostic channel.

---

## Task 5 — Atomic dependency-edge replacement and correct tolerant runtime order

**Risk:**  
Semantic: HIGH  
Implementation fanout: module graph/linker/session

**Classification:** STRUCTURAL.

### Forward/reverse import edges

Introduce one helper/operation that reconciles old/new dependencies atomically:

```text
old forward set
new forward set
    ↓
remove reverse edges for old-new
add reverse edges for new-old
publish new forward set
```

Do not rely on “remove module then later rebuild reverse map” sequencing.

### Tolerant runtime graph

When blocking cycle nodes:

1. remove/ignore blocked nodes and their edges in the surviving runtime graph view;
2. recompute a real topological initialization order for surviving nodes;
3. retain diagnostic/cycle provenance separately;
4. when merging component graphs, never use `unwrap_or_default()` to silently turn an invalid combined graph into an empty initialization order.

### Required evidence

1. Late-failure transaction regression now passes.
2. Interface-invalid current edit publishes partial current state without stale interface.
3. Invalid transitive dependency publishes partial state.
4. A import B → A import C leaves exact reverse edges.
5. Runtime cycle X↔Y with independent Z→W yields blocked X/Y and valid topological order for Z/W.

Recommended checkpoint gate:

```bash
RUSTFLAGS='' cargo test -p phalcom-modules --test workspace_session
RUSTFLAGS='' cargo test -p phalcom-modules --test linking
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic module_query_provenance
```

Add one targeted LSP partial-publication test only if module-session evidence cannot prove current snapshot delivery.

## Do not run yet

```bash
cargo test --workspace --all-targets
```

No new cross-workspace evidence is gained yet.

## Escalate immediately if

- transaction-safe overlay staging requires redesigning `SourceProvider` beyond a bounded adapter/view;
- an expected source-authored failure cannot be represented through current module diagnostics;
- runtime graph APIs cannot represent a filtered surviving graph without semantic loss.

## Checkpoint completion

- [ ] Tasks 1–5 implemented.
- [ ] late-failure atomicity proven.
- [ ] stale-current interface regression proven.
- [ ] transitive partial publication proven.
- [ ] forward/reverse edge parity proven.
- [ ] tolerant runtime survivor order proven.
- [ ] implementation-state document updated.
- [ ] no active A0 incident.

---

# 10. Checkpoint A1 — Indexed Topology Is a Production Product

Tasks:
- Task 6 — make `ModuleTopology` a retained workspace-session product.
- Task 7 — publish topology and reverse-import indexes into `SemanticSnapshot::ModuleQueryProducts`.
- Task 8 — retire normal production scan fallbacks and add query-work instrumentation.

## Why this is a checkpoint

A2 selective import validation requires stable topology facts. Plan B editor features also require O(children)/O(reverse-deps) module queries. The topology type already exists; A1 turns it from test/support infrastructure into authoritative production data.

## Entry conditions

- A0 COMPLETE.
- `WorkspaceModuleSession` can safely publish one committed generation.
- Existing `ModuleTopology` tests remain green.

## Working set

Primary:

- `phalcom-modules/src/topology.rs` — `ModuleTopology`.
- `phalcom-modules/src/session.rs` — committed topology lifecycle.
- `phalcom-modules/src/query.rs` — `ModuleQueryFacade` indexed paths/fallbacks.
- `phalcom-semantic/src/snapshot.rs` — `ModuleQueryProducts`, `module_queries()`.

Secondary:

- semantic session snapshot construction.
- module query tests.

Out of scope:

- exact topology-delta dependency validation; A2 owns that.
- reference/source indexes; Plan B.

## Semantic contract established

- topology for generation N is immutable and belongs to the same current module world as interfaces/imports;
- reverse importers are direct retained products, not recomputed by scanning resolution maps;
- ordinary production module queries never reconstruct topology from strings/maps.

---

## Task 6 — Retain current `ModuleTopology` in the module session

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: module session/topology

**Classification:** EXACT in owner, STRUCTURAL in delta mechanics.

### Edit operations

1. Add current topology to committed `WorkspaceModuleSession` state.
2. Build/update it transaction-locally from current universe, valid interfaces, and source identities.
3. Preserve generation alignment with `WorkspaceModuleSession::generation()`.
4. Prefer delta update APIs if simple; rebuilding `ModuleTopology` from current committed module maps is acceptable initially if it occurs once per topology-changing transaction and is measured. A7 may optimize further if this is material.
5. Expose an immutable accessor.

Do not derive topology in the LSP.

---

## Task 7 — Publish topology and reverse imports into `ModuleQueryProducts`

**Risk:**  
Semantic: LOW  
Implementation fanout: semantic snapshot plumbing

**Classification:** EXACT.

Extend `ModuleQueryProducts` with retained immutable products equivalent to:

```rust
pub topology: Arc<ModuleTopology>,
pub reverse_imports: Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>>,
```

Use canonical types already owned by `phalcom-modules`.

Update:

```text
ModuleQueryProducts::new
ModuleQueryProducts::empty
SemanticSnapshot::module_queries
snapshot construction in SemanticWorkspaceSession
```

so production `ModuleQueryFacade` receives `.with_topology(...)` / `.with_reverse_imports(...)` or equivalent direct constructor inputs.

---

## Task 8 — Remove normal scan fallbacks and instrument them

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: module query facade/tests

**Classification:** STRUCTURAL.

### Required behavior

For production snapshot-backed facade:

```text
module_children(parent) → topology.children[parent]
reverse_importers(module) → reverse index[module]
module_for_source(source) → topology/source_modules
```

Fallback scan implementations may remain for isolated unit construction only if clearly marked/test-only or explicit compatibility constructors. They must not be silently used by `SemanticSnapshot`.

Add a debug/test counter or explicit test facade proving:

```text
production_query_fallback_scans == 0
```

for representative module completion/navigation/query suites.

### Required evidence

```bash
RUSTFLAGS='' cargo test -p phalcom-modules --test query
RUSTFLAGS='' cargo test -p phalcom-modules --test topology
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic module_query_provenance
```

Add a synthetic topology test with thousands of nodes only if it can deterministically assert indexed work rather than wall-clock timing.

## Checkpoint completion

- [ ] session publishes current topology.
- [ ] snapshot owns topology + reverse imports.
- [ ] `module_queries()` uses indexed products.
- [ ] zero production fallback scans proven.
- [ ] no A1 incident.

---

# 11. Checkpoint A2 — Stable Import-Site Resolution and Validate-Before-Resolve

Tasks:
- Task 9 — define stable `ImportSiteId` and assign it during interface/source extraction.
- Task 10 — migrate resolution products/maps to import-site identity and retain prefix targets.
- Task 11 — record positive and negative topology dependencies precisely.
- Task 12 — introduce topology deltas and validate retained resolution products before resolver invocation.
- Task 13 — maintain exact forward/reverse import-site indexes.
- Task 14 — add directory topology snapshots and align filesystem cache lifecycle.

## Why this is a checkpoint

A2 is the first major performance boundary. The current system can retain module-level resolution products, but one topology/interface change still causes broad path work and failed resolutions cannot prove they remain valid.

After A2:

> Import resolution becomes a retained product per authored import site. Filesystem/topology events validate those products; only sites whose recorded assumptions changed invoke `ModuleResolver`.

## Entry conditions

- A1 COMPLETE.
- Current topology and reverse-import indexes are authoritative.
- Transactional commit from A0 can atomically replace resolution/index products.

## Working set

Primary:

- `phalcom-modules/src/interface.rs` — import surfaces and stable source/interface identity.
- `phalcom-modules/src/resolver.rs` — `ImportResolutionProduct`, trace/dependency capture.
- `phalcom-modules/src/session.rs` — retained import products, resolution loop, reverse edges.
- `phalcom-modules/src/source.rs` — filesystem cache/directory topology.
- `phalcom-modules/src/topology.rs` — topology delta/input keys.

Secondary:

- semantic source-index context only for prefix-product plumbing; do not perform Plan-B indexing changes.
- module resolution/integration tests.

Out of scope:

- semantic dependency capture from exact imports; A4.
- component linking selectivity; A3 consumes A2 products.

## Semantic risks

- making `ImportSiteId` depend directly on mutable byte offsets;
- reusing an import product after its site changed meaning but retained the same path spelling;
- recording only positive targets and missing absence dependencies;
- using raw filesystem path strings instead of canonical topology facts;
- invalidating every site whenever the filesystem provider generation changes;
- retaining stale reverse edges when an import retargets.

## Hostile cases

- two identical textual import paths in the same module.
- path range moves because unrelated source text is inserted.
- 20 imports; only one path changes.
- missing `.foo`; unrelated `bar.ph` is created.
- missing `.foo`; `foo.ph` is created.
- package exposure changes without physical target movement.
- project/import-root mapping changes.
- `import foo.bar.baz` where each prefix must retain its own canonical module target.

---

## Task 9 — Define and assign stable `ImportSiteId`

**Risk:**  
Semantic: HIGH  
Implementation fanout: interface/resolver/session/tests

**Classification:** STRUCTURAL.

### Required identity properties

An `ImportSiteId` must distinguish:

```text
same importer + two identical written imports at different authored sites
```

and remain stable across edits that do not semantically replace/reorder that import site under the repository's chosen structural identity rules.

Prefer deriving the local ID from the interface/source structural indexing mechanism rather than byte offset alone.

Possible shapes:

```rust
pub struct ImportSiteLocalId(pub u32);

pub struct ImportSiteId {
    pub importer: ModuleId,
    pub local: ImportSiteLocalId,
}
```

If the interface already preserves deterministic preamble order, a local ordinal may be sufficient for the first implementation **only if** edits before the import do not cause unrelated import sites to be mis-associated. If ordinal stability is inadequate, derive the identity from the same stable source-site structure used elsewhere.

### Required tests

- duplicate textual paths have distinct IDs;
- body-only edit retains all import site IDs;
- editing one import leaves unaffected import-site products reusable.

---

## Task 10 — Migrate retained resolution products to site identity and add prefix provenance

**Risk:**  
Semantic: HIGH  
Implementation fanout: modules + semantic plumbing

**Classification:** STRUCTURAL.

### Replace retained authority

From:

```rust
BTreeMap<(ModuleId, String), Arc<ImportResolutionProduct>>
```

To:

```rust
BTreeMap<ImportSiteId, Arc<ImportResolutionProduct>>
```

or equivalent canonical map.

Temporary compatibility projection:

```text
(importer, written path) → final target
```

may remain for old consumers during A2/A3 migration, but it must be derived from site products and must reject/handle ambiguous duplicate written paths rather than silently selecting one.

### Prefix provenance

Extend successful resolution tracing so:

```phalcom
import foo.bar.baz
```

can retain conceptually:

```text
foo         → ModuleId(foo)
foo.bar     → ModuleId(foo.bar)
foo.bar.baz → ModuleId(foo.bar.baz)
```

For relative roots, retain the meaningful resolved package/root and subsequent segments according to actual Phalcom semantics.

This data is produced by the canonical resolver, not reconstructed by the semantic source index.

---

## Task 11 — Record positive and negative topology dependencies

**Risk:**  
Semantic: HIGH  
Implementation fanout: resolver/source/topology

**Classification:** STRUCTURAL.

### Extend `ResolutionTopologyDependencies`

Record every topology fact whose change can alter resolution.

At minimum support canonical representations of:

- selected import root / target project;
- consulted package exposure interfaces;
- required ancestor package markers;
- file/package candidate existence;
- missing candidate/child facts for failed resolution;
- nested project boundary checks;
- final target if present.

A failed `ModuleNotFound`/`PackageNotFound` result MUST no longer publish an empty dependency set merely because there is no target.

### Negative dependency principle

For a failed `.foo` lookup, the product must encode enough information that:

```text
create unrelated bar.ph
```

can prove `.foo` remains unchanged without invoking the resolver, while:

```text
create foo.ph
```

invalidates it.

Do not over-model every filesystem syscall. Model canonical facts that determine Phalcom resolution.

---

## Task 12 — Topology delta and validate-before-resolve

**Risk:**  
Semantic: HIGH  
Implementation fanout: session/topology/resolver

**Classification:** STRUCTURAL.

Introduce a transaction-local `TopologyDelta` or equivalent change summary with enough precision to answer:

```rust
fn resolution_product_may_have_changed(
    product: &ImportResolutionProduct,
    delta: &TopologyDelta,
) -> bool
```

The exact API name is flexible.

### Resolution selection algorithm

```text
import site syntax changed?      → resolve
new import site?                 → resolve
old site removed?                → remove product/edges
relevant topology fact changed?  → resolve
otherwise                        → reuse product
```

A provider topology generation bump alone is not sufficient reason to rerun every import site.

### Required statistics

Add counters such as:

```text
import_sites_considered
import_sites_validated
import_sites_reused
imports_resolved
negative_resolutions_reused
```

Preserve existing stats where names overlap; do not duplicate counters with ambiguous meanings.

---

## Task 13 — Exact forward/reverse import-site indexes

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: session/query products

**Classification:** STRUCTURAL.

Maintain at least:

```text
importer ModuleId → ImportSiteIds
target ModuleId → ImportSiteIds
ImportSiteId → target/result
```

Then derive module-level reverse importers as a cheap projection for existing APIs.

When a site retargets:

```text
old target reverse-site edge removed
new target reverse-site edge added
```

within the same transaction commit.

This index becomes the A3 linker affected-input source and later Plan-B provenance source.

---

## Task 14 — Directory topology snapshot cache

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: filesystem provider

**Classification:** STRUCTURAL.

Current provider resolution cache by `(generation, project, ModulePath)` is already useful and should remain.

Add a directory-level cache so one directory read can answer repeated candidate questions without repeated `is_file`/`is_dir` probes.

Conceptually:

```rust
DirectorySnapshot {
    entries: name → kind/file/dir facts,
    package_marker_present: bool,
    project_marker_present: bool,
}
```

Requirements:

- obey canonical kebab-name checks;
- invalidate only affected directory snapshots on topology events where event path is known;
- keep source text cache invalidation separate;
- preserve confinement/canonicalization checks where security/correctness requires them;
- do not copy Python-specific resolver behavior.

### Required evidence for A2

1. **20 imports, edit one path**

```text
imports_resolved = 1
import_resolutions_reused = 19
```

2. **body-only edit**

```text
imports_resolved = 0
```

3. **export-only semantic edit with unchanged import paths**

```text
imports_resolved = 0
```

4. **negative unrelated topology event**

```text
.foo unresolved
create unrelated bar.ph
imports_resolved for .foo = 0
negative resolution reused
```

5. **negative relevant topology event**

```text
create foo.ph
only sites whose absence fact matches foo are re-resolved
```

6. **prefix provenance**

Each authored segment of a compound path has the expected canonical prefix target.

Recommended gate:

```bash
RUSTFLAGS='' cargo test -p phalcom-modules
```

Do not run full LSP yet unless an existing LSP module-navigation fixture is directly changed by compatibility projection removal.

## Checkpoint completion

- [ ] stable site identities proven.
- [ ] retained site-keyed products installed.
- [ ] negative evidence retained.
- [ ] validate-before-resolve active.
- [ ] exact reverse site index active.
- [ ] directory snapshots active.
- [ ] deterministic work-count tests pass.
- [ ] no A2 incident.

---

# 12. Checkpoint A3 — Affected-Component Incremental Linking

Tasks:
- Task 15 — define retained component identity/membership over the current import/link graph.
- Task 16 — compute the affected-component frontier from interface and import-site deltas.
- Task 17 — split public linked-surface identity from private linked-dependency identity.
- Task 18 — retain and delta-replace linked module/graph contributions by component.
- Task 19 — close strict/tolerant parity and merged-runtime-order correctness.

## Why this is a checkpoint

A2 ensures only genuinely changed import sites are re-resolved. Without A3, a small resolution/interface delta can still trigger linking of every disconnected component. A3 ensures the same locality continues through linking.

After A3:

> A component is a retained linker product. An update recomputes only components whose canonical interface/resolution inputs changed, then atomically replaces their linked modules and graph contributions.

## Entry conditions

- A2 COMPLETE.
- Import-site forward/reverse indexes are canonical.
- A0 tolerant runtime publication is correct.

## Working set

Primary:

- `phalcom-modules/src/session.rs` — current component loop and retained linked maps.
- `phalcom-modules/src/linker.rs` — component linking and graph production.
- `phalcom-modules/src/graph.rs` — component graph contribution/merge.
- `phalcom-modules/src/fingerprint.rs` — linked product identity.

Secondary:

- topology for component membership helpers.
- linker/session tests.

Out of scope:

- semantic DB exact export dependencies; A4.
- source/reference indexes.

## Semantic risks

- conflating undirected connectivity with runtime/semantic dependency direction;
- reusing a component after a private import target changed because public fingerprint remained stable;
- making private linkage changes propagate as public API changes;
- failing to replace old graph edges when a component changes;
- unstable component IDs causing all components to appear changed after unrelated edits.

---

## Task 15 — Retained component identity and membership

**Risk:**  
Semantic: HIGH  
Implementation fanout: linker/session/graph

**Classification:** STRUCTURAL.

Define the exact graph relation that determines one linker recomputation unit using current linker semantics. The unit must include all modules that must be considered together for canonical export resolution/cycles.

Do not assume the graph is a DAG.

Possible implementation:

```text
ComponentId = deterministic representative ModuleId or interned component key
module → ComponentId
ComponentId → member ModuleIds
```

Component identity should remain stable when an unrelated disconnected component changes. If membership itself changes, affected old/new components are recomputed.

---

## Task 16 — Compute affected components from canonical deltas

**Risk:**  
Semantic: HIGH  
Implementation fanout: session

**Classification:** STRUCTURAL.

Seed affected components from:

- valid unlinked interface product changed;
- module added/removed/reidentified;
- import-site target/result changed;
- component membership changed;
- package/exposure input changed in a way that changes linked interface input.

Do **not** seed from:

- body-only source edit with stable interface;
- import site whose retained resolution product validated unchanged;
- unrelated topology event.

Then link each affected component exactly once.

---

## Task 17 — Split public linked surface and private dependency fingerprints

**Risk:**  
Semantic: HIGH  
Implementation fanout: fingerprint/session/semantic bridge

**Classification:** EXACT in semantic split; STRUCTURAL in naming.

Preserve current `LinkedInterfaceFingerprint` semantics as public surface identity or rename it only with complete caller migration.

Add a second fingerprint/product containing private linkage facts needed to decide whether the component/module's internal link product can be reused.

At minimum private identity must account for:

- linked import target identity;
- `linked_reads`;
- runtime dependencies;
- graph edge contribution that can change runtime/reference behavior.

Do not include source ranges in public semantic fingerprint unless they are semantically meaningful.

---

## Task 18 — Retain component linked/graph contributions

**Risk:**  
Semantic: HIGH  
Implementation fanout: session/linker/graph

**Classification:** STRUCTURAL.

Maintain retained products sufficient to reconstruct `LinkedProgram` without relinking unchanged components.

Conceptually:

```text
ComponentLinkedProduct {
    modules
    reference graph contribution
    semantic graph contribution
    runtime graph contribution
    blocked modules
    diagnostics
    public fingerprints
    private dependency fingerprints
}
```

On update:

```text
unchanged component → retain Arc/product
changed component   → compute replacement
removed component   → delete contribution
merge current contributions → LinkedProgram
```

Prefer structural sharing of per-component maps/Arcs where practical, but do not make Plan-A correctness depend on a new persistent-map library.

---

## Task 19 — Strict/tolerant parity and merged runtime order

**Risk:**  
Semantic: HIGH  
Implementation fanout: linker/session/tests

**Classification:** STRUCTURAL.

Ensure:

- strict link of a valid closed component and tolerant link of the same valid component produce equivalent linked modules/public exports/runtime edges;
- tolerant diagnostics/blocked sets are extra products, not alternate linking semantics;
- merged runtime initialization order is computed from the current merged surviving runtime graph, not concatenated component-local vectors if cross-component runtime edges can exist;
- if component decomposition guarantees no cross-component runtime edges, encode/assert that invariant and compose deterministically.

### Required evidence

Fixture:

```text
Component 1: A ↔ B (semantic/reference cycle if valid)
Component 2: C → D
Component 3: E
```

Edit B body only:

```text
linked_components = 0
```

Edit B public interface:

```text
linked_components_recomputed = 1
C/D/E component products retained
```

Change one import target in C:

```text
only C/D component affected unless membership changes
```

Private dependency retarget with public exports stable:

```text
private dependency fingerprint changes
public linked fingerprint stable
semantic downstream public consumers not invalidated solely by public fingerprint
```

Recommended gate:

```bash
RUSTFLAGS='' cargo test -p phalcom-modules
```

## Checkpoint completion

- [ ] stable retained component model.
- [ ] affected frontier proven.
- [ ] public/private fingerprint split installed.
- [ ] graph contributions delta-replaced.
- [ ] strict/tolerant parity proven.
- [ ] disconnected component work-count tests pass.
- [ ] no A3 incident.

---

# 13. Checkpoint A4 — Exact Module Facts Join the Existing SemanticDb Dependency Graph

Tasks:
- Task 20 — define minimal exact module-semantic query keys/products.
- Task 21 — publish/import these products from canonical module/link state into `SemanticDb`.
- Task 22 — record exact linked-name and public-export reads at semantic lookup boundaries.
- Task 23 — make absence and re-export retargeting first-class dependency behavior.
- Task 24 — preserve explicit aggregate dependencies only for genuinely aggregate operations.

## Why this is a checkpoint

This is the pivotal architecture checkpoint.

Phalcom already has dynamic query dependencies, input/product fingerprint separation, lazy current-revision validation, and product-stability propagation. The missing piece is that module/name-resolution facts are still too coarse, so callable bodies compensate by hashing the whole workspace source-resolution and linked-program state.

A4 supplies exact module facts to the existing DB. A5 then removes the coarse safety hashes.

After A4:

```text
exact import/local/public-name meaning
        ↓
SemanticDb product fingerprint
        ↓
DependencyEdge
        ↓
exact semantic consumer
```

There is still only one semantic dependency engine.

## Entry conditions

- A3 COMPLETE.
- exact import-site products and affected linked products are current.
- existing DB product-stability tests remain green.

## Working set

Primary:

- `phalcom-semantic/src/db/key.rs` — query identity.
- `phalcom-semantic/src/db/product.rs` — typed products.
- `phalcom-semantic/src/db/fingerprint.rs` — semantic product fingerprints.
- `phalcom-semantic/src/db/query.rs` — publication/query helpers.
- `phalcom-semantic/src/checker/analysis.rs` — `SemanticDependency` vocabulary.
- `phalcom-semantic/src/checker/context.rs` — `TrackingTypeResolver` lookup capture.
- `phalcom-semantic/src/resolver.rs` — `LinkedTypeResolver` exact lookup boundaries.
- `phalcom-semantic/src/session.rs` — publication of module-derived query roots.

Secondary:

- `phalcom-modules` exact product accessors from A2/A3.
- semantic incremental/product-stability tests.

Out of scope:

- deleting global safety fingerprints before exact dependencies are proven; A5.
- rename/name-rewrite identity; Plan B.

## Semantic risks

- duplicating `DependencyIndex` with a module-specific reverse-dependency graph;
- using `SymbolId` as public export identity when re-exporting module/public name is the semantic fact consumed;
- failing to record `Absent` lookup dependency;
- recording only final `DeclarationId`, missing re-export retarget semantics;
- keeping `LinkedInterface(current_module)` in addition to exact dependencies for every lookup, defeating precision;
- making query products source-range-sensitive and causing irrelevant invalidation.

---

## Task 20 — Define the minimal exact query/product vocabulary

**Risk:**  
Semantic: HIGH  
Implementation fanout: semantic DB identity/product/fingerprint

**Classification:** STRUCTURAL.

Recommended minimum:

```text
QueryKey::ResolvedImport(ImportSiteId)
QueryKey::LinkedName(<module-local-name identity>)
QueryKey::PublicExport(<module-public-name identity>)
```

If current `QueryKey::ResolvedImports(ModuleId)` is useful for aggregate consumers, retain it as an aggregate/fallback product. Do not force all exact consumers through it.

### Identity requirements

`PublicExport` identity MUST contain the **exporting module + public spelling**, not merely the final upstream `SymbolId`.

Example:

```phalcom
// b.ph
export Foo as PublicFoo from .a
```

The downstream fact is:

```text
PublicExport(b, "PublicFoo")
```

whose value may point to upstream `a::Foo`.

### Product requirements

Every exact name product has an explicit absence representation.

Semantic product fingerprints should hash semantic identity/target and relevant metadata only; source range/provenance belongs in input/presentation products unless it changes semantic meaning.

---

## Task 21 — Publish exact module products into SemanticDb

**Risk:**  
Semantic: HIGH  
Implementation fanout: semantic session/db

**Classification:** STRUCTURAL.

At semantic workspace update, publish/validate roots derived from current module state:

```text
ResolvedImport(site)
LinkedName(module,name)
PublicExport(module,name)
```

Do not eagerly materialize every possible absent name in the universe. Use demand-driven query creation for exact lookup where appropriate.

For present names/imports, eager publication of compact products is acceptable if it simplifies integration and is measured.

Use existing:

```text
InputFingerprint
ProductFingerprint
publish_product_ready
validate_reuse
discard_for_recompute
```

### Product dependency chain

A recommended dependency relationship is:

```text
ResolvedImport(site)
    depends on canonical retained import product input

LinkedName(module, local)
    depends on exact relevant linked module/private linkage product or exact import product

PublicExport(module, name)
    depends on linked public surface and, for re-export, exact upstream export/name facts as needed
```

Do not depend every exact public-export query directly on the entire workspace `LinkedProgram`.

---

## Task 22 — Record exact reads at semantic resolver boundaries

**Risk:**  
Semantic: HIGH  
Implementation fanout: checker/resolver/db query bridge

**Classification:** STRUCTURAL.

Modify the canonical type/name resolution path so an exact read records the exact product it consumed.

Examples:

### Local declaration

```text
resolve root `User` locally
→ record DeclarationShell(User) as today
→ no unnecessary whole-module export dependency
```

### Imported local alias

```text
resolve local root `Models`
→ record LinkedName(current_module, "Models")
→ if resolving Models.User, record PublicExport(models_module, "User")
→ record DeclarationShell(final User) as needed
```

### Re-export

```text
consumer resolves b.PublicFoo
→ record PublicExport(b, "PublicFoo")
→ final declaration/surface dependency may also be recorded
```

This preserves both the name binding and final declaration semantics.

### Missing local/import/public name

```text
lookup fails
→ record LinkedName(...)=Absent or PublicExport(...)=Absent
```

This is the mechanism that later allows removal of whole-workspace missing-name safety fingerprints.

---

## Task 23 — Prove absence and re-export retargeting behavior

**Risk:**  
Semantic: HIGH  
Implementation fanout: tests + exact lookup product logic

**Classification:** EXACT behavior.

Required regressions:

1. **Previously missing public name appears**

```text
Consumer refers to Provider.Missing
PublicExport(Provider,"Missing") = Absent
add export Missing
exact consumer invalidates/recomputes
unrelated consumers do not
```

2. **Unrelated export added**

```text
Consumer uses Provider.Foo
add Provider.Bar
PublicExport(Provider,"Foo") product fingerprint unchanged
Consumer body revalidates without recompute
```

3. **Re-export retarget**

```text
B exports PublicFoo → A1.Foo
change B to export PublicFoo → A2.Foo
PublicExport(B,"PublicFoo") changes
consumer recomputes even if A1.Foo and A2.Foo happen to have equal-looking declaration surfaces
```

4. **Explicit local alias unaffected by upstream public rename semantics**

Plan A only proves semantic target dependence; textual rename behavior is deferred to Plan B.

---

## Task 24 — Retain coarse aggregate dependencies only where semantically aggregate

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: checker/editor query consumers

**Classification:** STRUCTURAL.

Audit every `SemanticDependency::LinkedInterface(module)` recording.

Classify each as:

```text
exact lookup → replace with exact LinkedName/PublicExport dependency
aggregate export enumeration → retain LinkedInterface dependency
unknown/dynamic reflection requiring all names → retain conservative aggregate dependency explicitly
```

Do not delete aggregate dependencies blindly.

### Required evidence

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic incremental
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic imported_resolution
```

Add exact query-level tests that inspect dependency edges for representative callables.

Negative/deletion search:

- no newly introduced `ModuleSemanticDependencies`, `ModuleChanges`, or second dependency-index type unless an approved incident explicitly justifies it.

## Checkpoint completion

- [ ] exact query/product families installed.
- [ ] exact name reads recorded.
- [ ] absence is a stable product.
- [ ] re-export retarget dependency proven.
- [ ] aggregate `LinkedInterface` dependency audited.
- [ ] no parallel semantic invalidation engine exists.
- [ ] A4 tests green.

---

# 14. Checkpoint A5 — Remove Coarse Whole-Workspace Semantic Safety Fingerprints

Tasks:
- Task 25 — remove all-source resolution hashing from callable-body direct input.
- Task 26 — remove whole-`LinkedProgram` hashing from callable-body direct input.
- Task 27 — prove exact propagation barriers under high fanout and previously-missing-name cases.

## Why this is a checkpoint

A4 adds exact dependencies while the old safety nets still protect correctness. A5 is the moment the architecture begins receiving its full performance benefit.

A5 MUST NOT begin until A4 absence and re-export retarget tests are green.

After A5:

> `CallableBody` direct input identity describes the callable's own direct non-query inputs. Cross-module/source-resolution meaning is represented by explicit semantic dependencies and product fingerprints.

## Entry conditions

- A4 COMPLETE.
- Exact absent name/import/export facts are recorded.
- Cold/incremental correctness fixtures exist for missing-name addition and re-export retarget.

## Working set

Primary:

- `phalcom-semantic/src/db/fingerprint.rs` — `callable_body_input_fingerprint_with_formal_inputs`, `source_resolution_input_fingerprint`, `semantic_component_product_fingerprint` callers.
- `phalcom-semantic/src/db/query.rs` — body query reuse/dependency validation.
- `phalcom-semantic/tests/semantic/incremental/product_stability.rs` and related incremental suites.

Secondary:

- checker dependency capture if a regression exposes one missing exact edge.

Out of scope:

- arbitrary micro-optimization of hash functions.
- TypeStore clone/COW.

## Semantic risks

- removing a conservative input before exact missing-name dependency exists;
- accidentally turning source range/provenance changes into semantic stability when current source attachments need refresh;
- hidden semantic lookup path that bypasses tracking resolver;
- tests passing only because dependent body is eagerly recomputed elsewhere.

---

## Task 25 — Remove `source_resolution_input_fingerprint(all sources)` from callable-body direct inputs

**Risk:**  
Semantic: HIGH  
Implementation fanout: fingerprint/query tests

**Classification:** EXACT target, STRUCTURAL final helper cleanup.

Current code intentionally hashes every source interface/namespace as a safety net for missing-name cases.

Once A4 exact `Absent` name facts are proven, remove that global input from ordinary callable-body fingerprinting.

Keep `source_resolution_input_fingerprint` only if another legitimate aggregate query still consumes it. Otherwise delete it and its tests/callers.

### Required negative gate

Search for callable-body input construction and prove no path reintroduces:

```text
iterate every workspace source
hash every unlinked interface
```

as a direct body input.

---

## Task 26 — Remove whole `LinkedProgram` semantic-component hash from callable-body direct inputs

**Risk:**  
Semantic: HIGH  
Implementation fanout: fingerprint/query dependencies

**Classification:** EXACT target.

Remove the entire linked-program hash from ordinary body direct input once all ordinary name/link reads are exact dependencies.

If a body operation genuinely consumes an aggregate linked fact, record an explicit aggregate `QueryKey` dependency instead of smuggling it into the input fingerprint.

### Preserve

Direct inputs may still include:

- callable identity;
- body syntax/content identity;
- stable TypeStore/workspace identity as currently required;
- owner-local field lifecycle facts until/if they become query products;
- other non-query inputs that are genuinely direct.

Do not optimize unrelated fingerprint code in this task.

---

## Task 27 — High-fanout and product-stability proof

**Risk:**  
Semantic: HIGH  
Implementation fanout: synthetic/incremental tests

**Classification:** EXACT behavior.

Required load-bearing fixtures:

### Fixture A — unused export change

```text
Provider exports Foo, Bar, Baz
Consumer uses Foo only
edit Bar semantic surface
```

Expected:

```text
Provider relevant products recompute
PublicExport(Provider,"Foo") product stable
Consumer CallableBody original computation revision retained
Consumer validated_revision advances
consumer body recomputations = 0
```

### Fixture B — 5,000 reverse importers / 100 actual consumers

Synthetic workspace where all modules import/provider-connect broadly but only 100 read the changed exact export.

Expected semantic work:

```text
cheap reverse candidates may be >100
actual body recomputations ≈ exact consumers only
unrelated bodies structurally reused
```

Do not require zero candidate inspection if the current reverse index is module-granular. The acceptance condition is that expensive semantic recomputation is proportional to actual exact consumers.

### Fixture C — stable downstream output stops second-layer propagation

```text
A change invalidates B
B recomputes but publishes same product
C depends on B product
C validates/reuses
```

### Fixture D — previously absent name appears

Proves correctness after safety-hash removal.

### Fixture E — cold/incremental equivalence

For each above source end state, compare cold and incremental semantic presentations/diagnostics/query products without comparing raw `TypeId` across independent stores.

Recommended gate:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic incremental
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic product_stability
```

## Checkpoint completion

- [ ] all-source body input hash removed.
- [ ] whole-linked-program body input hash removed.
- [ ] exact missing-name correctness retained.
- [ ] unused export change does not recompute consumer body.
- [ ] product-stability second-layer barrier proven.
- [ ] cold/incremental equivalence proven.
- [ ] no A5 incident.

---

# 15. Checkpoint A6 — Incremental Semantic Workspace Aggregation

Tasks:
- Task 28 — separate source-local structural semantic shards from global semantic realization.
- Task 29 — make declaration namespace/table composition delta-maintained.
- Task 30 — make hierarchy/supertype direct-edge products delta-maintained/query-owned.
- Task 31 — make type-alias/generic-header contributions module-sharded.
- Task 32 — publish declaration surfaces/callable/field signatures through exact query worklists instead of whole-workspace reconstruction.
- Task 33 — compose immutable snapshot semantic aggregates from retained shards with cold/incremental parity.

## Why this is a checkpoint

After A5, cross-module dependency propagation is precise, but `SemanticWorkspaceSession::update_with_budget_and_cancel()` still performs broad loops that reconstruct declaration/hierarchy/alias and related workspace state from every source.

A6 removes this remaining O(workspace) orchestration without replacing the existing query engine.

After A6:

> The semantic workspace owns retained per-module structural/semantic contributions. Changed modules replace their contributions; exact `SemanticDb` queries recompute only affected semantic products; immutable snapshots compose the current retained world.

## Entry conditions

- A5 COMPLETE.
- Exact module/name dependencies are active.
- Existing declaration/hierarchy/callable product fingerprints and query keys remain authoritative.

## Working set

Primary:

- `phalcom-semantic/src/session.rs` — current global orchestration.
- `phalcom-semantic/src/db/query.rs` — declaration/hierarchy/signature query ownership.
- `phalcom-semantic/src/db/key.rs` / `product.rs` / `fingerprint.rs` — exact query products.
- declaration, hierarchy, type-alias, signature table modules.
- `phalcom-semantic/src/snapshot.rs` — retained immutable aggregate publication.

Secondary:

- source/index builder only where structural declaration shards are naturally sourced; do not implement Plan-B reference-index deltas.

Out of scope:

- TypeStore persistent snapshot/COW redesign.
- retention/demotion tiers.
- rename/reference graph.

## Semantic risks

- removing global predeclaration visibility required for mutually recursive declarations;
- changing declaration identity between cold and incremental paths;
- failing to remove obsolete declaration/alias/hierarchy entries on source deletion or reidentification;
- incremental hierarchy cycle detection diverging from cold behavior;
- query scheduling order accidentally becoming semantic order;
- retaining stale TypeIds after declaration removal/redefinition;
- duplicating semantic tables instead of composing one authoritative retained table.

## Hostile cases

- two modules with mutually visible declarations through legal imports.
- superclass changes from A to B.
- alias target changes through imported re-export.
- generic declaration header changes while body is stable.
- module deleted and recreated with new identity.
- declaration removed then same spelling reintroduced.
- SCC of callable bodies where only one signature changes.

---

## Task 28 — Introduce retained per-module structural semantic shards

**Risk:**  
Semantic: HIGH  
Implementation fanout: semantic session/declaration extraction

**Classification:** STRUCTURAL.

Separate source-local structure that can be extracted without solving cross-module semantic meaning.

Recommended shard responsibilities:

```text
ModuleSemanticStructureShard
    declaration identities/blueprints
    callable/field source declarations
    type-alias declarations/syntax
    superclass/header syntax references
    generic binder/header syntax
    source-local diagnostic/provenance inputs
```

The shard should not duplicate `UnlinkedModuleInterface`; reuse interface/source products where they already contain the needed canonical structural fact.

### Reuse rule

If `ParsedModule`/relevant structural fingerprint is unchanged, retain the shard `Arc`.

If body-only syntax changes but declaration structure remains semantically unchanged, decide whether the shard can also remain stable via a structural fingerprint rather than rebuilding from raw source. Measure and keep the first implementation simple.

---

## Task 29 — Delta-maintain declaration namespace/table composition

**Risk:**  
Semantic: HIGH  
Implementation fanout: session/declarations/TypeStore interaction

**Classification:** STRUCTURAL.

Replace:

```text
clone base declarations
scan every source
insert every current declaration
```

with:

```text
persistent base declarations
+
module declaration contributions
```

Transaction/update algorithm:

```text
remove old contribution for changed/removed module
add new structural declarations for changed module
preserve untouched contributions
```

Maintain canonical `DeclarationId` identity exactly.

Where TypeStore forms must exist for all declarations before semantic realization, allocate/ensure only newly introduced declaration forms and reuse existing canonical forms for unchanged declarations.

Deletion must remove current table visibility and purge query identities; TypeStore interning storage may retain unreachable interned forms if current TypeStore semantics require it. Do not conflate table visibility with arena memory reclamation.

---

## Task 30 — Delta-maintain hierarchy and supertype direct-edge products

**Risk:**  
Semantic: HIGH  
Implementation fanout: hierarchy/query/session

**Classification:** STRUCTURAL.

The current DB already has `HierarchyEdge(DeclarationId)` query ownership.

Refactor global hierarchy reconstruction so direct edges are produced/validated per declaration and aggregate `MapTypeHierarchy` is updated from changed edge products.

### Required semantics

- direct edge lookup records exact linked/public-name dependencies through A4;
- unchanged hierarchy edge product retains fingerprint and downstream consumers revalidate;
- cycle detection remains correct when one edge changes;
- transitive queries consume direct edges rather than a monolithic hierarchy fingerprint where possible.

---

## Task 31 — Module-shard aliases and generic headers

**Risk:**  
Semantic: HIGH  
Implementation fanout: alias/generic resolution/session

**Classification:** STRUCTURAL.

Current alias processing globally computes dependency order/cycles across all aliases.

Refactor in two stages:

1. retain source-local alias declarations/dependency references per module;
2. maintain an exact global alias dependency graph and recompute only affected SCC/order regions when alias declarations or resolved dependencies change.

Do not assume alias graph is acyclic; cycles are diagnostics.

Generic declaration/header products should similarly be query-/declaration-owned and reused when syntax + consumed semantic dependencies are unchanged.

If full alias-SCC incrementalization is too large for one patch, preserve a bounded global SCC pass over the compact alias graph while eliminating repeated full source/AST lowering. Record its complexity and metric; do not silently leave a full-source scan.

---

## Task 32 — Query/worklist-driven declaration surfaces and signatures

**Risk:**  
Semantic: HIGH  
Implementation fanout: session/db query orchestration

**Classification:** STRUCTURAL.

Use existing query keys/products as the semantic work units:

```text
DeclarationShell
HierarchyEdge
DeclarationSurface
CallableSignature
FieldSignature
CallableBody
EnumDeclaration / AssociatedSurface where applicable
```

The session should schedule/ensure:

- products for changed structural declarations;
- products selected because dependency validation failed;
- products required for the published snapshot/open workspace policy.

It should not reconstruct every surface/signature from every source and then ask the DB whether it changed.

Preserve current recursion/fixpoint semantics where query dependencies cycle; do not impose an artificial DAG requirement.

---

## Task 33 — Compose snapshots from retained semantic shards and prove parity

**Risk:**  
Semantic: HIGH  
Implementation fanout: session/snapshot/tests

**Classification:** STRUCTURAL.

Build immutable snapshot aggregate maps from retained current module contributions and DB products.

Prefer Arc reuse for unchanged per-module products. Do not require a new persistent collection library.

### Stats to add/clarify

```text
semantic_structure_shards_recomputed
semantic_structure_shards_reused
declaration_products_recomputed/reused
hierarchy_edges_recomputed/reused
alias_regions_recomputed/reused
callable_signatures_recomputed/reused
field_signatures_recomputed/reused
callable_bodies_recomputed/reused
```

Avoid double-counting “visited” as “recomputed”.

### Required evidence

1. body-only edit in one module:

```text
one relevant source/body product recomputes
other module semantic structure shards reused
unrelated declaration/hierarchy/signature products reused
```

2. one public declaration signature change:

```text
changed declaration/signature recomputes
exact consumers selected through DB dependencies
unrelated modules/shards untouched
```

3. superclass edge edit:

```text
one direct hierarchy edge changes
transitive affected semantic consumers recompute
unrelated hierarchy edges reused
```

4. alias retarget:

```text
only affected alias SCC/region and consumers recompute
```

5. delete/reidentify module:

```text
old shard removed
query products purged
no stale declaration visibility
```

6. cold/incremental equivalence across all above.

Recommended checkpoint gate:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-modules
```

Run targeted `phalcom-core` compiler/module parity tests that consume these products, but do not require the known-red full core corpus to become green.

## Checkpoint completion

- [ ] source-local semantic structure sharded.
- [ ] declarations delta-maintained.
- [ ] hierarchy direct edges delta-maintained/query-owned.
- [ ] alias/global compact graph incrementalized or explicitly bounded.
- [ ] surfaces/signatures scheduled by exact query needs.
- [ ] snapshot aggregate parity proven.
- [ ] no A6 incident.

---

# 16. Checkpoint A7 — Performance and Release Closure for Plan A

Tasks:
- Task 34 — finalize work-count instrumentation and invariant assertions.
- Task 35 — build deterministic synthetic module/dependency benchmark fixtures.
- Task 36 — run Plan-A performance acceptance matrix.
- Task 37 — run compiler/LSP parity and current full-LSP regression gate.
- Task 38 — compare workspace-wide failures against the recorded baseline and publish the Plan-A handoff to Plan B.

## Why this is a checkpoint

A7 does not add architectural semantics. It proves that A0–A6 jointly deliver the intended update complexity without regressing existing IDE/compiler behavior.

The checkpoint is complete only when performance claims are supported by deterministic work counts, not merely wall-clock observations.

## Entry conditions

- A0–A6 COMPLETE.
- no active correctness incident.
- all new metrics have stable definitions.

## Working set

Primary:

- module and semantic stats/metrics.
- test fixtures under existing module/semantic incremental test organization.
- LSP integration tests.
- implementation-state/handoff documentation.

Secondary:

- benchmark harness if repository conventions support it.

Out of scope:

- Plan-B source/reference index performance.
- rename.
- retention tiers.
- editor overlay transactions.
- scheduler lane separation.

---

## Task 34 — Finalize deterministic work metrics

**Risk:**  
Semantic: LOW  
Implementation fanout: metrics/test support

**Classification:** EXACT behavior.

At Plan-A completion metrics must distinguish at least:

### Module/update

```text
interfaces_built
interfaces_reused
import_sites_considered
import_sites_validated
imports_resolved
import_resolutions_reused
negative_resolutions_reused
linked_components_considered
linked_components_recomputed
linked_modules_recomputed
linked_modules_reused
```

### Semantic dependency

```text
query_products_recomputed
query_products_revalidated
exact_name_products_recomputed/reused
reverse_candidates_considered (if available)
semantic_dependents_recomputed
semantic_dependents_reused
```

### Semantic workspace shards

```text
semantic_structure_shards_recomputed/reused
hierarchy_edges_recomputed/reused
alias_regions_recomputed/reused
callable_signatures_recomputed/reused
callable_bodies_recomputed/reused
```

Metrics must describe actual work, not just changed source count.

---

## Task 35 — Deterministic synthetic fixtures

**Risk:**  
Semantic: LOW  
Implementation fanout: tests/fixture helpers

**Classification:** STRUCTURAL.

Create reusable fixture builders for controlled graphs:

```text
linear chain
star/high-fanout provider
multiple disconnected components
import-heavy module
negative import population
re-export chain
hierarchy fanout
alias SCCs
```

Target sizes:

```text
1,000 modules  — required where test runtime is acceptable
10,000 modules — benchmark/ignored or dedicated perf gate if normal suite cost is too high
50,000 modules — benchmark/manual/CI perf lane, not mandatory unit suite
```

The point is deterministic work counts and asymptotic evidence, not making ordinary tests slow.

---

## Task 36 — Plan-A performance acceptance matrix

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: evidence only unless a gate fails

**Classification:** EXACT acceptance criteria.

### PA-1 — body-only edit

Expected warm work:

```text
changed source parse/body work only
interface semantic product stable
imports_resolved = 0
linked_components_recomputed = 0
cross-module semantic consumers recomputed = 0
```

### PA-2 — one of 20 import paths changes

```text
imports_resolved = 1
19 import products reused
only affected linked component recomputed
```

### PA-3 — unrelated file creation while negative import exists

```text
negative import product validated/reused
resolver not invoked for that site
```

### PA-4 — missing target appears

```text
only matching negative site(s) re-resolved
linked/semantic propagation follows changed exact products
```

### PA-5 — unused public export changes

```text
provider public surface changes
Consumer uses different export
Consumer body recomputations = 0
```

### PA-6 — high fanout exact export

```text
5,000 reverse-connected modules
~100 exact consumers of changed name
expensive semantic recomputation proportional to ~100, not 5,000
```

### PA-7 — disconnected components

```text
change component A
components B/C linked products retained
```

### PA-8 — stable intermediate product

```text
A change causes B recompute
B product fingerprint stable
C body/product reused
```

### PA-9 — declaration/hierarchy isolation

```text
one direct hierarchy/header change
unrelated semantic shards reused
```

### PA-10 — cold/incremental parity

For final identical source state, compare:

- module resolutions;
- linked public exports;
- blocked/current diagnostics;
- declaration identities/presentations;
- hierarchy relations;
- callable/field signature presentations;
- callable diagnostics/result states.

Do not compare raw `TypeId` values across independent TypeStores.

---

## Task 37 — Compiler/LSP parity and full-LSP regression gate

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: evidence

**Classification:** EXACT acceptance.

Run:

```bash
RUSTFLAGS='' cargo test -p phalcom-modules
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-lsp
```

The full LSP suite was green at the C7 recorded baseline and therefore MUST remain green.

Also run existing compiler/workspace module parity fixtures proving strict compiler and workspace resolve/link identities agree.

No LSP-specific workaround is acceptable if a Plan-A change breaks one of these tests.

---

## Task 38 — Workspace baseline comparison and Plan-B handoff

**Risk:**  
Semantic: MEDIUM  
Implementation fanout: evidence/docs

**Classification:** EXACT.

Run:

```bash
RUSTFLAGS='' cargo test --workspace --all-targets
```

Compare exact failure names against the A0 recorded baseline.

Acceptance:

```text
no new failures
no newly failing touched module/semantic/LSP lane
no existing failure hidden by weaker assertions or ignored tests
```

If some pre-existing `phalcom-core` failures become green because Plan A fixes their underlying module behavior, record that improvement. Do not require unrelated remaining baseline failures to be repaired as part of Plan A.

Publish a Plan-B handoff containing:

```text
final HEAD
all Plan-A invariants and metrics
exact retained product APIs
ImportSiteId and prefix-product APIs
exact semantic query keys/products
snapshot topology/reverse-index APIs
remaining source/reference-index limitations
known baseline failures
performance matrix results
```

## Checkpoint completion

- [ ] all Plan-A metrics finalized.
- [ ] deterministic synthetic fixtures landed.
- [ ] PA-1 through PA-10 pass at required scale.
- [ ] `phalcom-modules` green.
- [ ] `phalcom-semantic` green.
- [ ] full `phalcom-lsp` green.
- [ ] no new workspace-wide failures.
- [ ] Plan-B handoff published.
- [ ] Plan A COMPLETE.

---

# 17. Detailed Dependency/Publication Design Notes

This section constrains implementation choices that cross multiple checkpoints.

## 17.1 Do not build a second `ModuleChanges` propagation system

The tempting Pyrefly-shaped design would be:

```rust
ModuleDependencies
ModuleChanges
invalidated_by(...)
```

Phalcom does not need a second version of that architecture because it already has the more general:

```text
QueryKey
InputFingerprint
ProductFingerprint
DependencyEdge
DependencyIndex
validate_reuse
```

The Phalcom adaptation is to make module facts query-addressable.

If a candidate implementation adds a second semantic reverse-dependency index, stop and prove why the existing `DependencyIndex` cannot represent the required edge.

---

## 17.2 Fingerprint semantics

Maintain the following distinction rigorously:

```text
InputFingerprint
    direct non-query inputs to this computation

ProductFingerprint
    semantic meaning published by this computation
```

Examples:

### `ResolvedImport(site)`

Input fingerprint may include:

- import-site syntax;
- importer ownership/context identity;
- exact relevant topology input generation/facts.

Product fingerprint includes:

- success/failure semantic outcome;
- canonical target;
- prefix targets where semantically relevant.

### `PublicExport(module,name)`

Input fingerprint may be tied to current linked public-surface product/lookup input.

Product fingerprint includes:

```text
Absent
or
canonical linked export target + semantic export metadata
```

Source range movement must not change product fingerprint.

### `LinkedName(module,name)`

Product fingerprint includes semantic local-name binding meaning, not local source range.

---

## 17.3 Missing-name correctness is load-bearing

The whole-workspace hashes in current callable-body input exist partly because a failed name lookup cannot otherwise record a dependency on something that does not exist.

Therefore A4 must represent absence explicitly.

Incorrect:

```text
lookup Foo → None
record no dependency
```

Correct:

```text
lookup Foo
→ query LinkedName/PublicExport key
→ product = Absent
→ body records dependency on Absent product fingerprint
```

Then later:

```text
Absent → Present(target)
```

changes the product fingerprint and invalidates the exact consumer.

This is the precondition for A5.

---

## 17.4 Re-export identity is the public name, not only the final declaration

For:

```phalcom
// a.ph
export Foo

// b.ph
export Foo as PublicFoo from .a

// c.ph
from .b import PublicFoo
```

The semantic dependency chain must preserve:

```text
c local linked name PublicFoo
    ↓
PublicExport(b,"PublicFoo")
    ↓
upstream target a::Foo
    ↓
DeclarationShell/Surface(a::Foo)
```

If B retargets `PublicFoo` to a different upstream declaration with a coincidentally identical surface, the public-export product still changes because canonical target identity changed.

Plan B later adds textual/name rewrite identity; Plan A only establishes semantic binding precision.

---

# 18. Deletion and Migration Gates

The implementation is incomplete if new architecture is added while old coarse mechanisms remain authoritative.

## A2 deletion gates

By A2 completion:

- retained import product authority is no longer `(ModuleId, String)`;
- module/path string maps are compatibility projections only, if still present;
- failed resolution dependency sets are not empty by default;
- broad module-level “resolve all imports because interface changed” logic is removed.

## A3 deletion gates

By A3 completion:

- session no longer loops over/link-recomputes every disconnected component after any non-body change;
- graph merge no longer silently defaults invalid runtime order to empty.

## A4/A5 deletion gates

By A5 completion:

- ordinary exact name lookup does not depend on `LinkedInterface(module)` solely because exact product identity is unavailable;
- callable body direct input no longer hashes all source-resolution state;
- callable body direct input no longer hashes the entire `LinkedProgram`;
- no parallel module semantic dependency engine exists.

## A6 deletion gates

By A6 completion:

- `SemanticWorkspaceSession::update_with_budget_and_cancel` does not rebuild declaration namespace/hierarchy/alias semantic inputs by scanning all sources on every ordinary edit;
- removed module semantic contributions are explicitly deleted/purged;
- unchanged module semantic shards are reused rather than reconstructed.

---

# 19. Tempting Wrong Fixes

## Wrong fix 1 — “C2 documentation says transactions are atomic, so skip A0”

Repository source is authoritative. Current `apply_batch` still mutates substantial live state before final rebuild success. Fix the source architecture, not the documentation narrative.

## Wrong fix 2 — clone the entire session for every edit

This would provide atomicity while preserving O(workspace) edit cost. It violates the performance purpose of Plan A.

## Wrong fix 3 — keep stale interface as current because it improves IDE continuity

Last-known-good and current truth are distinct products. Current invalid source must publish partial/blocked state, not stale semantic success.

## Wrong fix 4 — invalidate all resolver caches on any filesystem event and rely on fast re-resolution

A topology event is only a candidate invalidation. Retained resolution dependencies must prove which import sites actually need resolver work.

## Wrong fix 5 — put linked reads/runtime dependencies into the public linked-interface fingerprint

That would make private implementation changes look like public API changes and increase downstream invalidation. Add a separate private linked-dependency product.

## Wrong fix 6 — copy Pyrefly `ModuleDeps` beside `SemanticDb`

Phalcom already has product-fingerprint dependency validation. Extend the query-product vocabulary instead.

## Wrong fix 7 — retain `LinkedInterface(module)` dependency in addition to every exact export/name dependency “for safety”

That defeats the precision gain. Keep aggregate dependency only for genuinely aggregate semantic operations.

## Wrong fix 8 — remove the whole-workspace callable safety hashes before absence is query-addressable

That creates stale unresolved-name bugs. A4 must be complete before A5.

## Wrong fix 9 — optimize with threads before eliminating unnecessary work

Parallel relinking/rechecking of irrelevant modules is still irrelevant work. Plan A is about work avoidance first.

## Wrong fix 10 — solve A6 by creating a second declaration/type universe per module

There remains one semantic workspace TypeStore and one canonical declaration identity system. Shards are retained contributions, not independent semantic universes.

---

# 20. Testing and Evidence Policy

Testing is checkpoint-driven, not task-ritual-driven.

## 20.1 Task-local loop

For compile-heavy edits:

```bash
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
```

Run exact focused tests while developing high-risk behavior.

Do not run full workspace after every task.

## 20.2 Checkpoint gates

### A0

Module session/linker hostile correctness only.

### A1

Topology/query/snapshot indexed behavior.

### A2

Full `phalcom-modules` plus exact work-count fixtures.

### A3

Full `phalcom-modules` linking/session suite.

### A4

Semantic DB/dependency/imported-resolution suites.

### A5

Semantic incremental/product-stability suites.

### A6

Full `phalcom-semantic` + `phalcom-modules`, plus touched compiler parity tests.

### A7

Full affected crates, full LSP, then workspace baseline comparison.

## 20.3 Time-based benchmarks

Wall-clock measurements are secondary evidence.

Primary acceptance uses:

```text
number of resolver calls
number of linked components
number of semantic products recomputed
number of shards replaced
```

Only after deterministic work counts are correct should A7 record p50/p95 or RSS if useful.

---

# 21. Performance Complexity Targets

These are architectural targets, not promises of exact constant factors.

## Warm body-only edit

Target work:

```text
O(changed source parse/body semantic work)
```

No import resolution, component linking, or unrelated module semantic recomputation.

## One import-site edit

Target work:

```text
O(changed import site validation/resolution
  + affected component link
  + exact semantic consumers of changed linked facts)
```

not O(all imports in importer + workspace link).

## Topology event

Target work:

```text
O(affected directory/topology facts
  + import sites whose recorded dependencies intersect those facts
  + semantic consequences)
```

## Public export change

Target semantic recomputation:

```text
O(consumers of changed exact export products)
```

with cheap module-level reverse candidate inspection permitted where current index granularity requires it.

## Semantic workspace aggregation

Target:

```text
O(changed module structural shards
  + query products whose dependencies changed)
```

not O(total source modules) for ordinary edits.

---

# 22. Plan-A Final Architecture State

When Plan A is complete, Phalcom's compiler/LSP module substrate should satisfy this pipeline:

```text
                     ┌─────────────────────────┐
                     │ committed module world  │
                     └────────────┬────────────┘
                                  │
                      private COW transaction
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
        changed sources       topology delta      overlays/ownership
              │                   │                   │
              └───────────────┬───┴───────────────────┘
                              │
                   current interfaces/products
                              │
                    ImportSite validation
                              │
            ┌─────────────────┴─────────────────┐
            │                                   │
         reuse site                         resolve site
            │                                   │
            └─────────────────┬─────────────────┘
                              │
                   affected linked components
                              │
             public surface + private link facts
                              │
                exact semantic module products
          ResolvedImport / LinkedName / PublicExport
                              │
                    existing SemanticDb
              InputFingerprint / ProductFingerprint
                   DependencyEdge validation
                              │
                  exact affected consumers
                              │
                 retained semantic shards
                              │
                 immutable SemanticSnapshot
                              │
                   compiler + LSP consumers
```

The most important resulting property is:

> A provider/module change is not itself a reason to recompute a consumer. The consumer recomputes only when one of the exact semantic products it observed changes meaning.

---

# 23. Explicit Plan-B Boundary

Plan A MUST stop after semantic/module incrementality and performance closure.

The following are deliberately deferred to the companion implementation plan:

```text
incremental global SourceSemanticIndex contribution replacement
separate direct definition index
local-vs-cross-module reference index split
semantic-vs-textual rewrite references
export/re-export public name rewrite identity
prepare_rename / workspace rename
rename conflict checking
rich definition-vs-declaration-vs-origin navigation API
retention tiers (Interface/Diagnostic/Indexed/Full)
TypeStore structural sharing/COW unless A7 proves it is blocking
noncommittable editor overlay transactions
interactive/background/heavy scheduling lanes
open-file prioritization
large-workspace reference/rename latency closure
```

Plan B consumes the exact module/import/name products and stable update behavior established here. It must not compensate for missing Plan-A precision with LSP-owned inference or scanning.

---

# 24. Final Completion Checklist

## Architecture

- [ ] `WorkspaceModuleSession` commits atomically after all fallible derivation.
- [ ] stale prior interfaces/links cannot masquerade as current products.
- [ ] transitive source errors publish current partial state.
- [ ] `ModuleTopology` and reverse import indexes are production snapshot products.
- [ ] stable `ImportSiteId` is canonical retained import identity.
- [ ] positive and negative resolution dependencies are recorded.
- [ ] import sites validate before resolver invocation.
- [ ] compound import prefixes retain canonical target provenance.
- [ ] filesystem directory topology is cached/incrementally invalidated.
- [ ] linking recomputes affected components only.
- [ ] public linked surface and private linked dependency fingerprints are separate.
- [ ] component graph contributions are retained/delta-replaced.
- [ ] exact import/local-name/public-export facts are `SemanticDb` query products.
- [ ] absence is a first-class exact semantic product.
- [ ] ordinary exact lookup records exact dependencies.
- [ ] no second semantic dependency engine exists.
- [ ] callable-body input no longer hashes all sources or the whole linked program.
- [ ] declaration/hierarchy/alias/signature workspace state is module-sharded/incremental.

## Correctness evidence

- [ ] A0 hostile transaction/currentness regressions pass.
- [ ] A2 positive/negative resolution tests pass.
- [ ] A3 disconnected-component/link graph tests pass.
- [ ] A4 exact name/absence/re-export dependency tests pass.
- [ ] A5 product-stability and missing-name correctness pass without safety hashes.
- [ ] A6 cold/incremental semantic parity passes.

## Performance evidence

- [ ] body-only edit resolves zero imports and links zero components.
- [ ] 1-of-20 import edit resolves one site.
- [ ] unrelated topology event reuses unaffected negative resolution.
- [ ] unused export edit recomputes zero exact consumers.
- [ ] high-fanout semantic recomputation follows exact consumers, not all importers.
- [ ] unaffected disconnected linked components are reused.
- [ ] unchanged semantic module shards are reused.

## Release evidence

- [ ] `phalcom-modules` suite green.
- [ ] `phalcom-semantic` suite green.
- [ ] full `phalcom-lsp` suite green.
- [ ] workspace-wide run introduces no new failures relative to A0 baseline.
- [ ] Plan-B handoff records exact final APIs, metrics, and remaining work.

---

# 25. Recommended Commit Granularity

Keep commits bisectable by semantic sub-boundary rather than one commit per tiny edit.

Suggested pattern:

```text
A0
  test(modules): expose late workspace-transaction failure regression
  refactor(modules): stage workspace mutations behind commit barrier
  fix(modules): publish current invalid interfaces as blocked products
  fix(modules): make transitive source failures tolerant
  fix(modules): reconcile dependency edges and survivor runtime order

A1
  feat(modules): retain canonical workspace topology product
  feat(semantic): publish topology and reverse imports in snapshots
  perf(modules): remove production module-query scans

A2
  feat(modules): add stable import-site identities
  feat(modules): retain prefix-aware import resolution products
  feat(modules): capture negative topology dependencies
  perf(modules): validate import products before re-resolution
  perf(modules): cache directory topology snapshots

A3
  feat(modules): retain linked component products
  perf(modules): relink affected components only
  refactor(modules): split public and private linked fingerprints
  fix(modules): delta-merge module graph contributions

A4
  feat(semantic): add exact resolved-import and name/export query products
  feat(semantic): record exact module-name reads
  test(semantic): prove absence and re-export retarget invalidation

A5
  perf(semantic): remove workspace source-resolution body fingerprint
  perf(semantic): remove full linked-program body fingerprint
  test(semantic): prove load-bearing product stability

A6
  refactor(semantic): retain module semantic structure shards
  perf(semantic): delta-maintain declaration and hierarchy state
  perf(semantic): incrementalize alias/header products
  perf(semantic): drive surfaces and signatures through query worklists

A7
  test(perf): add deterministic module incrementality fixtures
  chore(architecture): record Plan-A closure and Plan-B handoff
```

Do not mix Plan-B rename/reference/retention work into these commits.

---

# 26. Execution Principle

The implementing agent should repeatedly ask one question when deciding whether work is necessary:

```text
What exact product changed meaning,
and which recorded consumer observed that product?
```

If the only answer is:

```text
"the workspace changed"
```

or:

```text
"the provider module changed"
```

then the implementation is still too coarse unless the operation is intentionally aggregate.

That is the architectural standard Plan A is intended to establish.
