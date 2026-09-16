---
id: LANG005.C6.P2
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C6.P1
follows: LANG005.C6.P1
supersedes: null
prepared: 2026-09-16
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: af2d0aec4e9a7fe2a4b6d02cc9a6b573f6ab62d8
---

# LANG005.C6.P2 — Generic Associated Projections, Refinements, Equality, and Inference

> **For Luna / agentic implementers:** Execute this plan task-by-task under the repository's Luna patch-grade workflow. Re-read the live tree before editing, adapt mechanical drift locally, and STOP AND CONSULT rather than changing any FIXED architecture below. Use sparse, discriminating BUILD tests. This plan deliberately begins with a bounded P1 hardening slice because P2 would otherwise build richer associated evidence on top of two identified evidence-slot correctness hazards. Do not reinterpret that prerequisite work as authorization to redesign C6.P1.

**Goal:** Extend Phalcom's generic-constraint/evidence architecture so generic associated projections such as `T::Item` and `<T as Iterator>::Item`, associated specialization such as `I |= Iterator<Item = T>`, and general projection equality such as `I::Item == T` form canonical semantic propositions, participate in normalization and bounded generic inference, survive public metadata/cross-module boundaries, and remain entirely semantic rather than becoming runtime projection search.

**Architecture:** Preserve the P1 `GenericSignature` / `GenericConstraint` / `GenericEvidenceEnvironment` authority and the C5 `TypeData::AssociatedProjection` / `types::normalize_type` authority. P2 makes generic signature formation staged and evidence-aware: direct conformance anchors are established before contextual projection identities are resolved; associated specialization elaborates to existing `Conforms + Equivalent` propositions; equality/refinement becomes queryable through the same lexical evidence environment; projection-aware inference feeds newly established associated values back into the existing generic inference session through a bounded fixed point. Stable metadata gains a general associated-projection type node. Compiler/VM behavior remains limited to P1 behavioral evidence transport; there is no runtime associated-type solver.

**Tech stack:** Rust workspace (`phalcom-ast`, `phalcom-semantic`, `phalcom-type-meta`, `phalcom-type-syntax`, `phalcom-native-meta`, `phalcom-core`, `phalcom-lsp`), semantic DB/incremental products, stable semantic metadata, generic inference, associated projection normalization, source index/editor products, Phalcom source fixtures, Cargo integration tests.

**Primary specification:** `docs/specs/generic-constraints.md`

**Principal predecessor records:**
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md`
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md`
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-walkthrough.md`
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-handoff.md`

**Target repository path when landed:**

```text
docs/implementation/LANG005/LANG005.C6/
  LANG005.C6-CHECKPOINT.md
  LANG005.C6-GUIDANCE.md
  LANG005.C6.P2-generic-associated-projections-refinements-equality-and-inference-plan.md
  LANG005.C6.P2-walkthrough.md
  LANG005.C6.P2-handoff.md
```

---

## 0. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High or an equivalently capable implementation model working under constrained architectural authority.

The implementer:

- may adapt helper names, local decomposition, imports, and equivalent private APIs to the live repository;
- must preserve the semantic ownership, projection identity, generic evidence, metadata, incremental, lowering, runtime, and inference invariants below;
- must not improvise material architecture;
- must not create a second associated-projection normalizer beside `types::normalize_type`;
- must not create a second generic evidence/proof environment beside `GenericEvidenceEnvironment`;
- must not make associated specialization part of `TraitRef` identity;
- must not use raw source ordering as semantic identity;
- must not treat qualification syntax as conformance proof;
- must not search the open world for a generic subject merely because an associated output type is known;
- must not move projection proof, normalization, or associated lookup into the compiler or VM;
- must follow the BUILD / STABILIZE / CERTIFY testing budget rather than testing reflexively;
- must classify unrelated failures before spending implementation time on them;
- must preserve unrelated modified/staged/untracked work;
- must keep the shared `LANG005.C6-CHECKPOINT.md` current at durable gates only;
- must produce the required P2 walkthrough and P2-to-P3 handoff;
- must STOP AND CONSULT when any global or task-local trigger fires;
- must not commit or push unless the surrounding workflow/user instruction explicitly authorizes it.

Testing is evidence gathering, not ritual. A test is run because it discriminates an invariant, validates a durable interface, or satisfies a named gate. Do not run a broad suite merely because an edit occurred.

---

## 1. Goal

Implement the generic associated-projection/equality layer required by the remainder of C6 without reopening C5 exact associated-type architecture or P1 direct generic conformance architecture.

The minimum positive semantic examples are:

### 1.1 Contextual generic projection

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
}

class Tools {
  @class
  copy<I>(_ iterator: I, _ value: I::Item) -> I::Item
      where I |= Iterator
  {
      value
  }
}
```

`I::Item` is valid because the lexical generic evidence proves exactly one applicable trait associated requirement named `Item` for subject `I`. No exact associated value is required to form the symbolic projection.

### 1.2 Qualified projection

```phalcom
class Tools {
  @class
  copy<I>(_ iterator: I, _ value: <I as Iterator>::Item)
      -> <I as Iterator>::Item
      where I |= Iterator
  {
      value
  }
}
```

Qualification identifies the associated requirement; it does not manufacture `I |= Iterator`.

### 1.3 Associated specialization

```phalcom
class Tools {
  @class
  first<I, T>(_ iterator: I) -> Option<T>
      where I |= Iterator<Item = T>
  {
      iterator.next
  }
}
```

The generic contract canonically means:

```text
Conforms(I, Iterator)

Equivalent(
    AssociatedProjection(
        subject = I,
        trait_ref = Iterator,
        requirement = Iterator::Item
    ),
    T
)
```

### 1.4 Associated-evidence-driven inference

Given exact evidence:

```text
UserIterator |= Iterator
<UserIterator as Iterator>::Item == User
```

and:

```text
I = UserIterator
<I as Iterator>::Item == T
```

infer:

```text
T = User
```

P2 must not allow reverse open-world inference from `T = User` to an arbitrary unknown `I`.

---

## 2. Checkpoint acceptance objective

`LANG005.C6` owns the complete generic-constraint/evidence model required for later conditional conformance proof, generic Iterator/Iterable laws, and production iteration migration.

P2 is the **generic associated-constraint closure**, not full C6 closure.

### 2.1 P2 bounded acceptance objective

> Generic or qualified associated projections form canonical `TypeData::AssociatedProjection` values when their trait identity and conformance evidence are justified; `Trait<Assoc = Type>` specialization elaborates into ordinary direct conformance plus canonical type equivalence; lexical equality/refinement evidence participates in the existing projection normalizer; associated evidence can feed generic type inference through a bounded fixed point without open-world reverse search; generic owner and method signatures can use those projections; stable metadata can represent projection-bearing public generic contracts; imported consumers reconstruct equivalent behavior; editor/navigation and incremental dependency products retain canonical associated identity; and compiler/runtime layers continue to transport only already-selected behavioral evidence.

### 2.2 P2 is not checkpoint closure

```text
C6.P1
    direct generic conformance/evidence
        ↓
C6.P2
    generic + qualified associated projections
    associated specialization/equality
    projection-driven inference
        ↓
C6.P3
    conditional conformances
    nested recursive obligations
    cycles/memoization/coherence
        ↓
C6.P4
    Iterator / Iterable generic laws
    blanket/derived implication decision
        ↓
C6.P5
    production for/collection/stateful iterator migration
    cursor retirement
        ↓
C6.P6
    remaining declaration surfaces
    higher-kind closure where specified
    aliases/GADT/tooling/performance/stress/certification
```

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: af2d0aec4e9a7fe2a4b6d02cc9a6b573f6ab62d8
revision subject: feat(lang005): complete C6.P1 generic conformance vertical
relevant predecessor: LANG005.C6.P1
predecessor state: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
checkpoint state: IN_PROGRESS / PARTIAL / FOCUSED_TESTED
```

### 3.1 Live facts verified during planning

1. `GenericConstraint` already has `Subtype`, `Equivalent`, and `Conforms { subject: TypeTerm, trait_ref: TraitRef }`.
2. `GenericConstraintSyntax::Conforms` already accepts a subject and multiple direct trait bounds.
3. `GenericConstraint` instances are sorted/deduplicated during generic signature formation.
4. `GenericEvidenceEnvironment` owns the canonical constraint slice plus indexed direct conformance assumptions and a deterministic behavioral evidence layout.
5. `GenericEvidenceLayout::build(...)` has checked `u16` slot-capacity failure, but current environment constructors use `unwrap_or_default()` and can silently convert overflow into an empty layout.
6. compiler call-environment construction currently builds conformance slot recipes by `push`, so an unmaterialized earlier semantic slot can compact later slot positions.
7. `TraitRef::form_identity(...)` exists and checks declaration identity, arity, ordinary argument kind, and canonical argument validity without proving trait generic constraints.
8. generic bound formation still terminates with raw `TraitRef::new(...)` rather than routing through `form_identity`.
9. `TraitRef::form(...)` remains a combined identity + older constraint-validation path; there is not yet one generic-evidence-aware trait application well-formedness authority.
10. `CheckingContext` carries both `generic_evidence` and `ambient_constraints`; body construction currently derives the latter from the former.
11. `TypeData::AssociatedProjection` is already a general carrier with `subject`, `TraitRef`, and `AssociatedTypeRequirementId`.
12. source projection resolution currently restricts generic projection formation to C5 `Self` context.
13. `types::normalize_type` is the canonical associated projection normalizer and already traverses nested type structure.
14. generic inference currently converts many structural `TypeData` variants recursively, but `AssociatedProjection` remains opaque as `InferenceTerm::Canonical`.
15. stable metadata has `StableTraitApplicationRef` and `GenericConstraintRef::Conforms`, but stable `TypeNode` has no associated projection node.
16. metadata export explicitly rejects `TypeData::AssociatedProjection(_)` as non-exportable.
17. P1 stable reflection/navigation and LSP `::` presentation are already landed.
18. semantic tests use modules under `phalcom-semantic/tests/semantic/`; P1 direct conformance tests live in `capabilities/generic_conformance.rs`.
19. core runtime tests use modules under `phalcom-core/tests/core/language/`; P1 runtime tests live in `language/generic_conformance.rs`.
20. current `main` remained exactly `af2d0aec4e9a7fe2a4b6d02cc9a6b573f6ab62d8` at plan creation.

> **Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.**

---

## 4. Required reads before implementation

### 4.1 Workflow authority

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
docs/workflow/sol-principal-architect-project-doctrine.md   # if present live
```

### 4.2 C6 state

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-canonical-generic-conformance-evidence-and-runtime-transport-plan.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-handoff.md
```

### 4.3 C5 associated-type takeover authority

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-associated-projection-formation-normalization-and-evidence-hardening-plan.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P3-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P3-handoff.md
```

### 4.4 Normative language authority

```text
docs/specs/README.md
docs/specs/generic-constraints.md
authoritative associated-types specification
authoritative traits/conformance specification
authoritative generic/type-formation specification
authoritative kind specification
authoritative subtyping/equivalence specification
authoritative callable/selector/dispatch specification
authoritative exact enum-case specification
```

### 4.5 Production architecture

Re-read live equivalents of:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/editor.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/validate.rs
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/vm/dispatch.rs
phalcom-lsp/src/backend.rs
```

### 4.6 Test conventions

```text
phalcom-semantic/tests/semantic/README.md
phalcom-semantic/tests/semantic/capabilities/mod.rs
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs
phalcom-core/tests/README.md
phalcom-core/tests/core/language/mod.rs
phalcom-core/tests/core/language/generic_conformance.rs
phalcom-core/tests/core/language/associated_traits.rs
live LSP associated-type navigation/hover tests
```

---

## 5. Normative authority

### 5.1 Relations

```text
:     kind classification
|=    trait conformance / evidence requirement
<:    subtype relation
==    canonical semantic type equivalence
```

Associated specialization:

```phalcom
where I |= Iterator<Item = T>
```

means:

```text
I |= Iterator
<I as Iterator>::Item == T
```

