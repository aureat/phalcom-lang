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

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic`
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls::bodies` — 21 passed, 1926 filtered out
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::signatures` — 4 passed
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::record_rows_field_keys` — 14 passed
- `git diff --check`

The body tests cover callable-local `Self` constraints, shadowing, unrelated binder rejection, `Self<T>` rejection, receiver/type-`Self` separation, context-only permutation invalidation, and incremental/cold parity. Record-row tests cover all required carriers, tails, trait arguments, and covering row materialization.

## Audit hardening and Telogue integration (2026-10-06)

- Migrated the four Telogue covering-impl returns to bare `Self`. `query.ph` now annotates its variadic parameter as `Pack<*Goals>`; its record constructor uses inferred `Query { ... }` syntax because `Query<*Goals> { ... }` is rejected by the parser.
- Added `modules_telogue_project::telogue_project_compiles_from_declared_entry`, which loads `packages/telogue` through its manifest and declared `prolog.main` entry.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core telogue_project_compiles_from_declared_entry` selected one test and reached semantic analysis, but the project does not compile. It reports two generic-pack contribution errors in `main.ph`, invalid positional-lane projection errors for `Ts::Map<Term>` in `predicate.ph`, and a generic-inference conflict at the `goals` field value in `query.ph`.
- The `query.ph` inference conflict also reproduced with an explicit `Query<*Goals>` return annotation, so it is not caused by the bare `Self` return. The `main.ph` and `predicate.ph` failures are likewise outside the C4.1 `Self` specialization change. These Telogue package blockers remain separate from the focused C4.1 semantic result; full-project compilation is not established.
- The focused body selector now passes 21 tests, including six deterministic three-slot permutations and the nested type/row/projection leak check. The semantic crate check passed after the callable-body field rename. No `Self<...>` applications remain in `packages/telogue`.
- Callable-body query/request and checker fields are now named `receiver_value_override` and `instance_self_override`; body fingerprint inputs use `instance_self_override`. `TypeFormationSite::self_type_override` and its resolver method remain unchanged.
- The scoped implementation `git diff --check` passes. The supplied `LANG007.C4.P4.R1-revised-covering-impl-and-Telogue-integration-audit-ee7e8ee7.md` remains untouched; its existing trailing whitespace on line 3 is excluded from the scoped check.

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
