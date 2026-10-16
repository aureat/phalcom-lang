# Plan 2 source-semantic coverage ledger

This ledger tracks the 106 laws from Plan 2. A slot is marked `READY` only
when an enabled source-level test proves the law at its required depth. Foundation
and database tests remain valuable, but do not satisfy source slots by themselves.

Status vocabulary: `READY`, `BLOCKED`, `RED-CAPABILITY`, `STAGED`, `GATED`.

Current ledger count after C12 type-system closure: **93 READY, 7 STAGED, 5 GATED, 1 BLOCKED**.
A green test run never promotes a staged/gated law implicitly; promotion requires
a named source test and a concrete semantic oracle.

| Slot | Status | Source evidence or concrete prerequisite |
| --- | --- | --- |
| E01 | READY | `authority::exact_literal_proof_refutes_annotation...`; `flow_branches::same_type_branch_results...` |
| E02 | READY | `authority::direct_constructor_result...`; `authority::inherited_constructor_specializes_self...` |
| E03 | READY | `generics::assumed_generic_argument...`; callable parameter assertions in `authority` |
| E04 | READY | `authority::unknown_initializer_allows_developer_annotation...` |
| E05 | STAGED | Source unresolved-name plus contract ownership oracle |
| E06 | READY | `flow_branches::known_branch_does_not_hide_reachable_unknown...`; `deep_regressions::branch_join_preserves_exact_unknown_reason` |
| E07 | READY | `checker_smoke` dynamic-boundary scenarios |
| E08 | READY | `generics::mixed_generic_return_uses_weakest_value_support` |
| B01 | READY | `authority::compatible_supertype_annotation...`; `flow_branches::branch_union_validates...` |
| B02 | READY | `authority::exact_literal_proof_refutes_annotation...` |
| B03 | READY | `authority::binding_kind_controls_mutability...` |
| B04 | READY | `authority::annotation_diagnostic_root_cause...`; `deep_regressions::refuted_branch_write_preserves_recovery_union_and_diagnostic_owner` |
| B05 | READY | `flow_branches::branch_local_shadow_does_not_mutate_outer...` |
| B06 | READY | `structural::tuple_destructuring_with_broad_contract...` |
| B07 | READY | `authority::annotation_diagnostic_root_cause...` |
| D01 | READY | `dispatch_capabilities::chained_dispatch...`; `dispatch_class_side::instance_receiver...` |
| D02 | READY | `dispatch_class_side::class_object_dispatches...` |
| D03 | READY | `dispatch_capabilities::wrong_class_instance_dispatch_side...` |
| D04 | READY | `dispatch_class_side::super_send...`; inherited callable tests in `authority` |
| D05 | READY | `authority::inherited_constructor_specializes_self...`; `self_types` |
| D06 | READY | `capabilities::variance::source_nested_and_higher_kinded_self_specialize_to_receiver` |
| D07 | READY | `dispatch_capabilities::selector_label_mismatch...` |
| D08 | READY | `dispatch_capabilities::argument_refutation_preserves...` |
| F01 | READY | `families::values::captured_variant_family_preserves_exact_authorized_member_shapes`; `families::invocation::behavioral_family_source_preserves_pattern_storage_and_dispatch_side` |
| F02 | READY | `families::invocation::behavioral_family_source_preserves_pattern_storage_and_dispatch_side` |
| F03 | READY | `families::values::captured_variant_family_preserves_exact_authorized_member_shapes`; `families::invocation::behavioral_family_source_preserves_pattern_storage_and_dispatch_side` |
| F04 | READY | `families::invocation::{stored_family_call_publishes_static_application_resolution,behavioral_family_source_preserves_pattern_storage_and_dispatch_side}` |
| F05 | READY | `integration::family_capabilities::stored_family_capture_preserves_its_denotation_through_local_flow` |
| F06 | READY | `families::invocation::behavioral_family_source_preserves_pattern_storage_and_dispatch_side` |
| F07 | READY | `associated::inheritance::inherited_associated_lookup_keeps_descendant_lookup_and_ancestor_definition` |
| F08 | READY | `families::invocation::family_wrong_shape_reports_associated_call_shape` |
| F09 | READY | `families::invocation::{generic_behavioral_family_recovers_canonical_signature_for_expected_result,generic_associated_variant_family_recovers_constructor_signature}` |
| G01 | READY | `capabilities::constraints::source_constraints_preserve_class_and_callable_owners` |
| G02 | READY | `generics::generic_identity_solves...` |
| G03 | READY | `generics::generic_pair_solves...` |
| G04 | READY | `capabilities::variance::source_nested_and_higher_kinded_self_specialize_to_receiver` |
| G05 | READY | `capabilities::constraints::source_generic_superclass_constraint_substitutes_owner_parameter` |
| G06 | READY | `generics::expected_result_context_constrains...` |
| G07 | READY | `generics::conflicting_generic_constraints...` |
| G08 | READY | `generics::expected_context_selects_but_does_not_establish_result_only_generic` |
| G09 | READY | `capabilities::higher_kinded_generics::hkt_self_specializes_through_multi_hop_transformed_inheritance` |
| C01 | READY | `capabilities::constraints::source_equivalence_and_lower_bound_constraints_control_calls` |
| C02 | READY | `capabilities::constraints::source_constraints_preserve_class_and_callable_owners` |
| C03 | READY | `capabilities::constraints::source_equivalence_and_lower_bound_constraints_control_calls` |
| C04 | READY | `capabilities::constraints::source_equivalence_and_lower_bound_constraints_control_calls` |
| C05 | READY | `capabilities::constraints::source_constraints_preserve_class_and_callable_owners` |
| C06 | READY | `capabilities::constraints::source_generic_superclass_constraint_substitutes_owner_parameter` |
| C07 | READY | `capabilities::constraints::source_equivalence_and_lower_bound_constraints_control_calls` |
| K01 | READY | `integration::workspace::generic_declaration_kind_matches_published_signature` |
| K02 | READY | `integration::workspace::generic_declaration_kind_matches_published_signature`; `capabilities::aliases::source_aliases_preserve_canonical_forms_and_generic_inference` |
| K03 | READY | `integration::workspace::generic_declaration_kind_matches_published_signature` |
| K04 | READY | `capabilities::generics::wrong_kind_higher_kinded_argument_is_rejected_structurally` |
| K05 | READY | `capabilities::aliases::source_aliases_preserve_canonical_forms_and_generic_inference` |
| K06 | READY | `capabilities::aliases::source_aliases_preserve_canonical_forms_and_generic_inference` |
| K07 | READY | `capabilities::aliases::source_aliases_preserve_canonical_forms_and_generic_inference` |
| K08 | READY | `capabilities::aliases::source_aliases_preserve_canonical_forms_and_generic_inference` |
| K09 | BLOCKED | Dedicated source arity/kind diagnostic oracle remains unproven; lower-level kind rejection coverage is retained |
| V01 | READY | `capabilities::variance::source_variance_reaches_canonical_relation_and_superclass_projection` |
| V02 | READY | `capabilities::variance::source_variance_reaches_canonical_relation_and_superclass_projection` |
| V03 | READY | `capabilities::variance::source_variance_reaches_canonical_relation_and_superclass_projection` |
| V04 | READY | `capabilities::variance::source_variance_reaches_canonical_relation_and_superclass_projection` |
| V05 | READY | `capabilities::variance::source_variance_reaches_canonical_relation_and_superclass_projection` |
| V06 | READY | `capabilities::variance::source_nested_callable_occurrence_has_contravariant_polarity` |
| S01 | READY | `structural::nested_tuple_composes...` |
| S02 | STAGED | Canonical labeled tuple source structure |
| S03 | READY | `structural::record_literal_preserves_structural_field_types` asserts exact closed fields |
| S04 | READY | `structural::heterogeneous_collection...` |
| S05 | READY | `patterns::nested_tuple_pattern_recursively_establishes_each_leaf` |
| S06 | READY | `patterns::collection_and_destructure_facts_preserve_element_shapes` |
| S07 | GATED | Pattern mismatch ownership and recovery |
| S08 | READY | `patterns::generic_pair_result_can_be_destructured_without_losing_components` |
| L01 | READY | `flow_branches::same_type_branch_results...`; `deep_regressions::same_type_branch_writes_publish_flow_provenance` |
| L02 | READY | `flow_branches::heterogeneous_branch_results...` |
| L03 | READY | `flow_branches::returning_branch...` |
| L04 | READY | `flow_branches::throwing_branch...` |
| L05 | READY | `flow_branches::nested_branch_results...`; exact unknown trace in `deep_regressions` |
| L06 | READY | `flow_loops::loop_join_includes_preheader...`; `deep_regressions::loop_join_publishes_flow_provenance_without_widening_to_contract` |
| L07 | READY | `flow_loops::break_and_continue...` |
| L08 | READY | `flow_loops::captured_block_write...`; `deep_regressions::closure_construction_preserves_outer_flow_provenance` |
| I01 | READY | `iteration::custom_iterable_element_type_comes_from_protocol_not_first_generic_argument` |
| I02 | READY | `iteration::generic_iterator_receiver_substitution_selects_second_parameter` |
| I03 | READY | `iteration::for_tuple_pattern_decomposes_protocol_element_type` |
| I04 | GATED | Multi-lane/index iteration source protocol |
| I05 | READY | Unknown protocol result remains incomplete |
| P01 | STAGED | Contextual closure parameter publication |
| P02 | READY | `callable_publication_capabilities::branch_derived_tail...` |
| P03 | GATED | Source callable subtyping relation |
| P04 | STAGED | Nested closure capture identity oracle |
| P05 | READY | `callable_publication_capabilities::branch_derived_tail...` |
| P06 | STAGED | Multi-hop callable publication invalidation |
| P07 | READY | `callable_publication_capabilities::recursive_inference...` |
| A01 | READY | `fields::field_facts_survive_constructor_and_general_writes`; default and constructor lifecycle tests |
| A02 | STAGED | Generic receiver field specialization |
| A03 | GATED | Inherited Self-typed field source API |
| A04 | READY | `capabilities::aliases::source_aliases_preserve_canonical_forms_and_generic_inference` |
| A05 | READY | `capabilities::aliases::source_aliases_preserve_canonical_forms_and_generic_inference`; `integration::workspace::transparent_aliases_preserve_canonical_forms_and_kinds` |
| M01 | READY | `integration::workspace::same_leaf_name...` |
| M02 | READY | `integration::workspace` imported inheritance/call target scenarios |
| M03 | STAGED | Re-export source graph publication |
| M04 | READY | `incremental::db` and `checker_dependencies` invalidation scenarios |
| X01 | READY | `authority::binding_contract_explanation...`; explanation foundation |
| X02 | READY | `authority::annotation_diagnostic_root_cause...` |
| X03 | READY | `incremental::callable_dependencies` and `checker_dependencies` |
| X04 | READY | `integration::advisory` and `foundations::advisory_domain` |
| X05 | READY | `integration::native_conformance` |
| X06 | GATED | Reflection source API; retain lower-level metadata tests |

