# Numeric Conformance Specification

This document defines the independent models, fixtures, properties, differential tests, and ship gates required for the numeric implementation.

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
10. Primitive, intrinsic, and specialized-path equivalence tests.
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

### REP-5 — Closed tower

Reject attempts to subclass `Number`, `Int`, or `Float`, and reject every post-bootstrap method mutation of those classes. A user class defining numeric-looking selectors remains an ordinary `Object` descendant: custom-receiver sends work through ordinary dispatch, built-in receivers invoke no reverse/coercion hook, and the user value does not enter built-in numeric-key canonicalization.

## 5. Literal and text tests

### LIT-1 — Valid forms

Test every grammar production, radix, separator location, exponent form, range boundary, and Int/Float classification rule.

### LIT-2 — Candidate boundaries

Pin malformed adjacent identifiers, radix candidates, exponent candidates, range syntax, EOF truncation, and the first invalid UTF-8 byte offset.

### LIT-3 — Exactness

Large integer literals must equal an independent arbitrary-precision oracle and must never round through Float.

### LIT-4 — Float parsing

For each fixture, compare raw `u64` bits against an independent exact-decimal oracle. Include signed zero, every subnormal boundary, minimum normal, adjacent representable values, midpoint ties with even and odd lower candidates, maximum finite, overflow midpoint, long significands, huge exponents, and underscore-equivalent spellings.

### LIT-5 — LargeInt encoding

Reject empty magnitudes, zero sign forms, leading zero bytes, signed-64-bit values encoded as `LargeIntV1`, length mismatches, unknown versions, and policy-exceeding canonical constants. Verify module-local deduplication and one materialization per entry.

### TXT-1 — Constructor grammar

Source literals and runtime constructors share the finite conversion kernel but retain distinct entry grammars and error delivery.

### TXT-2 — Byte offsets

Test every documented malformed-text offset, including multibyte UTF-8 input and EOF failures.

### TXT-3 — Canonical rendering

For random and targeted binary64 bits, render then parse and require identical bits, except NaN payloads intentionally canonicalize. Pin every notation boundary around exponents `-7`, `-6`, `20`, and `21`, integral `.0`, exponent spelling, signed zero, infinities, and NaN.

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

Test every Int/Float pair, conversion overflow, signed zero, infinities, NaNs, and result class.

### DIV-2 — Int `~/` and `%`

For all sign pairs and large values, verify exact floor quotient, divisor-signed remainder, divide-by-zero errors, and the reconstruction laws.

### DIV-3 — Exact mixed `~/`

Compare Int/finite-Float and Float/Int results against an exact rational oracle beyond `2^53`, across subnormals, and at huge exponent gaps.

### DIV-4 — Float `%`

Test every row of the special-case table. For finite nonzero inputs, compare the exact floor remainder followed by one ties-to-even binary64 rounding. Pin divisor-signed zero, underflowed zero, and rounded results that equal divisor magnitude.

### DIV-5 — Laws

For Int operands and nonzero divisor:

```text
a == (a ~/ b) * b + (a % b)
abs(a % b) < abs(b)
```

For finite Float-domain remainder, verify the exact pre-rounding sign and bound rather than requiring a rounded Float reconstruction identity.

## 8. Comparison tests

### CMP-1 — Symmetry

Mixed equality and ordinary comparison must be invariant under operand-order reversal where the relation is symmetric.

### CMP-2 — Precision boundaries

Test exact Int/Float comparison around `2^53`, every binary64 exponent transition, maximum finite Float, and arbitrary-precision integral Float values.

### CMP-3 — NaN

Public equality is non-reflexive and ordinary order predicates are false. Numeric-key equality and `totalCompare` merge every NaN.

### CMP-4 — Infinity and signed zero

Pin ordinary and total comparison for infinities, Int zero, and both signed zeroes.

### CMP-5 — Total comparison

Property-test raw Float bit patterns and mixed Int/Float values for totality on key classes, transitivity, reversal symmetry, and the exact sequence:

```text
-Infinity < finite exact values < +Infinity < NaN
```

Equal mathematical representations, all zero forms, and all NaNs must return `Ordering.equal`.

## 9. Hash and collection tests

### HASH-1 — Key implication

For the complete corpus and randomized values:

```text
numericKeyEqual(a, b) => a.hash == b.hash
```

### HASH-2 — Canonical rational stream

