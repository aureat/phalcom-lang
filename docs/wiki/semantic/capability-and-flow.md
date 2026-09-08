# Capability and flow

> Sources: semantic-analyzer chapters 05–08, 11; SEMA004
> Raw: [semantic specification and program snapshot](../raw/semantic/2026-09-08-semantic-sources.md)
> Updated: 2026-09-08

Capability and flow analysis tracks what a program is allowed to do and what a control path establishes. Binding contracts, current knowledge, assignment, joins, loops, predicates, fields, and callable returns are distinct products.

## Reconciliation

A type test or predicate can contribute narrowing evidence only through a recognized semantic relation. Branches join their facts; loops summarize entry/exit and must preserve the declared authority of effects and outcomes. An advisory observation cannot satisfy a formal capability obligation.

## Consumers

The compiler consumes formal products for lowering, the editor consumes projections for presentation, and diagnostics consumes owned error reasons. [Compiler/VM](../runtime/compiler-vm-boundary.md) does not duplicate flow rules. SEMA004 is in progress and partial.
