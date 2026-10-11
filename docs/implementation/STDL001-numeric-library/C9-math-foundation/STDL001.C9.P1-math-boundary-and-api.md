---
id: STDL001.C9.P1
category: STDL
program: STDL001
checkpoint: STDL001.C9
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: UNVERIFIED
---

# STDL001.C9.P1 — Math boundary and API consolidation

## Objective

Apply the user's definitive `universe.math` proposal to the live `feat/complex-math`
tree. Keep the base math facade independent of advanced numerical algorithms,
move special functions behind the optional package boundary, and place the
proposed Float and stable-reduction operations on their canonical owners.

## Fixed decisions

- `universe.math` remains Float-based and keeps its constants, elementary
  functions, exact integer/combinatorial algorithms, and complex child.
- The public special-function path is `numerics.special`; its source backend
  uses only public `universe.math` APIs and error types.
- `Float.fma` is a fused operation. `Float.copySign` and `Float.scalePow2` are
  representation-sensitive Float methods; scale exponents remain arbitrary
  precision `Int` values.
- `meanList` is removed from the public facade. `mean` and `logSumExp` consume
  `Iterable<Float>` inputs; `logSumExp` remains one-pass and constant-space.
- `logAddExp` handles NaN and infinities explicitly. `log1mexp` returns NaN
  outside its real domain and uses `ln1p`/`expm1` branches near cancellation.
- `log1pExp` and the proposal's P3 integer additions are deferred.

## Implementation tasks

| Task | Work | State |
| --- | --- | --- |
| T1 | Move the special-function sources into `packages/numerics`; migrate probability imports and the existing special-function test clients | COMPLETE |
| T2 | Add Float-owned fused multiply-add, sign-copy, and integer power-of-two scaling; move Complex helper uses onto Float | COMPLETE |
| T3 | Add stable log-domain operations, make `logSumExp` generic/streaming, remove `meanList`, and update statistics delegation | COMPLETE |
| T4 | Regenerate native surface records and run focused Rust compilation checks | COMPLETE |

## Verification

- Native surface generation completed with 427 declarations.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core -p phalcom-modules -p phalcom-native-surface` passed.
- Phalcom package/runtime acceptance and numerical behavior matrices were not
  run. This plan is complete for the requested source edits, not for numerical
  certification.

## Residual work

The checkpoint handoff identifies the focused source acceptance that remains.
Do not classify this plan as `FOCUSED_TESTED` based on Rust compilation or a
clean Git tree.
