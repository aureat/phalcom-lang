# Core Library Specification (`docs/spec/core/`)

This directory holds the specification for Phalcom's **core library**: the
kernel classes, singletons, and protocols — part VM-native, part written in
Phalcom (`core.ph`) — that must exist before any user program can run, together
with the bootstrap procedure that assembles them into a consistent object graph
at VM startup.

It is the executable-facing companion to the design specs one level up
([`../object-model.md`](../object-model.md),
[`../values-and-absence.md`](../values-and-absence.md),
[`../functions.md`](../functions.md),
[`../control-flow.md`](../control-flow.md)) and to the experimental
[`../experimental/bootstrapping-and-self-hosting.md`](../experimental/bootstrapping-and-self-hosting.md).
Where those describe the *target* model, the documents here fix the **as-built
baseline** and the **rules any future core-library change must obey**.

**Start here for a class-by-class view:** [`core-classes.md`](./core-classes.md)
is the consolidated reference — for each kernel class, its role, structure,
interface (floor + `.ph` methods), governing ADR, and landed/pending status. It
reads *by class*; the census/catalog/object-model docs read by primitive, by
delta, and by target respectively (the axis table at the top of that file).

## Provenance

Every table here is derived from ground-truth source, not aspiration:

| Fact | Authoritative source |
|---|---|
| Native primitive set | [`phalcom-core/src/universe.rs`](../../../../phalcom-core/src/universe.rs) → `install_primitives` |
| Bootstrap sequence | [`phalcom-core/src/vm.rs`](../../../../phalcom-core/src/vm.rs) → `VM::new` / `install_core` |
| Tower construction | `universe.rs` → `create_core_classes` / `make_core_class` |
| Surface protocol (`.ph`) | [`phalcom-core/core/core.ph`](../../../../phalcom-core/core/core.ph) |
| Selector encoding | [`phalcom-core/src/primitive/mod.rs`](../../../../phalcom-core/src/primitive/mod.rs) (`make_signature`, `Sig`) + [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) |
| Freeze policy | [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md) |

## Baseline & drift policy

These docs reconcile against a **live tree**, so staleness is kept explicit
rather than silent. **This section is the single source of truth for the
baseline pin.** Every other doc in this directory inherits it and carries only a
one-line back-reference here, rather than restating the landing history — so the
pin is updated in exactly one place.

**Current baseline — post-U-ERR (core-library track U-CORE-1..6 complete, plus
the post-U-CORE floor amendments U-COLLTYPES and U-ERR).**

