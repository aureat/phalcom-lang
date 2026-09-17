---
id: LANG005.C6.P4
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
depends_on:
  - LANG005.C6.P3
follows: LANG005.C6.P3
supersedes: null
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: 2da270877ee34693357784d9e2077f63da721802
planning_revision_subject: "feat(lang): implement conditional conformance proof evidence"
requirements_analysis: LANG005.C6.P4-requirements-analysis.md
executor_profile: GPT-5.6-Luna-High-or-Extra-High
---

# LANG005.C6.P4 — Iterator / Iterable Generic Laws, Guarded Derived Conformance, Trait-Contract Entailment, and Canonical Universe Staging

> **For agentic workers:** execute this plan task-by-task. The preferred execution style is one coherent task per implementation session or fresh implementation agent, with review at every named gate. Do not skip task-local verification merely because a later gate exists. Do not run broad test suites unless this plan explicitly names them.
>
> **Required workflow:** read `AGENTS.md`, `docs/workflow/implementation-record-lifecycle-convention.md`, and this plan before editing. Follow repository BUILD / STABILIZE / CERTIFY doctrine exactly.
>
> **Required sub-skill when available:** use the repository's plan-execution / subagent-development workflow rather than improvising a second execution process.

**Goal:** Implement the general semantic machinery and canonical Universe law by which every proven `Iterator` conformance derives one coherent `Iterable` conformance, while enforcing the `Iterable` associated-type contract, preserving P3 recursive proof/evidence/runtime boundaries, and leaving production collection/`for` migration to C6.P5.

**Architecture:** P4 adds two distinct but composable semantic mechanisms. First, trait-level `GenericSignature` contracts become subject-aware so `Iterable` can require `Self::Iterator |= Iterator` and `<Self::Iterator as Iterator>::Item == Self::Item`, and those propositions are implied by an assumed or exact `Iterable` conformance. Second, source `impl<I> Iterable for I where I |= Iterator` is represented as a guarded derived-conformance rule rather than an ordinary nominal `ConformanceTarget`; the P3 `ConformanceProofContext` applies that rule, constructs complete derived evidence including behavioral and associated bindings, and lowers the finite evidence DAG through the existing runtime type/conformance environments with zero VM proof search.

**Tech Stack:** Rust workspace; `phalcom-ast`; `phalcom-semantic`; `phalcom-core`; canonical `GenericSignature`, `TraitRef`, `AssociatedTypeRequirementId`, `TypeData::AssociatedProjection`, `ConformanceProofContext`, `ConformanceIndex`, `GenericConformanceEvidence`, `RuntimeTypeEnvironment`, `RuntimeConformanceEnvironment`; source-authored Phalcom Universe.

**Spec / analysis:** `LANG005.C6.P4-requirements-analysis.md`, `docs/specs/generic-constraints.md`, `docs/specs/objects/traits.md`, `docs/specs/objects/impl.md`, `docs/specs/associated-types.md`, `docs/specs/iterator.md`.

---

# 0. Executor contract

This plan is designed specifically for a capable Luna implementation agent operating under constrained architectural authority.

The implementer:

- may adapt private helper names, imports, local decomposition, and mechanically drifted paths to the live repository;
- must preserve every `FIXED` architecture rule and `INV-*` invariant in this plan;
- must not reinterpret P4 as permission to redesign traits, associated types, generic inference, conformance coherence, runtime type reification, or production iteration;
- must work from tests that discriminate the current task rather than running broad suites reflexively;
- must preserve unrelated user edits and unrelated staged/untracked files;
- must classify unrelated failures `A/B/C/D` under the repository baseline policy;
- must STOP AND CONSULT when any escalation trigger fires;
- must keep `LANG005.C6-CHECKPOINT.md` current only at durable events;
- must produce `LANG005.C6.P4-walkthrough.md` and `LANG005.C6.P4-handoff.md` at completion.

Testing is evidence gathering, not a ritual.

A passing narrow test answers one question. It is not a command to run every wider suite.

A failed broad test outside this plan is not automatically P4 work.

---

# 1. Goal

After P4:

```text
proof: I |= Iterator
        ↓
general language rule
        ↓
proof: I |= Iterable
        ↓
derived associated bindings:
    <I as Iterable>::Item
        == <I as Iterator>::Item

    <I as Iterable>::Iterator
        == I
        ↓
derived behavioral witness:
    Iterable.iterator
        returns self
```

and every independently established `Iterable` conformance must satisfy:

```text
<Self as Iterable>::Iterator |= Iterator

<<Self as Iterable>::Iterator as Iterator>::Item
    ==
<Self as Iterable>::Item
```

The standard-library/Universe declaration must exercise these general mechanisms.

There must be no semantic or runtime branch equivalent to:

```rust
if trait_ref == ITERABLE && receiver_conforms_to_iterator {
    // synthesize special result
}
```

---

# 2. Checkpoint acceptance objective

`LANG005.C6` owns generic conformance assumptions/evidence, associated refinement/equality, recursive conditional proof, the Iterator/Iterable generic law, and later production iteration migration.

P4 is **one of several plans** in C6.

P4 is not checkpoint closure.

P4 is the semantic-law and Universe-staging plan between:

```text
P3:
    conditional conformances
    recursive proof
    nested evidence
    runtime derived evidence transport

and

P5:
    production List/Set/Map/Tuple/Bytes/Range migration
    stateful iterator objects
    for semantic typing
    for compiler lowering
    lazy iterator migration
    cursor/runtime-superclass retirement
```

P4 must finish with P5 able to consume an already-ratified, already-executable generic Iterator⇒Iterable theorem rather than designing that theorem during collection migration.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 2da270877ee34693357784d9e2077f63da721802
subject: feat(lang): implement conditional conformance proof evidence
predecessor: LANG005.C6.P3
```

The planning audit confirmed no newer main commit at plan preparation time.

Verified live facts:

1. `LANG005.C6-CHECKPOINT.md` front matter already marks:
   ```text
   active_plan: LANG005.C6.P4
   latest_completed_plan: LANG005.C6.P3
   next_plan: LANG005.C6.P4
   ```
   but body/current-action prose must be checked for stale P3 wording.

2. `phalcom-semantic/src/generic_evidence.rs` currently owns:
   ```rust
   GenericConformanceKey
   AssumedConformanceEvidence
   NestedGenericEvidence
   DerivedConformanceEvidence
   GenericConformanceEvidence
   ProofMemoState::{Visiting, Completed}
   GenericConformanceProof
   GenericEquivalenceProof
   GenericEvidenceLayout
   GenericEvidenceEnvironment
   ConformanceProofContext
   prove_conformance_with_context(...)
   query_generic_conformance_proof(...)
   ```

3. The live P3 interfaces differ from stale walkthrough wording:
   ```text
   live:
       ProofMemoState::Visiting
       ConformanceProofContext.memo
       DerivedConformanceEvidence

   stale record language:
       PendingCycle
       proof_cache
       DerivedGenericEvidence / ExactSourceTemplate
   ```

4. `ConformanceProofContext` currently carries:
   ```text
   lexical_evidence
   conformance_index
   witness_plans
   trait_surfaces
   declarations
   hierarchy
   store
   memo
   stack
   budget
   cancellation
   ```

5. P3 runtime evidence is already lowered through:
   ```text
   GenericCallConformanceSource::Derived
       ↓
   RuntimeConformanceSlotRecipe::DerivedConditional
       ↓
   RuntimeConformanceSlot
       contains method + type_environment + conformance_environment
   ```

6. Current exact/source conformance ownership remains centered in:
   ```text
   phalcom-semantic/src/impls.rs
   ```
   and `ConformanceTarget` remains nominal/exact-case:
   ```rust
   Declaration(DeclarationId)
   ExactEnumCase(VariantId)
   ```

7. `TraitDef` already contains:
   ```text
   generic_parameters
   where_clause
   members
   ```
   but P4 must VERIFY-FIRST whether the trait where-clause resolver can see `Self` and the trait's own associated requirement identities.

8. `TraitHeader` currently contains:
   ```rust
   declaration
   generic_signature
   source
   ```

9. `TraitSurface` separately owns:
   ```text
   behavioral requirements
   associated type declarations
   ```
   with `AssociatedTypeRequirementId` as the canonical associated identity.

10. `TraitRef::form_identity(...)` verifies:
    ```text
    declaration kind
    arity
    kinds
    canonical arguments
    ```
    while legacy `TraitRef::form(...)` still contains ordinary generic-constraint validation; conformance constraints are explicitly deferred to the generic proof layer.

11. P3 tests currently live at:
    ```text
    phalcom-semantic/tests/semantic/capabilities/conditional_conformance.rs
    phalcom-core/tests/core/language/conditional_conformance.rs
    ```

12. P1 predecessor runtime tests with three ignored/scheduled cases live at:
    ```text
    phalcom-core/tests/core/language/generic_conformance.rs
    ```

13. Semantic integration tests are registered through:
    ```text
    phalcom-semantic/tests/semantic.rs
    phalcom-semantic/tests/semantic/mod.rs
    phalcom-semantic/tests/semantic/capabilities/mod.rs
    phalcom-semantic/tests/semantic/incremental/mod.rs
    ```

14. Core source-language tests are registered through:
    ```text
    phalcom-core/tests/core/language/mod.rs
    ```

15. Current production Universe still exposes the legacy cursor hierarchy in:
    ```text
    phalcom-core/core/universe/src/collections/iterable.ph
    ```
    including:
    ```phalcom
    @native
    class Iterable<T> is Object { ... }

    class Iterator is Iterable { ... }

    class SourceIterator is Iterator { ... }
    class MapIterator is Iterator { ... }
    class FilterIterator is Iterator { ... }
    ...
    ```

16. Runtime bootstrap/invariants still know the legacy `Iterable` class:
    ```text
    phalcom-core/src/vm/bootstrap.rs
    phalcom-core/src/universe/invariants.rs
    phalcom-native-meta/src/universe.rs
    ```

17. The normative `docs/specs/iterator.md` already states the target behavior:
    ```text
    every Iterator is Iterable
    Iterable.Item = Iterator.Item
    Iterable.Iterator = subject
    iterator returns self
    ```
    but intentionally leaves the generic evidence mechanism unspecified.

18. `docs/specs/objects/traits.md` says conformance is explicit unless another specification defines derived/synthesized conformance; this is the opening P4 uses.

19. `docs/specs/objects/impl.md` currently defines nominal/exact targets. P4 must amend it narrowly for guarded generic-target conformance rules rather than redefining ordinary nominal targets.

20. Phalcom has no source-level free functions. All executable examples in this plan are methods/getters or impl witnesses. Do not introduce free-function fixtures.

> Re-read every named live path before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

# 4. Required reads before implementation

Read, in this order, before first edit:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md

docs/implementation/LANG005/LANG005.C6/
    LANG005.C6.P3-conditional-conformances-recursive-proof-evidence-cycles-and-coherence-plan.md
    LANG005.C6.P3-walkthrough.md
    LANG005.C6.P3-handoff.md

LANG005.C6.P4-requirements-analysis.md

docs/specs/README.md
docs/specs/generic-constraints.md
docs/specs/objects/traits.md
docs/specs/objects/impl.md
docs/specs/associated-types.md
docs/specs/iterator.md

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md

phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/mod.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/db/{product,key,query,fingerprint}.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/trait_dispatch.rs

phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/impl_syntax.rs
phalcom-ast/tests/trait_syntax.rs

phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/vm/dispatch.rs

phalcom-core/core/universe/src/collections/iterable.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/universe/invariants.rs
phalcom-native-meta/src/universe.rs

phalcom-semantic/tests/semantic/capabilities/
    generic_conformance.rs
    generic_associated_constraints.rs
    conditional_conformance.rs

phalcom-core/tests/core/language/
    generic_conformance.rs
    conditional_conformance.rs
```

Do not broadly rescan the repository unless one of these files demonstrates that a plan premise is mechanically stale.

---

# 5. Normative authority

Authority order for P4:

```text
1. accepted/current docs/specs normative language rules
2. user-ratified project decisions
3. LANG005.C6 checkpoint durable invariants
4. predecessor implemented live architecture
5. this implementation plan
6. historical plans/proposals
7. stale tests/comments
```

## 5.1 Normative behavior

P4 relies on these language rules:

```text
Phalcom has no free functions.

Conformance:
    nominal
    evidence-based
    not structural coincidence

GenericConstraint:
    Conforms
    Equivalent
    Subtype

where:
    canonical GenericSignature contract
    source order non-semantic

Associated projections:
    canonical trait-owned requirement identity
    generic/qualified projection supported
    types::normalize_type is the single normalizer

Iterator:
    type Item
    next -> Option<Self::Item>
    iterator { self }

Iterable:
    type Item
    type Iterator
    iterator -> Self::Iterator

Iterable law:
    Self::Iterator |= Iterator
    <Self::Iterator as Iterator>::Item == Self::Item

Iterator theorem:
    Iterator ⇒ Iterable
    Iterable.Item = Iterator.Item
    Iterable.Iterator = Self
    Iterable.iterator = self

Cycle:
    recursion alone is not proof

Coherence:
    one exact subject + exact TraitRef has one applicable proof
    no source-order winner
    no most-specific conditional conformance rule

Runtime:
    consumes semantic evidence
    does not perform conformance/projection proof search
```

## 5.2 Current implementation

The live P1–P3 architecture is implementation truth where verified against source.

Do not treat stale walkthrough symbol names as live APIs.

## 5.3 Future/non-normative proposals not activated by P4

P4 does not activate:

```text
supertraits
trait inheritance
trait objects/existentials
negative conformance
specialization
generic associated types
higher-kinded associated declarations
open-world reverse inference
runtime conformance solver
production stateful collection migration
```

---

# 6. Takeover state

## 6.1 Stable interface map

