# U-REPL — completion, hints, and highlighting surface

Companion to [`err-plan.md`](plan.md), at the same grain. `err-plan.md` specs the evaluation
substrate (§D1–§D10); this specs what the user actually sees. Decisions here are
lettered §S1–§S9 to stay distinct from the plan's §D-series. Nothing is open.

Prerequisite: §D8 (live oracle from the start) and §D9 (structured selectors).

## §S1 — Oracle access is a snapshot, never a live borrow

The completer and highlighter query an **immutable snapshot**, rebuilt once per cell
boundary — after `unwind_to(0, 0)` (§D10), before the next prompt renders.

```
ReplSnapshot {
    globals:   HashMap<Symbol, ClassId>,     // name -> class of its current value
    members:   HashMap<ClassId, Vec<Member>>,// full chain, walked once, own-depth tagged
    classes:   HashMap<Symbol, ClassId>,     // class-valued globals, for class-side receivers
}
```

Rejected — **live borrow of the `Universe`**: exact, but a keystroke query borrows a
VM that is not guaranteed at rest once a cell can leave a fiber suspended (§D10
preserves suspended fibers deliberately). Rejected — **reflective self-hosting**
(`x.class.methods` as real sends): elegant and self-describing, but executes
user-reachable code on a keystroke. That is Node's getter-side-effect trap; DevTools
needed V8's `throwOnSideEffect` to make console completion safe. A snapshot has no
such surface.

Snapshot staleness is bounded by construction: nothing executes between cells, so a
snapshot taken at the boundary is exact for the whole editing session that follows it.

## §S2 — The static layer sees the current line only

`phalcom-lsp` is fed **only the line being typed**, never an accumulated buffer of
prior cells.

Rationale is PDR-0001 ruling 6. Cells *shadow*: a later `class Foo` binds a new
class. Replaying all cells as one synthetic document would therefore present two
`class Foo` declarations to the index — and under PDR-0002 that is
`class.already_defined`, a hard error. Reconstructing shadowing inside the static
layer would reimplement cell semantics in a second place, with a second chance to get
it wrong.

