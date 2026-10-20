---
id: PERF003.C2.P6
program: PERF003
checkpoint: PERF003.C2
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# PERF003.C2.P6 — User-Only Backend / Local Complexity Closure Walkthrough

Final certified completion record on `perf/universe-characterization`, 2026-10-14.
This plan establishes module-indexed backend iteration, eliminates whole-world $O(N \times M)$
scans from compiler projection and lowering, and binds runtime lowering to local work units.

## 1. What Was Implemented

### P6.T0 / P6.T1 — Baseline & Observability
- Grounded execution on verified commit `69c618c8`.
- Added 15 new cardinality counters in `telos-observability::PerfCounter`:
  - `CompilerBackendCallableAnalysesVisited`, `CompilerBackendCallableDefinitionsVisited`, `CompilerBackendDeclarationsVisited`, `CompilerBackendDataDeclarationsVisited`, `CompilerBackendEnumDeclarationsVisited`, `CompilerBackendCanonicalEntitiesVisited`, `CompilerBackendEntitiesRejectedByModule`
  - `CompilerBindingOccurrenceFullScans`, `CompilerBindingOccurrencesVisited`, `CompilerBindingOccurrencesResolvedByTargetIndex`
  - `CompilerTraitRequirementLayoutsBuilt`, `CompilerTraitRequirementSlotsLookedUp`, `CompilerTraitSurfacesVisitedForSlotLookup`
  - `CompilerInherentImplDomainLookups`, `CompilerConditionalMembersScannedForImplDomain`

### P6.T2 / P6.T3 — Module-Indexed Callable Analyses & Definitions
- Introduced `CallableAnalysisTable` in `phalcom-semantic::checker::analysis`.
  - Maintains `by_id: HashMap<CallableId, Arc<CallableAnalysis>>` and `by_module: HashMap<ModuleId, BTreeSet<CallableId>>`.
  - Exposes $O(\text{local})$ `iter_module(&self, module: &ModuleId)`.
  - Implements immutable `Deref<Target = HashMap<...>>` for full backwards compatibility with test harnesses and existing readers.
  - Published into `SemanticSnapshot::callable_analyses` via `From<HashMap<...>>`.
- Added `callable_analyses_in_module` and `callable_definitions_in_module` to `SemanticSnapshot`.

### P6.T4 — Module-Indexed Declarations, Data Semantics, and Enum Semantics
- Extended `DeclarationTypeTable` in `phalcom-semantic::declarations` with `module_declarations` secondary index.
  - Added `iter_module(&self, module: &ModuleId)` and `intrinsic_runtime_declarations_in_module(&self, module: &ModuleId)`.
- Extended `DataSemanticTable` in `phalcom-semantic::data_semantics` with `module_data` index and `iter_module`.
- Extended `EnumSemanticTable` in `phalcom-semantic::enum_semantics` with `module_enums` index and `iter_module`.

### P6.T5 — Reverse-Target Binding Occurrence Resolution
- Replaced the nested $O(\text{callables} \times \text{occurrences})$ loop in `phalcom-core::modules::semantic_lowering` with direct reverse lookup:
  - Queries `snapshot.source_index().references().target_set(&SemanticTargetId::Binding(decl_site))` in $O(1)$.
  - Resolves occurrence ranges directly without scanning `module_index.occurrences.all()`.
  - Records `CompilerBindingOccurrencesResolvedByTargetIndex`.
  - Retains a single-pass module-wide fallback if target references are absent.

### P6.T6 — ModuleLoweringInputView Facade
- Introduced `ModuleLoweringInputView<'a>` in `phalcom-core::modules::semantic_lowering`.
- Encapsulates module-bounded access to:
  - `callable_analyses()`
  - `callable_definitions()`
  - `data_declarations()`
  - `enum_declarations()`
  - `intrinsic_runtime_declarations()`
- Eliminated all `.iter().filter(|...| module == ...)` scans in module lowering.
- In `compile.rs`, replaced outer scans over `analyzed.linked.modules` with direct iteration over `&analyzed.runtime_modules`.

### P6.T7 — Precomputed Trait Requirement Layouts
- Introduced `TraitRequirementLayout` in `phalcom-semantic::traits`.
  - Precomputes canonical requirement slots in BTree key order upon `surface.finalize_requirement_layout()`.
  - Exposes `requirement_slot(&self, requirement: &TraitRequirementId) -> Option<u32>`.
- Replaced whole-snapshot `trait_default_requirement_slots` scans in `project_trait_invocation_spec` and `project_for_iteration_requirement` with trait-local lookup on the owning `TraitSurface`.

### P6.T8 — Direct Inherent Impl Domain Lookup
- Added `inherent_impl_domains: Arc<ParentMap<ImplId, Arc<InherentImplDomain>>>` to `SurfaceDispatchResolver`.
- Automatically indexed during `register_conditional_members`.
- Exposes $O(1)$ `inherent_impl_domain(&self, impl_id: &ImplId)`.
- Replaced the linear scan `conditional_members().values().flat_map(...).find(...)` in `compile.rs` with `snapshot.dispatch.inherent_impl_domain(source_impl)`.

## 2. Verification Evidence Matrix

| Gate | Target / Test Filter | Result | Time |
|---|---|---|---|
| Module Lowering Local Products | `cargo test -p phalcom-core --lib modules::compile::tests::lowering_input_view_iterates_only_local_module_products` | **PASS** (1 selected) | 40.64 s |
| Core Compile Suite | `cargo test -p phalcom-core --lib modules::compile::tests` | **PASS** (4 selected) | 50.59 s |
| Canonical Universe Program | `cargo test -p phalcom-core --lib modules::canonical_universe::tests` | **PASS** (7 selected) | 39.56 s |
| Runtime Reflection Metadata | `cargo test -p phalcom-core --test core applied_record_row_metadata_display_preserves_field_types_and_tail` | **PASS** (1 selected) | 0.01 s |
| Core Typing Suite | `cargo test -p phalcom-core --lib typing::` | **PASS** (13 selected) | 0.01 s |
| Canonical Image Library | `cargo test -p phalcom-semantic --lib canonical_image` | **PASS** (15 selected) | 39.06 s |
| Parent Map Tests | `cargo test -p phalcom-semantic --lib parent_map` | **PASS** (2 selected) | 0.00 s |
| Image Zero-Copy Guard | `cargo test -p phalcom-semantic --test semantic from_image_zero_canonical_entries_copied_cardinality_guard --features perf-observability` | **PASS** (1 selected) | 40.76 s |
| Schema Compatibility | `cargo test -p phalcom-type-meta --test schema_compat` | **PASS** (23 selected) | 0.01 s |
| Core Check (Default) | `cargo check -p phalcom-core` | **PASS** (clean) | 3.36 s |
| Core Check (Observability) | `cargo check -p phalcom-core --features perf-observability` | **PASS** (clean) | 9.85 s |
| Semantic Check (Observability) | `cargo check -p phalcom-semantic --features perf-observability` | **PASS** (clean) | 0.09 s |
| Observability Crate Check | `cargo check -p telos-observability` | **PASS** (clean) | 1.69 s |
| Diff Hygiene & Formatting | `git diff --check && rustfmt ...` | **PASS** (clean) | 0.00 s |
