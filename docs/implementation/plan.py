from pathlib import Path
from textwrap import dedent

outdir = Path("LANG007/LANG007.C3")

req = dedent(r'''
---
id: LANG007.C3.requirements-analysis
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: requirements-analysis
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
title: "Type Data and Unified Algebraic Static Domains"
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
depends_on:
  - LANG007.C1
  - LANG007.C2
plans:
  - LANG007.C3.P1
  - LANG007.C3.P2
  - LANG007.C3.P3
---

# LANG007.C3 — Requirements Analysis

## Type Data and Unified Algebraic Static Domains

## 0. Executive conclusion

`LANG007.C3` should implement one semantic algebra for Phalcom declarations and static domains across three bounded plans:

```text
P1
    unify runtime algebraic declarations under `data`
    +
    establish `type data` declaration/case/static-term foundations

P2
    generalize static-domain classification, heterogeneous generic application,
    static inference, associated `type data`, and `::` lookup

P3
    expose intrinsic structural domains through Universe,
    migrate canonical library declarations and tooling/corpus,
    certify metadata/incremental/runtime boundaries,
    close C3
```

The live post-C2 repository strongly favors **extending the existing canonical kinded-term architecture rather than introducing a second general static-term universe**.

The central implementation recommendation is:

```text
TypeId
    remains the store-relative canonical handle for ordinary kinded static terms

ProperTypeId
    remains the proof that a TypeId is specifically a proper Type

KindId / KindData
    remains the compact classifier used by generic application and inference
    and is generalized with ONE dynamic nominal-static-domain form

StaticDomainId
    identifies a user-defined or specialized nominal static domain

TypeData
    gains canonical closed static terms such as const/static-data constructor terms

StaticArgument
    is the heterogeneous generic/application envelope
    ├── Term(TypeId)
    └── RecordRow(RecordRowId)
         (or an exact row-term equivalent reusing the current row subsystem)

RecordRow
    remains its existing dedicated canonical/solver domain

InferVarId
    remains the single query-local inference-variable identity for all non-row domains,
    classified by KindId

RecordRowVarId / row inference
    remains the row-specific solver authority
```

This is effectively the attached specification's "extend canonical type-level term storage" option, refined for the live repository's deliberate row split. It does **not** make arbitrary static terms proper types; `ProperTypeId` continues to enforce that boundary.

C3 must also **reuse the mature runtime sum/ADT/GADT implementation**. Replacing `enum` with sum-shaped `data` is a source/semantic ownership migration, not an authorization to rewrite exact-case identity, GADT evidence, exhaustiveness, case dispatch, or runtime representation.

---

# Part I — Acceptance objective and authority

## 1. Checkpoint acceptance objective

`LANG007.C3` is accepted when Phalcom has one coherent algebraic declaration and static-domain model satisfying all of the following:

1. `data` is the sole canonical runtime algebraic declaration keyword for products, closed sums, ADTs, and GADTs.
2. `enum` has no distinct semantic declaration category.
3. `type data` introduces closed nominal type-system-only domains with canonical static inhabitants and no runtime inhabitants.
4. User-defined `type data` domains classify generic/static parameters.
5. Generic/static applications can preserve heterogeneous arguments without coercing them into fake proper types.
6. Associated `type data` declarations work for the accepted nominal owner categories and participate in the canonical `::` associated namespace.
7. Existing type-member result-domain semantics operate over user-defined static domains.
8. Runtime exact-case, GADT, pattern/exhaustiveness, inherent-case behavior, and conformance semantics remain intact under sum-shaped `data`.
9. `TypePack`, `LabeledTypePack`, `TupleEntries`, and `RecordRow` can be exposed as `intrinsic type data` without replacing C1/C2 canonical terms.
10. `Unit`, `Tuple`, and `Record` can be exposed coherently as `intrinsic data` without forcing general data-object runtime representation.
11. `*`, `**`, and `***` retain their C2 positional, labeled, and complete-lane meanings.
12. Metadata, imports, source identity, and incremental analysis preserve canonical static-domain and static-term identity.
13. LSP/source indexing understands the new declarations and associated lookup paths.
14. Static/type-data normalization remains compiler-semantic work; the VM does not become a general static-term solver.
15. Current authoritative specifications no longer contradict the unified model.
16. Focused and corpus-level regressions show no semantic loss in existing product, ADT/GADT, generic, associated-type, pack, Tuple, Record, or runtime behavior.

This checkpoint is larger than a syntax migration. It changes the semantic classification model used by generics, static arguments, associated lookup, metadata, and tooling.

---

## 2. Normative and planning authority

### 2.1 Desired language behavior

The C3 specification supplied for this checkpoint establishes these fixed design directions:

```text
data
    runtime algebraic domain

type data
    type-system-only algebraic/static domain

shape
    Product | Sum

implementation provenance
    independent from phase

ownership
    module | associated nominal owner

intrinsic data
    compiler-owned runtime algebraic declaration

intrinsic type data
    compiler-owned static domain declaration
```

It additionally fixes:

```text
enum
    not a canonical source declaration category

*
    positional lane

**
    labeled lane

***
    complete two-lane pack

...
    not generic/type-pack syntax
```

The attached C3 specification is the semantic design authority for planning this checkpoint. Before implementation is considered complete, its accepted semantics must be migrated into the owning `docs/specs/**` documents.

### 2.2 Inherited accepted architecture

`LANG007.C2` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` at the live post-C2 baseline.

C3 inherits and must not redesign:

- C1/C2 kinded `TypeId` canonical terms;
- `ProperTypeId`;
- `RecordRowId` and row-specific inference;
- `TypePack`, `LabeledTypePack`, `CompleteTypePack`;
- flat positional/labeled concat;
- `TupleFromPacks`;
- `CallableFromPacks`;
- `GenericParameterMode`;
- binder-aligned generic applications;
- `GenericBinderHeader`;
- checked substitution/materialization;
- whole-pack `InferVarId(KindId)` inference;
- schema-v4 scoped pack metadata;
- no pack sequence decomposition;
- no runtime pack execution.

### 2.3 Live repository authority

This analysis was grounded against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   62a6082ab463387c812d558e5561508267ecc63c
subject:    feat(lang007): implement whole-pack inference, persistence, and close C2 (LANG007.C2.P3)
```

Implementation plans must re-read the relevant live paths before editing.

Mechanical drift may be adapted.

Architectural drift requires consultation.

---

## 3. Repository lifecycle state

### 3.1 C2 is complete

The C2 handoff and walkthrough establish:

```text
status:       COMPLETE
completion:   IMPLEMENTED
verification: FOCUSED_TESTED
```

All C2 P3 gates passed.

### 3.2 Program records contain future-facing drift

At the planning baseline, `PROGRAM.md` / `STATUS.md` still contain stale future-facing C2 language in some sections even though the C2 checkpoint itself is complete.

Examples of drift include old next-plan references and pre-ratification pack wording.

This is not a semantic blocker for C3 planning.

Required response:

- C3 documents become the new checkpoint planning authority.
- `LANG007.C3.P1` must rebaseline the active `PROGRAM.md` / `STATUS.md` lifecycle state at plan start.
- Historical completed records must not be rewritten merely to make their old wording look current.

---

# Part II — Live repository analysis

## 4. Current declaration split

The live source still has a hard AST distinction:

```rust
Statement::Enum(EnumDef)
Statement::Data(DataDef)
```

### 4.1 Current `DataDef`

`DataDef` is runtime-product-only.

Its shape is:

```rust
enum DataShapeSyntax {
    Tuple { components, ... },
    Record { components, ... },
}
```

Its semantic product is centered on:

```text
DataInfo
DataShape::{Tuple, Record}
DataComponentSemantic
DataConstructorSignature
DataConstructorId
DataSemanticTable
DataDeclarationProduct
```

This implementation is valuable and must be preserved as the product branch of the new unified algebraic declaration model.

### 4.2 Current `EnumDef`

`EnumDef` is the mature runtime closed-sum/GADT path.

The current sum model includes:

```text
VariantDecl
VariantId
VariantFamilyId
VariantFieldId
EnumInfo
VariantInfo
VariantShape
VariantConstructorSignature
CaseTypeEnvironment
ExactCase TypeData
EnumSemanticTable
EnumDeclarationProduct
EnumRequirementsProduct
pattern/exhaustiveness products
runtime lowering/reification products
```

This is not disposable legacy code.

C3 must move **semantic ownership** from "`enum` declaration" to "runtime sum-shaped `data` declaration" while retaining the exact-case/GADT machinery.

### 4.3 Current sum syntax is not yet C3 syntax

The live `VariantDecl` still assumes the pre-C3 enum surface:

- variant marker state is explicit;
- tuple payload is the primary payload representation;
- many parser/tests branch on `Statement::Enum`;
- runtime/lowering paths access `enum_def.variants`.

C3 P1 therefore owns a real parser/AST migration, not just token substitution.

---

## 5. Current canonical type-level term model

The live model is already more general than the name `TypeId` suggests.

The source describes `TypeId` as a:

> store/snapshot-local canonical identifier for a type-level form.

A `TypeId` may represent:

```text
proper Type
higher-kinded form
TypePack
LabeledTypePack
CompleteTypePack
symbolic pack concat
type lambda
type member projection
other non-proper kinded terms
```

`ProperTypeId` explicitly proves:

```text
kind == Type
```

This is the crucial C3 seam.

### 5.1 Consequence

C3 should **not** introduce an independent, all-purpose `StaticTermId` arena merely because user-defined static terms are new.

Doing so would duplicate:

- interning;
- kind/domain classification;
- substitution;
- inference;
- metadata export;
- fingerprinting;
- scoped lambda handling;
- normalization.

### 5.2 Required extension

The type-level term universe should be generalized so a `TypeId` may also denote:

```text
compile-time constant term
type-data constructor/case term
symbolic type-data term containing parameters
```

while `ProperTypeId` continues to reject every such non-`Type` term.

---

## 6. Current `KindData` and generic classifier model

Current base kinds:

```rust
KindData::Type
KindData::RecordRow
KindData::TypePack
KindData::LabeledTypePack
KindData::CompleteTypePack
KindData::Arrow { parameters, result }
```

Generic parameter state is currently centered on:

```text
TypeParameterData.kind: KindId
GenericBinderHeader.parameter_kinds
GenericSignature.parameter_kinds
InferenceVariable.kind
```

`GenericParameterMode` is already orthogonal.

This architecture is a strong fit for C3 if generalized carefully.

### 6.1 Required decision

Do **not** create one compiler enum case per user `type data` declaration.

Instead add one dynamic nominal-domain classifier:

```rust
KindData::StaticDomain(StaticDomainId)
```

or an exact equivalent.

The open-ended identity is in `StaticDomainId`, not in the closed Rust enum.

This satisfies both requirements:

```text
user-defined static domains are nominal/open-ended

and

existing generic/inference machinery remains KindId-directed
```

### 6.2 `StaticDomainId`

Conceptually:

```rust
pub struct StaticDomainId(u32);

pub enum StaticDomainData {
    // primitive compiler-defined constant domain, initially Int
    ConstInt,

    // module-scope type data
    Nominal {
        declaration: DeclarationId,
    },

    // P2 associated-domain specialization
    Associated {
        declaration: AssociatedStaticDomainDeclarationId,
        owner_environment: StaticInstantiation,
    },
}
```

Exact Rust layout is not normative.

Important law:

```text
StaticDomainId
    is canonical store-relative identity

stable metadata
    uses declaration/environment stable references
    not the raw integer
```

### 6.3 Existing structural domains

Do not re-encode:

```text
Type
RecordRow
TypePack
LabeledTypePack
CompleteTypePack
```

as nominal user-domain nodes internally.

Their existing optimized `KindData` forms remain canonical.

P3 `intrinsic type data` declarations are the **source-visible declaration surface over those existing domains**.

---

## 7. Current heterogeneous seams

C3 does not begin from zero.

Type-member application already distinguishes:

```rust
TypeMemberArgument::Type(TypeId)
TypeMemberArgument::Const(ConstTerm)
TypeMemberArgument::RecordRow(RecordRowId)
```

and domains include:

```text
Kind(KindId)
ConstInt
Type(TypeId)
```

This proves the semantic system already has a heterogeneous static-argument problem, but only in a bounded subsystem.

Generic type application is still narrower:

```rust
TypeData::Applied {
    origin: TypeId,
    arguments: Box<[TypeId]>,
}
```

`GenericInstantiation` is:

```text
type_bindings: TypeParameterId -> TypeId
row_bindings:  TypeParameterId -> RecordRowId
```

This is the seam P2 must generalize.

---

## 8. Current row boundary

`RecordRow` is deliberately separate:

```text
RecordRowId
RecordRowData
RecordRowTail::{Closed, Parameter, Projection}
RecordRowSolver / row inference
```

`type_level_binding_for_parameter` special-cases row parameters instead of constructing a `TypeData::Parameter`.

This is an accepted pre-C3 architecture.

### 8.1 C3 requirement

Do not collapse row identity into `TypeId` merely to make heterogeneous applications easier.

Instead introduce a generic/static application envelope:

```rust
pub enum StaticArgument {
    Term(TypeId),
    RecordRow(RecordRowId),
}
```

or an exact equivalent that also represents the existing symbolic row-tail forms through the canonical row subsystem.

For most domains:

```text
Type
const Int
type-data domain
TypePack
LabeledTypePack
CompleteTypePack
higher-kinded term
```

the argument is `Term(TypeId)`.

For `RecordRow`, the argument remains row-owned.

This is the minimum representation generalization that supports C3 without undoing C1.

---

# Part III — Architectural decisions

## 9. Decision register

| ID | Decision | Status |
|---|---|---|
| C3-D01 | One canonical algebraic declaration model with orthogonal phase and shape | FIXED |
| C3-D02 | Final canonical runtime keyword is `data`; `enum` is compatibility-only during migration | FIXED |
| C3-D03 | `type data` is semantic phase `TypeOnly`, not a parser-only flag | FIXED |
| C3-D04 | Reuse exact-case/GADT/exhaustiveness machinery for runtime sums | FIXED |
| C3-D05 | `TypeId` remains canonical handle for non-row static terms | RECOMMENDED/FIXED FOR PLANNING |
| C3-D06 | `ProperTypeId` remains the proof boundary for actual types | FIXED |
| C3-D07 | User nominal static domains use one dynamic `KindData::StaticDomain(...)` class, not one Rust enum variant per declaration | FIXED |
| C3-D08 | Generic application uses `StaticArgument` envelope; RecordRow remains separate | FIXED |
| C3-D09 | Compile-time Int becomes a canonical static term/domain usable by generic application; no runtime equality participates | FIXED |
| C3-D10 | `GenericParameterMode` remains orthogonal to static domain | FIXED |
| C3-D11 | `InferVarId` remains the non-row inference identity; its `KindId` may denote a nominal static domain | FIXED |
| C3-D12 | First-order type-data constructor unification only; no general compile-time evaluator | FIXED |
| C3-D13 | Associated `type data` is inherent declaration ownership, not trait associated-type evidence | FIXED |
| C3-D14 | Associated static namespace extends the existing `AssociatedSurface` authority | FIXED |
| C3-D15 | Trait generic parameters over new user static domains are not required for C3; reject them unless migrated safely by the ordinary generic path | SCOPE BOUNDARY |
| C3-D16 | `TupleEntries` maps to the C2 complete-pack domain | FIXED |
| C3-D17 | `intrinsic data Tuple/Record/Unit` is a semantic source facade; runtime physical representation remains specialized | FIXED |
| C3-D18 | Stable metadata records declaration/domain identity structurally; never source strings | FIXED |
| C3-D19 | Legacy `enum` and runtime `data` compatibility parsing must never create different semantic identities | FIXED |
| C3-D20 | Public/new APIs converge to algebraic/domain/case terminology; internal Enum/Variant compatibility names may survive temporarily | FIXED |

---

## 10. Unified algebraic declaration architecture

### 10.1 Canonical source AST

Target conceptual AST:

```rust
pub struct AlgebraicDef {
    pub name: String,
    pub name_range: SourceRange,
    pub phase: AlgebraicPhase,
    pub implementation: ImplementationKind,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub shape: AlgebraicShapeSyntax,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,

    // compatibility/source presentation only, if retained during migration
    pub legacy_enum_keyword_range: Option<SourceRange>,
}

pub enum AlgebraicPhase {
    Runtime,
    TypeOnly,
}

pub enum AlgebraicShapeSyntax {
    Product(ProductShapeSyntax),
    Sum(SumShapeSyntax),
}

pub enum ProductShapeSyntax {
    Tuple {
        components: Vec<DataComponentSyntax>,
        constructor_visibility: Visibility,
        ...
    },
    Record {
        components: Vec<DataComponentSyntax>,
        constructor_visibility: Visibility,
        ...
    },
}

pub struct SumShapeSyntax {
    pub cases: Vec<AlgebraicCaseSyntax>,
    ...
}

pub enum CasePayloadSyntax {
    Singleton,
    Tuple {
        parameters: Vec<ParameterDef>,
        explicit_empty: bool,
    },
    Record {
        components: Vec<DataComponentSyntax>,
    },
}
```

Exact names are flexible.

Semantic distinctions are not.

### 10.2 Statement category

Final semantic AST should have one runtime/type-only algebraic declaration path.

Preferred endpoint:

```rust
Statement::Data(AlgebraicDef)
```

with `phase` distinguishing `data` and `type data`.

A separate `Statement::TypeData` is mechanically acceptable only if both lower immediately to the same `AlgebraicDef` and no semantic consumer branches on distinct declaration categories.

Final C3 must not retain:

```text
Statement::Enum -> one semantic world
Statement::Data -> another semantic world
```

### 10.3 Legacy `enum`

P1 may retain compatibility parsing:

```phalcom
enum Option<T> { ... }
```

but it must lower to:

```text
AlgebraicPhase::Runtime
AlgebraicShape::Sum
```

using exactly the same case identities and products as:

```phalcom
data Option<T> { ... }
```

The legacy keyword span may be retained only for migration diagnostics/presentation.

It may not enter:

- declaration identity;
- type identity;
- fingerprints;
- metadata;
- lowering decisions.

P3 removes compatibility acceptance after corpus/Universe migration unless a user-visible compatibility policy explicitly extends it.

---

## 11. Product/sum discrimination

Braced `data`/`type data` bodies must be syntactically classified without semantic lookup.

Required interpretations:

```phalcom
data Person {
    name: String
    age: Int
}
```

→ record product.

```phalcom
data State {
    Ready
    Failed(_ reason: Error)
}
```

→ sum.

```phalcom
data Impossible {}
```

→ empty sum.

```phalcom
data UnitLike()
```

→ zero-component product.

Mixed root body:

```phalcom
data Invalid {
    value: Int
    None
}
```

must produce a dedicated mixed-algebraic-body diagnostic.

Do not choose meaning by first member encountered.

---

## 12. Runtime case syntax

P1 must preserve the semantic distinction:

```phalcom
data Example {
    Singleton
    Constructed()
}
```

Meaning:

```text
Singleton
    associated singleton value

Constructed()
    zero-argument constructor family
```

They are not interchangeable merely because both carry zero payload fields.

Record-payload cases are new required surface:

```phalcom
data Message {
    Request {
        method: Method
        path: String
    }

    Closed
}
```

The existing tuple-payload/GADT machinery should be generalized to consume a neutral case payload abstraction.

---

## 13. Semantic declaration authority

Introduce one authoritative semantic product or facade:

```rust
pub struct AlgebraicDeclarationInfo {
    pub owner: DeclarationId,
    pub phase: AlgebraicPhase,
    pub implementation: ImplementationKind,
    pub generic_signature: Option<GenericSignature>,
    pub representation: AlgebraicRepresentation,
    pub source: Option<SemanticSourceSpan>,
}

pub enum AlgebraicRepresentation {
    Product(AlgebraicProductInfo),
    Sum(AlgebraicSumInfo),
}
```

### 13.1 Product branch

Product info derives the current facts:

```text
DataShape
components
DataConstructorSignature
constructor visibility
result type template
```

### 13.2 Runtime sum branch

Sum info derives/reuses:

```text
EnumInfo
VariantInfo
VariantConstructorSignature
CaseTypeEnvironment
exact-case templates
requirements/default behavior
```

### 13.3 Type-only branch

Type-only product/sum uses the same declaration/case identities but produces static constructor terms rather than runtime constructors/exact runtime values.

### 13.4 Transitional adapters

To keep P1 implementable, existing tables may remain as compatibility projections:

```text
DataSemanticTable
EnumSemanticTable
```

but they must be populated from the authoritative algebraic product.

Forbidden:

```text
build AlgebraicDeclarationInfo
and separately rebuild EnumInfo from AST
and separately rebuild DataInfo from AST
```

There must be one semantic parse/lowering truth.

---

## 14. Case identity migration

Current `VariantId` is already declaration-owned and selector-based.

C3 does not need a second independent case identity.

Recommended migration:

```rust
pub struct AlgebraicCaseId {
    owner: DeclarationId,
    selector: Selector,
}

// compatibility alias during migration
pub type VariantId = AlgebraicCaseId;
```

or an equivalent neutral facade preserving existing stable identity.

This has two benefits:

1. runtime exact-case machinery continues to refer to the same identity;
2. type-data cases can use the neutral case identity without implying runtime callability.

Runtime-only products may continue to use names such as `VariantConstructorId` temporarily.

New stable metadata and new source-facing APIs should use `case`, not `enum variant`, unless referring to legacy compatibility.

---

# Part IV — Static domains and canonical terms

## 15. Static-domain classifier architecture

### 15.1 Built-in domains

Existing:

```text
Type
RecordRow
TypePack
LabeledTypePack
CompleteTypePack
```

remain current optimized `KindData` variants.

### 15.2 Nominal domains

Add one open-ended classifier:

```rust
KindData::StaticDomain(StaticDomainId)
```

A different `type data` declaration receives a different `StaticDomainId`, hence a different `KindId`.

Example:

```phalcom
type data Width(_ value: Int)
type data Height(_ value: Int)
```

must produce:

```text
domain(Width) != domain(Height)
```

even though constructor structures match.

### 15.3 Associated domains

P2 extends `StaticDomainData` to include owner specialization.

For:

```phalcom
class Container<T: Type> {
    type data State {
        Empty
        Present(_ value: T)
    }
}
```

the domain family declaration identity is:

```text
Container::State
```

while concrete domains include:

```text
Container<Int>::State
Container<String>::State
```

These are distinct domain specializations sharing one declaration identity.

The owner environment must be canonical semantic data, not source text.

---

## 16. Canonical static terms

### 16.1 Compile-time constants

C3 should normalize supported compile-time constants into the same canonical term graph used by generic application.

Initial required primitive:

```text
Int
```

Conceptual form:

```rust
TypeData::ConstInt(i64)
```

with a non-`Type` static-domain kind.

This does not turn runtime integer values into type-level terms automatically.

Only syntax/contexts accepted by the static language form these terms.

### 16.2 Type-data terms

Conceptual representation:

```rust
pub struct AlgebraicStaticTerm {
    pub domain: StaticDomainId,
    pub constructor: AlgebraicStaticConstructorId,
    pub arguments: Box<[StaticArgument]>,
}

TypeData::StaticData(AlgebraicStaticTerm)
```

A product-shaped `type data` has one primary static constructor identity.

A sum-shaped `type data` has one case constructor/singleton identity per declared case.

Canonical identity includes:

```text
domain declaration identity
constructor/case identity
owner specialization when associated
canonical static arguments
```

Runtime equality/hashing/method dispatch does not participate.

### 16.3 Symbolic terms

A well-formed static term containing a parameter remains canonical/symbolic:

```text
Shape::Vector(N)
```

where `N` is a parameter of the compile-time Int domain.

It must not become:

```text
Dynamic
Never
unknown proper Type
```

merely because `N` is unresolved.

---

## 17. `ProperTypeId` boundary

This boundary is mandatory.

For:

```phalcom
Shape::Vector(3)
```

the canonical term may use a `TypeId` handle internally, but:

```rust
store.proper_type(shape_vector_3)
```

must fail because its kind/domain is not `Type`.

Every API that publishes ordinary value-type knowledge must continue to require or validate `ProperTypeId`.

This prevents the implementation mistake:

```text
"stored in TypeId"
    therefore
"is a runtime type"
```

which is false already for packs and becomes even more important in C3.

---

## 18. Heterogeneous generic/static arguments

### 18.1 Target representation

Generalize canonical application:

```rust
pub enum StaticArgument {
    Term(TypeId),
    RecordRow(RecordRowId),
}
```

Conceptually:

```text
StaticArgument::Term
    Type
    type constructor
    const Int
    type-data term
    TypePack
    LabeledTypePack
    CompleteTypePack / TupleEntries
    other TypeStore-kind terms

StaticArgument::RecordRow
    canonical row-domain term
```

### 18.2 `TypeData::Applied`

Target:

```rust
TypeData::Applied {
    origin: TypeId,
    arguments: Box<[StaticArgument]>,
}
```

The result remains a `TypeId` because an application itself is a canonical kinded/static term.

`TypeStore::apply_*` computes each argument's classifier:

```text
Term(t)      -> kind_of(t)
RecordRow(r) -> KindId::RECORD_ROW
```

and applies the origin's arrow domain contract.

### 18.3 Compatibility helpers

Preserve narrow helpers for common type-only callers:

```rust
apply_type_form(origin, &[TypeId])
```

may wrap each argument as `StaticArgument::Term`.

Introduce a canonical generalized API such as:

```rust
apply_static_form(origin, &[StaticArgument])
```

or equivalent.

Do not force every existing type-only caller to manually construct the envelope.

### 18.4 Scoped/lambda representation

P2 must provide an equivalent scoped argument form so aliases/type lambdas can preserve heterogeneous arguments:

```rust
ScopedStaticArgument::Term(ScopedTypeId)
ScopedStaticArgument::RecordRow(ScopedRecordRowTerm)
```

Exact row-scoped representation should reuse existing `ScopedRecordTail` / row machinery.

Do not stringify or lower rows into fake TypeIds.

---

## 19. Generic parameter contracts

### 19.1 Domain remains separate from mode

For:

```phalcom
P: TypePack
```

the domain is TypePack and mode is Single.

For:

```phalcom
*Ts
```

the domain is TypePack and mode is PositionalPack.

C2 law remains:

```text
domain != binder mode
```

### 19.2 Reuse `KindId`

Because nominal static domains are representable as dynamic `KindData::StaticDomain`, current generic parameter products may continue using:

```text
TypeParameterData.kind
GenericBinderHeader.parameter_kinds
GenericSignature.parameter_kinds
```

as the canonical compact classifier.

Do not add an independent `parameter_domain` field carrying the same identity unless `kind` becomes explicitly derived and non-authoritative.

One source of truth is required.

### 19.3 Variance

Pack binder invariance remains.

User type-data/static constant parameters should initially be invariant unless an existing variance rule demonstrably applies.

C3 does not introduce a variance theory for arbitrary static domains.

---

## 20. Static generic argument syntax

Current generic application syntax accepts type annotations plus C2 pack expansion.

P2 must generalize the *single argument* grammar to a static argument expression category.

Conceptual AST:

```rust
pub enum GenericArgumentSyntax {
    Single(StaticArgumentSyntax),
    Expansion(TypePackExpansionSyntax),
}

pub enum StaticArgumentSyntax {
    TypeForm(TypeAnnotation),
    IntLiteral { value, range },
    StaticConstructor(StaticTermSyntax),
    // row syntax uses the existing row/type annotation representation
}
```

The exact factoring may instead make `TypeAnnotation` a broader static-term syntax tree.

Requirement:

```text
Tensor<Float, Shape::Vector(3)>
Integer<32, Endian::Big>
```

must parse without treating `3`, `32`, or `Endian::Big` as runtime expressions.

Do not reuse arbitrary runtime `Expr` as the static argument grammar; doing so would accidentally authorize compile-time execution.

---

# Part V — Type data

## 21. `type data` declaration semantics

A `type data` declaration:

- has nominal declaration identity;
- has a `StaticDomainId`;
- may be product or sum;
- has canonical constructor/case identities;
- has no runtime constructor target;
- has no runtime values;
- has no VM class/object requirement;
- may appear as a generic/static parameter domain;
- participates in substitution, inference, metadata, imports, source index, and associated lookup;
- is closed;
- uses compiler-defined structural identity.

### 21.1 Static payload validity

Every type-data payload component must have a supported static domain.

Valid examples:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

type data TypeShape {
    One(_ element: Type)
    Many(_ elements: TypePack)
}
```

A payload requiring an ordinary runtime value with no static-term semantics is invalid.

### 21.2 Product term

```phalcom
type data Coordinate(
    _ x: Int,
    _ y: Int
)
```

forms canonical static terms only in static contexts:

```phalcom
Coordinate(3, 4)
```

### 21.3 Sum term

```phalcom
type data Endian {
    Little
    Big
}
```

forms:

```text
Endian::Little
Endian::Big
```

### 21.4 No runtime construction

A runtime binding:

```phalcom
let x: Endian = Endian::Little
```

is invalid because `Endian` is not a runtime value type.

The compiler must reject it before lowering.

---

## 22. Type-data GADTs

C3 includes type-data GADT syntax/identity where payloads/results remain in the supported static language.

Example:

```phalcom
type data StaticExpr<T: Type> {
    Int(_ value: Int) -> StaticExpr<Int>
    Bool(_ value: Bool) -> StaticExpr<Bool>
}
```

Required semantics:

- same-declaration result specialization;
- case-local equalities;
- constructor applicability;
- structural static-term inference;
- stable metadata.

Not required:

- arbitrary user-defined type-level match/evaluator;
- recursive normalization functions over `StaticExpr`.

---

# Part VI — Inference and substitution

## 23. Reuse `InferVarId`

Current inference variables already carry:

```text
InferVarId
kind: KindId
solution: TypeId
```

With dynamic nominal static-domain kinds, P2 can reuse this architecture.

For:

```phalcom
S: Shape
```

instantiate:

```text
InferVarId(kind = KindId(StaticDomain(Shape)))
```

No `StaticInferVarId` is needed.

### 23.1 Row exception

Row parameters continue through row inference.

Do not force them into the non-row `InferVarId -> TypeId` solution map.

---

## 24. Type-data unification

Required first-order laws:

### Same constructor

```text
Shape::Vector(N)
=
Shape::Vector(3)

=> N = 3
```

when `N` has the compile-time Int domain.

### Different constructor

```text
Shape::Scalar
=
Shape::Vector(N)
```

fails structurally.

### Different nominal domain

```text
Shape::Vector(3)
=
OtherShape::Vector(3)
```

fails even when selector/payload matches.

### No user equality

Never invoke:

```text
==
hash
runtime methods
runtime reflection
object identity
```

to solve static terms.

---

## 25. Substitution/materialization

Checked substitution must recursively handle:

- `StaticArgument`;
- static constant terms;
- type-data constructor terms;
- associated-domain specialization;
- heterogeneous `Applied`;
- existing pack terms;
- existing type-member projections;
- row arguments.

When substitution makes a term invalid, use the checked failure path.

Examples:

```text
wrong type-data constructor argument domain
invalid associated owner environment
row mismatch
pack duplicate exposed after substitution
```

must not leave malformed interned terms.

---

# Part VII — Associated domains and `::`

## 26. Current associated namespace

The live repository already has:

```text
AssociatedSurface
AssociatedFamilyInfo
AssociatedFamilyKind
AssociatedMemberId
AssociatedResolution
AssociatedOwnerResolution
```

and declaration-owned variant/data-constructor families.

This is the correct authority to extend.

Do not create:

```text
AssociatedTypeDataTable
with independent name lookup rules
```

that bypasses `AssociatedSurface`.

---

## 27. Target associated categories

Generalize associated metadata conceptually:

```rust
enum AssociatedFamilyKind {
    Case,
    DataConstructor,
    StaticDomain,
    StaticMember,
    // existing categories as needed
}

enum AssociatedMemberId {
    Case(AlgebraicCaseId),
    DataConstructor(DataConstructorId),
    DataComponent(DataComponentId),
    StaticDomain(AssociatedStaticDomainDeclarationId),
    StaticMember(TypeMemberDeclarationId),
}
```

Exact decomposition may use separate declaration/non-family entries.

The invariant is one associated base namespace authority.

---

## 28. Associated `type data`

Required P2 source:

```phalcom
class Parser {
    type data State {
        Start
        InString
        Done
    }

    type Initial: State
}
```

Required identities:

```text
Parser::State
Parser::State::Start
Parser::Initial
```

For a generic owner:

```phalcom
class Container<T: Type> {
    type data State {
        Empty
        Present(_ value: T)
    }
}
```

the declaration remains:

```text
Container::State
```

while concrete domain specializations carry the owner environment.

### 28.1 Required owners

C3 requires inherent associated type-data declarations at least on:

```text
class
runtime data root (product or sum)
```

C3 does not require:

```text
trait type-data requirement
exact-case-owned type-data domain
arbitrary local/block-owned type-data
```

unless separately amended.

---

## 29. Trait-associated type boundary

Existing trait-associated `type` requirements are evidence-bound conformance requirements.

Associated `type data` is declaration-owned.

They may share:

```text
::
source indexing
presentation helpers
associated name collision machinery
```

They must not share semantic ownership.

Do not make:

```text
class C { type data State { ... } }
```

produce a trait-associated-type requirement ID or conformance evidence slot.

---

## 30. Trait generic static-domain boundary

C3 does not require arbitrary user-defined static-domain generic parameters on traits.

Recommended bounded policy:

```text
classes/data/type aliases/type members
    migrate to general static domains

trait declarations / trait applications
    retain currently supported parameter-domain set
    and diagnose unsupported user `type data` domains
    until the conformance/coherence representation is explicitly migrated
```

If live implementation work proves trait arguments already support generalized `StaticArgument` without changing coherence/evidence identity, a later P2 plan may broaden this boundary.

That broadening must be explicit, tested, and must not be silently inferred from storage compatibility.

---

# Part VIII — Metadata, persistence, and incremental semantics

## 31. Stable metadata requirements

Stable metadata must represent at least:

```text
AlgebraicPhase
AlgebraicShape
algebraic declaration identity
case identity
product component identity/shape
type-data static-domain identity
type-data static constructor/case term
associated static-domain declaration
associated domain specialization
StaticArgument
generic parameter domain
heterogeneous Applied arguments
```

Stable identities use declaration/module/project references.

Never serialize:

```text
raw StaticDomainId
raw TypeId
raw InferVarId
source spelling as identity
```

---

## 32. Schema migration

C2 ended at metadata schema v4.

C3 introduces persistent semantic categories.

Therefore:

- the first P1 metadata landing requires an explicit monotonic schema transition;
- if P2 changes the already-published application argument encoding or adds associated-domain representation not reserved by P1, P2 must make another explicit schema transition;
- P3 must not silently mutate an earlier C3 schema version;
- compatibility decoding may map old enum-shaped metadata into runtime sum algebraic metadata during a bounded window;
- new metadata must not re-export `Enum` as an independent canonical declaration category.

Do not pre-select exact numeric versions beyond the rule that v4 is the baseline and every incompatible/additive public-format transition is explicit.

A P1 patch plan should lock the exact version after auditing live serialization compatibility policy.

---

## 33. Fingerprints

Interface/structural fingerprints must change for semantic changes to:

```text
Runtime <-> TypeOnly phase
Product <-> Sum
case set
case payload structure
singleton <-> explicit zero-arg constructor
GADT result specialization
type-data static domain identity
static constructor argument
associated domain ownership
owner specialization
generic parameter domain
StaticArgument value
intrinsic provenance when semantically relevant
```

They must not change solely because:

```text
legacy `enum` vs canonical `data` spelling
source range movement
private helper name
runtime physical representation choice
```

---

## 34. Incremental invalidation

Required edit matrix:

```text
data product -> data sum
data -> type data
type data -> data
case add/remove/rename
case payload edit
case generic edit
GADT result edit
static domain parameter edit
static argument edit
associated type-data owner/member edit
associated namespace collision introduced/removed
intrinsic declaration surface edit
legacy enum source migrated to equivalent data spelling
```

Cold and incremental final semantic state must agree.

The last migration case is especially important:

```text
enum X { A }
->
data X { A }
```

must not invent a new declaration/case semantic identity solely because source keyword changed.

---

# Part IX — Runtime/compiler boundary

## 35. Runtime `data`

Runtime products/sums continue using specialized lowering.

Allowed representations remain:

```text
DataObject
AdtCaseObject
singleton/immediate case
TupleObject
RecordObject
scalar replacement
other existing optimized form
```

Unified source semantics do not imply unified storage.

---

## 36. `type data`

Type-data inhabitants produce no runtime construction.

Forbidden:

```text
TypeDataObject
TypeDataCaseObject
VM opcode for static constructor formation
VM static-term comparison
runtime type-data case lookup
runtime static inference
```

Compiler/LSP/reflection may expose descriptors.

Descriptors are not inhabitants of the static domain.

---

## 37. Intrinsic structural declarations

P3 maps source-visible intrinsic declarations onto existing semantic/runtime authorities.

Conceptual:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

### 37.1 Mapping rules

```text
TypePack
    existing KindId::TYPE_PACK / TypeData::TypePack authority

LabeledTypePack
    existing KindId::LABELED_TYPE_PACK / LabeledTypePack authority

TupleEntries
    existing KindId::COMPLETE_TYPE_PACK / CompleteTypePack authority

RecordRow
    existing KindId::RECORD_ROW / row authority

Unit
    existing Unit/zero Tuple authority

Tuple<***Es>
    existing Tuple specialization/structural runtime authority

Record<R>
    existing Record/RecordRow structural authority
```

No new parallel canonical term is introduced merely because Universe now declares the domain.

---

# Part X — Tooling and specification migration

## 38. Source index

Required semantic targets:

```text
algebraic declaration
product component
case
type-data domain
type-data constructor/case
associated type-data declaration
generic static-domain reference
static constructor reference
associated lookup segment
```

Rename/navigation must operate on canonical semantic identity.

Legacy `enum` spelling may have a migration diagnostic but must still target the same declaration during the compatibility window.

---

## 39. LSP

P3 required baseline:

- hover: runtime `data` product/sum classification;
- hover: `type data` and static domain;
- definition: `Shape::Vector`;
- definition: `Parser::State`;
- definition: `Parser::State::Done`;
- completion: associated type-data domains/cases;
- rename: declaration/case semantic identity;
- diagnostics: obsolete `enum`;
- navigation: structural intrinsic member where already supported.

LSP consumes semantic/index products.

It does not reconstruct phase/shape/case identity from tokens.

---

## 40. Authoritative specification migration

C3 must update the current owners.

### `docs/specs/objects/data-objects.md`

Generalize/split into unified runtime algebraic data semantics.

Remove the statement that `data` is not a one-case enum.

Preserve product transparency, construction, immutability, exactness, representation freedom.

### `docs/specs/objects/enums.md`

Migrate sum/exact-case/GADT semantics to the unified data owner.

Retain as historical/redirect document only if repository documentation policy prefers that.

### `docs/specs/type-level-domains.md`

Generalize the static-domain model to nominal user-defined domains.

Document `TypeId` implementation details only in implementation records, not normative language semantics.

### `docs/specs/type-members.md`

Permit user-defined static result domains and describe interaction with associated-domain lookup.

### `docs/specs/associated-types.md`

State explicitly:

```text
trait associated type requirement
    !=
inherent associated type-data declaration
```

### `docs/specs/objects/impl.md`

Replace enum-root/exact-enum-case source terminology with runtime sum-data/exact-case semantics.

Define static type-member `impl` legality for type-data domains.

### Pattern/GADT owners

Trigger closed-sum logic from algebraic shape, not `Statement::Enum`.

### `docs/specs/universe.md`

Migrate Option/Result/Ordering examples from `enum` to `data` and expose intrinsic structural declarations.

---

# Part XI — Plan decomposition

## 41. Plan dependency graph

```text
C2 COMPLETE
    ↓
C3.P1
Unified algebraic declarations + top-level type-data foundation
    ↓
C3.P2
general static domains + heterogeneous application/inference
+ associated type data + persistence
    ↓
C3.P3
Universe/intrinsic surface + corpus/tooling migration
+ full certification + C3 closure
```

P1 must not attempt to finish P2's heterogeneous generic application.

P2 must not migrate Universe before its metadata/application semantics are stable.

P3 must not invent new static-domain representation.

---

# Part XII — LANG007.C3.P1 requirements

## 42. P1 title

**LANG007.C3.P1 — Unified Algebraic Data Declarations and Type-Data Foundation**

## 43. P1 objective

Establish one canonical algebraic source/semantic declaration model, move runtime sums from `enum` semantic ownership to sum-shaped `data`, and land top-level `type data` declarations with canonical static-domain/case terms while preserving runtime ADT/GADT behavior.

---

## 44. P1 required scope

### 44.1 Parser/AST

Implement:

- `data` tuple product;
- `data` record product;
- `data` singleton sum;
- singleton comma shorthand;
- tuple-payload cases;
- record-payload cases;
- explicit zero-arg constructor case;
- case-local generics;
- GADT result specialization;
- empty product vs empty sum;
- mixed-body rejection;
- `type data` versions of supported algebraic shapes;
- `intrinsic data`;
- `intrinsic type data`;
- legacy `enum` compatibility lowering to canonical algebraic AST.

### 44.2 Unified semantic product

Introduce authoritative:

```text
AlgebraicDeclarationInfo
AlgebraicPhase
AlgebraicRepresentation
case identity abstraction
```

or exact equivalents.

Current DataInfo/EnumInfo may remain adapters.

### 44.3 Runtime sum migration

Preserve:

- `VariantId` identity (or neutral alias);
- exact-case type identity;
- case family lookup;
- case-local generic environments;
- GADT result checks;
- exact-case impl targets;
- root requirements/default behavior;
- conformance on exact cases;
- pattern/exhaustiveness/redundancy;
- runtime lowering/reification;
- current Option/Result native associations.

### 44.4 Type-data foundation

Land:

- `StaticDomainId`;
- `KindData::StaticDomain` or exact equivalent;
- top-level type-data domain identity;
- canonical static-data term representation;
- initially required static literal term support for type-data payloads, especially Int;
- static payload validation;
- structural type-data term canonicalization;
- runtime-use rejection;
- no VM lowering for type-data inhabitants.

### 44.5 Intrinsic legality

Validate:

```text
intrinsic data
intrinsic type data
```

as implementation provenance + declaration phase, not new declaration categories.

### 44.6 Metadata foundation

Publish stable metadata for:

- algebraic phase;
- product/sum shape;
- top-level type-data declaration;
- cases/constructors;
- static-domain identity;
- type-data static term if exposed in public signatures.

P1 metadata is the first C3 schema transition.

### 44.7 Specifications

Update enough authoritative specs at P1 completion that newly enabled source syntax is not knowingly contradicted by current normative documentation.

At minimum:

```text
data-objects
enum/sum owner
type-level-domains foundation
```

---

## 45. P1 non-goals

P1 must not expand into:

- language-wide heterogeneous generic applications;
- associated `type data`;
- generic inference over type-data domains except minimal constructor-local correctness;
- full static aliases over arbitrary domains;
- Universe structural-domain exposure;
- full LSP completion/hover migration;
- corpus-wide enum removal;
- arbitrary compile-time functions;
- trait-owned type-data requirements;
- trait generic migration;
- runtime type-data objects.

---

## 46. P1 gate

P1 is accepted when:

```text
runtime data products work

runtime data sums work through the same canonical algebraic source/semantic declaration path

existing ADT/GADT exactness remains green

type data declarations have nominal domain/case identity

type-data terms canonicalize structurally

type-data terms cannot materialize as runtime values

legacy enum compatibility (if retained) creates exactly the same semantic identity as data

no second ADT/GADT implementation exists

no VM static-term execution path exists
```

---

## 47. P1 highest-risk areas

1. AST convergence accidentally erasing exact-case selector/shape data.
2. Product constructor visibility leaking into sum semantics.
3. singleton and explicit `()` case collapsing.
4. record product vs sum body ambiguity.
5. runtime compiler still branching exclusively on `Statement::Enum`.
6. duplicate semantic products from separate data/enum builders.
7. type-data terms accidentally entering TypeKnowledge as proper types.
8. static constructor syntax accidentally parsed as runtime call expression.
9. metadata exporting source spellings instead of stable identities.
10. compatibility `enum` producing a distinct declaration fingerprint.

---

# Part XIII — LANG007.C3.P2 requirements

## 48. P2 title

**LANG007.C3.P2 — Generalized Static Domains, Heterogeneous Application, and Associated Type Data**

## 49. P2 objective

Make user-defined static domains first-class throughout generic application, substitution, first-order inference, aliases/type members, imports/metadata, and associated `::` lookup.

---

## 50. P2 required scope

### 50.1 Heterogeneous generic application

Generalize:

```text
TypeData::Applied
TypeStore application APIs
GenericInstantiation
TypeSubstitution
TypeLambdaData/scoped forms
metadata TypeNode::Applied
formatting/presentation
```

to use the canonical `StaticArgument` model.

### 50.2 Static argument syntax

Support at least:

```phalcom
Tensor<Float, Shape::Vector(3)>
Integer<32, Endian::Big>
```

plus existing type and pack application.

### 50.3 Binder domains

Resolve:

```phalcom
T: Type
N: Int
S: Shape
P: TypePack
L: LabeledTypePack
E: TupleEntries
R: RecordRow
```

through one domain-classification system.

Mode remains orthogonal.

### 50.4 Type-data inference

Extend `InferenceTerm`/solver as required for:

- canonical const/static-data terms;
- recursive static-data constructor terms;
- `InferVarId` with nominal static-domain `KindId`;
- same-constructor first-order unification;
- different-constructor/domain refutation;
- materialization through canonical TypeStore constructors.

No search/backtracking evaluator.

### 50.5 Checked substitution

Re-canonicalize all heterogeneous arguments and static terms.

Preserve C2 checked pack behavior.

### 50.6 Type aliases/static definitions

Support accepted static-result syntax:

```phalcom
type DefaultShape: Shape = Shape::Scalar
type P: TypePack = T::Positionals
```

while retaining existing unannotated alias meaning as proper-type alias where specified.

### 50.7 Type members

Generalize type-member parameter/result domains to the same canonical static-domain model.

Do not maintain a separate ad hoc `ConstInt`/user-domain interpretation once the common static-term representation is established.

Compatibility wrappers may survive internally.

### 50.8 Associated `type data`

Implement in class/runtime-data owners.

Publish declaration-owned associated domain families/specializations.

### 50.9 Unified `::`

Extend `AssociatedSurface`/associated lookup to resolve:

```text
runtime case
data constructor
associated static domain
associated static term/type member
intrinsic type member
```

under one namespace collision authority.

### 50.10 Metadata/imports

Stable round trip for:

- nominal static domain;
- associated domain declaration/specialization;
- heterogeneous `Applied`;
- static arguments;
- static aliases;
- static generic signatures.

### 50.11 Incremental/source index

Cold/incremental parity for domain and term edits.

Index all static-domain references/associated paths.

### 50.12 Trait boundary

Reject unsupported user static-domain trait generic parameters unless a focused migration demonstrates conformance/coherence safety.

Do not fake support by storing terms in existing trait argument slots without auditing matching/evidence.

---

## 51. P2 non-goals

- general trait-associated type-data requirements;
- GAT expansion;
- theorem proving;
- arbitrary static functions;
- type-level runtime method execution;
- generalized static pattern matching;
- automatic runtime-value lifting into static terms;
- pack sequence decomposition;
- Universe/corpus final migration;
- VM static solver.

---

## 52. P2 gate

The following must have complete canonical semantics across source, checking, inference, metadata, imports, and incremental analysis:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

data Tensor<Element: Type, S: Shape>()

type V: Type = Tensor<Float, Shape::Vector(3)>

class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

Additionally:

```text
Parser::State
Parser::State::Done
specialized generic-owner associated domains
```

must use canonical declaration/environment identity.

---

## 53. P2 highest-risk areas

1. widening `TypeId` APIs without preserving `ProperTypeId`;
2. duplicating KindId with a second generic-domain registry;
3. forcing RecordRow into TypeId and breaking row invariants;
4. changing `Applied` without updating scoped aliases/substitution/fingerprints;
5. trait/conformance consumers assuming all arguments are proper types;
6. inference variable solving wrong nominal static domain;
7. associated domain lookup bypassing `AssociatedSurface`;
8. associated domain specialization encoded by rendered owner string;
9. type member subsystem retaining a contradictory second domain model;
10. metadata v4/vNext compatibility silently losing new argument categories;
11. runtime compiler seeing static-data terms as executable calls.

---

# Part XIV — LANG007.C3.P3 requirements

## 54. P3 title

**LANG007.C3.P3 — Intrinsic Structural Domain Surface, Universe Migration, Tooling, and Certification**

## 55. P3 objective

Expose the completed C3 model through canonical Universe declarations, migrate built-in/runtime algebraic source and public terminology, complete tooling/corpus coverage, and certify C3 end to end.

---

## 56. P3 required scope

### 56.1 Intrinsic static domains

Canonical Universe surface equivalent to:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow
```

### 56.2 Intrinsic structural runtime data

Equivalent to:

```phalcom
intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

### 56.3 Tuple static members

Equivalent to:

```phalcom
impl Tuple<***Es> {
    intrinsic type Entries: TupleEntries
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type At<I: Int>: Type
    intrinsic type Map<F: Type -> Type>: Type
}
```

### 56.4 TupleEntries bridge

Prove:

```text
TupleEntries public domain
    ==
C2 CompleteTypePack canonical semantic domain
```

without a second term representation.

### 56.5 Pack equivalence

Certify:

```text
*T   == T::Positionals
**T  == T::Labeled
***T == T::Entries
```

where valid.

### 56.6 Core sum migration

Migrate:

```text
Option
Result
Ordering
other canonical closed sums
```

from legacy `enum` syntax to `data`.

No identity change caused solely by source keyword migration.

### 56.7 Legacy enum removal

After source/corpus migration:

- reject or obsolete `enum` according to the final compatibility decision;
- remove canonical formatter/docs generation of `enum`;
- ensure new metadata/reflection says data/sum/case.

### 56.8 LSP/tooling

Complete hover/definition/completion/rename/diagnostics baseline.

### 56.9 Corpus migration

Migrate repository-owned source fixtures, Universe, examples, tests, and active docs.

Historical implementation records remain historical.

### 56.10 Certification

Prove:

- metadata stable round trip;
- cold/incremental parity;
- exact-case/GADT behavior;
- product behavior;
- generic static argument behavior;
- associated domain identity;
- packs/rows/TupleEntries;
- LSP/source index;
- runtime-negative type-data boundary;
- representation freedom;
- bounded normalization.

---

## 57. P3 non-goals

- release-wide refactors unrelated to C3;
- renaming every historical Rust enum-related private helper;
- type-data runtime reification as values;
- arbitrary static evaluation;
- trait-owned type-data requirements;
- new pack features;
- new Tuple/Record runtime representation.

---

## 58. P3 gate

C3 can close only when all checkpoint acceptance criteria in §1 pass.

Expected final lifecycle:

```yaml
LANG007.C3.P1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3.P2:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3.P3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

`RELEASE_COMPLETE` requires separate evidence and must not be inferred from checkpoint closure.

---

# Part XV — Cross-plan invariants

## 59. Global C3 invariants

```text
INV-C3-01 — `data` is the sole canonical runtime algebraic declaration keyword.
INV-C3-02 — `type data` is a semantic TypeOnly phase with no runtime inhabitants.
INV-C3-03 — phase, shape, provenance, and ownership are orthogonal.
INV-C3-04 — product versus sum is determined by algebraic shape, not a separate declaration category.
INV-C3-05 — runtime sum case sets remain closed and declaration-owned.
INV-C3-06 — runtime exact-case identity remains canonical and nominal.
INV-C3-07 — GADT result specialization/evidence survives the syntax migration.
INV-C3-08 — representation-bearing algebra lives in declarations; behavior lives in impls.
INV-C3-09 — source unification does not require runtime storage unification.
INV-C3-10 — TypeId remains a canonical handle for kinded non-row static terms.
INV-C3-11 — ProperTypeId remains the proof of the Type domain.
INV-C3-12 — user static domains are open-ended nominal identities, not per-domain compiler enum variants.
INV-C3-13 — GenericParameterMode remains orthogonal to static domain.
INV-C3-14 — RecordRow remains a distinct static domain with its existing dedicated canonical/solver machinery.
INV-C3-15 — LabeledTypePack remains distinct from RecordRow.
INV-C3-16 — TupleEntries maps to C2 CompleteTypePack and is not a proper Tuple type.
INV-C3-17 — StaticArgument is the canonical heterogeneous application envelope.
INV-C3-18 — no static term is encoded as a fake ProperTypeId.
INV-C3-19 — non-row static inference reuses InferVarId classified by canonical domain KindId.
INV-C3-20 — type-data unification is structural/first-order and invokes no user/runtime equality.
INV-C3-21 — unresolved valid static terms remain symbolic.
INV-C3-22 — checked substitution re-canonicalizes static terms and fails closed on invalidity.
INV-C3-23 — associated type-data declarations are inherent declaration-owned entities, not trait evidence requirements.
INV-C3-24 — one associated namespace authority owns case/domain/static-member collision semantics.
INV-C3-25 — associated domain specialization contains canonical owner environment identity.
INV-C3-26 — legacy enum compatibility cannot create a second declaration/case identity.
INV-C3-27 — metadata uses stable declaration/domain/argument structure, never source strings as identity.
INV-C3-28 — query-local inference IDs never enter durable metadata.
INV-C3-29 — incremental and cold final semantic products converge.
INV-C3-30 — TypePack/LabeledTypePack/CompleteTypePack/RecordRow C1/C2 authorities are reused.
INV-C3-31 — *, **, *** retain C2 meanings; ... is not pack syntax.
INV-C3-32 — intrinsic source declarations are facades over canonical compiler semantics, not duplicate semantic owners.
INV-C3-33 — type-data term formation requires no runtime object/opcode/solver.
INV-C3-34 — runtime products/sums preserve representation freedom and exact runtime identity rules.
INV-C3-35 — authoritative specs must be migrated before C3 closes.
INV-C3-36 — tooling consumes semantic identity rather than token/string reconstruction.
INV-C3-37 — trait-associated type requirements remain semantically distinct.
INV-C3-38 — unsupported trait static-domain generics fail explicitly rather than being partially encoded.
INV-C3-39 — normal declaration/static-term operations remain approximately linear in reachable structure.
INV-C3-40 — C3 must not expand into unrestricted compile-time execution.
```

---

# Part XVI — Testing requirements

## 60. Coverage matrix

### Syntax / shape

- tuple product;
- record product;
- singleton sum;
- comma singleton shorthand;
- tuple-payload sum;
- record-payload sum;
- zero-arg constructor distinct from singleton;
- GADT sum;
- empty product;
- empty sum;
- mixed-body rejection;
- type-data equivalents;
- intrinsic data/type-data;
- legacy enum migration.

### Identity

- structurally equal runtime declarations remain nominally distinct;
- structurally equal type-data declarations remain distinct domains;
- equal static terms canonicalize;
- unequal cases/constructors do not;
- legacy enum/data spellings during compatibility share identity;
- associated domain specialization includes owner environment.

### Generic/static application

- `Tensor<Float, Shape::Vector(3)>`;
- `Integer<32, Endian::Big>`;
- wrong static domain;
- fixed TypePack argument still distinct from `*Ts`;
- RecordRow argument via `StaticArgument`;
- partial generic application;
- alias through heterogeneous arguments;
- cross-module static argument.

### Inference

- same type-data constructor selects payload inference variable;
- different constructor refutes;
- different domain refutes;
- nominal static domain `InferVarId` kind mismatch;
- symbolic static term stays symbolic;
- no runtime equality hook;
- C2 pack inference remains unchanged.

### Associated

- `Parser::State`;
- `Parser::State::Done`;
- associated static term;
- generic-owner specialization;
- unspecialized family where concrete domain required;
- collision with type member/case/callable family;
- trait associated type remains separate.

### ADT/GADT preservation

- exact-case construction;
- exact-case impl;
- case conformance;
- GADT branch refinement;
- exhaustive match;
- non-exhaustive match;
- redundant branch;
- root default/requirement completeness;
- singleton/private visibility.

### Persistence/incremental

- phase edit;
- shape edit;
- case add/remove;
- payload edit;
- GADT result edit;
- static-domain binder edit;
- static-argument edit;
- associated domain edit;
- metadata round trip;
- imported/cold equality;
- no raw IDs;
- equivalent enum->data migration avoids semantic identity churn.

### Runtime boundary

- type-data term emits no construction bytecode;
- no type-data VM object;
- no static solver opcode;
- runtime data product/sum behavior unchanged;
- Tuple/Record representation stays specialized.

### Tooling

- definition;
- hover;
- completion;
- rename;
- obsolete enum diagnostic;
- associated path segment indexing.

---

## 61. Verification strategy by plan

### P1

Primary suites:

```text
phalcom-ast parser/algebraic syntax
phalcom-semantic ADT/data declaration suites
exact-case/GADT focused suites
phalcom-core algebraic-data focused suites
metadata focused tests
```

Do not run workspace-wide suites after each AST migration step.

### P2

Primary suites:

```text
type annotations/generic application
type model/substitution
inference
type members
associated resolution
metadata
imported resolution
incremental
source index
```

Keep trait/conformance suites focused on proving the explicit unsupported/supported boundary.

### P3

Primary suites:

```text
Universe
core algebraic data
pack/type-member closure
source/LSP integration
corpus migration
metadata/incremental parity
runtime-negative audit
```

P3 is the checkpoint certification plan and may deliberately execute broader C3-specific corpus gates.

---

# Part XVII — Performance and resource requirements

## 62. Complexity targets

Ordinary operations should be approximately linear in reachable input size.

Required:

```text
case duplicate detection
    expected O(n)

static constructor interning
    hash-consed

StaticArgument application
    O(argument count + child canonicalization)

substitution
    proportional to reachable static-term graph

associated lookup
    indexed by owner/base, not whole-program scan

metadata
    graph encoding, not stringify/reparse

pack structures
    retain C2 flatness guarantees
```

C3 does not introduce a general evaluator and therefore must not create a new unbounded compile-time execution budget.

---

# Part XVIII — Failure modes

## 63. Plausible but architecturally wrong implementations

### FM-01 — every type-data domain becomes a new `KindData` Rust variant

Why wrong:

```text
closed compiler enum
    would become source declaration registry
```

Correct:

```text
one dynamic nominal StaticDomain kind form
```

### FM-02 — add a second all-purpose `StaticTermId` arena

Why wrong:

duplicates TypeStore/C2 canonicalization.

Correct:

extend the TypeId kinded term universe; preserve row specialization.

### FM-03 — store `Shape::Vector(3)` with `KindId::TYPE`

Why wrong:

it becomes a fake proper type.

Correct:

nominal static-domain kind; `ProperTypeId` rejects it.

### FM-04 — put RecordRow into TypeId just to simplify `Applied`

Why wrong:

undoes accepted row ownership.

Correct:

`StaticArgument` row variant.

### FM-05 — implement sums again under DataInfo

Why wrong:

duplicates mature ADT/GADT authority.

Correct:

one algebraic authority with existing runtime-sum products as derived adapters.

### FM-06 — keep `enum` and `data` as separate semantic categories sharing syntax

Why wrong:

violates C3's central model.

### FM-07 — parse static generic arguments as arbitrary runtime expressions

Why wrong:

implicitly creates compile-time execution.

### FM-08 — associated type data uses trait associated-type evidence IDs

Why wrong:

different ownership semantics.

### FM-09 — create a second associated-domain name lookup table

Why wrong:

duplicates `AssociatedSurface`.

### FM-10 — represent associated specialization by `"Container<Int>::State"` string

Why wrong:

unstable spelling-based identity.

### FM-11 — expose TupleEntries with a new term type

Why wrong:

duplicates CompleteTypePack.

### FM-12 — make intrinsic Tuple use general data-object storage

Why wrong:

semantic declaration surface does not constrain representation.

### FM-13 — migrate enum keyword and accidentally churn VariantId/stable identity

Why wrong:

source keyword is not semantic identity.

### FM-14 — silently accept trait static-domain generics because storage happens to fit

Why wrong:

conformance/coherence semantics may not.

---

# Part XIX — Escalation conditions

## 64. Mandatory STOP / CONSULT triggers

Stop architecture work and consult if any of the following becomes true:

1. `TypeId` + dynamic nominal `KindData` cannot represent type-data terms without making them proper types.
2. Heterogeneous `Applied` cannot preserve row arguments without replacing the row solver/identity architecture.
3. Runtime Data/Enum semantic convergence would erase exact-case or GADT evidence.
4. Legacy enum/data compatibility cannot share declaration/case identity.
5. Generalized generic application requires a mandatory trait/conformance representation redesign larger than this checkpoint boundary.
6. Associated `type data` cannot use `AssociatedSurface` without redefining trait-associated evidence.
7. Associated domain specialization cannot be represented structurally without source strings.
8. CompleteTypePack observably differs from required TupleEntries semantics.
9. Static metadata cannot be versioned without leaking raw session/store IDs.
10. A type-data implementation requires VM execution to establish semantic identity.
11. Parser/AST phase is the only place distinguishing runtime data from type data.
12. ProperTypeId enforcement would need weakening.
13. RecordRow would need to become LabeledTypePack or vice versa.
14. C2 pack inference/decomposition invariants would need weakening.
15. `enum` compatibility produces a second semantic product or fingerprint.
16. P2 requires arbitrary static-function evaluation to make the specified examples work.
17. Two plausible static-domain inference designs have materially different identity/soundness consequences.
18. A focused semantic failure survives one serious correction.
19. Passing tests requires weakening any `INV-C3-*`.
20. Implementation scope enters an unplanned major subsystem with new semantic authority.

---

# Part XX — Repository impact map

## 65. P1 primary areas

Expected:

```text
phalcom-ast/src/{ast,parser,lexer,error}.rs
phalcom-modules interface/declaration extraction
phalcom-semantic/src/{data_semantics,enum_semantics,identity}.rs
phalcom-semantic/src/checker/{data_declaration,enum_declaration,...}.rs
phalcom-semantic/src/{associated,impls,match_semantics,...}.rs
phalcom-semantic/src/types/{id,kind,store}.rs
phalcom-semantic/src/db/{product,query,fingerprint}.rs
phalcom-semantic/src/metadata/export.rs
phalcom-type-meta
phalcom-core/src/compiler/lib/{data_decl,enum_decl,...}.rs
authoritative specs
```

## 66. P2 primary areas

Expected:

```text
phalcom-semantic/src/types/{store,application,annotation,parameter,instantiation,substitution,type_lambda,type_member}.rs
phalcom-semantic/src/checker/{inference,associated,...}.rs
phalcom-semantic/src/{associated,type_alias,session,source_index}.rs
phalcom-semantic/src/metadata/export.rs
phalcom-type-meta
phalcom-modules interface metadata as needed
phalcom-lsp source/navigation paths
```

## 67. P3 primary areas

Expected:

```text
Universe sources/catalog
core_surface/native/source associations
compiler semantic lowering
phalcom-core structural runtime mappings
phalcom-lsp
repository source corpus/examples/tests
docs/specs
LANG007 lifecycle docs
```

This is a requirements impact map, not authorization to edit every listed file in every plan.

Each P1/P2/P3 patch-grade plan must re-audit exact live ownership and narrow the edit set.

---

# Part XXI — Baseline and compatibility

## 68. Inherited baselines

C2 handed off two unrelated trait/default-body failures:

```text
BASELINE-001
semantic::capabilities::traits::
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

C3 plans must not opportunistically repair them unless a C3 change causally interacts with them and blocks acceptance.

Use repository failure classification:

```text
A — definitely caused by active plan
B — probably caused
C — unclear; one bounded classification pass
D — unrelated/baseline
```

---

# Part XXII — Plan-specific prerequisites

## 69. P1 prerequisites

- C2 complete.
- C2 pack/static type-member invariants frozen.
- C3 source semantics accepted for planning.
- current data/enum exact-case implementation available.

## 70. P2 prerequisites

P1 must have established:

- one algebraic declaration identity model;
- top-level type-data static-domain identity;
- canonical static-data terms;
- phase/shape metadata;
- no runtime type-data inhabitants.

P2 must not begin by designing these again.

## 71. P3 prerequisites

P2 must have established:

- heterogeneous generic application;
- user static-domain binder classification;
- static inference/substitution;
- associated type-data;
- stable metadata/import/incremental identity.

P3 must not use Universe migration as a laboratory for unresolved P2 architecture.

---

# Part XXIII — Checkpoint-level acceptance matrix

## 72. P1 evidence required before P2

```text
[ ] one canonical algebraic AST/semantic input path
[ ] data product
[ ] data sum
[ ] GADT sum
[ ] type data top-level domain
[ ] canonical type-data terms
[ ] proper-type boundary
[ ] enum compatibility shares identity
[ ] metadata stable identity
[ ] runtime ADT behavior preserved
[ ] no VM type-data execution
```

## 73. P2 evidence required before P3

```text
[ ] StaticArgument canonical envelope
[ ] heterogeneous Applied identity
[ ] Int static terms in generic application
[ ] user type-data terms in generic application
[ ] row application uses row authority
[ ] domain-directed InferVarId
[ ] first-order static constructor unification
[ ] checked substitution
[ ] static aliases/result domains
[ ] associated type data
[ ] one :: namespace authority
[ ] metadata round trip
[ ] imported/cold parity
[ ] incremental parity
[ ] source index identity
[ ] trait boundary explicit
```

## 74. P3 evidence required for C3 closure

```text
[ ] intrinsic structural domain declarations
[ ] TupleEntries == CompleteTypePack
[ ] pack/member equivalences
[ ] Unit/Tuple/Record intrinsic data surface
[ ] Option/Result/Ordering migrated
[ ] enum compatibility removed/finalized
[ ] LSP/source tooling complete
[ ] corpus migrated
[ ] authoritative specs coherent
[ ] runtime representation unchanged where required
[ ] metadata/interface fingerprints stable
[ ] cold/incremental parity
[ ] performance/resource checks
[ ] no VM static-domain solver
[ ] C3 checkpoint/walkthrough/handoff truthful
```

---

# Part XXIV — Final recommendation

## 75. Recommended C3 architecture in one diagram

```text
SOURCE
│
├─ data
│   └─ AlgebraicDef(phase = Runtime)
│       ├─ Product
│       └─ Sum
│
└─ type data
    └─ AlgebraicDef(phase = TypeOnly)
        ├─ Product
        └─ Sum
            │
            ▼
SEMANTIC ALGEBRAIC AUTHORITY
AlgebraicDeclarationInfo
├─ declaration identity
├─ phase
├─ generic contract
├─ ProductInfo
└─ SumInfo / CaseInfo
    │
    ├─ Runtime branch
    │   ├─ existing DataInfo adapter
    │   ├─ existing EnumInfo/VariantInfo adapter
    │   ├─ ExactCase / GADT / match
    │   └─ existing specialized lowering
    │
    └─ TypeOnly branch
        ├─ StaticDomainId
        ├─ KindData::StaticDomain(domain)
        └─ TypeData::StaticData(...)
            │
            ▼
CANONICAL STATIC TERM SYSTEM
TypeId -------------------- ProperTypeId
│                            only when kind == Type
├─ proper types
├─ packs
├─ type lambdas
├─ projections
├─ const static terms
└─ type-data terms

RecordRowId
└─ remains separate

GENERIC APPLICATION
StaticArgument
├─ Term(TypeId)
└─ RecordRow(RecordRowId)

INFERENCE
InferVarId(KindId)
├─ Type
├─ packs
├─ const domain
└─ nominal StaticDomain
+
existing row solver for RecordRow

ASSOCIATED LOOKUP
AssociatedSurface
├─ cases
├─ constructors
├─ static domains
└─ static/type members

PERSISTENCE
stable declaration/domain/case/static-argument graph
(no raw store/query IDs)

RUNTIME
only Runtime data lowers to values
TypeOnly terms never become VM inhabitants
```

This architecture extends the already-proven C1/C2 semantic tower rather than starting a parallel compile-time language.

That is the central requirement for all three C3 plans.
''').lstrip()

