# Complex numerical backend

`kernel.ph` is source-authored Phalcom code adapted from CPython 3.13.9's
`Modules/cmathmodule.c` (commit `8183fa5e3f78ca6ab862de7fb8b14f3d929421e0`).
The adaptation keeps CPython's scaled finite algorithms and signed-zero branch
selection while changing the boundary contract to Phalcom:

- explicit `Complex<Float>` inputs are required;
- no Python `ValueError`/`OverflowError` wrappers are introduced;
- poles, zero divisors, infinities, and indeterminate inputs produce IEEE
  component values;
- captured Families expose the public surface from `package.ph`;
- final transcendental bits are backend-dependent within the documented finite
  accuracy tolerance.

The public branch contract is in `docs/specs/complex-numbers.md`:
negative-real cuts use the sign of the imaginary zero, inverse functions retain
their own cuts, and phase preserves both signed `-pi` and `+pi` endpoints.

`log1p` is a Phalcom extension: it uses a stable near-zero correction and keeps
zero input component signs. Integer powers use source exponentiation by squaring;
the separate principal complex power composes the logarithm and exponential.
The scalar-pair division helper in `numbers/float_complex.ph` uses separately
normalized products rather than CPython's direct division formula.

Finite regression fixtures are independent mpmath 1.4.1 references for exact
binary64 inputs, rounded once after 100–700 decimal digits of evaluation. The
component comparison tolerance is 8e-15 relative plus four minimum subnormals;
zero signs and infinity signs are compared exactly and NaNs by classification.
This is focused acceptance evidence, not a global correctly-rounded guarantee.
