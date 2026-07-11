# Invariant Requirements (U-CORE-0)

> **Status:** Normative. Enumerates the invariant assertions each U-CORE unit
> must **add** as it lands, and where each belongs — the in-VM boot check
> ([`verify_invariants`](../../../phalcom-core/src/universe.rs) L404, Phase H) or
> the external corpus ([`tests/invariants.rs`](../../../phalcom-core/tests/invariants.rs)).
> It converts the "coverage gaps" ledger in [`bootstrap-phases.md`](./bootstrap-phases.md)
> §6 into a per-unit checklist (R-INV) with acceptance-grade assertions.

> **Baseline:** HEAD `76b5f35`; last code-affecting commit `0da64d6`.

## 1. Two invariant surfaces (where an assertion goes)

| Surface | Runs | Cost of a violation | Use for |
|---|---|---|---|
| **`verify_invariants(&heap)`** — `universe.rs` L404, called at **Phase H** of `VM::new` and `.expect()`-ed | **every VM start**, before any user code | VM aborts at boot — impossible to run a program on a broken tower | invariants that must hold for the VM to be *sound at all*: tower shape, fixed-slot layouts, absence non-surfacing |
| **`tests/invariants.rs`** — `#[test]` corpus | `cargo test` only | red test | invariants that are expensive to check on every boot, or that assert *user-observable* behavior (a `.ph` snippet's output), or exhaustive sweeps across many classes |

**Rule of thumb:** if a violation would let a *malformed VM boot and mislead a
user*, it belongs in `verify_invariants`. If it is a regression guard on a
*behavior*, it belongs in the corpus. Several U-CORE invariants want **both** —
a cheap structural check at boot plus a behavioral guard in the corpus.

## 2. As-built coverage (the starting point)

`verify_invariants` currently asserts (by handle identity on the live graph):

- `Object.class ≠ Object`; `Object.superclass == None`.
- Instance-side: `Behavior→Object`, `Class→Behavior`, `Metaclass→Behavior`.
- Every apex metaclass `.class == Metaclass`; the closed loop
  `Metaclass.class == Metaclass class` and back.
- Metaclass-side supers: `Object class→Class`, `Behavior class→Object class`,
  `Class class→Behavior class`, `Metaclass class→Behavior class`.
- **Parallel rule for `Number` only** (`Number.class.superclass == Object.class`).
- Every metaclass superclass chain terminates within 64 steps.

`tests/invariants.rs` currently covers (a richer, behavioral set):

| Test | Guards |
|---|---|
| `surface_nil_is_unreachable_from_user_code` | Invariant 4 — no surface `nil` |
| `sentinel_surfaces_to_none_and_never_survives_as_nil` | `Value::Nil` → `None` at the boundary |
| `expression_result_absence_surfaces_to_none` | empty block/method result is `None`, not the sentinel |
| `some_construction_never_wraps_the_sentinel` | `Some(_)` rejects the private `Nil` |
| `some_can_wrap_the_none_singleton` | `Some(None)` is legal and distinct from `None` |
| `verify_invariants_holds_after_bootstrap` | Phase H passes on a clean boot |
| `metaclass_superclass_parallels_instance_superclass`, `…parallels_instance_superclass` (user class), `core_classes_have_correct_metaclass_and_superclass` | parallel rule across **more** classes than `verify_invariants` |
| `behavior_class_exists_in_tower`, `metaclass_responds_to_superclass_via_behavior`, `…closing_the_loop`, `class_is_instance_of_class_class_not_metaclass_directly`, `object_class_class_is_metaclass`, `object_has_no_superclass`, `walking_metaclass_superclass_chain_terminates` | tower apex |
| `subclass_field_offset_stability`, `subclass_static_field_offset_stability` | ADR-0011/0017 slot layout |

## 3. The four U-CORE-0-mandated gaps (close before/with U-CORE-1)

These four are named explicitly by the U-CORE-0 charter. They are **not** owned
by a single downstream unit — they harden the floor the whole roadmap builds on,
so they land with U-CORE-1 (the first implementation unit) or as a standalone
"invariants" slice.

### R-INV-0.1 — Floor-census audit (assert 73 bindings)
- **Where:** `tests/invariants.rs` (too expensive/reflective for boot).
- **Assertion:** reconstruct the installed `(class, selector)` set from a live
  `VM::new()` and assert it **equals** the census in [`floor-census.md`](./floor-census.md)
  — count **= 73**, and ideally the exact set, not just the cardinality.
- **Why:** floor drift is otherwise silent (floor-census §7); a stray
  `primitive!` or a dropped binding is an ADR-0019 violation that no test catches
  today. This turns the manual checksum into a red test.
- **Note:** the assertion must count **bindings**, not macro-call sites (`call`
  expands to 5 arities × 2 classes; see floor-census §1.1). Prefer enumerating
  the method dictionaries of the 16 floor-carrying classes over counting source.

### R-INV-0.2 — Parallel rule for **all** ordinary rows
- **Where:** extend `verify_invariants` (cheap; strengthens the boot check) **and**
  keep the corpus sweep.
- **Assertion:** for every ordinary kernel class `X` in `CoreClasses`
  (`Number, String, Bool, Symbol, Method, Function, Block, Option, Some, None,
  List, Module, System, Message, Nil`), `X.class.superclass == X.superclass.class`
  (ADR-0002). Today `verify_invariants` checks only `Number`.
- **Why:** bootstrap-phases §6 — the parallel rule is the one tower invariant a
  `make_core_class` reordering can silently break for a class the corpus does not
  happen to name.

### R-INV-0.3 — Absence non-surfacing at **boot**
- **Where:** add to `verify_invariants` (currently corpus-only).
- **Assertion:** the `None` singleton is a live value distinct from the private
  `Value::Nil` sentinel, and the `None` **global** resolves to that singleton
  *value*, not the `None` **class** (values-and-absence.md §3.1; the `core.ph`
  `DefineGlobal`-clobber hazard). A boot-time check catches a future `core.ph`
  edit that re-adds `class None {}` and silently rebinds the global.
- **Why:** Invariant 4 is currently asserted only in `tests/invariants.rs`, so a
  malformed VM that leaks the sentinel would still **boot** and only fail under
  test. This is exactly the "malformed VM misleads a user" case §1 reserves for
  `verify_invariants`.

### R-INV-0.4 — Fixed-slot layout of `Some` and `Message`
- **Where:** `verify_invariants` (structural, cheap).
- **Assertion:** `Some` has exactly **one** field (`_value` at slot 0) and
  `Message` exactly **four** (`selector`, `name`/target, `labels`, `args`), as
  stamped in `VM::new` (Phase E) — ADR-0011. Assert the field **count** on each
  class object post-boot.
- **Why:** bootstrap-phases §6 — an E/F reordering (layout seeded after the
  primitive that writes it) would corrupt the first `Some(_)` / first dNU miss
  and fail *silently* until that path runs. A boot assertion fences the E→F edge.

## 4. Per-unit invariant requirements (R-INV-N)

Each row is what that unit's implementation spec must add under **"invariants
this unit adds."** "H" = `verify_invariants` (boot); "C" = corpus.

### U-CORE-1 — kernel reflection (`hash`, `isA(_)`, `Behavior`/`Class` reflection)
| # | Invariant | Where |
|---|---|---|
| 1.1 | Closes R-INV-0.1…0.4 (this is the first impl unit — it stands up the audit substrate). | H + C |
| 1.2 | `isA(_)` is reflexive and superclass-closed: `x.isA(x.class)` is `true`; `x.isA(Object)` is `true` for every `x`; `x.isA(C)` ⇔ `C` is on `x.class`'s superclass chain. | C |
| 1.3 | `hash` is **consistent with `==`**: `a == b` ⇒ `a.hash == b.hash` (the Map/Set precondition, object-model §4). Assert over immediates (`Number`, `String`, `Symbol`, `Bool`) and identity objects. | C |
| 1.4 | `hash` is **stable** across calls on the same receiver within a run. | C |
| 1.5 | If `Method` is re-parented under `Function` (§4.1 ruling), the parallel rule (R-INV-0.2) still holds for `Method`, and `Method` responds to the `Function` call-protocol selectors. | H + C |
| 1.6 | `Behavior#name` / method-dictionary reflection returns the class's own data without mutating it (a reflective read is side-effect-free). | C |

### U-CORE-2 — absence + Boolean (residue; core landed `0da64d6`)
| # | Invariant | Where |
|---|---|---|
| 2.1 | **Fast-path ≡ deopt-path for the Some-lift** (ADR-0018): an inlined `ifTrue { A }` and the same site after a sacred-selector override both yield `Some(A)` on the taken arm and `None` on the untaken arm. This is the load-bearing check for the `WrapSome` op. | C |
| 2.2 | **Pop-context elision is observationally invisible:** discarding an `ifTrue`/`ifFalse` result (statement position) produces identical program output to using it — the elided `Some` allocation never changes semantics. | C |
| 2.3 | `ifTrue(_, ifFalse)` and `and`/`or` still return **raw** values (not `Some`-lifted) — the divergence fix is one-armed only (catalog §4.2). | C |
| 2.4 | Every `Option` combinator routes through `match` (no combinator peeks at a variant tag): `isSome`/`isNone`/`ifNone`/`orElse` on a user-subclassed `Option` respect an overridden `match`. | C |

### U-CORE-3 — callables / Block (surface layer; mechanism landed U4/U10)
| # | Invariant | Where |
|---|---|---|
| 3.1 | `Block < Function` and (per §4.1) `Method < Function`; all three respond to `arity`/`name`; the parallel rule holds for each (R-INV-0.2 extended). | H + C |
| 3.2 | Non-local `return` from a block whose home frame is dead raises `DeadFrameError`, not a silent wrong-value (consumes U10; guard it survives the U-CORE-3 surface additions). | C |
| 3.3 | `Method#invokeOn(recv, args)` and `bound.call(args)` produce identical results for the same `(method, recv, args)` — binding is transparent. | C |
| 3.4 | `arity` reported by a `Method`/`Block` matches the arity the dispatcher requires (an arity-mismatch call is an `ArgumentError`, not a truncation). | C |

### U-CORE-4 — value classes (`toString` overrides, richer value protocol)
| # | Invariant | Where |
|---|---|---|
| 4.1 | **`toString` message vs `Value::to_string` print-path stay consistent:** for every value type, `x.toString` (the message) equals what `System.print(x)` renders (catalog §4.4). Assert for `Number`, `String`, `Symbol`, `Bool`, `None`, `Some(_)`, `List`. | C |
| 4.2 | The `Object#toString` default (`"<ClassName>"`, ADR-0015) is **preserved for user classes** — a per-type override on `Number` must not change what a user `Foo` instance prints. | C |
| 4.3 | `None.toString == "None"` and `Some(x).toString == "Some(" + x.toString + ")"` — the fixtures `absence_option_none` / `absence_var_defaults_to_none` / `binding_var_uninitialized` go green (pending-retirement §4). | C |
| 4.4 | Value `toString` is **total** (never raises) and never surfaces the `Nil` sentinel (Invariant 4 held through the new overrides). | C |

### U-CORE-5 — collection protocol **contract** (not new classes — ADR-0020)
| # | Invariant | Where |
|---|---|---|
| 5.1 | The contract is a set of selector + law assertions that **`List` already satisfies** (it is the reference implementation): `size ≥ 0`, `at(i)` for `0 ≤ i < size` total, `add` grows `size` by 1. Encode as a reusable conformance corpus keyed by "the collection under test." | C |
| 5.2 | Iteration order is deterministic and `each` visits exactly `size` elements once (the precondition every derived `map`/`reduce`/`filter` — U-STD — will rely on). | C |
| 5.3 | Whatever equality the contract mandates (see the Q5 ruling in [`decisions.md`](./decisions.md)) is **reflexive/symmetric/transitive** and consistent with `hash` (feeds R-INV-1.3 for `Map`/`Set` keys). | C |
| 5.4 | Adding a collection class later (Map/Set/Tuple/Range) is checked *against this corpus*, so the contract is the gate, not each class's ad-hoc tests. | C |

### U-CORE-6 — Error root + wire dNU → `MessageNotUnderstood`
| # | Invariant | Where |
|---|---|---|
| 6.1 | `MessageNotUnderstood < Error < Object`; the tower parallel rule holds for the new error rows (R-INV-0.2 extended). | H + C |
| 6.2 | A genuine miss (no method, dNU not overridden) raises a **surface** `MessageNotUnderstood` carrying the reified `Message` (census §2.14), **not** the native `RuntimeError` — and the raised object `isA(Error)`. | C |
| 6.3 | Only `Error` subclasses are throwable (ADR-0008): `throw 42` is rejected; `throw someError` unwinds. | C |
| 6.4 | An overriding `doesNotUnderstand(_)` (proxy) still **intercepts** before `MessageNotUnderstood` is raised — wiring the default raise must not break the U8 hook. | C |
| 6.5 | The floor census (R-INV-0.1) is updated in lockstep if U-CORE-6 adds any native error-raising primitive (likely an ADR-0019 amendment — see [`decisions.md`](./decisions.md) Q2). | C |

## 5. Sequencing note

R-INV-0.1…0.4 are a **hard prerequisite** for trusting every later unit's
invariant additions: without the floor-census audit, a unit that accidentally
adds a native primitive (violating ADR-0019) passes its own tests. Land the four
0.x assertions **first** (with U-CORE-1, or as a standalone slice ahead of it),
then each unit extends the parallel-rule sweep (0.2) and the census set (0.1) as
it adds classes.

## 6. Traceability

| Claim | Source |
|---|---|
| `verify_invariants` current assertions | `universe.rs` L404–480 |
| Corpus current tests | `tests/invariants.rs` (test-fn inventory) |
| Coverage-gap ledger | [`bootstrap-phases.md`](./bootstrap-phases.md) §6 |
| Floor count = 73 | [`floor-census.md`](./floor-census.md) §1.1 / §7 |
| `Some`/`Message` fixed slots | [`floor-census.md`](./floor-census.md) §2.8/§2.14; `vm.rs::new` (Phase E) |
| `hash`/`isA`/`==` consistency | [`object-model.md`](../object-model.md) §4 (Map/Set), §8 (protocol) |
| Some-lift fast≡deopt | [ADR-0018](../../adr/0018-sacred-selector-inliner-and-override-guard.md) amendment; `0da64d6` |
| Error mechanism | [ADR-0008](../../adr/0008-layered-exceptions-and-result.md) |
