# Memory Management & Garbage Collection

> Normative specification of object lifetime, reachability, and reclamation in
> the Phalcom runtime. Realises [ADR-0009](../../adr/0009-handle-arena-heap.md)
> (the handle heap) and [ADR-0050](../../adr/0050-non-moving-mark-sweep-collector.md)
> (the collector). The surface contract of `System.gc` lives in
> [system.md](system.md) §`gc`; this document specifies what backs it.

Related: [values-and-absence.md](values-and-absence.md) (the `Value` tags; the
private `nil` sentinel), [object-model.md](object-model.md) §6 (kernel cycle),
[ADR-0010](../../adr/0010-tagged-value-enum.md) (`Value` representation),
[ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md) (fibers).

---

## 1. The heap and object lifetime

All heap objects — instances, classes/metaclasses, methods, modules, closures,
strings, blocks, bound methods, upvalue cells, lists, fibers, maps, sets, tuples,
ranges, families — live in one central `Heap`, a generational arena
(`SlotMap<ObjRef, Object>`). Every heap object is named by a `Copy` **handle**
(`ObjRef`, or `ClassId` for classes); a handle is an index-plus-generation, not a
pointer. Immediates (`nil` sentinel, `Bool`, `Number`, `Symbol`) live inline in
`Value` and are **never** heap objects — they are neither allocated nor collected.

An object's lifetime begins at `Heap::alloc` and ends when the collector proves it
**unreachable** and sweeps its slot. Sweeping a slot bumps its generation, so any
handle still naming the freed object becomes **stale** and resolves to a defined
diagnostic (never to a different object, never to undefined behaviour).

**No finalization.** Object destruction runs no user code. There is no `Drop`
protocol, no `finalize`, no resurrection. Cleanup that must run on a code path
(`ensure`/`finally`) is driven by unwinding
([ADR-0008](../../adr/0008-error-handling-model.md)), never by collection. This is
a standing invariant, not an omission (§7, Invariant M4).

## 2. Reachability

An object is **live** iff it is reachable by a directed path of handles from a
**root**. Reachability is transitive through every handle an object stores.

### 2.1 Roots (normative)

The root set is exactly:

> **Verified against HEAD 2026-07-14.** `ClassId` is a type alias for `ObjRef`
> (`heap/mod.rs`), so every row below names the same handle type.
>
> **This table is no longer the enforcement mechanism — the code is.**
> `VM::collect_roots` (`vm/gc.rs`), `Universe::each_handle` and
> `CoreClasses::each_handle` are written as **exhaustive destructures**, so a new
> field on any of the three fails to compile until it is explicitly classified as a
> root or a non-root. This table documents that classification; it cannot drift
> ahead of the code without a build error. That inversion is deliberate: three
> roots (`sealed_classes`, `checking`, `ready_queue`) were missed by hand-auditing
> this table, the last of them *after* a dedicated audit pass — see forge finding F6.

| Root | Type | Note |
|---|---|---|
| the running fiber's operand stack | `VM::stack: Vec<Value>` | every operand |
| the running fiber's call frames | `VM::frames: Vec<CallFrame>` | each frame's `closure`, its `context` (`Instance`/`Class`/`Module` handle, **or** an `Immediate` `Value` that may be an object), **not** its `home_frame_token` (an index+generation, not a handle) |
| open upvalue cells | `VM::open_upvalues: BTreeMap<usize, ObjRef>` | the cell handles |
| the current fiber | `VM::current: ObjRef` | |
| the scheduler run queue | `VM::ready_queue: VecDeque<ObjRef>` | fibers `System.schedule(_)` has enqueued but not yet resumed (`started == false`). **Reachable from nowhere else** until the pump drains them — missing this root frees a scheduled fiber |
| loaded modules | `VM::modules`, `main_module`, `last_imported_module` | module handles |
| named classes | `VM::classes: HashMap<Symbol, ClassId>` | class handles |
| sealed-class registry | `VM::sealed_classes: HashMap<Symbol, ObjRef>` | the sealing class-object handles (U-ANNOT-LAYOUT, `@sealed`/`@variant`) |
| contract re-entrancy guard | `VM::checking: HashSet<ObjRef>` | receivers currently under `@invariant` checking — the live mirror of `FiberObject::checking` (U-ANNOT-CONTRACTS) |
| the kernel | `VM::universe` (`Universe`) | **pinned** — every handle it holds: `CoreClasses`' 31 `ClassId`s + the `none_singleton` `ObjRef`, **and** `Universe::module_registry: HashMap<String, ObjRef>`. Never swept (§6) |
| native temp roots | `VM::temp_roots: Vec<ObjRef>` | the §4 escape hatch |

