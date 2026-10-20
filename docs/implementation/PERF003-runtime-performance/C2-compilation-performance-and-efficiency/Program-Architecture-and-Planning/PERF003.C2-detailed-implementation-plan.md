---
id: PERF003.C2
category: PERF
program: PERF003
checkpoint: PERF003.C2
kind: patch-grade-implementation-plan
title: "Compilation Performance and Efficiency — Canonical Universe Image, Overlay Pipeline, Demand-Driven Publication, and Incremental Conformance"
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared_against_repository: phalcom
prepared_against_branch: perf/universe-characterization
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head: 303222c3b027c34ddacd4749f056e1095aabfa81
depends_on:
  - PERF003.C1
follows: null
supersedes: null
---

# PERF003.C2 — Detailed Patch-Grade Implementation Plan

## 0. Executor contract

This plan is designed for execution under constrained architectural authority.

The implementer:
- may adapt mechanical details (private helper names, module decomposition, imports, local borrow cleanups) to the live tree;
- must preserve the architecture, ownership boundaries, and invariants specified below;
- must not improvise material architecture or create ad-hoc caching shortcuts;
- must follow the testing and debugging budget rather than testing reflexively after every edit;
- must classify and defer unrelated baseline failures under the A/B/C/D failure policy;
- must STOP AND CONSULT when an escalation condition fires;
- must keep `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md` updated at meaningful milestones;
- must produce the required walkthrough and handoff deliverables upon completion.

Testing is evidence gathering, not a ritual. Do not run broad test suites merely because an edit occurred.

---

## 1. Goal

Eliminate the ~45-second Universe-dominated compilation floor across all one-shot and incremental compilation entry points, replacing whole-Universe reconstruction with an immutable, process-shared `CanonicalUniverseCompilationImage`, a lightweight user overlay pipeline, demand-driven product publication, indexed conformance settlement, and root-driven metadata reachability.

---

## 2. Checkpoint acceptance objective

This plan advances and fulfills the bounded objective of `PERF003.C2`:

> **A 500-byte program must compile as a 500-byte overlay on an immutable Universe, not as a request to reconstruct a substantial fraction of the Universe compiler state.**

The objective is considered achieved when:
1. Warm process one-shot compilation of tiny fixtures (345 B to 7.5 KB) scales with user code size rather than clustering around a ~45-second constant.
2. The compilation floor is structurally removed: structural cardinality counters in `telos-observability` prove **zero** canonical Universe reparses, **zero** canonical interface rebuilds, **zero** canonical callable-body re-analyses, **zero** presentation rebuilds in runtime mode, and **zero** global metadata declaration-by-callable sweeps.
3. Strict semantic parity is maintained: compiled bytecode, diagnostics, typing, conformance evidence, and runtime execution behavior remain identical to clean cold compilation.

---

## 3. Repository grounding

Prepared against:
```text
repository:             phalcom
worktree:               /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization
branch:                 perf/universe-characterization
main_baseline (SHA):    e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head:    303222c3b027c34ddacd4749f056e1095aabfa81
toolchain:              nightly-2026-07-10 (pinned in rust-toolchain.toml)
```

Verified repository facts:
1. `telos-observability` is active and equipped with comprehensive counters, phases, and query kinds across semantic update, compiler projection, and metadata export.
2. `UniverseSourceDeclarationCatalog::build` currently iterates all 40+ canonical modules and invokes `InterfaceBuilder::build` for each module on every call (`phalcom-modules/src/builtin_interface.rs:30-45`).
3. `ModuleResolver::load_interface` delegates directly to `BuiltinInterfaceBuilder::build_from_parsed`, bypassing the static `interface_cache()` (`phalcom-modules/src/resolver.rs:721-725`).
4. `ProgramCompiler::compile_entry_selection(EntrySelection::Inline)` parses inline source, manually creates a resolver, filters and loads 40 Universe modules into an empty `SemanticWorkspaceInput`, and runs `analyze_workspace` with `last_snapshot = None` (`phalcom-core/src/modules/compile.rs:556-630`).
5. `build_canonical_universe_program()` already computes a complete semantic analysis of the entire canonical Universe via `analyze_workspace`, but stores only `CompiledProgram` and discards the rich `SemanticSnapshot` (`phalcom-core/src/universe/canonical.rs`).
6. `SemanticSnapshot::new_with_callable_analyses` eagerly builds a `FormalSemanticProjection` that is immediately overwritten by caller-provided projections.
7. `SourceSemanticIndex::build_module` runs `TypeReferenceTargetCollector` three independent times over the same AST.
8. `AdvisoryWorkspace::update` runs a whole-workspace fixed-point solver across all 41 semantic modules unconditionally during runtime compilation.
9. `MetadataExporter` scans all declarations and all callables in the global snapshot, ignoring `MetadataProfile`.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

```text
- AGENTS.md
- docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/COMPILATION-1.md
- docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/COMPILATION-2.md
- docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md
- docs/workflow/luna-patch-grade-plan-schema.md
- docs/workflow/shared-consultation-escalation-protocol.md
- phalcom-modules/src/builtin_interface.rs
- phalcom-modules/src/resolver.rs
- phalcom-semantic/src/canonical_baseline.rs
- phalcom-semantic/src/session.rs
- phalcom-semantic/src/workspace.rs
- phalcom-core/src/modules/compile.rs
- phalcom-core/src/universe/canonical.rs
- phalcom-core/src/metadata.rs
- telos-observability/src/metric.rs
```

---

## 5. Normative authority

This plan is subordinate to and governed by:
1. `docs/specs/README.md` and related language specification documents for static semantics, typing, conformance, and runtime behavior.
2. `docs/workflow/implementation-record-lifecycle-convention.md` for checkpoint and plan lifecycles.
3. Analysis documents `COMPILATION-1.md` and `COMPILATION-2.md`.

Distinct authority layers:
- **Normative language rules:** Conformance dispatch, type inference, method resolution order, metadata reflection contracts, and diagnostic locations. Performance optimizations must preserve exact semantic equivalence.
- **Current implementation:** Ephemeral module sessions, `TypeStore::fork_from_baseline()`, `SemanticDb` undo journals, and `telos-observability`.
- **Target architecture:** `CanonicalUniverseCompilationImage`, user overlays, demand-driven product publication, and root-driven metadata reachability.

---

## 6. Takeover state

The implementation inherits the following stable components:

| Concept | Symbol / Path | Owner | Invariant |
|---|---|---|---|
| Pinned Toolchain | `rust-toolchain.toml` | Root | `nightly-2026-07-10` pinned |
| Performance Observability | `telos-observability` | Root crate | Low overhead, hierarchical spans, typed counters, run-scoped recorders |
| Persistent Type Store | `TypeStore::fork_from_baseline` | `phalcom-semantic` | Fork assigns new `TypeStoreId`, preserves all canonical prefix `TypeId` values |
| Transactional Semantic DB | `SemanticDb` | `phalcom-semantic` | Session-local undo journal; commit on success, rollback on error |
| Canonical Universe Baseline | `CanonicalUniverseSemanticBaseline` | `phalcom-semantic` | Process-shared `OnceLock`; partial formal tables |
| Retained Session Precedent | `ProgramCompilationSession` | `phalcom-core` | Replaces fixture text via `SetOverlayForModule` without re-seeding Universe sources |
| Canonical VM Program | `canonical_universe_program()` | `phalcom-core` | Process-shared `OnceLock`; compiles full canonical Universe bytecode |

---

## 7. Architecture

The target compiler architecture decouples the canonical Universe from user program compilation, organizing execution as a demand-driven product pipeline:

```text
                     ┌──────────────────────────────────────────────┐
                     │     Canonical Universe Compilation Image     │
                     │       (Process-Shared, Once, Immutable)      │
                     │                                              │
                     │ - Parsed modules & source locations          │
                     │ - Canonical interface set (148 modules)      │
                     │ - UniverseSourceDeclarationCatalog           │
                     │ - Topology & linked program                  │
                     │ - Finalized TypeStore prefix (frozen)        │
                     │ - Declarations, hierarchy, dispatch surfaces │
                     │ - ConformanceIndex & witness plans           │
                     │ - Trait dispatch index & derived rules       │
                     │ - Reusable canonical query image (roots)     │
                     │ - Canonical metadata pool (Pool 0)           │
                     │ - Optional canonical presentation shards     │
                     └──────────────────────┬───────────────────────┘
                                            │ immutable dependency
                                            │
                             ┌──────────────▼──────────────┐
                             │    User Compilation Overlay │
                             │                             │
                             │ - Synthetic/user module     │
                             │ - Forwarded interfaces      │
                             │ - Forked TypeStore prefix   │
                             │ - Overlay query database    │
                             │ - Delta semantics only      │
                             └──────────────┬──────────────┘
                                            │
                             demand-driven product request
                               (CompilationProductRequest)
                                            │
                     ┌──────────────────────┼──────────────────────┐
                     ▼                      ▼                      ▼
           Executable Lowering     Diagnostics Product     Tooling / Presentation
           - minimal source index  - when requested        - presentation shards
           - runtime roots only    - caller diagnostic lv    (tooling / check mode)
           - local metadata pool   - no advisory in run
             (references Pool 0)
```

### Key Architectural Concepts

