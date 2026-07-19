# Caches and fusion — source map (verified at HEAD)

All claims below are marked **VERIFIED** (read the line, or ran the program and
matched output) or **INFERRED**. Nothing is invented. Line numbers are current as
of this read; symbols are the durable anchor.

## THE DOMINATING QUESTION — answered first

**Global counter, or per-class epoch? Inline operand, or side table?**

- Method IC invalidation: **[global counter]**, not per-class epoch. **VERIFIED.**
  `world_version` is a single `u64` on the `VM` struct
  (`phalcom-core/src/vm/mod.rs::VM::world_version` @ L116), bumped unconditionally
  on *every* method install anywhere in the program — not scoped to the class
  being mutated, not scoped to a subtree. A method definition on class `Zebra`
  invalidates every IC in the program, including one that has never seen a
  `Zebra` instance.
- Storage: **[side table]**, not inline `Invoke` operand. **VERIFIED.**
  `Chunk::caches: Vec<Cell<Option<InlineCache>>>` (`chunk.rs::Chunk::caches` @ L50)
  is a separate array parallel to `code`, indexed by instruction position
  (`cache_ip`). `Bytecode::Invoke`'s own operand is unchanged — it still just
  carries `(arity, selector_idx)`.
- **No per-class epoch field exists anywhere in the codebase.** `ClassObject`
  (`phalcom-core/src/heap/class.rs::ClassObject` @ L25) has no `epoch`/`version`
  field — confirmed by reading the full struct (fields: `name`, `class`,
  `superclass`, `methods`, `field_slots`, `field_count`, `static_slots`,
  `base_names`, `attributes`, `attributes_frozen`; none is a version counter).
  `git log -S"world_version"` shows this landed as three small commits
  (`49e38b6` add counter + bump at all 6 sites, `d030908` add `InlineCache`
  struct, `f5e41f1` wire probe + refill), independent of and prior to the U-IC
  unit that *plans* the per-class design.

