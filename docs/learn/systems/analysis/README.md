# Analysis — the systems book

This folder is the requirements and design record for the systems book: a multi-file
markdown curriculum covering seven systems topics (concurrency, memory, coordination,
execution engines), written for one specific reader on one specific machine. The book's
content docs will live one level up, in `docs/learn/systems/`, as `NN-slug.md` files in
reading order with `00-map.md` as the entry point. Nothing in this folder is book content;
everything in it governs how book content gets written.

## Status

| Phase | State |
|---|---|
| Requirements | **Complete** — R1–R11, I1–I5, Q1–Q7, D1–D5 ruled (see `requirements.md`) |
| Scope | **Frozen** — core seven in, three gated, cuts normative (see `scope.md`) |
| Machine probes | First pass measured 2026-07-20; open probes listed (see `machine.md`) |
| Pilot (`01-event-loop.md`) | Not started — awaiting "proceed" |
| Remaining six docs | Blocked on pilot serving as exemplar |
| Visualization track | Fully deferred — planned later as its own effort |

## Files

- **`requirements.md`** — the learner profile, explicit and implicit requirements,
  quality gates that block a doc from "done", ruled decisions, and risks.
- **`scope.md`** — the normative in/gated/cut inventory, the reading order with its
  dependency rationale, and the book-level through-line.
- **`anatomy.md`** — the per-document contract: section obligations, the three recurring
  block types (concept, recall, at-the-machine), diagram policy, code-language policy,
  and register rules.
- **`machine.md`** — the measured fact sheet of the target machine (Apple M1 Pro) and
  the per-doc inventory of machine-grounded experiments.
- **`ideas.md`** — the idea ledger: every seed, anchor pairing, and demo concept ideated
  during requirements analysis, tagged by warrant so recalled claims get re-verified
  before they enter a doc.

## Provenance discipline

Facts in this folder are tagged where it matters: **[measured]** means observed on the
target machine on a stated date with the command shown; **[documented]** means stated by
authoritative documentation but not yet demonstrated here; **[recalled]** means from
training/memory and must be verified before appearing in a book doc. A book doc may only
assert what is [measured] or [documented]-and-cited; [recalled] items are work orders,
not facts.
