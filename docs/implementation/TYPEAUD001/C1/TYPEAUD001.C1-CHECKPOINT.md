---
id: TYPEAUD001.C1
category: TYPEAUD
program: TYPEAUD001
checkpoint: TYPEAUD001.C1
kind: checkpoint-record
title: "Type-System Audit Remediation and Cross-Layer Certification"
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
repository: aureat/phalcom-lang
branch: main
prepared: 2026-10-12
prepared_against_revision: b00d75b3ac9f50d37cf9abfddfde00917e099031
active_plan: TYPEAUD001.C1.P1
latest_completed_plan: null
next_plan: TYPEAUD001.C1.P1
requires:
  - Type-system audit Slice 2–6 source/spec findings
  - Repository workflow and normative specification authority
---

# TYPEAUD001.C1 — Canonical Checkpoint Record
## Type-System Correctness and Hardening

> **Single source of lifecycle truth for C1.** `TYPEAUD001.C1.P1` is `IN_PROGRESS / IMPLEMENTED / BASELINE_BLOCKED`: transactional authority/retry implementation and strict consumer migrations are present; focused transaction, retry, interner-root, compiler-error mapping, LSP scan, and workspace-indexing checks have run. G6/G7 integration coverage and G8 baseline-sensitive verification still have open items. INC-007/011 remain unclosed until their mandatory gates are complete. The live canonical location is `docs/implementation/TYPEAUD001/C1/TYPEAUD001.C1-CHECKPOINT.md`.

## 0. Checkpoint role and update discipline

This checkpoint is the *sole* durable C1 implementation-state ledger. It owns the plan registry, scope and invariants, stable takeover contracts, verification record, deferred/baseline issues, consultations/amendments, and the current next action. The program `STATUS.md` may summarize this ledger but may not redefine it; plan documents describe *intent*, not completed implementation.

Lifecycle transitions must obey `docs/workflow/implementation-record-lifecycle-convention.md`:

- `PROPOSED / NOT_STARTED / UNVERIFIED` means only that planning exists.
- At genuine implementation start: set C1/P1 `IN_PROGRESS`; record base revision and verified worktree state.
- `IMPLEMENTED` means intended source changes exist; `FOCUSED_TESTED` requires actual executed test evidence.
- `COMPLETE` for a plan is not equivalent to completion of C1; `RELEASE_COMPLETE` requires actual broad certification.
- Update the ledger at coherent task/gate/consultation milestones, not every command.
- Do not reconstruct test passes, commit SHAs, or a clean worktree from the planning record.

## 1. Bounded acceptance objective and ownership boundary

C1 closes the source-confirmed findings from the 2026-10-12 type-system audit using one coherent canonical semantic architecture and separately gated corrections. Work owned by this checkpoint:

1. Semantic workspace publication authority, candidate-state isolation, and failure/retry integrity.
2. Correct relation algebra for unions, generic parameter proof laws, and ordered Tuple subtyping.
3. Correct handling of Dynamic generic slots through relations and directly dependent consumers.
4. Faithful generic specialization of open Tuple/Record structural information.
5. Normative open-shape intrinsic projection behavior (`Map`, `Element`, `At`).
6. Regression tests and executable cross-layer equivalence/certification evidence.
7. An incident closure matrix showing concrete code/test evidence for each finding.

C1 does **not** own a general rewrite of the language parser, type-system feature expansion, VM redesign, all pre-existing repository failures, or implementation of unrelated LANG007/COLL/SEMA tasks. Fixes must occur in their existing canonical layer, not through parallel TYPEAUD-specific runtime/semantic logic.

### Completion contract

C1 may be completed only when every numbered P1–P5 plan is `COMPLETE` or explicitly superseded by an approved architectural amendment, and consolidated G6 has been executed and documented. A baseline exclusion may remain only when classified accurately with concrete acceptance reasoning; it cannot silently negate required type-system proof evidence.

## 2. Plan ledger — authoritative

