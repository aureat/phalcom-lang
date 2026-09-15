# LANG001.C3 — Lexical surface syntax delta spec

- **Status:** ✅ Landed — `dba9d49` (D1), `6660517` (D2), `ee244b2` (D3), `eb10b69` (D4), `fef1a7e` (D5), `d91cdf4` (docs). In-tree on `main`, no worktree; committed per green checkpoint D1→D2→D3→D5→D4.
- **Realizes:** [TDR-0020](../../../decisions/accepted/0020-string-interpolation-backslash-paren-sigil.md) (new — `\(expr)` sigil); spec [lexical-structure.md](../../../spec/current/lexical-structure.md) §5 (interpolation), §1 (newline suppression). Extends the U-FE hand-written lexer ([fe-front-end.md](../U-FE/fe-lexer-recursive-descent-parser.md), [TDR-0014](../../../decisions/accepted/0014-hand-written-lexer-and-recursive-descent-parser.md)).
- **Reviewer gate:** OFF per policy (surface syntax, not load-bearing-hierarchy) — self-verified on the green gate (`../../archive/phase2/STATE.md` §"U-LEX — LANDED"; reviewer roster line: "Reviewer OFF … U-LEX").

## Mission

Ship the five-part surface-syntax delta on top of the U-FE front end, **entirely within `phalcom-ast`** (+ the `lexical` fixture corpus). `phalcom-core/src` and `core.ph` were untouched — the desugarings live in the parser, matching the existing `if`/`while`/`??`/`?.` idiom of keeping the compiler-visible AST unchanged. The five parts: **D1** block comments, **D2** digit separators, **D3** lexer-level newline suppression, **D4** `\(expr)` string interpolation, **D5** `?.`/`??` end-to-end coverage.

## Surface / behavior

```phalcom
/* block comment — flat, does not nest */
let million = 1_000_000          // digit separators
let total = a +                  // trailing operator suppresses the newline;
            b                    //   this continues the same statement
let greeting = "hi \(name), you have \(count) msgs"   // \(expr) interpolation
let x = maybe?.field ?? fallback // ?. optional-send, ?? null-coalescing
```

- **D1 — block comments `/* … */`:** flat / non-nesting, scanned as trivia. EOF before `*/` is a lexical error (unterminated comment) with the real span.
- **D2 — digit separators `1_000_000`:** interior `_` between digits, stripped before parsing. A misplaced `_` (trailing, doubled, or adjacent to `.`) is a lexical error.
- **D3 — newline suppression:** a `Token::Newline` is swallowed when the previous significant token cannot end a statement, so trailing-operator continuations span physical lines. One-sided (keys on the previous token only), **not** parser ASI.
- **D4 — string interpolation `\(expr)`** (Swift-style sigil, ADR-0022): a string body with at least one `\(expr)` lexes to `Token::StringInterp`; plain strings still lex to `Token::String`. `\\(` is a literal `\(`.
- **D5 — `?.`/`??`:** the operators themselves shipped in U6; U-LEX adds an end-to-end `lexical` fixture only (no lexer/parser change).

## Implementation

All in `phalcom-ast/src/lexer.rs` (+ `token.rs`, `parser.rs` for D4 desugar):

- **D1** — `skip_trivia` now returns `Result<(), LexicalError>` (spec option (a); signature threaded through its sole caller `next()`). It consumes `/* … */` as flat trivia; EOF before `*/` returns the new `LexicalError::UnterminatedBlockComment(open..pos)`, lowered in `lex_error_to_syntax` to the existing `SyntaxErrorKind::UnterminatedComment` with the real offset-adjusted span. `error.rs` untouched.
- **D2** — `scan_number` accepts interior `_` via a new `scan_digits` helper, stripping separators before `parse::<f64>()`. A misplaced `_` → `LexicalError::InvalidToken` (**reused** — no new `SyntaxErrorKind`) carrying the `_` span. `Token::Number` unchanged.
- **D3** — new `Lexer.last_significant: Option<Token>` field + the free predicate `suppresses_following_newline(prev: &Token)`; `next()` loops and swallows a `Token::Newline` when the previous significant token is a suppressor. **Suppressor set (committed):** arithmetic `+ - * / %`; comparison `== != < <= > >=`; logical keywords `and or not`; assignment `= += -= *= /= %=`; Option ops `?? ?.`; openers/separators `, ( { [TDR-0020](../../../decisions/accepted/0020-string-interpolation-backslash-paren-sigil.md).
- **D4 desugar target:** the spec's illustrative desugar used `expr.toString`, but no value-type content `toString` exists yet (blocked on U-CORE-4). `String.new(expr)` is the working content-stringify today — [DEFERRED #30](../../DEFERRED.md) (same root cause as #19).
- **Interpolation scanning is balanced-paren only** — it does not understand a string literal nested inside a `\(…)` expression (`"\(f(")"))"` mis-terminates). Accepted for v1 — [DEFERRED #31](../../DEFERRED.md).
- **Block comments are flat (non-nesting)**; nested block comments and the reserved lone-`?` remain [DEFERRED #12/#32](../../DEFERRED.md).
- See also [deferred-work.md](../../../spec/current/deferred-work.md).

## Sources

- ADR: [TDR-0020](../../../decisions/accepted/0020-string-interpolation-backslash-paren-sigil.md); [TDR-0014](../../../decisions/accepted/0014-hand-written-lexer-and-recursive-descent-parser.md).
- Code: `phalcom-ast/src/lexer.rs` (`skip_trivia`, `scan_number`/`scan_digits`, `scan_string`, `suppresses_following_newline`, `next`), `phalcom-ast/src/token.rs` (`StringInterp`/`StringSegment`, `LexicalError::UnterminatedBlockComment`), `phalcom-ast/src/parser.rs` (interpolation desugar); tests `phalcom-ast/tests/lexer.rs` + `phalcom-core/tests/lang/lexical/`.
- Forge: [STATE.md](../../archive/phase2/STATE.md) §"U-LEX — LANDED". Per-unit planning record (`U-LEX-implementation-spec.md`, `U-LEX-plan.md`) folded into this spec; see git history.
- Deferred: [deferred-work.md](../../../spec/current/deferred-work.md); [DEFERRED.md](../../DEFERRED.md) #12/#30/#31/#32.
