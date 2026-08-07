# Phalcom Numeric Specification — Combined Edition

> **Generated:** 2026-07-29.
>
> This file concatenates the authoritative specification files for convenient review and download. Where navigation links differ, the individual files in the specification directory remain authoritative.


---

<!-- BEGIN README.md -->

# Phalcom Numeric Specification Set

> **Status:** Ratified architecture; normative consolidation dated **2026-07-29**.
>
> **Authority:** This set records the project-owner ratification of the numeric architecture developed during the review of the original numeric specifications. It supersedes conflicting language in the earlier `numeric-tower.md`, `float-protocol.md`, `numeric-literals.md`, `text-and-errors.md`, `bitwise.md`, and numeric index. Repository ADR/PDR status files must still be amended so the source tree reflects this ratification.
>
> **Version:** `NUMERIC-SPEC-2026-07-29`

## 1. Purpose

This directory defines Phalcom's numeric language contract, implementation architecture, conformance requirements, migration consequences, and unresolved decisions.

The set intentionally separates permanent language semantics from repository-specific implementation instructions. A source path, dependency version, primitive count, commit hash, or worktree condition cannot silently become part of the language contract.

## 2. Document map

| File | Status | Purpose |
|---|---|---|
| [`numeric-tower.md`](numeric-tower.md) | Normative | Public numeric types, arithmetic, comparison, division, remainder, conversion, keys, and laws. |
| [`float-protocol.md`](float-protocol.md) | Normative, with named open tables | Binary64 behavior, Float protocol, rounding, power architecture, NaN, signed zero, and total-order requirement. |
| [`numeric-literals.md`](numeric-literals.md) | Normative | Source literal grammar, candidate boundaries, classification, oversized constants, and compiler diagnostics. |
| [`text-and-errors.md`](text-and-errors.md) | Normative | Text constructors, canonical rendering, runtime error taxonomy, structured fields, and traceback rules. |
| [`bitwise.md`](bitwise.md) | Normative | Infinite-two's-complement Int operations, precedence, huge-count behavior, laws, and errors. |
| [`implementation.md`](implementation.md) | Implementation contract | Runtime representation, semantic kernel, constant pool, GC, hashing, resource controls, dispatch, primitive floor, and landing order. |
| [`conformance.md`](conformance.md) | Normative conformance contract | Reference model, test matrices, properties, edge corpus, differential rules, and ship gates. |
| [`migration.md`](migration.md) | Normative change inventory; release mechanism open | Breaking changes, compatibility consequences, source migration, and release requirements. |
| [`open-decisions.md`](open-decisions.md) | Open-decision register | The remaining unresolved names, algorithms, tables, defaults, and release choices. |
| [`amendment-map.md`](amendment-map.md) | Informative | Exact corrections and improvements relative to the original uploaded files. |
| [`NUMERIC-SPEC-COMBINED.md`](NUMERIC-SPEC-COMBINED.md) | Generated convenience edition | All documents concatenated in reading order. The individual files remain authoritative. |

## 3. Normative hierarchy

When documents overlap, apply this order:

1. A closed decision in this set overrides conflicting prose in an earlier numeric ADR, PDR, or specification until those records are amended.
2. `numeric-tower.md`, `float-protocol.md`, `numeric-literals.md`, `text-and-errors.md`, and `bitwise.md` define public semantics.
3. `conformance.md` defines what an implementation must prove.
4. `implementation.md` constrains architecture without exposing internal representation as public behavior.
5. `open-decisions.md` identifies deliberately unsettled points. An open item must not be inferred from examples or host-library behavior.
6. `migration.md` describes compatibility impact; the release mechanism remains open under **OD-NUM-010**.

The words **must**, **must not**, **shall**, and **shall not** are normative. **Should** records a strong default that may be departed from only with an explicit project decision. **May** grants permission.

## 4. Ratified decision index

| ID | Decision |
|---|---|
| NUM-001 | Public tower is `Number` with concrete `Int` and `Float`; `Int` is exact and unbounded. |
| NUM-002 | Small and large Int representations are private and canonicalized through one normalizer. |
| NUM-003 | Numeric behavior is implemented through operation-specific semantic paths, not one universal promotion helper. |
| NUM-004 | Float-producing mixed arithmetic may round; exact comparison, key equality, hashing, and `~/` may not round an Int to Float. |
| NUM-005 | Mixed Int/Float equality and order compare exact mathematical values. |
| NUM-006 | Finite Int-to-Float conversion rounds ties-to-even and raises on finite-range overflow. |
| NUM-007 | Float-to-Int conversion is explicit; `Int.new(Float)` always rejects. |
| NUM-008 | `/` always returns Float. |
| NUM-009 | `~/` returns exact Int and floors exact represented values. |
| NUM-010 | `%` follows floor-division semantics for Int and Float; a named `fmod` is deferred. |
| NUM-011 | Int nonnegative power is exact; Float-domain power has a special-case table and one-ULP finite-result bound. |
| NUM-012 | `rounded` uses nearest, ties to even. |
| NUM-013 | Public Float equality remains IEEE-like; a separate total-order operation shall exist. |
| NUM-014 | Map/Set numeric keys merge equal Int/Float values, signed zeroes, and all NaNs while preserving the first key representative. |
| NUM-015 | Numeric hashing is one coherent mathematical model and accepts arbitrary Int results from user-defined `hash`. |
| NUM-016 | Literal and constructor grammars share productions but remain separate entry grammars. |
| NUM-017 | Oversized integer constants are heap-independent compiler constants, not live VM object references. |
| NUM-018 | Numeric resource failures are deterministic policy failures, distinct at compile time and runtime. |
| NUM-019 | Int bitwise semantics are infinite two's complement; huge nonnegative counts remain semantically valid. |
| NUM-020 | Numeric operations remain ordinary selector dispatch; optimized paths are guarded and deoptimizable. |
| NUM-021 | Integer-only boundaries require Int; integral Float is not accepted. |
| NUM-022 | Runtime numeric failures are structured language `Error` values sent through the ordinary raise path. |
| NUM-023 | Conformance uses an independent mathematical model plus targeted differential oracles. |
| NUM-024 | The numeric specification is split into semantics, implementation, conformance, migration, and open decisions. |

## 5. Closed architectural invariants

The following are no longer open:

```text
1.class == Int
1.0.class == Float
Int.isA(Number)
Float.isA(Number)
Number is allocator-abstract
Bool is not numeric
Int is exact and unbounded
6 / 2 has class Float
~/ returns Int
Int and equal integral Float compare equal
Map and Set merge equal numeric keys
Float indices are rejected
```

## 6. Remaining decisions

No foundational tower decision remains open. The unresolved items are bounded and named:

- **Blocking public-semantics items:** OD-NUM-001 through OD-NUM-006.
- **Blocking implementation selections:** OD-NUM-007 through OD-NUM-009.
- **Blocking release-policy item:** OD-NUM-010.
- **Deferred extensibility item:** OD-NUM-011.

See [`open-decisions.md`](open-decisions.md). Open items must be resolved before the corresponding surface or subsystem ships; they do not reopen the ratified architecture.

## 7. Non-goals of this revision

This set does not introduce Decimal, Rational, Complex, fixed-width integer classes, unsigned integers, SIMD values, bitwise Float operations, implicit user-defined numeric coercions, or persistent hash values. It deliberately leaves a future numeric-extension protocol open.

<!-- END README.md -->


---

<!-- BEGIN numeric-tower.md -->

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
Int.isA(Number)
Float.isA(Number)
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

<!-- END numeric-tower.md -->


---

<!-- BEGIN float-protocol.md -->

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

<!-- END float-protocol.md -->


---

<!-- BEGIN numeric-literals.md -->

# Numeric Literals

> **Status:** Normative.
>
> **Decision set:** NUM-016 through NUM-018.
>
> **Scope:** Source-code literal grammar, lexical boundaries, token/AST classification, oversized integers, and compile-time policy diagnostics. Runtime String constructors are specified in [`text-and-errors.md`](text-and-errors.md).

## 1. Lexical productions

```ebnf
DIGIT        := "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
NZ-DIGIT     := "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
BIN-DIGIT    := "0" | "1" ;
OCT-DIGIT    := "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" ;
HEX-DIGIT    := DIGIT | "a" | "b" | "c" | "d" | "e" | "f"
                      | "A" | "B" | "C" | "D" | "E" | "F" ;

DEC-DIGITS   := DIGIT { DIGIT | "_" DIGIT } ;
BIN-DIGITS   := BIN-DIGIT { BIN-DIGIT | "_" BIN-DIGIT } ;
OCT-DIGITS   := OCT-DIGIT { OCT-DIGIT | "_" OCT-DIGIT } ;
HEX-DIGITS   := HEX-DIGIT { HEX-DIGIT | "_" HEX-DIGIT } ;

ZERO-INT     := "0" { "0" | "_" "0" } ;
NONZERO-INT  := NZ-DIGIT { DIGIT | "_" DIGIT } ;
DEC-INT      := ZERO-INT | NONZERO-INT ;
BIN-INT      := "0" ( "b" | "B" ) [ "_" ] BIN-DIGITS ;
OCT-INT      := "0" ( "o" | "O" ) [ "_" ] OCT-DIGITS ;
HEX-INT      := "0" ( "x" | "X" ) [ "_" ] HEX-DIGITS ;
INT          := DEC-INT | BIN-INT | OCT-INT | HEX-INT ;

EXPONENT     := ( "e" | "E" ) [ "+" | "-" ] DEC-DIGITS ;
FLOAT        := DEC-DIGITS "." DEC-DIGITS [ EXPONENT ]
              | "." DEC-DIGITS [ EXPONENT ]
              | DEC-DIGITS EXPONENT ;
```

`-` and `+` are not part of a source numeric literal. A leading sign is ordinary unary syntax.

## 2. Classification

- `INT` creates Int.
- `FLOAT` creates Float.
- An exponent always makes a Float, even when the mathematical value is integral.
- Integer magnitude is unlimited subject to compiler resource policy.
- A source Float is rounded directly to binary64, round-to-nearest ties-to-even.

Examples:

```phalcom
0
0_0_0
1_000_000
0b1101
0b_1101
0o755
0x_FF_A0_00
3.1415_9265
.25
2e10
6.02e-23
```

## 3. Leading zero rule