| ID | Scope | Status | Completion | Verification | Current disposition |
|---|---|---|---|---|---|
| G0 | Baseline, toolchain pin, affected API census and audit triage | IN_PROGRESS | PARTIAL | UNVERIFIED | P1 baseline and T1 evidence recorded; remaining checkpoint-wide source/test census is not claimed complete |
| `TYPEAUD001.C1.P1` | Transactional semantic publication and retry, INC-007/011 | IN_PROGRESS | IMPLEMENTED | BASELINE_BLOCKED | Transactional authority/retry implementation and strict consumer migrations are present; G1–G5 have focused coverage, G6/G7 remain partial, G8 has reproduced baseline failures; incident closure and G9 are pending |
| `TYPEAUD001.C1.P2` | Canonical relation algebra/ordered Tuple relation, INC-002/003/008/010 | PROPOSED | NOT_STARTED | UNVERIFIED | Patch-grade plan authored; remains proposed while P1 is active |
| `TYPEAUD001.C1.P3` | Dynamic static-generic argument semantics, INC-009 | PROPOSED | NOT_STARTED | UNVERIFIED | Scope defined; no patch-grade plan authored |
| `TYPEAUD001.C1.P4` | Lossless canonical materialization, INC-004/005 | PROPOSED | NOT_STARTED | UNVERIFIED | Scope defined; no patch-grade plan authored |
| `TYPEAUD001.C1.P5` | Open structural intrinsic normalization, INC-012/013 | PROPOSED | NOT_STARTED | UNVERIFIED | Scope defined; no patch-grade plan authored |
| G6 | Integrated cross-plan certification and incident closure | NOT_RUN | — | UNVERIFIED | Must follow accepted P1–P5 changes |

**Prepared plan records:** `TYPEAUD001.C1.P1-semantic-publication-luna-patch-grade-implementation-plan.md` (prepared against `b00d75b3`, amended as A1 on 2026-10-12) and `TYPEAUD001.C1.P2-relation-algebra-ordered-products-luna-patch-grade-implementation-plan.md` (prepared against `e2ae4a0f` on 2026-10-13). P1 remains the active plan. Plan-local tasks and gates are distinct from checkpoint-wide G0/G6; do not mark any of them passed from plan existence.

### Dependency/implementation sequence

```text
C1 G0
  +---- P1 (publication authority) --------------------------+
  +---- P2 (relations) ----- P3 (Dynamic consumers) ----------+
  +---- P4 (materialization) - P5 (structural intrinsics) ----+
                                                           |
                                               C1 G6 certification
```

P1, P2, and P4 can be developed in parallel only with explicit file ownership, integration discipline, and a consistent checked-out base. Avoid concurrently editing overlapping semantic-layer internals without coordination.

## 3. Audit issue ledger and ownership

| Incident | Confirmed observation | Plan / gate | Status at baseline | Closure proof required |
|---|---|---|---|---|
| INC-001 | AST/parser baseline: 300 pass, 7 fail, 1 ignored in first failing binary; stale snapshots and one record-pattern mismatch | G0 / G6 | BASELINE ISSUE, not an established semantic defect | Exact current toolchain/test rerun; separate fixture vs code failures; classified dispositions |
| INC-002 | Tuple labeled relation behaves as unordered label matching | P2 | OPEN | Ordered-lane counterexamples and valid remainder semantics |
| INC-003 | Target-union existential check can return `Blocked` before later `Proven` | P2 | OPEN | Quantified relation reducer, allocation-order independent tests |
| INC-004 | Tuple materializers discard independent positional/labeled remainders | P4 | OPEN | Lazy and checked specialization preserve typed/unconstrained open lanes |
| INC-005 | Record row tails and anonymous typed key/value constraints inadequately materialized | P4 | OPEN | Captured/anonymous/projection tail tests and substitution composition |
| INC-006 | Suspected open finite product projection erasure | None | **CLEARED** | Do not reopen without specific consumer defect and normative proof |
| INC-007 | Infallible semantic fallback masks infrastructure failure with stale/synthetic `Complete` snapshot | P1 | OPEN | Fallible authority API, fresh-failure injection, compiler/LSP routes |
| INC-008 | Source-union universal check can return `Blocked` before later definitive `Refuted` | P2 | OPEN | Universal aggregation and refutation-order independence |
| INC-009 | Applied generic Dynamic slots hard-refuted rather than producing boundary obligations | P3 | OPEN | Dynamic boundary, variance, and downstream consumer verification |
| INC-010 | Knowledge checker blocks universal subtype truths involving parameters | P2 | OPEN | Generic lawful proofs without unconstrained relation promotion |
| INC-011 | Infallible module-update consumes pending semantic deltas on failure; candidate state may contaminate later revisions | P1 | OPEN | Fault injection, delta retention, cold/incremental retry equivalence |
| INC-012 | Structural `Map<F>` loses open Tuple tails or preserves untransformed Record tails | P5 | OPEN | Shape-preserving mapping and symbolic tails per TDR-0087 |
| INC-013 | Open Tuple `Element`/`At<I>` collapse/reject based on fixed prefix | P5 | OPEN | Typed/open remainders, valid symbolic indexing per TDR-0086 |

