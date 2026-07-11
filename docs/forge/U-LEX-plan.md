# U-LEX — Work order: surface-syntax delta on the hand-written front end (dispatch-ready)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. **Reviewer OFF** —
self-verify on the green gate + `cargo doc`. Extends the landed U-FE front end (**ADR-0016**), not a
grammar file. Spec source of truth: [`lexical-structure.md`](../spec/lexical-structure.md). Runs in its
own worktree as a **Wave F** leaf, disjoint from U-STD (`core.ph`) and U8 (`phalcom-core/src`)._

---

## 0. Mission (one sentence)
Close the surface-syntax gap between the current hand-written lexer/parser (U-FE parity-with-old-grammar)
and [`lexical-structure.md`](../spec/lexical-structure.md) — block comments, digit separators, the
newline-suppression state machine, string interpolation, and the `?.`/`??` Option operators — **without
regressing any golden or `lang` corpus output**.

## 1. Hard guardrails (read before writing any code)
- **This EXTENDS ADR-0016's hand-written front end.** No parser generator, no grammar DSL, no `build.rs`.
  New syntax lands as targeted lexer lookahead + Pratt/precedence-table entries, per ADR-0016's own
  rationale ("context-sensitive constructs added with targeted lookahead").
- **Preserve `@L`/`@R` span semantics** exactly (ADR-0016): every new AST node spans first-token-start to
  last-token-end so existing AST/lexer snapshots and diagnostics stay stable.
- **Do NOT own these — they belong to other units already landed/scheduled before you:**
  - `var` keyword + `let`/`var` binding syntax → **U6** (ADR-0014). Do not touch binding parsing.
  - Block literal `{ x => … }` syntax + `{`-block-vs-map brace disambiguation → **U4** (blocks.md).
  - `construct` / field (`_foo`) declaration syntax → **U7** (classes.md).
  - Variadic `*`/spread parameter semantics (§8) → **U9** (Wave F+1). You add no spread parsing.
  - `throw`/`try`/`catch`/`finally` keywords (§10) → the error-handling unit (ADR-0008). **Do not add
    dead keyword tokens** with no consumer; defer (see §6 Deferred).
- **Collection literals `(a,b)` / `[…]` / `{a:1}` are OUT of scope** (see §4 Design + §6 Deferred): their
  lowering target has no ADR and their brace form entangles with U4. Deferred to a dedicated unit.
- Stay inside the write-set (§3). If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md). Do not self-approve merges.

## 2. Preconditions (verify first; do not assume)
- Runs in an **isolated worktree off `main`** seeded from the committed green base (U4, U6, U7 landed;
  their parser/lexer edits are present). Confirm `./scripts/verify.sh` is green before the first edit.
- **graphify-first:** re-run `graphify explain "Lexer"`, `graphify explain "Parser"`, and
  `graphify affected "Token"` on the actual HEAD to confirm the write-set below still matches reality and
  that U4/U6/U7 have not already introduced tokens you were about to add.
- Confirm the `lexical` `lang` label passes and note the current `blocks`/`bindings` pending-dir layout —
  your new cases live under `phalcom-core/tests/lang/lexical/`.

## 3. Confirmed write-set (tight; disjoint from U-STD and U8)
| File | Why it's in scope |
|---|---|
| `phalcom-ast/src/lexer.rs` | Block comments, digit separators, newline-suppression state machine, `?.`/`??` maximal-munch, interpolation scanning. |
| `phalcom-ast/src/token.rs` | New tokens: `QuestionDot`, `QuestionQuestion`, and interpolation-segment tokens (see §4). New `LexicalError` variants. |
| `phalcom-ast/src/ast.rs` | New nodes: interpolated-string node; `?.`/`??` lower into existing send/`MethodCallExpr` nodes where possible (prefer no new node). |
| `phalcom-ast/src/parser.rs` | Precedence-table entries for `?.`/`??`; interpolation → `toString`+concat desugar; consume suppressed-newline behaviour correctly. |
| `phalcom-ast/src/error.rs` | `Display`/kinds for the new `LexicalError` variants (unterminated block comment, unterminated/empty interpolation). |
| `phalcom-ast/tests/lexer.rs` | Lexer snapshot cases for every new lexeme. |
| `phalcom-core/tests/lang/lexical/*` | `.ph`/`.expected` corpus cases (data only — disjoint from U-STD's `classes/…` and U8's Rust). |

