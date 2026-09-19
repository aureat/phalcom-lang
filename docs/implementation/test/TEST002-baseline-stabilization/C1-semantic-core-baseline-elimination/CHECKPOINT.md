---
id: TEST002.C1
category: TEST
program: TEST002
checkpoint: TEST002.C1
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
requires:
  - LANG006.C1
---

# TEST002.C1 — Semantic/core baseline elimination

## Objective and ownership boundary

This checkpoint is a bounded corrective stabilization objective. It eliminates
the nineteen residual semantic/core failures recorded after LANG006.C1, either
by repairing the owning implementation or by migrating a test whose oracle
still encodes retired behavior. Correct semantic assertions are not weakened.

The checkpoint does not own new visibility syntax, subscript syntax redesign,
runtime trait proof, cursor compatibility, iterator-library redesign, or
workspace/release certification.

## Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
| --- | --- | --- | --- | --- | --- |
| TEST002.C1.P1 | Post-visibility semantic/core baseline failure elimination | IN_PROGRESS | PARTIAL | FOCUSED_TESTED | T0 baseline established; T1–T4 semantic corrections complete |

## Starting state

| Field | Value |
| --- | --- |
| Starting revision | `b67fda93ad9c7c7bed0cedd46bbe002e5147bd46` (`main`, `origin/main`) |
| Predecessor semantic baseline | 1,428 passed / 14 failed / 50 ignored |
| Predecessor core baseline | 607 passed / 5 failed / 28 ignored |
| T0 production edits | none |
| T0 result | all BL-01..BL-19 reproduced on the live revision |
| Active plan | TEST002.C1.P1 |

## Failure ledger at T0

| ID | Failure | Owner | T0 result | Planned disposition |
| --- | --- | --- | --- | --- |
| BL-01 | `opaque_call_drops_refined_field_current_without_rewriting_contract` | SEMA flow | FAIL: expected `Unknown(NoTypeEvidence)`, got validated `Object` lifecycle evidence | migrate stale oracle |
| BL-02 | `unknown_expansions_do_not_disappear` | SEMA composition | FAIL: unresolved-name cause became `SuppressedByInvalidCause` | repair |
| BL-03 | `assumed_list_receiver_caps_structural_index_result` | SEMA calls | FAIL: missing `Assumed` status | repair |
| BL-04 | `list_subscript_checks_index_contract` | SEMA calls | FAIL: missing result type/diagnostic | repair |
| BL-05 | `list_subscript_set_checks_index_and_value_and_returns_unit` | SEMA calls | FAIL: zero mismatches instead of two | repair |
| BL-06 | `map_subscript_checks_key_contract` | SEMA calls | FAIL: missing result type/diagnostic | repair |
| BL-07 | `adding_impl_member_does_not_invalidate_another_target_or_its_caller` | SEMA incremental | FAIL: unchanged caller `Arc` not retained | repair |
| BL-08 | `dependency_edit_remove_readd_recomputes_affected_summary_deterministically` | SEMA incremental | FAIL: signature edit did not invalidate caller as expected | verify/repair; oracle retained |
| BL-09 | `canonical_universe_reads_track_canonical_query_dependencies` | SEMA dependencies | FAIL: declaration-shell dependency absent | repair |
| BL-10 | `legacy_universe_core_path_is_not_a_dependency_sentinel` | SEMA dependencies | FAIL: legacy-looking Universe path suppressed | repair |
| BL-11 | `signature_edit_recomputes_exact_callers_and_reuses_unrelated_bodies` | SEMA incremental | FAIL: unrelated product revision advanced | repair |
| BL-12 | `superclass_edit_recomputes_hierarchy_consumers_without_touching_unrelated_bodies` | SEMA incremental | FAIL: unrelated product revision advanced | repair |
| BL-13 | `a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products` | SEMA incremental | FAIL: unrelated product revision advanced | repair |
| BL-14 | `unused_public_export_does_not_recompute_unrelated_consumer_body` | SEMA incremental | FAIL: unrelated consumer revision advanced | repair |
| BL-15 | `positional_spread_of_non_iterable_has_structured_boundary` | compiler/runtime iteration | FAIL: leaked `doesNotUnderstand('iterator')` | repair |
| BL-16 | `test_list_spread_traceback_includes_spread_site` | core observability | FAIL: stale cursor fixture hit `doesNotUnderstand('iterator')` | migrate stale fixture |
| BL-17 | `cv46_runtime_conditional_default_executes` | generic conformance runtime | FAIL: no active conformance target for `Printable.print` | repair |
| BL-18 | `bounded_generic_function_dispatches_exact_witness_and_default` | generic conformance runtime | FAIL: no active conformance target for `Printable.print` | repair |
| BL-19 | `traverse_body_resolves_symbolic_inherited_capabilities` | typing integration | FAIL: `Functor.map` blocked by `NoTypeEvidence` | repair |

## Established invariants

- LANG006.C1 visibility remains canonical: default visibility is public, access
  is exact and lookup precedes authorization.
- Opaque calls drop narrow field refinements but preserve a valid initialized
  field lifecycle contract.
- Semantic evidence, iteration proof, generic conformance selection, and
  incremental product reuse each retain their existing canonical owner.
- Only the aggregate Universe root is bootstrap dependency input; concrete
  Universe source modules remain fine-grained semantic dependencies.

## Verification ledger

| Gate | Scope | Result |
| --- | --- | --- |
| T0 | nineteen exact/narrow regression filters | COMPLETE: all nineteen reproduced; no production edits |
| T1/G1 | stale field-flow oracle and cursor-era traceback fixture | PASS: both tests now express current lifecycle/Iterator semantics |
| G2 | expression composition and canonical call application | PASS: 21 composition tests and 35 canonical call tests |
| T4 | canonical/legacy-looking Universe dependency ownership | PASS: both exact dependency tests |

## Deferred / baseline issues

BL-01 and BL-16 are resolved by test-authority migration; no production
behavior workaround was added. The remaining BL-02..BL-15 and BL-17..BL-19
entries remain active TEST002 work. Any newly discovered unrelated
failure will be classified under the repository baseline policy before being
recorded here.

## Active next action

Execute T5: restore exact incremental product reuse and propagation stopping.
