# Forge — Running State

_Orchestrator's live status board. Compact by design; detail lives in PLAN.md / DEFERRED.md / memory._

## Green gate (last checked: 2026-07-11)
- Single command: **`./scripts/verify.sh`** (build + test + clippy; `--fuzz`/`--miri` opt-in). Exit 0.
  U4 pass verified the equivalent manually (`cargo build --workspace`, `cargo test -p phalcom-core`,
  `cargo doc --workspace --no-deps`, `cargo clippy --workspace`) — all clean; the strict `verify.sh`
  ceremony itself was not required this pass (handoff-authorized, mirrors U2).
- Substrate: `phalcom-core/tests/golden.rs` (+fixtures), `phalcom-core/tests/invariants.rs` (10/10 passing — U2 landed the parallel-rule + Behavior-class targets, no `#[ignore]`d spec targets remain). `phalcom-core/tests/lang.rs`'s `blocks()` group is no longer `#[ignore]`d (U4).
- Golden corpus: `examples/core_new.ph`, `examples/person2.ph`, `tests/fixtures/golden/{hello,arithmetic,blocks_map_reduce,blocks_escaping_counter}.ph`. (Other examples excluded — they trip the parser `todo!()` panic, see F9/F10.)

## Phase log
| Phase | Status | Notes |
|---|---|---|
| 0. Stabilize (build) | ✅ done | Tree already compiles; was red in prior sessions. |
| 0. Stabilize (verify substrate) | ⏳ in progress | Golden `.ph` corpus + invariant harness + snapshot/fuzz lanes → one `verify` command. |
| 1a. Audit | ⏳ in progress | Lenses: object-model soundness, correctness-vs-spec (aligned surface), borrow/memory. |
| 1b. Verify | ⬜ pending | Adversarial refutation of surviving findings. |
| 2. Plan | ✅ done (2026-07-11) | **All 11 remaining units have dispatch-ready work orders** (`U2/U4/U5/U6/U7/U8/U9/U10/U11/U-LEX/U-STD-plan.md`), written by 6 parallel architects. Master map + open-decision register + new-ADR backlog → [`PHASE2-INDEX.md`](PHASE2-INDEX.md). **6 OPEN DECISIONS (DEC-A…F) await the user** before their sub-features build. |
| 3. Implement/Review | ⏳ in progress | U0 APPROVED (F9+F10). U-FE ✅, U3 ✅ (ADR-0012), U1 ✅ landed `6515ea3` — handle/arena heap + tagged Value (ADR-0009/0010). U2 ✅ landed — metaclass tower parallel rule + `Behavior` kernel + `verify_invariants()` (ADR-0002/0003); reviewer gate explicitly SKIPPED per user instruction, see [U2-progress.md](U2-progress.md). **U4 ✅ landed (2026-07-11)** — first-class blocks/closures, Lua-style open/closed upvalues, frame-token infrastructure (ADR-0013/0006); an independent `phalcom-reviewer` pass caught the runtime being stubbed out on the first cut (block `call` unwired, upvalue opcodes unimplemented, a golden regression), which a follow-up pass closed — see below. **U5 ✅ landed (2026-07-11, `83c908a`)** — operators lowered to sends + sacred-selector inliner with override-epoch deopt guard (ADR-0018); reviewer gate OFF per policy (not load-bearing-hierarchy). **NEXT = U6 (absence → Option, let/var); reviewer ON.** |

## U5 — LANDED ✅ (2026-07-11, `83c908a` on `main`, no worktree)
- **Layer 0 — operators are sends.** Removed the hardwired arithmetic/boolean/comparison/equality
  opcodes; every operator now compiles to an `Invoke` message send via `encode_selector`. Backing
  primitives registered on `Number` (sub/mul/mod/div — IEEE-754 div allows `inf`/`NaN`, no error —
  /comparisons/negated), `Boolean` (and/or/not/ifTrue/ifFalse/ifTrue(_:ifFalse:)), `Block`
  (whileTrue), `Object` (generic `==`/`!=` via `value_eq`).
- **Surface `if`/`while`** parse in `phalcom-ast` (`parse_if`/`parse_while`), desugaring to
  `ifTrue(_:ifFalse:)` / `whileTrue(_:)` sends over block literals (DEC-E = "U5 owns", now realized).
- **Layer 1 — sacred-selector inliner** (`compiler/inliner.rs`): recognizes literal-block sacred
  sends and emits guarded jump opcodes (`Jump`/`JumpIfFalse`/`Loop`/`GuardBool`/`GuardBlock`) —
  zero closure alloc, zero frames on the common path. Guarded by override-epoch pristine flags in
  `universe.rs`, flipped by `note_method_installed` on `Bytecode::Method`; a runtime override of a
  sacred selector fails the guard and deopts to the real send. Binary `and`/`or` route through the
  inliner too. **ADR-0018** records the design + four deliberate deviations (see below).
