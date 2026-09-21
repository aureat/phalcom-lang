---
id: TEST002.C1
category: TEST
program: TEST002
checkpoint: TEST002.C1
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
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
| TEST002.C1.P1 | Post-visibility baseline remediation through runtime conformance defaults | COMPLETE | PARTIAL | FOCUSED_TESTED | BL-01..BL-18 resolved/migrated; BL-19 blocked before its planned trace by generic override compatibility |
| TEST002.C1.P2 | Generic override compatibility and symbolic inheritance closure | PROPOSED | NOT_STARTED | UNVERIFIED | owns the prerequisite decision/repair required before BL-19 and final certification |

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
- Incremental scheduling does not seed reverse invalidation from aggregate
  linked-interface changes; import-preamble changes reopen only the exact
  affected declaration headers, and non-source-owned Universe products remain
  published while their dependencies validate.
- Positional expansion lowering consumes either the semantic Iterable/Iterator
  requirement product or a semantic known-invalid boundary. The Tuple/Unit
  product lane remains independent, and canonical lowering has no ordinary
  `iterator`/`next` eligibility fallback.

## Verification ledger

| Gate | Scope | Result |
| --- | --- | --- |
| T0 | nineteen exact/narrow regression filters | COMPLETE: all nineteen reproduced; no production edits |
| T1/G1 | stale field-flow oracle and cursor-era traceback fixture | PASS: both tests now express current lifecycle/Iterator semantics |
| G2 | expression composition and canonical call application | PASS: 21 composition tests and 35 canonical call tests |
| T4 | canonical/legacy-looking Universe dependency ownership | PASS: both exact dependency tests |
| T5/G3 | incremental ownership, reuse, exact invalidation, and cold equivalence | PASS: checker dependencies 10, callable dependencies 12, product stability 16, recomputation metrics 13, impl queries 55 |
| T6/G4-prep | positional expansion boundary and canonical iteration lowering | PASS: semantic iteration 6, canonical call 35, canonical invalid spread 1, canonical Iterable spread 1, outgoing-pack group 28, traceback spread 1 |
| T7 | exact/derived default activation environments | PASS: BL-18, BL-17, cv47, and cv48; selected defaults receive environments indexed by their compiled requirement slots |
| Override-validator boundary | user override probes and canonical bootstrap | PASS: incompatible and widening user probes pass; canonical provider-provenance support declarations no longer block bootstrap |
| P2 / generic override compatibility | callable-binder alpha equivalence, HKT specialization, constraints, variance, and monad integration | PASS: callable-local binders normalize after canonical ancestor specialization; 35 monad integration tests, 10 HKT tests, 3 constraint tests, and 4 variance tests pass |

## Deferred / baseline issues

BL-01 and BL-16 are resolved by test-authority migration. BL-15 is resolved
through the semantic known-invalid lowering product, the structured runtime
boundary, and direct compatibility coverage. BL-02..BL-19 are resolved.

**BL-19 — resolved by TEST002.C1.P2.** Generic override validation now
alpha-aligns corresponding callable-owned binders only after canonical
receiver-to-ancestor specialization. It preserves owner-relative declaration
parameters, `Self`, selector shape, kind/variance metadata, normalized generic
constraints, parameter contravariance, and result covariance. The authoritative
`traverse_body_resolves_symbolic_inherited_capabilities` discriminator and the
complete 35-test monad integration package pass.

The context-free interpreter path remains an
explicit untyped compatibility boundary because it has no semantic snapshot;
canonical analyzed modules do not use that fallback. Any newly discovered
unrelated failure will be classified under the repository baseline policy
before being recorded here.

## Consultation / amendment ledger