A decimal Int beginning with `0` may contain only additional zeroes and valid separators between zeroes.

Valid:

```text
0
00
0_0
00_00
```

Invalid:

```text
0123
00_10
```

This rule does not prohibit leading zeroes in Float forms such as `0123.0` or `00e2`.

## 4. Decimal-point and range boundary

Fractional digits are mandatory after a decimal point in a Float literal.

```phalcom
5.0          // Float
5e2          // Float
.25          // Float
5.toString   // Int receiver followed by ordinary send
5.e2         // Int receiver followed by ordinary send to selector e2
5..2         // Int, range token, Int
```

`5.` at end of input or before punctuation that cannot begin a selector is one malformed numeric candidate, not `Int(5)` followed by a free-standing dot.

A dot followed by an identifier-start character begins an ordinary send. No identifier is specially reinterpreted as a malformed exponent. Exponent syntax has no decimal point before `e`.

A dot followed by a digit begins a Float only when the dot is in a position where a primary expression may begin.

## 5. Separator rules

One underscore may appear only between two digits valid for the active radix. A radix prefix additionally permits one underscore immediately after the prefix.

Separators must not:

- be doubled;
- terminate a literal;
- touch a decimal point;
- touch an exponent marker;
- touch an exponent sign;
- precede the first digit except immediately after a radix prefix.

Examples rejected as one numeric candidate:

```text
1_
1__0
1_.0
1._0
1e_3
1e+_3
0x
0x_
```

## 6. Numeric-candidate boundary

The lexer must consume enough input to report one diagnostic for a malformed numeric candidate rather than emit a valid numeric prefix plus misleading trailing tokens.

### 6.1 Adjacent identifier characters

Without a token boundary, an identifier character immediately following a numeric candidate makes the complete adjacent candidate invalid.

Examples:

```text
1abc
1true
1n
7j
0xFFn
0xFFfoo
1.0n
```

The diagnostic covers the entire adjacent candidate.

### 6.2 Radix candidates

After `0b`, `0o`, or `0x`, the lexer consumes the contiguous alphanumeric/underscore candidate and validates it as one unit.

Thus these each produce one numeric-literal diagnostic:

```text
0b2
0bark
0o9
0xG
0xFFfoo
```

### 6.3 Exponent candidates

Once `e` or `E` is consumed as part of a decimal numeric candidate, an optional sign and at least one decimal digit are required.

These are one malformed candidate:

```text
1e
1e+
1e-
1efoo
1e+foo
```

### 6.4 Token boundaries

Whitespace, delimiters, operators, a dot-send boundary, and the range token terminate a valid candidate. No whitespace may occur inside a literal.

## 7. Invalid suffixes

`n`, `j`, and all other type-like suffixes are not part of Phalcom numeric syntax.

A future public BigInt, Decimal, Rational, or Complex literal requires a separate decision; suffix-like text is not reserved as valid syntax by this specification.

## 8. Token and AST contract

The front end preserves three literal classes:

```rust
Token::Int(i64)
Token::Float(f64)
Token::LargeInt { digits, radix }
```

The AST preserves the same trichotomy or an equivalent lossless representation.

Requirements:

- an Int literal must never round-trip through Float;
- oversized digits are normalized without separators;
- radix is preserved explicitly;
- Float token payload is the correctly rounded binary64 value;
- source spans cover the complete literal candidate.

Exact Rust type names are implementation details, but the information content is normative.

## 9. Oversized integer constants

An oversized integer literal is not allocated as a live VM object during front-end compilation.

Compiler output stores a heap-independent constant descriptor containing sign/magnitude information or equivalent normalized digits and radix. Runtime loading materializes it through the canonical Int normalization path.

The binary encoding remains open under **OD-NUM-008**.

## 10. Overflow, underflow, and policy limits

### 10.1 Float syntax

A syntax-valid Float literal:

- rounds to nearest binary64, ties to even;
- may become a subnormal;
- may underflow to signed zero;
- may overflow to signed infinity.

These are values, not lexical errors.

### 10.2 Int syntax

An Int literal remains exact. If it exceeds configured source numeric policy, the compiler emits `numeric.limit`, not `numeric.literal`.

### 10.3 Diagnostic distinction

- `numeric.literal`: malformed syntax or invalid digit/separator/candidate boundary.
- `numeric.limit`: syntax-valid literal rejected by configured resource policy.

## 11. Required source diagnostics

A numeric-literal diagnostic must provide:

- stable code (`numeric.literal` or `numeric.limit`);
- primary span covering the full candidate;
- radix or literal class when relevant;
- first offending byte offset relative to the candidate when meaningful;
- configured and actual size for `numeric.limit`.

Exact English prose is not normative.

## 12. Required tests

The conformance suite must cover:

- every valid form and prefix case;
- every legal separator boundary;
- all malformed examples in this document;
- `5.toString`, `5.e2`, and `5..2` disambiguation;
- identifier adjacency;
- exponent candidate consumption;
- `i64` minimum/maximum seams;
- exact oversized decimal, binary, octal, and hexadecimal round trips;
- Float overflow, underflow, subnormal, and ties-to-even parsing;
- source spans and offsets;
- constant-pool serialization and GC stress after materialization;
- low configured source-digit limits.

<!-- END numeric-literals.md -->


---

<!-- BEGIN text-and-errors.md -->

# Numeric Text, Rendering, and Errors

> **Status:** Normative.
>
> **Decision set:** NUM-006, NUM-007, NUM-016, NUM-018, NUM-022.
>
> **Related:** [`numeric-literals.md`](numeric-literals.md), [`numeric-tower.md`](numeric-tower.md), [`float-protocol.md`](float-protocol.md).

## 1. Constructor behavior

```phalcom
Int.new()       // 0
Float.new()     // 0.0
```

`Number.new` always raises `#abstractClass` through the non-overridable shared allocation guard.

| Argument | `Int.new(_)` | `Float.new(_)` |
|---|---|---|
| Int | identity | finite binary64 conversion or `#numericOverflow` |
| Float | always `#numericConversion` | identity |
| Bool | `#numericConversion` | `#numericConversion` |
| String | strict `INT-TEXT` | strict `FLOAT-TEXT` |
| Other | `#numericConversion` | `#numericConversion` |

`Int.new(Float)` rejects even an integral Float. Exact narrowing uses the explicit selector whose final spelling remains open under **OD-NUM-001**.

## 2. Shared text productions

```ebnf
DIGIT        := "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
NZ-DIGIT     := "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
DEC-DIGITS   := DIGIT { DIGIT | "_" DIGIT } ;
SIGN         := "+" | "-" ;
EXPONENT     := ( "e" | "E" ) [ SIGN ] DEC-DIGITS ;

ZERO-TEXT    := "0" { "0" | "_" "0" } ;
NONZERO-TEXT := NZ-DIGIT { DIGIT | "_" DIGIT } ;
UINT-TEXT    := ZERO-TEXT | NONZERO-TEXT ;
INT-TEXT     := [ SIGN ] UINT-TEXT ;

FINITE-FLOAT-TEXT := [ SIGN ] (
      DEC-DIGITS [ "." DEC-DIGITS ] [ EXPONENT ]
    | "." DEC-DIGITS [ EXPONENT ]
) ;

FLOAT-TEXT   := FINITE-FLOAT-TEXT
              | "Infinity"
              | "-Infinity"
              | "NaN" ;
```

Consequences:

- `Float.new("123")` is valid and produces `123.0`.
- `Int.new` is decimal-only.
- Text constructors do not accept `0x`, `0o`, or `0b` prefixes.
- Neither constructor trims whitespace.
- `+Infinity`, `+NaN`, `-NaN`, `nan`, and `inf` are rejected.
- No trailing type suffix is accepted.

## 3. Text conversion rounding

Finite Float text is converted directly to binary64 using round-to-nearest, ties to even.

- Overflow becomes signed infinity because the user explicitly requested a Float-domain value.
- Underflow may produce a subnormal or signed zero.
- Locale never affects parsing.

This differs intentionally from converting an existing finite Int to Float, where an out-of-range Int raises `#numericOverflow` rather than becoming infinity.

## 4. Numeric-text error offsets

Malformed text raises `#numericText` with a zero-based UTF-8 byte offset.

The offset is the first byte that prevents the entire input from matching the required grammar. If the failure is missing input at end-of-string, the offset equals the string byte length.

| Input | Target | Offset | Reason |
|---|---:|---:|---|
| `""` | Int or Float | `0` | expected first byte |
| `" 1"` | Int or Float | `0` | whitespace is not trimmed |
| `"1 "` | Int or Float | `1` | trailing byte |
| `"1_"` | Int or Float | `1` | separator requires following digit |
| `"1__0"` | Int or Float | `2` | second separator invalid |
| `"1e"` | Float | `2` | missing exponent digits at EOF |
| `"1e+"` | Float | `3` | missing exponent digits at EOF |
| `"+Infinity"` | Float | `0` | special value has no leading plus form |
| `"NaN0"` | Float | `3` | trailing byte |
| `"é"` | Int or Float | `0` | first UTF-8 byte is invalid |
| `"1é"` | Int or Float | `1` | first byte after valid prefix is invalid |

The primary source span covers the constructor argument expression. Implementations must not fabricate a source span inside the runtime String.

## 5. Int rendering

`Int.toString` returns:

- ungrouped base-10 digits;
- one leading `-` for negative values;
- no leading `+`;
- no exponent notation;
- no size cutoff;
- no representation-tier marker.

Examples:

```text
0
-1
9223372036854775808
1000000000000000000000000000000
```

## 6. Float rendering

`Float.toString` is deterministic and locale-independent.

### 6.1 Special values

```text
NaN
Infinity
-Infinity
0.0
-0.0
```

All NaN payloads render as `NaN`.

### 6.2 Finite values

Finite output is the shortest decimal that round-trips to the same binary64 value under the specified parser.

- Fixed notation is used when the scientific exponent `e` satisfies `-6 <= e <= 20`.
- Otherwise lowercase scientific notation is used.
- Scientific notation has no `+` sign in a positive exponent.
- An integral fixed result includes `.0` so the value remains visibly Float.
- The canonical tie-break between equal-length shortest candidates must be a standard, explicitly selected algorithm under **OD-NUM-009**; host default formatting is not normative.

Examples of required shape:

```text
1.0
0.000001
1e-7
100000000000000000000.0
1e21
```

