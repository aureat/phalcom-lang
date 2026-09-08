# Compiler and VM boundary

> Sources: compiler structure/runtime programs; COMP001–COMP002
> Raw: [runtime specification and program snapshot](../raw/runtime/2026-09-08-runtime-sources.md)
> Updated: 2026-09-08

The compiler lowers authoritative semantic products into executable instructions; the VM evaluates those instructions and performs dispatch, allocation, calls, and errors. The boundary is intentionally narrow: the compiler must not reconstruct semantic type or flow facts from runtime observations.

## Trace

Language syntax becomes an AST, semantic analysis produces types/contracts/flow, lowering chooses bytecode, and VM execution applies the runtime object model. Native calls cross through the [canonical surface](../native/canonical-surface.md), while diagnostics are assembled by [report products](../diagnostics/report-model.md).

COMP001 and COMP002 are proposed; this page records the ownership contract.
