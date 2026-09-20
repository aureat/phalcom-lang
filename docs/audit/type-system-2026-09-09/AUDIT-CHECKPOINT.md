# Type-system audit checkpoint — 2026-09-20

## Active repair

**Generic override compatibility and captured-write flow soundness**

- **Status:** IMPLEMENTED
- **Verification:** FOCUSED_TESTED
- **Scope:** `AUD-SEM-001` override-certification boundary and its shared generic-specialization invariant with `AUD-SEM-005`; `AUD-SEM-002` captured mutable-local invalidation.

## Root cause

Override validation correctly specialized declaration-owned generic parameters through the canonical receiver-to-owner projection, but compared callable-owned generic parameter IDs directly. Equivalent inherited and overriding method binders therefore differed by `TypeParameterOwner::Callable`, causing false `IncompatibleOverride` diagnostics for symbolic/HKT methods.

## Established rule

Override compatibility first projects both signatures through `specialize_receiver_to_owner`. It then alpha-aligns only corresponding candidate callable binders to inherited callable binders after matching arity, kinds, and variance metadata. Parameter contravariance and result covariance continue through the canonical relation engine. Generic constraints are normalized by the same substitution and must match; this conservative rule prevents strengthened caller obligations without introducing a duplicate constraint-implication engine.

Declaration-owned parameters, `Self`, selectors, labels, rest shape, and all non-callable free identities remain unchanged.

## Implementation

- `phalcom-semantic/src/checker/declaration.rs`
  - alpha-aligns callable generic binders after ancestor specialization;
  - checks generic kind/variance metadata and normalized constraints;
  - retains existing selector-shape and variance relation checks.
- `phalcom-semantic/tests/audit_probes/override_certification.rs`
  - adds a valid symbolic HKT override regression;
  - adds an incompatible symbolic HKT result regression.

## Evidence

Passed:

```text
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::alpha_equivalent_generic_hkt_override_is_accepted -- --nocapture
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::generic_hkt_override_with_incompatible_result_is_rejected -- --nocapture
cargo test -p phalcom-core --test core \
  traverse_body_resolves_symbolic_inherited_capabilities -- --nocapture
cargo test -p phalcom-core --test core \
  typing_integration::monads::bodies -- --nocapture
```

`typing_integration::monads::bodies` passed all three symbolic-body regressions, including `traverse_body_resolves_symbolic_inherited_capabilities`. The expected `Monad<F> -> Applicative<F> -> Functor<F>.map` resolution now reaches the asserted `Assumed` symbolic-evidence products without override diagnostics.

The full `audit_probes::override_certification` selector remains baseline-blocked by unrelated `AUD-SEM-002` captured-write coverage; 7 tests passed, 2 ignored retained audits, and only `captured_write_must_invalidate_previous_int_fact` failed. `cargo fmt --all -- --check` is also baseline-blocked by unrelated formatting in pre-existing modified files.

## Next action

`TEST002.C1.P1 / T8` resumed and reached its intended symbolic-evidence assertions. BL-19's generic-override prerequisite is resolved; no further work on this override gate or symbolic inherited-capability trace is currently required.

## AUD-SEM-002 — captured mutable write

### Root cause

Closure construction correctly restores the enclosing `FlowState`. Every call
then enters the shared opaque-call barrier, which invalidated mutable predicates
and field-current facts but left mutable local `BindingState.current` and its
version untouched. A captured `Int | String` local initialized with `Int` could
therefore retain `Established Int` after the invoked closure wrote `String`.

### Established rule

At an opaque call boundary, mutable binding current knowledge and denotation are
transient and must be invalidated; persistent `BindingContract` remains. The
canonical binding helper derives a conservative replacement: a source annotation
becomes assumed contract knowledge, while non-source contracts and unconstrained
bindings become `Unknown(NoTypeEvidence)`. The invalidated binding advances its
version and loses its stale explanation. Immutable locals remain precise, and
field fallback remains owned by the existing field-lifecycle path.

This intentionally invalidates uncaptured mutable locals too. It is the current
conservative opaque-call policy, documented by regression coverage; precise
captured-write summaries are future work and must not weaken this repair.

### Implementation and evidence

- `phalcom-semantic/src/checker/binding.rs` owns conservative current-knowledge
  derivation from a retained binding contract.
- `phalcom-semantic/src/checker/flow/state.rs` applies it to mutable locals in
  `invalidate_opaque_calls()` while retaining contracts and invalidating stale
  predicate/explanation/version state.
- `phalcom-semantic/tests/semantic/capabilities/deep_regressions.rs` covers
  annotated captured writes, aliases, nested helper invocation, same-type
  writes, immutable precision, and the intentional uncaptured-local boundary.

Passed:

```text
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::captured_write_must_invalidate_previous_int_fact -- --nocapture
cargo test -p phalcom-semantic --test semantic capabilities::deep_regressions -- --nocapture
cargo test -p phalcom-semantic --test semantic \
  capabilities::flow_loops::captured_block_write_is_not_applied_until_execution_is_proven -- --nocapture
```
