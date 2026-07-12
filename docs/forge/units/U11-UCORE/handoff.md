# /forge handoff — resume prompt (2026-07-12)

> **SUPERSEDED.** This handoff's "immediate next steps" are done: the forge/spec-core rosters are
> reconciled, ADR-0023 is ratified Accepted, and U-CORE-1 is landed (`03764e3`). Use
> [`U-CORE-3-handoff.md`](../U-CORE-3/handoff.md) instead — it picks up at U-CORE-3. Kept here only as
> a historical record of the state before this session's consolidation pass.

Paste the block below into a fresh Claude Code session to continue.

---

You are the orchestrator continuing the **/forge** build of the **Phalcom** language (Rust
workspace, `main`, at the repo root). A prior session landed the last four units of the original
14-unit plan and paused deliberately before starting a second, newer work track. **Do NOT replay
that work — all state is on disk.** Invoke the forge skill (`.claude/skills/forge/SKILL.md`),
stay lean (delegate reading/writing to subagents, graphify first, never slurp source into your
own context), and persist results to `docs/forge/*` / `docs/spec/core/*`.

**Read these first — they ARE your context:**
- `docs/forge/STATE.md` — live status board, all landed units, all ratified decisions.
- `docs/forge/PHASE2-INDEX.md` — the original 14-unit plan roster + dependency graph (now fully
  landed) + the open-decision register (DEC-A…F, all resolved).
- `docs/spec/core/README.md` + `HANDOFF.md` — the **second track**: a core-library spec effort
  (U-CORE-0…6) that ran *concurrently* with the last batch, is now fully spec-complete, and has
  **not been implemented yet**. This is where you pick up.
- `docs/spec/core/decisions.md` — Q1/Q2/Q4/Q5 + two catalog/code divergences, all ruled.
- `docs/spec/core/U-CORE-1-implementation-spec.md` — the next unit to ground/implement.

**Status:** `./scripts/verify.sh` is green on `main` right now (build+test+clippy, no worktree in
use — every unit since U2 has landed in-tree on `main`, committed per green checkpoint). All 14
original units are landed: U1/U2/U3/U-FE/U4/U5/U6/U7/U-LIST/U8/U9/U10/U-LEX/U-STD/U11 (yes, 15 —
U-LIST and U-FE were added mid-flight). Last four landed this session:

- **U10** — non-local return (`Bytecode::ReturnNonLocal`, frame-token eager unwind, `DeadFrameError`).
  Corrected a bug in its own spec's `call_method` Primitive-arm guard (re-push, don't skip — the
  `run_until` drain check pops the pushed value). Commit `4e2ec73`.
