# U-LEX — Lexical Surface-Syntax Delta (as-built)

- **Status:** ✅ Landed — `dba9d49` (D1), `6660517` (D2), `ee244b2` (D3), `eb10b69` (D4), `fef1a7e` (D5), `d91cdf4` (docs). In-tree on `main`, no worktree; committed per green checkpoint D1→D2→D3→D5→D4.
- **Realizes:** [ADR-0022](../../../adr/0022-string-interpolation-backslash-paren-sigil.md) (new — `\(expr)` sigil); spec [lexical-structure.md](../../../spec/v0.2/lexical-structure.md) §5 (interpolation), §1 (newline suppression). Extends the U-FE hand-written lexer ([fe-front-end.md](../U-FE/as-built.md), [ADR-0016](../../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md)).
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
- **D3** — new `Lexer.last_significant: Option<Token>` field + the free predicate `suppresses_following_newline(prev: &Token)`; `next()` loops and swallows a `Token::Newline` when the previous significant token is a suppressor. **Suppressor set (committed):** arithmetic `+ - * / %`; comparison `== != < <= > >=`; logical keywords `and or not`; assignment `= += -= *= /= %=`; Option ops `?? ?.`; openers/separators `, ( { [ . ::` **and `Colon`** (the one judgment call — per §1's map/label shape); arrows `-> =>`. `last_significant` is only updated for non-`Newline` tokens.
- **D4** — new `Token::StringInterp(Vec<StringSegment>)` + `StringSegment` (`Literal` / `Expr`) in `token.rs`. `scan_string` splits on `\(…)` by balanced-paren depth, emitting ordered segments (plain strings still return `Token::String`; `\\(` = literal backslash-paren). The parser desugars `StringInterp` **in place** — no AST node, no compiler change — to a `+`-chain of `String` literal nodes and `String.new(expr)` stringify sends.
- **D5** — no code; one `lexical` PASS fixture `lexical_option_operators` exercising U6's `parse_coalesce`/`parse_optional_send` desugars end-to-end.

## Invariants & tests

- **Golden + `lang` corpus byte-identical** across D1–D5; only one existing lexer snapshot was legitimately re-blessed by D3 (`class_with_static_method` loses exactly the one `Token::Newline` after its first `{`), and two in-crate recovery tests whose sources ended in suppressor tokens were rewritten to value-ending lines (intent preserved).
- **New lexer snapshots:** `block_comment_is_skipped`, `numeric_digit_separators`, `newline_after_operator_is_suppressed`, `newline_after_value_is_preserved`, plus two string-interpolation snapshots.
- **New `lexical` PASS fixtures:** `comments_block`, `lexical_numeric_separator_float`, continuation/logical/guard fixtures (D3), `lexical_option_operators` (D5), the promoted+rewritten `lexical_string_interpolation` + a multi-expr/escape case. **NEGATIVE fixtures:** `syntax_unterminated_block_comment`, `syntax_double_digit_separator` (`1__0`).
- Green gate: `./scripts/verify.sh` exit 0; `cargo doc --workspace --no-deps` no new `phalcom-ast` warnings.

## Deviations & deferrals

- **D4 sigil override:** the architect recommended `{expr}`; the user ratified **`\(expr)`** (Swift-style) — recorded as [ADR-0022](../../../adr/0022-string-interpolation-backslash-paren-sigil.md).
- **D4 desugar target:** the spec's illustrative desugar used `expr.toString`, but no value-type content `toString` exists yet (blocked on U-CORE-4). `String.new(expr)` is the working content-stringify today — [DEFERRED #30](../../phase-next/DEFERRED.md) (same root cause as #19).
- **Interpolation scanning is balanced-paren only** — it does not understand a string literal nested inside a `\(…)` expression (`"\(f(")"))"` mis-terminates). Accepted for v1 — [DEFERRED #31](../../phase-next/DEFERRED.md).
- **Block comments are flat (non-nesting)**; nested block comments and the reserved lone-`?` remain [DEFERRED #12/#32](../../phase-next/DEFERRED.md).
- See also [deferred-work.md](../../../spec/v0.2/deferred-work.md).

## Sources

- ADR: [0022-string-interpolation-backslash-paren-sigil.md](../../../adr/0022-string-interpolation-backslash-paren-sigil.md); [0016-hand-written-lexer-and-recursive-descent-parser.md](../../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md).
- Code: `phalcom-ast/src/lexer.rs` (`skip_trivia`, `scan_number`/`scan_digits`, `scan_string`, `suppresses_following_newline`, `next`), `phalcom-ast/src/token.rs` (`StringInterp`/`StringSegment`, `LexicalError::UnterminatedBlockComment`), `phalcom-ast/src/parser.rs` (interpolation desugar); tests `phalcom-ast/tests/lexer.rs` + `phalcom-core/tests/lang/lexical/`.
- Forge: [STATE.md](../../archive/phase2/STATE.md) §"U-LEX — LANDED". Per-unit planning record (`U-LEX-implementation-spec.md`, `U-LEX-plan.md`) folded into this spec; see git history.
- Deferred: [deferred-work.md](../../../spec/v0.2/deferred-work.md); [DEFERRED.md](../../phase-next/DEFERRED.md) #12/#30/#31/#32.
