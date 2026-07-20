# §07 — Stage 6: command namespace (§S9)

**Phase B — branch-only.** `phalcom-repl/**` only. Smallest stage; one hazard that will
cost a day if missed.

## 1. The namespace (RULED)

REPL commands take a leading `:` at the start of a line.

| command | specced | implemented here | meaning |
|---|---|---|---|
| `:reload` | yes | **yes** | discard session state, re-run accumulated cells in order |
| `:reset` | yes | no | discard session state, keep nothing |
| `:help` | yes | no | list commands |

`:reset` and `:help` are specced now and unimplemented **on purpose**. Specifying the
namespace up front is what keeps the *next* command from being invented ad hoc with a
different prefix or a different state discipline. Building all three is scope this unit
does not need.

## 2. Why `:` is safe — by grammar, not convention

Every `Colon` the parser accepts is **medial**: argument labels and annotations, always
preceded by an identifier (`phalcom-ast/src/parser.rs:1479`, `:2467`, `:2790`). **No
Phalcom expression can begin with `:`.** So the command namespace cannot shadow a legal
input, now or after the namespace grows.

Dispatch commands **before** the validator ([§04](04-continuation.md)) sees the line —
`:reload` is not Phalcom source and must never be classified as incomplete.

**Rejected — bare-word commands** (`reload`): they collide with identifiers, so a user
with a `reload` binding either loses it or shadows the command, and which one wins
becomes a rule nobody can remember. **Rejected — `/reload`**: no grammar conflict, but
`/` reads as a path or a division, and every REPL precedent for `/` is a chat client,
not a language console.

## 3. `:reload` re-runs; it does not replay-as-persistence

§D1 rejected replay as the **mechanism** for session state and specced a persistent
session module instead. **That rejection stands.**

`:reload` is an explicit, user-invoked escape hatch for the case §D1's model cannot
serve — a cell was edited in the user's head and the session should be rebuilt from the
corrected sequence. The distinction is that replay here is **requested and visible**, not
silent and load-bearing.

Input is `ReplSession.history` ([§02 §1.2](02-session-and-cells.md)), in submission
order.

## 4. The hazard — a fresh `Compiler` and a fresh session module

> `:reload` must build a **fresh `Compiler` and a fresh session module**, not re-feed
> cells to the existing one.

Three separate rulings key off per-`Compiler` state:

1. §D4's two-set immutability ([§03](03-immutability.md)),
2. U-BINDINGS' same-scope redeclaration ban (`compiler/lib/scope.rs:118`, `:170`),
3. PDR-0002's registration of class declarations in the same `global_bindings` map.

Re-running `let x = 1` through the surviving `Compiler` trips the redeclaration ban;
re-running `class Foo {}` trips `class.already_defined`. **`:reload` would fail on any
session that had declared anything** — which is every session worth reloading.

A fresh `Compiler` is what makes the replay legal. This is the same interaction the
cross-cell regression test in [§03](03-immutability.md) guards, and the same reason that
test must not be weakened.

Since each cell already compiles through its own `Compiler` ([§02](02-session-and-cells.md)),
the correct implementation is: **discard the module, create a new one, and re-run the
history through the ordinary cell path.** If that is what the code does, the hazard
cannot occur. Do not add a bespoke reload path that bypasses the cell loop.

## 5. Failure semantics

A cell that raised during the original session will raise again on reload. `:reload`
reports the failing cell and **stops**, leaving the session at the state reached so far.

Rationale: continuing past a failure produces a session that never existed — later cells
run against bindings the failing cell should have produced. Stopping is reproducible and
explains itself. Say which cell failed.

## 6. Tests

| Test | Asserts |
|---|---|
| `reload_rebuilds_session_from_history` | after `:reload`, prior bindings hold and rebinding still works |
| `reload_survives_declarations` | **the §4 hazard** — a session with `let x = 1` and `class Foo {}` reloads clean |
| `reload_stops_at_failing_cell` | §5 — reports the cell, halts, session usable |
| `colon_prefix_never_parses_as_source` | `:reload` is dispatched, never validated or compiled |

`reload_survives_declarations` is the one that matters. Written naively — reloading a
session that declared nothing — it passes against the broken implementation.

## 7. Write-set

| Path | Change |
|---|---|
| `phalcom-repl/src/repl.rs` | command dispatch; `:reload`; history replay through the cell path |
| `phalcom-repl/src/main.rs` | route `:`-prefixed lines before the validator |

`:reset` and `:help` are **recognized and reported as unimplemented**, not silently
treated as source. An unknown `:command` reports "unknown command" and lists the three —
which is `:help`'s body, so implementing that error message very nearly implements
`:help`. Do not let it: keep the specced/unimplemented distinction honest, or update the
table above to say `:help` shipped.

**Conflict risk vs class work: none.**

## 8. Gate

Workspace green. Manual: bind something, define a class, run `:reload`, confirm the
session rebuilds and both survive; run `:reset`, confirm it reports as unimplemented
rather than erroring as bad syntax.
