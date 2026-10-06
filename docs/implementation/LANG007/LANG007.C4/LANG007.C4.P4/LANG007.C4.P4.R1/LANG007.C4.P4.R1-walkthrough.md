---
id: LANG007.C4.P4.R1
kind: implementation-walkthrough
title: "P4 corrective issues companion — initial record organization"
status: INTERIM
completion: PARTIAL
verification: UNVERIFIED
recorded_against_revision: 2d306c13da02fb3b85ee362498cf4c0280de3825
recorded: 2026-10-06
---

# LANG007.C4.P4.R1 — Interim walkthrough

## Outcome

Created the R1 companion lifecycle record under P4 and moved the complete
20-file corpus from the former implementation and audit `LANG007.C4.1`
folders. At the user's direction, also moved the two active Hypothesis
lifecycle records from the package into a named `hypothesis/` section. The
moves preserve their findings and cited evidence revisions. Files now have
descriptive names and are grouped by role.

## Organization

- `corrective-history/` contains the eight implementation handoff, follow-up,
  audit, walkthrough, and Telogue-integration records.
- `issue-audits/` contains the six A1, A2, I1, T2, T3, and V1 audit reports.
- `issue-plans/` contains their six paired implementation or certification
  plans.
- `hypothesis/` contains the Hypothesis-specific incident report and runtime
  result ledger, each with `hypothesis` in its filename.
- The R1 plan is the issue register and lifecycle entrypoint. This walkthrough
  records the initial organization; the handoff identifies the next actions.

Internal references were updated to use the new locations, including the
Hypothesis package README and the C3 checkpoint's incident link. The C4
checkpoint and P4 handoff identify R1 as the companion and preserve P4's
existing active objective and verification classification.

## Work and verification boundaries

This record organization does not implement or verify the issue plans. No
compiler tests or project build were run for this documentation-only change.
The historical focused-test results remain attributed to their original
walkthrough. A1/A2 remain classified as confirmed at the audit revision, I1,
T2, and T3 remain open proof or coverage obligations, V1 remains open, and the
Telogue project integration result remains the recorded failure.

R1 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`. P4 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED` under the C4 checkpoint.