Exact boundary outputs must be pinned by conformance fixtures after OD-NUM-009 selects the algorithm.

## 7. Error model

Every user-visible runtime numeric failure is a language `Error` value delivered through the ordinary raise/traceback path.

The stable contract is:

- error `kind`;
- structured fields;
- primary and secondary span rules;
- operation semantics.

Complete English messages are informative and may improve without a language-version change.

## 8. Stable numeric error taxonomy

| Condition | Kind | Required structured fields | Primary span |
|---|---|---|---|
| Exact `~/` or Int `%` by zero | `#divideByZero` | `operator` | operator token |
| Numeric zero to negative power | `#divideByZero` | `operator: #**` | `**` token |
| NaN/infinity in exact narrowing or `~/` | `#nonFiniteNumber` | `operation`, `valueClass` | receiver or operator |
| Int-to-Float finite-range overflow | `#numericOverflow` | `operation`, `targetType: Float` | conversion argument or operator |
| Rejected constructor/narrowing conversion | `#numericConversion` | `sourceType`, `targetType`, `operation` | constructor argument or receiver |
| Malformed numeric text | `#numericText` | `targetType`, `byteOffset` | constructor argument expression |
| Configured numeric policy exceeded | `#numericLimit` | `operation`, `limit`, `requested` where known | allocating operation |
| Negative or non-Int shift count | `#invalidShift` | `countType` or `count` | shift operator; count secondary span |
| Negative or non-Int bit index | `#invalidBitIndex` | `indexType` or `index` | call argument |
| `trailingZeros` of zero or analogous partial query | `#undefinedNumericOperation` | `operation`, `receiverType` | receiver/call |
| Allocation targeting Number | `#abstractClass` | `class: Number` | constructor/class expression |
| User hash returns non-Int | `#invalidHash` | `actualType` | keyed-operation key expression |

Ordinary type-domain failures, such as a Float index passed to a list, use the language's standard type error with expected type `Int`.

## 9. Informative message forms

Implementations should produce concise messages such as:

```text
cannot floor-divide by zero
cannot convert Int to finite Float: value is out of range
invalid Float text at byte 3
numeric power exceeds configured bit limit
shift count must be a non-negative Int
hash must return Int, got Float
```

These examples are not byte-for-byte compatibility requirements.

## 10. Traceback and source spans

If a failing frame has source and an instruction span, rendering must show the innermost source line and caret label.

If source or span is unavailable, rendering must show the structured Error and traceback without inventing a location.

This applies uniformly to:

- binary arithmetic;
- unary narrowing;
- constructors;
- shifts and bit queries;
- resource-policy guards;
- reflective allocation;
- keyed collection hash consumption.

No dedicated host exception path may bypass language Error construction.

## 11. Compiler diagnostics

Numeric source failures are not runtime Error values.

| Code | Meaning |
|---|---|
| `numeric.literal` | malformed numeric source candidate |
| `numeric.limit` | syntax-valid numeric source exceeds compiler policy |

Compiler diagnostics must carry source spans and structured metadata analogous to runtime errors where practical.

## 12. Reflection

`Number`, `Int`, and `Float` remain visible to normal reflection. `Number.respondsTo(#new)` may be true even though invoking allocation raises `#abstractClass`.

Abstractness is enforced in the allocation mechanism, not by deleting selectors or relying on an overridable method body.

<!-- END text-and-errors.md -->


---

<!-- BEGIN bitwise.md -->

# Bitwise Operations on `Int`

> **Status:** Normative.
>
> **Decision set:** NUM-019 through NUM-022.
>
> **Dependency:** The numeric tower, exact floor division, floor remainder, and numeric resource policy must land first.

## 1. Mathematical model

Int is exact and unbounded. Bitwise semantics use infinite two's complement:

- nonnegative Int values have infinitely many leading zero bits;
- negative Int values have infinitely many leading one bits.

For Int `x`, `y`, and nonnegative Int `n`:

```text
x << n == x * (2 ** n)
x >> n == x ~/ (2 ** n)
~x     == -x - 1
```

`&`, `|`, and `^` are defined pointwise over infinite sign-extended bit sequences.

## 2. Public selector surface

| Source | Selector | Result | Error conditions |
|---|---|---:|---|
| `x & y` | `&(_)` | Int | `y` is not Int |
| `x | y` | `|(_)` | Int | `y` is not Int |
| `x ^ y` | `^(_)` | Int | `y` is not Int |
| `~x` | `~()` | Int | none |
| `x << n` | `<<(_)` | Int | `n` not Int; negative `n`; `#numericLimit` |
| `x >> n` | `>>(_)` | Int | `n` not Int; negative `n` |
| `x.bitAt(i)` | `bitAt(_)` | Bool | `i` not Int; negative `i` |
| `x.bitCount` | `bitCount` | Int | none |
| `x.bitLength` | `bitLength` | Int | none |
| `x.trailingZeros` | `trailingZeros` | Int | `x == 0` |

All rows are ordinary dynamically dispatched selectors. Symbolic selectors are reflectable, definable, overridable, and super-sendable.

No named aliases such as `bitAnd`, `bitNot`, `shl`, or `shr` are introduced.

## 3. Query semantics

### 3.1 `bitAt(i)`

Returns the sign-extended bit at zero-based index `i`.

```phalcom
5.bitAt(0)          // true
5.bitAt(1)          // false
(-1).bitAt(1000)    // true
```

### 3.2 `bitLength`

Returns the number of bits required to represent `abs(x)` without a sign bit.

```phalcom
0.bitLength       // 0
1.bitLength       // 1
5.bitLength       // 3
(-5).bitLength    // 3
```

### 3.3 `bitCount`

Returns the count of one bits in the finite magnitude representation of `abs(x)`.

```phalcom
0.bitCount       // 0
5.bitCount       // 2
(-5).bitCount    // 2
```

It does not count the infinite leading ones of a negative value.

### 3.4 `trailingZeros`

Returns the exponent of the largest power of two dividing `abs(x)`. It is sign-independent.

```phalcom
8.trailingZeros       // 3
(-8).trailingZeros    // 3
```

Zero has no finite result and raises `#undefinedNumericOperation` with `operation: #trailingZeros`.

## 4. Huge counts and indexes

A nonnegative count or index is semantically valid even when it does not fit the host's pointer-sized integer.

### 4.1 Huge right shift

For a count larger than any significant magnitude bit:

```text
x >> huge == 0   when x >= 0
x >> huge == -1  when x < 0
```

An implementation must short-circuit these cases and must not reject the count merely because it cannot be represented as `usize`.

### 4.2 Huge `bitAt`

For an index larger than any magnitude bit:

```text
x.bitAt(huge) == false  when x >= 0
x.bitAt(huge) == true   when x < 0
```

### 4.3 Huge left shift

Left shift may require allocation proportional to the result bit length. It must preflight configured resource policy.

A policy violation raises `#numericLimit`; raw host allocation failure is not promised as a catchable language Error.

## 5. Syntax, tokens, and precedence

Where prefixes overlap, tokens use maximal munch:

```text
~/   before ~ and /
<<   before <
>>   before >
**   before *
```

`~` is prefix-only. Other bitwise operators are infix-only.

Complete relevant precedence, tight to loose:

```text
postfix sends/calls/indexing
**                              right-associative
prefix + - ~                    unary; see asymmetry below
* / % ~/
+ -
<< >>
&
^
|
comparison and equality
and
or
```

Power has unary asymmetry:

```phalcom
~2 ** 3      // ~(2 ** 3)
2 ** ~3      // 2 ** (~3)
-~2 ** 3     // -(~(2 ** 3))
```

All binary bitwise and shift levels are left-associative.

The precedence deliberately parses:

```phalcom
flags & mask == 0
```

as:

```phalcom
(flags & mask) == 0
```

No bitwise compound-assignment tokens are introduced.

## 6. Error contract

- Negative or non-Int shift count: `#invalidShift`.
- Negative or non-Int bit index: `#invalidBitIndex`.
- Left-shift resource-policy violation: `#numericLimit`.
- `trailingZeros` on zero: `#undefinedNumericOperation`.
- Non-Int ordinary operand: standard type error expecting Int.

Errors use the ordinary language Raise path and the source-span rules in [`text-and-errors.md`](text-and-errors.md).

## 7. Laws

For all Int `x`, `y` and nonnegative Int `n`, `m`, whenever every allocating operation completes within configured resource policy:

1. `(x & y) | (x ^ y) == x | y`.
2. `~(x & y) == (~x) | (~y)`.
3. `(x & y) + (x | y) == x + y`.
4. `x ^ y == (x | y) - (x & y)`.
5. `(x << n) << m == x << (n + m)`.
6. `(x << n) >> n == x`.
7. `~~x == x`.
8. `x >> n == x ~/ (2 ** n)`.
9. `x.bitAt(n)` equals the `n`th infinite-two's-complement bit.
10. Results normalize to the immediate Int representation whenever they fit its private range.

## 8. Implementation-floor decision

The public selector surface is ratified. The subset implemented as VM-blessed primitives remains open under **OD-NUM-006**.

The implementation must resolve that item and amend the frozen primitive census before U-BITWISE lands. Deriving a selector in Phalcom is acceptable only if asymptotic behavior and measured performance remain suitable for arbitrary-precision values.

## 9. Required conformance

Tests must cover:

- every operator and selector;
- positive, zero, and negative operands;
- every sign pair;
- immediate/large promotion and demotion seams;
- counts around `0`, word boundaries, representation boundaries, and values larger than `usize`;
- parser precedence and maximal munch;
- selector symbols, symbolic method definitions, overrides, and super-sends;
- error kinds and spans;
- resource-policy preflight;
- a Python differential oracle for shared infinite-two's-complement semantics;
- independent algebraic property tests.

## 10. Non-goals

Float bitwise operations, unsigned right shift, fixed-width wrapping, rotations, leading-zero counts without a width, bitwise Bytes bulk operations, and compound assignment are outside this specification.

A future width-bearing numeric family may define `Int32`, `UInt64`, wrapping arithmetic, logical right shift, rotations, and width-dependent zero counts without changing unbounded Int.

<!-- END bitwise.md -->


---

<!-- BEGIN implementation.md -->

# Numeric Runtime and Compiler Implementation Contract

