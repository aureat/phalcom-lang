# SC-1 — Type Formation, Kinds, Generic Declarations, and Type-Level Source Semantics

## Technical Specification

**Status:** Proposed implementation specification  
**Repository:** `aureat/phalcom-lang`  
**Repository baseline audited:** `01e19adb86186d67212b558ba76f54f79e2b5d9f`  
**Parent commit for most of the type-formation audit:** `49d74f9a7d95f695c8ff38c954eca938e6fec16f`  
**Date:** 2026-09-01  
**Scope owner:** `phalcom-semantic` with required source/interface support in `phalcom-ast` and `phalcom-modules`

---

# 1. Purpose

SC-1 closes the semantic gap between Phalcom's already-rich type syntax and the compiler's canonical type/kind model.

The repository already contains substantial pieces of the intended design:

- canonical `TypeId` and `KindId` stores;
- arrow kinds and the distinct `RecordRow` kind;
- owner/index-stable generic parameter identities;
- declaration and callable generic signatures;
- subtype/equivalence `where` constraints;
- partial type application;
- a scoped type-lambda arena with de Bruijn-style bound nodes;
- source class/enum generic publication;
- source method generic publication;
- generic superclass templates;
- `SemanticDenotation::TypeForm(TypeId)`;
- query-owned declaration and callable products.

SC-1 is therefore **not** a rewrite of the type system. It is a semantic-completion pass that makes every ratified type-level source construct either:

1. lower to a valid canonical form with a correct kind and stable declaration/binder identity; or
2. terminate with an explicit non-success outcome that describes why publication cannot occur.

The implementation must stop converting invalid, unresolved, blocked, cancelled, or unsupported type formation into generic `UnknownReason::UnannotatedDeclaration` recovery.

The implementation must also stop silently assigning meaning that source did not have, especially:

- `KindSyntax::Invalid -> KindId::TYPE`;
- an open record tail being discarded and published as a closed record;
- a resolved declaration lacking semantic publication being fabricated as an ad-hoc nominal type;
- a source type-lambda body being lowered as a free canonical type rather than under its lambda binders.

---

# 2. Normative scope

SC-1 includes:

1. proper types and type constructors;
2. `Type`, `RecordRow`, and explicit arrow kinds;
3. source generic binders;
4. declaration-site variance;
5. generic `where` clauses using subtype and equivalence constraints;
6. canonical type application, including partial application;
7. higher-kinded parameters expressed through explicit arrow kinds;
8. source type lambdas;
9. owner/side-relative `Self`;
10. generic superclass templates;
11. transparent type aliases;
12. value-space type-form expressions;
13. exact source-lowering outcome semantics;
14. safe handling of `RecordRow`-kinded generic binders before SC-3;
15. an explicit SC-3 boundary for open record tails.

SC-1 does **not** introduce:

- first-class `forall` types;
- higher-rank or rank-N polymorphism;
- impredicative instantiation;
- public kind variables or prenex kind polymorphism;
- `Type :: Type`;
- universe levels;
- dependent types or dependent kinds;
- generic default type arguments;
- finite exact-set generic constraints;
- intersection types;
- effect rows;
- general row solving;
- public `lacks` syntax;
- generic getters, setters, or indexers;
- runtime monomorphization of generic classes;
- per-instance generic type tokens;
- arbitrary runtime `applyKind`.

Ordinary explicitly declared class/type/callable generics remain in scope. Type lambdas and explicit arrow kinds provide the higher-kinded abstraction required by the currently ratified language without adding higher-rank polymorphism.

---

# 3. Repository-grounded current-state audit

The following table is normative input to this specification. It reflects `main` at the audited HEAD rather than older planning documents.

| Area | Current repository state | SC-1 action |
|---|---|---|
| Canonical inference variables | `TypeData` has no inference-variable variant | Preserve as invariant; add publication tests |
| Kinds | `KindId::TYPE`, `KindId::RECORD_ROW`, and arrow kinds exist | Harden source lowering; invalid syntax must never become `Type` |
| Type application | `TypeStore::apply_type_form` supports kind checking, partial application, and lambda beta reduction | Keep as low-level canonical operator |
| Generic parameter identity | `TypeParameterOwner` + owner/index `TypeParameterId` exists | Preserve; make source binder construction atomic |
| Generic signatures | `GenericSignature` stores parameters and subtype/equivalence constraints | Preserve; stop publishing partially lowered invalid signatures |
| Source class/enum generics | `session.rs` now resolves and publishes signatures after predeclaration | Harden; do not rebuild from scratch |
| Source method generics | `checker/declaration_signature.rs` resolves method generic signatures and `where` clauses | Harden; preserve canonical source/native boundary |
| Generic superclass | `session.rs` builds `GenericSupertypeTemplate` | Harden outcome handling and `Self` context |
| Type lambdas | `type_lambda.rs` has scoped bound representation and beta reducer | Fix source lowering: bind source lambda parameters |
| `Self` | canonical `SelfTypeTerm` exists | Fix source context: owner + dispatch side must be explicit |
| Type aliases | AST/parser/runtime no-op exist; module interface does not collect aliases; semantic product absent | Implement authoritative semantic alias product and transparent expansion |
| Type-form expression | `Expr::TypeForm` checker arm and `SemanticDenotation::TypeForm` exist | Fix it to accept constructor-kinded forms and separate denotation from runtime value type |
| Record rows | canonical row substrate and solver exist | SC-1 only prevents row-binder panic and tail erasure; solving remains SC-3 |
| Source lowering result | `TypeFormResolution::{Known, Dynamic, Unknown}` | Replace with explicit type-formation outcomes |
| Invalid kind syntax | `resolve_kind_syntax` maps invalid syntax to `KindId::TYPE` | Remove this recovery |
| Missing declaration publication | type reference can fall back to `store.nominal_type(decl)` | Remove fabrication |
| Record tail | `TypeAnnotationExpr::Record { tail: _, ... }` | Never discard |
| Generic row binder | `resolve_generic_signature` calls `parameter_form` for every binder; `parameter_form` asserts against `RecordRow` | Fix before SC-3 to eliminate panic |
| Module aliases | `InterfaceBuilder` collects class/enum/let but not `Statement::TypeAlias` | Add alias declaration surface |