| Concept | Current symbol/path | Owner | P4 invariant |
|---|---|---|---|
| generic proposition | `GenericConstraint` in `phalcom-semantic/src/types/parameter.rs` | semantic type system | remain sole canonical constraint form |
| direct conformance key | `GenericConformanceKey` | `generic_evidence.rs` | subject + exact `TraitRef` |
| lexical assumed evidence | `GenericEvidenceEnvironment` | `generic_evidence.rs` | evidence authority for generic bodies |
| direct/recursive proof | `prove_conformance_with_context` | `generic_evidence.rs` | extend, do not duplicate |
| proof session | `ConformanceProofContext` | `generic_evidence.rs` | memo/cycle/budget/cancellation authority |
| cycle state | `ProofMemoState::Visiting` | `generic_evidence.rs` | cycle remains non-proof |
| exact source evidence | `ConformanceEvidence` | `impls.rs` | nominal/exact source proof only |
| conditional derived evidence | `DerivedConformanceEvidence` | `generic_evidence.rs` | preserve existing meaning while generalizing origin if needed |
| nominal source conformance | `ConformanceContribution` | `impls.rs` | keep C4/P3 semantics intact |
| target identity | `ConformanceTarget` | `impls.rs` | remain nominal/exact-case |
| source witness plan | `ConformanceWitnessPlan` | `impls.rs` | existing nominal witness/default authority |
| trait identity | `TraitRef` | `traits.rs` | trait declaration + ordinary args only |
| trait generic contract | `TraitHeader.generic_signature` | `traits.rs` | one canonical contract |
| trait behavior/associated surface | `TraitSurface` | `traits.rs` | no second associated registry |
| associated declaration identity | `AssociatedTypeRequirementId` | `traits.rs` | unchanged |
| projection representation | `TypeData::AssociatedProjection` | type store | unchanged |
| normalization | `types::normalize_type` | semantic types | sole recursive projection normalizer |
| runtime type environment | existing registry/id | core typing | separate from conformance env |
| runtime conformance env | existing registry/id | core typing | finite preselected evidence |
| runtime derived recipe | `RuntimeConformanceSlotRecipe::DerivedConditional` | core typing | reuse/generalize mechanically if information content fits |
| trait runtime dispatch | `InvokeTraitRequirement` path | compiler/VM | no proof search |
| Universe cursor implementation | `collections/iterable.ph` | core Universe | transitional P4 staging only |
| legacy bootstrap Iterable class | `bootstrap.rs` / native meta | runtime | preserve until safe P5 handoff |

---

# 7. Architecture

P4 implements two logically independent layers.

## 7.1 Layer A — trait-contract entailment

`Iterable` must own a trait GenericSignature equivalent to:

```phalcom
trait Iterable
    where
        Self::Iterator |= Iterator,
        <Self::Iterator as Iterator>::Item == Self::Item
{
    type Item
    type Iterator

    iterator -> Self::Iterator
}
```

Semantic formation:

```text
TraitDef
    ↓
trait identity + type parameters
    ↓
associated declaration headers
    Item
    Iterator
    ↓
form trait GenericSignature with subject-aware Self
    ↓
publish TraitHeader / TraitSurface
```

Semantic proof:

```text
proof S |= Iterable
    ↓
instantiate Iterable GenericSignature:
    Self → S
    ↓
prove:
    <S as Iterable>::Iterator |= Iterator
    <<S as Iterable>::Iterator as Iterator>::Item
        ==
    <S as Iterable>::Item
```

Generic assumed evidence:

```text
lexical T |= Iterable
    ↓
trait-contract entailment
    ↓
lexical/provable:
    T::Iterator |= Iterator
    T::Iterator::Item == T::Item
```

The contract is evidence-derived semantic knowledge, not a synthetic source rewrite performed in every consumer.

---

## 7.2 Layer B — guarded derived conformance

Source form:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
{
    type Item = <I as Iterator>::Item
    type Iterator = I

    iterator {
        self
    }
}
```

This source is **not** an ordinary C4 nominal `ConformanceTarget`.

It lowers to a separate semantic rule product conceptually:

```rust
pub struct DerivedConformanceRule {
    pub source_impl: ImplId,
    pub subject_parameter: TypeParameterId,
    pub subject_template: TypeTerm,
    pub consequent_trait_ref: TraitRef,
    pub generic_signature: GenericSignature,
    pub requirement_views: BTreeMap<TraitRequirementId, InstantiatedTraitRequirement>,
    pub requirements: BTreeMap<TraitRequirementId, DerivedRequirementSelectionTemplate>,
    pub associated_types: BTreeMap<AssociatedTypeRequirementId, DerivedAssociatedTypeBindingTemplate>,
    pub source_module: ModuleId,
    pub source: SemanticSourceSpan,
    pub fingerprint: ProductFingerprint,
}
```

Exact names and private decomposition are mechanically flexible.

Required information content is FIXED.

Rule index:

```text
DerivedConformanceRuleIndex
    key:
        consequent trait declaration
    value:
        deterministic rule candidates
```

P3 proof integration:

```text
prove_conformance_with_context(S |= TraitRef)
    1. lexical assumed evidence
    2. existing exact/conditional nominal source evidence
    3. relevant derived-rule candidates
```

For a rule match:

```text
bind I = S
instantiate GenericSignature
prove every nested obligation
materialize associated bindings
materialize behavior selections
validate consequent trait contract
construct derived evidence
```

The same `ConformanceProofContext` owns all nested proving and memoization.

---

## 7.3 Derived evidence

Current P3:

```rust
pub struct DerivedConformanceEvidence {
    pub exact_source: Arc<ConformanceEvidence>,
    pub impl_bindings: TypeSubstitution,
    pub nested: Box<[NestedGenericEvidence]>,
}
```

A guarded rule cannot fabricate `exact_source`.

P4 should generalize the **derived origin**, not weaken `ConformanceEvidence`.

Preferred conceptual shape:

```rust
pub enum DerivedConformanceOrigin {
    ConditionalSource {
        exact_source: Arc<ConformanceEvidence>,
    },

    GuardedRule {
        rule: Arc<DerivedConformanceRule>,
        exact_subject: TypeId,
        exact_trait_ref: TraitRef,
    },
}

pub struct DerivedConformanceEvidence {
    pub origin: DerivedConformanceOrigin,
    pub impl_bindings: TypeSubstitution,
    pub requirement_views: BTreeMap<TraitRequirementId, InstantiatedTraitRequirement>,
    pub requirements: BTreeMap<TraitRequirementId, RequirementSelectionTemplateOrDerived>,
    pub associated_types: BTreeMap<AssociatedTypeRequirementId, ExactAssociatedTypeBindingOrDerived>,
    pub nested: Box<[NestedGenericEvidence]>,
}
```

A different representation is acceptable if it preserves all of these semantics.

Do not manufacture:

```text
fake declaration target
fake source ConformanceEvidence
fake ImplId meaning
```

---

## 7.4 Evidence consumer abstraction

P4 should avoid scattering variant-specific matching across projection/lowering.

Introduce or extend semantic operations conceptually equivalent to:

```rust
fn exact_subject(evidence: &GenericConformanceEvidence) -> Option<TypeId>;

fn exact_trait_ref(evidence: &GenericConformanceEvidence) -> Option<&TraitRef>;

fn associated_binding(
    evidence: &GenericConformanceEvidence,
    requirement: &AssociatedTypeRequirementId,
) -> AssociatedBindingLookup;

fn requirement_selection(
    evidence: &GenericConformanceEvidence,
    requirement: &TraitRequirementId,
) -> RequirementSelectionLookup;
```

Names are flexible.

The architectural requirement is that consumers ask the evidence abstraction rather than assume:

```text
Derived => exact_source.associated_types
```

---

## 7.5 Coherence

No ranking.

No direct-wins rule.

For a concrete goal:

```text
S + TraitRef
```

there may be at most one successful proof across:

```text
ordinary exact/conditional nominal conformance
guarded derived rules
```

If an exact nominal `Iterable` conformance and the Iterator-derived rule both prove:

```text
S |= Iterable
```

the result is a coherence conflict.

Reason: direct-wins would permit an Iterator to violate the normative Iterable Item/Iterator theorem.

Initial rule-rule coherence is conservative:

```text
if two derived rules can produce the same consequent exact relation
and applicability cannot be proven disjoint by already-existing canonical identity rules:
    reject / coherence conflict
```

Do not build a condition-domain disjointness solver.

---

## 7.6 Ownership

Ordinary C4 ownership remains:

```text
source owns trait
or
source owns nominal target
```

A guarded generic-target rule has no nominal target owner.

P4 rule:

```text
source module must own consequent trait
```

This is sufficient for the Universe-owned `Iterator ⇒ Iterable` theorem and prevents third-party global trait implication declarations.

---

## 7.7 Runtime

Semantic layer decides:

```text
rule applicability
nested proof
coherence
associated bindings
behavioral selections
trait-contract satisfaction
```

Core/compiler receives finite evidence recipes:

```text
callable
type bindings
nested conformance slots
```

VM receives:

```text
RuntimeTypeEnvironmentId
RuntimeConformanceEnvironmentId
preselected callable / requirement slot
```

VM does not receive:

```text
DerivedConformanceRule
GenericConstraint
ConformanceProofContext
AssociatedProjection to solve
```

---

# 8. Ownership boundaries

## 8.1 P4 owns

- P3 closure preflight relevant to P4.
- subject-aware trait contract formation and proof;
- implied constraints from assumed trait conformance;
- guarded generic-target conformance rule syntax/semantic classification;
- derived-rule indexing;
- rule ownership/coherence;
- recursive rule application under P3 proof context;
- generalized derived evidence sufficient for behavior + associated bindings;
- derived associated projection normalization;
- runtime evidence lowering for guarded rules;
- cross-module publication and incremental invalidation;
- focused source-index/LSP correctness for new source forms;
- canonical Universe `Iterator`/`Iterable` trait declarations and theorem;
- compatibility staging required to install the canonical traits while preserving P5 work.

## 8.2 P4 does not own

- production `for` semantic checker migration;
- production `for` bytecode/lowering migration;
- List/Set/Map/Tuple/Bytes/Range production conformances;
- Map entry iteration behavior switch;
- RangeIterator production implementation;
- stateful lazy adapter migration;
- removal of `iterate(_)`;
- removal of `iteratorValue(_)`;
- removal of the runtime legacy iterable superclass unless transition staging proves a purely mechanical internal rename is necessary;
- supertraits;
- specialization;
- negative constraints;
- trait objects;
- GATs;
- full C6 certification.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| generic proposition | `GenericConstraint` / `GenericSignature` | checker/prover/fingerprints | separate “iterator law constraint” enum |
| lexical assumed conformance | `GenericEvidenceEnvironment` | body checking/call proving | per-consumer assumed trait set |
| direct recursive proof | `ConformanceProofContext` | projections/calls/rules | iterator-specific proof engine |
| exact source conformance | `ConformanceEvidence` | projection/dispatch/lowering | fake evidence for generic rule |
| guarded derived rule | new semantic rule product | proof/coherence/metadata | compiler-only rule table |
| associated identity | `AssociatedTypeRequirementId` | projections/evidence/tooling | name-based rule binding |
| trait contract | `TraitHeader.generic_signature` + subject instantiation | conformance completeness/generic assumptions | extra TraitContract struct with duplicate constraints |
| projection normalization | `types::normalize_type` | checker/proof/export | derived-rule normalizer |
| rule coherence | semantic conformance proof/index layer | diagnostics/snapshot | runtime ambiguity resolution |
| executable evidence recipe | compiler/core typing | VM | VM source-rule scan |
| runtime type env | core typing registry | frames/dispatch | merged all-purpose env |
| runtime conformance env | core typing registry | frames/trait dispatch | per-object conformance array |
| Iterator/Iterable law | real Universe source + normative spec | semantic/compiler/runtime | hard-coded trait ID branch |

---

# 9. Global invariants

`INV-01 — No free functions.`  
Phalcom executable source is method/member based. P4 examples/tests must not introduce top-level declared functions.

`INV-02 — Two-law separation.`  
The `Iterable` trait contract and the `Iterator ⇒ Iterable` derived theorem are separate semantic mechanisms.

`INV-03 — One GenericSignature authority.`  
Trait contracts and derived-rule conditions use existing `GenericSignature`.

`INV-04 — TraitRef remains identity-only.`  
`TraitRef` is trait declaration + ordinary arguments; subject `Self` is not embedded in it.

`INV-05 — Subject-aware contract proof.`  
A trait's constraints are instantiated with an explicit conformance subject.

`INV-06 — Associated identity remains C5 identity.`  
`AssociatedTypeRequirementId` remains trait-owned and is never reconstructed by name at proof time.

`INV-07 — One normalizer.`  
`types::normalize_type` remains the only associated projection normalization authority.

`INV-08 — No fake nominal target.`  
A bare generic rule subject does not become `ConformanceTarget::Declaration` or another pretend exact target.

`INV-09 — ConformanceTarget remains nominal/exact-case.`  
C4 exact target semantics are preserved.

`INV-10 — Derived rule is proof rule, not exact source conformance.`  
Rule provenance and applied exact proof are distinct.

`INV-11 — Guarded rule only.`  
Initial generic-target rule requires a meaningful generic constraint contract.

`INV-12 — Consequent-trait ownership.`  
A generic-target derived rule is authorized only from the consequent trait's owning module.

`INV-13 — Nominal conformance remains nominal.`  
Matching `next`/`iterator` members never imply Iterator or Iterable.

`INV-14 — No rule ranking.`  
Source order, import order, “more specific”, or explicit-vs-derived precedence never selects a winner.

`INV-15 — One exact proof.`  
One exact subject + exact TraitRef may have at most one successful coherent conformance proof.

`INV-16 — P3 conditional coherence preserved.`  
Where-clause differences do not legalize overlapping nominal conformance heads.

`INV-17 — Cycle is not proof.`  
`ProofMemoState::Visiting` re-entry never becomes automatic success.

`INV-18 — Same proof session.`  
Derived-rule antecedents, trait-contract consequences, and projection normalization share the active P3 proof context.

`INV-19 — Rich terminals survive.`  
Missing, ambiguous, incomplete, coherence conflict, unknown, blocked, dynamic, cancelled, budget-exceeded, and internal failure remain distinct where already modeled.

`INV-20 — No reverse open-world inference.`  
A desired associated output does not trigger search for a subject satisfying a trait.

`INV-21 — Derived evidence is complete evidence.`  
Guarded rule evidence contains the behavioral, associated, substitution, and nested evidence required by consumers.

`INV-22 — P3 derived conditional evidence remains valid.`  
Existing conditional conformance behavior is not redefined to fit the guarded-rule case.

`INV-23 — No fake exact_source.`  
Guarded rule evidence never fabricates an `Arc<ConformanceEvidence>`.

`INV-24 — Projection through derived evidence is semantic.`  
`Iterable::Item` / `Iterable::Iterator` normalize before runtime.

`INV-25 — Runtime is proof-free.`  
VM executes preselected evidence only.

`INV-26 — Runtime type/conformance environments stay separate.`

`INV-27 — Registry reuse remains structural.`  
Equal runtime environments intern/reuse; repeated calls do not allocate permanent environment records linearly with call count.

`INV-28 — Evidence lifetime remains rooted by existing ownership.`  
No unrooted persistent side table.

`INV-29 — Canonical Iterator theorem is general.`  
The semantic solver contains no special branch on the Iterator/Iterable names/IDs.

`INV-30 — Universe exercises the language.`  
The standard law is expressed in real source declarations.

`INV-31 — P4 does not perform P5.`  
Legacy production iteration remains operational until P5.

`INV-32 — Iterator identity.`  
For an Iterator subject, the derived Iterable `iterator` getter returns the same traversal object.

`INV-33 — Associated theorem identity.`  
Derived `Iterable::Iterator` is the exact subject; derived `Iterable::Item` is the subject's Iterator Item projection.

`INV-34 — Iterable contract universal.`  
Every valid Iterable conformance, explicit or derived, satisfies Iterator conformance and Item equality.

`INV-35 — Generic assumptions entail trait contracts.`  
`T |= Iterable` is sufficient to use the constraints declared by Iterable without redundant source constraints.

`INV-36 — Stable semantic fingerprints exclude source order/ranges.`

`INV-37 — Incremental equals cold semantics.`

`INV-38 — Cross-module proof uses published semantic products.`  
No downstream source reparsing.

`INV-39 — Tooling consumes canonical targets.`  
Source index/LSP do not implement a second solver.

`INV-40 — Scope discipline.`  
A failure outside the P4 acceptance surface is recorded/classified rather than opportunistically repaired.

---

# 10. Non-goals

P4 must not implement:

```text
trait inheritance / supertraits
trait composition as a separate mechanism
trait objects / existential packaging
negative conformances
negative bounds
specialization
"more-specific wins"
derived-rule priority
condition-domain SAT/disjointness
general overlap theorem prover
generic associated types
higher-kinded associated declarations
higher-kinded derived rule subjects
structural protocol conformance
automatic Dynamic proof
runtime trait rule search
runtime associated type lookup
reverse associated-output inference
production collection iteration migration
production for-loop iterator+next lowering
production Map entry migration
production RangeIterator
production lazy iterator rewrite
cursor API retirement
full checkpoint/release certification
```

---

# 11. Expected impact map

## 11.1 Expected source areas

### Syntax / AST

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/impl_syntax.rs
phalcom-ast/tests/trait_syntax.rs
```

