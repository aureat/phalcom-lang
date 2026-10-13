---
id: PERF003.C1.P5
category: PERF
program: PERF003
checkpoint: PERF003.C1
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
baseline_head: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head_at_t0: 303222c3b027c34ddacd4749f056e1095aabfa81
depends_on: []
follows: null
supersedes: null
deferred_reason: null
---
# PERF003.C1.P5 — Compiler & Semantic Performance Observability

**Program:** `PERF003`
**Checkpoint:** `PERF003.C1`
**Plan:** `PERF003.C1.P5`
**Title:** Compiler & Semantic Performance Observability
**Primary new crate:** `telos-observability`
**Plan type:** Luna patch-grade implementation plan
**Status:** IN_PROGRESS — T0–T7 implementation is in place; deterministic T6 test coverage remains outstanding
**Current-main source baseline (`BASELINE_HEAD`):** `e2ae4a0fd360e4075ce8ad229a3e39e1531fb771`
**T0 execution-tree HEAD:** `303222c3b027c34ddacd4749f056e1095aabfa81`
**Execution branch:** `perf/universe-characterization`, updated from current `main` before implementation
**Scope:** instrumentation and observability only; no compiler optimization or semantic behavior change

---

## 1. Objective

Install a permanent, versioned, low-overhead compiler-performance observability architecture spanning:

- canonical Universe source/bootstrap construction;
- module discovery, loading, interface construction, import resolution, and linking;
- canonical semantic baseline construction;
- `SemanticWorkspaceSession` incremental analysis;
- semantic query reuse/revalidation/recomputation;
- callable-body analysis and reanalysis;
- type-store and aggregate semantic-state copying;
- dependency-graph propagation;
- source traversal;
- generic inference, relation, and conformance-search work;
- compiler projection;
- metadata export;
- semantic-to-runtime lowering;
- ordinary project/package/standalone/inline compilation;
- `ProgramCompilationSession`;
- VM/native bootstrap;
- Universe bytecode compilation and initializer execution;
- persistent benchmarking and comparison through `phalcom-perf`.

The system must answer both:

> Where did compilation time go?

and:

> What compiler work caused that time, how much of it was repeated, and which semantic entities caused it?

P5 must establish the measurement substrate before any attempt to optimize Universe bootstrapping, semantic analysis, compiler projection, or runtime startup.

---

## 2. Architectural principle

P5 is not a one-off Universe profiler.

It establishes a long-lived compiler observability contract:

```text
                     Compilation / Analysis Run
                                │
             ┌──────────────────┼──────────────────┐
             │                  │                  │
             ▼                  ▼                  ▼
       hierarchical spans    work counters     attribution
             │                  │                  │
             │                  │                  │
        elapsed time       recomputation       module
        child phases       copy volume         callable
                           graph traversal      declaration
                           source traversal     impl
                           search volume        query kind
             │                  │                  │
             └──────────────────┼──────────────────┘
                                ▼
                      versioned run report
                                │
                         `phalcom-perf`
                                │
                   baseline / A-B comparison
```

The observability architecture must cover both:

```text
canonical Universe compilation
ordinary user compilation
```

through the same instrumentation model.

Universe is a workload classification, not a separate profiler.

---

## 3. Hard scope boundaries

### 3.1 P5 is authorized to

- create `telos-observability`;
- add low-level observability types, spans, counters, recorders, and reports;
- add feature gates and runtime observability levels;
- add stable performance-phase and metric taxonomies;
- classify `QueryKey` into a stable `QueryKind`;
- instrument semantic query requests/reuse/revalidation/recomputation;
- instrument canonical Universe compilation and bootstrap phases;
- instrument semantic baseline construction;
- instrument `SemanticWorkspaceSession` update phases;
- instrument compiler projection and metadata export;
- instrument known large semantic-state copies without changing copy semantics;
- instrument dependency closures and traversal counts;
- instrument source traversal volumes;
- instrument callable analysis multiplicity;
- instrument generic/conformance search volume where bounded and low-risk;
- add module/entity attribution;
- add versioned JSON reporting;
- integrate compiler profiles into `phalcom-perf`;
- add deterministic structural tests for instrumentation;
- benchmark and document instrumentation overhead.

### 3.2 P5 is NOT authorized to

- precompile or persist Universe;
- create a Universe artifact;
- create a runtime image or heap snapshot;
- remove or alter Universe source seeding;
- change TypeStore storage architecture;
- replace deep copies with overlays or persistent structures;
- change query invalidation semantics;
- change semantic phase ordering;
- change callable reanalysis behavior;
- optimize metadata scans;
- optimize conformance search;
- parallelize semantic analysis;
- change module discovery/linking semantics;
- cache new compiler products for performance;
- alter VM bootstrap semantics;
- remove runtime initializers;
- reuse runtime bytecode across VMs;
- change compiler correctness behavior.

Instrumentation changes may make existing work observable, but must not reduce or increase semantic work except for unavoidable measurement overhead.

---

## 4. Repository-state and execution record

This file is exclusively the `PERF003.C1.P5` plan. It is additive and does not modify,
replace, amend, or renumber any existing `PERF003.C1.P1` through `PERF003.C1.P4` plan or
record; those P1–P4 files remain as they are. P1 remains partially implemented with its
separate T7 frozen-runtime-image architecture stop.

The original startup characterization was prepared against
`b00d75b3ac9f50d37cf9abfddfde00917e099031`. Before P5 implementation, the user directed
that the latest committed progress from `main` be brought into the dedicated performance
worktree. `main` and `origin/main` both resolved to
`e2ae4a0fd360e4075ce8ad229a3e39e1531fb771`. That revision was merged into
`perf/universe-characterization`, producing execution-tree HEAD
`303222c3b027c34ddacd4749f056e1095aabfa81`. This merge includes the main commits
`10f7297c` and `e2ae4a0f`, plus the prior committed characterization assets. No stashed
or uncommitted main-worktree changes were applied, copied, or used as inputs. The earlier
characterization report remains evidence for `b00d75b3...`; it is not treated as evidence
for the current-main source baseline.

P5 records both revisions in every profile/report: `BASELINE_HEAD` identifies the exact
committed `main` source baseline, and `execution_tree_head` identifies the exact checked-out
tree used for a run. When P5 source changes exist, report their commit/tree separately;
never relabel the merged execution tree as the `main` baseline.

Initial state:

```text
HISTORICAL_CHARACTERIZATION_HEAD=b00d75b3ac9f50d37cf9abfddfde00917e099031
BASELINE_HEAD=e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
T0_EXECUTION_TREE_HEAD=303222c3b027c34ddacd4749f056e1095aabfa81
branch=perf/universe-characterization
toolchain=nightly-2026-07-10
worktree=/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization
```

At T0, `main` and `origin/main` matched. The dedicated branch was at the recorded
execution-tree HEAD and contained only the user-provided P5 plan as an untracked
task artifact before this synchronization edit; the main worktree was not modified.

Current source facts established at T0:

- `TypeStore` uses `im::HashMap` and `im::Vector`. Its derived clone shares
  persistent collection structure; `fork_from_baseline()` assigns a fresh
  `TypeStoreId`. A fork is not a full node copy.
- `SemanticUpdateStats` reports eight whole snapshot-map materializations at
  the current publication seam and zero TypeStore snapshot deep copies. These
  are separate work categories.
- Each semantic update opens a `SemanticDb` undo-journal transaction. The journal
  retains revision-local recompute/revalidate sets, clones scheduler state, and
  lazily saves prior per-key query-state, product, last-known-good, and
  module-to-query-key values as they mutate. Success commits; failure rolls the
  database back and restores prior store/static values/semantic-structure shards.
  Instrumentation must separate attempted work from successful publication work.

T0 re-audit confirmed two pre-existing execution blockers on the updated source state:

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test language-corpus corpus::booleans -- --exact` selects one test and fails before VM construction with `legacy metadata cannot represent a Record remainder outside the Symbol key domain`.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 run -p phalcom-core --example perf_universe_characterization --features test-timings -- tiers 2` completes kernel/native controls, then `VM::new()` fails with `intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration`.

