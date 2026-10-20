---
id: PERF003.C2.P7.PATCH0
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# PERF003.C2.P7.PATCH0 Handoff

## Delivery state

- Branch: `perf/universe-characterization`.
- PATCH0 implementation commit: `8284893769d121dc2fe33ed0c866a402ff48bd35`.
- This is the implementation HEAD for the source/test unit.
- The PATCH0 implementation commit contains only the fingerprint implementation
  and its incremental fingerprint tests. The supplied PATCH0/PATCH1/PATCH2
  plans, audit material, consolidated patch documents, and checkpoint/lifecycle
  records are delivered in the documentation commit directly after it.
- The delivery branch HEAD is that documentation commit; the complete branch
  tip is reported after push. Worktree status is verified clean after delivery.

## Established fingerprint contract

`callable_body_product_fingerprint` now hashes:

- `ExpressionAnalysis.binding_write`;
- all `AssociatedResolutionKind` variants and common owner/form/family fields;
- complete static and dynamic `family_applications` selections;
- `BoundFamily.family_type`, selector spec, and complete bound-member semantics;
- nested associated callable-reference payloads through the same associated
  resolution helper used by the top-level map;
- trait-dispatch selections through the same field set already used by
  `hash_trait_dispatch_site`.

The implementation is field-wise, exhaustive, and directly traverses ordered
maps/slices. It does not hash `dependency_fingerprint`, `Debug` output, source
ranges newly omitted by the prior contract, diagnostic presentation, or local
diagnostic cause numbering.

## Focused evidence

- Focused fingerprint module filter: 23 selected, 23 passed.
- Command used: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test callable-fingerprint-temporary callable_body_product -- --nocapture`.
- This ran the existing `fingerprints.rs` module with its normal `support` module
  using a temporary explicit Cargo test target. The temporary Cargo entry and
  harness were removed after the run.
- Exact callable-analysis semantic baseline:
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib callable_analysis -- --test-threads=1`
  — 19 selected, 19 passed.
- The plan's normal semantic integration target could not compile because
  `phalcom-semantic/tests/semantic/integration/canonical_image.rs:203` invokes
  `local_len()` on the frozen enum-requirement owner wrapper. This inherited
  test compile blocker is outside PATCH0 scope and was not changed.
- Artifact callable-analysis baseline:
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test semantic_world_callable_analysis_test -- --test-threads=1`
  — 9 selected, 4 passed, 5 inherited failures with unchanged identities.
- `rustfmt --edition 2024 --check` on the two scoped Rust files and
  `git diff --check` passed.

## Inherited artifact failures

1. `associated_family_capture_round_trips_subset_and_optional_targets` —
   `Allocation(UnaccountedBoundary)`.
2. `behavioral_family_capture_round_trips_conditional_and_unconditional_targets` —
   conditional dispatch lacks serialized implementation-member authority.
3. `callable_analysis_status_is_preserved_by_semantic_world_codec` —
   `Allocation(UnaccountedBoundary)`.
4. `callable_flow_graph_round_trips_owner_local_nodes_edges_and_entry_exit` —
   `Allocation(UnaccountedBoundary)`.
5. `explicit_dynamic_call_argument_round_trips_without_a_type_node` —
   `Allocation(UnaccountedBoundary)`.

No artifact schema, import/export guard, semantic allocation guard, or allocation
policy changed in PATCH0.

## PATCH1 takeover

The `PATCH0_callable_fingerprint` prerequisite is SATISFIED. PATCH1 may use the
repaired native fingerprint after rebuilding the complete relocated owner.
PATCH1 remains otherwise outstanding: this handoff does not establish portable
owner transport, cross-process raw fingerprint equality, allocation-exact
reconstruction, query-image correctness, or source-denied parity.

Do not revert the exhaustive associated helper to `DataExpansionField`-only
hashing during PATCH1 schema work. Imported associated callable references must
recompute through the same native semantic fingerprint path.

P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED. P8 remains blocked.
