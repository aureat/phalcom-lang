---
id: LANG007.C3.P2
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG007.C3.P1
  - LANG007.C2
follows: LANG007.C3.P1
supersedes: null
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 60a27f79d69aa3e79c22e3d19df221466c7db920
prepared: 2026-09-22
title: "Generalized Static Domains, Heterogeneous Application, and Associated Type Data"
---

# LANG007.C3.P2 — Generalized Static Domains, Heterogeneous Application, and Associated Type Data

## 0. Executor contract

This is a Luna-oriented patch-grade implementation plan for the second implementation
plan of `LANG007.C3`.

It converts the accepted C3 architecture and the repository-grounded
`LANG007.C3.P2-requirements-analysis.md` into one bounded executable work package.

The implementer may adapt private helper names, local decomposition, imports,
borrow-checker mechanics, test fixture placement, and other mechanical details to the
live repository. The implementer must not silently change the architecture, canonical
identity, domain model, ownership model, runtime/static boundary, trait boundary,
metadata identity rules, or accepted language behavior fixed by this plan.

The operating rules are:

- verify live paths and signatures before editing;
- consume the completed P1 walkthrough/handoff before rediscovering P1;
- preserve unrelated modified, staged, and untracked work;
- keep `TypeStore`/semantic products as the formal static-semantic authority;
- keep `ProperTypeId` as the proof boundary for actual runtime/proper types;
- keep `RecordRow` in its accepted dedicated canonical and inference domain;
- use `KindId` as the one generic/static-domain classifier;
- use `StaticArgument` as the one heterogeneous application envelope;
- preserve the mature C1/C2 pack/row implementation;
- preserve the mature runtime ADT/GADT/exact-case implementation;
- do not infer static identity by executing runtime Phalcom behavior;
- use sparse, discriminating tests during BUILD mode;
- classify and defer unrelated/baseline failures;
- STOP AND CONSULT when a trigger in this plan fires;
- keep the shared `LANG007.C3` checkpoint current;
- create the required P2 walkthrough and P2→P3 handoff.

Testing is evidence gathering, not a ritual. Do not run broad suites merely because an
edit occurred.

### 0.1 Hard predecessor rule

This plan was prepared while P1 was still partially implemented at
`60a27f79d69aa3e79c22e3d19df221466c7db920`.

**Do not execute P2 implementation against that partial revision.**

P2 starts only after P1 truthfully establishes its own acceptance objective, including:

```text
the repository builds at the focused P1 baseline
TypeOnly declarations are genuine static domains, not proper Types
top-level type-data terms are canonical compiler static terms
static-domain/static-constructor identity is canonical
TypeOnly cases do not publish runtime invocation semantics
ProperTypeId rejects non-Type static terms
P1 metadata/declaration phase handling is no longer semantically false
P1 focused verification is recorded
P1 walkthrough and P1→P2 handoff exist
```

If any one of those statements is false at execution time, **stop P2 immediately**.
The missing work belongs to `LANG007.C3.P1` corrective closure. Do not absorb it into
P2 merely to make the current branch progress.

---

## 1. Goal

Make arbitrary supported static domains first-class through generic binder
classification, heterogeneous application, checked substitution, scoped type
functions, first-order inference, static aliases, type members, associated `type data`,
stable metadata/imports, and incremental/source identity—without weakening the
proper-type boundary or introducing runtime static evaluation.

---

## 2. Checkpoint acceptance objective

`LANG007.C3` owns the unified algebraic/static-domain model:

```text
data
    runtime algebraic domain

type data
    type-system-only algebraic/static domain
```

P2 is the semantic generalization plan between:

```text
P1
    unified algebraic declarations + top-level type-data foundation

P2
    arbitrary static-domain application/inference/associated domains

P3
    intrinsic structural-domain surface + Universe/corpus/tooling migration
    + checkpoint certification
```

P2 is not checkpoint closure.

P2 is successful when all of the following are true:

1. generic binders can be classified by ordinary `Type`, primitive compile-time
   domains such as `Int`, user-defined `type data` domains, the existing pack
   domains, and `RecordRow`;
2. generic application stores canonical heterogeneous arguments rather than assuming
   every argument is a proper/type-level `TypeId`;
3. `TypeData::Applied` can contain `StaticArgument::Term(TypeId)` and
   `StaticArgument::RecordRow(RecordRowId)` without collapsing rows into TypeIds;
4. existing type-only application remains a convenience projection of the generalized
   application path;
5. type lambdas/scoped terms can preserve heterogeneous application and static-data
   terms containing bound variables;
6. checked substitution/materialization recursively rebuilds heterogeneous static terms
   through canonical constructors and never silently keeps an invalid pre-substitution
   term;
7. `InferVarId` can solve arbitrary non-row static domains using its existing `KindId`
   classifier, including structural first-order unification of type-data constructor
   terms;
8. arbitrary static domains do not acquire a general subtype lattice or evaluator;
9. static aliases/definitions can declare a result domain such as `Shape` or
   `TypePack`;
10. type-member domains use the same canonical domain/term model rather than a
    contradictory permanent `ConstInt` subsystem;
11. associated `type data` is supported for the accepted declaration owners with stable
    declaration identity and canonical generic-owner specialization;
12. `AssociatedSurface` remains the one `::` namespace/collision authority across
    runtime cases/constructors, associated static domains, static terms/type members,
    and intrinsic type members;
13. unsupported trait generic parameters over arbitrary user static domains are
    rejected explicitly rather than accidentally accepted by type-only trait evidence
    machinery;
14. stable metadata/imports round-trip static-domain identity, heterogeneous applied
    arguments, static-data terms, static aliases, and associated-domain
    specializations;
15. incremental results equal cold results for static-domain declaration, case,
    application, alias, and associated-domain edits;
16. source indexing can attach static-domain and associated paths to canonical semantic
    identity;
17. there is no VM static solver, runtime type-data object model, or runtime
    user-method execution for static normalization.

P3 remains responsible for final Universe structural-domain exposure, broad canonical
source/corpus migration, final legacy terminology cleanup, full tooling polish, and
checkpoint certification.

---

## 3. Repository grounding

Prepared against the latest pushed partial-P1 revision available during planning:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   60a27f79d69aa3e79c22e3d19df221466c7db920
subject:    feat(semantic): type data foundation
parent:     62a6082ab463387c812d558e5561508267ecc63c
date:       2026-09-22
```

### 3.1 Execution baseline

The actual P2 execution revision **must be the completed P1 revision**, not the revision
above.

At plan start record:

```text
P1 completion revision:
P1 verification:
P1 walkthrough:
P1 handoff:
```

If those are unavailable, this plan is not executable yet.

### 3.2 Verified repository facts relevant to P2

The relevant crates remain:

```text
phalcom-ast/
phalcom-modules/
phalcom-semantic/
phalcom-type-meta/
phalcom-core/
phalcom-lsp/
docs/
```

P2 must preserve the C1/C2 substrate:

```text
TypeId / ProperTypeId
KindId / KindData
RecordRowId and row arena
row inference
TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks
GenericParameterMode
GenericBinderHeader
whole-pack InferVarId inference
checked C2 pack normalization
stable scoped type-lambda metadata
```

The partial-P1 tree already introduced the intended P2 seams:

```rust
StaticDomainId
KindData::StaticDomain(...)
TypeData::ConstInt(...)
TypeData::StaticData(...)
StaticArgument::{Term, RecordRow}
```

P2 must use the completed P1 forms rather than inventing parallel replacements.

The pre-P2 `TypeData::Applied` representation is type-only:

```rust
TypeData::Applied {
    origin: TypeId,
    arguments: Box<[TypeId]>,
}
```

The pre-P2 generic argument AST is also type-form-oriented:

```rust
GenericArgumentSyntax::Single(TypeAnnotation)
GenericArgumentSyntax::Expansion(...)
```

The pre-P2 generic binder domain grammar is represented by `KindSyntax`, whose
closed parser recognizes the built-in kind/domain names rather than arbitrary nominal
static-domain declarations.

`InferenceVariable` already carries:

```rust
kind: KindId
```

and therefore does not require a second static-domain inference-variable identity.

`GenericInstantiation` already separates TypeId and RecordRow bindings, which is an
important architecture precedent for the `StaticArgument` envelope.

The type-member subsystem already contains a bounded heterogeneous seam:

```rust
TypeMemberDomain::{Kind(...), ConstInt, Type(...)}
TypeMemberArgument::{Type(...), Const(...), RecordRow(...)}
```

P2 must converge this subsystem onto the canonical static-domain model rather than
preserve it as a second permanent term universe.

The associated namespace currently centers on:

```text
AssociatedSurface
AssociatedFamilyKind
AssociatedMemberId
AssociatedResolution
AssociatedOwnerResolution
```

and must be extended rather than bypassed.

The metadata baseline before C3 was schema v4. P1 may have changed that baseline.
P2 must inspect the completed P1 metadata version and compatibility contract before
making any schema change.

### 3.3 Repository-wide application migration surface

A live pre-plan search found `TypeData::Applied` consumers across at least:

```text
phalcom-semantic/src/advisory/formal.rs
phalcom-semantic/src/export.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/types/rigid.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/src/types/variance.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/composition.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/row_inference.rs
```

This list is a migration map, not permission to mechanically widen every consumer.
The implementer must classify each consumer by semantic responsibility.

### 3.4 Known predecessor risks to verify closed

The requirements audit against the partial P1 commit found the following blockers:

```text
new TypeData variants were not integrated into all exhaustive matches
TypeOnly declarations were still predeclared as Type
TypeOnly semantic construction still reused runtime DataInfo/EnumInfo too directly
static-domain/constructor structures were not yet fully interned
metadata collapsed StaticDomain to Type
module declaration summaries lost TypeOnly phase
TypeOnly associated surfaces still looked runtime-oriented
static constructor identity used an insufficient name+arity scaffold
```

P2 is permitted to assume those are corrected **only after the P1 handoff and live tree
confirm them**.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat
> architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read these before substantive P2 edits:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-GUIDANCE.md

docs/implementation/LANG007/LANG007.C3/LANG007.C3.P1-...-walkthrough.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P1-...-handoff.md

docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-requirements-analysis.md
the accepted LANG007.C3 unified algebraic/type-data specification
the accepted C3 decisions referenced by the checkpoint

docs/implementation/LANG007/LANG007.C2/LANG007.C2-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-walkthrough.md

docs/specs/README.md
the effective type-system/generic-application specification
the effective type-level-domain/type-member specification
the effective data/algebraic declaration specification
the effective impl/trait specification
the effective module/import specification

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
```

Before each task, read only the exact source paths named in that task plus directly
adjacent helpers needed to understand the call path.

Do not broadly re-audit C1/C2/P1 if the P1 handoff and live tree agree.

---

## 5. Normative authority

### 5.1 Accepted static-domain model

The accepted model is:

```text
TypeId
    canonical handle for every non-row static term

ProperTypeId
    proof that a TypeId is specifically kind Type

KindId / KindData
    compact classifier
    including nominal static-domain kinds

StaticDomainId
    store-relative canonical domain identity

StaticArgument
    Term(TypeId)
    RecordRow(RecordRowId)

RecordRowId
    canonical row-domain handle outside TypeId

InferVarId
    non-row inference variable classified by KindId
```

This plan must not introduce a second generic-domain registry.

### 5.2 Accepted binder examples

P2 must support the semantic model represented by:

```phalcom
data Tensor<Element: Type, S: Shape>()
data Integer<Bits: Int, Order: Endian>()
```

and preserve existing:

```text
T: Type
P: TypePack
L: LabeledTypePack
E: TupleEntries
R: RecordRow
```

`TupleEntries` is the public C3 name for the existing complete two-lane pack domain.
The P2 implementation must preserve the internal C2 substrate so P3 can expose that
name canonically.

### 5.3 Accepted application examples

P2 must support:

```phalcom
Tensor<Float, Shape::Vector(3)>
Integer<32, Endian::Big>
```

without treating `3` or `Shape::Vector(3)` as runtime values or fake proper types.

### 5.4 Accepted inference boundary

P2 supports first-order structural static inference.

Required:

```text
Shape::Vector(N)
=
Shape::Vector(3)

=> N = 3
```

Not required:

```text
constructor search
backtracking
arbitrary static function reduction
type-level runtime method execution
general theorem proving
pack sequence decomposition
```

### 5.5 Accepted associated-domain model

For:

```phalcom
class Container<T: Type> {
    type data State {
        Empty
        Present(_ value: Type)
    }
}
```

there is one declaration-owned associated domain:

```text
Container::State
```

and multiple concrete specializations:

```text
Container<Int>::State
Container<String>::State
```

Concrete identity is:

```text
associated domain declaration identity
+
canonical owner environment
```

Never a formatted owner string.

### 5.6 Accepted runtime boundary

Type-data terms are compiler semantic terms.

They do not require:

```text
runtime objects
VM static constructors
runtime equality/hash
runtime method execution
runtime static solver
```

### 5.7 Current implementation is not normative