It does not create a new semantic relation.

### 5.2 Projection identity

Contextual and qualified syntax may converge to one semantic identity:

```text
subject
+
applied TraitRef
+
AssociatedTypeRequirementId
```

### 5.3 Contextual projection

`T::Item` requires exactly one applicable associated requirement named `Item` from the available lexical evidence. Zero candidates is insufficient evidence; multiple distinct candidates is ambiguity. Source order never chooses.

### 5.4 Qualified projection

`<T as Trait>::Item` supplies trait identity but still requires proof of `T |= Trait`.

### 5.5 Associated specialization

For:

```phalcom
where C |= Converts<A, B, Error = E>
```

the applied trait identity is `Converts<A, B>` and the additional proposition is:

```text
<C as Converts<A, B>>::Error == E
```

### 5.6 Duplicate specialization

Duplicate specialization in one trait requirement is invalid before canonical top-level deduplication:

```phalcom
Iterator<Item = A, Item = A>
Iterator<Item = A, Item = B>
```

### 5.7 Associated-driven inference

Allowed:

```text
I = ConcreteIterator
ConcreteIterator::Item = User
<I as Iterator>::Item == T
⇒ T = User
```

Forbidden:

```text
T = User
<I as Iterator>::Item == T
⇒ search world for I
```

### 5.8 Runtime boundary

Projection identity, equality, normalization, and inference are semantic. The VM never resolves associated declarations, searches conformances, normalizes projections, or solves generic equality.

---

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Current symbol/path | Owner | P2 invariant |
|---|---|---|---|
| generic propositions | `GenericConstraint` | semantic type system | no associated-specific relation |
| direct conformance | `GenericConstraint::Conforms` | semantic type system | specialization elaborates into this + `Equivalent` |
| equality | `GenericConstraint::Equivalent` | semantic type system | projection equality remains this |
| trait identity | `TraitRef` | traits | specialization excluded from identity |
| trait identity formation | `TraitRef::form_identity` | traits/type formation | new source trait references use it |
| associated identity | `AssociatedTypeRequirementId` | trait surface | name is presentation, not identity |
| projection carrier | `TypeData::AssociatedProjection` | type store | reuse canonical carrier |
| projection normalizer | `types::normalize_type` | semantic types | singular recursive authority |
| lexical generic evidence | `GenericEvidenceEnvironment` | semantic generic evidence | add equality/refinement views here |
| exact conformance evidence | `ConformanceEvidence` | C4/C5 | remains exact proof |
| generic proof | `query_generic_conformance_proof` | generic evidence | direct proof authority |
| behavioral evidence slot | `GenericEvidenceSlotKey` | generic evidence | associated types are not slots |
| call specialization | `CallSpecialization` | checker | normal type bindings + behavioral evidence |
| stable trait application | `StableTraitApplicationRef` | type metadata | reuse in projection node |
| runtime type environment | `RuntimeTypeEnvironment` | core | separate from conformance |
| runtime conformance environment | `RuntimeConformanceEnvironment` | core | behavioral method handles only |
| abstract trait call | `InvokeTraitRequirement` | VM/compiler | unchanged |

### 6.2 Freeze P1

```text
assumed evidence != exact ImplId evidence
source order irrelevant
Dynamic not conformance proof
compiler/VM do not search conformances
type and conformance runtime environments remain separate
```

### 6.3 Freeze C5

```text
projection is canonical TypeData
associated identity is trait-owned
normalize_type is singular
recursive/blocked states remain explicit
associated requirements are not behavioral methods
```

---

## 7. Architecture

### 7.1 Authority flow

```text
source where/signature syntax
    ↓
structured AST
    ↓
staged generic signature formation
    ├─ final kinds
    ├─ TraitRef identities
    ├─ direct Conforms anchors
    ├─ projection identities
    ├─ specialization elaboration
    └─ canonical Equivalent propositions
    ↓
GenericEvidenceEnvironment
    ├─ conformance assumptions
    ├─ equality/refinement view
    ├─ associated candidate index
    └─ behavioral evidence layout
    ↓
generic proof + normalize_type
    ↓
projection-aware bounded inference
    ↓
CallSpecialization
    ├─ ordinary type bindings
    └─ P1 behavioral evidence sources
    ↓
stable metadata / compiler
    ↓
VM receives only selected behavioral evidence
```

### 7.2 Staged generic signature formation

The implementation must semantically perform:

```text
A. merge parameter kind facts
B. intern parameter identities
C. form direct trait identities / Conforms anchors
D. build provisional complete lexical conformance evidence
E. resolve contextual/qualified projection identities
F. elaborate associated specializations into Equivalent
G. validate trait applications/projection well-formedness
H. canonicalize Equivalent orientation + sort/dedup
I. publish GenericSignature
```

The code need not literally use these phase names.

### 7.3 Generic evidence after P2

One environment should efficiently answer equivalents of:

```text
constraints()
conformance_for(subject, trait_ref)
conformances_for_subject(subject)
associated_candidates(subject, name)
equivalence/refinement query
behavioral evidence layout
```

### 7.4 Projection normalization

For projection `P = <S as Trait<A...>>::Assoc`:

1. normalize/instantiate subject and ordinary trait args as appropriate;
2. prove conformance through canonical exact/assumed proof;
3. exact proof: reuse C5 associated binding and recursively normalize;
4. assumed proof: check lexical equality/refinement for `P`;
5. unique refinement: recursively normalize refinement;
6. assumed without refinement: retain symbolic projection;
7. preserve ambiguous/recursive/blocked/dynamic/cancel/budget/internal outcomes.

### 7.5 Inference

Preferred general form:

```rust
InferenceTerm::AssociatedProjection {
    subject: Box<InferenceTerm>,
    trait_ref: InferenceTraitRef,
    requirement: AssociatedTypeRequirementId,
}
```

Equivalent architecture is allowed if it structurally represents projections everywhere ordinary types occur.

The solver uses a bounded feedback worklist:

```text
ordinary solve
→ materialize newly-known projection subjects
→ prove conformance / normalize projection
→ inject equality
→ solve again only if information changed
```

### 7.6 Stable metadata

Add a general stable projection node equivalent to:

```rust
TypeNode::AssociatedProjection {
    subject: TypeNodeId,
    trait_application: StableTraitApplicationRef,
    requirement: StableAssociatedTypeRequirementRef,
}
```

Reuse one stable trait-application identity.

---

## 8. Ownership boundaries

### Owns

- P1 evidence-layout overflow and slot-position hardening required by P2;
- missing P1 substrate regressions;
- canonical where-kind syntax integration if still absent;
- structured associated specialization syntax;
- contextual `T::Item`;
- qualified `<T as Trait>::Item`;
- specialization elaboration;
- symmetric equality canonicalization;
- lexical equality/refinement lookup;
- generic projection normalization;
- projection-aware inference;
- owner/method signature projection formation;
- stable projection metadata;
- cross-module contracts;
- reflection/navigation/incremental integration.

### Does not own

- conditional conformance recursion;
- overlap specialization;
- blanket conformances;
- Iterator⇒Iterable law;
- production iterator migration;
- supertraits;
- trait objects/existentials;
- generic associated types;
- higher-kinded associated declarations;
- reverse open-world inference;
- generalized GADT solving;
- runtime associated dictionaries/bytecode.

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| parameter kind | generic signature/type parameter | formation/inference | where-kind side table |
| direct conformance | `GenericConstraint::Conforms` | evidence/proof | checker-local bound list |
| equality | `GenericConstraint::Equivalent` | normalization/inference | associated-specific equality relation |
| trait identity | `TraitRef` | proof/projection | specialization-bearing trait ref |
| associated identity | `AssociatedTypeRequirementId` | projection/editor | name string identity |
| lexical assumptions | `GenericEvidenceEnvironment` | body/proof/normalization | projection evidence env |
| normalization | `types::normalize_type` | checker/inference | generic-only normalizer |
| inference substitution | existing inference session | call/result | projection-local map |
| runtime behavioral evidence | `RuntimeConformanceEnvironment` | VM | associated runtime slots |
| stable projection | stable type graph | metadata/import | constraint-only encoding |

---

## 9. Global invariants

`INV-01` — `GenericConstraint` remains the single generic proposition authority.

`INV-02` — associated specialization elaborates to `Conforms + Equivalent`.

`INV-03` — projection identity is `(subject, TraitRef, AssociatedTypeRequirementId)`.

`INV-04` — contextual and qualified spellings converge when they denote the same projection.

`INV-05` — associated specialization is not part of `TraitRef`.

`INV-06` — all source-formed trait identities pass canonical identity formation.

`INV-07` — trait identity formation and trait generic-contract proof are separate.

`INV-08` — qualification identifies a trait but does not prove conformance.

`INV-09` — contextual projection requires exactly one applicable associated requirement.

`INV-10` — missing, unique, and ambiguous contextual projection are distinct outcomes.

`INV-11` — duplicate associated specialization is invalid before semantic deduplication.

`INV-12` — ordinary trait args and associated specializations remain separate lanes.

`INV-13` — `Equivalent` is symmetric and canonically oriented.

`INV-14` — semantically equivalent constraint order/regrouping is a no-op.

`INV-15` — binder and where kind facts feed one kind authority and conflicts diagnose.

`INV-16` — `GenericEvidenceEnvironment` remains the one lexical generic evidence authority.

`INV-17` — `ambient_constraints` receives no new P2 proof/normalization authority.

`INV-18` — evidence-layout construction failure is never silently defaulted.

`INV-19` — runtime evidence recipes preserve semantic slot positions exactly.

`INV-20` — associated type requirements never become behavioral runtime evidence slots.

`INV-21` — assumed evidence remains distinct from exact source evidence.

`INV-22` — `types::normalize_type` remains the singular recursive projection normalizer.

`INV-23` — exact bindings and lexical refinements use that same normalizer.

`INV-24` — assumed conformance without refinement may justify symbolic projection but not invent a value.

`INV-25` — terminal proof/normalization states are never collapsed to missing/success.

`INV-26` — associated evidence refines inference only after subject/trait application is sufficiently identified.

`INV-27` — inference never reverse-searches the world for a projection subject.

`INV-28` — projections work in ordinary type positions, not only specialization syntax.

`INV-29` — generic callers may satisfy callee associated equality from lexical evidence without runtime equality slots.

`INV-30` — receiver specialization, owner evidence, and method evidence compose before projection reasoning.

`INV-31` — exact enum-case subject identity remains exact through projection normalization.

`INV-32` — `Dynamic` is not projection conformance proof.

`INV-33` — stable metadata carries projection identity without snapshot-local IDs.

`INV-34` — imported projection-bearing contracts reconstruct source-local semantics.

`INV-35` — semantic fingerprints represent canonical propositions, not source spelling/order.

`INV-36` — editor/source targets use canonical trait/associated identities.

`INV-37` — VM performs no associated lookup/proof/normalization/inference.

`INV-38` — runtime type and conformance environments remain separate.

`INV-39` — equal runtime conformance environments remain internable/reusable.

`INV-40` — conditional/nested proof remains P3.


---

## 10. Non-goals

Do not expand this plan into:

```text
conditional generic impl proof
conditional conformance recursion
proof graph cycle memoization for P3 candidates
conditional overlap specialization
blanket impls over arbitrary T
Iterator/Iterable implication
supertraits
trait object construction
existential associated types
generic associated types
higher-kinded trait subject declaration syntax
higher-kinded associated requirements
production Array/Map/Range iterator migration
for-loop lowering replacement
cursor protocol retirement
runtime associated-type tables
new projection bytecodes
new runtime proof solver
full alias/GADT closure
workspace-wide performance optimization
general semantic DB redesign
```

A later task may consume the P2 substrate. It must not be pulled into this plan merely because a helper could be generalized further.

---

## 11. Expected impact map

### 11.1 Expected source areas