The ledger is intentionally honest about product prerequisites. Gated and
staged slots remain promotion targets; they are not deleted tests or ignored
coverage. New source fixtures must use canonical pipe-block syntax.

C12 non-type-system remainder: `E05`, `S02`, `S07`, `I04`, `P01`, `P03`,
`P04`, `P06`, `A02`, `A03`, `M03`, and `X06` remain staged/gated for
epistemic diagnostics, structural pattern diagnostics, protocol lanes,
callable/publication depth, generic receiver fields, module re-export, or
reflection-source coverage. `K09` is explicitly blocked above because no
dedicated source arity/kind diagnostic oracle is currently ratified.

`BodyExitFacts` trace assertions for nested `return`/`throw` remain a separate
product-hardening prerequisite: the current published product does not yet
represent those abrupt paths with sufficient fidelity, so this ledger does not
claim that deeper oracle merely because the branch value tests are green.

## Part 06 closure laws

| Law | Status | Evidence |
| --- | --- | --- |
| C1-C2 canonical identity and core shadowing | READY | `foundations::authority_boundaries::{user_object_name_is_not_universal_supertype,user_function_name_is_not_callable_supertype}` |
| C3-C4 live TypeStore materialization | READY | `foundations::authority_boundaries::generic_supertype_specialization_materializes_in_live_store` |
| C5 relation non-strengthening | READY | `foundations::authority_boundaries::proven_relation_does_not_upgrade_assumed_actual` |
| C6-C8 comparison result proof and single evaluation | READY | `foundations::authority_boundaries::{comparison_chain_single_evaluation_and_operation_conjunction,comparison_chain_missing_operator_fails_closed}` |
| C9 membership fail-closed | READY | `foundations::authority_boundaries::membership_fails_closed_to_unknown` |
| C10-C12 contextual-empty authority | READY | `foundations::authority_boundaries::{contextual_empty_list_inherits_expected_contract_authority,contextual_empty_map_preserves_expected_type}`; wrong collection context remains formal Unknown |
| C13-C14 established-site and recovery quarantine | READY | `docs/impl/semantic/semantic-correctness/part-4/2026-08-29-semantic-authority-audit.md`; authority and composition regressions |
| C15 incremental equivalence | READY | `incremental::type_store_revisions`, `incremental::fingerprints`, `incremental::callable_dependencies` |
| C16 presentation non-authority | READY | `integration::{advisory_analysis,presentation,editor_type_hints}` |
| C17 no hidden language design | READY | membership and unsupported comparison links stay `Unknown(UncheckedExpression)` |

