# Recon — C1, the restricted loop

Phase 1 of [AUTHORING.md](../AUTHORING.md), for doc **C1** of
[CONCURRENCY-PLAN.md](../CONCURRENCY-PLAN.md). Grounded at HEAD, 2026-07-19.

Scope discipline: this answers the four recon questions and stops. It does not decide prose.

---

## 1. Architecture vs representation

**Architecture.** Cooperative, single-threaded, **restricted re-entrant loop** — ADR-0030 §4's
Option A, whose lineage the ADR names as Lua 5.1. There is exactly one dispatch loop
(`dispatch.rs::VM::run_until_inner`), and it is documented as *"the inner dispatch loop, **unaware
of fibers**"* (`dispatch.rs::VM::run_until_inner` doc comment, ~L470). Fibers are not scheduled by
it. Nothing in the instruction loop branches on which fiber is running.

**Representation — this is where the consequences live.** A running fiber's execution state is
**not in its `FiberObject`**. It is in four fields on the VM itself, and the running fiber's own
struct fields are *empty*:

```rust
// heap/fiber.rs::FiberObject ~L67-120  (doc comments abridged)
pub struct FiberObject {
    pub stack: Vec<Value>,                        // "empty while running — mirrored by VM::stack"
    pub frames: Vec<CallFrame>,                   // "empty while running — mirrored by VM::frames"
    pub open_upvalues: BTreeMap<usize, ObjRef>,   // "empty while running"
    pub checking: HashSet<ObjRef>,                // "empty while running"
    ...
}
```

A switch is `mem::take` in both directions over exactly those four fields:

```rust
// primitive/fiber.rs::store_live_into ~L29-43
let frames = std::mem::take(&mut vm.frames);
let stack = std::mem::take(&mut vm.stack);
let open_upvalues = std::mem::take(&mut vm.open_upvalues);
let checking = std::mem::take(&mut vm.checking);
```

So `vm.current: ObjRef` is a **bookkeeping handle, not an indirection**. The loop never dereferences
it to find its stack. There is no per-instruction "which fiber am I" cost, because there is nothing
to ask — there is one set of live buffers and whoever owns them *is* the running fiber.

**Both halves of the doc fall out of that one representational fact.** Because there is exactly one
set of live buffers, and because every nested re-entrant `run_until(base_frames)` is holding a plain
`usize` **index into them**:

```rust
// vm/send.rs::send_dynamic ~L223, ~L233-235
let base_frames = self.frames.len();
...
self.native_reentry_depth += 1;
let result = self.run_until(base_frames);
self.native_reentry_depth -= 1;
```

…a swap underneath that call turns `base_frames` into an index into a *different fiber's* vector.
The O(1) switch and the yield restriction are **the same fact seen from two sides**: swapping
buffers is cheap precisely because nobody holds a pointer into them, and it is illegal precisely
when somebody does.

---

## 2. The grip, grounded

> **A fiber switch is not a jump, and not a scheduler decision. It is `mem::take` on four VM
> fields — and the dispatch loop is never told it happened. That is why a switch costs O(1); it is
> also exactly why a switch is illegal whenever a native `run_until` is sitting on the Rust stack
> holding an index into the buffers about to be swapped.**

The plan's candidate grip ("a swap of which buffers the one loop is looking at") survives contact
with the source and sharpens: the loop is not *looking at* buffers belonging to a fiber, it **owns
the only ones there are**. The restriction is not an extra rule bolted on; it is the same sentence.

---

## 3. What was actually deliberated

Unlike every VM-track doc, **the design space here is real, not reconstructed.** ADR-0030
*Alternatives considered* records four rejections with bills attached:

| Branch | Recorded fate | Bill the ADR names |
|---|---|---|
| **A** — restricted re-entrant loop | **Taken** | forecloses the callback generator |
| **B** — full trampoline, yield anywhere | "Not now" — **additively reachable from A** | invasive rewrite of the whole primitive/callback protocol |
| **C** — stackful coroutines (real native stacks) | Rejected | `unsafe` stack-switch dependency, and **permanently constrains the GC** — every parked native stack becomes a root a moving collector must scan/relocate (named crown-jewel conflict *stackful-fiber ⊗ moving-GC*, weakening ADR-0009) |
| Preemptive / multithreaded | Rejected | needs a memory model + locks throughout the object model |
| Resumable (Smalltalk) suspension for failures | Out of scope | ADR-0008 propagation is terminating |

