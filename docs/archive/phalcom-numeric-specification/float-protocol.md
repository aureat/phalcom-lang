# Float Protocol and Binary64 Semantics

> **Status:** Normative except the explicitly named open selector spellings and special-case tables.
>
> **Decision set:** NUM-004 through NUM-015.
>
> **Related:** [`numeric-tower.md`](numeric-tower.md), [`text-and-errors.md`](text-and-errors.md), [`open-decisions.md`](open-decisions.md).

## 1. Representation and exact interpretation

`Float` is IEEE-754 binary64.

A finite Float is interpreted exactly from its sign, exponent, and significand bits. Conceptually:

```text
exactValue(f) = sign × significand × 2^exponent
```

This exact dyadic value is used for mixed comparison, exact floor division, numeric-key equality, and canonical hashing. Decimal source spelling is irrelevant after parsing.

Basic Float-domain `+`, `-`, `*`, and `/` use binary64 round-to-nearest, ties-to-even. No operation silently contracts into fused multiply-add unless a future operation explicitly requests FMA semantics.

Underflow may produce a subnormal or signed zero. Signed zero is preserved where the specified binary64 operation preserves it.

## 2. Classification and public comparison

| Expression or selector | Result |
|---|---|
| `NaN == NaN` | `false` |
| `NaN != NaN` | `true` |
| Any ordinary ordered comparison involving NaN | `false` |
| `+0.0 == -0.0` | `true` |
| `isNaN` | true only for NaN |
| `isFinite` | true only for finite values, including signed zero and subnormals |
| `isInfinite` | true only for positive or negative infinity |
| `isInteger` | true only for finite fractionless Float, including both zeroes |

Finite mixed Int/Float comparison uses exact mathematical values. It must remain correct beyond `2^53`.

## 3. Protocol table

| Selector | `Int` behavior | Finite Float behavior | NaN | ±Infinity |
|---|---|---|---|---|
| `abs` | exact Int magnitude | Float absolute value; `abs(-0.0)` is `+0.0` | NaN | `Infinity` |
| `sign` | Int `-1`, `0`, or `1` | Int `-1`, `0`, or `1`; both zeroes return `0` | `#nonFiniteNumber` | Int `-1` or `1` |
| `floor` | self | greatest exact Int not greater than receiver | `#nonFiniteNumber` | `#nonFiniteNumber` |
| `ceil` | self | least exact Int not less than receiver | `#nonFiniteNumber` | `#nonFiniteNumber` |
| `truncated` | self | exact toward-zero Int | `#nonFiniteNumber` | `#nonFiniteNumber` |
| `rounded` | self | nearest exact Int, ties to even | `#nonFiniteNumber` | `#nonFiniteNumber` |
| exact-integral conversion selector | self | exact Int iff fractionless | `#numericConversion` | `#numericConversion` |
| `isInteger` | true | finite and fractionless | false | false |
| `isNaN` | false | false | true | false |
| `isFinite` | true | true | false | false |
| `isInfinite` | false | false | false | true |

The public spelling of the exact-integral conversion selector remains open under **OD-NUM-001**. This document uses “exact-integral conversion selector” descriptively, not as source syntax.

## 4. Exact narrowing algorithms

`floor`, `ceil`, `truncated`, `rounded`, and exact-integral conversion must decode binary64 directly into an arbitrary-precision integer operation. They must not narrow through `i64`.

Consequently, a finite value such as `1.0e300` may narrow to an Int whose runtime representation is large.

### 4.1 Ties-to-even rounding

For finite `x`, `rounded` selects the nearest integer. If `x` lies exactly halfway between two integers, it selects the even integer.

Examples:

```phalcom
2.5.rounded       // 2
3.5.rounded       // 4
(-2.5).rounded    // -2
(-3.5).rounded    // -4
```

Implementations must not use `floor(x + 0.5)` or `ceil(x - 0.5)` because those encode ties-away and may also introduce binary64 addition rounding.

