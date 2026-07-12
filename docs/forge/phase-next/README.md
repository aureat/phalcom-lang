# `docs/forge/phase-next/`

**This folder always points at the current phase of forge work.** When it closes, its contents
move into a new `docs/forge/archive/phaseN/` folder (see [`../archive/phase2/`](../archive/phase2/)
for the last example) and a fresh `phase-next/` is created for whatever comes after — same four
files, updated content.

| File | Role |
|---|---|
| [`INDEX.md`](INDEX.md) | Standing coordination map: write-set collision matrix, build-order discipline, resolved-decision register, successor-track roster. |
| [`STATE.md`](STATE.md) | Current green-gate status, current position, standing conventions. |
| [`HANDOFF.md`](HANDOFF.md) | Paste-into-a-fresh-session resume prompt. |
| [`DEFERRED.md`](DEFERRED.md) | Open deferral ledger, carried forward from the prior phase. |

Per-unit work orders and handoffs live one level up, under [`../units/<unit>/`](../units/).
