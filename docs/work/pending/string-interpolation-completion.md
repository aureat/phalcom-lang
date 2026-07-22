# Phalcom String Interpolation Correctness Implementation Specification

**Status:** Implementation contract
**Date:** 2026-07-22
**Normative dependency:** [`docs/spec/current/string-interpolation.md`](../../spec/current/string-interpolation.md)

> **For agentic workers:** Implement this specification test-first. Do not preserve current behavior where it conflicts with the normative language specification.

## 1. Goal

Make Phalcom string literals and interpolation fully conform to the normative language specification, including:

- conventional escape decoding;
- rejection of physical newlines in ordinary quoted strings;
- mode-aware interpolation boundary scanning;
- exact-one-expression parsing;
- precise UTF-8 byte diagnostics;
- reference AST lowering;
- left-to-right and exactly-once runtime behavior; and
- complete lexer, parser, and runtime regression coverage.

The motivating regression is:

```bash
./target/debug/phalcom -i 'System.print("\\(String.new(")"))")'
```

The Phalcom source seen by the front end is:

```phalcom
System.print("\(String.new(")"))")
```

The expression body is `String.new(")")`. The parenthesis inside the nested string must not affect the outer interpolation depth.

## 1.1 Accepted implementation decisions

PDR-0029 records these rulings. They do **not** reopen any language semantics
locked in Section 2. The historical options remain for provenance; the selected
option is binding.

| ID | Question | Options | Selected option | Why |
|---|---|---|---|---|
| OI-1 | What document is authoritative during implementation? | **A.** Depend on the canonical current-spec file above and promote its status from `Normative draft` to the repository's accepted/current status before implementation. **B.** Treat the draft as a temporary implementation contract. **C.** Copy it into a second canonical path. | **A** | One canonical source prevents split specifications. A feature implementation should not silently ratify a draft. |
| OI-2 | How should grammar represent physical newlines in strings? | **A.** Amend `string_char` so physical LF and CRLF are legal literal content. **B.** Keep the grammar's raw-LF, CRLF, and CR exclusion and defer multiline string literals. | **B** | Multiline syntax is a separate literal-design axis; ordinary double-quoted strings remain single-line. `\\n` and `\\r` remain valid escapes; triple-quoted syntax does not exist yet. |
| OI-3 | Which documentation is changed? | **A.** Update a named active-document set; preserve accepted ADR history, adding a supersession/status note only where it could mislead. **B.** Rewrite every historical mention. **C.** Update only code comments. | **A** | Historical ADR rationale is provenance, not an active contract. Unbounded rewrites destroy that provenance and make completion untestable. |
| OI-4 | How are required diagnostic codes represented? | **A.** Derive stable codes from `SyntaxErrorKind` and expose them through `SyntaxError::code()`: `string.invalid_escape`, `string.interpolation.unterminated`, `string.interpolation.empty`, and `string.raw_newline`. **B.** Store a second code field on `SyntaxError`. **C.** Add only Rust enum variants and document no stable codes. | **A** | Derived codes cannot disagree with error kinds and preserve the existing message-and-range representation. |
| OI-5 | Where do tests live? | **A.** Extend the existing lexer/parser test module and existing runtime fixture harness; create dedicated integration files only if those are the established local convention. **B.** Always create the two paths listed in Section 4. | **A** | Preserves repository test discovery and avoids test infrastructure changes unrelated to interpolation. Every required case remains mandatory. |
| OI-6 | How is arbitrary-UTF-8 robustness tested? | **A.** Add property tests only if an existing workspace test dependency and convention support them; otherwise add the deterministic adversarial corpus in Section 10.7. **B.** Introduce a new fuzz/property dependency. **C.** Omit robustness coverage. | **A** | Gives coverage without expanding dependencies; C violates the acceptance criterion. |
| OI-7 | Where is non-String `toString` validation enforced? | **A.** Use ordinary `String#+` validation; if it is currently permissive, repair that operation so its normal String-required error applies. **B.** Add a dedicated interpolation-only validation operation. **C.** Coerce or stringify a second time. | **A** | Preserves ordinary AST lowering and one runtime rule. B adds feature-specific runtime machinery; C violates the normative semantics. |
| OI-8 | Which ranges do synthetic lowering nodes receive? | **A.** Give synthetic `Add` and `GetProperty("toString")` nodes the full outer-string range. **B.** Use narrower segment ranges. | **A** | Matches the reference lowering and keeps inner-expression diagnostic ranges separate. Existing snapshots must be updated to this contract, not treated as an implicit competing specification. |

### OI-3 active-document set

Option OI-3A means updating these active surfaces:

- `docs/spec/current/string-interpolation.md`;
- `docs/spec/current/syntax/grammar.md`;
- `docs/spec/current/syntax/lexical.md`;
- `docs/spec/current/syntax/expressions.md`, if its interpolation summary needs the new escape/error wording;
- `docs/spec/current/syntax/README.md`, if it indexes the canonical interpolation document; and
- source comments in `phalcom-ast` that describe the old behavior.