Everything else is reached **transitively**, not rooted directly. In particular a
**parked** (non-current) fiber is *not* a top-level root: it is reached only via
the `resumer`/caller chain from `VM::current`, or via a live handle some other
object holds. A parked fiber that nothing references is garbage, and the objects
its saved stack alone kept alive die with it.

### 2.2 Non-roots (normative — do not add these)

- **Interned symbols.** `Symbol`s live in the interner, not the heap; they are
  never collected and require no root.
- **`ClassLayout`** (`VM::field_layouts`), `constructor_aliases`,
  `has_new_construct`, `class_parents`. These hold only `Symbol`s and integers —
  **no handles** — so they contribute no roots. (Re-verified on HEAD 2026-07-14:
  `ClassLayout { name: Symbol, field_slots: IndexMap<Symbol, u16>, field_count: u16,
  static_field_slots: IndexMap<Symbol, u16>, static_field_count: u16 }`;
  `class_parents: HashMap<Symbol, Symbol>`;
  `constructor_aliases: HashMap<(Symbol, Symbol), Symbol>`;
  `has_new_construct: HashSet<Symbol>`. Note the near-miss: `VM::sealed_classes`
  *looks* like a peer of these but holds `ObjRef` values — it **is** a root, §2.1.)
- **Heap string content.** Strings are ordinary collectible objects. There is no
  strong content-addressed string table; "interned by content" denotes
  value-equality (`Value::value_eq` compares string content), not handle dedup.
  A live string is retained only by a live holder.

### 2.3 Outgoing edges (what tracing must visit)

Tracing visits, per `Object` variant, every handle and every `Value` it stores.

> **Regenerated from HEAD 2026-07-14** against all **16** `Object` variants and
> every field of each payload struct. This table is normative and exhaustive: a
> field absent here is asserted to hold **no handle**. The collector's `match` over
> `Object` must likewise be exhaustive (§3), so a new variant cannot compile without
> declaring its edges — but a new *field* on an existing variant can, which is why
> this table lists fields, not just variants.

| Variant | Edges to trace | Explicitly **not** edges |
|---|---|---|
| **Instance** | `class`, each `slots` `Value` | — |
| **Class** | `class` (metaclass), `superclass`, each `methods` value (`IndexMap<Symbol, ObjRef>` — methods are heap objects), each `static_slots` `Value`, each `attributes` `Value` | `name: String` (a **Rust** `String`, not a heap object), `field_slots`, `base_names`, `field_count`, `attributes_frozen` — `Symbol`s/ints/bool |
| **Method** | `kind` when `MethodKind::Closure(ObjRef)`, `holder`, each `contracts` entry's `.1` `Value`, each `attributes` `Value` | `kind` when `MethodKind::Primitive(fn)` (a Rust fn pointer — no Phalcom handle), `signature`, each `contracts` entry's `.0` `Symbol`, `attributes_frozen` |
| **Module** | `closure`, each `globals` `Value`, each `attributes` `Value` | `name_sym`, `name`, `path`, `source` (`Arc<String>`), `name_to_slot`, `attributes_frozen` |
| **Closure** | `module`, each `upvalues` handle, each `callable.chunk.constants` `Value` (string literals, selector symbols) | `callable.upvalues` (`UpvalueDescriptor` — `is_local`/`index`), `max_slots`, `num_upvalues`, `arity`, `name_sym` |
| **Str** | **none** — a leaf | `value: String`, `hash: u32` |
| **Block** | `closure` | `home_frame_token: FrameToken` (index+generation, not a handle) |
| **BoundMethod** | `method`, `receiver` | — |
| **Upvalue** | `Open { fiber, .. }` → the **`fiber` handle**; `Closed(Value)` → the `Value` | `Open { slot }` — a stack index |
| **List** | each `elements` `Value` | — |
| **Fiber** | each saved `stack` `Value`, each saved `frames` `CallFrame` (as under `VM::frames`, §2.1), each `open_upvalues` value, `resumer`, `result`, `entry`, each `checking` handle | `status`, `started`, `resume_slot`, `floor_depth`, `resume_mode` |
| **Map** / **Set** | per `entries` tuple `(Value, Value, i64)`: `.0` (key) and `.1` (value) | `.2` (the cached hash), `index: HashMap<i64, Vec<usize>>` |
| **Tuple** | each `elements` `Value` | — |
| **Range** | `start`, `end` | `inclusive: bool` |
| **Family** | `recv` | `selector: Symbol`, `open: bool` |

