# Frame identity

*VM track, Doc 6 — the last one. It cannot be read first.*

Five documents have been writing IOUs to this one.

[Doc 3 (frames)](frames.md) told you a **lie**, and labelled it: `CallFrame` has a `generation`
field and an `Option<FrameToken>`, and you were told to treat both as "just fields for now." [Doc 2
(compiled artifact)](compiled-artifact.md) showed you a `BlockObject` and skipped past the second of
its two fields. [Doc 4 (message send)](message-send.md) pushed frames without saying what made one
distinguishable from another. [Doc 1 (execution loop)](execution-loop.md) showed you a drain check —
`if self.frames.len() <= base_frames { return ... }` — and you will find out here that a non-local
`return` terminates by *arranging for that check to fire*, rather than by returning at all.
And [`upvalues.md`](upvalues.md) introduced `FrameToken` from the **closure's** side: what a captured
`return` needs. This doc closes it from the **frame's** side: what a recycled slot owes.

That is why it is last. The mechanism is a genuine knot — you cannot explain the token without the
frame, the frame without the send, the send without the artifact, or the unwind without the loop.

If you have read `upvalues.md`, you have already seen the four-line liveness compare, the
`DeadFrameError` output, and the Smalltalk name. Those are not this document's payload. **This
document is about the two halves of the token having different scopes**, and everything that falls
out of that.

---

## The grip

> A `FrameToken` is a pointer deliberately split in two. `frame_index` is **where to look**;
> `generation` is **who it was**. The first is fast, fiber-local, and recycled. The second is
> VM-global and never reused. Every non-local return *dereferences* with the cheap half and is only
> ever *believed* because of the expensive one.

That asymmetry is the whole design. Hold it loosely for now — the doc's job is to make you re-derive
it, and then to show you the one place it is written down as an invariant.

---

## The problem is the *control* half, and it has no data-half solution

The classical name for closures outliving their creating frame is the **upward funarg problem**:
*downward* funargs (a function passed *into* a callee) are trivially safe under stack discipline,
because the callee's activation nests inside the caller's; *upward* funargs escape outward — into a
return value, a field, a callback — and outlive the activation that built them.

Nearly every treatment of funarg stops at the **data half**: a closure closes over *variables*, those
variables lived in a frame, the frame is popped, so the storage must be given a lifetime independent
of the frame. That is solved territory, and `upvalues.md` is Phalcom's chapter on it: open cells that
alias a live slot, closed cells that own a copied value.

The **control half** is a different question and inherits nothing from that answer. Non-local return
is not "read a captured variable." It is *"resume execution at a specific point in a specific,
already-running computation, unwinding everything between here and there."* That is a claim about
**control state** — is there still a live activation to return *to* — not about **data state** — does
this storage still hold a coherent value.

The two diverge completely. Heap-promoting a closed upvalue makes the *variable* immortal: an
ordinary garbage-collected cell, readable correctly forever. It does exactly nothing about whether
the *activation* — this specific episode of "this method, called from this caller, at this moment"
— is still around to be returned to. **An environment can be permanently alive while its owning
activation is permanently gone.** Solve capture perfectly and you have not started on this.

And the structure the activation lives in recycles. Doc 3 established that a `CallFrame` is `Copy`
and lives by value in a plain `Vec`; returning is `pop`, unwinding is `truncate`. The entire
performance argument for that shape is that slots are reused the instant they are free. Which is
precisely what makes naming an activation hard: the name was a perfectly good name when it was
minted. The activation returns. Its index is reused by an unrelated call. Dereference the old name
now and it finds *something* — a real, live frame. It is simply the wrong one, and nothing about a
bare index says so.

This is the **ABA problem**, and the name comes from lock-free data structures, not language
runtimes: a thread reads head pointer `A`, another thread pops `A`, pushes `B`, pops `B`, and the
allocator hands back `A`'s exact address for a new node. Head is bit-for-bit `A` again. The first
thread's compare-and-swap succeeds and believes nothing happened. *(The term is widely attested in
lock-free literature from the 1990s on — Treiber's stack is the teaching example — but it is
practitioner folklore without one citable first use; stated here as attribution, not as a
citation.)* The transplant is exact rather than analogical: a raw frame index is a bit pattern
standing in for "the same logical activation is still there," failing for the identical reason a raw
head pointer does.

---

## The distinguishing program

Two features wear similar syntax. The clean way to separate them is a language that gives them
different keywords. **Ruby** is the canonical pair — `lambda` and `proc` differ in exactly and only
this respect:

```ruby
def call_it_now(callable)
  result = callable.call(1)
  puts "callable returned #{result} to me"
end

call_it_now(lambda { |x| return x * 10 })
# => prints "callable returned 10 to me".
#    lambda's `return` is an ordinary function return.

call_it_now(proc { |x| return x * 10 })
# => prints nothing. `return` inside a proc means "make the method that
#    LEXICALLY encloses me — call_it_now — return, right now, unwinding
#    through the call to callable.call."
```

