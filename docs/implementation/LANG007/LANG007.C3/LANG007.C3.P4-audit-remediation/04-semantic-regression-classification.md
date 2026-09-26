# LANG007.C3.P4 semantic failure classification

Date: 2026-09-27
Authority: `c3-p4-final-audit.md` §§19–24 and Priority 3; handoff §7.

## Evidence boundary

The current run is at `9e58d3b77928d410da2875570a413f567168db7a` (`main`):

```text
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic --test semantic
rustc 1.99.0-nightly (af3d95584 2026-07-09)
1711 selected; 1629 passed; 40 failed; 42 ignored; exit 101
```

The immediate clean pre-P4 source predecessor is
`c38625360ed0b78da22cdb30f01469b3b1f5baf1`, direct parent of the first P4
semantic change `0d6a6e9bbf74c82e5c3587eb4153fde589cbedd5`. A `git archive`
snapshot at `/tmp/phalcom-p4-c386-baseline.kqlbHJ` was tested without switching
or creating a branch/worktree:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
rustc 1.99.0-nightly (af3d95584 2026-07-09)
1693 selected; 1609 passed; 42 failed; 42 ignored; exit 101
```

Every one of the current 40 selectors was present and failed at c386. The two
additional c386 failures—`generic_associated_constraints::where_clause_kind_fact_unifies_with_generic_parameter`
and `foundations::inference::p3_complete_pack_projection_reduces_after_subject_solution`—pass
on current main. The 42 ignored tests in each run are neither failures nor
coverage. The earlier current-main run under a directory-overridden August
toolchain is supplemental only and is not used in this comparison.

These current-main failure results are the pre-remediation baseline at
`9e58d3b`. Two exact selectors have since passed after the alias migration and
invalid-header publication fix, as recorded in their post-fix cells. The full
semantic target has not been rerun after those edits.

Classification rule: a source-backed earlier PASS plus current FAIL is
`CURRENT REGRESSION`; note when it already failed at c386, since that means the
regression predates P4. A test with no known-good PASS and a c386 FAIL is
`TRUE CLEAN-PREDECESSOR BASELINE`. The variadic alias case is
`INTENTIONAL P4 MIGRATION` because it now fails specifically on the ratified
explicit pack-contribution syntax. Historical PASS evidence is recorded
separately from the immediate predecessor result; it does not erase the c386
failure. The 35 earlier-green failures below were already present at the
immediate P4 predecessor, so they are not attributed to P4 changes.

Historical evidence keys:

- **E1** — `docs/audit/type-system-2026-09-09/evidence/semantic-baseline.txt`,
  1180-test recorded PASS output, committed at `9fb98f60` (line per row).
- **E2** — `LANG007.C2-CHECKPOINT.md:117`, alias PASS, recorded with C2.P2
  commit `7da5b043`.
- **E3** — `LANG007.C2.P3-walkthrough.md:106-114`, pack-mode/incremental PASS,
  C2.P3 closure `62a6082a`.
- **E4** — `CONC002.C1-CHECKPOINT.md:189`, 19-test generics PASS includes the
  exact generic-return selector; recorded in `ecc08066`.
- **E5** — `LANG005.C5-CHECKPOINT.md:1064` and
  `LANG007.C1.P3-handoff.md:55-70`, prior trait-default baseline failures
  (commits `e3e38137` and `1dfc9dc4`).
- **E6** — `c3-p4-final-audit.md` §§19–24, committed at `f98b3810`, calls out
  earlier green evidence for the alias, generic identity, flow, editor, and
  workspace cases. E1/E2/E3/E4 provide the more specific run records where
  available.

The listed failure symptoms are observations from test assertions/output, not
causal findings. `not yet isolated` means no implementation cause or fix is
claimed. This is the audit's Priority 3 classification performed as
remediation. Although P4 T13.1 later includes failure classification, T13's
preconditions T1–T12 are not met; this record does not satisfy or complete T13.
The P4 task column names a feature task only where direct ownership is
evidenced; otherwise feature ownership remains unassigned pending isolation.
The post-fix results currently cover the explicit-alias migration and invalid
generic-header publication fix recorded below; no broad post-fix gate has run.

## Per-selector classification

`FAIL / FAIL` means current pinned run / c386 pinned run. “P4 task ownership”
names a directly evidenced feature task, if one is established; classification
itself belongs to the audit's Priority 3 remediation, not P4 T13.

| test | current result | known/predecessor result | classification | P4 task ownership | root cause | fix/migration | post-fix result |
|---|---|---|---|---|---|---|---|
| `export_invalidation::test_reexport_retargeting_invalidates_exact_consumer` | FAIL | c386 FAIL; earlier PASS E1:11 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Observed stats show 0 callables recomputed and 1 reused after retarget; exact causal edge not yet isolated. | Trace consumer dependency/fingerprint publication; no fix selected. | NOT RUN |
| `semantic::capabilities::aliases::variadic_callable_alias_lowers_and_beta_reduces` | FAIL | c386 FAIL; C2 PASS E2 | INTENTIONAL P4 MIGRATION | Audit Priority 3 classification; T2 explicit expansion contract and T11 fixture migration | The former `Fn<Int, String>` supplied direct contributions to variadic `*Ps`; the diagnostic requires explicit pack expansion. | Migrate only this positive fixture to `Fn<*(Int, String)>`; assertions unchanged. | PASS: 1 selected, 1 passed with pinned toolchain exact selector |
| `semantic::capabilities::authority::compatible_supertype_annotation_is_proven_but_narrow_value_knowledge_is_preserved` | FAIL | c386 FAIL; earlier PASS E1:342 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Assertion observes different type identities where the narrower established fact should remain; causal path not yet isolated. | Trace annotation constraint versus established-value publication; no fix selected. | NOT RUN |
| `semantic::capabilities::authority::ordinary_inherited_class_return_is_not_specialized_like_constructor_self` | FAIL | c386 FAIL; earlier PASS E1:355 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Assertion expected a specialized type and observed `None`; causal path not yet isolated. | Trace inherited callable signature/result specialization; no fix selected. | NOT RUN |
| `semantic::capabilities::authority::refuted_annotation_cannot_override_proven_constructor_type_or_downstream_dispatch` | FAIL | c386 FAIL; earlier PASS E1:357 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Assertion observes the wrong type identity after a refuted annotation; causal path not yet isolated. | Trace constructor proof and annotation-refutation publication; no fix selected. | NOT RUN |
| `semantic::capabilities::dispatch::capability_dispatch::constructor_super_chain_preserves_instance_and_class_side_results` | FAIL | c386 FAIL; earlier PASS E1:411 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Binding assertion reports `Unknown(NoTypeEvidence)` where a fact was expected; exact transition not yet isolated. | Trace constructor/super call product and subsequent binding state; no fix selected. | NOT RUN |
| `semantic::capabilities::dispatch::capability_dispatch::invalid_dispatch_argument_does_not_suppress_sibling_dispatch` | FAIL | c386 FAIL; earlier PASS E1:412 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Expected sibling product is absent; causal path not yet isolated. | Trace invalid-argument diagnostic isolation from sibling dispatch; no fix selected. | NOT RUN |
| `semantic::capabilities::dispatch::capability_dispatch::selector_label_mismatch_is_distinguished_from_argument_type_mismatch` | FAIL | c386 FAIL; earlier PASS E1:416 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Fixture binding assertion fails; exact selector-versus-argument diagnostic mismatch not yet isolated. | Trace structured selector identity and argument checking; no fix selected. | NOT RUN |
| `semantic::capabilities::dispatch::capability_dispatch::wrong_class_instance_dispatch_side_is_not_laundered_into_dynamic_unknown` | FAIL | c386 FAIL; earlier PASS E1:418 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Test’s wrong-side dispatch oracle fails; exact result/diagnostic not yet isolated. | Trace class/instance side selection and invalidity publication; no fix selected. | NOT RUN |
| `semantic::capabilities::dynamic_boundaries::dynamic_spread_preserves_independent_known_fact` | FAIL | c386 FAIL; earlier PASS E1:421 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Independent known-fact assertion fails; invalidation site not yet isolated. | Trace spread effect and independent fact ownership; no fix selected. | NOT RUN |
| `semantic::capabilities::dynamic_boundaries::reflective_dynamic_pack_stays_conservative_but_keeps_known_fact` | FAIL | c386 FAIL; earlier PASS E1:420 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Known-fact assertion fails; exact reflective boundary not yet isolated. | Trace reflection result and independent fact preservation; no fix selected. | NOT RUN |
| `semantic::capabilities::fields::field_facts_survive_constructor_and_general_writes` | FAIL | c386 FAIL; earlier PASS E1:427 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | The test’s established field-fact oracle fails; exact write transition not yet isolated. | Trace field current-state invalidation separately from its persistent contract; no fix selected. | NOT RUN |
| `semantic::capabilities::flow_branches::contradictory_branch_does_not_publish_bindings_or_diagnostics` | FAIL | c386 FAIL; earlier PASS E1:448 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Branch/pruning publication oracle fails; exact join transition not yet isolated. | Trace contradictory branch transfer and join; no fix selected. | NOT RUN |
| `semantic::capabilities::flow_branches::if_not_true_and_if_not_false_prune_correct_branches` | FAIL | c386 FAIL; earlier PASS E1:452 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Negated-condition pruning oracle fails; exact refinement not yet isolated. | Trace predicate refinement and branch join; no fix selected. | NOT RUN |
| `semantic::capabilities::flow_branches::trusted_type_test_prunes_branch_contradicting_established_literal_type` | FAIL | c386 FAIL; earlier PASS E1:464 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Type-test pruning oracle fails; exact refinement path not yet isolated. | Trace trusted test result through branch construction; no fix selected. | NOT RUN |
| `semantic::capabilities::flow_loops::break_and_continue_preserve_loop_exit_and_backedge_facts` | FAIL | c386 FAIL; earlier PASS E1:469 and flow-module PASS E6/C9 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Expected backedge/exit facts are not preserved; exact transfer not yet isolated. | Trace break/continue edges through loop fixed-point; no fix selected. | NOT RUN |
| `semantic::capabilities::flow_loops::field_mutation_through_loop_preserves_contract` | FAIL | c386 FAIL; earlier PASS E1:474 and flow-module PASS E6/C9 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Field contract oracle fails through loop mutation; exact state transition not yet isolated. | Trace loop field-current invalidation separately from contract; no fix selected. | NOT RUN |
| `semantic::capabilities::flow_loops::loop_fixpoint_preserves_mutated_integer_and_abrupt_edges` | FAIL | c386 FAIL; earlier PASS E1:475 and flow-module PASS E6/C9 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Expected integer/final abrupt-edge facts are not preserved; exact transfer not yet isolated. | Trace fixed-point convergence and abrupt exits; no fix selected. | NOT RUN |
| `semantic::capabilities::generic_conformance::receiver_owner_evidence_is_published_without_owner_inference` | FAIL | c386 FAIL; no known-good record found | TRUE CLEAN-PREDECESSOR BASELINE | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Publication assertion fails; proof/binding cause not yet isolated. | No remediation selected; preserve as predecessor baseline pending assignment. | NOT RUN |
| `semantic::capabilities::generics::generic_callable_return_inference_requires_exact_parameter_domain` | FAIL | c386 FAIL; PASS in 19-test `capabilities::generics` gate E4 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Exact parameter-domain inference oracle fails; cause not yet isolated. | Trace fixed generic-domain handling; do not infer a pack-specific fix without evidence. | NOT RUN |
| `semantic::capabilities::generics::generic_calls_in_one_body_do_not_share_type_variables` | FAIL | c386 FAIL; earlier PASS E1:488 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Call-local variable isolation oracle fails; allocator/frame cause not yet isolated. | Trace inference-variable ownership across sibling calls; no fix selected. | NOT RUN |
| `semantic::capabilities::generics::generic_conflict_is_local_to_call_and_sibling_remains_established` | FAIL | c386 FAIL; earlier PASS E1:489 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Sibling-after-conflict oracle fails; cause not yet isolated. | Trace conflict discharge and sibling publication; no fix selected. | NOT RUN |
| `semantic::capabilities::generics::generic_identity_solves_parameter_from_argument_and_specializes_return` | FAIL | c386 FAIL; earlier PASS E1:487; audit E6 §20 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Ordinary `T: Type` identity specialization fails; exact loss not yet isolated. | Trace fixed generic argument/return inference; do not apply pack rules to ordinary parameters. | NOT RUN |
| `semantic::capabilities::generics::universe_future_preserves_payload_types_through_async_map_then_await` | FAIL | c386 FAIL; no known-good record found | TRUE CLEAN-PREDECESSOR BASELINE | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Payload-preservation assertion fails; type propagation cause not yet isolated. | No remediation selected; preserve as predecessor baseline pending assignment. | NOT RUN |
| `semantic::capabilities::higher_order::direct_and_explicit_call_on_same_block_publish_same_result` | FAIL | c386 FAIL; earlier PASS E1:514 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Direct/explicit result parity oracle fails; call-product cause not yet isolated. | Compare both call paths’ published products; no fix selected. | NOT RUN |
| `semantic::capabilities::self_types::test_constructor_self_type_specialization_and_counterexample` | FAIL | c386 FAIL; earlier PASS E1:538 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Constructor `Self` specialization/counterexample oracle fails; cause not yet isolated. | Trace exact constructor target and specialized result; no fix selected. | NOT RUN |
| `semantic::capabilities::structural::contextual_product_components_propagate_to_nested_literals` | FAIL | c386 FAIL; earlier PASS E1:541 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Nested contextual component oracle fails; propagation point not yet isolated. | Trace expected component type through nested literal analysis; no fix selected. | NOT RUN |
| `semantic::capabilities::traits::trait_default_replays_external_nominal_callable_dependencies` | FAIL | c386 FAIL; prior failure E5 | TRUE CLEAN-PREDECESSOR BASELINE | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Earlier record attributes the baseline to unexpected Universe `Bool` declaration dependencies/capabilities. | No P4 remediation selected; retain existing baseline classification. | NOT RUN |
| `semantic::capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities` | FAIL | c386 FAIL; prior failure E5 | TRUE CLEAN-PREDECESSOR BASELINE | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Earlier record attributes the baseline to unexpected Universe `Bool` declaration dependencies/capabilities. | No P4 remediation selected; retain existing baseline classification. | NOT RUN |
| `semantic::capabilities::variance::source_nested_and_higher_kinded_self_specialize_to_receiver` | FAIL | c386 FAIL; earlier PASS E1:558 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Nested/HKT receiver substitution oracle fails; cause not yet isolated. | Trace receiver substitution through variance positions; no fix selected. | NOT RUN |
| `semantic::families::invocation::behavioral_family_source_preserves_pattern_storage_and_dispatch_side` | FAIL | c386 FAIL; earlier PASS E1:561 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Family pattern/storage/dispatch-side oracle fails; cause not yet isolated. | Trace canonical family source product and dispatch side; no fix selected. | NOT RUN |
| `semantic::foundations::expression_composition::incompatible_open_record_expansions_have_explicit_diagnostic` | FAIL | c386 FAIL; earlier PASS E1:661 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Open-record incompatibility diagnostic oracle fails; exact solver/composition path not yet isolated. | Trace row expansion conflict and diagnostic ownership; no fix selected. | NOT RUN |
| `semantic::foundations::expression_composition::map_expansion_does_not_fabricate_record_row` | FAIL | c386 FAIL; earlier PASS E1:665 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | No-fabricated-row oracle fails; cause not yet isolated. | Trace map expansion into record-row evidence; no fix selected. | NOT RUN |
| `semantic::foundations::expression_composition::map_unknown_key_and_value_do_not_disappear` | FAIL | c386 FAIL; earlier PASS E1:695 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Unknown key/value preservation oracle fails; cause not yet isolated. | Trace unknown key/value through map composition; no fix selected. | NOT RUN |
| `semantic::foundations::semantic_correctness_regressions::causal_invalidity_does_not_suppress_analyzable_downstream_dispatch` | FAIL | c386 FAIL; earlier PASS E1:834 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Downstream analysis oracle fails; cause not yet isolated. | Trace invalid parent and independent child dispatch; no fix selected. | NOT RUN |
| `semantic::golden::golden_07_unknown_authority` | FAIL | c386 FAIL; earlier PASS E1:865 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Golden expected-output mismatch; precise underlying product not yet isolated. | Trace the first differing semantic product; no fix selected. | NOT RUN |
| `semantic::golden::golden_12_higher_order_mutation` | FAIL | c386 FAIL; earlier PASS E1:867 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; no narrower P4 task assigned pending causal isolation | Golden expected-output mismatch; precise underlying product not yet isolated. | Trace the first differing semantic product; no fix selected. | NOT RUN |
| `semantic::incremental::type_store_revisions::p3_mode_only_pack_binder_edit_invalidates_dependents_to_cold_parity` | FAIL | c386 FAIL; C2 PASS E3 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; T12 incremental fingerprints/cold-incremental parity | Cold/incremental selector fails; exact divergence not yet isolated. | Compare final semantic products before changing recomputation expectations; no fix selected. | NOT RUN |
| `semantic::integration::editor::editor_visible_symbols_use_prelude_policy_and_preserve_local_shadowing` | FAIL before fix; PASS after fix | c386 FAIL; earlier PASS E1:1081; audit E6 §23 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; T12 editor/source-index projection | `visible_symbols_at` required a matching `snapshot.declarations` product for every prelude target. Canonical `Unit` is source-backed in `PreludeTypeMap` but has no such per-snapshot nominal product, so it was omitted; local `Int` shadowing was already correct. | Treat explicit `PreludeTypeMap` entries as the canonical visible target authority; preserve local/declaration name suppression and do not synthesize a target. Test asserts `Unit` resolves to its canonical source declaration. | PASS: exact editor selector, explicit source-backed prelude-map control, and local-resolution-precedence control each selected/passed 1 test (pinned nightly-2026-07-10). |
| `semantic::integration::workspace::invalid_generic_kind_does_not_publish_ready_declaration_header` | FAIL before fix; PASS after fix | c386 FAIL; earlier PASS E1:1166; audit E6 §23 | CURRENT REGRESSION — already present at c386 | Audit Priority 3 classification; T12 workspace/declaration publication | Final generic class/data formation returned `Invalid`, but the predeclared shell remained in the declarations table. | Add only `Invalid` final generic signatures to `blocked_declarations`; preserve retryable/nonterminal outcomes. Regression checks invalid class/data absence, diagnostic retention, and valid class/data publication. | PASS: exact selector 1 passed; valid generic kind/signature controls each 1 passed (pinned nightly-2026-07-10). |

## Disposition

The matrix contains 35 `CURRENT REGRESSION` cases with older source-backed PASS
evidence, 4 `TRUE CLEAN-PREDECESSOR BASELINE` cases, and 1
`INTENTIONAL P4 MIGRATION`. The 35 regressions were already present at c386;
their causal owners and remediation priority remain for review. This audit
Priority 3 record does not complete P4 T13, whose T1–T12 preconditions remain
unmet. The two c386-only failures now passing on current main are noted above
and are not among the current 40. Implementation-side edits so far are the
explicit-alias fixture migration plus the invalid generic-header publication
correction recorded above. Their exact focused tests passed; no broad
post-fix semantic run is part of this classification pass.
