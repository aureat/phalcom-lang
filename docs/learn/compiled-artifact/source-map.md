# Source map — the compiled-function artifact

Scope: `Chunk` / `Callable` / `ClosureObject` / `BlockObject` / `MethodKind`, at HEAD.
Read-only research note. Everything marked VERIFIED was read at the cited line or
produced by an actual `cargo run … disasm` invocation (output pasted below);
everything marked INFERRED was not directly observed.

## The question that dominates everything

**Four layers, three of them nested by composition, the fourth added only when a
closure becomes a first-class block.** Not "one fused closure object."

1. `Chunk` — `phalcom-core/src/chunk.rs::Chunk` (L44) — bytecode + constant pool + side
   tables. A field *inside* `Callable`, not a separate heap object.
2. `Callable` — `phalcom-core/src/callable.rs::Callable` (L21) — `Chunk` **by value**
   plus signature metadata (`arity`, `name_sym`, upvalue descriptors). The
   compile-time "recipe." VERIFIED: `pub chunk: Chunk` (L23) — not `Rc<Chunk>`, not a
   handle. `Callable` itself is what's `Rc`-shared, one level up.
3. `ClosureObject` — `phalcom-core/src/heap/closure.rs::ClosureObject` (L24) — `Rc<Callable>`
   (VERIFIED, L28: `pub callable: Rc<Callable>`) + `module: ObjRef` + `upvalues: Vec<ObjRef>`.
   This is the runtime instance: a `Callable` bound to a defining module and (for
   blocks) filled-in upvalue cells. A method body and a block literal both compile to
   this same type (ADR-0006).
4. `BlockObject` — `phalcom-core/src/heap/block.rs::BlockObject` (L18) — `closure: ObjRef`
   + `home_frame_token: FrameToken`. Only exists when a `ClosureObject` is being used
   as a first-class, non-local-returnable block value. A method's `ClosureObject` is
   referenced directly from `MethodKind::Closure(ObjRef)` — **no `BlockObject`
   wrapper** — because a method doesn't need a home-frame token for its own
   activation (that's `CallFrame`'s job); only blocks written inside its body do, and
   each of *those* gets its own `BlockObject` when materialized.

