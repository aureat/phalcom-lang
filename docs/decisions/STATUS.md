# Decision status tracker

One row per decision in this folder. Binding maintenance rules are in
[`README.md`](README.md) — the short version: status lives here **and** in the decision
file's own header, never anywhere else, and the two are changed in the same edit.

**Status** is Proposed / Accepted / Retired (Retired covers superseded and deferred).
**Shipped** is whether the design is implemented in the tree, independent of paper status —
the two drift apart in both directions in this repo. `✅` = code-verified this pass, with the
evidence named; `❌` = verified absent; `—` = nothing to ship; `?` = not checked, do not
assume either way.

Numbering continues the ADR sequence (last ADR is 0064) so migration is a `git mv` with no
renumbering. ADR-0001…0064 remain tracked in [`../adr/STATUS.md`](../adr/STATUS.md) until
they move here.

| # | Title | Status | Supersedes | Superseded by | Shipped |
|---|---|---|---|---|---|
| [0065](0065-classes-are-closed.md) | Classes are closed: remove class reopening | Accepted | ADR-0026 (Axis 1) | | ❌ ruled 2026-07-19, unimplemented |

## Cross-tracker obligations

A decision here that supersedes an ADR must update the ADR in **both** places — the ADR
file's own status line, and its row in `../adr/STATUS.md`.

- **0065 → ADR-0026**: done 2026-07-19. ADR-0026 flipped to Retired in
  `../adr/accepted/0026-class-hierarchy-mutability.md` and in `../adr/STATUS.md`. The file
  stays in `accepted/` pending migration — its own status line is authoritative, not its
  path, which is precisely the ADR-layout defect this folder exists to remove.

## Known defects in the ADR records (fix during migration, not before)

Found while writing 0065. Recorded so migration does not have to rediscover them:

- `../adr/README.md` carries a row with a literal `n` for its number, linking
  `accepted/n-class-hierarchy-mutability.md` — a file that does not exist. The real file is
  `0026-class-hierarchy-mutability.md`.
- ADR-0014 is marked Superseded in its own body but listed Accepted in `../adr/README.md`.
- `../adr/README.md` and `../adr/STATUS.md` are two indexes over the same set and disagree.
  One should not survive migration.
