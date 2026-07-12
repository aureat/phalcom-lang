# /forge handoff — resume prompt (2026-07-11)

Paste the block below into a fresh Claude Code session (Opus, high effort) to continue.

---

You are the orchestrator continuing the **/forge** build of the **Phalcom** language (Rust
workspace, branch `feat/classes`, at the repo root). A prior session completed Phases 0–2 and the
front-end rewrite. **Do NOT replay that work — all state is on disk.** Invoke the forge skill
(`.claude/skills/forge/SKILL.md`), stay lean (delegate reading/writing to subagents, graphify
first, never slurp source into your own context), and persist results to `docs/forge/*`.

**Read these first — they ARE your context:**
- `docs/forge/STATE.md` — live status + every ratified decision + follow-ups.
- `docs/forge/PLAN.md` — verified audit findings (F1–F10) + the 14-unit plan (waves, write-sets).
- `docs/forge/U1-heap-brief.md` — the next unit, fully staged.
- `docs/forge/parallel-tasks.md` — 6 independent briefs (A–F) the user is dispatching separately.
- `docs/forge/lang-test-corpus-brief.md` — Phalcom test-corpus brief (also user-dispatched).
- `docs/rust-documentation-guidelines.md` — MANDATORY rustdoc rule; reviewers block on missing docs.
- `docs/adr/0009`–`0016` — ratified architecture decisions.

**Status:** Build/test/clippy GREEN via `./scripts/verify.sh` (golden corpus + invariants up).
Audit + adversarial verify + plan done. U0 (front-end panic fixes F9/F10) approved. **U-FE
(hand-written lexer + parser; LALRPOP removed from `phalcom-ast`) DONE + green, ADR-0016 filed**
(not independently reviewed — user confirmed it's finished).

**Ratified decisions (one-way doors — do not re-open):** handle/arena heap (ADR-0009); tagged
`Value` enum, NaN-boxing deferred (0010); static slot layout (0011); label-encoded selectors +
IC-ready dispatch (0012); Lua-style open/closed upvalues + frame-token non-local return (0013);
`let`+`var` (0014); `Object` default `toString` = `"<ClassName>"` (0015); hand-written front end
(0016). **Process:** redesign-first (don't preserve the old Wren-style substrate); commit green
base → parallelize in worktrees; independent review ONLY on load-bearing units **U1, U2, U3, U4,
U6** (others self-verify on the green gate + `cargo doc`).

**Immediate next steps (in order):**
1. Confirm `./scripts/verify.sh` is green. Handle the two U-FE follow-ups in `STATE.md`:
   spot-check the 1-line `phalcom-core/bin/phalcom/cli.rs` edit; note DEFERRED #1 (residual
   `lalrpop-util` + dead `CompilerError::ParseError` in `phalcom-core` — fold into U1 or cleanup).
2. `graphify update .` — the graph is stale after the front-end rewrite; refresh before querying.
3. **Commit the green state as a WIP base on `feat/classes`** — the user APPROVED committing to
   unlock clean worktree isolation. Confirm scope with the user, then commit.
4. Launch **U1 (handle/arena heap + tagged `Value`)** per `docs/forge/U1-heap-brief.md` in a
   worktree → independent review (load-bearing) → integrate → verify → commit. U1 is a
   behavior-preserving representation migration (do NOT fix the metaclass bug F2 — that's U2).
   Its write-set (from `graphify affected "PhRef"`/`"Value"`) is nearly all of `phalcom-core/src`
   + `phalcom-common/refs.rs` + the disasm bin — so it runs alone.
5. Then fan the core into waves per `PLAN.md`: **U2** metaclass tower + `verify_invariants()`
   (fixes F2/F4/F6) → **U3** selector/Signature + dispatch (fixes F1/F7/F8) → **U4** blocks →
   **U5** control-flow-as-message → **U6** absence→Option (`let`/`var`) → **U7** static fields +
   `construct` → **Wave F** (U8 dNU/`perform` ‖ U-LEX surface ‖ U-STD `core.ph`, then U10 non-local
   return) → **Wave F+1** (U9 variadics ‖ U11 Bool tower).

**Running in parallel (user dispatches to independent agents):** Briefs A–F + the test corpus —
disjoint write-sets (`phalcom-common/range`, `phalcom-repl`, `docs/spec`, `.github`, `fuzz/dict`,
root `README`+`CONTRIBUTING`, `phalcom-core/tests/lang`). They can't touch the VM spine; integrate
each with a `verify.sh` re-run. Coordinate the commit so their outputs are captured.

Keep your own context lean: reconstruct state from these files + graphify, not from transcripts.
Update `STATE.md`/`PLAN.md`/`DEFERRED.md` as you go.

---
