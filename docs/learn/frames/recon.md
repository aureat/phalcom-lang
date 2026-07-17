# Recon — `frames.md` (VM track Doc 3)

Phase 1 scout. Four questions, nothing else. Cheap reads only; Agent B runs deep.

## 1. Architecture vs representation

**Architecture (the shape).** A call stack — a LIFO stack of activation records. Push
one record on a call, pop it on return, unwind many on a non-local return. Textbook. The
*shape* holds no surprise; every bytecode VM has this.

**Representation (where the consequences live).** The activation record is a **`Copy`
value**, not a heap object.

- `CallFrame` (`phalcom-core/src/frame.rs::CallFrame` @ L66) is `#[derive(Debug, Clone, Copy)]`.
  Every field it holds is itself `Copy`: `closure: ObjRef` (an integer handle, not a pointer),
  `context: CallContext` (Copy enum of Copy handles/`Value`s), `ip: usize`, `stack_offset: usize`,
  `caller_source: Option<SourceRange>`, `generation: u64`, `home_frame_token: Option<FrameToken>`.
- Because the record is `Copy`, the stack is a **plain `Vec<CallFrame>`** — no `Rc<RefCell<Frame>>`,
  no owning parent pointer, **no borrow-panic surface** (frame.rs module doc L1–6 says exactly this).
- **The Vec's position order *is* the caller chain.** There is **no `parent`/`f_back` field** on
  `CallFrame`. `frames[i-1]` is the caller of `frames[i]`. Unwinding is `Vec::truncate`
  (`dispatch.rs::unwind_to` @ L112: `self.frames.truncate(frames_len)`), a return is `frames.pop()`
  (`dispatch.rs` @ L1099). You do not unlink a node; you shorten an array.

The consequence axis is representation: **frame-as-value-in-array** vs **frame-as-heap-object-with-parent-pointer**.
This is the direct frame-level payoff of ADR-0009 (handle arena): once every link is a `Copy`
handle, the frame can be `Copy`, so the array works with zero borrow pain.

Cite: `frame.rs::CallFrame` @ L66 (the `Copy` derive + field list); module doc L1–6 (the *why*).

## 2. The grip, grounded

> **A frame is a value, not an object.** It is a flat `Copy` record in a `Vec`, so the call
> stack is an array you **truncate**, not a linked list you unlink — and the caller is just the
> record one slot down, never a pointer the frame carries.

Collapses the confusion a reader imports from Python/Smalltalk (frames are heap objects you can
grab and inspect) and from tree-walkers (frames are the native C stack). Phalcom is neither:
frames are reified enough to live in a data structure the VM owns, but cheap enough to be `Copy`
values pushed and truncated.

## 3. What was actually deliberated (ADR) vs pedagogical reconstruction

- **Deliberated — ADR-0009 (handle-arena-heap).** *Decision:* objects live in a central `Heap`,
  referenced by `Copy` integer handles (`ObjRef`/`ClassId`); no `Rc`, no `RefCell`.
  *Alternatives considered:* (a) `Rc<RefCell<T>>` + intentional kernel cycle — rejected: keeps the
  `RefCell` borrow-panic surface; (b) an immediate tracing `Gc<T>` — rejected as too much scope up
  front. **This is the decision that makes `CallFrame` `Copy`.** The frame being a value is a
  *consequence* of ADR-0009, not its own ADR — say so.
- **Deliberated — ADR-0013 (closure-upvalues-and-frame-token-return).** *Decision:* non-local
  return uses a **frame token** = frame index + **generation counter**; a generation mismatch
  raises `DeadFrameError`. *Alternatives:* by-value snapshot capture (rejected: breaks shared
  mutation); raw frame pointer with no generation (rejected: reused slot aliases a stale pointer).
  **This is Doc 6 (frame-identity) territory** — the `generation`/`home_frame_token` fields on the
  frame are a forward-pointer lie here.
- **Deliberated — ADR-0018/U5 (referenced in frame.rs L51–61).** The `CallContext::Immediate`
  variant exists because a *closure* method reopened onto an immediate (`Bool`/`Number`/`Symbol`)
  has **no `ObjRef`** to point at, so the context carries the `Value` itself. A real scar, worth a
  short trace.
- **Reconstructed (NOT an ADR).** The "stack of activation records / LIFO push-pop" framing and
  the design-space walk (heap-object-frame vs array-of-values vs native-stack) are **pedagogical
  scaffolding**. Phalcom did not hold a frame-representation bake-off; the Copy-value-in-`Vec`
  form *fell out* of ADR-0009. The honesty pass (§5.2) must state this.

## 4. Brief-steering notes

