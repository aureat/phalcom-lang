# PERF003.C2.P2 — Complete Canonical Universe Formal Image and Immutable Canonical Query Roots

**Artifact type:** Patch-grade implementation plan and architect/implementer handoff  
**Program:** `PERF003` — Compilation Performance and Efficiency  
**Plan ID:** `PERF003.C2.P2` (L0 approved by user on 2026-10-13; prior Package B is P7)  
**Status:** `IN_PROGRESS / PARTIAL / UNVERIFIED` for the new P2 scope; upstream C2 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`  
**Branch under discussion:** `perf/universe-characterization`  
**Latest commits reported by implementer:** `bf1a69c6` (formal image, query roots, exact witness lookup), `c931faea` (docs/lifecycle); clean and synchronized with remote  
**Evidence limitations:** This document is grounded in the supplied checkpoint, the corrected architecture investigation, the proposed unified implementation specification, and the supplied prior audit. It is **not** a fresh checkout review or a successful test report. Exact current signatures and line numbers must be checked against HEAD before edits.

---

## 1. Mandate and outcome

Implement the two interdependent, but separately reviewable, foundations:

1. **Complete Canonical Universe Formal Image:** create and process-share a finished, immutable `CanonicalUniverseFormalImage` that preserves the fully committed Universe `SemanticSnapshot`, final type/static-value lineage, the canonical module dependencies, and provenance. Build it by reusing the ordinary semantic analyzer; do not create a second canonical semantic implementation, and never confuse it with the partial `CanonicalUniverseSemanticBaseline`.
2. **Immutable Canonical Query Roots:** export eligible *committed* canonical query products from the completing `SemanticDb` into a stable read-only `CanonicalSemanticQueryImage`, including world identity, product/input fingerprints, and immutable dependency observations. Provide a sound path for later local `SemanticDb` read-through without copying or mutating the underlying canonical query state.

**Definition of success:** A cold canonical world is built exactly once per compatible world identity; every subsequent consumer can obtain the same finalized semantic and eligible query objects without rerunning canonical formal analysis; query roots never carry mutable session state, incomplete computation, foreign workspace identity, or invalid arena lineage. The existing canonical compiler can consume the completed image without triggering a second canonical formal pass, subject to differential semantic/runtime parity.

**Boundaries:** This plan prepares a canonical-aware workspace query seam and may add controlled read-through where independently provable. It does **not** change one-shot compilation to an image-backed ephemeral overlay, remove 40 injected Universe source modules, rewrite `WorkspaceModuleSession`/`ModuleLinker`, replace all aggregate tables, rework conformance fixed points, implement portable `.telpkg`, or change VM startup. Those are separately gated successors.

## 2. Lifecycle collision: required explicit decision before implementation

The **current attached `CHECKPOINT.md` labels `PERF003.C2.P2` as “Consumer-Specific Publication & Demand-Driven Products,” Package B**, with `IN_PROGRESS / PARTIAL / UNVERIFIED` status. The same checkpoint labels `P1` “Canonical Universe Compilation Image & One-Shot Overlay.” The **proposed** unified architecture instead orders `P2` module image, `P3` formal image, `P4` query overlay and `P7` publication. The user explicitly names *this* formal-image/query-root work `P2`.

These identifiers conflict; **do not silently overwrite, delete, or claim completion of the existing Package B P2 work**. Before source work, submit the following narrow lifecycle amendment for adoption:

- Reserve `PERF003.C2.P2` for the **new** “Complete Canonical Universe Formal Image and Immutable Canonical Query Roots” plan at the user's direction.
- Move the previously checkpointed publication package to an unambiguous replacement identifier, **proposed `PERF003.C2.P7`** consistent with the unified ordering, but only after checking all existing plan documents and cross-references. If P7 is already occupied, select another available ID and update the complete ledger atomically.
- Record that the new P2 *depends on canonical module products*, regardless of the current active P1 umbrella. The canonical module-image dependency may remain a prerequisite delivered by P1 or may be split into its own accepted subtask; do not falsely claim it exists because `canonical_image` was declared.
- Preserve already landed publication work and its test evidence under a distinct tracked scope. Update `active_plan`, `next_plan`, revision/evidence fields, the program/status/guidance documents, and plan links consistently. The top-level C2 checkpoint remains `IN_PROGRESS / PARTIAL / UNVERIFIED` until corresponding acceptance gates pass.
- The unified artifact document is still **proposed** in the current checkpoint. Adopt **only the specific architectural amendments** required for this P2 scope and record the decision; do not represent the entire document as accepted by implication.

**Blocking gate L0:** The ledger has a unique identifier for every work package; no prior P2 results or acceptance evidence are lost; the current implementation head is recorded separately from the latest *tested* head.

## 3. Evidence baseline: source findings, measurements and uncertainty

### 3.1 Established observations from supplied records

| Observation | Evidence class | Consequence for P2 |
|---|---|---|
| Historical inline fixtures have an approximately 45–47-second size-insensitive Universe-dominated floor | PERF003.C1 characterization quoted in checkpoint/investigation | Optimize repeated Universe semantic lifetime, not fixture parser speed |
| `CanonicalUniverseProgram` already invokes ordinary complete Universe analysis before core projection | Source-confirmed earlier audit | Capture that same *finished* semantic publication and eliminate redundant analysis |
| `CanonicalUniverseSemanticBaseline` is partial: declarations/signatures/etc., not all final trait, conformance, witness, body and query products | Corrected architecture/unified spec | Do not reuse baseline as if it were the final image |
| `TypeStore::fork_from_baseline()` preserves canonical numeric prefix but creates new `TypeStoreId` | Checkpoint / prior repository audit | Freeze **final**, not preliminary, prefix and verify cross-store compatibility |
| `SemanticWorkspaceSession` uses `SemanticDb` undo-journal transaction | Checkpoint | Freeze only after successful commit and do not persist transaction state |
| `a7b75980` declares/tests `phalcom_semantic::canonical_image`, but `phalcom-semantic/src/canonical_image.rs` is missing | Latest checkpoint | First compile/build gate is currently expected to fail or is at least not established |
| Prior verified snapshot: total 10.571 s, semantic update 10.441 s; 3,896 final callable passes / 3.618 s | Prior measured checkpoint snapshot; **not** current commit | P2 must report canonical pass counts; local fixed-point optimization is out of scope |
| Prior verified publication reduced source index 41 modules/603.428 ms → 1 module/0.364 ms, advisory 280.829 ms → 0.750 µs | Prior measured snapshot, noncontrolled comparability | Preserve consumer-specific behavior; do not conflate these wins with untested image work |
| New publication/filtering changes from `a7b75980` have not been tested | Latest checkpoint | Tests from `99b84ecc` cannot certify HEAD |

### 3.2 Source touchpoints verified by *prior* audit, to reconfirm at HEAD

| Crate/file | Symbols or responsibilities | Planned use |
|---|---|---|
| `phalcom-semantic/src/lib.rs` | Public module declaration `canonical_image` | Restore missing module, export intended API |
| **NEW** `phalcom-semantic/src/canonical_image.rs` | Image ownership, builder/accessors/caching (proposed) | Main P2 file |
| `phalcom-semantic/src/baseline.rs` or actual baseline owner (**locate**) | `CanonicalUniverseSemanticBaseline` | Bootstrap input only |
| `phalcom-semantic/src/session.rs` | `SemanticWorkspaceSession`, `analyze_workspace`, update/commit, current session constructors | Safe bootstrap path, capture committed snapshot and query state |
| `phalcom-semantic/src/snapshot.rs` | `SemanticSnapshot`, `new_with_callable_analyses` | Authoritative complete snapshot; no duplicate semantic products |
| `phalcom-semantic/src/db/mod.rs` | `SemanticDb`, local transaction and query stores | Freeze hook and possible origin-aware API |
| `phalcom-semantic/src/db/query.rs` | Query state, product fingerprints and currentness (verify exact owners) | Pure committed canonical query extraction and read-through contract |
| `phalcom-semantic/src/workspace.rs` | `SemanticWorkspaceInput` and precomputed interfaces | Full canonical bootstrap input, avoid unnecessary reconstruction |
| `phalcom-modules/src/builtin_interface.rs` | `UniverseSourceDeclarationCatalog`, `BuiltinInterfaceBuilder` | Existing canonical module/interface provenance |
| `phalcom-modules/src/resolver.rs` | `ModuleResolver::load_interface` | Existing cache; no unnecessary provider sharing |
| `phalcom-modules/src/session.rs`, `linker.rs`, `topology.rs` | `WorkspaceModuleSession`, linker/topology | Prerequisites for later external-boundary cutover; **not** rewrite in P2 |
| `phalcom-core/src/modules/canonical_universe.rs` | `build_canonical_universe_program`, `CanonicalUniverseProgram` | Existing complete analysis and parity reference; consume image without second analysis |
| `phalcom-core/src/modules/compile.rs`, `compile_session.rs` | `ProgramCompiler`, `ProgramCompilationSession` | Compile/parity harness only; one-shot cutover excluded |
| `telos-observability` and existing performance fixture | Compiler spans/cardinality | Cold/warm gates and provenance counters |

**Verification-first rule:** Actual module paths, concrete constructors, query-key variants, query product ownership and transactional commit signatures may have changed after the audited revision. Use symbol search at the current clean HEAD and amend this file if needed. Proposed Rust snippets below describe contracts, not guaranteed drop-in patches.

## 4. Architecture decisions (normative for P2)

**D1 — image is final, not the partial foundation.** Build the canonical semantic world through the ordinary complete Universe analysis; freeze the successful terminal state.

**D2 — strict layering.** Module graph/image data belongs to `phalcom-modules`; completed formal semantics and query roots belong to `phalcom-semantic`; executable/runtime products belong to `phalcom-core`. A semantic image may contain `Arc` references to module products, not own new mutable linker implementations.

**D3 — snapshot is authoritative.** Keep a single complete `Arc<SemanticSnapshot>` rather than mirroring declarations, hierarchy, witness, conformance and callable maps as 20+ separately owned fields. Add accessors only where justified.

**D4 — no bootstrap recursion.** The foundation constructor is explicitly independent of the completed image cache; ordinary image-backed session creation is a separate entry path.

**D5 — immutable query image, not frozen mutable DB.** Export only stable committed products and dependencies; never scheduler, undo journal, revision transaction, mutable reverse graph, or metrics.

**D6 — distinct authority/lifetime domains.** `last_snapshot` and `last_known_good` are **workspace-local publication state**, never aliases for the canonical snapshot. P2 must not install canonical state in these fields.

**D7 — preserve type/static lineage.** Freeze final canonical type prefix and compatible static-value root; reject an incompatible consumer instead of reinterpretation.

**D8 — closure and negative-fact soundness.** A `Ready` row alone does not establish global reusability. Workspace-dependent proofs, negative candidate-set facts, and mutable-origin dependencies must not be frozen as unconditional canonical truths.

**D9 — cache per semantic-world identity.** The builtin provider can use process-shared initialization only when identity and error determinism are guaranteed. Override/test providers receive private worlds.

**D10 — no eager consumer cutover.** First prove construction and export without changing mutable-workspace behavior, then introduce bounded core-image consumption, then optional query read-through only if safely tested.

## 5. Intended dependency graph and construction pipeline

```text
Canonical Universe source/provider
      │
      ▼
