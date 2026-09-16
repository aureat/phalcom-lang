---
id: LANG005.C6.P1
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C5
follows: LANG005.C5.P3
supersedes: null
prepared: 2026-09-16
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: 14352f3824e8dcfc9f5b3f46d9f4a4bb5cf48709
---

# LANG005.C6.P1 — Canonical Generic Conformance Evidence and Runtime Transport

> **For Luna / agentic implementers:** Execute this plan task-by-task under the repository's Luna patch-grade workflow. Re-read the live tree before editing, adapt mechanical drift locally, and STOP AND CONSULT rather than changing any FIXED architecture below. Use sparse, discriminating BUILD tests. Do not reinterpret this plan as authorization to implement generic associated projections, conditional conformances, the production iterator migration, or the remainder of LANG005.C6.

**Goal:** Establish one complete direct generic-conformance evidence vertical in Phalcom: source `where Subject |= TraitApplication` constraints become canonical generic propositions and lexical assumed evidence, generic bodies dispatch through that evidence, generic applications discharge direct obligations from exact or enclosing assumed evidence, compiler/runtime lowering transports deterministic requirement targets through the existing conformance-environment channel, and public generic contracts survive stable metadata, cross-module, incremental, reflection, and tooling boundaries without runtime proof search.

**Architecture:** Extend the existing `GenericSignature` / `GenericConstraint` authority with a general `Conforms(subject, TraitRef)` proposition and introduce one canonical semantic generic-evidence layer. Exact C4/C5 `ConformanceEvidence` remains exact source-proof authority; generic assumptions remain abstract evidence and never masquerade as `ImplId` or exact evidence. Semantic call specialization records whether each callee conformance obligation is discharged by exact evidence or forwarded caller evidence. Core lowering projects that decision into separate type and conformance call-entry recipes; the VM only copies/materializes already-selected method handles into the existing interned `RuntimeConformanceEnvironment` and continues to execute `InvokeTraitRequirement`.

**Tech stack:** Rust workspace (`phalcom-ast`, `phalcom-semantic`, `phalcom-type-meta`, `phalcom-type-syntax`, `phalcom-native-meta`, `phalcom-core`, `phalcom-lsp`), semantic DB/incremental products, stable metadata, compiler/bytecode/VM integration, Phalcom source fixtures, Cargo integration tests.

**Primary specification:** `docs/specs/generic-constraints.md`

**Target repository path when landed:**

```text
docs/implementation/LANG005/LANG005.C6/
  LANG005.C6-CHECKPOINT.md
  LANG005.C6.P1-canonical-generic-conformance-evidence-and-runtime-transport-plan.md
```

At planning revision `14352f3824e8dcfc9f5b3f46d9f4a4bb5cf48709`, the `LANG005.C6/` directory does not yet exist. Verify that before creating it; if another agent/user has already initialized C6, consume that live checkpoint record instead of creating a competing one.

---

## 0. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High or an equivalently capable implementation model working under constrained architectural authority.

The implementer:

- may adapt helper names, local decomposition, imports, and equivalent private APIs to the live repository;
- must preserve the semantic ownership, identity, evidence, metadata, lowering, runtime, and lifetime invariants below;
- must not improvise material architecture;
- must not create a second trait-bound checker/solver beside the existing generic-constraint system;
- must not represent generic assumptions as fake source conformances;
- must not move generic proof or trait discovery into the compiler or VM;
- must follow the BUILD / STABILIZE / CERTIFY testing budget rather than testing reflexively;
- must classify unrelated failures before spending implementation time on them;
- must keep the shared `LANG005.C6-CHECKPOINT.md` current at durable gates;
- must create the required P1 walkthrough and P1-to-P2 handoff;
- must STOP AND CONSULT when any global or task-local trigger fires;
- must not commit or push unless the surrounding workflow/user instruction explicitly authorizes it.

Testing is evidence gathering, not ritual. A test is run because it discriminates an invariant, validates a durable interface, or satisfies a named gate.

---

## 1. Goal

Implement the direct generic trait-conformance/evidence foundation required by all later C6 work, while preserving C4/C5 exact conformance and associated-type architecture and without prematurely implementing associated specialization, recursive/conditional proof, blanket conformances, or production iteration migration.

The minimum end-to-end user-visible success case is:

```phalcom
trait Printable {
  print -> String
}

class User {}

impl Printable for User {
  print { "user" }
}

class Renderer {
  @class
  render<T>(_ value: T) -> String
      where T |= Printable
  {
      value.print
  }
}

Renderer.render(User.new())
```

The generic body is accepted because `T |= Printable` supplies assumed evidence. The concrete call is accepted because `User` has exact `Printable` evidence. Runtime execution invokes the selected witness/default through an evidence slot; it does not perform structural method lookup or conformance search.

A second mandatory success case proves composition rather than only concrete specialization:

```phalcom
class Forwarder {
  inner<T>(_ value: T) -> String
      where T |= Printable
  {
      value.print
  }

  outer<T>(_ value: T) -> String
      where T |= Printable
  {
      self.inner(value)
  }
}
```

`outer` satisfies `inner` from enclosing assumed evidence. The compiler/runtime forward the caller's evidence slot into the callee's environment; no exact `T` conformance is invented.

---

## 2. Checkpoint acceptance objective

`LANG005.C6` eventually owns the complete generic-constraint/evidence model required for generic associated projections, associated specialization/equality, conditional conformances, nested proof, iterator/iterable laws, and production iteration migration.

P1 is the **foundation vertical**, not full checkpoint closure.

### 2.1 P1 bounded acceptance objective

> A `where Subject |= TraitApplication` constraint, including same-subject `+` conjunction, elaborates into canonical direct conformance obligations inside the existing generic signature system; those obligations become lexical assumed evidence while generic bodies are checked; bound-provided trait behavior resolves through canonical trait requirement identity; direct generic applications discharge obligations after ordinary generic inference from either exact C4/C5 conformance evidence or compatible enclosing assumed evidence; semantic call specialization retains that proof source; semantic-to-core lowering projects deterministic evidence-slot recipes; runtime call entry materializes/interns an ordinary `RuntimeConformanceEnvironment` without conformance search; generic body requirement calls execute through the existing abstract trait-requirement mechanism; and public/cross-module generic signatures preserve direct conformance bounds as stable metadata with incremental, reflection, navigation, and presentation parity.

### 2.2 P1 is not checkpoint closure

```text
C6.P1
    direct generic conformance propositions/evidence
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
    canonical Universe migration
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

The exact future numbering may be amended if repository evidence warrants it. The architectural dependencies above must not be bypassed.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 14352f3824e8dcfc9f5b3f46d9f4a4bb5cf48709
relevant predecessor: LANG005.C5.P3
predecessor checkpoint state: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

### 3.1 Recent drift verified during planning

```text
14352f3824e8dcfc9f5b3f46d9f4a4bb5cf48709
docs(work): record test relocation and modular audit probes work log

ed9272bddb2dcb043bc24457a142657bab13fe71
refactor(tests): relocate src test modules to tests, organize modular audit probes, and move audit fixtures

8c389d7a59dddf51fff93efb1bebe2732bd0881b
refactor(tests): rename historical checkpoint and stage test files to descriptive names

b6cf027ea5332c7e18a3ec390eef2e107390b773
feat(lang005): complete C5 associated-type vertical integration, iterator substrate, and certification (P3)
```

New C6 tests must use descriptive semantic names; do not reintroduce historical names such as `p3`, `stage4`, `plan_b`, or checkpoint-internal implementation-history labels.

### 3.2 Live facts verified at planning revision

1. `GenericConstraint` currently has only `Subtype` and `Equivalent`; `GenericSignature` owns parameters, kinds, variances, `constraint_shapes`, and canonical constraints.
2. `GenericConstraintSyntax` currently has `Subtype`, `Equivalent`, and `Invalid`; `GenericParameterSyntax` still carries binder-local `kind: Option<KindSyntax>`.
3. `<:` is a dedicated token; `|` exists as `Token::Pipe`; there is no dedicated `|=` token.
4. `CheckingContext` still carries `ambient_constraints: Vec<GenericConstraint>` and body construction uses it for conditional inherent-impl domains rather than a full lexical generic proof environment.
5. `TraitRef::form` currently validates declaration, arity, argument kinds **and** referenced-trait generic constraints.
6. `CallSpecialization` currently publishes callable identity plus canonical type-parameter bindings, not conformance evidence sources.
7. exact ordinary trait dispatch is exact-evidence oriented; `TraitDispatchSite::{AbstractRequirement,Evidenced}` does not yet represent generic assumed evidence.
8. stable `GenericConstraintRef` has only `Subtype` and `Equivalent`.
9. `RuntimeConformanceEnvironment` is an interned positional method-handle slot array and explicitly is not a semantic lookup table.
10. exact trait calls and abstract requirement calls already lower through `TraitInvocationSpec` / `TraitRequirementInvocationSpec`.
11. `RuntimeTypeEnvironmentRegistry` and `RuntimeConformanceEnvironmentRegistry` are separate runtime authorities.
12. associated-type hover still formats `` `Trait.Item` `` instead of `` `Trait::Item` ``.
13. no `LANG005.C6/` implementation directory exists at the planning revision.
14. semantic tests are now organized under the single `tests/semantic.rs` integration binary; core compiler/runtime tests live under `phalcom-core/tests/core/`.

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

### 4.2 C5 takeover state

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P3-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P3-handoff.md
```

### 4.3 Normative language authority

```text
docs/specs/README.md
docs/specs/generic-constraints.md
docs/specs/objects/traits.md                     # or live authoritative equivalent
authoritative associated-types specification
authoritative generic/type-formation/kind specification
authoritative subtyping/equivalence specification
authoritative callable/selector/dispatch specification
authoritative iterator specification, if already landed
```

If a historical C5 record conflicts with the normative generic-constraints specification, the normative specification wins.

### 4.4 Exact source/test conventions

At minimum re-read:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs

phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/source_index/builder.rs

phalcom-type-meta/src/generic.rs
phalcom-type-syntax/src/lib.rs
phalcom-native-meta/src/types.rs

phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/chunk.rs
phalcom-core/src/compiler/
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs

phalcom-lsp/src/backend.rs

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
phalcom-semantic/tests/semantic/capabilities/constraints.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs
phalcom-core/tests/core/language/associated_traits.rs
phalcom-core/tests/core/language/mod.rs
```

Use `rg`, exact symbol navigation, and `cargo test ... -- --list` to rediscover mechanically moved tests. Do not restart a broad repository audit.

---

## 5. Normative authority

### 5.1 Relation model

The relation families are semantically distinct:

```text
:     kind classification
|=    trait conformance / evidence requirement
<:    subtype relation
==    canonical semantic type equivalence
```

Trait conformance is not subtyping, kinding, assignment, or structural method matching.

### 5.2 Parameter introduction versus constraints

Generic parameter lists introduce parameters:

```phalcom
<T, U, F>
```

`where` constrains already-bound/resolvable entities.

Do not introduce:

```phalcom
<T: Printable>
```

as trait-bound syntax.

### 5.3 Same-subject conjunction

```phalcom
where T |= Printable + Hashable
```

means:

```text
Conforms(T, Printable)
Conforms(T, Hashable)
```

No combined trait identity is created. Source order is semantically irrelevant.

### 5.4 Associated syntax distinction

```text
value.method                  behavioral message send

