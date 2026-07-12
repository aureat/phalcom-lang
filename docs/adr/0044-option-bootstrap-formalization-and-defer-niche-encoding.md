# 44. `Option` bootstrap formalization; defer niche-encoding

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0007](0007-option-as-abstract-with-some-none.md) (abstract
  `Option` + `Some`/`None`), [ADR-0010](0010-tagged-value-enum.md) (tagged
  `Value` enum — the representation a niche would change), [ADR-0009](0009-handle-arena-heap.md)
  (handle heap — where the `None` singleton lives), `docs/spec/v0.2/values-and-absence.md` §3
  (surface `None` vs private `nil`, Invariant 4), `docs/spec/v0.2/open-questions.md` Q13,
  `docs/forge/units/U17/plan.md`, `docs/forge/STATE.md` (DEC-U17 resolution record)

## Context

[U17](../forge/units/U17/plan.md) grounds in open-question **Q13**, which raised
two coupled worries about `Option`:

1. **Bootstrap cycle (correctness).** If instance fields default to `None`, and
   `None` were an ordinary class instance with fields, constructing `None` would
   need `None` to already exist — a bootstrap cycle.
2. **Representation (performance).** Every `.class` / `match` on an `Option`
   currently costs a heap fetch through the tagged `Value` enum (ADR-0010); a
   *niche encoding* of `Some`/`None` directly into `Value` could remove it.

**The correctness half is already resolved by the landed implementation.**
`Option`/`Some`/`None` shipped in U6/U-STD. `None` is a **VM-blessed heap
singleton** (`none_singleton`, `phalcom-core/src/value.rs`) — a zero-allocation,
identity-comparable instance — and the `None` class **has no instance fields**.
Because `None` carries no fields, the "fields default to `None`" rule never
forces `None`'s own construction to read a field, so **the bootstrap cycle Q13
warns about does not arise.** What remained for this unit was (a) to write that
down, and (b) to rule on the niche optimization.

## Decision

### 1. Formalize the bootstrap resolution (documentation)

Record as ratified invariants (already true in the tree, now ADR-anchored):

- **`None` is a blessed, fieldless, zero-allocation singleton.** `None == None`
  holds by **identity**; constructing/referencing `None` allocates nothing
  (values-and-absence §3.1). It is special-cased relative to ordinary classes.
- **No bootstrap cycle.** Because `None` has no fields, the field-default rule
  never re-enters `None` construction. The correctness concern in Q13 is closed.
- **`nil` stays private and distinct from `None`** (Invariant 4). The internal
  `Value::Nil` sentinel and the surface `None` object are **not** the same value
  and must never become confusable — a point any future representation change is
  bound by.

### 2. Defer niche-encoding — **DEC-U17 = A (defer)**

**Do not niche-encode `Option` into `Value` now.** Resolved by orchestrator
autonomous authority, 2026-07-12 (the plan's deferred-leaning recommendation;
reversible, per the standing delegated-decision protocol).

- Niche-encoding is a **pure performance optimization behind the unchanged
  `Option` surface** — it removes a heap fetch on `.class`/`match`, changing no
  user-visible semantics. It is therefore safe to defer and add later.
- It belongs with the other representation-level speed work (NaN-boxing / tagged
  `Value` compaction, ADR-0010), which is not yet scheduled. Doing it in
  isolation now would commit a `Value`-layout choice ahead of that broader pass.
- **Guardrails on any future niche** (binding on the deferred work): `None`
  stays identity-comparable and zero-allocation; the niche for `None` must not
  make `Value::Nil` and surface `None` confusable (Invariant 4); `Some`/`None`/
  `match`/combinators (U6/U-STD) stay observationally identical.

U17 is therefore a **small documentation unit with a deferred optimization** —
it adds this ADR (and the spec cross-reference in values-and-absence §3), and no
runtime change.

## Consequences

- **Positive.** The `Option` bootstrap story is written down and no longer an
  open question; the fieldless-singleton design that avoids the cycle is
  protected by an ADR rather than left implicit in `value.rs`.
- **Positive.** The `Value` layout stays free for a single coherent
  representation pass (niche + NaN-boxing together) instead of a piecemeal
  commitment now.
- **Negative / accepted.** `.class`/`match` on an `Option` keeps its heap fetch
  for v0.2 — a small, measured-later cost, explicitly the thing the deferred
  niche would remove.
- **Revisit trigger.** The representation/speed pass (NaN-boxing or a `Value`
  compaction) — at which point niche-encoding `Option` is designed and landed
  together with it, under the guardrails above. A superseding ADR records it.
