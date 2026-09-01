# SC-1 — Type Formation, Kinds, Generic Declarations, and Type-Level Source Semantics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every ratified Phalcom source type form lower through one explicit, canonical, publication-safe semantic path, including kinds, generic declarations, type lambdas, `Self`, generic superclasses, transparent aliases, and value-space type forms.

**Architecture:** Keep `TypeStore` as the canonical low-level type/kind interner. Add an explicit type-formation outcome layer above it; make lexical type-level bindings kind/domain-aware; use the existing scoped type-lambda arena for source lambdas and generic aliases; generalize declaration-shell publication so aliases participate in the same DB dependency graph without pretending to be nominal classes; make every source consumer use published canonical signatures rather than reconstructing type semantics ad hoc.

**Tech Stack:** Rust; `phalcom-ast`; `phalcom-modules`; `phalcom-semantic`; existing `SemanticDb`, `TypeStore`, `TypeLambdaArena`, declaration/signature tables, semantic test fixtures.

**Spec:** `SC-1-type-formation-kinds-generics-technical-spec.md`

**Repository baseline:** `aureat/phalcom-lang@01e19adb86186d67212b558ba76f54f79e2b5d9f`

---

# Global constraints

1. Do not add `TypeData::Infer`, a row solver variable, or any query-local metavariable to `TypeStore`.
2. Do not encode `Dynamic` as a `TypeId`.
3. Do not create a fake nominal/class-object type for a type alias.
4. Do not make `TypeStore::apply_type_form` own source diagnostics or declaration `where` policy.
5. Do not enable general open-row solving in this plan. SC-3 owns it.
6. Do not leave `tail: _` in record annotation lowering.
7. Do not preserve `KindSyntax::Invalid -> KindId::TYPE`.
8. Do not publish a `GenericSignature` after dropping a malformed constraint.
9. Do not call `TypeStore::parameter_form` for a `RecordRow`-kinded binder.
10. Do not give class-side members ambient instance generic parameters.
11. Do not route `Expr::TypeForm` through the proper-type-only annotation boundary.
12. Do not add first-class `forall`, rank-N polymorphism, public kind variables, generic defaults, or finite-set bounds.
13. Prefer extending existing tests and fixtures over creating a parallel semantic harness.
14. Every semantic change starts with a failing test.
15. After every task, run the smallest focused test target before moving on.
16. Do not modify runtime selector/class/layout behavior to implement this plan.

---

# Task 0 — Establish and record the implementation baseline

**Files:**

- Read: `phalcom-semantic/src/types/annotation.rs`
- Read: `phalcom-semantic/src/types/store.rs`
- Read: `phalcom-semantic/src/types/parameter.rs`
- Read: `phalcom-semantic/src/types/type_lambda.rs`
- Read: `phalcom-semantic/src/declarations.rs`
- Read: `phalcom-semantic/src/session.rs`
- Read: `phalcom-semantic/src/checker/declaration_signature.rs`
- Read: `phalcom-semantic/src/checker/declaration.rs`
- Read: `phalcom-semantic/src/checker/expression.rs`
- Read: `phalcom-semantic/src/db/{key,product,query,fingerprint}.rs`
- Read: `phalcom-semantic/src/snapshot.rs`
- Read: `phalcom-modules/src/{interface,declaration,graph}.rs`
- Read: `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
- Read: `phalcom-semantic/tests/semantic/integration/workspace.rs`

## Step 0.1 — Create an isolated branch/worktree

- [ ] Create a feature branch named something like:

```bash
git switch -c semantic/sc1-type-formation-completion
```

If the implementation environment uses worktrees, create one according to the repository's normal workflow.

## Step 0.2 — Verify the baseline commit

- [ ] Run:

```bash
git rev-parse HEAD
```

- [ ] Record the exact SHA in the implementation PR/notes.
- [ ] If HEAD is newer than `01e19adb86186d67212b558ba76f54f79e2b5d9f`, inspect diffs touching the files listed above before applying this plan.
- [ ] Do not blindly overwrite newer source.

## Step 0.3 — Run the current focused tests

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic foundations::type_annotations
```

If the test binary/module path differs, use:

```bash
cargo test -p phalcom-semantic type_annotations
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic workspace
```

- [ ] Run:

```bash
cargo test -p phalcom-modules interface
```

- [ ] Record failures that already exist before SC-1.

**Expected result:** Existing baseline tests pass, or existing failures are recorded and demonstrably pre-existing.

---

# Task 1 — Introduce an explicit type-formation outcome algebra

**Primary file:** `phalcom-semantic/src/types/annotation.rs`  
**Supporting files:** `phalcom-semantic/src/types/outcome.rs`, `phalcom-semantic/src/diagnostic.rs`  
**Tests:** `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`

Current code anchor:

```rust
pub enum TypeFormResolution {
    Known(TypeId),
    Dynamic,
    Unknown(UnknownReason),
}
```

This is the first thing to replace.

## Step 1.1 — Add failing outcome-classification tests

- [ ] In `type_annotations.rs`, add one test for each of these current defects:

1. invalid type application does not report `UnannotatedDeclaration`;
2. unresolved name remains unresolved;
3. unsaturated proper-type annotation is invalid, not “unannotated”;
4. invalid/recovered type syntax is invalid;
5. explicit `Dynamic` remains a distinct outcome.

Suggested test names:

```rust
type_formation_distinguishes_unresolved_from_invalid
type_formation_never_uses_unannotated_for_application_failure
proper_type_boundary_reports_unsaturated_constructor_as_invalid
dynamic_type_formation_remains_explicit
```

- [ ] Make the test assertions target the new result categories before implementing them. They should fail to compile until Step 1.2.

## Step 1.2 — Add reason enums

- [ ] In `types/annotation.rs`, immediately above `TypeFormResolution`, add:

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

If existing source makes an additional payload clearly necessary, add it now rather than storing a free-form string.

## Step 1.3 — Replace `TypeFormResolution`

- [ ] Replace:

```rust
pub enum TypeFormResolution {
    Known(TypeId),
    Dynamic,
    Unknown(UnknownReason),
}
```

with a generic outcome:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationOutcome<T> {
    Ready(T),
    Dynamic,
    Missing(TypeFormationMissing),
    Unresolved(TypeFormationUnresolved),
    Invalid(TypeFormationInvalid),
    Blocked(crate::types::outcome::BlockReason),
    Cancelled,
    BudgetExceeded(crate::types::outcome::BudgetReport),
    InternalFailure(String),
}

