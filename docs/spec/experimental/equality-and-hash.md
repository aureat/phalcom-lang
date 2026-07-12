# Equality & hash ladder (proposed — untracked soundness gap)

- Status: Proposed · no open-Q covers this; **soundness teeth**
- Axis: values (equality ladder), object-model §8

> **Partially superseded (2026-07-12).** This note heavily overlaps the now-landed
> [ADR-0023](../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md) and
> [core/decisions.md](../core/decisions.md) Q1/Q5 — it is a **candidate to become
> normative**. Also: NaN reasoning that started from "`Number` is f64" predates
> [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md); under the
> split, NaN lives **only on `Float`** (`Int` is exact), so NaN-specific claims now
> read `Float`, not `Number`. Index: [deferred-work.md](../deferred-work.md).

## Problem

`Object` defines identity `==`/`hash`, but the value-type ladder is undefined, and
two concrete hazards follow from committed decisions:

1. **`Float` is f64 (ADR-0005; split per ADR-0024) → `nan == nan` is `false`.** Breaks `==`
   reflexivity and makes a NaN `Map`/`Set` key unfindable.
2. **A mutable instance used as a `Map`/`Set` key, then mutated,** silently
   corrupts the table (its bucket no longer matches its `hash`).

## Decision

**A fixed equality ladder + a stated `hash`/`==` contract.**

- **Contract:** `a == b ⇒ a.hash == b.hash`; `==` is reflexive, symmetric,
  transitive; `hash` is stable for the lifetime a value is used as a key.
- **Identity types** (`==` is identity): user instances by default, `Block`,
  `Method`, `Fiber`, `Future`, `Module`, `Class`.
- **Value types** (`==` is structural): `Number`, `String`, `Symbol`, `Bool`,
  `Tuple`, `List`, `Range`, `Option`; `Map`/`Set` structural over entries.
- **NaN rule:** IEEE `==` is preserved for arithmetic (`nan == nan` → `false`),
  but **hash-keying canonicalizes** — `Map`/`Set` use a total key-equality
  (`sameValueZero`: all NaNs equal, `-0 == +0`) so keys stay findable. The `==`
  *message* and the *hashing predicate* are deliberately distinct.
- **Mutable-key rule:** only values whose `hash` is stable may key a `Map`/`Set`.
  A mutable user instance keys **by identity** (default `hash`), which is stable;
  overriding `==`/`hash` to be structural on a mutable class is the author's
  contract to keep the key immutable-while-used. Documented, not enforced.

## Precludes

- A single fused "`==` also drives hashing" model — NaN forces the split between
  the arithmetic `==` message and the container key-equality predicate. Accepted.
- Structural-by-default user instances (would make every field mutation a
  key-corruption risk). Identity default stands.
