---
id: STDL001.C6
category: STDL
program: STDL001
checkpoint: STDL001.C6
kind: checkpoint-record
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
last_updated: 2026-10-11
active_plan: STDL001.C6.P1
---

# STDL001.C6 — Exact Rational

## Objective

Add a canonical immutable `Rational` value over arbitrary-precision `Int`, with
normalized construction, exact arithmetic and ordering, explicit conversion,
value equality and hashing, and compatibility with generic `Complex<T>` and
`Quaternion<T>` arithmetic through the existing numeric capabilities.
`Complex<Rational>` also has exact division and integer powers.

## Plan ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
| --- | --- | --- | --- | --- | --- |
| `STDL001.C6.P1` | Canonical Rational source type and numbers facade integration | BLOCKED | PARTIAL | UNVERIFIED | implementation is present; acceptance is blocked by a branch-only stack overflow in canonical Universe semantic expression analysis; clean `origin/main` completes the reproducer; see [incident report](../stack-overflow-incident-2026-10-11.md) |

## Fixed invariants

- `Rational` is concrete, not publicly generic.
- Every public construction normalizes to a positive denominator and coprime
  components; zero is stored as `0/1`.
- Division remains exact and rejects a zero divisor.
- Conversion to Float is explicit and approximate; no implicit numeric
  promotion is introduced.
- The data representation stays private behind the validating constructor.

## Stable interface

- `universe.numbers.Rational`
- `Rational.new(numerator, denominator)` and `Rational.fromInt(value)`
- Exact arithmetic, comparison, normalization-aware equality and hashing,
  `toFloat`, `toIntExact`, `floor`, `ceil`, and `truncated`
- Existing `Zero`, `One`, `Addable`, `Subtractable`, `Multipliable`,
  `Negatable`, and `IntegerPower` capabilities

## Verification and deferred work

The source API and numbers-facade exports, plus exact `Complex<Rational>`
division and powers, are present. Rational source acceptance is blocked by a
branch-only stack overflow in recursive semantic analysis of a canonical
Universe expression. Source syntax, runtime behavior, and generic
`Complex<Rational>` / `Quaternion<Rational>` behavior remain unverified. See
the [incident report](../stack-overflow-incident-2026-10-11.md).

## Next action

Wait for the owner-level architectural ruling and a passing minimal-project
CLI regression before resuming focused Rational source acceptance.
