# `docs/forge/` — the Phalcom forge working record

This directory is the **planning-and-execution record** of the `/forge` method. Normative spec
lives under [`../spec/`](../spec/) (current: `../spec/v0.2/`); the per-unit **as-built** specs (the
translated, authoritative record of what landed) live under [`../spec/v0.2/units/`](units/).

## The method — six roles across four phases

| Phase | Roles | Output |
|---|---|---|
| **0 · Stabilize** | `phalcom-stabilizer` | green build + verification substrate |
| **1 · Audit / Verify** | `phalcom-auditor`, `phalcom-verifier` | refuted-or-confirmed findings |
| **2 · Plan** | `phalcom-architect` | dependency-ordered, spec-grounded unit work orders |
| **3 · Implement / Review** | `phalcom-implementer`, `phalcom-reviewer` | landed units, each gated on a green verify |

A green-build gate sits at every phase boundary. See the `forge` skill for the full method.

## Layout

```
docs/forge/
  STATE.md        ← current gate, standing conventions, landing log. Start here.
  INDEX.md        ← coordination map: collision matrix, build order, decision register.
  DEFERRED.md     ← open deferral ledger (Confirmed Backlog first).
  units/          ← one folder per unit: work orders, handoffs.
  archive/        ← closed phases (phase1, phase2, …), kept for provenance.
```

**[`STATE.md`](STATE.md)** is the live entry point, with [`INDEX.md`](INDEX.md) (coordination
map) and [`DEFERRED.md`](DEFERRED.md) (open ledger) beside it. Per-phase handoff prompts live at
this level as `HANDOFF-<TOPIC>.md`.

> **The `phase-next/` convention was retired 2026-07-15.** This directory used to keep a
> `phase-next/` folder holding four files (`INDEX`/`STATE`/`HANDOFF`/`DEFERRED`) that were moved
> to `archive/phaseN/` when a phase closed. In practice the convention drifted: the live
> `STATE.md`/`DEFERRED.md` migrated to this level while the `phase-next/` copies went stale (its
> `STATE.md` still described the U-CORE track as in-flight months after U-CORE-1..6 landed), and
> the two pairs had to be reconciled by hand. The folder is now merged up here and deleted;
> `archive/phase1/` and `archive/phase2/` remain as provenance. Archive a closed phase by copying
> the current `STATE.md`/`INDEX.md`/`DEFERRED.md` into `archive/phaseN/`, not by moving a folder.

**[`units/`](units/)** — one folder per unit (`U-CORE-3/`, `U12/`, …), holding that unit's plan /
implementation-spec / handoff docs. Units for **landed** spine work (U1–U11, U-LIST, U-LEX, U-STD)
were translated into as-built specs under `../spec/v0.2/units/` and removed here (recoverable in
git history) — see those specs' "Sources" sections for landing commits.

**[`archive/`](archive/)**:
- [`phase1/`](archive/phase1/) — the spec-consistency audit + solutions, test-corpus plan/handoff/brief.
- [`phase2/`](archive/phase2/) — the Phase-2 planning index, master `PLAN.md`, `parallel-tasks.md`,
  and the closed-out `STATE.md`/`HANDOFF.md` for that phase.

**[`UNITS-TRACKER.md`](UNITS-TRACKER.md)** — cross-cutting index over `units/`, grouped by
feature area (concurrency, collections, error-handling, …) instead of by unit number, with
checkboxes and a landing-order timeline. A view, not a fork of the roster (see Conventions).

## Conventions

- Don't fork the roster: status of record is [`STATE.md`](STATE.md) + the as-built specs +
  [`../spec/v0.2/core/README.md`](../spec/v0.2/core/README.md). [`INDEX.md`](INDEX.md) *points*,
  it does not re-list. [`UNITS-TRACKER.md`](UNITS-TRACKER.md) is a read grouping of that same
  roster, not a second source of truth — update the roster first, then refresh the tracker.
- **One `STATE.md`, one `DEFERRED.md`, at this level.** Do not create a second copy in a
  subfolder — that duplication is exactly what the retired `phase-next/` convention produced, and
  reconciling the two stale-vs-live pairs cost a whole pass on 2026-07-15.
- **Never renumber `DEFERRED.md`'s numbered backlog.** ~18 docs under `units/` cite entries by
  number (`DEFERRED #30`). Numbers are frozen; append at #34+; strike in place.
- When a phase closes: copy `STATE.md`/`INDEX.md`/`DEFERRED.md` into `archive/phaseN/` for
  provenance and keep working in the live files.
