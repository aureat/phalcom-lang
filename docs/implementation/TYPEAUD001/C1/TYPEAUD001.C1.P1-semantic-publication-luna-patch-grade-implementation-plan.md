---
id: TYPEAUD001.C1.P1
category: semantic-correctness
program: TYPEAUD001
checkpoint: TYPEAUD001.C1
kind: implementation-plan
title: "Semantic Publication Integrity and Transactional Workspace Updates"
status: IN_PROGRESS
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
branch: main
prepared_against_revision: b00d75b3ac9f50d37cf9abfddfde00917e099031
prepared: 2026-10-12
planner_grade: luna-patch-grade
depends_on:
  - TYPEAUD001.C1.G0-baseline-and-authority-gate
follows: null
supersedes: null
incidents:
  - INC-007
  - INC-011
amendments:
  - TYPEAUD001.C1.P1.A1
---

# TYPEAUD001.C1.P1 — Luna Patch-Grade Implementation Plan
## Semantic Publication Integrity, Transactional Updates, and Failure Recovery

> **Execution mandate:** Correct the publication-authority and failure/retry defects identified by the type-system audit. This plan is an implementation contract, **not** a claim that implementation or tests have been performed. The program/checkpoint identifiers are proposed; if the repository has reserved them or adopted another identifier, perform a lifecycle-only identifier reconciliation before code edits, preserving this plan's technical contract.

## 0. Executor contract

The implementation agent works on the user-authorized branch (target: `main`) and must follow `AGENTS.md`, `docs/workflow/luna-patch-grade-plan-schema.md`, `docs/workflow/implementation-record-lifecycle-convention.md`, and `docs/workflow/shared-consultation-escalation-protocol.md`.

- **FIXED:** Preserve source/module lifecycle commits independently of semantic publication; failed semantic candidates are not authoritative; retry must be cold-equivalent; no fake `Complete` snapshot; no ordinary per-edit full database/interner deep copy; no parallel LSP semantic authority.
- **MECHANICALLY FLEXIBLE:** Private helper names, staging/journal implementation details **within the specified transaction model**, private file decomposition, conversion traits, and test fixture placement.
- **VERIFY-FIRST:** Exact live signatures, existing tests, module-session generation semantics, transaction write-set, scheduler mutation escape hatches, and whether a natural failure reproducer exists.
- **STOP AND CONSULT:** A required public contract, ownership boundary, interner identity invariant, incremental performance contract, or semantic outcome must change beyond the rulings here; or the transactional journal cannot cover a mutable state without redesigning its owner.
- Do **not** change other type-system audit incidents, language semantics, or VM behavior to make these gates pass.
- Do **not** run broad workspace tests reflexively. Follow the selected verification ladder and record the actual number of selected/passed tests.
- Update the shared checkpoint only at meaningful lifecycle, invariant, consultation, or verification milestones; produce the walkthrough and handoff at completion.

**Important safety rule:** This plan does not authorize deleting/resetting unrelated user changes, blindly rebasing, or pushing without the executor's actual authorization context. Read current worktree/branch instructions first.

---

## 1. Goal and checkpoint acceptance

**Goal:** Make all authoritative semantic workspace publications explicit, all candidate semantic mutations transactional, all uncommitted module changes retryable, and all compiler/LSP consumers generation-correct under failure, cancellation, budget exhaustion, and success.

This plan closes only **INC-007** and **INC-011** inside the wider type-system remediation checkpoint. It does **not** close TYPEAUD001.C1, whose remaining planned work covers relation algebra, Dynamic arguments, structural materialization, and intrinsic projections.

**Minimum successful result:**

```text
Accepted module/source changes remain current independently of semantic failure.
A failed semantic update returns an explicit failure and publishes no new authority.
No failed candidate mutates the committed semantic roots used for a later update.
Pending semantic changes survive failed attempts, including an empty-batch retry.
A later successful publication is semantically equivalent to a fresh analysis.
Compiler/Universe bootstrap never use fallback products as successful analysis.
LSP interactive and scan publications never mark an unsuccessful candidate fresh.
Normal successful incrementality preserves exact dependency-based reuse.
```

## 2. Baseline and repository grounding

| Fact | Confirmed owner / location | Consequence |
|---|---|---|
| Fresh semantic entry | `phalcom-semantic/src/workspace.rs:117–127`, `analyze_workspace` | Currently constructs a session, calls infallible `update`, returns snapshot only. |
| Infallible fallback | `phalcom-semantic/src/session.rs:922–960`, `update`/`update_with_delta` | Swallows all `QueryOutcome` failures; returns stale or synthetic `Complete` snapshot. |
| Strict update | `session.rs:963–~978`, `update_with_budget_and_cancel` | Existing strict public API to preserve/extend. |
| Strict module update | `session.rs:811–849`, `update_module_workspace_with_cancel` | Restores pending merged delta on `Err`; no success-shaped fallback. |
| Infallible module update | `session.rs:864–~893`, `update_module_workspace` | Takes pending delta; on failure never restores it. |
| Module batch convenience | `session.rs:755–809`, `apply_module_mutation[s]`, `_with_deep_modules`, `_with_cancel` | Infallible paths expose semantic failure as module-success result. |
| Snapshot state | `phalcom-semantic/src/snapshot.rs:134–145`, `:295–356`, `:869–873` | `Complete` is constructor default; `has_errors` only sees source diagnostics. |
| Query DB | `phalcom-semantic/src/db/mod.rs:34–50`, `:95–101`, `:300–460` | Revision, Ready products, LKG, dependency/index state mutate before workspace commit. |
| DB state model | `phalcom-semantic/src/db/state.rs` | `Ready/Blocked/Cancelled/BudgetExceeded/Failed` already distinguished. |
| Session pre-commit mutations | `session.rs:984`, `:1055`, `:1109–1129`, `:1255–1283`, `:1482` | Revision, shards, retirement, invalidations, sources/fingerprints, deep roots mutate before final snapshot. |
| Late commit | `session.rs:8522–8552` | Final snapshot plus several derived roots installed only after many possible failure sites. |
| Interactive LSP | `phalcom-lsp/src/analysis_service.rs:794–865` | Uses strict/cancellable publication and request-visible epoch check. |
| LSP scanner | `analysis_service.rs:1118–1130` | Uses infallible module-batch conveniences; must migrate. |
| Compiler one-shot | `phalcom-core/src/modules/compile.rs:618,726` | Calls infallible `analyze_workspace`. |
| Compiler project | `compile.rs:797–805` | Already strict; preserve behavior. |
| Compiler projection | `compile.rs:827–843` | `has_errors` is not formal publication authority. |
| Persistent compiler | `phalcom-core/src/modules/compile_session.rs` | Already uses strict module update. |
| Universe bootstrap | `phalcom-core/src/modules/canonical_universe.rs:188–203` | Uses `analyze_workspace` under panic catcher; requires structured update-error mapping. |
| Current failure test | `phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs`, `cancelled_update_preserves_last_known_good` | Verifies retained snapshot, **not** complete transactional state. |
| Canonical workflow | `docs/workflow/luna-patch-grade-plan-schema.md` and lifecycle convention | Plans must have task/gate evidence, consultation and handoff. |

**Base:** `aureat/phalcom-lang`, `main`, `b00d75b3ac9f50d37cf9abfddfde00917e099031`. The base commit follows the audit slice-6 SHA but modifies LANG007.C4 static-evaluation specialization, not the primary P1 publication paths. Reconfirm immediately before execution.

**First commands in local checkout:**

```sh
git status --short
git rev-parse HEAD
git rev-parse origin/main
git log -1 --oneline
cat rust-toolchain.toml
rg -n 'fn (update_with_delta|update_with_budget_and_cancel_and_delta|update_module_workspace|apply_module_mutations)|pub fn analyze_workspace' phalcom-semantic/src
```

**Repository-drift rule:** Re-read live names, implementations, and tests. Mechanical offsets may change. If the core session/db mutation model or module/source commit boundary changed materially, **STOP AND CONSULT** instead of applying a stale transaction plan.

### 2.1 Must-read authority

```text
AGENTS.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/specs/README.md
docs/wiki/semantic/workspace-incrementality.md

docs/audit/type-system-2026-10-12/incidents/
  INC-007-semantic-update-failure-disguised-as-complete-snapshot.md
```

INC-011 is from audit Slice 5; if its incident Markdown is not present in the live repository, recover it from the audit handoff or the supplied downloadable record. Do not invent an unavailable repository path; attach/reconcile the existing incident document into the audit location as a documentation action only when authorized.

```text
phalcom-semantic/src/{session.rs,workspace.rs,snapshot.rs,semantic_shard.rs}
phalcom-semantic/src/db/{mod.rs,state.rs,dependency.rs,query.rs,scheduler.rs,metrics.rs}
phalcom-modules/src/  [targeted WorkspaceModuleSession implementation]
phalcom-core/src/modules/{compile.rs,compile_session.rs,canonical_universe.rs}
phalcom-lsp/src/{analysis_service.rs,request_context.rs}
phalcom-semantic/tests/semantic/incremental/{type_store_revisions.rs,recomputation_metrics.rs,workspace_indexing.rs}
phalcom-lsp/tests/single_world_cutover.rs
```

