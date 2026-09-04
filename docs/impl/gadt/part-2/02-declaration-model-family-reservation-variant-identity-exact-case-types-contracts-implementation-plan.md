# Phalcom ADT/GADT + Associated Lookup
## Part 2 — Declaration Model, Family Reservation, Variant Identity, Exact Case Types, and Closed-Enum Requirements
### Detailed Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Use test-driven development for semantic behavior changes and `superpowers:verification-before-completion` before claiming completion.

**Repository:** `aureat/phalcom-lang`  
**Verified planning baseline:** `1892bcff51f75dd2f3df2a0661b03371250d4090`  
**Baseline commit:** `docs(semantic): record correctness plans and authority audit`  
**Part:** 2 of 6  
**Target document path:** `docs/impl/adt-gadt-associated-lookup/part-2/02-declaration-model-family-reservation-variant-identity-exact-case-types-contracts-implementation-plan.md`

---

# 0. Implementation Preconditions

Part 2 depends on the Part 1 enum/associated AST implementation.

At the verified baseline, `main` still has no `Statement::Enum`. Before beginning:

- [ ] Fetch/rebase current `main`.
- [ ] Record the actual implementation baseline SHA.
- [ ] Confirm Part 1 has landed.
- [ ] Confirm the post-Part-1 AST names for:
  - [ ] `Statement::Enum`;
  - [ ] `EnumDef`;
  - [ ] `EnumMember`;
  - [ ] `EnumBehaviorMember`;
  - [ ] `VariantDecl`;
  - [ ] `VariantPayloadSyntax`;
  - [ ] `VariantBody`.
- [ ] Confirm `@variant None` is represented distinctly from `@variant None()`:
  - [ ] bare singleton => `payload: None`;
  - [ ] explicit zero-arg constructor => `payload: Some(... parameters: [])`.
- [ ] Confirm Part 1 did not lower new enum syntax through legacy `ClassMember::Variant`.
- [ ] Run the Part 1 focused tests before touching semantic code.
- [ ] If post-Part-1 names differ from this plan, update references mechanically without changing Part 2 semantics.

Do not implement Part 3 associated-expression resolution as part of this work.

---

# 1. Baseline Source Map

The plan is grounded in these verified current files/symbols.

```text
phalcom-modules/src/declaration.rs
    DeclarationKind::Adt
    DeclarationShellTable

phalcom-modules/src/interface.rs
    InterfaceBuilder::build
    currently collects Statement::Class, not enum

phalcom-semantic/src/identity.rs
    CallableId
    CallableParameterId
    FieldId
    SemanticTargetId
    SourceOwner

phalcom-semantic/src/types/id.rs
    TypeId
    ProperTypeId
    TypeParameterId

phalcom-semantic/src/types/store.rs
    TypeData
    TypeStore
    apply_type_form
    nominal_form
    nominal_type

phalcom-semantic/src/declarations.rs
    DeclarationTypeInfo
    DeclarationTypeTable

phalcom-semantic/src/types/parameter.rs
    TypeParameterOwner
    GenericConstraint
    GenericSignature
    TypeTerm

phalcom-semantic/src/types/annotation.rs
    resolve_type_form
    resolve_type_annotation
    resolve_generic_signature

phalcom-semantic/src/types/environment.rs
    TypeEnvironment
    TypeView

phalcom-semantic/src/types/substitution.rs
    TypeSubstitution
    substitution_for_applied

phalcom-semantic/src/types/relation.rs
    TypeHierarchy
    MapTypeHierarchy
    check_subtype_impl

phalcom-semantic/src/surface.rs
    MemberSurface
    DeclarationSurface

phalcom-semantic/src/dispatch.rs
    DispatchResolver
    SurfaceDispatchResolver
    CallableSignature

phalcom-semantic/src/checker/declaration_signature.rs
    callable_id_for_member
    semantic_signature_for_member
    project_semantic_signature

phalcom-semantic/src/signature.rs
    CallableSemanticSignature
    CallableSignatureTable

phalcom-semantic/src/checker/inference.rs
    InferenceTerm
    InferenceSession

phalcom-semantic/src/db/key.rs
    QueryKey

phalcom-semantic/src/db/product.rs
    SemanticProduct
    DeclarationSurfaceProduct

phalcom-semantic/src/db/query.rs
    query_declaration_shell
    query_declaration_surface
    query_callable_signature

phalcom-semantic/src/session.rs
    SemanticWorkspaceSession
    update_with_budget_and_cancel

phalcom-semantic/src/snapshot.rs
    SemanticSnapshot

phalcom-semantic/src/source_index/*
    SourceSemanticIndex
    SemanticTargetId integration

phalcom-semantic/src/diagnostic.rs
    DiagnosticCode

phalcom-semantic/src/presentation.rs
    TypePresenter

phalcom-semantic/src/contracts/spec.rs
    existing Design-by-Contract model; do not reuse name
```

---

# 2. Task 1 — Make Enums Real Module Declarations

**Files:**

```text
phalcom-modules/src/interface.rs
phalcom-modules/tests/... (existing interface test module/files)
phalcom-semantic/src/session.rs
```

## Goal

Make an enum root visible to module namespace/linking exactly as a type declaration, and predeclare it as `DeclarationKind::Adt`.

## Steps

- [ ] Add a failing module-interface test:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