Two edges are easy to miss and each would free a live object:

- **`Upvalue::Open` carries a `fiber: ObjRef`**, not merely a stack index. The
  aliased slot lives on *that* fiber's stack, which is the `VM` mirror **only while
  that fiber is current** — otherwise it is parked inside the `FiberObject`. So an
  `Open` cell is *not* "already traced as a root"; tracing its `fiber` handle is
  what reaches the parked stack holding the slot.
- **`Block` holds a `closure`**, and a `Block` is the only thing retaining it in a
  `[…]`-value-passed-around program.

The tracer visits `Value` children through a single `Value` object accessor, not
by pattern-matching `Value`'s tags, so a future `Value` representation change
(NaN-boxing) does not touch the collector.

**While a fiber is `current`, its authoritative state is the `VM` mirror** and its
own `FiberObject` `stack`/`frames`/`open_upvalues`/`checking` buffers are stale
(emptied by `store_live_into`/`load_live_from`) — the tracer roots the mirror
(§2.1) and does not re-trace the current fiber's internal buffers. Tracing them
anyway is harmless (they are empty), but the mirror is the truth.

## 3. The collector

Reclamation is **non-moving, precise, stop-the-world mark-sweep**
([ADR-0050](../../adr/0050-non-moving-mark-sweep-collector.md)):

1. **Mark.** Clear the mark set (a `SecondaryMap<ObjRef, ()>` — marks live beside
   the objects, not in them). Push every root (§2.1) onto an explicit worklist,
   marking on push. Pop until empty; for each object, visit its outgoing edges
   (§2.3), marking-and-pushing each unmarked child. Marking never recurses on the
   native stack.
2. **Sweep.** Retain exactly the marked slots (`SlotMap::retain`); every unmarked
   slot is removed, returning it to the free list and bumping its generation.
3. Clear the mark set.

The collector is **non-moving**: a surviving object keeps its handle for life.
This is required, not incidental — inline-cache tags
([ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md)), `==`
object identity (`Value::value_eq`), and the `Value`s held in suspended fiber
stacks all assume a handle names the same object across a collection.

No write barrier is required, because the collector is stop-the-world: it observes
a consistent heap at a single safepoint.

## 4. Safepoints and native code

The collector runs **only at a safepoint**: a point in the interpreter loop where
`VM::stack`/`frames` are the complete, precise root truth. `Heap::alloc` records
allocation and, when the live-size threshold is crossed, **latches** a pending-GC
flag rather than collecting in place; the dispatch loop services the flag at a
back-edge. Collection therefore **never** runs in the middle of a native
primitive, so `ObjRef`s held in a primitive's Rust locals across a *single*
allocation are never observed by the collector.

**The one hazard — re-entrant native handles.** A native primitive that holds a
freshly allocated handle across a call which re-enters the interpreter
(`send_dynamic`, `block_call`, `invoke_method_object`) *can* reach a safepoint
while that handle is live only in a Rust local — invisible to the root set. If the
object is otherwise unreachable, it is swept, and the later use resolves to the
stale-handle diagnostic (defined, but a lost live object — a bug).

Such a primitive **must** protect the handle:

```rust
vm.push_temp_root(h);          // h is a fresh alloc held across a re-entrant send
let r = vm.send_dynamic(recv, sel, &args)?;
vm.pop_temp_root();            // paired; the temp-root stack is a root (§2.1)
```

Handles that already live on the VM operand stack (receiver, arguments) need no
temp root — they are rooted there. The temp-root requirement applies only to
handles held *solely* in native locals across a re-entrant send.

## 5. `System.gc`

`System.gc` forces one full mark-sweep at the current safepoint and returns `None`
([system.md](system.md) §`gc`, [values-and-absence.md](values-and-absence.md)).
It guarantees:

- every object unreachable from the roots (§2.1) at the call is reclaimed;
- **no** finalizer or user code runs during collection (§1);
- **no** object is moved and **no** handle changes — a handle live before the call
  names the same object after it;
- the kernel (§6) survives regardless of reachability.

It makes **no** guarantee about compaction, address stability beyond handles, or
returning memory to the OS.

## 6. Kernel pinning and invariants

