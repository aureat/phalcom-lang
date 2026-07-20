# 01 — Coroutines and the suspension problem

> **The one question:** *where does a suspended computation's execution state live?*
> Every coroutine, generator, fiber, green thread, and `async` runtime ever built is an
> answer to it. The answer determines the garbage collector, the FFI boundary, the shape of
> the standard library, and which idioms are expressible. Almost nothing else about a
> concurrency design is genuinely free.

---

## 1. Conway's distinction, and why it forces the question

**`[R]`** Melvin Conway's 1963 paper, "Design of a Separable Transition-Diagram Compiler"
(*CACM* 6(7), 396–408), introduced the term *coroutine* while solving a mundane engineering
problem: a COBOL compiler whose passes each wanted to be written as a loop over the whole
input. The conventional structure ran pass one to completion, wrote an intermediate tape, then
ran pass two over that tape. Conway's alternative was to run the passes as peers — pass one
produces one token, transfers control to pass two, and when control eventually comes back,
pass one *continues from where it stopped* rather than restarting.

(Provenance note: this citation is `[R]`, recalled and not verified against the primary source.
See [`00-provenance-and-citation-discipline.md`](00-provenance-and-citation-discipline.md) for
why that distinction is stated so insistently here of all places.)

The conceptual payload is a single asymmetry:

> A **subroutine** is a master–servant relationship. The callee is created at the call and
> destroyed at the return; its lifetime is strictly nested inside its caller's.
> A **coroutine** relationship has no master. Both parties persist across a transfer of
> control; neither is destroyed by handing over.

Now observe what nesting buys, because it is what coroutines give up. A subroutine's local
state can live on a stack *precisely because* lifetimes nest: calls arrive last-in-first-out,
so a stack pointer is a sufficient allocator, `push` is a call, `pop` is a return, and
deallocation is free. The stack discipline is not a convention — it is a data structure that
is only correct under the nesting assumption.

A coroutine violates the assumption by construction. It outlives the transfer that suspended
it. Its locals, its instruction pointer, and its own chain of pending calls must survive in
something that is not the caller's stack. Hence the question at the top of this file, and hence
the fact that "add coroutines" is never a local change to a runtime: it is a change to where
execution state is allowed to live.

A second distinction, developed after Conway and worth keeping separate from the first:

- **Symmetric** coroutines transfer to a *named* peer (`transfer(to: other)`). There is no
  caller/callee relationship at all — control moves sideways.
- **Asymmetric** coroutines have a resume/yield pair: a resumer calls into the coroutine, and
  `yield` returns control to whoever resumed, along an implicit link. **`[R]`** This is the
  shape Lua chose and the shape almost every modern implementation chose, because the implicit
  resumer link is exactly what a scheduler needs in order to exist; symmetric transfer pushes
  scheduling into user code.

**`[V]`** Phalcom is asymmetric: `FiberObject` carries an explicit `resumer` field, set at
resume time, and failure propagation walks the resumer chain
(`docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md` §2;
`phalcom-core/src/primitive/fiber.rs`).

---

## 2. The four answers, and what each one costs

Every implementation lands in one of four positions. The taxonomy is worth internalizing
because the costs are not commensurable — they land on different parts of the system, so
"which is best" is never answerable without knowing what else you have committed to.

### (a) State is already heap data — the trampolined interpreter

If the interpreter never uses the host language's call stack to represent guest-language
calls — if a guest call is *pushing a frame onto a vector* rather than *calling a host
function* — then the entire suspended state of a computation is already an ordinary data
structure. Suspension becomes trivial: move the vectors somewhere and stop reading them.

**`[V]`** This is Phalcom's baseline. Pure Phalcom→Phalcom sends are trampolined through one
`run_until` loop with no native recursion, so a thousand-deep guest call chain leaves the Rust
stack flat. The consequence is stated in ADR-0030 §3: a fiber switch relocates the VM's
"current stack / current frames" into the `FiberObject` and never copies stacks.

The cost is that *everything* must obey the rule, and the moment one primitive breaks it, the
guarantee is gone for any computation passing through that primitive. Which brings us to the
actual situation.