1. **`CanonicalUniverseCompilationImage`**: A single authoritative immutable artifact constructed once per process. It supersedes the partial `CanonicalUniverseSemanticBaseline` and the discarded snapshot in `canonical_universe_program()`, retaining all formal products.
2. **Revision Ancestry (`Revision U -> Revision U+1`)**: A fresh one-shot compilation treats `CanonicalUniverseCompilationImage` as `last_snapshot = Some(CanonicalUniverse)`. The user module is the only added module. Changed canonical modules = 0, added user modules = 1.
3. **Overlay Module Session**: `ProgramCompiler::compile_entry_selection` internally runs an ephemeral overlay session using `WorkspaceModuleSession` and `SetOverlayForModule`, eliminating the 40-module source injection.
4. **Demand-Driven Products (`CompilationProductRequest`)**: Compilation declares what it needs (`RuntimeMinimal`, `Tooling`, `Check`, `Proof`). Runtime execution compilation bypasses whole-Universe source presentation indexing and disables whole-workspace advisory analysis.
5. **Indexed Conformance Settlement**: Conformance witness lookup uses `ConformanceWitnessIndex` (keyed by `(ImplId, CallableSide, Selector)`), replacing linear scans over all global callables.
6. **Root-Driven Metadata Reachability**: Metadata export executes a reachability closure from actual runtime roots, emitting user declarations plus external references to canonical Universe Metadata Pool 0.
7. **Module-Indexed Backend Tables**: Lowering tables are indexed by `ModuleId`, eliminating `O(N_universe × M_user)` linear filtering.

---

## 8. Ownership boundaries

### 8.1 Owns
- Process-wide construction, caching, and distribution of `CanonicalUniverseCompilationImage`.
- Ephemeral overlay compilation pipeline in `ProgramCompiler::compile_entry_selection`.
- Interface, import, and topology forwarding from module resolution into semantic analysis.
- `CompilationProductRequest` and demand-driven filtering of source presentation, advisory, and formal projections.
- Conformance witness indexing, cached requirement views, and memoized generic proofs.
- Profile-driven metadata reachability analysis and cross-pool external references.
- Backend module indexing and trait-slot precomputation.
- Integration with `telos-observability` structural counters and regression guards.

### 8.2 Does not own
- Language syntax, grammar, AST definitions (`phalcom-ast`).
- VM bytecode instruction set, opcode dispatch, fiber scheduling (`phalcom-core` execution).
- Garbage collection algorithms and heap layout (`phalcom-core` memory).
- Concrete IDE language server protocol implementation (`phalcom-lsp`).
- Native runtime C/FFI primitives (`phalcom-native-*`).

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Canonical Universe source declarations | `UniverseSourceDeclarationCatalog` (shared OnceLock) | `ModuleResolver`, `InterfaceBuilder` | Ad-hoc per-resolver rebuilds |
| Canonical Universe interfaces | `CanonicalInterfaceSet` in compilation image | `ModuleResolver`, `ModuleLinker`, semantic analyzer | Re-parsing canonical source to build interfaces |
| Canonical semantic entities & proofs | `CanonicalUniverseCompilationImage` | `SemanticWorkspaceSession`, lowering, compiler | Re-analyzing canonical callable bodies/conformance |
| Type identity | Canonical `TypeStore` prefix | Forked session `TypeStore` instances | Independent allocation of canonical type IDs |
| Conformance witness locations | `ConformanceWitnessIndex` | Conformance planner, lowering | Linear filtering of global callable tables |
| Metadata records | `MetadataExporter` reachability graph | Compiled program metadata pool | Scanning unreferenced canonical declarations |

---

## 9. Global invariants

- `INV-01`: **Zero Universe Mutation** — A compilation session must never mutate canonical products. All session-specific modifications occur in persistent overlays or forked stores.
- `INV-02`: **TypeStore Prefix Stability** — Forked `TypeStore` instances must retain exact numeric identity for all canonical type IDs, kind IDs, parameter IDs, and family IDs.
- `INV-03`: **One-Shot / Retained Parity** — For any program source, `ProgramCompiler::compile_entry_selection(source)` and `ProgramCompilationSession::compile_fixture(source)` must produce semantically identical diagnostics, callable signatures, type results, witness selections, and executable bytecode.
- `INV-04`: **Sound Query Invalidations** — Cross-session reuse must be sound: canonical query roots are invalidated if Universe source, semantic rules, or native surfaces change.
- `INV-05`: **Zero Re-Parsing of Canonical Sources** — After process warm-up, compiling an inline fixture must perform exactly zero canonical Universe source parses or interface derivations.
- `INV-06`: **Demand-Driven Tooling Products** — Executable compilation in `RuntimeMinimal` mode must not construct presentation source indices for the Universe or execute whole-workspace advisory fixed points.
- `INV-07`: **Linear Conformance Lookup** — Finding an explicit conformance witness for a trait requirement must be an `O(1)` or `O(log N)` indexed lookup, never a linear scan over all global callables.
- `INV-08`: **Deterministic Output** — Compilation output (bytecode, constant pools, metadata IDs, diagnostic messages and locations) must be deterministic and invariant to whether compilation is run cold or warm.
- `INV-09`: **Isolated Error Recovery** — A parse or semantic failure in user code must leave the process-shared canonical compilation image intact and uncorrupted.
- `INV-10`: **Reachability-Bounded Metadata** — Metadata emitted for an inline program must be bounded by the reachability closure of its executable roots and must reference canonical Universe metadata by pool-qualified handle.

---

## 10. Non-goals

- Implementing on-disk serialization of the compilation image (in-memory process sharing is the scope of C2; on-disk caching is a separate future concern).
- Modifying VM bytecode execution or interpreter dispatch loops.
- Redesigning `TypeStore` persistent collection primitives (existing `im::HashMap` / `im::Vector` design is proven sound).
- Altering the language's normative type checking or trait conformance rules.
- Writing custom test harness wrappers (e.g. optimizing `run_inline` alone) to artificially hide compiler latency.

---

## 11. Expected impact map

### Expected source areas
- `phalcom-modules/src/builtin_interface.rs`: Process-shared declaration catalog; canonical interface caching.
- `phalcom-modules/src/resolver.rs`: Direct delegation to canonical interface set.
- `phalcom-modules/src/linker.rs`: Interface preservation and forwarding.
- `phalcom-semantic/src/canonical_baseline.rs`: Expansion into `CanonicalUniverseCompilationImage`.
- `phalcom-semantic/src/session.rs`: Overlay session initialization, revision ancestry (`last_snapshot = Some(...)`).
- `phalcom-semantic/src/workspace.rs`: Product request configuration, demand-driven presentation/advisory.
- `phalcom-semantic/src/checker/impl_conformance.rs`: `ConformanceWitnessIndex`, cached requirement views, structural witness plans.
- `phalcom-semantic/src/checker/analysis.rs`: SCC-driven callable return settlement, module-value staging.
- `phalcom-semantic/src/source_index.rs`: Single-pass `resolve_type_source_targets`, range lookup indexes.
- `phalcom-core/src/modules/compile.rs`: One-shot overlay routing, `CompilationProductRequest::runtime_minimal`.
- `phalcom-core/src/universe/canonical.rs`: Retention of full semantic compilation image.
- `phalcom-core/src/metadata.rs`: Profile-driven reachability closure, cross-pool external metadata handles.
- `telos-observability/src/metric.rs`: Additional cardinality counters for structural regression gates.

### Expected tests
- `phalcom-modules/tests/builtin_catalog.rs`: Catalog sharing, zero rebuild tests.
- `phalcom-semantic/tests/semantic/canonical_baseline.rs`: Full formal image census and immutability tests.
- `phalcom-semantic/tests/semantic/conformance_indexing.rs`: Witness index resolution and coherence tests.
- `phalcom-core/tests/core/modules/compile_session.rs`: One-shot versus persistent session equivalence tests.
- `phalcom-core/tests/core/compiler/cardinality_gates.rs`: Structural counter assertions (0 canonical reparses, 0 body passes).

---

## 12. Implementer decision authority

### 12.1 FIXED
- `CanonicalUniverseCompilationImage` is process-shared, immutable, and process-global via `OnceLock`.
- Compilation of one-shot entries must occur via ephemeral overlay on top of the canonical revision ancestry.
- No second body analysis of callables that do not read module-level values.
- Runtime execution compilation must default to `AdvisoryLevel::None` and `PresentationLevel::None`.
- Conformance witness lookup must use direct indexed maps.
- User metadata bundles must reference canonical Universe metadata via cross-pool references.

### 12.2 MECHANICALLY FLEXIBLE
- Internal field and helper names for indexes and caches.
- Private submodule decomposition within `phalcom-semantic` and `phalcom-core`.
- Choice of index container types (`im::OrdMap`, `BTreeMap`, or `HashMap` where determinism is preserved).
- Fine-grained ordering of non-dependent tasks within each phase.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `UniverseSourceDeclarationCatalog::build` can be called once safely | `phalcom-modules/src/builtin_interface.rs` | Ensure no test overrides mutate global catalog |
| `TypeStore::fork_from_baseline` preserves all canonical `TypeId`s | `phalcom-semantic/src/types/store.rs` | Re-audit prefix preservation logic |
| `SemanticSnapshot::new` callers can supply parts directly | `phalcom-semantic/src/snapshot.rs` | Introduce `SemanticSnapshot::from_parts` constructor |
| Runtime execution does not consume `AdvisoryWorkspace` | `phalcom-core/src/modules/semantic_lowering.rs` | Verify lowering dependencies; consult if advisory facts are required |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing and submit a consultation packet if any of the following occurs:
1. Canonical compilation image cannot be shared across threads due to an unforeseen non-`Send`/`Sync` type.
2. Runtime lowering is found to depend on an advisory analysis fact that requires the whole-workspace fixed-point solver.
3. Type store prefix IDs diverge between one-shot compilation and persistent session compilation.
4. Downstream bytecode lowering requires full `FormalSemanticProjection` presentation data.
5. Cross-pool metadata references require breaking changes to VM heap representation or GC tracing.
6. A semantic regression occurs in trait conformance or generic proof resolution that survives one serious correction cycle.
7. Structural cardinality counters cannot reach zero for canonical rebuilds without altering normative language semantics.

