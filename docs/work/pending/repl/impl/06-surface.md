# §06 — Stage 5: completion, hints, highlighting, latency (§S3–§S7)

**Phase B — branch-only.** `phalcom-repl/**` only. The largest stage, and the one the
user actually sees.

Depends on [§05](05-oracle-and-selectors.md): every layer here reads the snapshot.

## 1. §S5 — highlighting in three layers

| layer | source | when | degrades to |
|---|---|---|---|
| **L1 lexical** | `phalcom_ast::lexer` token stream | every keystroke, synchronous | — (always available) |
| **L2 syntactic** | `phalcom-lsp` `semantic_tokens` AST pass | debounced (§S6) | L1 |
| **L3 live** | §S1 snapshot | debounced (§S6) | L2 |

**Every layer only ever *refines* the one below.** A layer that cannot answer leaves the
lower layer's coloring untouched. **None may remove or downgrade a token.** This is the
invariant that keeps a half-typed line from flickering.

### 1.1 L1 replaces the regex battery outright

Delete from `phalcom-repl/src/highlighter.rs`:

```
:15  RE_STRING          :16  RE_LINE_COMMENT    :17  RE_BLOCK_COMMENT
:18  RE_KEYWORD         :22  RE_BOOL_NIL        :23  RE_NUMBER
:24  RE_IDENT
```

…and with them the string/comment mask hack, the `has_non_ident_boundaries` fixups, and
every false keyword match inside a string literal. The lexer already knows exactly where
those boundaries are; the regexes were approximating a tokenizer that exists.

Once the battery is gone, check whether `regex` and `once_cell` are still used anywhere
in `phalcom-repl`. If not, drop them from `Cargo.toml` — §S8 already removed
`rustyline`, `fancy-regex`, and `anyhow` on the same principle.

### 1.2 L2 is reused, not reimplemented

`phalcom-lsp/src/semantic_tokens.rs` — `tokens_for(text, line_index)` at `:340`, legend
at `:139`. 672 lines: a flat pass plus AST-assisted declaration-name refinement. Its
parser recovers from syntax errors, so it still produces tokens for a half-typed line.
Call it. Do not port it.

### 1.3 L3 is the REPL-only payoff

An identifier absent from the snapshot's `globals`, from module globals, and from
enclosing locals is **unbound** and renders dimmed. That is a pre-flight
`doesNotUnderstand`. No editor can do this, because no editor knows what has *run*.

It is also the cheapest query in the system — three hash lookups.

## 2. §S3 — ranking

Candidates sort by `(own_depth, name)`: a receiver's own members first, then each
superclass in chain order, alphabetical within each tier. `List`'s `add` precedes
`Iterable`'s `map`, which precedes `Object`'s `toString`.

Depth comes free from [§05 §1.1](05-oracle-and-selectors.md).

**Kind filtering reuses the LSP's existing `ReceiverKind` split**
(`phalcom-lsp/src/completion.rs:44`; entry point `completions()` at `:376`). An instance
receiver is offered instance-side members only; a class receiver gets `static` /
`construct` only. That distinction already exists and **must not be rebuilt**.

**Rejected — frequency/recency weighting.** Better ergonomics, but requires usage data
that does not exist. Revisit once the REPL has history worth mining.

> Superclass chains are not encoded in `core-table.json`
> (`docs/deferred/core-table-inheritance.md`). The REPL is **not** blocked: the live
> snapshot walks real chains from the VM. That gap constrains the editor, not this unit.

### 2.1 Delete `guess_type_from_name`

`phalcom-repl/src/completer.rs:219`, called at `:197` with the comment
`// heuristic; see below`. It infers `List` from a trailing `s`. The snapshot knows the
real class. Delete both the function and its call site.

`completer.rs` is rewritten wholesale in this stage — `Completer` impl at `:51`,
`Hinter` impl at `:76`.

## 3. §S4 — hints

**Ghost text.** The top-ranked §S3 candidate renders dim inline, accepted with Tab.

The existing `Hinter` is **miswired** and is rewritten, not extended: it returns
`line + hint` from `handle` and a literal `"\t"` from `complete_hint`
(`completer.rs:76`, `:86`).

**Value echo.** After a cell evaluates, its result renders as `// => <value>`.

> **Hazard.** Rendering a value sends `toString`, which is user-overridable and may raise
> or loop. This runs *after* evaluation, so executing user code is already expected — but
> the echo **must be error-guarded and must not turn a successful cell into a failed
> one.** A raising `toString` degrades to the class name. Never to a failed cell.

