# U-FE — Hand-written Lexer + Recursive-descent Parser (as-built)

- **Status:** ✅ Landed — ADR-0016 front-end rewrite (in-tree on `main`; F10 EOF fix `f6d8753`, invalid-token span `4045085`). No single squash sha — the unit predates the per-unit commit discipline; it is recorded as landed in `../../../../forge/STATE.md` (Phase log + "U-FE follow-ups").
- **Realizes:** [ADR-0016](../../../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md); spec [lexical-structure.md](../../lexical-structure.md), [messages-and-selectors.md](../../messages-and-selectors.md). Closes forge fixes **F9** (`SyntaxError::Display` was `todo!()` → panicked on any parse error) and **F10** (trailing-newline file failed to parse).
- **Reviewer gate:** Not independently reviewed — the session ended before a `phalcom-reviewer` pass; the unit self-reports green build + `cargo doc` clean (`../../../../forge/STATE.md` §"U-FE follow-ups"). User confirmed the parser/lexer finished; later units (U4/U6/U7/U-LEX) extended it in place, exercising it heavily.

## Mission

Replace the generated front end — `logos` derive-macro lexer + LALRPOP grammar (`parser.lalrpop`) compiled by a `build.rs` codegen step — with a fully hand-written lexer and recursive-descent/Pratt parser in `phalcom-ast`. The rewrite keeps the *accepted language at exact parity* with the old grammar (no new surface syntax) while giving the front end an ordinary-Rust foundation that later units can extend with targeted lookahead instead of fighting LALR conflicts. It also folds in the two forge front-end defects (F9, F10) and drops the `lalrpop`/`lalrpop-util`/`logos` build-time dependencies from `phalcom-ast`.

## Surface / behavior

No new syntax — this unit is a re-implementation at parity. User-visible behavior changes are the two bug fixes:

- **F9:** a syntax error now renders a real spanned `miette` diagnostic and the CLI exits non-zero, instead of panicking through a `todo!()` in `SyntaxError`'s `Display`.
- **F10:** a file ending in a trailing newline (and the empty / whitespace-only file) now parses cleanly.

```phalcom
class Point {
  x => self.x
  move(dx, dy) { ... }
}
// trailing newline below now parses cleanly (F10)
```

The lexer treats **newlines as tokens** (`Token::Newline` for `\n` / `\r\n`), skips `//` line comments and horizontal whitespace as **trivia**, and injects **one single `Token::Eof`** — a zero-width point at the end of the last real token, so trailing trivia is located correctly (the F10 fix replaced a synthetic trailing-newline token with this EOF marker).

## Implementation

Front end, all in `phalcom-ast/src` (`phalcom-core` compiler and every AST snapshot unchanged — `@L`/`@R` span semantics preserved):

- **`lexer.rs`** — a byte-oriented scanner with maximal munch, no generator and no regex/`logos`. It emits the existing `Token` set (`token.rs`), skips trivia (`//` comments + horizontal whitespace), and is a `std::iter::Iterator` that injects the single `Token::Eof` once the source is exhausted, then `None`.
- **`token.rs`** — the `Token` alphabet (keywords, operators, punctuation, and the three data-carrying literals `Identifier`/`String`/`Number`) plus the lexer-level `LexicalError`, later lowered into `error.rs::SyntaxError` by `parse_source`. The `Debug` representation is stable and snapshotted by `phalcom-ast/tests/lexer.rs`.
- **`parser.rs`** — recursive descent for statements/declarations, precedence climbing (Pratt-style) for expressions and message sends over a fixed operator-precedence table (the piece most likely to drift from spec — covered by AST snapshot tests). Produces the same `ast.rs` nodes as the LALRPOP parser. **Panic-mode error recovery**: on a bad top-level statement it records the error and synchronises to the next statement boundary, so one parse surfaces many diagnostics. `parse_source` keeps the historical single-error contract (returns the first error); a new `parse` entry point returns the full recovered set.
- **`ast.rs` / `error.rs`** — AST node definitions unchanged; `SyntaxError::Display` now renders a real spanned diagnostic (F9).
- **LALRPOP removed from the front end**: `parser.lalrpop`, the `build.rs` codegen step, and the `lalrpop`/`lalrpop-util`/`logos` deps dropped from `phalcom-ast`.

## Invariants & tests

- **AST/snapshot parity:** the compiler and all AST snapshots are unchanged because span semantics and node shapes match the old grammar; the lexer `Debug` snapshots in `phalcom-ast/tests/lexer.rs` pin the token stream (e.g. `newlines_are_tokens_and_spaces_are_trivia`, `eof_sits_at_end_of_last_real_token_not_trailing_whitespace`).
- **EOF invariant:** exactly one `Token::Eof`, a zero-width span at the end of the last real token; trailing whitespace/newlines do not appear as tokens after it.
- **Recovery:** the multi-error path (`parse`) synchronises to statement boundaries — exercised by the in-crate `recovers_and_reports_multiple_errors` / `recovers_across_multiple_broken_statements` parser tests.
- Self-verified green: `cargo build`/`cargo test` for `phalcom-ast`, golden `.ph` corpus, `cargo doc` clean.

## Deviations & deferrals

- **Not independently reviewed** — session ended before the reviewer pass; see reviewer-gate note above.
- **Out-of-write-set edit:** U-FE touched `phalcom-core/bin/phalcom/cli.rs` (one line, migrating off the deleted `ProgramParser` to `parse_source`) as the sole build blocker; reported for spot-check in `../../../../forge/STATE.md`.
- **Residual LALRPOP in `phalcom-core`:** a dead `CompilerError::ParseError` variant + `From` impl referencing `lalrpop_util::ParseError`, plus `lalrpop-util`/`lalrpop` entries in `phalcom-core/Cargo.toml`, still compile and keep LALRPOP in the workspace dependency graph. Removing them is out of this unit's write-set — tracked as a follow-up (ADR-0016 §Consequences).
- **Greenfield syntax deferred to later units** that *extend* this parser: blocks/closures (U4), `let`/`var` (U6), `construct` (U7), and the U-LEX surface delta (block comments, digit separators, newline suppression, `\(expr)` interpolation — see [lex-lexical-delta.md](lex-lexical-delta.md)). Nested block comments / lone-`?` remain [DEFERRED #12/#32](../../../../forge/DEFERRED.md).

## Sources

- ADR: [0016-hand-written-lexer-and-recursive-descent-parser.md](../../../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md)
- Code: `phalcom-ast/src/lexer.rs`, `phalcom-ast/src/token.rs`, `phalcom-ast/src/parser.rs`, `phalcom-ast/src/ast.rs`, `phalcom-ast/src/error.rs`; tests `phalcom-ast/tests/lexer.rs`, `phalcom-ast/tests/parser.rs`.
- Forge: [STATE.md](../../../../forge/STATE.md) (Phase log; "U-FE follow-ups"); [PHASE2-INDEX.md](../../../../forge/PHASE2-INDEX.md).
- Deferred: [deferred-work.md](../../deferred-work.md); [DEFERRED.md](../../../../forge/DEFERRED.md) #12/#32.
