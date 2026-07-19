# Decisions

The home for Phalcom's architecture decision records. Replaces `docs/adr/`, which is
being migrated here.

## Why this exists separately

`docs/adr/` files decisions into `accepted/` / `proposed/` / `retired/` subfolders, so a
status change is a **file move**. That guarantees drift: the file moves and the index
doesn't, or the index flips and the file's own status line doesn't. The tree currently
carries several such mismatches (an ADR marked Superseded in its own body but Accepted in
the index; an index row with a literal `n` for its number and a filename that does not
exist).

This folder removes the cause:

1. **Flat directory.** Every decision is one file at the top level. Status is never
   encoded in a path, so a status change is never a file move.
2. **One tracker.** [`STATUS.md`](STATUS.md) is the only index. Status lives in the
   decision file's own header *and* its tracker row — those two, and nothing else.
3. **Numbering continues from the ADR sequence.** The last ADR is 0064, so the first
   decision here is 0065. Migration is then a pure `git mv` with no renumbering and no
   collisions. (Note: 0034 does not exist — the ADR sequence skips it.)

## Writing one

Header block, then prose:

```markdown
# 65. Short imperative title

- Status: Accepted
- Date: YYYY-MM-DD
- Supersedes: ADR-0026 (one clause on what specifically is reversed)
- Related: <specs, units, other decisions>

## Context
## Decision
## Consequences
## Alternatives considered
```

Ground claims in `file.rs:line` or a reproduction. "Verified" without evidence is not
verified.

## Maintenance rules — binding

1. **Two-way sync.** Change a decision's status line → change its `STATUS.md` row in the
   same edit. Change a row → change the file. Never let them disagree after your edit.
2. **Record supersession when it is ruled**, not later. The superseded decision gets a
   dated callout and a status flip; the superseding one names it in its `Supersedes:`
   header; the tracker's Superseded-by column names it too.
3. **Shipped is independent of status.** A decision can be Accepted and unimplemented, or
   implemented and later reversed. Track them separately, and only mark shipped with
   evidence you actually produced.
4. **Don't guess.** Unverified stays `?`.

## Migration status

`docs/adr/` is still authoritative for ADR-0001…0064 and keeps its own `STATUS.md` until
those files move here. Both trackers are live in the meantime; a decision recorded here
that supersedes an ADR must update the ADR's row in *both* places.
