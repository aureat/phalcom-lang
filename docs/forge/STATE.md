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

## In flight (this session)
- **U-COLLTYPES** — native `Map`/`Set`/`Tuple`/`Range` arena arms + `.ph` protocol;
  ADR-0039 ratified (`bdbdaaf`) + per-phase floor bump (+21); graduates U-COLL pendings.
  Write-set `heap.rs`/`universe.rs`/`value.rs`/`core.ph`/new `primitive/*`. In-tree,
  reviewer ON. Mid Phase 1 (Map+Set). Sole active writer now.
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
  precluded). U12 becomes a tiny affirm-ADR, no runtime change.
- **DEC-U13a → A** — sealed-after-definition (superclass fixed at creation, method
  reopening kept). **U13b → A** — single inheritance, defer traits/mixins/MI. Preserves
  one-probe dispatch + ADR-0011 slot/IC stability. U13 = small enforcement + ADR unit,
  conservative form disjoint from `phalcom-ast` (`class.rs`/`vm.rs`/invariants).
- **DEC-U15 → A + A** — relative file-path resolution (`import "./x"`) + whole-module
  binding (`import "x" as X`, members via sends). Greenfield: `parser.rs` + new `module.rs`.
- **DEC-U18 → A** — no default arguments now; selector identity pristine, add later if
  wanted. U18 = tiny affirm-ADR.

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