## Semantic gate ignore audit — 2026-09-01

This section records the ignore-marker audit for the crate's unified
`tests/semantic.rs` integration binary. It is a crate-local tracking record,
not a workspace completion claim.

### Scope and current result

Command:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --nocapture
```

This is the `phalcom-semantic` package integration test only. It is not
`cargo test --workspace`; workspace-wide tests were not run.

The source contains 100 `#[ignore]` markers. The earlier count of 99 omitted
the intentionally ignored fail-fast child-process test. This audit removed 48
requested markers and corrected/activated one additional diagnostic fixture,
leaving 51 ignored tests.

Current working-tree result:

```text
active semantic gate: 905 passed, 2 failed, 51 ignored
forced ignored run:   20 passed, 31 failed
```

All 49 newly active tests passed. The corrected test is now named
`match_diag_04_payload_arity_mismatch_points_at_projection`; its fixture tests
payload arity and diagnostic range, rather than claiming unimplemented private
cross-module visibility.

### Two active failures

These are the only failures in the active gate after the 49-test activation:

| Test | Classification | Observation |
| --- | --- | --- |
| `foundations::expression_engine::test_keyword_argument_mismatch_detected` | Semantic implementation bug | Passing `"invalid"` to an `Int` keyword parameter produces no `ArgumentMismatch`. The call checker is not validating this keyword argument path. |
| `support::regressions::union_expectation_rejects_wrong_structural_members` | Test-harness bug | The oracle intentionally panics, but the workspace uses `panic = "abort"`; `catch_unwind` cannot catch it. Use a child-process assertion or a non-panicking oracle API. |

### Remaining ignored tests: 51

