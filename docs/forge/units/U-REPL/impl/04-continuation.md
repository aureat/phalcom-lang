# §04 — Stage 3: multi-line continuation (§D7 REPL half)

**Phase B — branch-only. Touches `phalcom-repl/**` and nothing else.** First stage with
zero conflict surface against the class work.

## 1. What already landed

The **parser half** shipped as `2fe6aba`. `Parser::error_here` routes `Token::Eof` to
`SyntaxErrorKind::UnrecognizedEof { expected }`; every other token still yields
`UnrecognizedToken { token, expected }`. `class Foo {` now renders
`Unexpected end of file. Expected "}"`.

`phalcom-ast/tests/probe_continuation.rs` already asserts the full classification table
and is the reference implementation of the rule below — **read it before writing the
validator**; the validator must agree with it exactly.

## 2. The classification rule

```rust
if parsed.errors.is_empty()                                        { Complete }
else if any error is SyntaxErrorKind::UnrecognizedEof { .. }       { Incomplete }
else                                                               { Invalid }
```

That is the whole rule. Do not add delimiter counting, and do not special-case strings.

| input | verdict |
|---|---|
| `let x = 1`, `` (empty) | complete |
| `class Foo {`, `let x = 1 +`, `foo(1,`, `[1, 2,`, `if (x) {` | incomplete |
| `let s = "abc` | incomplete — strings span lines (precondition 8) |
| `let x = )`, `1 +* 2` | invalid — non-empty offending token |

**Unterminated strings report two errors** — `UnterminatedString` *and* the trailing EOF
error. The "any error is `UnrecognizedEof`" phrasing handles this correctly; a rule
written as "the *first* error is `UnrecognizedEof`" does not. This is why the rule is
`any`, not `first`.

Rejected (plan.md §D7): delimiter counting (blind to `let x = 1 +`); Python's `codeop`
compile-and-retry (exists only because CPython exposes no clean signal — Phalcom now
has one).

## 3. The reedline `Validator`

Precondition 9: reedline 0.41 exposes `Validator` / `ValidationResult`, invoked **per
submission, not per keystroke**. So the cost of a full parse here is irrelevant to
§S6's hard real-time constraint — this is not on the input path.

```rust
impl Validator for PhalcomValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        match classify(line) {
            Verdict::Incomplete => ValidationResult::Incomplete,
            _ => ValidationResult::Complete,   // Invalid submits and reports
        }
    }
}
```

**`Invalid` submits.** A genuinely malformed line must reach the evaluator so the user
sees the diagnostic. Holding it in the buffer waiting for input that cannot fix it is
the trap §4 exists to prevent.

Wire at `phalcom-repl/src/main.rs:196`, where `Reedline::create()` is chained
(`.with_edit_mode`, `.with_completer`, `.with_highlighter`) — add `.with_validator`.

## 4. Escape hatches — a mis-detected `Incomplete` must never trap the user

Three, all required:

- **Blank line submits as-is.** If the buffer is non-empty and the user submits an empty
  line, submit the buffer regardless of verdict. This is the escape from a wrong
  `Incomplete`, and it is the only one that works when the classifier is the thing that
  is broken.
- **Ctrl-C discards** the pending buffer and returns to a fresh primary prompt without
  exiting the REPL.
- **`...` continuation prompt** so the state is visible. `PhPrompt`
  (`phalcom-repl/src/main.rs:99`) already carries primary and continuation strings and
  implements `reedline::Prompt` at `:119` — set the continuation indicator, do not build
  a second prompt type.

## 5. Trailing `\` — explicit continuation

A line ending in `\` continues unconditionally, whatever the classifier says.

**It must be stripped and joined *before* lexing.** `\` is not in the grammar and lexes
as an invalid token. Strip the trailing `\`, join with the next line, and lex the result.

> **Consequence, accepted in plan.md §D7:** compiled text ≠ typed text, so byte offsets
> shift. `ModuleObject.sources` (§D2) stores the **compiled** text, so spans stay valid;
> history and echo therefore show the joined form, not what was typed. Do not try to
> store the typed text — that reintroduces exactly the span/source mismatch §D2 removed.

Decide and document the join: whether `a \` + `b` becomes `a b` or `ab`. Recommended
`a b` (replace `\` with a space) — it matches shell precedent and cannot fuse two
identifiers into one.

## 6. Tests

Classification is already covered by `probe_continuation.rs`. This stage adds:

| Test | Asserts |
|---|---|
| `validator_matches_probe_classification` | the validator agrees with `probe_continuation.rs` on all 15 inputs |
| `blank_line_submits_incomplete_buffer` | escape hatch fires regardless of verdict |
| `trailing_backslash_joins_before_lexing` | `let x = 1 + \` / `2` compiles; no `InvalidToken` |
| `invalid_input_submits_rather_than_waiting` | `let x = )` reaches the evaluator |

The first is the one that stops drift: two implementations of one rule, in two crates,
will diverge unless something asserts they agree. If duplicating the table is awkward
across the crate boundary, prefer exporting `classify` from `phalcom-ast` and having
both call it — a shared implementation is better than a shared test.

## 7. Write-set

| Path | Change |
|---|---|
| `phalcom-repl/src/main.rs` | `PhalcomValidator`; `.with_validator`; continuation indicator; Ctrl-C |
| `phalcom-repl/src/repl.rs` | `\` joining before submission |
| `phalcom-ast/src/parser.rs` | **only if** `classify` is exported for sharing (§6) |

**Conflict risk vs class work:** none for the `phalcom-repl` files. U-CLASSCLOSE touches
`phalcom-ast/src/parser.rs` (nested-class rejection at `:1557-1559`) — if §6's shared
`classify` is added, it goes at the error/classification region, far from that, but this
is the one line of this stage that is not conflict-free. Prefer duplicating the rule over
creating a `parser.rs` conflict if the class work is mid-flight.

## 8. Gate

Workspace green. Manual: `cargo run -p phalcom-repl`, then type `class Foo {`, confirm
the `...` prompt appears, add `}`, confirm it evaluates; type `let x = )`, confirm it
errors immediately rather than waiting; type `class Bar {` then a blank line, confirm it
submits and reports.
