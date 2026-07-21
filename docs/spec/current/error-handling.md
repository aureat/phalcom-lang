# Error Handling

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0008](../../adr/0008-layered-exceptions-and-result.md) (layered exceptions + `Result`, terminating semantics) ·
[ADR-0031](../../adr/0031-error-handling-surface-syntax.md) (surface syntax: `throw`/`try`/`catch`/`on`/`ensure`)

Phalcom has **two** failure channels, layered rather than competing:

- **Exceptions** — `throw` an [`Error`](object-model.md) and unwind the stack. For
  *exceptional* and cross-cutting failures: bugs, invariant violations, dead
  frames, and anything that arises deep in the VM (`doesNotUnderstand`, arity
  mismatch, `DeadFrameError`).
- **`Result` / `Option`** — ordinary values ([Values & Absence](values-and-absence.md)).
  For *expected, local* failures you want visible in the type: parse, validate,
  lookup.

Neither is primary. The runtime must raise regardless — VM-internal failures
cannot be `Result`s — and expected outcomes are cleaner as values than as control
flow. Cheap bridges (§5) mean code is never trapped in the wrong channel.

## 1. Raising: `throw`

```phalcom
throw ArgumentError("age must be >= 0")
```

`throw expr` unwinds the stack (§4). It is surface sugar for the reflective form
`expr.raise()` ([Object Model §Errors](object-model.md)), exactly as `return x`
relates to the VM's unwind machinery.

**Only `Error` subclasses are throwable.** `throw "oops"` is a compile error. This
is a deliberate, signposted deviation from JavaScript's throw-anything: typed
handlers (§2) and the `message` protocol depend on every thrown value being an
`Error`.

## 2. Handling: a `Block` protocol, with sugar

Everything is a message ([Invariant 1](README.md)), so the primitive is sends on a
protected block — the same shape as control flow:

```phalcom
{ risky() }
  .on(TypeError) { e => recover(e) }   // Smalltalk on:do: — typed handler
  .on(RangeError) { e => fallback() }
  .ensure { cleanup() }                // finally — always runs (§4)
```

- `on(_)(_)` installs a handler for one `Error` class (and its subclasses).
  Handlers chain; the **first** matching class wins; an unmatched error keeps
  unwinding.
- `ensure(_)` runs its block on every exit path (§4).

### The `try` statement (sugar)

`try` / `on` / `catch` / `ensure` desugar directly to the block protocol
([ADR-0031](../../adr/0031-error-handling-surface-syntax.md)):

```phalcom
try {
  risky()
} on TypeError e {
  recover(e)
} catch e {
  fallback(e)
} ensure {
  cleanup()
}
```

- `on T e { … }` ≡ `.on(T) { e => … }` — a typed handler for `T` and its
  subclasses. Clauses chain; the **first** matching class wins.
- `catch e { … }` ≡ `.on(Error) { e => … }` — catch-all, since `Error` is the root
  of the raisable hierarchy.
- `ensure { … }` ≡ `.ensure { … }` — runs on every exit path (§4).

Each keyword mirrors the block-protocol method of the **same name**, so the sugar
adds no semantics the protocol lacks; it exists so a JavaScript programmer is not
surprised ([Invariant 6](README.md)). `on`/`catch`/`ensure` are **contextual
keywords** (reserved only as `try`-clauses), so the `.on()`/`.ensure()` selectors
and the `Fiber>>try` message keep working.

## 3. Terminating, not resumable

A `throw` **always unwinds**. Phalcom does not adopt Smalltalk's resumable
conditions (`resume:`): keeping the raising frame alive plus a handler-return
protocol is heavy, rarely used, and fights the frame-token unwinding already in
the VM ([Blocks §5](blocks.md), [Functions §3](functions.md)). A handler runs
*after* the stack between `throw` and the handler has been discarded.

`retry` (re-run the protected block) is a natural future addition — the block is
still live — but is deliberately left out of Draft 0.1.

## 4. Unwinding is one primitive

Non-local `return`, `throw`, and a fiber's `abort` are **three sources of the same
mechanism: stack unwinding.** The VM's unwind carries either a `Return`
(frame-token) or a `Raise(error)` payload ([Functions §3](functions.md)); a
`Raise` unwinds frame by frame until it meets a matching `on(_)` handler or the
fiber boundary.

Three consequences fall out, rather than being separate rules:

1. **`ensure` fires on *any* unwind through it** — `throw`, non-local `return`, or
   `abort` — as well as on normal fallthrough. Most languages special-case this;
   here it is the definition of `ensure`.
2. **A handler block is an ordinary block.** `return` inside a `catch` does
   non-local return to *its* home method ([Blocks §5](blocks.md)); a `throw`
   inside a handler or `ensure` unwinds outward from there. No special handler
   frame.
3. **An unhandled `throw`** that reaches the fiber entry is not a new case: it
   hands off to the concurrency path already specified — the entry's error unwinds
   its stack, sets `failed`, stores the `Error`, and resumes the resumer
   ([Fibers & Futures](concurrency.md)). Errors cross fiber boundaries only through
   `call`/`await` (propagate) or `try`/`catch` (capture), never implicitly.

## 5. Bridges between the two channels

The layering only works because conversion is trivial in both directions
([`Result`](values-and-absence.md)):

| Direction | Form | Meaning |
|-----------|------|---------|
| throw → value | `{ risky() }.attempt()` | run the block, capturing a `throw` into `Err(e)`; success is `Ok(v)`. Returns `Result`. |
| value → throw | `result.unwrap()` | the value, or `throw` the contained `Err` |
| absence ↔ error | `option.okOr(err)` / `result.ok()` | reserved in [ADR-0007](../../adr/0007-option-as-abstract-with-some-none.md) |

`attempt()` is the synchronous sibling of the fiber-level `try` ([concurrency.md](concurrency.md)):
both mean "run this, and hand me the failure as a value instead of propagating it."

```phalcom
let parsed = { Int.parse(input) }.attempt()   // Result<Int, Error>
parsed.map { n => n * 2 }.unwrapOr(0)
```

## 6. What can be caught

Every built-in failure is an `Error` subclass ([Object Model §Errors](object-model.md)),
so all of them are catchable with the same protocol:

```phalcom
{ obj.frobnicate() }.on(MessageNotUnderstood) { e => … }   // works today (reified via Raise)
{ list[99] }.on(RangeError) { e => … }                     // NEVER FIRES — see note below
{ escapedBlock.call() }.on(DeadFrameError) { e => … }      // NEVER FIRES — see note below
```

> **Correction (2026-07-20).** The second and third examples cannot work: `RangeError` and
> `DeadFrameError` kernel classes do not exist, and every non-`Raise` native error is wrapped,
> on catch, into a generic base `Error` instance — so those handlers silently never fire.
> The **ruled** replacement is [PDR-0010](../../pdr/0010-errors-carry-structure-and-cheap-origin.md)
> §2 (ratified 2026-07-20): every `Error` carries a `kind` Symbol, matched as
>
> ```phalcom
> { list[99] }.on(Error) { e => if (e.kind == #range) { … } }
> ```
>
> Kernel classes per condition are deliberately **not** minted (PDR-0001 makes them the most
> expensive answer). Normative `kind` table:
> [`traceback/implementation-spec.md`](../traceback/implementation-spec.md) §8.1. Unimplemented
> until traceback plan units T3/T6 land; until then only `Error`, `MessageNotUnderstood`, and
> `CannotYieldAcrossNativeFrame` are matchable classes.

User-defined errors subclass `Error` (or a more specific built-in) and participate
identically.
