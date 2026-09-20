---
id: TEST002.C2
category: TEST
program: TEST002
checkpoint: TEST002.C2
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
requires:
  - TEST002.C1.P2
---

# TEST002.C2 - Runtime correctness remediation

## Objective and ownership boundary

This checkpoint owns the retained confirmed runtime-correctness findings from
the 2026-09-07 audit. It preserves compact bytecode, stable runtime identity,
GC ownership, native boundary atomicity, exact numeric domains, Resource-v2,
and Streams-v2 architecture established by TEST002.C2.P1. It does not own
workspace/release certification, the deferred executable-chunk verifier, or
unmeasured runtime/cache optimization.

## Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
| --- | --- | --- | --- | --- | --- |
| TEST002.C2.P1 | Runtime correctness remediation across retained AUD-RUNTIME findings | IN_PROGRESS | PARTIAL | UNVERIFIED | Active from `770cf561d04206b57f6b4b8b3bc96d86eaa77307`; first cluster is AUD-RUNTIME-006 numeric domains |

## Starting state

| Field | Value |
| --- | --- |
| Starting revision | `770cf561d04206b57f6b4b8b3bc96d86eaa77307` (`main`) |
| Starting tree | Clean except unrelated untracked `docs/implementation/TEST002/TEST002.C2/` and `docs/implementation/plan.py`; preserved |
| Inherited prerequisite | TEST002.C1.P2 is `IMPLEMENTED` + `FOCUSED_TESTED`; BL-19 resolved |
| Active plan | TEST002.C2.P1 |
| Active cluster | AUD-RUNTIME-006 numeric domain contracts |

## Established invariants

- `~/` computes exact represented-value floor division and returns exact `Int`.
- Float-domain `%` uses floor remainder, exact dyadic arithmetic, one final
  correctly-rounded binary64 result, and divisor-signed zero.
- Numeric conversion follows operation-specific domains; ordinary Float-result
  promotion is not reused for exact operations.
- `float_to_rational` is the canonical finite-binary64 decoder.
- No guest Rational value is introduced by this remediation.

## Stable interface / takeover map

| Fact | Owner | Consumer |
| --- | --- | --- |
| Exact finite Float decoding | `phalcom-core/src/primitive/float.rs::float_to_rational` | exact numeric primitives |
| Numeric primitive dispatch | `phalcom-core/src/primitive/number.rs` | native sends and language corpus |
| Retained numeric probe | `phalcom-core/tests/core/audit_probes/numeric_contracts.rs` | C2 focused evidence |

## Verification ledger

| Gate | Scope | Result |
| --- | --- | --- |
| T0 | AUD-RUNTIME-006 retained native probe before repair | FAIL reproduced: mixed `~/` rounded `9007199254740993` to `9007199254740992`; Float `%` used host remainder signs |
| T8/G4 | Exact `~/` and Float `%` numeric slice | PASS: retained and expanded numeric audit regressions, dyadic rounding unit, `numeric_values`, and `numeric_diagnostics` all pass |

## AUD-RUNTIME-006 root-cause record

Observed: mixed `Int`/`Float` `~/` converted the exact integer through the
shared `PromotedPair::Float` path, and Float `%` used Rust's truncating `%`.

Causal hypothesis: operation-independent promotion discarded represented-value
identity before exact quotient/remainder logic could run.

Evidence: the retained probe failed with `9007199254740993 ~/ 1.0 ==
9007199254740992` and `-5.0 % 3.0 == -2.0`; `number_floor_div` and
`number_mod` both matched on `PromotedPair::Float`.

Predicted effect: operation-specific dyadic arithmetic using
`float_to_rational`, with one final binary64 rounding for Float `%`, preserves
the exact quotient and divisor-signed floor remainder.

Discriminating test: `audit_probes::numeric_contracts` now covers the retained
cases, 2^53 neighbors, mixed signs, special values, and signed zero; the
private dyadic helper covers ties and subnormal boundaries.

## Durable numeric interface

- `number_floor_div` now extracts exact `BigInt`/finite-Float dyadics and uses
  `div_floor` without converting exact `Int` operands to Float.
- `number_mod` keeps Int-only remainder exact and computes Float-domain floor
  remainder from exact dyadics, rounding once with ties-to-even.
- `dyadic_to_float` is the private canonical exact dyadic-to-binary64 helper;
  `float_to_rational` remains the canonical binary64 decoder.

## Deferred / baseline issue ledger

- AUD-RUNTIME-001, -002, -003, -004, -005, -007, and -008 remain active C2
  clusters and are not represented as fixed by this numeric slice.
- AUD-RUNTIME-005.1 full executable-chunk verification, .2 rest/native
  metadata optimization, and .3 sparse per-instruction metadata remain
  deferred hardening/performance work as specified by P1.
- Unrelated pre-existing failures remain outside this checkpoint unless a
  focused result proves they are caused by a C2 change.

## Consultation / amendment ledger

| Incident | Plan/task | Trigger | Decision |
| --- | --- | --- | --- |
| None | None | No architectural escalation at start | None |

## Active plan and next action

- Active plan: TEST002.C2.P1.
- Latest completed plan: TEST002.C1.P2 (inherited).
- Next action: reproduce the retained AUD-RUNTIME-006 probe, then add its
  focused regression before changing numeric production code.
