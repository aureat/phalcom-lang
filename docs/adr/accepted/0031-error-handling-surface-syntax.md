# 31. Error-handling surface syntax: `throw` / `try` / `catch` / `on` / `ensure`

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0008](0008-layered-exceptions-and-result.md) (error **model** — layered,
  terminating, one unwind, handling as a `Block` protocol);
  [`docs/spec/v0.2/error-handling.md`](../../spec/v0.2/error-handling.md);
  [`docs/spec/v0.2/blocks.md`](../../spec/v0.2/blocks.md);
  [`docs/spec/v0.2/concurrency.md`](../../spec/v0.2/concurrency.md) (`Fiber>>try`/`abort`)

## Context

[ADR-0008](0008-layered-exceptions-and-result.md) fixed the error **model**:
`throw` an `Error` and unwind (only `Error` subclasses are throwable); `Result`
for expected failure; **handling is a `Block` protocol** — `blk.on(ErrorClass){ e => … }`,
`blk.ensure{ … }`; terminating, not resumable; `throw`/`return`/`abort` are one
unwind primitive so `ensure` fires on any of them. ADR-0008 sketched a
JavaScript-shaped `try`/`catch`/`finally` surface *illustratively* but did **not**
ratify the keyword spelling — that was tracked as a genuinely-open decision
([deferred-work §2](../../spec/v0.2/deferred-work.md); [decisions.md Q2](../../spec/v0.2/core/decisions.md))
blocking U-CORE-6's non-minimal slice. This ADR fixes the spelling. It does **not**
touch the model, and does not supersede ADR-0008.

## Decision

**A C-family keyword surface that is 1:1 sugar over the ADR-0008 block protocol.**

### 1. Raising — `throw`

```phalcom
throw ArgumentError("age must be >= 0")
```

`throw expr` is sugar for `expr.raise()`; only `Error` subclasses are throwable
(ADR-0008). Unchanged from the existing spec.

### 2. Handling — the `try` statement

```phalcom
try {
  let conn = Db.connect()
  conn.query(sql)
} on TimeoutError e {          // typed handler; clauses chain, first match wins
  retry()
} on NotFoundError e {
  log("missing: " + e.message)
} catch e {                    // catch-all: any Error
  report(e)
} ensure {                     // always runs — normal exit, throw, return, or abort
  conn.close()
}
```

### 3. It is pure sugar — the desugaring is transparent

Each clause lowers to exactly one method of the **same name** on the protected
block (ADR-0008), so there is no keyword→selector rename hop:

```phalcom
{ ... }
  .on(TimeoutError)  { e => retry() }
  .on(NotFoundError) { e => log("missing: " + e.message) }
  .on(Error)         { e => report(e) }     // ← catch e
  .ensure            { conn.close() }
```

- `try { … }` — the protected block (the receiver).
- `on T e { … }` ≡ `.on(T) { e => … }` — a typed handler for `T` and its subclasses.
- `catch e { … }` ≡ `.on(Error) { e => … }` — catch-all, since `Error` is the root.
- `ensure { … }` ≡ `.ensure { … }` — runs on every exit path.

No new semantics, and **no new floor primitive** — the surface compiles to sends
that already exist under ADR-0008.

### 4. `on` / `catch` / `ensure` are contextual keywords

They are reserved **only as clauses of a `try` statement**. Everywhere else they
remain ordinary identifiers/selectors — critically, the block-protocol methods
`on(_)(_)` and `ensure(_)` (which the sugar itself desugars to) stay callable, and
`Fiber>>try` remains a valid selector. `try` is reserved at statement-leading
position; a message send `fiber.try` / `fiber.try(v)` is unaffected (it is in
message position, not statement-leading position).

### 5. `ensure`, not `finally`

The cleanup keyword is `ensure`, mirroring the block-protocol method `.ensure{}`
exactly. This refines the illustrative `finally` in ADR-0008's prose — the ratified
spelling keeps the sugar and the method identical.

## Consequences

- **1:1 transparency.** Every keyword maps to one block-protocol method of the same
  name; a reader can mechanically desugar a `try` statement with no lookup table.
- **Grammar fit.** Each clause is a trailing `{}` block, consistent with Phalcom's
  brace grammar (no `begin…end`).
- **Full ergonomics.** Typed dispatch (`on T e`) plus catch-all (`catch e`) covers
  both Smalltalk-style typed handling and JS-style catch-all; terminating semantics
  match a JS/Swift/Rust reader's `try`/`catch` intuition.
- **Lexer/parser work.** `try` becomes statement-leading-reserved; `on`/`catch`/`ensure`
  become contextual keywords active only in `try`-clause position — the rule that
  preserves `.on()`/`.ensure()` selectors and the `Fiber>>try` message.
- **Unblocks U-CORE-6.** The non-minimal slice (a `try`/`catch` surface over the
  reified `Error`/`MessageNotUnderstood` hierarchy) now has a ratified spelling.
- `ensure` firing on `return`/`throw`/`abort` alike is inherited from ADR-0008's one
  unwind primitive — unchanged here.

## Alternatives considered

- **`raise` / `begin` / `rescue` / `ensure` (Ruby).** Rejected: `begin … rescue X => e … end`
  imports a statement-keyword block delimiter foreign to Phalcom's `{}` grammar, and
  `raise`/`rescue` are a rename hop from the `.raise()`/`.on()` methods.
- **Smalltalk `on:do:` only (no keyword sugar).** The raw block protocol
  (`{…}.on(T){ e => … }.ensure{ … }`) is always available and remains the primitive;
  most programmers expect a `try`/`catch` surface, and the keyword sugar is purely
  additive over it. Not chosen as the *only* surface.
- **`finally` instead of `ensure`.** Rejected — a rename hop from `.ensure{}`.
- **`catch (e: T)` typed-catch (JS-with-annotation; the earlier `error-handling.md`
  draft).** Superseded by the `on T e` clause, which mirrors `.on(T){}` and avoids
  two spellings for a typed catch.