---

## 14. Debugging budget

- **Mechanical failures:** Up to 3 coherent correction cycles (inspect -> identify root cause -> make one correction -> rerun discriminating test).
- **Semantic failures:** Before editing, record Observed, Hypothesis, Evidence, Prediction, and Discriminating Test. Allow 1 serious corrective attempt. If unresolved: STOP AND CONSULT.
- **Architectural failures:** 0 speculative attempts. Consult immediately upon encountering an architectural conflict.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Required Case |
|---|---|---|---|
| CV-01 | One-shot parity | INV-03 | Compile fixture via `compile_entry_selection` and `ProgramCompilationSession`; compare AST, snapshot, and bytecode |
| CV-02 | Canonical immutability | INV-01 | Run 10 sequential different fixtures; assert canonical image pointer and counters unchanged |
| CV-03 | Zero canonical reparsing | INV-05 | Compile 345-byte fixture warm; assert `canonical_interfaces_rebuilt == 0` |
| CV-04 | Zero canonical body analysis | INV-05 | Compile inline fixture; assert `canonical_body_analyses == 0` |
| CV-05 | Runtime presentation exclusion | INV-06 | In `RuntimeMinimal` mode, assert `presentation_universe_rebuilt == 0` |
| CV-06 | Advisory bypass | INV-06 | In `RuntimeMinimal` mode, assert `advisory_iterations == 0` |
| CV-07 | Witness index accuracy | INV-07 | Resolve explicit witnesses, default witnesses, and inherited witnesses; verify identical resolution |
| CV-08 | Generic conformance memoization | INV-04 | Exercise duplicate generic proof requests; assert proof query executed once and cached |
| CV-09 | Single-pass type targets | INV-05 | Assert `TypeReferenceTargetCollector` runs exactly 1 time per module |
| CV-10 | Error isolation | INV-09 | Compile invalid fixture A (parse/type error), followed by valid fixture B; assert B succeeds |
| CV-11 | Diagnostic parity | INV-08 | Compile error fixture under old and new pipeline; assert exact match in diagnostics and source spans |
| CV-12 | Metadata reachability | INV-10 | Compile tiny fixture; assert metadata contains only local roots and pool-0 external references |
| CV-13 | Backend scan indexing | INV-07 | Lower fixture in workspace with 148 modules; assert lowering time scales with fixture module |
| CV-14 | Override provider isolation | INV-04 | Run tests with custom Universe provider; assert custom image built privately without polluting canonical |
| CV-15 | Inferred return SCC | INV-03 | Cyclic mutually recursive functions infer correct returns without infinite re-analysis |
| CV-16 | Deterministic IDs | INV-08 | Compile same fixture in two separate processes; assert byte-identical bytecode and constant pools |
| CV-17 | Module-value staging | INV-05 | Callable body without module values analyzed exactly once |
| CV-18 | Conformance coherence | INV-04 | Partitioned candidate bucket coherence check rejects overlapping conformances identical to old scan |

---

## 16. Verification execution budget

### Modes
- **BUILD MODE:** Focused unit tests and sparse discriminating commands only. Continue through incomplete intermediate states.
- **STABILIZE MODE:** Enter at named verification gates (G1–G6). Run directly affected module suites and focused regressions.
- **CERTIFY MODE:** Enter only at checkpoint closure. Run full package verification.

### Verification Ladder
- `T0`: Exact reproducer or new regression test.
- `T1`: Directly affected feature tests.
- `T2`: Owning crate test target (`cargo test -p <crate> --test <suite> <filter>`).
- `T3`: Cross-layer integration (`cargo test -p phalcom-core --test core`).

### DO NOT RUN during BUILD
- `cargo test --workspace --all-targets` (wastes compute on unrelated baseline failures).
- `cargo clippy --workspace` (reserved for CERTIFY mode).
- Unrelated language corpus groups (e.g. `corpus::booleans` which has known baseline metadata failures).

---

## 17. Baseline / unrelated failure policy

Failures encountered during execution are classified under the standard policy:
- **A / B (caused by current patch):** Active responsibility; diagnose, formulate causal hypothesis, and fix within budget.
- **C (unclear relationship):** Perform one bounded diagnostic pass. If nonblocking, record in checkpoint ledger and continue.
- **D (clearly unrelated / pre-existing baseline):** Record immediately in checkpoint ledger; do not spend compute fixing it. Specifically, the following known baseline failures are pre-classified as **D**:
  - `corpus::booleans` legacy metadata remainder failure.
  - `VM::new()` ProductKind source declaration blocker in `perf_universe_characterization`.
  - `corpus::blocks` missing `fold(initial,using)`.

---

# 18. Tasks

The implementation is partitioned into five sequential, coordinated packages plus an observability and certification phase.

```text
Phase 1: Package A — Canonical Universe Dependency Image & One-Shot Overlay (T1–T6)
Phase 2: Package B — Consumer-Specific Semantic Publication & Demand-Driven Products (T7–T11)
Phase 3: Package C — Conformance and Callable Settlement Incrementalization (T12–T17)
Phase 4: Package D — Profile/Root-Driven Metadata Reachability Architecture (T18–T20)
Phase 5: Package E — Backend Scans, Trait Slots, and Local Complexity Elimination (T21–T24)
Phase 6: Observability & Certification (T25–T26)
```

---

## Phase 1: Package A — Canonical Universe Dependency Image & One-Shot Overlay

### T1 — Process-Shared `UniverseSourceDeclarationCatalog` and Resolver Interface Reuse

#### Purpose
Eliminate the repetitive 148-interface rebuild loop inside `ModuleResolver` by making `UniverseSourceDeclarationCatalog` a process-shared singleton and ensuring `ModuleResolver::load_interface` consumes the canonical interface cache.

#### Preconditions
None.

#### Consumes
`phalcom-modules::builtin_interface::UniverseSourceDeclarationCatalog`.

#### Produces
- Process-shared `UniverseSourceDeclarationCatalog` via `OnceLock`.
- `ModuleResolver::load_interface` checks `interface_cache` for `ProjectIdentity::Universe` before delegating to `build_from_parsed`.

#### Files and symbols
- **Read:** `phalcom-modules/src/builtin_interface.rs`, `phalcom-modules/src/resolver.rs`
- **Modify:** `phalcom-modules/src/builtin_interface.rs`, `phalcom-modules/src/resolver.rs`
- **Tests:** `phalcom-modules/tests/builtin_catalog.rs`, `phalcom-modules/tests/parse_cache.rs`

#### Required implementation shape
In `builtin_interface.rs`:
```rust
static CANONICAL_CATALOG: OnceLock<Result<UniverseSourceDeclarationCatalog, ModuleLoadError>> = OnceLock::new();

impl UniverseSourceDeclarationCatalog {
    pub fn canonical(provider: &UniverseSourceProvider) -> Result<&'static Self, ModuleLoadError> {
        if provider.has_test_source_overrides() {
            // Test override: build private instance
            return Self::build(provider).map(|c| Box::leak(Box::new(c)) as &'static Self);
        }
        CANONICAL_CATALOG.get_or_init(|| Self::build(provider)).as_ref().map_err(|e| e.clone())
    }
}
```
In `resolver.rs` (`load_interface`):
```rust
if module_id.project == ProjectIdentity::Universe {
    let result = crate::builtin_interface::BuiltinInterfaceBuilder::build(&self.universe_source, module_id);
    self.interface_cache.insert(module_id.clone(), result.clone());
    return result;
}
```

#### Forbidden approaches
- Do not remove the test-source override path.
- Do not construct an ad-hoc local catalog per resolver instance.

#### Test changes required
- `CV-03`: Add a regression test verifying that consecutive resolver instances do not rebuild Universe interfaces.

#### Tests to run now
```sh
cargo test -p phalcom-modules --test builtin_catalog
cargo test -p phalcom-modules --test parse_cache
```

#### Acceptance
Calling `load_interface` on 10 fresh resolvers for `ModuleId::universe_root()` builds the catalog and interfaces exactly once.

---

### T2 — Module Pipeline Product Forwarding in `SemanticWorkspaceInput`

#### Purpose
Preserve parsed modules, unlinked interfaces, import resolution products, and module topology through the compilation pipeline, eliminating redundant re-derivation in downstream layers.

#### Preconditions
T1 complete.

#### Consumes
`UnlinkedModuleInterface`, `ModuleTopology`, `ImportProducts` from `phalcom-modules`.

#### Produces
Updated `SemanticWorkspaceInput` carrying pre-derived interfaces, import products, and topology.

#### Files and symbols
- **Read:** `phalcom-semantic/src/workspace.rs`, `phalcom-semantic/src/session.rs`
- **Modify:** `phalcom-semantic/src/workspace.rs`, `phalcom-semantic/src/session.rs`, `phalcom-core/src/modules/compile.rs`
- **Tests:** `phalcom-semantic/tests/semantic/workspace_inputs.rs`

