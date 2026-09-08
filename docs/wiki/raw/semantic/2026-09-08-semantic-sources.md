# Raw source snapshot: semantic

> Captured: 2026-09-08
> Source area: semantic analyzer specification and semantic-program boundaries
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/spec/semantic-analyzer/README.md ---
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


--- docs/spec/semantic-analyzer/01-semantic-analysis-model.md ---
# Phalcom Semantic Analyzer Specification
## 01 — Semantic Analysis Model

**Status:** Normative semantic-analyzer specification.

**Purpose:** Define the analyzer-wide semantic machine: the products it computes, the authority and ownership boundaries between subsystems, the internal information-flow model, and the externally observable behavior that all subsystem implementations must preserve.

**Scope:** This document specifies the conceptual implementation model and externally observable contract of Phalcom's compiler-owned semantic analyzer. It is intentionally more concrete than the language-level typing specification and intentionally less concrete than the Rust source. It defines what semantic information exists, which subsystem owns it, how it flows through analysis, and what consumers may rely on.

---

## 1. Purpose of the semantic analyzer

Phalcom's semantic analyzer is not merely an accept/reject type checker. It is the compiler-owned system that interprets source programs into a persistent semantic model that can be consumed by compilation, diagnostics, explanation, semantic tooling, and later advisory analysis. Its central responsibility is to preserve the distinction between facts that have been established, assumptions supplied by language contracts, contextual constraints, contradictions, dynamic boundaries, incomplete analysis, and invalidity.

The analyzer therefore answers several different questions at once:

1. **What does this source construct denote?**
2. **What type knowledge is available for its runtime value, if any?**
3. **How strong is that knowledge, and what evidence supports it?**
4. **Did the analyzer complete the semantic operation?**
5. **Is the result causally dependent on an invalid upstream operation?**
6. **Which callable, declaration, field, binding, or type-level object was resolved?**
7. **Which semantic constraints or judgments were applied?**
8. **Which diagnostics own failures, and which later nodes merely depend on those failures?**
9. **Which parts of the result are semantically significant for incremental reuse?**

No single enum or `TypeId` answers all of these questions. The analyzer is correct only when these dimensions remain distinct through analysis and publication.

---

## 2. The semantic product model

At the expression level, the conceptual semantic product is:

```text
ExpressionSemanticResult
├── type knowledge
├── analysis status
├── causal invalidity
├── semantic denotation
├── resolved callable / dispatch identity, where applicable
├── evidence origin and provenance
├── constraint / judgment evidence
└── explanation dependencies
```

The concrete Rust implementation may package these fields in more than one internal object before publication. That representation is not normative. The normative requirement is that every semantically meaningful dimension produced during analysis survives until the published semantic product or an explicitly documented higher-level summary that preserves equivalent information.

At the binding level, the conceptual product is:

```text
BindingSemanticState
├── stable binding identity
├── persistent assignment contract, if any
├── current flow-sensitive type knowledge
├── contract/current consistency
├── mutability
├── current denotation
├── causal invalidity
├── version / flow generation where required
└── explanation / provenance references
```

At the callable level, the product is:

```text
CallableSemanticProduct
├── callable identity and signature
├── body-local expression products
├── body-local binding products
├── call resolutions
├── flow graph / flow summaries
├── return summary
├── diagnostics and causal ownership
├── explanations
├── semantic dependencies
└── callable analysis status
```

The workspace/snapshot layer publishes these products without changing their semantic meaning. Presentation and LSP layers may project or summarize them, but must not retroactively alter formal analysis.

---

## 3. Formal analysis and advisory analysis

Phalcom distinguishes compiler-owned **formal semantic knowledge** from advisory observations or heuristics.

Formal knowledge is eligible to participate in hard semantic judgments such as assignability, binding-contract validation, return checking, generic constraints, and compiler acceptance. Advisory information is observational. It may improve presentation or offer a more specific runtime-shape hypothesis, but it cannot become formal evidence merely because it agrees with a likely runtime shape.

The dependency direction is:

