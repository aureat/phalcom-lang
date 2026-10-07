# 5. Float semantics and numerical robustness

## 5.1 Float means binary64, not mathematical Real

`Float` is a concrete IEEE-754 binary64 value. The current runtime stores its
bits directly (`value/repr.rs`) and current tests exercise NaN, infinity, signed
zero, subnormals, and domain results. The public contract should explicitly
expose this fact through classification and metadata, while avoiding promises
the backend cannot make.

Recommended classification surface:

```text
isNaN -> Bool
isFinite -> Bool
isInfinite -> Bool
isNormal -> Bool
isSubnormal -> Bool
isPositiveZero -> Bool
isNegativeZero -> Bool
classify -> FloatClass
```

`FloatClass` may be a closed data type with `Nan`, `NegativeInfinity`,
`NegativeNormal`, `NegativeSubnormal`, `NegativeZero`, `PositiveZero`,
`PositiveSubnormal`, `PositiveNormal`, and `PositiveInfinity`, or a smaller
predicate-only API if the project wants to minimize surface. A classification
value is more useful than making users reconstruct IEEE categories from several
predicates, but the exact enum is a V1 choice.

## 5.2 Equality, ordering, and hashing

Keep three relations distinct:

1. **Numeric equality (`==`)** — current numeric contract treats equal Int and
   integral Float values as equal; NaN is unequal to itself; `+0.0` and `-0.0`
   compare numerically equal.
2. **Exact sameness (`===`)** — current object/value semantics distinguish
   representation/sameness and do not perform numeric coercion. Signed zero and
   Float versus Int therefore must not be collapsed here.
3. **Total ordering** — needed for deterministic sorting and ordered maps. It must
   be an explicitly named operation, not an accidental reuse of `<`. A plausible
   IEEE total order puts negative NaNs before negative infinity and positive NaNs
   after positive infinity, with payload ordering specified only if Telos chooses
   to expose it. V1 can provide `totalCompare` only for Float after deciding the
   NaN ordering.

For numeric hashing, the invariant remains:

```text
a == b  =>  a.hash == b.hash
```

Finite integral Float values hash as equal Int values; both zero signs hash as
zero; NaNs follow one documented key policy. Rational must join this policy only
when its cross-domain equality is explicitly accepted.

## 5.3 Elementary functions

The existing Float-only `universe.math` surface is the correct V1 shape:

```text
sqrt cbrt hypot
exp exp2 expm1
ln ln1p log log2 log10
sin cos tan asin acos atan atan2
sinh cosh tanh asinh acosh atanh
```

The current native layer maps to Rust `f64` functions. The current math audit
correctly rejects a blanket one-ULP or cross-platform bitwise guarantee for
functions whose Rust backend documents unspecified precision. The public contract
should say:

- `sqrt` may inherit the backend's stronger correctly-rounded guarantee;
- other functions follow the supported binary64 backend and may vary in final
  bits across platform/toolchain versions;
- NaN payload bits are not part of the source-level contract;
- domain-invalid Float inputs generally produce IEEE results such as NaN or
  infinities rather than a new broad `MathDomainError` hierarchy.

## 5.4 Generic function families

Do not make `sqrt(x)` a single unconstrained generic function yet. There are
three distinct meanings:

1. **Float elementary function** — current `math.sqrt(Float) -> Float`.
2. **Exact algebraic square root** — may return a Rational, an exact algebraic
   value, or a complex value; no current foundation chooses the result type.
3. **Complex square root** — branch-sensitive and backend-specific.

Expose distinct families or namespaces until Telos has a generic associated
result mechanism that can express these differences. This is more predictable
than making `sqrt` accept `Number` and silently converting.

## 5.5 Special functions

`erf`, `erfc`, `gamma`, `logGamma`, and `beta` should not enter the first
foundational implementation merely because Python's `math` contains them. They
are valuable forward-compatibility requirements for probability and statistics,
but they require a backend dependency and accuracy/overflow contract. Design
their future ownership now as `universe.math.special` or a sibling package;
implement after the base backend policy is decided.
