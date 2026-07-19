# Fiber switching and the restricted-yield guard — source map at HEAD

Read-only investigation. Claims are VERIFIED (source read directly, or a real
`.ph` program compiled/run with `./target/release/phalcom` and its output
inspected) unless marked **INFERRED**. Grounded via `graphify query` /
`explain` / `affected` first, then targeted reads — no tree sweep. Commit
context: `main` at `f7f481b` at investigation time (working tree has
unrelated pending doc changes; nothing under `phalcom-core/src` or
`phalcom-core/tests` is dirty).

Entry symbols (via the mandated `graphify` calls): `FiberObject` @
`phalcom-core/src/heap/fiber.rs:62`, `.switch_to_fiber_and_deliver()` @
`phalcom-core/src/vm/dispatch.rs:352`, `.run_until()` @ `:221`,
`.run_until_inner()` @ `:477`. `graphify explain "native_reentry_depth"` and
`graphify explain "0030-fibers-and-futures-cooperative-concurrency"` returned
no node (the field and the ADR are not modeled as first-class graph nodes at
this snapshot); those two were located by `grep` per the fallback rule
("only grep after graphify has oriented you").

---

## THE QUESTION THAT DOMINATES EVERYTHING

**How is a fiber switch communicated to the dispatch loop?**

**(b) — a boolean flag field on the VM, checked once, by name, in exactly one
place.** Not (a) a typed return value, not (c) a `frames.len()` delta, not
(d) a re-read of `self.current`, not (e) unsignalled.

The field: `phalcom-core/src/vm/mod.rs::VM::switch_pending` @ L79:

```rust
pub(crate) switch_pending: bool,
```

Set by every switching primitive just before it returns
(`primitive/fiber.rs::fiber_resume` @ L319, `fiber_yield` @ L350: both end
`vm.switch_pending = true;`). Read exactly once, in
`vm/send.rs::VM::call_method`'s `Primitive` arm @ L54:

```rust
if self.switch_pending {
    self.switch_pending = false;
}
```

**Two mechanisms coexist in that same post-call reconciliation, for two
different things** — see §3 for the full three-way arm:

1. **The typed flag (`switch_pending`)** — detects a *fiber switch*.
2. **The `frames.len()` delta** (`self.frames.len() >= frames_before`) —
   detects a *non-local return* that unwound frames inside the primitive
   (pre-dates fibers entirely; U10/ADR-0013).

**This is a confirmed divergence from ADR-0030 §4/§5.** The ADR's own text:

> "the fiber-switch signal is typed, not a length delta... `call`/`yield`
> reconcile with the dispatch loop through an explicit `ControlFlow`/switch
> value **out of the primitive**" (§5)

HEAD does not do that. There is no `ControlFlow`-shaped return type anywhere
in `primitive/fiber.rs` — every switching primitive returns a plain
`PhResult<Value>` (`Ok(Value::Nil)`), identical in shape to any other
primitive. The "typed signal" the ADR calls for is instead **(b)**, a `bool`
field on `VM` that a primitive mutates as a side effect. This is not an
oversight: the U-FIBER implementation spec records it as a deliberate,
named deviation —
`docs/forge/units/U-FIBER/implementation-spec.md:332`, **D-FIB-5**:

> "typed return vs. VM flag. *Recommended:* `native_fn -> ...` [a typed
> return]... (The typed return is cleaner and is recommended.)"

— and the shipped code's own doc comment concedes the trade explicitly,
`vm/mod.rs:76-78`:

> "See D-FIB-5: this flag is the 'VM flag' alternative to threading a typed
> return through every `PrimitiveFn` (which would touch all ~70 existing
> primitives) — explicitly sanctioned by the implementation spec as the
> pragmatic choice."

So: ADR-0030 §5 specifies (a); HEAD implements (b); the implementation spec
that bridged ADR → code recorded (a) as "recommended" and shipped (b) anyway
as "pragmatic." Do not read the ADR's §5 prose as describing HEAD.

---

## 1. `FiberObject`, `FiberStatus`, `FiberResumeMode`

`phalcom-core/src/heap/fiber.rs::FiberObject` @ L62, quoted in full:

```rust
pub struct FiberObject {
    pub stack: Vec<Value>,
    pub frames: Vec<CallFrame>,
    pub open_upvalues: BTreeMap<usize, ObjRef>,
    pub status: FiberStatus,
    pub resumer: Option<ObjRef>,
    pub result: Value,
    pub entry: Option<ObjRef>,
    pub started: bool,
    pub resume_slot: usize,
    pub floor_depth: usize,
    pub resume_mode: FiberResumeMode,
    pub checking: HashSet<ObjRef>,
}
```