#### Required implementation shape
Extend `SemanticWorkspaceInput` to make pre-computed products authoritative:
```rust
pub struct SemanticWorkspaceInput {
    pub linked: Arc<LinkedProgram>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub interfaces: Arc<BTreeMap<ModuleId, Arc<UnlinkedModuleInterface>>>,
    pub import_products: Arc<BTreeMap<ModuleId, Arc<ModuleImportProducts>>>,
    pub topology: Option<Arc<ModuleTopology>>,
    pub reverse_imports: Option<Arc<ReverseImportIndex>>,
    pub delta: Option<WorkspaceDelta>,
    ...
}
```
Ensure `SemanticWorkspaceSession::update` consumes `input.interfaces` directly rather than traversing `input.sources` to rebuild interfaces.

#### Forbidden approaches
- Do not discard `input.interfaces` to reconstruct them from AST.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic workspace
```

#### Acceptance
Semantic analysis of a workspace with pre-populated `interfaces` performs zero calls to `InterfaceBuilder::build`.

---

### T3 — Canonical Universe Baseline Revision Ancestry

#### Purpose
Treat the canonical Universe as the base semantic revision (`last_snapshot = Some(CanonicalUniverse)`), so that user fixtures compile as `Revision U -> Revision U+1` rather than triggering cold whole-workspace analysis.

#### Preconditions
T2 complete.

#### Consumes
`CanonicalUniverseSemanticBaseline` or `CanonicalUniverseCompilationImage`.

#### Produces
`SemanticWorkspaceSession::from_baseline` initializes `last_snapshot` and `last_known_good` with the canonical Universe formal snapshot.

#### Files and symbols
- **Read:** `phalcom-semantic/src/session.rs`, `phalcom-semantic/src/canonical_baseline.rs`
- **Modify:** `phalcom-semantic/src/session.rs`, `phalcom-semantic/src/canonical_baseline.rs`
- **Tests:** `phalcom-semantic/tests/semantic/session_ancestry.rs`

#### Required implementation shape
In `session.rs` (`from_baseline`):
```rust
// Instead of last_snapshot: None:
last_snapshot: Some(baseline.canonical_snapshot.clone()),
last_known_good: Some(baseline.canonical_snapshot.clone()),
```
In `update()`, when computing changed modules:
```rust
let delta = WorkspaceDelta {
    added: user_modules,
    changed: BTreeSet::new(), // canonical modules are unchanged
    removed: BTreeSet::new(),
};
```

#### Forbidden approaches
- Do not mark canonical Universe modules as `changed` or `added` when starting a session.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic incremental
```

#### Acceptance
A fresh session created from the baseline reports `changed_canonical_modules == 0` and `recomputed_canonical_callables == 0` during fixture update.

---

### T4 — One-Shot Compiler Ephemeral Module-Session Architecture

#### Purpose
Rewrite `ProgramCompiler::compile_entry_selection(EntrySelection::Inline)` to compile through an ephemeral `WorkspaceModuleSession` overlay instead of injecting 40 Universe modules into an empty workspace.

#### Preconditions
T1–T3 complete.

#### Consumes
`WorkspaceModuleSession`, `SetOverlayForModule`.

#### Produces
Elimination of the manual 40-module load loop in `phalcom-core/src/modules/compile.rs:594-630`.

#### Files and symbols
- **Read:** `phalcom-core/src/modules/compile.rs`, `phalcom-core/src/modules/compile_session.rs`
- **Modify:** `phalcom-core/src/modules/compile.rs`
- **Tests:** `phalcom-core/tests/core/modules/compile_session.rs`, `phalcom-core/tests/core/language/data_e2e.rs`

#### Required implementation shape
Refactor `compile_entry_selection` for `EntrySelection::Inline` and `StandaloneModule`:
```rust
// Internally instantiate an ephemeral session initialized from the canonical image:
let mut session = ProgramCompilationSession::new_ephemeral()?;
let compiled = session.compile_fixture(&source_text)?;
Ok(compiled)
```
Ensure `new_ephemeral` uses the shared process canonical image and forked type store without creating persistent disk state.

#### Forbidden approaches
- Do not maintain the manual 40-module source filter loop.
- Do not make this optimization accessible only via `run_inline` test helpers.

#### Tests to run now
```sh
cargo test -p phalcom-core --test core execution_static_const
cargo test -p phalcom-core --test core compiler_static_const
```

#### Acceptance
`compile_entry_selection(Inline)` succeeds with zero 40-module synthetic injections and identical compiled bytecode.

---

### T5 — `CanonicalUniverseCompilationImage` Durable Representation

#### Purpose
Create the full, immutable `CanonicalUniverseCompilationImage` in `phalcom-semantic`, replacing the dual linking and discarded formal snapshot with one authoritative process-shared image.

#### Preconditions
T1–T4 complete.

#### Consumes
Full formal semantic result from `canonical_universe_program()`.

#### Produces
`CanonicalUniverseCompilationImage` retaining parsed modules, interfaces, topology, linked program, finalized TypeStore prefix, declarations, hierarchy, dispatch, callable signatures, algebraic products, conformance index, witness plans, trait dispatch, and query roots.

#### Files and symbols
- **Read:** `phalcom-core/src/universe/canonical.rs`, `phalcom-semantic/src/canonical_baseline.rs`
- **Create:** `phalcom-semantic/src/compilation_image.rs`
- **Modify:** `phalcom-semantic/src/lib.rs`, `phalcom-semantic/src/canonical_baseline.rs`, `phalcom-core/src/universe/canonical.rs`
- **Tests:** `phalcom-semantic/tests/semantic/compilation_image.rs`

#### Required implementation shape
Define `CanonicalUniverseCompilationImage` as specified in Section 7.
In `phalcom-core/src/universe/canonical.rs`:
```rust
// In build_canonical_universe_program():
let image = CanonicalUniverseCompilationImage::build_canonical(&provider, &semantic)?;
CANONICAL_IMAGE.set(image.clone()).ok();
```
Expose `canonical_compilation_image() -> Arc<CanonicalUniverseCompilationImage>` for all downstream consumers.

#### Forbidden approaches
- Do not discard the semantic snapshot in `build_canonical_universe_program()`.
- Do not allow mutation of any `Arc` product inside the image.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic canonical_baseline
```

#### Acceptance
Process initializes `CanonicalUniverseCompilationImage` once; all subsequent queries retrieve immutable `Arc` references in O(1).

---

### T6 — Cross-Session Query Dependency Integration (External Green Query Roots)

#### Purpose
Allow `SemanticDb` in a user session to reference canonical query results as external green inputs, eliminating repeated publication of canonical declarations, signatures, and hierarchy edges into fresh databases.

#### Preconditions
T5 complete.

#### Consumes
`CanonicalSemanticQueryImage` from `CanonicalUniverseCompilationImage`.

#### Produces
`SemanticDb` queries check `self.canonical.query_image` before local recomputation; canonical roots never trigger downstream invalidation during user edits.

#### Files and symbols
- **Read:** `phalcom-semantic/src/db.rs`, `phalcom-semantic/src/checker/analysis.rs`
- **Modify:** `phalcom-semantic/src/db.rs`, `phalcom-semantic/src/session.rs`
- **Tests:** `phalcom-semantic/tests/semantic/query_overlay.rs`

#### Required implementation shape
In `SemanticDb`:
```rust
pub struct SemanticDb {
    canonical_queries: Arc<CanonicalSemanticQueryImage>,
    local_products: BTreeMap<QueryKey, QueryProduct>,
    ...
}

impl SemanticDb {
    pub fn get_query<Q: Query>(&self, key: &Q::Key) -> Option<&Q::Value> {
        if let Some(local) = self.local_products.get(key) {
            return Some(local);
        }
        self.canonical_queries.get::<Q>(key)
    }
}
```

#### Forbidden approaches
- Do not deep-clone the canonical `SemanticDb`.
- Do not add user dependency edges into the canonical query image.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic query
```

#### Acceptance
Fresh session does not execute query bootstrap loops for canonical declarations, signatures, or hierarchy edges.

---

## Phase 2: Package B — Consumer-Specific Semantic Publication & Demand-Driven Products

### T7 — `CompilationProductRequest` Capability Profiling

#### Purpose
Introduce explicit capability profiling to compilation, allowing callers to specify whether executable lowering, diagnostics, reflection metadata, presentation, or advisory products are required.

#### Preconditions
Phase 1 complete.

#### Consumes
Compilation parameters.

#### Produces
`CompilationProductRequest` enum/struct with presets (`runtime_minimal`, `runtime_public`, `check`, `tooling`, `proof`).

#### Files and symbols
- **Read:** `phalcom-core/src/modules/compile.rs`, `phalcom-semantic/src/workspace.rs`
- **Create:** `phalcom-semantic/src/request.rs`
- **Modify:** `phalcom-semantic/src/workspace.rs`, `phalcom-core/src/modules/compile.rs`
- **Tests:** `phalcom-semantic/tests/semantic/product_request.rs`

#### Required implementation shape
```rust
#[derive(Clone, Debug)]
pub struct CompilationProductRequest {
    pub executable: bool,
    pub diagnostics: DiagnosticLevel,
    pub reflection: MetadataProfile,
    pub presentation: PresentationLevel,
    pub advisory: AdvisoryLevel,
}

impl CompilationProductRequest {
    pub fn runtime_minimal() -> Self {
        Self {
            executable: true,
            diagnostics: DiagnosticLevel::ErrorsOnly,
            reflection: MetadataProfile::RuntimeMinimal,
            presentation: PresentationLevel::None,
            advisory: AdvisoryLevel::None,
        }
    }
    pub fn tooling() -> Self { ... }
}
```
Route `ProgramCompiler::compile_entry_selection` with `CompilationProductRequest::runtime_minimal()`.