The kernel — every class, metaclass, and singleton reachable from `CoreClasses`,
including the `Metaclass`-is-an-instance-of-itself apex and the shared `None` /
`True` / `False` singletons — is **pinned**: traced from the root set every cycle
and never swept, even were it momentarily unreachable from user code. Mark-sweep
collects the kernel cycle correctly (rooting it once, the mark bit stops the
loop), so pinning is a liveness guarantee, not a cycle workaround.

`verify_invariants()` gains a post-collection check: after any collection, every
`CoreClasses` handle still resolves, and the metaclass apex is intact.

### Invariants (M-series)

- **M1 — handle stability.** A collection never changes the handle of a surviving
  object. (Enables IC tags, `==` identity, fiber-stack `Value`s.)
- **M2 — precise roots.** Collection observes the exact root set of §2.1; it never
  conservatively scans the native stack and never retains an object no root
  reaches (modulo the kernel pin, M5).
- **M3 — no dangling liveness.** No live object is swept. The safepoint +
  temp-root discipline (§4) is what discharges this for native code.
- **M4 — no finalization.** Collection runs no user code and never resurrects an
  object (§1).
- **M5 — kernel liveness.** The kernel survives every collection (§6).
- **M6 — defined staleness.** A handle to a swept object resolves to the
  stale-handle diagnostic, never to a live object or undefined behaviour.

## 7. Representation discipline & what stays open

- **Object slot size.** `size_of::<Object>()` is bounded by discipline: fat
  variants are boxed so the enum stays small and the arena stays cache-dense. The
  `SlotMap` sizes every slot to the fattest variant, so an unboxed fat variant taxes
  *every* string, range, and instance on the hot `heap.get` path threaded through
  all dispatch.

  **Measured pre-boxing on HEAD 2026-07-14: 280 B** — up from the 256 B ADR-0050
  recorded, because `ClassObject` gained `attributes: Vec<Value>` (+24 B) and
  `attributes_frozen` (U-ANNOT). `ClassObject` alone *is* the 280 B. The descending
  ladder, which decides how many variants must be boxed to hit a given bound:

  | Variant | `size_of` payload |
  |---|---|
  | `ClassObject` | 280 |
  | `FiberObject` | 176 |
  | `ModuleObject` | 168 |
  | `ClosureObject` | 160 |
  | `MethodObject` | 88 |
  | `MapObject` (`Map`/`Set`) | 72 |
  | `RangeObject` | 40 |
  | `StringObject` | 32 |
  | `Instance`/`Block`/`List`/`BoundMethod`/`Upvalue`/`Family` | ≤24 |
  | `TupleObject` | 16 |

  Boxing must therefore cover **`Class`, `Fiber`, `Module`, `Closure`, `Method`,
  `Map`/`Set`** — six variants — for `Range` (40 B) to become the cap. Boxing
  `Instance` is *counterproductive*: at 24 B it is already below the floor and is
  the most-allocated variant, so a `Box` would add an indirection and an allocation
  for no size win. (ADR-0050 §9's variant list predates this measurement; this table
  supersedes it.)
- **`Value` size.** `Value` is a 16-byte tagged enum today. NaN-boxing to a single
  8-byte word is deferred behind the `Value` API
  ([ADR-0010](../../adr/0010-tagged-value-enum.md)); §2.3's object accessor is the
  seam that keeps the collector independent of this.
- **Fiber-stack pooling.** Fiber operand/frame `Vec`s may be pooled and reused
  across fiber deaths to cut allocation churn; this is a memory-management
  optimization orthogonal to reclamation and changes no observable semantics.
  **Measured null pre-collector** (forge finding F5, `docs/forge/perf-log/findings.md`):
  built and A/B'd against Skynet, indistinguishable, because RSS there is dominated
  by ~1M immortal `FiberObject` shells that only reclamation can free — pooling
  their buffers cannot move it. Orthogonal to the collector in *mechanism*, but not
  in *sequence*: it is only measurable once sweeping frees the shells. Re-measure
  after the collector against a high-fiber-turnover workload; do not land it before.

The following remain deliberately **open**, each implementable behind the handle
surface without a semantic change:

- **Generational collection** (nursery + remembered set) — needs a write barrier;
  gated on a measured need. To keep the retrofit cheap, mutation is funnelled
  through a small set of choke-point methods now.
- **Incremental collection** (tri-color marking) — colors the mark table; needs a
  barrier; gated on a measured pause.
- **Compaction / moving** — would require a handle→slot indirection to preserve M1;
  explicitly reversible-open, not planned.
