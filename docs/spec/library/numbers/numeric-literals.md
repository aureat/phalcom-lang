# Numeric Literals

This document defines numeric source candidates, tokenization, classification, source limits, constant encoding obligations, and compiler diagnostics.

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

An integer literal outside the immediate signed 64-bit range remains a legal Int literal when it satisfies the active source and value limits.

Compiler-owned bytecode stores it as a canonical `LargeIntV1` constant-pool entry containing:

```text
sign: Positive | Negative
magnitude: minimal nonempty unsigned big-endian bytes
```

The first magnitude byte is nonzero. Zero, negative zero, empty magnitudes, redundant leading zeroes, and values representable as signed 64-bit immediate Int are forbidden in `LargeIntV1`.

The compiler deduplicates equal large constants within a module after numeric canonicalization, independent of source radix or separators. Runtime materialization and validation are defined in the [numeric runtime implementation plan](../../../implementation/roadmap/numbers-runtime-implementation.md).

## 10. Overflow, underflow, and policy limits

### 10.1 Float syntax

A syntax-valid Float literal converts directly to binary64 using round-to-nearest, ties to even.

- Overflow produces signed infinity.
- Underflow produces the correctly rounded subnormal or signed zero.
- Locale has no effect.

### 10.2 Int syntax

A syntax-valid Int literal produces its exact mathematical integer subject to `maxIntegerBits` and `maxNumericAllocationBytes`. It never overflows into Float.

### 10.3 Source digit accounting

`maxSourceNumericDigits` counts every digit character in one numeric source token, uniformly across all radices. It includes leading zeroes, fractional digits, and exponent digits. It excludes sign, radix prefix, decimal point, exponent marker, and digit separators.

Built-in profiles are:

| Limit | `standard` | `sandbox` |
|---|---:|---:|
| `maxSourceNumericDigits` | `100_000` | `4_096` |
| `maxTextConversionDigits` | `100_000` | `4_096` |
| `maxIntegerBits` | `8_388_608` | `262_144` |
| `maxNumericAllocationBytes` | `2_097_152` | `65_536` |

Power-of-two radices receive no exemption or multiplier. The profile values are platform-independent.

### 10.4 Diagnostic distinction

Malformed numeric source reports `numeric.literal`. A syntax-valid candidate that exceeds source policy reports `numeric.limit`. The compiler must not report a host parser or allocator failure as either language diagnostic.

## 11. Required source diagnostics

A numeric-literal diagnostic must provide:

- stable code (`numeric.literal` or `numeric.limit`);
- primary span covering the full candidate;
- radix or literal class when relevant;
- first offending byte offset relative to the candidate when meaningful;
- `limit`, `maximum`, `observedAtLeast`, and radix where applicable for `numeric.limit`.

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
