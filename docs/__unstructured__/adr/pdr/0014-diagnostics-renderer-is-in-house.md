# PDR-0014 — The diagnostics renderer is in-house; miette leaves the workspace

- Status: Accepted
- Date: 2026-07-20
- Related: [`docs/spec/current/traceback/implementation-spec.md`](../spec/current/traceback/implementation-spec.md)
  §1 (the full grounds and the architecture this record commits to),
  [`docs/spec/current/traceback/color.md`](../spec/current/traceback/color.md) (the palette discipline that
  drove the choice), [PDR-0010](0010-errors-carry-structure-and-cheap-origin.md) (what errors
  carry; this record governs how they print)

## Context

`miette` has been a declared workspace dependency with **zero imports** since it was added.
`CLAUDE.md` names "thiserror + miette" as the error convention; the miette half has always been
aspirational. This phantom has already produced a wrong decision once — decision 0066 asserted
diagnostics were "rendered as miette labels" and had to be amended (`bb4f365`). Meanwhile the
traceback specification (`docs/spec/current/traceback/`) fixes hard rendering commitments: 13 semantic
color roles, 16-ANSI-indices-only, no backgrounds, a shared ASCII-fallback axis, JSON streams
that force color off, and surfaces (traceback frame lines, fiber-switch log, recursive
disassembly, JSON event streams) that are not diagnostics in miette's `Report` sense at all.

## Decision

The diagnostic renderer is **built in-house**: a semantic-role style layer
(`diagnostics/style.rs`), a caret/snippet renderer (`diagnostics/caret.rs`), and per-surface
formatters, per implementation-spec §1–§3. **`miette` is removed from the workspace**, and
`CLAUDE.md`'s convention line is corrected to "thiserror" alone when the substrate unit (plan
T2) lands. The catalog's `╭─ │ · ╰──` visual style is kept — implemented, not imported.
`color-print` is an interim implementation detail that leaves the tree when the last
`ceprintln!` call site migrates to the style layer.

Grounds (condensed; full version in the spec):

1. Most rendering surfaces are not miette-shaped — adopting it still means owning a second
   renderer for frames, logs, disasm, and JSON. Two systems permanently.
2. The `color.md` discipline is a hard spec that is one line each to *own* and a fight each to
   *impose* on `GraphicalReportHandler`.
3. miette's genuinely hard part — multi-label spans — is bounded here at two labels per
   snippet.
4. Golden fixtures assert fields, not bytes, so the corpus-migration cost cuts equally both
   ways and decides nothing.

## Consequences

- Do not re-add miette (or another report-rendering crate) citing the old convention; this
  record supersedes that convention. Renderer changes go through the traceback spec.
- All SGR emission concentrates in `Styler::paint`; raw escapes or markup outside
  `diagnostics/style.rs` are a review-blockable defect once T2 lands.
- The rendering substrate is testable as plain string functions (the reason the caret
  arithmetic can be unit-tested at all).

## Alternatives rejected

- **Adopt miette for real.** Free multi-label spans, help/note slots, severity — at the cost of
  a permanent second renderer for the non-diagnostic surfaces and a theming fight with the
  palette discipline.
- **Keep the dependency dormant.** The status quo, and the documented source of a wrong
  premise. A declared-but-unused dependency is a standing invitation to "finish wiring it".
