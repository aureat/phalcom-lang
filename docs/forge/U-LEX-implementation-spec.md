# U-LEX — Implementation Specification (supersedes U-LEX-plan.md on conflict)

_Grounded against actual HEAD as of commit `4e2ec73` (U10 landed, green;
history since the plan was written on 2026-07-11: `b99ad22`/`806c9ea` U8,
`c9805d0` U9, `4e2ec73` U10). This document exists because `U-LEX-plan.md` was
written in a batch planning session **before U6 actually landed**, and its
single largest work item — **D5 (`?.`/`??` Option operators)** — turned out to
have been implemented **in full** as part of U6 (commit `3bc6ede`). Two smaller
write-set claims (which file owns the error plumbing; which token names to add)
are likewise stale. **Where this document and `U-LEX-plan.md` disagree, follow
this document.** Where this document is silent, `U-LEX-plan.md` still governs
(mission framing, guardrails, mandatory rules, return-contract shape)._

Written for a **medium-effort implementer** — every non-trivial decision below
is already made. Do not re-derive them. If you hit a fact that contradicts this
doc, STOP and report the conflict rather than guessing — this doc's job is to
have already done the HEAD archaeology so you don't have to.

---

## 0. The four corrections to `U-LEX-plan.md`

1. **D5 (`?.` and `??`) is ALREADY LANDED — do not touch it, do not re-add it.**
   This is the headline correction, exactly analogous to U9's "`signature.rs`
   is a dead stub" and U10's "`disasm.rs` needs no change." The plan's D5 (§4.5,
   build-order step 4) assumes `?.`/`??` are new tokens U-LEX must add, gated on
   "depends on U4 + U6." In reality **U6 shipped the entire feature** (commit
   `3bc6ede`, "feat(u6): absence→Option + let/var bindings — core wiring"),
   because the operators desugar to `Option.map`/`Option.orElse` and U6 owned
   the `Option` model. Confirmed at HEAD:
   - **Tokens already exist** (`phalcom-ast/src/token.rs`): `Token::Question`
     (`151-156`, lone `?`, reserved/unused by the grammar), `Token::CoalesceQuestion`
     (`157-162`, `??`), `Token::QuestionDot` (`163-168`, `?.`) — with rustdoc
     already citing ADR-0007, `values-and-absence.md §3.4`, and `lexical-structure.md §9`.
   - **The lexer already maximal-munches them** (`phalcom-ast/src/lexer.rs:270-274`):
     `?? ` → `CoalesceQuestion`, `?.` → `QuestionDot`, lone `?` → `Question`,
     with the exact comment "`Multi-char ?? and ?. take priority over a lone ?`".
   - **The parser already desugars both, exactly as the plan's D5 prescribes:**
     - `parse_coalesce` (`phalcom-ast/src/parser.rs:877-898`) lowers `a ?? b` to
       `a.orElse { b }` — right-associative (recurses on the right operand),
       sitting between `parse_assignment` (`820`) and `parse_binary(1)` (`879`),
       i.e. looser than every arithmetic/comparison operator but tighter than
       assignment. Matches `lexical-structure.md §9` precisely.
     - `parse_optional_send` (`parser.rs:1077-1118`), reached from `parse_call`'s
       postfix loop (`parser.rs:994-997`), lowers `opt?.m(a)` to
       `opt.map { <recv> => <recv>.m(a) }` — left-associative through the loop,
       so a chain `a?.b?.c` groups `(a?.b)?.c` and the first `None`
       short-circuits. The mapper uses a **synthetic, non-lexable ` recv`
       parameter name** (leading space — cannot collide with a user variable),
       which is strictly better than the plan's suggested `it`.
   - **`STATE.md` line 273 records it as a shipped U6 deliverable** ("`??` / `?.`
     parser desugar (values-and-absence §3.4–3.5)").
   - **Consequences for you:** the plan asks D5 to add `Token::QuestionQuestion`.
     That name is **wrong** (the landed token is `CoalesceQuestion`) and the work
     is **done**. Do **not** add any `?.`/`??` token, lexer arm, parser rule, or
     desugar. Touching that code would collide with U6's landed implementation.
     Your only obligation to D5 is a **new PASS corpus fixture** proving it works
     end-to-end (§5) — U6 added the machinery but no `lexical`-labelled `.ph`
     acceptance case exercises it; add one so the surface operator is covered
     under your label.
   - **Semantics reconciliation (the question the dispatcher asked):** the plan's
     D5 description (`a ?? b → a.orElse { b }`, `opt?.m(x) → opt.map { it => it.m(x) }`)
     **is consistent with U6's landed Option model** — no reconciliation of
     meaning is needed. The only mismatch is that the plan predates U6 *landing*
     the feature, not that it describes it wrongly.

2. **D1's diagnostic kind already exists; the error plumbing does NOT live where
   the plan says.** The plan's §3 write-set lists `phalcom-ast/src/error.rs` for
   "`Display`/kinds for the new `LexicalError` variants (unterminated block
   comment ...)." At HEAD:
   - `SyntaxErrorKind::UnterminatedComment` **already exists**, fully documented,
     with `#[error("Unterminated comment")]` (`phalcom-ast/src/error.rs:156-158`)
     — a strong signal D1 was anticipated. **`error.rs` needs NO change for D1.**
   - The `LexicalError` enum lives in **`token.rs`** (`201-215`), not `error.rs`.
     A new `LexicalError::UnterminatedBlockComment(Range<usize>)` variant is added
     **there**.
   - The `LexicalError` → `SyntaxError` lowering lives in **`parser.rs`**
     (`lex_error_to_syntax`, `parser.rs:122-145`). D1 adds one arm there mapping
     the new variant to `SyntaxErrorKind::UnterminatedComment`, carrying the real
     span (`(span.start + offset)..(span.end + offset)`), mirroring the
     `UnterminatedString` arm at `parser.rs:136-139`. **Do not repeat DEFERRED #1's
     bug** — do not lower to a zero-width `0..0` range the way the
     `InvalidInteger`/`InvalidFloat` arms (`124-131`) currently do.
   So D1's error path is **token.rs (new variant) + parser.rs (new lowering arm),
   with error.rs untouched** — not "error.rs" as the plan states.

