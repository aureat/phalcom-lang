# Diagnostics

> Sources: diagnostic rendering specifications; DIAG001
> Raw: [diagnostic specification and program snapshot](../raw/diagnostics/2026-09-08-diagnostics-sources.md)
> Updated: 2026-09-08

The diagnostics domain owns protocol-neutral report products, terminal rendering, source snippets/locations, and result/error/traceback presentation. It neighbors [semantic](../semantic/overview.md) for diagnostic ownership, [editor](../editor/overview.md) for protocol adaptation, [tooling](../tooling/overview.md) for CLI output, and [runtime](../runtime/overview.md) for runtime errors.

## Reading order

- [Report model](report-model.md)
- [Source snippets and locations](source-snippets-and-locations.md)
- [Terminal rendering](terminal-rendering.md)
- [Result, error, and traceback surfaces](result-error-traceback.md)

DIAG001 is in progress and partial. Rendering evidence does not establish the semantic cause of a diagnostic.
