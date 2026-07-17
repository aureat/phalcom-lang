# Phalcom closures: how upvalues are captured — source map at HEAD

Read-only investigation. All claims below are VERIFIED against source at HEAD
(commit `f874e6c` and worktree state) unless marked INFERRED. Line numbers are
current as of this read; symbols are the stable anchor.

## THE ANSWER: yes, Lua-style open/closed upvalues, and it is fully implemented

Phalcom implements **Lua-style open/closed upvalues**, not flat-copy capture,
not box-everything, not heap-allocated frames, and not capture-by-value-only.
This is a deliberate, ADR-backed design (ADR-0013), not an accident of
convenience, and it is live end-to-end: compiler resolution, bytecode,
VM runtime, GC tracing, and fiber interaction all implement it consistently.
It is also empirically verified below by running real `.ph` programs, not
just read from source.

The settling evidence:

- `phalcom-core/src/heap/upvalue.rs::Upvalue` — a two-variant enum, `Open { fiber, slot }` / `Closed(Value)`. This *is* the open/closed cell.
- `phalcom-core/src/vm/dispatch.rs::VM::capture_upvalue` (~L61) and `VM::close_upvalues_from` (~L79) — the promote-on-scope-exit machinery, keyed by a `BTreeMap<usize, ObjRef>` of currently-open cells.
- `phalcom-core/src/compiler/lib/scope.rs::Compiler::resolve_upvalue_in` (~L149) — the textbook Lua/Crafting-Interpreters recursive resolver (`is_local` in the immediate parent vs. chain through an ancestor's own upvalue list).
- ADR: `docs/adr/accepted/0013-closure-upvalues-and-frame-token-return.md`, Status: Accepted, 2026-07-11. Explicitly named "Open/closed upvalues and frame-token non-local return", explicitly rejects by-value snapshot capture as an alternative because it "breaks shared [mutation]".

There is no partial/stubbed state here: block escape, shared mutation between
two closures, nested (grandparent) capture, cross-fiber capture, per-iteration
loop-variable freshness, and the dead-frame trap for non-local return are all
implemented AND covered by passing `.ph` test fixtures, three of which were
re-run live during this investigation (below) and matched their `.expected`
files exactly.

---

## 1. The data structures

### The upvalue cell — `phalcom-core/src/heap/upvalue.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Upvalue {
    Open {
        /// The fiber whose stack holds the home slot.
        fiber: ObjRef,
        /// The absolute slot index on that fiber's stack.
        slot: usize,
    },
    Closed(Value),
}
```

- `Open.fiber` — not present in vanilla Lua; added because the VM stack is
  swapped per-fiber (ADR-0030), so a slot index alone is ambiguous — it must
  be resolved against the stack of whichever `FiberObject` actually owns the
  home frame, not whatever fiber happens to be currently running.
- `Open.slot` — an **absolute index** into that fiber's `Vec<Value>` stack, not a pointer.
- `Closed(Value)` — the promoted, self-contained copy.

### The closure — `phalcom-core/src/heap/closure.rs::ClosureObject`

```rust
#[derive(Debug, Clone)]
pub struct ClosureObject {
    pub callable: Rc<Callable>,
    pub module: ObjRef,
    pub upvalues: Vec<ObjRef>,
}
```

`upvalues: Vec<ObjRef>` is a vector of **handles to heap-allocated `Upvalue`
cells**, not the cells themselves. The compiler emits a *template* closure
constant with an empty `upvalues` vec; the VM materializes a fresh,
upvalue-filled `ClosureObject` on every execution of `Bytecode::Closure`
(closure.rs doc comment, ~L18-22).

### The compile-time descriptor — `phalcom-core/src/callable.rs::UpvalueDescriptor`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpvalueDescriptor {
    /// True if the variable is in the immediately enclosing stack frame.
    /// False if it is captured from a outer frame.
    pub is_local: bool,
    /// The index on the stack (if `is_local` is true) or the index in the outer
    /// closure's upvalue list (if `is_local` is false).
    pub index: usize,
}
```

This is baked into `Callable::upvalues` (a `Vec<UpvalueDescriptor>` on the
compiled `Callable`, `callable.rs` ~L21-34) — this is the classic
Lua "is it a local of my immediate parent, or an upvalue I inherit through my
parent" encoding.

