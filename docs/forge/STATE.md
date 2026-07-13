# Forge session state — dispatching the U12–U20 / U-COLL batch

Three-worktree consolidation (U-INH + U-ITER + U-FIBER) landed `main` at `0de7496`,
green. Now dispatching the in-flight planning batch (U12–U20, U-COLL, U-COLLTYPES,
U-ERR, U-FUTURE) per `phase-next/INDEX.md` §build-order.

## Landed (do not redispatch)
Spine U0–U11 + U-FE/U-LEX/U-LIST/U-STD. Core track U-CORE-1..6 (floor → 88).
U-INH, U-ITER, U-FIBER. Gate green at `0de7496` (`./scripts/verify.sh`).
Later-landed (accepted): U-COLL, U-COLLTYPES, U-ITER-FIX, U-ERR, U15, U14, U16-Open,
U-LEX-HASH, U16-Pinned, U-ERR-FIX, U-FIBER-FIX, U-FUTURE-A, U-REOPEN-FIX. ADR-affirm-closed:
U12/U13/U17/U18. Floor → 113. Tip `e85f31a`.

## Landed this session
- **U-COLL** ✅ — collection literals. `1274504` (list) / `5bc31e8` (tuple) /
  `dc9eab0` (map §6 disambiguation + pending diagnostic). Gate green, hand-verified
  (`[1,2,3].map{…}`→`[2,3,4]`, `(1+2)*3`→9 grouping intact). Reviewer OFF, orchestrator-
  accepted. Net floor 0, no `core.ph`. `parse_comma_exprs` factored for U14. Pending
  `literal_tuple.ph`/`literal_map_symbol_keys.ph` graduate with U-COLLTYPES. One
  entailed out-of-write-set edit accepted: `syntax_unexpected_token.ph` (list literal
  made `[1,2,3]` valid → migrated to stray `=>`, intent preserved).

- **U-ITER-FIX** ✅ — loop-control follow-ons: `9288ad5` (deopt trap) · `ac4f721`
  (`while` break/continue) · `08a323b` (loop-var freshness) · `b566e6b` (jump dedup).
  Reviewer APPROVED (clean-worktree gate green; trap confirmed loud, exit 1). Closes
  U-ITER DEFERRED items 1/2/3/6. **Follow-ons (do when tree clean):** strike DEFERRED.md
  L21-24; give the deopt trap a descriptive `Error.new(_)` message (currently stderr `None`).

## Landed this session (cont.)
- **U-COLLTYPES** ✅ — native `Map`/`Set`/`Tuple`/`Range` arena arms + `.ph` protocol.
  `bdbdaaf`(ADR-0039)/`be8426e`(Map+Set)/`2d140f0`(Tuple-orphan)/`f934cf1`(Range+Phase2-spine)/
  `10e1715`(`{k:v}`→Map). Floor 88→109 (+21). **Reviewer BLOCK → ACCEPTED functional** at
  `10e1715` (green, sound, all load-bearing checks pass); block was history-honesty only
  (`2d140f0` orphaned dead code + false msg; Phase-2 spine actually in `f934cf1`). Squash OFF
  (shared main). Remedy = append-only as-built correction, plan.md §11. Reviewer non-blocking
  obs → DEFERRED.md (do when tree clean).
- **item5** ✅ — U-FIBER×generator fixtures `bf80c21`. Both PASS (for-generator suspends;
  each-generator raises `CannotYieldAcrossNativeFrame`). No bug. Mark DEFERRED item-5 resolved.

