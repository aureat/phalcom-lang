# Phalcom Diagnostics

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-diagnostics source snapshot](../raw/diagnostics/2026-09-07-phalcom-diagnostics.md)
> Updated: 2026-09-07

## Overview

`phalcom-diagnostics` is Phalcom's rendering substrate for structured diagnostics. It owns severity roles, source labels, caret snippets, Unicode-aware column arithmetic, glyph selection, ANSI styling, and report formatting while deliberately performing no semantic analysis.

## Reports and sections

The report layer models six severities and maps them to semantic rendering roles. A report can carry a title, optional code, source snippets, notes/help lines, and protocol-neutral sections. Sections have four kinds: explanation, guidance, context, and type trace; each selects its own heading/body styling.

`format_diagnostic` assembles these components in a stable order: headline, snippets, non-empty rich sections, then notes/help. Color emission is delegated to `Styler`, so report formatting does not duplicate ANSI policy.

## Source snippets

A `Snippet` renders one or more labeled source spans in a boxed layout. Labels carry a `SourceRange`, text, and primary/secondary kind. Primary and supporting spans use different semantic roles, and labels are ordered by source position.

The renderer resolves a source location from byte offsets, expands tabs, measures display width with Unicode-width information, and windows long lines around the focused span. It supports optional file names and multiline labels, while an empty label list renders no snippet.

## Style and terminal policy

The style layer defines a closed set of semantic roles, a small ANSI color set, composable text weights, and explicit `ColorMode` choices: automatic, always, or never. `GlyphSet` provides Unicode and ASCII repertoires for rails, corners, underlines, arrows, ellipses, and severity markers.

`RenderConfig` resolves color from CLI preference, plain mode, `NO_COLOR`, and TTY state. Plain mode selects ASCII glyphs and disables automatic color; explicit always-color remains authoritative. The default width constant is 80 display columns when no narrower context is supplied.

## Evidence and tests

The crate root re-exports the report, snippet, style, and label APIs. Tests cover no-color and forced-color report output, trace styling, Unicode/ASCII configuration, tab and display-width behavior, label ordering, and multiline rendering. Its manifest depends on `phalcom-common`, `unicode-width`, and `clap`.

## See Also

- [Phalcom Common](../common/phalcom-common.md)