pub type TypeFormResolution = TypeFormationOutcome<TypeId>;
pub type KindResolution = TypeFormationOutcome<KindId>;
```

- [ ] Add convenience methods:

```rust
impl<T> TypeFormationOutcome<T> {
    pub fn ready(value: T) -> Self { Self::Ready(value) }
    pub fn as_ready(&self) -> Option<&T> { ... }
    pub fn into_ready(self) -> Option<T> { ... }
    pub fn is_terminal_failure(&self) -> bool { ... }
}
```

Do not add a convenience method that maps every non-ready state to one `UnknownReason`.

## Step 1.4 — Add propagation helpers

- [ ] Add a helper that converts a nested `TypeFormationOutcome<A>` into a parent outcome without reclassifying it.

A simple pattern is preferable:

```rust
macro_rules! ready_or_propagate {
    ($expr:expr) => {
        match $expr {
            TypeFormationOutcome::Ready(value) => value,
            TypeFormationOutcome::Dynamic => return TypeFormationOutcome::Dynamic,
            TypeFormationOutcome::Missing(reason) => return TypeFormationOutcome::Missing(reason),
            TypeFormationOutcome::Unresolved(reason) => return TypeFormationOutcome::Unresolved(reason),
            TypeFormationOutcome::Invalid(reason) => return TypeFormationOutcome::Invalid(reason),
            TypeFormationOutcome::Blocked(reason) => return TypeFormationOutcome::Blocked(reason),
            TypeFormationOutcome::Cancelled => return TypeFormationOutcome::Cancelled,
            TypeFormationOutcome::BudgetExceeded(report) => return TypeFormationOutcome::BudgetExceeded(report),
            TypeFormationOutcome::InternalFailure(failure) => return TypeFormationOutcome::InternalFailure(failure),
        }
    };
}
```

If the project avoids local macros for this style, implement a small `map_ready`/`and_then` API instead.

## Step 1.5 — Keep checker `TypeKnowledge` conversion at one boundary

- [ ] Locate `resolve_type_annotation`.
- [ ] Change it so it is the proper-type/value-annotation adapter.
- [ ] Do **not** let arbitrary callers reconstruct the conversion themselves.

Target behavior:

```rust
match resolve_type_form(...) {
    TypeFormationOutcome::Ready(form) if store.kind_of(form) == KindId::TYPE => {
        TypeKnowledge::established(form, EvidenceOrigin::DeveloperAnnotation)
    }
    TypeFormationOutcome::Ready(form) => {
        // emit KindExpectedType
        TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
    }
    TypeFormationOutcome::Dynamic => {
        TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)
    }
    TypeFormationOutcome::Unresolved(TypeFormationUnresolved::Name(name)) => {
        TypeKnowledge::Unknown(UnknownReason::UnresolvedName(name))
    }
    TypeFormationOutcome::Invalid(_) => {
        TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
    }
    TypeFormationOutcome::Blocked(_) => {
        TypeKnowledge::Unknown(UnknownReason::InferenceBlocked)
    }
    TypeFormationOutcome::Cancelled => {
        TypeKnowledge::Unknown(UnknownReason::InferenceCancelled)
    }
    TypeFormationOutcome::BudgetExceeded(_) => {
        TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded)
    }
    TypeFormationOutcome::Missing(_) | TypeFormationOutcome::InternalFailure(_) => {
        TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
    }
}
```

Use a more domain-specific existing `UnknownReason` if one already exists by implementation time; do not invent a false semantic type.

## Step 1.6 — Compile and fix exhaustive matches

- [ ] Run:

```bash
cargo check -p phalcom-semantic
```

- [ ] Fix every exhaustive `TypeFormResolution` match.
- [ ] Do not use `_ => Unknown(...)`.
- [ ] For each caller, preserve the exact terminal category.

## Step 1.7 — Run focused tests

- [ ] Run:

```bash
cargo test -p phalcom-semantic type_annotations
```

**Expected result:** New outcome tests pass; old valid annotation tests still pass.

---

# Task 2 — Make kind lowering explicit and non-recovering

**File:** `phalcom-semantic/src/types/annotation.rs`  
**Tests:** `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`

Current defect:

```rust
KindSyntax::Invalid { .. } => KindId::TYPE
```

## Step 2.1 — Add the failing invalid-kind test

- [ ] Construct or parse a generic declaration with recovered/invalid kind syntax.
- [ ] Assert:
  - a diagnostic is emitted;
  - no valid generic signature is returned;
  - no parameter is published with `KindId::TYPE` merely because syntax was invalid.

## Step 2.2 — Change the signature

- [ ] Replace:

```rust
pub fn resolve_kind_syntax(store: &mut TypeStore, kind: &KindSyntax) -> KindId
```

with:

```rust
pub fn resolve_kind_syntax(
    store: &mut TypeStore,
    kind: &KindSyntax,
) -> KindResolution
```

If cancellation/budget control is introduced as a context in Task 3, add the control argument at that point rather than changing this function twice.

## Step 2.3 — Rewrite every kind arm

- [ ] `Type` -> `Ready(KindId::TYPE)`.
- [ ] `RecordRow` -> `Ready(KindId::RECORD_ROW)`.
- [ ] Arrow:
  - recursively resolve each parameter;
  - propagate any non-ready result;
  - recursively resolve result kind;
  - call `store.arrow_kind`;
  - return `Ready(kind_id)`.
- [ ] Invalid -> `Invalid(TypeFormationInvalid::InvalidKindSyntax)`.

## Step 2.4 — Update all callers

Search:

```bash
rg "resolve_kind_syntax" phalcom-semantic
```

- [ ] Update `resolve_generic_signature`.
- [ ] Update source type-lambda lowering.
- [ ] Update declaration predeclaration code in `session.rs` if it directly calls this helper.
- [ ] Update tests/utilities.

No caller may unwrap an invalid kind to `Type`.

## Step 2.5 — Run tests

```bash
cargo test -p phalcom-semantic type_annotations
cargo check -p phalcom-semantic
```

---

# Task 3 — Replace the `TypeId`-only lexical generic binding model

**Files:**

- Modify: `phalcom-semantic/src/types/annotation.rs`
- Modify as needed: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/declaration_signature.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Tests: `type_annotations.rs`

The current trait method:

```rust
fn resolve_type_parameter(&self, name: &str) -> Option<TypeId>
```

cannot represent `R: RecordRow`.

## Step 3.1 — Add the domain enum

- [ ] Add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeLevelBinding {
    TypeForm(TypeId),
    RecordRow(crate::types::parameter::TypeParameterId),
}
```

## Step 3.2 — Change `TypeResolver`

- [ ] Replace the primary generic lookup API with:

```rust
fn resolve_type_level_binding(&self, _name: &str) -> Option<TypeLevelBinding> {
    None
}
```

- [ ] Keep a temporary compatibility method only if necessary:

```rust
fn resolve_type_parameter(&self, name: &str) -> Option<TypeId> {
    match self.resolve_type_level_binding(name) {
        Some(TypeLevelBinding::TypeForm(form)) => Some(form),
        _ => None,
    }
}
```

Mark this compatibility method for deletion within SC-1.

## Step 3.3 — Change resolver storage

- [ ] Change `SimpleTypeResolver.type_parameters` from:

```rust
HashMap<String, TypeId>
```

to:

```rust
HashMap<String, TypeLevelBinding>
```

- [ ] Change `ScopedTypeResolver.type_parameters` the same way.

- [ ] Rename the field to `type_level_bindings` if doing so does not create excessive churn. The clearer name is preferred.

## Step 3.4 — Add insertion helpers

- [ ] Replace/extend:

```rust
insert_parameter(name, ty)
```

with:

```rust
insert_type_form_binding(name, form)
insert_record_row_binding(name, parameter_id)
```

## Step 3.5 — Change reference lowering

In `resolve_type_form`, when resolving an unqualified reference:

- [ ] If `TypeLevelBinding::TypeForm(form)`, return `Ready(form)`.
- [ ] If `TypeLevelBinding::RecordRow(_)`, emit `KindExpectedType` or a dedicated diagnostic and return:

```rust
Invalid(TypeFormationInvalid::ExpectedProperType {
    actual: KindId::RECORD_ROW,
})
```

A row binding becomes legal only in the specific record-tail branch owned by SC-3.

## Step 3.6 — Fix generic-signature binder environment construction

Inside `resolve_generic_signature`:

- [ ] Intern every `TypeParameterData`.
- [ ] Read its kind.
- [ ] For `KindId::RECORD_ROW`, insert `TypeLevelBinding::RecordRow(param_id)`.
- [ ] For all ordinary/arrow kinds, call `store.parameter_form(param_id)` and insert `TypeLevelBinding::TypeForm(form)`.

This is the direct fix for the current `parameter_form` assertion risk.

## Step 3.7 — Add row-binder safety test

Add:

```rust
#[test]
fn record_row_generic_binder_does_not_create_type_parameter_form() { ... }
```

Assert:

- signature parameter exists;
- its stored kind is `KindId::RECORD_ROW`;
- constructing the signature does not panic;
- a direct use of `R` as an ordinary proper type is invalid.

## Step 3.8 — Remove compatibility lookup

After all production call sites use `resolve_type_level_binding`:

```bash
rg "resolve_type_parameter" phalcom-semantic
```

- [ ] Delete the compatibility method if no non-test user remains.
- [ ] Update expression-level type-parameter lookup to intentionally accept only `TypeLevelBinding::TypeForm`.

---

# Task 4 — Build one capture-safe scoped source type-form lowerer

**Files:**

- Modify: `phalcom-semantic/src/types/annotation.rs`
- Reuse: `phalcom-semantic/src/types/type_lambda.rs`
- Possibly create: `phalcom-semantic/src/types/scoped_lowering.rs`
- Tests: `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`