**Normative/provenance hierarchy:** `docs/specs` governs language semantics; `docs/wiki/semantic/workspace-incrementality.md` describes the intended immutable/candidate publication contract; existing code defines the current mechanism but does not override it; incident reports are evidence and acceptance obligations, not new language specifications. Preserve the distinction between successfully published `Partial` workspace analysis and failed infrastructure computation.

---

## 3. Stable takeover interfaces and authority

| Concept | Canonical owner | Contract to preserve |
|---|---|---|
| Accepted source/overlay revision | `WorkspaceModuleSession` (`phalcom-modules`) | Can advance before semantic success; must not roll back simply because semantic work is cancelled. |
| Semantic computation and query products | `SemanticWorkspaceSession` + `SemanticDb` | Only coherent committed generation establishes current formal authority. |
| Immutable published semantics | `Arc<SemanticSnapshot>` | No mutation after publication; generation/source/provenance stable. |
| Formal query completion | `QueryOutcome<T>` | Distinguish source diagnostics from operational failure. |
| Recovery/LKG | Semantic session, presentation adapter | Previous snapshot may be read-only advisory recovery; never fresh authority. |
| Editor request visibility | LSP worker/epoch & snapshot publisher | Latest-wins epoch validation remains separate from semantic commit. |
| Compiler admissibility | Compiler entry points | Require successful formal semantic publication before lowering. |
| TypeId / interner | Canonical `TypeStore` | Existing session store identity and snapshot references remain valid. |
| Incremental invalidation | Semantic DB dependency index + module delta | Exact dependency ownership and fingerprints remain canonical; no second LSP invalidator. |

**Direction of authority:** accepted source/module input → canonical semantic transaction → immutable successful semantic snapshot → compiler/LSP/metadata consumers. Recoverable stale snapshots are *presentation-only* and never strengthen formal facts.

---

## 4. Architecture — FIXED contracts

### 4.1 Separate three generations and two visibility boundaries

At minimum track:

```text
accepted_module_generation      newest committed source/module state
attempted_semantic_generation   module generation being analyzed
published_semantic_generation   generation of last successful formal publication
```

There are **two distinct linearization points**:

1. **Semantic-session commit:** installs one internally coherent semantic state and immutable snapshot.
2. **LSP request-visible publication:** installs the committed candidate only if the requesting worker epoch remains current.

A candidate may succeed at point 1 and be discarded as a superseded request-visible publication at point 2. That is not a failed semantic transaction; later source edits must still be correctly tracked. Never conflate source-generation acceptance with LSP epoch acceptance.

### 4.2 Public result boundary

**Required public behavior** (illustrative Rust shapes; exact names are mechanically flexible):

```rust
pub enum SemanticUpdateFailureKind {
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(String),
}

pub struct SemanticUpdateFailure {
    pub attempted_generation: u64,
    pub published_generation: Option<u64>,
    pub kind: SemanticUpdateFailureKind,
}

pub fn update(
    &mut self,
    input: SemanticWorkspaceInput,
) -> Result<SemanticWorkspaceUpdate, SemanticUpdateFailure>;

pub fn update_with_budget_and_cancel(
    &mut self,
    input: SemanticWorkspaceInput,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> Result<SemanticWorkspaceUpdate, SemanticUpdateFailure>;

pub fn analyze_workspace(
    input: SemanticWorkspaceInput,
) -> Result<SemanticAnalysis, SemanticUpdateFailure>;
```

`update_module_workspace` must also be strict. Module mutation convenience APIs must distinguish module-layer and semantic-layer errors, e.g.:

```rust
pub enum WorkspacePublicationError {
    Module(WorkspaceModuleSessionError),
    Semantic(SemanticUpdateFailure),
}
```

A compatibility overload retaining the existing `QueryOutcome<()>` error shape for already-strict callers is acceptable only as a *lossless adapter* of the one canonical error path. **Forbidden:** public infallible `update`/`apply_module_mutations` returning a conventional `SemanticWorkspaceUpdate` after swallowed failure.

**No need to add `Failed` or `Stale` to `SnapshotStatus` to represent failed updates.** A failure does not produce an authoritative new snapshot; `Complete`/`Partial` classify genuinely successful snapshots. If the LSP displays recovery, use an explicitly typed `RecoveryView` with `attempted_generation`, original `published_generation`, failure reason, and `Option<Arc<SemanticSnapshot>>`. Keep old snapshot `generation`, sources, and diagnostic provenance unchanged.

### 4.3 Transaction model: candidate roots + reversible mutation boundary

**Architectural choice:** Implement a *semantic publication transaction* with per-key DB mutation journaling or equivalent persistent candidate-root overlay, plus staged session-owned derived roots. The transaction must isolate all future-authority-relevant writes. Avoid eagerly cloning the entire SemanticDb/TypeStore on every update.

The implementer may choose between **DB undo journal** and **DB persistent candidate overlay** *only if* both satisfy the identical required interface/semantics below and the chosen mechanism is justified by a read/write/cost census in T1. The implementer does not have authority to choose shallow snapshot-only rollback, unconditional global cache clearing, or per-edit deep cloning instead. If the two compliant choices have materially different observable identity or performance effects, **STOP AND CONSULT** rather than silently selecting one.

Logical internal interface:

```rust
// Conceptual API; not required to use these exact identifiers.
let attempt = session.begin_semantic_attempt(input, merged_delta, cancel)?;
let candidate = attempt.evaluate()?;
let validated = attempt.validate(candidate)?;
let publication = attempt.commit(validated); // one authoritative commit point
// Any error path discards/restores candidate writes and retains pending work.
```

In the DB, cover **all** mutators, including:

- `begin_revision`, revision-recomputed/revalidated sets;
- `set_state`, query key index additions;
- `discard_for_recompute`, `publish_ready`, `publish_product_ready`, `update_callable_body_product`;
- `invalidate`, `invalidate_roots`, `retire_query`, `purge_module`;
- `validate_reuse`, `validate_ready`, dependency recording/replacement;
- `products`, `last_known_good`, dependency index forward/reverse edges;
- scheduler state or queued work if mutations escape via `scheduler_mut`;
- metrics visibility (may track attempted work separately, but cannot falsely claim published work).

For an undo journal, capture original values **before first write per transaction/key**, including removed query identities and prior dependency adjacency. Rollback in a dependency-safe order, then restore revision metadata and publication roots. For a candidate overlay, preserve old roots until commit and ensure reads within an attempt see candidate changes consistently. **Do not copy or restore just the `QueryState` map and ignore the dependency index.**

Session staged roots must include `sources`, `source_fingerprints`, `semantic_structure_shards`, `field_lifecycle_fingerprints`, dependency/alias maps, diagnostic maps, static values, TypeStore references, scheduling frontier, and every other mutable field revealed in T1. Safe append-only interners may retain allocations only if proofs/tests establish that failed internings cannot change identity, fingerprints, memory lifetime, or subsequent formal results. Staging or rollback is otherwise required.

**Strong recommendation:** Extract an internal candidate-state wrapper and a small commit function before rewriting the 12,000-line analysis flow. Prefer moving existing computed local maps into that wrapper. Do not duplicate the semantic solver or its business logic.

#### Amendment A1 — persistent interner roots (2026-10-12)

The T1 consultation found that the original transaction choice under-specified snapshot-retained interners: `SemanticSnapshot` holds `Arc<TypeStore>` and `Arc<StaticValueStore>`, while update evaluation calls `Arc::make_mut`; both stores currently deep-clone their arenas/maps, and TypeStore can update provenance for an existing parameter ID. An in-place append journal would therefore mutate retained snapshot state, while the current `type_store_snapshot_deep_copies = 0` counter does not observe those `Arc::make_mut` copies.

**Adopted design:** use a per-key undo journal for `SemanticDb`, plus persistent copy-on-write roots inside `TypeStore`, its nested `TypeLambdaArena`, and `StaticValueStore`. Candidate store clones must structurally share committed arena/index data and write new entries or provenance changes into candidate-owned persistent nodes. A candidate keeps the same store ID and all previously allocated numeric IDs. Existing snapshot roots must continue to resolve those IDs with the provenance they had when the snapshot was published. Abort drops candidate roots; commit installs them with the rest of the semantic publication. Do not replace this with eager full-store cloning, global store IDs, or an append-only exception that lets failed candidate allocations affect later formal results.

This amends T1/T5, CV-09/CV-25, and G3/G8. Focused tests must exercise a retained old snapshot across failed and successful source-provenance updates, new interning followed by abort/commit, stable store/type IDs, and structural sharing or equivalent instrumentation proving an ordinary update does not copy an entire store arena. The existing zero-valued snapshot-copy counter is not acceptance evidence by itself.

### 4.4 Pending delta is a durable ledger

The pending delta describes *all accepted module/source changes since the last successful semantic commit*, not only the most recent failed attempt.

**Forbidden before success:** `self.pending_semantic_delta.take()` without guaranteed restoration along **all** failure paths.

```rust
let combined = combine(
    self.pending_semantic_delta.as_ref(),
    &current_delta,
);
match self.try_commit_semantic(input, &combined) {
    Ok(publication) => {
        self.pending_semantic_delta = None;
        Ok(publication)
    }
    Err(error) => {
        self.pending_semantic_delta = Some(combined);
        Err(error)
    }
}
```

