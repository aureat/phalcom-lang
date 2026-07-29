# U7 — Fixed instance slot layout + `construct` + class-side static fields (as-built)

- **Status:** ✅ Landed — `b619448` (parse `construct`) → `f38e591` (layout + `construct` + static fields + dispatch fix) → `561f7e2` (two missing negative goldens). In-tree on `main`, no worktree.
- **Realizes:** [ADR-0011](../../../adr/0011-static-instance-slot-layout.md) (static per-class instance slot layout), [ADR-0017](../../../adr/0017-class-side-stored-static-fields.md) (class-side stored static fields, DEC-D); spec [classes §1](../../../spec/current/classes.md) (Constructors), [classes §2](../../../spec/current/classes.md) (Fields), [object-model §5](../../../spec/current/object-model.md) (metaclass tower). Reuses U6's private `Value::Nil` sentinel + surfacing helper.
- **Reviewer gate:** **OFF** per STATE.md policy (U7 is not hierarchy-load-bearing) — self-verified on the green gate + `cargo doc` clean.

## Mission
Replace the per-instance `IndexMap<Symbol, Value>` with a **fixed `Box<[Value]>` slot vector**
indexed by a per-class field table computed once at class-definition time (ADR-0011); make
read-before-write a compile error; add the **`construct`** initializer; and extend the same
slot mechanism one level up the tower so class objects get their own **stored static fields**
(ADR-0017 / DEC-D).

## Surface / behavior
- **Instance fields** are declared implicitly by `_`-prefixed assignment; there is no field
  declaration syntax. A field **read** whose name is assigned nowhere in the class is a
  compile error (`ReadBeforeWrite`, catches the `_naem` typo class).
- **`construct`** is a class-side initializer keyword. Multiple constructors are distinguished
  by **selector** (`new(name:age:)` vs `new(name:)` are two distinct initializers dispatched
  by arity/labels) — no default args, no arity coercion. A `construct` body implicitly
  `return self`. A class declaring `@constructor
new(...)` has **no** user-visible bare
  allocator; a mismatched-arity `new(...)` is a compile error.
- **`static _count = 0`** declares mutable per-class state stored on the class object, shared
  across all instances (not per-instance).
- **Unassigned slot → `None`** for both instance and static slots.

```phalcom
class Counter {
  static _count = 0
  @constructor
  new() { _count = _count + 1 }
  count => _count
}
Counter.new()
Counter.new()
System.print(Counter._count)   // → 2  (shared class state)
```

## Implementation
- **`instance.rs`** — `InstanceObject { class: ClassId, slots: Box<[Value]> }` (replaces the
  `IndexMap`). `new(class, field_count)` fills `slots` with the private `Value::Nil` sentinel.
- **`class.rs`** — `ClassObject` gains `field_slots: IndexMap<Symbol, u16>` (name → offset) +
  `field_count: u16` (instance layout) **and** `static_slots: Box<[Value]>` (ADR-0017); plus
  a name→slot resolver over the class's own table only (non-inherited). The instance table is
  the class's `field_slots`; the static table is the **metaclass's** `field_slots`.
- **`compiler/lib.rs`** — a whole-class field-collection pass over every method/getter/setter/
  `construct` body assigns instance offsets in first-assignment order, then fixes the layout;
  a parallel `static`-field pass collects into the metaclass field table + class-object static
  offsets. `Expr::Field` reads/writes lower to slot ops; `static _field` reads/writes target
  the class object's `static_slots`, not `self`. `construct` lowers to alloc-fresh-instance
  (`NewInstance`) + bind `self` (`SetLocal(0)`) + run body + implicit `return self`, installed
  class-side (`is_static = true` → metaclass) under a `SignatureKind::Initializer(arity)`
  selector (no new selector kind — `method.rs`/`encode_selector` already renders `init name(labels:)`).
- **`bytecode.rs`** — `GetField(u16)`/`SetField(u16)` operand meaning changed from a constant
  index of the field `Symbol` to a **direct slot index** (opcode arity unchanged); `NewInstance`
  alloc opcode. `disasm.rs` label text updated to slot semantics.
- **`vm.rs`** — executes slot-indexed field ops (an out-of-range/unwritten slot reads
  `Value::Nil`, surfaced to `None` via U6's helper — the sentinel never leaks); executes
  `construct`/alloc. **Constructor-dispatch fix:** `@constructor
new()` installs under the
  `Initializer` selector (`"init new()"`), but the call-site compiler for `Counter.new()`
  always encoded `SignatureKind::Method` (`"new()"`), silently resolving to the inherited
  `Object::new` bare-allocation primitive. A compile-time `VM.constructor_aliases:
  HashMap<(Symbol, Symbol), Symbol>` redirects a literal `ClassName.method(...)` call site
  to the matching `Initializer` selector; `VM.has_new_construct` makes a mismatched-arity
  `new(...)` a compile error rather than a silent fallthrough.

## Invariants & tests
- **Offset stability under inheritance** (`invariants.rs`): a subclass writing `_name` gets a
  **fresh** slot appended after the superclass's `[0..k)`; the superclass's offsets are never
  renumbered (`subclass_field_offset_stability`, `subclass_static_field_offset_stability`).
- **Positive goldens** (graduated out of `classes/pending/`): `class_construct_name`,
  `class_construct_returns_self`, `class_construct_selector_dispatch`,
  `class_field_unassigned_reads_none`, `class_static_field_unassigned_reads_none`,
  `class_static_field_shared_state`.
- **Negative goldens** (`561f7e2`, must fail to compile):
  `compile_error_field_read_before_write` (a `_naem` typo),
  `compile_error_no_matching_constructor` (mismatched-arity `new(...)` where a `construct new`
  exists → no user-visible bare allocator).
- **Green gate:** `verify.sh` exit 0; `cargo doc --workspace --no-deps` clean;
  `cargo clippy --workspace --all-targets` clean (also fixed one pre-existing `clone_on_copy`
  in `vm.rs`'s `Dup` handling).

## Deviations & deferrals
- **`construct` reuses the pre-existing `SignatureKind::Initializer`** — no new selector kind,
  no change to selector encoding or method lookup (U3 untouched).
- **Unassigned slots hold the raw `Value::Nil` sentinel, never a constructed `None`** —
  materializing a `None` instance into every fresh slot would reintroduce the bootstrap-absence
  cycle U6 solved.
- **DEC-D (static stored fields) was gated on ADR-0017 being Accepted first** — it was, before
  this slice landed.
- **Precluded (acceptable per ADR-0011/0017, good for a future inline cache):** adding a field
  to a *live* class / `become:`-style reshape (offsets frozen at definition); shared *protected*
  inherited fields (subclasses use accessors). Reparenting / `reshape`-with-migration is sealed
  by policy but left implementable → [deferred-work §1](../../../spec/current/deferred-work.md) (ADR-0026).
- **Identity-dispatch ⊗ optional arity:** any future default/optional arg on `construct` would
  change the selector and miss — a cross-cutting decision, not a local one
  ([deferred-work §1](../../../spec/current/deferred-work.md), open-Q12).

## Sources
- Forge: `U7-plan.md` (folded into this spec; see git history), [STATE.md](../../archive/phase2/STATE.md) "U7 — LANDED".
- Commits `b619448`, `f38e591`, `561f7e2`.
- Code: `phalcom-core/src/{instance,class,compiler/lib,bytecode,vm}.rs`,
  `phalcom-core/src/primitive/{object,class,nil}.rs`, `phalcom-core/bin/phalcom/disasm.rs`,
  `phalcom-ast/src/{token,lexer,ast,parser}.rs`.