- `phalcom-ast/src/ast.rs` — structured trait requirement, associated specialization, qualified projection, where-kind syntax as needed.
- `phalcom-ast/src/parser.rs` — parse new source forms and duplicate/order diagnostics.
- `phalcom-semantic/src/types/annotation.rs` — staged generic signature formation, trait identity formation, generic contextual/qualified projection formation.
- `phalcom-semantic/src/types/parameter.rs` — only if canonicalization helpers belong with constraints; avoid new relation variants.
- `phalcom-semantic/src/types/store.rs` / associated projection owner — reuse canonical projection interning.
- `phalcom-semantic/src/generic_evidence.rs` — checked construction, equality/refinement indexes/query surface.
- `phalcom-semantic/src/traits.rs` — canonical trait application identity/well-formedness seam.
- `phalcom-semantic/src/checker/context.rs` / `body.rs` — lexical environment composition and compatibility audit.
- `phalcom-semantic/src/checker/declaration_signature.rs` — owner/method evidence-aware parameter/return formation.
- `phalcom-semantic/src/checker/inference.rs` — projection-aware inference term/deferred representation.
- `phalcom-semantic/src/checker/call.rs` — bounded projection/equality feedback loop and generic call discharge.
- `phalcom-semantic/src/db/fingerprint.rs` — canonical projection/equality signature hashing.
- `phalcom-semantic/src/metadata/export.rs` — stable projection export.
- `phalcom-semantic/src/source_index/builder.rs` / `editor.rs` — projection and specialization source targets.
- `phalcom-type-meta/src/type_node.rs` — stable associated projection node.
- `phalcom-type-meta/src/generic.rs` or shared stable trait identity owner — reuse `StableTraitApplicationRef`.
- `phalcom-type-meta/src/validate.rs` — projection metadata validation.
- `phalcom-core/src/modules/compile.rs` — P1 evidence slot position hardening.
- `phalcom-core/src/typing/conformance_environment.rs` / inspection — P1 resource regression seam if needed.
- `phalcom-lsp/src/backend.rs` — hover/navigation presentation if semantic editor products do not already make behavior automatic.

### 11.2 Expected new tests

Create, unless live organization has already introduced equivalent semantic names:

```text
phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs
phalcom-core/tests/core/language/generic_associated_constraints.rs
```

Register in:

```text
phalcom-semantic/tests/semantic/capabilities/mod.rs
phalcom-core/tests/core/language/mod.rs
```

Extend existing focused files where ownership is clearer:

```text
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs
existing LSP associated-type tests
existing metadata validation/export tests
```

### 11.3 Expected documentation/state

- update `LANG005.C6-CHECKPOINT.md` at durable P2 gates;
- create `LANG005.C6.P2-walkthrough.md` after implementation;
- create `LANG005.C6.P2-handoff.md` for P3;
- do not rewrite `LANG005.C6-GUIDANCE.md` unless a persistent C6 rule actually changes.

### 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem not anticipated here when that entry changes semantic ownership, identity, lifetime, stable representation, inference authority, normalization authority, or runtime responsibility.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. `GenericConstraint` is the canonical generic proposition authority.
2. associated specialization elaborates to `Conforms + Equivalent`.
3. projection semantic identity is subject + `TraitRef` + associated requirement identity.
4. `TraitRef` does not include associated specialization.
5. identity formation and trait generic-contract proof are separate.
6. contextual projection uses complete lexical evidence, not source order.
7. qualified projection still requires conformance evidence.
8. `GenericEvidenceEnvironment` is the single lexical generic evidence authority.
9. `types::normalize_type` is the singular projection normalizer.
10. associated evidence may refine inference only after subject identification; no reverse world search.
11. stable metadata gets a general projection type representation rather than a constraint-only escape hatch.
12. the VM does not become an associated-type solver.
13. P1 exact/assumed evidence distinction remains.
14. P3 conditional proof remains out of scope.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- helper/type names for structured trait requirement syntax;
- whether structured trait requirement lives directly inside `GenericConstraintSyntax::Conforms` or a dedicated AST helper type;
- private map/index implementation inside `GenericEvidenceEnvironment`;
- exact API name for checked environment construction;
- exact API name for generic equality query;
- exact enum/struct spelling for inference projection terms;
- exact stable metadata module containing `StableTraitApplicationRef`;
- whether normalization receives a new mode variant or an expanded evidence context;
- test file decomposition when repository conventions prefer an existing file;
- private DB query granularity;
- local diagnostics wording, provided semantic categories remain distinct.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| parser already accepts arbitrary contextual `Type::Assoc` subjects structurally | `phalcom-ast/src/parser.rs` | broaden mechanically; do not change projection semantics |
| current AST associated projection can represent generic subject unchanged | `phalcom-ast/src/ast.rs` | mechanically extend spans/fields; preserve identity |
| `GenericEvidenceEnvironment::from_constraints/compose` still swallow layout errors | `generic_evidence.rs` | keep regression/invariant; skip redundant code repair |
| compiler still compacts runtime conformance recipes | `phalcom-core/src/modules/compile.rs` | keep regression/invariant; skip redundant repair |
| `TraitRef::form_identity` remains usable without generic proof | `traits.rs` | if absent materially, STOP AND CONSULT |
| `resolve_trait_ref_for_conformance` still bypasses `form_identity` | `types/annotation.rs` | adapt to live formation seam |
| C5 `normalize_type` supports adding lexical generic evidence without duplicate recursion | live normalization owner | if not, STOP AND CONSULT |
| inference has one extensible structural `InferenceTerm` authority | `checker/inference.rs` | if projection support would require a second solver, STOP AND CONSULT |
| stable type metadata still rejects associated projections | metadata exporter/type node | if already fixed, consume live representation |
| owner generic evidence can be reconstructed while building member signatures | declaration signature/body setup | if owner evidence requires new lifetime/ownership architecture, STOP AND CONSULT |
| source index has a canonical associated target variant | source index/editor | if absent after C5, STOP AND CONSULT before inventing a parallel target |
| runtime evidence registry exposes no test count | conformance environment registry | add minimal inspection only if needed |
| method references depending on generic evidence are a supported current surface | callable-reference semantic/lowering tests | if support requires new capture/lifetime architecture, defer and consult |
| where-clause kind syntax `where F: ...` remains absent | parser/AST | implement as P2 syntax integration; if newer spec removed it, follow spec |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing and build a consultation packet if any of the following occurs:

1. architecture appears impossible or materially wrong;
2. a core plan premise is false;
3. ownership/identity/lifetime/representation/semantic-authority boundary must change;
4. a downstream contract must materially change;
5. semantic expected behavior/test oracle appears wrong;
6. the same nontrivial semantic failure persists after one serious correction;
7. two plausible fixes have materially different architecture;
8. an unexpected subsystem is required;
9. a workaround/special-case architecture is emerging;
10. duplicate canonical reasoning/state would be introduced;
11. soundness-critical behavior is uncertain;
12. spec/code/test/plan authorities materially conflict;
13. an invariant must be weakened to pass tests;
14. scope becomes materially more cross-cutting than planned;
15. contextual generic projection cannot reuse `TypeData::AssociatedProjection`;
16. P2 would require adding specialization data to `TraitRef`;
17. P2 would require a second recursive projection normalizer;
18. projection-aware inference appears to require open-world conformance/type search;
19. qualified projection can only be implemented by skipping conformance proof;
20. generic equality proof requires P3-style recursive conditional candidate search;
21. stable metadata can only carry projections using snapshot-local semantic IDs;
22. owner/method generic projection formation requires a new persistent runtime evidence-capture model;
23. a required behavioral evidence slot cannot be materialized and the only apparent fix is runtime lookup;
24. associated specialization syntax conflicts materially with authoritative grammar;
25. conditional conformance logic starts entering P2 to make tests pass.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

One cycle is:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

Do not use this budget to try three architectural variants.

### 14.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

### 14.4 Inference-specific debugging discipline

When a projection-driven inference test fails, classify it before editing:

```text
A. projection identity never formed
B. conformance proof unavailable
C. projection failed to normalize/refine
D. normalized equality was not injected into inference
E. inference solved but result publication lost substitution
F. diagnostic/status classification is wrong
```

Do not rewrite the solver until the failing stage is identified.

Never rerun an unchanged failing test unless something relevant to the failure changed.

---

## 15. Testing surface analysis

The following coverage IDs are mandatory plan obligations. Equivalent test names are acceptable if live repository mechanics differ.

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | P1 hardening | INV-18 | evidence-layout overflow is a checked propagated failure |
| CV-02 | P1 hardening | INV-19 | compiler recipe preserves semantic slot indices |
| CV-03 | P1 regression | INV-19, INV-21 | same trait on `T` and `U` uses distinct runtime slots |
| CV-04 | P1 regression | INV-30 | owner-generic evidence executes after receiver specialization |
| CV-05 | P1 regression | INV-30 | owner + method-local evidence execute together |
| CV-06 | P1 regression | INV-31 | exact enum-case conformance stays exact |
| CV-07 | P1 regression | INV-39 | repeated identical calls reuse conformance environments |
| CV-08 | P1 regression | INV-21 | concrete missing conformance is rejected |
| CV-09 | recovery | INV-25 | failed bounded call does not contaminate later analysis |
| CV-10 | syntax/kind | INV-15 | `where F: Type -> Type` contributes canonical kind |
| CV-11 | syntax/kind | INV-15 | equivalent binder + where kind facts agree |
| CV-12 | syntax/kind | INV-15 | conflicting kind facts diagnose |
| CV-13 | trait identity | INV-06 | wrong trait arity in generic bound rejects |
| CV-14 | trait identity | INV-06 | wrong ordinary trait argument kind rejects |
| CV-15 | parser | INV-03 | `T::Item` parses as associated projection |
| CV-16 | parser | INV-03, INV-08 | `<T as Trait>::Item` preserves subject/trait/name |
| CV-17 | parser | INV-05, INV-12 | `Trait<Item = T>` has structured specialization |
| CV-18 | parser | INV-12 | ordinary args + specialization parse distinctly |
| CV-19 | parser negative | INV-12 | positional arg after specialization rejects |
| CV-20 | parser negative | INV-11 | duplicate associated specialization rejects even identical RHS |
| CV-21 | formation | INV-09 | unique contextual `T::Item` forms canonical requirement |
| CV-22 | formation negative | INV-10 | contextual missing associated evidence diagnoses |
| CV-23 | formation ambiguity | INV-09, INV-10 | two traits with distinct `Item` requirements are ambiguous |
| CV-24 | qualification | INV-08 | qualified projection without conformance proof rejects |
| CV-25 | qualification | INV-03 | qualified projection forms explicit trait-owned requirement |
| CV-26 | ordering | INV-14 | equality before/after conformance anchor has same meaning |
| CV-27 | specialization | INV-02 | `I |= Iterator<Item = T>` expands to `Conforms + Equivalent` |
| CV-28 | specialization negative | INV-03 | unknown specialization associated name rejects |
| CV-29 | kind | INV-15 | specialization RHS must be proper Type |
| CV-30 | canonicalization | INV-04 | contextual/qualified spellings converge to same projection identity |
| CV-31 | canonicalization | INV-02, INV-14 | specialization sugar equals expanded conformance + equality fingerprint |
| CV-32 | canonicalization | INV-13 | `A == B` and `B == A` canonicalize identically |
| CV-33 | normalization | INV-22, INV-23 | exact associated evidence normalizes generic projection |
| CV-34 | normalization | INV-24 | assumed conformance without refinement stays valid symbolic projection |
| CV-35 | normalization | INV-23 | assumed specialization/equality refines projection |
| CV-36 | normalization terminal | INV-25 | recursive projection stays recursive |
| CV-37 | inference | INV-26 | known `I` + exact associated value determines `T` |
| CV-38 | inference | INV-26 | associated equality participates with expected-result evidence |
| CV-39 | inference | INV-28 | projection in parameter type participates in inference |
| CV-40 | inference | INV-28 | nested projection-bearing type participates in inference |
| CV-41 | inference negative | INV-27 | known associated output does not reverse-search unknown subject |
| CV-42 | forwarding | INV-29 | caller lexical associated equality satisfies callee equality |
| CV-43 | owner scope | INV-30 | owner + method constraints compose for projection formation |
| CV-44 | receiver | INV-30 | receiver specialization feeds associated subject substitution |
| CV-45 | Dynamic | INV-32 | Dynamic does not prove qualified projection conformance |
| CV-46 | exact case | INV-31 | exact enum-case associated binding normalizes without widening |
| CV-47 | metadata | INV-33 | stable projection type node round-trips |
| CV-48 | metadata | INV-33 | `Equivalent` containing projection round-trips |
| CV-49 | metadata | INV-33 | stable associated requirement identity validates |
| CV-50 | cross-module | INV-34 | imported specialized generic callable retains contract |
| CV-51 | cross-module inference | INV-34 | consumer infers associated-derived type from imported contract |
| CV-52 | reflection | INV-33 | projection-bearing equality is inspectable without trait-as-type lie |
| CV-53 | source index | INV-36 | contextual associated name targets canonical associated declaration |
| CV-54 | source index | INV-36 | qualified trait + associated names target canonical declarations |
| CV-55 | source index | INV-36 | specialization name targets trait-owned associated declaration |
| CV-56 | incremental | INV-35 | trait associated-surface change invalidates affected consumer |
| CV-57 | incremental no-op | INV-14, INV-35 | specialization sugar/expanded form preserve semantic fingerprint |
| CV-58 | regression | INV-22 | existing C5 `Self::Item` source/exact normalization remains green |
| CV-59 | regression | INV-21 | existing P1 direct generic conformance suite remains green |
| CV-60 | runtime boundary | INV-20, INV-37 | P2 adds no associated runtime lookup/slot behavior |
| CV-61 | method reference audit | INV-21, INV-30 | evidence-dependent callable reference is covered or explicitly deferred after verified audit |
| CV-62 | ambient compatibility | INV-16, INV-17 | P2 proof/normalization does not consult independent ambient vector state |
| CV-63 | terminal proof states | INV-25 | blocked/cancel/budget/internal projection proof states are not collapsed |
| CV-64 | publication | INV-33 | projection-bearing public generic signature exports; nonpublishable forms reject |

