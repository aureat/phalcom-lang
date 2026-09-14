---
id: LANG005.C3.P1
category: LANG
program: LANG005
checkpoint: LANG005.C3
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG005.C2.P3
blocked_by:
  - LANG005.C2.P3
follows: LANG005.C2.P3
supersedes: null
prepared: 2026-09-14
repository: aureat/phalcom-lang
repository_baseline: 986568da050d1bbfa7f1d769c9e1f4d58eb81cf3
audit_revision: 2026-09-14
---

# LANG005.C3.P1 — First-Class Trait Declarations and Abstract Trait Surfaces

## Audited Patch-Grade Implementation Plan

> **Primary executor:** Phalcom Luna-class implementer model.  
> **Execution rule:** this plan is intentionally over-specified at semantic boundaries and under-specified at private mechanical boundaries. Reuse the repository's canonical identities and semantic owners wherever they already express the required fact. Do not create duplicate trait-specific infrastructure merely because a trait is a new language category.

---

# 0. Plan Status and Hard Predecessor Gate

This plan is ready for implementation **only after** `LANG005.C2.P3` is implemented and its actual landed interfaces are verified.

Current audited repository state:

```text
repository:
    aureat/phalcom-lang

main:
    986568da050d1bbfa7f1d769c9e1f4d58eb81cf3

C2.P1:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

C2.P2:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

C2.P3:
    PLANNED / NOT_STARTED / UNVERIFIED

C3:
    BLOCKED / NOT_STARTED / UNVERIFIED
```

Do **not** implement C3 on the audited baseline before C2.P3 lands.

The first C3 execution step is:

```text
G0
    verify C2.P3 takeover
    import actual P3 symbol map
    update C3 checkpoint baseline
    reconcile any live drift
```

No trait source code should be edited before G0 passes.

---

# 1. Goal

Implement first-class Phalcom `trait` declarations as reusable, non-storage behavioral contracts with:

```text
dedicated trait declaration syntax
canonical declaration identity
declaration-owned generic trait signatures
TraitRef
TraitRequirementId
TraitSurface
bodyless requirements
bodyful defaults
abstract owner-relative Self
one-time default body analysis
incremental/query/source-index integration
compiler/runtime non-class semantics
normative protocol→trait specification migration
```

Target language shape:

```phalcom
trait Equatable<T> {
  equals(_ other: T) -> Bool

  notEquals(_ other: T) -> Bool {
    !equals(other)
  }
}
```

Target semantic shape:

```text
DeclarationId("Equatable")
DeclarationKind::Trait

TraitInfo
    declaration
    GenericSignature<T>

TraitSurface
    requirement equals(_)
        TraitRequirementId
        trait-owned CallableId
        CallableSemanticSignature
        default = none

    requirement notEquals(_)
        TraitRequirementId
        trait-owned CallableId
        CallableSemanticSignature
        default = body attached to same trait-owned CallableId
```

The trait has no owned instance representation and does not create an ordinary runtime class.

---

# 2. Companion Authorities

Implementation must read and use:

```text
LANG005.C3-CHECKPOINT.md
LANG005.C3-GUIDANCE.md
LANG005.C3.P1-requirements-analysis.md
this plan
```

The checkpoint amendment ledger records repository-audit corrections that override stale placeholder shapes in any earlier copy of the plan.

Current adopted corrections include:

```text
C3-A01
    lifecycle/checkpoint normalization

C3-A02
    minimize new identity types

C3-A03
    reuse existing SemanticTargetId declaration/callable targets

C3-A04
    normalize shared IndexMethodDef body representation

C3-A05
    expand protocol-era spec migration audit

C3-A06
    classify LANG005 support files by owning checkpoint

C3-A07
    use repository A/B/C/D failure taxonomy
```

This audited plan incorporates those corrections directly.

---

# 3. Checkpoint Acceptance Objective

C3.P1 is accepted only when all of the following are true.

1. `trait` is a dedicated top-level declaration, not `@protocol class` sugar.
2. Module/interface declaration kind is canonically `Trait`.
3. Trait declarations reuse the existing `DeclarationId` universe.
4. Trait generics are declaration-owned but are **not** forced into nominal `DeclarationTypeTable` entries.
5. `TraitRef` exists as a contract reference over a trait declaration plus generic arguments.
6. `TraitRef` is not automatically an inhabitable `TypeId`.
7. Trait methods/getters/setters/index members reuse the shared behavior-member grammar.
8. Shared `IndexMethodDef` is normalized so declaration-only index requirements are representable without a trait-only AST.
9. Every trait behavioral member has a `TraitRequirementId`.
10. Trait member source/default bodies reuse the canonical trait-owned `CallableId`.
11. No mandatory `TraitId`, `TraitMemberId`, or `TraitDefaultId` is introduced unless live implementation proves a genuinely missing semantic dimension.
12. Bodyless trait members are requirements.
13. Bodyful trait members are requirements with defaults.
14. Trait signatures reuse canonical callable signature formation.
15. Trait generic callable signatures are formed with explicit trait generic resolver bindings rather than fake nominal trait types.
16. Defaults are analyzed exactly once.
17. Default body analysis receives trait-owned generic bindings explicitly; it does not depend solely on `DeclarationTypeTable`.
18. Trait `Self` reuses the canonical owner-relative `SelfTypeTerm`.
19. Member lookup on trait `Self` consults `TraitSurface`.
20. Calls through abstract trait `Self` reuse the canonical call argument/generic checker.
21. Abstract requirement calls have no executable runtime `InvocationTargetId`.
22. Calls from one default to another trait member remain contract-relative rather than hard-binding to the default body.
23. Trait requirements/defaults are never inserted into `DeclarationSurface`.
24. Trait requirements/defaults are never inserted into C2 conditional inherent surfaces.
25. Enum closed requirements remain enum-specific.
26. No concrete conformance or witness selection occurs.
27. No associated type support is implemented.
28. No generic trait-conformance constraints are implemented.
29. No final metatype/class-object conformance syntax is ratified.
30. Source indexing reuses canonical `SemanticTargetId::Declaration` / `::Callable` unless a proven source identity gap exists.
31. Trait surface/default-body invalidation is fine-grained.
32. Cold and incremental analysis agree.
33. Compiler accepts trait-containing source without creating an ordinary runtime class.
34. Trait defaults are not installed into target runtime method tables.
35. Ordinary VM dispatch does not scan traits.
36. Active protocol-era normative documents are reconciled so one effective trait model remains.
37. Focused verification passes.
38. C3 walkthrough and C4 handoff are produced.
39. `LANG005.C3-CHECKPOINT.md` is updated to `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

---

# 4. Repository Grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 986568da050d1bbfa7f1d769c9e1f4d58eb81cf3
```

## 4.1 Verified live facts

### Behavior syntax

The repository already has:

```rust
pub enum BehaviorMember {
    Method(...),
    Getter(...),
    Setter(...),
    Index(...),
}
```

