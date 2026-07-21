# Float protocol

**Status:** Normative. Ratified by [PDR-0027](../../../pdr/0027-float-protocol-and-explicit-narrowing.md).
This document completes the Float surface left open by PDR-0025. The tower rules remain in
[numeric-tower.md](numeric-tower.md); parsing, rendering, and errors are in
[text-and-errors.md](text-and-errors.md).

## 1. Binary64 model

`Float` is IEEE-754 binary64. `+`, `-`, `*`, `/`, `%`, and Float-taking `**` perform the host
binary64 operation. No operation silently uses a fused multiply-add. Overflow produces signed
infinity where IEEE requires it; underflow may produce a subnormal or signed zero; signed zero is
preserved by arithmetic where IEEE preserves it. Platform implementations must use IEEE binary64;
they may differ only in unobservable NaN payload bits.

## 2. Equality, order, and keys

Public comparison uses IEEE comparison.

| expression | result |
|---|---|
| `NaN == NaN` | `false` |
| `NaN != NaN` | `true` |
| ordered comparison with `NaN` | `false` |
| `+0.0 == -0.0` | `true` |
| finite/infinite ordered comparisons | IEEE numeric order (`-Infinity < finite < Infinity`) |

`NaN` has no public ordering and no public total-order selector. `Int` and a finite integral
`Float` compare equal exactly when their mathematical values agree, including values beyond
`2^53`; conversion must use the represented binary64 value, not decimal spelling.

`Map` and `Set` use an internal numeric-key relation: all NaNs compare as the same key, signed
zeroes compare as the same key, and equal Int/Float values compare as the same key. Hashing follows
that relation. It does not alter `==`.

## 3. Protocol and exact narrowing

| Selector | `Int` | finite `Float` | non-finite `Float` |
|---|---|---|---|
| `abs` | exact `Int` magnitude | `Float` IEEE absolute value | IEEE result |
| `sign` | `Int` -1/0/1 | `Int` -1/0/1 | ±Infinity → ±1; NaN raises `#nonFiniteNumber` |
| `floor` | self | exact greatest `Int` | raises `#nonFiniteNumber` |
| `ceil` | self | exact least `Int` | raises `#nonFiniteNumber` |
| `truncated` | self | exact toward-zero `Int` | raises `#nonFiniteNumber` |
| `rounded` | self | exact nearest `Int`, ties away from zero | raises `#nonFiniteNumber` |
| `isInteger` | true | finite and fractionless | false |
| `isNaN` | false | IEEE `is_nan` | true only for NaN |
| `isFinite` | true | IEEE `is_finite` | false |
| `isInfinite` | false | IEEE `is_infinite` | true only for ±Infinity |

`-0.0.sign == 0`; `(-0.0).isInteger` is true. Narrowing converts directly from binary64 to
`BigInt`, then through `normalize(BigInt)`, never through `i64`; `1.0e300.floor` may therefore be
`LargeInt`. `rounded(x)` is `floor(x + 0.5)` for nonnegative x and `ceil(x - 0.5)` otherwise.

## 4. Power

`**` is selector `**(_)`, parsed right-associatively with Python's unary asymmetry.

```phalcom
-2 ** 2      // -(2 ** 2) == -4
2 ** -2      // 2 ** (-2) == 0.25
2 ** 3 ** 2  // 2 ** (3 ** 2)
```

`Int ** Int` with a nonnegative exponent is exact and normalizes to `Int` / `LargeInt`. A negative
Int exponent produces `Float`. If either operand is Float, use binary64 power. `0 ** negative`
raises `#divideByZero`; other binary64 domain outcomes, including a negative base to a nonintegral
Float exponent, are IEEE NaN.

## 5. Hash contract

`hash` returns `Int` only. For finite integral Float values, hash the exact integer value at every
magnitude, not only through the safe-integer range. `+0.0` and `-0.0` hash alike. All NaNs hash
alike for keyed collections; infinities have stable distinct hashes. A user-defined `hash` that
returns anything except Int raises `#invalidHash` when a keyed collection consumes it.
