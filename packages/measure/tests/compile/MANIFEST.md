# Compile probes

## Pass

- `const-data-runtime-call.ph` — static data materialization preserves behavior.
- `semantic-product.ph` — associated output produces `Quantity<Length,Rational>`.
- `exact-unit-observation.ph` — exact observation is kind-constrained.
- `affine-difference.ph` — point subtraction projects `PointKind::Difference`.

## Fail

- `energy-plus-torque.ph` — same dimension does not erase semantic kind.
- `affine-unit-composition.ph` — affine units do not conform to UnitLike.
- `point-plus-point.ph` — affine points are not addable.
- `wrong-unit-observation.ph` — unit observation is kind-safe.
- `unsupported-semantic-product.ph` — missing semantic algebra is a type error.

- `const-prefix-runtime-call.ph` — package const Prefix materializes with behavior.
- `absorbed-plus-equivalent-dose.ph` — same dimension, distinct dose kinds.
- `frequency-plus-radioactivity.ph` — same inverse-time dimension, distinct kinds.