Phalcom has only the second kind. A `{ ... }` block's `return` is always the non-local one; it
targets the method whose *text* contains the block. Which means Phalcom has, unavoidably, the failure
mode:

```phalcom
class Maker {
  make() { return { return 1 } }
}
let escaped = Maker.new().make()   // make() has already returned. Its activation is gone.
System.print(escaped.call())       // the block's `return` targets... what?
```

That is `tests/lang/runtime-errors/runtime_non_local_return_dead_frame.ph`, verbatim. The block is
invoked correctly, with the right arity, by code with every right to call it — and its destination no
longer exists. Ruby answers this with `LocalJumpError`. The rest of this document is about producing
that failure *deliberately and detectably* rather than as memory corruption.

---

## The design space

The question, precisely: you need a **name** for an activation such that (1) minting it is cheap
enough to do on every call that might be captured, (2) using it to jump back is cheap enough to do on
every non-local return, and (3) using one whose target has been recycled is **detectable**, not
silently wrong.

**An honesty note before the walk.** ADR-0013's *Alternatives considered* has exactly two entries,
and only one of them is about identity: **by-value snapshot capture** (a *capture* alternative,
rejected for breaking shared mutation) and **"raw frame pointer with no generation counter."** So the
deliberated space was (a) vs (b) below, full stop. Branches (c) through (f) are **pedagogical
reconstruction** — they are the space as it exists in the literature, not the decision as it
happened. They earn their place by showing what the choice forecloses, not by having been on
anyone's whiteboard.

### (a) Raw index — the branch that was actually rejected

Take it seriously first, because it is *free*. One word. No comparison, no second field, no
allocation. If activations live in an array, "the fourteenth slot" is a `usize`, and dereferencing it
costs one array index — no different from the pointer-chasing the VM already does constantly. This is
what a naive interpreter reaches for, and what C's `setjmp`/`longjmp` effectively gives you: a
`jmp_buf` is a saved stack state you jump back to by address, with no guarantee whatsoever that the
frame which called `setjmp` is still live. Using it after that frame returns is undefined behavior,
not a checked error. C simply does not offer the check.

ADR-0013 kills it in one sentence, and the sentence is the ABA problem in domain clothes:

> a reused frame slot would alias a stale pointer to a live frame, silently returning to the wrong
> method. The generation counter is what makes the dead-frame case detectable.

### (b) Location + serial

Keep the index — one word, O(1), no search — and pair it with a counter that distinguishes
activations occupying the same location at different times. Minting copies the current counter value
into the name. Validating is one comparison. This is the **generational index** pattern, and the
whole of it is that a monotonic witness restores meaning to bit-equality of the location half.

