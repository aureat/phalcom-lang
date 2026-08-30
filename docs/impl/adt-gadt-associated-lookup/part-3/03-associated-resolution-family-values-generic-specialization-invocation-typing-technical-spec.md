# Phalcom ADT/GADT + Associated Lookup
## Part 3 — Associated Resolution, Family Values, Generic Specialization, Invocation, and Typing

**Status:** Technical specification / requirements analysis  
**Series:** ADT/GADT + Associated Lookup, Part 3 of 6  
**Repository:** `aureat/phalcom-lang`  
**Verified planning branch:** `feat/adts`  
**Verified planning baseline:** `2c8b5840fc5a864968cb2a832540fbcba868d9f8`  
**Baseline commit:** `docs: reorganize language specifications`  
**Intended repository path:** `docs/impl/adt-gadt-associated-lookup/part-3/03-associated-resolution-family-values-generic-specialization-invocation-typing-technical-spec.md`

---

# 1. Executive Summary

Part 1 made `enum`, `@variant`, and the associated-lookup grammar explicit in the AST. Part 2 specifies the declaration universe that Part 3 consumes: enum roots, variants, exact-case types, GADT case environments, variant constructor signatures, associated family identities, associated family publication, visibility, and closed-enum requirements.

Part 3 establishes what associated expressions mean to `phalcom-semantic`.

After Part 3, the compiler must be able to analyze and type expressions such as:

```phalcom
Option<Int>::None
Option<Int>::None()
Option<Int>::None::()
Option::Some(42)
Option::Some::(_)
Option<Int>::Some::*
Option<Int>::Some::*(42)
Response::Error("failed")
System::print::(_)
(System::print::(_))("hello")
Expr<Int>::Int(42)
```

and reject expressions such as:

```phalcom
value::Some                 # ordinary runtime value is not an associated owner
Expr<String>::Int(42)       # GADT result conflicts with explicit owner specialization
Option::Missing::*          # no such associated family
Option::None::*()           # fails if only singleton #None exists and no #None() member exists
```

The central architecture is:

```text
source associated expression
        ↓
resolve declaration-backed owner type form
        ↓
resolve static effective associated family
        ↓
apply exact selector / whole-family / call-shape operation
        ↓
check accessibility
        ↓
specialize owner generics + GADT equalities
        ↓
reuse canonical call binding/inference for callable members
        ↓
publish precise TypeKnowledge + semantic denotation
        ↓
publish machine-readable AssociatedResolution
        ↓
Part 4 consumes the resolution for runtime lowering
```

`::` is **not** another spelling of message dispatch. It never performs ordinary runtime lookup, never falls through to `doesNotUnderstand`, and never requires the compiler to create a legacy runtime `Family` object before a statically known call can proceed.

A first-class `::*` family is a stable, statically resolved, heterogeneous member set. It may contain singleton value members as well as callable members. Its type is structural; its exact declaration identity remains a separate semantic denotation.

Part 3 deliberately stops before runtime layout, allocation, bytecode, VM family representation, case-class representation, discriminants, boxing, and `match`. Those remain Parts 4 and 5.

---

# 2. Verification Boundary and Source of Truth

## 2.1 Repository state inspected for this specification

The `feat/adts` branch was rechecked before drafting this document and resolved to:

```text
2c8b5840fc5a864968cb2a832540fbcba868d9f8
```

At that revision:

- the Part 1 technical specification and implementation plan are committed under `docs/impl/adt-gadt-associated-lookup/part-1/`;
- the Part 2 technical specification and implementation plan are committed under `docs/impl/adt-gadt-associated-lookup/part-2/`;
- the Part 1 AST/parser implementation is present on the branch;
- the user's current Part 2 implementation work is explicitly described as **uncommitted WIP** and therefore cannot be inspected through the GitHub connector used for this analysis.

Part 3 implementation must therefore perform a short post-Part-2 archaeology pass before editing. Exact Rust names in this document follow the committed Part 2 specification and plan; if the WIP implementation has structurally equivalent renamed symbols, the implementation should adapt mechanically rather than create duplicate semantic models.

## 2.2 Precedence

When sources disagree, use this order:

1. decisions explicitly ratified in the project conversation;
2. this Part 3 specification for Part 3 semantics;
3. the committed Part 2 specification for declaration products;
4. the committed Part 1 specification for syntax/AST;
5. current repository implementation reality for exact file/symbol names.

Do not let legacy runtime `Family`, `MakeFamily`, method-reference comments, or old `::` behavior override the new associated model.

---

# 3. Ratified Decisions Carried Into Part 3

The following decisions are normative.

## 3.1 Dot, associated lookup, and reflection are distinct mechanisms

```text
.     ordinary message send / behavioral dispatch
::    static associated lookup / family resolution
>>    retained existing mechanism; semantics outside this Part 3 design
```

`>>` is intentionally not redesigned here.

A future live provider/rebindable capability abstraction is also deferred. Phalcom currently does not support monkey-patching methods by adding/removing/replacing them during ordinary execution, and Part 3 must not introduce such a feature indirectly.

## 3.2 `::` has static semantics

The declaration/family/member selected by `::` is established by semantic analysis for the current program revision.

A first-class family is not a live view that re-reads a runtime class dictionary later. A direct associated call does not perform runtime virtual lookup after semantic selection.

Runtime implementation addresses may be replaceable by tooling such as future hot reload, but the semantic target identity and family membership do not change underneath the analyzed expression.

## 3.3 Behavioral inheritance may contribute to the effective static associated surface

Class-side behavior remains behavior. If behavior is inherited, an associated reference may statically resolve that inherited behavior.

Example:

```phalcom
class Base {
    @class
    build() { ... }

    @class
    build(_ x: Int) { ... }
}

class Derived is Base {
    @class
    build(_ x: Int) { ... }

    @class
    build(_ x: Int, _ y: Int) { ... }
}
```

The effective static family for:

```phalcom
Derived::build::*
```

contains:

```text
#build()      → Base
#build(_)     → Derived
#build(_,_)   → Derived
```

The nearest declaration wins for the same exact selector. Different selector shapes compose into the effective family.

This is a **static hierarchy walk**. It is not ordinary class-object dispatch and must not enter the generic `Class` instance-behavior tail used by runtime class-object message sending.

Direct associated declarations, including enum variants, do not inherit.

## 3.4 Whole-family identity remains one owner + one selector base

`AssociatedFamilyId` is nominal family identity. Base-name reservation remains family-wide.

For a variant family:

```phalcom
enum Example {
    @variant None
    @variant None()
    @variant None(_ value: Int)
}
```

all three exact members belong to one `None` family:

```text
#None
#None()
#None(_)
```

## 3.5 Getter and zero-argument method are never conflated

```phalcom
Owner::name       # exact getter #name
Owner::name::     # explicit alias for exact getter #name
Owner::name::()   # exact zero-argument method #name()
Owner::name()     # direct family invocation, Method kind, zero slots
```

A family invocation with `()` never selects a getter merely because the getter takes no explicit arguments.

## 3.6 Singleton and constructor variants are distinct

```phalcom
@variant None
```

is a canonical singleton value represented by selector `#None`.

```phalcom
@variant None()
```

is a zero-argument variant constructor represented by selector `#None()`.

Only the latter has `VariantConstructorId` / `VariantConstructorSignature`.

## 3.7 Variant constructors are callable but not methods

Variant constructors must not be smuggled into `CallableId` merely to reuse method dispatch. Part 3 must generalize application target identity enough to invoke both behavioral callables and variant constructors without erasing that distinction.

## 3.8 Exact known associated calls should bypass family reification

```phalcom
Option::Some(42)
```

must resolve directly to the selected member and canonical call checker. It must not semantically mean:

```text
materialize Option::Some::*
then invoke it
```

Part 4 may compile it directly.

---

# 4. Part 1 Implementation Findings Relevant to Part 3

## 4.1 Dedicated enum AST exists

The landed AST contains:

```rust
Statement::Enum(EnumDef)
```

with dedicated:

```text
EnumDef
EnumMember
EnumBehaviorMember
VariantDecl
VariantPayloadSyntax
VariantBody
```

`VariantDecl.payload == None` means the getter-shaped singleton form. `Some(VariantPayloadSyntax { parameters: [] })` preserves the distinct explicit zero-argument constructor.

Part 3 must consume Part 2 semantic products derived from this AST. It must not reinterpret variants directly from `ClassMember::Variant` or legacy sealed-class expansion.

## 4.2 Dedicated associated expression AST exists

Part 1 landed:

```rust
Expr::AssociatedLookup(Box<AssociatedLookupExpr>)
Expr::AssociatedInvoke(Box<AssociatedInvokeExpr>)
```

`AssociatedLookupExpr` retains the receiver, separator range, and normalized associated member syntax. Named modes distinguish getter lookup, exact narrowing, and whole family. Operators/subscripts preserve exact selector syntax.

`AssociatedInvokeExpr` is separate from ordinary call syntax, which is important: `owner::name(args)` is already represented as direct associated invocation rather than “lookup getter then call it.”

## 4.3 Core lowering remains intentionally blocked

The compiler still rejects `AssociatedLookup` and `AssociatedInvoke` as not lowered yet. That staging is correct and must remain through Part 3.

Part 3 produces semantic resolution records. Part 4 chooses bytecode/runtime representation.

## 4.4 Some stale comments remain

The landed AST still contains comments near the new associated nodes referring to old “method reference” / bound forms. Part 3 should clean those comments when touching the affected semantic boundary, but no AST redesign is required.

---

# 5. Current Semantic Architecture Relevant to Part 3

## 5.1 Existing call analysis is the canonical application engine

`phalcom-semantic/src/checker/call.rs` already owns:

- `ApplicationArgument`;
- `static_call_shape`;
- static positional/labeled argument binding;
- bidirectional argument analysis;
- generic inference through `InferenceSession`;
- expected-return constraints;
- causal invalidity/status propagation;
- `apply_resolved_callable`.

Part 3 must reuse this engine after associated member selection.

Do not implement an enum-specific argument binder or a separate family generic solver.

## 5.2 `CallableApplicationTarget` currently assumes behavioral identity

Current code carries:

```rust
pub(crate) struct CallableApplicationTarget {
    pub signature: CallableSignature,
    pub callable: Option<CallableId>,
    pub authority: CallTargetAuthority,
    pub specialization: Option<DispatchSignatureSpecialization>,
}
```

That is insufficient for variant constructors because `VariantConstructorId` is intentionally not a `CallableId`.

Part 3 must generalize selected invocation identity.

## 5.3 Canonical callable type is single-signature

Current `TypeData::Callable(CallableType)` represents one callable shape.

A first-class family that supports multiple selector kinds/shapes cannot be represented soundly as either one callable or a union of callable types.

A union means “the value is one of these alternatives.” A family means “this one value supports all these published member operations.”

Therefore Part 3 requires a dedicated canonical family type.

## 5.3.1 Canonical rest-call projection is currently lossy and blocks rest binding

The landed canonical declaration signature retains lane-aware `RestMode::{None, Positional, Labeled, Complete}`, but the current application projection reduces that fact to `CallableParameter.rest: bool`, and `checker/call.rs::bind_static_arguments` returns `UnsupportedRestShape` whenever any projected parameter is rest-shaped.

This is material to Part 3 because the ratified family-selection rule says an exact selector wins before a compatible rest member. Part 3 therefore cannot claim semantic rest-family applicability merely by inspecting VM/rest metadata. It must first preserve the semantic rest lane in projected callable signatures/types and make the canonical static argument binder capable of proving statically known rest calls. Dynamic expansions/computed labels remain a separate dynamic-shape boundary.

---

## 5.4 Explicit kinds already exist

`TypeStore` assigns every canonical type form a `KindId`. Generic nominal forms can have arrow kinds and `apply_type_form` supports partial application.

Examples:

```text
Option        : Type -> Type
Option<Int>   : Type
Result        : Type -> Type -> Type
Result<Int>   : Type -> Type
```

Associated ownership must therefore be based on a declaration-backed **type form**, not on “kind must equal `Type`.”

## 5.5 Type-form denotation already exists

A resolved type name currently synthesizes its runtime class-object value type while carrying:

```rust
SemanticDenotation::TypeForm(info.form)
```

This is the correct starting point for associated owner resolution. Do not infer owner identity merely from runtime `ClassObject` type or spelling.

## 5.6 Expression products need richer associated identity

`ExpressionAnalysis` currently has:

```text
knowledge
optional CallableId
denotation
status
explanation
optional CallResolutionId
```

Part 4 needs more information than this for associated expressions. Part 3 must publish a machine-readable associated resolution record rather than force later lowering to re-resolve AST syntax.

## 5.7 Source indexing currently traverses but does not attach associated targets

The current source-index builder visits associated receivers and invocation arguments but does not attach a target to the associated base/selector occurrence.

Part 3 must attach exact semantic targets from formal resolution.

---

# 6. Part 2 Contract Consumed by Part 3

The committed Part 2 specification requires the following semantic products or their structural equivalents:

```text
EnumSemanticTable / EnumInfo
AssociatedFamilyTable
AssociatedSurface
AssociatedFamilyInfo
AssociatedFamilyKind
AssociatedMemberId
VariantInfo
VariantConstructorSignature
VariantVisibility
CaseTypeEnvironment
TypeData::ExactCase
VariantId
VariantConstructorId
AssociatedFamilyId
```

The relevant conceptual shapes are:

```rust
pub enum AssociatedFamilyKind {
    Behavioral,
    Variant,
}

pub enum AssociatedMemberId {
    Behavioral(CallableId),
    Variant(VariantId),
}

pub struct AssociatedFamilyInfo {
    pub id: AssociatedFamilyId,
    pub kind: AssociatedFamilyKind,
    pub members: Box<[AssociatedMemberId]>,
}
```

Part 3 consumes those products. It must not rebuild variants/families by rescanning enum AST.

If the Part 2 WIP implementation materially diverges from these shapes, reconcile the divergence before implementing Part 3.

---

# 7. Goals

Part 3 is complete when all of the following are true.

1. `AssociatedLookupExpr` and `AssociatedInvokeExpr` are formally analyzed by `phalcom-semantic`.
2. `::` resolves only declaration-backed type-form owners.
3. Bare and partially applied generic type forms are valid associated owners.
4. Ordinary runtime values do not gain associated lookup from their runtime class.
5. Static effective behavioral families include statically inherited class-side behavior with deterministic exact-selector shadowing.
6. Direct associated declarations/variants do not inherit.
7. Exact getter, exact method, setter, operator, and subscript lookup remain selector-exact.
8. Variant singleton getter lookup produces the singleton exact-case value.
9. Behavioral getter lookup reifies the exact getter callable/member rather than invoking it.
10. Exact variant constructors are first-class callable values without becoming methods.
11. `::*` reifies the complete accessible family view.
12. A first-class family has a canonical structural family type plus captured associated-value denotation retaining nominal family identity, lookup-owner specialization, and access-filtered exact member bindings.
13. Family types preserve selector kind and distinguish value members from callable members.
14. Family invocation uses Method-kind call shape; getter is never a zero-argument method fallback.
15. Direct `owner::name(args)` and call-on-family share one member-selection/application algorithm.
16. Every ordinary call on a family publishes a body-local family-application resolution so later lowering never repeats member selection.
17. Known exact family calls use existing canonical call binding/inference.
18. Explicit/partial owner generic specialization composes with argument inference and expected-result inference.
19. GADT constructor result constraints are checked against explicit owner specialization.
20. Remaining unresolved generic variables are never silently widened to `Dynamic`, `Object`, or another default.
21. Visibility is enforced at associated acquisition/selection and reified values behave as stable capabilities.
22. Dynamic argument packs, if supported, route only among the statically frozen candidate set.
23. Source occurrences target exact variants, constructors/callables, or associated family identities.
24. Callable-body incremental dependencies include associated/enum semantic products consumed during resolution.
25. Part 3 publishes machine-readable `AssociatedResolution` and `FamilyApplicationResolution` products suitable for Part 4 lowering.
26. No Part 3 code routes new `::` semantics through legacy `MakeFamily`, ordinary dispatch fallback, or dNU.

---

# 8. Non-Goals

Part 3 does not implement:

- runtime enum representation;
- runtime exact case classes;
- allocation policy;
- inline/unboxed variant layout;
- discriminants/tags;
- boxing at dynamic-erasure boundaries;
- runtime family object layout;
- new bytecodes for associated calls;
- compiler lowering of `AssociatedLookupExpr` or `AssociatedInvokeExpr`;
- VM execution of new associated constructs;
- `match` syntax;
- exhaustiveness;
- payload projection in patterns;
- branch-local GADT equality introduction;
- `>>` redesign;
- monkey-patching;
- live provider/rebindable capability semantics;
- general Hindley–Milner let-generalization;
- general first-class `forall` source syntax;
- a source annotation grammar for structural family types;
- broad reflection metadata migration;
- LSP-owned semantic lookup.