#### 31 fail when explicitly forced

These remain ignored because they expose actual prerequisites, implementation
gaps, placeholder fixtures, stale logic, or intentional process termination.

Parser/fixture prerequisites — 10:

```text
semantic::adts::matching::bindings::review_m6_04_outer_same_name_binding_is_restored_after_match
semantic::adts::matching::exhaustiveness::match_exh_13_tuple_product_is_exhaustive_when_all_products_are_covered
semantic::adts::matching::exhaustiveness::match_exh_14_list_partition_is_exhaustive
semantic::adts::matching::exhaustiveness::review_m5_02_missing_two_field_combination_is_complete
semantic::adts::matching::exhaustiveness::review_m5_03_nested_multi_field_witness_preserves_child_tree
semantic::adts::matching::exhaustiveness::review_m5_04_labeled_multi_field_witness_maps_external_labels
semantic::adts::matching::flow::match_flow_03_abrupt_arm_is_excluded_from_normal_result_join
semantic::adts::matching::flow::match_flow_04_all_abrupt_arms_have_never_result
semantic::adts::matching::flow::match_flow_10_branch_writes_join_after_match
semantic::adts::variants::adt_variant_08_private_variant_name_is_not_explicitly_acquirable
```

The source fixtures use syntax the parser currently rejects: missing statement
separators, tuple/list pattern forms, string literal patterns, `return` as a
match-arm expression, or `_Dog` as a variant name. These are parser or fixture
gaps, not evidence against the underlying semantic law.

Semantic diagnostic, resolution, exhaustiveness, and GADT gaps — 5:

```text
semantic::adts::matching::diagnostics::match_diag_02_ambiguous_variant_has_owner_candidates
semantic::adts::matching::diagnostics::match_diag_03_inaccessible_variant_points_at_explicit_name
semantic::adts::matching::exhaustiveness::match_exh_06_callable_family_leaves_singleton_residual
semantic::adts::matching::gadt_refinement::match_gadt_06_nested_gadt_proof_is_branch_local
semantic::adts::matching::resolution::match_res_08_ambiguous_contextual_owner_reports_no_arbitrary_candidate
```

These are genuine semantic implementation gaps: missing ambiguity/visibility
diagnostics, incorrect callable-family residual classification, missing nested
GADT branch proof, and missing ambiguous-owner resolution behavior.

Golden pipeline implementation gaps — 12:

```text
semantic::golden::golden_01_generic_self_chain
semantic::golden::golden_02_flow_pattern_publication
semantic::golden::golden_03_iterator_chain
semantic::golden::golden_04_family_callable
semantic::golden::golden_05_program_parses
semantic::golden::golden_05_type_lambda_constraints
semantic::golden::golden_06_workspace_chain
semantic::golden::golden_08_variance_recovery
semantic::golden::golden_09_closure_flow
semantic::golden::golden_10_mixed_pipeline
semantic::golden::golden_11_recursive_fixed_point
semantic::golden::golden_15_row_effect_contract
```

These cover generic `Self`, component-wise flow products, iterator inference,
formal Family calls, type-lambda parsing/constraints, linked workspace
publication, variance recovery, closure typing, record publication, recursive
fixed points, and row/effect contracts. They are broader implementation gaps,
not canonical-Universe integration failures.

Explicit placeholder visibility tests — 2:

```text
semantic::adts::variants::adt_variant_09_construction_visibility_is_independent_from_match_universe
semantic::adts::variants::adt_variant_10_private_payload_rejects_projection_but_allows_wildcard_ignore
```

`adt_variant_09` creates one local module and then explicitly panics because it
has no producer/consumer fixture for construction versus match visibility.
`adt_variant_10` does the same for private payload projection versus wildcard
matching. Neither test currently provides a product verdict.

Stale logical test — 1:

```text
semantic::incremental::adts::adt_incr_09_visibility_edit_invalidates_acquisition_without_shrinking_match_universe
```

The fixture changes a branch literal from `1` to `2`; it does not edit
visibility. The expected debug-product difference is therefore logically
stale. Rewrite the scenario around an actual cross-module visibility edit.

Intentional fail-fast child — 1:

```text
semantic::support::regressions::fail_fast_policy_panics_only_after_recording_incident_child
```

The parent test launches this child in a subprocess, enables `FailFast`, and
asserts that the child exits unsuccessfully only after recording the internal
incident. Running the child directly is supposed to fail; it remains ignored
so the ordinary test gate does not treat intentional termination as a product
failure.

#### 20 pass when explicitly forced

One is the benchmark and remains ignored by policy:

```text
semantic::adts::matching::pattern_space::review_m2_05_union_normalization_benchmark_shapes_are_registered
```

The other 19 are green but remain gated because they are vacuous, unsupported,
performance-only, or do not exercise the complete claimed boundary:

Alias dependency/contraction coverage — 5:

```text
semantic::adts::exact_cases::adt_exact_06_transparent_alias_union_matches_direct_exact_union
semantic::adts::matching::exhaustiveness::match_exh_08_alias_union_exhaustiveness_is_not_root_widened
semantic::incremental::adts::adt_incr_07_alias_union_expansion_invalidates_exhaustiveness
semantic::incremental::adts::adt_incr_08_alias_union_contraction_updates_residual_witness
semantic::incremental::match_analysis::adt_incr_match_analysis_records_alias_union_dependency
```