Its cost, stated up front: **validity is checked, never guaranteed**. Nothing stops you holding a
stale token indefinitely; a type checker sees a pair of integers and has no opinion about whether
they currently denote anything. And a fixed-width counter wraps — held until [the bill](#what-it-costs).

### (c) A heap-allocated activation object — and why Doc 3 already foreclosed it

The other genuinely different answer: stop putting activations in a reusable array. Make each one a
real heap object. Smalltalk-80's `MethodContext`/`BlockContext` work this way, as do CPython's frame
objects. Once an activation is an ordinary object, "is it still live" stops being a bespoke problem
and becomes an instance of a question the memory manager already answers for everything. A strong
reference makes liveness a *real query with a real answer* — there is an actual object to ask.
Activations become reifiable, inspectable, in Smalltalk's case resumable.

What it forecloses is exactly what makes (b) attractive: activations can no longer be `Copy` values
in a flat array. **Every call becomes a heap allocation**, plus ongoing mark/scan traffic for as long
as any reference survives.

Note what this means for Phalcom, and note the direction of causation, because it is the opposite of
what you would guess. Phalcom did not evaluate (c) and reject it on cost. Doc 3 showed that
`CallFrame` is `Copy` because **ADR-0009** made every cross-object link a `Copy` handle — a decision
taken for entirely unrelated reasons, about the heap. `Copy` frames in a `Vec` followed from that;
branch (c) was foreclosed as a side effect, before frame identity was a question anyone had asked.
By the time ADR-0013 was written, the only live fork was (a) vs (b) — which is exactly the fork the
ADR records.

### (d), (e), (f) — briefly, and one of them for the wrong reason

**(d) A side liveness table.** Keep frames dumb and maintain a separate bitset or hash set of live
locations; validation becomes a lookup rather than a compare. Buys a frame representation untouched
by the scheme, at the cost of a second source of truth that must stay in lockstep with every site
that creates, recycles, or bulk-truncates frames. A missed update is a silent soundness hole rather
than a structural guarantee — and in a design whose unwind primitive is a single `Vec::truncate`
that can drop a hundred frames at once, "every site that recycles a frame" is a harder set to
enumerate than it sounds.

**(e) Static escape prevention** — prove at compile time via linear types, regions, or borrow-checked
lifetimes that no reference to an activation outlives it, removing both the check and the failure
mode, at the cost of a type system able to express the proof and a heavy constraint on closures that
escape dynamically.

**(f) Unforgeable capability / nonce** — cut, and cut for answering a *different question*. A
capability guarantees a name cannot be **counterfeited** by someone never given one. That is
authentication. It says nothing about whether a legitimately-issued name still points at something
alive; a capability scheme still needs (b), (c), or (d) underneath it to answer liveness.
Unforgeability and liveness are independent axes.

---

## Phalcom's token — and the first place theory is wrong about it

```rust
// phalcom-core/src/frame.rs::FrameToken (~L19)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameToken {
    /// The VM frame index the token refers to.
    pub frame_index: usize,
    /// The generation associated with that frame activation.
    pub generation: u64,
}
```

Branch (b), then. But the standard account of branch (b) — the one every generational-arena library
implements — says the counter **lives with the slot**: each array position keeps its own generation,
bumped whenever that position is reused, and validation reads *the slot's* current generation.

**Phalcom's counter does not live with the slot. There is no slot metadata at all.** A `Vec<CallFrame>`
position has no identity, no header, nothing to stamp. The generation lives on the **occupant**:

```rust
// phalcom-core/src/frame.rs::CallFrame (~L78)
/// Monotonically-assigned generation for this activation.
pub generation: u64,
```

So validation is not "ask the slot whether it has been reused." It is *"look at whatever frame is
sitting at that index right now, and ask whether it is the one I meant"*:

```rust
// phalcom-core/src/vm/dispatch.rs, Bytecode::ReturnNonLocal (~L1130)
let is_live = self.frames
    .get(token.frame_index)
    .is_some_and(|home| home.generation == token.generation);
```

`.get()` handles "that index no longer exists" (the stack shrank); the compare handles "that index
exists but is someone else." One expression, both failure modes. The inversion matters because it is
what makes the counter *global* rather than *per-slot* — a slot has no counter to be per-. And the
global counter is the thing this entire document is about.

Two more type-level details worth reading off the declarations, because they encode real rules:

- `BlockObject::home_frame_token` is a **bare** `FrameToken` (`heap/block.rs` ~L22). Every block, by
  construction, has a home. There is no such thing as a homeless block.
- `CallFrame::home_frame_token` is an **`Option<FrameToken>`** (`frame.rs` ~L94), populated *only*
  for a block invocation. An ordinary method activation has none; its `return` compiles to
  `Bytecode::Return` and never reads it.

So the type system does encode *which* activations may non-locally return. It does not, and cannot,
encode whether a token is still valid — that remains a runtime question, which is branch (b)'s
standing cost.

---

## The lifecycle: mint, stamp, carry, check

**Mint** — `vm/dispatch.rs::VM::new_call_frame` (~L29). Doc 3 already noted this builds but does not
push:

```rust
let generation = self.next_frame_generation;
self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
let mut frame = CallFrame::new(closure, context, ip, stack_offset, caller_source);
frame.generation = generation;
```

`CallFrame::new` itself leaves `generation: 0`; every caller must stamp. Four sites push a frame and
all four are stamped — but only three route through `new_call_frame`. `interpret.rs::run_in_module`
(~L170) open-codes the same read/bump/stamp inline rather than calling the helper. It is duplicated,
not omitted: no frame reaches the stack at the constructor's `generation: 0` default. Stated as a
fact about the code, which is all it is.

**Stamp** — the token is minted for the *creating* frame at the moment a block literal is evaluated:

```rust
// vm/dispatch.rs, Bytecode::Closure (~L602)
let token = self.current_frame_token().expect("closure created inside a frame");
let block = self.heap.alloc(Object::Block(BlockObject::new(new_closure, token)));
```

```rust
// vm/dispatch.rs::VM::current_frame_token (~L50)
self.frames.last().map(|frame| frame.token(self.frames.len() - 1))
```

`frame_index` is `frames.len() - 1` — the innermost frame *right then*. You can watch this happen.
Disassembling `l.each { x => System.print(x) }`:

```
0007: GetGlobal(6)
0008: Closure(7)      <- the BlockObject is materialized and stamped HERE
0009: Invoke(1, 8)    <- and only THEN is `each` called
```

The `Closure` opcode runs *before* the `Invoke`. The home recorded is therefore always the activation
that was executing when the `{ ... }` text was evaluated — not the method that eventually invokes the
block, and not transitively further out. That ordering is the whole reason the token means "lexically
enclosing."

**Carry.** On invocation, `primitive/block.rs` (~L151) copies the token off the `BlockObject` and onto
the pushed `CallFrame`:

```rust
frame.home_frame_token = home_frame_token;
vm.frames.push(frame);
```

Why the detour? `frame.rs`'s own doc answers it (~L83): the `BlockObject` "is not otherwise reachable
from a live `CallFrame`, which only stores the `ClosureObject` handle." A `CallFrame` points at
compiled code, not at the block wrapper that carried the token. So the token is re-homed onto the
frame, and `ReturnNonLocal` reads "what am I unwinding to" straight off the frame it is already
executing in.

**Check.** Covered above, and revisited under [failure atomicity](#failure-atomicity) — because
*when* it happens turns out to matter as much as *what* it compares.

---

## Predict, then check

Everything so far works within one call stack. Now push on it.

A block escapes its home method — the token in it is already stale. But instead of being invoked on
the same stack, it is handed to **a different fiber** as that fiber's entry point:

```phalcom
class Maker {
  make() { return { return 1 } }
}
let escaped = Maker.new().make()
let f = Fiber.new(escaped)
f.call()
```

Doc 3's Lie #2 told you `VM::frames` is a *live mirror*: a fiber switch `mem::take`s the whole `Vec`
out and swaps another one in. So by the time this block runs, `VM::frames` is a **completely
different array** — one that was built from scratch, that numbers its slots from zero, and that very
plausibly has a live frame sitting at whatever index the stale token names.

The token's `frame_index` is not merely stale here. It is *meaningless* — an index into an array it
was never measured against, which nonetheless is in range and does contain a real, live frame.

**Before reading on: what stops this token from validating against that unrelated, live frame?**

Take the question seriously, because it has a wrong answer that sounds right. "The generation won't
match" is only true if the generation was drawn from a numbering the second fiber shares. If each
fiber owned its own counter — the natural refactor, since each fiber already owns its own frames,
stack, and open upvalues — then both counters start at zero and march upward independently, and a
token minted as `(1, 7)` on fiber A would validate, cleanly and silently, against the unrelated frame
that happens to be `(1, 7)` on fiber B. The compare would pass. The VM would truncate a stack it has
no business truncating.

The answer is that `next_frame_generation` lives on the **VM**, not the fiber:

```rust
// phalcom-core/src/primitive/fiber.rs::store_live_into (~L30)
let frames = std::mem::take(&mut vm.frames);
let stack = std::mem::take(&mut vm.stack);
let open_upvalues = std::mem::take(&mut vm.open_upvalues);
let checking = std::mem::take(&mut vm.checking);
```

Four fields move. `next_frame_generation` is not among them, and an exhaustive search of every write
to it in `phalcom-core/src` finds exactly two increment sites (`dispatch.rs` ~L37, `interpret.rs` ~L171)
plus one initializer — none inside any fiber-switch path. The counter is never swapped, parked, or
reset. Every activation the VM has ever pushed, on any fiber, has a distinct serial.

**And this is written down.** It is not a happy accident anyone is retrofitting a rationale onto —
ADR-0030 §6 states it as a named invariant:

> Once `self.frames` is the *current* fiber's vector, [ADR-0013]'s `ReturnNonLocal` searches only
> that fiber; a token whose home is on another fiber fails the generation check → `DeadFrameError`.
> **Invariant:** the VM-global monotonic `next_frame_generation` counter **must not** be relocated
> into `FiberObject` — it is the only thing making a cross-fiber token globally non-matching.

Run it:

```
$ phalcom fiber_cross_fiber_non_local_return_dead_frame.ph
non-local return from a block whose home method frame is no longer alive (DeadFrameError)
```

Byte-identical to the same-fiber case. There is no fiber-aware code on this path at all — no fiber id
in the token, no third component, no special case. **The cross-fiber hazard is closed by the scope of
a counter**, and closed so completely that the two golden tests exercise structurally different
situations through one unbranched compare.

That is the grip, earned: the cheap half is ambiguous in *two* dimensions — recycled within a fiber
by `truncate`, and meaningless across fibers by the swap — and the expensive half is unique across
both. The design survives a hazard it never has to mention.

---

## Failure atomicity

Look at the order of operations in `ReturnNonLocal`, and at the comment justifying it:

```rust
// A stale index or a generation mismatch means the home method already
// returned — raise `DeadFrameError` *before* touching any VM state, so a
// caught error leaves the stack consistent.
let is_live = ...;
if !is_live { return Err(RuntimeError::DeadFrameError.into()); }

let home_stack_offset = self.frames[token.frame_index].stack_offset;
let return_value = self.stack.pop().unwrap_or(Value::Nil);
let return_value = self.surface_absence(return_value);
self.close_upvalues_from(home_stack_offset);
self.stack.truncate(home_stack_offset);
self.stack.push(return_value);
self.frames.truncate(token.frame_index);
```

Not one byte of VM state is mutated before the check. This ordering has a name: **failure atomicity**,
more usually discussed as the **strong exception guarantee** — an operation either completes in full
or has no visible effect at all. *(The basic / strong / nothrow taxonomy is due to David Abrahams,
from mid-1990s C++ STL exception-safety work; the attribution is confident, the exact original venue
is not, so it is offered as attribution rather than citation.)*

An implementation that unwound optimistically and checked partway through would leave a **torn**
machine: some frames popped, some upvalues closed, then a failure — with no state the recovery code
can trust. Worse, whether recovery is safe would become a property of *how far the unwind happened to
get*, an accident of statement order rather than a designed invariant.

**And here the guarantee is load-bearing rather than theoretical, because `DeadFrameError` is
catchable.** Run it:

```phalcom
try {
  System.print(escaped.call())
} catch e {
  System.print("caught: " + e.message)
}
System.print("after")
```

```
caught: non-local return from a block whose home method frame is no longer alive (DeadFrameError)
after
```

Execution continues, on the same VM, with the same stack — which is exactly the situation the
check-first ordering exists to make safe. A torn stack here would not be an abstract soundness worry;
it would be the state the *next* line of user code runs against.

> **Doc/code mismatch, verified at HEAD.** The rustdoc on `primitive/block.rs::block_on` (~L226) says
> `DeadFrameError` is among the outcomes `on` does *not* catch — "`on` catches only `Raise`;
> re-propagated unchanged." The code directly beneath it (~L253) wraps **any** non-`Raise` error into
> a kernel `Error` instance and lets `isA` decide, so a catch-all `catch e` matches it. The observed
> output above is the code's behavior; the comment is stale. Flagged here rather than repeated as
> fact, and not fixed — that is a source change, not a doc change.

---

## The hard trace: unwinding past two blocks and a `.ph` method

The escaping-block failure is the easy case, and `frames.md` and `upvalues.md` have both already shown
it. Trace the one that *succeeds* under load —
`tests/lang/blocks/blocks_non_local_return_two_deep.ph`, which prints `8`:

```phalcom
class Finder {
  findFirstEven(numbers) {
    numbers.each { n =>
      (n > 0).ifTrue {
        (n % 2 == 0).ifTrue { return n }
      }
    }
    return None
  }
}
```

Intuition says: `return` sits three block-literals deep, inside a method called by another method, so
the unwind must walk out through several block frames. Intuition is wrong twice.

**First: the two `ifTrue` blocks are not blocks at runtime.** They are sacred-inlined
([Doc 5](caches-and-fusion.md)) — compiled straight into the enclosing chunk. Disassembling a nested
`ifTrue` shows what actually gets emitted:

```
0005: GuardBool(21)
0006: JumpIfFalse(18)
   ...
0012: GuardBool(8)
0013: JumpIfFalse(5)
```

No `Closure` opcode. No `BlockObject`. No frame. Both levels collapse to guards and jumps. So
`return n`, though lexically two `ifTrue`s deep, is just more bytecode inside the **one** real block
— `{ n => ... }` — and compiles to `ReturnNonLocal` because *that* is the block literal it lives in.

**Second: `each` is not a native.** `List` has no `each`; it inherits `Iterable#each` (`core.ph` ~L654),
which is ordinary Phalcom:

```phalcom
each(f) {
  for (x in self) {
    f.call(x)
  }
}
```

That is a real method activation, pushed by an ordinary send, with `home_frame_token: None`.

So at the instant `ReturnNonLocal` executes, four frames are live:

| idx | frame | pushed by | `home_frame_token` |
|---|---|---|---|
| 0 | module top level | `run_in_module` | — |
| 1 | `findFirstEven(numbers)` | ordinary `Invoke` | `None` |
| 2 | `Iterable#each(f)` | ordinary `Invoke` | `None` |
| 3 | `{ n => ... }` body | `block_call` from `f.call(x)` | `Some((1, g₁))` |

`frame_index` is **1**, not 3 and not 2, for the reason the `Closure`-before-`Invoke` ordering
established: the block literal was evaluated inside `findFirstEven`'s own bytecode, before `.each`
was ever called, when `frames.len() - 1 == 1`.

Then:

- `frames.get(1)` is still `findFirstEven`, never popped, generation intact → **live**.
- `home_stack_offset = frames[1].stack_offset`.
- `close_upvalues_from(home_stack_offset)` — one call covers *all* three doomed frames, because each
  one's `stack_offset` is at or above the home's. Any capture escaping any of them is promoted before
  the storage is reclaimed.
- `stack.truncate(home_stack_offset)`, then `stack.push(8)`.
- `frames.truncate(1)` — keeps `[0, 1)`. **Frames 1, 2 and 3 all vanish, including the home frame
  itself.**

That last point looks like an off-by-one and is not. `token.frame_index` is the home frame's own
absolute index, and `Vec::truncate(n)` keeps `[0, n)`. Truncating *to* the home index drops the home
frame — which is exactly what an ordinary `Bytecode::Return` executed by frame 1 would have done
(`self.frames.pop()`). Truncating to `frame_index + 1` would leave `findFirstEven` on the stack
forever, mid-flight, with its value already pushed below it. The value now sits exactly where a
normal return from `findFirstEven` would have left it.

*(The four-frame layout is derived from verified pieces — the `Closure`/`Invoke` emission order and
the absence of a `Closure` opcode for `ifTrue` are both from real disassembly above; `Iterable#each`
was read from `core.ph`. It was not captured from a live frame dump: the `vm-trace` feature produces
no output at HEAD, because the CLI binary hardcodes its tracing filter to `LevelFilter::OFF`, and
`disasm` only walks the top-level module chunk, never method or block bytecode.)*

---

## How it stops: the knot closes on Doc 1

The handler ends without returning anything. The comment is explicit that this is deliberate:

> Do NOT `return Ok(_)` here — let the loop continue.

Doc 1 showed you the dispatch loop's halt condition, and it is completely untouched by any of this:

```rust
// vm/dispatch.rs::run_until_inner (~L490)
if self.frames.len() <= base_frames {
    let result = self.stack.pop().unwrap_or(Value::Nil);
    return Ok(self.surface_absence(result));
}
```

Three nested `run_until` calls are on the *Rust* stack when `ReturnNonLocal` fires, because
`f.call(x)` re-enters the interpreter natively (`block_call` bumps `native_reentry_depth` and calls
`run_until`). The innermost of them was entered with `base_frames = 2`. After the truncate,
`frames.len()` is `1`. On its very next top-of-loop check, `1 <= 2` — it drains the pushed value and
returns it as an ordinary `Ok`, up through `block_call`, through `call_method`'s primitive arm, and
onward. The next loop out, entered with `base_frames = 1`, sees `1 <= 1` and does the same.

**A non-local return does not terminate by returning. It rearranges state so that the ordinary halt
condition becomes true, and then gets out of the way.** This is why the whole unwind has to happen
eagerly, in one shot, inside the handler: control cannot reach the home frame any other way, because
Rust frames belonging to `run_until` and `block_call` sit *between* the block and its home, and the
only thing all of them share is `self.frames`. Mutate the shared structure, and every intervening
Rust frame unwinds itself on its own next check.

That is the knot's last strand. Doc 1's drain check looked, at the time, like a trivial "are we
done." It is also the exit mechanism for a control-flow feature that had not been introduced yet.

---

## What it costs, and what a generation cannot buy

**It costs the collector nothing — and buys no keep-alive whatsoever.** `heap/trace.rs` (~L35) is
explicit:

> `home_frame_token` is **not** an edge: a `FrameToken` is an index plus a generation counter, not a
> handle.

This is a stronger statement than "weak." A weak reference is still a category the collector must
know about, walk, and clear. A `FrameToken` is two integers, as inert to the collector as a hash code
— nothing traces it, nothing clears it, nothing needs to know it exists. `vm/gc.rs::collect_roots`
destructures `VM` exhaustively (so a newly added field fails to compile until classified) and lists
`next_frame_generation: _` among the non-roots.

The consequence cuts both ways and both halves matter. Contrast branch (c): a **strong** reference to
a heap activation would leak in precisely the case this mechanism exists for — an escaped,
never-invoked-again block would pin its dead home activation, every local in it, and everything those
locals reach, forever, purely so that a hypothetical future call could report an error. Phalcom's
token cannot leak, because it does not retain. It also cannot protect: holding it does not delay the
slot's reuse by one instruction. **All a token can ever do is be compared, after the fact.**

**Wraparound is unhandled at HEAD.** `next_frame_generation` is a bare `u64` incremented by
`wrapping_add(1)`, with no guard, no reserved tombstone value, no comment, no test, and nothing in
any ADR discussing exhaustion. At 64 bits, wrapping requires a number of activations not reachable at
any plausible call rate within any plausible process lifetime, and "do nothing" is the standard
industry answer at that width. But the honest form is: **that reasoning is not written down anywhere
in this repository.** It is an absence, not a documented judgment, and this document is not going to
manufacture the judgment on the code's behalf.

**Detection, never prevention.** The token cannot stop the escape. `DeadFrameError` is a runtime
answer to what branch (e) would answer at compile time. Phalcom bought generality — a block can be
stored in a list, handed to a native, or made a fiber's entry point, all decided at runtime — and paid
with a failure mode that has to be documented, tested, and (as it turns out) caught.

**And the runtime cost of the check itself is unmeasured.** It is one bounds-checked index plus one
`u64` compare, on the `ReturnNonLocal` path only — it is not in the dispatch loop's hot path and does
not appear in `docs/forge/perf-log/SCOREBOARD.md`. No number is quoted here because no number exists.

---

## The comparisons that earn their place

**Smalltalk-80 — the ancestor, and a structurally different version of the problem.** Activations are
objects: `MethodContext` for a method, `BlockContext` for a block. `^` in a block targets the block's
*home context*, recorded at creation — the same idea as `home_frame_token`, one abstraction level up.
`blocks.md` §5 names Smalltalk's failure `BlockCannotReturn`. *(Squeak, Pharo and VisualWorks spell the
exact class and selector differently; the mechanism is certain, one canonical identifier is not.)*

The paragraph worth isolating: because a `MethodContext` is an ordinary object, "is my home still
live" is not a bespoke question needing bespoke machinery — it is an instance of a question the object
memory answers for everything. A returned context does not *cease to exist*; it stops being on an
active chain and becomes ordinary garbage. Reuse, in an array-of-frames design, is not a state
transition experienced by a continuing object — it is a **literal overwrite** of storage that used to
mean one thing and now, bit-for-bit, means another, with nothing in between. **The generation counter
exists to manufacture, for two words and no allocation, an approximation of the enduring identity a
real heap object gets for free by being a real heap object.** Smalltalk pays for the real version with
a heap allocation per call. Implementations that wanted cheap call/return moved activations onto flat
reusable storage — and then had to reinvent, out of smaller parts, an identity guarantee heap objects
had never given up. Phalcom is downstream of that trade, though it arrived there via ADR-0009's
handles rather than by re-running the argument.

**Generational arenas and ECS entity ids — the pattern with a name.** This is the highest-value
comparison, because it is the one place the mechanism is explicitly *named* in circulation:
"generation," "generational index." Game engines pack an index and a version into an entity handle
precisely so a stale handle to a despawned entity fails cleanly instead of silently addressing
whatever now occupies the slot; `slotmap` and `generational-arena` ship it as a general Rust
primitive. Phalcom uses the same pattern twice, with two different failure modes — and
`upvalues.md` already drew that rhyme (`ObjRef` resolving to `None` versus `FrameToken` raising
`DeadFrameError`), so it is not re-drawn here. What is *new* is the inversion noted above: the
libraries put the generation **on the slot**; Phalcom puts it **on the occupant**, because a `Vec`
slot has nowhere to put one.

**Rust — the same question answered twice, at two phases.** Statically, the borrow checker is branch
(e) shipped in a mainstream language: it proves references don't outlive their scope, which is a large
part of why "capture a way to jump back to a specific caller" is not a pattern Rust programs reach
for. Dynamically, wherever a Rust program *does* want a recyclable-slot handle whose target may be
gone, the ecosystem reaches for branch (b) — because the borrow checker cannot see into a
runtime-managed arena. The two are not competitors; they partition one question by whether the escape
is statically analyzable. Phalcom, hosted *in* Rust, uses the language that can do (e) to implement
(b) — because its own users' escapes are not statically analyzable, by design.

**JavaScript — the bill for not having the feature.** A `forEach` callback cannot reach its caller's
control flow:

```js
arr.forEach(x => {
  if (x === target) return;   // does NOT stop the loop
});
```

No error. That is worse than an error: it fails *silently*, iterating past the point the author meant
to stop. `some`/`every` exist as workarounds precisely because they let the *iterating method*
interpret an ordinary local return as a stop signal — the only lever available. This is the zero-cost
degenerate case of detection-versus-prevention: prevention not by proof, but by omission. Phalcom's
`_two_deep` fixture is the program JS cannot write, and `DeadFrameError` is the invoice.

**Cut:** *Java* (lambdas have no non-local return — a second, redundant "doesn't have it" after JS).
*Go* (its scar is loop-variable capture, which is the **data** half, and blurring it would undo this
doc's opening distinction). *Lua* (its upvalue closing is the data half and already spent in
`upvalues.md`; `error`/`pcall` is general unwinding, not activation naming). *C#* (same shape as the
Java cut, and already spent in `upvalues.md`).