Accepted ADRs remain historical records. Do not rewrite their original decision
text; add an explicit amendment or status note only when an unqualified current
claim remains misleading after the active-spec updates.

## 2. Locked design decisions

The implementer MUST use these decisions and MUST NOT reopen them during implementation.

1. Keep the existing `Token::StringInterp(Vec<StringSegment>)` architecture.
2. Keep interpolation expressions as raw source slices that are re-lexed and re-parsed.
3. Replace raw parenthesis counting with a mode-aware interpolation-body scanner.
4. Reuse the real string and comment scanners inside interpolation; do not implement reduced duplicate rules.
5. Adopt `\"`, `\\`, `\n`, `\t`, and `\r`.
6. Treat `\(` as the interpolation opener.
7. Reject every other escape.
8. Require exactly one expression after leading/trailing trivia.
9. Add distinct invalid-escape, unterminated-interpolation, empty-interpolation, and raw-newline-in-string diagnostics.
10. Reject physical LF, CRLF, and lone CR while scanning ordinary quoted-string content. Newlines remain valid in interpolation expression code.
11. Keep interpolation lowering as ordinary AST nodes.
12. Use exactly one `GetProperty("toString")` per expression segment.
13. Seed an empty String accumulator when the first segment is an expression.
14. Preserve left-associative concatenation.
15. Give synthetic `Add` and `GetProperty("toString")` nodes the full outer-string range.
16. Require runtime String validation for every `toString` result.
17. Preserve language-wide non-nesting block-comment behavior.

## 3. Current defects and code locations

The attached code establishes these current defects.

### 3.1 `phalcom-ast/src/lexer.rs`

`Lexer::scan_string` currently:

- recognizes only `\\` and `\(`;
- preserves every other escape literally;
- accepts physical LF, CRLF, and lone CR as ordinary string content;
- scans interpolation bodies by incrementing on every raw `(`;
- decrements on every raw `)`;
- ignores nested string and comment modes; and
- reports EOF inside interpolation as `UnterminatedString`.

`Lexer::skip_trivia` contains independent line-comment and block-comment scanning logic that should be extracted for reuse.

### 3.2 `phalcom-ast/src/token.rs`

`StringSegment::Expr` currently stores:

```rust
Expr {
    source: String,
    start: usize,
}
```

It lacks the body end, so an empty or trivia-only interpolation cannot be diagnosed using its exact body range without reconstructing it.

`LexicalError` lacks invalid-escape, unterminated-interpolation, and raw-newline-in-string variants.

Token documentation still describes `String.new` lowering and must be corrected.

### 3.3 `phalcom-ast/src/error.rs`

`SyntaxErrorKind` lacks:

- invalid string escape;
- unterminated string interpolation; and
- empty string interpolation; and
- raw newline in a string literal.

### 3.4 `phalcom-ast/src/parser.rs`

`push_lex_error` maps every unterminated string-like mode to expected `"`.

`Parser::desugar_string_interp` correctly emits `GetProperty("toString")`, but:

- its documentation still says `String.new`;
- an interpolation-only string can become the raw getter result;
- the first expression segment is not forced through String concatenation.

`Parser::parse_interp_expr` calls the whole-program parser and takes only the first statement. It does not reject a second expression or statement separator, and it widens empty-body errors to the entire string range.

## 4. Files to modify or create

Repository-relative paths:

```text
Modify:
  phalcom-ast/src/token.rs
  phalcom-ast/src/error.rs
  phalcom-ast/src/lexer.rs
  phalcom-ast/src/parser.rs
  docs/spec/current/syntax/grammar.md
  docs/spec/current/syntax/lexical.md
  docs/spec/current/string-interpolation.md
  active documentation and source comments named by OI-3
```

Place tests according to OI-5. All cases in Sections 10 through 13 remain
mandatory regardless of test-file layout.

Do not move unrelated code or perform broad parser refactoring.

## 5. Required data-model changes

### 5.1 `StringSegment`