Runtime lowering is Part 4. Pattern elimination is Part 5. Core migration/reflection cleanup is Part 6.

---

# 9. Normative Vocabulary

Use these terms consistently.

**Associated owner form** — the canonical declaration-backed `TypeId` denoted by the expression left of `::`; it may be unsaturated or partially applied and therefore have an arrow kind.

**Lookup owner** — the root `DeclarationId` whose associated namespace is queried.

**Associated family** — the nominal owner/base family identified by `AssociatedFamilyId`.

**Effective associated family** — the statically composed family visible from a lookup owner, including inherited behavioral members where allowed.

**Associated member** — one exact member of a family, preserving selector kind and semantic category.

**Family view** — an owner-specialized and access-filtered reification of an associated family.

**Invocation target** — an executable selected member: behavioral `CallableId` or `VariantConstructorId`.

**Associated resolution** — the body-local semantic record describing the owner, family, exact selected member(s), specialization, result type, and static/dynamic routing mode for one associated expression.

---
# 10. Associated Owner Resolution

## 10.1 Owner resolution starts from semantic denotation

Part 1 represents explicit generic type-form expressions such as `Option<Int>` as `Expr::TypeForm`. The landed expression checker does not yet give that AST form formal expression semantics. Part 3 must close that prerequisite: resolve the contained type annotation through the canonical type resolver/kind checker, synthesize the appropriate class-object value knowledge for the nominal origin, and attach `SemanticDenotation::TypeForm(applied_or_partial_form)`. The denotation, not the runtime class-object type, carries the specialization used by `::`.

For:

```phalcom
Option::Some
```

analyze the receiver `Option` normally. A declaration-backed type name should carry:

```rust
SemanticDenotation::TypeForm(option_form)
```

The associated resolver must consume that denotation.

Do not resolve the owner by:

- extracting the runtime class from `TypeKnowledge` alone;
- re-reading identifier text;
- ordinary message dispatch;
- guessing from a class-object shape.

This permits aliases/bindings that preserve the exact type-form denotation to remain meaningful.

## 10.2 The owner need not have kind `Type`

These are valid owner forms:

```text
Option              : Type -> Type
Option<Int>         : Type
Result              : Type -> Type -> Type
Result<Int>         : Type -> Type
```

The requirement is:

> the owner form has a canonical nominal declaration origin and denotes that declaration's type constructor or a valid partial/full application of it.

`TypeStore` already has the kind information required to validate partial application.

## 10.3 Recover declaration origin and supplied generic arguments

The resolver needs a normalized owner descriptor conceptually equivalent to:

```rust
pub struct AssociatedOwnerResolution {
    pub form: TypeId,
    pub declaration: DeclarationId,
    pub supplied_arguments: Box<[TypeId]>,
    pub residual_kind: KindId,
}
```

The exact Rust shape is flexible.

For:

```phalcom
Result<Int>::Ok
```

this descriptor records:

```text
declaration = Result
supplied_arguments = [Int]
residual kind = Type -> Type
```

Do not invent fresh anonymous declaration identities for specializations.

## 10.4 Invalid owners

An ordinary runtime value is not an associated owner:

```phalcom
const x = Option::Some(1)
x::Some                 # invalid
```

A generic type parameter is not declaration-backed merely because its kind is known:

```phalcom
method<T: Type>(...) {
    T::make              # unsupported in Part 3 unless a future associated constraint proves a declaration family
}
```

Likewise, unioned or dynamic type forms are not speculatively searched.

Diagnostics must distinguish:

```text
owner has no exact type-form denotation
owner type form is not declaration-backed
owner declaration has no associated surface
```

rather than collapsing all cases into “member missing.”

---

# 11. Static Effective Associated Family Resolution

## 11.1 Variant/direct associated families

Variants are direct associated declarations of the enum root. They do not inherit into unrelated declarations.

For an enum owner, Part 3 consumes the Part 2 `AssociatedFamilyInfo` directly, subject to generic specialization and visibility.

## 11.2 Behavioral families

Behavioral associated lookup is a static projection of class-side behavior.

For a class lookup owner `D`, build the effective family by statically walking source inheritance from `D` toward its declared superclass chain.

For each exact selector:

1. the nearest class-side declaration wins;
2. a different exact selector of the same base may be contributed by an ancestor;
3. the effective set is deterministic and duplicate-free by exact `Selector`.

Do **not** continue from the end of the source class hierarchy into `Class` instance behavior merely because ordinary class-object dispatch does so.

## 11.3 Family-category conflict remains a declaration error

If Part 2 correctly enforces base-name reservation, a single effective owner surface does not contain both a variant family and behavioral family with the same base.

Part 3 should treat such a state as an internal semantic invariant violation, not attempt speculative “try behavioral, then variant” resolution.

## 11.4 Effective family result

The resolver should produce an immutable, canonical-order view conceptually equivalent to:

```rust
pub struct EffectiveAssociatedFamily {
    pub id: AssociatedFamilyId,
    pub lookup_owner: DeclarationId,
    pub kind: AssociatedFamilyKind,
    pub members: Box<[AssociatedMemberId]>,
}
```

For an effective behavioral family, `id.owner` is the **lookup owner**, not necessarily the defining owner of any member. Thus `PureChild::build::*` has `AssociatedFamilyId(PureChild, build)` even when every selected `CallableId` is defined by `Base`. This identity represents the effective static associated surface being referenced; exact member IDs continue to retain their defining declarations.

A behavioral family must be found by walking the source hierarchy even when the lookup owner has no local family of that base. The existence of a local behavioral contribution is not a prerequisite for inherited associated lookup.

For inherited behavioral members, the `CallableId` retains the defining owner. `lookup_owner` remains the declaration written on the left of `::`.

This distinction is required for source navigation, `Self` specialization, visibility, diagnostics, and Part 4 lowering.

---

# 12. First-Class Family Semantic Model

## 12.1 A family is heterogeneous

A first-class associated family is not merely an overloaded function.

Example:

```phalcom
enum Option<T> {
    @variant None
    @variant None()
    @variant None(_ value: T)
}
```

`Option::None::*` contains:

```text
#None       singleton value member
#None()     callable constructor member
#None(_)    callable constructor member
```

A behavioral family may likewise contain different selector kinds:

```text
#value
#value()
#value=(put)
```

The structural family model must preserve selector kind.

## 12.2 Family type and family identity are separate

The exact family declaration identity is:

```rust
AssociatedFamilyId
```

The value type describes what member operations the family supports.

Two unrelated families may have structurally compatible types while retaining different denotations.

This follows the existing Phalcom distinction between type knowledge and semantic denotation.

## 12.3 Canonical family type

Part 3 should add a compact canonical family type to `TypeStore`.

