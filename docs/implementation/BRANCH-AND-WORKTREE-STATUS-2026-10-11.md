# Phalcom Branch and Worktree Status — Pre-Consolidation Inventory

**Snapshot date:** 2026-10-11
**Repository:** `/Users/altunhasanli/dev/phalcom/phalcom`
**Purpose:** Preserve the pre-consolidation branch/worktree assessment and record the consolidation outcome below. This is not a test or release certification.

> The inventory sections below describe the state before consolidation. See
> **Consolidation outcome** at the end for the current repository state.

## At a glance

- There are **four named local branches**: `main`, `codex/lang007-c4-p10`, `feat/telos-collection-upgrade`, and `codex/coll006-p4-followup-unaccepted`.
- `main` and `codex/lang007-c4-p10` point to the **same commit** (`6e5287af`), but their separate worktrees contain different uncommitted work.
- The collection branch is a separate, substantial work stream: it is **15 commits ahead and 25 behind `main`**, before counting its uncommitted P6 work.
- The local records do not identify GitHub issue numbers. The issue names below are the repository's checkpoint/plan tasks and recorded blockers.
- No merge or consolidation had been performed when this inventory was written.

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

## Consolidation outcome

Consolidation was performed on `main` on 2026-10-11. At the end of this record,
the root checkout is clean and `main` is 20 commits ahead of `origin/main`.

| Commit | Consolidated work |
|---|---|
| `2247e337` | Merged `feat/telos-collection-upgrade` into `main`; manually resolved the six reported conflicts. |
| `5bed257e` | Applied the saved collection worktree delta as a separate P6/P7 source change. The accepted P6/P7 decision files and plans are included. |
| `9afab8f4` | Applied the saved LANG007 worktree delta as a separate change, including the P10 implementation and its relocated planning records. |
| `4838bc75` | Preserved the latest stash's proposed class-initialization plan as a separate documentation change. The stash remains in the stash list. |

Conflict dispositions:

- Kept both sides' semantic-checker changes in `analyze_expression_with_publication` and exported both the algebraic-realization and sequence-publication products.
- Preserved `LoadTypeParameterClass` at its existing main-line opcode index 117 and assigned the newly merged collection opcodes indices 118–127. Updated the VM bytecode index table.
- Regenerated `phalcom-native-surface/src/generated.rs` from merged primitive declarations. The generator reported 481 declarations after P6/P7 source integration.
- Resolved the decision-number collision by retaining collection TDR-0091, TDR-0092, and TDR-0093, and renumbering the main primitive-classification decision to TDR-0094 with its registry path updated.
- Adapted the existing runtime-typing identity test's `ConstInt` values to the merged `StaticInt` representation.

Disposition of separately preserved work:

- `codex/coll006-p4-followup-unaccepted` remains at `82725c90`, unchanged and unmerged. It remains a separately reviewable, explicitly unaccepted change.
- The LANG007, collection, and math worktrees were left intact. Their pre-consolidation branch tips and full dirty worktree contents are recoverable under `refs/codex/consolidation-snapshots/20261011/`.
- The three untracked math-worktree documents are exact copies of the P10 incident/census/plan files now under `LANG007.C4.P10/`; they were not added again at the alternate `C4P10/` path.
- All 13 original stash entries remain present and each stash object has a snapshot ref. The latest class-initialization plan was copied into `main`; older stashes were preserved but not applied.
- Detached comparison worktrees remain unchanged.

At the time of the initial consolidation, no tests were executed. The native-surface artifact was regenerated and its consistency check passed at 481 declarations. Library-only compile checks passed for `phalcom-core`, `phalcom-semantic`, and `phalcom-lsp`. A core test-target compile check remains blocked by a call at `phalcom-core/src/vm/send.rs:1960` that omits the sixth argument added on the existing main line; the call is unchanged from the pre-consolidation main revision and was left as a baseline issue. The merge-caused `StaticInt` test mismatch was adapted and no longer appears in that check. Original plan records retain their prior focused-test evidence. COLL006 P7 is active: the saved source includes Bag implementation files, but its T5 gate has not been reconciled or newly verified; Bijection remains outstanding.

## Follow-up review of the unaccepted COLL006 commit

The unique commit `82725c90` was replayed on consolidated `main` (`170792cd`)
in `.worktrees/coll006-p4-followup-review`, branch
`codex/coll006-p4-followup-review`. The original
`codex/coll006-p4-followup-unaccepted` branch and its source worktree were not
modified. The replay applied without merge conflicts and was split into these
local, unpushed commits:

| Commit | Review disposition |
|---|---|
| `06afc16c` | Computed-Record-key canonical signature-owner correction, amended LANG007 C4 P4 plan/checkpoint, and focused regressions. P4 remains partial and unverified overall. |
| `f98ccaae` | Preserve unknown qualified module calls as unresolved rather than diagnosing them as known non-callables. |
| `9fede15a` | Add inference diagnostic-range and Record-row identity regressions; align the metadata fixture with its Symbol-only remainder domain. |
| `fd5f7835` | Preserve compiler scratch bookkeeping across the loop-pattern failure branch; update iteration fixtures. |
| `7cf72a78` | Migrate core Bytes/Path/reflection fixtures to explicit typed collections and current package exposure semantics. |
| `51d09753` | Format only the changed Rust ranges after the separate commits were reviewed. |

The `foundations::static_constants` filter changed from 21 passed / 3 failed
on the baseline to 25/25 passed after the signature-owner correction. The
individual focused semantic regressions and `cargo check -p phalcom-core --lib`
also passed. Core runtime tests were not run because the existing core
test-target compile blocker at `phalcom-core/src/vm/send.rs:1960` remains.
The follow-up branch remains local and has not been merged into `main` or
pushed; the original unaccepted branch remains available for comparison.

Targeted rustfmt checking passed for 47 changed Rust ranges across 20 files.
The earlier workspace-wide `cargo fmt --all -- --check` reported formatting
differences across many files outside this follow-up; no repository-wide
formatting sweep was applied.
