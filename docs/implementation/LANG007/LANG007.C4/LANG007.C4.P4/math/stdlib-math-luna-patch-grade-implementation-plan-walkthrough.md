---
kind: implementation-walkthrough
plan: stdlib-math-luna-patch-grade-implementation-plan.md
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
branch: stdlib-random-math
date: 2026-10-07
---

# `universe.math` continuation walkthrough — T9 onward

This records the T9–T12 continuation. It is not a completion claim for the
whole math plan: T7's Float edge/accuracy matrix and T8's integer
property/boundary matrix remain open, and M1 remains partial.

## Implemented

- T9: linked public module values are resolved at the canonical module-value
  boundary to their checked `SymbolId`/`ModuleBindingFact`. Member bodies now
  receive the canonical export environment and linked selective imports;
  `CallableBody` records `ModuleStaticBindings` dependencies. Existing Family
  call checking remains the sole call-checking owner.
- T9 regressions cover selective and qualified Family calls in member bodies,
  wrong argument types, provider contract changes, and warm/incremental versus
  cold-analysis parity.
- T10: native-surface generation and `--check` pass with 383 declarations;
  generation produced no file changes.
- T11: package topology, static-constant identity flow, Float accuracy limits,
  public package notes, and accepted receiver-exposure debt were updated.
- T12 focused gates passed for the exported constant read, re-export/GC
  identity, hidden-child rejection, math corpus, semantic Family typing, and
  crate checks for core/modules/native-meta/native-surface. `git diff --check`
  and targeted `rustfmt --check` on changed Rust files pass.

## Verification limits and residuals

- `RUSTFLAGS='' cargo test -p phalcom-core` ran 163 library tests: 160 passed
  and three failed in unrelated existing areas: Option-match override routing
  (`universe::tests::kernel_option_match_override_reroutes_every_combinator`),
  captured setter shape validation
  (`vm::send::tests::captured_setter_rejects_an_ordinary_positional_rhs_lane`),
  and stack-walk ordering
  (`vm::walk::tests::walk_orders_oldest_first_with_selector_shaped_names`).
  These failures are outside the changed semantic/module-value and math
  surfaces; no fix was attempted.
- `cargo fmt --all -- --check` fails on existing formatting drift in other
  workspace files, including previously established static-constant/runtime
  code. Changed Rust files were individually formatted and pass targeted
  checking; unrelated files were not reformatted.
- The plan is therefore `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`, not
  complete. The focused feature evidence is positive, but broad certification
  and T7/T8 are outstanding.

## Delivery state

The implementation continuation was committed and pushed on
`stdlib-random-math` as `2c248f93c7047080908453a6d46039eb3a768ccc`. The paired
handoff records the exact continuation point.