Prefer creating `types/scoped_lowering.rs` if `annotation.rs` becomes harder to navigate after Task 3.

## Step 4.1 — Write failing binder tests first

Add tests:

```rust
type_lambda_body_uses_bound_node
type_lambda_alpha_renaming_is_semantically_equal
nested_type_lambda_preserves_outer_and_inner_binders
type_lambda_keeps_declaration_parameter_free
type_lambda_beta_reduction_substitutes_without_capture
partial_type_lambda_application_returns_residual_lambda
```

The first test must inspect `TypeLambdaArena`/`ScopedTypeData` rather than merely checking that `TypeData::Lambda(_)` exists.

For `<T> =>> T`, assert:

```rust
matches!(
    arena.get_scoped(lambda.body),
    ScopedTypeData::Bound { depth: 0, index: 0 }
)
```

## Step 4.2 — Add binder-stack types

Add a private implementation type:

```rust
#[derive(Clone, Debug)]
struct ScopedBinder {
    name: Box<str>,
    kind: KindId,
}

#[derive(Default)]
struct ScopedBinderStack {
    layers: Vec<Box<[ScopedBinder]>>,
}
```

Add:

```rust
fn resolve(&self, name: &str) -> Option<(u32, u32, KindId)>
```

Search layers from innermost to outermost.

Return:

- `depth = 0` for innermost layer;
- increasing depth for outer layers;
- binder index within the layer.

## Step 4.3 — Add `lower_scoped_type_form`

Implement:

```rust
fn lower_scoped_type_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    aliases: &TypeAliasTable, // add after Task 10; use an empty/optional view temporarily
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    binders: &mut ScopedBinderStack,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<ScopedTypeId>
```

If aliases do not exist yet, start with the current declaration parameters and add the alias argument in Task 10.

## Step 4.4 — Implement reference lowering

Order:

1. scoped lambda binder -> `ScopedTypeData::Bound`;
2. enclosing lexical `TypeLevelBinding::TypeForm` -> `ScopedTypeData::Free(form)`;
3. `RecordRow` binding in an ordinary scoped type position -> invalid kind;
4. builtins (`Never`, `Unit`);
5. declaration/alias reference -> `ScopedTypeData::Free(form)`;
6. unresolved -> explicit unresolved result.

`Dynamic` is not a canonical scoped type node. If dynamic is allowed in a type-form subposition by language policy, represent it as a separate outcome and let the containing form propagate that boundary rather than inventing a `TypeId`.

## Step 4.5 — Implement structural forms

For each AST form, intern the corresponding scoped node:

- application -> `ScopedTypeData::Applied`;
- union -> `ScopedTypeData::Union`;
- tuple -> `ScopedTypeData::Tuple`;
- closed record -> `ScopedTypeData::Record`;
- callable -> `ScopedTypeData::Callable`;
- nested type lambda -> recursively create `TypeLambdaId`, then `ScopedTypeData::Lambda`.

Each field/element/parameter that requires a proper type must verify the resolved scoped expression's kind.

Add a helper:

```rust
fn scoped_kind(
    store: &mut TypeStore,
    scoped: ScopedTypeId,
    binders: &ScopedBinderStack,
) -> TypeFormationOutcome<KindId>
```

or reuse an existing arena kind computation if it already exposes the required operation.

## Step 4.6 — Handle open record tails honestly

If the AST record has `tail.is_some()`:

- [ ] emit `OpenRecordTailUnavailable`;
- [ ] return `Invalid(TypeFormationInvalid::UnsupportedOpenRecordTail)` or the chosen blocked variant;
- [ ] do not intern a closed `ScopedTypeData::Record`.

## Step 4.7 — Replace the current source lambda lowering

Delete the current sequence equivalent to:

```rust
let body_res = resolve_type_form(... body ...);
let scoped_body = store.arena_mut().intern_scoped(ScopedTypeData::Free(body_ty));
```

Replace it with:

1. resolve parameter kinds;
2. push one binder layer;
3. lower body through `lower_scoped_type_form`;
4. compute body/result kind;
5. pop binder layer;
6. call `store.lambda(...)`;
7. attach `TypeLambdaProvenance` with source parameter names/ranges.

## Step 4.8 — Run focused tests

```bash
cargo test -p phalcom-semantic type_lambda
```

Then:

```bash
cargo test -p phalcom-semantic type_annotations
```

---

# Task 5 — Make generic-signature lowering atomic and variance-aware

**Files:**

- Modify: `phalcom-semantic/src/types/annotation.rs`
- Modify: `phalcom-semantic/src/types/parameter.rs` if helper validation belongs there
- Modify: `phalcom-semantic/src/diagnostic.rs`
- Tests: `type_annotations.rs`

## Step 5.1 — Change `resolve_generic_signature` return type

Replace:

```rust
pub fn resolve_generic_signature(...) -> GenericSignature
```

with:

```rust
pub fn resolve_generic_signature(...)
    -> TypeFormationOutcome<GenericSignature>
```

Do not return an empty/partial signature after an error.

## Step 5.2 — Resolve all parameter kinds before interning the final signature

- [ ] Build a temporary vector:

```rust
struct PendingGenericParameter {
    name: Box<str>,
    kind: KindId,
    variance: Variance,
    source: ...,
}
```

- [ ] If any kind is non-ready, return the non-ready result before publishing a valid `GenericSignature`.

Stable `TypeParameterId` allocation may still occur as part of semantic identity, but the declaration table must not expose the signature as valid until all inputs succeed.

## Step 5.3 — Validate variance placement

Add a parameter describing the binder site:

```rust
pub enum GenericBinderSite {
    NominalDeclaration,
    Callable,
    TypeAlias,
}
```

Add it to `resolve_generic_signature`.

Rules:

```text
NominalDeclaration -> source variance allowed
Callable           -> non-invariant source variance rejected
TypeAlias          -> non-invariant source variance rejected
```

Type-lambda parameters do not use `resolve_generic_signature`; their AST already has a separate parameter form.

## Step 5.4 — Build the domain-aware scoped resolver

For each parameter:

```rust
match data.kind {
    KindId::RECORD_ROW => {
        bindings.insert(name, TypeLevelBinding::RecordRow(param_id));
    }
    _ => {
        let form = store.parameter_form(param_id);
        bindings.insert(name, TypeLevelBinding::TypeForm(form));
    }
}
```

## Step 5.5 — Lower every `where` constraint

For each `GenericConstraintSyntax`:

- [ ] lower both operands;
- [ ] require each to be an ordinary type form;
- [ ] if either operand is row-domain or invalid, fail the signature;
- [ ] append exactly one canonical `GenericConstraint`.

Delete conditional code equivalent to:

```rust
if let (Known(left), Known(right)) = (...) {
    constraints.push(...)
}
```

that silently drops failures.

## Step 5.6 — Add publishability validation

In `types/parameter.rs`, add:

```rust
impl GenericSignature {
    pub fn validate_publishable(&self, store: &TypeStore) -> Result<(), GenericSignaturePublicationError> {
        ...
    }
}
```

Check:

- parameter IDs exist in the store;
- each parameter owner matches `self.owner`;
- indices are contiguous and in signature order;
- no constraint contains a solver-local `InferVarId`;
- every canonical constraint term is a store-owned canonical form;
- `RecordRow` parameters are not exposed as `TypeData::Parameter`.

Call this before DB/snapshot publication in Task 6.

## Step 5.7 — Update call sites exhaustively

Search:

```bash
rg "resolve_generic_signature" phalcom-semantic
```

Update:

- `session.rs` class publication;
- `session.rs` enum publication;
- `checker/declaration_signature.rs` method publication;
- tests;
- later alias publication.

Each caller must handle non-ready results explicitly and attach diagnostics/blocked state rather than using a default empty signature.

## Step 5.8 — Run tests

```bash
cargo test -p phalcom-semantic generic_signature
cargo check -p phalcom-semantic
```

---

# Task 6 — Consolidate source declaration generic publication

**Files:**

- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/declarations.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Tests: `phalcom-semantic/tests/semantic/integration/workspace.rs`

Current `session.rs` already:

1. predeclares generic class/enum constructor kinds;
2. later calls `resolve_generic_signature`;
3. later reinserts `DeclarationTypeInfo`.

Do not replace this with an older monomorphic implementation.

## Step 6.1 — Add consistency tests

Add tests proving:

```text
DeclarationTypeInfo.kind
==
kind derived from GenericSignature parameter kinds
```

for:

```phalcom
class Box<T> {}
class Transformer<F: Type -> Type, T> {}
```

Add a negative test where invalid kind syntax cannot leave a ready declaration with a mismatched kind.

## Step 6.2 — Add a declaration-header builder

In `declarations.rs` add a helper data type:

```rust
pub struct NominalDeclarationHeader {
    pub declaration: DeclarationId,
    pub form: TypeId,
    pub class_object_type: TypeId,
    pub kind: KindId,
    pub generic_signature: Option<GenericSignature>,
}
```

or equivalent.

Add a constructor/helper that computes `kind` from the successful generic signature.

The purpose is to stop separately computing binder kinds in two source passes with potentially different recovery behavior.

## Step 6.3 — Keep a minimal predeclaration shell

The first source pass still needs stable declaration identity and a form kind for mutual references.

Use parsed binder kind syntax only to build a **predeclaration shell outcome**.

Do not call invalid kind syntax `Type`.

If a declaration's kind is invalid:

- keep its stable declaration shell for diagnostics/cycle ownership;
- mark semantic realization invalid/blocked;
- do not publish a successful `DeclarationTypeInfo`.

## Step 6.4 — Publish the finalized header exactly once

After module shells/resolvers are ready:

1. resolve generic signature;
2. derive final constructor kind from signature parameter kinds;
3. create/lookup nominal form using that exact kind;
4. create class-object type;
5. lower superclass template;
6. validate publishability;
7. publish the DB declaration shell.

Avoid repeatedly reinserting semantically different `DeclarationTypeInfo` into a mutable table during one realization pass.

## Step 6.5 — Harden fingerprints

Ensure `declaration_shell_input_fingerprint` and product fingerprint include:

- kind structure;
- generic parameter kinds;
- generic parameter variance;
- constraints;
- supertype template structure.

Use stable structural export/fingerprint logic, not raw `TypeId` integer values as cross-store meaning.

## Step 6.6 — Run workspace tests

```bash
cargo test -p phalcom-semantic workspace
cargo test -p phalcom-semantic fingerprints
```

---

# Task 7 — Make `Self` owner/side explicit

**Files:**

- Modify: `phalcom-semantic/src/types/annotation.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/declaration_signature.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Tests: add `phalcom-semantic/tests/semantic/foundations/self_types.rs` or extend existing receiver/inheritance tests.

Current `resolve_type_form` obtains only `resolver.current_declaration()` and hardcodes:

```rust
side: DispatchSide::Instance
```

## Step 7.1 — Add `TypeFormationSite`

In `types/annotation.rs` add:

```rust
#[derive(Clone, Debug)]
pub struct TypeFormationSite {
    pub module: ModuleId,
    pub self_term: Option<SelfTypeTerm>,
}

impl TypeFormationSite {
    pub fn module(module: ModuleId) -> Self { ... }

    pub fn member(
        module: ModuleId,
        owner: DeclarationId,
        side: DispatchSide,
    ) -> Self {
        Self {
            module,
            self_term: Some(SelfTypeTerm {
                owner,
                side,
                role: SelfRole::InstanceType,
            }),
        }
    }
}
```

If the class-side role requires a distinct existing `SelfRole`, use the ratified representation already used by the receiver-specialization code. Do not invent a role solely for presentation.

## Step 7.2 — Add the site to lowering APIs

Replace `current_module: &ModuleId` parameters in:

- `resolve_type_form`;
- `resolve_type_annotation`;
- scoped lowering;
- `resolve_generic_signature` where constraints need `Self`;

with `site: &TypeFormationSite` or add `site` alongside the module until the migration is complete.

Use `site.module` for diagnostics/name resolution.

## Step 7.3 — Replace `SelfType` lowering

Replace the current branch with:

```rust
TypeAnnotationExpr::SelfType { range } => match &site.self_term {
    Some(term) => TypeFormationOutcome::Ready(store.self_type(term.clone())),
    None => {
        diagnostics.push(...SelfOutsideTypeContext...);
        TypeFormationOutcome::Unresolved(TypeFormationUnresolved::SelfOutsideOwner)
    }
}
```

Delete the instance-side hardcode.

## Step 7.4 — Construct the correct site for every member

In `checker/declaration_signature.rs`:

- [ ] compute `side` before resolving annotations;
- [ ] construct `TypeFormationSite::member(module, owner, side)`;
- [ ] use it for parameter annotations;
- [ ] use it for return annotations;
- [ ] use it for method `where` constraints.

In body/field paths, use `ctx.current_side`.

In session-level class generic/superclass lowering, use instance-side owner context where `Self` is permitted.

## Step 7.5 — Add tests

Test:

- instance getter returning `Self`;
- class-side getter/method returning `Self`;
- `Self` outside class context;
- inherited source signature containing `Self`.

Inspect the `SelfTypeTerm` and assert owner + side exactly.

## Step 7.6 — Delete `TypeResolver::current_declaration`

After all `Self` lowering uses `TypeFormationSite`:

```bash
rg "current_declaration" phalcom-semantic
```

If the trait method has no other semantic purpose:

- [ ] remove it from `TypeResolver`;
- [ ] remove `SimpleTypeResolver.enclosing_declaration`;
- [ ] update tests to construct a `TypeFormationSite`.

This deletion makes it impossible to reintroduce the owner/side ambiguity accidentally.

---

# Task 8 — Enforce side-aware declaration generic scope

**Files:**

- Modify: `phalcom-semantic/src/checker/declaration_signature.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` if a shared helper belongs there
- Tests: new/extended callable signature tests

Current `semantic_signature_for_syntax` builds declaration generic parameter bindings before dispatch-side specialization. Current `check_class_bodies` likewise constructs one resolver for the entire generic class.

## Step 8.1 — Add negative test first

Create a generic class with a class-side member that refers to ambient `T` without declaring its own generic.

Use the existing class-side syntax/attribute (`@class` or static syntax).

Assert:

- the class-side member annotation cannot resolve `T`;
- an appropriate unresolved generic/type annotation diagnostic is produced.

Add a positive instance-side test proving `T` still resolves.

## Step 8.2 — Add helper

In `checker/declaration_signature.rs` or `checker/context.rs` add:

```rust
fn declaration_type_level_bindings_for_side(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    side: DispatchSide,
) -> HashMap<String, TypeLevelBinding>
```

Implementation:

```rust
if side == DispatchSide::Class {
    return HashMap::new();
}

let Some(sig) = ctx.declaration_generic_signature(owner) else {
    return HashMap::new();
};

sig.parameters
    .iter()
    .map(|id| {
        let data = ctx.store.type_parameter(*id);
        let binding = if data.kind == KindId::RECORD_ROW {
            TypeLevelBinding::RecordRow(*id)
        } else {
            TypeLevelBinding::TypeForm(ctx.store.parameter_form(*id))
        };
        (data.name.to_string(), binding)
    })
    .collect()
```

Be careful with Rust borrow order: clone parameter IDs/data needed before mutably borrowing `store` for `parameter_form`.

## Step 8.3 — Use helper in signature lowering

In `semantic_signature_for_syntax`:

1. determine `side`;
2. call the helper;
3. build `ScopedTypeResolver` from those bindings;
4. for a generic method, overlay method-local binders on top.

Method-local generics shadow declaration generics by ordinary lexical scope rules.

## Step 8.4 — Use helper in body checking

In `check_class_bodies`:

- [ ] remove the one `type_params_map` built before iterating members;
- [ ] inside the member loop, after computing `side`, build the resolver for that side;
- [ ] check that member using that resolver.

If production workspace body analysis no longer uses this compatibility function, still fix or delete it; do not leave a second semantically wrong path.

## Step 8.5 — Constructor special case

Audit constructor signature lowering.

- [ ] Do not re-enable ambient class-side `T` merely because constructors return instances.
- [ ] Constructor result formation should use the canonical owner/receiver specialization mechanism.
- [ ] If a constructor needs its own generic variables, they are callable generics.

Add a regression test.

## Step 8.6 — Run tests

```bash
cargo test -p phalcom-semantic declaration_signature
cargo test -p phalcom-semantic generic
```

---

# Task 9 — Harden generic superclass formation

**Files:**

- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/declarations.rs`
- Modify relation/substitution code only if a verified bug is found
- Tests: `workspace.rs`, receiver/inheritance tests

