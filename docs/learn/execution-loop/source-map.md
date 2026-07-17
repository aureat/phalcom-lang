# The VM execution loop — source map at HEAD

Read-only investigation. Claims below are VERIFIED (source read directly, or a
real `.ph` program compiled/run and its output inspected) unless marked
INFERRED. Grounded via `graphify query`/`explain`/`affected` first, then
targeted reads — no tree sweep. Line numbers are current as of this read;
symbols are the stable anchor per repo convention. Commit context: `main` at
the point of this read (see `git log` — `5b06736` is HEAD's parent chain at
investigation time; not independently pinned to a SHA here).

Entry symbols (via `graphify query`/`explain`/`affected`, per the mandated
tool order): `Bytecode` enum @ `phalcom-core/src/bytecode.rs:48`, `Chunk` @
`phalcom-core/src/chunk.rs:44`, `VM` (impl block) @
`phalcom-core/src/vm/dispatch.rs:21`, `.run_until_inner()` @
`phalcom-core/src/vm/dispatch.rs:477`, reverse-affected by `graphify affected
".run_until_inner()"` → `.run_until()` @ `:221` and `.run()` @ `:204`.

---

## THE HEADLINE QUESTION — answered first

**Representation: a TYPED-ENUM VECTOR, `Vec<Bytecode>` — not a byte array.**
**Dispatch: a plain Rust `match` on the enum discriminant — not threaded, not computed-goto.**

The settling line, `phalcom-core/src/chunk.rs::Chunk` @ L45:

```rust
pub struct Chunk {
    pub code: Vec<Bytecode>,
```

