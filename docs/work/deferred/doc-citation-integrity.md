# Deferred: nothing checks documentation citations (unowned)

Surfaced 2026-07-20 by a sweep of every `file:line` citation in
[`docs/forge/UNITS-TRACKER.md`](../forge/UNITS-TRACKER.md) against `e33e8e5`. Recorded here rather
than fixed because the fix is tooling, and because the sweep found a second failure that tooling
would **not** have caught — see item 2, which is the more important half.

Governed in spirit by [`docs/theory/00-provenance-and-citation-discipline.md`](../theory/00-provenance-and-citation-discipline.md).
That file states the rules for *writing* a citation; nothing enforces them afterward.

---

## 1. `file:line` citations rot silently, and no check exists

**Measured, not estimated.** Of 24 citations in `UNITS-TRACKER.md`, **15 had drifted.** Worst
cases:

| Cited | Actual at `e33e8e5` | Drift |
|---|---|---|
| `dispatch.rs:870-885` (`VM::variadic_selector_cache`) | `:477-483` | ~390 lines |
| `vm/mod.rs:101,136,172,194` (the four class maps) | `:173,238,274,296` | ~70–100 |
| `core.ph:309` (`Iterable` root) | `:646` | ~340 |
| `core.ph:1205-1281` (lazy views) | `:1366-1443` | ~160 |
| `vm/dispatch.rs:398-427` (`invoke_at`) | `:433`, probe `:445` | ~35 |
| `vm/mod.rs:116` (`world_version`) | `:203` | ~87 |

The nine that survived were almost all struct definitions near the top of a file
(`interner.rs:10`, `heap/class.rs:25`, `chunk.rs:10-18`, `value/mod.rs:121`, `lexer.rs:216`). That
is the pattern: **citation durability is a function of distance from the top of the file**, not of
how carefully it was written.

**Why it is worse than it sounds.** A drifted citation does not look broken. The file is still
right, the surrounding prose is still true, and a reader who spot-checks one reference and lands
in roughly the right area concludes the row is sound. This is the failure mode
`docs/theory/00` names: a mostly-true record acting as a credential for its wrong parts.

**Observed propagation, one session, three artifacts:** a stale range in `UNITS-TRACKER.md` →
copied into a project memory file → cited into a newly written analysis doc. It was caught only
because the analysis carried a provenance ledger that forced each delegated citation to be
re-opened. Nothing structural stopped it at any of the three hops.

**Direction (not a ruling).** Two options, cheapest first:

- **(a) Prefer symbols to lines, and stop there.** Write `grep "fn invoke_at"` or
  `pub struct ClassKey` instead of `dispatch.rs:433`. Greppable, survives every refactor short of
  a rename, needs no tooling. Reserve `file:line` for cases where a *specific line's content* is
  the point (a magic constant, a one-line guard), and treat it as perishable. `UNITS-TRACKER.md`'s
  header now says this.
- **(b) A checker.** A script that extracts every `` `path:line` `` from `docs/**/*.md`, confirms
  the path exists, and warns when the file's mtime or last-touching commit postdates the doc's.
  It **cannot** verify the line says what the prose claims, so it would catch staleness, not
  wrongness. Worth roughly what it costs — which is small; it is a regex and a `git log`.

Option (a) is the real fix and needs no unit. (b) is optional and should not be built until (a)
has been applied to the docs that move fastest.

**Scope if anyone does (b):** `docs/forge/UNITS-TRACKER.md`, `docs/forge/DEFERRED.md`, and
`docs/spec/current/**` carry the most citations and move the fastest. `docs/adr/` is frozen and can
be skipped.

## 2. Trackers go wrong about *whether things exist* faster than about *where they are*

**This is the finding that matters, and no linter would have caught it.**

The same sweep found two `UNITS-TRACKER.md` rows whose **status** was false, not merely their line
numbers. `U-CLASSNS` and `U-CLASSCLOSE` both read *"design only, zero code"* while both had fully
landed — `d3b6cd2` / `8b4465c` / `14cdfb9`, then `7c2cfab`. One of them had shipped a
user-visible language change (classes can no longer be reopened) days earlier.

This is a **repeat**. That file's own header already records the identical failure, dated one day
prior:

> "the 2026-07-13 pass had gone ~60 commits stale and three rows were materially wrong: `U-SEQ`
> and `U-STRING` were marked not-started but had fully landed, and `U-IC`/`U-HOTPATH` were marked
> not-started/blocked while each is partially built."

Twice in eight days, same file, same direction — **always understating what exists, never
overstating.** That asymmetry is diagnostic. Rows are written when work is *planned* and updated
when someone *remembers*; landing a unit has no step that touches the tracker, so the error can
only ever run one way.

**Direction (not a ruling).** The asymmetry suggests the fix is not "audit more often":

- The cheap version is a **convention**: a unit is not done until its tracker row is flipped in
  the same commit that lands it. Costs nothing, and puts the update where the information is.
- The mechanical version is to **derive** the checkbox rather than assert it — a row that names
  its landing commits could have its `[x]`/`[ ]` computed by asking git whether those commits are
  reachable. That inverts the drift the way `VM::collect_roots`'s exhaustive destructure inverted
  the GC-roots table (`docs/spec/current/memory-management.md` §2.1): the doc stops being able to
  disagree with the tree silently.

Neither is scoped. The convention is worth adopting immediately regardless; it is free.

**Related standing rule:** never trust a stored claim about landed state — diff against `main`.
That rule already exists in project memory precisely because a note claiming work was "unmerged,
on branch X" was wrong in every particular and produced a bogus at-risk-work report. This is the
third instance of the same class.