Only if VERIFY-FIRST shows the existing AST/parser cannot represent:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
```

or cannot form trait where clauses referencing `Self`/associated projections.

Do not add syntax merely because the semantic layer rejects the current source.

### Semantic trait contracts

```text
phalcom-semantic/src/traits.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/mod.rs
```

### Conformance/rule products

Preferred new focused file:

```text
phalcom-semantic/src/derived_conformance.rs
```

or equivalent focused module if repository conventions make another location clearer.

Existing owners:

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/trait_dispatch.rs
```

Do not make `impls.rs` absorb every new rule-specific helper if a focused module keeps ownership clearer.

### Projection / proof integration

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/types/projection.rs
```

### DB / stable products / incremental

```text
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/snapshot.rs
```

### Source index / LSP-visible semantic occurrence

```text
phalcom-semantic/src/source_index/builder.rs
```

`phalcom-lsp` only if current canonical source-index integration needs an explicit presentation update.

### Core lowering/runtime

```text
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/vm/dispatch.rs
```

Expected to be a generalization/reuse of P3 mechanics, not a new execution subsystem.

### Universe staging

```text
phalcom-core/core/universe/src/collections/iterable.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/universe/invariants.rs
phalcom-native-meta/src/universe.rs
```

Only the files actually required by the selected transition strategy may be touched.

### Specifications / state

```text
docs/specs/objects/traits.md
docs/specs/objects/impl.md
docs/specs/generic-constraints.md
docs/specs/iterator.md

docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P3-handoff.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P3-walkthrough.md

docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-handoff.md
```

---

## 11.2 Expected tests

Create focused P4 semantic module:

```text
phalcom-semantic/tests/semantic/capabilities/derived_conformance.rs
```

Register in:

```text
phalcom-semantic/tests/semantic/capabilities/mod.rs
```

Create focused P4 core/runtime module:

```text
phalcom-core/tests/core/language/derived_conformance.rs
```

Register in:

```text
phalcom-core/tests/core/language/mod.rs
```

Create incremental module when rule product publication is implemented:

```text
phalcom-semantic/tests/semantic/incremental/derived_conformance.rs
```

Register in:

```text
phalcom-semantic/tests/semantic/incremental/mod.rs
```

Extend predecessor tests only where they are the canonical owner of the regression:

```text
phalcom-core/tests/core/language/generic_conformance.rs
phalcom-semantic/tests/semantic/capabilities/conditional_conformance.rs
```

Avoid one giant P4 test file that mixes source parser tests, semantic proof tests, runtime execution, and incremental metrics.

---

## 11.3 Unexpected-touch rule

Mechanical adjacent touch is allowed.

Examples:

```text
module registration
public re-export
diagnostic enum wiring
stable product enum variant
test helper reuse
formatter import cleanup
```

STOP AND CONSULT before entering a materially different subsystem because the architecture requires it.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

1. P4's two-law separation.
2. `ConformanceTarget` exact/nominal meaning.
3. `TraitRef` identity meaning.
4. `AssociatedTypeRequirementId` identity.
5. `GenericSignature` as constraint authority.
6. P3 `ConformanceProofContext` as recursive proof authority.
7. cycle-as-non-proof semantics.
8. no rule ranking / no “direct wins”.
9. consequent-trait ownership for generic-target rule.
10. general derived rule mechanism instead of Iterator special case.
11. no fake `ConformanceEvidence`.
12. `types::normalize_type` as sole projection normalizer.
13. separate runtime type/conformance environments.
14. zero VM proof search.
15. P4/P5 boundary.
16. method-only Phalcom executable examples.
17. Iterator/Iterable associated laws.
18. source-order-independent canonical identity.
19. one exact relation => one coherent proof.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- `DerivedConformanceRule` exact Rust name;
- `DerivedConformanceRuleIndex` exact name;
- file split between `derived_conformance.rs` and `impls.rs`;
- private helper names;
- private evidence accessor names;
- whether derived origin is an enum field or separate evidence variant;
- whether runtime recipe variant is renamed from `DerivedConditional` to a more general name, **only if** behavior/information remains identical and predecessor tests remain valid;
- exact diagnostic wording if semantic distinction is preserved;
- test helper names;
- private map/index representation (`BTreeMap`, deterministic vector, etc.) if canonical ordering is preserved;
- exact transition-internal legacy class names, if Universe staging requires them.

## 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| no commits after planning baseline | `git log -n 5 --oneline` | re-ground paths; STOP only for architectural drift |
| checkout clean enough to edit safely | `git status --short` | preserve unrelated work; never reset |
| P3 ignored tests still ignored | `generic_conformance.rs` | run/disposition actual current form |
| trait where-clause parser accepts `Self` projection syntax | AST/parser + parser test | add parser support only if syntax missing |
| trait header resolution currently has enough associated identity context | `traits.rs`, annotation formation | add staged associated-header pass; STOP if identity must change |
| bare impl generic target parses | `impl_syntax.rs` / parser | add syntax support if absent |
| semantic target resolver rejects bare parameter only because target is non-nominal | `impls.rs` | route to derived-rule resolver; STOP if grammar fundamentally differs |
| `DerivedConformanceEvidence` consumers assume `exact_source` | search live tree | generalize consumer abstraction before rule use |
| P3 runtime recipe can carry rule witness + nested evidence | core typing/compile | reuse/generalize; STOP if new runtime solver/lifetime object required |
| runtime env registry exposes a count/len or test-observable intern behavior | core typing tests | add test-only/publicly harmless metric or structural identity assertion |
| callable reference already preserves evidence env | callable/core code + predecessor tests | Outcome A regression or Outcome B documented deferral |
| legacy public `Iterable` / `Iterator` classes can be transitionally internalized without P5 | Universe/bootstrap audit | choose internal transition namespace or STOP if simultaneous P5 required |
| source index already walks trait where-clause | `source_index/builder.rs` | extend canonical occurrence mapping |
| cross-module semantic product can represent rule | DB/product/interface path | add stable rule product; STOP if snapshot-local IDs would leak |

---

# 13. Global STOP / CONSULT triggers

STOP editing and build a consultation packet if any occurs:

1. a generic target must be represented as a fake nominal declaration;
2. `ConformanceTarget` would need to lose nominal/exact semantics;
3. `TraitRef` would need to include subject `Self`;
4. associated declaration identity would need to be name-based or duplicated;
5. a second constraint representation is needed;
6. a second projection normalizer is needed;
7. a second recursive proof engine is needed;
8. correct derived-rule proving requires coinductive cycle success;
9. exact evidence must be fabricated for a guarded rule;
10. explicit-vs-derived coherence appears to require precedence;
11. overlapping derived rules require specialization or SAT/disjointness;
12. rule ownership cannot be enforced through consequent trait ownership;
13. derived proof requires runtime source-rule scanning;
14. VM must inspect `GenericConstraint`;
15. VM must normalize `AssociatedProjection`;
16. runtime type and conformance environments must be merged;
17. per-object generic evidence arrays become necessary;
18. callable-reference support requires a new persistent heap evidence object;
19. GC/rooting correctness becomes nonlocal/uncertain;
20. stable metadata would need `TypeId`, infer IDs, or runtime env IDs;
21. P4 cannot install canonical traits without simultaneously rewriting production `for`;
22. P4 cannot install canonical traits without migrating all collections;
23. a plan task would legalize conditional nominal overlap that P3 rejects;
24. reverse/open-world inference becomes necessary;
25. `Dynamic` would have to count as static proof;
26. a P3/P2/P1 invariant must be weakened to pass a P4 test;
27. the same semantic failure survives one serious evidence-driven correction;
28. live repository evidence contradicts the normative iterator law;
29. a new subsystem becomes architecturally necessary rather than mechanically adjacent;
30. the implementation becomes materially larger than the task decomposition below.

Triggered consultation is not self-waivable.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Maximum roughly three coherent correction cycles while each cycle makes progress.

A cycle is:

```text
inspect exact failure
identify concrete local cause
make one coherent correction
rerun smallest useful discriminator
```

Examples:

```text
missing import
renamed private helper
borrow checker cleanup
module registration
incorrect test filter
simple enum exhaustiveness
```

## 14.2 Semantic failures

Before a nontrivial semantic edit record locally:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure.

If essentially the same semantic failure remains: STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative fixes.

Consult immediately.

## 14.4 Do not thrash

Do not rerun an unchanged failure if nothing relevant changed.

Do not switch to a broad suite because the narrow test is difficult.

---

# 15. Testing surface analysis

The requirements analysis defines `P4-CV-01..80`.

This plan groups them into execution surfaces.

## 15.1 Trait contract / formation

| Coverage | Invariants | Required evidence |
|---|---|---|
| CV-01 | 03–07 | trait where-clause with `Self` forms |
| CV-02 | 05–07 | `Self::Iterator` forms against own associated identity |
| CV-03 | 05–07 | nested qualified Item equality forms |
| CV-04 | 03,36 | trait-contract source order canonical |
| CV-05 | 06,19 | invalid associated projection diagnoses |
| CV-06 | 03 | no associated-declaration-bound syntax required |
| CV-07 | 34 | valid explicit Iterable contract succeeds |
| CV-08 | 34 | non-Iterator associated iterator rejected |
| CV-09 | 34 | mismatched Item rejected |
| CV-10 | 35 | assumed Iterable implies iterator conformance |
| CV-11 | 35 | assumed Iterable implies Item equality |
| CV-12 | 35,38 | imported assumption implication works |
| CV-13 | 13,19 | Dynamic not static proof |
| CV-14 | 17 | recursive trait contract terminates |

## 15.2 Derived rule source / coherence

| Coverage | Invariants | Required evidence |
|---|---|---|
| CV-15 | 08–12 | guarded generic target rule forms |
| CV-16 | 11 | unguarded generic-target rule rejected |
| CV-17 | 12 | external consequent-trait rule rejected |
| CV-18 | 36 | binder spelling irrelevant |
| CV-19 | 36 | where order fingerprint stable |
| CV-20 | 06,21 | rule associated binding uses canonical requirement |
| CV-21 | 21 | witness remains rule/conformance-owned |
| CV-47 | 14–16 | explicit + derived exact relation conflict |
| CV-48 | 14–16 | derived rule overlap conflict |
| CV-49 | 14,36 | declaration order irrelevant |
| CV-50 | 14–16,38 | imported rule conflict detected |
| CV-51 | 16 | P3 conditional same-head rejection green |
| CV-52 | 09,16 | exact enum case identity stays exact |

## 15.3 Recursive rule proof / associated evidence

| Coverage | Invariants | Required evidence |
|---|---|---|
| CV-22 | 18,29 | exact Iterator derives Iterable |
| CV-23 | 13 | non-Iterator does not |
| CV-24 | 18,35 | assumed Iterator derives Iterable |
| CV-25 | 18,22 | conditional Iterator feeds derived Iterable |
| CV-26 | 18 | nested derived chain works |
| CV-27 | 17 | self-cycle blocks |
| CV-28 | 17 | mutual cycle blocks |
| CV-29 | 19 | budget state preserved |
| CV-30 | 19 | cancellation preserved |
| CV-31 | 19 | blocked/unknown provenance preserved |
| CV-32 | 19,21 | failed rule has no partial evidence |
| CV-33 | 24,33 | Iterable::Item => Iterator::Item |
| CV-34 | 24,33 | Iterable::Iterator => exact subject |
| CV-35 | 07,24 | nested structural projection normalizes |
| CV-36 | 24,35 | symbolic generic projection remains canonical |
| CV-37 | 24,38 | cross-module derived projection |
| CV-38 | 07,24 | equality solver consumes derived binding |
| CV-39 | 25 | no runtime associated lookup |

## 15.4 Runtime evidence

| Coverage | Invariants | Required evidence |
|---|---|---|
| CV-40 | 21,32 | derived iterator getter returns identical object |
| CV-41 | 21,29,35 | generic callee requiring Iterable accepts Iterator evidence |
| CV-42 | 21,26 | derived witness receives type env |
| CV-43 | 21,26 | derived witness receives nested conformance env |
| CV-44 | 27 | repeated calls reuse env |
| CV-45 | 28 | post-GC invocation uses retained evidence |
| CV-46 | 25 | VM has no rule/proof branch |

## 15.5 Incremental / publication

| Coverage | Invariants | Required evidence |
|---|---|---|
| CV-53 | 38 | provider rule proves consumer |
| CV-54 | 25,38 | provider rule runtime execution |
| CV-55 | 37 | antecedent removal invalidates |
| CV-56 | 37 | re-add restores |
| CV-57 | 37 | rule condition edit invalidates |
| CV-58 | 36–37 | canonical reorder no-op |
| CV-59 | 34,37 | trait contract edit invalidates |
| CV-60 | 37 | cold/incremental parity |
| CV-61 | 18 | derived index avoids global scan |
| CV-62 | 17–18 | Iterator proof reused during contract validation |
| CV-63 | 17–18 | unique-goal scaling evidence |
| CV-64 | 27 | runtime env registry bounded |
| CV-65 | 27 | repeated calls do not grow linearly |

## 15.6 Universe / P5 boundary

| Coverage | Invariants | Required evidence |
|---|---|---|
| CV-66 | 30 | canonical Universe Iterator loads |
| CV-67 | 30,34 | canonical Universe Iterable loads |
| CV-68 | 29–33 | Universe theorem executes |
| CV-69 | 31 | legacy for smoke unchanged |
| CV-70 | 31 | legacy collection cursor smoke unchanged |
| CV-71 | 31 | no collection migration |
| CV-72 | 31 | no cursor selector deletion |

## 15.7 P3 closure carryover

| Coverage | Required evidence |
|---|---|
| CV-73 | owner generic runtime test disposition |
| CV-74 | owner+method evidence runtime test disposition |
| CV-75 | exact enum-case generic test disposition |
| CV-76 | callable-ref Outcome A/B |
| CV-77 | memo/resource discriminator |
| CV-78 | runtime env reuse discriminator |
| CV-79 | rooting discriminator |
| CV-80 | records synchronized |

---

# 16. Verification execution budget

## 16.1 Modes

### BUILD MODE

Default.

Run only the smallest discriminator needed by the current task.

### STABILIZE MODE

At `G0..G7`.

Run focused predecessor and directly affected suites.

### CERTIFY MODE

Not part of ordinary P4.

Do not run workspace/release certification unless the user explicitly asks or a real architectural incident makes it necessary.

---

## 16.2 Verification ladder

```text
L0 exact new regression / direct reproducer
L1 new focused feature module
L2 owning semantic/core subsystem
L3 predecessor adjacent cross-layer integration
L4 broad crate
L5 workspace/release
```

P4 BUILD should live mostly at L0/L1.

---

## 16.3 Mandatory during BUILD

Task-specific commands below.

Common environment:

```sh
RUSTFLAGS='' RUSTC_WRAPPER=''
```

Run Cargo commands serially.

Confirm filters select nonzero tests.

---

## 16.4 Mandatory at named gates

Only commands named under `G0..G7`.

---

## 16.5 Only if evidence demands

```sh
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-core --test core
cargo check -p phalcom-lsp
```

Use only when a focused failure indicates broad module interaction.

---

## 16.6 Explicitly DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace
cargo test -p phalcom-core --test language-corpus   # whole corpus
release builds
full LSP integration
all benchmarks
```