These local alias checks now pass, but complete alias dependency/publication
semantics have not been promoted as a release claim. They are next candidates
for unignore after the current uncommitted alias work is explicitly accepted.

Visibility and associated-family coverage — 5:

```text
semantic::adts::associated::scenarios::adt_assoc_11_inherited_family_keeps_lookup_and_definition_owners
semantic::adts::associated::scenarios::adt_assoc_14_private_member_is_not_explicitly_acquirable
semantic::adts::associated::scenarios::adt_assoc_15_frozen_family_does_not_acquire_later_members
semantic::adts::associated::scenarios::adt_assoc_16_family_value_does_not_escape_member_visibility
semantic::adts::associated::visibility::visibility_scenarios_require_cross_module_fixture_support
```

These fixtures remain single-module or assert only non-empty/exact local
products. They do not prove inheritance ownership, private acquisition,
frozen-family behavior, capability escape, or cross-module visibility.

Unsupported record/map pattern coverage — 5:

```text
semantic::adts::matching::patterns::review_c4_01_record_pattern_is_not_silently_converted_to_wildcard
semantic::adts::matching::patterns::review_c4_02_map_pattern_is_not_silently_converted_to_wildcard
semantic::adts::matching::patterns::review_c4_03_unsupported_record_does_not_make_match_exhaustive
semantic::adts::matching::patterns::review_c4_04_unsupported_map_does_not_make_later_arm_redundant
semantic::adts::matching::patterns::review_c4_05_resolver_has_no_catch_all_wildcard_fallback
```

These are conservative-fallback safety checks. They do not establish that
record/map pattern resolution has landed.

Other incomplete boundaries — 4:

```text
semantic::adts::matching::exhaustiveness::match_exh_15_open_object_requires_wildcard
semantic::adts::matching::gadt_refinement::match_gadt_10_branch_proof_does_not_leak_after_match
semantic::adts::matching::conformance::review_x_02_invalid_semantic_match_never_reaches_lowering
semantic::adts::matching::pattern_space::review_m2_04_union_dedup_growth_is_near_linear
```

`match_exh_15` depends on unsupported record-pattern behavior. `match_gadt_10`
has no post-match use that could observe leakage. `review_x_02` runs semantic
analysis but not the lowering boundary. `review_m2_04` lacks the instrumentation
needed to prove normalization complexity.

### Historical 16-failure comparison

On clean pushed commit `1c78f5d2`, the semantic gate had 16 failures: 14
tests could not retrieve their `Probe` callable through the fixture semantic
index, one refined-return diagnostic was missing, and one body-query
invalidation test returned a non-ready product. Later uncommitted semantic
edits make those 16 pass in the current working tree. The current two active
failures are the keyword-argument checker bug and the panic-abort test-harness
bug recorded above.

This audit does not claim semantic or workspace completion. The remaining
ignored failures and the two active failures require the classifications and
follow-up work recorded here.

## Consolidation-universe semantic run — 2026-10-15

This later run supersedes the September audit counts for the current
feat/consolidation-universe checkout. It does not rewrite the historical
September results above.

### Results

The regular semantic binary was run with only the stack-overflowing test
filtered out:

    RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic -- --skip ordinary_float_horner_expression_is_stack_safe_without_realization --quiet

Result: 2,104 passed, 94 failed, 42 ignored, and 1 filtered out. The filtered
test, ordinary_float_horner_expression_is_stack_safe_without_realization,
overflows its 2 MiB analysis thread and aborts the test process. The separate
typed_expression_remains_a_lightweight_recursive_carrier test ran and failed:
TypedExpression measured 896 bytes against a 760-byte limit.

Before changing ignore markers, the forced ignored run was:

    RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic -- --ignored --quiet

Result: 19 passed, 23 failed. The 19 passing tests were then moved into the
regular semantic gate by removing their ignore attributes. Each was rerun
individually in normal mode; all 19 passed. The live ignored-test listing now
contains 23 tests. The full regular gate was not rerun after this marker-only
change, so 2,123 passed / 94 failed / 23 ignored is an arithmetic projection,
not a completed post-change suite result.

Removing an ignore marker records that the test passes in this checkout. It
does not by itself promote a broader semantic law to READY: several enabled
fixtures assert a narrower local property than their historical test name or
gate reason implies. Their oracle scope should be tightened before relying on
them as evidence for cross-module visibility, inheritance, post-match flow,
or lowering integration.

### Newly enabled tests