Change the expression segment to carry the full body range relative to the current lexer input:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum StringSegment {
    Literal(String),
    Expr {
        source: String,
        range: std::ops::Range<usize>,
    },
}
```

Requirements:

- `range.start` is the first byte after `\(`.
- `range.end` is the byte position of the matching `)`.
- `source == input[range.clone()]`.
- For `\()`, `range.start == range.end`.
- Ranges remain local to the lexer input. The parser adds its own base offset.
- Empty literal segments remain omitted.

Do not add a second start field. `range.start` replaces it.

### 5.2 `LexicalError`

Add:

```rust
InvalidEscape(std::ops::Range<usize>),
UnterminatedInterpolation(std::ops::Range<usize>),
RawNewlineInString(std::ops::Range<usize>),
```

Range contracts:

- `InvalidEscape` covers the backslash and the following Unicode scalar.
- `UnterminatedInterpolation` begins at the backslash of the unmatched `\(` and ends at EOF.
- `RawNewlineInString` covers the complete physical newline: one byte for LF or lone CR, and both bytes for CRLF. The lexer reports it immediately and does not continue the current string across that newline.

A trailing backslash followed by EOF in an otherwise unclosed string remains `UnterminatedString`, because the string's closing quote is absent.

### 5.3 `SyntaxErrorKind`

Add stable variants:

```rust
#[error("Invalid string escape")]
InvalidStringEscape,

#[error("Unterminated string interpolation")]
UnterminatedInterpolation,

#[error("String interpolation requires an expression")]
EmptyInterpolation,

#[error("Raw newline is not allowed in a string literal")]
RawNewlineInString,
```

Expose stable codes without duplicated error state:

```rust
impl SyntaxErrorKind {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidStringEscape => "string.invalid_escape",
            Self::UnterminatedInterpolation => "string.interpolation.unterminated",
            Self::EmptyInterpolation => "string.interpolation.empty",
            Self::RawNewlineInString => "string.raw_newline",
            // Existing syntax errors receive stable codes as they are formalized.
            _ => "syntax.error",
        }
    }
}

impl SyntaxError {
    pub const fn code(&self) -> &'static str {
        self.kind.code()
    }
}
```

`SyntaxErrorKind` remains the sole source of truth for a diagnostic's code;
do not add a code field to `SyntaxError`.

## 6. Lexer design

### 6.1 Extract reusable comment scanners

Refactor comment consumption into focused helpers:

```rust
fn scan_line_comment(&mut self);

fn scan_block_comment(&mut self) -> Result<(), LexicalError>;
```

Contracts:

```text
scan_line_comment:
  precondition: cursor is on the first `/` of `//`
  consumes through the byte before LF, CRLF, or EOF
  leaves the newline unconsumed

scan_block_comment:
  precondition: cursor is on the first `/` of `/*`
  consumes through and including the first `*/`
  returns UnterminatedBlockComment(open..EOF) if no closer exists
  remains non-nesting
