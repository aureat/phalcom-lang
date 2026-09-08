# Indexed access and ranges

> Sources: ranges/iteration and collections specifications; COLL003
> Raw: [collection specification and program snapshot](../raw/collections/2026-09-08-collections-sources.md)
> Updated: 2026-09-08

Indexed access separates strict programmer-error paths from safe total paths. Range values are half-open intervals with explicit bounds and are used for slicing, replacement, and iteration.

## Contract

A safe read returns an absence value when an index is outside the collection; a strict operation reports a diagnostic for an invalid index or range. Range validation owns ordering and bound checks, while individual collection products own whether an operation copies, mutates, or returns a view.

## Iteration

Range iteration and collection iteration share protocol concepts but have different termination obligations. [Traversal](traversal.md) owns eager/lazy boundedness; [diagnostics source locations](../diagnostics/source-snippets-and-locations.md) uses the same half-open interval terminology for source bytes.
