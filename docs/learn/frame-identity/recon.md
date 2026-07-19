# Recon — frame identity (VM track, Doc 6)

Phase 1 per [AUTHORING.md](../AUTHORING.md). Cheap scout. Everything below was read at HEAD
(`79e5a3e`) or observed by running a program. Nothing is assumed.

---

## 1. Architecture vs representation

**Architecture (the shape).** Generational-handle validation, applied to the call stack. Same
algorithm family as a generational arena / slotmap: an identifier is a *pair* — a recyclable
location plus a monotonically-issued serial — and every dereference is
`lookup(location)` followed by `serial == expected`. Two states, live and stale; staleness is
*detected*, never prevented. The unwind itself is not a walk: it is a single `Vec::truncate`.

**Representation (what it holds).** `phalcom-core/src/frame.rs::FrameToken` (~L19):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameToken {
    pub frame_index: usize,
    pub generation: u64,
}
```

Not a pointer. Not a `Weak`. Not a handle into the heap arena. **A `usize` index into
`VM::frames` plus a `u64` serial** — so the token is `Copy`, and `Option<FrameToken>` keeps
`CallFrame` `Copy` (`frame.rs` ~L92 says exactly this).

**The consequence that only shows up in representation — and it is the doc's payload.** The two
halves have *different scopes*:

- `frame_index` indexes `VM::frames` (`vm/mod.rs` ~L53), which is **per-fiber**: `primitive/fiber.rs`
  `std::mem::take`s the whole `Vec` in and out on every switch (~L30/L51). An index is therefore
  meaningful only relative to whichever fiber's frames are currently mounted. It is recycled twice
  over — by `truncate` within a fiber, and by the swap across fibers.
- `generation` comes from `VM::next_frame_generation` (`vm/mod.rs` ~L109), which is **VM-global**
  and is *not* swapped by the fiber code. Every activation the VM has ever pushed, on any fiber,
  has a distinct serial.

So the cheap half is ambiguous in two dimensions and the expensive half is unique across all of
them. This is why the cross-fiber escape test lands on the same one-line compare as the
intra-fiber one, with no fiber-specific code — verified live in §5.

---

## 2. The grip (grounded)

> **A `FrameToken` is a pointer deliberately split in two: `frame_index` is *where to look*,
> `generation` is *who it was*. The first is fast, fiber-local, and recycled; the second is
> globally unique and never reused. Every non-local return dereferences with the first and is
> only ever *trusted* because of the second — and the entire safety argument is that the cheap
> half is never believed on its own.**

Corollary the doc must earn: this is not a *frame* mechanism. It is Phalcom's one universal
policy for naming something that can die (ADR-0009's `ObjRef` is the same pair with a different
failure mode: `None` instead of `DeadFrameError`). `upvalues.md` already named this rhyme; Doc 6
must go past it, not restate it (§6).

---

## 3. What was actually deliberated

`docs/adr/accepted/0013-closure-upvalues-and-frame-token-return.md`, *Alternatives considered* —
**two** entries, and only one is about identity:

1. **By-value snapshot capture** — a *capture* alternative, not an identity one. Rejected because
   it breaks shared mutation; noted as also fighting non-local return.
2. **Raw frame pointer with no generation counter.** Verbatim: *"a reused frame slot would alias a
   stale pointer to a live frame, silently returning to the wrong method. The generation counter is
   what makes the dead-frame case detectable."*

That is the whole deliberated space for identity: **generation stamp vs. raw pointer.** Anything
else the doc walks (`Weak`/refcount, a side liveness table, unforgeable capabilities, scanning the
frame array for the home closure, shadow-stack registration, Smalltalk's `BlockContext` as a real
heap object) is **pedagogical reconstruction and must be labelled as such** (§5.2 honesty pass).

The ADR also claims a forward consequence — *"the frame token also unifies with `throw` and fiber
`abort` as one stack-unwinding primitive"*. Agent B must check whether that unification exists at
HEAD or is aspirational; the doc must not repeat it unchecked.

**Filename drift, noted not fixed:** the ADR file is `0013-closure-upvalues-and-frame-token-return.md`,
but rustdoc across `frame.rs`, `dispatch.rs`, `bytecode.rs` links it as
`0013-block-closure-upvalues.md`. Doc 6 must cite the real filename.

---

## 4. Mechanism, as read (four points, all verified)

- **Mint.** `vm/dispatch.rs::VM::new_call_frame` (~L29): reads `next_frame_generation`,
  `wrapping_add(1)`, stamps `frame.generation`. Builds; does **not** push (four push sites — Doc 3
  established this). **A fifth, out-of-band site exists:** `interpret.rs::run_in_module` (~L170-173)
  open-codes the same read/bump/stamp instead of calling `new_call_frame`. The module entry frame is
  therefore stamped by a duplicated line, not by the mint function. Worth stating; it is a fact about
  the code, not a criticism to editorialize.
- **Stamp.** `dispatch.rs` `Bytecode::Closure` (~L602): `let token = self.current_frame_token()
  .expect("closure created inside a frame");` then `BlockObject::new(new_closure, token)`. So the
  token is captured at *block creation*, from the creating frame — `current_frame_token` (~L50)
  builds it as `frames.last().map(|f| f.token(self.frames.len() - 1))`.
- **Carry.** `heap/block.rs::BlockObject::home_frame_token` (~L22) holds it. On invocation,
  `primitive/block.rs` (~L151) copies it onto the *pushed `CallFrame`* — `frame.home_frame_token =
  home_frame_token`. `frame.rs` ~L83-91 states the reason outright: the `BlockObject` is not
  otherwise reachable from a live `CallFrame` (which stores only the `ClosureObject` handle), so
  `ReturnNonLocal` reads the token off the executing frame instead. Ordinary calls leave it `None`.
- **Check, then unwind.** `dispatch.rs` `Bytecode::ReturnNonLocal` (~L1110-1161). Order is
  load-bearing and commented as such: liveness compare → `DeadFrameError` **before any VM state is
  touched** ("so a caught error leaves the stack consistent"), then pop value, then
  `close_upvalues_from(home_stack_offset)`, then `stack.truncate`, then push value, then
  `frames.truncate(token.frame_index)`. And explicitly **not** `return Ok(_)` — the comment says to
  let the loop continue so the ordinary top-of-loop drain check yields the value.

That last point is the knot closing on **Doc 1**: a non-local return terminates by *arranging state
so the existing halt condition fires*, not by returning. Doc 1 owns that drain check.

Also read: `heap/trace.rs` (~L35, ~L143) — `home_frame_token` is deliberately **not** a GC edge,
"an index plus a generation counter, not a handle." A token does not keep its frame alive. That is
the difference from `Weak` stated in code.

---

## 5. Observed, not inferred

Both run at HEAD, output copied verbatim:

```
$ cargo run -q -p phalcom-core --bin phalcom -- \
    phalcom-core/tests/lang/runtime-errors/runtime_non_local_return_dead_frame.ph