```

`skip_trivia` MUST call these helpers.

The interpolation scanner MUST call the same helpers.

### 6.2 Add interpolation-body scanning

Add a helper with this responsibility:

```rust
fn scan_interpolation_body(
    &mut self,
    interpolation_open: usize,
) -> Result<StringSegment, LexicalError>;
```

Precondition:

- `interpolation_open` is the byte position of the opening delimiter's backslash.
- `self.pos` is the first byte after `\(`.

Required behavior:

```rust
fn scan_interpolation_body(
    &mut self,
    interpolation_open: usize,
) -> Result<StringSegment, LexicalError> {
    let body_start = self.pos;
    let mut depth = 1usize;

    loop {
        match self.peek_at(0) {
            None => {
                return Err(LexicalError::UnterminatedInterpolation(
                    interpolation_open..self.pos,
                ));
            }

            Some(b'"') => {
                // Consume the complete nested string, including nested
                // interpolations, using the real string scanner.
                let _ = self.scan_string()?;
            }

            Some(b'/') if self.peek_at(1) == Some(b'/') => {
                self.scan_line_comment();
            }

            Some(b'/') if self.peek_at(1) == Some(b'*') => {
                self.scan_block_comment()?;
            }

            Some(b'(') => {
                depth += 1;
                self.pos += 1;
            }

            Some(b')') => {
                depth -= 1;
                if depth == 0 {
                    let body_end = self.pos;
                    let source = self.input[body_start..body_end].to_string();
                    self.pos += 1;
                    return Ok(StringSegment::Expr {
                        source,
                        range: body_start..body_end,
                    });
                }
                self.pos += 1;
            }

            Some(_) => {
                self.pos += self.char_len_at(self.pos);
            }
        }
    }
}
```

The final implementation may differ mechanically, but it MUST preserve every branch and contract above.

Important:

- Calling `scan_string()` recursively is intentional.
- Discarding the returned nested-string token is safe because the outer body still captures the original source slice.
- A lexical error from the nested string or comment propagates unchanged.
- The outer interpolation depth changes only in expression-code mode.

### 6.3 Rewrite escape decoding in `scan_string`

Process a backslash using maximal, deterministic matching:

```text
\(
\"
\\
\n
\t
\r
other => InvalidEscape
```

The implementation must consume escapes left-to-right.

Required decoded values:

```rust
literal.push('"');   // \"
literal.push('\\');  // \\
literal.push('\n');  // \n
literal.push('\t');  // \t
literal.push('\r');  // \r
```

For `\(`:

1. Flush a non-empty literal segment.
2. Mark the token as interpolated.
3. Record the delimiter's backslash position.
4. Advance past `\(`.
5. Call `scan_interpolation_body`.
6. Push its expression segment.

Unknown escapes:

```rust
let start = self.pos;
let next_len = self.char_len_at(self.pos + 1);
let end = self.pos + 1 + next_len;
self.pos = end;
return Err(LexicalError::InvalidEscape(start..end));
```

Guard the `self.pos + 1` access at EOF. At EOF return `UnterminatedString(open..self.pos)`.

Physical newlines are handled before ordinary literal-character consumption:

```text
LF       => RawNewlineInString(pos..pos + 1)
CRLF     => RawNewlineInString(pos..pos + 2)
lone CR  => RawNewlineInString(pos..pos + 1)
```

Advance through the complete reported sequence before returning the error. This
rule applies whenever `scan_string` is active, including a nested string inside
an interpolation. It does not apply to interpolation expression code: ordinary
Phalcom newline rules continue to govern the body between an interpolation
opener and its matching close.

### 6.4 Required lexer invariants

After implementation:

- The iterator always makes progress.
- Every successful token and error range lies on UTF-8 boundaries.
- `source == input[range]` for every expression segment.
- A nested string can contain interpolation recursively.
- A line comment does not consume its newline.
- A block comment consumes internal newlines and remains flat.
- The outer scanner resumes immediately after the interpolation's closing `)`.
- Physical LF, CRLF, and lone CR never become ordinary quoted-string content.

## 7. Parser and diagnostic design

### 7.1 Lexical-error lowering

Update `lex_error_to_syntax`:

```rust
LexicalError::InvalidEscape(span) => SyntaxError {
    kind: SyntaxErrorKind::InvalidStringEscape,
    range: add_offset(span, offset),
},

LexicalError::UnterminatedInterpolation(span) => SyntaxError {
    kind: SyntaxErrorKind::UnterminatedInterpolation,
    range: add_offset(span, offset),
},

LexicalError::RawNewlineInString(span) => SyntaxError {
    kind: SyntaxErrorKind::RawNewlineInString,
    range: add_offset(span, offset),
},
```

Retain existing mappings.

A small helper is recommended:

```rust
fn add_offset(span: Range<usize>, offset: usize) -> Range<usize> {
    (span.start + offset)..(span.end + offset)
}
```

### 7.2 REPL completeness

Update `push_lex_error`:

```rust
let expected_closer = match &err {
    LexicalError::UnterminatedBlockComment(_) => Some("`*/`"),
    LexicalError::UnterminatedString(_) => Some("`\"`"),
    LexicalError::UnterminatedInterpolation(_) => Some("`)`"),
    _ => None,
};
```

Do not co-emit `UnrecognizedEof` for invalid escapes, raw newlines, or malformed expressions.

The EOF point remains `lowered.range.end..lowered.range.end`.

### 7.3 Exact-one-expression entry point

Replace whole-program parsing in `parse_interp_expr`.

Use an isolated parser instance over the body source and require one expression followed only by newlines/trivia and EOF.

Recommended signature:

```rust
fn parse_interp_expr(
    &self,
    source: &str,
    body_range: Range<usize>,
) -> ParserResult<Expr>;
```

Required algorithm:

```rust
fn parse_interp_expr(
    &self,
    source: &str,
    body_range: Range<usize>,
) -> ParserResult<Expr> {
    let absolute_start = self.offset + body_range.start;
    let absolute_range =
        (self.offset + body_range.start)..(self.offset + body_range.end);

    let mut parser = Parser::new(source, absolute_start);

    if let Some(err) = parser.errors.first().cloned() {
        return Err(err);
    }

    parser.skip_newlines();

    if matches!(parser.peek(), Token::Eof) {
        return Err(SyntaxError {
            kind: SyntaxErrorKind::EmptyInterpolation,
            range: absolute_range,
        });
    }

    let expr = parser.parse_expr()?;

    parser.skip_newlines();

    if !matches!(parser.peek(), Token::Eof) {
        return Err(parser.error_here(vec![
            "end of interpolation".to_string(),
        ]));
    }

    Ok(expr)
}
```

The implementation MUST reject:

```phalcom
"\(1; 2)"
"\(1
  2)"
"\(let x = 1)"
"\(return 1)"
```

It MUST accept a block as one expression:

```phalcom
"\({ first(); second() })"
```

Do not call `parse_source` from `parse_interp_expr`.

### 7.4 AST lowering

Update `desugar_string_interp` to consume the new segment range.

Each expression segment:

```rust
let inner = self.parse_interp_expr(&source, range)?;
let stringified = Expr::GetProperty(Box::new(GetPropertyExpr {
    object: inner,
    property: "toString".to_string(),
    range: outer_range,
}));
```

Use a String accumulator when the first segment is an expression.

One correct structure is:

```rust
let starts_with_expr =
    matches!(segments.first(), Some(StringSegment::Expr { .. }));

let mut acc = if starts_with_expr {
    Some(Expr::String {
        value: String::new(),
        range: outer_range,
    })
} else {
    None
};

for segment in segments {
    let part = match segment {
        StringSegment::Literal(value) => Expr::String {
            value,
            range: outer_range,
        },
        StringSegment::Expr { source, range } => {
            let inner = self.parse_interp_expr(&source, range)?;
            Expr::GetProperty(Box::new(GetPropertyExpr {
                object: inner,
                property: "toString".to_string(),
                range: outer_range,
            }))
        }
    };

    acc = Some(match acc {
        None => part,
        Some(left) => Expr::Binary(Box::new(BinaryExpr {
            op: BinaryOp::Add,
            left,
            right: part,
            range: outer_range,
        })),
    });
}
```

The final function may retain a defensive empty-String fallback, although a `StringInterp` token always contains an expression segment.

Required shapes:

```text
"a \(x) b"
  Add(Add(String("a "), GetProperty(x, "toString")), String(" b"))

"\(x)"
  Add(String(""), GetProperty(x, "toString"))

"\(a)\(b)"
  Add(Add(String(""), GetProperty(a, "toString")),
                         GetProperty(b, "toString"))
```

Every expression occurs once. Every expression segment has one and only one `toString` getter.

### 7.5 Documentation cleanup

Update all comments and docs that describe interpolation as `String.new(expr)`.

The authoritative wording is:

```text
Each interpolation expression is evaluated once and sent the `toString`
getter once. The String results are concatenated from left to right.
```

Update `Token::StringInterp`, `StringSegment`, `scan_string`, and `desugar_string_interp` documentation in the same change that alters behavior.

## 8. Runtime conformance

### 8.1 Evaluation order

Confirm the compiler evaluates `BinaryExpr` operands left-to-right and evaluates a `GetPropertyExpr` receiver once.

If this is not already guaranteed, fix the compiler before declaring interpolation conformant.

### 8.2 String-result validation

The seeded empty String accumulator makes interpolation-only strings use String concatenation.

Confirm `String#+`:

- requires its right operand to be a String;
- returns a String; and
- raises the normal stable type error for a non-String operand.

Apply OI-7. Do not silently stringify the non-String result a second time.

### 8.3 Getter behavior

Confirm `toString` is dispatched as a getter selector, not a zero-argument method selector.

Do not replace `GetProperty` with `MethodCall`.

### 8.4 Exceptions

No interpolation-specific catch or retry is permitted. Expression failures, missing getters, getter failures, and String type errors propagate through ordinary runtime mechanisms.

## 9. Test strategy

Use three layers:

1. Lexer tests: token segmentation, escapes, modes, and local ranges.
2. Parser tests: exact expression grammar, diagnostics, offsets, and AST shape.
3. Runtime tests: order, exactly-once behavior, type checking, and exceptions.

Each bug fix begins with a failing test. Run the narrow test before the implementation change and verify that its failure demonstrates the intended defect.

## 10. Lexer test requirements

Use raw Rust strings for Phalcom source wherever possible.

A helper equivalent to the current inline `spans` helper is sufficient:

```rust
fn lex(src: &str) -> Vec<(Token, usize, usize)> {
    Lexer::new(src)
        .map(|item| {
            let (start, token, end) = item.expect("source should lex");
            (token, start, end)
        })
        .collect()
}
```

Add direct error helpers for expected failures.

### 10.1 Plain strings and escapes

Required cases:

| Test | Phalcom source | Expected value |
|---|---|---|
| empty | `""` | `""` |
| quote | `"\""` | `"` |
| backslash | `"\\"` | `\` |
| newline | `"\n"` | U+000A |
| tab | `"\t"` | U+0009 |
| carriage return | `"\r"` | U+000D |
| combined | `"a\nb\tc\rd"` | decoded controls |
| escaped interpolation opener | `"\\("` | literal `\(` |
| backslash then interpolation | `"\\\(x)"` | literal `\`, then expression `x` |

Reject at least:

```text
"\q"
"\u"
"\0"
```

Assert the invalid-escape range covers the two source characters, or the full UTF-8 scalar after the backslash for a non-ASCII unknown escape.

Reject physical newlines in ordinary quoted strings:

| Test | Source shape | Expected range |
|---|---|---|
| raw LF | quote, LF, closing quote | LF byte |
| raw CRLF | quote, CRLF, closing quote | complete CRLF sequence |
| lone raw CR | quote, CR, closing quote | CR byte |
| nested raw newline | raw newline in a nested string inside an interpolation | newline in nested string |
| post-interpolation raw newline | completed interpolation followed by a raw newline before outer quote | newline sequence |

Each case reports `SyntaxErrorKind::RawNewlineInString`, message `Raw newline is
not allowed in a string literal`, and code `string.raw_newline`. Confirm that
escaped newline and carriage-return forms remain valid decoded escapes.

### 10.2 Segment boundaries

Required cases:

```phalcom
"\(x)"
"a \(x)"
"\(x) b"
"a \(x) b"
"\(a)\(b)"
"a \(b)c \(d)"
```

Assert:

- segment order;
- decoded literal values;
- exact expression source;
- exact local expression body range;
- complete outer token span;
- one EOF token.

### 10.3 Parenthesis balancing

Required valid cases:

```phalcom
"\(outer(inner(value)))"
"\((value))"
"\(String.new(")"))"
"\(String.new("("))"
"\(f("(", g(")")))"
```

The exact reported regression MUST be included.

### 10.4 Comments

Required valid cases:

```phalcom
"\(1 /* ) */ + 2)"
"\(1 /* ( */ + 2)"
"\(1 + // )
2)"
"\("/* not a comment ) */")"
"\("// not a comment )")"
"\(1 /* " ) ( */ + 2)"
```

Assert the captured source includes comments and newlines exactly as written.

### 10.5 Nested interpolation

Required:

```phalcom
"\("nested \(value)")"
"\("left \("inner") right")"
```

Assert the outer body ends after the complete nested string, not at a parenthesis inside it.

### 10.6 Unterminated modes

Required primary errors and ranges:

| Source shape | Primary error | Expected closer |
|---|---|---|
| opening quote, no quote | UnterminatedString | `"` |
| `\(`, no matching `)` | UnterminatedInterpolation | `)` |
| closed interpolation, no outer quote | UnterminatedString | `"` |
| unclosed nested string | UnterminatedString at nested quote | `"` |
| unclosed block comment | UnterminatedComment at `/*` | `*/` |

Include Unicode before each open mode to verify byte positions.

Raw newlines are not unterminated modes. They report only
`RawNewlineInString`; no `UnrecognizedEof` diagnostic is co-emitted.

### 10.7 Lexer robustness properties

Apply OI-6. Do not add a new fuzz dependency solely for this change.

Properties:

```text
arbitrary UTF-8 never panics
successful spans are ordered
all span boundaries are UTF-8 boundaries
expression source equals the original input slice
parentheses inserted inside nested strings do not change outer closure
parentheses inserted inside comments do not change outer closure
```

If OI-6 selects deterministic coverage, include cases for every listed
property, with UTF-8 inputs and mode-boundary adversaries.

## 11. Parser test requirements

### 11.1 Exactly one expression

Accept:

```phalcom
"\(x)"
"\((x))"
"\(f(a, b))"
"\(
x
)"
"\(/* comment */ x)"
"\({ first(); second() })"
```

Reject:

```phalcom
"\()"
"\( /* comment only */ )"
"\(let x = 1)"
"\(return 1)"
"\(1; 2)"
"\(1
2)"
"\(1 extra)"
```

For empty and trivia-only cases, assert `SyntaxErrorKind::EmptyInterpolation`.

For additional input, assert the range begins at the first trailing token.

### 11.2 Malformed inner expressions

Required:

```phalcom
"\(1 +)"
"\(f(1,))"
"\(object.)"
"\((1)"
"\(1 2)"
```

Assert the ordinary parser error kind and absolute body-local range.

### 11.3 Nonzero offsets and Unicode

Call the parser with a nonzero base offset.

Required source examples:

```phalcom
"α🙂 \(value)"
"α \(€)"
```

Assert:

```text
absolute diagnostic start =
    parser base offset +
    UTF-8 byte offset inside source
```

Do not calculate expected positions using `.chars().count()`.

### 11.4 AST shape helpers

Do not rely only on full `Debug` snapshots. Add focused assertion helpers:

```rust
fn assert_string(value: &Expr, expected: &str);
fn assert_add(value: &Expr) -> (&Expr, &Expr);
fn assert_to_string_getter(value: &Expr) -> &Expr;
```

Required structural tests:

1. `"\(x)"` is `Add(String(""), GetProperty(x, "toString"))`.
2. `"a \(x)"` is `Add(String("a "), GetProperty(x, "toString"))`.
3. `"a \(x) b"` is left-nested with final `String(" b")`.
4. `"\(a)\(b)"` is left-nested from an empty String accumulator.
5. Every expression segment produces one getter.
6. No getter is represented by `MethodCall`.
7. Inner expressions appear once.
8. Segment order is preserved.
9. A plain string remains one `Expr::String`.
10. Synthetic node ranges remain consistent with the established outer-string convention.

## 12. Runtime test requirements

Create small Phalcom fixtures inside the existing runtime-test harness.

Use a trace object, counter, or mutable list to make evaluation observable. Do not infer exactly-once behavior only from final text.

### 12.1 Left-to-right and exactly once

A test equivalent to:

```phalcom
let trace = List.new()

class Probe {
    construct new(label:, trace:) {
        _label = label
        _trace = trace
    }

    value {
        _trace.add("expr:" + _label)
        self
    }

    toString {
        _trace.add("string:" + _label)
        _label
    }
}

let a = Probe.new(label: "a", trace: trace)
let b = Probe.new(label: "b", trace: trace)

let result = "\(a.value)-\(b.value)"
```

Assert:

```text
result == "a-b"

trace == [
  "expr:a",
  "string:a",
  "expr:b",
  "string:b"
]
```

Adapt construction syntax to the landed Phalcom API, but preserve the observable sequence.

### 12.2 No literal stringification

Verify literal segments do not invoke `toString`.

### 12.3 Expression throws

The first interpolation expression throws. Assert:

- the same exception reaches the caller;
- its `toString` is not invoked;
- later expressions are not evaluated.

### 12.4 `toString` throws

The expression evaluates, then `toString` throws. Assert:

- the same exception reaches the caller;
- later expressions are not evaluated;
- no operation is retried.

### 12.5 Non-String `toString`

Define an object whose `toString` getter returns a Number or Bool.

Test both:

```phalcom
"\(bad)"
"prefix \(bad) suffix"
```

Both MUST raise the same String-required runtime type error.

### 12.6 User override

Define a user object whose `toString` getter returns a custom visible value. Assert interpolation uses the override rather than native/default object rendering.

### 12.7 Missing getter

An object without a usable `toString` getter follows ordinary missing-message behavior. Interpolation must not substitute native formatting silently.

## 13. Diagnostic test matrix

Every case below needs an assertion for kind and byte range.

| Category | Minimal case |
|---|---|
| invalid escape | `"\q"` |
| raw LF in ordinary string | quote, LF, closing quote |
| raw CRLF in ordinary string | quote, CRLF, closing quote |
| lone raw CR in ordinary string | quote, CR, closing quote |
| raw newline in nested ordinary string | interpolation containing a nested string with LF, CRLF, or CR |
| raw newline after interpolation | completed interpolation followed by LF, CRLF, or CR before outer quote |
| unterminated outer string | `"abc` |
| unterminated interpolation | `"\(abc` |
| outer quote missing after interpolation | `"\(abc)` |
| empty body | `"\()"` |
| trivia-only body | `"\( /*x*/ )"` |
| malformed expression | `"\(1 +)"` |
| second statement | `"\(1; 2)"` |
| second expression after newline | `"\(1\n2)"` |
| nested string missing quote | interpolation containing an unmatched nested `"` |
| nested block comment missing closer | interpolation containing unmatched `/*` |
| Unicode offset | `"α🙂 \(1 +)"` |
| nonzero parser offset | same source parsed with offset 100 |

For every unterminated lexical mode, also assert the co-emitted `UnrecognizedEof` expected closer.

## 14. Documentation changes

### 14.1 Grammar

Update the grammar production near the current string-escape rule to list:

```text
\"  \\  \n  \t  \r  \(
```

Make it explicit that `\(` opens interpolation and is not part of the decoded literal.
Keep the grammar's raw-LF, CRLF, and CR exclusion. Triple-quoted syntax does
not exist. Multiline string literal syntax is deferred by PDR-0029.

### 14.2 Lexical structure

Document mode-aware closure:

```text
The closing parenthesis is balanced in expression-code mode. Parentheses
inside nested strings and comments do not affect interpolation depth.
```

Document unknown escapes and raw LF/CRLF/CR in ordinary quoted strings as
errors. State that newlines remain governed by ordinary expression rules inside
an interpolation body.

### 14.3 Desugaring text

Update the active-document set in OI-3. Replace active `String.new(expr)`
interpolation examples with `expr.toString` getter dispatch and
left-associative concatenation from Section 7.4. Preserve historical ADR text
unless OI-3 calls for a clarifying amendment or status note.

### 14.4 Diagnostics

Document invalid escape, unterminated interpolation, empty interpolation, and
raw newline in string categories; their stable codes; and expected REPL closers.

## 15. Implementation sequence

Complete these tasks in order. Each task must finish with its focused tests passing.

### Task 1: Lock failing regressions

Add lexer and parser tests for:

```text
nested string containing `)`
nested string containing `(`
parenthesis in line comment
parenthesis in block comment
empty interpolation
two expressions in one interpolation
unterminated interpolation expects `)`
raw LF, CRLF, and lone CR in ordinary strings
raw newline in a nested ordinary string
raw newline after a completed interpolation segment
```

