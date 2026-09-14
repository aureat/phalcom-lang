# PDR-0034 — Add indentation-safe multiline string text blocks

- Status: Accepted
- Date: 2026-08-15
- Amends: PDR-0029 ruling 2 and its multiline-literal deferral; preserves all interpolation rulings
- Related: ../spec/current/syntax/lexical.md, ../spec/current/string-interpolation.md, ../work/analyses/multiline-string-literals.md

---

## Context

Phalcom strings previously supported only single-line interpreted literals delimited by `"..."` with interpolation `\(expr)`. Physical newlines inside `"..."` were rejected with `string.raw_newline`.

Multiline text blocks were deferred in PDR-0029 and tracked in `docs/work/deferred/multiline-string-literals.md` and analyzed in `docs/work/analyses/multiline-string-literals.md`.

## Decision

We adopt explicit interpreted multiline text blocks delimited by `"""` with closing-delimiter-defined indentation margin, normalized physical line endings, standard Phalcom escapes, and standard `\(expr)` interpolation.

### 1. Delimiters and Shape
- Opening delimiter: `"""` followed by optional horizontal whitespace (spaces/tabs) and a mandatory physical newline (`\n` or `\r\n`). Non-whitespace on the same line as opening `"""` is an invalid opening error (`string.multiline.opening_newline`).
- Closing line: A line containing indentation whitespace (margin) followed by `"""` and optional horizontal whitespace, terminated by a physical newline or EOF.
- The opening structural newline and pre-closing structural newline do not become part of the string value.

### 2. Margin and Indentation
- Exact-prefix rule: The exact byte sequence of horizontal whitespace preceding the closing `"""` defines the block margin.
- Every nonblank physical line in the block must start with the exact margin prefix bytes. If not, emit `string.multiline.indentation`.
- Margin prefix is stripped from each nonblank line. Any additional indentation beyond the margin is preserved as string content.
- Blank lines (containing only spaces/tabs before newline/EOF) are exempt from margin requirements, have all whitespace stripped, and contribute `\n`.

### 3. Normalization and Escapes
- Physical line breaks (`\n` and `\r\n`) normalize to `\n`.
- Raw lone CR is rejected (`string.multiline.invalid_line_ending`).
- Escapes follow standard interpreted string escapes (`\"`, `\\`, `\n`, `\t`, `\r`, `\(`). All other escapes trigger `string.invalid_escape`.

### 4. Interpolation
- `\(expr)` works identically to ordinary strings, parsing an expression and lowering to `toString` concatenation.
- Expression source ranges in AST remain absolute byte ranges into the original lexer input (not dedented coordinates).
- Multiline text blocks can be nested within interpolation expressions and vice versa.

### 5. Delimiter Collision & Symbols
- `"""` within content lines that does not meet the isolated closing line criteria is treated as literal content `"""`.
- Quoted symbols remain single-line `#"..."`. `#"""` is not a multiline symbol.

### 6. Representation & System Integration
- The lexer emits standard `Token::String` and `Token::StringInterp`. No new AST or runtime representation is added.
- Unterminated multiline string (`string.multiline.unterminated`) triggers REPL continuation with expected closer `"""`.
- LSP semantic token encoding splits multiline token ranges across physical lines so no single semantic token spans multiple lines.

---

## Consequences

- No new runtime type or bytecode opcode needed.
- Existing AST and parser lowering for string interpolation is completely reused.
