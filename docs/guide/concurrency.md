# Concurrency

Phalcom has exactly one concurrency primitive: a suspendable call stack you can
pause and resume by hand. Everything else — generators, async/await, a
scheduler — is a library built on top of it.

None of this runs yet. `Fiber` and `Future` are reserved class names with a
design that's fully specified and ratified
([ADR-0030](../adr/0030-fibers-and-futures-cooperative-concurrency.md)), but no
primitives behind them — see
[concurrency.md](../spec/current/concurrency.md) for the "unrealized today" list.
This page teaches the target semantics: what you'll write once the concurrency
unit lands, and why it's shaped this way.

## `Fiber` — a call stack you can pause

A `Fiber` wraps a block as an independently suspendable call stack: its own
value stack, its own frames, not the caller's. `call` resumes it; `Fiber.yield`
inside it suspends, handing a value out to whoever called `call` — and the
*next* `call` hands a value back in as `yield`'s return.

```phalcom
let counter = Fiber.new {
  var n = 0
  while (true) { Fiber.yield(n); n = n + 1 }   // suspends here each time
}
counter.call()   // 0  — runs until the first yield
counter.call()   // 1  — resumes right after that yield, runs to the next
counter.call()   // 2
```

There is no preemption: a fiber runs until it explicitly `yield`s, returns, or
raises, and nothing else can interrupt it mid-expression. That's the whole
concurrency story — no locks, no shared-memory races, because only one fiber
is ever actually running.

| Signature | Side | Meaning |
|---|---|---|
| `@constructor new(_)` | class | wrap a block as a not-yet-started fiber |
| `call` / `call(_)` | instance | resume; the argument becomes `yield`'s return value (or the entry's parameter, first time) |
| `try` / `try(_)` | instance | like `call`, but a failure comes back as an `Error` value instead of propagating |
| `yield(_)` | class | suspend the *running* fiber, handing a value to whoever resumed it |
| `current` | class | the fiber now running |
| `isDone` | instance | `true` once the entry has returned or raised |

`yield` and `current` are class-side because they always act on whichever
fiber is running — you can't yield a fiber you're merely holding a reference
to, only the one you're inside.

## The restricted-yield rule

`yield` can't cross every kind of frame. Specifically: **you cannot yield out
of a native callback**. If a native primitive — `.each`, `.map`, `perform`, any
combinator that calls a block back into Phalcom — has your block on the stack
when you call `Fiber.yield`, the VM raises `CannotYieldAcrossNativeFrame`
instead of suspending.

```phalcom
Fiber.new { list.each { x => Fiber.yield(x) } }   // ✗ CannotYieldAcrossNativeFrame
Fiber.new { for (x in list) { Fiber.yield(x) } }   // ✓ suspends freely
```

The reason is structural, not a missing feature: `.each` is a Rust function
that calls your block and waits synchronously for its `Value` back. Suspending
mid-`yield` there would mean capturing a **native Rust stack frame** as part of
the fiber's parked state — and that frame isn't something the VM can snapshot
and resume later, only unwind. `for`, by contrast, lowers to an *inlined*
`while` over the cursor protocol
([ADR-0018](../adr/0018-sacred-selector-inliner-and-override-guard.md),
[iteration.md](../spec/current/iteration.md) §2) — one bytecode chunk, no native
frame in between, so `yield` suspends exactly where it's called.

So the pattern that stays legal is anything that yields at the **top level of
the fiber's own body** — plain sends, inlined `while`/`ifTrue`, and `for`
loops over the cursor protocol. What's foreclosed is yielding from *inside a
block you handed to someone else's native code*. This is a guard, not a
permanent wall: lifting it for callback-shaped generators is tracked as the
additive follow-up
[ADR-0033](../adr/0033-amend-fiber-execution-trampolined-block-callsite.md).
Until then, reach for `for` wherever you'd have reached for `.each { yield }`.

## Generators are just fibers that yield

A "generator" isn't a separate feature — it's a `Fiber` whose body produces
values instead of a single result, paired with the
[iteration cursor protocol](collections.md): each `call` pulls the next
element, lazily, only computing as far as the consumer asks.

```phalcom
let squares = Fiber.new {
  for (n in 1..1000000) { Fiber.yield(n * n) }
}
squares.call()   // 1
squares.call()   // 4
squares.call()   // 9   — the millionth square is never computed unless asked for
```

Wrapping that in a class whose `iterate`/`iteratorValue` drive the fiber turns
it into an ordinary `for`-loop target — a lazy sequence *is*, under the hood, a
fiber-backed producer. See [Collections](collections.md) for the cursor
protocol itself.

## `Future` — the async layer

`Future` is a thin state machine over `Fiber`: a value that's `pending`,
`fulfilled`, or `rejected`. `async` runs a block on a fresh fiber and hands you
back a future for its result; `await` suspends the *current* fiber until that
future settles, and a **scheduler** — a run loop over ready fibers — is what
resumes it when it does.

```phalcom
let f = Future.async { slowComputation() }   // starts running on its own fiber
doOtherWork()                                // this fiber keeps going
let result = f.await                         // suspends until f settles
```

`await` is direct-style suspension, not blocking — the fiber yields control to
the scheduler, which runs other ready work and resumes this one once the value
is there. `then`/`map`/`catch` are the continuation-passing form of the same
thing; both bottom out in the future's waiter list, so pick whichever reads
better at the call site.

`Future` adds no VM mechanism of its own — no new opcode, no new stack. It's
built entirely from `Fiber` plus a ready-queue, which is deliberate: the
concurrency primitive stays singular, and `Future` is "just" the ergonomic
layer over it. Full interface and the scheduler's contract are in
[concurrency.md](../spec/current/concurrency.md) §2.

---

Next: [The Object Model](object-model.md) — the class/metaclass tower that
`Fiber` and every other heap type sits inside.