Pre-P2 details such as:

```text
Applied arguments = TypeId[]
KindSyntax closed over built-ins
ConstTerm::Int in the type-member subsystem
AssociatedFamilyKind only Variant/DataConstructor
```

are baseline implementation constraints, not language rules.

### 5.8 Future work is not P2 authority

P3 owns:

```text
Universe public intrinsic structural domains
final TupleEntries public declaration surface
broad corpus migration
legacy enum final compatibility/removal
full LSP/tooling polish
checkpoint-wide certification
```

---

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Expected P1/C2 symbol/path | Owner | P2 invariant |
|---|---|---|---|
| Canonical non-row static term | `TypeId` — `phalcom-semantic/src/types/id.rs` | TypeStore | remains the term carrier |
| Proper-type proof | `ProperTypeId` | TypeStore/API boundary | never weakened |
| Generic/static classifier | `KindId` / `KindData` | TypeStore | remains sole classifier |
| Nominal static domain | `StaticDomainId` / static-domain arena | TypeStore | one canonical domain identity |
| Heterogeneous argument | `StaticArgument` | semantic type layer | one envelope |
| Record row | `RecordRowId` | row subsystem | remains separate from TypeId |
| Generic modes | `GenericParameterMode` | generic signature | remains orthogonal to domain |
| Generic binder header | `GenericBinderHeader` | declarations/type formation | stores exact parameter kinds/modes |
| Generic signature | `GenericSignature` | semantic declaration authority | exact domain constraints |
| Applied form | `TypeData::Applied` | TypeStore | generalized, flat, canonical |
| Type lambdas | `TypeLambdaArena`, `ScopedTypeData` | semantic type layer | preserve heterogeneous terms |
| Inference variables | `InferVarId`, `InferenceVariable` | query-local inference | classified by KindId |
| Row inference | `row_inference` | checker | remains dedicated |
| Static terms | `ConstInt`, `StaticData` | TypeStore | structural canonical identity |
| Runtime exact case | `TypeData::ExactCase` / `VariantId` | runtime ADT semantics | not reused as type-data term identity |
| Type member | `TypeMemberDomain`, `TypeMemberArgument` | type-member subsystem | converge to canonical static model |
| Associated namespace | `AssociatedSurface` | semantic associated layer | one collision authority |
| Associated owner specialization | `AssociatedOwnerResolution` / specialization helpers | semantic associated layer | heterogeneous owner environment |
| Metadata | `phalcom-type-meta` graph | stable artifact model | explicit static-domain nodes |
| Module/incremental identity | `semantic_shard`, DB fingerprints | semantic workspace | preserves phase/domain facts |

### 6.2 Known inherited baseline failures

Carry forward only failures recorded in the completed P1 checkpoint/handoff.

Historically C2 recorded two unrelated trait-suite baseline failures:

```text
BASELINE-001
semantic::capabilities::traits::trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Do not repair inherited failures during P2 unless:

- P2 causally changes them; or
- they block a required P2 gate and cannot be proven unrelated.

---

## 7. Architecture

### 7.1 Authority flow

P2 must establish this direction of authority:

```text
source static-domain syntax
    ↓
semantic declaration/domain resolution
    ↓
KindId / StaticDomainId
    ↓
canonical static argument elaboration
    ↓
TypeStore heterogeneous application
    ↓
checked substitution / scoped type functions / inference
    ↓
aliases / type members / associated static domains
    ↓
stable metadata + incremental products
    ↓
compiler consumers only where runtime Types remain relevant
```

The compiler/VM must not recreate static-domain meaning.

### 7.2 Classifier architecture

All generic/static binders use `KindId`.

Examples:

```text
T: Type
    KindId::TYPE

P: TypePack
    KindId::TYPE_PACK

L: LabeledTypePack
    KindId::LABELED_TYPE_PACK

E: TupleEntries
    KindId::COMPLETE_TYPE_PACK

R: RecordRow
    KindId::RECORD_ROW

N: Int
    KindData::StaticDomain(ConstIntDomainId)

S: Shape
    KindData::StaticDomain(ShapeDomainId)
```

Mode remains independent:

```text
GenericParameterMode
    Single
    PositionalPack
    LabeledPack
    CompletePack
```

Do not encode pack mode as a nominal static domain.

### 7.3 Canonical application architecture

The target semantic shape is:

```rust
pub enum StaticArgument {
    Term(TypeId),
    RecordRow(RecordRowId),
}

pub enum TypeData {
    // ...
    Applied {
        origin: TypeId,
        arguments: Box<[StaticArgument]>,
    },
    // ...
}
```

The generalized application API is conceptually:

```rust
pub fn apply_static_form(
    &mut self,
    origin: TypeId,
    arguments: &[StaticArgument],
) -> Result<TypeId, StaticApplicationError>;
```

A type-only wrapper remains:

```rust
pub fn apply_type_form(
    &mut self,
    origin: TypeId,
    arguments: &[TypeId],
) -> Result<TypeId, TypeApplicationError>;
```

implemented by wrapping every argument as `StaticArgument::Term`.

The type-only wrapper is a compatibility/convenience projection. It is not a second
application implementation.

### 7.4 Flat applied-form canonicalization

Preserve the C1/C2 flat-application law:

```text
apply_static_form(
    Applied(F, [A]),
    [B, C]
)

= Applied(F, [A, B, C])
```

subject to kind/application validation.

Do not create nested left-associated `Applied(Applied(...), ...)` trees.

### 7.5 Static term identity

A type-data static term is defined structurally by canonical identity:

```text
concrete static domain
constructor/case declaration identity
canonical static arguments
```

For compile-time integer constants:

```text
ConstInt(3)
```

the integer value itself is part of structural canonical identity.

No runtime equality, hashing, or message send is consulted.

### 7.6 Scoped architecture

A type lambda may mention bound static terms:

```text
N : Int
Shape::Vector(N)
```

Therefore the scoped representation must support structural static arguments and static
constructor terms containing bound variables.

Conceptually:

```rust
enum ScopedStaticArgument {
    Term(ScopedTypeId),
    RecordRow(ScopedRecordRowTerm),
}

enum ScopedTypeData {
    // ...
    Applied {
        origin: ScopedTypeId,
        arguments: Box<[ScopedStaticArgument]>,
    },
    StaticData {
        domain: ScopedStaticDomain,
        constructor: StaticConstructorDeclarationIdentity,
        arguments: Box<[ScopedStaticArgument]>,
    },
    // ...
}
```

The exact row representation should reuse the established scoped row machinery.

### 7.7 Inference architecture

Reuse:

```text
InferVarId
InferenceVariable.kind: KindId
```

A user static-domain variable is not a new variable category.

Inference terms gain enough structure to represent symbolic static constructor terms and
heterogeneous applied arguments.

Static-domain solving is exact/domain-directed first-order unification.

### 7.8 Alias/type-member architecture

Static aliases and type members use the same domain resolver as generic binders.

Examples:

```phalcom
type DefaultShape: Shape = Shape::Scalar
type P: TypePack = T::Positionals
```

Unannotated alias behavior remains the existing proper-Type alias default unless the
effective specification says otherwise.

The existing type-member `ConstInt` seam becomes a compatibility projection onto the
canonical `ConstInt` static domain/term model.

### 7.9 Associated-domain architecture

An associated type-data declaration has stable declaration identity.

Conceptually:

```rust
AssociatedStaticDomainDeclarationId {
    owner: DeclarationId,
    name: Box<str>,
}
```

Concrete associated-domain specialization is structural:

```rust
StaticDomainData::Associated {
    declaration: AssociatedStaticDomainDeclarationId,
    owner_environment: StaticInstantiation,
}
```

Do not clone declarations per generic specialization.

### 7.10 Unified associated namespace

`AssociatedSurface` remains the owner/base namespace authority.

It must be able to classify names that denote:

```text
runtime algebraic cases
runtime data constructors
associated static domains
associated static terms / type members
intrinsic type members
```

The exact enum variants may differ, but collision discovery and lookup category selection
must pass through one authority.

### 7.11 Metadata architecture

Stable metadata must structurally represent:

```text
StaticDomain
ConstInt
StaticData
heterogeneous Applied arguments
static aliases
generic signatures with static-domain parameter kinds
associated static-domain declaration identity
associated owner specialization environment
scoped heterogeneous applications/static terms
```

Store-local IDs are never serialized as stable meaning.

### 7.12 Runtime boundary

P2 may modify compiler-facing semantic projections where necessary to preserve correct
runtime Types after the application representation changes.

P2 must not add:

```text
type-data runtime allocation
static term VM opcodes
static constructor VM opcodes
runtime static unification
runtime metadata solely to execute type-level computation
```

---

## 8. Ownership boundaries

### 8.1 Owns

P2 owns:

- arbitrary supported static-domain binder resolution;
- heterogeneous generic argument syntax/elaboration;
- generalized static application in `TypeStore`;
- heterogeneous application decomposition;
- checked heterogeneous instantiation/substitution/materialization;
- scoped heterogeneous/static-data terms;
- first-order static-domain inference;
- static alias result domains;
- type-member convergence onto canonical static domains/terms;
- associated type-data declarations and specialization;
- unified associated namespace categories required by P2;
- static-domain metadata/import persistence;
- incremental/source identity for P2 semantics;
- trait boundary diagnostics for unsupported static-domain generics;
- compile-time performance correction directly required by generalized application;
- P2 focused stabilization and handoff.

### 8.2 Does not own

P2 does not own:

- finishing unfinished P1 correctness;
- final Universe `TupleEntries` / intrinsic structural-domain declaration exposure;
- broad standard-library/corpus migration;
- final legacy `enum` removal;
- arbitrary compile-time function execution;
- trait-associated type-data requirements;
- GAT redesign;
- generalized static pattern matching;
- theorem proving;
- pack decomposition/search;
- VM static solving;
- runtime type-data objects;
- redesigning C1/C2 record-row or pack semantics;
- redesigning runtime ADT/GADT semantics;
- broad unrelated compiler/runtime cleanup.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Parameter static domain | `KindId` in generic binder/signature | application, inference, metadata | separate generic-domain registry |
| Nominal static-domain identity | TypeStore static-domain arena | kinding, static terms, metadata | name-string identity |
| Static argument | `StaticArgument` | application, substitution, associated specialization | bespoke per-subsystem argument enums |
| RecordRow term identity | row arena / `RecordRowId` | application, substitution, inference | fake row TypeId |
| Applied term | TypeStore `TypeData::Applied` | all type consumers | nested/consumer-local application terms |
| Compile-time Int term | canonical `TypeData::ConstInt` | generic args, type members, inference | independent permanent `ConstTerm::Int` identity |
| Type-data term | canonical `TypeData::StaticData` | application, inference, metadata | runtime object/value identity |
| Static inference variable | `InferVarId(kind)` | solver | `StaticInferVarId` |
| Static alias result | alias semantic product + canonical term | resolver, metadata | text-level alias replay |
| Associated static-domain declaration | semantic identity layer | AssociatedSurface, metadata | synthetic top-level declaration name |
| Associated domain specialization | declaration + canonical owner environment | lookup, metadata, inference | rendered owner string |
| Associated namespace collision | `AssociatedSurface` | checker/editor | parallel static-domain lookup table |
| Stable serialization | `phalcom-type-meta` nodes | imports/tooling | raw semantic IDs |
| Runtime type admissibility | `ProperTypeId` / kind checks | checker/compiler | “anything in TypeStore is a type” assumption |

---

## 9. Global invariants

### INV-P2-01 — P1 truth is a hard prerequisite

P2 never compensates for unfinished P1 by preserving semantically false TypeOnly
representation.

### INV-P2-02 — one classifier

`KindId` remains the sole generic/static-domain classifier.

### INV-P2-03 — proper Type remains a proof boundary

A non-`KindId::TYPE` `TypeId` cannot be used as a runtime/proper type merely because
it is stored in `TypeStore`.

### INV-P2-04 — one heterogeneous argument envelope

`StaticArgument` is the canonical generalized application argument representation.

### INV-P2-05 — RecordRow stays separate

Rows remain `RecordRowId`/row solver values and are never encoded as fake TypeIds.

### INV-P2-06 — applied forms remain flat and canonical

Successful generalized application produces one hash-consed flat applied form.

### INV-P2-07 — static identity is structural

Static term equality and canonicalization use compiler structural identity only.

### INV-P2-08 — static application is checked

Every argument is checked against the origin's exact domain/kind contract.

### INV-P2-09 — substitution is checked and fail-closed

No substitution/materialization path silently falls back to the old term after a failed
re-canonicalization.

### INV-P2-10 — scoped terms preserve domain category

Type lambdas/aliases cannot lose whether an argument is a normal static term or
RecordRow.

### INV-P2-11 — one inference-variable model

All non-row static domains use `InferVarId(kind)`.

### INV-P2-12 — static inference is first-order

Same constructor recurses; different constructor/domain refutes; no search/backtracking.

### INV-P2-13 — arbitrary static domains do not imply subtyping

User static-domain terms use definitional/structural equality unless an explicitly
accepted future feature adds more.

### INV-P2-14 — type members converge

Type-member constants/domains are projections of canonical static-domain semantics, not
a separate permanent term universe.

### INV-P2-15 — associated-domain declaration identity is stable

An associated static domain is declaration-owned; specialization is declaration plus
canonical owner environment.

### INV-P2-16 — no string semantic identity

Formatted names such as `Container<Int>::State` are presentation only.

### INV-P2-17 — one associated namespace authority

`AssociatedSurface` decides owner/base category and collisions for every P2-associated
entity.

### INV-P2-18 — trait boundary is explicit

Unsupported trait generic use of arbitrary user static domains is rejected; existing
trait evidence/coherence code is not silently widened.

### INV-P2-19 — persistence is lossless

Stable metadata round-trips static domains, terms, heterogeneous applications, and
associated specialization without collapsing them to `Type`.

### INV-P2-20 — incremental equals cold

For equivalent source, final incremental semantic products equal final cold semantic
products.

### INV-P2-21 — no runtime static solver

P2 adds no VM/runtime execution machinery for static domains.

### INV-P2-22 — type-lambda application is not arena-size copying

One beta reduction must not clone work proportional to the whole accumulated
`TypeLambdaArena`.

---

## 10. Non-goals

Do not expand P2 into:

```text
general compile-time interpreter
arbitrary static functions
runtime method execution during type checking
trait requirements that bind whole type-data domains
generalized associated type-data requirements in traits
GAT redesign
theorem proving
static pattern-matching language
automatic runtime-value lifting
pack partition/decomposition search
dependent runtime values
VM static term objects/opcodes
Tuple/Record runtime representation redesign
full P3 Universe migration
full P3 LSP/corpus certification
```

---

## 11. Expected impact map

### 11.1 Expected parser/AST areas

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs
phalcom-ast/tests/parser.rs
phalcom-ast/tests/data_syntax.rs
```