This is pseudocode. Preserve old + newly accepted changes with a **non-destructive merge** or a scoped guard. Invariants must hold for changed, removed, and reintroduced modules, identity replacements, graph changes, and deep-analysis scheduling. Record exact **base successful generation** and attempted generation if needed to validate the ledger. A set union of all delta fields is **not** automatically correct for identity transitions; inspect the actual `WorkspaceModuleUpdate` and `SemanticModuleDelta` meaning first.

A zero-mutation retry after failure must be supported and must analyze the already-accepted latest module state. A cancellation-triggered empty batch must not erase pending work.

### 4.5 Source/module transaction remains independent

`WorkspaceModuleSession::apply_batch` may commit accepted overlays/topology *before* semantic analysis starts. This is intentional. On semantic failure do **not** roll back module session source state; keep latest accepted inputs and retry semantic publication from them. Distinguish this from rolling back the **semantic session's copied source/fingerprint roots** to its last committed semantic generation.

### 4.6 All-or-nothing semantic commit

Publish candidate only when:

1. Input module generation and source/topology identity remain the intended target.
2. All required queries completed sufficiently for the publication policy; blocked *source* modules are not confused with operational `QueryOutcome::Blocked`.
3. Candidate dependency index, products, fingerprints, semantic source roots, static values, and TypeStore references are internally coherent.
4. No cancellation was observed before the semantic commit linearization point.
5. Candidate's diagnostic/source-index/projection roots refer to the same generation.
6. `last_snapshot` and session-owned successful state are installed as one logical commit.
7. Publication effects are emitted only from committed state.
8. Pending ledger is consumed only after commit succeeds.

`SnapshotStatus::Partial { blocked_modules }` may still be an **honestly successful** publication under the pre-existing partial-workspace contract. Its acceptance by compiler is a separate executable-product rule; P1 must not ban it universally.

### 4.7 Invariants (test references are mandatory)

| ID | Durable invariant |
|---|---|
| INV-01 | `Err` from semantic analysis never yields an authoritative new `SemanticWorkspaceUpdate` or snapshot. |
| INV-02 | No synthetic `Complete` snapshot is manufactured after first-generation failure. |
| INV-03 | Published `Arc<SemanticSnapshot>` and every authoritative session root remain consistent across failure. |
| INV-04 | Candidate query Ready/products/dependency edges cannot contaminate committed DB reuse eligibility. |
| INV-05 | Accepted source/module generation is preserved independently of failed semantic publication. |
| INV-06 | Pending semantic changes/deep frontier survive all failures and an empty-batch retry. |
| INV-07 | Every successful retry after failures agrees with cold analysis of identical final sources. |
| INV-08 | `Cancelled`, `BudgetExceeded`, `Blocked`, `Failed` remain distinguishable from diagnostics and each other. |
| INV-09 | Compiler/Universe bootstrap require strict formal publication before lowering; `has_errors()` alone is insufficient. |
| INV-10 | LSP scan + interactive workers never publish failed or superseded candidates as current; stale recovery retains original revisions. |
| INV-11 | Committed TypeStore identity and retained TypeIds remain valid across candidate failure and later success. |
| INV-12 | Unchanged successful edits preserve incremental cache precision; no routine whole-state deep-copy fallback. |
| INV-13 | `SnapshotStatus::Partial` remains a valid successful state only under the existing, explicit completeness policy. |
| INV-14 | Semantic-session commit and LSP request-visible epoch commit remain separate, correctly ordered decisions. |
| INV-15 | Query/database failure occurs at an explicit, testable transaction phase; no silent swallowing of infrastructure failure. |

### 4.8 Non-goals

No general subtype/variance/union changes, Dynamic semantics, Tuple/Record materialization fixes, structural intrinsic work, language syntax, VM redesign, module architecture rewrite, or broad diagnostic redefinition. INC-001 parser/snapshot baseline remains separately tracked. No blanket conversion of every source error into infrastructure failure. No redesign of successfully authored `Partial` snapshots except where failure authority genuinely requires it.

---

## 5. Implementation ownership and impact map

### Production files — planned

| File | Required change |
|---|---|
| `phalcom-semantic/src/session.rs` | One strict update coordinator; candidate roots; guarded pending ledger; single semantic commit; failure test seam; typed failure/recovery. |
| `phalcom-semantic/src/db/mod.rs` | Transaction mutation boundary and rollback/commit for DB roots, products, query revisions, LKG, indexes. |
| `phalcom-semantic/src/db/dependency.rs` | Accessor or undo/overlay helper for forward/reverse edge mutation, only if DB transaction requires it. |
| `phalcom-semantic/src/db/state.rs` | Lossless outcome→typed update failure conversion if needed; preserve QueryOutcome variants. |
| `phalcom-semantic/src/db/scheduler.rs` | Candidate scheduling restore/isolation if needed, after write-set census. |
| `phalcom-semantic/src/workspace.rs` | Fallible `analyze_workspace`/`analyze_single_module`; no ordinary success-shaped failure. |
| `phalcom-semantic/src/snapshot.rs` | Only minimal constructor/authority guard changes; do not add failure as normal snapshot status. |
| `phalcom-semantic/src/lib.rs` | Re-export new result/error/recovery types consistently. |
| `phalcom-core/src/modules/compile.rs` | Migrate inline + standalone analysis, preserve strict project route, compiler authority checks. |
| `phalcom-core/src/modules/compile_session.rs` | Adapt lossless error types without downgrading strict semantics. |
| `phalcom-core/src/modules/canonical_universe.rs` | Structured semantic update failure mapping; preserve existing panic distinction and bootstrap-specific diagnostics rule. |
| `phalcom-lsp/src/analysis_service.rs` | Scan paths move to strict batch publication; preserve interactive cancellation/epoch checks; classify errors instead of ignoring. |
| `phalcom-lsp/src/request_context.rs` | Only changes needed to enforce explicit stale/exact provenance. |
| `phalcom-modules/...` | Read-only contract inspection first; any material change to module/source commit semantics requires consultation. |

### Focused new/updated tests — planned

- `phalcom-semantic/tests/semantic/incremental/publication_transaction.rs` (new; register in owning `mod.rs`/test harness).
- `phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs` (extend existing tests).
- `phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs` (reuse precision).
- `phalcom-semantic/tests/semantic/incremental/workspace_indexing.rs` (source index equivalence).
- `phalcom-semantic/tests/semantic/incremental/db_base.rs` (DB journal/overlay direct regressions).
- `phalcom-core/tests/core/modules/...` (new focused strict-publication integration tests, registered according to existing harness).
- `phalcom-lsp/tests/single_world_cutover.rs` and targeted `phalcom-lsp` test module(s).

Test-only hooks placed in private `#[cfg(test)]` code cannot automatically be exercised from external integration crates; **decide a supported test-support feature or internal-unit-test placement in T1**, without exposing fault injection as a production public API.

### Expected lifecycle files — planned, not currently asserted to exist

```text
docs/implementation/TYPEAUD001/C1/
  TYPEAUD001.C1-CHECKPOINT.md
  TYPEAUD001.C1.P1-semantic-publication-luna-patch-grade-implementation-plan.md
  TYPEAUD001.C1.P1-walkthrough.md
  TYPEAUD001.C1.P1-handoff.md
```

**Unexpected-touch rule:** Adjacent helper changes are allowed when mechanically essential. Entering unrelated semantic domains, changing compiler diagnostic/source semantics, or altering module commit ownership requires a consultation.

---

## 6. Implementation decision authority and preflight assumptions

### FIXED

1. Semantic failure is a `Result::Err`, not source diagnostic, empty-success snapshot, or stale-success return.
2. Accepted module changes remain committed across semantic failure.
3. Semantic publication is one logical commit across snapshot, DB, derived roots, and pending ledger.
4. Last-known-good is **read-only recovery**, not current-generation proof.
5. DB mutation journal/overlay is complete; no partial-rollback approximation.
6. Successful partial analysis remains supported where previously accepted.
7. Performance: ordinary successful edits remain incremental; cold fallback only as a **bounded exceptional recovery choice requiring consultation**, never default.
8. No Rust `panic!`, `unwrap`, or `expect` is introduced to convert recoverable semantic update failures into pseudo-success or crashes.

### MECHANICALLY FLEXIBLE

- Private structure placement, helper signatures, error wrapper names, trait implementations, Rust module registration, test fixtures, documentation table organization, and compatibility adapters that preserve exact failure semantics.

### VERIFY-FIRST checklist

| Assumption | Required inspection | Failure response |
|---|---|---|
| No new strict authority API already landed | `workspace.rs`, `session.rs`, `lib.rs` at execution HEAD | Reconcile duplicate path mechanically or consult if semantic contract differs. |
| All SemanticDb writes pass centralized mutators | `db/{mod,query,scheduler,dependency}.rs`; `rg 'db_mut\(|scheduler_mut\('` | If direct mutation escapes transaction, extend isolation; consult if owner redesign required. |
| `TypeStore` Arc copy semantics preserve stable identity | `types/store.rs`, existing `one_session_has_one_type_store_identity_across_revisions` | Design candidate store handling before editing. |
| `static_values` can be staged safely | `static_value/store.rs`, session usages | Consult if identity/proof provenance would change. |
| `WorkspaceModuleSession` accepts source before semantic success | source module implementation, `session.rs:793–805` | Do not alter that boundary unilaterally. |
| All public callers can migrate | repo-wide `rg` of APIs | Escalate unexpected external contract break; provide adapters without hiding failure. |
| Existing partial snapshot is intentional | LSP Local-mode and blocked-module handling | Preserve successful partial mode. |
| Test harness supports failure injection | `phalcom-semantic/tests/semantic.rs`, crate features | Establish isolated test-only hooks; no public production injection. |