## Step 9.1 — Split “absent” from “written but failed”

Current `DeclarationTypeInfo` uses:

```rust
pub supertype_template: Option<GenericSupertypeTemplate>
```

That represents successful presence/absence but not formation status.

Add a declaration-realization status or keep failure in the query state/diagnostic product.

Preferred approach:

- successful no-superclass -> `supertype_template: None`;
- successful superclass -> `Some(template)`;
- written invalid superclass -> declaration shell query is not `Ready`.

Do not publish invalid written superclass as `None`.

## Step 9.2 — Update session lowering

When processing `class_def.superclass`:

```rust
match resolve_type_form(...) {
    Ready(form) => { require kind Type; create template; }
    ... => { preserve exact terminal state; do not publish ready declaration; }
}
```

## Step 9.3 — Add kind gate

If the final superclass form kind is not `Type`:

- emit `KindExpectedType`;
- return invalid declaration realization;
- never put a constructor-kinded form in `GenericSupertypeTemplate`.

## Step 9.4 — Verify specialization

Extend existing generic superclass tests to inspect:

```text
Box<T> -> Container<T>
Box<Int> -> Container<Int>
```

Use canonical substitution APIs rather than comparing source strings.

## Step 9.5 — Verify runtime invariance

If there are runtime class/superclass metadata tests, add an assertion that adding generic static metadata does not change erased runtime superclass identity.

---

# Task 10 — Implement transparent type aliases end-to-end

This is the largest SC-1 task.

**Files:**

- Modify: `phalcom-modules/src/interface.rs`
- Modify: module/interface tests
- Modify: `phalcom-semantic/src/declarations.rs`
- Create: `phalcom-semantic/src/type_alias.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-semantic/src/types/annotation.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/db/key.rs`
- Modify: `phalcom-semantic/src/db/product.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: source-index/navigation projection if required to expose alias declaration identity
- Tests: module interface, type annotations, workspace, incremental fingerprints

## Step 10.1 — Make aliases real module declarations

In `InterfaceBuilder::build`, Pass 1 currently handles `Class`, `Enum`, and `Let`.

Add:

```rust
Statement::TypeAlias(alias) => {
    let range = (alias.range.start..alias.name_range.end).into();
    Self::validate_dunder(&alias.name, DunderRole::Binding, range)?;
    Self::collect_declaration(
        &alias.name,
        true,
        range,
        &mut namespace,
        &mut declarations,
    )?;
}
```

Use the precise available alias range fields from `TypeAliasDef`.

- [ ] Add interface tests for:
  - alias declaration exists;
  - duplicate alias/class name rejected;
  - alias can be exported;
  - imported alias resolves as a linked binding;
  - alias has no runtime initialization dependency merely because its type body references another type.

## Step 10.2 — Ensure module declaration blueprints classify aliases

Find the code that converts parsed declarations into `DeclarationBlueprint`.

- [ ] Add/verify:

```rust
Statement::TypeAlias(alias) => DeclarationBlueprint {
    id: ...,
    kind: DeclarationKind::Alias,
}
```

`phalcom-modules/src/declaration.rs::DeclarationKind::Alias` already exists; use it.

## Step 10.3 — Create `type_alias.rs`

Add:

```rust
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::DeclarationId;
use crate::types::{GenericSignature, KindId, TypeId};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TypeAliasInfo {
    pub declaration: DeclarationId,
    pub kind: KindId,
    pub generic_signature: Option<GenericSignature>,
    pub form: TypeId,
    pub dependencies: Box<[DeclarationId]>,
    pub source: SemanticSourceSpan,
}

#[derive(Clone, Debug, Default)]
pub struct TypeAliasTable {
    aliases: HashMap<DeclarationId, TypeAliasInfo>,
}
```

Add methods:

```rust
new()
insert(info)
get(&DeclarationId)
form(&DeclarationId) -> Option<TypeId>
generic_signature(&DeclarationId)
iter()
contains_key()
```

If the project standardizes on `BTreeMap` for deterministic products, use `BTreeMap` instead of `HashMap`.

## Step 10.4 — Generalize the semantic declaration shell product

In `declarations.rs` add:

```rust
#[derive(Clone, Debug)]
pub enum TypeDeclarationShell {
    Nominal(DeclarationTypeInfo),
    Alias(crate::type_alias::TypeAliasInfo),
}

impl TypeDeclarationShell {
    pub fn declaration(&self) -> &DeclarationId { ... }
}
```

Do not add `class_object_type` to `TypeAliasInfo`.

## Step 10.5 — Change `SemanticProduct::DeclarationShell`

In `db/product.rs` replace:

```rust
DeclarationShell(Arc<DeclarationTypeInfo>)
```

with:

```rust
DeclarationShell(Arc<TypeDeclarationShell>)
```

Update:

```rust
as_declaration_shell()
```

to return `Arc<TypeDeclarationShell>`.

Add convenience accessors if useful:

```rust
as_nominal_declaration_shell()
as_alias_declaration_shell()
```

Do not make downstream callers match raw product variants repeatedly if a helper is clearer.

## Step 10.6 — Change query fingerprinting

In `db/fingerprint.rs`:

- [ ] change `declaration_shell_input_fingerprint` to accept `TypeDeclarationShell`;
- [ ] change product fingerprint likewise;
- [ ] dispatch to nominal or alias structural hashing.

Add:

```rust
fn hash_type_alias_info(
    info: &TypeAliasInfo,
    hasher: &mut impl Hasher,
)
```

Hash structurally:

- declaration stable identity;
- kind via stable kind traversal;
- generic signature parameter kinds/variance/constraints;
- alias form via stable type traversal;
- dependencies sorted by stable declaration identity.

Do not hash source ranges into the semantic product fingerprint unless the existing DB distinguishes source-identity and semantic-product fingerprints. Range-only edits should not invalidate formal consumers.

## Step 10.7 — Update `query_declaration_shell`

Change:

```rust
pub fn query_declaration_shell(
    db: &mut SemanticDb,
    info: Arc<DeclarationTypeInfo>,
) -> QueryOutcome<Arc<DeclarationTypeInfo>>
```

to:

```rust
pub fn query_declaration_shell(
    db: &mut SemanticDb,
    shell: Arc<TypeDeclarationShell>,
) -> QueryOutcome<Arc<TypeDeclarationShell>>
```

Use `shell.declaration()` for `QueryKey::DeclarationShell`.

Update all callers.

## Step 10.8 — Add alias table to snapshot

In `snapshot.rs` add:

```rust
pub type_aliases: Arc<TypeAliasTable>,
```

Initialize it in every `SemanticSnapshot` constructor.

Add:

```rust
pub fn with_type_aliases(
    mut self,
    aliases: Arc<TypeAliasTable>,
) -> Self {
    self.type_aliases = aliases;
    self
}
```

Update all snapshot construction call sites and tests.

## Step 10.9 — Predeclare alias binder kinds

In `session.rs`, before alias body lowering:

1. discover every `Statement::TypeAlias`;
2. allocate stable `DeclarationId`;
3. resolve generic binder kinds only;
4. compute alias constructor kind:

```text
no binders -> kind of body, determined during realization
binders    -> (binder kinds...) -> body kind
```

Because the body kind can itself be constructor-kinded, do not assume alias result kind is always `Type`.

For a generic alias, the final form is a canonical type lambda whose result kind is the body kind.

## Step 10.10 — Add alias-specific name lookup

Extend type-form resolution to consult both:

- `DeclarationTypeTable` for nominal/ADT forms;
- `TypeAliasTable` for alias forms.

Replace the current reference fallback:

```rust
declarations.form(&decl)
    .unwrap_or_else(|| store.nominal_type(decl))
