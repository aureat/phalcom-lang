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
- Successor core-library track (U-CORE-1..6): **U-CORE-1..5 LANDED; U-CORE-6 (final unit) in
  flight.** This session (2026-07-12) landed, in recommended order 3→2→4→5:
  - U-CORE-3 `10ebd06` — Method reflection (`methodFor`/`invokeOn`/`bind`/`selector`/`holder`) +
    `Object::BoundMethod` heap arm; floor 80→85 (ADR-0028).
  - U-CORE-2 residue `ce84283` — the R-INV-2.1..2.4 absence/control-flow corpus fixtures the
    unit had specified but never landed (no code change).
  - U-CORE-4 `2061795` — per-type value `toString` + Symbol `#`-sigil render unification
    (BD-CORE4-2); floor 85→86 (ADR-0036). Re-pinned the golden corpus (`<X instance>`→proper
    toString) incl. U-CORE-3's method-reflection fixture (`selector.toString`→`#greet(_:)`).
  - U-CORE-5 `bc161fb` — structural `List#==`/`!=` (`.ph` over the floor, native `list_eq`
    rejected by spec §3.1) + reusable collection ContractSpec; **+0 floor**.
  - U-CORE-6 `85c4e1d` — Error root + `MessageNotUnderstood < Error`; dNU miss now raises a
    surface `MessageNotUnderstood` via the unified unwind (`RuntimeError::Raise`, ADR-0008
    sibling of U10's Return); +2 floor → 88 (ADR-0037). Proxy path preserved; Arity/Type stay native.
  - **U-CORE core-library track (U-CORE-1..6) is COMPLETE.** Floor 73 → 88 across the track.
    Reserved for a future unit: `ArgumentError`/`TypeError` reification (Arity/Type still native).
- Pre-grounds committed for the tail: U-CORE-4/5/6 as-built specs re-anchored to current HEAD.
- In-flight planning batch (U12–U20, U-COLL) under [`../units/`](../units/) — not yet dispatched.

## Standing conventions (carried forward — not recorded elsewhere)

**Review policy.** Independent reviewer ON only for load-bearing units (can corrupt the object
model): historically U1, U2, U3, U4, U6. Everything else self-verifies on the green gate + `cargo
doc` clean. Apply the same load-bearing test to new units.

**Worktree seeding hazard — DO NOT worktree-isolate subagents on this tree (2026-07-12).**
Beyond the branch-from-HEAD staleness: a worktree-isolated implementer this session was handed a
worktree at an *ancient* base (predating `docs/forge/` and the test corpus), couldn't build, and
while investigating ran `git stash push -u` in the SHARED main checkout — silently yanking every
concurrently-running agent's uncommitted work into `stash@{0}`. Recovered via
`git show 'stash@{0}^3:PATH'` (untracked) / `git show 'stash@{0}:PATH'` (tracked). Rule: run the
U-CORE tail IN-TREE, one active writer at a time, commit each unit the instant it is green (a
committed diff cannot be stashed away); parallelize only by DISJOINT write-set (docs vs
`tests/lang/**` vs `src/`), never by worktree. The `verify.sh` green gate is the backstop that
catches a mid-flight clobber (a half-reverted tree won't compile).

**Design mandate (user, 2026-07-11).** "Build the architecture. You don't have to preserve the
current implementation. Architecture and design should be built on best practices." — spec is the
design source of truth; redesign-first, not preserve-and-patch.

**`core.ph` / `phalcom-ast` collision rules.** See [`../phase-next/INDEX.md`](INDEX.md) §2 — never
co-schedule two editors of either.

## Deferral ledger
[`DEFERRED.md`](DEFERRED.md) (this folder) — live, carried forward from the prior phase.
