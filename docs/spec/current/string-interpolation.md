# Phalcom String Literals and Interpolation

**Status:** Accepted by [PDR-0029](../../pdr/0029-string-literals-and-interpolation-completion.md)
**Target:** Phalcom language specification
**Date:** 2026-07-22

## 1. Scope

This document defines the lexical grammar, parsing rules, evaluation semantics, diagnostics, source ranges, and conformance requirements for double-quoted string literals and string interpolation.

It supersedes any earlier rule that:

- treats an interpolation body as raw text balanced only by counting `(` and `)`;
- lowers interpolation through `String.new(expression)`;
- preserves unknown backslash escapes literally; or
- permits more than one top-level expression inside a single interpolation.

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

## 2. Source model

Phalcom source is UTF-8.

All source ranges used by diagnostics and front-end data structures are half-open UTF-8 byte ranges:

```text
start..end
```

`start` is inclusive and `end` is exclusive. A point diagnostic is represented by a zero-width range where `start == end`.

A scanner MUST advance by complete UTF-8 scalar values when it is not consuming fixed ASCII syntax. It MUST NOT split a UTF-8 scalar or report a range ending inside one.

## 3. Lexical grammar

### 3.1 String literal

A string literal is either a single-line double-quoted string or a triple-quoted multiline text block ([PDR-0034](../../pdr/0034-multiline-string-text-blocks.md)):

```text
string-literal ::= single-line-string | multiline-text-block

single-line-string ::= `"` string-part* `"`
multiline-text-block ::= `"""` hspace* newline multiline-body margin `"""`

string-part    ::= literal-character
                 | escape-sequence
                 | interpolation

escape-sequence ::= `\"`
                  | `\\`
                  | `\n`
                  | `\t`
                  | `\r`

interpolation  ::= `\(` interpolation-expression `)`
```

`literal-character` is any Unicode scalar value other than `"` or `\`.

Physical LF and CRLF are invalid inside a single-line double-quoted string; multiline text blocks (`"""`) are used for strings spanning physical lines.

### 3.2 Escape values

The supported escapes decode as follows:

| Source | String value |
|---|---|
| `\"` | U+0022 QUOTATION MARK |
| `\\` | U+005C REVERSE SOLIDUS |
| `\n` | U+000A LINE FEED |
| `\t` | U+0009 CHARACTER TABULATION |
| `\r` | U+000D CARRIAGE RETURN |

`\(` is not a character escape. It opens an interpolation.

No other escape is valid. An unknown escape such as `\q` is a syntax error. It MUST NOT be preserved as two literal characters.

A backslash is processed left-to-right and consumes its escape partner. Therefore:

```phalcom
"\\("
```

contains the literal two-character text `\(` and does not open an interpolation.

Likewise:

```phalcom
"\\\(value)"
```

contains one literal backslash followed by one interpolation.

### 3.3 Quotes

A `"` in string-literal mode closes the current string unless it is consumed by the `\"` escape.

A quote inside an interpolation begins an ordinary nested Phalcom string literal. The nested string is lexed using this entire specification, including its own escapes and interpolations.

## 4. Interpolation boundary scanning

### 4.1 Opening delimiter

An interpolation begins when string-literal mode encounters the two-byte sequence:

```text
\(
```

The backslash and opening parenthesis are the interpolation opening delimiter. They do not contribute characters to the resulting String.

### 4.2 Closing delimiter

The interpolation closes at the `)` that balances its opening `(` while the scanner is in expression-code mode.

The scanner begins the interpolation body at parenthesis depth one.

In expression-code mode:

- `(` increases parenthesis depth by one.
- `)` decreases parenthesis depth by one.
- A `)` that decreases depth to zero closes the interpolation.
- A nested string literal is consumed as one lexical construct and does not affect the outer interpolation depth.
- A line comment is consumed as one lexical construct and parentheses inside it do not affect depth.
- A block comment is consumed as one lexical construct and parentheses inside it do not affect depth.
- Any other character is consumed according to ordinary Phalcom lexical rules.

Only parentheses affect interpolation depth. Braces and brackets do not.

### 4.3 Nested lexical modes

The interpolation scanner MUST apply the same string and comment rules used outside interpolation. It MUST NOT contain an approximate or reduced duplicate of those rules.

These examples are valid and the marked parentheses do not close or deepen the outer interpolation:

```phalcom
"\(String.new(")"))"
"\(String.new("("))"
"\(value /* ) ( */ + other)"
"\(value // )
   + other)"
"\("nested \(value)")"
```

Comment delimiters inside nested strings are ordinary string text:

```phalcom
"\("/* not a comment ) */")"
"\("// not a comment )")"
```

Quote characters inside comments do not enter string mode:

```phalcom
"\(value /* " ) ( */ + other)"
```

### 4.4 Comment rules

