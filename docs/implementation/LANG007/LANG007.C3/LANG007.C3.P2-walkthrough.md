---
id: LANG007.C3.P2
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# LANG007.C3.P2 — Walkthrough

## Final state

P2 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` at committed revision
`518f66d19941eb24e379e60f64aa89df66643a8a` (`fix(semantic): thread active type
lambda arena`). The INC-C3-11 correction and lifecycle records are committed.
C3 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; P3 is active and owns
intrinsic structural exposure, Universe/corpus migration, and C3 certification.

P2 closes generalized static domains, heterogeneous application and inference,
associated `type data`, metadata representation, module/import identity, incremental
parity, and the T9 resource invariant. The accepted architecture decisions in
INC-C3-8/9/10 remain in force. INC-C3-11's architect ruling is implemented here.

## INC-C3-11: one active lambda arena

`TypeStore::apply_static_form` and `TypeStore::substitute_type_lambda` previously
cloned the complete `TypeLambdaArena`. Both public boundaries now move the arena out
and restore it on ordinary returns. Private active-arena-aware operations pass that
same arena through recursive beta reduction, nested canonical applications, and
free-capture substitution. `TypeStore` remains the canonical `TypeId` authority and
`TypeLambdaArena` remains the scoped/lambda interner. No rollback, transaction,
journal, or panic-recovery semantics were added.

The regression set is in `foundations::type_annotations`:

| Case | Evidence |
|---|---|
| Fresh captured nested lambda applied during the same outer reduction | `fresh_captured_nested_lambda_is_applied_during_outer_reduction` |
| Existing capture and nested higher-kinded shadowing | `nested_type_lambda_beta_reduction_avoids_capture`, `nested_higher_kinded_lambda_shadowing_is_capture_safe` |
| Repeated full and partial application identity | `type_lambda_applications_preserve_canonical_identity`, `partial_type_lambda_application_returns_residual_lambda`, `partial_type_lambda_application_can_be_applied_again` |
| Wrong-kind error restores arena and preserves existing IDs | `checked_lambda_application_failure_restores_the_active_arena` |
| Free-capture substitution containing a nested lambda application | `free_capture_specialization_threads_nested_lambda_applications` |
| Higher-kinded application | `higher_kinded_type_lambda_applies_constructor_and_argument` |

The exact command `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations -- --nocapture` selected 49 tests and passed 49/49. A source search found no whole `TypeLambdaArena` clone under `phalcom-semantic/src/types`.

## Final focused evidence

| Coverage | Evidence and result |
|---|---|
| CV-P2-01, predecessor foundation | P1/P2 predecessor evidence in the checkpoint; P1 remains complete and focused-tested. |
| CV-P2-02, binder/domain semantics | `foundations::kinds` (18), `foundations::static_domain_inference` (6), both pass. |
| CV-P2-03, heterogeneous application | `foundations::generic_application` (11), pass. |
| CV-P2-04, row application/substitution | `foundations::record_row_inference` (6), pass; row-polymorphism integration has three known clean-predecessor failures. |
| CV-P2-05, ProperType | T9 audit: runtime typing, layout, subtyping, hierarchy, and dispatch retain proper-Type admission; `type_data_case_cannot_be_used_as_a_runtime_value` passes. |
| CV-P2-06, repeated canonical formation | `type_lambda_applications_preserve_canonical_identity` and `nested_type_application_canonicalizes` pass. |
| CV-P2-07, static identity | Static-domain and static-data identity inequality/canonicalization cases in `foundations::kinds` pass. |
| CV-P2-08, checked substitution failure | `checked_substitution_reports_invalid_heterogeneous_application` and `foundations::substitution` (7) pass. |
| CV-P2-09, scoped/beta reduction | `foundations::type_annotations` (49), including capture, shadowing, partial application, and restoration, passes. |
| CV-P2-10, same-constructor inference | `foundations::generic_application` (11), pass. |
| CV-P2-11, constructor/domain refutation | `foundations::static_domain_inference` (6), pass. |
| CV-P2-12, static alias | Static-alias and result-domain evidence in the checkpoint; focused alias checks pass. |
| CV-P2-13, type-member convergence | Type-member projection, alias, generic-header, and selector identity evidence in the checkpoint; `incremental::type_members` (23) passes. |
| CV-P2-14, associated declaration/specialization | `adts::associated` (22 passed, 5 fixture-gated ignored), plus associated owner metadata/import tests. |
| CV-P2-15, unified `::` collision authority | Associated-surface collision introduction/removal and cold-parity cases in `incremental::type_members` pass. |
| CV-P2-16, trait boundary | The unsupported static-domain trait binder produces the targeted diagnostic and withholds invalid evidence; two trait-suite dependency failures are documented as predecessor baselines in the checkpoint. |
| CV-P2-17, metadata round trip | `phalcom-type-meta` (14) passes; 24/25 semantic metadata tests pass, with one clean-predecessor failure described below. |
| CV-P2-18, incremental/cold parity | `incremental::type_members` (23), `incremental::adts` (13 passed, 3 gated ignored), imported provider edits (14), and source index (22) pass. |
| CV-P2-19, runtime-negative boundary | Runtime isolation audit finds no static evaluator or runtime static-case invocation; runtime-negative test evidence is recorded in the checkpoint. |
| CV-P2-20, C1/C2 regression | P1/P2 prerequisite evidence is recorded in the checkpoint; the focused pack/kind/application regression groups pass. |
| CV-P2-21, arena resource invariant | Four new arena regressions pass; source search finds no whole-arena clone in the semantic types implementation. |
| CV-P2-22, domain-specific diagnostics | Static-domain inference and targeted trait-boundary diagnostic evidence pass as recorded above and in the checkpoint. |

Additional final commands passed: `foundations::generics_core` (8),
`foundations::substitution` (7), `foundations::kinds` (18),
`adts::associated` (22 passed, 5 gated ignored), `integration::imported_resolution`
(14), `integration::source_index` (22), `incremental::type_members` (23),
`incremental::adts` (13 passed, 3 gated ignored), `phalcom-type-meta` schema tests
(14), and `cargo check -p phalcom-core`.

The repository demonstrates metadata export/validation and canonical workspace/module
linking with stable provider identity. It does not contain a separate general
`SemanticMetadataBundle -> semantic declaration graph importer`; this walkthrough
does not claim one.

## T9 audits and deferred items

- **Runtime isolation:** no P2 VM bytecode, runtime static constructor object, static
  unifier, or runtime normalization invocation was introduced.
- **ProperType:** generalized static terms do not cross proper-runtime-type boundaries.
- **Applied consumers:** general semantic traversals preserve `StaticArgument::Term` and
  `StaticArgument::RecordRow`; consumers behind proven type-only/runtime boundaries
  explicitly project or reject. The different-origin supertype projection path remains
  type-only; heterogeneous row inheritance is not a P2 acceptance requirement.
- **Trait boundary:** unsupported static-domain trait binders fail closed and do not
  fabricate conformance evidence.
- **Resource:** both whole-arena clones are gone. The bounded move-out/restore operation
  is O(1) for arena ownership and keeps one active arena in recursive normalization.
- **Performance follow-up:** declaration signature graph edge reconstruction/SCC work
  can traverse the full graph. It remains a non-blocking performance follow-up.

## Baselines and checks not passed

- `integration::metadata::test_metadata_lambda_alpha_equivalence` fails with
  `InvalidKindApplication("callee kind is not an arrow")`; the clean-predecessor
  reproduction is recorded in the checkpoint. The other 24 tests in that module pass.
- `integration::record_row_polymorphism` has three clean-predecessor failures:
  `row_conflict_message_does_not_expose_solver_variables`,
  `repeated_row_parameter_conflict_is_rejected`, and
  `return_record_prefix_contributes_lacks_constraint_to_row_inference`. Eight pass.
- `cargo fmt --all -- --check` reports repository formatting drift. Checking the four
  modified Rust files from HEAD with the current formatter confirms drift was already
  present before this patch; the workspace also reports unrelated paths. No broad
  formatting rewrite was applied.
- Workspace tests, clippy, and release certification were not run; they are outside
  P2's focused acceptance gate.

## Lifecycle

P2 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. C3 remains
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. P3 T1/G1 has passed at
`518f66d19941eb24e379e60f64aa89df66643a8a`; T2/G2 is next. Do not reopen the
accepted P2 architecture without new source/spec evidence.
