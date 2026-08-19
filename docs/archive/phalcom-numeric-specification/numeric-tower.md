# The Numeric Tower: `Int`, `Float`, and `Number`

> **Status:** Normative.
>
> **Decision set:** NUM-001 through NUM-015, NUM-020 through NUM-022.
>
> **Scope:** Public numeric classes, values, arithmetic, conversion, comparison, division, remainder, keys, hashing obligations, reflection, and integer-only boundaries. Runtime layout and landing mechanics are specified separately in [`implementation.md`](implementation.md).

## 1. Tower and value domains

```text
        Object
          │
       Number
        ╱   ╲
      Int   Float
```

### 1.1 `Number`

`Number` is a real, visible, allocator-abstract class.

- `Number` has no direct instances.
- Every direct, inherited, reflective, or otherwise shared allocation path targeting `Number` raises `#abstractClass`.
- Abstractness affects allocation only. It does not hide the class, remove inherited protocol, or disable ordinary selector reflection.
- `Number` may define representation-independent derived methods in Phalcom code.
- Representation-sensitive arithmetic is owned by `Int` and `Float`, not concealed in one shared VM primitive on `Number`.

### 1.2 `Int`

`Int` denotes the mathematical integers.

- Values are exact.
- Magnitude is unbounded subject only to configured resource policy.
- Small and large runtime representations are unobservable.
- Crossing an implementation representation boundary does not change `.class`, equality, hashing, rendering, dispatch, or reflection.

### 1.3 `Float`

`Float` denotes IEEE-754 binary64 values:

- finite normal values;
- finite subnormal values;
- positive and negative zero;
- positive and negative infinity;
- NaN values.

The exact mathematical value of a finite Float is the dyadic rational encoded by its binary64 bits, not its source spelling and not the shortest decimal used to render it.

## 2. Core terminology

For this specification:

- `exactValue(i: Int)` is the corresponding mathematical integer.
- `exactValue(f: finite Float)` is the exact dyadic rational represented by `f`.
- A **Float-domain operation** returns Float and may round according to binary64.
- An **exact operation** must not convert an arbitrary Int to Float as an intermediate approximation.
- **Numeric equality** is the public `==` relation over Number values.
- **Numeric-key equality** is the internal Map/Set relation defined in §9.

## 3. Selector result matrix

| Operation | `Int`, `Int` | At least one `Float` | Notes |
|---|---:|---:|---|
| `+`, `-`, `*` | exact `Int` | `Float` | Mixed arithmetic converts Int operands to finite binary64 or raises `#numericOverflow`. |
| `/` | `Float` | `Float` | Always Float-domain division. |
| `~/` | exact `Int` | exact `Int` | Floors exact represented values; never rounds an Int to Float. |
| `%` | exact `Int` | `Float` | Floor-remainder semantics in both domains. |
| `**` | exact `Int` for nonnegative Int exponent; otherwise Float | `Float` | Detailed in §7 and [`float-protocol.md`](float-protocol.md). |
| `<`, `<=`, `>`, `>=` | exact order | exact mathematical order or unordered for NaN | No lossy mixed comparison. |
| `==`, `!=` | exact equality | exact mathematical equality, except NaN follows IEEE public equality | Symmetric across operand order. |
| `negated` | exact `Int` | IEEE Float negation | `i64::MIN`-like implementation boundaries are not observable. |
| `hash` | `Int` | `Int` | Must agree with numeric-key equality. |
| `toString` | `String` | `String` | Canonical rendering is defined in [`text-and-errors.md`](text-and-errors.md). |

## 4. Arithmetic promotion

### 4.1 No universal promotion relation

There is no single rule that first coerces both operands and then serves every numeric operation. The operation family determines whether approximation is permitted.

Float-producing arithmetic may convert an Int to Float. Exact comparison, hashing, key equality, and `~/` may not.

### 4.2 Int-to-Float conversion used by arithmetic

A finite Int converted to Float is rounded to nearest binary64, ties to even.

- If the rounded result is finite, conversion succeeds.
- If the mathematical Int is outside the finite binary64 range, conversion raises `#numericOverflow`.
- Conversion must not silently turn a finite exact Int into infinity.

This rule applies to explicit `Float.new(Int)` and to implicit conversion required by Float-domain arithmetic.

### 4.3 Conversion failure ordering

Operands are evaluated before numeric dispatch according to the language's ordinary evaluation order. After evaluation, receiver conversion is attempted before argument conversion. A conversion failure occurs before the arithmetic operation is executed.

