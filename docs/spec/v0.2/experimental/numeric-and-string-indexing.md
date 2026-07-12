# Integer ops & string indexing on flat `Number` (proposed — untracked gap)

- Status: Proposed · extends open-Q2 (which tracks only the *surface* split)
- Axis: values (numeric model)

> **Partially superseded (2026-07-12).** The numeric-representation premises here are
> superseded by [ADR-0024](../../../adr/0024-numeric-surface-split-int-float-and-division.md):
> `Int` is now exact and unbounded (auto-promoting bignum), `Float` is a distinct type,
> and the surface split is **decided** — so this doc's "one flat f64 / safe-integer
> boundary 2⁵³ / no bignum in Draft 0.1 / split stays open (open-Q2)" claims are now
> **false**. The *integral-index* and *codepoint-string* decisions still stand.
> Superseded lines are annotated inline below. Index: [deferred-work.md](../deferred-work.md).

## Problem

`Number` is one flat f64 (ADR-0005). <!-- SUPERSEDED by ADR-0024: `Int` is exact/unbounded, `Float` distinct; not one flat f64. --> Undefined: what indexes a `List`/`String`,
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
  <!-- SUPERSEDED by ADR-0024: `Int` is exact and unbounded (auto-promoting bignum); no 2⁵³ boundary, bignum is not deferred. -->
- **String indexing is by Unicode scalar (codepoint), not byte.** `s[i]` →
  `Option<String>` (a one-scalar string), `s.length` counts scalars. Byte access
  is an explicit separate API. This keeps indices stable under the UTF-8
  representation without exposing byte offsets.

## Precludes

- A distinct `Integer` runtime type *underneath* (still one f64) — the surface
  split (open-Q2) stays open, but code must target the abstract numeric protocol,
  not assume f64, so an `Int`/`Float` split isn't foreclosed.
  <!-- SUPERSEDED by ADR-0024: the split is now RESOLVED — `Int` (exact/unbounded) + `Float` are distinct runtime types; open-Q2 is closed. -->
- O(1) `String` random indexing — codepoint indexing over UTF-8 is O(n) unless a
  break table is added later; accepted for Draft 0.1.