C3 should reuse this shared behavior-only AST category.

### Callable identity

The repository already has canonical callable identity:

```rust
CallableId {
    owner: CallableOwnerId,
    selector,
    side,
}
```

and `CallableOwnerId::Declaration(DeclarationId)` is sufficient for a trait-declared source callable/default.

### Requirement identity precedent

C2 already has:

```rust
EnumRequirementId {
    owner: DeclarationId,
    selector: Selector,
}
```

which is distinct from implementation `CallableId`.

C3 should follow this semantic pattern for reusable trait requirements.

### Declaration identity

`DeclarationId` is already the named declaration identity.

Trait does not need a second declaration identity merely because its category is new.

### Declaration kind migration seam

Current module declaration kind includes:

```text
Class
Protocol
Adt
Data
Alias
```

No production `DeclarationKind::Protocol` use was found in the audit.

The expected migration seam is:

```text
Protocol
    ↓
Trait
```

after a final compatibility check.

### Source target identity

The semantic source target model already has:

```text
SemanticTargetId::Declaration(DeclarationId)
SemanticTargetId::Callable(CallableId)
```

C3 should reuse them.

### Type-level lexical binding domain

Current:

```rust
pub enum TypeLevelBinding {
    TypeForm(TypeId),
    RecordRow(TypeParameterId),
}
```

This is a lexical generic binder domain.

It is **not** the named declaration namespace.

Do not add `TypeLevelBinding::Trait`.

### Nominal declaration type table

Current `DeclarationTypeTable` stores:

```text
DeclarationId
nominal TypeId form
class-object TypeId
KindId
GenericSignature
supertype template
```

`NominalDeclarationHeader::from_signature` creates nominal forms and class-object types.

A C3 trait intentionally has neither an ordinary nominal value type nor a class-object runtime type.

Do not insert traits into `DeclarationTypeTable` merely to reuse generic signature storage.

### Canonical signature builder seam

Current signature lowering already exposes:

```text
semantic_signature_for_syntax_with_resolver(...)
```

This is the correct seam for trait member signature formation.

### Callable body analyzer assumption

The current body analyzer obtains declaration generics from `DeclarationTypeTable`.

That assumption must be generalized narrowly for trait defaults.

Do not satisfy it by fabricating a nominal trait table entry.

### Current call target

Current ordinary exact dispatch constructs:

```text
CallableId
+
InvocationTargetId::Behavioral(callable)
```

An abstract trait requirement call must not manufacture that executable target.

### Index member gap

Current `IndexMethodDef` still stores an executable statement body and parser uses `parse_method_block()`.

`CallableSyntaxRef::has_body()` treats every index member as bodyful.

The LANG005 support surface explicitly contains bodyless index getter/setter requirements.

Therefore C3 must normalize shared index member body representation.

---

# 5. Required Reads Before Editing

## Governance

```text
AGENTS.md
docs/spec/README.md
docs/implementation/README.md
docs/workflow/implementation-record-lifecycle-convention.md
```

## C3 state

```text
LANG005.C3-CHECKPOINT.md
LANG005.C3-GUIDANCE.md
LANG005.C3.P1-requirements-analysis.md
this plan
```

## C2 takeover

After P3 implementation:

```text
LANG005.C2-CHECKPOINT.md
LANG005.C2.P3 walkthrough
LANG005.C2.P3 handoff
```

## Trait/protocol authority cluster

Audit at minimum:

```text
docs/spec/typing/README.md
docs/spec/typing/STATUS.md
docs/spec/typing/01-protocol-foundation.md
docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md
docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md
```

Then search active `docs/spec/` for:

```text
@protocol class
Protocol descriptor
signature-only protocol
structural protocol conformance
protocol default
protocol class-side
```

Do not globally replace ordinary English “protocol”.

## LANG005 support examples

Read:

```text
docs/implementation/LANG005/support/comparable.ph
docs/implementation/LANG005/support/sized.ph
docs/implementation/LANG005/support/indexable.ph
docs/implementation/LANG005/support/iterable.ph
docs/implementation/LANG005/support/core_trait_impls.ph
docs/implementation/LANG005/support/metatype_conformance_example.ph
```

Use `comparable.ph` / `sized.ph` directly as C3 design references.

Use other files only for forward-compatibility review; they include C4/C5/C6 or provisional syntax.

---

# 6. Architecture

## 6.1 Semantic flow

```text
source trait declaration
        ↓
DeclarationId + DeclarationKind::Trait
        ↓
TraitInfo
    generic signature
        ↓
TraitSurface
    ├─ TraitRequirementId
    ├─ trait-owned CallableId
    ├─ CallableSemanticSignature
    ├─ visibility
    └─ optional default body/source
            ↓
      CallableAnalysis
      under abstract Self
```

## 6.2 Future flow, deliberately not implemented

```text
Target type
+ TraitRef
+ TraitSurface
+ effective target behavior
        ↓
C4 conformance
        ↓
witness/default mapping
```

C3 publishes the left-hand products only.

---

# 7. Identity Model

## 7.1 Trait declaration

Canonical identity:

```text
DeclarationId
+ declaration kind = Trait
```

No mandatory `TraitId`.

A validated wrapper is mechanically permitted if it prevents invalid API use, but it must remain one-to-one with `DeclarationId` and must not create a second identity universe.

## 7.2 Trait requirement

Required new semantic identity:

```rust
pub struct TraitRequirementId {
    pub owner: DeclarationId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

Exact field/naming mechanics are flexible.

Identity law:

```text
TraitRequirementId
    != trait-owned source CallableId
    != future target witness CallableId
```

## 7.3 Trait source callable/default

Reuse:

```rust
CallableId {
    owner: CallableOwnerId::Declaration(trait_declaration),
    selector,
    side,
}
```

This is the canonical source callable identity for:

```text
signature
member-local generics
default body
source navigation
fingerprinting
callable analysis
```

No mandatory `TraitMemberId`.

No mandatory `TraitDefaultId`.

## 7.4 Generic ownership

Trait declaration generics:

```text
TypeParameterOwner::Declaration(trait DeclarationId)
```

Trait member-local generics:

```text
TypeParameterOwner::Callable(trait-owned CallableId)
```

Do not add `TypeParameterOwner::Trait`.

---

# 8. Trait Generic Metadata Must Not Become Nominal Type Metadata

This is fixed architecture.

A trait declaration must have a semantic product such as:

```rust
pub struct TraitInfo {
    pub declaration: DeclarationId,
    pub generic_signature: Option<GenericSignature>,
    pub source: Option<SemanticSourceSpan>,
}
```

Do not insert a C3 trait into nominal `DeclarationTypeTable`.

Do not call `NominalDeclarationHeader::from_signature` for a trait.

Do not create:

```text
trait nominal TypeId
trait class-object TypeId
trait superclass template
```

merely to preserve generic information.

Trait generic signature lookup should come from the trait product/query.

---

# 9. TraitRef

Required concept:

```rust
pub struct TraitRef {
    pub declaration: DeclarationId,
    pub arguments: Box<[TypeId]>,
}
```

The declaration must be verified as `DeclarationKind::Trait`.

`TraitRef` means:

```text
this trait contract instantiated by these type arguments
```

It does not mean:

```text
a runtime trait object
a nominal value type
conformance evidence
a witness table
```

## TraitRef formation

Resolve source reference:

```text
source static type-like path
    ↓
