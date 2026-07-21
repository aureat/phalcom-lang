# Traceback track — serial handoff (2026-07-20)

The parallel fleet was wound down mid-track (usage window). This folder contains one
**copy-paste-ready agent brief per remaining unit**. Run them **serially, one fresh agent per
unit**, in the order below. Each brief is self-contained; the normative specs are
[`../implementation-spec.md`](../implementation-spec.md) and [`../plan.md`](../plan.md).

## State at handoff (verify against `git log` before trusting — house rule)

| Unit | State | Evidence |
|---|---|---|
| G1/G2 gates | ✅ done | PDR-0010 ratified `27c4a1a`; E002 fixed `a265684` + `3306fdf` |
| G0 Map/Set | ✅ COMPLETE, worktree-verified green | `861e21a` (core: reentrant_depth lock, fallible accessors, `RuntimeError::ConcurrentMutation`), `f43c00c` (docs), `df9bf45` (5 fixtures, negative-controlled) |
| T1 walk/accessor | ⚠️ ~95% — committed `e046696`, needs a short FINISHER pass | Done: walk API (oldest-first, expand seam, 3 tests), `span_at`/`line_at` + ALL `.spans[` sites migrated (invoke_at, SuperSend, runtime_error), `FiberObject::seq` (root=1). Finisher owes: (a) full `cargo test && cargo clippy --workspace` gate (agent wrapped before running it), (b) fix the order-dependent flaky `walk_orders_oldest_first_with_selector_shaped_names` test, (c) class-qualified frame names (`Cart.total` composition) — deliberately deferred, design note in walk.rs module doc; either finish it or explicitly hand to T4. `runtime_error` not swapped onto StackWalk (fine — T4 rewrites it anyway); `FrameName::Native` unconstructed (correct, T4/IS §5.5) |
| T2 substrate | ✅ COMPLETE | `3ec895d` (deps), `523583c` (style.rs), `255d8b8` (caret.rs), `703c9e6` (mod.rs migration + flags + CLAUDE.md), `15ca990` (DEFERRED). 15 new tests green; miette gone. Known seams left FOR T4/T5 by design: `Snippet` not yet consumed by a real renderer; `print_*` read `RenderConfig` via a `RENDER_CONFIG` OnceLock (T4/T5 replace with explicit param); TTY width hardcoded 80 (DEFERRED.md) |
| T3, T4, T5, T6, T7, T8 | not started | briefs in this folder |

**If T1/T2 left UNCOMMITTED work:** the files are on disk. First agent for that unit starts
with: `git status --porcelain`, read the diffs, decide keep-or-redo per the brief, and commits
with pathspec once green. Do not `git checkout --` anything before reading it — another
session's live edits may be interleaved; touch only the unit's declared write-set.

## Serial order

1. ~~Finish T2~~ — DONE (see table).
2. **Finish T1** — short pass, start from `e046696` (all work committed, tree clean): run the
   full gate, fix the flaky walk test (order-dependent in full-workspace runs, passes in
   isolation — per T2's report), and resolve the class-qualified-name deferral (walk.rs module
   doc has the design note). Brief for context:
   [`T1-walk-and-accessor.md`](T1-walk-and-accessor.md).
3. **T5** — [`T5-entry-paths-exit-codes.md`](T5-entry-paths-exit-codes.md) (needs T2).
4. **T3** — [`T3-capture-and-kind.md`](T3-capture-and-kind.md) (needs T1's walk vocabulary).
5. **T4** — [`T4-traceback-renderer.md`](T4-traceback-renderer.md) (needs T1+T2+T3; the
   visible payoff — run the bogusSelector before/after check it specifies).
6. **T7** — [`T7-observability.md`](T7-observability.md).
7. **T6** — [`T6-messages-didyoumean.md`](T6-messages-didyoumean.md).
8. **T8** — fixture sweep: color-off invariance across catalog examples, canary audit,
   fixtures README stating the field-assert contract (implementation-spec §11). Small; can
   ride T6's agent.

## How to launch each (manual, serial)

Open a fresh session, paste the brief file's content verbatim, prepended with:

> You are implementing one unit of Phalcom's traceback track. Repo:
> /Users/altunhasanli/dev/phalcom/phalcom, work on `main` directly. Read the files the brief
> names BEFORE coding. Obey every GIT rule in the brief (pathspec commits only, never
> `git add -a`, never `git checkout -b`; other sessions may have live edits — touch only your
> write-set). Gate: cargo build && cargo test && cargo clippy --workspace. Rustdoc mandatory
> on all public items. First step: `git log --oneline -15` + `git status` to verify the
> handoff table above against reality — landed-state claims go stale in this repo.

One agent at a time; wait for its final report + green gate before starting the next. After
each unit lands, tick it in [`../plan.md`](../plan.md).

## Standing hazards (do not relearn these)

- Concurrent sessions commit to main continuously — pathspec discipline is what keeps them
  safe from each other.
- Negative-control every new test (run it against a broken/pre-fix tree once).
- Re-derive fixes from code, not from recorded prescriptions.
- Perf claims only from docs/forge/perf-log/SCOREBOARD.md.
- `--trace-format=json` output is never colored; stdout stays byte-exact (goldens); all
  diagnostics on stderr.