---

## What you can now re-derive

Delete `FrameToken`. From three constraints —

1. **frames are `Copy` values in a recycled `Vec`** (Doc 3, downstream of ADR-0009),
2. **a block can outlive the activation it names** (Doc 2 / `upvalues.md`), and
3. **the failure must be recoverable, not undefined** —

you rebuild the whole mechanism. (1) forecloses heap-object activations, so identity must be
manufactured rather than inherited. Recycling means an index alone is ABA-vulnerable, so the name
needs a second component. (1) again means there is no slot to hang a counter on, so the counter goes
on the occupant and is therefore global rather than per-slot. Global-per-VM (not per-fiber) is then
forced the moment a token can cross a fiber boundary — which (2) permits. And (3) forces the compare
to precede every mutation, because a caught `DeadFrameError` resumes on the same stack.

Two fields, one compare, one ordering rule. Everything else — the eager one-shot unwind, the
truncate-to-`frame_index`, the hand-off to Doc 1's drain check — follows from frames being an array
whose bookkeeping collapses to a length.

**Lie #1, from [`frames.md`](frames.md#lie-1), is now destroyed.** `generation` and `home_frame_token`
were never "just fields." They are the identity mechanism that branch (b) of Doc 3's fork owed, and
this is the bill being paid.

---

