# The Parked Fiber

*Concurrency track, Doc C2. Track plan: [CONCURRENCY-PLAN.md](../CONCURRENCY-PLAN.md).
Reads after [C1 — The Restricted Loop](restricted-loop.md).*

## The grip

A `FiberObject` is not a fiber. It is **the set of buffers a fiber is not currently using.**

While a fiber runs, its object is empty — its stack, frames, and open upvalues are the VM's own
fields. Park it and they move back. C1 established that much and used it. What C1 did not ask is the
question this doc is about:

**a fiber has twelve fields, and only four of them move. Why those four?**

The answer is not "the big ones" or "the ones that change." It is a three-way partition with three
different justifications, and once you can state the criterion you can predict the membership of the
set without looking — including the one field whose membership is, at HEAD, unexercisable.

And one word has to go first. Everyone in this repository calls the switch a *swap* — ADR-0030 §3
heads its section "Fiber switch is an O(1) pointer swap"; [Doc 3](../vm/frames.md) says the buffers
are swapped "as a unit"; C1 inherited it. It is not a swap. It is `mem::take` — a **move**, twice,
in opposite directions. The difference is invisible when both halves complete, and it is the whole
story when one of them doesn't.

---

## The debt this pays

Three shipped docs left an IOU here.

- **[Doc 3 (frames)](../vm/frames.md), [Lie #2](../vm/frames.md#lie-2).** "The VM has *a* frame
  stack" — really `VM::frames` is "a **live mirror** of the currently-running fiber's own buffer."
  Doc 3 deferred the mechanics and, in deferring them, quoted a field doc reading *"an O(1)
  pointer-free copy (a `Vec` swap)."* **That text is not at HEAD any more** (the field now reads
  "empty while running — mirrored by `VM::stack`", `heap/fiber.rs::FiberObject` @ ~L64). Paying this
  debt means correcting the word as well as expanding it — see [§ Move is not swap](#move-is-not-swap).
- **[Doc 6 (frame identity)](../vm/frame-identity.md)** borrowed ADR-0030 §6's
  `next_frame_generation` invariant as a given. [§ The counter that must not move](#the-counter-that-must-not-move)
  gives it back.
- **[C1](restricted-loop.md)** used the four `mem::take`s as a one-line fact and named the handoff
  explicitly: *"C2 owns the four fields as a mechanism — what each one is, why
  `next_frame_generation` pointedly stays VM-global, how a parked fiber is a GC root."*

So the four-take block is this doc's **premise**, not its reveal. If you have not read
[C1 § The swap, in four fields](restricted-loop.md#the-swap-in-four-fields), read it; it is quoted
there in full and will not be re-derived here.

**No other language appears in this document, and that is the filter working, not an omission.** The
comparison this subject would reach for — Lua's `lua_State`-per-coroutine, Go's copying stack growth,
Wren's fiber semantics, the stackful/stackless vocabulary — is spent in full by
[C1 § The fork that was actually argued](restricted-loop.md#the-fork-that-was-actually-argued) and
its Lua, Go, and Wren sections. Repeating it here would be a survey. What is left for C2 is a
partition with exactly one implemented answer and no occupants on other branches; a comparison
aimed at no confusion is decoration. Cut: Lua, Go, Wren, Ruby `Fiber`, CPython generators.

What stays owed elsewhere: fiber *failure* — the `call`/`try` cascade, the floor's teardown, and the
[E002 upvalue crash](../../errors/E002-fiber-floor-upvalue-crash.md) — is **C3**'s. The scheduler,
`System.schedule(_)`, and `Future` are **C4**'s. The `GetUpvalue`/`SetUpvalue` fiber-aware read
branch belongs to [`upvalues.md`](../vm/upvalues.md) and is not re-opened.

---

## Twelve fields, three fates

Here is the whole object, sorted not by declaration order but by what happens to each field when
control leaves.

| | Field | Fate on a switch |
|---|---|---|
| **Moves** | `stack`, `frames`, `open_upvalues`, `checking` | `mem::take`n out of the VM into this object; taken back out on resume |
| **Resident** | `status`, `resumer`, `result`, `entry`, `started`, `resume_slot`, `floor_depth`, `resume_mode` | never mirrored; read and written **on the object**, in place, by whoever is running |
| **Not here at all** | `next_frame_generation` | stays a `VM` field, and ADR-0030 §6 makes relocating it a named violation |

Verified field by field against `heap/fiber.rs::FiberObject` @ ~L62 and both halves of the switch
(`primitive/fiber.rs::store_live_into` @ ~L29, `::load_live_from` @ ~L49): those two functions
mention exactly four fields and no others; the remaining eight appear only as direct
`heap.fiber_mut(x).field = …` writes.

The middle row is the one nobody writes down, and it is the row that explains the criterion. Ask why
`status` is not mirrored the way `stack` is. Mirroring it would be *wrong*, not merely wasteful:
`status` is read **about** a fiber by whoever else is running — `fiber_resume` checks the callee's
status before touching anything (`primitive/fiber.rs::fiber_resume` @ ~L252), and the callee is by
definition not the running fiber. A mirrored `status` would be the mirror of the wrong fiber. Same
for `resumer`, `result`, `started`, `resume_mode`.

So the criterion is not size, or mutability, or hotness:

> **A field moves if it is state the running fiber uses *as itself*. A field stays resident if it is
> state some other fiber reads *about* this one.**

`resume_slot` and `floor_depth` are the edge cases that prove it. Both are per-fiber `usize`s that
look exactly like bookkeeping you would expect on the VM. Both stay resident, because both are read
across the boundary: `resume_slot` is written by the fiber that parks and consumed by the switch
that restores it (`vm/dispatch.rs::switch_to_fiber_and_deliver` @ ~L352), and `floor_depth` is
compared against `VM::native_reentry_depth` by the guard C1 documented. State that is *about* a
suspension cannot live in the mirror, because during the suspension there is no mirror to live in.

---

## Why those four move — and it is not one reason

The four that move look like a set. They are two sets wearing one `mem::take` block.

**Three move for a representational reason.** `stack`, `frames`, and `open_upvalues` are bound to
each other by **indices**. A `CallFrame`'s `stack_offset` indexes the value stack;
`open_upvalues` is a `BTreeMap<usize, ObjRef>` keyed by absolute value-stack index. The field's own
doc says it outright:

> *"Kept per-fiber because it is stack-index-keyed and each fiber has its own stack; swapping it
> with `stack`/`frames` prevents a cross-fiber slot-index collision."*
> — `heap/fiber.rs::FiberObject::open_upvalues` @ ~L73

An index is only meaningful against the buffer it was computed for. Move one of these three without
the others and you do not get a slow program or a confusing one; you get slot 4 of the wrong fiber's
stack. They are not three fields that happen to travel together — they are one datum in three
containers.

**One moves for a semantic reason.** `checking` is a `HashSet<ObjRef>` — the identity set of
receivers currently inside an `@invariant` re-entrancy guard (ADR-0052 Fix 1). It has no stack
dependence at all; nothing indexes it. It moves because a guarded call could `yield` mid-body, and
one fiber's in-flight guard bookkeeping must not be visible to whichever fiber runs next. That is a
statement about *meaning*, not about representation.

This matters more than a taxonomy. If you know the first reason only, you will predict the swap set
wrongly the moment someone adds per-fiber state that is not stack-keyed — a dynamic-variable stack,
an `ensure` chain, a handler list. Each of those would join the set for `checking`'s reason, not for
`open_upvalues`'s.

> **The count has drifted three times, in three places.** `store_live_into`'s own doc comment says
> it moves the "`frames`/`stack`/`open_upvalues`" **three** while its body moves four; the failure
> path's comment says "all three" (see below); Doc 3 described the set before `checking` joined it.
> `checking` was appended to a mechanism whose prose nobody updated. That is the ordinary way a
> two-reason set decays into a list.

---

## The fourth field swaps for a hazard that cannot currently happen

Now the part that surprised me, and which I had wrong going in.

If `checking` moves so that a guarded method can `yield` mid-body — can it? Trace the two gates:

- `fiber_yield` refuses unless `native_reentry_depth == floor_depth` (`primitive/fiber.rs` @ ~L338).
- `fiber_resume` refuses, for `call` and `try` alike, unless `native_reentry_depth == 0`
  (@ ~L248) — C1's "wider, sound over-restriction."

And the `@invariant` weave populates `checking` by calling the native `__invariantEnter` primitive,
which wraps the guarded body in a **native re-entrant frame** — the very thing both gates test for.
The negative-lane fixture says so in its own header: `Fiber.yield` inside an `@invariant`-guarded
body hard-errors today, *"same restriction as `.each { }`"*
(`tests/lang/runtime-errors/contracts_invariant_fiber_yield.ph`).

So the window in which `checking` can be non-empty is a window in which **every switch primitive is
already forbidden**. The fourth member of the swap set is swapped for a scenario HEAD forecloses
somewhere else entirely.

> **Labelled, because the evidence is asymmetric.** That the `@invariant` weave populates `checking`
> only from inside a native re-entrant frame is *verified* — the two primitives are its only writers
> (`primitive/object.rs::object_invariant_enter`/`_exit`), and the negative fixture pins the
> resulting behaviour. That *no other* path can populate `checking` is **inferred** from those being
> the only writers, not proven against every shape the weave can compile. If one exists, the
> unreachability argument below weakens and the clear-set gap becomes live.

It is not dead code, and it is not wrong. It is correct code for a restriction that is expected to
be narrowed — exactly like the relative-vs-absolute guard predicate C1 found in the same family
(`floor_depth` is provably always `0` at HEAD). Both are generality written for a world that has not
arrived. Naming that pattern is worth more than either instance: **this codebase repeatedly pays a
small amount of complexity to keep a future lift additive**, and it does so silently, so a reader who
assumes every branch is reachable will over-read the design.

> **Honesty, and a correction to my own recon.** I came into this expecting a tidier finding: the
> fiber-failure path clears `frames`, `stack`, and `open_upvalues` but never `checking`
> (`vm/dispatch.rs::run_until` @ ~L319-321, comment: *"clear all three parked fields here"*), which
> reads like a retention leak. The omission is real and verified. The leak is not currently
> reachable, for the reason just given — and `checking` is in any case a traced *edge* of a
> `FiberObject` (`heap/trace.rs::trace_object`'s `Fiber` arm), not a root in this collector's sense
> (`vm/gc.rs::collect_roots` roots only `VM::checking`, the live mirror). Both halves of the tidy
> version were wrong. What survives is smaller and more interesting: **the clear-set and the swap-set
> disagree**, and nothing at HEAD can make you notice.

---

## Predict before you read

Two resumes of the same fiber, same syntax both times:

```phalcom
let f = Fiber.new { first_arg =>
  System.print("fiber: started, first_arg = " + first_arg.toString)
  let got = Fiber.yield("from-yield")
  System.print("fiber: resumed after yield, got = " + got.toString)
  "fiber-done"
}

let y = f.call("hello")
let d = f.call("world")
```

**Where does `"world"` arrive?**

The natural answer — the one most readers give, and the one the syntax is actively encouraging — is
that it arrives the same way `"hello"` did. Both are `f.call(x)`. Same receiver, same selector, same
arity. Whatever mechanism delivered the first argument delivers the second.

Observed output (run at HEAD in a clean worktree, verbatim):

```
main: about to first-resume (fresh entry frame path)
fiber: started, first_arg = hello
main: first .call() returned from-yield
main: about to second-resume (load_live_from + resume_slot path)
fiber: resumed after yield, got = world
main: second .call() returned fiber-done
```

`"hello"` arrived as the entry closure's **parameter**. `"world"` arrived as the **return value of
`Fiber.yield`**, in the middle of a frame that was already on the stack. One expression, two
structurally unrelated delivery mechanisms, chosen by a `bool`:

```rust
// primitive/fiber.rs::fiber_resume @ ~L298 — first resume
if let Some((entry, closure_id, home_frame_token)) = entry_call {
    // `vm.stack`/`vm.frames` are empty here (just taken by `store_live_into` above),
    // so the callee's fresh window starts at 0.
    let stack_offset = vm.stack.len();
    vm.stack.push(Value::Obj(entry));
    vm.stack.extend_from_slice(args);
    …
    vm.frames.push(frame);
    vm.heap.fiber_mut(callee_ref).started = true;
} else {                                            // @ ~L308 — every later resume
    load_live_from(vm, callee_ref);
    let delivered = args.first().copied().unwrap_or_else(|| vm.none_value());
    let slot = vm.heap.fiber(callee_ref).resume_slot;
    vm.stack.truncate(slot);
    vm.stack.push(delivered);
}
```

Notice what the first branch does **not** call: `load_live_from`. A first resume is not a restore.
There is nothing to restore — the fiber has never run. It takes the resumer's buffers away and then
builds a fresh frame in the emptied mirror. So "a switch is a swap" describes exactly *one of the
two* resume paths.

This is also where `resume_slot` earns its place in the resident row. The second branch truncates to
a length recorded by the fiber *at its own yield*, then pushes. The value has to land exactly where
the `Fiber.yield(…)` send's window was, because that expression is mid-evaluation and its result
slot is waiting. A fiber must remember where it was interrupted, and that memory cannot live in the
mirror, because while it is suspended it has no mirror.

---

## Move is not swap

`store_live_into` is four `mem::take`s. `mem::take` replaces the source with `Default::default()`
and hands you the original. So between the take and the assignment into the `FiberObject`, the only
owner of a whole call stack is **a Rust local variable**.

Under a genuine swap, an error returned mid-switch is harmless: the state is in both places or in
neither, and you undo it by swapping back. Under a move there is no "back." Return early after the
take and the resumer's stack is in the callee's object, or dropped, and the resumer is now running on
an empty mirror.

That is not hypothetical. It is a fixed bug, and its fix is a comment in the source:

> *"Resolve and validate the entry callable **before** any state mutation … Doing this after
> `store_live_into` was a real bug — see the regression golden
> `fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph`."*
> — `primitive/fiber.rs::fiber_resume` @ ~L262-268

Arity validation for a first resume happens roughly forty lines before the take. It is not defensive
ordering; it is the only ordering that is correct, because the operation it precedes is
irreversible. **Validate before you take** is the whole discipline that a move-based switch imposes
and a swap-based one would not.

Two honest qualifications, because the shape of this story invites overclaiming:

1. **The regression test does not show a visible before/after.** Its own header says so: *"Neither
   defect was externally observable in this exact shape — `outer` is unconditionally marked `Failed`
   by the same fiber-floor cascade either way, discarding its state before anything reads it."* It
   locks an invariant; it does not demonstrate a corruption. The user-visible half of that fixture is
   a different bug in the same commit — the error message said `call` when raised from `try()`.
   Expected and observed output both:

   ```
   Error
   Method call expected 1 argument, got 0
   root continues
   ```

2. **No such gap exists at HEAD.** Reading every line between each `store_live_into` and its paired
   restore: in `fiber_resume` (@ ~L293 onward) and in `fiber_yield` (@ ~L347, whose next statement is
   `switch_to_fiber_and_deliver`, which calls `load_live_from` internally) there is no `?`, no
   fallible call, and no early `return`. The window is closed by construction today. The lesson is
   about what the representation *demands*, not about a live bug.

---

## The counter that must not move

`next_frame_generation` is a `u64` on the VM, incremented on every frame push. It is the one piece of
per-execution state that looks per-fiber and is forbidden from becoming so:

> **Invariant:** the VM-global monotonic `next_frame_generation` counter **must not** be relocated
> into `FiberObject` — it is the only thing making a cross-fiber token globally non-matching.
> — ADR-0030 §6

Re-derive it rather than memorizing it. A `FrameToken` is a `(frame_index, generation)` pair
([Doc 6](../vm/frame-identity.md): a pointer split into *where to look* and *who it was*). A block
carries one to reach its home frame for a non-local return. Now give each fiber its own counter, and
both fibers start at 0. Fiber A mints `(2, 7)`. Fiber B, independently, pushes its own frames until
it too has a frame at index 2 with generation 7. A block from A performs a non-local return while B
is running, and the token **matches** — a live frame belonging to a fiber that never called it.
Instead of the `DeadFrameError` the design promises, control returns into an unrelated activation.

The global counter makes that pair unforgeable across fibers by construction. It is worth seeing
which row of the partition table this is: the counter is not "about" a fiber at all. It is the
namespace the tokens are minted in, and a namespace that partitions per fiber stops being a
namespace.

Note the pleasing inversion with the previous section. `open_upvalues` is stack-keyed, therefore it
**must** move. `next_frame_generation` is identity-keyed, therefore it **must not**. Both are
consequences of the same question — *what is this index meaningful against?* — asked about a stack in
one case and about the whole program in the other.

---

## Parked fibers and the collector

ADR-0030 §7 states the invariant this way:

> *"a `FiberObject`'s value stack and frame stack are GC roots for as long as the fiber is reachable
> and not `done`/`failed` — **not only** the `current` fiber's."*

Read as a specification of mechanism, that says: enumerate the live fibers, root them all. That is
**not** what HEAD does. `vm/gc.rs::collect_roots` destructures the entire `VM` — an exhaustive
destructure, so the compiler enforces that no field is silently forgotten — and among fibers it
pushes exactly two things: `*current`, and `ready_queue` (scheduled-but-never-started fibers).
**There is no registry of live fibers.** A parked fiber is reached the way any other object is:
transitively.

Two paths do the work. `heap/trace.rs::trace_object`'s `Fiber` arm traces `fiber.resumer`, so
tracing the running fiber walks the entire resumer chain back to root, one hop at a time — even if
no variable anywhere names those ancestors. And ordinary value reachability covers everything else:
a fiber handle in a local, a global, a list, an upvalue.

That second path is worth stating because the resumer chain alone does not cover a fiber you hold and
have not yet resumed; the ADR's own wording, read as mechanism, would have you believe the collector
knows about fibers as a category. It does not. It knows about `current`, about a queue, and about
edges.

The ancestor-only case is the one worth testing rather than arguing, since it is the one where the
program itself holds no reference at all. Run a forced `System.gc` from inside a grandchild fiber
while its parent and the root are parked and named by nothing but `resumer` links, with live strings
on each parked stack:

```
main: secret built = root-secret-value-42
f2: forced gc while root+f1 parked (unreachable except via resumer chain)
f1: f2 returned f2-done
f1: f1local after gc = f1-local-value-7
main: f1 returned f1-done
main: secret after nested fiber + gc = root-secret-value-42
```

Both parked stacks survive the collection intact. **Verified for this shape**; the general invariant
— that every reachable fiber's buffers are reached — is *inferred* from the exhaustive destructure
plus the wildcard-free `trace_object` match, not exhaustively proven over every container shape.

And this is where ADR-0030's rejection of stackful coroutines pays off, in a subsystem that did not
exist when the choice was made. Because a parked fiber's stack is a `Vec` **inside an arena object**,
the collector reaches it with the same `trace_value` it uses for a list. Had the stacks been native
machine stacks, no amount of tracing would reach them — the collector would need conservative
scanning or precise stack maps, and a future compactor could never move what they point at. C1 walked
that branch and killed it; this is the invoice it would have sent.

---

## Pooling: measured, negative, off

A fiber costs two `Vec` allocations, and the obvious optimization is to recycle them. That machinery
exists — `FiberObject::new_entry_with_buffers`, `VM::fiber_pool`, and a recycle block in
`vm/dispatch.rs` that returns a finished fiber's buffers to a bounded free list.

All of it is behind `#[cfg(feature = "fiber-pool")]`, and `phalcom-core/Cargo.toml` reads
`default = []`. **It is compiled out of every ordinary build**, and it is off because it was measured
and lost. From [`perf-log/findings.md`](../../forge/perf-log/findings.md) F10 — same machine,
release, three reps:

| fibers | user (no pool → pool) | peak RSS (no pool → pool) |
|---|---|---|
| 100k | 0.06 → 0.06 s | 52.7 MB → 98.0 MB (**+86%**) |
| 500k | 0.31 → 0.31 s | 309 MB → 539 MB (**+74%**) |
| 1M | 0.62 → **0.85 s (+37%)** | 635 MB → 1090 MB (**+72%**) |

*"The RSS cost is linear in fibers created: ~450 B per fiber, dead on."* The ruling on the same page:
*"the flag stays, stays off, and is not to be used."* A separate experiment — presizing the buffers
rather than pooling them — also measured negative (`fiber_churn` user +20.0%, RSS +121.3%) and was
reverted.

The mechanism is not mysterious once you have the partition in view. A pool trades memory for
allocator calls, and it retains a buffer at whatever capacity its last owner grew it to. A fiber's
stack is `Vec`-shaped, so the recycled capacity is the *high-water mark* of a previous fiber, handed
to one that may need four slots. Held across many fibers, that is the +72–86% RSS, and at 1M fibers
the extra memory traffic costs more than the allocations it saved.

*(Every number in this section comes from `perf-log`. Nothing else in this document has been
timed — "O(1)" here is structural, four container moves independent of depth, and is not a
measurement.)*

---

## What you can now re-derive

- **Which fields move**, without looking: state the running fiber uses *as itself* moves; state other
  fibers read *about* it stays resident; a minting namespace stays global.
- **Why `open_upvalues` cannot move separately** from `stack` and `frames` — its keys are indices
  into one of them.
- **Why `checking` moves for a different reason than the other three** — and therefore what future
  per-fiber state would join the set (handler chains, dynamic variables) and what would not.
- **Why `status` and `resumer` are not mirrored** — they are read about a fiber that is, by
  definition, not the one running.
- **Why validation precedes `store_live_into`** — the operation is a move, and a half-finished move
  has no inverse.
- **Why a first resume looks nothing like a later one** — there is no parked state to restore, so
  the argument becomes a parameter rather than the value of a `yield` expression.
- **Why a per-fiber generation counter would be a correctness bug**, not an optimization — two
  fibers would mint identical `(index, generation)` pairs and a non-local return would land in a
  stranger's frame.
- **Why the collector needs no fiber registry** — `current` plus the traced `resumer` edge plus
  ordinary value reachability already covers every reachable fiber.

One line: **a fiber object is a parking lot, the partition says what is allowed to park there, and
the parking operation is a move — so every interesting bug in this subsystem is a move that did not
finish.**

---

## Anchors

| Symbol | Location |
|---|---|
| `heap/fiber.rs::FiberObject` | @ ~L62 — all twelve fields; the partition |
| `heap/fiber.rs::FiberStatus` / `::FiberResumeMode` | @ ~L12 / ~L37 |
| `primitive/fiber.rs::store_live_into` / `::load_live_from` | @ ~L29 / ~L49 — the four moves; doc comment says "three" |
| `primitive/fiber.rs::fiber_resume` | @ ~L247 — validate-before-take @ ~L262, first-resume branch @ ~L298, restore branch @ ~L308 |
| `primitive/fiber.rs::fiber_yield` | @ ~L333 — `store_live_into` @ ~L347 |
| `vm/dispatch.rs::VM::switch_to_fiber_and_deliver` | @ ~L352 — `load_live_from` + `resume_slot` truncate-and-push |
| `vm/dispatch.rs::VM::run_until` | @ ~L306-321 — failure cascade; clears three of the four |
| `vm/mod.rs::VM::next_frame_generation` | @ ~L109 — VM-global by invariant |
| `vm/gc.rs::VM::collect_roots` | @ ~L18 — exhaustive destructure; `current` + `ready_queue`, no fiber registry |
| `heap/trace.rs::trace_object` | `Fiber` arm — traces stack, frames, open upvalues, `resumer`, `result`, `entry`, `checking` |
| `primitive/object.rs::object_invariant_enter` / `_exit` | @ ~L336 / ~L353 — the only writers of `checking` |
| ADR-0030 §2/§3/§6/§7 + Alternatives | [`docs/adr/accepted/0030-…`](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) |
| F10 pooling measurement | [`perf-log/findings.md`](../../forge/perf-log/findings.md) |

Fixtures: `phalcom-core/tests/lang/concurrency/` —
`fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph` (the validate-before-take
regression), `concurrency_fiber_yield_resume.ph` (`0`/`1`/`2` across three resumes);
`tests/lang/runtime-errors/contracts_invariant_fiber_yield.ph` (negative lane — `yield` under an
`@invariant` guard).

---

## Forward pointers

- **C3 — when a fiber fails.** The `call`/`try` cascade this doc touched only to note its clear-set,
  `capture_error_value`, and the two confirmed scars — the
  [E002 upvalue-close crash](../../errors/E002-fiber-floor-upvalue-crash.md) and the `block_ensure`
  unrooted-result UAF. Both are the same family as this doc's thesis: **a value held live across an
  operation the recovery path does not see.**
- **C4 — futures.** `ready_queue` appeared here as a GC root and `System.schedule(_)` was skipped
  entirely; they decide *which* fiber resumes, reusing this machinery unchanged.
- **Unresolved, and no unit owns it:** the clear-set/swap-set disagreement — the failure cascade
  clears three of the four parked fields. Unreachable today for the reason given above; it becomes
  reachable the moment either the `@invariant` weave stops going through a native frame or the
  resume-side over-restriction is narrowed. Whichever lands first should clear `checking` in the same
  commit. **This doc describes it and deliberately does not prescribe a fix** — on this repo's
  confirmed backlog, four of six reproduced diagnoses had wrong prescriptions.