Implementations may optimize conversion only if they preserve the same observable error and span.

## 5. Division

### 5.1 `/`: Float division

`/(_)` always returns Float.

```phalcom
7 / 2       // 3.5
6 / 2       // 3.0
```

For Int operands, both operands are converted according to §4.2 before binary64 division.

After successful conversion, `/` follows binary64 division semantics, including:

```phalcom
1 / 0       // Infinity
-1 / 0      // -Infinity
0 / 0       // NaN
```

A finite Int too large for finite binary64 raises `#numericOverflow` before division. `/` does not use fused operations.

### 5.2 `~/`: exact floor division

For finite numeric operands `a` and `b`, where `b` is nonzero:

```text
a ~/ b = floor(exactValue(a) / exactValue(b))
```

The result is exact Int.

Consequences:

```phalcom
 7 ~/  2 ==  3
-7 ~/  2 == -4
 7 ~/ -2 == -4
-7 ~/ -2 ==  3
```

`~/` does not first perform rounded binary64 division. Mixed Int/Float `~/` therefore remains correct beyond `2^53`.

Errors:

- zero divisor: `#divideByZero`;
- NaN or infinity operand: `#nonFiniteNumber`;
- configured result/allocation limit exceeded: `#numericLimit`.

## 6. Remainder

### 6.1 Exact Int remainder

For Int `a` and nonzero Int `b`:

```text
q = a ~/ b
r = a - q * b
a % b = r
```

Therefore the remainder is zero or has the divisor's sign:

```phalcom
 7 %  2 ==  1
-7 %  2 ==  1
 7 % -2 == -1
-7 % -2 == -1
```

The following hold:

```text
a == (a ~/ b) * b + (a % b)
abs(a % b) < abs(b)
```

An implementation must not use a truncating host remainder without correction and must not use always-nonnegative Euclidean remainder when the divisor is negative.

### 6.2 Float-domain remainder

If either operand is Float, ordinary Float-domain conversion first applies to any Int operand. For finite converted operands `a` and nonzero `b`:

```text
q       = floor(exactValue(a) / exactValue(b))
r_exact = exactValue(a) - q * exactValue(b)
a % b   = correctlyRoundedBinary64(r_exact)
```

The zero result carries the divisor's sign. A nonzero result has the divisor's sign.

The exact relation is defined before final binary64 rounding. A Float arithmetic reconstruction may itself round and is not required to reproduce `a` bit-for-bit.

The non-finite special-case table is open under **OD-NUM-003**. A zero divisor yields NaN rather than a raised error once the operation is in the Float domain.

A named truncating remainder such as `fmod(_)` is outside this specification and requires measured demand.

## 7. Power

### 7.1 Syntax

`**` is selector `**(_)`. It is right-associative and has Python-like unary asymmetry:

```phalcom
-2 ** 2      // -(2 ** 2)
2 ** -2      // 2 ** (-2)
2 ** 3 ** 2  // 2 ** (3 ** 2)
```

### 7.2 Exact Int power

For Int base and nonnegative Int exponent:

```text
base ** exponent
```

returns exact Int, normalized across private representations.

The implementation must preflight configured resource policy. It must not attempt an allocation known to exceed policy and then rely on host allocation failure.

### 7.3 Float-result power

A negative Int exponent or any Float operand produces Float.

Every numeric zero base raised to a negative numeric exponent raises `#divideByZero`, including `+0.0` and `-0.0`.

For ordinary finite Float-domain results, the implementation must be within one ULP of the correctly rounded real result. It must not promise platform identity beyond the normative special cases and accuracy bound.

The complete NaN, infinity, signed-zero, negative-base, and integral-exponent table remains open under **OD-NUM-003**. Host `pow` behavior is not itself normative.

## 8. Equality and order

### 8.1 Public equality

Public equality is numeric across Int and Float:

```phalcom
1 == 1.0
9007199254740992 == 9007199254740992.0
9007199254740993 != 9007199254740992.0
```

Mixed equality compares exact represented values and must not be implemented as `int as Float == float`.

Float special values follow IEEE public equality:

```phalcom
NaN == NaN      // false
NaN != NaN      // true
+0.0 == -0.0    // true
```

Equality is symmetric across operand order.

### 8.2 Ordered comparison

Finite Int and finite Float values compare by exact mathematical value. Infinities order outside finite values:

```text
-Infinity < every finite numeric value < Infinity
```

