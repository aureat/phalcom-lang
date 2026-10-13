---
id: PERF003.C2
program: PERF003
kind: optimization
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: PERF003.C2.P1
latest_completed_plan: null
next_plan: PERF003.C2.P1
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head: 99b84eccf191dab1892fc50e027fc1ced180fb3b
---

# PERF003.C2 — Compilation Performance and Efficiency

## 0. Checkpoint state

```yaml
program: PERF003
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: PERF003.C2.P1
latest_completed_plan: null
next_plan: PERF003.C2.P1
next_action: >
  Complete the canonical Universe formal-image implementation and its focused
  tests. The current tree declares and tests canonical_image but does not yet
  contain its implementation module. Reconcile the proposed unified-artifact
  specification with the active plan before accepting architectural changes;
  preserve image/workspace ownership and do not cut compilation over until
  parity and build-once behavior are proven.
```

This record is the single durable state authority for `PERF003.C2`.

---

## 1. Bounded acceptance objective

Eliminate the ~45-second Universe-dominated compilation floor for both one-shot
and retained-session compilation. Transform the compiler pipeline from
repeatedly reconstructing the Universe compiler state into an immutable
**Canonical Universe Compilation Image** dependency model, where user fixtures
compile as small overlays with demand-driven product publication.

The checkpoint must establish:

```text
1. Immutable Canonical Universe Compilation Image
   (CanonicalUniverseCompilationImage)
   Process-shared, immutable, containing parsed modules, declaration catalog,
   interfaces, import products, topology, linked program, finalized TypeStore prefix,
   declarations, hierarchy, dispatch surfaces, callable signatures, algebraic products,
   conformance index, witness plans, trait dispatch, and reusable query roots.

2. One-shot compilation as an ephemeral overlay
   (ProgramCompiler::compile_entry_selection)
   Replaces the synthetic 40-module source injection and raw SemanticWorkspaceInput::new
   with an ephemeral WorkspaceModuleSession overlay on top of the canonical image.
   Revision U (canonical Universe) -> Revision U+1 (canonical Universe + user module).

3. Consumer-specific semantic product requests
   (CompilationProductRequest)
   Runtime execution compilation requests runtime-minimal semantics; eager
   generation of whole-Universe source presentation, advisory fixed points, and
   redundant formal projections is eliminated.

4. Incremental conformance and callable settlement
   Direct indexed conformance witness lookup, cached requirement views,
   separation of structural witness plans from return-sensitive completion,
   SCC-driven inferred-return propagation, and memoized generic conformance queries.

5. Root-driven metadata reachability architecture
   MetadataProfile enforced via explicit reachability closures from executable roots;
   cross-pool external metadata references for canonical Universe entities.

6. Elimination of O(N×M) backend scans
   Module-indexed callable and declaration tables, precomputed trait requirement-slot
   maps, and indexed inherent implementation domains.

7. telos-observability structural regression guards
   Automated cardinality assertions in CI ensuring that warm inline fixture compilation
   performs zero canonical reparses, rebuilds, body analyses, or presentation sweeps.
```

The target compilation model for a 500-byte user program:

```text
Before:
  ~45-47 seconds wall time
  148 Universe interfaces rebuilt
  UniverseSourceDeclarationCatalog rebuilt from scratch
  40 Universe source modules injected and re-analyzed
  1,238 canonical callable bodies analyzed
  Whole-Universe source presentation index constructed
  41-module advisory fixed point iterated
  Global metadata declaration × callable scan

After:
  < 200 ms wall time (in optimized profile)
  0 canonical Universe interfaces rebuilt
  0 UniverseSourceDeclarationCatalog builds
  0 canonical Universe sources in fixture semantic workspace
  0 canonical callable body analyses
  0 presentation Universe modules rebuilt (in runtime-minimal mode)
  0 advisory workspace iterations (in runtime-minimal mode)
  0 metadata sweeps over canonical declarations
```

---

## 2. Plan ledger

