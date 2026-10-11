# Phalcom Branch and Worktree Status

**Snapshot date:** 2026-10-11
**Repository:** `/Users/altunhasanli/dev/phalcom/phalcom`
**Purpose:** Map local branches and worktrees to their checkpoints, plans, and current blockers. This is a status inventory, not a test or release certification.

## At a glance

- There are **four named local branches**: `main`, `codex/lang007-c4-p10`, `feat/telos-collection-upgrade`, and `codex/coll006-p4-followup-unaccepted`.
- `main` and `codex/lang007-c4-p10` point to the **same commit** (`6e5287af`), but their separate worktrees contain different uncommitted work.
- The collection branch is a separate, substantial work stream: it is **15 commits ahead and 25 behind `main`**, before counting its uncommitted P6 work.
- The local records do not identify GitHub issue numbers. The issue names below are the repository's checkpoint/plan tasks and recorded blockers.
- No merge or consolidation has been performed as part of this inventory.

## Active work streams

| Branch and checkout | Checkpoint and plan | What it is doing | Current state / blocker |
|---|---|---|---|
| `main` — repository root | `LANG005.C6` → `LANG005.C6.P7`; also the primary checkout for `LANG007.C4.P4` | P7 addresses source/native generic signature publication and generic `Iterable` inference/evidence used by numeric APIs. C4/P4 addresses `FieldKey` and unordered Record semantics. | `LANG005.C6.P7` is `IN_PROGRESS / PARTIAL / UNVERIFIED`; the P7 record identifies loss of the authored `Option.match` generic signature at native signature publication as a confirmed issue. C4/P4 remains the checkpoint's primary plan. Its immediate recorded action is to rebase the corrective plan and complete T0's live-owner matrix, then converge static-constant formation and the bounded exact-owner/replay seam; T7.0E is a later acceptance gate. The checkout has six modified Rust files and an untracked P10 completion plan. The Rust edits mix trace instrumentation and hierarchy/projection investigation; they are not cleanly labeled to one plan in the worktree. |
| `codex/lang007-c4-p10` — `/Users/altunhasanli/.codex/worktrees/lang007-c4-p10-5099/phalcom` | `LANG007.C4` → parallel plan `LANG007.C4.P10` | Adds explicit semantic `While` through AST/parser, checking, flow, static evaluation, and runtime lowering. | Uncommitted implementation; no commit or push. Focused work found two open acceptance failures: comparison-loop flow loses an established `Int` fact after `continue`, and the source runtime loop fixture did not terminate. The P10 handoff says to keep those gates blocked and not consolidate the worktree. The new completion plan is currently an untracked file in the `main` checkout and says execution must be on `main`, so plan location and implementation location do not currently agree. |
| `feat/telos-collection-upgrade` — `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/telos-collections-upgrade` | `COLL006.C1` → active `COLL006.C1.P6` | Implements the ordered mutable hash family: `MutableBag`, `IndexedDict`, and `OrderedSet`, reusing the Map/Set key engine. | Branch tip `ffa2ab96` matches its remote, but the worktree has extensive uncommitted P6 source and plan changes. P6 is `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. The latest checkpoint says the IndexedDict semantic correction passed five focused regressions, while the combined runtime gate still fails `orderedCollision` and `mutableKeyPolicy`; P6 G4 remains open. P7 (`Bag`/`Bijection`) is proposed and gated on P6 closure. The overall C1 checkpoint remains `BASELINE_BLOCKED` by the separate P5 associated-family invocation residual. |
| `codex/coll006-p4-followup-unaccepted` — no worktree | Local parked branch; tip `82725c90` | Preserves an explicitly unaccepted pre-P5 correctness follow-up. | Not checked out and has no current handoff/worktree. Its tip is 12 commits not reachable from `main`; treat it as parked historical WIP, not as the active collection P6 checkout. |

### Important separation inside `LANG007.C4`

The C4 checkpoint is `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. Its metadata still names **P4 as the primary active plan** and P3 as the latest completed plan. P10 is a concurrent plan and does not replace the checkpoint's active-plan field. The P10 implementation has its own partial walkthrough and handoff in the `codex/lang007-c4-p10` worktree; it is not complete or certified.

The untracked P10 completion plan in the `main` checkout continues `LANG007.C4.P10` and lists remaining R0–R7 work. It should not be confused with the P4 completion plan or with `COLL006.C1.P6` (a different program's P6).

## Collection-branch consolidation status

The separate **“Check collection branch merge”** task is investigating a read-only merge preview. Its current findings are:

- `main` has 25 commits beyond the common base; `feat/telos-collection-upgrade` has 15.
- The preview reports six committed-file conflicts: `docs/decisions/README.md`, `phalcom-core/src/bytecode.rs`, `phalcom-core/src/universe/primitives.rs`, `phalcom-native-surface/src/generated.rs`, `phalcom-semantic/src/checker/expression.rs`, and `phalcom-semantic/src/checker/mod.rs`.
- There is a decision-number collision: `main` uses TDR-0091 for primitive declaration classification, while the collection branch uses TDR-0091 for contextual homogeneous sequence literals. The collection branch also has TDR-0092 for its approved P6 native floor.
- Both sides have local edits that must be preserved. The P6 implementation and its new decision/spec files are not part of the committed branch tip yet.

This means the feature branch has useful committed P5 history plus valuable **uncommitted P6 work**, but it is not a clean merge candidate today. The conflict list also does not cover semantic integration risks.

## Detached comparison worktrees

These are checkouts without a named branch; they should not be mistaken for additional feature branches.

| Checkout | State | Interpretation |
|---|---|---|
| `/private/tmp/phalcom-coll006-base-78cfd` at `78cfd920` | Clean, detached | COLL006 comparison/base snapshot. |
| `/private/tmp/phalcom-main-c2dd635` at `c2dd635e` | Detached; one added trailing newline in `phalcom-semantic/src/checker/typed_expr.rs` | Comparison snapshot, not substantive implementation work. |
| `/private/tmp/phalcom-origin-main-compare-20261011` at `c2dd635e` | Clean, detached | `main` comparison snapshot. |
| `.worktrees/complex-math` at `e8fd0729` | Detached; three untracked older P10 planning/incident documents under `LANG007.C4/C4P10/` | Math snapshot with leftover P10 documents, not a named `complex-math` branch or current P10 implementation. |

## Suggested ownership map for resuming

1. Keep `codex/lang007-c4-p10` as the location of the current explicit-`While` implementation until its two failed gates are resolved and its plan/location mismatch is reconciled.
2. Continue collection work only under `feat/telos-collection-upgrade` and `COLL006.C1.P6`; do not start P7 while P6 G4 is open.
3. Before staging from the repository-root `main` checkout, label each dirty path to its owning plan. The current edits include generic call/inference tracing and source-hierarchy investigation, while the collection branch contains related semantic work in overlapping areas.
4. Finish the separate merge/conflict analysis before choosing a consolidation order. Preserve both worktrees' uncommitted changes and resolve the TDR numbering collision explicitly.

## Records consulted

- `docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md` and `LANG005.C6.P7-canonical-iterable-inference-and-evidence-transport.md`
- `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md`, the P4 completion plan, and the P10 partial walkthrough/handoff in the P10 worktree
- `docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md`, the P6 plan and incident records in the collection worktree
- Local branch refs, worktree status, and current task notes as of the snapshot date
