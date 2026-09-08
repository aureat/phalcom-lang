# Raw source snapshot: runtime

> Captured: 2026-09-08
> Source area: runtime specifications and runtime/compiler-program boundaries
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/spec/current/object-model.md ---
# Object Model

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0002](../../adr/0002-metaclass-tower-parallel-rule.md) (metaclass tower parallel rule) ·
[ADR-0003](../../adr/0003-introduce-behavior-kernel-class.md) (Behavior kernel class) ·
[ADR-0009](../../adr/0009-handle-arena-heap.md) (handle/arena heap) ·
[ADR-0010](../../adr/0010-tagged-value-enum.md) (tagged Value enum)

This part defines the **kernel**: the class/metaclass tower and the catalog of
core classes. Surface semantics live in the sibling parts. It is reconciled with
the [Values & Absence](values-and-absence.md) decisions (private `nil` + `Option`,
abstract `Bool` with `True`/`False` subclasses ([ADR-0004](../../adr/0004-boolean-as-abstract-bool-with-true-false.md)),
`Block` as the closure class) and the instance-display decision in
[ADR-0015](../../adr/0015-object-default-tostring.md).

---

## 1. Principles

1. **Everything is an object.** `true`, `42`, `"hi"`, a block, a class, a method,
   a module — all respond to messages. (`nil` is the sole exception: it is a
   private VM sentinel, not a surface value — see [Values & Absence](values-and-absence.md).)
2. **Every object is an instance of exactly one class.** `value.class` is total.
3. **Every class is an object**, hence an instance of a class — its *metaclass*.
4. **Message send is the only computational primitive.** The compiler may
   *inline* some sends (`if`, `+`, `and`) but the semantics are method sends.
5. **Single inheritance.** One `superclass` per class; `Object` is the root.
6. **Uniform tower.** Class-side (`@class`, `@constructor`) methods obey the same
   inheritance rules as instance-side methods, via the parallel metaclass
   hierarchy (§5, [ADR-0002](../../adr/0002-metaclass-tower-parallel-rule.md)). No class is special-cased to lack a metaclass.

---

## 2. Core rules

- Two relationships define the model:
  - **instance-of** (`x.class`): every object → its class. Never `nil`.
  - **inherits-from** (`C.superclass`): every class → its superclass, or *none*
    at the root (`Object`).
- **Method lookup** walks the *class* of the receiver up the `superclass` chain
  (see [Method Lookup](method-lookup.md)). It never consults the metaclass chain
  for an instance send, nor the instance chain for a class-side send.
- A class carries: a `name`, a `superclass`, a `metaclass` (its `.class`), a
  method dictionary keyed by **label-encoded selector symbol**
  (see [Messages & Selectors](messages-and-selectors.md)), and a fixed instance
  field layout (see [Classes §Fields](classes.md)).
- **Abstract** classes define protocol but are never the direct class of a live
  value (e.g. `Behavior`, `Number`). **Immediate/primitive** classes have live
  values in a non-`Instance` VM representation (e.g. `Float` → `f64`, `Int` →
  tagged `i64` / heap `LargeInt`).

### 2.1 Member namespaces and access

Inside a member body, `_field` and `self._field` denote the same source field;
`__field` and `self.__field` likewise denote the same privileged implementation
field. Field access is receiver-local: `other._field` and `other.__field` are
rejected rather than converted into sends. Implementation fields and `_$name`
selectors are available only to privileged core/runtime source.

Ordinary unresolved names use `local → upvalue → known global → implicit self`
resolution (known globals include declared classes, let/const bindings, linked
module imports, and generated compile-time global declarations such as `@variant`
siblings). Namespace-directed `_field`, `__field`, and `_$name` forms always
target `self`; nested blocks retain the enclosing member's lexical receiver and
access class.

`@private` and `@protected` control selector access, never naming. Private
members are callable only from their defining source class. Protected members
are callable from that class and its subclasses. Lookup, cached dispatch,
`perform`, `respondsTo`, `methodFor`, and invocation through a reified method all
apply the same access check. Physical method enumeration may expose that a
selector exists, but it does not grant invocation authority.

---

## 3. Value representation

The VM's tagged value maps onto classes as follows. `x.class` is total for every
surface value; primitives bypass the generic instance representation.
**Ratified representation: [ADR-0010](../../adr/0010-tagged-value-enum.md).
Object references are `ObjRef` handles into the arena heap: [ADR-0009](../../adr/0009-handle-arena-heap.md).**

| Surface value | Class | Notes |
|---------------|-------|-------|
| `true` / `false` | `True` / `False` | abstract `Bool` with concrete singleton subclasses `True`/`False` ([ADR-0004](../../adr/0004-boolean-as-abstract-bool-with-true-false.md)); `true.class == True`. `ifTrue`/`ifFalse`/`and`/`or`/`not` live on `Bool`, inherited. |
| `42` | `Int` | exact, unbounded integer (§4 note; [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md)) |
| `3.14` | `Float` | IEEE-754 `f64` (§4 note; [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md)) |
| `"hi"` | `String` | immutable, interpolating |
| `#name` / selectors | `Symbol` | interned — see [Selectors, Symbols & References §2](selectors.md#2-symbol-literals-) for the name-symbol (`#name`) vs. selector-symbol (`#name(_,to,duration)`) distinction |
| `{ x => … }` | `Block` | closures / block literals |
| a compiled method | `Method` | reified send target |
| `(3, 4)` | `Tuple` | fixed-arity product |
| `[1, 2]` | `List` | |

--- docs/spec/current/memory-management.md ---
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
strings, blocks, bound methods, upvalue cells, lists, fibers, maps, sets, bytes,
tuples, records, ranges, families, arbitrary-precision integers, and private
compiler builders — live in one central `Heap`, a generational arena
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
| the kernel | `VM::universe` (`Universe`) | **pinned** — every handle it holds: `CoreClasses`' class IDs and `Universe::module_registry: HashMap<String, ObjRef`. `None` is immediate and has no singleton handle; wrapped immediate `Some` payloads are traced through `Value::gc_obj_ref()`. Never swept (§6) |
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

--- docs/spec/current/error-handling.md ---
# Error Handling

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0008](../../adr/0008-layered-exceptions-and-result.md) (layered exceptions + `Result`, terminating semantics) ·
[ADR-0031](../../adr/0031-error-handling-surface-syntax.md) (surface syntax: `throw`/`try`/`catch`/`on`/`ensure`)