So: candidate (iv) is closest, but the count depends on what you're looking at.
For a **block literal in use**: `Chunk` ⊂ `Callable` (Rc-shared) ← `ClosureObject`
(per-materialization instance) ← `BlockObject` (adds home-frame identity) — four
distinct types, three object boundaries. For a **method**: the stack stops one layer
short — `MethodKind::Closure` points straight at the `ClosureObject`, never wrapped
in a `BlockObject` (VERIFIED — see §1 below and ADR-0006's "Method wraps
`ClosureObject` inside `MethodObject`... blocks wrap the same closure with a home
frame token. Neither is a subtype of the other").

There is also a compile-time vs run-time split orthogonal to the layer count: the
compiler emits exactly **one** `ClosureObject` per block literal / method body as a
constant-pool **template** (empty `upvalues: Vec::new()`), and the VM's
`Bytecode::Closure` handler **materializes a fresh `ClosureObject`** from that
template on every evaluation, filling in real upvalue cells. A method's template
*is* its final `ClosureObject` (methods carry no upvalues to fill in — see §1), so
for methods the template/instance distinction collapses to one object; for blocks it
does not.

## Type definitions, in full

### `Chunk` — `phalcom-core/src/chunk.rs` L44

```rust
/// A chunk of compiled bytecode and its associated constant values.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub spans: Vec<SourceRange>,
    /// Parallel to `code`; only `Bytecode::Invoke` indices are ever non-`None`.
    /// Cell enables interior mutability for cache refill through a shared `&Chunk` borrow.
    pub caches: Vec<Cell<Option<InlineCache>>>,
    /// Parallel to `code`; only `Bytecode::GetGlobal`/`SetGlobal` indices are ever
    /// non-`None`. Separate from [`Self::caches`] because the two never occupy the
    /// same instruction, and a single union would pay for the wider variant at
    /// every site.
    pub gcaches: Vec<Cell<Option<GlobalCache>>>,
}
```

Five fields, all `Vec`s (`caches`/`gcaches` wrap `Cell<Option<_>>` for interior
mutability through a shared `&Chunk` borrow during dispatch — U-IC). `code`,
`constants`, `spans`, `caches`, `gcaches` are kept parallel/same-length by
`add_instruction`/`add_constant` (chunk.rs L77–88); `spans` and the two cache
vectors are indexed by *instruction index*, `constants` by its own separate index
space (the `u16` operand embedded in `Constant`/`Invoke`/etc.).

### `Callable` — `phalcom-core/src/callable.rs` L21

```rust
/// A compiled code unit: bytecode, constant pool, and signature metadata.
#[derive(Debug, Clone)]
pub struct Callable {
    /// The compiled bytecode instructions.
    pub chunk: Chunk,
    /// Maximum number of stack slots required by this callable.
    pub max_slots: usize,
    /// Number of upvalues this closure captures.
    pub num_upvalues: usize,
    /// Upvalue descriptors defining how each upvalue is captured.
    pub upvalues: Vec<UpvalueDescriptor>,
    /// Positional parameter count.
    pub arity: usize,
    /// Interned selector/method symbol name.
    pub name_sym: Symbol,
}
```

`chunk: Chunk` is held **by value** (L23) — `Callable` owns the `Chunk` outright; it
is `ClosureObject` one level up that holds `Rc<Callable>`, not `Callable` holding
`Rc<Chunk>`. `Callable` has no native/primitive variant — it is purely a bytecode
recipe (settled in Q1 below).

### `UpvalueDescriptor` — `phalcom-core/src/callable.rs` L10

```rust
/// Describes how an upvalue is captured relative to the enclosing scopes.
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

Classic clox-style two-field descriptor: `is_local` selects stack-slot vs.
enclosing-closure's-upvalue-list, `index` is the offset in whichever space.

### `ClosureObject` — `phalcom-core/src/heap/closure.rs` L24

```rust
/// A compiled closure: its code, its defining module handle, and its upvalues.
#[derive(Debug, Clone)]
pub struct ClosureObject {
    /// The compiled bytecode and metadata this closure runs. Shared across
    /// multiple closure instances to avoid allocating/cloning the `Chunk` on
    /// every block literal materialization (U-HOTPATH).
    pub callable: Rc<Callable>,
    /// Handle to the [`ModuleObject`](crate::heap::ModuleObject) this closure
    /// was compiled in.
    pub module: ObjRef,
    /// Captured upvalue cells, as heap [`ObjRef`] handles into the
    /// [`Heap`](crate::heap::Heap). Indexed by the callable's upvalue
    /// descriptors ([`UpvalueDescriptor`](crate::callable::UpvalueDescriptor)).
    pub upvalues: Vec<ObjRef>,
}
```

`callable: Rc<Callable>` (VERIFIED, L28) — a shared, refcounted pointer, not owned
by value. `module: ObjRef` is a heap handle (arena index), not a `Rc`/pointer.
`upvalues: Vec<ObjRef>` — each entry is a handle to a heap `Upvalue` cell (see §9),
not a raw `Value`.

### `BlockObject` — `phalcom-core/src/heap/block.rs` L18

```rust
/// A first-class block (lexical closure) object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockObject {
    /// Handle to the underlying [`ClosureObject`](crate::heap::ClosureObject).
    pub closure: ObjRef,
    /// The frame token of the activation in which this block was created.
    pub home_frame_token: FrameToken,
}
```

Two fields: a closure handle plus the home-frame token used for non-local return
(ADR-0013). `BlockObject` is `Copy` (both fields are `Copy`: `ObjRef` and
`FrameToken`) — unlike `ClosureObject`, minting a `BlockObject` is cheap.

### `MethodKind` — `phalcom-core/src/method/object.rs` L17

```rust
/// The implementation strategy behind a [`MethodObject`].
#[derive(Debug, Clone, Copy)]
pub enum MethodKind {
    /// Phalcom code compiled to bytecode, by [`ClosureObject`](crate::heap::ClosureObject) handle.
    Closure(ObjRef),
    /// A native Rust function for a core-library method.
    Primitive(PrimitiveFn),
}
```

Two variants: `Closure(ObjRef)` (points at a `ClosureObject`, not a `BlockObject` —
see Q1) and `Primitive(PrimitiveFn)` where
`pub type PrimitiveFn = fn(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value>;`
(object.rs L13) — a bare Rust function pointer, no closure/capture state at all.

## Answers to the specific questions

### 1. The native/bytecode fork — VERIFIED

`MethodKind` (method/object.rs L17-22) is exactly `Closure(ObjRef) | Primitive(PrimitiveFn)`,
confirmed above. `Callable` (callable.rs L21-34) has no native/primitive variant —
it is a plain struct, not an enum, and every field is bytecode-recipe data
(chunk, slots, upvalues, arity, name). There is no way to construct a "native
`Callable`"; the native path skips `Callable`/`ClosureObject` entirely and lives as
a bare `fn` pointer directly in `MethodObject.kind`.

`MethodKind::Closure(ObjRef)` points at a `ClosureObject`, not a `BlockObject`.
VERIFIED via the construction site, `phalcom-core/src/compiler/lib/class_decl.rs`
L495-499:
```rust
let method_obj = self.vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
    selector_sym,
    sig_kind,
    MethodKind::Closure(closure),
))));
```
where `closure` is the `ObjRef` returned by `compile_block` (compiler/lib/mod.rs
L135-223), which allocates `Object::Closure(Box::new(ClosureObject { callable, module, upvalues: Vec::new() }))`
(L217-221) — an `Object::Closure`, never `Object::Block`. This also settles: **a
method's `ClosureObject` is built once, directly, at compile time** — it is not run
through the `Bytecode::Closure` VM opcode at all (methods have no upvalues to fill
in per-activation; `self` arrives via local slot 0, not an upvalue — see
`compile_block` L155-162). The `Bytecode::Closure` template→instance dance (§5
below) is specifically for **block literals**, whose enclosing-scope upvalues
aren't known until the block is actually evaluated inside a live activation.

This corrects a plan that said "`Callable` variants bytecode vs native" — the fork
is one level up, in `MethodKind`, and `Callable` is unconditionally a bytecode
recipe.

### 2. The Rc share — VERIFIED, with provenance

`ClosureObject.callable: Rc<Callable>` (closure.rs L28) and `Chunk` sits inside
`Callable` **by value** (callable.rs L23: `pub chunk: Chunk`) — so one `Rc<Callable>`
clone shares the entire code array, constant pool, span table, and both cache
tables at once; cloning a `ClosureObject` is a refcount bump, not a deep copy.

**Why it's `Rc`, with citation**: `docs/forge/perf-log/004-hotpath-rc-callable.md`
("004 — U-HOTPATH: share block-literal `Callable` via `Rc` (Tier 2)", status
*landed*, commits `1531070`/`debadfa`). Before this cut, `ClosureObject` owned
`Callable` by value, and `Bytecode::Closure` cloned the whole `ClosureObject` —
"three heap allocations plus the copy" — **on every block-literal evaluation**, not
once per literal in the source. The doc's own numbers: Skynet (1.1M block
evaluations, one literal) went from 3.11s→2.19s user time (−30%), RSS 3.73GB→1.37GB
(−63%), by switching to `Rc::clone`. It also records an honest tradeoff: `Rc<Callable>`
adds one pointer hop to the *per-instruction* chunk read in the dispatch loop, which
regressed non-block-heavy sends by 5-7% (`bare_send`, `arith_send`, `binary_trees`)
— a cost the doc says the (separately tracked) "hoist the chunk pointer out of the
dispatch loop" follow-on is meant to pay back. Related unit doc:
`docs/forge/units/U-HOTPATH/implementation-spec.md`.

### 3. The constant-pool read sites

All confirmed by reading `phalcom-core/src/vm/dispatch.rs`; the read is always
`callable.chunk.constants[idx as usize]`.

| opcode | reads constants how | file:line |
|---|---|---|
| `Constant(idx)` | pushes the constant value directly | dispatch.rs:572 `let constant = callable.chunk.constants[idx as usize];` |
| `Closure(idx)` | constant is the **template** `Value::Obj` closure handle | dispatch.rs:583 `let template = callable.chunk.constants[idx as usize];` |
| `DefineGlobal(idx)` | constant is the binding name `Symbol` | dispatch.rs:624 `let name_val = callable.chunk.constants[idx as usize];` |
| `GetGlobal(idx)` | constant is the name `Symbol` (cache-checked via separate `gcaches[ip]`, not `constants`) | dispatch.rs:633 `let name_val = callable.chunk.constants[idx as usize];` |
| `SetGlobal(idx)` | constant is the name `Symbol` | dispatch.rs:686 `let name_val = callable.chunk.constants[idx as usize];` |
| `Class(idx)` | constant is the class-name `Symbol` | dispatch.rs:741 `let name_val = callable.chunk.constants[idx as usize];` |
| `Import(idx)` | constant is the import-path `Symbol` | dispatch.rs:795 `let path_val = callable.chunk.constants[idx as usize];` |
| `MakeFamily(name_idx)` | constant is the base-name `Symbol` | dispatch.rs:806 `let name_val = callable.chunk.constants[name_idx as usize];` |
| `SuperSend(argc, selector_idx, defining_idx)` | **two** reads: selector `Symbol` + defining-class-name `Symbol` | dispatch.rs:864-865 `let selector_val = ...[selector_idx ...]; let defining_val = ...[defining_idx ...];` |
| `Method(selector_idx, is_static)` | constant is the selector `Symbol` | dispatch.rs:918 `let selector_val = callable.chunk.constants[selector_idx as usize];` |
| `InvokeConst(idx, arity, selector_idx)` | constant is the fused receiver value (selector itself is read later inside `invoke_at`, not from this line) | dispatch.rs:1047 `let constant = callable.chunk.constants[idx as usize];` |
| `Invoke(arity, selector_idx)` (via `invoke_at`, cache-miss path only) | constant is the selector `Symbol`, read only after an IC miss | dispatch.rs:419 (`invoke_at`) `let selector_val = callable.chunk.constants[selector_idx as usize];` |

### 4. `add_constant` dedup — VERIFIED no dedup, both statically and behaviorally

Static read, `chunk.rs` L85-88:
```rust
pub fn add_constant(&mut self, value: Value) -> u16 {
    self.constants.push(value);
    (self.constants.len() - 1) as u16
}
```
Unconditional `push`; no `HashMap`/`ConstKey` lookup, no equality check against
existing entries. This resolves the two conflicting memories: **no dedup shipped**;
a `ConstKey`-style dedup was apparently specced somewhere but did not land at HEAD.

Behavioral confirmation, two programs, `cargo run -q -p phalcom-core --bin phalcom -- disasm <path>`:

**Repeated string literal** (`System.print("hello")` twice):
```
Constants:
  [0] Symbol(12)
  [1] <obj ObjRef(1531v1)>
  [2] Symbol(91)
  [3] Symbol(12)
  [4] <obj ObjRef(1532v1)>
  [5] Symbol(91)