3. **The `lexical` lang label already exists, passes, and already has the pending
   fixtures for your unblocked work staged.** The plan (§2, §3) treats
   `phalcom-core/tests/lang/lexical/*` as a directory you populate from scratch.
   At HEAD it is live:
   - `lang.rs:8-11` — `fn lexical() { check_pass("lexical") }` (runs in the green
     gate); `lang.rs:13-17` — `#[ignore] fn lexical_pending() { check_pending("lexical") }`.
   - Active PASS cases: `comments_inline.ph` (only `//`), `lexical_comment_only.ph`,
     `lexical_trailing_comment.ph`, `lexical_multi_statement_{semicolon,newline}.ph`.
   - **Pending cases already written for your work:**
     `pending/lexical_numeric_separator.ph` (D2 — `System.print(1_000_000)`,
     expects `1000000`) and `pending/lexical_string_interpolation.ph` (D4 —
     `let name = "Ada"` / `System.print("{name} is great")`, expects `Ada is great`).
   - **Promotion mechanism** (from `support/mod.rs::check_cases`, `137-163`):
     `check_pending` runs cases under `.../lexical/pending/` and asserts stdout
     matches (`assert_stdout_exact`), but the `lexical_pending` test is `#[ignore]`
     so they stay out of the green gate. A feature "graduates" by **`git mv`-ing
     the `.ph` + `.expected` pair out of `pending/` up into `lexical/`**, where the
     non-ignored `lexical()` test picks it up. So when D2 lands you `git mv`
     `pending/lexical_numeric_separator.*` → `lexical/`; you do not author a new
     fixture for it.