**Live-while-running vs parked-while-running** (per the doc comments on each
field, L63-117): exactly four fields are the "mirrored" set —
`stack`, `frames`, `open_upvalues`, `checking` — **empty while the fiber is
`Running`** (their real content lives in `VM::stack`/`frames`/
`open_upvalues`/`checking`), and populated only while the fiber is parked
(`Suspended`/`Done`/`Failed`). The other eight fields (`status`, `resumer`,
`result`, `entry`, `started`, `resume_slot`, `floor_depth`, `resume_mode`)
are **not mirrored anywhere** — they live in the `FiberObject` at all times,
running or parked.

`FiberStatus` @ L12, quoted in full:

```rust
pub enum FiberStatus {
    Suspended,
    Running,
    Done,
    Failed,
}
```

`FiberResumeMode` @ L37, quoted in full:

```rust
pub enum FiberResumeMode {
    Call,
    Try,
}
```

Recorded on the **callee** at resume time (`fiber_resume` sets
`vm.heap.fiber_mut(callee_ref).resume_mode = mode;`, `primitive/fiber.rs:296`)
so the fiber-floor capture in `run_until` (§4/§6 of the ADR) knows, once the
callee eventually finishes or fails — possibly many switches later — whether
to re-raise into the resumer (`Call`) or hand it the `Error` as a value
(`Try`).

---

## 2. The switch mechanism — `store_live_into` / `load_live_from` / `switch_to_fiber_and_deliver`

All three in `phalcom-core/src/primitive/fiber.rs` (first two) and
`phalcom-core/src/vm/dispatch.rs::VM::switch_to_fiber_and_deliver` @ L352
(third).

`store_live_into` @ `fiber.rs:29`, quoted in full:

```rust
pub(crate) fn store_live_into(vm: &mut VM, fiber_ref: ObjRef) {
    let frames = std::mem::take(&mut vm.frames);
    let stack = std::mem::take(&mut vm.stack);
    let open_upvalues = std::mem::take(&mut vm.open_upvalues);
    let checking = std::mem::take(&mut vm.checking);
    let fiber = vm.heap.fiber_mut(fiber_ref);
    fiber.frames = frames;
    fiber.stack = stack;
    fiber.open_upvalues = open_upvalues;
    fiber.checking = checking;
}
```

`load_live_from` @ `fiber.rs:49` is the exact mirror (four `mem::take`s the
other direction, `fiber.* -> vm.*`).

**Exactly four `VM` fields move, always the same four:** `frames`, `stack`,
`open_upvalues`, `checking`. Nothing else — `current`, `native_reentry_depth`,
`next_frame_generation`, `world_version`, `classes`, `modules`, `interner`,
`universe` are untouched by either function; they are VM-global and shared
across every fiber by design (ADR-0030 §6's invariant on
`next_frame_generation` specifically).

`switch_to_fiber_and_deliver` @ `dispatch.rs:352`, quoted in full:

```rust
pub(crate) fn switch_to_fiber_and_deliver(&mut self, target: ObjRef, value: Value) {
    self.current = target;
    crate::primitive::fiber::load_live_from(self, target);
    let slot = self.heap.fiber(target).resume_slot;
    self.stack.truncate(slot);
    self.stack.push(value);
    self.heap.fiber_mut(target).status = crate::heap::FiberStatus::Running;
}
```

This is the shared "land a value at a fiber's resume point" step, reused by
an ordinary `yield`→resumer handoff and by both branches of the fiber-floor
capture (success delivering the return value, `try`-mode failure delivering
the captured `Error`) — `fiber_yield` calls it directly (`fiber.rs:349`)
rather than hand-inlining the same four steps.