Run:

```bash
cargo test -p phalcom-ast string_interpolation -- --nocapture
```

Expected before fixes: multiple failures demonstrating current behavior.

Commit:

```bash
git add phalcom-ast
git commit -m "test(ast): expose string interpolation correctness gaps"
```

### Task 2: Add error and segment data

Modify `token.rs` and `error.rs`.

Update exhaustive matches in `parser.rs`.

Run:

```bash
cargo test -p phalcom-ast
```

Expected: compile succeeds; new failing behavior tests remain.

Commit:

```bash
git add phalcom-ast/src/token.rs phalcom-ast/src/error.rs phalcom-ast/src/parser.rs
git commit -m "refactor(ast): model interpolation ranges and errors"
```

### Task 3: Refactor comment scanning

Extract shared comment helpers and retain all existing trivia behavior.

Run:

```bash
cargo test -p phalcom-ast lexer
```

Expected: all pre-existing lexer tests pass.

Commit:

```bash
git add phalcom-ast/src/lexer.rs
git commit -m "refactor(ast): share comment scanners across lexer modes"
```

### Task 4: Implement escapes and mode-aware interpolation

Implement Sections 6.2 and 6.3.

Run:

```bash
cargo test -p phalcom-ast string_interpolation -- --nocapture
cargo test -p phalcom-ast lexer
```

