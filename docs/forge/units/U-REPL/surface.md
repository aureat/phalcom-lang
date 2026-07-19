# U-REPL — completion, hints, and highlighting surface

Companion to [`plan.md`](plan.md), at the same grain. `plan.md` specs the evaluation
substrate (§D1–§D10); this specs what the user actually sees. Decisions here are
lettered §S1–§S6 to stay distinct from the plan's §D-series.

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

Rationale is decision 0065 ruling 6. Cells *shadow*: a later `class Foo` binds a new
class. Replaying all cells as one synthetic document would therefore present two
`class Foo` declarations to the index — and under decision 0066 that is
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

## Open

- **DEC-REPL-B — snippet tab-stops.** reedline has no tab-stop engine; the LSP emits
  snippet-format items. Degrade to `name(` + cursor placement. A real tab-stop
  implementation is its own unit. *(Recommended resolution, not yet ruled.)*
- **DEC-REPL-C — dead editor stack.** `phalcom-repl/src/rustyline/` is unused and both
  `rustyline` and `reedline` are dependencies. Delete before building on reedline.
- **REPL command namespace.** `:reload` (re-run accumulated cells — the replay model
  rejected as the *persistence* mechanism in §D1, but sound as an explicit escape
  hatch), `:reset`, `:help`. Namespace specced; only `:reload` implemented.

## Write-set delta

Beyond `plan.md`'s: `phalcom-repl/src/{completer,highlighter,helper,editor}.rs`
rewritten; `phalcom-repl/src/rustyline/` deleted; `phalcom-lsp` consumed as a library,
never modified.