**Signature hints are deferred.** Showing `at(index)` after `foo.at(` needs parameter
*names*; selectors carry *labels*. `at(_)` has no name to show, and ADR-0012's encoding
never records one. Delivering it means threading parameter names from `MethodObject`
through to the snapshot — a real dig, not a polish pass. Out of scope.

## 4. §S7 — completion inserts a call opening (DEC-REPL-B, RULED)

Accepting a completion inserts `name(` with the cursor between the parens. **Arity-0
selectors insert the bare name**, no parens, no cursor move — §D9's structured selectors
carry arity, so this is a read, not a guess.

`phalcom-lsp` emits LSP snippet-format items (`at(${1:index})`). Inserting that text
verbatim is the one unacceptable option — it puts literal `${1:...}` in the buffer. So
**strip the placeholders**.

Do not build a tab-stop engine. It needs a placeholder state machine owned by the line
editor, a rule for what Tab means when stops are pending versus when ghost text is
pending (§S4 already binds Tab), and an escape path for nested and abandoned stops. Its
own unit. The degraded form is not a placeholder for a real one — `name(` with the cursor
inside is what most consoles do and is correct on its own terms.

## 5. §S6 — latency

The governing preference, stated by the user and binding on this design:

> not laggy; it's fast, but it's okay if the REPL takes some time intentionally before
> giving suggestions

**Hard constraint — the input loop never blocks.** Keystroke echo and L1 highlighting are
synchronous and `O(line length)`. A REPL line is short; a relex is microseconds. Nothing
in L2, L3, completion, or the snapshot may sit on this path.

**Soft budget — deferred beats eager.** L2, L3, and hint computation run **debounced on
idle**. Start at ~100 ms and tune **by feel, not by benchmark**.

Consequences, all three binding:

- **Debounce is a feature, not a fallback.** Do not "optimize" it away later. A future
  reader will see a deliberate delay and want to remove it; this paragraph is why they
  should not.
- **No caching layer is specced.** This repo measures first (`docs/forge/perf-log/`), and
  a debounced recompute on a short line is very likely already under budget. Add caching
  when a measurement demands it.
- **The snapshot is built once per cell**, off the keystroke path entirely, so its cost
  is invisible regardless of size.

## 6. Tests

Most of this stage is terminal behavior and resists automated testing. Test what is pure;
check the rest by hand and say so.

| Test | Asserts |
|---|---|
| `ranking_puts_own_before_inherited` | `(own_depth, name)` order on a real chain |
| `arity_zero_inserts_bare_name` | §S7 — no parens for arity 0 |
| `arity_n_inserts_call_opening` | §S7 — `name(`, cursor inside, no `${1:}` |
| `l1_never_keywords_inside_strings` | the regex-battery bug L1 fixes |
| `layers_only_refine` | L2/L3 absent leaves L1's tokens intact |
| `value_echo_survives_raising_tostring` | **the §S4 hazard** — degrades to class name, cell still succeeds |

`value_echo_survives_raising_tostring` is the one with teeth: define a class whose
`toString` raises, bind an instance, evaluate it as a cell, and assert the cell reports
**success** with a degraded rendering.

## 7. Write-set

| Path | Change |
|---|---|
| `phalcom-repl/src/highlighter.rs` | rewritten — three layers; regex battery deleted |
| `phalcom-repl/src/completer.rs` | rewritten — snapshot-backed; `guess_type_from_name` deleted |
| `phalcom-repl/src/main.rs` | debounce wiring; Tab binding |
| `phalcom-repl/src/common.rs` | `KEYWORDS` — see below |
| `phalcom-repl/Cargo.toml` | drop `regex` / `once_cell` if now unused |

`common.rs`'s `KEYWORDS` table exists to seed the highlighter's keyword regex. Once L1
reads the lexer's token stream, the table has **one** remaining consumer (completion) and
possibly none — the lexer is the authority on what a keyword is. Check, and delete it if
it is dead. It was already a drift risk: `const` was missing from it until `ebc0a63`.

**Conflict risk vs class work: none.**

## 8. Gate

Workspace green. Then a real session, because nothing above proves the thing works:

```
cargo run -p phalcom-repl
```

- type `let xs = [1, 2, 3]`, then `xs.` — own members before inherited, no `Initializer`
- type `undefinedThing` — renders dimmed (L3)
- type `"class"` — the word inside the string is **not** keyword-colored (the L1 fix)
- accept an arity-0 completion — bare name, no parens
- confirm suggestions arrive after a short, deliberate pause and typing never stutters
