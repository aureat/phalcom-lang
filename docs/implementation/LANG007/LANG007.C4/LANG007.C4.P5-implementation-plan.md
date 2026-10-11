---
id: LANG007.C4.P5
category: LANG
program: LANG007
checkpoint: LANG007.C4
plan: LANG007.C4.P5
kind: implementation-plan
title: "Product/Sum Reflection and Compiler-Derived Structural Capabilities"
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: c02e74757e5f7ebcd6a58debcd157936a475b009
prepared: 2026-10-11
depends_on:
  - LANG007.C4.P3
  - LANG007.C4.P4
consumes_foundation:
  - LANG007.C4.P10
blocks:
  - LANG007.C4.P6
  - LANG007.C4.P7
  - LANG007.C4.P8
  - LANG007.C4.P9
---

# LANG007.C4.P5 — Product/Sum Reflection and Compiler-Derived Structural Capabilities

## Patch-grade implementation plan grounded in the live repository

---

# 0. Executor contract

This plan implements the C4 public structural capability/reflection layer **on top of existing semantic authorities**. It is not permission to redesign Pack, Record rows, generic conformance, ADT semantics, static values, runtime product layout, or the P10 structural classifier.

The implementer must:

- re-read the live files named by each task before editing;
- rebaseline `main` and the active P4 state before production changes;
- preserve `LANG007.C4.P4` as the owner of generalized `FieldKey`, Record-row semantics, unordered lookup, Record pattern semantics, and computed-key formation;
- preserve `LANG007.C4.P10` as the owner of the lower structural-classification foundation already merged into `main`;
- use the existing `StaticValueDomain` model rather than inventing a second static-domain taxonomy;
- make type/member projection semantics domain-neutral without encoding ordinary static values as fake `TypeId`s;
- extend the existing generic conformance proof authority rather than creating a second structural-trait solver;
- give compiler-derived structural evidence explicit provenance rather than fabricating source `ImplId`s;
- derive both capability and public reflection from lower canonical structural facts; neither may be used to prove the other;
- use `DataInfo` / `DataSemanticTable`, `EnumInfo` / `VariantInfo` / `EnumSemanticTable`, `TypeStore`, and P4 row facts as canonical semantic inputs;
- preserve nominal type identity even when two reflected shapes are equal;
- preserve Symbol-only Tuple/Pack/call labels and heterogeneous `FieldKey` only for unordered Record/product fields;
- keep public Variant reflection shallow and declaration ordered;
- keep GADT branch proof/refinement metadata private to semantic matching;
- use ordinary `StaticValueStore` nodes (`Data`, `Variant`, `Pack`, `Record`, `Type`, `Symbol`) for reduced public reflection values;
- keep P8 responsible for durable cross-module static-value serialization/materialization;
- keep P6 responsible for general static `for` / `match` / proof control;
- keep P7 responsible for general recursive static computation/residualization;
- keep P9 responsible for final ontology/tooling/documentation retirement and checkpoint-wide certification;
- STOP AND CONSULT when a fixed authority boundary in §4 or a stop condition in §21 fails.

A source edit alone is not completion evidence. Each task has explicit focused gates.

---

# 1. Repository anchor and current lifecycle truth

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   c02e74757e5f7ebcd6a58debcd157936a475b009
date:       2026-10-11
```

The current C4 ledger on `main` records:

```text
P3   COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P4   IN_PROGRESS / PARTIAL / UNVERIFIED
P10  IN_PROGRESS / PARTIAL / UNVERIFIED
P5   PROPOSED / NOT_STARTED / UNVERIFIED
```

`P4` explicitly blocks `P5`. P4's active corrective completion plan still requires owner-orchestration/replay closure, computed-key alias closure, T7.0E acceptance, Record-pattern work, builder complexity hardening, metadata/API/incremental closure, and final lifecycle certification.

Therefore:

> **P5 may prepare and implement P4-independent foundations, but P5 cannot reach its unordered-product acceptance gate or be marked complete until the final P4 authority is integrated.**

P10 has already delivered the correct lower-level structural substrate for ordered products and sums. P5 must consume it rather than reimplement it.

---

# 2. Live facts that determine the P5 implementation

## 2.1 Lower structural classification already exists

Live file:

```text
phalcom-semantic/src/structural.rs
```

The file explicitly states that it is below public capability/reflection and uses exact semantic types and declaration products.

Current classification:

```rust
pub enum StructuralClassification {
    Unsupported,
    OrderedProduct(OrderedProductForm),
    Sum { owner: DeclarationId },
}
```

Current ordered forms:

```rust
pub enum OrderedProductForm {
    Unit,
    Tuple,
    TupleFromPacks,
    Pack,
    PackFromPacks,
    Data { owner: DeclarationId },
}
```

Current shallow ordered descriptor:

```rust
pub struct OrderedProductStructure {
    pub ty: TypeId,
    pub origin: OrderedProductOrigin,
    pub positional_entries: Box<[OrderedProductEntry]>,
    pub labeled_entries: Box<[OrderedProductEntry]>,
    pub positional_remainder: AnonymousTupleRemainder,
    pub labeled_remainder: AnonymousTupleRemainder,
    pub symbolic_lanes: Option<(TypeId, TypeId)>,
    pub state: DataProductShapeState,
}
```

Current shallow Sum descriptor:

```rust
pub struct SumStructure {
    pub ty: TypeId,
    pub owner: DeclarationId,
    pub variants: Box<[SpecializedVariantStructure]>,
}
```

This foundation already preserves:

- Unit;
- Tuple;
- Tuple-from-packs;
- first-class Pack;
- Pack-from-packs;
- tuple-shaped nominal data;
- Sum declaration identity;
- declaration-ordered exact Variants;
- specialized immediate payload Type references;
- repeated Variant family identity;
- shallow recursive structure.

It deliberately does **not** yet classify Record/unordered products.

### P5 consequence

`structural.rs` remains the lower authority. P5 extends it only where P4's finalized unordered facts are required.

Do not create:

```text
ProductClassifier
ReflectionClassifier
SumReflectionClassifier
StructuralTraitClassifier
```

as parallel systems.

---

## 2.2 Type-member domains are still kind-only

Live file:

```text
phalcom-semantic/src/types/type_member.rs
```

Current model:

```rust
pub enum TypeMemberDomain {
    Kind(KindId),
}
```

But P2 already established:

```rust
pub enum StaticValueDomain {
    Kind(KindId),
    ValueType(TypeId),
}
```

in:

```text
phalcom-semantic/src/static_domain.rs
```

The current adapter intentionally returns `None` for `ValueType`.

This is now a hard blocker because normative P5 members return ordinary static values:

```text
Product::Kind        : ProductKind
Product::Shape       : ProductShape
OrderedProduct::*    : Pack / OrderedShape values
UnorderedProduct::*  : UnorderedShape values
Sum::Variants        : Variants = PositionalPack<Variant>
```

These are not kinded `TypeId` terms.

---

## 2.3 Member parameter domains are also incomplete

The AST already has:

```rust
pub enum TypeMemberParameterDomainSyntax {
    Kind(KindSyntax),
    ConstType(TypeAnnotation),
}
```

but the checker currently lowers `ConstType(annotation)` through:

```text
resolve_type_form
→ static_domain_kind_of_form
→ TypeMemberDomain::Kind(...)
```

rather than retaining an ordinary `ValueType(TypeId)` domain.

That means the syntax model anticipates static value-domain parameters, but semantic representation collapses them back into kinds.

This directly blocks:

```phalcom
impl UnorderedShape {
    intrinsic type Lookup<K: FieldKey>: FieldLookup
}
```

because `K` is a **static value of ordinary type `FieldKey`**, not a proper Type term or a `ConstInt` kinded term.

---

## 2.4 Type-member arguments are not general static values

Current:

```rust
pub enum ConstTerm {
    Int(i64),
}