- ADT associated scenarios: adt_assoc_11_inherited_family_keeps_lookup_and_definition_owners; adt_assoc_14_private_member_is_not_explicitly_acquirable; adt_assoc_15_frozen_family_does_not_acquire_later_members; adt_assoc_16_family_value_does_not_escape_member_visibility.
- ADT visibility and exact cases: visibility_scenarios_require_cross_module_fixture_support; adt_exact_06_transparent_alias_union_matches_direct_exact_union.
- Match semantics: review_x_02_invalid_semantic_match_never_reaches_lowering; match_exh_08_alias_union_exhaustiveness_is_not_root_widened; match_exh_15_open_object_requires_wildcard; match_gadt_10_branch_proof_does_not_leak_after_match.
- Pattern fallback: review_c4_01_record_pattern_is_not_silently_converted_to_wildcard; review_c4_02_map_pattern_is_not_silently_converted_to_wildcard; review_c4_03_unsupported_record_does_not_make_match_exhaustive; review_c4_04_unsupported_map_does_not_make_later_arm_redundant; review_c4_05_resolver_has_no_catch_all_wildcard_fallback.
- Golden and incremental: golden_05_program_parses; golden_05_type_lambda_constraints; adt_incr_07_alias_union_expansion_invalidates_exhaustiveness; adt_incr_match_analysis_records_alias_union_dependency.

### Regular-run failures (94)

The names below are relative to the semantic integration test binary.

- Top-level: export_invalidation::test_previously_missing_public_name_appears_invalidates_consumer; adts::existentials::structural_wrapper_cannot_hide_local_existential_escape; advanced::integration_matrix::test_matrix_1_record_row_subtyping_and_unification.
- Capabilities: algebraic_realization::typed_expression_remains_a_lightweight_recursive_carrier; aliases::source_alias_cycle_and_malformed_target_are_quarantined; authority::annotation_diagnostic_root_cause_marks_binding_without_erasing_value; flow_loops::{break_and_continue_preserve_loop_exit_and_backedge_facts, field_mutation_through_loop_preserves_contract, loop_fixpoint_preserves_mutated_integer_and_abrupt_edges}; generic_associated_constraints::infer_generic_method_call_with_associated_specialization; generics::{generic_pack_query_constructor_preserves_abstract_field_lane, higher_kinded_parameter_infers_list_constructor_and_element, universe_future_preserves_payload_types_through_async_map_then_and_await}; iteration::{custom_iterable_element_type_comes_from_protocol_not_first_generic_argument, for_tuple_pattern_decomposes_protocol_element_type, generic_iterator_receiver_substitution_selects_second_parameter, heterogeneous_collection_iteration_preserves_element_union, iterator_adapter_fixture_preserves_item_projections_and_chain_equality}; patterns::collection_and_destructure_facts_preserve_element_shapes; structural::contextual_product_components_propagate_to_nested_literals; traits::{source_universe_integer_constants_and_trait_defaults_publish_to_clients, trait_default_replays_external_nominal_callable_dependencies, trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities}.
- Foundations: authority_boundaries::contextual_empty_list_inherits_expected_contract_authority; bidirectional_calls::test_bidirectional_empty_collections; canonical_call_application::missing_iterable_conformance_fails_closed_without_cursor_protocol; expression_composition::{assumed_expansion_weakens_aggregate, assumed_list_element_weakens_aggregate, for_loop_preserves_iterable_unknown_reason, incompatible_open_record_expansions_have_explicit_diagnostic, list_unknown_element_does_not_disappear, map_expansion_does_not_fabricate_record_row, unknown_expansions_do_not_disappear}; expression_engine::{test_block_and_control_flow_typing, test_for_loop_protocol_custom_iterable}; flow_graph::test_protocol_derived_iteration_typing_no_name_matching; static_values_pack::static_pack_storage_slicing_and_equality; static_values_sequence::static_sequence_materialization_is_recursive_and_truthfully_deferred.
- Golden and impl queries: golden::{golden_07_unknown_authority, golden_12_higher_order_mutation}; impls::queries::{adding_impl_member_does_not_invalidate_another_target_or_its_caller, unrelated_class_body_edit_reuses_impl_callable_and_surface}.
- Incremental: adts::adt_incr_10_unrelated_method_edit_reuses_match_analysis; callable_dependencies::{case_a_unchanged_caller_changed_callee_signature_recomputes, case_b_callee_body_change_signature_unchanged_reuses_caller, case_c_field_annotation_change_recomputes_reader, case_d_superclass_change_recomputes_inherited_call, case_f_unrelated_edit_reuses_unaffected_callables, case_j_generic_supertype_template_edit_invalidates_specialized_caller_and_matches_cold_rebuild, dependency_edit_remove_readd_recomputes_affected_summary_deterministically, direct_result_cycle_diagnostics_track_incremental_edges_and_match_cold_analysis, inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once}; checker_dependencies::{checker_body_sources_do_not_bypass_tracked_resolver_or_declaration_helpers, declaration_metadata_read_records_declaration_shell_dependency, legacy_universe_core_path_is_not_a_dependency_sentinel}; declaration_authority::field_signature_is_a_first_class_incremental_product; derived_conformance::trait_contract_edit_invalidates_derived_evidence; generic_contracts::constructor_generic_bound_edit_invalidates_dependent_call_and_matches_cold; product_stability::{binding_flow_edit_rechecks_only_the_affected_callable, body_only_edit_stops_at_stable_semantic_products, diagnostic_aggregate_matches_cold_after_repair_and_unrelated_edit, field_mutability_edit_invalidates_field_and_write_products, inferred_cross_module_result_updates_dependents_and_matches_cold_analysis, pa8_recomputed_stable_intermediate_allows_downstream_reuse, previously_absent_name_addition_recomputes_exact_consumer, range_only_body_edit_reuses_semantic_callers, signature_edit_recomputes_exact_callers_and_reuses_unrelated_bodies, stable_intermediate_callable_product_stops_later_propagation, superclass_edit_recomputes_hierarchy_consumers_without_touching_unrelated_bodies, unrelated_edit_retains_unaffected_module_diagnostics}; recomputation_metrics::{a6_cross_module_alias_relowers_when_provider_shell_changes, a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products, a6_high_fanout_unused_provider_body_edit_has_no_external_semantic_recompute, pa9_large_hierarchy_edit_isolates_unrelated_formal_products, unused_public_export_does_not_recompute_unrelated_consumer_body}; record_rows::{row_body_only_edit_preserves_signature_semantics, row_prefix_edit_invalidates_dependent_call}; t12_6::t12_6_tuple_and_record_intrinsic_member_result_kind_edits_match_cold; type_members::{associated_static_case_alias_uses_domain_catalog, canonical_universe_intrinsic_type_members_match_evaluator_catalog, static_case_rename_invalidates_alias_and_matches_cold_analysis}; type_store_revisions::{publication_effects_distinguish_initial_graph_build_from_body_edit, stable_type_store_id_and_snapshots_across_revisions}.
- Integration and workspace: advisory::analysis::formal_projection_preserves_causal_invalidity_and_binding_contracts; imported_resolution::{imported_nominal_binder_header_supports_variadic_alias_forwarding, imported_static_domain_provider_edits_recompute_exact_consumer_and_match_cold, p3_cross_module_provider_pack_alias_survives_and_recomputes_cold_parity}; metadata::legacy_record_metadata_fails_closed_for_non_symbol_keys_and_domains; native_conformance::test_native_surface_conformance; record_row_polymorphism::{repeated_row_parameter_conflict_is_rejected, row_conflict_message_does_not_expose_solver_variables}; workspace::{alias_cycles_are_rejected_before_publication, imported_alias_cycle_is_rejected_before_publication, transparent_aliases_preserve_canonical_forms_and_kinds}.

