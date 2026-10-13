# PERF003.C2.P3 walkthrough — conformance indexing and settlement increments

Status: IN_PROGRESS / PARTIAL / FOCUSED_TESTED at active increment scope.
Branch: `perf/universe-characterization` (worktree: `.worktrees/perf-universe-characterization`).
Committed code at `c34f9c3f`.

## 1. Conformance Index Partitioning & Parity

- `ConformanceIndex` partitions contributions by trait, closed nominal family, and open/unclassified heads:
  - `by_trait: BTreeMap<DeclarationId, BTreeSet<ImplId>>`
  - `by_trait_family: BTreeMap<DeclarationId, BTreeMap<DeclarationId, BTreeSet<ImplId>>>`
  - `open_by_trait: BTreeMap<DeclarationId, BTreeSet<ImplId>>`
  - `family_by_impl: BTreeMap<ImplId, DeclarationId>`
- `insert_with_store` extracts the nominal conformance family when target head matches the target declaration, keeping open/blanket/unresolved heads in the open partition.
- `candidates_for_query` narrows lookups to matching nominal family plus open heads for the requested trait, falling back to all trait entries for non-nominal targets.
- `overlap_conflicts` checks pairs across identical families and open heads instead of an all-pairs N^2 scan.
- Parity verified: `impls::conformance_index_partition_tests::partitions_queries_and_coherence_match_brute_force_through_mutations` passed with exact match against brute-force query and overlap checks through mutations and removals.

## 2. Inferred-Result Settlement & Dependency Edges

- Global inferred-result rescans are replaced by deterministic affected-result/SCC worklist propagation in `phalcom-semantic/src/session.rs`.
- Downstream callable-result consumers are scheduled only when published result contracts actually change.
- Recomputed callable bodies re-record their observed dependencies via `callable_analysis_dependencies` rather than inheriting stale edge sets.
- Regression passed: `semantic::incremental::callable_dependencies::inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once` (1 selected, 43.24 s).

## 3. Module-Value Dependency Precision

- Imported and local checked bindings are seeded lazily with lexical provenance in `CheckingContext`.
- Qualified module reads record `LinkedName`, `PublicExport`, `ModuleStaticBindingCatalog`, and positive `ModuleStaticBindings` edges only when the binding is looked up, not on blanket import presence.
- Removed blanket `has_module_value_environment` boolean from `CallableBodyFormalInputFingerprint`.
- Absent `PublicExport` edges invalidate when an export is subsequently added in a new revision.
- Regressions passed:
  - `missing_qualified_module_value_export_invalidates_when_export_is_added`
  - `qualified_module_value_dependencies_follow_reads_not_import_seeding_or_shadowed_names`

## 4. Witness Plan Construction Inspection & First Separation Step

- `build_conformance_witness_plan` consumes pre-instantiated normalized `requirement_views` from candidate discovery, eliminating duplicate instantiation.
- `publish_conformance_witness_plans` skips Universe conformances by source-module ownership (`source_module.project.is_universe()`), ensuring local augmentations remain eligible.
- Inspection of persistent retained structural plans across revisions:
  - Canonical `ConformanceWitnessPlan` authority is preserved; no competing cache is introduced.
  - A durable retained structural plan requires separate structural vs result-sensitive fingerprints, along with explicit positive/negative eligibility query edges. This remains planned successor work.

## 5. Canonical Image & One-Shot Seam

- `SemanticWorkspaceInput::from_interfaces` retains authoritative module pipeline interfaces without re-derivation.
- `ProgramCompiler` uses `from_interfaces` for entry compilation.
- `ModuleResolver::load_interface` does not load parsed source for canonical Universe modules.
- Regressions passed:
  - `from_interfaces_retains_authoritative_products_without_rebuilding`
  - `canonical_interface_load_does_not_load_parsed_source_per_resolver`

## 6. Verification Status

- `RUSTFLAGS='' cargo check -p phalcom-semantic`: PASS
- `RUSTFLAGS='' cargo check -p phalcom-core --features perf-observability`: PASS
- `RUSTFLAGS='' cargo check -p phalcom-modules`: PASS
- `git diff --check`: PASS (clean)
- Broader parity, rollback, runtime/compiler, and performance benchmark suites remain deliberately deferred in BUILD MODE.
