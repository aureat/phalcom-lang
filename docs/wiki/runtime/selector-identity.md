# Selector identity

> Sources: shared selector identity contract
> Raw: [shared utility source snapshot](../raw/common/2026-09-07-phalcom-common.md)
> Updated: 2026-09-08

Selector identity is the dispatch key shared by parser-facing calls, native declarations, catalog lookup, runtime sends, and reflection. Exact selectors distinguish named/subscript bases, getter/setter/method kinds, and ordered labels; selector patterns are predicates, not dispatch keys.

## Decoding

The exact decoder validates canonical source-like syntax and slot ordering. Runtime decoding is total and converts malformed or legacy transport text into a reflection-safe selector; the permissive rest-family path is deliberately not equivalent to strict validation.

[Canonical native surface](../native/canonical-surface.md) indexes selectors. [Callable contracts](../type-system/callable-and-generic-contracts.md) explains how labels and rest parameters are checked.