These baseline failures block full-runtime benchmark acceptance, but do not authorize
semantics or runtime workarounds. Implementation may proceed on separable observability
substrate and instrumentation tasks; gates that require a successful canonical compile or
full VM must remain `BASELINE_BLOCKED` until the owning correctness fixes land.

---

## 5. Existing facilities to preserve and integrate

P5 must reuse existing instrumentation rather than create competing authorities.

### 5.1 `phalcom-core/src/instrumentation.rs`

Existing structural counters:

```text
FULL_VM_BOOTSTRAPS
CANONICAL_UNIVERSE_PRODUCT_BUILDS
BOOTSTRAP_IMAGE_INSTANTIATIONS
UNIVERSE_MODULE_COMPILATIONS
UNIVERSE_INITIALIZER_EXECUTIONS
```

Existing timing models:

```text
VmBootstrapTiming
FixtureTiming
```

These remain compatibility/test surfaces during P5.

They may delegate to the new observability model where practical, but P5 must not break current tests merely to replace them.

### 5.2 `phalcom-semantic/src/instrumentation.rs`

Existing structural counters:

```text
SEMANTIC_BASELINE_BUILDS
SEMANTIC_SESSION_INSTANTIATIONS
```

Retain these for structural reuse assertions.

### 5.3 `SemanticUpdateStats`

`SemanticUpdateStats` remains the canonical semantic incremental-work summary.

Do not duplicate these counters into a second mutable semantic accounting implementation.

The new observability report must incorporate or transform `SemanticUpdateStats`.

### 5.4 `SemanticDb`

Preserve:

```text
revision_recomputed
revision_revalidated
query_keys_by_module
DependencyIndex
QueryScheduler
QueryMetrics
```

P5 augments query observability at these existing semantic ownership boundaries.

### 5.5 `phalcom-perf`

`phalcom-core/bin/perf/` remains the persistent benchmark/run comparison authority.

Do not create a parallel standalone performance database.

---

## 6. New crate: `telos-observability`

### 6.1 Responsibility

`telos-observability` owns general compiler/runtime performance-observability primitives.

It must not own semantic/compiler meaning.

Recommended dependency direction:

```text
telos-observability
        ▲
        │
phalcom-modules
        ▲
        │
phalcom-semantic
        ▲
        │
phalcom-core
```

The exact dependency graph may differ because `phalcom-modules` may not need observability immediately, but the rule is:

> no observability type may require an upward dependency from a lower compiler layer.

### 6.2 Initial dependencies

Prefer:

```text
std
serde
serde_json
```

If serialization is only required in consumers, keep serialization support feature-gated.

Avoid dependencies on:

```text
phalcom-core
phalcom-semantic
phalcom-modules
```

unless the dependency graph still remains acyclic and there is a compelling reason.

Domain-specific IDs should be converted to light report keys by the owning crate.

### 6.3 Required top-level types

P5 must establish equivalents of:

```rust
pub enum ObservabilityLevel {
    Off,
    Summary,
    Detailed,
    Trace,
}

pub struct PerfRecorder { ... }

#[derive(Clone)]
pub struct PerfContext { ... }

pub struct PerfSpanGuard { ... }

pub struct PerfRunMetadata { ... }

pub struct PerfReport { ... }
```

Exact names may vary, but responsibilities may not.

---

## 7. Run-scoped ownership

The primary instrumentation model must be run-scoped.

Do not make performance reports depend on resetting global atomics.

Required model:

```text
CompilationRun
    └── PerfRecorder
          ├── spans
          ├── counters
          ├── query summaries
          ├── module summaries
          ├── entity summaries
          └── structural metrics
```

A recorder must be isolated from unrelated parallel compilations.

### 7.1 Concurrency contract

`PerfContext` and the underlying recorder must be:

```text
Send + Sync
```

unless a hard architectural constraint discovered during implementation requires consultation.

Even if current analysis is synchronous, the observability architecture must not preclude future parallel compilation.

### 7.2 Global counters

Process-global atomics may remain only for:

- process singleton assertions;
- bootstrap construction counts;
- existing regression tests.

They are not the authoritative performance-report model.

---

## 8. Process-cold canonical instrumentation

Canonical products use process-global `OnceLock`s.

Cold canonical profiling therefore requires recorder installation before first canonical access.

P5 must provide an explicit mechanism for:

```text
fresh process
    ↓
install process cold profiler
    ↓
canonical_universe_program()
    ↓
all nested work attributed to same profile
```

Do not attempt to simulate cold canonical construction inside a process after the singleton is initialized.

The benchmark harness must distinguish:

```text
process-cold
process-warm
session-cold
session-warm
VM-cold inside warm process
```

---

## 9. Stable phase taxonomy

Phase names must be centrally defined and versioned.

Do not use unconstrained string phase names at call sites as the authoritative representation.

Recommended phase root taxonomy:

```rust
pub enum PerfPhase {
    Compilation,
    SourceDiscovery,
    SourceLoad,
    Parse,
    InterfaceBuild,
    ImportResolution,
    Link,

    CanonicalUniverse,
    CanonicalSourceIndex,
    CanonicalNativeVerification,
    CanonicalSemanticBaseline,
    CanonicalSemanticAnalysis,
    CanonicalCompilerProjection,
    CanonicalBootstrapPlanning,

    SemanticSessionInit,
    SemanticUpdate,
    SemanticSnapshotPublication,

    CompilerProjection,
    MetadataExport,
    SemanticLowering,

    VmBootstrap,
    VmKernel,
    VmNative,
    UniverseVmCompilation,
    UniverseInitializerExecution,

    UserExecution,

    Other,
}
```

Subphase representation may use a stable enum hierarchy or stable typed identifiers.

Requirements:

- deterministic naming;
- schema version;
- stable serialized representation;
- no accidental renaming without schema consideration;
- child spans must retain parent identity.

---

## 10. Span model

A span records:

```text
phase
start
duration
parent
outcome
optional scope attribution
optional lightweight tags
```

Outcomes must include:

```text
success
semantic_error
parse_error
link_error
bootstrap_error
cancelled
budget_exceeded
blocked
panic_boundary
other_failure
```

RAII guards must close on ordinary early-return error paths.

Do not claim successful completion for a phase that terminated with an error.

P5 does not need to recover from arbitrary Rust process aborts.

---

## 11. Observability levels

### 11.1 `Off`

No collection.

When the feature itself is absent, instrumentation should compile away where practical.

### 11.2 `Summary`

Benchmark-grade.

Collect:

- major phase spans;
- aggregate work counters;
- `SemanticUpdateStats`;
- query-kind aggregate counts;
- copy count/volume;
- module cardinalities;
- bounded derived metrics.

No full per-query event retention.

### 11.3 `Detailed`

Diagnostic mode.

Additionally collect:

- module attribution;
- callable multiplicity;
- query-kind timing;
- top-N expensive/repeated entities;
- dependency-closure metrics;
- search-space metrics.

### 11.4 `Trace`

High-detail diagnostic mode.

May retain or stream selected high-cardinality events.

Trace timings must not be presented as benchmark-grade absolute timings.

---

## 12. Feature gating

Introduce a permanent feature, recommended:

```toml
perf-observability = [...]
```

in relevant crates.

Existing:

```toml
test-timings
```

should remain supported.

Recommended compatibility:

```text
test-timings ⇒ perf-observability
```

during transition.

Requirements:

- default compiler builds do not enable detailed performance collection;
- runtime `Off` does not allocate large report structures;
- no per-opcode instrumentation is added;
- `vm-trace` remains independent;
- observer overhead is measured before acceptance.

---

## 13. Canonical Universe top-level instrumentation

Instrument `build_canonical_universe_program()` or its post-main equivalent with stable spans:

```text
canonical_universe.total

canonical_universe.source_index
canonical_universe.native_verification

canonical_universe.interface_loading
canonical_universe.import_resolution
canonical_universe.linking

canonical_universe.semantic.total

canonical_universe.match_validation

canonical_universe.compiler_projection.total

canonical_universe.bootstrap_planning
canonical_universe.product_validation
```

