# 33. Amend the fiber execution model — trampoline the bytecode block call-site

- Status: Deferred (past v0.2 — the callback-generator ergonomic is delivered for
  v0.2 by [ADR-0035](../accepted/0035-iteration-protocol-cursor.md)'s `for`/cursor loop, which
  lowers to an inlined `while` and suspends freely under
  [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md) §4. Revisit this
  amendment as the general lift — for `.each { yield }` and other native-callback
  generators that `for` cannot express — when that becomes a real need, landing it
  with the [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md) §5
  fiber-switch signal per §Decision 4 below)
- Date: 2026-07-12
- Related: [ADR-0035](../accepted/0035-iteration-protocol-cursor.md) (the v0.2 resolution — `for`/cursor);
  [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md)
  (fiber execution model — **§4 amended here**); [ADR-0013](../accepted/0013-closure-upvalues-and-frame-token-return.md)
  (block closures + frame-token non-local return); [ADR-0008](../accepted/0008-layered-exceptions-and-result.md)
  (one unwind primitive / error floor); [ADR-0018](../accepted/0018-sacred-selector-inliner-and-override-guard.md)
  (sacred-selector inliner); [ADR-0019](../accepted/0019-freeze-vm-blessed-primitive-floor.md)
  (frozen floor); [`docs/spec/v0.2/concurrency.md`](../spec/v0.2/concurrency.md) §1

## Context

[ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md) §4 fixed the
fiber execution model at **restricted Option A**: `Fiber.yield` integrates only
with the top-level `run_until`, and a yield attempted while a re-entrant native
primitive sits on the Rust stack raises `CannotYieldAcrossNativeFrame`. It also
foreclosed, deliberately, the **callback generator**:

```phalcom
Fiber.new { list.each { x => Fiber.yield(x) } }   // ✗ CannotYieldAcrossNativeFrame
```

with a documented index-iteration workaround, and promised in its Consequences
that "**A → B (full trampoline) is purely additive** — de-recursing the callback
primitives later just removes the guard."

A code-grounded re-audit sharpens *why* that example fails and shows the additive
step is smaller than "de-recurse every callback primitive":

1. **`List.each` is already pure Phalcom** — an *inlined* `while` over
   `f.call(self.at(i))` ([`core.ph`](../../phalcom-core/core/core.ph) `each`).
   The `while` is lowered to `Jump`/`Loop` by the [ADR-0018](../accepted/0018-sacred-selector-inliner-and-override-guard.md)
   inliner and pushes **no** frame. So `each`/`map`/`filter`/`reduce`/`includes`
   contribute no native frame *of their own*.
2. **The one native frame is `f.call(x)` itself.** `f.call(x)` (and its `f(x)`
   sugar) compiles to a plain `Invoke` of selector `call(_:)`
   ([`bytecode.rs`](../../phalcom-core/src/bytecode.rs) `Invoke`), which resolves
   to the **`block_call` primitive**. `block_call` ends in
   `vm.run_until(base_frames)` — a **recursive Rust call**
   ([`block.rs`](../../phalcom-core/src/primitive/block.rs) `block_call`) reached
   through the `MethodKind::Primitive` arm of `call_method`
   ([`vm.rs`](../../phalcom-core/src/vm.rs) `call_method`). That nested `run_until`
   *is* the native Rust frame wedged between the fiber floor and the `yield`.

Contrast the `MethodKind::Closure` arm ([`vm.rs`](../../phalcom-core/src/vm.rs)
`call_method`): a send to a closure **pushes a `CallFrame` onto `self.frames`
and returns `Ok(())`** — no recursion; the *existing* top loop drains it. That is
the trampoline the fiber model already relies on for pure Phalcom→Phalcom sends.

So the entire ✅/✗ split reduces to a single fact: **is there a `block_call`
(→ recursive `run_until`) between the fiber floor and the `yield`?** The canonical
generator has none (inlined `while` + a direct `Fiber.yield` send); the callback
generator has exactly one (the `f.call(x)` inside `each`). De-recursing that one
call-site — not the whole callback-primitive family — turns every `.ph` combinator
built on `f.call` yield-transparent at once.

### What "de-recurse the call-site" means, concretely

A **native frame** is a Rust activation on the real call stack (a `run_until`
recursion): opaque to the VM, un-suspendable, only exitable by running to
completion. A **VM frame** is a `CallFrame` entry in `self.frames`: data the VM
owns, freely pushed/popped/parked, and — per [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md)
§2 — living *inside* the `FiberObject` so it survives a park. De-recursing
converts the former into the latter for the block call-path:

```
BEFORE (block_call primitive, re-entrant):
  run_until(fiber floor)         ← Rust frame
    call_method [Primitive arm]  ← Rust frame     three opaque Rust frames sit between
      block_call                 ← Rust frame     the floor and the yield; Fiber.yield
        run_until(base)          ← Rust frame     cannot repoint `current` and return
          Fiber.yield ─────────────────────────   up through them.  → raises.

AFTER (CallBlock opcode, trampolined):
  run_until(fiber floor)         ← the ONE Rust frame
    self.frames = [ …, each-frame, block-frame ]  ← all DATA, inside the FiberObject
          Fiber.yield ───────────  repoints `current`, returns up the single run_until.
                                   Suspended position = self.frames + self.stack. ✅
```

The removal of the native frame *is* the enabling change. Its cost is the
bookkeeping that the Rust stack was doing for free (§Consequences).

## Decision

**Amend [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md) §4: block
invocation reached from Phalcom bytecode is trampolined, not re-entrant. The
re-entrant `block_call` primitive is retained for native callers.**

### 1. A `CallBlock` call-site, trampolined like `MethodKind::Closure`

The compiler emits block application (`f.call(x)`, `f(x)`, and the trailing-block
combinator call form) as a call-site that, when the receiver resolves to a
`Block`/`Closure`, **pushes the block's `CallFrame` onto `self.frames` and returns
to the current `run_until`** — exactly the `MethodKind::Closure` arm's behaviour,
including the dummy receiver slot and arity check that `block_call` performs today
([`block.rs`](../../phalcom-core/src/primitive/block.rs) `block_call`), and the
`home_frame_token` stamp for non-local return ([ADR-0013](../accepted/0013-closure-upvalues-and-frame-token-return.md);
[`block.rs`](../../phalcom-core/src/primitive/block.rs) `block_call` frame stamp).
No new native frame is created.

The **compile-time** placement (recognising `call`/`value` at the call-site, in
the [ADR-0018](../accepted/0018-sacred-selector-inliner-and-override-guard.md) sacred-selector
spirit) is chosen over a runtime "is-receiver-a-Block?" branch in the `Invoke`
handler: the latter taxes the single hottest opcode in the VM on every send. The
accepted consequence is that a block invoked **reflectively** — `blk.perform(#call,
[x])`, or `call` reached through a dynamic send — still routes through the
primitive and is **not** yield-transparent (§Consequences, residue).

### 2. The re-entrant `block_call` primitive is retained for native callers

`block_call` stays exactly as-is. Native Rust callers that need a synchronous
`Value` back from a block continue to use it and remain non-yield-transparent:
the non-inlined `whileTrue`/`whileFalse` fallback
([`block.rs`](../../phalcom-core/src/primitive/block.rs) `block_while_true`),
`perform`/`sendDynamic`, `doesNotUnderstand` forwarding, and any future Rust-side
combinator. A `Fiber.yield` under one of those still raises
`CannotYieldAcrossNativeFrame`. This is the smaller, retained tail of Option A —
not a regression, and itself additively removable later (full Option B).

### 3. Non-local return, error unwind, and BoundMethod fall to the ordinary paths

With the block frame on `self.frames`, a block-body `return`
(`Bytecode::ReturnNonLocal`) and any raised `Error` unwind through `run_until`'s
**ordinary** handlers ([ADR-0013](../accepted/0013-closure-upvalues-and-frame-token-return.md)
token search; [ADR-0008](../accepted/0008-layered-exceptions-and-result.md) error floor,
stopping at the fiber floor). This **replaces** the non-local-return detection
that the `MethodKind::Primitive` arm performs today by sniffing the `self.frames`
length delta after `native_fn` returns and re-pushing the value
([`vm.rs`](../../phalcom-core/src/vm.rs) `call_method`, Primitive arm). That
heuristic is deleted for the trampolined path — a strict simplification, and the
same "typed signal, not a length delta" discipline [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md)
§5 mandates for the fiber switch.

`BoundMethod` receivers (which `block_call` intercepts *before* `resolve_callable`
and routes to `invoke_method_object`, [`block.rs`](../../phalcom-core/src/primitive/block.rs)
`block_call`) and unbound `Method`/reflective receivers are **not** trampolined by
the `CallBlock` path; they fall back to the existing primitive. The opcode
trampolines the pure `Block`/`Closure` case only.

### 4. Sequencing constraint — lands with, or after, the typed fiber-switch signal

Trampolining multiplies the number of `CallFrame` push/pop events during a
fiber's lifetime. [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md)
§5 already requires that a fiber switch be reconciled with the dispatch loop by an
**explicit typed `ControlFlow`/switch value, not** the `self.frames.len()`
heuristic — because a switch also moves `frames.len()`. This amendment **must not**
land before that typed signal exists: added block-frame churn would otherwise
widen the surface over which a length-delta heuristic could misread a switch as a
return (or vice-versa). `CallBlock` is therefore part of the fiber unit, not a
standalone pre-fiber slice.

## Consequences