## Anchors

- `phalcom-core/src/frame.rs::FrameToken` (~L19) — the pair. `CallFrame::generation` (~L78),
  `CallFrame::home_frame_token` (~L80–94, including the rustdoc explaining the re-homing).
- `phalcom-core/src/heap/block.rs::BlockObject::home_frame_token` (~L22) — bare, not `Option`.
- `phalcom-core/src/vm/dispatch.rs::VM::new_call_frame` (~L29) — mint;
  `VM::current_frame_token` (~L50) — `frames.len() - 1`; `Bytecode::Closure` (~L602) — stamp site;
  `Bytecode::ReturnNonLocal` (~L1110–1161) — check, unwind, no `return Ok`.
- `phalcom-core/src/vm/dispatch.rs::VM::run_until_inner` (~L490) — the unmodified drain check that
  terminates the unwind.
- `phalcom-core/src/primitive/block.rs` (~L151) — carry onto the frame; (~L226 doc vs ~L253 code) —
  the catchability mismatch.
- `phalcom-core/src/primitive/fiber.rs::store_live_into` (~L30) — the four fields that swap, and the
  one that doesn't; (~L272, ~L305) — carrying a token onto another fiber's stack.
- `phalcom-core/src/vm/mod.rs::VM::next_frame_generation` (~L109) — VM-global.
  `phalcom-core/src/interpret.rs::run_in_module` (~L170) — the duplicated stamp.