## 5. Signed zero

Signed zero is observable through canonical rendering and total ordering, but not through public equality or numeric-key equality.

Required behavior:

```phalcom
-0.0 == 0.0             // true
(-0.0).sign             // 0
(-0.0).isInteger        // true
(-0.0).toString         // "-0.0"
```

Arithmetic follows the specified binary64 operation's signed-zero behavior unless this specification gives a stronger rule. Float `%` produces zero with the divisor's sign.

## 6. NaN

NaN remains unordered and non-reflexive under public equality. NaN payload bits are not exposed by the ordinary numeric protocol.

Map and Set treat all NaNs as one equivalent numeric key. All NaNs therefore feed the same canonical numeric hash input.

A future bit-inspection or serialization API may preserve payloads without changing public numeric equality or key behavior.

## 7. Float-domain remainder

Float `%` is a floor remainder, not host `fmod`.

For finite nonzero operands, after any required Int-to-Float conversion:

```text
q       = floor(exactValue(a) / exactValue(b))
r_exact = exactValue(a) - q × exactValue(b)
result  = correctlyRoundedBinary64(r_exact)
```

A nonzero result has the divisor's sign. A zero result carries the divisor's sign.

A Float zero divisor produces NaN. The remaining infinity/NaN cases must be finalized in the special-case table under **OD-NUM-003**.

## 8. Power

### 8.1 General domains

- Int base and nonnegative Int exponent: exact Int power.
- Int base and negative Int exponent: Float result.
- Any Float operand: Float result.
- Numeric zero base with negative numeric exponent: `#divideByZero` before Float-domain power.

An Int exponent remains semantically an exact integer exponent even when it does not fit a host integer or finite Float. Implementations may use exponentiation by squaring, parity checks, magnitude short-circuits, and resource policy; they must not silently change an Int exponent into an approximate Float merely for convenience.

### 8.2 Accuracy

For ordinary finite inputs whose mathematical real result is defined and finite, Float power must produce a binary64 result within one ULP of the correctly rounded value.

Platform implementations may differ within that bound. The previous claim that differences are restricted to NaN payloads is rejected.

### 8.3 Required special-case table

A normative table must settle at least:

- `NaN ** 0`;
- `1 ** NaN` and `(-1) ** ±Infinity`;
- `±0.0` with positive/negative, odd/even, integral/nonintegral exponents;
- `±Infinity` with finite and infinite exponents;
- negative finite base with nonintegral Float exponent;
- result sign for negative zero and negative infinity;
- overflow, underflow, and NaN production.

The table remains open under **OD-NUM-003**. Until it is ratified, host `pow` output is not a substitute for the missing language decision.

## 9. Total order

A separate total-order operation shall exist. It must be deterministic and suitable for sorting values that include NaN and signed zero.

Ordinary comparisons remain unchanged.

Still open:

- public selector spelling (**OD-NUM-001**);
- ordering of NaNs, NaN payloads, signed zeroes, and equal Int/Float representations (**OD-NUM-002**).

An implementation must not expose a provisional ordering as stable language behavior.

## 10. Hash and key obligations

For Float:

- every finite integral value hashes like the exact equal Int, regardless of magnitude;
- `+0.0` and `-0.0` hash alike;
- all NaNs hash alike for numeric keys;
- positive and negative infinity have distinct canonical hash inputs;
- equal ordinary finite Float values hash alike.

The hash model is specified at the numeric-tower level; exact constants and seed policy remain open under **OD-NUM-004**.

## 11. Conformance boundaries

Tests must include:

- every binary64 class: normal, subnormal, signed zero, infinity, quiet/signaling NaN bit patterns where constructible;
- exact mixed comparison at and around every integer precision boundary;
- narrowing beyond `i64` and beyond `2^53`;
- all ties-to-even quadrants;
- signed-zero rendering and remainder;
- one-ULP power checks against a higher-precision oracle;
- the complete ratified power/remainder special-case tables once OD-NUM-003 closes.
