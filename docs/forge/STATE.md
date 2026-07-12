# Forge session state — dispatching the U12–U20 / U-COLL batch

Three-worktree consolidation (U-INH + U-ITER + U-FIBER) landed `main` at `0de7496`,
green. Now dispatching the in-flight planning batch (U12–U20, U-COLL, U-COLLTYPES,
U-ERR, U-FUTURE) per `phase-next/INDEX.md` §build-order.

## Landed (do not redispatch)
Spine U0–U11 + U-FE/U-LEX/U-LIST/U-STD. Core track U-CORE-1..6 (floor → 88).
U-INH, U-ITER, U-FIBER. Gate green at `0de7496` (`./scripts/verify.sh`).

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
- **U-ERR** — DISPATCHED (`a0e2afec`). throw/try/catch/on/ensure + Result/Ok/Err (ADR-0007/8/31/38,
  all pre-exist). Write-set compiler/lib.rs + parser.rs + block.rs + core.ph + small vm.rs +
  tests/errors. Reviewer ON. Base HEAD `5d84ad8` clean+green.
- **Serial queue behind U-ERR** (all collide on compiler/lib.rs or core.ph, cannot co-run):
  U15 (modules, brief staged) → U16 (`::` refs) → U17 (Option-bootstrap ADR, mostly docs — hold
  til U-ERR settles Result/Option relationship) → U-ITER-FIX follow-ons (strike DEFERRED L21-24 +
  descriptive deopt-trap msg, compiler/lib.rs ~L1341) → method-reopening bug (ADR-0018 violation,
  U13-filed, compiler+Bytecode::Class). Fire in order as U-ERR then each frees the spine.
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
On U-COLL completion: verify green + `git show --stat`, then dispatch U-COLLTYPES
(re-ground its §0/§4.1 anchors on HEAD first), reviewer ON. Integrate U-FUTURE
plan into the queue once the architect returns.