Reason: P4 has narrow semantic/runtime acceptance and known unrelated baselines. Broad execution would waste compute and obscure causality.

---

# 17. Baseline / unrelated failure policy

Classification:

```text
A — definitely caused by current P4 patch
B — probably caused by current P4 patch
C — unclear
D — clearly unrelated / inherited baseline
```

Rules:

- A/B: fix within P4.
- C: one bounded classification pass; if nonblocking and still unclear, record in checkpoint and continue.
- D: record and continue immediately.
- Do not modify expected behavior or skip tests to make a broad gate green.
- Do not spend adviser budget on D unless D prevents proving P4 acceptance.

Inherited items to preserve unless reclassified by live evidence:

```text
legacy cursor / Range / Option.match call-entry issues from C5
known unrelated Bool capability baselines
```

P4-specific predecessor tests are different: CV-73..75 must be dispositioned because they touch the same evidence architecture.

---

# 18. Tasks

---

## T0 — Re-ground P4 and close the P3 predecessor ambiguity

### Purpose

Start from actual repository state, repair stale continuation records, and determine whether P3's scheduled runtime gaps are green, active P4 blockers, or documented deferrals before P4 builds another evidence layer.

### Preconditions

- no production P4 edit made;
- current checkout available;
- requirements analysis read.

### Consumes

```text
P3 implementation
P3 walkthrough
P3 handoff
P1 scheduled runtime tests
checkpoint/guidance
```

### Produces

```text
truthful P4 starting revision
P3 closure classifications
correct live interface names in continuation records
active P4 checkpoint state
explicit callable-reference Outcome A/B
resource-test gap disposition
```

### Files and symbols

**Read:**

```text
AGENTS.md
docs/workflow/*
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P3-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P3-handoff.md

phalcom-semantic/src/generic_evidence.rs
phalcom-core/tests/core/language/generic_conformance.rs
phalcom-core/tests/core/language/conditional_conformance.rs
```

**Modify only after evidence:**

```text
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md
LANG005.C6.P3-walkthrough.md
LANG005.C6.P3-handoff.md
```

Do not rewrite predecessor history; correct only materially false continuation names/state and append bounded closure facts.

### Required implementation shape

1. Record:
   ```sh
   git status --short
   git log -n 5 --oneline
   ```

2. Verify live interfaces:
   ```text
   ProofMemoState::Visiting
   ConformanceProofContext.memo
   DerivedConformanceEvidence
   ```

3. List the three scheduled/ignored tests:
   ```sh
   cargo test -p phalcom-core --test core language::generic_conformance -- --list
   ```

4. Run individually:
   ```text
   owner_generic_evidence_executes_after_receiver_specialization
   owner_and_method_local_evidence_execute_together
   exact_enum_case_conformance_executes
   ```

5. Classify each:
   - PASS when explicitly included/ignored override => predecessor closure green;
   - fail caused by same P1/P3 evidence transport => A/B and must be fixed before T3;
   - unrelated baseline => D and record;
   - genuinely new persistent-capture architecture => C/consult depending on impact.

6. Inspect callable-reference environment handling.
   Record:
   ```text
   Outcome A:
       existing bound/detached callable path preserves/reconstructs both environments
       -> P4 adds regression later

   Outcome B:
       correct behavior needs new persistent evidence-capture architecture
       -> explicitly defer and do not pretend P4 proves it
   ```

7. Inspect current P3 memo/resource tests.
   Record whether they expose:
   ```text
   proof unique-goal count / completed memo reuse
   runtime conformance registry cardinality
   post-GC deferred invocation
   ```
   If not, mark them required P4 discriminators for T4/T6.

### Forbidden approaches

- no feature implementation;
- no broad P3 re-audit;
- no fixing unrelated legacy Range/cursor failures;
- no rewriting P3 architecture;
- no changing ignored status simply to claim green.

### Test changes required

No new production feature tests yet.

If a predecessor ignored test passes when explicitly executed and is clearly intended to become active, unignore it only if its fixture is valid and it belongs to predecessor acceptance. Otherwise record the evidence and leave activation for the owning task.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core language::generic_conformance -- --list

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core owner_generic_evidence_executes_after_receiver_specialization -- --ignored --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core owner_and_method_local_evidence_execute_together -- --ignored --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core exact_enum_case_conformance_executes -- --ignored --nocapture
```

If test names have mechanically drifted, use the listed full names and run the actual equivalents.

### Tests explicitly deferred

All P4 feature tests.

### Acceptance

T0 completes when:

```text
start revision recorded
three predecessor runtime cases classified
callable-reference outcome recorded
memo/env/rooting coverage gaps explicitly identified
checkpoint/guidance say P4 is active
P3 continuation records no longer tell P4 to use stale live symbol names
```

### Local STOP / CONSULT triggers

- any predecessor failure proves P3 exact/derived evidence architecture materially unsound;
- owner/method evidence requires per-object evidence arrays;
- exact enum case cannot be fixed without widening target identity;
- callable-ref Outcome B is required for ordinary P4 runtime correctness rather than optional reference UX.

### Checkpoint update

Yes.

Record T0 classification and any baseline IDs.

### Commit boundary

Documentation-only T0 changes may be committed separately:

```sh
git add docs/implementation/LANG005/LANG005.C6/
git commit -m "docs(lang005): ground C6 P4 takeover"
```

Do not include unrelated source edits.

---

## G0 — P4 takeover gate

Purpose: prove the execution baseline is known before adding new semantics.

Expected:

```text
P3 stable interfaces verified against live source
predecessor scheduled tests classified
no unresolved A/B predecessor failure
P4 active records truthful
```

Do not broaden after PASS.

---

## T1 — Make trait generic contracts subject-aware and capable of referring to own associated declarations

### Purpose

Allow `Iterable` to own its two associated laws in the ordinary trait `GenericSignature` without new associated-declaration-bound syntax.

### Preconditions

- G0 passed;
- no unresolved P3 A/B blocker.

### Consumes

```text
TraitDef.where_clause
TraitHeader.generic_signature
TraitSurface.associated_types
AssociatedTypeRequirementId
TypeFormationSite
resolve_generic_signature
GenericConstraint
TypeData::AssociatedProjection
```

### Produces

A single canonical trait contract capable of representing:

```text
Self::Iterator |= Iterator

<Self::Iterator as Iterator>::Item
    ==
Self::Item
```

and a subject-instantiation helper/API later tasks use.

### Files and symbols

**Read:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/trait_syntax.rs

phalcom-semantic/src/traits.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/db/fingerprint.rs
```

**Modify expected:**

```text
phalcom-semantic/src/traits.rs
phalcom-semantic/src/types/annotation.rs
```

Possible mechanical support:

```text
phalcom-ast/src/parser.rs
phalcom-ast/tests/trait_syntax.rs
```

only if syntax currently cannot parse/represent the contract.

**Tests:**

Create/register:

```text
phalcom-semantic/tests/semantic/capabilities/derived_conformance.rs
phalcom-semantic/tests/semantic/capabilities/mod.rs
```

### Required implementation shape

#### A. Pre-scan associated headers

Trait contract formation must know the trait-owned associated identities before resolving its where-clause.

Do not duplicate identity.

Preferred staging:

```text
trait declaration identity
    ↓
allocate generic parameter identities/kinds
    ↓
scan associated declarations in source order
    create AssociatedTypeRequirementId(owner, stable_index)
    create minimal associated header context
    ↓
resolve trait where-clause GenericSignature
    formation site:
        owner = trait declaration
        Self = abstract subject
        associated context = same canonical IDs
    ↓
build full TraitSurface behavior
    reusing same associated IDs
```

If current `TraitSurface` construction already pre-scans associated declarations, factor/reuse that pass rather than adding another index algorithm.

#### B. Represent abstract Self correctly

Do not add Self to `TraitRef`.

Use the existing `TypeTerm::SelfType` / `TypeFormationSite` machinery.

The stored trait GenericSignature may contain subject-relative terms/projections.

#### C. Subject instantiation API

Add a helper conceptually:

```rust
pub fn instantiate_trait_generic_contract(
    store: &mut TypeStore,
    trait_surface_or_header: &...,
    subject: TypeIdOrTypeTerm,
    trait_ref: &TraitRef,
) -> InstantiatedTraitContract
```

or equivalent.

It must:

```text
bind trait ordinary parameters
bind Self to subject
preserve canonical associated requirement IDs
return canonical instantiated GenericConstraint values
```

Do not prove them yet in this task.

#### D. Fingerprinting

Trait contract fingerprint must be based on canonical semantic content, not source range/order noise.

### Concrete test source

Use method-based Phalcom fixtures only.

Positive formation fixture:

```phalcom
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}
```

If `Stepper` needs a behavioral requirement to be a useful trait fixture, add a getter; do not introduce free functions.

### Test changes required

Implement CV-01..06.

Suggested test names:

```text
trait_contract_where_self_forms
trait_contract_where_self_associated_projection_forms
trait_contract_qualified_nested_projection_forms
trait_contract_constraint_order_is_canonical
trait_contract_unknown_own_associated_projection_diagnoses
trait_contract_does_not_require_associated_declaration_bound_syntax
```

### Tests to run now

For each red/green cycle:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
derived_conformance::trait_contract_where_self_forms -- --nocapture
```

then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture
```

Do not run core runtime tests yet.

### Tests explicitly deferred

```text
P3 full semantic suite
core runtime
incremental
Universe
```

### Acceptance

- trait where constraints can canonically mention `Self` and own associated projections;
- same associated declaration IDs are used in trait surface and contract;
- no duplicate associated registry exists;
- `TraitRef` unchanged;
- all T1 formation tests pass.

### Local STOP / CONSULT triggers

- associated identity must be allocated twice and reconciled by name;
- `Self` must be inserted into `TraitRef`;
- parser grammar would need a materially new projection syntax beyond current spec;
- trait contract cannot be represented by `GenericSignature`;
- contract storage would need a second canonical constraints field.

### Checkpoint update

Record only if T1 establishes a new durable staging invariant.

### Commit

```sh
git add \
  phalcom-semantic/src/traits.rs \
  phalcom-semantic/src/types/annotation.rs \
  phalcom-semantic/tests/semantic/capabilities/derived_conformance.rs \
  phalcom-semantic/tests/semantic/capabilities/mod.rs \
  phalcom-ast/src/parser.rs \
  phalcom-ast/tests/trait_syntax.rs

git commit -m "feat(semantic): form subject-aware trait contracts"
```

Stage only files actually changed.

---

## T2 — Enforce trait contracts and derive implied constraints from trait evidence

### Purpose

Make a proven or assumed conformance to a trait carry the trait's own declared constraints as semantic consequences.

### Preconditions

- T1 complete;
- trait contract instantiation exists.

### Consumes

```text
GenericEvidenceEnvironment
ConformanceProofContext
GenericConformanceEvidence
TraitHeader/TraitSurface generic signature
instantiate_trait_generic_contract
prove_constraint / relation proving
```

### Produces

1. Exact `S |= Trait` cannot be considered valid if the instantiated trait contract fails.
2. Generic `T |= Trait` can entail the trait contract inside the body.
3. `TraitRef` identity remains independent from proof.

### Files and symbols

**Modify expected:**

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/checker/context.rs
```

Possible projection integration:

```text
phalcom-semantic/src/types/projection.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/derived_conformance.rs
```

### Required implementation shape

#### A. Exact/source conformance completeness

When proving exact `S |= Iterable`:

```text
resolve candidate/source evidence
    ↓
instantiate Iterable trait contract with S
    ↓
prove contract constraints using:
    current exact evidence
    current lexical evidence
    current P3 proof context
    associated bindings of this candidate/evidence
    ↓
only then publish Proven(...)
```

Be careful with self-reference:

```text
proving S |= Iterable
    may require normalizing <S as Iterable>::Iterator
```

The contract must be able to use the candidate evidence being constructed without requesting a final globally published result and deadlocking itself.

Reuse the C5 source-plan / staged associated-binding pattern:

```text
candidate-local associated binding plan
    is available to contract normalization
