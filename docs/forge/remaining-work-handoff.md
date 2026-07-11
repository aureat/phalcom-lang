# Handoff — Phalcom VM spine: U2 → U4 → U5 (planning + implementation, no verification)

## Where we are (2026-07-11)
- **`main @ 53fe397`**, clean. Landed: U-FE (ADR-0016), U0, **U3** (ADR-0012 selectors), **U1** (`6515ea3` — handle/arena heap + tagged `Value`, ADR-0009/0010; `PhRef`/`RefCell`/`Rc::new_cyclic` all gone, VM owns a `slotmap` `Heap`, handles are `Copy` `ObjRef`/`ClassId`).
- **Phase-2 planning is DONE:** every remaining unit already has a dispatch-ready work order (`docs/forge/U2-plan.md`, `U4`…`U11`, `U-LEX`, `U-STD`). Master map = [`PHASE2-INDEX.md`](PHASE2-INDEX.md) (roster, dep graph, write-set collision matrix, open-decision register). `STATE.md` is current.

## Do this next — serial spine, in order
**U2 → U4 → U5.** (U3 is already landed; the user's "u2/u3/u4" = the next three spine units.)
| Unit | Plan | Mission |
|---|---|---|
| U2 | `U2-plan.md` | metaclass tower parallel rule + `Behavior` kernel + `verify_invariants()`; un-ignore/rewrite the invariant tests |
| U4 | `U4-plan.md` | first-class blocks/closures, Lua open/closed upvalues, frame-token infra (ships NO non-local return — that's U10) |
| U5 | `U5-plan.md` | control-flow-as-message + sacred-selector inliner w/ deopt guard |

## Working model — follow exactly
1. **One unit at a time, each in its own worktree off `main`:** `git worktree add ../phalcom.worktrees/<unit> -b feat/<unit> main`. Merge (squash) to `main` when the unit is done, then delete the branch + worktree.
2. **Keep the architect, but light:** each `U<n>-plan.md` predates the U1 merge — first do a tight `phalcom-architect` pass that **reconciles the plan against the landed heap substrate** (`Heap`/`ObjRef`/`ClassId`/tagged `Value`; no `PhRef`/`RefCell`) and **breaks the unit into bounded slices**. Fold any plan-cited ADR edits (U2: ADR-0002 pointer note "Rc::new_cyclic superseded by 0009", flip ADR-0003 → Accepted).
3. **SLICE across FRESH subagents — never let one grind to huge context** (the hard lesson from U1: a single implementer ballooned to ~176k). Each slice = one fresh `phalcom-implementer`, bounded scope, commits a checkpoint + updates `docs/forge/U<n>-progress.md`, then STOPS. Spawn a fresh subagent for the next slice from the committed state + progress log.
4. **Planning + implementation ONLY — NO verification this pass.** Skip the independent `phalcom-reviewer` gate, the adversarial verify, and the full `./scripts/verify.sh` green-gate ceremony. Implementers still build their code to get it working, but do not run the review/verify phases. (Trade-off: correctness is unverified until a later dedicated verification pass — flag anything risky in `U<n>-progress.md`.)
5. Behavior/spec: build the architecture per spec/ADR (redesign-first, don't preserve the old substrate for its own sake). Full rustdoc stays mandatory.
6. git: conventional commits ending `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.

## Open decisions — the user must answer before the named sub-feature builds
Only these block the U2→U4→U5 batch (rest are in `PHASE2-INDEX.md §4`):
- **DEC-E (blocks U5):** who owns `if`/`while`/`for` surface parsing? No control-flow AST node exists. Architect recommends **U5 owns** parse-time desugaring to block sends (adds `phalcom-ast` to U5's write-set). **Get the user's call before U5's parsing slice.**
- U2 and U4 have **no** blocking open decision — proceed.

## Read first (authority)
`PHASE2-INDEX.md` · the relevant `U<n>-plan.md` · `STATE.md` · ADRs (`0002/0003` for U2, `0013/0006` for U4, `control-flow.md` + new **ADR-0017** for U5) · `docs/spec/` · `docs/rust-documentation-guidelines.md`. graphify-first orientation (`graphify affected "<sym>"`, `graphify explain "<type>"`) before reading source.

## Loose ends
- `main` is unpushed (well ahead of `origin/main`) — push when ready.
- Untracked local tooling dirs `.agents/skills/`, `.codex/agents/` left uncommitted on purpose (skill/agent exports; not sure they belong in VCS — user's call).