- `phalcom-core/src/heap/trace.rs` (~L35, ~L143) — not a GC edge. `vm/gc.rs::collect_roots` (~L83) —
  classified non-root by exhaustive destructure.
- `phalcom-core/src/error.rs::RuntimeError::DeadFrameError` (~L138–151).
- ADR-0013 (`0013-closure-upvalues-and-frame-token-return.md`) — Decision + the two Alternatives.
  ADR-0030 §6 — the "must not be relocated into `FiberObject`" invariant. ADR-0009 — why frames are
  `Copy` at all. `docs/spec/current/blocks.md` §5 — the surface promise.
- Fixtures: `blocks/blocks_non_local_return{,_bare,_two_deep,_in_loop}.ph`,
  `control-flow/control_flow_inline_non_local_return.ph` (inlined `if`/`while` is an *ordinary*
  return — a useful contrast), `runtime-errors/runtime_non_local_return_dead_frame.ph`,
  `concurrency/negative/fiber_cross_fiber_non_local_return_dead_frame.ph`, and the Rust invariants
  `invoke_on_preserves_dead_frame_fencing_for_escaping_blocks` /
  `cross_fiber_non_local_return_raises_dead_frame_error` (`phalcom-core/tests/invariants.rs`).

## Unverified, partial, or open — stated rather than smuggled