Canonical module products / CanonicalUniverseModuleImage [P1 dependency]
      │
      ▼
existing CanonicalUniverseSemanticBaseline (partial foundation)
      │
      ▼
explicit SemanticWorkspaceSession::from_foundation(...) [bootstrap only]
      │
      ▼
ordinary FULL canonical semantic update using complete canonical linked source
      │
      ▼
transaction successfully committed
      │
      ├── final SemanticSnapshot
      ├── final TypeStore + StaticValueStore lineage
      └── committed SemanticDb query products / fingerprints / dependencies
      │
      ▼
validate semantic completeness and query eligibility
      │
      ▼
CanonicalUniverseFormalImage
      ├── reference_snapshot: Arc<SemanticSnapshot>
      ├── query_image: Arc<CanonicalSemanticQueryImage>
      ├── modules: Arc<...>
      └── world + prefix identities
      │
      ├── phalcom-core canonical compiler consumes snapshot (P2 gate)
      └── future local workspace: immutable parent (subsequent plan)
```

**Absolutely forbidden:**

```text
image() → analyze_workspace() → SemanticWorkspaceSession::new()
        → image() → ...

image.snapshot → SemanticWorkspaceSession.last_snapshot

SemanticDb::clone() → globally cached mutable canonical DB

ready query row → automatically reusable without world/dependency proof
```

## 6. API and data-design proposal

### 6.1 Image identity and canonicality

Introduce an opaque `CanonicalSemanticWorldId`, with provenance sufficient to distinguish differing semantic configurations and providers:

```rust
// Proposed shape. Prefer an opaque digest/newtype in production.
pub struct CanonicalSemanticWorldId(/* canonical module graph + semantic ABI digest */);
pub struct CanonicalTypePrefixId(/* final prefix digest/identity */);
```

The identity calculation must incorporate all semantically influential canonical inputs: provider/Universe source set, canonical module/interface fingerprints, language/semantic feature configuration, intrinsic/native signature ABI and any compiler-versioned semantic compatibility contract. Derive it deterministically from the authoritative module-image provenance. No mutable pointer identity, a generic `ProjectIdentity::Universe`, or blanket `is_canonical` flag is sufficient. Separate *in-memory world identity* from any future on-disk artifact digest.

### 6.2 Formal image

```rust
// PROPOSED — adjust accessors/types after inspecting HEAD.
pub struct CanonicalUniverseFormalImage {
    world: CanonicalSemanticWorldId,
    modules: Arc<CanonicalUniverseModuleImage>,
    reference_snapshot: Arc<SemanticSnapshot>,
    queries: Arc<CanonicalSemanticQueryImage>,
    type_prefix: CanonicalTypePrefixId,
    // Either preserve static root in reference_snapshot, or explicit Arc/root.
}

