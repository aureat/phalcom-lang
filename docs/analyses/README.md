# `docs/analyses/`

**Feasibility and comparison studies.** Each file answers one question of the form *"can Phalcom
do X"* or *"should Phalcom borrow X from Y"*, grounded in both trees, and stops at an answer.

## What belongs here

- Studies of an external artifact against Phalcom's actual capability surface.
- "What would this cost / what does it preclude" investigations that precede a decision.

## What does not

| Kind | Goes to |
|---|---|
| a ruling | [`../decisions/`](../decisions/) (PDR) |
| a normative contract | [`../spec/`](../spec/) |
| a finding from work already under way | [`../design-notes/`](../design-notes/) |
| a measurement | [`../forge/perf-log/`](../forge/perf-log/) |
| a unit of work | [`../forge/units/`](../forge/units/) |

An analysis **never authorizes implementation.** If it concludes something should be built, the
next artifact is a PDR, not a branch.

## Rules

1. **Status line up front**, plus the Phalcom HEAD sha and the external artifact's version. A study
   without a baseline is unfalsifiable a month later.
2. **Citation discipline of [`../theory/00-provenance-and-citation-discipline.md`](../theory/00-provenance-and-citation-discipline.md)
   applies in full** — warrant tags (`[V]`/`[M]`/`[R]`/`[X]`/`[O]`) on every claim, and "verified"
   only alongside the artifact that was opened.
3. **A provenance ledger section**, naming what was read first-hand versus delegated. Delegated
   citations get spot-checked before they ship, and anything that fails the check is recorded as
   `[X]` rather than quietly corrected.
4. **Separate "is it expressible" from "is it worth doing."** They come apart, they have different
   shelf lives, and conflating them is how a snapshot gets read as a permanent verdict.

## Index

- [`hashbrown-in-phalcom.md`](hashbrown-in-phalcom.md) — could the SwissTable hash map be written
  in Phalcom? No, twice: four missing capabilities today, and a constant-factor inversion that
  survives closing all four. Notes that ~half of hashbrown's complexity dissolves under a
  handle-arena GC, and that ratified-but-unbuilt PDR-0012 does **not** close the gap.
