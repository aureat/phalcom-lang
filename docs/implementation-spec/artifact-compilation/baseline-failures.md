# Baseline failures (slice 0)

Recorded 2026-10-23 at anchor `62ddc6acd`.

## Lib-test compile errors

`cargo test -p phalcom-semantic --lib --no-run` failed with 3 errors. They
**predate** `48435bb71`: the identical 3 errors reproduce at `141e01442`
(throwaway worktree).

| File | Cause | Fix |
|---|---|---|
| `session/publication_tests.rs` | helper renamed `extend_canonical_trait_resolver_authority` -> `extend_immutable_trait_resolver_authority` (`09e622175`) | follow the rename |
| `static_value/artifact.rs` (test) | `ArtifactTypeId` not imported | fully qualified path |
| `types/artifact.rs` (test) | `Box<str> == &str` comparison | compare the single path segment as `&str` |

## Runtime failures (pre-existing, unrelated to slices A–D)

With the 3 fixes applied, `cargo test -p phalcom-semantic --lib` gives
370 passed / 8 failed at the anchor, and 368 passed / 8 failed at `141e01442`
(+2 tests that did not compile there). The same 8 fail at both revisions:

- `artifact::compatibility_tests::package_artifact_import_registers_trusted_receipt_in_retained_workspace_graph`
- `artifact::export::proof::tests::callable_proof_source_publication_round_trips_gadt_authority_and_budget_rollback`
- `artifact::export::proof::tests::callable_proof_alpha_renaming_preserves_wire_projection_and_body_fingerprint`
- `canonical_image::negative_fact_tests::list_get_returns_inferred_anonymous_variant_union_contract`
- `checker::analysis::callable_analysis_table_tests::callable_proof_fingerprint_preserves_local_branch_authority`
- `checker::context::tests::tracking_resolver_demands_the_exact_unready_type_member_owner`
- `canonical_image::tests::loaded_formal_image_final_arc_failure_does_not_commit_and_retry_succeeds`
- `types::proof::tests::callable_proof_publication_preserves_nested_constructor_visibility`

These are not caused by this program. Any later run must show exactly this set
and no other failure.

## Slice A–D unit tests

`artifact::projection::tests::*` (2 tests) pass. No slice-C/D export test
exercises a non-Universe scope yet (see 04).

## `phalcom-compiler-artifact` test target

`semantic_world_enum_requirements_test::enum_requirement_rows_round_trip_all_statuses_and_source_order`
fails with `Allocation(UnaccountedBoundary)` in the decode of enum requirement
records. It does not touch the module world and is unrelated to this program.
All other tests of the crate pass.