non-local return from a block whose home method frame is no longer alive (DeadFrameError)

$ cargo run -q -p phalcom-core --bin phalcom -- \
    phalcom-core/tests/lang/concurrency/negative/fiber_cross_fiber_non_local_return_dead_frame.ph
non-local return from a block whose home method frame is no longer alive (DeadFrameError)
```

The second is the interesting one: the block escapes `make()` and is then run as a *different
fiber's* entry (`Fiber.new(escaped)` — `primitive/fiber.rs` ~L272 pulls `home_frame_token` off the
`BlockObject` and installs it on the new fiber's entry frame at ~L305). Different frames `Vec`,
same global generation counter, same one compare, same error.

Positive-lane companions exist and must be traced against, not just cited:
`tests/lang/blocks/blocks_non_local_return{,_bare,_two_deep,_in_loop}.ph`,
`tests/lang/control-flow/control_flow_inline_non_local_return.ph`. `_two_deep` unwinds past two
block frames *and* a native `each` frame and prints `8`.

---

## 6. Brief-steering notes

**What Doc 6 must not re-do.** `upvalues.md` (~L750-780) already prints the `is_live` snippet, the
`DeadFrameError` output, the Smalltalk `BlockCannotReturn` comparison, and the "index plus
generation, name then check" rhyme. `frames.md` (~L192, Lie #1) already promised the mechanism and
already showed the error firing. **Restating the compare is the failure mode for this doc.** The
new payload is: the two-scope split (§1), the ordering discipline, the token's residence on the
*frame* rather than the block, the eager one-shot unwind and its hand-off to Doc 1's drain check,
and what a generation *cannot* buy.

**Agent A — go deep on:** raw pointer vs. tagged/generational identity as a general naming problem
(the ABA problem by name, and where it comes from); `Weak`/refcount liveness and precisely what it
costs relative to a serial (allocation, non-`Copy`, an edge the GC must see); Smalltalk-80's
`BlockContext`/`BlockCannotReturn` and *why* a language whose contexts are real heap objects has a
different problem than one whose frames are array slots; the general "check before mutate" rule for
recoverable errors (failure atomicity). **One sentence each:** unforgeable capabilities, linear
types/regions preventing escape statically, scanning for the home frame. **Do not tell A which
branch Phalcom took**; ask for the space.

**Agent B — must confirm, with lines:**
1. The two-scope claim: `frames` swapped per fiber, `next_frame_generation` never swapped. Quote both.
2. Is `wrapping_add` reachable in practice — what happens on u64 wrap, and is there any guard? State
   plainly if there is none.
3. Does the ADR's "unifies with `throw` and fiber `abort` as one stack-unwinding primitive" exist at
   HEAD, or is it aspirational? Check `unwind_to` callers and `primitive/block.rs::block_on`.
4. What happens to a `DeadFrameError` raised *inside a native window* — `vm/send.rs` ~L74 and ~L257,
   `primitive/block.rs` ~L226 all mention it. Is it catchable by user `on(_)`? Run something.
5. Confirm the duplicated stamp in `interpret.rs::run_in_module` (~L170) and whether any other site
   bumps `next_frame_generation`.
6. `vm/gc.rs` ~L83 lists `next_frame_generation: _` in what looks like an exhaustive destructure —
   confirm the counter is deliberately not a GC root and say why.
7. Is `frames.truncate(token.frame_index)` correct when the home frame is *not* the frame directly
   below — i.e. does it remove the home frame itself? Trace `_two_deep` and report the actual depths.
