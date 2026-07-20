# PDR-0007 — Bounded call depth and native re-entrancy: two counters, one error

- Status: Accepted
- Date: 2026-07-20
- Related: [ADR-0008](../adr/accepted/0008-layered-exceptions-and-result.md) (terminating unwind;
  `ensure` fires on any unwind — the mechanism this error must travel),
  [ADR-0030](../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) §4
  (`native_reentry_depth`, the existing re-entrancy guard this record generalizes),
  [ADR-0018](../adr/accepted/0018-sacred-selector-inliner-and-override-guard.md) (why a depth
  cap does not bound loops), [PDR-0003](0003-no-user-visible-threads-fibers-and-isolates.md)
  (single VM thread, so the counters need no atomics)

## Context

Phalcom has **no resource limits of any kind**. The `language-design` overlay states it
outright — *"resource limits (stack depth / allocation caps) unspecified."*

Measured 2026-07-20 on `c346200`, with

```phalcom
class Boom {
  construct new() {}
  go(n) { return self.go(n + 1) }
}
Boom.new().go(0)
```

the interpreter ran for **five minutes without erroring, overflowing, or aborting**, and was
still running when killed. It does not crash, because Phalcom's call frames are not on the
native stack: `VM::frames` is a heap `Vec<CallFrame>`. Unbounded recursion therefore grows the
heap until the OS OOM-kills the process — no diagnostic, no unwind, no `ensure`, no traceback,
nothing a user or an embedding host can catch.

That is strictly worse than a stack overflow. A `StackOverflowError` is a *defined* outcome;
this is a hang.

**There are two independent exhaustion paths, and they are not interchangeable:**

| Path | Grows | Reached by |
|---|---|---|
| `.ph` recursion | `VM::frames` — **heap** | ordinary Phalcom method/block recursion |
| Native re-entrancy | the **Rust stack**, via recursive `run_until` | `send_dynamic` (`vm/send.rs:218`), `doesNotUnderstand` chains, primitives that call back into `.ph`, REPL value echo |

`vm/send.rs:229` already documents the second path — *"Re-entrant native frame (ADR-0030 §4): a
fiber switch is forbidden while this recursive `run_until` is on the Rust call stack."* ADR-0030
introduced `native_reentry_depth` as a *correctness* guard for fiber switching. It is not a
resource limit, and nothing caps it.

A single counter cannot cover both: heap frames are cheap and can safely number in the tens of
thousands, while each native re-entry consumes a real Rust stack frame and will segfault the
process — an abort Rust cannot catch — at a far smaller count.

## Decision

### 1. Two counters

- **`.ph` frame depth** — checked where a frame is pushed. Ceiling **10,000**.
- **Native re-entrancy depth** — the existing ADR-0030 §4 counter, now also a limit. Ceiling
  **32**.

> **Amended 2026-07-20, during implementation.** This section first proposed **200** for the
> native ceiling. That number was a guess and measurement refuted it: on a standard 2 MiB
> thread stack — what Rust gives test and spawned threads, versus the main thread's 8 MiB —
> 200 still aborts the process before the counter can fire. Measured: 128 aborts, 64 and 32
> survive. 32 is taken, leaving margin for deeper interpreter frames and smaller embedder
> stacks. A ceiling that holds only on the main thread does not protect the resource it exists
> to protect, and §4's "there is no after" is exactly why this had to be measured rather than
> reasoned about.

Both are per-VM, and therefore per-fiber in effect: [PDR-0003](0003-no-user-visible-threads-fibers-and-isolates.md)
guarantees one VM thread, so neither counter needs atomics.

### 2. One error class

Exceeding either raises a single ordinary Phalcom `Error` subclass. It is a normal raise, so
ADR-0008's terminating unwind applies unchanged and **`ensure` blocks fire**. It is catchable.

The message names which limit was hit and its value. A limit the user cannot identify is a
limit they cannot work around.

### 3. The ceilings are constants, not a public knob

No `setRecursionLimit`. If a ceiling proves wrong, it is changed in the VM with a benchmark
justifying the new number.

### 4. Native re-entrancy is checked before recursing, not after

The native counter is tested at the point of re-entry, *before* the recursive `run_until` call.
Detecting native exhaustion after the fact is not possible — the process is already gone.

## Consequences

- Infinite recursion becomes a catchable Phalcom error instead of an OOM hang.
- Embedding hosts get a bounded failure mode. Today a one-line script takes the host down.
- `ensure` and `try` work across the failure, because it is an ordinary raise.
- Two counters, two ceilings, two tests — one for each path. The native path's test must
  actually drive re-entrancy (a `doesNotUnderstand` chain or a raising `toString` under value
  echo), not just deep `.ph` recursion, or it silently tests the wrong counter.

**Phalcom gets a real advantage here worth recording.** Because `.ph` frames are heap-allocated,
the `.ph` ceiling is an *honest* knob — raising it does not move the failure to a segfault.
CPython's `sys.setrecursionlimit` is a notorious footgun precisely because CPython frames still
consume C stack: raising the limit converts a clean `RecursionError` into a hard crash. Phalcom
cannot have that bug on the `.ph` path. It *can* on the native path, which is exactly why the
two ceilings are separate and why the native one is not user-adjustable.

**The cost, named plainly:** any legitimately deep recursion above 10,000 frames now fails
where it previously succeeded (slowly). Phalcom has no tail-call elimination, so deeply
recursive algorithms must be written iteratively or restructured. Ruby (`SystemStackError`) and
Python (`RecursionError`, default 1000) both impose this and both are usable languages; 10,000
is chosen well above their defaults because Phalcom's message-send idiom is frame-hungry — a
`for` loop body already costs several `.ph` frames per element.

**What this precludes.** A hard ceiling forecloses unbounded-depth algorithms without a
superseding PDR. It does *not* preclude adding tail-call elimination later, which would make
the ceiling irrelevant for the recursive shapes that matter most — and TCE remains the right
long-term answer for a language whose control flow is message sends.

**Scope — what this deliberately does not cover.** This is a *depth* limit, not a CPU or
allocation limit:

- **Loops are unbounded.** ADR-0018's sacred-selector inliner lowers `whileTrue` to `Jump`,
  pushing no frame. `while (true) {}` spins forever and this record does not touch it.
- **Allocation is unbounded.** `List` growth in a loop still exhausts the heap.

Both are real, both are separate axes (`security.md` Axis 6), and conflating them with depth
would produce a limit that satisfies nobody. They need their own record if they are ever wanted.

## Alternatives rejected

- **One shared counter.** Simplest, and wrong: any ceiling safe for the Rust stack (~200) makes
  ordinary `.ph` recursion useless, and any ceiling useful for `.ph` (~10,000) segfaults on the
  native path long before it trips.
- **Native stack probing** (Rust `stacker`, Go-style growable stacks). Measures the real
  resource rather than a proxy, and would let the native path grow safely. Rejected for now as
  a dependency and a portability surface disproportionate to the problem; §1's counter is a
  proxy but a sound one. Revisit if native re-entrancy depth ever becomes load-bearing.
- **Instruction/time budget (gas).** Bounds loops as well as recursion — the only option here
  that does. Rejected: per-op accounting taxes the dispatch loop that ADR-0051's whole
  performance program is trying to make cheaper, and it answers a question nobody has asked yet.
- **A user-settable limit.** Python's approach, and Python's footgun. Rejected under §3.
- **Do nothing.** The status quo is a five-minute measured hang with no diagnostic. Rejected —
  and noting that this was never a *decision*, only an omission nobody had written down.