| Fact | Value |
|---|---|
| Last floor-affecting commit | this unit (**U-ERR**) — `Block#on(_,_)`/`Block#ensure(_)`, floor **109 → 111** (track total **73 → 111**: ADR-0023/0028/0036/0037 to 88, ADR-0039's three U-COLLTYPES phases to 109, ADR-0038 to 111) |
| Primitive floor | **111** installed `(class, selector)` bindings · **96** distinct native fns · **21** floor-carrying classes (of **27** named kernel classes) · **7** sacred selectors — see [`floor-census.md`](./floor-census.md) for the full enumeration and amendment-by-amendment history |
| Decisions closed by ADR | [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md) (`hash` + kernel-reflection floor), [ADR-0024](../../../adr/0024-numeric-surface-split-int-float-and-division.md) (Int/Float split — exact bignum `Int` + `Float`, `/` true ÷, `~/` floor ÷), [ADR-0025](../../../adr/0025-external-internal-parameter-names.md) (external/internal param names), [ADR-0026](../../../adr/0026-class-hierarchy-mutability.md) (hierarchy mutability — methods open, reparent sealed), [ADR-0027](../../../adr/0027-modules-as-files-with-public-by-default-imports.md) (modules-as-files, public-by-default imports), [ADR-0037](../../../adr/0037-amend-floor-admit-error-root.md) (`Error#message`/`raise`), [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md) (`Map`/`Set`/`Tuple`/`Range`, +21), [ADR-0038](../../../adr/0038-amend-floor-admit-block-on-ensure.md) (`Block#on`/`ensure`, +2 — this unit) |

**Landing history** (chronological; only U-CORE-1 added a floor binding). U8
(`Object` reflective surface + the `Message` class), U9 (variadics), **U-CORE-2
core bulk** (`0da64d6` — Bool `Some`-lift + core `Option` combinators), U10
(non-local return), U-LEX (surface syntax, `\(expr)` interpolation), U-STD
(Option/List transform combinators — the group re-scoped out of U-CORE-2), U11
(`True`/`False` singleton subclasses of `Bool`), then **U-CORE-1** (`03764e3` —
the sole floor bump, `hash`×5 + `Behavior#name`/`methods`, +7 → **80**).
Kernel-class count moved 19 → 21 (U11's `True`/`False`) with no floor delta —
"classes added" does **not** imply "bindings added". ADR-0024–0027 landed at
`0b21e60`, so the core-library specs no longer treat Int/Float, param labels,
hierarchy mutability, or modules as undecided.

**Drift policy.**

- Each data-bearing doc inherits the pin above. When a forge unit lands new floor
  primitives, kernel classes, or `.ph` protocol, re-baseline
  [`floor-census.md`](./floor-census.md) and [`catalog-delta.md`](./catalog-delta.md)
  and bump the table above — in one place.
- The floor count is **machine-checked**, not a manual checksum: the R-INV-0.1
  audit (`floor_census_matches_installed_bindings` in
  [`tests/invariants.rs`](../../../../phalcom-core/tests/invariants.rs))
  reconstructs the installed set from a live `VM::new()` and fails on drift.
- Recommended cadence: a U-CORE-0 "refresh" pass before each U-CORE-N unit
  starts, so that unit plans against ground truth. (The 65→73 jump at the U-LIST
  spine and the 73→80 jump at U-CORE-1 are the two re-baselines this policy
  exists to catch.)

## Deliverables

This is **U-CORE-0** — the reconcile-and-census unit that must land before any
core-library *implementation* unit, because Q1–Q3 (see the requirements
analysis) all hang off it. Status of the planned set:

| # | Deliverable | File | Status |
|---|---|---|---|
| 1 | **Primitive floor census** (ADR-0019 audit) | [`floor-census.md`](./floor-census.md) | ✅ landed |
| 2 | **Bootstrap phase table** (phase-scoped invariant ledger) | [`bootstrap-phases.md`](./bootstrap-phases.md) | ✅ landed |
| 3 | Sacred-selector set (R-SACRED) | folded into `floor-census.md` §5 | ✅ landed |
| 4 | **Baseline delta table** (catalog × {native, `.ph`, pending}) | [`catalog-delta.md`](./catalog-delta.md) | ✅ landed |
| 5 | **`_pending` → active retirement map** | [`pending-retirement.md`](./pending-retirement.md) | ✅ landed |
| 6 | **Invariant requirements per unit** | [`invariant-requirements.md`](./invariant-requirements.md) | ✅ landed |
| 7 | **"Must not preclude" forward-compat checklist** | [`forward-compat.md`](./forward-compat.md) | ✅ landed |

**U-CORE-0 is complete (7/7).** The gating decisions (Q1 `hash`, Q2 errors, Q4
prelude, Q5 collections, §4.1 `Method` superclass, §4.4 per-type `toString`) are
ruled in [`decisions.md`](./decisions.md).

## Implementation specs (U-CORE-1…6)

Each downstream unit has a **dispatch-ready implementation spec** a
`phalcom-implementer` can execute — grounded in the U-CORE-0 docs above, citing
spec §/ADR, the native-vs-`.ph` split, the `_pending` tests it flips
([`pending-retirement.md`](./pending-retirement.md) §4), the invariants it adds
([`invariant-requirements.md`](./invariant-requirements.md)), and a
[`forward-compat.md`](./forward-compat.md) "must not preclude" check. All six are
authored:

| Unit | Spec | Native floor Δ (from 73) | Flips directly |
|---|---|---|---|
| U-CORE-1 kernel reflection | [`U-CORE-1-implementation-spec.md`](../../../forge/units/U-CORE-1/as-built.md) | **+7 — as-built/landed** (`hash`×5 + `Behavior#name`/`methods`, 73 → 80); `isA` is `.ph`; `Method < Function` re-parent applied | `metaclass_is_a` (flipped) |
| U-CORE-2 absence + Boolean | [`U-CORE-2-implementation-spec.md`](../../../forge/units/U-CORE-2/as-built.md) | **0** (bulk landed `0da64d6`; verify/harden) | — |
| U-CORE-3 callables/Block | [`U-CORE-3-implementation-spec.md`](../../../forge/units/U-CORE-3/as-built.md) | **+5** (`methodFor`/`invokeOn`/`bind`/`signature`/`holder`) | — (all U-LEX-gated) |
| U-CORE-4 value classes | [`U-CORE-4-implementation-spec.md`](../../../forge/units/U-CORE-4/as-built.md) | **+1** (`Number#toString`) | `absence_option_none`, `absence_var_defaults_to_none`, `binding_var_uninitialized` |
| U-CORE-5 collection contract | [`U-CORE-5-implementation-spec.md`](../../../forge/units/U-CORE-5/as-built.md) | **0** (contract + `.ph` `List#==`/`!=`) | — (enables reduce/Map/Set) |
| U-CORE-6 errors | [`U-CORE-6-implementation-spec.md`](../../../forge/units/U-CORE-6/as-built.md) | **+2** (`Error#message`/`raise`) | — (needs error-syntax) |

These refine — and in places subsume — the older, coarser forge planning for
`U-STD` (base-surface growth) and `U11` (Bool tower) tracked in
[`../../forge/archive/phase2/PHASE2-INDEX.md`](../../../forge/archive/phase2/PHASE2-INDEX.md).

### Cross-spec integration notes (read before dispatching any unit)

The six specs were authored in parallel; each states its delta from the **same
base of 73**. An implementer must reconcile the following across units:

1. **Floor deltas are cumulative.** U-CORE-1's **+7** has landed, so the floor is
   now **80** (the new base); if the rest land, it reaches `80 + 5 + 1 + 2 = 88`.
   Each remaining spec's "73 → N" was authored as a *delta from the old base of
   73*, not a running total. The floor-census audit (R-INV-0.1) must bump in
   lockstep with each unit's installs.
2. **Four ADR-0019 amendments are folded into one omnibus
   [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)**
   (U-CORE-1 `hash`/`Behavior`, U-CORE-3 Method reflection, U-CORE-4
   `Number#toString`, U-CORE-6 `Error#message`/`raise`) — **ratified Accepted**
   rather than four competing amendments to ADR-0019. Each named primitive is
   *installed* by its owning unit when that unit lands; ADR-0023 only admits them
   to the floor in principle. (The individual specs were authored when 0021 was
   the highest ADR and some say "ADR-0022"; 0022 was taken by U-LEX's
   string-interpolation ADR, so **0023** is correct — fix any spec that still
   says 0022.)
3. **U-CORE-1 lands first** — it stands up the invariant substrate (R-INV-0.1…0.4)
   the other units extend, and owns the `isA`+`hash` that **U-CORE-5 hard-depends
   on** (its structural `List#==` type-guard needs `isA`).
4. **U-CORE-1 and U-CORE-3 both edit `create_core_classes` and both do the §4.1
   `Method < Function` re-parent** ("whichever lands first; the other asserts") —
   they **must not share a parallel wave**.
5. **U-CORE-4 and U-STD both touch `Value::to_string` / `core.ph`** — coordinate
   the `core.ph` edit order (never co-schedule two `core.ph` editors). U-CORE-4
   also re-pins **9 currently-green fixtures'** `.expected` (substrate → pretty
   output) in the same change.
6. **Reflection/error fixtures are U-LEX-gated:** U-CORE-3 and U-CORE-6's pending
   flips need U-LEX surface syntax (`#…`, `::`, error sugar), so each sets its
   acceptance on a **new** unit-local fixture in already-supported syntax
   (pending-retirement §4).

## Status

**U-CORE-0 is complete (7/7)** and the gating decisions are ruled
([`decisions.md`](./decisions.md)); all six U-CORE-1…6 implementation specs are
authored (see the table above, now in
[`../../../forge/units/`](../../../forge/units/)), and **ADR-0023 is ratified**
(cross-spec note 2 above) — the floor gate is clear.

**Implementation is underway.** **U-CORE-1 has landed** (`03764e3`/`b1109c2` —
`Object#hash` + per-immediate hashes, `Behavior#name`/`methods`, `isA`, and the
`Method < Function` re-parent; floor 73 → **80**); **U-CORE-2's core bulk landed**
earlier (`0da64d6`). The **track head is U-CORE-3** (callables/Block), with
U-CORE-4/5/6 to follow. The next job is to dispatch the remaining units to
`phalcom-implementer` in dependency order — re-grounded against the post-U-CORE-1
baseline above before dispatch, honoring the wave constraints in the cross-spec
notes.