### First-class block wrapper — `phalcom-core/src/heap/block.rs::BlockObject`

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockObject {
    pub closure: ObjRef,
    pub home_frame_token: FrameToken,
}
```

A `Block` is a `ClosureObject` handle plus a `FrameToken` — the non-local
return machinery (§13 below), orthogonal to upvalue capture itself.

---

## 2. Open vs closed representation, and how the Rust self-reference trap was avoided

Encoding: a plain **enum variant** (`Upvalue::Open{..}` / `Upvalue::Closed(..)`)
— no `Option`, no tagged pointer, no `Rc<RefCell<..>>`.

**The self-pointing-struct problem the prompt asked about does not arise
here, by construction**, because neither state holds a Rust reference or raw
pointer into anything:

- `Open` holds `slot: usize` — an **index**, not a pointer, into the owning
  fiber's `Vec<Value>` stack.
- Every cross-object link in this codebase (`ObjRef`, used for `fiber` above
  and for the closure's `upvalues: Vec<ObjRef>`) is a **generational
  slotmap key** (`phalcom-core/src/heap/mod.rs::ObjRef`, via
  `slotmap::new_key_type!`), not a pointer either.

Quoted rationale, `heap/mod.rs` module doc (~L1-25), realizing ADR-0009:

> "keys (`ObjRef`) are `Copy` and generational, so a stale handle resolves to
> a clean `None` rather than undefined behavior (no use-after-free);
> interior mutability lives here, in the arena, instead of in a per-object
> `RefCell`, which removes the double-borrow panic hazard entirely... The
> arena is a `slotmap::SlotMap`... with **zero `unsafe`** in this crate."

So the answer to "how did the implementation solve the self-referential
struct problem": **it sidesteps it entirely** — the whole heap (including
every `Upvalue` cell, every `ClosureObject`, every `Object` variant) lives in
one `slotmap::SlotMap<ObjRef, Object>` arena owned by `Heap`
(`phalcom-core/src/heap/mod.rs::Heap`, ~L88-97), and objects refer to each
other only via `Copy` index+generation handles resolved through the arena.
No `Rc<RefCell<T>>`, no `Cell`, no raw pointers, no `unsafe` anywhere in this
mechanism (per the module doc's explicit claim, "zero `unsafe` in this
crate" — VERIFIED by the doc's assertion and by the accessor code read,
not independently re-audited for every line of the crate).

This also happens to be *why* growing the VM's stack `Vec` cannot corrupt an
open upvalue (see §11) — an `Open` upvalue never points at stack memory
directly, only at a `(fiber, slot)` pair resolved fresh on every access.

---

## 3. Is the read path branchless? No — two branches, by design

`phalcom-core/src/vm/dispatch.rs`, `Bytecode::GetUpvalue` handler (~L1052-1069):

```rust
Bytecode::GetUpvalue(idx) => {
    let cell = self.heap.closure(closure_id).upvalues[idx as usize];
    let value = match *self.heap.upvalue(cell) {
        Upvalue::Open { fiber, slot } => {
            if fiber == self.current {
                self.stack[slot]
            } else {
                self.heap.fiber(fiber).stack[slot]
            }
        }
        Upvalue::Closed(value) => value,
    };
    let value = self.surface_absence(value);
    self.stack.push(value);
}
```

Two branch points on every read: (1) `Open` vs `Closed`, and (2), only
inside `Open`, whether the owning fiber is the one currently running (reads
the live `self.stack` if so, else the target `FiberObject`'s parked stack).
`SetUpvalue` (~L1071-1086) mirrors this exactly for writes. This is *not* the
single-indirection-works-in-both-states design some Lua-family
implementations use — Phalcom pays a real branch (two, when cross-fiber) on
every upvalue access. INFERRED characterization: this trades a small amount
of per-access branching for a representation that generalizes cleanly across
fiber-parked stacks without needing pointer patching on every fiber switch.

---

## 4. The identity invariant — enforced per-VM (mirroring per-fiber) via a sorted `BTreeMap`

`phalcom-core/src/vm/mod.rs`:
```rust
pub(crate) open_upvalues: BTreeMap<usize, ObjRef>,
```
keyed by **absolute stack slot index**. This is the live, currently-running
fiber's map. Every `FiberObject` (`phalcom-core/src/heap/fiber.rs`, ~L78)
also carries its own `open_upvalues: BTreeMap<usize, ObjRef>` — the *parked*
mirror, populated when that fiber is swapped out and merged back into
`VM::open_upvalues` when it becomes current again (INFERRED from the doc
comments at `vm/mod.rs` ~L203 and `fiber.rs` ~L54-78 describing this as "the
live mirror"; the actual swap function was not read line-by-line). It is a
`BTreeMap`, so it is sorted by construction — this is what makes
`close_upvalues_from` a cheap range scan rather than a linear filter.

Search/insert function, `VM::capture_upvalue` (`vm/dispatch.rs` ~L61-68):

```rust
fn capture_upvalue(&mut self, stack_index: usize) -> ObjRef {
    if let Some(&existing) = self.open_upvalues.get(&stack_index) {
        return existing;
    }
    let cell = self.heap.alloc(Object::Upvalue(Upvalue::Open { fiber: self.current, slot: stack_index }));
    self.open_upvalues.insert(stack_index, cell);
    cell
}
```

Lookup-or-insert on the slot index: a second closure capturing the same live
local gets back the *same* `ObjRef`, guaranteeing shared identity. This was
directly verified at runtime (§4a below).

### §4a. Empirical verification: shared-cell identity

`phalcom-core/tests/lang/blocks/blocks_shared_upvalue_two_closures.ph`:
```
var count = 0
let inc = { count = count + 1 }
let show = { count }

