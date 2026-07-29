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
