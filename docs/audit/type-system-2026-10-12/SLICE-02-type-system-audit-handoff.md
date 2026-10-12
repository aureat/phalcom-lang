# Telos Type-System Audit — Slice 02 Handoff

**Date:** 2026-10-12  
**Repository:** `aureat/phalcom-lang` (`main`)  
**Original audit reference:** `5942666901b387519c5cd502bed2611cf97b53c4`  
**Live GitHub-indexed source revision inspected:** `608c8c6cd3346556f3bbc98f0713a6df6e0a72ac`  
**Read-only:** Yes. Only downloadable audit Markdown files were produced in the assistant working environment. No repository source, tests, snapshots, GitHub commits, or working tree were modified.

## 1. Scope completed during this turn

1. Re-read the attached audit state summary, `INSTRUCTIONS.md`, `BASELINE.md`, and incidents `INC-001` through `INC-003`.
2. Queried the connected GitHub repository for current `types/relation.rs`, `environment.rs`, `instantiation.rs`, `substitution.rs`, `store.rs`, `row.rs`, and relevant normative specs.
3. Compared original baseline with indexed revision `608c8c6`; `compare_commits` reports **ahead by two commits**, and `608c8c6` identifies the completed LANG007.C4.P7.T7 residual-specialization implementation. This is a remote checked-in state, not proof of the local uncommitted files that the prior agent described.
4. Verified the source-level code paths underlying `INC-002` (labeled Tuple order) and `INC-003` (blocked-first union evaluation) are still present. No runnable Rust test was executed in this turn.
5. Recorded two **new, separate** source-level incidents immediately during investigation:
   - **INC-004** — Tuple specialization silently erases anonymous remainder lanes and can normalize open empty-fixed Tuple to `Unit`.
   - **INC-005** — Record specialization ignores captured row binding in `TypeView::materialize` and fails to specialize anonymous tail payloads in both materialization routes.
6. Inspected adjacent semantic structures and control paths, without converting unverified risks into additional defects.

## 2. Incident register (cumulative)

| ID | Concern | Severity | Current evidence | Follow-up |
|---|---|---|---|---|
| INC-001 | Workspace baseline parser and snapshot failures | P2 baseline blocker | Prior run: 300 pass, 7 fail, 1 ignored at the failing AST/parser test binary | No new workspace command executed; do not infer present suite status |
| INC-002 | Labeled Tuple/Pack subtype matching ignores order | P1; soundness candidate | Source/spec contradiction still visible in `relation.rs:433–548` and `relation.rs:555–687` | Standalone canonical-type fixture, then checked compiler consumer |
| INC-003 | Target union returns `Blocked` before later `Proven` alternative | P1; completeness | Source control-flow still visible in `relation.rs:273–282`; projection blocked case visible `relation.rs:405–426` | Explicit allocation-order experiment, proper projection, budget/cancel variants |
| INC-004 | Open Tuple remainder erasure by both materializers | P1; incorrect canonical transformation | `environment.rs:213–223`, `instantiation.rs:172–184`, `store.rs:1057–1097` and normative §3.2 | API probe plus typechecking/realization consumer |
| INC-005 | Record-row tail binding/payload incomplete | P1; incomplete specialization | `environment.rs:224–240`, `instantiation.rs:347–405`, `substitution.rs:207–245`, normative §3.4 | API differential fixture and source-reachable generic Record |

**Important evidence standard:** INC-002/003/004/005 are source-level findings, not independently test-reproduced failures. The current workspace is not mounted in this assistant's container, so the pinned Rust toolchain, tests, and local worktree status were not re-run. Report behavioral confidence and source confidence separately; do not claim P0 exploitation or release gate success.

## 3. Corrected baseline/toolchain notation

The attached `BASELINE.md` says `Rust toolchain: nightly-2026-07-10 (local rustc 1.99.0-nightly (1a98b1e13 2026-08-07))`, which misleadingly identifies the command's compiler as the pinned one. The earlier agent's attached audit summary explicitly establishes:

- The **repository file** `rust-toolchain.toml` pins `nightly-2026-07-10`.
- A **directory override** selected floating `nightly-aarch64-apple-darwin` during the original baseline commands.
- The toolchain **actually used by the recorded baseline** was `rustc 1.99.0-nightly (1a98b1e13 2026-08-07)` / Cargo `1.99.0-nightly (c79e8f894 2026-08-04)`.
- An explicitly **pinned** toolchain reports `rustc 1.99.0-nightly (af3d95584 2026-07-09)`.
- The preceding audit was on **Darwin arm64**; `.cargo/config.toml` configured `-Zunstable-options`, `-Zthreads=4`, `-Ctarget-cpu=native`, jobs 8, `RUST_MIN_STACK=33554432`.

The original `BASELINE.md` attached by the user has **not** been modified. This corrected statement is an addendum, not a new test result.

## 4. Structural materialization correspondence matrix

