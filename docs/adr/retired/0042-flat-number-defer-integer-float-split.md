# 42. Flat `Number` now; defer the `Integer` / `Float` split

- Status: **Superseded by [ADR-0024](../accepted/0024-numeric-surface-split-int-float-and-division.md)** (ruled 2026-07-14)
- Date: 2026-07-12

> **Superseded (2026-07-14) by [ADR-0024](../accepted/0024-numeric-surface-split-int-float-and-division.md).**
> This ADR and ADR-0024 were both accepted ~12h apart on 2026-07-12 without either
> citing the other — a genuine unreconciled contradiction, not a sequenced decision.
> This ADR's own §"Related" lists ADR-0012/ADR-0009 only and never mentions ADR-0024
> or ADR-0005, and its Consequences anticipated "a later split arrives as an additive
> amendment ADR (superseding this one)" without knowing one already existed. The user
> has now ruled directly: **the split (ADR-0024) is Phalcom's committed numeric
> surface.** `DEC-U12 = A` below is reversed. The code (`class Number {}`, flat) has
> not yet been updated to match ADR-0024 — this is unbuilt, not merely undocumented.
- Related: [ADR-0012](../accepted/0012-selector-signature-encoding-and-dispatch.md)
  (one-hashmap-probe dispatch — a later numeric-type split must not add a
  dispatch axis), [ADR-0009](../accepted/0009-handle-arena-heap.md) (handle heap — leaves a
  future boxed-bignum / tagged-int representation implementable without
  disturbing existing references), `docs/spec/current/object-model.md` (the
  `Number` core class), `docs/forge/units/U12/plan.md`,
  `docs/forge/STATE.md` (DEC-U12 resolution record)

## Context

[U12](../forge/units/U12/plan.md) asks whether Phalcom's numeric surface should
stay a **single flat `Number`** (one class, `f64`-backed, as the tree ships
today) or split now into a `Integer` / `Float` (or a wider numeric tower)
hierarchy with distinct representations and promotion rules.

This was **BLOCKED-ON-DECISION** in the U12 work order (DEC-U12). A split is a
one-way-ish surface commitment: once user code can observe `1` as an `Integer`
distinct from `1.0` as a `Float` — distinct classes, distinct `is_a`, distinct
overflow/precision semantics — removing or merging that distinction later is a
breaking change, whereas *adding* it later to a flat `Number` is additive for
most programs.

## Decision

**DEC-U12 = A — keep a single flat `Number` (f64) for v0.2; defer the
`Integer` / `Float` split.** Resolved by orchestrator autonomous authority,
2026-07-12 (the architect-recommended conservative option; reversible
pre-release, per the standing delegated-decision protocol).

- No runtime change lands with this unit. `Number` remains the sole numeric
  class, `f64`-backed, exactly as bootstrapped in `core.ph`.
- The split is **not precluded.** The handle heap (ADR-0009) and the
  signature-keyed dispatch (ADR-0012) both leave a future `Integer`/`Float`
  representation — including a tagged small-int or boxed bignum — implementable
  without perturbing existing object references or adding a dispatch axis.
- U12 is therefore a **tiny affirm-ADR unit**: it records the ruling and its
  reversibility, and adds no code.

## Consequences

- **Positive.** No premature commitment to promotion/coercion rules,
  overflow behavior, or literal typing that v0.2 has no motivating use for.
  Selector identity and the numeric primitive surface stay minimal.
- **Positive.** A later split arrives as an additive amendment ADR (superseding
  this one for the numeric axis) rather than a breaking redesign.
- **Negative / accepted.** Programs that would want exact integer arithmetic
  (arbitrary precision, no `f64` rounding at 2^53) do not get it in v0.2. This
  is acceptable for the current spec scope and is the explicit thing the future
  split would address.
- **Revisit trigger.** A concrete v0.x feature that needs exact-integer
  semantics, distinct numeric `is_a`, or representation-sensitive performance —
  at which point a superseding ADR designs the tower against real requirements.