Expected reasons:

```text
arbitrary static-domain binder syntax
heterogeneous generic arguments
static alias result domains
associated type-data declaration syntax
```

### 11.2 Expected semantic type/store areas

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/variance.rs
phalcom-semantic/src/types/rigid.rs
```

### 11.3 Expected checker/inference areas

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/composition.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/algebraic_declaration.rs
```

### 11.4 Expected declaration/identity/associated areas

```text
phalcom-semantic/src/algebraic_semantics.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/type_alias.rs
phalcom-semantic/src/associated.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
```

### 11.5 Expected module/incremental/source-index areas

```text
phalcom-modules/src/declaration.rs
phalcom-modules/src/interface.rs

phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/source_index/symbol.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/source_index/mod.rs
```

### 11.6 Expected metadata areas

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/declaration.rs
phalcom-type-meta/src/bundle.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/tests/schema_compat.rs

phalcom-semantic/src/metadata/export.rs
```

### 11.7 Expected compiler/runtime areas

Expected compiler changes are limited to mechanical adaptation of semantic application
representation and proof that static terms do not lower.

Likely areas:

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/mod.rs
```

A substantial VM/runtime implementation is unexpected.

### 11.8 Expected tests

Likely:

```text
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/adts/declarations.rs
phalcom-semantic/tests/semantic/adts/associated/lookup.rs
phalcom-semantic/tests/semantic/adts/associated/scenarios.rs
phalcom-semantic/tests/semantic/incremental/adts.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
plus a focused new static-domain/type-data test module if existing placement is poor

phalcom-type-meta/tests/schema_compat.rs
```

### 11.9 Expected documentation/state

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md

LANG007.C3.P2-walkthrough.md
LANG007.C3.P2-handoff.md
```

### 11.10 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem for an architectural
reason, especially:

```text
GC
concurrency/fibers
runtime object layout
VM instruction set
trait coherence/evidence redesign
module linker identity redesign
global compiler IR redesign
```

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

- `TypeId` as the non-row canonical static-term carrier;
- `ProperTypeId` as the proper-Type proof boundary;
- `RecordRowId` as a dedicated non-TypeId domain;
- `KindId` as the classifier for generic/static binders;
- `StaticArgument` as the heterogeneous application envelope;
- exact structural identity for static terms;
- first-order, non-search static inference;
- declaration+environment associated-domain specialization identity;
- `AssociatedSurface` as namespace/collision authority;
- explicit rejection of unsupported arbitrary static-domain trait generic parameters;
- stable metadata structural identity rules;
- zero runtime static evaluator/solver;
- C2 pack/row semantics;
- mature runtime ADT/GADT/exact-case semantics.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- private helper names;
- the exact name `apply_static_form` if an equivalent canonical API is clearer;
- `StaticApplicationError` internal enum decomposition;
- whether `GenericInstantiation` physically uses one `StaticArgument` map or separate
  term/row maps with generalized accessors;
- local parser helper decomposition;
- exact Rust enum variant names for associated static result categories;
- exact metadata node factoring/indexing;
- source-index display category names;
- test module placement within the repository's documented conventions;
- borrow-safe mechanics used to avoid type-lambda arena cloning.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| P1 is complete and focused-tested | C3 checkpoint + P1 walkthrough/handoff + live source | stop P2; return to P1 |
| TypeOnly domains have a canonical interner | `types/store.rs`, P1 handoff | stop P2 if absent |
| `StaticArgument` still exists with term/row split | type/store modules | adapt mechanically only if equivalent |
| `KindData::StaticDomain` remains canonical | kind/store modules | STOP if replaced architecturally |
| P1 metadata has an explicit static-domain representation | `phalcom-type-meta`, exporter | stop P2 if metadata still lies |
| `TypeData::Applied` still stores type-only args | `types/store.rs` | adapt plan mechanically if P1 reserved heterogeneous shape |
| `GenericArgumentSyntax` still lacks static terms | AST/parser | adapt mechanically |
| `InferenceVariable.kind` remains `KindId` | inference.rs | STOP if solver identity changed |
| trait refs remain type-only | traits/impls metadata | preserve explicit rejection boundary |
| `AssociatedSurface` remains canonical owner/base namespace | associated.rs | STOP if P1 changed authority |
| beta reduction still clones the arena | `types/store.rs` | skip correction if already fixed |
| runtime-data associated declaration syntax is ratified | effective spec | if absent/ambiguous, consult before inventing syntax |
| metadata schema baseline is known | `phalcom-type-meta/src/header.rs` + P1 walkthrough | select migration from live baseline |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing and build a consultation packet if any of the following
occurs:

1. P1 is not actually complete enough to satisfy the hard predecessor rule;
2. the accepted `TypeId`/`ProperTypeId`/`RecordRowId` architecture appears unable to
   represent a required P2 case;
3. implementing user static-domain binders appears to require a second classifier
   registry;
4. heterogeneous application appears to require encoding RecordRow as TypeId;
5. a proper/runtime type API would need to accept arbitrary static terms;
6. associated-domain identity appears to require rendered strings or synthetic
   top-level declarations;
7. trait evidence/coherence would require material widening rather than targeted
   rejection;
8. first-order unification cannot express a required accepted P2 inference behavior;
9. implementation appears to require runtime Phalcom method execution;
10. implementation appears to require new VM static-term objects/opcodes;
11. runtime-data-owned associated `type data` syntax is not unambiguously established by
    the effective specification;
12. metadata compatibility cannot represent the new static categories without a
    material schema-policy decision;
13. a core plan premise is contradicted by live repository evidence;
14. ownership, identity, lifetime, representation, or semantic authority must change;
15. a downstream contract relied upon by later tasks must materially change;
16. the same nontrivial semantic failure persists after one serious correction;
17. two plausible solutions have materially different architectural consequences;
18. an invariant must be weakened merely to pass tests;
19. a workaround/consumer-specific parallel source of truth is emerging;
20. scope becomes materially broader than the expected impact map.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while each cycle has a concrete cause and
makes progress.

A cycle is:

```text
inspect failure
identify concrete cause
make one coherent correction
rerun the smallest useful discriminator
```

Examples:

```text
exhaustive match update
import/rename drift
borrow-checker repair
test selector correction
metadata enum exhaustiveness
```

Do not make speculative random patches.

### 14.2 Semantic failures

Before a nontrivial correction record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic problem.

If substantially the same failure remains: STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture attempts.

Consult immediately.

Never rerun an unchanged failing test unless relevant code/evidence changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-P2-01 | P1 predecessor truth | INV-01 | P1 focused baseline and TypeOnly domain facts are real |
| CV-P2-02 | binder classification | INV-02, INV-08 | `T: Type`, `N: Int`, `S: Shape`, packs, row |
| CV-P2-03 | heterogeneous application | INV-04, INV-06, INV-08 | `Tensor<Float, Shape::Vector(3)>` |
| CV-P2-04 | row preservation | INV-05, INV-10 | RecordRow argument survives application/substitution |
| CV-P2-05 | proper-Type boundary | INV-03 | static term rejected where runtime type required |
| CV-P2-06 | canonical identity | INV-06, INV-07 | repeated static application/term formation interns identically |
| CV-P2-07 | static term inequality | INV-07 | constructor/domain/payload differences remain distinct |
| CV-P2-08 | checked substitution | INV-09 | invalid substitution reports error, never stale fallback |
| CV-P2-09 | scoped preservation | INV-10, INV-22 | bound static term reduces without arena clone |
| CV-P2-10 | first-order inference | INV-11, INV-12 | `Vector(N)` unifies with `Vector(3)` |
| CV-P2-11 | inference refutation | INV-12, INV-13 | different domain/constructor rejects |
| CV-P2-12 | static aliases | INV-02, INV-09 | `type DefaultShape: Shape = ...` |
| CV-P2-13 | type-member convergence | INV-14 | const Int and user domain use canonical terms |
| CV-P2-14 | associated declaration identity | INV-15, INV-16 | one `Container::State` declaration, specialized environments differ |
| CV-P2-15 | unified `::` namespace | INV-17 | collisions/categories resolved by AssociatedSurface |
| CV-P2-16 | trait boundary | INV-18 | unsupported user static-domain trait binder/application rejected |
| CV-P2-17 | metadata round trip | INV-19 | domains/terms/Applied/associated specialization survive export/import |
| CV-P2-18 | cold/incremental parity | INV-20 | provider/domain/case/application edits |
| CV-P2-19 | runtime negative | INV-03, INV-21 | no static-term runtime materialization/VM execution |
| CV-P2-20 | regression | INV-03, INV-05 | existing C1/C2 Type/pack/row generic applications unchanged |
| CV-P2-21 | performance | INV-22 | beta reduction no longer clones whole arena |
| CV-P2-22 | diagnostics | INV-02, INV-08 | actual nominal domain names in mismatch errors |

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Use sparse, discriminating feedback only.

Prefer:

```text
cargo check of the owning crate
exact new regression
single semantic test filter
metadata unit/schema test
```

Do not broad-test an intentionally incomplete representation migration.

#### STABILIZE MODE

Run focused regressions and directly affected semantic/type-meta suites after all P2
tasks form a coherent system.

#### CERTIFY MODE

P2 is not C3 checkpoint closure. Do not run workspace/release certification by reflex.

Only run broad certification if:

- the user explicitly requests it;
- the checkpoint plan is amended to require it; or
- evidence shows a broader gate is needed to establish P2 correctness.

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

At coherent task boundaries, as applicable:

```sh
RUSTFLAGS='' cargo check -p phalcom-ast
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo check -p phalcom-type-meta
```

Do not run all three after every edit. Run the one that answers the current question.

For exact semantic test selection, first verify names if necessary:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic -- --list
```

### 16.4 Mandatory at named gates

The exact commands are listed in §19.

### 16.5 Mandatory during STABILIZE

At final P2 stabilization:

```sh
cargo fmt --all -- --check

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generic_application
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generics_core
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::associated
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::adts
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::metadata

RUSTFLAGS='' cargo test -p phalcom-type-meta
```

Adapt filters mechanically if the live semantic test module names differ; verify filters
select actual tests.

Also run:

```sh
RUSTFLAGS='' cargo check -p phalcom-core
```

to prove the compiler still consumes the generalized semantic model without new runtime
static machinery.

### 16.6 Only if evidence demands

```text
broader phalcom-semantic crate tests
phalcom-core affected runtime/corpus tests
module/linker tests
LSP tests
```

### 16.7 Explicitly deferred / DO NOT RUN during ordinary BUILD

Do not run by reflex:

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Do not run broad unrelated language corpora after every task.

Do not run full P3 migration/tooling suites.

Reason: P2 is a cross-cutting semantic representation migration; broad validation before
the full static-domain pipeline is coherent creates noise rather than useful evidence.

---

## 17. Baseline / unrelated failure policy

Classify unexpected failures:

```text
A — definitely caused by P2
B — probably caused by P2
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active responsibility;
- C: one bounded classification pass; if still unclear, nonblocking, and outside P2
  acceptance, record and continue;
- D: record and continue immediately;
- do not consume adviser compute on D unless it blocks a P2 gate;
- do not weaken assertions, skip tests, or alter expected semantics merely to make broad
  gates green;
- if a suspected baseline must be confirmed, compare non-destructively with the P1
  completion revision.

Record meaningful deferred failures in the C3 checkpoint.

---

# 18. Tasks

## T1 — Establish the truthful P2 baseline and freeze the generalized-domain contract

### Purpose

Prove that completed P1 supplies the required static-domain foundation, activate P2 in
checkpoint state, and create focused P2 regressions that describe the generalized-domain
contract before representation widening begins.

### Preconditions

- P1 is `IMPLEMENTED` and at least `FOCUSED_TESTED`;
- P1 walkthrough/handoff exist;
- the working tree/revision to be used for P2 is known;
- P1 no longer contains the semantic defects listed in the P2 requirements audit.

### Consumes

- completed P1 `StaticDomainId`/domain interner;
- canonical `ConstInt` and `StaticData` terms;
- P1 static constructor/case declaration identity;
- P1 TypeOnly/runtime-use rejection;
- C2 pack/row substrate;
- C3 checkpoint state.

### Produces

- recorded P2 starting revision;
- checkpoint plan ledger marks P2 active;
- verified live interface map for P2;
- focused tests/fixtures for binder domains and generalized application syntax that may
  initially fail until T2/T3;
- no semantic representation migration yet.

### Files and symbols

**Read:**

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
P1 walkthrough/handoff

phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/checker/algebraic_declaration.rs
phalcom-semantic/src/algebraic_semantics.rs
phalcom-semantic/src/metadata/export.rs
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/kind.rs
```

**Modify:**

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
optionally the focused semantic test module that will host P2 regressions
```

**Tests:**

```text
P1-focused static-domain tests named by P1 handoff
new P2 binder/application regressions
```

### Required implementation shape

1. Record the completed P1 revision.
2. Confirm:
   - TypeOnly domains resolve to nominal static-domain kinds;
   - `ProperTypeId` rejects static-domain terms;
   - TypeOnly terms have no runtime invocation target;
   - stable metadata does not encode static domain as `Type`;
   - repeated static term formation canonicalizes.
3. Confirm the canonical concrete Rust types the rest of P2 will use.
4. Add or stage focused P2 tests for:
   ```phalcom
   data Tensor<T: Type, S: Shape>()
   data Integer<Bits: Int, Order: Endian>()
   ```
   and:
   ```phalcom
   Tensor<Float, Shape::Vector(3)>
   Integer<32, Endian::Big>
   ```
5. Do not implement generalized application in T1.

### Forbidden approaches

- no P2 patch that compensates for missing P1 semantics;
- no second static-domain interner;
- no placeholder “treat StaticDomain as Type” compatibility;
- no broad application migration yet.

### Test changes required

Coverage:

```text
CV-P2-01
CV-P2-02
CV-P2-03
CV-P2-05
```

The P2 syntax/application regressions may remain expected-failing until T2/T3 if the
repository convention supports test-first staging. Do not weaken parser/semantic
assertions merely to get an intermediate green state.

### Tests to run now

Run the exact P1 handoff gate that proves the predecessor foundation.

Also run:

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
```

If it fails due to unfinished P1 static-domain semantics, stop P2.

### Acceptance

T1 passes when:

```text
P1 prerequisite truth is independently confirmed
P2 active revision is recorded
all fixed P2 interface assumptions have concrete live symbols
no P1 corrective code is being hidden inside P2
```

### Local STOP / CONSULT triggers

Stop immediately if:

- P1 metadata still collapses static domains;
- TypeOnly declarations still have ordinary proper-Type/class-object semantics;
- the canonical P1 static constructor identity cannot distinguish accepted case shapes;
- the P1 handoff claims an invariant the live tree contradicts.

### Checkpoint update

Record:

```text
P2 active
starting revision
confirmed P1 foundation
any inherited baseline failures
```

---

## T2 — Generalize static-domain syntax, binder resolution, and static argument elaboration

### Purpose

Make arbitrary supported static domains expressible in generic binder/result-domain
positions and make static generic arguments parse/elaborate against those domains,
without yet widening every downstream `Applied` consumer.

### Preconditions

- T1 accepted;
- P1 domain interning is truthful;
- the live static-domain declaration resolver is known.

### Consumes

```text
KindId / KindData
StaticDomainId
GenericParameterSyntax
GenericBinderHeader
GenericSignature
TypeAnnotation/type parser
P1 static term formation
```

### Produces

- source syntax for `N: Int`, `S: Shape`;
- one generalized static-domain resolver;
- bounded static argument syntax;
- domain-directed static argument elaboration;
- explicit trait-site rejection for unsupported arbitrary static-domain parameters;
- no alternate domain registry.

### Files and symbols

**Read/modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs

phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
```

**Tests:**

```text
phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
```

### Required implementation shape

#### A. Generalize binder-domain syntax

The source after `:` in a generic binder must be able to denote:

```text
Type
RecordRow
TypePack
LabeledTypePack
TupleEntries / complete pack domain where accepted
Int
Shape
qualified/imported nominal static domains
arrow kinds/type constructors where already supported
```

Two mechanical representations are acceptable:

```text
extend KindSyntax with nominal/static-domain references
```

or:

```text
introduce a more explicit StaticDomainSyntax and migrate binder fields
```

Prefer the smallest change that preserves the accepted architecture.

Do not turn arbitrary runtime expressions into binder-domain syntax.

#### B. Make domain resolution semantic

Current store-only closed helpers such as:

```rust
resolve_kind_syntax(store, syntax)
```

are insufficient for a nominal domain reference.

The generalized resolver must have access to:

```text
current module
import/name resolution
declaration/static-domain facts
TypeStore
```

and return one canonical `KindId`.

Built-in domain spellings may remain parser-recognized shortcuts, but nominal static
domains must resolve through declaration identity.

#### C. Preserve mode orthogonality

This remains valid:

```text
parameter domain
    KindId

parameter mode
    GenericParameterMode
```

Do not encode `*`, `**`, `***` semantics into nominal static domains.

#### D. Generalize generic argument AST

Replace the type-only `Single(TypeAnnotation)` assumption with a bounded static argument
syntax.

Conceptually:

```rust
enum StaticArgumentSyntax {
    TypeForm(TypeAnnotation),
    IntLiteral { value: i64, range: SourceRange },
    StaticTerm(StaticTermSyntax),
}

enum GenericArgumentSyntax {
    Single(StaticArgumentSyntax),
    Expansion(TypePackExpansionSyntax),
}
```

Exact factoring may reuse existing `TypeMemberArgumentSyntax` parsing primitives where
semantics align, but do not make type-member syntax the canonical language model.

#### E. Bound static term grammar

Static generic arguments may include the accepted structural forms:

```text
integer literal
type form
type-data singleton term
type-data constructor term
associated static-domain/static-case path once T6/T7 lands
existing compiler static projections
pack expansions already owned by C2
```

They may not include general runtime expressions.

#### F. Domain-directed elaboration

At a generic application site:

1. resolve the generic origin;
2. obtain binder kinds/modes;
3. parse/elaborate each source argument in the expected domain;
4. form canonical `StaticArgument`;
5. defer actual generalized `Applied` construction to the T3 application API.

Examples:

```text
expected Int domain
source 32
=> Term(ConstInt(32))

expected Shape domain
source Shape::Vector(3)
=> Term(StaticData(...))

expected Type
source Float
=> Term(Float TypeId)

expected RecordRow
source row argument
=> RecordRow(RecordRowId)
```

#### G. Trait boundary

If a trait generic declaration/application uses an arbitrary user static domain and the
trait evidence stack remains type-only:

```text
emit a targeted unsupported-domain diagnostic
```

Do not pretend support by putting arbitrary static terms into existing trait arguments.

A dedicated `GenericBinderSite::Trait` is recommended if it makes this boundary
mechanically explicit.

### Forbidden approaches

- no general expression parser for static generic arguments;
- no `String`-based domain resolution;
- no second kind/domain registry;
- no implicit runtime-value lifting;
- no silent arbitrary static-domain trait support;
- no pack-mode redesign.

### Test changes required

Coverage:

```text
CV-P2-02
CV-P2-03
CV-P2-16
CV-P2-20
CV-P2-22
```

Required negative cases:

```text
unknown nominal static domain
runtime data declaration used as binder domain
wrong static argument syntax
runtime call/expression used as static argument
trait user-static-domain generic rejected
```

### Tests to run now

```sh
RUSTFLAGS='' cargo test -p phalcom-ast -- parser
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generics_core
```

Use narrower filters if available.

### Tests explicitly deferred

Do not run associated-domain, metadata, or incremental suites yet.

### Acceptance

T2 passes when:

```text
N: Int resolves to the canonical ConstInt static domain kind
S: Shape resolves to the exact nominal Shape domain kind
ordinary Type/pack/row binder behavior remains unchanged
static arguments elaborate to canonical StaticArgument values
trait boundary fails explicitly rather than accidentally
```

### Local STOP / CONSULT triggers

Stop if:

- nominal domain resolution requires bypassing declaration/module identity;
- `KindId` cannot represent the required binder domain without a new registry;
- grammar ambiguity forces runtime expression parsing;
- trait support appears to require coherence/evidence redesign.

### Checkpoint update

Record the generalized domain resolver and source-level static argument contract as
durable interfaces.

---

## T3 — Generalize canonical application, instantiation, substitution, and scoped type functions

### Purpose

Widen the central static application representation to heterogeneous arguments and make
every canonical transform preserve/revalidate that representation, including scoped type
lambdas.

### Preconditions

- T2 accepted;
- static arguments can be elaborated independently;
- C2 pack/row invariants are green at focused level.

### Consumes

```text
StaticArgument
KindId
TypeData::Applied
GenericInstantiation
TypeSubstitution
TypeLambdaArena / ScopedTypeData
P1 static terms
C2 row/pack canonicalization
```

### Produces

- generalized flat `TypeData::Applied`;
- canonical generalized application API;
- type-only compatibility wrapper;
- heterogeneous applied decomposition;
- generalized `GenericInstantiation`;
- fail-closed checked substitution/materialization;
- scoped heterogeneous/static-data terms;
- beta reduction without whole-arena clone;
- downstream consumer classification at application representation boundary.

### Files and symbols

**Primary read/modify:**

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/rigid.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/variance.rs
```

**Secondary mechanical consumers:**

```text
phalcom-semantic/src/advisory/formal.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/src/export.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/composition.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
existing C2 pack/row tests
```

### Required implementation shape

#### A. Generalized application API

Introduce one authoritative path conceptually equivalent to:

```rust
fn apply_static_form(
    &mut self,
    origin: TypeId,
    arguments: &[StaticArgument],
) -> Result<TypeId, StaticApplicationError>
```

It must:

1. classify every argument;
2. apply the origin kind to those argument kinds;
3. reject wrong-domain arguments;
4. preserve residual kind for partial applications where existing semantics allow it;
5. beta-reduce type lambdas;
6. flatten an existing `Applied`;
7. intern the final term.

#### B. Type-only wrapper

Retain `apply_type_form` or equivalent for callers that already have only TypeId
arguments.

It must delegate to the generalized implementation.

Do not maintain two application algorithms.

#### C. Generalized decomposition

Any helper equivalent to:

```text
applied_nominal_parts
specialize receiver to owner
extract origin + arguments
```

must preserve `StaticArgument`.

Where a downstream runtime/proper-type consumer semantically requires only type-valued
generic arguments, validate that explicitly instead of destructuring with an assumed
`TypeId`.

#### D. Classify all `TypeData::Applied` consumers

For each repository consumer, record whether it is:

```text
A — general static-term consumer
B — proper/runtime-type consumer
C — trait/conformance consumer
D — presentation/advisory consumer
```

Implementation behavior:

- A: preserve all `StaticArgument` variants;
- B: accept only categories semantically valid for that operation;
- C: retain the explicit trait boundary unless deliberately audited;
- D: format/inspect without creating semantic identity.

This classification can live in implementation notes/walkthrough; it need not become a
source file.

#### E. GenericInstantiation

Prefer either:

```rust
term_bindings: HashMap<TypeParameterId, TypeId>
row_bindings: HashMap<TypeParameterId, RecordRowId>
```

with generalized accessors, or:

```rust
bindings: HashMap<TypeParameterId, StaticArgument>
```

if that makes existing row logic no worse.

The public semantic operation should be generalized:

```text
bind_static
static_binding
```

even if storage remains split.

#### F. Checked substitution

Remove fail-open behavior such as:

```rust
apply(...).unwrap_or(original)
```

from the canonical checked path.

Every rebuilt term goes through the proper checked constructor.

Extend the error model to preserve failure category:

```text
application
static-data formation
pack normalization
row normalization
associated-domain formation (once T6 lands)
```

#### G. Recursive static-data substitution

For:

```text
Shape::Vector(N)
[N -> 3]
```

materialize:

```text
the exact canonical Shape::Vector(3) TypeId
```

The same rule applies inside `Applied` arguments.