| Canonical form | `TypeView::materialize` / `materialize_view` | `materialize_type` / checked instantiation | `TypeSubstitution::apply` | Audit disposition |
|---|---|---|---|---|
| Tuple fixed elements | Recurses | Recurses | Recurses | Fixed elements appear supported |
| Tuple positional + labeled remainder | **Both discarded**, through `store.tuple` | **Both discarded**, through `store.tuple` | Preserved and typed contents specialized | **INC-004** |
| Pack fixed elements + two remainder lanes | Preserved via `store.open_pack` | Preserved via `store.open_pack` | Needs end-to-end control test | Positive control path |
| Record fixed fields | Recurses | Recurses | Recurses | Fixed fields appear supported |
| Record captured row parameter | **Does not consume `env.row_bindings`** | Resolves bound rows or rejects unsolved tail (mode-dependent) | Type-only substitution retains parameter | **INC-005** |
| Record anonymous tail key/value constraints | **Unchanged** | **Unchanged** | Explicitly recurses | **INC-005** |
| `SelfType` | `env.self_binding` applied | Unchanged | Unchanged | **Investigate independently**; no checked-materializer consumers found in indexed search |
| Dynamic escape | `TypeView::materialize_knowledge` handles direct root Parameter; `Applied` static argument has dynamic branch | Checked generic instantiation exports only types/rows | Type substitution has no dynamic semantic contract | **Investigate** nested Dynamic treatment; not yet an incident |
| Record-row-bearing nested static arguments | `TypeView` appears to forward raw row ID in `Applied`, `StaticData`, type-member projections | Checked instantiation traverses `RecordRow` static arguments | Type substitution traverses Applied row arguments | **Investigate** as INC-005 extension |

## 5. Adjacent investigation backlog — next bounded slice

**Priority 1 — standalone semantic API reproduction.** In a real checkout, first check `git status --short`, branch and HEAD. Use a standalone audit-folder Cargo/Rust test harness against already-exported semantic APIs; do not edit `phalcom-semantic` source/tests. Run explicitly with `cargo +nightly-2026-07-10` (or verify effective pinned rustc) to eliminate directory override. Record full command logs and normalize TypeId allocation order. Test INC-002/003/004/005 and update confidence/status with exact observed outcomes. Never infer compilation or runtime results from source inspection.

**Priority 2 — type projection of open Tuple/Pack.** `store.rs:1233–1277` forms `project_positionals` and `project_labeled` by projecting only the fixed elements of `TypeData::Tuple`/`Pack`. Verify whether normative `Positionals`/`Labeled` projections must retain anonymous lane remainders and whether kinded pack terms can represent them. Source currently drops tails in these projection methods; investigate consumer expectations before registering INC-006.

**Priority 3 — `Self` and dynamic materialization contract.** `TypeView::materialize_checked` calls `to_generic_instantiation`, which exports only type and row bindings. `SelfType` is unchanged by `instantiation.rs`; direct nested dynamic variables in `TypeView::materialize` may be handled inconsistently. Determine whether checked materialization is an independently supported API and trace all actual clients. Register a new incident only with a supported violated contract and meaningful reachability.

**Priority 4 — incremental fallback.** In `session.rs:927–960`, private `update_with_delta` catches any failed update and returns the last known-good snapshot (or creates one with default products), without a visible `failed/stale` marker on the return. `update_with_budget_and_cancel` at `session.rs:963–970` propagates `Result`, and core compiler reportedly uses the Result path. Trace every caller of the convenience path and every consumer of its `SemanticWorkspaceUpdate` before deciding whether this is an actual formal-semantic publication issue or a benign presentation-only recovery.

**Priority 5 — relation law and budget audit.** Check target union aggregate `Blocked`, `Cancelled`, `BudgetExceeded`, source union conjunction, invariant applied argument matching, `visited` recursion and pair budget accounting; construct permutation/metamorphic regressions. Existing tuple-lane and union candidate tests may not include ordered-label permutation and blocked-first alternatives.

## 6. Further operational requirements

- Maintain no implementation authorization. Do not fix code, accept snapshots, mutate GitHub, or overwrite unrelated changes.
- Normative source is `docs/specs/`; proposed specs and implementation histories are not substitutes for accepted semantics.
- Keep statuses separate: **confirmed by source path**, **behaviorally reproduced**, **verified end-to-end**, **candidate**, **cleared**.
- Avoid double-counting independent symptoms when a shared canonical traversal causes them; link incidents explicitly, preserving separate type invariants and correction owners.
- Runtime/semantic and cold/incremental equivalent tests are needed before claiming full validation.
- New downloadable incidents should use distinct files `INC-006-...md` and above; maintain next-slice handoff independently.

## 7. Audit turn disposition

**Completion:** Bounded slice complete: two new source-confirmed incidents plus verification of still-present INC-002/003 code and a corrected baseline toolchain addendum.  
**Not completed:** Cargo probes, executable regressions, source reachability, user-facing miscompilation confirmation, full panic census, compiler/VM lowering trace, and cold/incremental differential execution.  
**Next prompt action:** Continue a bounded slice starting with API reproductions if repository/test execution becomes available; otherwise investigate `store.project_positionals`/`project_labeled` and `Self`/dynamic specialization based on live source, issuing only substantiated new incidents.