```

Do not solve by inserting provisional global success.

#### B. Assumed evidence entailment

For lexical:

```text
T |= Iterable
```

make the contract constraints available to the active evidence environment/prover.

Preferred design:

```text
GenericEvidenceEnvironment
    stores explicit canonical assumptions
    +
    exposes lazy trait-contract consequences through proof
```

rather than eagerly expanding every trait transitive closure on environment creation.

A query for:

```text
T::Iterator |= Iterator
```

may:

```text
inspect lexical T |= Iterable
instantiate Iterable contract
prove/query matching consequence
memoize within current proof session
```

Exact local mechanics are flexible.

#### C. Avoid infinite implication closure

Do not recursively pre-expand trait contracts to a fixed point on environment construction.

Use proof-session memoization.

#### D. Terminal propagation

Contract proof failure must distinguish:

```text
missing
ambiguous
blocked
dynamic
cancelled
budget
internal
```

when underlying P3 APIs already distinguish them.

### Concrete semantic fixtures

Valid explicit:

```phalcom
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}

class IntStepper {}

impl Stepper for IntStepper {
    type Item = Int
}

class IntStream {}

impl Stream for IntStream {
    type Item = Int
    type Iterator = IntStepper

    iterator {
        IntStepper.new()
    }
}
```

Invalid iterator conformance:

```phalcom
class NotStepper {}

impl Stream for BadStream {
    type Item = Int
    type Iterator = NotStepper
    ...
}
```

Invalid Item equality:

```text
Stream.Item = Int
Stream.Iterator = StringStepper
Stepper(StringStepper).Item = String
```

Generic assumed body must be a method:

```phalcom
class StreamUse {
    inspect<S>(_ source: S)
        where S |= Stream
    {
        // operations/tests must force proof that S::Iterator |= Stepper
    }
}
```

### Test changes required

Implement CV-07..14.

Suggested names:

```text
explicit_trait_contract_validates_successfully
trait_contract_rejects_nonconforming_associated_iterator
trait_contract_rejects_associated_item_mismatch
assumed_trait_evidence_entails_associated_conformance
assumed_trait_evidence_entails_associated_equality
imported_assumption_entails_trait_contract
dynamic_does_not_satisfy_trait_contract
trait_contract_recursive_implication_terminates_without_success
```

### Tests to run now

Narrow first:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
derived_conformance::explicit_trait_contract_validates_successfully -- --nocapture
```

Then module:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture
```

### Tests explicitly deferred

Core runtime and Universe.

### Acceptance

- explicit Iterable-like contracts are validated;
- assumed conformance entails own trait constraints;
- cycles do not become proof;
- no `TraitRef` semantic expansion;
- no eager global transitive closure.

### Local STOP / CONSULT triggers

- source conformance contract validation can only work by publishing provisional final evidence globally;
- proof requires coinductive success;
- trait consequences require changing `GenericConstraint` semantics;
- contract implications cannot be scoped to the active lexical/proof environment.

### Checkpoint update

Record durable “trait conformance entails trait GenericSignature under subject instantiation” invariant.

### Commit

```sh
git add phalcom-semantic/src phalcom-semantic/tests/semantic/capabilities
git commit -m "feat(semantic): enforce and entail trait contracts"
```

Stage narrowly.

---

## G1 — Trait-contract gate

Purpose: establish Law A before implementing Law B.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::generic_associated_constraints -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::generic_conformance -- --nocapture
```

Expected:

```text
new trait-contract tests PASS
P2 generic associated constraint regressions PASS
P1 direct generic conformance regressions PASS
```

Do not run core runtime after G1.

---

## T3 — Introduce guarded generic-target derived-conformance source products and coherence

### Purpose

Represent source `impl<I> Trait for I where ...` as a first-class guarded proof rule without mutating ordinary nominal `ConformanceTarget`.

### Preconditions

- G1 passed;
- no active P3 blocker.

### Consumes

```text
ImplDef
impl-owned GenericSignature
ImplId
ConformanceContribution / target resolver
conformance ownership/coherence rules
TraitRef
source module ownership
```

### Produces

```text
DerivedConformanceRule
DerivedConformanceRuleIndex
stable rule fingerprint
source diagnostic classification
rule ownership/coherence queries
```

### Files and symbols

**Read/modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/impl_syntax.rs

phalcom-semantic/src/impls.rs
phalcom-semantic/src/checker/mod.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/lib.rs
```

**Preferred create:**

```text
phalcom-semantic/src/derived_conformance.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/derived_conformance.rs
```

### Required implementation shape

#### A. Parser / AST

First test current parser with:

```phalcom
trait Printable {}

impl<T> Printable for T
    where T |= Printable
{
}
```

This fixture is semantic nonsense as a self-cycle but is useful for syntax if needed.

If parser already accepts the target type name `T`, do not change parser.

If target syntax is rejected syntactically despite being a valid `TypeAnnotationExpr`, minimally allow the already-bound impl generic parameter target.

No free-function syntax.

#### B. Semantic target classification

Existing ordinary path:

```text
resolve conformance target
    -> nominal declaration / exact enum case
    -> ConformanceContribution
```

New branch must occur only when target resolves exactly to an impl-owned type parameter.

Conceptually:

```rust
enum ResolvedConformanceDeclaration {
    Nominal(ResolvedConformanceHead),
    GuardedRule(ResolvedDerivedConformanceRuleHead),
}
```

or an equivalent split.

Do not add:

```rust
ConformanceTarget::TypeParameter(...)
```

unless an adviser explicitly amends this plan.

#### C. Guard requirement

Reject:

```phalcom
impl<T> Printable for T {}
```

for the initial feature.

Accept only when the impl has a meaningful canonical GenericSignature.

At minimum:

```text
nonempty constraints
subject parameter actually participates in applicability
```

A rule with only unrelated `U |= Something` while target is `T` should not qualify as a safely guarded initial form without explicit design justification.

Preferred initial restriction:

```text
at least one canonical constraint transitively mentions the subject parameter
```

Do not build a full dependency theorem; simple canonical occurrence is sufficient for P4.

#### D. Ownership

For a generic-target rule:

```text
source module == consequent trait declaration module
```

No target-owner alternative.

#### E. Rule product

Retain:

```text
source ImplId
source module
subject parameter/template
consequent TraitRef template
GenericSignature
source range
associated binding source syntax/plan
behavior source members
```

Build behavior/associated plans using the same canonical trait requirements as ordinary conformances.

If practical, factor reusable conformance-body planning helpers from `impls.rs`.

Do not copy witness compatibility logic.

#### F. Rule indexing

Create deterministic index by consequent trait declaration.

Do not scan all workspace rules for each proof.

#### G. Coherence

At publication / semantic composition:

```text
derived rule vs derived rule
    potentially same consequent relation
    -> conflict unless identity proves non-overlap through already-existing exact head distinctions

derived rule vs ordinary nominal conformance
    conflict is checked when an exact proof actually has both successful candidates
```

Do not attempt to enumerate all future types at publication time.

No rule ranking.

### Required source diagnostics

Add distinct codes/messages as appropriate for:

```text
unguarded generic-target conformance
unauthorized generic-target derived conformance
overlapping derived conformance rules
```

Reuse an existing general conformance-coherence diagnostic only if it preserves the distinction and testability.

### Test changes required

Implement CV-15..21 and source-level pieces of CV-47..52.

Suggested test names:

```text
guarded_generic_target_conformance_forms_rule
unguarded_generic_target_conformance_is_rejected
generic_target_rule_requires_consequent_trait_ownership
generic_target_rule_parameter_renaming_is_semantically_equal
generic_target_rule_constraint_order_is_fingerprint_stable
generic_target_rule_associated_binding_uses_trait_owned_requirement
generic_target_rule_witness_does_not_enter_inherent_surface
overlapping_generic_target_rules_are_rejected
generic_target_rule_order_does_not_select_winner
```

Keep explicit-vs-derived exact conflict acceptance test red until T4 if proof application is needed to reveal it.

### Tests to run now

Start with parser only if needed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-ast impl_syntax
```

Then focused semantic:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
derived_conformance::guarded_generic_target_conformance_forms_rule -- --nocapture
```

then module.

### Tests explicitly deferred

Runtime, Universe, broad incremental.

### Acceptance

- guarded target parameter source has first-class semantic rule product;
- ordinary nominal `ConformanceTarget` unchanged;
- unguarded universal conformance rejected;
- rule ownership enforced;
- rule fingerprint canonical;
- rule witness/associated identities use existing trait products;
- no ranking introduced.

### Local STOP / CONSULT triggers

- parser requires introducing a new target-pattern language rather than allowing an existing type parameter;
- target parameter cannot be distinguished from nominal target without changing global type identity;
- rule body planning requires duplicating witness/associated algorithms;
- ownership needs an orphan model broader than consequent trait ownership;
- rule-rule coherence requires solving arbitrary conditions.

### Checkpoint update

Record guarded rule identity/ownership once implemented.

### Commit

```sh
git add \
  phalcom-ast \
  phalcom-semantic/src \
  phalcom-semantic/tests/semantic/capabilities/derived_conformance.rs
git commit -m "feat(semantic): add guarded derived conformance rules"
```

---

## T4 — Apply guarded rules in the recursive P3 proof engine and prove coherence

### Purpose

Make `prove_conformance_with_context` consume derived rules as ordinary proof candidates, preserving P3 memoization, terminal states, and coherence.

### Preconditions

- T3 rule index exists;
- G1 trait contract works.

### Consumes

```text
ConformanceProofContext
GenericConformanceKey
GenericConformanceProof
ProofMemoState
NestedGenericEvidence
DerivedConformanceRuleIndex
GenericSignature constraint proving
```

### Produces

```text
exact and assumed derived-rule proof
nested rule proof
cycle-safe rule proof
explicit-vs-derived conflict
resource/memo evidence
```

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/derived_conformance.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/checker/context.rs
```

Potential context field:

```text
derived_conformance_rules: Option<&DerivedConformanceRuleIndex>
```

or an equivalent canonical snapshot view.

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/derived_conformance.rs
```

### Required implementation shape

#### A. Context authority

Add the rule index to `ConformanceProofContext`.

Do not create a separate rule proof context.

#### B. Candidate order is not semantic precedence

Implementation may evaluate candidate categories in a deterministic order, but successful candidates are collected/coherence-checked.

Conceptually:

```text
candidate successes = []

if lexical assumption proves goal:
    add assumed proof

if nominal source path proves goal:
    add exact/conditional proof

for relevant derived rules:
    if rule head matches:
        prove antecedents recursively
        if successful:
            add derived-rule proof

classify:
    0 successes -> preserve strongest failure / Missing
    1 success -> Proven
    >1 coherent-distinct successes -> Ambiguous/CoherenceConflict
```

Preserve the existing exact semantics for lexical assumptions if P1 intentionally gives lexical evidence authority; do not silently redefine assumption-vs-source ambiguity. VERIFY-FIRST and preserve established behavior.

#### C. Rule matching

For P4 initial form:

```text
subject template = impl-owned type parameter
bind parameter directly to goal subject
instantiate consequent TraitRef
exact-match goal TraitRef
```

No reverse inference.

#### D. Antecedent proof

For each canonical GenericSignature constraint:

```text
Conforms:
    prove_conformance_with_context

Equivalent:
    existing recursive normalization/equivalence path

Subtype:
    existing bounded relation path
```

Collect `NestedGenericEvidence`.

#### E. Cycle handling

On re-entering an active key:

```text
ProofMemoState::Visiting
    ->
GenericConformanceProof::Blocked(
    BlockReason::RecursiveFixpoint
)
```

No provisional success.

#### F. Memo instrumentation

Add a test-observable proof-session statistic only if needed, conceptually:

```rust
pub struct ProofMetrics {
    pub goals_started: usize,
    pub memo_hits: usize,
    pub completed: usize,
}
```

This may be test-only or lightweight internal telemetry.

The required discriminator:

```text
path count > unique canonical goals
memo_hits > 0
goals_started remains bounded
```

Do not add production logging.

#### G. Coherence

When exact nominal and derived rule both prove the same goal:

```text
CoherenceConflict
```

Do not select the direct one.

### Test changes required

Implement CV-22..32, CV-47..52, CV-61..63.

Use generic method fixtures, not free functions.

Example generic forwarding fixture:

```phalcom
class Use {
    needsIterable<T>(_ value: T)
        where T |= Iterable
    {
        value.iterator
    }

    throughIterator<T>(_ value: T)
        where T |= Iterator
    {
        self.needsIterable(value)
    }
}
```

This should become legal through the derived rule.

Cycle fixtures:

```text
rule A: T |= B => T |= A
rule B: T |= A => T |= B
no base evidence
=> blocked/missing, never Proven
```

Use actual valid trait names/source syntax.

### Tests to run now

Exact red/green:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
derived_conformance::exact_iterator_like_evidence_derives_iterable_like_evidence -- --nocapture
```

Module:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture
```

P3 regression:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::conditional_conformance -- --nocapture
```

### Tests explicitly deferred

Core runtime and Universe.

### Acceptance

- rule candidates are indexed and recursively proven by P3 context;
- exact, assumed, conditional, and nested evidence can feed rules;
- cycles remain non-proof;
- multiple successful proof sources conflict;
- memoization has a real reuse discriminator;
- no global rule scan on each goal.

### Local STOP / CONSULT triggers

- existing assumption semantics make “collect all candidate success” architecturally incompatible;
- explicit-vs-derived conflict cannot be represented without changing public proof result semantics materially;
- correct rule application requires reverse inference;
- cycle tests require coinductive success;
- candidate index cannot be snapshot/incremental product without global mutable state.

### Checkpoint update

Record rule proof integration and coherence once G2 passes.

### Commit

```sh
git add phalcom-semantic/src phalcom-semantic/tests/semantic/capabilities
git commit -m "feat(semantic): prove guarded derived conformances recursively"
```

---

## G2 — Guarded rule proof/coherence gate

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::conditional_conformance -- --nocapture
```

Expected:

```text
derived proof/cycle/coherence cases PASS
all P3 conditional conformance tests PASS
memo reuse discriminator demonstrates bounded unique-goal work
```

Do not broaden.

---

## T5 — Generalize derived evidence for behavioral and associated products

### Purpose

Allow a guarded rule to produce complete semantic evidence rather than a boolean implication.

### Preconditions

- G2 passes;
- rule application produces a successful proof candidate.

### Consumes

```text
DerivedConformanceEvidence
GenericConformanceEvidence
ConformanceEvidence
RequirementSelectionTemplate
ExactAssociatedTypeBinding
TypeSubstitution
AssociatedTypeRequirementId
TraitRequirementId
types::normalize_type
```

### Produces

```text
derived evidence origin distinction
guarded-rule associated bindings
guarded-rule behavior selections
generic evidence accessors
projection normalization through rule evidence
```

### Files and symbols

**Modify expected:**

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/derived_conformance.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/checker/analysis.rs
```

