# U11 — Bool tower: abstract `Bool` + `True`/`False` singletons (as-built)

- **Status:** ✅ Landed — `23cafe2` (Rust wiring) → `96b440c` (`core.ph` skeletons + fixtures) → `c0e1066` (docs: STATE + object-model §3/§4 reconciliation + the implementation spec). In-tree on `main`, no worktree.
- **Realizes:** [ADR-0004](../../../adr/0004-boolean-as-abstract-bool-with-true-false.md) (Bool as abstract with True/False); spec [object-model §3](../../../spec/current/object-model.md) (value representation), [§4](../../../spec/current/object-model.md) (core class catalog), [§5](../../../spec/current/object-model.md) (metaclass tower). Governed by the U11 implementation spec (`U11-implementation-spec.md`, superseding `U11-plan.md`; folded into this spec, see git history). Consumes U5's sacred-selector inliner ([ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md)) and U-CORE-2's `ifTrue`/`ifFalse` `Some`-lift.
- **Reviewer gate:** **OFF** per STATE.md policy — self-verified (proved `true.class == True` **and** `ifTrue`→`Some` / `and`/`or` short-circuit through the split).

## Mission
Split the Bool type per ADR-0004: make `Bool` **abstract** with concrete **singleton
subclasses `True`/`False`**, so `true.class == True` and `false.class == False`, while the
six sacred control selectors stay native primitives on `Bool` and are reached by inheritance.
Tiny, purely additive, mostly Rust — **zero new floor primitives** (census stays 80), **no
new `Value` variant** (`Value::Bool(b)` unchanged).

## Surface / behavior
- `true.class == True`, `false.class == False`; `True.superclass == Bool` and
  `False.superclass == Bool`. `Bool` is abstract — never the direct class of any value
  (`true.class == Bool` is `false`).
- The six sacred selectors (`not`, `and`, `or`, `ifTrue`, `ifFalse`, `ifTrue:ifFalse:`) still
  work on `true`/`false` receivers, resolving through `True`/`False` → inherited from `Bool`.
- Value rendering is unchanged: `System.print(true)` → `true`, so every existing golden stays
  byte-identical.

```phalcom
System.print(true.class == True)        // → true
System.print(True.superclass == Bool)   // → true
System.print(true.not)                  // → false  (inherited from Bool)
System.print(true.ifTrue { 42 }.isSome) // → true   (U-CORE-2 Some-lift survives the split)
```

## Implementation
- **`value.rs::class` — the single behaviour-changing edit.** The `Value::Bool` arm now
  selects the class by the payload — allocation-free, a plain `ClassId` field read on the hot
  dispatch path:
  ```rust
  Value::Bool(b) => {
      if *b { vm.universe.classes.true_class }
      else  { vm.universe.classes.false_class }
  }
  ```
  (was `Value::Bool(_) => vm.universe.classes.bool_class`).
- **`universe.rs`** — `create_core_classes` adds `True`/`False` rows immediately after
  `bool_class` (both super `bool_class`, via `make_core_class`, so `Bool` becomes the abstract
  parent); two `pub true_class` / `pub false_class` fields on the `CoreClasses` struct + the
  returned literal.
- **`vm.rs::install_core`** — `add_class!(true_class)` / `add_class!(false_class)` bind the
  `"True"`/`"False"` globals and insert them into `self.classes` (so the `core.ph` reopens
  resolve the bootstrapped rows rather than forging shadows).
- **`primitive/mod.rs`** — `ClassName::True = "True"` / `ClassName::False = "False"` consts
  (capitalized class names, distinct from `ObjectName::True/False` = the value spellings
  `"true"`/`"false"`).
- **`core.ph`** — empty `class True {}` / `class False {}` reopens for surface-visibility
  parity; harmless no-ops (their globals name the class objects, so `DefineGlobal` re-emits
  the same binding — unlike `None`, whose global names a singleton *value*). No method bodies.
- **Untouched (spec-confirmed):** `boolean.rs`, `primitive/boolean.rs` (sacred primitives +
  `bool_class_new` + the DEFERRED `println!`s stay verbatim), `install_primitives`'s Bool
  block, and `verify_invariants` (**re-run, not edited** — U-CORE-1's domain).

## Invariants & tests
- **Sacred inliner is transparent to the split (why KEEP is safe).** `Bytecode::GuardBool`
  keys on the `Value::Bool` **representation** + the `bool_sacred_pristine` epoch, *never* on
  class identity, so the class split is invisible to the fast path; the deopt path resolves
  through `True`/`False` → inherits `Bool`'s primitive. The epoch hook `note_method_installed`
  stays hard-keyed to `bool_class`; because **no methods land on `True`/`False`**, that key is
  never wrong.
- **`verify_invariants` re-passes** with the two new rows unmodified — `make_core_class` wires
  each metaclass by the same ADR-0002 parallel rule; floor census stays at 80 bindings.
- **New `booleans/` PASS fixtures** (label already active): `bool_class_identity` (class
  identity + `superclass`), `bool_sacred_through_split` (`not`/`and`/`or` + rendering through
  the split), `bool_iftrue_option` (U-CORE-2 `Some`-lift survives). The two existing
  short-circuit fixtures stay byte-identical.
- **Green gate:** `verify.sh` exit 0; `cargo doc --workspace --no-deps` no new warnings;
  goldens byte-identical.

## Deviations & deferrals
- **D1 = KEEP (resolved by the implementation spec §0.1, no user escalation).** The six sacred
  selectors stay native primitives on abstract `Bool` and are **inherited** by `True`/`False`,
  rather than ADR-0004's literal per-subclass bodies. MOVE was rejected: it would force
  extending `note_method_installed` to watch `true_class`/`false_class` (else a `True` override
  is silently ignored — the exact unsoundness ADR-0018 prevents), re-prove the U-CORE-2
  `Some`-lift ≡ inliner parity, and add floor bindings breaking the 80-binding census. ADR-0004's
  literal per-subclass form, if ever wanted, is a separate later unit (soft gate BD-U11-D1).
- **Singletons without a heap instance:** `True`/`False` are singleton *classes* by virtue of
  `Value::Bool` having exactly two immediate inhabitants — no heap singleton is needed (unlike
  `None`), and `==` remains value equality.
- **Doc-only reconciliation:** object-model.md §3/§4 updated to make `True`/`False`
  surface-visible and `Bool` abstract (`c0e1066`) — resolving the pre-existing drift vs ADR-0004.
- **Scheduling constraint (BD-U11-SCHED):** U11's Rust write-set (`universe.rs::create_core_classes`,
  `CoreClasses`, `core.ph` Bool area) collides with U-CORE-1/4/6; it must not be co-scheduled in
  the same parallel wave. Landed first as recommended.

## Sources
- Forge: `U11-implementation-spec.md`, `U11-plan.md` (folded into this spec; see git history), [STATE.md](../../archive/phase2/u0-state.md) "U11 — LANDED".
- Commits `23cafe2`, `96b440c`, `c0e1066`.
- Code: `phalcom-core/src/{value,universe,vm}.rs`, `phalcom-core/src/primitive/mod.rs`,
  `core/core.ph`; docs `docs/spec/object-model.md` §3/§4.