```text
source/module state + language semantics
        ↓

--- docs/spec/semantic-analyzer/09-semantic-products-incrementality-and-fingerprints.md ---
# Phalcom Semantic Analyzer Specification
## 09 — Semantic Products, Incrementality, and Fingerprints

**Status:** Normative semantic-analyzer specification.

**Purpose:** Specify semantic product identity, dependency ownership, fingerprint equivalence, invalidation, and the distinction between semantic changes and incidental source/allocator changes.

---

## 1. Incrementality is part of semantic correctness

An incremental analyzer can produce wrong answers even when every fresh computation is correct if it reuses a stale semantic product after a meaningful dependency changed.

Therefore semantic product identity is a correctness contract.

The fundamental rule is:

> Two products may share a semantic fingerprint only when all downstream-observable semantic behavior represented by that product is equivalent.

Conversely, incidental changes that do not alter semantic meaning should not force unnecessary downstream recomputation.

---

## 2. Three kinds of identity

The implementation must distinguish:

```text
semantic identity
input/source identity
ephemeral allocator identity
```

### 2.1 Semantic identity

Meaning that downstream semantic consumers may observe.

Examples:

```text
type
evidence status/origin where observable
binding contract
binding consistency
mutability
analysis status
causal shape
resolved callable identity
generic constraints
flow epistemic state
```

### 2.2 Input/source identity

Information that may require recomputing a product's payload or ranges even if semantic meaning is unchanged.

Examples:

```text
source text
source range shifts
presentation spans
comments/docs depending on product design
```

### 2.3 Ephemeral identity

Allocator-local values with no meaning across equivalent analyses.

Examples:

```text
DiagnosticCauseId allocation number
arena insertion index when order is incidental
temporary solver variable numbers not exposed semantically
```

Ephemeral identity must not accidentally become semantic cache identity.

---

## 3. Product boundaries

Representative semantic products include:

```text
declaration shell
declaration surface
callable signature
callable body / CallableAnalysis
flow summaries
source semantic indices
semantic snapshot projections
advisory products
```

--- docs/spec/semantic-analyzer/10-semantic-identity-source-sites-and-attachments.md ---
# Phalcom Semantic Analyzer Specification
## 10 — Semantic Identity, Source Sites, and Attachments

**Status:** Normative semantic-analyzer specification.

**Purpose:** Define semantic and source identities, their lifetimes, their stability boundaries, and the canonical attachments that connect source occurrences to semantic targets.

---

## 1. Identity is not location

Identity answers which semantic entity a fact concerns. Source location answers where a representation appears in one source revision. Revision identity answers which semantic world owns a fact.

These relations may coincide for simple declarations and diverge under edits, imports, generated declarations, specialization, recovery, and snapshot replacement.

The governing law is:

> Canonical identity must be carried when available. It must not be reconstructed from name, spelling, source range, or nearest-match search.

---

## 2. Identity domains

The analyzer distinguishes at least these conceptual domains:

| Domain | Representative identity | Meaning and lifetime |
|---|---|---|
| workspace | `WorkspaceId` | one persistent semantic workspace instance |
| semantic revision | `SemanticRevision` | one committed semantic world within a workspace |
| snapshot | `SnapshotId` | immutable publication identity, including required store/revision guards |
| project | `ProjectIdentity` | canonical project identity under module ownership rules |
| module | `ModuleId` | canonical module identity across semantic products |
| declaration | `DeclarationId` | canonical declared semantic entity |
| callable | `CallableId` | canonical owner, selector, and dispatch side |
| field | `FieldId` | canonical owner, name/member identity, and dispatch side |
| source owner | `SourceOwner` | namespace that owns snapshot-local source sites |
| source site | `SourceSiteId` | snapshot-local source occurrence or attachment site |
| external source reference | `SourceSiteRef` | source site guarded by owning snapshot |
| body | `BodyId` | snapshot-local callable or top-level body-analysis identity |
| binding | `BindingId` | snapshot-local local binding identity within body analysis |
| expression | `ExpressionId` | snapshot-local expression identity within body analysis |
| diagnostic cause | `DiagnosticCauseId` | snapshot-local owning contradiction identity |
| explanation | `ExplanationId` | snapshot-local explanation/provenance node identity |

Exact implementation names may differ. The domains, ownership, and non-aliasing laws must remain observable.

---

## 3. Stability matrix

| Identity | Range participates | Name participates | Stable across snapshot | Externally retainable |
|---|---:|---:|---:|---:|
| project/module/declaration/callable/field | no, except through canonical source identity inputs | only where language identity explicitly includes it | yes when canonical identity rules say entity survived | yes |
| `SourceSiteId` | no | no | no | only through snapshot guard |
| `BodyId` / `BindingId` / `ExpressionId` | no | no | no | only through owning published product/reference |
| diagnostic/explanation ID | no | no | no | only through owning snapshot/product |
| `SourceSiteRef` | carries snapshot guard, not range identity | no | rejects different snapshot | yes as guarded handle |

A source edit may move a site without changing its canonical target. Conversely, identical ranges or spellings do not make two targets identical.

---

## 4. Source sites and canonical targets

A source site describes a source occurrence owned by one immutable snapshot. A semantic target describes what that occurrence denotes when canonical resolution succeeds.

Conceptually:

```text
SourceSiteId -> SemanticTargetId

SemanticTargetId =
    Binding(declaration site)
  | Declaration(DeclarationId)
  | Callable(CallableId)
  | Field(FieldId)
  | Module(ModuleId)
```

Unresolved spelling is not a semantic target. Recovery may publish an unresolved occurrence with explanation/status, but it must not fabricate target identity.

---

## 5. Required attachments

The semantic snapshot must publish explicit canonical relationships where analysis establishes them:

```text
source binding declaration <-> body BindingId
formal ExpressionId        <-> SourceSiteId
source callable occurrence  -> CallableId
field/member occurrence     -> FieldId or canonical declaration target
import/module occurrence    -> ModuleId / DeclarationId
diagnostic-owning judgment  -> DiagnosticCauseId
semantic fact               -> ExplanationId, where explanation is published

