# 8. Layered exceptions + `Result`, with terminating semantics

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/current/error-handling.md`; `docs/spec/current/values-and-absence.md` §4; [ADR-0007](0007-option-as-abstract-with-some-none.md); `docs/spec/current/object-model.md` §Errors; `docs/spec/current/concurrency.md`

## Context

[Open Question §9](../../spec/current/open-questions.md) asked whether Phalcom should use
`throw`/`try`/`catch` exceptions or `Result` as a sibling of `Option`. The two are
usually framed as a choice, but the existing spec already constrains it:

- The VM **must** raise. `doesNotUnderstand`, arity mismatch, and `DeadFrameError`
  arise deep in the runtime and cannot be surfaced as a `Result`
  ([Object Model §Errors](../../spec/current/object-model.md), [Blocks §5](../../spec/current/blocks.md)).
- `Result` is already reserved as `Option`'s sibling ([ADR-0007](0007-option-as-abstract-with-some-none.md)).
- Non-local `return` is already implemented as a frame-token stack unwind
  ([Functions §3](../../spec/current/functions.md)); fibers already `abort`/propagate/capture
  ([Fibers & Futures](../../spec/current/concurrency.md)).

So exceptions exist whether or not we bless them, and `Result` exists as a value.
The open question is really how they compose — and whether exceptions are
resumable (Smalltalk conditions) or terminating (Swift/Rust/JS).

## Decision

**Layer both; do not choose.**

- **Exceptions** — `throw` an `Error`, unwind the stack — for exceptional and
  cross-cutting failure. `throw expr` is sugar for `expr.raise()`. Only `Error`
  subclasses are throwable.
- **`Result` / `Option`** — values — for expected, local failure. `Result` mirrors
  `Option`'s abstract-superclass-with-two-subclasses shape (`Ok`/`Err`).
- **Bridges** make the channels interconvertible: `{ … }.attempt() -> Result`,
  `result.unwrap()` (re-`throw`s), `option.okOr(err)`, `result.ok()`.
- **Handling is a `Block` protocol** — `blk.on(ErrorClass){ e => … }`,
  `blk.ensure{ … }` — with `try`/`catch`/`finally` as pure JavaScript-shaped sugar
  over it.

**Terminating, not resumable.** A `throw` always unwinds; Phalcom rejects
Smalltalk's `resume:`. Resumable conditions require keeping the raising frame alive
and a handler-return protocol — heavy, rarely used, and in tension with the
frame-token unwinding already in the VM.

**One unwind primitive.** Non-local `return`, `throw`, and fiber `abort` are three
payloads of a single stack-unwinding mechanism (`Return` token vs `Raise(error)`).
`ensure` therefore fires on *any* unwind through it, not just exceptions.

## Consequences

- No either/or trap: VM-internal failures stay exceptions; domain failures can be
  type-visible `Result`s; conversion is one send either way.
- `try`/`catch`/`finally` add no semantics beyond the block protocol — a JS
  programmer's mental model works, and the primitive stays message-based.
- `ensure`/`finally` firing on non-local `return` and `abort` (not only `throw`) is
  the one subtle rule, and it falls out of the unified unwind rather than being a
  special case.
- Terminating semantics compose with the existing frame-token machinery; no
  resumable-condition bookkeeping.
- New kernel value classes `Result`/`Ok`/`Err` to bootstrap alongside
  `Option`/`Some`/`None`.

## Alternatives considered

- **Exceptions only.** Forces expected/recoverable failures through control flow
  and loses type-visible error paths; `Option` would stand alone and asymmetric.
- **`Result` only.** Impossible — the VM must raise for `doesNotUnderstand`,
  dead frames, and arity errors, which have no `Result` to thread through.
- **Resumable conditions (Smalltalk).** More expressive, but costly to implement
  and at odds with the terminating unwind the VM already performs. Deferred; a
  future `retry` on the protected block covers the common case cheaply.
- **Throw-anything (JavaScript).** Rejected: typed `on(_)` handlers and the
  `Error>>message` protocol require every thrown value to be an `Error`.