**Audit evidence status:** The semantic findings above are supported by inspected source/spec contradictions. Their individual executable reproductions and post-fix tests have **not** yet been run as part of TYPEAUD001. INC-009 overlaps a previously documented audit finding; retain provenance rather than claiming it was newly discovered.

## 4. Established takeover facts versus target invariants

### 4.1 Existing source facts confirmed at planning revision

- The module/source layer may accept edits before semantic publication; current strict module-update path deliberately preserves newest accepted source state on semantic cancellation.
- `SemanticWorkspaceSession::update_with_budget_and_cancel` and `update_module_workspace_with_cancel` already return explicit `Result` outcomes.
- `session.update()` / `update_with_delta()` can catch failures and synthesize a success-shaped update; `workspace::analyze_workspace` currently calls that path.
- Strict persistent compiler and interactive LSP paths exist and must remain strict.
- SemanticDb mutates query revisions, Ready products, dependency edges, retirement, and validation state during candidate work; maintaining only an immutable `last_snapshot` is not sufficient evidence of rollback.
- TypeStore identity and normal incremental reuse are existing tests/architecture to preserve.
- `SnapshotStatus::Partial` may be an intentionally successful publication under an existing consumer-specific contract; do not equate it to infrastructure failure.

These are **takeover observations**, not accepted correctness claims about the unfixed implementation.

### 4.2 Target invariants — NOT YET VERIFIED

| ID | Required C1 invariant | Implementer owner |
|---|---|---|
| INV-C1-01 | Failed candidate updates never create successful current semantic authority | P1 |
| INV-C1-02 | Pending semantic changes survive failure, retries remain cold-equivalent | P1 |
| INV-C1-03 | Proof aggregation follows quantified union laws; epistemic and operational outcomes remain distinct | P2 |
| INV-C1-04 | Tuple positional and labeled components retain ordered lane identity | P2/P4 |
| INV-C1-05 | Dynamic generic slots are boundary obligations, not fake static proofs | P3 |
| INV-C1-06 | Specialization preserves open Tuple/Record shapes, symbolic identity and domain | P4 |
| INV-C1-07 | Structural type-member normalization never invents closed/reduced results for open shapes | P5 |
| INV-C1-08 | All compiler/LSP/metadata consumers use canonical semantic results; no parallel authority | P1–P5 |
| INV-C1-09 | Canonical TypeId, interner identity, source generation and incremental reuse remain coherent | P1/P4/G6 |
| INV-C1-10 | Required laws have executed source-level and cross-layer regressions; evidence reflects actual runs | Each plan/G6 |

Do not move a target invariant into the **established implementation** ledger until a concrete accepted gate proves it.

## 5. Stable interfaces and takeover map

| Domain | Existing interface / source owner | Planned consumer guarantee |
|---|---|---|
| Module source identity | `WorkspaceModuleSession`, `WorkspaceModuleUpdate` (`phalcom-modules`) | Newest accepted source persists through semantic failure |
| Semantic generation | `SemanticWorkspaceSession`, `SemanticWorkspaceInput`, `SemanticWorkspaceUpdate` | Strict, transactionally committed semantic authority after P1 |
| Query outcomes | `phalcom-semantic/src/db/state.rs::QueryOutcome` | Operational vs epistemic distinctions preserved |
| Query caches/deps | `phalcom-semantic/src/db/{mod,dependency}.rs` | Candidate work cannot contaminate committed reuse |
| Snapshot | `Arc<SemanticSnapshot>`, `SnapshotStatus` | Explicit generation and successful completion |
| Compiler | `phalcom-core/src/modules/{compile,compile_session,canonical_universe}.rs` | Only strict publication enters lowering |
| LSP | `phalcom-lsp/src/analysis_service.rs` | Epoch-scoped request-visible publication, stale recovery only |
| Subtype relation | `phalcom-semantic/src/types/{relation,outcome}.rs` | Quantified correctness and typed outcomes |
| Type specialization | `phalcom-semantic/src/types/{environment,instantiation,substitution}.rs` | Lossless structural reconstruction |
| Structural intrinsics | `phalcom-semantic/src/types/{projection,type_member}.rs` | Symbolic/closed/open operations reflect accepted specs |

