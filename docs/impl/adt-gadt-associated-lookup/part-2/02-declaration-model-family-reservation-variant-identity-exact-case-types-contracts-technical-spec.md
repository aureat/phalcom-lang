# Phalcom ADT/GADT + Associated Lookup
## Part 2 — Declaration Model, Family Reservation, Variant Identity, Exact Case Types, and Closed-Enum Requirements

**Status:** Technical specification  
**Series:** ADT/GADT + Associated Lookup, Part 2 of 6  
**Repository:** `aureat/phalcom-lang`  
**Verified `main` baseline:** `1892bcff51f75dd2f3df2a0661b03371250d4090`  
**Baseline commit:** `docs(semantic): record correctness plans and authority audit`  
**Intended repository path:** `docs/impl/adt-gadt-associated-lookup/part-2/02-declaration-model-family-reservation-variant-identity-exact-case-types-contracts-technical-spec.md`

---

# 1. Executive Summary

Part 1 establishes syntax and AST identity for `enum`, `@variant`, and the new associated-lookup grammar. Part 2 establishes what those declarations *mean* to `phalcom-semantic`.

The central requirement is that enum cases become first-class static semantic entities without being misrepresented as ordinary methods, synthetic source classes, runtime discriminants, or heavyweight boxed values.

After Part 2, the compiler must be able to answer, without performing `::` expression resolution or runtime lowering:

- what enum declaration a source `enum` denotes;
- what the enum's canonical generic type form is;
- what each exact variant is, by complete selector identity;
- which exact variants belong to the same reserved associated family;
- whether a variant is a singleton value or a constructor, including the distinct zero-argument constructor form;
- what each variant payload field means and what type it has;
- what canonical enum specialization a variant inhabits;
- what canonical exact-case type represents that precise case;
- what GADT equalities the case declaration establishes;
- what behavior is shared by the enum root;
- what behavior is implemented or overridden by a specific case;
- what signature-only enum-root requirements every concrete case must satisfy;
- what family names are reserved on an effective associated owner surface;
- what variant visibility applies independently to naming/matching, construction, and payload access;
- what source locations target the exact variant identity;
- what immutable, incrementally reusable semantic products expose all of the above.

Part 2 deliberately does **not** resolve expressions such as `Option::Some(1)`. That is Part 3. It deliberately does **not** choose tagged layouts, heap allocation, boxing, discriminants, or runtime case-class representations. That is Part 4. It does not implement `match` elimination, projection, exhaustiveness, or branch refinement. That is Part 5.

The architectural invariant is:

```text
source enum declaration
    ↓
DeclarationId for enum root
    ↓
EnumInfo
    ├── VariantFamilyId / AssociatedFamilyId
    ├── VariantId (complete selector identity)
    ├── VariantInfo
    │     ├── payload field metadata
    │     ├── result type template
    │     ├── GADT case equality environment
    │     ├── exact-case type template
    │     └── optional VariantConstructorSignature
    ├── enum-root behavior
    ├── per-case behavior
    └── closed-enum requirements

TypeStore
    ├── canonical Expr<Int>            = normal Applied type
    └── canonical exact case           = ExactCase(VariantTypeId, Expr<Int>)

AssociatedSurface
    ├── declared class-side behavioral families
    ├── declared variant families
    └── inherited behavioral family reservations only
```

`phalcom-semantic` remains the only static semantic authority.

---

# 2. Inputs, Precedence, and Verification Boundary

## 2.1 Normative inputs

This specification follows the project handoff and the decisions ratified in the conversation. In particular, it incorporates the accepted correction that:

```phalcom
@variant None
@variant None()
```

are both valid and semantically distinct.

The first is a canonical singleton. The second is a zero-argument variant constructor that creates an empty case instance.

## 2.2 Attached-design-document limitation

The original handoff referred to attached ADT/GADT and associated-family design documents. Those attachments were not available through the conversation file index during Part 1, and no new attachment contents became available while preparing Part 2. This specification therefore does not invent or paraphrase unavailable attachment text.

Where the handoff explicitly ratified a decision, that decision is treated as normative. Repository code is treated as implementation truth.

## 2.3 Repository verification

`main` was rechecked before this Part 2 design and remained:

```text
1892bcff51f75dd2f3df2a0661b03371250d4090
```

The baseline does **not** yet contain the Part 1 enum/associated AST implementation. Therefore this specification distinguishes:

- **current-baseline symbols** — verified against `1892bcff...`;
- **Part 1 prerequisite symbols** — expected to exist once Part 1 is implemented;
- **Part 2 proposed symbols** — introduced by this specification.

An implementation agent must rebase against the actual post-Part-1 HEAD and reconcile exact names before editing.

---

# 3. Current Repository Archaeology

This section records the implementation facts that materially constrain Part 2.

## 3.1 Stable declaration and callable identity

`phalcom-semantic/src/identity.rs` currently defines:

```rust
pub struct CallableId {
    pub owner: DeclarationId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

and `SemanticTargetId` currently supports bindings, declarations, callables, fields, and modules, but no exact enum variant target.

This confirms that the current `CallableId` is a behavioral method identity, not a suitable identity for a variant declaration itself.

## 3.2 Canonical type store

`phalcom-semantic/src/types/id.rs` defines `TypeId(pub u32)` as a store/snapshot-local canonical handle.

`phalcom-semantic/src/types/store.rs` hash-conses `(TypeData, KindId)` and currently contains:

```text
Never
Unit
ClassObject
Nominal
Applied
Union
Tuple
Record
Callable
Parameter
Lambda
SelfType
```

There is no exact-case type.

Generic application is already canonicalized. `Expr<Int>` therefore naturally uses the existing:

```rust
TypeData::Applied {
    origin,
    arguments,
}
```

machinery. Part 2 must extend this type universe rather than build a second enum-specific generic type system.

## 3.3 Declaration type metadata

`phalcom-semantic/src/declarations.rs` publishes:

```rust
DeclarationTypeInfo {
    declaration,
    form,
    class_object_type,
    kind,
    generic_signature,
    supertype_template,
}
```

through `DeclarationTypeTable`.

This is the correct home for the enum root's ordinary nominal type form, but it is not enough to describe the enum's closed cases or associated families.

## 3.4 Generic equality infrastructure already exists

`phalcom-semantic/src/types/parameter.rs` already has:

```rust
GenericConstraint::Equivalent { left, right }
```

and `phalcom-semantic/src/types/constraint.rs` has:

```rust
TypeConstraint::Equal(TypeId, TypeId)
```

`types/annotation.rs::resolve_generic_signature` already lowers source equivalence constraints.

Part 2 should reuse this conceptual equality model for GADT case facts instead of inventing an unrelated proof language.

## 3.5 Type specialization infrastructure

`types/environment.rs` has `TypeEnvironment` and `TypeView`.

`types/substitution.rs` has `TypeSubstitution` and recursive specialization of existing `TypeData` forms.

Both must learn `ExactCase`. The GADT case-equality environment should be a declaration-owned semantic product and should use the same canonical `TypeId` terms.

## 3.6 Current relation engine is declaration-hierarchy-oriented

`types/relation.rs` models ordinary class hierarchy using `DeclarationId` and handles nominal/applied/structural/callable relations.

There is no exact case relation.

Part 2 must add:

```text
ExactCase(V, E) <: E
```

without synthesizing a fake source class declaration for every exact case.

## 3.7 Current dispatch surface is behavioral

`phalcom-semantic/src/surface.rs` publishes instance and class behavioral surfaces.

`phalcom-semantic/src/dispatch.rs` resolves ordinary message-send behavior from those surfaces.

This is *not* an associated-declaration namespace. Associated family publication must be a separate semantic surface.

## 3.8 Declaration source-to-semantic boundary

`checker/declaration_signature.rs` is explicitly the canonical source-to-semantic boundary for source callable declarations. `CallableSemanticSignature` is declaration-owned and dispatch is a projection.

Part 2 should follow the same rule:

> enum/variant semantics are canonical declaration products first; dispatch, associated lookup, runtime objects, and editor presentation are projections/consumers.

## 3.9 Incremental semantic publication pipeline

`phalcom-semantic/src/session.rs` currently:

1. refreshes parsed/unlinked products;
2. predeclares source declarations;
3. constructs the linked resolver;
4. enriches the semantic graph;
5. realizes declaration shells and generic metadata;
6. materializes signatures and behavioral surfaces;
7. checks bodies;
8. freezes an immutable snapshot.

The predeclaration loops currently recognize only `Statement::Class`.

Part 2 must insert enum roots into this pipeline, not create an independent checker pass outside the DB/snapshot model.

## 3.10 The module layer already anticipates ADTs

`phalcom-modules/src/declaration.rs` already contains:

```rust
pub enum DeclarationKind {
    Class,
    Protocol,
    Adt,
    Alias,
}
```

This is a strong existing seam. New enum roots should predeclare as `DeclarationKind::Adt`.

## 3.11 Module interfaces currently omit enum declarations

`phalcom-modules/src/interface.rs::InterfaceBuilder::build` currently collects top-level classes and bindings, but not enums.

Part 2 must make enum roots ordinary immutable module declarations so imports, exports, and linked type resolution can find them.

## 3.12 Existing contracts module means Design by Contract

`phalcom-semantic/src/contracts/spec.rs` models `@requires`, `@ensures`, and `@invariant`.

Therefore closed-enum signature obligations must not be introduced under a generic Rust module called `contracts`. This specification uses **enum requirements** / **case obligations**.

## 3.13 Source indexing uses exact semantic target identities

`source_index` records `SemanticTargetId` values and builds target-to-occurrence maps.

A variant declaration/name occurrence therefore needs a real `SemanticTargetId::Variant(VariantId)` rather than a fake callable or declaration target.

---

# 4. Goals

Part 2 must establish all of the following.

1. Enum roots are canonical source declarations and canonical nominal generic type forms.
2. Variants have stable semantic identity by complete selector.
3. Families have stable identity by owner + base.
4. Exact cases have canonical `TypeId`s.
5. `Expr<Int>` remains an ordinary canonical applied type.
6. Exact-case types are more precise than, and subtype the, corresponding enum specialization.
7. GADT case results create declaration-owned equality facts.
8. Case equality is available while resolving/checking that case's payload and behavior.
9. Singleton, zero-argument constructor, and payload constructor variants remain distinct.
10. Variant constructors are first-class callable declarations conceptually, but are not methods.
11. Enum-root shared/default behavior and per-case behavior are modeled.
12. Signature-only enum-root behavior creates closed-enum obligations.
13. Family reservation is explicit and deterministic.
14. Associated declaration inheritance and behavioral inheritance remain distinct.
15. Variant visibility has separate name/match, construction, and payload axes.
16. Enum/variant products participate in incremental DB publication and immutable snapshots.
17. No exact-case design choice requires heavyweight runtime boxing.
18. No Part 2 implementation creates a second semantic authority in `phalcom-lsp`.

---

# 5. Non-Goals

Part 2 does not implement:

- `AssociatedLookupExpr` resolution;
- `AssociatedInvokeExpr` resolution;
- overload selection for `owner::name(args)`;
- first-class family values;
- bound family values;
- associated-expression diagnostics such as missing family/member at use sites;
- runtime enum layouts;
- runtime discriminants;
- allocation strategy;
- runtime case classes;
- boxing at dynamic-erasure boundaries;
- bytecode for variants;
- `match`;
- exhaustiveness;
- variant pattern projection;
- branch-local GADT refinement;
- core `Option` migration;
- reflection metadata for exact cases;
- final LSP hover/completion/go-to-definition support for enums.

Those are Parts 3–6 or later editor work.

---

# 6. Normative Identity Model

## 6.1 Enum root

The enum root uses the normal stable declaration identity:

```rust
DeclarationId
```

Example:

```phalcom
enum Expr<T> { ... }
```

has an ordinary declaration identity conceptually equivalent to:

```text
DeclarationId(module, "Expr")
```

and an ordinary generic nominal type form.

There is no separate `EnumId` required merely to duplicate `DeclarationId`.

## 6.2 Variant identity

A variant's stable semantic identity is:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantId {
    pub owner: DeclarationId,
    pub selector: Selector,
}
```

The selector is the *complete exact selector*.

Therefore:

```phalcom
enum Example {
    @variant None
    @variant None()
    @variant None(_ value: Int)
}
```

produces three different stable identities:

```text
VariantId(Example, #None)
VariantId(Example, #None())
VariantId(Example, #None(_))
```

This identity is source-semantic and cross-revision-stable as long as owner and selector remain stable.

A runtime tag/discriminant is **not** a `VariantId`.

## 6.3 Variant family identity

Variant-family identity remains explicitly distinct:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantFamilyId {
    pub owner: DeclarationId,
    pub base_name: Box<str>,
}
```

All three variants above belong to:

```text
VariantFamilyId(Example, "None")
```

## 6.4 Universal associated family identity

Part 3 needs one family namespace for variants and class-side behavioral members, including named operators/subscripts.

Part 2 therefore introduces:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssociatedFamilyId {
    pub owner: DeclarationId,
    pub base: SelectorBase,
}
```

`VariantFamilyId` remains a typed variant-specific identity and can convert to:

```rust
AssociatedFamilyId {
    owner,
    base: SelectorBase::Named(base_name),
}
```

This avoids redesigning family identity in Part 3 for operators/subscripts.

## 6.5 Payload field identity

Variant payload storage/projection identity is separate from constructor identity:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantFieldId {
    pub variant: VariantId,
    pub index: u32,
}
```

Index, not local field spelling, is the stable field identity because selector slots do not include local parameter names.

Example:

```phalcom
@variant Error(_ value: Int, reason message: String)
```

has payload fields at indices 0 and 1. The constructor's external selector identity is still `#Error(_,reason)`.

## 6.6 Variant constructor identity

Constructors are not methods:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantConstructorId {
    pub variant: VariantId,
}
```

Only constructor-shaped variants have this identity.

Bare singleton:

```phalcom
@variant None
```

has no `VariantConstructorId`.

Zero-argument constructor:

```phalcom
@variant None()
```

does.

---

# 7. Compact Variant Handles for Canonical Types

Stable source identity and hot type-store identity have different requirements.

`VariantId` intentionally contains a `DeclarationId` and a structural `Selector`. It is excellent for:

- DB keys;
- source indexing;
- diagnostics;
- cross-revision semantic identity;
- exact member lookup.

It is unnecessarily large to duplicate inside every canonical `TypeData::ExactCase`.

Part 2 therefore introduces a store-relative compact handle:

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantTypeId(pub u32);
```

`TypeStore` owns an append-only interner:

```rust
variant_identities: Vec<VariantId>,
variant_identity_to_id: HashMap<VariantId, VariantTypeId>,
```

with APIs conceptually like:

```rust
pub fn intern_variant_identity(&mut self, variant: VariantId) -> VariantTypeId;
pub fn variant_identity(&self, id: VariantTypeId) -> &VariantId;
```

This follows the existing persistent type-store rule:

- an old `VariantTypeId` never changes denotation;
- renaming/changing the selector allocates a new handle;
- retained snapshots preserve old meanings;
- hot hashing/comparison uses a compact integer.

`VariantTypeId` must never escape as the public semantic identity of a variant.

---

# 8. Canonical Exact-Case Types

## 8.1 New TypeData form

Add:

```rust
TypeData::ExactCase {
    variant: VariantTypeId,
    enum_type: TypeId,
}
```

