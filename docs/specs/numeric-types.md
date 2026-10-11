# Standard Numeric Types

This specification records source-visible contracts for the exact `Rational`
type and the binary64-specific additions to `Float`. It composes with
[`complex-numbers.md`](complex-numbers.md), which owns Complex arithmetic.

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

`nextUp` and `nextDown` return the adjacent representable binary64 value toward
positive and negative infinity. NaN remains NaN; stepping outward from the
matching infinity leaves it unchanged. Stepping from either zero produces the
smallest subnormal value with the requested sign.

`ulp` returns the representable spacing at the magnitude of its receiver. It
returns the smallest positive subnormal for either zero, positive infinity for
either infinity, and NaN for NaN. At the largest finite magnitude it uses the
spacing to the preceding finite value.

`totalCompare(other)` is separate from numeric `compare` and relational
operators. It provides the IEEE total order of binary64 values, distinguishing
signed zero and ordering negative NaNs before negative infinity and positive
NaNs after positive infinity. NaN payload bits participate in ordering when
present; the language does not promise that arithmetic preserves a payload.

The class-side constants `epsilon`, `minPositiveSubnormal`, `minNormal`, and
`maxFinite` expose binary64 machine metadata. They are exact binary64 constants,
not approximations computed at runtime.