DeclarationId
    ↓
verify DeclarationKind::Trait
    ↓
read trait GenericSignature
    ↓
validate arity/kinds/existing ordinary constraints
    ↓
TraitRef
```

Do not route named traits through `TypeLevelBinding`.

`TypeLevelBinding` remains lexical binder infrastructure.

---

# 10. TraitSurface

Canonical concept:

```rust
pub struct TraitSurface {
    pub declaration: DeclarationId,
    pub generic_signature: Option<GenericSignature>,
    pub members: ...,
    pub diagnostics: ...,
}
```

Each member must expose repository-equivalent facts:

```text
TraitRequirementId
trait-owned CallableId
CallableSemanticSignature
visibility
source
default-present?
```

Do not store body analysis inside the structural surface product.

Do not copy the surface into ordinary declaration dispatch.

---

# 11. Body Classification

Fixed law:

```text
MemberBody::Declaration
    -> requirement only

MemberBody::Block(...)
    -> requirement + default
```

For a bodyful trait member:

```text
requirement identity
    TraitRequirementId

default/body identity
    trait-owned CallableId
```

Bodyless ↔ bodyful transition with same selector/side:

```text
TraitRequirementId
    unchanged

CallableId
    unchanged

TraitSurface default availability
    changes

Callable body product
    appears/disappears
```

---

# 12. Shared Index Body Normalization

This is required C3 work.

## 12.1 Current mismatch

Current:

```text
MethodDef.body
    MemberBody

GetterDef.body
    MemberBody

SetterDef.body
    MemberBody

IndexMethodDef.body
    statement vector
```

Current parser:

```text
parse_index_member(...)
    -> parse_method_block()
```

Current signature helper:

```text
CallableSyntaxRef::Index(_)
    has_body() == true
```

Current consumers assume bodyful index accessors.

## 12.2 Required target

Normalize:

```rust
IndexMethodDef {
    ...
    body: MemberBody,
}
```

or exact repository-equivalent representation.

## 12.3 Required audit surface

Before editing, search every `IndexMethodDef.body` consumer.

At minimum the audit already found:

```text
phalcom-ast parser
phalcom-semantic checker/declaration_signature
phalcom-semantic session callable body extraction
phalcom-semantic semantic_shard
phalcom-semantic db/fingerprint
phalcom-semantic source_index builder/occurrence
phalcom-core compiler class declaration
phalcom-core semantic lowering
phalcom-core product optimizer
compiler attributes / AST-generated index members
tests
```

Update all mechanically.

## 12.4 Parser behavior

Trait context must accept bodyless:

```phalcom
trait IntIndexable {
  [_ index: Int] -> Int
}
```

and bodyless setter form using the canonical existing setter signature semantics.

Do **not** change index-setter return semantics in C3 merely because a support fixture shows `Unit`.

The C3-owned change is:

```text
declaration-only index contract support
```

not:

```text
redesign subscript setter return semantics
```

If setter return semantics need a separate language correction, stop and scope it independently.

## 12.5 Non-trait behavior

Existing bodyful class/impl index members must continue working.

If ordinary classes/impls do not permit bodyless index declarations, reject them semantically/parser-contextually rather than globally allowing abstract bodies everywhere.

## 12.6 Fingerprint behavior

After normalization:

```text
structural signature fingerprint
    must distinguish declaration/body presence only where semantically relevant

body fingerprint
    hashes block content only when body exists
```

Bodyless index requirements must behave like bodyless methods/getters/setters in incremental products.

---

# 13. Trait Signature Formation

Reuse canonical source-to-semantic callable signature machinery.

Use:

```text
callable_id_for_syntax(...)
semantic_signature_for_syntax_with_resolver(...)
```

or live equivalents.

## Trait generic resolver

Build trait generic bindings from `TraitInfo.generic_signature`.

Conceptually:

```text
TraitInfo.generic_signature
    ↓
TypeParameterId list
    ↓
type_level_binding_for_parameter(...)
    ↓
ScopedTypeResolver
    ↓
semantic_signature_for_syntax_with_resolver(...)
```

This is correct because `TypeLevelBinding` is being used for **lexical trait parameters**, not named trait declarations.

## Do not

```text
insert trait into DeclarationTypeTable
construct trait nominal form
construct trait class-object type
modify TypeResolver to pretend trait is an ordinary proper type
```

---

# 14. Callable Body Analysis Generalization

Current body analysis assumes owner generics come from nominal `DeclarationTypeTable`.

C3 must generalize this narrowly.

Preferred direction:

```rust
CallableBodyRequest {
    ...
    owner_generic_signature: Option<&GenericSignature>,
}
```

or:

```rust
BodyAnalysisContext {
    ...
    owner_type_parameters: ...
}
```

or repository-equivalent.

Rules:

1. Existing class/enum/data behavior may continue to use `DeclarationTypeTable`.
2. Trait default analysis passes `TraitInfo.generic_signature` explicitly.
3. Callable-local generics still come from the canonical declared signature.
4. Trait defaults must not be made nominal solely to feed owner generic bindings.
5. The generalized body analyzer must remain usable by existing body analysis callers without semantic changes.

Add one focused regression proving a generic trait default can resolve its trait-owned `T` inside the body.

---

# 15. Abstract Self

Reuse the existing owner-relative:

```text
SelfTypeTerm {
    owner: trait DeclarationId,
    side: Instance,
    role: InstanceType
}
```

No `TraitSelfType`.

No synthetic conformer.

No fake class.

Trait default body context:

```text
current declaration
    = trait DeclarationId

current callable
    = trait-owned CallableId

current side
    = Instance

Self
    = owner-relative SelfTypeTerm

trait generic environment
    = TraitInfo generic signature

member generic environment
    = CallableSemanticSignature generics

fields
    = none

superclass
    = none

conformance evidence
    = none
```

---

# 16. Trait-Self Member Lookup

Do not register `TraitSurface` into `SurfaceDispatchResolver`.

Ordinary dispatch is backed by concrete declaration surfaces and receiver type registrations.

Trait default checking needs a separate abstract lookup path.

Preferred conceptual API:

```text
resolve_trait_self_member(
    trait_declaration,
    selector,
    side
)
    -> trait requirement/signature