No positive P2 test may claim conditional conformance proof, blanket impls, Iterator⇒Iterable implication, production iteration migration, supertraits, trait objects, generic associated types, or higher-kinded trait-subject support.

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Sparse, discriminating feedback only. Use one or two exact tests per task until the architecture becomes coherent.

#### STABILIZE MODE

Run the P2 semantic capability module, directly affected C5/P1 regressions, metadata/incremental focus, P2 core runtime module, and LSP focus.

#### CERTIFY MODE

Broader crate/workspace verification is **not** automatically part of P2. Only run it if checkpoint workflow or explicit user instruction requests certification.

### 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

### 16.3 Mandatory during BUILD

Use commands equivalent to:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints::<exact_test>
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::<exact_regression>
cargo test -p phalcom-core --test core language::generic_associated_constraints::<exact_test>
```

Parser-only work should use the smallest `phalcom-ast` parser test command available live.

Metadata-only work may run an exact semantic/export/validation test without running core runtime tests.

### 16.4 Mandatory during STABILIZE

At plan completion, run at minimum:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
cargo test -p phalcom-core --test core language::generic_associated_constraints
cargo test -p phalcom-core --test core language::generic_conformance
```

Then run exact directly affected C5 associated projection tests, metadata tests, incremental fingerprint tests, and LSP associated-type tests named in the implementation walkthrough.

### 16.5 Only if evidence demands

- full `phalcom-ast` tests if parser changes reveal shared grammar regressions;
- full `phalcom-semantic` tests if focused semantic modules expose unexplained cross-capability breakage;
- full `phalcom-type-meta` tests if schema changes touch generic serialization broadly;
- full `phalcom-core` tests if runtime recipe hardening changes shared call-entry behavior.

### 16.6 Explicitly deferred / DO NOT RUN during ordinary BUILD

```text
cargo test --workspace
cargo clippy --workspace --all-targets
release/profile performance suites
broad examples corpus
production iterator migration tests unrelated to P2
all concurrency/fiber suites
all module-system suites
all LSP tests after each semantic edit
```

---

## 17. Baseline / unrelated failure policy

Classify every failure:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

- A/B: active P2 responsibility.
- C: perform one bounded classification pass; if nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks required acceptance.
- Do not weaken/skip assertions to make a broad gate green.
- Do not opportunistically repair legacy iteration/runtime failures outside P2.
- If an inherited C5/P1 baseline is reproduced through the new P2 vertical and shares the same root cause, reclassify only with concrete evidence.
- Record new deferred failures in the C6 checkpoint ledger.

The previously known Universe recomputation/capability baseline remains unrelated unless P2 source changes demonstrably alter its causal path.


---

# 18. Tasks

## T1 — Harden P1 evidence layout/transport and close prerequisite regressions

### Purpose

Remove two source-level hazards that would make richer P2 evidence unsound: silent generic evidence-layout overflow and positional runtime recipe compaction. Add the missing narrow P1 regressions that P2 depends upon.

### Preconditions

- P1 direct conformance suite is green at the starting revision according to the P1 walkthrough/handoff.
- live code still matches one or both identified hazards, or equivalent regression tests demonstrate the issue.

### Consumes

- `GenericEvidenceLayout::build`
- `GenericEvidenceEnvironment`
- `GenericEvidenceSlotKey`
- `RuntimeConformanceSlotRecipe`
- `RuntimeConformanceEnvironment`
- P1 exact/forwarded call-specialization products

### Produces

- checked evidence-environment construction;
- exact preservation of semantic slot positions in runtime recipes;
- focused P1 substrate regressions for owner evidence, same-trait/different-subject slots, exact cases, failure isolation, and interning.

### Files and symbols

**Read:**
- `phalcom-semantic/src/generic_evidence.rs`
- `phalcom-semantic/src/checker/body.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/src/typing/environment.rs`
- `phalcom-core/src/typing/conformance_environment.rs`
- `phalcom-core/src/vm/dispatch.rs`

**Modify as needed:**
- `phalcom-semantic/src/generic_evidence.rs`
- direct callers of checked environment construction
- `phalcom-core/src/modules/compile.rs`
- minimal runtime registry inspection seam only if required for cardinality test

**Tests:**
- extend `phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs`
- extend `phalcom-core/tests/core/language/generic_conformance.rs`
- add a narrow unit test near `GenericEvidenceLayout` if generating >65k real trait requirements is unreasonable

### Required implementation shape

#### A. Checked layout construction

Replace silent defaulting equivalent to:

```rust
GenericEvidenceLayout::build(...).unwrap_or_default()
```

with a checked path.

Preferred shape:

```rust
pub fn try_from_constraints(...) -> Result<GenericEvidenceEnvironment, GenericEvidenceLayoutError>
```

and:

```rust
pub fn try_compose(...) -> Result<GenericEvidenceEnvironment, GenericEvidenceLayoutError>
```

or an equivalent checked constructor that cannot erase overflow.

Map capacity failure into the repository's structured semantic invalid/blocked/error path. Do not reinterpret it as missing evidence.

#### B. Preserve slot position

When materializing callee runtime evidence, allocate recipe entries against the complete semantic layout:

```text
semantic slot 0 -> recipe slot 0
semantic slot 1 -> recipe slot 1
...
```

Do not create semantic indexing by repeated successful `Vec::push`.

If a required behavioral slot cannot materialize, return a structured compile error. Do not omit the slot or fall back to runtime lookup.

If `RequirementSelectionTemplate::DataComponent` can appear for a behavioral generic evidence slot, trace its established C5 execution path. Reuse that path or STOP AND CONSULT.

#### C. P1 prerequisite regressions

Add focused coverage for:

```text
T |= Trait
U |= Trait
```

with different runtime values selecting different witnesses.

Add owner-generic execution through a receiver-specialized instance method.

Add owner + method-local evidence composition if current surface supports it cleanly.

Add exact enum-case conformance proof/execution.

Add concrete missing-conformance rejection and failed-call isolation.

Add environment reuse/cardinality evidence. If no public count exists, add a minimal `len()`/test inspection API to `RuntimeConformanceEnvironmentRegistry`; do not expose semantic lookup.

### T1 clarification — receiver-owner call finalization (2026-09-16)

Authorized independent correction includes `checker/call.rs`,
`checker/expression.rs`, and `checker/context.rs`. Receiver-fixed owner bindings
must remain outside method-local inference activation. Successful application
finalizes a single specialization containing owner and method evidence; abstract
receiver bindings forward lexical evidence. Failed checking must not publish a
partial executable specialization. Semantic body checking, lowering, and compiler
recipe formation use checked evidence construction/composition.

Per user scope, exact-variant conformance/inference and enum lowering are excluded
from this correction. Preserve their existing edits and defer their T1 gate.

### Execution sequence

- [ ] Write overflow regression proving current silent fallback is invalid.
- [ ] Run only that test and confirm failure on the live hazard.
- [ ] Implement checked construction and propagate failure.
- [ ] Run overflow regression and existing evidence-layout tests.
- [ ] Write slot-position regression with same trait/different subjects.
- [ ] Harden compiler recipe construction against positional drift.
- [ ] Add owner-generic / exact-case / failure-isolation / reuse regressions.
- [ ] Run only P1 semantic/core generic-conformance modules.
- [ ] Record durable hardening invariants in C6 checkpoint if production interfaces changed.

### Forbidden approaches

- no runtime conformance search fallback;
- no increasing slot width merely to avoid checked failure;
- no truncation/saturation;
- no source-order evidence indexing;
- no associated-type runtime slots;
- no broad VM redesign;
- no P2 projection implementation yet.

### Test changes required

Covers:

```text
CV-01 CV-02 CV-03 CV-04 CV-05 CV-06 CV-07 CV-08 CV-09
```

### Tests to run now

Use exact discovered names. Minimum module gates:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
cargo test -p phalcom-core --test core language::generic_conformance
```

Do not run workspace tests.

### Tests explicitly deferred

- new P2 syntax/projection tests;
- metadata projection tests;
- LSP tests;
- full semantic/core crates.

### Acceptance

P1 evidence construction cannot silently erase layout overflow; runtime recipe position is provably identical to semantic slot identity; P1 direct generic conformance still executes; owner-generic and same-trait/distinct-subject cases are covered; exact-case identity and environment reuse have focused evidence.

### Local STOP / CONSULT triggers

- a behavioral generic evidence slot legitimately has no executable runtime target;
- preserving slot position requires changing VM slot identity/lifetime semantics;
- owner-generic execution requires a new persistent evidence-capture architecture;
- exact enum-case evidence is widened before P2 touches it;
- registry interning changes would affect GC/rooting policy.

### Checkpoint update

Record T1 as a durable P1-hardening gate only if production interfaces changed. Do not mark P2 projection semantics active yet.

---

## T2 — Add structured generic-associated syntax and canonical where-kind syntax

### Purpose

Create source/AST structures that distinguish ordinary trait type arguments, associated specializations, qualified projection, and kind constraints without contaminating semantic identity.

### Preconditions

- T1 accepted.
- authoritative generic-constraints grammar re-read.
- live parser structure for type forms and where clauses verified.

### Consumes

- current `GenericConstraintSyntax`
- current `TypeAnnotationExpr::AssociatedTypeProjection`
- `KindSyntax`
- current `|=` / `==` where grammar

### Produces

- structured trait requirement syntax;
- associated specialization syntax;
- qualified associated projection syntax;
- where-clause kind-fact syntax;
- parser diagnostics for duplicate specialization and positional-after-specialization.

### Files and symbols

**Modify:**
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- token/lexer only if live grammar genuinely lacks a required token

**Tests:**
- existing `phalcom-ast` parser tests in live location

### Required implementation shape

Introduce a structured AST concept equivalent to:

```rust
pub struct TraitRequirementSyntax {
    pub trait_ref: TraitReferenceSyntax,
    pub associated_specializations: Vec<AssociatedSpecializationSyntax>,
    pub range: SourceRange,
}