> **Status:** Binding implementation architecture; repository paths and exact type names are non-normative examples.
>
> **Purpose:** Realize the public specifications without leaking representation tiers, duplicating semantic algorithms, or binding compiler artifacts to one VM heap.

## 1. Architectural requirements

The implementation must provide:

1. separate runtime Int and Float values;
2. one canonical Int normalization path;
3. operation-specific numeric semantic functions;
4. heap-independent large numeric constants;
5. exact mixed comparison and exact floor division;
6. coherent numeric-key equality and hashing;
7. deterministic numeric resource guards;
8. structured Error construction through one Raise path;
9. guarded, invalidatable fast paths only after semantics are complete.

## 2. Runtime value representation

A representative Rust shape is:

```rust
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    Obj(ObjRef),
}

pub enum Object {
    // ...
    LargeInt(BigInt),
}
```

Exact names are not normative. Required behavior is:

- immediate Int covers the common private range;
- LargeInt holds every other integer;
- both surface as class `Int`;
- Float stores binary64 exactly, including signed zero and NaN payload bits;
- the old flat numeric arm is removed rather than retained as a migration alias.

Removing the old arm is an intentional exhaustiveness tool. Semantic matches must be updated explicitly rather than hidden by wildcard arms.

## 3. Canonical Int normalization

All arithmetic, parsing, constant loading, deserialization, and FFI paths that construct an arbitrary-precision integer must call one function equivalent to:

```rust
fn normalize(value: BigInt) -> Value
```

Invariant:

```text
Value::Obj(LargeInt(x)) implies x is outside the immediate Int range.
```

No other code path may create a LargeInt runtime value.

Debug and test builds must assert this invariant. Deserialization and module loading are included; canonicalization is not limited to arithmetic results.

## 4. Heap and GC

`LargeInt` contains no VM object references and therefore traces no child `ObjRef`s. It still requires an explicit tracer arm.

A runtime large constant is rooted exactly like every other loaded constant. The compiler must not rely on a temporary compilation heap object remaining alive.

Whether `BigInt` is inline or boxed inside the object union is an implementation measurement. Confirm actual object size and arena slot policy; do not encode guessed library layout into the language specification.

## 5. Heap-independent constant pool

Compiler output must contain an abstract constant form, for example:

```rust
enum Constant {
    Int(i64),
    LargeInt {
        negative: bool,
        magnitude: Arc<[u8]>,
    },
    FloatBits(u64),
    // ...
}
```

Requirements:

- no live VM `ObjRef` in serialized/compiler-owned bytecode;
- Float constants preserve exact bits;
- large Int constants preserve exact value;
- module loading materializes large Int through `normalize`;
- loaded constants are rooted by the module/function constant owner;
- encoding is versioned.

Sign representation, endianness, deduplication, and sharing remain open under **OD-NUM-008**.

## 6. Numeric semantic kernel

Do not use one `Promoted` helper for all operations.

The implementation should expose internal functions conceptually equivalent to:

```rust
float_arithmetic(lhs, rhs, op)
exact_compare(lhs, rhs)
exact_floor_divide(lhs, rhs)
floor_remainder(lhs, rhs)
numeric_equal(lhs, rhs)
numeric_key_equal(lhs, rhs)
numeric_hash(value)
convert_int_to_float(value)
convert_float_to_int(value, mode)
```

These are semantic boundaries, not necessarily one module or one public API.

Every primitive and fast path must share these algorithms or prove bit-for-bit/exception-for-exception equivalence against them.

## 7. Small-Int fast path

Common Int/Int arithmetic must not allocate BigInt unconditionally.

For `+`, `-`, `*`, and negation:

1. attempt checked immediate arithmetic;
2. on overflow, promote operands to BigInt;
3. compute exactly;
4. call `normalize`.

Division, remainder, power, shifts, and conversions use operation-specific paths.

## 8. Exact Float decomposition

Implement one binary64 decoder returning an exact classification:

```text
FiniteDyadic {
    sign,
    integer_significand,
    binary_exponent
}
PositiveInfinity
NegativeInfinity
NaN
```

The decoder is reused by:

- exact Int/Float comparison;
- Float-to-Int narrowing;
- exact `~/`;
- Float floor remainder;
- integral-Float hash canonicalization;
- total order once OD-NUM-002 closes.

Do not independently rederive exponent/significand logic in each primitive.

## 9. Exact comparison algorithm

The comparator returns:

```text
Less | Equal | Greater | Unordered
```

Rules:

- Int/Int compares arbitrary-precision integers.
- Float/Float ordinary finite order may use binary64 comparison after classifying NaN and signed zero consistently.
- Int/finite Float compares the Int against the Float's exact dyadic value without converting the Int to Float.
- NaN returns `Unordered`.
- infinities order outside finite values.

Public predicates map `Unordered` to false except `!=`, which follows negated public equality.

## 10. Exact floor division algorithm

The general `~/` path converts finite values to an exact rational pair.

A practical representation is:

```text
Int              => numerator / 1
finite Float     => signed integer significand × 2^e
```

Normalize both operands to integer numerator and a power-of-two denominator, then compute mathematical floor quotient using arbitrary-precision integers.

The implementation must handle all sign pairs explicitly and must not rely on truncating host division semantics.

Optimization is permitted for:

- Int/Int immediate operands;
- powers of two;
- huge quotient short-circuits;
- exact integral Float values.

Every optimization must match the general exact model.

## 11. Floor remainder algorithms

### 11.1 Int remainder

Compute quotient using floor division and derive:

```text
r = a - q * b
```

This is correct for negative divisors. `rem_euclid` alone is not.

### 11.2 Float remainder

After Float-domain conversion of any Int operand:

1. decode both finite Floats exactly;
2. compute exact floor quotient;
3. compute exact dyadic remainder;
4. round once to binary64, ties to even;
5. force a zero result to carry the divisor's sign.

Non-finite dispatch follows the table to be ratified under OD-NUM-003.

## 12. Power implementation

Use separate paths:

- exact BigInt exponentiation for nonnegative Int exponent;
- binary64 integer-exponent algorithm when the exponent is Int but the result domain is Float;
- general Float exponent algorithm for Float exponent;
- explicit special-case dispatch before the generic library call;
- one-ULP conformance against a high-precision oracle.

An exact Int exponent must not be approximated to Float merely because it exceeds a host integer width. Resource policy may reject excessive computation deterministically.

A host `pow` may be an implementation component only after the runtime enforces the ratified special cases and accuracy contract.

## 13. Numeric hashing

The hash subsystem has two layers:

1. **Canonical numeric hash input**, which makes equal numeric keys identical across Int/Float representations.
2. **VM hash mixing**, which may use a per-run seed and internal width.

Required canonicalization:

- Int: exact arbitrary-precision value;
- finite Float: exact dyadic rational;
- integral Float: same canonical numeric value as equal Int;
- signed zero: one canonical zero;
- NaN: one canonical NaN token;
- infinities: distinct positive and negative tokens.

A user-defined `hash` result is an arbitrary Int. Consume every significant bit before or during reduction; accepting only immediate Int is a representation leak.

Algorithm constants and seed policy remain open under OD-NUM-004.

## 14. Map and Set integration

Map and Set must use the same numeric-key equality and hash canonicalization.

Insertion behavior:

- probe by canonical hash and numeric-key equality;
- if equivalent key exists, preserve stored key and insertion position;
- replace only Map value;
- retain existing Set representative.

Required internal tests must insert equivalent keys in both orders and inspect retrieval, deletion, size, iteration key class, signed-zero rendering, and NaN behavior.

## 15. Class and protocol layout

Representation-sensitive methods are installed independently on Int and Float:

```text
+ - * / % ~/ **
< <= > >=
negated
hash
toString
new and new(_), as applicable to class side
```

Number carries no shared VM arithmetic implementation. It may carry ordinary derived protocol where semantics are genuinely common.

The exact placement of classification/narrowing defaults should minimize primitive-floor growth without lying about implementation ownership. Any inherited Number method must be semantically correct for every future Number subtype or be documented as sealed to the kernel tower.

## 16. Primitive floor

Every new VM-blessed binding must be recorded in the frozen primitive census.

Requirements:

- preserve explicit removal and addition counts rather than hiding removals inside a net number;
- keep Number in the census as a zero-primitive tripwire where intended;
- add Int and Float to every core-class census/list;
- amend the governing floor decision in the same change;
- resolve bitwise primitive composition under OD-NUM-006 before U-BITWISE lands.

Live tests, not prose totals, are the source of record.

## 17. Integer-only boundaries

Every primitive accepting indexes, arities, counts, offsets, lengths, or loop counters must require Int.

Centralize extraction helpers:

```rust
expect_int(value)
expect_nonnegative_int(value)
expect_index(value, bound)
```

Do not retain an integral-Float compatibility arm. Conversion is explicit at the call site.

A nonnegative Int larger than host address space is still an Int; boundary helpers distinguish:

- semantic type validity;
- collection bounds;
- host-size representability;
- configured resource policy.

They must not report “expected Int” when the actual problem is out-of-range Int.

## 18. Numeric resource policy

Per-VM policy must support at least:

```text
maxSourceNumericDigits
maxTextConversionDigits
maxIntegerBits
maxNumericAllocationBytes
```

Source policy may be owned by compiler configuration derived from the VM/project profile.

Operations must preflight when a safe upper bound is available:

- left shift;
- exact nonnegative power;
- text-to-Int conversion;
- oversized literal materialization;
- multiplication/addition where result-size bounds trigger configured policy.

`#numericLimit` is not an OOM recovery mechanism. Host allocation failure follows the runtime's general fatal/resource-exhaustion policy unless a separate allocator contract exists.

Defaults and configuration API remain open under OD-NUM-005.

## 19. Error construction

All runtime numeric failures construct language Error values and return through one Raise mechanism.

Do not expose parallel host-only variants for divide by zero, invalid shift, or conversion. Internal helper enums may exist, but they must be translated before crossing the user-observable boundary.

Error builders should accept structured fields and source-span context, avoiding string parsing in tests and tooling.

## 20. Guarded fast paths

Numeric selectors remain dynamically dispatched.

A fast path must guard:

- concrete receiver and argument representation;
- expected class identity;
- selector/method version or pristine epoch;
- integer overflow or exceptional conditions;
- resource-policy preconditions.

On failure, it falls back to ordinary selector dispatch.

