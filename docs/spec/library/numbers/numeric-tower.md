# The Numeric Tower: `Int`, `Float`, and `Number`

This document defines the public numeric types and all representation-independent numeric semantics. Runtime layout and intrinsic execution are specified in the [numeric runtime implementation plan](../../../implementation/roadmap/numbers-runtime-implementation.md).

## 1. Tower and value domains

```text
        Object
          │
       Number
        ╱   ╲
      Int   Float
```

### 1.1 `Number`

`Number` is a visible, allocator-abstract, closed kernel class.

- `Number` has no direct instances.
- Every direct, inherited, reflective, or shared allocation path targeting `Number` raises `#abstractClass`.
- Abstractness affects allocation only; normal class and selector reflection remains available.
- `Number` may own representation-independent derived protocol.
- Representation-sensitive operations are implemented for the concrete built-in classes.
- User-defined classes cannot subclass `Number` or register as members of the built-in numeric tower.

### 1.2 `Int`

`Int` denotes the mathematical integers.

- Values are exact.
- Magnitude is unbounded except for the active numeric resource policy.
- Immediate and heap-backed representations are private.
- Crossing a private representation boundary does not change class, equality, hashing, rendering, selector behavior, or reflection.

### 1.3 `Float`

`Float` denotes every IEEE-754 binary64 value: normal and subnormal finite values, both signed zeroes, both infinities, and NaNs.

The exact mathematical value of a finite Float is the dyadic rational encoded by its bits. Source spelling and rendered decimal text do not define its value.

### 1.4 Closed tower and ordinary user arithmetic

User-defined numeric-like classes inherit from `Object` or another user class and implement ordinary selectors. They do not participate in built-in promotion, exact mixed comparison, total numeric comparison, numeric-key equality, or canonical numeric hashing.

For an unsupported intrinsic operand tuple, execution falls back to the ordinary receiver send. Thus `custom + 3` may be implemented by `custom`, while `3 + custom` follows `Int#+(_)` and performs no reverse-operation or coercion hook.

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
| `<`, `<=`, `>`, `>=` | exact order | exact mathematical order, or false for NaN | No lossy mixed comparison. |
| `==`, `!=` | exact equality | exact mathematical equality except IEEE NaN behavior | Symmetric across operand order. |
| `totalCompare(_)` | `Ordering` | `Ordering` | Total preorder over Number instances; see §8.3. |
| `negated` | exact `Int` | IEEE Float negation | Private fixed-width boundaries are unobservable. |
| `floor`, `ceil`, `truncated`, `rounded`, `toIntExact` | self | exact `Int` or specified error | No narrowing through a host fixed-width integer. |
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

No `~/=` compound-assignment form is introduced.

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

The remainder is zero or has the divisor's sign:

```phalcom
 7 %  2 ==  1
-7 %  2 ==  1
 7 % -2 == -1
-7 % -2 == -1
```

The laws are:

```text
a == (a ~/ b) * b + (a % b)
abs(a % b) < abs(b)
```

An implementation must not inherit truncating host remainder semantics or always-nonnegative Euclidean remainder semantics.

### 6.2 Float-domain remainder

If either operand is Float, any Int operand first undergoes ordinary Float-domain conversion. For finite converted operands `a` and nonzero `b`:

```text
q       = floor(exactValue(a) / exactValue(b))
r_exact = exactValue(a) - q * exactValue(b)
a % b   = correctlyRoundedBinary64(r_exact)
```

The exact remainder is rounded once, ties to even. A nonzero result has the divisor's sign. A zero result carries the divisor's sign. The rounded result may equal the divisor in magnitude even though the exact remainder does not.

Special cases:

| Dividend | Divisor | Result |
|---|---|---|
| finite nonzero | `±0.0` | NaN |
| `±0.0` | finite nonzero | zero with the divisor's sign |
| finite | `±Infinity` | NaN |
| `±Infinity` | finite | NaN |
| `±Infinity` | `±Infinity` | NaN |
| NaN | any | NaN |
| any | NaN | NaN |

Float-domain remainder raises no divide-by-zero or non-finite error. Host `fmod` plus a sign patch is not normative.

## 7. Power

### 7.1 Syntax

`**` is selector `**(_)`. It is right-associative and has Python-like unary asymmetry:

```phalcom
-2 ** 2      // -(2 ** 2)
2 ** -2      // 2 ** (-2)
2 ** 3 ** 2  // 2 ** (3 ** 2)
```

### 7.2 Exact Int power

An Int base with a nonnegative Int exponent returns exact Int. The implementation preflights the active resource policy and does not rely on allocator failure for a predictably excessive result.

### 7.3 Float-result power

A negative Int exponent or any Float operand produces Float. The complete special-value behavior is defined in [`float-protocol.md`](float-protocol.md).

The following identities take precedence. They return Int `1` on the exact Int path and Float `1.0` on the Float path:

```text
x ** 0 = unit
1 ** y = unit
```

After those identities, any remaining NaN operand produces NaN. Every numeric zero base with a negative numeric exponent raises `#divideByZero`, including exponent `-Infinity`.

A negative finite nonzero base requires a finite integral exponent; otherwise the result is NaN. Exact Int exponents retain exact integrality and parity without conversion to Float or a host-width integer.

For ordinary finite real results, the Float result must be within one ULP of the correctly rounded value. Host `pow` is an implementation component only after the normative dispatch table is applied.

## 8. Equality and order

### 8.1 Public equality

