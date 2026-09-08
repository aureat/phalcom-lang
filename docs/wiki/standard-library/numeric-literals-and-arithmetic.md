# Numeric literals and arithmetic

> Sources: numeric conformance specification; STDL001
> Raw: [standard-library specification and program snapshot](../raw/standard-library/2026-09-08-standard-library-sources.md)
> Updated: 2026-09-08

Numeric literals and arithmetic require exact classification, radix/separator validation, floor quotient/remainder semantics, and explicit handling of overflow, infinities, NaNs, and rendering boundaries. The lexer/parser and runtime are separate evidence layers.

The reference model is implementation-independent; host operations are not authoritative where their width, remainder, or power rules differ. STDL001 is in progress and partial. See [numeric contracts](numeric-contracts.md).