impl CanonicalUniverseFormalImage {
    pub fn world(&self) -> &CanonicalSemanticWorldId;
    pub fn snapshot(&self) -> &Arc<SemanticSnapshot>;
    pub fn queries(&self) -> &Arc<CanonicalSemanticQueryImage>;
    pub fn type_prefix(&self) -> &CanonicalTypePrefixId;
}
```

If a complete `CanonicalUniverseModuleImage` does not yet exist, **do not put a placeholder/fictional type into compiling source**. Instead deliver a small immutable `CanonicalModuleProducts` adapter backed by actual module products, explicitly record the prerequisite and migrate when the module image is available. There must be one authoritative module product identity; avoid reconstructing all 148 interfaces as an image-building side effect.

### 6.3 Builder and bootstrap split

```rust
// One-shot bootstrap-only internal entry, not default session creation.
fn analyze_canonical_universe_from_foundation(
    modules: Arc<CanonicalUniverseModuleImage>,
    foundation: Arc<CanonicalUniverseSemanticBaseline>,
) -> Result<CommittedCanonicalSemanticPublication, CanonicalImageError>;

struct CommittedCanonicalSemanticPublication {
    snapshot: Arc<SemanticSnapshot>,
    // Purpose-built, read-only export view of committed query rows;
    // not an Arc<SemanticDb> stored into the image.
    query_export: CommittedQueryExport,
}

fn freeze_canonical_formal_image(
    publication: CommittedCanonicalSemanticPublication,
    modules: Arc<CanonicalUniverseModuleImage>,
) -> Result<CanonicalUniverseFormalImage, CanonicalImageError>;
```

An actual `CommittedQueryExport` may be a borrowed iterator consumed *immediately after commit* and before the session is dropped, rather than an owned type. Keep query-export implementation within semantic crate visibility. The builder should return structured errors rather than panic for expected compilation failure; use specific recovery/cancellation policy.

### 6.4 Public initialization API

```rust
pub fn canonical_universe_formal_image(
    provider: &UniverseSourceProvider,
) -> Result<Arc<CanonicalUniverseFormalImage>, CanonicalImageError>;
```

For production canonical provider: `OnceLock<Result<Arc<_>, _>>` may be used if error cloning, determinism and world identity are verified. A private path is compulsory for overridden providers. If initialization failure depends on cancellation/temporary resource state, **do not poison the process-wide singleton with that transient error**. Use an appropriate retry-capable state machine or establish that these errors are impossible/handled outside initialization.

### 6.5 Query image representation

```rust
pub struct CanonicalSemanticQueryImage {
    world: CanonicalSemanticWorldId,
    entries: Arc<CanonicalQueryMap>,
}

pub struct CanonicalQueryEntry {
    pub input_fingerprint: InputFingerprint,
    pub product_fingerprint: ProductFingerprint,
    pub product: Arc<SemanticProduct>,
    pub dependencies: Arc<[ImmutableQueryDependency]>,
}

