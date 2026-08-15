# 44. `Option` bootstrap formalization; defer niche-encoding

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0007](0007-option-as-abstract-with-some-none.md) (abstract
  `Option` + `Some`/`None`), [ADR-0010](0010-tagged-value-enum.md) (tagged
  `Value` enum — the representation a niche would change), [ADR-0009](0009-handle-arena-heap.md)
  (handle heap — where wrapped `Some` payloads may point), `docs/spec/current/values-and-absence.md` §3
  (surface `None` vs private `nil`, Invariant 4), `docs/spec/current/open-questions.md` Q13,
  `../../forge/units/U17/u17.md`, `docs/forge/STATE.md` (DEC-U17 resolution record)

> **Amended 2026-08-11 by PDR-0033.** The bootstrap formalization remains, but
> `None` is now an immediate value with no `none_singleton` handle and `Some` is
> represented by bounded immediate variants. The deferred physical-encoding
> question remains open.

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
`Option`/`Some`/`None` shipped in U6/U-STD. `None` is an **immediate value** with
no heap handle, and the `None` class **has no instance fields**. `Some` is also
immediate, with bounded `Some1`…`Some7` variants. Because neither variant uses
instance-field defaulting, the bootstrap cycle Q13 warns about does not arise.
The remaining physical encoding question is deferred by PDR-0033.

## Decision

### 1. Formalize the bootstrap resolution (documentation)

Record as ratified invariants (already true in the tree, now ADR-anchored):

- **`None` is a fieldless immediate variant.** `None == None` holds by value;
  constructing/referencing it allocates nothing (values-and-absence §3.1). It is
  represented separately from the `None` class object.
- **No bootstrap cycle.** Because `None` has no fields, the field-default rule
  never re-enters `None` construction. The correctness concern in Q13 is closed.
- **`nil` stays private and distinct from `None`** (Invariant 4). The internal
  `Value::Nil` sentinel and the surface immediate `None` value are **not** the
  same value and must never become confusable — a point any future representation change is
  bound by.

### 2. Defer final physical encoding — **DEC-U17 = A (defer)**

**Do not finalize NaN-boxing, pointer tagging, or a niche layout now.** The
correctness-first immediate `Option` substrate is admitted by PDR-0033; final
physical encoding remains a later representation pass.

- The final encoding is a **pure performance optimization behind the unchanged
  `Option` surface** and can be deferred without changing user-visible semantics.
- It belongs with the other representation-level speed work (NaN-boxing / tagged
  `Value` compaction, ADR-0010), which is not yet scheduled. Doing it in
  isolation now would commit a `Value`-layout choice ahead of that broader pass.
- **Guardrails on any future encoding** (binding on the deferred work): `None`
  stays value-comparable and allocation-free; the encoding must not
  make `Value::Nil` and surface `None` confusable (Invariant 4); `Some`/`None`/
  `match`/combinators (U6/U-STD) stay observationally identical.

U17's bootstrap ruling remains documentation; PDR-0033 supplies the immediate
runtime substrate while leaving final physical encoding deferred.

## Consequences

- **Positive.** The `Option` bootstrap story is written down and no longer an
  open question; the fieldless-immediate design that avoids the cycle is
  protected by an ADR and PDR rather than left implicit in `value.rs`.
- **Positive.** The `Value` layout stays free for a single coherent
  representation pass (niche + NaN-boxing together) instead of a piecemeal
  commitment now.
- **Negative / accepted.** The final bit-level `Option` encoding remains deferred
  — a measured-later representation cost, explicitly the thing the deferred
  niche would remove.
- **Revisit trigger.** The representation/speed pass (NaN-boxing or a `Value`
  compaction) — at which point niche-encoding `Option` is designed and landed
  together with it, under the guardrails above. A superseding ADR records it.
