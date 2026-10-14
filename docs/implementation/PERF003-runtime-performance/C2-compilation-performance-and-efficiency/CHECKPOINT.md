---
id: PERF003.C2
program: PERF003
kind: optimization
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
latest_completed_plan_verification: FOCUSED_TESTED
active_plan: PERF003.C2.P7
latest_completed_plan: PERF003.C2.P6
next_plan: PERF003.C2.P8
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head: 3ca30fca
latest_tested_revision: 69c618c8
latest_tested_predecessor: c34f9c3fee75f33304a85c229dcf0a5133816479
---

# PERF003.C2 — Compilation Performance and Efficiency

## 0. Checkpoint state

```yaml
program: PERF003
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
latest_completed_plan_verification: FOCUSED_TESTED
active_plan: PERF003.C2.P7
latest_completed_plan: PERF003.C2.P6
next_plan: PERF003.C2.P8
next_action: >
  Execute P7.T0/T5 shared export-context and persistence-manifest refinement,
  then T7.A structural primary contracts before T6 static ADT relocation.
  Follow P7/PERF003.C2.P7-remaining-execution-plan.md through T15, then the
  prepared P8 detailed plan after P7 compiler interfaces are implemented.
  Compiler-only checks are authorized; behavioral acceptance remains deferred.
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
| `PERF003.C2.P4` | Profile/Root-Driven Metadata Reachability Architecture | Package D | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | T1 options, reachability closure T2–T5, PR-A intrinsic classification, PR-B schema v13/contextual decode/fingerprints, canonical external authority T6, registry stable indexes T7, cross-pool reflection T8, compatibility validation T9, and observability guards T10 implemented and certified. |
| `PERF003.C2.P5` | Canonical Publication and Persistent Image Boundary | Canonical publication / image boundary | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Explicit compiler image product contract, zero-copy parent overlays across all core tables, canonical persistence inventory/capabilities, query classification, CanonicalCompilerImageView boundary, and observability cardinality guards (CanonicalAggregateEntriesCopied == 0) implemented and certified. |
| `PERF003.C2.P6` | User-Only Backend/Local Complexity Closure | Package E | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | ModuleLoweringInputView, module-indexed callable analyses/definitions/declarations/data/enums, reverse-target occurrence resolution, precomputed TraitRequirementLayout, and direct inherent impl domain indexing implemented and certified. |
| `PERF003.C2.P7` (historical) | Consumer-Specific Publication & Demand-Driven Products | Package B (historical) | IN_PROGRESS | PARTIAL | UNVERIFIED | Historical label retained for lifecycle tracking; consumer-specific publication semantics are normalized into P5 (Slice B), while normalized P7 represents the .telosc container schema and persisted semantic compiler image |
| `PERF003.C2.P7` (normalized) | Telosc persisted semantic compiler image | Compiler artifact | IN_PROGRESS | PARTIAL | UNVERIFIED | T1–T5 code present; remaining execution companion prepared; T6–T12 code and G7 acceptance remain open |
| `PERF003.C2.P8` | Portable executable Universe image | Executable artifact | PLANNED | NOT_STARTED | UNVERIFIED | Detailed plan prepared; BUILD depends on complete P7 compiler interfaces; deferred P7/P8 acceptance still controls closure |

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
| `P4.T6 / P5.T10 canonical compiler projection` | Historical singleton and reflection test failure during `universe:numbers.complex` lowering: missing executable `Zero.zero` or `One.one` class-side trait slot | Resolved | [Incident packet](PERF003.C2.P5-trait-evidence-slot-incident.md): Root cause was `publish_conformance_witness_plans` in `session.rs` unconditionally skipping Universe conformances during canonical image build. Constrained by `db.canonical_image().is_some()`; `canonical_universe_program_is_process_singleton` PASS (43.19 s) and row display PASS. |
| `cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata` | Local semantic test workspace rejects canonical Universe sources before metadata export | D for PR-B; P5 fixture compatibility unclear | One bounded classification pass: failure is `canonical Universe sources cannot enter a local workspace; use an explicit provider image` at workspace update. Preserve the fixture; address in the workspace/provider owning work rather than changing expected metadata. |
| `corpus::blocks` | `blocks_argument_to_method.ph` fails with `[1, 2, 3, 4] does not understand 'fold(initial,using)'` | D | Pre-existing missing library method recorded in PERF003.C1. |
| `semantic::integration::advisory::analysis::formal_projection_preserves_causal_invalidity_and_binding_contracts` | Expected one causal-invalidity marker but current formal projection publishes a different state | D | Reproduces in isolation outside the demand-driven request path; deferred for the owning formal-projection work. |
| `incremental::type_store_revisions` callable-count assertions | Existing first-publication callable recomputation counts differ from stale expected values | D | Reproduces in isolation after all source/formal/advisory assertions pass; nonblocking for product-request work. |

---

## 6. Consultations and amendments

- P4 prerequisite resumption (2026-10-13): user authorized resuming the
  previously stopped P4 PR-B/schema and canonical RuntimePublic metadata work
  required by P5.G1. The user specifically approved decode-time graph
  normalization for pre-v13 metadata: insert canonical Symbol key-domain
  nodes, relocate affected global/scoped node references and prove old-fixture
  compatibility. The accepted no-lossy-export, distinct semantic/runtime
  identity and complete RuntimePublic invariants remain binding. This is
  authorization to implement and verify, not evidence that P4 or P5 passed.

- P4.PR-B legacy Symbol identity clarification (2026-10-13): valid old bundles
  can have Record fields without a Symbol type node; the bundle header does not
  supply a stable canonical Universe Symbol declaration identity. The user
  selected loader-supplied, validated canonical Symbol identity for graph
  normalization. Standalone decode of an affected legacy bundle without that
  context must return a typed need-for-context error rather than guess from
  `producer_version`. Unaffected legacy bundles retain normal standalone
  decoding. This is an approved Level 1 identity clarification; it does not
  weaken pre-v13 compatibility for correctly contextualized loaders.

- P5 conformance ownership clarification (2026-10-13): the live first-update
  path clones canonical `ConformanceIndex` (five correlated maps), witness-plan
  map, and `DerivedConformanceRuleIndex` (trait buckets and impl map). Replacing
  these with a simple parent map would change joint query/coherence behavior and
  `rules_for_trait` slice semantics. The user selected a layered canonical/local
  conformance view with merged query and coherence traversal and exact parity
  tests. No G6 copy exception is approved; conformance evidence must not be
  strengthened by aggregation or fail-open negative lookup.

- P4.PR-B fingerprint migration clarification (2026-10-13): decode-time
  normalization of pre-v13 Record graphs must retain their existing v12
  structural fingerprints as version-tagged legacy provenance. These are not
  equivalent to the v13 semantic export fingerprints; cross-version digest
  comparisons must refuse compatibility until semantic authority re-exports
  the normalized graph. Do not duplicate semantic fingerprint computation in
  `phalcom-type-meta` merely to migrate legacy bytes. This choice preserves
  one hashing authority and a fail-closed mixed-version boundary.

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

The current route is [P7 remaining execution](P7/PERF003.C2.P7-remaining-execution-plan.md),
starting with the shared guarded deterministic export context/persistence manifest,
then structural primary contracts required by static ADT relocation. The
[P8 detailed plan](P8/PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md)
follows once the P7 compiler interface is implemented. Earlier P2/P4/P5 next-action
notes below are historical evidence, not current routing.

Use compiler-driven checks only. P7/P8 behavioral acceptance and source-free proofs
remain deferred; their statuses must not be promoted from inherited P6 evidence.
Preserve the dirty implementation patch and documentation moves. No commit/push is requested.

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


### P5 dependency authority clarification (2026-10-13)

Adviser ruling: canonical query freeze eligibility and immutable aggregate
publication are distinct. Keep query freeze fail-closed. An image-owned derived
index may expose exact positive structural dependency fingerprints from the
validated final snapshot, without synthesizing Ready rows or reusable queries.
Local states take precedence. Hierarchy evidence must preserve its validated
source-sensitive owner fingerprint and agree with final declaration/parent
identity. Frozen-query overlap must agree. Record exact edges and treat these
as immutable leaves in dependency validation. This is a Level 1 clarification
of P5, with no new inference or mutable semantic authority.

Alias oracle clarification: invalid cycle participants and transitive alias
dependents must be quarantined; unaffected canonical and valid local aliases
must survive. Global-empty table assertions are stale fixture assumptions.


### P5 parent-overlay and dependency continuation (2026-10-13, committed as `e71057d9` over `0b40dbbb`)

- `ParentMap` provides O(1) immutable parent attachment, deterministic parent/local
  iteration, local-only mutation, and rejection of canonical identity collisions.
- Dispatch surfaces, conditional-member sets, type registrations, data/enum,
  associated surfaces, enum requirements, and type-member tables use shared
  canonical parents. Published snapshot surfaces share dispatch's map. Resolver
  associated/type-member views retain these maps without flattening the parent.
- First-update factories use the image-backed base overlays rather than cloning
  the canonical snapshot; retained updates copy only local contributions.
- Exact aggregate dependency fingerprints preserve edge evidence without granting
  aggregate-only products query readiness. Frozen roots must agree with settled
  structural authority; stale rows and their transitive dependents are rejected.
- Alias fingerprints include the formed result, fixing imported-alias relowering
  with unchanged consumer syntax. Cycle quarantine removes participants and their
  alias dependents, cached shells/static definitions, and retained resolver forms.
  Unaffected canonical and valid local aliases survive, including valid→cycle→valid.
- Adviser confirmed foreign inherent implementations of canonical declarations
  are forbidden by `docs/specs/objects/impl.md`; strict parent collisions remain.
- Independent focused review found a computed-key readiness consumer; corrected
  to exact dependency evidence. No remaining actionable review findings.

Completed evidence:

- `incremental::recomputation_metrics`: PASS, 13 selected, 120.97 s. All four
  inherited failures now pass; no shell assertion was weakened.
- `integration::workspace`: PASS, 18 selected, 44.89 s, including the new
  quarantine/preservation/recovery regression.
- `capabilities::aliases`: PASS, 7 selected, 6.57 s. Three stale global-empty
  alias assertions now check exact invalid identities, preserving diagnostics
  and declaration absence.
- Final `--lib canonical_image`: PASS, 12 selected, 41.84 s. Includes aggregate-only
  exact evidence, invented-key rejection, fingerprint mismatch rejection,
  local-state precedence, and stale structural freeze/dependent closure.

Residual: ordinary session DB structural rows can lag final inferred-return
settlement. This continuation excludes stale rows from canonical freezing;
certification of general settlement is not claimed. Residual copied maps
(alias/static products, callable definitions and other P5.T6 Slice 4 families),
T7–T13, P4-dependent metadata composition, persisted `.telosc`, broad release
verification, and timing certification remain open. Overall P5 remains
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

Final continuation gates:

- `--lib parent`: PASS, 10 selected, 0.06 s; includes both new `ParentMap`
  regressions and parent dispatch lookup/local removal.
- `integration::canonical_image`: PASS, 4 selected, 123.33 s. Includes actual
  source parity/provider isolation and two user-only replacement updates proving
  zero local entries in the migrated canonical tables and shared surface identity.
- Computed Record-key callable-signature regression: PASS, one selected, 41.15 s.
- `cargo check -p phalcom-core --features perf-observability`: PASS, 16.30 s.
- `cargo check -p phalcom-lsp`: PASS, 11.48 s; existing unused import warning.
- `git diff --check`: PASS. Repository formatter applied only to changed regions;
  no whole-workspace formatting or release certification claimed.

Source delivery: `e71057d9` on `perf/universe-characterization`. Commit and push
were authorized on 2026-10-13; lifecycle records accompany this source revision.
The continuation is focused-tested; residual families and metadata/artifact
acceptance gates remain open.

### P5 residual lookup and query-classification increment (2026-10-13, `eeca76f3`)

- `TypeAliasTable`, `StaticDefinitionTable` and `StaticConstantTable` attach
  immutable canonical parents; local removals and duplicate-ID rejection do
  not mutate/shadow canonical entries. First local update creates empty
  overlays; later updates clone local contributions only. The retained alias
  resolver now reads the committed snapshot's alias table instead of a second
  map of all canonical forms. Its static-definition lookup retains the parent
  table rather than flattening it into a `BTreeMap`.
- Accepted callable definitions now use `ParentMap` in the published snapshot;
  local revisions share canonical entries by reference. This does not yet
  cover remaining static-case catalogs/signatures, associated static domains,
  module binding maps and other copied aggregate families.
- `canonical_query_persistence(QueryKind)` is the single exhaustive family
  classification consumed by in-memory freeze and available to future artifact
  projection. It preserves the exact prior eligible/open-world/unsupported
  sets. Row readiness, final settlement and dependency closure remain separate
  fail-closed checks; no query is made Ready by family classification alone.

Focused evidence for the increment: canonical sharing/replacement integration
passed (one selected; first observed the alias, static constant and callable
definition copies before their corrections); 13 incremental recomputation tests,
seven alias capability tests and 18 workspace integration tests passed after
alias/static migration. Canonical-image library filter passed 13 selected,
including query classification and existing freeze contracts. After callable
definition migration, the exact image/replacement regression and an affected
incremental callable/body edit test passed. `cargo check -p phalcom-core
--features perf-observability` passed after the final code change. Existing
unused `TypeHierarchy` test import warning remains unrelated. Scoped diff
whitespace check passed. Full workspace/release, formatting, benchmarks,
P4 metadata integration and `.telosc` verification remain deferred; P5 is
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

### P5 parent-overlay continuation and P4 prerequisite gate (2026-10-13, `ca3c3c03` / `9143262b`)

- Image-backed local updates now share canonical module binding facts,
  constructor paths and trait headers/surfaces without flattening their maps.
  `ParentMap` exposes parent-only iteration and constant-time parent counts.
  Canonical static-case catalogs/signatures have zero entries in the live image;
  no general parent contract is claimed for them.
- Focused evidence: canonical-image integration PASS (four selected, 137.44 s);
  workspace integration PASS (18, 43.23 s); exact module-value dependency
  PASS (one, 47.24 s); parent-map library PASS (two); core with observability
  check PASS. Parent-count checks passed in the three-revision image
  replacement regression. PR-A Record/Pack native anchor test PASS (one).
- Exact P4 baseline: `canonical_universe_program_is_process_singleton` FAIL
  (one selected, 48.06 s) at public `FixedFields` metadata export because v12
  cannot encode its non-Symbol Record remainder. No schema changes were retained;
  PR-B, P4.T6 and P5.T10 remain incomplete. The semantic test harness retains
  an unrelated unused-import warning.
- Conformance index, witness plans and derived-rule buckets still clone
  canonical entries at first local update. The user approved a layered
  canonical/local view, not a P5.G6 copy exception; proof/coherence parity
  must be verified before claiming zero canonical aggregate copies.

### P5 layered conformance progress (2026-10-13, `7c9bd6d7`)

- `ConformanceIndex` now reads immutable canonical contributions through a
  parent; local secondary indexes remain disjoint. Exact queries merge only
  indexed parent/local candidates, while coherence considers canonical-local
  cross pairs without copying parent buckets into the workspace. Iteration
  remains stable-ID ordered and canonical entries retain reference identity.
- `DerivedConformanceRuleIndex` retains canonical trait buckets and an impl
  index through a parent. A borrowed two-slice trait view preserves indexed
  access, cardinality and iteration without flattening. General rule iteration
  remains stable-ID ordered for downstream fingerprints.
- Focused evidence: two conformance partition/brute-force library tests PASS,
  including a new canonical/local overlap and removal regression; derived
  parent library regression PASS; derived incremental/cold parity and condition
  edit regressions PASS (one each); canonical-image replacement regression
  PASS (one); core with observability check PASS. Canonical witness-plan map
  is empty in the present image, so its existing cloning copied zero parent
  entries in this run, but its general parent contract remains pending.
- P5.T6/T7/T8/T11/T12 and P4.PR-B/T6 are still open. No P5.G6 or P5.G1
  certification follows from these focused checks.

### Concurrent P5 persistence-inventory continuation (`791a0c0d`–`c600b12a`)

- Remote commits introduced a 22-family `CanonicalPersistenceClass` /
  `CanonicalProductFamily` inventory and `CanonicalImageCapabilities` in
  `canonical_image.rs`. Source-semantic attachments remain conservatively
  `RequiredSemantic` without an artifact-only reconstruction contract.
  T8 still needs complete compatibility/provenance and P4 metadata inputs.
- Remote conformance and derived-rule changes overlapped the locally tested
  `7c9bd6d7` work. The merge retains the validated cross-boundary coherence,
  stable-ID iteration and borrowed trait-rule view, alongside the new inventory
  and useful remote test/cardinality additions. Remote edits were GitHub-only
  and unverified when published; integrated-tree verification is separate.
- Witness-plan map, T7/T8 closure, P4-dependent T10 and T11–T13 remain open.

Integrated-tree verification (after resolving the concurrent conformance
implementations): `cargo check -p phalcom-semantic` PASS; conformance partition
library filter PASS (two), canonical-image library filter PASS (14, including
the inventory classification test), derived-rule parent library regression
PASS (one), canonical-image integration PASS (four, 125.66 s), derived-rule
incremental/cold parity PASS (one), and core observability check PASS. Remote
source had no test evidence when published; these results apply to the
reconciled tree only. The semantic integration test harness still reports the
unrelated unused `TypeHierarchy` import warning.

### P4.PR-B Record metadata increment (2026-10-13, uncommitted over `55452273`)

- Schema v13 retains tagged `Symbol/String/Int/Bool` keys and explicit global,
  scoped and static-row Record remainder domains. Semantic export fingerprints
  include key kind/value and remainder key/value type identities; publication
  retains a public `FixedFields`-shaped alias with its `FieldKey` domain.
- Pre-v13 Symbol field names decode without context. Anonymous legacy Record
  remainders require a loader-supplied canonical Symbol declaration; absent or
  invalid context is typed. Contextual decode inserts canonical global/scoped
  nodes, relocates references, validates, and marks old digests with an explicit
  fingerprint-schema version. The normalized graph round-trips as v13 without
  a second relocation. Mixed-version fingerprint equality fails closed.
- Runtime metadata conversion, identity, display and row static-argument
  projection retain heterogeneous keys and both remainder domains. Canonical
  RuntimePublic installation/reflection acceptance is **not** established.
- Evidence: `phalcom-type-meta --test schema_compat` PASS (23 selected);
  semantic exact export/alias/fingerprint/scoped-row tests PASS (one each);
  `phalcom-core --lib typing::identity::tests` PASS (8 selected);
  runtime row substitution PASS (one); core observability `cargo check` PASS;
  `cargo fmt --package phalcom-type-meta -- --check` PASS; scoped diff check
  PASS. Workspace format check is incomplete on pre-existing canonical-image
  formatting changes outside PR-B.
- Owning semantic metadata integration filter: 36 PASS / 1 FAIL (37 selected);
  the sole failure is the local-workspace canonical-source fixture above,
  before metadata export. Do not count this filter as passing.
- Trait evidence slot incident resolved: `publish_conformance_witness_plans` in
  `session.rs` was updated to check `if db.canonical_image().is_some() && contribution.source_module.project.is_universe()`,
  ensuring Universe witness plans are published and stored in the canonical image snapshot.
  `canonical_universe_program_is_process_singleton` PASS (one selected, 43.19 s);
  all 6 tests in `modules::canonical_universe::tests` PASS;
  all 10 unit tests in `canonical_image::tests` PASS;
  all 4 canonical formal image integration tests PASS (130.56 s);
  `applied_record_row_metadata_display_preserves_field_types_and_tail` PASS;
  `modules::compile::tests` PASS (3 selected, including RangeIterator closed bounds);
  `metadata::reachability::tests` PASS (4 selected, including nested nominal discovery).
  Canonical compiler projection is now unblocked and certified. P4.T6 canonical
  authority installation and P5 witness-plan parent-sharing can proceed.


## P7 continuation — 2026-10-14 source-free module boundary

- Active plan: PERF003.C2.P7; completion PARTIAL; verification UNVERIFIED.
- Starting revision: 3ca30fca. Existing T1/T2 edits and documentation relocation
  were inherited and preserved; this continuation is uncommitted.
- CanonicalExternalModuleWorld now separates completed linked/interfaces/topology
  authority from CanonicalModuleProducts source-build provenance.
- Module sessions, resolvers, and linkers consume external authority without a
  UniverseSourceProvider. Semantic image sessions use completed canonical linking
  authority; optional source-backed presentation remains a separate capability.
- Artifact integrity uses BLAKE3-256; stable identity schema compatibility is
  checked. The reader rejects unsupported flags, compression, known section
  schemas, and executable capability without its payload.
- Container/architecture specification: docs/specs/compiler-artifacts/telosc.md.
- User instruction: targeted compiler-driven checks only; no tests or broader
  verification for this continuation. No behavioral acceptance gate is promoted.
- Next action: P7.T4 MODULE_WORLD DTO/projection/load, then T5–T15 and P8.
  No universe.telosc writer/loader, executable payload, or CLI artifact path
  is implemented by this continuation.

Final compiler-only evidence: cargo check -p phalcom-core -p phalcom-lsp --lib
passed in 10.41 seconds with flags cleared. LSP's unchanged backend.rs reports
an unused TypeHierarchy import. Behavioral verification remains UNVERIFIED.


## P7 continuation — module-world transport and type-graph relocation

- Active plan PERF003.C2.P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED.
- T4 transport/projection/materialization code implemented: ModuleWorldV1,
  artifact-local module IDs, bounded binary codec, module-owner conversion,
  fingerprint/namespace/topology/DAG validation, and source-free external-world
  constructors. No ModuleLinker, parser, or interface builder is called by import.
- T5 projection/relocation code implemented: exhaustive type/kind/scoped node
  schemas, distinct arena ID families, stable owner codecs, fresh TypeStore,
  normal interner reconstruction, checked graph dependencies, and explicit
  TypeRelocation vectors for later payload consumers.
- Durable distinction: TypeStore contains historical parameter versions and a
  separate current `(owner,index)` selection. Persistence retains both rather
  than collapsing versions. Parameter variance overrides are also primary state.
- Scoped union recipes preserve order/repetition. Global unions normalize native
  member order after relocation; scoped lowering does not perform that normalization.
- Completed compiler-only check: flags cleared, cargo check -p phalcom-core
  -p phalcom-lsp --lib passed in 8.31 seconds. Unchanged LSP backend reports its
  existing unused TypeHierarchy import. No tests or behavioral verification run.
- T4/T5 parity, malformed-input behavior, cross-process determinism, and
  source-free acceptance remain unproved. No acceptance gate promoted.
- Next action: P7.T6 static-value graph and relocation, then T7–T15 and P8.
  Data/variant value interning requires canonical DataInfo/VariantInfo; preserve
  that validation in the loader rather than constructing unchecked value nodes.
- No complete universe.telosc writer/loader, CLI artifact commands, or executable
  persistence exists yet. Changes remain uncommitted.

Final compiler-only check after codec error classification: flags cleared,
cargo check -p phalcom-core -p phalcom-lsp --lib passed in 5.30 seconds.
Only the unchanged LSP unused TypeHierarchy import warning remains.


## Review corrections — 2026-10-14

The three requested review fixes are implemented in the existing worktree:

- Tuple, Record, and Pack static constants again project self-contained recursive
  descriptors and use the canonical owner-held runtime materialization path,
  including when nested inside immutable sequences.
- Both demand-driven and ordinary-session constant publication retain the
  evaluator-owned residual store for symbolic outcomes. Cycle products carry no
  residual store. Published products retain ownership through their existing Arc.
- TypeStore artifact relocation rejects RecordRow parameters in TypeData::Parameter
  and rejects a node kind differing from its relocated parameter definition.

Compiler-only validation completed with local flags and wrapper cleared:
`cargo check -p phalcom-core -p phalcom-lsp --lib`,
`cargo check -p phalcom-semantic --test semantic -p phalcom-core --test core --lib`,
and `cargo check -p phalcom-core --tests` all passed. Existing unused TypeHierarchy
imports were reported in the LSP backend and semantic workspace integration tests.
Regression coverage was added/updated for structural owner materialization,
nested descriptor contents/labels, residual ownership after evaluator teardown,
and valid/malformed parameter artifact relocation. Tests were compiled but not
executed, honoring the existing compiler-only instruction. No behavioral gate or
checkpoint/plan status was promoted; P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED.
No architectural consultation or plan amendment was needed. Continue with the
existing P7 next action. No commit or push was requested.


## P7/P8 execution planning — 2026-10-14

- Planning artifacts: [shared research/context](Program-Architecture-and-Planning/PERF003.C2.P7-P8-execution-research.md), [P7 execution companion](P7/PERF003.C2.P7-remaining-execution-plan.md), [P8 detailed plan](P8/PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md), and [inline plan review](Program-Architecture-and-Planning/PERF003.C2.P7-P8-plan-review.md).
- P7 retains its original task numbers and G7.0–G7.11 gates. T4/T5 code is implemented but unaccepted. Next implementation adds shared guarded deterministic projection and an exhaustive persistence manifest; type-only DataInfo/VariantInfo contracts precede static ADT import.
- Required compiler attachments include checked static operations: the live checked-body root still contains syntax. Required products cannot be dropped; an affected evaluator contract requiring redesign must use the consultation protocol before implementation.
- P8 adds isolated-VM prototype capture, complete non-eager module coverage, executable validation, rooted fresh allocation, source-free native setup, exact bootstrap order, later-import prototypes and explicit CLI/API wiring. Native-floor module setup currently reads UniverseSourceProvider and is explicitly in scope.
- GSD research/planning/review is adapted to repository-native records. No competing `.planning` state, automatic execution, independent checker claim or source changes were made. Inline self-review is planning evidence only.
- Compiler-only feedback remains the user-authorized policy. BUILD can continue from completed P7 APIs into P8; P7/P8 COMPLETE and source-free artifact claims require deferred behavioral proof. P9 discovery/cache/distribution remains outside the execution scope.
