# Frames

> **A frame is a value, not an object.** It is a flat `Copy` record living in a plain `Vec`, so
> the call stack is an array you **truncate**, not a linked list you unlink — and the caller is
> just the record one slot down, never a pointer the frame carries.

[The Execution Loop](execution-loop.md) showed the VM as one `while` loop over a `match`. [The
Compiled Artifact](compiled-artifact.md) showed *what* runs: a `Callable` recipe, instantiated
into a `ClosureObject`. This doc closes the gap between them — **where** a closure runs. Each time
the loop enters a closure, it opens one *activation*: a scratch space holding this call's receiver,
its locals, and where to resume the caller. That activation is a **frame**. The whole doc is about
one question, and it is not "does a VM need frames" (obviously it does) — it is **what kind of
thing is a frame, in memory?**

That question has three real answers, and the choice forecloses whole categories of feature
(reflection, coroutines, cheap fibers) before any syntax is designed. Phalcom's answer is
forced by a decision made elsewhere — the handle arena — and once you see that, the frame's shape
is re-derivable rather than arbitrary. That is the target: by the end, you should be able to throw
away `frame.rs` and rebuild `CallFrame` from two constraints.

## What one activation must hold

Strip a call to the state that is genuinely *per-activation* — not per-method, not global:

- **where to resume** — a saved instruction pointer, plus *which* frame to hand the result back to;
- **the receiver** — `self` for the duration (or a module, for top-level code);
- **a window of locals and temporaries** — the arguments, named locals, and the evaluator's
  scratch slots. Its size is a *static* property of the method body (a fixed max), which is the
  fact that makes one of the three representations possible at all;
- **a link to the caller** — who resumes when this returns. This is the single field where the
  three designs genuinely diverge. "How is the caller found" is almost the whole design question.

Calls nest — if `a` calls `b` calls `c`, then `c` finishes before `b`, and `b` before `a` — so the
activations live on a **stack**, LIFO. That much is textbook, and Phalcom did not deliberate it; it
is the shared premise of all three branches below.

## The fork: what *kind* of thing is a frame?

Three coherent engineering positions. Take each on its own terms before billing it — a strawman
would teach the answer without the question.

### (a) A heap object, linked to its caller by a pointer

Make a frame an ordinary heap object, managed by the same GC as everything else, reachable through
an ordinary reference. An enormous amount falls out *for free*, because "object" already comes with
identity, arbitrary lifetime, and reflectability. A frame can outlive its call — someone just holds
the reference. A debugger walks object references instead of needing a bespoke stack-walk protocol.
"What is running, who called it, who called *that*?" is an ordinary query over ordinary data: follow
the parent pointer. Generators and coroutines become almost trivial — "suspend an activation" is
just "stop touching this object for a while," which the GC already supports.

This is **CPython's** classic design: a `PyFrameObject` per call, linked to its caller by an
`f_back` field, with `sys._getframe()` handing your program a live reference to exactly that chain.

The bill: every call is now a heap allocation and every return makes it garbage — allocation
pressure at the highest-frequency event in the system. And the parent-pointer chain is a
self-referential linked list the runtime mutates while walking it: a node owned by whoever holds it
that *also* points back at (transitively) its owner. That is precisely the shape a Rust-style borrow
checker is worst at. It can be done — `Rc<RefCell<Frame>>` exists for exactly this — but every such
escape hatch re-adds at runtime the bookkeeping (refcounts, borrow checks, weak-vs-strong to break
the cycle) that a flat array simply never needs.

### (b) A flat array of value-records — windows carved from one shared array

Because a call's frame size is known *before* the call (static property of the callee), there is no
reason to ask an allocator for memory per call. Pre-reserve one big contiguous array; a call
"carves out the next *N* slots"; a return moves the cursor back. No allocator on the hot path. It is
cache-friendly — deep call chains touch adjacent, recently-written memory *(this is the array
branch's general property, an argument for the design; Phalcom has not published a frame-locality
measurement, so read it as reasoning, not a benchmark)*. And **"who called this" is just "whatever
is one slot below in the same array"** — array position *is* the caller relationship, for free, a
direct consequence of the LIFO nesting.

This is **Lua's** shape: one shared array of values per state is the register file every activation
draws a base/top window from.