inc.call()
inc.call()
System.print(show.call())
inc.call()
System.print(show.call())
```
Ran live via `cargo run -p phalcom-core --bin phalcom`. Output:
```
2
3
```
Matches `blocks_shared_upvalue_two_closures.expected` exactly — two
independently-created closures over the same `var` observe each other's
mutations, proving the cell is aliased, not copied per-closure.

---

## 5. Closing — both at block/scope exit AND at frame return; not return-only

Core function, `VM::close_upvalues_from` (`vm/dispatch.rs` ~L79-86):

```rust
fn close_upvalues_from(&mut self, from: usize) {
    let to_close: Vec<usize> = self.open_upvalues.range(from..).map(|(&idx, _)| idx).collect();
    for idx in to_close {
        let cell = self.open_upvalues.remove(&idx).expect("open upvalue present");
        let value = self.stack[idx];
        *self.heap.upvalue_mut(cell) = Upvalue::Closed(value);
    }
}
```

Call sites (all VERIFIED in `vm/dispatch.rs`):

| Site | Opcode/event | Boundary closed |
|---|---|---|
| `Bytecode::CloseUpvalue(slot)` handler (~L1088-1092) | explicit compiler-emitted opcode | `stack_offset + slot` and above |
| `Bytecode::Return` handler (~L1093-1109) | normal method/block return | `popped.stack_offset` and above |
| `Bytecode::ReturnNonLocal` handler (~L1110-1153) | non-local `return` through a block | `home_stack_offset` and above (covers every frame popped in the unwind, in one call) |
| `VM::unwind_to` (~L110-114) | caught `Raise`/exception unwind | `stack_len` (snapshot) and above |

So closing is **not** return-only. The compiler proactively emits
`Bytecode::CloseUpvalue` at ordinary **block/scope exit**
(`compiler/lib/scope.rs::Compiler::end_scope`, ~L91-106):

```rust
pub(crate) fn end_scope(&mut self, range: SourceRange) {
    let func = self.functions.last_mut().unwrap();
    func.scope_depth -= 1;
    let scope_depth = func.scope_depth;
    let mut to_close = Vec::new();
    while func.num_locals > 0 && func.locals[func.num_locals - 1].depth > scope_depth {
        func.num_locals -= 1;
        let local = func.locals.pop().unwrap();
        if local.is_captured {
            to_close.push(func.num_locals as u16);
        }
    }
    for slot in to_close {
        self.emit(Bytecode::CloseUpvalue(slot), range);
    }
}
```

only for locals actually marked `is_captured` (dead-cheap no-op avoidance for
the common uncaptured case). The `Bytecode::Return` handler closes again
unconditionally as a backstop — explicitly documented as idempotent
("the VM's `Return` closes them again idempotently for the explicit-return
path", `scope.rs` ~L88-90) since `close_upvalues_from` removes what it
closes, so a re-scan of an already-closed range is a no-op.

Also used surgically for the JS-`let`-style per-iteration loop fix — see §9.

---

## 6. The compiler side — a textbook Lua-style recursive resolver

`phalcom-core/src/compiler/lib/scope.rs::Compiler::resolve_upvalue_in` (~L149-168):

```rust
fn resolve_upvalue_in(&mut self, func_idx: usize, name: Symbol) -> Option<usize> {
    if func_idx == 0 {
        return None;
    }
    let enclosing = func_idx - 1;

    // 1. Resolve as a local in the enclosing function -> capture it directly.
    if let Some(slot) = self.resolve_local_in(enclosing, name) {
        self.functions[enclosing].locals[slot].is_captured = true;
        return Some(self.add_upvalue(func_idx, slot, true));
    }

    // 2. Otherwise resolve recursively as an upvalue of the enclosing
    //    function and chain through it.
    if let Some(upvalue_idx) = self.resolve_upvalue_in(enclosing, name) {
        return Some(self.add_upvalue(func_idx, upvalue_idx, false));
    }

    None
}
```

and the deduplicating recorder, `Compiler::add_upvalue` (~L172-181):

```rust
fn add_upvalue(&mut self, func_idx: usize, index: usize, is_local: bool) -> usize {
    let upvalues = &mut self.functions[func_idx].upvalues;
    for (i, upval) in upvalues.iter().enumerate() {
        if upval.index == index && upval.is_local == is_local {
            return i;
        }
    }
    upvalues.push(UpvalueDescriptor { is_local, index });
    upvalues.len() - 1
}
```

This is exactly the "is it a local here? a local in the enclosing function?
an upvalue of the enclosing function?" algorithm the prompt asked about,
walking `Compiler::functions` — a **stack** of `FunctionState`, indexed by
plain integer position, explicitly to avoid "aliasing `&mut` references and
... raw parent pointers" (`compiler/lib/state.rs::FunctionState` doc, ~L24-31).

What gets baked into the instruction: a `u16` **index into the closure's own
`upvalues` list** (`Bytecode::GetUpvalue(u16)` / `SetUpvalue(u16)`) — never a
raw stack slot at the use site. The indirection from that index to an actual
`(fiber, slot)` or `Closed(Value)` happens at runtime through the
`ClosureObject::upvalues: Vec<ObjRef>` materialized at `Bytecode::Closure`
time (§7 below).

---

## 7. Bytecode — the four upvalue/closure opcodes

From the enum definition, `phalcom-core/src/bytecode.rs` (~L176-190):

```rust
/// Creates a closure from a template.
/// 0: constant index of the template Callable/ClosureObject.
Closure(u16),