```

or a small dispatch facade that can distinguish:

```text
concrete nominal dispatch
trait-contract abstract dispatch
```

The exact private shape is flexible.

Fixed semantic rule:

```text
abstract trait Self lookup
    reads TraitSurface
```

not:

```text
DeclarationSurface
class hierarchy
future conformer
runtime method table
```

---

# 17. Abstract Trait Calls Must Not Manufacture Runtime Invocation Targets

This is a fixed audited correction.

Current exact callable target creates:

```text
target = InvocationTargetId::Behavioral(callable)
```

That is correct for executable concrete behavior.

A trait requirement call inside a default is abstract contract application.

It must preserve:

```text
trait-owned CallableId
canonical CallableSignature
generic argument checking
argument binding
expected-result checking
diagnostic/explanation ownership
```

but must **not** claim an executable runtime behavioral target exists.

Preferred semantic result:

```text
CallableApplicationTarget
    callable = Some(trait-owned CallableId)
    target = None
    authority = TraitContract / AbstractRequirement
```

The exact authority enum name is flexible.

Do not use:

```text
CallTargetAuthority::ExactDispatch
+
InvocationTargetId::Behavioral(trait-owned callable)
```

for an abstract requirement.

Do not make compiler lowering responsible for discovering that the call is abstract.

C3 default body analysis is semantic only; C4 later supplies executable witness/default selection.

---

# 18. Default-to-Default Calls

Even when the selected trait member itself has a default body:

```phalcom
trait T {
  a { b() }
  b { ... }
}
```

the call to `b()` inside `a` resolves against:

```text
TraitRequirementId(b)
+
contract signature
```

not directly to `b`'s default body.

Why:

```text
a future conformance may provide a concrete witness for b
```

The default is fallback implementation metadata, not static call binding.

---

# 19. Trait Storage and Member Legality

Trait bodies may contain behavioral members only.

Reject:

```text
fields
stored components
constructors
class storage declarations
class invariants
enum variants
nested inherent impl declarations
```

Getter/setter/index requirements are capabilities.

They do not inject storage.

C4 may later decide what concrete member can witness them.

---

# 20. Class-Side / Metatype Boundary

C3.P1 is instance-contract-first.

Do not ratify final class-object trait conformance syntax.

If a class-side marker is encountered in a trait member:

```text
diagnose/defer
```

unless a newer accepted decision explicitly supersedes this plan.

The support file form:

```phalcom
impl Parser for class Person
```

is provisional future syntax and not C3 authority.

---

# 21. Associated Types Boundary

Associated types belong to C5.

Do not implement:

```phalcom
type Item
type Cursor
type Output
```

inside traits.

Do not encode them as generic parameters.

Do not create projections.

Do not let the `iterable.ph` / `indexable.ph` support fixtures pull C5 into C3.

Use concrete type arguments in C3 tests.

---

# 22. Generic Trait Constraints Boundary

Trait-conformance constraints belong to C6.

Do not add:

```text
GenericConstraint::Conforms
T: Trait
T conforms Trait
```

as new semantics in C3.

Existing ordinary generic constraints remain available.

Example:

```phalcom
trait NumericView<T>
where T <: Number
{
  value -> T
}
```

may use already-existing subtype constraints.

---

# 23. TraitRef Namespace Resolution

Named trait references must use declaration lookup.

Do not extend lexical `TypeLevelBinding`.

Conceptually:

```text
resolve static symbol
    ↓
DeclarationId
    ↓
declaration shell kind
    ↓
Trait?
    yes -> form TraitRef
    no  -> wrong declaration category diagnostic
```

This keeps:

```text
named declaration namespace
```

separate from:

```text
lexical generic binder namespace
```

---

# 24. Module Interface Integration

Add `Statement::Trait` to module interface collection exactly as other named declarations participate.

Trait declarations:

```text
create module namespace bindings
can collide with class/data/enum/alias names
can be imported/exported
```

Inherent `impl` remains non-binding.

No runtime class semantics should be inferred from the module interface fact that a trait is a named declaration.

---

# 25. DeclarationKind Migration

Expected:

```rust
pub enum DeclarationKind {
    Class,
    Trait,
    Adt,
    Data,
    Alias,
}
```

Before changing `Protocol` to `Trait`, search:

```text
serialization
metadata schemas
cache formats
external plugin APIs
generated/native metadata
```

The audit found no production use, so rename/replace is the default.

If a stable compatibility dependency is found:

```text
STOP AND CONSULT
```

Do not keep both `Protocol` and `Trait` as semantic categories merely to avoid a rename.

---

# 26. Source Index and LSP

Use canonical targets:

```text
trait declaration
    SemanticTargetId::Declaration(DeclarationId)

trait member/default source
    SemanticTargetId::Callable(CallableId)
```

`TraitRequirementId` remains semantic contract identity and does not need to become a separate navigation target.

LSP may map trait declarations to an Interface-like presentation kind.

Do not introduce LSP-local semantic identity.

Do not reconstruct trait surfaces in the LSP.

---

# 27. Incremental Product Model

At minimum separate:

```text
TraitInfo/header
TraitSurface
Callable body/default analysis
```

Recommended query intent:

```text
TraitInfo(DeclarationId)
TraitSurface(DeclarationId)
CallableBody(trait-owned CallableId)
```

Exact `QueryKey` spelling is flexible.

## Header edit

Changes:

```text
trait generic signature
TraitInfo
TraitSurface
dependent TraitRef formation
```

## Member signature edit

Changes:

```text
CallableSemanticSignature
TraitSurface
future dependents
```

## Selector edit

Changes:

```text
trait-owned CallableId
TraitRequirementId
TraitSurface
```

## Default body-only edit

Changes:

```text
CallableBody / CallableAnalysis
body fingerprint
```

Does not change:

```text
DeclarationId
TraitRequirementId
trait-owned CallableId
TraitSurface signature facts
unrelated members
```

## Bodyless ↔ bodyful

Changes:

```text
default availability
TraitSurface
CallableBody existence
```

Preserves:

```text
TraitRequirementId
trait-owned CallableId
```

---

# 28. Fingerprint Requirements

Trait structural/header fingerprint should include only semantic structure such as:

```text
trait declaration identity
generic header
member selectors
member signatures
visibility
default-present flag/source identity
```

Do not hash full default statement content into the TraitSurface fingerprint.

Callable body fingerprint owns statement content.

Index normalization must follow the same separation.

---

# 29. Compiler / Runtime Boundary

Preferred compiler handling:

```text
Statement::Trait
    compile-time/type-level declaration
    emit no ordinary runtime class allocation