4. **D1/D2/D3 need NO `ast.rs` and NO `token.rs`-token changes — only D4 does
   (and D4 is blocked).** The plan's §3 write-set lists `phalcom-ast/src/ast.rs`
   ("interpolated-string node") and `token.rs` ("new tokens ... interpolation-segment
   tokens") as if broadly in scope. Ground truth:
   - **D1 (block comments)** is pure trivia — extend `skip_trivia`
     (`lexer.rs:80-100`); it produces **no token** and needs no AST node. Its only
     `token.rs` touch is the `LexicalError::UnterminatedBlockComment` variant
     (correction 2).
   - **D2 (digit separators)** folds into the existing `Token::Number(f64)` — strip
     `_` before `parse::<f64>()` in `scan_number` (`lexer.rs:142-155`). **No new
     token, no AST node.**
   - **D3 (newline suppression)** is lexer-internal state — **no new token, no AST
     node** (it emits *fewer* `Token::Newline`s, never a new variant).
   - **D4 (interpolation)** is the *only* item needing a new `token.rs` token and a
     new `ast.rs` node — and D4 is **BLOCKED-ON-DECISION** (§2 D4). So under the
     unblocked slice you ship, **`ast.rs` is untouched** and `token.rs` is touched
     for exactly one thing: the D1 `LexicalError` variant.

---

## 1. Preconditions — already verified, do not re-check

- **U1–U10 are merged and green** (HEAD `4e2ec73`). `./scripts/verify.sh` is green
  on `main` right now. Run it once before your first edit to confirm your baseline.
- **`phalcom-ast` is quiescent — you are the sole editor.** `git status` at HEAD
  shows only docs/config changes (no `.rs`). U4/U5/U6/U7 (the other four
  `phalcom-ast` contenders per `PHASE2-INDEX.md §3`) have **all landed**; U-LEX is
  the last `phalcom-ast` contender, and no spine `phalcom-ast` work is concurrent.
  The collision-matrix constraint "U-LEX must run alone in `phalcom-ast`" is
  satisfied. **U-STD (`core.ph`) and U11 (`core.ph`/primitives) touch neither
  `lexer.rs`/`parser.rs`/`token.rs`/`ast.rs`/`error.rs` nor your corpus dir**, so
  U-LEX ‖ U-STD (and ‖ U11) is genuinely disjoint — safe to co-schedule.
- **`?.`/`??` are done** (§0 correction 1) — do not open that code.
- **`SyntaxErrorKind::UnterminatedComment` exists** (`error.rs:156-158`); D1 reuses
  it (§0 correction 2).
- **The lexer is a plain byte scanner, hand-written** (`lexer.rs`, ADR-0016). It
  is *not* generated — extend it with ordinary Rust. `skip_trivia` (`80-100`),
  `scan_number` (`142-155`), `scan_string` (`208-221`), `scan_operator` (`233-290`),
  and the `Lexer` struct (`41-55`) are the surfaces you touch. `next()`
  (`298-318`) is the single emit point.
- **The lexer has AST/insta snapshots** at `phalcom-ast/tests/lexer.rs` (16
  cases, `insta::assert_debug_snapshot!` of the token stream). **D3 will change
  exactly one existing snapshot** — see §2 D3 and §5. Accept snapshot diffs with
  `cargo insta review` / `INSTA_UPDATE=always cargo test -p phalcom-ast`, and
  **confirm the diff set is exactly what §5 predicts** (an unexpected extra diff
  means the suppression predicate is wrong).
- **The golden harness** (`phalcom-core/tests/golden.rs`) runs
  `examples/{core_new,person,person2,calculator}.ph` + a few `fixtures/golden/*.ph`
  through the real CLI and asserts byte-exact stdout. **These are the D3
  regression tripwires** — every one must stay byte-identical (§5).
- **DEFERRED entries for this unit's out-of-scope tail already exist** — do not
  duplicate them: **#6** (collection-literal lowering `(a,b)`/`[…]`/`{a:1}`, owned
  by U-LEX, needs a new ADR), **#12** (lexer polish: nested block comments, lone-`?`
  ternary, real span through `LexicalError`), **#20** (collection combinators/literal
  syntax). Update/extend, don't re-file.
- **Isolation:** the plan says "own worktree." U8/U9/U10 all actually landed
  **in-tree on `main`, no worktree**. Confirm with the user which convention to
  use before starting (U9/U10 specs made the same call). Commit at each green
  checkpoint, same convention as U9/U10. Run `graphify update . --no-cluster`
  before each commit.

---

## 2. Design decisions

### D1 — Block comments `/* … */` (spec §2) — UNBLOCKED

Extend `skip_trivia` (`lexer.rs:80-100`) to consume `/* … */` as trivia, alongside
the existing `//` arm. **Non-nesting** (spec §2 shows a flat block comment and is
silent on nesting; a flat scan keeps `skip_trivia` a simple loop and matches the
C/Java surface expectation for the shown form). On end-of-input before `*/`,
return `LexicalError::UnterminatedBlockComment(open..self.pos)` with the real span.

