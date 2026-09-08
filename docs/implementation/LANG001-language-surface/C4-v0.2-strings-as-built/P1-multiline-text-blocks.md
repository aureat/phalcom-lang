# Phalcom Multiline Text Blocks (`"""`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit interpreted multiline text blocks delimited by `"""`, with closing-delimiter-defined indentation, normalized physical line endings, ordinary Phalcom escapes, ordinary `\(expr)` interpolation, precise diagnostics, REPL continuation, and correct LSP semantic-token behavior—without introducing a second runtime string type or a parallel interpolation implementation.

**Architecture:** Extend the hand-written lexer with a multiline-string lexical mode that ultimately emits the existing `Token::String` / `Token::StringInterp` shapes. Keep the parser AST lowering and runtime `String` semantics unchanged. Add dedicated lexical/syntax diagnostics for text-block structure, wire unterminated text blocks into the existing parser-driven REPL completeness signal, and make LSP semantic-token encoding line-safe for tokens that span physical lines. Ratify the surface as PDR-0034 and update the canonical lexical/interpolation specifications.

**Tech Stack:** Rust workspace (`phalcom-ast`, `phalcom-core`, `phalcom-repl`, `phalcom-lsp`), hand-written UTF-8 byte lexer, recursive-descent/Pratt parser, `insta` parser/lexer snapshots, end-to-end `.ph` language corpus, Tower LSP semantic tokens.

---

## 0. Repository Baseline and Scope Lock

This plan is grounded against current `aureat/phalcom-lang` `main` at:

```text
61dae3400ba810d8f709725974e3c51838762905
```

Inspected on **2026-08-15**.

Key current files and observed blob SHAs:

| File | Baseline SHA | Why it matters |
|---|---:|---|
| `phalcom-ast/src/lexer.rs` | `10a6ce5aead22dba317c0c0fa7066ba56df23434` | String scanning, interpolation scanning, newline handling |
| `phalcom-ast/src/token.rs` | `8cb54eb53b319ac0b04922407c8823d8783e451c` | `Token`, `StringSegment`, `LexicalError` |
| `phalcom-ast/src/parser.rs` | `69f51dfb4b6e4228ae7afb70ec3b5d0ee8f041bf` | Lex-error lowering, REPL EOF continuation, interpolation lowering |
| `phalcom-ast/src/error.rs` | current `main` | Stable syntax diagnostic kinds/codes |
| `phalcom-ast/tests/lexer.rs` | `08bbd020e895e099033adb7276a7537603f33bb3` | Direct lexer regression tests |
| `phalcom-ast/tests/parser.rs` | `ea87ee3dc8b79b7811f70ff4c2401940c249d2dc` | Parser/AST snapshots and string parse tests |
| `phalcom-ast/tests/probe_continuation.rs` | `eb12229ef35a60bc4d63b0493ad535960ab68281` | Parser-driven REPL completeness contract |
| `phalcom-lsp/src/semantic_tokens.rs` | `7d9fa9fcd7fbaa32bc6020e0a158f79fe2bec73c` | Lexer-driven semantic tokens; currently assumes single-line token ranges |
| `phalcom-lsp/src/line_index.rs` | `c19013e6da85768bd5a7e162d60f0564e08b60bb` | UTF-8 byte ↔ UTF-16 LSP coordinate conversion |
| `phalcom-repl/src/highlighter.rs` | `7b9ad255bba21adca5aa78be81790e8836397ff3` | Reuses lexer token boundaries for string coloring |
| `docs/spec/current/syntax/lexical.md` | current `main` | Canonical lexical grammar; explicitly defers multiline strings |
| `docs/spec/current/string-interpolation.md` | current `main` | Accepted interpolation semantics and source-range contract |
| `docs/pdr/0029-string-literals-and-interpolation-completion.md` | `2ed757b491b7d6573d250d08da0a78d9e1af3624` | Explicitly defers multiline literals |
| `docs/work/deferred/multiline-string-literals.md` | `b1ff8ac5d4168a2bfca2430545a1c0144934c315` | Deferred-design marker |
| `docs/work/analyses/multiline-string-literals.md` | `1c4bc66110769e3f654d36b5978cb2229d4d82a5` | Existing analysis containing the now-approved semantics |
| `docs/pdr/STATUS.md` | `fce98eb6e567097331c824fe74f0ef9a620e5371` | PDR tracker; latest number is 0033 |

### Scope lock

Implement **one new surface syntax**, not a new value category:

```phalcom
message = """
    Hello, \(name).

    Welcome to Phalcom.
    """
```

The resulting value is an ordinary `String`.

Do **not** add:

- a `MultilineString` runtime class;
- an AST `Expr::MultilineString` variant;
- a `Token::MultilineString` public token variant;
- raw-string semantics;
- heredocs;
- implicit string concatenation;
- backslash physical-line continuation in ordinary `"..."` strings;
- `#"""..."""` multiline quoted symbols;
- a separate interpolation evaluator or compiler opcode;
- heuristic/common-minimum dedentation.

These are independent language-design axes and must not leak into this implementation.

---

# 1. Ratified Surface Semantics

The implementation must encode these rules literally.

## 1.1 Grammar

Conceptually:

```text
MULTILINE-STRING :=
    `"""`
    HSPACE*
    PHYSICAL-NEWLINE
    MULTILINE-CONTENT
    CLOSING-LINE

CLOSING-LINE :=
    MARGIN
    `"""`
    HSPACE*
    (PHYSICAL-NEWLINE | EOF)

HSPACE := " " | "\t"

PHYSICAL-NEWLINE := "\n" | "\r\n"
```

The source newline after the opening delimiter is structural. The source newline immediately before the closing line is structural. Neither becomes part of the value.

Ordinary `"..."` strings remain single-line and retain `string.raw_newline`.

## 1.2 Opening delimiter

Valid:

```phalcom
"""
text
"""
```

Also valid:

```phalcom
"""
text
"""
```

Spaces/tabs between opening `"""` and its physical newline are ignored.

Invalid:

```phalcom
"""text
"""
```

A non-horizontal-whitespace character after the opening delimiter before the first physical newline is a structural error.

Special EOF rule:

```phalcom
"""
```

with EOF immediately after the opening delimiter—or after only spaces/tabs—is **incomplete**, not irrecoverably malformed. It reports an unterminated multiline string and participates in REPL continuation.

## 1.3 Closing delimiter and margin

The closing delimiter is recognized only at the **top-level multiline-string lexical mode** on a physical line whose shape is:

```text
<margin>"""<optional spaces/tabs><newline-or-EOF>
```

The exact bytes before `"""` on that line are the block margin.

Example:

```phalcom
message = """
    first
        second
    third
    """
```

Value:

```text
first
    second
third
```

The margin is four ASCII spaces.

### Exact-prefix rule

Margin is a **source prefix**, not a visual column count.

If the closing line begins:

```text
\t··"""
```

(where `·` denotes a space), every nonblank source line in the block must begin with the exact byte sequence:

```text
\t··
```

A line beginning with four spaces is not considered equivalent to one tab. No tab-width policy is needed anywhere in the lexer, formatter, or LSP.

This is the deterministic operational meaning of “at least the closing delimiter’s indentation.”

## 1.4 Nonblank-line validation

Every nonblank physical line between the opening structural newline and closing line must start with the exact margin prefix.

If it does not, emit:

```text
string.multiline.indentation
```

No minimum-common-indent fallback. No partial trimming.

Example, invalid:

```phalcom
message = """
    first
  second
    """
```

because `second` does not begin with the four-space closing margin.

## 1.5 Blank lines

A blank physical line is a line containing only spaces/tabs before its physical newline or before the content boundary.

Blank lines:

- are exempt from the minimum-margin requirement;
- have all source-only horizontal whitespace discarded;
- contribute exactly one normalized `\n` when they are semantically between content lines.

This deliberately prevents source formatting whitespace on blank lines from becoming runtime data.

Example:

```phalcom
message = """
    first

    second
    """
```

Value:

```text
first

second
```

The spaces on the blank line do not survive.

## 1.6 Additional indentation

After stripping the exact margin prefix, all additional indentation on nonblank lines is value data.

```phalcom
message = """
    first
        nested
          deeper
    """
```

Value:

```text
first
    nested
      deeper
```

## 1.7 Physical newline normalization

Inside a multiline text block:

- LF source newline → `\n`
- CRLF source newline → `\n`

A raw lone CR is not a valid physical line ending. Reject it with a multiline-specific lexical diagnostic rather than silently treating it as a line break.

Escaped `\r` remains valid and produces U+000D as it does in ordinary strings.

## 1.8 Escapes

Use exactly the ordinary interpreted-string escape table:

| Source | Value |
|---|---|
| `\"` | `"` |
| `\\` | `\` |
| `\n` | LF |
| `\t` | TAB |
| `\r` | CR |
| `\(` | starts interpolation |

Every other escape remains `string.invalid_escape`.

Do not create a multiline-specific escape table.

## 1.9 Interpolation

`\(` starts interpolation exactly as in ordinary strings.

```phalcom
message = """
    Hello, \(user.name)!
    """
```

Interpolation:

- parses exactly one expression;
- preserves current left-to-right lowering;
- dispatches `toString` exactly once;
- preserves current source-range semantics;
- supports nested ordinary strings;
- supports nested multiline strings after this feature lands;
- may itself span physical lines.

There is no multiline-specific runtime interpolation path.

## 1.10 Triple quotes inside content

Only an isolated top-level closing-line candidate terminates the block.

Therefore triple quotes embedded in content are legal when they do not satisfy the closing-line shape:

```phalcom
text = """
    She wrote """hello""" in the note.
    """
```

Value:

```text
She wrote """hello""" in the note.
```

Likewise:

```phalcom
text = """
    """not a delimiter
    """
```

has a first content line beginning with the literal text `"""not a delimiter`.

This sharply reduces delimiter-collision problems without adding variable-length quote delimiters.

## 1.11 Closing-line trailing whitespace

Allow spaces/tabs after the closing `"""`, but no other character before newline/EOF.

The lexer token should end immediately after the three quotes. Ordinary lexer trivia handling consumes any following horizontal whitespace, and the following physical newline remains an ordinary statement newline token.

This preserves statement termination and avoids hiding the post-delimiter newline inside the string token.

## 1.12 No multiline quoted symbols

Keep:

```phalcom
#"symbol"
```

single-line.

Do not interpret:

```phalcom
#"""
...
"""
```

as a quoted symbol. Under current tokenization, `#` followed by `"` enters `scan_quoted_symbol`; explicitly guard/document this surface rather than accidentally making it multiline.

---

# 2. Architectural Decision: Reuse Existing String Tokens

## Why

Current public front-end representation is already correct:

```rust
Token::String(String)
Token::StringInterp(Vec<StringSegment>)
```

and:

```rust
pub enum StringSegment {
    Literal(String),
    Expr {
        source: String,
        range: Range<usize>,
    },
}
```

A multiline block differs only in **source spelling and lexical normalization**. After scanning:

```phalcom
"""
    hello
    """
```

is semantically indistinguishable from:

```phalcom
"hello"
```

Likewise:

```phalcom
"""
    hello \(name)
    """
```

can feed exactly the same interpolation lowering as:

```phalcom
"hello \(name)"
```

## Consequences

No production change should be required in:

- AST string expression variants;
- compiler string constant emission;
- `String` object allocation;
- string concatenation semantics;
- `toString` dispatch;
- VM bytecode;
- GC;
- runtime type system.

This is a major simplification and should be protected by tests. If implementation begins adding runtime multiline-string machinery, stop: the design has drifted.

---

# 3. Lexer Design

Primary file:

```text
phalcom-ast/src/lexer.rs
```

Primary anchors on baseline:

- `Lexer::scan_token`
- `Lexer::scan_interpolation_body`
- `Lexer::scan_string`
- `Lexer::scan_quoted_symbol`
- `Lexer::char_len_at`

## 3.1 Centralize string-mode dispatch

### Current

`scan_token` directly does:

```rust
b'"' => self.scan_string(),
```

and `scan_interpolation_body` directly does:

```rust
Some(b'"') => {
    let _ = self.scan_string()?;
}
```

That becomes wrong when `"""` can appear inside an interpolation expression.

### Edit

Add a private dispatcher:

```rust
fn scan_string_like(&mut self) -> Result<Token, LexicalError> {
    if self.peek_at(0) == Some(b'"')
        && self.peek_at(1) == Some(b'"')
        && self.peek_at(2) == Some(b'"')
    {
        self.scan_multiline_string()
    } else {
        self.scan_string()
    }
}
```

Change `scan_token`:

```rust
b'"' => self.scan_string_like(),
```

Change `scan_interpolation_body`:

```rust
Some(b'"') => {
    let _ = self.scan_string_like()?;
}
```

Do not duplicate quote-lookahead logic in two callers.

### Quoted symbol guard

Keep this branch before ordinary quote dispatch:

```rust
b'#' if self.peek_at(1) == Some(b'"') => self.scan_quoted_symbol(),
```

`scan_quoted_symbol` continues to consume one opening quote only and reject raw newline.

Add a test proving `#"""...` is not accepted as a multiline string symbol.

## 3.2 Implement multiline scanning as a two-phase lexical operation

Do **not** dedent decoded text before the closing margin is known.

Do **not** decode the entire raw block into a temporary string and then split lines: escaped `\n` would become indistinguishable from a physical source newline, corrupting indentation semantics.

Use this structure:

```rust
fn scan_multiline_string(&mut self) -> Result<Token, LexicalError> {
    let open = self.pos;

    let body_start = self.scan_multiline_opening(open)?;
    let boundary = self.discover_multiline_boundary(open, body_start)?;

    self.validate_multiline_margin(
        body_start,
        boundary.value_end,
        &boundary.margin,
    )?;

    let token = self.decode_multiline_body(
        body_start,
        boundary.value_end,
        &boundary.margin,
    )?;

    self.pos = boundary.close_end;
    Ok(token)
}
```

Recommended private metadata:

```rust
struct MultilineBoundary {
    /// First source byte after opening structural newline.
    body_start: usize,

    /// End of semantic content, excluding newline immediately before close.
    value_end: usize,

    /// Byte offset of first quote in closing delimiter.
    close_start: usize,

    /// Byte offset immediately after closing `"""`.
    close_end: usize,

    /// Exact spaces/tabs before closing delimiter.
    margin: String,
}
```

This struct can remain private to `lexer.rs`.

### Why two phase

The closing delimiter supplies the margin. Before it is found, the lexer cannot know what to strip.

Two-phase scanning keeps three concerns cleanly separated:

1. **Discover lexical boundary** while respecting interpolation/nested strings.
2. **Validate source indentation** against the discovered margin.
3. **Decode semantic value** while stripping only physical source margins.

This is easier to audit than trying to retroactively mutate already-decoded `StringSegment::Literal` values.

The cost is a second linear pass over each multiline literal. Accept it. Correctness is substantially more important here, and even very large text blocks remain O(n).

## 3.3 Opening scanner

Add:

```rust
fn scan_multiline_opening(&mut self, open: usize) -> Result<usize, LexicalError>
```

Behavior:

1. Assert/lookahead that `self.pos..self.pos+3 == b"\"\"\""`.
2. Advance three bytes.
3. Consume only `b' '` and `b'\t'`.
4. If next bytes are:
   - `\n`: consume 1, return new `self.pos`;
   - `\r\n`: consume 2, return new `self.pos`;
   - EOF: return `UnterminatedMultilineString(open..self.pos)`;
   - anything else: consume enough to produce a useful span and return `InvalidMultilineStringOpening`.

Do not let `skip_trivia` participate here. The opening structural line is part of one lexical token.

## 3.4 Boundary discovery

Add:

```rust
fn discover_multiline_boundary(
    &mut self,
    open: usize,
    body_start: usize,
) -> Result<MultilineBoundary, LexicalError>
```

The scan runs in **top-level multiline-string mode**.

At every physical source-line start reachable in that mode:

1. Look ahead through spaces/tabs.
2. If the next three bytes are `"""`:
3. Look after the quotes through optional spaces/tabs.
4. If next is LF, CRLF, or EOF, this is the closing line.
5. Save its exact pre-quote whitespace as `margin`.
6. Compute `value_end` by removing the line terminator immediately preceding `close_line_start`:
   - preceding `\r\n` → subtract 2;
   - preceding `\n` → subtract 1;
   - no preceding newline only occurs for an empty body directly after the opening structural newline; use `body_start`.

At ordinary top-level content:

- `\(`: validate/open interpolation and call `scan_interpolation_body`;
- valid non-interpolation escapes: skip the escape pair while preserving source positions;
- invalid escape: report existing `InvalidEscape` immediately;
- LF/CRLF: advance;
- raw lone CR: report multiline raw-carriage-return diagnostic;
- any UTF-8 scalar: advance with `char_len_at`.

### Critical lexical-mode rule

Do not inspect potential closing lines *inside* `scan_interpolation_body`.

That function already understands:

- nested parentheses;
- ordinary nested strings;
- comments.

After `scan_string_like` is introduced, it will also understand nested multiline strings.

Thus this is valid and must not terminate the outer text block at the inner delimiter:

```phalcom
outer = """
    value = \(call("""
        nested
        """))
    done
    """
```

## 3.5 Rewind before decode

`discover_multiline_boundary` advances `self.pos` while searching.

Before semantic decode:

```rust
self.pos = body_start;
```

After decode:

```rust
self.pos = boundary.close_end;
```

Do not expose the discovery pass’s temporary cursor state.

## 3.6 Margin validation

Add:

```rust
fn validate_multiline_margin(
    &self,
    body_start: usize,
    value_end: usize,
    margin: &str,
) -> Result<(), LexicalError>
```

Operate over the original source bytes. Do not use decoded text.

For each physical line intersecting `body_start..value_end`:

1. Determine whether the line is blank: all bytes before LF/CRLF/content-end are spaces/tabs.
2. If blank: accept.
3. If nonblank and `margin.is_empty()`: accept.
4. Otherwise require `line.starts_with(margin.as_bytes())`.
5. On mismatch, report `InvalidMultilineStringIndentation(span)`.

Recommended range:

- start at physical line start;
- end at the first nonmatching byte, or at the first non-whitespace byte when the line is simply under-indented;
- if the line starts immediately with content and required margin is nonempty, use a zero-width/one-byte diagnostic at line start according to existing repository rendering conventions.

Do not convert tabs to columns.

## 3.7 Body decoder

Add:

```rust
fn decode_multiline_body(
    &mut self,
    body_start: usize,
    value_end: usize,
    margin: &str,
) -> Result<Token, LexicalError>
```

Its output construction should mirror `scan_string`:

```rust
let mut segments = Vec::new();
let mut literal = String::new();
let mut interpolated = false;
```

But physical-line handling differs.

### At each physical line start

If blank:

- consume all spaces/tabs;
- do not append them.

If nonblank:

- consume exactly `margin.len()` bytes;
- append nothing for the margin.

Then scan body characters.

### Physical newline

LF:

```rust
literal.push('\n');
self.pos += 1;
```

CRLF:

```rust
literal.push('\n');
self.pos += 2;
```

The `value_end` boundary guarantees the structural newline before the closing delimiter is not decoded.

### Escapes

Factor escape handling if practical so ordinary and multiline strings cannot drift.

Recommended helper shape:

```rust
enum EscapeAction {
    Char(char),
    Backslash,
    Interpolation,
}

fn classify_string_escape(&self, pos: usize) -> Result<EscapeAction, LexicalError>
```

However, do not perform a broad refactor if it makes the patch harder to review. A small shared helper is preferred; exact duplicated escape match tables are acceptable only temporarily and should be covered by table-driven parity tests.

### Interpolation

When seeing `\(`:

```rust
interpolated = true;

if !literal.is_empty() {
    segments.push(StringSegment::Literal(std::mem::take(&mut literal)));
}

let interpolation_open = self.pos;
self.pos += 2;
let expr = self.scan_interpolation_body(interpolation_open)?;
segments.push(expr);
```

Keep `StringSegment::Expr.range` in the original lexer-input byte coordinate system.

Do not dedent or normalize the `Expr.source` payload. It remains exact raw source between delimiters. The parser treats margin indentation as trivia; preserving it is required for absolute diagnostic ranges.

### Quotes

In multiline mode, ordinary `"` characters are literal data.

The body decoder stops at `value_end`, so it does not need quote-based termination. Therefore:

```rust
Some(b'"') => {
    literal.push('"');
    self.pos += 1;
}
```

is valid for body quotes, including `"""` that were not the isolated closing line.

### Finalize token

Exactly mirror ordinary string behavior:

```rust
if !interpolated {
    return Ok(Token::String(literal));
}

if !literal.is_empty() {
    segments.push(StringSegment::Literal(literal));
}

Ok(Token::StringInterp(segments))
```

Empty literal segments remain omitted.

## 3.8 Preserve source ranges

This invariant is non-negotiable:

> Decoded/dedented literal text changes value bytes, but interpolation expression ranges continue to identify the original source bytes.

Example:

```phalcom
let x = """
        α \(value)
        """
```

If `value` begins at source byte N, its `StringSegment::Expr.range.start` must remain N relative to the current lexer input, not a position in the dedented value.

Add a Unicode + indentation test that asserts exact range slices.

---

# 4. Token and Lexical Error Changes

Primary file:

```text
phalcom-ast/src/token.rs
```

## 4.1 Do not add token variants

Update docs on:

```rust
Token::String
Token::StringInterp
```

to say both ordinary and multiline interpreted strings emit these variants.

Update stale links from:

```text
docs/spec/lexical-structure.md
```

to the current canonical paths where touched.

## 4.2 Add lexical errors

Add:

```rust
UnterminatedMultilineString(Range<usize>),

InvalidMultilineStringOpening(Range<usize>),

InvalidMultilineStringIndentation(Range<usize>),

InvalidMultilineStringLineEnding(Range<usize>),
```

Recommended meanings:

### `UnterminatedMultilineString`

Range: opening `"""` through EOF.

Use for:

- no closing delimiter;
- EOF immediately after opening delimiter;
- EOF after opening delimiter + only horizontal whitespace.

### `InvalidMultilineStringOpening`

Range: from first illegal byte after opening delimiter/optional HSPACE through at least that scalar.

Use for:

```phalcom
"""same-line content
```

### `InvalidMultilineStringIndentation`

Range: offending line indentation/prefix.

### `InvalidMultilineStringLineEnding`

Range: raw lone CR.

Do not overload `RawNewlineInString`; that diagnostic exists specifically because physical newlines are illegal in ordinary strings.

---

# 5. User-Facing Diagnostics

Primary file:

```text
phalcom-ast/src/error.rs
```

Add `SyntaxErrorKind` variants corresponding 1:1 to the new lexical errors.

Recommended stable codes:

| Kind | Code | Message |
|---|---|---|
| `UnterminatedMultilineString` | `string.multiline.unterminated` | `Unterminated multiline string` |
| `InvalidMultilineStringOpening` | `string.multiline.opening_newline` | `Multiline string opening delimiter must be followed by a newline` |
| `InvalidMultilineStringIndentation` | `string.multiline.indentation` | `Multiline string line is indented less than the closing delimiter` |
| `InvalidMultilineStringLineEnding` | `string.multiline.invalid_line_ending` | `Invalid multiline string line ending` |

Update every exhaustive match, especially:

```rust
SyntaxErrorKind::code()
```

and any `Display`/message match in `error.rs`.

### Diagnostic design constraints

- Stable codes matter more than exact capitalization.
- Do not widen indentation errors to the whole string.
- Do not report a parser error when the lexer has enough information to report a structural text-block error.
- Keep ordinary-string codes unchanged.
- Unknown escapes in multiline blocks continue to be `string.invalid_escape`, not a new code.

---

# 6. Parser Error Lowering and REPL Completeness

Primary file:

```text
phalcom-ast/src/parser.rs
```

Baseline anchors are near the top of the file:

```rust
fn lex_error_to_syntax(...)
fn push_lex_error(...)
```

## 6.1 Extend `lex_error_to_syntax`

Add arms:

```rust
LexicalError::UnterminatedMultilineString(span) => SyntaxError {
    kind: SyntaxErrorKind::UnterminatedMultilineString,
    range: (span.start + offset)..(span.end + offset),
},

LexicalError::InvalidMultilineStringOpening(span) => SyntaxError {
    kind: SyntaxErrorKind::InvalidMultilineStringOpening,
    range: (span.start + offset)..(span.end + offset),
},

LexicalError::InvalidMultilineStringIndentation(span) => SyntaxError {
    kind: SyntaxErrorKind::InvalidMultilineStringIndentation,
    range: (span.start + offset)..(span.end + offset),
},

LexicalError::InvalidMultilineStringLineEnding(span) => SyntaxError {
    kind: SyntaxErrorKind::InvalidMultilineStringLineEnding,
    range: (span.start + offset)..(span.end + offset),
},
```

## 6.2 Extend parser-driven continuation

Current `push_lex_error` deliberately co-emits `UnrecognizedEof` for open lexical modes. Add:

```rust
LexicalError::UnterminatedMultilineString(_) => Some("`\"\"\"`"),
```

to:

```rust
let expected_closer = match &err { ... };
```

This is required by PDR-0006’s existing architecture.

Do not add multiline logic to `phalcom-repl`’s validator. The repository explicitly centralized this decision in parser diagnostics.

## 6.3 Parser/AST lowering remains untouched

The parser already accepts:

```rust
Token::String(...)
Token::StringInterp(...)
```

and lowers interpolation to existing String concatenation / `toString` access.

Do not alter that code unless a failing test proves an existing bug independent of lexical representation.

Add tests proving multiline strings travel through the same path.

---

# 7. Interpolation Scanner Integration

Primary file:

```text
phalcom-ast/src/lexer.rs
```

The current `scan_interpolation_body` has:

```rust
Some(b'"') => {
    let _ = self.scan_string()?;
}
```

Change it to the centralized string dispatcher.

This enables all of:

```phalcom
"\(call("""
    nested
    """))"
```

and:

```phalcom
"""
    \(call("ordinary"))
    """
```

and nested interpolation inside nested multiline strings.

### Do not fork interpolation balancing logic

The existing scanner already establishes the important invariant:

- only parentheses in expression-code mode affect interpolation depth;
- strings/comments are lexically opaque to outer depth.

Multiline support should join this mechanism, not duplicate it.

---

# 8. LSP Semantic Tokens: Required Multiline Fix

Primary file:

```text
phalcom-lsp/src/semantic_tokens.rs
```

This is a required production edit.

## 8.1 Existing issue

Current `encode` computes:

```rust
let start_pos = line_index.position(token.start);
let length: u32 = text[token.start..token.end]
    .chars()
    .map(|c| c.len_utf16() as u32)
    .sum();
```

That is valid only when `token.start..token.end` lies on one physical line.

A plain multiline `Token::String` will span several lines. LSP semantic tokens must be line-local; encoding one token whose `length` includes newline and following-line text is invalid/incorrect.

## 8.2 Add generic line splitting

Do not special-case only strings in `encode`. Make the semantic-token transport layer robust for any future multiline lexical range.

Add a helper:

```rust
fn split_raw_token_by_lines(
    text: &str,
    token: RawToken,
    out: &mut Vec<RawToken>,
)
```

or:

```rust
fn line_localize(text: &str, raw: &[RawToken]) -> Vec<RawToken>
```

Algorithm for each `RawToken`:

1. Walk `text[token.start..token.end]` by source byte.
2. Emit one non-empty `RawToken` per physical line fragment.
3. Exclude LF from token ranges.
4. For CRLF, exclude both CR and LF from colored content.
5. Preserve the original `kind`.
6. Skip zero-length fragments on empty lines.
7. Keep ranges source-ordered.

Then `encode` works only with line-local tokens.

Recommended call shape:

```rust
let localized = line_localize(text, raw);
encode_line_local(text, line_index, &localized)
```

or localize internally before delta encoding.

## 8.3 Fix existing interpolated-string range offset bug while here

Current `push_string_interp` computes:

```rust
let expr_open = token_start + range.start;
```

But `StringSegment::Expr.range` is documented as a byte range **within the current lexer input**, not relative to the containing string token.

For top-level text with a string beginning after byte 0, adding `token_start` again double-offsets interpolation expressions.

Make the coordinate spaces explicit.

Change the function signature from conceptually:

```rust
push_string_interp(
    segments,
    token_start,
    token_end,
    out,
)
```

to:

```rust
push_string_interp(
    segments,
    input_offset,
    token_start,
    token_end,
    out,
)
```

Call from `collect_tokens`:

```rust
Token::StringInterp(segments) => {
    push_string_interp(
        &segments,
        offset,
        offset + start,
        offset + end,
        out,
    );
}
```

Then:

```rust
let expr_open = input_offset + range.start;
```

not:

```rust
token_start + range.start
```

The delimiter gap remains:

```rust
let backslash_paren = expr_open.saturating_sub(2);
```

and after the expression:

```rust
cursor = input_offset + range.end + 1;
```

This change is tightly related to multiline interpolation because exact cross-line coloring depends on correct expression ranges. Pin it with tests so it cannot regress.

## 8.4 LSP tests

In the existing `#[cfg(test)] mod tests` in `semantic_tokens.rs`, add:

### Plain multiline string

Input:

```text
let s = """
    α
    beta
    """
```

Assert:

- no semantic token spans physical lines;
- each emitted string fragment’s length is UTF-16 units for that line only;
- Unicode line length is correct.

### Multiline interpolation

Input with prefix bytes before string:

```phalcom
let s = """
    value \(x + 1)
    """
```

Assert semantic kinds/ranges:

- string fragments around expression;
- `x` is variable;
- `+` operator;
- `1` number;
- all absolute source positions are correct.

### Nested string after nonzero offset

Add a direct regression for the pre-existing offset bug:

```phalcom
let prefix = 0
let s = "a \(x) b"
```

Assert the `x` token position points at the actual source `x`, not `token_start + range.start`.

### CRLF

Use an in-memory Rust test string containing `\r\n`.

Assert no emitted semantic token includes `\r` or `\n` in its length.

---

# 9. REPL Surface

Primary files:

```text
phalcom-ast/tests/probe_continuation.rs
phalcom-repl/src/highlighter.rs
```

## 9.1 Continuation tests

Extend `probe_continuation.rs`.

Add incomplete cases:

```rust
("let s = \"\"\"", Verdict::Incomplete),
("let s = \"\"\"   ", Verdict::Incomplete),
("let s = \"\"\"\n    hello", Verdict::Incomplete),
("let s = \"\"\"\n    hello\n", Verdict::Incomplete),
```

Add complete case:

```rust
(
    "let s = \"\"\"\n    hello\n    \"\"\"",
    Verdict::Complete,
),
```

Add fatal cases:

```rust
(
    "let s = \"\"\"same line\n    \"\"\"",
    Verdict::Error,
),
```

and an indentation failure with a present closing delimiter.

The test should also inspect `UnrecognizedEof.expected` and assert it contains:

```text
"""
```

for unterminated multiline strings.

## 9.2 Highlighter

`phalcom-repl/src/highlighter.rs` already colors:

```rust
Token::String(_) | Token::StringInterp(_)
```

as string.

Because public token variants are reused, no semantic classification change is expected.

Before changing production code, add/execute a focused test or manual REPL validation proving Reedline’s `StyledText::style_range` accepts the full multiline buffer range. If it does, leave `highlighter.rs` unchanged except stale comments if necessary.

If Reedline requires per-line styling, apply the same line-fragment principle as the LSP, but do not preemptively fork string classification.

---

# 10. Lexer Test Matrix

Primary file:

```text
phalcom-ast/tests/lexer.rs
```

Add deterministic tests rather than a new test framework.

## 10.1 Basic value tests

### Basic dedent

```rust
let src = "\"\"\"\n    first\n        second\n    third\n    \"\"\"";
```

Expected:

```rust
Token::String("first\n    second\nthird".into())
```

### Empty block

```text
"""
"""
```

Expected value:

```text
""
```

### Closing margin zero

```text
"""
first
  second
"""
```

Expected:

```text
first
  second
```

### Opening trailing HSPACE

```text
"""
    hello
    """
```

Expected `hello`.

## 10.2 Blank-line tests

Cover:

- blank line with zero spaces;
- blank line shorter than margin;
- blank line with more whitespace than margin;
- consecutive blank lines;
- trailing semantic blank line before closing line.

Pin canonical blank-line whitespace removal.

## 10.3 Exact-prefix indentation

Cover:

- four-space margin + four-space content: pass;
- four-space margin + two-space nonblank line: fail;
- tab margin + tab content: pass;
- tab margin + spaces content: fail;
- mixed `\t  ` margin + exact mixed prefix: pass;
- same visual indentation with a different source prefix: fail.

## 10.4 Newlines

Cover in-memory source strings for:

- all LF;
- all CRLF;
- mixed LF/CRLF;
- raw lone CR failure.

Expected value always uses `\n` for physical source line breaks.

## 10.5 Escapes

Table-drive all ordinary escape forms inside multiline strings:

```text
\"
\\
\n
\t
\r
```

and verify unknown escape uses the existing invalid-escape error.

Also assert escaped `\n` and physical newline remain semantically distinguishable during indentation handling.

## 10.6 Quotes

Cover:

```text
"a quote"
"""inside a content line"""
"""prefix at line start but not isolated
```

Only the isolated delimiter line closes.

## 10.7 Interpolation

Cover:

- interpolation-only semantic body;
- literal prefix/suffix;
- adjacent interpolations;
- interpolation at first non-margin byte;
- interpolation after additional indentation;
- multiline interpolation expression;
- nested ordinary string;
- nested multiline string;
- comments/parentheses inside interpolation;
- nested interpolation inside nested multiline string.

## 10.8 Source ranges

Use a source with:

- nonzero prefix before text block;
- UTF-8 (`α`, emoji);
- indentation;
- interpolation.

Assert:

```rust
&src[range.clone()] == expected_expression_source
```

and exact absolute byte indices.

## 10.9 Diagnostics

Assert lexical variant/range for:

- invalid opening same-line content;
- unterminated opener at EOF;
- unterminated body;
- under-indented line;
- invalid escape;
- raw lone CR.

---

# 11. Parser and AST Tests

Primary file:

```text
phalcom-ast/tests/parser.rs
```

Add tests proving lexical novelty does not produce AST novelty.

## 11.1 Plain multiline literal

Parse:

```phalcom
return """
    first
    second
    """
```

Assert the resulting AST expression is the existing String literal node and contains:

```text
first\nsecond
```

No new AST variant.

## 11.2 Multiline interpolation

Parse:

```phalcom
return """
    hello \(value)
    """
```

Assert the same shape used by ordinary interpolation:

- String literal segment;
- `GetProperty(..., "toString")`;
- left-associative `+` chain as applicable.

Use direct structural assertions for the critical invariant; snapshot additionally if consistent with existing file style.

## 11.3 Interpolation-only block

Parse a block whose semantic content is only interpolation:

```phalcom
return """
    \(value)
    """
```

Confirm existing empty-String accumulator behavior remains intact.

## 11.4 Absolute inner diagnostic range

Use malformed interpolation inside an indented multiline string and verify the parser diagnostic underlines the inner expression source, not the whole outer block and not a dedented coordinate.

---

# 12. End-to-End Runtime Corpus

Primary harness:

```text
phalcom-core/tests/lang.rs
phalcom-core/tests/lang/MANIFEST.md
phalcom-core/tests/lang/string/
phalcom-core/tests/lang/syntax-errors/
```

The corpus model is `.ph` + sibling `.expected`.

## 12.1 Add positive string cases

Create:

```text
phalcom-core/tests/lang/string/string_multiline_basic.ph
phalcom-core/tests/lang/string/string_multiline_basic.expected

phalcom-core/tests/lang/string/string_multiline_interpolation.ph
phalcom-core/tests/lang/string/string_multiline_interpolation.expected

phalcom-core/tests/lang/string/string_multiline_quotes.ph
phalcom-core/tests/lang/string/string_multiline_quotes.expected
```

Example `string_multiline_basic.ph`:

```phalcom
System.print("""
    first
        second
    third
    """)
```

Expected:

```text
first
    second
third
```

Example interpolation:

```phalcom
let name = "Phalcom"
System.print("""
    hello \(name)
    """)
```

Expected:

```text
hello Phalcom
```

## 12.2 Add negative cases

Create under the lane already used for syntax diagnostics:

```text
phalcom-core/tests/lang/syntax-errors/string_multiline_bad_opening.ph
phalcom-core/tests/lang/syntax-errors/string_multiline_bad_opening.expected

phalcom-core/tests/lang/syntax-errors/string_multiline_bad_indent.ph
phalcom-core/tests/lang/syntax-errors/string_multiline_bad_indent.expected

phalcom-core/tests/lang/syntax-errors/string_multiline_unterminated.ph
phalcom-core/tests/lang/syntax-errors/string_multiline_unterminated.expected
```

Expected files should pin stable diagnostic code/message substrings according to the harness convention, not fragile full renderer output unless that lane already requires it.

## 12.3 Update manifest

Update:

```text
phalcom-core/tests/lang/MANIFEST.md
```

with the new PASS/NEGATIVE cases and feature note.

Do not invent a new language-corpus label solely for multiline strings.

---

# 13. Specification and PDR Governance

The repository explicitly says never implement against an unratified design record. The user decision on 2026-08-15 settles the semantics, so record it before production code.

## 13.1 Create PDR-0034

Create:

```text
docs/pdr/0034-multiline-string-text-blocks.md
```

Header:

```markdown
# PDR-0034 — Add indentation-safe multiline string text blocks

- Status: Accepted
- Date: 2026-08-15
- Amends: PDR-0029 ruling 2 and its multiline-literal deferral; preserves all interpolation rulings
- Related: ../spec/current/syntax/lexical.md, ../spec/current/string-interpolation.md, ../work/analyses/multiline-string-literals.md
```

Decision must pin:

1. `"""` delimiter.
2. Structural opening newline.
3. Isolated closing line.
4. Closing indentation defines exact-prefix margin.
5. Nonblank lines must start with margin.
6. Blank-line whitespace canonicalizes to empty.
7. Margin stripped; additional indentation preserved.
8. LF/CRLF normalize to `\n`.
9. Escapes/interpolation same as ordinary interpreted strings.
10. Ordinary strings remain single-line.
11. Triple quotes not satisfying closing-line shape are literal content.
12. No multiline quoted-symbol form.
13. Reuse existing `String` token/AST/runtime representation.
14. Formatter reindent safety is a design requirement.
15. Stable diagnostic codes.
16. REPL incomplete-mode obligation.
17. LSP multiline-token obligation.

Alternatives rejected should include:

- multiline ordinary `"..."`;
- Python-style triple strings with indentation preserved;
- minimum-common-indent trimming;
- visual-column/tab expansion;
- raw triple strings in this unit;
- new AST/runtime string kind;
- heredoc syntax;
- treating any `"""` sequence as terminator regardless of line position.

## 13.2 Update PDR tracker

Edit:

```text
docs/pdr/STATUS.md
```

Add row 0034.

Before implementation:

```text
Accepted ... | ❌ ruled 2026-08-15, unimplemented
```

After verified implementation, update the same row to `✅` with actual commit/test evidence.

Do not mark shipped before verification.

## 13.3 Amend PDR-0029 carefully

Edit:

```text
docs/pdr/0029-string-literals-and-interpolation-completion.md
```

Do not retire or supersede the whole record. Its interpolation decisions remain authoritative.

Add a dated amendment note explaining that its multiline deferral was resolved by PDR-0034.

Change wording such as:

```text
Dedicated multiline string literal syntax is deferred...
```

to historical language:

```text
At the time of PDR-0029 this was deferred; PDR-0034 later admits `"""` text blocks.
```

Preserve PDR-0029’s ordinary-string single-line rule.

## 13.4 Update canonical lexical spec

Edit:

```text
docs/spec/current/syntax/lexical.md
```

Replace §7’s “multiline deferred” text with distinct productions:

```text
STRING := "\"" { STRING-SEGMENT } "\""

MULTILINE-STRING :=
    "\"\"\"" HSPACE* NEWLINE
    MULTILINE-CONTENT
    MULTILINE-CLOSE
```

Document the exact-prefix margin algorithm normatively.

Explicitly state ordinary `"..."` still rejects raw physical newline.

## 13.5 Update interpolation spec

Edit:

```text
docs/spec/current/string-interpolation.md
```

Update lexical scope so interpreted string semantics apply to:

- ordinary strings;
- multiline text blocks.

Preserve the exact-one-expression, range, lowering, and runtime sections.

Add multiline examples and state interpolation `Expr.source` remains original source bytes, not dedented source.

## 13.6 Resolve deferred work marker

Edit:

```text
docs/work/deferred/multiline-string-literals.md
```

Preferred minimal-history approach: retain the file path because PDR-0029 links to it, but replace/add a top-level resolution notice:

```markdown
> Resolved 2026-08-15 by PDR-0034. This file records the former deferral.
```

Do not leave it reading as an active prohibition.

## 13.7 Mark analysis as decided

Edit:

```text
docs/work/analyses/multiline-string-literals.md
```

Add a short header linking PDR-0034 and saying the recommendation was accepted.

Do not duplicate the normative specification there; canonical rules belong in spec/PDR.

---

# 14. Formatter Implications

No formatter implementation was identified in the current production workspace during inspection. Therefore this feature must not invent one.

Still, the lexical/spec design must guarantee a future formatter can safely reindent a whole text block.

Given:

```phalcom
message = """
    first
    second
    """
```

moving it four spaces right:

```phalcom
if condition {
    message = """
        first
        second
        """
}
```

must preserve the value.

This follows automatically because the formatter would shift:

- every content line’s margin prefix;
- the closing delimiter margin;

by the same source prefix.

Do not specify a formatter that independently normalizes “minimum indentation.” The closing delimiter is authoritative.

---

# 15. Implementation Tasks

## Task 1 — Ratify the design in repository governance

**Files**
- Create: `docs/pdr/0034-multiline-string-text-blocks.md`
- Modify: `docs/pdr/STATUS.md`
- Modify: `docs/pdr/0029-string-literals-and-interpolation-completion.md`
- Modify: `docs/work/deferred/multiline-string-literals.md`
- Modify: `docs/work/analyses/multiline-string-literals.md`

**Consumes:** approved semantics in existing analysis.

**Produces:** accepted, implementation-authorizing PDR.

- [ ] Write PDR-0034 with all rules in §1 above.
- [ ] Add tracker row as Accepted / unshipped.
- [ ] Add dated amendment to PDR-0029 without superseding its interpolation rulings.
- [ ] Mark deferred work item resolved.
- [ ] Mark analysis as decided, linking PDR-0034.
- [ ] Run repository markdown/link checks if present.
- [ ] Commit:

```bash
git add docs/pdr docs/work
git commit -m "docs: ratify multiline string text blocks"
```

## Task 2 — Write failing lexer and diagnostic tests first

**Files**
- Modify: `phalcom-ast/tests/lexer.rs`
- Modify: `phalcom-ast/tests/probe_continuation.rs`
- Modify: `phalcom-ast/tests/parser.rs`

**Consumes:** existing lexer/test harness.

**Produces:** executable contract before scanner changes.

- [ ] Add basic dedent test.
- [ ] Add exact-prefix mixed-tab tests.
- [ ] Add blank-line tests.
- [ ] Add LF/CRLF normalization tests.
- [ ] Add opening-rule failure.
- [ ] Add indentation failure.
- [ ] Add unterminated text-block tests.
- [ ] Add ordinary escape parity tests.
- [ ] Add interpolation/range tests.
- [ ] Add nested multiline-in-interpolation test.
- [ ] Add REPL incomplete/complete/fatal classification cases.
- [ ] Add parser AST-shape tests.
- [ ] Run:

```bash
cargo test -p phalcom-ast
```

Expected: new tests fail because `"""` has no text-block semantics yet.

- [ ] Commit tests separately if project practice allows red commits only on feature branches; otherwise keep staged until Task 5.

## Task 3 — Add lexical/syntax error vocabulary

**Files**
- Modify: `phalcom-ast/src/token.rs`
- Modify: `phalcom-ast/src/error.rs`
- Modify: `phalcom-ast/src/parser.rs`

**Consumes:** Task 2 diagnostic expectations.

**Produces:** stable structured errors.

- [ ] Add four `LexicalError` variants.
- [ ] Add matching `SyntaxErrorKind` variants.
- [ ] Add stable codes/messages.
- [ ] Extend `lex_error_to_syntax`.
- [ ] Extend `push_lex_error` for `UnterminatedMultilineString` → expected `"""`.
- [ ] Compile:

```bash
cargo check -p phalcom-ast
```

- [ ] Run targeted continuation tests:

```bash
cargo test -p phalcom-ast --test probe_continuation
```

Expected: compile succeeds; feature tests still fail until scanner lands.

## Task 4 — Centralize string-mode dispatch

**Files**
- Modify: `phalcom-ast/src/lexer.rs`

**Consumes:** existing `scan_string`, `scan_interpolation_body`.

**Produces:** one quote-mode entry point supporting nesting.

- [ ] Add `scan_string_like`.
- [ ] Route `scan_token` quote branch through it.
- [ ] Route nested quote branch in `scan_interpolation_body` through it.
- [ ] Leave `scan_quoted_symbol` separate.
- [ ] Update stale comments/spec links touched in this region.
- [ ] Run existing ordinary-string/interpolation tests to prove no regression:

```bash
cargo test -p phalcom-ast --test lexer string
cargo test -p phalcom-ast --test parser interpolated_string_parses
```

## Task 5 — Implement two-phase multiline scanner

**Files**
- Modify: `phalcom-ast/src/lexer.rs`

**Consumes:** Task 3 errors, Task 4 dispatcher.

**Produces:** `Token::String` / `Token::StringInterp` from `"""`.

- [ ] Add private `MultilineBoundary`.
- [ ] Add `scan_multiline_opening`.
- [ ] Add boundary-discovery pass.
- [ ] Make discovery skip interpolation bodies with the real interpolation scanner.
- [ ] Recognize only isolated top-level closing lines.
- [ ] Compute exact margin.
- [ ] Compute `value_end` excluding the structural pre-close newline.
- [ ] Add exact-prefix margin validator.
- [ ] Canonicalize blank-line source whitespace.
- [ ] Add body decoder with source-line margin stripping.
- [ ] Normalize LF/CRLF physical newlines to `\n`.
- [ ] Reuse ordinary escape semantics.
- [ ] Preserve raw interpolation source/ranges.
- [ ] Make non-closing quote sequences literal.
- [ ] Set `self.pos` to immediately after closing quotes, leaving trailing HSPACE/newline to ordinary lexer handling.
- [ ] Run:

```bash
cargo test -p phalcom-ast --test lexer
cargo test -p phalcom-ast --test probe_continuation
cargo test -p phalcom-ast --test parser
```

All new front-end tests should now pass.

- [ ] Run full crate:

```bash
cargo test -p phalcom-ast
```

- [ ] Commit:

```bash
git add phalcom-ast
git commit -m "feat(ast): add multiline string text blocks"
```

## Task 6 — Add end-to-end language corpus

**Files**
- Create positive fixtures under `phalcom-core/tests/lang/string/`
- Create negative fixtures under `phalcom-core/tests/lang/syntax-errors/`
- Modify: `phalcom-core/tests/lang/MANIFEST.md`

**Consumes:** completed lexer/parser behavior.

**Produces:** runtime conformance.

- [ ] Add basic runtime value fixture.
- [ ] Add interpolation runtime fixture.
- [ ] Add embedded triple-quotes fixture.
- [ ] Add invalid opening fixture.
- [ ] Add bad-indent fixture.
- [ ] Add unterminated fixture.
- [ ] Update manifest.
- [ ] Run relevant lanes:

```bash
cargo test -p phalcom-core --test lang string
cargo test -p phalcom-core --test lang syntax_errors
```

- [ ] Run whole language corpus:

```bash
cargo test -p phalcom-core --test lang
```

- [ ] Commit:

```bash
git add phalcom-core/tests
git commit -m "test: cover multiline string semantics"
```

## Task 7 — Fix LSP multiline semantic-token transport and interpolation offsets

**Files**
- Modify: `phalcom-lsp/src/semantic_tokens.rs`

**Consumes:** multiline lexer tokens + `LineIndex`.

**Produces:** valid line-local LSP semantic tokens.

- [ ] First add failing test demonstrating one multiline `RawToken` currently becomes one cross-line semantic token.
- [ ] Add failing regression proving nonzero-offset interpolation range is double-shifted by current `token_start + range.start`.
- [ ] Change `push_string_interp` to receive current lexer-input `offset`.
- [ ] Compute absolute expression ranges as `input_offset + range.start/end`.
- [ ] Add generic raw-token line localization.
- [ ] Exclude LF and CRLF terminators from emitted token fragments.
- [ ] Keep UTF-16 length computation per localized line fragment.
- [ ] Add Unicode multiline test.
- [ ] Add multiline interpolation test.
- [ ] Add CRLF semantic-token test.
- [ ] Run:

```bash
cargo test -p phalcom-lsp semantic_tokens
cargo test -p phalcom-lsp line_index
cargo test -p phalcom-lsp
```

- [ ] Commit:

```bash
git add phalcom-lsp/src/semantic_tokens.rs
git commit -m "fix(lsp): encode multiline string semantic tokens"
```

## Task 8 — Update canonical specifications

**Files**
- Modify: `docs/spec/current/syntax/lexical.md`
- Modify: `docs/spec/current/string-interpolation.md`

**Consumes:** accepted PDR + tested implementation.

**Produces:** canonical as-built spec.

- [ ] Add `MULTILINE-STRING` grammar.
- [ ] Specify exact-prefix margin and blank-line behavior.
- [ ] Specify physical newline normalization.
- [ ] Specify quote/delimiter recognition.
- [ ] Specify errors/codes.
- [ ] Update interpolation spec to cover both interpreted forms.
- [ ] State raw `Expr.source`/range preservation.
- [ ] Remove “multiline deferred” language from active spec.
- [ ] Keep ordinary single-line string newline prohibition.
- [ ] Commit:

```bash
git add docs/spec
git commit -m "docs: specify multiline string text blocks"
```

## Task 9 — REPL/highlighter validation

**Files**
- Verify: `phalcom-repl/src/highlighter.rs`
- Modify only if tests demonstrate necessity.

**Consumes:** lexer and parser signals.

**Produces:** interactive multiline entry with correct continuation/coloring.

- [ ] Run REPL continuation tests from `phalcom-ast`.
- [ ] Run `phalcom-repl` tests:

```bash
cargo test -p phalcom-repl
```

- [ ] Manually exercise, if project workflow includes interactive checks:

```text
>>> let message = """
...     hello
...     world
...     """
```

Expected: REPL remains in continuation mode until closing delimiter appears.

- [ ] Verify string highlighter does not panic on a multiline buffer.
- [ ] If no highlighter code change is needed, record that explicitly in implementation log/PR.
- [ ] If line splitting is required by Reedline, add a focused helper/test rather than changing token classification.

## Task 10 — Full verification and shipped-state update

**Files**
- Modify: `docs/pdr/STATUS.md` only after all gates pass.
- Optionally add implementation log following repository convention.

- [ ] Format:

```bash
cargo fmt --all --check
```

- [ ] Static check:

```bash
cargo check --workspace
```

- [ ] Clippy if workspace CI uses it:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

- [ ] Full tests:

```bash
cargo test --workspace
```

- [ ] If repository has additional generated/graph checks, run them according to `AGENTS.md`/CI.
- [ ] Verify no ordinary string behavior changed.
- [ ] Verify `#"...“` behavior unchanged.
- [ ] Verify LSP tests produce no cross-line semantic token.
- [ ] Verify REPL incomplete text block is classified incomplete, not fatal.
- [ ] Update PDR-0034 tracker row to `✅ shipped` with real commit/test evidence.
- [ ] Commit final evidence update:

```bash
git add docs/pdr/STATUS.md
git commit -m "docs: mark multiline text blocks shipped"
```

---

# 16. Proposed Core Code Shape

This is intentionally a design-level skeleton, not a blind copy-paste patch. Names should remain close to this shape so responsibilities stay auditable.

```rust
fn scan_string_like(&mut self) -> Result<Token, LexicalError> {
    if self.peek_at(0) == Some(b'"')
        && self.peek_at(1) == Some(b'"')
        && self.peek_at(2) == Some(b'"')
    {
        self.scan_multiline_string()
    } else {
        self.scan_string()
    }
}

fn scan_multiline_string(&mut self) -> Result<Token, LexicalError> {
    let open = self.pos;
    let body_start = self.scan_multiline_opening(open)?;

    self.pos = body_start;
    let boundary = self.discover_multiline_boundary(open, body_start)?;

    self.validate_multiline_margin(
        body_start,
        boundary.value_end,
        &boundary.margin,
    )?;

    self.pos = body_start;
    let token = self.decode_multiline_body(
        body_start,
        boundary.value_end,
        &boundary.margin,
    )?;

    self.pos = boundary.close_end;
    Ok(token)
}
```

Keep boundary discovery and semantic decode separate.

A helper for closing-line recognition should return `None` unless all conditions hold:

```rust
fn multiline_close_at_line_start(
    &self,
    line_start: usize,
) -> Option<(usize, usize, String)> {
    // 1. scan HSPACE => quote_start
    // 2. require """ at quote_start
    // 3. scan trailing HSPACE
    // 4. require LF / CRLF / EOF
    // 5. return (quote_start, quote_start + 3, exact margin)
}
```

Do not consume the post-closing newline in this helper.

---

# 17. Acceptance Criteria

The feature is complete only if every statement below is true.

## Syntax

- [ ] Ordinary `"..."` strings still reject a physical LF/CRLF.
- [ ] `"""` opens multiline mode only when followed by HSPACE + newline.
- [ ] Closing `"""` is isolated on its line apart from indentation/trailing HSPACE.
- [ ] Embedded non-closing `"""` is literal content.
- [ ] `#"""` does not silently become a multiline symbol syntax.

## Value semantics

- [ ] Opening structural newline is absent from value.
- [ ] Pre-closing structural newline is absent from value.
- [ ] Closing margin is stripped from all nonblank physical content lines.
- [ ] Additional indentation survives.
- [ ] Blank-line source indentation does not survive.
- [ ] LF/CRLF become `\n`.
- [ ] Escape table is exactly ordinary interpreted-string escape table.

## Interpolation

- [ ] `\(expr)` works unchanged.
- [ ] Expression evaluates once.
- [ ] `toString` behavior unchanged.
- [ ] Nested multiline strings inside interpolation work.
- [ ] Parentheses in nested multiline content do not change outer interpolation depth.
- [ ] Inner parser diagnostics use original absolute source ranges.

## Diagnostics

- [ ] Unterminated text block has stable multiline-specific code.
- [ ] Bad opening line has stable code.
- [ ] Bad margin has stable code and narrow span.
- [ ] Raw lone CR has stable code.
- [ ] Unknown escapes remain `string.invalid_escape`.
- [ ] Ordinary raw newline remains `string.raw_newline`.

## REPL

- [ ] Unterminated `"""` co-emits `UnrecognizedEof`.
- [ ] Expected closer names `"""`.
- [ ] Bad opening/indentation is fatal, not “keep reading.”
- [ ] No duplicate grammar model is introduced in REPL validator.

## LSP

- [ ] No semantic token spans a physical line boundary.
- [ ] UTF-16 lengths are computed per line fragment.
- [ ] CRLF terminators are not included in semantic-token length.
- [ ] Interpolation expression positions are correct when containing string starts after byte zero.
- [ ] Multiline interpolation is colored as string gaps + recursively typed expression tokens.

## Architecture

- [ ] No public multiline-string token variant.
- [ ] No new AST variant.
- [ ] No runtime multiline-string value.
- [ ] No new VM/compiler interpolation path.
- [ ] Existing ordinary string/interpolation tests remain green.

---

# 18. Risks and Failure Modes

## Risk A — Dedenting decoded text

**Failure:** escaped `\n` becomes indistinguishable from physical newline and gets margin processing.

**Prevention:** discover margin from raw source; strip margin while decoding physical source, never afterward over decoded value.

## Risk B — Closing delimiter inside interpolation

**Failure:** outer string closes on a nested text block’s delimiter.

**Prevention:** discovery pass skips `\(...)` through the real interpolation scanner; nested strings use `scan_string_like`.

## Risk C — Source range drift

**Failure:** dedentation shifts interpolation diagnostics/LSP positions.

**Prevention:** `StringSegment::Expr.range` remains raw source coordinates; never convert it to dedented-value offsets.

## Risk D — Tabs interpreted visually

**Failure:** formatter/editor tab width changes language semantics.

**Prevention:** margin is exact source prefix bytes, not columns.

## Risk E — REPL treats open text block as fatal

**Failure:** entering a multiline literal interactively becomes impossible.

**Prevention:** `UnterminatedMultilineString` participates in `push_lex_error` expected-closer path.

## Risk F — LSP sends illegal cross-line semantic token

**Failure:** clients mis-highlight or reject semantic token payload.

**Prevention:** generic line localization before encoding.

## Risk G — Duplicate interpolation implementation

**Failure:** ordinary and multiline strings diverge in balancing, comments, nested strings, or diagnostics.

**Prevention:** reuse `scan_interpolation_body` and centralized quote-mode dispatcher.

## Risk H — `"""` creates unexpected symbol syntax

**Failure:** `#"""` accidentally gets interpreted through quoted-symbol scanner.

**Prevention:** explicit negative lexer test; keep symbol syntax out of PDR-0034.

---

# 19. Review Checklist Before Merge

### Design consistency

- [ ] PDR-0034, lexical spec, interpolation spec, code, and tests agree on all 17 acceptance categories.
- [ ] PDR-0029 historical deferral is amended but its interpolation semantics remain intact.
- [ ] No “multiline deferred” claim remains in active/current spec text.

### Code quality

- [ ] No approximate nested lexical scanner.
- [ ] No unchecked UTF-8 byte stepping for arbitrary characters.
- [ ] Fixed ASCII delimiters advance by known byte counts only.
- [ ] All source ranges remain half-open UTF-8 byte ranges.
- [ ] New exhaustive enum matches compile without wildcard hiding.

### Tests

- [ ] LF and CRLF.
- [ ] Unicode.
- [ ] tabs + spaces.
- [ ] blank lines.
- [ ] quotes/triple quotes.
- [ ] escapes.
- [ ] interpolation.
- [ ] nested multiline string.
- [ ] malformed indentation.
- [ ] invalid opener.
- [ ] EOF continuation.
- [ ] LSP line splitting.
- [ ] LSP interpolation absolute offsets.
- [ ] end-to-end runtime values.

### Non-goals verified

- [ ] No raw string.
- [ ] No heredoc.
- [ ] No implicit concatenation.
- [ ] No multiline symbol.
- [ ] No formatter project.
- [ ] No runtime type addition.

---

# 20. Recommended Commit Sequence

Keep review boundaries narrow:

```text
1. docs: ratify multiline string text blocks
2. test(ast): pin multiline string lexical contract
3. feat(ast): add multiline string text blocks
4. test: cover multiline string runtime semantics
5. fix(lsp): encode multiline string semantic tokens
6. docs: specify multiline string text blocks
7. docs: mark multiline text blocks shipped
```

If the project requires tests and implementation in the same green commit, fold commits 2 and 3 together while still writing tests first locally.

---

# 21. Final Implementation Principle

Treat `"""` as a **source-normalization form for ordinary `String`**, not a new semantic entity.

The lexer owns:

- delimiter recognition;
- structural newlines;
- exact-prefix margin validation;
- margin removal;
- physical newline normalization;
- escapes;
- interpolation segmentation.

The parser owns exactly what it owns today.

The runtime owns exactly what it owns today.

The LSP receives ordinary lexer tokens but must become robust to multiline source ranges.

That boundary keeps Phalcom’s language model small while giving multiline text unusually strong formatting stability and diagnostics.
