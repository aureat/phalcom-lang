# Semantic capability baseline ledger

Historical 40-test baseline retained by Plan 1. Four later generic epistemic
tests are tracked separately in `generics.rs`.

## Branches (12)

- `same_type_branch_results_establish_single_result_type`
- `heterogeneous_branch_results_join_into_union`
- `branch_union_validates_common_supertype_without_widening_current_fact`
- `returning_branch_does_not_contribute_value_to_continuing_join`
- `throwing_branch_is_excluded_from_reachable_value_join`
- `same_type_writes_in_both_branches_preserve_flow_type`
- `divergent_branch_assignments_join_current_binding_types`
- `branch_join_preserves_narrow_flow_under_broad_declared_contract`
- `refuted_branch_assignment_does_not_fabricate_declared_flow_fact`
- `branch_local_shadow_does_not_mutate_outer_binding_flow`
- `nested_branch_results_compose_transitively`
- `known_branch_does_not_hide_reachable_unknown_branch_in_formal_analysis`

## Loops and blocks (4)

- `loop_same_type_assignment_preserves_current_type`
- `loop_join_includes_preheader_and_body_types`
- `break_and_continue_preserve_loop_exit_and_backedge_facts`
- `captured_block_write_is_not_applied_until_execution_is_proven`

## Structural (8)

- `nested_tuple_composes_exact_constituent_facts`
- `tuple_supertype_annotation_preserves_specific_product_fact`
- `tuple_component_refutation_preserves_actual_product_fact`
- `branch_product_results_preserve_component_precision`
- `heterogeneous_collection_infers_union_element_type`
- `record_literal_preserves_structural_field_types`
- `tuple_destructuring_establishes_independent_component_bindings`
- `tuple_destructuring_with_broad_contract_keeps_specific_components`

## Dispatch (5)

- `chained_dispatch_preserves_constructor_specialization_without_binding_storage`
- `multiple_hop_call_chain_preserves_each_intermediate_result`
- `wrong_class_instance_dispatch_side_is_not_laundered_into_dynamic_unknown`
- `selector_label_mismatch_is_distinguished_from_argument_type_mismatch`
- `argument_refutation_preserves_independently_known_call_return_type`

## Generics (4)

- `generic_identity_solves_parameter_from_argument_and_specializes_return`
- `generic_pair_solves_two_independent_variables`
- `expected_result_context_constrains_generic_without_merely_overwriting_call_fact`
- `conflicting_generic_constraints_are_refuted_instead_of_using_expected_annotation_as_fact`

## Callables (4)

- `branch_derived_tail_type_is_published_to_unannotated_callable_signature`
- `explicit_broad_return_contract_preserves_narrow_branch_evidence`
- `one_bad_return_branch_is_refuted_without_rewriting_branch_fact`
- `recursive_inference_fails_honestly_without_inventing_unit_or_nominal_type`

## Iteration and advisory (3)

- `custom_iterable_element_type_comes_from_protocol_not_first_generic_argument`
- `constructor_branch_nested_inside_collection_preserves_composed_specific_type`
- `formal_unknown_branch_with_declared_contract_remains_assumed_not_established`

## Later additions (4)

- `assumed_generic_argument_yields_assumed_generic_return`
- `mixed_generic_return_uses_weakest_value_support`
- `independent_fixed_generic_return_stays_established`
- `expected_context_cannot_fabricate_missing_generic_return`

## Complex scenario coverage

These scenarios exercise combinations of capabilities. Ownership follows the
semantic harness taxonomy: local source behavior lives under `capabilities`,
module linking under `integration`, and revision/cache behavior under
`incremental`.

| Scenario | Owner | Status | Test |
| --- | --- | --- | --- |
| Refinement plus abrupt branch | `flow_branches` | RED-CAPABILITY | `refined_branch_with_abrupt_else_publishes_only_normal_value` |
| Loop fixed point plus `break`/`continue` | `flow_loops` | READY | `loop_fixpoint_preserves_mutated_integer_and_abrupt_edges` |
| Closure capture plus nested return | `callable_publication` | READY | `closure_capture_and_non_local_return_keep_outer_summary_separate` |
| Higher-order closure invocation | `higher_order` | GATED | `higher_order_block_call_propagates_captured_result` |
| Constructor plus instance/class `super` | `dispatch_capabilities` | READY | `constructor_super_chain_preserves_instance_and_class_side_results` |
| Field initializer/write/read lifecycle | `fields` | GATED | `field_facts_survive_constructor_and_general_writes` |
| Dynamic spread and reflection boundary | `dynamic_boundaries` | READY | `dynamic_spread_preserves_independent_known_fact`; `reflective_dynamic_pack_stays_conservative_but_keeps_known_fact` |
| Collection rest plus product destructuring | `patterns` | GATED | `collection_and_destructure_facts_preserve_element_shapes` |
| Exported class method through import | `integration::workspace` | READY | `exported_constructor_and_method_feed_importing_client_summary` |
| Dependency edit/remove/re-add lifecycle | `incremental::callable_dependencies` | RED-CAPABILITY | `dependency_edit_remove_readd_recomputes_affected_summary_deterministically` |
