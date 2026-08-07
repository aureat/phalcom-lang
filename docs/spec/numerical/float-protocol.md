# Float Protocol and Binary64 Semantics

This document defines all Float-specific behavior over IEEE-754 binary64, including exact interpretation, narrowing, signed zero, NaN, remainder, power, and total comparison.

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

| Selector     | `Int` behavior        | Finite Float behavior                         | NaN                  | ±Infinity            |
| ------------ | --------------------- | --------------------------------------------- | -------------------- | -------------------- |
| `abs`        | exact Int magnitude   | Float absolute value; `abs(-0.0)` is `+0.0`   | NaN                  | `Infinity`           |
| `sign`       | Int `-1`, `0`, or `1` | Int `-1`, `0`, or `1`; both zeroes return `0` | `#nonFiniteNumber`   | Int `-1` or `1`      |
| `floor`      | self                  | greatest exact Int not greater than receiver  | `#nonFiniteNumber`   | `#nonFiniteNumber`   |
| `ceil`       | self                  | least exact Int not less than receiver        | `#nonFiniteNumber`   | `#nonFiniteNumber`   |
| `truncated`  | self                  | exact toward-zero Int                         | `#nonFiniteNumber`   | `#nonFiniteNumber`   |
| `rounded`    | self                  | nearest exact Int, ties to even               | `#nonFiniteNumber`   | `#nonFiniteNumber`   |
| `toIntExact` | self                  | exact Int iff fractionless                    | `#numericConversion` | `#numericConversion` |
| `isInteger`  | true                  | finite and fractionless                       | false                | false                |
| `isNaN`      | false                 | false                                         | true                 | false                |
| `isFinite`   | true                  | true                                          | false                | false                |
| `isInfinite` | false                 | false                                         | false                | true                 |

`toIntExact` is the public exact-integral conversion getter.

## 4. Exact narrowing algorithms

`floor`, `ceil`, `truncated`, `rounded`, and `toIntExact` must decode binary64 directly into an arbitrary-precision integer operation. They must not narrow through `i64`.

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

Signed zero is observable through canonical rendering and raw binary64-preserving interfaces, but public equality, numeric-key equality, and `totalCompare` merge both zero signs.

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

The exact result is rounded once, ties to even. A nonzero result has the divisor's sign. A zero result carries the divisor's sign, including a zero created by underflow. The rounded result may equal the divisor in magnitude.

| Dividend | Divisor | Result |
|---|---|---|
| finite nonzero | `±0.0` | NaN |
| `±0.0` | finite nonzero | zero with divisor sign |
| finite | `±Infinity` | NaN |
| `±Infinity` | finite | NaN |
| `±Infinity` | `±Infinity` | NaN |
| NaN | any | NaN |
| any | NaN | NaN |

No Float-domain row raises. A host `fmod` implementation, even with sign correction, is not normative.

## 8. Power

### 8.1 Result domain and precedence

- Int base with nonnegative Int exponent uses the exact Int path.
- Int base with negative Int exponent uses the Float path.
- Any Float operand uses the Float path.
- The exact Int path returns Int; every identity or special value in the Float path returns Float.

The following rules are applied in order:

1. `x ** 0` returns the unit value of the selected result domain, including when `x` is NaN or infinite.
2. `1 ** y` returns the unit value of the selected result domain, including when `y` is NaN.
3. A remaining NaN operand produces NaN.
4. Every numeric zero base with a negative numeric exponent raises `#divideByZero`, including exponent `-Infinity`.

An Int exponent retains exact integrality and parity without conversion to Float or a host-width integer. A finite Float exponent is integral exactly when its encoded binary64 value is an integer; parity is derived from that exact value.

### 8.2 Positive finite nonzero bases

For a finite exponent, compute the real power.

For infinite exponents:

| Base | `+Infinity` exponent | `-Infinity` exponent |
|---|---|---|
| `0 < base < 1` | `+0.0` | `+Infinity` |
| `base == 1` | `1.0` | `1.0` |
| `base > 1` | `+Infinity` | `+0.0` |

### 8.3 Negative finite nonzero bases

A finite negative base requires a finite integral exponent. A finite nonintegral Float exponent or either infinite exponent produces NaN.

For an integral exponent, odd parity produces a negative result and even parity produces a positive result.

### 8.4 Zero bases

For a positive exponent, a zero base produces zero. The result is `-0.0` only when the base is `-0.0` and the exponent is a finite odd integral value; otherwise it is `+0.0`.

For exponent zero, the result is the unit value of the selected result domain. Every negative exponent raises `#divideByZero`.

### 8.5 Infinite bases

| Base | Exponent | Result |
|---|---|---|
| `+Infinity` | positive finite or `+Infinity` | `+Infinity` |
| `+Infinity` | negative finite or `-Infinity` | `+0.0` |
| `+Infinity` | zero | `1.0` |
| `-Infinity` | positive finite odd integer | `-Infinity` |
| `-Infinity` | positive finite even integer | `+Infinity` |
| `-Infinity` | negative finite odd integer | `-0.0` |
| `-Infinity` | negative finite even integer | `+0.0` |
| `-Infinity` | zero | `1.0` |
| `-Infinity` | finite nonintegral or either infinity | NaN |

### 8.6 Overflow, underflow, and accuracy

Overflow and underflow carry a negative sign only for a negative base with an odd integral exponent. All other overflow and underflow results are positive.

For ordinary finite real results, the implementation must be within one ULP of the correctly rounded binary64 result. A host `pow` routine is non-normative and may be called only after this dispatch table is enforced.

## 9. Total order

`totalCompare(_)` returns `Ordering.less`, `Ordering.equal`, or `Ordering.greater`.

```text
-Infinity
< finite numeric values by exact mathematical value
< +Infinity
< NaN
```

`-0.0`, Int zero, and `+0.0` compare equal. All NaNs compare equal; NaN sign, payload, and signaling state are ignored. Equal Int and Float values receive no representation tie-breaker.

The operation is deterministic across platforms and matches numeric-key equivalence. Ordinary comparisons retain IEEE NaN behavior.

## 10. Hash and key obligations

For Float:

- every finite integral value hashes like the exact equal Int, regardless of magnitude;
- nonintegral finite values hash from their complete reduced exact dyadic rational;
- `+0.0` and `-0.0` map to canonical zero;
- every NaN maps to one canonical private token;
- positive and negative infinity map to distinct private tokens.

The public built-in hash is a VM-seeded nonnegative 64-bit Int and is not stable across VM runs. Collection bucket placement applies an additional per-collection keyed finalizer.

## 11. Conformance boundaries

Tests must include:

- every binary64 class: normal, subnormal, signed zero, infinity, and diverse NaN bit patterns;
- exact mixed comparison around every integer precision boundary;
- narrowing beyond `i64` and beyond `2^53`;
- all ties-to-even quadrants;
- signed-zero rendering, remainder, power, overflow, and underflow;
- every row of the remainder and power tables;
- one-ULP power checks against a higher-precision oracle;
- total-order transitivity and raw-bit-pattern property tests.