Recommended representation:

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FamilyTypeId(pub u32);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FamilyOperationShape {
    pub kind: SelectorKind,
    pub slots: Box<[SelectorSlot]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FamilyMemberTypeKind {
    Value,
    Callable,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FamilyMemberType {
    pub operation: FamilyOperationShape,
    pub member_kind: FamilyMemberTypeKind,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FamilyType {
    pub members: Box<[FamilyMemberType]>,
}
```

and:

```rust
TypeData::Family(FamilyTypeId)
```

`ty` means:

- for `Value`: the produced/stored value type;
- for `Callable`: a canonical `TypeData::Callable` type describing invocation after exact member acquisition.

This is a recommended concrete shape, not a requirement that every struct live in `store.rs`. A focused `types/family.rs` arena is preferred.

## 12.4 Base name is not part of structural family type identity

`Some::*` and `Ok::*` should not be structurally incompatible merely because their source base names differ.

The operation shape contains selector **kind + slots**, not the selector base. The captured family denotation retains nominal `AssociatedFamilyId` (including the base) plus lookup-owner/capability provenance.

This permits higher-order APIs to eventually accept structurally compatible families.

## 12.5 Canonicalization

Family member entries must be:

- sorted by operation shape;
- duplicate-free;
- deterministic across hash-map traversal order;
- independent of source ranges;
- independent of runtime addresses.

A duplicate operation shape with incompatible semantic member descriptions is an invariant failure because Part 2 reservation/effective-family composition should already have selected one exact member per selector.

## 12.6 Family type is not a union

Never represent:

```text
family { () -> A, (Int) -> B }
```

as:

```text
(() -> A) | ((Int) -> B)
```

The union says the value may be either callable. A family promises that one value supports both operations.

## 12.7 Structural relation

At minimum, Part 3 should make family types participate soundly in canonical type relation.

Recommended rule:

```text
ProvidedFamily <: RequiredFamily
```

iff every required operation shape exists in the provided family and the corresponding member types are compatible.

For callable members, reuse normal callable variance/relation.

For value members, require provided value type to be assignable/subtype-compatible with the required value type.

Extra provided operations are allowed (width subtyping).

No source annotation syntax for family types is required in Part 3.

---

# 13. Exact Associated Lookup

## 13.1 Named getter lookup

```phalcom
Owner::name
Owner::name::
```

both request exact selector:

```text
#name
```

Absence of `#name` is an exact-member error. It does not mean “return the whole family.”

## 13.2 Exact method lookup

```phalcom
Owner::name::()
Owner::name::(_)
Owner::name::(_, reason)
```

request exact Method-kind selectors.

## 13.3 Setter, operator, and subscript lookup

Existing Part 1 exact selector syntax must map directly to canonical exact selectors.

No extra dynamic family-search layer is introduced.

## 13.4 Variant singleton exact lookup

For a variant member whose Part 2 shape is singleton:

```phalcom
Option<Int>::None
```

produces the canonical singleton exact-case value type, specialized to the owner arguments.

The expression denotation targets the exact `VariantId`.

There is no `VariantConstructorId` for this member.

## 13.5 Behavioral getter exact lookup

For behavioral getter:

```phalcom
Math::pi
```

associated lookup reifies the exact getter callable/member. It does **not** execute the getter.

After exact reification:

```phalcom
(Math::pi)()
```

is ordinary invocation of the already-selected getter callable.

This differs from family invocation:

```phalcom
Math::pi::*()
```

which asks the family for Method selector `#pi()`. If no `#pi()` member exists, it fails; it never falls back to getter `#pi`.

## 13.6 Exact variant constructor lookup

```phalcom
Option<Int>::Some::(_)
```

reifies an exact callable value whose semantic executable identity is `VariantConstructorId` and whose result is the specialized exact-case type.

It must not fabricate a class-side method `CallableId`.

---

# 14. Whole-Family Reification

## 14.1 Syntax

```phalcom
Owner::name::*
```

reifies a first-class family view.

## 14.2 Reification result

The expression publishes:

- canonical structural `TypeData::Family` knowledge;
- `SemanticDenotation::AssociatedValue(AssociatedValueDenotation::Family { ... })`, retaining the nominal `AssociatedFamilyId`, lookup owner form, and exact accessible captured members;
- a machine-readable associated resolution containing the exact accessible member set and owner specialization.

## 14.3 Visibility filtering

The canonical semantic family contains all effective members. The reified family value contains the members accessible at the acquisition site.

Once reified, the value behaves as a capability: later invocation does not repeat source-level visibility checks based on the caller's lexical scope.

If the family exists but no member can legally be acquired, diagnose inaccessibility rather than “missing family.”

## 14.4 Getter-only/singleton-only family is valid

A family containing only:

```text
#End : singleton value
```

is still a valid first-class family.

It simply has no Method-kind call operation.

Attempting `family()` should produce a family call-shape diagnostic, not claim the family value itself is malformed.

---

# 15. Direct Associated Invocation

## 15.1 Static call pipeline

For:

```phalcom
Response::Error("failed")
```

perform:

```text
resolve owner type form
→ resolve Error family
→ derive Method-kind selector shape from arguments
→ select exact member if present
→ otherwise consider compatible statically published rest member
→ check accessibility
→ specialize owner/GADT generics
→ adapt selected member to canonical invocation target
→ run existing argument/generic/expected-result call checker
→ publish exact result
```

## 15.2 No family materialization requirement

A statically shaped direct associated call does not create a first-class family value semantically.

The resolution record may mention the family for provenance/dependency tracking, but the execution target is the selected exact member.

## 15.3 Family invocation is Method-kind only

Ordinary parentheses create a Method selector shape.

Therefore, for family:

```text
#None
#None()
#None(_)
```

these resolve as:

```phalcom
Option::None        # #None singleton
Option::None()      # #None() constructor
Option::None(1)     # #None(_) constructor
```

and:

```phalcom
Option::None::*()   # #None()
```

never selects `#None`.

## 15.4 Exact member beats rest

If an exact selector exists for the static call shape, it is selected before any compatible rest member.

If that exact member is inaccessible, report the access violation. Do not skip the exact member and silently select an accessible rest member.

Rest applicability must be established by the canonical semantic call-shape model, preserving positional, labeled, and complete rest lanes. The current `rest: bool` projection / unconditional `UnsupportedRestShape` path is insufficient and must be repaired as an enabling Part 3 change. Static associated selection must not copy VM `MethodFamily` routing rules or use runtime dispatch as semantic evidence.

This preserves dispatch/family semantics independently of access control.

---

# 16. Ordinary Invocation of Reified Family Values

Part 1 parses a postfix call on a family expression through the ordinary callable path.

Part 3 should recognize `TypeData::Family` when analyzing ordinary invocation / `.call` and route it through the same family-selection engine used by `AssociatedInvokeExpr`.

Conceptually:

```text
Owner::name(args)
        └──────────→ FamilyApplicationEngine

Owner::name::*(args)
        └──────────→ FamilyApplicationEngine
```

After a member is selected, both converge on the canonical call checker.

Do not dispatch `.call` dynamically against a runtime class representing a family.

When the family expression is syntactically immediate and owner specialization is not yet fixed, the analyzer may fuse lookup + application so argument/expected-result inference can instantiate generics before publishing an intermediate family value.

## 16.1 Ordinary family calls also need a persistent semantic resolution

A call on a stored family is not itself an `AssociatedLookupExpr` or `AssociatedInvokeExpr`:

```phalcom
const f = Derived<Int>::build::*
const x = f(value)
```

Part 4 must not inspect `TypeData::Family`, a captured denotation, and the source call shape and then repeat Part 3's member-selection algorithm. Part 3 therefore publishes a body-local **family application resolution** for every ordinary call whose callee is a family value.

Recommended range-free shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilyApplicationResolution {
    pub family_type: TypeId,
    pub selection: FamilyApplicationSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FamilyApplicationSelection {
    Static {
        operation: FamilyOperationShape,
        target: Option<InvocationTargetId>,
        callable_type: TypeId,
        result_type: TypeId,
    },
    Dynamic {
        candidates: Box<[FamilyApplicationCandidate]>,
        result_type: Option<TypeId>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilyApplicationCandidate {
    pub operation: FamilyOperationShape,
    pub target: Option<InvocationTargetId>,
    pub callable_type: TypeId,
}
```

`target: Some(...)` is available when captured associated denotation proves the exact static executable identity. `target: None` is valid after an abstraction/merge has retained only the structural family type. In that case Part 4 invokes/routes through the runtime family value; it must not recover a target through associated lookup.

The implementation may integrate this with a future general `CallResolution` arena and use the existing `CallResolutionId` scaffold, but Part 3 must not depend on such a broader refactor. A focused `FamilyApplicationResolutionIndex = BTreeMap<ExpressionId, ...>` is sufficient.

Thus Part 3 publishes two complementary records:

```text
AssociatedResolutionIndex
    associated syntax acquisition/direct invocation

FamilyApplicationResolutionIndex
    ordinary invocation of an already-first-class family value
```

---

# 17. Dynamic Argument Packs

## 17.1 Static family, dynamic shape

Expressions containing expansions or computed labels may have unknown call shape at analysis time.

The family itself remains statically known.

The resolver must freeze:

- family identity;
- owner specialization information known so far;
- accessible candidate members;
- candidate signatures.

Only operation-shape routing may be deferred.

## 17.2 No ordinary dispatch reopening

At runtime, a Part 4 router may inspect the argument pack and choose among the frozen members.

It must not:

- search subclass/runtime dictionaries for additional members;
- invoke ordinary dNU if the shape does not match;
- turn a variant family into method dispatch.

Failure is an associated-family call-shape error.

## 17.3 Static result knowledge

When a finite candidate set permits a sound result join, Part 3 may publish a union/join of candidate result types.

If generic dependencies or dynamic labels make that unsound or unbounded, publish the existing appropriate dynamic/blocked boundary instead of inventing a result type.

Part 3 does not require Part 4 runtime support to execute this path yet; it must nevertheless publish truthful resolution metadata. Direct associated dynamic invocation records `AssociatedResolutionKind::DynamicInvoke`; a dynamic call on an already-reified family records `FamilyApplicationSelection::Dynamic`. Both freeze the same finite semantic candidate universe and neither may be reconstructed in Part 4.

---
# 18. Generic Owner Specialization

## 18.1 Generic constraints come from several places

Part 3 must combine:

1. explicit/partial owner arguments;
2. declaration generic constraints;
3. GADT case equalities;
4. invocation argument types;
5. expected result type.

Example:

```phalcom
Option<Int>::Some(42)
```

starts with:

```text
T = Int
```

from the owner.

Example:

```phalcom
Option::Some(42)
```

starts with `T` open and derives:

```text
42 : Int
T = Int
```

from the constructor parameter.

Example:

```phalcom
const x: Option<Int> = Option::None
```

has no payload argument but can derive `T = Int` from expected result context.

## 18.2 Reuse canonical type-parameter identities and kinds

Owner generic parameters already have `TypeParameterId` and explicit kinds.

Part 3 must not create a second enum-specific parameter namespace.

An unsaturated declaration is not itself “unknown.” Its type constructor and residual kind are known. What may remain underconstrained is the **value specialization** produced by a particular lookup/invocation.

## 18.3 Partial owner application

For:

```phalcom
Result<Int>::Ok
```

only the first declaration parameter is fixed.

The associated resolver must construct a substitution/environment for supplied declaration parameters and retain residual parameters for later inference.

Do not require the owner form to be fully saturated before associated lookup.

## 18.4 Existing specialization machinery should be extracted/reused

`CheckingContext` already specializes callable signatures for applied message receivers using `substitution_for_applied_receiver` and `specialize_self_type`. That helper is not sufficient unchanged for an inherited generic associated member because the lookup owner and defining owner can have different generic forms.

For example:

```phalcom
class Base<T> {
    @class make(_ value: T) -> T { value }
}
class Derived<U> is Base<List<U>> {}
```

`Derived<Int>::make::(_)` resolves the exact behavioral declaration defined by `Base`, but its defining-owner form is `Base<List<Int>>`. The resolver must therefore:

```text
lookup owner form Derived<Int>
    -> project through the declared generic supertype template(s)
    -> defining owner form Base<List<Int>>
    -> substitute Base's declaration parameters in the callable signature
    -> specialize owner-relative Self using the original lookup owner form
```

This projection must compose across multiple generic inheritance hops. Do not bind a defining owner's type parameters directly from the lookup owner's arguments by positional index.

Part 3 should extract/generalize the existing substitution and `Self` specialization machinery around this two-owner distinction rather than copy it into a second generic engine. Associated lookup differs in **how the target is found**, not in the semantics of canonical type substitution once the correct defining-owner form is established.

---

# 19. Decision Gate for Bare Generic Reified Values

This is the principal generic language decision that remains intentionally unratified. Part 3 must preserve enough semantic information to support either policy without silently choosing one during implementation.

## 19.1 What is known

A bare generic declaration form such as:

```text
Option : Type -> Type
```

is completely known and is a valid associated owner.

Likewise, the declaration template for:

```phalcom
Option::Some::(_)
```

is known conceptually as:

```text
T -> ExactCase(Some, Option<T>)
```

The problem is not kind inference or family identity. The unresolved question is how a **stored first-class value** carries declaration-provided universal quantification when no concrete `T` is available.

## 19.2 Behavior required regardless of the final policy

Part 3 does not introduce arbitrary Hindley–Milner let-generalization or first-use monomorphization. First-use monomorphization is rejected because it makes typing depend on use order.

The associated instantiation layer must preserve residual declaration binders together with their explicit `KindId`s while applying every constraint already available from the owner, GADT case environment, call arguments, and expected result.

These must work independently of the unresolved reification policy:

```phalcom
Option::Some(42)
Option<Int>::Some::(_)
Option<Int>::Some::*
Option::Some::*(42)
const x: Option<Int> = Option::None
```

Never turn an unresolved declaration parameter into `Dynamic`, `Object`, `Any`, or another erased default merely to finish family/constructor typing.

## 19.3 Decision Gate G1 — escaping declaration-polymorphic associated values

Before implementation commits to the behavior of:

```phalcom
const f = Option::Some::*
const g = Option::Some::(_)
```

with no contextual specialization, one of these policies must be ratified explicitly:

### G1-A — contextual-only reification in v1

The resolver may carry a residual template internally while analyzing the expression, but a value that escapes with unresolved declaration binders is underconstrained unless an expected callable/family type specializes those binders.

Advantages: smallest value-type extension and no implicit quantified-value feature.

Cost: a declaration that is semantically polymorphic cannot be stored in bare form without explicit/contextual specialization.

### G1-B — declaration-provided rank-1 schemes

Reifying an associated declaration preserves only the universal binders already declared by its owner/declaration. Each invocation independently instantiates those binders.

Conceptually:

```text
Option::Some::(_)
    : forall T: Type. T -> ExactCase(Some, Option<T>)
```

This is **not** arbitrary expression generalization and does not imply HM inference for ordinary local expressions.

Advantages: constructors/families remain genuinely first-class and functional-programming-friendly.

Cost: the formal value-type/instantiation layer needs a truthful rank-1 scheme representation and flow rules.

## 19.4 Direct invocation remains a separate question

A performed invocation that leaves a result-relevant declaration parameter unsolved, for example conceptually `Result::Ok(1)` where `E` appears only in `Result<Int,E>`, cannot be made precise merely by calling the declaration polymorphic. Unless Phalcom separately ratifies an existential result model, the direct call remains underconstrained when owner + arguments + expected result + GADT facts cannot solve `E`.

Do not confuse this concrete-result problem with G1's stored-declaration polymorphism.

## 19.5 Architecture must remain policy-neutral until G1 is recorded

`AssociatedOwnerResolution`, specialized member templates, family views, and `AssociatedResolution` must retain residual declaration binder identity and kinds. Do not bake monomorphic-only assumptions into `AssociatedFamilyInfo`, family type interning, or the Part 4 handoff.

Once G1 is recorded, the implementation spec/plan may select the corresponding reification behavior without changing `::` syntax, family identity, static family membership, or direct invocation semantics.

---

# 20. GADT Constructor Specialization and Owner Compatibility

Consider:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
}
```

Part 2 publishes the result template and case environment proving the case inhabits `Expr<Int>`.

Part 3 must enforce:

```phalcom
Expr::Int(42)          # valid, result exact case of Expr<Int>
Expr<Int>::Int(42)     # valid
Expr<String>::Int(42)  # invalid
```

The invalid case is not merely a payload argument mismatch. The explicit associated owner specialization conflicts with the variant's declared result specialization.

The same compatibility check applies when reifying the constructor:

```phalcom
Expr<String>::Int::(_)
```

must not yield a callable that silently ignores `Expr<String>`.

Required algorithm:

1. start with declaration parameter environment implied by owner type-form arguments;
2. import the variant's `CaseTypeEnvironment` equivalences;
3. normalize/solve those equalities;
4. reject contradictions;
5. specialize payload parameter and exact result templates;
6. feed remaining call-level variables into the existing inference session.

The exact case remains:

```text
ExactCase(VariantTypeId(Int), Expr<Int>)
```

and not merely the enum root.

---

# 21. Generalized Invocation Target Identity

Part 3 needs an executable identity that does not lie about variant constructors.

Recommended:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InvocationTargetId {
    Behavioral(CallableId),
    VariantConstructor(VariantConstructorId),
}
```

`CallableApplicationTarget` or its replacement should carry:

```text
signature
optional InvocationTargetId
application authority
specialization metadata
```

Existing ordinary message sends continue to use `Behavioral(CallableId)`.

Exact structural callable values may have no declaration identity.

This generalization must propagate through:

- call result metadata;
- explanation steps that currently assume `CallableId`;
- dependency recording;
- diagnostics/guidance;
- expression attachment;
- source targets where appropriate.

Do not redesign all callable identities in the language unless necessary. The requirement is a truthful executable target abstraction at the application boundary.

---

# 22. Adapting Variant Constructor Signatures to the Canonical Call Engine

Part 2 `VariantConstructorSignature` should be projected into the same parameter/return representation used by `apply_resolved_callable`.

Projection must preserve:

- external labels;
- parameter order;
- parameter types after owner/GADT specialization;
- exact-case return type;
- evidence origin `ConstructorSemantics`;
- residual generic variables;
- source provenance.

The projection is an application adapter, not a semantic reclassification as a method.

If `CallableSignature.kind` remains the convenient application-level signature container, it may be used as an execution signature so long as the selected identity is still `VariantConstructorId` and no class-side dispatch surface is populated for it.

---

# 23. Associated and Family-Application Resolution Products

Part 4 must not reinterpret associated AST or repeat family member selection for ordinary calls on first-class family values.

Part 3 therefore publishes a body-local associated resolution record for every successfully or partially resolved associated expression, plus the `FamilyApplicationResolution` described in §16.1 for ordinary family-value calls.

Recommended shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedResolution {
    pub owner_form: TypeId,
    pub lookup_owner: DeclarationId,
    pub family: AssociatedFamilyId,
    pub kind: AssociatedResolutionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssociatedResolutionKind {
    ExactValue {
        member: AssociatedMemberId,
        value_type: TypeId,
    },
    ExactCallable {
        member: AssociatedMemberId,
        target: InvocationTargetId,
        callable_type: TypeId,
    },
    Family {
        family_type: TypeId,
        members: Box<[SpecializedAssociatedMember]>,
    },
    StaticInvoke {
        member: AssociatedMemberId,
        target: InvocationTargetId,
        result_type: TypeId,
    },
    DynamicInvoke {
        candidates: Box<[SpecializedAssociatedMember]>,
        result_type: Option<TypeId>,
    },
}
```

The exact storage strategy may use a snapshot-local ID/arena rather than embedding the full record directly into `ExpressionAnalysis`.

Requirements:

- range/source syntax is not part of semantic identity;
- candidate/member order is canonical;
- lookup owner and defining member identity both survive;
- owner generic specialization survives;
- Part 4 can distinguish exact value materialization, exact callable reification, family reification, direct static invocation, ordinary family-value static application, and deferred shape routing without re-running resolution;
- ordinary family applications with lost nominal denotation still publish their selected structural operation/callable type with `target: None`, making the runtime-family-value path explicit rather than inviting semantic rediscovery.

---

# 24. Semantic Denotation and Captured Associated Values

A first-class associated value needs more provenance than the nominal declaration/family ID alone.

This matters immediately for inheritance, generics, and capability capture. For example:

```phalcom
const f = Derived<Int>::build::*
f(...)
```

may contain a member whose defining `CallableId` belongs to `Base`, while the associated value is statically bound to lookup owner `Derived<Int>`. Likewise, a private member captured in an authorized scope must remain callable after the value is passed elsewhere without re-running visibility at the recipient site.

Therefore these are **insufficient** denotations:

```text
AssociatedMember(AssociatedMemberId)       # loses lookup owner/specialization
AssociatedFamily(AssociatedFamilyId)       # loses access-filtered captured membership
```

Part 3 must preserve the captured associated view itself.

Recommended range-free shapes:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedAssociatedMember {
    pub operation: FamilyOperationShape,
    pub member: AssociatedMemberId,
    pub target: Option<InvocationTargetId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssociatedValueDenotation {
    Exact {
        owner_form: TypeId,
        lookup_owner: DeclarationId,
        member: AssociatedMemberId,
        target: Option<InvocationTargetId>,
    },
    Family {
        owner_form: TypeId,
        lookup_owner: DeclarationId,
        family: AssociatedFamilyId,
        members: Arc<[CapturedAssociatedMember]>,
    },
}

pub enum SemanticDenotation {
    TypeForm(TypeId),
    Kind(KindId),
    AssociatedValue(AssociatedValueDenotation),
}
```

The exact Rust placement may put the capture structs in `associated.rs` and reference them from `types/denotation.rs`; they must remain compiler-owned semantic data, not advisory/runtime objects.

`SemanticDenotation` must stop deriving `Copy`; binding/flow code clones the range-free denotation as needed. `Arc<[...]>` or a snapshot-local immutable capture handle is preferred for family membership so ordinary binding reads do not copy the entire family table.

Denotation rules:

```text
Option<Int>::None
    → Exact(owner_form = Option<Int>, member = VariantId(#None), target = None)

Option<Int>::Some::(_)
    → Exact(owner_form = Option<Int>, member = VariantId(#Some(_)),
            target = VariantConstructorId)

Derived<Int>::build::(_)
    → Exact(owner_form = Derived<Int>, lookup_owner = Derived,
            member = defining CallableId (possibly Base),
            target = Behavioral(defining CallableId))

Derived<Int>::build::*
    → Family(owner_form = Derived<Int>, family identity,
             access-filtered exact captured member bindings)
```

The structural `TypeData::Callable` / `TypeData::Family` still says **what operations are type-correct**. The denotation says **which static associated capability was captured and against which lookup-owner specialization**.

Flow/merge rules:

- identical captured denotations survive a binding/branch merge;
- different captured denotations are dropped while structural type knowledge may still merge;
- dropping denotation must not make typing unsound: a later structural family call may be type-checked without a nominal `InvocationTargetId`, and Part 4 must invoke the runtime family value rather than semantically rediscover a family;
- a captured exact/family denotation is never reconstructed by re-running visibility or inheritance lookup at the later invocation site.

Direct invocation expressions retain selected target in `AssociatedResolution`; the final constructed result value does not claim that it denotes the constructor declaration.

---

# 25. Visibility and Capability Semantics

## 25.1 Behavioral visibility

Use compiler-owned member visibility and static source authority.

Inherited behavior uses the defining member's visibility under the lookup context.

## 25.2 Variant visibility

Part 2 separates:

```text
name/match visibility
construction visibility
payload visibility
```

Part 3 associated construction/acquisition consumes the **construction** axis.

For a singleton variant:

```phalcom
Enum::Secret
```

obtains the case value and therefore requires construction/production access even though there is no constructor callable.

The name axis remains relevant to matching/recognition in Part 5.

## 25.3 Exact inaccessible member does not disappear

If an exact member exists but is inaccessible:

```text
resolve exact member
→ access error
```

Do not treat it as absent and do not fall through to rest selection.

## 25.4 Whole-family capture

`Owner::foo::*` captures only accessible members.

The resulting value is a stable capability. Passing it to code in another lexical scope does not re-run private/protected lookup from the recipient's scope. Its `AssociatedValueDenotation::Family` (or equivalent immutable capture handle) preserves the lookup-owner specialization and exact access-filtered member bindings needed for local formal optimization.

If exact denotation is later lost at a branch merge or abstraction boundary, callers may use only the structural family interface and runtime family value; they must not reconstruct privileges or a defining target by re-resolving the nominal family at the new lexical site.

This is not the deferred live-provider feature; the family membership and implementation identities remain statically resolved at acquisition.

---

# 26. Source Identity and Navigation

Part 3 should attach semantic targets to the associated member/base token ranges using the formal resolution result.

Required mappings:

| Source expression | Associated source target |
|---|---|
| `Option::None` | `SemanticTargetId::Variant(VariantId(#None))` |
| `Option::Some::(_)` | exact variant target; constructor identity remains in resolution metadata |
| `Option::Some(1)` | selected exact variant target |
| `System::print::(_)` | `SemanticTargetId::Callable(CallableId)` |
| `System::print(1)` | selected exact callable target |
| `Option::Some::*` | `SemanticTargetId::AssociatedFamily(AssociatedFamilyId)` |

Part 2 already requires `SemanticTargetId::Variant`. Part 3 should add `SemanticTargetId::AssociatedFamily` if Part 2 WIP did not.

Inherited behavioral lookup targets the defining `CallableId`, while `AssociatedResolution.lookup_owner` records the type written on the left.

The source index must remain a publisher of compiler-owned facts. It must not implement a second associated resolver.

---

# 27. Incremental Dependencies

Associated resolution occurs during callable-body analysis and should record the semantic products it consumes.

Part 2 is expected to introduce enum/associated query products. Part 3 should extend `SemanticDependency` as necessary, for example:

```rust
SemanticDependency::EnumDeclaration(DeclarationId)
SemanticDependency::AssociatedSurface(DeclarationId)
```

and continue to use:

```text
CallableSignature(CallableId)
HierarchyEdge(DeclarationId)
DeclarationShell(DeclarationId)
```

for behavioral inherited members/type forms where appropriate.

A caller of `Option::Some(1)` should be invalidated by:

- removal/change of `Some(_)` variant signature;
- change to its GADT result;
- construction visibility change;
- associated family membership change.

It should not be invalidated merely because an unrelated case body implementation changed without semantic signature impact.

A behavioral `Derived::build::*` dependency must include the hierarchy edges and associated/class-side signature products used to form its effective family.

---

# 28. Diagnostics

Part 3 should add stable diagnostic codes rather than reuse generic dynamic-dispatch errors.

Recommended codes:

```text
associated.owner.unresolved
associated.owner.not_type_form
associated.owner.not_declaration_backed
associated.family.missing
associated.family.inaccessible
associated.member.missing
associated.member.inaccessible
associated.member.not_constructible
associated.call.shape_missing
associated.call.ambiguous
associated.call.dynamic_shape
associated.generic.underconstrained
associated.generic.owner_conflict
associated.gadt.owner_conflict
associated.family.type_invalid
```

Messages should expose:

- resolved owner when known;
- requested exact selector or Method call shape;
- available shapes in the family when helpful;
- whether a member exists but is inaccessible;
- generic constraints that conflict;
- the GADT result specialization that made an explicit owner impossible.

Do not report dNU-style “message not understood” for `::` failures.

---

# 29. Explanation / Proof Recording

Part 3 should produce machine-readable explanation nodes sufficient to answer:

```text
why did this family resolve?
why was this exact member selected?
what owner specialization was applied?
what generic/GADT equality fixed this result?
why was another shape inapplicable?
```

Exact UI/presentation polish belongs to later diagnostics work, but the reasoning should be recorded now.

Recommended derivation concepts:

```text
AssociatedOwnerResolution
AssociatedFamilyResolution
AssociatedMemberSelection
AssociatedFamilyCapture
OwnerTypeSpecialization
GadtOwnerCompatibility
VariantConstructorSelection
```

After an executable member is selected, reuse existing callable-selection, argument, generic-inference, return-type, and relation explanation machinery.

Do not create a second explanation arena for associated expressions.

---

# 30. Advisory Analysis

The current advisory domain already contains legacy family/method shapes. It is explicitly not formal type authority.

Part 3 should adapt advisory analysis only after formal associated resolution exists.

Rules:

- formal `AssociatedResolution` is authoritative;
- advisory may project runtime shape from it;
- advisory must not independently rediscover an associated family from receiver shape;
- legacy `Family`/`MethodFamily` shapes must not define the language meaning of `::`;
- if Part 4 runtime representation is not yet available, advisory may conservatively report unknown runtime shape while preserving formal type facts.

---

# 31. Presentation

`TypePresenter` should gain deterministic internal/user-facing rendering for family types.

A diagnostic/hover-friendly form may be:

```text
family {
    #None        : ExactCase<None, Option<Int>>
    #None()      : () -> ExactCase<None(), Option<Int>>
    #None(_)     : (Int) -> ExactCase<None(_), Option<Int>>
}
```

This does not ratify source annotation syntax.

Presentation must preserve:

- getter versus `()`;
- labels without colon in selector identity;
- value versus callable family member;
- canonical ordering.

---

# 32. Part 4 Handoff Contract

Part 3 is complete only if Part 4 can lower associated expressions without performing semantic lookup again.

For each associated expression Part 4 must be able to ask:

```text
what exact owner/family was resolved?
what exact member/target was selected?
was this a singleton value, exact callable, family value, static invocation, or dynamic-shape invocation?
what generic specialization applies?
what exact result type was proven?
what finite candidate table is allowed for dynamic-shape routing?
```

Part 4 remains free to choose:

- zero-allocation direct calls;
- compact immediate singleton encodings;
- runtime family descriptors;
- code pointers/thunks;
- erased/shared generic code versus specialization;
- dynamic erasure boxing.

Those representation choices must not alter Part 3 semantic identity.

---
# 33. End-to-End Semantic Examples

## 33.1 Singleton versus zero-argument constructor

```phalcom
enum Option<T> {
    @variant None
    @variant None()
}

const singleton: Option<Int> = Option::None
const fresh: Option<Int> = Option::None()
const ctor = Option<Int>::None::()
const fresh2 = ctor()
```

Required facts:

```text
Option::None
    family = Option/None
    member = #None
    kind = Variant singleton
    type = ExactCase(#None, Option<Int>)

Option::None()
    family = Option/None
    selected member = #None()
    target = VariantConstructorId(#None())
    type = ExactCase(#None(), Option<Int>)

Option<Int>::None::()
    exact callable target = VariantConstructorId(#None())
    callable type = () -> ExactCase(#None(), Option<Int>)
```

## 33.2 Bare generic constructor invocation

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
}

const x = Option::Some(42)
```

Required inference:

```text
owner form Option : Type -> Type
constructor template T -> Option<T>::Some
argument 42 : Int
solve T = Int
result ExactCase(#Some(_), Option<Int>)
```

No `Dynamic` defaulting.

## 33.3 Expected-result inference for singleton

```phalcom
const x: Option<Int> = Option::None
```

Required inference:

```text
singleton result template Option<T>::None
expected root Option<Int>
solve T = Int
publish ExactCase(#None, Option<Int>)
```

## 33.4 Partially applied owner

```phalcom
enum Result<T, E> {
    @variant Ok(_ value: T)
}

const ctor = Result<Int, String>::Ok::(_)
```

The owner fixes both parameters. The exact constructor type is:

```text
(Int) -> ExactCase(#Ok(_), Result<Int, String>)
```

For an owner such as `Result<Int>` the resolver keeps `E` residual and requires contextual instantiation before a reified value escapes under the Part 3 v1 policy.

## 33.5 GADT conflict

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
}

const ok = Expr<Int>::Int(1)
const bad = Expr<String>::Int(1)
```

`bad` must diagnose an owner/GADT specialization contradiction, even though the payload argument itself is valid.

## 33.6 Effective inherited behavioral family

```phalcom
class Base {
    @class build() -> Base { ... }
    @class build(_ value: Int) -> Base { ... }
}

class Derived is Base {
    @class build(_ value: Int) -> Derived { ... }
    @class build(config config: Config) -> Derived { ... }
}

const f = Derived::build::*
```

Effective family members:

```text
#build()        → Base
#build(_)       → Derived
#build(config)  → Derived
```

No `Class` instance behavior is appended to this lookup.

## 33.7 Exact getter versus family call

```phalcom
class Math {
    @class
    get pi -> Float { ... }
}

const getter = Math::pi
const p = getter()
const q = Math::pi::*()   # error: no #pi() Method member
```

The first exact lookup resolves `#pi` as a behavioral getter callable. The family call requests `#pi()` and fails.

## 33.8 Family as higher-order value

```phalcom
const make = Option<Int>::Some::*
const a = make(1)
```

Required:

```text
make type = structural family containing Method #Some(_) callable operation
make denotation = captured AssociatedValueDenotation::Family {
    owner_form = Option<Int>,
    family = AssociatedFamilyId(Option, Some),
    members = [#Some(_) -> VariantConstructorId(...)]
}
call selects the frozen #Some(_) member from that capture
result exact case Option<Int>::Some
```

No runtime associated search is performed.

## 33.9 Dynamic pack

```phalcom
const f = Response::Error::*
f(*args)
```

Part 3 records only the accessible statically resolved `Error` members as routing candidates. Part 4 may route the runtime pack among those candidates, but cannot discover another member dynamically.

---

# 34. Required Test Matrix

Part 3 implementation must cover at least the following categories.

## 34.1 Owner resolution

- bare generic nominal owner;
- fully applied owner;
- partially applied owner;
- alias/binding preserving `TypeForm` denotation;
- ordinary runtime value rejected;
- dynamic/unknown owner blocked without fallback;
- generic parameter owner rejected unless future constraint design explicitly supports it.

## 34.2 Exact lookup

- singleton getter;
- zero-arg constructor ref;
- payload constructor ref;
- behavioral getter ref;
- behavioral method ref;
- setter exact ref;
- operator exact ref;
- subscript exact ref;
- missing exact selector while same base family exists;
- explicit `owner::name::` alias equals `owner::name` semantically.

## 34.3 Family capture

- variant family containing singleton + zeroarg + unary constructor;
- behavioral family containing getter + method + setter;
- getter-only family is valid;
- family structural type canonicalization independent of declaration/base identity;
- distinct nominal family denotations can share structural type;
- inaccessible members filtered from captured family;
- all-inaccessible family capture diagnosed.

## 34.4 Invocation

- direct associated invocation exact shape;
- first-class family invocation exact shape;
- getter not selected for `()`;
- exact member beats rest;
- inaccessible exact does not fall through to rest;
- argument label mismatch uses canonical call diagnostics;
- generic call inference reuses existing call engine;
- immediate family call can infer owner generic from arguments.

## 34.5 Generics/GADTs

- `Option::Some(1)` infers `Option<Int>`;
- `Option<Int>::Some(1)` validates explicit owner;
- expected result specializes singleton;
- partially applied generic owner retains residual parameters;
- escaping unspecialized generic family follows the recorded G1 policy and is never defaulted to Dynamic/Object;
- GADT result fixes owner parameter;
- explicit GADT owner contradiction diagnosed;
- exact constructor ref also checks GADT owner compatibility.

## 34.6 Inheritance

- inherited class-side associated behavior resolves statically;
- inherited generic behavior projects the lookup owner through one-hop and multi-hop generic superclass templates before defining-owner substitution;
- nearest same-selector override wins;
- ancestor different shapes remain in effective family;
- descendant-added shape joins family;
- source hierarchy end does not enter `Class` instance behavior;
- direct variant/associated family does not inherit.

## 34.7 Source and incremental

- exact variant source occurrence points to `VariantId`;
- family occurrence points to `AssociatedFamilyId`;
- inherited behavioral member points to defining `CallableId`;
- changing family membership invalidates dependent body;
- changing unrelated body implementation without signature change does not invalidate family user;
- source-range-only movement does not change range-free family type/resolution fingerprints.

## 34.8 Negative architecture tests

Search/assert that new associated expression analysis does not call:

```text
ordinary resolve_dispatch_target as the family resolver
legacy MakeFamily
runtime dNU paths
```

Ordinary call application after target selection may of course reuse call checking, but target discovery remains associated-only.

---

# 35. Performance and Representation Requirements

Part 3 is semantic code, but its data model must not force expensive Part 4 representation.

Requirements:

1. static direct invocation resolves to one compact exact semantic target;
2. family type interning is hash-consed/canonical rather than copied structurally at every expression;
3. `AssociatedResolution` uses compact `TypeId`/identity references rather than AST/range duplication for semantic identity;
4. family member ordering is canonical and suitable for deterministic fingerprints;
5. direct known calls do not semantically require family object allocation;
6. exact case types remain compact `VariantTypeId + enum TypeId` as specified in Part 2;
7. generics remain compile-time substitutions/evidence unless Part 4 independently chooses runtime reification/specialization;
8. no Part 3 structure requires one runtime code body per generic specialization.

---

# 36. Rejected Alternatives

## 36.1 Dynamic `::` dispatch

Rejected. `::` is statically resolved; `.` remains ordinary runtime message dispatch.

## 36.2 Live `::*` family view

Rejected for ordinary family values. Future provider/rebindable capability semantics are deferred to a separate explicit mechanism.

## 36.3 Family as union of callables

Rejected because it means one alternative rather than one value supporting many operations.

## 36.4 Getter equals zero-argument method

Rejected. Selector kind remains part of operation identity.

## 36.5 Bare associated name always means family

Rejected. `owner::name` is exact getter `#name`; whole family is explicitly `owner::name::*`.

## 36.6 Try behavioral lookup then variant lookup

Rejected. Part 2 family category/reservation determines the namespace; ambiguity here is an invariant failure.

## 36.7 Variant constructors as class-side methods

Rejected. They are first-class executable declarations with `VariantConstructorId`.

## 36.8 Runtime class-object dispatch as behavioral associated resolution

Rejected. Static inherited behavior may be composed from source hierarchy, but `::` does not execute ordinary class-object lookup and does not enter `Class` instance behavior.

## 36.9 First-use specialization of stored generic family values

Rejected because it makes type meaning order-dependent.

## 36.10 Default unresolved generics to Dynamic/Object

Rejected because it erases proof precision and hides underconstraint.

## 36.11 General `forall`/HM generalization as an implicit Part 3 side effect

Rejected as an *implicit* side effect. Part 3 preserves residual declaration binders. Decision Gate G1 may explicitly ratify **declaration-provided rank-1 schemes** for associated declaration values; that narrower feature must not be conflated with general HM/`forall` generalization of arbitrary expressions.

---

# 37. Decision Register

| Decision | Status in Part 3 |
|---|---|
| `::` static semantics | Ratified / normative |
| `.` ordinary dispatch only | Ratified / normative |
| retain `>>`, revisit later | Ratified / outside scope |
| future provider/live capability | Deferred |
| effective inherited class-side behavior visible to `::` | Normative Part 3 amendment/clarification |
| variants/direct associated declarations inherit | Rejected |
| family is heterogeneous exact member set | Normative Part 3 model |
| family type encoded as union | Rejected |
| family structural type + captured nominal/capability denotation | Normative Part 3 model |
| getter participates as zeroarg Method invocation | Rejected |
| variant singleton lookup returns value | Normative |
| behavioral getter lookup reifies callable | Normative |
| exact known calls bypass family reification | Normative |
| dynamic pack may route within frozen family | Supported semantic model; Part 4 executes |
| generic owner must be kind `Type` | Rejected; declaration-backed arrow-kind forms valid |
| bare direct generic invocation infers from args/context | Required |
| escaped bare generic associated value/family | **Decision Gate G1:** choose contextual-only v1 or declaration-provided rank-1 schemes before implementation commits |
| first-use generic specialization | Rejected |
| unresolved generic becomes Dynamic/Object | Rejected |
| Part 4 re-resolves associated AST | Rejected |

---

# 38. Acceptance Criteria

Part 3 is accepted only when all of these statements are true in code and tests.

1. `phalcom-semantic` analyzes every Part 1 associated AST form.
2. `::` target discovery never falls back to ordinary message dispatch.
3. declaration-backed unsaturated/partial type forms can own associated lookup.
4. runtime values without `TypeForm` denotation cannot use associated lookup merely from runtime class type.
5. inherited class-side behavioral members compose into a static effective family with deterministic selector shadowing.
5a. inherited generic behavioral signatures are specialized through canonical generic-supertype projection to their defining owner while `Self` remains lookup-owner-relative.
6. direct associated declarations/variants do not inherit.
7. `#name` and `#name()` remain distinct through lookup, family typing, and invocation.
8. singleton `Option::None` and constructor `Option::None()` produce distinct exact case identities/types.
9. variant constructors remain non-method semantic identities.
10. `Owner::family::*` produces a canonical structural family type plus captured denotation retaining family identity, lookup-owner specialization, and access-filtered exact member bindings.
11. family type is not represented as a union.
12. direct invocation and call-on-family share canonical family selection/application logic.
13. selected executable members reuse the existing call binder/generic inference engine.
13a. the canonical call projection/binder preserves positional/labeled/complete rest lanes so exact-before-rest family selection is semantically provable.
14. known direct calls can be represented as exact target resolutions without materialized family values.
15. explicit owner generic arguments, call arguments, expected result, and GADT equalities compose into one specialization result.
16. incompatible GADT owner specialization is diagnosed.
17. unresolved generic variables are never silently widened.
17a. reified exact members/families preserve lookup-owner specialization and access-filtered captured targets through local binding flow; nominal family ID alone is not treated as sufficient capability provenance.
18. source occurrences attach exact compiler-owned associated targets.
19. associated lookup dependencies participate in incremental invalidation.
20. `AssociatedResolution` gives Part 4 sufficient information to lower without semantic re-resolution.
21. legacy `MakeFamily`, VM dNU, runtime class dictionary behavior, and `>>` are not used to define new `::` semantics.
22. no runtime representation choice is accidentally fixed by Part 3 semantic structures.

---

# 39. Final Architectural Summary

Part 3 should leave the compiler with the following semantic pipeline:

```text
TYPE FORM / KIND SYSTEM
    Option : Type -> Type
       │
       ▼
ASSOCIATED OWNER RESOLUTION
    DeclarationId(Option)
    supplied generic arguments
       │
       ▼
PART 2 ASSOCIATED DECLARATION PRODUCTS
    AssociatedFamilyId(Option, Some)
    exact member identities/signatures
       │
       ▼
STATIC EFFECTIVE FAMILY
    direct variant family
    or statically inherited class-side behavioral family
       │
       ├──────── exact lookup ────────→ singleton value / exact callable
       │
       ├──────── whole `::*` ─────────→ structural Family Type + captured family denotation
       │
       └──────── invocation ──────────→ Method-shape member selection
                                              │
                                              ▼
                                 GENERIC/GADT SPECIALIZATION
                                              │
                                              ▼
                                  CANONICAL CALL APPLICATION
                                              │
                                              ▼
                                  PRECISE EXACT RESULT TYPE
                                              │
                                              ▼
                                  AssociatedResolution
                                              │
                                              ▼
                                       PART 4 LOWERING
```

This preserves Phalcom's object/message model and functional first-class callable model simultaneously: `.` owns dynamic behavior, `::` owns static associated identity, families are stable structural capabilities, and variants remain exact data constructors rather than fake methods.
