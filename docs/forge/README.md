# `docs/forge/` — the Phalcom forge working record

This directory is the **planning-and-execution record** of the `/forge` method. Normative spec
lives under [`../spec/`](../spec/) (current: `../spec/v0.2/`); the per-unit **as-built** specs (the
translated, authoritative record of what landed) live under [`../spec/v0.2/units/`](../spec/v0.2/units/).

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
  phase-next/     ← ALWAYS the current phase. Start here.
  units/          ← one folder per unit: work orders, handoffs.
  archive/        ← closed phases (phase1, phase2, …), kept for provenance.
```

**[`phase-next/`](phase-next/)** is the live entry point — `INDEX.md` (coordination map),
`STATE.md` (current gate/position), `HANDOFF.md` (resume prompt), `DEFERRED.md` (open ledger).
When a phase closes, its contents move to `archive/phaseN/` and a fresh `phase-next/` starts.

**[`units/`](units/)** — one folder per unit (`U-CORE-3/`, `U12/`, …), holding that unit's plan /
implementation-spec / handoff docs. Units for **landed** spine work (U1–U11, U-LIST, U-LEX, U-STD)
were translated into as-built specs under `../spec/v0.2/units/` and removed here (recoverable in
git history) — see those specs' "Sources" sections for landing commits.

**[`archive/`](archive/)**:
- [`phase1/`](archive/phase1/) — the spec-consistency audit + solutions, test-corpus plan/handoff/brief.
- [`phase2/`](archive/phase2/) — the Phase-2 planning index, master `PLAN.md`, `parallel-tasks.md`,
  and the closed-out `STATE.md`/`HANDOFF.md` for that phase.

## Conventions

- Don't fork the roster: status of record is `phase-next/STATE.md` + the as-built specs +
  `../spec/v0.2/core/README.md`. `phase-next/INDEX.md` *points*, it does not re-list.
- When a phase closes: `git mv phase-next/* archive/phaseN/`, then write a fresh `phase-next/`.