#### H. Scoped heterogeneous representation

Generalize scoped application arguments.

Add structural scoped static-data terms for bound variables.

Do not represent a bound static-data constructor application as a formatted/free string.

#### I. Beta reduction performance

Remove whole-arena cloning if still present.

A move-out/restore pattern such as `std::mem::take` is acceptable if semantics remain
correct.

Required complexity invariant:

```text
one beta reduction does not clone the entire existing lambda arena
```

#### J. Relation/variance behavior

Do not invent variance/subtyping semantics for arbitrary static terms.

Where variance traversal needs to inspect generic arguments:

- recurse into Type-domain arguments according to the existing declaration variance;
- treat invariant/static exact-domain arguments according to the declaration's actual
  supported semantics;
- do not call runtime subtype logic on `Shape::Vector(3)`.

If existing nominal generic variance metadata is Type-specific and cannot safely cover
static terms, preserve conservative exact equality and consult before inventing rules.

### Forbidden approaches

- no fake TypeId for rows;
- no nested applied trees;
- no unchecked substitution fallback;
- no blanket `_ =>` match arms hiding new semantic categories;
- no runtime subtype semantics for arbitrary static domains;
- no duplicate type-only application algorithm;
- no whole-arena cloning on beta reduction.

### Test changes required

Coverage:

```text
CV-P2-03
CV-P2-04
CV-P2-05
CV-P2-06
CV-P2-07
CV-P2-08
CV-P2-09
CV-P2-20
CV-P2-21
```

Required focused cases:

```text
Type-only generic application still canonicalizes
ConstInt/static-data arguments canonicalize
RecordRow application category survives
partial application preserves residual kind
repeated application produces same canonical TypeId
substitution changes nested static-data payload
wrong-domain substitution errors
pack duplicate exposed by substitution still errors
lambda with bound Int/static-data term beta-reduces
```

### Tests to run now

Start with:

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
```

Then focused:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generic_application
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations
```

Run exact C2 row/pack tests named by the C2/P1 handoff if the generalized application
touches those paths.

### Tests explicitly deferred

Do not run inference/associated/metadata broad suites until later tasks.

### Acceptance

T3 passes when:

```text
all canonical application construction uses StaticArgument
type-only callers remain source-compatible through a wrapper
rows remain dedicated
substitution/materialization is checked
scoped terms preserve heterogeneous arguments
beta reduction no longer clones the whole arena
existing C2 pack/row behavior remains focused-green
```

### Local STOP / CONSULT triggers

Stop if:

- any required proper-type operation cannot be separated from general static application;
- existing variance/subtyping requires a material new static-domain theory;
- row arguments cannot survive scoped representation without redesigning row identity;
- beta-reduction correction appears to change lambda identity/capture semantics.

### Checkpoint update

Record:

```text
canonical generalized application API
StaticArgument Applied representation
checked substitution invariant
scoped heterogeneous-term invariant
beta-reduction resource invariant
```

---

## T4 — Extend first-order inference to arbitrary non-row static domains

### Purpose

Make the existing `InferVarId(kind)` solver understand structural static-data terms and
heterogeneous applications while preserving row inference and avoiding search.

### Preconditions

- T3 accepted;
- generalized application/substitution is canonical and checked;
- scoped static terms are representable.

### Consumes

```text
InferVarId
InferenceVariable.kind
InferenceTerm
StaticArgument
StaticData
ConstInt
TypeStore generalized application
row inference
```

### Produces

- inference terms for symbolic static-data constructor terms;
- heterogeneous inference applied arguments;
- exact domain-safe solving;
- same-constructor recursive unification;
- different-constructor/domain refutation;
- occurs-check traversal through static terms;
- canonical materialization through TypeStore.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/types/store.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
new focused static-domain inference tests if necessary
```

### Required implementation shape

#### A. Preserve variable identity

Continue using:

```rust
InferenceVariable {
    id: InferVarId,
    kind: KindId,
    ...
}
```

For:

```text
S: Shape
```

the solver variable kind is exactly:

```text
KindData::StaticDomain(ShapeDomainId)
```

For:

```text
N: Int
```

it is the canonical ConstInt static-domain kind.

#### B. Generalize inference application arguments

Conceptually:

```rust
enum InferenceStaticArgument {
    Term(InferenceTerm),
    RecordRow(InferenceRecordTerm),
}
```

`InferenceTerm::Applied` uses this envelope.

Reuse the existing row inference representation for the row category.

#### C. Structural static-data term

Add a solver-local term conceptually equivalent to:

```rust
InferenceTerm::StaticData {
    domain: StaticDomainId,
    constructor: AlgebraicStaticConstructorId,
    arguments: Box<[InferenceStaticArgument]>,
}
```

A fully concrete term may remain `InferenceTerm::Canonical(TypeId)`.

Structural lifting is needed only when a child contains an inference/rigid/scoped term.

#### D. Unification laws

Implement:

```text
StaticData(D, C, A[])
unify
StaticData(D, C, B[])
    => pairwise unify A/B

StaticData(D1, ...)
unify
StaticData(D2, ...)
where D1 != D2
    => structural mismatch

same domain, different constructor
    => structural mismatch
```

For `ConstInt`:

```text
Canonical(ConstInt(3)) == Canonical(ConstInt(3))
Canonical(ConstInt(3)) != Canonical(ConstInt(4))
```

An Int-domain variable may solve to a canonical `ConstInt`.

#### E. Domain safety

When solving one variable, the candidate's resulting classifier must equal the
variable's `KindId`.

Do not solve:

```text
Shape var -> Type
Shape var -> Int
Shape var -> OtherShape
Int var   -> Shape
```

#### F. No new static subtyping

Existing subtype bounds remain meaningful for Type-domain inference.

For arbitrary nominal static domains, use exact/equivalence structural reasoning only.

If a solver path tries to derive subtype bounds between arbitrary static terms, return a
targeted unsupported/mismatch result rather than invoking Type subtyping.

#### G. Occurs check

Traverse:

```text
StaticData arguments
heterogeneous Applied arguments
nested packs
rows through existing row path
```

#### H. Materialization

Solver result materialization must call canonical TypeStore APIs:

```text
const_int
static_data
apply_static_form
pack/row constructors
```

No solver-local variable may enter a published TypeId or metadata product.

### Forbidden approaches

- no `StaticInferVarId`;
- no constructor enumeration/search;
- no backtracking;
- no arbitrary evaluation;
- no fake subtype lattice;
- no conversion of row variables into normal inference terms merely for convenience.

### Test changes required

Coverage:

```text
CV-P2-10
CV-P2-11
CV-P2-03
CV-P2-04
CV-P2-20
```

Required tests:

```text
Vector(N) with Vector(3) => N=3
Vector(N) with Scalar => mismatch
Shape::Vector(3) with OtherShape::Vector(3) => mismatch
nested static constructor recursive unification
wrong-domain inference variable candidate rejected
underconstrained static variable remains underconstrained
static argument in generic application contributes inference
existing Type generic inference unchanged
existing whole-pack inference unchanged
```

### Tests to run now

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generic_application
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generics_core
```

Add the exact new static-inference filter if placed elsewhere.

### Acceptance

T4 passes when static-domain variables infer/refute structurally with no new inference
identity and no backtracking.

### Architecture amendment — query-context row coordination (2026-09-23)

INC-C3-8 amends T4's row lifetime boundary after live call-path review. Preserve the
distinct `InferVarId` and `RecordRowVarId` spaces and the `InferenceStaticArgument`
envelope. A query-context-owned coordinator owns the ordinary inference session and a
separate row solver, routes mixed equations by category, and keeps row variables alive
for the inference-context lifetime. Do not hold a `RefCell` borrow across argument
analysis or nested calls.

Frame completion must solve/check only the row roots required by that frame's result.
Unresolved cross-frame equations remain available to the parent; a child result that
still depends on an unresolved outer row remains symbolic/blocked and cannot be
materialized as a concrete row. Replace the current all-allocated-variable underconstraint
check for coordinated solves with explicit frame-scoped root checks. Row solver IDs must
never enter canonical types, metadata, or published products.

This changes solver ownership/lifetime, not language row semantics or variable identity.
The amended focused gate covers mixed `Applied` and `StaticData` arguments, mismatch and
occurs checks, distinct identities, retained underconstraint, child-to-parent resolution,
and unchanged ordinary type inference. Do not implement T4 without the coordinator and
frame-scope behavior in place.

#### Row payload composition amendment — 2026-09-23

The query-context coordinator uses the existing `RecordRowVarId` space and generalizes
the row solver over its field payload `V` (keeping `TypeId` as the default/current use).
The row solver owns label/tail shape unification and emits `(V, V)` obligations for
same-label fields; it must not treat Rust equality of field payloads as semantic field
equality. The inference coordinator instantiates it with `V = InferenceTerm`, lifts
canonical fields as `InferenceTerm::Canonical(TypeId)`, and keeps inferred fields as
their existing terms. This preserves separate row/type identities and creates no fake
`TypeId` values.

The coordinator alternates ordinary inference propagation, queued mixed row equations,
row-shape solving, and emitted field-pair `Equivalent` constraints to a fixed point under
one query budget/cancellation boundary. Its materializer resolves every row field payload
through ordinary inference before canonical row creation; unresolved or cyclic
cross-domain references remain blocked. Cross-domain occurs checking traverses ordinary
terms stored in row fields and row substitutions. Acceptance additionally covers
inference through shared row labels, inferred fields carried by tail extension,
conflicting field values, unresolved rows, cross-domain occurs rejection, and nested
frames that retain an outer-row dependency.

`GenericApplicationSession` becomes a lightweight per-call binding view over the
query-context coordinator; it must not own a row solver. A nested call whose row solution
depends on an unresolved ancestor keeps its result as the existing same-context
`SymbolicInferenceResult` and does not attempt canonical `GenericInstantiation`. The
parent continues solving through the shared coordinator. Materialize only when the
result's row dependencies are concrete, and report underconstraint only at root-context
completion before solver state is dropped. If a call owns the root context, symbolic row
terms cannot escape that context.

### Local STOP / CONSULT triggers

Stop if:

- a required accepted case needs constructor search;
- existing subtype solver architecture cannot isolate static equality from Type
  subtyping without material redesign;
- row/static unification requires merging row solver identity into general inference.

### Checkpoint update

Record first-order static inference and domain-safe `InferVarId(kind)` as durable P2
invariants.

---

## T5 — Generalize static aliases and converge type-member domains

### Purpose

Expose arbitrary static domains through `type` definitions and type members using the
same canonical domain resolver, arguments, and terms established by T2–T4.

### Preconditions

- T4 accepted;
- generalized domain resolver/application/substitution/inference is stable.

### Consumes

```text
TypeAliasDef / TypeAliasInfo
type alias resolver
TypeMemberDef
TypeMemberDomain / TypeMemberArgument
ConstTerm::Int compatibility seam
StaticArgument
KindId
```

### Produces

- static alias result-domain syntax;
- domain-correct static alias bodies;
- canonical non-Type alias terms;
- type-member user-domain parameters/results;
- type-member Int constants backed by canonical `ConstInt`;
- no contradictory second constant/static-domain term universe.

### Files and symbols

**Read/modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs

phalcom-semantic/src/type_alias.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/declarations.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
existing type-member tests
```

### Required implementation shape

#### A. Static alias syntax

Support:

```phalcom
type DefaultShape: Shape = Shape::Scalar
type P: TypePack = T::Positionals
```

Unannotated alias syntax preserves existing Type alias meaning:

```phalcom
type Name = SomeType
```

unless the effective specification explicitly says otherwise.

#### B. Alias body representation

The alias semantic product stores:

```text
declared result kind/domain
canonical static term
dependencies
source
```

A Type-valued alias remains a Type alias.

A Shape-valued alias is a static definition whose `form` is a non-proper TypeId with the
Shape domain kind.

Do not create a parallel `StaticAliasTable` unless the live alias owner architecture
makes one unavoidable and a consultation approves it.

#### C. Type-member parameter/result domains

Generalize type-member declaration parsing/resolution so examples such as:

```phalcom
type Initial: State
type Transform<S: Shape>: Shape
```

use the same static-domain resolver as generic binders.

#### C.1 Declaration product and associated-surface publication (amendment INC-C3-1)

The live checker currently parses `TypeMemberDef` but does not validate or publish it.
Implement the missing semantic declaration path with this ownership split:

- A declaration semantic product owns the validated signature and stable member
  identity. Identity is derived from the declaration owner and canonical selector;
  it must survive snapshots and must not be a store-local numeric `TypeMemberId`.
- `AssociatedSurface` owns name admission, collision decisions, and lookup selection.
  Its type-member entry refers to the stable declaration identity/category; lookup then
  retrieves the checked signature by that identity.
- Do not create a separately queried name-to-signature type-member registry.
  `DeclarationTypeTable` remains nominal type-form metadata and must not own member
  signatures.
- Resolve parameter and result domains through the canonical static-domain resolver.
  Keep member-local binders separate from the owner's generic environment.
- Projection identity must include the selected member identity, specialized owner
  context, and canonical arguments. Intrinsic/domain-native operations retain identity
  distinct from source-declared members.
- T5 implements declaration validation/product publication and focused semantic tests;
  it does not add a competing `::` lookup path. T7 routes lookup and collision selection
  through `AssociatedSurface`.

The type-member source fingerprint must include selector, all parameter/result domain
syntax, implementation kind, and other signature-affecting source. A changed signature
must invalidate its semantic product and dependent surface/projection consumers, while
unrelated members remain reusable.

#### D. ConstInt convergence

If compatibility structures such as:

```text
TypeMemberDomain::ConstInt
ConstTerm::Int
```

remain for local API compatibility, canonicalize them immediately to/from:

```text
ConstInt static domain KindId
TypeData::ConstInt
StaticArgument::Term
```

There must be one semantic identity for the integer static term.

#### E. Type-member projections

Projection formation/substitution/metadata must preserve arbitrary static term arguments
and result domains.

### Forbidden approaches

- no second alias static evaluator;
- no type-member-only nominal static-domain resolver;
- no permanent independent Int term identity;
- no runtime expression evaluation in alias bodies.

### Test changes required

Coverage:

```text
CV-P2-12
CV-P2-13
CV-P2-08
CV-P2-17
CV-P2-22
```

Required cases:

```text
DefaultShape: Shape succeeds
wrong Shape alias body rejects
P: TypePack succeeds
unannotated type alias remains Type-valued
type member parameter over Shape succeeds
type member result Shape succeeds
existing Int-indexed type member uses canonical ConstInt
wrong type-member argument domain rejects
```

### Tests to run now

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generic_application
```