```

Do not add:

```text
ClassId per trait
class-object instance for trait semantics
superclass edge
field slots
FinalizeClass
trait method installation on target classes
runtime trait scan during Invoke
conformance registry
witness table
trait vtable
```

Type aliases provide a useful compile-time-only precedent, but verify module/export requirements rather than blindly copying the branch.

If runtime export/linkage genuinely requires a trait descriptor:

```text
STOP AND CONSULT
```

Do not solve it with a fake class.

---

# 30. Documentation Migration

C3 must leave one effective trait model.

Audit and reconcile:

```text
docs/spec/typing/README.md
docs/spec/typing/STATUS.md
docs/spec/typing/01-protocol-foundation.md
docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md
docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md
```

Search active specs for:

```text
@protocol class
Protocol descriptor
signature-only protocol
structural protocol conformance
protocol default semantics
class-side protocol requirements
```

Do not globally replace unrelated uses of “protocol”.

Archive or mark superseded historical language-feature material according to spec governance.

---

# 31. LANG005 Support-File Ownership

## Direct C3 shape references

```text
comparable.ph
sized.ph
```

Use for:

```text
generic trait parameters
bodyless requirements
bodyful defaults
getter requirements
default calls through self
```

## Partial C3 relevance

```text
indexable.ph
```

Use only for:

```text
bodyless index requirement AST/surface shape
```

Do not implement its associated types in C3.

## Later-checkpoint references

```text
iterable.ph
core_trait_impls.ph
metatype_conformance_example.ph
```

These contain C4/C5/C6 or provisional syntax.

Use only for forward-compatibility review.

---

# 32. Diagnostics

Add stable repository-conventional diagnostics for at least:

```text
trait duplicate selector/side
trait storage/field forbidden
trait constructor forbidden
trait invariant forbidden
trait class-side member deferred/unsupported
trait associated type deferred/unsupported
trait used as ordinary value type if not supported
trait reference wrong declaration kind
trait reference arity mismatch
trait reference kind mismatch
trait default unknown abstract member
trait default illegal field access
trait default illegal super
trait default return mismatch
legacy @protocol class migration/unsupported path if retained
```

Do not defer these errors to compiler/runtime when parser/semantic layers own them.

---

# 33. Global Invariants

`INV-01` — Trait is a distinct declaration kind, not class sugar.

`INV-02` — Trait declaration identity stays in `DeclarationId`.

`INV-03` — No mandatory parallel `TraitId` universe.

`INV-04` — Trait requirements have `TraitRequirementId`.

`INV-05` — Trait member/default source identity reuses `CallableId`.

`INV-06` — No mandatory `TraitMemberId`.

`INV-07` — No mandatory `TraitDefaultId`.

`INV-08` — `TraitRequirementId != future target witness CallableId`.

`INV-09` — Trait generics are declaration-owned.

`INV-10` — Trait generic metadata is not stored by fabricating a nominal trait entry in `DeclarationTypeTable`.

`INV-11` — Named trait declarations are not represented by `TypeLevelBinding`.

`INV-12` — `TraitRef` is contract identity, not ordinary value `TypeId`.

`INV-13` — `TraitSurface != DeclarationSurface`.

`INV-14` — `TraitSurface != conditional inherent surface`.

`INV-15` — `TraitRequirementId != EnumRequirementId`.

`INV-16` — Every trait behavioral member is a requirement.

`INV-17` — Bodyful member = same requirement + default.

`INV-18` — Shared `IndexMethodDef` supports declaration-only bodies after normalization.

`INV-19` — C3 does not redesign index setter return semantics.

`INV-20` — Complete TraitSurface exists before default bodies are checked.

`INV-21` — Abstract Self reuses `SelfTypeTerm`.

`INV-22` — Trait Self lookup reads TraitSurface.

`INV-23` — Abstract trait call targets have no runtime invocation target.

`INV-24` — Default-to-default calls remain contract-relative.

`INV-25` — Defaults are checked once.

`INV-26` — Traits own no instance storage/layout/superclass.

`INV-27` — No conformance is created in C3.

`INV-28` — No associated types are implemented in C3.

`INV-29` — No trait-conformance generic constraint is implemented in C3.

`INV-30` — No final metatype-conformance syntax is invented.

`INV-31` — Source indexing reuses canonical Declaration/Callable targets.

`INV-32` — Default body-only edits do not change requirement/callable identity.

`INV-33` — Cold and incremental results agree.

`INV-34` — Compiler creates no ordinary runtime trait class.

`INV-35` — VM ordinary send does not scan traits.

---

# 34. Implementer Decision Authority

## FIXED

Luna may not change:

```text
identity reuse model
TraitRequirementId distinction
no nominal trait type-table entry
no TypeLevelBinding::Trait
TraitSurface separation
shared index MemberBody normalization
abstract Self semantics
semantic-only abstract call target
no conformance
no associated types
no conformance constraints
no runtime trait class
```

## MECHANICALLY FLEXIBLE

Luna may adapt:

```text
private module/file names
helper names
map/set types
wrapper convenience types
query key spelling
diagnostic code spelling
test file split
LSP presentation kind
```

## VERIFY-FIRST

Inspect live tree for:

```text
C2.P3 final symbols
DeclarationKind compatibility
all IndexMethodDef.body consumers
trait parser placement
TraitInfo storage/query location
body analyzer extension seam
call target authority enum extension
module runtime binding behavior
spec archive location
```

Mechanical drift: adapt.

Architectural drift: consult.

---

# 35. STOP / CONSULT Triggers

Stop if:

1. C2.P3 is incomplete.
2. C2.P3 changes the final behavioral-surface ownership model materially.
3. Shared `BehaviorMember` cannot support trait members.
4. Shared index body normalization cannot be made without breaking established class/impl semantics.
5. Trait parsing requires reusing `ClassDef` as semantic authority.
6. Trait generic metadata can only be stored by creating a nominal trait type/class object.
7. TraitRef can only work by becoming an ordinary value TypeId.
8. Named traits appear to require extending lexical `TypeLevelBinding`.
9. Default checking requires concrete conformance evidence.
10. Abstract trait calls can only reuse the call checker by manufacturing runtime `InvocationTargetId::Behavioral`.
11. TraitSurface must be inserted into ordinary declaration dispatch.
12. VM ordinary dispatch must scan TraitSurface.
13. Associated types become necessary to complete the P1 acceptance objective.
14. Generic trait-conformance constraints become necessary.
15. Final metatype syntax must be chosen.
16. `DeclarationKind::Protocol` is part of a stable external contract.
17. Module linkage forces a runtime fake trait class.
18. Incrementality requires a semantic DB redesign rather than normal new products.
19. A newer canonical spec contradicts the plan.
20. One serious semantic correction does not resolve the same semantic failure.
21. A test only passes by weakening an invariant.

---

# 36. Testing Surface

Required coverage IDs:

```text
CV-01 trait keyword parses
CV-02 trait declaration/name ranges
CV-03 module namespace binding
CV-04 import/export trait declaration
CV-05 DeclarationKind::Trait
CV-06 no Protocol/ Trait dual semantic category
CV-07 generic trait declaration signature
CV-08 no nominal trait TypeId/class-object insertion
CV-09 TraitRef non-generic
CV-10 TraitRef generic
CV-11 TraitRef wrong kind/category/arity
CV-12 trait not silently accepted as ordinary inhabitable value type
CV-13 method requirement
CV-14 getter requirement
CV-15 setter requirement
CV-16 index getter requirement
CV-17 index setter requirement
CV-18 existing bodyful class index preserved
CV-19 existing bodyful impl index preserved
CV-20 TraitRequirementId stable
CV-21 trait-owned CallableId stable
CV-22 duplicate selector conflict
CV-23 generic member requirement
CV-24 Self in signature
CV-25 bodyful default
CV-26 default calls requirement
CV-27 default calls later-declared requirement
CV-28 default calls another default contract-relatively
CV-29 generic trait default resolves trait T
CV-30 generic member default resolves member generic
CV-31 default illegal field access
CV-32 default illegal super
CV-33 abstract call target has target=None / non-runtime authority
CV-34 no DeclarationSurface registration
CV-35 no conditional inherent registration
CV-36 enum requirement identity remains separate
CV-37 body-only default edit preserves TraitRequirementId
CV-38 body-only default edit preserves trait-owned CallableId
CV-39 signature edit invalidates TraitSurface
CV-40 bodyless↔bodyful preserves identities
CV-41 cold/incremental parity
CV-42 source target declaration reuse
CV-43 source target callable reuse
CV-44 compiler accepts unused trait
CV-45 no runtime class allocation
CV-46 no trait default method injection
CV-47 ordinary send unaffected by unused trait
CV-48 protocol-era active spec reconciled
CV-49 associated type remains deferred
CV-50 conformance syntax remains deferred
```

---

# 37. Verification Budget

## BUILD

Run:

```text
one exact parser/index regression after AST unit
one exact semantic requirement regression after TraitSurface unit
one exact default regression after abstract Self unit
one incremental mutation regression after query unit
one core compile/no-runtime regression after compiler unit
```

Do not run full workspace repeatedly.

## STABILIZE

Run:

```text
trait AST focused target
module trait focused target
semantic trait declarations/surfaces/defaults
incremental trait tests
existing focused index accessor regressions affected by shared body normalization
core language::traits
LSP focused test only if LSP-specific code changed
```

## CERTIFY

Run final focused acceptance + format.

Do not automatically call C3 `RELEASE_COMPLETE`.

---

# 38. Failure Classification

Use repository lifecycle convention:

```text
A — definitely caused by current change
B — probably caused by current change
C — unclear
D — clearly pre-existing / unrelated
```

Expected red tests for functionality not yet implemented are tracked separately as planned-red task conditions, not as baseline failure classes.

Policy:

```text
A/B
    fix before gate