/// Pushes the value of a captured upvalue onto the stack.
/// 0: index in the closure's upvalue list.
GetUpvalue(u16),

/// Sets the value of a captured upvalue to the top value on the stack.
/// 0: index in the closure's upvalue list.
SetUpvalue(u16),

/// Closes any open upvalues pointing to slot index or above.
/// 0: stack slot index.
CloseUpvalue(u16),
```

`CloseUpvalue`'s own doc already states the "or above" range semantics that
`close_upvalues_from` implements. Opcode byte values (from the same file,
~L390-393): `Closure` = 19, `GetUpvalue` = 20, `SetUpvalue` = 21,
`CloseUpvalue` = 22.

### `Bytecode::Closure` handler — where materialization happens

`vm/dispatch.rs` ~L577-605:

```rust
Bytecode::Closure(idx) => {
    let template = callable.chunk.constants[idx as usize];
    let Value::Obj(template_id) = template else { .. };
    let descriptors = self.heap.closure(template_id).callable.upvalues.clone();
    let callable = Rc::clone(&self.heap.closure(template_id).callable);
    let module = self.heap.closure(template_id).module;

    let mut upvalues = Vec::with_capacity(descriptors.len());
    for desc in &descriptors {
        let cell = if desc.is_local {
            self.capture_upvalue(stack_offset + desc.index)
        } else {
            self.heap.closure(closure_id).upvalues[desc.index]
        };
        upvalues.push(cell);
    }

    let new_closure = self.heap.alloc(Object::Closure(Box::new(ClosureObject { callable, module, upvalues })));
    let token = self.current_frame_token().expect("closure created inside a frame");
    let block = self.heap.alloc(Object::Block(BlockObject::new(new_closure, token)));
    self.stack.push(Value::Obj(block));
}
```

Again, exactly the Lua/Crafting-Interpreters `OP_CLOSURE` algorithm: for each
descriptor, either lazily open a fresh cell over the *current* frame's stack
slot (`is_local: true`) or copy the handle straight out of the *currently
executing* closure's own upvalue list (`is_local: false`, the chained case).

---

## 8. Nested (grandparent) capture — handled, and empirically verified

Handled by the recursion in §6 (`resolve_upvalue_in` calling itself on
`enclosing`) with no depth limit other than the function-nesting stack size.
A closure that captures a *grandparent's* local through an intermediate
function that never itself references that variable still gets an
`UpvalueDescriptor{ is_local: false, .. }` chained through the intermediate
function's own `upvalues` list — `add_upvalue`'s dedup means the intermediate
function pays for exactly one upvalue slot per distinct captured name, even
though it never reads it directly.

Empirical verification — `phalcom-core/tests/lang/blocks/blocks_nested_closure_capture.ph`:
```
let makeAdder = { base =>
  { n => n + base }
}
let addTen = makeAdder.call(10)
System.print(addTen.call(5))
System.print(addTen.call(32))
```
Ran live. Output: `15` then `42` — matches `.expected` exactly. This is
two-levels-removed capture (the inner block reads `base` from the outer
block's own frame, which is itself gone by the time `addTen` is called
later), and it also demonstrates open→closed promotion surviving frame exit
correctly (the outer block's frame is long dead when `addTen.call(5)` runs).

---

## 9. Loop scoping — settled definitively, both from the compiler AND by running real programs. `for` is fresh-per-iteration (JS `let`); bare `while` is shared (JS `var`)

This is an explicit, *deliberately different* pair of behaviors for the two
loop forms, not a single blanket policy — and it is the one place in this
investigation where the compiler code is annotated as a **documented bug
fix** (U-ITER-FIX item 3, spec §3.3).

### `for` — fresh binding per iteration

`phalcom-core/src/compiler/lib/loops.rs::Compiler::compile_for`, the
per-step close before rebind (~L165-186):

```rust
// step: (the `continue` target) advance the cursor.
let step_label = self.chunk_len();

