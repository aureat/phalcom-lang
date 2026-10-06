---
id: LANG007.C4.1
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# LANG007.C4.1 corrective closure walkthrough

## Implemented

- Centralized owner-relative `Self` term and generic-constraint specialization in `types/substitution.rs`.
- Applied the shared transformation to body-local callable signatures and dispatch specialization.
- Preserved callable-local binder identities and inference terms.
- Kept `specialize_self_type()` structurally closed over RecordRow fields, anonymous tails, projection tails, static row arguments, projection subjects, and type-member row arguments.
- Published signature-formation diagnostics from inherent contributions so invalid `Self<T>` applications are reported as `ApplicationNotConstructor`.
- Added exact `(ImplId, source_member_index)` source-body provenance coverage.
- Made inherent body receiver overrides dispatch-side aware: class-side
  lowercase `self` remains a class object while type-level `Self` remains the
  source-relative instance target.
- Added an exact class-side receiver/dispatch regression and an explicit
  `CallableBody -> CallableDefinition` dependency assertion.

## Focused evidence

Passing:

- `RUSTFLAGS='' cargo check -p phalcom-semantic`
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::bodies` — 18 passed
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::signatures` — 4 passed
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::record_rows_field_keys` — 14 passed
- `git diff --check`

The body tests cover callable-local `Self` constraints, shadowing, unrelated binder rejection, `Self<T>` rejection, receiver/type-`Self` separation, context-only permutation invalidation, and incremental/cold parity. Record-row tests cover all required carriers, tails, trait arguments, and covering row materialization.

## Baseline classification

`impls::queries::adding_impl_member_does_not_invalidate_another_target_or_its_caller` was reproduced on clean predecessor `677bb585` and repaired in the current corrective follow-up. The failure came from coarse callable-body input identity, which included workspace-wide source-resolution state and a pre-signature/direct-contract mismatch during revalidation.

The broad `cargo fmt --all -- --check` gate reports unrelated existing formatting drift outside this patch. Modified files were formatted directly and pass `git diff --check`.

## Corrective follow-up

The callable-body fingerprint now uses the canonical body inputs consistently:
nominal bodies rely on their exact query dependencies, while body-relative
covering/self contexts retain their local contract marker and lifecycle facts.
Workspace-wide source-resolution and aggregate conformance fingerprints no
longer force unrelated bodies to recompute. The regression now passes with the
full `impls::queries` and `impls::bodies` groups.
