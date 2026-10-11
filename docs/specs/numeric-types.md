# Standard Numeric Types

This specification records source-visible contracts for `Int`, exact
`Rational`, and the binary64-specific additions to `Float`. It composes with
[`complex-numbers.md`](complex-numbers.md), which owns Complex arithmetic.
The accepted integer and numeric-surface decisions are
[TDR-0022](../decisions/accepted/0022-numeric-surface-split-int-float-and-division.md),
[TDR-0064](../decisions/accepted/0064-numeric-tower-implementation-and-floor-amendment.md),
[TDR-0068](../decisions/accepted/0068-bitwise-operations-on-int.md),
[TDR-0069](../decisions/accepted/0069-numeric-tower-residue-rulings.md),
[TDR-0070](../decisions/accepted/0070-numeric-literals.md), and
[TDR-0071](../decisions/accepted/0071-float-protocol-and-explicit-narrowing.md).

## Int

`Number` is abstract and cannot be instantiated. `Int` and `Float` are its
concrete numeric types: exact, unbounded `Int` and IEEE-754 binary64 `Float`.
`Int` literals and integer-only operations preserve
exactness without machine-word wraparound; small and large runtime
representations do not change the public `Int` identity.

`Int` and `Float` are distinct types, but numeric equality compares their
mathematical values across the two representations: `1 == 1.0`. Their hashes
must agree whenever they compare equal. For `+`, `-`, and `*`, two `Int`
operands produce an exact `Int`; if either operand is `Float`, the operation is
in the `Float` domain and converting the `Int` operand may round.

`/` is true division and returns `Float`, including for two `Int` operands.
`~/` returns an exact `Int` using floor semantics for all `Int`/`Float` operand
combinations; zero divisors, non-finite operands, and non-finite quotients are
errors. Integer `%` uses floor-division remainder semantics; Float remainder
retains its Float contract.

There is no general implicit conversion between numeric types. `Float.new(Int)`
is an explicit widening request that follows binary64 rounding and may lose
precision. `Int.new(Float)` rejects Float arguments, including integral values;
Float-to-Int narrowing is performed by an explicitly named rounding operation.
`Int.new` accepts decimal integer text; `Float.new` accepts decimal Float text
and the exact special spellings `NaN`, `Infinity`, and `-Infinity`. Neither
constructor accepts surrounding whitespace or radix prefixes. Exact integer
operations do not silently pass through binary64.

`Int ** Int` is exact for nonnegative exponents and returns `Float` for negative
exponents. A Float operand selects binary64 power. Zero raised to a negative
exponent raises the divide-by-zero failure. Integer bitwise operations are
infinite two's-complement operations: `&`, `|`, `^`, unary `~`, `<<`, and `>>`;
named bit queries are `bitAt`, `bitCount`, `bitLength`, and `trailingZeros`.
Left shift is exact multiplication by a power of two; right shift is arithmetic
floor division by that power. `bitAt` is sign-aware, while `bitCount` and
`bitLength` use the magnitude. Negative shift counts and `trailingZeros` on
zero raise `ArgumentError`.

Keyed collections use a SameValueZero-style equivalence for numeric keys,
separate from ordinary equality: all NaNs share a key class, the two zero signs
share a key class, and numerically equal integral `Int` and `Float` values
share a key class. Their hashes agree. This relation is used by `Map` and
`Set`; it does not change Float `==` or provide a NaN ordering.

The standard integer algorithms exposed by `universe.math` are specified in
[`numeric-math.md`](numeric-math.md). The language-level rounding operations on
Float and exact `Rational` conversion are described below.

## Rational

`universe.numbers.Rational` is a concrete immutable ratio of arbitrary-precision
`Int` values. It is not a subtype of `Number`, and it does not implicitly
convert to or from `Int` or `Float`.

The data representation is private. `Rational.new(numerator, denominator)`
rejects a zero denominator with the same `#divideByZero` runtime failure as
integer exact division, moves any sign to the numerator, and divides both
components by their greatest common divisor. Zero has the canonical form
`0/1`; every nonzero value has a positive denominator and coprime components.
`Rational.fromInt(value)` constructs `value/1`.

Addition, subtraction, multiplication, division, negation, integer powers,
comparison, and equality operate on exact integer values. Division and inverse
reject zero. Equality and hashing use the canonical numerator/denominator, so
equivalent ratios compare equal and have equal hashes. Ordering is total.

`toIntExact` rejects nonintegral values. `floor`, `ceil`, and `truncated` are
explicit narrowing operations with their usual mathematical meanings: toward
negative infinity, toward positive infinity, and toward zero respectively.
`toFloat` explicitly approximates the ratio as binary64; it may round, overflow,
or underflow according to binary64 range and precision.

Rational conforms to the numeric capabilities required by generic Complex and
Quaternion arithmetic. This does not add automatic promotion between numeric
domains.

`Complex<Rational>` also provides exact division and integer powers. Its
division rejects an exact zero divisor; negative integer powers remain exact.

## Float representability and ordering

`Float` is IEEE-754 binary64. The predicates `isNormal`, `isSubnormal`,
`isPositiveZero`, and `isNegativeZero` distinguish the corresponding
representations; the two zero predicates are mutually exclusive. Existing
numeric equality continues to treat both zero signs as equal and NaN as unequal
to itself.

The public relational operators follow IEEE comparisons: ordered comparisons
involving NaN are false, `NaN != NaN`, and positive and negative zero compare
equal. NaN has no public ordering.

`floor`, `ceil`, `truncated`, and `rounded` explicitly convert a finite Float
to exact Int, respectively toward negative infinity, positive infinity, zero,
and to the nearest integer with ties away from zero. `toIntExact` succeeds only
for finite integral values. These narrowing operations reject NaN and
infinities. `isInteger`, `isNaN`, `isFinite`, and `isInfinite` expose the
corresponding classifications.

`nextUp` and `nextDown` return the adjacent representable binary64 value toward
positive and negative infinity. NaN remains NaN; stepping outward from the
matching infinity leaves it unchanged. Stepping from either zero produces the
smallest subnormal value with the requested sign.

`ulp` returns the representable spacing at the magnitude of its receiver. It
returns the smallest positive subnormal for either zero, positive infinity for
either infinity, and NaN for NaN. At the largest finite magnitude it uses the
spacing to the preceding finite value.

The class-side constants `epsilon`, `minPositiveSubnormal`, `minNormal`, and
`maxFinite` expose binary64 machine metadata. They are exact binary64 constants,
not approximations computed at runtime.

### Known source divergence

The live source currently exposes `Float.totalCompare`, while accepted
TDR-0071 specifies that NaN has no public ordering and adds no total-order
operation. This binding is a known source/spec divergence; it is not part of
the normative Float contract unless an accepted amendment resolves the conflict.
