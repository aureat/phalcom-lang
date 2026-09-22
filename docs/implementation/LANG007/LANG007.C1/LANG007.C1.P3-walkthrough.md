---
id: LANG007.C1.P3
kind: implementation-plan-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
planning_revision: 6551576035a6e55718497c457fac6073b322d599
implementation_revision: 6551576035a6e55718497c457fac6073b322d599
checkpoint: LANG007.C1
program: LANG007
prepared: 2026-09-22
---

# LANG007.C1.P3 — Walkthrough

## 1. Executive Summary

`LANG007.C1.P3` is the checkpoint-closing plan for `LANG007.C1`. It establishes the authoritative as-built architecture baseline for Phalcom's compiler-assisted type-level structural operations, validates cross-layer vertical integration, adds dedicated identity, metadata, incremental cold-parity, and structural composition test suites, audits the runtime-negative and deferred-surface boundaries, re-baselines the future LANG007 roadmap, and successfully closes the `LANG007.C1` checkpoint as `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

## 2. Invariant-to-Evidence Acceptance Matrix (CV-01 .. CV-28)

| Coverage ID | Dimension | Invariant Mapping | Evidence Source | Result |
|---|---|---|---|---|
| `CV-01` | Domain identity | INV-02, INV-04 | `type_annotations`, `integration::type_member_closure` | `PASS` |
| `CV-02` | Const domain | INV-03 | `foundations::type_members`, `integration::type_member_closure` | `PASS` |
| `CV-03` | Member identity | INV-05, INV-06, INV-07, INV-08 | `test_alias_identity_subscript_and_at_converge`, `test_domain_identity_map_members_remain_distinct` | `PASS` |
| `CV-04` | Associated compatibility | INV-09 | `test_associated_type_compatibility_preserves_trait_projections` | `PASS` |
| `CV-05` | Symbolic normalization | INV-10 | `test_symbolic_projection_survives_generic_substitution` | `PASS` |
| `CV-06` | Invalidity | INV-10, INV-15 | `test_index_boundaries_and_out_of_bounds`, `test_at_rejects_non_integer_argument` | `PASS` |
| `CV-07` | Substitution | INV-11 | `test_symbolic_projection_survives_generic_substitution` | `PASS` |
| `CV-08` | Inference lifetime | INV-11 | `record_row_inference`, `test_metadata_rejects_inference_variable_signature_before_publication` | `PASS` |
| `CV-09` | Tuple Map | INV-12, INV-13 | `test_tuple_and_record_map_projection`, `test_structural_map_at_composition_law` | `PASS` |
| `CV-10` | Record Map | INV-12, INV-14 | `test_tuple_and_record_map_projection` | `PASS` |
| `CV-11` | At semantics | INV-15, INV-16 | `test_tuple_subscript_indexing`, `test_index_boundaries_and_out_of_bounds` | `PASS` |
| `CV-12` | Map complexity | INV-16 | `test_moderate_density_tuple_indexing_and_mapping` | `PASS` |
| `CV-13` | Zero products | INV-18 | `test_zero_product_semantics_unit_vs_empty_record` | `PASS` |
| `CV-14` | Stable metadata | INV-19 | `test_stable_metadata_export_for_packs_and_projections`, `integration::metadata` | `PASS` |
| `CV-15` | Module boundary | INV-20 | `test_stable_metadata_export_for_packs_and_projections`, `integration::metadata` | `PASS` |
| `CV-16` | Cold/incremental parity | INV-21 | `incremental::type_members` | `PASS` |
| `CV-17` | Invalidation scope | INV-22 | `incremental::type_members`, `incremental::recomputation_metrics` | `PASS` |
| `CV-18` | Source alias identity | INV-23 | `test_alias_identity_subscript_and_at_converge` | `PASS` |
| `CV-19` | Source domain distinction | INV-08, INV-23 | `test_domain_identity_map_members_remain_distinct` | `PASS` |
| `CV-20` | Intrinsic boundary | INV-24, INV-25 | `parser::parses_intrinsic_declarations`, `parser::intrinsic_attribute_is_not_builtin`, `native_conformance` | `PASS` |
| `CV-21` | No VM execution | INV-26, INV-27 | Static architecture review of `phalcom-core` compiler / lowering / native | `PASS` |
| `CV-22` | Union boundary | INV-17 | `foundations::type_annotations::lowers_union_and_rejects_unsaturated_or_invalid_applications` | `PASS` |
| `CV-23` | Deferred surface audit | INV-28, INV-29 | Negative AST/semantic search | `PASS` |
| `CV-24` | Duplicate authority audit | INV-02, INV-05, INV-30 | Singularity audit across type store, member registry, projection normalizer | `PASS` |
| `CV-25` | Dense structure | INV-16 | `test_moderate_density_tuple_indexing_and_mapping` (64 elements) | `PASS` |
| `CV-26` | Roadmap re-baseline | INV-30 | Updated `PROGRAM.md` capability breakdown | `PASS` |
| `CV-27` | Checkpoint truth | INV-31, INV-32, INV-33 | Checkpoint record, walkthrough, handoff completion | `PASS` |
| `CV-28` | Baseline ledger | INV-33 | `BASELINE-001` and `BASELINE-002` classified and retained | `PASS` |

