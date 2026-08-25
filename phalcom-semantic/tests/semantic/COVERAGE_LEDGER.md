# Plan 2 source-semantic coverage ledger

This ledger tracks the 106 laws from Plan 2. A slot is marked `READY` only
when an enabled source-level test proves the law at its required depth. Foundation
and database tests remain valuable, but do not satisfy source slots by themselves.

Status vocabulary: `READY`, `RED-CAPABILITY`, `STAGED`, `GATED`.

Current ledger count after the corrective pass: **54 READY, 17 STAGED, 35 GATED**.
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
| D06 | STAGED | Nested `Box<Self>` source return publication and Self substitution |
| D07 | READY | `dispatch_capabilities::selector_label_mismatch...` |
| D08 | READY | `dispatch_capabilities::argument_refutation_preserves...` |
| F01 | GATED | Formal Family capture API and source-level getter capture |
| F02 | GATED | Formal Family slot/label publication |
| F03 | GATED | Pattern Family capture semantics |
| F04 | GATED | Formal Family invocation dispatch integration |
| F05 | GATED | Family value storage through source binding |
| F06 | GATED | Formal instance/class Family distinction |
| F07 | GATED | Family hierarchy dependency publication |
| F08 | GATED | Wrong-shape Family call diagnostics |
| F09 | GATED | Generic/Self Family specialization |
| G01 | STAGED | Source generic owner identity oracle for class and method parameters |
| G02 | READY | `generics::generic_identity_solves...` |
| G03 | READY | `generics::generic_pair_solves...` |
| G04 | STAGED | Source receiver generic application to member return |
| G05 | STAGED | Generic receiver and argument constraints across nested source applications |
| G06 | READY | `generics::expected_result_context_constrains...` |
| G07 | READY | `generics::conflicting_generic_constraints...` |
| G08 | READY | `generics::expected_context_cannot_fabricate...` |
| G09 | STAGED | Multi-hop specialized generic callable publication |
| C01 | GATED | Source `where T <: Number` checking and bound diagnostics |
| C02 | GATED | Method-owned source bound invocation checking |
| C03 | GATED | Conjunctive source constraint solver |
| C04 | GATED | Source equivalence constraint syntax and solver |
| C05 | GATED | Coexisting class/method constraint owners |
| C06 | GATED | Generic superclass constraint substitution |
| C07 | GATED | Distinct bound-violation diagnostic |
| K01 | STAGED | Source-level proper-type/kind publication oracle |
| K02 | STAGED | Source constructor kind publication |
| K03 | STAGED | Multi-parameter constructor kind publication |
| K04 | GATED | Wrong-kinded annotation diagnostic |
| K05 | GATED | Type-lambda source syntax and lowering |
| K06 | GATED | Type-lambda alpha normalization |
| K07 | GATED | Type-lambda beta reduction |
| K08 | GATED | Nested type-lambda binder scoping |
| K09 | GATED | Type-lambda arity/kind diagnostics |
| V01 | GATED | Source variance declaration and relation checking |
| V02 | GATED | Source contravariant relation checking |
| V03 | GATED | Source invariant relation checking |
| V04 | STAGED | Variance-aware binding contract relation |
| V05 | GATED | Generic superclass variance substitution |
| V06 | GATED | Nested variance in callable occurrences |
| S01 | READY | `structural::nested_tuple_composes...` |
| S02 | STAGED | Canonical labeled tuple source structure |
| S03 | READY | `structural::record_literal_preserves_structural_field_types` asserts exact closed fields |
| S04 | READY | `structural::heterogeneous_collection...` |
| S05 | READY | `patterns::nested_tuple_pattern_recursively_establishes_each_leaf` |
| S06 | GATED | List/rest pattern semantic leaves |
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
| A01 | STAGED | Source field declaration/read authority |
| A02 | STAGED | Generic receiver field specialization |
| A03 | GATED | Inherited Self-typed field source API |
| A04 | GATED | Source alias declaration and provenance |
| A05 | GATED | Generic/nested alias normalization |
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

`BodyExitFacts` trace assertions for nested `return`/`throw` remain a separate
product-hardening prerequisite: the current published product does not yet
represent those abrupt paths with sufficient fidelity, so this ledger does not
claim that deeper oracle merely because the branch value tests are green.
