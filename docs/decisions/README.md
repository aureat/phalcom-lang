# Phalcom Design Records (PDR)

The home for Phalcom's design decisions. A **PDR** is one ruled question: what was decided,
what it costs, and what it forecloses. Cited as `PDR-0001`, one file per record, numbered
from 0001 in this folder.

Replaces `docs/adr/`, which is frozen — see [Relationship to the ADRs](#relationship-to-the-adrs).

## Why this exists separately

`docs/adr/` files decisions into `accepted/` / `proposed/` / `retired/` subfolders, so a
status change is a **file move**. That guarantees drift: the file moves and the index
doesn't, or the index flips and the file's own status line doesn't. The ADR tree carries
several such mismatches — an ADR marked Superseded in its own body but Accepted in the index,
an index row with a literal `n` for its number linking a file that does not exist, and two
indexes (`README.md` and `STATUS.md`) that disagree with each other.

This folder removes the causes:

1. **Flat directory.** Every PDR is one file at the top level. Status is never encoded in a
   path, so a status change is never a file move.
2. **One tracker.** [`STATUS.md`](STATUS.md) is the only index. Status lives in the PDR's own
   header *and* its tracker row — those two, and nothing else.
3. **Own numbering from 0001.** PDR numbers are independent of ADR numbers. `PDR-0043` and
   `ADR-0043` are different documents; always write the prefix.

## Writing one

````markdown
# PDR-0001 — Short imperative title

- Status: Accepted
- Date: YYYY-MM-DD
- Supersedes: PDR-0000 / ADR-0000 (one clause naming what specifically is reversed)
- Amends: PDR-0000 (one clause)
- Related: <specs, units, other PDRs>

## Context
## Decision
## Consequences
## Alternatives rejected
````

Binding conventions for the body:

- **Ground every claim** in `file.rs:line`, a command's output, or a reproduction. "Verified"
  without evidence is not verified.
- **Name the cost explicitly.** A PDR that lists only benefits has not finished thinking. Use
  a "the cost, named plainly" sentence in Consequences.
- **State what the decision precludes** — which future feature, optimization, or invariant is
  now harder or closed. This is the check that stops a local ruling becoming a global regret.
- **Cite precedent with consequence.** Not "Ruby does X" but "Ruby does X, which forces Y."
- **Alternatives rejected is not optional.** Include "deferring the ruling" when the failure
  mode of not deciding is silent.

## Maintenance rules — binding

1. **Two-way sync.** Change a PDR's status line → change its `STATUS.md` row in the same edit.
   Change a row → change the file. Never let them disagree after your edit.
2. **Record supersession when it is ruled**, not later. The superseded record gets a dated
   callout and a status flip; the superseding one names it in its `Supersedes:` header; the
   tracker's Superseded-by column names it too.
3. **Shipped is independent of status.** A PDR can be Accepted and unimplemented, or
   implemented and later reversed. Track them separately, and only mark shipped with evidence
   you actually produced.
4. **Don't guess.** Unverified stays `?`.
5. **Never design on an unratified record.** If a plan depends on a Proposed PDR or ADR, say so
   in the plan and ratify first. Building against a Proposed record ratifies it by fait
   accompli, which is how a decision gets made without anyone deciding it.

## Relationship to the ADRs

`docs/adr/` is **frozen**: ADR-0001…0064 keep their numbers, their files, and their own
`STATUS.md`. Nothing is bulk-migrated, and the ~5000 `ADR-00NN` citations across the tree stay
valid.

An ADR is rewritten as a PDR only when it is *actually revisited* — reopened, superseded, or
materially amended. At that point it receives a fresh PDR number, the ADR's status line and
`../adr/STATUS.md` row are updated to point at the PDR, and the pairing is recorded below.

### ADR → PDR mapping

| ADR | Superseded / amended by | Scope |
|---|---|---|
| [ADR-0026](../adr/accepted/0026-class-hierarchy-mutability.md) | [PDR-0001](0001-classes-are-closed.md) | Axis 1 (reopening) only; Axis 2 (reparenting sealed) kept |

A PDR that supersedes an ADR must update the ADR in **both** places — the ADR file's own
status line, and its row in `../adr/STATUS.md` — and add a row here.

## Known defects in the ADR records

Recorded so a future reader does not rediscover them. These are **not** scheduled for repair;
the ADRs are frozen, and each is fixed only if that ADR is revisited.

- `../adr/README.md` carries a row with a literal `n` for its number, linking
  `accepted/n-class-hierarchy-mutability.md` — a file that does not exist. The real file is
  `0026-class-hierarchy-mutability.md`.
- ADR-0014 is marked Superseded in its own body but listed Accepted in `../adr/README.md`.
- `../adr/README.md` and `../adr/STATUS.md` are two indexes over the same set and disagree.
- The ADR sequence skips 0034 — there is no such record.