Expected: lexer-mode and escape tests pass; parser exact-expression tests may still fail.

Commit:

```bash
git add phalcom-ast
git commit -m "fix(ast): scan interpolation with nested lexical modes"
```

### Task 5: Enforce exactly one expression

Replace `parse_source(...).statements.next()` with the isolated expression parser.

Run:

```bash
cargo test -p phalcom-ast string_interpolation -- --nocapture
cargo test -p phalcom-ast parser
```

Expected: empty, trailing-input, malformed-expression, and offset tests pass.

Commit:

```bash
git add phalcom-ast
git commit -m "fix(ast): require one expression per interpolation"
```

### Task 6: Lock AST lowering

Seed the empty accumulator for expression-leading strings and add structural tests.

Run:

```bash
cargo test -p phalcom-ast string_interpolation -- --nocapture
```

Expected: all lexer and parser interpolation tests pass.

Commit:

```bash
git add phalcom-ast
git commit -m "fix(ast): make interpolation lowering always produce String"
```

### Task 7: Verify runtime semantics

Add runtime tests from Section 12. Inspect and fix runtime/compiler behavior only where a test proves nonconformance.

Run:

```bash
cargo test -p phalcom-core string_interpolation -- --nocapture
```

Expected: all order, exactly-once, override, exception, and type tests pass.