- **U13** ✅ — hierarchy-stability policy `5d84ad8`. Sealing was ALREADY enforced (ADR-0026,
  prior session; `class_set_superclass`→`InvalidSetSuper` "Can't set superclass of a class").
  Delivered: invariants test + golden negative + ADR-0041 (DEC-U13a/b=A; MI/C3 rejected, only
  stateless-traits-at-finalization pre-approved). Zero runtime-logic change → orchestrator-accepted.
  **New bug filed (DEFERRED):** method-reopening broken — 2nd `class A{}` block OVERWRITES not
  merges, violates ADR-0018 (pre-existing, repro'd on clean HEAD; compiler/lib.rs+Bytecode::Class).
- **item4** ✅ — List combinator migration `c35171a`. `each`→`for (x in self)` (protocol-driven,
  no block_call); map/filter/reduce/includes transitive. Behavior-preserving, goldens byte-exact.

## Single-writer phase (compiler-spine chokepoint — everything left touches compiler/lib.rs or core.ph)
- **U-ERR** ✅ LANDED `7c901cf` (42 files, +952/-68). throw=AST node→`.raise()` (no bytecode);
  try/on/catch/ensure=pure parser desugar (no new AST); `Block#on/ensure` native via `VM::unwind_to`;
  Result/Ok/Err pure `.ph`. **Floor +2** (block_class) under ADR-0038 (Accepted), census 109→111.
  Graduated 2 pending + 7 PASS + 2 negatives. Caught+fixed Error-subclass field-aliasing (reopened
  `Error` w/ `construct new(msg)` + bare `new()`; reverted a wrong compiler-fix that broke a golden).
  Verify green. **Reviewer APPROVE** (`ada613e4`) — all 8 focus areas pass (unwind_to order,
  try/on/ensure desugar, throw-reject scope, on isA-match, Error-reopen field layout, floor +2,
  fixture honesty, docs); spot-checks match real binary. **U-ERR ACCEPTED.** 3 non-blocking nits:
  ADR-0038 stale count (FIXED → 109→111), unwind_to doc overstates order-equiv (vm.rs, minor),
  multiple-catch not grammar-rejected (dead code, not unsound). DEC-ERR-B=(B): Ok/Err bare-call
  sugar deferred (filed).
- **U15** ✅ LANDED `6188973` — `import "./x" as X` (relative-path + whole-module bind), source-only,
  memoized `Module` registry (single HashMap, insert-before-compile = cycle marker), member access via
  `Module#doesNotUnderstand`. New `Bytecode::Import` + ADR-0045 + spec `modules.md`. **Floor +1**
  (Module DNU fails derivability) → ADR-0045 superseding amendment, census **111→112**, invariants
  lockstep — STOP-and-reported. 5 PASS + 2 NEGATIVE goldens + 8 lib companions; MANIFEST self-reconciled
  →367. Write-set deltas (cli.rs entry-path fix, interpret.rs scaffolding) reported. Verify green.
  **Reviewer APPROVE** (`a1a95771`) — all 9 focus areas pass, live-verified (cycle <5s no hang,
  deep-relative resolves to self not entry, no global pollution, floor +1 lockstep, source-only,
  single-HashMap design concurred correct). **U15 ACCEPTED.** 1 non-blocking filed to DEFERRED:
  `register_source` (vm.rs:396) SOURCE_MAP keyed by logical name not canonical path (same-basename
  diagnostic-source overwrite) + pre-existing duplicate insert.
- **U14** ✅ LANDED `0769316` — destructuring `let (a,b)=t` / `let [first,*rest]=xs`. Desugar to
  `at(_)` (single-RHS-eval into scratch local), arity guard → `Error.new().raise()` (clean), `*rest`
  last enforced at parse. ADR-0046 + spec destructuring.md + Q7→RESOLVED. **Floor 0** (pure desugar).
  6 positive + 4 negative goldens, all lane-correct (lane lesson applied). 7 snapshot re-baselines
  (LetBinding.name→pattern, legit). Verify green. **Reviewer APPROVE** (`a4cd9922`) — all areas pass,
  live-verified (single-eval, arity boundaries, nested, snapshot honesty); shipping list/*rest now
  judged sound. **U14 ACCEPTED.** No new DEFERRED (implementer self-filed 4 minor obs).
- **U16** — STOPPED pre-edit on 2 real plan-vs-reality blockers (adversarial gate worked), adjudicated
  + RE-SCOPED; re-dispatched fresh as U16-Open (reviewer ON):
  - **Blocker 1:** Pinned form `obj::#sel` needs `#`-symbol-literal lexing (selectors §2, unlanded,
    lexer.rs/token.rs — outside U16 write-set). **DEFER Pinned** → new prerequisite unit **U-LEX-HASH**.
  - **Blocker 2:** Q14 already RESOLVED in spec (Family callable-only; reflective mirror deferred to a
    unified reflection unit). Plan §4/§6 was stale. **Honor the ruling** → callable-only, NO reflective surface.
  - **U16 now = Open-form `::` only, callable-only:** `obj::m`/`Type::m` → `Object::Family` heap variant,
    `family_class#doesNotUnderstand` call-router (encode_selector→send_dynamic, no new dispatch), base-name
    index via new `FinalizeClass` opcode, empty-family check honors DNU. Floor +1 (the one call-router) →
    ADR-0047. Does NOT touch lexer.rs/token.rs.
  - **LANDED** `dfb96ff` (feat) + `41b7227` (deferred-filing). Gate green, cargo doc clean. As-built:
    Family defines ONLY `doesNotUnderstand(_:)`; bare `f(args)`→`call(...)` misses→DNU router
    `decode_selector`→`encode_selector(name,labels,kind)`→`send_dynamic` (U8 path, no 2nd dispatch).
    TWO opcodes: `FinalizeClass` (class-tail, finalizes base_names incl. metaclass) + `MakeFamily`
    (compiles `Expr::MethodRef`, builds Family + reference-time empty-check). `finalize_all_core_base_names`
    in `VM::new` covers kernel rows w/ no `.ph` reopen. `ClassObject.base_names: HashMap<Symbol,Vec<Symbol>>`.
    Floor 112→113 (§2.16), invariants.rs `core_class_rows` 27→28 lockstep. Census real path =
    `docs/spec/v0.2/core/floor-census.md` (brief typo said docs/forge; no docs/forge/floor-census.md exists).
    6 goldens (`tests/lang/family/`, 5 pos + 1 negative-lane). Empty-check verified DNU-honoring live.
    **Reviewer APPROVE** (`a07b3f78`) — all 8 load-bearing checks PASS (single-dispatch, inheritance-flatten,
    DNU-honoring empty-check, floor +1 lockstep, two-opcode wiring, Value-minimality, lane-correct goldens,
    rustdoc), verify green + doc clean in throwaway worktree, no spec deviation, `Type::m` metaclass path
    confirmed. **U16-Open ACCEPTED.** LAST feature unit complete. 2 DEFERRED filed (IC population,
    Family/Method.bind unification). → serial spine tail begins (U-LEX-HASH next).

## Roster update (post-U16 blocker adjudication)
- **U-LEX-HASH** (NEW prerequisite) — `#` symbol literals (selectors §2): atomic Logos token, R2
  validate/canonicalize, shebang-offset-0 carve-out. Owns lexer.rs + token.rs. Unblocks: U16-Pinned +
  the known `#IDENT` map-symbol-key DEFERRED item (line 12) + future `perform`/reflection selector symbols.
  Contends phalcom-ast → serialize after U16-Open frees it.
- **U16-Pinned** (follow-on) — adds Pinned `obj::#sel` form to U16, after U-LEX-HASH lands.
- **Reflective Family mirror** — deferred per Q14 ruling to a future unified reflection unit (with U8
  Message/perform/respondsTo surface).
- **Housekeeping done (tree clean post-U-ERR):** 4 test-wave bugs filed to DEFERRED
  (SUPER-STATIC/SUPER-OP-SYNTAX/NOT-KEYWORD/PRINT-TOSTRING); MANIFEST reconciled 229→**360**
  (PASS 292/NEG 40/PEND 28). Method-reopening bug already in DEFERRED (U13-filed).
- **U-LEX-HASH** ✅ ACCEPTED `fac45ae` (`#` symbol literals — name + selector forms; hand-scanner not Logos
  per ADR-0016; canonicalization reuses `encode_selector` → selector-symbol==method-identity proven; coupled
  Symbol#== fix `value.rs` value_eq; graduated `literal_map_symbol_keys.ph` + `functions_method_bind.ph`;
  6 goldens + 7 snapshots, 2 negative-lane; **floor 0**). `#[]`/`#+(_)` paren-operator forms deferred (no
  `[]` method-def grammar to canonicalize against) → DEFERRED, spec §2 mark honest. Reviewer APPROVE (10/10),
  clean-checkout verified. **NB:** implementer's 1st commit `1dd03f1` staged tests-only (source uncommitted,
  in-tree gate passed on dirty tree); orchestrator amended source in → `fac45ae`, clean-checkout re-verified.
  Lesson banked [[clean-checkout-verify-each-commit]].
- **U16-Pinned** ✅ ACCEPTED `71c703d` (pinned `::#sel(...)` arm alongside Open; `MethodRefKind::{Open,Pinned}`,
  `FamilyObject.name→selector + open:bool`, router branch validates pinned arity + dispatches exact selector
  verbatim; bare `::#name` rejected; exact-overload golden proves `#move(to,duration)` vs `#move(_,_)` dispatch
  distinct methods; **floor 0**, no ADR). Reviewer APPROVE (10/10), clean-checkout verified, self-contained commit.
  Discriminator (`(`-in-selector-string) scrutinized SOUND + filed to DEFERRED for future hardening. **Full `::`
  method-reference feature (Open + Pinned) COMPLETE.** 2 non-blocking nits → DEFERRED.
- **U-ERR-FIX** ✅ ACCEPTED `dd2e178` (4 test-wave bugs, floor 0): PRINT-TOSTRING (`value.rs::to_display_string`
  routes user objects through `toString` send) · SUPER-STATIC (`compiler/lib.rs::compile_super_send` re-anchors
  to metaclass in static ctx — write-set expansion, reviewer-ratified justified) · SUPER-OP-SYNTAX (`parser.rs::
  parse_property_name` admits operator tokens) · NOT-KEYWORD (WIRE — spec lists `not`; `parse_unary`). Reviewer
  APPROVE (4/4 + 9 cross-cut), clean-checkout green+doc-clean. Blemish: e904b57+b5ac831 red-in-iso (disclosed,
  cumulatively fixed @69c1157; tip green). 4 DEFERRED struck.
- **⚠ CONCURRENT SESSION active on main** — big iteration/string track (6 unit plans: U-IS/U-ITERABLE/
  U-NATIVE-MARKER/U-NEG/U-SEQ/U-STRING; ADR-0048 cursor-sentinel/Iterable root, ADR-0049 string-byte floor
  amendment). As of dd2e178 still DOCS/PLANNING only, no source uncommitted. Will churn spine (vm.rs/compiler/
  core.ph/floor-census/invariants) heavily soon. My bug-fix tail C/D contend those → run tail collision-aware:
  `git status` before each dispatch, pick a group with no live concurrent SOURCE, else PAUSE. NEVER two writers
  on one shared file. Explicit-path staging always; their uncommitted docs live in the tree.
- **U-FIBER-FIX** ✅ ACCEPTED `a3e23e8` (`1451f62`+`a3e23e8`, floor 0): fiber_abort root-guard · resume-gate
  message · run_until parked-frame clear (§5.1) · fiber_yield dedup vs switch_to_fiber_and_deliver (byte-for-byte
  verified) · C-FIB-5 cross-fiber DeadFrameError golden+invariant. Reviewer APPROVE (9/9), clean-checkout
  green+doc-clean, collision-clean (item #5 not deferred — invariants.rs was clean). 5 DEFERRED struck.
- **method-reopening ROOT-CAUSED** (investigation a4dc24114): `vm.rs:1346` `Bytecode::Class` unconditionally
  `create_class`→`classes.insert` overwrites — 2nd same-name user block orphans 1st's ClassId. Compile-time
  reuse guard (lib.rs:886/964) can't fire (whole unit compiles before any `Bytecode::Class` runs → `classes[A]`
  empty for both). Bootstrap safe (Rust stub pre-registers → .ph block takes Constant branch). Fix: reuse existing
  ClassId at vm.rs:1346. Floor 0. Wrinkle: field-adding reopen won't relayout → rule method-only in-scope, defer
  field-adding.
- **U-REOPEN-FIX** ✅ LANDED `e85f31a` (Group C, floor 0): Fix 1 method-reopening — `Bytecode::Class` (vm.rs)
  now checks `self.classes` at runtime and REUSES the already-registered ClassId instead of `create_class`
  shadowing, so a 2nd surface `class A {}` block APPENDS methods onto block 1's dict (ADR-0018; existing
  `Bytecode::Method` epoch-bump unchanged). Field-adding + superclass-change reopen out of scope → rejected at
  compile time with clear diagnostic (detected via persist-within-unit `field_layouts`/`class_parents`). Fix 2
  materialized-block break/continue (U-ITER-FIX item 1(a)) — `emit_deopt_block_control_trap` raises
  `Error.new(message)` w/ descriptive msg (was bare `Error.new()`), fails loud. Graduated
  `iteration/pending/{break,continue}_across_materialized_block.ph` → NEGATIVE lane. Both DEFERRED entries struck
  RESOLVED. Files: compiler/lib.rs +57, vm.rs +44, tests/lang.rs, DEFERRED.md, 2 fixtures moved.
- **Serial spine bug-fix tail (Group C landed — remaining groups, concurrent still docs-only):**
  DONE: Symbol#==·U-ERR 4-wave [PRINT-TOSTRING/SUPER-STATIC/SUPER-OP-SYNTAX/NOT-KEYWORD] (`dd2e178`)·iter
  deopt-trap+method-reopening (`e85f31a`)·U-FIBER-FIX cluster (`a3e23e8`). REMAINING: **SuperSend-IC+SOURCE_MAP**
  group → then **U-SCHED** (U-FUTURE Slice B). RECONCILE-FIRST: `has_new_construct` guard (DEFERRED L18) may
  already be fixed per [[ctor-inherit-guard-fix]] — verify before implementing.
- **U17** ✅ closed — Option-bootstrap formalization, affirm-ADR-0044. DEC-U17=A: defer
  niche-encoding (perf-only, belongs with NaN-boxing pass; None fieldless-singleton bootstrap
  already resolved, now ADR-anchored). No code. (Trivial follow-on: spec cross-ref in
  values-and-absence §3 — delicate file, skipped for now.)
- U12/U18 ✅ closed (affirm-ADR-0042/0043, `f16b58a`).

## Test-deepening wave (parallel-safe w/ U-ERR, tests-only, oracle=f54e3bf clean binary)
Corpus was 228 basic fixtures. 4 agents authoring ADVERSARIAL goldens, disjoint dirs, all
pin `.expected` byte-exact to real interpreter output, leave UNTRACKED (no commit), NO MANIFEST
(orchestrator reconciles counts + batch-commits + full-verify after):
- `acda1e03` — OO tower: inheritance/ + metaclass/ + dispatch/ (deep super chains, static-inherit,
  selector identity foo/foo(_)/foo(_,_), metaclass walk).
- `a0793d27` — collections/ + list/ (Map overwrite/remove/mutable-key-reject, Set dedup, Tuple
  arity/immutability/as-key, Range empty/descending/laziness/bound-eq — FRESH U-COLLTYPES code).
- `a014ff55` — blocks/ + iteration/ (shared-upvalue, stored-closure freshness, nested break/continue,
  2-deep non-local return, arity mismatch).
- `a18ca1c0` — absence/ + option/ + values/ (paired ifTrue two-armed, Option chain short-circuit,
  nil-vs-None, number-format/toString/nested-render edges).
Suspected bugs → reported not pinned-as-pass (Symbol#== + method-reopening already known-deferred).

### NEW bugs surfaced by test wave (file to DEFERRED after U-ERR lands + tree clean; both touch parser/compiler/vm = U-ERR territory, fix later):
- **BUG-SUPER-STATIC** (acda1e03, real correctness): class-side `super.<name>` raises DNU even
  for a legit inherited static. Instance-side `super` works; metaclass-side super lookup broken.
  Repro: `static bark => super.greet + "-dog"` on `Dog extends Animal` → `<class Dog> does not
  understand 'greet'`. Plain static override (no super) works. Future fix unit (metaclass super path).
- **BUG-SUPER-OP-SYNTAX** (acda1e03, grammar gap): `super.+(other)` unparseable — `parse_property_name`
  (phalcom-ast/src/parser.rs:1279) accepts only Identifier/Class after `.`, rejects operator tokens.
  Blocks super-calling an overridden operator. Lower priority (operators still overridable, just not
  super-callable). Future grammar unit.
- a014ff55 closures/iteration: 13 fixtures, ZERO bugs (while-vs-for freshness asymmetry confirmed by design).
- **BUG-NOT-KEYWORD** (ad07e494, dead token): `not true` keyword negation is a PARSE ERROR —
  `Token::Not` lexes but parser never consumes it as unary prefix; only `!` works. Fix: either wire
  `not` as prefix in parser, or remove the dead token. File to DEFERRED after U-ERR (parser = spine).
  (`if 5 {}` bare-form absence is BY DESIGN — conditionals via `.ifTrue`/`.ifFalse` — not a bug.)

### Round-2 test wave (arithmetic/booleans + reflection/bindings/system, oracle aa1bb3d, untracked):
- ad07e494 arithmetic+booleans: 13 fixtures (float precision, neg-modulo, inf div-zero, bignum,
  and/or raw-operand + Bool-only, short-circuit). 1 bug (BUG-NOT-KEYWORD above).
- a17250c5 reflection+bindings+system: 12 fixtures (class/superclass/metaclass walk, Q5 hash
  identity-vs-structural, perform, let-shadow scoping, block-local var freshness, print renderings).
  1 bug (BUG-PRINT-TOSTRING below).
- **BUG-PRINT-TOSTRING** (a17250c5, correctness): `System.print(obj)` diverges from `obj.toString`
  for USER classes/instances/metaclasses — native print path (`Value::to_string`) skips the
  `Object#toString` `.ph` override that collections/Some/None DO route through. Repro:
  `p.toString`→`<Point>` but `System.print(p)`→`<Point instance>`. File to DEFERRED after U-ERR
  (print path = value.rs/vm.rs). Blocks a user-object-print golden until fixed.

Round-2 total 25 fixtures (arithmetic 10, booleans 3→2 pos, reflection 6, bindings 4, system 2).
Corpus 284 → 309. Round-2 committed `bb01171`, one lane-fix `d18d290` (moved a DNU error fixture
booleans/→runtime-errors/ negative lane; suite green 27/0). **Test-lane gotcha → memory
[[phalcom-golden-test-lanes]].** 4 bugs banked for DEFERRED (file after U-ERR): BUG-SUPER-STATIC,
BUG-SUPER-OP-SYNTAX, BUG-NOT-KEYWORD, BUG-PRINT-TOSTRING.

## Test-corpus deepening COMPLETE (green). THREE waves, 91 adversarial fixtures, 228→319.
- Wave 1 (56): OO tower, collections, closures, absence — `aa1bb3d`.
- Wave 2 (25): arithmetic/booleans, reflection/bindings/system — `bb01171` + lane-fix `d18d290`.
- Wave 3 (10): concurrency (fiber+future deep) — pre-commit verified, committing.
16 dirs deepened. 4 bugs surfaced (BUG-SUPER-STATIC/SUPER-OP-SYNTAX/NOT-KEYWORD/PRINT-TOSTRING).
Sole outstanding writer: U-ERR (a0e2afec, spine). On U-ERR land: reconcile MANIFEST (+91 tests +
U-ERR's), file the 4 bugs to DEFERRED, review U-ERR diff (reviewer ON), full combined verify, fire U15.

## In flight (this session)
- **U-FUTURE-A** ✅ — Slice A settle-once `Future` `f0d128a`. Pure `.ph`, zero native, floor-0,
  green, 3 concurrency goldens (C-FUT-1/3/4/8 settled halves). Reviewer OFF, orchestrator-accepted.
  core.ph FREE. Slice B (async/await/drain) still gated on DEC-FUT-SCHED / U-SCHED.
- **item4** (→U-STD) — DISPATCHED. Migrate List `each`/`map`/`filter`/`reduce`/`includes`
  off `size`/`at` onto `iterate(_)`/`iteratorValue(_)` (DEC-ITER-A). core.ph-only. Reviewer ON.
- **U-FUTURE** — plan landed (`docs/forge/units/U-FUTURE/plan.md`). Verdict:
  **Slice A** (settle-once `Future`: `value`/`error`/`isReady`/`value` + settled
  `then`/`map`/`catch`) is **pure `.ph`, zero native — ready now**; **Slice B**
  (`async`/`await` + drain) needs a native scheduler seam (`System.schedule(_)` +
  `Fiber#isDone`, the unowned `U-SCHED`) → **DEC-FUT-SCHED** (below).

## Unblocked ready-queue (no user decision; single active code-writer in-tree)
Serialize code-writers on the shared tree (one at a time, commit-on-green — the
`git stash -u` clobber hazard forbids two concurrent in-tree implementers):

1. **U-COLL** — running.
2. **U-COLLTYPES** — native `Map`/`Set`/`Tuple`/`Range` arena arms; graduates
   U-COLL's pending tuple/map goldens. Reviewer ON. Gate DEC-CT-A **cleared**:
   ADR-0039 (`+21` floor, 80→101, Status: Proposed — ratifies with the unit like
   ADR-0028 did) is drafted with the exact raw-primitive census. Write-set
   `heap.rs`/`universe.rs`/`core.ph` + per-phase census bump. Dispatch on U-COLL land.
3. **U-ERR** — `throw`/`try`/`catch`/`on`/`ensure` + `Result`/`Ok`/`Err`
   (ADR-0008/0031/0007; ADR-0038 block-on-ensure floor already drafted). Reviewer
   ON. Contends `parser.rs` → after U-COLL. U-CORE-6 error root (dep) landed.

4. **U-ITER-FIX** — U-ITER loop-control follow-ons (`docs/forge/units/U-ITER-FIX/plan.md`):
   deopt-block break/continue silent no-op (runtime trap, U-CORE-6 unwind landed), `while`
   break/continue, loop-var capture freshness, jump-helper dedup. Reviewer ON. Edits
   `compiler/lib.rs` + `inliner.rs` → **serialize with U-ERR** (shared `compiler/lib.rs`).

5. **U-FUTURE Slice A** — settle-once `Future` + settled combinators, pure `.ph`,
   zero native (`docs/forge/units/U-FUTURE/plan.md`). Edits `core.ph` → serialize
   with U-COLLTYPES / U-STD item-4 (never two `core.ph` editors). No decision needed
   (DEC-FUT-SCHED gates only Slice B).

Soft-flag / lower-priority ready: U16 (`::` method refs, after U13), U17 (Option
bootstrap ADR — mostly docs).

**Tests-only follow-ons (parallel-safe, any-time):**
- **Item 5 (→ U-FIBER)** — author `iteration/for_generator_suspends.ph` (C-ITER-8) +
  `each_generator_raises.ph` (never created) and verify against landed U-FIBER, activate.
- **Item 4 (→ U-STD)** — migrate `List` `each`/`map`/`filter`/`reduce`/`includes` off
  `size`/`at` onto `iterate(_)`/`iteratorValue(_)` (DEC-ITER-A resolved). Edits `core.ph`.

## Design forks — RESOLVED (orchestrator autonomous authority, user "do on your own"
## 2026-07-12; conservative/reversible, revisit if user objects)
- **DEC-U12 → A** — keep flat `Number` (f64) now; defer `Integer`/`Float` split (not
  precluded). ✅ LANDED as affirm-ADR-0042 (`f16b58a`), no runtime change.
- **DEC-U13a → A** — sealed-after-definition (superclass fixed at creation, method
  reopening kept). **U13b → A** — single inheritance, defer traits/mixins/MI. Preserves
  one-probe dispatch + ADR-0011 slot/IC stability. U13 = small enforcement + ADR unit,
  conservative form disjoint from `phalcom-ast` (`class.rs`/`vm.rs`/invariants).
- **DEC-U15 → A + A** — relative file-path resolution (`import "./x"`) + whole-module
  binding (`import "x" as X`, members via sends). Greenfield: `parser.rs` + new `module.rs`.
- **DEC-U18 → A** — no default arguments now; selector identity pristine, add later if
  wanted. ✅ LANDED as affirm-ADR-0043 (`f16b58a`), no runtime change.

## Still user-only
- **ADR-0039** already ratified. No open user-only decisions remain; DEC-FUT-SCHED resolved above.
- **DEC-FUT-SCHED** — ✅ RESOLVED (orchestrator, autonomous authority 2026-07-12):
  **Slice A only** for v1 (settle-once `Future`, pure `.ph`, zero `vm.rs` risk). Slice B
  (native scheduler: FIFO + `Fiber#isDone` + root-drive) deferred to an owned `U-SCHED`.
  Reversible pre-release; revisit if user objects.
- (**ADR-0039** +21 floor — **RATIFIED by user 2026-07-12, all four arms**; U-COLLTYPES
  unblocked and resumed. Implementer Phase 0 flips Status→Accepted as landing record.)

The unblocked queue (U-COLL→U-COLLTYPES→U-ERR) runs regardless of these.

## Next
Bug-fix tail: only **SuperSend-IC + SOURCE_MAP** group left, then **U-SCHED** (U-FUTURE Slice B).
BUT ⚠ concurrent iteration/string track (U-IS/U-ITERABLE/U-SEQ/U-STRING/U-NEG/U-NATIVE-MARKER,
ADR-0048/0049) will churn spine (vm.rs/compiler/core.ph/floor-census/invariants) heavily — as of
`e85f31a` still docs/planning only, no source uncommitted. Run tail collision-aware: `git status`
before each dispatch; if their SOURCE goes live on a shared file, PAUSE (never two writers on one file).

---

## 2026-07-13 5:3xpm — M-ATTR-ROOT dispatch + root-drive-pump fix (stale entry above, new session)

U-SCHED (`34246a8`, ready-queue + `System.schedule`/`runScheduled`) already landed on `main`
since this STATE.md was last written — superseded the "Next" section above. Also landed since:
U-LSP Stage 1-3 (`ba4bf25`), decorator ADRs 0052/0053/0057/0058 ratified, `PLAN-DECORATORS.md`
written (critical path: **M-ATTR-ROOT → M-METAOBJECT → {M-INSTALL|M-LAYOUT-SLOTS|M-RUNTIME} →
decorators**).

**In flight now (2 agents, disjoint write-sets, worktree-isolated):**
- `phalcom-implementer` on **M-ATTR-ROOT** (`docs/forge/HANDOFF-M-ATTR-ROOT.md`) — write-set:
  `heap/class.rs`, `method/object.rs`, `heap/module.rs`, `primitive/attribute.rs` (new),
  `primitive/mod.rs`, `universe/primitives.rs`, `compiler/attributes.rs`,
  `compiler/lib/class_decl.rs`, `core.ph`, goldens. Gate: `annotation_unknown_error` +
  `contracts_invariant_fiber_yield` go green.
- `phalcom-implementer` on **root-drive-pump gap** — real bug, NOT a regression: despite
  `concurrency.md:239`'s "VM::run drains the ready-queue" and U-SCHED's commit message claiming
  it landed, `run_until`'s top-level-completion return (`vm/dispatch.rs:242`) never drains
  `VM::ready_queue` — no `drain_ready_queue` fn exists anywhere. `runScheduled` (`core.ph:778`)
  is manual-only. Write-set: `vm/dispatch.rs` only. Gate:
  `concurrency_sched_fifo_order.ph` → `a\nb\nc\n`.

**Next after both land:** M-METAOBJECT (`Method.fromBlock`/`Behavior.defineMethod`) — serialize
after M-ATTR-ROOT, shares `core.ph`+`universe/primitives.rs`. D-DELEGATE is NOT parallelizable
despite no M-ATTR-ROOT dependency — blocked externally on U-ANNOT-LAYOUT (not landed) and
collides on `compiler/attributes.rs` anyway.

Baseline `cargo test -p phalcom-core --test lang` pre-existing reds (independent, not blockers):
`indexing`/`indexing_negative` (U-LEX-HASH `#[]` gap), `errors::annotation_construct_own_fields`
(U-ANNOT-LAYOUT step 3/7, in-flight concurrent session).
