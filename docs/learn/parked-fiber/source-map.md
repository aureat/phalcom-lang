# Source map: how a parked fiber's state is stored and moved on a fiber switch

Scope: read-only investigation at HEAD (`0ce6a9c`, worktree `wt-c2`). No source
edited. All line numbers are as-read at this commit; symbols are the load-bearing
anchor, lines rot.

---

## THE QUESTION THAT DOMINATES EVERYTHING

**When fiber F is running and fiber G is parked, what does the VM physically hold
for F's value stack and call stack?**

**Answer: (b) — the buffers themselves, moved out of the parked fiber's
`FiberObject` by ownership transfer (`std::mem::take`), leaving the parked
object's fields at their empty default. This is a `Vec`/`BTreeMap`/`HashSet`
*swap*, not a pointer indirection, not a copy, and not a shared index into one
VM-wide stack.**

Ruling out the other candidates, each against a quoted line:

- **(a) pointer/handle indirection** — REFUTED. `VM::frames`/`VM::stack` are
  plain owned collections, not references into the heap:
  `pub(crate) frames: Vec<CallFrame>` and `pub(crate) stack: Vec<Value>`
  (`phalcom-core/src/vm/mod.rs` L53, L56). There is no `&mut` or handle field on
  `VM` pointing "into" a `FiberObject`; the running fiber's state is these two
  `Vec`s directly, full stop. Ordinary field access (`self.stack.push(...)`) is
  the fast path; there is no extra dereference through the current `FiberObject`
  on every stack operation.