Required work counters:

```text
source units
interfaces
imports
linked modules
root-reachable modules
bootstrap modules
native descriptors
native anchors
```

The current opaque `canonical_universe_product` duration may remain as a compatibility aggregate but must derive from the new phase tree where possible.

---

## 14. Canonical semantic baseline instrumentation

Instrument the canonical semantic-baseline builder.

Required phases:

```text
semantic.canonical_baseline.total

semantic.canonical_baseline.linked_world

semantic.canonical_baseline.native_surfaces

semantic.canonical_baseline.predeclaration
semantic.canonical_baseline.trait_census
semantic.canonical_baseline.hierarchy

semantic.canonical_baseline.class_surfaces
semantic.canonical_baseline.callable_signatures

semantic.canonical_baseline.data
semantic.canonical_baseline.enums

semantic.canonical_baseline.inherent_impls
semantic.canonical_baseline.effective_surfaces

semantic.canonical_baseline.enum_behavior
semantic.canonical_baseline.enum_requirements

semantic.canonical_baseline.associated_surfaces
semantic.canonical_baseline.type_members
```

Post-type-system re-audit may rename or split concrete phases if the source structure changed.

Required counts:

```text
modules scanned
top-level statements visited
declarations
classes
traits
data declarations
enum declarations
impl declarations
callables/signatures
associated products
type members
```

---

## 15. `SemanticWorkspaceSession` phase ledger

Instrument one update as a stable semantic phase tree.

Required initial taxonomy:

```text
semantic.update.total

semantic.update.inputs
semantic.update.invalidation
semantic.update.structural_shards
semantic.update.dependency_graphs

semantic.update.generic_headers
semantic.update.declarations
semantic.update.aliases
semantic.update.hierarchy

semantic.update.callable_signatures
semantic.update.field_signatures
semantic.update.declaration_surfaces

semantic.update.data
semantic.update.enums

semantic.update.conformances
semantic.update.associated
semantic.update.type_members

semantic.update.callable_bodies.initial
semantic.update.callable_bodies.downstream
semantic.update.callable_bodies.cycle
semantic.update.callable_bodies.conformance
semantic.update.callable_bodies.final_revalidation

semantic.update.module_bodies

semantic.update.static_definitions
semantic.update.static_constants

semantic.update.source_index
semantic.update.formal_projection
semantic.update.advisory

semantic.update.diagnostics
semantic.update.snapshot_publication
semantic.update.effects
```

Requirements:

- align boundaries to real implementation phases;
- do not wrap arbitrary small code fragments;
- do not change semantic ordering;
- nested phase totals must reconcile reasonably with the parent span;
- unattributed residual time must remain visible.

---

## 16. `SemanticUpdateStats` integration

`SemanticUpdateStats` remains the semantic incremental-work authority.

At profile finalization, copy/transform its fields into the report.

Do not separately increment new duplicate counters for:

```text
modules_recomputed
callables_recomputed
callables_reused
semantic_structure_shards_recomputed
...
```

unless the new value measures a genuinely different concept.

The report should preserve both:

```text
time
work count
```

for semantic phases where possible.

### 16.1 Attempted work versus successful publication

`SemanticWorkspaceSession` performs an update inside a `SemanticDb` transaction.
Every profile must distinguish:

```text
attempted update work
committed successful publication work
rolled-back failed/cancelled attempt work
```

Query and phase events observed before rollback remain evidence of attempted work;
they must not be attributed to a successfully published snapshot. Record transaction
outcome and, where available without changing semantics, undo-journal activity by
category (query-state entries, module-key entries, product entries, last-known-good
entries, dependency-index journal entries, and scheduler snapshot size/work).
Keep attempt-local counters separate from committed-publication aggregates. A
cancelled update rejected before transaction start is a distinct outcome and must
not be counted as a journaled attempt. Do not alter transaction, cancellation, or
publication behavior to simplify these metrics.

---

## 17. Query taxonomy

Add a stable:

```rust
pub enum QueryKind { ... }
```

and:

```rust
impl QueryKey {
    pub fn kind(&self) -> QueryKind;
}
```

Every current `QueryKey` variant must map exactly once.

The mapping must be exhaustively matched by the compiler so new query kinds cannot silently become `Other`.

If an `Other` escape hatch is required for compatibility, new `QueryKey` variants must fail an observability coverage test until classified.

---

## 18. Query lifecycle semantics

Define precise query events:

```text
request
direct_reuse
revalidated_reuse
recompute_start
recompute_finish
invalidated
blocked
cancelled
budget_exceeded
failed
```

Do not use ambiguous `hit/miss` semantics in the authoritative report.

For every `QueryKind`, collect:

```text
requests
direct reuse
revalidated reuse
recomputed
invalidated
blocked
cancelled
budget exhausted
failed
recompute duration total
recompute duration max
```

Optional in Detailed mode:

```text
recompute p50/p90/p99
```

if sufficient samples exist.

---

## 19. Query timing implementation constraint

A query timing guard must not retain a borrow of `SemanticDb`.

Required pattern:

```text
create lightweight probe
release db borrow
perform query
finish probe
```

Probe state may retain:

```text
Arc/handle to recorder
QueryKind
optional module/entity attribution
Instant
```

but not an exclusive semantic database borrow.

This is required to avoid invasive borrow-graph changes.

---

## 20. Reconcile `QueryMetrics`

Audit current `QueryMetrics`.

P5 must:

1. document existing metric semantics;
2. correct ambiguous names/recording where needed;
3. wire metrics that remain meaningful;
4. avoid maintaining contradictory query accounting systems.

Preferred long-term counters:

```text
requests
direct_reuse
revalidated
recomputed
invalidations
cancellations
budget_exhaustions
```

`relation_steps` and `scc_iterations` may remain if properly wired and meaningful.

No semantic behavior may change to make the metrics cleaner.

---

## 21. Callable-body multiplicity

Instrumentation must record per-`CallableId` body-analysis multiplicity.

At minimum:

```text
unique callable bodies analysed
total body analyses
```

Detailed mode must record:

```text
analyses by callable
analyses by reason
```

Reasons must distinguish at least:

```text
initial
downstream
cycle
conformance-plan
final revalidation
other
```

Required derived metric:

```text
body analysis amplification =
    total body analyses / unique callables analyzed
```

Required histogram:

```text
1x
2x
3x
4x
5x+
```

Required top-N:

```text
most reanalysed callables
```

Do not assert durations in deterministic unit tests.

---

## 22. Module attribution

Major work must be attributable to `ModuleId` where semantically meaningful.

Required module summaries should support:

```text
phase time
query recomputes
callable analyses
signature work
source traversal
projection work
```

Classification must distinguish:

```text
Universe
current project
dependency project
synthetic/standalone
```

If an operation is workspace-global, retain explicit workspace attribution rather than assigning it to an arbitrary module.

---

## 23. Universe work inside ordinary compilation

This is a mandatory P5 metric because future optimizations must prove that ordinary packages stop reprocessing Universe source.

For each ordinary compile, record:

```text
Universe modules present
Universe modules considered
Universe modules recomputed
Universe query recomputes
Universe callable bodies analysed
Universe statement visits
Universe semantic phase time where attributable

user project modules
dependency modules
```

Required derived metric:

```text
Universe semantic work ratio
```

defined using deterministic work counters where possible rather than only wall time.

---

## 24. TypeStore fork, copy-on-write, and materialization accounting

The current `TypeStore` uses persistent `im::HashMap` and `im::Vector` collections.
`fork_from_baseline()` clones their shared roots and assigns a fresh store ID; it does
not copy every node. P5 must not label a persistent-root fork as a deep copy or infer
copied-node volume from the number of elements already present in the baseline.

Report these categories separately:

```text
baseline fork count and duration
baseline logical sizes by collection/category at fork
post-fork mutation operation counts by collection/category
directly observed copy-on-write node/byte work, only if the collection API exposes it
whole-map materialization count, identity, duration/cardinality when directly available
```