#### Forbidden approaches
- Do not default runtime execution to full tooling/presentation publication.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic product_request
```

#### Acceptance
Compilation pipelines accept `CompilationProductRequest` and selectively skip downstream product construction based on request flags.

---

### T8 — Separation of Presentation Source Index from Core Semantic Source Data

#### Purpose
Split the monolithic source index into `CoreSourceSemanticIndex` (for user runtime modules only) and `PresentationSourceIndex` (for IDE navigation and source presentation), preventing runtime builds from indexing the 148-module Universe.

#### Preconditions
T7 complete.

#### Consumes
`CompilationProductRequest`.

#### Produces
`CoreSourceSemanticIndex` containing only source data required for executable lowering; `PresentationSourceIndex` generated only when `presentation != PresentationLevel::None`.

#### Files and symbols
- **Read:** `phalcom-semantic/src/source_index.rs`, `phalcom-semantic/src/session.rs`
- **Modify:** `phalcom-semantic/src/source_index.rs`, `phalcom-semantic/src/session.rs`, `phalcom-semantic/src/snapshot.rs`
- **Tests:** `phalcom-semantic/tests/semantic/source_index.rs`

#### Required implementation shape
In `session.rs` (`publish_snapshot`):
```rust
let source_index = if request.presentation.is_active() {
    Some(self.build_presentation_source_index()?)
} else {
    None
};
let core_source_data = self.build_core_source_semantic_index(&user_modules)?;
```

#### Forbidden approaches
- Do not traverse Universe ASTs to build source ranges or navigation targets during `runtime_minimal` compilation.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic source_index
```

#### Acceptance
In `runtime_minimal` mode, `source_index_module_rebuilds == 0` for all canonical Universe modules.

---

### T9 — Single-Pass `ResolvedTypeSourceTargets` AST Traversal

#### Purpose
Collapse the three separate AST traversals for nominal type references, type-member projections, and type parameters into a single unified collector pass.

#### Preconditions
T8 complete.

#### Consumes
`TypeReferenceTargetCollector`.

#### Produces
`ResolvedTypeSourceTargets` struct populated in a single AST visit; eliminate redundant loops over module conformances.

#### Files and symbols
- **Read:** `phalcom-semantic/src/source_index.rs`
- **Modify:** `phalcom-semantic/src/source_index.rs`
- **Tests:** `phalcom-semantic/tests/semantic/type_targets.rs`

#### Required implementation shape
```rust
pub struct ResolvedTypeSourceTargets {
    pub nominal: BTreeMap<SourceRange, DeclarationId>,
    pub member_projections: BTreeMap<SourceRange, TypeMemberDeclarationId>,
    pub type_parameters: TypeParameterSourceTargets,
}

pub fn resolve_type_source_targets(
    module: &ModuleId,
    program: &Program,
    resolver: &NameResolver,
) -> ResolvedTypeSourceTargets {
    // Single visitor collecting nominal, projections, and parameters
}
```
In conformance index construction, replace the per-`impl` full-module scan with an indexed lookup into `targets.nominal`.

#### Forbidden approaches
- Do not traverse the module AST per conformance implementation (`O(I × P)` complexity).

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic type_targets
```

#### Acceptance
For any rebuilt module, `TypeReferenceTargetCollector` runs exactly once (`full_ast_traversals == 1`).

---

### T10 — Decoupling Advisory Workspace from Runtime Execution

#### Purpose
Eliminate the unconditional whole-workspace fixed-point solver run across 41 semantic modules during executable compilation.

#### Preconditions
T7 complete.

#### Consumes
`CompilationProductRequest::advisory`.

#### Produces
When `advisory == AdvisoryLevel::None`, `AdvisoryWorkspace::update` returns an empty/stub advisory product immediately without iterating callable flows.

#### Files and symbols
- **Read:** `phalcom-semantic/src/advisory.rs`, `phalcom-semantic/src/session.rs`
- **Modify:** `phalcom-semantic/src/advisory.rs`, `phalcom-semantic/src/session.rs`
- **Tests:** `phalcom-semantic/tests/semantic/advisory.rs`

#### Required implementation shape
In `session.rs`:
```rust
let advisory = if request.advisory != AdvisoryLevel::None {
    self.run_advisory_analysis(request.advisory)?
} else {
    Arc::new(AdvisoryWorkspace::empty())
};
```
Verify that no runtime lowering logic inspects `snapshot.advisory`.

#### Forbidden approaches
- Do not run the 41-module fixed point when compiling for execution.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic advisory
```

#### Acceptance
In `runtime_minimal` compilation, `advisory_outer_iterations == 0` and `advisory_body_flows == 0`.

---

### T11 — Snapshot Constructor Normalization (`from_parts`)

#### Purpose
Eliminate redundant internal recomputation of `FormalSemanticProjection` in `SemanticSnapshot::new_with_callable_analyses`.

#### Preconditions
T7–T10 complete.

#### Consumes
Pre-computed semantic products.

#### Produces
`SemanticSnapshot::from_parts(SemanticSnapshotParts { ... })` pure assembly constructor; eliminates temporary projection overwrite.

#### Files and symbols
- **Read:** `phalcom-semantic/src/snapshot.rs`, `phalcom-semantic/src/session.rs`
- **Modify:** `phalcom-semantic/src/snapshot.rs`, `phalcom-semantic/src/session.rs`
- **Tests:** `phalcom-semantic/tests/semantic/snapshot_construction.rs`

#### Required implementation shape
```rust
pub struct SemanticSnapshotParts {
    pub declarations: Arc<DeclarationTypeTable>,
    pub signatures: Arc<CallableSignatureTable>,
    pub analyses: Arc<CallableAnalysisTable>,
    pub formal_projection: Option<Arc<FormalSemanticProjection>>,
    pub source_index: Option<Arc<SourceSemanticIndex>>,
    pub advisory: Arc<AdvisoryWorkspace>,
    ...
}

impl SemanticSnapshot {
    pub fn from_parts(parts: SemanticSnapshotParts) -> Self { ... }
}
```
Remove internal calls to `build_formal_semantic_projection` inside constructors.

#### Forbidden approaches
- Never construct a formal projection and immediately replace/overwrite it.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic snapshot_construction
```

#### Acceptance
`redundant_snapshot_formal_projections == 0` during any successful compilation.

---

## Phase 3: Package C — Conformance and Callable Settlement Incrementalization

### T12 — Module-Value Environment Staging & Dependency Sensitivity

#### Purpose
Eliminate the systematic second callable-body analysis caused by checking bodies before establishing the module-value environment.

#### Preconditions
Phase 1–2 complete.

#### Consumes
Module export and binding environment.

#### Produces
Module-value environment established before callable-body checking; bodies that do not reference module values are analyzed exactly once.

#### Files and symbols
- **Read:** `phalcom-semantic/src/checker/analysis.rs`, `phalcom-semantic/src/checker/module_values.rs`
- **Modify:** `phalcom-semantic/src/checker/analysis.rs`
- **Tests:** `phalcom-semantic/tests/semantic/module_value_staging.rs`

#### Required implementation shape
Reorder semantic phases:
```text
1. declarations / signatures
2. module binding and value environment
3. callable body analysis
```
If a callable body does not access module-level values, its analysis fingerprint must not include a boolean indicating whether the module environment has been published.

#### Forbidden approaches
- Do not use blanket `has_module_value_environment: bool` in body query fingerprints.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic module_value_staging
```

#### Acceptance
For callables without module-value dependencies, `callable_body_analyses_per_callable == 1`.

---

### T13 — SCC-Driven Inferred-Return Propagation

#### Purpose
Replace iterative whole-workspace re-analysis of unknown-return callables with an explicit SCC-driven dependency graph, filtering out recomputations whose observable return contracts did not change.

#### Preconditions
T12 complete.

#### Consumes
Callable return dependency edges.

#### Produces
Tarjan SCC decomposition of callable return dependencies; acyclic components processed once; cyclic SCCs iterated to fixed point; propagation halted when contract fingerprint is unchanged.

#### Files and symbols
- **Read:** `phalcom-semantic/src/checker/analysis.rs`
- **Modify:** `phalcom-semantic/src/checker/analysis.rs`
- **Tests:** `phalcom-semantic/tests/semantic/inferred_return_scc.rs`

#### Required implementation shape
```rust
let sccs = compute_callable_return_sccs(&callables);
for scc in sccs {
    if scc.is_acyclic() {
        analyze_callable(scc.single());
    } else {
        iterate_scc_to_fixed_point(scc);
    }
    if scc.public_contract_fingerprint_unchanged() {
        continue; // Do not invalidate downstream SCCs
    }
    propagate_invalidation(scc.dependents());
}
```

#### Forbidden approaches
- Do not re-analyze downstream callables if the newly inferred return type is identical to the previous pass.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic inferred_return_scc
```

#### Acceptance
Mutually recursive and chained return inference settles in minimal passes; downstream callables are not re-analyzed when upstream return types are unchanged.

---

### T14 — Indexed `ConformanceWitnessIndex` for Direct Requirement Lookup

#### Purpose
Replace linear search over the global callable table during witness planning with a direct multi-key `ConformanceWitnessIndex`.

#### Preconditions
Phase 1–2 complete.

#### Consumes
Conformance-owned callable signatures.

#### Produces
`ConformanceWitnessIndex` mapping `(ImplId, CallableSide, Selector) -> CallableId`.

#### Files and symbols
- **Read:** `phalcom-semantic/src/checker/impl_conformance.rs`
- **Create:** `phalcom-semantic/src/checker/witness_index.rs`
- **Modify:** `phalcom-semantic/src/checker/impl_conformance.rs`
- **Tests:** `phalcom-semantic/tests/semantic/witness_indexing.rs`

#### Required implementation shape
```rust
pub struct ConformanceWitnessIndex {
    explicit: HashMap<(ImplId, CallableSide, Selector), CallableId>,
}

