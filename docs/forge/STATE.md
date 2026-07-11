# Forge — Running State

_Orchestrator's live status board. Compact by design; detail lives in PLAN.md / DEFERRED.md / memory._

## Green gate (last checked: 2026-07-11)
- Single command: **`./scripts/verify.sh`** (build + test + clippy; `--fuzz`/`--miri` opt-in). Exit 0.
- Substrate: `phalcom-core/tests/golden.rs` (+fixtures), `phalcom-core/tests/invariants.rs` (2 `#[ignore]`d spec targets: parallel-superclass rule, Behavior class).
- Golden corpus: `examples/core_new.ph`, `examples/person2.ph`, `tests/fixtures/golden/{hello,arithmetic}.ph`. (Other examples excluded — they trip the parser `todo!()` panic, see F9/F10.)

## Phase log
| Phase | Status | Notes |
|---|---|---|
| 0. Stabilize (build) | ✅ done | Tree already compiles; was red in prior sessions. |
| 0. Stabilize (verify substrate) | ⏳ in progress | Golden `.ph` corpus + invariant harness + snapshot/fuzz lanes → one `verify` command. |
| 1a. Audit | ⏳ in progress | Lenses: object-model soundness, correctness-vs-spec (aligned surface), borrow/memory. |
| 1b. Verify | ⬜ pending | Adversarial refutation of surviving findings. |
| 2. Plan | ✅ done (2026-07-11) | **All 11 remaining units have dispatch-ready work orders** (`U2/U4/U5/U6/U7/U8/U9/U10/U11/U-LEX/U-STD-plan.md`), written by 6 parallel architects. Master map + open-decision register + new-ADR backlog → [`PHASE2-INDEX.md`](PHASE2-INDEX.md). **6 OPEN DECISIONS (DEC-A…F) await the user** before their sub-features build. |
| 3. Implement/Review | ⏳ in progress | U0 APPROVED (F9+F10). U-FE ✅, U3 ✅ (ADR-0012), **U1 ✅ landed `6515ea3`** — handle/arena heap + tagged Value (ADR-0009/0010), sliced impl (4 fresh subagents) + independent `phalcom-reviewer` gate (caught+fixed a Symbol/Module `==` regression); LALRPOP fully gone. **NEXT = U2 (metaclass tower + `verify_invariants()`), plan ready ([U2-plan.md](U2-plan.md)).** Then U4→U5→U6→U7. |