### Required implementation shape

#### A. Generalize origin

Preserve existing P3 conditional evidence.

Preferred:

```rust
pub enum DerivedConformanceOrigin {
    ConditionalSource {
        exact_source: Arc<ConformanceEvidence>,
    },
    GuardedRule {
        rule: Arc<DerivedConformanceRule>,
        exact_subject: TypeId,
        exact_trait_ref: TraitRef,
    },
}
```

Migrate existing construction mechanically.

#### B. Materialize rule bindings

For rule:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
{
    type Item = <I as Iterator>::Item
    type Iterator = I
    iterator { self }
}
```

after `I = ExactIter`:

```text
associated:
    Iterable.Item
        = normalize(<ExactIter as Iterator>::Item)
    Iterable.Iterator
        = ExactIter

behavior:
    Iterable.iterator
        = rule-owned witness callable

nested:
    evidence for ExactIter |= Iterator
```

#### C. Symbolic/assumed subject

For abstract generic `T |= Iterator`, derived Iterable evidence may retain symbolic projection:

```text
Iterable.Item
    = <T as Iterator>::Item
```

Do not force exact TypeId when the semantic mode is legitimately generic.

#### D. Evidence consumer helpers

Centralize lookup.

Projection normalization must ask generic evidence for the binding.

Trait dispatch must ask generic evidence for selected requirement.

Do not pattern-match down to `exact_source` in consumers.

#### E. Trait contract validation

The derived Iterator⇒Iterable evidence must itself satisfy the Iterable trait contract.

This should naturally reuse the original nested Iterator proof through memoization.

#### F. Failure atomicity

If:

```text
antecedent succeeds
associated binding normalization blocks/fails
trait contract fails
witness selection fails
```

do not publish partial derived evidence.

### Test changes required

Implement CV-33..39 and strengthen CV-62.

Suggested tests:

```text
derived_associated_item_normalizes_through_antecedent_evidence
derived_associated_iterator_is_exact_subject
derived_projection_normalizes_inside_nested_type
generic_derived_associated_projection_remains_symbolic
derived_equality_solver_consumes_rule_binding
derived_evidence_does_not_require_runtime_projection_lookup
derived_iterator_rule_satisfies_iterable_contract_using_memoized_iterator_proof
failed_derived_binding_does_not_publish_partial_evidence
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
derived_conformance::derived_associated_item_normalizes_through_antecedent_evidence -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::generic_associated_constraints -- --nocapture
```

### Tests explicitly deferred

Core runtime, cross-module incremental, Universe.

### Acceptance

- guarded derived evidence carries complete associated/behavioral meaning;
- existing P3 conditional evidence remains green;
- projection normalizer consumes generic evidence without new normalizer;
- rule law satisfies trait contract;
- failed evidence construction is atomic.

### Local STOP / CONSULT triggers

- correct projection lookup requires runtime evidence;
- `ConformanceEvidence` must be weakened/redefined;
- P3 conditional derived evidence loses exact source provenance;
- symbolic generic projection cannot be represented by existing associated projection type;
- behavioral selection would need to copy rule witness into target inherent surface.

### Checkpoint update

Record derived evidence origin and consumer abstraction after G3.

### Commit

```sh
git add phalcom-semantic/src phalcom-semantic/tests/semantic/capabilities
git commit -m "feat(semantic): materialize guarded derived conformance evidence"
```

---

## G3 — Derived evidence / associated-law gate

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::generic_associated_constraints -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::conditional_conformance -- --nocapture
```

Expected all pass.

---

## T6 — Lower guarded derived evidence through existing runtime environments

### Purpose

Execute rule-owned witnesses through the existing P3 type/conformance environment transport with no runtime proof logic.

### Preconditions

- G3 passes;
- semantic evidence contains finite executable requirement selections.

### Consumes

```text
GenericCallConformanceSource::Derived
RuntimeConformanceSlotRecipe
RuntimeConformanceSlot
RuntimeTypeEnvironment
RuntimeConformanceEnvironment
compiler module lowering
VM trait requirement dispatch
```

### Produces

```text
runtime execution of derived Iterable.iterator
type environment materialization
nested Iterator evidence environment materialization
environment reuse evidence
GC/lifetime evidence
```

### Files and symbols

**Modify expected:**

```text
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/vm/dispatch.rs
```

Potentially semantic compile projection:

```text
phalcom-semantic/src/checker/analysis.rs
```

**Create/register tests:**

```text
phalcom-core/tests/core/language/derived_conformance.rs
phalcom-core/tests/core/language/mod.rs
```

### Required implementation shape

#### A. Reuse recipe architecture

If current:

```rust
RuntimeConformanceSlotRecipe::DerivedConditional { ... }
```

already carries:

```text
selected callable
type bindings
nested slot recipes
```

reuse it for guarded rules.

If the variant is semantically tied to an exact source template, generalize mechanically to something like:

```rust
RuntimeConformanceSlotRecipe::Derived {
    ...
}
```

and preserve all predecessor behavior.

Do not add:

```text
DerivedIterator
BlanketIterable
RuleLookup
```

runtime variants.

#### B. Type environment

For exact subject `MyIter`:

```text
I -> MyIter
```

materializes in the call environment.

#### C. Nested conformance environment

The rule witness receives nested evidence slots for antecedent constraints such as:

```text
I |= Iterator
```

using the same deterministic slot identity rules as P1/P3.

#### D. Registry reuse

Add a discriminator around environment interning.

Required result:

```text
invoke same derived specialization N times
number of distinct type env entries stays constant after first materialization
number of distinct conformance env entries stays constant after first materialization
```

If production registry has no safe `len` inspection, add a test-only helper or compare returned environment IDs.

Do not add production mutable counters solely for tests if identity equality suffices.

#### E. GC/lifetime

Required strong sequence:

```text
obtain/capture callable or deferred operation that depends on derived evidence
force GC
invoke after GC
assert behavior/evidence still valid
```

If callable-reference Outcome B was recorded in T0, use another existing delayed activation path that legitimately persists the environment, and keep callable-reference deferral explicit.

#### F. VM audit

Search final VM/core diff and live VM files.

There must be no:

```text
GenericConstraint
DerivedConformanceRule
prove_conformance
Iterator trait ID conditional
AssociatedProjection normalization
```

inside VM execution logic.

### Runtime fixture

Use classes/methods only.

Conceptual source:

```phalcom
trait Stepper {
    type Item
    next -> Option<Self::Item>
}

trait Stream {
    type Item
    type Iterator
    iterator -> Self::Iterator
}

class CounterIter {
    ...
}

impl Stepper for CounterIter {
    type Item = Int
    ...
}

impl<I> Stream for I
    where I |= Stepper
{
    type Item = <I as Stepper>::Item
    type Iterator = I

    iterator {
        self
    }
}

class RuntimeProbe {
    @class
    same<I>(_ value: I) -> Bool
        where I |= Stream
    {
        value.iterator === value
    }
}
```

No free functions.

### Test changes required

Implement CV-40..46, CV-64..65, and P3 closure CV-78..79 where still missing.

Suggested tests:

```text
guarded_derived_witness_executes_with_type_environment
guarded_derived_witness_executes_with_nested_conformance_environment
iterator_like_derived_iterable_iterator_preserves_identity
generic_callee_requiring_iterable_accepts_iterator_derived_evidence
repeated_guarded_derived_calls_reuse_runtime_environments
guarded_derived_evidence_survives_gc_before_deferred_invocation
vm_executes_guarded_derived_evidence_without_rule_solver
```

### Tests to run now

Exact:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
derived_conformance::iterator_like_derived_iterable_iterator_preserves_identity -- --nocapture
```

Module:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
language::derived_conformance -- --nocapture
```

Predecessor runtime regression:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
language::conditional_conformance -- --nocapture
```

Then the T0 predecessor cases that touched runtime evidence if they were activated/fixed.

### Tests explicitly deferred

Universe production iteration smoke until T8.

### Acceptance

- derived rule behavior executes;
- type and nested conformance envs arrive correctly;
- repeated calls reuse env identity;
- GC/deferred invocation path is safe;
- P3 runtime conditional tests remain green;
- VM has no rule solver.

### Local STOP / CONSULT triggers

- runtime needs source rule objects;
- runtime needs proof search;
- a new persistent evidence-capture object is required;
- env IDs cannot be rooted/reconstructed using existing ownership;
- per-object evidence array becomes necessary.

### Checkpoint update

Record runtime reuse/rooting evidence after G4.

### Commit

```sh
git add phalcom-core/src phalcom-core/tests/core/language
git commit -m "feat(core): execute guarded derived conformance evidence"
```

---

## G4 — Runtime evidence gate

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
language::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
language::conditional_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
language::generic_conformance -- --nocapture
```

For `generic_conformance`, record ignored/scheduled tests separately; do not call ignored tests PASS unless actually run.

Expected:

```text
new P4 runtime module green
P3 conditional runtime green
P1 active runtime regressions unchanged/green
registry reuse discriminator green
post-GC discriminator green
```

---

## T7 — Publish derived rules durably across modules and incremental revisions

### Purpose

Make guarded derived rules snapshot-owned stable semantic products that downstream modules can consume without source reparsing and that invalidate precisely.

### Preconditions

- G4 passes.

### Consumes

```text
DerivedConformanceRule
DerivedConformanceRuleIndex
DB products/query keys
SemanticDependency
fingerprints
module mutation machinery
snapshot conformance products
source index
```

### Produces

```text
cross-module rule publication
stable rule fingerprint
precise dependency tracking
cold/incremental parity
source occurrence mapping
```

### Files and symbols

**Modify expected:**

```text
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/lib.rs
```

Maybe module interface/stable metadata owner if live architecture requires:

```text
phalcom-modules/...
phalcom-type-meta/...
```

Touch only after VERIFY-FIRST shows the current conformance source product crosses that boundary there.

**Create/register tests:**

```text
phalcom-semantic/tests/semantic/incremental/derived_conformance.rs
phalcom-semantic/tests/semantic/incremental/mod.rs
```

Extend capability P4 file for simple cross-module semantic proof.

### Required implementation shape

#### A. Stable product

A downstream snapshot must obtain the rule's canonical content without reparsing provider source.

Stable identity must include semantic equivalents of:

```text
source ImplId/provenance
canonical subject parameter/template
consequent stable TraitRef
canonical GenericSignature
associated binding templates
requirement selection identity
```

Do not persist:

```text
TypeId
inference variables
proof memo entries
runtime environment IDs
source ranges as semantic identity
```

Use repository stable-type export mechanisms.

#### B. Dependency recording

A derived proof consumer must depend on:

```text
consequent trait rule set
antecedent conformance facts it actually proves
trait contract
associated binding dependencies
```

Do not add “all conformances changed” global invalidation.

#### C. Canonical no-op reorder

Changing:

```text
where T |= A, T |= B
```

to:

```text
where T |= B, T |= A
```

must preserve semantic rule fingerprint and avoid unnecessary dependent recomputation where current infrastructure can observe it.

#### D. Mutation matrix

Build a provider/consumer test:

Provider:

```text
traits
derived rule
base Iterator-like conformance
```

Consumer:

```text
generic or exact Stream/Iterable use
```

Mutations:

```text
remove antecedent conformance
re-add
change rule condition
restore
reorder equivalent conditions
change trait contract
```

Compare incremental snapshot with cold rebuild at each meaningful state.

#### E. Source index

Map:

```text
rule consequent trait name
antecedent trait name
associated projections
associated binding names
trait where-clause projections
```

to canonical semantic targets.

Do not teach LSP to solve rules.

### Test changes required

Implement CV-53..60.

Suggested incremental tests:

```text
cross_module_guarded_rule_proves_downstream_goal
cross_module_guarded_rule_runtime_products_are_publishable
antecedent_conformance_removal_invalidates_derived_consumer
antecedent_conformance_readd_restores_derived_consumer
derived_rule_condition_edit_invalidates_consumer
derived_rule_constraint_reorder_is_semantic_noop
trait_contract_edit_invalidates_derived_evidence
derived_rule_incremental_matches_cold
```

### Tests to run now

Capability cross-module:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
derived_conformance::cross_module_guarded_rule_proves_downstream_goal -- --nocapture
```

Incremental module:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
incremental::derived_conformance -- --nocapture
```

If runtime cross-module is tested in core, run one exact core test, not the whole core suite.

### Tests explicitly deferred

Full LSP suite.

### Acceptance

- downstream proof consumes published rule;
- condition/conformance edits invalidate correctly;
- reorder is semantically stable;
- cold/incremental parity;
- no snapshot-local IDs leak into stable product;
- source index uses canonical targets.

### Local STOP / CONSULT triggers

- stable product cannot represent associated binding template without raw `TypeId`;
- downstream proof requires provider source reparsing;
- incremental correctness requires global invalidation;
- source index would need a second rule resolver.

### Checkpoint update

Record cross-module/incremental gate evidence.

### Commit

```sh
git add \
  phalcom-semantic/src \
  phalcom-semantic/tests/semantic/incremental \
  phalcom-semantic/tests/semantic/capabilities
git commit -m "feat(semantic): publish and invalidate derived conformance rules"
```

---

## G5 — Durable publication / incremental gate

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
incremental::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture
```

If DB/fingerprint changes materially touched generic contracts, also:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
incremental::generic_contracts -- --nocapture
```

Do not broaden after PASS.

---

## T8 — Ratify specifications and install the canonical Iterator / Iterable Universe law with transition staging

### Purpose

Make the standard Iterator/Iterable law a real source use of P4's general language mechanism while preserving production cursor behavior for P5.

### Preconditions

- G5 passes;
- generic derived rule works cross-module;
- transition-name audit complete.

### Consumes

```text
normative iterator model
trait contract
guarded derived rule
Universe source loading
legacy Iterable/Iterator classes
bootstrap/native class registrations
```

### Produces

```text
normative derived-conformance rule
canonical Universe Iterator trait
canonical Universe Iterable trait
real Universe Iterator⇒Iterable rule
transitional coexistence with cursor production
explicit P5 takeover boundary
```

### Files and symbols

**Normative docs:**

```text
docs/specs/objects/traits.md
docs/specs/objects/impl.md
docs/specs/generic-constraints.md
docs/specs/iterator.md
```

**Universe expected:**

```text
phalcom-core/core/universe/src/collections/iterable.ph
phalcom-core/core/universe/src/collections/package.ph
```

**Transition support only if required:**

```text
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/universe/invariants.rs
phalcom-native-meta/src/universe.rs
```

