# STDL001.C9.P1 walkthrough

## Delivered

- Moved the gamma, beta, error-function, inverse-tail, and log-combinatorics
  implementation into the optional `packages/numerics` project at
  `numerics.special`. The kernel now calls captured public `universe.math`
  functions and imports math-owned numerical errors.
- Removed `math.special` from the canonical Universe module graph. Probability
  now declares `numerics` as a path dependency and imports `numerics.special`.
  Existing special-function clients in the core audit test were migrated to
  compile with that explicit optional dependency.
- Added `Float.fma` using `f64::mul_add`, `Float.copySign` using IEEE sign-bit
  copying, and `Float.scalePow2` with bounded work for arbitrary-precision
  exponents. Complex scalar-pair arithmetic uses the Float methods instead of
  local sign/scaling copies.
- Added `math.logAddExp` and `math.log1mexp`; changed `math.logSumExp` to an
  incremental compensated reduction over `Iterable<Float>`; removed the
  separate public `meanList` implementation and routed statistics through the
  canonical `mean` reduction.
- Regenerated the native surface table.

## Evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root .` — passed; generated 427 primitive records.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core -p phalcom-modules -p phalcom-native-surface` — passed.
- No Phalcom source package or numerical behavior test was run for this plan.

## Status

Implementation is complete for the requested boundary/API edit. Verification
remains `UNVERIFIED` until the migrated package and new source APIs receive
focused Phalcom acceptance.
