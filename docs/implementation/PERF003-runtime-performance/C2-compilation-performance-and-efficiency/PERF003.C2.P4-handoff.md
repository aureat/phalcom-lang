# PERF003.C2.P4 handoff — Profile/Root-Driven Metadata Reachability Architecture

Continue on `perf/universe-characterization` in `.worktrees/perf-universe-characterization`.
`CHECKPOINT.md` owns durable status and next action.

---

## 1. Established in this increment

1. **Metadata Reachability Engine (`phalcom-semantic/src/metadata/reachability.rs`)**:
   - `MetadataRootSet` accumulates entry modules, runtime type IDs, declarations, callables, and fields.
   - `MetadataRetentionRequest` pairs roots with `MetadataProfile`.
   - `compute_reachability_closure` implements transitive traversal through nominal types, callable signatures, fields, supertypes, generic constraints, and aliases.
   - Canonical Universe entities (`module_id.project.is_universe()`) are strictly elided as external authorities.
   - Proven by unit tests: `metadata_root_set_tracks_cardinalities`, `reachability_closure_minimal_vs_public_retention`, and `reachability_closure_elides_canonical_authorities`.

2. **Callable & Field Owner Indexes (`phalcom-semantic/src/signature.rs`)**:
   - `CallableSignatureTable` indexes callables by `CallableOwnerId` via `owner_callables`. Exposes `iter_owner` and `iter_declaration_owner`.
   - `FieldSignatureTable` indexes fields by `DeclarationId` via `owner_fields`. Exposes `iter_owner`.
   - Incremental mutation/removal maintains the indexes; tested via `callable_owner_index_tracks_membership_removal_and_cloning` and `field_owner_index_tracks_membership_removal_and_cloning`.
   - Eliminates $O(\text{declarations} \times \text{callables})$ table scans.

3. **Observability Counters (`telos-observability/src/metric.rs`)**:
   - 14 `PerfCounter` variants added for metadata roots, dependency edges, retained entities, elisions, and lookups.

---

## 2. Invariants and Constraints for Successor Work

- **Default Profile**: Retain `MetadataProfile::RuntimePublic` as the default for compilation unless explicitly overridden; do not globally switch to `RuntimeMinimal` across all compiler callers.
- **Cross-Pool Identities**: Numeric `MetadataPoolId(0)` must never be serialized or assumed to be the canonical pool ID; resolution between user and canonical pools must occur via stable identities (`StableDeclarationRef`, `StableCallableRef`, `StableFieldRef`).
- **Layer Independence**: `phalcom-type-meta` bundle validation must not depend on VM runtime registry state.
- **Verification Mode**: Maintain BUILD MODE. Avoid running broad workspace test or clippy sweeps prematurely.

---

## 3. Ordered Next Actions

1. **`P4.T1` — Compiler Metadata Retention Options Plumbing**:
   - In `phalcom-core/src/modules/compile.rs`: add `ProgramCompilationOptions { pub metadata_profile: MetadataProfile }` with `Default = RuntimePublic`.
   - Add `compile_analyzed_with_options` and thread `options.metadata_profile` to `project_analyzed_inner`.
   - Update `compile_session.rs` to allow `compile_fixture_with_options`.

2. **`P4.T5` — Cut Exporter Over to Closure**:
   - In `phalcom-semantic/src/metadata/export.rs`: add `build_bundle_from_closure(mut self, closure: &MetadataReachabilityClosure, runtime_roots: &[(&ModuleId, &str, TypeId)])`.
   - Replace raw table loops with closure iteration (`closure.declarations`, `closure.callables`, `closure.fields`, `closure.aliases`).
   - Use `callables.iter_declaration_owner(decl_id)` and `fields.iter_owner(decl_id)`.
   - Wire `field_signatures` from `SemanticSnapshot` into exporter.

3. **`P4.T6` — Canonical Metadata External Authority Partitioning**:
   - In `compile.rs:project_analyzed_inner`: exclude canonical Universe declarations from ordinary user compilation closures. Canonical entities are installed once in pool 0 during bootstrap.

4. **`P4.T7` & `P4.T8` — Runtime Registry Stable-Identity Indexes & Reflection Cutover**:
   - In `phalcom-core/src/typing/registry.rs`: add `declaration_metadata: HashMap<StableDeclarationRef, RuntimeDeclarationMetadataRef>`, `callable_metadata`, and `field_metadata`.
   - Populate upon pool registration. Add `lookup_declaration_metadata`, `lookup_callable_metadata`, `lookup_field_metadata`.
   - In `phalcom-core/src/typing/inspect.rs`: replace linear scans and same-pool assumptions in `declaration_record_for_class`, `base_constructor_arity`, and `lookup_member` with cross-pool registry lookups.

5. **`P4.T9` — Canonical Dependency Compatibility**:
   - Record `MetadataDependencyIdentity::CanonicalUniverse { interface_fingerprint }` in `CompiledProgram` and validate on materialization.

6. **`P4.T11` — Verification**:
   - Run focused compiler and reflection regression suites:
     - `cargo test -p phalcom-core --test core compiler::product_optimization`
     - `cargo test -p phalcom-core --test core reflection::`

---

## 4. Key Verification Commands

```sh
# Reachability and signature index tests
RUSTFLAGS='' cargo test -p phalcom-semantic --lib metadata::reachability::tests
RUSTFLAGS='' cargo test -p phalcom-semantic --lib signature::tests

# Crate checks
RUSTFLAGS='' cargo check -p telos-observability
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo check -p phalcom-core
```
