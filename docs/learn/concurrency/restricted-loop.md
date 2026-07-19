# The Restricted Loop

*Concurrency track, Doc C1. Track plan: [CONCURRENCY-PLAN.md](../CONCURRENCY-PLAN.md).*

## The grip

A fiber switch is not a jump, and not a scheduler decision.

It is `mem::take` on four fields of the VM — and **the dispatch loop is never told it happened.**
The loop that was running instructions before the switch is the same loop, at the same place in the
same `while`, running instructions after it. Nothing about it was saved, restored, or redirected.
What changed is *which buffers it owns*.

That single sentence buys both halves of this document. It is why a switch costs O(1) regardless of
how deep the fiber was. And it is why a switch is **illegal** whenever a native Rust frame is
sitting on the call stack holding an index into the buffers that are about to be swapped away.

The cheapness and the restriction are not two design decisions that happen to coexist. They are one
fact, read from two directions.

---

## The debt this pays

[Doc 4 (message-send)](../vm/message-send.md) drew the primitive-call path as "call native code,
place the result." Then it admitted, as [Lie #2](../vm/message-send.md#lie-2), that the real return
path branches three ways, and deferred one branch — `switch_pending`, *"a fiber switch firing inside
a primitive"* — to "the concurrency doc."

This is that doc, and that branch is [§ How the loop finds out](#how-the-loop-finds-out-it-doesnt).

Three things stay owed elsewhere on purpose. [Doc 1](../vm/execution-loop.md) already explained why
the hoisted `Rc<Callable>` guard keys on `closure_id` and *deliberately not* on `ip` — because a
switch swaps `self.frames` wholesale. That is paid; this doc only supplies the half Doc 1 could not,
which is *why the swap is wholesale*. **C2** owns the four fields as a mechanism — what each one is,
why `next_frame_generation` pointedly stays VM-global, how a parked fiber is a GC root. **C3** owns
what happens when a fiber fails. Here they are facts with a line number attached, not subjects.

---

## Why an interpreter can suspend itself for free

Start with the thing that makes fibers look easy, because the restriction is a consequence of it.

A bytecode interpreter is a `while` over a `match`. Its call chain is not the machine's call
chain — a Phalcom method call does not emit a Rust `call` instruction. It pushes a `CallFrame` onto
a `Vec` and keeps iterating the same loop ([Doc 3](../vm/frames.md): a frame is a `Copy` value in a
`Vec`, with no parent pointer). The instruction pointer is a `usize` in that struct. The operand
stack is another `Vec`.

So "where the computation is" is entirely **data the runtime allocated and owns**. Suspending it
requires no unwinding in the operating-system sense, no register saves, no stack pointer games. You
stop dereferencing the `Vec`. To resume, you start dereferencing it again.

It is worth naming the four things suspension actually demands, because every branch later in this
document differs in exactly which one it gives up:

1. **Capture** — record enough to reconstruct the execution point later.
2. **Ownership** — the captured state must belong to something the mechanism controls, so it is
   legal to stop touching it, let other code run, and come back to it unchanged.
3. **Relocation** — if memory moves under it (a moving collector, a growing stack), the captured
   state must survive that consistently.
4. **Resumption** — hand control back to exactly the captured point.

An interpreter's own frame stack satisfies all four almost by accident. Capture is "stop looking at
it." Ownership is trivially true — the runtime allocated it. Relocation is whatever the allocator
already does for heap data. Resumption is "look at it again."

**A Rust stack frame satisfies none of them.** Its memory validity is governed by the ABI and the
CPU, not the runtime. It cannot be serialized, cannot be relocated — moving a live C-convention
frame is precisely the operation that calling convention assumes never happens — and it cannot be
re-entered in the middle, because `call`/`ret` has no "come back to the middle of me later" mode.

Everything below is an answer to: *so what happens when one of those is in the middle of the chain?*

---

## The swap, in four fields

Here is a running fiber's state, and the surprise is where it is **not**:

```rust
// heap/fiber.rs::FiberObject @ ~L62 — field docs abridged
pub struct FiberObject {
    pub stack: Vec<Value>,                        // "empty while running — mirrored by VM::stack"
    pub frames: Vec<CallFrame>,                   // "empty while running — mirrored by VM::frames"
    pub open_upvalues: BTreeMap<usize, ObjRef>,   // "empty while running"
    pub checking: HashSet<ObjRef>,                // "empty while running"
    ...
}
```

A running fiber's `FiberObject` is **empty**. Its execution state lives in four fields on the VM
itself. The `FiberObject` is where state goes when the fiber *stops* running.

Parking is one function, and it is the whole mechanism:

```rust
// primitive/fiber.rs::store_live_into @ ~L29
let frames        = std::mem::take(&mut vm.frames);
let stack         = std::mem::take(&mut vm.stack);
let open_upvalues = std::mem::take(&mut vm.open_upvalues);
let checking      = std::mem::take(&mut vm.checking);
let fiber = vm.heap.fiber_mut(fiber_ref);
fiber.frames = frames;  fiber.stack = stack;
fiber.open_upvalues = open_upvalues;  fiber.checking = checking;
```

`load_live_from` is the same four moves reversed. Four `Vec`/map moves — pointer, length,
capacity — regardless of how many frames are on the stack. Nothing is copied element-wise, nothing
is rebased, nothing is walked.

> **Why nothing needs rebasing** is a decision from [Doc 3](../vm/frames.md) paying off in a
> subsystem that did not exist when it was made: a `CallFrame`'s `stack_offset` is
> **window-relative**, not absolute. A per-fiber stack always starts at index 0, so frames restored
> into a different `Vec` are already correct. Had offsets been absolute, every switch would have
> been a walk over every frame.

This is why `vm.current: ObjRef` is **bookkeeping, not indirection**. The loop never dereferences it
to find its stack. There is nothing to dereference — there is one set of live buffers, and whoever
owns them *is* the running fiber. A switch does not redirect the loop at a different fiber. It
changes what the loop's own fields contain.

> **Correction to a claim in [Doc 1](../vm/execution-loop.md).** Doc 1 called the inner loop
> "deliberately fiber-unaware." Its *exit test, opcode fetch, and dispatch structure* are —
> verified by reading the loop — but two arms are not: `GetUpvalue`/`SetUpvalue`
> (`vm/dispatch.rs` @ ~L1052-1087) branch on `fiber == self.current`, because an open upvalue can
> be captured by a closure whose home frame is parked on another fiber. That branch is
> [`upvalues.md`](../vm/upvalues.md)'s subject, not this doc's — it is about upvalue *ownership*,
> not control transfer. Doc 1's claim is right about the machinery this doc is about, and needs
> that one qualification.

---

## How the loop finds out (it doesn't)

If a switch changes the VM's fields out from under the loop, something has to stop the ordinary
post-call bookkeeping from running against the wrong fiber's stack. Here is where that happens —
Doc 4's Lie #2, in full:

```rust
// vm/send.rs::VM::call_method @ ~L19, the Primitive arm
let receiver_idx  = self.stack.len() - 1 - arity;
let frames_before = self.frames.len();
self.switch_pending = false;
let result = native_fn(self, &receiver, &args[..arity]);
result.map(|result| {
    if self.switch_pending {                       // (1) a fiber switch fired inside native_fn
        self.switch_pending = false;
    } else if self.frames.len() >= frames_before { // (2) ordinary primitive return
        self.stack.truncate(receiver_idx);
        self.stack.push(result);
    } else {                                       // (3) a non-local return unwound through it
        self.stack.push(result);
    }
})
```

Read arm (1) carefully, because it is the load-bearing one and it does **nothing**.

`receiver_idx` was computed against the stack of the fiber that was running *before* the call. If
`native_fn` was `fiber_yield`, that stack is now parked inside a `FiberObject` and `self.stack`
belongs to somebody else. `truncate(receiver_idx)` would cut into a stranger's window. So the arm
clears the flag and touches nothing: the switching primitive already left the incoming fiber's stack
exactly as the loop needs to find it.

Arm (3) is why the flag has to exist at all. A frame-count delta cannot distinguish these cases,
**because a switch changes `frames.len()` too** — it replaces the whole vector. A switch that
happened to reduce the frame count would be misread as a non-local return and the VM would push a
value into the wrong fiber. The two mechanisms coexist because they answer different questions, and
neither replaced the other.

One easily-missed line: `self.switch_pending = false` **before** the call. It clears a stale `true`
from an earlier primitive in the same expression, so this call's branch cannot be misled by someone
else's switch.

> **Honest gap — the ADR specified a different mechanism.** ADR-0030 §5 calls for *"an explicit
> `ControlFlow`/switch value **out of the primitive**."* HEAD has no such return type; every
> switching primitive returns a plain `PhResult<Value>`, and the signal is
> `VM::switch_pending: bool` (`vm/mod.rs` @ ~L79) — a field the primitive sets as a side effect.
> This is **not drift**: the U-FIBER implementation spec recorded it as decision **D-FIB-5**,
> naming the typed return as "recommended" and the flag as the pragmatic choice, because threading
> a typed return would touch all ~70 existing primitives. The *substance* of the ADR's decision
> shipped — the signal is explicit rather than inferred from a length delta. The *mechanism* it
> named did not, knowingly. Do not read §5's prose as describing the code.

---

## Predict before you read

You now have both halves. A switch swaps the VM's four fields wholesale. And you have seen that a
native primitive can call *back into* the loop — that is what arm (3) is about.

Three fiber bodies. Two of them run. One raises `CannotYieldAcrossNativeFrame`. Which, and why?

```phalcom
// A
Fiber.new { var n = 0; while (true) { Fiber.yield(n); n = n + 1 } }

// B
Fiber.new { list.each { x => Fiber.yield(x) } }

// C
Fiber.new { var i = 0; while (i < list.size) { Fiber.yield(list.at(i)); i = i + 1 } }
```

Observed at HEAD — A prints `0 1 2`, C prints `10 20 30`, and B:

```
cannot switch fibers across a native call frame (e.g. inside .each { })
```

Most readers get B right and get the **reason** wrong. The natural answer is: *`each` is a
built-in, so it is written in Rust, so there is a Rust frame in the middle.* That answer is wrong,
and the way it is wrong is the most interesting thing in this document.

---

## `each` is written in Phalcom

```phalcom
// core/core.ph::Iterable#each @ ~L654
each(f) {
  for (x in self) {
    f.call(x)
  }
}
```

`List` does not implement `each` natively. It inherits it from `Iterable`, in Phalcom source. And
`for` is *not* a native loop either — the compiler lowers it to a `$cursor`/`Loop`/`JumpIfNone`
sequence of direct jumps **inside the same chunk** (`compiler/lib/loops.rs::compile_for` @ ~L120),
exactly as frameless as A's `while`.

So B's iteration is frameless. B's loop is frameless. The list walk is frameless. Everything about
B is frameless except one thing:

```phalcom
f.call(x)
```

`call(_)` on a block resolves to `MethodKind::Primitive(block_call)`, and `block_call` does this:

```rust
// primitive/block.rs::block_call @ ~L155
vm.native_reentry_depth += 1;
let result = vm.run_until(base_frames);
vm.native_reentry_depth -= 1;
```

That is a **recursive call to the interpreter, on the Rust stack.** One per element.

The restriction has nothing to do with which language a library was written in. Rewrite the whole
standard library in Phalcom and it does not move. The line is drawn at exactly one place:

> **Invoking a block is the only ordinary construct that re-enters the interpreter through the
> host stack.** Everything the compiler lowers to jumps — `while`, `for`, `ifTrue:` — stays inside
> one chunk and costs no native frame. Everything that reaches user code by *calling a block value*
> pays one.

A and C are legal because they never call a block. B is illegal because it calls one per element.
Same iteration, same semantics, same language — and the suspend/no-suspend line falls between them
because of how the *compiler* routes control, not because of what the library is made of.

That is a genuinely strange property, and worth sitting with: **a language-visible capability falls
directly out of an optimizer's decision about what got inlined.** Nobody designed
iteration-by-loop-construct to be more suspendable than iteration-by-higher-order-function. It is
an artifact of which subsystem owns control flow at the point of interest — the compiler's lowering
(owned, replayable, in-chunk) versus a block invocation (opaque, on the host stack). A future
compiler that lowered block calls differently would silently move the line with no change to the
language spec at all.

> **Correcting the track plan.** [CONCURRENCY-PLAN.md](../CONCURRENCY-PLAN.md) §3 attributed this
> to ADR-0018's sacred-selector inliner. That is right for `while` and wrong as the general rule:
> `for`'s lowering is a separate compiler mechanism, not one of the inliner's five sacred
> selectors, and it is equally frameless. The rule is about block invocation, not about the
> inliner. The corpus knows this — the fixture `each_generator_raises` names the distinction in its
> own comment.

---

## Down the illegal path, once

Trace B, since it is the case intuition gets wrong.

1. `f.call()` resumes the fiber. `fiber_resume` parks the root fiber's four fields into its
   `FiberObject`, moves the callee's in, pushes the entry frame. `native_reentry_depth` is `0`.
2. The fiber body runs `list.each { … }` — an ordinary `Invoke`. A `CallFrame` is pushed. Still
   `0`; a Phalcom method call is not a native call.
3. Inside `each`, `for (x in self)` lowers to jumps. No frame, no native call. Still `0`.
4. `f.call(x)` — an `Invoke` of `call(_)` on a block. `call_method` reaches
   `MethodKind::Primitive(block_call)`.
5. `block_call` snapshots `base_frames = self.frames.len()`, increments `native_reentry_depth` to
   **1**, and calls `vm.run_until(base_frames)` — **a Rust-stack recursion into the interpreter.**
6. That nested loop runs the block body and reaches `Fiber.yield(x)`.
7. The guard fires.

Now the part that makes the guard inevitable rather than arbitrary. Suppose it did not fire.
`fiber_yield` would call `store_live_into` and `mem::take` `vm.frames` — and step 5's Rust frame is
still live, several layers down the host stack, holding `base_frames`: a plain `usize` index into
the vector that was just moved away. When the resumed fiber eventually returned, that nested
`run_until` would resume its loop comparing `self.frames.len()` against a floor computed for a
**different fiber's** call stack. It would drain to the wrong depth — either returning early through
frames that are still live, or running off the bottom of a stack that never had that many frames.

And there is no way to fix it up. The runtime cannot rewrite `base_frames`, because it is a local
variable in a Rust stack frame it does not own and cannot inspect. That is the whole restriction, in
one sentence: **the interpreter can move its own data, but it cannot reach into the host stack to
tell a suspended Rust frame that its index means something different now.**

The guard is a counter comparison in `fiber_yield`:

```rust
// primitive/fiber.rs::fiber_yield @ ~L338
if vm.native_reentry_depth != vm.heap.fiber(me).floor_depth {
    return Err(cannot_yield_across_native_frame(vm));
}
```

Four sites maintain that counter, each wrapping a re-entrant `run_until`: `block_call` (the one
users hit), `send_dynamic` (reflective `perform`), `invoke_method_object` (`Method#invokeOn`), and
`import_module`. All four have the identical hazard — a `base_frames` snapshot taken against the
currently running fiber.

`CannotYieldAcrossNativeFrame` is a **surface `Error`**, not a VM panic. Program B under `try()`
instead of `call()` exits `0` and prints `CannotYieldAcrossNativeFrame`. A program can catch its own
violation of this rule. What happens to a fiber that fails is **C3**'s subject.

---

## The fork that was actually argued

Every VM-track doc had to confess that its design-space walk was pedagogical reconstruction — no
bake-off was ever held for stack-vs-register or for frame representation. **That confession does not
apply here, and this is the first doc in the course where it does not.** ADR-0030 records four
rejected branches with bills attached. What follows is the decision as it happened.

### A — restricted (taken)

Suspension's domain is exactly the frames the loop owns; attempting it under a re-entrant native
frame raises. Costs a counter and a comparison. Sound by construction: it never tries to freeze a
native frame because it refuses to be in that position. Forecloses the callback generator.

### B — full trampoline

Stop re-entering natively. Rewrite every callback-taking primitive so that instead of recursing into
`run_until`, it pushes a continuation frame onto the interpreter's *own* frame stack and returns to
the loop. Nothing native is left mid-flight, so suspension works everywhere.

The ADR calls A→B *"purely additive."* That is true at one level and misleading at another, and the
distinction is worth having:

- **At the level of programs it is genuinely additive.** No program that worked stops working. You
  can convert `block_call` and leave `send_dynamic` raising forever; both states are consistent. The
  restricted domain shrinks monotonically, one primitive at a time.
- **At the level of engineering it is a rewrite,** repeated per primitive. What makes native
  recursion convenient is that the Rust compiler already maintains the primitive's state machine
  for free — "where am I in this loop" is an instruction pointer plus locals. Trampolining means
  making that state machine *explicit*: a data type naming every point a pause could land and every
  piece of transient state that must survive it. There is no generic transformation for arbitrary
  native code. And the calling protocol changes globally — a primitive can no longer just
  `return value`; it must be able to say "push this continuation and re-enter the loop instead."

So: a one-way ratchet you can turn as slowly as you like, at a cost paid almost entirely **outside**
the fiber subsystem, by whoever maintains the primitives. Never a flag flip. The ADR's "additive" is
right about compatibility and optimistic about labour.

### C — stackful coroutines

Give each fiber a real native stack and switch the stack pointer. `yield` from ten frames deep
inside a callback becomes no different from yielding at the top: nothing unwinds, nothing is
rewritten, the parked frames sit exactly as the ABI left them. No per-primitive rewrite, ever. This
does not shrink the restricted domain — it deletes the distinction.

The usual objection is `unsafe` and portability. **That is not the real bill, and the ADR knew it.**

A tracing collector must find its roots. With one stack, "the roots on the stack" means one call
chain, and the collector can arrange to run only at points whose shape it fully controls. Stackful
coroutines multiply that by the number of live fibers, and each parked stack froze *wherever the
program happened to call `yield`* — possibly inside a leaf utility, inside FFI, anywhere. That
forces one of two commitments:

- **Conservative scanning** — treat any word that looks like a pointer as one. Cheap to build, and
  it permanently forecloses a **compacting** collector for anything reachable from a parked stack:
  you cannot overwrite a word with a relocated address when you do not know it is a pointer rather
  than an integer with the same bits.
- **Precise stack maps everywhere** — the compiler emits, for every function at every possible
  suspend point, which slots hold live references. This permits compaction, and it couples codegen
  and collector at the level of *every function that could appear in any fiber's chain*.

Either way the deciding subsystem is **the garbage collector**, not the fiber implementation. The
ADR names this exactly — a crown-jewel conflict, *stackful-fiber ⊗ moving-GC*, directly weakening
[ADR-0009](../../adr/accepted/0009-handle-arena-heap.md)'s handle-arena heap — and rejects C on
those grounds: *"The power is not worth an irreversible GC commitment."*

**That asymmetry is the decision's spine.** A→B is reversible in place: an unconverted primitive
keeps raising, affecting nothing else. A→C is a one-way door in a subsystem that has nothing
conceptually to do with concurrency, binding every allocation site in the runtime forever.

Phalcom does not have a moving collector today. It rejected C to keep the option.

### The edges

**Preemption / OS threads** was rejected as requiring a memory model and locks throughout the object
model — "the singular cooperative primitive is the whole point." **Resumable (Smalltalk) suspension
of failures** was ruled out of scope; ADR-0008 propagation is terminating.

---

## Lua: the same rule, and the escape route, in one language's history

ADR-0030 names its lineage as *"audit Option A / Lua-5.1 style,"* and the resemblance goes further
than the ADR claims.

Lua represents Lua-to-Lua calls on its own data stack inside the `lua_State`, not the C stack —
the same "runtime-owned frames" property, arrived at independently. So a pure Lua call chain
suspends freely. The trouble is any sequence where a C function invoked from Lua calls *back* into
Lua: `table.sort` with a Lua comparator, a C library function taking a Lua callback. That C
activation is a real frame on the real C stack, and Lua 5.1 makes yielding under it a hard error
rather than undefined behaviour — in substance, *attempt to yield across a C-call boundary*.

That is Phalcom's rule, in a language that shipped it years earlier, for the identical structural
reason. It is also the same rule with a different **seam**: Lua's boundary is the C API, Phalcom's
is `Block#call`. Lua's C functions are foreign; Phalcom's `each` is not foreign at all, and the
boundary still appears, because what matters was never foreignness — it was whether control reaches
the suspend point through a frame the runtime owns.

**Then Lua narrowed it**, and this is why it is the highest-value comparison available: it is a
shipped instance of the A→B path, at the smallest possible grain. Lua 5.2/5.3 added `lua_yieldk`
with `lua_callk`/`lua_pcallk`. A C function that wants a yieldable callback can no longer call
`lua_call` and write straight-line C after it. It passes a **continuation function** — a separate C
function pointer plus a context value — which the runtime invokes *from scratch* when the coroutine
resumes. The original C frame and its locals are gone; only the continuation pointer and context
survive.

Which is exactly the trampoline transformation: split the primitive into "before the callback" and
"after," communicating through runtime-visible state rather than host locals. Done by hand, opt-in,
one function at a time.

And precisely what it did **not** buy is the same thing the ADR's "additive" glosses over: an
unmodified C function that calls `lua_call` expecting a normal return still hits the same error it
always did. Lua did not remove the restriction. It made it purchasable per call site, for the price
of a rewrite.

*(Lua's exact 5.1 error wording and the precise `lua_yieldk`/`lua_callk` signatures across the
5.2/5.3 line are recalled, **not verified against Lua's source** for this document. The mechanism
and history are what the argument rests on.)*

---

## The name for this: coloring

JavaScript gives the vocabulary Phalcom lacks. An `async function` is a different **color** from an
ordinary one; a synchronous function cannot `await`; the color is infectious upward. The term comes
from Bob Nystrom's *"What Color is Your Function?"* — worth knowing that Nystrom also wrote **Wren**,
the language whose fiber semantics Phalcom's own test corpus is ported from.

The generalizing move is to notice JS's restriction has **no native frame anywhere in it**, and yet
has the identical shape. So the dividing line was never native-versus-interpreted. It is: *does this
frame's control state live in a representation the suspension mechanism can capture?* An `async
function` is compiled into a resumable state machine with heap-allocated locals — the trampoline
transformation, applied automatically by the compiler. An ordinary JS function is not, so to the
suspension mechanism it is exactly as opaque as a Rust `block_call` frame. Plain generators do not
lift this either: you cannot `yield` from a plain function called by a generator, only from the
generator's own body or via `yield*`. That is the definition of a **stackless** coroutine — suspends
only from its own activation, not from arbitrary nested calls — as against a **stackful** one.

Phalcom is stackless and asymmetric: `yield` returns to whoever resumed, tracked by a `resumer` link
that is a *dynamic* caller chain rather than a fixed parent.

The sharp contrast is not mechanism but **when you find out**. JS's coloring is static, source-visible,
and enforced by the compiler: a function either is or is not declared `async`. Phalcom's is dynamic,
invisible in the source, and **discovered by hitting the error at runtime** — B and C are the same
program to the naked eye. Same invariant. JS surfaced it in the type system; Phalcom leaves it
implicit in the shape of the call graph.

---

## Go, and the boundary that comes back

Go took branch C as far as anyone has shipped it: every goroutine gets a real, growable stack, and
the runtime can suspend one nearly anywhere. It pays the GC bill on the expensive side — the compiler
emits **precise stack maps**, which is what makes Go stacks not merely scannable but *movable*: the
runtime copies an entire goroutine stack to a larger allocation as it grows, rewriting every internal
pointer, including pointers into the stack itself.

The confirming detail is the one worth carrying away. Go's own answer to native re-entrancy —
`cgo` — reintroduces the identical restriction: a goroutine inside a `cgo` call is pinned to its OS
thread and is not movable the way ordinary Go code is, precisely because C frames carry none of the
stack-map metadata Go guarantees for its own.

The runtime that spent the most engineering budget in this entire design space on making suspension
general **hits the same wall the moment the frames are genuinely foreign.** The restriction is not a
symptom of an under-resourced implementation. It is a property of the boundary between frames the
runtime's metadata describes and frames it does not — and that boundary reappears wherever a foreign
calling convention does, no matter how much has been built on the near side.

*(Go's cgo/preemption specifics and the version that introduced asynchronous preemption are recalled
loosely and **not verified** here.)*

### Wren, and what our own corpus does and does not prove

`phalcom-core/tests/lang/concurrency/` contains ten `concurrency_fiber_wren_*` fixtures, each ported
from a named Wren test: `yield.wren`, `try_value.wren`, `call_done.wren`, `abort_not_string.wren`,
and so on. Phalcom validated its **observable** fiber semantics against Wren's directly — implicit
`None` on falling off the end, `try` capturing what `call` re-raises, `Fiber.abort(_)` accepting any
value.

That is first-party evidence, and it is evidence about **Phalcom**: it says Phalcom's surface is a
deliberate port, not a coincidence. It says nothing about Wren's *implementation*, which was not
examined for this document. Wren belongs in this cast for what the fixtures prove and no further.

### Cut from the cast

**Erlang/BEAM** — reduction-counted scheduling with no re-entrant-native-callback problem in the
paths it covers; its real analogue (long-running NIFs, dirty schedulers) is a scheduler-fairness
argument, a different axis. **Ruby `Fiber`** — branch C, but restating the GC bill with different
proper nouns; Go carries it better. **Python `asyncio`** — isomorphic to the JS coloring argument.
**C#/Kotlin** — same, with one footnote worth keeping: Kotlin's `suspend` threads a literal
`Continuation` parameter through generated bytecode, making the trampoline transform about as
explicit as it gets. A footnote on JS's lesson, not a second lesson.

---

## Three places HEAD and the ADR disagree

The design space above is real. The implementation record is not identical to it, and a doc that
smoothed that over would be flattering the codebase.

**1. The signal.** Covered above: ADR §5 specifies a typed `ControlFlow` return; HEAD ships a `bool`
field, recorded deliberately as D-FIB-5.

**2. HEAD restricts more than the ADR does.** §4 forecloses only *yielding* under a native frame.
HEAD also forbids **resuming** — `Fiber#call`/`try` — whenever `native_reentry_depth != 0`, and says
so on purpose:

> *"spec §6's restriction table only forecloses yielding underneath a native frame, so this is a
> deliberately **wider, sound over-restriction** … any switch underneath it — resume or yield
> alike — would corrupt [`base_frames`]."*
> — `primitive/fiber.rs::cannot_resume_across_native_frame` @ ~L86

Sound: the hazard is symmetric, so the restriction should be. It has its own error wording naming
*resume* rather than *yield*, and its own regression fixture
(`negative/fiber_resume_gate_call_native_frame.ph`).

This is the "ship wide and sound, narrow it call site by call site" pattern — the same one Lua
followed. A blanket rule rejects some programs that would in fact have been safe, and buys an
invariant that survives every future refactor of the primitives it restricts without re-auditing
anything. When primitives are numerous and change often, that is the right trade.

**3. A guard more general than the machine can exercise.** The two gates use *different* predicates.
Resume checks `!= 0`, absolute. Yield checks `!= floor_depth`, relative to a floor each fiber records
when it starts running. The relative form is the correct general shape: a yield's hazard is depth
accrued *since this fiber last resumed*, not absolute depth.

Except `floor_depth` is **provably always `0` at HEAD** — verified, twice, adversarially. Its only
non-constant writer is in `fiber_resume` @ ~L317, dominated by that function's own `!= 0` early
return, with no re-entrant call in between that could change the depth; every other initializer is
the literal `0`.

So the relative check is currently *exactly equivalent* to `!= 0`. It is not dead code — it is the
shape the guard would need if finding #2's wider restriction were ever narrowed back to the ADR's
line. But it buys nothing today, and this doc will not pretend the two-predicate design is a live
distinction. It is generality written for a world that has not arrived.

*(No performance claim is made anywhere here. "O(1)" is structural — four `Vec`/map moves,
independent of stack depth — not a measurement. Nothing in this document has been timed; this repo
keeps its numbers in [`perf-log/SCOREBOARD.md`](../../forge/perf-log/SCOREBOARD.md).)*

---

## What you can now re-derive

Nothing above needed memorizing. Given the constraints, it falls out:

- **Why a switch is O(1) regardless of depth** — because the running fiber's state is four fields on
  the VM, and parking is four `mem::take`s. Not because anything was optimized.
- **Why the loop needs no per-instruction fiber check** — because there is nothing to check. One set
  of live buffers exists; owning them *is* running.
- **Why `switch_pending` cannot be a frame-count delta** — a switch replaces the frame vector, so it
  changes the count too, and would be misread as a non-local return.
- **Why yielding inside `each` is illegal but inside `while` is fine** — not because `each` is
  native (it is written in Phalcom), but because reaching the yield point through `f.call(x)` puts a
  Rust `run_until` on the host stack, holding a `base_frames` index into the very vector a switch
  would move away.
- **Why the runtime cannot just fix that index up** — it is a local in a Rust frame the runtime does
  not own and cannot inspect. Hence a refusal rather than a repair.
- **Why the restriction is a `yield`-side *and* `call`-side rule** — the hazard is symmetric even
  though the spec only wrote down one half.
- **Why stackful coroutines were rejected by an argument about the garbage collector** — a parked
  native stack is a root frozen at an arbitrary program point, forcing either conservative scanning
  (no compaction) or universal precise stack maps (whole-compiler commitment). Concurrency would
  have bound the collector forever.

And the one-line version: **the loop can move its own data, but it cannot reach into the host stack
to tell a suspended Rust frame that its index means something different now.**

---

## Anchors

| Symbol | Location |
|---|---|
| `heap/fiber.rs::FiberObject` | @ ~L62 — the four mirrored fields, `floor_depth`, `resumer` |
| `heap/fiber.rs::FiberStatus` / `::FiberResumeMode` | @ ~L12 / ~L37 |
| `primitive/fiber.rs::store_live_into` / `::load_live_from` | @ ~L29 / ~L49 — the swap |
| `primitive/fiber.rs::fiber_resume` | @ ~L247 — absolute gate @ ~L248, `floor_depth` write @ ~L317 |
| `primitive/fiber.rs::fiber_yield` | @ ~L333 — relative gate @ ~L338 |
| `primitive/fiber.rs::cannot_resume_across_native_frame` | @ ~L86 — the wider-than-spec rationale |
| `vm/send.rs::VM::call_method` | @ ~L19 — the three-way `Primitive` arm |
| `vm/mod.rs::VM::switch_pending` / `::native_reentry_depth` | @ ~L79 / ~L91 |
| `vm/dispatch.rs::VM::run_until` / `::run_until_inner` | @ ~L221 / ~L477 |
| `vm/dispatch.rs::VM::switch_to_fiber_and_deliver` | @ ~L352 |
| `primitive/block.rs::block_call` | @ ~L117, depth increment @ ~L158 |
| `compiler/lib/loops.rs::compile_for` | @ ~L120 — the frameless `for` lowering |
| `core/core.ph::Iterable#each` | @ ~L654 — `each` is Phalcom, not Rust |
| ADR-0030 §4/§5/Alternatives | [`docs/adr/accepted/0030-…`](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) |
| D-FIB-5 (typed return vs VM flag) | `docs/forge/units/U-FIBER/implementation-spec.md` @ ~L332 |

Fixtures: `phalcom-core/tests/lang/concurrency/` — `concurrency_fiber_restricted_yield_guard.ph`,
`each_generator_raises.ph`, `negative/fiber_resume_gate_call_native_frame.ph`, and the ten
`concurrency_fiber_wren_*` ports.

---

## Forward pointers

- **C2 — the parked fiber.** The four fields as a *mechanism*: what each holds, why
  `next_frame_generation` deliberately stays VM-global (so a cross-fiber return token fails its
  generation check), fiber-stack pooling, and why a parked fiber must be a GC root even though
  nothing is running it. This doc used the swap; C2 explains it.
- **C3 — when a fiber fails.** `CannotYieldAcrossNativeFrame` was named here as a catchable surface
  `Error`. What the fiber floor does on the way down — the `call`/`try` cascade, `capture_error_value`,
  and the parked-state teardown — is C3's.
- **C4 — futures.** `System.schedule(_)` and the root-drive pump appear inside `run_until` and were
  deliberately skipped: they decide *which* fiber resumes and *when*, and reuse this doc's switch
  machinery completely unchanged.
- **[`upvalues.md`](../vm/upvalues.md)** — owns the `GetUpvalue`/`SetUpvalue` fiber-aware branch this
  doc flagged as a qualification to Doc 1.
- **Unresolved:** whether the resume-side over-restriction (#2) should ever be narrowed to the ADR's
  line. Doing so is what would make `floor_depth` (#3) load-bearing. No unit owns this.
