# Call frames / activation records — source map

Read-only source map at HEAD. Oriented via `graphify explain "CallFrame"` /
`graphify affected "CallFrame"` / `graphify query` before reading; regions below
are the ones those queries pointed at.

## THE QUESTION THAT DOMINATES EVERYTHING

Candidate answers considered:
- (a) heap-allocated object with an owning caller/parent pointer (CPython
  `PyFrameObject.f_back`-style)
- (b) a `Copy` value stored by position in a flat `Vec`, no parent pointer
- (c) something else

**The code shows (b).** `phalcom-core/src/frame.rs::CallFrame` (~L65-66):

```rust
/// A single closure activation: its code handle, receiver, and stack window.
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
```

`#[derive(Debug, Clone, Copy)]` is the line that settles it: `CallFrame` is a
plain `Copy` struct, held in `Vec<CallFrame>` (`VM::frames`,
`FiberObject::frames` — see §5). There is no `Rc`, no `Box`, no arena handle
*to the frame itself* (its fields are handles into the heap, but the frame
struct is not itself a heap object).

**Is there a `parent`/`f_back`/caller-pointer field? NO.** Every field of
`CallFrame` (§1) is a `Copy` handle or primitive; none of them is "the
previous frame." — VERIFIED by reading the full struct (below).

**What encodes the caller relation instead, since there is no pointer:**
*position in the `Vec`.* The caller of the frame at index `i` is, by
construction, the frame at index `i - 1` in the same `Vec` — pushed
immediately before it, popped immediately after it returns (LIFO). The
"link" is implicit in Vec ordering, not stored in the struct at all. This is
possible only because the frame is `Copy`: nothing needs a parent pointer to
stay valid across a push/pop, because nothing borrows a frame — every read is
a by-value copy out of the `Vec` (e.g. `let popped = self.frames.pop().unwrap()`,
§4). Two things track *cross-frame* relationships instead of a parent pointer,
and both are auxiliary, not part of `CallFrame`:
- `stack_offset` (a field on the frame) locates *this* frame's own operand
  window in the shared value stack — not a pointer to another frame.
- `home_frame_token: Option<FrameToken>` (a field on the frame, §1) encodes
  which *other* activation a non-local `return` unwinds to — but it is a
  `(frame_index, generation)` pair compared against the live `Vec`, not a
  pointer, and it is populated only for block activations, not the general
  caller link.

## 1. `phalcom-core/src/frame.rs::CallFrame` (~L65-95)

Module-level doc (`frame.rs` ~L1-6) — VERIFIED, quoted in full:

```rust
//! Call frames and their receiver context.
//!
//! A [`CallFrame`] is a single method/closure activation. Because every link it
//! holds is now a `Copy` handle ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md))
//! the whole frame is `Copy`, so the VM keeps frames in a plain `Vec` with no
//! `Rc<RefCell<T>>` and no borrow-panic surface.
```

Full struct definition and derive (~L65-95) — VERIFIED, quoted in full:

```rust
/// A single closure activation: its code handle, receiver, and stack window.
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    /// Handle to the [`ClosureObject`](crate::heap::ClosureObject) executing.
    pub closure: ObjRef,
    /// The receiver context (`self`) for this activation.
    pub context: CallContext,
    /// Instruction pointer: an index into the closure's bytecode chunk.
    pub ip: usize,
    /// Index into the VM value stack where this frame's window begins (receiver
    /// then arguments).
    pub stack_offset: usize,
    /// Source span of the call site, for stack traces.
    pub caller_source: Option<SourceRange>,
    /// Monotonically-assigned generation for this activation.
    pub generation: u64,
    /// The home-frame token this activation returns *through* on a non-local
    /// `return`, or `None` for an ordinary method/closure activation.
    ...
    pub home_frame_token: Option<FrameToken>,
}
```

Field-by-field `Copy` confirmation:

| Field | Type | Copy? |
|---|---|---|
| `closure` | `ObjRef` | yes — `ObjRef` is an arena index (integer handle, ADR-0009) |
| `context` | `CallContext` | yes — `#[derive(Debug, Clone, Copy)]` on the enum itself (§2) |
| `ip` | `usize` | yes — primitive |
| `stack_offset` | `usize` | yes — primitive |
| `caller_source` | `Option<SourceRange>` | yes — `SourceRange` from `phalcom-common` is a plain span type |
| `generation` | `u64` | yes — primitive |
| `home_frame_token` | `Option<FrameToken>` | yes — `FrameToken` is itself `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` (frame.rs ~L18), so `Option<FrameToken>` is `Copy` too |

No field is `Rc<_>`, `Box<_>`, `RefCell<_>`, or a raw/native pointer of any
kind — every field is either a primitive or an `ObjRef`/`ClassId`-style arena
handle. **Confirmed: no caller/parent-pointer field exists.**

Tie to ADR-0009 (handle-arena-heap, §8): the whole reason `CallFrame` can be
`Copy` is that ADR-0009 replaced `Rc<RefCell<T>>` object links with `Copy`
integer handles (`ObjRef`) into a central `Heap` arena. `CallFrame::closure`
and the `ObjRef`s inside `CallContext` are exactly those handles. Because
dereferencing always goes through the `Heap` rather than through a pointer
embedded in the frame, the frame itself never needs `Rc`/`RefCell` to keep
its links valid — it can be copied freely, and the VM stores it in a plain
`Vec<CallFrame>` (§5) "with no `Rc<RefCell>` and no borrow-panic surface" (the
module doc, quoted above).

## 2. `phalcom-core/src/frame.rs::CallContext` (~L33-62)

All four variants — VERIFIED, quoted in full:

```rust
/// The receiver a frame is executing against.
#[derive(Debug, Clone, Copy)]
pub enum CallContext {
    /// Executing a method on a user-defined instance.
    Instance {
        /// Handle to the receiver instance.
        instance: ObjRef,
    },
    /// Executing a (static) method on a class.
    Class {
        /// Handle to the receiver class.
        class: ObjRef,
    },
    /// Executing top-level module code.
    Module {
        /// Handle to the running module.
        module: ObjRef,
    },
    /// Executing a closure-backed (non-primitive) method on an **immediate**
    /// receiver (`Bool`/`Number`/`Symbol`) — e.g. a user-defined sacred
    /// selector reopened onto the kernel `Bool` class (U5, ADR-0018: needed
    /// to make the sacred-selector inliner's override-epoch deopt guard
    /// exercisable, since only a closure method — never a primitive — needs
    /// a `CallContext` at all). Carries the receiver `Value` itself, since
    /// there is no [`ObjRef`] to point at.
    Immediate {
        /// The immediate receiver value.
        value: Value,
    },
}
```

Why `Immediate` carries a `Value` and not an `ObjRef` (ADR-0018 / U5): `Bool`,
`Number`, and `Symbol` receivers are *immediate* values in Phalcom's
representation — they live directly in a `Value` (unboxed), not as arena
objects with an `ObjRef` handle. `Instance`/`Class`/`Module` all point at
real heap objects, so `ObjRef` is the right handle for them. But when a
*user* reopens a closure (non-primitive) method onto the kernel `Bool` class
and it runs with `self` bound to `true`/`false`, there is no arena object
backing that receiver to hand an `ObjRef` to — so the variant stores the
`Value` directly. The doc comment ties this specifically to the sacred-selector
inliner's override-epoch deopt guard: that guard only needs to be exercised
when a *closure* method (not a primitive) sits behind a sacred selector on an
immediate receiver, which is precisely the case this variant exists to carry
a `CallContext` for at all.

## 3. PUSH: `phalcom-core/src/vm/dispatch.rs::VM::new_call_frame` (~L29-42)

Full body — VERIFIED, quoted:

