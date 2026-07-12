# Errors

Phalcom keeps failure in two lanes and never makes you guess which one you're
in: `Result` for failure you expect and want to compute with, `throw`/`try` for
failure that blows past the local frame. Both are ordinary sends underneath —
sugar, not new mechanism.

## Two channels, not one

[Values](values.md) already introduced the shape: `Ok(v)` / `Err(e)`, mirroring
`Option`'s `Some`/`None`, method-for-method (`map`, `flatMap`, `unwrapOr`,
`match`). Reach for it when failure is *expected and local* — parsing,
validation, a lookup that might miss:

```phalcom
Int.parse(input)         // Result<Int, Error>
  .map { n => n * 2 }
  .unwrapOr(0)
```

`throw` is for the other case: *exceptional*, cross-cutting failure — bugs,
invariant violations, `doesNotUnderstand`, arity mismatches, anything that
surfaces deep in the VM where threading a `Result` back through every caller
would be noise, not signal. Neither channel is primary; the full rationale is
in [Error Handling](../spec/v0.2/error-handling.md) and
[Result](../spec/v0.2/result.md).

## Raising: `throw`

```phalcom
throw ArgumentError("age must be >= 0")
```

`throw expr` unwinds the stack. It is sugar for the reflective send
`expr.raise()` — the same relationship `return x` has to the VM's own unwind
machinery.

Only `Error` subclasses are throwable — `throw "oops"` is a compile error, a
deliberate departure from JavaScript's throw-anything. Typed handlers (below)
and the `message` protocol both depend on every thrown value actually being an
`Error`.

## Handling: a `Block` protocol, with sugar

Because everything is a message, the primitive isn't a special `try` form —
it's sends on a protected block, the same shape control flow already takes:

```phalcom
{ risky() }
  .on(TypeError) { e => recover(e) }    // typed handler — Smalltalk's on:do:
  .on(RangeError) { e => fallback() }   // handlers chain, first match wins
  .ensure { cleanup() }                 // always runs, success or failure
```

`on(_)(_)` installs a handler for one `Error` class (and its subclasses);
an error that matches nothing keeps unwinding past the block. `ensure(_)` runs
on every exit path — normal return, `throw`, or non-local return through it.

### `try` is sugar over that protocol

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

Each clause desugars to the block-protocol method of the same name:

| Clause | Desugars to | Meaning |
|---|---|---|
| `on T e { .. }` | `.on(T) { e => .. }` | typed handler for `T` and its subclasses |
| `catch e { .. }` | `.on(Error) { e => .. }` | catch-all — `Error` is the root of the hierarchy |
| `ensure { .. }` | `.ensure { .. }` | cleanup, runs on any exit |

`on`/`catch`/`ensure` are **contextual keywords** — reserved only as `try`
clauses, so `.on()`/`.ensure()` stay valid selectors and `Fiber>>try` keeps
working. The sugar adds nothing the protocol didn't already have; it exists so
the shape reads the way a JavaScript programmer expects.

A `throw` always unwinds — Phalcom doesn't have Smalltalk's resumable
conditions (`resume:`). A handler runs *after* the frames between the `throw`
and the handler are gone, and `return` inside a handler does an ordinary
non-local return to its own home method, not a jump back into the raiser. See
[Error Handling §3-4](../spec/v0.2/error-handling.md) if you need the unwind
mechanics.

## The bridges

The two channels stay cheap to cross precisely because conversion is one send
each direction:

```phalcom
let parsed = { Int.parse(input) }.attempt()   // throw -> Result: Ok(v) or Err(e)
parsed.map { n => n * 2 }.unwrapOr(0)

result.unwrap()          // Result -> throw: the Ok value, or throw the Err
opt.okOr(err)            // Option -> Result: Some(v) -> Ok(v), None -> Err(err)
result.ok()              // Result -> Option: Ok(v) -> Some(v), Err(_) -> None
```

`attempt()` is the block-level move: wrap risky code in a block, call
`.attempt()`, and a `throw` inside becomes an `Err` instead of propagating.
It's the synchronous sibling of the fiber-level `try` in
[Concurrency](concurrency.md) — both mean "hand me the failure as a value
instead of unwinding past me."

## What you can catch

Every built-in failure — `MessageNotUnderstood`, `RangeError`,
`DeadFrameError`, and friends — is an ordinary `Error` subclass, so they're all
catchable through the same protocol, and user-defined errors (subclass `Error`
or something more specific) participate identically:

```phalcom
{ obj.frobnicate() }.on(MessageNotUnderstood) { e => log(e.message) }
{ list[99] }.on(RangeError) { e => 0 }
```

For the full built-in hierarchy and the object-model side of `Error`, see
[Object Model](object-model.md) and
[Error Handling §6](../spec/v0.2/error-handling.md).

---

Next: [Modules](modules.md) — files as modules, `import`, and what a name
actually resolves to.