export Option
```

Assert `Option` appears in the unlinked declaration namespace as immutable.

- [ ] Add duplicate namespace test:

```phalcom
class Result {}
enum Result {}
```

must be rejected by existing duplicate-declaration machinery.

- [ ] In `InterfaceBuilder::build`, extend pass 1:

```rust
Statement::Enum(enum_def) => {
    let range = (enum_def.range.start..enum_def.name_range.end).into();
    Self::validate_dunder(&enum_def.name, DunderRole::Binding, range)?;
    Self::collect_declaration(
        &enum_def.name,
        true,
        range,
        &mut namespace,
        &mut declarations,
    )?;
}
```

Adapt exact range construction to post-Part-1 AST.

- [ ] In `SemanticWorkspaceSession::update_with_budget_and_cancel`, update every declaration-predeclaration loop that currently matches only `Statement::Class`.

Prefer a helper:

```rust
enum SourceTypeDeclaration<'a> {
    Class(&'a ClassDef),
    Enum(&'a EnumDef),
}
```

or small functions:

```rust
fn source_declaration_name(stmt: &Statement) -> Option<(&str, SourceRange)>;
fn source_declaration_kind(stmt: &Statement) -> Option<DeclarationKind>;
fn source_generic_parameters(stmt: &Statement) -> Option<&[GenericParameterSyntax]>;
fn source_where_clause(stmt: &Statement) -> Option<&WhereClauseSyntax>;
```

Do not copy large class-only loops and create an enum-specific parallel pipeline.

- [ ] Predeclare enum roots with:

```rust
DeclarationBlueprint {
    id: decl_id,
    kind: DeclarationKind::Adt,
}
```

- [ ] Create normal enum root `DeclarationTypeInfo` with the same generic-kind logic as classes.
- [ ] Do not create a source superclass edge for enums.
- [ ] Resolve enum generic signatures with existing `resolve_generic_signature`.

## Tests

Run:

```bash
cargo test -p phalcom-modules
cargo test -p phalcom-semantic semantic::foundations::generics_core
```

## Acceptance

- [ ] enum names import/export/link like normal immutable declarations;
- [ ] enum roots use normal canonical nominal forms;
- [ ] enum shell kind is `Adt`;
- [ ] no variant receives a module declaration shell.

---

# 3. Task 2 — Add the Variant Identity Primitives Needed by Callable Ownership

**File:**

```text
phalcom-semantic/src/identity.rs
```

Add the minimum stable identity required before `CallableId` can own a case method:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantId {
    pub owner: DeclarationId,
    pub selector: Selector,
}
```

Tests first:

- [ ] same owner + same selector is equal;
- [ ] `#None`, `#None()`, `#None(_)` are distinct;
- [ ] same selector under different owner is distinct.

Do not add a runtime discriminant here.

---

# 4. Task 3 — Complete Variant/Family Identities and Generalize Callable Ownership

**Files:**

```text
phalcom-semantic/src/identity.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/*
phalcom-semantic/src/presentation.rs
phalcom-lsp/src/backend.rs            # mechanical compile adaptation only if needed
other files found by rg
```

## Complete identities

Add:

```rust
pub struct VariantFamilyId {
    pub owner: DeclarationId,
    pub base_name: Box<str>,
}

pub struct AssociatedFamilyId {
    pub owner: DeclarationId,
    pub base: SelectorBase,
}

pub struct VariantFieldId {
    pub variant: VariantId,
    pub index: u32,
}

pub struct VariantConstructorId {
    pub variant: VariantId,
}
```

Add helpers:

```rust
impl VariantId {
    pub fn family(&self) -> Option<VariantFamilyId>;
}

impl VariantFamilyId {
    pub fn associated(&self) -> AssociatedFamilyId;
}
```

## Generalize behavioral callable ownership

Case-local methods need unique canonical callable identity without fake declaration IDs.

Add:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CallableOwnerId {
    Declaration(DeclarationId),
    Variant(VariantId),
}
```

Change:

```rust
pub struct CallableId {
    pub owner: CallableOwnerId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

Provide compatibility constructors/helpers:

```rust
impl CallableId {
    pub fn method(owner: DeclarationId, selector: Selector, side: DispatchSide) -> Self;
    pub fn case_method(owner: VariantId, selector: Selector) -> Self;
    pub fn declaration_owner(&self) -> &DeclarationId;
    pub fn module(&self) -> &ModuleId;
}

impl CallableOwnerId {
    pub fn declaration(&self) -> &DeclarationId;
    pub fn module(&self) -> &ModuleId;
}
```

Keep `CallableId::new(DeclarationId, ...)` temporarily if it materially reduces churn, but route it to `CallableOwnerId::Declaration`.

`CallableSemanticSignature.owner` may remain the enclosing `DeclarationId` for compatibility; exact lexical behavior ownership is `signature.callable.owner`.

## Mechanical audit

Run:

```bash
rg -n '\.owner\.module|\.owner\.name|callable\.owner|signature\.owner' \
  phalcom-semantic phalcom-lsp phalcom-core
```

- [ ] replace enclosing-module uses with `callable.module()`;
- [ ] replace root-declaration uses with `callable.declaration_owner()`;
- [ ] update DB/source/presentation hashing deterministically;
- [ ] keep LSP changes mechanical—no variant resolver in LSP.

## Tests

- [ ] declaration-owned vs variant-owned callables are distinct;
- [ ] case callable ordering/hash is deterministic;
- [ ] all existing class callable identity tests remain valid;
- [ ] the three `None` selectors share one `VariantFamilyId`;
- [ ] same exact selector under different enum owner remains distinct.

---

# 5. Task 4 — Add Compact `VariantTypeId` and `TypeData::ExactCase`

**Files:**

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/mod.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
```

## Add compact ID

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantTypeId(pub u32);
```

Add `index()` / `from_index()` following existing ID conventions.

## Extend TypeStore

Fields:

```rust
variant_identities: Vec<VariantId>,
variant_identity_to_id: HashMap<VariantId, VariantTypeId>,
```

Initialize in `TypeStore::new`.

Methods:

```rust
pub fn intern_variant_identity(
    &mut self,
    variant: VariantId,
) -> VariantTypeId;

pub fn variant_identity(
    &self,
    id: VariantTypeId,
) -> &VariantId;
```

Add:

```rust
TypeData::ExactCase {
    variant: VariantTypeId,
    enum_type: TypeId,
}
```

## Exact-case API

Implement:

```rust
pub fn nominal_origin_declaration(
    &self,
    ty: TypeId,
) -> Option<&DeclarationId>;
```

It should peel nested `Applied` origins until `Nominal`.

Then:

```rust
pub fn exact_case_type(
    &mut self,
    variant: &VariantId,
    enum_type: TypeId,
) -> Result<TypeId, ExactCaseTypeError>;
```

Validation:

- [ ] `enum_type` kind is `KindId::TYPE`;
- [ ] enum type has nominal origin;
- [ ] nominal origin equals `variant.owner`.

Intern with `KindId::TYPE`.

## Canonicalization tests

Add tests:

```rust
let expr_int_1 = store.apply_type_form(expr_form, &[int]).unwrap();
let expr_int_2 = store.apply_type_form(expr_form, &[int]).unwrap();
assert_eq!(expr_int_1, expr_int_2);

let case1 = store.exact_case_type(&int_variant, expr_int_1).unwrap();
let case2 = store.exact_case_type(&int_variant, expr_int_2).unwrap();
assert_eq!(case1, case2);
```

Also:

- [ ] different variant same enum type differs;
- [ ] same variant different enum specialization differs;
- [ ] wrong owner rejected;
- [ ] exact case kind is `Type`;
- [ ] cloned store preserves handle denotation.

## Performance acceptance

- [ ] `TypeData::ExactCase` contains no `Selector`, `DeclarationId`, source range, equality set, runtime discriminant, or payload metadata;
- [ ] hot identity is `VariantTypeId + TypeId`.

---

# 6. Task 5 — Audit Every Canonical Type Consumer for `ExactCase`

**Files include, but are not limited to:**

```text
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/advisory/formal.rs
phalcom-semantic/src/metadata/export.rs
```

## Mandatory search

Run:

```bash
rg -n 'TypeData::|match .*store\.get|match .*TypeData' phalcom-semantic
```

Create a temporary checklist of every exhaustive match.

## TypeView

In `materialize_view`:

```rust
TypeData::ExactCase { variant, enum_type } => {
    let specialized = materialize_view(store, enum_type, env);
    store.intern_exact_case_handle(variant, specialized)
}
```

Expose a safe internal handle-based exact-case method after initial owner validation.

## TypeSubstitution

In `TypeSubstitution::apply`, recursively substitute `enum_type`.

In `specialize_self_type`, preserve the exact case while specializing its root component as needed.

## Relation

In `check_subtype_impl`:

```rust
(TypeData::ExactCase { enum_type, .. }, _) => {
    check_subtype_impl(
        store,
        hierarchy,
        enum_type,
        sup,
        budget,
        cancellation,
        visited,
    )
}
```

Keep the earlier `sub == sup` identity fast path.

Do not make `EnumType <: ExactCase`.

## Inference

Add:

```rust
InferenceTerm::ExactCase {
    variant: VariantTypeId,
    enum_type: Box<InferenceTerm>,
}
```

Update:

- [ ] `type_id_to_inference`;
- [ ] occurs checks;
- [ ] structural equivalence;
- [ ] materialization;
- [ ] term variable collection;
- [ ] kind handling.

The variant handle must match for two exact-case inference terms to unify as exact cases.

## Presentation

Update `TypeStore::format_type` or equivalent to print non-source internal syntax:

```text
ExactCase<Expr::Int(_), Expr<Int>>
```

Do not invent public source type syntax.

## Advisory / metadata

Until Parts 4/6:

- [ ] advisory shape may widen exact case to its `enum_type`;
- [ ] metadata export may emit enum-root type rather than exact-case reflection metadata;
- [ ] document this as staging behavior.

## Tests

- [ ] exact subtype;
- [ ] wrong specialization refutation;
- [ ] substitution;
- [ ] inference traversal;
- [ ] type formatting;
- [ ] no panic in advisory/metadata tests.

---

# 7. Task 6 — Introduce Enum Semantic Data Structures

**New file:**

```text
phalcom-semantic/src/enum_semantics.rs
```

**Modify:**

```text
phalcom-semantic/src/lib.rs
```

## Implement identities/data

```rust
pub enum VariantShape {
    Singleton,
    Constructor,
}
```

```rust
pub struct VariantVisibility {
    pub name: MemberVisibility,
    pub construct: MemberVisibility,
    pub payload: MemberVisibility,
}
```

```rust
pub struct VariantFieldSemantic {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub declared_type: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}
```

```rust
pub struct VariantConstructorParameter {
    pub field: VariantFieldId,
    pub external_label: Option<Box<str>>,
    pub local_name: Box<str>,
    pub declared_type: DeclaredTypeFact,
}
```

```rust
pub struct VariantConstructorSignature {
    pub constructor: VariantConstructorId,
    pub parameters: Box<[VariantConstructorParameter]>,
    pub result_type_template: TypeId,
    pub exact_case_template: TypeId,
    pub source: SemanticSourceSpan,
}
```

```rust
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
pub struct EnumSemanticTable { ... }
```

Prefer immutable/sorted boxed arrays in products and `HashMap` indexes in aggregate snapshot tables.

## Invariants

Constructor:

```rust
VariantShape::Singleton => constructor.is_none()
VariantShape::Constructor => constructor.is_some()
```

For `None()`:

```text
shape = Constructor
parameters = []
```

Add debug assertions and tests.

---

# 8. Task 7 — Implement GADT Case Equality Derivation

**New file recommended:**

```text
phalcom-semantic/src/types/case_environment.rs
```

or:

```text
phalcom-semantic/src/enum_semantics/gadt.rs
```

Do not bury this in parser code.

## Data

```rust
pub struct CaseTypeEnvironment {
    pub bindings: BTreeMap<TypeParameterId, TypeId>,
    pub equalities: Box<[GenericConstraint]>,
}
```

Add:

```rust
pub enum CaseEnvironmentError {
    ResultWrongOwner { ... },
    ResultUnsaturated { ... },
    ResultNotProper { ... },
    CyclicEquality { parameter: TypeParameterId, rhs: TypeId },
}
```

## Helper 1 — Build default result

```rust
fn default_enum_result_type(
    store: &mut TypeStore,
    info: &DeclarationTypeInfo,
) -> Result<TypeId, ...>
```

For generic enum:

1. get declaration form;
2. get declaration type parameter forms in order;
3. `apply_type_form(form, &parameter_forms)`;
4. require proper type.

## Helper 2 — Decompose explicit result

Add a reusable store helper:

```rust
pub fn applied_nominal_parts(
    &self,
    ty: TypeId,
) -> Option<(&DeclarationId, &[TypeId])>;
```

or return owned data to avoid borrow conflicts.

For a non-generic nominal root, return zero arguments.

## Helper 3 — Derive equations

Given root parameters and result args, derive:

```rust
GenericConstraint::Equivalent {
    left: TypeTerm::Canonical(parameter_form),
    right: TypeTerm::Canonical(result_arg),
}
```

## Helper 4 — Normalize bindings

Implement a declaration equality solver that:

- [ ] treats enum declaration parameters as variables;
- [ ] supports parameter-to-parameter equivalence;
- [ ] recursively rewrites RHS terms;
- [ ] performs occurs check;
- [ ] leaves unconstrained/self-equal parameters unchanged;
- [ ] deterministically chooses representatives;
- [ ] preserves equalities for proof/explanation.

Do not use call-site `InferenceOutcome::Underconstrained` as an error for unconstrained GADT parameters.

## Tests

```text
Expr<T> -> Expr<Int>      => T -> Int
Equal<A,B> -> Equal<A,A>  => B -> A
Pair<A,B> -> Pair<B,Int>  => A -> Int, B -> Int after normalization
Cycle<T> -> Cycle<List<T>> => cyclic equality error
```

Also test a non-GADT default result produces an empty environment.

---

# 9. Task 8 — Build Enum Declaration Semantics From AST

**New/modified:**

```text
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/checker/mod.rs
```

## Entry point

Implement conceptually:

```rust
pub fn build_enum_semantics(
    ctx: &mut EnumDeclarationContext<'_>,
    enum_def: &EnumDef,
) -> EnumDeclarationProduct;
```

Do not use ordinary class attribute expansion.

## Steps per enum

- [ ] obtain enum `DeclarationId`;
- [ ] obtain already-published `DeclarationTypeInfo`;
- [ ] build default result type;
- [ ] create enum-scoped type resolver from declaration generic signature;
- [ ] iterate variants in source order;
- [ ] derive structural selector using Part 1 helper;
- [ ] construct stable `VariantId`;
- [ ] detect duplicate exact variant selector;
- [ ] derive `VariantFamilyId`;
- [ ] map singleton/constructor shape from `payload: Option`;
- [ ] resolve explicit result annotation or use default;
- [ ] validate result owner/saturation/proper kind;
- [ ] derive `CaseTypeEnvironment`;
- [ ] intern `VariantTypeId`;
- [ ] build exact-case template;
- [ ] resolve payload field annotations under enum scope + case environment;
- [ ] build constructor signature only for constructor shape;
- [ ] derive variant visibility;
- [ ] retain source spans;
- [ ] collect root requirements/behavior signatures in later tasks.

## Crucial shape code

Use explicit syntax distinction, not parameter count:

```rust
let shape = match &variant.payload {
    None => VariantShape::Singleton,
    Some(_) => VariantShape::Constructor,
};
```

Never write:

```rust
if parameters.is_empty() { Singleton }
```

because that would collapse `None()` into `None`.

---

# 10. Task 9 — Add Enum DB Query/Product/Fingerprint

**Files:**

```text
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/mod.rs
```

## Query key

Add:

```rust
QueryKey::EnumDeclaration(DeclarationId)
```

## Product

Add:

```rust
pub struct EnumDeclarationProduct {
    pub info: Arc<EnumInfo>,
    pub variants: Arc<[VariantInfo]>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}
```

This product is structural. Do not include root/case behavioral signatures or requirement statuses; those depend on this product and are published separately.

Add:

```rust
SemanticProduct::EnumDeclaration(Arc<EnumDeclarationProduct>)
```

and accessor:

```rust
pub fn as_enum_declaration(
    &self,
) -> Option<&Arc<EnumDeclarationProduct>>;
```

Add query:

```rust
pub fn query_enum_declaration(
    db: &mut SemanticDb,
    ...
) -> QueryOutcome<Arc<EnumDeclarationProduct>>;
```

Dependencies:

- [ ] parsed module;
- [ ] linked interface;
- [ ] enum declaration shell;
- [ ] any type declarations referenced by resolved annotations through existing dependency recording model.

If current type-resolution queries do not record per-declaration dependencies at this layer, match existing `DeclarationSurface` dependency granularity rather than creating an inconsistent special case.

## Fingerprints

Add:

```rust
enum_declaration_input_fingerprint(...)
enum_declaration_product_fingerprint(...)
```

Semantic fingerprint includes variant structure, payload/result type facts, exact-case templates, GADT environments, constructor signatures, and visibility. It excludes root/case behavioral callable signatures, requirement status, and executable body AST.

Add incremental unit tests.

---

# 11. Task 10 — Publish Enum Semantics in SemanticWorkspaceSession

**File:**

```text
phalcom-semantic/src/session.rs
```

## Position in pipeline

After declaration shells/generic signatures are current and before case callable bodies are checked:

```text
DeclarationShell
    ↓
EnumDeclaration (structural case semantics)
    ↓
root/case CallableSignature products
    ├── EnumRequirements
    ├── AssociatedSurface
    └── case/root CallableBody products
```

## Implementation

- [ ] collect all enum products into an `EnumSemanticTable`;
- [ ] merge their diagnostics into `diags_by_module`;
- [ ] ensure a failure to resolve one enum does not fabricate variants;
- [ ] make case callable signature queries depend on current enum product.

Do not build enum semantics from request-time editor code.

---

# 12. Task 11 — Publish Variant Constructor Signatures

**Files:**

```text
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/checker/enum_declaration.rs
tests
```

## Rules

Bare singleton:

```phalcom
@variant None
```

```rust
constructor: None
```

Zero arg:

```phalcom
@variant None()
```

```rust
constructor: Some(
    VariantConstructorSignature {
        parameters: Box::new([]),
        ...
    }
)
```

Payload:

```phalcom
@variant Some(_ value: T)
```

has one constructor parameter tied to `VariantFieldId(0)`.

`result_type_template` is the enum specialization.

`exact_case_template` is the exact case.

## Evidence

Parameter source annotations remain developer declaration facts.

Constructor result is established by constructor/declaration semantics, not by a source-authored return guess.

Do not add variant constructors to ordinary class-side `DeclarationSurface`.

---

# 13. Task 12 — Publish Enum-Root and Case-Local Callable Signatures

**Files:**

```text
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/db/query.rs
```

## Refactor signature builder

Avoid duplicating all method/getter/setter signature logic.

Extract source-member-neutral helpers from `semantic_signature_for_member`:

```rust
struct BehavioralMemberView<'a> {
    selector: Selector,
    parameters: &'a [ParameterDef],
    return_annotation: Option<&'a TypeAnnotation>,
    generic_parameters: &'a [GenericParameterSyntax],
    where_clause: Option<&'a WhereClauseSyntax>,
    body: BehavioralBodyKind,
    attributes: &'a [Attribute],
    ...
}
```

or smaller per-kind helpers.

Both `ClassMember` and Part 1 `EnumBehaviorMember` should feed the canonical builder.

## Enum root

Owner:

```rust
CallableOwnerId::Declaration(enum_decl)
```

Instance/class side follows enum-root member syntax.

Bodyful instance member => shared/default behavior.

Declaration-only instance member => enum requirement (Task 15).

Bodyful class-side member => class-side behavioral callable and associated family member.

Declaration-only class-side enum behavior => reject in v1.

## Case body

Owner:

```rust
CallableOwnerId::Variant(variant_id)
```

Side:

```rust
DispatchSide::Instance
```

Reject:

- [ ] `@class` / static;
- [ ] constructor annotations;
- [ ] declaration-only body.

Resolve annotations under case environment.

## Tests

- [ ] root/case same selector have different `CallableId`;
- [ ] case signature sees GADT specialization;
- [ ] case generic method binder identity remains callable-owned;
- [ ] source spans attach to exact case callable.

---

# 14. Task 13 — Add Exact-Case Behavior Surfaces and Dispatch Inheritance

**Files:**

```text
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/surface.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
tests/semantic/capabilities/dispatch_capabilities.rs
```

## Resolver storage

Extend `SurfaceDispatchResolver`:

```rust
case_surfaces: HashMap<VariantTypeId, MemberSurface>,
case_roots: HashMap<VariantTypeId, DeclarationId>,
```

Only instance behavior is needed for cases.

Methods:

```rust
pub fn register_case_surface(
    &mut self,
    variant: VariantTypeId,
    root: DeclarationId,
    surface: MemberSurface,
);
```

Extend `DispatchResolver` with an exact-case path, or centralize in `CheckingContext`.

Recommended trait method:

```rust
fn resolve_exact_case_dispatch(
    &self,
    variant: VariantTypeId,
    enum_receiver: TypeId,
    selector: &Selector,
    lookup: DispatchLookup,
) -> DispatchResult;
```

Default can fall back to root dispatch if the implementation has no case surface.

`SurfaceDispatchResolver`:

1. exact selector in case surface;
2. otherwise enum-root instance behavior;
3. preserve normal generic specialization using `enum_receiver`.

## Central checker path

Ensure message-send synthesis checks:

```rust
match store.get(receiver_type) {
    TypeData::ExactCase { variant, enum_type } => {
        dispatch.resolve_exact_case_dispatch(...)
    }
    _ => dispatch.resolve_dispatch(...)
}
```

Prefer one helper in `CheckingContext` rather than duplicating this in every call path.

## Tests

- [ ] root method inherited by exact case;
- [ ] case override selected;
- [ ] new case-only method selected on exact type;
- [ ] root enum type does not see case-only method;
- [ ] case static surface does not exist.

This remains **dot/message-send behavior**, not `::` associated resolution.

---

# 15. Task 14 — Implement Enum Closed-Enum Requirements

**New file:**

```text
phalcom-semantic/src/enum_requirements.rs
```

**Modify:**

```text
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
```

## Incremental product

Add:

```rust
QueryKey::EnumRequirements(DeclarationId)
```

with:

```rust
pub struct EnumRequirementsProduct {
    pub owner: DeclarationId,
    pub requirements: Arc<[EnumRequirement]>,
    pub case_statuses: Arc<[CaseRequirementResult]>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}
```

and `SemanticProduct::EnumRequirements(...)`.

This product depends on structural `EnumDeclaration` plus the relevant root/case `CallableSignature` products. It must not depend on executable callable bodies.

## Data

```rust
pub struct EnumRequirementId {
    pub owner: DeclarationId,
    pub selector: Selector,
}
```

```rust
pub struct EnumRequirement {
    pub id: EnumRequirementId,
    pub signature: CallableSemanticSignature,
    pub source: SemanticSourceSpan,
}
```

```rust
pub enum CaseRequirementStatus {
    Satisfied { implementation: CallableId },
    Missing,
    Incompatible { implementation: CallableId },
    Blocked,
}
```

## Collection rule

For enum-root **instance** behavior:

```rust
MemberBody::Declaration => requirement
MemberBody::Block(_)     => shared/default behavior
```

Getter/setter declaration bodies may participate if Part 1 allows them.

`IndexMethodDef` currently uses `Vec<Statement>` rather than `MemberBody`; do not invent bodyless index syntax in Part 2.

## Validation

For each concrete variant and each requirement:

1. specialize root requirement under `variant.case_environment`;
2. look for exact selector on case behavior surface;
3. if absent => `Missing`;
4. compare generic binder shape;
5. compare rest shape;
6. compare parameters invariantly/equivalently;
7. compare return covariantly/assignably;
8. unresolved types => `Blocked`;
9. emit diagnostic.

Bodyful root default means no missing obligation when a case does not override it.

If a case overrides bodyful root behavior, run the same compatibility check.

## GADT test

Required:

```phalcom
enum Expr<T> {
    eval -> T

    @variant Int(_ value: Int) -> Expr<Int> {
        eval -> Int { value }
    }
}
```

must satisfy.

---

# 16. Task 15 — Check Case Bodies Under the GADT Environment

**Files:**

```text
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/session.rs
```

## Context state

Add an explicit current-case context rather than mutating global declaration generic metadata:

```rust
pub struct ActiveCaseContext {
    pub variant: VariantId,
    pub exact_case_template: TypeId,
    pub environment: CaseTypeEnvironment,
}
```

`CheckingContext` gets:

```rust
current_case: Option<ActiveCaseContext>
```

or an equivalent scoped guard.

## Type resolution

Case member annotations:

1. resolve in enum declaration generic scope;
2. apply case environment;
3. preserve source-declared fact/provenance where needed.

## `Self`

When current case exists, lexical `Self` must resolve/specialize to `exact_case_template`.

Do not globally redefine enum root `Self`.

## Payload fields

Bind receiver-owned payload field facts for case body checking using `VariantFieldId` / case field lookup.

Do not pretend they are locals created by the constructor invocation.

## Callable body query

Update callable lookup by `CallableOwnerId`:

```text
Declaration owner -> find class/enum-root member
Variant owner     -> find exact variant body member
```

Add DB dependency:

```rust
SemanticDependency::EnumDeclaration(enum_owner)
```

if `CallableAnalysis::SemanticDependency` is the current mechanism.

## Tests

- [ ] case body sees specialized `Self`;
- [ ] case body sees payload field type;
- [ ] case body type mismatch is diagnosed under specialized GADT types;
- [ ] changing GADT result invalidates/rechecks case body.

---

# 17. Task 16 — Implement Associated Family Publication and Reservation

**New file:**

```text
phalcom-semantic/src/associated.rs
```

**Modify:**

```text
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
```

## Data

```rust
pub enum AssociatedFamilyKind {
    Behavioral,
    Variant,
}
```

```rust
pub enum AssociatedMemberId {
    Behavioral(CallableId),
    Variant(VariantId),
}
```

```rust
pub struct AssociatedFamilyInfo {
    pub id: AssociatedFamilyId,
    pub kind: AssociatedFamilyKind,
    pub members: Box<[AssociatedMemberId]>,
}
```

```rust
pub struct AssociatedSurface {
    pub owner: DeclarationId,
    pub families: BTreeMap<SelectorBase, AssociatedFamilyInfo>,
}
```

```rust
pub struct AssociatedFamilyTable {
    by_owner: HashMap<DeclarationId, AssociatedSurface>,
}
```

## DB

Add:

```rust
QueryKey::AssociatedSurface(DeclarationId)
SemanticProduct::AssociatedSurface(Arc<AssociatedSurfaceProduct>)
```

## Direct behavioral members

Collect **class-side** behavioral callables from canonical callable signatures/surface.

Do not collect instance methods.

## Variant members

For enum owner, collect every `VariantId` from `EnumDeclarationProduct`.

Group by base.

## Conflict rule

When same base contains both categories:

```text
Behavioral + Variant => diagnostic
```

No fallback precedence.

## Inheritance

For class declarations:

1. get source superclass;
2. start with inherited *behavioral* families that are actually inherited;
3. apply direct behavioral overrides/extensions;
4. apply direct associated declarations;
5. conflict if a direct associated family tries to occupy inherited behavioral base.

Do **not** inherit variant/direct associated families.

Enums have no source subclass extension path.

## Tests

Create focused `associated_surface.rs` semantic tests:

- [ ] instance method base + variant base is legal;
- [ ] class-side method base + variant base conflicts;
- [ ] getter and zero-arg method same behavioral family legal;
- [ ] variant singleton/zero-arg/unary same family legal;
- [ ] inherited public behavioral family reserves descendant name;
- [ ] subclass behavioral override/extension legal;
- [ ] private non-inherited behavior does not reserve descendant;
- [ ] associated variant family is not inherited.

---

# 18. Task 17 — Implement Three-Axis Variant Visibility

**Files:**

```text
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/diagnostic.rs
```

## Defaults

```rust
VariantVisibility {
    name: Public,
    construct: Public,
    payload: Public,
}
```

## `@private`

Map variant-level `@private` to:

```text
name = Public
construct = Private
payload = Public
```

This implements "can match, cannot construct/obtain through the associated producer." For a constructor-shaped case it blocks constructor reference/invocation; for a bare singleton it blocks acquiring the singleton through `Enum::Case` while preserving name/match visibility.

## Other attributes

- [ ] reject unsupported `@protected` on variant in v1;
- [ ] preserve `@internal` only if current project visibility semantics provide a clear construction interpretation; otherwise emit a targeted unsupported-visibility diagnostic rather than guessing;
- [ ] do not invent payload-private source syntax.

## Tests

- [ ] private constructor still in enum case set;
- [ ] private constructor still subject to requirements;
- [ ] matching/name visibility remains public metadata;
- [ ] construct visibility private.

Part 3 will enforce construction visibility at associated invocation/reference sites.

---

# 19. Task 18 — Extend Source Semantic Targets for Variants

**Files:**

```text
phalcom-semantic/src/identity.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
```

## Identity

Add:

```rust
SemanticTargetId::Variant(VariantId)
```

Optionally add `VariantField` only if payload declaration/source targeting is implemented in this same change. Do not block variant targeting on it.

## Builder

When visiting enum syntax:

- [ ] enum name target => `Declaration(enum_decl_id)`;
- [ ] variant name target => exact `Variant(variant_id)`;
- [ ] overloaded same-base variants keep distinct exact targets by selector/source node.

Update target fingerprinting/hashing.

## LSP boundary

Any LSP match on `SemanticTargetId` may need a mechanical exhaustiveness update.

Do not add variant go-to-definition UI behavior here if that belongs to the later LSP part.

---

# 20. Task 19 — Add Enum/Associated Products to Immutable Snapshot

**Files:**

```text
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
tests/semantic/integration/workspace.rs
tests/semantic/incremental/*
```

## Add fields

```rust
pub enum_semantics: Arc<EnumSemanticTable>,
pub enum_requirements: Arc<EnumRequirementTable>,
pub associated_families: Arc<AssociatedFamilyTable>,
```

Update all snapshot constructors.

Prefer adding an input struct/builder if constructor arity becomes unmaintainable rather than appending positional arguments indefinitely.

## Accessors

```rust
pub fn enum_info(&self, owner: &DeclarationId) -> Option<&EnumInfo>;
pub fn variant(&self, id: &VariantId) -> Option<&VariantInfo>;
pub fn associated_surface(&self, owner: &DeclarationId) -> Option<&AssociatedSurface>;
```

## Tests

- [ ] snapshot can answer exact variant query without AST re-analysis;
- [ ] retained old snapshot preserves old `VariantTypeId` denotation after a rename in a newer revision;
- [ ] new snapshot receives new variant identity.

---

# 21. Task 20 — Add Enum-Specific Diagnostic Codes

**File:**

```text
phalcom-semantic/src/diagnostic.rs
```

Add variants and string codes:

```rust
EnumVariantDuplicate
EnumVariantResultWrongOwner
EnumVariantResultUnsaturated
EnumVariantResultInvalid
EnumVariantGadtCyclicEquality
EnumVariantVisibilityInvalid
EnumCaseStaticBehaviorUnsupported
EnumCaseDeclarationOnlyBehavior
EnumFamilyCategoryConflict
EnumFamilyInheritedBehaviorConflict
EnumRequirementIncomplete
EnumRequirementMissing
EnumRequirementIncompatible
```

Suggested strings:

```text
enum.variant.duplicate
enum.variant.result_wrong_owner
enum.variant.result_unsaturated
enum.variant.result_invalid
enum.variant.gadt_cyclic_equality
enum.variant.visibility_invalid
enum.case.static_behavior_unsupported
enum.case.declaration_only_behavior
enum.family.category_conflict
enum.family.inherited_behavior_conflict
enum.requirement.incomplete
enum.requirement.missing
enum.requirement.incompatible
```

Use existing annotation/kind/application diagnostics instead of duplicating them when the failure is already represented accurately.

Add unit test that every new code has stable `as_str()` output if such coverage exists.

---

# 22. Task 21 — Add Type/Enum Presentation Staging

**Files:**

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/advisory/formal.rs
```

## Exact type format

Implement deterministic internal formatting:

```text
ExactCase<Expr::Int(_), Expr<Int>>
```

This is not declared source syntax.

## Callable presentation

If `CallablePresentation::from_signature` assumes `signature.owner.name`, continue using enclosing enum declaration owner for case methods.

If exact owner information is needed, add a separate field later; do not stringify `CallableOwnerId` as source type syntax.

## Metadata/advisory

Use conservative enum-root projection where exact-case support is not yet a public contract.

Add comments referencing Part 4/6 staging.

---

# 23. Task 22 — Add Focused Semantic Test Suites

Recommended new files:

```text
phalcom-semantic/tests/semantic/foundations/enum_identity.rs
phalcom-semantic/tests/semantic/foundations/exact_case_types.rs
phalcom-semantic/tests/semantic/capabilities/enum_declarations.rs
phalcom-semantic/tests/semantic/capabilities/gadt_cases.rs
phalcom-semantic/tests/semantic/capabilities/enum_requirements.rs
phalcom-semantic/tests/semantic/capabilities/associated_surfaces.rs
```

Wire into the existing semantic test module structure.

## Required cases

### Identity

```phalcom
enum Example {
    @variant None
    @variant None()
    @variant None(_ value: Int)
}
```

Assert three variants, one variant family.

### Canonical types

Build `Option<Int>` twice, exact `Some` twice.

### GADT

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
}
```

Assert result and equality.

### Multi-parameter

```phalcom
enum Equal<A, B> {
    @variant Refl(_ value: A) -> Equal<A, A>
}
```

Assert `B -> A`.

### Cyclic

Use a result that forces `T = List<T>` and assert diagnostic.

### Requirement

```phalcom
enum Expr<T> {
    eval -> T

    @variant Int(_ value: Int) -> Expr<Int> {
        eval -> Int { value }
    }
}
```

Assert satisfied.

### Missing requirement

Add another case with no `eval`, assert diagnostic.

### Family conflict

Use class-side enum behavior with same base as variant, assert family-category diagnostic.

### Legal instance same-base

Use instance method with same base and assert no associated family conflict.

---

# 24. Task 23 — Add Incremental Regression Tests

**Files:**

```text
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/query_ownership.rs
```

or new dedicated enum incremental file.

## Cases

- [ ] first update publishes `QueryKey::EnumDeclaration`.
- [ ] unchanged source reuses product.
- [ ] edit only case method body expression:
  - callable body recomputes;
  - enum semantic product fingerprint remains stable when signature unchanged.
- [ ] change variant from `None` to `None()`:
  - enum product changes;
  - associated surface changes;
  - exact `VariantId` changes.
- [ ] change GADT result `Expr<Int>` to `Expr<Bool>`:
  - enum product changes;
  - case callable bodies depending on it invalidate.
- [ ] trivia/range movement does not change range-free semantic product fingerprint.
- [ ] retained old snapshot's exact type still formats/resolves to old variant identity.

---

# 25. Task 24 — Audit Legacy Variant Architecture Is Not Reused

**Files/search:**

```bash
rg -n 'ClassMember::Variant|VariantDef|Sealed|expand.*variant|@variant' \
  phalcom-core phalcom-semantic phalcom-ast
```

## Verify

- [ ] new `Statement::Enum` never enters legacy sealed-class variant expansion;
- [ ] no new enum variant becomes sibling top-level class;
- [ ] no new variant receives class-side `CallableId` as constructor;
- [ ] no exact case becomes ordinary `DeclarationTypeInfo`;
- [ ] no family reservation uses speculative method-vs-variant fallback.

Do not remove the legacy implementation if Part 6 still needs to migrate old tests/core code.

Add explicit comments where both architectures temporarily coexist.

---

# 26. Task 25 — Workspace-Wide `CallableOwnerId` and `TypeData` Exhaustiveness Audit

Before broad verification:

```bash
rg -n 'CallableId\s*\{|CallableId::new|\.owner\.module|\.owner\.name' .
rg -n 'TypeData::' phalcom-semantic
rg -n 'SemanticTargetId::' phalcom-semantic phalcom-lsp
rg -n 'Statement::Class' phalcom-modules phalcom-semantic
```

For each result:

- [ ] classify whether enum/case identity must be supported;
- [ ] eliminate direct structural assumptions where helper methods are available;
- [ ] do not add catch-all `_ =>` merely to silence an exhaustive semantic match if that would discard exact-case semantics.

Particularly inspect:

```text
phalcom-semantic/src/checker
phalcom-semantic/src/db
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/source_index
phalcom-semantic/src/advisory
phalcom-semantic/src/metadata
phalcom-lsp/src/backend.rs
```

---

# 27. Task 26 — Focused Verification

Run in this order.

## Formatting

```bash
cargo fmt --all
cargo fmt --all -- --check
```

## Module layer

```bash
cargo test -p phalcom-modules
```

## Semantic unit/foundation tests

```bash
cargo test -p phalcom-semantic enum_identity
cargo test -p phalcom-semantic exact_case
cargo test -p phalcom-semantic enum_declarations
cargo test -p phalcom-semantic gadt
cargo test -p phalcom-semantic enum_requirements
cargo test -p phalcom-semantic associated_surface
```

Adapt filters to actual test names.

## Existing sensitive suites

```bash
cargo test -p phalcom-semantic semantic::foundations::type_model
cargo test -p phalcom-semantic semantic::foundations::generics_core
cargo test -p phalcom-semantic semantic::foundations::substitution
cargo test -p phalcom-semantic semantic::capabilities::dispatch_capabilities
cargo test -p phalcom-semantic semantic::integration::source_index
cargo test -p phalcom-semantic semantic::incremental::fingerprints
cargo test -p phalcom-semantic semantic::incremental::query_ownership
```

## Dependent crates

```bash
cargo test -p phalcom-core
cargo test -p phalcom-lsp
```

Part 2 must not implement new LSP semantics, but the crate must compile against identity changes.

---

# 28. Task 27 — Full Verification

Run:

```bash
cargo test --workspace
```

Then:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

If the workspace has a documented baseline failure unrelated to this branch:

- [ ] capture the exact failing command;
- [ ] capture the exact test/lint;
- [ ] prove the same failure on the implementation baseline;
- [ ] do not label the Part 2 work fully green unless all new/affected tests pass.

---

# 29. Task 28 — Semantic Invariant Assertions Before Completion

Before reporting success, explicitly verify each statement.

- [ ] `Expr<Int>` is a normal canonical `Applied` type.
- [ ] exact case is `TypeData::ExactCase`.
- [ ] exact case has kind `Type`.
- [ ] exact case key uses compact `VariantTypeId`.
- [ ] stable semantic variant identity remains complete `VariantId`.
- [ ] `@variant None` is singleton.
- [ ] `@variant None()` is zero-arg constructor.
- [ ] `@variant None(_)` is payload constructor.
- [ ] the three may share one family.
- [ ] singleton has no constructor signature.
- [ ] variant constructor is not an ordinary method.
- [ ] exact case subtypes its enum result.
- [ ] exact cases are not synthetic nominal declarations.
- [ ] GADT equality is separate from exact-case identity.
- [ ] GADT equality is in scope while checking case behavior.
- [ ] root declaration-only behavior is a closed-enum requirement.
- [ ] root bodyful behavior is inherited default behavior.
- [ ] every concrete case participates in requirements.
- [ ] private construction does not hide a case from matching/requirements.
- [ ] instance behavior does not reserve associated variant family names.
- [ ] class-side behavior does reserve associated family names.
- [ ] inherited behavioral families reserve descendant associated names.
- [ ] associated declarations do not inherit.
- [ ] no new semantic resolver was added to `phalcom-lsp`.
- [ ] no runtime layout/boxing/discriminant behavior was fixed in Part 2.
- [ ] new enum syntax does not use legacy sealed-class variant expansion.

---

# 30. Suggested Commit Sequence

Keep commits reviewable.

1. `feat(modules): publish enum declaration shells`
2. `refactor(semantic): generalize callable owner identity`
3. `feat(semantic): add variant and associated family identities`
4. `feat(types): canonicalize exact enum case types`
5. `feat(types): propagate exact cases through relations and substitution`
6. `feat(semantic): publish enum declaration products`
7. `feat(semantic): derive GADT case equality environments`
8. `feat(semantic): publish variant constructor signatures`
9. `feat(semantic): publish enum and case behavior`
10. `feat(semantic): enforce closed enum requirements`
11. `feat(semantic): publish associated family surfaces`
12. `feat(semantic): index exact variant targets`
13. `test(semantic): cover enum declarations and incremental reuse`

Do not force this exact split if TDD naturally produces a smaller coherent commit, but avoid one monolithic change.

---

# 31. Required Completion Report

At completion report:

```text
1. implementation branch
2. implementation baseline SHA
3. final HEAD SHA
4. Part 1 prerequisite SHA
5. files changed
6. new semantic identities
7. TypeData changes
8. enum DB products added
9. associated family product added
10. callable-owner migration performed
11. GADT equality implementation
12. requirement/contract behavior
13. visibility behavior
14. source-index changes
15. focused tests and results
16. full workspace results
17. fmt/clippy results
18. unrelated baseline failures, if any
19. deviations from this plan
20. discoveries that should change Part 3
```

Also explicitly state:

```text
- no GitHub/main mutation outside requested branch workflow
- no Part 3 associated-expression resolution implemented
- no runtime enum layout implemented
- no match implementation added
- no LSP semantic authority introduced
```

---

# 32. Part 2 Definition of Done

Part 2 is done when source enums produce a complete immutable semantic declaration model and all downstream compiler consumers can safely reason about canonical exact cases.

The implementation boundary is:

```text
Part 1
    syntax / AST

Part 2  ← THIS PLAN
    declaration identity
    canonical exact types
    GADT case facts
    variant constructor declarations
    root/case behavior
    closed-enum obligations
    associated family reservation
    incremental semantic publication

Part 3
    :: lookup
    family values
    exact constructor refs
    family invocation
    call typing

Part 4
    runtime representation / lowering

Part 5
    match / elimination / refinement

Part 6
    core migration / reflection / cleanup
```

Do not begin Part 3 in the Part 2 implementation branch.
