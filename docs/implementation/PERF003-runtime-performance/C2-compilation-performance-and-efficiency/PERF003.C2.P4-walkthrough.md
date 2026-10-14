# PERF003.C2.P4 walkthrough — Profile/Root-Driven Metadata Reachability Architecture (Package D Foundation)

Status: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
Branch: `perf/universe-characterization` (worktree: `.worktrees/perf-universe-characterization`).

Latest execution boundary: explicitly stopped by the user on 2026-10-13.
T1 and the prior incident/handoff were committed and pushed in `e74a9213`.
The subsequent T5 / PR-A slice is uncommitted. Section 8 below supersedes prior
next-action and failing-fixture statements where its newer evidence applies.

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

1. **`P4.T1`**: Plumbing implemented in the 2026-10-13 continuation; retained-session end-to-end gate remains blocked.
2. **`P4.T5`**: Cut `MetadataExporter::build_bundle` over to `compute_reachability_closure` and closure-scoped entity iteration.
3. **`P4.T6`**: Enforce canonical metadata partitioning in compiler lowering (only canonical bootstrap exports Universe entities).
4. **`P4.T7` & `P4.T8`**: Add stable identity indexes (`StableDeclarationRef`, `StableCallableRef`, `StableFieldRef`) to `RuntimeTypingRegistry` and update `phalcom-core/src/typing/inspect.rs` to eliminate same-pool reflection assumptions.
5. **`P4.T9`**: Record canonical interface fingerprint compatibility descriptor in `CompiledProgram` and validate upon materialization.
6. **`P4.T11`**: Run focused compiler and reflection regression suites.

## 7. Inline continuation — 2026-10-13

Continued from clean `a82dfe32` in the named worktree, without subagents.
Changes remain uncommitted; no commit/push was requested for this continuation.

- `compile.rs`: adds `ProgramCompilationOptions`, default RuntimePublic,
  `compile_analyzed_with_options`, and explicit threading through projection to
  the exporter. Canonical bootstrap retains its existing default.
- `compile_session.rs`: adds `compile_fixture_with_options`; the original API
  delegates with default options. Semantic publication remains runtime-minimal.
- `modules/mod.rs`: exports the options type.
- An isolated empty analyzed-program projection asserts the default and all four
  explicit metadata headers. This proves the options seam only, not retention,
  execution, or cross-pool reflection.
- A retained-session regression asserts default public -> explicit minimal ->
  default public headers and no canonical presentation shards. It currently
  fails on the inherited whole-world export problem before its first assertion.
  It is neither ignored nor weakened.

Completed evidence:

| Check | Result |
|---|---|
| New regression before implementation | Compilation rejected missing options/API; one fixture assertion was mechanically corrected to use the published source-index API |
| `RUSTFLAGS='' cargo check -p phalcom-core --features perf-observability` | PASS, 18.99 s |
| `RUSTFLAGS='' cargo test -p phalcom-core --lib empty_projection_preserves_explicit_metadata_profile` | PASS, 1 selected, 0.01 s execution |
| `RUSTFLAGS='' cargo test -p phalcom-core --lib metadata_profile_is_explicit_and_independent_of_semantic_publication` | FAIL, 1 selected, 41.58 s; legacy non-Symbol Record remainder export |
| Exact `modules::canonical_universe::tests::canonical_universe_program_is_process_singleton` | FAIL, 1 selected, 0.05 s; ProductKind source-index rejection before projection |
| Scoped rustfmt check | Own formatting corrected; predecessor `compile_session.rs` line wrapping remains outside this patch |
| `git diff --check` | PASS |

Both runtime/export messages were already recorded in CHECKPOINT section 5.
The ProductKind rejection is before the modified projection/options code. It
now invalidates the usable canonical compiled authority premise required by T6.
No bootstrap correction, schema change, profile weakening, or plan amendment
was attempted. Mandatory trigger 2 is recorded in the
[incident](PERF003.C2.P4-canonical-authority-incident.md).

T1 source plumbing is IMPLEMENTED with focused seam evidence; its end-to-end
verification is BASELINE_BLOCKED. P4 overall remains PARTIAL / BASELINE_BLOCKED.
T5–T9, compiler/reflection suites, profile retention matrix, cardinality gates,
and performance measurements remain unperformed. No performance improvement or
P5 readiness is claimed.

Additional live-source prerequisite before exporter cutover: the inherited
closure includes all owned members for non-minimal profiles, without published
visibility input. `SemanticSnapshot::surfaces` already owns member visibility;
the eventual cutover must consume it. This observation is not permission to
invent visibility policy or treat foundation unit tests as complete retention
certification.

## 8. Accepted-ruling continuation, stopped by user