**The spine of the decision is the A→B vs A→C asymmetry**: A→B is purely additive, A→C is an
irreversible GC commitment. The doc's design-space walk must carry that asymmetry, and **must not
copy-paste the VM track's "this space is a pedagogical reconstruction" caveat — here it would be
false.**

---

## 4. Findings that change the doc

Four things recon settled that the plan did not know. All verified, not inferred.

### F1 — Plan §6's headline question: the typed signal **shipped**. C1 is a fork doc.

Plan §6 flagged this as "the single highest-value thing to verify" — whether ADR §5's typed switch
signal shipped or the `frames.len()` heuristic is still in place. **It shipped**, and the
`frames.len()` heuristic still exists *alongside* it, for the different job it was always doing
(non-local return). `vm/send.rs::VM::call_method`'s `Primitive` arm is a **three-way** branch, and
its own comment names the distinction:

```rust
// vm/send.rs::VM::call_method ~L54-67
if self.switch_pending {
    // ... The typed signal (not the `frames.len()` heuristic below) is what
    // distinguishes this from an ordinary return or a non-local return ...
    self.switch_pending = false;
} else if self.frames.len() >= frames_before {   // ordinary primitive return
} else {                                          // ReturnNonLocal fired inside native_fn
}
```

C1 is therefore a **fork** doc, as planned — not a landed-vs-planned doc.

### F2 — but the signal is a `bool` side-channel, not the `ControlFlow` value the ADR specified

ADR §5 specifies *"an explicit `ControlFlow`/switch value **out of the primitive**."* What shipped is
`VM::switch_pending: bool` (`vm/mod.rs` ~L79) — a field the primitive **sets** and `call_method`
**reads**, i.e. a side channel, not a return value. The *decision's substance* (typed, not a length
delta) shipped; the *named mechanism* did not.

**Honesty consequence:** the doc may not quote §5's "value out of the primitive" as though it
described the code. This is a §5.2 item.

### F3 — HEAD is **deliberately stricter than the ADR**, and says so

ADR §4 forecloses only **yielding** under a native frame. HEAD *also* forbids **resuming**
(`Fiber#call`/`try`) at any nonzero depth, and the code documents this as intentional:

```rust
// primitive/fiber.rs::cannot_resume_across_native_frame ~L86-96 (doc comment)
/// spec §6's restriction table only forecloses yielding underneath a native
/// frame, so this is a deliberately wider, sound over-restriction (a
/// nested `run_until`'s `base_frames` is computed against the currently
/// running fiber, which any switch underneath it — resume or yield alike —
/// would corrupt).
```

So the two guards use **different predicates**, which is itself the teachable shape:

| Site | Predicate |
|---|---|
| `fiber_resume` (`call`/`try`), ~L248 | `vm.native_reentry_depth != 0` — **absolute** |
| `fiber_yield`, ~L338 | `vm.native_reentry_depth != vm.heap.fiber(me).floor_depth` — **relative to this fiber's floor** |

### F4 — `floor_depth` is provably always `0` at HEAD (dead generality, not dead code)

Consequence of F3, and verified by reading every writer. `floor_depth`'s **only** non-zero-constant
writer is `primitive/fiber.rs::fiber_resume` ~L317:

```rust
vm.heap.fiber_mut(callee_ref).floor_depth = vm.native_reentry_depth;
```

…which is dominated, in the same function, by the early return at ~L248 that fires whenever
`native_reentry_depth != 0`. Every other initializer (`new_entry`, `new_entry_with_buffers`, `root`
in `heap/fiber.rs`) sets it to `0`. Therefore the value assigned is always `0`, and `fiber_yield`'s
relative check is **currently exactly equivalent** to `!= 0`.

The field is anticipatory generality: it is the shape the guard needs *if* F3's wider
resume-restriction is ever narrowed back to the ADR's §4 line. This is good doc material — a guard
written for a machine that cannot yet exercise it — but the doc must state it as *currently
equivalent to `!= 0`*, not as a live two-case distinction. **§5.3 claims-ledger item.**