pub enum TypeMemberArgument {
    Type(TypeId),
    Const(ConstTerm),
    RecordRow(RecordRowId),
}
```

Current validation reports domain mismatch only in `KindId` terms.

This is adequate for:

```text
At<I: Int-static-domain>
Map<F: Type -> Type>
RecordRow operations
```

but not:

```text
Lookup<K: FieldKey>
```

or future ordinary-value arguments.

P5 therefore requires a genuine domain-neutral member-argument representation.

---

## 2.5 Existing symbolic projection is stored in TypeStore and is kinded-term oriented

Current symbolic carrier:

```rust
pub struct TypeLevelProjection {
    pub subject: ProjectionSubject,
    pub selector: TypeMemberSelector,
    pub arguments: Box<[TypeMemberArgument]>,
    pub source_declaration: Option<TypeMemberDeclarationId>,
}
```

It is stored as a TypeStore node and participates in:

```text
types/store.rs
types/projection.rs
types/environment.rs
types/substitution.rs
types/instantiation.rs
types/annotation.rs
checker/inference.rs
checker/gadt_proof.rs
metadata/export.rs
```

This machinery must remain valid for kinded/static type terms.

However the normative type-member spec now says:

```text
Projection(subject, member declaration identity, member arguments, applicable evidence)
```

and explicitly states:

> A projection is domain-neutral. It is not required to be represented as a proper type merely because it appears in type-level syntax.

### P5 consequence

P5 must generalize the **semantic projection model** without forcing ordinary `StaticValueRef` identity into the TypeStore type-node arena.

The preferred architecture is defined in §5.2.

---

## 2.6 Trait-associated declarations are hard-coded to `Type`

Current `TraitAssociatedTypeRequirement` retains:

```text
requirement identity
name
kind: KindId
source
```

and `build_trait_surface(...)` currently publishes every associated declaration with:

```rust
kind: KindId::TYPE
```

The AST parser currently accepts only:

```phalcom
type Item
```

for trait-associated declarations. The parser's comment is explicitly:

```text
trait-owned associated type declaration (`type Item`)
```

and no `: resultDomain` is parsed.

This contradicts the current normative associated-type/type-member specification, which permits an explicit associated-member result domain.

P5's canonical traits need:

```phalcom
type Kind: ProductKind
type Shape: ProductShape
type OrderedShape: OrderedShape
type Variants: Variants
```

Therefore P5 must complete associated declaration result-domain syntax and semantic storage.

---

## 2.7 Associated conformance bindings remain Type/Dynamic only

Current:

```rust
pub enum AssociatedTypeBindingValue {
    Type(TypeId),
    Dynamic,
}
```

For P5 compiler-derived structural traits, reduced associated values are ordinary static values, not TypeIds.

P5 should not force arbitrary user-authored ordinary-value associated bindings into scope unless necessary, but the core evidence/result carrier must become capable of representing P5's compiler-derived static values without lying about their domain.

---

## 2.8 Generic conformance evidence has no compiler-derived structural provenance

Current:

```rust
pub enum GenericConformanceEvidence {
    Exact(Arc<ConformanceEvidence>),
    Assumed(AssumedConformanceEvidence),
    Derived(Arc<DerivedConformanceEvidence>),
}
```

`Derived` means either:

```text
ConditionalSource
GuardedRule
```

and source-derived evidence is tied to source `ImplId` / conformance products.

Normative P5 requires:

```text
Product
OrderedProduct
UnorderedProduct
Sum
```

to be **compiler-derived sealed capabilities**.

P5 cannot model these by generating fake source impls.

---

## 2.9 The generic proof engine is the correct integration owner

Current direct proof authority:

```text
phalcom-semantic/src/generic_evidence.rs
    ConformanceProofContext
    evaluate_conformance_goal
    prove_conformance_with_context
    query_generic_conformance_proof
```

The proof order is currently:

```text
1. lexical assumptions
2A. nominal exact/conditional source conformances
2B. guarded derived rules
3. classify success/failure/coherence
```

P5 should add compiler-derived structural candidate proof **inside this authority**, not beside it.

The context already owns:

```text
memo
goal stack
budget
cancellation
trait surfaces
conformance index
derived rules
declarations
hierarchy
TypeStore
```

P5 will need to provide the lower structural authorities to this context in a bounded way.

---

## 2.10 Adding a new evidence variant has multiple consumers

Live references to `GenericConformanceEvidence::*` occur across:

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/iteration.rs
phalcom-semantic/src/checker/algebraic_realization.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
```

plus semantic/core tests.

P5 must audit every exhaustive match. Structural evidence has no source witness callable and should not silently enter paths that expect an `ImplId`.

---

## 2.11 P4 already provides the correct unordered lookup semantic primitive

Live:

```text
phalcom-semantic/src/types/relation.rs
```

contains:

```rust
pub enum UnorderedFieldLookup {
    Required(TypeId),
    Optional(TypeId),
    Absent,
    Unknown(RelationOutcome),
}
```

and:

```rust
pub fn lookup_unordered_field(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    row_id: RecordRowId,
    key: &FieldKey,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> UnorderedFieldLookup
```

This is the authority P5 must reuse for public:

```phalcom
UnorderedShape::Lookup<K>
```

P5 must not create reflection-local row lookup semantics.

---

## 2.12 StaticValueStore already has the needed reduced-value substrate

Current `StaticValueStore` supports:

```text
StaticValueData::Type
StaticValueData::Tuple
StaticValueData::Record
StaticValueData::Data
StaticValueData::Variant
StaticValueData::Pack
StaticValueData::Symbol
```

and validated constructors including:

```text
intern_data(...)
intern_variant(...)
intern_record(...)
intern_pack(...)
```

Therefore P5 does **not** need a new `StaticValueData::ProductShape` or reflection-specific value arena.

Reduced P5 reflection should be ordinary immutable Telos data.

---

## 2.13 Canonical Universe identity is exact and source-path based

Current:

```text
phalcom-semantic/src/core_surface/identity.rs
phalcom-native-meta/src/universe.rs
```

provide:

```rust
pub fn universe_declaration(key: UniverseKey) -> DeclarationId
```

where `UniverseKey::source_path()` is canonical semantic ownership.

Current `CoreDeclarationIds` does not yet include P5 structural reflection declarations.

P5 must register exact canonical identities. No source-name comparison is acceptable.

---

## 2.14 The public P5 structural surface is not yet present in Universe source

Current `universe.reflection` package exposes module/project/type-oriented reflection but no structural-reflection module.

The P5 source declarations do not exist yet for:

```text
Product
OrderedProduct
UnorderedProduct
Sum

ProductKind
ProductShape
OrderedTail
OrderedShape

Presence
FieldInfo
ProductField
UnorderedTail
UnorderedShape
FieldLookup
UnorderedShapeView

Variant
Variants
VariantNames
```

P5 must add actual canonical Universe declarations.

---

# 3. Normative target

P5 must deliver the following public semantic model.

## 3.1 Product capabilities

```phalcom
trait Product {
    type Kind: ProductKind
    type Shape: ProductShape
}

trait OrderedProduct {
    type OrderedShape: OrderedShape
    type Entries: CompletePack<Type>
    type PositionalEntries: PositionalPack<Type>
    type LabeledEntries: LabeledPack<Type>
}

trait UnorderedProduct {
    type UnorderedShape: UnorderedShape
}
```

The traits are independent. The compiler proves both relevant capabilities from one lower classification:

```text
ordered product
    -> Product
    -> OrderedProduct

unordered product
    -> Product
    -> UnorderedProduct
```

No inheritance relationship is required between the traits.

---

## 3.2 Product reflection data

```phalcom
intrinsic data ProductKind {
    Ordered
    Unordered
}

intrinsic data ProductShape {
    Ordered(_ shape: OrderedShape)
    Unordered(_ shape: UnorderedShape)
}
```

---

## 3.3 Ordered structural data

```phalcom
intrinsic data OrderedTail<Rest> {
    Closed
    Typed(_ valueType: Type)
    Unconstrained
    Captured(_ rest: Rest)
}

type PositionalTail =
    OrderedTail<PositionalPack<Type>>

type LabeledTail =
    OrderedTail<LabeledPack<Type>>

intrinsic data OrderedShape(
    _ entries: CompletePack<Type>,
    _ positionalTail: PositionalTail,
    _ labeledTail: LabeledTail,
)
```

---

## 3.4 Unordered structural data

```phalcom
intrinsic data Presence {
    Required
    Optional
}

intrinsic data FieldInfo(
    _ type: Type,
    _ presence: Presence,
)

intrinsic data ProductField(
    _ key: FieldKey,
    _ type: Type,
    _ presence: Presence,
)

type FixedFields =
    Record<**[FieldKey]: FieldInfo>

intrinsic data UnorderedTail {
    Closed
    Typed(_ keyType: Type, _ valueType: Type)
    Unconstrained(_ keyType: Type)
    Captured(_ rest: UnorderedShape)
}

intrinsic data UnorderedShape(
    _ fields: FixedFields,
    _ tail: UnorderedTail,
)

intrinsic data FieldLookup {
    Required(_ type: Type)
    Optional(_ type: Type)
    Absent
    Unknown
}

intrinsic data UnorderedShapeView {
    Empty(_ tail: UnorderedTail)
    Field(_ field: ProductField, _ rest: UnorderedShape)
}

impl UnorderedShape {
    intrinsic type Lookup<K: FieldKey>: FieldLookup
    intrinsic type View: UnorderedShapeView
    intrinsic type Tail: UnorderedTail
}
```

---

## 3.5 Sum capability/reflection

```phalcom
trait Sum {
    type Variants: Variants
}

intrinsic data Variant(
    _ name: Symbol,
    _ shape: ProductShape,
)

type Variants =
    PositionalPack<Variant>

type VariantNames =
    PositionalPack<Symbol>
```

Variant order is declaration order. Repeated Symbol names remain separate exact alternatives.

---

# 4. Fixed authority matrix

This table is normative for implementation decisions.

| Semantic fact | Existing owner P5 must use |
|---|---|
| Product/Sum category | `StructuralClassifier` in `phalcom-semantic/src/structural.rs` |
| ordered entries/tails | `OrderedProductStructure` |
| nominal product shape | `DataInfo`, `materialize_data_product_shape` |
| Sum declaration | `EnumInfo` |
| exact Variant | `VariantInfo`, `VariantId` |
| repeated-name family | `VariantFamilyId` |
| exact specialized case | `ExactCaseTypeReflection` / exact-case TypeStore forms |
| heterogeneous key identity/order | `FieldKey`, `FieldKey::canonical_cmp` |
| Record row/tail | `RecordRowData`, `RecordRowTail`, P4 row store |
| unordered field lookup | `lookup_unordered_field` |
| generic conformance proof | `prove_conformance_with_context` |
| lexical generic assumption | `GenericEvidenceEnvironment` |
| canonical associated requirement | `AssociatedTypeRequirementId` / `TraitSurface` |
| static domain | `StaticValueDomain` |
| kinded symbolic projection | existing TypeStore `TypeLevelProjection` |
| reduced static reflection value | `StaticValueStore` |
| runtime physical product layout | existing `phalcom-core` product runtime; **not semantic source** |
| durable static metadata/materialization | P8, not P5 |