Mutation operation counts are not physical COW node/byte counts. If the persistent
collection implementation does not expose exact detach/allocation work through a
bounded, low-overhead observation point, report that physical COW volume as
`unavailable`; do not estimate it from logical inserts or call it copied content.
Keep the existing `SemanticUpdateStats` facts visible: current snapshot publication
reports eight whole-map materializations and zero TypeStore snapshot deep copies.
Attribute these values to the relevant update attempt and expose committed versus
rolled-back publication outcomes as specified in §16.1.

Use separate reason enums for persistent forks and actual full copies, for example:

```rust
pub enum TypeStoreForkReason {
    BaselineFork,
    Other,
}

pub enum TypeStoreCopyReason {
    CompilerMaterialization,
    SemanticScratch,
    Metadata,
    Test,
    Other,
}
```

Actual full copies/materializations must use an instrumentation-aware helper while
retaining exact semantics. `fork_from_baseline()` uses a distinct fork event and reason;
it must not route through a full-copy helper:

```rust
store.full_copy_for(reason)
```

or equivalent.

Required fork and physical-copy metrics:

```text
fork count
fork duration
logical baseline sizes by type/kind/parameter/row/family/static-domain/static-constructor category
physical COW nodes/bytes copied, if directly observable
full-copy count and duration, only for actual full copies
full-copy content volume by category, only for actual full copies
```

Do not optimize or alter store copying in P5. Keep persistent-root fork counts,
actual COW work, and snapshot full-map materializations as distinct report series.

Derived copy amplification is permitted only for directly measured physical copy
volume or full-map materialization volume with a meaningful retained-content
denominator. Do not include root forks or logical mutation counts in its numerator:

```text
TypeStore copy amplification
```

where a useful retained-node denominator is available.

---

## 25. Aggregate semantic table copying and snapshot materialization

Instrument known large aggregate clones in semantic publication/reanalysis paths.

Candidates include:

```text
CallableSignatureTable
DeclarationTypeTable
MapTypeHierarchy
SurfaceDispatchResolver
TypeAliasTable
conformance tables/indexes
associated-surface tables
```

Do not instrument every Rust `Clone`.

Required data:

```text
copy reason
copy count
entries copied
duration
```

The eight current whole-snapshot-map materializations reported by
`SemanticUpdateStats` are a known publication category, not TypeStore copy volume.
Identify the map/root category at the instrumentation seam; distinguish the
materialization attempt from a committed publication, and report retained-root
reuse/replacement alongside it. If current code exposes no direct entry count or
duration, report those fields as unavailable instead of manufacturing a count.

Where a final retained size exists, calculate:

```text
copy amplification =
    cumulative entries copied / final retained entries
```

If a type cannot expose a cheap cardinality, do not add expensive instrumentation solely to compute it without consultation.

---

## 26. Dependency graph traversal metrics

Instrument major `DependencyIndex::reverse_closure` and equivalent graph traversals.

Required reason enum, for example:

```rust
pub enum DependencyTraversalReason {
    Invalidation,
    SemanticWorkSeeding,
    DownstreamBodies,
    ChangedSignaturePropagation,
    CycleReanalysis,
    ConformancePlanChange,
    Retirement,
    Other,
}
```

Required metrics:

```text
seed count
nodes visited
edges traversed where cheaply available
result count
duration
```

Required derived metric:

```text
fanout amplification =
    visited nodes / max(seed count, 1)
```

If current traversal APIs do not expose visited-edge count cheaply, P5 may extend internal return statistics without changing traversal semantics.

---

## 27. Source traversal metrics

Track source traversal at semantic phase boundaries.

Required metrics:

```text
unique modules
unique top-level statement count
module visits
top-level statement visits
```

Do not place an atomic increment inside every AST node visit in Summary mode.

Preferred approach:

```text
when scanning a complete module:
    +1 module visit
    +program.statements.len() statement visits
```

Required derived metric:

```text
source traversal amplification =
    total top-level statement visits / unique top-level statements
```

Detailed mode may add deeper expression counts if later evidence requires it.

---

## 28. Generic/type-relation/conformance work counters

Establish counters at existing algorithmic choke points.

Initial required counters:

```text
subtype/relation checks
relation steps
type normalization calls where identifiable
generic inference sessions
constraints generated
constraints solved

conformance queries
conformance candidates considered
proofs attempted
derived proofs
nested evidence/proof depth

specialization sites
specializations materialized
```

P5 does not need to instrument every internal type-system function.

Prioritize choke points that represent meaningful algorithmic work.

Counters must be cheap enough for Summary or limited to Detailed mode.

---

## 29. Search-space distributions

Detailed mode should support bounded distributions for:

```text
conformance candidate count
generic constraints per inference
nested proof depth
```

At minimum record:

```text
count
sum
max
```

If an efficient bounded histogram is implemented, add:

```text
p50
p90
p99
```

Do not retain every observation in Summary mode.

---

## 30. Compiler projection instrumentation

Instrument `ProgramCompiler::project_analyzed()` or the post-rebase equivalent.

Required phases:

```text
compiler_projection.total

compiler_projection.runtime_root_discovery
compiler_projection.metadata_export

compiler_projection.runtime_type_recipes
compiler_projection.generic_evidence
compiler_projection.conditional_calls
compiler_projection.trait_environments

compiler_projection.module_lowering
compiler_projection.materialization_plans
```

Use the same phase model for:

```text
canonical Universe
ordinary project
standalone module
incremental fixture
```

The run subject determines context.

---

## 31. Conditional/specialized call projection counters

Required counters around conditional/specialized application projection:

```text
conditional application sites
specialization sites
TypeStore persistent-root forks
TypeStore post-fork mutation operations
directly observed COW work, when available
full TypeStore materializations, if any
materialize_type calls
evidence slots materialized
nested evidence slots
conformance environment recipes
```

This must be able to prove or reject the current suspicion that compiler projection performs large store copies per specialized site.

No copy elimination is allowed in P5.

---

## 32. Metadata export instrumentation

Instrument metadata export separately.

Required phases:

```text
metadata.total
metadata.project_identity_validation
metadata.declarations
metadata.aliases
metadata.callables
metadata.fields
metadata.runtime_roots
metadata.static_domains
metadata.fingerprints
```

Required counters:

```text
declarations exported
aliases exported
callables exported
fields exported
runtime roots exported

type export requests
unique type nodes emitted
type export reuse

generic signature requests
unique generic signatures emitted

callable-owner comparisons
```

The `callable-owner comparisons` counter is mandatory because current source contains declaration×callable ownership scanning.

Do not index it in P5.

---

## 33. Runtime bootstrap instrumentation

Continue to measure runtime bootstrapping after compiler-side canonical work.

Required phase tree:

```text
vm.bootstrap.total

vm.kernel
vm.native

vm.universe.source_bindings
vm.universe.static_constants
vm.universe.primitive_origins
vm.universe.metadata_install

vm.universe.bytecode_compile.total
vm.universe.initializers.total

vm.universe.source_primitives
vm.universe.alias_binding
vm.universe.semantic_roots
vm.universe.invariant_validation
```

Detailed mode should allow per-Universe-module:

```text
bytecode compilation duration
initializer duration
```

No per-opcode instrumentation is part of P5.

---

## 34. Report schema

Create a versioned machine-readable report.

Required top-level shape:

```text
schema_version
run_metadata
subject
outcome
wall duration
phases
work counters
semantic stats
query metrics
module metrics
entity metrics
structural metrics
derived metrics
```

### 34.1 Required run metadata

```text
git SHA
branch
dirty state where available
compiler/tool version
Rust toolchain
build profile
Cargo features
OS
architecture
host key where available
instrumentation level
run kind
process cold/warm state
```

### 34.2 Subject kinds

At least:

```text
CanonicalUniverse
Project
Package
StandaloneModule
Inline
IncrementalUpdate
VmBootstrap
Other
```

### 34.3 Schema evolution

- start at schema version 1;
- preserve explicit enum spellings;
- do not silently reinterpret an existing serialized field;
- add fields compatibly where possible;
- bump schema when meaning changes incompatibly.

---