Phalcom has **two** failure channels, layered rather than competing:

- **Exceptions** — `throw` an [`Error`](object-model.md) and unwind the stack. For
  *exceptional* and cross-cutting failures: bugs, invariant violations, dead
  frames, and anything that arises deep in the VM (`doesNotUnderstand`, arity
  mismatch, `DeadFrameError`).
- **`Result` / `Option`** — ordinary values ([Values & Absence](values-and-absence.md)).
  For *expected, local* failures you want visible in the type: parse, validate,
  lookup.

Neither is primary. The runtime must raise regardless — VM-internal failures
cannot be `Result`s — and expected outcomes are cleaner as values than as control
flow. Cheap bridges (§5) mean code is never trapped in the wrong channel.

## 1. Raising: `throw`

```phalcom
throw ArgumentError("age must be >= 0")
```

`throw expr` unwinds the stack (§4). It is surface sugar for the reflective form
`expr.raise()` ([Object Model §Errors](object-model.md)), exactly as `return x`
relates to the VM's unwind machinery.

**Only `Error` subclasses are throwable.** `throw "oops"` is a compile error. This
is a deliberate, signposted deviation from JavaScript's throw-anything: typed
handlers (§2) and the `message` protocol depend on every thrown value being an
`Error`.

## 2. Handling: a `Block` protocol, with sugar

Everything is a message ([Invariant 1](README.md)), so the primitive is sends on a
protected block — the same shape as control flow:

```phalcom
{ risky() }
  .on(TypeError) { e => recover(e) }   // Smalltalk on:do: — typed handler
  .on(RangeError) { e => fallback() }
  .ensure { cleanup() }                // finally — always runs (§4)
```

- `on(_)(_)` installs a handler for one `Error` class (and its subclasses).
  Handlers chain; the **first** matching class wins; an unmatched error keeps
  unwinding.
- `ensure(_)` runs its block on every exit path (§4).

### The `try` statement (sugar)

`try` / `on` / `catch` / `ensure` desugar directly to the block protocol
([ADR-0031](../../adr/0031-error-handling-surface-syntax.md)):

```phalcom
try {
  risky()
} on TypeError e {
  recover(e)
} catch e {
  fallback(e)
} ensure {
  cleanup()
}
```

- `on T e { … }` ≡ `.on(T) { e => … }` — a typed handler for `T` and its
  subclasses. Clauses chain; the **first** matching class wins.
- `catch e { … }` ≡ `.on(Error) { e => … }` — catch-all, since `Error` is the root
  of the raisable hierarchy.
- `ensure { … }` ≡ `.ensure { … }` — runs on every exit path (§4).

Each keyword mirrors the block-protocol method of the **same name**, so the sugar
adds no semantics the protocol lacks; it exists so a JavaScript programmer is not
surprised ([Invariant 6](README.md)). `on`/`catch`/`ensure` are **contextual
keywords** (reserved only as `try`-clauses), so the `.on()`/`.ensure()` selectors
and the `Fiber>>try` message keep working.

## 3. Terminating, not resumable

A `throw` **always unwinds**. Phalcom does not adopt Smalltalk's resumable
conditions (`resume:`): keeping the raising frame alive plus a handler-return
protocol is heavy, rarely used, and fights the frame-token unwinding already in
the VM ([Blocks §5](blocks.md), [Functions §3](functions.md)). A handler runs
*after* the stack between `throw` and the handler has been discarded.

`retry` (re-run the protected block) is a natural future addition — the block is
still live — but is deliberately left out of Draft 0.1.

## 4. Unwinding is one primitive

--- docs/implementation/RUNT001-runtime-representation/PROGRAM.md ---
---
id: RUNT001
category: RUNT
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# RUNT001 — runtime representation

This program owns runtime representation records that are not part of the
ADT/GADT or canonical-universe programs.

--- docs/implementation/RUNT002-core-runtime/PROGRAM.md ---
---
id: RUNT002
category: RUNT
kind: as-built
status: COMPLETE
completion: IMPLEMENTED
verification: UNVERIFIED
---

# RUNT002 — core runtime

This program contains the core-runtime implementation records as an as-built
evidence set.

--- docs/implementation/COMP001-compiler-structure/PROGRAM.md ---
---
id: COMP001
category: COMP
kind: refactor
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# COMP001 — compiler structure

This program owns structural compiler refactors that preserve behavior while
separating compiler, VM, and Universe responsibilities.

--- docs/implementation/COMP002-compiler-runtime/PROGRAM.md ---
---
id: COMP002
category: COMP
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# COMP002 — compiler runtime

This program owns pending compiler/runtime lowering and execution integration
records that do not belong to a structural refactor.

--- docs/implementation/MEMM001-garbage-collection/PROGRAM.md ---
---
id: MEMM001
category: MEMM
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# MEMM001 — garbage collection

This program owns garbage-collection implementation and its stepwise runtime
integration.
