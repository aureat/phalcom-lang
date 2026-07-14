# /forge handoff — resume prompt (2026-07-12, post-U-CORE-1)

Paste the block below into a fresh Claude Code session to continue.

---

You are the orchestrator continuing the **/forge** build of the **Phalcom** language (Rust
workspace, `main`, at the repo root). The prior session consolidated the tree (reconciled the
`PHASE2-INDEX.md` ↔ `docs/spec/core/` roster fork, ratified ADR-0023) and landed **U-CORE-1**
(kernel reflection). **Do NOT replay that work — all state is on disk.** Invoke the forge skill
(`.claude/skills/forge/SKILL.md`), stay lean (delegate reading/writing to subagents, graphify
first, never slurp source into your own context).

**Read these first — they ARE your context:**
- `docs/forge/STATE.md` — live status board; see the "U-CORE-1 — LANDED" section for exactly what
  shipped, and the phase-log NEXT pointer.
- `docs/forge/PHASE2-INDEX.md` §7 — the successor-track roster (U-CORE-1..6), now the single index
  of record (no more forked rosters — this was DEFERRED #29, now resolved).
- `docs/spec/core/decisions.md`, `docs/spec/core/U-CORE-3-implementation-spec.md` — the next unit.
- `docs/adr/accepted/0023-amend-floor-admit-hash-and-kernel-reflection.md` — the omnibus floor amendment,
  **Accepted**. U-CORE-3's own floor delta (+5, `Method` reflection surface) is already authorized
  by this ADR — nothing to draft or ratify, only ground and implement.

**Status:** `./scripts/verify.sh` is green on `main` (build+test+clippy), no worktree in use. HEAD
is `03764e3`. Floor is **80 bindings** (73 base + U-CORE-1's 7).

**What landed this session (in order):**
1. **Docs consolidation** (`de03f26`) — PHASE2-INDEX.md §7 now points at `docs/spec/core/` as the
   index of record; refreshed stale baseline pins (76b5f35→0f84232) across
   README.md/decisions.md/invariant-requirements.md/forward-compat.md; recomputed catalog-delta.md
   §3's rollup table (True/False + Option now fully-✅ rows). Ratified **ADR-0023** (Accepted) —
   the omnibus ADR-0019 amendment covering U-CORE-1/3/4/6's floor additions in one document instead
   of four.
2. **U-CORE-1 spec re-grounding** (`010268c`, via `phalcom-architect`) — fixed the ADR-0022→0023
   numbering error, repinned the baseline header to HEAD, refreshed every drifted file:line anchor
   (U11 had shifted `universe.rs` line numbers by ~11 lines), folded `True`/`False` into the
   R-INV-0.2 invariant enumeration.
3. **U-CORE-1 implementation** (`03764e3`, via `phalcom-implementer`, two dispatch passes — the
   first agent was deliberately stopped mid-task for context management, its partial working-tree
   state (all 7 primitive bodies + the `Method` re-parent written, only 1 of 7 installs wired) was
   handed to a fresh agent with an explicit checkpoint description, no work lost) — kernel
   reflection: `Object#hash` + 4 immediate `hash` overrides, `Behavior#name`/`methods`,
   `Object#isA(_)` (`.ph`), `Method < Function` re-parent, the R-INV-0.x invariant-harness substrate
   every later U-CORE unit extends. Floor 73→80. Independently hand-verified against the built CLI
   by the orchestrator (not just the test harness) before being accepted as done.

**Immediate next steps, in order:**
1. **Re-ground `docs/spec/core/U-CORE-3-implementation-spec.md`** against current HEAD `03764e3`
   the same way U-CORE-1 was re-grounded (dispatch `phalcom-architect`) — floor is now 80, not 73;
   `Method < Function` is already done (U-CORE-3's own spec likely has a "whichever lands first"
   clause for this — it should now read "already done by U-CORE-1, assert don't redo," per
   decisions.md §4.1's own instruction); confirm file:line anchors in `universe.rs`/`class.rs`
   haven't drifted from U-CORE-1's edits.
2. **Dispatch `phalcom-implementer` for U-CORE-3** (callables/Block/Method reflection —
   `Object#methodFor(_)`, `Method#invokeOn(_,_)`, `Method#bind(_)`, `Method#selector`,
   `Method#holder`, +5 floor bindings, 80→85). This is the hard prereq for any iteration method
   built after it (per `docs/spec/core/README.md`'s recommended order: U-CORE-1 → **3** → 2 → 4 →
   5 → 6).
3. Continue in the recommended order: U-CORE-2 (residue check — mostly landed already at
   `0da64d6`), then U-CORE-4 (value-class `toString`), U-CORE-5 (collection contract), U-CORE-6
   (Error root + dNU wiring).
4. Reviewer is **OFF** for U-CORE-1 (confirmed, added to STATE.md's policy list this session) —
   U-CORE-3..6 are very likely the same (not on the ON list: U1/U2/U3/U4/U6), but confirm against
   STATE.md's current policy line before assuming for each unit.

**Working conventions validated this session (keep using them):**
- In-tree on `main`, no worktree, committed per green checkpoint, `graphify update . --no-cluster`
  before each commit.
- Re-ground a stale spec via `phalcom-architect` before dispatching the implementer — this has now
  caught real staleness (wrong ADR number, stale line anchors, missing U11 rows) on 4+ units in a
  row.
- **Concurrent-session hazard is live and ongoing**: other sessions are actively adding
  `docs/forge/U12..U20-plan.md`, `docs/forge/U-COLL-plan.md`, and new ADRs (0024–0027) to this same
  working tree. Never `git add -A`/`git add .` — stage only your unit's explicit write-set, and run
  `git status --short` before every commit to confirm nothing stray got swept in. This is the
  single most important operational rule right now.
- **When a long implementer run needs to be interrupted** (e.g. for context management), don't just
  kill it — capture the exact working-tree state first (`git status`/`git diff --stat`, and enough
  detail on *which specific pieces* are done vs. pending) and hand that snapshot to a fresh agent in
  its dispatch prompt. This was done successfully mid-U-CORE-1 with zero rework.
- **Independently verify the implementer's self-report** before accepting a unit as done: re-run
  `./scripts/verify.sh` yourself, `git show --stat` the commit to confirm the diff matches the
  claimed write-set, and hand-drive at least the fixtures' actual expected behavior through the
  built CLI (`cargo run -p phalcom-core --bin phalcom -- <fixture>.ph`) rather than trusting the
  test harness alone.

Keep your own context lean: reconstruct state from `STATE.md`/`PHASE2-INDEX.md`/`docs/spec/core/`
+ graphify, not from transcripts.

---