pub struct AssociatedSpecializationSyntax {
    pub name: String,
    pub name_range: SourceRange,
    pub value: TypeAnnotation,
    pub range: SourceRange,
}
```

Exact names are flexible.

Change `GenericConstraintSyntax::Conforms` to retain bounds as structured trait requirements rather than opaque `TypeAnnotation` if necessary.

Parse:

```phalcom
T |= Printable
T |= Converts<A, B>
T |= Iterator<Item = A>
T |= Converts<A, B, Error = E>
T |= A + B<Item = X>
```

Once a named `Assoc = Type` specialization appears in a trait argument list, reject later positional arguments:

```phalcom
Trait<A, Item = B, C>
```

Reject duplicate specialization even if RHS values are identical.

Parse qualified projection:

```phalcom
<T as Iterator>::Item
<T as Converts<A, B>>::Error
```

Preserve source spans for subject, trait reference, associated name, and whole projection.

Add where kind fact:

```phalcom
where F: Type -> Type
```

as a syntax-level generic fact. It must later feed parameter kind resolution, not become semantic `GenericConstraint::Kind`.

### Execution sequence

- [ ] Add failing parser tests for all positive forms.
- [ ] Add parser diagnostics for duplicate/ordering negatives.
- [ ] Add AST carriers with full source spans.
- [ ] Implement parsing without semantic lowering.
- [ ] Add `where F: ...` syntax fact.
- [ ] Run exact parser tests only.
- [ ] Verify old direct `|=` and `+` parsing remains intact.

### Forbidden approaches

- no parsing associated specialization as an extra positional type argument;
- no storing `Item = T` inside `TraitRef`;
- no stringly encoded qualified projection;
- no semantic associated lookup in parser;
- no duplicate specialization deduplication;
- no solver edits in this task.

### Test changes required

Covers:

```text
CV-10 CV-11 CV-12
CV-15 CV-16 CV-17 CV-18 CV-19 CV-20
```

CV-11/CV-12 semantic behavior completes in T3.

### Tests to run now

Run exact `phalcom-ast` parser tests. Use `cargo test -p phalcom-ast -- --list` to discover current filter names.

> **Do not run semantic/runtime integration tests after this task. The intermediate state is intentionally incomplete.**

### Tests explicitly deferred

- semantic trait identity;
- contextual projection resolution;
- inference;
- metadata;
- core runtime.

### Acceptance

AST can losslessly distinguish ordinary trait arguments, associated specialization, qualified projection, contextual projection, and where-kind facts. Invalid duplicate/ordering syntax is diagnosed before semantic canonicalization.

### Local STOP / CONSULT triggers

- authoritative grammar makes associated specialization materially ambiguous with ordinary type syntax;
- qualified projection conflicts with an existing type grammar in a way requiring semantic redesign;
- parser support would require changing associated identity.

### Checkpoint update

No checkpoint update unless the AST carrier becomes a durable cross-plan interface explicitly worth recording.

---

## T3 — Stage generic signature formation and canonicalize projection/equality propositions

### Purpose

Make generic signature formation source-order-independent and capable of forming canonical direct conformance anchors, contextual/qualified projection identities, associated specialization, symmetric equality, and unified parameter kinds.

### Preconditions

- T2 syntax available.
- T1 checked generic evidence construction available.
- `TraitRef::form_identity` and C5 projection interning verified live.

### Consumes

- structured T2 syntax
- `TraitRef::form_identity`
- `GenericConstraint::{Conforms, Equivalent, Subtype}`
- `AssociatedTypeRequirementId`
- trait surfaces
- type-level parameter resolver
- `GenericEvidenceEnvironment`

### Produces

- staged `resolve_generic_signature`;
- canonical direct conformance seed set;
- generic contextual projection formation;
- qualified projection formation;
- specialization elaboration;
- canonical symmetric equality orientation;
- unified where/binder kind resolution;
- a generic-evidence-aware trait application well-formedness seam.

### Files and symbols

**Modify:**
- `phalcom-semantic/src/types/annotation.rs`
- `phalcom-semantic/src/types/parameter.rs` only for canonicalization helpers if appropriate
- `phalcom-semantic/src/traits.rs`
- trait-surface lookup helpers as needed

**Tests:**
- create `phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs`
- register in `phalcom-semantic/tests/semantic/capabilities/mod.rs`
- focused type-annotation tests if useful

### Required implementation shape

#### A. Unify parameter kind facts before parameter interning

Collect binder-local and where-clause kind facts for each generic parameter.

Rules:

```text
none + none   -> Type default
one source    -> that kind
same + same   -> one canonical kind
conflict      -> diagnostic + invalid
```

Do not publish `GenericConstraint::Kind`.

#### B. Canonical source trait identity formation

Replace source generic bound construction equivalent to:

```rust
TraitRef::new(declaration, arguments)
```

with `TraitRef::form_identity(...)` or its live canonical equivalent.

Map `NotTrait`, arity, kind, and invalid argument failures to source diagnostics.

#### C. Separate trait identity from application well-formedness

Establish an operation equivalent to:

```rust
validate_trait_application(
    trait_ref: &TraitRef,
    evidence: &GenericEvidenceEnvironment,
    proof_context: ...
) -> TraitApplicationWellFormedness
```

For P2, handle direct currently-supported generic contracts. If validation requires P3 recursive conditional proof, return an honest blocked/deferred state and consult if normative behavior requires it now.

#### D. Build direct conformance anchors first

Resolve all direct trait requirements to canonical `Conforms` anchors independent of source order.

Associated specialization contributes the direct `Conforms` anchor immediately; its equality elaboration waits until projection identity can form.

Build provisional evidence from lexical owner evidence plus all current-signature direct anchors.

#### E. Contextual generic projection

For `T::Item`:

1. resolve subject;
2. inspect applicable direct conformances for that subject;
3. inspect corresponding trait surfaces;
4. retain traits with associated requirement `Item`;
5. canonicalize candidates by `AssociatedTypeRequirementId`;
6. require exactly one distinct identity.

Do not global-scan traits and do not choose first source match.

#### F. Qualified projection

For `<T as Iterator>::Item`:

1. form subject;
2. form canonical `TraitRef`;
3. resolve `Item` in that trait surface;
4. prove `T |= Iterator`;
5. intern canonical projection.

Qualification is not proof.

#### G. Specialization elaboration

For:

```phalcom
I |= Iterator<Item = T>
```

publish:

```text
Conforms(I, Iterator)
Equivalent(<I as Iterator>::Item, T)
```

Use the same qualified projection identity helper as standalone qualified syntax.

#### H. Canonical equality orientation

Before sort/dedup, orient `Equivalent(left, right)` by canonical semantic term ordering so `A == B` and `B == A` publish identically.

### Execution sequence

- [ ] Add tests for trait arity/kind errors in generic bounds.
- [ ] Integrate canonical `TraitRef` identity formation.
- [ ] Add kind-source merge/conflict tests and implement unified kinds.
- [ ] Add source-order permutation test where equality precedes conformance.
- [ ] Refactor signature formation into staged passes.
- [ ] Add contextual success/missing/ambiguity tests.
- [ ] Add qualified success/missing-conformance tests.
- [ ] Add specialization elaboration test inspecting canonical `GenericSignature`.
- [ ] Add equality symmetry and sugar/expanded fingerprint tests.
- [ ] Run only new semantic module plus exact P1 regressions.

### Forbidden approaches

- no raw source `TraitRef::new` as validation;
- no source-order projection resolution;
- no global associated-name search;
- no first-match ambiguity resolution;
- no specialization-bearing `TraitRef`;
- no new associated-specific semantic relation;
- no inference implementation yet;
- no P3 conditional proof.

### Test changes required

Covers:

```text
CV-10 CV-11 CV-12 CV-13 CV-14
CV-21 CV-22 CV-23 CV-24 CV-25 CV-26
CV-27 CV-28 CV-29 CV-30 CV-31 CV-32
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::generic_conformance_constraint_lowering_and_ordering
```

Use narrower filters during BUILD.

### Tests explicitly deferred

- projection-driven call inference;
- cross-module metadata;
- core execution;
- LSP.

### Acceptance

Generic signatures are source-order-independent; direct trait identity is validated canonically; contextual/qualified projections form canonical identities; associated specialization lowers to ordinary propositions; equality symmetry is canonicalized; kinds have one authority.

### Local STOP / CONSULT triggers

- contextual projection requires global conformance scanning;
- trait application validation cannot be separated from P3 recursive proof;
- specialization cannot converge to explicit `Equivalent` without semantic redesign;
- equality canonicalization conflicts with a direction-sensitive meaning of current `Equivalent`.

### Checkpoint update

Record durable projection formation and specialization elaboration interfaces after focused tests pass.

---

## T4 — Extend GenericEvidenceEnvironment with equality/refinement and generic projection normalization

### Purpose

Make lexical generic associated equality queryable through the P1 evidence environment and extend the C5 normalizer to consume exact or assumed/refined generic evidence without creating another normalization authority.

### Preconditions

- T3 canonical projection/equality propositions publish correctly.
- C5 normalizer architecture re-read.
- direct generic proof API remains available.

### Consumes

- canonical `GenericConstraint::Equivalent`
- canonical `GenericConstraint::Conforms`
- `GenericEvidenceEnvironment`
- `query_generic_conformance_proof`
- `TypeData::AssociatedProjection`
- `types::normalize_type`
- exact C5 associated binding evidence

### Produces

- efficient lexical equivalence/refinement view;
- canonical generic equality query;
- generic projection normalization context/mode;
- exact vs assumed symbolic/refined outcomes with full terminal-state preservation.

### Files and symbols

**Modify:**
- `phalcom-semantic/src/generic_evidence.rs`
- live owner of `normalize_type` / projection normalization context
- small helpers in `types/parameter.rs` / `types/store.rs` if required

**Tests:**
- `phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs`
- directly affected C5 associated projection tests

### Required implementation shape

#### A. Equality/refinement view

Build an environment-owned representation capable of answering:

```text
are terms canonically equivalent under lexical facts?
does projection P have a direct unique refinement?
which facts support the result?
```

A union-find/equivalence-class representation is acceptable if it preserves provenance and does not become a second inference solver. A bounded indexed graph is also acceptable.

#### B. Equality proof

Provide one semantic query equivalent to:

```rust
query_generic_equivalence(
    left,
    right,
    evidence_env,
    normalization_context
) -> GenericEquivalenceProof
```

Result categories must preserve proven/not-proven/ambiguous/blocked/dynamic/recursive/cancel/budget/internal distinctions, whether via a new enum or existing outcome types.

Proof may use:

```text
canonical identity
lexical Equivalent facts
symmetry
finite transitivity
projection normalization
```

Do not expand to unrestricted theorem proving.

#### C. Generic projection normalization

Extend existing normalization context so projection can use exact or generic evidence.

- exact conformance: reuse C5 associated binding;
- assumed conformance + unique refinement: recursively normalize refinement;
- assumed conformance without refinement: retain symbolic projection;
- missing proof: reject/unknown per formation context;
- preserve terminal outcomes.

#### D. Equivalence-aware conformance matching

If lexical facts prove:

```text
T == U
U |= Iterator
```

then projection/conformance reasoning for `T` may use the actual evidence supplied for `U`.

Preserve the actual supplying `GenericConformanceKey`; do not fabricate a second runtime slot.

### Execution sequence

- [ ] Add assumed symbolic and assumed refined normalization tests.
- [ ] Add exact normalization through generic subject test.
- [ ] Add equality transitivity/symmetry test needed by forwarding.
- [ ] Implement equality/refinement index inside `GenericEvidenceEnvironment`.
- [ ] Extend normalizer to consume generic evidence.
- [ ] Add recursive/blocked terminal-state tests.
- [ ] Add equivalence-aware conformance lookup test.
- [ ] Run new semantic module + exact C5 normalization tests.

### Forbidden approaches

- no second recursive normalizer;
- no `Iterator` special case;
- no arbitrary equality-neighbor substitution;
- no dropping proof provenance;
- no equality-derived runtime slots;
- no P3 conditional recursion.

### Test changes required

Covers:

```text
CV-33 CV-34 CV-35 CV-36 CV-58 CV-62 CV-63
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints::<normalization filters>
```

Then run exact current C5 normalization regressions discovered via `-- --list`.

### Tests explicitly deferred

- generic call inference;
- metadata;
- core runtime;
- LSP.

### Acceptance

One lexical evidence environment can prove/refine generic associated equalities; exact and assumed projection normalization both go through `types::normalize_type`; assumed-only projection remains symbolic; terminal states remain explicit.

### Local STOP / CONSULT triggers

- generic equality requires a second recursive normalizer;
- equivalence-aware conformance lookup changes runtime evidence slot identity;
- transitive equality requires unbounded theorem solving;
- C5 normalizer cannot accept generic evidence without changing canonical ownership.

### Checkpoint update

Record generic equality/refinement query and generic projection normalization as durable P2 interfaces.

---

## T5 — Make declaration/member signature formation projection-aware across owner and method evidence

### Purpose

Ensure generic projections are usable where ordinary types are usable in published class/method signatures, with correct owner + method lexical evidence composition and receiver specialization.

### Preconditions

- T3 formation and T4 normalization/equality are available.
- current declaration signature construction order verified.

### Consumes

- owner generic signature
- method generic signature
- `GenericEvidenceEnvironment`
- projection-aware type formation context
- existing receiver specialization/type substitution

### Produces

- projection-aware parameter/return type formation;
- owner generic projection formation;
- composed owner + method evidence context;
- preserved receiver specialization semantics;
- bounded audit of `ambient_constraints`.

### Files and symbols

**Modify:**
- `phalcom-semantic/src/checker/declaration_signature.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/body.rs`
- type formation site/context as required

**Tests:**
- `phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs`
- extend P1 owner composition tests if useful

### Required implementation shape

Current method signature construction already resolves callable generics before parameter/return annotations. Preserve that dependency.

After method generic signature formation, construct one lexical formation environment from:

```text
declaration owner GenericSignature
+
callable GenericSignature
```

and use it to resolve parameter/return projections.

Owner-generic evidence must also be available to other declaration signature positions that current language semantics allow.

Receiver specialization remains ordinary type substitution. A specialized receiver binding such as `I = UserIterator` is established by existing receiver/generic machinery; projection normalization then consumes that fact.

Audit remaining `ambient_constraints` consumers. P2 proof/normalization logic must use `generic_evidence`. If the vector remains necessary for conditional inherent-member compatibility, document it as a derived compatibility view.

### Execution sequence

- [ ] Add failing owner-generic signature projection test.
- [ ] Add owner + method projection/equality composition test.
- [ ] Thread lexical evidence into parameter/return formation.
- [ ] Confirm body and signature use the same projection `TypeId`.
- [ ] Audit `ambient_constraints` reads with `rg`.
- [ ] Migrate P2-relevant reasoning to `generic_evidence`.
- [ ] Add receiver specialization regression.
- [ ] Run focused semantic module and P1 owner-evidence regression.

### Forbidden approaches

- no body-only projection support while published signatures remain unsupported;
- no method-local evidence model that ignores owner evidence;
- no receiver-specialization bypass;
- no P2 reasoning added directly to ambient vector;
- no runtime associated type environment.

### Test changes required

Covers:

```text
CV-39 CV-43 CV-44 CV-62
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints::<owner/signature filters>
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::owner_and_callable_conformance_assumptions_compose_lexically
```

### Tests explicitly deferred

- associated-derived call inference;
- metadata/cross-module;
- LSP.

### Acceptance

Owner and method generic evidence form one lexical projection context; projection-bearing parameter/return types publish canonical semantic types; receiver specialization remains the source of owner type substitution; no P2 reasoning depends on an independent ambient vector.

### Local STOP / CONSULT triggers

- owner signature formation requires a declaration-pipeline redesign;
- receiver specialization is unavailable where projection semantics need it and the apparent workaround is runtime lookup;
- evidence-aware signature formation creates a cycle between trait surface and generic signature publication.

### Checkpoint update

Record owner/method lexical projection composition if it becomes a durable interface.


---

## T6 — Add projection-aware generic inference and call-site equality discharge

### Purpose

Allow associated evidence to contribute new generic type solutions after the projection subject is known, including generic-to-generic forwarding of associated equality, while preserving P1 inference staging and preventing reverse open-world search.

### Preconditions

- T4 generic projection normalization/equality query works.
- T5 projection-bearing signatures form.
- existing inference session architecture re-read.

### Consumes

- `InferenceTerm`
- `GenericInstantiation`
- `GenericApplicationDomains`
- generic argument/expected-result constraints
- `query_generic_conformance_proof`
- generic equality/refinement query
- `normalize_type`
- `CallSpecialization`

### Produces

- projection-aware inference term/deferred structure;
- bounded associated feedback loop;
- associated-derived parameter solutions;
- call-site equality validation/forwarding;
- truthful diagnostics for underconstrained/conflicting associated inference.

### Files and symbols

**Modify:**
- `phalcom-semantic/src/checker/inference.rs`
- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/checker/row_inference.rs` only where structural traversal requires projection support
- `phalcom-semantic/src/types/instantiation.rs` / substitution only if projection traversal is incomplete
- `phalcom-semantic/src/checker/analysis.rs` only if call specialization needs additional semantic facts