A generic method-version invalidation system is preferred. If unavailable, independent Int and Float epochs are required. The mechanism remains open under OD-NUM-007.

No arithmetic opcode may be permanently specialized against the old flat Float representation.

## 21. Rendering fast paths

Int and Float `toString` overrides invalidate independently.

A Number-only pristine flag is incorrect because installing `Int#toString` or `Float#toString` does not modify Number's method row.

Prefer generic selector-version guards shared with arithmetic and other leaf types.

## 22. Dependency policy

Use a mature arbitrary-precision integer library rather than hand-rolling BigInt arithmetic.

Pin value-semantic dependencies centrally and deliberately. Take conversion-trait dependencies only when actually used.

The library is an implementation component; Phalcom's public arithmetic, floor division, remainder, limits, text, and hash semantics remain defined by these specifications, not by library defaults.

## 23. Implementation phases

Each phase must end at a compiling, testable checkpoint:

1. **Semantic kernel and constants:** exact Float decoder, normalization, heap-independent constant forms.
2. **Representation:** Int/Float runtime arms, LargeInt object, exhaustive match migration.
3. **Tower and reflection:** classes, abstract allocation, class identity, Number protocol placement.
4. **Literals and text:** token/AST split, exact constants, corrected grammar, conversion diagnostics.
5. **Arithmetic and conversions:** exact Int arithmetic, Float-domain conversion, `/`, narrowing.
6. **Division and remainder:** exact `~/`, floor `%`, all sign cases.
7. **Equality, keys, and hashing:** land atomically to avoid `==`/hash incoherence.
8. **Strict Int boundaries:** remove integral-Float indexes and update fixtures.
9. **Float protocol and power:** after OD-NUM-003 and OD-NUM-009 close.
10. **Bitwise:** after OD-NUM-006 and resource limits close.
11. **Fast paths:** only after semantic conformance is green.
12. **Docs/status/migration:** ADR/PDR status and release notes in the same landing sequence.

## 24. Documentation requirements

Every public runtime item and every semantic helper must explain:

- exact versus Float-domain behavior;
- canonicalization responsibility;
- errors and resource policy;
- why a host primitive is or is not semantically equivalent.

Comments that retain obsolete flat-Number, stable-English-message, or Float-index assumptions are correctness defects.

<!-- END implementation.md -->


---

<!-- BEGIN conformance.md -->

# Numeric Conformance Specification

> **Status:** Normative.
>
> **Decision set:** NUM-023 and every semantic decision referenced by the tests below.
>
> **Rule:** A numeric implementation is incomplete until the required properties, boundary matrices, error contracts, and dispatch behavior pass in both generic and optimized execution modes.

## 1. Reference-model architecture

The primary oracle is an implementation-independent mathematical model.

Use:

- arbitrary-precision integers for Int;
- exact dyadic decomposition for finite Float;
- exact rational comparison for mixed values;
- mathematical floor quotient and derived remainder;
- explicit binary64 rounding for Float results;
- canonical numeric-key and hash-input modeling;
- higher-precision real arithmetic for Float power accuracy.

External languages may be differential oracles only where Phalcom intentionally shares semantics:

- Python is suitable for infinite-two's-complement bitwise behavior and many floor-division cases.
- Host Rust operations are not authoritative where Rust truncates remainder, limits integer width, or delegates `pow` to platform libraries.

## 2. Required test layers

1. Lexer unit tests.
2. Parser/precedence unit tests.
3. Compiler constant tests.
4. Runtime semantic unit tests.
5. GC/rooting stress tests.
6. Public-language golden tests.
7. Negative/error golden tests.
8. Property-based tests.
9. Differential tests against selected oracles.
10. Generic-dispatch versus optimized-path equivalence tests.
11. Primitive-floor and class-reflection invariant tests.
12. Performance measurements recorded separately from semantic acceptance.

## 3. Canonical value corpus

Every relevant operation must sample:

### 3.1 Int

```text
0
±1
±2
±(2^52 - 1), ±2^52, ±(2^52 + 1)
±(2^53 - 1), ±2^53, ±(2^53 + 1)
±(2^62 - 1), ±2^62
±(2^63 - 1), ±2^63, ±(2^63 + 1)
±2^100
values immediately inside/outside the private immediate range
configured resource-limit boundaries
```

### 3.2 Float

```text
+0.0, -0.0
smallest positive/negative subnormal
largest subnormal
smallest normal
1.0 and adjacent representable values
2^52, 2^53 and adjacent representable values
largest finite positive/negative values
+Infinity, -Infinity
multiple NaN bit patterns and signs where constructible
values immediately around half-integers
values around rendering exponent thresholds -6 and 20
```

## 4. Representation invariants

### REP-1 — Class identity

```phalcom
1.class == Int
1.0.class == Float
(2 ** 200).class == Int
```

### REP-2 — Canonical demotion

A computation that crosses into LargeInt and returns into the private immediate range must produce the canonical immediate representation. This is asserted at Rust/runtime level.

### REP-3 — No surface tier leak

Equality, hash, rendering, reflection, pattern matching, method dispatch, and errors must not expose LargeInt as a separate type.

### REP-4 — Constant rooting

Load a module containing many oversized Int constants under GC stress. Every constant remains exact and live after repeated collections and module/function lifetime transitions.

## 5. Literal and text tests

### LIT-1 — Valid forms

Cover every grammar production, prefix case, separator placement, exponent sign, leading-zero form, and radix.

### LIT-2 — Candidate boundaries

Pin:

```text
5.toString
5.e2
5..2
5.
1abc
1e
1e+
1efoo
0xFFfoo
```

Verify token stream, diagnostic code, full primary span, and offending offset.

### LIT-3 — Exactness

Oversized decimal, binary, octal, and hexadecimal literals round-trip exactly and do not pass through Float.

### LIT-4 — Float parsing

Test correctly rounded ties, subnormal production, signed underflow zero, largest finite values, and overflow to infinity.

### TXT-1 — Constructor grammar

Test no trimming, decimal-only Int text, finite Float integer text, special-value case sensitivity, and rejection of signed NaN/positive Infinity spelling.

### TXT-2 — Byte offsets

Pin every offset example in `text-and-errors.md`, including multibyte UTF-8 failures.

## 6. Arithmetic tests

### ARI-1 — Small exact arithmetic

```phalcom
1 + 2 == 3
1 - 2 == -1
3 * 4 == 12
```

Results have class Int.

### ARI-2 — Promotion and demotion

Cover every checked-overflow seam and a result that later demotes.

### ARI-3 — Mixed Float domain

Verify result class Float, ties-to-even Int conversion, and `#numericOverflow` when a finite Int cannot convert to finite binary64.

### ARI-4 — Negation boundary

Negating the minimum immediate Int promotes without trap or wrap.

## 7. Division and remainder matrices

### DIV-1 — `/`

```phalcom
7 / 2 == 3.5
(6 / 2).class == Float
1 / 0 == Infinity
0 / 0 is NaN
```

Also test oversized Int conversion failure before division.

### DIV-2 — Int `~/` and `%`

For representative magnitudes, test all four sign pairs:

| `a` | `b` | `a ~/ b` | `a % b` |
|---:|---:|---:|---:|
| `7` | `2` | `3` | `1` |
| `-7` | `2` | `-4` | `1` |
| `7` | `-2` | `-4` | `-1` |
| `-7` | `-2` | `3` | `-1` |

### DIV-3 — Exact mixed `~/`

Use values where converting the Int to Float would alter the quotient, including values beyond `2^53` and near integer quotient boundaries.

### DIV-4 — Float remainder

Test floor-sign behavior, signed zero, exact model before final rounding, and every ratified non-finite table row after OD-NUM-003 closes.

### DIV-5 — Laws

For nonzero Int `b`:

```text
a == (a ~/ b) * b + (a % b)
abs(a % b) < abs(b)
```

Generate arbitrary signs and LargeInt magnitudes.

## 8. Comparison tests

### CMP-1 — Symmetry

For every mixed pair, test both operand orders.

### CMP-2 — Precision boundaries

Pin comparisons around every Float integer precision boundary, especially:

```phalcom
9007199254740993 > 9007199254740992.0
9007199254740992 == 9007199254740992.0
```

### CMP-3 — NaN

Every ordinary ordered predicate with NaN is false. `NaN == NaN` is false and `NaN != NaN` is true.

### CMP-4 — Infinity and signed zero

Pin infinity order and zero equality.

### CMP-5 — Total order

After OD-NUM-001/002 close, exhaustively test the full sequence, antisymmetry, transitivity, and deterministic behavior across equivalent Int/Float representations.

## 9. Hash and collection tests

### HASH-1 — Equality implication

Property:

```text
numericKeyEqual(a, b) => hash(a) == hash(b)
```

### HASH-2 — Large integral Float

Test equal Int/Float values beyond `2^53`, including large powers of two.

### HASH-3 — Arbitrary user hash Int

A user-defined `hash` returning a LargeInt is accepted and reduced using all significant bits.

### HASH-4 — Invalid return

Every non-Int return, including integral Float, raises `#invalidHash` with correct key span.

### KEY-1 — Representative preservation

Insert equivalent keys in both orders and inspect:

- size;
- retrieval through both representations;
- deletion through both representations;
- iterated key class and rendering;
- insertion position.

Cover Int/Float, signed zero, and multiple NaNs.

## 10. Float protocol tests

### FLT-1 — Classification

Cover every binary64 category.

### FLT-2 — Narrowing

Test values inside and beyond `i64`, negative values, subnormals, zeroes, and non-finite errors.

### FLT-3 — Ties to even

Test positive and negative half-integers around even and odd neighbors, including large exactly representable halves.

### FLT-4 — Exact-integral conversion

After selector naming closes, test finite integral success and fractional/non-finite failure.

### FLT-5 — Rendering

Require parse-render-parse bit identity for every finite Float and signed zero. NaN renders canonically.

### FLT-6 — Power

After OD-NUM-003 closes:

- exhaust the special-case table;
- test one-ULP accuracy against a high-precision oracle;
- test huge exact Int exponents without approximate exponent conversion;
- test resource-policy behavior.

## 11. Bitwise tests

### BIT-1 — Differential oracle

Compare shared operations against Python for a corpus containing all signs and representation seams.

### BIT-2 — Algebraic laws

Generate arbitrary Ints and counts under resource policy and assert every law in `bitwise.md`.