- **U-LEX** — block comments, numeric digit separators, lexer-level newline suppression, `?.`/`??`
  coverage (already landed by U6, just added a fixture), and string interpolation. **Sigil
  `\(expr)`** (Swift-style) was ratified by the user, overriding the architect's `{expr}`
  recommendation — [ADR-0022](../../../adr/0022-string-interpolation-backslash-paren-sigil.md). Desugars
  to `String.new(expr) + ...` (not `.toString`, since value-type `toString` doesn't exist yet —
  DEFERRED #30). Commits `dba9d49`..`d91cdf4`.
- **U-STD** — hit a real scope conflict: the plan's Object/Number/String/Symbol/System scope was
  ~90% already shipped or re-carved to not-yet-scheduled `U-CORE-N` units (see below). **User
  ratified "Option B"**: build the unblocked residual instead — `Option.{map,flatMap,filter,
  ifSome,unwrapOr}` + `List.{map,reduce,filter,includes,isEmpty,at(_,put:)}`, pure `.ph`, zero new
  primitives. `List.reduce` spelling: `reduce(_:_:)`. Commits `176d454`/`5e2b395`/`454f2b8`.
- **U11** — Bool tower: abstract `Bool` keeps the six sacred selectors (moving them to `True`/
  `False` would break the sacred-selector inliner's override-epoch tracking); `True`/`False` are
  near-empty singleton subclasses. Commits `23cafe2`/`96b440c`/`c0e1066`.

**The unresolved thread — read this before dispatching anything:** while the batch above was
running, a *separate concurrent process* (same git identity, not dispatched by this session) was
independently authoring a whole second spec track — `docs/spec/core/` — that re-derives the core
library's ground truth from scratch and **re-carves unit boundaries** the original
`PHASE2-INDEX.md` doesn't know about (`U-CORE-1` through `U-CORE-6`). This is *why* U-STD hit a
scope conflict. **`docs/forge/PHASE2-INDEX.md`'s roster and `docs/spec/core/`'s taxonomy still
disagree** — nobody has reconciled them (DEFERRED #29 names this explicitly). The user was asked
and chose to pause rather than reconcile or start U-CORE-1 immediately — that choice still stands
until they say otherwise.

**What `docs/spec/core/` actually contains (fully spec-complete, zero code written against it):**
- **U-CORE-0** (requirements/rulings): 7/7 docs done — floor census (73 native bindings),
  bootstrap phases, sacred-selector set, catalog delta, pending-retirement map,
  invariant-requirements, forward-compat checklist.
- **Gating decisions, all ruled** (`decisions.md`): Q1 `Object#hash` **is** a floor primitive
  (needs an **ADR-0019 amendment** — get user sign-off before implementing, it's a one-way-door
  precedent on the frozen floor). Q2 confirms ADR-0008 (layered exceptions+Result, terminating,
  no redesign). Q4 prelude = core module's auto-imported exports. Q5 collections are mutable by
  default, structural `==`, hashable iff immutable. Plus two divergence fixes: re-parent
  `Method < Function` (ADR-0006 was being violated), and per-type `toString` deferred to U-CORE-4.
- **Six dispatch-ready implementation specs**, `U-CORE-1..6-implementation-spec.md`, recommended
  order:
  1. **U-CORE-1** — kernel reflection: `Object#hash`, `Object#isA(_)`, Behavior/Class reflection,
     the `Method < Function` re-parent. Hard prereq for `Map`/`Set` (U-STD, later). Needs the
     ADR-0019 amendment ratified first.
  2. U-CORE-2 — mostly already landed (`0da64d6`, pre-dates this session): Bool half-Option fix +
     core `Option` combinators. Only residue may remain — check the spec for what's left.
  3. **U-CORE-3** — callables/Block reflection. **Hard prereq for any iteration method** built
     after it.
  4. U-CORE-4 — value classes: per-type `toString` overrides (closes DEFERRED #30 from U-LEX).
  5. U-CORE-5 — collection protocol *contract* (shared interface `List` already satisfies; not
     new classes).
  6. U-CORE-6 — `Error` root + wire the existing dNU miss path to raise `MessageNotUnderstood`
     through the unified unwind (per ADR-0008). Reserve, don't build, `Result`/`try`/`catch`.

**Immediate next steps (in order), once the user says go:**
1. **Reconcile `PHASE2-INDEX.md` vs `docs/spec/core/`** — fold the U-CORE-N roster into the forge
   index's unit table so scheduling doesn't fork again (this was flagged, not fixed).
2. Re-ground `U-CORE-1-implementation-spec.md` against current HEAD (`c0e1066` at last check) the
   same way U9/U10/U-LEX/U-STD/U11 were re-grounded before implementing — three separate times
   this session a stale plan turned out to have wrong assumptions about the live tree.
3. Get the user to ratify the **ADR-0019 amendment** (adding `Object#hash` to the frozen
   primitive floor) — do not implement past that gate.
4. Dispatch `phalcom-implementer` for U-CORE-1. Continue in the order above: U-CORE-3 next (the
   iteration-method prereq), then 2 (residue check)/4/5/6.
5. Reviewer is OFF for every non-load-bearing unit per `STATE.md` policy (`./scripts/verify.sh`
   exit 0 is the sole gate) — U-CORE-1..6 are not on the load-bearing list (U1/U2/U4/U6), so this
   almost certainly carries over, but confirm against `STATE.md` before assuming.

**Working conventions established this session (keep using them):**
- Every unit lands **in-tree on `main`, no worktree**, committed per green checkpoint, `graphify
  update . --no-cluster` before each commit.
- When a unit's own plan turns out stale against HEAD, write a fresh `*-implementation-spec.md`
  (via `phalcom-architect`) before dispatching the implementer — this caught real bugs 3+ times.
- When an implementer's spec surfaces a genuine open decision (scope conflict, syntax choice,
  ADR-precedent question), **stop and ask the user** — do not resolve it yourself. This happened
  twice (U-STD's Option A/B scope call, U-LEX's interpolation sigil) and both times the user's
  answer differed from the architect's own recommendation.
- Check in on background implementer/architect agents frequently and **verify their commits
  yourself** (`git log`, `git show`, re-run `./scripts/verify.sh`) rather than trusting the
  self-report alone — this caught nothing wrong this session, but is the standing bar.

Keep your own context lean: reconstruct state from `STATE.md`/`PHASE2-INDEX.md`/`docs/spec/core/`
+ graphify, not from transcripts.

---