## 35. Human report

Render human-readable output from the same report model.

Do not create independent ad hoc text-only metrics.

Required sections:

```text
Run identity
Outcome
Total wall time

Top phases
Semantic work
Query work
Callable analysis multiplicity
TypeStore/aggregate copying
Dependency propagation
Source traversal
Search-space metrics
Most expensive modules
Most repeated entities
Universe-vs-user work
Runtime bootstrap
Derived amplification metrics
```

Top-N limits must prevent unbounded output.

---

## 36. Derived metrics

P5 report analysis must calculate:

### 36.1 Callable reanalysis amplification

```text
total callable body analyses
----------------------------
unique callable bodies analysed
```

### 36.2 Source traversal amplification

```text
total top-level statement visits
--------------------------------
unique top-level statements
```

### 36.3 Dependency fanout

```text
visited dependency nodes
------------------------
seed nodes
```

### 36.4 Query recomputation ratio

```text
queries recomputed
------------------
queries requested
```

### 36.5 Copy amplification

Where available:

```text
entries/nodes cumulatively copied
---------------------------------
final retained entries/nodes
```

### 36.6 Universe work ratio

Prefer deterministic work units:

```text
Universe semantic work
----------------------
all semantic work
```

Wall-time ratio may also be reported where attribution is complete.

---

## 37. High-cardinality storage limits

Summary mode must not retain every event.

Aggregate by:

```text
phase
query kind
module
reason
```

Detailed mode may retain bounded:

```text
top-N expensive callables
top-N repeated callables
top-N query keys
top-N modules
top-N dependency traversals
```

Trace mode may stream detailed events to a file.

Do not keep an unbounded in-memory trace.

---

## 38. Source/privacy constraints

Reports must never contain source text by default.

Allowed identities:

```text
module identity
declaration identity/name
callable selector
query kind
source path when appropriate
```

Do not record:

```text
full source content
runtime secret values
arbitrary user data
```

A report should be reasonably safe to attach to an issue without leaking source bodies.

---

## 39. Failure-path requirements

Failed compilations must still produce profiles.

Required outcomes include:

```text
parse failure
module/link failure
semantic error
semantic update failure
metadata failure
compiler projection failure
VM bootstrap failure
cancellation
budget exhaustion
panic boundary
```

All completed phases must remain in the report.

Later phases must not be marked successful if never reached.

This requirement is mandatory because the previous performance characterization was blocked by correctness failures and still produced useful partial evidence.

---

## 40. `phalcom-perf` integration

Extend `phalcom-perf` rather than adding a parallel benchmark harness.

Required capabilities:

```text
run instrumented case
capture internal profile
store profile with sample
show internal profile
compare two profiles
```

### 40.1 Storage

A stored performance sample should be capable of containing:

```text
wall/resource metrics
+
optional compiler internal profile
```

### 40.2 Comparison

Extend comparison to report deltas for:

```text
phase durations
query recomputation
query reuse/revalidation
callable analysis count
TypeStore persistent-root fork count
directly measured TypeStore COW/full-copy volume, when available
snapshot/aggregate materialization count and directly measured volume
source traversal
dependency fanout
Universe semantic work
metadata comparisons
```

### 40.3 Structural comparison

A structural regression must remain visible even if wall time is noisy.

Example:

```text
wall time: +1%
callable analyses: +72%
```

The comparison must surface this.

P5 does not need to enforce CI thresholds yet.

---

## 41. Required benchmark suite

After instrumentation lands and `main` is correct, define stable P5 characterization cases.

### B1 — canonical Universe cold process

Fresh process:

```text
canonical_universe_program()
```

### B2 — canonical semantic baseline cold construction

If cleanly isolatable.

### B3 — semantic session instantiation

Warm canonical baseline:

```text
SemanticWorkspaceSession::new()
```

multiple samples.

### B4 — tiny ordinary project/package compile

Minimal program.

### B5 — tiny standalone compile

Covers standalone Universe support path.

### B6 — medium ordinary package

Representative non-Universe workload.

### B7 — feature-dense package

Exercise:

```text
generics
traits
conformances
associated members
records/packs
static/type-level features
```

### B8 — incremental updates

Same session, variants:

```text
small body edit
signature edit
declaration addition
import edit
```

### B9 — warm full VM

Canonical compiler product already initialized, repeated:

```text
VM::new()
```

---

## 42. Workload cardinalities

Every compiler profile should capture cheap workload cardinalities where available:

```text
source bytes
module count
declaration count
callable count
impl count
trait count
generic declaration count
query product count
TypeStore type count
```

These enable normalized comparisons and future scaling analysis.

Do not add expensive whole-workspace scans solely to compute cardinalities if the values are already naturally known during compilation.

---

## 43. Instrumentation-overhead gate

P5 must benchmark its own overhead.

Run:

```text
A. feature disabled
B. feature compiled, runtime Off
C. Summary enabled
D. Detailed enabled
```

Against at least:

```text
tiny package compile
medium package compile
warm incremental update
```

### Acceptance targets

Feature disabled:

```text
effectively zero instrumentation overhead
```

Compiled + Off:

```text
target ≤ 1–2% median wall overhead
```

Summary:

```text
target ≤ 5% median wall overhead
```

Detailed:

```text
diagnostic; no strict low-overhead guarantee
```

Trace:

```text
diagnostic only; timings not benchmark-grade
```

If Summary exceeds the target materially, P5 must optimize the instrumentation implementation before acceptance or escalate with evidence.

---

## 44. Deterministic testing requirements

Never assert timing durations in unit tests.

Assert structural accounting only.

### 44.1 Query lifecycle tests

Test:

```text
first query:
  requested 1
  recomputed 1

same cached query:
  reuse increments
  recomputed unchanged

new revision, unchanged dependencies:
  revalidation increments
  recomputed unchanged

changed dependency:
  recomputed increments
```

### 44.2 Module attribution

Two-module fixture.

Verify work belongs to correct modules.

### 44.3 Callable multiplicity

Use a fixture known to trigger a controlled reanalysis path.

Verify counts and reason classification.

### 44.4 TypeStore fork and copy accounting

Create known baseline-fork, post-fork mutation, and actual full-copy/materialization
paths. Assert that a persistent-root fork is not counted as copied nodes.

Verify:

```text
fork reason and count
baseline logical cardinalities
mutation-operation count
direct COW node/byte count, if exposed, otherwise explicit unavailable state
actual full-copy reason and count, if present
snapshot map materialization count, separately
attempt outcome and committed-publication attribution
```

### 44.5 Dependency traversal

Use deterministic graph.

Verify:

```text
seeds
visited nodes
result count
reason
```

### 44.6 Failure profiles

Compile invalid source.

Verify:

```text
report exists
outcome is failure
completed phases retained
unreached phases absent
```

### 44.7 JSON schema

Round-trip:

```text
serialize
deserialize
schema_version preserved
enum values preserved
```

### 44.8 Parallel isolation

Run two independent profiles concurrently or in parallel tests.

Verify metrics do not leak between recorders.

---

## 45. Compatibility tests

Required builds:

```bash
cargo check -p telos-observability --all-targets

cargo check -p phalcom-semantic --all-targets
cargo check -p phalcom-semantic --all-targets --features perf-observability

cargo check -p phalcom-core --all-targets
cargo check -p phalcom-core --all-targets --features perf-observability
cargo check -p phalcom-core --all-targets --features test-timings
```

Use the repository's pinned toolchain and normal `RUSTFLAGS`/wrapper conventions.

If feature propagation differs after the re-audit, update commands accordingly.

---

## 46. Core regression gates

P5 must run focused correctness suites proving instrumentation does not alter semantics.

At minimum:

```text
semantic incremental DB tests
semantic query ownership/reuse tests
ProgramCompilationSession tests
canonical Universe singleton/reuse tests
VM bootstrap focused tests
language corpus representative lane
```

The exact broad gate must be chosen from the post-type-system clean repository state.

Do not hide pre-existing baseline failures.

Record clean-predecessor/baseline dispositions where necessary.

---

## 47. Phase reconciliation tests

