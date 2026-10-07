# 4. Exact arithmetic: Int, Rational, conversion, and algorithms

## 4.1 `Int`

The recommended public contract is already reflected by the current tree and
accepted decisions:

- all integer literals denote `Int`;
- `Int` is exact and unbounded subject to the configured numeric resource policy;
- small values may be immediate, large values may be `LargeInt`, but both report
  the same public class;
- `+`, `-`, `*`, `~/`, shifts, and nonnegative integer powers preserve exactness;
- `/` produces `Float` by explicit language rule;
- `Int.new(Float)` is not an implicit narrowing door; use `floor`, `ceil`,
  `truncated`, `rounded`, or `toIntExact` according to intent.

The existing `normalize_bigint` path and the exact large-integer fixtures are the
right implementation seam. New exact types should reuse that semantic policy,
not duplicate integer storage rules.

## 4.2 `Rational`

### Recommended shape

`Rational` should be a nominal, transparent, shallowly immutable data value with
semantic components conceptually equivalent to:

```text
Rational { numerator: Int, denominator: Int }
```

The stored invariant is:

```text
denominator != 0
denominator > 0
gcd(abs(numerator), denominator) == 1
Rational(0, d) == Rational(0, 1)
```

The physical fields need not be exposed as a stable layout. A constructor must
normalize before a value becomes observable. This follows the data-object rule
that semantic state is defined by declared components and representation is free.

### Generic or concrete?

Do **not** make public `Rational<T>` in V1. Telos's current `Int` is already an
arbitrary-precision surface type, so `Rational<Int>` adds no user-facing choice
while exposing implementation strategy. A future fixed-width or custom integer
library may introduce a separate generic exact ratio family if it can state its
overflow and normalization laws. That future family should conform to the same
capabilities, not redefine `Rational`.

### Operations

V1 Rational operations should include:

```text
Rational.new(numerator: Int, denominator: Int) -> Rational
Rational.fromInt(value: Int) -> Rational
numerator -> Int
inverse -> Rational        // raises for zero
abs -> Rational
sign -> Int
+ - * /                   // Rational-preserving where defined
compare -> Ordering        // total
== != hash                 // normalized value semantics
toFloat -> Float           // explicit, approximate
```

Use an exact rational division route, not the built-in `/` rule. If `q / 0` is
called, raise the existing divide-by-zero error family. If a future API needs a
non-throwing result, it should return the existing `Result` shape explicitly.

### Formatting and hashing

Canonical text should be `numerator/denominator` except that denominator `1` may
render as the integer form if the project adopts that convention. Hashing must be
based on normalized mathematical value. `Rational(2, 4)` and `Rational(1, 2)`
must compare and hash alike. A Rational equal to an Int should have the same
numeric hash if Telos's numeric-key contract keeps cross-domain numeric equality.

## 4.3 Exactness-preserving conversions

| Conversion | Recommendation |
|---|---|
| `Int -> Rational` | Exact, explicit or constructor-specific; denominator `1` |
| `Rational -> Int` | Only `toIntExact`; raises for non-integral values; explicit `floor`/`ceil`/`truncated` for narrowing |
| `Int -> Float` | Explicit `Float.new`; binary64 rounding is accepted and documented |
| `Float -> Rational` | Defer general conversion; if added, name it `fromFloatExact` and define that it captures the exact dyadic binary64 value, not the decimal spelling |
| `Rational -> Float` | Explicit `toFloat`; binary64 approximation and overflow behavior documented |
| `Complex<T> -> T` | No implicit conversion; `real`/`imag` accessors |

`Rational.fromFloatExact(0.1)` would mean the exact binary64 dyadic value of
`0.1`, which surprises users expecting `1/10`. Therefore it should not be a
casual constructor. A future `fromDecimal` parser can provide decimal intent.

## 4.4 Exact integer utilities

Keep these in `universe.math` or a closely related exact-numeric child because
they are useful to Rational and future probability packages:

### V1 foundation

```text

`gcd`, `lcm`, and `isqrt` already have source-authored implementations and
fixtures. Their laws are recorded in the current math audit. `powInt` can be a
small source-level wrapper over existing `**` if a distinct name adds clarity;
avoid duplicate semantics. `modPow`, factorial, combinations, permutations,
primality, and factorization need resource/algorithm design and are not required
for the first public math foundation.

### Numerically robust helpers

Also foundational, but not all must be in the first package release:

- Euclidean quotient/remainder variants with explicit names;
- `fma` on Float;
- compensated or pairwise summation for Float;
- `ilog2`/bit-length and exact power-of-two predicates;
- `nextUp`, `nextDown`, `ulp`, `copySign` on Float;
- `isClose` with an explicit tolerance policy.

These are better owned by `Int`/`Float` protocols where they describe one scalar
representation, not dumped into `math` as aliases.