### Forced ignored-test failures (23)

These failures were observed before the 19 passing cases were enabled.

- Parser or fixture failures: adts::matching::bindings::review_m6_04_outer_same_name_binding_is_restored_after_match; adts::matching::exhaustiveness::{review_m5_02_missing_two_field_combination_is_complete, review_m5_03_nested_multi_field_witness_preserves_child_tree, review_m5_04_labeled_multi_field_witness_maps_external_labels}; adts::matching::flow::{match_flow_03_abrupt_arm_is_excluded_from_normal_result_join, match_flow_04_all_abrupt_arms_have_never_result, match_flow_10_branch_writes_join_after_match}; adts::variants::adt_variant_08_private_variant_name_is_not_explicitly_acquirable. The parser rejected statement separators, string patterns, return expressions in match arms, and the _Dog variant spelling before the intended semantic checks.
- Gated visibility placeholders: adts::variants::{adt_variant_09_construction_visibility_is_independent_from_match_universe, adt_variant_10_private_payload_rejects_projection_but_allows_wildcard_ignore}. These explicitly panic because the cross-module fixtures are not implemented.
- Gated golden behavior: golden::{golden_01_generic_self_chain, golden_02_flow_pattern_publication, golden_03_iterator_chain, golden_04_family_callable, golden_06_workspace_chain, golden_08_variance_recovery, golden_09_closure_flow, golden_10_mixed_pipeline, golden_11_recursive_fixed_point, golden_15_row_effect_contract}.
- Gated incremental behavior: incremental::adts::{adt_incr_08_alias_union_contraction_updates_residual_witness, adt_incr_09_visibility_edit_invalidates_acquisition_without_shrinking_match_universe}. The first did not publish the expected match product; the second produced identical match resolution before and after the purported visibility edit.
- Intentional child-process failure: support::regressions::fail_fast_policy_panics_only_after_recording_incident_child. Running this child directly is expected to panic; its parent test owns the process-level assertion.

### Fresh stability baseline — 2026-10-16, `ce8c6777`

The post-enable full semantic run is now observed rather than projected:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --skip ordinary_float_horner_expression_is_stack_safe_without_realization --quiet
  2,123 passed; 94 failed; 23 ignored; 1 filtered out; 946.52s
```

The 94 named failures in the preceding inventory reproduced on `ce8c6777`.
The 23 ignored tests were not run in this baseline; their last forced-run
classification remains the earlier recorded evidence. A zero-test exact
filter attempt for the stack test was discarded as non-evidence, then the
fully qualified test was run separately:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::algebraic_realization::ordinary_float_horner_expression_is_stack_safe_without_realization -- --exact --nocapture
  1 selected; analysis thread overflowed its explicit 2 MiB stack; test process aborted with SIGABRT
```

The size-budget failure also reproduced in the full run:
`TypedExpression` is 896 bytes against the test's 760-byte maximum.