For selected successful runs:

```text
sum of direct child phase durations
≤ parent duration + measurement tolerance
```

Do not require exact equality because:

```text
unattributed code
timer resolution
overlap/concurrency
```

may exist.

Report residual:

```text
parent duration - child duration sum
```

A large unexplained residual must remain visible.

---

## 48. Implementation task sequence

### T0 — Baseline re-audit and plan synchronization

Read:

```text
AGENTS.md
workflow/lifecycle conventions
PERF003 checkpoint/plan/incident records
current instrumentation source
semantic DB/query/session source
canonical Universe source
compiler projection
metadata export
VM bootstrap
phalcom-perf
```

Record:

```text
HEAD
branch
status
toolchain
feature topology
current baseline test state
```

Update source anchors in the plan if needed.

**Gate T0:** implementation work may begin only after repository-state record exists.

**T0 result:** complete. `BASELINE_HEAD`, execution-tree HEAD, branch, worktree,
toolchain, current feature-gated harness, source metric semantics, transaction
commit/rollback boundary, and the two current-main baseline blockers are recorded
in §4 and the checkpoint. T1 began only after this state was recorded.

---

### T1 — Create `telos-observability`

Add crate to workspace.

Implement:

```text
ObservabilityLevel
PerfRecorder
PerfContext
PerfSpanGuard
run metadata
phase taxonomy
basic counters
report schema
JSON serialization
```

Add isolated unit tests.

No compiler instrumentation yet except smoke integration.

**Gate T1:** crate compiles independently; round-trip report tests pass.

---

### T2 — Feature propagation and compatibility bridge

Add:

```text
perf-observability
```

features.

Bridge:

```text
test-timings
```

to new feature where appropriate.

Keep existing `FixtureTiming`/bootstrap counters working.

**Gate T2:** default and instrumented builds compile.

---

### T3 — Top-level compiler/Universe spans

Instrument:

```text
canonical_universe_program
ProgramAnalyzer
ProgramCompilationSession
ProgramCompiler
VM bootstrap
```

Add process-cold context installation.

Produce first coarse report.

**Gate T3:** a cold canonical run produces a valid hierarchical profile.

---

### T4 — Canonical semantic baseline spans

Instrument baseline phases and source traversal counts.

**Gate T4:** canonical baseline work is independently visible from full semantic update.

**T4 result:** implemented and build-checked in the default and
`perf-observability` semantic configurations. The direct `semantic-baseline`
profile completed independently of VM startup and emitted schema-v1 spans for
all required baseline phases. On the recorded run, the cold baseline took
265.585 ms; class surfaces accounted for 73.193 ms, linked-world setup for
56.414 ms, native surfaces for 17.114 ms, callable-signature work for 11.229 ms,
and inherent-impl construction for 9.853 ms. The report counted 888 module scan
visits across phase passes and 5,352 top-level statement visits. The full profile
is `samples/PERF003.C1.P5-T4-semantic-baseline.json`. These are a single
instrumented sample, not a comparative benchmark. No tests were added or run;
T1's report round-trip test gate remains outstanding.

---

### T5 — Semantic update phase ledger

Add stable phase spans across `SemanticWorkspaceSession` update.

Integrate `SemanticUpdateStats`.

**Gate T5:** ordinary and canonical semantic updates show phase breakdown and work counts.

**T5 result:** implemented and build-checked in default and
`perf-observability` semantic configurations. The profile example now supports
ordinary and canonical source-backed semantic-update workloads. The canonical
update completed and emitted schema-v1 baseline plus semantic-update spans and
committed work counters. Its single instrumented development-profile sample
measured 69.161 s in `semantic.update.total` and 69.710 s process elapsed;
callable-signature spans totaled 18.388 s, final body revalidation 25.798 s,
declaration surfaces 6.083 s, initial callable bodies 4.568 s, and source index
2.668 s. The committed update counters include 148 modules recomputed, 1,747
callable signatures recomputed, 1,238 callable bodies recomputed, 9,318 query
products recomputed, eight whole-map snapshot materializations, and one commit.
The ordinary one-module update also completed; its `semantic.update.total`
sample was 2.327 s and included 2,650 recomputed query products. Exact reports:
`samples/PERF003.C1.P5-T5-canonical-semantic-update.json` and
`samples/PERF003.C1.P5-T5-semantic-update.json`. Both name
`BASELINE_HEAD=e2ae4a0fd360e4075ce8ad229a3e39e1531fb771` and execution-tree
HEAD `303222c3b027c34ddacd4749f056e1095aabfa81`. These are single instrumented
samples, not comparative benchmarks. No tests were added or run; T1's report
round-trip test gate remains outstanding.

---

### T6 — Query instrumentation

Add:

```text
QueryKind
query lifecycle accounting
recompute timing
module attribution
```

Reconcile `QueryMetrics`.

**Gate T6:** deterministic query lifecycle tests pass.

**T6 result:** implemented and build-checked in default and
`perf-observability` semantic configurations; the core profile example also
build-checks with observability enabled. Every current `QueryKey` variant maps
exhaustively to a stable `QueryKind`. Schema v2 reports aggregate request,
direct reuse, revalidated reuse, successful product recomputation, invalidation,
blocked, cancelled, budget-exhausted, and failed events by query kind. Detailed
profiles split those aggregates by owning module/entity. Recompute duration is
reported as observed total and maximum; failed, blocked, cancelled, and
budget-exhausted attempts that entered recomputation contribute their observed
duration. V1 reports remain readable with an empty query-metric field.

`QueryMetrics::hits` now means successful cached product retrievals, while
`misses` counts completed query recomputation attempts; publishing a new query
product no longer increments `hits`. The report-level query metrics describe
work observed during attempts, including work later rolled back. T5's committed
`SemanticUpdateStats` remain the authority for successful publication work.

The ordinary and canonical source-backed update samples are
`samples/PERF003.C1.P5-T6-semantic-update.json` and
`samples/PERF003.C1.P5-T6-canonical-semantic-update.json`. Both name
`BASELINE_HEAD=e2ae4a0fd360e4075ce8ad229a3e39e1531fb771` and execution-tree
HEAD `303222c3b027c34ddacd4749f056e1095aabfa81`. The ordinary run's update
span was 1.464 s and query accounting reported 2,774 requests, 115 direct
reuses, 2,659 product recomputations, and 9 invalidations. The canonical run's
update span was 73.084 s and reported 82,838 requests, 70,918 direct reuses,
12,797 product recomputations, and 3,371 invalidations. It also recorded one
failed advisory-callable query while the enclosing semantic update committed
successfully; the report does not carry a failure reason, so this event remains
unclassified. Canonical query-kind recompute-duration totals were highest for
declaration surfaces (3.450 s), callable bodies (2.534 s), and callable
signatures (195 ms). Query durations are inclusive and can overlap when queries
are nested; their sum is not process wall time. These are single instrumented
samples, not comparative benchmark claims. No tests were added or run; the
deterministic query lifecycle test gate remains outstanding.

---

### T7 — Callable multiplicity and dependency propagation

Add:

```text
callable body reason/multiplicity
reverse closure traversal metrics
```

**Gate T7:** reanalysis and graph amplification reports are generated.

**T7 result:** implemented and build-checked in default and
`perf-observability` semantic configurations; the profile example also
build-checks with observability enabled. Semantic update profiles now count
callable-body query attempts by initial, downstream-signature, cycle-retraction,
conformance-change, module-fact, and final inferred-result reanalysis passes.
Every reverse-closure traversal records input seed entries, unique nodes visited,
and reverse dependency edges examined.

The canonical sample is
`samples/PERF003.C1.P5-T7-canonical-semantic-update.json`. It names the pinned
baseline and execution-tree HEAD. This first-session source-backed workload
recorded 1,238 initial-pass body queries, 6 conformance-change pass queries,
1,035 module-fact pass queries, and 7,527 final inferred-result body analyses.
Downstream-signature and cycle-retraction query counters were zero in this cold
workload; it did not exercise an edited follow-up revision. Across all reverse
closures, the report counted 1,957 seed entries, 3,393 unique visited nodes,
and 7,889 reverse edges examined. The enclosing update committed successfully.
This is a single instrumented profile, not a comparative benchmark. No tests
were added or run.