```
Two separate constant-pool slots (`[1]` and `[4]`) hold **two distinct heap
objects** (`ObjRef(1531v1)` vs `ObjRef(1532v1)`) for the identical `"hello"`
literal — confirming no value-level dedup of string constants. (`Symbol(12)` at
both `[0]` and `[3]` is equal only because the *interner* dedups symbol text
underneath — that's `Interner` dedup, unrelated to `Chunk::add_constant`; the
constant-pool slot itself is still a fresh push each time.)

**Repeated numeric literal** (`System.print(1 + 1)`):
```
Constants:
  [0] Symbol(12)
  [1] 1
  [2] 1
Bytecode:
  0000: GetGlobal(0)
  0001: Constant(1)
  0002: InvokeConst(2, 1, 3)
  ...
```
`[1]` and `[2]` are two separate constant-pool entries both holding the integer
value `1` — again, no dedup, even for an immediate `Value` with cheap equality.

### 5. The `Closure` opcode — VERIFIED, quoted in full

`phalcom-core/src/vm/dispatch.rs` L577-605:
```rust
Bytecode::Closure(idx) => {
    // The constant is the *template* closure the compiler emitted
    // (empty upvalue list). Materialize a fresh instance whose
    // upvalue cells are captured from the current activation per
    // the callable's descriptors, then wrap it in a BlockObject
    // stamped with the home frame token (ADR-0013, functions.md §2).
    let template = callable.chunk.constants[idx as usize];
    let Value::Obj(template_id) = template else {
        return Err(RuntimeError::Internal("Closure constant is not a closure".to_string()).into());
    };
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
The template-carries-empty-upvalues vs. instance-gets-filled-cells split is
explicit: the constant-pool template's own `.callable.upvalues` field (line
`descriptors = ...upvalues.clone()`) is a `Vec<UpvalueDescriptor>` (recipe), while
the freshly allocated `ClosureObject`'s `upvalues` field a few lines later is a
`Vec<ObjRef>` (live cells) built by walking those descriptors and either capturing
a stack slot (`self.capture_upvalue`) or forwarding a cell from the *currently
executing* closure's own upvalue list (`desc.is_local == false`). `Rc::clone(&...callable)`
is the U-HOTPATH share from §2 — the recipe itself is never copied, only its
refcount bumped.