Commit:

```bash
git add phalcom-core
git commit -m "test(core): enforce interpolation runtime semantics"
```

If runtime code changes are necessary, use a separate focused commit:

```bash
git commit -m "fix(core): validate interpolation stringification results"
```

### Task 8: Update specifications and stale comments

Apply Section 14 and OI-1 through OI-4. Do not copy the normative document to
a second path.

Run:

```bash
rg -n "String\.new\(.*interpol|interpol.*String\.new|Any other `\\\\x` is left verbatim" \
  docs phalcom-ast/src
```

Expected: no stale interpolation contract remains.

Commit:

```bash
git add docs phalcom-ast/src
git commit -m "docs: specify string interpolation correctness"
```

### Task 9: Full verification

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then run the original reproduction:

```bash
cargo run -- -i 'System.print("\\(String.new(")"))")'
```

Expected: successful parse and execution with the nested string's `)` treated as data.

Commit formatting-only changes only if `cargo fmt --check` required them:

```bash
git add .
git commit -m "style: format interpolation changes"
```

## 16. Acceptance criteria

The work is complete only when all statements below are true.

### Lexer

- [ ] Every documented escape decodes correctly.
- [ ] Unknown escapes fail with exact ranges.
- [ ] Physical LF, CRLF, and lone CR fail immediately with `RawNewlineInString` and `string.raw_newline`; escaped newline and carriage-return forms remain valid.
- [ ] Nested strings do not affect outer interpolation depth.
- [ ] Line and block comments do not affect depth.
- [ ] Nested interpolation inside nested strings works.
- [ ] Unterminated interpolation is distinct from unterminated string.
- [ ] UTF-8 range boundaries are exact.
- [ ] The lexer never panics on arbitrary UTF-8 input.

