# Implementation Units — v0.2

Per-unit implementation specifications. Each **family** is a folder; each unit is
`<n>-<name>.md`. These describe *how a slice of the language was (or will be) built* —
grounded in the ADRs and the normative spec one level up.

## Families

### [`U-CORE/`](U-CORE/) — the core-library track
Kernel reflection, value protocols, collections, and errors, built in Phalcom over the
frozen primitive floor. Foundational rulings/census for this track live in
[`../core/`](../core/) (decisions, floor-census, forward-compat, invariant-requirements).

| Unit | Spec | Status |
|---|---|---|
| U-CORE-1 | [1-kernel-reflection](U-CORE/1-kernel-reflection.md) | ✅ landed (`03764e3`/`b1109c2`) — `hash`, `isA`, `Behavior` reflection, `Method < Function` |
| U-CORE-2 | [2-bool-and-option-residue](U-CORE/2-bool-and-option-residue.md) | mostly landed (`0da64d6`); verify/harden |
| U-CORE-3 | [3-callable-reflection](U-CORE/3-callable-reflection.md) | dispatch-ready — **next** |
| U-CORE-4 | [4-value-tostring](U-CORE/4-value-tostring.md) | dispatch-ready |
| U-CORE-5 | [5-collection-contract](U-CORE/5-collection-contract.md) | dispatch-ready |
| U-CORE-6 | [6-errors](U-CORE/6-errors.md) | dispatch-ready |

### `U/` — the language-spine units (forge track)
As-built specifications for the landed spine units (U1–U11, U-LIST, U-LEX, U-STD, U-FE):
what each unit implemented and how. Translated from the `forge/` planning record.
*(Being authored — see [deferred-work.md](../deferred-work.md) and `docs/forge/` for the
source planning material until these land.)*

## Convention
- `<n>-<name>.md` — numeric prefix orders the units within a family.
- A unit spec cites the ADR(s) and spec section it realizes, and records the commit(s)
  where it landed once built.
