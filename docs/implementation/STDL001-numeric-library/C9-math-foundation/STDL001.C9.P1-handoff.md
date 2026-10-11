# STDL001.C9.P1 handoff

## Current state

`STDL001.C9.P1` is `COMPLETE / IMPLEMENTED / UNVERIFIED`. The code and package
boundary edits are present. Rust compilation and native surface generation
passed, but source-language package/API behavior remains unverified.

## Stable takeover map

- Optional package: `packages/numerics/project.toml`, root source, and
  `src/special/{package,kernel}.ph`.
- Universe base math: `phalcom-core/core/universe/src/math/{package,reduction}.ph`.
- Float source/native API: `phalcom-core/core/universe/src/scalar/number.ph` and
  `phalcom-core/src/primitive/math.rs`.
- Complex Float helper consumers: `phalcom-core/core/universe/src/numbers/float_complex.ph`.
- Probability package dependency/imports: `packages/probability/project.toml`
  and its `special` consumers.
- Migrated special-function test clients:
  `phalcom-core/tests/core/audit_probes/math.rs`.

## Next focused work

1. Compile and run a client that depends on `numerics` and imports
   `numerics.special`; exercise representative gamma/beta functions.
2. Exercise `Float.fma`, `Float.copySign`, and `Float.scalePow2` at signed-zero,
   subnormal, overflow, and large-`Int` exponent boundaries.
3. Exercise `logAddExp`, `log1mexp`, `logSumExp` with an iterator, and the
   statistics `mean` delegation, including the documented NaN/infinity/domain
   cases.
4. Run the migrated special-function reference and probability source-client
   tests if the focused package checks pass.

The numerical behavior must be measured before promoting the checkpoint to
`FOCUSED_TESTED`. `log1pExp` and P3 integer additions remain deferred.
