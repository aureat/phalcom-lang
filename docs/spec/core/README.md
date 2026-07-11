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

## Provenance

Every table here is derived from ground-truth source, not aspiration:

| Fact | Authoritative source |
|---|---|
| Native primitive set | [`phalcom-core/src/universe.rs`](../../../phalcom-core/src/universe.rs) → `install_primitives` |
| Bootstrap sequence | [`phalcom-core/src/vm.rs`](../../../phalcom-core/src/vm.rs) → `VM::new` / `install_core` |
| Tower construction | `universe.rs` → `create_core_classes` / `make_core_class` |
| Surface protocol (`.ph`) | [`phalcom-core/core/core.ph`](../../../phalcom-core/core/core.ph) |
| Selector encoding | [`phalcom-core/src/primitive/mod.rs`](../../../phalcom-core/src/primitive/mod.rs) (`make_signature`, `Sig`) + [ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md) |
| Freeze policy | [ADR-0019](../../adr/0019-freeze-vm-blessed-primitive-floor.md) |

## Baseline & drift policy

These docs reconcile against a **live tree**, so staleness is kept explicit
rather than silent: each data-bearing doc pins the commit it reflects.

- **Current baseline:** HEAD `76b5f35`; last code-affecting commit `0da64d6`
  (**U-CORE-2 partial** — see below). Folds in **U8** (`Object` reflective
  surface + the `Message` class), **U9** (variadics), and **U-CORE-2's** Bool
  half-Option fix + core `Option` combinators. Floor count is **unchanged at 73**
  across this bump: `0da64d6` added a `wrap_some` helper and a `WrapSome`
  bytecode op, neither of which is a bound floor primitive.
- **U-CORE-2 already partly landed** (`0da64d6`): `Bool#ifTrue`/`ifFalse` now
  `Some`-lift the taken arm (closing catalog-delta §4.2), the sacred inliner
  `Some`-lifts in lockstep via a new `WrapSome` op (ADR-0018 amendment), and
  `core.ph`'s `Option` reopen gained `ifNone`/`orElse`/`isSome`/`isNone`. The
  transform/extract combinators (`ifSome`/`map`/`unwrapOr`/…) are re-scoped to
  U-STD (catalog-delta §2.2); the U-CORE-2 implementation spec covers only the
  residue (absence invariants, `None`/`Some` surface `toString`).
- When a forge unit lands new floor primitives, kernel classes, or `.ph`
  protocol, re-baseline [`floor-census.md`](./floor-census.md) and
  [`catalog-delta.md`](./catalog-delta.md) and bump the pin. Recommended cadence:
  a U-CORE-0 "refresh" pass before each U-CORE-N unit starts, so that unit plans
  against ground truth.
- The 65→73 binding jump between the initial commit (`a2dd17b`, the U-LIST spine)
  and the U9 baseline — U8/U9 landed concurrently mid-session — was the first
  such re-baseline, and the reason for this policy; the `c9805d0`→`0da64d6` bump
  (U-CORE-2, no floor delta) is the second.

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

Downstream implementation units (U-CORE-1 kernel reflection, U-CORE-2
absence+Boolean, U-CORE-3 callables, U-CORE-4 value classes, U-CORE-5
collection contract, U-CORE-6 errors) each get a dispatch-ready implementation
spec (`U-CORE-N-implementation-spec.md`). These refine — and in places
subsume — the older, coarser forge planning for `U-STD` (base-surface growth)
and `U11` (Bool tower) tracked in
[`../../forge/PHASE2-INDEX.md`](../../forge/PHASE2-INDEX.md); see each spec's
"relationship to the forge spine" note.

**Continuation:** [`HANDOFF.md`](./HANDOFF.md) is the paste-ready prompt for a
fresh session to carry this work from here (finish #5–#7, close the gating
decisions, author the per-unit implementation specs).
