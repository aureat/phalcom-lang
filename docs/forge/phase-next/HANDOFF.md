# /forge handoff — resume prompt (2026-07-12)

Paste this into a fresh Claude Code session to continue.

---

You are the orchestrator continuing the **/forge** build of **Phalcom** (Rust workspace, branch
`main`, repo root). The 15-unit spine roster (U-FE, U0–U11, U-LIST, U-LEX, U-STD) is **fully
landed**. Work has moved to the **U-CORE core-library successor track**.

**Read these first:**
- [`STATE.md`](STATE.md) — current gate + position.
- [`INDEX.md`](INDEX.md) — standing coordination rules (collision matrix, build order, successor
  track roster).
- [`DEFERRED.md`](DEFERRED.md) — open deferral ledger.
- [`../units/U-CORE-3/handoff.md`](../units/U-CORE-3/handoff.md) — the next unit, staged.
- [`../../spec/v0.2/core/README.md`](../../spec/v0.2/core/README.md) — index of record for the
  U-CORE track.

**Status:** Green gate (`./scripts/verify.sh`). U-CORE-1 landed (kernel reflection, `03764e3`).
U-CORE-2 mostly landed (`0da64d6`), residue only. **NEXT = U-CORE-3** (callable/Block/Method
reflection — hard prereq for iteration methods).

**Immediate next steps:**
1. Confirm the green gate.
2. Launch U-CORE-3 per its handoff doc → independent review only if load-bearing (see `STATE.md`
   review policy) → integrate → verify → commit.
3. Then U-CORE-4 (value `toString`) → U-CORE-5 (collection contract) → U-CORE-6 (`Error` root).
4. A concurrent planning batch (U12–U20, U-COLL) exists under `../units/` — check with the user
   before touching it; it may be actively edited by another session.

Keep context lean: reconstruct state from these files + graphify, not from transcripts. Update
`STATE.md`/`DEFERRED.md` as you go; when this phase closes, archive it the same way Phase 2 was
archived (see `../archive/phase2/`) and start a fresh `phase-next/`.

---