**Tests:**
- `phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs`

### Required implementation shape

#### A. Projection-aware inference representation

Preferred:

```rust
InferenceTerm::AssociatedProjection {
    subject: Box<InferenceTerm>,
    trait_ref: InferenceTraitRef,
    requirement: AssociatedTypeRequirementId,
}
```

with ordinary trait arguments represented as inference terms.

Exact spelling is flexible.

`type_id_to_inference` and row-aware conversion must recurse through projection subject and trait arguments.

`materialize` must reconstruct canonical projection when enough structure is canonical.

Audit every `InferenceTerm` visitor:

```text
term_variables
occurs check
term support/proof
substitution
materialization
structural comparison
row-variable traversal
diagnostic projected-parameter discovery
```

No visitor may silently treat associated projection as opaque if it can contain current inference variables.

#### B. Associated feedback worklist

After ordinary argument/expected constraints produce a partial or solved substitution:

1. instantiate relevant `Conforms` and `Equivalent` propositions;
2. identify projection equalities whose subjects/trait args are sufficiently materialized;
3. prove required conformance;
4. normalize/refine projection;
5. inject resulting equality as an inference relation with correct support/provenance;
6. propagate/solve;
7. repeat only if new substitutions or projection resolutions appeared.

Bound iterations by work-item state and existing query budget/cancellation.

Stable unresolved obligations terminate as underconstrained.

#### C. One-way information discipline

Do not schedule reverse search from known RHS.

This remains underconstrained:

```text
<I as Iterator>::Item == User
I unresolved
```

This may solve:

```text
I = UserIterator
<I as Iterator>::Item == T
```

when exact evidence establishes `UserIterator::Item = User`.

#### D. Call-site conformance + equality discharge

P1 direct conformance still produces exact or forwarded behavioral evidence.

P2 additionally validates instantiated `Equivalent` constraints.

Generic-to-generic caller equality can discharge callee equality at compile time. No runtime equality slot exists.

If equality is unproven, diagnose generic constraint unsatisfied.

If blocked/ambiguous, preserve corresponding analysis status.

#### E. Result publication

When associated evidence solves `T`, publish the normal type binding through existing `CallSpecialization.bindings`.

Do not add an associated substitution map.

### Execution sequence

- [ ] Add unit/semantic test showing projection structural conversion to inference form.
- [ ] Extend inference traversal/materialization comprehensively.
- [ ] Add failing `I -> Item -> T` inference case.
- [ ] Implement bounded associated worklist.
- [ ] Add expected-result + associated equality case.
- [ ] Add projection-in-parameter and nested projection tests.
- [ ] Add reverse-search negative.
- [ ] Add generic-to-generic associated equality forwarding.
- [ ] Add Dynamic and terminal-state diagnostics.
- [ ] Run P2 semantic module.
- [ ] Run directly affected generic inference regressions only.

### Forbidden approaches

- no `Iterator` special case;
- no scanning exact conformance index for arbitrary unknown subjects;
- no second substitution map;
- no retry-until-success loop without progress measure;
- no blocked→missing collapse;
- no runtime inference;
- no P3 recursive conditional candidate search.

### Test changes required

Covers:

```text
CV-37 CV-38 CV-39 CV-40 CV-41 CV-42 CV-45 CV-63
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints
```

Then exact existing generic inference tests identified by current test organization.

### Tests explicitly deferred

- stable metadata;
- imported cross-module inference;
- core runtime integration;
- full semantic suite.

### Acceptance

A known subject plus exact/generic associated evidence can solve another type parameter; projections nested in ordinary type positions participate in inference; generic-to-generic equality discharges compile-time; unknown subjects do not trigger reverse search; terminal states remain truthful.

### Local STOP / CONSULT triggers

- projection-aware inference requires replacing the current solver wholesale;
- worklist cannot be bounded without global search;
- needed equality requires P3 conditional conformance recursion;
- row inference interaction requires a second row solver;
- inference support/provenance becomes ambiguous or unsound.

### Checkpoint update

Record projection-aware inference as a durable P2 gate after focused tests pass.

---

## T7 — Add durable stable metadata, reflection, and cross-module reconstruction for projections

### Purpose

Make projection-bearing public generic contracts durable across stable metadata and module boundaries rather than source-local checker facts.

### Preconditions

- T3 canonical projections/equalities established.
- T6 call inference works source-locally.
- metadata schema/version rules re-read.

### Consumes

- semantic `TypeData::AssociatedProjection`
- `StableTraitApplicationRef`
- semantic/stable generic constraints
- metadata exporter/importer/validator
- reflection APIs

### Produces

- stable associated projection type node;
- stable associated requirement identity;
- projection export/import/validation/fingerprint support;
- relation-aware reflection;
- cross-module associated-specialized generic inference.

### Files and symbols

**Modify:**
- `phalcom-type-meta/src/type_node.rs`
- `phalcom-type-meta/src/generic.rs` or shared stable trait identity module
- `phalcom-type-meta/src/validate.rs`
- `phalcom-type-meta/src/fingerprint.rs` if explicit
- metadata schema/version constants where required
- `phalcom-semantic/src/metadata/export.rs`
- semantic metadata import/reconstruction path
- reflection/typing inspection path used by P1

**Tests:**
- `phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs`
- metadata validation/export tests in live location
- `phalcom-core/tests/core/language/generic_associated_constraints.rs`

### Required implementation shape

#### A. Stable associated requirement identity

Introduce one stable identity equivalent to:

```rust
pub struct StableAssociatedTypeRequirementRef {
    pub owner: StableDeclarationRef,
    pub index: u32,
}
```

An equivalent representation may omit redundant owner if validation through the enclosing trait application is stronger.

Name may be retained for display/provenance but must not replace canonical identity.

#### B. Stable projection type node

Add a general stable node equivalent to:

```rust
TypeNode::AssociatedProjection {
    subject: TypeNodeId,
    trait_application: StableTraitApplicationRef,
    requirement: StableAssociatedTypeRequirementRef,
}
```

It must be a normal type graph node so projections can occur in equality operands, callable types, tuples, records, applications, and future consumers.

#### C. Export/import

Replace current projection `NonExportableForm` behavior.

Export recursively:

```text
subject
ordinary trait args
stable trait declaration
stable associated requirement
kind
structural fingerprint
```

Import reconstructs canonical semantic projection against the live trait surface.

#### D. Validation

Reject at least:

```text
missing subject node
missing trait argument node
invalid stable trait declaration
associated requirement not owned by qualifying trait
invalid projection kind
malformed node/index references
```

#### E. Reflection