// U-ITER-FIX item 3 (spec §3.3): the loop variable is one local slot
// rebound every iteration via `SetLocal` below — without this, every
// closure the body captured it in would share the *same* open
// upvalue cell and all observe the loop's final value. Closing it
// here, before the rebind, promotes this iteration's cell (if any
// closure actually opened one) to an immutable heap copy; the next
// `SetLocal` then writes to a plain stack slot with no attached
// upvalue, so a closure captured on the *next* iteration lazily opens
// a brand-new cell instead of reusing this one — each iteration gets
// its own snapshot ...
if self.functions.last().unwrap().locals[binding_slot].is_captured {
    self.emit(Bytecode::CloseUpvalue(binding_slot as u16), range);
}
```

The loop's binding is still physically **one stack slot**, reused every
iteration — the freshness comes entirely from proactively closing (heap-
promoting) whatever cell was opened over that slot *before* the next
iteration's `SetLocal` writes a new value into it, so a closure captured on
iteration N+1 lazily opens a brand-new cell rather than reusing N's
(now-closed, now-detached) one. `continue` lands exactly at `step_label`
(i.e. at this close), so a closure captured before a `continue` still gets
its own cell.

### bare `while` — one slot, shared across all iterations, never closed early

No equivalent mechanism exists for `while`; its counter is an ordinary
`var` declared once before the loop and mutated in place
(`Bytecode::SetLocal`) every iteration with no interposed `CloseUpvalue`.
Every closure created across iterations shares the one open cell.

### Empirical verification — both loop forms, run live

`phalcom-core/tests/lang/iteration/for_loop_var_capture_freshness.ph`:
```
var closures = List.new()
for (x in List.new().add(0).add(1).add(2)) {
  closures.add({ x })
}
for (c in closures) {
  System.print(c.call())
}
```
Ran via `cargo run -p phalcom-core --bin phalcom`. Actual output:
```
0
1
2
```
Matches `.expected` exactly (JS-`let`-style fresh binding — NOT the `var` bug).

`phalcom-core/tests/lang/iteration/while_loop_var_shared_across_iterations.ph`
(identical shape, but with an explicit `var i` counter and a bare `while`):
Ran live. Actual output:
```
3
3
3
```
Matches `.expected` exactly — the classic shared-binding bug, reproduced
*intentionally* as the documented counterpoint to `for`'s fix. The test's own
header comment states this precisely: "A bare `while` has no such
machinery — its counter is one `var` declared ONCE before the loop and
mutated in place every iteration... Closures captured over it all alias the
SAME open upvalue cell."

**Bottom line for Q9: it depends on the loop form.** `for` gets fresh
per-iteration bindings (deliberately fixed, ADR/spec-cited). Bare `while`
does not and is not intended to — it is specified and tested as sharing one
binding, matching plain variable-mutation semantics rather than loop-sugar
semantics.

---

## 10. Fibers/coroutines — live interaction at HEAD, not just planned

Unlike the "planned U-FIBER work" framing in the prompt, upvalue×fiber
interaction is **already implemented and tested**, motivated directly by
ADR-0030 (fibers/futures).

- `Upvalue::Open` carries `fiber: ObjRef` specifically so a closure resumed
  on a different fiber than the one whose stack holds its home slot still
  reads/writes the correct stack (§1, §3).
- `VM::open_upvalues` is the *live fiber's* mirror; each `FiberObject` parks
  its own `open_upvalues: BTreeMap<usize, ObjRef>` while not current
  (`heap/fiber.rs` ~L78, ~L131/153/174 for the three constructors that all
  initialize it empty).
- Failure-path cleanup, `vm/dispatch.rs` ~L305-321: when a fiber fails
  uncaught, its parked state (`frames`, `stack`, **and** `open_upvalues`) is
  explicitly cleared as "pure retention" once the fiber transitions to
  `Failed` and can never resume again.
- GC traces a parked fiber's own `open_upvalues.values()` (`heap/trace.rs`
  ~L169) so upvalues owned by a currently-inactive fiber are not
  reclaimed out from under it.

### Empirical verification — cross-fiber upvalue read AND write

`phalcom-core/tests/lang/concurrency/concurrency_fiber_captures_enclosing_local.ph`,
whose own header comment cites this exact mechanism ("`gen`'s frame is
parked while `f` is resumed, so `x` is reached across fibers"):
```
class Gen {
  static run() {
    var x = 7
    let f = Fiber.new {
      Fiber.yield(x)   // read x across the fiber boundary (gen's frame parked)
      x = 99           // write x back into gen's parked stack
    }
    System.print(f.call())   // -> 7 (the yielded read)
    f.call()                 // resume: runs `x = 99`
    return x                 // gen observes the cross-fiber write
  }
}
System.print(Gen.run())      // -> 99
```
Ran live. Output: `7` then `99` — matches `.expected` exactly. A fiber body
reads and then writes a still-live local of a *parked, different* fiber's
frame through the very same open-upvalue cell, and the write is observed
back on the original fiber. This is the strongest possible evidence this
mechanism is not a stub: it is the specific regression test the fiber-aware
`Open{fiber,slot}` field exists to guard, and it passes.

---

## 11. GC / memory soundness of open upvalues — no raw pointer, no realloc hazard; checked specifically as asked

**An open upvalue does NOT hold a raw pointer into the stack.** It holds
`slot: usize` — a plain index — plus `fiber: ObjRef`, a generational arena
handle. Every read/write re-derives the actual memory location at access
time via `self.stack[slot]` or `self.heap.fiber(fiber).stack[slot]`
(quoted in full in §3).

**Consequence: `Vec<Value>` reallocation on stack growth is a non-issue by
construction**, not something separately "prevented" or "pre-sized" —
there is no pointer anywhere in the `Upvalue` type that a `Vec` grow/move
could invalidate. This is the same design choice Lua's real implementation
makes (indices, not raw stack pointers, in the upvalue-open-list use case)
and it sidesteps the classic self-referential-Vec-pointer trap entirely
rather than solving it with `unsafe` or pinning.

Two things this reasoning does NOT need to worry about, and this
investigation explicitly checked:
- Heap arena reallocation: irrelevant too — `ObjRef` is a slotmap key
  (index+generation), not a pointer into the arena's backing storage, so a
  slotmap resize behind the scenes cannot invalidate any `ObjRef` (§2).
- Cross-fiber stack swap: handled by resolving through the *correct* fiber's
  stack at access time (`fiber == self.current` branch, §3) rather than by
  keeping a pointer that would dangle across the swap.

GC correctness for the `Upvalue` object itself — `phalcom-core/src/heap/trace.rs`
(~L150-155):
```rust
Object::Upvalue(upvalue) => match upvalue {
    Upvalue::Open { fiber, slot: _ } => push(*fiber),
    Upvalue::Closed(value) => trace_value(*value, push),
},
```
An `Open` cell roots its owning `fiber` handle (so the `FiberObject` — and
transitively its parked stack — cannot be collected while an open upvalue
still points at it); a `Closed` cell traces the value it now owns directly.
`VM::collect_roots` (`vm/gc.rs` ~L29-102) roots `open_upvalues.values()` (the
live mirror) explicitly, with an exhaustive-destructure design specifically
built so a newly-added `VM` field fails to compile until someone classifies
it as root/non-root (the doc comment cites a real prior miss, `sealed_classes`
+ `checking` + `ready_queue`, forge finding F6 — INFERRED as evidence this
discipline is taken seriously, not that it's infallible).

---

## 12. Every use site — concise table

From `graphify affected "Upvalue"` and `graphify affected "ClosureObject"`,
narrowed to the load-bearing subset (both traversals returned the same
`heap/mod.rs`/`heap/accessors.rs`/`heap/trace.rs` cluster, consistent with a
single point of heap access):

| Symbol | File:line | Role |
|---|---|---|
| `Heap::upvalue` / `Heap::upvalue_mut` | `heap/accessors.rs:412,424` | typed accessor into the arena, panics on wrong-variant/stale handle |
| `Heap::closure` | `heap/accessors.rs:135` | typed accessor for `ClosureObject` |
| `Object::Upvalue` variant | `heap/object.rs:24` (enum), referenced `heap/mod.rs:230` (debug name) | tags the arena slot as holding an `Upvalue` |
| `Heap::alloc` | `heap/mod.rs:141` | allocation entry point used by `capture_upvalue` |
| `trace_object` | `heap/trace.rs:71`, upvalue arm ~150-155 | GC marking |
| `VM::capture_upvalue` / `close_upvalues_from` | `vm/dispatch.rs:61,79` | the open/search/close core (§4, §5) |
| `Bytecode::{Closure,GetUpvalue,SetUpvalue,CloseUpvalue}` handlers | `vm/dispatch.rs:577,1052,1071,1088` | runtime opcode execution |
| `VM::open_upvalues` field | `vm/mod.rs:124` | live-fiber open-cell map |
| `FiberObject::open_upvalues` field | `heap/fiber.rs:78` | parked-fiber mirror |
| `VM::collect_roots` | `vm/gc.rs:29` | roots the live map |
| `Compiler::resolve_upvalue` / `resolve_upvalue_in` / `add_upvalue` | `compiler/lib/scope.rs:143,149,172` | compile-time resolution (§6) |
| `UpvalueDescriptor` | `callable.rs:10` | compiled descriptor baked into `Callable` |
| `Local::is_captured` | `compiler/lib/state.rs:14` | marks a local for scope-exit closing |
| `Compiler::end_scope` | `compiler/lib/scope.rs:91` | scope-exit `CloseUpvalue` emission |
| `Compiler::compile_for` | `compiler/lib/loops.rs:120` | per-iteration close for freshness (§9) |

Not exhaustively re-verified line-by-line beyond what's quoted above; this
table reflects the graphify traversal plus the files this investigation
actually opened.

---

## 13. Non-local return — implemented, traps cleanly on a dead home frame

Separate mechanism from upvalue capture (same ADR, same opcode family
adjacency, different data): a `FrameToken { frame_index: usize, generation: u64 }`
(`phalcom-core/src/frame.rs::FrameToken`, ~L18-24) stamps every `CallFrame` at
creation with a **monotonically increasing generation**
(`VM::new_call_frame`, `vm/dispatch.rs` ~L29-42). A `BlockObject` created
inside a frame is stamped with that frame's current token
(`Bytecode::Closure` handler, `self.current_frame_token()`, §7).

`Bytecode::ReturnNonLocal` handler (`vm/dispatch.rs` ~L1110-1153) resolves the
executing frame's `home_frame_token`, then:

```rust
let is_live = self
    .frames
    .get(token.frame_index)
    .is_some_and(|home| home.generation == token.generation);
