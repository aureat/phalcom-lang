# PERF003.C2.P4 walkthrough — Profile/Root-Driven Metadata Reachability Architecture (Package D Foundation)

Status: IN_PROGRESS / PARTIAL / FOCUSED_TESTED at active increment scope.
Branch: `perf/universe-characterization` (worktree: `.worktrees/perf-universe-characterization`).

---

## 1. Executive Summary

This increment implements the core semantic foundations of **PERF003.C2.P4** ("Profile/Root-Driven Metadata Reachability Architecture"):
1. **Metadata Reachability Engine (`P4.T2`, `P4.T3`)**: Implemented `MetadataRootSet`, `MetadataRetentionRequest`, `MetadataReachabilityClosure`, `ReachabilityMetrics`, and `compute_reachability_closure(...)` in `phalcom-semantic/src/metadata/reachability.rs`.
2. **Declaration-Owner Indexing (`P4.T4`)**: Added secondary index maps to `CallableSignatureTable` and `FieldSignatureTable` to eliminate $O(\text{declarations} \times \text{all callables})$ whole-table owner scans.
3. **Canonical Authority Elision (`P4.T3`, `P4.T6`)**: Enforced the invariant that canonical Universe entities (`module_id.project.is_universe()`) are never duplicated into ordinary user bundles; their nominal references remain cross-pool pointers to the canonical pool.
4. **Observability Metrics (`P4.T10`)**: Added 14 new `PerfCounter` variants in `telos-observability/src/metric.rs` tracking metadata roots, traversed dependencies, retained entities, elisions, and lookups.

---

## 2. Declaration-Owner Indexing (`phalcom-semantic/src/signature.rs`)

Previously, associating callables and fields with their enclosing declarations required scanning every entry in `CallableSignatureTable` and `FieldSignatureTable` for each declaration.

- **`CallableSignatureTable`**:
  - Added `owner_callables: HashMap<CallableOwnerId, BTreeSet<CallableId>>`.
  - Added accessor `iter_owner(&'a self, owner: &CallableOwnerId) -> impl Iterator<Item = &'a CallableId>`.
  - Added accessor `iter_declaration_owner(&'a self, owner: &DeclarationId) -> impl Iterator<Item = &'a CallableId>`.
  - Updated `insert`, `remove`, and `remove_module` to maintain the owner index incrementally.
  - Clone and mutation semantics verified via unit test `callable_owner_index_tracks_membership_removal_and_cloning`.

- **`FieldSignatureTable`**:
  - Added `owner_fields: HashMap<DeclarationId, BTreeSet<FieldId>>`.
  - Added accessor `iter_owner(&'a self, owner: &DeclarationId) -> impl Iterator<Item = &'a FieldId>`.
  - Updated `insert`, `remove`, and `remove_module` to maintain the owner index incrementally.
  - Clone and mutation semantics verified via unit test `field_owner_index_tracks_membership_removal_and_cloning`.

---

## 3. Metadata Reachability Engine (`phalcom-semantic/src/metadata/reachability.rs`)

Implemented the reachability closure traversal that replaces aggregate semantic table dumps with focused, profile-bounded slices:

- **`MetadataRootSet`**:
  - Tracks entry module IDs, runtime root type IDs, root declaration IDs, root callable IDs, and root field IDs.
  - Provides builder methods `insert_module`, `insert_runtime_type`, `insert_declaration`, `insert_callable`, and `insert_field`.

- **`MetadataRetentionRequest`**:
  - Bundles a `MetadataProfile` with a `MetadataRootSet`.

- **`compute_reachability_closure`**:
  - Traverses from seed roots through nominal type hierarchies, callable parameter/return types, field types, and type alias expansions.
  - Under `RuntimeMinimal`, only retains declarations that appear in runtime type roots, entry modules, or dependencies of referenced callables/fields.
  - Under `RuntimePublic`, retains all public declarations and members in root modules, plus their transitive dependencies.
  - Traverses nominal forms, class object types, supertype templates, generic constraints, and parameter shapes via `TypeStore`.
  - Elides canonical Universe declarations (`is_universe()`), ensuring they are not redundantly emitted into user program bundles.
  - Records traversal metrics in `ReachabilityMetrics`.

---

## 4. Observability Metrics (`telos-observability/src/metric.rs`)

Added 14 `PerfCounter` variants:
- `MetadataRootsSelected`
- `MetadataEntitiesVisited`
- `MetadataDependencyEdgesTraversed`
- `MetadataDeclarationsRetained`
- `MetadataDeclarationsRejectedByProfile`
- `MetadataAliasesRetained`
- `MetadataCallablesRetained`
- `MetadataFieldsRetained`
- `MetadataCanonicalDeclarationsElided`
- `MetadataCanonicalCallablesElided`
- `MetadataCanonicalFieldsElided`
- `MetadataExternalLookups`
- `MetadataExternalLookupFailures`
- `MetadataIdentityConflicts`

---

## 5. Verification Evidence

1. `cargo check -p telos-observability`: PASS (clean).
2. `cargo check -p phalcom-semantic`: PASS (clean).
3. `cargo test -p phalcom-semantic --lib signature::tests`:
   - `signature::tests::callable_owner_index_tracks_membership_removal_and_cloning`: PASS
   - `signature::tests::field_owner_index_tracks_membership_removal_and_cloning`: PASS
   - Total 6 passed, 0 failed.
4. `cargo test -p phalcom-semantic --lib metadata::reachability::tests`:
   - `metadata::reachability::tests::metadata_root_set_tracks_cardinalities`: PASS
   - `metadata::reachability::tests::reachability_closure_minimal_vs_public_retention`: PASS
   - `metadata::reachability::tests::reachability_closure_elides_canonical_authorities`: PASS
   - Total 3 passed, 0 failed.

---

## 6. Remaining Package D Tasks for Next Increment

1. **`P4.T1`**: Thread `ProgramCompilationOptions { metadata_profile }` through `phalcom-core/src/modules/compile.rs` and session fixtures.
2. **`P4.T5`**: Cut `MetadataExporter::build_bundle` over to `compute_reachability_closure` and closure-scoped entity iteration.
3. **`P4.T6`**: Enforce canonical metadata partitioning in compiler lowering (only canonical bootstrap exports Universe entities).
4. **`P4.T7` & `P4.T8`**: Add stable identity indexes (`StableDeclarationRef`, `StableCallableRef`, `StableFieldRef`) to `RuntimeTypingRegistry` and update `phalcom-core/src/typing/inspect.rs` to eliminate same-pool reflection assumptions.
5. **`P4.T9`**: Record canonical interface fingerprint compatibility descriptor in `CompiledProgram` and validate upon materialization.
6. **`P4.T11`**: Run focused compiler and reflection regression suites.
