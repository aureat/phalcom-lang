# Architecture

## Four distinct concepts

`measure` never conflates:

1. semantic kind,
2. structural dimension,
3. unit/coordinate mapping,
4. runtime numeric representation.

`Energy` and `Torque`, for example, share a dimension but are distinct kinds.

## Unit elimination

`km(5)` normalizes to a `Quantity<Length,Rational>` and does not retain `km`.
Use `Measure` when representation must survive.

## Structural vs semantic algebra

`kg * m / s**2` is structural `UnitExpr` algebra.

`mass * acceleration` is semantic quantity algebra and requires an explicit
`ProductKind<Acceleration>` relation for `Mass`.

There is deliberately no dimension -> semantic-kind inverse lookup.

## Prefixes and powered units

A Prefix scales the unit value it receives exactly once. It does not inspect a
unit's dimension and does not exponentiate itself automatically.

Therefore:

```telos
let cm2 = centi(m) ** 2
```

has scale `10^-4 m²`, while `centi(m2)` would mean a distinct area unit scaled
by `10^-2` relative to `m²`. This is intentional: prefix behavior stays local
and context-free.