C
    one bounded classification pass
    if nonblocking and still unclear, record

D
    record and defer
```

Never rerun an unchanged failure without a changed hypothesis/patch.

---

# 39. Gate Sequence

```text
G0
    C2.P3 takeover + drift reconciliation

G1
    trait syntax
    module declaration kind
    shared index MemberBody normalization

G2
    TraitInfo generic metadata
    TraitRef
    TraitRequirementId
    TraitSurface
    canonical CallableId reuse

G3
    body analyzer owner-generic generalization
    abstract Self lookup
    semantic-only abstract calls
    default analysis

G4
    query/fingerprint separation
    source index
    LSP presentation

G5
    compiler/runtime non-class boundary
    protocol spec migration closure

G6
    focused certification
    checkpoint update
    walkthrough
    C4 handoff
```

---

# Task T0 — C2.P3 Takeover and Plan Drift Gate

## Purpose

Start C3 only from the final C2 behavioral architecture.

## Required operations

1. Record:
   ```sh
   git status --short
   git branch --show-current
   git rev-parse HEAD
   ```
2. Read C2.P3 walkthrough and handoff.
3. Map P3's actual landed products to:
   ```text
   conditional impl domain
   conditional member index
   receiver-effective lookup
   semantic conditional selection
   compiler/lowering selection
   incremental fingerprints
   tooling surface
   ```
4. Verify:
   ```text
   unconditional DeclarationSurface remains separate
   EnumBehaviorProduct remains enum-specific
   no runtime impl scan
   no per-applied-class shortcut unless explicitly ratified by P3
   ```
5. Update `LANG005.C3-CHECKPOINT.md`:
   ```text
   baseline revision
   takeover map
   plan drift
   active plan = P1 IN_PROGRESS
   ```

## Acceptance

G0 passes only when C2 is checkpoint-complete and P3's actual interfaces are known.

## STOP

Any predecessor incompleteness or material architecture drift.

---

# Task T1 — Canonical `trait` Syntax and Shared Index Body Normalization

## Purpose

Create a dedicated trait declaration and repair the one shared behavior-member form that cannot yet represent requirements.

## Primary areas

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
AST tests
```

## Trait AST

Conceptually:

```rust
pub struct TraitDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub members: Vec<BehaviorMember>,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}
```

No:

```text
superclass
fields
invariants
variants
data components
```

Add:

```text
Statement::Trait(TraitDef)
```

## Parser

Parse:

```phalcom
trait Printable {
  print -> String
}
```

Reuse:

```text
generic parameter parsing
where clause parsing
BehaviorMember parsing
MemberBody declaration/block
```

## Index normalization

Change shared `IndexMethodDef.body` to `MemberBody` or exact equivalent.

Update parser so trait context can parse:

```phalcom
trait IntIndexable {
  [_ index: Int] -> Int
}
```

and bodyful existing index accessors remain valid.

Do not redesign index setter return semantics.

## Consumer audit

Update all production exhaustive/body consumers.

At minimum inspect:

```text
semantic signature helper
session callable body extraction
semantic_shard
fingerprint
source_index builder
source_index occurrence
compiler class declaration
semantic lowering
product optimizer
attributes
tests
```

## Tests

Add:

```text
empty trait
bodyless method
bodyful default
getter requirement
setter requirement
index getter requirement
index setter requirement
generic trait
where clause
range assertions
field rejected
constructor rejected
class-side deferred
```

Also run focused existing bodyful index tests.

## Acceptance

`Statement::Trait` exists and shared index members support declaration-only body shape without regressing existing index behavior.

---

# Task T2 — Module Declaration Kind and Trait Header Product

## Purpose

Publish trait as a named declaration while keeping it non-nominal in the type store.

## Primary areas

```text
phalcom-modules/src/declaration.rs
phalcom-modules/src/interface.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/session.rs or trait header owner
phalcom-semantic/src/traits.rs or equivalent
```

## Declaration kind

Migrate:

```text
DeclarationKind::Protocol
    -> DeclarationKind::Trait
```

after compatibility verification.

Add `Statement::Trait` to module interface collection.

## TraitInfo

Create repository-equivalent:

```rust
TraitInfo {
    declaration: DeclarationId,
    generic_signature: Option<GenericSignature>,
    source: ...
}
```