if !is_live {
    return Err(RuntimeError::DeadFrameError.into());
}
```

A stale index or generation mismatch (home method already returned) raises
`RuntimeError::DeadFrameError` — a clean, catchable runtime error, not
corruption or UB. This is checked *before* any stack mutation. If live, it
closes every open upvalue at or above the home frame's window in one call
(covers every popped frame in the unwind), truncates, and pushes the return
value — described in the handler's own comment as "eagerly, in one shot"
because there may be several nested `run_until`/`block_call` Rust frames
between the executing bytecode and the true home frame.

### Empirical verification

`phalcom-core/tests/lang/runtime-errors/runtime_non_local_return_dead_frame.ph`:
```
class Maker {
  make() { return { return 1 } }
}
let escaped = Maker.new().make()
System.print(escaped.call())
```
Ran live. Output:
```
non-local return from a block whose home method frame is no longer alive (DeadFrameError)
```
A block containing `return` escapes its creating method (`make` returns the
block without invoking it); invoking it later, after `make`'s frame is
long gone, traps as `DeadFrameError` rather than corrupting the stack —
matching Smalltalk's `BlockCannotReturn` semantics per the test's own header
comment. Confirms the trap-not-corrupt claim directly, not just from reading
the handler.

---

## 14. Spec/ADR — bounded search, as instructed

**Primary ADR**: `docs/adr/accepted/0013-closure-upvalues-and-frame-token-return.md`
— "13. Open/closed upvalues and frame-token non-local return", Status:
Accepted, 2026-07-11. Summarized in the intro and §5/§13 above; its
"Decision" section states the design in the same terms this investigation
independently derived from code, and its "Alternatives considered" section
explicitly rejects by-value snapshot capture for breaking shared mutation
of captured `var`s.

Grep of `docs/adr/accepted/` for closure/upvalue mentions also turned up
peripheral references in ADR-0006 (function-as-abstract-callable-root, the
`Block`/`Method` shared-representation decision this all sits on), ADR-0030
(fibers, the reason `Open` carries a `fiber` field), ADR-0050 (mark-sweep GC,
root-set discipline for `open_upvalues`), and several `amend-floor-admit-*`
ADRs with incidental mentions — not read in full; out of scope per the
"bounded, do not sweep" instruction.

Grep of `docs/spec/v0.2/` for "upvalue" (case-insensitive) hit
`deferred-work.md`, `implementation-status.md`, `functions.md`,
`performance.md`, `memory-management.md`, `concurrency.md`, plus a few
`core/`/`experimental/`/`drafts/` files. Not opened — timeboxed per
instruction; `blocks.md §5` is cited repeatedly by the source comments
themselves as the normative spec section for capture/escape semantics and is
the one worth reading next if more spec grounding is wanted, but this
investigation did not open it.

---

## 15. Tests / fixtures exercising closures or capture

All paths relative to repo root; all are golden-style `.ph` + `.expected`
pairs unless noted. Three marked `[RAN]` were executed live during this
investigation (§4a, §8, §9, §10, §13 above) and matched their `.expected`
exactly; the rest were located but not re-run.

| Path | Covers |
|---|---|
| `phalcom-core/tests/lang/blocks/blocks_shared_upvalue_two_closures.ph` **[RAN]** | identity invariant: two closures over one `var` share one cell |
| `phalcom-core/tests/lang/blocks/blocks_nested_closure_capture.ph` **[RAN]** | grandparent/chained capture + open→closed survival past frame exit |
| `phalcom-core/tests/lang/blocks/blocks_mutation_visible_in_enclosing_scope.ph` | mutation through a block visible in its home scope while open |
| `phalcom-core/tests/lang/blocks/blocks_escape.ph` | a block outliving its creating frame |
| `phalcom-core/tests/lang/iteration/for_loop_var_capture_freshness.ph` **[RAN]** | `for`'s per-iteration fresh binding (JS `let`-style) |
| `phalcom-core/tests/lang/iteration/while_loop_var_shared_across_iterations.ph` **[RAN]** | bare `while`'s shared binding (JS `var`-style), intentional counterpoint |
| `phalcom-core/tests/lang/iteration/negative/break_across_materialized_block.ph` | control-flow/closure interaction edge case (not upvalue-specific; not opened) |
| `phalcom-core/tests/lang/concurrency/concurrency_fiber_captures_enclosing_local.ph` **[RAN]** | cross-fiber open-upvalue read + write through a parked frame |
| `phalcom-core/tests/lang/functions/functions_block_equality.ph` | block/closure identity semantics (not opened in detail) |
| `phalcom-core/tests/lang/bindings/binding_let_shadow_inner_block.ph` | shadowing interaction with block scoping (not opened) |
| `phalcom-core/tests/lang/bindings/binding_var_local_to_block_not_shared_across_calls.ph` | per-call freshness of a block-local `var` (not opened) |
| `phalcom-core/tests/lang/sequence/sequence_laziness_closure_runs_on_iteration_only.ph` | closures used for lazy sequence evaluation (not opened) |
| `phalcom-core/tests/lang/runtime-errors/runtime_non_local_return_dead_frame.ph` **[RAN]** | `DeadFrameError` trap on a non-local return through a dead home frame |
| `phalcom-core/tests/lang/concurrency/negative/fiber_cross_fiber_non_local_return_dead_frame.ph` | `DeadFrameError` combined with fiber crossing (not opened) |
| `phalcom-core/tests/lang/concurrency/concurrency_fiber_multi_round_accumulate_then_complete.ph` | incidentally references `DeadFrameError` in comments (not opened) |

---

## What was inferred vs. verified — summary

**Verified** (read the exact source, or ran the program and matched
`.expected`): the `Upvalue` enum; `ClosureObject`; `UpvalueDescriptor`;
`Callable`; `BlockObject`; `FrameToken`/`CallFrame`; `capture_upvalue`;
`close_upvalues_from`; `unwind_to`; the `GetUpvalue`/`SetUpvalue`/
`CloseUpvalue`/`Closure`/`Return`/`ReturnNonLocal` opcode handlers;
`resolve_upvalue`/`resolve_upvalue_in`/`add_upvalue`; `end_scope`;
`compile_for`'s per-iteration close; the `Object::Upvalue` GC trace arm;
`VM::collect_roots`'s explicit destructure and its rooting of
`open_upvalues`; the `ObjRef`/`Heap`/slotmap arena design; five `.ph` test
programs run live end-to-end against the actual `phalcom` CLI binary built
from this tree.

**Inferred, not independently re-derived from source**: the exact fiber-swap
function that moves `VM::open_upvalues` into/out of a `FiberObject::open_upvalues`
on switch (its *existence and effect* is verified by the doc comments at
`vm/mod.rs`/`heap/fiber.rs` and by the passing cross-fiber test in §10, but
the swap function's own body was not opened); the "zero `unsafe` in this
crate" claim for `heap/mod.rs` is quoted from that file's own module doc, not
independently confirmed by grepping for `unsafe` across the whole heap
module; the full content of `docs/spec/v0.2/blocks.md §5` (cited repeatedly
by source comments as the normative spec) was not opened, per the
instruction to timebox spec reading.

**Confirmed not to exist / not applicable**: no `Rc<RefCell<..>>` anywhere in
this mechanism; no raw pointers into the stack; no realloc hazard (by
construction, not by a separate mitigation); no stub or partial state in any
of the fourteen questions above — every mechanism asked about is live and
either directly quoted or empirically exercised.
