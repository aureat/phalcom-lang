# Type formation and inference

> Sources: semantic-analyzer chapters 01–08; SEMA001
> Raw: [semantic specification and program snapshot](../raw/semantic/2026-09-08-semantic-sources.md)
> Updated: 2026-09-08

Type formation composes expression evidence into a canonical type result. The analyzer separates language type, type knowledge/evidence, analysis status, causal invalidity, advisory runtime shape, identity, ranges, and presentation.

## Pipeline

The analyzer parses declarations and expressions, establishes binding/contracts, synthesizes or checks expression types, solves generic constraints, reconciles relation outcomes, and publishes callable/compound products. A failure can be invalid yet analyzable; a diagnostic is not automatically a reason to discard every downstream product.

## Inference boundary

Inference variables and constraints are local to analysis. Published types and snapshots contain stable nodes and explicit unknown/dynamic/unavailable slots, never inference-local identities. Constraint support is part of the result; an assumed relation cannot silently become established evidence.

## Status

SEMA001 is in progress and partial. This article states the normative separation and the intended ownership boundary, not release completion.
