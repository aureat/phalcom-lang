# Source snippets and locations

> Sources: diagnostic snippets; shared source ranges; LSPX003
> Raw: [diagnostic specification and program snapshot](../raw/diagnostics/2026-09-08-diagnostics-sources.md)
> Updated: 2026-09-08

Source labels attach a primary or supporting span to source text. Ranges are half-open byte intervals, and display locations must respect UTF-8 scalar boundaries, tabs, Unicode display width, multiline labels, and focused windows around long lines.

## Ownership

[Shared source ranges](../runtime/selector-identity.md) are reusable identity primitives, not a diagnostic renderer. Diagnostics resolves text to a snippet; [editor source locations](../editor/source-locations.md) turns the same semantic source site into a protocol location.

Empty labels produce no snippet. Sorting by source position keeps output deterministic.