pub enum QueryFreezeRejection {
    Computing,
    Cancelled,
    BudgetExceeded,
    BlockedRequiresClassification,
    FailedRequiresClassification,
    WorkspaceOrigin,
    MutableDependency,
    ForeignSemanticWorld,
    IncompatibleTypePrefix,
    IncompatibleStaticValueStore,
    NegativeLocalCandidateSet,
    NonrelocatableReference,
    UnsupportedQueryKind,
}
```

These are conceptual. The concrete query table may have heterogeneous typed products; **do not erase typing into `Any` or dynamic downcasts merely to fit this sketch**. Retain the existing typed `SemanticDb` product architecture through an enum, per-query-family tables, or type-preserving snapshot/export abstraction. The freeze protocol must expose ineligibility and prove retained objects have no mutable session ownership.

## 7. Formal completeness requirements

The final image must preserve the same semantics as the full canonical source-backed compilation, not merely pass `Arc::ptr_eq`. Inventory and audit at least:

| Family | Required facts | Verification requirement |
|---|---|---|
| Declarations | Module/type/field/callable/trait/data/enum and associated declarations | Count and stable identity equality against completed reference |
| Type/kind | Final `TypeStore` prefix, type parameters, kinds, aliases, normalized canonical refs | Prefix/lineage equality |
| Signatures | Constructors, fields, static/member callables, inferred results | No missing or provisional signatures |
| Hierarchy and dispatch | Inheritance edges, effective member surfaces, method/selector dispatch | Candidate parity |
| Trait surfaces | Header/requirements/defaults/associated members | Exact requirement and associated semantics |
| Conformance | Explicit and derived rules, coherence evidence, witness plans and trait dispatch | Witness/overlap parity, no canonical rescanning |
| Generic proof | Closed canonical proofs and reusable generic rule descriptions | Exclude local-dependent proof conclusions |
| Algebraic products | Enum/data constructors, variants, representation constraints | Product parity |
| Static semantics | Static definitions/domains/constants/value refs | Store identity and content compatibility |
| Callable analyses | Completed body-analysis results needed by downstream consumers | Final analyses, not initial passes |
| Diagnostics | Canonical diagnostics and tolerated failure status | Error/diagnostic equivalence |
| Query products | Committed ready, eligible products; fingerprints and closed dependencies | Census + export rejection accounting |
| Presentation/advisory | Where already frozen/needed, explicit requested/omitted state | Do not mandate whole-Universe editor products on runtime path |

If any semantic product necessary for local type checking is absent, report it as a **blocking completeness gap** rather than silently recomputing it and calling the image complete. Allow a compatibility fallback only if separately instrumented and accepted as a temporary explicit debt item.

## 8. Arena provenance and static-value invariants

**TypeStore.** Freeze the **final** store obtained after ordinary full analysis. The partial bootstrap store is insufficient. `fork_from_baseline()` may preserve raw canonical numeric `TypeId`, `TypeParameterId`, `KindId` while returning a fresh store instance ID. Establish a typed prefix contract for all cross-image raw IDs. A workspace using the image must extend that same prefix, never reinterpret indices from an unrelated store. Validate mismatch intentionally with tests.

**StaticValueStore.** `StaticValueRef` carries `StaticValueStoreId`. The compatible snapshot/static root must be shared/cloned following established static-store lineage rules; a fresh unrelated `StaticValueStore::new()` cannot be substituted when reading image products. Check static values nested in generic evidence, canonical signatures and witness products, not only top-level constants.

**Multi-world isolation.** For concurrent worlds/providers, a product is valid iff its world, type prefix and static lineage are compatible. Reject cross-world imports using structured errors, rather than relying on incidental ID equality.

**P2 gate A:** Initial image has the complete finalized canonical arena and static environment. Foreign prefix and foreign static-store test fixtures are rejected, and existing canonical reference products retain meaning in a compatible fork.

## 9. Freeze protocol: from committed DB to immutable query roots

### 9.1 Transaction boundary

The sequence is strict:

```text
open canonical semantic transaction
  → run ordinary full Universe semantic update
  → finalize callable/conformance/static/query products
  → commit transaction successfully
  → obtain authoritative final snapshot
  → export immutable committed query products
  → validate query/product/dependency closure
  → construct Arc-owned query image
  → construct Arc-owned formal image
  → publish process-shared result
```

There may be multiple nested transaction scopes. Confirm the actual `SemanticDb` commit point and that the snapshot and query export correspond to the same successful semantic publication. Do not freeze before commit and trust a later commit to succeed; do not publish a partial image on error.

### 9.2 Query census, classification and retention

For every query entry in the committed source DB, classify:

- Query kind and expected product type;
- Terminal state (`Ready`, `Blocked`, `Failed`, `Cancelled`, etc.);
- Canonical vs workspace origin;
- Input/product fingerprint availability;
- Input/world/type/static lineage compatibility;
- Dependency edges and observed candidate-set/negative facts;
- Any mutable session, source revision, journal or allocator reference;
- Freeze outcome and explicit rejection reason.

**Hard rule:** `Computing`, `Cancelled` and `BudgetExceeded` are never exported as ready; `Blocked`/`Failed` must be interpreted against the existing tolerated diagnostic contract, not indiscriminately discarded or published as a successful query. A ready result with a foreign/local dependency is not eligible for canonical-global retention.

### 9.3 Dependency closure and soundness

A query may depend on another canonical query, canonical input, or a local/external/environmental fact. A globally retained canonical product requires closed canonical dependencies, or an explicit parameterized representation whose free dependencies are subsequently checked by the local consumer. Reject cycle/closure violations unless the existing semantic engine defines their valid committed representation. Never treat a negative search result (e.g., no local conformance candidate) as an eternal canonical-world fact.

**Conformance rule:** Canonical explicit conformance heads and canonical derived rules are immutable canonical facts; an application of those rules to a local type/evidence context is workspace-specific. Reuse only complete closed canonical proofs. Do not add local-first precedence or alter coherence; the logical candidate set remains `canonical explicit ∪ canonical derived ∪ local explicit ∪ local derived`.

### 9.4 Do not snapshot `SemanticDb`

Exclude mutable:

```text
workspace revision, current transaction, undo journal,
scheduler/in-flight worklist, query execution state,
mutable reverse invalidation graph, revision validation sets,
mutable recomputation metrics, local last-known-good state,
source-input lifecycle fingerprints.
```

Keep products as immutable `Arc` or persistent-value roots and read-only maps. An export census must demonstrate which bytes/products are actually retained rather than merely counting source DB rows.

**P2 gate B:** A successful full canonical semantic update yields an immutable query image with independently inspectable count-by-kind and reject-by-reason summaries; zero partial/in-flight products are represented as valid ready entries; retention and dependency-closure tests pass.

## 10. Query read protocol and future workspace overlay seam

P2 should establish the immutable canonical query **read protocol**, while avoiding an unreviewed wholesale switch of mutable workspace semantics. Where a narrow read-through can be validated, gate it explicitly. Proposed interface:

```rust
pub enum SemanticProductOrigin {
    Canonical(CanonicalSemanticWorldId),
    Workspace(WorkspaceId),
}

