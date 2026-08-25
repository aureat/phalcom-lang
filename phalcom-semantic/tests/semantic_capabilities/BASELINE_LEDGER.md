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
