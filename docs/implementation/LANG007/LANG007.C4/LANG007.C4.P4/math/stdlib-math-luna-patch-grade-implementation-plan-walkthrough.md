---
kind: implementation-walkthrough
plan: stdlib-math-luna-patch-grade-implementation-plan.md
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
branch: stdlib-random-math
date: 2026-10-07
---

# `universe.math` completion walkthrough

This records the T7–T12 completion continuation. The bounded `universe.math`
plan is complete. Random, secure random, and distributions remain outside this
plan.

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
- T7/T8 focused numeric coverage was added in
  `phalcom-core/tests/core/audit_probes/math.rs`. The probe suite passed 11/11 and
  verifies the six hard-coded Float constant bit patterns against both the
  documented literals and the standard-library binary64 constants, all 24
  Float descriptor registrations and purity metadata, same-platform host
  operation equivalence, IEEE domain/signed-zero/subnormal behavior, direct
  no-allocation native calls, independent BigInt gcd/lcm laws, isqrt boundary
  and invariant coverage through 10^1000, negative-input ArgumentError, and
  numeric-limit enforcement.

## Verification limits and residuals

Final T12 evidence acquired on `de9b1657` plus the numeric tests:

| Gate | Command / selection | Result |
|---|---|---|
| A | `git diff --check`; targeted `rustfmt --edition 2024 --check` on the two test files | PASS |
| A | `cargo fmt --all -- --check` | FAIL, inherited workspace formatting drift |
| B | `RUSTFLAGS='' cargo check -p phalcom-core`, `-p phalcom-modules`, `-p phalcom-native-meta`, `-p phalcom-native-surface` | All PASS |
| C | `RUSTFLAGS='' cargo test -p phalcom-core --test core exported_static_constant_materializes_for_selected_and_module_reads -- --nocapture` | PASS, 1 |
| C | Same harness, `exported_static_constants_keep_canonical_identity_across_reexports_and_gc` | PASS, 1 |
| D | `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus math -- --nocapture` | PASS, 1; four fixtures |
| E | `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_ -- --nocapture` | 54 PASS, 6 FAIL; classified below |
| F | `RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check` | PASS, 383 declarations; generated artifact unchanged |
| F | `RUSTFLAGS='' cargo test -p phalcom-core --lib native:: -- --nocapture` | PASS, 11 |
| G | Core `universe_math_exports_exact_families_and_rejects_non_matching_numeric_arguments` and `universe_math_implementation_children_are_not_exposed` | PASS, 1 each |
| G | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic families::invocation -- --nocapture` | PASS, 11; includes warm/cold provider mutation |
| H | `RUSTFLAGS='' cargo test -p phalcom-core` | 160 PASS, 3 FAIL in library tests; Cargo stops before integration targets |
| T7/T8 | `RUSTFLAGS='' cargo test -p phalcom-core --test core audit_probes::math -- --nocapture` | PASS, 11 |

The numeric tests add 617 Float backend-equivalence cases (21 unary kernels
times 19 IEEE/finite inputs, 81 pairs each for atan2/hypot, and 56 log/base
pairs), plus a 12-by-12 exact-integer cross-product matrix. Native purity calls
run through every registered kernel entry twice and compare heap handles and
numeric policy before/after; source review confirms all kernels ignore VM and
receiver state and perform no time/random access. NaNs are compared by
classification, never payload bits. No numerical dependency was added.

An intermediate gcd resource fixture using a same-module negative const exposed
`static constant negative ... missing successful lowering projection`. This
is a D baseline static-const projection issue outside the math API; no fix was
attempted. The gcd policy test instead passes an admitted exact value through
the public Family and discriminates the checked negation inside `gcd`.

The wider module cluster's six failures are D relative to this test-only
continuation: five fixture cases call non-callable exported values
(`project_hierarchy`'s `b.calculate(21)` and `builtin_client`'s
`json.parse("test")`), and Telogue fails generic inference in `query`.
They do not import math. All module linking/parity/math tests passed. No
production source or existing test expectations were changed to address them.

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
- The plan is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`. The focused feature
  evidence is positive. `BASELINE_BLOCKED` records the pre-existing
  workspace formatting drift and unrelated core/module failures;
  it does not indicate a math failure.

## Amendments and deferred verification

No semantic or architectural amendment was needed. T9's canonical module-value
owner and the hidden-child/exact-Family architecture are preserved. The final
gate accepts the explicitly recorded baseline exceptions under plan §26.
Workspace build/test/clippy release certification and optional high-precision
transcendental checks were not run. The latter are not required by T7's
same-platform backend contract. C4/P4/R1 and random acceptance are unchanged.

## Delivery state

The implementation continuation was committed and pushed on
`stdlib-random-math` as `2c248f93c7047080908453a6d46039eb3a768ccc`. This
numeric completion is recorded by the scoped commit titled
`test: complete universe math verification`. The paired handoff
records the final math state and the exact residual baseline exceptions.