Public equality is numeric across Int and Float:

```phalcom
1 == 1.0
9007199254740992 == 9007199254740992.0
9007199254740993 != 9007199254740992.0
```

Mixed equality compares exact represented values and must not convert the Int to Float.

Float special values follow IEEE public equality:

```phalcom
NaN == NaN      // false
NaN != NaN      // true
+0.0 == -0.0    // true
```

### 8.2 Ordered comparison

Finite Int and Float values compare by exact mathematical value. Infinities order outside finite values:

```text
-Infinity < every finite numeric value < Infinity
```

Every ordinary ordered comparison involving NaN returns false.

### 8.3 `totalCompare(_)`

`Number#totalCompare(other)` returns exactly one canonical singleton:

```text
Ordering.less
Ordering.equal
Ordering.greater
```

Its order is:

```text
-Infinity
< finite numeric values by exact mathematical value
< +Infinity
< NaN
```

Additional rules:

- Mathematically equal Int and Float values compare `Ordering.equal`.
- `-0.0`, Int zero, and `+0.0` compare equal.
- Every NaN compares equal to every NaN.
- NaN sign, payload, and signaling status are ignored.
- No representation tie-breaker is applied.

A non-Number argument raises the standard type-domain error expecting `Number`.

The operation is a total preorder over concrete Number instances and a total order over numeric-key equivalence classes. Its equality result coincides with numeric-key equality, not public `==` for NaN.

## 9. Numeric keys and hashing

### 9.1 Numeric-key equality

Map and Set use a numeric-key relation distinct from public equality. It merges:

- equal finite Int and Float values at every magnitude;
- Int zero, `+0.0`, and `-0.0`;
- every NaN with every other NaN.

It distinguishes positive and negative infinity and otherwise follows exact finite numeric equality.

### 9.2 Representative preservation

When insertion finds an equivalent existing Map key, the existing key and iteration position are retained and only the associated value is replaced. Set insertion likewise retains the first representative and position.

Thus inserting `1` and then `1.0` leaves `1` as the iterated representative.

### 9.3 Canonical mathematical hash input

Numeric hashing is defined over an exact canonical numeric key:

- Int `i` maps to reduced rational `i/1`.
- A finite Float maps to its exact reduced dyadic rational `m/n`, with `n > 0` and `gcd(|m|, n) = 1`.
- Both signed zeroes map to `0/1`.
- Every NaN maps to one private NaN token.
- Positive and negative infinity map to distinct private tokens.

The complete arbitrary-precision numerator and denominator are consumed. There is no textual, host-width, fixed-prime, or fixed-width pre-reduction.

The public `hash` getter returns a nonnegative Int in `0..<2**64` for built-in values. Built-in hashes are VM-seeded and stable only within one VM instance. They are not persistent identifiers.

A user-defined `hash` may return any signed arbitrary-precision Int. Collections consume its sign, significant length, and every magnitude bit; truncation to a host integer is forbidden. A non-Int result raises `#invalidHash`.

Map and Set apply a separate per-collection keyed finalizer before bucket placement. Hash collisions never imply equality; key equivalence is checked after a hash match.

Non-numeric user keys remain outside numeric canonicalization. A user-defined `hash` must remain stable while the value is stored as a key, and values considered equivalent by the collection must produce compatible hashes. A class that bases equality and hashing on mutable state is responsible for not mutating that state while keyed.

## 10. Conversion and Number protocol

### 10.1 Constructors

```phalcom
Int.new()       // 0
Float.new()     // 0.0
```

`Number.new` always raises `#abstractClass` through the shared allocation guard.

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
- `toIntExact`.

`Number#toIntExact` is getter-style. On Int it returns the receiver. On Float it converts the exact binary64 mathematical value only when finite and fractionless. Both signed zeroes produce Int zero. Fractional values, infinities, and NaNs raise `#numericConversion`.

All narrowing operations decode binary64 directly and can return arbitrary-precision Int.

## 11. Integer-only boundaries

Indices, sizes, arities, loop counters, shift counts, bit indexes, and every API requiring an integer quantity accept Int, not merely a numerically integral value. APIs that report such quantities return Int.

```phalcom
list.at(2)       // valid
list.at(2.0)     // type error
```

This is a type-domain rule, not an exact-conversion request. Callers must narrow explicitly before crossing the boundary.

String indexing follows the same boundary: `String#at(_)` requires Int, indexes Unicode scalar values rather than UTF-8 bytes, and returns `Option<String>` containing a one-scalar String when present. `String#size` counts Unicode scalars. Raw byte access is an explicit separate protocol.

## 12. Reflection and kernel closure

The following are required:

```phalcom
1.class == Int
1.0.class == Float
Int.isA(Number)
Float.isA(Number)
```

Large Int representation is never reflected as a distinct class.

`Number`, `Int`, and `Float` expose their methods to ordinary reflection, selector literals, explicit `perform`, method objects, and super sends during core construction. After kernel bootstrap, their method dictionaries and superclass relationships are immutable and cannot be changed by user code or reflection.

Ordinary numeric syntax may therefore execute an observationally equivalent closed-kernel intrinsic without performing method lookup. Explicit reflective invocation still uses the reflected method.

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

This specification does not add Decimal, Rational, Complex, fixed-width integers, unsigned integers, implicit user-defined numeric coercion, reverse numeric operators, a public canonical-exact-value protocol, persistent hash values, or a public `LargeInt` class. No placeholder selector names are reserved for future numeric extension.
