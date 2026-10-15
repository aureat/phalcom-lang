---
id: PERF003.C2.P7.PATCH0
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# PERF003.C2.P7.PATCH0 Walkthrough

## Starting and final revisions

- Starting revision: `21a54705c33dae95ee9c77d86789b4911635e6b0`.
- PATCH0 implementation commit: `8284893769d121dc2fe33ed0c866a402ff48bd35`.
- The lifecycle and supplied plan records are delivered in the following
  documentation commit; the final clean branch revision is reported in the
  handoff/final delivery status.

## Production changes

`phalcom-semantic/src/db/fingerprint.rs` now includes `ExpressionAnalysis.binding_write`
and routes associated resolutions, callable references, and family applications
through private field-wise hash helpers. The associated helper hashes common
owner/form/family fields and every selected variant payload. Specialized
associated members hash member identity, operation, value type, and optional
target. Ordered owner maps are traversed directly without sorting or scratch
vectors.

`FamilyApplicationResolution` hashes family type, kind, and complete static or
dynamic selection. Static selections include operation, optional target,
callable type, and result type. Dynamic selections include ordered candidates
and optional result type.

`BoundFamily` callable references hash family type, `BehavioralFamilySpec`, and
every member's operation, member kind, target, callable type, conditional
selection, conditional capability, and trait-dispatch selection. Nested
`Associated` callable references reuse `hash_associated_resolution`.

Trait-dispatch selection hashing was extracted from `hash_trait_dispatch_site`
and reused by bound members. The existing evidenced-selection dimensions remain
unchanged: source implementation, exact target/type reference, requirement,
evidence fingerprint, selected requirement template, and requirement-target
map. `dependency_fingerprint` remains excluded.

## Exhaustive associated-resolution coverage

| Variant | Hashed payload |
|---|---|
| `ExactValue` | member, value type |
| `DataExpansionField` | logical index, value type |
| `ExactCallable` | member, target, callable type |
| `Family` | family type, ordered specialized members |
| `StaticInvoke` | member, target, result type, argument mapping |
| `StaticDomain` | member, domain |
| `StaticTerm` | domain, case, term |
| `DynamicInvoke` | ordered candidates, optional result type |

Every variant also hashes `owner_form`, `lookup_owner`, and optional family
identity. The match is exhaustive and has no wildcard arm.

## Tests and verification

Added 12 discriminators in
`phalcom-semantic/tests/semantic/incremental/fingerprints.rs`: binding write;
static family target and kind; dynamic family candidate; associated variant,
payload, and family owner; bound family type, selector, and member type; nested
associated payload; and shared top-level/nested associated-helper sensitivity.
Existing product invariance tests remained in the same focused filter.

The repository's normal `--test semantic` target is blocked before selection by
an inherited compile error at `tests/semantic/integration/canonical_image.rs:203`:
`EnumRequirementOwnerMap` does not expose the old `local_len()` method. That
out-of-scope test was left unchanged. The exact fingerprint module was instead
compiled with its normal incremental support module through a temporary test
target and harness; both temporary files/Cargo edits were removed after the run.

| Command | Result |
|---|---|
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test callable-fingerprint-temporary callable_body_product -- --nocapture` (temporary harness) | 23 selected, 23 passed |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib callable_analysis -- --test-threads=1` | 19 selected, 19 passed |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic callable_body_product -- --nocapture` | Did not compile: inherited `local_len()` error above |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test semantic_world_callable_analysis_test -- --test-threads=1` | 9 selected, 4 passed, 5 inherited failures below |
| `rustfmt --edition 2024 --check phalcom-semantic/src/db/fingerprint.rs phalcom-semantic/tests/semantic/incremental/fingerprints.rs` | Passed |
| `git diff --check` | Passed |

The pre-existing artifact failures retained their exact identities:

- `associated_family_capture_round_trips_subset_and_optional_targets` —
  `Allocation(UnaccountedBoundary)`.
- `behavioral_family_capture_round_trips_conditional_and_unconditional_targets` —
  `LimitExceeded("conditional dispatch selection has no serialized implementation-member authority")`.
- `callable_analysis_status_is_preserved_by_semantic_world_codec` —
  `Allocation(UnaccountedBoundary)`.
- `callable_flow_graph_round_trips_owner_local_nodes_edges_and_entry_exit` —
  `Allocation(UnaccountedBoundary)`.
- `explicit_dynamic_call_argument_round_trips_without_a_type_node` —
  `Allocation(UnaccountedBoundary)`.

The callable-analysis library baseline was 19/19 before and after PATCH0. The
artifact baseline stayed 4/9 with the same five failures. No artifact DTO,
schema, import/export guard, allocation accounting, or allocation policy code
changed.

## Completion and deferred work

PATCH0 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED. PATCH1's callable fingerprint
prerequisite is satisfied; PATCH1 remains otherwise unstarted. P7 remains
IN_PROGRESS / PARTIAL / UNVERIFIED. Portable owner reconstruction, source-denied
compilation, allocation closure, determinism, full semantic parity, and P5/P6
acceptance remain deferred to the later P7 plans.