impl ConformanceWitnessIndex {
    pub fn build(signatures: &CallableSignatureTable) -> Self { ... }
    pub fn lookup(&self, impl_id: ImplId, side: CallableSide, selector: &Selector) -> Option<CallableId> {
        self.explicit.get(&(impl_id, side, selector.clone())).copied()
    }
}
```
Replace `callable_signatures.iter().find_map(...)` in witness planner with `index.lookup(...)`.

#### Forbidden approaches
- Do not scan the entire callable table per trait requirement.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic witness_indexing
```

#### Acceptance
Witness retrieval complexity is O(1) per requirement; `witness_signature_probes` equals the number of explicit requirements.

---

### T15 — Cached Requirement Views & Structural / Final Witness Plan Separation

#### Purpose
Split witness planning into a stable `StructuralConformancePlan` and a return-sensitive `FinalWitnessTyping` phase; cache normalized `ConformanceRequirementView`.

#### Preconditions
T14 complete.

#### Consumes
Trait declarations, impl declarations.

#### Produces
`ConformanceRequirementView` memoized per `(ImplId, TraitRef)`; `StructuralConformancePlan` built once and reused across return-settlement passes.

#### Files and symbols
- **Read:** `phalcom-semantic/src/checker/impl_conformance.rs`
- **Modify:** `phalcom-semantic/src/checker/impl_conformance.rs`
- **Tests:** `phalcom-semantic/tests/semantic/witness_plans.rs`

#### Required implementation shape
```rust
pub struct StructuralConformancePlan {
    pub impl_id: ImplId,
    pub explicit_witnesses: Arc<[CallableId]>,
    pub default_candidates: Arc<[CallableId]>,
    pub substitutions: Arc<GenericSubstitutions>,
}

pub struct FinalWitnessTyping {
    pub specialized_signatures: Arc<[SpecializedSignature]>,
}
```
Only re-evaluate `FinalWitnessTyping` when an inferred return actually changes an associated requirement.

#### Forbidden approaches
- Do not discard the provisional plan and re-derive candidate ownership and generic substitutions from scratch in final planning.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic witness_plans
```

#### Acceptance
Structural witness planning executes once per conformance; return settlement re-types only affected signatures.

---

### T16 — Multi-Key `ConformanceIndex` and Partitioned Coherence Checking

#### Purpose
Index `ConformanceIndex` by target family and canonical conformance head; partition coherence checking into trait/family buckets.

#### Preconditions
Phase 1–2 complete.

#### Consumes
Conformances.

#### Produces
Bucketed candidate lookup (`by_trait_and_family`, `exact_heads`); pairwise coherence check limited to matching buckets.

#### Files and symbols
- **Read:** `phalcom-semantic/src/conformance.rs`, `phalcom-semantic/src/checker/impl_conformance.rs`
- **Modify:** `phalcom-semantic/src/conformance.rs`, `phalcom-semantic/src/checker/impl_conformance.rs`
- **Tests:** `phalcom-semantic/tests/semantic/conformance_buckets.rs`

#### Required implementation shape
```rust
pub struct ConformanceIndex {
    by_impl: BTreeMap<ImplId, Arc<ConformanceDeclaration>>,
    by_trait: HashMap<TraitDeclarationId, Vec<ImplId>>,
    by_trait_and_family: HashMap<(TraitDeclarationId, TypeFamilyKey), Vec<ImplId>>,
    exact_heads: HashMap<CanonicalConformanceHead, ImplId>,
}
```
In coherence checking, compare candidates pairwise only within the same `(TraitDeclarationId, TypeFamilyKey)` bucket.

#### Forbidden approaches
- Do not compare all conformances pairwise across unrelated traits or disjoint type families.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic conformance_buckets
```

#### Acceptance
`ConformanceIndex::query_exact` performs direct hash lookup; coherence checking runtime scales linearly with bucket size.

---

### T17 — Memoized `GenericConformanceKey` Semantic Query

#### Purpose
Promote `GenericConformanceKey` to a first-class, session-wide memoized semantic query in `SemanticDb`.

#### Preconditions
T16 complete.

#### Consumes
Generic proof queries.

#### Produces
`db.generic_conformance(key)` memoized query; results reused across expressions and body-analysis passes.

#### Files and symbols
- **Read:** `phalcom-semantic/src/generic_evidence.rs`, `phalcom-semantic/src/db.rs`
- **Modify:** `phalcom-semantic/src/generic_evidence.rs`, `phalcom-semantic/src/db.rs`
- **Tests:** `phalcom-semantic/tests/semantic/generic_conformance_query.rs`

#### Required implementation shape
Define `GenericConformance` in `QueryKey`. Route proof requests through `db.generic_conformance(key)`. Cache proof trees and validity fingerprints.

#### Forbidden approaches
- Do not discard proof search results between different call sites or different body passes.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic generic_conformance_query
```

#### Acceptance
Duplicate generic conformance goals across expressions or body revalidation passes hit query cache with zero proof search steps.

---

## Phase 4: Package D — Profile/Root-Driven Metadata Reachability Architecture

### T18 — `MetadataRetentionPolicy` and Reachability Closure Engine

#### Purpose
Implement true profile-driven metadata export: select root entities based on `MetadataProfile` and compute a reachability closure, eliminating whole-world declaration sweeps.

#### Preconditions
Phase 1–2 complete.

#### Consumes
`MetadataProfile`, `SemanticSnapshot`.

#### Produces
`MetadataRetentionPolicy` trait; reachability traversal marking referenced declarations, types, and callables.

#### Files and symbols
- **Read:** `phalcom-core/src/metadata.rs`
- **Create:** `phalcom-core/src/metadata/reachability.rs`
- **Modify:** `phalcom-core/src/metadata.rs`
- **Tests:** `phalcom-core/tests/core/compiler/metadata_reachability.rs`

#### Required implementation shape
```rust
pub trait MetadataRetentionPolicy {
    fn declaration_is_root(&self, decl: &DeclarationId, info: &DeclarationInfo) -> bool;
    fn callable_is_root(&self, callable: &CallableId, sig: &CallableSemanticSignature) -> bool;
}