Verify exact reduced rational equivalence for integral and nonintegral Float values, including magnitudes beyond `2^53`, signed zero, infinities, and diverse NaNs.

### HASH-3 — VM seed behavior

With a fixed test seed, built-in hashes are reproducible. With independent random seeds, no cross-VM stability is asserted. Built-in results remain within `0..<2**64`.

### HASH-4 — Arbitrary user hash Int

Use values differing only in high bits, sign, or significant length to prove that per-collection finalization consumes the entire arbitrary-precision Int. A non-Int result raises `#invalidHash`.

### HASH-5 — Per-collection salt

With deterministic test configuration, verify that equal public hashes may produce different bucket placement in distinct collections while lookup remains correct.

### KEY-1 — Representative preservation

Insert equivalent keys in different representations and require first representative and iteration position preservation for Map and Set. Collision tests must prove equality is checked after hash match.

## 10. Float protocol tests

### FLT-1 — Classification

Cover normal, subnormal, both zeroes, infinities, and diverse quiet/signaling NaN bit patterns where constructible.

### FLT-2 — Narrowing

Test `floor`, `ceil`, `truncated`, `rounded`, and `toIntExact` beyond `i64`, at fractional boundaries, and for every non-finite class.

### FLT-3 — Ties to even

Cover positive and negative half-integers with even and odd adjacent integers.

### FLT-4 — Exact conversion

`toIntExact` returns self for Int, maps both zeroes to Int zero, converts every finite fractionless Float exactly, and raises `#numericConversion` for fractional or non-finite Float.

### FLT-5 — Rendering

Pin Ryū candidate selection and Phalcom notation independently, then round-trip through the specified parser.

### FLT-6 — Power

Test every identity and table row, exact Int exponent parity beyond host width, negative-base domain rejection, signed zero and infinity results, and overflow/underflow sign. Ordinary finite real results are checked against a high-precision oracle with a one-ULP bound.

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

Test precedence, selector symbols, operator method definitions on user classes, explicit reflective invocation of Int methods, and rejection of attempts to modify frozen Int methods.

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

Test the named `standard` and `sandbox` profiles and custom policies with very small limits.

Required boundaries:

- exact acceptance at each maximum and rejection at maximum plus one;
- uniform digit accounting across radices for source, runtime parsing, and rendering;
- Int rendering rejection at `maxTextConversionDigits + 1`;
- exponent and leading-zero digit counting;
- disabled fields represented by `None`;
- zero as an active limit;
- exact Int bit length and logical byte charge;
- Float logical charge of eight bytes;
- short-circuiting huge right shifts and bit queries;
- no rejection from a loose upper bound alone;
- compile-time `numeric.limit` versus runtime `#numericLimit`;
- real allocator failure not relabeled as policy failure;
- cache-key policy fingerprints and bytecode revalidation under stricter policy.

## 14. Primitive and intrinsic equivalence

For every optimized numeric selector and operand tuple:

1. execute through the installed primitive method;
2. execute through the interpreter intrinsic;
3. execute through any statically specialized opcode;
4. compare raw values, result class, errors, structured fields, spans, evaluation order, signed zero, NaN class, and resource-policy effects.

Unsupported tuples must fall back to ordinary selector dispatch. Explicit `perform` and method-object invocation must continue to use the reflected method.

Kernel-freeze tests must prove that user source and reflection cannot add, replace, or remove methods on `Number`, `Int`, or `Float`. No epoch or pristine guard is required after the freeze.

## 15. Primitive-floor invariants

The census test must prove:

- every representation-sensitive numeric binding is installed on its specified class;
- all ten bitwise selectors are present individually as Int primitives;
- no public or private multiplexed raw-bitwise selector leaks into reflection;
- `Number` remains allocator-abstract;
- `Int` and `Float` remain the only concrete built-in Number descendants;
- the kernel freeze prevents post-bootstrap mutation.

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

The numeric release may ship only when:

1. every document in this specification is implemented without semantic placeholders;
2. the independent arithmetic, comparison, hash, parser, renderer, and power oracles pass;
3. every special-case table row is pinned;
4. resource profiles and bytecode validation pass at exact boundaries;
5. primitive and intrinsic paths are observationally equivalent;
6. stale bytecode is rejected cleanly;
7. the primitive census and kernel-freeze invariants pass;
8. migration documentation and diagnostics describe the atomic break;
9. all modified golden outputs carry an understood semantic reason.

