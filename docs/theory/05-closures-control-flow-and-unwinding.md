# 05 — Closures, control flow, and unwinding

> **Thesis:** a closure is a promise that a variable outlives the frame that declared it, and a
> non-local return is a promise that a frame can be targeted after control has left it. Both
> promises are about *lifetime*, and both are kept by the same move: **never hold an address for
> something that can die — hold a name, and check.**

---

## 1. The upward funarg problem

**`[V]`** The canonical framing, cited in this repo at `docs/learn/vm/upvalues.md:41-43`: the term
traces to Joseph Weizenbaum's work on early Lisp implementations in the late 1960s, and the
canonical treatment is **Joel Moses's 1970 MIT AI Lab memo, *The Function of `FUNCTION` in LISP,
or Why the FUNARG Problem Should Be Called the Environment Problem***.

Moses's reframing is the whole content: the problem is not about functions, it is about
**environments**. A frame is a *loan*. When a closure escapes the frame that created it, the
loan is called in while the borrower still holds a reference.

**`[R]`** The design space that follows is a menu of ways to refuse or honor the loan, and every
language sits somewhere on it:

- **Refuse to allow the escape.** Classical Pascal and Algol 60 simply would not let you return a
  procedure. **`[V]`** The repo notes that GCC's nested-function extension reintroduces exactly
  this hazard, and dangling trampolines are its well-known consequence.
- **Allow escape, forbid mutation.** Java requires a captured local be `final` or effectively
  final. **`[V]`** The `final int[] count = {0}` idiom "is not a workaround to a missing feature —
  it is proof of the feature's necessity."
- **Heap-allocate every activation.** Smalltalk's `MethodContext`/`BlockContext` are ordinary heap
  objects, so the upward funarg problem **cannot arise, structurally**. The cost is an allocation
  per call.
- **Promote lazily.** **`[V]`** Deutsch and Schiffman's 1984 implementation keeps contexts in a
  stack-shaped region for the common case and promotes one to a real heap object only when
  something actually outlives the assumption.
- **Rewrite at compile time.** Assignment conversion — the Scheme literature's term — boxes only
  mutable captured variables.
- **Two-state cell: open, then closed.** Lua's answer, and Phalcom's.

**`[V]`** A separation worth stating because it is routinely conflated: *"Smalltalk-style object
model" and "heap-allocate every activation" are two separate axes.* Phalcom takes the first and
declines the second.

---

## 2. The address/name inversion

This is the sharpest single idea in the closure design, and it recurs three times in the codebase.

**`[R]`** Lua's `UpVal` holds an **address** — a `Value *location` pointing into the stack while
open. Closing uses the famous self-pointing trick: copy the value into the cell, then point
`location` at the cell's own field. The payoff is a genuinely **branchless** read: `*uv->location`
is correct in both states, forever.

**`[V]`** Phalcom's holds a **name**:

```rust
enum Upvalue { Open { fiber: ObjRef, slot: usize }, Closed(Value) }
```

The `fiber` field is the Phalcom-specific addition — vanilla Lua assumes one global stack. Because
an open upvalue never holds a stack pointer, only a `(fiber, slot)` pair resolved fresh on access,
a stale handle resolves to a clean `None` rather than undefined behavior.

**The trade, stated in both directions.** Lua pays: every subsystem that *moves memory* must walk
the open-upvalue list and rewrite pointers — stack growth does it, a moving collector does it, and
coroutine stacks have no recorded identity to rewrite *against*. Phalcom pays: the read path
branches twice (open versus closed; then owning fiber current versus parked), and closure creation
does a `BTreeMap` find-or-create even when it finds. **`[V]`** The repo names the consequence
honestly: "this is why 'closures in a hot loop' is a recurring caution across every ecosystem
sharing this design."

What Phalcom buys is that **the stack-realloc hazard does not exist and was not solved — it was
dissolved.** And it keeps a moving collector reachable, which given that the shipped collector is
non-moving and compaction is a live future option, is the more valuable half.

**`[V]`** What it *precludes*, stated plainly: the branchless read is now unreachable without
changing the representation.

**`[V]`** The generalization the repo draws, and the reason this file exists: "**Phalcom never
holds an address for something that can die. It holds a name, and checks.** One idea, applied
three times… When the codebase looks like it has many moving parts, it usually has one part,
moved." The three applications are upvalues (`(fiber, slot)`), object references (`ObjRef` index +
generation), and frame targets (`FrameToken`).

---

## 3. The identity invariant, and ordering that is load-bearing