Trait::Item
Self::Item
T::Item
<T as Trait>::Item            associated selection/projection
```

Standalone equivalence uses `==`:

```phalcom
T::Item == A
```

Single `=` is reserved for named associated specialization:

```phalcom
T |= Iterator<Item = A>
```

P1 does not implement those generic associated semantics.

### 5.5 `Dynamic`

`Dynamic` is never automatic proof of a static `|=` requirement.

### 5.6 General conformance subject

The primitive is:

```text
Conforms(subject, trait_application)
```

not:

```text
BoundOnGenericParameter(parameter, trait)
```

It must be representationally capable of:

```phalcom
T |= Printable
List<T> |= Printable
F<T> |= Serializable
I::Item |= Hashable
F |= Functor
```

where surrounding kind/projection features exist.

### 5.7 Trait subject kinds in P1

Positive P1 execution targets the current ordinary proper-type trait subject model. Do not invent higher-kinded trait subject declaration syntax to activate `F |= Functor`.

---

## 6. Takeover state

C5 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

### 6.1 Stable interface map

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Generic signature | `GenericSignature` | semantic | one canonical generic contract |
| Existing constraints | `GenericConstraint::{Subtype,Equivalent}` | semantic | preserved behavior |
| Applied trait identity | `TraitRef` | semantic | declaration + canonical args; not runtime type |
| Behavioral requirement | `TraitRequirementId` | semantic | trait-owned stable identity |
| Associated requirement | `AssociatedTypeRequirementId` | semantic | trait-owned stable identity |
| Trait surface | `TraitSurface` | semantic | canonical published trait contract |
| Exact conformance discovery | `ConformanceIndex` | semantic | source conformance authority |
| Exact proof | `ConformanceEvidence` / `ConformanceResolution` | semantic | never fabricated |
| Exact ordinary trait dispatch | `TraitDispatch*` | semantic | compiler consumes selection |
| Associated projection | `TypeData::AssociatedProjection` | semantic | C5 canonical representation |
| Projection normalization | `types::normalize_type` | semantic | sole recursive C5 normalizer |
| Runtime type substitution | `RuntimeTypeEnvironment` | runtime | type substitutions only |
| Runtime evidence handles | `RuntimeConformanceEnvironment` | runtime | executable selected handles only |
| Abstract requirement invoke | `InvokeTraitRequirement` | VM | active evidence slot call |
| Stable generic metadata | `GenericConstraintRef` | metadata | durable generic contract |
| Tooling associated target | `SemanticTargetId::AssociatedType` | semantic/tooling | canonical navigation identity |

### 6.2 Frozen C5 identities

Do not redesign:

```text
ImplId
TraitRef
TraitRequirementId
AssociatedTypeRequirementId
TraitSurface
ConformanceIndex
ConformanceHeadMatch
ConformanceWitnessPlan
ConformanceAssociatedTypePlan
ConformanceEvidence
ConformanceCompleteness
TraitDispatch*
TypeData::AssociatedProjection
types::normalize_type
SemanticTargetId::AssociatedType
RuntimeTypeEnvironment
RuntimeConformanceEnvironment
```

---

## 7. Architecture

### 7.1 Authority flow

```text
source where-clause
        ↓
AST constraint syntax
        ↓
canonical GenericSignature
  GenericConstraint::Conforms
        ↓
lexical GenericEvidenceEnvironment
        ↓
direct conformance proof
    ├── ProvenAssumed
    └── ProvenExact
        ↓
generic-body dispatch / generic-call discharge
        ↓
CallSpecialization + evidence bindings
        ↓
semantic-to-core lowering recipe
    ├── type bindings
    └── conformance slot recipes
        ↓
RuntimeTypeEnvironment + RuntimeConformanceEnvironment
        ↓
InvokeTraitRequirement
```

Semantic analysis owns proof and identity decisions. Core/VM only consume/project them.

### 7.2 Canonical semantic extension

```rust
pub enum GenericConstraint {
    Subtype {
        lower: TypeTerm,
        upper: TypeTerm,
    },
    Equivalent {
        left: TypeTerm,
        right: TypeTerm,
    },
    Conforms {
        subject: TypeTerm,
        trait_ref: TraitRef,
    },
}
```

Exact spelling may adapt; semantic shape is FIXED.

### 7.3 Canonical conformance key

Conceptually:

```rust
pub struct GenericConformanceKey {
    pub subject: TypeTerm,
    pub trait_ref: TraitRef,
}
```

If `GenericConstraint::Conforms` itself safely serves as the deterministic key, a wrapper is optional. Source spans/spelling are not identity.

### 7.4 Canonicalization

```text
source constraints
    ↓
expand same-subject +
    ↓
resolve canonical subject / TraitRef
    ↓
stable semantic sort
    ↓
deduplicate exact propositions
    ↓
publish GenericSignature
```

Thus these are semantically equal:

```phalcom
where T |= A + B
```

```phalcom
where T |= B + A
```

```phalcom
where
    T |= A,
    T |= B
```

### 7.5 Kind constraints

If the live path remains additive, add:

```phalcom
where F: (Type) -> Type
```

as syntax that merges into existing `GenericSignature.parameter_kinds`.

Do **not** add semantic `GenericConstraint::Kind`.

Preserve binder-local kind syntax as compatibility input unless a newer normative migration explicitly removes it. Conflicting kind facts are hard errors.

### 7.6 TraitRef formation split

Establish:

```text
TraitRef identity formation:
    declaration
    ordinary type arguments
    arity
    argument kinds

separate from

trait-application well-formedness:
    instantiate referenced GenericSignature
    discharge obligations under current evidence
```

Preferred API shape:

```rust
TraitRef::form_identity(...)
```

plus generic-evidence-owned application proof. Exact helper names may adapt.

### 7.7 Generic evidence environment

Recommended new semantic owner:

```text
phalcom-semantic/src/generic_evidence.rs
```

Conceptually:

```rust
pub struct GenericEvidenceEnvironment {
    // canonical lexical propositions
    constraints: ...,
    // indexed direct conformance assumptions
    conformances: BTreeMap<GenericConformanceKey, AssumedConformanceEvidence>,
    // deterministic executable behavioral evidence contract
    layout: GenericEvidenceLayout,
}
```

Required semantics:

- lexical and compositional;
- canonical/deduplicated;
- indexed for direct assumptions;
- provenance preserving;
- exact-versus-assumed aware;
- future P2/P3 consumable;
- no fake `ImplId`;
- no fake `ConformanceEvidence`.

`CheckingContext.ambient_constraints` must cease to be an independently mutated authority. Existing conditional-inherent code may consume a derived relation view from the evidence environment.

### 7.8 Lexical composition

A method sees owner plus callable generic propositions:

```phalcom
class Box<T>
    where T |= Printable
{
  inspect<U>(_ other: U)
      where U |= Debug
  {
    // both assumptions are visible
  }
}
```

### 7.9 Direct proof API

Conceptually:

```rust
pub enum GenericConformanceEvidence {
    Exact(Arc<ConformanceEvidence>),
    Assumed(AssumedConformanceEvidence),
}

pub enum GenericConformanceProof {
    Proven(GenericConformanceEvidence),
    Missing,
    Ambiguous(...),
    Incomplete(...),
    Unknown(...),
    Blocked(...),
    Dynamic(...),
    Cancelled,
    BudgetExceeded(...),
    InternalFailure(...),
}
```

P1 proof sequence:

```text
normalize/kind-check direct subject
    ↓
matching lexical assumption?
    ├── yes → ProvenAssumed
    └── no
         ↓
    subject exact enough?
         ↓
    delegate to existing exact ConformanceResolution
         ↓
    preserve rich outcome
```

P1 does not recursively evaluate conditional conformances.

### 7.10 Generic-body trait dispatch

Under `T |= Printable`, `value.print` resolves through `Printable`'s canonical trait surface and `TraitRequirementId`.

Add a separate assumed dispatch origin conceptually:

```rust
pub struct AssumedTraitDispatchSelection {
    pub conformance: GenericConformanceKey,
    pub requirement: TraitRequirementId,
    pub signature: CallableSemanticSignature,
}

pub enum TraitDispatchSite {
    AbstractRequirement { requirement: TraitRequirementId },
    Evidenced(TraitDispatchSelection),
    Assumed(AssumedTraitDispatchSelection),
}
```

Do not inject members into inherent target tables and do not forge exact selection provenance.

### 7.11 Evidence-slot identity

A requirement ID alone is insufficient because the same trait may be assumed for different subjects.

Define semantic slot identity as:

```rust
pub struct GenericEvidenceSlotKey {
    pub conformance: GenericConformanceKey,
    pub requirement: TraitRequirementId,
}
```

and deterministic layout:

```rust
pub struct GenericEvidenceLayout {
    pub slots: Box<[(GenericEvidenceSlotKey, u16)]>,
}
```

Required properties:

- source-order independent;
- `+`/comma-spelling independent;
- duplicates do not duplicate slots;
- same requirement under different conformance subjects does not alias;
- checked u16 capacity, never truncation.

### 7.12 Generic call discharge

```text
argument / expected-type inference
    ↓
existing subtype/equivalence solving
    ↓
candidate canonical substitution
    ↓
instantiate direct Conforms obligations
    ↓
prove:
    exact C4/C5 evidence
    OR enclosing assumed evidence
    ↓
publish call evidence recipe
```

Do not turn direct conformance into an ordinary unification relation in P1.

### 7.13 Call specialization evidence

Conceptually extend the call product:

```rust
pub enum GenericCallConformanceSource {
    Exact {
        evidence: Arc<ConformanceEvidence>,
    },
    Forwarded {
        caller_conformance: GenericConformanceKey,
    },
}

pub struct GenericCallConformanceBinding {
    pub callee_conformance: GenericConformanceKey,
    pub source: GenericCallConformanceSource,
}

pub struct CallSpecialization {
    pub callable: CallableId,
    pub bindings: Box<[(TypeParameterId, TypeId)]>,
    pub conformances: Box<[GenericCallConformanceBinding]>,
}
```

Lowering must never re-prove which source applies.

### 7.14 Runtime call-entry recipe

Keep logical lanes separate:

```text
semantic CallSpecialization
    ├── type bindings
    └── conformance evidence bindings
            ↓
core lowering
    ├── Runtime type recipe
    └── Runtime conformance slot recipe
            ↓
VM call entry
    ├── RuntimeTypeEnvironmentId
    └── RuntimeConformanceEnvironmentId
```

Conformance slot sources are:

```text
Exact:
    compiler-prepared selected method handle

Forwarded:
    caller environment slot M
```

### 7.15 Runtime execution

```text
construct callee evidence slots
    ↓
intern RuntimeConformanceEnvironment
    ↓
enter callee with type + conformance environment
    ↓
InvokeTraitRequirement(slot)
```

No runtime conformance search, trait syntax inspection, or projection normalization.

### 7.16 Stable metadata

Conceptually:

```rust
pub struct StableTraitApplicationRef {
    pub declaration: StableDeclarationRef,
    pub arguments: Box<[TypeNodeId]>,
}

