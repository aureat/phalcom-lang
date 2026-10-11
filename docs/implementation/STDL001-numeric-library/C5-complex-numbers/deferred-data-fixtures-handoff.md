# Handoff: four deferred data-fixture failures

State: RESOLVED. The follow-up reproduced all four failures, corrected the eight
fixture declarations and preserved every assertion. All four exact selectors
and the complete `language::data_e2e` gate passed (32/32). See
`deferred-data-fixtures-walkthrough.md` and CHECKPOINT.md for closure evidence.
The instructions below describe the original repair task.

## Anchor and scope

Continue from Complex implementation commit `a72d1f79`, published on
`origin/feat/complex-math`. Worktree: `.worktrees/complex-math`.
Fix the four tests below in `phalcom-core/tests/core/language/data_e2e.rs`.
Read repository guidance, `phalcom-core/tests/README.md`, and the checkpoint's
deferred-failure ledger. Keep the passing Complex identity/evidence interfaces.
Work inline; significant architectural changes require user approval.

## Recorded failures and first corrections

| Test | Recorded cause / next action |
| --- | --- |
| `data_record_evaluates_arguments_in_source_order` | `n` is immutable but reassigned by `next`; declare `let mut n`. Keep `first == 2`, `second == 1`. |
| `data_source_order_with_exception_proves_evaluation_semantics` | `order` and `threw` are reassigned immutable bindings; declare both with `let mut`. Keep `threw == true`, `order == 10`. |
| `p4_shape_generic_construction_through_generic_callable_keeps_behavior_class_identity` | Typed static `const` Factory results lack successful lowering projections. Verify the fixture's runtime construction intent; use typed runtime `let` bindings if confirmed. Preserve shared behavior class and distinct exact types. |
| `p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable` | Same static `const` problem. Verify runtime intent before using typed `let` bindings. Preserve equal IDs for repeated Int rows, distinct Int/String IDs, shared behavior class and empty layout. |

Authority: `docs/specs/bindings-and-fields.md` and `docs/specs/static-evaluation.md`.
If either row test genuinely requires compile-time evaluation, diagnose the owning
static-evaluation/lowering path instead of replacing its phase. Preserve every
behavioral assertion; do not weaken checks or skip tests.

## Verification

The earlier data group run had 27 passes and five failures; its patch-specific
layout guard failure was corrected separately. The four above were classified
unrelated, pre-runtime failures and have not been rerun since that diagnosis.
Reproduce each before editing and run Cargo serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::data_record_evaluates_arguments_in_source_order -- --exact --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::data_source_order_with_exception_proves_evaluation_semantics -- --exact --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::p4_shape_generic_construction_through_generic_callable_keeps_behavior_class_identity -- --exact --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable -- --exact --nocapture
# Coherent acceptance gate after the focused corrections:
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e -- --nocapture --test-threads=1
```

Confirm actual test selection. Record causes, changes and completed evidence in
the deferred ledger and a follow-up walkthrough. Workspace/release certification
remains deferred. Complex is already IMPLEMENTED / FOCUSED_TESTED; re-test it only
if the correction changes a shared production path.
