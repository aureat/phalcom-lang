# Bootstrap Phase Table (U-CORE-0)

> **Status:** Normative. Defines the ordered phases `VM::new` runs to bring the
> kernel object graph into a consistent state, the **preconditions and
> postconditions** of each phase, and — critically — the invariants each phase
> is *allowed to leave temporarily violated*. A flat "the invariants always
> hold" claim is **false during bootstrap**; this document replaces it with a
> phase-scoped ledger.

## 1. The fixpoint problem (why phases exist)

The kernel tower is cyclic: `Metaclass.class == Metaclass class` and
`(Metaclass class).class == Metaclass`; `Object` has no superclass yet
`Object class` inherits from `Class` ([`../object-model.md`](../object-model.md)
§5–6). No pure top-down or bottom-up construction order exists, so the tower is
built by **allocate-then-patch** ([ADR-0009](../../../adr/0009-handle-arena-heap.md)):
every class row is first allocated *bare* to obtain its `ClassId`, then its
`class` and `superclass` handles are written in place. Between allocation and
patching the graph is deliberately inconsistent. The phases below fence those
windows.

## 2. The phases

Source of truth: [`vm.rs::VM::new`](../../../../phalcom-core/src/vm.rs) L116–168 and
[`universe.rs`](../../../../phalcom-core/src/universe.rs).

| # | Phase | Code site | Produces |
|---|---|---|---|
| **A** | Substrate | `Interner::with_capacity(100)`, `Heap::new()` | empty interner + heap |
| **B** | Tower allocate-then-patch | `Universe::new` → `create_core_classes` | 19 named kernel classes (incl. `Message`, U8) (+ their metaclasses) + `None` singleton, fully wired |
| **C** | VM struct assembly | `VM { … }` literal | frames/stack/module maps, `universe` moved in |
| **D** | Core module + globals | `install_core` | core module registered; class globals + `None`-value global bound |
| **E** | Fixed-slot layouts | inline block, `VM::new` | `Some._value` at slot 0, plus the `Message` four-slot layout ([ADR-0011](../../../adr/0011-static-instance-slot-layout.md)) |
| **F** | Primitive floor install | `Universe::install_primitives` | all 80 native bindings ([`floor-census.md`](./floor-census.md)) |
| **G** | Run `core.ph` | `run_core_module` | `.ph` reopens attached (List protocol, `Option` combinators (U-CORE-2, `0da64d6`), skeletons, `System.print`) |
| **H** | Invariant verification | `verify_invariants().expect(…)` | asserts §5–6 apex table, or aborts |

### 2.1 Phase B internal steps

`create_core_classes` follows object-model.md §6's seven-step order:

1. Allocate the **8 apex rows** bare: `Object`, `Behavior`, `Class`,
   `Metaclass` + their four metaclasses.
2. Wire instance-of (`.class`) links — including the closed loop
   `Metaclass ↔ Metaclass class`.
3. Wire instance-side superclasses (`Object.superclass = None`, `Behavior→Object`,
   `Class→Behavior`, `Metaclass→Behavior`).
4. Wire metaclass-side superclasses by the parallel rule
   ([ADR-0002](../../../adr/0002-metaclass-tower-parallel-rule.md)):
   `(X class).superclass == (X.superclass) class`.
5. `make_core_class` for the ordinary rows, **in this load order**:
   `Number, String, Nil, Bool, Method, Function, Block(<Function), Symbol,
   Module, System`, then absence `Option, Some(<Option), None(<Option)`, then
   allocate the **`None` singleton**, then `List` (positioned per ADR-0020 after
   absence, before any dependant), then `Message` (U8, `< Object`, after `List`
   per its `args`-dependency note).
6. (returns `CoreClasses`; `verify_invariants` is step 7, deferred to Phase H.)

## 3. Phase-scoped invariant ledger

"Must hold" = an invariant a phase relies on as a precondition. "May be
violated" = an invariant not yet established and therefore unsafe to assert
mid-phase.

| Phase | Must already hold | May still be violated |
|---|---|---|
| A | — | everything (no classes exist) |
| B.1 | heap allocates stable `ClassId`s | all `.class`/`.superclass` links (bare rows) |
| B.2 | apex rows allocated | all superclass links |
| B.3–B.4 | apex `.class` links set | ordinary-class links (not created yet) |
| B.5 | apex fully wired (make_core_class reads `superclass.class`) | method dictionaries empty; `Some` has no field layout |
| C | tower wired (Phase B done) | no globals; no primitives; `core.ph` not run |
| D | `universe.classes` populated | primitives absent (globals point at method-less rows) |
| E | `Some` class exists | `some_new` not yet callable (installed in F) |
| F | `Some` layout set (E) — `some_new` allocates a 1-field instance | `core.ph` reopens not yet attached |
| G | **all** primitives installed (reopens call `rawAt`, `<`, `call`, …) | — (this is the last mutation) |
| H | G complete | *nothing* — full §5–6 invariant set asserted here |