--- docs/spec/semantic-analyzer/11-advisory-analysis-and-authority.md ---
# Phalcom Semantic Analyzer Specification
## 11 — Advisory Analysis and Authority

**Status:** Normative semantic-analyzer specification.

**Purpose:** Define advisory runtime-shape analysis as a compiler-owned abstract domain, its authority boundary relative to formal semantics, its resolution and fixed-point rules, and its publication contract.

---

## 1. Advisory semantics is a separate domain

Formal type knowledge and advisory runtime-shape knowledge answer different questions.

```text
formal TypeKnowledge
    what proposition is established, assumed, unknown, or dynamic

advisory ValueShape
    bounded prediction of runtime value/member/collection shape for tooling
```

Advisory facts are not language types, proof facts, assignability premises, or optimizer correctness facts.

---

## 2. Advisory product

An advisory fact/product conceptually carries:

```text
AdvisoryFact
├── ValueShape
├── AdvisoryOrigin / provenance
├── confidence or precision class
├── AdvisoryProductStatus
├── canonical source/target attachment
├── input/product fingerprint
└── bounded explanation
```

Shape and status are orthogonal. “No advisory product published” is distinct from “analysis completed and widened to unknown.” Cancelled, budget-exhausted, blocked, and internally failed advisory computations must not masquerade as ordinary unknown shapes.

---

## 3. Authority law

The direction is:

```text
formal products -> optional advisory seed/projection
advisory products -> presentation and non-authoritative tooling
```

The reverse direction is forbidden. Advisory analysis cannot:

- construct `Established` or `Assumed` formal knowledge;
- satisfy or refute a formal relation;
- repair a formal invalid/unknown/dynamic result;
- create hard compiler diagnostics from disagreement;
- authorize proof or unsafe optimization.

---

## 4. Formal/advisory composition matrix

| Formal product | Advisory product | Formal result | Permitted presentation |
|---|---|---|---|
| `Established(T), Ready` | compatible shape | unchanged | normal formal type; optional advisory detail |
| `Established(T), Ready` | incompatible shape | unchanged | formal wins; optional advisory-disagreement explanation |
| `Assumed(T)` | compatible/incompatible shape | unchanged | show formal assumption normally; advisory detail may be labeled |
| `Unknown(R)` | useful shape | remains `Unknown(R)` | advisory-only enrichment may be shown without claiming formal proof |
| `Dynamic(D)` | useful shape | remains dynamic | advisory runtime prediction may be shown as non-authoritative |
| known + `Invalid(C)` | any shape | knowledge/status/cause unchanged | preserve diagnostic and formal fact; advisory cannot repair |
| `Blocked` / `Cancelled` / budget / internal failure | available shape | terminal formal state unchanged | advisory may be omitted or shown with incomplete-formal explanation |

Presentation policy must never blend disagreement into a synthetic union type or confidence ladder between formal and advisory authority.

---

## 5. Formal-to-advisory projection

Projection is explicit and one-way. A known nominal/applied type may seed a broad compatible shape. An assumed formal proposition may guide advisory analysis but cannot create exact advisory observation merely because it is a formal premise.

Unknown or dynamic formal knowledge does not by itself supply a concrete advisory seed. Advisory analysis may still derive a shape from its own legitimate syntax/flow/runtime-shape rules.

Projection uses canonical type/declaration structure, never class-name strings or presentation text.

---

## 6. Canonical resolution

Advisory member and call resolution must use canonical compiler identities, selector rules, visibility, linked exports, and dispatch surfaces.

Required laws:


--- docs/implementation/SEMA001-type-formation-and-inference/PROGRAM.md ---
---
id: SEMA001
category: SEMA
kind: type-formation-and-inference
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# SEMA001 — type formation and inference

This program owns canonical type formation, expression typing, generic
inference, advanced kinds, row typing, type-system closure, and typing
integration. Runtime metadata, semantic presentation, cross-consumer
integration, and verification are separate programs.

--- docs/implementation/SEMA002-semantic-authority-and-identity/PROGRAM.md ---
---
id: SEMA002
category: SEMA
kind: semantic-authority
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# SEMA002 — semantic authority and identity

This program owns the formal epistemic foundation and canonical identity
projection required for one semantic world. Workspace/LSP cutover and later
capability/control-flow work are separate programs.

--- docs/implementation/SEMA003-semantic-workspace-and-lsp/PROGRAM.md ---
---
id: SEMA003
category: SEMA
kind: workspace-and-lsp
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# SEMA003 — semantic workspace and lsp

This program owns persistent workspace state, semantic snapshot publication,
LSP cutover, and editor-facing consumption of canonical semantic products.