The `enum_type` is the canonical proper enum specialization inhabited by that exact case.

Examples:

```text
Expr<Int>

ExactCase(Expr::Int(_), Expr<Int>)
ExactCase(Expr::Add(_,_), Expr<Int>)
```

and:

```text
Option<Int>

ExactCase(Option::Some(_), Option<Int>)
ExactCase(Option::None, Option<Int>)
```

## 8.2 `Expr<Int>` remains ordinary canonical generic application

No special GADT or enum type application representation is introduced.

Repeated construction of:

```text
Expr<Int>
```

must return the same `TypeId` in one `TypeStore`, using existing `apply_type_form`.

## 8.3 Exact cases are also canonical

Repeated requests for the same pair:

```text
VariantId(Expr::Int(_))
+
canonical Expr<Int>
```

must return the same exact-case `TypeId`.

Different variant, same root specialization:

```text
ExactCase(Int, Expr<Int>)
!=
ExactCase(Add, Expr<Int>)
```

Same variant, different valid specialization:

```text
ExactCase(Some, Option<Int>)
!=
ExactCase(Some, Option<String>)
```

## 8.4 Owner validation

A low-level exact-case constructor must reject a mismatch such as:

```text
variant = Option::Some(_)
enum_type = Result<Int, Error>
```

The type store can validate by peeling `Applied` to its nominal origin and comparing it with `VariantId.owner`.

Conceptual API:

```rust
pub fn exact_case_type(
    &mut self,
    variant: &VariantId,
    enum_type: TypeId,
) -> Result<TypeId, ExactCaseTypeError>;
```

Errors should include:

```rust
pub enum ExactCaseTypeError {
    EnumTypeMustBeProper { actual_kind: KindId },
    EnumTypeHasNoNominalOrigin { enum_type: TypeId },
    OwnerMismatch {
        variant_owner: DeclarationId,
        enum_owner: DeclarationId,
    },
}
```

## 8.5 Exact cases are proper types

Every `ExactCase` has kind `Type`.

The `enum_type` may contain declaration parameters:

```text
Option<T>
ExactCase(Some, Option<T>)
```

This is still a proper type template inside the generic declaration.

## 8.6 Lazy specialization

Part 2 may materialize one exact-case *template* per variant declaration:

```text
ExactCase(Some, Option<T>)
ExactCase(Int, Expr<Int>)
```

It must not eagerly materialize every future Cartesian product:

```text
all variants × all generic specializations
```

Part 3 can specialize the template when actual generic arguments are known.

## 8.7 No proof history in exact-case identity

The following derivations must converge to the same exact type:

```text
explicit GADT result
constructor result specialization
future match refinement
future control-flow proof
```

Proof/equality history is not part of `TypeData::ExactCase`.

---

# 9. Exact-Case Type Relations

Add the fundamental rule:

```text
ExactCase(V, E) <: E
```

Then normal enum-root relations continue from `E`.

For example:

```text
ExactCase(Expr::Int(_), Expr<Int>)
    <: Expr<Int>
    <: Object
```

For invariant `Expr<T>`:

```text
ExactCase(Expr::Int(_), Expr<Int>)
    </: Expr<String>
```

unless normal generic relation rules independently establish that relation.

Two different exact cases do not subtype each other merely because they share a root.

```text
ExactCase(Int, Expr<Int>)
</:
ExactCase(Add, Expr<Int>)
```

Exact cases are semantically final.

Do not add fake `DeclarationId` superclass nodes for them.

---

# 10. Union and Flow Precision Policy

Do **not** globally change `TypeStore::union` to collapse every union of exact cases sharing a root.

This would destroy useful future information:

```text
ExactCase(A,E) | ExactCase(B,E)
```

can represent a strict subset of a closed enum and will be useful for Part 5 match/exhaustiveness reasoning.

Current `join_type_knowledge` can continue to form unions.

Contextual widening to the enum root may be added where an expected type or explicit boundary makes precision unnecessary, but that is not a canonicalization rule.

Therefore:

```text
canonical exactness = preserved
contextual widening = optional consumer policy
```

---

# 11. Enum Declaration Semantic Product

Introduce a focused semantic module, recommended:

```text
phalcom-semantic/src/enum_semantics.rs
```

Avoid `contracts.rs` naming and avoid tying this to runtime layout.

Core structures:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumInfo {
    pub owner: DeclarationId,
    pub root_form: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub default_result_type: TypeId,
    pub variants: Box<[VariantId]>,
    pub variant_families: Box<[VariantFamilyId]>,
    pub source: Option<SemanticSourceSpan>,
}
```

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VariantShape {
    Singleton,
    Constructor,
}
```

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantInfo {
    pub id: VariantId,
    pub type_handle: VariantTypeId,
    pub family: VariantFamilyId,
    pub shape: VariantShape,
    pub fields: Box<[VariantFieldSemantic]>,
    pub result_type_template: TypeId,
    pub exact_case_template: TypeId,
    pub case_environment: CaseTypeEnvironment,
    pub constructor: Option<VariantConstructorSignature>,
    pub visibility: VariantVisibility,
    pub source: SemanticSourceSpan,
}
```

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantFieldSemantic {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub declared_type: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}
```

An aggregate table:

```rust
#[derive(Clone, Debug, Default)]
pub struct EnumSemanticTable {
    by_owner: HashMap<DeclarationId, EnumInfo>,
    variants: HashMap<VariantId, VariantInfo>,
    families: HashMap<VariantFamilyId, Box<[VariantId]>>,
}
```

Queries must support:

```rust
pub fn enum_info(&self, owner: &DeclarationId) -> Option<&EnumInfo>;
pub fn variant(&self, id: &VariantId) -> Option<&VariantInfo>;
pub fn variant_by_selector(
    &self,
    owner: &DeclarationId,
    selector: &Selector,
) -> Option<&VariantInfo>;
pub fn variant_family(
    &self,
    id: &VariantFamilyId,
) -> Option<&[VariantId]>;
```

---

# 12. Singleton vs Zero-Argument Constructor Semantics

This distinction is normative.

## 12.1 Bare singleton

```phalcom
@variant None
```

means:

```text
selector = #None
shape = Singleton
fields = []
constructor = None
```

The semantic declaration denotes one canonical singleton case value.

Part 4 decides its physical representation.

## 12.2 Explicit zero-argument constructor

```phalcom
@variant None()
```

means:

```text
selector = #None()
shape = Constructor
fields = []
constructor = Some(VariantConstructorSignature { parameters = [] })
```

Construction creates an empty case instance.

Identity-sensitive runtime semantics are Part 4, but Part 2 must preserve the constructor/value distinction.

## 12.3 Payload constructor

```phalcom
@variant Some(_ value: T)
```

means:

```text
selector = #Some(_)
shape = Constructor
fields = [value: T]
constructor = Some(...)
```

## 12.4 Same family

All may coexist:

```phalcom
enum Example {
    @variant None
    @variant None()
    @variant None(_ value: Int)
}
```

because exact selector identity differs while base family identity is the same.

---

# 13. Variant Constructor Signatures

Variant constructors are first-class callable declarations conceptually, but they are not methods and must not be placed into ordinary message dispatch.

Define:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantConstructorSignature {
    pub constructor: VariantConstructorId,
    pub parameters: Box<[VariantConstructorParameter]>,
    pub result_type_template: TypeId,
    pub exact_case_template: TypeId,
    pub source: SemanticSourceSpan,
}
```

A constructor parameter may reference the corresponding payload field:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantConstructorParameter {
    pub field: VariantFieldId,
    pub external_label: Option<Box<str>>,
    pub local_name: Box<str>,
    pub declared_type: DeclaredTypeFact,
}
```

Part 2 publishes this signature.

Part 3 will:

- infer/specialize generic arguments;
- select the applicable exact constructor from a family;
- produce a first-class callable value when exact narrowing is requested;
- type direct invocation.

Do not publish a singleton as a zero-arg constructor.

---

# 14. Default Enum Result Type

For:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
}
```

a variant with no explicit result annotation receives:

```text
Option<T>
```

as `result_type_template`.

For a non-generic enum:

```phalcom
enum Direction {
    @variant North
}
```

the default is:

```text
Direction
```

The default result must be produced using the normal canonical generic application machinery.

---

# 15. GADT Result Semantics

Consider:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

For `Int`:

```text
result_type_template = Expr<Int>
case equality        = T ≡ Int
exact_case_template  = ExactCase(Expr::Int(_), Expr<Int>)
```

## 15.1 Result annotation validation

An explicit variant result must:

1. resolve with normal type-annotation resolution;
2. be a proper type;
3. be fully saturated;
4. have a nominal origin equal to the enclosing enum `DeclarationId`;
5. have argument kinds valid under the enum's generic signature.

Reject:

```phalcom
@variant Bad(...) -> Option<Int>
```

inside `Expr`.

Reject an unsaturated generic result:

```phalcom
@variant Bad(...) -> Expr
```

when `Expr` requires type arguments.

## 15.2 Positional equality derivation

If the enum root has parameters:

```text
P0, P1, ... Pn
```

and the result is:

```text
Enum<R0, R1, ... Rn>
```

the case introduces equations:

```text
P0 ≡ R0
P1 ≡ R1
...
Pn ≡ Rn
```

These are declaration-owned case facts.

## 15.3 Case equality environment

Introduce:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseTypeEnvironment {
    pub bindings: BTreeMap<TypeParameterId, TypeId>,
    pub equalities: Box<[GenericConstraint]>,
}
```

The exact representation may use sorted boxed pairs instead of `BTreeMap` for compact immutable products, but semantics must be deterministic.

The environment must normalize transitive equalities.

Example:

```text
A ≡ B
B ≡ Int
```

must specialize both `A` and `B` to `Int`.

## 15.4 Occurs check

Reject impossible cyclic refinements such as a case result that implies:

```text
T ≡ List<T>
```

unless the type system later explicitly adds equi-recursive types.

Part 2 must not silently publish a cyclic rewrite environment.

## 15.5 Unconstrained parameters remain lexical parameters

An equation such as:

```text
A ≡ A
```

does not make `A` unknown or underconstrained. It simply adds no specialization.

The GADT case environment is not ordinary call-site inference and must not require every declaration parameter to solve to a concrete type.

## 15.6 Equality is separate from exact type identity

The exact case contains the canonical result type.

The equality environment explains what may be assumed in the lexical generic declaration context.

For:

```phalcom
@variant Int(_ value: Int) -> Expr<Int>
```

these are separate facts:

```text
exact-case result:
    ExactCase(Int, Expr<Int>)

case theorem:
    T ≡ Int
```

---

# 16. GADT Equality Scope

Case equality must be active while checking all semantics owned by the case:

- payload field type annotations;
- variant constructor signature;
- case-local method/getter/setter annotations;
- case-local method bodies;
- compatibility with enum-root requirements;
- compatibility with enum-root default behavior when overridden.

Part 5 will later bring the same declaration-owned equality facts into a `match` branch when the corresponding case has been proven.

`match` does not invent the GADT equality.

---

# 17. Callable Ownership for Per-Case Behavior

Current `CallableId.owner: DeclarationId` cannot uniquely identify both:

```text
enum root method #area
Circle case override #area
Rectangle case override #area
```

without fake declaration IDs.