**`[V]`** Two closures capturing the same variable must share **one cell**, or a write through one
is invisible to the other. Enforced by find-or-create keyed on the absolute stack slot, in a
**sorted `BTreeMap<stack_index, ObjRef>`** — sorted specifically so that closing is a cheap range
scan: everything above a threshold is dead.

Two ordering facts that are correctness, not style:

- **`close_upvalues_from` must run *before* `stack.truncate`**, while the slot values still exist.
  Reverse it and you close upvalues over freed slots.
- **`ReturnNonLocal` closes all the way down to the home frame's stack offset**, not merely the
  innermost scope — because everything between the block and its home is also about to vanish.

**`[V]`** Closing is **idempotent by construction**, because `close_upvalues_from` removes what it
closes. That is worth noticing as a technique: idempotence achieved by making the operation
consume its own input, rather than by a flag someone must remember to check.

---

## 4. Per-iteration freshness, and a bug three languages shipped

**`[V]`** The loop-variable capture bug — `fns.forEach(f => f())` printing `3, 3, 3` — is solved
at compile time. The compiler emits `CloseUpvalue(binding_slot)` at the step label, *before* the
cursor advance and rebind, **but only if `Local::is_captured` was set during body compilation**.
Each iteration's cell is promoted to `Closed` holding that iteration's value; the next iteration
opens a fresh one. The `is_captured` gate avoids churn in the common case.

**`[R]`** The comparative record is unusually rich, and it is a good illustration of how long a
default survives once shipped:

- JavaScript's `var` had it; `let` fixed it.
- **`[V]`** Go shipped the identical bug from 2009 and changed the default to a fresh per-iteration
  variable in **Go 1.22 (2024)** — fifteen years.
- **`[V]`** C# 5.0 (2012) made a rare breaking change to `foreach` to fix it, and **pointedly did
  not change the C-style `for` loop**.

**`[V]`** That last asymmetry produced the rule this project adopted, and it was reached by
*joining two independent findings* — one about C#'s history, one about Phalcom's own behavior:

> **The construct that hands you an element gets a fresh binding. The construct where you visibly
> mutate your own counter does not.**

The justification is about user expectation rather than implementation: in `for x in xs`, `x` is
delivered per iteration and the user never wrote a mutation, so freshness is what they meant. In
`while (i < n) { … i = i + 1 }`, the user is visibly mutating one variable, and silently giving
them a fresh one per iteration would be the surprise.

---

## 5. Frame tokens: converting a memory hazard into a language error

**`[V]`** A `FrameToken` is `(frame_index: usize, generation: u64)`. `new_call_frame` bumps a
VM-global counter and stamps every activation. A block literal is stamped with the current frame
token at creation; `block_call` copies that token onto the executing frame *before* pushing;
`ReturnNonLocal` validates `frames.get(idx).generation == token.generation`.

**`[V]`** The repo's own framing is the best available summary of why this design exists: the token
is "**a pointer deliberately split in two**" — `frame_index` is *where to look* (fast, fiber-local,
recycled twice over: by `truncate` within a fiber and by the wholesale `mem::take` across fibers),
and `generation` is *who it was* (VM-global, never reused).

**The inversion worth stealing.** Every generational-arena library stamps the *slot*. Here the
counter lives on the **occupant**, because a `Vec` position has no header to stamp. Validation
therefore reads: *look at whatever frame is sitting at that index right now, and ask whether it is
the one I meant.* **`[V]`** And that inversion is precisely what forces the counter to be global —
which is precisely what closes the cross-fiber hazard **with no fiber-aware code on the path at
all.** A representational constraint produced a concurrency guarantee for free.

**`[V]`** Three disciplines make it sound:

1. **Check before mutate.** The liveness compare precedes every state change, so `DeadFrameError`
   is catchable with a consistent stack. This is the strong exception guarantee, applied to an
   interpreter's own bookkeeping. (The repo attributes the basic/strong/nothrow taxonomy to David
   Abrahams and explicitly labels it *attribution rather than citation* because the venue is
   uncertain — a small piece of discipline worth copying.)
2. **One-shot unwind with no explicit return.** The handler closes upvalues, truncates stack and
   frames, pushes the value, and deliberately does *not* return `Ok` — it lets the dispatch loop's
   existing drain check fire naturally, and every intervening native `run_until` unwinds itself on
   its own next check.
3. **`ReturnNonLocal` carries no operand.** The target lives entirely in
   `CallFrame::home_frame_token`. That is what lets it unwind correctly across nested re-entrant
   interpreter invocations.

