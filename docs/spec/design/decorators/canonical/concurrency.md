# Concurrency decorators — evaluated against ADR-0030

- Status: **Mostly rejections, recorded so they stay rejected.** The
  committed model: cooperative, single-threaded, `Fiber` as the sole
  primitive; `Future`/`async`/`await` are a pure library layer; restricted
  yield (native re-entrant frames raise `CannotYieldAcrossNativeFrame`).
  Fibers are **uncolored** — any function can suspend; there is no
  async/sync split to annotate. That single fact decides this whole file.

## `@async` — rejected

The obvious proposal: `@async fetchAll() { … }` wraps the body so a call
returns a `Future` (Install-tier `wrap` in `Future.run`). It is cheap to
build and wrong to build:

1. **It smuggles function coloring into an uncolored language.** The
   generic hazard (async/await splits the function universe and every
   higher-order API) is precisely what ADR-0030's fiber choice avoids.
   `@async` recreates the split *invisibly*: the signature says `fetchAll()`,
   the return value is secretly a `Future`, and every caller must know to
   `.await` — but nothing at the call site or in the selector says so.
   Python/C# put `async` in the signature grammar because callers must see
   it; a decorator is the one place it cannot be seen from a call site.
2. **It violates philosophy check 5** (a decorator may not change a member's
   observable contract): `T` → `Future` is a type change behind a sigil.
3. **The explicit form is already one send**: `Future.run { self.fetchAll() }`
   at the call site says exactly what happens, composes with combinators,
   and follows the ADR's "no new mechanism" line.

`@await` has no coherent decorator reading at all (await is an expression
operation, and attributes are member-position only). Rejected without a
design.

## `@synchronized` — the one real concurrency decorator

Specified in [behavioral.md](behavioral.md); it belongs to this file's story
because its re-derivation *from* ADR-0030 (OS mutex → cooperative monitor
guarding the suspension window) is the model for how concurrency decorators
must be designed here: name the hazard that exists in *this* machine
(interleaving across yields), not the one imported from threaded languages.

## `@suspends` — proposed, optional, passive

A passive metadata marker: "this member may yield/await" (directly or through
its callees). Explicitly **advisory** — no enforcement, because suspension
is transitive and dynamic and Phalcom has no effect system to check it
(same honesty posture as the contract purity floor).

What it buys, cheaply: (a) the `@synchronized`-on-a-non-suspending-method
lint gets its inverse signal; (b) tooling/docs can surface "may suspend" at
call sites near fiber boundaries; (c) the `CannotYieldAcrossNativeFrame`
diagnostic can name the marked member in its hint. What it costs: one more
passive attribute (retention only, `runtime: false` trivially). Risk:
unmaintained markers lie — same body-drift hazard as `@native`, likewise
unsolvable cheaply. Net: **worth shipping only alongside a consumer** (the
lint or the diagnostic hint); a marker nobody reads is noise. PDR-trivial
when its consumer lands; not before.

## Scheduling / fiber-priority decorators — rejected

`@priority(…)`, `@background`, `@deadline(…)`: all presuppose a scheduler
*policy*, and the overlay records scheduler fairness as OPEN — the ready
queue is a mechanism, not a scheduler. Decorating methods with policy hints
for a policy that does not exist is speculation squared. Rejected until the
fairness question is ruled; even then, scheduling is plausibly a property of
*spawn sites* (`Future.run(priority: …)`), not of methods — a method does not
know what deserves priority, its caller does. (Precedent: Java's thread
priorities — method-level priority annotations never emerged anywhere
because priority is a runtime/creation-site property; the one language that
tied scheduling to declarations, Ada's pragma Priority on tasks, ties it to
the *task object*, the spawn-site analogue.)

## Structured concurrency decorators — premature

`@scoped`/`@nursery`-style markers await the open structured-concurrency and
cancellation questions (overlay: OPEN — `abort` terminates one fiber, no
cascade). Nothing to decorate until the semantics exist.

## What this precludes

- Any decorator whose effect is "calls to this method now return a different
  kind of value." This is the general rule `@async` instantiates, and it
  binds future proposals (`@cached` returning a `Lazy`, `@stream` returning
  an iterator) equally: change the body, never the contract.
- Concurrency annotations that imply enforcement the runtime cannot do.
  Advisory markers must say "advisory" in their own doc, every time.