### F5 — the ADR's own canonical "works" program does not compile at HEAD

Run live (`./target/release/phalcom`, release build at HEAD):

| Program | ADR says | Observed at HEAD |
|---|---|---|
| `Fiber.new { let n = 0; while (true) { Fiber.yield(n); n = n + 1 } }` | ✅ works | **compile error:** `Cannot reassign immutable `let` binding 'n'; declare it with `var` to allow mutation.` |
| same, with `var n = 0` | — | ✅ prints `0`, `1`, `2` |
| `Fiber.new { list.each { x => Fiber.yield(x) } }` | ✗ `CannotYieldAcrossNativeFrame` | ✅ as documented: `cannot switch fibers across a native call frame (e.g. inside .each { })` |
| ADR's prescribed rewrite (`while (i < list.size) { Fiber.yield(list.at(i)); … }`) | works | ✅ prints `10`, `20` |

The ADR predates the `let`/`var` split. **The doc must transcribe the example with `var`** and may
note the drift; it must not copy the ADR's snippet verbatim.

---

## 5. Brief-steering notes

### Agent A (theory) — emphasis, *without* the answer

- **Deep:** the re-entrant-interpreter problem itself — why a host-language call stack under the
  interpreter is what colors a runtime's suspend capability. This is the doc's real subject and A
  must supply the vocabulary.
- **Deep:** restricted-yield vs full trampoline (de-recursing every callback primitive) — *including
  the additivity question*: is one reachable from the other without rewriting the first?
- **Deep:** stackful coroutines, with the bill stated in **GC terms** (a parked native stack as a
  root a moving/compacting collector must scan and relocate). Recon flags this as the strongest
  bill in the space and A must not treat it as merely an `unsafe` complaint.
- **Medium:** Lua 5.1's `attempt to yield across a C-call boundary`, and 5.2/5.3's lift via
  `lua_yieldk`/continuations. This is the single highest-value comparison available: the same error,
  in the language the design descends from, *plus a shipped example of the exact escape route*.
- **One paragraph:** preemption / OS threads as the outer boundary of the space.
- **One sentence:** resumable (Smalltalk-style) suspension for failures.
- **Vocabulary A must import:** function coloring; re-entrancy; trampolining/de-recursion; stackful
  vs stackless; symmetric vs asymmetric coroutines; the C-call boundary.
- **Do not tell A which branch Phalcom took.** A is judged on the space, not the answer.

### Agent B (source map) — must-confirm list

1. **Headline:** how is a fiber switch signalled to the dispatch loop — a typed return value, a VM
   flag, a frame-length delta, or not signalled at all? Quote the line. *(Recon says: `bool` flag,
   F2 — B must confirm independently and census every read/write site.)*
2. Adversarially re-check **F4**: is `floor_depth` reachable with a nonzero value by *any* path?
   Search every writer, not just `fiber_resume`.
3. Full census of `native_reentry_depth` increment sites (`interpret.rs`, `send.rs` ×2,
   `block.rs`) — what each one is re-entering, one line each.
4. Quote `call_method`'s three-way `Primitive` arm in full; it is the doc's central artifact.
5. Confirm the `run_until` / `run_until_inner` split and the `base_frames != 0` fast-path early
   return, with its comment about which call sites can reach the floor capture.
6. **Confirm the inliner claim mechanically:** does `while` really lower to `Jump`/`Loop` inside one
   chunk with no frame push and no native re-entry, while `each` reaches `block_call`? Disassembly
   if reachable; otherwise mark INFERRED per plan §7.
7. Run the F5 program table again independently and report observed output.
8. The `concurrency_fiber_wren_*` fixture family — what Wren semantics were validated against, one
   line each. (Comparison-cast evidence.)
9. Scope check: `ready_queue` / `System.schedule` / the root-drive pump appear in `run_until`. Is
   any of it load-bearing for C1, or is it C4's? Report, do not expand.

---

## 6. Feeds forward

- C2 gets the four-field `mem::take` mechanism and `next_frame_generation`'s deliberate exclusion.
- C3 gets `capture_error_value`, the `Call`/`Try` cascade loop, and the parked-state clearing in the
  failure path — all read in passing here, none of it C1's subject.