```

with explicit logic:

```rust
if let Some(form) = declarations.form(&decl) {
    Ready(form)
} else if let Some(form) = aliases.form(&decl) {
    Ready(form)
} else {
    Missing(TypeFormationMissing::DeclarationProduct(decl))
}
```

Never fabricate a nominal type.

## Step 10.11 — Lower non-generic aliases

For:

```phalcom
type UserId = Int
```

- [ ] lower the body through ordinary `resolve_type_form`;
- [ ] use the resulting canonical form directly;
- [ ] `TypeAliasInfo.kind = store.kind_of(form)`;
- [ ] `generic_signature = None`;
- [ ] publish alias shell and table entry.

## Step 10.12 — Lower generic aliases through scoped lowering

For:

```phalcom
type Pair<T> = (T, T)
```

- [ ] resolve the alias generic signature with `GenericBinderSite::TypeAlias`;
- [ ] create one scoped binder layer from the signature parameter names/kinds;
- [ ] lower the alias body with `lower_scoped_type_form`;
- [ ] compute body result kind;
- [ ] call `store.lambda(parameter_kinds, scoped_body, result_kind)`;
- [ ] use the resulting lambda `TypeId` as `TypeAliasInfo.form`;
- [ ] use the lambda's full kind as `TypeAliasInfo.kind`.

Attach source provenance to the lambda arena using alias parameter names/ranges.

## Step 10.13 — Collect alias dependencies

During alias body lowering, record every resolved declaration reference.

At minimum collect alias-to-alias dependencies.

Store deterministic sorted unique dependencies in `TypeAliasInfo.dependencies`.

Record DB dependency edges:

```text
DeclarationShell(alias A)
    -> DeclarationShell(alias B)
```

for each referenced alias B.

If the body references a nominal declaration whose public type identity affects the alias form, record that declaration shell dependency too.

## Step 10.14 — Detect alias cycles before final publication

Build a directed graph over alias declarations using the collected alias dependencies.

Use an existing SCC implementation if available; do not add a second generic graph algorithm if `SemanticGraph::components()` already provides SCC components suitable for this purpose.

Reject:

- SCC with more than one alias;
- singleton SCC with a self edge.

Emit deterministic `TypeAliasCycle` diagnostics.

Do not repeatedly expand aliases to discover the cycle.

## Step 10.15 — Preserve transparent equality

Do not add `TypeData::Alias`.

A reference to an alias resolves to `TypeAliasInfo.form`.

Therefore:

```text
type UserId = Int
UserId
```

returns the same canonical form as `Int`.

Navigation identity comes from the resolved source occurrence/declaration, not from a distinct semantic type node.

## Step 10.16 — Wire source indexing/navigation

Find where `Statement::TypeAlias` is currently ignored in:

- `source_index/builder.rs`;
- `source_index/occurrence.rs`;
- editor navigation declaration registration.

Add declaration occurrence/definition identity so:

- go-to-definition on alias use reaches alias declaration;
- hover can show alias provenance while consuming the canonical expanded form;
- source index does not invent type equality.

This is semantic identity plumbing, not LSP-side alias inference.

## Step 10.17 — Add alias tests

Add tests covering:

1. `type UserId = Int`;
2. `type Pair<T> = (T, T)`;
3. constructor-kinded alias;
4. alias used in field/method annotation;
5. alias imported across module;
6. alias exported;
7. alias-to-alias chain;
8. self cycle;
9. mutual cycle;
10. generic alias alpha-safe body;
11. alias edit changes dependent signature fingerprint;
12. range-only alias edit does not change semantic fingerprint.

## Step 10.18 — Run tests

```bash
cargo test -p phalcom-modules interface
cargo test -p phalcom-semantic type_alias
cargo test -p phalcom-semantic fingerprints
cargo test -p phalcom-semantic workspace
```

---

# Task 11 — Correct value-space type-form semantics

**Files:**

- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` if a helper is needed
- Reuse: `phalcom-semantic/src/types/denotation.rs`
- Tests: add/extend denotation tests

Current code anchor:

```rust
Expr::TypeForm(annotation) => {
    let resolver = ctx.resolver.inner();
    let (knowledge, causal_invalidity) =
        ctx.resolve_type_annotation(resolver, annotation);
    ...
}
```

This is wrong for constructor-kinded forms because `resolve_type_annotation` requires kind `Type`.

## Step 11.1 — Write failing tests

Add tests for:

```phalcom
const a = Int
const b = List
const c = Map<String>
const d = <T> =>> List<T>
```

For each, inspect the expression's `SemanticDenotation::TypeForm(form)`.

Assert the denotation kinds:

```text
Int             -> Type
List            -> Type -> Type
Map<String>     -> Type -> Type
lambda          -> Type -> Type
```

Also assert each expression has a legitimate ordinary value `TypeKnowledge`, separate from the denoted form.

## Step 11.2 — Add a checker wrapper for `resolve_type_form`

In `CheckingContext` add:

```rust
pub fn resolve_type_form(
    &mut self,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    annotation: &TypeAnnotation,
) -> (TypeFormResolution, CausalInvalidity)
```

It should:

- collect diagnostics;
- emit them under the current checker causal frame;
- return the exact formation outcome;
- not enforce proper-kind `Type`.

Keep the existing `resolve_type_annotation` wrapper for annotations.

## Step 11.3 — Replace the `Expr::TypeForm` call

Use:

```rust
let (resolution, causal_invalidity) =
    ctx.resolve_type_form(resolver, &site, annotation);
```

Match all outcome variants explicitly.

## Step 11.4 — Determine the runtime/value type separately

For a declaration-backed form:

- resolve the root declaration from canonical form where possible;
- use the published declaration's `class_object_type`/descriptor value type.

For a lambda/structural constructor form:

- use the compiler's canonical type-form descriptor value type if one exists;
- if runtime materialization support is not yet available, use a formal descriptor/class-object supertype already defined by the semantic model;
- do **not** use the denoted form itself as the ordinary value type merely because it is convenient.

If the runtime/value descriptor typing for non-declaration-backed type forms is genuinely not implemented, preserve a clear semantic blocked/unknown **value type** while still retaining `SemanticDenotation::TypeForm(form)` only if the existing `TypedExpression` contract safely permits that. Prefer completing the descriptor value type if the repository already has the class/type-object core ID.

## Step 11.5 — Remove fallback class-object fabrication

Current code may call:

```rust
ctx.store.class_object_type(decl.clone())
```

when `ctx.declaration_info(decl)` is absent.

Replace this with explicit missing declaration-product handling.

A linked type declaration missing its semantic shell is an analyzer/publication failure, not a new class-object type.

## Step 11.6 — Run denotation tests

```bash
cargo test -p phalcom-semantic denotation
cargo test -p phalcom-semantic type_form
```

---

# Task 12 — Stop silently erasing open record tails

**File:** `phalcom-semantic/src/types/annotation.rs`  
**Tests:** `type_annotations.rs`

Current code:

```rust
TypeAnnotationExpr::Record {
    fields,
    tail: _,
    ...
}
```

## Step 12.1 — Add failing test

Construct:

```phalcom
#{ name: String, | R }
```

with `R: RecordRow`.

Assert the result is **not** the same as closed:

```phalcom
#{ name: String }
```

## Step 12.2 — Bind the tail variable in the pattern

Replace:

```rust
tail: _,
```

with:

```rust
tail,
```

## Step 12.3 — Add explicit SC-3 handoff

Before closed-record interning:

```rust
if tail.is_some() {
    diagnostics.push(SemanticDiagnostic::error_in(
        site.module.clone(),
        DiagnosticCode::OpenRecordTailUnavailable,
        "open record rows are parsed but their semantic row lowering is completed in SC-3",
        annotation.range,
    ));
    return TypeFormationOutcome::Invalid(
        TypeFormationInvalid::UnsupportedOpenRecordTail,
    );
}
```

If the project chooses `Blocked` rather than `Invalid` for staged feature availability, use that consistently in spec/tests.

## Step 12.4 — Keep closed records unchanged