Add the narrow existing type-member test filter discovered from `-- --list`.

### Tests explicitly deferred

Associated-domain and full metadata tests remain deferred until T6–T8.

### Acceptance

T5 passes when aliases and type members speak the same static-domain language as generic
application and no independent constant-domain identity remains semantically observable.

### Local STOP / CONSULT triggers

Stop if:

- existing alias persistence assumes `kind == Type` in a way that requires a new
  parallel alias subsystem;
- type-member semantics require a fundamentally different domain model;
- a compatibility facade cannot be made identity-preserving.

### Checkpoint update

Record static alias and type-member domain convergence as durable interfaces.
Record the declaration-product/`AssociatedSurface` split, stable identity, fingerprint
dependencies, and focused incremental evidence under INC-C3-1.

### Amendment record

```text
Amendment: INC-C3-1 (2026-09-22)
Verdict: PLAN NEEDS CLARIFICATION
Decision: checked signatures belong to declaration semantic products keyed by stable
  owner-plus-canonical-selector identity; AssociatedSurface stores references and is
  the only name-admission, collision, and lookup-selection authority.
Constraint: do not use DeclarationTypeTable for member signatures or add an independently
  queried type-member name registry. T5 publishes; T7 consumes through the surface.
```

---

## T6 — Add associated `type data` declaration identity and generic-owner specialization

### Purpose

Introduce declaration-owned associated static domains on the accepted owner kinds and
give concrete specializations canonical owner environments.

### Preconditions

- T5 accepted;
- static application/substitution can represent heterogeneous owner environments;
- effective source syntax for all required owner categories has been verified.

### Consumes

```text
AlgebraicDef / P1 TypeOnly algebraic semantics
ClassMember / declaration AST
StaticDomainId / StaticDomainData
GenericInstantiation / StaticArgument
semantic declaration identity
```

### Produces

- AST/source representation for associated type-data declarations;
- stable associated static-domain declaration identity;
- canonical concrete associated-domain specialization;
- static constructor declarations owned by the associated domain declaration;
- no runtime inhabitants/invocation targets.

### Files and symbols

**Read/modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs

phalcom-semantic/src/identity.rs
phalcom-semantic/src/algebraic_semantics.rs
phalcom-semantic/src/checker/algebraic_declaration.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/semantic_shard.rs
```

**Tests:**

```text
phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/adts/declarations.rs
new/focused associated type-data tests
```

### Required implementation shape

#### A. Class-owned associated type data

Support at least:

```phalcom
class Parser {
    type data State {
        Start
        Done
    }
}
```

The AST should reuse the canonical P1 `AlgebraicDef`/case payload representation rather
than define a second type-data body grammar.

A class member category such as:

```text
ClassMember::TypeData(...)
```

or an equivalent associated-declaration wrapper is acceptable.

#### B. Runtime-data-owned associated type data

P2 requirements include class/runtime-data owners. The accepted source forms are:

```phalcom
class Parser {
    type data State {
        Start
        Done
    }
}

data Packet(bytes: ByteArray)

impl Packet {
    type data State {
        Unread
        Parsed
    }
}
```

Class-owned declarations appear in the class body. A runtime `data` owner's
representation body remains closed to its components/cases; its associated
`type data` is declared in an unguarded, covering inherent `impl` in the
owner's module. A conformance `impl`, guarded inherent `impl`, or partial-target
inherent `impl` cannot introduce the domain.

The target declaration is the semantic owner in both forms. The inherent
`impl` location is provenance only. Declaration identity is stable owner plus
associated name; canonical owner arguments specialize that declaration into a
concrete domain. Duplicate names across direct declarations and inherent
implementation fragments are invalid. AssociatedSurface remains the authority
for collisions with other associated categories. Associated declarations have
no member-local generic parameters; specialization uses only the canonical
owner generic environment.

The architecture is fixed:

```text
representation-bearing algebraic shape
    is separate from
associated declarations
```

Do not treat `type data` as a product field or sum case.

Associated static cases denote terms in their domain and create no runtime
inhabitants or invocation targets. Cross-specification ownership is recorded
in `docs/specs/type-level-domains.md`, `docs/specs/type-members.md`,
`docs/specs/objects/data-objects.md`, and `docs/specs/objects/impl.md`.

#### C. Declaration identity

Introduce a stable declaration identity equivalent to:

```rust
AssociatedStaticDomainDeclarationId {
    owner: DeclarationId,
    name: Box<str>,
}
```

Exact factoring may reuse a general associated-member ID if that is already canonical.

Do not create synthetic top-level declaration names.

#### D. Concrete specialization

For:

```phalcom
class Container<T: Type, N: Int> {
    type data State { ... }
}
```

the concrete domain:

```text
Container<Int, 3>::State
```

is:

```text
declaration(Container::State)
+
canonical owner environment [Type(Int), ConstInt(3)]
```

The specialization is canonical/interned.

#### E. Constructor/case declaration identity

Static cases belong to the associated domain declaration.

The constructor declaration identity preserves the exact selector/case shape, including:

```text
singleton
explicit empty constructor
positional payload
labeled/record payload
```

Concrete terms carry the concrete domain specialization plus the declaration-owned case
identity.

#### F. Generic owner substitution

When the owner environment contains parameters, substitution reconstructs the concrete
associated domain through the canonical domain interner.

No formatted-owner-string round trip.

### Forbidden approaches

- no synthetic module-level declaration name;
- no per-specialization cloned declarations;
- no string identity;
- no runtime invocation target for associated type-data cases;
- no independent associated-domain generic argument model;
- no source grammar invention if the effective spec is ambiguous.

### Test changes required

Coverage:

```text
CV-P2-14
CV-P2-06
CV-P2-07
CV-P2-19
```

Required tests:

```text
parse and publish class-body Parser::State
parse and publish inherent-impl Packet::State
reject type data inside a runtime data representation body
reject type data inside a conformance impl
reject duplicate associated domain names across inherent impl fragments
Parser::State declaration identity
Parser::State::Start static term
Container<Int>::State canonical specialization
Container<String>::State different specialization
same associated declaration identity across specializations
repeated specialization canonicalizes
two generic-owner specializations have distinct concrete domains
singleton vs explicit empty constructor identity remains distinct
associated term cannot materialize as runtime value
```

### Tests to run now

```sh
RUSTFLAGS='' cargo test -p phalcom-ast -- parser
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations
```

Run the exact new associated type-data test filter.

### Tests explicitly deferred

Do not run the full associated lookup suite until T7 extends `AssociatedSurface`.

### Acceptance

T6 passes when associated static-domain declarations and concrete specializations have
stable canonical identity independent of rendered source strings.

### Local STOP / CONSULT triggers

Mandatory stop if:

- runtime-data owner syntax is ambiguous;
- associated domain identity cannot fit declaration+environment without redesigning
  stable identity;
- implementation would clone declarations per specialization;
- a runtime invocation target appears necessary.

### Checkpoint update

Record associated domain declaration and specialization identity.

---

## T7 — Extend unified `::` lookup and collision authority for static domains and terms

### Purpose

Make `AssociatedSurface` and associated resolution understand associated static domains,
static terms/type members, and existing runtime associated entities under one namespace
authority.

### Preconditions

- T6 accepted;
- associated declarations and specializations exist independently of lookup.

### Consumes

```text
AssociatedSurface
AssociatedFamilyKind
AssociatedMemberId
AssociatedResolution
AssociatedOwnerResolution
type-member lookup
runtime variant/data constructor families
associated static-domain identity
StaticArgument owner environments
```

### Produces

- unified owner/base category discovery;
- associated static-domain resolution result;
- associated static term resolution result;
- heterogeneous owner specialization;
- lookup selection through the published `AssociatedSurface` entry, followed by
  declaration-product retrieval by stable type-member identity;
- collision checks spanning P2 categories;
- no runtime invocation semantics for static results.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/checker/type_member_declaration.rs
phalcom-semantic/src/associated.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/source_index/mod.rs
```

For source type members, T7 must consume the T5 `AssociatedSurface` selection and
resolve its `TypeMemberDeclarationId` to the checked declaration product. It must not
perform a separate name-based lookup or infer an independent category precedence.

The resolver boundary is amended under `INC-C3-3`: type formation uses a
session/query-provided associated-member resolution context. That context selects the
member only through the owner's published `AssociatedSurface`, retrieves the checked
signature by `TypeMemberDeclarationId`, and records dependencies on both the surface
and selected declaration product. The selected stable declaration identity is retained
in the canonical projection. It must not add a separately queried name registry or
perform an intrinsic-then-source fallback chain; intrinsic members remain governed by
their existing intrinsic identity and recognition rules.

The user additionally directed that source-member projection resolution be available
in all declaration annotations, not only body checking or post-publication consumers.
The live session currently lowers aliases before it completes generic headers and
publishes associated surfaces after callable signatures. Therefore the phase order
must be reorganized so the canonical owner/header and surface selection products are
available before every annotation consumer. Do not publish a provisional second
surface or add a parallel name index. Preserve the existing alias/generic-header
dependency contract through one declaration dependency graph. Order acyclic alias
and source-member signature dependencies topologically. A cycle containing source
type-member signatures is invalid under the ratified Type Members rule: withhold the
checked products for every participant and emit one deterministic type-signature-
cycle diagnostic for the component, anchored to its lowest stable declaration
identity. Ordinary stuck projection normalization is not such a declaration cycle.
The shared `AssociatedSurface` remains the sole name-selection authority; the
declaration dependency graph schedules/checks products and does not provide lookup.

**Tests:**

```text
phalcom-semantic/tests/semantic/adts/associated/lookup.rs
phalcom-semantic/tests/semantic/adts/associated/scenarios.rs
phalcom-semantic/tests/semantic/associated/invocation.rs
```

### Required implementation shape

#### A. Generalize associated categories

The exact Rust enums may vary.

Semantically the namespace must represent at least:

```text
runtime case family
runtime data constructor
associated static domain
associated static member/type member
```

Do not force a static domain into `Variant` or `DataConstructor`.

#### B. One collision authority

For one owner/base name, `AssociatedSurface` decides whether categories conflict.

Examples to cover where syntactically possible:

```text
case named State
associated type data State
type member State
class behavior State
```

Collision policy must follow the effective C3 specification.

Do not let static-domain lookup bypass `AssociatedSurface`.

#### C. Heterogeneous associated owner

Generalize:

```text
supplied_arguments: Vec<TypeId>
```

to the canonical static argument/instantiation model.

This is required for:

```text
Container<Int, 3, Shape::Scalar>::State
```

#### D. Static result categories

`AssociatedResolutionKind` or an equivalent semantic result must distinguish:

```text
static domain
static term
runtime value
runtime callable/family
type member projection
```

A static-domain result has no `InvocationTargetId`.

A static term has no runtime invocation target.

Nominal and associated static domains converge before case selection. A nominal
`type data` owner reached through `::` establishes its canonical
`StaticDomainDeclarationId`; the case is selected through the same semantic
static-domain selection machinery as a case reached after associated-domain lookup.
The existing static-term resolution product carries `StaticCaseId` to downstream
consumers. Publish that resolution before runtime-value legality is checked, so an
invalid runtime use can still have a canonical target and the `TypeOnlyRuntimeValue`
diagnostic. Static-domain/case selection records a dependency on the semantic product
that owns the domain's case set. `CallableSourceAttachment` consumes the resolution;
it never searches declaration cases or source spelling.