Division of labour: **static answers syntax for text that has not run; live answers
everything that has.** They do not overlap, so the §D8 merge rule ("live wins for
names that exist at runtime") reduces to "no conflict is reachable."

## §S3 — Ranking: own members before inherited

Completion candidates sort by `(own_depth, name)` — a receiver's own members first,
then each superclass in chain order, alphabetical within each tier. So `List`'s `add`
precedes `Iterable`'s `map`, which precedes `Object`'s `toString`.

Depth is free: §S1's snapshot already walks the chain to build `members`, so it tags
depth on the way.

Kind filtering reuses the LSP's existing `ReceiverKind` split — an instance receiver
is offered instance-side members only; a class receiver gets `static`/`construct`
only. That distinction already exists in `phalcom-lsp/src/completion.rs` and must not
be rebuilt.

Rejected — **frequency/recency weighting**: better ergonomics, but requires usage data
that does not exist yet. Revisit once the REPL has history worth mining.

Note this needs superclass chains, which `core-table.json` does not encode
(`docs/deferred/core-table-inheritance.md`). The REPL is **not** blocked by that: the
live snapshot walks real chains from the VM. That gap constrains the editor, not this.

## §S4 — Hints: ghost text and value echo; signature hints deferred

**Ghost text.** The top-ranked §S3 candidate renders dim inline, accepted with Tab.
The existing `Hinter` impl is currently miswired — it returns `line + hint` from
`handle` and a literal `"\t"` from `complete_hint` — and is rewritten, not extended.

**Value echo.** After a cell evaluates, its result renders as `// => <value>`.

> Hazard: rendering a value sends `toString`, which is user-overridable and may raise
> or loop. This runs *after* evaluation, so executing user code is already expected —
> but the echo must be error-guarded and must not turn a successful cell into a failed
> one. A raising `toString` degrades to the class name, never to a failed cell.

**Signature hints are deferred.** Showing `at(index)` after `foo.at(` needs parameter
*names*; selectors carry *labels*. `at(_)` has no name to show, and ADR-0012's
encoding never records one. Delivering this means threading parameter names from
`MethodObject` through to the snapshot — a real dig, not a polish pass, and out of
scope here.

## §S5 — Highlighting in three layers

| layer | source | when | degrades to |
|---|---|---|---|
| **L1 lexical** | `phalcom_ast::lexer` token stream | every keystroke, synchronous | — (always available) |
| **L2 syntactic** | `phalcom-lsp` `semantic_tokens` AST pass | debounced (§S6) | L1 |
| **L3 live** | §S1 snapshot | debounced (§S6) | L2 |

**L1** replaces the regex highlighter outright. It removes the string/comment mask
hack, the `has_non_ident_boundaries` fixups, and every false keyword match inside a
string literal — the lexer already knows exactly where those boundaries are.

**L2** is reused from `phalcom-lsp/src/semantic_tokens.rs` (672 lines, flat pass plus
AST-assisted declaration-name refinement). It is not reimplemented. Its parser
recovers from syntax errors, so it still produces tokens for a half-typed line.

**L3** is the REPL-only payoff and the cheapest query in the system: an identifier
absent from the snapshot's `globals`, from module globals, and from enclosing locals
is **unbound** and renders dimmed. That is a pre-flight `doesNotUnderstand` — no
editor can do it, because no editor knows what has *run*.

Every layer only ever *refines* the one below. A layer that cannot answer leaves the
lower layer's coloring untouched; none may remove or downgrade a token.

Deleted by this section: `PhalcomHighlighter`'s regex battery, and
`completer::guess_type_from_name` (which infers `List` from a trailing `s`).

## §S6 — Latency: input is hard-real-time, suggestions are not

The governing preference, stated by the user and binding on this design:

> not laggy; it's fast, but it's okay if the REPL takes some time intentionally
> before giving suggestions

Two tiers, and they are not negotiable against each other:

**Hard constraint — the input loop never blocks.** Keystroke echo and L1 highlighting
are synchronous and `O(line length)`. A REPL line is short; a relex is microseconds.
Nothing in L2/L3, completion, or the snapshot may sit on this path.

**Soft budget — deferred is preferred over eager.** L2, L3, and hint computation run
**debounced on idle** (start at ~100 ms; tune by feel, not by benchmark). A brief,
predictable pause before suggestions appear is explicitly the desired behavior. Racing
to recompute on every keystroke is the failure mode being avoided, even where it would
be affordable.

Consequences:
- Debounce is a **feature**, not a fallback. Do not "optimize" it away later.
- No caching layer is specced. This repo measures first (`docs/forge/perf-log/`), and
  a debounced recompute on a short line is very likely already under budget. Add
  caching when a measurement demands it, not before.
- The snapshot is built **once per cell**, off the keystroke path entirely, so its
  cost is invisible regardless of size.

## §S7 — DEC-REPL-B: completion inserts a call opening, not a snippet

**RULED.** Accepting a completion inserts `name(` and places the cursor between the
parens. Selectors of arity 0 insert the bare name with no parens and no cursor move —
§D9's structured selectors already carry arity, so this is a read, not a guess.

reedline has no tab-stop engine, and `phalcom-lsp` emits LSP snippet-format items
(`at(${1:index})`). Something has to give. Inserting the snippet text verbatim is the
one unacceptable option: it puts literal `${1:...}` garbage in the buffer. So the
choice is between stripping the placeholders and building a tab-stop engine.

Stripping wins for now because a real implementation is not a polish pass. Tab-stops
need a placeholder state machine owned by the line editor, a rule for what Tab means
when stops are pending versus when ghost text is pending (§S4 already binds Tab), and
an escape path for nested and abandoned stops. That is its own unit with its own
interaction surface against §S6's debounce. It is not smuggled in here.

The degraded form is not a placeholder for the real one — `name(` with the cursor
inside is what most consoles do, and it is correct on its own terms.

## §S8 — DEC-REPL-C: the rustyline stack is deleted, not ported

**RULED.** The first step of implementation — before any of §S4's or §S5's rewrites —
is to delete `phalcom-repl/src/rustyline/` and drop the `rustyline` dependency from
`phalcom-repl/Cargo.toml`. Both `rustyline` and `reedline` are dependencies today;
`src/*.rs` is the live reedline path.

Ordering is the whole point. §S4 rewrites the `Hinter`, §S5 replaces the highlighter
and deletes `completer::guess_type_from_name`. Against two editor stacks each of those
is either done twice or done once and left inconsistent, and the copy that is not
exercised is the one that rots. Deleting first collapses the rewrite to a single
target.

This also settles a documented hedge rather than leaving it: `CLAUDE.md` currently
describes the parallel stack as "alternate/experimental" and tells the reader to
"treat `src/*.rs` as the active path unless verifying otherwise." After this deletion
there is nothing to verify against, and that sentence is updated to say so.

Rejected — **keeping it as a fallback**: a fallback that is never entered is not a
fallback, it is a second thing to break. §S6's two-tier latency model, §S1's snapshot
lifetime, and §D10's cell boundary are all specced against the reedline path
specifically; a rustyline copy would drift from all three the moment it stopped being
compiled against them.

## §S9 — Command namespace: `:reload`, `:reset`, `:help`; only `:reload` built

**RULED.** REPL commands take a leading `:` at the start of a line. Three are specced;
one is implemented in this unit.

| command | specced | implemented here | meaning |
|---|---|---|---|
| `:reload` | yes | **yes** | discard session state, re-run accumulated cells in order |
| `:reset` | yes | no | discard session state, keep nothing |
| `:help` | yes | no | list commands |

The `:` prefix is safe by grammar, not by convention: every `Colon` in the parser is
medial — argument labels and annotations, always preceded by an identifier
(`parser.rs:1479`, `:2467`, `:2790`). No Phalcom expression can begin with `:`, so the
command namespace cannot shadow a legal input, now or after the namespace grows.

**`:reload` re-runs; it does not replay-as-persistence.** §D1 rejected replay as the
*mechanism* for session state and specced a persistent session module instead. That
rejection stands. `:reload` is an explicit, user-invoked escape hatch for the case
§D1's model cannot serve — a cell was edited in the user's head and the session should
be rebuilt from the corrected sequence. The distinction is that replay here is
requested and visible, not silent and load-bearing.

> Hazard — `:reload` must build a **fresh `Compiler` and a fresh session module**, not
> re-feed cells to the existing one. Three separate rulings key off per-`Compiler`
> state: §D4's two-set immutability, U-BINDINGS' same-scope redeclaration ban
> (`scope.rs:181`), and PDR-0002's registration of class declarations in the same
> `global_bindings` map. Re-running `let x = 1` or `class Foo {}` through the surviving
> `Compiler` trips the redeclaration ban and `class.already_defined` respectively —
> `:reload` would fail on any session that declared anything. A fresh `Compiler` is
> what makes the replay legal, and it is the same reason the cross-cell regression test
> in `err-plan.md` must not be weakened: it is the only guard on this interaction.

`:reset` and `:help` are specced now and unimplemented on purpose. Specifying the
namespace up front is what keeps the *next* command from being invented ad hoc with a
different prefix or a different state discipline; building all three is scope this unit
does not need.

Rejected — **bare-word commands** (`reload`): they collide with identifiers, so a user
with a `reload` binding either loses it or shadows the command, and which one wins
becomes a rule nobody can remember. Rejected — **`/reload`**: no grammar conflict, but
`/` reads as a path or a division and every REPL precedent for `/` is a chat client,
not a language console.

## Write-set delta

Beyond `err-plan.md`'s: `phalcom-repl/src/{completer,highlighter,helper,editor}.rs`
rewritten; `phalcom-repl/src/rustyline/` deleted (§S8); `phalcom-repl/Cargo.toml`
loses the `rustyline` dependency (§S8); `CLAUDE.md`'s "alternate/experimental editor
stack" sentence updated (§S8); `phalcom-lsp` consumed as a library, never modified.