The bill: a frame here is *not* a first-class value the language can hand around. It has no identity
that survives the next call reusing its slots. Any introspection — a stack trace, a `caller()`
builtin — must be a **side channel that copies information out**, not a live handle into the array.
Lua shows the seam exactly: `debug.getinfo(level)` hands back a fresh table *snapshot*, never a
reference to activation N, because activation N is not a value with identity in Lua's object model.
And a value that legitimately wants to name "that specific past activation, even after it popped" (a
non-local return target, a paused generator) needs a *separate identity mechanism* layered on top —
a generation counter or token that tells "the record in slot 12 now" from "a different record that
sat in slot 12 three calls ago." That machinery is real and this branch owes it. **(Hold that
thought — Phalcom has exactly such a token, and it is [Lie #1](#lie-1) of this doc.)**

### (c) The native machine stack — no bookkeeping at all

Don't build a frame abstraction: let each language call *be* a host call — a recursive `eval` in C,
or a real `call` instruction. The host's calling convention already has a return address, local
storage, and correct unwinding. Cheapest possible option, at native speed. This is what most
**tree-walking interpreters** do.

The bill: there is no data structure to inspect because there is no data structure. Reflection is
closed off — you cannot address "the current frame" as a value; it was never reified. Worse, it
*fights fibers*: suspending one logical thread and resuming another on the same OS thread means
swapping the native stack itself, which is not portable. And the GC has no frame abstraction to
consult, so it must scan the native stack conservatively or make the compiler emit precise
root metadata — the exact bookkeeping this branch was avoiding, moved elsewhere. For a language
with fibers and a tracing GC — both of which Phalcom has — branch (c) is a non-starter.

### Where these branches came from — and the honest caveat

The three-way walk above is **pedagogical scaffolding**. Phalcom did not run a frame-representation
bake-off and pick (b). There is no ADR titled "how to represent a frame." What actually happened is
narrower and is the subject of the next section: a *different* decision — how the object graph is
stored — made the frame's representation a foregone conclusion. Keep the fork in mind as the space
of what *could* have been; do not mistake it for a decision log.

## Predict before you read

Here is the whole design question as one prediction. **CPython's frame carries `f_back`, a pointer
to its caller, so a frame can walk its own call chain. Phalcom's frames live in a `Vec`. Does
`CallFrame` carry a caller pointer — a `parent`, an `f_back`, a `caller: &CallFrame`?**

Answer before scrolling. The grip already contains the answer, and a reader who derives it owns the
representation instead of being handed it.

...

**No.** There is no caller field. The `Vec`'s *position* is the chain: `frames[i-1]` is the caller
of `frames[i]`. If you predicted "no, because the array already orders them," you have re-derived
branch (b) from the LIFO premise — which is exactly the move the rest of the doc formalizes.

## Phalcom's frame is a `Copy` value — and here is why it had to be

```rust
/// A single closure activation: its code handle, receiver, and stack window.
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    pub closure: ObjRef,                        // the ClosureObject executing (Doc 2)
    pub context: CallContext,                   // the receiver — self
    pub ip: usize,                              // instruction pointer into the chunk
    pub stack_offset: usize,                    // where this frame's window starts in the value stack
    pub caller_source: Option<SourceRange>,     // call-site span, for traces
    pub generation: u64,                        // ← Lie #1
    pub home_frame_token: Option<FrameToken>,   // ← Lie #1
}
```

`frame.rs::CallFrame` (~L66). The `#[derive(..., Copy)]` on the top line is the whole story. **Every
field is itself `Copy`**: `ObjRef` is an integer index into a heap arena (not a pointer);
`CallContext` is a `Copy` enum of handles; `ip`/`stack_offset`/`generation` are integers;
`FrameToken` is `Copy`, so `Option<FrameToken>` is too. Nothing is `Rc`, `Box`, `RefCell`, or a raw
pointer.

Why does that matter? Because a `Copy` record can live **by value in a plain `Vec<CallFrame>`** with
no `Rc<RefCell<Frame>>` and — the module doc says this outright (`frame.rs` ~L1) — **no
borrow-panic surface**. Nothing borrows a frame across a push or pop; every read copies the value
out. That is what makes "no caller pointer" *safe*: in branch (a), the reason you reach for
`Rc<RefCell>` is to keep a parent link valid while the runtime mutates both ends. Delete the
parent link and the whole hazard evaporates.

And here is the re-derivation the doc promised. The frame is `Copy` **because every link it holds is
a `Copy` handle** — and *that* is not a frame decision at all. It is **ADR-0009 (the handle arena)**:
objects live in a central `Heap` referenced by `Copy` integer handles (`ObjRef`), explicitly to
kill the `Rc<RefCell>` borrow-panic surface and the inert cycle-breaker the old model carried.
ADR-0009's rejected alternatives were "keep `Rc<RefCell>`" and "a full tracing `Gc<T>` now." It says
nothing about frames. But once *every object reference is a `Copy` integer*, a frame built from those
references is `Copy` too — for free — and branch (b) becomes not just available but the path of no
resistance. **Two constraints — "references are `Copy` handles" and "the VM must own its call stack"
— force the frame to be a value in an array.** You did not need `frame.rs` to predict its shape;
you needed ADR-0009.

## The lifecycle: push, resume, and the payoff — truncate, don't unlink

A frame is a **register window**: two arrays working together. The small **frame array**
(`VM::frames`) holds the fixed-shape records above; the big shared **value stack** (`VM::stack`)
holds every activation's locals and temporaries, back to back. A frame's `stack_offset` marks where
*its* window begins in the value stack.

```
  frame array (VM::frames: Vec<CallFrame>)          value stack (VM::stack: Vec<Value>)
  ┌──────────────────────────────┐                  ┌───────────────────────────────────┐
  │ [0] module   off=0           │───window────────▶│ 0: (module's slots)               │
  │ [1] a()      off=3           │───window───┐     │ 3: self, arg, local ...           │  ← a()'s window
  │ [2] b()      off=7           │──window─┐  └────▶ │ 7: self, local ...                │  ← b()'s window
  │ [3] c()      off=10  ◀── top │─win─┐   └───────▶ │ 10: self ...                      │  ← c()'s window
  └──────────────────────────────┘     └───────────▶ │ 12: (c's live temporaries)        │
        caller of [3] is [2]:                        └───────────────────────────────────┘
        one slot down. No pointer.
```

- **Push (on a call).** `dispatch.rs::VM::new_call_frame` (~L29) builds the record — receiver,
  `ip = 0`, and a `stack_offset` equal to where the caller's window ends — then a call site pushes
  it onto `VM::frames`. Note `new_call_frame` *builds but does not push*: four sites push (an
  ordinary method send in `send.rs`, a block invocation in `primitive/block.rs`, a fiber's entry
  activation in `primitive/fiber.rs`, and the module entry frame in `interpret.rs`). Every local
  access the callee makes is then `stack_offset + i` for a compiler-fixed `i` — locals and "the
  shared value stack" are the same memory, addressed as disjoint adjacent ranges.

  <a id="lie-1"></a>**Lie #1.** `new_call_frame` also does `self.next_frame_generation.wrapping_add(1)`
  and stamps the frame's `generation`; a block invocation additionally copies a `home_frame_token`
  into the frame. Treat `generation` and `home_frame_token` as *just fields* for now. Their real
  job — giving a popped-and-reused slot a distinguishable identity so a non-local `return` can find
  its home activation or fail cleanly with `DeadFrameError` — is the identity mechanism branch (b)
  owes, and it is the entire subject of **[Doc 6 (frame identity)](frame-identity.md)**. That is the generation counter
  the fork warned this branch would need.

- **Return (`Bytecode::Return`, ~L1099).** `let popped = self.frames.pop().unwrap();` then
  `self.stack.truncate(popped.stack_offset)` — drop this frame's window, keep everything below
  (the caller's). The new top of the frame array *is* the caller, automatically. Nothing was
  relinked, because there was never a link: "the record one slot down" was the caller pointer,
  structurally, from the moment it was pushed.

- **Unwind (the payoff).** This is where branch (b)'s *shape*, not just its per-call cost, earns
  the design. Discarding many activations at once — an exception, a non-local return past a hundred
  nested calls — is `dispatch.rs::VM::unwind_to` (~L110):

  ```rust
  pub(crate) fn unwind_to(&mut self, stack_len: usize, frames_len: usize) {
      self.close_upvalues_from(stack_len);
      self.frames.truncate(frames_len);   // every intervening frame gone in one length update
      self.stack.truncate(stack_len);
  }
  ```

  A hundred frames vanish in one `truncate`. Contrast branch (a): a parent-pointer chain has no
  "length" to trim — you walk N `f_back` links and tear down each owned node one at a time. Branch
  (b) turns an O(N) sequence of node teardowns into an O(1) length update. (The values *inside* the
  discarded slots may still need per-slot cleanup — that cost doesn't vanish — but the *frame
  bookkeeping* collapses to one operation regardless of depth.) `ReturnNonLocal` uses the same
  primitive, `frames.truncate(token.frame_index)`; the *token* part is Lie #1 / Doc 6, but the
  unwind itself is just this truncate.

## The receiver, and the case where the model breaks

`context: CallContext` is the frame's `self`. Most of the time it is what you'd guess — a handle to
the receiver:

```rust
pub enum CallContext {
    Instance { instance: ObjRef },   // a method on a user instance
    Class    { class: ObjRef },      // a static method on a class
    Module   { module: ObjRef },     // top-level module code
    Immediate { value: Value },      // ← the one that breaks "receiver is always a handle"
}
```

`frame.rs::CallContext` (~L34). Three variants carry an `ObjRef` — a handle into the heap arena,
consistent with everything above. The fourth is the counterintuitive case worth tracing, because it
is where the tidy story "a receiver is always a handle" fails.

An **immediate** — a `Bool`, `Number`, or `Symbol` — is *not* a heap object in Phalcom. It lives
unboxed, directly inside a `Value`. There is no arena object, so **there is no `ObjRef` to point
at**. Normally that's fine: immediates are handled by primitives that never build a `CallContext` at
all. But a user can *reopen* the kernel `Bool` class and add a closure-backed method — and a closure
method, unlike a primitive, needs a real activation with a real receiver. With no handle to store,
`Immediate` carries the `Value` itself. (Per ADR-0018/U5, this variant exists specifically so the
sacred-selector inliner's override-epoch deopt guard is *exercisable* by exactly this
closure-on-an-immediate case — a narrow, deliberate reason, not a grand receiver-uniformity design.)
The GC handles it honestly: when tracing a frame's context, an `Immediate`'s `Value` is rooted only
`if let Some(id) = value.as_obj()` — an unboxed immediate roots nothing, because there is nothing on
the heap to keep alive.

## A frame is a value — and still a GC root

That last point generalizes into a tension worth naming, because it is easy to misread "frames are
plain values, not heap objects" as "frames are outside the GC's concern." False. A frame holds a
`closure` handle and a receiver handle — live references *into* the heap the collector manages. So
the frame must be part of the **root set**: `gc.rs::VM::collect_roots` (~L32) walks `vm.frames`
directly and calls `trace.rs::trace_frame` (~L37) on each, which pushes the frame's `closure` and
its context's handle as roots. Under branch (a) this is invisible — frames are heap objects, traced
like any other. Under branch (b) it is an *explicit extra step*: a plain value is not something the
ordinary heap trace would ever visit, so root enumeration must walk the frame array by hand. Being
cheap to represent did not exempt the frame from being a root; it only changed how the collector
finds out that it is one.

<a id="lie-2"></a>**Lie #2.** This doc says "the VM has *a* frame stack." Really `VM::frames` is a
**live mirror** of the currently-running fiber's own buffer. Each `FiberObject` (`heap/fiber.rs`
~L72) has its own parked `frames` `Vec`; a fiber switch swaps the live and parked buffers as a unit
— "an O(1) pointer-free copy (a `Vec` swap)," per the field's own doc. This is *why* `stack_offset`
is **fiber-relative**: each fiber's value stack is based at 0, so offsets need no rebasing on
switch, and frames+stack+upvalues must swap together or a stale offset would address another fiber's
slots. The full mechanics belong to the concurrency material (spec `concurrency.md`); here, read
"the frame stack" as "the running fiber's frame stack."

## Where Phalcom sits — and an honest gap

The three branches are points on a spectrum of **reification**: turning implicit control-flow state
into an addressable value the program can hold. Branch (c) doesn't reify frames at all. Branch (a)
reifies them all the way up to the object model — a frame is an *instance*, like any `String`, which
is **Smalltalk's** `thisContext`: reflectable, and because a context carries its own resume point,
*resumable* into continuations and coroutines with nothing bolted on. Smalltalk is the ancestor of
the *semantics* Phalcom's object model reaches for.

Phalcom sits in the middle, with Lua: the *runtime* reifies frames as internal values, but the
*language* is handed no first-class frame. And right now it sits **below** Lua on the reflection
axis. The machinery to render a multi-frame trace exists — `dispatch.rs::VM::runtime_error` clones
`self.frames`, walks them in reverse, and builds a per-frame `SourceLoc` for `print_rt` to render as
a "Traceback." But at HEAD that path is **not wired to the CLI**: `bin/phalcom/cli.rs::cmd_run` (~L161)
just does `eprintln!("{e}")` on error, and `runtime_error` is only reached through
`interpret_source`, used solely by the Rust test/bench harnesses. Run a three-deep call that throws
through the actual binary and you get the bare message —

```
$ phalcom deep_throw.ph
boom-from-inner
```

— not a stack trace, even though the correctly-ordered frame data was sitting right there in
`self.frames`. This is not designed minimalism; it is an unwired path, verified by tracing the call
chain at HEAD. Worth stating plainly so the doc doesn't flatter the code: the *data* for a Lua-grade
(copy-out) trace is present and correct; the CLI just doesn't ask for it yet.

The frame's *identity* mechanism, on the other hand, is fully live — which you can see without any
of Doc 6's machinery. Invoke a block after its home method has already returned, and:

```
$ phalcom dead_frame.ph
non-local return from a block whose home method frame is no longer alive (DeadFrameError)
```

The frame that was popped is gone, its slot's identity no longer matches, and the VM turns a
would-be memory-safety hazard into a clean runtime error. *How* the token detects that is Lie #1 →
Doc 6. That it detects it is the proof that branch (b)'s owed identity mechanism is paid.

## What you can now re-derive

Delete `frame.rs`. From two constraints —

1. **object references are `Copy` integer handles** (ADR-0009, decided for entirely other reasons), and
2. **the VM must own its call stack** (for fibers to swap it and the GC to root it) —

you can rebuild the frame: it is `Copy` (constraint 1), so it lives by value in a `Vec` (constraint
2), so it needs **no caller pointer** (array position is the chain), so returning is `pop` and
unwinding is `truncate`, and — because a `Vec` slot gets reused — it needs a **generation stamp** to
give a past activation a distinguishable identity (the one field whose job this doc deferred). Five
consequences, one type, from two premises. That is the whole of `CallFrame`, and none of it was a
frame decision — it fell out of the handle arena.

---

## Anchors

- `phalcom-core/src/frame.rs::CallFrame` (~L66) — the `Copy` record; module doc (~L1) for the
  "every link is a `Copy` handle → plain `Vec`, no borrow-panic surface" claim.
- `phalcom-core/src/frame.rs::CallContext` (~L34) — four receiver variants; `Immediate` (~L58) and
  its ADR-0018/U5 rationale.
- `phalcom-core/src/vm/dispatch.rs::VM::new_call_frame` (~L29) — build + generation bump (push is at
  the four call sites, not here).
- `phalcom-core/src/vm/dispatch.rs` — `Bytecode::Return` pop (~L1099); `VM::unwind_to` truncate
  (~L110).
- `phalcom-core/src/vm/mod.rs::VM::frames` (~L53) — the live mirror; `heap/fiber.rs::FiberObject::frames`
  (~L72) — per-fiber parked backing store (Lie #2).
- `phalcom-core/src/vm/gc.rs::VM::collect_roots` (~L32) → `heap/trace.rs::trace_frame` (~L37) —
  frames are GC roots.
- `phalcom-core/bin/phalcom/cli.rs::cmd_run` (~L137, `eprintln!` at ~L162) — the CLI error path that
  skips the traceback builder at HEAD.
- ADR-0009 (handle-arena-heap) — the deliberated decision that makes `CallFrame` `Copy`. ADR-0013
  (closure-upvalues-and-frame-token-return) — owns `generation`/`FrameToken`/`DeadFrameError` (Doc 6).

## Forward pointers

- **Doc 4 (message send)** — what a call site *does* to push a frame: method lookup, argument
  layout, and the `new_call_frame`-then-push the four call sites share.
- **[Doc 6 (frame identity)](frame-identity.md)** — destroys Lie #1: what `generation` and `home_frame_token` are for,
  how non-local return and `DeadFrameError` use the token, and why a raw frame pointer without a
  generation would be a memory-safety bug.