#### E. Nested `::`

Required:

```text
Parser::State
Parser::State::Done

Container<Int>::State
Container<Int>::State::Done
```

The first lookup returns a domain denotation; the second lookup resolves inside that
static domain.

#### F. Type member coexistence

Existing intrinsic/type-member `::` lookup must remain in the same namespace model and
must not gain a separate priority chain that can disagree with `AssociatedSurface`.

### Forbidden approaches

- no secondary static-domain lookup table queried before/after AssociatedSurface;
- no runtime callable wrapper for static terms;
- no rendered owner strings;
- no “try variant, then type member, then static domain” ad hoc priority chain outside
  the canonical surface.

### Test changes required

Coverage:

```text
CV-P2-14
CV-P2-15
CV-P2-22
CV-P2-19
```

Required cases:

```text
Parser::State
Parser::State::Done
generic owner associated lookup
collision between static domain and existing associated category
missing static domain diagnostic
wrong category diagnostic
ordinary runtime variant/data constructor lookup unchanged
existing type-member lookup unchanged
```

### Tests to run now

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::associated
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic associated::invocation
```

Adapt exact filters after checking `-- --list`.

### Acceptance

T7 passes when every P2-associated `::` category resolves through one canonical
associated surface, with static-domain/static-term results kept distinct from runtime
invocation.

### Local STOP / CONSULT triggers

Stop if:

- collision rules are under-specified;
- lookup ordering requires a second authority;
- existing behavior/intrinsic type-member precedence materially conflicts with accepted
  C3 semantics.

Cycle policy amendment: the user accepted the INC-C3-6 recommendation on
2026-09-23. `docs/specs/type-members.md` now rejects declaration-signature cycles
crossing transparent aliases and source type-member signatures while preserving
valid symbolic projections. T7 must implement deterministic diagnostics and
component invalidation before phase reorganization is complete.

Phase-order finding (INC-C3-6 continuation): even a same-module alias application
`type V: Type = Tensor<Int, Shape::Scalar>` currently reports unresolved `Tensor`,
because aliases are lowered before source nominal generic signatures are built. The
current alias/member graph therefore does not yet satisfy all-annotation availability.
The user accepted adding nominal/trait generic headers to the unified declaration graph.
An SCC containing a source type-member signature uses the ratified deterministic
rejection/withholding rule; alias-only and generic-header-only cycle behavior remains
unchanged. The implementation must preserve exact component/dependent invalidation.

### Checkpoint update

Record unified `::` namespace and heterogeneous associated-owner specialization as
durable interfaces.

---

### Ratified T7 extension — static-case signatures and kind forms

The user ratified the architecture in `INC-C3-10.md` on 2026-09-23. This
amendment extends T7 without changing its lookup authority:

1. Replace transitional expanded `KindSyntax::Named` fields with
   `KindSyntax::StaticDomain { form: Box<TypeAnnotation>, range }`. Resolve the
   form through `resolve_type_form`, preserve applied heterogeneous owner
   arguments, and require the resolved canonical kind to be a static domain.
2. Publish one `StaticCaseSignatureProduct` per constructor-shaped case, keyed
   by stable `StaticCaseId`. Catalog identity remains early and survives invalid
   payload contracts. Singleton getters need no signature; `Foo()` does.
3. Add the product/query/dependency to semantic DB and resolver surfaces, then
   add `StaticCaseSignature` to the existing Alias/Header/Member graph. The
   catalog is never a signature-graph node.
4. Treat each SCC containing Member or StaticCaseSignature as one
   `TypeSignatureCycle`; preserve current alias-only and Header/Alias-only
   classifications. Header recursion from payloads is legal and there is no
   reverse header-to-case-signature edge.
5. Schedule and invalidate case signatures with aliases, headers, and members.
   Static-case selection is catalog-owned; constructor validation is
   signature-owned; TypeStore only interns the selected canonical term.
6. Refactor algebraic builders and annotation/expression consumers to use the
   catalog and checked signature, then verify recursive domains, invalid
   contracts, selected/sibling invalidation, real mixed SCCs, and
   cold/incremental parity.

This amendment does not close T7/T9 or waive T4/T8 acceptance gates.

## T8 — Persist static domains and prove cross-module/incremental equivalence

### Purpose

Make the full P2 semantic graph stable across metadata/export/import and incremental
workspace updates, including associated-domain identity.

### Preconditions

- T7 accepted;
- all P2 semantic categories exist canonically in memory.

### Consumes

```text
KindData::StaticDomain
StaticArgument
TypeData::ConstInt
TypeData::StaticData
heterogeneous TypeData::Applied
scoped static terms
static aliases
associated static-domain declaration/specialization
generic signatures
semantic shard / DB fingerprints
source index
```

### Produces

- lossless stable metadata graph;
- explicit schema migration from completed-P1 baseline;
- metadata validation;
- cross-module imported static-domain identity;
- static-domain/source reference indexing;
- incremental invalidation for P2 edits;
- cold/incremental parity.

### Files and symbols

**Read/modify — metadata:**

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/declaration.rs
phalcom-type-meta/src/bundle.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/tests/schema_compat.rs

phalcom-semantic/src/metadata/export.rs
```

**Read/modify — incremental/module/source index:**

```text
phalcom-modules/src/declaration.rs
phalcom-modules/src/interface.rs

phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/source_index/symbol.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/source_index/mod.rs
```

**Tests:**

```text
phalcom-type-meta/tests/schema_compat.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/incremental/adts.rs
module/import semantic tests
source-index tests
```

### Required implementation shape

#### A. Stable static-domain references

Stable metadata uses declaration identity.

Conceptually:

```rust
enum StaticDomainRef {
    ConstInt,
    Nominal {
        declaration: StableDeclarationRef,
    },
    Associated {
        declaration: StableAssociatedStaticDomainDeclarationRef,
        owner_environment: Box<[StaticArgumentRef]>,
    },
}
```

Indexed graph factoring is encouraged if it reduces duplication.

#### B. Kind graph

`KindNode` must distinguish a static domain from `Type`.

Never export:

```text
StaticDomain -> KindNode::Type
```

#### C. Static arguments

Add stable heterogeneous argument representation:

```rust
enum StaticArgumentRef {
    Term(TypeNodeId),
    RecordRow(...),
}
```

Use the existing stable row representation where possible.

#### D. Type nodes

Stable type-term graph must encode:

```text
ConstInt
StaticData
Applied { origin, StaticArgumentRef[] }
```

Static-data constructor identity must be stable declaration identity, not store-local
numeric ID.

#### E. Scoped nodes

`ScopedTypeNode::Applied` and scoped static-data forms must preserve bound static terms
and heterogeneous row/term categories.

#### F. Generic signatures

A parameter kind referencing `Shape` must round-trip to the exact same stable static
domain declaration identity.

#### G. Aliases

A `Shape`-valued static alias must preserve:

```text
result domain
canonical static term
dependencies
```

#### H. Associated domains

Round-trip:

```text
associated declaration identity
owner environment
static case declaration identity
concrete static term
```

#### I. Schema migration

Inspect the completed P1 schema baseline.

Requirements:

```text
old type-only Applied
    decodes as Term arguments in the new schema when compatibility is supported

new heterogeneous Applied
    never silently downcasts to old Type-only representation

unknown static-domain declaration
    validation failure

raw semantic IDs
    never serialized as stable identity
```

Do not choose compatibility behavior by guess.

#### J. Cross-module import

Provider:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}
```

Consumer:

```phalcom
import geometry { Shape }

data Tensor<S: Shape>()
type V: Type = Tensor<Shape::Vector(3)>
```

must resolve provider declaration/domain identity structurally.

#### K. Incremental invalidation

Cover at least:

```text
type-data domain rename
case add/remove/rename
case payload domain edit
generic binder domain edit
static application argument edit
static alias result-domain/body edit
associated type-data add/remove/rename
associated case edit
generic owner parameter edit
associated owner specialization edit
associated namespace collision introduced/removed
imported provider static-domain edit
```

Final incremental results must equal cold results.

#### L. Source index

Index:

```text
static-domain declaration reference
static case/constructor reference
associated-domain path
associated static-case path
static alias reference
```

For nominal type-data cases, the callable semantic product must publish the selected
`StaticCaseId` even when the enclosing runtime expression is rejected. The runtime
diagnostic and canonical reference target coexist. Case add/remove/rename must
invalidate selection through the static-domain declaration dependency, not through
source-index spelling or an independent case registry.

Editor display taxonomy may remain partially legacy if P3 owns final polish, but exact
semantic targets must be canonical.

### Forbidden approaches

- no raw `StaticDomainId`/`TypeId` persistence;
- no stringified owner environment;
- no StaticDomain→Type compatibility collapse;
- no whole-workspace rescans when existing exact dependency keys can express the edit;
- no source-index target reconstruction by source spelling.

### Test changes required

Coverage:

```text
CV-P2-17
CV-P2-18
CV-P2-14
CV-P2-15
CV-P2-06
```

Required metadata cases:

```text
ConstInt round trip
nominal static domain kind round trip
StaticData round trip
heterogeneous Applied round trip
RecordRow argument round trip
static alias round trip
associated domain specialization round trip
scoped bound static term round trip
schema compatibility rejection/upgrade cases
```

Required incremental cases:

```text
provider case payload change
static argument literal change
associated domain case change
owner generic argument change
collision introduced then removed
incremental == cold
```

### Tests to run now

```sh
RUSTFLAGS='' cargo test -p phalcom-type-meta
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::metadata
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::adts
```

Run the narrow module/import/source-index tests affected by the actual implementation.

### Acceptance

T8 passes when the complete P2 semantic graph survives stable metadata/import and
incremental recomputation without loss, collapse, string identity, or cold/incremental
divergence.

### Local STOP / CONSULT triggers

Stop if:

- schema compatibility policy is ambiguous;
- stable associated-domain identity cannot be represented without changing global
  project identity semantics;
- incremental correctness appears to require a broad workspace scan because the
  canonical dependency graph lacks an identity edge.

### Architecture amendment — schema-v8 static-domain catalog (2026-09-23)

INC-C3-9 closes the metadata schema decision. Schema v8 adds a typed catalog keyed by
stable nominal or associated static-domain declaration identity. Each catalog record
contains the owning declaration's ordered parameter kinds/modes and sorted case names
with ordered payload kinds. Associated specialization arguments stay on the existing
`StaticDomainRef::Associated` and are checked against that declaration shape.

Schemas v1-v7 retain their existing validation behavior. The new catalog field defaults
empty when an older bundle is decoded; schema-v8 bundles require catalog membership and
validate associated owner argument arity/kinds, static-data case membership, payload
arity/kinds, heterogeneous row fields/tails, and global/scoped `Applied` argument and
result kinds. Snapshot-backed export must consume the current canonical nominal
data/enum and associated declaration products. It must not enumerate TypeStore
interning history as the source of currently declared cases. Cross-module catalog
composition, import reconstruction, exact provider invalidation, and cold/incremental
parity remain acceptance requirements.

The current schema/exporter/validator slice compiles. No focused tests were run, so the
T8 implementation remains `UNVERIFIED` and P2 remains partial.

### Checkpoint update

Record schema version/compatibility, cross-module identity, and incremental equivalence
evidence.

---

## T9 — Stabilize P2, prove runtime isolation, update lifecycle state, and hand off to P3

### Purpose

Close the P2 implementation plan with focused semantic regression evidence, runtime
isolation proof, resource/soundness checks, checkpoint updates, walkthrough, and P3
handoff.

### Preconditions

- T1–T8 accepted;
- no open architectural consultation remains unresolved.

### Consumes

All P2 interfaces.

### Produces

- focused P2 verification evidence;
- runtime/compiler isolation evidence;
- performance evidence for type-lambda application;
- updated C3 checkpoint/PROGRAM/STATUS;
- `LANG007.C3.P2-walkthrough.md`;
- `LANG007.C3.P2-handoff.md`;
- next action set to P3 planning/execution according to checkpoint state.

### Files and symbols

**Read/modify:**

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
relevant effective specs only if implementation exposed a documented discrepancy
```

**Create:**

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-walkthrough.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-handoff.md
```

### Required implementation shape

#### A. Runtime isolation audit

Search the final diff for evidence that P2 did **not** create:

```text
new static-domain VM bytecodes
runtime static constructor objects
runtime static unifier
runtime message dispatch for static normalization
```

Compiler changes should be projections/adapters only.

#### B. ProperType audit

Audit all newly generalized API boundaries where a `TypeId` can reach runtime typing:

```text
call parameter/return type
field type
subtyping relation
class hierarchy
dispatch receiver type
runtime layout/lowering
```

Confirm those paths still require proper `Type`.

#### C. Application consumer audit

Ensure every `TypeData::Applied` consumer is intentionally categorized.

No leftover implicit `arguments: TypeId[]` assumption should remain.

#### D. Trait boundary audit

Prove targeted unsupported-domain diagnostics rather than silent evidence mismatch.

#### E. Resource audit

Confirm no full lambda-arena clone on ordinary beta reduction.

Use code-level/targeted regression evidence; do not add heavy benchmark ceremony unless
the live implementation suggests a measurable regression risk beyond the known clone.

#### F. Lifecycle truth

Update:

```text
P2 plan status
completion
verification
durable P2 invariants
schema version if changed
consultations/amendments
deferred failures
next plan/action
```

Do not mark C3 complete. P3 remains.

### Forbidden approaches

- no opportunistic P3 migration;
- no broad unrelated refactors;
- no changing verification state beyond actual evidence;
- no claiming release certification without release gates.

### Tests to run now

Run the G5 final focused gate in §19.

Also:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' cargo check -p phalcom-core
```

