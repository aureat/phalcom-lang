# Result, error, and traceback surfaces

> Sources: error handling and traceback specifications; DIAG001
> Raw: [diagnostic specification and program snapshot](../raw/diagnostics/2026-09-08-diagnostics-sources.md)
> Updated: 2026-09-08

Result/error values and traceback reports are related but distinct. A `Result` transports success/failure through program code; an error/traceback product explains an exceptional path to a user or host.

## Boundary

The VM records frames and runtime causes. Semantic analysis owns static diagnostic causes. Diagnostics converts either owned cause into report sections and source labels without deciding whether a failure is recoverable. DIAG001 is in progress and partial.