pub fn compute_metadata_reachability(
    snapshot: &SemanticSnapshot,
    policy: &dyn MetadataRetentionPolicy,
) -> MetadataReachabilityClosure { ... }
```
Only emit metadata records for entities present in the reachability closure.

#### Forbidden approaches
- Do not iterate `snapshot.declarations` unconditionally during `RuntimeMinimal` export.

#### Tests to run now
```sh
cargo test -p phalcom-core --test core metadata_reachability
```

#### Acceptance
In `RuntimeMinimal` mode, metadata record count for an inline fixture is proportional to fixture roots rather than total Universe declarations.

---

### T19 — `CallableOwnerIndex` for Constant-Time Owner Probes

#### Purpose
Eliminate the nested loop in metadata export that scans every callable in the program for every declaration.

#### Preconditions
T18 complete.

#### Consumes
`CallableSignatureTable`.

#### Produces
`CallableOwnerIndex` mapping `DeclarationId -> Vec<CallableId>` and `ImplId -> Vec<CallableId>`.

#### Files and symbols
- **Read:** `phalcom-semantic/src/types/signatures.rs`, `phalcom-core/src/metadata.rs`
- **Modify:** `phalcom-semantic/src/types/signatures.rs`, `phalcom-core/src/metadata.rs`
- **Tests:** `phalcom-semantic/tests/semantic/callable_owner_index.rs`

#### Required implementation shape
Expose `callables_by_owner(&self, owner: CallableOwnerId) -> &[CallableId]` on `CallableSignatureTable`. In `metadata.rs`, retrieve a declaration's callables via `signatures.callables_by_owner(decl_id)` instead of filtering the global table.

#### Forbidden approaches
- Do not run `signatures.iter().filter(|c| c.owner == decl)` inside a per-declaration loop.

#### Tests to run now
```sh
cargo test -p phalcom-core --test core metadata
```

#### Acceptance
`metadata_callable_owner_comparisons` drops from `O(Declarations × Callables)` to `O(Reachable Callables)`.

---

### T20 — Cross-Pool External Metadata References for Canonical Universe

#### Purpose
Allow user program metadata bundles to reference canonical Universe metadata via pool-qualified handles instead of duplicating canonical records into every bundle.

#### Preconditions
T18–T19 complete.

#### Consumes
`CanonicalUniverseCompilationImage.metadata` (Pool 0).

#### Produces
Pool-qualified metadata handles (`MetadataPoolId`, `RecordIndex`); user programs export Pool N referencing Pool 0.

#### Files and symbols
- **Read:** `phalcom-core/src/metadata.rs`, `phalcom-core/src/vm/metadata.rs`
- **Modify:** `phalcom-core/src/metadata.rs`, `phalcom-core/src/vm/metadata.rs`
- **Tests:** `phalcom-core/tests/core/compiler/cross_pool_metadata.rs`

#### Required implementation shape
```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataHandle {
    Canonical(UniverseMetadataIndex),
    Local(LocalMetadataIndex),
}
```
Update VM metadata loader to resolve external references against the already-installed canonical Universe metadata pool.

#### Forbidden approaches
- Do not delete Universe records without establishing valid pool-qualified references.

#### Tests to run now
```sh
cargo test -p phalcom-core --test core cross_pool_metadata
```

#### Acceptance
User program metadata size decreases by > 90%; canonical Universe types reflect correctly at runtime via Pool 0 handles.

---

## Phase 5: Package E — Backend Scans, Trait Slots, and Local Complexity Elimination

### T21 — Persistent Module-Indexed Backend Tables

#### Purpose
Replace global table filtering (`global_table.iter().filter(|x| x.module == runtime_module)`) in compiler lowering with persistent module-indexed partitions.

#### Preconditions
Phase 1–2 complete.

#### Consumes
Lowering tables.

#### Produces
Indexes:
- `ModuleId -> Vec<CallableId>`
- `ModuleId -> Vec<DeclarationId>`
- `ModuleId -> Vec<SourceAttachment>`

#### Files and symbols
- **Read:** `phalcom-core/src/modules/semantic_lowering.rs`, `phalcom-core/src/modules/compile.rs`
- **Modify:** `phalcom-core/src/modules/semantic_lowering.rs`, `phalcom-core/src/modules/compile.rs`
- **Tests:** `phalcom-core/tests/core/compiler/module_indexes.rs`

#### Required implementation shape
Structure backend lowering tables as `BTreeMap<ModuleId, Arc<[EntityId]>>`. Lowering of a user fixture accesses only `table.get(&fixture_module)`.

#### Forbidden approaches
- Do not iterate across the 148 Universe modules when lowering an inline fixture module.

#### Tests to run now
```sh
cargo test -p phalcom-core --test core compiler
```

#### Acceptance
Lowering execution time for a fixture is independent of total workspace module count.

---

### T22 — Canonical `TraitRequirementSlotIndex` Precomputation & Inherent-Impl Domain Index

#### Purpose
Precompute global trait requirement-to-slot assignments in `CanonicalUniverseCompilationImage` and index inherent implementation domains by `ImplId`.

#### Preconditions
Phase 1–2 complete.

#### Consumes
Trait requirements and inherent impls.

#### Produces
- `TraitRequirementSlotIndex` in compilation image.
- `HashMap<ImplId, InherentImplDomain>` replacing scans across conditional members.

#### Files and symbols
- **Read:** `phalcom-core/src/typing/conformance.rs`, `phalcom-core/src/modules/compile.rs`
- **Modify:** `phalcom-core/src/typing/conformance.rs`, `phalcom-core/src/modules/compile.rs`
- **Tests:** `phalcom-core/tests/core/compiler/trait_slots.rs`

#### Required implementation shape
In `compile.rs`:
```rust
fn inherent_impl_domain_for_callable<'a>(
    snapshot: &'a SemanticSnapshot,
    callable: &CallableId,
    expected_impl: Option<&ImplId>,
) -> Option<&'a InherentImplDomain> {
    snapshot.inherent_impl_domains_by_impl.get(source_impl)
}
```

#### Forbidden approaches
- Do not reconstruct requirement-to-slot maps per trait invocation during bytecode generation.

#### Tests to run now
```sh
cargo test -p phalcom-core --test core trait_slots
```

#### Acceptance
`TraitRequirementSlotIndex` constructed once; inherent impl domain lookup is O(1).

---

### T23 — Module Source Lookup Range Indexes (`ModuleSourceLookupIndex`)

#### Purpose
Replace `O(F × E)` and `O(E × O)` linear scans for source sites and occurrences with logarithmic range-indexed lookup structures.

#### Preconditions
Phase 2 complete.

#### Consumes
Source sites and occurrences.

#### Produces
`ModuleSourceLookupIndex` mapping `SourceRange -> SmallVec<[SourceSiteId; 2]>` and `SourceRange -> SmallVec<[OccurrenceId; 2]>`.

#### Files and symbols
- **Read:** `phalcom-semantic/src/source_index.rs`
- **Modify:** `phalcom-semantic/src/source_index.rs`
- **Tests:** `phalcom-semantic/tests/semantic/source_lookup.rs`

#### Required implementation shape
```rust
pub struct ModuleSourceLookupIndex {
    by_site_id: HashMap<SourceSiteId, SourceSite>,
    occurrences_by_range: BTreeMap<SourceRange, SmallVec<[OccurrenceId; 2]>>,
    expression_sites_by_range: BTreeMap<SourceRange, SmallVec<[SourceSiteId; 2]>>,
}
```

#### Forbidden approaches
- Do not filter expression site vectors linearly by range.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic source_lookup
```

#### Acceptance
Source site attachment and range queries execute in logarithmic time.

---

### T24 — String-Alloc and BTreeMap Thrash Elimination in Hot Paths

#### Purpose
Replace temporary formatted strings in structural fingerprints with typed hasher feeds; eliminate avoidable BTreeMap cloning in session publication.

#### Preconditions
T1–T23 complete.

#### Consumes
Structural fingerprinting call sites.

#### Produces
Hasher feeds directly writing numeric IDs; persistent sharded snapshot maps (`snapshot_full_map_materializations == 0`).

#### Files and symbols
- **Read:** `phalcom-semantic/src/session.rs`, `phalcom-semantic/src/checker/analysis.rs`
- **Modify:** `phalcom-semantic/src/session.rs`, `phalcom-semantic/src/checker/analysis.rs`
- **Tests:** `phalcom-semantic/tests/semantic/snapshot_materialization.rs`

#### Required implementation shape
Replace `format!("trait-environment-{}-{}-{}-{}-{}", ...)` with a direct composite tuple hasher feed:
```rust
let mut hasher = DefaultHasher::new();
range.hash(&mut hasher);
source_impl.hash(&mut hasher);
requirement.hash(&mut hasher);
parameter.hash(&mut hasher);
let key = TraitEnvironmentKey(hasher.finish());
```

#### Forbidden approaches
- Do not allocate heap `String`s solely to form map keys in hot compiler loops.

#### Tests to run now
```sh
cargo test -p phalcom-semantic --test semantic snapshot_materialization
```

#### Acceptance
`SnapshotFullMapMaterializations` drops from 8 to 0 for single-module updates.

---

## Phase 6: Observability & Certification

### T25 — `telos-observability` Work-Cardinality Regression Guards

#### Purpose
Establish automated CI test assertions on `telos-observability` structural cardinality counters, ensuring that hardware speedups cannot mask algorithmic regressions.

#### Preconditions
T1–T24 complete.

#### Consumes
`telos-observability` metrics.

#### Produces
A dedicated test suite in `phalcom-core/tests/core/compiler/cardinality_gates.rs` asserting exact counter bounds for warm fixture compilation.

#### Files and symbols
- **Read:** `telos-observability/src/metric.rs`, `telos-observability/src/report.rs`
- **Create:** `phalcom-core/tests/core/compiler/cardinality_gates.rs`
- **Tests:** `phalcom-core/tests/core/compiler/cardinality_gates.rs`

#### Required implementation shape
```rust
#[test]
fn test_warm_compilation_cardinality_gates() {
    let mut session = ProgramCompilationSession::new_ephemeral().unwrap();
    // Warm up session
    session.compile_fixture("let x = 1").unwrap();

    let recorder = PerfRecorder::new(ObservabilityLevel::Detailed);
    perf_context::with_recorder(&recorder, || {
        session.compile_fixture("let y = 2").unwrap();
    });

    let report = recorder.finish();
    assert_eq!(report.counter(PerfCounter::CanonicalUniverseProductBuilds), 0);
    assert_eq!(report.counter(PerfCounter::SemanticCallableBodyInitialPassQueries), 1);
    assert_eq!(report.counter(PerfCounter::SemanticCallableBodiesRecomputed), 1);
}
```

#### Forbidden approaches
- Do not use elapsed time as the sole regression guard in CI.

#### Tests to run now
```sh
cargo test -p phalcom-core --test core cardinality_gates
```

#### Acceptance
All mandatory structural cardinality counters in Section 21 pass with exact values.

---

### T26 — Multi-Tier Verification & Performance Benchmark Integration (`phalcom-perf`)

#### Purpose
Integrate compiler profile comparison into `phalcom-perf`, benchmark warm inline compilation against the baseline, and verify semantic parity across test corpora.

#### Preconditions
T25 complete.

#### Consumes
`phalcom-perf profile attach/compare`, representative fixtures (345 B, 485 B, 7.5 KB, 92 KB).

#### Produces
Benchmark report establishing elimination of the ~45s floor; comparative profiles against baseline.

#### Files and symbols
- **Read:** `phalcom-core/bin/perf/profile.rs`, `phalcom-core/bin/perf/store.rs`
- **Tests:** Run `phalcom-perf` comparison commands.

#### Required implementation shape
Execute comparative benchmarks across:
- 345 B fixture
- 485 B fixture
- 7.5 KB fixture
Compare cold versus warm; compare one-shot versus persistent session.

#### Forbidden approaches
- Do not claim success based on unoptimized `O0` tests alone; run benchmarks with optimized profile.

#### Tests to run now
```sh
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus corpus::classes
```

#### Acceptance
Warm compilation time for tiny fixtures drops from ~45 seconds to < 200 ms; scaling correlates with fixture size.

---

# 19. Verification gates

## G1 — Package A Structural Gate (Module & Interface Reuse)
- **Purpose:** Prove that warm one-shot compilation rebuilds zero canonical Universe interfaces and zero declaration catalogs.
- **Run:**
  ```sh
  cargo test -p phalcom-modules --test builtin_catalog
  cargo test -p phalcom-core --test core compiler_static_const
  ```