## 4. Design decisions (grounded in `lexical-structure.md` + ADR-0016 / ADR-0007)

### D1 — Block comments `/* */` (§2)
Extend `skip_trivia` to consume `/* … */` as trivia. **Non-nesting** (spec §2 shows a flat block comment
and is silent on nesting; non-nesting keeps the scanner a simple state machine and matches C/Java surface
expectation for the shown form). Unterminated block comment → new `LexicalError::UnterminatedBlockComment(span)`.
_Forward-looking:_ if nested comments are later wanted, the change is local to `skip_trivia` — note it in
DEFERRED, do not pre-build it.

### D2 — Digit separators `1_000_000` (§4)
In `scan_number`, accept `_` **between** digits (never leading, never trailing, never doubled, never
adjacent to the `.`). Strip `_` before `parse::<f64>()`. Reject a trailing/`.`-adjacent `_` as a lexical
error rather than silently. Preserve the existing `1..2` / `3.method` decimal-point rule unchanged.

### D3 — Newline-suppression state machine (§1) — **the load-bearing item**
Spec §1 is explicit and prescriptive: newline suppression is **a lexer-level state machine, NOT parser
ASI**. Implement in the lexer: track the last *significant* emitted token; when about to emit
`Token::Newline`, **suppress it** (skip, emit nothing) if the previous significant token *cannot end a
statement* — i.e. a binary operator (`+ - * / % == != < <= > >= and or`), `,`, `(`, `{`, `[`, `=>`, `->`,
`.`, `::`, `?.`, `??`, `=`, and the compound-assign ops. Otherwise emit the `Newline` as today.
- **Data structure:** add a `last_significant: Option<Token>` field on `Lexer` (updated only on non-trivia,
  non-newline emits) + a `fn suppresses_following_newline(tok: &Token) -> bool` predicate table.
