---
id: TYPEAUD001
category: TYPEAUD
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
repository: aureat/phalcom-lang
branch: main
prepared: 2026-10-12
prepared_against_revision: b00d75b3ac9f50d37cf9abfddfde00917e099031
active_checkpoint: TYPEAUD001.C1
next_checkpoint: TYPEAUD001.C1
next_plan: TYPEAUD001.C1.P1
---

# TYPEAUD001 — Type-System Correctness and Audit Remediation

> **Current lifecycle truth:** `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`. P1's transactional publication implementation and strict compiler/LSP migrations are present; G1–G5 have focused evidence, while G6/G7 are partial and G8 has baseline failures reproduced on the starting revision. INC-007/011 remain open pending all mandatory gates. The checkpoint and program `STATUS.md` carry current execution state.

## 1. Program mandate

Remediate source-confirmed type-system audit findings under one coherent architectural authority while separating changes into bounded, independently verifiable implementation plans. The end state is reliable semantic publication, logically correct proof outcomes, faithful generic/Dynamic representations, lossless structural specialization, and sound structural intrinsic operations. Type-system behavior must remain consistent from the semantic database and checker through compiler, metadata, and editor consumers.

**Acceptance principle:** Each incident is closed only after its architectural contract is repaired, discriminating tests execute successfully, affected consumers are audited, and durable verification evidence is recorded. A successful code edit, compilation, or clean working tree is not sufficient.

### Authority and coordination

- Current normative language semantics: `docs/specs/` and ratified accepted decisions. The audit is evidence, not a replacement specification.
- Repository workflow: `AGENTS.md`, `docs/workflow/implementation-record-lifecycle-convention.md`, `docs/workflow/luna-patch-grade-plan-schema.md`, and shared consultation protocol.
- The **only shared C1 execution-state authority**: `C1/TYPEAUD001.C1-CHECKPOINT.md`.
- The program-level roll-up is `STATUS.md`; it must be derived from, and must never contradict, the checkpoint.
- Plan documents express implementation *intent*; walkthroughs, test logs, commits, and the checkpoint provide implementation *evidence*.
- Prior LANG007, SEMA, COLL, COMP, and LSPX implementations retain their original ownership. TYPEAUD001 authorizes bounded corrections to them, not a second semantic authority or wholesale program takeover.

## 2. Program scope

| Program domain | Required outcome | Primary architectural owner |
|---|---|---|
| Semantic publication | Failed queries cannot publish apparently successful snapshots or poison subsequent incremental computation | `phalcom-semantic::session`, `SemanticDb` |
| Canonical relation algebra | Sound union conjunction/disjunction, ordered Tuple relations, and valid generic proof laws | `phalcom-semantic::types::relation` |
| Dynamic generics | `StaticArgument::DynamicType` remains a boundary obligation, neither a fabricated static type nor an automatic nominal mismatch | Type relation and checked consumers |
| Materialization and specialization | Tuple remainders and Record row constraints survive substitution, specialization, and checked materialization | `types::{environment,instantiation,substitution}` |
| Structural intrinsic projections | Open-shape `Map`, `Element`, and `At` behave according to normative structural semantics | `types::projection`, `types::type_member` |
| Certification | Verified cross-layer regressions, cold/incremental parity, baseline reconciliation, incident closure | Shared C1 checkpoint and implementation owners |

### Explicit non-goals

No independent redesign of language syntax, type algebra outside the incident scope, the VM's general execution model, global module architecture, or all compiler diagnostics. Do not reinterpret accepted specs to bless a faulty implementation. Do not treat INC-006 as an open defect: it was cleared by normative specification review.

## 3. C1 checkpoint and plan dependency map

**Checkpoint:** [`TYPEAUD001.C1`](C1/TYPEAUD001.C1-CHECKPOINT.md) — bounded remediation of the 2026-10-12 audit register.

| Plan / gate | Workstream | Audit findings | Lifecycle at program creation |
|---|---|---|---|
| **G0** | Baseline/toolchain evidence and ownership census | INC-001 and all plan prerequisites | P1 baseline and T1 complete; checkpoint-wide G0 remains open |
| **P1** | Semantic publication and transactional updates | INC-007, INC-011 | `IN_PROGRESS / PARTIAL / UNVERIFIED`; T0/T1 complete, A1 adopted, T2 RED established |
| **P2** | Relation algebra and Tuple subtyping | INC-002, INC-003, INC-008, INC-010 | Scope proposed; patch-grade plan not yet authored |
| **P3** | Dynamic generic boundary semantics | INC-009 | Scope proposed; patch-grade plan not yet authored |
| **P4** | Canonical specialization and materialization | INC-004, INC-005 | Scope proposed; patch-grade plan not yet authored |
| **P5** | Structural intrinsic normalization | INC-012, INC-013 | Scope proposed; patch-grade plan not yet authored |
| **G6** | Consolidated certification and incident closeout | All open findings | NOT_RUN; checkpoint-level gate |

```text
            G0 — baseline and authority
             |           |           |
             v           v           v
            P1          P2          P4
        publication   relations  materialization
                         |           |
                         v           v
                        P3          P5
                     Dynamic      intrinsics
             \           |          /
              +----------+---------+
                         |
                         v
                 G6 — certification
```