If the generalized representation touched compiler integration beyond compile-only
mechanics, run the narrow affected core tests identified by the diff.

### Acceptance

T9 passes when:

```text
P2 implementation is coherent
focused P2 semantic/type-meta/incremental gates pass
runtime static machinery remains absent
proper-Type boundary is intact
resource invariant is satisfied
checkpoint truth is current
walkthrough/handoff are complete
```

### Local STOP / CONSULT triggers

Stop if stabilization reveals:

- a hidden proper-Type boundary weakening;
- runtime static machinery is required to pass an accepted P2 test;
- trait coherence failures are causally introduced by generalized domains;
- incremental results differ from cold after one serious correction.

### Checkpoint update

Final P2 state and next action.

---

# 19. Verification gates

## G1 — P1 prerequisite and domain syntax gate

### Purpose

Prove P2 starts from a truthful P1 foundation and the generalized binder/domain syntax is
correct.

### Run

First run the exact P1 focused gate from the P1 handoff.

Then:

```sh
RUSTFLAGS='' cargo check -p phalcom-ast
RUSTFLAGS='' cargo check -p phalcom-semantic

RUSTFLAGS='' cargo test -p phalcom-ast -- parser
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generics_core
```

### Expected

```text
P1 static-domain foundation remains green
N: Int / S: Shape domain resolution tests pass
existing Type/pack/row binder tests remain green
unsupported arbitrary static-domain trait generic tests fail with the intended diagnostic
```

Do not broaden after PASS.

---

## G2 — Canonical heterogeneous application and checked transformation gate

### Purpose

Prove generalized application, row preservation, substitution, scoped representation,
and beta-reduction resource behavior.

### Run

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generic_application
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations
```

Run the exact C2 pack/row tests named by the P1 handoff if not covered by those filters.

### Expected

```text
heterogeneous application passes
RecordRow remains non-TypeId
static term canonicalization passes
checked substitution failure is observable
type lambda static terms reduce correctly
existing pack/row applications remain unchanged
no whole-arena beta-reduction clone remains in the implementation
```

Do not broaden after PASS.

---

## G3 — Static inference and definition gate

### Purpose

Prove first-order inference, static aliases, and type-member convergence.

### Run

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generic_application
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generics_core
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations
```

Also run the exact existing type-member filter identified via `-- --list`.

### Expected

```text
same-constructor static unification solves payload variables
different constructor/domain refutes
underconstrained static variables remain underconstrained
static aliases preserve declared result domains
type-member Int/user domains use canonical static terms
```

---

## G4 — Associated domain and unified namespace gate

### Purpose

Prove declaration+environment associated identity and one `::` namespace authority.

### Run

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::declarations
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::associated
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic associated::invocation
```

Adapt filters mechanically if module names differ; confirm nonzero selected tests.

### Expected

```text
Parser::State resolves as a static domain
Parser::State::Done resolves as a static term
generic owner specializations preserve one declaration identity + distinct environments
collision tests use AssociatedSurface
runtime variant/data constructor lookup remains unchanged
```

---

## G5 — Metadata, incremental, runtime isolation, and final P2 focused gate

### Purpose

Prove P2 end-to-end without performing P3/workspace certification.

### Run

```sh
cargo fmt --all -- --check

RUSTFLAGS='' cargo test -p phalcom-type-meta

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generic_application
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::generics_core
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::associated
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::metadata
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::adts

RUSTFLAGS='' cargo check -p phalcom-core
```

Run any narrow module/import/source-index suite that T8 changed and that is not exercised
by the commands above.

### Expected

```text
all focused P2 coverage passes
metadata schema/validator tests pass
cross-module imported static identity passes
incremental == cold
core compiler still checks
no VM/runtime static evaluator was introduced
```

### Broad tests

Do not automatically run:

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

P3/checkpoint certification may run them later.

---

## 20. Final focused acceptance

At P2 completion, the walkthrough must map every coverage obligation to concrete test
evidence.

Required mapping template:

| Coverage ID | Test / evidence | Result |
|---|---|---|
| CV-P2-01 | completed P1 gate + T1 preflight | PASS / actual |
| CV-P2-02 | binder-domain semantic tests | PASS / actual |
| CV-P2-03 | heterogeneous generic application | PASS / actual |
| CV-P2-04 | row argument application/substitution | PASS / actual |
| CV-P2-05 | ProperType negative test | PASS / actual |
| CV-P2-06 | repeated canonical formation | PASS / actual |
| CV-P2-07 | static identity inequality | PASS / actual |
| CV-P2-08 | checked substitution failures | PASS / actual |
| CV-P2-09 | scoped/beta reduction | PASS / actual |
| CV-P2-10 | same-constructor inference | PASS / actual |
| CV-P2-11 | constructor/domain refutation | PASS / actual |
| CV-P2-12 | static alias | PASS / actual |
| CV-P2-13 | type-member convergence | PASS / actual |
| CV-P2-14 | associated declaration/specialization | PASS / actual |
| CV-P2-15 | unified `::` collision authority | PASS / actual |
| CV-P2-16 | trait boundary | PASS / actual |
| CV-P2-17 | metadata round trip | PASS / actual |
| CV-P2-18 | incremental/cold parity | PASS / actual |
| CV-P2-19 | runtime-negative boundary | PASS / actual |
| CV-P2-20 | C1/C2 regression | PASS / actual |
| CV-P2-21 | beta-reduction resource invariant | PASS / actual |
| CV-P2-22 | domain-specific diagnostics | PASS / actual |

P2 may be completed as:

```text
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

if the named focused gates pass and no broader release gate was run.

Do not claim `RELEASE_COMPLETE`.

---

## 21. Performance/resource evidence

P2 has real compile-time resource requirements.

### 21.1 Required evidence

Record evidence for:

```text
StaticDomain interning
    hash-consed/cached; no declaration-wide scan per formation

StaticData interning
    work proportional to immediate arguments

Applied flattening
    one flat argument vector rather than nested trees

domain resolution
    canonical IDs after source elaboration; no repeated namespace lookup in solver

associated specialization
    canonical/interened by declaration + owner environment

type-lambda beta reduction
    no whole-arena clone
```

### 21.2 No unnecessary benchmark ceremony

A dedicated benchmark is optional unless implementation evidence indicates a measurable
regression risk.

For `INV-P2-22`, code inspection plus a focused regression/unit test proving no arena
clone path is acceptable evidence.

### 21.3 Performance STOP trigger

Consult if correct implementation appears to require:

```text
linear scan of all static domains for every application
whole-workspace scan for local incremental edits
cloning complete type/static arenas per normalization
exponential/backtracking static inference
```

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

Update C3 checkpoint:

```text
active plan: LANG007.C3.P2
starting revision: completed P1 revision
P1 predecessor verification: recorded
P2 completion: NOT_STARTED / then IN_PROGRESS per repository vocabulary
```

### 22.2 During plan

Update checkpoint only for durable events:

```text
generalized application API established
static inference invariant established
associated-domain identity established
metadata schema migration established
consultation/amendment
new baseline/deferred failure
coherent verification gate completion
```

Do not turn the checkpoint into an edit log.

### 22.3 At plan completion

Record:

```text
P2 status
completion
verification
final revision
generalized application invariant
inference invariant
associated-domain identity invariant
schema version/compatibility
incremental equivalence evidence
runtime isolation evidence
performance/resource evidence
deferred failures
consultations/amendments
next action: LANG007.C3.P3
```

Do not mark C3 complete.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-walkthrough.md
```

It must contain:

1. final revision/state;
2. implementation summary;
3. exact static-domain syntax implemented;
4. generalized application representation/API;
5. `TypeData::Applied` consumer migration summary;
6. instantiation/substitution/scoped-term changes;
7. first-order static inference behavior;
8. alias/type-member convergence;
9. associated type-data identity/specialization;
10. unified `::` namespace behavior;
11. metadata/schema migration;
12. incremental/source-index changes;
13. proper-Type/runtime boundary audit;
14. performance/resource changes;
15. deviations from plan;
16. consultations/amendments;
17. tests added;
18. tests actually run;
19. tests deliberately deferred;
20. residual/baseline failures;
21. exact completion/verification classification.

Do not claim a test/gate that was not completed.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-handoff.md
```

The P2→P3 handoff must include:

```text
current revision
P2 verification state
stable TypeStore static-domain/application interfaces
StaticArgument representation
static inference behavior
static alias/type-member interfaces
associated-domain identity
AssociatedSurface categories
metadata schema/version
cross-module import behavior
incremental invalidation facts
known baseline/deferred failures
P3 objective
must-read files
first recommended commands
things P3 must not redesign
```

Explicitly tell P3 not to:

```text
replace KindId static-domain classification
replace StaticArgument
merge RecordRow into TypeId
reintroduce ConstTerm as independent identity
move static evaluation into the VM
rebuild associated identity from strings
```

---

## 25. Completion truth table

Keep these facts separate:

| Fact | Meaning |
|---|---|
| source written | edits exist |
| P2 tests added | regression surface exists |
| owning crates compile | mechanical integration exists |
| focused tests pass | planned semantic behaviors are evidenced |
| metadata tests pass | stable representation is evidenced |
| incremental tests pass | cold/incremental parity is evidenced |
| core checks | compiler still consumes semantics |
| P2 implemented | all plan deliverables are present |
| P2 focused-tested | required P2 gates passed |
| C3 complete | **false until P3** |
| release certified | only true after explicit release gates |

---

## 26. Plan self-review

Before delivery/execution handoff, verify all of the following.

### Architecture

- [ ] P1 corrective work is not hidden in P2.
- [ ] `KindId` remains the only classifier.
- [ ] `StaticArgument` is the only generalized application envelope.
- [ ] RecordRow remains outside TypeId.
- [ ] `ProperTypeId` boundary is explicit.
- [ ] static terms use structural compiler identity only.
- [ ] no runtime evaluator/solver is planned.
- [ ] associated specialization is declaration + environment.
- [ ] `AssociatedSurface` remains namespace authority.
- [ ] trait arbitrary-static-domain behavior is explicitly bounded.

### Luna executability

- [ ] every task has preconditions;
- [ ] every task names consumed/produced interfaces;
- [ ] file areas are named;
- [ ] forbidden approaches are explicit;
- [ ] local STOP/CONSULT triggers exist;
- [ ] tasks form a dependency order.

### Testing

- [ ] coverage IDs map to important invariants;
- [ ] build testing is intentionally sparse;
- [ ] broad suites are not required after each task;
- [ ] filters must select actual tests;
- [ ] final focused gate covers semantics, metadata, incremental behavior, and compiler integration;
- [ ] P3/workspace certification is not accidentally pulled into P2.

### Lifecycle

- [ ] C3 checkpoint remains shared state;
- [ ] walkthrough is required;
- [ ] handoff is required;
- [ ] P2 does not claim C3 closure;
- [ ] next action is P3.

### Residual planning risks

The executor must pay particular attention to:

```text
runtime-data-owned associated type-data source grammar
trait generic boundary
variance/relation consumers of heterogeneous Applied
metadata schema compatibility from the completed-P1 baseline
source-index taxonomy versus P3 tooling ownership
```

Those are not permission to improvise. Use the STOP/CONSULT rules when the live tree or
effective specification makes them materially ambiguous.

---

# 27. Implementation sequence summary

Execute in this order:

```text
T1
    verify truthful P1 baseline
    activate P2
        ↓
T2
    static-domain binder syntax/resolution
    static generic argument syntax/elaboration
        ↓
T3
    generalized Applied / StaticArgument
    instantiation
    checked substitution
    scoped type functions
    beta-reduction resource fix
        ↓
T4
    first-order static-domain inference
        ↓
T5
    static aliases
    type-member convergence
        ↓
T6
    associated type-data declaration identity
    generic-owner specialization
        ↓
T7
    unified AssociatedSurface / ::
        ↓
T8
    stable metadata/imports
    incremental/source index
        ↓
T9
    final focused stabilization
    runtime/proper-Type/resource audit
    checkpoint update
    walkthrough + P3 handoff
```

The implementation order is intentional.

Do not begin inference before canonical generalized application/substitution exists.

Do not begin associated-domain lookup before associated-domain declaration/specialization
identity exists.

Do not finalize metadata before every in-memory P2 semantic category is canonical.

Do not start P2 at all until P1 is truthful.