pub enum GenericConstraintRef {
    Subtype { ... },
    Equivalent { ... },
    Conforms {
        subject: TypeNodeId,
        trait_application: StableTraitApplicationRef,
    },
}
```

Do not serialize semantic `TypeId`, semantic `TraitRef`, inference variables, evidence objects, or runtime environment IDs.

### 7.17 Reflection

Do not pretend conformance is a binary type/type relation. Extend reflection with relation-aware conformance accessors/tagged data so users can observe:

```text
subject
trait declaration/application
trait arguments
```

### 7.18 Incremental identity

Semantic generic-contract fingerprints must depend on canonical propositions, not raw source order/presentation. `constraint_shapes` may remain source/presentation metadata, but it must not cause `A+B` versus `B+A` to become semantically distinct.

---

## 8. Ownership boundaries

### Owns

- lexical `|=` syntax;
- same-subject `+`;
- direct canonical `GenericConstraint::Conforms`;
- additive canonical where-kind syntax if bounded verification succeeds;
- canonicalization/dedup/order independence;
- lexical evidence environment;
- direct assumed/exact proof;
- generic-body assumed trait dispatch;
- post-inference bound discharge;
- generic-to-generic forwarding;
- deterministic evidence layouts;
- compiler/runtime evidence transport;
- stable direct-bound metadata/reflection;
- cross-module/incremental/tooling integration;
- inherited associated-hover `::` correction.

### Does not own

- generic `T::Item`;
- `<T as Trait>::Item`;
- `Trait<Item = T>`;
- associated-driven inference;
- conditional conformances;
- recursive/nested proof;
- blanket/derived conformances;
- `Iterator => Iterable`;
- production iterator/collection/`for` migration;
- Map/Range/lazy adapter migration;
- cursor retirement;
- aliases/GADT full generic constraint closure;
- supertraits;
- trait objects;
- higher-kinded trait subject declaration syntax.

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| source relation spelling | parser/AST | semantic formation/tooling | checker string parsing |
| generic propositions | `GenericSignature` | checker/metadata/incremental/reflection | trait-bound side table |
| direct conformance identity | `GenericConstraint::Conforms` / key | evidence/proof/lowering | parameter-only shortcut |
| trait application identity | `TraitRef` | proof/dispatch/metadata | runtime type identity |
| trait application proof | generic evidence layer | declarations/calls | `TraitRef` constructor |
| lexical assumptions | `GenericEvidenceEnvironment` | dispatch/calls/future P2/P3 | flat per-consumer vectors |
| exact proof | `ConformanceEvidence` | generic proof/exact dispatch | fake generic exact evidence |
| assumed proof | generic evidence layer | dispatch/forwarding | fake source conformance |
| call discharge | semantic call specialization | core lowering | compiler proof |
| slot identity | conformance key + requirement | lowering/runtime | requirement alone |
| runtime type substitution | `RuntimeTypeEnvironment` | VM/runtime typing | conformance env |
| executable evidence | `RuntimeConformanceEnvironment` | requirement invoke | semantic lookup |
| stable conformance | stable metadata | import/reflection | snapshot IDs |
| bound navigation | source index | LSP | text matching |

---

## 9. Global invariants

**INV-01 — One generic-contract authority.** `GenericSignature` remains canonical for parameters, kinds, variances, subtype/equivalence, and direct conformance propositions.

**INV-02 — General conformance subject.** `Conforms(subject, TraitRef)` is not restricted to a naked generic parameter.

**INV-03 — Conjunction is sugar only.** `T |= A + B` becomes independent obligations; no combined trait exists.

**INV-04 — Order independence/idempotence.** Source order, `+` versus comma grouping, and duplicate identical obligations do not alter semantic identity or evidence layout.

**INV-05 — Singular kind authority.** `where F: Kind` feeds existing parameter kinds; no semantic `GenericConstraint::Kind`.

**INV-06 — Trait identity and proof are separate.** `TraitRef` owns identity; generic evidence owns proof.

**INV-07 — Generic assumptions are lexical evidence.** Owner + callable constraints compose in one environment.

**INV-08 — Assumed evidence is not exact evidence.** No fake `ImplId`, source conformance, exact evidence, or exact fingerprint.

**INV-09 — Trait members remain trait members.** Generic bounds expose canonical trait requirements; no inherent method injection.

**INV-10 — Structural coincidence is insufficient.** Matching selectors do not prove conformance.

**INV-11 — Generic evidence composes.** Bounded generic caller can forward evidence to bounded generic callee.

**INV-12 — Conformance follows ordinary inference.** P1 discharges direct bounds after current type solving.

**INV-13 — Blocked is not false; Dynamic is not proof.** Preserve proof-state distinctions.

**INV-14 — Semantic layer owns proof.** Compiler/runtime consume recipes and never search conformances.

**INV-15 — Runtime type/conformance state remain separate.**

**INV-16 — Slot identity is complete.** `(GenericConformanceKey, TraitRequirementId)` identifies a generic evidence slot.

**INV-17 — Slot layout is deterministic.** Source order cannot define runtime slot identity.

**INV-18 — Existing abstract requirement execution is reused.** Prefer `InvokeTraitRequirement`; duplicate opcode requires consultation.

**INV-19 — Runtime environments are interned/reused.** Growth tracks distinct evidence selections/layouts, not call count.

**INV-20 — Stable metadata contains only stable identities.**

**INV-21 — Public/cross-module contracts are complete.**

**INV-22 — Incremental and cold semantics converge.**

**INV-23 — Tooling consumes canonical semantic identity.**

**INV-24 — Associated notation uses `::`.**

**INV-25 — C5 and existing subtype/equivalence semantics remain valid.**

---

## 10. Non-goals

P1 must not implement:

```text
T::Item generic projection semantics
<T as Trait>::Item qualified projection semantics
Trait<Item = T> associated specialization
associated-type-driven inference
conditional conformance
nested recursive conformance proof
proof-cycle/memoized worklist
blanket/derived conformance
Iterator => Iterable general law
production Iterator/Iterable migration
production for checker/lowering migration
Map direct-iteration semantic change
Range/lazy iterator rewrite
cursor protocol retirement
generic alias/GADT conformance-bound closure
supertraits
trait objects/existentials
higher-kinded trait subject declaration syntax not already normatively specified
```

Do not perform a broad metadata, runtime typing, or language-spec redesign merely because the direct relation touches those systems.


---

## 11. Expected impact map

All paths are planning-revision expectations. Re-read before editing.

### 11.1 AST/parser

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/lexer.rs
phalcom-ast/tests/parser.rs
```

### 11.2 Semantic generic/evidence layer

```text
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/native.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/lib.rs
```

Recommended focused new owner:

```text
phalcom-semantic/src/generic_evidence.rs
```

Use an existing better focused module only if it already owns exactly this responsibility.

### 11.3 Stable/native/runtime type metadata

```text
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/... live validation/import helpers
phalcom-type-syntax/src/lib.rs
phalcom-native-meta/src/types.rs
```

Native-authored direct conformance constraints are VERIFY-FIRST. Exhaustive relation handling must be correct even if the native authoring surface remains deferred.

### 11.4 Core lowering/compiler/runtime

```text
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/chunk.rs
phalcom-core/src/compiler/
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/frame.rs                         # only if call-entry plumbing requires
```

### 11.5 Tooling

```text
phalcom-lsp/src/backend.rs
phalcom-lsp/src/semantic_tokens.rs               # if new token classification requires
phalcom-lsp/tests/... live descriptive hover/navigation tests
```

### 11.6 Semantic tests

```text
phalcom-semantic/tests/semantic/capabilities/constraints.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/capabilities/mod.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/integration/
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs
```

Recommended new test module:

```text
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
```

### 11.7 Core tests

Recommended:

```text
phalcom-core/tests/core/language/generic_conformance.rs
phalcom-core/tests/core/language/mod.rs
```

Use `associated_traits.rs` only for focused C5 regressions; do not turn it into a mixed historical C6 bucket.

### 11.8 Documentation/state

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-canonical-generic-conformance-evidence-and-runtime-transport-plan.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-handoff.md
```

### 11.9 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem when doing so changes semantic authority, canonical identity, stable metadata contracts, runtime lifetime/rooting, or P1 acceptance scope.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. `GenericSignature` as the single generic-contract authority.
2. general `Conforms(subject, TraitRef)` semantics.
3. `+` as independent obligation sugar.
4. source-order-independent/deduplicated semantic identity.
5. existing kind authority rather than a semantic `GenericConstraint::Kind`.
6. separation of `TraitRef` identity from application proof.
7. lexical generic evidence environment as generic-assumption authority.
8. exact versus assumed proof-source distinction.
9. prohibition on fake `ImplId` / `ConformanceEvidence`.
10. trait requirement identity as generic-bound behavioral identity.
11. structural coincidence not satisfying `|=`.
12. generic-to-generic evidence forwarding.
13. post-inference direct-conformance discharge.
14. `Dynamic` not proving a static bound.
15. semantic proof authority; no compiler/VM proof search.
16. separate runtime type/conformance environments.
17. slot identity `(conformance key, TraitRequirementId)`.
18. deterministic source-order-independent slot layout.
19. runtime conformance environment interning/reuse.
20. stable metadata containing stable identities only.
21. cross-module preservation.
22. incremental/cold equivalence.
23. canonical source-index/LSP identities.
24. associated `::` presentation.
25. all explicit P1 non-goals.

### 12.2 MECHANICALLY FLEXIBLE

Luna may adapt:

- exact names of private evidence structs/enums;
- helper method names;
- private map/vector/set implementation choices;
- whether a named canonicalization helper lives in `types/parameter.rs` or `generic_evidence.rs`;
- source span/provenance container details;
- parser helper decomposition;
- diagnostic wording that preserves the same semantic category;
- exact descriptive test function names;
- whether core call-entry conformance recipes are fields on `RuntimeCallEnvironmentRecipe` or a sibling referenced product, provided runtime authorities remain separate;
- compiler helper decomposition;
- mechanical path drift after test relocation;
- imports/borrow-checker/signature cleanup.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| live HEAD is only mechanical drift from `14352f3` | git status/branch/revision/log | adapt mechanically; consult if C6 architecture already landed differently |
| C6 directory/checkpoint is still absent | `docs/implementation/LANG005/` | consume existing live record; never duplicate |
| `GenericConstraint` still has two relation variants | `types/parameter.rs` | integrate with already-landed canonical extension; consult on competing authority |
| `TraitRef::form` still performs generic-contract proof | `traits.rs` + callers | adapt split to live API; consult if identity ownership changed |
| checker still has flat ambient constraints | `checker/context.rs`, `checker/body.rs` | migrate equivalent substrate; consult if another evidence authority exists |
| `CallSpecialization` still carries only type bindings | `checker/analysis.rs` | extend live canonical product |
| trait dispatch site still has only abstract/exact origins | `trait_dispatch.rs` | preserve live origins and add explicit generic-assumed origin |
| stable metadata still lacks conformance | `phalcom-type-meta/src/generic.rs` | reuse any already-landed stable representation |
| runtime conformance env remains positional handles | core typing module | adapt mechanically; consult if identity/lifetime changed |
| `InvokeTraitRequirement` can safely represent generic assumed slots | semantic lowering/compiler/VM | reuse; STOP/CONSULT if incompatible |
| generic semantic product fingerprint still considers `constraint_shapes` | `db/fingerprint.rs` | preserve any already-fixed canonical identity |
| current traits use ordinary `Type` subject authority | trait/kind spec + `TraitHeader` | do not invent higher-kind subject syntax |
| reflection remains binary-relation shaped | type syntax/runtime typing inspection | add relation-aware extension; consult on broader breaking migration |
| relocated test architecture remains current | test READMEs + `-- --list` | use live descriptive paths/filters |
| associated hover still uses `.` | LSP backend/tests | if already fixed, preserve; do not duplicate |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing the affected issue and build a consultation packet if any of the following occurs.

### Repository/workflow

1. planned architecture appears impossible or materially wrong;
2. a core premise is disproved by live source;
3. spec/plan/live code/relevant tests materially conflict;
4. scope expands into a materially different subsystem for architectural reasons;
5. a stable downstream contract relied upon by later tasks must materially change.

### P1-specific architecture

6. generic assumptions appear to require fake `ImplId` or fake `ConformanceEvidence`;
7. assumed generic trait dispatch cannot use canonical trait requirement identity without a second dispatch identity system;
8. correct generic forwarding would require runtime conformance declaration search;
9. runtime evidence transport would require semantic `TraitRef`, `TypeId`, or syntax in VM state;
10. `TraitRef` identity cannot be separated from recursive/nested proof without redesigning trait identity;
11. stable direct-conformance metadata would require serializing snapshot-local semantic IDs;
12. a P1 direct bound requires generic associated projection fixed-point inference;
13. a P1 direct bound requires conditional-conformance recursion;
14. evidence slot identity/layout must depend on source order;
15. same-trait/different-subject evidence cannot be represented without slot aliasing;
16. reusing `InvokeTraitRequirement` would change identity/lifetime semantics rather than broaden evidence source;
17. evidence transport requires changing GC rooting/lifetime policy;
18. correct owner-level generic evidence requires per-object conformance arrays/new runtime generic reification not already planned;
19. `where F: Kind` requires broad kind-system identity/syntax migration rather than additive normalization;
20. relation-aware reflection requires a materially breaking public runtime reflection redesign;
21. passing tests would require weakening exact conformance, ambiguity, `Dynamic`, or C5 projection invariants.

### Debugging

22. the same nontrivial semantic failure survives one serious evidence-based correction;
23. two plausible fixes have materially different ownership/identity consequences;
24. a special-case compiler/VM workaround is emerging;
25. a second canonical cache/index/proof authority would be introduced.

A triggered consultation cannot be self-waived.

### Consultation packet

```text
plan/task:
invariant:
expected architecture:
live revision:
minimal reproducer:
relevant symbols/call path:
scoped diff:
observed result:
root-cause hypothesis:
alternative hypothesis if material:
serious correction already attempted:
exact trigger:
decision requested:
```

Ask a decision question, not for a fresh repository audit.

---

## 14. Debugging budget

### Mechanical failures

Allow up to three coherent correction cycles while each has a concrete cause and makes progress:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest useful discriminator
```

Examples: import/signature drift, enum exhaustiveness, borrow cleanup, relocated test selector.

### Semantic failures

Before editing:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic problem. If it persists: STOP AND CONSULT.

### Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless something relevant changed.

---

## 15. Testing surface analysis

Every coverage ID must appear in the final walkthrough mapping.

