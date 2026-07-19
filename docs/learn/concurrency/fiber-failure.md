# C3 — When a fiber fails

*Concurrency track, doc 3 of 4. Read [C1 — the restricted loop](restricted-loop.md) and
[C2 — the parked fiber](parked-fiber.md) first; this doc uses their vocabulary (floor, park, the four
`mem::take`s, the native-frame guard) without re-deriving any of it.*

---

## The grip

> **An error leaves a fiber twice over, and the two exits are different machines.** Inside the fiber
> it is a Rust `Err` walking down through frames, closing every escaped capture cell *before* it
> reclaims the slot that cell points at. At the fiber floor it stops being an unwind and becomes a
> value in a slot: the frames are not popped, they are dropped in bulk. Containment is implemented as
> **deletion**, not as a completed unwind — and everything the unwind was doing on the way down is
> exactly what the deletion forgets.

Both halves are deliberate and both are correct in isolation. The bug is the seam.

---

## The debt this pays, and the borders

[C1](restricted-loop.md) named `CannotYieldAcrossNativeFrame` a catchable surface `Error` and left
"what the floor does on the way down" here. [C2](parked-fiber.md) noted the failure path's clear-set
in passing — as a fact about the *swap* set — and handed over "the `call`/`try` cascade,
`capture_error_value`, and the two confirmed scars."