The no-tail path continues canonical closed-row construction.

## Step 12.5 — Run tests

```bash
cargo test -p phalcom-semantic structural_record
cargo test -p phalcom-semantic type_annotations
```

---

# Task 13 — Remove semantic fabrication/recovery fallbacks

**Files:**

- `phalcom-semantic/src/types/annotation.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/context.rs`
- any declaration/signature resolver that manufactures nominal forms

## Step 13.1 — Search for declaration-form fabrication

Run:

```bash
rg "nominal_type\(" phalcom-semantic/src
rg "class_object_type\(" phalcom-semantic/src
rg "unwrap_or_else.*nominal" phalcom-semantic/src
```

Classify every occurrence:

1. legitimate canonical declaration creation;
2. legitimate bootstrap;
3. fallback after a supposedly resolved/published declaration;
4. test helper.

Only category 3 is removed.

## Step 13.2 — Replace resolved-name fallback

In `resolve_type_form`, replace:

```rust
declarations.form(&decl)
    .unwrap_or_else(|| store.nominal_type(decl))
```

with nominal table / alias table lookup followed by:

```rust
TypeFormationOutcome::Missing(
    TypeFormationMissing::DeclarationProduct(decl)
)
```

Emit an internal/blocked semantic diagnostic according to ownership.

## Step 13.3 — Replace expression fallback

For `Expr::Var` resolving a type name and for `Expr::SelfVar`/`SuperVar` class-side descriptor lookup:

- do not synthesize a nominal/class-object type if the declaration shell is missing;
- return explicit unavailable knowledge with a causal internal/block status.

## Step 13.4 — Search for `UnannotatedDeclaration` misuse

Run:

```bash
rg "UnannotatedDeclaration" phalcom-semantic/src/types phalcom-semantic/src/checker
```

For each use, ask:

> Is the source actually missing an annotation?

If no, replace it with the correct failure classification.

Do not globally delete `UnannotatedDeclaration`; it remains a legitimate value-analysis reason.

## Step 13.5 — Add regression tests

Tests must prove that removing a required semantic declaration product cannot accidentally create a new nominal `TypeId`.

---

# Task 14 — Close type-formation query dependencies and fingerprints

**Files:**

- `phalcom-semantic/src/db/query.rs`
- `phalcom-semantic/src/db/fingerprint.rs`
- `phalcom-semantic/src/checker/analysis.rs` if semantic dependency enum needs alias coverage
- `phalcom-semantic/src/session.rs`
- tests under `semantic/incremental/`

## Step 14.1 — Make alias/declaration dependencies explicit

When a callable signature lowers an annotation through alias `A`, ensure its dependency set contains:

```rust
SemanticDependency::DeclarationShell(A)
```

If the existing dependency recorder only sees final canonical `TypeId`, add resolution provenance to the lowering result or a dependency sink to the formation context.

Recommended addition:

```rust
pub trait TypeFormationDependencySink {
    fn declaration(&mut self, id: &DeclarationId);
}
```

The DB-backed lowering context records these into the query dependency recorder.

Simple unit tests can use a no-op sink.

## Step 14.2 — Record generic declaration dependencies

A source type application referencing a generic declaration must depend on that declaration shell, because changes to:

- parameter kind;
- variance;
- constraints;

can invalidate an unchanged consumer even when the nominal declaration name stays the same.

## Step 14.3 — Add incremental tests

Create a test matrix:

1. alias body semantic edit -> consumer recomputes;
2. alias range/comment edit -> consumer reuses;
3. generic parameter kind edit -> consumer signature recomputes;
4. generic `where` edit -> consumer recomputes;
5. unrelated body-only edit -> declaration/type consumer reuses;
6. superclass template edit -> inherited consumer recomputes.

Use `SemanticDb` metrics or query revision/fingerprint assertions already used in current incremental tests.

## Step 14.4 — Cold/incremental parity

For each edit scenario:

- create final source directly in a new session;
- arrive at same final source through incremental edits;
- export structural kind/type/signature/alias facts;
- compare for equality.

Never compare raw `TypeId` numbers across stores.

---

# Task 15 — Harden tests for canonical type-lambda and generic laws

**Files:**

- extend `type_annotations.rs`;
- optionally create `phalcom-semantic/tests/semantic/foundations/type_lambdas.rs`;
- optionally create `generic_declarations.rs`.

Add the following exact semantic law tests.

## Step 15.1 — Alpha equivalence

```text
<T> =>> List<T>
<U> =>> List<U>
```

must intern/equal semantically.

## Step 15.2 — Capture avoidance

Apply:

```text
(<T> =>> <U> =>> (T, U))<Int>
```

and inspect residual lambda body.

It must mean:

```text
<U> =>> (Int, U)
```

not:

```text
<U> =>> (U, U)
```

## Step 15.3 — Higher-kinded parameter

Test:

```phalcom
<F: Type -> Type, T> =>> F<T>
```

with `F := List`, `T := Int`.

Result must be `List<Int>`.

## Step 15.4 — Partial application kind

Test:

```text
Map<String>
```

kind exactly `Type -> Type`.

## Step 15.5 — Row binder negative law

`R: RecordRow` cannot appear as:

```phalcom
const x: R = ...
```

No panic, no `TypeData::Parameter`, correct diagnostic.

## Step 15.6 — No solver IDs publish

Construct all public SC-1 forms and traverse them.

Assert no canonical/public type node contains query-local inference IDs.

---

# Task 16 — Update semantic export/read-model support only where SC-1 requires it

**Files to inspect:**

- `phalcom-semantic/src/export.rs`
- `phalcom-type-meta/src/declaration.rs`
- current metadata exporter modules

`export.rs::CompiledTypeRef` currently rejects `TypeData::Lambda`, `SelfType`, `ClassObject`, and `Family`.

Do **not** turn this transitional recursive adapter into the final metadata architecture if the repository is already migrating to `phalcom-type-meta`.

## Step 16.1 — Identify the active durable metadata path

- [ ] Find all production callers of `export_type_form`.
- [ ] Find production constructors of `phalcom_type_meta::TypeAliasRecord`.
- [ ] Determine which path is authoritative for compiled metadata at current HEAD.

## Step 16.2 — Make SC-1 semantic facts transportable through the active path

At minimum the active durable path must be able to transport:

- declaration kind;
- generic parameter kind/variance;
- generic constraints;
- type lambda structure or an indexed equivalent;
- transparent alias declaration identity + target/form;
- `Self` as owner-relative semantic form where published signatures require it.

If this belongs to the later SC-6 metadata migration, add explicit compile-time guards/tests showing SC-1 snapshots contain the facts and document the remaining projection handoff. Do not create a second metadata schema.

## Step 16.3 — Add hostile/publication validation

Before exporting a signature/alias:

- validate all IDs belong to the current store/snapshot;
- reject solver variables;
- reject unresolved formation outcomes;
- reject cyclic aliases;
- reject row solver variables.

---

# Task 17 — Remove compatibility paths that reconstruct declaration signatures

**Files:**

- `phalcom-semantic/src/checker/declaration.rs`
- `phalcom-semantic/src/checker/declaration_signature.rs`
- `phalcom-semantic/src/session.rs`
- search all checker code

## Step 17.1 — Search for annotation re-lowering after signature publication

Run:

```bash
rg "return_annotation" phalcom-semantic/src/checker
rg "resolve_type_annotation" phalcom-semantic/src/checker
rg "resolve_generic_signature" phalcom-semantic/src/checker
```

Classify each call:

- declaration-time signature publication;
- body-local explicit annotation;
- duplicate reconstruction of an already-published callable signature.

## Step 17.2 — Remove duplicate signature reconstruction

For body analysis of a known `CallableId`:

- retrieve `CallableSemanticSignature`;
- bind canonical parameters from the signature;
- use canonical return contract;
- do not reparse method generic binders/where clauses.

If a compatibility path such as `check_class_bodies` remains only for old one-shot analysis, either:
- project it from canonical signatures; or
- delete it after proving no production caller needs it.

## Step 17.3 — Keep generic getters out

Do not generalize getter AST/signature generics here.

`Getter` must remain `generics: None` in SC-1.

