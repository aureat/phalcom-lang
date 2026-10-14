# IMPLEMENTATION INCIDENT

## Identity and evidence

`LANG007.C4.P7.T10 / G10`, `main` at `cb8a068c` plus scoped work.
`StaticResidualData::Pack` currently reports `Kind(TYPE_PACK)` and has no
checked result type. The canonical `StaticValueStore::Pack` is a first-class
ordinary value: `static_value_type` forms a proper Pack type, while
`domain_of` cannot classify a structural value without semantic tables.
The C4 structural-products spec §2 distinguishes this value from C3 kinded
`TypePack` terms. The current residual payload cannot faithfully describe
both domains.

## Trigger and invariant

Repository RED triggers 6 and 16: residual result-domain representation
conflicts with accepted Pack semantics. P7-I10 requires cold/open
specialization parity and P7's domain-neutral term envelope must preserve
the actual checked result domain.

## User ruling — 2026-10-13

Change `StaticResidualData::Pack` to carry its checked proper `result_type`
and report `ValueType(result_type)`. Kinded `TypePack` terms stay with the
existing TypeStore/Parameter/Apply paths. Specialization must intern a fully
known Pack through the canonical `StaticValueStore` and retain ordered
positional/labeled lanes while still partially known. The corresponding
P7 T10/G10 acceptance is amended; no legacy Kind(TYPE_PACK) behavior is
inherited for ordinary Pack values.
