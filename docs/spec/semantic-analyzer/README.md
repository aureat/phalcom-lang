# Phalcom Semantic Analyzer Implementation Specification

This directory contains the implementation-level semantic specification for Phalcom's compiler-owned semantic analyzer.

It sits between the language/type-system specification and the Rust implementation:

```text
language and typing semantics
        ↓
semantic analyzer implementation specification  ← this set
        ↓
Rust/query/database implementation
        ↓
compiler, diagnostics, advisory projection, LSP
```

The documents specify both **internal semantic behavior** and **externally observable behavior**. They deliberately do not freeze incidental Rust mechanics such as field ordering, helper names, local loop structure, or exact container choice.

## Documents

1. [`01-semantic-analysis-model.md`](01-semantic-analysis-model.md) — analyzer purpose, product model, ownership, pipeline, formal/advisory boundary, information preservation.
2. [`02-type-knowledge-and-evidence.md`](02-type-knowledge-and-evidence.md) — `TypeKnowledge`, established/assumed evidence, authority, provenance, weakening, assumptions, unknown/dynamic.
3. [`03-analysis-status-causality-and-recovery.md`](03-analysis-status-causality-and-recovery.md) — `AnalysisStatus`, causal invalidity, diagnostic ownership, suppression, invalid-but-analyzable recovery.
4. [`04-expression-analysis-and-contextual-typing.md`](04-expression-analysis-and-contextual-typing.md) — expression pipeline, synthesis/checking, expected context, calls, branch/compound propagation, expression publication.
5. [`05-binding-and-flow-analysis.md`](05-binding-and-flow-analysis.md) — binding identity, contracts, current facts, consistency, initialization, assignment, joins, loops, widening, flow summaries.
6. [`06-relations-reconciliation-and-semantic-judgments.md`](06-relations-reconciliation-and-semantic-judgments.md) — structured relation outcomes, consumer mapping, binding reconciliation, diagnostics, terminal outcomes.
7. [`07-generic-inference-engine.md`](07-generic-inference-engine.md) — kinds, inference variables, constraints, failures, support/taint, expected results, terminal inference, fixed-return independence.
8. [`08-callable-analysis-and-publication.md`](08-callable-analysis-and-publication.md) — signatures, body entry, parameter contracts, return summaries, callable status, call-result promotion, publication.
9. [`09-semantic-products-incrementality-and-fingerprints.md`](09-semantic-products-incrementality-and-fingerprints.md) — semantic identity, fingerprints, dependency ownership, reuse, cold/incremental equivalence, formal/advisory separation.

## Normative precedence

These documents consolidate and elaborate the semantic implementation behavior established by:

1. `phalcom_semantic_correctness_part1_corrections_and_amendments.md` — higher precedence where it conflicts;
2. `phalcom_semantic_correctness_single_world_takeover_part1_formal_epistemic_foundation_spec.md`;
3. later implementation work only where it does not contradict those semantics.

The old implementation checklist is useful as history and test inventory but is not itself proof that a checked behavior satisfies the semantic contract.

## Repository reference point

Non-normative implementation references were grounded through GitHub `main` at:

```text
c3b82e4b88469ef9fc79aa65a03e0bed95dc908d
fix(semantic): preserve generic evidence and advisory agreement
2026-08-25
```

The repository is expected to evolve. When code shape changes, update implementation notes rather than weakening the semantic contract to match a transient implementation.

## How to use these documents

For implementation work:

1. read `01` first;
2. read the subsystem document for the code being changed;
3. identify the internal semantic transformation being modified;
4. identify the external observable behavior guaranteed by that transformation;
5. add or adjust conformance tests before changing implementation behavior.

For review:

- a code change is not correct merely because its final `TypeId` is correct;
- check status, causality, provenance, contract/current separation, terminal outcome propagation, and incremental identity where applicable;
- reject implementations that recover by fabricating ordinary types or by collapsing structured outcomes into booleans.

For tests, use all three layers where relevant:

```text
internal law tests
    +
source-level composition tests
    +
incremental/differential tests
```