- **The callback generator works, verbatim.** `Fiber.new { list.each { x =>
  Fiber.yield(x) } }` suspends correctly, as do generators over
  `map`/`filter`/`reduce`/`includes` and any `.ph` combinator built on `f.call`
  — because none of them any longer interposes a native frame. The
  [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md) §4 index-iteration
  workaround becomes optional rather than required.
- **A strict simplification of the return path.** The Primitive-arm
  `frames.len()` re-push heuristic ([`vm.rs`](../../phalcom-core/src/vm.rs)
  `call_method`) is removed for block calls; non-local return rides one uniform
  `run_until` path. Backtraces gain uniform `CallFrame`s for block activations.
- **Runtime cost is a net win.** No recursive interpreter setup per block call,
  and deep combinator nests stop consuming the native Rust stack (no Rust
  stack-overflow on deeply nested `each`), at the price of a slightly deeper
  heap-backed `self.frames`/`self.stack`. Hot `each` gets marginally faster.
- **The real cost is owned unwind bookkeeping.** The recursive `run_until`
  previously let a block's `return`/error ride Rust's own stack unwinding for
  free. Trampolining moves that bookkeeping into the VM (pop the right
  `CallFrame`s, land the value in the right slot, route errors to the fiber
  floor). This is the one genuine correctness risk and the bulk of the review
  surface — it touches the [ADR-0013](../accepted/0013-closure-upvalues-and-frame-token-return.md)
  frame-token return and [ADR-0008](../accepted/0008-layered-exceptions-and-result.md) error
  floor, the most load-bearing unwind machinery in the VM.
- **Two implementations of "invoke a block" coexist.** The trampolined
  `CallBlock` (Phalcom bytecode) and the re-entrant `block_call` primitive (native
  callers) must stay byte-for-byte equivalent in arity check, dummy receiver slot,
  and `home_frame_token` stamping. Standing maintenance tax; divergence is a
  latent bug class. Mitigation: `CallBlock` handles only the pure `Block`/`Closure`
  case and delegates everything else to the primitive.
- **Residue (retained Option A tail).** Not yield-transparent: reflective/dynamic
  `call` (§Decision 1), and blocks invoked from genuinely native Rust callers
  (§Decision 2). `CannotYieldAcrossNativeFrame` remains a real, catchable error on
  those paths — a strictly smaller set than under [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md)
  §4, and additively removable by the eventual full Option B.
- **No new floor surface, no GC commitment.** `CallBlock` is a dispatch-shape
  change, not a new native capability, so it does not amend the
  [ADR-0019](../accepted/0019-freeze-vm-blessed-primitive-floor.md) floor. Block activations
  remain VM `CallFrame`s inside the arena `FiberObject`
  ([ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md) §2/§7); no native
  fiber stacks are introduced, so [ADR-0009](../accepted/0009-handle-arena-heap.md)'s
  moving-ready arena claim is preserved. This is the promised additive A→B step,
  taken as one slice rather than the whole callback-primitive family.

## Alternatives considered

- **Runtime dispatch branch in `Invoke`** (trampoline whenever the receiver is a
  `Block`, no new opcode). Catches reflective/dynamic `call` too, closing the
  §Decision 1 residue — but taxes every message send with a receiver type-check on
  the VM's hottest path. Rejected for the hot-path cost; the compile-time opcode's
  residue is narrow and honest.
- **Full Option B now** (de-recurse `block_call`, `perform`, dNU forward, and every
  Rust combinator). Strictly more capable — closes the entire residue — but is the
  large invasive rewrite [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md)
  §Alternatives deferred. This amendment is the minimal high-leverage subset;
  Option B remains additively reachable from here.
- **`for`/cursor iteration ([ADR-0035](../accepted/0035-iteration-protocol-cursor.md)) — the
  chosen v0.2 resolution.** `for (x in coll) { Fiber.yield(x) }` lowers to an inlined
  `while` over the two-selector cursor protocol, emitting no `block_call`, so the
  yield suspends freely under [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md)
  §4 with zero VM/GC risk — and it also gives `break`/`continue`, which a block handed
  to `.each` cannot. This delivers the common generator ergonomic for v0.2 without this
  amendment, which is why ADR-0033 is Deferred. It does **not** close the residue: `for`
  cannot express a `.each { yield }`, a stored-block generator, or a user native
  combinator that yields — those remain the reason this amendment exists.
  ([ADR-0035](../accepted/0035-iteration-protocol-cursor.md) §4/§6 deliberately keep iteration
  selectors non-inlined and `.each { yield }` raising, so inlining `each` is **not** an
  available alternative — an earlier draft that did so was dropped for contradicting it.)
- **Do nothing (keep restricted Option A).** The [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md)
  §4 workaround stands. Rejected as the target end-state because the callback
  generator is the natural iteration idiom; but valid until the fiber unit is
  scheduled, since this amendment is bound to that unit by §Decision 4.