**`[V]`** What the token deliberately **cannot** do: it does not retain. A strong reference to a
heap activation would leak in exactly the case the mechanism exists for — an escaped,
never-invoked-again block would pin its dead home activation and everything reachable from it,
forever, purely so a hypothetical future call could report an error. So: "**Phalcom's token cannot
leak, because it does not retain. It also cannot protect.**" **Detection, never prevention.**

**`[R]`** The comparative bill: Ruby's `lambda` and `proc` differ in exactly and only this respect,
and Ruby answers with `LocalJumpError`. C's `setjmp`/`longjmp` gives you the jump and simply does
not offer the check. And JavaScript pays for not having the feature at all —
`arr.forEach(x => { if (x === target) return; })` does not stop the loop, which is *worse* than an
error because it fails silently.

**`[V]`** One honest gap, recorded rather than smuggled: generation wraparound is unhandled — a
bare `u64` with `wrapping_add(1)`, no guard, no tombstone, no test, nothing in any decision record.
The repo's note is a model of how to write this kind of thing: "It is an absence, not a documented
judgment, and this document is not going to manufacture the judgment on the code's behalf."

---

## 6. One unwind primitive

**`[V]`** `return` from a block, `throw`, and fiber `abort` are **one** stack-unwinding primitive.
The direct consequence is that `ensure` fires on *any* of them — you do not get three cleanup
stories and a matrix of which ones run.

**`[V]`** The error surface is pure sugar over message sends. `try` / `on T e` / `catch e` /
`ensure` are contextual keywords desugaring to `.on(Class){handler}` and `.ensure(cleanup)`, with
each subsequent clause wrapping the previous result in a fresh zero-parameter block. `catch e` is
literally `.on(Error){e => …}` where `Error` is the hierarchy root. At the primitive level,
`block_on` snapshots `(stack_len, frames_len)`, sends `isA(_)` through **ordinary `.ph` dispatch**,
and calls `unwind_to` on a match — so first-match-wins chaining falls out for free, with **zero
special-casing in the dispatch loop**.

**`[V]`** `block_ensure` never catches. It runs cleanup on every exit path, and a cleanup that
itself raises or non-locally-returns **supersedes** the pending outcome.

**`[V]`** Only two native primitives were admitted for all of this (`Block#on`, `Block#ensure`),
and the admission argument is the one that matters: implementing them in the language is
*impossible*, not merely slow — a guest-language body cannot see the VM's raise payload or the
non-local-return frame-shrink. That is the correct standard for a primitive.

---

## 7. Three failures worth studying

**`[X]` The probe that could not run.** `Block#on` implements typed catch by sending `isA(_)` to
the error — a full method dispatch, which needs a frame. It probed **before** unwinding. When the
error was a call-depth-ceiling error, `isA` could not get a frame to run in, making the depth error
**permanently uncatchable**. Fix: unwind immediately on any error, then probe.

> **When the exhausted resource is the stack, recovery logic cannot run inside the frame context
> that exhausted it.**

A separable defect in the same area: the depth error was a bare error variant rather than one
carrying a surface `Error`, and `try`/`catch` intercepts only the latter — so it bypassed the
exception system entirely, quietly violating the "single unwind primitive" model that section 6
describes.

**`[X]` The guessed ceiling that only held on one thread.** A native-reentrancy limit was proposed
at 200, by guess. Binary search: 128 aborts, 64 survives, **32 chosen with margin**. Root cause of
the mismatch: spawned threads and tests get 2 MiB stacks while main gets 8 MiB. The recorded
principle — *"A ceiling that holds only on the main thread does not protect the resource"* — comes
with a second one explaining why measurement was unavoidable: native stack overflow **aborts the
process**, so the check must precede the recursion. *"There is no after."*

**`[X]` An invariant asserted in a comment and false in the code.** The inliner's source claims the
inlined and non-inlined paths are observationally identical. They are not:
`(true).ifTrue { return "A" }` yields `"A"` inlined and **`Some("A")`** through the real send,
because the primitive wraps a payload that is actually an in-flight non-local return, which the
block-call path cannot distinguish from a completed value. Filed as a defect rather than papered
over. The lesson is section 6 of
[`06-mechanism-versus-policy.md`](06-mechanism-versus-policy.md) in miniature: a fast path must
deopt to *exactly* the slow-path result, and "exactly" includes the interaction with every other
control-flow feature, not just the happy path.