checkpoint = dedent(r'''
---
id: LANG007.C3
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: checkpoint-record
title: "Type Data and Unified Algebraic Static Domains"
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
requires:
  - LANG007.C1
  - LANG007.C2
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
active_plan: null
latest_completed_plan: null
next_plan: LANG007.C3.P1
---

# LANG007.C3 — Checkpoint Record

## Type Data and Unified Algebraic Static Domains

## 0. Checkpoint state

```yaml
program: LANG007
checkpoint: LANG007.C3
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
active_plan: null
latest_completed_plan: null
next_plan: LANG007.C3.P1
blockers: none known at planning time
```

This checkpoint record is the single durable state authority for C3 once implementation begins.

The checkpoint must be changed to `IN_PROGRESS` when P1 starts, not merely because planning documents exist.

---

## 1. Objective

Establish one coherent algebraic declaration and static-domain system in which:

```text
data
    defines runtime closed algebraic domains
    including products, sums, ADTs, and GADTs

type data
    defines type-system-only closed algebraic/static domains

associated type data
    participates in the declaration-owned :: namespace

intrinsic data / intrinsic type data
    expose compiler-owned structural domains through ordinary language semantics
```

while preserving:

- exact-case/GADT behavior;
- C1/C2 pack and type-member identity;
- generic binder-mode semantics;
- row separation;
- metadata stability;
- incremental correctness;
- runtime representation freedom;
- the no-runtime-static-solver boundary.

---

## 2. Bounded acceptance statement

C3 is complete only when this statement is true:

> Runtime products and closed sums use one canonical `data` algebraic declaration model; `type data` provides nominal canonical static domains and terms; generic/static application, inference, substitution, aliases, associated lookup, metadata, imports, and incremental analysis preserve user static-domain identity; compiler intrinsic structural domains are exposed through that same model without duplicating C1/C2 terms; tooling and canonical source have migrated from `enum`; and no VM static-term execution authority has been introduced.

---

## 3. Ownership boundary

### 3.1 C3 owns

- unified runtime algebraic declaration syntax/semantic identity;
- `enum` source migration/removal;
- product/sum shape classification;
- runtime case syntax under `data`;
- `type data`;
- nominal static-domain identity;
- canonical type-data static terms;
- compile-time static Int term integration needed by C3;
- heterogeneous generic/static argument application;
- domain-directed non-row inference over user static domains;
- associated `type data`;
- associated static-domain lookup through `::`;
- stable metadata for C3 domains/terms/applications;
- cross-module/incremental identity;
- intrinsic structural domain source surface;
- Universe/corpus migration;
- C3 LSP/source-index closure;
- specification migration;
- checkpoint certification.

### 3.2 C3 does not own

- arbitrary compile-time execution;
- a macro system;
- a theorem prover;
- user-defined primitive kinds;
- user-defined static equality hooks;
- arbitrary type-level pattern functions;
- trait requirements that bind whole `type data` domains;
- GAT expansion beyond existing accepted semantics;
- a general trait static-domain generic redesign unless separately amended;
- automatic lifting of runtime values into static terms;
- new pack decomposition/backtracking;
- runtime ArgumentPack ABI redesign;
- runtime type-data values;
- VM static-domain solving;
- forced unification of Tuple/Record/data runtime storage;
- release-wide unrelated cleanup.

---

## 4. Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG007.C3.P1` | Unified algebraic `data`; enum semantic migration; top-level `type data` foundation | PROPOSED | NOT_STARTED | UNVERIFIED | next |
| `LANG007.C3.P2` | General static domains; heterogeneous application/inference; associated `type data`; metadata/import/incremental | PROPOSED | NOT_STARTED | UNVERIFIED | waits on P1 |
| `LANG007.C3.P3` | Intrinsic structural surface; Universe/corpus/tooling migration; certification and closure | PROPOSED | NOT_STARTED | UNVERIFIED | waits on P2 |

No additional P4 is planned.

If implementation reveals a bounded corrective closure needed for C3 acceptance, add a new plan in this checkpoint rather than falsely closing P3.

---

## 5. Predecessor state

### 5.1 LANG007.C1

C1 established the type-level domain/member substrate C3 relies on.

C3 must preserve:

- canonical `TypeId`/`KindId`;
- `ProperTypeId`;
- parameterized type-member identity;
- structural Tuple/Record operations;
- RecordRow separation;
- compiler-owned closed normalization.

### 5.2 LANG007.C2

C2 is:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

Inherited C2 facts:

```text
*   positional
**  labeled
*** complete

TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks

GenericParameterMode
GenericBinderHeader
checked substitution/materialization
whole-pack InferVarId(KindId) inference
metadata schema v4 scoped packs
cold/incremental parity
no runtime pack solver
```

Do not redesign these.

---

## 6. Repository baseline

Planning baseline:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   62a6082ab463387c812d558e5561508267ecc63c
```

Relevant post-C2 observation:

- C2 checkpoint records are complete.
- Future-facing `PROGRAM.md` / `STATUS.md` contain some stale C2 next-action wording.
- P1 start must rebaseline active program state to C3.

This planning document does not rewrite historical C2 records.

---

## 7. Inherited established invariants

These are already established before C3 begins.

### 7.1 Canonical type/static substrate

```text
C2-I01 — TypeId is a store-relative canonical type-level term handle.
C2-I02 — ProperTypeId proves the Type domain.
C2-I03 — KindId classifies canonical TypeId terms.
C2-I04 — RecordRow remains a separate row-domain representation/solver.
C2-I05 — LabeledTypePack is not RecordRow.
C2-I06 — CompleteTypePack is not Tuple.
```

### 7.2 Packs

```text
C2-I07 — * means positional lane.
C2-I08 — ** means labeled lane.
C2-I09 — *** means complete lane.
C2-I10 — ... is not pack syntax.
C2-I11 — GenericParameterMode is independent from KindId.
C2-I12 — Single(TypePack) != PositionalPack(TypePack).
C2-I13 — whole-pack inference uses ordinary InferVarId(KindId).
C2-I14 — no pack partition/decomposition solver exists.
C2-I15 — C2 materialization re-enters canonical TypeStore constructors.
```

### 7.3 Persistence/runtime

```text
C2-I16 — stable metadata is structural, not source-string identity.
C2-I17 — query-local inference IDs do not enter durable products.
C2-I18 — cold and incremental pack semantics converge.
C2-I19 — no runtime pack object/opcode/solver exists.
```

---

## 8. Required C3 acceptance invariants

These are requirements, not yet implementation facts.

They become "established" in this checkpoint only after the owning plan implements and verifies them.

```text
C3-I01 — `data` is the sole canonical runtime algebraic declaration category.
C3-I02 — `type data` is semantic phase TypeOnly with no runtime inhabitants.
C3-I03 — phase, shape, implementation provenance, and ownership are orthogonal.
C3-I04 — Product/Sum shape, not keyword category, triggers algebraic semantics.
C3-I05 — runtime sum exact-case and GADT semantics are preserved.
C3-I06 — one algebraic semantic authority owns product/case structure.
C3-I07 — TypeId remains canonical carrier for non-row kinded static terms.
C3-I08 — ProperTypeId remains the proper-Type proof boundary.
C3-I09 — user static domains use nominal StaticDomain identity, not per-domain Rust enum cases.
C3-I10 — StaticArgument is the heterogeneous application envelope.
C3-I11 — RecordRow remains row-owned under heterogeneous applications.
C3-I12 — GenericParameterMode stays orthogonal to static domain.
C3-I13 — non-row static inference reuses InferVarId classified by canonical domain.
C3-I14 — type-data unification is closed structural first-order unification.
C3-I15 — runtime equality/dispatch never defines static identity.
C3-I16 — valid unresolved static terms remain symbolic.
C3-I17 — associated type-data is declaration-owned, not trait-evidence-owned.
C3-I18 — one associated namespace authority owns collision/lookup semantics.
C3-I19 — associated domain specialization preserves owner environment.
C3-I20 — legacy enum compatibility never creates distinct semantic identity.
C3-I21 — TupleEntries is the public surface over C2 CompleteTypePack.
C3-I22 — intrinsic declarations do not replace existing C1/C2 canonical term owners.
C3-I23 — metadata stores stable declaration/domain/static-argument structure.
C3-I24 — incremental and cold results converge for C3 edits.
C3-I25 — type-data inhabitants require no VM value/object/opcode/solver.
C3-I26 — runtime algebraic storage remains representation-independent.
C3-I27 — LSP/source tools consume semantic products, not rendered-name reconstruction.
C3-I28 — current authoritative specs are coherent before checkpoint closure.
```

---

## 9. Stable takeover map

| Concept | Live owner | C3 rule |
|---|---|---|
| canonical type-level term | `TypeStore` / `TypeId` | extend; do not duplicate |
| proper type proof | `ProperTypeId` | preserve strictly |
| kind/domain classifier | `KindId` / `KindData` | extend with nominal static-domain form |
| row term | `RecordRowId` / row subsystem | preserve separate |
| generic binder mode | `GenericParameterMode` | preserve orthogonal |
| generic early contract | `GenericBinderHeader` | extend domain support |
| generic signature | `GenericSignature` | extend domain support |
| non-row inference var | `InferVarId(KindId)` | reuse |
| runtime product | `DataInfo` / `DataSemanticTable` | derive from unified algebraic authority |
| runtime sum | `EnumInfo` / `VariantInfo` | derive/reuse under unified algebraic authority |
| exact case | `VariantId` + `TypeData::ExactCase` | preserve identity; neutral terminology may wrap |
| GADT environment | `CaseTypeEnvironment` | preserve |
| associated lookup | `AssociatedSurface` / checker associated resolver | extend |
| type-member projection | type-member subsystem | integrate with common static domains |
| packs | C1/C2 TypeStore constructors | preserve |
| complete Tuple entries | `CompleteTypePack` | expose as `TupleEntries` |
| metadata | `phalcom-type-meta`, schema v4 baseline | version explicitly for C3 |
| Universe identity | canonical Universe modules | migrate source surface; keep identity |
| runtime data/sum lowering | `phalcom-core` specialized lowering | preserve representation freedom |

---

## 10. Architecture decisions adopted for planning

### A. Canonical term carrier

Use:

```text
TypeId + KindId
```

for all non-row canonical static terms, including new const/type-data terms.

Do not introduce a second general static-term arena.

### B. User static domain classifier

Use one dynamic nominal classifier equivalent to:

```rust
KindData::StaticDomain(StaticDomainId)
```

Do not add one compiler enum variant per source declaration.

### C. Heterogeneous application

Use a canonical envelope equivalent to:

```rust
StaticArgument::Term(TypeId)
StaticArgument::RecordRow(RecordRowId)
```

and migrate `Applied`, scoped application, substitution, metadata, and formatting.

### D. Inference

Reuse:

```text
InferVarId(kind)
```

for non-row user static domains.

Keep row inference separate.

### E. Algebraic declarations

Use one semantic declaration product/facade with:

```text
phase
shape
generic contract
product/case structure
```

DataInfo/EnumInfo may remain compatibility projections.

### F. Associated domains

Extend `AssociatedSurface`.

Do not create an independent static-domain namespace.

### G. Trait boundary

Whole associated `type data` requirements are out of scope.

User static-domain trait generics are explicitly unsupported until a plan proves safe conformance/coherence migration.

### H. Runtime boundary

Only Runtime algebraic data lower to runtime inhabitants.

TypeOnly terms never do.

---

## 11. P1 acceptance contract

`LANG007.C3.P1` must establish:

```text
source:
    canonical data Product/Sum
    type data Product/Sum
    legacy enum -> same algebraic AST during compatibility

semantics:
    one algebraic declaration authority
    preserved DataInfo/EnumInfo derived behavior
    top-level nominal StaticDomainId
    canonical type-data terms

soundness:
    ProperTypeId boundary
    static payload validity
    no runtime type-data inhabitants

runtime:
    exact-case/GADT/match/impl behavior preserved

persistence:
    first stable C3 algebraic/static-domain metadata
```

P1 must not complete heterogeneous generic application or associated type data.

---

## 12. P2 acceptance contract

`LANG007.C3.P2` must establish:

```text
StaticArgument
heterogeneous Applied
generic/static binder domains
type-data/const static arguments
domain-directed inference
checked substitution/materialization
static aliases/result domains
associated type data
unified :: lookup
metadata/import round trip
cold/incremental parity
source-index identity
explicit trait boundary
```

Representative source must work:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

data Tensor<Element: Type, S: Shape>()

type V: Type = Tensor<Float, Shape::Vector(3)>

class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

---

## 13. P3 acceptance contract

`LANG007.C3.P3` must establish and certify:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

and:

```text
TupleEntries == CompleteTypePack semantic domain

*T   == T::Positionals
**T  == T::Labeled
***T == T::Entries
```

where valid.

P3 also owns:

- Option/Result/Ordering migration to `data`;
- final enum compatibility removal/finalization;
- LSP tooling;
- corpus/examples/active docs migration;
- authoritative spec closure;
- metadata/interface fingerprint certification;
- cold/incremental parity;
- performance/resource evidence;
- runtime-negative audit;
- C3 checkpoint closure.

---

## 14. Verification ledger

No C3 implementation verification has run yet.

| Gate | Scope | Result | Meaning |
|---|---|---|---|
| predecessor C2.P3 G1–G5 | C2 substrate | PASS | C3 starts from verified packs/inference/metadata/runtime boundary |
| C3 gates | none | NOT RUN | checkpoint remains UNVERIFIED |

Do not copy predecessor PASS status into C3 verification.

---

## 15. Deferred / baseline issue ledger

### BASELINE-001

```text
Observed:
    semantic::capabilities::traits::
    trait_default_replays_external_nominal_callable_dependencies

Classification:
    inherited baseline / unrelated unless C3 proves causal interaction

Why nonblocking at C3 start:
    C2 explicitly deferred it; C3 architecture does not require trait-default repair.

When to revisit:
    only if a C3 focused trait-boundary test is blocked by the same cause.
```

### BASELINE-002

```text
Observed:
    semantic::capabilities::traits::
    trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities

Classification:
    inherited baseline / unrelated unless C3 proves causal interaction

Why nonblocking at C3 start:
    C2 explicitly deferred it.

When to revisit:
    only if a required C3 trait/conformance compatibility gate depends on it.
```

---

## 16. Known planning risks

| Risk | Severity | Owner plan |
|---|---|---|
| AST data/enum convergence loses GADT/case precision | critical | P1 |
| duplicate DataInfo/EnumInfo authority survives | high | P1 |
| type-data term accidentally considered ProperType | critical | P1/P2 |
| static-domain classifier becomes per-declaration compiler enum | high | P1 |
| heterogeneous `Applied` breaks row architecture | critical | P2 |
| trait/conformance assumes TypeId args are proper types | high | P2 |
| associated domain becomes second namespace | critical | P2 |
| associated specialization uses source strings | critical | P2 |
| metadata silently changes schema semantics | high | P1/P2 |
| TupleEntries duplicates CompleteTypePack | critical | P3 |
| intrinsic Tuple forces general data storage | high | P3 |
| enum keyword migration churns identity | high | P1/P3 |
| static syntax accidentally authorizes runtime expression execution | critical | P1/P2 |
| C3 grows into general compile-time evaluation | critical | all |

---

## 17. Consultation / amendment ledger

No C3 consultation incidents yet.

Use entries:

```text
INC-C3-<n>
Plan/task:
Trigger:
Observed:
Decision:
Architecture impact:
Plan amendment:
Verification required:
Status:
```

Material changes to:

- canonical term carrier;
- ProperTypeId boundary;
- row ownership;
- generic application representation;
- exact-case identity;
- associated namespace ownership;
- runtime/static execution boundary;

require a principal-architect amendment, not local implementer improvisation.

---

## 18. Mandatory checkpoint STOP conditions

Implementation must stop and consult if:

1. TypeId cannot represent new non-row static terms without weakening ProperTypeId.
2. RecordRow must be collapsed into TypeId to implement heterogeneous applications.
3. Data/Enum convergence breaks exact-case/GADT identity.
4. associated type-data requires trait evidence ownership.
5. one associated namespace cannot cover the required categories without duplicate lookup authority.
6. metadata requires source strings for identity.
7. CompleteTypePack cannot faithfully implement TupleEntries.
8. the VM must execute type-data constructors for semantic correctness.
9. enum/data compatibility produces different declaration or case identity.
10. C3 requires arbitrary static functions/evaluation to satisfy its accepted examples.
11. trait static-domain support becomes mandatory and requires coherence architecture changes.
12. any C2 pack/row invariant must be weakened.

---

## 19. Checkpoint verification strategy

### P1 evidence

Focus:

```text
parser/AST
algebraic semantic declaration
runtime ADT/GADT preservation
type-data canonical identity
metadata foundation
runtime-negative type-data boundary
```

### P2 evidence

Focus:

```text
generic/static application
inference
substitution
associated lookup
metadata/imports
incremental
source index
```

### P3 evidence

Focus:

```text
Universe/intrinsic structural domains
core algebraic data
TupleEntries/pack equivalence
LSP
corpus migration
spec consistency
cold/incremental certification
runtime-negative audit
```

Workspace/release-wide certification is not implied by `FOCUSED_TESTED`.

---

## 20. Active plan and next action

Current:

```text
active plan:
    null

next plan:
    LANG007.C3.P1

next action:
    create the patch-grade P1 implementation plan from:
      - LANG007.C3-requirements-analysis.md
      - LANG007.C3-CHECKPOINT.md
      - LANG007.C3-GUIDANCE.md
      - C2.P3 handoff/walkthrough
      - live repository
```

At P1 start:

1. verify local revision/worktree;
2. re-read live C3 target paths;
3. update `PROGRAM.md` / `STATUS.md` active lifecycle state;
4. mark checkpoint `IN_PROGRESS / PARTIAL / UNVERIFIED`;
5. mark P1 active;
6. do not begin P2 representation work opportunistically.

---

## 21. C3 completion truth table

C3 may be marked `COMPLETE` only when:

```text
[ ] P1 complete
[ ] P2 complete
[ ] P3 complete
[ ] unified algebraic source/semantic model implemented
[ ] type-data static domains implemented
[ ] heterogeneous static application implemented
[ ] associated type data implemented
[ ] intrinsic structural domains exposed
[ ] enum canonical source migrated/finalized
[ ] exact-case/GADT behavior preserved
[ ] metadata/import/incremental identity certified
[ ] LSP/source-index baseline certified
[ ] authoritative specifications coherent
[ ] runtime-negative type-data boundary certified
[ ] deferred failures classified
[ ] checkpoint state truthful
[ ] successor action explicit
```

Do not infer `RELEASE_COMPLETE` from this truth table.
''').lstrip()

