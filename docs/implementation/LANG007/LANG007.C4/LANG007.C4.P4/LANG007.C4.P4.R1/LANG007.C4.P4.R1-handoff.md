---
id: LANG007.C4.P4.R1
kind: implementation-handoff
title: "P4 corrective issues companion — continuation handoff"
status: INTERIM
completion: PARTIAL
verification: UNVERIFIED
recorded_against_revision: 2d306c13da02fb3b85ee362498cf4c0280de3825
recorded: 2026-10-06
updated: 2026-10-07
consolidated_branch: main
---

# LANG007.C4.P4.R1 — Interim handoff

R1 is an active companion issue register. Its status is
`IN_PROGRESS / PARTIAL / UNVERIFIED`. The C4 checkpoint remains authoritative
for P4 state. P4's remaining-scope work now starts with T0 in the corrective
completion plan: rebaseline the live owner matrix before production edits.

## Read first

1. [`LANG007.C4 checkpoint`](../../LANG007.C4-CHECKPOINT.md) for the shared
   C4 state and current P4 objective.
2. [`P4 corrective completion plan`](../LANG007.C4.P4-completion-luna-patch-grade-implementation-plan.md)
   for the current remaining-scope architecture and T0 entry gate.
3. [`LANG007.C4.P4 handoff`](../LANG007.C4.P4-handoff.md) for inherited P4
   interfaces and verified T1-T6 state.
4. [`LANG007.C4.P4.R1 plan`](LANG007.C4.P4.R1-plan.md) for the companion
   findings and sequencing.
5. The paired issue audit and plan for the specific finding being continued.

## Adjacent stdlib guidance audit handoff

The user also requested review of the adjacent [`stdlib-random-math.md`](stdlib-random-math.md)
guide, committed as `3369adf1` on `stdlib-random-math`. A partial math
implementation was later pushed as `3ab77b6e`. These remain outside the R1 issue
register and do not supersede the P4 corrective completion plan. The math
implementation is `IN_PROGRESS / PARTIAL / UNVERIFIED`; no tests were run
during delivery, and its last recorded focused discriminator fails on
module-qualified constant getter dispatch.

Read the [math implementation handoff](stdlib-random-math-implementation-handoff.md)
for the changed-file inventory, exact failing output, invariants, and remaining
verification gates. Review the code from remote branch `stdlib-random-math`.

Audit these points against the live source and accepted specifications before
using the guide as an implementation contract:

1. Method-family capture/export, native bindings, and package layout for the
   proposed `math` API.
2. IEEE special cases, numeric accuracy targets, backend choice, and
   cross-platform determinism.
3. `Int` semantics, arbitrary-precision behavior, and exact integer algorithms.
4. Xoshiro256++/SplitMix64 details, seed/replay contract, 64-bit operations,
   bounded arbitrary-precision sampling, and deterministic consumption order
   for `choose` and `shuffle`.
5. `SecureRandom` host capability, entropy-failure behavior, separation from
   deterministic PRNGs, and isolation from VM hash-hardening state.
6. Whether the investigation checklist and identified possible standard-library
   gaps are complete without expanding compiler/runtime scope.

The companion parse/Telos plan update records existing blockers: `and`/`or`
value-position expressions may produce `Object` instead of `Bool`, and
associated-type dispatch or data-method lowering may block source fixtures.
Preserve those as owning-layer issues; do not treat this guide as resolving
them.

The original doc-only delivery ran `git diff --check`; the subsequent code
delivery also ran `git diff --check` but no behavioral tests. Do not promote the
math slice to focused-tested based on commit delivery. C4/P4/R1 statuses and
the next action to continue P4 T0 remain unchanged.

## Inherited findings

- A1 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED on main at `09ac903b`: inherit
  its shared evidence query and dedicated walkthrough/handoff.
- A2 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED on main at `1002ab70`
  (starting revision `4a1cd6c7`). Inherit [its handoff](LANG007.C4.P4.R1-A2-handoff.md):
  typed proof/opening/merge uncertainty, declaration-backed blocked patterns,
  shared controls, and fail-closed exact/enum/union domains. Focused matching,
  proof/coverage units, lowering guard, and semantic check passed. The separate
  resolver unit failure is deferred classification C, not a confirmed baseline.
- I1 is an incremental proof obligation. Its audit did not demonstrate a
  production failure.
- T2 and T3 are coverage obligations for published coordinate purity and
  source-level nested forms.
- V1 is current-head certification work and depends on the focused prerequisites
  in its plan.
- TEL-1 source/tests are consolidated as `c2d72fed` on main. The original
  declared-entry compilation gate passed. Expanded generic-Pack correctness
  remains partial: GoalQuery empty/heterogeneous rest calls are underconstrained,
  and two additional generics failures are unclassified. Its latest dedicated
  walkthrough/handoff supersede the historical project-failure observation.
- The Hypothesis runtime corpus record reports 9 passed, 1 failed, and 1
  blocked at `ad1841f2`. The deterministic property-search case lacks an active
  runtime target for `Strategy.draw<P>`; integer replay panics during
  conformance-callable lowering. The incident report records these as separate
  blockers and keeps its scope package-side.
- The historical covering-impl correction has focused semantic evidence in
  its walkthrough. That evidence does not establish broad certification or a
  passing Telogue project compile by itself; the later TEL-1 record supplies
  that separate focused evidence.

These classifications are inherited from the records. Reassess them only
when new live evidence is acquired; do not silently mark an issue complete
from source changes alone.

## Fixed continuation boundaries

- Keep R1 as a companion to P4, not a second C4 checkpoint.
- Follow the current P4 corrective completion plan for its remaining scope;
  R1 issue tracking does not replace that plan or its gates.
- Follow the paired plans for A1, A2, I1, T2, T3, and V1. Escalate if a
  proposed correction changes semantic authority, proof outcomes, identity,
  or incremental dependency ownership.
- Keep the Telogue compile diagnostics distinct from the historical focused
  covering-impl result. Resolve each diagnostic at its owning layer and retain
  the project-entry regression as the integration discriminator.
- Keep the Hypothesis incident's compiler/runtime failures recorded in its
  named package lifecycle files. Do not broaden that package incident into
  compiler or VM implementation work through this P4 companion.
- Preserve unrelated working-tree changes. The organization task itself made
  no source-code changes.

## Next actions

1. Check the live status and scoped diff before continuing.
2. Inherit completed A1/A2 and begin I1 through its absence-invalidation plan;
   record meaningful focused evidence in the C4 checkpoint.
3. Complete I1, T2, and T3 obligations in the order allowed by their plans.
4. Resolve or separately plan the Telogue package blockers with their owning
   semantic/syntax layers identified.
5. Run V1 only after its prerequisites are satisfied, then update R1 and the
   shared C4 checkpoint with evidence actually acquired.

## Consolidated delivery

All current source/test progress is preserved on local main in `09ac903b` and
`c2d72fed`; this handoff and companion lifecycle records are updated in the
subsequent documentation commit. No remote push or new behavioral verification
was requested by consolidation. No worktree was discarded.

The user subsequently confirmed that consolidation was pushed and its local
branch deleted. A2 continuation stayed inline on main; implementation/tests and
checkpoint are committed as `1002ab70`. The subsequent user request authorizes
commit/push delivery, with completion records synchronized afterward. No checkout
progress was discarded. The A2 handoff supersedes the earlier next-action
instruction to begin A2.
