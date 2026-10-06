---
id: LANG007.C4.P4.R1-TEL-1
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
implementation_revision: c2d72fed
consolidated_branch: main
updated: 2026-10-06
---

# TEL-1 generic-pack integration walkthrough — 2026-10-06

Status: Original project-compilation objective: COMPLETE / IMPLEMENTED / FOCUSED_TESTED.
Expanded generic-Pack correctness: IN_PROGRESS / PARTIAL / FOCUSED_TESTED;
the positive GoalQuery call-site regression remains failing.
Source and tests consolidated on local `main` as `c2d72fed`; no push performed.

## Implemented behavior

Telogue Predicate arguments use explicit positional expansion:
Predicate<*(Person, Person)>. The existing binder-aligned generic application
regression verifies empty applications, valid expansion, and rejection of unmarked
pack contributions. No generic domain checks were relaxed.

Domain-native member lookup follows semantic kind for symbolic positional,
labeled, and complete pack binders and composed projections. It still requires
the matching Universe source member and intrinsic evaluator. CompleteTypePack Map
validates its unary type-constructor argument consistently with the other lanes.

A distinct solver-local PackFromPacks term exposes lane variables throughout
canonical/scoped conversion, substitution, expected-type propagation, variable/row
traversal, occurs checks, proof/support propagation, equality, subtype handling,
and materialization. Proper Pack identity is retained rather than converted to Tuple.

The last Relationship constructor failure was a residual projection constraint:
an independently solved subject still appeared as a variable in the original
projection term. Projection equality now applies established substitutions forward
before testing congruence. It does not infer subjects or arguments from projection
equality. The regression checks both constraint orders; existing non-injectivity
regressions still pass. Query.of and the mapped Relationship construction now
compile through the complete Telogue project graph.

## Verification

Completed gates:
- cargo check -p phalcom-semantic, with RUSTFLAGS and RUSTC_WRAPPER cleared.
- foundations::inference: 53 passed.
- foundations::type_members: 12 passed.
- variadic_nominal_application_is_binder_aligned_and_empty_specializes: 1 passed.
- generic_pack_query_constructor_preserves_abstract_field_lane: 1 passed.
- generic_pack_contributions_expand_concrete_types_in_applied_arguments: 1 passed.
- abstract_pack_map_is_valid_in_parameters_and_record_fields: 1 passed.
- impls::bodies: 21 passed, retaining covering-coordinate and Self invariants.
- telogue_project_compiles_from_declared_entry: 1 passed. The strengthened test
  verifies entry main and all eleven imported Telogue module identities in the
  compiled graph, including logical identity_token for physical identity-token.ph.
- Scoped git diff --check passed.

The first graph assertion used a physical kebab name rather than the logical
snake_case identity; that test plumbing error was corrected and the gate rerun.
The earlier bare Relationship constructor probe was reverted.

## Scope and remaining limits

No workspace/release certification or runtime execution gate was requested or run.
Existing deprecated TypeAlias warnings remain. Open scoped Pack remainder inference
and canonical closed Pack element inference are broader follow-up features, outside
this handoff's accepted abstract-lane/project-compilation objective.

Declaration-canonical published signatures and distinct receiver_value_override /
instance_self_override fields were preserved. Hypothesis, adjacent P4 records,
and the supplied historical audit were left unchanged. The preceding turn's own
checkpoint appendix was removed to honor the supplied handoff's document scope.
The parent P4/R1 classifications are not promoted by the original project-gate completion.

## Expanded correctness coverage

The user requested additional useful correctness tests. Ten regressions were
added: seven solver regressions, two source regressions, and one incremental
regression. Nine pass; one intentionally retains the valid-source acceptance
oracle and currently fails. It is marked RED-CAPABILITY, without ignore/skip.

New solver coverage verifies empty, positional-only, labeled-only, and mixed
Packs; label/value order; exact materialization and contextual expectations;
Pack versus Tuple equality/subtyping rejection in both directions; lane-kind
mismatch; recursive element occurs checks; weakest proof retention in either
lane; constructor-argument congruence after independent selection; and residual
obligations for distinct constructors. foundations::inference now passes 60 tests.

Source constructor coverage tests all four declaration-field / initializer-field
order permutations using a union-valued Term-like mapping. It passes.

The incremental regression changes Wrap<T> from List<T> to Option<T>. It applies
the actual Map constructor to Int (the abstract formatter hides lambda bodies),
proves the contract changed, compares warm and cold results, and checks that an
unrelated callable signature fingerprint is unchanged. It passes.

The new GoalQuery call-site regression fails with GenericInferenceUnderconstrained
for both GoalQuery.of() and GoalQuery.of(1, "two") inside QueryProbe.run. The
abstract constructor body passes, but ordinary generic rest call inference is
not complete. This is a real remaining acceptance gap; passing the Telogue
manifest project gate does not prove these call-site cases. Expanded generic-Pack
correctness is IN_PROGRESS / PARTIAL / FOCUSED_TESTED, with this failed gate.

capabilities::generics completed with 23 passed and 3 failed. Besides the new
GoalQuery regression, universe_future_preserves_payload_types_through_async_map_then_and_await
and unsaturated_generic_constructors_are_cleanly_rejected_in_proper_type_positions
failed. These are unclassified (C), not established baseline failures. No
assertions were weakened, tests skipped, or production fixes attempted during
this additional test-only slice. Scoped diff whitespace check passed.

## Consolidation record

All TEL-1 source changes, passing coverage, and the unignored RED-CAPABILITY
regression are preserved in `c2d72fed` on `main`. The evidence above is inherited
from the completed TEL-1 runs; no tests were rerun during Git consolidation.
The passing declared-entry project gate does not close expanded generic-rest
call-site correctness, open scoped Pack remainders, closed canonical element
inference, runtime execution, or release certification. The C4 checkpoint and
R1 issue register now reflect this distinction.