Line and block comments inside an interpolation use the language's normal comment rules.

A line comment begins with `//` and continues to the next physical newline or end of input. Its terminating newline, when present, remains part of the captured interpolation source and is processed by the expression lexer normally.

A block comment begins with `/*` and ends at the next `*/`. Block comments are non-nesting unless a separate Phalcom specification explicitly changes the language-wide block-comment rule.

## 5. Interpolation expression grammar

### 5.1 Exactly one expression

The text between `\(` and its matching `)` MUST parse as exactly one Phalcom expression.

Leading and trailing trivia are permitted. Trivia includes spaces, comments, and newlines accepted by the normal lexer.

The parser MUST reject:

- an empty body;
- a body containing only trivia;
- a statement declaration such as `let`, `const`, `return`, `throw`, `break`, `continue`, or `import`;
- a top-level statement separator;
- a second top-level expression; and
- any trailing token that is not trivia.

Valid:

```phalcom
"\(value)"
"\((value))"
"\(call(first, second))"
"\(
    value
)"
"\(
    /* explanation */
    call(
        first,
        second
    )
)"
"\({ first(); second() })"
```

The last example is valid because the block is one expression even though the block body contains statements.

Invalid:

```phalcom
"\()"
"\( /* comment only */ )"
"\(let value = 1)"
"\(1; 2)"
"\(1
  2)"
```

### 5.2 Newlines

Newlines inside an interpolation are handled by the ordinary Phalcom newline and continuation rules.

A newline that is legal inside an expression remains legal. A newline that terminates one expression followed by another expression causes the interpolation to be rejected as containing trailing input.

## 6. Segment model

A string containing interpolation is conceptually divided into ordered segments:

```text
Literal(text)
Expression(source)
```

Literal segments contain decoded String values. Expression segments contain the exact original source bytes between the interpolation delimiters.

Empty literal segments are omitted. Expression segments are never omitted.

Examples:

```phalcom
"a \(x) b"
```

has:

```text
Literal("a ")
Expression("x")
Literal(" b")
```

```phalcom
"\(a)\(b)"
```

has:

```text
Expression("a")
Expression("b")
```

```phalcom
"\(x)"
```

has:

```text
Expression("x")
```

The expression segment's source range covers only the body between `\(` and `)`. For `\()`, that range is zero-width.

## 7. Evaluation semantics

### 7.1 Result type

Every successfully evaluated string literal produces a `String`.

An interpolated string MUST NOT evaluate to the raw value returned by an interpolation expression or its `toString` getter.

### 7.2 Evaluation order

Segments are evaluated strictly from left to right.

For each literal segment, its decoded text is appended to the result.

For each expression segment, the implementation performs these operations in order:

1. Evaluate the expression exactly once.
2. Send the `toString` getter exactly once to the resulting value.
3. Require the getter result to be a `String`.
4. Append that String exactly once to the accumulated result.

No later segment may be evaluated before an earlier segment completes.

### 7.3 `toString` selector

String interpolation uses the getter selector:

```phalcom
value.toString
```

It does not use:

```phalcom
value.toString()
```

and does not call:

```phalcom
String.new(value)
```

A user-defined `toString` override therefore participates in interpolation through ordinary getter dispatch.

### 7.4 Invalid `toString` result

If `toString` returns a non-String value, interpolation raises the language's normal runtime type error for a String-required operation.

This requirement applies equally to:

```phalcom
"\(value)"
```

and:

```phalcom
"prefix \(value) suffix"
```

An interpolation-only string MUST NOT allow a non-String result to escape merely because no source literal segment surrounds it.

### 7.5 Exceptions and side effects

If evaluation of an interpolation expression throws, that exception propagates unchanged.

If its `toString` getter throws, that exception propagates unchanged.

After either failure:

- no later segment is evaluated;
- no earlier segment is re-evaluated; and
- the failing expression or getter is not retried.

Observable side effects therefore occur in source order and at most once per segment operation.

### 7.6 As-if optimization

An implementation MAY build the result with a buffer, builder, specialized bytecode, or another optimization instead of repeated `String#+` sends, provided all observable behavior is identical to Sections 7.1 through 7.5.

In particular, an optimization MUST preserve:

- left-to-right expression evaluation;
- exactly-once expression evaluation;
- exactly-once `toString` dispatch;
- String-result validation; and
- exception ordering.

## 8. Reference front-end lowering

This section is normative for the Phalcom reference parser and its AST conformance tests.

### 8.1 Expression segments

Every expression segment becomes exactly one getter-send node whose property is `toString`.

Conceptually:

```text
Expression(e) → GetProperty(e, "toString")
```

It MUST NOT become a zero-argument method-call node.

### 8.2 Concatenation

The parser folds segments with left-associative String concatenation.

If the first segment is an expression, the parser first introduces an empty String accumulator. This ensures an interpolation-only string is still a String-producing concatenation and validates the `toString` result.

