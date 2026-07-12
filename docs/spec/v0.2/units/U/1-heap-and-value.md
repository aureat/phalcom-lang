# U1 — Heap & Value (as-built)

- **Status:** ✅ Landed — `6515ea3` (`feat(u1): handle/arena heap + tagged Value`, 2026-07-11).
- **Realizes:** [ADR-0009](../../../../adr/0009-handle-arena-heap.md) (handle/arena heap) + [ADR-0010](../../../../adr/0010-tagged-value-enum.md) (tagged `Value` enum); spec [object-model §3](../../object-model.md) (value representation) and [§6](../../object-model.md) (bootstrap), [values-and-absence §2](../../values-and-absence.md).
- **Reviewer gate:** **ON** (load-bearing — can corrupt the object model). Independent `phalcom-reviewer` **PASSED** after a scoped fixer restored a `Symbol`/`Module` `==`/`!=` semantics regression (`value_eq` had fallen through to derived `PartialEq`).

## Mission
Migrate the object graph off `Rc<RefCell<T>>`/`PhRef` onto a central `slotmap`-backed `Heap` addressed by `Copy` generational handles, and define `Value` as a tagged enum carrying immediates inline and heap objects by handle. This removes the `RefCell` double-borrow panic surface and the inert Rc cycle-breaker (F5), and makes the substrate inline-cache- and GC-ready — a behavior-preserving migration.

## Surface / behavior
No surface-language change; behavior preserved observationally (goldens byte-identical). The change is representational:

- Every heap object is reached through a handle, not a pointer. Comparing two `Value::Obj` handles tests **object identity**; string content-equality is a separate path (`Value::value_eq`).
- The cyclic kernel (a metaclass that is an instance of itself) is now expressed as handles pointing at each other — no ownership paradox, no `Rc::new_cyclic`.

## Implementation
- **`phalcom-core/src/heap.rs`** — the central `Heap` is a `slotmap::SlotMap<ObjRef, Object>` (zero `unsafe` at the call site). `new_key_type! { ObjRef }` is a `Copy` generational key; `ClassId` is a documentation type-alias of `ObjRef` for class-typed fields. `Object` is the tagged payload stored per slot: `Instance`, `Class`, `Method`, `Module`, `Closure`, `Str`, `Block`, `Upvalue`, `List` (the last two added by later units). Accessors (`get`/`get_mut`/`class`/`class_mut`, `alloc`/`alloc_class`/`alloc_string`/`alloc_list`) dereference a handle through the arena. Generational keys make a stale handle resolve to `None` rather than undefined behavior (no use-after-free).
- **`phalcom-core/src/value.rs`** — `Value` is a `#[derive(Clone, Copy)]` tagged enum: `Nil` (private uninitialized-slot sentinel), `Bool(bool)`, `Number(f64)`, `Symbol(Symbol)`, `Obj(ObjRef)`. Every arm is `Copy`, so values move freely on the VM stack and in constant pools without cloning or refcounting. `Value::class` is total for every arm except the private `Nil`.
- **Ownership.** The `VM` owns exactly one `Heap`; methods that formerly called `self.borrow()`/`borrow_mut()` now take `&Heap`/`&mut Heap`.
- **Bootstrap.** ADR-0002's allocate-then-wire ordering becomes **allocate-then-patch over `ClassId`s** ("instance of itself" = a handle pointing at itself). ADR-0009 supersedes `Rc::new_cyclic`.
- Migration touched `class.rs`, `instance.rs`, `method.rs`, `module.rs`, `closure.rs`, `frame.rs`, `interpret.rs`, `compiler/lib.rs`, and the primitives — see the `6515ea3 --stat`.

## Invariants & tests
- **F2** (metaclass parallel-superclass deviation) preserved observationally — its fix was deferred to [U2](2-metaclass-tower.md).
- `verify.sh` green; golden corpus byte-identical; `cargo doc` clean.
- The reviewer-blocked `Symbol`/`Module` `==`/`!=` regression was restored via `value_eq` before merge.

## Deviations & deferrals
- **Reviewer-blessed deviation:** the compiler allocates constants directly via `&mut VM` into the one VM-owned `Heap` (not the plan's heap-free-descriptor approach) — sound for U1 (single heap, VM-lifetime handles). A "true heap-free compiler" is deferred.
- **NaN-boxing is deferred** behind the same enum API (ADR-0010) — packing `Value` into a single NaN-tagged `f64` word is a later optimization, not this unit. See [deferred-work](../../deferred-work.md).
- **A tracing collector is not built** — the arena is *designed* to host one (stable handles survive relocation/reclamation) but reclamation is deferred (ADR-0009). `System.gc` remains future work.
- Every dereference threads a heap reference — the deliberate cost of removing pointer aliasing.
- Closed **DEFERRED #1**: LALRPOP fully removed from the workspace dependency graph.

## Sources
- forge: [`STATE.md`](../../../../forge/STATE.md) "U1 — LANDED", [`U1-plan.md`](../../../../forge/U1-plan.md), [`U1-progress.md`](../../../../forge/U1-progress.md).
- code: `phalcom-core/src/heap.rs`, `phalcom-core/src/value.rs`.
- ADRs: [0009](../../../../adr/0009-handle-arena-heap.md), [0010](../../../../adr/0010-tagged-value-enum.md).
- landing: `6515ea3` (squash of `feat/u1-heap`, slices 1–4 + review fix).