P1, P2, and P4 may be developed concurrently only when distinct code ownership and a consistent base are established. P3 consumes P2 relation rules; P5 consumes P4 structural reconstruction. Execution order must not be confused with checkpoint completion.

**P1 plan:** [`TYPEAUD001.C1.P1-semantic-publication-luna-patch-grade-implementation-plan.md`](C1/TYPEAUD001.C1.P1-semantic-publication-luna-patch-grade-implementation-plan.md). Implementation is active; source corrections exist and mandatory G6–G8 evidence is still open.

## 4. Audit incident register at formation

There are **11 open source-confirmed semantic findings**, **one baseline verification issue**, and **one cleared candidate**, not 13 open defects.

| ID | Classification | Owning action | Initial state |
|---|---|---|---|
| INC-001 | Workspace/parser-test baseline evidence, including stale AST snapshots | G0 baseline; broader G6 reconciliation | OPEN / UNVERIFIED as remediation |
| INC-002 | Tuple labeled order ignored by relation engine | P2 | OPEN / SOURCE_CONFIRMED |
| INC-003 | Target union early `Blocked` masks later proof | P2 | OPEN / SOURCE_CONFIRMED |
| INC-004 | Tuple remainders lost in materialization | P4 | OPEN / SOURCE_CONFIRMED |
| INC-005 | Record anonymous/captured tails incorrectly specialized | P4 | OPEN / SOURCE_CONFIRMED |
| INC-006 | Finite product lane projection suspected of losing open tails | None | **CLEARED** — not a defect absent new consumer-specific evidence |
| INC-007 | Failed semantic update disguised as successful snapshot | P1 | OPEN / SOURCE_CONFIRMED; executable trigger not tested |
| INC-008 | Source union early `Blocked` masks later refutation | P2 | OPEN / SOURCE_CONFIRMED |
| INC-009 | Dynamic generic argument hard-refuted in applied relation | P3 | OPEN / SOURCE_CONFIRMED; historically identified and revalidated |
| INC-010 | Generic-parameter guard masks universal subtype proofs | P2 | OPEN / SOURCE_CONFIRMED |
| INC-011 | Failed infallible module update loses semantic delta | P1 | OPEN / SOURCE_CONFIRMED; retry reproduction pending |
| INC-012 | Structural `Map` fails open Tuple/Record shape requirements | P5 | OPEN / SOURCE_CONFIRMED |
| INC-013 | Open Tuple `Element`/`At` ignore possible remainders | P5 | OPEN / SOURCE_CONFIRMED |

**Audit provenance:** `docs/audit/type-system-2026-10-12/` where available, plus Slice 2–6 audit handoffs. INC-011–013 may exist as supplied downloadable reports before being added to the live repository. An absent GitHub incident file is not evidence that the finding was closed.

## 5. Program-wide acceptance invariants

1. **Formal authority:** only successfully committed semantic generations authorize downstream compilation.
2. **Failure isolation:** candidate analysis failure cannot corrupt published state or suppress future invalidation.
3. **Logical relation consistency:** `Blocked` cannot improperly short-circuit later decisive union proof/refutation.
4. **Structural identity:** ordered Tuple labels remain ordered, and independent remainders remain independent.
5. **Materialization fidelity:** substitutions preserve domain, structure, and symbolic information.
6. **Dynamic honesty:** runtime Dynamic boundaries cannot be silently treated as static evidence or ordinary `TypeId` arguments.
7. **Symbolic fidelity:** well-formed unresolved type-level operations remain symbolic; no invented concrete result.
8. **Consumer coherence:** semantic facts are authored in the semantic layer and consumed by compiler/LSP, not reconstructed separately.
9. **Resource discipline:** correctness fixes preserve normal incremental reuse and avoid unbounded fallback processing.
10. **Evidence truth:** the checkpoint is not complete until every in-scope plan and certification gate is accounted for.

## 6. Program records and required repository placement

```text
docs/implementation/TYPEAUD001/
  PROGRAM.md
  STATUS.md
  C1/
    TYPEAUD001.C1-CHECKPOINT.md
    TYPEAUD001.C1-GUIDANCE.md
    TYPEAUD001.C1.P1-semantic-publication-luna-patch-grade-implementation-plan.md
    TYPEAUD001.C1.P1-walkthrough.md                 # create on P1 completion
    TYPEAUD001.C1.P1-handoff.md                     # create on P1 completion
    # P2–P5 plans and their walkthroughs/handoffs: create when actually authored/executed
```

The canonical index `docs/implementation/INDEX.md` now registers TYPEAUD001. The active plan, guidance, and checkpoint live under `docs/implementation/TYPEAUD001/C1/`.

## 7. Initial program lifecycle

```yaml
program: TYPEAUD001
checkpoint: TYPEAUD001.C1
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
active_plan: null
latest_completed_plan: null
next_plan: TYPEAUD001.C1.P1
blocking_before_source_edits:
  - Confirm program ID/path and current main checkout.
  - Classify G0 baseline and build the P1 mutable-write-set/caller census.
  - Register P1 as IN_PROGRESS only when implementation actually starts.
```

**Current next action:** Complete P1's remaining compiler failure-route and LSP recovery-provenance evidence, resolve or baseline-qualify G8 reuse results, then close incidents and write the P1 walkthrough/handoff. The P1 implementation delivery does not close these gates. Continue to P2–P5 only after P1 reaches its accepted lifecycle state.