## 3.1 Primary implementation anchors

The implementation is centered on these existing files and symbols:

### AST and module identity

- `phalcom-ast/src/ast.rs`
  - `TypeAnnotation`
  - `TypeAnnotationExpr`
  - `KindSyntax`
  - `GenericParameterSyntax`
  - `GenericConstraintSyntax`
  - `WhereClauseSyntax`
  - `TypeAliasDef`
  - `Expr::TypeForm`

- `phalcom-modules/src/interface.rs`
  - `InterfaceBuilder::build`
  - `DeclarationSurface`
  - `UnlinkedModuleInterface`

- `phalcom-modules/src/declaration.rs`
  - `DeclarationKind`
  - `DeclarationBlueprint`
  - `DeclarationShell`
  - `DeclarationShellTable`

- `phalcom-modules/src/graph.rs`
  - `SemanticNodeId`
  - `SemanticEdgeKind`
  - `SemanticGraph`

### Canonical semantic type model

- `phalcom-semantic/src/types/store.rs`
  - `TypeData`
  - `TypeStore`
  - `TypeStore::parameter_form`
  - `TypeStore::apply_type_form`
  - `TypeStore::arrow_kind`
  - `TypeStore::lambda`
  - canonical record-row storage

- `phalcom-semantic/src/types/parameter.rs`
  - `TypeParameterOwner`
  - `TypeParameterData`
  - `GenericConstraint`
  - `GenericSignature`
  - `SelfTypeTerm`
  - `TypeTerm`

- `phalcom-semantic/src/types/type_lambda.rs`
  - `ScopedTypeData`
  - `TypeLambdaData`
  - `TypeLambdaArena`
  - beta reduction and scoped substitution

- `phalcom-semantic/src/types/annotation.rs`
  - `TypeResolver`
  - `ScopedTypeResolver`
  - `TypeFormResolution`
  - `resolve_kind_syntax`
  - `resolve_type_form`
  - `resolve_type_annotation`
  - `resolve_generic_signature`

### Declaration publication and checking

- `phalcom-semantic/src/declarations.rs`
  - `DeclarationTypeInfo`
  - `GenericSupertypeTemplate`
  - `DeclarationTypeTable`

- `phalcom-semantic/src/session.rs`
  - `SemanticWorkspaceSession`
  - source declaration predeclaration
  - declaration generic-signature publication
  - superclass-template publication

- `phalcom-semantic/src/checker/declaration_signature.rs`
  - `semantic_signature_for_syntax`
  - `semantic_field_signature_for_member`
  - `CallableSyntaxRef`

- `phalcom-semantic/src/checker/declaration.rs`
  - `member_side`
  - `register_class_surface`
  - `check_class_bodies`

- `phalcom-semantic/src/checker/expression.rs`
  - `Expr::TypeForm` arm
  - ordinary type-name expression resolution

### Query/snapshot layer

- `phalcom-semantic/src/db/key.rs`
- `phalcom-semantic/src/db/product.rs`
- `phalcom-semantic/src/db/query.rs`
- `phalcom-semantic/src/db/fingerprint.rs`
- `phalcom-semantic/src/snapshot.rs`

### Existing focused tests

- `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
- `phalcom-semantic/tests/semantic/integration/workspace.rs`
- `phalcom-semantic/tests/semantic/incremental/fingerprints.rs`
- module-interface tests under `phalcom-modules/tests/`

---

# 4. Semantic vocabulary

SC-1 standardizes four concepts that must remain distinct.

## 4.1 Kind

A **kind** classifies a type-level form.

The currently ratified grammar is:

```text
Kind ::= Type
       | RecordRow
       | (Kind, ...) -> Kind
