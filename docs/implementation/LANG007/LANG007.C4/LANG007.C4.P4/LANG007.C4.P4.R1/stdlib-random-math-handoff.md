---
kind: documentation-handoff
title: "stdlib math and random guidance — auditor handoff"
status: INTERIM
completion: PARTIAL
verification: UNVERIFIED
recorded: 2026-10-07
branch: stdlib-random-math
guidance_commit: 3369adf1
implementation_commit: 3ab77b6e
---

# stdlib `math` and `random` guidance — audit handoff

## Read first

1. [`stdlib-random-math.md`](stdlib-random-math.md), the proposed design and
   implementation guidance.
2. [`LANG007.C4.P4.R1-stdlib-parse-telos-follow-on-plan.md`](LANG007.C4.P4.R1-stdlib-parse-telos-follow-on-plan.md),
   whose final sections clarify known source-language blockers and acceptance
   gates.
3. [`LANG007.C4 checkpoint`](../../LANG007.C4-CHECKPOINT.md) and
   [`R1 companion record`](LANG007.C4.P4.R1-plan.md) for the explicit boundary:
   this stdlib guide does not advance P4/C4 acceptance or alter R1 issue state.

## Audit questions

Verify the proposals against live source and accepted specifications before
adopting them as an implementation contract:

- Confirm the method-family capture/export, native binding, package layout, and
  `Int` operation assumptions from the current source.
- Review the float backend, IEEE edge-case rules, finite-value accuracy target,
  and supported-platform determinism requirements.
- Check xoshiro256++/SplitMix64 algorithms, exact seed validation, golden replay
  vectors, 64-bit arithmetic, rejection sampling, and the ordering/random-word
  consumption contracts for `choose` and `shuffle`.
- Verify that secure system randomness has an explicit host capability and
  failure path, cannot silently fall back to a deterministic or predictable
  source, and remains isolated from VM hash-hardening state.
- Classify possible gaps such as `Int.abs`, bit rotations, mutable
  random-access collection support, unsigned machine-word types, mutation
  typing, and secure host capability modeling. Do not treat listed candidates
  as confirmed defects without evidence.
- Preserve the known parse/Telos issues at their owning layer: `and`/`or` in
  value position may yield `Object` instead of `Bool`; associated-type dispatch
  or data-method lowering may block source fixtures.

## Evidence boundary

Commit `3369adf1` contains documentation only. A partial math implementation
was subsequently delivered in `3ab77b6e`. The implementation is
`IN_PROGRESS / PARTIAL / UNVERIFIED`; its last recorded focused discriminator
fails because module-qualified constant getter dispatch attempts to call the
resolved Float (`3.141592653589793 does not understand 'call'` at
`main.ph:4:17`). No tests were run during delivery. Review the
[`implementation handoff`](stdlib-random-math-implementation-handoff.md) for
the full code audit, exact ownership risks, and next focused gates. The guide
remains unverified, and this work does not alter existing C4/P4/R1 statuses.