---

### T8 — TypeStore fork/COW and aggregate-materialization accounting

Instrument:

```text
TypeStore baseline forks, post-fork mutation operations, and directly observable COW work
whole-map snapshot materializations
major aggregate semantic copies
attempted versus committed publication outcome and rollback-journal work
```

No optimizations.

**Gate T8:** report identifies fork counts, actual measured copy volume where available,
map materialization counts, copy reason, and publication outcome without conflating
logical writes, persistent-root forks, or successful publication work.

**T8 result:** implemented and build-checked with the default and
`perf-observability` semantic configurations. The ordinary semantic-update
profile in `samples/PERF003.C1.P5-T8-semantic-update.json` recorded one
persistent-root fork containing 1,194 type, 11 kind, 136 parameter, 5 row, 1
static-domain, and 60 variant entries. It recorded 22 logical post-fork
mutation operations (15 type, 6 parameter, 1 row), one semantic update attempt
and commit, eight snapshot materialization attempts, and DB undo-journal counts
of 2,650 query-state, 110 module-key, 2,650 product, 2,650 last-known-good,
2,650 forward-dependency, and 6 reverse-dependency entries. Physical COW
node/byte work and per-map materialization identity, volume, and duration are
explicitly unavailable; the logical mutation count is not a proxy for physical
copies. Profile elapsed time was approximately 1.86 seconds for this single
sample. The T8 deterministic lifecycle/rollback test gate was not run.

---

### T9 — Source traversal and search-space counters

Add:

```text
source traversal amplification
type relation counters
generic inference counters
conformance search counters
```

Keep Summary-mode overhead bounded.

**Gate T9:** structured search-volume data exists without semantic changes.

**T9 result:** implemented and build-checked in default and
`perf-observability` configurations. The canonical semantic-update profile in
`samples/PERF003.C1.P5-T9-canonical-semantic-update.json` recorded 148 unique
modules and 669 unique top-level statements, with 888 module visits and 5,352
top-level statement visits (8.0x statement traversal amplification). It also
recorded 4,812 subtype checks, 30,903 assignability checks, 5,903 recursive
relation steps, and 28,416 projection/native normalization calls. Generic
inference recorded 625 sessions, 1,167 generated constraints, 1,560
propagation/solve calls, and 2,764 visited constraints. Conformance search
recorded 329 exact queries, 35,861 eligible candidates considered, and 210
matching heads; generic conformance proof recorded 128 proof attempts and 11
derived proofs. Detailed distributions reported candidate count per query
(count 329, sum 35,861, max 109), constraints per solve (count 1,560, sum
2,444, max 5), and nested proof depth (count 128, sum 149, max 2). Elapsed
profile time was approximately 72.4 seconds for this single instrumented
canonical update. These counts describe work on the canonical source workload;
they do not establish comparative performance or prove all ordinary-project
traversals are instrumented. No tests were added or run.

---

### T10 — Compiler projection instrumentation

Instrument:

```text
runtime roots
metadata export
type recipes
generic evidence
conditional calls
trait environments
module lowering
materialization plans
```

Add conditional/specialization counters.

**Gate T10:** canonical and ordinary projections share one phase model.

**T10 result:** phase/counter instrumentation is implemented and build-checked
for the core crate in default and `perf-observability` configurations. Both
ordinary compilation and canonical bootstrap enter the same
`ProgramCompiler::project_analyzed()` path, so they share the projection phase
model. The inline compiler profile in
`samples/PERF003.C1.P5-T10-compiler-projection.json` reached runtime-root
discovery (5.85 ms) and metadata export (24.69 ms), then stopped at the known
baseline error `legacy metadata cannot represent a Record remainder outside
the Symbol key domain`. The compiler projection span ended as `other_failure`;
later type-recipe, evidence, trait-environment, module-lowering, and
materialization-plan phases were not reached. The profile's total wall time was
44.6 seconds, dominated by the preceding inline analysis. Its simple fixture
did not produce nonzero projection counters before the metadata failure. Thus
the shared phase model is implemented, but end-to-end projection measurements
remain baseline-blocked. No tests were added or run.

---

### T11 — Metadata export instrumentation

Add metadata phases and work counters, including:

```text
callable-owner comparisons
type export requests/reuse
signature export requests/reuse
```

**Gate T11:** metadata cost and structural work are independently visible.

**T11 result:** metadata phase spans and structural counters are implemented
and build-checked in default and `perf-observability` configurations. The
inline profile reached project-identity validation (0.18 ms), declaration
export (22.41 ms), and alias export (0.42 ms), where the existing
Record-remainder limitation stopped the bundle. Before failure it exported 279
declarations and recorded 494,667 callable-owner comparisons, 427 type-export
requests, 287 unique emitted type nodes, 139 type reuse hits, and 64 generic
signature requests/64 unique signatures. Callables, fields, runtime roots,
static domains, and fingerprint completion were not reached in this workload.
The owner-comparison total reflects current nested scan work; it is measurement
evidence only and does not authorize a P5 optimization. This sample is a single
run and baseline-blocked. No tests were added or run.

---

### T12 — `phalcom-perf` report integration

Store internal compiler profiles with samples. Support standalone profile-only
samples as well as sidecars attached to external process-level runs.

Implement:

```text
show profile
compare profile
top-N summaries
derived amplification metrics
```

**Gate T12:** two stored runs can be compared structurally and temporally.

**T12 result:** `phalcom-perf profile capture`, `attach`, `show`, and `compare`
are implemented behind `perf-observability`. Capture stores a validated
profile-only elapsed-time sample; attach stores a sidecar beside an external
process-level sample and rejects source-revision or dirty-state mismatches.
Human and JSON summaries include bounded top-N phase/counter views, query
lifecycle totals, phase/counter/query deltas, and derived source-traversal,
dependency-fanout, and query-recomputation ratios. Callable-body and copy
amplification are explicitly unavailable where the report lacks a valid
denominator or physical-copy measurement. Two separately captured B2 reports
for the pinned baseline compared as structurally compatible: the observed
single-pair internal elapsed times were 76.9 ms and 54.5 ms. This is CLI-path
evidence only, not a performance conclusion. `profile show`, `capture`, and
`compare` were exercised; no test suite was run. Focused default and
feature-enabled `phalcom-perf` build checks passed. Gate T12 is complete at this
bounded scope.

---

### T13 — Benchmark suite and cold/warm harness

Add stable P5 benchmark subjects B1–B9.

Ensure fresh-process cases genuinely launch fresh processes.

**Gate T13:** cold/warm classifications are reproducible.

**T13 result:** the release-mode runner starts a fresh OS process per sample,
records exit codes, and requires the pinned `BASELINE_HEAD`. The report schema
now carries an optional typed lifecycle classification. B1–B6 have workload
entry points or documented candidates: B1 and B4 remain baseline-blocked; B2
has a completed earlier profile; B3 warms the semantic baseline outside the
recorder before measuring session creation; B5 compiles the standalone module
fixture; B6 uses a four-module project candidate whose medium-size status still
needs acceptance. B7 and B8 workloads are not implemented, and B9 is blocked by
B1. The A/B/C/D shell driver is available. Schema, CLI, and example build
checks passed, and both scripts passed syntax validation. No sample was run, so
Gate T13 remains unverified and the B1–B9 suite is incomplete.

---

### T14 — Observer-overhead characterization

Run A/B/C/D modes.

Record results.

If Summary overhead is too high, optimize observability implementation only.

**Gate T14:** overhead acceptance documented.

**T14 result:** the runner can collect `off`, `summary`, `detailed`, and `trace`
profiles in separate fresh processes for one named workload, with the pinned
baseline and lifecycle written into each report. The matrix has not been run;
no observer-overhead percentage or acceptance claim is available. Gate T14 is
unverified.

---

### T15 — Final acceptance characterization

Run:

```text
cold canonical Universe
tiny project
medium project
feature-dense project
incremental edits
warm VM bootstrap
```