---

## 7. Test surfaces and evidence matrix

Every item must have a **RED counterexample where feasible** and a GREEN post-fix result. If a RED cannot be executed safely, record exact source proof plus a deterministic post-fix test; do not claim an executable pre-fix failure occurred.

| Coverage ID | Case | Invariants | Primary gate |
|---|---|---|---|
| CV-01 | Fresh session, injected `Failed`: `analyze_workspace` Err, no snapshot | 01,02,08 | G2 |
| CV-02 | Fresh session, `BudgetExceeded` and `Blocked`: distinct errors | 01,02,08 | G2 |
| CV-03 | G1 successful; G2 pre-work cancellation; LKG retains G1 text/generation | 03,05,08 | G3 |
| CV-04 | Fail after `db.begin_revision`; old Ready/validation state safe | 03,04 | G3 |
| CV-05 | Fail after structural-shard creation/replacement | 03,07 | G3 |
| CV-06 | Fail after query retirement/purge and reverse-dependency edge deletion | 03,04,07 | G3 |
| CV-07 | Fail after source fingerprint/source-root staging | 03,07 | G3 |
| CV-08 | Fail after new Ready query/product/LKG insertion | 03,04,07 | G3 |
| CV-09 | Fail after candidate static values/TypeStore interning | 03,11 | G3 |
| CV-10 | Fail just before final commit; no authority leaks | 01,03,04,15 | G3 |
| CV-11 | Cancel vs success linearization; no partial result | 03,14 | G3 |
| CV-12 | Multiple failures accumulate provider/consumer deltas | 06,07 | G4 |
| CV-13 | Removal/re-addition and identity replacement across failures | 06,07 | G4 |
| CV-14 | Empty-batch retry after failed generation | 06,07 | G4 |
| CV-15 | Deep-analysis frontier retained across unsuccessful attempts | 06,13 | G4 |
| CV-16 | Cold/incremental formal products equivalent after failures | 04,07,11 | G5 |
| CV-17 | Diagnostic/source index and formal facts equivalent after retry | 03,07 | G5 |
| CV-18 | Inline, standalone, package compiler paths reject failed analysis | 01,09 | G6 |
| CV-19 | Canonical Universe bootstrap maps semantic Err distinctly from panic | 08,09 | G6 |
| CV-20 | LSP interactive cancelled/superseded candidate never request-visible | 10,14 | G7 |
| CV-21 | LSP Full/Local scans reject failed candidate | 01,10,13 | G7 |
| CV-22 | Stale recovery retains original text/ranges/diagnostic revision | 03,10 | G7 |
| CV-23 | Successful partial workspace snapshots retain existing behavior | 08,13 | G7 |
| CV-24 | Unchanged, body-only, header-only edits retain exact reuse | 07,12 | G8 |
| CV-25 | No routine deep-copy of TypeStore or full per-edit SemanticDb | 11,12 | G8 |
| CV-26 | First success after budget exhaustion uses all new source deltas | 06,07,08 | G4 |
| CV-27 | Removed query LKG not resurrected under reintroduced identity | 04,06,07 | G4 |
| CV-28 | Changed module topology, imports, and changed provider dependency frontier | 04,06,07 | G5 |
| CV-29 | Stale LSP batch after successful *session* commit still not request-visible | 10,14 | G7 |
| CV-30 | Structured failure never becomes ordinary source diagnostic | 01,08 | G2/G6 |
| CV-31 | Retained snapshots preserve TypeStore provenance and IDs across failed/successful candidate updates; candidate interning aborts cleanly and ordinary updates share backing without whole-store copies | 03,11,12 | G3/G8 |

### Test-only failure seam

Provide deterministic fault points (private feature/test harness as appropriate):

```rust
pub(crate) enum SemanticFaultPoint {
    AfterRevisionBegin,
    AfterStructuralShards,
    AfterQueryRetirement,
    AfterSourceFingerprints,
    AfterReadyQueryPublish,
    AfterStaticValueMutation,
    BeforeCommit,
}
```

**Use established QueryOutcome failures, not injected panics**, for the primary transaction tests. Inject panics only if the repository explicitly supports unwind recovery as a contractual behavior, which is **not** required by P1. No sleeps/races; use barriers/epoch fixtures for LSP order tests. The injector must not depend on `cfg(test)` alone when exercised from integration test binaries; use the existing testing convention.

### Differential oracle

For final module set `W`:

```text
cold := fresh_session.try_update(W)
incremental := session(
    publish(W1),
    fail(W2),
    fail(W3),
    publish(W)
)

compare:
  declarations, resolved types, callable signatures,
  callable analyses and formal facts, hierarchy,
  static product representations, diagnostics,
  source index, imported/exported products,
  conformance products, identity/dependency evidence.
```

Compare **semantic contents under canonical stable identity rules**; do not equate raw interner allocation indices or incidental map traversal order across independent stores.

---

## 8. Task contracts

### T0 — Baseline, lifecycle registration, and incident evidence

**Purpose:** Pin live HEAD, record baseline classifications, create checkpoint ledger if absent, and explicitly retain audit evidence without pretending current suite is clean.

**Preconditions:** User-authorized worktree and branch; read repository guidance; no unrelated dirty-tree reset.

**Read:** `AGENTS.md`, `rust-toolchain.toml`, workflow schema, audit INC-007/011, checkpoint conventions, relevant preexisting baseline INC-001.\
**Modify/create:** only checkpoint/plan lifecycle records and baseline evidence.\
**Tests:** No broad tests here; collect existing narrow baseline evidence and toolchain info; run targeted `cargo check` if the checkout is already prepared and applicable.

**Steps:**
1. Record `HEAD`, `origin/main`, dirty files, toolchain override vs pin; **do not assume** the developer machine's directory override matches `rust-toolchain.toml`.
2. Reconcile planned program ID with existing directory names; create/update one shared checkpoint `TYPEAUD001.C1-CHECKPOINT.md` only if not already present.
3. Record known INC-001 baseline failures (7 parser/AST test failures from earlier audit) as **baseline**, not new P1 regressions; recheck if already fixed.
4. Record incident provenance and complete acceptance map. If INC-011 document is not checked in, preserve its ID/description and explicitly record the missing source artifact until supplied.
5. Freeze expected affected crates and tests. No implementation code changes yet.

**Acceptance:** Checkout provenance recorded, lifecycle status `IN_PROGRESS / NOT_STARTED / UNVERIFIED`, baseline classified, no unrelated files touched.\
**STOP:** Unexpected uncommitted P1 implementation or incompatible live authority contract; consult before overwriting.

### T1 — Complete mutable-state and caller census (blocking design gate)

**Purpose:** Make transaction feasibility explicit before modifying production logic.

**Read:** `session.rs:~574–8555`, DB mutators and dependencies, `workspace.rs`, module-session source commits, compiler/LSP paths, `TypeStore`/static-values clone/identity rules.\
**Modify:** No semantic source changes. Produce a brief design appendix/section in checkpoint or plan amendment if the census reveals required mechanical details; no independent competing state document.

**Required work:**
1. Inventory **every** mutation within `update_with_budget_and_cancel_and_delta`: `self.*` assignments, `Arc::make_mut`, `self.db` mutators, calls passing mutable references into query/static evaluators, `take`, `drain`, moves, caches, and metrics.
2. Classify writes: (A) logically committed publication root, (B) candidate query cache/index, (C) harmless append-only allocation proven safe, (D) instrumentational attempts, (E) source/module state owned outside the semantic transaction.
3. For each state, specify candidate isolation, journal undo, or postfailure invalidation *with proof of cold-equivalence*. Unjustified "safe because old snapshot is immutable" is not sufficient.
4. Audit `scheduler_mut` and any `db_mut` access for mutation escape hatches; determine how transactions intercept them.
5. Enumerate every `analyze_workspace`, `analyze_single_module`, `.update`, `update_module_workspace`, `apply_module_mutations`, and `compile_analyzed` call in production/tests; classify strict vs recovered vs source-tolerant.
6. Verify pending-delta representation suffices for remove/re-add/identity change and zero-mutation retry; if not, specify the smallest ledger augmentation.
7. Choose the DB strategy from the write-set/cost evidence. Use a per-key undo journal for SemanticDb; capture query state/product/LKG, module-key ownership, dependency adjacency, revision metadata, and scheduler changes before first mutation. Do not assume whole-DB cloning is acceptable.
8. Apply amendment A1 to TypeStore, TypeLambdaArena, and StaticValueStore: candidate roots structurally share immutable committed storage, preserve the session store ID and all existing numeric IDs, and version mutable provenance per root. Identify every write surface and the narrowest persistent backing representation that supports the current APIs.
9. Define exact semantic commit linearization and LSP visible-publication boundary.