- **Why lexer not parser:** ADR-0016 + spec §1 both forbid parser lookahead ASI ("that is how JavaScript
  acquired the `return\n{}` bug"). Keep the parser's newline-as-terminator logic untouched; it simply sees
  fewer newlines.

### D4 — String interpolation `"{expr}"` + `\{` escape (§5) — **BLOCKED-ON-DECISION (open-Q5)**
Spec §5 assumes `{expr}`, but [open-questions.md Q5](../spec/open-questions.md) is **OPEN** (`{}` vs
`${}` vs `\(…)`). See §7. Design (syntax-agnostic once Q5 is ratified):
- **Lexer:** on `"`, scan into segments — literal runs and `{`-delimited expression runs. Emit either
  (a) a single `Token::StringInterp` carrying an ordered `Vec` of segments (literal text | raw expr source
  slice), or (b) a token trilogy (`InterpStart`/`InterpExpr`/`InterpEnd`). **Recommend (a)** — the whole
  string is one lexeme, keeping maximal-munch simple and spans local; the parser re-parses each expr slice
  via `parse_source` at the segment offset (spans stay absolute via the existing `offset` plumbing).
- `\{` is a literal brace; `\\` a literal backslash. Unterminated interpolation / unterminated string →
  `LexicalError` variants with precise spans.
- **Parser/desugar:** an interpolated string lowers to `seg0.toString + seg1.toString + …` (§5: "each
  `{expr}` desugars to a `toString` send and string concatenation"). New AST node `StringInterp { parts }`
  where each part is `Literal(String) | Expr(Expr)`; the compiler already has `+`/`toString` sends.
- **Do not implement past the Q5 gate** — if Q5 is unratified at dispatch time, ship D1–D3 + D5 and leave
  D4 as a follow-up rather than guessing the sigil.

### D5 — `?.` and `??` Option operators (§9; ADR-0007) — depends on U4 + U6
- **Lexer:** two new maximal-munch tokens `QuestionDot` (`?.`) and `QuestionQuestion` (`??`). A lone `?`
  stays reserved (keep the existing `Token::Question`, unused by the grammar — reserve for future ternary).
- **Precedence/associativity (spec §9, authoritative — do not re-derive):**
  - `?.` sits at **member-access precedence (same as `.`), left-associative**: `a?.b?.c` ≡ `(a?.b)?.c`.
  - `??` is **low-precedence binary, right-associative**, looser than comparison/arithmetic but tighter
    than assignment: `a ?? b ?? c` ≡ `a ?? (b ?? c)`.
- **Desugar (spec §9 / values-and-absence §3.4):** `a ?? b` → `a.orElse { b }`; `opt?.m(x)` →
  `opt.map { it => it.m(x) }`. Both lower to existing `MethodCallExpr` + a block literal — **reuse U4's
  block node**, do not invent one. Short-circuit falls out of `orElse`/`map` on `Option` (U6).
- **Depends on U4 (block literal AST) + U6 (`Option`/`orElse`/`map`)** both being present in the base.

## 5. Build order (each step independently greppable/testable)
1. **D1 block comments** → lexer + `lexer.rs` snapshot. Verify green.
2. **D2 digit separators** → lexer + snapshot + a `lexical` corpus case. Verify green.
3. **D3 newline suppression** → lexer field + predicate + snapshots; **then run the full golden + `lang`
   corpus** — this is where multi-line chained sends in `person2.ph`/`core_new.ph` are most at risk. Verify green.
4. **D5 `?.`/`??`** → tokens, precedence-table entries, desugar; `lexical` corpus cases exercising
   short-circuit. Verify green.
5. **D4 interpolation** — **only if open-Q5 is ratified** (§7). Tokens + `StringInterp` node + desugar +
   corpus. Otherwise stop at step 4 and hand D4 to a follow-up.

## 6. Deferred (append to `DEFERRED.md`, do not build here)
- **Collection literals** `(a,b)` tuple / `[…]` list / `{a:1}` map — lowering target unspecified (no ADR;
  entangles with U4 brace disambiguation §6). Needs a NEW ADR (collection-literal lowering) + likely its
  own unit. **Flag for the `documentation-and-adrs` skill.**
- **`throw`/`try`/`catch`/`finally`** keyword tokens (§10) — land with the error-handling unit (ADR-0008),
  not as dead tokens here.
- **Nested block comments** — local `skip_trivia` change if ever wanted.
- **Lone `?` ternary/try-operator** — token reserved, no grammar yet.

## 7. BLOCKED-ON-DECISION
- **open-Q5 — string interpolation sigil.** Spec §5 assumes `{expr}`; Q5 lists `${expr}` and `\(expr)` as
  live alternatives and is **unresolved**. Options: **(a)** `{expr}` (spec §5 default, no sigil noise, but
  collides visually with block/map braces), **(b)** `${expr}` (shell/JS-familiar, unambiguous), **(c)**
  `\(expr)` (Swift). **Recommendation: (a) `{expr}`** — the spec already commits to it and `\{` escapes the
  literal brace; ratify Q5 = `{}` (a one-paragraph ADR via the `documentation-and-adrs` skill) then
  implement D4. **Do not pick unilaterally.** D1–D3 + D5 are unblocked and ship regardless.

## 8. Mandatory rules
- **Docs** ([guidelines](../rust-documentation-guidelines.md)): `///` on every new public item (tokens,
  error variants, AST nodes, lexer/parser methods) with intra-doc links and `lexical-structure.md` §/ADR
  citations; `# Errors` on new fallible fns. `cargo doc --workspace --no-deps` adds **no new warnings**.
- **Green gate:** `./scripts/verify.sh` exits 0. Golden output byte-identical; the `lexical` `lang` label
  passes with your new cases; no new clippy warnings (fix pre-existing ones in files you rewrite).
- **graphify update** `.` `--no-cluster` after edits.

## 9. Return contract (self-verify; reviewer OFF)
Report: which of D1–D5 shipped (and D4's Q5 status) · new tokens/AST nodes + precedence entries added ·
proof the newline-suppression machine kept every golden byte-identical (`verify.sh` tail) · `cargo doc`
tail · any DEFERRED entries filed. Explicitly confirm you did **not** touch binding (U6), block (U4), or
`construct` (U7) parsing.