### BIT-3 — Huge count behavior

Use positive counts larger than `usize::MAX` represented as Int:

- right shift returns sign fill;
- `bitAt` returns sign extension;
- left shift raises `#numericLimit` under a low configured policy, not `#invalidShift`.

### BIT-4 — Syntax and dispatch

Test precedence, selector symbols, operator method definitions, overrides, and super-sends.

### BIT-5 — Query semantics

Test magnitude-based `bitCount`/`bitLength`, sign-independent `trailingZeros`, and the zero error.

## 12. Error tests

For every numeric error kind:

- assert `kind` and structured fields;
- assert primary and secondary spans;
- assert ordinary Raise traceback path;
- assert behavior when source/span is absent;
- do not assert complete English prose except in dedicated rendering smoke tests.

## 13. Resource-policy tests

Configure deliberately low limits and test:

- source literals;
- String constructors;
- left shift;
- exact power;
- large multiplication;
- constant materialization.

Verify:

- deterministic preflight where possible;
- `numeric.limit` at compile time;
- `#numericLimit` at runtime;
- limit and requested-size metadata;
- no classification of positive huge counts as invalid type/count;
- trusted and sandbox profiles once OD-NUM-005 closes.

## 14. Dispatch and optimization equivalence

For every optimized numeric selector:

1. execute with fast paths enabled;
2. execute through forced generic dispatch;
3. compare value bits/Int exactness, class, errors, spans, and side effects;
4. install an override and prove deoptimization/invalidation;
5. exercise overflow and resource-limit deopt edges.

Rendering overrides on Int and Float must invalidate independently.

## 15. Primitive-floor invariants

Tests must assert:

- Int and Float are present in core-class census;
- Number carries the intended zero representation-sensitive VM primitives;
- each concrete selector binding is enumerated;
- removals and additions are explicit;
- bitwise floor count matches the OD-NUM-006 result;
- the executable census, not prose, determines the total.

## 16. Golden-change discipline

Any changed golden must be listed in the implementation return report with the semantic reason.

Expected breaking categories include:

- Float `%` sign behavior;
- ties-to-even `rounded`;
- rejected integral-Float indices;
- Int-to-Float overflow errors;
- `5.e2` tokenization;
- structured error-message text changes.

Silently regenerating the full golden corpus is forbidden.

## 17. Ship gates

The numeric update cannot ship until:

1. OD-NUM-001 through OD-NUM-006 and OD-NUM-010 are closed.
2. All normative tests above pass.
3. Generic and optimized paths are equivalent.
4. Constant-pool GC stress is green.
5. Equality, numeric-key equality, and hashing land atomically.
6. Primitive-floor records and executable census agree.
7. Migration documentation reflects the chosen release policy.
8. Performance changes are measured and recorded, even though performance regressions alone do not redefine semantics.

<!-- END conformance.md -->


---

<!-- BEGIN migration.md -->

# Numeric Update Migration and Compatibility

> **Status:** Normative inventory of semantic changes. The release mechanism remains open under **OD-NUM-010**.

## 1. Migration principle

The numeric update is a language-semantic correction, not a compatibility shim over the old flat binary64 Number model.

The implementation must not preserve obsolete behavior indefinitely through hidden coercions, representation aliases, or operator-specific exceptions. Where a transition period is chosen, warnings and compatibility behavior must have a named removal release.

## 2. Breaking changes

### 2.1 Class identity

Before the update, whole and fractional numeric literals may have shared one flat numeric representation. After the update:

```phalcom
1.class       // Int
1.0.class     // Float
```

Code performing exact class checks must be audited.

### 2.2 Exact integer arithmetic

Int arithmetic no longer rounds through binary64. Programs accidentally relying on large-integer precision loss will change result.

### 2.3 Division

`/` always returns Float, including divisible Int operands:

```phalcom
6 / 2        // 3.0
```

Use `~/` when an Int quotient is required.

### 2.4 Remainder

`%` follows floor-division semantics for both Int and Float. Existing Float code relying on truncating `fmod` sign behavior changes:

```phalcom
-7.0 % 2.0   // 1.0 after this update
```

A future named `fmod` operation is not part of this release.

### 2.5 Rounding

`rounded` uses ties to even rather than ties away from zero:

```phalcom
2.5.rounded       // 2
(-2.5).rounded    // -2
```

### 2.6 Int-to-Float overflow

Converting a finite exact Int outside the finite binary64 range raises `#numericOverflow` rather than producing infinity.

Float source/text overflow may still produce infinity because it begins in the Float domain.

### 2.7 Float-to-Int construction

`Int.new(Float)` always raises. Replace it with the explicitly selected narrowing operation:

```text
floor / ceil / truncated / rounded / exact-integral conversion
```

### 2.8 Integer-only boundaries

Integral Float no longer qualifies as an index, arity, count, shift count, or similar integer-only quantity.

```phalcom
list.at(2.0)     // type error
```

Migrate by preserving Int throughout the calculation or explicitly narrowing before the boundary.

### 2.9 Numeric keys

Map and Set merge equal numeric representations, signed zeroes, and NaNs. The first inserted representative is preserved during replacement.

Programs iterating keys may observe the first representative rather than the most recent equivalent key spelling.

### 2.10 Error contracts

Tooling must inspect structured error kinds and fields rather than exact English messages.

### 2.11 Numeric tokenization

`5.e2` is an ordinary send on Int `5`, while `5e2` is a Float literal.

Identifier-like suffixes directly adjacent to numeric candidates are rejected as one malformed candidate.

### 2.12 Number allocation

Every attempt to allocate `Number` raises `#abstractClass`, including reflective and inherited allocation paths.

## 3. Source-audit checklist

Search for:

```text
/ used where an Int result is required
% with negative Float operands
rounded at half-integer boundaries
Int.new receiving Float
Float.new receiving very large Int
collection indexes or sizes represented as Float
class checks against Number
Map/Set keys mixing Int, Float, NaN, or signed zero
string comparisons against exact error messages
5.e... source forms
user-defined hash returning Float or assuming i64 width
```

## 4. Library and native-extension audit

Native APIs must update:

- Value pattern matches for separate Int and Float arms;
- arbitrary-precision Int handling;
- canonical normalization;
- numeric equality and hash callbacks;
- Int-only argument extraction;
- structured numeric Error construction;
- constant serialization;
- method invalidation/version guards.

A wildcard match that silently routes Int or Float to “other” is not an acceptable migration.

## 5. Data and serialization

`toString` is deterministic but not a bit-preserving NaN payload serialization format.

Persistent formats must specify:

- Int arbitrary precision;
- Float exact bits or a canonical decimal contract;
- signed zero policy;
- NaN payload policy;
- versioning independent of VM hash values.

Hash results must never be stored as persistent identities.

## 6. Release strategies under consideration

The semantic inventory is fixed; the deployment strategy is open.

### Option A — Immediate pre-1.0 break

Ship all changes in one language release with migration notes and no compatibility mode.

Advantages:

- no dual semantics;
- smallest implementation surface;
- avoids users depending on transitional behavior.

Risks:

- abrupt source breakage;
- harder ecosystem migration if adoption is already broad.

### Option B — One-release warnings

Recognize selected old forms temporarily, emit warnings, and remove them in the next release.

Potential candidates:

- integral Float indexes;
- `Int.new(integralFloat)`;
- old half-away rounding requests if statically recognizable.

Float `%` cannot be safely dual-interpreted without explicit mode or operator spelling.

### Option C — Language edition

Bind source modules to a language edition, preserving old semantics for old-edition code.

Advantages:

- controlled migration;
- reproducible old code.

Risks:

- numeric values crossing edition boundaries become extremely complex;
- Map/Set equality and hashing cannot safely vary per module;
- VM and standard library must support two semantic universes.

### Option D — Runtime compatibility mode

A VM-wide flag selects old or new numeric behavior.

This is strongly disfavored because libraries cannot know which semantics their callers use, and cached artifacts become mode-sensitive.

## 7. Constraints on the release decision

Whatever OD-NUM-010 chooses:

1. equality and hashing may not be split across compatibility modes inside one VM;
2. Map/Set key semantics must be VM-global and coherent;
3. serialized bytecode must record any source-edition dependency;
4. warnings must have a removal target;
5. documentation and examples must default to the new semantics;
6. no compatibility behavior may leak LargeInt as a public class;
7. old exact-English error matching receives no compatibility guarantee.

## 8. Recommended migration order for users

1. Replace Float indexes/counts with Int-producing calculations.
2. Replace implicit Float-to-Int construction with explicit narrowing.
3. Audit `/` result assumptions.
4. Audit negative `%`, especially Float operands.
5. Audit half-integer rounding.
6. Audit large Int to Float conversion.
7. Audit numeric Map/Set keys and key iteration.
8. Switch error handling to structured kinds/fields.
9. Re-run parser diagnostics for numeric-adjacent identifiers.
10. Rebaseline only the goldens whose semantic reason is understood.

## 9. Documentation landing requirements

The release change must update in one coherent series:

- numeric ADR/PDR records and status;
- language specification index;
- primitive-floor amendment and census;
- standard library API docs;
- migration guide and release notes;
- compiler diagnostic catalogue;
- runtime Error catalogue;
- examples and tutorials that use `/`, `%`, rounding, indexes, or constructors.

<!-- END migration.md -->


---

<!-- BEGIN open-decisions.md -->

# Numeric Open-Decision Register

> **Status:** Open items remaining after the 2026-07-29 numeric architecture ratification.
>
> **Rule:** These items do not reopen the public tower or the ratified semantic architecture. Each decision must be resolved explicitly; host behavior, provisional implementation, and examples are not implicit ratification.

## 1. Summary

| ID | Decision | Class | Blocks |
|---|---|---|---|
| OD-NUM-001 | Public names for exact Float-to-Int conversion and total comparison | Surface naming | Float protocol publication |
| OD-NUM-002 | Exact total-order sequence | Public semantics | Total-order implementation |
| OD-NUM-003 | Complete Float `%` and `**` special-case tables | Public semantics | Float protocol and power shipping |
| OD-NUM-004 | Numeric hash reduction, constants, width, and seed policy | Runtime semantics/implementation | Map/Set conformance |
| OD-NUM-005 | Resource-limit defaults and configuration API | Runtime/compiler policy | Numeric limits shipping |
| OD-NUM-006 | Native-versus-derived bitwise selector composition | VM architecture/floor | U-BITWISE |
| OD-NUM-007 | Fast-path invalidation mechanism | VM architecture | Numeric optimization |
| OD-NUM-008 | Large-constant binary encoding and sharing | Compiler/runtime architecture | Serialized constants |
| OD-NUM-009 | Decimal parser and shortest-render algorithm | Implementation selection | Canonical Float text fixtures |
| OD-NUM-010 | Compatibility and release strategy | Release policy | Numeric update release |
| OD-NUM-011 | Future public numeric-extension protocol | Deferred language architecture | Future numeric types only |