**Tests:**

```text
phalcom-core/tests/core/language/derived_conformance.rs
existing iteration smoke/corpus filters
```

### Required implementation shape

#### A. Normative spec amendment first in this task

`traits.md` must define:

```text
derived/synthesized conformance category
guarded generic-target rule
evidence semantics
one-proof coherence
```

`impl.md` must define:

```text
ordinary nominal/exact targets remain unchanged
guarded generic-target conformance rule is a separate conformance form
consequent-trait ownership
```

`generic-constraints.md` must clarify:

```text
trait's own GenericSignature is entailed by conformance evidence
Self is instantiated from conformance subject
rule GenericSignature uses ordinary constraint semantics
no reverse search
```

`iterator.md` must explicitly state that the Iterator theorem is realized by the generic derived-conformance mechanism and show the canonical source law.

No free functions in spec examples.

#### B. Canonical target source

Final public semantic declarations should be equivalent to:

```phalcom
trait Iterator {
    type Item

    next -> Option<Self::Item>

    iterator {
        self
    }
}

trait Iterable
    where
        Self::Iterator |= Iterator,
        <Self::Iterator as Iterator>::Item == Self::Item
{
    type Item
    type Iterator

    iterator -> Self::Iterator
}

impl<I> Iterable for I
    where I |= Iterator
{
    type Item = <I as Iterator>::Item
    type Iterator = I

    iterator {
        self
    }
}
```

Ordering may need `Iterator` before `Iterable` so the contract can resolve it.

#### C. Transition name collision

Current source has classes named `Iterable` and `Iterator`.

Preferred staging is **U-A** if live audit proves safe:

```text
legacy cursor base classes become internal transition names
canonical public names become traits
legacy production collection/adapters temporarily inherit internal bases
```

Example internal names are mechanically flexible:

```text
_$LegacyIterable
_$LegacyIterator
```

or repository-conventional internal identifiers.

The transition base must not be presented as the new semantic trait.

If U-A requires broad public/native identity breakage, use **U-B** only with a recorded plan amendment:

```text
canonical traits installed in a stable internal transition module/namespace
public flip deferred to P5
```

If neither can preserve production behavior without performing P5, STOP AND CONSULT.

#### D. Do not migrate production consumers

Do not add P4 production conformances for:

```text
List
Set
Map
Tuple
Bytes
Range
```

except a minimal private fixture used solely to prove Universe trait loading if required.

Do not rewrite `for`.

Do not rewrite lazy adapters.

Do not delete cursor methods.

#### E. Universe theorem test

Add a custom user iterator using the canonical Universe `Iterator` trait and call a generic method requiring canonical Universe `Iterable`.

That proves the real Universe declaration uses the generic rule.

Example:

```phalcom
class OneIter {
    mut _done: Bool

    @constructor
    new {
        _done = false
    }

    next -> Option<Int> {
        if (_done) {
            return None
        }
        _done = true
        Some(1)
    }
}

impl Iterator for OneIter {
    type Item = Int
}

class IteratorLawProbe {
    @class
    sameTraversal<I>(_ iterator: I) -> Bool
        where I |= Iterable
    {
        iterator.iterator === iterator
    }
}
```

Adapt actual constructor/return syntax to current Phalcom grammar; preserve method-only behavior.

### Test changes required

Implement CV-66..72.

Add:

```text
universe_iterator_trait_loads
universe_iterable_trait_contract_loads
universe_iterator_derives_iterable
universe_iterator_derived_item_projection_is_int
```

Legacy smoke must remain unchanged.

Identify focused existing filters for:

```text
for lowering/disassembly
List cursor iteration
Map cursor behavior
lazy cursor pipeline
```

Do not update their expectations to the P5 model.

### Tests to run now

New Universe P4 runtime:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
derived_conformance::universe_iterator_derives_iterable -- --nocapture
```

Then P4 core module.

Then exact legacy smoke filters determined from current test list.

At minimum preserve the C5/P3 smoke equivalents of:

```text
language-corpus list iteration
iteration disassembly smoke
iteration negative smoke
```

Use current filter names after `-- --list`.

### Tests explicitly deferred

Full collection corpus and P5 target behavior.

### Acceptance

- normative specs describe actual P4 mechanism;
- canonical Iterator/Iterable source uses ordinary language features;
- real Universe theorem works;
- legacy production cursor/for behavior still works;
- no collection/for migration enters P4.

### Local STOP / CONSULT triggers

- public trait names cannot coexist with production legacy behavior without P5;
- native bootstrap requires canonical trait to be a runtime class;
- trait declarations cannot be loaded from Universe without a new runtime trait descriptor architecture;
- installing canonical law changes production `for` semantics;
- transition requires changing all collection classes now.

### Checkpoint update

Record canonical Universe law and exact transition state.

### Commit

Split normative+Universe changes into one coherent commit if they are inseparable, otherwise:

```sh
git commit -m "docs(spec): define guarded derived conformance semantics"
git commit -m "feat(universe): install Iterator Iterable derived law"
```

---

## G6 — Canonical Universe / P5-boundary gate

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
language::derived_conformance -- --nocapture
```

Then named legacy smoke filters only.

Also run semantic P4 module once because real Universe source may expose additional contract formation issues:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture
```

Expected:

```text
canonical law green
legacy cursor/for smoke green
no P5 behavior change
```

---

## T9 — Final focused stabilization, coverage closure, records, walkthrough, and P5 handoff

### Purpose

Prove all P4 obligations with the intended narrow test set, update durable checkpoint truth, and hand P5 a stable semantic foundation.

### Preconditions

- G6 passes;
- no unresolved A/B failure.

### Consumes

All prior P4 artifacts.

### Produces

```text
final focused verification ledger
P4 walkthrough
P4 handoff
checkpoint P4 COMPLETE/IMPLEMENTED/FOCUSED_TESTED
P5 active/next action
residual baseline ledger
```

### Files

**Modify:**

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md
```

**Create:**

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-handoff.md
```

### Required implementation shape

#### A. Coverage ledger

Map every `P4-CV-01..80` to:

```text
PASS
N/A with architectural reason
DEFERRED with explicit accepted reason
```

Required semantic obligations may not be silently `N/A`.

#### B. Verify no forbidden special case

Targeted searches:

```sh
rg -n "Iterator|Iterable" phalcom-semantic/src/generic_evidence.rs phalcom-semantic/src/derived_conformance.rs
```

Expected:

```text
no semantic proof branch special-casing canonical Iterator/Iterable declaration identity
```

Names may occur in comments/tests/docs; inspect context.

VM audit:

```sh
rg -n "GenericConstraint|DerivedConformanceRule|prove_conformance|AssociatedProjection" \
  phalcom-core/src/vm phalcom-core/src/typing
```

Any hit in VM execution code must be explained; semantic proof logic is forbidden.

#### C. Focused regression set

Run serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::conditional_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::generic_associated_constraints -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
capabilities::generic_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
incremental::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
language::derived_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
language::conditional_conformance -- --nocapture
```

Run the exact T0 predecessor cases if they remain outside active modules.

Run exact legacy iteration smoke selected in T8.

#### D. Affected-crate compile

If P4 touched:

```text
phalcom-ast
phalcom-semantic
phalcom-type-meta
phalcom-core
phalcom-lsp
```

run `cargo check` only for crates actually touched.

Likely:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core
```

Add `phalcom-type-meta` / `phalcom-lsp` only if modified.

#### E. Diff hygiene

```sh
git diff --check
git status --short
```

Do not require unrelated clean working tree.

#### F. Walkthrough truth

Walkthrough must use live names, not planned pseudocode names when implementation differed.

#### G. P5 handoff

P5 must inherit:

```text
canonical Iterator trait
canonical Iterable trait
enforced Iterable contract
general Iterator⇒Iterable derived rule
stable rule publication
derived associated evidence
runtime evidence execution
exact transition state of legacy cursor classes
P4 baseline/residual issues
```

P5 must not re-design derived conformance.

### Tests explicitly deferred

Still do not run:

```text
cargo test --workspace
cargo clippy --workspace
full language corpus
release build
```

unless a focused failure requires one.

### Acceptance

P4 can be marked:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only when:

- required P4 coverage is accounted for;
- all named focused gates are green or an explicit accepted baseline classification applies;
- checkpoint records actual architecture;
- walkthrough/handoff exist;
- next action is P5;
- no forbidden Iterator-specific solver/runtime hack exists.

### Local STOP / CONSULT triggers

- any required semantic CV remains unproven because architecture is incomplete;
- final focused gate exposes the same unresolved semantic failure after one serious correction;
- P5 handoff cannot state a stable canonical law.

### Commit

```sh
git add \
  docs/implementation/LANG005/LANG005.C6/ \
  docs/specs/ \
  <remaining authorized P4 source/test files>

git commit -m "docs(lang005): complete C6 P4 Iterator Iterable law"
```

If code and docs completion changes were already committed task-wise, final commit should contain records only.

---

# 19. Verification gates summary

## G0 — takeover

Proves:

```text
live P3 interfaces known
predecessor gaps classified
P4 state truthful
```

## G1 — trait contract

Proves:

```text
subject-aware trait GenericSignature
own associated projections
contract enforcement
assumed-evidence entailment
```

## G2 — guarded rule proof/coherence

Proves:

```text
rule source product
recursive rule application
cycle termination
no ranking
memo reuse
```

## G3 — complete derived evidence

Proves:

```text
behavior + associated bindings
projection normalization
trait law satisfaction
P2/P3 compatibility
```

## G4 — runtime evidence

Proves:

```text
finite lowering
nested env
identity behavior
reuse
rooting
zero VM solver
```

## G5 — durable/incremental

Proves:

```text
cross-module publication
precise invalidation
fingerprint stability
cold parity
```

## G6 — Universe law

Proves:

```text
real canonical standard traits/rule
legacy production path preserved
P5 scope not consumed
```

## G7 — final focused stabilization

T9 final acceptance.

---

# 20. Final focused acceptance matrix

The walkthrough must contain a final table with at least:

| Requirement group | Required result |
|---|---|
| P3 closure | CV-73..80 accounted |
| trait formation | CV-01..06 PASS |
| trait contract | CV-07..14 PASS |
| rule formation | CV-15..21 PASS |
| recursive proof | CV-22..32 PASS |
| associated evidence | CV-33..39 PASS |
| runtime | CV-40..46 PASS |
| coherence | CV-47..52 PASS |
| cross-module/incremental | CV-53..60 PASS |
| performance/resources | CV-61..65 PASS with structural metric |
| Universe/P5 boundary | CV-66..72 PASS |

`N/A` is acceptable only when the requirements analysis explicitly allows an architectural branch such as callable-reference Outcome B and the handoff records the deferral.

---

# 21. Performance / resource evidence

P4 is not a benchmark plan, but it changes recursive proof and environment construction, so two structural metrics are mandatory.

## 21.1 Proof metric

Construct one fixture where the proof graph revisits the same Iterator goal through:

```text
rule antecedent
+
Iterable contract validation
+
associated projection normalization
```

Required evidence:

```text
proof path requests > unique canonical goals
memo_hits >= 1
completed canonical goals remain bounded
```

Do not require timing thresholds.

## 21.2 Rule lookup metric

Demonstrate rule candidate lookup is keyed by consequent trait.

A unit-level structural assertion is sufficient:

```text
N unrelated rules
1 Iterable rule
query Iterable
candidate count == relevant subset
```

Do not benchmark all workspace rules.

## 21.3 Runtime environment metric

For repeated same exact specialization:

```text
initial distinct type/conformance env count = X
after first call = Y
after N repeated calls = Y
```

or equivalent identical environment ID proof.

No timing threshold.

## 21.4 GC/lifetime

Evidence must execute after GC, not merely compute before GC and inspect a stored scalar after.

---

# 22. Checkpoint bookkeeping

## At plan start

Set/confirm:

```yaml
active_plan: LANG005.C6.P4
latest_completed_plan: LANG005.C6.P3
next_plan: LANG005.C6.P4
```

P4 ledger entry:

```text
IN_PROGRESS / PARTIAL / UNVERIFIED
```

after real implementation begins.

## During plan

Update only after:

```text
G0 predecessor closure classification
G1 trait-contract invariant
G2 derived rule/coherence invariant
G4 runtime evidence invariant
G5 cross-module/incremental invariant
G6 Universe transition invariant
consultation/amendment
new deferred baseline
```

Do not log every test.

## At completion

P4 entry:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

Checkpoint remains:

```text
IN_PROGRESS / PARTIAL / FOCUSED_TESTED
```

because P5/P6 remain.

Set:

```yaml
active_plan: LANG005.C6.P5
latest_completed_plan: LANG005.C6.P4
next_plan: LANG005.C6.P5
```

only if P5 is indeed the next approved action.

---

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-walkthrough.md
```

Required sections:

```text
1. Plan identity and final lifecycle state
2. Objective and outcome
3. Live architecture actually implemented
4. Trait contract staging
5. Derived rule product/index
6. Proof integration and cycle/coherence behavior
7. Derived evidence shape
8. Runtime lowering/environment behavior
9. Stable metadata/incremental behavior
10. Universe transition strategy actually chosen
11. Important files/symbols changed
12. Deviations from this plan
13. Consultations/amendments
14. Tests added
15. Tests actually executed
16. Tests deliberately deferred
17. P4-CV coverage mapping
18. Performance/resource evidence
19. Residual/baseline failures
20. Verification classification
21. P5 takeover summary
```

Never claim:

```text
workspace green
release certified
all tests pass
```

unless those gates actually ran.

