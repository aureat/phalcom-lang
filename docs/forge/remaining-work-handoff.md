# Handoff — remaining Phalcom VM build-out

## Where we are (2026-07-11)
- **Single branch `main @ 231e43b`**, `./scripts/verify.sh` **green** (build + test + clippy + golden + `lang.rs` corpus 9✓/7 pending + invariants). One worktree, clean.
- **Landed:** hand-written front end (U-FE, ADR-0016) · U0 panic fixes · **U3** dispatch/selector redesign + F1/F7 (label-encoded selectors, IC-ready) · labeled acceptance corpus `phalcom-core/tests/lang.rs`.
- **The spine was reordered on purpose:** U3 shipped *before* U1, on the pre-heap `Rc<RefCell>` substrate — to have real material to migrate and audit.

## Read first (authority)
`docs/forge/STATE.md` (status + ratified decisions) · `docs/forge/PLAN.md` (findings/waves) · `docs/forge/U1-plan.md` (next unit's full work order) · ADRs `0007`–`0016` (one-way doors) · `docs/spec/` · `docs/rust-documentation-guidelines.md` (rustdoc is mandatory).

## Working model — learned the hard way
1. **One VM-spine unit at a time, each in its OWN isolated git worktree off `main`** (`git worktree add ../phalcom.worktrees/<unit> feat/<unit>`). **Never run two spine units in the same working tree** — they collide, clobber, and reset each other.
2. Merge a unit to `main` only when `verify.sh` is green **and** (if load-bearing) independently reviewed; then delete its branch + worktree.
3. Parallelize ONLY genuinely-disjoint units (Wave F), each in its own worktree.
4. Every unit: green gate + `cargo doc --workspace --no-deps` clean + full rustdoc + ADR citations. graphify-first orientation.
5. **Independent adversarial review** (`phalcom-reviewer`) for load-bearing units = **U1, U2, U4, U6**; others self-verify on the green gate.
6. git: `feat/<unit>` off `main`; conventional commits ending `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.

## Remaining units — in order
| # | Unit | Spec / ADR | Fixes | Review |
|---|------|-----------|-------|:--:|
| 1 | **U1 — handle/arena heap + tagged `Value`** — migrate the current (U3-inclusive) code off `Rc<RefCell>`/`PhRef`; `slotmap`; VM owns `Heap`; allocate-then-patch bootstrap; fold DEFERRED #1. **Behavior-preserving; do NOT fix F2.** | ADR-0009/0010 · `U1-plan.md` | F5 | ✅ |
| 2 | **U2 — metaclass tower + `verify_invariants()`** — parallel rule, `Behavior` kernel, un-ignore the 2 invariants. | ADR-0002/0003 · object-model §5–6 | F2/F5/F6 | ✅ |
| 3 | **U4 — blocks/closures** — Lua-style open/closed upvalues; frame-token non-local return. | ADR-0013 · blocks.md | — | ✅ |
| 4 | **U5 — control-flow-as-message + inliner** — if/while/and/or → sends; inline sacred selectors; deopt guard. | control-flow.md | — | — |
| 5 | **U6 — absence → `Option`** — `let`/`var`, no surface `nil`, `if(opt)` is a compile error. | ADR-0007/0014 · values-and-absence.md | — | ✅ |
| 6 | **U7 — static fields + `construct`.** | classes.md | — | — |
| 7 | **Wave F (parallel, own worktrees)** — U8 dNU/`perform` ‖ U-LEX surface syntax ‖ U-STD `core.ph`; then U10 non-local return. | messages / lexical / blocks | — | — |
| 8 | **Wave F+1 (parallel)** — U9 variadics ‖ U11 Bool tower. | — | — | — |

## While migrating (U1 note)
You are the first careful reader of the freshly-written U3 code. Surface any bugs/spec-deviations/smells (file:line) into your report + `docs/forge/DEFERRED.md`; don't fix inline (keep U1 behavior-preserving).

## Loose ends
- 1 git stash remains (`WIP on feat/classes … stack overflow …`) on a defunct `phalcom-vm` layout — drop or ignore; not applicable to the current `phalcom-core` tree.
- `main` is **+44 ahead of `origin/main`, unpushed** — push when ready.
- `STATE.md` predates the reorder/consolidation — refresh it (U3 done, U1 next) at the start of the next session.