guidance = dedent(r'''
---
id: LANG007.C3.guidance
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: checkpoint-guidance
title: "Type Data and Unified Algebraic Static Domains"
status: PROPOSED
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
---

# LANG007.C3 — Implementation Guidance

## Type Data and Unified Algebraic Static Domains

## 0. Purpose

This document is the compact architectural guidance for all `LANG007.C3` implementation plans.

Use it with:

```text
LANG007.C3-requirements-analysis.md
LANG007.C3-CHECKPOINT.md
the active P1/P2/P3 patch-grade plan
predecessor walkthrough/handoff
live repository
current authoritative specs
```

This guidance is not a replacement for the active numbered plan.

It fixes the architectural direction that later plans must preserve.

---

# 1. C3 in one sentence

Unify runtime products and sums under `data`, add nominal type-system-only `type data` domains, generalize static generic application and associated lookup around those domains, then expose the compiler's existing structural domains through the same public language without duplicating C1/C2 semantics or introducing runtime static evaluation.

---

# 2. Read this first

C3 is **semantic unification**, not physical representation unification.

Correct:

```text
one source declaration model
one semantic algebraic identity model
one static-domain classification model
one associated namespace authority
one generic application model

multiple optimized internal/runtime representations
    are still allowed
```

Incorrect:

```text
"everything is data"
therefore
"everything must use DataObject storage"
```

Incorrect:

```text
"type data looks like values"
therefore
"create runtime objects for it"
```

Incorrect:

```text
"user static domains are new"
therefore
"build a second type/static-term engine"
```

---

# 3. Non-negotiable inherited C2 rules

Do not redesign:

```text
*
    positional pack

**
    labeled pack

***
    complete pack

...
    NOT pack syntax
```

Do not redesign:

```text
TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks
GenericParameterMode
GenericBinderHeader
whole-pack InferVarId(KindId)
checked pack substitution/materialization
schema-v4 scoped pack metadata
```

Keep:

```text
LabeledTypePack != RecordRow
CompleteTypePack != Tuple
Single(TypePack) != PositionalPack(TypePack)
no pack partition solver
no runtime pack solver
```

If C3 appears to require weakening one of these, STOP AND CONSULT.

---

# 4. Primary architecture decision

## 4.1 Do not create a second static-term universe

The live repository already uses `TypeId` for many non-proper static/type-level terms.

Therefore C3 should extend the current canonical term graph.

Target:

```text
TypeId
    canonical handle for non-row kinded static terms

ProperTypeId
    proof that one TypeId specifically denotes a proper Type

RecordRowId
    canonical row-domain handle, retained separately
```

New type-data/const terms may be represented by `TypeId`.

That does **not** make them proper types.

---

## 4.2 User-defined static domains

Add one dynamic classifier, conceptually:

```rust
KindData::StaticDomain(StaticDomainId)
```

Do not add:

```rust
KindData::Shape
KindData::Endian
KindData::ParserState
...
```

User declarations are data in a domain registry, not Rust enum variants.

---

## 4.3 Heterogeneous application

General canonical envelope:

```rust
enum StaticArgument {
    Term(TypeId),
    RecordRow(RecordRowId),
}
```

Why:

```text
TypeId
    already works for Type, packs, type lambdas, projections,
    and can work for const/type-data terms

RecordRow
    deliberately has separate identity and inference machinery
```

Do not solve this by forcing rows into TypeId.

---

## 4.4 Generic parameter classification

Prefer preserving:

```text
TypeParameterData.kind: KindId
GenericBinderHeader.parameter_kinds
GenericSignature.parameter_kinds
InferenceVariable.kind
```

with dynamic nominal static-domain `KindId`s.

Do not maintain both:

```text
kind
and
domain
```

as independent authorities for the same parameter.

If the implementation introduces a `StaticDomainRef` helper, one representation must be derived from the other.

---

# 5. Unified algebraic declaration model

## 5.1 Four independent axes

Every algebraic declaration has:

```text
phase
    Runtime | TypeOnly

shape
    Product | Sum

implementation provenance
    Phalcom | Native | Intrinsic | ...

ownership
    module | associated nominal owner
```

Never infer one axis from another.

Examples:

```phalcom
data Point(_ x: Int, _ y: Int)
```

```text
phase          Runtime
shape          Product
implementation Phalcom
ownership      module
```

```phalcom
intrinsic type data TypePack
```

```text
phase          TypeOnly
shape          compiler-defined algebraic/static domain
implementation Intrinsic
ownership      Universe module
```

---

## 5.2 Target AST

Prefer one canonical AST shape:

```rust
AlgebraicDef {
    phase,
    shape,
    implementation,
    ...
}
```

Final semantic consumers must not branch:

```text
Statement::Data => product semantics
Statement::Enum => sum semantics
```

P1 may retain a compatibility source flag/span for `enum`.

That flag is presentation/diagnostic state only.

---

# 6. Parser guidance

## 6.1 Braced body discrimination

These are different:

```phalcom
data Person {
    name: String
}
```

record product.

```phalcom
data State {
    Ready
}
```

sum.

Use syntactic shape.

Do not use name resolution.

---

## 6.2 Empty forms

Keep:

```phalcom
data Token()
```

zero-component product, one semantic value per exact runtime specialization.

Keep:

```phalcom
data Impossible {}
```

empty sum, zero inhabitants.

Same algebraic distinction for `type data`.

---

## 6.3 Singleton vs zero-arg constructor

Never collapse:

```phalcom
data X {
    A
    B()
}
```

`A` is singleton associated value.

`B()` is constructor family.

Existing runtime exact-case behavior must continue to observe the distinction.

---

## 6.4 Legacy `enum`

Compatibility route, if P1 keeps one:

```text
parse enum
    ↓
produce canonical Runtime + Sum AlgebraicDef
    ↓
emit migration diagnostic
```

Forbidden:

```text
parse enum
    ↓
EnumDef semantic product
    ↓
later convert to Data
```

if that path creates a separate semantic identity.

P3 removes/finalizes compatibility only after the repository corpus has migrated.

---

# 7. Preserve ADT/GADT machinery

The current enum subsystem is not throwaway code.

Reuse:

```text
VariantId
VariantInfo
VariantConstructorSignature
CaseTypeEnvironment
ExactCase TypeData
enum requirement/default semantics
match exhaustiveness
GADT refinement
exact-case impl/conformance
runtime case lowering
reflection/reification
```

Neutralize ownership gradually.

A safe compatibility technique is a neutral case ID/facade backed by the same owner+selector identity.

Do not mass-rewrite exact-case machinery solely to remove the word `Variant`.

New public APIs and stable metadata should prefer `case`.

---

# 8. One algebraic semantic authority

The target is one authoritative product/facade:

```text
AlgebraicDeclarationInfo
    owner
    phase
    generics
    representation
      Product
      Sum
```

Compatibility views may project:

```text
DataInfo
EnumInfo
VariantInfo
```

for current consumers.

The forbidden architecture is:

```text
AST
├─ independently build AlgebraicInfo
├─ independently build DataInfo
└─ independently build EnumInfo
```

That creates divergent truth.

Derive all adapters from one authoritative declaration product.

---

# 9. Type-data canonical identity

## 9.1 Domain

Every `type data` introduces one nominal static domain.

```phalcom
type data Width(_ value: Int)
type data Height(_ value: Int)
```

must remain distinct.

## 9.2 Term

Canonical term identity is:

```text
domain identity
+ constructor/case identity
+ owner specialization if associated
+ canonical arguments
```

No runtime equality.

No object address.

No user hash.

No runtime method dispatch.

---

# 10. Const/static literal guidance

C3 needs at least compile-time `Int` because accepted source includes:

```phalcom
Shape::Vector(3)
Integer<32, Endian::Big>
```

Do not parse these literals as runtime expression evaluation.

Represent supported static constants canonically in the static term system.

Conceptually:

```rust
TypeData::ConstInt(3)
```

classified by the compiler-supported static Int domain.

Do not automatically lift arbitrary runtime `Int` expressions.

---

# 11. ProperTypeId is sacred

Any C3 patch that makes this legal internally:

```text
Shape::Vector(3)
    accepted where API requires ProperTypeId
```

is unsound.

The fact that both are stored behind `TypeId` is an implementation detail.

Always preserve:

```text
kind/domain == Type
    required for ProperTypeId
```

C3 tests must contain negative proper-type boundary cases.

---

# 12. StaticArgument guidance

## 12.1 Canonical representation

```rust
StaticArgument::Term(TypeId)
StaticArgument::RecordRow(RecordRowId)
```

or exact equivalent.

## 12.2 `Applied`

Generalize:

```rust
TypeData::Applied {
    origin,
    arguments: Box<[StaticArgument]>,
}
```

## 12.3 Compatibility API

Retain a convenient type-only helper.

Example:

```rust
apply_type_form(origin, &[TypeId])
```

may internally wrap:

```rust
StaticArgument::Term
```

Use a generalized canonical helper for heterogeneous application.

Do not force every current caller into noisy migration if an adapter is safe.

---

# 13. RecordRow guidance

RecordRow remains:

```text
unordered/keyed row identity
open-tail semantics
row-specific solver
```

It is not:

```text
LabeledTypePack
```

and it is not to be rewritten as a fake TypeId term merely for C3 application storage.

If heterogeneous application needs a row parameter:

- use the existing canonical row representation;
- preserve row-tail parameter/projection identity;
- let `StaticArgument` carry it.

---

# 14. Scoped aliases/type lambdas

When `Applied` becomes heterogeneous, remember that aliases/type lambdas are a separate representation layer.

Audit:

```text
ScopedTypeData::Applied
TypeLambdaData
beta reduction
scoped substitution
stable ScopedTypeNode
metadata export
```

A P2 implementation that changes only global `TypeData::Applied` is incomplete.

Do not wait until metadata tests to discover this.

---

# 15. Generic binder guidance

## 15.1 Domain vs mode

Keep:

```text
P: TypePack
    domain = TypePack
    mode = Single

*Ts
    domain = TypePack
    mode = PositionalPack
```

Likewise:

```text
S: Shape
    domain = Shape
    mode = Single
```

## 15.2 Variance

Do not invent user static-domain variance semantics in C3.

Default/invariant unless an existing rule already applies safely.

---

# 16. Inference guidance

## 16.1 Reuse InferVarId

For user domain:

```text
Shape
```

create ordinary query-local:

```text
InferVarId(kind = StaticDomain(Shape))
```

No new static-var ID family.

## 16.2 First-order structural rules

Same constructor:

```text
Vector(N) = Vector(3)
    => N = 3
```

Different constructor:

```text
Scalar != Vector(N)
```

Different domain:

```text
Shape::Vector(3) != Other::Vector(3)
```

## 16.3 No evaluator

Do not implement:

```text
if static term has method, call method
run user function
execute match
evaluate arbitrary closure
```

C3 terms are closed canonical algebra, not general compile-time programs.

---

# 17. Type members

The existing type-member subsystem already demonstrates heterogeneous domains.

P2 should converge it toward the common static-domain model.

Do not leave:

```text
generic application domain model A
type-member domain model B
```

as independent sources of truth.

Compatibility enums/wrappers are acceptable during migration if they translate to the canonical domain/argument model.

---

# 18. Associated `type data`

## 18.1 Required semantics

```phalcom
class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

One associated namespace:

```text
Parser::State
Parser::State::Start
Parser::Initial
```

## 18.2 Generic owner

```phalcom
class C<T: Type> {
    type data State {
        Empty
        Present(_ value: T)
    }
}
```

Keep:

```text
declaration identity:
    C::State

specialization:
    C<Int>::State
    C<String>::State
```

Do not duplicate the declaration per specialization.

---

# 19. AssociatedSurface is the lookup owner

Current:

```text
AssociatedSurface
AssociatedFamilyInfo
AssociatedFamilyKind
AssociatedMemberId
AssociatedResolution
```

Extend this system.

Do not create a second lookup pipeline solely for static domains.

Namespace collision diagnostics must cover:

```text
case family
data constructor
associated static domain
type/static member
associated callable family
other declaration-owned associated base
```

No source-order winner.

---

# 20. Trait-associated types are different

Keep this distinction explicit in code and docs:

```text
trait associated type
    evidence-bound requirement
    supplied/specialized by conformance

associated type data
    inherent declaration-owned static domain
    specialized by owner environment
```

Shared syntax/lookup helper does not imply shared evidence ownership.

C3 does not add:

```phalcom
trait X {
    type data State
}
```

as a requirement form.

---

# 21. Trait generic boundary

Do not let storage capability define language semantics.

If a generalized `TypeId` can physically contain a type-data term, that does not prove trait coherence/evidence knows how to reason about it.

Default C3 policy:

```text
reject unsupported user type-data domains on trait generic parameters
```

until explicitly migrated.

If a P2 plan decides to support them:

- audit TraitRef;
- conformance head identity;
- coherence;
- substitution;
- evidence environment;
- metadata;
- inference;
- diagnostics;
- incremental fingerprints.

That becomes a deliberate P2 architecture item, not a side effect.

---

# 22. Metadata guidance

## 22.1 Stable facts

Metadata must encode:

```text
phase
shape
declaration
case
static domain
static term
StaticArgument
associated static domain
owner specialization
generic parameter domain
```

## 22.2 Identity

Never encode identity by:

```text
"Shape::Vector(3)" string
rendered type name
source path text without stable module identity
raw TypeId
raw StaticDomainId
raw InferVarId
```

## 22.3 Schema

Baseline is C2 schema v4.

Every public encoding change is explicitly versioned.

Do not silently add meaning to v4.

Do not rewrite historical fixtures merely to avoid compatibility handling.

---

# 23. Incremental guidance

Fingerprint semantic facts, not syntax accidents.

Should change:

```text
data -> type data
Product -> Sum
case set
payload domain
GADT result
static-domain binder
static argument
associated domain owner/environment
```

Should not change solely for:

```text
equivalent legacy enum -> data migration
source range movement
runtime representation selection
private helper rename
```

Cold and incremental final states must agree.

---

# 24. Runtime guidance

## 24.1 Runtime data

Keep specialized runtime paths.

No requirement to merge:

```text
DataObject
AdtCaseObject
TupleObject
RecordObject
immediates
```

## 24.2 Type data

Never add runtime inhabitants.

Static term formation stops in semantic/compiler metadata.

## 24.3 Intrinsic data

`intrinsic data Tuple<***Es>` means:

```text
source-visible semantic declaration
backed by compiler/runtime-owned specialized representation
```

It does not mean:

```text
construct a generic DataObject tuple
```

---

# 25. TupleEntries guidance

`TupleEntries` is the public static-domain name for the complete two-lane tuple entry shape.

Canonical internal authority stays:

```text
KindId::COMPLETE_TYPE_PACK
TypeData::CompleteTypePack
project_complete
project_positionals
project_labeled
```

Required public equivalences:

```text
T::Entries == ***T
T::Positionals == *T
T::Labeled == **T
```

Do not introduce:

```text
TypeData::TupleEntries
```

if it would be a second representation of the same semantic term.

A source declaration descriptor/facade is fine.

---

# 26. Universe migration guidance

Do not migrate Universe before P2 static-domain semantics are stable.

P3 migration order should be:

```text
1. intrinsic static-domain declarations
2. intrinsic Unit/Tuple/Record declaration facades
3. structural type members
4. core closed sums Option/Result/Ordering -> data
5. remaining repository-owned enum source
6. tooling/presentation terminology
7. remove/finalize enum compatibility parser
```

Every migration preserves canonical declaration/module identity.

---

# 27. Specification migration guidance

The implementation must not end C3 with source behavior contradicting `docs/specs`.

P1 should begin semantic-owner updates.

P3 finishes cleanup.

Do not rewrite historical implementation records.

Primary owners:

```text
docs/specs/objects/data-objects.md
docs/specs/objects/enums.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/associated-types.md
docs/specs/objects/impl.md
docs/specs/universe.md
pattern/GADT owning specs
```

---

# 28. P1 boundaries

P1 owns:

```text
canonical algebraic AST
data Product/Sum
runtime sum migration
type data top-level
StaticDomainId foundation
canonical static-data terms
proper-type rejection
metadata foundation
```

P1 does not own:

```text
general heterogeneous Applied
associated type data
full static generic inference
Universe migration
LSP completion
corpus removal of enum
```

If P1 starts changing `TraitRef` or associated domain specialization, it is drifting into P2.

---

# 29. P2 boundaries

P2 owns:

```text
StaticArgument
heterogeneous Applied
generic domain resolution
static argument grammar
domain inference
checked substitution
static aliases
type-member domain convergence
associated type data
:: lookup
metadata/import/incremental/source-index
```

P2 does not own:

```text
Universe final migration
enum final removal
general static evaluator
trait type-data requirements
```

---

# 30. P3 boundaries

P3 owns:

```text
intrinsic source surface
TupleEntries bridge
Universe migration
core sum migration
enum final compatibility decision
tooling
corpus
spec closure
certification
```

P3 should mostly consume P1/P2 architecture.

If P3 requires a new static-term carrier or new associated-domain identity model, P2 was incomplete: STOP AND CONSULT rather than inventing it in P3.

---

# 31. Recommended P1 planning emphasis

A P1 patch-grade plan should pre-decide:

1. exact AlgebraicDef AST shape;
2. exact compatibility handling for legacy enum;
3. product/sum braced-body parser discrimination;
4. record-payload case representation;
5. neutral case identity migration technique;
6. authoritative semantic product and adapter strategy;
7. exact static-domain store representation;
8. exact type-data term TypeData variants;
9. static Int term formation needed by type-data constructors;
10. exact metadata schema transition;
11. runtime compiler adapter path;
12. focused ADT/GADT preservation gates.

Do not leave these decisions to implementation improvisation.

---

# 32. Recommended P2 planning emphasis

A P2 patch-grade plan should pre-decide:

1. exact `StaticArgument` enum;
2. `Applied` migration API;
3. scoped heterogeneous argument representation;
4. row symbolic argument representation;
5. static argument syntax AST;
6. generic binder-domain resolution;
7. type-data inference term variants;
8. materialization failures;
9. type-member domain migration;
10. associated-domain declaration IDs;
11. owner specialization representation;
12. AssociatedSurface categories;
13. trait unsupported/supported boundary;
14. metadata schema transition;
15. incremental dependency keys;
16. source-index/LSP semantic targets.

---

# 33. Recommended P3 planning emphasis

A P3 patch-grade plan should pre-decide:

1. exact Universe source files to migrate;
2. intrinsic declaration-to-canonical-domain registration;
3. TupleEntries mapping;
4. static type-member source declarations;
5. Option/Result/Ordering migration ordering;
6. enum compatibility removal point;
7. corpus migration scope;
8. LSP required interactions;
9. metadata/interface compatibility gates;
10. performance/resource evidence;
11. runtime-negative audit;
12. C3 closure truth table.

---

# 34. Testing doctrine

Testing is evidence gathering.

During P1/P2 BUILD mode, do not run workspace tests reflexively.

Use:

```text
T0 exact new regression
T1 direct feature module
T2 owning subsystem
T3 adjacent integration
T4 affected crate
T5 workspace/release only at explicit certification
```

P3 may deliberately broaden because it is checkpoint closure.

---

# 35. Coverage dimensions that matter

Prioritize:

```text
canonical identity
negative domain behavior
empty/singleton/zero-arg boundaries
GADT preservation
heterogeneous composition
aliasing
cross-module persistence
cold/incremental equality
source identity
metadata compatibility
runtime-negative behavior
tooling identity
```

Do not add concurrency/GC testing merely because it exists elsewhere unless a C3 change actually affects those mechanisms.

---

# 36. Baseline failures

Inherited:

```text
BASELINE-001
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Do not repair during C3 unless causal evidence shows a C3-owned regression.

Classify:

```text
A caused -> fix
B probably caused -> investigate/fix
C unclear -> one bounded pass, then defer if nonblocking
D unrelated -> record/defer
```

---

# 37. Debugging budget

Mechanical:

```text
up to ~3 coherent correction cycles with progress
```

Semantic:

```text
record:
Observed
Hypothesis
Evidence
Prediction
Discriminating test

allow one serious correction
then STOP AND CONSULT
```

Architecture:

```text
zero speculative redesign attempts
STOP AND CONSULT immediately
```

---

# 38. Global STOP / CONSULT triggers

Stop if any of these occur:

1. new static terms require weakening ProperTypeId;
2. a second general static-term arena is about to be introduced;
3. every user static domain would require a Rust enum variant;
4. RecordRow must be collapsed into TypeId/LabeledTypePack;
5. Data/Enum convergence loses exact-case/GADT evidence;
6. legacy enum/data create different identity;
7. sum behavior requires a second ADT implementation;
8. generic application cannot become heterogeneous without wholesale trait-system redesign;
9. unsupported trait static-domain behavior would be silently accepted;
10. associated type data needs a separate name-resolution authority;
11. associated domain identity requires rendered strings;
12. trait associated-type evidence is being reused for inherent type data;
13. TupleEntries cannot map directly to CompleteTypePack;
14. intrinsic Tuple/Record would require replacing current runtime representation;
15. static type-data correctness requires VM execution;
16. arbitrary runtime `Expr` is being reused as compile-time evaluation;
17. arbitrary static functions/pattern evaluation become necessary;
18. metadata would serialize raw store/session IDs;
19. cold/incremental equality cannot be restored without global invalidation;
20. a C2 pack invariant needs weakening;
21. the same semantic failure survives one serious correction;
22. source/spec/test authorities materially conflict.

---

# 39. Consultation packet

Use:

```markdown
# C3 IMPLEMENTATION INCIDENT

## Identity
Plan/task/gate/revision

## Invariants
Exact C3-I / INV-C3 IDs

## Observed
Concrete symptom

## Minimal reproduction
Exact source/test/command

## Relevant path
A -> B -> C

## Evidence
Code/product/test facts only

## Hypothesis
Best causal explanation

## Serious correction already attempted
If any

## Architectural alternatives
Only plausible alternatives

## Decision requested
One narrow question
```

---

# 40. Checkpoint bookkeeping

At P1 start:

```text
C3 status        -> IN_PROGRESS
completion       -> PARTIAL
verification     -> UNVERIFIED
active plan      -> P1
PROGRAM/STATUS   -> rebaseline C3
```

At each coherent plan gate:

- record durable new architecture;
- record actual verification;
- record consultation decisions;
- record deferred failures.

Do not turn CHECKPOINT into an edit log.

At P3 closure:

```text
P1/P2/P3
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

C3
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

only if the checkpoint truth table passes.

---

# 41. Required plan completion artifacts

Every P1/P2/P3 normally produces:

```text
LANG007.C3.P<n>-walkthrough.md
LANG007.C3.P<n>-handoff.md
```

P3 handoff should summarize the frozen C3 substrate and the next accepted LANG007 roadmap action.

---

# 42. Do not re-explore

Unless live evidence contradicts the baseline, do not spend implementation time re-deciding:

```text
whether data should replace enum
whether type data has runtime values
whether type-data equality calls user ==
whether packs use * ** ***
whether TupleEntries should replace CompleteTypePack
whether RecordRow == LabeledTypePack
whether GenericParameterMode == static domain
whether the VM should solve type data
whether associated type data is a trait requirement
whether runtime Tuple must become DataObject
```

These are already decided for C3.

---

# 43. What still requires per-plan verification

Verify against live source before each plan:

```text
exact AST/parser symbols
exact DB query products
exact adapter consumers
exact metadata schema version
exact trait/conformance argument assumptions
exact associated namespace consumers
exact Universe source locations
exact LSP product APIs
exact test module names
```

Mechanical drift is expected.

Architectural drift is not.

---

# 44. Final architectural summary

```text
one algebraic language
    data       -> runtime inhabitants
    type data  -> static inhabitants

one canonical static tower
    KindId classifies
    TypeId stores non-row static terms
    ProperTypeId proves actual Type
    RecordRow keeps dedicated authority

one generic application model
    StaticArgument

one non-row inference identity
    InferVarId(kind)

one associated lookup authority
    AssociatedSurface / ::

one stable persistence model
    declaration/domain/case/argument structure

specialized runtime representations
    remain free beneath the language model

no general compile-time evaluator
no VM static solver
```

If later implementation work remains inside these boundaries, C3 can be delivered without destabilizing C1/C2.
''').lstrip()

files = {
    "LANG007.C3-requirements-analysis.md": req,
    "LANG007.C3-CHECKPOINT.md": checkpoint,
    "LANG007.C3-GUIDANCE.md": guidance,
}

for name, content in files.items():
    p = outdir / name
    p.write_text(content, encoding="utf-8")
    print(f"{name}: {p.stat().st_size:,} bytes, {len(content.splitlines()):,} lines")

# Also create the user's literal typo filename as a byte-identical convenience alias
# without making it the canonical repository name.
alias = outdir / "LANG007.3C3-GUIDANCE.md"
alias.write_text(guidance, encoding="utf-8")
print(f"{alias.name}: {alias.stat().st_size:,} bytes, {len(guidance.splitlines()):,} lines (alias)")
