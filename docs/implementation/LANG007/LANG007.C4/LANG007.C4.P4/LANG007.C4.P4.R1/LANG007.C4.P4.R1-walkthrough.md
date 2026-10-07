---
id: LANG007.C4.P4.R1
kind: implementation-walkthrough
title: "P4 corrective issues companion — initial record organization"
status: INTERIM
completion: PARTIAL
verification: UNVERIFIED
recorded_against_revision: 2d306c13da02fb3b85ee362498cf4c0280de3825
recorded: 2026-10-06
updated: 2026-10-07
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

## Subsequent consolidation — 2026-10-06

The initial organization evidence above is historical. A1 has since completed
with focused tests (`09ac903b`); TEL-1's original project-compilation gate passed
and its source/tests are preserved at `c2d72fed`, while expanded generic-rest
inference remains partial with an unignored positive failing regression.
The updated issue register, shared C4 checkpoint, and dedicated A1/TEL-1
walkthroughs/handoffs record current state. R1 overall remains partial/unverified.
Consolidation performed no behavioral tests or push and discarded no progress.

## Subsequent A2 correction — 2026-10-06

The user confirmed the consolidation was pushed. A2 then ran inline on local
main from `4a1cd6c7`; implementation is committed as `1002ab70`. Constructor proof,
opening, and merging now preserve blocked uncertainty through semantic patterns
and coverage; the compiler rejects the blocked match before lowering. A2 is
COMPLETE / IMPLEMENTED / FOCUSED_TESTED: 155 matching tests, 4 proof units,
3 coverage units, and 1 compiler guard passed, along with semantic check. The
[detailed walkthrough](LANG007.C4.P4.R1-A2-walkthrough.md) records the red/green
source discriminator, invalid source probe, deferred resolver failure, and
unrun broad gates. R1/P4 remain partial/unverified; I1 is next.

The subsequent user-directed delivery commits the A2 implementation/tests and
checkpoint together, then synchronizes these lifecycle records. Delivery does
not rerun behavioral tests or promote the focused verification classification.

## Adjacent stdlib guidance delivery — 2026-10-07

Added [`stdlib-random-math.md`](stdlib-random-math.md), a design and
implementation-guidance document for proposed `universe.math` and
`universe.random` APIs. The related parse/Telos follow-on plan also received
scope and gate clarifications. Both files were committed on branch
`stdlib-random-math` at `3369adf1` and pushed to `origin/stdlib-random-math`.

This is documentation-only work outside the R1 issue register and P4/C4
acceptance. `git diff --check` passed before delivery; no tests or builds were
run. No implementation, spec ratification, or behavioral verification is
claimed. The new guide's proposals remain UNVERIFIED, while R1 and P4 retain
their existing `IN_PROGRESS / PARTIAL / UNVERIFIED` classifications and open
findings.