- **ADR-0013's claim that "the frame token also unifies with `throw` and fiber `abort` as one
  stack-unwinding primitive" is partial at HEAD.** `VM::unwind_to` is a genuinely shared primitive
  and mirrors `ReturnNonLocal`'s close-then-truncate order exactly, but it takes raw lengths, not a
  `FrameToken`. `block_on`/`block_ensure` coordinate with non-local return by comparing
  `frames.len()`, not by sharing a token. No fiber-`abort`-via-token path exists at all. Shipped in
  *shape*, aspirational in *unification*.
- **`vm-trace` produces no output** — the CLI hardcodes `LevelFilter::OFF`; `disasm` covers only the
  top-level module chunk. The frame table in the hard trace is therefore derived from verified
  emission order and source, not captured live, and is labelled as such where it appears.
- **The `block_on` catchability rustdoc is wrong** (see the flag above). Reported, not fixed.
- **No perf number is quoted** for the liveness check; nothing in `perf-log/SCOREBOARD.md` measures
  it.
- **Wraparound behavior is undocumented at HEAD** — the risk assessment in this doc is the general
  one for 64-bit counters, not a claim about anything the repository decided.

## Forward pointers

- **Concurrency / fibers** — five VM docs now owe this one. This document leaned on the fiber swap
  (`mem::take` of four fields) as an established fact and on ADR-0030 §6 for the invariant, but never
  explained *why* `frames` is a per-fiber mirror in the first place, what a yield actually does, or
  how a fiber's failure interacts with open upvalues. It is the track's largest unpaid debt.
- **The sacred-selector inliner** — [Doc 5](caches-and-fusion.md) opened it; this doc leaned on it
  hard (the `ifTrue` blocks that are not blocks) and it deserves its own treatment.
- **`SuperSend`** — its own opcode, uncached, still unexplained (Docs 4 and 5 both point at it).