Hard rules:

```text
capability != reflection
reflection != capability
runtime layout != semantic structural identity
name spelling != canonical structural identity
```

Both capability and reflection consume the lower structural facts.

---

# 5. Architectural decisions P5 must adopt

## 5.1 Converge `TypeMemberDomain` with `StaticValueDomain`

P2 intentionally left the adapter seam for P5.

Preferred end-state:

```rust
pub enum TypeMemberDomain {
    Kind(KindId),
    ValueType(TypeId),
}
```

with total conversions:

```text
TypeMemberDomain <-> StaticValueDomain
```

or, if module boundaries allow without cycles, replace `TypeMemberDomain` with `StaticValueDomain` directly in type-member signatures.

Do not preserve a permanent model where:

```text
static definitions understand ValueType
type members do not
```

### Required semantics

```text
Kind(Type)
    accepts/provides kinded Type terms

ValueType(ProductShape)
    accepts/provides ordinary static ProductShape values

ValueType(FieldKey)
    accepts ordinary static FieldKey values
```

---

## 5.2 Generalize projection identity without polluting TypeStore with ordinary value-arena IDs

The existing `TypeLevelProjection` is a TypeStore node and is deeply integrated into type substitution/inference/metadata. Preserve it for kinded result domains.

Introduce one **domain-neutral projection identity layer** above representation:

```rust
pub struct MemberProjection {
    pub subject: MemberProjectionSubject,
    pub declaration: MemberDeclarationIdentity,
    pub arguments: Box<[MemberProjectionArgument]>,
    pub evidence: Option<...>,
    pub result_domain: StaticValueDomain,
}
```

Possible argument categories:

```rust
pub enum MemberProjectionArgument {
    Kinded(TypeMemberArgument),
    StaticValue(StaticValueRef),
}
```

or a more general snapshot-safe static term/value reference if one already exists after live re-read.

Representation rule:

```text
result domain Kind(_)
    -> may lower to existing TypeLevelProjection / TypeStore node

result domain ValueType(_)
    -> must not become TypeData
    -> reduce to StaticValueRef when closed
    -> remain a symbolic MemberProjection when valid-but-stuck
```

This preserves the normative **one projection semantics** while allowing two storage representations appropriate to the result domain.

Do not embed `StaticValueRef` into ordinary `TypeData::TypeMemberProjection` merely to avoid adding a domain-neutral semantic carrier. `TypeStore` and `StaticValueStore` are distinct arenas with separate snapshot identities.

---

## 5.3 Add explicit projection outcomes

Use one typed outcome for P5 member reduction:

```rust
pub enum StaticMemberProjectionOutcome {
    ReducedValue(StaticValueRef),
    ReducedKinded(TypeId),
    Stuck(MemberProjection),
    Invalid(...),
    Unknown(...),
    Blocked(...),
    Dynamic(...),
    Cancelled,
    BudgetExceeded(...),
    InternalFailure(...),
}
```

Exact naming may follow existing outcome conventions.

Required language mapping:

```text
Reduced -> canonical result
Stuck   -> valid symbolic projection
Invalid -> concrete misuse/domain/arity/operation error
```

Internal proof/resource states remain distinct.

Never turn `Stuck` into `Dynamic`.

---

## 5.4 Add explicit compiler-derived conformance evidence

Preferred form:

```rust
pub enum CompilerDerivedConformanceKind {
    Product,
    OrderedProduct,
    UnorderedProduct,
    Sum,
}

pub struct CompilerDerivedConformanceEvidence {
    pub key: GenericConformanceKey,
    pub kind: CompilerDerivedConformanceKind,
    pub classification: StructuralClassification,
}
```

and:

```rust
pub enum GenericConformanceEvidence {
    Exact(...),
    Assumed(...),
    Derived(...),
    CompilerDerived(Arc<CompilerDerivedConformanceEvidence>),
}
```

Alternative naming such as `Structural` is acceptable if the variant is not falsely source-backed.

Requirements:

- no `ImplId`;
- no fake source range;
- no witness callable table;
- exact canonical trait identity;
- exact canonical subject Type for closed proof;
- deterministic fingerprint;
- generic lexical assumptions remain `Assumed`.

---

## 5.5 Structural proof must be a candidate in the existing proof engine

Integrate into:

```rust
evaluate_conformance_goal(...)
```

Suggested order:

```text
1. lexical assumption
2. compiler-derived structural candidate
3. source exact/conditional candidates
4. guarded rules
5. classify success/failure
```

or treat structural proof as a candidate accumulated with 2A/2B.

The important coherence rule is:

> Ordinary source impls of canonical structural traits must be rejected before publication, so a structural candidate never competes with a legitimate source candidate.

That makes structural proof deterministic and prevents artificial coherence conflicts.

---

## 5.6 Seal exact canonical structural trait identities, not names

Add an exact canonical registry, conceptually:

```rust
pub struct StructuralUniverseIds {
    pub product: DeclarationId,
    pub ordered_product: DeclarationId,
    pub unordered_product: DeclarationId,
    pub sum: DeclarationId,

    pub product_kind: DeclarationId,
    pub product_shape: DeclarationId,
    ...
}
```

This may be integrated into `CoreDeclarationIds` if consistent with current repository direction, but a dedicated owner is clearer because not every structural reflection type is a native class anchor.

No logic may use:

```rust
declaration.name == "Product"
```

as semantic authority.

---

## 5.7 Reject source impls of canonical structural traits

In conformance-head admission, after trait resolution and before coherence publication:

```text
if trait_ref.declaration is canonical structural capability:
    diagnostic
    contribution.eligible = false
```

Do not accept the contribution and ignore it later.

User-defined unrelated traits named `Product`, `Sum`, etc. remain ordinary.

---

## 5.8 Public reflection is ordinary `StaticValueStore` data

Construct:

```text
ProductKind
ProductShape
OrderedTail
OrderedShape
Presence
FieldInfo
ProductField
UnorderedTail
UnorderedShape
FieldLookup
UnorderedShapeView
Variant
```

through canonical data/variant declaration metadata and existing:

```text
StaticValueStore::intern_data
StaticValueStore::intern_variant
StaticValueStore::intern_record
StaticValueStore::intern_pack
StaticValueStore::intern_type
StaticValueStore::intern_symbol
```

Do not add reflection-specific `StaticValueData` node variants.

---

# 6. Task graph

```text
                      ┌──────── T1 domain convergence ─────────┐
                      │                                        │
T0 rebaseline ────────┤                                        ▼
                      │                              T2 domain-neutral projection
                      │                                        │
                      └──────── T3 associated syntax/surface ──┤
                                                               ▼
                                                      T4 Universe surface/IDs
                                                               │
                            ┌──────────────────────────────────┤
                            ▼                                  ▼
                    T5 structural evidence             T6 ordered reflection
                            │                                  │
                            └──────────────┬───────────────────┘
                                           │
                               P4 completion barrier S1
                                           │
                                           ▼
                              T7 unordered classification
                                           │
                                           ▼
                              T8 unordered reflection/members
                                           │
                                           ▼
                                   T9 Sum reflection
                                           │
                                           ▼
                           T10 generic/stuck specialization
                                           │
                                           ▼
                             T11 incremental/tooling closure
                                           │
                                           ▼
                                 T12 stabilize/certify
```

T1–T6 can be developed before final P4 completion if they do not consume unfinished unordered semantics.

T7–T8 require the final P4 owner/row/pattern/type-formation contract.

---

# 7. T0 — Rebaseline and freeze the exact write matrix

## Goal

Prove the live repository still matches this plan before edits.

## Required checks

Record:

```text
git revision / remote synchronization
C4 checkpoint status
P4 current completion gate
P10 current integrated revision/status
working tree cleanliness
```

Re-open:

```text
docs/specs/type-members.md
docs/specs/associated-types.md
docs/specs/structural-products.md
docs/specs/structural-sums.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
P4 completion plan/handoff
P10 plan/walkthrough/handoff if present
```

Audit exact current symbols:

```text
TypeMemberDomain
TypeMemberParameterDomainSyntax
TypeMemberResultSyntax
TypeMemberArgument
TypeLevelProjection
AssociatedTypeDeclaration
TraitAssociatedTypeRequirement
AssociatedTypeBindingValue
GenericConformanceEvidence
ConformanceProofContext
StructuralClassification
OrderedProductStructure
SumStructure
UnorderedFieldLookup
StaticValueStore
UniverseKey
CoreDeclarationIds
UNIVERSE_NODES
```

## Deliverable

Add a short T0 implementation note to the P5 walkthrough recording any drift.

## Gate G0

Proceed only if:

- P10 lower structural classifier remains canonical;
- no independent P5 structural implementation has landed;
- the member-domain gap still exists or has an equivalent unresolved form;
- P4 remains the unordered semantic owner.

---