**Agent A (theory) — emphasis:**
- Go DEEP on the fork: **how do you represent an activation record?** Three tempting branches —
  (1) heap-allocated frame object with an owning parent/`f_back` pointer (reifiable, reflectable,
  but per-call alloc + a self-referential chain the borrow checker hates); (2) a flat array of
  value-records / register windows (push=write, pop=decrement, no per-call alloc); (3) the native
  machine stack (a tree-walker or JIT — implicit, fast, but unreifiable and fights green threads).
  Make each genuinely tempting with its bill.
- Go DEEP on **frame reification as a spectrum**: from "frames don't exist as data" (native stack)
  to "frames are ordinary first-class objects" (Smalltalk `thisContext`, Python `sys._getframe`).
  Name the term *reification*. This is the vocabulary the reader lacks.
- Distinguishing program: a program whose behaviour reveals whether frames are reified — e.g.
  reflective stack introspection (`sys._getframe`, `thisContext`) that only works if frames are
  objects; or a recursion-depth / stack-overflow observation. Make it concrete.
- ONE sentence only on: opcode dispatch internals (Doc 1), what a closure/callable holds (Doc 2),
  the generation-counter mechanics (Doc 6). Do not spend weight there.
- Comparison cast to develop: **Lua** (CallInfo register windows — the array branch, ancestor),
  **CPython** (`PyFrameObject`, `f_back` linked parent, `sys._getframe` reification — the heap-object
  branch WITH the bill; note 3.11 "zero-cost frames" changed the alloc story — flag it), **Smalltalk**
  (reified `MethodContext`/`BlockContext`, `thisContext` — the extreme; ancestor of the *semantics*).
  Cut candidates to name: JVM, Ruby MRI, generic tree-walkers.

**Agent B (source map) — must confirm (symbol-first):**
- `frame.rs::CallFrame` @ L66 — quote the full struct + the `Copy` derive. Confirm **no `parent`/
  caller-pointer field exists**. This is the load-bearing negative.
- `frame.rs::CallContext` @ L34 — quote all four variants; confirm `Immediate { value: Value }`
  and read the L51–61 doc explaining why (ADR-0018/U5).
- `frame.rs` module doc L1–6 — the "every link is a `Copy` handle, so the whole frame is `Copy`,
  so plain `Vec` with no `Rc<RefCell>`" claim. Tie to ADR-0009.
- **Push:** `dispatch.rs::new_call_frame` @ ~L29 — how a frame is constructed and pushed; confirm
  the generation bump (`next_frame_generation.wrapping_add(1)` @ ~L38).
- **Pop / unwind:** `dispatch.rs` @ ~L1099 (`frames.pop()`), `unwind_to` @ ~L110 (`frames.truncate`).
  Confirm the caller resumes as `frames[len-1]` after a pop (no relink).
- **Where frames live:** `VM::frames: Vec<CallFrame>` (`vm/mod.rs` @ ~L53) is the LIVE working
  buffer; `FiberObject::frames: Vec<CallFrame>` (`heap/fiber.rs` @ ~L72) is the per-fiber backing
  store; they mirror and swap on a fiber switch. Confirm `stack_offset` is an index into the shared
  value stack (`frame.rs` L74–75) and is **fiber-relative** (fiber.rs L77 comment on cross-fiber
  slot collision). Keep this BOUNDED — deep fiber mechanics are out of scope.
- **GC touch:** `trace.rs::trace_frame` @ ~L37 and `gc.rs::collect_roots` @ ~L32 — frames are GC
  roots; the collector walks `vm.frames` and traces each frame's `closure`/`context` handles. One
  line.
- **Behavioural (RUN it):** a `.ph` program with nested calls; observe a stack trace / error
  backtrace to show the frame stack is real and ordered. And — if reachable — a `DeadFrameError`
  (block invoked after home frame returned) to show the frame identity check *exists* (but defer
  the mechanism to Doc 6).

**Predict-then-check candidate (for §5.4):** *Python's frame carries `f_back`, a pointer to its
caller, so a frame can walk its own call chain. Phalcom's frames live in a `Vec`. Does `CallFrame`
carry a caller pointer?* Answer: **no** — the Vec position is the chain; `frames[i-1]` is the
caller, and unwinding is `truncate`, not pointer-chasing. A reader who predicts "no, because the
array already orders them" has derived the representation.

**Lies to mark (spiral):**
- **Lie A** — `generation` + `home_frame_token` on the frame are treated here as "just fields";
  their real job (frame identity, `DeadFrameError`, non-local return) → **Doc 6 (frame-identity)**.
- **Lie B** — "the VM has *a* frame stack" simplifies a per-fiber buffer mirrored into a live VM
  working copy and swapped on fiber switch → concurrency (spec `concurrency.md`); Doc 6 touches the
  identity angle.
- **Lie C (destroy)** — Doc 2 said a `ClosureObject` is "the recipe made runnable." Doc 3 shows
  *where* it runs: a frame is one activation *of* that closure, pairing the `closure` handle with a
  receiver `context` and a stack window. Pays off Doc 2's forward pointer.