The ruling is saved as `PERF003.C2.P4-architectural-ruling.md`, referenced from
the active plan and checkpoint. It adds PR-A/PR-B prerequisites to T6 and
authorizes continuing T5. No subagents were used; T1 source/defaults and its
retained-session assertions were left unchanged.

### T5 working-tree implementation

- `metadata/export.rs`: explicit `build_bundle_from_closure`; declaration,
  alias, callable, field and project-identity loops bounded by retained IDs;
  owner indexes for declaration callable/field lists; field records/lists wired.
- Legacy detached fixture calls renamed to `build_complete_test_bundle`.
  The complete-table/catalog path is explicit and is not used by compiler
  production callers. This is an API migration, not a claim that all renamed
  test cases have executed.
- Production static-domain catalog selection starts from exported domain IDs
  and retained declaration products, following constructor-kind dependencies.
- New `MetadataEntityIdentity` and `MetadataExportError::Projection` preserve
  profile, retained top-level entity, failing TypeId and typed underlying cause.
  Ordinary compilation now uses `ProgramCompileError::Metadata` rather than
  stringifying exporter failures as I/O errors.
- Known declared type facts fail closed on export errors instead of becoming
  unavailable silently. Inferred-return fallback behavior still needs review.
- `reachability.rs`: optional published member-surface input filters callable
  and field retention to Public when supplied. Compiler production supplies it.
- `compile.rs`: roots from executable type/call facts, published local Class/Data
  declaration IDs and linked public exports; public aliases retained; field
  signatures supplied; metrics recorded. Canonical bootstrap uses a predicate
  that retains its own canonical records; ordinary compilation excludes them.
- `semantic_lowering.rs`: data-construction metadata uses an explicit closure.

This is an incomplete production cutover. In particular, ordinary canonical
elision is present in the patch, while T6 runtime loading/indexing is not; no
runtime or cross-pool certification is implied. The inherited closure's type
traversal is incomplete compared with exporter recursion and must be reconciled
before T5 completion. Error context reset/root-to-recursive-path coverage and
retained fingerprint behavior also need final review.

### PR-A working-tree implementation

`native/source.rs::index_program` classifies intrinsic runtime data before
entering the native facade path. Canonical Tuple/Record/Pack keep their existing
native anchors; other canonical runtime-class identities and wrong-module
identities fail closed. Non-runtime-catalog compiler intrinsic data stays in
the parsed semantic program; sums use normal enum/census indexing and create
no native anchors. No UniverseKey/catalog changes or ProductKind allowlist.

### Completed evidence

| Gate | Result |
|---|---|
| New retained alias/provenance regression | PASS, 1 selected, 0.01 s |
| Retained-session profile independence regression after T5 | PASS, 1 selected, 43.93 s |
| Compiler-owned intrinsic data regression before PR-A | FAIL on ProductKind, as expected |
| `native::source::tests` after PR-A | PASS, 9 selected, 0.05 s; includes bundled source corpus |
| `metadata::reachability::tests` after nested fix | PASS, 4 selected, 0.01 s; validates Record/Callable/Pack/Projection discovery, elision, rollback |

The alias test checks a fixed String key, not a non-Symbol remainder. PR-A's
focused fixture asserts Tuple anchors; dedicated Record/Pack facade regression
coverage remains to be added. Source-index success does not prove canonical
native contracts, semantic projection or RuntimePublic metadata export.

### PR-B investigation at stop

No `phalcom-type-meta` source, schema version, codec, validation, runtime identity
or reflection representation was changed. Schema remains v12. Investigation
confirmed the producer is JSON serde-based (`encode.rs`), advertises schemas
1–12, and currently represents fixed fields by Symbol-only `name` and anonymous
remainders without key domains, globally and in scoped/row forms.

The contemplated v13 representation was the ruling's tagged `RecordKeyRef`
(Symbol/String/exact StaticInt/Bool) and explicit key/value type IDs on
remainders. Validation ordering/duplicates, child topology/kinds, exporter
fingerprints, runtime row conversion, display and identity normalization all
need to carry those facts. The exact migration implementation is undecided.

A possible legacy decode approach was being evaluated: transform legacy JSON
fields into Symbol keys and materialize canonical Symbol key-domain nodes,
remapping global/scoped node references while preserving their topological
order and semantic identity. This is investigation only. Prepending nodes
requires shifting all global and scoped references, including generics,
kind-domain environments, declarations, runtime roots and occurrences; simply
appending a Symbol node would create forward edges. No migration was selected
or coded, and no new schema tests were written.

### Stop and delivery