P1 relation-aware reflection must expose projection-bearing `Equivalent` without pretending traits are value types.

#### F. Cross-module discriminator

Provider exports a generic method equivalent to:

```phalcom
nextValue<I, T>(_ iterator: I) -> Option<T>
    where I |= Iterator<Item = T>
```

Consumer passes exact iterator with `Item = User`.

Consumer reconstructs and uses `T = User`.

### Execution sequence

- [ ] Add stable node/validator unit tests.
- [ ] Add exporter regression proving projection-bearing signature exports.
- [ ] Implement stable associated requirement identity.
- [ ] Implement stable projection node + fingerprinting.
- [ ] Implement semantic export/import reconstruction.
- [ ] Add metadata validation negatives.
- [ ] Extend reflection tests.
- [ ] Add cross-module provider/consumer semantic test.
- [ ] Add core execution test.
- [ ] Run focused metadata + cross-module tests.

### Forbidden approaches

- no semantic `TypeId` serialization;
- no semantic `TraitRef` serialization;
- no name-only associated identity;
- no constraint-only projection encoding;
- no source reparsing on import;
- no runtime projection resolver.

### Test changes required

Covers:

```text
CV-47 CV-48 CV-49 CV-50 CV-51 CV-52 CV-64
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints::<metadata filters>
cargo test -p phalcom-core --test core language::generic_associated_constraints::<cross-module filter>
```

Run exact `phalcom-type-meta` validator tests if placed there.

### Tests explicitly deferred

- LSP/source index;
- full metadata crate;
- workspace.

### Acceptance

Projection-bearing public generic contracts export, validate, import, reflect, and execute across module boundaries with canonical identity and no snapshot-local IDs.

### Local STOP / CONSULT triggers

- stable projection identity cannot be reconstructed without source;
- schema change requires compatibility/versioning governance outside normal rules;
- stable trait identity duplication appears necessary;
- associated requirement identity would rely on names rather than C5 nominal identity;
- cross-module inference depends on P3 proof.

### Checkpoint update

Record stable projection metadata and cross-module gate.

---

## T8 — Integrate source index, editor/LSP, and incremental dependency/fingerprint semantics

### Purpose

Make P2 source forms editor-visible and incrementally correct using canonical semantic identities.

### Preconditions

- T7 durable projection identity exists.
- semantic formation exposes canonical ranges/targets.

### Consumes

- `AssociatedTypeRequirementId`
- `TraitRef`
- structured AST ranges
- source index target products
- semantic DB fingerprints/dependencies
- LSP associated hover/navigation infrastructure

### Produces

- go-to-definition/hover for contextual/qualified/specialization forms;
- canonical trait and associated targets;
- targeted invalidation on trait associated-surface changes;
- semantic no-op fingerprints for equivalent source forms.

### Files and symbols

**Modify as needed:**
- `phalcom-semantic/src/source_index/builder.rs`
- semantic target/occurrence owner
- `phalcom-semantic/src/editor.rs`
- `phalcom-semantic/src/db/fingerprint.rs`
- dependency/query declarations
- `phalcom-lsp/src/backend.rs`

**Tests:**
- `phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs`
- `phalcom-semantic/tests/semantic/incremental/fingerprints.rs`
- `phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs`
- live LSP associated navigation/hover test file

### Required implementation shape

#### A. Source index

For `I::Item`, map `Item` to canonical `AssociatedTypeRequirementId`.

For `<I as Iterator>::Item`, map `Iterator` to trait `DeclarationId` and `Item` to associated identity.

For `Iterator<Item = T>`, map specialization name `Item` through the already-resolved trait requirement.

Never globally resolve by spelling.

#### B. LSP/editor

Hover/navigation must use canonical semantic target.

Ambiguous contextual projection must not produce arbitrary definition.

Use canonical `::` associated spelling.

#### C. Incremental dependencies

Projection formation depends on:

```text
generic signature conformance facts
trait header identity
trait surface associated requirements
associated requirement identity/name
```

Projection normalization additionally may depend on exact conformance associated binding and generic equality/refinement.

Use existing dependency products; do not blanket-invalidate modules/workspace.

#### D. Fingerprints

These must share semantic fingerprint when otherwise equivalent:

```phalcom
where I |= Iterator<Item = T>
```

and:

```phalcom
where I |= Iterator, <I as Iterator>::Item == T
```

Likewise `A == B` / `B == A`.

Source fingerprints may differ.

### Execution sequence

- [ ] Add source-index tests for contextual/qualified/specialization forms.
- [ ] Add ambiguity navigation negative.
- [ ] Add semantic fingerprint parity tests.
- [ ] Add associated-surface invalidation test.
- [ ] Implement source target mapping.
- [ ] Add/adjust dependency edges.
- [ ] Update LSP only where semantic editor layer does not already make behavior automatic.
- [ ] Run exact incremental and LSP tests.

### Forbidden approaches

- no LSP text scanning for associated owner;
- no global invalidation;
- no source-spelling semantic fingerprint;
- no arbitrary ambiguity target.

### Test changes required

Covers:

```text
CV-53 CV-54 CV-55 CV-56 CV-57
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints::<source-index filters>
cargo test -p phalcom-semantic --test semantic incremental::fingerprints::<exact P2 filter>
cargo test -p phalcom-semantic --test semantic incremental::recomputation_metrics::<exact P2 filter>
```

Then exact live LSP associated tests.

### Tests explicitly deferred

- full LSP suite;
- workspace.

### Acceptance

New syntax navigates to canonical declarations; ambiguity has no arbitrary target; equivalent semantic forms share fingerprints; associated-surface edits invalidate affected consumers under existing dependency architecture.

### Local STOP / CONSULT triggers

- canonical target representation for associated requirement is absent;
- dependency correctness requires whole-workspace invalidation;
- LSP must independently solve generic evidence.

### Checkpoint update

Record incremental/tooling gate if passed.

---

## T9 — Audit generic method references and close the runtime boundary

### Purpose

Perform the bounded P1/P2 audit of callable references whose execution may depend on generic evidence, prove P2 requires no associated runtime mechanism, and add only the regression supported by current callable-reference architecture.

### Preconditions

- T1 runtime hardening passed.
- T5 owner/method evidence composition exists.
- callable-reference semantic/lowering path re-read.

### Consumes

- current callable reference products
- bound-method environment propagation
- runtime type/conformance environment semantics
- generic evidence-dependent body dispatch

### Produces

One of two honest outcomes:

**Outcome A — supported without architectural change**
- focused semantic/core regression proving referenced method execution retains/reconstructs required generic environments.

**Outcome B — unsupported without new capture/lifetime architecture**
- explicit checkpoint/handoff deferral describing the boundary;
- no speculative partial implementation.

### Files and symbols

**Read:**
- callable-reference semantic resolution
- core lowering for callable references
- closure/bound-method environment storage
- P1 generic call environment propagation

**Modify only if Outcome A is mechanical:**
- relevant callable-reference tests
- minimal environment propagation fix preserving existing architecture

### Required implementation shape

Test a method whose body contains assumed trait dispatch derived from owner or method generic evidence.

If invocation already uses existing call environment machinery, add regression and only local propagation repair.

If evidence must be captured in a newly persistent heap object or requires a new lifetime/GC contract, STOP AND CONSULT and record Outcome B.

Perform an architecture audit confirming there is no:

```text
RuntimeAssociatedTypeEnvironment
projection slot recipe
associated-type VM bytecode
runtime projection search
```

### Execution sequence

- [ ] Trace callable-reference creation/invocation.
- [ ] Build smallest generic-evidence-dependent fixture.
- [ ] Determine Outcome A or B before production edits.
- [ ] If A, add regression and minimal compatible fix.
- [ ] If B, record concrete missing contract and do not implement.
- [ ] Confirm P2 core execution remains behavioral-evidence-only.

### Forbidden approaches

- no new persistent evidence-capture object without consultation;
- no associated runtime dictionary;
- no runtime projection normalization;
- no broad callable-reference redesign.

### Test changes required

Covers:

```text
CV-60 CV-61
```

### Tests to run now

Only the exact callable-reference test plus P1/P2 core generic modules if production code changed.

### Tests explicitly deferred

- full closure/reference suite unless shared mechanics changed;
- workspace.

### Acceptance

Existing callable-reference architecture is either proven compatible with generic evidence or the limitation is explicitly deferred with a concrete architecture boundary. P2 adds no runtime associated-type mechanism.

### Local STOP / CONSULT triggers

Any new persistent evidence-capture/lifetime/GC representation is required.

### Checkpoint update

Record the audit conclusion if it affects future P3/P6 work.

---

## T10 — Stabilize P2, certify focused acceptance, and write durable records

### Purpose

Run bounded stabilization gates, classify failures, map required coverage to evidence, update checkpoint state, and produce P2 walkthrough/handoff.

### Preconditions

- T1-T9 accepted or an allowed documented T9 deferral exists.
- no unresolved A/B failures in required coverage.

### Consumes

All P2 architecture and tests.

### Produces

- focused verification record;
- updated C6 checkpoint ledger/state;
- `LANG005.C6.P2-walkthrough.md`;
- `LANG005.C6.P2-handoff.md`;
- next action for C6.P3.

### Files and symbols

**Modify:**
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md`

**Create:**
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-walkthrough.md`
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-handoff.md`

Do not modify guidance unless a persistent C6 rule genuinely changed.

### Required implementation shape

Run the named gates in Section 19.

Build final CV mapping from actual live test names and outcomes.

Confirm:

```text
no P3 conditional proof
no Iterator/Iterable law
no production iterator migration
no runtime associated lookup
no duplicate evidence/normalization authority
```

### Execution sequence

- [ ] Run G1-G6 in order.
- [ ] Classify each failure A/B/C/D.
- [ ] Fix only A/B; bounded pass on C; record D.
- [ ] Build CV-01..CV-64 map with actual test names.
- [ ] Confirm no forbidden later-C6 surface was implemented.
- [ ] Confirm metadata round-trip.
- [ ] Confirm runtime boundary.
- [ ] Update checkpoint truthfully.
- [ ] Write walkthrough.
- [ ] Write P3 handoff.
- [ ] Run plan self-review.

### Forbidden approaches

- no workspace certification by reflex;
- no weakening assertions;
- no closing C6 checkpoint;
- no `CERTIFIED` status without certification;
- no starting P3 during P2 stabilization.

### Test changes required

Covers all remaining obligations:

```text
CV-01 .. CV-64
```

### Tests to run now

Use Section 19 gates.

### Tests explicitly deferred

- workspace/release certification;
- P3 conditional proof tests;
- P4/P5 migration;
- unrelated baselines.

### Acceptance

Every mandatory P2 coverage item has PASS evidence or the one explicitly allowed callable-reference architectural deferral; no A/B failure remains; P2 is complete/implemented/focused-tested; C6 remains in progress; P3 handoff is actionable.

### Local STOP / CONSULT triggers

- final acceptance requires weakening an invariant;
- a P2-owned semantic failure survives one serious correction;
- P2 can only pass by implementing P3 conditional proof;
- metadata compatibility requires an unplanned governance decision.

### Checkpoint update

At completion, if evidence supports it:

```yaml
plan: LANG005.C6.P2
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

Keep checkpoint itself:

```yaml
status: IN_PROGRESS
completion: PARTIAL
```

unless separate checkpoint acceptance authority explicitly closes C6.

---

# 19. Verification gates

## G1 — P1 substrate integrity gate

Purpose: prove P2 is not building on silent evidence-layout failure or slot-position drift.

Run:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
cargo test -p phalcom-core --test core language::generic_conformance
```

Expected:

```text
P1 generic-conformance tests PASS
checked slot-capacity regression PASS
same-trait/different-subject runtime regression PASS
owner-generic runtime regression PASS
exact-case regression PASS
environment reuse regression PASS
```

Do not broaden after PASS.

---

## G2 — Canonical generic associated semantics gate

Purpose: prove syntax, projection identity, specialization elaboration, equality canonicalization, and normalization as one semantic layer.

Run:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints
```

