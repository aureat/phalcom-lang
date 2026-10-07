# 3. Semantic model and numerical ontology

## 3.1 What is a number?

Telos should use several distinct meanings rather than one overloaded taxonomy.

### Runtime category

`Number` remains the abstract built-in runtime class above `Int` and `Float`.
It supports common dynamic dispatch and diagnostics. It does not assert that a
value is ordered, exact, finite, field-like, or suitable for a specific
algorithm. The current `Number` primitive surface in
`phalcom-core/src/primitive/number.rs` is already a dynamic operation boundary,
not a static generic law.

### Concrete value type

The initial concrete built-ins are:

| Name | Meaning |
|---|---|
| `Int` | Exact mathematical integer, arbitrary precision at the surface; current implementation uses immediate plus `LargeInt` tiers |
| `Float` | IEEE-754 binary64 value, including NaN, infinities, signed zero, subnormals |
| `Rational` | Proposed exact normalized quotient of two nonzero-denominator `Int` values |
| `Complex<T>` | Proposed exact pair of components of one component type, with operation-specific requirements |

`Real` should **not** be a concrete runtime type in Phase 1. It is a useful
mathematical category for documentation and future constraints, but it conflates
exact rationals and approximate binary64 values if made nominal too early.

### Capability abstraction

The useful generic vocabulary is behavioral:

- `Additive<T>` or `Additive` — zero and addition;
- `AdditiveGroup<T>` — additive inverse/subtraction;
- `Multiplicative<T>` — one and multiplication;
- `RingLike<T>` — the combination needed by exact polynomial-style algorithms;
- `Division<T>` — division with a declared result policy;
- `Ordered<T>` — comparison returning `Ordering` or an explicitly partial result;
- `ExactInteger<T>` — divisibility, gcd/lcm, shifts, and integer powers;
- `RealElementary<T>` — finite/approximate elementary functions, if a backend can promise them;
- `Conjugate<T>` / `Norm<T>` — the smallest complex-related capabilities.

These are proposed traits/capabilities, not a required hierarchy. A type can
conform to several independent capabilities without inheriting one from another.

### Semantic property

`Exact`, `Approximate`, `Finite`, `NaNCapable`, and `Ordered` should initially be
documentation-level and capability-level facts. They should not become marker
classes that users test through runtime class identity. An arbitrary third-party
decimal can be exact for some operations and approximate for others; a marker
class cannot express that nuance.

## 3.2 Why not a numeric tower?

Three candidate models were considered:

### A. Nominal tower (`Number` → `Complex` → `Real` → `Rational` → `Int`)

**Benefit:** familiar names and runtime tests.

**Failure:** inheritance says too much. Complex is not ordered; Float is not
mathematically Real; Rational's exact division does not share Float's result
semantics; and a class hierarchy cannot express operation-specific promotion.

### B. Structural “has these methods” numeric values

**Benefit:** flexible third-party participation.

**Failure:** current Telos traits are nominal and explicit. Structural coincidence
does not establish conformance (`traits.md:418-453`). Reconstructing structural

### C. Nominal concrete values plus independent capability traits — **recommended**

`Int`, `Float`, `Rational`, and `Complex<T>` own representation and semantic

## 3.3 Exact and approximate are not supertypes

Recommended laws:

- `Int` operations remain exact until an explicitly Float-producing operation is
  selected.
- `Rational` operations remain exact and normalized.
- `Float` operations follow binary64; they may overflow, underflow, produce NaN,
  or lose precision.
- `Complex<T>` inherits the exactness and special-value behavior of `T` for
  component arithmetic, but complex elementary operations add their own backend
  contract.

Thus `Rational` can be exact while `Rational.toFloat` is approximate, and
`Complex<Rational>` can be exact for `+`, `-`, `*`, and `/` when the denominator
is nonzero, while `Complex<Float>` is approximate.

## 3.4 Runtime versus static meaning

Current Phalcom has both dynamic dispatch and a growing semantic/type product
layer. The design must keep these separate:

- runtime class identity tells the VM how to dispatch;
- generic bounds and trait evidence tell semantic analysis which members/laws are
  available;
- static proofs may establish exactness or bounds for an algorithm but do not
  create runtime classes;
- representation tiers such as `LargeInt` must remain hidden behind `Int`.

The generic numerical layer should consume canonical trait/evidence products,
not inspect runtime tags in user-level generic code.