**Key reading:** the object model is only *fully* consistent at the **end of
Phase H**. Code that runs earlier (e.g. a primitive during Phase F) must not
assume any invariant a later phase establishes.

## 4. The hard ordering edges (partial order)

Not all phases are freely reorderable. These four edges are load-bearing and
must be preserved by any change to `VM::new`:

1. **B → D.** `install_core` reads `self.universe.classes.*` to bind globals;
   the tower must exist first.
2. **E → F.** `some_new` (F) constructs a `Some` with one field; the `_value`
   slot layout (E) must be seeded first, or construction writes out of bounds.
   *(This is why the `Some` layout block sits between `install_core` and
   `install_primitives`, not in `create_core_classes`.)*
3. **F → G.** `core.ph` reopens invoke the primitives they wrap
   (`List.at(_)` → `rawAt(_)`, `List.each` → `Block#call`/`Number#<`). Running
   `core.ph` before the floor is installed leaves every skeleton inert — the
   historical bug U-LIST surfaced (see `vm.rs` L152–160).
4. **G → H.** `verify_invariants` asserts the quiescent shape; it must run after
   the last graph mutation (`core.ph` can attach methods but not re-wire the
   tower, so H after G is safe and final).

Everything else (e.g. the relative order of unrelated `make_core_class` calls
within B.5, beyond the ADR-0020 `List`-after-absence constraint) is free.

## 5. Self-hosting layering rule for `core.ph` (R-BOOT-2)

Within `core.ph` there is a *second* dependency order. A `.ph` method body may
only send selectors that are, at the moment `run_core_module` executes:

- **(a)** native floor primitives already installed in Phase F, or
- **(b)** `.ph` methods defined **earlier in load order** on an
  already-attached class, or
- **(c)** methods on the **same class** defined above it in the class body.

The current `core.ph` satisfies this trivially: only `List` and `Option` have
real bodies. `List`'s dependencies (`Block#call`, `Number#<`, `while` lowering,
same-class `size`/`at`) and `Option`'s (the `match` floor eliminator + `Block#call`,
all category (a)) resolve within the already-installed floor. **This acyclicity
is a requirement, not an accident** — U-CORE-2…5 must produce a topological load
order and show it is acyclic, or identify the cycle and break it with a native
seed method. `Option`'s combinators (U-CORE-2, `0da64d6`) are the first
non-`List` `.ph` bodies to test this rule, and they pass it: they send only
`match` (native, Phase F) and `Block#call` (native, Phase F), never a
later-defined class.

> **Anti-requirement:** a `.ph` method must never depend on a class defined
> *later* in `core.ph`, nor on a combinator that is itself defined in terms of
> it. The `List.each`→`Block#call` direction is fine; a hypothetical
> `Block#call` written in terms of `List` would be a cycle.

## 6. Quiescent invariant set (Phase H) & known gaps

`verify_invariants` ([`universe.rs`](../../../../phalcom-core/src/universe.rs)
L373–451) asserts, by handle identity on the live graph:

- `Object.class ≠ Object`; `Object.superclass == None`.
- `Behavior→Object`, `Class→Behavior`, `Metaclass→Behavior` (instance side).
- Every apex metaclass's `.class == Metaclass`; the closed loop
  `Metaclass.class == Metaclass class` and back.
- Metaclass-side superclasses (`Object class→Class`, `Behavior class→Object class`,
  `Class class→Behavior class`, `Metaclass class→Behavior class`).
- Parallel rule for an ordinary class (`Number.class.superclass == Object.class`).
- Every metaclass superclass chain terminates within 64 steps (cycle → error,
  not hang).

**Coverage gaps to close in later U-CORE units (feeds `invariant-requirements.md`):**

| Gap | Why it matters |
|---|---|
| No assertion that the floor census (80 bindings) is intact | floor drift is silent (see `floor-census.md` §7) |
| Parallel rule checked only for `Number` | other ordinary rows unverified in-VM (the `tests/invariants.rs` corpus covers more, but `verify_invariants` itself does not) |
| No absence-invariant check in `verify_invariants` | `Value::Nil` non-surfacing is asserted only in `tests/invariants.rs`, not at boot |
| `Some` field layout not asserted post-boot | an E/F reordering would fail silently until first `Some(_)` |

These are **not** blockers for U-CORE-0; they are the requirement list that
U-CORE units 1–6 each extend as they add classes (R-INV).

## 7. Traceability

| Section | Source |
|---|---|
| §2 phase list | `vm.rs::new` L116–168 |
| §2.1 tower steps | `universe.rs::create_core_classes` L90–185 |
| §3–§4 edge E→F | `vm.rs` L142–149 |
| §4 edge F→G rationale | `vm.rs` L152–161 |
| §6 quiescent set | `universe.rs::verify_invariants` L373–451 |
| §6 corpus | [`../../../phalcom-core/tests/invariants.rs`](../../../../phalcom-core/tests/invariants.rs) |
