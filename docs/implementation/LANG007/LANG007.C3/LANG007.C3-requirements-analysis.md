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