# 8. T1 — Generalize member parameter/result domains

## Primary files

```text
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/static_domain.rs
phalcom-semantic/src/checker/type_member_declaration.rs
phalcom-semantic/src/types/annotation.rs
```

Likely affected tests:

```text
phalcom-semantic/tests/semantic/foundations/static_domains.rs
phalcom-semantic/tests/semantic/foundations/type_members.rs
phalcom-semantic/tests/semantic/incremental/type_members.rs
phalcom-semantic/tests/semantic/integration/type_member_closure.rs
```

## Changes

### T1.1 Expand/converge `TypeMemberDomain`

Target:

```rust
pub enum TypeMemberDomain {
    Kind(KindId),
    ValueType(TypeId),
}
```

Make conversion with `StaticValueDomain` total.

Update:

```text
is_kind / is_value_type style helpers
fingerprinting
Debug/equality/hash
signature construction
```

### T1.2 Fix `ConstType` parameter lowering

Current behavior:

```text
ConstType(T)
→ static_domain_kind_of_form(T)
→ Kind(...)
```

Required:

```text
ConstType(T)
→ resolve T as proper ordinary Type
→ ValueType(T)
```

The term “ConstType” in the AST is historical presentation; its semantic meaning is an ordinary static value-domain.

### T1.3 Fix `TypeMemberResultSyntax::Type`

Current behavior again converts the form to a static-domain kind.

Required:

```text
TypeMemberResultSyntax::Type(ProductShape)
    -> ValueType(ProductShape)
```

while:

```text
TypeMemberResultSyntax::Kind(Type)
    -> Kind(TYPE)
```

### T1.4 Generalize argument-domain validation result

Current error carries:

```text
expected: KindId
actual: KindId
```

Replace with domain-aware information, e.g.:

```rust
ArgumentDomainMismatch {
    index: usize,
    expected: StaticValueDomain,
    actual: StaticArgumentDomain,
}
```

or equivalent.

### T1.5 Remove irrefutable `TypeMemberDomain::Kind` destructures

Audit all production matches in:

```text
types/annotation.rs
checker/type_member_declaration.rs
types/projection.rs
checker/inference.rs
metadata/export.rs
```

Kind-only operations may explicitly reject `ValueType` where they are semantically inapplicable.

Do not silently reinterpret a value result as `KindId::TYPE`.

## Required tests

1. existing kinded type members remain unchanged;
2. `type Lookup<K: FieldKey>: FieldLookup` signature publishes:
   - parameter `ValueType(FieldKey)`;
   - result `ValueType(FieldLookup)`;
3. a parameter `F: Type -> Type` remains `Kind(Arrow(...))`;
4. a result `: Type` written as the kind syntax remains kinded Type;
5. ordinary proper type domains are no longer rejected by `StaticValueDomain -> TypeMemberDomain`.

## Gate G1

No production `TypeMemberDomain` consumer assumes the enum has only `Kind`.

---

# 9. T2 — Domain-neutral member arguments and projections

## Problem

`Lookup<K: FieldKey>` requires an ordinary static value argument. Existing `TypeMemberArgument` can only represent:

```text
TypeId
ConstInt
RecordRowId
```

and existing `TypeLevelProjection` is stored inside TypeStore.

## Primary files

```text
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/gadt_proof.rs
```

Potential new module:

```text
phalcom-semantic/src/member_projection.rs
```

## T2.1 Introduce domain-neutral argument identity

Preferred conceptual model:

```rust
pub enum MemberProjectionArgument {
    Kinded(TypeMemberArgument),
    Value(StaticValueRef),
}
```

If the existing static semantic layer has a more durable store-qualified value reference appropriate to use, prefer it.

Do **not** put value-store-local raw indexes into durable TypeStore metadata.

## T2.2 Introduce domain-neutral projection identity

Conceptual:

```rust
pub struct MemberProjection {
    pub subject: ProjectionSubject,
    pub member: MemberDeclarationIdentity,
    pub arguments: Box<[MemberProjectionArgument]>,
    pub evidence: Option<GenericConformanceKey>,
    pub result_domain: StaticValueDomain,
}
```

For trait-associated members, include exact:

```text
TraitRef
AssociatedTypeRequirementId
subject
```

rather than only a name.

## T2.3 Preserve TypeStore `TypeLevelProjection` for kinded results

When:

```text
result_domain == Kind(_)
```

lower/reuse the existing:

```text
TypeLevelProjection
TypeData::TypeMemberProjection
```

This avoids a destructive rewrite of established C1–C3 type substitution/inference/metadata.

When:

```text
result_domain == ValueType(_)
```

do not create a TypeStore node.

## T2.4 Add projection outcome type

Required states:

```text
ReducedKinded(TypeId)
ReducedValue(StaticValueRef)
Stuck(MemberProjection)
Invalid
Unknown
Blocked
Dynamic
Cancelled
BudgetExceeded
InternalFailure
```

Reuse existing reason types where possible.

## T2.5 Generalize application syntax lowering

When parsing/lowering:

```phalcom
UnorderedShape::Lookup<#x>
```

or equivalent static member syntax:

- resolve `#x` through the ordinary static-value semantics;
- verify it is a `FieldKey` value;
- store the canonical static value reference in the projection argument;
- do not coerce it to Symbol or Type.

If current grammar accepts only limited member-argument syntax, extend only the minimum general static-value argument surface needed by normative P5. Do not implement arbitrary static expression residualization here; that is P7 territory.

## T2.6 Keep symbolic projections canonical

If subject/evidence is abstract but operation is valid:

```text
return Stuck(MemberProjection)
```

not `Dynamic`.

## Required tests

- closed FieldKey argument validates;
- String/Int/Bool/Symbol remain distinct;
- wrong ordinary value type is rejected;
- kinded existing `At<I>` remains normalized as before;
- stuck `Product::Shape` / `Sum::Variants` projection can be represented without TypeId fabrication;
- no StaticValueRef is inserted into `TypeData::TypeMemberProjection`.

## Gate G2

The semantic projection layer is domain-neutral while TypeStore remains the owner only for kinded projection results.

---

# 10. T3 — Associated declaration result-domain syntax and semantic surface