### (b) State is partly on the host stack — the restricted model

Real interpreters have native primitives that need a guest value back synchronously. A
collection combinator implemented in the host language calls a guest block and must have its
result before it can continue. The natural implementation re-enters the interpreter loop
recursively, and now host stack frames sit *between* the coroutine's entry and its yield point.

**`[V]`** ADR-0030 §1 names this precisely and calls it the crown-jewel hazard
**native-stack frames ⊗ suspendable control**:

> When the running fiber is inside such a primitive, native Rust frames sit between the fiber's
> entry and the `Fiber.yield` call site, and those frames *are* the fiber's suspended position —
> you cannot repoint a handle and return through them without destroying it.

The restricted answer is to forbid the situation rather than solve it: detect that host frames
are on the stack and raise instead of suspending. **`[V]`** Phalcom raises
`CannotYieldAcrossNativeFrame`, guarded by comparing `VM::native_reentry_depth` against the
fiber's recorded `floor_depth` (`phalcom-core/src/primitive/fiber.rs:1-13`).

The cost is an expressiveness hole with a sharp edge, because the hole is not where a user
would predict it. **`[V]`** From ADR-0030 §4:

```phalcom
Fiber.new { let n = 0; while (true) { Fiber.yield(n); n = n + 1 } }   // ✅ legal
Fiber.new { list.each { x => Fiber.yield(x) } }                       // ✗ raises
```

The first works only because the sacred-selector inliner (ADR-0018) lowers `while` to `Jump`/`Loop`
opcodes inside a single chunk — no frame push, no native frame. The second fails because `each`
is a native combinator. The user-visible rule is therefore not "don't yield in a loop" but
"don't yield underneath a *native* callback," and which callbacks are native is an
implementation fact the surface syntax does nothing to advertise. That is the real tax: not
the missing capability, but that its boundary is invisible.

### (c) De-recurse everything — the full trampoline

Rewrite every callback-taking primitive so it pushes work onto the guest frame stack instead
of calling the interpreter recursively. Then case (b) never arises and yield is legal
everywhere.

**`[V]`** ADR-0030 lists this as Alternative B and defers it, with the decisive property being
not capability but *reachability*: "A → **B (full trampoline)** is purely *additive* —
de-recursing the callback primitives later just removes the guard. Shipping A forecloses
nothing."

The cost is a rewrite of the primitive/callback protocol — every combinator, `perform`, and the
does-not-understand forwarding path. It is invasive, it complicates every future primitive
(each one must be written in a state-machine style), and it buys an idiom whose absence has a
mechanical workaround (index iteration).

### (d) Give each coroutine a real machine stack — stackful

Allocate a native stack per coroutine and switch with assembly. Yield crosses host frames
because the host frames go with the coroutine.

**`[V]`** ADR-0030 rejects this as Alternative C, and the reason is worth quoting because it is
a *different kind* of argument than the others:

> it adds an `unsafe` stack-switch dependency and **permanently constrains the GC** — every
> parked fiber's native stack becomes a root the future moving collector must scan/relocate
> (crown-jewel *stackful-fiber ⊗ moving-GC*) […] The power is not worth an irreversible GC
> commitment.

The others are capability trades. This one is a **reversibility** trade. Option A and B live on
the same monotonic path; C leaves it permanently. A native stack is opaque memory containing
interior pointers the collector cannot interpret, so a moving collector can neither relocate
what it points to nor precisely identify what is live. Choosing C would silently retire
ADR-0009's claim that a moving collector can drop in behind the handle heap.

**The generalizable rule:** when comparing designs, separate *what it can do* from *what it
lets you do later*. An option that is strictly more capable but forecloses a future axis is
often worse than the restricted option that keeps the axis open — and the foreclosure is
usually invisible in a feature comparison, because it is a fact about designs you have not
written yet.

---

## 3. How the switch actually works, and why it is O(1)

**`[V]`** Two functions, ten lines each, implement the entire mechanism
(`phalcom-core/src/primitive/fiber.rs:29` and `:49`):

