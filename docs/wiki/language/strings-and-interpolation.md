# Strings and interpolation

> Sources: accepted string-interpolation specification
> Raw: [language specification and program snapshot](../raw/language/2026-09-08-language-sources.md)
> Updated: 2026-09-08

Phalcom source is UTF-8. String literals use double quotes or triple quotes, and interpolation is introduced by `\\(`. A literal interpolation opener is escaped as `\\\\(`. Each interpolation contains one top-level expression and lowers conceptually to `toString` plus concatenation.

## Source ranges

Front-end and diagnostics ranges are half-open UTF-8 byte offsets. The scanner must advance by complete Unicode scalar values and must never report an endpoint inside a scalar. Nested delimiters are parsed using token structure, not raw parenthesis counting.

## Errors and ownership

Unknown escapes, incomplete interpolation, and malformed bodies are lexical/parse diagnostics. [Source snippets and locations](../diagnostics/source-snippets-and-locations.md) renders their ranges; the semantic layer does not repair malformed source. Runtime string representation and hashing belong to the runtime/value boundary.