Generate complete reports.

Do not optimize compiler behavior based on results within P5.

**Gate T15:** P5 produces sufficient evidence to plan P6.

**T15 result:** not run. Required full compiler projections and VM startup
remain blocked by the baseline alias-export and ProductKind source-index
failures; B7/B8 are also not yet implemented. Do not treat the partial T4–T11
profiles as the final acceptance matrix. Gate T15 is unverified.

---

## 49. Deliverables

P5 must produce or update at least:

### Source

```text
telos-observability/
  Cargo.toml
  src/
    lib.rs
    context.rs
    phase.rs
    metric.rs
    recorder.rs
    report.rs
    serialize.rs
```

Exact file organization may vary.

### Integration

Updated:

```text
workspace Cargo.toml
phalcom-semantic/Cargo.toml
phalcom-core/Cargo.toml
semantic instrumentation/query/session code
canonical Universe compiler path
ProgramAnalyzer / ProgramCompilationSession
ProgramCompiler projection
metadata exporter
VM bootstrap
phalcom-perf
```

### Lifecycle/docs

Create/update:

```text
PERF003.C1.P5-detailed-implementation-plan.md
PERF003.C1.P5-status.md
PERF003.C1.P5-guidance.md
PERF003.C1.P5-benchmark-methodology.md
PERF003.C1.P5-observability-schema.md
PERF003.C1.P5-overhead-report.md
PERF003.C1.P5-characterization-handoff.md
PERF003.C1-CHECKPOINT.md
```

Use repository lifecycle conventions after re-audit.

### Evidence

Store:

```text
machine-readable profile samples
benchmark commands
build/profile metadata
overhead measurements
comparison output
focused test results
```

Do not commit large unbounded trace files unless repository conventions explicitly allow it.

---

## 50. Acceptance characterization questions

At P5 completion, evidence must be sufficient to answer:

1. how much cold canonical time is source/module work?
2. how much is canonical semantic baseline work?
3. how much is full Universe semantic update?
4. how much is compiler projection?
5. how much is metadata export?
6. how much is VM/native bootstrap?
7. how much is Universe bytecode compilation?
8. how much is initializer execution?
9. how many Universe callable bodies exist?
10. how many total callable analyses occur?
11. which reanalysis wave contributes most?
12. which modules cause the most semantic work?
13. how many TypeStore persistent-root forks and actual full copies occur, and why?
14. what post-fork mutation work and directly observable physical COW volume occur?
15. how many whole-map materializations occur, what content volume is directly
    available, and how much large-table copy amplification can be calculated?
16. how large are dependency-graph closures?
17. how much source traversal amplification exists?
18. which QueryKinds dominate recomputation?
19. which QueryKinds dominate recompute time?
20. how much Universe work occurs in an ordinary tiny package compile?
21. how much generic/conformance search work occurs?
22. how much work occurs in successful versus failed compilation?
23. what remains unexplained by timed child phases?
24. what is the measurement overhead of the profiler itself?

If these questions cannot be answered, P5 is incomplete.

---

## 51. Stop/consultation conditions

Stop and request architectural review before proceeding if implementation would require:

- changing semantic query semantics;
- changing invalidation propagation;
- moving semantic ownership solely to support profiling;
- changing canonical identity rules;
- changing TypeStore storage or ID semantics;
- changing compiler/runtime dependency layering;
- introducing a global recorder that prevents concurrent compilation;
- adding instrumentation whose Summary overhead is materially above the target and cannot be reduced without semantic restructuring;
- changing failure/cancellation behavior;
- changing metadata semantics;
- changing VM bootstrap behavior;
- introducing an artifact/cache to make instrumentation practical;
- redesigning `SemanticDb` products solely for profiling;
- crossing the previously blocked runtime-image/T7 ownership boundary.

P5 must prefer partial observability with explicit gaps over an architectural change disguised as instrumentation.

---

## 52. Non-regression invariants

P5 must preserve:

```text
same semantic diagnostics
same semantic products
same stable IDs
same TypeStore prefix semantics
same query invalidation decisions
same callable analysis results
same metadata contents
same lowering contents
same compiled program behavior
same VM bootstrap behavior
same test outcomes
```

Instrumentation may add:

```text
timing
counts
attribution
reporting
```

only.

---

## 53. Definition of done

P5 is complete only when:

- `telos-observability` exists as the shared observability substrate;
- canonical Universe compilation produces a versioned hierarchical profile;
- ordinary package compilation produces the same style of profile;
- incremental semantic updates expose reuse/recompute behavior;
- query work is classified by `QueryKind`;
- callable analysis multiplicity is visible;
- module attribution works;
- Universe work inside ordinary compilation is measurable;
- TypeStore persistent-root forks, post-fork mutations, actual full copies, and
  directly observable physical COW volume are represented as separate metrics;
- whole-map snapshot materializations and major aggregate copies are measurable
  without being attributed to TypeStore forks;
- major aggregate-copy/materialization amplification is measurable where direct
  counts and retained-size denominators are available, with unavailable values
  represented explicitly;
- dependency fanout is measurable;
- source traversal amplification is measurable;
- generic/conformance search volume is measurable;
- compiler projection and metadata export are independently visible;
- VM bootstrap remains independently visible;
- failed compilations still yield valid partial profiles;
- reports are machine-readable and versioned;
- `phalcom-perf` stores and compares profiles;
- instrumentation-off behavior remains unchanged;
- instrumentation overhead is quantified and accepted;
- no compiler optimization is mixed into the baseline;
- final characterization evidence is sufficient to author P6 from measurements rather than hypothesis.

---

## 54. Expected P5 terminal output

After P5, one cold Universe run should be capable of producing evidence of this form:

```text
Telos Compiler Performance Profile
==================================

Subject: canonical Universe
Outcome: success
Modules: 148
Total: 18.422 s

Top phases
----------
semantic workspace update              10.219 s   55.5%
canonical semantic baseline             3.488 s   18.9%
compiler projection                     3.107 s   16.9%
source/link/native                      0.944 s    5.1%
other                                   0.664 s    3.6%

Semantic query work
-------------------
requests                                18,412
recomputed                               7,384
revalidated                              4,803
direct reuse                             6,225

Callable bodies
---------------
unique                                     812
total analyses                             1,447
reanalysis amplification                   1.78x

TypeStore
---------
baseline persistent-root forks                1
fork duration                         measured
TypeStore snapshot deep copies                0
post-fork mutation operations          measured
physical COW node/byte copies       unavailable unless directly observable

Semantic publication
--------------------
whole-map materializations                    8
attempt outcomes             committed / rolled-back / failed-before-start
undo-journal activity        per category where directly observable

Dependency propagation
----------------------
largest reverse closure                  3,114 nodes
seeds                                         7
fanout amplification                     444.9x

Source traversal
----------------
unique top-level statements              3,012
statement visits                        24,991
traversal amplification                   8.30x

Most expensive modules
----------------------
universe.collections...                   ...
universe.numbers...                       ...
universe.text...                          ...
```

The exact values are illustrative only.

The required outcome is the ability to produce this quality of causal performance evidence.

---

## 55. Architectural outcome expected from P5

P5 does not decide the optimization.

P5 makes later decisions evidence-driven.

Likely P6 candidates include, depending on measured results:

```text
canonical Universe single-pass semantic construction
removal of Universe source reanalysis from ordinary packages
TypeStore overlay/fork redesign
aggregate semantic snapshot persistence
query invalidation/reanalysis reduction
conformance-search optimization
metadata export indexing
compiler projection materialization reuse
runtime bootstrap optimization
```

No candidate is authorized merely because it currently looks suspicious.

P5 exists specifically to rank them quantitatively.

---

## 56. Final execution rule

**Measure first. Preserve the baseline. Optimize later.**

The implementer must resist opportunistic performance fixes encountered while wiring instrumentation.

Every such finding should be recorded as:

```text
candidate optimization
source location
observed metric
suspected complexity
future plan recommendation
```

and left unchanged until P5 characterization is complete.

Only then should P6 be selected.