| Coverage ID | Dimension | Invariants | Required case |
|---|---|---|---|
| CV-01 | lexical success | INV-01 | dedicated `|=` lexing; ordinary `|` unchanged |
| CV-02 | parse success | INV-01, INV-02 | `where T |= Printable` |
| CV-03 | conjunction | INV-03, INV-04 | `T |= A + B` → two obligations |
| CV-04 | applied trait identity | INV-02, INV-06 | `T |= Converts<A, B>` |
| CV-05 | kind reconciliation | INV-05 | `where F: (Type) -> Type`; conflict is error |
| CV-06 | general subject | INV-02 | `List<T> |= Printable` forms canonical subject |
| CV-07 | duplicate idempotence | INV-04 | duplicate identical bounds publish one proposition/layout contribution |
| CV-08 | order/sugar equivalence | INV-04, INV-17, INV-22 | `A+B`, `B+A`, comma grouping semantically equal |
| CV-09 | body assumption | INV-07, INV-09 | trait-only member callable under bound |
| CV-10 | negative body | INV-09, INV-10 | same member unavailable without evidence |
| CV-11 | structural negative | INV-10 | same-shaped methods do not satisfy bound |
| CV-12 | lexical composition | INV-07 | owner + callable bounds visible together |
| CV-13 | exact discharge | INV-08, INV-12 | concrete conforming caller succeeds |
| CV-14 | missing evidence | INV-08 | nonconforming caller rejected |
| CV-15 | dynamic boundary | INV-13 | `Dynamic` not static proof |
| CV-16 | generic forwarding | INV-11 | bounded caller satisfies bounded callee |
| CV-17 | multiple bounds | INV-16, INV-17 | independent slots dispatch correctly |
| CV-18 | same trait/different subjects | INV-16 | `T |= A, U |= A` slots remain distinct |
| CV-19 | ambiguity | INV-04, INV-09 | two assumed traits with same selector remain ambiguous |
| CV-20 | trait default execution | INV-14, INV-18 | selected default executes |
| CV-21 | explicit witness execution | INV-14, INV-18, INV-25 | detached witness executes |
| CV-22 | exact case | INV-08, INV-25 | exact enum-case conformance remains exact |
| CV-23 | stable metadata | INV-20 | conformance subject + applied trait round-trip |
| CV-24 | cross-module | INV-20, INV-21 | exported bounded callable retains contract and executes |
| CV-25 | reflection | INV-20 | conformance observed without trait-as-type lie |
| CV-26 | incremental change | INV-22 | add/remove/change invalidates affected consumers |
| CV-27 | incremental semantic no-op | INV-04, INV-22 | reorder/regroup preserves semantic fingerprint/cold parity |
| CV-28 | runtime repetition | INV-19 | repeated identical calls do not grow env registry per invocation |
| CV-29 | source navigation | INV-23 | bound trait resolves to canonical declaration |
| CV-30 | associated presentation | INV-24 | `Trait::Item` hover |
| CV-31 | existing constraint regression | INV-25 | subtype/equivalence behavior stays green |
| CV-32 | exact trait regression | INV-25 | C4/C5 exact trait/default/bound-method behavior stays green |
| CV-33 | publication | INV-01, INV-20 | valid Conforms publishes; inference-local subject/args rejected |
| CV-34 | slot boundary | INV-16, INV-17 | checked u16 slot capacity; no truncation |
| CV-35 | terminal states | INV-13 | blocked/cancel/budget/internal not collapsed to missing |
| CV-36 | recovery | INV-08, INV-22 | failed bound call does not contaminate later analysis |

No P1 positive test should claim support for generic associated projection/specialization, conditional conformance, recursive proof, blanket conformance, Iterator⇒Iterable, production iteration, or higher-kinded trait-subject conformance.

---

## 16. Verification execution budget

### Modes

**BUILD MODE:** sparse exact regressions/new feature tests only.

**STABILIZE MODE:** focused affected semantic/core/tooling surfaces.

**CERTIFY MODE:** broader workspace/release gates only when explicitly required.

### Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

Do not automatically climb after PASS.

### Mandatory during BUILD

- AST/parser tasks: exact new lexer/parser tests only.
- semantic contract/evidence tasks: exact `capabilities::generic_conformance` tests plus exact existing constraint regression when touched.
- dispatch/call tasks: exact body/call evidence tests.
- lowering/runtime tasks: exact `language::generic_conformance` tests.
- metadata/incremental/tooling tasks: exact round-trip/cross-module/fingerprint/navigation/hover tests.

### Mandatory during STABILIZE

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
cargo test -p phalcom-semantic --test semantic capabilities::constraints
cargo test -p phalcom-core --test core language::generic_conformance
```

Also run the exact affected metadata/incremental/LSP modules named by the gates.

### Only if evidence demands

```sh
cargo test -p phalcom-semantic --test semantic capabilities::traits
cargo test -p phalcom-semantic --test semantic impls::queries
cargo test -p phalcom-core --test core language::associated_traits
cargo test -p phalcom-core --test core
cargo test -p phalcom-semantic --test semantic
```

### Explicitly DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace
full language-corpus
all semantic integration tests
all core tests
all LSP tests
release builds
```

Cargo validation commands should run serially when competing builds would waste resources. Confirm filters select actual tests; zero selected tests are not evidence.

---

## 17. Baseline / unrelated failure policy

Classify:

```text
A — definitely caused by P1
B — probably caused by P1
C — unclear
D — clearly unrelated / inherited baseline
```

- A/B: active responsibility.
- C: one bounded classification pass; defer if nonblocking and still unclear.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks P1 acceptance.
- Do not weaken/skip assertions to make broad gates green.

### Inherited C5 baselines

```text
C5-BL-01 / C5-BL-04
    legacy generic call-entry / Range / Option.match issue

C5-BL-07
    two Universe Bool dependency/capability failures
```

For P1:

```text
new bounded-generic direct call reproduces BL-01/04 root defect
    → P1-owned blocker

new bounded-generic direct call executes cleanly
    → preserve baseline for later production iteration/runtime work
```

`C5-BL-07` remains D unless P1 demonstrably intersects it.

Do not repair the legacy iterator stack merely because P1 touches generic call-entry infrastructure.


---

# 18. Tasks

## T1 — Extend source syntax and canonical generic contracts

### Purpose

Add the direct conformance relation to the lexical/parser/AST surface and publish it through the existing canonical `GenericSignature` system with order-independent/deduplicated semantics. Reconcile canonical where-clause kind syntax with existing kind authority if the verified live path remains additive.

### Preconditions

- C5 takeover invariants accepted.
- normative generic-constraints spec read.
- live parser/token paths re-read.
- C6 checkpoint record initialized/active per §22.
- no competing C6 generic-bound implementation found on live `main`.

### Consumes

```text
Token / lexer operator conventions
GenericConstraintSyntax
WhereClauseSyntax
GenericParameterSyntax / KindSyntax
GenericSignature
GenericConstraint::{Subtype,Equivalent}
type annotation resolution
generic signature fingerprinting
```

### Produces

1. source `|=` relation token/syntax;
2. same-subject conformance conjunction syntax;
3. canonical `GenericConstraint::Conforms`;
4. canonical dedup/order normalization;
5. stable semantic subject + `TraitRef` identity formation;
6. `where F: Kind` normalized into existing kind metadata if VERIFY-FIRST succeeds;
7. publication validation for direct conformance constraints.

### Files and symbols

**Read/modify:**

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/db/fingerprint.rs
```

**Audit/modify exhaustive consumers as needed:**

```text
phalcom-semantic/src/types/native.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-type-syntax/src/lib.rs
phalcom-native-meta/src/types.rs
```

**Tests:**

```text
phalcom-ast/tests/lexer.rs
phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/capabilities/constraints.rs
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs  # create
phalcom-semantic/tests/semantic/capabilities/mod.rs                  # register live module
```

### Required implementation shape

#### T1.A — Dedicated lexical relation

Add a dedicated lexical token for `|=` using the existing longest-match operator pattern. Recommended semantic name:

```rust
Token::Conforms
```

The exact variant name is mechanically flexible.

Requirements:

- `|=` lexes as one relation token;
- ordinary `|` remains `Token::Pipe`;
- semantic token classification treats `|=` as an operator;
- no change to closure-pipe or bitwise-OR behavior.

#### T1.B — AST relation

Add conformance syntax capable of preserving:

```phalcom
subject |= Trait
subject |= A + B
subject |= Converts<X, Y>
```

The subject must be a type-level expression/annotation, not a bare generic parameter/string.

Future associated specialization may need richer trait-bound argument syntax. If the cleanest AST reserves that carrier now, that is allowed, but P1 semantic formation must diagnose/defer associated specialization rather than publish false direct `Conforms` semantics.

#### T1.C — Kind constraint syntax

If live parser architecture confirms the additive path, add:

```phalcom
where F: (Type) -> Type
```

as a syntax-only kind constraint.

Semantic formation must merge it into the existing parameter kind entry.

Do not add semantic `GenericConstraint::Kind`.

If explicit where-kind support requires a broad higher-kind parser/type-identity rewrite: STOP AND CONSULT and leave that migration for a bounded follow-up rather than expanding P1.

#### T1.D — Semantic direct conformance

Publish:

```rust
GenericConstraint::Conforms {
    subject: TypeTerm,
    trait_ref: TraitRef,
}
```

The `TraitRef` is canonical applied-trait identity, not proof.

#### T1.E — Canonicalization

Introduce one canonicalization pass that:

1. expands `+`;
2. canonicalizes subject/trait identity;
3. orders semantic constraints deterministically;
4. removes exact duplicate propositions;
5. keeps diagnostic/navigation provenance separately.

Existing subtype/equivalence semantics remain intact.

#### T1.F — Fingerprint correction

Audit `hash_generic_signature`.

Semantic/product fingerprinting must be based on canonical semantic facts. Raw `constraint_shapes` may remain for source presentation/backward compatibility but must not make semantically equivalent reorder/regrouping a different public contract.

Do not make source/input fingerprinting insensitive to source text; only semantic product identity must normalize.

#### T1.G — Publication validation

Extend `GenericSignature::validate_publishable`:

- validate the conformance subject term;
- validate every canonical trait type argument;
- reject inference-local subject/arguments crossing publication;
- preserve current subtype/equivalence publication rules.

### Forbidden approaches

- no `BoundOnGenericParameter(TypeParameterId, Trait)`;
- no checker-local bound table disconnected from `GenericSignature`;
- no compound/synthetic trait identity;
- no source-order slot numbering;
- no `T: Trait` trait-bound syntax;
- no standalone `T::Item = A`;
- no associated specialization semantics;
- no second kind authority;
- no semantic identity based on rendered source shape.

### Test changes required

Cover:

```text
CV-01 CV-02 CV-03 CV-04 CV-05 CV-06
CV-07 CV-08 CV-31 CV-33
```

Parser tests must assert actual AST structure rather than only parse success. Semantic tests must inspect canonical `GenericSignature` constraints/kinds.

### Tests to run now

```sh
cargo test -p phalcom-ast --test lexer lexes_conformance_constraint_operator
cargo test -p phalcom-ast --test parser parses_direct_trait_conformance_constraint
cargo test -p phalcom-ast --test parser parses_same_subject_trait_conjunction
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::canonical_direct_conformance_constraints_are_order_independent
cargo test -p phalcom-semantic --test semantic capabilities::constraints::source_equivalence_and_lower_bound_constraints_control_calls
```

Verify exact names with `-- --list` if path/module drift occurred.

### Tests explicitly deferred

- runtime execution;
- metadata round-trip;
- cross-module execution;
- broad semantic/core suites.

### Acceptance

Direct conformance is a canonical `GenericSignature` fact with correct syntax, identity, dedup/order behavior, publication validation, and no regression to existing subtype/equivalence constraints.

### Local STOP / CONSULT triggers

- parser design would force `|=` through ordinary expression/assignment parsing;
- general conformance subject cannot be represented without materially changing type-term architecture;
- canonicalization requires changing accepted subtype/equivalence semantics;
- where-kind support becomes a broad kind-system rewrite;
- semantic identity cannot be separated from raw `constraint_shapes`.

### Checkpoint update

Record the canonical direct relation plus order/idempotence invariant.

---

## T2 — Split TraitRef identity from proof and establish direct conformance proof

### Purpose

Separate applied-trait identity construction from generic-contract well-formedness and introduce one canonical direct conformance proof operation that can return exact or assumed evidence without flattening terminal proof states.

### Preconditions

- T1 publishes canonical `GenericConstraint::Conforms`.
- canonical conformance keys can be constructed.
- C4/C5 exact `ConformanceResolution` remains valid.

### Consumes

```text
TraitRef
TraitHeaderTable / TraitSurfaceTable
GenericSignature
ConformanceIndex
ConformanceEvidence
ConformanceResolution
TypeHierarchy
TypeStore
```

### Produces

1. identity-only `TraitRef` formation;
2. separate trait-application well-formedness;
3. semantic generic conformance key;
4. typed direct conformance proof result;
5. exact/assumed successful proof-source distinction;
6. preservation of rich exact terminal states.

### Files and symbols

**Create (recommended):**

```text
phalcom-semantic/src/generic_evidence.rs
```

**Modify:**

```text
phalcom-semantic/src/lib.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/snapshot.rs            # if public snapshot proof entry point belongs here
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
```

### Required implementation shape

#### T2.A — TraitRef identity-only formation

Refactor current `TraitRef::form` responsibilities so canonical formation checks:

```text
declaration is a trait
arguments are canonical
arity matches
argument kinds match
```

but does not recursively own generic constraint proof.

Preferred API:

```rust
TraitRef::form_identity(...)
```

Migrate all production callers so one obvious identity constructor remains.

#### T2.B — Trait application well-formedness

Create a separate operation receiving current proof/evidence context and discharging the referenced trait's `GenericSignature`.

For P1:

- subtype/equivalence trait constraints continue to work;
- direct conformance constraints can be proved from assumed/exact evidence;
- recursive conditional proof is not performed;
- blocked dependent applications stay blocked/unknown.

#### T2.C — Proof model

Recommended shape:

```rust
pub enum GenericConformanceEvidence {
    Exact(Arc<ConformanceEvidence>),
    Assumed(AssumedConformanceEvidence),
}
```

`AssumedConformanceEvidence` contains canonical conformance identity plus origin/provenance sufficient for diagnostics/lowering, never a fake `ImplId`.

Proof outcome must preserve at least:

```text
Proven exact / assumed
Missing / NotDeclared
Ambiguous / coherence conflict
Incomplete
Unknown / underconstrained
Blocked
Dynamic
Cancelled
Budget exceeded
Internal failure
```

#### T2.D — Exact fallback

For exact/direct subjects not satisfied by an assumption, delegate to existing exact conformance authority. Do not clone C4/C5 proof logic.

#### T2.E — P3 seam

Provide an explicit proof-context seam where P3 can later add conditional candidates, nested obligations, memoization, cycle state, and proof budgets without replacing P1's public proof model.

### Forbidden approaches

- no bool-only proof API;
- no fake exact evidence;
- no recursive conditional-conformance search;
- no compiler/VM proof;
- no duplicate exact conformance index;
- no exact-case widening;
- no `Dynamic => Proven`.

### Test changes required

Cover:

```text
CV-13 CV-14 CV-15 CV-22 CV-33 CV-35
```

At this task, assumed proof may use a unit-level evidence fixture; lexical population arrives in T3.

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::direct_conformance_proof_preserves_exact_and_terminal_outcomes
```

