# Probability foundations

Status: implementation draft, currently blocked on canonical generic call inference.
The proposed Iterable entry points do not yet compile; see
`MATH-GENERIC-ITERABLE-INCIDENT.md`. The tests retain the required iterator
contract and are failing until this boundary is resolved.

This optional source package defines checked Float transformations and
normalization from math proposals 16 and 22:

- `logit`, `logOdds`, `odds`, `invLogit`
- `normalizeWeights`, `normalizeLogWeights`, `normalizedLogWeights`
- `UndefinedNormalizationError`

It depends only on Universe math and canonical collection protocols. It has no
RNG state and does not import the optional special-function backend.

Inputs are `Iterable` values with Float items. A normalization consumes its input
once into a private List, validates it, and returns a fresh List in input order.
Linear weights must be nonnegative finite Floats; logs may be finite or negative
infinity. Empty input raises ArgumentError; all-zero/all-impossible input raises
UndefinedNormalizationError. Huge finite weights use maximum scaling and
compensated summation. Log normalization shifts before reduction and never
subtracts an absolute log normalizer from an enormous score. Positive outputs
may underflow to zero; direct normalized logs retain finite values or raise
math.NumericalRangeError when an interior log is unrepresentable.

Distribution descriptors, sampling, conjugate updates, and Monte Carlo are
not implemented by this initial package. No full probability V1 claim is made.
