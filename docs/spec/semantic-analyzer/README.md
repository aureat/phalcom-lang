# Phalcom Semantic Analyzer Specification

This directory is the normative contract for Phalcom's compiler-owned semantic analyzer.

It defines semantic products, authority, identities, analysis transformations, incrementality, publication, and consumer behavior. It does not freeze incidental Rust mechanics such as field order, helper names, local loop structure, or container choice.

```text
language and typing semantics
        ↓
compiler-owned semantic analyzer
        ↓
immutable semantic snapshot
        ↓
compiler, diagnostics, advisory presentation, LSP, lints, refactoring
```

## Authority

These chapters state the effective semantic-analyzer rules. Historical design documents, implementation specifications, plans, checklists, handoffs, and repository analyses remain valuable implementation sources, but they do not override a rule reconciled here.

A rule from an implementation plan becomes normative only when verified, reconciled with existing chapters, and stated as one effective rule in this directory. Keeping a plan open preserves its implementation work and completion gates; it does not create a second normative hierarchy.

When two chapters appear to conflict, resolve the conflict in the chapter that owns the concept and replace duplicate wording with a cross-reference. Do not establish precedence by document date or implementation phase.

## Chapters

1. [`01-semantic-analysis-model.md`](01-semantic-analysis-model.md) — analyzer constitution, product model, authority, ownership, pipeline, and analyzer-wide invariants.
2. [`02-type-knowledge-and-evidence.md`](02-type-knowledge-and-evidence.md) — `TypeKnowledge`, evidence authority, epistemic support, provenance, unknown, and dynamic knowledge.
3. [`03-analysis-status-causality-and-recovery.md`](03-analysis-status-causality-and-recovery.md) — `AnalysisStatus`, causal invalidity, diagnostic ownership, suppression, and invalid-but-analyzable recovery.
4. [`04-expression-analysis-and-contextual-typing.md`](04-expression-analysis-and-contextual-typing.md) — expression composition, synthesis/checking, contextual expectations, operation protocols, and compound products.
5. [`05-binding-and-flow-analysis.md`](05-binding-and-flow-analysis.md) — binding identity, contracts, current knowledge, consistency, assignment, joins, loops, and flow summaries.
6. [`06-relations-reconciliation-and-semantic-judgments.md`](06-relations-reconciliation-and-semantic-judgments.md) — structured relation outcomes, consumer mappings, reconciliation, diagnostic ownership, and terminal propagation.
7. [`07-generic-inference-engine.md`](07-generic-inference-engine.md) — inference variables, constraints, solver progress, support, expected results, and terminal inference.
8. [`08-callable-analysis-and-publication.md`](08-callable-analysis-and-publication.md) — signatures, body entry, return summaries, result authority, constructor identity, and call publication.
9. [`09-semantic-products-incrementality-and-fingerprints.md`](09-semantic-products-incrementality-and-fingerprints.md) — product identity, fingerprints, dependency ownership, reuse, effects, and cold/incremental equivalence.
10. [`10-semantic-identity-source-sites-and-attachments.md`](10-semantic-identity-source-sites-and-attachments.md) — canonical identities, lifetimes, source sites, attachments, and snapshot guards.
11. [`11-advisory-analysis-and-authority.md`](11-advisory-analysis-and-authority.md) — advisory runtime-shape domain, authority separation, canonical dispatch, contributions, and fixed points.
12. [`12-workspace-lifecycle-transactions-and-publication.md`](12-workspace-lifecycle-transactions-and-publication.md) — persistent workspace lifecycle, candidate transactions, failure containment, invalidation, and atomic publication.
13. [`13-semantic-consumers-and-request-consistency.md`](13-semantic-consumers-and-request-consistency.md) — exact/stale/unmapped requests, snapshot pinning, consumer authority, and fallback policy.

## Canonical concept ownership

| Concept | Owner |
|---|---|
| analyzer purpose, lanes, global invariants | 01 |
| `Established`, `Assumed`, `Unknown`, `Dynamic`, support | 02 |
| `Ready`, `Invalid`, `Suppressed`, terminal status, causality | 03 |
| expression and compound-operation composition | 04 |
| binding contract/current state and flow | 05 |
| relation outcomes and consumer reconciliation | 06 |
| generic constraint solving and return-variable support | 07 |
| callable/body/result publication and constructor call semantics | 08 |
| fingerprints, dependencies, equivalence, recomputation/change | 09 |
| semantic/source/revision identity and attachments | 10 |
| advisory domain, authority, flow, contributions, convergence | 11 |
| workspace/source lifecycle and transactional publication | 12 |
| snapshot consumers and request consistency | 13 |

Subsidiary chapters may state consequences of an owned rule but must link to its canonical definition.

## Required separations

The specification never collapses:

```text
language type
!= type knowledge/evidence
!= analysis status
!= causal invalidity
!= advisory runtime-shape fact
!= semantic/source identity
!= source range
!= snapshot/revision identity
!= presentation
```

Formal semantic products are authoritative for compiler judgments. Advisory facts may enrich tooling but cannot strengthen formal knowledge. Consumers query one immutable snapshot and cannot reconstruct competing semantic truth.

## Using the specification

For implementation work:

1. read chapter 01;
2. read the chapter that owns the changed concept;
3. follow cross-references for identity, authority, publication, and consumer consequences;
4. preserve every observable semantic-product dimension;
5. add direct law, source-composition, incremental, and consumer tests where applicable;
6. keep implementation status and completion evidence in implementation plans/checklists.

For review, a change is not correct merely because its final `TypeId` is correct. Check knowledge authority, status, causality, identity, provenance, relation outcomes, dependency ownership, snapshot coherence, and consumer behavior.

## Conformance layers

Use all applicable layers:

```text
internal semantic-law tests
    + source-level composition tests
    + cold/incremental differential tests
    + workspace lifecycle/publication tests
    + semantic consumer/request tests
```

Focused passing tests establish evidence for their slice. Implementation or migration completion remains open until its owning plan/checklist gates are satisfied.

## Manifest

[`MANIFEST.md`](MANIFEST.md) records the exact file set, byte sizes, and SHA-256 hashes. Regenerate it after changing this directory; it contains no semantic authority.
