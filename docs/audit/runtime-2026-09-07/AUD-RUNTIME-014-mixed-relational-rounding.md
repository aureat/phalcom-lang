# AUD-RUNTIME-014 - Mixed relational comparisons round before ordering

## Classification

- Severity: High
- Category: Numeric domain correctness
- Confidence: Confirmed source candidate; focused execution pending.

## Observed

Mixed `Int`/`Float` relational operations use the shared promotion path and
compare converted Float values. Values beyond 2^53 can collapse onto the same
binary64 representation, while the numeric specification requires exact mixed
numeric ordering.

## Reproducer and repair direction

Compare `9007199254740993` with `9007199254740992.0` through `<`, `<=`, `>`
and `>=`; the exact result is false, false, true, true. Repair the canonical
numeric comparison owner with dyadic cross-multiplication, without changing
the already repaired exact quotient/remainder domains.