```

Examples:

```text
Int                         :: Type
List                        :: Type -> Type
Map                         :: (Type, Type) -> Type
Functor                     :: (Type -> Type) -> Type
R                           :: RecordRow
```

`RecordRow` is a separate semantic domain. It does not become an ordinary proper type merely because its binder is represented by a `TypeParameterId`.

No variance belongs to arrow-kind parameters. Variance is a declaration/property of nominal generic parameters, not of kind arrows.

## 4.2 Type form

A **type form** is any canonical type-level expression classified by a kind.

A proper type is a type form whose kind is exactly `Type`.

Thus:

```text
List            -- type form, not proper type
Map<String>     -- type form, not proper type
List<Int>       -- type form and proper type
<T> =>> List<T> -- type form, constructor kind
```

## 4.3 Proper type

A **proper type** is a type form of kind `Type` that can classify ordinary values.

Value annotations require proper types unless the syntax is an explicit dynamic escape.

Invalid:

```phalcom
const xs: List = ...
```

because `List :: Type -> Type`.

Valid:

```phalcom
const xs: List<Int> = ...
```

## 4.4 Type-form value

A type-level form may itself appear in value position through `Expr::TypeForm`.

That expression has two independent semantic facts:

```text
runtime/value type of the descriptor value
semantic denotation = canonical TypeId
```

For example, a type constructor value may denote:

```text
List :: Type -> Type
```

even though the runtime value's ordinary class/type is a type-descriptor/class-object representation.

The checker must not force the denoted form to have kind `Type` merely because the form occurs as a value.

---

# 5. Canonical type and kind laws

## 5.1 No solver metavariable is a canonical `TypeId`

`TypeData` must remain free of query-local inference variables.

Solver-local IDs such as `InferVarId` and `RecordRowVarId` may refer to canonical types but may not be interned as canonical type nodes and may never be published in declaration/callable metadata.

## 5.2 Canonical type equality is semantic equality

A `TypeId` is an interned canonical semantic identity within one `TypeStore`.

Source aliases, source spelling, source ranges, and generic parameter names do not alter canonical equality.

## 5.3 Kinds are canonical

Equivalent arrow kinds must intern to the same semantic kind identity.

The kind of a partially applied type constructor is its residual arrow kind.

For example:

```text
Map                     :: (Type, Type) -> Type
Map<String>             :: Type -> Type
Map<String, Int>        :: Type
```

## 5.4 Low-level application is kind-directed, not declaration-policy-directed

`TypeStore::apply_type_form` remains the low-level canonical operator.

It owns:

- constructor-kind checking;
- arity against residual kind;
- partial application;
- canonical `Applied` interning;
- type-lambda beta reduction.

It does **not** become the owner of:

- source diagnostics;
- declaration visibility;
- declaration `where` policy;
- source alias navigation;
- query cancellation policy.

Those remain higher-level semantic concerns.

---

# 6. Source type-formation outcome algebra

The current three-way result:

```rust
pub enum TypeFormResolution {
    Known(TypeId),
    Dynamic,
    Unknown(UnknownReason),
}
```

is insufficient because it merges invalid source, unresolved source, unavailable dependencies, and analysis failures.

SC-1 introduces an explicit source-formation result family.

Recommended target:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationOutcome<T> {
    Ready(T),
    Dynamic,
    Missing(TypeFormationMissing),
    Unresolved(TypeFormationUnresolved),
    Invalid(TypeFormationInvalid),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(String),
}

pub type TypeFormResolution = TypeFormationOutcome<TypeId>;
pub type KindResolution = TypeFormationOutcome<KindId>;
```