Run focused `capabilities::traits`/`impls::queries` regressions only if the migrated `TraitRef` path requires them.

### Tests explicitly deferred

- body dispatch;
- generic call forwarding;
- VM execution;
- metadata.

### Acceptance

Trait identity formation no longer owns generic proof, and one direct proof API distinguishes exact/assumed evidence while preserving exact proof terminal states.

### Local STOP / CONSULT triggers

- production TraitRef formation callers prove materially broader semantics than planning found;
- identity/proof cannot be split without changing `TraitRef` meaning;
- exact proof terminal states cannot be preserved;
- accepted P1 cases require recursive conditional proof.

### Checkpoint update

Record TraitRef identity/proof ownership split and proof API.

---

## T3 — Build the lexical GenericEvidenceEnvironment and deterministic evidence layout

### Purpose

Replace the flat ambient constraint authority with one lexical, compositional generic evidence environment and define deterministic evidence-slot identity for owner/callable assumptions.

### Preconditions

- T1 canonical constraints exist.
- T2 proof/evidence types exist.
- owner/callable generic signatures are queryable while bodies are built.

### Consumes

```text
GenericSignature
GenericConstraint
GenericConformanceKey
AssumedConformanceEvidence
CheckingContext
body owner/callable signature publication
conditional inherent applicability relation consumers
TraitSurfaceTable
```

### Produces

1. `GenericEvidenceEnvironment`;
2. owner + callable lexical composition;
3. indexed direct conformance assumptions;
4. relation view for existing conditional inherent consumers;
5. `GenericEvidenceSlotKey`;
6. deterministic `GenericEvidenceLayout`;
7. provenance for diagnostics/lowering.

### Files and symbols

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/editor.rs                  # if it consumes ambient constraints
phalcom-semantic/src/db/fingerprint.rs          # if layout/product becomes fingerprinted
```

Tests:

```text
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
```

### Required implementation shape

#### T3.A — Replace ambient authority

`CheckingContext` should carry one canonical evidence environment/view.

Existing `ambient_constraints` must be removed or become a strictly derived compatibility view during migration; it must not remain independently mutable.

#### T3.B — Lexical composition

At body setup compose:

```text
enclosing declaration generic signature
+
callable generic signature
+
legitimate existing conditional-inherent domain assumptions
```

while retaining provenance boundaries.

#### T3.C — Direct conformance index

Index assumptions by canonical key. Direct member/proof lookup must not scan every generic constraint for every send.

#### T3.D — Slot key

Identity is:

```text
canonical conformance obligation
+
TraitRequirementId
```

even if represented internally as a tuple.

#### T3.E — Deterministic layout

Expand visible direct conformance assumptions over canonical behavioral trait requirements and assign deterministic slots.

```text
source reorder       → same layout
A+B vs comma         → same layout
duplicate bound      → same layout
T |= A, U |= A       → distinct A.requirement slots
```

Associated type requirements do not consume executable behavioral slots.

#### T3.F — Slot limit

Use checked conversion/allocation for `u16` indices. Follow existing bytecode overflow conventions; never truncate.

### Forbidden approaches

- no full constraint scan per send;
- no source-order slot indexes;
- no `TraitRequirementId -> slot` map that aliases subjects;
- no independently assembled ambient vectors;
- no runtime type arguments in conformance layout;
- no associated type executable slots.

### Test changes required

Cover:

```text
CV-07 CV-08 CV-12 CV-17 CV-18 CV-34
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::owner_and_callable_conformance_assumptions_compose_lexically
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::generic_evidence_layout_is_order_independent_and_subject_sensitive
```

### Tests explicitly deferred

- member dispatch execution;
- call forwarding execution;
- core runtime tests.

### Acceptance

Every body has one canonical lexical evidence environment; assumptions are indexed; deterministic slots distinguish conformance subject plus trait requirement independent of source order/sugar.

### Local STOP / CONSULT triggers

- owner-level assumptions cannot be derived from existing owner/callable semantic products without new identity;
- conditional inherent applicability requires an independently mutable constraint authority;
- deterministic layout would require source positions;
- same-trait/different-subject evidence cannot be represented without changing runtime semantics materially.

### Checkpoint update

Record lexical evidence environment and complete evidence-slot identity/layout.

---

## T4 — Resolve generic-body trait behavior through assumed evidence

### Purpose

Make trait-only behavior semantically available on generic subjects under direct conformance bounds while preserving ordinary trait ambiguity and exact C4/C5 dispatch semantics.

### Preconditions

- T3 lexical evidence environment/layout exists.
- trait surfaces are canonical/queryable.
- T2 proof can return assumed evidence.

### Consumes

```text
GenericEvidenceEnvironment
TraitSurfaceTable
TraitRequirementId
GenericEvidenceSlotKey
existing trait/member dispatch
ExpressionAnalysis trait dispatch product
```

### Produces

1. assumed generic trait member candidates;
2. explicit assumed trait dispatch site/selection;
3. instantiated requirement signatures;
4. ambiguity behavior across assumed traits;
5. no-bound/structural negative behavior.

### Files and symbols

```text
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expression.rs       # if live lookup routes here
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/db/fingerprint.rs
```

Tests:

```text
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-semantic/tests/semantic/impls/queries.rs
```

### Required implementation shape

#### T4.A — Separate assumed dispatch origin

Add an explicit assumed generic evidence origin, conceptually:

```rust
TraitDispatchSite::Assumed(AssumedTraitDispatchSelection)
```

Do not force assumed evidence into exact `TraitDispatchSelection`, whose current live fields contain `source_impl` and exact evidence provenance.

#### T4.B — Requirement discovery

When the evidence environment contains direct conformance assumptions compatible with the receiver:

1. inspect canonical assumed trait surfaces;
2. select by selector/side;
3. instantiate requirement signatures under generic subject/trait arguments;
4. retain `(conformance key, requirement)` as executable identity.

#### T4.C — Ambiguity

If independent assumed traits expose the same selector and do not converge under existing rules, preserve trait ambiguity; never select by bound/source order.

#### T4.D — Structural negative

Same-shaped inherent methods are not proof of `|=`.

### Forbidden approaches

- no target method-table injection;
- no first-bound-wins;
- no string-only bound resolution;
- no fake exact candidate;
- no runtime class dictionary fallback for trait-only assumed behavior;
- no naked-generic-parameter-only special case.

### Test changes required

Cover:

```text
CV-09 CV-10 CV-11 CV-12 CV-19
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::trait_behavior_is_available_under_assumed_conformance
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::structural_method_coincidence_does_not_prove_generic_conformance
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::assumed_trait_selector_ambiguity_is_not_source_ordered
```

### Tests explicitly deferred

- runtime execution;
- generic call forwarding execution;
- cross-module execution.

### Acceptance

Generic-body analysis publishes canonical assumed trait dispatch sites while no-bound, structural-coincidence, and ambiguity behavior remain correct.

### Local STOP / CONSULT triggers

- assumed dispatch requires turning `TraitDispatchIndex` into a generic proof authority;
- member signature instantiation requires P2 generic associated projections;
- exact and assumed dispatch cannot coexist without changing exact identity;
- ambiguity semantics conflict with accepted C4 behavior.

### Checkpoint update

Record assumed dispatch semantic product and trait-evidenced ownership.

---

## T5 — Discharge generic call conformance obligations and forward assumed evidence

### Purpose

Extend generic callable application so direct conformance obligations are checked after ordinary type inference and semantic specialization records whether each obligation is satisfied by exact evidence or caller-assumed evidence.

### Preconditions

- T2 proof API exists.
- T3 evidence environment/layout exists.
- T4 can publish assumed trait sites.
- existing generic inference/subtype/equivalence behavior is green.

### Consumes

```text
GenericSignature.constraints
generic application inference results
CallSpecialization
GenericEvidenceEnvironment
GenericConformanceProof
ConformanceEvidence
callee generic evidence layout
```

### Produces

1. post-inference conformance discharge;
2. direct-bound diagnostics/outcomes;
3. generic call conformance bindings;
4. exact evidence source records;
5. forwarded ambient evidence source records;
6. generic-to-generic composition.

### Files and symbols

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/db/fingerprint.rs
```

Tests:

```text
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-semantic/tests/semantic/capabilities/constraints.rs
```

### Required implementation shape

#### T5.A — Preserve current inference relations

Refactor generic constraint handling explicitly:

```text
Subtype / Equivalent
    → existing inference relation path

Conforms
    → retained post-solve proof obligation
```

Do not force conformance into unification to satisfy enum exhaustiveness.

#### T5.B — Instantiate after candidate substitution

After type solving:

1. substitute each direct conformance subject and ordinary trait argument;
2. build canonical instantiated conformance keys;
3. invoke direct proof under caller evidence;
4. preserve proof outcome.

#### T5.C — Publish proof source

Extend `CallSpecialization` or an adjacent canonical call product with conformance bindings.

Each callee direct bound records:

```text
Exact:
    exact ConformanceEvidence

Forwarded:
    caller GenericConformanceKey / evidence source
```

Compiler lowering must not decide which applies.

#### T5.D — Generic forwarding

Mandatory:

```phalcom
class Forwarder {
  inner<T>(_ value: T) -> String
      where T |= Printable
  {
      value.print
  }

  outer<T>(_ value: T) -> String
      where T |= Printable
  {
      self.inner(value)
  }
}
```

The `inner` call records forwarded evidence, not missing exact evidence.

#### T5.E — Exact cases

An exact enum-case inferred subject remains exact; do not widen to enum root.

#### T5.F — Dynamic/blocked

`Dynamic` does not satisfy the bound. Underconstrained subjects remain blocked/underdetermined rather than falsely nonconforming.

#### T5.G — Failure isolation

A failed bound application must not contaminate lexical evidence/inference state for subsequent independent calls.

### Forbidden approaches

- no conformance-as-unification relation;
- no "method exists" proof;
- no compiler proof;
- no exact re-proof of abstract caller `T`;
- no exact-case widening;
- no `Dynamic` success;
- no lossy bool outcome.

### Test changes required

Cover:

```text
CV-13 CV-14 CV-15 CV-16 CV-22 CV-31 CV-35 CV-36
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::conforming_concrete_argument_discharges_direct_bound
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::bounded_generic_caller_forwards_evidence_to_bounded_callee
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::dynamic_does_not_discharge_static_conformance_bound
cargo test -p phalcom-semantic --test semantic capabilities::constraints::source_equivalence_and_lower_bound_constraints_control_calls
```

### Tests explicitly deferred

- compiler/VM execution;
- metadata/cross-module;
- broad semantic suite.

### Acceptance

The semantic call product contains canonical type specialization plus complete direct conformance discharge information, with exact/forwarded sources distinguished and existing inference semantics preserved.

### Local STOP / CONSULT triggers

- direct conformance must feed new type equalities before P2;
- canonical call specialization cannot carry evidence without a competing call authority;
- owner-level generic discharge requires new runtime reification/per-object evidence;
- exact-case identity must be weakened.

### Checkpoint update

Record generic application staging and exact-versus-forwarded evidence binding.


---

## T6 — Lower generic evidence through compiler call entry and existing runtime conformance environments

### Purpose

Project semantic direct-bound evidence into deterministic executable call-entry recipes, execute generic-body trait requirements with the existing abstract requirement mechanism, and prove runtime environment reuse/rooting behavior.

### Preconditions

- T4 assumed body dispatch is published.
- T5 call specialization includes complete conformance evidence bindings.
- T3 deterministic generic evidence layouts exist.
- existing C4 trait invocation/runtime environment tests are green.

### Consumes

```text
CallSpecialization
GenericEvidenceLayout
GenericEvidenceSlotKey
AssumedTraitDispatchSelection
exact ConformanceEvidence requirement selections
TraitInvocationSpec / TraitRequirementInvocationSpec
RuntimeCallEnvironmentRecipe
RuntimeTypeEnvironment
RuntimeConformanceEnvironment
InvokeTraitRequirement
detached method preparation
```

### Produces

1. backend-facing conformance slot recipes;
2. exact selected-target materialization;
3. forwarded caller-slot materialization;
4. callee `RuntimeConformanceEnvironment` construction/interning;
5. assumed body dispatch lowering to requirement slots;
6. default/explicit witness execution;
7. environment cardinality regression.

### Files and symbols

**Modify as live call paths require:**

```text
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/chunk.rs
phalcom-core/src/compiler/
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/frame.rs                         # only if call-entry plumbing requires
```

**Tests:**

```text
phalcom-core/tests/core/language/generic_conformance.rs   # create
phalcom-core/tests/core/language/mod.rs                   # register
phalcom-core/tests/core/language/associated_traits.rs     # focused C5 regression only
```

### Required implementation shape

#### T6.A — Core projection remains non-semantic

Semantic lowering receives already-discharged evidence bindings. It may translate stable semantic requirement selections into compiler-executable descriptors. It must not query the conformance index to decide whether a generic bound holds.

#### T6.B — Separate recipe lanes

Keep type and conformance recipe data logically separate.

A single call-entry recipe may aggregate both if that is the cleanest repository-native representation, but it must preserve:

```text
type bindings       → RuntimeTypeEnvironment
evidence slot plan  → RuntimeConformanceEnvironment
```

Do not merge the registries or put type arguments in conformance slots.

#### T6.C — Exact slot source

For an exact generic-call evidence binding, use exact `ConformanceEvidence` requirement selections to prepare the callable/default/witness handles required by the callee layout.

Reuse existing detached-method preparation/rooting. VM state must receive executable handles/descriptors, not semantic proof objects.

#### T6.D — Forwarded slot source

For forwarded evidence, translate the caller's semantic `GenericEvidenceSlotKey` to the caller layout's deterministic slot.

At runtime:

```text
callee slot N <- caller conformance environment slot M
```

No exact conformance lookup occurs.

#### T6.E — Generic body invocation

Lower assumed trait dispatch to its `GenericEvidenceSlotKey`, then to the deterministic slot and existing:

```text
InvokeTraitRequirement { slot, arity }
```

or live equivalent.

Do not add a second generic-bound opcode unless the existing opcode is genuinely incompatible; that requires consultation.

#### T6.F — Call-frame propagation

Every activation that needs generic evidence receives both:

```text
RuntimeTypeEnvironmentId
RuntimeConformanceEnvironmentId
```

Existing non-generic/no-evidence paths continue to use EMPTY environments as appropriate.

#### T6.G — Interner reuse

Use `RuntimeConformanceEnvironmentRegistry::intern`.

Add test-visible cardinality access only if necessary and keep it test-oriented or generally useful. Prove repeated identical-evidence invocations do not create one permanent environment per call.

#### T6.H — GC/rooting

Generic evidence method handles must be rooted through the existing registry `handles()` path. Do not add a second unrooted handle store.

### Forbidden approaches

- no VM conformance search;
- no syntax/trait-name lookup in runtime recipe;
- no semantic `TypeId`/`TraitRef` in runtime environment;
- no permanent environment per invocation when identical environment can intern;
- no requirement-only slot identity;
- no duplicate opcode without consultation;
- no ordinary class-dictionary fallback for trait-only assumed behavior;
- no per-object generic evidence arrays.

### Test changes required

Cover:

```text
CV-16 CV-17 CV-18 CV-20 CV-21 CV-22 CV-28 CV-32 CV-34
```

Required runtime discriminators:

1. explicit conformance witness executes through generic evidence;
2. selected trait default executes;
3. generic caller forwards evidence to generic callee;
4. multiple independent bounds select correct targets;
5. same trait on two generic subjects does not cross-wire slots;
6. exact enum-case evidence stays exact;
7. repeated same-evidence calls reuse environment.

### Tests to run now

After the coherent runtime slice is executable:

```sh
cargo test -p phalcom-core --test core language::generic_conformance::generic_bound_executes_explicit_conformance_witness
cargo test -p phalcom-core --test core language::generic_conformance::generic_bound_executes_selected_trait_default
cargo test -p phalcom-core --test core language::generic_conformance::generic_to_generic_call_forwards_runtime_evidence
cargo test -p phalcom-core --test core language::generic_conformance::same_trait_on_distinct_generic_subjects_uses_distinct_slots
cargo test -p phalcom-core --test core language::generic_conformance::repeated_calls_reuse_conformance_environment
```

### Tests explicitly deferred

- metadata/cross-module runtime until T7;
- broad core suite;
- production iterator tests;
- workspace verification.

### Acceptance

Direct bounded generic execution works through existing conformance-environment semantics for exact and forwarded evidence, defaults/witnesses remain correct/rooted, and repeated identical evidence does not create one permanent environment per invocation.

### Local STOP / CONSULT triggers

- `InvokeTraitRequirement` cannot represent generic layout semantics safely;
- compiler must re-run semantic trait selection;
- forwarded evidence cannot be copied without runtime trait lookup;
- call entry requires per-object evidence storage;
- GC/rooting policy must change;
- interning cannot prevent invocation-count growth without changing lifetime ownership.

### Checkpoint update

Record executable evidence recipe contract, reused opcode/environment architecture, and cardinality/rooting evidence at the first passing runtime gate.

---

## T7 — Make direct conformance bounds durable across metadata, reflection, and modules

### Purpose

Preserve direct generic conformance contracts through stable metadata/module boundaries and expose them faithfully through runtime typing/reflection without treating a trait as a nominal type.

### Preconditions

- T1 canonical direct bound exists.
- T5 semantic discharge works.
- T6 execution works in one module.
- stable metadata/type-node exporter/importer paths re-read.

### Consumes

```text
GenericSignature
GenericConstraint::Conforms
TraitRef
StableDeclarationRef
TypeNodeId
GenericConstraintRef
metadata exporter/importer/validator
runtime typing generic-constraint inspection
```

### Produces

1. stable trait application metadata value;
2. `GenericConstraintRef::Conforms`;
3. export/import/validation/fingerprint parity;
4. relation-aware reflection;
5. cross-module bounded callable type-check/execution.

### Files and symbols

```text
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/... live bundle/validation/import helpers
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/types/native.rs
phalcom-type-syntax/src/lib.rs
phalcom-native-meta/src/types.rs                 # audit/exhaustive support as required
phalcom-core/src/typing/inspect.rs               # or live reflection owner
phalcom-core/src/typing/registry.rs              # if live importer/inspection requires
```

Tests:

```text
live semantic metadata/integration test module
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-core/tests/core/language/generic_conformance.rs
relevant phalcom-type-meta tests
```

### Required implementation shape

#### T7.A — Stable trait application

Introduce a stable representation equivalent to:

```rust
pub struct StableTraitApplicationRef {
    pub declaration: StableDeclarationRef,
    pub arguments: Box<[TypeNodeId]>,
}
```

The exact stable declaration reference type must follow the live metadata identity model.

#### T7.B — Stable direct relation

Extend:

```rust
GenericConstraintRef::Conforms {
    subject: TypeNodeId,
    trait_application: StableTraitApplicationRef,
}
```

Validate referenced type nodes and stable declaration identity using existing metadata validation/import architecture. Semantic import must re-establish arity/kind/well-formedness rather than blindly trusting bytes.

#### T7.C — No associated projection metadata in P1

Do not add stable associated-projection type nodes solely for P1. P2 owns public constraints such as:

```phalcom
I::Item == T
```

#### T7.D — Reflection

Extend runtime/type-syntax relation tags to include conformance.

If current reflection exposes binary left/right accessors, preserve those for subtype/equivalence and add conformance-specific access to:

```text
subject
trait declaration/application
trait arguments
```

Do not fabricate a "right type" for a trait.

#### T7.E — Cross-module

Create a provider/consumer test:

```text
provider:
    exports bounded generic callable + relevant trait/concrete conformance

consumer:
    imports callable/concrete type
    call discharges imported bound
    runtime executes selected witness/default
```

The imported generic signature/metadata—not source re-reading—must preserve the contract.

#### T7.F — Native metadata

Audit `GenericConstraintSpec` and native generic formation.

If P1 acceptance does not require native declarations to author conformance bounds, update exhaustive handling and define explicit unsupported behavior rather than coercing a direct conformance relation into subtype/equivalence.

### Forbidden approaches

- no semantic `TypeId` serialization;
- no semantic `TraitRef` serialization;
- no trait-as-nominal-type hack;
- no dropping public bounds at export;
- no binary reflection lie;
- no raw generic associated projection metadata;
- no source-text-based cross-module proof.

### Test changes required

Cover:

```text
CV-23 CV-24 CV-25 CV-33
```

### Tests to run now

