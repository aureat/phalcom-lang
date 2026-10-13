---
id: PERF003.C2
program: PERF003
kind: optimization
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: PERF003.C2.P2
latest_completed_plan: null
next_plan: PERF003.C2.P2
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head: c931faeaf801cdf92b7a39c77404792e94a60026
latest_tested_predecessor: bf1a69c6089e4879bbaa024fc202e14a808147a7
---

# PERF003.C2 — Compilation Performance and Efficiency

## 0. Checkpoint state

```yaml
program: PERF003
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: PERF003.C2.P2
latest_completed_plan: null
next_plan: PERF003.C2.P2
next_action: >
  Continue the image-backed ordinary semantic/module-session and external-module
  ownership cutover. The image, core acquisition seam and opt-in canonical database
  parent are implemented. P3 exact witness lookup and module-bounded signature
  enumeration are implemented. User explicitly deferred broad verification and
  requested continued compiler optimizations; P2 G5/G6 remain deferred.
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
| `PERF003.C2.P1` | Canonical Universe Compilation Image & One-Shot Overlay | Package A | IN_PROGRESS | PARTIAL | UNVERIFIED | Shared declaration/interface caches and workspace interface retention are established at `99b84ecc`; commit `a7b75980` added formal-image scaffolding; P2 now owns the implemented formal/query foundation; module-session cutover remains |
| `PERF003.C2.P7` | Consumer-Specific Publication & Demand-Driven Products | Package B | IN_PROGRESS | PARTIAL | UNVERIFIED | Earlier advisory-only slice has focused evidence; commit `a7b75980` adds `PresentationLevel` source-index filtering and Full → RuntimeMinimal → Full coverage changes, which remain unverified; unified source-target traversal remains |
| `PERF003.C2.P2` | Complete Canonical Universe Formal Image and Immutable Canonical Query Roots | Formal/query foundation | IN_PROGRESS | PARTIAL | UNVERIFIED | Formal image, query freeze/reader, provenance validation and core acquisition implemented in the task working-tree patch over `82051480`; focused evidence below; G5/G6 certification remains deferred |
| `PERF003.C2.P3` | Conformance Indexing & Inferred-Result Settlement | Package C | IN_PROGRESS | PARTIAL | UNVERIFIED | Exact indexed witness lookup and module-bounded effective-surface signature enumeration implemented; one focused lookup/edit regression passed. Requirement caching, structural/final plan split, SCC return settlement and generic proof memoization remain |
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

**Inherited implementation increment — UNVERIFIED (commit `a7b75980`, before P2).** Since execution base `99b84ecc`, the semantic request now distinguishes presentation and advisory products. Runtime-minimal publication filters canonical Universe source-index shards while retaining local source/formal products, and publication effects/counters are narrowed to the requested products. Integration tests cover minimal publication and Full → RuntimeMinimal → Full presentation coverage. That revision declared `phalcom_semantic::canonical_image` and added process-sharing/completeness scaffolding, but lacked `canonical_image.rs`. P2 supplies the module in the current task patch. No test or build result in this record applies to commit `a7b75980`. The old evidence does not certify the new P2 patch.

### P2 task patch (2026-10-13, committed at `bf1a69c6`; lifecycle docs `c931faea`)

- Semantic-owned builder runs the ordinary complete canonical update, then freezes
  committed products. Core obtains its snapshot without a second formal analysis.
- Module-owned `CanonicalModuleProducts` is the explicit bounded P1 dependency
  adapter. Full external-module image/one-shot overlay cutover remains successor work.
- Query roots have exact world source provenance, final type/static roots, typed
  payloads, immutable forward observations and exhaustive row rejection accounting.
  Reader binding rejects foreign worlds/prefixes/static roots. An explicit
  `SemanticDb::with_canonical_image` now supplies immutable-origin read-through;
  ordinary session constructors are not switched yet. Local state, invalidation,
  discard, and rollback remain local. Canonical dependencies retain forward
  observations without local reverse edges; canonical publication is rejected.
- Focused concurrent first-access test: PASS (1 selected), eight callers / one
  image build and complete semantic update; ten warm readers add no analysis.
  Unoptimized debug sample: cold 70.812812667 s; ten warm readers 32.958 µs.
  Census: 148 modules, 9,318 rows examined, 2,763 retained, 6,555 rejected, 2,432
  final types. Ordinary publication retains four internal incidents; preserved,
  not investigated or certified by this task.
- First focused freeze/reader unit gate: PASS (5 selected). Final expanded unit
  and final build-check results are recorded in the walkthrough.
- The user explicitly requested fast code delivery and no baseline investigation.
  Source/provider differential parity, core runtime parity, broader existing
  publication suites and the full G6 performance matrix are deferred. P2 remains
  IN_PROGRESS / PARTIAL / UNVERIFIED; focused checks do not complete the plan.

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

- L2 (2026-10-13): User requested continuing the compiler optimizations in the
  attached “Canonical Universe Image Implementation and Next Major Performance
  Optimizations” response and explicitly deferred broader tests/verification.
  This expands the earlier image-only scope to successor optimizations. Implemented
  section 10's opt-in database parent and section 15.1's exact witness lookup.
  Full session/module ownership cutover is still pending; no broad performance or
  semantic parity claim follows from these focused increments.

- L0 (2026-10-13): User approved reserving P2 for the attached formal-image/query-root plan and moving existing Package B publication work to P7. No existing P7 ledger package was found. Historical evidence is preserved. Adopt only D1–D10 of the new P2 plan for this scope; the unified specification remains proposed as a whole. No workspace source-ownership cutover is authorized by this amendment. Starting head `82051480`; latest tested predecessor `99b84ecc`; baseline `cargo check -p phalcom-semantic` fails E0583 for the missing image module.

- Consultation based on `COMPILATION-1.md` and `COMPILATION-2.md` confirmed the 5-package architecture.
- Reaffirmed: no workaround harnesses (e.g. changing `run_inline` alone); optimization must live underneath `ProgramCompiler::compile_entry_selection`.
- Reaffirmed: canonical image must be strictly immutable; sessions compile user code as persistent overlays.
- Reaffirmed: structural work cardinality (measured via `telos-observability`) is the primary gate, preventing faster hardware from masking algorithmic regressions.
- `PERF003-C2-Unified-Compiler-Artifact-Implementation-Spec.md` was added as a proposed architecture for review. It is not an adopted plan amendment; the active detailed implementation plan and this checkpoint remain the execution and lifecycle records until an explicit decision is recorded.

---

## 7. Next action

Continue ordinary semantic sessions over the completed image and immutable external
module catalog, then remove canonical source discovery/seeding at that boundary.
Keep canonical snapshots separate from local `last_snapshot`/`last_known_good` and
preserve final arena prefixes, local negative dependencies, and conformance coherence.
The opt-in DB API is ready; no production ordinary-session cutover is claimed.
Broader source/provider/runtime parity and the observability matrix are deferred
at the user's request. See [walkthrough](PERF003.C2.P2-walkthrough.md),
[handoff](PERF003.C2.P2-handoff.md), and
[P3 increment](PERF003.C2.P3-indexed-witness-lookup-implementation-plan.md).

### Focused successor evidence (2026-10-13)

- `canonical_image_database_reads_are_immutable_leaves_across_local_rollback`:
  PASS, 1 selected, 69.43 s including cold canonical bootstrap. Verifies identity
  sharing, canonical-origin currentness/reuse, forward dependency recording,
  no canonical invalidation/retirement/publication, and local transaction rollback.
- `conformance_witness_lookup_preserves_exact_owner_side_and_selector_after_edits`:
  PASS, 1 selected. Exact impl/side lookup distinguishes inherent declarations,
  survives table cloning, and observes removals. Final rerun also passed module
  enumeration assertions (1 selected, 0 failures; 11.64 s compilation).
- Existing image evidence: 8 focused exporter/provenance tests and one concurrent
  first-access integration test passed before this successor increment. These
  results do not certify ordinary-session externalization or runtime performance.

- Final `RUSTFLAGS='' cargo check -p phalcom-core --features perf-observability`:
  PASS (9.21 s) after the database-parent and indexed lookup changes. Subsequent
  error-message-only adjustment identifies immutable publication distinctly.