Note what `switch_to_fiber_and_deliver` does **not** do: it does not call
`store_live_into` for the *outgoing* fiber. Every call site parks the
outgoing fiber itself, first (`fiber_yield` calls `store_live_into(vm, me)`
at `fiber.rs:347` before calling this; `run_until`'s fiber-floor-capture call
sites park nothing further because the finishing fiber's live state is
already the VM's own empty-after-drain state). `fiber_resume` is the one
exception worth naming: it parks the *resumer* (`store_live_into(vm,
resumer_ref)`, `fiber.rs:293`) and then does the callee's own load/deliver
**inline** (`fiber.rs:298-316`) rather than through this helper, because a
first-time resume has no parked callee state to load — it pushes a fresh
entry frame instead.

---

## 3. `VM::call_method`'s `Primitive` arm — in full

`phalcom-core/src/vm/send.rs::VM::call_method` @ L19, the `Primitive` arm
(L22-94), quoted in full:

```rust
MethodKind::Primitive(native_fn) => {
    let receiver_idx = self.stack.len() - 1 - arity;
    let receiver = self.stack[receiver_idx];
    let frames_before = self.frames.len();
    self.switch_pending = false;
    const INLINE_ARGS: usize = 8;
    let result = if arity <= INLINE_ARGS {
        let mut args = [Value::Nil; INLINE_ARGS];
        args[..arity].copy_from_slice(&self.stack[receiver_idx + 1..]);
        native_fn(self, &receiver, &args[..arity])
    } else {
        let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
        native_fn(self, &receiver, &args)
    };
    result.map(|result| {
        if self.switch_pending {
            self.switch_pending = false;
        } else if self.frames.len() >= frames_before {
            self.stack.truncate(receiver_idx);
            self.stack.push(result);
        } else {
            self.stack.push(result);
        }
    })
}
```

Three arms, all keyed on state the native call itself may have mutated:

1. **`self.switch_pending` true** — a fiber switch fired inside `native_fn`
   (`fiber_call`/`fiber_try`/`fiber_yield`). `self.frames`/`self.stack` now
   belong to a **different** fiber than the one `receiver_idx` was computed
   against; touching `result`/the stack here would corrupt the new fiber's
   window. The arm does nothing but clear the flag — the switching primitive
   already left the new current fiber's stack exactly where the dispatch
   loop expects it.
2. **`self.frames.len() >= frames_before`** — the ordinary case: `native_fn`
   pushed no frames (or pushed and popped them all, net zero), so
   `receiver_idx` still means what it meant before the call. Collapse the
   receiver+args window and land `result` in its place.
3. **`self.frames.len() < frames_before`** — neither of the above: a
   `Bytecode::ReturnNonLocal` fired *inside* `native_fn` (e.g. `block_call`
   ran a block whose `return` unwound past this call site), popping frames
   out from under this call. `receiver_idx` now points above the unwound
   stack top, so the ordinary truncate would be a silent no-op; the arm
   re-pushes `result` instead, to be picked up by whichever `run_until`'s
   drain check finds its floor here.

`self.switch_pending = false;` immediately **before** the call (L30) is load-
bearing too, not just decoration: it clears any stale `true` a *previous*
primitive call left set, so this call's own three-way branch cannot be
misled by a switch that happened earlier in the same expression.

---

## 4. `run_until` vs `run_until_inner`

`phalcom-core/src/vm/dispatch.rs::VM::run_until` @ L221. The `base_frames !=
0` fast path, quoted:

```rust
pub(crate) fn run_until(&mut self, base_frames: usize) -> PhResult<Value> {
    if base_frames != 0 {
        return self.run_until_inner(base_frames);
    }
    loop {
        match self.run_until_inner(0) {
            ...
        }
    }
}
```

The comment above it (L222-233) makes the dominance argument explicit:
the fiber-floor capture only applies at `base_frames == 0` because (1) a
switch is legal only at `native_reentry_depth == 0` (§5's guard), and (2)
*every* `native_reentry_depth == 0` call site happens to pass `base_frames
== 0` too — either `run()` calling `run_until(0)` directly, or this same
wrapper recursing after a switch. Every re-entrant call site
(`block_call`/`send_dynamic`/`invoke_method_object`) increments
`native_reentry_depth` **and** passes `base_frames == self.frames.len() >=
1` in the same breath, so it structurally cannot reach the `base_frames ==
0` branch. **VERIFIED** by reading every call site of `run_until`
(`dispatch.rs:238` `run_until(0)`; `send.rs:234,277`;
`interpret.rs:270`; `primitive/block.rs:159` — all four non-zero-`base_frames`
sites pair a `native_reentry_depth` increment with a `base_frames =
self.frames.len()` snapshot taken *before* pushing the callee frame).

`run_until_inner` @ L477 — doc header, quoted:

> "The inner dispatch loop, unaware of fibers: drains bytecode until
> [`Self::frames`] shrinks to `base_frames`, or a [`RuntimeError`]
> propagates out."

**Is the inner loop genuinely fiber-unaware? Partially — verified by
reading the whole ~730-line loop, not assumed.** The loop's own control —
its exit test (`self.frames.len() <= base_frames`, L491), its opcode fetch,
its dispatch `match` — never reads `self.current` or any `FiberObject`
field, and is exactly the pre-U-FIBER loop (the doc's own claim). **But two
opcode arms are not fiber-unaware:** `Bytecode::GetUpvalue` (L1052-1070) and
`Bytecode::SetUpvalue` (L1071-1087) both branch on `fiber == self.current`:

```rust
Upvalue::Open { fiber, slot } => {
    if fiber == self.current {
        self.stack[slot]
    } else {
        self.heap.fiber(fiber).stack[slot]
    }
}
```

This exists because an open upvalue can be captured by a closure whose home
frame is on a fiber that later parks while the closure itself escapes and
runs on a different fiber (the `concurrency_fiber_captures_enclosing_local.ph`
fixture, §10) — reading/writing through it must reach the *owning* fiber's
possibly-parked stack, not whatever fiber happens to be running. So: the
loop's **exit condition and dispatch structure** are fiber-unaware
(unchanged from pre-fiber code); **two specific arms** are fiber-aware by
necessity, for a reason orthogonal to the switch/guard mechanism this doc
maps (upvalue ownership, not control transfer).

---

## 5. The two guards

Both in `phalcom-core/src/primitive/fiber.rs`.

`fiber_resume` (backing both `fiber_call`/`fiber_try`) @ L247, the guard
(L248-250) — **absolute**:

```rust
if vm.native_reentry_depth != 0 {
    return Err(cannot_resume_across_native_frame(vm));
}
```

`fiber_yield` @ L333, the guard (L338-340) — **relative to the fiber's own
recorded floor**:

```rust
if vm.native_reentry_depth != vm.heap.fiber(me).floor_depth {
    return Err(cannot_yield_across_native_frame(vm));
}
```

**The asymmetry, from the code's own doc comments** (`fiber.rs:86-98`,
`cannot_resume_across_native_frame`'s doc):

> "This is a *resume*, not a *yield* — spec §6's restriction table only
> forecloses yielding underneath a native frame, so this is a deliberately
> **wider, sound over-restriction** (a nested `run_until`'s `base_frames` is
> computed against the currently running fiber, which any switch underneath
> it — resume or yield alike — would corrupt)."

So: the *spec* only forbids `yield` under a native frame; a resume attempted
under one is not separately specified as illegal, but HEAD forbids it anyway
because the hazard (`base_frames` computed against the fiber that is about
to be swapped out) is identical for both directions. `fiber_resume`'s guard
is written against absolute `0` because a resumer that is itself already
under **any** native reentry (depth `>0`) is unconditionally unsafe to
switch away from — there is no floor to be relative to, because the
resuming fiber's own floor is irrelevant to whether swapping *it* out is
safe.

`fiber_yield`'s guard is relative to `floor_depth` because a yield's hazard
is about depth *accrued since this fiber's own last resume*, not depth in
absolute terms — a fiber that was itself resumed from underneath a native
frame (hypothetically, if `fiber_resume`'s own guard ever allowed that) would
have a nonzero floor, and a subsequent yield should compare against *that*
floor, not `0`. §6 shows this hypothetical is currently unreachable.

---

## 6. ADVERSARIAL CHECK — `floor_depth` always `0`?

**VERIFIED-TRUE.** Every writer of `floor_depth`, full census
(`grep -n "floor_depth"` across `phalcom-core/src`, 4 assignment sites, all
others are reads or doc comments):

1. `heap/fiber.rs:138` (`FiberObject::new_entry`) — constant `0`.
2. `heap/fiber.rs:160` (`FiberObject::new_entry_with_buffers`, fiber-pool
   feature) — constant `0`.
3. `heap/fiber.rs:181` (`FiberObject::root`) — constant `0`.
4. `primitive/fiber.rs:317` (`fiber_resume`) — the only non-constant writer:
   `vm.heap.fiber_mut(callee_ref).floor_depth = vm.native_reentry_depth;`

Site 4 is the one that could in principle write something other than `0`.
It cannot, at HEAD, by direct dominance: `fiber_resume`'s **first**
statement (L248-250) is

```rust
if vm.native_reentry_depth != 0 {
    return Err(cannot_resume_across_native_frame(vm));
}
```

an early return that fires on any nonzero depth. Everything from that check
to the L317 write (L251-316: fiber-status checks, arity validation,
`store_live_into`, stack/frame pushing for a first resume or
`load_live_from` for a subsequent one) touches only `vm.heap`, `vm.stack`,
`vm.frames`, `vm.current` — **no call in this stretch re-enters
`run_until`, `send_dynamic`, `block_call`, or anything else that could
change `vm.native_reentry_depth`.** I read the full body of `fiber_resume`
(`fiber.rs:247-321`) to confirm this — it is straight-line VM/heap
manipulation with no native re-entrant call anywhere in it. So by the time
L317 runs, `vm.native_reentry_depth` is provably still `0` — the same value
the L248 check just verified.

**Conclusion: `floor_depth` is always `0` at HEAD**, exactly as claimed.
Consequently, `fiber_yield`'s relative check (§5) —
`vm.native_reentry_depth != vm.heap.fiber(me).floor_depth` — is currently
**exactly equivalent to** `vm.native_reentry_depth != 0` in every reachable
program. The relative form is not dead code (it is what a future world where
`fiber_resume`'s own guard is relaxed would need), but it buys nothing yet.

---

## 7. Census of `native_reentry_depth` increment/decrement sites

Four sites, one pair each (all `+= 1` immediately before a re-entrant
`run_until`, `-= 1` immediately after):

| Site | What is being re-entered | Why it must be counted |
|---|---|---|
| `interpret.rs::Interpreter::import_module` @ L269/271 | The imported module's own top-level unit, run synchronously to completion before `import` returns a value | A fiber switch mid-import would corrupt the `base_frames` this call computed against the *currently running* fiber |
| `vm/send.rs::VM::send_dynamic` @ L233/235 | The method resolved for a reflective `Object#perform`/`performWith` (and transitively the `doesNotUnderstand(_)` forward) | Same hazard — `send_dynamic`'s own `base_frames` is a snapshot of the running fiber's frame count |
| `vm/send.rs::VM::invoke_method_object` @ L276/278 | The exact method behind `Method#invokeOn(_,_)` / a bound method's `call` (`Method#bind(_)`) | Same hazard, no lookup step but the same synchronous-recovery re-entry |
| `primitive/block.rs::block_call` @ L158/160 | A `Block`/`Closure`'s own body, via `Function#call` (`f.call(args)`) — the primitive underneath every `.ph`-level collection combinator (`each`, `map`, `reduce`, …) that invokes a block | Same hazard, and this is the one users hit in practice: `list.each { Fiber.yield(x) }` routes here |

---

## 8. MECHANICAL CONFIRMATION of the inliner claim

**Tooling caveat, verified first:** `phalcom-core/bin/phalcom/disasm.rs`
(21 lines, quoted in full):

```rust
pub fn disassemble_source(source: &str) -> Result<(), PhError> {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<main>");
    let closure = vm.compile_closure(module, source)?;
    let chunk = vm.heap.closure(closure).callable.chunk.clone();
    println!("Constants:");
    for (i, constant) in chunk.constants.iter().enumerate() {
        println!("  [{}] {:?}", i, constant);
    }
    println!("\nBytecode:");
    for (i, instr) in chunk.code.iter().enumerate() {
        println!("  {:04}: {:?}", i, instr);
    }
    Ok(())
}
```

**Confirmed: it dumps exactly one `Chunk` — the top-level compiled unit —
and never recurses into an `Object::Closure`/`Object::Block` constant.**
Running `disasm` on the literal `Fiber.new { while (true) {...} }` /
`Fiber.new { list.each {...} }` scripts therefore shows only
`Closure(idx)` placeholders for the fiber body, not its internals — the
memory note "`disasm` only walks the top-level chunk" is confirmed exactly.

**Working around it:** block/closure compilation is structural, not
fiber-aware — `compile_while_true`/`compile_for`/an ordinary `Invoke` emit
identically whether the enclosing unit is a top-level module, a method
body, or a `Fiber.new { }` block literal. So an equivalent **top-level**
`while` and an equivalent top-level `.each` (same constructs, no `Fiber.new`
wrapper) exercise the same compiler code paths and are directly
disassemblable. This is a sound proxy, not a claim about the literal fiber
body's own chunk (which I did not, and structurally could not, disassemble
through this CLI).

**Half 1 — `while` is frameless, CONFIRMED mechanically.** `disasm` on:

```phalcom
var n = 0
while (n < 3) { n = n + 1 }
```

produces (fast-path slice):

```
0002: GuardBlock(12)
0003: GetGlobal(2)
0004: InvokeConst(3, 1, 4)
0005: Invoke(1, 4)
0006: JumpIfFalse(6)
0007: GetGlobal(5)
0008: InvokeConst(6, 1, 7)
0009: Invoke(1, 7)
0010: SetGlobal(8)
0011: Pop
0012: Loop(-10)
```

`GuardBlock`/`JumpIfFalse`/`Loop` — the condition and body are spliced
directly into the enclosing chunk; the only `Closure` opcodes in the whole
program (`0015`/`0016`, not shown above) are the **deopt fallback**'s
literal-block materialization, reachable only if the guard fails (e.g.
`Block`'s sacred selectors were overridden) — never taken on the fast path.
No frame push, no native call, on the path that runs.

**Half 2 — `.each { }` is an ordinary `Invoke`, and the native reentry is
`Block#call`, CONFIRMED mechanically + at the source level.** `disasm` on:

```phalcom
let list = [1, 2, 3]
list.each { x => System.print(x) }
```

produces:

```
0009: GetGlobal(9)
0010: Closure(10)
0011: Invoke(1, 11)
```

The block literal is materialized as a real `ClosureObject` (`Closure(10)`)
and passed as an ordinary argument to an ordinary `Invoke` of `each(_)` —
**no inlining, no guard, no jump**. Reading `each`'s own definition, it is
**not** a Rust primitive at all — `List` inherits it from
`Iterable#each(f)` in `.ph` source, `phalcom-core/core/core.ph:654-658`:

```phalcom
each(f) {
  for (x in self) {
    f.call(x)
  }
}
```

`for` here is the compiler's own direct-jump lowering (`compile_for`,
`compiler/lib/loops.rs:120`, a `$cursor`/`Loop`/`JumpIfNone` sequence in the
same chunk — a **separate** mechanism from ADR-0018's sacred-selector
inliner, not one of its five selectors, but frameless for the same reason).
The one native-reentrant frame per iteration is `f.call(x)` — an ordinary
`Invoke` of `call(_)` on a `Block` value, which resolves to
`MethodKind::Primitive(block_call)` (`primitive/block.rs:117`, §7's fourth
row). **This precision goes beyond the ADR's own framing**: it is not
"`.each` is special-cased to re-enter natively," it is "`.each`'s `.ph`
implementation loops for free via `for`, and pays one native frame per
element only because it calls the block through the ordinary `Function#call`
primitive" — confirmed by reading `core.ph` directly, not inferred from the
ADR's prose.

**Empirical confirmation** (§9, `t2.ph`/`t5.ph`): the actual runtime error
text — `"cannot switch fibers across a native call frame (e.g. inside .each
{ })"` / `"cannot resume a fiber across a native call frame..."` — fires
exactly where this trace predicts, both for a `yield` and (§9's fifth
program) a `call` attempted from inside `.each`.

Nothing here is marked INFERRED — every half was either read directly in
source or shown via `disasm` + a matching runtime reproduction.

---

## 9. Observed program output

Built: `cargo build --release -p phalcom-core --bin phalcom` (clean, one
pre-existing unrelated warning — unused `init_selector_cache` field — no
errors). Invoked as `./target/release/phalcom <file>.ph`.

**Program 1** — counting generator via inlined `while`:

```phalcom
let f = Fiber.new { var n = 0; while (true) { Fiber.yield(n); n = n + 1 } }
System.print(f.call())
System.print(f.call())
System.print(f.call())
```

Output:

```
0
1
2
```

(Note: the ADR's own snippet uses `let n = 0`, which does **not** compile at
HEAD — independently reproduced: `let n = 0; ...; n = n + 1` → `Error:
Cannot reassign immutable \`let\` binding 'n'; declare it with \`var\` to
allow mutation.` I used `var n` above to get a runnable program; see §12.)

**Program 2** — `list.each { Fiber.yield(x) }`:

```phalcom
let list = [10, 20, 30]
let f = Fiber.new { list.each { x => Fiber.yield(x) } }
System.print(f.call())
```

Output (exit code 1):

```
cannot switch fibers across a native call frame (e.g. inside .each { })
```

**Program 3** — index-iteration rewrite (the workaround the ADR names):

```phalcom
let list = [10, 20, 30]
let f = Fiber.new {
  var i = 0
  while (i < list.size) {
    Fiber.yield(list.at(i))
    i = i + 1
  }
}
System.print(f.call())
System.print(f.call())
System.print(f.call())
System.print(f.call())
```

Output:

```
10
20
30
None
```

(Fourth `call()` drains the fiber past its last `yield`; the body falls off
the end with no explicit `return`, surfacing `None` — matches §1's
`surface_absence` behavior.)

**Program 4** — same callback generator, `f.try()` instead of `f.call()`:

```phalcom
let list = [10, 20, 30]
let f = Fiber.new { list.each { x => Fiber.yield(x) } }
let result = f.try()
System.print(result.class.name)
```

Output (exit code 0 — `try` **captures** rather than propagates):

```
CannotYieldAcrossNativeFrame
```

**Program 5** — the resume-side gate, `Fiber#call` from inside
`[1].each { }`:

```phalcom
let f = Fiber.new { 42 }
[1].each { x =>
  System.print(f.call())
}
```

Output (exit code 1):

```
cannot resume a fiber across a native call frame (e.g. inside .each { })
```

All five match the source-level prediction exactly; none required
adjusting the mental model built from §§2-8.

---

## 10. Fixtures — `phalcom-core/tests/lang/concurrency/`

**Positive lane** (top-level dir, 32 `.ph`/`.expected` pairs) —
one line each:

*Fiber core:*
- `concurrency_fiber_abort_then_resume_fails` — after `abort`+`try` leaves a
  fiber `Failed`, a second resume (via a nested driver) raises the same
  "cannot resume a finished fiber" as `Done`.
- `concurrency_fiber_call_resume_value` — a `call(_)` argument is delivered
  as `Fiber.yield(_)`'s return value.
- `concurrency_fiber_captures_enclosing_local` — a fiber body closing over a
  parked enclosing frame's local reads/writes through the *owning* fiber's
  stack (regression for the cross-fiber open-upvalue panic; §4's
  `GetUpvalue`/`SetUpvalue` fiber-aware branch, exercised directly).
- `concurrency_fiber_is_done_and_error_once_failed` /
  `_is_done_false_while_suspended` / `_is_done_true_once_done` —
  `isDone`/`error` reflect exactly `Failed`/mid-generator/`Done`.
- `concurrency_fiber_multi_round_accumulate_then_complete` — state
  accumulates across three `call(_)` resumes, then completes by falling off
  the end.
- `concurrency_fiber_nested_current_identity` — `Fiber.current` inside a
  fiber-driven-fiber answers the innermost, not the outer.
- **`concurrency_fiber_restricted_yield_guard`** — `yield` under a
  `Function#call`-driven block (not the sacred-inlined `whileTrue`) raises
  `CannotYieldAcrossNativeFrame` — this is the guard's own canonical
  positive-lane regression test (labelled `status: PASS` because the raise
  itself, caught via `try()`, is the expected/passing behavior).
- `concurrency_fiber_try_abort_current` — `try` captures an uncaught
  `Fiber.abort` as the delivered `Error`.
- `concurrency_fiber_two_way_channel` — `yield`'s return value both
  delivers and receives across successive resumes (not a one-shot echo).
- `concurrency_fiber_yield_resume` — baseline: successive counter yields
  across resumes.
- `fiber_first_resume_arity_mismatch_does_not_corrupt_resumer` — regression:
  a failed arity check on a not-yet-started callee must not have already
  stolen the resumer's live stacks (`store_live_into` ordering bug).
- `each_generator_raises` — `Fiber.yield` reached through `List#each`'s
  native block-call frame raises the same error `for`'s direct-jump
  lowering never triggers; names the §8 distinction explicitly in its own
  comment.

*Wren-ported family* (`concurrency_fiber_wren_*`) — each one line, what Wren
semantics it validates against:
- `_abort_number_captured` — Wren `abort_not_string.wren`: `Fiber.abort(_)`
  accepts any value, not only strings.
- `_abort_string_captured` — Wren `abort.wren` (the `try()`-return half):
  `abort`'s payload comes back from `try()` unwrapped.
- `_call_basic` — Wren `call.wren`: a no-yield entry runs to completion on
  the first `call()`.
- `_call_return_implicit_none` — Wren `call_return_implicit_null.wren`: no
  explicit `return`/`yield` still delivers the last statement's value
  (`None` here) as `call()`'s result.
- `_is_done_and_error` — Wren `is_done.wren` + `error.wren` merged:
  `isDone`/`error` semantics across `Suspended`/`Done`/`Failed`.
- `_try_value_error_capture` — Wren `try_value.wren`: `try(_)` delivers a
  first-resume argument and captures a later uncaught failure as the result.
- `_try_value_yield` — Wren `try_value_yield.wren`: a first-resume argument
  plus a suspend-at-`yield`, then a second `try(_)` delivers the yield's
  return value and captures the subsequent failure.
- `_try_without_error` — Wren `try_without_error.wren`: `try` behaves
  exactly like `call` when nothing raises.
- `_yield_sequence` — Wren `yield.wren`: three bare `yield()`s interleave
  with the caller's own prints across three resumes.
- `_yield_with_value_implicit_none` — Wren `yield_with_value.wren`: two
  valued yields, then falling off the end delivers implicit `None` on the
  third `call()`.

*Futures/scheduler* (out of this doc's scope per §11, named only): the
`concurrency_future_*` family (Slice-A `then`/`map`/`catch` synchronous-
settle semantics) and `concurrency_sched_*` (FIFO order, root-drive pump,
`runScheduled` draining) — these exercise `ready_queue`/`System.schedule`,
not the switch/guard mechanism itself.

**Negative lane** (`negative/`, 8 pairs):
- `fiber_abort_root_raises` — aborting the root fiber (no resumer) is
  illegal, uncaught.
- `fiber_call_finished_uncaught` — Wren `call_done.wren`: resuming a `Done`
  fiber, uncaught (contrast the *caught* form in the positive-lane
  `_abort_then_resume_fails`).
- `fiber_cross_fiber_non_local_return_dead_frame` — a captured `return`
  whose home frame is on a different, now-gone fiber raises
  `DeadFrameError` via the frame-token generation check.
- `fiber_new_wrong_arg_type` — Wren `new_wrong_arg_type.wren`: `Fiber.new(_)`
  requires a `Block`/`Closure`.
- `fiber_reenter_direct_call` — Wren `call_direct_reenter.wren`: a fiber
  resuming itself from inside its own entry is `Running`, distinct message
  from the finished-fiber case.
- **`fiber_resume_gate_call_native_frame`** — `Fiber#call` from inside
  `List#each`'s block-call frame is rejected by the *resume* gate
  (`fiber_resume`'s absolute check, §5), with wording naming "resume," not
  reusing `yield`'s wording — this is §5's asymmetry, exercised as its own
  regression test.
- `fiber_try_finished_uncaught` — Wren `try_done.wren`: `try()` on `Done`
  shares the same guard as `call()`, uncaught here.
- `fiber_try_first_resume_arity_mismatch_names_try` — regression: a
  first-resume arity error via `try(_)` must name "try," not hardcode
  "call."
- `fiber_yield_no_resumer` — Wren
  `yield_with_value_from_main.wren`: the root fiber has no resumer, so
  top-level `Fiber.yield(_)` is illegal.

---

## 11. SCOPE CHECK — `ready_queue`, `System.schedule`, root-drive pump

Not load-bearing for the switch mechanism or the yield restriction
specifically — a separate scheduling concern layered on top of the same,
unmodified switch primitives. `System.schedule(_)`
(`primitive/system.rs::system_schedule` @ L56) does exactly one thing:
`vm.ready_queue.push_back(fiber_ref)` (L61) — a plain `VecDeque` push, no
interaction with `native_reentry_depth`, `switch_pending`, or `floor_depth`
at all. The root-drive pump (`dispatch.rs:261-265`, inside `run_until`'s
`Ok(value)` arm, when the finishing fiber has no resumer) pops the queue and
calls `fiber_try` — the **ordinary** resume primitive, subject to the exact
same `native_reentry_depth == 0` gate (§5) as any other `call`/`try`. So:
scheduling decides *which* fiber gets resumed and *when* a resume is
triggered automatically; it reuses the switch/guard machinery this doc maps
completely unchanged, rather than extending or bypassing it.

---

## 12. ADR-0030 — bounded read (§4, §5, Alternatives)

**§4 (Execution model, restricted Option A):** ratifies exactly what HEAD
implements structurally — `yield` integrates only with the top-level
`run_until`; a re-entrant native frame on the Rust stack forecloses it;
`while`/`ifTrue:` inline to `Jump`/`Loop` (no frame, no native reentry) so
the canonical generator works; `list.each { Fiber.yield(x) }` is foreclosed,
with the index-iteration rewrite as the documented workaround. **Confirmed
against HEAD in full** (§§4, 8, 9).

One divergence in §4's own illustrative snippet: it is written `let n = 0;
... n = n + 1`. **This does not compile at HEAD** — independently
reproduced (§9's note): `Cannot reassign immutable \`let\` binding 'n';
declare it with \`var\` to allow mutation.` The ADR predates the `let`/`var`
mutability split; its example needs `var` to run today. Cosmetic relative to
the execution-model claim, but a real gap between the ADR's literal text and
a runnable HEAD program.

**§5 (typed signal, not a length delta):** **the load-bearing divergence**,
detailed in full at the top of this doc. The ADR specifies a typed
`ControlFlow`/return-value signal *out of the primitive*; HEAD implements a
`bool` flag on `VM` (`switch_pending`) instead, a deviation the U-FIBER
implementation spec recorded and named (D-FIB-5) rather than silently
drifting into. The *effect* the ADR wanted — "conflating [a switch] with an
ordinary return would misread a swap as a return" — is achieved either way;
only the *mechanism* differs from the ADR's literal prescription.

**Alternatives considered (B — full trampoline, C — stackful coroutines,
preemptive, resumable-Smalltalk):** all four are rejection records, not
implementation claims, so there is nothing in HEAD to confirm or contradict
them against — no trampolining of callback primitives exists (B not built,
matching "Not now"), no native stack-switching dependency exists in
`Cargo.toml`/`phalcom-core/src` (C not built, matching "Rejected"). Nothing
to flag here; these sections describe roads not taken, and HEAD indeed has
not taken them.

---

## Anchors referenced (symbol-first)

- `phalcom-core/src/heap/fiber.rs::FiberObject` @ L62, `::FiberStatus` @ L12,
  `::FiberResumeMode` @ L37, `::new_entry`/`::root` @ L124/L170
- `phalcom-core/src/vm/mod.rs::VM::switch_pending` @ L79,
  `::native_reentry_depth` @ L91
- `phalcom-core/src/vm/dispatch.rs::VM::run_until` @ L221,
  `::run_until_inner` @ L477, `::switch_to_fiber_and_deliver` @ L352
- `phalcom-core/src/vm/send.rs::VM::call_method` @ L19 (`Primitive` arm
  L22-94), `::send_dynamic` @ L218, `::invoke_method_object` @ L259
- `phalcom-core/src/primitive/fiber.rs::store_live_into`/`load_live_from` @
  L29/L49, `::fiber_resume` @ L247, `::fiber_yield` @ L333
- `phalcom-core/src/primitive/block.rs::block_call` @ L117
- `phalcom-core/src/interpret.rs::Interpreter::import_module` @ L231
- `phalcom-core/src/compiler/lib/loops.rs::compile_for` @ L120
- `phalcom-core/core/core.ph::Iterable#each` @ L654
- `phalcom-core/bin/phalcom/disasm.rs::disassemble_source` @ L7
- `docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md`
- `docs/forge/units/U-FIBER/implementation-spec.md` (D-FIB-5, L332)