Expected:

```text
contextual/qualified projection PASS
specialization elaboration PASS
ambiguity/missing diagnostics PASS
kind integration PASS
canonical equality PASS
generic normalization PASS
terminal states PASS
```

---

## G3 — Projection-driven inference gate

Purpose: prove associated evidence can solve generic parameters without reverse search.

Run the P2 semantic module plus directly affected current generic inference tests.

Expected:

```text
known subject + exact associated evidence determines RHS generic
projection in ordinary type positions participates
generic-to-generic equality forwarding works
unknown subject remains underconstrained
Dynamic remains invalid evidence
```

On failure classify the stage per Section 14.4 before editing.

---

## G4 — Durable metadata/cross-module gate

Purpose: prove P2 is not source-local.

Run:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints::<metadata/cross-module filters>
cargo test -p phalcom-core --test core language::generic_associated_constraints
```

plus exact metadata validator tests.

Expected:

```text
projection stable node exports/imports
projection-bearing Equivalent round-trips
provider/consumer contract reconstructs
consumer associated-derived inference succeeds
core execution observes correct result
```

---

## G5 — Incremental/tooling gate

Purpose: prove canonical identity drives invalidation and editor behavior.

Run exact P2 tests in:

```text
semantic incremental fingerprints
semantic recomputation metrics
source index/editor
LSP associated navigation/hover
```

Expected:

```text
associated-surface change invalidates affected consumer
sugar/expanded semantic fingerprint parity holds
qualified/contextual/specialization navigation targets canonical declarations
ambiguous contextual projection has no arbitrary target
```

---

## G6 — Focused P2 stabilization gate

Purpose: prove P2 plus inherited P1/C5 regressions together.

Run:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
cargo test -p phalcom-core --test core language::generic_associated_constraints
cargo test -p phalcom-core --test core language::generic_conformance
```

Then run only exact current C5 associated-projection, metadata, incremental, and LSP filters documented in the walkthrough.

Expected:

```text
all required focused P2/P1/C5 tests PASS
or unrelated baseline failures classified D and recorded
no runtime associated lookup/proof mechanism introduced
```

Do not automatically proceed to workspace certification.

---

## 20. Final focused acceptance

At plan completion, create a walkthrough table:

| Coverage ID | Concrete test | Result | Notes |
|---|---|---|---|
| CV-01 | actual live test name | PASS | checked overflow |
| ... | ... | ... | ... |
| CV-64 | actual live test name | PASS | publication |

Rules:

1. every CV maps to a concrete test or, where appropriate, a code-structure audit plus test;
2. `CV-61` may be `DEFERRED-CONSULT` only if callable-reference support demonstrably requires a new persistent evidence-capture/lifetime architecture;
3. no other core semantic coverage item may be silently deferred;
4. equivalent source forms must be tested through semantic product equality/fingerprint, not only equal output;
5. runtime boundary assertions must be backed by source audit and absence of new runtime associated machinery;
6. final status reflects tests actually run, not tests planned.

Do not automatically run workspace certification.

---

## 21. Performance/resource evidence

P2 has structural performance requirements but does not require benchmark ceremony.

### 21.1 Generic evidence indexing

Acceptance evidence:

```text
contextual associated lookup uses an environment index or bounded subject-local lookup
not a scan of all project traits/conformances per projection
```

Record the data structure/query path in the walkthrough.

### 21.2 Equality/refinement lookup

Acceptance evidence:

```text
lexical equality/refinement view is built/reused per evidence environment or proof context
not rescanned from the entire GenericSignature for every normalization
```

### 21.3 Inference worklist

Acceptance evidence:

```text
each pending projection obligation reaches:
resolved
stable-unresolved
conflicting
ambiguous
or terminal blocked/cancelled/budget/internal
```

Repeated iterations require newly established information.

Add a convergence regression if the chosen worklist could loop.

### 21.4 Runtime conformance environment reuse

Retain T1 structural cardinality evidence:

```text
N identical invocations do not create N distinct equal RuntimeConformanceEnvironment values
```

No microbenchmark is required.

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

Update `LANG005.C6-CHECKPOINT.md` only after implementation actually begins:

```text
active plan: LANG005.C6.P2
starting revision: actual live HEAD
P1 inherited state: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

Do not mark P2 active merely because the plan file exists.

### 22.2 During plan

Update only for durable events:

- T1 checked slot-layout/transport invariant established;
- staged projection formation interface established;
- generic equality/normalization interface established;
- projection-aware inference gate passes;
- stable projection metadata contract established;
- accepted consultation/amendment;
- meaningful baseline reclassification;
- named verification gate passes.

Do not log each test command.

### 22.3 At plan completion

Record:

```text
P2 state
focused verification gates actually passed
durable interfaces/invariants
deferred failures
consultations/amendments
callable-reference audit conclusion
next action: plan/execute C6.P3
```

C6 remains in progress.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-walkthrough.md
```

Must contain:

1. exact final revision;
2. final P2 status/completion/verification;
3. architecture implemented;
4. P1 hardening performed;
5. parser/AST changes;
6. staged generic-signature formation;
7. projection/equality evidence architecture;
8. inference worklist architecture;
9. stable metadata representation;
10. cross-module behavior;
11. source-index/LSP/incremental integration;
12. callable-reference audit outcome;
13. deviations;
14. consultations/amendments;
15. tests added;
16. tests actually run;
17. CV-01..CV-64 result map;
18. tests deferred;
19. unrelated/baseline failures;
20. performance/resource evidence;
21. residual risks.

Do not claim workspace/release certification unless it occurred.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-handoff.md
```

It must give P3 an immediately usable takeover state.

### 24.1 Current state

```text
revision
P2 completion state
checkpoint state
focused gates passed
deferred baselines
```

### 24.2 Frozen inherited interfaces

At minimum:

```text
GenericConstraint::Conforms
GenericConstraint::Equivalent
GenericEvidenceEnvironment conformance/equality views
generic conformance proof API
generic equality proof/query API
TypeData::AssociatedProjection
generic projection normalization mode/context
projection-aware inference representation
stable associated projection metadata node
stable associated requirement identity
RuntimeConformanceEnvironment behavioral-only rule
```

### 24.3 P3 next objective

P3 may add:

```text
conditional conformance candidates
nested obligations
recursive proof
proof memoization
cycle handling
conditional coherence
```

It must consume P1/P2 authorities rather than adding new ones.

### 24.4 Things P3 must not redesign

```text
projection identity
TraitRef identity
GenericConstraint relation model
GenericEvidenceEnvironment ownership
normalize_type ownership
stable projection node
runtime no-proof boundary
```

### 24.5 First recommended P3 reads/commands

List actual final paths/tests discovered during P2.

---

## 25. Completion truth table

Do not conflate:

```text
plan file written
implementation started
source written
tests added
targeted tests passed
focused affected suites passed
P2 walkthrough written
P2 handoff written
P2 accepted
C6 checkpoint complete
workspace/release certified
```

Expected normal P2 completion truth:

```yaml
plan:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

checkpoint:
  status: IN_PROGRESS
  completion: PARTIAL
```

unless actual verification/authority says otherwise.

---

## 26. Plan self-review

Before implementation handoff, verify:

- [x] every requirements-analysis finding has an owning task;
- [x] P1 evidence overflow and slot-compaction hazards are addressed before P2 semantics;
- [x] owner-generic and same-trait/different-subject regressions are explicit;
- [x] syntax distinguishes ordinary trait args from associated specialization;
- [x] where-kind syntax feeds existing kind authority;
- [x] TraitRef identity and application proof are separated;
- [x] contextual projection is source-order-independent;
- [x] qualified projection still requires conformance proof;
- [x] specialization is `Conforms + Equivalent`;
- [x] `Equivalent` symmetry is canonicalized;
- [x] one lexical evidence environment owns conformance + equality/refinement;
- [x] one projection normalizer remains authoritative;
- [x] projection-aware inference is general across ordinary type positions;
- [x] reverse open-world subject search is forbidden;
- [x] stable metadata has a general projection type node;
- [x] cross-module reconstruction is mandatory;
- [x] incremental/editor behavior uses canonical associated identity;
- [x] runtime remains behavioral-evidence-only;
- [x] callable-reference uncertainty has an objective audit/deferral path;
- [x] conditional proof remains P3;
- [x] testing budget is focused;
- [x] escalation triggers are objective;
- [x] walkthrough/handoff obligations are explicit;
- [x] no placeholders remain;
- [x] tasks form a valid dependency order.

---

# Appendix A — Task dependency graph

```text
T1  P1 substrate hardening
 │
 └─────► T2 syntax
          │
          └─────► T3 staged formation + canonical propositions
                    │
                    ├─────► T4 equality/refinement + normalization
                    │         │
                    │         └─────► T5 signature/owner integration
                    │                   │
                    │                   └─────► T6 projection inference
                    │                               │
                    │                               └─────► T7 metadata/cross-module
                    │                                           │
                    │                                           └─────► T8 tooling/incremental
                    │
                    └────────────────────────────────────────────► T9 callable-ref/runtime audit

T1..T9
   └─────► T10 stabilization + records
```

T9 may begin its read-only audit earlier, but production edits should wait until T1/T5 make evidence contracts clear.

---

# Appendix B — Canonical semantic examples

## B.1 Specialization and expanded form converge

Source A:

```phalcom
where I |= Iterator<Item = T>
```

Source B:

```phalcom
where
    I |= Iterator,
    <I as Iterator>::Item == T
```

Canonical result:

```text
Conforms(I, Iterator)

Equivalent(
    AssociatedProjection(
        subject = I,
        trait_ref = Iterator,
        requirement = Iterator::Item
    ),
    T
)
```

after canonical operand orientation.

## B.2 Contextual and qualified projection converge when unambiguous

Given:

```phalcom
where I |= Iterator
```

both:

```phalcom
I::Item
<I as Iterator>::Item
```

must intern the same semantic projection identity.

## B.3 Contextual ambiguity

Given:

```phalcom
where T |= A + B
```

and both traits own distinct `type Item`, then `T::Item` is ambiguous.

Explicit alternatives remain:

```phalcom
<T as A>::Item
<T as B>::Item
```

when corresponding conformance evidence exists.

## B.4 Assumed symbolic projection

Given only:

```phalcom
where I |= Iterator
```

`I::Item` is valid even if no equality determines its value. It remains a symbolic canonical projection.

## B.5 Associated-driven inference

Given exact evidence:

```text
UserIterator |= Iterator
<UserIterator as Iterator>::Item == User
```

and generic contract:

```text
<I as Iterator>::Item == T
```

plus argument evidence:

```text
I = UserIterator
```

derive:

```text
T = User
```

Do not derive `I` from `T = User` alone.

---

# Appendix C — Required P2 failure categories

Use existing diagnostic architecture. Exact code names are flexible; semantic distinctions are not:

```text
associated projection missing evidence
associated projection ambiguous
qualified projection missing conformance
unknown associated requirement
duplicate associated specialization
invalid positional argument after specialization
trait application arity mismatch
trait application kind mismatch
generic kind fact conflict
associated specialization RHS kind mismatch
generic associated equality unsatisfied
generic associated inference underconstrained
generic associated inference conflict
recursive projection
blocked projection proof
Dynamic cannot prove conformance
stable metadata projection invalid
```

Do not collapse these to one generic `AnnotationUnresolved` diagnostic where the semantic distinction is known.

---

# Appendix D — P2 acceptance statement

P2 is accepted when the following statement is true and backed by focused tests:

> A Phalcom generic contract may name an associated type contextually or through an explicit trait qualifier, refine it through associated specialization or ordinary `==` equivalence, use that projection anywhere an ordinary type is legal, and allow exact or enclosing generic evidence to determine another type parameter during generic application. The canonical semantics are independent of source ordering/spelling, public projection-bearing contracts survive stable metadata and module boundaries, editor/incremental products use canonical trait-owned identities, and runtime execution continues to receive only already-selected behavioral evidence with no associated-type search or proof in the VM.
