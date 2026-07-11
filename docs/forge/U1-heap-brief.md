# Staged brief — U1: handle/arena heap + tagged `Value` (the VM foundation)

_Launch this AFTER the green base is committed, in a clean git worktree. It is the foundational
core unit — everything downstream codes against the types it introduces. Load-bearing → reviewed._

## Unit
Migrate the Phalcom object graph from `Rc<RefCell<T>>` handles (`PhRef`) to a **central handle/
arena heap** with `Copy` handles, and make `Value` a tagged `enum` — per **ADR-0009** (heap) and
**ADR-0010** (Value repr). This kills the metaclass-kernel `Rc` cycle leak (audit finding F5) and
the `RefCell` double-borrow panic surface, and makes the object graph inline-cache- and GC-ready.

## Scope discipline — this is a BEHAVIOR-PRESERVING representation migration
- Same language semantics, new substrate. Every golden program and every currently-passing
  invariant test MUST stay green with identical output.
- **Do NOT fix the metaclass parallel-superclass bug (F2) here — that is U2.** Preserve current
  tower behavior so the gate stays green. ADR-0009 supersedes ADR-0002's `Rc::new_cyclic`
  mechanism with "allocate handles, then patch their fields" — implement the bootstrap that way,
  but keep the *observable* wiring identical to today.
- No new language features. Pure representation change.

## Design (from ADR-0009 / ADR-0010)
- A `Heap` owning object storage; objects referenced by a `Copy` handle (`ObjRef`, plus typed
  aliases like `ClassId` if useful). Consider `slotmap`/`generational-arena` vs a hand-rolled
  `Vec`-arena — justify the choice in code docs; generational handles preferred (use-after-free
  becomes a clean error, not UB). Interior mutability moves into the heap, not per-object `RefCell`.
- `Value` = tagged `enum`: `Number(f64)`, `Bool(bool)`, `Obj(ObjRef)`, a **private** `Nil`
  sentinel (not surface-visible), interned `Symbol`. NaN-boxing stays deferred (same API later).
- The `VM` owns the `Heap`; object methods that today do `self.borrow()` take `&Heap`/`&mut Heap`.

## Write-set (LARGE — derived from `graphify affected "PhRef"` + `"Value"`; this unit runs alone)
- `phalcom-common/src/refs.rs` (+ `lib.rs` exports) — replace/repurpose `PhRef`.
- `phalcom-core/src/heap.rs` (NEW) — the `Heap` + handle types.
- `phalcom-core/src/value.rs` — the tagged `Value`.
- `phalcom-core/src/{class,instance,closure,frame,method,module,universe}.rs` — handle-based.
- `phalcom-core/src/vm.rs` — VM owns the heap; dispatch/bootstrap use handles.
- `phalcom-core/src/compiler/lib.rs` — constant/handle references.
- `phalcom-core/src/primitive/*.rs` — take/return `Value`/handles.
- `phalcom-core/bin/phalcom/disasm.rs` — handle-aware disassembly.
- `phalcom-core/Cargo.toml` — arena crate dep if used.
- Re-run `graphify affected` on the ACTUAL committed base first to confirm nothing else references `PhRef`.

## Mandatory rules
- **Documentation** (`docs/rust-documentation-guidelines.md`): `//!` on the new `heap` module +
  every touched module; `///` on every public item — `Heap`, `ObjRef`, `Value` + all variants,
  every migrated method — with `# Panics`/`# Safety` (if any `unsafe` in the arena; prefer safe
  generational indices and a `// SAFETY:` note on any unsafe block), intra-doc links, and cite
  ADR-0009/0010. `cargo doc --workspace --no-deps` adds no new warnings.
- **Green gate**: `./scripts/verify.sh` green (build+test+clippy+golden+invariants). If the
  `#[ignore]`d spec-target invariants exist, they stay ignored (U2 addresses them).
- **Orientation**: graphify-first; run `graphify affected` on every type you change.
- **Best practices** (`rust-best-practices`, `rust-sanitizers-miri` if any unsafe): no
  reintroduced borrow hazards; the whole point is to remove them.
- Stay in the write-set; if forced outside it, STOP and report a conflict. Append out-of-scope
  ideas to `docs/forge/DEFERRED.md`.

## Return
Design summary (heap/handle choice + why, Value layout), files changed, how goldens+invariants
stayed green, `verify.sh` + `cargo doc` tails, confirmation F2 was NOT touched, DEFERRED entries.
Do not self-approve — a reviewer verifies representation correctness + borrow-hazard removal.
