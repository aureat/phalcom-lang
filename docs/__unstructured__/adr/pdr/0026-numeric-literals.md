# PDR-0026 — Numeric literal grammar: radix prefixes, separators, decimal exponents, no `n` suffix

- Status: **Accepted** (ratified 2026-07-21)
- Depends on: [ADR-0024](../adr/accepted/0024-numeric-surface-split-int-float-and-division.md), [PDR-0012](0012-numeric-tower-implementation-and-floor-amendment.md)
- Spec: [`docs/spec/library/numbers/numeric-literals.md`](../spec/library/numbers/numeric-literals.md)

## Decision

1. `Int` literals are exact and unbounded: decimal (`100`), binary (`0b1101`), octal (`0o755`), and hexadecimal (`0xFE`). Prefix and hexadecimal digits are case-insensitive.
2. Decimal `Float` literals require either a fractional digit sequence (`3.14`, `.25`) or an exponent (`2e10`, `6.02e-23`). `5.` and `5.e2` are invalid. This preserves unambiguous dot sends: `5.foo` is always a send and `5..2` is always a range.
3. A decimal exponent always produces `Float`; base-prefixed literals accept neither decimal point nor exponent.
4. `_` groups digits. One underscore is permitted between digits of the same radix; one is also permitted immediately after a base prefix (`0x_FF`). All other placements are lexical errors.
5. There is **no `n` suffix**. `LargeInt` is an implementation tier of public `Int`, not a public `BigInt` type. A suffix would have no observable meaning and would violate normalization by forcing a representation choice.
6. Invalid numeric text is one lexical diagnostic, never a sequence of plausible tokens. This includes invalid base digits, missing digits after a prefix or exponent, and invalid separators.
7. A decimal integer with a non-zero leading digit after an initial zero is rejected (`0123`); use an explicit base prefix. Zero-only decimal forms remain valid.

## Consequences

- Lexer/parser literals retain the `Int`/`Float` distinction required by PDR-0012. Oversized integers carry normalized digits plus radix to the compiler, which constructs `BigInt` without making `phalcom-ast` depend on `num-bigint`.
- IEEE overflow while decoding a syntactically valid decimal float produces `±infinity`; malformed syntax is still a lexical error.
- Complex literals, imaginary suffixes, NaN/Infinity spellings, hex floats, and user-visible fixed-width integers remain deferred.

## Alternatives rejected

- **JavaScript `n`.** JavaScript needs it because `bigint` is a separate public type. Phalcom has one public exact `Int`.
- **Trailing-dot floats.** Python/JavaScript-style `5.` conflicts with Phalcom's central dot-send syntax for negligible readability benefit.
- **Implicit octal.** Explicit `0o` prevents leading-zero ambiguity.