Blocking for the first complete numeric release: **OD-NUM-001 through OD-NUM-006, OD-NUM-008 through OD-NUM-010**. OD-NUM-007 blocks optimized paths but not a correct generic implementation. OD-NUM-011 is deliberately deferred.

---

## OD-NUM-001 — Public selector names

### Question

Choose source spellings for:

1. exact integral Float-to-Int conversion;
2. total numeric comparison.

### Exact-conversion options

| Option | Example | Strength | Cost |
|---|---|---|---|
| A | `x.toIntExact` | Explicit target and failure condition; familiar | `to` may imply conversion rather than query |
| B | `x.integerExact` | Compact, noun-like | Less conventional English |
| C | `x.asIntExact` | Signals checked view/conversion | `as` often suggests non-allocating reinterpretation |
| D | `x.exactInt` | Very compact | Reads like a property rather than failing operation |

### Total-comparison options

| Option | Example | Strength | Cost |
|---|---|---|---|
| A | `a.totalCompare(b)` | Clear and direct | Verb order differs from IEEE naming |
| B | `a.compareTotal(b)` | Close to IEEE `totalOrder` vocabulary | Slightly less natural at call site |
| C | `a.compare(b, total: true)` | Avoids selector growth | Mode flag weakens semantic clarity and dispatch identity |
| D | `a.totalOrder(b)` | Closest to IEEE terminology | Sounds Boolean rather than three-way comparison |

### Constraints

- Exact conversion must not be confused with `truncated` or value-dependent construction.
- Total comparison returns an ordering value or conventional negative/zero/positive Int; that result type must be specified with the name.
- Names must compose with selector literals, reflection, override, and super-send syntax.

### Non-ratified editorial default

`toIntExact` and `totalCompare(_)` are the clearest pair.

### Closure artifact

A PDR specifying exact selector signatures, return type, errors, reflection, and examples.

---

## OD-NUM-002 — Exact total-order sequence

### Question

Define a deterministic total order over Int and Float, including NaN, signed zero, infinities, and representationally distinct but numerically equal values.

### Options

#### A. IEEE-754 `totalOrder` adapted to Int

- Order Float values by IEEE totalOrder.
- Insert each Int at its mathematical position.
- Define a tie-break between equal Int and Float representations.

Strength: established treatment of NaN signs/payloads and signed zero.

Risk: exposes NaN payload ordering that ordinary Phalcom protocol otherwise hides.

#### B. Numeric-first canonical total order

Suggested coarse sequence:

```text
-Infinity
finite numeric values by exact mathematical value
+Infinity
NaN
```

Tie-break equal numeric values by canonical type/representation, and canonicalize all NaNs together.

Strength: simple, stable, aligned with public numeric concepts.

Risk: discards payload distinctions; must decide `-0.0`/`+0.0` and Int/Float ties.

#### C. Key-equivalence total preorder plus representative tie-break

First compare numeric-key equivalence classes, then compare representation only for deterministic sorting.

Strength: aligns sorted collections with Map/Set equivalence.

Risk: a “compare == 0” result may not mean public `==`, because NaNs are one key class.

### Required subdecisions

- Does total comparison return zero for two NaNs?
- Are NaN sign and payload observable in ordering?
- Is `-0.0` ordered before `+0.0` despite public equality?
- Does `1` compare-total equal `1.0`, or is one ordered first?
- What ordering value type is returned?

### Acceptance criteria

- total, antisymmetric, and transitive;
- deterministic across supported platforms;
- explicitly documented relation to `==` and numeric-key equality;
- suitable for sort and persistent index ordering;
- property-tested over raw Float bit patterns.

---

## OD-NUM-003 — Float `%` and `**` special-case tables

### Question

Complete the normative result/error matrix for non-finite and signed-zero cases.

### Float `%` cases requiring decisions

| Dividend | Divisor | Candidate choices |
|---|---|---|
| finite nonzero | `±0.0` | NaN (ratified direction) |
| `±0.0` | finite nonzero | signed zero matching divisor (ratified finite rule) |
| finite | `±Infinity` | finite dividend, signed zero, or NaN |
| `±Infinity` | finite | NaN or host-compatible rule |
| `±Infinity` | `±Infinity` | NaN |
| NaN | any | NaN |
| any | NaN | NaN |

The table must preserve the floor-remainder model where mathematically meaningful and avoid inheriting accidental host `fmod` behavior.

### Float `**` cases requiring decisions

At minimum:

```text
NaN ** 0
1 ** NaN
(-1) ** ±Infinity
±0.0 ** positive/negative exponent
negative zero with odd/even integral exponent
±Infinity ** finite exponent
finite base ** ±Infinity
negative finite base ** nonintegral Float
NaN propagation exceptions
```

### Options

A. Adopt a named platform-independent table closely matching a major established `pow` contract.

B. Adopt IEEE-754 recommended `pow` behavior where specified and define the remaining language cases.

C. Define a smaller Phalcom table optimized for conceptual regularity, even where it differs from common libraries.

### Constraints

- numeric zero to negative numeric exponent raises `#divideByZero` before table dispatch;
- ordinary finite real results remain within one ULP;
- exact Int exponent classification must not be lost through Float conversion;
- table results must pin signed zero and infinity signs;
- behavior must be platform-independent even if the underlying approximation differs within one ULP.

### Closure artifact

Two exhaustive tables plus bit-pattern conformance fixtures.

---

## OD-NUM-004 — Numeric hash algorithm

### Question

Choose the concrete canonical reduction and VM mixing model while preserving cross-type equality.

### Options

#### A. Modular rational hash

Represent every finite numeric value as exact rational `m/n`, reduce modulo a fixed prime, then apply sign and special tokens.

Strengths:

- natural extension to Rational and Decimal;
- mathematically coherent across Int/Float;
- arbitrary Int user hashes can use the same modular reduction.

Costs:

- modular inverse logic;
- careful denominator-divisible-by-modulus handling;
- exact constants become part of implementation compatibility policy.

#### B. Canonical numeric byte encoding plus keyed hash

Encode exact canonical numeric value, then feed bytes to the VM's keyed hash.

Strengths:

- straightforward per-run randomization;
- flexible internal width;
- one general mixing backend.

Costs:

- canonical rational byte encoding must make equal Int/integral Float byte-identical;
- potentially allocates or streams large values;
- future Decimal/Rational canonicalization still required.

#### C. Hybrid

Use mathematical reduction to a fixed-width canonical integer, then keyed-mix that integer for hash-table placement.

Strengths: separates equality coherence from collision hardening.

Costs: two stages and more constants.

### Required subdecisions

- internal width (`u64`, `usize`, or another fixed contract);
- per-run seed and mixing function;
- special tokens for NaN and infinities;
- arbitrary user-returned Int reduction;
- whether public `hash` exposes canonical pre-mix or VM-mixed result;
- collision behavior is ordinary and not confused with equality.

### Constraints

- all significant bits of arbitrary Int hash results matter;
- `numericKeyEqual` implies equal hash;
- hash is not persistent across VM runs unless separately promised;
- algorithm must be denial-of-service resistant for untrusted keys.

---

## OD-NUM-005 — Resource-limit defaults and configuration

### Question

Choose defaults, profiles, and configuration lifetime for numeric policy.

### Required controls

```text
maxSourceNumericDigits
maxTextConversionDigits
maxIntegerBits
maxNumericAllocationBytes
```

### Options

#### A. One universal default profile

Simple but cannot serve both trusted scientific workloads and untrusted embedding safely.

#### B. Trusted and sandbox profiles

Trusted profile uses generous/disabled arithmetic limits; sandbox uses finite deterministic limits.

This is the leading architecture.

#### C. No defaults; embedder must configure

Explicit but unsafe for command-line and novice use.

### Required subdecisions

- concrete defaults;
- whether source digit limit depends on radix;
- whether power-of-two radices receive higher/exempt linear-conversion limits;
- configuration at VM creation versus mutable at runtime;
- module cache key interaction;
- whether loaded bytecode is revalidated under a stricter VM policy;
- behavior for results whose exact size is expensive to predict;
- general execution/allocation budget integration.

### Constraints

- policy failure is deterministic;
- source uses `numeric.limit`, runtime uses `#numericLimit`;
- OOM is not misreported as policy failure;
- huge right shift/bitAt can short-circuit without allocation;
- tests can install very low limits cheaply.

---

## OD-NUM-006 — Bitwise primitive composition

### Question

Which ratified bitwise selectors are VM-blessed primitives and which are derived Phalcom methods?

### Options

#### A. All ten native

```text
& | ^ ~ << >> bitAt bitCount bitLength trailingZeros
```

Strength: best predictable performance.

Cost: largest frozen-floor increase and duplicated public bindings.

#### B. Operators native, queries derived

Native:

```text
& | ^ ~ << >>
```

Derived:

```text
bitAt bitCount bitLength trailingZeros
```

Strength: smaller floor.

Risk: poor asymptotics for large Int if derivation loops through bits.

#### C. Operators plus one introspection primitive

Native operators plus `bitLength`; derive remaining queries using internal arithmetic or private helpers.

Strength: potential balance.

Risk: `bitCount` and `trailingZeros` may still need efficient limb access unavailable to Phalcom code.

#### D. Private low-level limb/query primitive

Expose a smaller private VM hook and implement public selectors in core Phalcom code.

Strength: public floor may remain conceptually derived.

Risk: private primitive is still VM-blessed capability and must be audited; reflection/override behavior becomes more complex.

### Required evidence

- asymptotic analysis for LargeInt;
- benchmark on sparse/dense large values;
- primitive-floor derivability review;
- exact census amendment;
- no accidental public private-tier leakage.

---

## OD-NUM-007 — Fast-path invalidation

### Question

