# Lexical structure

> Sources: `docs/spec/current/lexical-structure.md`
> Raw: [language specification and program snapshot](../raw/language/2026-09-08-language-sources.md)
> Updated: 2026-09-08

Lexing owns token boundaries and newline suppression; it does not decide semantic types. Statements are newline-terminated and semicolons are optional. A newline is suppressed when the preceding token cannot end a statement, so parser lookahead does not implement automatic semicolon insertion.

## Token families

Regular identifiers, source fields, implementation fields, implementation selectors, and the standalone positional marker are distinct lexical classes. The distinction is intentional: field ownership and privileged implementation naming are not inferred from a trailing underscore.

Comments are line or block comments. Numeric literals admit digit separators. Strings are UTF-8 text with a dedicated interpolation escape. Tuples, lists, maps, and blocks are distinct primary forms; `Set(...)` is a send rather than a set literal.

## Brace disambiguation

In expression position, one token of lookahead distinguishes a map from a block:

| Lookahead | Form |
| --- | --- |
| `IDENT :` | map literal |
| `IDENT ,` or `IDENT =>` | parameterized block |
| `}` | empty block |
| other | zero-parameter block |

The empty map is `Map()`; `{}` is an empty block. A brace beginning a statement is invalid.

## Boundary

This article records syntax rules. [Semantic type formation](../semantic/type-formation-and-inference.md) decides what parsed expressions mean, while [runtime representation](../runtime/representation-and-object-model.md) decides how values are stored.