This is a takeover map, **not** a list of interfaces already corrected.

## 6. Verification ledger

P1 G0 has executed. The current test results below are baseline/scope evidence only; they do not close P1 acceptance gates or C1 G0.

| Gate | Scope | Command/target | Result | Evidence / next action |
|---|---|---|---|---|
| C1 G0 | Toolchain, source/test baseline, source-path census | Determine at execution T0 | NOT_RUN | Record live HEAD, dirty tree, pinned toolchain and exact selected test counts |
| P1 G0 | Baseline and scope | Pinned-toolchain semantic cancellation test; current INC-001 parser reproducer | PASS | `RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions::cancelled_update_preserves_last_known_good` — 1 passed, 2,159 filtered. `RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' cargo test -p phalcom-ast parser::record_pattern_remainder_is_terminal_and_map_patterns_are_explicit` — 1 failed, 307 filtered; classified D/unrelated INC-001 baseline. Pinned nightly used because the directory override is later than `rust-toolchain.toml`. |
| P1 T1 | Mutable-state/caller census and design decision | Source inspection + read-only architecture consultation | COMPLETE (design only) | Per-key DB journal and persistent interner roots adopted in plan amendment A1; no implementation or behavioral gate is implied |
| P1 T2 | Failure oracles and publication authority | Pinned-toolchain semantic lib unit tests | RED ORACLES ESTABLISHED | `cargo test -p phalcom-semantic --lib session::publication_tests -- --list` selected 4 tests. The 4-test run failed at the intended pre-fix authority assertions; a focused repeat of `failed_module_update_retains_its_pending_semantic_delta` failed because both accepted modules were absent from pending state after repeated injected failures. Existing P1 G0 cancellation test passed (1 selected). These are pre-fix regressions, not green acceptance. |
| P1 G1 | Authority API | Strict session/workspace/module APIs; semantic tests | FOCUSED PASS | Failures use typed results; the synthetic success fallback is removed. |
| P1 G2 | Fresh failure | `session::publication_tests` | FOCUSED PASS | Fresh injected failure returns `Err` with no snapshot; operational failure categories remain distinct. |
| P1 G3 | Transactional rollback | Publication, DB journal, persistent-root tests | FOCUSED PASS | 13 publication tests, 2 DB transaction tests, and 3 persistent-root tests pass, including candidate failpoints, provenance, and stable IDs. |
| P1 G4 | Durable retry | `session::publication_tests` | FOCUSED PASS | Repeated failures, remove/re-add, empty-batch retry, deep frontier, and zero-budget exhaustion preserve accepted changes. |
| P1 G5 | Cold equivalence | `session::publication_tests`, `workspace_indexing` | FOCUSED PASS | Retry projection equals cold analysis; `workspace_indexing` selected 19, all passed, including cold parity and import retargeting. |
| P1 G6 | Compiler strictness | Compiler/Universe focused unit tests; core/LSP test-target build | PARTIAL | Typed error mapping tests pass (1 each); affected targets compile. End-to-end injected failures through every compiler entry remain unverified. |
| P1 G7 | LSP cutover | LSP scan-publication and single-world tests | PARTIAL | 2 scan classifier tests, 1 scan publication integration test, and 2 cutover tests pass. Failed-scan classification and epoch staleness are covered; recovery range/provenance integration remains unverified. |
| P1 G8 | Resource reuse | Persistent roots, workspace indexing, recomputation, type-store revisions | BASELINE_BLOCKED | Structural-sharing tests pass. Current reuse suites reproduce the same failures on base `b00d75b3`; no P1 regression is established. Full benchmark comparison is incomplete. |
| P1 G9 | Lifecycle closure | This record and active plan | INCOMPLETE | Keep P1 active; finish G6/G7/G8 evidence, then record incident dispositions and completion artifacts. |
| P2–P5 | Relation, Dynamic, specialization, intrinsic acceptance | Define in their respective plans | NOT_PLANNED_AS_TESTS | Author complete patch-grade plans before source work |
| C1 G6 | Consolidated certification | Define at checkpoint integration stage | NOT_RUN | Require cross-plan law-based, compiler/LSP, cold/incremental results |