Choose the guard/invalidation mechanism for optimized numeric selectors and rendering.

### Options

A. Generic per-class/per-selector method version epochs.

B. Independent Int and Float pristine flags for each optimized selector family.

C. Global method-table generation.

D. No numeric fast paths initially.

### Constraints

- installing or replacing an Int method invalidates Int paths without depending on Number's row;
- Float overrides invalidate independently;
- subclass/closed-class rules are respected;
- generic and optimized errors/spans are identical;
- mechanism should generalize beyond numbers if practical.

### Non-ratified default

Use generic per-selector versions if already available or inexpensive; otherwise ship correct generic dispatch and defer fast paths rather than introduce a numeric-only invalidation architecture prematurely.

---

## OD-NUM-008 — Large constant encoding

### Question

Choose the serialized, heap-independent encoding of LargeInt constants.

### Options

A. Sign plus big-endian magnitude bytes.

B. Sign plus little-endian magnitude limbs.

C. Minimal two's-complement bytes.

D. Normalized source digits plus radix.

### Evaluation

- A is canonical, compact, language-neutral, and independent of host limb size.
- B is fast for one BigInt library but couples artifacts to library/host details unless normalized.
- C aligns with bitwise intuition but requires canonical sign-extension rules.
- D is simple for compiler handoff but inefficient for repeated loading and leaves parse cost in runtime.

### Required subdecisions

- endianness;
- zero representation;
- sign canonicalization;
- versioning;
- deduplication;
- per-module versus per-VM sharing;
- maximum encoded length under policy;
- hash/checksum for corrupted bytecode.

### Non-ratified default

Sign plus minimal big-endian magnitude bytes, with zero represented only by immediate `Int(0)`.

---

## OD-NUM-009 — Float parser and renderer implementation

### Question

Choose implementations that satisfy correctly rounded parsing and deterministic shortest rendering.

### Parser options

A. Standard-library parser after proving correct rounding and cross-platform consistency.

B. Dedicated correctly rounded decimal-to-binary64 algorithm/library.

C. Custom implementation.

### Renderer options

A. Ryu-family shortest formatter.

B. Schubfach/Dragonbox-family formatter.

C. Standard-library shortest formatting with a normative post-processing layer.

### Constraints

- parser rounds ties to even;
- renderer round-trips through the selected parser;
- exact notation thresholds and exponent spelling are enforced;
- signed zero and special spellings are canonical;
- output is identical across supported platforms;
- dependency license and maintenance are acceptable;
- boundary fixtures are generated from the normative algorithm, not host output.

### Closure artifact

Dependency/algorithm decision plus a generated corpus pinning every formatting boundary class.

---

## OD-NUM-010 — Release and compatibility strategy

### Question

How does the project deploy the breaking numeric semantics?

### Options

A. Immediate pre-1.0 break with migration notes.

B. One-release warning period for mechanically detectable old forms.

C. Language edition recorded per source module.

D. VM-wide compatibility mode.

### Constraints

- equality/hash/key semantics cannot vary inside one VM;
- Float `%` cannot safely have ambiguous semantics without explicit source mode;
- cached bytecode must record edition dependencies if editions are chosen;
- warnings require a fixed removal release;
- new documentation defaults to new semantics;
- runtime mode flags are strongly disfavored because library behavior becomes caller-dependent.

### Non-ratified default

If Phalcom remains pre-1.0 with a small ecosystem, choose immediate break. Otherwise use a narrowly scoped source edition, but keep key equality, hashing, and runtime representation VM-global.

---

## OD-NUM-011 — Future public numeric-extension protocol

### Question

May user-defined numeric classes participate in built-in mixed arithmetic, exact comparison, hashing, and numeric-key equality?

### Options

A. Kernel tower remains closed; user classes use ordinary methods without built-in cross-type integration.

B. Public conversion/coercion hooks.

C. Public canonical exact-value protocol.

D. Attribute/multimethod registration into the numeric semantic kernel.

### Risks

- asymmetric comparison and coercion;
- hash/equality disagreement;
- callback reentrancy during Map probing;
- performance and invalidation complexity;
- ambiguous result-type selection;
- security/resource behavior from user callbacks.

### Current disposition

Deferred. The internal canonical numeric-value layer may be designed for future extension, but no unrestricted user protocol ships with the Int/Float update.

---

## 2. Decision closure checklist

Every closed OD must update:

1. the affected normative document;
2. conformance tests;
3. implementation architecture where relevant;
4. migration notes if user-visible;
5. ADR/PDR and status records;
6. primitive census if VM bindings change;
7. this register, replacing **Open** with the final decision and date.

<!-- END open-decisions.md -->


---

<!-- BEGIN amendment-map.md -->

# Amendment Map from the Original Numeric Specifications

> **Status:** Informative.
>
> This document records how the revised set changes, corrects, or redistributes the original uploaded files. It is not a substitute for the normative documents.

## 1. Original `README.md`

### Problems corrected

- The original index omitted the normative numeric-literal and bitwise documents.
- It listed ratifying records without giving readers a complete specification map.

### Revision

The new README indexes semantics, implementation, conformance, migration, open decisions, and the combined edition. It distinguishes authority and open items.

## 2. Original `numeric-tower.md`

### Problems corrected

- Mixed permanent semantics with source paths, stale commit baseline, concurrency hazards, dependency details, and implementation phases.
- Called itself implementation-ready while leaving constant-pool rooting as a ship gate.
- Used one general promotion shape that could not serve exact comparison or exact mixed `~/`.
- Suggested `rem_euclid`-style behavior despite negative-divisor floor remainder.
- Retained pending-ratification wording after ratification.
- Deferred strict Int-only index boundaries despite the architecture now ratifying immediate tightening.
- Described Number as both protocol home and “empty class.”
- Carried obsolete decimal-only/`Token::BigInt(String)` pseudocode after radix literals were ratified.
- Suggested a host/runtime zero-division variant inconsistent with structured language Errors.

### Revision

Permanent semantics now live in `numeric-tower.md`; runtime/compiler details live in `implementation.md`; tests live in `conformance.md`.

The revised semantics:

- separate lossy Float arithmetic from exact comparison, hashing, keys, and `~/`;
- define exact dyadic floor division;
- define floor remainder for Int and Float;
- require Int-to-Float overflow errors;
- require strict Int-only boundaries;
- clarify allocator-abstract versus method-empty;
- land equality, key relation, and hashing as one coherent subsystem.

## 3. Original `float-protocol.md`

### Problems corrected

- Delegated Float `%` to host `fmod`, creating tower-wide remainder inconsistency.
- Used ties-away rounding.
- Claimed host Float power could vary only in NaN payload bits.
- Had no public total-order operation.
- Left zero-negative-power behavior ambiguous for Float zero/mixed operands.
- Defined user hash return as Int but did not account for heap-backed large Int consumption.

### Revision

- Float `%` is floor remainder.
- `rounded` uses ties to even.
- Float power has explicit special cases plus one-ULP ordinary finite accuracy.
- every numeric zero to negative power raises;
- a total-order operation is required, with name/order still open;
- exact Float decoding is shared across comparison, narrowing, division, and hashing.

## 4. Original `numeric-literals.md`

### Problems corrected

- Used `DIGIT` without defining it.
- Did not fully specify malformed-candidate boundaries.
- Did not settle `5.e2` after the architectural review.
- Required compiler-minted LargeInt objects in a constant pool, leaving heap/GC coupling.
- Used one diagnostic code for syntax without distinguishing policy excess.

### Revision

- Grammar is self-contained.
- Adjacent identifier, radix, exponent, dot-send, and range boundaries are explicit.
- `5.e2` is an ordinary send.
- large constants use heap-independent descriptors.
- `numeric.literal` and `numeric.limit` are distinct.

## 5. Original `text-and-errors.md`

### Problems corrected

- Referenced undefined/mismatched grammar nonterminals.
- Called Float conversion from arbitrary Int a “widening.”
- Did not specify decimal conversion rounding or underflow.
- Made exact English message templates stable.
- Did not define exact malformed-text byte-offset selection.
- Introduced `#numericLimit` without a configured policy model.
- Omitted `trailingZeros(0)` error kind.

### Revision

- Constructor grammar is complete and shares named productions.
- Int-to-Float conversion rounds ties-to-even and raises on finite-range overflow.
- text parsing underflow/overflow behavior is explicit.
- error kinds/fields/spans are stable; prose is not.
- byte offsets use first offending UTF-8 byte or EOF length.
- compiler and runtime policy failures are distinct.
- `#undefinedNumericOperation` covers partial numeric queries.

## 6. Original `bitwise.md`

### Problems corrected

- Referenced undefined `2.pow(n)` instead of `2 ** n`.
- Omitted `**` and prefix `~` interaction from precedence.
- Used “magnitude-independent” for `trailingZeros` instead of sign-independent.
- Named allocation failure rather than deterministic `#numericLimit`.
- Did not define huge nonnegative counts that exceed `usize`.
- Did not account for primitive-floor growth.
- Stated unconditional algebraic laws despite resource-policy failure.

### Revision

- laws use the actual power selector;
- complete relevant precedence is stated;
- huge right shifts and bit indexes short-circuit by sign extension;
- left shift uses policy preflight;
- trailing-zero error is structured;
- laws are qualified by successful completion under policy;
- primitive composition is tracked as OD-NUM-006.

## 7. New `implementation.md`

Created to hold material that should not be permanent language semantics:

- Value/Object shapes;
- canonical normalization;
- exact Float decomposition;
- semantic-kernel boundaries;
- heap-independent constants;
- GC/rooting;
- hash architecture;
- class/primitive placement;
- resource-policy hooks;
- dispatch invalidation;
- phased landing plan.

## 8. New `conformance.md`

Created because examples embedded in design prose were insufficient to pin:

- mixed precision boundaries;
- all floor-division sign pairs;
- Float bit classes;
- Map/Set representative preservation;
- arbitrary LargeInt user hashes;
- parser candidate consumption;
- resource limits;
- generic/optimized equivalence;
- primitive-floor invariants.

## 9. New `migration.md`

Created to isolate breaking behavior and prevent compatibility choices from contaminating core semantics.

## 10. New `open-decisions.md`

Created so unresolved names, tables, constants, defaults, encodings, algorithms, and release policy cannot be mistaken for implementation freedom or silently inherited from host behavior.

<!-- END amendment-map.md -->