Use live metadata test names discovered before editing, plus:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::stable_metadata_preserves_direct_conformance_constraint
cargo test -p phalcom-core --test core language::generic_conformance::cross_module_bounded_generic_call_executes_selected_evidence
```

Run a focused type-meta target if the live crate has a direct metadata test target.

### Tests explicitly deferred

- associated projection metadata;
- broad runtime typing suite;
- workspace tests.

### Acceptance

A public direct conformance bound round-trips through stable metadata, imported bounded generic calls type-check/execute from durable semantic products, and reflection exposes conformance as its own relation.

### Local STOP / CONSULT triggers

- no stable declaration identity can represent a trait without new metadata identity architecture;
- importer requires snapshot IDs;
- accepted P1 direct bounds require associated-projection metadata;
- reflection requires a broad breaking redesign rather than bounded additive extension.

### Checkpoint update

Record stable direct conformance metadata shape and cross-module evidence.

---

## T8 — Close incremental, source-index, LSP, and semantic identity integration

### Purpose

Make direct bounds incrementally correct and editor-visible through canonical semantic identity, correct inherited associated-type `::` presentation, and prove source reorder/regrouping is a semantic no-op.

### Preconditions

- T1/T3 canonical identity exists.
- T7 public metadata exists.
- current incremental/source-index/LSP tests re-read after relocation.

### Consumes

```text
GenericSignature product fingerprints
GenericEvidenceEnvironment/layout
source-index where-clause walker
canonical trait declaration identity
LSP semantic snapshot consumers
```

### Produces

1. owner-complete invalidation for bound changes;
2. semantic no-op behavior for reorder/regrouping;
3. cold/incremental parity;
4. canonical trait definition/navigation from `where`;
5. corrected associated hover notation.

### Files and symbols

```text
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/editor.rs                  # if live navigation requires
phalcom-semantic/src/session.rs                 # only if dependency publication requires
phalcom-lsp/src/backend.rs
phalcom-lsp/src/semantic_tokens.rs              # if token classification requires
```

Tests:

```text
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs
live semantic integration source-index/editor test module
live phalcom-lsp hover/navigation tests
```

### Required implementation shape

#### T8.A — Change invalidation

Adding/removing/changing a bound must invalidate:

```text
owning GenericSignature
body analysis/evidence consuming it
generic applications discharging it
public metadata/interface fingerprint
tooling/source-index products presenting it
```

Do not broadly invalidate unrelated trait surfaces/conformances.

#### T8.B — Semantic no-op reorder

Changing:

```phalcom
where T |= A + B
```

to:

```phalcom
where T |= B + A
```

or equivalent comma grouping must preserve semantic generic-contract fingerprint and evidence layout.

Input/source fingerprints may update because text changed. Downstream semantic meaning should remain reusable.

#### T8.C — Cold/incremental parity

After mutations:

```text
incremental result == fresh cold analysis
```

for statuses, canonical constraints, bound availability, and diagnostics.

#### T8.D — Source index

The trait identifier in:

```phalcom
where T |= Printable
```

must map to canonical trait declaration identity. LSP must consume that; no text lookup.

#### T8.E — Associated hover correction

Change live display:

```text
Cell.Item
```

to:

```text
Cell::Item
```

and update the relocated descriptive test. This is presentation only; do not reopen C5 associated identity.

#### T8.F — Semantic token

If the new lexical relation needs LSP token classification, classify `|=` as an operator via the existing operator path.

### Forbidden approaches

- no `invalidate_all`;
- no source spelling as semantic identity;
- no LSP text resolver;
- no `.` associated notation;
- no semantic rebuild solely for `A+B` → `B+A`.

### Test changes required

Cover:

```text
CV-08 CV-26 CV-27 CV-29 CV-30
```

### Tests to run now

Use exact live test names; representative commands:

```sh
cargo test -p phalcom-semantic --test semantic incremental::fingerprints::generic_conformance_bound_reorder_preserves_semantic_fingerprint
cargo test -p phalcom-semantic --test semantic incremental::recomputation_metrics::generic_conformance_change_invalidates_only_affected_products
cargo test -p phalcom-semantic --test semantic integration::generic_conformance_source_index
cargo test -p phalcom-lsp associated_type_hover_uses_colon_colon
cargo test -p phalcom-lsp generic_conformance_bound_go_to_definition_uses_trait_identity
```

### Tests explicitly deferred

- broad LSP JSON-RPC suite;
- broad incremental suite;
- workspace tests.

### Acceptance

Bound changes invalidate correct consumers, reorder/regrouping preserves semantic identity, cold/incremental products match, bound navigation uses canonical trait identity, and associated hover renders `::`.

### Local STOP / CONSULT triggers

- semantic product fingerprint cannot separate from source shape without broader incremental redesign;
- source index lacks canonical trait target ability and would require text fallback;
- correcting `::` exposes a normative disagreement rather than presentation defect.

### Checkpoint update

Record incremental identity law, source-index/tooling integration, and presentation correction.

---

## T9 — Stabilize the P1 vertical, classify baselines, and publish durable records

### Purpose

Prove the complete bounded P1 objective through focused semantic/compiler/runtime/module/incremental/tooling evidence, classify baselines, and leave C6 ready for P2.

### Preconditions

- T1–T8 complete.
- no unresolved A/B failures.
- consultations/amendments recorded.

### Consumes

All P1 products and coverage obligations.

### Produces

1. final focused verification mapping;
2. runtime resource/cardinality evidence;
3. baseline classification;
4. checkpoint ledger update;
5. P1 walkthrough;
6. P1-to-P2 handoff.

### Files and symbols

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-handoff.md
```

No new feature code should be introduced in T9 except focused corrections required by failed P1 acceptance evidence.

### Required implementation shape

#### T9.A — Coverage ledger

Map every `CV-*` to:

```text
test name
command/target
PASS/FAIL
what the result proves
```

#### T9.B — Baseline classification

Re-evaluate C5-BL-01/04 only through the new bounded generic call path. Do not repair unrelated iterator behavior.

#### T9.C — Resource evidence

Record:

```text
initial runtime conformance environment count
distinct evidence layouts/selections exercised
number of repeated invocations
final count
interpretation
```

Growth must track genuinely distinct environments, not call count.

#### T9.D — Verification truthfulness

Expected normal terminal state after focused success:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

Do not claim `RELEASE_COMPLETE` without release/workspace certification.

### Forbidden approaches

- no broad "fix whatever fails" sweep;
- no weakened assertions;
- no silently skipped coverage;
- no release certification claim from focused tests;
- no P2 implementation during P1 stabilization.

### Tests to run now

Run the named gates in §19, not an ad-hoc maximal suite.

### Acceptance

All P1 coverage maps to passing focused evidence or an explicit approved amendment; no P1-owned blockers remain; resource behavior is proven; checkpoint/walkthrough/handoff are accurate.

### Local STOP / CONSULT triggers

- final acceptance exposes an unplanned architectural gap;
- only unrelated broad repair can make a focused P1 test pass;
- required coverage can pass only by weakening a fixed invariant.

### Checkpoint update

Perform final P1 bookkeeping and set next action to C6.P2, not P2 code.

---

# 19. Verification gates

## G1 — Canonical generic contract gate

**Purpose:** Prove source `|=` and canonical direct obligations exist without regressing current generic relations.

Run:

```sh
cargo test -p phalcom-ast --test lexer lexes_conformance_constraint_operator
cargo test -p phalcom-ast --test parser parses_direct_trait_conformance_constraint
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
cargo test -p phalcom-semantic --test semantic capabilities::constraints
```

Expected:

```text
direct bounds parse/form
+ expands canonically
duplicates/order normalize
where-kind behavior matches accepted P1 decision
existing subtype/equivalence stays green
```

Do not broaden after PASS.

---

## G2 — Semantic evidence and composition gate

**Purpose:** Prove lexical assumptions, generic-body dispatch, direct call discharge, and forwarding before runtime integration.

Run:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
```

If exact trait resolver code changed materially, add only affected focused `impls::queries` tests.

Expected discriminators:

```text
bound-provided method success
no-bound failure
structural coincidence failure
owner+callable composition
exact concrete discharge
nonconforming failure
Dynamic failure
generic-to-generic forwarding
multiple independent bounds
same-trait/different-subject slot distinction
ambiguity
exact enum case
terminal proof-state preservation
```

---

## G3 — Compiler/runtime evidence transport gate

**Purpose:** Prove evidence reaches executable runtime handles without runtime proof.

Run:

```sh
cargo test -p phalcom-core --test core language::generic_conformance
```

Expected:

```text
explicit witness executes
trait default executes
generic forwarding executes
multiple evidence slots dispatch correctly
same trait/different subject does not cross-wire
exact enum-case evidence stays exact
repeated calls reuse environment
```

If shared C4/C5 trait runtime code changed materially, run focused affected regressions in `language::associated_traits`.

---

## G4 — Durable metadata and cross-module gate

**Purpose:** Prove a direct bound is a durable public contract, not checker-local state.

Run live focused metadata/integration tests plus:

```sh
cargo test -p phalcom-core --test core language::generic_conformance::cross_module_bounded_generic_call_executes_selected_evidence
```

Expected:

```text
GenericConstraintRef::Conforms round-trips
stable trait application survives
imported signature retains bound
consumer discharges it
runtime executes correct target
reflection reports conformance relation truthfully
```

---

## G5 — Incremental and tooling gate

**Purpose:** Prove semantic identity/invalidation/editor behavior.

Run exact live tests covering:

```text
bound add/remove/change
bound reorder/regroup no-op
cold/incremental parity
source-index definition target
LSP bound navigation
associated :: hover
```

Representative:

```sh
cargo test -p phalcom-semantic --test semantic incremental::fingerprints
cargo test -p phalcom-semantic --test semantic incremental::recomputation_metrics
cargo test -p phalcom-lsp associated_type_hover_uses_colon_colon
cargo test -p phalcom-lsp generic_conformance_bound_go_to_definition_uses_trait_identity
```

Use narrower filters where available.

---

## G6 — Final focused P1 stabilization gate

Mandatory:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
cargo test -p phalcom-semantic --test semantic capabilities::constraints
cargo test -p phalcom-core --test core language::generic_conformance
```

Additionally run exact metadata/incremental/LSP tests modified by P1 and directly affected C4/C5 regression modules where shared code justifies it.

Expected:

```text
all P1 coverage obligations have evidence
no unresolved A/B failures
baselines classified
no environment growth per invocation
touched C5 exact behavior remains green
```

Do not automatically run workspace tests or workspace clippy.

---

## 20. Final focused acceptance

P1 may be marked `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only when the walkthrough maps every coverage ID to actual evidence.

| Coverage ID | Required final evidence |
|---|---|
| CV-01 | dedicated `|=` lexer test; ordinary `|` preserved |
| CV-02 | direct bound AST test |
| CV-03 | conjunction canonicalization |
| CV-04 | generic trait argument preservation |
| CV-05 | one kind authority or explicit approved amendment |
| CV-06 | general subject formation |
| CV-07 | duplicate idempotence |
| CV-08 | reorder/sugar semantic equivalence |
| CV-09 | body assumed trait behavior |
| CV-10 | missing bound negative |
| CV-11 | structural coincidence negative |
| CV-12 | owner/callable lexical composition |
| CV-13 | exact concrete discharge |
| CV-14 | missing concrete evidence diagnostic |
| CV-15 | Dynamic not proof |
| CV-16 | generic forwarding |
| CV-17 | multiple independent bound slots |
| CV-18 | same trait / distinct subject slots |
| CV-19 | ambiguity |
| CV-20 | default execution |
| CV-21 | explicit witness execution |
| CV-22 | exact enum-case evidence |
| CV-23 | stable metadata |
| CV-24 | cross-module contract/execution |
| CV-25 | relation-aware reflection |
| CV-26 | incremental semantic change |
| CV-27 | incremental semantic no-op reorder |
| CV-28 | runtime interner/cardinality |
| CV-29 | canonical navigation |
| CV-30 | associated `::` hover |
| CV-31 | subtype/equivalence regression |
| CV-32 | exact trait regression |
| CV-33 | publication validation |
| CV-34 | checked slot capacity |
| CV-35 | terminal proof states |
| CV-36 | failed-call isolation |

If a coverage item changes through an approved amendment, record the consultation/amendment explicitly rather than silently deleting it.

---

## 21. Performance/resource evidence

P1 is not a benchmark plan, but it has concrete resource requirements.

### 21.1 Assumed evidence lookup

Acceptable evidence:

- implementation inspection showing direct assumption/member lookup uses the environment's canonical index rather than a full constraint scan per send;
- focused multiple-bound semantic tests.

No benchmark required.

### 21.2 Repeated exact proof

Within one generic application/proof context, identical direct obligation proof should reuse existing exact semantic query/evidence reuse where available.

Do not add a second memoization cache if current snapshot/query architecture already provides reuse. The walkthrough must state which authority provides it.

### 21.3 Runtime environment cardinality

Mandatory structural regression:

```text
environment count before
repeat same bounded generic specialization many times
environment count after
```

Expected growth is bounded by genuinely distinct evidence layouts/selections, not invocation count.

### 21.4 GC/rooting

Generic-bound executable handles must remain reachable through the existing runtime conformance registry rooting path. No new root family is expected.

---

## 22. Checkpoint bookkeeping

### At plan start

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Inspect `docs/implementation/LANG005/`.

At planning revision no C6 directory exists. If still absent, initialize:

```text
docs/implementation/LANG005/LANG005.C6/
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
```

Recommended checkpoint front matter:

```yaml
---
id: LANG005.C6
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
requires:
  - LANG005.C5