Implementation shape:
- In `skip_trivia`, add an arm `b'/' if self.peek_at(1) == Some(b'*')`: advance
  past `/*`, then scan until `*/` (or EOF). **Problem:** `skip_trivia` currently
  returns `()` and cannot report an error. Two clean options — pick one, document
  the choice in your return report:
  - **(a, recommended)** Have `skip_trivia` return `Result<(), LexicalError>` (or
    an `Option<LexicalError>`), and thread it through the two callers (`next()` at
    `299`, and — check — any direct call). This is the least surprising and keeps
    the error at the exact byte where scanning failed.
  - **(b)** Detect the unterminated block comment lazily in `scan_token`/`next` by
    having `skip_trivia` set a `pending_error: Option<LexicalError>` field on
    `Lexer` that `next()` drains. More state, avoids changing `skip_trivia`'s
    signature. Only prefer this if (a)'s signature change ripples further than
    expected.
- Error plumbing (§0 correction 2): add `LexicalError::UnterminatedBlockComment(Range<usize>)`
  to `token.rs`; add the lowering arm to `lex_error_to_syntax` (`parser.rs:122-145`)
  mapping it to `SyntaxErrorKind::UnterminatedComment` (already exists) with the
  offset-adjusted span. **No `error.rs` change.**

_Forward-looking:_ nested block comments are DEFERRED #12 — leave a one-line note,
do not pre-build a nesting counter.

### D2 — Digit separators `1_000_000` (spec §4) — UNBLOCKED

In `scan_number` (`lexer.rs:142-155`), accept `_` **between** digits: never
leading (already impossible — a number can't start with `_`, it would lex as an
identifier), never trailing, never doubled, never adjacent to the `.`. Strip all
`_` from the matched slice before `parse::<f64>()` (`slice.replace('_', "").parse::<f64>()`
or scan into an owned buffer). Preserve the existing `1..2` / `3.method` decimal
rule unchanged (the `.`-followed-by-digit guard at `lexer.rs:147` stays).

Rejection: a malformed separator (trailing `_`, `_` adjacent to `.`, or `__`) is a
**lexical error** — not a silent strip. **Reuse `LexicalError::InvalidToken(span)`**
(lowers to the existing `SyntaxErrorKind::InvalidToken`, zero new error plumbing)
with the span of the offending `_`; do not add a new `SyntaxErrorKind`. Document
that choice.

`Token` is unchanged (still `Token::Number(f64)`) — **no snapshot regression on
existing lexer snapshots** (none contain `_` in a number). Add one **new** lexer
snapshot for `1_000_000` (§5), and **`git mv` the pending `lexical_numeric_separator`
fixture** into the active corpus.

### D3 — Newline-suppression state machine (spec §1) — UNBLOCKED · **the load-bearing item**

Spec §1 is explicit and prescriptive: newline suppression is a **lexer-level state
machine, NOT parser ASI** ("that is how JavaScript acquired the `return\n{}` bug").
Implement it in the lexer, leaving the parser's newline-as-terminator logic
untouched — the parser simply sees fewer newlines.

**Data structure.** Add one field to `Lexer` (`lexer.rs:41-55`):
`last_significant: Option<Token>` (or a small `enum`/`bool` if you prefer — but you
need the *token identity*, so `Option<Token>` is simplest). Update it in `next()`
(`lexer.rs:298-318`) **only** on a successful non-`Newline`, non-trivia emit (trivia
is already skipped before `scan_token`; you only ever call `scan_token` on real
tokens, so update `last_significant` on every `Ok(token)` that is not
`Token::Newline`). Then, when `scan_token` yields `Token::Newline`, consult a
`fn suppresses_following_newline(prev: &Token) -> bool` predicate: if
`last_significant` is `Some(t)` and `suppresses_following_newline(&t)` is true,
**skip the newline and loop to scan the next token** instead of emitting it. (Cleanest
site: a small `loop` in `next()` around the `scan_token` call, or recurse `next()`.)

