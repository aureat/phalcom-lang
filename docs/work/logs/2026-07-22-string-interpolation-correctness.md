# Phalcom String Interpolation Correctness Implementation

- Date: 2026-07-22
- Specification: `docs/work/pending/string-interpolation-completion.md`
- Normative Spec: `docs/spec/current/string-interpolation.md`
- PDR: [PDR-0029](../../pdr/0029-string-literals-and-interpolation-completion.md)

## 1. Summary

Brought Phalcom double-quoted string literals and string interpolation into full compliance with the normative specification:
- conventional escape sequence decoding (`\"`, `\\`, `\n`, `\t`, `\r`);
- rejection of physical line breaks (LF, CRLF, lone CR) in ordinary double-quoted strings with `string.raw_newline`;
- mode-aware interpolation boundary scanning (`\(` through balancing `)` aware of nested strings, line comments, block comments, and parenthesized sub-expressions);
- exact-one-expression rule enforcing single expression bodies and rejecting empty/trivia-only bodies;
- stable diagnostic codes (`string.invalid_escape`, `string.interpolation.unterminated`, `string.interpolation.empty`, `string.raw_newline`);
- AST lowering desugaring string interpolation to left-associative String concatenation with `toString` sends;
- runtime integration tests covering left-to-right evaluation, String type validation, exception short-circuiting, and user `toString` overrides;
- active specification document updates.

## 2. Key Code Changes

### AST (`phalcom-ast`)
- `token.rs`: Updated `StringSegment::Expr` to store `range: Range<usize>`. Added `LexicalError` variants: `InvalidEscape`, `UnterminatedInterpolation`, `RawNewlineInString`.
- `error.rs`: Added `SyntaxErrorKind` variants and exposed stable error codes via `SyntaxErrorKind::code(&self)` and `SyntaxError::code(&self)`.
- `lexer.rs`: Added reusable comment scanning helpers and `scan_interpolation_body` for mode-aware boundary scanning. Updated `scan_string` to decode valid escapes, reject raw newlines, and scan interpolation bodies.
- `parser.rs`: Updated `lex_error_to_syntax` and `push_lex_error`. Implemented `parse_interp_expr` to enforce exact-one-expression requirement. Lowered interpolation segments left-associatively with `toString` getter calls on outer string range.

### Core (`phalcom-core`)
- Added integration tests covering string interpolation evaluation order, exactly-once getter execution, runtime String return validation, exception handling, and custom `toString` overrides.

### Documentation (`docs/`)
- Updated `docs/spec/current/string-interpolation.md`, `docs/spec/current/syntax/grammar.md`, `docs/spec/current/syntax/lexical.md`, `docs/spec/current/syntax/expressions.md`, and `docs/spec/current/syntax/README.md`.

## 3. Verification

- `cargo fmt --check`: Passed
- `cargo clippy --workspace --all-targets -- -D warnings`: Verified for `phalcom-ast` and workspace lib targets
- `cargo test --workspace`: Passed (all unit, integration, conformance, diagnostic, and REPL test suites passed)
- Reproduction command: `cargo run -- -i 'System.print("\(String.new(")"))")'` returned `)` as specified without syntax error.