| Incident | Plan/task | Trigger | Evidence | Decision needed |
| --- | --- | --- | --- | --- |
| T7 derived default environment | TEST002.C1.P1 / T7 | F6 remained after its one serious correction | BL-18 now passes with a nested environment indexed by the trait default body’s global requirement slot. BL-17 initially entered `Printable.label` without its required `print` slot because `DerivedConditional` omitted nested slots for selected trait defaults. | **Adopted:** build the derived default’s slot-indexed nested environment from `DerivedConformanceEvidence::requirements()`, while preserving antecedent nested-evidence handling for non-default derived witnesses. BL-17/18 and cv47/cv48 pass. |
| T8 generic override blocker | TEST002.C1.P1 / T8 | Plan premise disproved; generic override compatibility is an unexpected semantic-authority boundary | The canonical-provider provenance exclusion preserves the override validator for synthetic fixtures. The requested migration of the monad fixture from `Universe` to `Synthetic` still emits the same four `IncompatibleOverride` diagnostics before BL-19 runs. | **Adopted:** split to P2. Preserve the BL-19 oracle and validator coverage; P2 must specify/repair generic override compatibility before resuming the symbolic evidence trace. |
| P2 generic override repair | TEST002.C1.P2 / T1–T3 | Callable-owned generic binder identity caused alpha-equivalent signatures to fail canonical subtype checks | `EitherMonad<E>` and inherited `Monad<<X> =>> Either<E, X>>` signatures had equivalent declaration-specialized forms but distinct callable parameter owners. | **Implemented:** substitute candidate callable binders with corresponding inherited binders after canonical specialization, require equal kind/variance metadata and normalized constraints, then retain existing relation checks. BL-19 and all targeted monad/HKT/constraint/variance coverage pass. |
| Corpus stabilization follow-up | post-TEST002 corpus audit | Source-file compilation omitted source-authored Universe conformance modules, so canonical `for` lowering had no `Iterable` lane in corpus modules | A standalone-module regression for `for value in [1, 2]` failed before projection; inline compilation passed. Loading the relevant Universe source closure and linking all supplied interfaces restores the published lane. | **Implemented:** standalone compilation now includes collection/scalar source conformance modules and uses the complete linked source universe before semantic analysis. The compiler continues to consume the semantic lane only. |
| Untyped empty iteration / chained generic factory | post-TEST002 corpus audit | `for value in []` lacks an item-type proof; `Map.new().at(key, put: value)` reports inner-call underconstraint before the outer receiver call can constrain `K, V` | Empty list synthesis intentionally produces `Unknown(NoTypeEvidence)` absent `List<T>` context; no published lane is therefore correct. Nested receiver calls currently analyze `Map.new()` under `ExpectedType::None`, emitting `GenericInferenceUnderconstrained` before `at` arguments are available. | **Escalated:** do not manufacture `Iterable` or `Dynamic` fallback evidence. Empty-literal typing requires a specification decision; nested generic receiver inference requires a shared/delayed inference-frame design. |

The unrelated override-certification change initially blocked all core tests: the
new validator emitted false-positive diagnostics for canonical Universe support
sources. By user decision, its eligibility is now restricted to canonical
provider provenance rather than merely a Universe-looking module identity;
synthetic fixtures retain validator coverage. The wider question of aligning
existing Universe/native contracts with ordinary override compatibility is
deferred outside TEST002.

That provenance boundary leaves a second, distinct scope conflict: the existing
T8 monad typing fixture intentionally analyzes a synthetic-source program under
the legacy `ModuleId::universe_root()` test identity. The new validator now
rejects its pre-existing generic `Monad`/`Functor` overrides before BL-19 can
reach the planned symbolic-evidence discriminator. By user direction, TEST002
tried migrating that fixture to a real synthetic module identity; the same four
override diagnostics remained. The migration was reverted. The evidence now
shows generic override compatibility—not source provenance—is the blocker.
Repairing it is a separate language-design task, and broadening the exclusion
would suppress ordinary synthetic-fixture coverage.

## Active next action

Run the checkpoint's remaining G4/G5 certification targets and update the
checkpoint/program records from their completed evidence. P2 is complete as
`IMPLEMENTED` + `FOCUSED_TESTED`; its broad certification remains deferred to
the checkpoint gate.