**Tests now:** None; this is a pre-implementation design gate.\
**Acceptance:** 100% of mutable state mapped to a recovery/commit policy; all consumer signatures and test-harness adapters accounted for; DB journal selected with explicit write/cost evidence; persistent interner-root ownership and provenance behavior specified with a structural-sharing discriminator.\
**STOP:** An unjournalable indirect write, unknown TypeStore identity behavior, or a module/semantic ownership contract conflict: raise consultation with traces and alternatives.

### T2 — Build RED failure-injection oracles and publication authority tests

**Purpose:** Establish falsifiable INC-007/011 regressions before architectural edits.

**Read:** Semantic test harness, `type_store_revisions.rs`, `db_base.rs`, `workspace_indexing.rs`.\
**Create:** `phalcom-semantic/tests/semantic/incremental/publication_transaction.rs` (or approved harness-local equivalent), test-only hook(s) in session/DB.\
**Modify:** test registry and minimal testing access only.

**Required work:**
1. Implement fault seam at `AfterRevisionBegin`, `AfterStructuralShards`, `AfterQueryRetirement`, `AfterSourceFingerprints`, `AfterReadyQueryPublish`, `AfterStaticValueMutation`, `BeforeCommit`.
2. Exercise original public `update`/whole-workspace behavior under a deterministic fault to establish old false-success where reproducible; record RED evidence *before* migrating the API.
3. Exercise original infallible `update_module_workspace` or associated batch API across `G1 successful -> G2 fault -> G3 update` to expose lost pending delta where practical; if direct private fault cannot be injected without a minimal source seam, record structural RED evidence first.
4. Write future Green assertions: no published candidate on failure, source generation unchanged for old snapshot, uncommitted delta retained, correct error classification.
5. Make injection *per-session* and deterministic; no mutable global shared tests that bleed across concurrent test cases.

**Tests now:** Only new discriminating tests by exact name filter. Verify `--list` returns nonzero selection.\
**Acceptance:** Precise test oracle covering CV-01, CV-03, CV-04, CV-10, CV-12 is demonstrably discriminating or documented as requiring the production seam first.\
**STOP:** Test fixture can only reproduce defect by violating public API contracts or patching normative semantics.

### T3 — Introduce explicit authoritative results and recovery separation

**Purpose:** Make failure visible to every semantic authority caller.

**Read:** `session.rs:~811–980`, `workspace.rs`, `snapshot.rs`, `db/state.rs`, `lib.rs`.\
**Modify:** `phalcom-semantic/src/{session.rs,workspace.rs,lib.rs}`, narrow `db/state.rs`/`snapshot.rs` if needed.\
**Tests:** Semantic public API tests and failure-injection cases CV-01,02,03,30.

**Implementation:**
1. Add lossless failure type mapping from `QueryOutcome<()>` to `SemanticUpdateFailure`; no silent `.to_string()` in the canonical semantic API.
2. Convert `SemanticWorkspaceSession::update` and its private fallback helper into a strict `Result` (or replace with a strict canonical `try_update` and remove/rename the old authoritative symbol).
3. Convert `analyze_workspace` and `analyze_single_module` to `Result`; do not create `Complete` synthetic failure snapshot.
4. Keep a separately named `recovery_view(...)`/`last_published_snapshot()` for presentation, optionally carrying original generation+failure.
5. Ensure `SnapshotStatus::Partial` still describes successful partial publication, not operational failure.
6. Keep old `QueryOutcome<()>` strict callers operational through lossless wrappers while migrating production; do not add a second independent semantic engine.
7. Record expected temporary compiler/test compile breakages for T7/T8 migration; do not permanently patch callers by `.unwrap_or_else(|_| empty_snapshot())`.

**Tests now:** Narrow `phalcom-semantic` tests.\
**Acceptance:** Failure cannot return ordinary `SemanticAnalysis`/`SemanticWorkspaceUpdate`; error classification exact.\
**STOP:** An external API requires infallible success-shaped analysis and cannot accept typed failure; consult on compatibility contract.

### T4 — Make SemanticDb mutations transactional

**Purpose:** Protect query revisions, products, LKG, keys, scheduler, and dependencies from unsuccessful workspace attempts.

**Read:** `db/{mod.rs,state.rs,dependency.rs,query.rs,scheduler.rs}`, T1 census.\
**Modify:** `phalcom-semantic/src/db/{mod.rs,dependency.rs,scheduler.rs,...}` only as needed; minimal transaction coordinator in semantic crate.\
**Tests:** Internal/isolated DB transaction tests in `db_base.rs` or an internal `#[cfg(test)]` module; CV-04,06,08,10,27.

**Implementation shape:**
1. Add an internal `SemanticDbTransaction` (or equivalent candidate view) that begins with a base committed revision.
2. Ensure all DB writes required by the semantic update are transaction-aware. If using undo journal, capture first-write old `QueryState`, product, LKG, `query_keys_by_module`, forward/reverse index adjacency, revision and recorded current-revision sets. If using overlay, candidate reads must see candidate edges/products and committed readers must not.
3. Support `commit` and `abort` without leaving `Computing`, candidate `Ready`, retired, or stale validation state in committed authority.
4. Preserve query-level product-stability optimization and root-vs-reverse invalidation behavior; do not change which semantic products should be recomputed.
5. Do not prematurely publish candidate LKG. Its staging follows the transaction.
6. Cover direct mutating scheduler/metrics paths found in T1. Metrics may report attempted computations, but committed counters must not be misleading.
7. Use failpoints after cache publication and query retirement; compare DB state/reuse behavior before, after failed attempt, and after successful retry.

**Forbidden:** Cloning `SemanticDb` wholesale for each successful edit, only rolling back `last_snapshot`, clearing the entire DB after every edit, or silently accepting phantom query `Ready` states.

**Tests now:** DB direct tests + semantic short-chain reproducer.\
**Acceptance:** DB rollback/overlay isolation is complete; CV-04/06/08/10/27 pass; old query dependencies remain correct.\
**STOP:** Mutators bypass the transaction or restoring edges/product references cannot preserve identity: consult immediately.

### T5 — Stage session-owned semantic roots and interned values

**Purpose:** Eliminate source fingerprint/shard/static-value/derived-root leaks across failure.

**Read:** `session.rs` full mutation census, `semantic_shard.rs`, `static_value/store.rs`, `types/store.rs`.\
**Modify:** `session.rs`, `types/store.rs`, `types/type_lambda.rs`, `static_value/store.rs`, and optional private transaction helper module.\
**Tests:** CV-05,07,09,11,16,31.

**Implementation shape:**
1. Encapsulate candidate source fingerprints, semantic structure shards, field lifecycle roots, declaration and generic dependency graphs, alias maps, diagnostic contributions, and source indexes in staging or journal-backed restore.
2. Convert early `self.semantic_structure_shards = ...` and `self.sources = ...` / `source_fingerprints` writes into candidate publication or journal-tracked writes. Avoid refactoring all semantic inference logic into a duplicate solver.
3. Implement amendment A1: make candidate TypeStore, nested TypeLambdaArena, and StaticValueStore roots structurally share their committed backing data. Candidate insertions and updates, including `TypeParameterData.source` provenance refreshes, must not mutate a retained snapshot root. Preserve store IDs and all committed numeric TypeIds/other interner IDs. Do not deep-clone whole arenas on ordinary `Arc::make_mut` paths.
4. Record derived roots written late (`last_snapshot`, `last_known_good`, aliases, diagnostics, dependency maps), and ensure they are applied at a single commit.
5. Audit `requested_deep_modules.take` so failure does not consume intent; do not treat deep frontier as disposable local state.
6. Ensure drop/abort path performs restoration in **all** `Err` branches including budget and blocked outcomes; avoid relying on manual rollback at a few return sites.

**Tests now:** Phase-specific fault regressions; no broad suite until candidate implementation is coherent.\
**Acceptance:** Session roots are either committed together or unaffected; CV-05/07/09/11/31 pass; retained snapshots preserve their source provenance across candidate failure and later success; committed interner IDs remain stable and shared backing is demonstrated.\
**STOP:** Candidate TypeStore changes make canonical identity or memory resource obligations impossible without broader design.

### T6 — Durable semantic delta and retry lifecycle

**Purpose:** Eliminate INC-011 and enforce latest-source catch-up.

**Read:** `session.rs:~755–905`, `WorkspaceModuleUpdate`/`WorkspaceModuleSession` implementation, LSP Local-mode frontier.\
**Modify:** `session.rs` pending ledger/strict module entry points, minor module interface adapters if mechanically necessary and authorized.\
**Tests:** CV-12–15,26–28.

**Implementation shape:**
1. Remove pre-success destructive `take` of pending delta. Make merge pure or guard-scoped.
2. Make **all** module update APIs strict. For mixed module+semantic result use a typed aggregate error while preserving original module diagnostics.
3. Accumulate every accepted update over the last successful semantic generation. Reconcile remove/re-add, reidentification, changed linked edges, overlay/revision replacement, and graph changes using actual latest state.
4. Preserve deep-analysis roots through failed publication and merge them with newly requested roots; do not accidentally widen every successful Local scan to global deep solve.
5. Add a deterministic `retry_pending_semantic_update` path or prove that empty mutation batch reaches the same strict retry semantics.
6. Clear pending work only after semantic commit. On error, do not roll back accepted module state.
7. Tie successful publication generation to the actual committed module session generation.

