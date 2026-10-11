# Four data-fixture repairs: walkthrough

State: COMPLETE / IMPLEMENTED / FOCUSED_TESTED.
Repository anchor: `0076ad54`, `.worktrees/complex-math`, `feat/complex-math`.
The user requested continuation of `deferred-data-fixtures-handoff.md`.

## Outcome

All four deferred tests now pass. The complete `language::data_e2e` acceptance
gate passed: 32 selected tests, zero failures, zero ignored.

Eight declarations changed in `phalcom-core/tests/core/language/data_e2e.rs`.
Every assertion remains intact. No production code changed.

## Causes and corrections

| Test | Reproduced failure | Correction and preserved assertions |
| --- | --- | --- |
| `data_record_evaluates_arguments_in_source_order` | Semantic `AssignmentToImmutable` for `n` | `let mut n`; first remains 2 and second remains 1 |
| `data_source_order_with_exception_proves_evaluation_semantics` | Semantic `AssignmentToImmutable` for `order` and `threw` | Both use `let mut`; threw remains true and order remains 10 |
| `p4_shape_generic_construction_through_generic_callable_keeps_behavior_class_identity` | Missing successful static-constant lowering projection for `int_record` | Two typed runtime `let` bindings; shared behavior class and distinct exact types |
| `p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable` | Missing successful static-constant lowering projection for `int_row` | Three typed runtime `let` bindings; repeated Int descriptor reuse, distinct Int/String descriptors and exact types, shared behavior class and empty layout |

The normative binding specification makes plain `let` immutable and requires
`let mut` for reassignment. The static-evaluation specification excludes arbitrary
runtime method execution. These row fixtures call ordinary generic Factory
methods and inspect runtime descriptors. Their predecessor records in
`LANG007.C3-CHECKPOINT.md` (INC-C3-15/16) establish runtime specialization as the
tested contract. Typed runtime bindings preserve that contract and its expected
result context.

This follows the handoff's fixture correction path. No architectural consultation
or plan amendment was needed.

## Verification

Each command below selected exactly one test. All four failed before correction
with the recorded diagnostic, then passed with the original assertions.

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::data_record_evaluates_arguments_in_source_order -- --exact --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::data_source_order_with_exception_proves_evaluation_semantics -- --exact --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::p4_shape_generic_construction_through_generic_callable_keeps_behavior_class_identity -- --exact --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable -- --exact --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e -- --nocapture --test-threads=1
rustfmt --check --config skip_children=true phalcom-core/tests/core/language/data_e2e.rs
git diff --check
```

The complete acceptance gate passed 32/32 in 503.86 seconds. Its displayed
positional-only closure traceback is expected by the passing negative test.
Scoped formatting and diff checks passed. The source diff contains only the
eight declaration corrections.

Workspace build/test/clippy, broad crate suites and release certification remain
deferred. The existing Complex evidence was retained because no shared production
path changed. No deferred fixture failure remains in this acceptance gate.
CHECKPOINT.md owns the updated verification ledger; the original repair handoff
is marked resolved.