### Baseline observation and current classification

INC-001 reported a parser/AST test binary with **300 passed, 7 failed, 1 ignored** in an earlier workspace. The exact record-pattern parser reproducer was rerun at the current revision and still fails with `UnrecognizedToken { token: "name", expected: ["\"#\""] }`. Six stale snapshot failures were not rerun; the AST source diff from the earlier baseline reference to current HEAD is empty, and those tests create `.snap.new` files. Treat those six as historical, unconfirmed-at-current-revision baseline observations. The checkout's directory override is `nightly-aarch64-apple-darwin` (`rustc 1.99.0-nightly`, 2026-08-07); commands above explicitly used pinned `nightly-2026-07-10` (`rustc 1.99.0-nightly`, 2026-07-09).

### P1 T1 mutable-state census and implementation disposition

| State owner | Live mutation/evidence | Candidate classification and transaction obligation |
|---|---|---|
| `SemanticDb` | Revision/event sets; query state, products/LKG, module key index, dependency index, scheduler, metrics. | Implemented per-key undo journal covers states/products/LKG, module ownership, both dependency directions, revision metadata, and scheduler. Metrics remain attempt telemetry. No whole-DB clone. |
| Session source/shard roots | `semantic_structure_shards` changes before query work; `sources` and fingerprints install at commit. | Shards restore from the prior root on error; source/fingerprint roots commit only after the final fallible/cancellation point. |
| Session-derived roots | Field lifecycle, generic/signature dependencies, alias maps/forms, diagnostics, and default lifecycle are built as candidates. | Installed with the snapshot at the single semantic commit point. |
| Deep-analysis frontier | Requested deep modules feed candidate scheduling. | Retained on failure and cleared only after commit; the focused retry test passes. |
| Type/static-value interners | Snapshot-retained `Arc` roots previously deep-cloned through `Arc::make_mut`; TypeStore may refresh parameter provenance. | Persistent `im::Vector`/`im::HashMap` backing in TypeStore, nested TypeLambdaArena, and StaticValueStore preserves store/numeric IDs and per-snapshot provenance; pointer-sharing tests pass. |
| Module/source session | Module batches commit accepted source/topology before semantic update (`session.rs:811–849`). | Separate module commit remains authoritative input and must not be rolled back on semantic failure. |

**Consumer census:** production routes and test adapters were migrated. Inline/standalone compiler analysis propagates typed failure; canonical Universe separates semantic error from panic; LSP scans report failed semantic publication and do not call `publish_snapshot` on `Err`. The semantic failpoint remains private to unit tests. `compile_analyzed` remains the lowering boundary, not an analysis entry point.

**Pending-ledger and publication boundary:** `SemanticModuleDelta` merges changed, removed, identity-changed modules, and graph changes without destructive `take`, filters against current source membership, and clears only after semantic commit. Repeated-failure, remove/re-add, empty-batch, deep-frontier, and zero-budget retry tests pass. The module source commit remains `WorkspaceModuleSession::apply_batch`; semantic publication is the final root/snapshot installation; LSP request-visible publication is separate and epoch-checked.

**T1 disposition:** The census is implemented under adopted amendment A1. DB rollback and interner persistent roots have focused structural and behavior tests. No architecture amendment beyond A1 was needed.

**T2/T3–T10 evidence:** The per-session failpoint covers revision begin, shard replacement, query retirement, source-fingerprint staging, static-value mutation, query publication, pre-commit failure, and cancellation at commit. `session::publication_tests` now selects 13 passing tests. T8/T9 code migrations are present, but end-to-end failure injection across every compiler entry and recovery-source provenance remain open as recorded in the gate table.

## 7. Deferred and baseline issue ledger