```rust
pub(crate) fn store_live_into(vm: &mut VM, fiber_ref: ObjRef) {
    let frames = std::mem::take(&mut vm.frames);
    let stack = std::mem::take(&mut vm.stack);
    let open_upvalues = std::mem::take(&mut vm.open_upvalues);
    let checking = std::mem::take(&mut vm.checking);
    let fiber = vm.heap.fiber_mut(fiber_ref);
    fiber.frames = frames;
    fiber.stack = stack;
    fiber.open_upvalues = open_upvalues;
    fiber.checking = checking;
}
```

Two properties make this constant-time, and both are consequences of earlier decisions that
were not made with fibers in mind:

**Moving a `Vec` moves three machine words.** Pointer, length, capacity. The elements are never
touched. "O(1) switch" does not mean "a fast copy" — it means *no copy occurs*, at any size.
A fiber suspended a million frames deep parks in the same time as one suspended at depth two.

**`[V]` Frame offsets are frame-relative.** ADR-0030 §3: "`CallFrame.stack_offset` stays
**frame-relative**, so per-fiber stacks starting at 0 need no rebasing." Had offsets been
absolute into a single global stack, every switch would require walking the frame vector and
fixing each offset — O(depth), and the whole design collapses into a copying scheme. This is a
representation choice made for unrelated reasons that turned out to be the load-bearing
precondition for cheap suspension. Worth noticing as a pattern: *the properties that make a
feature cheap are usually established long before the feature is designed.*

### The fourth field, and why it is the interesting one

Three of the four moved fields are obvious: frames, values, open upvalues. The fourth,
`checking`, is where the design gets educational. **`[V]`** From the source comment:

> `checking` (ADR-0052 Fix 1, U-ANNOT-CONTRACTS) swaps alongside the three fields above for the
> same reason: an `@invariant`-guarded call can `yield` mid-body, so this fiber's in-flight
> guard bookkeeping must park with it rather than leak into whichever fiber runs next.

`@invariant` is a decorator feature; re-entrancy bookkeeping for it is a decorator implementation
detail. It has nothing to do with concurrency — until you notice that a guarded call can suspend
in the middle, at which point per-VM bookkeeping becomes per-fiber bookkeeping or it is simply
wrong.

**The generalizable rule, and it is the most useful thing in this file:** *any VM-global mutable
state that can be non-empty at a suspension point is a bug unless it is explicitly classified as
fiber-local or fiber-global.* Adding coroutines retroactively reclassifies every field in your
interpreter struct. Most implementations discover this one field at a time, in production, as
mysterious cross-task contamination.

### The one field that deliberately stays behind

**`[V]`** ADR-0030 §6, stated as a hard constraint on all future work:

> **Invariant:** the VM-global monotonic `next_frame_generation` counter **must not** be
> relocated into `FiberObject` — it is the only thing making a cross-fiber token globally
> non-matching.

The mechanism: non-local return (ADR-0013) targets its home frame through a token carrying a
pointer and a generation number. Validation compares the generation at the target against the
generation in the token. Because the counter is global and monotonic, every frame ever created
in the process has a unique generation, so a token minted on one fiber can never coincidentally
match a frame on another — it fails validation and raises `DeadFrameError`.

Park the counter per-fiber and two fibers independently mint generation 7. A cross-fiber
non-local return then finds a frame that *validates* and is a completely different frame. The
failure mode is not an error; it is a return into unrelated live state — a type-confusion bug
with no diagnostic.

**The generalizable rule:** a uniqueness invariant is only as strong as the scope of the
counter that produces it. Sharding a counter for locality silently downgrades global uniqueness
to per-shard uniqueness, and any check that relied on the global property becomes a check that
passes when it should fail. This is the same class of error as reusing IDs after a restart.

---

## 4. Move versus swap — terminology that is load-bearing

**`[V]`** ADR-0030 §3 says "O(1) pointer **swap**." The implementation performs `std::mem::take`,
which is a **move**. A reconnaissance pass flagged the mismatch across three documents plus the
ADR itself.

This is not pedantry, and the test suite proves it:

- A **swap** is symmetric and total. Both halves exist somewhere at every instant. Interrupt it
  and the state is recoverable.
- A **move** empties the source. Interrupt it and whatever was in flight is gone.