## Generic signature

Resolve with:

```text
TypeParameterOwner::Declaration(declaration)
```

Do not:

```text
NominalDeclarationHeader::from_signature
DeclarationTypeTable::insert(trait)
store.nominal_form(trait)
store.class_object_type(trait)
```

## Tests

```text
module namespace collision
import/export trait
DeclarationKind::Trait
generic signature owner
no DeclarationTypeTable trait entry
no class hierarchy edge
```

## Acceptance

Trait declaration shell/header exists independently of nominal runtime type metadata.

---

# Task T3 — TraitRef Formation

## Purpose

Create canonical generic trait contract references.

## Primary areas

```text
phalcom-semantic/src/traits.rs
type/static symbol resolution helpers
semantic tests
```

## Required path

```text
TypeAnnotation/StaticSymbolRef-like syntax
    ↓
DeclarationId lookup
    ↓
verify DeclarationKind::Trait
    ↓
TraitInfo generic signature
    ↓
resolve type arguments
    ↓
arity/kind/existing ordinary constraint validation
    ↓
TraitRef
```

Do not add `TypeLevelBinding::Trait`.

Do not create `TypeData::Trait`.

Do not treat trait declarations as ordinary proper value types.

## Tests

```text
non-generic TraitRef
generic TraitRef
wrong arity
wrong kind
wrong declaration kind
trait in ordinary value type context rejected/deferred
equivalent references equal
```

## Acceptance

C4 has a durable trait contract reference independent of conformance and runtime value typing.

---

# Task T4 — TraitRequirementId, TraitSurface, and Signature Publication

## Purpose

Build the abstract contract.

## Primary areas

```text
phalcom-semantic/src/traits.rs
checker/declaration_signature.rs
session/query wiring
diagnostics
tests
```

## Requirement identity

Add:

```rust
TraitRequirementId {
    owner: DeclarationId,
    selector: Selector,
    side: DispatchSide,
}
```

or exact equivalent.

## Callable identity reuse

For each member:

```text
owner = CallableOwnerId::Declaration(trait DeclarationId)
selector = canonical selector
side = instance in P1
```

Use the resulting `CallableId`.

## Trait generic resolver

Build explicit scoped bindings from `TraitInfo.generic_signature`.

Call:

```text
semantic_signature_for_syntax_with_resolver(...)
```

Do not depend on `DeclarationTypeTable`.

## Duplicate law

Same trait + side + selector conflicts even if parameter/return annotations differ.

No overload by types.

## Surface

Publish:

```text
TraitSurface
    requirements
    signatures
    visibility
    default-present/source metadata
```

Do not analyze bodies yet.

## Tests

```text
method/getter/setter/index requirement
bodyful member default-present
generic member
Self in signature
duplicate selector
visibility
no DeclarationSurface insertion
no enum requirement reuse
```

## Acceptance

Complete trait contract surface exists independently of default body analysis.

---

# Gate G2 — Surface Complete Before Bodies

Before continuing, prove:

```text
all member signatures are available
source order does not matter
TraitSurface can be queried without analyzing a default body
```

Do not proceed if body analysis is needed to build the surface.

---

# Task T5 — Generalize Callable Body Owner-Generic Inputs

## Purpose

Remove the nominal-only assumption that declaration generics must come from `DeclarationTypeTable`.

## Primary areas

```text
phalcom-semantic/src/checker/body.rs
body analysis call sites
tests
```

## Required change

Add a narrow explicit owner-generic input.

Example:

```rust
CallableBodyRequest {
    ...
    owner_generics: Option<&GenericSignature>,
}
```

or equivalent.

Existing nominal callers may continue to derive owner generics from `DeclarationTypeTable`.

Trait default caller passes `TraitInfo.generic_signature`.

## Constraints

Do not redesign all declaration type storage.

Do not insert traits into nominal tables.

Do not change existing constructor/class-body generic behavior.

## Tests

```text
existing generic class method still works
generic trait default can resolve trait T
member-local generic shadows/extends trait generic correctly
```

## Acceptance

Body analysis no longer requires nominal type metadata solely to expose owner generic bindings.

---

# Task T6 — Abstract Self Lookup and Semantic-Only Trait Contract Calls

## Purpose

Type-check defaults once against the trait contract.

## Primary areas

```text
phalcom-semantic/src/checker/context.rs
checker/expression.rs
checker/call.rs
trait lookup adapter
traits.rs
tests
```

## Trait default context

Set:

```text
current callable = trait-owned CallableId
current declaration = trait DeclarationId
current side = Instance
Self = existing SelfTypeTerm
field signatures = none
superclass capability = none
trait surface = active abstract contract
```

## Lookup

When receiver is the current trait's abstract Self:

```text
selector
    ↓
TraitSurface
    ↓
TraitRequirementId + CallableSemanticSignature
```

Do not register TraitSurface with ordinary `SurfaceDispatchResolver`.

## Application target

Extend canonical application target with semantic-only trait contract authority.

Conceptually:

```rust
CallableApplicationTarget {
    signature,
    callable: Some(trait_owned_callable),
    target: None,
    authority: TraitContract,
    ...
}
```

Name mechanically flexible.

## Critical negative rule

Do not create:

```text
InvocationTargetId::Behavioral(trait_owned_callable)
```

for abstract requirement calls.

## Default-to-default

Even if the requirement has a default:

```text
lookup = contract requirement/signature
```

not direct default body invocation.

## Tests

```text
default calls bodyless requirement
default calls later requirement
default calls bodyful member contract-relatively
generic trait default
Self return
unknown member rejected
field access rejected
super rejected
abstract application has no executable target
```

## Acceptance

Defaults are semantically valid without conformance or runtime target selection.

---

# Gate G3 — Default Semantics

G3 passes only when:

```text
default body analyzed once
trait Self lookup is surface-based
abstract calls reuse canonical application checking
no runtime InvocationTargetId is manufactured
no concrete witness/conformance exists
```

---

# Task T7 — Query, Fingerprint, Incremental, and Source Tooling Integration

## Purpose

Make C3 durable and incremental.

## Primary areas

```text
phalcom-semantic/src/db/
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/source_index/
phalcom-lsp/ only if needed
incremental tests
```

## Products

Repository-equivalent query separation:

```text
TraitInfo
TraitSurface
CallableBody(trait-owned CallableId)
```

## Fingerprints

Header/surface must not hash full body statements.

Body fingerprint owns statements.

## Source targets

Use:

```text
SemanticTargetId::Declaration
SemanticTargetId::Callable
```

Do not add trait-specific target variants without concrete need.

## LSP

Use semantic source products.

Presentation may map trait to Interface-like UI kind.

No LSP-local trait solver.

## Incremental tests

At minimum:

```text
default body-only edit
member return-type edit
selector edit
bodyless↔bodyful edit
trait generic edit
index requirement body presence edit where meaningful
cold/incremental equivalence
```