## 3. Test Modules Added / Updated

1. **`phalcom-semantic/tests/semantic/integration/type_member_closure.rs`** (NEW):
   - `test_alias_identity_subscript_and_at_converge`: Verifies `T[I]` and `T::At<I>` produce the identical `TypeMemberSelector::Subscript` selector, resolve to `IntrinsicTypeOperation::TupleAt`, and normalize to the exact same canonical type for positive and negative indices.
   - `test_domain_identity_map_members_remain_distinct`: Verifies that same-spelled `Map` on `Tuple`, `Record`, `TypePack`, `LabeledTypePack`, and `RecordRow` resolve to 5 distinct `IntrinsicTypeOperation` variants.
   - `test_associated_type_compatibility_preserves_trait_projections`: Proves that trait associated projections retain distinct `TypeData::AssociatedProjection` representation and serialize to `TypeNode::AssociatedProjection`.
   - `test_structural_map_at_composition_law`: Proves the fundamental algebra law `(T::Map<F>)::At<I> ≡ F<T::At<I>>` across positional and labeled elements.
   - `test_zero_product_semantics_unit_vs_empty_record`: Proves `Tuple([], []) ≡ Unit` and `Record({})` is a distinct static Record type, and both preserve their respective forms under `Map<F>`.
   - `test_index_boundaries_and_out_of_bounds`: Proves boundary indexing (0, max, -1, -N) and ensures out-of-bounds indices return `TypeFormResolution::Invalid`.
   - `test_symbolic_projection_survives_generic_substitution`: Proves that abstract projections with type parameters remain symbolic and substitute cleanly via `TypeSubstitution` without leaking query-local inference variables.
   - `test_stable_metadata_export_for_packs_and_projections`: Proves `TypePack` and `LabeledTypePack` export to valid metadata bundles and yield deterministic fingerprints across separate store instances.
   - `test_moderate_density_tuple_indexing_and_mapping`: Proves 64-element Tuple direct O(1) indexing and O(n) Map traversal.

2. **`phalcom-semantic/tests/semantic/incremental/type_members.rs`** (NEW):
   - `test_incremental_tuple_at_edit_matches_cold_session`: Proves that modifying an `At` index from `0` to `1` in an incremental workspace session produces identical formatted return types and diagnostic states as a cold rebuild.
   - `test_incremental_tuple_map_edit_matches_cold_session`: Proves that modifying a structural `Map` constructor from `List` to `Option` produces identical final states between incremental and cold sessions.

## 4. Verification Evidence Ledger

| Suite | Command | Result |
|---|---|---|
| AST intrinsic parsing | `cargo test -p phalcom-ast --test integration intrinsic` | `3 passed` |
| Semantic type members | `cargo test -p phalcom-semantic --test semantic foundations::type_members` | `6 passed` |
| Semantic type annotations | `cargo test -p phalcom-semantic --test semantic foundations::type_annotations` | `40 passed` |
| Record row inference | `cargo test -p phalcom-semantic --test semantic foundations::record_row_inference` | `3 passed` |
| Generic application | `cargo test -p phalcom-semantic --test semantic foundations::generic_application` | `9 passed` |
| Metadata integration | `cargo test -p phalcom-semantic --test semantic integration::metadata` | `14 passed` |
| Native conformance | `cargo test -p phalcom-semantic --test semantic integration::native_conformance` | `3 passed` |
| Type member closure | `cargo test -p phalcom-semantic --test semantic integration::type_member_closure` | `9 passed` |
| Incremental type members | `cargo test -p phalcom-semantic --test semantic incremental::type_members` | `2 passed` |
| Crate checks | `cargo check -p phalcom-ast -p phalcom-type-meta -p phalcom-semantic -p phalcom-core` | `Clean` |
| Formatting & diff check | `cargo fmt --all -- --check && git diff --check` | `Clean` |

## 5. Checkpoint Closure Outcome

`LANG007.C1` is complete and verified at the `FOCUSED_TESTED` level.
- `LANG007.C1.P3`: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`
- `LANG007.C1`: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`
