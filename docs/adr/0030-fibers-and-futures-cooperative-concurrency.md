# 30. Fibers and Futures: cooperative concurrency on a restricted re-entrant loop

- Status: Accepted
- Date: 2026-07-12
- Related: [`docs/spec/v0.2/concurrency.md`](../spec/v0.2/concurrency.md);
  [`docs/spec/v0.2/core/forward-compat.md`](../spec/v0.2/core/forward-compat.md) §7;
  [ADR-0009](0009-handle-arena-heap.md) (handle heap);
  [ADR-0010](0010-tagged-value-enum.md) (tagged `Value`);
  [ADR-0013](0013-closure-upvalues-and-frame-token-return.md) (frame-token return);
  [ADR-0008](0008-layered-exceptions-and-result.md) (one unwind primitive);
  [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (floor)

## Context

`Fiber`/`Future` was the only major subsystem fully specified
([`concurrency.md`](../spec/v0.2/concurrency.md)) yet ADR-less, and its execution
model was the last **genuinely-open decision** blocking the fiber unit
([`deferred-work.md`](../spec/v0.2/deferred-work.md) §2). Two things were
unrecorded and load-bearing:

1. **How fibers execute on the existing VM.** The
   [forward-compat §7](../spec/v0.2/core/forward-compat.md) code-grounded audit
   found that the VM dispatch loop is **re-entrant across native frames**: pure
   Phalcom→Phalcom sends are trampolined (one `run_until` loop, no native
   recursion — `vm.rs` `call_method` Closure arm), but every path where a Rust
   primitive needs a synchronous `Value` back from Phalcom code
   (`block_call`, `send_dynamic`/`perform`, `forward_does_not_understand`, and
   transitively every collection combinator that calls a block — `List.each`,
   `Option.map`, `reduce`) **re-enters `run_until` recursively, growing the native
   Rust stack**. When the running fiber is inside such a primitive, native Rust
   frames sit between the fiber's entry and the `Fiber.yield` call site, and those
   frames *are* the fiber's suspended position — you cannot repoint a handle and
   return through them without destroying it. This is the design-space crown-jewel
   hazard **native-stack frames ⊗ suspendable control**.

2. **Which already-landed VM decisions the fiber machinery must preserve.** The
   audit verified seven (D1–D7) against the tree and surfaced invariants (e.g. the
   VM-global frame generation counter) that a pre-fiber refactor must not break.

The surface (`Fiber`, `Future`, `async`/`await`, scheduler) is settled in the
spec. This ADR ratifies it *and* fixes the execution model, so the fiber unit can
be scheduled.

## Decision

**Cooperative, single-threaded fibers on a restricted re-entrant loop (audit
Option A / Lua-5.1 style), with `Future` as a pure library layer.**

### 1. `Fiber` is the sole concurrency primitive

Cooperative, single-threaded, no preemption. `Future`, `async`/`await`,
generators, and the scheduler all derive from it. No data races by construction;
no locks in the object model. A running fiber runs until it explicitly `yield`s,
`await`s, returns, or raises. The surface is [`concurrency.md`](../spec/v0.2/concurrency.md)
§1–2 (`new`/`call`/`try`/`yield`/`current`/`abort`; `Future.async`/`await`/`then`).

### 2. `Fiber` is a heap object, not a new `Value` arm

A `FiberObject` is one more arena variant (`Object::Fiber`, [ADR-0009](0009-handle-arena-heap.md))
reached through `Value::Obj(ObjRef)`, exactly as native `List` is — **no new
`Value::Fiber` arm** ([ADR-0010](0010-tagged-value-enum.md)). It owns its own
`stack: Vec<Value>` and `frames: Vec<CallFrame>`, a `status`, a `resumer`, a
result slot, and its entry closure. (This supersedes `concurrency.md` §1's older
`Value::Fiber(PhRef<FiberObject>)` phrasing, which predates the handle heap.)

### 3. Fiber switch is an O(1) pointer swap

The VM's "current stack / current frames" relocate behind a `current: ObjRef`
into the running `FiberObject`; `call`/`yield` swap **which fiber the dispatch
loop reads**, never copying stacks. `CallFrame.stack_offset` stays
**frame-relative**, so per-fiber stacks starting at 0 need no rebasing.

### 4. Execution model — **restricted (Option A)**

`Fiber.yield` integrates only with the **top-level** `run_until`. If the running
fiber tries to yield while a re-entrant primitive (a native `block_call` and
everything above it) is on the native Rust stack, the VM raises
**`CannotYieldAcrossNativeFrame`** (a thrown `Error`, per §7 below), rather than
corrupting the suspended position.

- **Works:** any fiber whose body uses pure Phalcom sends and **inlined** control
  flow. The [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md) inliner
  lowers `while`/`ifTrue:` to `Jump`/`Loop` opcodes within one chunk — no frame
  push, no native frame — so the canonical generator suspends freely:

  ```phalcom
  Fiber.new { let n = 0; while (true) { Fiber.yield(n); n = n + 1 } }   // ✅
  ```

- **Foreclosed under A:** the *callback generator*, where `yield` sits under a
  native combinator's `block_call`:

  ```phalcom
  Fiber.new { list.each { x => Fiber.yield(x) } }   // ✗ CannotYieldAcrossNativeFrame
  ```

  Rewrite with index iteration (`while (i < list.size) { Fiber.yield(list.at(i)); … }`).

### 5. The fiber-switch signal is typed, not a length delta

`call`/`yield` reconcile with the dispatch loop through an explicit
`ControlFlow`/switch value out of the primitive — **not** the `frames.len()`
heuristic that the primitive arm currently uses to detect a non-local return
(`vm.rs` D5). A fiber switch also changes `frames.len()`; conflating the two would
misread a swap as a return. This keeps the eventual Option-B lift clean.

### 6. Non-local return and unwind stay fiber-local

Once `self.frames` is the *current* fiber's vector, [ADR-0013](0013-closure-upvalues-and-frame-token-return.md)'s
`ReturnNonLocal` searches only that fiber; a token whose home is on another fiber
fails the generation check → `DeadFrameError`. **Invariant:** the VM-global
monotonic `next_frame_generation` counter **must not** be relocated into
`FiberObject` — it is the only thing making a cross-fiber token globally
non-matching. Likewise the [ADR-0008](0008-layered-exceptions-and-result.md)
error unwind operates on `self.frames` only and stops at the **fiber floor**, so a
failing fiber captures its `Error` into its result slot instead of terminating the
host.

### 7. Fibers are GC roots even when parked

**Invariant (before any tracing/compacting GC lands):** a `FiberObject`'s value
stack and frame stack are GC roots for as long as the fiber is reachable and not
`done`/`failed` — **not only** the `current` fiber's. A collector that scans only
`current` would free objects held solely by a parked fiber. Keeping the stacks
*inside the arena object* (§2) is what lets the future collector reach them.

## Consequences

- **Smallest correct step.** Option A is a small VM change (a typed switch signal
  honored by the top loop) with **no collection rewrite**, and it keeps the spec's
  own canonical example working.
- **Monotonic upgrade path.** A → **B (full trampoline)** is purely *additive* —
  de-recursing the callback primitives later just removes the guard. Shipping A
  forecloses nothing. (A → C is not additive; see Alternatives.)
- **GC design stays open.** No native fiber stacks means nothing new for a moving
  collector to scan or relocate; [ADR-0009](0009-handle-arena-heap.md)'s
  moving-ready arena claim is preserved intact.
- **New floor surface (ADR-0019 amendment).** The Fiber/Future primitive set —
  `call`/`yield`/`current`/`abort`, a `Yield` opcode, per-fiber stack machinery,
  and the scheduler hooks exposed through [`System`](../spec/v0.2/system.md) — is a
  deliberate extension of the frozen floor, authorized here per the
  [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) amendment convention (as
  [ADR-0020](0020-kernel-list-native-array-protocol.md)/[ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md)
  did for `List`/`hash`). `Future` adds **no** VM mechanism beyond `Fiber` + a
  ready-queue.
- **`CannotYieldAcrossNativeFrame` is a real, catchable error** users can hit; the
  spec documents the restriction and the index-iteration workaround.
- Five pre-fiber invariants (§5–§7 plus frame-relative `stack_offset` and
  no-new-`Value`-arm) bind every unit that touches the VM before the fiber unit
  lands ([forward-compat §7.3](../spec/v0.2/core/forward-compat.md)).

## Alternatives considered

- **B — full trampoline (yield anywhere).** De-recurse *every* callback primitive
  (`block_call`, `each`, `map`, `reduce`, `perform`, dNU forward) so they push work
  onto the VM frame stack instead of calling `run_until`. Strictly more capable
  than A, but a large, invasive rewrite of the primitive/callback protocol. **Not
  now** — reachable additively from A once a real need for callback generators
  appears.
- **C — stackful coroutines.** Give each fiber a real native stack
  (`corosensei`/`makecontext`-style switch) so `yield` crosses native frames.
  Rejected: it adds an `unsafe` stack-switch dependency and **permanently
  constrains the GC** — every parked fiber's native stack becomes a root the future
  moving collector must scan/relocate (crown-jewel *stackful-fiber ⊗ moving-GC*),
  directly weakening [ADR-0009](0009-handle-arena-heap.md). The power is not worth
  an irreversible GC commitment.
- **Preemptive / multithreaded fibers.** Rejected — would require a memory model
  and locks throughout the object model; the singular cooperative primitive is the
  whole point.
- **Resumable (Smalltalk) suspension for failures.** Out of scope; error
  propagation is terminating per [ADR-0008](0008-layered-exceptions-and-result.md).
