# U-LEX-HASH — `#` symbol literals

Status: **PLANNED** (dispatch-ready). Prerequisite unit surfaced by the U16 blocker
adjudication (see [STATE.md](../../STATE.md) U16 section). Fire on the serial spine tail
**after U16-Open is accepted** — contends `phalcom-ast`, single-writer.

## Role
Lex `#`-prefixed symbol literals so the language has first-class **name symbols** and
**selector symbols**. Unblocks:
- **U16-Pinned** (`obj::#sel` pinned method references — the deferred half of U16).
- The `#IDENT` map-symbol-key hole ([DEFERRED.md](../../DEFERRED.md) — `m.at(#a)`), graduating
  `tests/lang/collections/pending/literal_map_symbol_keys.ph`.
- Future `perform` / reflection selector symbols.

## Spec anchor — [selectors.md §2](../../../spec/current/selectors.md) (AUTHORITATIVE, fully specified)
Two value types, both backed by an interned `Symbol`:

| Literal | Type | Meaning |
| --- | --- | --- |
| `#move` | **Name symbol** | bare method name; identifies a *family* (map keys, `respondsTo`, reflection) |
| `#move(_,to,duration)`, `#+`, `#==`, `#[]` | **Selector symbol** | complete method identity (`perform`, pinned refs) |

`perform` accepting only selector symbols is **not** this unit (no `perform` surface yet) —
U-LEX-HASH only lexes + interns both forms.

Also grounds: [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md)
(canonical selector string is the intern key — reuse `encode_selector`/interner, do NOT fork
a parallel canonicalizer); selectors §1 R2 (positionals precede labels — reject interior
positionals `#move(to,_)`).

## Lexing (selectors §2 rules, verbatim)
- **Single atomic Logos token.** Spec-given regex:
  `#[a-zA-Z_][a-zA-Z0-9_]*(\([^)]*\))?` + a separate branch for operator selectors (`#+`, `#==`, `#[]`).
- **Adjacency OUTSIDE parens required** (ASI-hazard guard): `#`, name, `(` contiguous.
  `# move` → not a symbol; `#move (a,b)` → `#move` name-symbol **then** a parenthesized expr.
- **Whitespace INSIDE parens is free** (stripped at canonicalization).
- **Shebang carve-out:** `#!` at byte-offset 0 is a shebang line, NOT a symbol. Verify existing
  lexer shebang handling (graphify `explain` the lexer entry); ensure the `#` rule cannot swallow a
  leading `#!…` line. Add an offset-0 `#!` skip before the symbol rule if absent.

## Value / AST
- **`Value::Symbol(Symbol)` already exists** (value.rs:41) — **no new Value arm, no heap variant.**
- AST: add a symbol-literal expression node (or reuse the existing literal family — check `ast.rs`).
  Name-symbol and selector-symbol both lower to a `Value::Symbol` constant; only the interned
  string differs (`"move"` vs canonical `"move(_,to,duration)"`).
- Compiler: emit as a `Constant` (same path as other literal constants); intern the
  (canonicalized) string via `interner.rs` at compile time.

## Canonicalization
- Name symbol: intern the bare name as-is.
- Selector symbol: strip inner whitespace, validate R2 (no interior positionals), intern the
  canonical form — MUST equal what a same-signature method definition interns, so `#move(_,to)`
  is the *same* `Symbol` as the selector a `move(_,to:)` method registers. **Reuse the existing
  selector-encoding routine (ADR-0012); do not fork it.**

## Coupled bug-fix — Symbol#== (MUST ride with this unit)
[DEFERRED.md](../../DEFERRED.md) `value.rs` `value_eq` (~L253): no `(Value::Symbol, Value::Symbol)`
arm → two independently-interned-but-identical symbols never compare `==`. Without the fix,
`m.at(#a)` re-compares an independently-interned key and returns `None` — so the fixture this unit
graduates (`literal_map_symbol_keys.ph`) still fails on retrieval **even after `#a` lexes**.
**Fold the 1-line fix `(Value::Symbol(a), Value::Symbol(b)) => a == b` INTO this unit** (value.rs is
then in the write-set for exactly that arm) and note it in the commit. Do not graduate the fixture on
lexing alone.

## Write-set (owns lexer+token; STOP-and-report if outside)
- `phalcom-ast/src/{lexer.rs, token.rs}` — **OWNS** (the reason this is its own unit)
- `phalcom-ast/src/{ast.rs (symbol-literal node), parser.rs (token → node)}`
- `phalcom-core/src/compiler/lib.rs` (emit Symbol constant + canonicalize/intern)
- `phalcom-core/src/value.rs` — **the Symbol#== arm ONLY** (coupled fix above)
- `phalcom-core/tests/` (goldens + graduate the pending fixture)
- `docs/spec/current/selectors.md` §2 (mark IMPLEMENTED); `docs/forge/DEFERRED.md` (strike the two
  resolved entries: `#IDENT` lexer hole + Symbol#==)
- **Floor: expect +0** (pure lex/parse/compile + existing `Value::Symbol` + a 1-line eq arm; no new
  native binding). If a primitive turns out needed → STOP-and-report per ADR-0019.

## Tests / graduation
- **Positive goldens** (stdout byte-exact): `#name` prints/compares; name-symbol as map key
  round-trips; a selector symbol interns to the same identity as the method it names.
- **Negative lane** (`check_negative`): `#move(to,_)` interior-positional → validation error;
  `# move` adjacency case.
- **Graduate** `tests/lang/collections/pending/literal_map_symbol_keys.ph` (`m.at(#a)`) — depends on
  BOTH the `#a` token and the Symbol#== fix (both in this unit).
- WORKTREE-VERIFY the whole batch before commit ([[phalcom-golden-test-lanes]]).

## Reviewer
ON — spine-adjacent (lexer + selector-identity + a `value_eq` correctness arm). Independent
phalcom-reviewer pass; writer ≠ approver.

## Return shape (implementer)
commit SHA(s) · token regex + operator branch · shebang carve-out handling · canonicalization reuse
of `encode_selector` · Symbol#== disposition · goldens (mark negative) · fixture graduation · floor
delta (exp 0) · verify.sh + cargo doc tails · write-set confirm.

## Follow-on
- **U16-Pinned** — adds the `obj::#sel` pinned form to U16 once selector symbols lex.
- Set/range literals (`#{…}`, `..`/`...`) remain separate future lexer units (DEFERRED) — NOT this unit.