impl SemanticDb {
    pub fn product_origin(&self, key: &QueryKey) -> Option<SemanticProductOrigin>;
    pub fn product(&self, key: &QueryKey) -> Option<&Arc<SemanticProduct>>;
    pub fn ready_product_fingerprint(&self, key: &QueryKey) -> Option<ProductFingerprint>;
    pub fn is_current(&self, key: &QueryKey) -> bool;
    pub fn local_query_state(&self, key: &QueryKey) -> Option<&QueryState>;
}
```

**Critical invariant:** `query_state()` / `local_query_state()` represent mutable workspace query state only. Existing callers use checks such as `if db.query_state(&key).is_some() { db.discard_for_recompute(&key); }`. Do **not** return a synthetic mutable `QueryState::Ready` for an immutable canonical image row. Read-only lookup may fall through from local current product to canonical product, with strict origin checking. Local mutation, discard and invalidation APIs must reject canonical origins or never see them.

Currentness rules differ:

```text
canonical product current iff:
    compatible canonical world identity
    + immutable product/dependency fingerprint closure valid
    + valid type/static lineage

local product current iff:
    local revision and observed dependency validations pass
```

Local queries may record a *forward* dependency on canonical key/fingerprint. Do not create workspace-owned reverse invalidation seeds for canonical products, because local edits cannot mutate them. Still record negative/candidate-set dependencies so that a new local conformance, import, or evidence binding can invalidate prior local proof outcomes even when all positive canonical fingerprints are unchanged.

**Required proof of correctness:** The query read protocol must not make negative facts permanently true, lose dependency observations, bypass semantic fallback needed for a local type, or change diagnostics/recovery. Defer broad consumer integration until a distinct successor gate if necessary; P2 completion can prove read-only image access plus a scoped semantic DB seam without a full one-shot cutover.

## 11. Core compiler reuse without a second canonical semantic pass

`phalcom-core/src/modules/canonical_universe.rs::build_canonical_universe_program` previously performs complete semantic analysis and immediately projects it into the compiled canonical program. Preserve the existing **core-owned** runtime/native descriptor verification and compiler projection, but refactor the semantic acquisition boundary:

```text
BEFORE:
    build canonical linked sources
    → analyze_workspace
    → SemanticSnapshot
    → compiler projection

AFTER:
    get canonical formal image
    → image.reference_snapshot
    → same compiler projection