### Parser

- [ ] Empty and trivia-only bodies fail precisely.
- [ ] Exactly one expression is required.
- [ ] Additional statements and expressions are rejected.
- [ ] Inner diagnostics retain absolute byte ranges.
- [ ] Every expression segment produces one getter send.
- [ ] Getter selector is `toString`, not `toString()`.
- [ ] Concatenation is left-associative.
- [ ] Expression-leading interpolation begins with an empty String accumulator.
- [ ] Plain strings remain plain String AST nodes.

### Runtime

- [ ] Expressions evaluate left-to-right.
- [ ] Every expression evaluates once.
- [ ] Every `toString` getter runs once.
- [ ] User overrides are honored.
- [ ] Non-String results fail consistently.
- [ ] Exceptions propagate unchanged.
- [ ] Later segments do not run after failure.
- [ ] Every successful interpolation result is a String.

### Documentation and regression

- [ ] Grammar and lexer behavior agree.
- [ ] No active documentation names `String.new` as interpolation lowering.
- [ ] New diagnostics and expected closers are documented.
- [ ] `SyntaxError::code()` derives stable interpolation and raw-newline codes from `SyntaxErrorKind`.
- [ ] The original reproduction passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.

## 17. Non-goals

This work does not:

- add raw strings, heredocs, triple-quoted strings, or alternate quote delimiters;
- make block comments nest;
- add format specifiers such as `\(value:hex)`;
- add interpolation-specific AST variants;
- change general statement-newline rules;
- change the selector identity of `toString`;
- introduce implicit coercion for a non-String `toString` result; or
- refactor unrelated lexer or parser architecture.