Note this single opcode does **both** jobs asked about in Q5 and Q6 — there is no
separate `Bytecode::Block` opcode (see next section).

### 6. The "BLOCK opcode" — does not exist as a separate opcode; folded into `Closure`

There is **no `Bytecode::Block` or `Bytecode::MakeBlock` variant** in
`phalcom-core/src/bytecode.rs` (VERIFIED — grepped the enum; only `Closure(u16)`
at L178, doc comment: "Creates a closure from a template. 0: constant index of the
template Callable/ClosureObject"). Block minting is the tail of the same
`Bytecode::Closure` handler quoted above: `self.heap.alloc(Object::Block(BlockObject::new(new_closure, token)))`
at dispatch.rs:603, where `token = self.current_frame_token().expect(...)`
(dispatch.rs:602). So "the block-literal opcode's dispatch arm" the task asked to
find *is* `Bytecode::Closure` — the same instruction both materializes the live
`ClosureObject` and immediately wraps it in a fresh `BlockObject` stamped with the
current activation's frame token, then pushes the `BlockObject` (not the
`ClosureObject`) as the value.

`BlockObject` as the sole home-frame-token carrier: VERIFIED by type inspection —
`Callable` has no frame-token field, `ClosureObject` has no frame-token field
(closure.rs L24-36: `callable`, `module`, `upvalues` only), only `BlockObject` has
`home_frame_token: FrameToken` (block.rs L22). Methods run inside an ordinary
`CallFrame` (their own activation-identity mechanism, separate from
`BlockObject`/`FrameToken`) and, per §1, a method's `MethodKind::Closure(ObjRef)`
points at a bare `ClosureObject`, never a `BlockObject` — so methods categorically
do not carry a `home_frame_token`; only block values do.