Every ordinary ordered comparison involving NaN returns false.

### 8.3 Total order

Phalcom shall expose a separate total-order operation suitable for deterministic sorting, indexes, and serialization. Its public selector spelling and exact special-value sequence remain open under **OD-NUM-001** and **OD-NUM-002**.

Ordinary `<`, `<=`, `>`, `>=`, and `==` are not changed to satisfy total ordering.

## 9. Numeric keys and hashing

### 9.1 Numeric-key equality

Map and Set use a numeric-key relation distinct from public equality.

The relation merges:

- equal Int and finite integral Float values at every magnitude;
- `+0.0` and `-0.0`;
- every NaN with every other NaN.

It otherwise follows exact numeric equality for finite values and distinguishes positive and negative infinity.

### 9.2 Representative preservation

When insertion finds an equivalent existing Map key:

- the existing key object/value is retained;
- its iteration position is retained;
- only the associated value is replaced.

When adding an equivalent Set member, the first representative and its position are retained.

Thus inserting `1` and then `1.0` leaves `1` as the iterated representative.

### 9.3 Hash contract

`hash` returns Int only.

The numeric hash model must satisfy:

```text
numericKeyEqual(a, b) => numericHash(a) == numericHash(b)
```

Consequences:

- every finite integral Float hashes like the exact equal Int, including values beyond `2^53`;
- signed zeroes hash alike;
- all NaNs hash alike for keyed collections;
- positive and negative infinity have distinct canonical inputs to the hash model;
- future exact Rational or Decimal values can join the same model without breaking existing equality.

A user-defined `hash` may return any Int, including an Int larger than the immediate representation. A keyed collection must reduce the complete integer into its internal hash width. Every non-Int return raises `#invalidHash`.

The exact reduction algorithm and seed policy remain open under **OD-NUM-004**. Hash values are not persistent serialization identifiers.

## 10. Conversion and Number protocol

### 10.1 Constructors

```phalcom
Int.new()       // 0
Float.new()     // 0.0
```

`Number.new` always raises `#abstractClass` through the shared allocation guard.

Constructor conversion matrix:

| Argument | `Int.new(_)` | `Float.new(_)` |
|---|---|---|
| Int | identity | rounded finite binary64 or `#numericOverflow` |
| Float | always `#numericConversion` | identity |
| Bool | `#numericConversion` | `#numericConversion` |
| String | strict Int text parse | strict Float text parse |
| Other | `#numericConversion` | `#numericConversion` |

### 10.2 Explicit Float narrowing

Float narrowing is never hidden in `Int.new(Float)`.

The protocol includes:

- `floor`;
- `ceil`;
- `truncated`;
- `rounded`;
- one exact-integral conversion selector whose spelling remains open under **OD-NUM-001**.

The exact-integral conversion succeeds only when the receiver is finite and fractionless; otherwise it raises `#numericConversion` or the final ratified exact-conversion error kind.

## 11. Integer-only boundaries

Indices, sizes, arities, loop counters, shift counts, bit indexes, and every API requiring an integer quantity accept Int, not merely a numerically integral value.

```phalcom
list.at(2)       // valid
list.at(2.0)     // type error
```

This is a type-domain rule, not an exact-conversion request. Callers must narrow explicitly before crossing the boundary.

## 12. Reflection

The following are required:

```phalcom
1.class == Int
1.0.class == Float
Int.is(Number)
Float.is(Number)
```

Large Int representation is never reflected as a distinct class.

`Number`, `Int`, and `Float` participate in ordinary selector lookup, method definition, override, selector literals, super-sends, and reflection. Abstractness does not make inherited selectors disappear.

## 13. Required laws

Subject to ordinary error conditions and configured resource policy:

1. Int arithmetic is exact.
2. Numeric equality is symmetric.
3. Numeric-key equality is an equivalence relation.
4. Numeric-key equality implies equal numeric hashes.
5. For Int `b != 0`, `a == (a ~/ b) * b + (a % b)`.
6. For Int `b != 0`, `abs(a % b) < abs(b)`.
7. For finite `a`, nonzero finite `b`, `a ~/ b == floor(exactValue(a) / exactValue(b))`.
8. A value crossing from small to large Int and back is observationally identical to one that never crossed.

## 14. Non-goals

This specification does not add Decimal, Rational, Complex, fixed-width integers, unsigned integers, implicit user-defined numeric coercion, persistent hash values, or a public `LargeInt` class.
