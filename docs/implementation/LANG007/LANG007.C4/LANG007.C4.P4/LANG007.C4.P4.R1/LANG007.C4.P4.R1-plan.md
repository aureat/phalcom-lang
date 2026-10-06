---
id: LANG007.C4.P4.R1
parent_plan: LANG007.C4.P4
kind: implementation-companion
title: "P4 corrective issues and historical closure record"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
recorded_against_revision: 2d306c13da02fb3b85ee362498cf4c0280de3825
recorded: 2026-10-06
updated: 2026-10-06
consolidated_branch: main
---

# LANG007.C4.P4.R1 — Corrective issues companion

## Purpose and authority

R1 is the companion issue register for corrective work and unresolved findings
recorded during the P4 implementation period. It brings the former
`LANG007.C4.1` implementation notes and audit packets into the P4 record tree
without creating a competing checkpoint. The `LANG007.C4` checkpoint remains
the shared state authority, and the P4 plan remains the authority for P4's
FieldKey and computed-key acceptance work.

The files retain their original findings and evidence. Their former `C4.1`
identity is historical provenance; the R1 path is their current lifecycle
home. The move and this record do not claim that open findings have been
resolved or that their historical evidence is current-head verification.

## Lifecycle state

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
parent_plan: LANG007.C4.P4
next_action: Resolve or explicitly defer the open R1 findings through their paired issue plans, then run broad certification only after focused prerequisites are satisfied.
```

The covering-inherent-impl correction and its recorded focused checks remain
completed historical work. R1 stays active because its issue corpus includes
unresolved incremental and source-coverage obligations, current-head certification work,
expanded Telogue generic-Pack inference failures, and package-scoped Hypothesis runtime
blockers whose incident record is co-located here.

## Issue register

| ID | Finding at recorded revision | Current record state | Detailed records |
|---|---|---|---|
| A1 | Match coverage has a private equality-normalization authority; confirmed architectural defect at `ee7e8ee7` | Complete / IMPLEMENTED / FOCUSED_TESTED; 6 shared-query unit tests and 153 matching tests passed ([handoff](LANG007.C4.P4.R1-A1-handoff.md)) | [Audit](issue-audits/LANG007.C4.P4.R1-AUD-A1-match-equality-authority.md), [plan](issue-plans/LANG007.C4.P4.R1-PLAN-A1-match-equality-authority.md) |
| A2 | Constructor proof handling collapses refutation and blocked proof; confirmed architectural defect at `ee7e8ee7` | Complete / IMPLEMENTED / FOCUSED_TESTED on main at `1002ab70`; 155 matching, 4 proof-unit, 3 coverage-unit, and 1 lowering-guard tests passed ([handoff](LANG007.C4.P4.R1-A2-handoff.md)) | [Audit](issue-audits/LANG007.C4.P4.R1-AUD-A2-constructor-proof-outcome.md), [plan](issue-plans/LANG007.C4.P4.R1-PLAN-A2-constructor-proof-outcome.md) |
| I1 | Callable-body invalidation lacks absence-transition certification | Open proof obligation; no production failure demonstrated by the audit | [Audit](issue-audits/LANG007.C4.P4.R1-AUD-I1-conformance-absence-invalidation.md), [plan](issue-plans/LANG007.C4.P4.R1-PLAN-I1-conformance-absence-invalidation.md) |
| T2 | Public covering-signature coordinate purity needs recursive coverage | Open verification obligation; no implementation defect demonstrated by the audit | [Audit](issue-audits/LANG007.C4.P4.R1-AUD-T2-public-signature-coordinate-purity.md), [plan](issue-plans/LANG007.C4.P4.R1-PLAN-T2-public-signature-coordinate-purity.md) |
| T3 | Nested covering forms need more source-level certification | Open verification obligation; no implementation defect demonstrated by the audit | [Audit](issue-audits/LANG007.C4.P4.R1-AUD-T3-nested-covering-source-certification.md), [plan](issue-plans/LANG007.C4.P4.R1-PLAN-T3-nested-covering-source-certification.md) |
| V1 | Broad current-head certification is absent | Open certification obligation | [Audit](issue-audits/LANG007.C4.P4.R1-AUD-V1-broad-certification.md), [plan](issue-plans/LANG007.C4.P4.R1-PLAN-V1-broad-certification.md) |
| TEL-1 | Telogue project compilation reaches semantic analysis and fails on generic-pack contributions, positional-lane projections, and `Query.of` inference | Original project-compilation objective COMPLETE / IMPLEMENTED / FOCUSED_TESTED at `c2d72fed`; expanded inference IN_PROGRESS / PARTIAL / FOCUSED_TESTED with failing GoalQuery calls ([handoff](LANG007.C4.P4.R1-TEL-1-pack-handoff.md)) | [Architect findings](corrective-history/LANG007.C4.P4.R1-Telogue-project-integration-findings-for-architect.md), [revised audit](corrective-history/LANG007.C4.P4.R1-revised-covering-impl-and-Telogue-integration-audit-ee7e8ee7.md) |
| HYP-1 | Hypothesis runtime corpus is 9 passed, 1 failed, 1 blocked; `Strategy.draw<P>` lacks an active runtime conformance target and integer replay hits a compiler panic | `BLOCKED / PARTIAL / FOCUSED_TESTED` at `ad1841f2`; the report keeps remediation scope package-side and records compiler/runtime blockers without changing those layers | [Incident report](hypothesis/LANG007.C4.P4.R1-hypothesis-incident-report.md), [test results](hypothesis/LANG007.C4.P4.R1-hypothesis-runtime-test-results.md) |

The classifications above preserve what each source record says. They do not
assert that any audit was rerun against the current working tree.

## Corrective history retained

`corrective-history/` preserves the sequence of implementation handoffs,
walkthroughs, audits, and integration findings:

1. Covering inherent-impl body-coordinate closure.
2. Incremental callable-body reuse follow-up.
3. Receiver-separation audit and required correction.
4. Match-coverage and incremental corrective audit at `24c25d5`.
5. Revised covering-impl audit and Telogue integration follow-up at
   `ee7e8ee7`.

The historical walkthrough records focused semantic evidence for the
covering-impl correction. The Telogue project integration result remains a
failure as documented in its findings; that result must not be presented as a
passing project compile.

## Work organization

The paired issue plans are the detailed implementation contracts. R1 tracks
their shared state and ordering:

1. Inherit completed A1 (`09ac903b`) and A2 (`1002ab70`)
   through their dedicated walkthroughs/handoffs; continue I1.
2. Establish the I1 absence-invalidation behavior with the planned incremental
   transitions.
3. Complete T2 and T3 coverage obligations without weakening the published
   signature or source-body coordinate invariants.
4. Inherit the passing Telogue project-entry gate at `c2d72fed`; resolve the
   expanded GoalQuery generic-rest call-site inference gap without weakening
   its positive source oracle. Classify the two additional generics failures.
5. Run V1 broad certification after the focused prerequisites are complete.

The Hypothesis incident records are co-located here at the user's direction.
Their package lifecycle remains independently `BLOCKED / PARTIAL /
FOCUSED_TESTED`; the incident report scopes this work to recording the generic
dispatch and replay blockers without implementing compiler or VM changes.

P4's remaining-scope execution order is now governed by the corrective
completion plan. Begin with its T0 live-owner rebaseline, then converge
`StaticConstant` formation before extending owner scheduling or alias support.
T7.0E remains an acceptance gate, and Record-pattern work remains gated on it.
R1 records the companion corrective findings and their evidence alongside the
P4 completion work.

The Hypothesis package incident and runtime result records are co-located here
at the user's direction. Their own lifecycle remains
`BLOCKED / PARTIAL / FOCUSED_TESTED`; this association does not expand their
package-only repair scope.

## File map

- [`corrective-history/`](corrective-history/) — prior corrective handoffs,
  audits, walkthrough, and Telogue findings.
- [`issue-audits/`](issue-audits/) — six issue and verification audits.
- [`issue-plans/`](issue-plans/) — paired implementation and certification
  plans.
- [`LANG007.C4.P4-completion-luna-patch-grade-implementation-plan.md`](../LANG007.C4.P4-completion-luna-patch-grade-implementation-plan.md)
  — current P4 remaining-scope execution plan.
- [`hypothesis/`](hypothesis/) — Hypothesis-specific incident and runtime
  result records; filenames carry the package name explicitly.
- [`LANG007.C4.P4.R1-walkthrough.md`](LANG007.C4.P4.R1-walkthrough.md) — this
  record's interim documentation walkthrough.
- [`LANG007.C4.P4.R1-handoff.md`](LANG007.C4.P4.R1-handoff.md) — next-step
  continuation guidance.

## Local main consolidation — 2026-10-06

A1 source/tests are committed as `09ac903b`; TEL-1 source/tests as `c2d72fed`.
Lifecycle files are delivered in the following documentation commit. The local
Telogue branch has no unique commits. All extant detached worktree commits are
already ancestors of main; the pending imported static-domain regression in
the P2 verification worktree already exists identically on main. No checkout
was reset, deleted, or archived, and no work was discarded. No new behavioral
tests or remote push ran for this consolidation. R1 remains partial/unverified
overall; A1 focused completion is not V1 certification.