| ID | Observation / classification | Why not folded into P1 | Revisit |
|---|---|---|---|
| INC-001 | Current parser reproducer still fails (D, unrelated); six stale snapshot failures remain historical/unconfirmed at current revision | P1 owns semantic publication; avoid generating snapshot artifacts during this gate | C1 G6 |
| INC-002–005, INC-008–010, INC-012–013 | Independent source-confirmed semantic findings | Assigned to P2–P5 rather than P1 | Their owning plans; C1 G6 |
| INC-006 | Cleared, withdrawn candidate | Normative finite-lane projection model contradicted the suspected defect | Only on fresh concrete evidence |
| INC-007 / INC-011 | Implementation corrections are present; forced failure, rollback, repeated retry, empty-batch recovery, budget exhaustion, and cold-projection tests pass. Compiler/LSP route coverage and G8 closure evidence remain open. | Do not close before all P1-required gates | P1 G6–G9 |
| P1 recomputation baseline | `recomputation_metrics`: 8 passed, 5 failed on current code; the exact five failures and reported values reproduced on base `b00d75b3` (D). `type_store_revisions`: 19 passed, 3 failed on current and the same three on base (D). | These do not establish a P1 regression; retain as baseline blockers for G8 | C1 G6 / owning baseline work |
| Canonical bootstrap baseline | `dependency_free_project_compiles_then_bootstraps_and_runs` failed on current and base `b00d75b3` with the same legacy open-Record/Symbol-key metadata failure and stack overflow (D). | Do not broaden P1 into metadata/type-system semantics | C1 G6 / relevant incident owner |

If another baseline failure is encountered, record observed command, reproduction, classification (A/B/C/D under Luna schema), affected gate and owner; do not silently widen the patch.

## 8. Consultation and amendment ledger

| Date | Incident / task | Trigger | Decision | Plan amendment | Status |
|---|---|---|---|---|---|
| 2026-10-12 | Program setup | Planning-only scope | Adopt provisional TYPEAUD001 C1 + five-plan architecture; do not claim code work | None | PLANNING DECISION; not an implementation consultation |
| 2026-10-12 | TYPEAUD001.C1.P1.T1 / A1 | Retained snapshots share Arc roots; `Arc::make_mut` deep-clones arenas, and TypeStore mutates existing parameter provenance | Per-key SemanticDb undo journal plus persistent COW roots in TypeStore, TypeLambdaArena, and StaticValueStore; preserve store/numeric IDs and snapshot-local provenance | P1 plan A1; add CV-31 and extend T5/G3/G8 | ADOPTED; T1 complete |
| 2026-10-12 | TYPEAUD001.C1.P1.T2 | Existing infallible routes swallow injected semantic failure and destructive pending-delta merge loses accepted changes | Four focused pre-fix assertions established; private failpoint does not expose a production API | T2 tests in `phalcom-semantic/src/session/publication_tests.rs` | RED ORACLES ESTABLISHED; proceed to T3 |
| 2026-10-12 | TYPEAUD001.C1.P1 implementation | Session/DB candidates mutate before final publication; strict callers depended on legacy infallible APIs | Adopted A1; implemented DB undo, persistent interner roots, strict result APIs, retry ledger, compiler/Universe/LSP propagation; no architecture change beyond A1 | Existing P1 amendment A1 | IMPLEMENTED; G6/G7/G8 evidence still incomplete |

**Future escalation record format:** source excerpt and revision; observation, prediction and minimal discriminating test; affected owner/invariant; alternative approaches; required architect ruling; amendment and new gate. STOP on changes to source/module commit ownership, public formal authority, TypeStore identity, accepted specifications, or soundness-critical semantics. A triggered consultation is not self-waived.

## 9. Active plan and next action

```yaml
checkpoint: TYPEAUD001.C1
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
active_plan: TYPEAUD001.C1.P1
latest_completed_plan: null
next_plan: TYPEAUD001.C1.P1
next_action: >
  Complete the remaining P1 G6 compiler failure-route and G7 recovery-provenance
  checks, close G8 with baseline-qualified reuse evidence, then record incident
  dispositions and P1 completion artifacts. Keep INC-007/011 open until then.
```

**P1 execution evidence:** Implementation commit `10f7297c` is pushed to `origin/main`, based on `b00d75b3`. Focused results and baseline comparisons are recorded in the active plan. Workspace integration selected 19 tests and passed all 19; publication unit tests selected 13 and passed all 13, including durable retry after budget exhaustion. Broad workspace tests, clippy, and release certification were not run. The plan remains active because G6–G8 evidence and T11 acceptance work are incomplete.

**Completion artifacts for each plan:** its own walkthrough and handoff, updated test/verification ledger, incident closeout evidence. No C1 completion or release certification is claimed here.