**Tests now:** One focused module lifecycle chain; no parser or unrelated broad tests.\
**Acceptance:** Zero lost delta after repeated failures; the next successful snapshot is cold-equivalent to latest accepted sources; CV-12–15/26–28 pass.\
**STOP:** Representing correct module lifecycle changes requires rewriting `phalcom-modules` publication identity model.

### T7 — Integrate the single semantic commit coordinator

**Purpose:** Join DB, session roots, snapshot construction, and pending ledger into one unambiguous commit.

**Read:** `session.rs:~973–8555`, T4/T5/T6 interfaces, snapshot construction and `SemanticPublicationEffects`.\
**Modify:** `session.rs` and private transaction machinery only.\
**Tests:** CV-01–17,30 after focused implementation; phase-injection suite.

**Implementation shape:**
1. Establish candidate creation and a guard which defaults to abort on any early return.
2. Require a validation phase with coherent generation, query products, dependencies, diagnostics, source index, and static-value roots.
3. Establish exactly one logical commit: install all successful DB/session roots, `last_snapshot`, LKG and derived aliases, and clear pending work.
4. Preserve prior snapshot on every error; preserve failed generation in `SemanticUpdateFailure`.
5. Emit publication effects/stats for *committed* publication only. Preserve metrics for attempted work separately if necessary.
6. Ensure cancellation is checked immediately before commit and the commit is indivisible to readers; distinguish later LSP epoch discard.
7. Verify that no additional mutation occurs after the commit that can fail and leave partially committed state.
8. Prove that a successful snapshot's `generation`, `sources`, `id/store`, `source_index`, and retained roots are internally consistent.

**Tests now:** Full new semantic transaction suite plus existing canceled-update test.\
**Acceptance:** G2–G5 can pass without compiler/LSP migration; no incomplete candidate accessible through authoritative DB or snapshot API.\
**STOP:** Atomic commit cannot be specified without changing the public snapshot/TypeId lifetime authority.

### T8 — Migrate compiler and canonical Universe consumers

**Purpose:** Ensure every executable entry requires strict semantic publication.

**Read:** `phalcom-core/src/modules/{compile.rs,compile_session.rs,canonical_universe.rs}`, `workspace.rs` result API.\
**Modify:** Narrow affected compiler paths and associated integration tests.\
**Tests:** CV-18,19,30; representative valid compilation parity.

**Implementation shape:**
1. Convert inline analysis at `compile.rs:~618` to use fallible `analyze_workspace` and map failure to `ProgramCompileError::SemanticUpdate` or typed equivalent.
2. Convert standalone analysis at `:~726` likewise.
3. Preserve project `discover_and_analyze` strict path and persistent `CompileSession` behavior; avoid adding double analysis.
4. Migrate canonical Universe bootstrap `catch_unwind` to distinguish *panic* from **non-panicking semantic failure** and map each into the correct `CanonicalUniverseBuildError` category.
5. Retain bootstrap's intentional special handling of source diagnostics but **never** accept an unsuccessful semantic update.
6. Ensure `AnalyzedProgram` construction only follows strict success; keep downstream `has_errors` as a source-diagnostic guard, not a surrogate publication-success check.
7. Add fault-capable compiler tests for all affected entry families, including no emitted `CompiledProgram` on injected semantic failure; retain normal valid-output tests.

**Tests now:** Focused compiler module tests; only widen after pass.\
**Acceptance:** G6 and CV-18/19/30 pass.\
**STOP:** Compiler requires a policy change for legitimate `Partial` semantic snapshots; consult with concrete executable-product evidence.

### T9 — Migrate LSP scans and enforce stale recovery provenance

**Purpose:** Remove success-shaped semantic fallback from scan paths while preserving latest-wins behavior and interactive responsiveness.

**Read:** `phalcom-lsp/src/analysis_service.rs:~750–903,~1080–1150`, `request_context.rs`, existing LSP tests.\
**Modify:** LSP scan publication/error handling, only necessary request/diagnostic provenance code.\
**Tests:** CV-20–23,29 and scan/interactive parity.

**Implementation shape:**
1. Change both Full and Local `apply_module_mutations` scanner calls to strict-return publication. On semantic `Err`, do not call `publish_snapshot`.
2. Keep the interactive cancellable strict path and existing worker epoch check. No independent LSP semantic cache or second authority.
3. Preserve partial successful Local-mode publication and its restricted deep-analysis frontier.
4. Emit explicit error/stale events on failures; do not silently drop scan errors or replace diagnostics with an empty set.
5. Associate recovery snapshots with **their original published generation and source revision**; prohibit request context from treating recovery as current-generation formal analysis.
6. Test failed scan → interactive edit → successful retry, including stale request range and diagnostic provenance; test semantic commit followed by request-visible epoch supersession separately.
7. Preserve ability to show prior information under policy while forbidding forged fresh analysis.

**Tests now:** Focused `phalcom-lsp` tests; no complete editor/extension suite during BUILD.\
**Acceptance:** G7; CV-20–23/29 pass.\
**STOP:** Correct recovery requires changing source overlay authority or the public LSP protocol contract outside P1.

### T10 — Differential recovery and performance stabilization

**Purpose:** Prove correctness after failure and preserve normal incremental performance.

**Read:** semantic incremental test helpers, module lifecycle integration tests, `recomputation_metrics.rs`, DB performance counters.\
**Modify:** Tests, instrumentation, and narrowly justified performance corrections within P1 machinery.\
**Tests:** CV-16,17,24,25,28,31; gates G5/G8.

**Required tests:**
1. Cold vs incremental after repeated faults (provider/header/body/alias changes).
2. Removed and reintroduced module identity; import graph changes; static values and formal proof products.
3. Deterministic semantic fingerprint equality (subject to stable identity/normalization rules).
4. Normal successful unchanged/body-only edits preserve documented reuse counts; no whole-workspace cache invalidation introduced to gain correctness.
5. Benchmark/metric comparison with pinned baseline for no-op and focused edits. Record actual interner backing-sharing or copied-node/arena evidence and query recomputation; `type_store_snapshot_deep_copies = 0` alone does not demonstrate structural sharing. **Do not invent a pass threshold without baseline.**
6. Require nonzero selected tests; avoid rerunning a failing unchanged case.

**Tests now:** Focused semantic + owning compiler/LSP suites; broad workspace checks only if relevant certification requires them.\
**Acceptance:** G5/G8 pass with evidence; performance has no unexplained severe regression.\
**STOP:** Transaction cannot meet reuse budget without alternate architecture; consult rather than weakening rollback correctness.

### T11 — Documentation, incident closure, walkthrough, and handoff

**Purpose:** Record durable truth and unblock P2–P5 of remediation program.

**Read:** lifecycle convention and shared checkpoint, test evidence, implementation diff.\
**Modify/create:** shared checkpoint, approved incident dispositions, `TYPEAUD001.C1.P1-walkthrough.md`, `TYPEAUD001.C1.P1-handoff.md`.\
**Tests:** None unless acceptance evidence is missing or new code changes require the smallest affected gate.

**Required deliverables:**
1. Full source changes map and public API migration map.
2. Exact transaction mechanism, linearization point, rollback/no-leak proof, and cost analysis.
3. Regession test table CV-01–30 with actual command, selected count, pass/fail; classify unexecutable pre-fix reproducer honestly.
4. Compiler/LSP integration evidence and source/semantic generation policy.
5. Deferred baseline failures with classifications, not disguised as fixed P1 failures.
6. INC-007 and INC-011 each marked fixed only with corresponding gates.
7. Clean and accurate lifecycle metadata; walkthrough/handoff with latest revision, residual risks, and P2 takeover contracts.

**Acceptance:** G9 complete. Plan may be `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` **only if** all mandatory focused gates passed; `RELEASE_COMPLETE` requires separate broad release certification and must not be inferred from source changes.

---

## 9. Verification execution ladder and commands

### BUILD mode — deliberately narrow

Read `rust-toolchain.toml`; record active toolchain and any directory override. **Do not assume** a generic `cargo +nightly` uses the pinned date.

