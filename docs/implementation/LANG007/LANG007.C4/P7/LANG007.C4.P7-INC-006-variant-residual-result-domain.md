# IMPLEMENTATION INCIDENT

## Identity and trigger

- `LANG007.C4.P7.T10 / G10`, `main` at `cb8a068c` plus scoped work.
- RED triggers 8 and 11: a canonical Variant residual carries an exact-case
  result type, while its checked static definition may declare the Sum type.
  Accepting the result requires changing the evaluator's cross-layer result
  contract, not a mechanical enum match.

## Evidence and reproducer

```
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic partially_known_variant_preserves_constructor_and_specializes
```

`PartialVariant<N: Int>: Option<Int> { Option::Some(N) }` checks without
diagnostics. Its canonical `CheckedStaticConstruction` reports `VariantId`
`Option::Some` with `ExactCase` result type `TypeId(1233)`; the definition's
declared domain is `Option<Int>` (`TypeId(1222)` in this session). The
evaluator's strict `validate_result_domain` rejects the resulting Variant
residual before specialization. The corresponding exact construction and
the existing concrete match/construction regressions still pass.

## Invariant and options

P7-I10 requires cold exact and open-then-specialize to agree. The canonical
Variant and checked constructor identities must survive partial construction;
a wrong-domain user residual must still fail closed. A blanket acceptance of
all `ValueType` residuals would weaken that invariant. The two bounded
directions were canonical proof of `ExactCase <: declared Sum`, or an enclosing
`Apply` fallback losing the direct Variant result.

## User ruling — 2026-10-13

Use the existing bounded subtype authority to validate the Variant
exact-case result against the definition's declared Sum domain. Preserve
direct canonical Variant residual specialization and distinct
blocked/dynamic/budget/cancellation terminals. Do not weaken validation of
unrelated Parameter/Proof/MemberProjection residuals. This is a Level-2
result-domain amendment to T10; follow with the cold/open Variant regression.
