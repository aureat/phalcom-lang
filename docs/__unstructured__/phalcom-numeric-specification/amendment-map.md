# Amendment Map from the Original Numeric Specifications

> **Status:** Informative.
>
> This document records how the revised set changes, corrects, or redistributes the original uploaded files. It is not a substitute for the normative documents.

## 1. Original `README.md`

### Problems corrected

- The original index omitted the normative numeric-literal and bitwise documents.
- It listed ratifying records without giving readers a complete specification map.

### Revision

The new README indexes semantics, implementation, conformance, migration, open decisions, and the combined edition. It distinguishes authority and open items.

## 2. Original `numeric-tower.md`

### Problems corrected

- Mixed permanent semantics with source paths, stale commit baseline, concurrency hazards, dependency details, and implementation phases.
- Called itself implementation-ready while leaving constant-pool rooting as a ship gate.
- Used one general promotion shape that could not serve exact comparison or exact mixed `~/`.
- Suggested `rem_euclid`-style behavior despite negative-divisor floor remainder.
- Retained pending-ratification wording after ratification.
- Deferred strict Int-only index boundaries despite the architecture now ratifying immediate tightening.
- Described Number as both protocol home and “empty class.”
- Carried obsolete decimal-only/`Token::BigInt(String)` pseudocode after radix literals were ratified.
- Suggested a host/runtime zero-division variant inconsistent with structured language Errors.

### Revision

Permanent semantics now live in `numeric-tower.md`; runtime/compiler details live in `implementation.md`; tests live in `conformance.md`.

The revised semantics:

- separate lossy Float arithmetic from exact comparison, hashing, keys, and `~/`;
- define exact dyadic floor division;
- define floor remainder for Int and Float;
- require Int-to-Float overflow errors;
- require strict Int-only boundaries;
- clarify allocator-abstract versus method-empty;
- land equality, key relation, and hashing as one coherent subsystem.

## 3. Original `float-protocol.md`

### Problems corrected

- Delegated Float `%` to host `fmod`, creating tower-wide remainder inconsistency.
- Used ties-away rounding.
- Claimed host Float power could vary only in NaN payload bits.
- Had no public total-order operation.
- Left zero-negative-power behavior ambiguous for Float zero/mixed operands.
- Defined user hash return as Int but did not account for heap-backed large Int consumption.

### Revision

- Float `%` is floor remainder.
- `rounded` uses ties to even.
- Float power has explicit special cases plus one-ULP ordinary finite accuracy.
- every numeric zero to negative power raises;
- a total-order operation is required, with name/order still open;
- exact Float decoding is shared across comparison, narrowing, division, and hashing.

## 4. Original `numeric-literals.md`

### Problems corrected

- Used `DIGIT` without defining it.
- Did not fully specify malformed-candidate boundaries.
- Did not settle `5.e2` after the architectural review.
- Required compiler-minted LargeInt objects in a constant pool, leaving heap/GC coupling.
- Used one diagnostic code for syntax without distinguishing policy excess.

### Revision

- Grammar is self-contained.
- Adjacent identifier, radix, exponent, dot-send, and range boundaries are explicit.
- `5.e2` is an ordinary send.
- large constants use heap-independent descriptors.
- `numeric.literal` and `numeric.limit` are distinct.

## 5. Original `text-and-errors.md`

### Problems corrected

- Referenced undefined/mismatched grammar nonterminals.
- Called Float conversion from arbitrary Int a “widening.”
- Did not specify decimal conversion rounding or underflow.
- Made exact English message templates stable.
- Did not define exact malformed-text byte-offset selection.
- Introduced `#numericLimit` without a configured policy model.
- Omitted `trailingZeros(0)` error kind.

### Revision

- Constructor grammar is complete and shares named productions.
- Int-to-Float conversion rounds ties-to-even and raises on finite-range overflow.
- text parsing underflow/overflow behavior is explicit.
- error kinds/fields/spans are stable; prose is not.
- byte offsets use first offending UTF-8 byte or EOF length.
- compiler and runtime policy failures are distinct.
- `#undefinedNumericOperation` covers partial numeric queries.

## 6. Original `bitwise.md`

### Problems corrected

- Referenced undefined `2.pow(n)` instead of `2 ** n`.
- Omitted `**` and prefix `~` interaction from precedence.
- Used “magnitude-independent” for `trailingZeros` instead of sign-independent.
- Named allocation failure rather than deterministic `#numericLimit`.
- Did not define huge nonnegative counts that exceed `usize`.
- Did not account for primitive-floor growth.
- Stated unconditional algebraic laws despite resource-policy failure.

### Revision

- laws use the actual power selector;
- complete relevant precedence is stated;
- huge right shifts and bit indexes short-circuit by sign extension;
- left shift uses policy preflight;
- trailing-zero error is structured;
- laws are qualified by successful completion under policy;
- primitive composition is tracked as OD-NUM-006.

## 7. New `implementation.md`

Created to hold material that should not be permanent language semantics:

- Value/Object shapes;
- canonical normalization;
- exact Float decomposition;
- semantic-kernel boundaries;
- heap-independent constants;
- GC/rooting;
- hash architecture;
- class/primitive placement;
- resource-policy hooks;
- dispatch invalidation;
- phased landing plan.

## 8. New `conformance.md`

Created because examples embedded in design prose were insufficient to pin:

- mixed precision boundaries;
- all floor-division sign pairs;
- Float bit classes;
- Map/Set representative preservation;
- arbitrary LargeInt user hashes;
- parser candidate consumption;
- resource limits;
- generic/optimized equivalence;
- primitive-floor invariants.

## 9. New `migration.md`

Created to isolate breaking behavior and prevent compatibility choices from contaminating core semantics.

## 10. New `open-decisions.md`

Created so unresolved names, tables, constants, defaults, encodings, algorithms, and release policy cannot be mistaken for implementation freedom or silently inherited from host behavior.