Part 2 therefore generalizes behavioral callable ownership:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CallableOwnerId {
    Declaration(DeclarationId),
    Variant(VariantId),
}
```

and:

```rust
pub struct CallableId {
    pub owner: CallableOwnerId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

Add helpers:

```rust
impl CallableOwnerId {
    pub fn declaration(&self) -> &DeclarationId;
    pub fn module(&self) -> &ModuleId;
}

impl CallableId {
    pub fn declaration_owner(&self) -> &DeclarationId;
    pub fn module(&self) -> &ModuleId;
}
```

Existing class methods use:

```rust
CallableOwnerId::Declaration(...)
```

Case-local methods use:

```rust
CallableOwnerId::Variant(...)
```

This allows the existing:

- `CallableSignatureTable`;
- `CallableBody` query;
- source attachment;
- explanation arena;
- generic method parameter owner;
- diagnostics;
- body analysis

to remain keyed by canonical callable identity without introducing a parallel case-method checker.

Variant constructors still do **not** use `CallableId`: they use `VariantConstructorId`.

This is an important separation:

```text
case behavior method     -> CallableId(owner = Variant)
variant constructor      -> VariantConstructorId
```

---

# 18. Compatibility Fields on CallableSemanticSignature

`CallableSemanticSignature` currently stores a declaration `owner`.

For migration, Part 2 may keep that field as the enclosing declaration root:

- ordinary class method -> class declaration;
- enum-root behavior -> enum declaration;
- case-local behavior -> enclosing enum declaration.

The canonical callable identity remains `signature.callable`.

All code that requires exact lexical behavior ownership must consult:

```text
signature.callable.owner
```

All code that needs the enclosing source declaration can consult the compatibility declaration owner.

A later callable-model cleanup may rename this field, but Part 2 should avoid an unrelated broad signature redesign.

---

# 19. Enum-Root Behavior

Enum root behavior has two categories.

## 19.1 Bodyful shared/default behavior

Example:

```phalcom
enum Shape {
    describe -> String {
        "shape"
    }

    @variant Circle(_ radius: Float)
}
```

`describe` is ordinary instance behavior declared by the enum root.

Every exact case inherits it unless it supplies a compatible override.

## 19.2 Signature-only closed-enum requirement

Example:

```phalcom
enum Shape {
    area -> Float

    @variant Circle(_ radius: Float) {
        area -> Float {
            ...
        }
    }

    @variant Rectangle(_ width: Float, _ height: Float) {
        area -> Float {
            ...
        }
    }
}
```

`area -> Float` is not an abstract open-class method. It is a closed-enum requirement.

Every concrete variant in this same enum must satisfy it.

## 19.3 Requirements apply to all variants

Construction visibility does not remove a variant from the closed sum.

A private-construction case must still satisfy enum requirements.

Singleton cases and zero-argument constructor cases are equally concrete cases and must satisfy requirements.

---

# 20. Enum Requirement Model

Use a dedicated module such as:

```text
phalcom-semantic/src/enum_requirements.rs
```

Define:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EnumRequirementId {
    pub owner: DeclarationId,
    pub selector: Selector,
}
```

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumRequirement {
    pub id: EnumRequirementId,
    pub signature: CallableSemanticSignature,
    pub source: SemanticSourceSpan,
}
```

and a per-case result:

```rust
pub enum CaseRequirementStatus {
    Satisfied { implementation: CallableId },
    Missing,
    Incompatible { implementation: CallableId },
    Blocked,
}
```

Requirements must be fully typed enough to compare. A signature-only declaration with unresolved required parameter/return types is itself erroneous and must not silently produce a satisfied obligation.

Requirement publication is a separate incremental product:

```rust
#[derive(Clone, Debug, Default)]
pub struct EnumRequirementTable {
    by_owner: HashMap<DeclarationId, Arc<EnumRequirementsProduct>>,
}
```

This avoids a dependency cycle in which structural `EnumDeclarationProduct` would need behavioral signatures before those signatures can themselves use the enum's GADT case environment.

---

# 21. GADT-Aware Requirement Specialization

Given:

```phalcom
enum Expr<T> {
    eval -> T

    @variant Int(_ value: Int) -> Expr<Int> {
        eval -> Int {
            value
        }
    }
}
```

the semantic process is:

```text
root requirement:
    eval -> T

Int case environment:
    T ≡ Int

specialized requirement:
    eval -> Int

case implementation:
    eval -> Int

result:
    satisfied
```

The case implementation is not "changing the contract."

The contract is interpreted under a theorem established by the case declaration.

---

# 22. Requirement/Override Compatibility

For v1, compatibility must be deterministic and sound.

## 22.1 Selector shape

Exact selector identity must match.

No heuristic base-name matching is allowed for requirement satisfaction.

## 22.2 Rest shape

Parameter rest modes must match.

## 22.3 Generic method binders

If enum behavior methods are generic, compare method generic signatures alpha-equivalently:

- same generic arity;
- same parameter kinds;
- same variance;
- equivalent constraints after binder correspondence.

Do not compare binder names.

## 22.4 Parameter types

After:

1. applying the case GADT environment to the root signature;
2. aligning method generic binders;

v1 requires parameter types to be equivalent/invariant.

This intentionally avoids introducing a separate sophisticated method-override variance calculus in this part.

## 22.5 Return type

Case implementation return type must be assignable/subtype-compatible with the specialized root return type.

## 22.6 Unknown and Dynamic

Unknown declaration facts cannot count as proof of compatibility.

An explicit `Dynamic` boundary remains an explicit dynamic boundary, not an established proof.

Requirement status should become `Blocked` or emit a targeted incomplete/incompatible diagnostic rather than laundering uncertainty into success.

---

# 23. Case-Local Behavior Rules

Per-variant bodies may declare instance behavior.

They may:

- implement a root requirement;
- override compatible root default behavior;
- add new case-specific behavior.

In v1:

- case-local behavior is instance-side only;
- `@class`/static behavior inside a variant body is rejected;
- case-local constructor declarations are rejected;
- bodyless declarations inside a concrete variant are rejected.

The exact case is concrete and final; abstract behavior belongs at the enum root.

This keeps associated declarations owned by the enum root and avoids an inaccessible "class side of an exact case" source model.

---

# 24. Payload Fields Inside Case Behavior

Each constructor payload parameter creates immutable case data described by `VariantFieldSemantic`.

Part 2 must make these fields available to case-local behavior checking as receiver-owned case fields.

This does **not** commit to a public source syntax for arbitrary exact-case field access.

It provides enough semantic identity for:

- case-local method bodies;
- future pattern projection in Part 5;
- payload-access visibility;
- future reflection in Part 6.

No ordinary source `FieldId` owned by a fake case `DeclarationId` is required.

---

# 25. Exact-Case Behavioral Inheritance Without Synthetic Classes

Exact static case types act like final subclasses of the enum root for behavior, but they are not represented as ordinary nominal source declarations.

Extend semantic dispatch support with a case behavior registry keyed by compact variant handle:

```rust
case_surfaces: HashMap<VariantTypeId, MemberSurface>,
case_roots: HashMap<VariantTypeId, DeclarationId>,
```

Add an exact-case-aware dispatch entry point conceptually like:

```rust
fn resolve_exact_case_dispatch(
    &self,
    variant: VariantTypeId,
    enum_receiver: TypeId,
    selector: &Selector,
    lookup: DispatchLookup,
) -> DispatchResult;
```

Resolution order for an exact case is:

```text
case-local instance behavior
    ↓ if exact selector absent
enum-root instance behavior
    ↓
normal enum-root behavioral inheritance
```

The `enum_receiver` carries generic specialization, so root/case signatures can be specialized without creating a surface for every `Option<Int>`, `Option<String>`, etc.

No associated variant declaration participates in dot/message dispatch.

---

# 26. Associated Owner Surface and Family Reservation

This is the key declaration rule needed by Part 3.

There are three conceptual spaces:

```text
1. instance behavioral surface
   value.name(...)

2. class-side/type behavioral declaration surface
   behavior that may be reified/narrowed through `Type::name` family/member forms

3. direct associated declaration surface
   variants and future associated data
```

Only 2 and 3 compete for the **associated family namespace**.

Instance behavior does not.

Therefore an enum may have an instance method with the same base spelling as a variant family without semantic ambiguity, because dot and double-colon are different language operations.

A class-side behavioral family and a variant family with the same base on the same effective associated owner surface conflict.

---

# 27. Associated Surface Model

Introduce:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AssociatedFamilyKind {
    Behavioral,
    Variant,
}
```

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssociatedMemberId {
    Behavioral(CallableId),
    Variant(VariantId),
}
```

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedFamilyInfo {
    pub id: AssociatedFamilyId,
    pub kind: AssociatedFamilyKind,
    pub members: Box<[AssociatedMemberId]>,
}
```

and:

```rust
#[derive(Clone, Debug, Default)]
pub struct AssociatedSurface {
    pub owner: DeclarationId,
    pub families: BTreeMap<SelectorBase, AssociatedFamilyInfo>,
}
```

A snapshot-level aggregate may be:

```rust
pub struct AssociatedFamilyTable {
    by_owner: HashMap<DeclarationId, AssociatedSurface>,
}
```

Part 2 publishes this table.

Part 3 consumes it.

---

# 28. Family Reservation Rules

## 28.1 Base name reserves the family

Within one effective associated owner surface, exact-selector difference does not permit mixing categories.

Illegal:

```text
variant family Error
behavioral family Error
```

even if members are:

```text
#Error(_)
#Error()
#Error
```

## 28.2 Multiple exact members of one family category are legal

Variant:

```phalcom
@variant None
@variant None()
@variant None(_ value: Int)
```

is legal.

Behavioral getter/method/setter members of one behavioral family may also coexist when their selectors are otherwise legal.

## 28.3 Duplicate exact variant selector is illegal

Two declarations producing the same `VariantId` are duplicates.

## 28.4 Behavioral inheritance reserves families

A behavioral family inherited through the ordinary source declaration hierarchy reserves its base on a descendant's effective associated surface.

## 28.5 Behavioral override/extension is legal

A descendant may override an inherited exact behavioral member or add another exact member to the same behavioral family.

That does not change the family category.

## 28.6 Direct associated declarations do not inherit

Variant families are owned by their enum declaration.

They are not copied to descendants.

Enums themselves are closed and cannot be subclassed to add cases.

## 28.7 Private behavioral members

Only behavior that is semantically inherited participates in inherited reservation.

A non-inherited private member does not reserve a descendant family merely because it exists in an ancestor's source.

Within its own declaration, it still occupies its base.

## 28.8 Metaclass/universal class-object behavior is not declaration inheritance

Universal runtime class-object behavior must not accidentally reserve every such family name on every enum.

Associated family inheritance follows source declaration behavioral inheritance, not incidental metaclass implementation detail.

---

# 29. `owner::name` Still Means Exact Getter

Family publication must not reinterpret syntax.

Even if:

```text
AssociatedFamilyId(Option, "Some")
```

exists, and contains only:

```text
#Some(_)
```

then:

```phalcom
Option::Some
```

does **not** mean the whole family.

Part 3 must report missing exact getter for that use.

Whole family remains:

```phalcom
Option::Some::*
```

Part 2 only publishes the facts required to make that deterministic.

---

# 30. Variant Visibility

Variant visibility is not a single bit.

Define:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VariantVisibility {
    pub name: MemberVisibility,
    pub construct: MemberVisibility,
    pub payload: MemberVisibility,
}
```

Semantic meanings:

- `name`: may the case identity be named/matched/discovered?
- `construct`: may source obtain the case through its associated producer surface? For constructor-shaped variants this means constructor reference/invocation; for a singleton it gates obtaining the singleton through `Enum::Case`.
- `payload`: may payload data be projected/accessed?

The model must exist now even though every future source spelling is not settled.

## 30.1 v1 `@private` rule

To realize the ratified "private variant constructors can match but cannot construct" requirement:

```phalcom
@private
@variant Hidden(_ value: Int)
```

means, by default:

```text
name      = Public
construct = Private
payload   = Public
```

A private constructor does not make the case absent from exhaustiveness or closed-enum requirements.

## 30.2 `@protected`

Because enums are closed and not subclass-extensible, `@protected` on a variant is not useful as a construction policy in v1. Reject it unless a newer normative design explicitly defines a meaning.

## 30.3 Payload-restriction syntax

No new payload-privacy syntax is invented in Part 2.

The axis exists and defaults to public. A later surface decision can map an attribute onto it without redesigning semantic identity.

---

# 31. Enum Closedness

Enums are closed nominal sums.

Part 2 must reject or structurally make impossible:

- source subclassing of an enum root;
- adding variants in another declaration;
- extending the case set through class inheritance;
- treating variant families as inheritable declarations.

Exact cases are final.

Protocols/traits/conformance are a separate concern and are not introduced here.

---

# 32. Module Interface and Declaration Shell Integration

Part 1 adds `Statement::Enum`.

Part 2 must update `phalcom-modules::InterfaceBuilder` so an enum name is collected as an immutable declaration exactly like a class name for module namespace/import/export purposes.

`SemanticWorkspaceSession` predeclaration must create:

```rust
DeclarationBlueprint {
    id: enum_decl_id,
    kind: DeclarationKind::Adt,
}
```

The enum root receives an ordinary `DeclarationTypeInfo`:

- normal nominal form;
- normal class-object type if the language represents type declarations as class objects at value level;
- normal generic signature;
- no source superclass template.

No exact case receives a module-level `DeclarationId`.

---

# 33. Incremental Semantic DB Products

Part 2 must make enum semantics explicit DB products.

Recommended keys:

```rust
QueryKey::EnumDeclaration(DeclarationId)
QueryKey::EnumRequirements(DeclarationId)
QueryKey::AssociatedSurface(DeclarationId)
```

Recommended products:

```rust
SemanticProduct::EnumDeclaration(Arc<EnumDeclarationProduct>)
SemanticProduct::EnumRequirements(Arc<EnumRequirementsProduct>)
SemanticProduct::AssociatedSurface(Arc<AssociatedSurfaceProduct>)
```

with:

```rust
pub struct EnumDeclarationProduct {
    pub info: Arc<EnumInfo>,
    pub variants: Arc<[VariantInfo]>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}
```

`EnumDeclarationProduct` is deliberately **structural**: it owns variant identity, payload/result semantics, exact-case templates, constructor signatures, visibility, and GADT case environments. It does not own root/case behavioral callable signatures or requirement-satisfaction results. Those are separate query products so executable-body and behavioral-signature edits do not unnecessarily redefine the enum's case structure.

Closed-enum obligations are published separately:

```rust
pub struct EnumRequirementsProduct {
    pub owner: DeclarationId,
    pub requirements: Arc<[EnumRequirement]>,
    pub case_statuses: Arc<[CaseRequirementResult]>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}
```

and:

```rust
pub struct AssociatedSurfaceProduct {
    pub surface: Arc<AssociatedSurface>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}
```

A single enum declaration product is preferable to one DB product per variant in v1. It keeps the dependency graph understandable while still allowing semantic fingerprint stability.

Part 3 can add narrower query keys only if measured invalidation pressure requires them.

---

# 34. Query Dependencies

Conceptual dependencies:

```text
ParsedModule
   ↓
Unlinked/LinkedInterface
   ↓
DeclarationShell(enum root)
   ↓
EnumDeclaration(enum root)
   ├── variant selectors
   ├── payload/result annotations
   ├── GADT case environments
   ├── constructor signatures
   └── exact-case templates
   ↓
root/case CallableSignature products
   ├── EnumRequirements(enum root)
   │      └── requirement compatibility/status diagnostics
   ├── case/root CallableBody products
   └── DeclarationSurface(enum root behavior)

DeclarationSurface(owner)
   ↓
AssociatedSurface(owner)
   ↑
EnumDeclaration(owner, if enum)
   ↑
superclass AssociatedSurface/behavioral surface, for classes only
```

Case body analysis must record a dependency on the enum declaration semantic product so a changed GADT equality or payload/result signature invalidates the relevant case body.

---

# 35. Fingerprinting and Incremental Stability

Enum declaration semantic fingerprints must include:

- owner identity;
- enum generic signature semantic identity;
- variant exact selectors;
- variant shapes;
- payload type facts;
- result type templates;
- exact-case templates;
- case equality environments;
- constructor signatures;
- visibility.

They must **not** include root/case behavioral callable signatures, requirement-satisfaction results, or executable body contents. Those have their own query products.

`EnumRequirementsProduct` fingerprints include the canonical root requirement signatures and the exact case implementation signatures/statuses they compare, but not executable method bodies.

This allows:

```text
case body implementation edit
```

to invalidate/recompute the callable body without needlessly changing the enum declaration product when its semantic signature is unchanged.

Source range movement should not change the range-free formal product fingerprint.

Diagnostics/source provenance may be retained alongside the product using the existing declaration-surface pattern.

---

# 36. Immutable Snapshot Publication

`SemanticSnapshot` should gain canonical read-only products such as:

```rust
pub enum_semantics: Arc<EnumSemanticTable>,
pub enum_requirements: Arc<EnumRequirementTable>,
pub associated_families: Arc<AssociatedFamilyTable>,
```

No request-time AST re-analysis should be required to answer:

- enumerate variants;
- find variant by selector;
- find family by base;
- get exact-case template;
- get GADT equalities;
- inspect variant visibility;
- inspect constructor signature;
- inspect enum requirement status.

This is compiler semantic state, not LSP state.

---

# 37. Source Index Integration

Extend:

```rust
SemanticTargetId
```

with:

```rust
Variant(VariantId)
```

and, where useful later:

```rust
VariantField(VariantFieldId)
```

Part 2 requires at least exact variant declaration/name targeting.

The variant-name source occurrence must resolve to the exact `VariantId`, not merely the enum root or base family.

This preserves the crucial distinction between:

```text
#None
#None()
#None(_)
```

even when all have the same textual base name.

---

# 38. Semantic Diagnostics

Add dedicated diagnostic codes. Recommended set:

```text
enum.variant.duplicate
enum.variant.result_wrong_owner
enum.variant.result_unsaturated
enum.variant.result_invalid
enum.variant.gadt_cyclic_equality
enum.variant.visibility_invalid
enum.variant.case_static_behavior_unsupported

enum.family.category_conflict
enum.family.inherited_behavior_conflict

enum.requirement.incomplete
enum.requirement.missing
enum.requirement.incompatible
enum.case.declaration_only_behavior
```

Use existing generic kind/application diagnostics where they already precisely describe type-annotation failures.

Diagnostics must distinguish:

- duplicate exact selector;
- legal same-base overload;
- category conflict at base-family level;
- missing implementation;
- implementation present but type-incompatible;
- requirement unable to be checked because declaration types are unresolved.

---

# 39. Diagnostic Evidence Policy

The checker remains authoritative.

A user annotation cannot override a proven enum/variant semantic fact.

Examples:

- a variant's result owner is proven from the declaration; annotating an unrelated root is an error;
- a constructor's exact case result is declaration semantics, not a developer guess;
- a case equality derived from a valid GADT result is established semantic evidence;
- unresolved annotation evidence remains unknown and cannot be laundered into a successful requirement proof.

Use existing `EvidenceOrigin::DeclarationSemantics` and `ConstructorSemantics` where appropriate.

A future dedicated `VariantSemantics` evidence origin may be added if diagnostics benefit, but it is not required merely to implement the model.

---

# 40. Type Presentation

The exact-case type is canonical even though source syntax for naming it is unresolved.

`TypeStore::format_type` must become exhaustive for `ExactCase`.

Until a source-level exact-case type spelling is ratified, use an explicitly non-source internal form such as:

```text
ExactCase<Expr::Int(_), Expr<Int>>
```

for low-level debug/semantic presentation.

Do not invent:

```text
dot-qualified exact-case type spellings
`Expr::Int<T>`-style invented exact-case type syntax
```

as source type syntax.

Part 8 can improve diagnostic prose without changing identity.

---

# 41. Metadata and Advisory Staging

Any current exhaustive `TypeData` consumer must be audited.

For subsystems that do not yet have an exact-case representation:

- formal semantic code must preserve `ExactCase`;
- advisory/runtime-shape code may conservatively project an exact case to its enum root shape until Part 4;
- external metadata serialization may conservatively widen to the enum root until Part 6 defines exact-case reflection metadata.

A consumer must not panic or reinterpret an exact case as an unrelated nominal class.

---

# 42. Performance Requirements

## 42.1 Compile-time storage

Hot exact-case key:

```text
VariantTypeId(u32) + TypeId(u32)
```

is compact.

`VariantId` remains stored once in the variant interner/table rather than copied into every exact type.

## 42.2 Lazy specialization

Do not precompute:

```text
variants × every observed generic specialization
```

Only materialize exact specialized types when demanded.

## 42.3 No runtime implication

`TypeData::ExactCase` is compiler metadata.

It does not require a runtime object to contain:

- `TypeId`;
- `VariantId`;
- a heap box;
- a type descriptor.

Part 4 remains free to use immediate values, tagged payloads, unboxed storage, singleton immediates, or optimized erasure boundaries.

## 42.4 No synthetic declaration explosion

Do not create one full `DeclarationId` + `DeclarationSurface` + nominal form + class-object form for every exact case merely to reuse the class model.

Case behavior is keyed directly by variant identity.

---

# 43. Required TypeData Exhaustiveness Audit

Adding `TypeData::ExactCase` requires an exhaustive repository audit.

At minimum verify and update:

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/advisory/formal.rs
phalcom-semantic/src/metadata/export.rs
```

and every other `match TypeData` site found by repository search.

Generic inference should not silently strip exactness. If an exact-case type contains generic parameters through its `enum_type`, inference/substitution must be able to recurse through that component.

A clean extension is:

```rust
InferenceTerm::ExactCase {
    variant: VariantTypeId,
    enum_type: Box<InferenceTerm>,
}
```

so parameter inference can see `Option<T>` inside `ExactCase(Some, Option<T>)`.

---

# 44. Requirements for `Self` in Case Behavior

Inside case-local behavior, `Self` should denote the exact case type template, not merely the enum root.

For:

```phalcom
@variant Int(_ value: Int) -> Expr<Int> {
    clone -> Self { ... }
}
```

the lexical `Self` conceptually denotes:

```text
ExactCase(Expr::Int(_), Expr<Int>)
```

This preserves precision.

The enclosing enum declaration remains available separately for inherited behavior and type-parameter scope.

---

# 45. Associated Behavior vs Variant Constructors

A variant constructor must not appear in:

```text
DeclarationSurface.class.callables
SurfaceDispatchResolver ordinary class-side dispatch
CallableSignatureTable as a method
```

Its declaration lives in enum semantics / associated surface.

A class-side behavioral method *does* remain an ordinary behavioral callable and may additionally be listed as a member of an associated behavioral family for `::` reification.

Thus the same canonical method identity can participate in:

```text
behavioral dispatch semantics
associated family publication
```

without changing the fact that `::` lookup itself is not message dispatch.

---

# 46. Detailed Examples

## 46.1 Ordinary generic ADT

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

Semantic result:

```text
root form:
    Option : Type -> Type

default result:
    Option<T>

variant:
    #Some(_)
    shape = Constructor
    field[0] = T
    result template = Option<T>
    exact template = ExactCase(Some, Option<T>)

variant:
    #None
    shape = Singleton
    fields = []
    result template = Option<T>
    exact template = ExactCase(None, Option<T>)
```

At future call specialization:

```text
T := Int
```

materializes:

```text
Option<Int>
ExactCase(Some, Option<Int>)
```

## 46.2 Singleton and empty-object factory

```phalcom
enum Marker {
    @variant Canonical
    @variant Fresh()
}
```

```text
#Canonical:
    singleton

#Fresh():
    zero-arg VariantConstructor
```

They are not interchangeable.

## 46.3 Same-base family

```phalcom
enum Example {
    @variant None
    @variant None()
    @variant None(_ value: Int)
}
```

```text
family:
    VariantFamilyId(Example, "None")

members:
    VariantId(Example, #None)
    VariantId(Example, #None())
    VariantId(Example, #None(_))
```

No duplicate error.

## 46.4 GADT

```phalcom
enum Expr<T> {
    eval -> T

    @variant Int(_ value: Int) -> Expr<Int> {
        eval -> Int { value }
    }

    @variant Bool(_ value: Bool) -> Expr<Bool> {
        eval -> Bool { value }
    }
}
```

`Int`:

```text
result = Expr<Int>
equality = T ≡ Int
exact = ExactCase(Int, Expr<Int>)

specialized requirement:
    eval -> Int
```

`Bool` similarly.

## 46.5 Multi-parameter equality

```phalcom
enum Equal<A, B> {
    @variant Refl(_ value: A) -> Equal<A, A>
}
```

The second result argument establishes:

```text
B ≡ A
```

The case environment keeps `A` lexical and rewrites `B` to `A`.

## 46.6 Illegal wrong result owner

```phalcom
enum Expr<T> {
    @variant Bad(_ value: Int) -> Option<Int>
}
```

Rejected before exact-case publication.

## 46.7 Family category conflict

Conceptually:

```phalcom
enum Response {
    @variant Error(_ reason: String)

    @class
    Error(_ code: Int) -> Response {
        ...
    }
}
```

Both occupy the associated base `Error` on `Response`.

Rejected as category conflict.

An *instance* method `Error(...)` would not create that conflict because it belongs to dot/message-send behavior.

---

# 47. Interaction With Part 3

Part 3 will consume:

```text
EnumSemanticTable
AssociatedFamilyTable
VariantConstructorSignature
VariantInfo
VariantVisibility
TypeData::ExactCase
CaseTypeEnvironment
```

Part 3 should not need to rediscover:

- variant selectors from AST;
- which members share a family;
- whether a family is variant or behavioral;
- GADT result equalities;
- whether a constructor is private;
- exact-case result templates.

That information must already be canonical Part 2 semantic state.

---

# 48. Interaction With Part 4

Part 4 maps:

```text
VariantId
VariantTypeId
exact case type
singleton/constructor shape
payload fields
```

to runtime metadata/layout.

Part 4 may assign physical discriminants, but:

```text
physical discriminant != VariantId
```

and:

```text
runtime case class != exact static type
```

remain invariant.

---

# 49. Interaction With Part 5

Part 5 match elimination will use:

```text
VariantId
VariantFieldId
exact case type
CaseTypeEnvironment
enum closed case set
VariantVisibility.name/payload
```

A successful case test refines the scrutinee to the exact case and introduces the already-published GADT equality facts into the branch.

---

# 50. Interaction With Part 6

Part 6 will:

- migrate core enums/`Option`;
- define reflection metadata;
- remove legacy sealed-class variant expansion;
- finalize core runtime/native integration;
- clean old method-family compatibility machinery where superseded.

Part 2 should not prematurely delete legacy runtime support required for the current workspace to build before later migration.

---

# 51. Explicitly Resolved Part 2 Decisions

This specification resolves the following internal-design choices.

1. `Expr<Int>` uses ordinary canonical `Applied`.
2. Exact cases use canonical `TypeData::ExactCase`.
3. Exact type identity is variant + canonical enum result type.
4. Stable `VariantId` remains structural.
5. Hot type data uses compact `VariantTypeId`.
6. Specialized exact cases are lazy.
7. Exact-case unions are not globally widened by canonicalization.
8. Enum root is `DeclarationId`, declaration kind `Adt`.
9. No synthetic ordinary declaration per exact case.
10. Per-case behavioral callables use a generalized callable owner.
11. Variant constructors use their own identity and are not methods.
12. GADT result equality uses canonical type terms and a case equality environment.
13. Case equality has an occurs check and transitive normalization.
14. Family namespace is distinct from instance message-send behavior.
15. Universal associated family identity uses `SelectorBase`.
16. Class-side behavioral families and direct variant families compete on the associated surface.
17. Associated declarations do not inherit.
18. Inherited behavioral families reserve names on descendants.
19. `@private` on a variant restricts construction while preserving matching/name visibility.
20. Exact case source type syntax remains deferred.
21. Case-local static behavior is deferred/rejected in v1.
22. Bodyless case-local behavior is rejected.
23. Signature-only root behavior is a closed-enum requirement.
24. Bodyful root behavior is inherited default behavior.
25. GADT equality is active during case declaration/body checking.

---

# 52. Deferred Decisions

The following are deliberately not decided in Part 2 because they do not block the semantic model.

- public source syntax for exact-case types;
- public payload-access restriction attribute syntax beyond current construction privacy;
- variant-local existential/generic binders;
- open/extensible enums;
- runtime representation;
- discriminant numbering;
- reflection API spelling;
- whether exact-case presentation receives a dedicated user-friendly formatter;
- final general callable taxonomy refactor beyond what case behavior requires;
- optimizer widening heuristics for exact-case unions;
- associated lookup ranking/invocation behavior, which belongs to Part 3.

---

# 53. Required Test Matrix

Part 2 is not complete without tests covering all of the following.

## 53.1 Module/declaration integration

- enum root collected by module interface;
- enum root importable/exportable;
- enum/class top-level duplicate name rejected;
- enum declaration shell uses `DeclarationKind::Adt`;
- generic enum obtains canonical declaration generic signature.

## 53.2 Identity

- `#None`, `#None()`, `#None(_)` produce distinct `VariantId`s;
- all share one `VariantFamilyId`;
- duplicate exact selector rejected;
- same exact selector under different enum owner is distinct.

## 53.3 Canonical exact types

- repeated `Expr<Int>` returns same `TypeId`;
- repeated same exact case returns same `TypeId`;
- different variant same enum type differs;
- same variant different specialization differs;
- invalid owner mismatch rejected;
- exact type has kind `Type`;
- compact variant identity survives `TypeStore` clone/snapshot semantics.

## 53.4 Type relation

- exact case subtypes enum specialization;
- exact case subtypes `Object` through root;
- exact case does not subtype wrong generic specialization;
- different exact cases remain distinct;
- unions preserve subset exactness.

## 53.5 Substitution/inference

- `ExactCase(Some, Option<T>)` specializes to `ExactCase(Some, Option<Int>)`;
- `TypeView` materializes exact case recursively;
- generic inference can see parameters inside exact case enum type;
- no exhaustive `TypeData` consumer panics.

## 53.6 GADT

- ordinary missing result yields default root application;
- `Expr<Int>` result creates `T ≡ Int`;
- transitive equality normalization works;
- parameter-to-parameter equality works;
- cyclic equality rejected;
- wrong result owner rejected;
- unsaturated result rejected;
- kind mismatch uses existing kind/application diagnostics.

## 53.7 Constructors

- singleton has no constructor signature;
- `None()` has zero-arg constructor signature;
- payload variant has ordered typed parameters;
- constructor result template is exact case;
- private construction state preserved.

## 53.8 Requirements

- signature-only root requirement satisfied by every case;
- missing implementation diagnosed;
- incompatible return diagnosed;
- incompatible parameter diagnosed;
- bodyful root default satisfies cases without override;
- compatible case override accepted;
- GADT specialization validates `eval -> T` against `eval -> Int`;
- singleton case is included in requirement checks;
- private-construction case is included;
- declaration-only case behavior rejected.

## 53.9 Families

- variant overloads share one family;
- instance behavior same base does not conflict;
- class-side behavior same base conflicts with variant family;
- inherited behavioral family reserves descendant base;
- same-kind inherited behavioral override/extension allowed;
- non-inherited private ancestor behavior does not reserve descendant;
- direct associated variants are not inherited.

## 53.10 Incremental DB

- enum semantic product is reusable when case body text changes but semantic declaration signature does not;
- changing variant selector changes enum product fingerprint;
- changing GADT result changes enum product fingerprint and invalidates case body dependency;
- changing class-side behavioral family affects associated surface;
- source-range-only movement does not change range-free semantic fingerprint.

## 53.11 Source identity

- variant declaration occurrence targets exact `SemanticTargetId::Variant`;
- same base overloaded variants retain different exact targets.

---

# 54. Acceptance Criteria

Part 2 is complete when all of the following are true.

1. `phalcom-semantic` can publish an immutable `EnumInfo` for every valid enum.
2. Every variant has a stable `VariantId`.
3. Every variant has a stable family identity.
4. `@variant None` and `@variant None()` remain distinct.
5. Constructor-shaped variants publish `VariantConstructorSignature`; singleton variants do not.
6. Payload fields have stable identities and canonical type facts.
7. Every variant has a canonical result type template.
8. Every variant has a canonical exact-case type template.
9. `Expr<Int>` uses normal canonical generic application.
10. Exact cases are canonical `TypeId`s and subtype their enum root specialization.
11. GADT case equalities are normalized, cycle-checked, and published.
12. GADT equality is available during case behavior/requirement checking.
13. Enum-root shared/default behavior is modeled as behavior.
14. Signature-only root behavior creates closed-enum obligations.
15. Requirement checks are selector-exact and GADT-aware.
16. Case methods have canonical callable identity without fake case declarations.
17. Variant constructors are not ordinary methods.
18. Associated family reservation is explicit and deterministic.
19. Class-side behavioral and variant families of the same base conflict on the same effective associated surface.
20. Instance methods do not collide merely because a variant has the same base.
21. Associated declarations do not inherit.
22. Behavioral family inheritance reservation works.
23. Variant visibility has three semantic axes.
24. Enum roots are module declarations and `DeclarationKind::Adt` shells.
25. Exact variant names participate in source indexing.
26. Enum/associated semantic products participate in DB fingerprinting and snapshots.
27. No new semantic resolver exists in `phalcom-lsp`.
28. No runtime layout/boxing decision is hard-coded.
29. Workspace tests, focused semantic tests, formatting, and clippy pass, or unrelated baseline failures are separately documented.

---

# 55. Architectural Invariant at End of Part 2

The implementation should now satisfy:

```text
Declaration identity
--------------------
Enum root                 -> DeclarationId
Variant family            -> VariantFamilyId
Universal family          -> AssociatedFamilyId
Exact variant             -> VariantId
Payload field             -> VariantFieldId
Variant constructor       -> VariantConstructorId
Case behavior method      -> CallableId(owner = Variant)

Canonical types
---------------
Enum<T>                   -> normal Nominal/Applied TypeId
Exact case                -> ExactCase(VariantTypeId, Enum<T>)
GADT equality             -> CaseTypeEnvironment, not type identity

Behavior
--------
enum root bodyful member  -> inherited default behavior
enum root declaration     -> closed-enum requirement
case member               -> exact-case behavior/override

Associated namespace
--------------------
class-side behavior        -> associated behavioral family
variant declaration        -> associated variant family
instance behavior          -> NOT in associated reservation namespace

Runtime
-------
not decided here
```

That is the semantic foundation Part 3 should consume.
