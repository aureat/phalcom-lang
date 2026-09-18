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