---

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-handoff.md
```

Keep it operationally compact.

Must contain:

```text
current revision
P4 final status
canonical Iterator trait identity/source
canonical Iterable trait identity/source
Iterable trait contract
guarded derived-conformance rule symbol/path
derived rule index symbol/path
proof integration symbol/path
derived evidence shape
runtime recipe shape
Universe transition state
P5 next objective
must-read files
baseline/deferred failures
first recommended P5 commands
```

Explicit “do not redesign” list:

```text
do not redesign Iterator⇒Iterable
do not replace derived rule with compiler special case
do not introduce supertraits for this law
do not alter associated identity
do not move proof into VM
do not reintroduce cursor as public semantic protocol
```

---

# 25. Completion truth table

Keep these separate:

| State | Meaning |
|---|---|
| source written | implementation text exists |
| semantic tests added | tests exist, not necessarily green |
| exact discriminator PASS | one property proven |
| task module PASS | task acceptance likely |
| gate PASS | coherent stage proven |
| P4 IMPLEMENTED | intended P4 implementation exists |
| P4 FOCUSED_TESTED | named P4 gates passed |
| C6 COMPLETE | false until later plans finish |
| RELEASE_COMPLETE | false unless explicit release certification occurs |

A clean Git tree is not proof.

A passing broad test is not permission to skip missing P4 coverage.

---

# 26. Plan self-review checklist

Before implementation begins, confirm this plan satisfies:

- [x] no Phalcom free-function examples;
- [x] P4/P5 boundary explicit;
- [x] trait contract and Iterator theorem separate;
- [x] `ConformanceTarget` preserved;
- [x] `TraitRef` preserved;
- [x] `AssociatedTypeRequirementId` preserved;
- [x] one GenericSignature authority;
- [x] one projection normalizer;
- [x] one recursive proof context;
- [x] cycle non-success preserved;
- [x] no candidate ranking;
- [x] no fake exact evidence;
- [x] runtime proof-free;
- [x] runtime env separation preserved;
- [x] cross-module/incremental owned;
- [x] performance/resource discriminators included;
- [x] P3 predecessor gaps integrated;
- [x] Luna has objective STOP/CONSULT triggers;
- [x] BUILD testing intentionally sparse;
- [x] tasks form a dependency order;
- [x] walkthrough/handoff/checkpoint obligations included;
- [x] no broad workspace certification is accidentally mandatory.

---

# 27. Task dependency graph

```text
T0  takeover + P3 closure
 |
 v
T1  trait contract formation
 |
 v
T2  trait contract proof / entailment
 |
 G1
 |
 v
T3  guarded derived rule product/index/coherence
 |
 v
T4  recursive rule proof
 |
 G2
 |
 v
T5  complete derived behavior + associated evidence
 |
 G3
 |
 v
T6  runtime lowering / reuse / rooting
 |
 G4
 |
 v
T7  cross-module / incremental / tooling
 |
 G5
 |
 v
T8  normative specs + canonical Universe law / staging
 |
 G6
 |
 v
T9  focused stabilization + records + P5 handoff
```

Do not execute T8 early merely because the final source syntax is known. The Universe should be the first production consumer of a proven general mechanism, not the test bed in which the mechanism is invented.

---

# 28. Recommended implementation session partition for Luna

For best context discipline, execute as separate bounded sessions:

```text
Session 1:
    T0 + G0

Session 2:
    T1
    T2
    G1

Session 3:
    T3
    T4
    G2

Session 4:
    T5
    G3

Session 5:
    T6
    G4

Session 6:
    T7
    G5

Session 7:
    T8
    G6

Session 8:
    T9 / G7
```

If a session is interrupted, update the P4 handoff/checkpoint only when a coherent task boundary has been reached. Do not create ad-hoc state files.

---

# 29. Final P5 takeover contract

P4 is successful only if the P5 planner/implementer may assume all of the following as established:

```text
1. `Iterator` is a canonical trait with:
       type Item
       next -> Option<Self::Item>
       iterator { self }

2. `Iterable` is a canonical trait with:
       type Item
       type Iterator
       iterator -> Self::Iterator

3. Every valid Iterable conformance proves:
       Self::Iterator |= Iterator
       Iterator(Self::Iterator).Item == Self::Item

4. Every proven Iterator conformance derives one coherent Iterable conformance.

5. Derived Iterator→Iterable evidence binds:
       Iterable.Item = Iterator.Item
       Iterable.Iterator = Self

6. Derived `Iterable.iterator` returns the same iterator object.

7. Generic `T |= Iterator` can satisfy a required `T |= Iterable`.

8. Generic `T |= Iterable` implies the trait's own associated constraints.

9. Associated projections through derived evidence normalize through `types::normalize_type`.

10. Derived rules are stable cross-module semantic products.

11. Incremental mutation of antecedent evidence/rule/trait contract invalidates exactly the relevant consumers.

12. Runtime receives finite preselected evidence via existing type/conformance environments.

13. VM performs no conformance proof or associated lookup.

14. The canonical Universe law uses the general language mechanism.

15. The exact transitional legacy cursor state is documented.

16. Production collection and `for` behavior are still legacy and therefore P5 work.
```

P5 may then focus exclusively on migration:

```text
collections
stateful iterators
Map entry items
RangeIterator
lazy adapters
for semantic typing
for lowering
legacy cursor/superclass retirement
```

without re-opening P4 architecture.

---

# 30. Requirements traceability

This table is the explicit bridge from the P4 requirements analysis to executable plan ownership. No foundational requirement may disappear merely because the implementation task uses different prose.

| Requirement | Analysis requirement | Owning task | Verification gate |
|---|---|---|---|
| `P4-F01` | record actual P4 start revision | `T0` | `G0` |
| `P4-F02` | synchronize checkpoint/guidance current action | `T0` | `G0` |
| `P4-F03` | correct P3 handoff live interface names | `T0` | `G0` |
| `P4-F04` | disposition owner-generic runtime regression | `T0` | `G0` |
| `P4-F05` | disposition owner+method evidence regression | `T0` | `G0` |
| `P4-F06` | disposition exact enum-case generic regression | `T0` | `G0` |
| `P4-F07` | callable-reference Outcome A/B recorded | `T0` | `G0` |
| `P4-F08` | P3 CV/P2R closure ledger recorded | `T0` | `G0` |
| `P4-F09` | memo unique-goal discriminator established | `T4` | `G2` |
| `P4-F10` | runtime environment cardinality discriminator established | `T6` | `G4` |
| `P4-F11` | post-GC evidence invocation discriminator established or strong existing proof cited | `T6` | `G4` |
| `P4-F12` | associated declaration-bound record corrected | `T0` | `G0` |
| `P4-TC01` | `Iterable` owns one canonical GenericSignature contract | `T1` | `G1` |
| `P4-TC02` | trait contract can reference `Self` | `T1` | `G1` |
| `P4-TC03` | trait contract can reference own associated projections | `T1` | `G1` |
| `P4-TC04` | associated requirement identity remains C5 identity | `T1` | `G1` |
| `P4-TC05` | trait contract instantiation substitutes subject for `Self` | `T1` | `G1` |
| `P4-TC06` | exact Iterable evidence validates `Iterator` associated conformance | `T2` | `G1` |
| `P4-TC07` | exact Iterable evidence validates Item equality | `T2` | `G1` |
| `P4-TC08` | assumed `T \|= Iterable` entails contract constraints | `T2` | `G1` |
| `P4-TC09` | entailment reuses GenericEvidenceEnvironment / P3 proof | `T2` | `G1` |
| `P4-TC10` | trait contract cycles remain non-proof | `T2` | `G1` |
| `P4-TC11` | `TraitRef` identity remains separate from contract proof | `T1` | `G1` |
| `P4-TC12` | no new associated-declaration-bound syntax required for P4 | `T1` | `G1` |
| `P4-DC01` | ordinary impl syntax can express guarded generic-target rule | `T3` | `G2` |
| `P4-DC02` | generic-target rule is semantically distinct from ConformanceTarget | `T3` | `G2` |
| `P4-DC03` | rule source has stable ImplId/provenance | `T3` | `G2` |
| `P4-DC04` | rule consequent has canonical TraitRef template | `T3` | `G2` |
| `P4-DC05` | rule conditions are ordinary GenericSignature | `T3` | `G2` |
| `P4-DC06` | naked generic rule requires nontrivial guard | `T3` | `G2` |
| `P4-DC07` | source module owns consequent trait | `T3` | `G2` |
| `P4-DC08` | rule index is keyed by consequent trait | `T3` | `G2` |
| `P4-DC09` | rule application uses P3 proof context | `T4` | `G2` |
| `P4-DC10` | rule application preserves rich proof terminals | `T4` | `G2` |
| `P4-DC11` | no reverse inference | `T4` | `G2` |
| `P4-DC12` | no structural coincidence | `T3` | `G2` |
| `P4-DC13` | rule can supply behavioral witnesses | `T3` | `G2` |
| `P4-DC14` | rule can supply associated binding templates | `T3` | `G2` |
| `P4-DC15` | rule proof can consume exact evidence | `T4` | `G2` |
| `P4-DC16` | rule proof can consume assumed evidence | `T4` | `G2` |
| `P4-DC17` | rule proof can consume derived conditional evidence | `T4` | `G2` |
| `P4-DC18` | rule proof can recursively consume derived-rule evidence | `T4` | `G2` |
| `P4-DC19` | derived-rule cycles terminate non-success | `T4` | `G2` |
| `P4-DC20` | no Iterator name appears in semantic solver special cases | `T4` | `G2` |
| `P4-EV01` | existing P3 conditional Derived evidence meaning preserved | `T5` | `G3` |
| `P4-EV02` | derived evidence origin can represent guarded rule | `T5` | `G3` |
| `P4-EV03` | no fake exact nominal ConformanceEvidence | `T5` | `G3` |
| `P4-EV04` | derived evidence materializes exact subject | `T5` | `G3` |
| `P4-EV05` | derived evidence materializes exact consequent TraitRef | `T5` | `G3` |
| `P4-EV06` | derived evidence exposes associated binding map | `T5` | `G3` |
| `P4-EV07` | derived evidence exposes requirement selections | `T5` | `G3` |
| `P4-EV08` | nested conformance evidence retained | `T5` | `G3` |
| `P4-EV09` | equivalence/subtype proofs do not create runtime slots | `T5` | `G3` |
| `P4-EV10` | projections normalize from derived evidence | `T5` | `G3` |
| `P4-EV11` | generic assumed Iterator yields symbolic derived Iterable | `T5` | `G3` |
| `P4-EV12` | exact Iterator yields exact/materialized derived Iterable | `T5` | `G3` |
| `P4-EV13` | failed rule proof publishes no partial evidence | `T5` | `G3` |
| `P4-EV14` | runtime recipe consumes finite evidence DAG | `T6` | `G4` |
| `P4-EV15` | VM performs zero derived-rule proof | `T6` | `G4` |
| `P4-COH01` | P3 nominal conditional no-overlap rule unchanged | `T3/T4` | `G2` |
| `P4-COH02` | derived rule has separate coherence category | `T3/T4` | `G2` |
| `P4-COH03` | no candidate order precedence | `T3/T4` | `G2` |
| `P4-COH04` | no most-specific rule | `T3/T4` | `G2` |
| `P4-COH05` | no condition-disjointness solver | `T3/T4` | `G2` |
| `P4-COH06` | explicit + successful derived same relation conflicts | `T4` | `G2` |
| `P4-COH07` | overlapping derived rules conflict | `T3/T4` | `G2` |
| `P4-COH08` | imported rules participate in coherence | `T4` | `G2` |
| `P4-COH09` | adding antecedent conformance can reveal conflict and invalidates dependents | `T7` | `G5` |
| `P4-COH10` | exact enum-case identity remains exact | `T4` | `G2` |
| `P4-COH11` | rule ownership is consequent-trait ownership | `T3/T4` | `G2` |
| `P4-COH12` | coherence conflict remains semantic, never runtime | `T6` | `G4` |
| `P4-IT01` | canonical Iterator trait uses associated Item | `T8` | `G6` |
| `P4-IT02` | next returns Option<Item> | `T8` | `G6` |
| `P4-IT03` | Iterator.iterator returns self | `T8` | `G6` |
| `P4-IT04` | every proven Iterator derives Iterable | `T4/T5/T8` | `G3/G6` |
| `P4-IT05` | derived Iterable::Item = Iterator::Item | `T5/T8` | `G3/G6` |
| `P4-IT06` | derived Iterable::Iterator = subject | `T5/T8` | `G3/G6` |
| `P4-IT07` | derived Iterable.iterator = self | `T5/T8` | `G3/G6` |
| `P4-IT08` | derived evidence satisfies Iterable trait contract | `T4/T5/T8` | `G3/G6` |
| `P4-IT09` | generic `T \|= Iterator` can satisfy callee `T \|= Iterable` | `T4/T5/T8` | `G3/G6` |
| `P4-IT10` | generic associated projection through derived Iterable normalizes | `T4/T5/T8` | `G3/G6` |
| `P4-IT11` | custom user Iterator receives theorem | `T8` | `G6` |
| `P4-IT12` | cross-module custom Iterator receives theorem | `T8` | `G6` |
| `P4-IT13` | structural next/iterator shape alone does not receive theorem | `T8` | `G6` |
| `P4-IT14` | Iterator theorem is not encoded as supertrait | `T8` | `G6` |
| `P4-IT15` | Iterator theorem has no compiler/VM branch | `T9` | `G7` |
| `P4-U01` | real Universe source defines canonical Iterator trait | `T8` | `G6` |
| `P4-U02` | real Universe source defines canonical Iterable trait | `T8` | `G6` |
| `P4-U03` | real Universe source defines Iterator⇒Iterable rule | `T8` | `G6` |
| `P4-U04` | public-name collision with legacy classes explicitly resolved | `T8` | `G6` |
| `P4-U05` | production cursor semantics remain functioning | `T8` | `G6` |
| `P4-U06` | production for lowering remains unchanged | `T8` | `G6` |
| `P4-U07` | collection migration remains P5 | `T8` | `G6` |
| `P4-U08` | lazy adapter migration remains P5 | `T8` | `G6` |
| `P4-U09` | legacy superclass retirement remains P5 | `T8` | `G6` |
| `P4-U10` | P4 handoff identifies exact P5 transition state | `T9` | `G7` |
| `P4-I01` | derived rule has stable semantic fingerprint | `T7` | `G5` |
| `P4-I02` | equivalent condition reorder is fingerprint-stable | `T7` | `G5` |
| `P4-I03` | rule condition edits invalidate consumers | `T7` | `G5` |
| `P4-I04` | antecedent conformance edits invalidate derived consumers | `T7` | `G5` |
| `P4-I05` | trait contract edits invalidate Iterable evidence | `T7` | `G5` |
| `P4-I06` | imported rule proves downstream goal | `T7` | `G5` |
| `P4-I07` | imported rule runtime behavior executes | `T7` | `G5` |
| `P4-I08` | cold/incremental parity | `T7` | `G5` |
| `P4-I09` | source index maps rule trait/projection occurrences | `T7` | `G5` |
| `P4-I10` | stable metadata contains no snapshot ids | `T7` | `G5` |

If live implementation evidence makes any row genuinely inapplicable, do not silently delete it. Record the disposition in the P4 walkthrough and checkpoint with the exact architectural reason.

# 31. Final executor directive

Implement this plan as a sequence of small, reviewable semantic patches.

Do not optimize for “fewest files changed.”

Optimize for:

```text
one canonical owner per fact
small proof surface
evidence before runtime
explicit lifecycle state
bounded testing
easy architectural review
```

When a task passes its discriminator, move forward.

When a baseline is unrelated, record it and move forward.

When architecture is contradicted, stop and consult rather than patching around it.