`Bytecode` (`phalcom-core/src/bytecode.rs::Bytecode` @ L48) is a Rust `enum`
whose variants carry their operands **as enum payload fields**, not as
trailing bytes:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bytecode {
    Constant(u16),
    ...
    Invoke(u8, u16),
    SuperSend(u8, u16, u16),
    ...
}
```

The fetch, `phalcom-core/src/vm/dispatch.rs::VM::run_until_inner` @ L544:

```rust
let opcode = callable.chunk.code[ip];
```

One indexed `Vec<Bytecode>` load pulls discriminant *and* operand(s) together
— there is no second read for an operand, because a `Bytecode` value already
*is* the whole instruction (discriminant + payload), sized to the widest
variant. `SuperSend(u8, u16, u16)` is that widest variant (confirmed by
reading every variant in the enum, `bytecode.rs:48-355`).

The dispatch, `dispatch.rs::VM::run_until_inner` @ L570:

```rust
match opcode {
    Bytecode::Constant(idx) => { ... }
    ...
}
```

A bare Rust `match` on the 37-variant enum (`Bytecode::VARIANTS = 37`,
`bytecode.rs:360`). Rust lowers an exhaustive, dense enum match like this to
a single jump table behind **one** indirect branch — not per-opcode threaded
dispatch (labels-as-values / computed goto), not a `switch`-on-byte-then-
`READ_BYTE()`-for-operand two-step. This is exactly what the design note
`docs/design-notes/bytecode-representation-and-borrowed-techniques.md` §B1/§B3
states and is quoted verbatim in §8 below.

**Do not infer a bytestream from the word "bytecode."** Phalcom is
architecturally a stack machine ("bytecode VM" in the conventional sense),
but its *representation* is a name-typed enum vector. The `Bytecode::Jump`
variant's own doc comment (`bytecode.rs:196-201`) says this explicitly, in
the source itself, independent of the design note:

> "a [`Chunk`] is a `Vec<Bytecode>`, not a byte stream, so there is no
> fixed-width encoding to economize"

---

## 1. The halt condition — a frame-stack drain, never an end-of-code check

`dispatch.rs::VM::run_until_inner` @ L490-497:

```rust
loop {
    if self.frames.len() <= base_frames {
        // A drained frame stack with nothing left to yield means the
        // top-level program (or a block activation) fell off its end:
        // surface that absence as `None`, never the private sentinel.
        let result = self.stack.pop().unwrap_or(Value::Nil);
        return Ok(self.surface_absence(result));
    }
```

VERIFIED: this is the *only* loop-exit test besides an `Err` propagating out
through `?`. There is no `ip`-vs-`code.len()` comparison anywhere in the
loop body — I read the full ~730-line loop (`dispatch.rs:490-1199`) and the
only places `ip` is compared against anything are the per-opcode bounds
checks inside individual arms (e.g. `GetLocal`'s `local_idx < self.stack.len()`),
never against `code.len()`.

`VM::run` @ `dispatch.rs:204`:

```rust
pub fn run(&mut self) -> PhResult<Value> {
    self.run_until(0)
}
```

`run()` is exactly `run_until(0)` — confirmed literally, one line.

`VM::run_until` @ `dispatch.rs:221` is **not** a thin wrapper — it is a
fiber-aware driver (detailed in §9), but its early-return path for a
non-zero base is exactly the re-entrant single-activation case the question
asks about, `dispatch.rs:234-236`:

```rust
if base_frames != 0 {
    return self.run_until_inner(base_frames);
}
```

Its own doc comment (`dispatch.rs:208-215`) states the intended non-zero use:

> "With `base_frames == 0` this is the top-level driver ([`Self::run`]).
> Block application ([`crate::primitive::block::block_call`]) uses a
> non-zero base to run a single block activation re-entrantly and recover
> its result without draining the caller's frames."

I did not open `primitive/block.rs::block_call` itself (out of scope for
this doc), but I traced a second, independently-read re-entrant call site
that confirms the pattern empirically: `VM::import_module`
(`phalcom-core/src/interpret.rs:265-272`):

```rust
let base_frames = self.frames.len();
let stack_offset = self.stack.len();
let frame = self.new_call_frame(closure, CallContext::Module { module: module_id }, 0, stack_offset, None);
self.frames.push(frame);
self.native_reentry_depth += 1;
let result = self.run_until(base_frames);
self.native_reentry_depth -= 1;
result?;
```

A fresh frame is pushed on top of the caller's live frames, and
`run_until(base_frames)` is called with the *pre-push* frame count as the
floor — so the loop runs exactly one activation (the imported module's top
level) and returns the instant that one frame pops, leaving the caller's own
frames untouched. Same shape `block_call` is documented to use.

**Confirmed:** `run()` = `run_until(0)`; a non-zero `base_frames` drains
exactly one re-entrant activation without disturbing the caller's frames.

---

## 2. The entry chain — how a source string reaches the loop

All four hops read directly in `phalcom-core/src/interpret.rs` and
`phalcom-core/src/vm/dispatch.rs`:

1. **`interpret.rs::VM::interpret_source`** @ L186 — top-level driver called
   by the CLI/REPL. Stores the source `Arc` on the module, then:
   ```rust
   let closure = self.compile_closure(module, source)...?;
   self.run_in_module(module, closure)...?;
   ```
2. **`interpret.rs::VM::compile_closure`** @ L142 — parses (`parse_source`)
   and compiles (`Compiler::new(self, module).compile(program)`) into a
   top-level `ObjRef` closure. No VM execution happens here.
3. **`interpret.rs::VM::run_in_module`** @ L163 — installs the module,
   **clears both stacks** (`self.frames.clear(); self.stack.clear();`,
   L167-168), then **pushes frame 0**:
   ```rust
   let mut frame = CallFrame::new(closure, CallContext::Module { module }, 0, 0, None);
   frame.generation = self.next_frame_generation;
   self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
   self.frames.push(frame);
   self.run()?;
   ```
   So yes to both recon questions: it clears the stacks (this is the
   *outermost* entry — `import_module`, by contrast, deliberately does
   **not** clear, since it runs mid-execution of the importer, per its own
   doc comment L213-220), and it does push frame 0 before calling `run()`.
4. **`dispatch.rs::VM::run`** @ L204 → **`dispatch.rs::VM::run_until`** @
   L221 → **`dispatch.rs::VM::run_until_inner`** @ L477 (§0/§1 above).

Chain: `interpret_source:186` → `compile_closure:142` → `run_in_module:163`
(clears stacks, pushes frame 0) → `run:204` → `run_until:221` →
`run_until_inner:477`.

---

## 3. The fetch + "free decode" claim

Already quoted in full in §0. Restated with the exact anchors:

- Fetch: `dispatch.rs::VM::run_until_inner` @ L544 — `let opcode = callable.chunk.code[ip];`
- Dispatch: `dispatch.rs::VM::run_until_inner` @ L570 — `match opcode {`

VERIFIED: there is no width-parse, no `READ_BYTE()`-equivalent, no separate
operand fetch anywhere between the fetch line and the `match`. The `ip`
variable indexes `Bytecode` *elements*, not bytes — `Chunk::fuse_superinstructions`'s
own doc comment (`chunk.rs:90-115`) is explicit that fused opcodes "advance
`ip` by 2" (instruction count), never a byte count.

Design note, `docs/design-notes/bytecode-representation-and-borrowed-techniques.md`
§B1 ("`Vec<Bytecode>` is not a bytestream, and the difference eats a class of
techniques"), quoted (its own wording, not paraphrased):

> "One indexed load pulls **the whole instruction** — discriminant *and*
> operand — into registers, sized by the widest variant (`SuperSend`, ~8
> bytes). `match opcode` ... switches on the discriminant; the operand is
> already in a register from the same load."
>
> "**The precondition for operand-folding is a separate operand fetch.** We
> do not have one. There is no second read for `GetLocal0` to delete — the
> load it would remove is the load that delivered the opcode itself, and
> that one is not skippable."

§B3 of the same note, on the dispatch-technique axis directly:

> "Rust's `match` lowers to a single jump table behind **one** indirect
> branch... Reaching for the predictor win means **threaded dispatch** (a
> `&&label`-style or tail-call dispatcher), which is a distinct, invasive
> change with its own risk profile — not a side effect of adding opcode
> variants."

So: `match` gives Phalcom one indirect branch (standard `switch`-style
dispatch). Direct/token threading and computed-goto are real, named,
*rejected-by-precondition* alternatives — not live code paths at HEAD.

---

## 4. The hoisted callable + its guard

`dispatch.rs::VM::run_until_inner` @ L478-536, quoted in full (this is the
"marked lie" region — the loop does not "just read the chunk"; it hoists a
cached `Rc<Callable>` across iterations):

```rust
let mut hoisted: Option<(ObjRef, Rc<Callable>)> = None;
loop {
    if self.frames.len() <= base_frames { ... }

    // Safepoint (memory-management.md §4): the *only* place collection runs. ...
    self.service_gc_safepoint();

    let frame = *self.frames.last().unwrap();
    let closure_id = frame.closure;
    let ip = frame.ip;
    let stack_offset = frame.stack_offset;

    // One compare replaces a SlotMap lookup (bounds + generation + enum
    // match) plus an `Rc` deref, per instruction (F14 S1a). The `Rc::clone`
    // is per *frame change*, not per opcode — a refcount bump on call and
    // return only, which is why this does not re-pay cut 004's per-instruction
    // `Rc` hop.
    //
    // Guarding on `closure_id` alone is sound **because only the callable is
    // hoisted, never `ip`**. A chunk is a property of the closure, so any
    // path that reaches this point with the same `closure_id` — including a
    // fiber switch, which swaps `self.frames` wholesale — is entitled to the
    // same chunk. `ip` and `stack_offset` are still read from the live frame
    // every iteration, so a switch into the *same* closure at a *different*
    // `ip` reloads the correct `ip` and executes correctly.
    //
    // **Do not extend this guard to cover a hoisted `ip`.** That is the
    // stale-across-fiber-switch bug U-HOTPATH §4 names as the one this unit
    // could ship: two fibers in the same closure at different `ip`s compare
    // equal here. Hoisting `ip` needs a frame-identity guard, not this one.
    let callable = match &hoisted {
        Some((id, callable)) if *id == closure_id => callable,
        _ => {
            let callable = Rc::clone(&self.heap.closure(closure_id).callable);
            &hoisted.insert((closure_id, callable)).1
        }
    };
```

**CONFIRMED exactly as the task specified**: the guard compares `closure_id`
only (`*id == closure_id`) and explicitly does **not** compare `ip`. The
comment's stated reason is precisely the fiber-touch beat: a fiber switch
swaps `self.frames` wholesale, so any frame reached here with a matching
`closure_id` is guaranteed to own the same `Chunk` (a `Callable`/`Chunk` is
a property of the *closure*, not of which fiber is executing it) — but `ip`
and `stack_offset` are read fresh from the live frame on every iteration
regardless of the hoist, so a same-closure/different-`ip` situation (two
fibers paused inside the same closure) still executes at the right `ip`.
Hoisting `ip` itself would break this and is explicitly flagged as the
mistake U-HOTPATH's own design doc names.

---

## 5. The GC safepoint

`dispatch.rs::VM::run_until_inner` @ L505, the single call site:

```rust
self.service_gc_safepoint();
```

Definition, `phalcom-core/src/vm/gc.rs::VM::service_gc_safepoint` @ L152-156:

```rust
/// Services a latched `gc_pending` — **safepoint only**.
///
/// Call this exclusively from the dispatch-loop back-edge, where `VM::stack`/
/// `frames` are the complete root truth. Never from `Heap::alloc` (Invariant L,
/// memory-management.md §4), and never mid-opcode: several opcodes have a window
/// where a value is popped or `split_off` the stack and held only in a Rust local.
pub(crate) fn service_gc_safepoint(&mut self) {
    if self.heap.gc_pending() {
        self.force_gc();
    }
}
```

**VERIFIED it is the sole automatic collection point**: `grep -rn
"service_gc_safepoint" phalcom-core/src/` returns exactly two hits — its own
definition (`gc.rs:152`) and the one call site (`dispatch.rs:505`). No other
call.

**VERIFIED `Heap::alloc` never collects — it only latches**,
`phalcom-core/src/heap/mod.rs::Heap::insert` @ L127-132 (called by `alloc`,
`alloc_class`, `alloc_string`, `alloc_list`, etc.):

```rust
fn insert(&mut self, object: Object) -> ObjRef {
    let id = self.objects.insert(object);
    if self.objects.len() >= self.next_gc {
        self.gc_pending = true; // LATCH ONLY — never collect here (Invariant L)
    }
    id
}
```

The comment names Invariant L explicitly, and the code matches it: growth
over `next_gc` sets a boolean flag; the actual sweep happens only later,
when `service_gc_safepoint` next runs at the loop's back-edge.

**One more manual trigger exists**, `grep -rn "force_gc(" phalcom-core/`
found one call outside `gc.rs` itself:
`phalcom-core/src/primitive/system.rs::system_gc` @ L93 (the `System.gc`
native primitive) calls `vm.force_gc()` directly, unconditionally, bypassing
the `gc_pending` latch. Its own doc comment (`system.rs:77-84`) justifies
this as still safe for the same underlying reason as the safepoint: "a
primitive runs at a dispatch safepoint by construction" — i.e. a native
primitive call happens synchronously from inside an `Invoke` arm with
`stack`/`frames` already coherent, so an explicit user-triggered `System.gc`
is not a violation of Invariant L, just a second, deliberate, non-latched
door onto the same underlying `force_gc`. The *automatic*, latch-driven path
still has exactly one door: the loop's own back-edge.

Why here (the comment on the loop, `dispatch.rs:499-504`):

> "Safepoint (memory-management.md §4): the *only* place collection runs.
> Here `stack`/`frames` are coherent — no opcode is mid-flight with a value
> popped into a Rust local. Servicing before reading `frame` is deliberate:
> a non-moving collector cannot invalidate the `CallFrame` we are about to
> copy, but keeping the whole read-decode-execute sequence GC-free is what
> makes that independent of the collector's future shape."

---

## 6. The opcode tour — complete map

From `grep -n "Bytecode::" phalcom-core/src/vm/dispatch.rs | grep "=>"` — 37
arms, matching `Bytecode::VARIANTS = 37` (`bytecode.rs:360`) exactly. Owner
column follows the task's stated mapping.

| Opcode | `dispatch.rs` L | Gloss | Owning doc |
|---|---|---|---|
| `Constant(idx)` | 571 | push constant-pool value | this doc (stack basics) |
| `Closure(idx)` | 577 | materialize closure from template, capture upvalues | upvalues doc |
| `Nil` | 617 | push `None` singleton (surface absence, never raw sentinel) | this doc (stack basics) |
| `True` | 618 | push boolean `true` | this doc (stack basics) |
| `False` | 619 | push boolean `false` | this doc (stack basics) |
| `Pop` | 620 | discard stack top | this doc (stack basics) |
| `DefineGlobal(idx)` | 623 | bind name in current module | compiler/globals |
| `GetGlobal(idx)` | 632 | read module/core global, IC-cached | compiler/globals |
| `SetGlobal(idx)` | 685 | write a module global | compiler/globals |
| `GetLocal(slot)` | 720 | read local stack slot, surface absence | frames (Doc 3) |
| `SetLocal(slot)` | 731 | write local stack slot | frames (Doc 3) |
| `Class(idx)` | 740 | create class, or reopen an existing one | compiler/globals |
| `Import(idx)` | 794 | resolve + run an imported module | compiler/globals |
| `MakeFamily(name_idx)` | 805 | build a bound `::` method-reference value | compiler/globals |
| `FinalizeClass` | 856 | rebuild class's flattened `base_names` index | compiler/globals |
| `SuperSend(argc, sel, defining)` | 863 | send above a statically-known class (`super.sel`) | send (Doc 4) |
| `Method(sel, is_static)` | 917 | attach a method object to a class | send (Doc 4) |
| `GetSelf` | 941 | push receiver at frame's stack offset | frames (Doc 3) |
| `GetField(slot)` | 945 | read instance/class field slot | frames (Doc 3) |
| `SetField(slot)` | 962 | write instance/class field slot | frames (Doc 3) |
| `NewInstance` | 990 | allocate instance (or reuse for super-construct) | frames (Doc 3) |
| `Dup` | 1015 | duplicate stack top | this doc (stack basics) |
| `WrapSome` | 1019 | wrap popped value in a fresh `Some` | this doc (stack basics) |
| `Invoke(arity, sel)` | 1024 | dynamic send via `invoke_at` (IC probe → lookup → dNU) | send (Doc 4); cache: Doc 5 |
| `InvokeLocal(slot, arity, sel)` | 1036 | fused `GetLocal`+`Invoke`, one dispatch | send (Doc 4); fusion: Doc 5 |
| `InvokeConst(idx, arity, sel)` | 1046 | fused `Constant`+`Invoke`, one dispatch | send (Doc 4); fusion: Doc 5 |
| `GetUpvalue(idx)` | 1052 | read a captured upvalue cell (open or closed) | upvalues doc |
| `SetUpvalue(idx)` | 1071 | write a captured upvalue cell | upvalues doc |
| `CloseUpvalue(slot)` | 1088 | heap-promote an open upvalue at/above a slot | upvalues doc |
| `Return` | 1093 | pop frame, close upvalues, yield or continue | frames (Doc 3) |
| `ReturnNonLocal` | 1110 | unwind eagerly to a block's home frame | frames (Doc 3) |
| `Jump(offset)` | 1162 | unconditional relative jump | this doc (control flow) |
| `JumpIfFalse(offset)` | 1163 | pop `Bool`, branch if false (type-checked) | this doc (control flow) |
| `JumpIfNone(offset)` | 1177 | pop cursor, branch on identity to `None` | this doc (control flow) |
| `Loop(offset)` | 1183 | backward relative jump (disassembly-only distinction from `Jump`) | this doc (control flow) |
| `GuardBool(offset)` | 1184 | deopt guard for inlined `Bool` sacred sends | Doc 5 (guards/caches) |
| `GuardBlock(offset)` | 1191 | deopt guard for inlined `whileTrue` | Doc 5 (guards/caches) |

37/37 arms accounted for. `Loop` and `Jump` share one handler function
(`apply_jump_offset`) — VERIFIED, both `Bytecode::Jump(offset) =>
self.apply_jump_offset(offset)` (L1162) and `Bytecode::Loop(offset) =>
self.apply_jump_offset(offset)` (L1183) call the identical private method;
`Loop` exists as a distinct opcode purely so disassembly reads as a loop
back-edge (`bytecode.rs:222-226`'s own doc comment says exactly this).

---

## 7. Run it live — the strongest evidence

**Build**, exact command:
```
cargo build -p phalcom-core --bin phalcom
```
(already up to date at investigation time — no source changes were made).

**Disassembler location**: `phalcom-core/bin/phalcom/disasm.rs::disassemble_source`,
quoted in full (21 lines total):

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

This *is* the flat-typed-array claim made literal: it iterates
`chunk.code.iter()` — a `Vec<Bytecode>` — and prints each element's `Debug`
representation directly. There is no byte-width parse anywhere in this
function; each loop iteration is one `Bytecode` value.

**CLI flag**: the `phalcom disasm` subcommand (`phalcom-core/bin/phalcom/cli.rs::DisasmArgs`,
`Commands::Disasm`), taking either a path or `-s`/`--source <source>` (its
own `--help` output, captured live, confirmed the short flag is `-s`, not
`-i` — the top-level `phalcom -i <source>` run path uses `-i`, but the
`disasm` subcommand's own `Args` struct auto-derives `-s` from the field name
`source`; I initially guessed `-i` from analogy and it errored, corrected by
running `--help`).

**Command run** (arithmetic precedence, `1 + 2 * 3`):
```
$ cat /tmp/.../tiny.ph
1 + 2 * 3
$ cargo run -q -p phalcom-core --bin phalcom -- disasm /tmp/.../tiny.ph
```

**Actual output** (warnings elided):
```
Constants:
  [0] 1
  [1] 2
  [2] 3
  [3] Symbol(59)
  [4] Symbol(57)

Bytecode:
  0000: Constant(0)
  0001: Constant(1)
  0002: InvokeConst(2, 1, 3)
  0003: Invoke(1, 3)
  0004: Invoke(1, 4)
  0005: Return
```

**Reading it** (VERIFIED by re-deriving the stack trace from the opcode
semantics read in §6, cross-checked against the arithmetic result):

| ip | instr | stack after | note |
|---|---|---|---|
| 0000 | `Constant(0)` | `[1]` | push `1` |
| 0001 | `Constant(1)` | `[1, 2]` | push `2` |
| 0002 | `InvokeConst(2, 1, 3)` | `[1, 6]` | fused `Constant(2)`+`Invoke(1,3)`: pushes `3`, sends selector `[3]` to receiver `2` (the value just below), arity 1 → `2 * 3 = 6` |
| 0003 | `Invoke(1, 3)` | `[1, 6]` | **dead code** — the original `Invoke` the fusion left behind at `ip+1`; never executed (the fused arm advances `ip` by 2, skipping it) |
| 0004 | `Invoke(1, 4)` | `[7]` | sends selector `[4]` to receiver `1`, arity 1, arg `6` → `1 + 6 = 7` |
| 0005 | `Return` | — | pop `7`, yield it |

This is a real, unplanned confirmation of `Chunk::fuse_superinstructions`'s
own doc comment (`chunk.rs:99-101`, read in an earlier step): "the original
`Invoke` is left in place at `p + 1` as dead code... every jump offset in
the chunk stays correct." Instruction `0003` is exactly that dead `Invoke`,
sitting inert in the flat array, visible in the disassembly, never reached
by control flow. (INFERRED: that selector constant `[3]` is `*` and `[4]` is
`+` — derived from the arithmetic result matching `1 + 2*3 = 7`, not from an
independently dumped symbol-name table. The mechanism claim — one flat
`Vec<Bytecode>`, fusion leaves dead code in place — is VERIFIED directly
from the listing regardless of what the selectors resolve to.)

**Second run**, showing `Pop` between two top-level statements (`Constant`
push/pop delta, the predict-then-check case from the brief):
```
$ printf '1\n2\n' > two_stmt.ph
$ cargo run -q -p phalcom-core --bin phalcom -- disasm two_stmt.ph
```
Output:
```
Constants:
  [0] 1
  [1] 2
Bytecode:
  0000: Constant(0)
  0001: Pop
  0002: Constant(1)
  0003: Return
```
Stack trace: `0000 Constant(0)` → `[1]`; `0001 Pop` → `[]` (the first
statement's value is a discarded expression-statement result); `0002
Constant(1)` → `[2]`; `0003 Return` → pops and yields `2`. This is the
`Constant`/`Pop` delta directly, from real output.

**`Dup` was not captured live.** The only compiler emission site for
`Bytecode::Dup` is `phalcom-core/src/compiler/lib/class_decl.rs:532`, inside
the underscore-prefixed-getter field-default-initializer path (a specific
class-body construct). A quick attempt (`class Point { var _x = 5 }`) did
**not** hit that path — it produced `Constant/Class/FinalizeClass/
DefineGlobal/GetGlobal/Invoke/Pop/Return` with no `Dup`, meaning that surface
syntax compiles through a different route than the one at `class_decl.rs:532`
(which triggers on `ClassMember::Getter` whose name starts with `_`, a
different AST shape than a plain `var` field declaration). Not chased
further — `Dup`'s handler (`dispatch.rs:1015-1018`, `let val =
*self.stack.last()...; self.stack.push(val);`) is simple enough that its
stack effect (`[a] → [a, a]`) is read directly from source with high
confidence; only the live trace is missing, not the semantics.

---

## 8. Spec/ADR — bounded

**`docs/design-notes/bytecode-representation-and-borrowed-techniques.md`**
(dated 2026-07-14, Status: FINDINGS, not an ADR) is the load-bearing doc for
§0/§3 above. Its own framing (§B5): "Before porting a technique, name the
property of the *source* VM it exploits, then check that property in ours."
The concrete finding is that Wren-style operand-folding superinstructions
and computed-goto threading both presuppose a byte-array `ip`/`READ_BYTE()`
step that Phalcom's `Vec<Bytecode>` representation does not have — so both
techniques are foreclosed by construction, not by choice. §B4 notes one
adjacent technique that *does* survive: **fusion** (`InvokeLocal`/
`InvokeConst`, seen live in §7), which cuts dispatches rather than fetches
and so is immune to the same precondition failure.

**ADR-0051** (`docs/adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md`),
Decision section (L60-94): adopts a "measure-first, tiered,
behavior-invariant" performance strategy. Governs the hoisted-`Callable`
optimization in §4 and the fusion pass cited above (both cite perf-log cuts,
consistent with ADR-0051's "no optimization lands without a reproducible
benchmark + attributed profile + before/after number" policy). Not read
beyond the Decision section — bounded per instruction.

No dedicated ADR exists for "the execution loop" itself as a design
question — it did not turn up in a grep for `dispatch loop`/`run_until_inner`/
`hoisted callable`/`computed goto`/`threaded dispatch` across `docs/adr/` and
`docs/design-notes/` beyond the two docs above (plus incidental hits in
ADR-0030/ADR-0050/ADR-0056/the retired ADR-0033, not opened — the loop's
*existence* as a fetch-decode-execute cycle is treated in this codebase as
foundational machinery, not a deliberated fork).

---

## 9. The fiber wrapper — one honest paragraph

`run_until_inner`'s own doc comment (`dispatch.rs:467-471`) states plainly
that it is fiber-unaware:

> "The inner dispatch loop, unaware of fibers: drains bytecode until
> [`Self::frames`] shrinks to `base_frames`, or a [`RuntimeError`]
> propagates out. [`Self::run_until`] wraps this with the fiber-floor
> capture (ADR-0030 §6); this function's own behavior is otherwise exactly
> the pre-U-FIBER `run_until`."

`run_until` (`dispatch.rs:221-341`) is more than a thin wrapper — I read it
in full: at `base_frames == 0` it loops on `run_until_inner(0)`'s `Result`,
and on `Ok` it checks whether the finishing fiber has a `resumer` (§ root
fiber → drains `ready_queue` once more as a "root-drive pump" before
returning, or delivers the value across a fiber switch and loops again); on
`Err` it captures the failure into the fiber's own result and cascades it up
a resumer chain. None of that fiber machinery is this doc's job — it is
flagged here only as the forward pointer the brief asked for. The one fact
that *does* belong in this doc, because it explains a line already quoted
in §4: the hoisted-`Callable` guard in `run_until_inner` keys on `closure_id`
and deliberately **not** on `ip`, precisely because a fiber switch — which
`run_until`'s wrapping logic can trigger between `run_until_inner` calls —
swaps `self.frames` wholesale; a `closure_id` match still guarantees the
same `Chunk`, while `ip`/`stack_offset` are re-read from the live frame every
iteration regardless, so the hoist stays correct across a switch without
itself needing to know fibers exist.

---

## What was verified vs. inferred — summary

**Verified** (read the exact source, or ran the disassembler and matched
output against opcode semantics also read from source): `Bytecode` enum in
full (`bytecode.rs:48-355`) and its `index()`/`VARIANTS` bookkeeping;
`Chunk` struct (`chunk.rs:44-56`); `VM::run`/`run_until`/`run_until_inner`
in full (`dispatch.rs:204-341`, `467-1201`); the drain-only halt condition;
the hoisted-`Callable` guard and its `closure_id`-not-`ip` comment; the
single `service_gc_safepoint` call site and its definition; `Heap::insert`'s
latch-only allocation path (Invariant L); the one manual `System.gc`
override; the complete 37-arm opcode `match` and its owning-doc mapping; the
full `interpret_source → compile_closure → run_in_module → run → run_until →
run_until_inner` entry chain, including `run_in_module`'s stack-clear and
frame-0 push; `import_module`'s independent re-entrant `run_until(base_frames)`
call as empirical confirmation of the non-zero-base pattern; the
disassembler's source (`disasm.rs`, 21 lines, read in full) and CLI flag
(`-s`/`--source`, confirmed via `--help` after an initial wrong guess); two
live disassembler runs (`1 + 2 * 3` showing `InvokeConst` fusion with its
dead-code `Invoke` left in place, and a two-statement program showing the
`Constant`/`Pop` delta) matched against the fusion doc comment and opcode
semantics; the `bytecode-representation-and-borrowed-techniques.md` §B1/§B3/§B4
findings, quoted verbatim; ADR-0051's Decision section header.

**Inferred, not independently re-derived**: that disassembled selector
constants `[3]`/`[4]` in the `1 + 2 * 3` run are literally the symbols `*`
and `+` (derived from the arithmetic result, not a separately dumped
symbol-name table); `block_call`'s own re-entrant `run_until` call (cited
only via `run_until`'s doc comment, not by opening `primitive/block.rs`);
the full internal control flow of `run_until`'s fiber-floor/ready-queue pump
(read once for context, explicitly out of scope — owned by the fibers doc,
not re-derived to any depth here); why the plain `var _x = 5` class-body
syntax did not hit the `Dup`-emitting getter path at `class_decl.rs:532`
(observed as a negative result, not traced to the actual AST branch taken).

**Confirmed not to exist at HEAD**: no dedicated ADR for the execution loop
as a design question; no threaded or computed-goto dispatch anywhere in
`run_until_inner` — the `match` is the whole dispatch, confirmed by reading
every line of the loop body, not sampled.