| Plan | Title | Package | Status | Completion | Verification | Notes |
|---|---|---|---|---|---|---|
| `PERF003.C2.P1` | Canonical Universe Compilation Image & One-Shot Overlay | Package A | IN_PROGRESS | PARTIAL | UNVERIFIED | Shared declaration/interface caches and workspace interface retention are established at `99b84ecc`; commit `a7b75980` adds canonical formal-image test scaffolding, but its declared `canonical_image` module is absent, and module-session cutover remains |
| `PERF003.C2.P2` | Consumer-Specific Publication & Demand-Driven Products | Package B | IN_PROGRESS | PARTIAL | UNVERIFIED | Earlier advisory-only slice has focused evidence; commit `a7b75980` adds `PresentationLevel` source-index filtering and Full → RuntimeMinimal → Full coverage changes, which remain unverified; unified source-target traversal remains |
| `PERF003.C2.P3` | Conformance Indexing & Inferred-Result Settlement | Package C | PROPOSED | NOT_STARTED | UNVERIFIED | Indexed witness lookup, cached requirement views, structural/final plan split, SCC return settlement, memoized generic proof query |
| `PERF003.C2.P4` | Profile/Root-Driven Metadata Reachability Architecture | Package D | PROPOSED | NOT_STARTED | UNVERIFIED | `MetadataRetentionPolicy`, reachability closures, `CallableOwnerIndex`, canonical metadata cross-pool references |
| `PERF003.C2.P5` | Backend Scans, Trait Slots, and Local Complexity Elimination | Package E | PROPOSED | NOT_STARTED | UNVERIFIED | Module-indexed backend tables, canonical trait-slot precomputation, inherent impl domain index, range lookup indexes |

---

## 3. Established architecture and invariants

- Baseline characterization (PERF003.C1) established that the ~45s floor is invariant to fixture size (345 B to 7.5 KB) and is caused by one-shot compilation repeatedly reconstructing canonical Universe state.
- `TypeStore` uses persistent `im` data structures and `fork_from_baseline()` preserves all canonical prefix IDs with identical numeric values while generating a fresh `TypeStoreId`.
- `SemanticWorkspaceSession` wraps updates in a `SemanticDb` undo-journal transaction; session forks must preserve canonical immutability.
- `telos-observability` provides exhaustive counters, hierarchical spans, and query metrics across compiler projection, semantic updates, TypeStore forks, and metadata export.
- `CanonicalUniverseProgram` already builds a complete semantic analysis of the Universe on first access; this formal result must be preserved in `CanonicalUniverseCompilationImage` rather than discarded.
- Canonical `UniverseSourceDeclarationCatalog` and interface projections are process-shared for the canonical provider; source-provider overrides remain isolated from those caches.
- `SemanticWorkspaceInput::new` retains the exact interfaces used to construct topology instead of forcing downstream reconstruction.
- `SemanticProductRequest::default()` preserves full tooling-compatible publication. Runtime compiler entry points explicitly request `runtime_minimal()`.
- Runtime-minimal publication retains non-Universe source/formal shards required by lowering, publishes no canonical Universe presentation shards, and publishes an empty advisory workspace.
- Canonical immutable dependencies must not be installed as workspace `last_snapshot`; the upcoming image remains a separate read-only lifetime domain.

---

## 4. Verification evidence

- Empirical characterization gathered in `COMPILATION-1.md` and `COMPILATION-2.md` confirms:
  1. `UniverseSourceDeclarationCatalog::build` and `InterfaceBuilder::build` are invoked for all Universe modules on every one-shot compilation.
  2. `ModuleResolver::load_interface` calls `build_from_parsed`, bypassing the static interface cache.
  3. `SemanticWorkspaceInput::new` derives topology and discards interfaces, forcing downstream re-derivation.
  4. `ProgramCompiler::compile_entry_selection` injects 40 Universe modules into an empty session where `last_snapshot = None`.
  5. `analyze_workspace` transiently computes the complete Universe semantic snapshot during `canonical_universe_program()`, but discards it immediately after lowering.
  6. Runtime compilation constructs a full presentation index for 148 Universe modules and runs a 41-module advisory fixed point.
  7. Metadata export scans all declarations and callables across the Universe regardless of requested profile.