- **(b) ownership transfer, moved out, source left empty** — CONFIRMED. See the
  full-text quotes of `store_live_into`/`load_live_from` below: both use
  `std::mem::take`, which (per `std::mem::take`'s own contract) replaces the
  source with `Default::default()` and returns the original value by move. No
  clone, no `Rc`, no shared buffer.
- **(c) a copy, with `FiberObject` holding a stale duplicate** — REFUTED.
  `mem::take` moves; it does not clone. After `store_live_into` runs,
  `vm.frames`/`vm.stack`/`vm.open_upvalues`/`vm.checking` are each the empty
  default (`Vec::new()`/`BTreeMap::new()`/`HashSet::new()`), and the
  `FiberObject` holds the *only* copy of the real data. There is exactly one
  live copy at all times, never two.
- **(d) index/base-offset into one shared VM-wide stack** — REFUTED as the
  general mechanism. Each fiber's stack is its own separately-owned `Vec`
  starting at index 0 (ADR-0030 §3: *"`CallFrame.stack_offset` stays
  **frame-relative**, so per-fiber stacks starting at 0 need no rebasing"* —
  confirmed in `heap/fiber.rs` L64-66: *"`stack_offset`s are window-relative
  (frame.rs, D3), so a per-fiber stack always based at index 0 needs no
  rebasing on switch."*). There is one exception worth flagging precisely, not
  glossing over: `FiberObject::resume_slot` (`heap/fiber.rs` L93-96) **is** an
  index/offset — the value-stack length to truncate the *resumer's* restored
  stack to before pushing the delivered value. But this is bookkeeping for
  where to *write* a single delivered value on restore, not the storage
  mechanism for the parked stack itself, which is still (b).
- **(e) something else / not implemented** — N/A; (b) is implemented and is the
  mechanism in production use today (confirmed by running programs below, Q5).

**Is the state ever in both places at once?** No. Every switch primitive calls
`store_live_into` (moving VM → FiberObject) and, on the resuming side,
`load_live_from` (moving FiberObject → VM) as paired, sequential, non-interleaved
steps — never a copy-then-keep-both. The line that proves it: `store_live_into`'s
`std::mem::take(&mut vm.frames)` (and the sibling three takes) each *replace* the
source field with its `Default`, in the same statement that captures the old
value into a local (`primitive/fiber.rs` L30-32, quoted in full below) — by the
time the next line runs, `vm.frames` is `Vec::new()`, and the only owner of the
real `Vec<CallFrame>` is the local `frames` binding about to be written into
`fiber.frames`. There is no window, even a single statement, where both `VM` and
the `FiberObject` hold live copies of the same buffer.

### `VM`'s fiber-relevant field declarations (`phalcom-core/src/vm/mod.rs`)

```rust
pub(crate) frames: Vec<CallFrame>,        // L53 — live mirror of `current`'s frames
pub(crate) stack: Vec<Value>,             // L56 — live mirror of `current`'s stack
pub(crate) current: ObjRef,               // L66 — the running Object::Fiber handle
pub(crate) switch_pending: bool,          // L79 — typed switch signal (D5)
pub(crate) native_reentry_depth: usize,   // L91 — restricted-yield-guard nesting depth
pub(crate) open_upvalues: BTreeMap<usize, ObjRef>, // L124 — live mirror of `current`'s open upvalues
pub(crate) ready_queue: VecDeque<ObjRef>, // L134 — scheduled-not-yet-started fibers
pub(crate) checking: std::collections::HashSet<ObjRef>, // L205 — live mirror of `current`'s @invariant guard set
pub(crate) next_frame_generation: u64,    // L109 — VM-global, never per-fiber (see §1 below)
#[cfg(feature = "fiber-pool")]
pub(crate) fiber_pool: Vec<(Vec<Value>, Vec<CallFrame>)>, // L233 — off by default
```

### `FiberObject`'s field declarations (`phalcom-core/src/heap/fiber.rs` L62-118)

Quoted in full under Q1 below (identical text; not repeated here to avoid
duplication per the "quote only load-bearing lines" rule — see §1).

### `primitive/fiber.rs::store_live_into`, in full (L22-43)

```rust
/// Moves `vm`'s live stacks (`frames`/`stack`/`open_upvalues`) into the
/// parked [`FiberObject`] behind `fiber_ref` (ADR-0030 §3).
///
/// Called on the fiber giving up the CPU, just before [`VM::current`] is
/// repointed at the fiber taking over. `mem::take` leaves the VM's live
/// mirror empty — the counterpart [`load_live_from`] fills it back in for
/// whichever fiber runs next.
pub(crate) fn store_live_into(vm: &mut VM, fiber_ref: ObjRef) {
    let frames = std::mem::take(&mut vm.frames);
    let stack = std::mem::take(&mut vm.stack);
    let open_upvalues = std::mem::take(&mut vm.open_upvalues);
    // `checking` (ADR-0052 Fix 1, U-ANNOT-CONTRACTS) swaps alongside the
    // three fields above for the same reason: an `@invariant`-guarded call
    // can `yield` mid-body, so this fiber's in-flight guard bookkeeping must
    // park with it rather than leak into whichever fiber runs next.
    let checking = std::mem::take(&mut vm.checking);
    let fiber = vm.heap.fiber_mut(fiber_ref);
    fiber.frames = frames;
    fiber.stack = stack;
    fiber.open_upvalues = open_upvalues;
    fiber.checking = checking;
}
```

### `primitive/fiber.rs::load_live_from`, in full (L45-59)

```rust
/// Moves the parked [`FiberObject`] behind `fiber_ref`'s stacks back into
/// `vm`'s live mirror (`frames`/`stack`/`open_upvalues`/`checking`) — the
/// reverse of [`store_live_into`], run on the fiber that is about to become
/// [`VM::current`].
pub(crate) fn load_live_from(vm: &mut VM, fiber_ref: ObjRef) {
    let fiber = vm.heap.fiber_mut(fiber_ref);
    let frames = std::mem::take(&mut fiber.frames);
    let stack = std::mem::take(&mut fiber.stack);
    let open_upvalues = std::mem::take(&mut fiber.open_upvalues);
    let checking = std::mem::take(&mut fiber.checking);
    vm.frames = frames;
    vm.stack = stack;
    vm.open_upvalues = open_upvalues;
    vm.checking = checking;
}
```

Note the comment in `store_live_into`'s doc block claiming it moves "three"
fields (`frames`/`stack`/`open_upvalues`) is stale relative to its own body,
which moves **four** (the trailing paragraph about `checking` was added later
without touching the summary line) — a small doc/code drift, not a behavior bug.

---

## 1. Type definitions and the swap/resident classification

### `heap/fiber.rs::FiberObject`, all fields with doc comments, in full (L62-118)

```rust
/// A cooperative, single-threaded fiber: its own value + call stacks, a
/// lifecycle [`FiberStatus`], a dynamic resumer link, a result slot, and its
/// entry closure ([ADR-0030] §2, `concurrency.md` §1).
///
/// A fiber owns its execution state so it can be parked and resumed by an O(1)
/// pointer swap of `vm.current` (ADR-0030 §3): while a fiber is
/// [`FiberStatus::Running`] its stacks live in the VM's live mirror
/// ([`VM::frames`]/`stack`/`open_upvalues`) and the fields here
/// are empty; while parked they hold the fiber's state. Keeping the stacks
/// **inside the arena object** (never in native Rust memory) is what lets a
/// future tracing GC reach a parked fiber's roots (ADR-0030 §7, D1).
pub struct FiberObject {
    /// The fiber's private operand stack (empty while running — mirrored by
    /// [`VM::stack`]). `stack_offset`s are window-relative
    /// (frame.rs, D3), so a per-fiber stack always based at index 0 needs no
    /// rebasing on switch.
    pub stack: Vec<Value>,                              // (i) SWAPPED
    /// The fiber's private call stack (empty while running — mirrored by
    /// [`VM::frames`]). Because the frame-generation counter
    /// stays VM-global (D4), a non-local `return` token whose home lives on
    /// another fiber fails the generation check → `DeadFrameError`.
    pub frames: Vec<CallFrame>,                         // (i) SWAPPED
    /// The fiber's private open-upvalue map, keyed by absolute value-stack
    /// index (empty while running — mirrored by
    /// [`VM::open_upvalues`]). Kept per-fiber because it is
    /// stack-index-keyed and each fiber has its own stack; swapping it with
    /// `stack`/`frames` prevents a cross-fiber slot-index collision.
    pub open_upvalues: BTreeMap<usize, ObjRef>,          // (i) SWAPPED
    /// The fiber's lifecycle state ([`FiberStatus`]).
    pub status: FiberStatus,                             // (ii) resident, r/w in place
    /// The fiber to hand control back to on `yield`/return/failure — a dynamic
    /// caller chain, not a fixed parent (`None` for the root fiber).
    pub resumer: Option<ObjRef>,                          // (ii) resident, r/w in place
    /// The last yielded/returned value, or the captured `Error` when
    /// [`FiberStatus::Failed`] (ADR-0030 §6).
    pub result: Value,                                    // (ii) resident, r/w in place
    /// The entry [`Object::Block`]/[`Object::Closure`] the fiber runs on first
    /// resume; `None` for the root fiber (which has no entry).
    pub entry: Option<ObjRef>,                            // (ii) resident, r/w in place (write-once at construction, read at first resume)
    /// Whether the entry frame has been pushed yet — `false` until the first
    /// `call`/`try`, then `true` for the fiber's life.
    pub started: bool,                                    // (ii) resident, r/w in place
    /// The value-stack length to truncate to (then push the delivered value)
    /// when this fiber is next resumed — recorded at the `yield` send whose
    /// window the resume value replaces (ADR-0030 §3).
    pub resume_slot: usize,                               // (ii) resident, r/w in place
    /// The `run_until` nesting depth captured when the fiber last began
    /// running — the fiber floor the restricted-yield guard compares against
    /// (ADR-0030 §4).
    pub floor_depth: usize,                               // (ii) resident, r/w in place
    /// How this fiber was last resumed ([`FiberResumeMode`]) — read at the
    /// fiber-floor capture when this fiber later finishes/fails.
    pub resume_mode: FiberResumeMode,                     // (ii) resident, r/w in place
    /// The identity set of receivers currently under `@invariant`
    /// re-entrancy-guard checking on this fiber (empty while running —
    /// mirrored by [`VM::checking`]) ... Kept per-fiber, swapped alongside
    /// `stack`/`frames`/`open_upvalues` on fiber switch, because a guarded
    /// call can `yield` mid-body.
    pub checking: HashSet<ObjRef>,                        // (i) SWAPPED
}
```

**Hypothesis check — verdict: CONFIRMED exactly as stated, field by field.**
Exactly four fields swap on every switch (`stack`, `frames`, `open_upvalues`,
`checking` — all four moved by `store_live_into`/`load_live_from`, quoted in
full above). The other eight (`status`, `resumer`, `result`, `entry`,
`started`, `resume_slot`, `floor_depth`, `resume_mode`) are read/written
directly on the `FiberObject` in place by `fiber_resume`/`fiber_yield`/
`vm/dispatch.rs`'s fiber-floor capture (e.g. `vm.heap.fiber_mut(callee_ref).status = FiberStatus::Running`,
`primitive/fiber.rs` L316) and never appear in either swap function. None is
"neither" (iii) — every field is classified.

### `VM`'s fiber-relevant fields

Already quoted above in the headline section (`vm/mod.rs` L53, L56, L66, L79,
L91, L109, L124, L134, L205, L233). All twelve fields there are classified in
`vm/gc.rs::collect_roots`'s exhaustive destructure (§3), which is independent
confirmation that this is the complete list the compiler is aware of.

### `next_frame_generation` is VM-global, never per-fiber

Confirmed: `pub(crate) next_frame_generation: u64,` (`vm/mod.rs` L109) sits
among the plain `VM` fields, not inside `FiberObject` (which has no such
field — absent from the full field list quoted above). `vm/gc.rs::collect_roots`'s
destructure explicitly classifies it a non-root, alongside the other flags:
`next_frame_generation: _,` (`vm/gc.rs` L83).

ADR-0030 §6's invariant, quoted:

> **Invariant:** the VM-global monotonic `next_frame_generation` counter
> **must not** be relocated into `FiberObject` — it is the only thing making a
> cross-fiber token globally non-matching.

(`docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md` §6,
L111-113.) The mechanism this protects: `ReturnNonLocal`'s `FrameToken` carries
a `frame_index` + `generation` (`frame.rs` L19-22); a token minted on fiber A
and later evaluated after control has switched to fiber B can only be detected
as stale (`DeadFrameError`) because the generation counter never resets or
partitions per fiber — a per-fiber counter would let a coincidentally-matching
`(frame_index, generation)` pair on a *different* fiber's frame be mistaken
for the same activation.

---

## 2. REFUTE-THIS: does the fiber-failure path clear `checking`?

**Claim under test:** *dispatch.rs's fiber-failure path clears only three of
the four parked fields and never `checking`, and because `checking`'s contents
are GC roots, this retains heap objects that would otherwise be collectable.*

**Verdict: PARTIAL.** The clearing omission is real and verified. The "GC
roots" framing and the practical-retention conclusion do not hold up under
adversarial checking — see below.

### The clearing omission — VERIFIED-TRUE

`vm/dispatch.rs::run_until`'s failure-cascade loop (L306-321):

```rust
loop {
    self.heap.fiber_mut(failed).status = crate::heap::FiberStatus::Failed;
    self.heap.fiber_mut(failed).result = error_value;
    // Spec §5.1: a `Failed` fiber can never resume, so its
    // parked state is pure retention — clear all three
    // parked fields here (not just `frames`). ...
    self.heap.fiber_mut(failed).frames.clear();
    self.heap.fiber_mut(failed).stack.clear();
    self.heap.fiber_mut(failed).open_upvalues.clear();
    let mode = self.heap.fiber(failed).resume_mode;
    ...
```

`checking` is never mentioned or cleared in this block. The comment even says
"all three" — an internally-accurate description of what the code does, but a
count that is one short of the swap quartet found in §1. Grepping every
writer of `FiberObject::checking`/`VM::checking` confirms there is no other
clearer:

- `primitive/object.rs::object_invariant_enter` (L336): `vm.checking.insert(*id)` — only ever touches the *live* `VM::checking` mirror, never a parked `FiberObject.checking` directly.
- `primitive/object.rs::object_invariant_exit` (L353): `vm.checking.remove(id)` — same, live mirror only.
- `primitive/fiber.rs::store_live_into`/`load_live_from` (quoted in full above) — the only two sites that ever touch `FiberObject::checking` directly, and both *move* it (swap), never clear it independent of a move.

So: no code path anywhere clears a parked `FiberObject.checking` except by
overwriting it wholesale on the next `load_live_from` for that same fiber (via
`mem::take`, which does clear it as a side effect of moving it out) — the
*failure* path specifically leaves it untouched, unlike its siblings.

### Is `checking` actually a "GC root"? — REFUTED as stated, retention mechanism is different

`heap/trace.rs::trace_object`'s `Object::Fiber` arm (L162-183) **does** trace a
parked fiber's `checking` set:

```rust
Object::Fiber(fiber) => {
    for value in &fiber.stack { trace_value(*value, push); }
    for frame in &fiber.frames { trace_frame(frame, push); }
    for cell in fiber.open_upvalues.values() { push(*cell); }
    if let Some(resumer) = fiber.resumer { push(resumer); }
    trace_value(fiber.result, push);
    if let Some(entry) = fiber.entry { push(entry); }
    // Receivers under `@invariant` re-entrancy checking (U-ANNOT-CONTRACTS).
    for receiver in &fiber.checking {
        push(*receiver);
    }
}
```

`vm/gc.rs::collect_roots` (L32-110, quoted in full in §3) does **not** put
`checking` (or any per-`FiberObject` field) in the VM-level root set for a
*parked* fiber — only `VM::checking` (the live mirror of `current`'s own set)
is a root, via the plain destructure-and-extend at L108: `out.extend(checking.iter().copied());`.
So a parked fiber's stale `checking` entries are not roots in the technical
sense used by this codebase (root = something `collect_roots` pushes
unconditionally); they are **outgoing edges of the `FiberObject`**, traced
only if and when the `FiberObject` itself is reached from a root. The claim's
"GC roots" wording overstates the mechanism, but the underlying retention
concern is directionally correct: as long as the `Failed` `FiberObject` stays
reachable (e.g. a variable still points at it), its stale `checking` entries
keep whatever receivers were mid-check alive past when `frames`/`stack`/
`open_upvalues` being cleared would otherwise have let them go.

### Is the retention actually reachable in a program? — investigated, and it is not currently constructible

Tracing the only mechanism that ever populates `checking`
(`object_invariant_enter`/`object_invariant_exit`, the `@invariant` weave)
against the two switch gates:

- `fiber_yield` refuses to park unless `vm.native_reentry_depth == fiber.floor_depth` (`primitive/fiber.rs` L338-340) — i.e. no *new* native re-entrant frame may have opened since this fiber was last resumed.
- `fiber_resume` refuses to switch at all — for **either** `call` or `try` — unless `vm.native_reentry_depth == 0` (`primitive/fiber.rs` L248-250), unconditionally, not relative to any floor.

The negative-lane fixture `phalcom-core/tests/lang/runtime-errors/contracts_invariant_fiber_yield.ph`
documents (status: NEGATIVE, i.e. a currently-broken/blocked shape, not a
passing golden) that **`Fiber.yield` inside an `@invariant`-guarded method body
currently hard-errors**, because `__invariantEnter`/`__invariantExit` wrap the
guarded body in a native re-entrant call frame — "same restriction as
`.each { }`" per its own comment (L4-10). That means the entire window during
which `checking` can be non-empty is *also* a window with
`native_reentry_depth` raised above any fiber's recorded floor — which
independently forecloses **both** switch primitives (`yield` via the
floor-delta check, and `call`/`try` via the unconditional `!= 0` gate) for as
long as `checking` holds anything.

**Conclusion: the missing `.checking.clear()` in the failure cascade is a
real code gap (confirmed by reading), but under HEAD's current invariant-guard
implementation there is no known path that reaches a fiber switch
(`store_live_into`) while `VM::checking` is non-empty — the guard mechanism
that populates `checking` always runs inside a native re-entrant frame that
blocks every switch primitive simultaneously.** This is inferred from static
reading of `object.rs`/`fiber.rs`/the negative fixture, not from an exhaustive
proof that no other path can populate `checking`; I did not find one, but I
did not attempt to fuzz the invariant-weave compiler for an alternate shape
either — flagged as a residual unknown, not asserted closed.

---

## 3. REFUTE-THIS ADVERSARIALLY: is reachability of parked fibers purely transitive via resumer + no registry?

**Claim under test:** *at HEAD there is no registry of live fibers;
`collect_roots` pushes only `*current` and `ready_queue`, so parked fibers are
reached purely transitively — via a callee's `resumer` back-link, and via the
resumer's own parked stack holding the callee's `Value::Obj` handle.*

**Verdict: VERIFIED for "no registry exists" and for the resumer-chain
mechanism as *one* reachability path; REFINED (not refuted, but incomplete as
stated) — it is not the *only* transitive path, since ordinary `Value`
reachability through modules/containers/upvalues also keeps any fiber a
program still references alive, independent of any resumer link.**

### Every root the collector pushes — full enumeration

`vm/gc.rs::collect_roots`, in full (L18-110):

```rust
pub fn collect_roots(&self, out: &mut Vec<ObjRef>) {
    let VM {
        heap: _,
        frames, stack, current, open_upvalues,
        ready_queue,
        modules, main_module, last_imported_module,
        classes,
        universe,
        sealed_classes,
        checking,
        interner: _, field_layouts: _, constructor_aliases: _,
        has_new_construct: _, class_parents: _,
        init_selector_cache: _, variadic_selector_cache: _,
        switch_pending: _, native_reentry_depth: _,
        next_frame_generation: _, world_version: _, start_time: _,
        compile_mode: _, strip_contract_metadata: _,
        #[cfg(feature = "fiber-pool")] fiber_pool: _,
    } = self;

    for frame in frames { trace_frame(frame, &mut |id| out.push(id)); }
    for value in stack { if let Some(id) = value.as_obj() { out.push(id); } }
    out.push(*current);
    out.extend(open_upvalues.values().copied());
    out.extend(ready_queue.iter().copied());
    out.extend(modules.values().copied());
    out.extend(main_module.iter().copied());
    out.extend(last_imported_module.iter().copied());
    out.extend(classes.values().copied());
    out.extend(sealed_classes.values().copied());
    out.extend(checking.iter().copied());
    universe.each_handle(&mut |id| out.push(id));
}
```

So the actual root set is: `current`'s frames (traced) + `current`'s stack
(handles only) + `current` itself + `current`'s open upvalues + `ready_queue`
(not-yet-started scheduled fibers) + module table + main/last-imported module
+ named classes + `sealed_classes` + `current`'s `checking` set + the pinned
kernel/import registry (`universe`). **No dedicated fiber registry exists** —
confirmed, matching the claim's premise exactly (this file's own header
comment even calls out that the original hand-audited table *missed*
`sealed_classes`/`checking`/`ready_queue`, gc.rs L26-31).

### Constructing a counter-example — attempted and failed; empirically tested

Reasoning for why a *resumable, non-`current`* fiber that a program still
references cannot become unreachable:

1. If it is `current`, it's a direct root (`out.push(*current)`).
2. If it is enqueued but not yet started, it's in `ready_queue`, a direct root.
3. If it is a live *ancestor* of `current` (resumed `current` at some point in
   the call chain, directly or transitively) and holds no other reference
   anywhere, it is reached because `trace_object`'s `Fiber` arm traces
   `fiber.resumer` (`heap/trace.rs` L172-174: `if let Some(resumer) = fiber.resumer { push(resumer); }`)
   — tracing the currently-reachable descendant (rooted directly or reached
   the same way) walks the whole ancestor chain back to the root fiber, one
   `resumer` hop at a time, regardless of whether any of those ancestors is
   otherwise referenced by a variable.
4. If it is reachable any other way a normal Phalcom value is reachable — a
   module-level global, a class static slot, a list/map/tuple element, an
   `Object::BoundMethod` receiver, an `Upvalue::Open { fiber, .. }`'s `fiber`
   handle, or sitting in a local variable on any *other* stack that is itself
   reachable — ordinary `trace_object`/`trace_value` recursion covers it: the
   match is exhaustive over `Object` variants (`trace.rs` L10-11, "no
   wildcard"), and every variant with a `Value`-typed field routes through
   `trace_value`/`Value::as_obj`.

I could not construct a shape where a resumable, reachable, non-`current`
fiber's stack is missed — every candidate collapses into one of the four cases
above. I tested the specific case the claim spotlights (a parked *ancestor*
kept alive purely by the resumer back-link, with **no other variable
referencing it**) empirically rather than only reasoning about it:

`nested_fiber_gc_reachability.ph` (scratch, run against this worktree's build):
root builds a freshly-concatenated (non-constant-pool) string `secret`, calls
`f1`, which builds its own local `f1local` and calls `f2`; `f2` calls
`System.gc` (forces a full mark-sweep, `primitive/system.rs::system_gc` →
`VM::force_gc`) while root and `f1` are both parked and reachable **only**
through `f2.resumer → f1`, `f1.resumer → root` — neither `root` nor `f1` is
ever itself the value of any Phalcom variable. Observed output (verbatim):

```
main: secret built = root-secret-value-42
f2: forced gc while root+f1 parked (unreachable except via resumer chain)
f1: f2 returned f2-done
f1: f1local after gc = f1-local-value-7
main: f1 returned f1-done
main: secret after nested fiber + gc = root-secret-value-42
```

Both `secret` and `f1local` survive the forced collection unchanged — the
resumer-chain mechanism holds for this shape. **VERIFIED** for this specific
program; **INFERRED** (not exhaustively proven for every object shape — e.g. a
fiber handle stored only inside a `Set`/`Range`, or captured only through a
doubly-nested closure upvalue chain, were not separately tested) as the
general invariant. The named invariant: *every `FiberObject` on the resumer
chain rooted at `current` is reached by repeated application of
`trace_object`'s `Fiber` arm's `resumer` edge, and `resumer` is set on every
resume and never cleared to `None`* (`primitive/fiber.rs` L295: `vm.heap.fiber_mut(callee_ref).resumer = Some(resumer_ref);`,
run every `fiber_resume`, never reset elsewhere).

---

## 4. Regression golden: `fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph`

Exists at `phalcom-core/tests/lang/concurrency/fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph`,
paired `.expected`. Fixture, in full:

```phalcom
// area: concurrency
// spec: concurrency.md; ADR-0030 §3/§4
// status: PASS
// Regression: `fiber_resume` (fiber.rs) used to steal the calling fiber's
// live stacks (`store_live_into`) *before* validating a not-yet-started
// callee's entry arity, and (independently) the arity error always named
// the signature "call" even when raised from `try()`. Neither defect was
// externally observable in this exact shape — `outer` is unconditionally
// marked `Failed` by the same fiber-floor cascade either way, discarding
// its state before anything reads it — but the ordering is still the wrong
// invariant (validate before mutating shared VM/heap state, not after) and
// the message text was a genuine, user-visible bug. Locks both: `root`
// (which resumed `outer` via `try()`) recovers the captured `Error` and
// keeps running, and the message correctly says "try" as the signature.

let inner = Fiber.new { x => x }
let outer = Fiber.new {
  inner.call()
  System.print("unreachable: outer body continues past inner.call")
}
let r = outer.try()
System.print(r.class.name)
System.print(r.message)
System.print("root continues")
```

`.expected`, in full:
```
Error
Method call expected 1 argument, got 0
root continues
```

**Ran it** (`cargo run -q -p phalcom-core --bin phalcom -- <path>` inside the
worktree, `cargo build -p phalcom-core --bin phalcom` succeeded with one
pre-existing unrelated `dead_code` warning on `init_selector_cache`).
**Observed stdout, verbatim:**

```
Error
Method call expected 1 argument, got 0
root continues
```

Matches `.expected` exactly. Note the fixture's own comment is explicit that
**neither original defect was externally observable in this exact shape** —
this is a "lock the correct ordering/invariant" regression test, not a
demonstration of a visible behavior difference; the doc being written should
not claim it shows an observable before/after contrast.

### Other early-return paths after a `store_live_into` and before the paired restore

Examined both call sites:

- **`fiber_resume`** (`primitive/fiber.rs` L247-321): `store_live_into(vm, resumer_ref)` runs at L293. Every line between there and the function's end (L293-320) is infallible field writes / `Vec` pushes / a `HashMap`-free match — there is no `?`-propagated fallible call and no early `return` anywhere in that span. The function either (a) pushes a fresh entry frame (first resume, no `load_live_from` — see Q5) or (b) calls `load_live_from` (already-started) — one of the two always runs before the function returns `Ok(Value::Nil)`. **No other early-return gap exists in `fiber_resume` today.**
- **`fiber_yield`** (`primitive/fiber.rs` L333-352): `store_live_into(vm, me)` runs at L347. The very next statement (L349) is `vm.switch_to_fiber_and_deliver(resumer, value)`, which unconditionally calls `load_live_from` internally (`vm/dispatch.rs` L352-359, quoted in Q5) — again no fallible call and no early return in between. **No gap here either.**

The one bug this class of hazard did produce (the arity-mismatch fix) was
pre-empted by moving the validation *before* `store_live_into` rather than by
guarding the post-store window — consistent with `fiber_resume`'s own comment
(L262-268: *"Doing this after `store_live_into` was a real bug"*), which
already describes the fix as reordering, not adding a recovery path.

---

## 5. The two-resume-path asymmetry

Both branches, quoted from `fiber_resume` (`primitive/fiber.rs`):

**First-resume branch (L298-307)** — pushes a fresh entry frame, never calls `load_live_from`:
```rust
if let Some((entry, closure_id, home_frame_token)) = entry_call {
    // `vm.stack`/`vm.frames` are empty here (just taken by
    // `store_live_into` above), so the callee's fresh window starts at 0.
    let stack_offset = vm.stack.len();
    vm.stack.push(Value::Obj(entry));
    vm.stack.extend_from_slice(args);
    let mut frame = vm.new_call_frame(closure_id, CallContext::Instance { instance: entry }, 0, stack_offset, None);
    frame.home_frame_token = home_frame_token;
    vm.frames.push(frame);
    vm.heap.fiber_mut(callee_ref).started = true;
}
```

**Already-started branch (L308-314)** — calls `load_live_from`, delivers into `resume_slot`:
```rust
else {
    load_live_from(vm, callee_ref);
    let delivered = args.first().copied().unwrap_or_else(|| vm.none_value());
    let slot = vm.heap.fiber(callee_ref).resume_slot;
    vm.stack.truncate(slot);
    vm.stack.push(delivered);
}
```

### Observable trace: `two_resume_paths.ph` (scratch, run in this worktree)

```phalcom
let f = Fiber.new { first_arg =>
  System.print("fiber: started, first_arg = " + first_arg.toString)
  let got = Fiber.yield("from-yield")
  System.print("fiber: resumed after yield, got = " + got.toString)
  "fiber-done"
}

System.print("main: about to first-resume (fresh entry frame path)")
let y = f.call("hello")
System.print("main: first .call() returned " + y.toString)

System.print("main: about to second-resume (load_live_from + resume_slot path)")
let d = f.call("world")
System.print("main: second .call() returned " + d.toString)
```

**Observed stdout, verbatim:**
```
main: about to first-resume (fresh entry frame path)
fiber: started, first_arg = hello
main: first .call() returned from-yield
main: about to second-resume (load_live_from + resume_slot path)
fiber: resumed after yield, got = world
main: second .call() returned fiber-done
```

This shows the asymmetry directly: the first `.call("hello")` delivers its
argument as the entry closure's *parameter* (`first_arg`), via a brand-new
`CallFrame` pushed at `stack_offset = 0`; the second `.call("world")` delivers
its argument as the *return value of `Fiber.yield(...)`* inside the
already-running frame, via `load_live_from` + `resume_slot` truncate-and-push —
two structurally different delivery mechanisms for what is syntactically the
same `f.call(x)` call.

### Existing fixture: `concurrency_fiber_yield_resume.ph`

```phalcom
// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS
// C-FIB-1: a Fiber yields successive counter values across resumes.

let counter = Fiber.new {
  var n = 0
  while (true) {
    Fiber.yield(n)
    n = n + 1
  }
}
System.print(counter.call())
System.print(counter.call())
System.print(counter.call())
```

`.expected`: `0`, `1`, `2`. **Ran it — observed stdout verbatim: `0`, `1`, `2`.**
Matches exactly.

---

## 6. Fiber-stack pooling — is it on?

`Cargo.toml` (`phalcom-core/Cargo.toml` L14-15):
```
[features]
default = []
```

**Not in `default`.** `fiber-pool` is an opt-in-only Cargo feature; the plain
`cargo build`/`cargo run` used throughout this investigation never enabled it,
and `FiberObject::new_entry_with_buffers`, `VM::fiber_pool`, and the recycle
block in `vm/dispatch.rs` (L277-286, `#[cfg(feature = "fiber-pool")]`) are
compiled out entirely by default — confirmed by their `#[cfg(feature = "fiber-pool")]`
gates (`heap/fiber.rs` L148, `vm/mod.rs` L232, `vm/dispatch.rs` L277).

### Actual measured numbers (`docs/forge/perf-log/findings.md`, **F10**)

> Same-machine A/B, release, `nopool` vs `--features fiber-pool`, 3 reps:
>
> | fibers | user (nopool → pool) | peak RSS (nopool → pool) | ΔRSS |
> |---|---|---|---|
> | 100k | 0.06 → 0.06 s | 52.7 MB → 98.0 MB | **+86%** |
> | 500k | 0.31 → 0.31 s | 309 MB → 539 MB | **+74%** |
> | 1M | 0.62 → **0.85 s (+37%)** | 635 MB → 1090 MB | **+72%** |
>
> Skynet, by contrast, is a wash on every axis (user 2.19 vs 2.20–2.31 s, RSS
> 1.437 vs 1.437 GB) — which is why F5, measuring Skynet, saw nothing.
>
> **The RSS cost is linear in fibers created: ~450 B per fiber, dead on.**

Ruling (`findings.md` F10 Consequences): *"Ruled: the flag stays, stays off,
and is not to be used (owner's call, 2026-07-14)."* This is corroborated in
`vm/mod.rs`'s own doc comment on `fiber_pool` (L228-231): *"Measured net
negative in whole-process A/B benchmarking (perf-log, 2026-07-14); gated off
by default, kept for future re-measurement."*

A related but **distinct** experiment — presizing the (non-pooled) fiber
buffers' initial capacity, not recycling them through a pool — is recorded
separately in `docs/forge/perf-log/negative-presize-fiber-vecs.md`, also
measured negative and reverted:

> | workload | base `user` | presized | Δ `user` | base RSS | presized | Δ RSS |
> |---|---|---|---|---|---|---|
> | `skynet` | 1.650 s | 1.690 s | **+2.4%** | 1306 MB | 1233 MB | −5.6% |
> | `fiber_churn` | 0.200 s | 0.240 s | **+20.0%** | 263 MB | **581 MB** | **+121.3%** |
> | `fibers` | 0.080 s | 0.090 s | **+12.5%** | 114 MB | 141 MB | **+23.2%** |

Both records report a **measured number**, not an unmeasured estimate; neither
is stated here as a general perf claim beyond what each table says.

---

## Bounded ADR read: ADR-0030 §2, §3, §6, §7, Alternatives considered

(`docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md`)

- **§2 — `Fiber` is a heap object, not a new `Value` arm.** A `FiberObject` is
  one more `Object::Fiber` arena variant, reached through `Value::Obj(ObjRef)`
  exactly like `List` — no `Value::Fiber` arm. Owns `stack: Vec<Value>`,
  `frames: Vec<CallFrame>`, `status`, `resumer`, a result slot, and its entry
  closure. Explicitly supersedes an older `concurrency.md` §1 phrasing
  (`Value::Fiber(PhRef<FiberObject>)`) that predates the handle-arena heap.
- **§3 — Fiber switch is an O(1) pointer swap.** "Current stack/current
  frames" relocate behind a `current: ObjRef`; `call`/`yield` swap *which
  fiber the dispatch loop reads*, never copying stacks.
  `CallFrame.stack_offset` stays frame-relative so per-fiber stacks starting
  at 0 need no rebasing — this is the ADR-level statement that HEAD's
  `mem::take`-based `store_live_into`/`load_live_from` implements.
- **§6 — Non-local return and unwind stay fiber-local.** Once `self.frames`
  is the current fiber's vector, `ReturnNonLocal` only searches that fiber; a
  token whose home is on another fiber fails the generation check →
  `DeadFrameError`. Names the `next_frame_generation`-stays-VM-global
  invariant quoted in full in §1 above. Error unwind likewise operates on
  `self.frames` only and stops at the fiber floor, so a failing fiber
  captures its `Error` into its result slot instead of terminating the host —
  this is the mechanism `vm/dispatch.rs::run_until`'s `Err(e)` branch (§2/§4
  above) implements.
- **§7 — Fibers are GC roots even when parked.** *"Invariant (before any
  tracing/compacting GC lands): a `FiberObject`'s value stack and frame stack
  are GC roots for as long as the fiber is reachable and not `done`/`failed`
  — not only the `current` fiber's."* At HEAD (a real mark-sweep now ships,
  ADR-0050), this invariant is realized **not** by literally enumerating every
  live fiber as a VM-level root, but by the combination verified in §3 above:
  `current` is rooted directly, the resumer chain is traced as an edge of
  each reached `FiberObject`, `ready_queue` is rooted explicitly, and every
  other path to a fiber goes through ordinary `Value`/container tracing.
- **Alternatives considered.** **B — full trampoline** (de-recurse every
  callback primitive so `yield` can cross native frames anywhere): rejected
  for now as a large, invasive rewrite, reachable additively from the shipped
  Option A later. **C — stackful coroutines** (a real native stack per
  fiber): rejected because it would permanently constrain the GC — every
  parked fiber's *native* stack would become a root a future moving collector
  must scan/relocate, which is exactly the crown-jewel conflict Option
  A/B avoid by keeping all fiber state inside the arena (§2). **Preemptive/
  multithreaded fibers:** rejected — would need a memory model and locks
  throughout the object model. **Resumable (Smalltalk) suspension for
  failures:** out of scope; error propagation stays terminating per
  ADR-0008.

---

## Scratch files used (not committed, live in scratchpad only)

- `/private/tmp/claude-501/-Users-altunhasanli-dev-phalcom-phalcom/6f32bf55-3202-48c3-87b3-5a5ca5045cd6/scratchpad/two_resume_paths.ph`
- `/private/tmp/claude-501/-Users-altunhasanli-dev-phalcom-phalcom/6f32bf55-3202-48c3-87b3-5a5ca5045cd6/scratchpad/nested_fiber_gc_reachability.ph`

## Unexplored lead (flagged, not chased — would be an eighth area)

An empirical (not just static) demonstration of §2's retained-`checking`
scenario would require deliberately bypassing/loosening the
`native_reentry_depth` gate (or finding some other, currently-unknown path
that populates `VM::checking` without also raising `native_reentry_depth`
above the acting fiber's floor) to actually get a fiber switch to occur while
`checking` is non-empty, then inspecting heap occupancy before/after a forced
`System.gc` on the resulting `Failed` fiber. That is a deeper probe than this
map's budget covers and is left as a lead, not a finding.
