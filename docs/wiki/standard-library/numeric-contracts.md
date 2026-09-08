# Numeric contracts

> Sources: numeric conformance specification; STDL001
> Raw: [standard-library specification and program snapshot](../raw/standard-library/2026-09-08-standard-library-sources.md)
> Updated: 2026-09-08

Numeric conformance uses implementation-independent reference models: arbitrary-precision integers, exact dyadic finite-float decomposition, exact rational comparison, mathematical floor quotient/remainder, and explicit binary64 rounding where required.

## Evidence layers

Required layers include lexer/parser, compiler constants, runtime semantics, GC rooting, public-language goldens, negative/error goldens, property/differential tests, primitive equivalence, reflection invariants, and separately recorded performance. Host Rust arithmetic is not authoritative where its semantics differ.

STDL001 is in progress and partial. A numeric benchmark is not a numeric correctness proof.
