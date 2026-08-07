# U-NUMBERS-03 — exact Int/LargeInt arithmetic and limits

## Outcome

One promotion/coercion helper implements exact integer arithmetic, floor division, bitwise
operations, and nonnegative integer power. Every heap allocation and work-proportional operation
is bounded.

## Write set

- `phalcom-core/src/primitive/{int.rs,mod.rs}` and number coercion helpers.
- `phalcom-core/src/{value,heap,vm}/…`: BigInt normalization, allocation accounting, roots.
- collection/hash call sites only where exhaustiveness requires Int return handling.
- arithmetic, BigInt, GC-stress, and adversarial-limit tests.

## Steps

1. Centralize pair promotion: Int/Int stays exact; a Float operand takes the Float path. Never
   duplicate type-pair matching across selectors.
2. Implement `+ - *`, negation, `/`, `%`, `~/`, bitwise selectors, shifts, and `**` from the
   ratified contracts. `%` and `~/` satisfy `a == (a ~/ b) * b + (a % b)` for nonzero `b`.
3. Use exponentiation by squaring for `Int ** nonnegative Int`; preflight estimated result bits
   and charge actual allocation/work against a VM numeric budget. Negative Int exponents take the
   Float path. Enforce shift-count limits before allocating.
4. Before enabling compiler-created `LargeInt` constants, prove every `ObjRef` is traced through
   compilation, constant-pool ownership, and execution. If proof fails, add an explicit root.
   This is a release gate, not an optional optimization.
5. Return `#numericLimit` for exceeded bit, shift, exponent, or allocation budget. Never map OOM
   or host panic into an unstructured crash.

## Acceptance matrix

- boundary operations around `i64::{MIN,MAX}`, cross to/from `LargeInt`, and return class checks.
- negative dividends/divisors for `%` and `~/`; all zero divisors yield `#divideByZero`.
- powers 0, 1, very large exact result, negative exponent, `0 ** -1`, and adversarial exponents.
- forced-GC compile/run tests with large literal constants and large arithmetic temporaries.
- limits reject hostile inputs deterministically while ordinary large values remain usable.
