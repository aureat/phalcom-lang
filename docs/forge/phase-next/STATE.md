# Phase-next — Running State

_Fresh as of 2026-07-12. The full landing history (every "— LANDED ✅" section, U1–U11 through
U-CORE-1) lives in the archived log: [`../archive/phase2/STATE.md`](../archive/phase2/STATE.md).
This file carries forward only what's still live: the current gate, the current position, and the
standing conventions that aren't recorded anywhere else._

## Green gate
`./scripts/verify.sh` green (build + test + clippy; golden `.ph` corpus + invariant harness up).
Re-check before starting new work.

## Current position
- Spine roster (U-FE, U0–U11, U-LIST, U-LEX, U-STD): **closed**, all landed.
- Successor core-library track (U-CORE-1..6): **in progress** — U-CORE-1 landed (`03764e3`),
  U-CORE-2 mostly landed (`0da64d6`, residue only). **NEXT = U-CORE-3** (callable/Block/Method
  reflection — see [`../units/U-CORE-3/handoff.md`](../units/U-CORE-3/handoff.md)).
- In-flight planning batch (U12–U20, U-COLL) under [`../units/`](../units/) — not yet dispatched.

## Standing conventions (carried forward — not recorded elsewhere)

**Review policy.** Independent reviewer ON only for load-bearing units (can corrupt the object
model): historically U1, U2, U3, U4, U6. Everything else self-verifies on the green gate + `cargo
doc` clean. Apply the same load-bearing test to new units.

**Worktree seeding hazard.** Worktrees branch from committed HEAD. Any uncommitted `docs/forge/`,
spec, or script change is invisible to a fresh worktree. Commit before spinning up parallel
worktree-isolated units; run serial/spine work in-tree.

**Design mandate (user, 2026-07-11).** "Build the architecture. You don't have to preserve the
current implementation. Architecture and design should be built on best practices." — spec is the
design source of truth; redesign-first, not preserve-and-patch.

**`core.ph` / `phalcom-ast` collision rules.** See [`../phase-next/INDEX.md`](INDEX.md) §2 — never
co-schedule two editors of either.

## Deferral ledger
[`DEFERRED.md`](DEFERRED.md) (this folder) — live, carried forward from the prior phase.