## Acceptance

Fine-grained invalidation and source navigation work without duplicate semantic identity.

---

# Task T8 — Compiler/Runtime Non-Class Boundary

## Purpose

Allow trait-containing modules without creating runtime trait behavior.

## Primary areas

```text
phalcom-core compiler statement dispatch
semantic lowering exhaustive matches
product optimizer / AST visitors
core tests
```

## Required behavior

```text
Statement::Trait
    compile-time semantic declaration
    no class allocation
    no FinalizeClass
    no field layout
    no method-table installation
```

Trait default bodies remain semantic source definitions in C3.

C4 will later decide how selected defaults become executable.

## Tests

```text
unused trait compiles
ordinary program around trait behaves unchanged
no trait ClassId/finalization path
default not installed on unrelated class
ordinary dispatch cannot discover trait-only requirement/default
```

## STOP

If module runtime binding requires an actual trait runtime object.

Do not create a fake class.

---

# Task T9 — Normative Protocol→Trait Specification Reconciliation

## Purpose

Remove competing active language rules.

## Required audit

Inspect all active protocol-era language-feature claims.

Migrate to one canonical trait specification.

Preserve historical material through archive/superseded marking.

## Must reconcile

```text
source spelling
declaration category
defaults
storage prohibition
future explicit conformance direction
runtime descriptor authority
class-side scope
associated-type deferral
```

Do not globally replace the English word protocol.

## Acceptance

`docs/spec/` has one effective trait declaration model.

---

# Task T10 — Focused Certification, Checkpoint Closure, Walkthrough, C4 Handoff

## Required operations

1. Run final focused tests.
2. Run `cargo fmt --all -- --check`.
3. Run negative searches.
4. Update C3 checkpoint.
5. Write P1 walkthrough.
6. Write C4 handoff.

## Negative searches

Inspect production code for:

```text
trait -> ClassDef flag
trait -> nominal DeclarationTypeTable entry
trait -> class_object_type
TypeLevelBinding::Trait
TraitSurface -> DeclarationSurface insertion
trait default -> target add_method
trait call -> InvocationTargetId::Behavioral before conformance
VM Invoke -> trait scan
duplicate trait-only index AST
```

Inspect active specs for legacy language-feature protocol claims.

## Checkpoint completion

Set:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only after evidence passes.

---

# 40. Expected Final Focused Acceptance Commands

Verify actual registered test names first.

Expected shape:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test trait_syntax

RUSTFLAGS='' cargo test -p phalcom-modules trait

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic traits

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental

RUSTFLAGS='' cargo test -p phalcom-core --test core language::traits

cargo fmt --all -- --check
```

Because `IndexMethodDef` is shared infrastructure, also run the narrow existing index parser/semantic/core regression target(s) affected by the body representation change.

Do not accept zero-test filters.

Do not reflexively run workspace-wide suites.

---

# 41. C4 Handoff Contract

C4 must be able to consume:

```text
how to recognize a trait DeclarationId
how to read TraitInfo
how to form TraitRef
how to query TraitSurface
how to enumerate TraitRequirementId
how to obtain the trait-owned CallableId/signature
how to locate a default body/source callable
how to substitute trait generic arguments
how abstract Self is represented
how visibility is retained
how source targets are represented
how queries/fingerprints are keyed
how C2.P3 receiver-effective inherent behavior is queried
what compiler/runtime currently does with traits
```

C4 must not need to reopen C3 identity design.

---

# 42. Completion Truth Table

| State | C3.P1 completion |
|---|---|
| trait parses only | NOT COMPLETE |
| TraitInfo/TraitRef only | NOT COMPLETE |
| TraitSurface exists but defaults not abstract-Self checked | NOT COMPLETE |
| defaults work by fake nominal trait class | INVALID |
| defaults work but index requirements unavailable | NOT COMPLETE |
| all semantics work but incremental/source/compiler boundaries missing | NOT COMPLETE |
| all tasks + focused evidence + docs + handoff complete | COMPLETE / FOCUSED_TESTED |

---

# 43. Plan Self-Review Checklist

Before claiming completion:

- [ ] C2.P3 was complete before C3 edits began.
- [ ] `trait` is a dedicated AST declaration.
- [ ] `DeclarationKind::Trait` is canonical.
- [ ] Trait declaration identity reuses `DeclarationId`.
- [ ] No unnecessary `TraitId` identity universe exists.
- [ ] `TraitRequirementId` exists.
- [ ] Trait member/default source identity reuses `CallableId`.
- [ ] No unnecessary `TraitMemberId` exists.
- [ ] No unnecessary `TraitDefaultId` exists.
- [ ] Trait generic signature is stored outside nominal `DeclarationTypeTable`.
- [ ] No nominal trait TypeId was fabricated.
- [ ] No trait class-object TypeId was fabricated.
- [ ] `TypeLevelBinding` was not extended with named-trait declarations.
- [ ] `TraitRef` is separate from ordinary inhabitable value types.
- [ ] `TraitSurface` is separate from inherent surfaces.
- [ ] Shared `IndexMethodDef` supports declaration-only bodies.
- [ ] Existing bodyful index behavior still passes focused regressions.
- [ ] C3 did not redesign index setter return semantics.
- [ ] Complete TraitSurface publication precedes default body checking.
- [ ] Default body analysis receives trait owner generics explicitly.
- [ ] Abstract `Self` reuses `SelfTypeTerm`.
- [ ] Trait Self lookup reads TraitSurface.
- [ ] Abstract requirement application reuses canonical argument/generic checking.
- [ ] Abstract requirement application has no executable `InvocationTargetId`.
- [ ] Default-to-default calls remain contract-relative.
- [ ] No conformance/witness relation exists yet.
- [ ] No associated type support was pulled forward.
- [ ] No trait generic conformance constraint was pulled forward.
- [ ] No final metatype syntax was invented.
- [ ] Source targets reuse canonical Declaration/Callable identity.
- [ ] Body-only edits preserve requirement/callable identity.
- [ ] Cold and incremental results agree.
- [ ] Compiler creates no runtime trait class/method injection.
- [ ] Protocol-era active spec conflict is reconciled.
- [ ] Focused test filters selected nonzero tests.
- [ ] C3 checkpoint is updated.
- [ ] Walkthrough exists.
- [ ] C4 handoff exists.

Any unchecked item means P1 is not complete.

---

# 44. Final Implementation Principle

> **C3 must introduce the trait contract without introducing a second parallel identity system or pretending the contract is a nominal runtime type. Reuse `DeclarationId`, `CallableId`, and existing source targets; add `TraitRequirementId`, `TraitRef`, and `TraitSurface` only where the semantics are genuinely new. Normalize index bodies in the shared AST so requirements are real. Check defaults once under abstract `Self`, route their member calls through the trait contract with no runtime invocation target, and leave every concrete conformance/witness decision to C4.**