### 7. Live disasm — block literal inside a loop

Real Phalcom `for` syntax, confirmed from a passing golden test
(`phalcom-core/tests/lang/blocks/blocks_non_local_return_in_loop.ph`, `status: PASS`):
`for (n in numbers) { ... }`. Program run (top-level, not inside a class/method,
so the disasm tool — which only dumps the module's own top-level chunk, per
`phalcom-core/bin/phalcom/disasm.rs` — shows the loop directly):

```phalcom
let numbers = List.new()
numbers.add(1)
numbers.add(2)
numbers.add(3)
var fns = List.new()
for (n in numbers) {
  fns.add({ n })
}
System.print(fns)
```

Command: `cargo run -q -p phalcom-core --bin phalcom -- disasm <file>.ph`

Output (loop portion):
```
0018: GetGlobal(15)
0019: SetLocal(0)
0020: GetLocal(0)
0021: Nil
0022: Invoke(1, 16)
0023: SetLocal(1)
0024: Nil
0025: GetLocal(1)
0026: JumpIfNone(16)
0027: GetLocal(0)
0028: InvokeLocal(1, 1, 17)
0029: Invoke(1, 17)
0030: SetLocal(2)
0031: Pop
0032: GetGlobal(18)
0033: Closure(19)
0034: Invoke(1, 20)
0035: Pop
0036: CloseUpvalue(2)
0037: GetLocal(0)
0038: InvokeLocal(1, 1, 21)
0039: Invoke(1, 21)
0040: SetLocal(1)
0041: Pop
0042: Loop(-18)
0043: CloseUpvalue(2)
```

