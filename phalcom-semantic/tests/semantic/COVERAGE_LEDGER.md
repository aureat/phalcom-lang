# Plan 2 source-semantic coverage ledger

This ledger tracks the 106 laws from Plan 2. A slot is marked `READY` only
when an enabled source-level test proves the law at its required depth. Foundation
and database tests remain valuable, but do not satisfy source slots by themselves.

Status vocabulary: `READY`, `RED-CAPABILITY`, `STAGED`, `GATED`.

Current ledger count after semantic capability gap closure: **56 READY, 16 STAGED, 34 GATED**.
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
| G08 | READY | `generics::expected_context_selects_but_does_not_establish_result_only_generic` |
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