## U1 — LANDED ✅ (2026-07-11, `6515ea3`)
- Migrated off `Rc<RefCell>`/`PhRef` → `slotmap`-backed `Heap` (Copy `ObjRef`/`ClassId`) + tagged `Value` (ADR-0009/0010). VM owns `Heap`; allocate-then-patch bootstrap; RefCell double-borrow panic surface removed. F2 preserved observationally (U2's job). DEFERRED #1 closed (LALRPOP gone from workspace). `verify.sh` green, goldens byte-identical, `cargo doc` clean.
- **Working model that worked:** implemented in an isolated worktree `feat/u1-heap` off `main`, sliced across **4 fresh subagents** (never let one grind to huge context — see memory `subagent-context-handoff`), each committing a checkpoint + `U1-progress.md` handoff. Independent `phalcom-reviewer` BLOCKED on a `Symbol`/`Module` `==`/`!=` semantics regression (`value_eq` fell through to derived `PartialEq`); a scoped fresh fixer restored `main`'s semantics; re-verified + merged (squash) + worktree deleted.
- **Reviewer-blessed deviation:** compiler allocates constants directly via `&mut VM` into the one VM-owned `Heap` (not the plan's heap-free descriptor approach) — sound for U1 (single heap, VM-lifetime handles); "true heap-free compiler" deferred.

## U-FE follow-ups (note for next session)
- U-FE edited `phalcom-core/bin/phalcom/cli.rs` (1 line — the sole build blocker, migrating off the deleted `ProgramParser` to `parse_source`). Out of its declared write-set but reported; spot-check it.
- DEFERRED #1: `phalcom-core` still carries `lalrpop-util` dep + dead `CompilerError::ParseError` + `From<lalrpop_util::ParseError>` — LALRPOP not yet gone from the *workspace* graph. **Folded into U1 §6.**
- U-FE was NOT independently reviewed (session ended). It self-reports green + docs clean. Next session: quick review OR proceed (user confirmed parser/lexer finished).

## Parallelization decision (user, 2026-07-11): commit green base, then parallelize
Sequence: **U-FE finishes → review → integrate → verify green → COMMIT WIP base on feat/classes → launch U1 (heap) in a clean worktree; 6 briefs + test corpus also move to isolated worktrees.** Committing fixes the stale-worktree-seeding tax (U0 had to hand-sync). Gating item = U-FE. After U1 lands, the core fans out into waves (U2 metaclass, U3 dispatch, …). U1 itself stays serial-first (everything depends on its Value/Heap types). Ask user before running the actual `git commit` (in-progress tree → one WIP commit).

## Front-end decision (user, 2026-07-11): drop LALRPOP, hand-write lexer+parser
New unit **U-FE**: hand-rolled lexer + recursive-descent/Pratt parser in `phalcom-ast`, at parity with current grammar + F9/F10, removing lalrpop deps. Rationale → **ADR-0016** (authored by U-FE). Load-bearing → reviewed. Runs in-tree. U0's parser.lalrpop fix is discarded (parser deleted); U0's `SyntaxError`/`Display` (F9) + golden re-enablement salvaged. Later units (U4 blocks, U6 let/var, U7 construct, U-LEX surface) EXTEND this hand-written parser instead of a grammar file.

## Review policy (user, 2026-07-11): review load-bearing units only
Independent reviewer ON: **U1, U2, U3, U4, U6** (heap, metaclass tower, dispatch, blocks, absence — can corrupt the object model). Reviewer OFF (self-verify on green gate + `cargo doc`): U5, U7, U8, U-LEX, U-STD, U9, U10, U11. All units still gate on `./scripts/verify.sh` green + docs.

## Worktree seeding hazard (learned at U0)
Worktrees branch from committed HEAD, but ALL forge work (spec, scripts/, docs/forge/, ADRs, golden corpus, the `todo!()` defect) is UNCOMMITTED in the live tree. So a worktree-isolated agent starts from a stale base missing everything. **Decision: run the serial spine (U1–U7) IN-TREE, no worktree isolation** — isolation is only needed for parallel disjoint-write units (Wave F), and before that wave commit the branch so worktrees seed correctly (ask user before committing).

## Design mandate (user, 2026-07-11)
**"Build the architecture. You don't have to preserve the current implementation. Architecture and design should be built on best practices."**
→ Phase 2 is redesign-first, NOT preserve-and-patch. Spec = design source of truth; free to replace the Wren/clox-style substrate. Keep current code only where it already matches the best-practice target. F9/F10 front-end fixes kept only as a prerequisite to run the golden corpus.

## Ratified decisions (user, 2026-07-11) — these are one-way doors, now closed
- **BD-1 Heap model → HANDLE/ARENA HEAP.** Central `Heap`, `Copy` `ObjRef`/`ClassId` handles. Kills F5 Rc-cycle leak + RefCell borrow-panic surface; IC/GC-ready. → ADR (heap ownership).
- **BD-2 Instance `toString` → `"<{ClassName}>"`.** (User overrode architect's `"{ClassName} instance"`.) A class's own `toString`/`name` = its own name (fixes F4). No `printString` selector.
- **BD-3 Closure capture → LUA-STYLE OPEN/CLOSED UPVALUES.** Escaping blocks + shared mutation. → ADR (closure/upvalue + frame-token non-local return).
- **BD-4 Bindings → ADOPT `let` + `var`.** `let` immutable, `var` mutable, `var x` uninitialized reads as absence/None. Resolves open-Q1.

## ADR mapping (AUTHORITATIVE — PLAN.md uses stale provisional numbers 0008–0012; use these)
Pre-existing: 0007 = Option/Some/None (absence model), 0008 = Layered exceptions + Result (open-Q9).
New (drafted 2026-07-11, Accepted):
- **0009** — handle/arena heap (BD-1) · **0010** — tagged `Value` enum (NaN-boxing deferred) · **0011** — static instance slot layout
- **0012** — label-encoded selectors + IC-ready dispatch (folds F1/F7/F8) · **0013** — open/closed upvalues + frame-token return (BD-3)
- **0014** — `let`/`var` bindings (BD-4, open-Q1) · **0015** — `Object` default `toString` = `"<ClassName>"` (BD-2, fixes F4)
NOTE: ADR-0002's *Consequences* mention an `Rc`-based `PhRef::new_cyclic` cycle-break mechanism now superseded by 0009's handle-patching — the parallel-rule DECISION stands; only the wiring mechanism changed. Add a pointer note on 0002 when U2 lands.

## Phase 3 execution order (from architect plan)
Critical path is ~serial (U1–U7 all touch vm.rs/compiler/bytecode):
`U0 front-end fix → [ADRs 0007+] → U1 heap → U2 metaclass tower+verify_invariants → U3 selector/Signature+Invoke → U4 blocks/closures → U5 control-flow-as-message → U6 absence→Option → U7 static fields+construct → Wave F fan-out (U8 dNU ‖ U-LEX ‖ U-STD; then U10) → Wave F+1 (U9 variadics ‖ U11 Bool)`.
Parallelizable lanes only: `core.ph` (U-STD) + lexer/grammar (U-LEX). Everything on the VM spine is serialized.

## Plan-of-record (from docs/spec/implementation-status.md §"Recommended order")
1. Selector redesign (#1) — critical path, everything depends on it.
2. Blocks (#2) — critical path.
3. Parallel: operators→sends (#3) + nil→Option (#4).
4. Metaclass tower fix + verify_invariants() — small, self-contained.
5. Features #5–#10 on the corrected core.
