# Phalcom semantic capability torture suite

This package contains **40 semantic specification probes** for `phalcom-semantic`.
They are deliberately written against the intended semantic model, not against
whatever behavior happens to pass today. Some tests are therefore expected to
fail until the corresponding checker capability is implemented or repaired.

## One Cargo integration-test binary

The layout is intentional:

```text
tests/
  semantic_capabilities.rs              # the ONLY Cargo integration target
  semantic_capabilities/
    support.rs                           # shared fixture/query/assertion helpers
    branches.rs                         # 12 tests
    loops_blocks.rs                     # 4 tests
    structural.rs                       # 8 tests
    dispatch.rs                         # 5 tests
    generics.rs                         # 4 tests
    callables.rs                        # 4 tests
    iteration_advisory.rs               # 3 tests
```

`tests/semantic_capabilities.rs` uses `#[path = ...] mod ...;` to load all
category files as ordinary modules. Cargo therefore compiles the suite as one
binary (`--test semantic_capabilities`) rather than treating every category
file as a separate integration target.

## Install

Extract/copy the `tests/` directory into the `phalcom-semantic` crate root.
No `Cargo.toml` change is required.

## Run

For diagnosis, run tests serially so failure output stays associated with the
correct test:

```bash
cargo test --test semantic_capabilities -- --test-threads=1 --nocapture
```

To run one category by module path, for example branches:

```bash
cargo test --test semantic_capabilities branches:: -- --test-threads=1 --nocapture
```

To run one probe:

```bash
cargo test --test semantic_capabilities \
  branches::divergent_branch_assignments_join_current_binding_types \
  -- --exact --nocapture
```

## What the suite is testing

### Branch and path composition — 12

1. `same_type_branch_results_establish_single_result_type`
2. `heterogeneous_branch_results_join_into_union`
3. `branch_union_validates_common_supertype_without_widening_current_fact`
4. `returning_branch_does_not_contribute_value_to_continuing_join`
5. `throwing_branch_is_excluded_from_reachable_value_join`
6. `same_type_writes_in_both_branches_preserve_flow_type`
7. `divergent_branch_assignments_join_current_binding_types`
8. `branch_join_preserves_narrow_flow_under_broad_declared_contract`
9. `refuted_branch_assignment_does_not_fabricate_declared_flow_fact`
10. `branch_local_shadow_does_not_mutate_outer_binding_flow`
11. `nested_branch_results_compose_transitively`
12. `known_branch_does_not_hide_reachable_unknown_branch_in_formal_analysis`

These probe whether desugared `if` semantics actually participate in formal
flow analysis: reachability, captured writes, joins, lexical identity, unions,
and the declaration/current-knowledge distinction.

### Loops and blocks — 4

13. `loop_same_type_assignment_preserves_current_type`
14. `loop_join_includes_preheader_and_body_types`
15. `break_and_continue_preserve_loop_exit_and_backedge_facts`
16. `captured_block_write_is_not_applied_until_execution_is_proven`

These probe loop fixed points/widening, zero-iteration paths, loop exits and
backedges, and the distinction between constructing a block and proving that
its captured effects execute.

### Structural/product inference and patterns — 8

17. `nested_tuple_composes_exact_constituent_facts`
18. `tuple_supertype_annotation_preserves_specific_product_fact`
19. `tuple_component_refutation_preserves_actual_product_fact`
20. `branch_product_results_preserve_component_precision`
21. `heterogeneous_collection_infers_union_element_type`
22. `record_literal_preserves_structural_field_types`
23. `tuple_destructuring_establishes_independent_component_bindings`
24. `tuple_destructuring_with_broad_contract_keeps_specific_components`

These test recursive product synthesis, structural assignability, precision
under broad contracts, list element unions, record typing, and recursive
pattern binding.

### Dispatch and call correctness — 5

25. `chained_dispatch_preserves_constructor_specialization_without_binding_storage`
26. `multiple_hop_call_chain_preserves_each_intermediate_result`
27. `wrong_class_instance_dispatch_side_is_not_laundered_into_dynamic_unknown`
28. `selector_label_mismatch_is_distinguished_from_argument_type_mismatch`
29. `argument_refutation_preserves_independently_known_call_return_type`

These separate dispatch correctness from binding-state bugs and test exact
selector shape, dispatch side, `Self` specialization, multi-hop result
propagation, and recovery from bad arguments.

### Generic inference — 4

30. `generic_identity_solves_parameter_from_argument_and_specializes_return`
31. `generic_pair_solves_two_independent_variables`
32. `expected_result_context_constrains_generic_without_merely_overwriting_call_fact`
33. `conflicting_generic_constraints_are_refuted_instead_of_using_expected_annotation_as_fact`

These test solver substitutions, independent variables, bidirectional expected
context, and conflict handling. The critical distinction is whether the call
expression itself becomes established, versus a later binding annotation merely
masking incomplete inference.

### Callable return inference and recursion — 4

34. `branch_derived_tail_type_is_published_to_unannotated_callable_signature`
35. `explicit_broad_return_contract_preserves_narrow_branch_evidence`
36. `one_bad_return_branch_is_refuted_without_rewriting_branch_fact`
37. `recursive_inference_fails_honestly_without_inventing_unit_or_nominal_type`

These test the boundary between body-local facts and published callable
signatures, declaration-versus-implementation evidence, branch return checking,
and recursive fixed-point honesty.

### Iteration, composition, and future advisory boundary — 3

38. `custom_iterable_element_type_comes_from_protocol_not_first_generic_argument`
39. `constructor_branch_nested_inside_collection_preserves_composed_specific_type`
40. `formal_unknown_branch_with_declared_contract_remains_assumed_not_established`

The first directly guards against the current generic-argument fallback in
iteration typing. The last is the formal half of the future formal/advisory
composition law: missing formal evidence may permit an explicit declaration to
supply an assumption, but must never become established proof merely because a
contract exists.

## Shared helper philosophy

`support.rs` deliberately queries published semantic products rather than only
checking `snapshot.has_errors()`. Tests can inspect:

- exact `ExpressionAnalysis` knowledge and `EvidenceStatus`;
- binding `declared`, `contract`, `current`, and `consistency` state;
- canonical `TypeData` unions, tuples, records, and generic applications;
- subtype relationships;
- diagnostics by `DiagnosticCode`;
- callable identities and body analyses;
- source-exact expression sites.

The goal is to expose the **path to the semantic answer**, not only the final
absence or presence of an error.

## Expected failures are useful

A red probe should be classified before it is fixed. Typical classes are:

- parser/source fixture mismatch;
- AST construct is accepted but formal analyzer does not cover it;
- exact synthesis works but branch/flow composition loses it;
- relation is correct but binding reconciliation loses precision;
- callable body establishes a result but the canonical signature does not
  publish it;
- exact dispatch fails before argument/type checking;
- generic solver loses or fabricates a constraint;
- recovery/diagnostic policy rewrites semantic truth;
- deliberate `Unknown` is being confused with checker incompleteness.

When a probe fails, inspect the expression/binding product first. Do not make a
passing LSP presentation test the first proof of a compiler semantic property.

## Verification note

The package was statically assembled against the current public repository API
visible while it was generated, including the newer `EvidenceStatus` /
`EvidenceOrigin` and binding-contract model. The execution environment used to
produce this archive did not contain a Rust toolchain or a local checkout, so
`cargo test --no-run` could not be executed here. The suite is intended to be
run immediately in the Phalcom checkout; parser/API mismatches, if any, should
be treated separately from semantic assertion failures.
