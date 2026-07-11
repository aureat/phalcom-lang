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
| 3. Implement/Review | ⏳ in progress | U0 APPROVED (F9+F10). U-FE ✅, U3 ✅ (ADR-0012), U1 ✅ landed `6515ea3` — handle/arena heap + tagged Value (ADR-0009/0010). U2 ✅ landed — metaclass tower parallel rule + `Behavior` kernel + `verify_invariants()` (ADR-0002/0003); reviewer gate explicitly SKIPPED per user instruction, see [U2-progress.md](U2-progress.md). **U4 ✅ landed (2026-07-11)** — first-class blocks/closures, Lua-style open/closed upvalues, frame-token infrastructure (ADR-0013/0006); an independent `phalcom-reviewer` pass caught the runtime being stubbed out on the first cut (block `call` unwired, upvalue opcodes unimplemented, a golden regression), which a follow-up pass closed — see below. **U5 ✅ landed (2026-07-11, `83c908a`)** — operators lowered to sends + sacred-selector inliner with override-epoch deopt guard (ADR-0018); reviewer gate OFF per policy (not load-bearing-hierarchy). **U6 ✅ landed (2026-07-11, `3bc6ede`/`5b239ab`/`318e752`/`51f56e4`)** — absence → Option, `let`/`var`, no surface `nil`, no-truthiness enforcement (ADR-0007/0014/**0021**); reviewer ON, BLOCKed once on inlined≠non-inlined body result, fixed in `51f56e4`, then PASSED. U5✅→U6✅. **U7 ✅ landed (2026-07-11, `f38e591`/`561f7e2`, in-tree, no worktree)** — fixed `Box<[Value]>` instance slot layout + `construct` initializer + class-side stored static fields (ADR-0011/ADR-0017); reviewer OFF per policy, self-verified on the green gate. See "U7 — LANDED" below. **ADR-0019/0020 ratified by the user (2026-07-11)**, clearing U-LIST-plan §0's gate. **U-LIST ✅ landed (2026-07-11, `c7c63fb`/`6fdf0c7`/`b2f7aec`, in-tree, no worktree)** — native `List` heap variant + floor primitives + `.ph` protocol; also fixed a pre-existing bug that made `core.ph` inert (see "U-LIST — LANDED" below). **NEXT = U8 (hard-stop boundary per `U-LIST-U8-implement-handoff.md`) — not yet dispatched.** |

## U-LIST — LANDED ✅ (2026-07-11, `c7c63fb`/`6fdf0c7`/`b2f7aec` on `main`, no worktree)
- **Native `List` heap variant (ADR-0020).** `ListObject { elements: Vec<Value> }` in new
  `list.rs`, mirroring `StringObject`; `Object::List` variant in `heap.rs` with
  `alloc_list`/`list`/`list_mut`/`as_list`. **Not** an `InstanceObject` — no U7 dependency,
  confirmed: created in `universe.rs::create_core_classes` the same way `Option`/`Bool`/`String`
  are, positioned right after `Option`/`Some`/`None`.
- **Five floor primitives (ADR-0019), not six.** `list_class_new` (public, backs `List.new()`),
  `list_raw_length`/`list_raw_at`/`list_raw_set`/`list_raw_push` (internal, `rawXxx`-named so
  `.ph` can wrap them without recursing on the public selector). No separate "grow" primitive —
  `rawPush` relies on `Vec::push`'s own amortized doubling (implementer's call, pre-authorized by
  the plan). `rawSet` is implemented and wired but **not** surfaced at the `.ph` layer this unit
  (no `at(_:put:)` yet — DEFERRED).
- **`.ph` protocol in `core.ph`:** `size => self.rawLength`, `at(_:)` and `add(_:)` wrap the raw
  primitives (`add` returns `self` for chaining), `each(_:)` is a `.ph` while-loop calling
  `f.call(self.at(i))` (proves block-calling into `List` iteration works). **`toString` is a
  native primitive, not `.ph`-defined** — deviation from the plan's suggested "each + concat"
  sketch, because no kernel primitive type has a general user-callable `.toString` yet (`Number`
  has none), so building it in `.ph` would render every non-`String` element as `"<ClassName>"`
  instead of its value. Recorded as a DEFERRED item: once `Number`/etc. get real `toString`
  primitives, `List.toString` can move to `.ph` over `each`.
- **Absence boundary.** `rawAt` returns `vm.none_value()` directly for an out-of-range index —
  never a panic, never the raw `Value::Nil` sentinel. A non-Number/negative/fractional/infinite
  index is `RuntimeError::Type` (reused, no new variant).
- **Found and fixed a pre-existing, codebase-wide bug while landing this: `core.ph` was
  registered as source (`VM::install_core`) but never actually compiled or executed** — not by
  the CLI, not by the test harness — so every existing `.ph` class-reopen skeleton
  (`Option`/`Some`/`String`/... ) was silently inert. `VM::new` now calls a new
  `VM::run_core_module()` right after `Universe::install_primitives`, which is what makes
  `List`'s `.ph` protocol (and every other core-class reopen) actually take effect. This in turn
  surfaced a second latent bug: `Statement::Class` unconditionally emits `DefineGlobal` at the
  end of every class body, reopen or not; for every other core class that's a no-op (the global
  already points at that class), but `None`'s global is deliberately bound to the shared
  singleton *instance* — so the (empty, purposeless) `class None {}` reopen was clobbering that
  binding back to the class the instant `core.ph` ran. Fixed by dropping that empty reopen
  (nothing was lost) and documenting the trap in `core.ph`; DEFERRED for whoever needs real
  `None` members later.
- **Tests:** new `list` corpus label (4 PASS goldens: construction/add/size/at, absence-at-
  out-of-range, `each` sum, `toString` bracket rendering) + 1 NEGATIVE in the shared
  `runtime-errors` label (non-Number index → type error). `MANIFEST.md` counts updated.
- **Green gate:** `cargo build --workspace` / `cargo test --workspace` / `cargo doc --workspace
  --no-deps` / `cargo clippy --workspace` all clean. Reviewer OFF per policy — self-verified.
- **Working model:** in-tree on `main`, no worktree.
- **Did not begin U8** — stopped at the hard boundary per `U-LIST-U8-implement-handoff.md`.

## U7 — LANDED ✅ (2026-07-11, `f38e591`/`561f7e2` on `main`, no worktree)
- **Fixed instance slot layout (ADR-0011).** `InstanceObject.fields: IndexMap<Symbol, Value>` →
  `slots: Box<[Value]>`; `GetField`/`SetField`'s `u16` operand is now a direct slot index (was a
  constant-index of the field `Symbol`; opcode arity unchanged, only the operand's meaning
  changed). Each `ClassObject` gets `field_slots: IndexMap<Symbol, u16>` + `field_count: u16`,
  computed once via a whole-class field-collection pass over every method/getter/setter/
  `construct` body (assignment order), then fixed. A subclass appends its own slots after the
  superclass's (`subclass_field_offset_stability` invariant test) — private fields are never
  renumbered or shared across the hierarchy.
- **Read-before-write is a compile error.** A field read whose name is in no assignment set
  anywhere in the class is `CompilerError::ReadBeforeWrite` (catches the `_naem` typo class of
  bug). Golden: `compile_error_field_read_before_write`.
- **`construct` initializer.** New `phalcom-ast` keyword; parses to
  `ClassMember::Construct(ConstructDef)`. Lowers to `SignatureKind::Initializer(arity)` — the
  selector kind `method.rs`/`encode_selector` already anticipated; no new selector kind, no
  change to selector encoding or method lookup (U3 untouched). Body: `NewInstance` alloc + bind
  `self` (`SetLocal(0)`) + run body + implicit `return self`. Installed class-side
  (`is_static=true` → metaclass), so `new(name:age:)` / `new(name:)` are two distinct
  `Initializer` selectors dispatched by arity/labels — no default args, no arity coercion
  (deliberately precluded per the plan's identity-dispatch⊗optional-arity hazard).
- **Class-side stored static fields (DEC-D, ADR-0017, user-ratified 2026-07-11).** Applies
  ADR-0011 one level up the tower: `ClassObject.static_slots: Box<[Value]>`, indexed by the
  **metaclass's** `field_slots` table. `static _count = 0` collects into the metaclass field
  table (not the class's), reads/writes target the class object's own `static_slots`, not
  `self`'s. Offset-stability holds up the tower too (`subclass_static_field_offset_stability`).
  ADR-0017 was **Accepted** before this slice landed, per the plan's gate.
- **Unassigned slot → `None`.** Both instance slots and static slots default to the private
  `Value::Nil` sentinel (never a constructed `None` — would reintroduce the bootstrap-absence
  cycle U6 solved) and surface as `None` via U6's helper on read. Goldens:
  `class_field_unassigned_reads_none`, `class_static_field_unassigned_reads_none`.
- **Constructor-dispatch bug found and fixed post-implementation.** `construct new()` installs
  under `"init new()"` (`Initializer`), but the call-site compiler for `Expr::MethodCall`
  *always* encoded `SignatureKind::Method` (`"new()"`) — so `Counter.new()` silently resolved to
  the inherited `Object::new` bare-allocation primitive instead of the constructor (no error;
  `_count` in `class_static_field_shared_state` just stayed `0`). Fixed with a compile-time
  `VM.constructor_aliases: HashMap<(Symbol, Symbol), Symbol>`: a literal `ClassName.method(...)`
  call site whose Method-style selector matches a class's declared `construct` is redirected to
  the `Initializer` selector. Also closed the matching negative case the plan required but no
  test exercised: a class with a `new`-named `construct` has no user-visible bare allocator — a
  mismatched-arity `new(...)` call is now `CompilerError::Message` via `VM.has_new_construct`,
  not a silent fallthrough. Golden: `compile_error_no_matching_constructor`.
- **Green gate:** `./scripts/verify.sh` exit 0; `cargo doc --workspace --no-deps` clean (no new
  warnings); `cargo clippy --workspace --all-targets` clean (also fixed one pre-existing
  `clone_on_copy` warning in `vm.rs`'s `Dup` handling, a file touched this unit).
  Reviewer gate **OFF** per STATE.md policy — self-verified on the green gate.
- **Working model:** in-tree on `main`, no worktree (handoff-authorized precedent).
- **Stopped at the U8/U-LIST hard boundary** per `U7-implement-handoff.md` — did not begin U8 or
  U-LIST implementation.

## U6 — LANDED ✅ (2026-07-11, `3bc6ede`/`5b239ab`/`318e752`/`51f56e4` on `main`, no worktree)
- **Absence → Option.** No surface `nil`; user code expresses absence exclusively through
  `Option` (ADR-0007). `Some`/`None` under abstract `Option`; `None` is a single shared
  singleton (identity-comparable, zero-allocation), `Some` carries one `_value` field.
  Construction is the explicit static send `Some.new(x)` (deliberate deviation — **no bare
  `Some(x)` call-construction syntax** in Phalcom). Sole eliminator is `match(some:none:)`
  (deviation — the keyword-labelled selector spelling, not a comma-form).
- **`Bytecode::Nil` → `None` surfacing (Invariant 4).** The VM keeps a private raw `nil`
  sentinel for **allocator/storage only** (uninitialized slots); it never reaches user code.
  Value-less positions surface as `None`: an empty/value-less block or method body yields
  `None`, a bare `return` returns `None` (deviation — `return` with no expr ≡ `return None`),
  a false `ifTrue` branch is `None`, `print`'s result is `None`, the root superclass reads
  `None`. `318e752` closed the Invariant-4 sentinel-leak; `51f56e4` restored inlined ≡
  non-inlined body results (value-less bodies yield `None` on both paths) — the reviewer's
  one BLOCK, independently confirmed fixed (all four repros print `<None instance>`).
- **`let`/`var` bindings (ADR-0014, BD-4).** `let` immutable, `var` mutable; `var x` with no
  initializer reads as `None`. `let` reassignment and `let` with no initializer are compile
  errors; surface `nil` is a compile error.
- **`??` / `?.` parser desugar (values-and-absence §3.4–3.5).** `a ?? b` and `opt?.foo`
  desugar in `phalcom-ast` (short-circuiting), threaded into the precedence table.
- **BD-U6-1 no-truthiness → Option A + new ADR-0021.** `Option`/`Some`/`None` never implement
  the boolean-branch protocol, so a non-`Bool` condition is a **hard runtime type error** (no
  coercion) via U5's `GuardBool` (ADR-0018). **Plus** the compiler rejects syntactically-literal
  Option conditions (`if (None)`, `if (Some.new(…))`) at compile time via `is_option_literal` /
  `branch_condition_of` (`compiler/lib.rs`). Refines spec §3.5's "compile error" to "compile
  error where statically detectable + hard runtime type error otherwise." **ADR-0021** records
  it (composes with U5's branch-opcode typing).
- **Deliberate deviations (pre-authorized defaults):** (1) `Some.new(x)`, not `Some(x)` — no
  call-construction syntax; (2) `match(some:none:)` selector spelling; (3) bare `return` → `None`.
- **Green gate:** `cargo build` / `cargo test --workspace` / `cargo doc --workspace --no-deps`
  all clean; `verify.sh` exit 0. Reviewer gate **ON** (load-bearing — can corrupt object model):
  BLOCKed on inlined≠non-inlined, fixed in `51f56e4`, re-verified, PASSED.
- **DEFERRED:** #13 (captured-`let` reassignment not rejected — the compile check is syntactic,
  indirection defeats it) and #14 (`if(opt)` literal-only detection + `OptionTruthiness`
  diagnostic carries no source span).
- **Working model:** in-tree on `main`, no worktree (handoff-authorized precedent from U2/U4/U5).

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
