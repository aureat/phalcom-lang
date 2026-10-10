---
id: STDL001.C6.P1
kind: implementation-plan
checkpoint: STDL001.C6
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
prepared_against_revision: fdbc2c41
last_updated: 2026-10-11
---

# STDL001.C6.P1 — Canonical Rational implementation

## Objective

Implement the Phase 1 proposal's concrete `Rational` API in
`universe.numbers`, keeping representation private and exactness explicit.

## Fixed architecture

- Use a private immutable data representation with a public validating class
  constructor.
- Normalize denominator sign and reduce by `gcd(abs(numerator), denominator)`.
- Implement exact operations directly over arbitrary-precision `Int`.
- Preserve explicit conversions and canonical normalized value semantics.
- Reuse existing numeric capabilities so generic complex and quaternion
  operations can instantiate for `Rational`.

## Tasks

| Task | State | Scope |
| --- | --- | --- |
| T1 | PARTIAL | Add normalized Rational representation, arithmetic, conversions, comparison, equality, hashing, and formatting. Exact `Complex<Rational>` division and integer powers are also source-defined. |
| T2 | PARTIAL | Register `Rational` in `universe.numbers` and provide existing algebra capabilities. |
| T3 | NOT_STARTED | Review source and prepare focused source/runtime acceptance coverage when testing is authorized. |

## Verification boundary

Implementation work may proceed without broad workspace verification. Source
acceptance and runtime behavior are not certified until a focused language test
or equivalent source compilation is explicitly authorized and completed.

## Completion criteria

- Public construction cannot expose a zero denominator or noncanonical value.
- Exact operations preserve normalized value semantics, including zero and
  negative denominators.
- Equal values produce equal hashes.
- Generic Complex/Quaternion methods can require the existing numeric
  capabilities from Rational without implicit conversion; `Complex<Rational>`
  division and integer powers remain exact.
- Focused source acceptance passes before the checkpoint is promoted.

## Current blocker — 2026-10-11

Source acceptance is paused because a dependency-free CLI project overflows the
native stack. The failing phase and owner are unknown. Preserve the current
architecture pending adviser guidance; see the
[incident report](../stack-overflow-incident-2026-10-11.md).
