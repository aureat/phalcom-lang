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
| 3. Implement/Review | ⏳ in progress | U0 APPROVED (F9+F10). U-FE ✅, U3 ✅ (ADR-0012), U1 ✅ landed `6515ea3` — handle/arena heap + tagged Value (ADR-0009/0010). U2 ✅ landed — metaclass tower parallel rule + `Behavior` kernel + `verify_invariants()` (ADR-0002/0003); reviewer gate explicitly SKIPPED per user instruction, see [U2-progress.md](U2-progress.md). **U4 ✅ landed (2026-07-11)** — first-class blocks/closures, Lua-style open/closed upvalues, frame-token infrastructure (ADR-0013/0006); an independent `phalcom-reviewer` pass caught the runtime being stubbed out on the first cut (block `call` unwired, upvalue opcodes unimplemented, a golden regression), which a follow-up pass closed — see below. **U5 ✅ landed (2026-07-11, `83c908a`)** — operators lowered to sends + sacred-selector inliner with override-epoch deopt guard (ADR-0018); reviewer gate OFF per policy (not load-bearing-hierarchy). **U6 ✅ landed (2026-07-11, `3bc6ede`/`5b239ab`/`318e752`/`51f56e4`)** — absence → Option, `let`/`var`, no surface `nil`, no-truthiness enforcement (ADR-0007/0014/**0021**); reviewer ON, BLOCKed once on inlined≠non-inlined body result, fixed in `51f56e4`, then PASSED. U5✅→U6✅. **U7 ✅ landed (2026-07-11, `f38e591`/`561f7e2`, in-tree, no worktree)** — fixed `Box<[Value]>` instance slot layout + `construct` initializer + class-side stored static fields (ADR-0011/ADR-0017); reviewer OFF per policy, self-verified on the green gate. See "U7 — LANDED" below. **ADR-0019/0020 ratified by the user (2026-07-11)**, clearing U-LIST-plan §0's gate. **U-LIST ✅ landed (2026-07-11, `c7c63fb`/`6fdf0c7`/`b2f7aec`, in-tree, no worktree)** — native `List` heap variant + floor primitives + `.ph` protocol; also fixed a pre-existing bug that made `core.ph` inert (see "U-LIST — LANDED" below). **U8 ✅ landed (2026-07-12, `b99ad22`/`806c9ea`, in-tree, no worktree)** — `doesNotUnderstand(_:)` miss forward + `Message` reification + `send_dynamic`/`perform`/`respondsTo` (ADR-0012); reviewer OFF per policy, self-verified on the green gate. See "U8 — LANDED" below. **U9 ✅ landed (2026-07-12, in-tree, no worktree, uncommitted at write time)** — rest parameters (`*name`), `SignatureKind::Variadic`/`(*)` selector encoding, VM call-prologue rest-arg collapse, derived-selector miss-path probe (messages-and-selectors.md §4); reviewer OFF per policy, self-verified on the green gate. See "U9 — LANDED" below. **U10 ✅ landed (2026-07-12, in-tree, no worktree)** — non-local `return` inside blocks (`Bytecode::ReturnNonLocal` + eager frame-token unwind + `DeadFrameError`), consuming U4's frame-token infra (ADR-0013, blocks.md §5); reviewer OFF per policy, self-verified on the green gate (corrected the spec's Primitive-arm guard — re-push, don't skip, since the drain check pops). See "U10 — LANDED" below. **U-STD ✅ landed (2026-07-12, in-tree, no worktree)** — Option (B) per user ratification: the `Option` (`map`/`flatMap`/`filter`/`ifSome`/`unwrapOr`) + `List` (`map`/`reduce`/`filter`/`includes`/`isEmpty`/`at(_:put:)`) combinator layer, all pure `.ph` over the frozen floor, zero new primitives (catalog-delta §2.2/§2.4); discharged the `core.ph` map/reduce/filter deferral + DEFERRED #25; reviewer OFF per policy, self-verified on the green gate. See "U-STD — LANDED" below. **U-LEX ✅ landed (2026-07-12, in-tree, no worktree)** — surface-syntax delta: block comments `/* … */`, numeric digit separators `1_000_000`, lexer-level newline suppression, `?.`/`??` coverage (U6-landed, fixture only), and `\(expr)` string interpolation (**ADR-0022**, user-ratified sigil override of the architect's `{expr}`); reviewer OFF per policy, self-verified on the green gate. See "U-LEX — LANDED" below. **U11 ✅ landed (2026-07-12, in-tree, no worktree)** — Bool tower: abstract `Bool` + concrete singleton subclasses `True`/`False` (ADR-0004); `Value::class` selects `True`/`False` by payload (`true.class == True`), the six sacred control selectors stay on `Bool` and are inherited (KEEP, D1); zero new floor primitives, no `Value` variant, goldens byte-identical; reviewer OFF per policy, self-verified. See "U11 — LANDED" below. **NEXT = U-CORE-N wave — not yet dispatched (do not co-schedule with U11-touched surfaces; see U11-spec §0.1 BD-U11-SCHED).** |

## U-LEX — LANDED ✅ (2026-07-12, in-tree on `main`, no worktree)
- **Scope:** the five-part surface-syntax delta from U-LEX-implementation-spec.md, all in
  `phalcom-ast` (+ `lexical` corpus). **`phalcom-core/src` and `core.ph` untouched.** New ADR: **ADR-0022**.
- **D1 — block comments `/* … */`.** Extended `skip_trivia` (now returns
  `Result<(), LexicalError>` — spec option (a), signature change threaded through the sole caller
  `next()`); flat/non-nesting. EOF before `*/` → new `LexicalError::UnterminatedBlockComment(open..pos)`,
  lowered in `lex_error_to_syntax` → existing `SyntaxErrorKind::UnterminatedComment` with the **real**
  offset-adjusted span (not `0..0`). `error.rs` untouched. Lexer snapshot + PASS `comments_block` +
  NEGATIVE `syntax_unterminated_block_comment`.
- **D2 — digit separators `1_000_000`.** `scan_number` accepts interior `_` between digits (new
  `scan_digits` helper), stripped before `parse::<f64>()`. Misplaced `_` (trailing/doubled/adjacent
  to `.`) → `LexicalError::InvalidToken` (**reused**, no new `SyntaxErrorKind`). `Token::Number`
  unchanged. Promoted `pending/lexical_numeric_separator`, added float PASS + NEGATIVE `1__0` +
  a lexer snapshot.
- **D3 — newline suppression (load-bearing).** New `Lexer.last_significant: Option<Token>` +
  free `suppresses_following_newline(prev)` predicate; `next()` loops and swallows a `Token::Newline`
  when the previous significant token cannot end a statement. **Suppressor set (committed):** `+ - * / %`,
  `== != < <= > >=`, `and or not`, `= += -= *= /= %=`, `?? ?.`, `, ( { [ . ::`, `-> =>`, **and `Colon`**
  (included per spec §1's map/label shape — the one judgment call). One-sided (keys on prev only), NOT
  parser ASI. **Only one existing snapshot blessed:** `class_with_static_method` loses exactly one
  `Token::Newline` (the one after its first `{`) — diff is a single removed `Newline,` line. Two
  in-crate recovery tests (`recovers_and_reports_multiple_errors`, `recovers_across_multiple_broken_statements`)
  had sources whose lines *ended in operators* → D3 legitimately joined them; both were updated to
  value-ending lines (intent preserved, one snapshot re-blessed). **Full golden + `lang` corpus stayed
  byte-identical.** New continuation/logical/guard PASS fixtures.
- **D5 — `?.`/`??` coverage (no code).** U6 already shipped the operators (parser desugar to
  `orElse`/`map`); added one `lexical` PASS fixture `lexical_option_operators` exercising both
  end-to-end. Did **not** touch the U6 lexer arm / `parse_coalesce` / `parse_optional_send` / tokens.
- **D4 — string interpolation `\(expr)` (ADR-0022, was BLOCKED-ON-DECISION, now user-ratified).**
  User overrode the architect's `{expr}` recommendation → **`\(expr)`** (Swift-style). New
  `Token::StringInterp(Vec<StringSegment>)` + `StringSegment` in `token.rs`; `scan_string` splits on
  `\(…)` (balanced parens; `\\(` = literal `\(`; plain strings still lex to `Token::String`). Parser
  desugars in place (no compiler-visible AST node, since the compiler is out of write-set — matches the
  `if`/`while`/`??`/`?.` idiom) to a `+`-chain of `String` literals and **`String.new(expr)`** stringify
  sends. **Deviation:** spec's illustrative desugar used `expr.toString`, but value-type content
  `toString` doesn't exist yet (blocked on U-CORE-4); `String.new(_)` is the working content-stringify
  today (DEFERRED #30). Promoted `pending/lexical_string_interpolation` (rewritten to `\(expr)`) + added
  multi-expr/escape PASS + two lexer snapshots.
- **Green gate:** `./scripts/verify.sh` exit 0; `cargo doc --workspace --no-deps` no new `phalcom-ast`
  warnings (the lone remaining warning is the pre-existing `phalcom-core` `some_new`→`wrap_some`, DEFERRED #26).
  Reviewer OFF per policy — self-verified.
- **New ADRs:** **ADR-0022** (string-interpolation `\(expr)` sigil). **DEFERRED:** #30 (desugar target),
  #31 (nested-string-in-interp edge), #32 (nested block comments / lone-`?` fold into #12).
- **Working model:** in-tree on `main`, no worktree. Committed per green checkpoint (D1→D2→D3→D5→D4).

## U11 — LANDED ✅ (2026-07-12, in-tree on `main`, no worktree)
- **Scope:** the Bool tower (ADR-0004) — abstract `Bool` + concrete singleton subclasses
  `True`/`False`, per **U11-implementation-spec.md** (supersedes U11-plan.md). Tiny, purely
  additive, mostly Rust. **Zero new floor primitives** (census stays 80); **no `Value` variant**
  (`Value::Bool(b)` unchanged).
- **D1 = KEEP (resolved by the spec §0.1, no user escalation needed).** The six sacred control
  selectors (`not`/`and`/`or`/`ifTrue`/`ifFalse`/`ifTrue:ifFalse:`) **stay native primitives on
  abstract `Bool`**; `True`/`False` are near-empty and inherit them. MOVE was rejected — it would
  have to extend `note_method_installed` (hard-keyed to `bool_class`), re-prove the U-CORE-2
  `Some`-lift ≡ inliner parity, and add floor bindings. **No methods land on `True`/`False`, so the
  epoch hook's `bool_class` key is untouched.**
- **The one behaviour-changing edit:** `value.rs::class` `Value::Bool` arm now selects
  `true_class`/`false_class` by the payload (`true → True`, `false → False`) — a plain `ClassId`
  field read, allocation-free, on the hot dispatch path. So `true.class == True`,
  `false.class == False`; `Bool` is never a direct class. The sacred-selector inliner (`GuardBool`)
  keys on the `Value::Bool` **representation** + `bool_sacred_pristine`, **not** class identity, so
  the split is invisible to it; the deopt path resolves through `True`/`False` → inherits `Bool`'s
  primitive.
- **Wiring:** `universe.rs` — `True`/`False` rows in `create_core_classes` (super `bool_class`) +
  two `CoreClasses` fields; `vm.rs::install_core` — `add_class!(true_class/false_class)` globals;
  `primitive/mod.rs` — `ClassName::True/False` consts; `core.ph` — empty `class True {}` /
  `class False {}` reopens (surface-visibility parity; harmless no-ops — their globals name the
  class objects, not a singleton, unlike `None`).
- **Untouched (spec-confirmed):** `boolean.rs`, `primitive/boolean.rs` (sacred primitives +
  `bool_class_new` + the DEFERRED #26/§0.3 `println!`s verbatim), `install_primitives`'s Bool block,
  and `verify_invariants` (**re-run, not edited** — U-CORE-1's domain; it passes with the two new
  rows since `make_core_class` wires each metaclass by the same ADR-0002 parallel rule). Did **not**
  touch `PHASE2-INDEX.md`.
- **Tests:** three new PASS fixtures in the already-active `tests/lang/booleans/` label —
  `bool_class_identity` (class identity + `superclass`), `bool_sacred_through_split`
  (`!`/`and`/`or` through the class split + rendering), `bool_iftrue_option` (U-CORE-2 `Some`-lift
  survives the split). The two existing short-circuit fixtures stay byte-identical.
- **Goldens byte-identical:** `Value::to_string` untouched, so all existing Bool output (`true`/
  `false`) is unchanged; `examples/*` and `tests/fixtures/golden/*` unchanged.
- **Docs:** object-model.md §3/§4 reconciled with ADR-0004 (`True`/`False` surface-visible, `Bool`
  abstract) — the doc-only edit PHASE2-INDEX §5 assigned to U11, landed here.
- **Green gate:** `./scripts/verify.sh` exit 0; `cargo doc --workspace --no-deps` no new warnings
  (the lone remaining warning is the pre-existing `some_new`→`wrap_some`, DEFERRED #26). Reviewer OFF
  per policy — self-verified (proved `true.class == True` **and** `ifTrue`→`Some` / `and`/`or`
  short-circuit through the split).
- **Working model:** in-tree on `main`, no worktree. Committed per green checkpoint. **Hard stop:**
  did not begin any U-CORE-N work.
  **Hard stop:** did not begin U11/U-STD/U-CORE-N.

## U-STD — LANDED ✅ (2026-07-12, in-tree on `main`, no worktree)
- **Scope: Option (B) per user ratification** (U-STD-implementation-spec.md §0.1). The plan's literal
  Object/Number/String/Symbol/System surface was ~90% already-landed or re-carved to future
  `U-CORE-N` units; U-STD built the genuinely-remaining, unblocked, additive `.ph` residual — the
  **Option + List combinator layer** (§2.6). **Zero new native primitives; no `primitive/*.rs`,
  `universe.rs`, `vm.rs`, `bytecode.rs`, or `phalcom-ast/*` touched.**
- **`Option` combinators (core.ph `Option` block, over the native `match` eliminator;
  values-and-absence.md §3.3, catalog-delta §2.2):** `map(f)` (lift + re-wrap), `flatMap(f)`
  (monadic bind, no re-wrap), `filter(pred)` (`Some(v)` kept iff `pred(v)`, else `None`, via
  value-yielding `if/else`), `ifSome(f)` (effect + `self` passthrough, mirror of `ifNone`),
  `unwrapOr(default)` (eager extract).
- **`List` combinators (core.ph `List` block, over `each`/`add`/`rawSet`/`List.new`;
  catalog-delta §2.4):** `map(f)`, `filter(pred)`, `reduce(init, f)`, `includes(x)`, `isEmpty`,
  `at(i, put:)`. **Selector spellings:** `reduce(_:_:)` — 2 positional args, the trailing block
  desugars to the 2nd (`l.reduce(0) { acc, x => acc + x }`); `at(_:put:)` — matches `rawSet`'s arity,
  labeled param named `put` (label == name), returns `self` for chaining. None of the combinators
  stringify an element (avoids the `toString`-message class-name trap, DEFERRED #19).
- **Discharged the `List`-block header comment** (`core.ph`): the "do not add `map`/`reduce`/`filter`"
  deferral is now false; reworded to note those bodies live below and **only list-literal syntax**
  `[a, b, c]` remains deferred (DEFERRED #6/#28). **Did not add list-literal syntax.**
- **Tests:** new active `option` lang label (`option_map_both_arms`, `option_filter`, `option_flatmap`,
  `option_ifsome_effect_and_passthrough`) + new `list/` PASS cases (`list_map_and_filter`,
  `list_reduce_sum`, `list_includes_and_isempty`, `list_at_put`). The `absence` label was **not**
  un-ignored (its `#[ignore]` reason is unrelated drift, §0.6); `system()`/`system_pending()` untouched.
- **Discharged DEFERRED #25:** `blocks/pending/blocks_argument_to_method.ph` was rewritten off the
  real `List.reduce` (list built with `List.new()`/`add(_)`, no literal) and **promoted** to the
  active `blocks/blocks_argument_to_method.ph`; the empty `blocks/pending/` dir was kept on disk so the
  ignored `blocks_pending` probe still finds a directory.
- **Goldens byte-identical:** `examples/core_new.ph`, `person2.ph`, `calculator.ph`, and
  `tests/fixtures/golden/*` unchanged (methods added only).
- **Green gate:** full `phalcom-core` suite + goldens green; `cargo doc --workspace --no-deps` no new
  warnings (Option B added no Rust). Reviewer OFF per policy — self-verified.
- **Working model:** in-tree on `main`, no worktree. **Hard stop:** did not begin U11/U-CORE-N.
- **Note for the orchestrator:** U11 (Bool tower) also edits `core.ph` and was held back for U-STD —
  it is now unblocked. The forge-index vs. `docs/spec/core/` scope-taxonomy divergence is filed as
  DEFERRED #29 (resolved for this unit via Option B).

## U10 — LANDED ✅ (2026-07-12, in-tree on `main`, no worktree)
- **Non-local return (`return` inside a block unwinds to the enclosing method, blocks.md §5,
  ADR-0013).** New `Bytecode::ReturnNonLocal` (no operand — the unwind target is read off the
  executing frame). The compiler emits it in place of `Bytecode::Return` for a `return` in a
  block-literal body: `FunctionState.is_block` (set `!is_method` in `compile_block`) gates the
  opcode choice in `Statement::Return`; method/constructor bodies keep `Bytecode::Return`.
- **Frame plumbing.** `CallFrame.home_frame_token: Option<FrameToken>` (`None` for ordinary
  method/closure calls, kept `Copy`). `primitive::block::resolve_callable` now surfaces the block's
  `home_frame_token` alongside the closure handle (`None` for a bare `Object::Closure`), and
  `block_call` stamps the pushed `CallFrame` with it (post-construction assignment — `CallFrame` is
  `Copy`, so `new_call_frame`'s signature stays stable). U4's `closure.rs`/`callable.rs` and the
  `Bytecode::Closure` handler were **not** touched.
- **Eager unwind (`vm.rs` `Bytecode::ReturnNonLocal` handler).** Every block invocation re-enters
  `run_until` recursively, so the home frame is always in an outer, suspended `run_until`. The
  handler unwinds in one shot at the point `return` executes: read the executing frame's
  `home_frame_token`; if no live frame matches `(frame_index, generation)`, raise
  `RuntimeError::DeadFrameError` **before** mutating any state; else `close_upvalues_from(home
  offset)` **before** truncating the stack, truncate the value stack to the home offset, push the
  (surfaced-to-`None`) return value, and `frames.truncate(home index)`. It does **not** `return
  Ok(_)` out of `run_until` — the unmodified top-of-loop drain check in each nested `run_until`
  picks the value up.
- **Primitive-arm guard (`call_method`, corrected vs U10-implementation-spec.md §2 pt3).** Snapshot
  `frames_before` before calling `native_fn`; if the frame count shrank, a non-local return unwound
  past this call site. The spec said to skip *both* the truncate and push — but that loses the value,
  because `run_until`'s drain check *pops* the value the handler pushed and returns it via `Ok`. The
  landed guard instead **skips only the stale `truncate(receiver_idx)` and re-pushes** the returned
  value, so it is re-established for the outer frame that resumes; each unwound level's push balances
  its drain-pop exactly (no duplicate, no loss). Verified specifically on the multi-level
  `findNegative`/`each`-calling-`.call()` case (a single-level `{ return x }.call()` never crosses
  more than one `run_until` boundary and would not have caught this).
- **Tests:** `blocks/blocks_non_local_return.ph` (multi-level `each` unwind → `-5`, PASS) and
  `blocks/blocks_non_local_return_bare.ph` (value-less `return` in a block surfaces `<None instance>`,
  PASS) — both cross a re-entrant `block_call`. `runtime-errors/runtime_non_local_return_dead_frame.ph`
  (escaped block called after its home method returned → `DeadFrameError`, NEGATIVE).
  `blocks/blocks_escape.ph` still passes byte-identical (upvalue promotion across the new unwind).
- **Pending fixtures (U10-implementation-spec.md §0 pt4):** `pending/blocks_non_local_return.ph`
  (`[3,-5,8]` list literal — doesn't parse) was rewritten off `List.new()`/`.add(_)` and **promoted**
  to `blocks/blocks_non_local_return.ph`; the stale pending pair was deleted.
  `pending/blocks_argument_to_method.ph` (`List.reduce(_)` — not in the kernel) stays pending with a
  comment noting it is blocked on U-STD's `reduce`, **not** on U10 (DEFERRED #25).
- **Green gate:** `./scripts/verify.sh` exit 0; `cargo doc --workspace --no-deps` clean (no new
  warnings — one pre-existing `nil.rs` `wrap_some` private-link warning is unrelated). Reviewer OFF
  per policy — self-verified, including the dead-frame path and upvalue-across-unwind promotion.
- **Working model:** in-tree on `main`, no worktree.
- **Hard stop:** did not begin U11/U-LEX/U-STD.

## U9 — LANDED ✅ (2026-07-12, in-tree on `main`, no worktree)
- **Rest parameters (`*name`, messages-and-selectors.md §4).** `ParameterDef.is_rest` (ast.rs);
  `parse_param_list` parses an optional leading `*` and rejects (clean diagnostic, not a panic) a
  rest parameter that isn't the list's last entry or that carries/follows a label. Block-literal
  params are parsed by a separate scanner in `Parser::parse_primary` and never reach
  `parse_param_list`, so no block-literal guard was needed there — block variadics still don't
  parse at all (DEFERRED #9, confirmed still open).
- **`SignatureKind::Variadic(u8)`** (payload = fixed/minimum positional arity `F`) in `method.rs`
  (the plan's "signature.rs" — that module is a dead stub, untouched). Selector spelling is
  always the bare `<name>(*)`, independent of `F` — `sum(*numbers)` and `format(fmt, *args)` both
  intern as `sum(*)`/`format(*)`; only `Signature.positional_arity`/`variadic` (set from the
  payload in `Signature::new`) distinguish them at runtime. `decode_selector`'s `Variadic` arm
  round-trips the name but not `F` (documented limitation — the selector text never carries it;
  only the dNU `Message`-reification path uses this today, which doesn't need the real `F`).
- **Compiler:** the `ClassMember::Method` arm computes `F = params.len() - 1` and selects
  `SignatureKind::Variadic(F)` over `SignatureKind::Method(arity)` when the last param `is_rest`;
  `compile_block` itself needed zero changes (params.len() already includes the rest param as an
  ordinary trailing local slot).
- **VM call prologue (`call_method`'s `MethodKind::Closure` arm).** Before building the new call
  frame: if the target method's signature is variadic, `Vec::split_off` the trailing
  `arity - fixed_arity` positional args off the value stack, wrap them in one `List`
  (`heap.alloc_list`), and push it back — `receiver_idx`/`stack_offset` are computed before this
  mutation and are unaffected by the tail collapse, so `CallFrame` slot addressing needs no other
  change. Verified by a black-box stack-depth-invariant golden (200 variadic calls in a loop).
- **Runtime dispatch probe** filling the `[U9 SEAM]` in `Bytecode::Invoke`'s miss arm: only an
  all-positional `SignatureKind::Method` selector (decoded via `decode_selector`) probes for a
  `<name>(*)` candidate via one ordinary `lookup_method` walk; a hit dispatches only if
  `arity >= positional_arity`, otherwise (same as an outright miss) falls through to the existing
  `forward_does_not_understand` — no new error variant, no duplicated dNU body.
- **Deliberate scope calls (per U9-implementation-spec.md §0, within handoff latitude):**
  (1) no new "variadic table" — reuses `ClassObject.methods: IndexMap<Symbol, ObjRef>` under the
  `(*)` selector; a same-name duplicate variadic silently overwins, same as any duplicate-selector
  redefinition today (DEFERRED #24). (2) no `callable.rs`/`closure.rs` changes — the variadic flag
  is read from `MethodObject.signature` directly in `call_method`. (3) **no
  `Bytecode::SendDynamic` / call-site spread (`f(*args)`)** — DEFERRED #21's forward-looking note
  ("U9 owns both the opcode and its handler") is superseded; spread-call syntax remains a future
  unit's job, `bytecode.rs`/`disasm.rs` untouched.
- **Tests:** new `variadics` PASS golden group (zero-prefix, fixed-prefix `F=1` prologue math,
  fixed-vs-variadic coexistence/dispatch-ordering, dNU-fallback-preserved, real-`List` rest
  binding, stack-depth invariant) + 2 new `syntax-errors` NEGATIVE goldens (rest not last, rest
  labelled). Two pre-existing `clippy::useless_conversion` nits in `parse_param_list` (unrelated
  to this unit's logic, already present in the parser diff this unit builds on) cleaned up in
  passing.
- **Green gate:** `./scripts/verify.sh` exit 0; `cargo doc --workspace --no-deps` clean (no new
  warnings); clippy clean. Reviewer OFF per policy — self-verified.
- **Working model:** in-tree on `main`, no worktree.
- **Hard stop:** did not begin U10/U11/U-LEX/U-STD.

## U8 — LANDED ✅ (2026-07-12, `b99ad22`/`806c9ea` on `main`, no worktree)
- **Lookup-miss → `doesNotUnderstand(_:)` forward (method-lookup.md §2, ADR-0012).** The
  `Bytecode::Invoke` miss arm no longer raises a hard error; it reifies the missed send as a
  `Message` and re-sends it as `doesNotUnderstand(_:)` up the receiver's chain, so a subclass
  (proxy) can intercept. A recursion guard: a receiver whose chain somehow lacks
  `doesNotUnderstand(_:)` is `RuntimeError::Internal`, never re-sent as another dNU.
- **`VM::send_dynamic(receiver, selector, args)` — the shared runtime-send workhorse.** Saves
  the frame count, pushes receiver+args at a fresh stack window, dispatches via
  `lookup_method` + `call_method` (falling through to the same dNU forward on a miss), then
  re-enters `run_until` to drain that one activation and return a synchronous `Value` — the
  same re-entrancy pattern as `block_call`, so it is callable from inside a primitive. Three
  consumers: `Object.perform(_:)` / `perform(_:_:)`, the dNU forward, and (deferred) a U9
  `SendDynamic` spread opcode.
- **`Message` = Rust-built four-slot `InstanceObject`, no `.ph`.** `VM::new_message` constructs
  it directly (slots `selector`/`name`/`labels`/`args`); field count stamped in `VM::new`
  mirroring `Some` (a `class X {}` reopen never applies a compiler field layout to a
  bootstrapped row, so a `.ph` `construct` would not work). Accessors are native getters on
  `message_class`; `labels` uses `""` for a positional argument so `labels.size == args.size`.
- **`method::decode_selector`** — the exact inverse of `encode_selector`, total (never panics;
  garbage → `Getter`), used for `Message` name/labels decomposition. 5 unit tests (round-trip
  across all six `SignatureKind`s, labeled selectors, setter-vs-operator disambiguation,
  garbage totality, subscripts).
- **`Object.respondsTo(_:)`** — pure exact-selector probe, never triggers dNU.
- **`RuntimeError::MethodNotFound` retired → `MessageNotUnderstood { selector, receiver }`**, the
  default-dNU raise (rendered `"{receiver} does not understand '{selector}'"`). **Four**
  behavior-change goldens updated (not one): `runtime_unknown_method`,
  `runtime_and_non_boolean_operand`, `runtime_comparison_unsupported`,
  `runtime_inline_guard_wrong_type`.
- **Deliberate scope calls (implementer, within handoff latitude):** (1) **no `Bytecode::SendDynamic`
  opcode this unit** — per BD-U8-2 nothing emits/decodes a spread call site yet, so a dead opcode
  with a guessed operand layout would be untestable and pre-empt U9's design; delivered the
  `send_dynamic` *helper* instead, opcode deferred to U9 (DEFERRED #21). (2) **No `core.ph` edit** —
  `doesNotUnderstand`/accessors are primitives and `add_class!` already registers the `Message`
  global, so the shared file stayed untouched (a subset of the plan's write-set). (3) dNU render
  format `"{receiver} does not understand '{selector}'"`.
- **Tests:** 5 new `dispatch` PASS goldens (Proxy/dNU forwarding, `Message` shape, `perform`
  parity, `respondsTo` true/false, dNU-preserves-dispatch / IC non-corruption) + 1 negative
  `runtime-errors` case (`perform` of an unknown selector re-enters dNU once, no loop) + the 4
  behavior-change goldens + 5 `method` unit tests.
- **Green gate:** `./scripts/verify.sh` exit 0; `cargo doc --workspace --no-deps` clean (no new
  warnings — `new_message` promoted to `pub` to keep intra-doc links valid); clippy clean.
  Reviewer OFF per policy — self-verified.
- **Working model:** in-tree on `main`, no worktree.
- **Stopped at the U8 hard boundary** — did not begin U9/U10/U11/U-LEX/U-STD.

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
