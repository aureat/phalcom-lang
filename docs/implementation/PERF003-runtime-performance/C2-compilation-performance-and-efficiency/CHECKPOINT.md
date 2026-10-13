---
id: PERF003.C2
program: PERF003
kind: optimization
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
active_plan: PERF003.C2.P5
latest_completed_plan: PERF003.C2.P3
next_plan: PERF003.C2.P6
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head: e74a9213
latest_tested_predecessor: c34f9c3fee75f33304a85c229dcf0a5133816479
---

# PERF003.C2 — Compilation Performance and Efficiency

## 0. Checkpoint state

```yaml
program: PERF003
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
active_plan: PERF003.C2.P5
latest_completed_plan: PERF003.C2.P3
next_plan: PERF003.C2.P6
next_action: >
  Normalized P5 (Canonical Publication & Persistent Image Boundary) in progress:
  T0-T4 complete in committed head 84227536. Provider isolation in canonical Universe
  declaration bootstrap resolved and verified across all 4 canonical image integration
  tests and 10 query/freeze unit tests. T5/T6 Slice 1 is implemented in aaac8106:
  DeclarationTypeTable, CallableSignatureTable, FieldSignatureTable, and
  MapTypeHierarchy now use immutable parent overlays, and image-backed sessions retain
  the canonical image while avoiding O(Universe) aggregate table copies. Slice 1 focused
  unit and canonical-image gates pass. Workspace and recomputation gates remain
  baseline-blocked by failures reproduced on clean 84227536; see the P5 handoff.
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
| `PERF003.C2.P2` | Complete Canonical Universe Formal Image and Immutable Canonical Query Roots | Formal/query foundation | IN_PROGRESS | PARTIAL | UNVERIFIED | Formal image, query freeze/reader, provenance validation and core acquisition implemented in the task working-tree patch over `82051480`; focused evidence below; G5/G6 certification remains deferred |
| `PERF003.C2.P3` | Conformance Indexing & Inferred-Result Settlement | Package C | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Exact indexed witness lookup, module-bounded effective-surface signature enumeration, ConformanceIndex partitioning (nominal family + open heads) with brute-force parity, SCC inferred-return settlement, and shared requirement-view passing implemented; focused regressions pass. Retained structural witness plans across revisions and generic proof memoization remain |
| `PERF003.C2.P4` | Profile/Root-Driven Metadata Reachability Architecture | Package D | IN_PROGRESS | PARTIAL | BASELINE_BLOCKED | User-stopped working-tree continuation over e74a9213. T1 accepted; retained-session profile test now passes after bounded T5 export. PR-A classification repair passes 9 source-index tests including actual corpus. T5 nested nominal reachability passes focused tests. PR-B schema migration (v13 heterogeneous Record keys) and T6 canonical authority remain bounded prerequisites; T6–T9 remain gated |
| `PERF003.C2.P5` | Canonical Publication and Persistent Image Boundary | Canonical publication / image boundary | IN_PROGRESS | PARTIAL | UNVERIFIED | Normalized P5: explicit canonical compiler-image product contract, consumer-specific publication cleanup, immutable canonical aggregate-parent architecture, query persistence classification, and formal persistence inventory |
| `PERF003.C2.P6` | User-Only Backend/Local Complexity Closure | Package E (historical P5) | PROPOSED | NOT_STARTED | UNVERIFIED | Normalized P6 (formerly P5): module-indexed backend tables, canonical trait-slot precomputation, inherent impl domain index, range lookup indexes, elimination of O(N×M) backend scans |
| `PERF003.C2.P7` (historical) | Consumer-Specific Publication & Demand-Driven Products | Package B (historical) | IN_PROGRESS | PARTIAL | UNVERIFIED | Historical label retained for lifecycle tracking; consumer-specific publication semantics are normalized into P5 (Slice B), while normalized P7 represents the .telosc container schema and persisted semantic compiler image |

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
- `MetadataReachabilityClosure` enforces `MetadataProfile` via explicit reachability traversal from roots (`MetadataRootSet`).
- Canonical Universe declarations, callables, and fields (`module_id.project.is_universe()`) are elided from ordinary user bundles by the reachability closure; only non-universe local entities and cross-pool nominal references are retained.
- `CallableSignatureTable` and `FieldSignatureTable` maintain declaration-owner reverse indexes (`owner_callables` and `owner_fields`), eliminating quadratic $O(\text{declarations} \times \text{all callables})$ scans.
- `telos-observability` tracks metadata reachability, canonical elision, and runtime lookup metrics via 14 new `PerfCounter` variants.
- T1 adds `ProgramCompilationOptions` and explicit analyzed/session compilation
  entry points. Existing APIs and canonical bootstrap default to RuntimePublic;
  semantic publication remains independent. The profile reaches the existing
  exporter, but production retention is still aggregate until T5.

---

## 4. Verification evidence

### P4 user-stopped T5 / PR-A continuation (2026-10-13, uncommitted over `e74a9213`)

- `metadata_profile_is_explicit_and_independent_of_semantic_publication`: PASS,
  1 selected, 43.93 s. Actual retained tiny compilation retains default public,
  accepts explicit minimal, returns to default public, and retains independent
  semantic publication. This supersedes the prior failure for that fixture.
- `retained_alias_error_keeps_root_and_unreachable_alias_is_not_exported`: PASS,
  1 selected, 0.01 s. Unretained alias omitted; explicitly retained String-key
  Record alias fails with entity/type context. It does not test remainder domains.
- `compiler_intrinsic_data_has_no_native_class_anchors`: observed FAIL before
  PR-A on ProductKind; after repair `native::source::tests`: PASS, 9 selected,
  0.05 s, including actual bundled source corpus and identity rejection tests.
- `metadata::reachability::tests`: PASS, 4 selected (including new
  `reachability_closure_discovers_nested_nominal_dependencies`), 0.01 s.
  Verifies recursive exporter-driven nominal discovery through Record,
  Callable, Pack, and AssociatedProjection, canonical module elision,
  failure rollback, and RuntimeMinimal filtering.
- No canonical singleton rerun, schema change/test, core integration reflection,
  broad suite, benchmark, independent review, or final formatting gate completed.
- Production T5 work is partial. Current closure traversal must be reconciled
  with the exporter's full recursive type graph before claiming completeness.
- No commit or push for this continuation. Source edits stopped immediately on
  user instruction; only lifecycle records updated afterward.


### P4 inline options continuation (2026-10-13, uncommitted over `a82dfe32`)

- Core `cargo check` with `RUSTFLAGS=''` and `perf-observability`: PASS (18.99 s).
- `empty_projection_preserves_explicit_metadata_profile`: PASS, 1 selected;
  default public and four explicit profile headers. Plumbing evidence only.
- Retained-session options regression: FAIL, 1 selected, inherited non-Symbol
  Record remainder export error before header assertions.
- Exact canonical singleton regression: FAIL, 1 selected, ProductKind
  source-index rejection before projection; canonical authority unavailable.
- Scoped diff hygiene passes. Broad compiler/reflection/runtime/performance gates
  deferred pending the incident ruling. No P4 completion or speedup claimed.

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
| `P4 canonical authority prerequisite` | Historical singleton failure at ProductKind source indexing | D | PR-A classification repaired with 9 focused source-index PASS, including actual corpus. Singleton/projection not rerun after repair; PR-B still blocks faithful canonical metadata and T6 proof. |
| `P4 retained-session options regression` | Historical ordinary aggregate export failure on non-Symbol Record remainder | D, resolved for exact tiny fixture | After T5 slice, default public → explicit minimal → default public test PASS (1 selected). Required retained heterogeneous Record representation remains a separate PR-B prerequisite. |
| `corpus::blocks` | `blocks_argument_to_method.ph` fails with `[1, 2, 3, 4] does not understand 'fold(initial,using)'` | D | Pre-existing missing library method recorded in PERF003.C1. |
| `semantic::integration::advisory::analysis::formal_projection_preserves_causal_invalidity_and_binding_contracts` | Expected one causal-invalidity marker but current formal projection publishes a different state | D | Reproduces in isolation outside the demand-driven request path; deferred for the owning formal-projection work. |
| `incremental::type_store_revisions` callable-count assertions | Existing first-publication callable recomputation counts differ from stale expected values | D | Reproduces in isolation after all source/formal/advisory assertions pass; nonblocking for product-request work. |

---

## 6. Consultations and amendments

- P4 architectural ruling accepted from the user (2026-10-13):
  [full ruling](PERF003.C2.P4-architectural-ruling.md). T1 stays unchanged.
  T5 continues with authoritative roots, no aggregate production fallback,
  and structured export provenance. PR-A classifies compiler-owned intrinsic
  data separately from native Tuple/Record/Pack facades, preserving fail-closed
  canonical identity checks. PR-B extends metadata for heterogeneous fixed
  Record keys and explicit remainder key domains, including scoped/row forms,
  fingerprints and legacy Symbol-key decoding. No lossy export or public
  surface omission. T6–T9 remain behind both prerequisites.

- P4 T5 reachability consultation (2026-10-13): following review of
  `metadata/reachability.rs`, the closure's partial type walker conflicted with
  the plan's exporter-owned recursive type traversal and missed nested local
  nominals. The adopted ruling is that a disposable `MetadataExporter` discovers
  nominal references through `export_type_form`; RuntimePublic adds new local
  declarations to roots and recomputes closure to a fixed point. Profile and
  external-identity filtering remain in the closure engine, and final export
  preserves retained-entity error provenance. The plan amendment is recorded
  in the active P4 plan. Implementation is present but unverified; no tests
  were run for this follow-up.

- P4 continuation, 2026-10-13: mandatory trigger 2 raised because the canonical
  compiled metadata authority cannot be built. The incident packet is
  [PERF003.C2.P4-canonical-authority-incident.md](PERF003.C2.P4-canonical-authority-incident.md).
  User required inline work with no subagents; no adviser was spawned and no
  ruling was assumed at that time. The subsequent accepted ruling above resolves
  that request and authorizes PR-A/PR-B; existing profile/semantic rules stand.

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

- `impls::conformance_index_partition_tests::partitions_queries_and_coherence_match_brute_force_through_mutations`:
  PASS, 1 selected, 0.01 s. Verifies exact equivalence between partitioned index
  (by trait, nominal family, and open heads) and brute-force query and overlap
  conflicts across insertions, mutations, and source removals.
- `semantic::incremental::callable_dependencies::inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once`:
  PASS, 1 selected, 43.24 s. Verifies that SCC and affected-result worklist
  propagation preserves stable downstream products without global rescans, and
  updates body dependency edges.
- `missing_qualified_module_value_export_invalidates_when_export_is_added`:
  PASS, 1 selected. Verifies negative export dependency revalidation when the
  export is added in a subsequent revision.
- `qualified_module_value_dependencies_follow_reads_not_import_seeding_or_shadowed_names`:
  PASS, 1 selected. Verifies that imported and local checked bindings record
  provenance edges only when looked up, not on blanket import presence.
- `from_interfaces_retains_authoritative_products_without_rebuilding`:
  PASS, 1 selected. Verifies that `SemanticWorkspaceInput::from_interfaces` preserves
  pre-derived interface pointers without rebuilding.
- `canonical_interface_load_does_not_load_parsed_source_per_resolver`:
  PASS, 1 selected. Verifies canonical Universe interfaces bypass parsed source loading.
- `canonical_image_database_reads_are_immutable_leaves_across_local_rollback`:
  PASS, 1 selected, 69.43 s including cold canonical bootstrap. Verifies identity
  sharing, canonical-origin currentness/reuse, forward dependency recording,
  no canonical invalidation/retirement/publication, and local transaction rollback.
- `conformance_witness_lookup_preserves_exact_owner_side_and_selector_after_edits`:
  PASS, 1 selected. Exact impl/side lookup distinguishes inherent declarations,
  survives table cloning, and observes removals.
- `metadata::reachability::tests::metadata_root_set_tracks_cardinalities`:
  PASS, 1 selected, 0.00 s. Verifies root tracking and cardinality computations.
- `metadata::reachability::tests::reachability_closure_minimal_vs_public_retention`:
  PASS, 1 selected, 0.01 s. Verifies profile-driven retention for `RuntimeMinimal` vs `RuntimePublic`.
- `metadata::reachability::tests::reachability_closure_elides_canonical_authorities`:
  PASS, 1 selected, 0.01 s. Verifies that canonical Universe declarations, callables, and fields are elided as external authorities.
- `signature::tests::callable_owner_index_tracks_membership_removal_and_cloning`:
  PASS, 1 selected, 0.00 s. Verifies secondary owner index in `CallableSignatureTable`.
- `signature::tests::field_owner_index_tracks_membership_removal_and_cloning`:
  PASS, 1 selected, 0.00 s. Verifies secondary owner index in `FieldSignatureTable`.
- `semantic::integration::canonical_image`:
  PASS, 4 selected, 135.74 s. Verifies complete canonical formal image properties:
  Send/Sync thread safety, process sharing across concurrent threads, external
  Universe isolation across local replacement, and source parity and provider isolation.
- `canonical_formal_image_source_parity_and_provider_isolation`:
  PASS, 1 selected. Verifies provider-isolated Universe declaration bootstrap with custom source overrides.
- `canonical_image::tests`:
  PASS, 10 selected, 39.27 s. Verifies canonical compiler image product completeness validation,
  frozen query selection, nonready query classification, database read immutability across rollback,
  world provenance digest collision rejection, and root validation.
- `declarations::DeclarationTypeTable` parent-aware architecture:
  Verified compilation and type checking with `with_parent`, delegation lookups, parent chain `iter()`, and local-only mutation.
- `RUSTFLAGS='' cargo check -p telos-observability`: PASS (clean).
- `RUSTFLAGS='' cargo check -p phalcom-semantic`: PASS (clean, 0 warnings, 0 errors).
- `RUSTFLAGS='' cargo check -p phalcom-core --features perf-observability`: PASS (17.16 s).
- `RUSTFLAGS='' cargo check -p phalcom-modules`: PASS (0.17 s).
- `git diff --check`: PASS (clean).