```

There is a bootstrap-dependency trap: if constructing the formal image indirectly needs the core canonical runtime program, while the core runtime program now requests the formal image, initialization cycles. Audit call graph first and keep the formal-image builder independent of `CanonicalUniverseProgram`. Its inputs must be module-layer + semantic-layer facts only.

**Core acceptance:** For one cold image creation and subsequent canonical program requests, count exactly one complete canonical formal analysis, no duplicate equivalent analysis, and identical semantic/compiler/runtime behavior to the prior source-backed path. Comparison should be semantic, not blanket byte-for-byte equality of session-local IDs.

## 12. Task decomposition, write sets and implementation guidance

### T0 — Reconcile IDs and verify current tree **[blocking]**

**Write set:** C2 checkpoint/plan ledger and minimally affected program/status/guidance documents (no sweeping doc rewrite). **Read set:** current branch HEAD, full affected crate build graph, `a7b75980` and `82051480` diffs, old P2 references.

1. Record current commit and previous tested snapshot separately; confirm clean tree and absent `canonical_image.rs`.
2. Resolve P2 identifier collision per §2.
3. Confirm the newly added canonical-image test's exact intended behavior and imports.
4. Capture a failing baseline build, then ensure the new module exists before further functional work.
5. Do not claim old tests cover current HEAD.

**Gate G0:** Unique ledger identity, accurate evidence state, and target crates compile with new module stub/implementation.

### T1 — Audit bootstrap graph, final source analysis and owner boundaries

**Write set:** None initially; then minimal semantic-only constructor seam if necessary. **Read:** `canonical_universe.rs`, `session.rs`, `baseline` owner, module products and core compiler callers.

1. Trace all paths to `CanonicalUniverseSemanticBaseline` construction, semantic session creation and `analyze_workspace`.
2. Find exact `SemanticDb` transaction commit and return-publication boundary.
3. Inventory final semantic tables and query DB products available at this boundary.
4. Draw concrete call graph demonstrating no `from_foundation → default new → canonical image` recursion.
5. Verify what module image/catalog is available without a new 148-interface rebuild.
6. Capture any core/native-only prerequisites and keep them above the semantic crate.

**Gate G1:** Reviewed call graph, source references, final-product census, provenance design, identified module-image prerequisite.

### T2 — Implement complete formal image build, process identity and publication

**Primary write set:** `phalcom-semantic/src/canonical_image.rs`, `lib.rs`, minimal semantic session constructor hook and tests; small module-layer immutable-product adapter only if required.

1. Implement explicit bootstrap-from-foundation entry, bypassing completed-image lookup.
2. Run ordinary full canonical semantic update; capture committed final snapshot.
3. Store image as immutable `Arc` with world ID, module reference and type/static provenance.
4. Ensure no duplicate copy/mirroring of every semantic aggregate.
5. Add canonical-production cache with private provider override path; do not permanently cache transient cancellation failures.
6. Assert one construction on sequential/concurrent access; test override isolation.

**Gate G2:** Fully complete snapshot, successful commit, no recursion or incorrect cache sharing, canonical compiler parity harness ready.

### T3 — Implement query freeze exporter and query-image representation

**Primary write set:** `phalcom-semantic/src/db/mod.rs`, `db/query.rs` or actual query-owned source, new query image/export tests, `canonical_image.rs`.

1. Add committed-only export traversal preserving typed query products.
2. Implement explicit query-state and origin eligibility policy.
3. Validate dependency closure and world/type/static compatibility.
4. Record rejected cases with concrete categories; never export in-flight rows.
5. Ensure no scheduler/journal/revision mutable state is reachable through the retained image.
6. Add determinism of query image contents/fingerprints across equivalent cold builds to the extent current identities permit.

**Gate G3:** Query-image census, positive retained entries, no ineligible retained products, exact classification for blocked/failed states, standalone typed lookup tests.

### T4 — Establish origin-aware product lookup and currentness seam

**Primary write set:** `phalcom-semantic/src/db/*`; local tests. **Constraint:** no broad local-source cutover.

1. Introduce query origin/product/fingerprint/currentness contract or compatible existing typed API.
2. Keep `query_state` strictly local/mutable.
3. Add explicit immutable read-only lookup to the canonical query image.
4. Prove local invalidation operations cannot mutate/discard canonical image products.
5. Test negative candidate-set changes and local proof invalidation.
6. If broad query read-through requires architectural changes exceeding scope, deliver typed image lookup and defer workspace wiring in a documented successor; do **not** fake integration with a global `Ready` row.

**Gate G4:** All local mutation/discard tests pass; canonical immutable products can be read by stable origin-aware API; no unsound reuse.

### T5 — Core canonical compiler consumes the image

**Primary write set:** `phalcom-core/src/modules/canonical_universe.rs` plus narrow adapter/tests; avoid changes to main one-shot path.

1. Extract semantic-building portion into semantic-owned image builder, without duplicating the analyzer.
2. Replace redundant full Universe semantic invocation with authoritative `image.snapshot()` consumer.
3. Preserve native descriptor/runtime contract validation, compiler projection, metadata and diagnostic semantics.
4. Add differential source-backed/reference test path (test-only if appropriate).
5. Assert exactly one full canonical analysis across image construction and core compilation.

**Gate G5:** Equivalent canonical compiled semantics/runtime behavior; zero second full canonical formal pass; no bootstrap cycle.

### T6 — Integrate observability, run tiered parity and handoff

**Primary write set:** `telos-observability` instrumentation, focused tests, C2 plan/checkpoint, implementation walkthrough and evidence records.

1. Add the structural counters in §13 with ownership labels and nonzero selector baselines.
2. Measure cold-first-build, same-process warm reuse, N fresh consumers, concurrent consumers, custom world and representative fixture compile.
3. Run all focused semantic/query/adapter parity tests and unaffected core smoke tests.
4. Record command, binary/profile, host, revision, test count, failure classification and raw profiling artifact paths.
5. Document deferred downstream cutovers precisely and correct ledger state.

**Gate G6:** Full P2 gate matrix meets acceptance; only then mark new P2 verified/complete. If the existing unrelated test baseline has failures, use same-revision predecessor controls and classify, not weakening assertions.

## 13. Professional instrumentation contract

Emit hierarchical spans with meaningful ownership and cardinality, preferably through the existing `telos-observability` namespace:

```text
canonical_image.cache_lookup
canonical_image.build
canonical_image.bootstrap_semantic_update
canonical_image.transaction_commit
canonical_image.snapshot_validate
canonical_image.query_census
canonical_image.query_freeze
canonical_image.freeze_validation
canonical_image.core_consumer
```

At least:

| Counter | Cold canonical world | Warm same world | Why |
|---|---:|---:|---|
| `canonical_image.builds` | 1 | 0 additional | Once-per-world construction |
| `canonical_image.cache_hits` | 0 or expected after init | >0 | Reuse observed at consumer seam |
| `canonical_image.complete_semantic_updates` | 1 | 0 additional | No repeated formal world analysis |
| `canonical_image.bootstrap_recursion_attempts` | 0 | 0 | No initialization cycles |
| `canonical_image.snapshot_completeness_failures` | 0 | 0 | Fully finalized image |
| `canonical_query.total_rows_examined` | Nonzero | 0 additional freeze work | Census actually ran |
| `canonical_query.ready_eligible` | Positive if eligible rows exist | Reused | Useful query image |
| `canonical_query.rejected_by_reason` | Classified | Stable | No silent loss/unsafe retention |
| `canonical_query.incomplete_published_as_ready` | 0 | 0 | Safety invariant |
| `canonical_query.immutable_hits` | On read tests | >0 | Genuine image lookup |
| `canonical_query.mutable_discards` | 0 | 0 | Immutability |
| `canonical_type_prefix.validation_errors` | 0 for valid path | 0 | Arena integrity |
| `canonical_provider.cross_world_reuses` | 0 | 0 | Isolation |
| `canonical_core.second_formal_analysis` | 0 | 0 | No duplicate core analysis |

Avoid asserting `0 canonical source modules in user workspace` as a *P2* gate; that is the subsequent source-ownership cutover. Likewise `0 canonical callable reanalyses per user fixture` belongs to that later gate. Collect those baseline counters now without misclassifying unfinished work as a failed P2 image-build milestone.

Tests must prove the selected benchmark/test population is nonzero; counter assertions on dead code are meaningless. Expose both cumulative and per-invocation deltas so a first-warm build is distinguishable from ordinary warm compilation.

## 14. Detailed verification matrix

### 14.1 Target crate/build gates

Run the exact current repository-compatible forms (adapt features at HEAD):

```bash
cargo check -p phalcom-semantic
cargo check -p phalcom-core --features perf-observability
cargo test -p phalcom-semantic --lib canonical_image -- --nocapture
cargo test -p phalcom-semantic --test incremental -- --nocapture
cargo test -p phalcom-modules --test parse_cache -- --nocapture
```

Do **not** assume integration test target names from earlier notes; list the targets first (`cargo test -p ... -- --list`) and use exact discovered selectors. Run existing targeted source-index/publication/advisory tests to ensure HEAD changes are still sound. Full suites and same-predecessor classification follow after focused correctness.

### 14.2 Image/build tests

- `image_built_from_full_publication_not_partial_foundation`: assert signatures, conformance/witness/callable products absent from foundation but present in completed image (choose known concrete examples).
- `image_retains_final_type_prefix_and_static_lineage`: compare before/after canonical image with authoritative full-run reference.
- `first_build_does_not_reenter_cache`: explicitly exercise default constructor traps.
- `concurrent_first_access_builds_once`: barrier-synchronized first initialization; not merely sequential `Arc::ptr_eq`.
- `provider_overrides_are_isolated`: distinct overridden provider changes a known semantic declaration without contaminating builtin cache.
- `failure_does_not_publish_partial_image`: induced error/cancellation path, validate correct retry/caching policy.
- `canonical_compiler_consumes_image_once`: compiler still produces equivalent code/metadata with no extra full semantic pass.

### 14.3 Query freeze tests

- `freeze_only_committed_ready_closed_products`: canonical-only dependencies and unchanged fingerprints.
- `freeze_rejects_inflight_cancelled_and_budgeted`: no partial entry falsely marked ready.
- `freeze_classifies_blocked_failed_without_erasing_diagnostics`: preserve established canonical status semantics.
- `freeze_rejects_local_dependent_proof_and_negative_candidate_set`: local conformance insertion can change outcome.
- `freeze_rejects_foreign_type_or_static_world`: lineage mismatch fails predictably.
- `freeze_retains_no_journal_scheduler_or_mutable_invalidation_graph`: API/structural or test-only integrity audit.
- `read_does_not_make_canonical_row_mutable`: discard/recompute paths operate only on local query state.
- `multi_workspace_reads_do_not_mutate_shared_image`: independent sessions, concurrent safe reads.

### 14.4 Differential parity

Use source-backed canonical full analysis as reference. Compare semantically by stable product identity/fingerprint and relevant diagnostics:

```text
complete declarations, types/kinds, callable signatures,
final callable analyses, inferred returns, hierarchy/dispatch,
trait requirements, explicit/derived conformances,
witness plans, generic evidence, static values,
canonical diagnostics, compiler-projection input facts,
canonical program behavior and runtime outputs.
```

Do not require byte equality where ephemeral workspace/store IDs legitimately differ. Do require stable canonical prefix meaning and unmodified observable language behavior.

### 14.5 Revision/failure attribution

The current checkpoint records some baseline failures outside this task (`corpus::booleans`, runtime `ProductKind` bootstrap, a missing `fold(initial,using)` method, formal-projection causality test, stale callable-count expectations). Do **not** mark them fixed or newly introduced without a same-state control. Keep test expectations intact. The absence of tests after `a7b75980` means new P2 evidence must be collected from the actual final implementation revision.

## 15. Performance experiments and acceptance interpretation

Test at minimum:

1. **Cold process, first world:** report unavoidable full image build time and memory, plus source-parity status.
2. **Same process, warm image lookups:** demonstrate no repeated image construction or canonical full semantic pass.
3. **Fresh consumers sharing a warm image:** at least ten fresh consumers/references, one common `Arc` image and one canonical build.
4. **Concurrent first calls:** one build under N callers, error behavior deterministic and no deadlock.
5. **Private provider override:** an independent world with no false cross-reuse.
6. **Canonical core program build:** compare prior source-backed path versus image-backed semantic acquisition, report work-cardinality deltas.
7. **Representative small fixture:** measure compiler totals, but recognize most fixed-floor elimination awaits later workspace externalization.

Performance budgets are structural first. **Do not require `<200 ms` full tiny fixture compilation as P2 acceptance**: that stretch target in the checkpoint depends on follow-on query overlay/module-ownership/metadata/local settlement work. Do not attribute historical 10.571-second evidence to the untested commits.

## 16. Failure modes and required rulings

| Failure mode | Cause | Required response |
|---|---|---|
| Initialization recursion/deadlock | Bootstrap session indirectly calls `canonical_image()` | Separate foundation constructor; reentrancy test; no global mutex held across nested semantic callbacks |
| Partial image presented as full | Freeze baseline or precommit snapshot | Reject; build from committed complete full update |
| One image for different providers | Cache keyed only by Universe project flag | World-keyed caching or private overridden-provider path |
| Transient failure cached permanently | `OnceLock<Result<...>>` retains cancellation/I/O/transient error | Classify error and choose retry-safe synchronization |
| Canonical query exposed as mutable `Ready` | DB conflates origin with local lifecycle | Keep `query_state` local-only and add immutable read interface |
| Reuse of local-negative conformance proof | Dependency closure omits candidate-set generation | Reject from frozen image or parameterize and validate locally |
| Lost static-value reference | Consumer creates independent store | Enforce compatible static lineage/fork |
| Canonical product wrong `TypeId` meaning | Prefix from pre-analysis baseline, foreign arena | Freeze final prefix; validate consumer provenance |
| `SemanticDb` captured wholesale | Mutable transaction/scheduler/state leaks into singleton | Purpose-built committed export projection only |
| Core–semantic crate cycle | Semantic image constructor calls core canonical compiler | Image builder semantic-only; core depends on image |
| Data duplication / high memory | Mirrors dozens of snapshot maps and DB storage | `Arc<SemanticSnapshot>` + compact query roots; measure retained bytes |
| Duplicate full canonical analysis | Core still calls `analyze_workspace` after image acquisition | Single semantic acquisition seam; structural counter |
| Snapshot treated as workspace previous | `last_snapshot = image.snapshot` | Forbidden; immutable external parent, local snapshots start `None` |
| Partial test acceptance | Old commit passes used for current HEAD | Collect revision-specific nonzero selectors and baseline comparisons |

Escalate any of these immediately rather than compensating with new special-case semantic shortcuts.

## 17. Exclusions and successor handoff contracts

This P2 **does not** implement:

- External-module lookup/linking and local-only `WorkspaceSourceState` ownership;
- `ProgramCompiler::compile_entry_selection` full ephemeral-overlay cutover and deletion of the 40-module source seeding;
- Large-scale persistent-module-sharded replacement of every semantic aggregate;
- SCC-driven inferred-return settling or module-value dependency-sensitive callable reanalysis;
- Indexed conformance candidate/witness scans and generic proof memoization beyond correctness of query freeze;
- Metadata root-driven reachability or cross-pool runtime metadata references;
- Portable code prototypes, artifact format, `.telpkg`/`.telproj`, disk serialization or VM heap snapshotting;
- A new `SemanticSnapshot::from_parts` rewrite unless needed for correctness of existing constructor behavior;
- Changes to the language's diagnostics, coherence, inferred types, generic evidence or runtime reflection behavior.

**Required handoff to subsequent image-backed workspace program:** Deliver the completed `CanonicalUniverseFormalImage`, a typed immutable `CanonicalSemanticQueryImage` API, exact provenance identities, the `SemanticDb` origin/currentness contract (implemented or gated), and the critical rule that local `last_snapshot` remains `None` on a fresh image-backed session. Document all currently necessary compatibility aggregate copies, query kinds not yet eligible, and any source-backed fallback still occurring. Follow-on work can then establish `CanonicalSemanticBase + WorkspaceLocalOverlay`, external module boundaries, source injection removal, then incremental structural optimization.

## 18. Deliverables to commit and review

1. This P2 plan and approved identifier/ledger amendment, with normative decisions recorded.
2. `phalcom-semantic/src/canonical_image.rs` implementation, complete snapshot ownership and stable API.
3. Explicit bootstrap-from-foundation path and no-recursion enforcement.
4. Formal image process sharing, custom-provider isolation and world/prefix provenance.
5. Committed query exporter, immutable query-image type(s), eligibility/rejection reporting.
6. Typed read-only canonical query lookup and local-mutable state separation (or explicitly gated downstream integration, with no unsound shortcuts).
7. Core canonical compiler semantic acquisition reuse, no second full Universe pass.
8. Comprehensive focused tests, differential parity tests and concurrency/negative-fact tests.
9. Observability spans/counters with cold-versus-warm evidence and exact tested revision.
10. Implementation walkthrough including files/symbols changed, source call-graph diff, query freeze census, known exclusions, measured results, test command outputs and classified failures.
11. Updated checkpoint/program/status/guidance links preserving prior publication work and evidence.

## 19. Review checklist / acceptance gates

A reviewer must be able to answer **YES** to each item:

- **L0:** P2 ID collision resolved without erasing the existing consumer-publication package.
- **G0:** Current implementation revision builds; new tests actually execute and select nonzero cases.
- **G1:** Source call graph proves semantic-owned, recursion-free bootstrap; module-image prerequisite is explicit.
- **G2:** The stored snapshot comes from a successfully committed *complete* canonical analysis, with full semantic census.
- **G2b:** Sequential and simultaneous callers reuse exactly one compatible-world image; overrides do not reuse builtin image.
- **A:** Final canonical `TypeStore` prefix and `StaticValueStore` lineage are preserved; incompatible origins reject.
- **G3:** Query root freeze contains only eligible immutable committed products, with complete rejection accounting and no mutable DB state.
- **G4:** Canonical query products cannot be discarded or invalidated through workspace-local query APIs; negative facts remain sound.
- **G5:** Core canonical compilation consumes the image without a redundant full canonical analysis and preserves semantics/runtime behavior.
- **G6:** Structural counts, parity suites, revision-specific measurements and failure classifications are documented.
- **Boundary:** No image snapshot seeded into `last_snapshot`, no invented canonical source cutover, no artificial `<200 ms` claim.

**Do not mark P2 `COMPLETE / VERIFIED` if any acceptance gate is missing.** Partial implementation must remain `IN_PROGRESS / PARTIAL / UNVERIFIED`, with the exact current HEAD and the evidence produced at that HEAD recorded.

## 20. Source references and provenance

The following are the documentary source basis for this plan. These paths are local to the current supplied conversation, not links to an independently verified current repository checkout.

- `CHECKPOINT.md` (updated by reported `82051480`): §0 state and next action; §1 objectives; §2 plan ledger and numbering collision; §3 invariants; §4 previous evidence and new unverified implementation; §6 proposed architecture lifecycle; §7 next action.
- `PERF003-C2-Unified-Compiler-Artifact-Implementation-Spec.md`: §§3–7 identities, canonical module/formal image, immutable query roots; §§14–15 performance program and ordering; §19 adoption/lifecycle. **Explicitly proposed in checkpoint, not blanket-adopted.**
- `PERF003.C2-CORRECTED-IMPROVED-AUTHORITATIVE-ARCHITECTURE.md`: §§23–35 complete canonical publication, bootstrap, committed query freeze, DB currentness, dependency and arena provenance; other sections describe fixed-cost mechanisms and necessary subsequent work.
- `Pasted text.txt`: preceding architect audit at `99b84ecc`; especially T3 `last_snapshot` blocker, module/formal/runtime ownership split, source-backed Universe discovery, runtime-minimal evidence, and revised plan sequence. **Older than newly reported `a7b75980`.**

### Epistemic labels

- **Established/source-backed at the previously audited revision:** ordinary complete Universe analysis exists; partial baseline is not full final image; full-workspace source injection and repeated query publication are historical bottlenecks; current mutable DB cannot be treated as canonical immutable query state.
- **Recorded after the latest pushes, not verified by tests:** missing `canonical_image.rs`, new presentation-filtering changes, current clean branch report and checkpoint ledger.
- **Proposed architecture / implementation contract:** `CanonicalSemanticWorldId`, `CanonicalTypePrefixId`, `CanonicalQueryEntry`, freezing protocol, typed lookup and new P2 plan name. Validate against HEAD and test before claiming implementation.

---

**Implementation directive:** Complete T0–T6 in dependency order; keep commits reviewable by gate; provide a read-only audit report with source-backed evidence before any broad compiler-session cutover. The desired result of P2 is a *correct, complete and reusable canonical semantic/query foundation*, not a fast-but-semantically-incomplete alternative analyzer.

## Execution amendment — 2026-10-13

L0 was explicitly approved by the user: existing publication Package B is P7.
The user then requested rapid code implementation, ignoring baseline investigation
and deferring broad verification to conserve time/usage. Implement the formal/query
foundation and core acquisition seam; retain all baseline diagnostics and guards.
Do not mark the full P2 gates complete from the focused delivery. Typed reader access
is the T4 seam; automatic workspace read-through remains a separately gated successor.
See the bootstrap audit, walkthrough and handoff for actual evidence and residual gates.