- **Expected:** Pass with `CanonicalUniverseProductBuilds == 0` and `UniverseSourceDeclarationCatalog::build == 0`.

## G2 — Package B Structural Gate (Presentation & Advisory Exclusion)
- **Purpose:** Prove that runtime execution compilation builds zero presentation modules and iterates zero advisory fixed points.
- **Run:**
  ```sh
  cargo test -p phalcom-semantic --test semantic product_request
  cargo test -p phalcom-semantic --test semantic advisory
  ```
- **Expected:** Pass with `PresentationUniverseRebuilds == 0` and `AdvisoryOuterIterations == 0`.

## G3 — Package C Semantic Gate (Witness Index & Conformance Parity)
- **Purpose:** Prove that direct indexed witness lookup and memoized generic proofs match existing conformance oracle exactly.
- **Run:**
  ```sh
  cargo test -p phalcom-semantic --test semantic witness_indexing
  cargo test -p phalcom-semantic --test semantic generic_conformance_query
  cargo test -p phalcom-core --test core language::traits
  cargo test -p phalcom-core --test core language::generic_conformance
  ```
- **Expected:** All focused conformance tests pass.

## G4 — Package D Metadata Gate (Profile Reachability & Pool 0 References)
- **Purpose:** Prove that user metadata bundles reference Pool 0 and contain zero unreferenced canonical declaration sweeps.
- **Run:**
  ```sh
  cargo test -p phalcom-core --test core metadata_reachability
  cargo test -p phalcom-core --test core cross_pool_metadata
  ```
- **Expected:** User metadata bundle size reduced by > 90%; VM reflection passes.

## G5 — Package E Backend Gate (Module Indexing & Lowering Independence)
- **Purpose:** Prove that runtime module lowering does not filter global Universe tables.
- **Run:**
  ```sh
  cargo test -p phalcom-core --test core module_indexes
  cargo test -p phalcom-core --test core execution_static_const
  ```
- **Expected:** Lowering scales with user module entity count.

## G6 — Checkpoint Full Acceptance Gate (Cardinality & Latency Elimination)
- **Purpose:** Prove the ~45s floor is eliminated across one-shot compilation while preserving full semantic parity.
- **Run:**
  ```sh
  cargo test -p phalcom-core --test core cardinality_gates
  cargo test -p phalcom-core --test language-corpus corpus::classes
  ```
- **Expected:** All cardinality counter assertions pass; fixture test body times reduce from ~45s to sub-second per fixture.

---

## 20. Final focused acceptance

At plan completion, the following matrix must be verified:

| Coverage ID | Test Selector | Expected Result |
|---|---|---|
| CV-01 | `phalcom-core::modules::compile_session::tests::cold_parity` | PASS |
| CV-02 | `phalcom-semantic::compilation_image::immutability_stress` | PASS |
| CV-03 | `phalcom-core::compiler::cardinality_gates::zero_canonical_rebuilds` | PASS |
| CV-04 | `phalcom-core::compiler::cardinality_gates::zero_canonical_body_analyses` | PASS |
| CV-05 | `phalcom-semantic::source_index::runtime_minimal_excludes_universe` | PASS |
| CV-06 | `phalcom-semantic::advisory::runtime_minimal_skips_fixed_point` | PASS |
| CV-07 | `phalcom-semantic::witness_indexing::explicit_and_default_lookup` | PASS |
| CV-08 | `phalcom-semantic::generic_conformance_query::memoized_reuse` | PASS |
| CV-09 | `phalcom-semantic::type_targets::single_pass_collector` | PASS |
| CV-10 | `phalcom-core::modules::compile_session::tests::failure_recovery` | PASS |
| CV-11 | `phalcom-core::compiler::diagnostic_parity` | PASS |
| CV-12 | `phalcom-core::compiler::metadata_reachability::minimal_profile` | PASS |
| CV-13 | `phalcom-core::compiler::module_indexes::lowering_complexity` | PASS |
| CV-14 | `phalcom-modules::builtin_catalog::custom_provider_isolation` | PASS |
| CV-15 | `phalcom-semantic::inferred_return_scc::cyclic_inference` | PASS |
| CV-16 | `phalcom-core::compiler::determinism::byte_identical_compilation` | PASS |
| CV-17 | `phalcom-semantic::module_value_staging::single_body_pass` | PASS |
| CV-18 | `phalcom-semantic::conformance_buckets::coherence_partitioning` | PASS |

---

## 21. Performance & structural cardinality evidence

Structural cardinality counters measured by `telos-observability` for a warm one-shot tiny fixture (e.g. `let x = 1` or 345 B class fixture) must satisfy:

| Counter / Work Metric | Prior Baseline | Required Result |
|---|---|---:|
| `canonical_universe_interfaces_rebuilt` | 148 | **0** |
| `universe_source_declaration_catalog_builds` | 1 | **0** |
| `canonical_universe_sources_in_workspace` | 40 | **0** |
| `canonical_callable_body_analyses` | 1,238 | **0** |
| `presentation_universe_modules_rebuilt` | 148 | **0** (in `RuntimeMinimal` mode) |
| `advisory_workspace_outer_iterations` | 3–5 | **0** (in `RuntimeMinimal` mode) |
| `redundant_snapshot_formal_projections` | 1 | **0** |
| `metadata_world_declaration_sweeps` | 1 | **0** |
| `metadata_callable_owner_scans` | O(Decl × Callables) | **0** (uses owner index) |
| `type_reference_target_collector_passes` | 3 + Impls | **1** per user module |
| `snapshot_full_map_materializations` | 8 | **0** |
| `canonical_conformance_proofs_recomputed` | ~1,000+ | **0** (reused from image) |

Wall-clock expectation: For warm process compilation under optimized test/release profiles, one-shot compilation latency for tiny fixtures must eliminate the fixed 45-second floor and scale directly with user code.

---

## 22. Checkpoint bookkeeping

### At plan start
- Mark `PERF003.C2` active in `docs/implementation/PERF003-runtime-performance/PROGRAM.md` and `STATUS.md`.
- Ensure `CHECKPOINT.md` records starting revision `e2ae4a0f` / `303222c3`.

### During plan execution
- Update `CHECKPOINT.md` only for durable events:
  - Completion of each Package phase (Phases 1–5).
  - Material architectural decision or consultation.
  - Recording newly classified deferred failures.
  - Successful passage of verification gates G1–G6.

### At plan completion
- Update `CHECKPOINT.md` status to `COMPLETE`, completion to `IMPLEMENTED`, verification to `FOCUSED_TESTED`.
- Create `PERF003.C2-walkthrough.md` and `PERF003.C2-handoff.md`.
- Set next action in `CHECKPOINT.md`.

---

## 23. Walkthrough deliverable specification

The post-execution walkthrough document `PERF003.C2-walkthrough.md` must record:
1. Executive summary of changes across packages A–E.
2. Exact source diff summary by crate.
3. Measured structural cardinality counters confirming all zero-invariants.
4. Wall-clock timing comparisons before and after optimization across benchmark fixtures (345 B, 485 B, 7.5 KB, 92 KB).
5. Deviations from the plan and recorded consultations.
6. Full test suite run log and classification of all passing/deferred tests.
7. Verification status truthfully reported as `FOCUSED_TESTED`.

---

## 24. Handoff deliverable specification

The handoff document `PERF003.C2-handoff.md` must supply:
1. Exact commit SHA and branch state.
2. Stable public and internal APIs established (`CanonicalUniverseCompilationImage`, `CompilationProductRequest`, `ConformanceWitnessIndex`, `CallableOwnerIndex`).
3. Summary of immutable invariants to be respected by future compiler/semantic tasks.
4. Guidelines for maintaining structural cardinality regression guards in future work.
5. First recommended verification commands for taking over the tree.

---

## 25. Completion truth table

```text
Source code implemented         != FOCUSED_TESTED
Focused tests passed            == FOCUSED_TESTED
FOCUSED_TESTED                  != RELEASE_COMPLETE
Clean git worktree              != Behavioral certification
Cardinally verified (0 rebuilds)== Architectural acceptance
```

Do not claim `COMPLETE` until all gates G1 through G6 have succeeded and evidence is recorded in `CHECKPOINT.md`.

---

## 26. Plan self-review checklist

- [x] Every invariant `INV-01` through `INV-10` has concrete implementation ownership in the tasks.
- [x] Every finding in `COMPILATION-1.md` and `COMPILATION-2.md` is addressed in Packages A–E.
- [x] Clear sequencing: lifecycle/ownership first (Package A), consumer profiling second (Package B), solver settlement third (Package C), metadata fourth (Package D), local loops fifth (Package E).
- [x] Objective STOP/CONSULT triggers defined for unexpected subsystem entries.
- [x] Debugging budgets bounded to prevent edit-test thrashing.
- [x] Testing ladder enforces narrow BUILD mode and defers broad suites.
- [x] Exact counter assertions specified in cardinality evidence table.
- [x] Deliverable walkthrough and handoff documents specified.
- [x] No placeholder text or ambiguous requirements remain.

## 2026-10-13 lifecycle amendment

Package B is tracked as PERF003.C2.P7. PERF003.C2.P2 is reserved for the separately adopted [formal-image/query-root plan](PERF003.C2.P2-canonical-formal-image-and-immutable-query-roots-implementation-plan.md). This amendment preserves existing Package B implementation and evidence, and does not adopt the complete proposed unified architecture.