Run only relevant compile/check feedback after a coherent code boundary:

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo check -p phalcom-core
RUSTFLAGS='' cargo check -p phalcom-lsp
```

These are proposed commands; adjust only if the local repository uses an equivalent toolchain invocation. Do not claim commands ran merely because they appear in the plan.

### Test registration and nonzero selection

New semantic test module should use distinctive `semantic_publication_transaction` names; **first inspect the test harness registration**, then:

```sh
cargo test -p phalcom-semantic --test semantic -- --list | rg 'semantic_publication_transaction|cancelled_update_preserves_last_known_good'
cargo test -p phalcom-semantic --test semantic semantic_publication_transaction -- --nocapture
cargo test -p phalcom-semantic --test semantic cancelled_update_preserves_last_known_good -- --nocapture
```

`rg` names here are the proposed test naming convention; if actual test module naming differs, modify the filter to the registered names but preserve a **nonzero selection assertion**. A test command selecting 0 tests does **not** count as evidence.

### STABILIZE mode — exact affected integrations

```sh
cargo test -p phalcom-semantic --test semantic recomputation_metrics -- --nocapture
cargo test -p phalcom-semantic --test semantic workspace_indexing -- --nocapture
cargo test -p phalcom-core --test core semantic_publication_authority -- --nocapture
cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture
```

`semantic_publication_authority` is a **new proposed filter**, not evidence of an existing test. Verify the test harness and selected counts before acceptance. For any new LSP integration test in another binary, use that binary rather than falsely reporting coverage in `single_world_cutover`.

### CERTIFY mode — only at appropriate gate

```sh
RUSTFLAGS='' cargo check --workspace --all-targets
```

Broader affected-crate tests may be run for final stabilization. Full workspace tests/clippy and unrelated language corpora are **deferred to checkpoint/release certification** unless a specific P1 failure requires them. Baseline INC-001 parser/AST failures must remain separately classified.

### Failure taxonomy

- **A:** Definitely introduced by P1 — fix in plan.
- **B:** Probably introduced — inspect/fix in plan.
- **C:** Unclear — one bounded classification pass, document residual if not blocking.
- **D:** Confirmed pre-existing or unrelated — ledger/defer, do not wander into another remediation plan.

Do not weaken/skip assertions to manufacture a green gate. Never rerun unchanged failing tests without a relevant code/fixture change.

---

## 10. Gates and hard acceptance matrix

| Gate | Title | Required CV coverage | Exact exit criteria |
|---|---|---|---|
| G0 | Baseline and scope | — | Toolchain, HEAD, dirty tree, incident ledger, affected calls and baseline classified; no code changes. |
| G1 | Authority API | 01–03,30 | Whole-workspace and module semantic authority APIs cannot conceal failure; valid partial publication preserved. |
| G2 | Fresh failure | 01,02,30 | First analysis failure yields `Err`; no `Complete` synthetic snapshot; distinct failure categories. |
| G3 | Transactional rollback | 03–11,27,31 | Every defined intermediate fault restores/quarantines candidate DB/session state; published snapshots, store IDs, numeric IDs, and per-root provenance remain safe. |
| G4 | Durable retry | 12–15,26,27 | Pending graph, source edits, module identity and scheduling frontier survive multiple failures and empty-batch retry. |
| G5 | Cold equivalence | 16,17,28 | Last successful incremental snapshot equals fresh analysis on authoritative projections/fingerprints. |
| G6 | Compiler strictness | 18,19,30 | Inline, standalone, project, persistent compiler, and Universe bootstrap propagate update failure; no compiled candidate. |
| G7 | LSP cutover | 20–23,29 | Interactive and scan strict; no failed/stale snapshot made request-visible as fresh; legitimate partial remains. |
| G8 | Resource reuse | 24,25,31 | Normal success preserves query precision; persistent interner roots structurally share committed arenas and do not copy whole stores; actual sharing/copy evidence and regression metrics recorded. |
| G9 | Lifecycle closure | All | Walkthrough/handoff, incident dispositions, checkpoint, exact commands and test counts, known residuals recorded. |

**Promotion rule:** T11/G9 must not be marked complete while any G1–G8 mandatory coverage remains unverified, unless an explicit consultation changes the plan and preserves the original invariant.

**Acceptance oracle for the central defect:**

```text
G1 successful at source generation 1;
G2 edit provider exports; fault after DB mutation;
G3 edit consumer; fault after fingerprint/state mutation;
G4 remove/reintroduce a module; fault before commit;
G5 retry with no additional edits -> strict successful semantic publication.

Require:
  published generations before G5 remained G1;
  latest accepted module state remained intact throughout;
  all pending changes contributed to G5;
  no old query/cache fingerprint or dependency created stale reuse;
  G5 canonical semantic products == fresh analysis of same final source;
  compiler and LSP never treated G2/G3/G4 as successful publication.