---
```

The record owns:

- bounded C6 objective;
- ownership/non-goals;
- plans ledger with P1 active;
- inherited C5 invariants;
- stable takeover map;
- baseline ledger;
- verification ledger;
- consultations/amendments;
- active plan/next action.

If a C6 record already exists, update it. Never create a competing state file.

### During plan

Update only for durable events:

- canonical direct relation established;
- TraitRef identity/proof split established;
- evidence environment/layout established;
- proof/call specialization interface established;
- runtime evidence transport gate passes;
- stable metadata contract established;
- consultation/amendment;
- meaningful baseline discovery/reclassification;
- coherent verification gate.

### At plan completion

Record:

```text
P1 status/completion/verification
final P1 invariants/interfaces
verification ledger
performance/resource evidence
baseline/deferred issues
consultations/amendments
next action: C6.P2
```

Expected normal state after focused success:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-walkthrough.md
```

It must contain:

- final lifecycle result;
- planning and final revision;
- architecture actually implemented;
- canonical `Conforms` representation;
- where-kind reconciliation result;
- TraitRef identity/proof split;
- GenericEvidenceEnvironment representation/composition;
- proof outcome/evidence source model;
- assumed trait dispatch product;
- call specialization evidence representation;
- evidence slot identity/layout;
- runtime call-entry recipe;
- reuse of `InvokeTraitRequirement`;
- stable metadata/reflection shape;
- incremental/tooling integration;
- important code changes by subsystem;
- deviations;
- consultation decisions;
- tests added;
- tests actually run;
- tests deferred;
- complete CV result mapping;
- runtime environment cardinality evidence;
- baseline classification;
- verification classification;
- residual risks and P2 prerequisites.

The walkthrough records what happened; it must not duplicate this plan.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P1-handoff.md
```

It must contain:

```text
current revision
P1 lifecycle state
canonical GenericConstraint::Conforms shape
GenericConformanceKey identity
GenericEvidenceEnvironment API/ownership
proof API + exact/assumed outcomes
TraitRef identity/proof APIs
assumed trait dispatch product
CallSpecialization evidence fields/product
GenericEvidenceSlotKey/layout
core/runtime evidence recipe contract
stable GenericConstraintRef::Conforms shape
reflection surface
incremental identity rules
source-index/LSP state
performance/cardinality evidence
deferred/baseline failures
must-read files
first recommended P2 commands
things P2 must not redesign
```

The next objective is P2:

```text
generic T::Item
qualified <T as Trait>::Item
associated specialization Trait<Item = T>
general projection/equivalence constraints
associated-evidence feedback into inference
stable metadata representation for generic associated projections
```

P2 consumes P1's evidence environment/proof APIs; it must not introduce another generic evidence mechanism.

---

## 25. Completion truth table

| State | Meaning |
|---|---|
| source written | code edits exist |
| tests added | feature/regression tests exist |
| targeted tests passed | exact discriminators passed |
| P1 focused gates passed | bounded P1 acceptance evidence established |
| P1 implemented | intended P1 feature exists |
| P1 complete | lifecycle records and required focused evidence complete |
| C6 accepted | all required C6 plans satisfy checkpoint objective |
| release certified | named broader release/workspace gates passed |

P1 completion does not imply C6 completion.

---

## 26. Plan self-review

### Architecture

- [x] one generic-contract authority;
- [x] general conformance subject;
- [x] trait identity and proof separated;
- [x] exact and assumed evidence distinct;
- [x] no fake source conformance;
- [x] generic behavior resolves through trait requirements;
- [x] generic forwarding first-class;
- [x] runtime type/conformance environments separate;
- [x] no VM proof/search;
- [x] slot identity includes conformance + requirement;
- [x] source-order-independent layout;
- [x] stable metadata uses stable identities;
- [x] incremental semantic identity separated from source presentation;
- [x] P2/P3 growth seams preserved.

### Luna executability

- [x] FIXED/FLEXIBLE/VERIFY-FIRST explicit;
- [x] coherent tasks with preconditions/inputs/outputs/files/shape/forbidden approaches/tests/acceptance/escalation;
- [x] objective STOP/CONSULT triggers;
- [x] bounded semantic debugging.

### Testing

- [x] success, negative, structural, ambiguity, composition, exact case, cross-module, metadata, incremental, repetition, tooling, and regressions covered;
- [x] tests tied to invariants;
- [x] BUILD breadth deliberately narrow;
- [x] workspace/release tests explicitly deferred.

### Lifecycle/scope

- [x] C6 checkpoint bootstrap specified because absent at planning revision;
- [x] walkthrough/handoff required;
- [x] lifecycle truth table explicit;
- [x] P1 does not claim checkpoint closure;
- [x] P2/P3/P4/P5 concerns remain deferred;
- [x] no compiler-only Iterator special case.

### No-placeholder review

This plan contains no implementation placeholders. VERIFY-FIRST entries identify concrete facts to inspect and state the response to mechanical versus architectural drift.

---

# Appendix A — Planned semantic product sketches

These are architectural contracts, not mandatory exact Rust spelling.

```rust
pub enum GenericConstraint {
    Subtype { lower: TypeTerm, upper: TypeTerm },
    Equivalent { left: TypeTerm, right: TypeTerm },
    Conforms { subject: TypeTerm, trait_ref: TraitRef },
}
```

```rust
pub struct GenericConformanceKey {
    pub subject: TypeTerm,
    pub trait_ref: TraitRef,
}
```

```rust
pub struct AssumedConformanceEvidence {
    pub key: GenericConformanceKey,
    pub origin: GenericEvidenceOrigin,
}
```

```rust
pub enum GenericConformanceEvidence {
    Exact(Arc<ConformanceEvidence>),
    Assumed(AssumedConformanceEvidence),
}
```

```rust
pub enum GenericConformanceProof {
    Proven(GenericConformanceEvidence),
    Missing,
    Ambiguous(/* canonical candidates/provenance */),
    Incomplete(/* existing exact state */),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}
```

```rust
pub struct GenericEvidenceSlotKey {
    pub conformance: GenericConformanceKey,
    pub requirement: TraitRequirementId,
}
```

```rust
pub struct GenericEvidenceLayout {
    pub slots: Box<[(GenericEvidenceSlotKey, u16)]>,
}
```

```rust
pub struct AssumedTraitDispatchSelection {
    pub conformance: GenericConformanceKey,
    pub requirement: TraitRequirementId,
    pub signature: CallableSemanticSignature,
}
```

```rust
pub enum TraitDispatchSite {
    AbstractRequirement { requirement: TraitRequirementId },
    Evidenced(TraitDispatchSelection),
    Assumed(AssumedTraitDispatchSelection),
}
```

```rust
pub enum GenericCallConformanceSource {
    Exact { evidence: Arc<ConformanceEvidence> },
    Forwarded { caller_conformance: GenericConformanceKey },
}

pub struct GenericCallConformanceBinding {
    pub callee_conformance: GenericConformanceKey,
    pub source: GenericCallConformanceSource,
}

pub struct CallSpecialization {
    pub callable: CallableId,
    pub bindings: Box<[(TypeParameterId, TypeId)]>,
    pub conformances: Box<[GenericCallConformanceBinding]>,
}
```

Final implementation may use equivalent repository-native types, but it must preserve the information/ownership above.

---

# Appendix B — Evidence flow examples

## B.1 Concrete caller

```phalcom
class Renderer {
  @class
  render<T>(_ value: T) -> String
      where T |= Printable
  {
      value.print
  }
}

Renderer.render(User.new())
```

```text
infer T = User
    ↓
instantiate Conforms(User, Printable)
    ↓
exact proof
    ↓
ConformanceEvidence(User, Printable)
    ↓
CallSpecialization: callee bound -> Exact(evidence)
    ↓
compiler prepares requirement targets
    ↓
runtime callee conformance environment
    ↓
generic body InvokeTraitRequirement(print-slot)
```

## B.2 Generic forwarding

```phalcom
class Forwarder {
  inner<T>(_ value: T) -> String
      where T |= Printable
  {
      value.print
  }

  outer<T>(_ value: T) -> String
      where T |= Printable
  {
      self.inner(value)
  }
}
```

```text
outer lexical environment:
    Assumed Conforms(T, Printable)
    ↓
inner substitution:
    inner.U = outer.T
    ↓
instantiate inner bound:
    Conforms(T, Printable)
    ↓
proof finds enclosing assumption
    ↓
CallSpecialization:
    inner bound -> Forwarded(outer conformance key)
    ↓
lowering:
    inner slot N <- outer slot M
    ↓
runtime copies handle
```

No exact source `impl` for abstract `T` is searched or invented.

## B.3 Same trait, different subjects

```phalcom
class Combiner {
  both<T, U>(_ left: T, _ right: U)
      where
          T |= Printable,
          U |= Printable
  {
      left.print
      right.print
  }
}
```

Correct identity:

```text
slot 0:
    (Conforms(T, Printable), Printable.print)

slot 1:
    (Conforms(U, Printable), Printable.print)
```

Forbidden aliasing:

```text
Printable.print -> slot 0
```

---

# Appendix C — P1 boundary versus later C6

| Capability | P1 | Later owner |
|---|---:|---|
| `T |= Trait` | yes | — |
| `T |= A + B` | yes | — |
| ordinary generic trait args | yes | — |
| `where F: Kind` | yes if additive VERIFY-FIRST succeeds | bounded follow-up if broader migration |
| lexical assumed evidence | yes | extended later |
| exact + assumed direct proof | yes | extended P3 |
| generic-body trait dispatch | yes | — |
| generic-to-generic forwarding | yes | — |
| stable direct-bound metadata | yes | — |
| `T::Item` | no | P2 |
| `<T as Trait>::Item` | no | P2 |
| `Trait<Item = T>` | no | P2 |
| associated-driven inference | no | P2 |
| conditional conformance | no | P3 |
| recursive/nested proof | no | P3 |
| cycles/memoized worklist | no | P3 |
| conditional overlap implementation | no | P3 |
| blanket/derived implication | no | P4 |
| `Iterator => Iterable` | no | P4 |
| Universe iterator generic laws | no | P4 |
| production `for` migration | no | P5 |
| collection/Map/Range/lazy migration | no | P5 |
| cursor retirement | no | P5 |
| alias/GADT full constraint closure | no | P6 |
| higher-kinded trait subject closure | no | P6/spec dependency |
| final checkpoint stress/certification | no | later C6 closure |

---

# Appendix D — Future design issues P1 must not settle accidentally

## D.1 Conditional-conformance overlap

Current Phalcom rejects overlapping conformance heads and has no "most specific wins" specialization. P1 preserves that state. P3 must verify authoritative coherence before conditional heads are activated.

## D.2 `Iterator => Iterable`

Generic constraints do not themselves create the theorem "every Iterator is Iterable". Supertraits are not part of P1, and a generalized blanket conformance is broader than current nominal conformance heads. No compiler-only Iterator special case is allowed.

## D.3 Higher-kinded trait subjects

The generic relation can represent `F |= Functor`, but P1 does not invent how a trait declares subject kind `(Type) -> Type`.

## D.4 Generic associated projection metadata

P1 can stably encode `Conforms(T, Printable)` because the subject/ordinary trait arguments use stable type nodes. P2 must explicitly solve durable representation for `I::Item == T`; C5's executable-metadata rejection of raw projections cannot simply be generalized to public generic constraints.

---

# Appendix E — Baseline quick reference

```text
C5-BL-01 / C5-BL-04
    old Range / Option.match generic call-entry issue

    if direct P1 generic evidence call reproduces same root defect:
        P1-owned blocker

    otherwise:
        retain baseline for later iteration/runtime maintenance

C5-BL-07
    Universe Bool capability/dependency failures

    default:
        D — unrelated baseline

    reclassify only if P1 changes intersect directly
```

Do not use C6 activation as a reason to repair unrelated legacy iteration.
