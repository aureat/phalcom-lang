# U-NEG — unify boolean negation on the `not` keyword

Status: **PLANNED** (dispatch-ready). Prerequisite/sibling of **U-IS**. Contends
`phalcom-ast` (parser) + `phalcom-core/core/core.ph` — single-writer, **serialize with U-IS**
(both touch parser.rs and core.ph). Fire U-NEG **before** U-IS.

## Role
Make `not` the single boolean-negation surface, per [is-tests.md](../../../spec/v0.2/is-tests.md)
§"Negation surface" and its "precludes prefix `!`". Two mechanical changes + one migration:
1. Wire the already-reserved `Token::Not` as a prefix unary operator (`not x`).
2. Retire prefix `!` (`Token::Bang`) as an expression operator. `!=` (`BangEqual`) is untouched.
3. Migrate existing `!x` sites in `core.ph` to `not x`.

## Spec anchor — [is-tests.md](../../../spec/v0.2/is-tests.md) (Status: Proposed; Fork A ratified in-note)
> "There is **no prefix `!`**. General boolean negation is `not x`; inequality stays `!=`."
> "Prefix `!` … Retired in favour of `not x`. `!=` survives as its own token."

Also [values-and-absence.md](../../../spec/v0.2/values-and-absence.md) (`Bool#not`, the `.not`
lowering target — unchanged; only the *surface trigger* moves from `!` to `not`).

## Current state (verified)
- `Token::Not` exists (token.rs:79) and lexes (`"not" => Token::Not`, lexer.rs:283) but is
  **not** consumed by `parse_unary` — a reserved-but-dead keyword today.
- Prefix `!` is **live**: `parse_unary` maps `Token::Bang => UnaryOp::Not` (parser.rs:1435),
  lowering to the `not` send (compiler lib.rs:2152, `UnaryOp::Not => "not"`).
- `Token::BangEqual` (`!=`) is a distinct two-char token (token.rs:153, lexer.rs:594) — **do
  not touch**; it is not prefix `!`.
- `core.ph` uses prefix `!` at **5 sites**: `return !(self == other)` at lines 411, 510, 570,
  619, 717 (the `!=`-derived-over-floor pattern). Comments at 383–384 also *name* `!` as a
  floor primitive — update that prose too.

## Changes
- **parser.rs `parse_unary` (~1432):** add `Token::Not => UnaryOp::Not`; **remove**
  `Token::Bang => UnaryOp::Not`. After removal, a bare `!` in expression position is a parse
  error (only `!=` remains a valid `!`-containing token). Confirm no other parser site treats
  `Token::Bang` as a prefix (grep; parser.rs:842 lists `Token::Bang` in a set — check whether
  that set is a valid-expression-start set that must drop `Bang` and gain `Not`).
- **Lowering unchanged:** `UnaryOp::Not` already lowers to the `not` send — no compiler edit.
- **core.ph migration:** rewrite the 5 `!(self == other)` → `not (self == other)`; update the
  383–384 comment naming `!` to name `not`.

## Write-set (STOP-and-report if outside)
- `phalcom-ast/src/parser.rs` — `parse_unary` arm swap; expression-start set (parser.rs:842) if it
  gates `Bang`.
- `phalcom-ast/src/token.rs` — doc-comment on `Token::Bang` (214, "logical-not / prefix operator")
  now stale: `Bang` survives only inside `BangEqual` lexing; reword or note. **Do not delete the
  `Bang` variant** — `scan_operator` (lexer.rs:594–595) still needs it to disambiguate `!=`.
- `phalcom-core/core/core.ph` — the 5 `!` sites + the 383–384 comment.
- `phalcom-core/tests/` — goldens.
- `docs/spec/v0.2/is-tests.md` — strike the implementation-note caveat "not yet wired /
  presently spelled `!`" once landed.
- **Floor: +0** (pure parse-surface swap; `Bool#not` already exists).

## Lexer note (decide during impl)
Prefix `!` no longer forms an expression, but `!=` must still lex. Keep `scan_operator`'s
`b'!' if next == Some(b'=')` → `BangEqual` **and** the fallthrough `b'!' => Bang`; the parser is
now the layer that rejects a standalone `Bang`. (Alternative: make the lexer emit an error on a
non-`=`-followed `!`. **Prefer the parser-reject path** — keeps `!` diagnosable and leaves the
door open if a future `!`-operator is wanted; note the choice in the return shape.)

## Tests / graduation
- **Positive goldens** (stdout byte-exact): `not true` → `false`; `not (1 == 2)` → `true`;
  the migrated `!=` paths (`3 != 4`, list/map/set/tuple `!=`) still print identically.
- **Negative lane** (`check_negative`): bare `!x` is now a parse error; `x !` is a parse error.
  `x != y` must **still parse** (regression guard that the retirement didn't nick `!=`).
- WORKTREE-VERIFY the batch before commit ([[phalcom-golden-test-lanes]]); `cargo test` must be
  green **because** the core.ph migration is complete (a missed `!` site fails to compile core).

## Reviewer
ON — touches the global negation surface + core bootstrap. Independent phalcom-reviewer;
writer ≠ approver. Reviewer must confirm: `!=` intact, all 5 core.ph sites migrated, no orphan
`Bang`-as-prefix path.

## Return shape (implementer)
commit SHA(s) · `parse_unary` arm swap · parser.rs:842 set disposition · lexer `!`/`!=` handling
choice · core.ph 5-site migration list · `Bang` variant retained for `!=` · goldens (mark negative,
incl. `!=` regression) · floor delta (exp 0) · verify tail · write-set confirm.

## Follow-on
- **U-IS** — the `is`/`is!`/`is not` operators; consumes this unit's unified `not` (the `is not`
  particle disambiguation is only *needed* once `not` is a prefix operator, which lands here).