```

---

## 11. Performance and resource requirements

**Normal successful path:** retain exact incremental query validation and product stability, partial snapshot sharing, immutable `Arc` snapshot publication, and stable TypeStore identity. The existing `type_store_snapshot_deep_copies = 0` metric does not observe `Arc::make_mut` cloning; acceptance requires evidence from the actual persistent backing representation or a counter that measures copied arena/node data. Do not introduce unconditional full-store copies merely to simplify transaction rollback.

Measure before/after for:

| Scenario | Evidence required |
|---|---|
| No-op update | Query recomputation/revalidation and elapsed allocation/copy behavior. |
| Single-body edit | Changed callable recomputed; unaffected callables remain reusable. |
| Generic/header edit | Correct dependent invalidation, not blanket workspace recompute. |
| Provider export edit | Required reverse-dependency frontier only. |
| Failure then retry | Additional correctness work acceptable on exceptional path; record it explicitly. |
| Repeated LSP edits | No unbounded pending delta or candidate journal growth. |

No invented absolute latency requirement. Explain any measurable regression with source ownership evidence; an architecture that forces routine full semantic recompute after every normal edit fails P1 even if correctness tests pass.

---

## 12. Bounded debugging and consultation

### Mechanical debugging

At most three coherent correction cycles with focused recheck:

```text
inspect -> root cause -> one local correction -> smallest discriminating test
```

### Semantic debugging

For the **same semantic failure**, one serious corrective attempt after recording:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

If unchanged failure persists: **STOP AND CONSULT**. Never silence a failed query, weaken generation validation, or clear all caches on the normal path as a workaround.

### Mandatory consultation triggers

1. The transaction cannot isolate `SemanticDb` mutations without changing DB identity or cross-crate query contracts.
2. Interner or static-value staging changes TypeId/store identity or snapshot lifetime.
3. Module/source session would need to roll back accepted source changes.
4. Existing `Partial` publication semantics conflict with strict success status.
5. Scanner or compiler requires swallowing failures to remain operational.
6. A fault injector must be exposed to arbitrary production callers.
7. Cold/incremental equivalence fails after one serious root-cause correction.
8. Two candidate transaction implementations have materially different correctness, concurrency, or performance effects.
9. A new hidden source of formal semantic authority would be introduced.
10. The public API change cannot be migrated without a genuinely incompatible external contract.
11. The proposed audit incident root cause is disproved by source/runtime evidence.
12. Required tests cannot establish the authority invariant due to missing test seam or nondeterminism.

**Consultation packet:** ID, file/line evidence, baseline/head, observed failing command and output, normative requirement, competing options, recommended architectural choice, minimal red test, proposed gate amendment, and source-change stop point. Record the disposition in the checkpoint consultation ledger before continuing.

---

## 13. Checkpoint, walkthrough, and handoff requirements

### Shared checkpoint

Use only one `TYPEAUD001.C1-CHECKPOINT.md` for program-wide durable truth. At plan start record P1 as active and audit incidents. During implementation update for stable public interfaces, transaction choice, consultation, new material invariant, or completed gate. At close record P1 disposition and P2 takeover boundary. Do not create an alternative privately maintained plan status document.

### Walkthrough

`TYPEAUD001.C1.P1-walkthrough.md` must contain:

- Implemented interface signatures and owner list;
- Exact authority separation, transaction state write-set, chosen journal/overlay mechanism, and commit point;
- Pending delta lifecycle, empty retry, accepted module source contract;
- Compiler and LSP caller migration outcomes;
- Fault injection and cold/incremental test results (actual selected/pass counts);
- Performance before/after evidence and known limitations;
- Deviations and consultation decisions;
- Incident closeout evidence and full diff/revision.

### Handoff

`TYPEAUD001.C1.P1-handoff.md` must contain:

- Final HEAD/branch, clean/dirty state, gates passed/deferred;
- Stable API/transaction contracts inherited by P2–P5;
- Exact tests and next commands;
- Baseline exclusions, follow-up incidents/risks;
- Items the next implementer should not redesign;
- Point to checkpoint as durable shared authority.

**Completion truth:** `IMPLEMENTED` means code exists; `FOCUSED_TESTED` requires recorded executed focused gates; `RELEASE_COMPLETE` requires the broader release evidence, not merely compilation or a clean tree.

---

## 14. Final architect self-review checklist (for execution handoff)

- [ ] Every API returning authoritative semantics now has an observable failure path.
- [ ] No first-generation failure can produce a synthetic success snapshot.
- [ ] Snapshot status distinguishes successfully `Complete`/`Partial`, not failure-as-success.
- [ ] Every DB mutation path is journaled/overlaid, including LKG and dependency indexes.
- [ ] All session-owned derived roots are staged, restored, or proven safely append-only.
- [ ] TypeStore and static-value identities remain valid across failure and recovery.
- [ ] Pending delta and deep frontier survive repeated failures; empty-batch retry works.
- [ ] Module/source accepted state is *not* rolled back on semantic cancellation.
- [ ] Compiler inline, standalone, package, persistent fixture, and Universe bootstrap audited.
- [ ] LSP interactive, Full scan, Local scan, diagnostics, source revision, epoch audited.
- [ ] Cold/incremental equivalence includes provider/consumer and module identity churn.
- [ ] Normal successful incremental reuse preserved; no unconditional DB/interner deep copy.
- [ ] Tests select nonzero cases and log exact outcomes.
- [ ] INC-007/011 are individually closed only after their gates, not after local code edits.
- [ ] All lifecycle metadata, walkthrough, handoff, and checkpoint changes are truthful.

## 15. Implementation start instruction

**Start with T0 and T1 only.** Verify the live repository and complete the state write-set/caller census. Then add discriminating failures in T2 before touching the primary implementation logic. Complete T3–T7 as one coherent transactional semantics change, migrate compiler/LSP in T8–T9, certify only required gates in T10, and close lifecycle in T11.

**Do not substitute local fallback fixes for the required transaction architecture.** The P1 acceptance claim is not that `update()` returns `Result` or that `pending_semantic_delta` is cloned; it is that failed semantic attempts **cannot publish or poison formal authority**, while the next successful incremental state remains correct and efficiently reusable.

---

### Planning provenance

Prepared from read-only inspection of connected GitHub repository `aureat/phalcom-lang` at `b00d75b3ac9f50d37cf9abfddfde00917e099031`, the previous P1 requirements analysis, the type-system audit incident findings, repository workflow schema, and directly inspected session/DB/compile/LSP source. **No repository code or tests were modified or run as part of plan authoring.** Line numbers are revision-specific and must be reconfirmed against the implementer's checkout.

## 16. Execution status — updated 2026-10-13

This section records implementation evidence and remaining acceptance work. It does not amend the FIXED architecture or waive the promotion rule in §10.

```yaml
status: IN_PROGRESS
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
branch: main
head: b00d75b3ac9f50d37cf9abfddfde00917e099031
origin_main: b00d75b3ac9f50d37cf9abfddfde00917e099031
working_tree: dirty; task changes uncommitted; unrelated audit/COLL006 files preserved
```

### Implemented source changes

| Area | Implementation |
|---|---|
| Semantic result boundary | Added typed `SemanticUpdateFailure` and `SemanticWorkspaceUpdateError`; `SemanticWorkspaceSession::{update,update_module_workspace}`, `analyze_workspace`, and `analyze_single_module` return `Result`. The success-shaped stale/synthetic fallback was removed. |
| SemanticDb transaction | Added per-key undo journal for query state, current products, LKG products, module query ownership, dependency forward/reverse rows, revision metadata, and scheduler state. Rollback restores these roots; metrics remain attempt telemetry. |
| Session candidate roots | Candidate shards, TypeStore, and StaticValueStore roots are restored on failure; source/fingerprint and derived publication roots install only at the final successful commit. Cancellation is checked at the commit boundary. |
| Persistent interners | Converted TypeStore, nested TypeLambdaArena, and StaticValueStore arenas/indexes to persistent `im::Vector`/`im::HashMap` backing. Stable store/numeric IDs and snapshot-local TypeParameter provenance are covered by sharing/provenance tests. |
| Retry ledger | Pending module deltas merge without destructive `take`, are reconciled against current module membership, and clear only after semantic success. Failed deep-analysis requests remain queued. |
| Compiler and Universe | Inline/standalone analysis propagates typed semantic errors; Universe bootstrap distinguishes semantic failure from panic. Project and persistent-session routes remain strict. |
| LSP | Interactive failure is surfaced; Full/Local scan errors emit an error and never call request-visible `publish_snapshot`. Epoch supersession remains separately checked. |

### Task and gate disposition

| Task/gate | Disposition | Evidence/remaining work |
|---|---|---|
| T0–T2 | COMPLETE | Baseline, census, A1 amendment, and pre-fix RED oracles were already recorded in the checkpoint. |
| T3–T7 | IMPLEMENTED / FOCUSED_TESTED | Strict result APIs, DB journal, persistent roots, final commit boundary, and retry behavior pass focused tests below. |
| T8 | IMPLEMENTED / PARTIAL | Compiler and Universe paths are migrated; typed mapping tests pass and core/LSP test targets compile. Failure injection through every compiler entry family remains unverified. |
| T9 | IMPLEMENTED / PARTIAL | Scan failure classification and epoch supersession tests pass. End-to-end stale recovery source/range provenance remains unverified. |
| T10 | PARTIAL / BASELINE_BLOCKED | Cold retry parity, workspace indexing, and persistent-root checks pass. Reuse suites have exact baseline failures; pinned benchmark comparison is incomplete. |
| G0–G5 | FOCUSED PASS | Authority, fresh failure, transactional rollback, durable retries, and retry-to-cold equivalence have selected nonzero focused evidence. |
| G6 | PARTIAL | Typed compiler error mapping and target compilation pass; all-entry failure injection/no-artifact coverage remains. |
| G7 | PARTIAL | Failed scan classification, scan publication success, and single-world cutover pass; stale recovery provenance/range integration remains. |
| G8 | BASELINE_BLOCKED | Structural sharing and workspace-index tests pass; recomputation/type-store baselines reproduce on starting revision. Full before/after benchmark evidence remains. |
| G9/T11 | NOT COMPLETE | Do not close INC-007/011 or mark the plan complete until G6–G8 gaps are resolved. Completion walkthrough/handoff records are intentionally not labeled complete. |

### Focused verification actually run

All Cargo commands below used `RUSTUP_TOOLCHAIN=nightly-2026-07-10`, `RUSTFLAGS=''`, and an isolated target directory under `/tmp` unless noted.

| Command/filter | Result |
|---|---|
| `cargo test -p phalcom-semantic --lib session::publication_tests -- --nocapture` | 13 selected, 13 passed, 171 filtered. Includes rollback at all test failpoints, stale LKG retention, typed failure categories, repeated edits, remove/re-add, empty retry, deep-frontier retry, budget exhaustion retry, and cold projection parity. |
| `cargo test -p phalcom-semantic --lib db::transaction_tests -- --nocapture` | 2 selected, 2 passed. |
| `cargo test -p phalcom-semantic --lib persistent_root -- --nocapture` | 3 selected, 3 passed; persistent interner roots retain shared backing and isolate candidate mutation. |
| `cargo test -p phalcom-semantic --test semantic workspace_indexing -- --nocapture` | 19 selected, 19 passed, 2,141 filtered; includes cold parity, import retargeting, structural sharing, and delta publication cases. |
| `cargo test -p phalcom-core --lib compiler_error_keeps_semantic_publication_failure_typed -- --nocapture` | 1 selected, 1 passed. |
| `cargo test -p phalcom-core --lib semantic_failure -- --nocapture` | 1 selected, 1 passed; semantic failure mapping remains distinct from panic. |
| `cargo test -p phalcom-lsp --lib analysis_service::scan_publication_tests -- --nocapture` | 2 selected, 2 passed; failed scans are not current, superseded commits are stale. |
| `cargo test -p phalcom-lsp --test scan_publication -- --nocapture` | 1 selected, 1 passed. |
| `cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture` | 2 selected, 2 passed. |
| `cargo test -p phalcom-semantic --test semantic --no-run` | Test target compiled. |
| `cargo test -p phalcom-core -p phalcom-lsp --no-run` | All selected core and LSP test targets compiled. |

### Baseline failures and residual verification

These failures were reproduced on the exact starting revision `b00d75b3` using a detached baseline checkout and are classified D under §9. They are not claimed as fixed or caused by this implementation.

| Target | Current result | Baseline result |
|---|---|---|
| `cargo test -p phalcom-semantic --test semantic recomputation_metrics -- --nocapture` | 8 passed, 5 failed, 2,147 filtered. Failures: `a6_cross_module_alias_relowers_when_provider_shell_changes`, `a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products`, `a6_high_fanout_unused_provider_body_edit_has_no_external_semantic_recompute`, `pa9_large_hierarchy_edit_isolates_unrelated_formal_products`, `unused_public_export_does_not_recompute_unrelated_consumer_body`. | Same five failures and reported values. |
| `cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions -- --nocapture` | 19 passed, 3 failed. Failures: `publication_effects_distinguish_initial_graph_build_from_body_edit`, `snapshot_metadata_catalog_excludes_stale_static_case_history`, `stable_type_store_id_and_snapshots_across_revisions`. | Same three failures. |
| `cargo test -p phalcom-core --test canonical-bootstrap-stack dependency_free_project_compiles_then_bootstraps_and_runs -- --nocapture` | Failed: legacy metadata cannot represent an open Record remainder outside the Symbol key domain; the same run reaches stack overflow. | Same failure on `b00d75b3`. |

The existing `INC-001` parser reproducer also remains a known unrelated baseline. Full workspace tests, workspace build/clippy, broad parser snapshots, and release certification were not run. A rustfmt check over every changed Rust file reported formatting diffs throughout the touched files, including surrounding code; no broad formatter pass was applied. The new `session/publication_tests.rs` module was formatted, and the two added core assertions were wrapped locally. `git diff --check` passed after implementation edits.

### Required continuation before plan completion

1. Add or otherwise establish fault-capable coverage proving inline, standalone, project/persistent, and Universe compiler entry points reject a failed semantic update before producing compiled output.
2. Verify the LSP failure path keeps the prior snapshot's source revision/ranges and cannot expose it as fresh for the failed generation; cover Full and Local scans or document a precise acceptance-equivalent shared path.
3. Complete G8 with baseline-qualified reuse/performance comparison and resolve whether the five metrics failures block P1 or are an accepted documented baseline.
4. Update INC-007/011 dispositions only after their required gates pass; then update checkpoint/program state and create the plan completion walkthrough/handoff.

No plan amendment beyond A1 was required. The user has authorized committing and pushing this P1 implementation on `main`. The implementation remains `IN_PROGRESS / IMPLEMENTED / BASELINE_BLOCKED`; G6–G8 and T11 remain open, and the user-requested delivery does not promote those gates. Unrelated audit and COLL006 drafts are preserved outside the P1 delivery.