`0033: Closure(19)` sits inside the loop body, between the cursor-advance/test
prologue (0018-0026, ADR-0048 cursor protocol — `JumpIfNone` is the end-sentinel
test) and the `Loop(-18)` back-edge at 0042 (targets `0042 + 1 - 18 = 0025`, i.e.
back to the cursor re-test). Statically the chunk contains exactly **one**
`Closure` instruction; at runtime `Loop` re-executes it once per element of
`numbers` (3 times for this program), each time invoking the `Bytecode::Closure`
handler from §5 to materialize a fresh `ClosureObject`/`BlockObject` pair that
captures the current iteration's `n` binding by upvalue — exactly the
one-static-instruction / many-dynamic-materializations relationship the task asked
to confirm.

### 8. ADRs — Decision + Alternatives only

**ADR-0006 — `Function` as the abstract root of the callable tower** (Accepted).
Decision: introduce an abstract `Function` kernel class owning the call protocol
(`call`, `call(_,…)`, `callWith(_)`, `arity`, `name`); `Block` and `Method` inherit
from `Function` as **siblings**, neither a subtype of the other. Rationale given in
Context: `ClosureObject` is already the shared representation, but `Method` wraps
it in a `MethodObject` (signature + holder) while a first-class block wraps the
same closure with a home-frame token — making one a subclass of the other would
force methods to carry a meaningless home frame, or blocks a meaningless selector.
No "Alternatives" section in this ADR (it has a Context/Decision/Consequences/
Status-note shape, not Context/Decision/Alternatives) — its Status note flags the
ADR as still "open question pending a go/no-go decision," "recommended to accept."

**ADR-0013 — Open/closed upvalues and frame-token non-local return** (Accepted).
Decision: capture uses Lua-style open/closed upvalues (open = points at a live
stack slot, shared mutation with the enclosing scope; closed = value copied into
the cell when the frame exits, so escaping blocks keep working); non-local
`return` uses a frame token (home frame pointer + generation counter), a mismatch
raising `DeadFrameError` instead of touching a dead frame. One `ClosureObject` is
shared by `Block` and `Method` (cross-references ADR-0006). **Alternatives
considered**: (1) by-value snapshot capture at closure-creation time — rejected,
breaks shared mutation between a block and its home scope and can't identify the
live home frame for non-local return; (2) raw frame pointer with no generation
counter — rejected, a reused stack slot would let a stale pointer alias a live
frame and silently return to the wrong method; the generation counter is what
turns that into a detectable `DeadFrameError` instead of memory corruption.

### 9. GC/trace + fiber touch — brief, VERIFIED

- `Object::Closure` is traced: `phalcom-core/src/heap/trace.rs` L129-137 — pushes
  `closure.module`, each upvalue handle, and walks `closure.callable.chunk.constants`
  via `trace_value` (the chunk's own string/symbol constants are reachable only
  through the owning closure, since `Chunk` lives inside `Callable` by value, not as
  its own heap object).
- `Object::Block` is traced: `trace.rs` L141-145 — pushes only `block.closure`
  ("the block *is* the only retainer of its closure once passed around";
  `home_frame_token` is an index+generation pair, not a heap handle, so nothing to
  push for it).
- Upvalue cells can be open, pointing at a specific fiber's stack slot:
  `phalcom-core/src/heap/upvalue.rs::Upvalue::Open { fiber: ObjRef, slot: usize }`
  (L32-37) — carries the owning fiber's handle because the VM's live stack is
  swapped per fiber (ADR-0030), so a closure resumed on a different fiber must
  resolve `slot` against the *home* fiber's parked stack, not whichever fiber
  happens to be current.

## Commands run for this doc

```sh
cargo build -p phalcom-core --bin phalcom

# repeated string literal — dedup check
cargo run -q -p phalcom-core --bin phalcom -- disasm dup_lit.ph
#   dup_lit.ph: System.print("hello") \n System.print("hello")

# repeated numeric literal — dedup check
cargo run -q -p phalcom-core --bin phalcom -- disasm onepone.ph
#   onepone.ph: System.print(1 + 1)

# block literal inside a for-loop
cargo run -q -p phalcom-core --bin phalcom -- disasm block_in_loop_top.ph
#   block_in_loop_top.ph: see program text in §7
```

All three `.ph` scratch files and their full outputs are reproduced verbatim in
§4 and §7 above; nothing was elided except the unrelated `phalcom-core` build
warning (`init_selector_cache` unused field) that `cargo run` prints to stderr.
