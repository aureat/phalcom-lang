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
| 5 | `_pending` → active retirement map | `pending-retirement.md` | ⏳ TODO |
| 6 | Invariant requirements per unit | `invariant-requirements.md` | ⏳ TODO |
| 7 | "Must not preclude" forward-compat checklist | `forward-compat.md` | ⏳ TODO |

Downstream implementation units (U-CORE-1 kernel reflection, U-CORE-2
absence+Boolean, U-CORE-3 callables, U-CORE-4 value classes, U-CORE-5
collection contract, U-CORE-6 errors) are planned in the requirements analysis
and tracked in [`../../forge/PHASE2-INDEX.md`](../../forge/PHASE2-INDEX.md).
