---
id: TEST002.C1.P2
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# TEST002.C1.P2 — Walkthrough

## Outcome

P2 resolved the generic override-validation blocker that prevented BL-19 from
reaching its symbolic inherited-capability assertions. No generic type-system
representation, kind rule, variance rule, or generic-constraint semantics was
redesigned.

## Root cause and repair

The validator already used the canonical receiver-to-owner specialization path
for declaration-owned generic parameters. It then compared callable-owned
generic parameter identities directly. Alpha-equivalent method binders such as
inherited `A, B` and override `X, Y` therefore failed the canonical relation
despite equivalent specialized signatures.

`checker/declaration.rs` now alpha-aligns only corresponding candidate
callable-owned binders to inherited binders. It first requires matching generic
arity, kind, and variance metadata, then normalizes generic constraints under
the same substitution. Existing selector-shape, parameter-contravariance, and
result-covariance relation checks remain authoritative.

This preserves declaration-owner identities, receiver-relative `Self`, and
all other free variables. It contains no fixture, module, capability, selector,
`Dynamic`, or `Object` fallback exception.

## Regression coverage

Added audit probes for:

- an accepted symbolic higher-kinded override through an `Either` constructor
  lambda;
- a rejected symbolic higher-kinded override with an incompatible result.

## Verification

Passed:

```text
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::alpha_equivalent_generic_hkt_override_is_accepted -- --nocapture
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::generic_hkt_override_with_incompatible_result_is_rejected -- --nocapture
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::incompatible_override_must_not_certify_int -- --nocapture
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::narrowing_override_parameter_is_rejected -- --nocapture
cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification::widening_override_parameter_is_accepted -- --nocapture
cargo test -p phalcom-semantic --test semantic capabilities::higher_kinded_generics -- --nocapture
cargo test -p phalcom-semantic --test semantic capabilities::constraints -- --nocapture
cargo test -p phalcom-semantic --test semantic capabilities::variance -- --nocapture
cargo test -p phalcom-core --test core typing_integration::monads -- --nocapture
```

Results: 10 HKT, 3 constraint, 4 variance, and 35 monad integration tests pass.
The authoritative BL-19 test passes and reaches `Functor.map` through
`Monad -> Applicative -> Functor` with `Assumed` symbolic evidence.

`audit_probes::override_certification` as a whole remains baseline-blocked only
by the unrelated retained AUD-SEM-002 captured-write probe; seven tests pass,
two are ignored retained audits, and that one probe fails. Workspace-wide
formatting is likewise not claimed because pre-existing unrelated edits fail
the global formatter gate. Scoped changed Rust files pass `rustfmt --check` and
`git diff --check` passes.