One of those two scars was paid while this doc was being written; see
[The family, one paid and one open](#the-family-one-paid-and-one-open).

**Not re-opened here:** the restricted-yield guard and `floor_depth` (C1); the twelve-field partition,
what parking *is*, and why `next_frame_generation` stays VM-global (C2); the `GetUpvalue` fiber-aware
read branch ([`upvalues.md`](../vm/upvalues.md)) — the crash below lands *in* that branch, and this
doc shows the panic and links rather than re-teaching the read; `System.schedule(_)`, the
ready-queue and `Future` (C4).

---

## Predict before you read

Two programs. They differ by one wrapper. One prints `captured`; the other panics the interpreter.
Decide which before scrolling.

```phalcom
// A
let leak = None
{
  let x = "captured"
  leak = { x }                 // a block capturing x escapes to an outer binding
  throw Error.new("boom")
}.on(Error) { e => System.print("caught: " + e.message) }
System.print(leak.call())
```

```phalcom
// B
let leak = None
let f = Fiber.new {
  let x = "captured"
  leak = { x }                 // identical
  throw Error.new("boom")
}
System.print(f.try().message)
System.print(leak.call())
```

The tempting reading is that **B** is the safer one. B's error never escapes its fiber at all — `try`
catches it by construction, the host is guaranteed to survive, and `f` is a clean, isolated failure
domain. A relies on a handler matching. If a design is going to drop something on the floor, surely
it is the one where the error crosses a boundary uncaught.

Verbatim, at HEAD:

```
$ phalcom a.ph
caught: boom
captured

$ phalcom b.ph
boom

thread 'main' panicked at phalcom-core/src/vm/dispatch.rs:1062:61:
index out of bounds: the len is 0 but the index is 1
```

A is fine. B — the *contained* one, the one that cannot possibly hurt anyone else — takes the process
down. And it is not a subtle corruption: it is an index into an empty `Vec`.

The reason is the grip. A's error is caught by a handler, so the VM runs a real unwind. B's error
reaches the fiber floor, where there is no unwind to run.

---

## Two exits, two machines

Here are the two paths side by side. This table is the doc.

| | **Inside the fiber** (A) | **At the fiber floor** (B) |
|---|---|---|
| Trigger | a `Raise` that a `Block#on(_)` handler matches | an error reaching the top of the fiber's own activation with no handler |
| Represented as | `Err(PhError)` returning up the **Rust** stack | a `Value` in `FiberObject::result`, plus `status = Failed` |
| Code | `unwind_to` (`vm/dispatch.rs:110-114`), called from `block_on` (`primitive/block.rs:274`) | the `Err` arm of `run_until` (`vm/dispatch.rs:290-338`) |
| What it does to the buffers | `close_upvalues_from(stack_len)`, **then** `frames.truncate`, **then** `stack.truncate` | `frames.clear()`, `stack.clear()`, `open_upvalues.clear()` — and, for the failing fiber, not even that (below) |
| Escaped captures | promoted to `Upvalue::Closed(value)` — the value is copied into the heap cell before its slot dies | left `Upvalue::Open { fiber, slot }`, pointing into a buffer that is now empty |

`unwind_to` is not merely correct by accident. Its rustdoc argues the case, in advance, against
exactly the thing the other path does:

```rust
/// Order matters: **close upvalues first**, then truncate — mirroring
/// `Bytecode::Return`/`Bytecode::ReturnNonLocal`'s own unwind order exactly
/// (`close_upvalues_from` before `stack.truncate`) — so a closure that escaped
/// the throwing block still observes its captured locals rather than a
/// use-after-free once its stack slot is reclaimed.
```
— `vm/dispatch.rs:97-103`

That sentence is program B, written down, before program B existed. The codebase predicted its own
bug in a doc comment and then built a second teardown path that does not call the function the
comment is about.

**Verified:** `close_upvalues_from` has four call sites — `unwind_to`, and the `CloseUpvalue`,
`Return`, `ReturnNonLocal` bytecode handlers (`dispatch.rs:1091`, `1103`, `1151`), all inside
`run_until_inner`'s ordinary dispatch loop. `unwind_to` has exactly one caller, `block_on`. No route,
direct or indirect, reaches any of them from the `Err` arm of `run_until`.

---

## What the floor actually does

```rust
Err(e) => {
    let error_value = self.capture_error_value(&e);
    let mut failed = self.current;
    loop {
        self.heap.fiber_mut(failed).status = crate::heap::FiberStatus::Failed;
        self.heap.fiber_mut(failed).result = error_value;
        self.heap.fiber_mut(failed).frames.clear();
        self.heap.fiber_mut(failed).stack.clear();
        self.heap.fiber_mut(failed).open_upvalues.clear();
        let mode = self.heap.fiber(failed).resume_mode;
        let Some(resumer) = self.heap.fiber(failed).resumer else {
            return Err(e);                       // root fiber: the host dies
        };
        match mode {
            FiberResumeMode::Try  => { self.switch_to_fiber_and_deliver(resumer, error_value); break }
            FiberResumeMode::Call => { failed = resumer }
        }
    }
}
```
— `vm/dispatch.rs:290-338`, comments elided

Two writes and three clears, per fiber. No frame is visited. Note what is *not* in this loop: no
`ip`, no `CallFrame`, nothing per-activation at all. It walks **fibers**, never frames.

### The disposal is in two places, and neither of them walks

This is the part worth slowing down for, because C2's framing — "the failure path clears three of the
four parked fields" — is true and is also not where the failing fiber's state actually goes.

Recall from C2 that a *running* fiber's `FiberObject` buffers are **empty**; its live state is in
`VM::frames`/`stack`/`open_upvalues`. The fiber that just raised is, by definition, the running one.
So for it, all three `clear()` calls are **no-ops**. They do real work only for an intermediate
`Call`-mode resumer walked later in the cascade, whose parked buffers genuinely hold what it stored
when it resumed the callee that failed.

The failing fiber's real state dies somewhere else entirely — in the ordinary restore:

```rust
pub(crate) fn load_live_from(vm: &mut VM, fiber_ref: ObjRef) {
    let frames = std::mem::take(&mut fiber.frames);
    let stack = std::mem::take(&mut fiber.stack);
    let open_upvalues = std::mem::take(&mut fiber.open_upvalues);
    let checking = std::mem::take(&mut fiber.checking);
    vm.frames = frames;
    vm.stack = stack;
    vm.open_upvalues = open_upvalues;      // <- drops the failed fiber's map, right here
    vm.checking = checking;
}
```
— `primitive/fiber.rs:49-59`

`vm.open_upvalues = open_upvalues` drops the previous value: a `BTreeMap<usize, ObjRef>` of live
`Upvalue::Open` cells, released by Rust's ordinary assignment semantics. No entry is iterated. No
`Open` becomes `Closed`.

So there are two bulk disposals on this path — an explicit `.clear()` for parked resumers and an
implicit drop-on-reassignment for the fiber that raised — and **the design has no per-cell step
anywhere**. That is why the crash is not a missing edge case. There is no loop to add an edge case to.

Program B's panic lands in the `GetUpvalue` handler at `dispatch.rs:1062`, on the cross-fiber arm
[`upvalues.md`](../vm/upvalues.md) owns: an `Upvalue::Open { fiber, slot }` whose `fiber` is not
`current` resolves through `heap.fiber(fiber).stack[slot]`. That `stack` was cleared, the fiber is
terminal and will never run again to refill it, and `slot` is 1.

This is [E002](../../errors/E002-fiber-floor-upvalue-crash.md), open at HEAD, independently
reproduced twice for this doc — once with `Fiber.abort` and `call`, once with `throw` and `try`. It
is not mode-specific and not API-specific.

> **This doc describes the defect and deliberately does not prescribe the fix.** On this repo's
> confirmed backlog, four of six reproduced diagnoses carried wrong prescriptions, and two of those
> would have broken the tree. E002 records a fix *direction*, explicitly marked unverified. Treat it
> as a hypothesis.

---

## The cascade

`FiberResumeMode` is recorded on the **callee** at resume time, because the resume call returns
immediately — an O(1) switch — long before anyone knows whether the callee will succeed.

- `f.try()` — a failure is delivered to the resumer as a `Value`.
- `f.call()` — a failure re-raises into the resumer as if it had been raised at the `call` site.

Under `call`, "re-raises into the resumer" is implemented by *not stopping*: the loop marks the
resumer `Failed` too, with the same error object, and keeps walking. Observed:

```phalcom
let inner = Fiber.new { throw Error.new("inner boom") }
let mid   = Fiber.new { inner.call(); System.print("MID CONTINUED") }
let e = mid.try()
System.print("host alive, e = " + e.message)
System.print(mid.isDone.toString + " " + inner.isDone.toString)
```
```
host alive, e = inner boom
true true
```

`MID CONTINUED` never prints, both fibers are terminal, the error arrives at the root's `try` as the
*inner* fiber's error object — the same instance, not a re-wrap — and the host survives. Swap
`inner.call()` for `inner.try()` and `mid` sees the error as a value and runs to completion,
returning `99`. The cascade stops at the first `Try`.

At the root there is no resumer, so `run_until` returns `Err` and the program ends. Containment is
therefore *relative*: `call` all the way up is exactly as fatal as no fibers at all, and `try`
anywhere in the chain is a firewall.

Two consequences worth carrying:

**The cascade runs no bytecode in any fiber it walks.** Not the resumer's next instruction, not a
handler, not a cleanup. It is a status-and-result write per hop. The code comment states this as
intent — "exactly as if `e` had been raised at each `call()` site in turn with no handler."

**No fixture exercises it.** Sixteen `concurrency/*.ph` fixtures combine `Fiber.new` with `.call()`;
none makes a fiber fail while resumed by a fiber that was itself resumed. The closest,
`concurrency_fiber_nested_current_identity.ph`, builds the two-deep shape and then never fails; and
`concurrency_sched_raising_fiber_does_not_abort_host.ph` is a single hop to the root. The cascade
loop's second-and-later iterations — the only part of this machine that distinguishes it from a
one-hop capture — are **untested at HEAD**. The programs in this section are the coverage.

### The value is not necessarily an `Error`

`capture_error_value` (`dispatch.rs:370-379`) passes a `RuntimeError::Raise`'s payload through
unchanged and wraps anything else — a native VM error with no surface form — in a kernel `Error`
instance carrying the rendered message. So a fiber's failure is always a *catchable value* and never
a Rust type, which is what makes `CannotYieldAcrossNativeFrame` (C1's foreclosure) an ordinary
`Error` a program can inspect.

"Always a value" is not "always an `Error`", though, and `Fiber.abort(_)` is where that shows:

```phalcom
let fa = Fiber.new { Fiber.abort(123) }
System.print(fa.try())            // 123        <- the raw number, unwrapped

let n = 123
let ft = Fiber.new { throw n }
System.print(ft.try())            // <MessageNotUnderstood>
```

`Fiber.abort(_)` type-checks nothing and raises its argument as-is (a Wren port —
`concurrency_fiber_wren_abort_number_captured.ph` asserts precisely this). `throw` desugars to
`.raise()`, which is installed only on `Error`, so `throw n` on a number dNUs and the *dNU's own*
error reaches the floor. Both arrive by the identical route; they diverge before it.

---

## The guard is what keeps this survivable

An obvious follow-up: if the cascade runs no cleanup, can it walk *past* an `ensure` and skip it?

Inside a fiber, cleanup works normally — the in-fiber path is the ordinary one:

```phalcom
let f = Fiber.new { { throw Error.new("x") }.ensure { System.print("CLEANUP RAN") } }
System.print(f.try().message)
```
```
CLEANUP RAN
x
```

But to have the *cascade* skip a cleanup you would need an intermediate fiber suspended inside an
active `ensure` — which requires resuming a fiber from inside one. C1's restricted-yield guard
forbids exactly that: `ensure` and `on` run their protected block through `block_call`, which raises
`native_reentry_depth`, and `fiber_resume` refuses any switch at non-zero depth
(`primitive/fiber.rs:248-250`). Attempting it produces the guard's error, not a cascade:

```phalcom
let c = Fiber.new { throw Error.new("c failed") }
let b = Fiber.new { { c.call() }.ensure { cleanupRan = true } }
b.try()
// cleanupRan == true; outcome is CannotYieldAcrossNativeFrame — c never started
```

`c.call()` is rejected before any switch, so `b` fails with the guard's error as an ordinary
single-fiber raise, and its `ensure` fires on the ordinary path. Adversarially checked: no program
was found in which a genuine multi-hop cascade walks through a fiber with a pending cleanup.

This is the second instance of a pattern C2 found for `checking`: **the restricted execution model
is quietly containing the consequences of the teardown's incompleteness.** That is worth stating
plainly in both directions. It is why E002 is the only reachable member of this family today — and it
is a reason to be careful, not comfortable, about C1's open question of narrowing the guard. Widening
what a fiber may do while suspended widens what the floor can drop.

---

## The family, one paid and one open

C2 handed this doc two scars. They are the same defect: *a value held live across a boundary that the
recovery scan does not cover.*

**[E001](../../errors/E001-gc-ensure-temp-root-uaf.md) — paid.** `block_ensure` kept the protected
block's pending result only in a Rust local while it ran the cleanup block re-entrantly; the root
scan enumerated stack, frames, fibers and universe, so a collection during cleanup swept a live
result and returned a dangling handle. At HEAD (`cdd2117`) `VM::push_temp_root` exists (`vm/gc.rs:148`),
`collect_roots` enumerates `temp_roots`, and `block_ensure` roots both the value and the `Raise`
error before calling cleanup (`primitive/block.rs:318-319`). Every repro in E001, plus the
error-carrying path and a nested case, now runs clean. The mechanism is depth-and-truncate rather
than push/pop, so a caller need not count its own pushes.

**[E002](../../errors/E002-fiber-floor-upvalue-crash.md) — open**, and program B above.

The contrast is the point. E001's fix had somewhere to go: there was a root enumeration, it was
missing an entry, and the fix added the entry. E002 has no equivalent hook, because **the failure
path has no unwind to extend** — you cannot add a step to a walk that does not exist. Whatever the
fix turns out to be, it has to introduce the walk, and that is a larger change than it looks from the
one-line symptom.

> **Two record corrections found while writing this**, both now fixed in-tree: E001 was still listed
> **OPEN** in [`docs/errors/README.md`](../../errors/README.md) and still asserted "`push_temp_root`
> has zero occurrences in the tree"; and E002's own reproduction used `var`, which stopped lexing
> when U-BINDINGS removed `Token::Var` (`42aafce`) — the recorded repro failed with a parse error
> rather than the crash. Rewritten with `let`, it panics exactly as recorded. A crash record whose
> repro no longer compiles is worse than no record.
>
> One **unverified lead**, surfaced and not chased: `block_on` (`primitive/block.rs:239-281`) builds a
> fresh `Error` instance and then calls `send_dynamic` re-entrantly without a temp root — the shape
> E001 was patched for, in a sibling primitive. Not reproduced; not a claim.

---

## The design space, and how much of it was argued

**Argued, in the ADR.** [ADR-0030 §6](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md)
deliberates *containment* explicitly: the ADR-0008 unwind "operates on `self.frames` only and stops at
the **fiber floor**, so a failing fiber captures its `Error` into its result slot instead of
terminating the host." That is a real decision and it is what shipped.

**Not argued anywhere.** Three things this doc had to reconstruct:

1. **The `call`/`try` split.** It appears in the spec and in `FiberResumeMode`'s rustdoc as a fact,
   never as a choice with a rejected alternative. Its actual provenance is a **port**: ten
   `concurrency_fiber_wren_*` fixtures ported from named Wren tests, which C1 established. A port is
   evidence about Phalcom's surface, not a deliberation.
2. **Whether "stops at the floor" means *unwinds to* the floor or *abandons at* it.** The ADR's
   wording implies the first. The implementation does the second. Nothing records that as a decision,
   which is what makes E002 an oversight rather than a trade-off.
3. **The cascade's no-bytecode rule.** Documented in a code comment; in no ADR.

### The branches, and who is on them

Two axes, and they are independent. *Is the failure contained to the fiber?* and *does the resumer
learn about it as a raised error or as a returned value?*

**Lua** is the ancestor and carries both delivery modes in one language: `coroutine.resume` returns
`false, err` — a value, no raise — while `coroutine.wrap` re-raises into the caller. Phalcom's
`try`/`call` is that pair, chosen per resume rather than per wrapper. *(Recalled, not verified
against a Lua implementation for this doc.)*

**Go** is the other branch on containment, and it is the sharp one. Goroutines have real stacks and
`recover` is per-goroutine — but an unrecovered panic in **any** goroutine terminates the entire
process. Go, with by far the most engineering invested in this design space, does not contain
goroutine failure at all; isolation is a property of *processes* there, not of the concurrency
primitive. Phalcom's containment is thus a real choice with a real alternative held by a serious
occupant, and the bill Go pays for its position is that a library's background goroutine can kill
your program. *(C1 spends Go on stacks, stack maps and `cgo`; this is a different axis and does not
restate it.)*

**Cut, with reasons.** **Wren** — the source of the `call`/`try` surface, but C1 already established
what our corpus does and does not prove about it; naming it twice adds a proper noun, not an idea.
**Ruby `Fiber`** — re-raises into the resumer with no value-mode counterpart; a strict subset of what
Lua already shows. **Erlang** — the canonical containment design, but its unit of isolation is a
supervised process with no shared heap, which is an argument about *supervision*, a different doc.
**JS generators / `async`** — vocabulary C4 will need for coloring, nothing to say about failure that
Lua does not.

---

## What you can now re-derive

1. Why program B crashes and program A does not, from the two-machines table alone.
2. Why the failing fiber's three `clear()` calls are no-ops, and where its state actually dies.
3. Why "add upvalue closing to the failure path" is not a one-line fix: there is no walk to add it to.
4. Why `call` and `try` are recorded on the callee rather than passed at the failure site — the resume
   returns before the outcome exists.
5. Why the cascade's no-cleanup behaviour is nearly unobservable today, and why that is C1's guard
   doing it rather than the failure path being careful.
6. Why a fiber's failure value is always catchable but not always an `Error`.

---

## Anchors

| Claim | Where |
|---|---|
| Fiber-floor capture + cascade | `phalcom-core/src/vm/dispatch.rs:290-338` |
| `capture_error_value` | `dispatch.rs:361-379` |
| `unwind_to`, and its ordering rustdoc | `dispatch.rs:88-114` (comment at `:97-103`) |
| `close_upvalues_from` + its four callers | `dispatch.rs:79-86`, `:111`, `:1091`, `:1103`, `:1151` |
| The crash site (`GetUpvalue`, cross-fiber arm) | `dispatch.rs:1058-1064` |
| Drop-on-reassignment of the live upvalue map | `primitive/fiber.rs:49-59` |
| The restricted-yield guard | `primitive/fiber.rs:248-250` |
| `FiberStatus` / `FiberResumeMode` | `heap/fiber.rs:11-44` |
| `Fiber.abort(_)` raises its argument unchecked | `primitive/fiber.rs:208-216` |
| `raise` installed only on `Error` | `primitive/error.rs:44-46`, `:61-65` |
| `push_temp_root` / `collect_roots` / `block_ensure` | `vm/gc.rs:148`, `:32-116`; `primitive/block.rs:318-319` |
| Containment decision | [ADR-0030 §6](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) |
| The two scars | [E001](../../errors/E001-gc-ensure-temp-root-uaf.md) (fixed), [E002](../../errors/E002-fiber-floor-upvalue-crash.md) (open) |

Fixtures read: `concurrency_fiber_is_done_and_error_once_failed.ph`,
`concurrency_fiber_abort_then_resume_fails.ph`, `concurrency_fiber_try_abort_current.ph`,
`concurrency_fiber_wren_try_value_error_capture.ph`, `concurrency_fiber_wren_abort_number_captured.ph`.

No runtime tracing exists (`vm-trace`'s subscriber is hardcoded to `LevelFilter::OFF`; the
disassembler walks only the top-level chunk), so every dynamic claim in this doc is either a line of
source or the verbatim output of a program that was run.

---

## Forward pointers

- **C4 — futures.** The `Ok` arm this doc quoted around also contains the root-drive pump that drains
  `ready_queue` when the top-level activation ends. `System.schedule(_)`, `Future`, and how a
  scheduled fiber's failure reaches the host (one hop, never a cascade) are C4's.
- **Owed, and no unit owns it:** the cascade has no test coverage past its first hop, and E002 is
  open with an unverified fix direction. Both are recorded, neither is prescribed here.
- **A qualification on C1.** Its open question — whether the resume-side over-restriction should be
  narrowed — now has a second cost attached. The guard is not only forbidding a capability; it is
  holding the lid on an incomplete teardown. Narrowing it makes E002's family reachable in shapes
  nothing tests today.
