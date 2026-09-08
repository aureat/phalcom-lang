# U2 — Metaclass Tower (as-built)

- **Status:** ✅ Landed — `037da3d` (`feat(u2): metaclass tower parallel rule + Behavior kernel + verify_invariants()`, 2026-07-11).
- **Realizes:** [ADR-0002](../../../adr/0002-metaclass-tower-parallel-rule.md) (metaclass parallel rule) + [ADR-0003](../../../adr/0003-introduce-behavior-kernel-class.md) (`Behavior` kernel class); spec [object-model §5](../../../spec/current/object-model.md) (metaclass tower) and [§6](../../../spec/current/object-model.md) (bootstrap order).
- **Reviewer gate:** policy = **ON** (load-bearing — can corrupt the object model), but the independent `phalcom-reviewer` pass was **explicitly SKIPPED this pass** per user instruction ("no reviewer or architect yet, just pure coding"). U2-progress.md flags a follow-up review as recommended before this is considered fully forge-verified.

## Mission
Fix the flat metaclass chain (F2): make the metaclass tower run *parallel* to the instance hierarchy so class-side (`static`) methods inherit correctly, introduce `Behavior` as the shared kernel superclass of `Class`/`Metaclass`, and add a `verify_invariants()` regression guard that asserts the apex shape on every bootstrap.

## Surface / behavior
Class-side (`static`) method inheritance now works: a subclass's static methods resolve through its parent's class-side methods. The governing rule is the **parallel rule**:

```
(X class).superclass  ==  (X.superclass) class
```

`Object class` (top of the metaclass chain) has superclass `Class`, closing the tower; `Metaclass` is an instance of itself. The kernel is the 8-row apex (`Object`, `Behavior`, `Class`, `Metaclass` + their four metaclasses) rather than the old collapsed 3-way `Metaclass`/`Class` wiring.

## Implementation
- **`phalcom-core/src/universe.rs`** — `create_core_classes` rewritten to the object-model §6 seven-step order: (1) allocate the 8 apex rows bare, (2) wire instance-of, (3) wire instance-side superclasses, (4) **wire metaclass-side superclasses by the parallel rule**, (5) create remaining core classes via `make_core_class`, (6) install primitives, (7) run `verify_invariants()`. `make_core_class` now computes `(name class).superclass = superclass.class` instead of the old hardcoded `class_class`. `CoreClasses` gained a `behavior_class: ClassId` field; `superclass`/`superclass=` primitives moved from `class_class` to `behavior_class` in `install_primitives`, so `Class` and `Metaclass` inherit them through the chain instead of each carrying a copy.
- **`Universe::verify_invariants(&self, heap: &Heap) -> Result<(), String>`** — asserts the full §5 apex table plus sanity checks: closed metaclass loop, the parallel rule on an ordinary core row, a bounded superclass-chain-terminates walk, and `Object.superclass == None`. (Later extended by U-CORE-1 to check the parallel rule for *all* ordinary rows and to audit the floor census.)
- **`phalcom-core/src/vm.rs`** — `VM::new()` calls `verify_invariants(&heap).expect(...)` right after `install_primitives` (a malformed kernel panics — it cannot run any program correctly). `VM::create_class` (the `Bytecode::Class` handler) fixed to the parallel rule for runtime user classes: a new metaclass's `.class` is `metaclass_class` (was `class_class`) and its superclass is `superclass.class` (falling back to `class_class` when `superclass` is `None`). `install_core`'s `add_class!` list gained `behavior_class` so `Behavior` is a core-module global.

## Invariants & tests
`phalcom-core/tests/invariants.rs` (10/10 passing, no `#[ignore]`d spec targets remain) exercises `verify_invariants` end-to-end through a real `VM`, asserting via **handle identity** on the live class graph:
- `verify_invariants_holds_after_bootstrap`, `metaclass_superclass_parallels_instance_superclass`, `behavior_class_exists_in_tower`, `metaclass_responds_to_superclass_via_behavior` (asserts `Behavior` defines the `superclass` getter and both `Class`/`Metaclass` inherit it via `lookup_method_in_hierarchy`, not their own copies).
- The 3 tests that encoded the old collapsed apex were **rewritten** (not just un-ignored) to assert the real 8-row shape — `metaclass_is_instance_of_metaclass_class_closing_the_loop`, `class_is_instance_of_class_class_not_metaclass_directly`, `object_class_class_is_metaclass`, `object_has_no_superclass`.

## Deviations & deferrals
- **Reviewer gate skipped** (see above) — the one explicit verification-risk deferral.
- Apex kernel rows use a **space** display name (`"Object class"`, matching the spec diagram) while runtime user classes use a **dot** (`"Object.class"`); the two naming conventions were not unified (out of scope).
- **F4** (`object_name` / instance `toString`, [ADR-0015](../../../adr/0015-object-default-tostring.md)) was scoped **out** of U2 — see [deferred-work](../../../spec/current/deferred-work.md).
- `clippy` and the strict byte-identical golden ceremony were not run this pass (golden tests did pass inside the full `cargo test -p phalcom-core`).
- ADR fold-ins landed here: ADR-0002 gained a "Superseded (U2)" pointer note (Rc `new_cyclic` → ADR-0009 handle-patching); ADR-0003's "Open question" note replaced with an "Implementation note (U2)" confirming Q11 resolved.

## Sources
- forge: [`u0-state.md`](../../archive/phase2/u0-state.md). Per-unit planning record (`U2-plan.md`, `U2-progress.md`) folded into this spec; see git history.
- code: `phalcom-core/src/universe.rs` (`create_core_classes`, `make_core_class`, `verify_invariants`), `phalcom-core/src/vm.rs` (`VM::new`, `VM::create_class`), `phalcom-core/tests/invariants.rs`.
- ADRs: [0002](../../../adr/0002-metaclass-tower-parallel-rule.md), [0003](../../../adr/0003-introduce-behavior-kernel-class.md).
- landing: `037da3d`.
