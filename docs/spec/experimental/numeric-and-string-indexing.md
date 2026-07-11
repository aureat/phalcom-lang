# Integer ops & string indexing on flat `Number` (proposed — untracked gap)

- Status: Proposed · extends open-Q2 (which tracks only the *surface* split)
- Axis: values (numeric model)

## Problem

`Number` is one flat f64 (ADR-0005). Undefined: what indexes a `List`/`String`,
how bitwise ops behave, and precision past 2⁵³. `list[2.5]`, `1 << 3`, and
`String` character access all depend on answers not yet written.

## Decision

- **Index/count values must be integral.** `list[i]` requires `i.isInteger`
  (`i.floor == i`, finite); otherwise `RangeError`. No silent truncation — a
  fractional index is a bug, not a round.
- **Bitwise ops** (`&`, `|`, `^`, `<<`, `>>`) coerce operands to a 32-bit integer
  view (ToInt32-style), matching JS; result is a `Number`. Out-of-range shifts
  wrap the count mod 32.
- **Safe-integer boundary:** integer identity holds only ≤ 2⁵³−1. Past that,
  arithmetic is defined but lossy; no separate bignum in Draft 0.1.
- **String indexing is by Unicode scalar (codepoint), not byte.** `s[i]` →
  `Option<String>` (a one-scalar string), `s.length` counts scalars. Byte access
  is an explicit separate API. This keeps indices stable under the UTF-8
  representation without exposing byte offsets.

## Precludes

- A distinct `Integer` runtime type *underneath* (still one f64) — the surface
  split (open-Q2) stays open, but code must target the abstract numeric protocol,
  not assume f64, so an `Int`/`Float` split isn't foreclosed.
- O(1) `String` random indexing — codepoint indexing over UTF-8 is O(n) unless a
  break table is added later; accepted for Draft 0.1.
