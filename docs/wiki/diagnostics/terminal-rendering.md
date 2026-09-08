# Terminal rendering

> Sources: traceback color/output specifications; diagnostic style model
> Raw: [diagnostic specification and program snapshot](../raw/diagnostics/2026-09-08-diagnostics-sources.md)
> Updated: 2026-09-08

Terminal rendering maps semantic roles to glyphs, ANSI styles, and width-aware layout. `ColorMode` distinguishes automatic, always, and never; plain mode selects ASCII glyphs and disables automatic color, while explicit always-color remains authoritative.

## Layout

The default width is a display-column policy, not a byte count. Glyph sets cover rails, corners, underlines, arrows, ellipses, and severity markers. Styling is centralized in a styler so report assembly remains protocol-neutral.

[Tooling output](../tooling/repl-commands.md) selects CLI policy; [editor products](../editor/semantic-editor-products.md) should not embed terminal escapes.