**`[V]`** Consequently `fiber_resume` validates the entry callable's arity *before* calling
`store_live_into`, never after. Validate after, and a bad-arity resume has already emptied the
resumer's stacks into a fiber that will never run — the resumer is destroyed by a *type error*
in its argument. The regression golden locks it:

```
phalcom-core/tests/lang/concurrency/fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph
expected: "Error\nMethod call expected 1 argument, got 0\nroot continues"
```

The final line, `root continues`, is the whole assertion.

**The generalizable rule:** in a design document, the difference between two words for "exchange
state" is the difference between an operation that has a safe interruption point and one that
does not. Prose that blurs it will produce code that validates in the wrong order, because the
reader who implements from the prose has been told the operation is repairable.

---

## 5. Asymmetries a coroutine implementation always has

Two, both `[V]` in Phalcom, both present in every asymmetric coroutine system and both usually
undocumented:

**First resume is not a resume.** On the first entry there is no parked state to restore; the
implementation pushes a fresh entry frame onto empty stacks and marks the fiber started. Every
later entry restores parked state, truncates the value stack to the recorded `resume_slot`, and
pushes the delivered value. Two code paths wearing one name — the source of a whole family of
bugs where a fix applied to "resume" only ever ran on one of them.

**Completion is not suspension.** A fiber that returns normally never parks. **`[V]`** On
failure, Phalcom clears the failed fiber's parked state entirely (`frames.clear()`,
`stack.clear()`, `open_upvalues.clear()`) precisely because it can never resume, and walks the
resumer chain to deliver the error — `try`-mode resumers receive it as a value and stop,
`call`-mode resumers cascade further up.

**`[V]`** Both paths converge on one four-step landing procedure —
`switch_to_fiber_and_deliver`: `load_live_from`, truncate to the slot, push the value, set
status `Running`. Used identically by yield, by successful completion, and by failure delivery.
Finding the single procedure that all transitions funnel through is the best available test of
whether a coroutine implementation is coherent or is three special cases in a trenchcoat.

---

## 6. Parked coroutines and the collector

**`[V]`** ADR-0030 §7 states the invariant, and it predates the collector it constrains:

> a `FiberObject`'s value stack and frame stack are GC roots for as long as the fiber is
> reachable and not `done`/`failed` — **not only** the `current` fiber's. A collector that scans
> only `current` would free objects held solely by a parked fiber.

Two things are worth extracting. First, the root set of a concurrent runtime is not "the running
stack" but "the union over all live coroutines," and a collector written before coroutines
existed will have baked in the singular. Second, the *reason* Phalcom can satisfy this cheaply
is that the stacks live inside an arena object — which is the same property that made option (d)
unacceptable. **One representation decision simultaneously determined that parked state is
traceable and that native stacks were never an option.** Design decisions with that much reach
are rare and worth cataloguing when you find them.

---

## 7. What remains open

**`[O]`** From the overlay's open register, all three still unresolved and all three genuinely
hard:

- **Structured concurrency / cancellation propagation.** ADR-0030 provides single-fiber `abort`,
  which terminates one fiber. It does not provide cascading cancellation of children. **`[R]`**
  The nursery/scope literature argues that spawn without a join point is the concurrency
  equivalent of `goto`; Phalcom currently has the `goto`.
- **`select` / `race`.** Not mentioned anywhere in ADR-0030. Note that these are hard to retrofit
  precisely because they require a coroutine to be blocked on *several* wake conditions at once,
  which touches the parked-state representation this whole file is about.
- **Scheduler fairness.** The ready-queue exists as mechanism; no fairness policy is specified.
  Consistent with the mechanism-versus-policy stance developed in
  [`06-mechanism-versus-policy.md`](06-mechanism-versus-policy.md) — but an unspecified policy is
  still a policy, and it is currently "whatever the queue does."

**`[O]`** A gap in the canonical reading list worth closing: `references/reading.md` covers
Ierusalimschy on Lua coroutines and Dybvig on continuations, but has no Conway entry at all —
the origin of the concept this project's entire concurrency design descends from is absent from
the bibliography while being confidently cited in the memory database. That inversion is the
Conway incident in one sentence.