- **Deliberate deviations from the U5 plan/spec (pre-authorized defaults):**
  (1) paired selector spelled `ifTrue(_:ifFalse:)` (keyword-labelled), not the spec's illustrative
  comma-form `ifTrue(_)ifFalse(_)` — matches Phalcom's actual selector model (ADR-0012);
  (2) jump offset width `i32`, not `i16` — inlining can grow a body past ±32 KB and silent
  truncation is a correctness bug;
  (3) **class reopening added** (`Statement::Class` attaches to an existing same-named global
  instead of shadowing) — prerequisite to make sacred-override *testable* from surface Phalcom;
  `install_core` now also registers kernel `Function`/`Block` as globals (were silently shadowed —
  the one real bug fixed this session);
  (4) `CallContext::Immediate` added — `Value::to_context` previously panicked invoking a
  closure-backed method on an immediate receiver (`Bool`/`Number`), which is exactly the post-deopt
  override path.
- **Two pre-existing goldens' `.expected` fixed as in-scope side effects** (they had pinned the
  old operator-dispatch bugs): `runtime_and_non_boolean_operand`, `class_instance_equality_identity`
  (and `runtime_comparison_unsupported`). Graduated from `pending/`:
  `class_operator_equals_custom_dispatch`, boolean short-circuit, `control_flow_if_else`. New
  goldens: `control_flow_inline_{override_honored,non_local_return}`, `control_flow_send_equivalence`,
  `control_flow_while_let`, `arithmetic_comparisons`, `runtime_inline_guard_wrong_type`.
- **Green gate:** `cargo build`/`cargo test --workspace`/`cargo doc --workspace --no-deps` all clean.
  Reviewer gate OFF (U5 is not hierarchy-load-bearing; policy line below lists ON units).
- **Working model:** in-tree on `main`, no worktree (handoff-authorized precedent from U2/U4).

## U4 — LANDED ✅ (2026-07-11, in-tree on `main`, no commit yet this turn)
- First-class blocks: `Expr::Block` in the AST/parser (braced multi-param, unbraced single-param
  expression-only, trailing-block sugar), postfix `(…)` desugars to `call(_:…)` (functions.md §1-2).
  `Value`/heap gain `BlockObject` (`block.rs`, new) wrapping a `ClosureObject` handle + a
  `FrameToken`, and a heap-owned `Upvalue` cell (`upvalue.rs`, new) that is `Open(stack_index)`
  while the enclosing scope is live and gets promoted to `Closed(Value)` on scope/frame exit
  (Lua-style, ADR-0013). Four new opcodes (`Closure`/`GetUpvalue`/`SetUpvalue`/`CloseUpvalue`)
  execute in the VM; `Function`(abstract)/`Block` installed under `Object` as siblings of `Method`;
  `verify_invariants()` stayed green throughout.
- **Frame-token infrastructure only** — `CallFrame` carries a monotonic generation, `BlockObject`
  stores the token of its creating activation, but **zero non-local-return behavior shipped**: no
  `ReturnNonLocal` opcode, no unwind logic, no such test. That is U10's job.
- **Two-pass landing, independently reviewed mid-flight.** The first cut had the front end +
  type scaffolding right but the runtime was stubbed (`block.call` returned "not wired yet",
  `GetUpvalue`/`SetUpvalue` were `RuntimeError::Internal`, `Closure` never allocated a
  `BlockObject`) and it silently regressed the `example_calculator` golden. A `phalcom-reviewer`
  pass caught this and returned `request-changes`. The gaps were then closed in the same session:
  `block_call`/`block_call_with` now push a real `CallFrame` and re-enter `VM::run_until`;
  `call`/`call(_:)`/`call(_:_:)`… registered per-arity (Phalcom dispatch keys on the arity-encoded
  selector, not a variadic entry point); the golden regression and a compiler raw-pointer soundness
  concern the reviewer flagged had already been fixed in-flight; `blocks()` in `tests/lang.rs`
  un-pended with 6 real cases (capture/escape/shared-mutation/`call`/`arity`); 2 new block goldens
  added (`blocks_map_reduce.ph`, `blocks_escaping_counter.ph`); 5 `cargo doc` warnings and 1 clippy
  warning in new code fixed. Final state: `cargo build --workspace` / `cargo test -p phalcom-core`
  / `cargo doc --workspace --no-deps` / `cargo clippy --workspace` all clean (one pre-existing,
  out-of-write-set clippy warning in `error.rs` left untouched).
- **Working model:** in-tree on `main`, no worktree (handoff-authorized precedent from U2).

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
