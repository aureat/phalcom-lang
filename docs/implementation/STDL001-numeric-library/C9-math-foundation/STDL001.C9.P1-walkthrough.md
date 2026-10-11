# STDL001.C9.P1 walkthrough

## Delivered

- Moved the gamma, beta, error-function, inverse-tail, and log-combinatorics
  implementation into `numerics.special`; its kernel calls public
  `universe.math` functions and imports math-owned numerical errors.
- Removed `math.special` from the canonical Universe module graph. Consolidated
  statistics and probability into sibling `numerics.statistics` and
  `numerics.probability` modules, and migrated their special/statistics imports
  and core audit clients to the shared `numerics` project.
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