Classification from this bounded pass:

- **Confirmed compiler robustness defect:** the Horner expression analysis
  overflows its explicitly bounded 2 MiB worker stack.
- **Confirmed representation-budget regression:** `TypedExpression` exceeds
  the asserted recursive-carrier budget by 136 bytes. Whether the budget or
  representation should change still needs source-history/design review.
- **Unresolved:** the other 93 failures in the 94-test failure set. They span
  semantic result/diagnostic assertions and incremental recomputation/product
  stability assertions. This run proves they reproduce, but does not by itself
  distinguish implementation defects, stale oracles, or unsupported cases.
  No environment-caused failure was identified.

The branch's seven focused functional-acceptance gates were rerun on the same
baseline and passed: canonical ordinary-user execution 1/1; positive
protocol/default execution 1/1; strict cycle rejection 1/1; fail-closed
mapping 2/2; modules linker 25/25; modules graph 5/5; semantic phase evidence
12/12. Those focused results do not classify the broader semantic failures.

### Audited stability baseline — 2026-10-16, `2b0e41ce`

The full semantic test run prior to the mode slice was re-audited:

```text
CARGO_TARGET_DIR=/tmp/phalcom-consolidation-check-20261016 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --skip ordinary_float_horner_expression_is_stack_safe_without_realization --quiet
  2,131 passed; 94 failed; 23 ignored; 1 filtered out; 978.92s
```

All 94 failed tests match the existing failure inventory. The pass count is
8 higher than the prior 2,123 count because `phase_evidence` expanded from 12
to 20 tests. The 23 ignored tests were not executed.

`TypedExpression` carrier size in this build measured 1,072 bytes, exceeding
the 760-byte upper limit (744-byte base plus two-pointer allowance).

## Recursive checker stabilization — 2026-10-17

Starting from `324ee315`, the unchanged Horner regression reproduced a process
abort on its 2 MiB analysis thread. The expression carrier exceeded its 760-byte
budget. `BlockReason::UnavailableSemanticOwner` embedded a full `QueryKey`,
enlarging every `AnalysisStatus`; solver-local terms also occupied every
expression carrier even when absent.

The correction keeps the exceptional owner key and optional solver result in
owned boxes. Exact owner identity, equality, fingerprints, query outcomes, and
single ownership remain unchanged. Publication, contextual realization,
inference constraints, and explanations run in their original order in a
post-descent helper. No evidence or diagnostics are removed. On this arm64 debug
build, the recursive publication frame fell from the recorded 13,568 bytes to
2,688 bytes. The stack allowance and carrier assertion are unchanged.

The semantic library tests also contained a missed call-site update from the
mode slice. The budget-exhaustion test now explicitly supplies Debug to the
updated private API; its assertions are unchanged.

Executed with `CARGO_TARGET_DIR=/tmp/phalcom-consolidation-check-20261016`
and empty `RUSTFLAGS`/`RUSTC_WRAPPER`:

```text
cargo test -p phalcom-semantic --test semantic capabilities::algebraic_realization -- --nocapture
  9 passed; 0 failed; 2240 filtered out; 8.20s
```

This proves the bounded Horner analysis and carrier regressions are repaired,
alongside the existing positive and negative realization cases. It does not
establish arbitrary-depth stack safety or close P4. The 94-failure full-suite
baseline has not been rerun after this correction; its carrier entry is now
resolved by focused evidence.

The new package probe also reproduced an 8 MiB lowering-thread overflow after
semantic analysis succeeded. The native crash report showed repeated
`compile_expr_want_inner` / `compile_bilateral_binary` frames. Binary expressions
now dispatch before the large non-binary lowering match. The existing binary
lowering body was moved without changing contextual-realization precedence,
operand order, lazy Boolean blocks, operator selection, or scratch-slot lifetime.

Additional completed semantic checks:

```text
cargo test -p phalcom-semantic --test semantic incremental::query_ownership -- --nocapture
  7 passed; 0 failed; 2242 filtered out; 2.95s
cargo test -p phalcom-semantic --lib session::publication_tests -- --nocapture
  13 passed; 0 failed; 179 filtered out; 9.19s
```

Production-path regression:

```text
cargo test -p phalcom-core --test core ordinary_float_horner_package_executes_all_coefficients -- --nocapture
  1 passed; 0 failed; 994 filtered out; 128.88s
```

The test uses the production package compiler and full Universe VM and asserts
exactly `2080.0`, the sum of coefficients 1 through 64 at `x = 1`. The original
8 MiB package-test stack is unchanged.

Broader gate: `cargo test -p phalcom-core --test language-corpus -- arithmetic booleans --nocapture`
selected two corpus groups and failed both (89.31s) before fixture execution:
`finalized executable link has no runtime read for class System from universe:concurrency.fiber`.
The retained fixture compiler still uses `ExecutableLinkedProgram::from_conservative_link`
instead of projecting the semantic snapshot's generated requirements. This is
an unresolved linkage blocker; the corpus run supplies no operator-runtime
acceptance evidence. The exact Horner package uses the production finalizer and
passes. Do not relax finalized-read checks to unblock the corpus.