## Primary files

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/source_index/*
phalcom-lsp/* only where current AST exhaustiveness requires it
```

## T3.1 Extend AST

Current:

```rust
pub struct AssociatedTypeDeclaration {
    pub name: String,
    pub name_range: SourceRange,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}
```

Target:

```rust
pub struct AssociatedTypeDeclaration {
    pub name: String,
    pub name_range: SourceRange,
    pub result_domain: Option<StaticDomainSyntax>, // or equivalent shared syntax
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}
```

Use the same static-domain grammar already used by general type-member declarations where possible.

## T3.2 Extend parser

Accept:

```phalcom
type Item
type Shape: ProductShape
type Variants: Variants
```

Preserve:

```text
unannotated associated declaration defaults to Kind(Type)
```

Continue to reject associated defaults unless separately specified:

```phalcom
type Item = Int
```

inside a trait remains invalid if defaults are out of scope.

Do not introduce GAT/member-local parameters in this task.

## T3.3 Generalize `TraitAssociatedTypeRequirement`

Replace:

```rust
kind: KindId
```

with:

```rust
domain: StaticValueDomain
```

or `TypeMemberDomain` if the project chooses that as the canonical representation after T1.

## T3.4 Fix `build_trait_surface`

Current hard-coded:

```rust
kind: KindId::TYPE
```

becomes:

```text
resolve explicit domain if present
otherwise Kind(Type)
```

Use canonical declaration/type resolution, not names.

## T3.5 Preserve associated declaration identity

Identity remains:

```rust
AssociatedTypeRequirementId {
    owner: DeclarationId,
    index: u32,
}
```

Do not change identity to the domain or name.

## T3.6 Source-index integration

Record references from result-domain syntax to the canonical declaration/type.

Keep the associated requirement target identity unchanged.

## Required tests

Parser:

```text
type Item
type Shape: ProductShape
malformed `type Shape:`
trait associated default rejection remains
```

Semantic:

```text
unannotated -> Kind(Type)
ProductShape annotation -> ValueType(ProductShape)
declaration order does not change identity semantics
duplicate associated names remain diagnosed
```

## Gate G3

The canonical trait surface can truthfully represent every P5 associated member domain.

---

# 11. T4 — Add canonical Universe structural reflection surface and identities

## Source placement

Preferred package:

```text
phalcom-core/core/universe/src/reflection/structural.ph
```

or a small structural subpackage if the source becomes too large.

T0 may adjust exact file placement to current Universe conventions, but all declarations should live under the canonical Universe reflection namespace.

## Primary infrastructure

```text
phalcom-core/core/universe/src/reflection/package.ph
phalcom-modules/src/builtin.rs
phalcom-native-meta/src/universe.rs      (only if these declarations require UniverseKey/native anchor identity)
phalcom-semantic/src/core_surface/identity.rs
```

## T4.1 Add public declarations

Add the declarations from §3.

Where an alias can be source-defined, keep it source-defined:

```phalcom
type Variants = PositionalPack<Variant>
type VariantNames = PositionalPack<Symbol>
type FixedFields = ...
```

Where semantics require compiler normalization, use the existing `intrinsic` declaration vocabulary.

## T4.2 Choose canonical identity strategy

Not every reflection data type necessarily needs a `UniverseKey` entry.

Use `UniverseKey` for declarations that participate in native-surface/native-runtime anchoring under current repository rules.

For semantic-only canonical source declarations, exact `DeclarationId` from the canonical Universe module path is sufficient.

Create one structural ID owner, for example:

```rust
pub struct StructuralUniverseIds {
    pub product: DeclarationId,
    pub ordered_product: DeclarationId,
    pub unordered_product: DeclarationId,
    pub sum: DeclarationId,
    pub product_kind: DeclarationId,
    pub product_shape: DeclarationId,
    pub ordered_shape: DeclarationId,
    pub unordered_shape: DeclarationId,
    pub field_lookup: DeclarationId,
    pub variant: DeclarationId,
    ...
}
```

Construct identities using exact Universe module paths.

Do not force all of these into `CoreDeclarationIds` if that would conflate native primitive roots with semantic source declarations.

## T4.3 Register module topology

Update:

```text
UNIVERSE_NODES
```

for physical module discovery.

Update authored:

```text
reflection/package.ph
```

for exposure.

Preserve:

```text
physical child topology != public visibility
```

## T4.4 Bootstrap/canonical baseline

Ensure the canonical Universe build can parse/check the new declarations with the T1–T3 domain semantics.

## Required tests

- canonical declarations resolve from Universe source;
- exact module/DeclarationId identity is stable;
- a user declaration with the same leaf name has a different identity;
- import/exposure works through authored package edges;
- minimal canonical Universe bootstrap remains green.

## Gate G4

P5 compiler logic can receive exact canonical structural declaration IDs with zero spelling tests.

---

# 12. T5 — Sealed compiler-derived structural conformance

## Primary files

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/checker/context.rs
```

Consumers to audit:

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/iteration.rs
phalcom-semantic/src/checker/algebraic_realization.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
```

## T5.1 Add structural evidence provenance

Implement the model from §5.4.

No source ImplId.

## T5.2 Extend proof context with lower structural authority

Preferred:

```rust
pub struct StructuralConformanceContext<'a> {
    pub classifier: StructuralClassifier<'a>,
    pub ids: &'a StructuralUniverseIds,
}
```

or equivalent references to:

```text
DataSemanticTable
EnumSemanticTable
StructuralUniverseIds
```

Attach optionally to `ConformanceProofContext`.

`db/query.rs` and direct proof callers must provide a coherent snapshot view.

Avoid creating another independently cached classification table.

## T5.3 Implement exact structural trait mapping

Conceptually:

```text
canonical Product:
    OrderedProduct(_)   -> proven Product
    UnorderedProduct(_) -> proven Product
    otherwise           -> no candidate

canonical OrderedProduct:
    OrderedProduct(_) -> proven
    otherwise         -> no candidate

canonical UnorderedProduct:
    UnorderedProduct(_) -> proven
    otherwise           -> no candidate

canonical Sum:
    Sum { .. } -> proven
    otherwise  -> no candidate
```

Non-proper kinds are not structural capability subjects.

## T5.4 Integrate into `evaluate_conformance_goal`

Lexical assumption remains highest-priority evidence for abstract generic subjects.

For concrete canonical subjects, add compiler-derived candidate proof.

Use the existing memo/budget/cancellation authority.

## T5.5 Seal source impl admission

In source conformance head construction/admission:

```text
if requested trait == one of canonical structural capability IDs:
    reject explicit impl
```

Diagnostic must be deterministic.

Suggested code/category:

```text
ConformanceCompilerDerivedTrait
```

Message:

```text
`Product` is a compiler-derived structural capability and cannot be implemented explicitly
```

Do not reject user traits with the same spelling.

## T5.6 Audit evidence consumers

Classify each match on `GenericConformanceEvidence`.

Rules:

```text
runtime witness required?
    Structural evidence has none.

associated static projection?
    Structural evidence is valid.

callable dispatch?
    Structural trait has no behavioral requirements; do not synthesize callable.

source dependency?
    Depend on structural semantic products, not ConformanceDispatch(ImplId).

fingerprint?
    hash subject + canonical trait + structural classification/evidence kind.
```

## T5.7 Fingerprinting

Add stable fingerprint coverage for `CompilerDerived`.

Do not hash:

```text
TypeId raw integer alone
arena allocation order
source range
```

Use existing semantic identity/fingerprint conventions.

## Required tests

Capability matrix:

| Subject | Product | OrderedProduct | UnorderedProduct | Sum |
|---|---:|---:|---:|---:|
| `Int` | no | no | no | no |
| `Unit` | yes | yes | no | no |
| Tuple | yes | yes | no | no |
| Pack | yes | yes | no | no |
| tuple-shaped nominal data | yes | yes | no | no |
| Record | yes after S1 | no | yes after S1 | no |
| record-shaped nominal data | yes after S1 | no | yes after S1 | no |
| enumerated data | no | no | no | yes |

Negative:

```text
explicit impl canonical Product -> diagnostic
explicit impl canonical Sum     -> diagnostic
user trait named Product        -> ordinary behavior
```

Generic:

```text
T |= Product -> Assumed evidence inside generic body
specialize T to Tuple -> exact compiler-derived proof in closed query
```

## Gate G5

All structural capabilities are proved through `GenericConformanceProof` and no structural proof fabricates source conformance identity.

---

# 13. T6 — Ordered Product associated reflection

## Primary files

```text
phalcom-semantic/src/structural.rs
new/existing structural reflection projector module
phalcom-semantic/src/static_value/store.rs (reuse; avoid structural changes)
trait-associated projection normalization path
```

## T6.1 Add `StructuralReflectionProjector`

Recommended owner:

```text
phalcom-semantic/src/structural_reflection.rs
```

Conceptually:

```rust
pub struct StructuralReflectionProjector<'a> {
    classifier: StructuralClassifier<'a>,
    ids: &'a StructuralUniverseIds,
    types: &'a mut TypeStore,
    values: &'a mut StaticValueStore,
    data: &'a DataSemanticTable,
    enums: &'a EnumSemanticTable,
    ...
}
```

It should expose operations by semantic member identity, not names.

## T6.2 Reduce `Product::Kind`

Ordered subject:

```text
ProductKind::Ordered
```

via ordinary static data/variant construction.

## T6.3 Reduce ordered `Product::Shape`

Algorithm:

```text
descriptor = classifier.ordered_product(subject)

if descriptor is concrete:
    entries = project fixed positional/labeled Type values into a CompletePack<Type>
    positionalTail = project remainder
    labeledTail = project remainder
    orderedShape = construct OrderedShape
    result = ProductShape::Ordered(orderedShape)

if descriptor carries unresolved symbolic lanes:
    return Stuck(member projection)
```

Do not pretend symbolic `TupleFromPacks` has zero entries.

## T6.4 Reduce `OrderedProduct` members

Produce:

```text
OrderedShape
Entries
PositionalEntries
LabeledEntries
```

from the same descriptor.

There must be one lane projection algorithm shared by all four members.

## T6.5 Ordered tail mapping

Map canonical remainder:

```text
AnonymousTupleRemainder::None
    -> Closed

Typed(T)
    -> Typed(Type(T))

Unconstrained
    -> Unconstrained

captured/correlated pack form
    -> Captured(pack value/term) when representable
    -> otherwise valid Stuck projection
```

Do not expand open tails as finite entries.

## T6.6 Nominal data

Use:

```text
materialize_data_product_shape
```

already called by the classifier.

Contained component Types are static `Type` values, not recursively reflected shapes.

## Required tests

- Unit zero product;
- positional Tuple;
- labeled Tuple preserving Symbol lane;
- Pack;
- open typed positional tail;
- open typed labeled tail;
- unconstrained tails;
- tuple-shaped generic data specialization;
- equal reflected shape but different nominal Type identities;
- recursive product remains shallow;
- symbolic TupleFromPacks / PackFromPacks returns `Stuck` until specialization.

## Gate G6

Every ordered P10 structural form has correct P5 public capability/associated projection semantics without runtime layout inspection.

---

# 14. Synchronization barrier S1 — require P4 completion for unordered work

Before T7:

1. confirm P4 T7.0E gate is accepted;
2. confirm final `FieldKey`/row owner matrix;
3. confirm computed-key aliases/static definitions publish canonical rows;
4. confirm Record patterns consume resolved `FieldKey`;
5. confirm `lookup_unordered_field` remains canonical;
6. confirm `RecordRowData`/tail identities after P4 closure;
7. confirm P4's metadata boundary remains P8-owned;
8. merge/rebase P5 branch as required and rerun G1–G6 focused tests.

If P4 changed a structural API named in this plan, adapt P5 to the final P4 owner rather than preserving a compatibility fork.

---

# 15. T7 — Extend lower structural classification to unordered products

## Primary file

```text
phalcom-semantic/src/structural.rs
```

Potential supporting files only if P4 lacks a clean decomposition helper:

```text
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/relation.rs
```

Do not create a second row model.

## T7.1 Extend classification

Conceptual:

```rust
pub enum UnorderedProductForm {
    Record,
    Data { owner: DeclarationId },
}

pub enum StructuralClassification {
    Unsupported,
    OrderedProduct(OrderedProductForm),
    UnorderedProduct(UnorderedProductForm),
    Sum { owner: DeclarationId },
}
```

Classification:

```text
TypeData::Record(...)
    -> UnorderedProduct(Record)

nominal/applied DataInfo.shape == DataShape::Record
    -> UnorderedProduct(Data { owner })
```

## T7.2 Add shallow unordered descriptor

Conceptual:

```rust
pub struct UnorderedProductEntry {
    pub component: Option<DataComponentId>,
    pub key: FieldKey,
    pub ty: Option<TypeId>,
    pub presence: StructuralMemberPresence,
}

pub struct UnorderedProductStructure {
    pub ty: TypeId,
    pub origin: UnorderedProductOrigin,
    pub fields: Box<[UnorderedProductEntry]>,
    pub tail: RecordRowTail,
    pub state: DataProductShapeState,
}
```

The exact tail carrier may use a safer reflection/decomposition type instead of cloning raw row state.

## T7.3 Canonical order

Consume P4's canonical row ordering.

Do not preserve source presentation order as public `UnorderedShape.fields` identity.

## T7.4 Nominal record data

Use `DataInfo` and existing materialization.

For nominal record components, preserve component identity internally but expose only normative `ProductField` / `FieldInfo`.

## Required tests

- anonymous closed Record;
- record-shaped nominal data;
- all four FieldKey kinds;
- canonical cross-kind order;
- optional presence;
- typed tail;
- unconstrained tail;
- captured/projection tail;
- generic specialization;
- same source name does not imply structural classification.

## Gate G7

P5 unordered descriptor is a pure projection of final P4 canonical semantics.

---

# 16. T8 — Unordered Product reflection and domain-native `UnorderedShape` members

## T8.1 Reduce `Product::Kind`

Unordered subject:

```text
ProductKind::Unordered
```

## T8.2 Reduce `Product::Shape`

Construct:

```text
ProductShape::Unordered(UnorderedShape(...))
```

from `UnorderedProductStructure`.

## T8.3 Build `FixedFields`

For each canonical fixed field:

```text
key      -> ordinary FieldKey static value
type     -> Type static value
presence -> Presence static value
value    -> FieldInfo(type, presence)
```

Build canonical static `Record` keyed by exact FieldKey.

No stringification/coercion.

## T8.4 Map tails

```text
Closed
    -> UnorderedTail::Closed

Anonymous Typed { key_type, value_type }
    -> UnorderedTail::Typed(Type(key_type), Type(value_type))

Anonymous Unconstrained { key_type }
    -> UnorderedTail::Unconstrained(Type(key_type))

Parameter / Projection / captured correlated tail
    -> Captured(rest) when exact public shape can be formed
    -> otherwise Stuck symbolic projection
```

Do not label an unresolved captured row as `Unknown` merely because reduction is not closed; `Stuck` is the projection state.

## T8.5 `UnorderedShape::Lookup<K>`

Inputs:

```text
subject = reduced or symbolically known UnorderedShape
K       = ordinary static FieldKey value
```

For a shape backed by canonical row identity, call:

```rust
lookup_unordered_field(...)
```

Map:

```text
Required(T) -> FieldLookup::Required(Type(T))
Optional(T) -> FieldLookup::Optional(Type(T))
Absent      -> FieldLookup::Absent
Unknown(*)  -> FieldLookup::Unknown
```

Do not expose internal `RelationOutcome` details publicly.

Internal cancellation/budget remains an evaluator/projection outcome, not public `FieldLookup::Unknown`.

## T8.6 `UnorderedShape::View`

One-step decomposition only:

```text
no fields:
    Empty(tail)

fields:
    Field(
        ProductField(first),
        restShape(offset + 1)
    )
```

Avoid rebuilding N-field Records on each step.

If an internal slice/view carrier is required, keep it semantic/internal and materialize the public `UnorderedShape` value only as needed.

## T8.7 `Tail`

Return the mapped canonical tail.

## Required tests

Lookup matrix:

```text
fixed required -> Required(T)
fixed optional -> Optional(T)
closed missing -> Absent
typed tail accepts key -> Optional(valueType)
typed tail rejects key domain -> Absent
unconstrained accepted key -> Unknown
captured unresolved -> symbolic/internal uncertain as specified
```

View:

- canonical FieldKey order;
- one-step rest;
- tail preserved after last fixed field;
- no Symbol/String conflation.

## Gate G8

All unordered public reflection delegates to P4 row/lookup semantics and preserves exact FieldKey identity.

---

# 17. T9 — Sum / Variant public reflection

## Primary inputs

```text
StructuralClassifier::sum
SumStructure
SpecializedVariantStructure
VariantReflection
ExactCaseTypeReflection
EnumInfo / VariantInfo
```

## T9.1 Reduce `Sum::Variants`

Algorithm:

```text
sum = classifier.sum(subject)

for exact variant in declaration order:
    name = intern Symbol from canonical variant family/name spelling
    shape = project immediate payload ProductShape
    value = construct public Variant(name, shape)

return intern_pack(positionals = variants, labeled = [])
```

## T9.2 Nullary case

Nullary Variant shape is an empty **ordered** ProductShape:

```text
entries = Pack()
positionalTail = Closed
labeledTail = Closed
```

Do not use `Unit` or `None` as `Variant.shape`.

## T9.3 Tuple payload

Use ordered payload reflection.

## T9.4 Record payload

Use T7/T8 unordered payload reflection.

## T9.5 Repeated names

Do not deduplicate.

Public:

```text
Variant(#Number, shape1)
Variant(#Number, shape2)
```

remain two entries.

Internally retain exact Variant identities for construction/specialization.

## T9.6 GADT boundary

Use already-specialized immediate field Types.

Do not export:

```text
branch equalities
rigid proof variables
proof environments
matcher refinements
```

## Required tests

- simple enum;
- generic `Result<T,E>`;
- specialized `Result<Int,String>`;
- recursive enum;
- repeated-name family;
- nullary case;
- tuple payload;
- Record payload;
- GADT immediate specialized payload types;
- declaration order stable.

## Gate G9

`Sum::Variants` is an ordinary positional Pack of public `Variant` values and never collapses exact alternatives.

---

# 18. T10 — Generic/stuck projections and specialization closure

This task prevents P5 from being “closed examples only”.

## T10.1 Generic capability assumption

Example:

```phalcom
fn f<T>()
where T |= Product
{
    // T::Shape is valid
}
```

Inside the generic body:

```text
conformance evidence = Assumed
projection = valid Stuck(MemberProjection)
```

No structural classifier may require a concrete `TypeData` form at this point.

## T10.2 Later specialization

When `T` becomes:

```text
Tuple<Int,String>
```

the same semantic member identity should reduce to an ordinary `ProductShape` StaticValue.

## T10.3 Symbolic ordered lanes

`TupleFromPacks` / `PackFromPacks` with unresolved pack terms remain stuck where a concrete Pack value cannot be formed.

Do not report empty lanes.

## T10.4 Symbolic unordered tails

Generic Record row parameters/projections remain valid symbolic shape/tail projections.

Do not map them to `Dynamic`.

## T10.5 Sum generics

`Result<T,E>::Variants` may contain symbolic Type references where the Type terms themselves are canonical generic parameters.

If the public static value can safely contain `StaticValueData::Type(parameter_form)`, use it. If some payload shape cannot be reduced because of an unresolved structural lane/tail, keep the outer member projection stuck rather than producing a partially false value.

## Required tests

- assumed Product;
- assumed Sum;
- specialization from stuck to reduced;
- substitution preserves member declaration identity;
- cold and specialized paths agree;
- no `Dynamic` fallback for valid abstract structural projections.

## Gate G10

Every normative structural member has a well-defined Reduced/Stuck distinction.

---

# 19. T11 — Incrementality, fingerprints, source index, and compiler integration

## 19.1 Generic conformance query

Keep:

```text
QueryKey::GenericConformance(GenericConformanceKey)
```

as the proof query.

Add dependencies/fingerprint inputs for structural proof on the exact lower semantic products.

Do not create a redundant `QueryKey::StructuralConformance` unless the existing query cannot express currentness.

## 19.2 Structural reflection projection currentness

Prefer ordinary dependency recording on:

```text
DataDeclaration(owner)
EnumDeclaration(owner)
relevant type/member declarations
P4 row/type owner facts
canonical structural trait/member surfaces
```

Do not key semantic identity by source range.

## 19.3 Invalidation matrix

Must invalidate:

```text
product component type change
Record key/presence/tail change
Variant add/remove/reorder
Variant payload type change
canonical structural trait/member declaration change
```

Should not invalidate structural shape:

```text
unrelated method body edit
behavior-only impl edit
unrelated module declaration
```

## 19.4 Evidence fingerprint

`CompilerDerived` evidence fingerprint must be deterministic and cold/warm stable.

## 19.5 Source index

Ensure:

```text
Product::Shape
Product::Kind
OrderedProduct::Entries
Sum::Variants
```

target the canonical trait-associated requirement IDs.

Result-domain type references in trait declarations should target their canonical declarations.

## 19.6 Core lowering/runtime consumers

Audit new `GenericConformanceEvidence::CompilerDerived` handling.

Structural traits have no runtime behavioral witnesses. Therefore:

```text
compiler lowering that asks for callable evidence
    must never try to extract an ImplId/callable from structural evidence.

pure static associated projection
    may consume structural evidence.
```

If runtime generic evidence layout includes only behavioral requirements, structural traits with zero behavior should allocate no runtime witness slots.

## Required tests

Incremental:

- product component edit invalidates shape;
- method body edit does not;
- enum reorder invalidates Variants order;
- FieldKey edit invalidates UnorderedShape;
- cold/warm proof fingerprints agree;
- user same-name traits do not invalidate canonical structural proof.

Cross-module semantic test:

- generic module consumes `T |= Product`;
- caller specializes with a structural Type;
- no fake runtime witness is required.

## Gate G11

P5 structural semantics are exact under incremental reuse and do not leak into runtime witness machinery.

---

# 20. T12 — Stabilize, certify, and close lifecycle

## 20.1 Static source audit before broad tests

Search production code for forbidden patterns:

```text
TypeMemberDomain::Kind(...) irrefutable destructures
AssociatedTypeRequirement kind: KindId hard-coding
associated declarations always KindId::TYPE
AssociatedTypeBindingValue assumed always Type
GenericConformanceEvidence matches missing CompilerDerived
trait name == "Product"/"Sum"
fake ImplId construction
reflection-specific StaticValueData variants
runtime ProductShape/ClassId used for semantic classification
Record reflection sorting by source order
Variant reflection deduplication by Symbol
Dynamic fallback for valid stuck structural projection
FieldKey stringification
Tuple/Pack label widened to FieldKey
```

Every hit must be classified as valid or fixed.

## 20.2 Focused verification order

Run in this order.

### Gate A — AST/domain foundation

```text
phalcom-ast trait syntax tests
static_domains
type_members
type_member_closure
associated-type capability tests
```

### Gate B — structural conformance

```text
generic_conformance
derived_conformance
conditional_conformance
new structural_capabilities selectors
```

### Gate C — ordered reflection

```text
structural_classification
ordered public reflection selectors
generic/stuck ordered selectors
```

### Gate D — unordered reflection

Only after S1:

```text
record_rows_field_keys
P4 acceptance selectors
unordered public reflection
Lookup/View/Tail selectors
```

### Gate E — Sum reflection

```text
existing adts/structural_reflection
new public Sum/Variant selectors
```

### Gate F — incremental

```text
incremental type members
incremental derived conformance
new incremental structural reflection
```

### Gate G — affected crate checks

At minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-core \
  -p phalcom-lsp
```

Use only crates actually touched if live workspace policy calls for narrower checks.

## 20.3 Broad certification

Run broader C4-required semantic/core suites only after focused gates pass.

Classify unrelated failures under the repository's A/B/C/D lifecycle convention. Do not fix unrelated baselines inside P5.

## 20.4 Lifecycle updates

Create/update:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P5/
    LANG007.C4.P5-implementation-plan.md
    LANG007.C4.P5-walkthrough.md
    LANG007.C4.P5-handoff.md
```

Update checkpoint only with evidence actually obtained.

Do not mark P5 complete while P4 remains incomplete.

---

# 21. Stop-and-consult conditions

## S-P5-1 — P4 row authority is not final

If unordered reflection requires guessing final computed-key/tail/owner semantics, stop T7/T8.

## S-P5-2 — ordinary static member domain cannot be represented without fake TypeId

Do not encode `ProductShape`, `FieldLookup`, `Variant`, or Pack values as Type terms.

Fix T1/T2 first.

## S-P5-3 — value argument would require StaticValueRef inside TypeStore type identity

Do not couple TypeStore projection nodes to StaticValueStore arena-local identities without an explicit architectural review. Use the domain-neutral projection layer.

## S-P5-4 — compiler-derived proof would require fake ImplId

Add explicit evidence provenance instead.

## S-P5-5 — canonical trait recognition requires spelling

Add/fix exact Universe declaration identity first.

## S-P5-6 — structural capability competes with a source impl

Canonical structural traits must be sealed at source-conformance admission. Do not resolve the conflict by precedence.

## S-P5-7 — reflection requires runtime layout inspection

Semantic direction is wrong. Use canonical semantic products.

## S-P5-8 — public Variant needs GADT proof metadata

Keep proof data matcher-internal.

## S-P5-9 — reflection recursively expands contained Types

Reflection must be shallow.

## S-P5-10 — P6 control features are needed to implement projector internals

Use direct semantic construction. Do not pull general static `for`/`match` into P5.

## S-P5-11 — P7 recursive residualization is required for ordinary P5 reduction

P5 shapes should be shallow. Preserve a stuck projection rather than implementing general recursion.

## S-P5-12 — durable cross-module StaticValue wire format becomes required

That is P8. Stop and define the boundary rather than serializing arena-local IDs.

---

# 22. Detailed file/symbol disposition

## Preserve as canonical lower authority

```text
phalcom-semantic/src/structural.rs
    StructuralClassifier
    OrderedProductForm
    OrderedProductStructure
    SumStructure

phalcom-semantic/src/data_semantics.rs
    DataInfo
    DataShape
    materialize_data_product_shape

phalcom-semantic/src/enum_semantics.rs
    EnumInfo
    VariantInfo
    VariantFamilyId
    VariantId

phalcom-semantic/src/reflection.rs
    VariantReflection
    ExactCaseTypeReflection

phalcom-semantic/src/field_key.rs
    FieldKey
    FieldKey::canonical_cmp

phalcom-semantic/src/types/relation.rs
    UnorderedFieldLookup
    lookup_unordered_field

phalcom-semantic/src/static_value/store.rs
    intern_data
    intern_variant
    intern_record
    intern_pack
```

## Generalize

```text
phalcom-semantic/src/types/type_member.rs
    TypeMemberDomain
    TypeMemberArgument / projection abstraction
    TypeMemberApplicationError

phalcom-semantic/src/static_domain.rs
    conversion with member domains

phalcom-semantic/src/checker/type_member_declaration.rs
    ConstType parameter lowering
    Type result-domain lowering

phalcom-ast/src/ast.rs
    AssociatedTypeDeclaration result domain

phalcom-ast/src/parser.rs
    parse_associated_type_declaration

phalcom-semantic/src/traits.rs
    TraitAssociatedTypeRequirement
    build_trait_surface

phalcom-semantic/src/impls.rs
    associated result carrier as needed
    canonical structural trait source-impl rejection

phalcom-semantic/src/generic_evidence.rs
    CompilerDerived evidence
    proof context structural authority
    evaluate_conformance_goal

phalcom-semantic/src/db/fingerprint.rs
    structural evidence fingerprint
```

## Extend after P4 barrier

```text
phalcom-semantic/src/structural.rs
    UnorderedProductForm
    UnorderedProductStructure
```

## Add likely new semantic module

```text
phalcom-semantic/src/structural_reflection.rs
```

## Add Universe source

```text
phalcom-core/core/universe/src/reflection/structural.ph
```

or final T0-chosen equivalent.

## Register canonical topology/identity

```text
phalcom-core/core/universe/src/reflection/package.ph
phalcom-modules/src/builtin.rs
phalcom-semantic/src/core_surface/identity.rs
phalcom-native-meta/src/universe.rs only if native anchor registration is truly required
```

## Audit new evidence consumers

```text
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/iteration.rs
phalcom-semantic/src/checker/algebraic_realization.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
```

---

# 23. Test suite plan

## 23.1 New semantic foundation suites

Recommended:

```text
phalcom-semantic/tests/semantic/foundations/
    structural_capabilities.rs
    structural_member_domains.rs
    structural_reflection_values.rs
```

Keep existing P10 lower tests intact.

## 23.2 ADT public reflection

Recommended:

```text
phalcom-semantic/tests/semantic/adts/
    product_reflection_public.rs
    sum_reflection_public.rs
```

## 23.3 Generic capability tests

Extend:

```text
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
```

for:

```text
Assumed Product
closed compiler-derived Product
closed Sum
source impl rejection
same-name user trait isolation
```

## 23.4 Incremental

Recommended:

```text
phalcom-semantic/tests/semantic/incremental/
    structural_reflection.rs
```

## 23.5 Core/runtime tests

Only add runtime tests where P5 necessarily crosses the runtime boundary.

P5 should mostly be semantic/static.

A useful negative runtime test is that generic structural-only constraints do not require a runtime witness slot/callable.

---

# 24. Acceptance matrix

| ID | Requirement | Evidence |
|---|---|---|
| P5-A01 | `TypeMemberDomain` supports ordinary static value types | focused domain tests |
| P5-A02 | `Lookup<K: FieldKey>` stores ValueType parameter/result domains | signature test |
| P5-A03 | associated declaration `type X: Domain` parses | AST parser tests |
| P5-A04 | trait surface retains explicit result domain | semantic trait test |
| P5-A05 | canonical Product/OrderedProduct/UnorderedProduct/Sum source declarations resolve | Universe test |
| P5-A06 | user same-name traits are ordinary | negative identity test |
| P5-A07 | canonical structural trait explicit impl is rejected | diagnostic test |
| P5-A08 | structural proof uses `GenericConformanceProof` | proof test |
| P5-A09 | structural proof has no fake `ImplId` | evidence assertion |
| P5-A10 | ordered Product proof matrix passes | capability tests |
| P5-A11 | unordered Product proof matrix passes after P4 | capability tests |
| P5-A12 | Sum proof matrix passes | capability tests |
| P5-A13 | Product::Kind reduced values correct | reflection tests |
| P5-A14 | ordered Shape preserves lanes/tails | reflection tests |
| P5-A15 | symbolic pack lanes remain Stuck | generic test |
| P5-A16 | unordered fields preserve exact FieldKey and Presence | reflection tests |
| P5-A17 | `Lookup` delegates to canonical row semantics | lookup matrix |
| P5-A18 | `View` is canonical one-step decomposition | view tests |
| P5-A19 | Sum Variants preserve declaration order | sum tests |
| P5-A20 | repeated Variant names are not collapsed | family test |
| P5-A21 | recursive reflection is shallow | recursive tests |
| P5-A22 | GADT proof metadata is absent from public Variant | structural assertion |
| P5-A23 | valid abstract structural projection is Stuck, not Dynamic | generic tests |
| P5-A24 | reduced public reflection values use ordinary StaticValue nodes | store assertions |
| P5-A25 | runtime layout is not structural source | source audit |
| P5-A26 | cold/warm incremental results agree | incremental tests |
| P5-A27 | unrelated behavior edit does not invalidate shape | incremental test |
| P5-A28 | no P3 Symbol-label regression | ordered-label regression |
| P5-A29 | P4 focused gates remain green | inherited regression |
| P5-A30 | lifecycle records match actual evidence | closure audit |

P5 cannot be `COMPLETE / IMPLEMENTED` until all applicable rows pass and P4 is complete.

---

# 25. Performance/resource contract

## Classification

```text
ordered classification          O(1) / immediate Type form + declaration lookup
sum classification              O(1) declaration lookup
unordered classification        O(1) + canonical row/declaration lookup
```

## Reflection

```text
ordered fixed entries           O(n)
unordered fixed fields          O(n) if P4 already canonicalized order
                                O(n log n) at most if one sort is unavoidable
Sum::Variants                   O(total immediate variant payload fields)
recursive contained Type        O(1) reference, not recursive expansion
```

## View

`UnorderedShape::View` should not reconstruct the complete remainder on every step.

Aim for:

```text
one-step view                   O(1) or amortized O(1) after shape materialization
full iteration                  O(n)
```

## Generic proof

Structural proof should not scan all source impls once canonical trait identity is recognized.

## Static values

Interning may hash-cons structural values through existing stores. Do not make reflection equality recursively inspect reflected Type shapes.

---

# 26. Incremental dependency model

Recommended dependencies:

```text
Product/OrderedProduct proof:
    exact subject Type identity
    relevant DataDeclaration when nominal
    canonical structural trait identity

UnorderedProduct proof:
    exact subject
    Record row/type structure
    relevant DataDeclaration
    canonical structural trait identity

Sum proof:
    EnumDeclaration owner
    canonical Sum trait identity

ProductShape:
    same lower structural facts
    structural reflection declaration identities

Sum::Variants:
    EnumDeclaration
    exact VariantInfo products
    immediate payload Type dependencies
```

Non-dependency:

```text
unrelated callable body
unrelated inherent methods
runtime ProductLayout registry
heap object layout
source ranges
```

---

# 27. Implementation/commit decomposition

Recommended coherent commits:

```text
1.  feat(lang007): generalize type-member static domains
2.  feat(lang007): add domain-neutral member projection carrier
3.  feat(lang007): support associated-member result domains
4.  feat(lang007): add structural reflection Universe surface
5.  feat(lang007): derive sealed structural conformance evidence
6.  feat(lang007): project ordered Product reflection
7.  feat(lang007): integrate final unordered structural classification
8.  feat(lang007): project unordered shape lookup and views
9.  feat(lang007): publish Sum Variant reflection
10. feat(lang007): close symbolic structural member specialization
11. test(lang007): close incremental and cross-module structural semantics
12. docs(lang007): close P5 walkthrough/handoff/checkpoint
```

Do not combine unrelated P4 fixes or P6/P7/P8 work into P5 commits.

---

# 28. Model delegation guidance

## Advanced/high-reasoning model required

Use a stronger model for:

```text
T1 TypeMemberDomain / StaticValueDomain convergence
T2 domain-neutral projection identity/storage
T3 trait associated result-domain semantics
T5 compiler-derived conformance evidence integration
all GenericConformanceEvidence exhaustive-match changes
T7 P4 unordered synchronization
T10 stuck/reduced generic specialization
T11 fingerprints/currentness
```

These can compile and still be semantically wrong.

## Medium model after APIs are frozen

Suitable for:

```text
T4 Universe declaration files/registration
T6 direct ordered StaticValue projection
T8 direct unordered StaticValue construction after row API is fixed
T9 Variant Pack construction
focused semantic tests
```

## Lower-end model

Suitable only after exact instructions exist:

```text
package exposure wiring
mechanical exhaustive-match arms explicitly specified by architect
straightforward positive/negative fixtures
walkthrough/handoff synchronization
```

Do not delegate architectural decisions about proof provenance, projection storage, or P4 row ownership to a lower-end model.

---

# 29. Debugging discipline

When a P5 test fails:

## Member-domain failure

Trace:

```text
syntax domain
→ AST
→ checked domain
→ projection argument/result
```

Do not patch with `KindId::TYPE`.

## Structural proof failure

Trace:

```text
GenericConformanceKey
→ canonical trait identity
→ structural classification
→ proof candidate
→ evidence fingerprint
```

Do not add fake source conformance.

## Reflection mismatch

Trace:

```text
subject Type
→ lower structural descriptor
→ projector
→ StaticValueStore node
```

Do not inspect runtime layout.

## Unordered lookup mismatch

Trace:

```text
FieldKey
→ RecordRow
→ lookup_unordered_field
→ public FieldLookup mapping
```

Do not implement a second lookup.

## Generic stuck/reduced mismatch

Trace:

```text
member identity
→ evidence
→ domain
→ available specialization
→ projection outcome
```

Do not map uncertainty to `Dynamic`.

---

# 30. Completion truth table

| Evidence | P5 complete? |
|---|---:|
| Universe declarations compile | No |
| member domains support ValueType | No |
| structural evidence variant exists | No |
| ordered Product reflection passes | No |
| Sum reflection passes | No |
| P4 still partial | **No** |
| unordered reflection implemented against provisional P4 | No |
| all focused P5 gates pass after P4 completion | Potentially |
| incremental/cold parity passes | Required |
| no fake TypeId / ImplId architecture audit passes | Required |
| walkthrough/handoff complete | Required |
| broader unrelated workspace failures exist but classified | Can still be focused-tested depending checkpoint policy |
| P6/P7/P8/P9 complete | Not required |

---

# 31. Handoff contract

`LANG007.C4.P5-handoff.md` must tell P6/P7/P8/P9 exactly what they inherit.

## P6 inherits

```text
canonical Product/Sum capabilities
ProductShape / OrderedShape / UnorderedShape values
Variants Pack
UnorderedShape::Lookup/View/Tail
Reduced/Stuck projection semantics
```

P6 must not redesign structural classification.

## P7 inherits

```text
domain-neutral static member projection identity
Stuck structural projections
ordinary reduced static reflection values
```

P7 may residualize/recursively compute using these; it must not change P5 identity.

## P8 inherits

```text
semantic StaticValue DAGs representing reflection
stable member/capability declaration identities
exact semantic dependencies
```

P8 owns durable encoding/materialization.

## P9 inherits

```text
public structural ontology now present
legacy Tuple/Record/Pack type-member surface still potentially present
tooling/source-index baseline
```

P9 owns final public cutover/retirement.

---

# 32. Final architectural target

At P5 completion the compiler should be explainable as:

```text
                         canonical Type
                              │
                              ▼
                    StructuralClassifier
                 (P10 + final P4 unordered)
                              │
                 ┌────────────┴────────────┐
                 │                         │
                 ▼                         ▼
       compiler-derived proof       reflection projector
                 │                         │
                 ▼                         ▼
      GenericConformanceProof         StaticValueStore
                 │                  Data / Variant / Pack /
                 │                     Record / Type
                 │                         │
                 ▼                         ▼
    T |= Product / Sum              ProductShape
    generic evidence                OrderedShape
                                    UnorderedShape
                                    Variants
```

The same lower facts feed both branches.

There is:

```text
one structural classifier
one Record-row/FieldKey authority
one generic conformance proof engine
one static domain vocabulary
one semantic projection identity model
one StaticValueStore
```

There is **not**:

```text
a reflection trait solver
a runtime structural classifier
fake source impls
fake TypeIds for reflection values
a second unordered row calculus
a second static value arena
```

---

# 33. Final execution directive

Implement P5 in this order:

```text
member domains
    ↓
domain-neutral projection
    ↓
associated result-domain syntax
    ↓
canonical Universe structural declarations
    ↓
compiler-derived sealed evidence
    ↓
ordered Product reflection
    ↓
P4 synchronization barrier
    ↓
unordered classification/reflection
    ↓
Sum Variant reflection
    ↓
generic/stuck specialization
    ↓
incremental/tooling closure
    ↓
focused certification and lifecycle closure
```

The two most important prohibitions are:

> **Do not represent an ordinary static reflection value as a TypeId.**

> **Do not represent compiler-derived structural capability evidence as a fabricated source ImplId.**

The repository already has the correct lower structural and static-value foundations. P5 succeeds by connecting those foundations to the existing type-member and conformance systems without introducing parallel semantic authorities.