```rust
/// Builds a [`CallFrame`] stamped with a fresh, monotonically-increasing
/// generation for the frame-token infrastructure
/// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
///
/// Every pushed activation gets its own generation so a [`BlockObject`]
/// created inside it can later (in U10) tell whether its home activation is
/// still live. U4 only mints and stores the token; it performs no unwinding.
pub(crate) fn new_call_frame(
    &mut self,
    closure: ObjRef,
    context: CallContext,
    ip: usize,
    stack_offset: usize,
    caller_source: Option<SourceRange>,
) -> CallFrame {
    let generation = self.next_frame_generation;
    self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
    let mut frame = CallFrame::new(closure, context, ip, stack_offset, caller_source);
    frame.generation = generation;
    frame
}
```

Generation bump — CONFIRMED: `self.next_frame_generation.wrapping_add(1)`
runs unconditionally every call, so every pushed activation gets a fresh
generation. `new_call_frame` only *builds* the frame; the caller is
responsible for pushing it onto `self.frames` — three call sites do this
(method send, block invocation, fiber entry; see §7's table). This function
itself does not push. Per the task's scope note, the generation/token's role
in non-local-return / `DeadFrameError` is the later frame-identity doc's
territory — not expanded further here beyond confirming the bump happens.

## 4. POP / UNWIND

**Return** — `dispatch.rs::VM::run` (Bytecode::Return handler, ~L1093-1109),
VERIFIED, quoted:

```rust
Bytecode::Return => {
    // A bare `return;` (no operand) or a method that produced no
    // value yields `None`, not the private sentinel
    // (values-and-absence.md; bare-`return`→`None` pre-authorized).
    let return_value = self.stack.pop().unwrap_or(Value::Nil);
    let return_value = self.surface_absence(return_value);
    let popped = self.frames.pop().unwrap();
    // Close any upvalues that still alias this frame's window so
    // escaping closures survive the frame's disappearance
    // (ADR-0013). Must run before the stack is truncated.
    self.close_upvalues_from(popped.stack_offset);
    self.stack.truncate(popped.stack_offset);
    if self.frames.len() <= base_frames {
        return Ok(return_value);
    }
    self.stack.push(return_value);
}
```

**`unwind_to`** — `dispatch.rs::VM::unwind_to` (~L110-114), VERIFIED, quoted
in full:

```rust
pub(crate) fn unwind_to(&mut self, stack_len: usize, frames_len: usize) {
    self.close_upvalues_from(stack_len);
    self.frames.truncate(frames_len);
    self.stack.truncate(stack_len);
}
```

**Confirmed: no relinking of any pointer on pop.** `self.frames.pop()`
removes the last `Vec` element and returns it by value (`CallFrame` is
`Copy`); the caller simply "resumes" because it is already sitting at the new
last position of `self.frames` — nothing writes into it, nothing patches a
field on it. Same for `unwind_to`'s `self.frames.truncate(frames_len)`: it
drops every element past index `frames_len`, and whatever is left at the new
end *is* the resumed caller, by Vec position alone. This is the direct
consequence of §"THE QUESTION": there is no parent pointer to relink because
there never was one.

(A third truncate exists at `ReturnNonLocal`, ~L1160,
`self.frames.truncate(token.frame_index)` — it is the same mechanism, jumping
past multiple frames at once using the frame-token's stored index. Per the
task's scope, the non-local-return mechanism itself is left to the
frame-identity doc; it is mentioned here only to note it's the same
Vec-truncate pop, not a different unwind primitive.)

## 5. WHERE FRAMES LIVE

`phalcom-core/src/vm/mod.rs::VM::frames` (~L45-53) — VERIFIED, quoted:

```rust
/// The active call stack, innermost frame last. [`CallFrame`] is `Copy`.
///
/// This is the **live mirror** of the currently-[`Object::Fiber`]-running
/// fiber's own `frames` buffer ([`crate::heap::FiberObject`],
/// [ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md)
/// §3, D-FIB-4): while [`Self::current`] runs, its state lives here; a
/// fiber switch stores this back into the parking fiber and loads the
/// resuming fiber's state in, an O(1) pointer-free copy (a `Vec` swap).
pub(crate) frames: Vec<CallFrame>,
```

`phalcom-core/src/heap/fiber.rs::FiberObject::frames` (~L68-72) — VERIFIED,
quoted:

```rust
/// The fiber's private call stack (empty while running — mirrored by
/// [`VM::frames`](crate::vm::VM)). Because the frame-generation counter
/// stays VM-global (D4), a non-local `return` token whose home lives on
/// another fiber fails the generation check → `DeadFrameError`.
pub frames: Vec<CallFrame>,
```

Relationship (3-4 lines): `VM::frames` is the live working buffer the
dispatch loop actually reads/writes while a fiber is running; each
`FiberObject::frames` is that same fiber's *parked* storage, which sits empty
while the fiber is current and holds its full frame stack while parked. A
fiber switch is a `Vec` swap between the two (`VM::frames` ⇄
`FiberObject.frames`) — "an O(1) pointer-free copy," per the doc above — not
a copy of individual frames or any relinking.

`stack_offset` (`frame.rs` ~L73-75, quoted in §1) is an index into
`VM::stack`/`FiberObject::stack`, the shared value stack — but "shared" means
shared *within one fiber's stack buffer*, not globally: it is
**fiber-relative**. `fiber.rs`'s doc on `FiberObject::stack` (~L63-66)
states this directly (VERIFIED, quoted):

```rust
/// The fiber's private operand stack (empty while running — mirrored by
/// [`VM::stack`](crate::vm::VM)). `stack_offset`s are window-relative
/// (frame.rs, D3), so a per-fiber stack always based at index 0 needs no
/// rebasing on switch.
pub stack: Vec<Value>,
```

and the sibling comment on `FiberObject::open_upvalues` (~L73-77) spells out
*why* frames/stack/upvalues must swap together rather than living in one
global pool (VERIFIED, quoted):

```rust
/// The fiber's private open-upvalue map, keyed by absolute value-stack
/// index (empty while running — mirrored by
/// [`VM::open_upvalues`](crate::vm::VM)). Kept per-fiber because it is
/// stack-index-keyed and each fiber has its own stack; swapping it with
/// `stack`/`frames` prevents a cross-fiber slot-index collision.
```

i.e. a `stack_offset` of, say, `3` means "slot 3 of *this fiber's* value
stack" — if two fibers' stacks were merged into one flat index space, or if
only `frames` swapped but not `stack`, the same numeric offset could
address unrelated slots in a different fiber. Swapping frames+stack+upvalues
together as one atomic unit is what keeps `stack_offset` meaningful.

## 6. GC — frames as roots

`phalcom-core/src/heap/trace.rs::trace_frame` (~L37-52) — VERIFIED, quoted
(short excerpt):

```rust
pub fn trace_frame(frame: &CallFrame, push: &mut impl FnMut(ObjRef)) {
    push(frame.closure);
    match frame.context {
        CallContext::Instance { instance } => push(instance),
        CallContext::Class { class } => push(class),
        CallContext::Module { module } => push(module),
        CallContext::Immediate { value } => {
            if let Some(id) = value.as_obj() {
                push(id);
            }
        }
    }
}
```

`phalcom-core/src/vm/gc.rs::VM::collect_roots` (~L32-93) — VERIFIED, the call
site:

```rust
for frame in frames {
    trace_frame(frame, &mut |id| out.push(id));
}
```

(`frames` here is `collect_roots`'s destructured binding of `VM::frames`,
per the doc at gc.rs ~L37-40: "`VM::frames`/`stack`/`open_upvalues` are the
*authoritative mirror* of `current`'s own buffers ... so rooting the mirror
is what keeps the running fiber's objects alive, not tracing its
`FiberObject`.")

Confirmed in 2-3 lines: the collector walks `vm.frames` (the live mirror,
§5) and calls `trace_frame` on each one, which pushes that frame's `closure`
handle and whichever `ObjRef` its `CallContext` variant carries (or nothing,
for an `Immediate` context with no backing object) as GC roots. This is how
every live activation keeps its executing closure and receiver reachable.

## 7. Every use site of `CallFrame`

From `graphify affected "CallFrame"` plus a follow-up grep for
`CallFrame`/`new_call_frame(` (graphify's traversal surfaced the files;
line numbers below are from direct reads):

| Symbol | file:line | What it does with a frame |
|---|---|---|
| `CallFrame` struct + `CallContext` enum | `frame.rs:19,35,66` | Type definitions (§1, §2) |
| `VM::new_call_frame` | `vm/dispatch.rs:29` | Builds a fresh `CallFrame`, stamping a new generation (§3) — does not push |
| `VM::current_frame_token` | `vm/dispatch.rs:50` | Reads `self.frames.last()` to get the innermost frame's token, for stamping new `BlockObject`s |
| `VM::unwind_to` | `vm/dispatch.rs:110` | `frames.truncate` + `stack.truncate` — bulk pop (§4) |
| `VM::runtime_error` | `vm/dispatch.rs:121-142` | Clones `self.frames`, iterates `.rev()`, builds `SourceLoc`s (module/method/span per frame) for a trace; calls `print_rt` |
| `Bytecode::Return` handler | `vm/dispatch.rs:1093-1109` | `self.frames.pop()` — the ordinary per-call pop (§4) |
| `Bytecode::ReturnNonLocal` handler | `vm/dispatch.rs:1110-1161` | `self.frames.truncate(token.frame_index)` — bulk pop past multiple frames using a stored `FrameToken` |
| `VM::run_in_module` | `interpret.rs:163-176` | Manually constructs and pushes the **entry frame** (`CallContext::Module`) — the one push site that does not go through `new_call_frame`/`send.rs`/`block.rs` |
| `VM::call_method` (ordinary method send) | `vm/send.rs:113` | `self.new_call_frame(...)` then pushes it — the method-call push site |
| `block_call` primitive | `primitive/block.rs:143-152` | `vm.new_call_frame(...)`, sets `frame.home_frame_token`, then `vm.frames.push(frame)` — the block-invocation push site |
| Fiber entry (`Fiber.call`/resume) | `primitive/fiber.rs:304` | `vm.new_call_frame(...)` for the fiber's first activation — the fiber-entry push site |
| `VM::frames` field | `vm/mod.rs:53` | The live working `Vec<CallFrame>` (§5) |
| `FiberObject::frames` field | `heap/fiber.rs:72` | Per-fiber parked backing store (§5) |
| `VM::fiber_pool` (`fiber-pool` feature) | `vm/mod.rs:233` | `Vec<(Vec<Value>, Vec<CallFrame>)>` — a free-list for recycling fiber stack/frame buffers; doc notes it measured **net negative** in benchmarking and is gated off by default |
| `trace_frame` | `heap/trace.rs:37` | Pushes a frame's `closure`/`context` handles as GC roots (§6) |
| `VM::collect_roots` | `vm/gc.rs:93` | Calls `trace_frame` on every frame in `vm.frames` (§6) |
| `error.rs` doc reference | `error.rs:143` | Doc-comment cross-reference to `CallFrame::home_frame_token` (no code use) |

## 8. Spec/ADR (bounded)

**ADR-0009 (handle-arena-heap)** — `docs/adr/accepted/0009-handle-arena-heap.md`:

- **Decision:** Objects live in a central `Heap`, referenced by `Copy`
  integer handles (`ObjRef` for heap objects, `ClassId` for classes) — "no
  `Rc`, no `RefCell`, no `MaybeWeak`." This is the decision that makes every
  field `CallFrame` holds a `Copy` handle, which is in turn what makes
  `CallFrame` itself derivable as `Copy` (§1).
- **Alternatives considered:** (1) keep `Rc<RefCell<T>>` + `Weak`
  cycle-breaker — rejected because the weak path was inert (kernel cycle
  never freed) and `RefCell` is a double-borrow-panic surface; (2) an
  immediate tracing `Gc<T>` — most Smalltalk-faithful but rejected as too
  much scope up front, with the handle heap chosen precisely so a tracing GC
  (§6 shows one now exists) could be layered on later without an API break.

**ADR-0013 (closure upvalues and frame-token return)** —
`docs/adr/accepted/0013-closure-upvalues-and-frame-token-return.md`: this is
the ADR that owns `FrameToken`/`generation`/`DeadFrameError` (the "home frame
plus generation counter, compared on non-local `return`" mechanism). Per this
task's scope, that mechanism is the later frame-identity doc's territory —
noted here only as the ADR of record; not summarized further.

## 9. BEHAVIOURAL — run, don't predict

**DeadFrameError — RAN, actual output verbatim.** Used the existing golden
fixture `phalcom-core/tests/lang/runtime-errors/runtime_non_local_return_dead_frame.ph`
(a block that captures `return`, escapes its home method via `Maker.make()`,
then is invoked separately — no live home frame to unwind to):

```
cargo run -q -p phalcom-core --bin phalcom phalcom-core/tests/lang/runtime-errors/runtime_non_local_return_dead_frame.ph
```

Actual stderr output (exit code 1):

```
non-local return from a block whose home method frame is no longer alive (DeadFrameError)
```

This is evidence the frame-identity check exists and fires — the mechanism
itself (comparing `FrameToken.generation` against the live frame) is left to
the later frame-identity doc per the task's scope.

**Nested-call frame order — RAN, actual output, with a documented surprise.**
Wrote a small `.ph` program (`A.outer()` → `A.middle()` → `A.inner()` →
`throw Error.new(...)`, three nested user-method frames plus the module
frame) and ran it through the actual `phalcom` CLI binary
(`./target/debug/phalcom <file>.ph`).

**Actual observed output (stderr, verbatim):**
```
boom-from-inner
```

No multi-frame trace was printed — just the flat error message. This was
initially surprising given `VM::runtime_error` (§7) clearly walks
`self.frames.clone().iter().rev()` to build a `SourceLoc` per frame and calls
`print_rt`, which prints a `"Traceback (most recent call last):"` header
followed by one entry per frame. Traced *why*, reading the actual call
chain — VERIFIED:

- The `phalcom` CLI's `cmd_run` (`phalcom-core/bin/phalcom/cli.rs:161-164`)
  calls `vm.run_in_module(module, closure)` directly and, on error, just does
  `eprintln!("{e}"); std::process::exit(1);` — it never calls
  `VM::runtime_error`/`print_rt` at all.
- `VM::runtime_error` is instead invoked from `VM::interpret_source`
  (`phalcom-core/src/interpret.rs:186-199`, `run_in_module(...).inspect_err(|err| { let _ = self.runtime_error(err.clone()); })`).
- `grep`-confirmed callers of `interpret_source`: only
  `phalcom-core/tests/gc.rs`, `phalcom-core/tests/invariants.rs`, and
  `phalcom-core/benches/vm_bench.rs` — all Rust-level test/bench harnesses,
  none of them the CLI binary.
- The golden test harness (`phalcom-core/tests/golden.rs:48-65`) spawns the
  compiled `phalcom` binary as a subprocess and asserts only on its stdout,
  matching what was observed: the binary's own error path never reaches
  `runtime_error`/`print_rt`, so no golden fixture's `.expected` file
  contains a multi-frame trace either (confirmed by reading
  `runtime_error_throw_uncaught.expected`, which contains only `boom`).

**So:** the multi-frame stack-trace-building code (`runtime_error` walking
`self.frames` in reverse, `print_rt`'s "Traceback" rendering) exists and is
correct-looking, but at HEAD it is **not reachable through the `phalcom` CLI
binary** — only through `interpret_source`, used solely by the Rust-level
test/bench harnesses. This is a verified fact about the current wiring, not
a claim about the frame stack's *capability* (the data needed for a full
trace — one `SourceLoc` per live frame, correctly ordered by `Vec` position —
is present and correct in `self.frames` at the point of an uncaught error;
it is only the CLI's error-reporting call chain that doesn't use it).