- Prior focused demand-driven publication evidence recorded at execution head `99b84ecc` (does not cover the current working-tree delta):
  1. `runtime_minimal_skips_advisory_publication` passes with one non-Universe source/formal shard and zero advisory products/recomputation counts.
  2. `product_request_changes_replace_presentation_coverage` passes for `Full -> RuntimeMinimal -> Full` in one retained session.
  3. `incremental::workspace_indexing`: 19 passed.
  4. `phalcom-modules --test parse_cache`: 2 passed.
  5. `cargo check -p phalcom-core --features perf-observability`: passed.
- Detailed compiler-projection profile `compiler-projection-detailed-20261013T023039Z-50783` measured:
  1. source-index recomputation: 1 module, `0.364 ms` (previous comparable sample: 41 modules, `603.428 ms`);
  2. advisory: `0.750 us` (pre-bypass sample: `280.829 ms`);
  3. formal projection: `0.014 ms` (previous comparable sample: `151.359 ms`);
  4. total: `10.571 s`; semantic update: `10.441 s`; unchanged final callable reanalysis remains dominant at 3,896 passes / `3.618 s`.

**Current implementation increment — UNVERIFIED (commit `a7b75980`).** Since execution base `99b84ecc`, the semantic request now distinguishes presentation and advisory products. Runtime-minimal publication filters canonical Universe source-index shards while retaining local source/formal products, and publication effects/counters are narrowed to the requested products. Integration tests cover minimal publication and Full → RuntimeMinimal → Full presentation coverage. The tree also declares `phalcom_semantic::canonical_image` and adds a process-sharing/completeness test, but `phalcom-semantic/src/canonical_image.rs` is absent; the formal-image implementation and its test are incomplete. No test or build result in this record applies to commit `a7b75980`. Do not describe the canonical formal image as implemented or verified.

---

## 5. Deferred and baseline failures

| ID | Failure | Classification | Disposition |
|---|---|---|---|
| `corpus::booleans` | Fails before VM construction: legacy metadata cannot represent Record remainder outside Symbol key domain | D | Pre-existing baseline compiler limitation recorded in PERF003.C1; nonblocking for Package A/B/C/E. Package D will resolve metadata reachability. |
| `perf_universe_characterization` VM::new() | Fails at runtime: intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration | D | Pre-existing runtime bootstrap blocker recorded in PERF003.C1; nonblocking for compiler and semantic optimization. |
| `corpus::blocks` | `blocks_argument_to_method.ph` fails with `[1, 2, 3, 4] does not understand 'fold(initial,using)'` | D | Pre-existing missing library method recorded in PERF003.C1. |
| `semantic::integration::advisory::analysis::formal_projection_preserves_causal_invalidity_and_binding_contracts` | Expected one causal-invalidity marker but current formal projection publishes a different state | D | Reproduces in isolation outside the demand-driven request path; deferred for the owning formal-projection work. |
| `incremental::type_store_revisions` callable-count assertions | Existing first-publication callable recomputation counts differ from stale expected values | D | Reproduces in isolation after all source/formal/advisory assertions pass; nonblocking for product-request work. |

---

## 6. Consultations and amendments

- Consultation based on `COMPILATION-1.md` and `COMPILATION-2.md` confirmed the 5-package architecture.
- Reaffirmed: no workaround harnesses (e.g. changing `run_inline` alone); optimization must live underneath `ProgramCompiler::compile_entry_selection`.
- Reaffirmed: canonical image must be strictly immutable; sessions compile user code as persistent overlays.
- Reaffirmed: structural work cardinality (measured via `telos-observability`) is the primary gate, preventing faster hardware from masking algorithmic regressions.
- `PERF003-C2-Unified-Compiler-Artifact-Implementation-Spec.md` was added as a proposed architecture for review. It is not an adopted plan amendment; the active detailed implementation plan and this checkpoint remain the execution and lifecycle records until an explicit decision is recorded.

---

## 7. Next action

Complete `phalcom-semantic/src/canonical_image.rs` and the focused formal-image test, then verify the current Package A/B working-tree changes. Keep the image separate from workspace `last_snapshot`; prove semantic parity and one build per process/world before canonical-aware query lookup, source-ownership changes, or compiler cutover.