Examples:

```phalcom
"a \(x) b \(y)"
```

lowers conceptually to:

```text
(((String("a ") + GetProperty(x, "toString"))
                  + String(" b"))
                  + GetProperty(y, "toString"))
```

```phalcom
"\(x)"
```

lowers conceptually to:

```text
String("") + GetProperty(x, "toString")
```

```phalcom
"\(a)\(b)"
```

lowers conceptually to:

```text
(String("") + GetProperty(a, "toString"))
           + GetProperty(b, "toString")
```

Every expression appears exactly once in the lowered AST.

A string without interpolation remains one ordinary String-literal AST node and is not rewritten into concatenation.

### 8.3 Synthetic ranges

The reference parser MAY assign the complete outer string-literal range to synthetic concatenation and `toString` getter nodes to preserve existing AST range conventions.

Errors produced while parsing an interpolation body MUST use the body's absolute source ranges and MUST NOT be widened to the whole outer string.

## 9. Diagnostics

The following diagnostic categories are required. The exact user-facing capitalization may follow the repository's diagnostic style, but the distinctions and ranges are normative.

### 9.1 Invalid escape

**Code:** `string.invalid_escape`
**Message:** `Invalid string escape`
**Range:** the backslash and the following escaped Unicode scalar.

Example:

```phalcom
"\q"
```

The range covers `\q`.

### 9.2 Unterminated string

**Code:** `string.unterminated`
**Message:** `Unterminated string`
**Range:** from the unmatched opening `"` to end of input.

The parser-completeness diagnostic MUST identify `"` as the expected closer.

### 9.3 Unterminated interpolation

**Code:** `string.interpolation.unterminated`
**Message:** `Unterminated string interpolation`
**Range:** from the unmatched `\(` delimiter to end of input.

The parser-completeness diagnostic MUST identify `)` as the expected closer.

### 9.4 Empty interpolation

**Code:** `string.interpolation.empty`
**Message:** `String interpolation requires an expression`
**Range:** the interpolation body range.

For `\()`, this is a zero-width point between `(` and `)`.

For `\( /* comment */ )`, it covers the trivia-only body.

### 9.5 Unterminated nested construct

If end of input occurs while a nested lexical construct is open, the innermost unclosed construct determines the primary diagnostic and expected closer.

Examples:

- an unclosed nested string reports `string.unterminated` and expects `"`;
- an unclosed block comment reports the ordinary unterminated-comment diagnostic and expects `*/`;
- a closed nested construct followed by an unclosed outer interpolation reports `string.interpolation.unterminated` and expects `)`.

### 9.6 Malformed inner expression

A malformed interpolation expression uses the ordinary parser diagnostic that would be produced for the same expression source.

Its range is translated to the absolute byte range in the containing source file.

The diagnostic MUST point into the interpolation body. It MUST NOT underline the complete outer string solely because the expression was parsed through interpolation.

### 9.7 Trailing input

A second top-level expression or statement separator after a valid expression is a syntax error at the first trailing token.

The diagnostic SHOULD state that the end of the interpolation was expected.

## 10. Unicode and range examples

Given:

```phalcom
"α🙂 \(value)"
```

the expression body's byte start is calculated after the UTF-8 bytes for `"α🙂 \(`, not by counting Unicode scalar values.

Given:

```phalcom
"α \(€)"
```

an invalid-token diagnostic for `€`, if `€` is not valid expression syntax, covers all three UTF-8 bytes of `€`.

Given a nonzero parser base offset, every interpolation-body diagnostic adds both:

1. the base offset of the parsed source unit; and
2. the body's local byte offset inside that source unit.

## 11. Conformance requirements

A conforming implementation MUST include automated tests for all of the following categories:

1. Plain strings and every supported escape.
2. Unknown escapes.
3. Interpolation-only, prefix-only, suffix-only, and surrounded interpolation.
4. Adjacent interpolations.
5. Nested calls and parenthesized expressions.
6. Parentheses inside nested strings.
7. Parentheses inside line and block comments.
8. Comment delimiters inside nested strings.
9. Nested interpolation inside a nested string.
10. Empty and trivia-only interpolation.
11. Unterminated outer string.
12. Unterminated interpolation.
13. Unterminated nested string and block comment.
14. Malformed inner expressions with absolute byte ranges.
15. Multiple top-level expressions and statement separators.
16. Unicode before and inside interpolation.
17. AST shape and left associativity.
18. Left-to-right runtime evaluation.
19. Exactly-once expression and `toString` evaluation.
20. Non-String and throwing `toString` behavior.

The following regression MUST pass:

```phalcom
System.print("\(String.new(")"))")
```

Its interpolation body is:

```phalcom
String.new(")")
```

The `)` inside the nested string does not close the interpolation.
