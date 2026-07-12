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

## In flight (this session) — TWO disjoint src writers in parallel
- **U-COLLTYPES** — native `Map`/`Set`/`Tuple`/`Range` arena arms + `.ph` protocol;
  ADR-0039 flip + per-phase floor bump (+21); graduates U-COLL pendings. Write-set
  `heap.rs`/`universe.rs`/`value.rs`/`core.ph`/new `primitive/*`. In-tree, reviewer ON.
- **U-ITER-FIX** — loop-control follow-ons (deopt trap · `while` break/continue ·
  loop-var freshness · jump dedup). Write-set `compiler/lib.rs`/`inliner.rs` + iteration
  tests. **Disjoint from U-COLLTYPES.** In-tree, reviewer ON.
- Both: no-stash / explicit-path staging / commit-on-green. Shared file = MANIFEST only.
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

## BLOCKED-ON-DECISION (user's call — forge does not pick; surfaced, non-blocking)
- **DEC-U12** — flat `Number` vs surface `Integer`/`Float` split.
- **DEC-U13a** — hierarchy: sealed-after-definition (Wren) vs mutable `superclass=`.
  **DEC-U13b** — single-inheritance only vs traits/mixins/MI.
- **DEC-U15** — module resolution + binding model.
- **DEC-U18** — support default arguments at all + expansion policy.
- **DEC-FUT-SCHED** — U-FUTURE Slice B: fold the native scheduler seam (FIFO +
  `Fiber#isDone` + root-drive) into U-FUTURE, or ship it first as an owned `U-SCHED`?
  Rec: Slice A only for v1 (pure `.ph`, zero `vm.rs` risk); defer Slice B. Slice A
  does not depend on this.
- (**ADR-0039** +21 floor — **RATIFIED by user 2026-07-12, all four arms**; U-COLLTYPES
  unblocked and resumed. Implementer Phase 0 flips Status→Accepted as landing record.)

The unblocked queue (U-COLL→U-COLLTYPES→U-ERR) runs regardless of these.

## Next
On U-COLL completion: verify green + `git show --stat`, then dispatch U-COLLTYPES
(re-ground its §0/§4.1 anchors on HEAD first), reviewer ON. Integrate U-FUTURE
plan into the queue once the architect returns.