The exact internal reason enums are domain-specific:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationMissing {
    Annotation,
    DeclarationProduct(DeclarationId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationUnresolved {
    Name(Box<str>),
    SelfOutsideOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationInvalid {
    Syntax,
    InvalidKindSyntax,
    ExpectedProperType { actual: KindId },
    NotAConstructor,
    TooManyTypeArguments,
    TypeArgumentKindMismatch,
    MalformedTypeLambda,
    DuplicateRecordField(Box<str>),
    GenericConstraintOperandNotType,
    InvalidVariance,
    UnsupportedOpenRecordTail,
}
```

The implementation may refine reason payloads, but the terminal categories are normative.

## 6.1 `UnknownReason` is not the type-formation result algebra

`TypeKnowledge::Unknown(UnknownReason)` describes the checker not having established a value type.

It is not the correct representation for an invalid source type form.

The conversion to `TypeKnowledge` happens only at the boundary where a value annotation must become checker knowledge.

Examples:

```text
unresolved annotation name
    -> TypeFormationOutcome::Unresolved
    -> diagnostic
    -> value checker may publish Unknown(UnresolvedName(...))

invalid application
    -> TypeFormationOutcome::Invalid
    -> diagnostic
    -> causal invalidity
    -> never "UnannotatedDeclaration"
```

## 6.2 Dynamic remains explicit

`Dynamic` remains an explicit source semantic boundary.

It is not:

- a canonical `TypeId`;
- a missing type;
- an inference default;
- an error recovery type.

---

# 7. Kind syntax lowering

`resolve_kind_syntax` must stop returning a bare `KindId`.

Target signature:

```rust
pub fn resolve_kind_syntax(
    store: &mut TypeStore,
    kind: &KindSyntax,
    control: &mut TypeFormationControl<'_>,
) -> KindResolution
```

or an equivalent context-based form.

Normative rules:

1. `KindSyntax::Type` -> `Ready(KindId::TYPE)`.
2. `KindSyntax::RecordRow` -> `Ready(KindId::RECORD_ROW)`.
3. arrow kind -> recursively lower all parameter/result kinds and intern the canonical arrow kind.
4. invalid/recovered source syntax -> `Invalid(InvalidKindSyntax)`.
5. cancellation/budget exhaustion propagate unchanged.
6. no invalid source can produce `KindId::TYPE` merely to keep analysis moving.

Recovery belongs in the caller's diagnostic/control policy, not in the canonical kind store.

---

# 8. Generic binders

## 8.1 Stable identity

A source generic binder is identified by owner and position:

```rust
TypeParameterId {
    owner,
    index,
}
```

The source name is presentation/provenance and lexical lookup information.

Renaming:

```phalcom
class Box<T>
```

to:

```phalcom
class Box<U>
```

without changing binder position/kind/variance must not conceptually create a distinct generic slot.

## 8.2 Binder domains

A generic parameter's declared kind determines what semantic domain its reference belongs to.

The current resolver API assumes every type parameter resolves to a `TypeId`:

```rust
fn resolve_type_parameter(&self, name: &str) -> Option<TypeId>;
```

That is insufficient for `RecordRow`.

SC-1 replaces the internal lexical binding model with:

```rust
#[derive(Clone, Debug)]
pub enum TypeLevelBinding {
    TypeForm(TypeId),
    RecordRow(TypeParameterId),
}
```

and a resolver operation conceptually equivalent to:

```rust
fn resolve_type_level_binding(&self, name: &str) -> Option<TypeLevelBinding>;
```

A `RecordRow` parameter is therefore stable and resolvable without constructing `TypeData::Parameter`.

This is required because `TypeStore::parameter_form` correctly rejects `RecordRow`-kinded parameters.

SC-3 later consumes `TypeLevelBinding::RecordRow` when lowering an open record tail.

## 8.3 Variance

Declaration-site variance is:

```text
+T   covariant
-T   contravariant
 T   invariant
```

Variance is valid only on nominal/declaration generic parameters for which variance is part of the declared type constructor contract.

Method-local generic parameters, type-alias lambda binders, and source type-lambda binders are invariant/no-variance binders unless a future feature explicitly changes this.

Invalid variance placement produces an explicit formation diagnostic and prevents successful signature publication.

---

# 9. Generic signatures and `where` clauses

A `GenericSignature` is a declaration-owned semantic object:

```rust
GenericSignature {
    owner,
    parameters,
    constraints,
}
```

Constraints are signature-owned, not parameter-owned.

Initial constraint forms remain:

```rust
GenericConstraint::Subtype { lower, upper }
GenericConstraint::Equivalent { left, right }
```

Source examples:

```phalcom
class Box<T>
where T <: Object
```

```phalcom
method<T, U>(...)
where T == U
```

```phalcom
method<T>(...)
where Number <: T
```

Lower bounds use operand order; no separate `LowerBound` syntax is needed.

## 9.1 Atomic publication

`resolve_generic_signature` must never silently omit a constraint because one operand failed to lower.

Current behavior conditionally appends a constraint only when both operands become `Known`.

Target behavior:

```text
all binders valid
AND
all binder kinds valid
AND
all constraints valid
    -> Ready(GenericSignature)

otherwise
    -> explicit non-success
    -> no canonical signature is published as if complete
```

A caller may retain a recovery shell for diagnostics, but a recovery shell is not a valid `GenericSignature`.

Recommended target signature:

```rust
pub fn resolve_generic_signature(
    ...,
) -> TypeFormationOutcome<GenericSignature>
```

## 9.2 Constraint operands

Subtype/equivalence constraints in SC-1 operate on ordinary type forms.

A `RecordRow` binder cannot appear directly as an operand to `T <: U`, because it is not a proper/type-constructor form in the ordinary type relation domain.

Future row constraints belong to SC-3.

---

# 10. Type application

Phalcom supports partial type application.

Given:

```text
Map :: (Type, Type) -> Type
```

all are valid type forms:

```text
Map
Map<String>
Map<String, Int>
```

Only the last is a proper type.

The source resolver must:

1. resolve the origin to a canonical type form;
2. resolve every supplied argument to a type form;
3. use `TypeStore::apply_type_form`;
4. preserve the exact `TypeApplicationError` category;
5. return `Invalid(...)` rather than `Unknown(UnannotatedDeclaration)`;
6. validate declaration-owned generic policy at the high-level semantic boundary where required.

A type application never defaults an unsolved or invalid argument to `Dynamic`, `Object`, `Unit`, or `Unknown`.

---

# 11. Source type lambdas

Source syntax:

```phalcom
<T> =>> List<T>
```

or with explicit kinds:

```phalcom
<F: Type -> Type, T> =>> F<T>
```

The canonical scoped representation already exists in `types/type_lambda.rs`.

SC-1 makes source lowering use it correctly.

## 11.1 Bound-variable law

For:

```phalcom
<T> =>> T
```

the body must be:

```rust
ScopedTypeData::Bound {
    depth: 0,
    index: 0,
}
```

It must not be:

```rust
ScopedTypeData::Free(...)
```

and must not resolve `T` through a declaration `TypeParameterId`.

## 11.2 Nested lambdas

For:

```phalcom
<T> =>> <U> =>> (T, U)
```

the outer and inner references must remain capture-safe.

The lowering environment is a stack of binder layers.

A reference is encoded as:

```text
depth = number of nested lambda layers between the reference and its binder
index = binder position within that layer
```

## 11.3 Free declaration parameters

For:

```phalcom
class C<T> {
    ...
}
```

and a type lambda appearing in a declaration context:

```phalcom
<U> =>> Pair<T, U>
```

`U` is scoped/bound.

`T` remains a free canonical declaration parameter:

```rust
ScopedTypeData::Free(type_id_for_C_T)
```

## 11.4 Alpha equivalence

These lower to the same semantic lambda:

```phalcom
<T> =>> List<T>
<U> =>> List<U>
```

Parameter names are provenance only.

## 11.5 Beta reduction

Existing `TypeStore::apply_type_form` / `TypeLambdaArena` beta reduction remains authoritative.

Example:

```text
(<T> =>> List<T>)<Int>
    == List<Int>
```

## 11.6 Scoped source lowerer

SC-1 introduces one recursive scoped lowerer shared by:

- source type-lambda bodies;
- generic transparent alias bodies.

Conceptual interface:

```rust
pub fn lower_scoped_type_form(
    ...,
    binders: &ScopedBinderStack,
    annotation: &TypeAnnotation,
    ...
) -> TypeFormationOutcome<ScopedTypeId>;
```

It must have explicit cases for every `TypeAnnotationExpr`.

Open record tails remain blocked until SC-3 rather than being erased.

---

# 12. `Self`

`Self` is contextual. It is not an ordinary symbol-table declaration.

Canonical representation already exists:

```rust
SelfTypeTerm {
    owner,
    side,
    role,
}
```

SC-1 requires lowering to receive the owner and dispatch side explicitly.

Normative rules:

### Instance-side member

```text
Self
    -> owner-relative instance type
```

### Class-side member

```text
Self
    -> owner-relative class-side form
```

### Outside a declaration/member context

```text
Self
    -> Unresolved(SelfOutsideOwner)
```

The current fallback:

```rust
side: DispatchSide::Instance
```

inside `resolve_type_form` is not acceptable.

`Self` context must not be inferred from a generic name resolver whose primary job is module/name lookup.

Recommended context type:

```rust
#[derive(Clone, Debug)]
pub struct TypeFormationSite {
    pub module: ModuleId,
    pub self_term: Option<SelfTypeTerm>,
}
```

Every declaration/member lowering path constructs the appropriate site.

---

# 13. Declaration generic scope and dispatch side

Phalcom separates instance-side declaration specialization from class-side behavior.

A generic class:

```phalcom
class Box<T> {
    ...
}
```

gives instance-side members access to declaration generic `T`.

Class-side members do not automatically receive an ambient instance `T`.

Therefore the semantic signature builder must not construct one declaration-generic resolver and reuse it blindly for both dispatch sides.

Target helper:

```rust
fn declaration_type_level_bindings_for_side(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    side: DispatchSide,
) -> HashMap<String, TypeLevelBinding>
```

Rules:

```text
Instance side -> declaration generic bindings
Class side    -> empty declaration instance-generic environment
```

A class-side factory that needs generics declares its own callable generics.

Constructor semantics remain separately defined by constructor lowering and return `Self`/owner-specialized instance forms according to the constructor contract; constructor syntax is not justification for leaking ambient instance binders into every class-side member.

---

# 14. Generic superclass templates

A generic superclass is stored statically as an unspecialized template.

Example:

```phalcom
class Box<T> is Container<T> {
    ...
}
```

The declaration product stores:

```text
Box form                :: Type -> Type
Box generic signature  = <T>
supertype template     = Container<T>
```

At a concrete receiver:

```text
Box<Int>
```

the hierarchy/member-specialization layer substitutes:

```text
T := Int
```

into the supertype template.

SC-1 requirements:

1. superclass lowering occurs in the declaration's instance generic environment;
2. the final template must have kind `Type`;
3. invalid/blocked superclass formation does not become `None` indistinguishably from “no superclass written”;
4. runtime erased superclass/class/metaclass identity remains unchanged by static generic metadata;
5. source `Self` used in any allowed superclass-related form preserves owner/side context.

---

# 15. Transparent type aliases

Source syntax:

```phalcom
type UserId = Int
type Pair<T> = (T, T)
```

A transparent alias:

- has stable declaration identity for navigation, provenance, query dependencies, diagnostics, exports, metadata, and incremental invalidation;
- does not create nominal semantic identity;
- does not create a runtime class;
- does not create an allocation identity;
- expands transparently for type equivalence.

Thus:

```text
UserId == Int
Pair<String> == (String, String)
```

at canonical semantic equality after expansion.

## 15.1 Module/interface identity

`InterfaceBuilder::build` must collect `Statement::TypeAlias` in the same module namespace as other immutable type-side declarations.

An alias can be imported and exported by stable declaration identity.

## 15.2 Semantic declaration product

Do not fake an alias as `DeclarationTypeInfo`.

`DeclarationTypeInfo` owns nominal concepts such as `class_object_type`.

Introduce a separate canonical alias record:

```rust
#[derive(Clone, Debug)]
pub struct TypeAliasInfo {
    pub declaration: DeclarationId,
    pub kind: KindId,
    pub generic_signature: Option<GenericSignature>,
    pub form: TypeId,
    pub dependencies: Box<[DeclarationId]>,
    pub source: SemanticSourceSpan,
}
```

For a non-generic alias:

```text
type UserId = Int

form = Int
kind = Type
```

For a generic alias:

```text
type Pair<T> = (T, T)

form = canonical lambda <0> =>> (bound(0), bound(0))
kind = Type -> Type
```

Generic alias body lowering reuses the scoped type-lambda source lowerer.

## 15.3 Dependency-visible shell

`QueryKey::DeclarationShell(DeclarationId)` should remain the dependency identity for a type-side declaration.

The semantic product should be generalized conceptually to:

```rust
pub enum TypeDeclarationShell {
    Nominal(DeclarationTypeInfo),
    Alias(TypeAliasInfo),
}
```

or an equivalent discriminated product.

This prevents creating a second incremental authority for aliases.

`DeclarationTypeTable` may remain the nominal lookup table used by dispatch; aliases are stored in a dedicated `TypeAliasTable` snapshot projection.

## 15.4 Alias cycles

Transparent recursive aliases are rejected initially.

Invalid:

```phalcom
type A = B
type B = A
```

Invalid:

```phalcom
type A = A
```

The compiler must:

1. predeclare alias identities and kinds/binders;
2. collect alias-to-alias dependencies;
3. detect SCC/self cycles;
4. diagnose each illegal cycle deterministically;
5. never attempt unbounded recursive expansion;
6. never publish a cyclic alias as a successful canonical form.

Cross-module alias cycles are subject to the same rule.

## 15.5 Alias provenance versus equality

Alias provenance is not encoded by creating a distinct `TypeData` identity.

Navigation and diagnostics use the source occurrence/declaration identity that resolved the alias.

Canonical equality uses the expanded `TypeId`.

---

# 16. Type-form values

`Expr::TypeForm` already exists, but the current checker routes it through `resolve_type_annotation`, which requires the result to be a proper type.

That incorrectly rejects constructor-kinded type-form values.

SC-1 changes the checker to resolve a type **form**, not a value annotation.

Target algorithm:

```text
Expr::TypeForm(annotation)
    ↓
resolve_type_form
    ↓
Ready(form)
    ↓
kind = store.kind_of(form)
    ↓
determine descriptor/class-object runtime value type
    ↓
TypedExpression {
    knowledge = runtime value type,
    denotation = SemanticDenotation::TypeForm(form)
}
```

Examples that must work:

```phalcom
const intType = Int
const listConstructor = List
const mapFromString = Map<String>
const f = <T> =>> List<T>
```

Their denotations may have kinds:

```text
Type
Type -> Type
Type -> Type
Type -> Type
```

respectively.

No runtime descriptor allocation is required merely to perform compile-time semantic analysis. Runtime materialization remains a later code-generation/runtime projection.

---

# 17. Open record-tail handoff to SC-3

Source AST already contains a record tail.

Current lowering discards it.

SC-1 must immediately replace this behavior.

Given:

```phalcom
#{ name: String, | R }
```

before SC-3 end-to-end row lowering is enabled, SC-1 returns:

```text
Invalid(UnsupportedOpenRecordTail)
```

or an explicit `Blocked(...)` feature-unavailable result, according to the final shared outcome convention.

It must **not** publish:

```phalcom
#{ name: String }
```

as if that were what the user wrote.

SC-1 also makes a `RecordRow` generic binder safe to declare and resolve as `TypeLevelBinding::RecordRow`.

SC-3 later changes only the tail-specific branch from blocked to canonical open-row lowering and integrates row inference/relations.

---

# 18. Diagnostics

SC-1 should add or refine diagnostic codes so that source failures are semantically specific.

Recommended additions to `DiagnosticCode`:

```rust
AnnotationInvalidKindSyntax,
GenericParameterInvalidVariance,
GenericSignatureInvalid,
GenericConstraintInvalidOperand,
SelfOutsideTypeContext,
TypeAliasCycle,
TypeAliasInvalidTarget,
OpenRecordTailUnavailable,
DeclarationTypeProductMissing,
```

Existing codes remain appropriate for:

```text
AnnotationUnresolved
AnnotationUnsupported
AnnotationUnsaturatedConstructor
KindExpectedType
ApplicationNotConstructor
ApplicationTooManyArguments
ApplicationArgumentKindMismatch
AnalysisBlocked
AnalysisBudgetExceeded
AnalysisInternalFailure
```

Diagnostic rules:

1. a single semantic defect should have one root cause;
2. dependent failures are suppressed/causally attached rather than duplicated;
3. invalid source never becomes `Dynamic`;
4. blocked/cancelled/budget/internal states are not reported as ordinary type mismatch;
5. successful declaration products are never published if their defining type form is invalid.

---

# 19. Query and incrementality requirements

SC-1 declaration/type-level products participate in the existing `SemanticDb`.

Required dependency examples:

```text
CallableSignature(C.use)
    -> DeclarationShell(Alias)

DeclarationShell(GenericClass)
    -> DeclarationShell(Superclass)

DeclarationShell(AliasA)
    -> DeclarationShell(AliasB)
```

Changing:

```phalcom
type Id = Int
```

to:

```phalcom
type Id = String
```

must invalidate unchanged consumers that resolved through `Id`.

Changing only source ranges/comments must not invalidate consumers when the public alias semantic fingerprint is unchanged.

Alias/product fingerprints include semantic content:

- declaration stable identity;
- kind;
- generic parameter kinds/variance;
- constraints;
- expanded canonical form in stable/exported structural form;
- dependency identities as required for direct-input identity.

They do not use raw `TypeId` numeric indices as cross-store structural fingerprints.

---

# 20. Snapshot requirements

`SemanticSnapshot` must expose authoritative ready products needed by consumers.

Nominal declaration information remains:

```rust
pub declarations: Arc<DeclarationTypeTable>
```

SC-1 adds:

```rust
pub type_aliases: Arc<TypeAliasTable>
```

or an equivalent immutable alias projection.

LSP/editor layers consume this snapshot product. They do not reparse alias bodies or reconstruct generic signatures.

---

# 21. Compatibility and deletion rules

SC-1 removes semantic fallbacks once canonical publication is available.

Delete or replace patterns equivalent to:

```rust
declarations.form(&decl)
    .unwrap_or_else(|| store.nominal_type(decl))
```

A name resolver proving that a declaration exists while the semantic declaration product is absent is not permission to fabricate a nominal type.

It must become an explicit missing/blocked/internal semantic state.

Likewise, remove type-formation uses of:

```rust
UnknownReason::UnannotatedDeclaration
```

for:

- kind mismatch;
- generic application error;
- invalid syntax;
- unsupported row tail;
- missing declaration product.

`UnannotatedDeclaration` remains valid only for genuinely unannotated value/declaration situations in the value checker.

---

# 22. Runtime invariants

SC-1 changes static semantic products, not runtime object identity rules.

The following must remain unchanged:

- selector identity;
- runtime class identity;
- metaclass identity;
- method dictionary lookup key;
- ordinary object allocation layout;
- generic instance representation;
- no per-instance generic token;
- no type-lambda solver object stored in ordinary runtime values.

Transparent aliases emit no runtime declaration/allocation identity.

Type-form runtime descriptors remain lazily/materialization-policy driven.

---

# 23. Required semantic laws

The implementation is accepted only if all of these laws hold.

## Kinds

1. invalid kind syntax never returns `KindId::TYPE`;
2. arrow-kind canonicalization is deterministic;
3. partial application produces the correct residual kind;
4. `RecordRow` is not a proper type.

## Generic binders

5. stable binder identity is owner + index;
6. a `RecordRow` binder never produces `TypeData::Parameter`;
7. row-binder declaration does not panic;
8. variance placement is validated;
9. invalid generic signatures are not partially published.

## Type lambdas

10. `<T> =>> T` contains a scoped bound reference;
11. alpha-renaming preserves semantic identity;
12. nested lambdas are capture-safe;
13. free declaration parameters remain free;
14. beta reduction preserves kinds;
15. partial lambda application produces a residual lambda.

## `Self`

16. instance-side `Self` carries instance side;
17. class-side `Self` carries class side;
18. `Self` outside an owner is invalid;
19. inherited specialization does not rewrite owner identity incorrectly.

## Generic declarations

20. declaration constructor kind matches binder kinds;
21. generic signature and declaration kind cannot disagree;
22. source class/enum/method generic signatures are published before dependent body analysis;
23. class-side members do not receive ambient instance generic bindings.

## Superclasses

24. generic superclass template is kind `Type`;
25. invalid written superclass is distinguishable from absent superclass;
26. receiver specialization substitutes the template.

## Aliases

27. aliases are module declarations;
28. aliases are transparent for semantic equality;
29. aliases retain declaration identity for navigation/invalidation;
30. generic aliases are capture-safe;
31. recursive transparent aliases are rejected;
32. alias edits invalidate consumers through DB dependencies;
33. aliases create no class object or runtime allocation identity.

## Type-form values

34. proper types can be values;
35. constructor-kinded forms can be values;
36. type-form value type and type-form denotation remain separate axes;
37. compile-time type-form use does not require eager runtime descriptor allocation.

## Outcomes

38. unresolved != invalid != blocked != cancelled != budget exceeded != internal failure;
39. `Unknown` != `Dynamic`;
40. no invalid formation is laundered into `UnannotatedDeclaration`.

## Record-tail boundary

41. an open record tail is never silently discarded;
42. SC-1 does not claim row solving is complete.

---

# 24. Required test matrix

At minimum, SC-1 must add tests for the following.

## Kind tests

```phalcom
class Box<T>
class FBox<F: Type -> Type>
```

plus malformed/recovered kind syntax proving no successful `Type` publication.

## Partial application

```text
Map
Map<String>
Map<String, Int>
```

Verify kinds exactly.

## Type lambda

```phalcom
<T> =>> T
<T> =>> List<T>
<T> =>> <U> =>> (T, U)
<F: Type -> Type, T> =>> F<T>
```

Verify bound depth/index and beta reduction.

## Generic publication

```phalcom
class Box<+T>
where T <: Object
```

Verify constructor kind, parameter variance, and constraint retention.

## Invalid generic publication

Malformed kind/constraint must produce no ready generic signature.

## Row binder safety

```phalcom
class Shape<R: RecordRow>
```

The declaration must not panic and `R` must not become `TypeData::Parameter`.

## `Self`

Instance-side and class-side member annotations must produce different `SelfTypeTerm.side` values.

## Class-side generic scope

A class-side member that refers to the class's instance generic `T` without declaring its own generic must be diagnosed rather than silently resolving the ambient `T`.

## Generic superclass

```phalcom
class Container<T> {}
class Box<T> is Container<T> {}
```

Verify the unspecialized `Container<T>` template and specialized inherited view.

## Transparent aliases

```phalcom
type UserId = Int
type Pair<T> = (T, T)
```

Verify import/export, kind, transparent equality, generic application, and dependency fingerprints.

## Alias cycles

```phalcom
type A = A
```

and:

```phalcom
type A = B
type B = A
```

must be rejected deterministically.

## Type-form values

```phalcom
const a = Int
const b = List
const c = Map<String>
const d = <T> =>> List<T>
```

Verify semantic denotation kind and ordinary value type separately.

## Record-tail safety

```phalcom
<R: RecordRow>
type Named = #{ name: String, | R }
```

Until SC-3 tail lowering is enabled, assert an explicit blocked/unsupported result and no closed-record publication.

## Incremental

Change:

```phalcom
type Id = Int
```

to:

```phalcom
type Id = String
```

and prove unchanged dependent signatures/bodies recompute while unrelated products reuse.

## Cold/incremental structural parity

Build the same final source:

1. from a clean session;
2. through a sequence of incremental edits.

Compare stable exported structural type/kind/generic/alias facts, not raw `TypeId` numbers.

---

# 25. Completion definition

SC-1 is complete when every currently ratified source-level type-formation construct has one authoritative path:

```text
source syntax
    ↓
module/declaration identity
    ↓
explicit type-formation outcome
    ↓
canonical kind/type/binder/signature/alias product
    ↓
SemanticDb dependency
    ↓
immutable snapshot projection
    ↓
checker / compiler / LSP / metadata consumers
```

There must be no alternate path that:

- invents a nominal type because publication is missing;
- re-lowers the same generic signature differently in a body checker;
- ignores a row tail;
- substitutes `Type` for malformed kind syntax;
- converts invalid type application into generic unknownness;
- treats a type-lambda binder as a free name;
- gives every class-side member the instance generic environment;
- requires a type-form value's denotation to be a proper type.

At that point SC-2 can rely on a closed, canonical type-formation substrate for generic callable inference rather than compensating for malformed source signatures.

---

# Appendix A — Current defects mapped to target fixes

| Current code | Problem | Target |
|---|---|---|
| `types/annotation.rs::resolve_kind_syntax -> KindId` | `KindSyntax::Invalid` becomes `Type` | `KindResolution`, explicit invalid |
| `TypeFormResolution::{Known,Dynamic,Unknown}` | collapses invalid/unresolved/unavailable states | explicit `TypeFormationOutcome` |
| reference fallback `store.nominal_type(decl)` | fabricates semantic type | missing/blocked declaration product |
| application error -> `Unknown(UnannotatedDeclaration)` | semantic failure laundered as missing annotation | `Invalid(TypeApplication...)` |
| record `tail: _` | open row becomes false closed row | explicit SC-3 handoff |
| type lambda body -> `ScopedTypeData::Free(body_ty)` | binder capture is wrong | recursive scoped lowerer |
| `resolve_generic_signature` creates `parameter_form` for every binder | `RecordRow` binder can assert/panic | domain-aware `TypeLevelBinding` |
| invalid constraint operands skipped | partial signature published | atomic signature outcome |
| `Self` hardcodes instance side | class-side semantics wrong | explicit `TypeFormationSite.self_term` |
| one declaration generic resolver for all member sides | class side can see ambient instance `T` | side-aware generic scope |
| no interface collection for `TypeAlias` | alias lacks stable linked declaration identity | collect alias declaration |
| no semantic alias product | parsed alias has no formal authority | `TypeAliasInfo` + DB shell |
| `Expr::TypeForm` uses proper-type annotation lowering | constructor values rejected | use `resolve_type_form` |
| fallback class object synthesis for missing declaration info | hides publication holes | explicit non-success |

---

# Appendix B — Deliberate handoffs

## SC-2

Consumes:

- valid canonical callable/declaration generic signatures;
- valid parameter kinds;
- valid `where` constraints;
- type application/kind semantics.

Owns:

- callable-local generic inference;
- expected-result-driven inference;
- argument constraints;
- inference ambiguity/underconstraint;
- call-site constraint solving.

## SC-3

Consumes:

- `KindId::RECORD_ROW`;
- stable row `TypeParameterId`;
- `TypeLevelBinding::RecordRow`;
- explicit source record tail.

Owns:

- canonical open row construction;
- row unification;
- lacks constraints;
- row inference;
- immutable structural record relations.

## SC-6

Consumes SC-1 products for:

- final store-independent metadata schema projection;
- reflection;
- LSP/platform convergence;
- full cold/incremental certification across external consumers.

SC-1 must nevertheless expose complete compiler-owned semantic products so SC-6 is a projection task, not a second semantics implementation.
