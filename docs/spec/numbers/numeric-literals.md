# Numeric literals

Part of the [Phalcom Language Specification](spec/current/README.md). **Status: Normative** — [PDR-0026](../../../pdr/0026-numeric-literals.md).

## 1. Forms and values

```text
INT          := DEC-INT | BIN-INT | OCT-INT | HEX-INT
DEC-INT      := ZERO-INT | NZ-DIGIT { DEC-GROUP }
ZERO-INT     := "0" { "0" | "_" "0" }
BIN-INT      := "0" ( "b" | "B" ) [ "_" ] BIN-DIGIT { BIN-GROUP }
OCT-INT      := "0" ( "o" | "O" ) [ "_" ] OCT-DIGIT { OCT-GROUP }
HEX-INT      := "0" ( "x" | "X" ) [ "_" ] HEX-DIGIT { HEX-GROUP }
FLOAT        := DEC-DIGITS "." DEC-DIGITS [ EXPONENT ]
              | "." DEC-DIGITS [ EXPONENT ]
              | DEC-DIGITS EXPONENT
EXPONENT     := ( "e" | "E" ) [ "+" | "-" ] DEC-DIGITS
DEC-DIGITS   := DIGIT { DEC-GROUP }
DEC-GROUP    := DIGIT | "_" DIGIT
BIN-GROUP    := BIN-DIGIT | "_" BIN-DIGIT
OCT-GROUP    := OCT-DIGIT | "_" OCT-DIGIT
HEX-GROUP    := HEX-DIGIT | "_" HEX-DIGIT
NZ-DIGIT     := "1".."9"
BIN-DIGIT    := "0" | "1"
OCT-DIGIT    := "0".."7"
HEX-DIGIT    := DIGIT | "a".."f" | "A".."F"
```

`DEC-INT`, `BIN-INT`, `OCT-INT`, and `HEX-INT` create `Int`. Integer magnitude is unlimited; `LargeInt` is never visible at the language surface. A literal that does not fit `i64` is carried to the compiler as `{ digits, radix }` and built through the tower's normalization constructor.

`FLOAT` creates IEEE-754 `Float`. An exponent makes a float even when the mathematical result is whole: `2e10.class == Float`. Syntax-valid overflow yields signed infinity; it is not a lexical error.

```phalcom
1_000_000    // Int
0x_FF_A0_00  // Int
0b1101       // Int
0o755        // Int
3.1415_9265  // Float
.25          // Float
2e10         // Float
6.02e-23     // Float
```

## 2. Dot and range boundary

Fractional digits are mandatory after a decimal point. Therefore `5.` and `5.e2` are invalid; write `5.0` or `5e2`. A dot followed by an identifier remains a send and two dots remain a range:

```phalcom
5.toString   // Int receiver, ordinary dot send
5..2         // Int .. Int
.25          // Float; dot followed immediately by a digit
```

No whitespace occurs within any numeric literal. A leading `-` is unary syntax, not part of a literal: `-0xFF` parses as negation of an `Int` literal.

## 3. Separators and errors

One `_` may appear between two digits valid for the active radix. Prefix forms additionally permit one immediately after the prefix, so `0x_FF` is valid. Separators may not touch a decimal point, exponent marker/sign, literal end, or another separator.

The lexer reports a single numeric-literal error for `0x`, `0b2`, `1_`, `1__0`, `1_.0`, `1e_3`, `1e+_3`, `0123`, and every other malformed form. It must consume through the malformed numeric candidate rather than emit an initial valid literal plus trailing tokens.

`n` and `j` are not numeric suffixes. `1n`, `0xFFn`, `1.0n`, and `7j` are invalid numeric candidates; future public `BigInt` or `Complex` types require their own decision.

## 4. Implementation and verification

The lexer emits `Token::Int(i64)`, `Token::Float(f64)`, or an oversized integer payload containing normalized digits and radix. The AST preserves this trichotomy; the compiler, not `phalcom-ast`, parses oversized digits with `num-bigint` and roots the resulting `LargeInt` in the constant pool.

Required tests cover each valid form, case-insensitive prefixes, separator positions, every malformed boundary above, `5.toString`, `5..2`, exponent classification, overflow, `i64` boundary and oversized literals, exact radix round trips, source spans, and GC stress for a constant-pool `LargeInt`.