User requested immediate stop and lifecycle/handoff reporting. Source edits and
validation stopped; only lifecycle files updated afterward. P4 remains
IN_PROGRESS / PARTIAL / BASELINE_BLOCKED with narrower focused T5/PR-A evidence.
No new commit/push, benchmark, broad certification or independent review.
The next implementer must consume the current handoff, not rerun the obsolete
incident diagnosis or change the accepted T1 defaults.

---

## 9. Full Implementation and Certification

### PR-B Schema v13 and Contextual Normalization
- Bumped schema version to 13 in `phalcom-type-meta`.
- Introduced `RecordKeyRef` (`Symbol`, `String`, `Int`, `Bool`) supporting heterogeneous Record keys.
- Extended `AnonymousRecordRemainderRef` and `ScopedAnonymousRecordRemainderRef` with explicit `key_type` and `value_type`.
- Implemented contextual legacy decoding in `legacy_record.rs`: legacy bundles require loader-supplied canonical Symbol identity to insert canonical Symbol nodes, relocate all node IDs, and preserve legacy provenance (`legacy_fingerprint_schema_version = Some(12)`).
- Updated semantic export fingerprints in `export.rs` to incorporate field key bytes and remainder key/value types.
- Updated runtime typing models (`identity.rs`, `inspect.rs`, `environment.rs`, `overlay.rs`, `primitive/typing.rs`) to preserve heterogeneous keys and remainder domains.

### Canonical Authority and Lowering Closure (T6)
- Resolved the missing trait evidence slot in `session.rs` by restricting the Universe conformance skip to overlay sessions (`if db.canonical_image().is_some() && contribution.source_module.project.is_universe()`).
- Enabled complete canonical Universe formal image generation with all Universe witness plans published.
- Unblocked canonical Universe executable lowering and compiler singleton projection (`canonical_universe_program_is_process_singleton`).

### Cross-Pool Metadata Indexes and Runtime Resolution (T7, T8)
- Added `RuntimeDeclarationMetadataRef` to `handle.rs`.
- Added O(1) declaration, callable, and field metadata indexes to `RuntimeTypingRegistry` populated on `register_pool`.
- Replaced linear pool scans in `inspect::declaration_record_for_class`, `inspect::lookup_member`, and `inspect::base_constructor_arity` with registry index lookups.
- Verified cross-pool metadata indexing via focused unit test `register_pool_indexes_declarations_callables_and_fields`.

### Compatibility Verification (T9)
- Enforced canonical compatibility checks (`semantic_model_version`, `native_surface_schema_version`) in `install_semantic_metadata_bundle`.
- Verified compatibility rejection on mismatched versions via unit test `metadata_bundle_installation_validates_canonical_compatibility`.

### Observability Metrics (T10)
- Verified all 14 `PerfCounter` variants in `telos-observability`.
- Verified reachability metric counters during reachability closure computation and metadata export.

### Certification Evidence Matrix

| Gate | Command / Test Filter | Result |
|---|---|---|
| Schema v13 & legacy decode | `cargo test -p phalcom-type-meta --test schema_compat` | PASS (23 passed, 0.01 s) |
| Runtime typing identity | `cargo test -p phalcom-core --lib typing::identity::tests` | PASS (8 passed, 0.00 s) |
| Complete typing suite | `cargo test -p phalcom-core --lib typing::` | PASS (13 passed, 0.00 s) |
| Reachability closure | `cargo test -p phalcom-semantic --lib metadata::reachability::tests` | PASS (4 passed, 0.03 s) |
| Record export & fingerprints | `cargo test -p phalcom-semantic --test semantic record_metadata_` | PASS (4 passed, 0.01 s) |
| Retained alias remainder | `cargo test -p phalcom-semantic --test semantic retained_public_fixed_fields_alias_keeps_heterogeneous_remainder` | PASS (1 passed, 0.00 s) |
| Canonical compiler singleton | `cargo test -p phalcom-core --lib modules::canonical_universe::tests` | PASS (6 passed, 41.09 s) |
| Module compilation options | `cargo test -p phalcom-core --lib modules::compile::tests` | PASS (3 passed, 40.36 s) |
| Metadata compatibility | `cargo test -p phalcom-core --lib modules::materialize::tests` | PASS (1 passed, 0.00 s) |
| Metadata row display | `cargo test -p phalcom-core --test core applied_record_row_metadata_display_preserves_field_types_and_tail` | PASS (1 passed, 0.01 s) |
| Core observability build | `cargo check -p phalcom-core --features perf-observability` | PASS (clean) |
| Semantic build check | `cargo check -p phalcom-semantic` | PASS (clean) |
| Type-meta formatting | `cargo fmt --package phalcom-type-meta -- --check` | PASS (clean) |
| Whitespace hygiene | `git diff --check` | PASS (clean) |