The separate generic-getter plan will modify that intentionally.

---

# Task 18 — Full verification and deletion ledger

## Step 18.1 — Formatting

```bash
cargo fmt --all
cargo fmt --all -- --check
```

## Step 18.2 — Focused package tests

```bash
cargo test -p phalcom-modules
cargo test -p phalcom-semantic
```

## Step 18.3 — Workspace check

```bash
cargo check --workspace
```

## Step 18.4 — Workspace tests

If practical for repository runtime:

```bash
cargo test --workspace
```

If the full suite is too large, run all compiler/type/module/LSP packages affected by the diff and record exactly what was omitted.

## Step 18.5 — Clippy if repository policy uses it

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Use the project's actual CI invocation if it differs.

## Step 18.6 — Search-based deletion gates

Run and inspect:

```bash
rg "KindSyntax::Invalid" phalcom-semantic/src/types/annotation.rs
rg "tail: _" phalcom-semantic/src/types/annotation.rs
rg "ScopedTypeData::Free\(body" phalcom-semantic/src/types
rg "unwrap_or_else.*nominal" phalcom-semantic/src
rg "resolve_type_parameter" phalcom-semantic/src
rg "UnannotatedDeclaration" phalcom-semantic/src/types
```

Expected:

- no invalid-kind-to-`Type` recovery;
- no ignored record tail;
- no source lambda whole-body-as-free shortcut;
- no resolved-declaration nominal fabrication;
- no stale TypeId-only generic binding API;
- no type-formation failure classified as unannotated.

## Step 18.7 — Alias deletion gates

Search:

```bash
rg "Statement::TypeAlias" phalcom-modules phalcom-semantic
```

Expected:

- module interface collects it;
- semantic session publishes it;
- source index observes it;
- runtime compiler may still intentionally emit no opcode.

## Step 18.8 — Type-form value gate

Search the `Expr::TypeForm` arm and verify it calls type-form lowering, not proper-type annotation lowering.

## Step 18.9 — Publishability gate

Add/assert a helper that walks every published:

- `DeclarationTypeInfo`;
- `TypeAliasInfo`;
- `CallableSemanticSignature`;
- `GenericSignature`;

and verifies:

- all referenced `TypeId`s are store-owned;
- all type forms have valid canonical kinds;
- no solver-local IDs publish;
- no unresolved alias cycles publish;
- no `RecordRow` binder is encoded as `TypeData::Parameter`.

## Step 18.10 — Incremental/cold equivalence gate

Run the new differential tests.

The final semantic snapshots must agree structurally for identical final source, regardless of whether they were produced cold or incrementally.

---

# Task 19 — Documentation and final commit organization

Prefer reviewable commits rather than one large alias/type-system commit.

Recommended sequence:

```text
test(semantic): expose sc1 type-formation failure categories
refactor(semantic): add explicit type-formation outcomes
fix(semantic): reject invalid kind syntax without Type recovery
refactor(semantic): make type-level bindings kind-domain aware
fix(semantic): lower source type lambdas capture-safely
fix(semantic): publish generic signatures atomically
refactor(semantic): consolidate source declaration generic publication
fix(semantic): make Self lowering owner and side aware
fix(semantic): isolate class-side generic scope
fix(semantic): harden generic superclass publication
feat(modules): expose type aliases as module declarations
feat(semantic): publish transparent alias declaration shells
feat(semantic): add cycle-safe generic alias lowering
fix(semantic): support constructor-kinded type-form values
fix(semantic): stop erasing open record tails
refactor(semantic): remove type-formation fabrication fallbacks
test(semantic): enforce sc1 incremental and cold equivalence
docs(semantic): record sc1 semantic completion invariants
```

Small deviations are fine when one change cannot compile independently, but keep alias work separate from outcome/lambda work where possible.

---

# Final acceptance checklist

SC-1 is not complete until every checkbox below is true.

## Type formation

- [ ] `TypeFormationOutcome` distinguishes ready/dynamic/missing/unresolved/invalid/blocked/cancelled/budget/internal.
- [ ] Invalid source no longer maps to `UnannotatedDeclaration`.
- [ ] Missing semantic declaration products are never fabricated as nominal types.

## Kinds

- [ ] `KindSyntax::Invalid` never returns `KindId::TYPE`.
- [ ] Arrow kinds lower canonically.
- [ ] Partial application kinds are correct.

## Generic binders

- [ ] Owner/index binder identity preserved.
- [ ] `RecordRow` binder construction cannot panic.
- [ ] `RecordRow` binder is not `TypeData::Parameter`.
- [ ] Variance placement validated.
- [ ] Invalid constraints prevent valid signature publication.

## Type lambdas

- [ ] Source binders lower to `ScopedTypeData::Bound`.
- [ ] Alpha equivalence tested.
- [ ] Capture avoidance tested.
- [ ] Nested lambdas tested.
- [ ] Higher-kinded binder/application tested.
- [ ] Partial beta reduction tested.

## `Self`

- [ ] `Self` owner is explicit.
- [ ] Instance/class side is explicit.
- [ ] Outside-owner `Self` invalid.
- [ ] Inherited specialization tests pass.

## Declaration scope

- [ ] Instance members see declaration generics.
- [ ] Class-side members do not see ambient instance generics.
- [ ] Method-local generics still work.
- [ ] Generic superclass template remains canonical and proper.

## Aliases

- [ ] `Statement::TypeAlias` is a module declaration.
- [ ] Alias has stable semantic shell identity.
- [ ] Alias has no fake class object.
- [ ] Alias expands transparently.
- [ ] Generic alias uses scoped lowering.
- [ ] Alias cycles rejected.
- [ ] Alias dependencies invalidate consumers.
- [ ] Import/export works.
- [ ] Source index/navigation identity exists.

## Type-form values

- [ ] `Expr::TypeForm(Int)` works.
- [ ] `Expr::TypeForm(List)` works.
- [ ] partial constructor value works.
- [ ] type-lambda value works.
- [ ] runtime/value type is distinct from semantic denotation.
- [ ] no eager runtime descriptor allocation is introduced by semantic checking.

## Row handoff

- [ ] open record tail is never discarded.
- [ ] SC-1 does not claim row solver completion.
- [ ] row binder identity is ready for SC-3.

## Query/incremental

- [ ] alias/generic declaration dependencies are recorded.
- [ ] semantic fingerprints are structural.
- [ ] range-only edits do not invalidate semantic consumers unnecessarily.
- [ ] cold and incremental final semantics agree.

## Verification

- [ ] `cargo fmt --check` passes.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test -p phalcom-modules` passes.
- [ ] `cargo test -p phalcom-semantic` passes.
- [ ] affected broader workspace tests pass.
- [ ] search-based deletion gates pass.
- [ ] no placeholder/TODO implementation remains for any SC-1 acceptance item.

---

# Beginner implementation notes

1. **Do not “fix” compiler errors by adding wildcard matches.** When an enum gains a new outcome, handle every variant deliberately.
2. **Clone IDs before mutably borrowing `TypeStore`.** Many Rust borrow errors in this code will come from reading parameter data and then calling a mutating interner. Extract the IDs/kinds/names into local owned values first.
3. **Do not compare raw `TypeId` values from different stores.** Use stable structural export/fingerprint helpers.
4. **When a test says “not ready,” inspect both the diagnostic and the query/result category.** Absence of a diagnostic is not proof of semantic success.
5. **Use existing canonical helpers.** Do not hand-create `TypeData::Applied` or type-lambda substitution when `TypeStore::apply_type_form` / `TypeLambdaArena` already owns it.
6. **Keep source names out of semantic lambda equality.** Names belong to provenance.
7. **Do not make aliases nominal to simplify lookup.** That would violate transparent alias semantics.
8. **Do not implement SC-3 in the record-tail branch.** The correct SC-1 result is an explicit handoff, not a partial row solver.
9. **Do not implement generic getters while touching callable generic helpers.** Getter generic support is intentionally a separate completion plan.
10. **When current HEAD differs from the audited SHA, re-audit the touched symbol before applying a replacement snippet.** The semantic intent in this plan is normative; exact surrounding Rust may have moved.
