---
id: LANG007.C4.P4.R1-A2
kind: implementation-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
recorded: 2026-10-06
starting_revision: 4a1cd6c7
implementation_revision: 1002ab70
consolidated_branch: main
---

# R1 A2 — Constructor proof outcomes handoff

Inherit A2 as COMPLETE / IMPLEMENTED / FOCUSED_TESTED on main at `1002ab70`.
Its implementation, regressions, and checkpoint update are committed together;
these completion records are synchronized in the following documentation commit.
The user authorized commit/push delivery after the inline implementation. The
preceding A1/TEL-1 consolidation was already pushed. Continue inline on main and
preserve the established interfaces. Delivery adds no behavioral verification.

Read the C4 checkpoint and R1 issue register first. The
[A2 walkthrough](LANG007.C4.P4.R1-A2-walkthrough.md) records focused results and
residual failures; its implementation amendment is in the paired issue plan.

## Established takeover interfaces

- `gadt_proof::LocalCaseProof` and `coverage::VariantOpenOutcome` distinguish
  Reachable, Refuted, and Blocked. Canonical `GadtProofResult` and `ProofMerge`
  preserve blocked uncertainty too. Every opening caller handles all outcomes.
- Only a supported contradiction or occurs-check failure can refute a case.
  Opaque local identity, unsupported projections/representations, unordered
  union pairings, cancellation, and exhausted limits cannot become omission.
- Constructor opening and local equality use the owning shared `CheckerControl`.
  Inhabitation and summaries pass it through; do not create a fresh control for
  nested queries. The compatibility local-proof wrapper is unused in production.
- Exact/enum/union decomposition is fail-closed. Union local members survive
  recursion; an Open member cannot conceal a Blocked member.
- `ResolvedVariantPattern.blocked` retains uncertainty with declaration-owned
  family/owner/selector identity. Nested/Or patterns and common-proof extraction
  forward blocked state to the owning coverage engine. Do not turn a blocked
  singleton into a binding or fabricate candidate proof metadata.
- LocalType, canonical IDs, rigid scopes, parent-rigid payload preservation, and
  materialization boundaries are unchanged. Unsupported local representation
  remains blocked rather than expanded through a second semantic authority.

## Verified and deferred

Final evidence: matching 155 passed / 16 existing ignored; proof units 4 passed;
coverage units 3 passed; compiler rejection guard 1 passed; semantic crate check
and scoped whitespace review passed. The valid nested source regression was
red before the patch and green afterward. These are focused gates, not release
certification.

The broader checker probe's unchanged resolver test
`tracking_resolver_demands_the_exact_unready_type_member_owner` remains deferred
classification C, without predecessor baseline proof. The new domain fixture's
failure is resolved. Do not weaken either oracle or label the resolver failure
baseline without evidence. An invalid constructor-result projection source
probe was removed; direct proof tests do not certify that source producer.

Broad semantic/core/workspace/lint/release gates and runtime checks remain unrun.
TEL-1's expanded GoalQuery rest-call inference and its two unclassified failures,
and the Hypothesis package incident, remain independent open work.

## Next action

Begin [I1 conformance absence-invalidation certification](issue-plans/LANG007.C4.P4.R1-PLAN-I1-conformance-absence-invalidation.md).
Its audit demonstrated a proof obligation, not a production defect: establish
live transition evidence before proposing a repair. Inherit A1/A2 without
re-auditing their established ownership/interfaces. R1/P4 stay partial/unverified;
P4's primary corrective completion plan remains authoritative. V1 broad
certification follows its prerequisites.