**The predicate — suppress iff the previous significant token cannot end a
statement.** Grounded in spec §1's list, mapped to the actual `Token` variants:
- Arithmetic: `Plus`, `Minus`, `Asterisk`, `Slash`, `Percent`
- Comparison: `EqualEqual`, `BangEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`
- Logical keywords: `And`, `Or`, `Not`
- Assignment: `Equal`, `PlusEqual`, `MinusEqual`, `AsteriskEqual`, `SlashEqual`, `PercentEqual`
- Option ops: `CoalesceQuestion`, `QuestionDot`
- Openers/separators: `Comma`, `LParen`, `LBrace`, `LBracket`, `Dot`, `ColonColon`
- Arrows: `Arrow` (`->`), `FatArrow` (`=>`)
- `Colon` — **the one judgment call.** A line ending in `:` (map key / label position)
  cannot end a statement, so suppression is defensible; but `:` is rarer at line-end
  and its map/label consumers are not fully landed. **Recommend: include `Colon`**
  (spec §1's "etc." plus the map-literal shape in §6 both point that way), but call
  it out explicitly in your return report so the decision is visible.

**Do NOT suppress after** (these CAN end a statement): `Identifier`, `String`,
`Number`, `RParen`, `RBrace`, `RBracket`, `SelfKw`, `Super`, `True`, `False`,
`Return`, `Break`, `Continue`, and the `Newline`/`Semicolon`/`Eof` markers
themselves. Keywords like `Let`/`Var`/`If`/`While`/`Class`/`Fn` are **not** in the
suppress set — spec §1 scopes the rule to operators/punctuation; do not over-reach
into statement keywords.

**One-sided only.** The rule keys on the **previous** (trailing) token, never the
*next* token. So leading-operator continuation (`foo\n.bar`, `x\n+ y`) is **not**
supported — that is spec-correct (§1 is about what the previous token was). Do not
build a two-sided lookahead.

**The concrete, predicted snapshot regression.** The lexer snapshot
`class_with_static_method` (`phalcom-ast/tests/lexer.rs:42-47`) tokenizes
`"class Point {\n  static origin { self }\n}"`. Under D3 the `\n` **after the first
`{`** is suppressed (previous token `LBrace`), so that snapshot loses exactly one
`Token::Newline`. The `\n` before the final `}` follows an inner `}` (`RBrace`, which
*can* end a statement) and is **preserved**. **This is the only existing snapshot
D3 changes** — bless it, and confirm via `cargo test -p phalcom-ast` that no other
snapshot diffs (an extra diff = a wrong predicate entry, most likely a missing
"can-end-statement" token).

**Parser-safety reasoning (why this is behavior-neutral on the running corpus).**
The suppressed newlines are, by construction, always *mid-expression / mid-body*
(after an operator or an opener), never at a statement boundary — a statement's
last token is always something that can end a statement, so its terminating newline
is never suppressed. `parse_top_item` (`parser.rs:345-374`) still sees the newline
after every real statement and after a class's closing `}`. Block/class bodies use
`skip_newlines` (`parser.rs:301-305`), which tolerates *zero* newlines, so removing
the newline right after `{` cannot break body parsing. Nonetheless **D3 is the item
most able to silently corrupt output**, so after landing it run the **full** golden
+ `lang` corpus and require every case byte-identical before committing.

### D4 — String interpolation `"{expr}"` + `\{` escape (spec §5) — **BLOCKED-ON-DECISION (open-Q5 / DEC-F)**

Spec §5 assumes `{expr}`, but **`open-questions.md` item 5 is still OPEN** ("`{name}`
is assumed; `${name}` and `\(name)` are alternatives" — confirmed unresolved at
HEAD). `PHASE2-INDEX.md §4 DEC-F` and §5 both record: **new ADR required before D4
builds.** Do **not** pick the sigil unilaterally.

- **Options:** (a) `{expr}` — spec §5 default, no sigil noise, but visually collides
  with block/map braces; `\{` escapes a literal brace. (b) `${expr}` — shell/JS
  familiar, unambiguous. (c) `\(expr)` — Swift.
- **Architect recommendation (unchanged from the plan / DEC-F): (a) `{expr}`.** The
  spec already commits to it and the staged pending fixture
  `pending/lexical_string_interpolation.ph` is written in the `{name}` form.
- **Gate:** if Q5 is unratified at dispatch time, **ship D1–D3 (+ the D5 coverage
  fixture) and leave D4 to a follow-up.** Do not implement past this gate.

Design (once Q5 is ratified — recorded here so the follow-up is turnkey):
- **Lexer:** on `"`, scan into ordered segments — literal runs and `{`-delimited
  expression runs. **Recommend a single `Token::StringInterp(Vec<StringSegment>)`
  token** (one lexeme, maximal-munch stays simple, spans stay local) over an
  `InterpStart`/`InterpExpr`/`InterpEnd` trilogy. `\{` → literal `{`, `\\` → literal
  `\`. Unterminated string / unterminated interpolation → `LexicalError` variants
  with precise spans. A plain string with no `{` still lexes to `Token::String`
  (do not regress the existing string path).
- **AST:** new node `StringInterp { parts: Vec<StringPart> }` where
  `StringPart = Literal(String) | Expr(Expr)`; each expr part is re-parsed from its
  source slice via the existing `offset` plumbing so spans stay absolute.
- **Desugar (spec §5):** `"{a} x {b}"` → `a.toString + " x " + b.toString`
  (`+`/`toString` sends already exist post-U5). Reuse existing `MethodCall`/`Binary`
  nodes — **no new evaluator.**
- **This is the only D-item touching `token.rs` (new token) and `ast.rs` (new node).**

---

## 3. Confirmed write-set (line numbers as of commit `4e2ec73` — re-grep before
editing; your own edits shift later lines)

| File | Exact change | D-item |
|---|---|---|
| `phalcom-ast/src/lexer.rs` | (D1) `skip_trivia` (`80-100`): add a `/* … */` arm; surface `UnterminatedBlockComment` (signature-change option a, or `pending_error` field option b — §2 D1). (D2) `scan_number` (`142-155`): accept interior `_`, strip before `parse`, reject bad placement via `InvalidToken`. (D3) `Lexer` struct (`41-55`): add `last_significant: Option<Token>`; `next()` (`298-318`): update it on non-`Newline` emits and suppress a following `Newline` per `suppresses_following_newline`; add that predicate `fn`. | D1,D2,D3 |
| `phalcom-ast/src/token.rs` | (D1) add `LexicalError::UnterminatedBlockComment(Range<usize>)` to the enum (`201-215`), full rustdoc. **No `?.`/`??` tokens (already exist, `151-168`).** (D4 only, when unblocked) add `Token::StringInterp(...)` + `StringSegment`. | D1 (D4) |
| `phalcom-ast/src/parser.rs` | (D1) add one arm to `lex_error_to_syntax` (`122-145`) mapping `UnterminatedBlockComment` → `SyntaxErrorKind::UnterminatedComment` with the offset-adjusted real span (mirror the `UnterminatedString` arm at `136-139`; do **not** use `0..0`). **No `parse_coalesce`/`parse_optional_send` changes (already landed).** (D4 only, when unblocked) parse `Token::StringInterp` into the `StringInterp` desugar. | D1 (D4) |
| `phalcom-ast/src/ast.rs` | **UNTOUCHED in the unblocked slice.** (D4 only, when unblocked) add `StringInterp { parts }` + `StringPart`. | (D4) |
| `phalcom-ast/src/error.rs` | **UNTOUCHED.** `SyntaxErrorKind::UnterminatedComment` already exists (`156-158`); D1 reuses it. (D4 only, if interpolation needs a distinct kind, add it here — else untouched.) | — (D4) |
| `phalcom-ast/tests/lexer.rs` | New snapshot tests: a `/* */` block comment (D1), `1_000_000` (D2), and a couple of newline-suppression cases (D3, e.g. `"a +\nb"` → no `Newline` between `+` and `b`; `"a\nb"` → `Newline` preserved). **Bless the one changed existing snapshot** `class_with_static_method` (§2 D3). | D1,D2,D3 |
| `phalcom-core/tests/lang/lexical/` | (D2) `git mv pending/lexical_numeric_separator.{ph,expected}` → active. (D1) new PASS fixture(s) for a block comment (there is none today — `comments_inline.ph` is `//`-only). (D3) new PASS fixture(s) for a multi-line continued expression that only parses because the newline is suppressed. (D5-coverage) new PASS fixture exercising `?.` and `??` end-to-end (§0 correction 1). (D4, when unblocked) `git mv pending/lexical_string_interpolation.*`. | all |

**Explicitly NOT touched:** `phalcom-ast/src/lexer.rs`'s `?.`/`??` arm (`270-274`,
done), `parser.rs`'s `parse_coalesce`/`parse_optional_send` (`877`/`1077`, done),
`token.rs`'s `Question`/`CoalesceQuestion`/`QuestionDot` (`151-168`, done),
`error.rs` (kind exists), `ast.rs` (D4-only), and anything in `phalcom-core/src`
or `core.ph` (out of your write-set — if forced there, STOP and report a conflict).

---

## 4. Build order (each step independently greppable + green-gate-verifiable)

1. **D1 block comments** → `skip_trivia` + `LexicalError` variant + `lex_error_to_syntax`
   arm; new lexer snapshot; new `lexical` PASS fixture; a negative fixture for the
   unterminated case if you want it under `syntax-errors` (optional). Verify green.
2. **D2 digit separators** → `scan_number`; new lexer snapshot; `git mv` the pending
   `lexical_numeric_separator` fixture. Verify green.
3. **D3 newline suppression** → `Lexer.last_significant` + predicate + `next()`
   suppression; new lexer snapshots; **bless `class_with_static_method`**; add the
   continuation PASS fixture. **Then run the FULL golden + `lang` corpus** — this is
   where `examples/person2.ph` / `person.ph` / `calculator.ph` and every `lang`
   golden are most at risk. Verify **byte-identical**.
4. **D5 coverage** → add the `?.`/`??` PASS fixture (no code). Verify green.
5. **D4 interpolation** — **only if open-Q5 / DEC-F is ratified.** Token + `StringInterp`
   node + desugar + `git mv` the pending fixture. Otherwise **stop at step 4** and
   hand D4 to a follow-up unit.

---

## 5. Test strategy — concrete fixtures

Corpus lives under `phalcom-core/tests/lang/lexical/` (PASS via the non-ignored
`lexical()` test) and `.../lexical/pending/` (via `#[ignore] lexical_pending()`).
Fixtures use the same `// area:` / `// spec:` / `// status:` header as the existing
cases; `System.print(...)` is the output primitive (see `comments_inline.ph`).

- **D1 block comment (PASS, new).** e.g. `System.print(/* inline */ 1)` → `1`, and a
  multi-line `/* line one\n line two */` before a `System.print(2)` → `2`. Proves
  block comments are trivia and a newline *inside* a block comment does not leak a
  `Token::Newline`.
- **D1 unterminated (NEGATIVE, optional but recommended).** `System.print(1) /* oops`
  under `tests/lang/syntax-errors/` (or `runtime-errors`), `.expected` = the
  substring `Unterminated comment` (matches `SyntaxErrorKind::UnterminatedComment`'s
  `Display`). Uses `check_negative`'s substring match.
- **D2 digit separators (PASS, promote).** `git mv pending/lexical_numeric_separator.*`
  → active (`System.print(1_000_000)` → `1000000`). Add a float case
  (`System.print(1_000.500_5)` → whatever `f64` prints) to prove `_` works on both
  sides of the `.` but not adjacent to it. Optionally a NEGATIVE case for `1__0` or
  `1_.0` asserting `Invalid token`.
- **D3 continuation (PASS, new) — the load-bearing case.** A program that only
  parses because the trailing-operator newline is suppressed, e.g.:
  ```phalcom
  let x = 1 +
          2 +
          3
  System.print(x)          // 6
  ```
  and an `and`/`or` variant, and a chained method call broken after `.` — wait, no:
  `.`-suppression is trailing, so `foo.\n bar` (dot at line end) is the suppressed
  form, **not** `foo\n.bar` (§2 D3, one-sided). Write the suppressible form.
- **D3 non-suppression guard (PASS, new).** Confirm `a\nb` still tokenizes as two
  statements (previous token `a`, an identifier, can end a statement) — a two-line
  program of two independent `System.print` calls that must both run.
- **D3 golden non-regression (assertion, not a new fixture).** After D3, the entire
  `phalcom-core/tests/golden.rs` set and every `lang` PASS label must stay
  byte-identical. This is the primary D3 acceptance gate — treat a single changed
  golden byte as a D3 failure, not a golden to re-bless.
- **D5 coverage (PASS, new).** Exercise the *already-landed* operators end-to-end so
  the surface is covered under your label, e.g. with `Some`/`None` from U6:
  `None ?? 5` → `5`; `Some.new(3) ?? 5` → `3` (or whatever `Some` prints);
  `Some.new(...)?.someMethod` short-circuiting to `None` on a `None` receiver.
  Check the exact `Some`/`None` construction/printing idiom in the existing
  `tests/lang/absence/*.ph` fixtures before writing this (do not invent surface
  syntax).
- **Lexer snapshots (`phalcom-ast/tests/lexer.rs`).** Add: a `/* */` case (D1), a
  `1_000_000` case (D2), a `"a +\nb"` case and a `"a\nb"` case (D3). Bless exactly
  one changed existing snapshot — `class_with_static_method` — and confirm no other
  snapshot diffs.

---

## 6. Mandatory rules (from `U-LEX-plan.md §8` — repeated for emphasis)

- **Docs.** Full rustdoc (`///`) on every new public item — the
  `LexicalError::UnterminatedBlockComment` variant, the `suppresses_following_newline`
  predicate and the `Lexer.last_significant` field, the new `lex_error_to_syntax`
  arm — with intra-doc links and `lexical-structure.md §`/ADR-0016 citations; `# Errors`
  on any new fallible fn (e.g. a `Result`-returning `skip_trivia`).
  `cargo doc --workspace --no-deps` adds **zero** new warnings.
- **Green gate.** `./scripts/verify.sh` exits 0 is your sole sign-off — **reviewer
  is OFF for U-LEX** (self-verify per `STATE.md` policy). Golden output
  byte-identical; the `lexical` label passes with your new cases; no new clippy
  warnings (fix pre-existing ones only in files you rewrite).
- **Preserve `@L`/`@R` span semantics** (ADR-0016): every new lexeme/node spans
  first-byte to last-byte so existing spans/diagnostics stay stable. Block-comment
  trivia carries no span into the token stream (it's skipped); the unterminated-comment
  *error* carries the real `open..pos` span.
- **No `?.`/`??`, binding (U6), block (U4), or `construct` (U7) parsing touched** —
  those are landed; confirm explicitly you left them alone.
- No `unsafe` expected. Run `graphify update . --no-cluster` before every commit;
  commit at each green checkpoint, never a non-compiling tree.
- **Hard stop when green:** do not begin U-STD/U11 or the deferred collection-literal
  unit. If Q5 is unresolved, D4 stops at its gate — do not guess the sigil.

---

## 7. Return contract (answer all of these — mirrors `U-LEX-plan.md §9`)

- Confirm (or report a deviation from) each of the four corrections in §0 — most
  importantly, confirm you **did not touch** `?.`/`??` and that D5's coverage is a
  new fixture only.
- Which of D1/D2/D3/(D4) shipped, and D4's open-Q5/DEC-F status at dispatch time.
- For **D1**: which `skip_trivia` error-surfacing option (a signature change vs a
  `pending_error` field) you chose, and the committed `lex_error_to_syntax` arm.
- For **D2**: how you reject a malformed separator (confirm you reused `InvalidToken`,
  no new `SyntaxErrorKind`), and whether you added a NEGATIVE fixture.
- For **D3** (load-bearing): the final `suppresses_following_newline` token set as
  committed, your `Colon` decision, **proof the newline machine kept every golden
  byte-identical** (`verify.sh` tail + the golden test names), and confirmation that
  the **only** blessed lexer snapshot was `class_with_static_method` (quote the
  one-line diff — one fewer `Token::Newline`).
- New/moved fixtures (list the `git mv`s out of `pending/` and the new PASS/NEGATIVE
  cases).
- Confirmation `error.rs` and `ast.rs` were **untouched** (unblocked slice), and that
  U6's `?.`/`??` code, U4 blocks, U7 `construct`, U6 bindings were untouched.
- Files changed, `verify.sh` tail, `cargo doc` tail.
- Any DEFERRED updates (fold into #6/#12/#20 — do not re-file); note if nested block
  comments or lone-`?` came up (both already #12).