This is the load-bearing finding: **the fine-grained per-class machinery U-IC's
plan recommends (DEC-IC-A) does not exist.** What exists is a simpler, already-
shipped mechanism — a global version stamp — that the U-IC plan's own
preconditions section describes as not yet populated ("no IC is populated (only
comment stubs at `vm.rs:1578,1630`"). That precondition text is now **stale**:
an IC is populated, just not the one U-IC designs. See §4.

Quoted definitions:

```rust
// vm/mod.rs @ L110-116
/// Global version counter for inline-cache invalidation.
///
/// Incremented whenever a method is added or replaced on any class.
/// Each monomorphic inline cache records the `world_version` at the time
/// its entry was populated; if `world_version` changes, the cache entry
/// is discarded and the method lookup is re-run (ADR-FUTURE: inline cache).
pub(crate) world_version: u64,
```

```rust
// vm/dispatch.rs @ L927, L930 — Bytecode::Method arm
if is_static {
    let meta = self.heap.class(class_id).class;
    self.heap.class_mut(meta).add_method(selector, method_id);
    self.world_version += 1;
} else {
    self.heap.class_mut(class_id).add_method(selector, method_id);
    self.world_version += 1;
    // Sacred-selector override-epoch tracking (ADR-0018): ...
    self.universe.note_method_installed(class_id, selector, &self.interner);
}
```

The bump is unconditional and identical for the static and instance branches —
there is no per-class filtering anywhere near it.

## 1. Cache type definitions

`InlineCache` — `phalcom-core/src/chunk.rs::InlineCache` @ L10, quoted in full:

```rust
/// One monomorphic inline-cache slot, owned by a single `Bytecode::Invoke` site.
#[derive(Debug, Clone, Copy)]
pub struct InlineCache {
    /// Receiver class the cached resolution was recorded for.
    pub class: ClassId,
    /// The resolved `MethodObject` handle.
    pub method: crate::heap::ObjRef,
    /// `VM.world_version` at record time; a mismatch means a method was
    /// (re)defined somewhere since, and the entry must be discarded.
    pub version: u64,
}
```

Recon's claimed shape `{ class: ClassId, method: ObjRef, version: u64 }` is
**confirmed exactly**, field names and types included.

`GlobalCache` — `phalcom-core/src/chunk.rs::GlobalCache` @ L30, quoted in full:

```rust
/// One global-resolution cache slot, owned by a single `Bytecode::GetGlobal` or
/// `Bytecode::SetGlobal` site.
#[derive(Debug, Clone, Copy)]
pub struct GlobalCache {
    /// Module the name actually resolved in — the accessing closure's own module,
    /// or the core module when it resolved through the fallback.
    pub module: crate::heap::ObjRef,
    /// Slot index within that module's `globals`.
    pub slot: usize,
    /// `globals_version` **of the accessing closure's module** (not of
    /// [`Self::module`]) at record time. A mismatch means that module declared a
    /// new name since, which may shadow this resolution, so the entry is dropped.
    pub version: u64,
}
```

Recon's claimed shape `{ module, slot index, version }` is **confirmed**. One
correction to recon: the version field this cache checks is **not** a field on
`Chunk` — it is `ModuleObject::globals_version: u64`
(`phalcom-core/src/heap/module.rs` @ L62, bumped at L141 in `ModuleObject::define`),
a per-module counter on the heap object, read fresh from
`self.heap.module(module_id).globals_version` on every probe. `chunk.rs` only
carries a doc-comment referencing it (L36) — the field itself lives on the
module, not the chunk. Recon's "chunk.rs ~L36" pointed at the comment, not the
storage; worth flagging since a reader chasing "where does version live" by line
number alone would land on the wrong file.

`Chunk::caches` — `phalcom-core/src/chunk.rs::Chunk::caches` @ L50, quoted:

```rust
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

Recon's claim `Vec<Cell<Option<InlineCache>>>`, parallel to `code`, indexed by
`ip`, is **confirmed exactly**, doc-comment invariant text included verbatim
("only `Bytecode::Invoke` indices are ever non-`None`"). `Chunk::add_instruction`
(@ L77) keeps the three arrays (`spans`, `caches`, `gcaches`) growing in lockstep
with `code` on every push — `self.caches.push(Cell::new(None))` /
`self.gcaches.push(Cell::new(None))` unconditionally, regardless of opcode, which
is what lets later code index any of them by raw `ip` without a bounds
special-case.

`gcaches` and the "separate table" rationale: **confirmed** by the doc-comment
above — `Invoke` and `GetGlobal`/`SetGlobal` never share an instruction slot, so
folding both cache shapes into one `enum` variant would pay the larger variant's
size (`InlineCache` carries a `ClassId` + `ObjRef` + `u64`; `GlobalCache` carries
an `ObjRef` + `usize` + `u64`) at every site of either kind, including sites that
never use the other shape.

## 2. Method-cache hit path — `VM::invoke_at`

`phalcom-core/src/vm/dispatch.rs::VM::invoke_at` @ L398, probe + refill quoted
in full:

```rust
fn invoke_at(&mut self, callable: &Callable, cache_ip: usize, arity: u8, selector_idx: u16) -> PhResult<()> {
    let arity = arity as usize;
    let receiver_idx = self.stack.len() - 1 - arity;
    let receiver = self.stack[receiver_idx];
    let receiver_class = receiver.class(self);

    // Cache probe. `chunk` is a shared borrow; the `Cell` is what lets us
    // write back through it (U-IC §2.1). `spans[cache_ip]` rides along in the
    // same borrow (S2): the loop head no longer reads it, and this arm
    // needs it for whichever `call_method` / dNU forward it reaches.
    let (cached, source_range) = {
        let chunk = &callable.chunk;
        let cached = chunk.caches[cache_ip].get().filter(|slot| {
            slot.class == receiver_class && slot.version == self.world_version
        }).map(|slot| slot.method);
        (cached, chunk.spans[cache_ip])
    };

    if let Some(method) = cached {
        self.call_method(&receiver, method, arity, source_range)?;
    } else {
        let selector_val = callable.chunk.constants[selector_idx as usize];
        let selector_sym = selector_val.as_symbol().unwrap();

        if let Some(method) = receiver.lookup_method(self, selector_sym) {
            // Refill. Both `receiver_class` and `world_version` are read
            // AFTER the lookup on purpose — see U-IC §2.3 hazard 2.
            let entry = crate::chunk::InlineCache { class: receiver_class, method, version: self.world_version };
            callable.chunk.caches[cache_ip].set(Some(entry));
            self.call_method(&receiver, method, arity, source_range)?;
        } else {
            /* variadic probe, then doesNotUnderstand(_) forward */
        }
    }
    /* ... */
}
```

- **Probe condition** — `slot.class == receiver_class && slot.version ==
  self.world_version`, matching recon exactly.
- **Refill** — writes through the `Cell` via `callable.chunk.caches[cache_ip].set(Some(entry))`,
  which is legal on a shared `&Chunk` borrow only because `caches` is
  `Vec<Cell<...>>`.
- **THE U-IC HAZARD** — **VERIFIED, and recon's ordering claim is correct.**
  `receiver_class` is read at L402, *before* `lookup_method` is called at L422 —
  but the value that goes into the cache entry at refill time
  (`class: receiver_class`) is the value captured before the lookup, not a
  fresh read after. What *is* read after the lookup, per the comment at
  L423-424 ("Both `receiver_class` and `world_version` are read AFTER the
  lookup on purpose"), is `self.world_version` at L425 — this is the load-
  bearing ordering. If `world_version` were snapshotted *before*
  `receiver.lookup_method(...)` and a re-entrant send during that lookup (e.g.
  a `doesNotUnderstand` handler, or a getter that itself triggers a method
  install) bumped `world_version`, the cache would stamp a resolution with a
  version *older* than the world it was actually resolved in — a hit on that
  stale-stamped entry would then serve a method chosen before some
  intervening redefinition, silently. Reading `self.world_version` at refill
  time (post-lookup) closes that window: the stamped version is always the
  version the world was actually in when the entry became valid, at the
  latest possible read point. `receiver_class` itself doesn't need the same
  care here because `lookup_method` cannot change the receiver's own class
  mid-call in this object model — only cause a method table to change, which
  `world_version` alone is tasked with catching.

## 3. `world_version`

**VERIFIED single global `u64` on `VM`.** Declaration quoted above (§ dominating
question). Initialized to `0` at `vm/bootstrap.rs:39`. Bumped at exactly six
sites (`git log` commit `49e38b6`'s own message says "add world_version counter
and bump at all 6 method-install sites" — grep confirms six, no more, no
fewer):

| Site | File:line |
|---|---|
| static method install | `vm/dispatch.rs` L927 |
| instance method install | `vm/dispatch.rs` L930 |
| `primitive/mod.rs` macro (2 call sites) | `primitive/mod.rs` L117, L133 |
| `universe/primitives.rs` (2 call sites) | `universe/primitives.rs` L164, L197 |

**No per-class epoch field anywhere** — confirmed by reading the full
`ClassObject` struct (`heap/class.rs` @ L25-66); the closest thing to
per-class invalidation state is `Universe`'s four *sacred-selector pristine
flags* (§7), which are a completely different, narrower mechanism (kernel
`Bool`/`Block`/leaf-`toString` guard-opcode fast paths, not the general method
IC).

## 4. Landed-vs-planned split

`docs/forge/units/U-IC/plan.md` line 3: `Status: **PLANNED** (dispatch-ready)`.
**Confirmed** — the file is still marked planned, not landed, as of this read.

- **Change 1 (selector-only interner): did NOT land.** `Symbol` is confirmed
  still a mixed space: `phalcom-core/src/interner.rs::Symbol` @ L10 —
  `pub struct Symbol(pub(crate) u32)`, one flat `u32` id space with no
  `SelectorId` newtype or separate selector interner anywhere in the crate.
- **Change 2 (per-class epoch + design-B own-method arrays): did NOT land.**
  `phalcom-core/src/heap/class.rs::MethodsMap` @ L17 —
  `type MethodsMap = IndexMap<Symbol, ObjRef>` — still a hashmap, one per class.
  `lookup_method_in_hierarchy` (`heap/class.rs` @ L74) still chain-walks:

  ```rust
  pub fn lookup_method_in_hierarchy(heap: &Heap, mut class: ClassId, selector: Symbol) -> Option<ObjRef> {
      loop {
          let current = heap.class(class);
          if let Some(&method) = current.methods.get(&selector) {
              return Some(method);
          }
          match current.superclass {
              Some(superclass) => class = superclass,
              None => return None,
          }
      }
  }
  ```

  One `IndexMap.get` hash probe per superclass level, exactly as before U-IC's
  proposed design B (per-class own-method flat arrays) would change it. No
  per-class epoch field exists to consume (ties to §3).
- **Change 3's `LOAD_LOCAL_0..15`/`LOAD_FIELD_THIS`: did NOT land.** Grepping
  `bytecode.rs` finds no such opcodes. **But a different superinstruction DID
  land**: cut-008's `InvokeLocal`/`InvokeConst` (§5) — a different fusion shape
  (fold a preceding `GetLocal`/`Constant` into the following `Invoke`, not an
  operand-free local-load).
- **What DID land, outside the U-IC plan's own scope:** the global-`world_version`
  monomorphic method IC (§0-3, commits `49e38b6`/`d030908`/`f5e41f1`), the
  `GetGlobal`/`SetGlobal` per-site `GlobalCache` (§6, commit `39d9042`, F12),
  and the `InvokeLocal`/`InvokeConst` fusion (§5, commit `1d2baea`, cut 008) —
  three of the four DEC-IC decisions the plan poses were effectively
  pre-answered by code that shipped under different unit names (F12, cut 008)
  before/alongside the plan being written:
  - DEC-IC-A (epoch granularity): plan recommends per-class; **HEAD ships
    global**, unresolved-per-plan but resolved-in-practice by already-shipped
    code the plan's own preconditions section does not yet reflect.
  - DEC-IC-C (slot storage): plan recommends side table; **HEAD already uses a
    side table** (`Chunk::caches`/`gcaches`), matching the recommendation.
  - DEC-IC-D (mono vs poly): plan recommends monomorphic v1; **HEAD is
    monomorphic** (`InlineCache` holds exactly one `class`/`method` pair, no
    array), matching the recommendation.

| | RUNS at HEAD | U-IC PLANS |
|---|---|---|
| Selector space | Single mixed `Symbol(u32)` | Dense `SelectorId` carved out |
| Method dict | `IndexMap<Symbol,ObjRef>` per class, chain-walked | Design-B per-class own-method flat arrays |
| IC invalidation | Single global `world_version: u64`, bumped on *any* method install anywhere | Per-class epoch, bumped up the affected subtree only (DEC-IC-A) |
| IC slot storage | Side table (`Chunk::caches`/`gcaches`, `Cell`-based) | Side table (same recommendation — already matches) |
| IC arity | Monomorphic (1 slot) | Monomorphic v1, extensible to PIC |
| Superinstructions | `InvokeLocal`/`InvokeConst` (fuse a preceding load into `Invoke`) | `LOAD_LOCAL_0..15`/`LOAD_FIELD_THIS` (operand-free local/field loads) |

The practical consequence of the global-vs-per-class gap: a method definition
anywhere in the program — even on an unrelated class the current hot loop never
touches — invalidates every warm IC in the process on its next probe. This is
sound (never serves a stale method) but not the fine-grained scheme the plan
was written to build; whether the coarseness matters is a measured question
(mutation frequency in steady-state programs is low — class/method
definitions cluster at program start), not one this read attempts to settle.

## 5. Superinstruction fusion — `Chunk::fuse_superinstructions`

`phalcom-core/src/chunk.rs::Chunk::fuse_superinstructions` @ L116 and
`Chunk::branch_targets` @ L137, quoted in full:

```rust
pub fn fuse_superinstructions(&mut self) {
    let targets = self.branch_targets();
    for p in 0..self.code.len().saturating_sub(1) {
        if targets.contains(&(p + 1)) {
            continue;
        }
        let Bytecode::Invoke(arity, selector) = self.code[p + 1] else { continue };
        self.code[p] = match self.code[p] {
            Bytecode::GetLocal(slot) => Bytecode::InvokeLocal(slot, arity, selector),
            Bytecode::Constant(idx) => Bytecode::InvokeConst(idx, arity, selector),
            _ => continue,
        };
    }
}

fn branch_targets(&self) -> HashSet<usize> {
    self.code
        .iter()
        .enumerate()
        .filter_map(|(b, op)| {
            let offset = match *op {
                Bytecode::Jump(o)
                | Bytecode::JumpIfFalse(o)
                | Bytecode::JumpIfNone(o)
                | Bytecode::Loop(o)
                | Bytecode::GuardBool(o)
                | Bytecode::GuardBlock(o) => o,
                _ => return None,
            };
            usize::try_from(b as i64 + 1 + offset as i64).ok()
        })
        .collect()
}
```

- `InvokeLocal(u16, u8, u16)` / `InvokeConst(u16, u8, u16)` — **VERIFIED** in
  `bytecode.rs` @ L344/L354. Operand shapes: `InvokeLocal` = (local slot,
  arity, selector const idx); `InvokeConst` = (constant pool idx, arity,
  selector const idx). Opcode indices 35/36 (`bytecode.rs` @ L406-407),
  `Bytecode` stays `Copy`/8 bytes — no width growth, since `SuperSend(u8, u16,
  u16)` already set the ceiling (perf-log 008).
- **In-place rewrite — VERIFIED.** `self.code[p] = ...` replaces the pair's
  *first* instruction only; the `Invoke` originally at `p+1` is left
  untouched in `self.code`, now dead (nothing branches to it, nothing falls
  through to it because the fused opcode advances `ip` by 2). `code.len()`
  is never mutated by this function — no `insert`/`remove`/`truncate` call
  exists in it — so `spans`, `caches`, `gcaches` (all sized to `code.len()`
  at construction and never resized after) stay aligned with `code` with zero
  re-indexing.
- **The fused arm reads its IC/span at `ip+1` — composes with the IC.**
  **VERIFIED**, `vm/dispatch.rs` @ L1036-1050:

  ```rust
  Bytecode::InvokeLocal(slot, arity, selector_idx) => {
      let local_idx = stack_offset + slot as usize;
      /* bounds check */
      let value = self.surface_absence(self.stack[local_idx]);
      self.stack.push(value);
      self.frames.last_mut().unwrap().ip += 1;
      self.invoke_at(callable, ip + 1, arity, selector_idx)?;
  }
  Bytecode::InvokeConst(idx, arity, selector_idx) => {
      let constant = callable.chunk.constants[idx as usize];
      self.stack.push(constant);
      self.frames.last_mut().unwrap().ip += 1;
      self.invoke_at(callable, ip + 1, arity, selector_idx)?;
  }
  ```

  `cache_ip = ip + 1` is passed explicitly as `invoke_at`'s second argument —
  the dead `Invoke`'s own slot in `chunk.caches`/`chunk.spans`. A fused send
  therefore probes the *exact same* cache cell the unfused `(GetLocal |
  Constant), Invoke` pair would have, and reports the same span on error. This
  is also why `Invoke`'s own body was extracted into `invoke_at` in the first
  place (perf-log 008): all three call shapes (`Invoke`, `InvokeLocal`,
  `InvokeConst`) share one probe/refill/miss implementation, so the IC logic
  cannot drift between the fused and unfused forms.
- **Jump-target guard — VERIFIED, quoted above.** `branch_targets()` collects
  every `Jump`/`JumpIfFalse`/`JumpIfNone`/`Loop`/`GuardBool`/`GuardBlock`
  target address (conservatively, whether or not the branch is taken at
  runtime), and the fusion loop's first check (`if targets.contains(&(p + 1))
  { continue; }`) skips any pair whose `Invoke` a branch could land on
  directly — protecting the invariant that every reachable `Invoke`-shaped
  entry point still finds a real `Invoke` there. Three chunk.rs unit tests
  exercise this directly: `fuses_both_pair_shapes_in_place` (the happy path),
  `refuses_to_fuse_a_pair_whose_invoke_is_a_jump_target` (forward jump onto
  the `Invoke`), `a_backward_loop_edge_also_pins_its_target` (a `Loop` whose
  target is *not* the `Invoke`, confirming the guard isn't overly
  conservative). Per perf-log 008, this guard fires in 0 chunks across
  `core.ph`, `for.ph`, and 60 lang fixtures — unmeasured-but-covered defensive
  surface, not something the shipped corpus exercises.

## 6. Global-variable cache fast paths — `GetGlobal`/`SetGlobal`

`GetGlobal` — `vm/dispatch.rs` @ L632-683. Probe:

```rust
let version = self.heap.module(module_id).globals_version;
let cached = callable.chunk.gcaches[ip].get();
if let Some(hit) = cached
    && hit.version == version
    && let Some(value) = self.heap.module(hit.module).get_by_slot(hit.slot)
{
    /* fast path: push value, continue */
}
```

On miss: resolves in the current module first, falls back to the core module
(`CORE_MODULE_NAME`) if not found locally, then refills `gcaches[ip]` with the
resolved `(module, slot, version)`.

`SetGlobal` — `vm/dispatch.rs` @ L685-718. **Confirmed no core-module
fallback**, exactly as recon claims. The comment at L691-693 states it
directly: *"Unlike `GetGlobal`, assignment has **no** core-module fallback:
writing a kernel name the module never declared is an error, not a write into
core. So a hit here always names this module's own slot."* The code matches:
the miss path only calls `self.heap.module(module_id).slot_of(name_sym)` — no
second probe of the core module — and an unresolved slot is a hard
`RuntimeError` ("Undefined variable '...'"), not a silent core write.

Both caches key on `ModuleObject::globals_version` (`heap/module.rs` @ L62,
bumped in `ModuleObject::define` @ L141) — a **per-module** counter, separate
from `world_version` and from any other module's counter, matching
`GlobalCache::version`'s doc comment ("of the accessing closure's module").

## 7. Guard opcodes — adjacent, not core (brief)

`GuardBool`/`GuardBlock` (`vm/dispatch.rs` @ L1184-1195) and
`note_method_installed` (`universe/mod.rs` @ L188) are the ADR-0018
sacred-selector pristine-flag fast path — a narrower, separate mechanism from
the general method IC. `note_method_installed` flips one of four boolean
"pristine" flags (`bool_sacred_pristine`, `block_sacred_pristine`,
`number_tostring_pristine`, `symbol_tostring_pristine`,
`str_tostring_pristine` — five, not four, on closer count) to `false` the
instant a sacred selector (`ifTrue(_)`, etc.) is redefined directly on kernel
`Bool`/`Block`/`Number`/`Symbol`/`String`. The guard opcodes then read the flag
each time to decide whether the compiler's inlined fast path (jump opcodes
instead of a real send, per ADR-0018) is still safe to take:

```rust
Bytecode::GuardBool(offset) => {
    let top = *self.stack.last()...;
    let takes_fast_path = matches!(top, Value::Bool(_)) && self.universe.bool_sacred_pristine;
    if !takes_fast_path { self.apply_jump_offset(offset); }
}
```

This is orthogonal to `world_version`/`InlineCache` — it guards the *inliner's*
compiled-in fast path for a handful of hardcoded selectors, not general method
dispatch.

## 8. Live fixtures — observed output

All runs: `cargo run -q -p phalcom-core --bin phalcom -- -i '<source>'`, build
was clean (`cargo build -p phalcom-core --bin phalcom` — one pre-existing dead-
code warning on an unrelated field, no errors).

**(a) Method sent in a tight loop:**

```
class Widget {
  tag() { return 1 }
}
var w = Widget.new()
var i = 0
while (i < 5) {
  System.print(w.tag())
  i = i + 1
}
```

Output:
```
1
1
1
1
1
```

**(b) THE INVALIDATION PROOF — reopen + redefine, no warm-up:**

```
class Widget {
  label() { return "v1" }
}
var w = Widget.new()
System.print(w.label())
class Widget {
  label() { return "v2" }
}
System.print(w.label())
```

Output:
```
v1
v2
```

**(b′) Same proof with the IC deliberately warmed first** (20 hits before
redefinition, to rule out "it never cached because there was only one send"):

```
class Widget {
  label() { return "v1" }
}
var w = Widget.new()
var i = 0
while (i < 20) {
  System.print(w.label())
  i = i + 1
}
class Widget {
  label() { return "v2" }
}
System.print(w.label())
```

Tail of output (last 6 lines shown, 20 total `v1`s precede):
```
v1
v1
v1
v1
v1
v2
```

The 20 warm `v1` sends prove the cache was live and hit repeatedly; the
`class Widget { label() {...} }` reopen bumps `world_version` (§3, the
"instance method install" site); the very next `w.label()` returns `v2`,
proving the stamped-version comparison in `invoke_at`'s probe (§2) rejects the
stale slot and re-resolves rather than serving the cached pre-redefinition
method.

**(c) Megamorphic site — one call site, several receiver classes, same
selector:**

```
class A { tag() { return "A" } }
class B { tag() { return "B" } }
class C { tag() { return "C" } }
var items = [A.new(), B.new(), A.new(), C.new(), B.new(), A.new(), C.new()]
var i = 0
while (i < 7) {
  System.print(items[i].tag())
  i = i + 1
}
```

Output:
```
A
B
A
C
B
A
C
```

Correct results across a receiver-class-churning site, no crash. (The IC is
monomorphic — §1/§4 — so this site's single slot thrashes on every access
after the first, falling back to `lookup_method_in_hierarchy` every time; the
test confirms the fallback is *correct*, not that it is fast — this doc does
not claim a PIC exists.)

**(d) Global-variable read in a loop:**

```
var x = 42
var i = 0
while (i < 5) {
  System.print(x)
  i = i + 1
}
```

Output:
```
42
42
42
42
42
```

## 9. Bounded ADR/plan/perf-log read

- **ADR-0012** (`docs/adr/accepted/0012-selector-signature-encoding-and-dispatch.md`)
  — Decision (L47-50): *"Dispatch is built **inline-cache-ready**: each call
  site owns a monomorphic cache slot (receiver class → resolved method), keyed
  by the stable class handle... The IC *population* may be deferred, but the
  dispatch shape must not preclude it."* Consequences (L67-69): *"Inline-cache
  population is deferred. Dispatch is built IC-ready here; the cache itself is
  a speed item in the deferred register, not part of the accepted decision."*
  Alternatives considered (L79-81): populating the IC immediately was
  considered and explicitly rejected at ADR time as premature ("front-loads
  polymorphic/megamorphic bookkeeping onto dispatch that is not yet correct").
  **Confirms the seam-reservation framing**: ADR-0012 fixed the shape (a
  `ClassId`-keyed slot per call site) at zero cost; population happened later,
  piecemeal, outside this ADR's own scope.
- **ADR-0051** (performance strategy) — one line: Tier 3 of the tiered strategy
  is explicitly named "`U-IC`: selector-only interner + monomorphic inline
  cache at the `ClassId` seam + superinstructions" (L89-90), i.e. the plan
  ADR-0051 points at is the same not-yet-fully-landed U-IC unit described in
  §4.
- **ADR-0018** (sacred-selector guards) — one line: the compiler recognizes a
  sacred selector with a literal block argument at the call site and emits
  jump opcodes instead of a real send, guarded by the pristine flags §7 reads.
- **ADR-0041** (hierarchy-stability policy) — mutations requiring invalidation:
  the ADR seals `superclass` at class creation (DEC-U13a, "methods stay open"),
  and its "what must this not preclude" section names the mutation that a
  *future* mutable-superclass feature would need to handle: *"invalidate every
  dependent inline cache (reusing the ADR-0018 override epoch)"* (L79-80) —
  language that assumes an epoch-shaped invalidation mechanism, which (per §0
  above) is not what shipped; what shipped is the coarser global counter. The
  ADR also states plainly (L106-107): *"Dispatch stays exactly one hashmap
  probe. No MRO walk exists anywhere in the VM."*
- **`docs/forge/perf-log/008-fuse-invoke-pairs.md`** — status **landed**
  `1d2baea`. Measured: **~3.3 ns/dispatch**, from two independent instruments
  in agreement — a differential measurement (3.56–3.68 ns/instruction, an
  upper bound including body cost) and the fusion cut read backwards
  (3.05–3.86 ns, dispatch alone since bodies are unchanged by fusion). Result
  table (quoted exactly): `string_equals` **−8.1%**, `for` **−5.1%**,
  `variadic_send` **−4.7%**, `bare_send` **−4.2%**, `fib` **−3.9%**, plus
  `binary_trees` −3.0%, `arith_send` −1.6% (noisy, bootstrap-dominated),
  `method_call` −1.5%, `skynet` −1.8%, `map_numeric` −0.2%. The F16-verdict
  flip: F16 had deferred superinstructions on three grounds, and reason 3 —
  *"the inliner already covers the classic arithmetic win"* — turned out to be
  **false**: the inliner's sacred set (`compiler/inliner.rs`) is
  `ifTrue(_)`/`ifFalse(_)`/`ifTrue(_:ifFalse:)`/`and(_)`/`or(_)`/`whileTrue(_)`
  — control flow only, never arithmetic; `1 + 2` still compiles to
  `Constant, Constant, Invoke`. `map_numeric`'s non-result: it removed the
  *most* dispatches of any row (18,000,003) yet moved only **−0.2%**, because
  its instructions individually cost **27.6 ns** each (F17) — hashing,
  allocation, GC-heavy work dwarfing the ~3.3 ns a fusion saves; a fusion buys
  dispatch time, and only a workload whose time *is* dispatch can spend that
  saving.

## 10. Use-site tables

**`world_version`** — read/write sites (grep-verified, exhaustive in
`phalcom-core/src`):

| Site | Op | File:line |
|---|---|---|
| `invoke_at` probe | read (compare) | `vm/dispatch.rs:411` |
| `invoke_at` refill | read (stamp) | `vm/dispatch.rs:425` |
| static method install | write (+=1) | `vm/dispatch.rs:927` |
| instance method install | write (+=1) | `vm/dispatch.rs:930` |
| primitive method-install macro (×2) | write (+=1) | `primitive/mod.rs:117,133` |
| universe bootstrap installs (×2) | write (+=1) | `universe/primitives.rs:164,197` |
| VM struct field decl | decl | `vm/mod.rs:116` |
| VM bootstrap init | decl (=0) | `vm/bootstrap.rs:39` |
| GC trace (explicitly not traced — it's a plain `u64`) | pass-through | `vm/gc.rs:84` |

**`InlineCache`** — construction/consumption sites:

| Site | Op | File:line |
|---|---|---|
| struct def | decl | `chunk.rs:10` |
| `caches` probe | read | `vm/dispatch.rs:410` |
| `caches` refill | write | `vm/dispatch.rs:426` |

**`Chunk::caches`/`gcaches`** — every index-access site (grep-exhaustive):

| Field | Site | File:line |
|---|---|---|
| `caches` | probe (`.get()`) | `vm/dispatch.rs:410` |
| `caches` | refill (`.set()`) | `vm/dispatch.rs:426` |
| `gcaches` | `GetGlobal` probe | `vm/dispatch.rs:643` |
| `gcaches` | `GetGlobal` refill | `vm/dispatch.rs:671` |
| `gcaches` | `SetGlobal` probe | `vm/dispatch.rs:695` |
| `gcaches` | `SetGlobal` refill | `vm/dispatch.rs:701` |

**`Chunk::fuse_superinstructions`** — called from exactly two `Callable`
construction sites, both in `compiler/lib/mod.rs:207` and `compiler/lib/mod.rs:245`.
`graphify affected .fuse_superinstructions()` independently surfaces only its
own three unit tests (`chunk.rs:171,186,198`) as depth-2 neighbors — the
compiler call sites are one hop further (call graph edge from `mod.rs`, not
picked up at depth 2 from the function node itself), which is why the grep
above is the ground truth for this table, not the graphify traversal alone.

## Corrections to recon worth flagging

1. `GlobalCache::version`'s underlying counter (`globals_version`) lives on
   `ModuleObject` (`heap/module.rs:62`), not on `Chunk` — recon's "chunk.rs
   ~L36" pointed at a doc-comment referencing the field, not its storage.
2. `note_method_installed` guards **five** pristine flags, not a single
   "sacred selector" flag — `bool_sacred_pristine`, `block_sacred_pristine`,
   `number_tostring_pristine`, `symbol_tostring_pristine`,
   `str_tostring_pristine`. Still one mechanism, ADR-0018-scoped, correctly
   described as adjacent-not-core to this doc.
3. The U-IC plan's "Preconditions" section (`plan.md` L33: "no IC is populated
   (only comment stubs at `vm.rs:1578,1630`, `bytecode.rs:92`)") is **stale**
   relative to HEAD — those line numbers likely predate the file's split into
   `vm/dispatch.rs`/`vm/mod.rs`, and more importantly a global-counter IC *is*
   populated now (commits `49e38b6`/`d030908`/`f5e41f1`, all landed). A future
   reader of the plan should not trust that precondition section without
   re-verifying against HEAD, exactly as this doc had to.
