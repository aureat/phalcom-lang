# Phalcom ADT/GADT + Associated Lookup — Part 3 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Use `superpowers:test-driven-development` for semantic changes and `superpowers:verification-before-completion` before claiming completion.

**Goal:** Implement compiler-owned static `::` associated resolution, exact member and family values, generic/GADT specialization, family invocation, source identity, and machine-readable resolution products without introducing runtime lowering.

**Architecture:** `phalcom-semantic` consumes the Part 2 enum/associated declaration products, resolves a declaration-backed type-form owner, composes a static effective family, then either reifies an exact value/callable/family or selects an executable member and reuses the existing canonical call checker. A new structural family type is interned in `TypeStore`; captured associated-value denotation separately retains nominal identity, lookup-owner specialization, and access-filtered exact targets, while `AssociatedResolution` records expression-level resolution for Part 4. `phalcom-core` continues to reject associated AST lowering until Part 4.

**Tech Stack:** Rust, `phalcom-ast`, `phalcom-common`, `phalcom-semantic`, existing `TypeStore`/kind system, `InferenceSession`, semantic DB/snapshot/source-index infrastructure, Cargo test/fmt/clippy.

**Spec:** `docs/impl/adt-gadt-associated-lookup/part-3/03-associated-resolution-family-values-generic-specialization-invocation-typing-technical-spec.md`

## Global Constraints

- Start from the actual post-Part-2 working tree/commit; the planning repository baseline was `feat/adts` at `2c8b5840fc5a864968cb2a832540fbcba868d9f8`, while Part 2 implementation was uncommitted during planning.
- `phalcom-semantic` is the sole static semantic authority.
- `.` remains ordinary message dispatch; `::` never falls back to ordinary dispatch or `doesNotUnderstand`.
- `>>` is retained and not redesigned in Part 3.
- No monkey-patching or live provider/rebindable-capability semantics are introduced.
- Variant constructors remain `VariantConstructorId`, not behavioral `CallableId` methods.
- `@variant None`, `@variant None()`, and `@variant None(_)` remain distinct exact selectors in one family.
- Getter `#name` and Method `#name()` remain distinct in every type/resolution/application representation.
- Bare/partial generic declaration forms are valid associated owners even when their kind is an arrow kind.
- Unresolved generic parameters must not default to `Dynamic`, `Object`, or another erased type.
- **Decision Gate G1:** declaration-provided rank-1 polymorphism for *reified* bare associated values/families is not ratified. Preserve residual declared binders and their kinds, but do not encode either “always generalize” or “always reject as underconstrained” as language semantics until G1 is resolved. Direct invocation may still diagnose a genuinely unsolved result specialization.
- Part 3 does not implement runtime representation, bytecodes, VM execution, `match`, or exhaustiveness.
- Part 4 must consume `AssociatedResolution`; it must not semantically re-resolve associated AST.

---

# 0. Preflight: Reconcile the Actual Part 2 WIP Before Editing

**Files to inspect:**

```text
docs/impl/adt-gadt-associated-lookup/part-1/*
docs/impl/adt-gadt-associated-lookup/part-2/*
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/enum_semantics.rs           # expected from Part 2
phalcom-semantic/src/associated.rs               # expected from Part 2
phalcom-semantic/src/checker/enum_declaration.rs # expected from Part 2
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs
```

**Produces:** a recorded implementation baseline, a mechanical name map from this plan to the actual Part 2 code, and an explicit resolution of every language-design gate required by executable tasks.

- [ ] **Step 1: Record the starting tree.**

Run:

```bash
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Preserve all existing uncommitted Part 2 work. Do not reset, stash, or overwrite it.

- [ ] **Step 2: Confirm the Part 1 AST invariants.**

Run:

```bash
rg -n 'Statement::Enum|struct EnumDef|struct VariantDecl|AssociatedLookup|AssociatedInvoke|AssociatedNamedMode' phalcom-ast/src
```

Verify:

```text
@variant None     => payload: None
@variant None()   => payload: Some(parameters = [])
owner::name()     => AssociatedInvoke
owner::name::()   => AssociatedLookup exact Method selector
owner::name::*    => AssociatedLookup Family mode
```

- [ ] **Step 3: Locate the actual Part 2 products.**

Run:

```bash
rg -n 'VariantId|VariantConstructorId|AssociatedFamilyId|AssociatedFamilyInfo|AssociatedSurface|VariantConstructorSignature|CaseTypeEnvironment|ExactCase' phalcom-semantic/src
```

If the WIP uses structurally equivalent names/paths, write the mapping at the top of a local implementation note and mechanically substitute those names throughout this plan. Do **not** create a second `AssociatedFamilyTable`, enum semantic table, or variant identity system.

- [ ] **Step 4: Confirm Part 2 dependency/query ownership.**

Run:

```bash
rg -n 'EnumDeclaration|AssociatedSurface|EnumRequirements' phalcom-semantic/src/db phalcom-semantic/src/session.rs phalcom-semantic/src/snapshot.rs
```

Required precondition: enum declaration semantics and associated surfaces are compiler-owned immutable products or snapshot tables.

- [ ] **Step 5: Resolve Decision Gate G1 before implementing escaped bare-generic reification behavior.**

The technical spec deliberately leaves automatic first-class polymorphism for reified bare owners unratified. Record exactly one project decision before Task 13 or any equivalent reification task asserts behavior for residual declaration binders:

```text
G1-A Contextual-only v1:
    Option::Some::* may retain a residual template during analysis,
    but an escaping value with unsolved declaration binders is diagnosed
    as underconstrained unless an expected callable/family type specializes it.

G1-B Declaration-provided rank-1 scheme:
    reifying Option::Some::* preserves the declaration's universal binders
    (with their explicit kinds) and each later invocation instantiates them
    independently. This is declaration polymorphism only, not arbitrary HM
    let-generalization.
```

Regardless of G1:

```text
- never default unresolved binders to Dynamic/Object/Any;
- preserve binder kinds;
- direct invocation must solve every result-relevant owner parameter that the
  resulting value type needs, unless a separately ratified existential model exists;
- do not use first-use monomorphization of a stored family/callable binding.
```

Until G1 is recorded, Tasks 1–12 and the non-reification infrastructure of later tasks may proceed, but Task 13's residual-binder behavior and its corresponding end-to-end assertions are blocked.

- [ ] **Step 6: Run focused prerequisite tests using the actual integration-test targets.**

```bash
cargo test -p phalcom-ast --test family_selector_syntax
cargo test -p phalcom-ast --test enum_syntax
cargo test -p phalcom-semantic --test semantic semantic::foundations::kinds
cargo test -p phalcom-semantic --test semantic semantic::foundations::generics_core
```

Also run every Part 2 test module added by the WIP. Record any pre-existing failure before Part 3 changes.

- [ ] **Step 7: Record the canonical rest-lane baseline before Task 14A.**

Verify the current post-Part-2 tree still preserves full source rest modes in canonical declaration signatures and inspect whether application projections still collapse them:

```bash
rg -n 'RestMode|rest: bool|UnsupportedRestShape|bind_static_arguments|project_semantic_signature' \
  phalcom-semantic/src/checker \
  phalcom-semantic/src/dispatch.rs \
  phalcom-semantic/src/types
```

If Part 2 already repairs canonical rest binding, map Task 14A onto that implementation and keep only its regression tests. Otherwise Task 14A is mandatory before Task 15.

- [ ] **Step 8: Commit only if Part 2 is already intended to be committed.**

Do not bundle unfinished Part 2 changes into a Part 3 commit merely to clean the tree. If the user's workflow keeps Part 2 WIP uncommitted, continue on that tree and keep Part 3 diffs logically separable.

---

# 1. File/Module Structure to Establish

Part 3 should converge on these responsibilities.

```text
phalcom-semantic/src/types/family.rs
    canonical structural family type arena
    operation/member shapes
    family type interning helpers

phalcom-semantic/src/checker/associated.rs
    associated owner resolution
    effective static family resolution
    exact member resolution
    whole-family reification
    family application / member selection
    GADT owner compatibility
    AssociatedResolution construction

phalcom-semantic/src/checker/call.rs
    generalized InvocationTargetId application adapter
    existing argument/generic inference remains canonical

phalcom-semantic/src/types/denotation.rs
    captured exact member/family denotations and flow preservation

phalcom-semantic/src/checker/analysis.rs
    AssociatedResolution attachment and semantic dependencies

phalcom-semantic/src/checker/context.rs
    access to Part 2 enum/associated products
    reusable owner signature specialization
    dependency tracking

phalcom-semantic/src/source_index/*
    publish exact associated occurrence targets from formal resolution

phalcom-semantic/src/types/{store,relation,substitution,environment}.rs
    exhaustive Family type support

phalcom-semantic/src/presentation.rs
    deterministic family type display
```

Do not put the family type arena into the already-large expression checker unless repository conventions make a focused module impossible.

---
# 2. Task 1 — Add Canonical Structural Family Types

**Files:**

```text
Create: phalcom-semantic/src/types/family.rs
Modify: phalcom-semantic/src/types/mod.rs
Modify: phalcom-semantic/src/types/id.rs
Modify: phalcom-semantic/src/types/store.rs
Test:   phalcom-semantic/tests/semantic/foundations/type_model.rs
Test:   phalcom-semantic/tests/semantic/foundations/mod.rs
```

**Interfaces:**

- Consumes: `TypeId`, `SelectorKind`, `SelectorSlot`, existing `TypeData::Callable`.
- Produces: `FamilyTypeId`, `FamilyOperationShape`, `FamilyMemberTypeKind`, `FamilyMemberType`, `FamilyType`, `TypeData::Family(FamilyTypeId)`, and `TypeStore::family_type(...)`.

- [ ] **Step 1: Write failing canonicalization and kind tests.**

Add tests proving that member order does not affect canonical identity and base name is absent from structural type identity:

```rust
let unary = FamilyMemberType::callable(
    FamilyOperationShape::method([SelectorSlot::Positional]),
    unary_callable,
);
let zero = FamilyMemberType::callable(
    FamilyOperationShape::method([]),
    zero_callable,
);
let a = store.family_type([unary.clone(), zero.clone()]).unwrap();
let b = store.family_type([zero, unary]).unwrap();
assert_eq!(a, b);
assert_eq!(store.kind_of(a), KindId::TYPE);
```

Also test:

```text
- a Value member and Callable member with the same slot count but different
  selector/member kinds remain distinct;
- cloning the TypeStore preserves FamilyTypeId/TypeData::Family denotation;
- a structural family value is a proper value type (`KindId::TYPE`), never an
  arrow-kinded type constructor merely because its entries are callable.
```

- [ ] **Step 2: Run the new tests and confirm they fail because the family type API is absent.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::type_model -- --nocapture
```

- [ ] **Step 3: Add the compact family type ID and focused arena.**

Implement the spec shapes, including constructors such as:

```rust
impl FamilyOperationShape {
    pub fn new(kind: SelectorKind, slots: impl Into<Box<[SelectorSlot]>>) -> Self;
    pub fn method(slots: impl Into<Box<[SelectorSlot]>>) -> Self;
}

impl FamilyMemberType {
    pub fn value(operation: FamilyOperationShape, ty: TypeId) -> Self;
    pub fn callable(operation: FamilyOperationShape, callable_ty: TypeId) -> Self;
}
```

Validate that `Callable` members point at `TypeData::Callable` and reject malformed family entries through a small `FamilyTypeError`.

- [ ] **Step 4: Extend `TypeData`.**

Add:

```rust
TypeData::Family(FamilyTypeId)
```

and `TypeStore` fields/accessors for the family arena/interner. Intern every `TypeData::Family` with `KindId::TYPE`.

- [ ] **Step 5: Canonicalize member ordering and reject duplicate operation shapes.**

Sort by `FamilyOperationShape`; duplicate exact operation shape is an error unless the full member entry is identical, in which case deduplicate it.

- [ ] **Step 6: Run focused tests.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::type_model
```

- [ ] **Step 7: Commit.**

```bash
git add phalcom-semantic/src/types phalcom-semantic/tests/semantic/foundations/type_model.rs
git commit -m "feat(types): add canonical associated family types"
```

---

# 3. Task 2 — Make Every Canonical Type Consumer Handle `TypeData::Family`

**Files:**

```text
Modify: phalcom-semantic/src/types/relation.rs
Modify: phalcom-semantic/src/types/substitution.rs
Modify: phalcom-semantic/src/types/environment.rs
Modify: phalcom-semantic/src/checker/inference.rs
Modify: phalcom-semantic/src/presentation.rs
Modify: every exhaustive TypeData match found by search
Test:   phalcom-semantic/tests/semantic/foundations/substitution.rs
Test:   phalcom-semantic/tests/semantic/foundations/inference.rs
Test:   phalcom-semantic/tests/semantic/foundations/type_model.rs
```

**Interfaces:**

- Consumes: Task 1 family arena.
- Produces: structurally sound substitution/relation/presentation of family member types.

- [ ] **Step 1: Audit exhaustive matches before editing.**

```bash
rg -n 'TypeData::|match .*store\.get|match .*TypeData' phalcom-semantic/src
```

Keep a temporary checklist in the implementation notes; delete it after all arms compile.

- [ ] **Step 2: Add failing substitution tests.**

Construct a family whose callable member contains a declaration type parameter and prove `TypeSubstitution::apply` rewrites the member's callable type while retaining operation shape.

- [ ] **Step 3: Add failing relation tests.**

Directly intern:

```text
required = family { Method(_) -> Product }
provided = family { Method(_) -> SpecialProduct, Method() -> Product }
```

under a hierarchy where `SpecialProduct <: Product`, and assert `provided <: required`.

Also assert missing required operation fails and value/callable member-kind mismatch fails.

- [ ] **Step 4: Implement family substitution/materialization.**

For every member, recursively substitute/materialize its `ty`, then re-intern the family through `TypeStore::family_type`.

- [ ] **Step 5: Implement family structural relation.**

Add a helper such as:

```rust
fn check_family_subtype_impl(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    provided: FamilyTypeId,
    required: FamilyTypeId,
    ...
) -> RelationOutcome<()>;
```

For each required operation, find the same operation in `provided`. For `Callable`, delegate to normal callable subtype relation. For `Value`, delegate to ordinary type relation. Extra provided members are accepted.

- [ ] **Step 6: Extend inference term conversion only as much as required.**

If inference currently lowers every canonical structural type into an `InferenceTerm`, add a `Family` term that recursively carries member body terms. If family values are treated as rigid expected types in current inference, document and test the rigid path instead of creating unnecessary inference variables.

- [ ] **Step 7: Add deterministic presentation.**

Render operation shapes preserving `#name` versus `#name()` semantics conceptually, but because base name is denotation rather than structural type identity, generic structural presentation may use:

```text
family { getter: Int; method(): String; method(_): (Int) -> Foo }
```

Associated-expression hover can prepend the nominal family base separately later.

- [ ] **Step 8: Run focused and full type tests.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::substitution
cargo test -p phalcom-semantic --test semantic semantic::foundations::inference
cargo test -p phalcom-semantic --test semantic semantic::foundations::type_model
```

- [ ] **Step 9: Commit.**

```bash
git add phalcom-semantic/src/types phalcom-semantic/src/checker/inference.rs phalcom-semantic/src/presentation.rs phalcom-semantic/tests/semantic/foundations
git commit -m "feat(types): propagate associated family types"
```

---

# 4. Task 3 — Introduce Truthful Invocation and Associated Resolution Identities

**Files:**

```text
Modify: phalcom-semantic/src/identity.rs
Modify: phalcom-semantic/src/checker/analysis.rs
Create: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/mod.rs
Test:   phalcom-semantic/tests/semantic/foundations/identity_diagnostics.rs
```

**Interfaces:**

- Consumes: Part 2 `VariantConstructorId`, `AssociatedFamilyId`, `AssociatedMemberId`.
- Produces: `InvocationTargetId`, `AssociatedResolution`, `AssociatedResolutionKind`, `SpecializedAssociatedMember`, `FamilyApplicationResolution`, and body-local resolution attachments.

- [ ] **Step 1: Write identity tests.**

Assert:

```rust
InvocationTargetId::Behavioral(callable.clone())
    != InvocationTargetId::VariantConstructor(constructor.clone())
```

and ordering/hash remain deterministic.

- [ ] **Step 2: Add the invocation target enum.**

```rust
pub enum InvocationTargetId {
    Behavioral(CallableId),
    VariantConstructor(VariantConstructorId),
}
```

Add helpers:

```rust
pub fn declaration_owner(&self) -> &DeclarationId;
pub fn module(&self) -> &ModuleId;
```

by delegating through the contained identity.

- [ ] **Step 3: Add the associated resolution model in `checker/associated.rs`.**

Use the technical-spec shape, with a `SpecializedAssociatedMember` carrying at least:

```rust
pub struct SpecializedAssociatedMember {
    pub member: AssociatedMemberId,
    pub operation: FamilyOperationShape,
    pub value_type: TypeId,
    pub target: Option<InvocationTargetId>,
}
```

For callable entries, `value_type` is their exact callable type. For singleton entries, it is the exact case type and `target` is `None`.

- [ ] **Step 4: Attach associated and family-application resolution products to callable-body analysis.**

Use body-local indexes:

```rust
pub type AssociatedResolutionIndex = BTreeMap<ExpressionId, AssociatedResolution>;
pub type FamilyApplicationResolutionIndex = BTreeMap<ExpressionId, FamilyApplicationResolution>;
```

Add both to `CallableAnalysis` / checking context rather than bloating every `ExpressionAnalysis` with large enums. Define `FamilyApplicationResolution` / `FamilyApplicationSelection` with the technical-spec §16.1 shape. Do not repurpose `ExpressionAnalysis.call: Option<CallResolutionId>` unless the implementation simultaneously introduces a real general call-resolution arena; today that ID is only a scaffold and has no repository-wide product behind it.

- [ ] **Step 5: Add allocation/publication helpers to `CheckingContext`.**

```rust
pub(crate) fn publish_associated_resolution(
    &mut self,
    expression: ExpressionId,
    resolution: AssociatedResolution,
);

pub(crate) fn publish_family_application_resolution(
    &mut self,
    expression: ExpressionId,
    resolution: FamilyApplicationResolution,
);
```

- [ ] **Step 6: Run compile-focused tests.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::identity_diagnostics
cargo check -p phalcom-semantic
```

- [ ] **Step 7: Commit.**

```bash
git add phalcom-semantic/src/identity.rs phalcom-semantic/src/checker
git commit -m "feat(semantic): model associated invocation resolutions"
```

---

# 5. Task 4 — Preserve Captured Associated Value/Family Denotation Through Flow

**Files:**

```text
Modify: phalcom-semantic/src/types/denotation.rs
Modify: phalcom-semantic/src/associated.rs               # use the actual Part 2 associated semantic module after preflight mapping
Modify: phalcom-semantic/src/checker/typed_expr.rs
Modify: phalcom-semantic/src/checker/analysis.rs
Modify: phalcom-semantic/src/checker/binding.rs
Modify: phalcom-semantic/src/checker/flow/state.rs
Modify: every `SemanticDenotation` exhaustive match found by search
Test:   phalcom-semantic/tests/semantic/foundations/knowledge.rs
Test:   phalcom-semantic/tests/semantic/foundations/expression_analysis.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Why this task is stronger than `AssociatedFamilyId` denotation:** an exact/family value can be acquired on `Derived<Int>` while its defining behavior belongs to `Base`; a whole-family capture is also access-filtered at acquisition. Storing only `AssociatedMemberId` / `AssociatedFamilyId` would lose lookup-owner specialization and captured capability membership and would force later calls to re-resolve inheritance/visibility, violating the ratified static/capability model.

**Interfaces:**

- Produces range-free captured denotations that preserve the static associated value actually acquired.

- [ ] **Step 1: Write failing exact-member flow tests.**

For an inherited generic behavioral member:

```phalcom
const make = Derived<Int>::make::(_)
const x = make([1])
```

(or the equivalent `::(_)` spelling), prove the binding retains:

```text
lookup owner form = Derived<Int>
defining member   = Base/... CallableId
invocation target = Behavioral(Base/...)
```

and the later call does not re-run associated inheritance lookup.

Also prove an exact `Option<Int>::Some::(_)` binding retains `owner_form = Option<Int>` plus `VariantConstructorId` target.

- [ ] **Step 2: Write failing family-capability merge tests.**

Capture the same family under two access contexts where the accessible member sets differ. Assert their captured denotations are unequal even if they share the same nominal `AssociatedFamilyId`. Identical captures survive a flow merge; different captures drop denotation while preserving whatever structural type knowledge can soundly join.

- [ ] **Step 3: Introduce immutable captured associated denotation shapes.**

Use range-free shapes equivalent to:

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

If the implementation prefers a snapshot-local immutable capture handle rather than `Arc<[...]>`, the handle must resolve to exactly this semantic information and must remain snapshot-safe. Do not use a runtime/advisory object ID.

- [ ] **Step 4: Remove `Copy` from `SemanticDenotation` and update flow/binding code deliberately.**

Convert `.copied()` / direct moves to `.cloned()` where the existing TypeForm/Kind paths need it. `ValueSemanticFact::merge` keeps denotation only when the complete captured denotation is equal.

- [ ] **Step 5: Keep structural type and nominal/capture provenance separate.**

Do not infer a captured family denotation merely from `TypeData::Family`; unrelated or differently-authorized captures may share a structural family type. Do not re-resolve a nominal family to recover a denotation after a merge drops it.

- [ ] **Step 6: Add constructors/helpers for exact and family captures.**

Helpers should canonicalize captured family-member order by `FamilyOperationShape`/exact selector and should be used by Tasks 11–13 instead of hand-assembling denotation in expression code.

- [ ] **Step 7: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::knowledge
cargo test -p phalcom-semantic --test semantic semantic::foundations::expression_analysis
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/associated.rs phalcom-semantic/src/types/denotation.rs phalcom-semantic/src/checker phalcom-semantic/tests/semantic/foundations
git commit -m "feat(semantic): preserve captured associated value denotations"
```

---

# 6. Task 5 — Wire Part 2 Enum/Associated Products Into `CheckingContext`

**Files:**

```text
Modify: phalcom-semantic/src/checker/context.rs
Modify: phalcom-semantic/src/checker/analysis.rs
Modify: phalcom-semantic/src/session.rs
Modify: phalcom-semantic/src/snapshot.rs
Modify: phalcom-semantic/src/db/query.rs
Test:   phalcom-semantic/tests/semantic/incremental/fingerprints.rs
```

**Interfaces:**

- Consumes: Part 2 `EnumSemanticTable`, `AssociatedFamilyTable`/`AssociatedSurface` and query products.
- Produces: tracked read-only accessors from body checking plus `SemanticDependency::EnumDeclaration` / `AssociatedSurface`.

- [ ] **Step 1: Add dependency variants if Part 2 WIP has not already done so.**

```rust
pub enum SemanticDependency {
    ...
    EnumDeclaration(DeclarationId),
    AssociatedSurface(DeclarationId),
}
```

- [ ] **Step 2: Add tracked associated/enum product access to `CheckingContext`.**

Prefer borrowed immutable tables/products:

```rust
pub(crate) fn enum_info(&self, owner: &DeclarationId) -> Option<&EnumInfo>;
pub(crate) fn associated_surface(&self, owner: &DeclarationId) -> Option<&AssociatedSurface>;
```

Each accessor records the corresponding semantic dependency before returning a query-owned product.

- [ ] **Step 3: Do not clone the full associated table per callable body.**

Follow the existing borrowed-dispatch pattern or snapshot-table references.

- [ ] **Step 4: Add incremental tests.**

A callable consuming `Option::Some` must record the enum/associated dependencies once associated analysis is wired later; for this task, unit-test the tracker helpers directly if the expression path is not yet connected.

- [ ] **Step 5: Compile and commit.**

```bash
cargo check -p phalcom-semantic
cargo test -p phalcom-semantic --test semantic semantic::incremental::fingerprints
git add phalcom-semantic/src/checker phalcom-semantic/src/session.rs phalcom-semantic/src/snapshot.rs phalcom-semantic/src/db
git commit -m "feat(semantic): expose tracked associated declaration products"
```

---

# 7. Task 6 — Resolve Declaration-Backed Associated Owner Type Forms

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/expression.rs
Modify: phalcom-semantic/src/types/store.rs
Modify: phalcom-semantic/src/diagnostic.rs
Test:   Create phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
Modify: phalcom-semantic/tests/semantic/foundations/mod.rs
```

**Interfaces:**

- Consumes: `SemanticDenotation::TypeForm`, explicit kind/application machinery.
- Produces: `AssociatedOwnerResolution`.

- [ ] **Step 1: Create the focused test module and write failing owner tests.**

Include source cases for:

```phalcom
enum Option<T> { @variant None }
const a = Option::None
const b = Option<Int>::None
```

plus all owner-category cases required by the spec:

```phalcom
const O = Option
const viaAlias = O::None              // binding preserves TypeForm denotation

const x = Option<Int>::None
const bad = x::None                   // ordinary runtime value rejected
```

Also cover:

```text
- an unresolved/Unknown owner: blocked/diagnosed without dispatch fallback;
- a Dynamic owner: explicit dynamic/blocked associated-owner boundary, never runtime class search;
- a generic type parameter `T` used as `T::member`: rejected because it has no single declaration-backed associated surface in Part 3;
- a partially applied owner retains its residual arrow kind.
```

Use the repository semantic fixture API to inspect diagnostics/expression products rather than parser-only assertions.

- [ ] **Step 2: Give `Expr::TypeForm` canonical expression semantics.**

In `checker/expression.rs`, add an explicit `Expr::TypeForm(annotation)` arm. Resolve the annotation/type form through the existing canonical type-annotation resolver and kind checker. For a declaration-backed nominal/applied form, synthesize the nominal origin's class-object value knowledge and attach:

```rust
SemanticDenotation::TypeForm(resolved_form)
```

This is required for `Option<Int>::Some` and partial owners such as `Result<Int>::Ok`. Do not use the class-object value type to recover generic specialization later; the `TypeForm` denotation is authoritative. Add direct tests that the expression product for `Option<Int>` carries the applied form and that its residual kind is correct.

- [ ] **Step 3: Add a `TypeStore` helper that peels applied nominal forms without requiring proper kind.**

```rust
pub fn nominal_application_parts(&self, form: TypeId) -> Option<(&DeclarationId, &[TypeId])>;
```

If borrowing through nested arena data makes that signature awkward, return an owned small descriptor instead. Do not flatten type lambdas into declarations.

- [ ] **Step 4: Implement owner resolution.**

```rust
pub(crate) fn resolve_associated_owner(
    ctx: &mut CheckingContext<'_>,
    typed_owner: &TypedExpression,
    range: SourceRange,
) -> Result<AssociatedOwnerResolution, AssociatedResolutionError>;
```

Require `typed_owner.denotation == Some(TypeForm(form))`, then recover declaration + supplied args + `ctx.store.kind_of(form)`.

- [ ] **Step 5: Add specific diagnostics.**

Add `DiagnosticCode` variants corresponding to:

```text
associated.owner.not_type_form
associated.owner.not_declaration_backed
associated.owner.unresolved
```

Use the associated operator/member range rather than blaming a later call argument.

- [ ] **Step 6: Add temporary expression arms.**

In `analyze_expression_inner`, route `Expr::AssociatedLookup` / `AssociatedInvoke` to new functions even if they still return a controlled `Unknown` after owner resolution. This replaces the generic fallback and proves the owner path is active.

- [ ] **Step 7: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
cargo check -p phalcom-semantic
git add phalcom-semantic/src phalcom-semantic/tests/semantic/foundations
git commit -m "feat(semantic): resolve associated type-form owners"
```

---

# 8. Task 7 — Build Static Effective Behavioral Families

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/context.rs
Modify: phalcom-semantic/src/associated.rs          # only when this is the actual Part 2 publication module
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: Part 2 direct associated surface, `TypeHierarchy`, canonical class-side callable signatures, Part 2 behavioral-family reservation/visibility facts.
- Produces: `EffectiveAssociatedFamily` with `lookup_owner` + exact defining `AssociatedMemberId`s.

- [ ] **Step 1: Write inheritance tests first, including the no-local-family case.**

Use:

```phalcom
class Base {
    @class build() -> Int { 0 }
    @class build(_ x: Int) -> Int { x }
}
class Derived is Base {
    @class build(_ x: Int) -> Int { x }
    @class build(config cfg: Config) -> Int { 1 }
}
class PureChild is Base {}
```

Assert:

```text
Derived::build::*
    #build()       defining owner Base
    #build(_)      defining owner Derived
    #build(config) defining owner Derived

PureChild::build::*
    #build()       defining owner Base
    #build(_)      defining owner Base
```

The second assertion is mandatory: an inherited behavioral family is resolvable even when the lookup owner publishes no local family of that base.

- [ ] **Step 2: Add inheritance-eligibility tests.**

Cover the exact Part 2 rules for private/non-inherited ancestor behavior. A member/family that Part 2 says does not reserve/inherit must not enter the effective family merely because an implementation surface happens to contain it.

- [ ] **Step 3: Add a negative test for the class-object behavior tail.**

Choose a method present on core `Class` instance behavior but absent from the declared source hierarchy and assert `Derived::thatName::*` does not become visible solely through class-object dispatch.

- [ ] **Step 4: Implement the source-hierarchy walk.**

```rust
pub(crate) fn resolve_effective_associated_family(
    ctx: &mut CheckingContext<'_>,
    owner: &AssociatedOwnerResolution,
    base: &SelectorBase,
) -> Result<EffectiveAssociatedFamily, AssociatedResolutionError>;
```

Normative algorithm:

```text
1. Read the lookup owner's direct associated family for `base`, if any.
2. If that direct family is Variant:
       return that direct variant family only.
   Part 2 reservation rules must already prevent an inherited behavioral
   category collision at the same effective base.
3. Otherwise walk the declared source hierarchy starting at lookup owner and
   continuing through each superclass, whether or not the lookup owner has a
   local Behavioral family.
4. At each owner, read only behavior that is eligible to participate in the
   inherited/effective associated surface according to Part 2 visibility and
   inheritance rules.
5. For each exact selector, keep the nearest eligible declaration; descendants
   may add other selectors to the same family.
6. Stop when the declared source hierarchy ends. Never append the ordinary
   class-object dispatch tail (`Class` instance behavior).
7. If no eligible behavioral contribution and no direct family exists, return
   MissingFamily.
```

The resulting effective family identity is `AssociatedFamilyId { owner: lookup_owner.declaration, base }`, even when every behavioral member is inherited. Exact member IDs retain their defining owners. This gives `PureChild::build::*` a stable family identity owned by `PureChild` without copying/reclassifying the inherited callables.

Record hierarchy and associated-surface dependencies through tracked context accessors.

- [ ] **Step 5: Treat mixed family-category state as an internal invariant failure.**

Do not “try method then variant.” If Part 2 publications contradict their reservation invariant, surface an internal semantic incident rather than inventing precedence.

- [ ] **Step 6: Canonicalize the member list by exact `Selector`.**

Nearest exact selector wins before sorting; the final stored list is deterministic.

- [ ] **Step 7: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker/associated.rs phalcom-semantic/src/checker/context.rs phalcom-semantic/src/associated.rs phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): compose static effective associated families"
```

---

# 9. Task 8 — Project Effective Members Into Specialized Member Templates

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/context.rs
Modify: phalcom-semantic/src/types/substitution.rs
Modify: phalcom-semantic/src/enum_semantics.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
Test:   phalcom-semantic/tests/semantic/foundations/generics_core.rs
```

**Interfaces:**

- Consumes: `AssociatedMemberId`, behavioral signatures, `VariantInfo`, `VariantConstructorSignature`, lookup-owner type form, generic supertype templates.
- Produces: `SpecializedAssociatedMember` templates before call-level inference.

- [ ] **Step 1: Write projection tests for all member categories.**

Cover:

```text
Behavioral getter           -> Callable member, Getter operation
Behavioral method #foo()    -> Callable member, Method operation
Behavioral setter           -> Callable member, Setter operation
Variant singleton           -> Value member, Getter operation
Variant zeroarg constructor -> Callable member, Method operation
Variant payload constructor -> Callable member, Method operation
```

- [ ] **Step 2: Write generic inherited-behavior tests before extracting helpers.**

At minimum:

```phalcom
class Base<T> {
    @class
    make(_ value: T) -> T { value }
}

class Derived<U> is Base<List<U>> {}
```

For `Derived<Int>::make::(_)`, assert the effective member still targets the defining `Base` `CallableId`, but its parameter/return contract is specialized to `List<Int>`.

Add a two-hop case as well, for example:

```text
Leaf<V> -> Mid<Map<V>> -> Base<List<Map<V>>>
```

and prove the defining-owner substitution is composed through every generic superclass template.

- [ ] **Step 3: Split defining-owner generic specialization from lookup-owner `Self` specialization.**

The existing `CheckingContext::specialize_dispatch_signature(receiver, ...)` is not sufficient by itself for inherited generic members: a signature defined by `Base<T>` must first receive a substitution for the **defining owner form**, while `Self` still refers to the **lookup owner form**.

Introduce focused helpers equivalent to:

```rust
pub(crate) fn project_owner_form_to_ancestor(
    &mut self,
    lookup_owner_form: TypeId,
    defining_owner: &DeclarationId,
) -> Result<TypeId, OwnerProjectionError>;

pub(crate) fn specialize_associated_behavior_signature(
    &mut self,
    lookup_owner_form: TypeId,
    defining_owner_form: TypeId,
    signature: CallableSignature,
) -> CallableSignature;
```

Required order:

```text
lookup form Derived<Int>
    -> project through GenericSupertypeTemplate(s)
    -> defining form Base<List<Int>>
    -> substitute Base's declaration binders in parameters/return
    -> specialize owner-relative Self using original lookup form Derived<Int>
```

Do not bind `Base<T>` parameters directly from `Derived<Int>` by positional index; the superclass template is semantic evidence and must be applied.

- [ ] **Step 4: Add behavioral member projection.**

Fetch the canonical defining signature from compiler-owned signature products/surfaces. Apply the defining-owner substitution and then lookup-owner `Self` specialization. Do not call ordinary dispatch to rediscover the target.

- [ ] **Step 5: Add variant member projection.**

For singleton variant, start from `VariantInfo.result_type_template` / exact-case template. For constructor variants, adapt the Part 2 constructor signature into an application signature while retaining `VariantConstructorId` as identity.

- [ ] **Step 6: Do not fully solve residual generic parameters yet.**

This task creates templates. Task 10 composes owner, GADT, expected-type, and call-argument constraints.

- [ ] **Step 7: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
cargo test -p phalcom-semantic --test semantic semantic::foundations::generics_core
git add phalcom-semantic/src/checker phalcom-semantic/src/types/substitution.rs phalcom-semantic/src/enum_semantics.rs phalcom-semantic/tests/semantic/foundations
git commit -m "feat(semantic): specialize associated member templates"
```

---

# 10. Task 9 — Intern Structural Family Types From Specialized Members

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/types/family.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: `SpecializedAssociatedMember`.
- Produces: canonical `TypeData::Family` for an effective family view.

- [ ] **Step 1: Write a failing `None` family test.**

For:

```phalcom
enum Option<T> {
    @variant None
    @variant None()
    @variant None(_ value: T)
}
```

under explicit `Option<Int>`, assert family type contains three operations:

```text
Getter []  + Value exact-case #None
Method []  + Callable () -> exact-case #None()
Method [_] + Callable (Int) -> exact-case #None(_)
```

- [ ] **Step 2: Write a base-name independence test.**

Create two unrelated families with identical operation/member types and assert the structural family `TypeId` is equal while their `AssociatedFamilyId` denotations differ.

- [ ] **Step 3: Implement `family_type_for_members`.**

```rust
pub(crate) fn family_type_for_members(
    store: &mut TypeStore,
    members: &[SpecializedAssociatedMember],
) -> Result<TypeId, FamilyTypeError>;
```

Map exact selector to `FamilyOperationShape { kind, slots }`; never include selector base in the structural type.

- [ ] **Step 4: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker/associated.rs phalcom-semantic/src/types/family.rs phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): type associated family views"
```

---
# 11. Task 10 — Implement Associated Generic Instantiation and Owner/GADT Constraint Composition

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/inference.rs
Modify: phalcom-semantic/src/types/substitution.rs
Modify: phalcom-semantic/src/diagnostic.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
Test:   phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs
```

**Interfaces:**

- Consumes: owner supplied args, Part 2 generic signature/case environment, `ExpectedType`, `ApplicationArgument`s.
- Produces: a solved or residual associated substitution plus explicit conflict/blocked/dynamic outcomes. Residual binders retain their explicit `KindId`s and are not themselves an error until the operation being performed requires concrete instantiation.

- [ ] **Step 1: Write failing generic invocation tests.**

Required cases:

```phalcom
const a = Option::Some(1)
const b: Option<Int> = Option::None
const c = Option<Int>::Some(1)
```

Assert precise `Option<Int>` exact cases and no Dynamic evidence.

- [ ] **Step 2: Write a partial-owner test.**

For `Result<Int>` verify the first declaration parameter is seeded as fixed and the second remains residual until context/call evidence provides it.

- [ ] **Step 3: Write a direct-invocation unsolved-result test, distinct from G1.**

For a constructor such as `Result<T,E>::Ok(T)`, an invocation `Result::Ok(1)` with no expected result leaves `E` result-relevant but unsolved. Unless the project separately ratifies existential result types, the invocation must be diagnosed/blocked as underconstrained; it must never choose `Dynamic`, `Object`, or a first-use specialization.

Do **not** use this test to decide the G1 behavior of a reified declaration value such as `const ok = Result::Ok::(_)` or `Result::Ok::*`.

- [ ] **Step 4: Implement an associated instantiation session.**

Do not duplicate the solver. Seed the existing `InferenceSession` with declaration parameter terms and constraints derived from:

```text
owner supplied arguments
owner declaration generic constraints
variant CaseTypeEnvironment equalities
expected result
call arguments (when invocation is being analyzed)
```

Use explicit `ConstraintOrigin` variants for owner/GADT constraints if existing origins cannot identify them.

- [ ] **Step 5: Materialize a structured outcome.**

Use existing inference/substitution APIs and distinguish at least:

```text
Solved { substitution }
Residual { substitution, binders: [(TypeParameterId, KindId), ...] }
Conflict
Blocked
DynamicBoundary
```

`Residual` is semantic information, not automatically `Unknown` and not automatically a quantified value type.

- [ ] **Step 6: Add dedicated diagnostics for concrete-operation failures.**

At minimum:

```text
associated.generic.underconstrained   # only where concrete instantiation is required
associated.generic.owner_conflict
associated.gadt.owner_conflict
```

- [ ] **Step 7: Preserve G1 neutrality.**

The instantiation layer must expose residual declared binders to Task 13. It must not itself decide whether an escaping reified associated value becomes a declaration-provided rank-1 scheme or is rejected without contextual specialization.

- [ ] **Step 8: Run generic proof-integrity tests.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
cargo test -p phalcom-semantic --test semantic semantic::foundations::generic_inference_proof_integrity
```

- [ ] **Step 9: Commit.**

```bash
git add phalcom-semantic/src/checker phalcom-semantic/src/types phalcom-semantic/src/diagnostic.rs phalcom-semantic/tests/semantic/foundations
git commit -m "feat(semantic): infer associated owner specializations"
```

---

# 12. Task 11 — Analyze Exact Variant Singleton Lookup

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/expression.rs
Modify: phalcom-semantic/src/types/denotation.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: exact Getter selector, `VariantInfo` singleton shape, Task 10 specialization.
- Produces: exact-case `TypedExpression`, variant denotation, `AssociatedResolutionKind::ExactValue`.

- [ ] **Step 1: Write failing singleton tests.**

Cover:

```phalcom
const a: Option<Int> = Option::None
const b = ConcreteEnum::None
```

Assert exact-case type, exact captured variant denotation (owner form + `VariantId`), ready status, and associated resolution attachment.

- [ ] **Step 2: Write negative tests.**

`Option::None` when only `@variant None()` exists must report exact getter missing rather than returning the family or constructor.

- [ ] **Step 3: Implement exact Getter member lookup.**

Resolve the family first, find exact selector `Selector::getter(base)`, then inspect `AssociatedMemberId`.

For singleton variant:

```rust
TypedExpression::new(
    TypeKnowledge::established(specialized_exact_case, EvidenceOrigin::ConstructorSemantics)
)
.with_denotation(SemanticDenotation::AssociatedValue(
    AssociatedValueDenotation::Exact {
        owner_form: owner.form,
        lookup_owner: owner.declaration.clone(),
        member: member.clone(),
        target: None,
    }
))
```

Use the project's preferred evidence origin if Part 2 introduced a variant-specific origin; do not claim runtime observation.

- [ ] **Step 4: Enforce construction visibility.**

Singleton acquisition is production of the case value and consumes the Part 2 construction visibility axis.

- [ ] **Step 5: Publish resolution and run tests.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
```

- [ ] **Step 6: Commit.**

```bash
git add phalcom-semantic/src/checker phalcom-semantic/src/types/denotation.rs phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): resolve exact variant singleton values"
```

---

# 13. Task 12 — Analyze Exact Behavioral and Variant Constructor References

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/expression.rs
Modify: phalcom-semantic/src/checker/typed_expr.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Produces exact callable `TypeData::Callable`, associated member denotation, and `ExactCallable` resolution for behavioral/constructor members.

- [ ] **Step 1: Write exact-ref tests.**

Cover every Part 1 exact associated lookup shape, not only named methods:

```phalcom
Option<Int>::Some::(_)
Option<Int>::None::()
System::print::(_)
Math::pi
Owner::name::                 // explicit getter alias; same target as Owner::name
Owner::name::=(put)           // setter
Owner::+(_)                   // operator exact reference
Owner::[_]                    // subscript-get exact reference
Owner::[_]=(put)              // subscript-set exact reference
```

Use fixture declarations that actually publish those behavioral selectors. Assert:

```text
variant constructor refs -> InvocationTargetId::VariantConstructor
behavioral refs          -> InvocationTargetId::Behavioral
Math::pi                 -> behavioral getter callable, not Float value
```

- [ ] **Step 2: Write getter-vs-zeroarg tests.**

Given behavioral `#value` and `#value()`, assert:

```phalcom
Owner::value      # getter target
Owner::value::()  # zeroarg Method target
```

remain distinct callable types and target IDs.

Also assert `Owner::value::` and `Owner::value` normalize to the same exact getter selector/target while preserving only source-spelling differences in ranges.

- [ ] **Step 3: Implement exact selector lookup for all `AssociatedMemberSyntax` modes.**

Normalize Part 1 syntax to canonical `Selector` once and search the effective family by exact selector.

- [ ] **Step 4: Intern callable types from specialized signatures.**

Use a helper that turns `CallableSignature` / constructor adapter into canonical `CallableType` while preserving labels/rest markers.

- [ ] **Step 5: Enforce visibility at acquisition.**

Behavioral exact refs use member visibility; variant constructor refs use construction visibility.

- [ ] **Step 6: Publish the captured exact-member denotation/resolution.**

The denotation must include `owner_form`/`lookup_owner`, exact `AssociatedMemberId`, and the `InvocationTargetId`; an inherited behavioral `CallableId` alone is not sufficient to reconstruct the bound static owner later.

- [ ] **Step 7: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): reify exact associated callable members"
```

---

# 14. Task 13 — Reify Whole `::*` Family Values

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/expression.rs
Modify: phalcom-semantic/src/diagnostic.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: effective family + specialized members + visibility + family type interning.
- Produces: family `TypedExpression`, family denotation, `AssociatedResolutionKind::Family`.

- [ ] **Step 1: Write the mixed variant-family test.**

For `None`, `None()`, `None(_)`, assert `Option<Int>::None::*` contains all three accessible entries and a canonical Family type.

- [ ] **Step 2: Write behavioral heterogeneous-family tests.**

Create a behavioral family containing an exact getter, zero-argument method, and setter with the same base. Assert the reified family preserves all three operation kinds distinctly. Also add a getter-only family and prove it reifies successfully even though later Method-kind invocation may fail.

- [ ] **Step 3: Write visibility-filtering tests.**

A family with one public and one private behavioral shape reified externally contains only the public member. Internal acquisition includes both.

- [ ] **Step 4: Implement `reify_associated_family`.**

```rust
fn reify_associated_family(
    ctx: &mut CheckingContext<'_>,
    owner: &AssociatedOwnerResolution,
    family: EffectiveAssociatedFamily,
    expected: &ExpectedType,
    range: SourceRange,
) -> TypedExpression;
```

Specialize as far as owner/expected context permits, filter access, intern family type, and attach an `AssociatedValueDenotation::Family` containing the lookup owner form plus the exact access-filtered captured member bindings in canonical order. Attach the full `AssociatedResolution` separately for the expression.

- [ ] **Step 5: Handle residual declaration binders according to Decision Gate G1.**

Common behavior regardless of G1:

```text
- retain residual TypeParameterId + KindId information in the associated template/resolution;
- never erase residual binders to Dynamic/Object/Any;
- never specialize a stored family by its first invocation.
```

If G1-A was ratified, diagnose an escaping family whose residual binders cannot be specialized by expected context. If G1-B was ratified, construct/use the declaration-provided rank-1 scheme representation selected by that decision and instantiate it independently per later call. Do not invent a third policy inside this task.

- [ ] **Step 6: Add `associated.family.inaccessible` diagnostic for zero accessible members.**

- [ ] **Step 7: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker phalcom-semantic/src/diagnostic.rs phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): reify static associated family values"
```

---

# 15. Task 14 — Generalize the Canonical Call Application Target for Variant Constructors

**Files:**

```text
Modify: phalcom-semantic/src/checker/call.rs
Modify: phalcom-semantic/src/checker/analysis.rs
Modify: phalcom-semantic/src/checker/context.rs
Modify: phalcom-semantic/src/checker/typed_expr.rs
Modify: phalcom-semantic/src/explain/node.rs
Test:   phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs
```

**Interfaces:**

- Consumes: `InvocationTargetId`.
- Produces: one call checker usable by behavioral callables and variant constructors without pretending constructors are methods.

- [ ] **Step 1: Add a direct unit test that applies a constructor-backed target through `apply_resolved_callable`.**

Construct a signature `(Int) -> ExactCase` and target identity `VariantConstructorId`; assert normal parameter checking and exact return publication work.

- [ ] **Step 2: Change only the internal application target identity.**

Replace the internal `CallableApplicationTarget` identity slot:

```rust
pub callable: Option<CallableId>
```

with:

```rust
pub target: Option<InvocationTargetId>
```

or an equivalently named field.

Keep compatibility constructors:

```rust
pub(crate) fn exact_behavioral(callable: CallableId, signature: CallableSignature) -> Self;
pub(crate) fn exact_variant_constructor(constructor: VariantConstructorId, signature: CallableSignature) -> Self;
```

Add a helper equivalent to:

```rust
impl InvocationTargetId {
    pub fn behavioral(&self) -> Option<&CallableId>;
}
```

- [ ] **Step 3: Preserve ordinary message-send behavior and compatibility fields.**

`from_dispatch` wraps the existing `CallableId` in `InvocationTargetId::Behavioral`.

Do **not** mechanically widen every existing `TypedExpression.callable`, `ExpressionAnalysis.callable`, `CallCheckResult.callable`, source-index callable attachment, or legacy diagnostic field to `InvocationTargetId`. Those fields currently mean a behavioral callable. Keep them behavioral-only and populate them with `target.behavioral().cloned()`.

Variant constructor identity is carried truthfully by `AssociatedResolution` and the generalized application/explanation target, not by lying in `Option<CallableId>` compatibility fields.

- [ ] **Step 4: Adapt shape diagnostics and explanation metadata.**

Where existing call-shape guidance requires a `CallableId`, emit `UseCallableShape` only for `InvocationTargetId::Behavioral`; add constructor-specific/generalized evidence only where the diagnostic model needs it. Explanation steps that genuinely mean “behavioral method” keep `CallableId`; associated/constructor selection uses a generalized target-bearing explanation node in `explain/node.rs`.

- [ ] **Step 5: Keep existing generic inference/application semantics unchanged apart from target identity plumbing.**

This is a refactor, not a second call path.

- [ ] **Step 6: Run all canonical call tests.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::canonical_call_application
cargo test -p phalcom-semantic --test semantic semantic::foundations::bidirectional_calls
```

- [ ] **Step 7: Commit.**

```bash
git add phalcom-semantic/src/checker phalcom-semantic/src/explain/node.rs phalcom-semantic/tests/semantic/foundations
git commit -m "refactor(semantic): generalize executable call targets"
```

---

# 15A. Task 14A — Preserve Rest Lanes and Make Canonical Static Call Binding Rest-Aware

**Why this task is mandatory:** the verified `feat/adts` call checker currently rejects every callable containing a rest parameter in `bind_static_arguments`, and `project_semantic_signature` collapses lane-aware `RestMode` into `rest: bool`. Part 3 cannot truthfully implement “exact selector before compatible rest-family member” until canonical call application can model the same static rest lanes soundly.

**Files:**

```text
Modify: phalcom-semantic/src/dispatch.rs
Modify: phalcom-semantic/src/signature.rs                 # only to map/preserve the existing canonical rest lane as needed
Modify: phalcom-semantic/src/checker/declaration_signature.rs
Modify: phalcom-semantic/src/checker/call.rs
Modify: phalcom-semantic/src/types/store.rs               # CallableParameterType projection
Modify: phalcom-semantic/src/types/relation.rs            # callable structural compatibility if rest representation changes
Modify: phalcom-semantic/src/types/substitution.rs        # copy/preserve the lane while recursively substituting callable parameter types
Test:   phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs
Test:   phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs
Test:   phalcom-semantic/tests/semantic/foundations/type_model.rs
```

**Interfaces:**

- Consumes: existing source/canonical `RestMode::{None, Positional, Labeled, Complete}` and `ApplicationArgument` lanes.
- Produces: lane-preserving callable projection plus a static argument binder that can prove statically known rest calls.

- [ ] **Step 1: Verify and freeze the current red behavior.**

Add tests demonstrating that a declaration signature already distinguishes positional/labeled/complete rest lanes but the projected call binder currently reports `UnsupportedRestShape`. These tests establish why the bool projection is insufficient.

- [ ] **Step 2: Define one semantic rest-lane representation for projected callable signatures/types.**

Do not keep `rest: bool` if it loses source-semantic distinctions. Reuse the canonical declaration rest lane if that type is appropriate outside AST-owned syntax; otherwise introduce a small semantic enum equivalent to:

```rust
pub enum CallableRestLane {
    None,
    Positional,
    Labeled,
    Complete,
}
```

Map `ParameterDef::rest_mode` / `CallableParameterSemantic.rest` into it exactly once at the source-to-semantic projection boundary.

- [ ] **Step 3: Preserve rest lane in `CallableParameter` and structural `CallableParameterType`.**

Update callable canonicalization, equality/hash, presentation if exposed, substitution, and callable subtype checks so different rest lanes are never conflated.

- [ ] **Step 4: Replace the unconditional `UnsupportedRestShape` path with a sound static binder.**

For a statically known argument shape:

```text
ordinary parameters bind first according to positional/labeled rules;
positional rest consumes remaining positional arguments only;
labeled rest consumes remaining static labeled arguments only;
complete rest consumes both remaining lanes;
no argument may bind twice;
required non-rest parameters must still be satisfied;
duplicate static labels remain an error.
```

`PackItem::Expand` and computed labels remain `StaticCallShape::Dynamic`; this task does not guess their runtime cardinality.

- [ ] **Step 5: Add relation/regression tests.**

Assert:

```text
- positional/labeled/complete rest shapes remain distinct structural callable types;
- existing non-rest callable variance behavior is unchanged;
- ordinary method calls using statically known rest arguments now type-check through the same binder;
- dynamic pack calls remain on the existing explicit dynamic boundary.
```

- [ ] **Step 6: Run the canonical call/type suites.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::canonical_call_application
cargo test -p phalcom-semantic --test semantic semantic::foundations::bidirectional_calls
cargo test -p phalcom-semantic --test semantic semantic::foundations::type_model
```

- [ ] **Step 7: Commit.**

```bash
git add phalcom-semantic/src/dispatch.rs phalcom-semantic/src/signature.rs phalcom-semantic/src/checker/declaration_signature.rs phalcom-semantic/src/checker/call.rs phalcom-semantic/src/types phalcom-semantic/tests/semantic/foundations
git commit -m "refactor(semantic): preserve callable rest lanes"
```

---

# 16. Task 15 — Implement Static Family Member Selection

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/call.rs
Modify: phalcom-semantic/src/diagnostic.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: effective/specialized family, `ApplicationArgument`, `StaticCallShape`, Task 14A lane-aware rest signatures/binder.
- Produces: selected `SpecializedAssociatedMember` or a structured family-selection failure.

- [ ] **Step 1: Write selection tests independent of expression synthesis.**

Build an in-memory family containing:

```text
Getter []
Method []
Method [_]
Method [_, reason]
```

Assert:

```text
Method [] call selects only Method []
Method [_] call selects only Method [_]
Getter [] is never a Method call candidate
a labeled call shape selects the exact label sequence, not a same-arity positional shape
exact Method shape wins before a compatible positional/labeled/complete rest entry
```

- [ ] **Step 2: Define structured selection result.**

```rust
pub enum AssociatedMemberSelection {
    Exact(SpecializedAssociatedMember),
    Rest(SpecializedAssociatedMember),
    Dynamic(Box<[SpecializedAssociatedMember]>),
}

pub enum AssociatedSelectionFailure {
    MissingShape { requested: FamilyOperationShape, available: Box<[FamilyOperationShape]> },
    InaccessibleExact { member: AssociatedMemberId },
    Ambiguous { candidates: Box<[AssociatedMemberId]> },
}
```

Use actual project error/result conventions if they already exist.

- [ ] **Step 3: Implement exact Method-shape selection.**

Build `FamilyOperationShape { kind: SelectorKind::Method, slots }` from `static_call_shape` and search exact members first.

- [ ] **Step 4: Integrate rest candidate applicability only after exact miss.**

Use the Task 14A canonical semantic rest-lane/binding helper. Exact operation lookup occurs first; only after an exact miss may a statically compatible rest member be considered. Do not copy VM dispatch or legacy `MethodFamily` routing rules into associated semantics.

- [ ] **Step 5: Enforce visibility before rest fallback semantics.**

If an exact shape exists in the semantic family but is inaccessible, return `InaccessibleExact`; do not continue to rest.

- [ ] **Step 6: Add diagnostics.**

```text
associated.call.shape_missing
associated.call.ambiguous
associated.member.inaccessible
```

Include available selectors in diagnostic context without flooding the primary message.

- [ ] **Step 7: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker phalcom-semantic/src/diagnostic.rs phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): select static associated family members"
```

---

# 17. Task 16 — Implement `AssociatedInvokeExpr` End to End

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/expression.rs
Modify: phalcom-semantic/src/checker/call.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
Test:   phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs
```

**Interfaces:**

- Consumes: `AssociatedInvokeExpr`, owner resolver, family resolver, generic instantiation, member selector, canonical call engine.
- Produces: precise `TypedExpression` + `StaticInvoke`/`DynamicInvoke` resolution.

- [ ] **Step 1: Write direct invocation tests.**

Required source cases:

```phalcom
Option::Some(1)
Option<Int>::Some(1)
Option::None()
Response::Error("failed")
```

Assert selected exact target and precise result type.

- [ ] **Step 2: Write the singleton/zeroarg distinction test.**

When a family contains both `#None` and `#None()`, assert `Option::None()` selects the constructor. When only singleton exists, assert `Option::None()` reports missing Method `#None()`.

- [ ] **Step 3: Implement `analyze_associated_invoke`.**

Pseudocode:

```rust
let owner_typed = analyze_expression(ctx, &invoke.receiver, &ExpectedType::None);
let owner = resolve_associated_owner(ctx, &owner_typed, invoke.range)?;
let family = resolve_effective_associated_family(ctx, &owner, &named_base)?;
let args = application_arguments(&invoke.args);
let specialization = prepare_associated_instantiation(ctx, &owner, &family, &args, expected)?;
let members = specialize_family_members(ctx, &family, &specialization)?;
let selection = select_family_member(ctx, &members, &args)?;
let target = application_target_for_selected_member(...)?;
let result = apply_resolved_callable(ctx, &target, &premise, &args, expected, invoke.range);
publish_associated_resolution(...);
return result.into();
```

Use project error propagation patterns and ensure all arguments are still analyzed on failure through `analyze_unresolved_application`-equivalent behavior.

- [ ] **Step 4: Preserve causal invalidity/status.**

An owner resolution failure must not suppress diagnostics from argument expressions; an invalid argument must not cause a fabricated result target.

- [ ] **Step 5: Publish selected source target and associated resolution.**

Do not set final value denotation to the constructor; the resolution carries invocation target provenance.

- [ ] **Step 6: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
cargo test -p phalcom-semantic --test semantic semantic::foundations::bidirectional_calls
git add phalcom-semantic/src/checker phalcom-semantic/tests/semantic/foundations
git commit -m "feat(semantic): type direct associated family invocation"
```

---

# 18. Task 17 — Invoke Reified `TypeData::Family` Values Through the Same Engine

**Files:**

```text
Modify: phalcom-semantic/src/checker/call.rs
Modify: phalcom-semantic/src/checker/expression.rs
Modify: phalcom-semantic/src/checker/associated.rs
Test:   phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: `TypedExpression` with `TypeData::Family` and associated family denotation/resolution.
- Produces: same member-selection/application semantics as direct associated invocation plus a persistent `FamilyApplicationResolution` for Part 4.

- [ ] **Step 1: Write reified family call tests.**

```phalcom
const make = Option<Int>::Some::*
const x = make(1)
```

Assert result equals direct `Option<Int>::Some(1)` exact case and selected target is the same `VariantConstructorId`.

- [ ] **Step 2: Write getter-not-callable-through-family test.**

A getter-only family `f` must reject `f()` with `associated.call.shape_missing`, even if exact getter reification `(Owner::getter)()` is valid.

- [ ] **Step 3: Add a `family_value_target`/application entry point.**

Do not convert the family to one `CallableSignature`. Route its member set to `select_family_member`, then adapt the selected member to `CallableApplicationTarget`.

- [ ] **Step 4: Consume captured family denotation when present; never reconstruct it.**

For a family flowing through a binding, `AssociatedValueDenotation::Family` supplies the frozen lookup-owner specialization and exact captured operation→member/target mapping. Select from that captured mapping after structural shape selection. Do not query `AssociatedSurface`, rerun inheritance, or rerun lexical visibility at the later call site.

If only a structural `TypeData::Family` remains because denotation was lost at a merge/abstraction boundary, type-check the selected structural callable operation without claiming a nominal `InvocationTargetId`. The runtime value itself will carry execution identity in Part 4; Part 3 must not regain privileges/identity by nominal re-resolution.

- [ ] **Step 5: Publish `FamilyApplicationResolution` for every family-value call.**

For static shape, record the selected `FamilyOperationShape`, structural callable type, result type, and `target: Some(InvocationTargetId)` only when captured denotation proves it. For a structural family whose denotation was dropped, record the same structural operation with `target: None`.

For dynamic shape, record the finite candidate operation/callable table with optional targets. Part 4 must be able to lower the call using this product without querying `AssociatedSurface` or re-running `select_family_member`.

Add a regression that performs a family call after denotation loss and asserts the application resolution exists with `target: None`.

- [ ] **Step 6: Fuse immediate unspecialized family invocation.**

For AST equivalent to:

```phalcom
Option::Some::*(1)
```

allow arguments/expected result to participate in generic instantiation before requiring a standalone monomorphic family value.

- [ ] **Step 7: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::canonical_call_application
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker phalcom-semantic/tests/semantic/foundations
git commit -m "feat(semantic): invoke first-class associated families"
```

---

# 19. Task 18 — Support Invocation of Exact Reified Getter/Constructor/Behavioral Callables

**Files:**

```text
Modify: phalcom-semantic/src/checker/call.rs
Modify: phalcom-semantic/src/checker/expression.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Ensures exact member reification composes with the ordinary callable value path.

- [ ] **Step 1: Write tests.**

```phalcom
const some = Option<Int>::Some::(_)
const a = some(1)

const getter = Math::pi
const p = getter()
```

- [ ] **Step 2: Preserve the captured invocation target and bound lookup owner when an exact associated callable flows through a binding.**

The current callable value path reconstructs a structural `call` signature from `TypeData::Callable` and loses declaration identity. Extend denotation-aware invocation so `AssociatedValueDenotation::Exact` supplies both the exact `InvocationTargetId` and the original `owner_form`/`lookup_owner`. For inherited class-side behavior, do not replace that owner with the defining `CallableId.owner`. Do not rerun associated resolution.

- [ ] **Step 3: Do not reinterpret a singleton variant value as a zeroarg constructor.**

```phalcom
(Option<Int>::None)()
```

is ordinary call on the singleton case value. It only succeeds if that case type itself has callable behavior; it never redirects to `#None()`.

- [ ] **Step 4: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): preserve exact associated callable identity"
```

---

# 20. Task 19 — Enforce GADT Owner Compatibility for Invocation and Reification

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/diagnostic.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: Part 2 `CaseTypeEnvironment` and result templates.
- Produces: specialized exact case or explicit GADT owner conflict.

- [ ] **Step 1: Write the canonical GADT tests.**

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
}
const a = Expr::Int(1)
const b = Expr<Int>::Int(1)
const c = Expr<String>::Int(1)
const d = Expr<String>::Int::(_)
```

Assert `a` and `b` succeed with exact `Expr<Int>` case; `c` and `d` diagnose the same owner/GADT contradiction category.

- [ ] **Step 2: Feed case equivalences into the associated instantiation session before call argument checking.**

- [ ] **Step 3: Detect contradictions separately from ordinary argument inference conflict.**

Use diagnostic `associated.gadt.owner_conflict` and attach both explicit owner specialization and variant result specialization evidence.

- [ ] **Step 4: Verify exact result type remains exact case after solving.**

Do not widen to enum root solely because the GADT result is specialized.

- [ ] **Step 5: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker/associated.rs phalcom-semantic/src/diagnostic.rs phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): enforce GADT associated owner compatibility"
```

---
# 21. Task 20 — Implement Frozen-Candidate Dynamic-Pack Semantics

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/call.rs
Modify: phalcom-semantic/src/diagnostic.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: `StaticCallShape::Dynamic`, effective accessible family members.
- Produces: `AssociatedResolutionKind::DynamicInvoke` for direct associated syntax and `FamilyApplicationSelection::Dynamic` for ordinary calls on reified family values, both with finite frozen candidate tables.

- [ ] **Step 1: Write a dynamic-pack semantic test.**

For a family with two callable shapes and:

```phalcom
const f = Response::Error::*
f(*args)
```

assert the formal resolution candidate list contains exactly those statically accessible family members and no runtime hierarchy search target.

- [ ] **Step 2: Write a no-dNU architecture test.**

The dynamic associated failure path must not use `UnknownReason::DynamicMessageSend` merely because a normal message-send helper was reused. Add an associated-specific dynamic boundary or blocked reason if necessary.

- [ ] **Step 3: Implement candidate filtering.**

For dynamic shape, retain only Method-kind callable members and Task 14A lane-aware rest candidates that could accept some runtime shape. Getter/value members are not call candidates.

- [ ] **Step 4: Compute finite result joins when sound.**

If all candidate returns are fully specialized known types, join them using canonical `join_type_knowledge`. If unresolved generics make the join unsound, publish the associated dynamic/blocked boundary.

- [ ] **Step 5: Publish the correct dynamic resolution product for both entry paths.**

For `Owner::family(*args)`, publish `AssociatedResolutionKind::DynamicInvoke`. For `f(*args)` where `f` is `TypeData::Family`, publish `FamilyApplicationSelection::Dynamic`. Include candidate operation/callable types and optional invocation targets in canonical order so Part 4 can implement runtime pack routing without semantic rediscovery.

- [ ] **Step 6: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker phalcom-semantic/src/diagnostic.rs phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): freeze dynamic associated family routing"
```

---

# 22. Task 21 — Complete Visibility Semantics and Capability Preservation

**Files:**

```text
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/context.rs
Modify: phalcom-semantic/src/diagnostic.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Consumes: behavioral `MemberVisibility`, Part 2 `VariantVisibility`.
- Produces: exact access errors and access-filtered stable family views.

- [ ] **Step 1: Write behavioral access tests.**

Cover public/private/protected/internal exact refs from inside/outside appropriate owners, including inherited defining owner.

- [ ] **Step 2: Write variant construction visibility tests.**

A private variant constructor and private singleton production fail outside the allowed context while remaining semantic variant identities for later matching work.

- [ ] **Step 3: Write exact-versus-rest access test.**

If exact Method shape exists but is private and a public rest method would otherwise accept the call, assert access error on the exact member.

- [ ] **Step 4: Write captured-capability tests for both preserved and erased denotation.**

First, acquire a private family/member inside the allowed owner and flow it through a local binding; assert the captured denotation contains the exact authorized lookup owner/member mapping and later invocation does not re-run lexical visibility.

Second, exercise an abstraction boundary that preserves the inferred structural family type but not exact denotation—for example an owner method that legally returns the captured family and an external caller that receives that structural family value. The external call may type-check through the structural family interface; it must **not** re-resolve the nominal private family or perform a fresh private visibility check. Part 4 will make the runtime value carry the actual capability.

- [ ] **Step 5: Centralize associated access checks.**

Add one helper for behavioral and one for variant production rather than scattering attribute checks through selection code.

- [ ] **Step 6: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/checker phalcom-semantic/src/diagnostic.rs phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
git commit -m "feat(semantic): enforce associated acquisition visibility"
```

---

# 23. Task 22 — Publish Associated Explanations and Evidence

**Files:**

```text
Modify: phalcom-semantic/src/explain/node.rs
Modify: phalcom-semantic/src/checker/associated.rs
Modify: phalcom-semantic/src/checker/expression.rs
Test:   phalcom-semantic/tests/semantic/foundations/explanations_graph.rs
Test:   phalcom-semantic/tests/semantic/foundations/associated_resolution.rs
```

**Interfaces:**

- Produces explanation DAG steps for owner/family/member/specialization decisions.

- [ ] **Step 1: Add failing explanation tests.**

For `Option::Some(1)`, assert the expression explanation graph can reach evidence for:

```text
owner Option
family Some
selected #Some(_)
argument Int
result Option<Int> exact case
```

For GADT owner conflict, assert the diagnostic explanation retains explicit owner specialization and case result equality evidence.

- [ ] **Step 2: Add focused explanation variants.**

Use names equivalent to:

```rust
AssociatedOwnerResolution { ... }
AssociatedFamilyResolution { ... }
AssociatedMemberSelection { ... }
AssociatedFamilyCapture { ... }
OwnerTypeSpecialization { ... }
GadtOwnerCompatibility { ... }
```

Keep them machine-readable; presentation prose is separate.

- [ ] **Step 3: Chain into existing call explanations after target selection.**

Do not duplicate argument or generic-inference explanation nodes.

- [ ] **Step 4: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::explanations_graph
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
git add phalcom-semantic/src/explain phalcom-semantic/src/checker phalcom-semantic/tests/semantic/foundations
git commit -m "feat(semantic): explain associated family resolution"
```

---

# 24. Task 23 — Attach Exact Associated Source Targets Through Formal Analysis

**Files:**

```text
Modify: phalcom-semantic/src/identity.rs
Modify: phalcom-semantic/src/source_index/occurrence.rs
Modify: phalcom-semantic/src/source_index/mod.rs
Modify: phalcom-semantic/src/source_index/builder.rs     # only for structural site/range support, never semantic resolution
Modify: phalcom-semantic/src/session.rs                 # only where formal attachments are assembled
Test:   phalcom-semantic/tests/semantic/integration/source_index.rs
```

**Interfaces:**

- Consumes: formal `AssociatedResolutionIndex` from `CallableAnalysis` plus Part 1 `base_range`/exact member ranges.
- Produces: exact associated token occurrences without rerunning associated resolution in the source index.

- [ ] **Step 1: Add `SemanticTargetId::AssociatedFamily` if Part 2 WIP did not already add it.**

```rust
AssociatedFamily(AssociatedFamilyId)
```

Part 2 should already provide `SemanticTargetId::Variant(VariantId)`; do not duplicate it. Variant constructor references target the exact `VariantId` for navigation unless the source-index identity model has already ratified a distinct constructor target; constructor execution identity remains in `AssociatedResolution`.

- [ ] **Step 2: Make syntax-owned occurrence collection record the associated member token even before a target is known.**

`OccurrenceBuilder::expr` currently only traverses the receiver/arguments for associated expressions. Change it so:

```text
AssociatedInvoke base token      -> OccurrenceKind::Member,   OccurrenceRole::Call
AssociatedLookup named/exact     -> OccurrenceKind::Member,   OccurrenceRole::Reference
AssociatedLookup operator        -> OccurrenceKind::Operator, OccurrenceRole::Reference
AssociatedLookup subscript       -> appropriate member/operator reference occurrence
```

Use the Part 1 member/base/selector token range, not the full associated expression range. Continue traversing receiver and arguments normally.

Do not attempt to resolve the target in `OccurrenceBuilder`.

- [ ] **Step 3: Extend `CallableSourceAttachment::from_analysis_with_incidents` to consume formal associated resolutions.**

For each analyzed associated expression, derive its canonical navigation target from `analysis.associated_resolutions`:

```text
Exact singleton variant          -> SemanticTargetId::Variant(VariantId)
Exact variant constructor ref    -> SemanticTargetId::Variant(VariantId)
Direct selected variant call     -> SemanticTargetId::Variant(VariantId)
Exact behavioral ref/call        -> SemanticTargetId::Callable(CallableId)
Whole family                     -> SemanticTargetId::AssociatedFamily(AssociatedFamilyId)
```

Attach that target to the formal expression site or a dedicated formal associated-target entry keyed by `ExpressionId`; do not infer it from `TypeData` or denotation alone.

- [ ] **Step 4: Project formal targets onto the contained associated token occurrence.**

The existing `SourceSemanticIndex::attach_formal_analysis` already projects formal call targets onto contained call occurrences. Generalize that projection narrowly:

```text
- Call resolution may project to the one contained associated Call occurrence.
- Exact/family references may project to the one contained associated Reference occurrence.
- Prefer the Part 1 selector/base token range when available.
- Never smear the target onto nested ordinary calls/references inside the receiver or arguments.
```

If more than one candidate occurrence fits, fail closed/retain an attachment incident rather than selecting by name heuristics.

- [ ] **Step 5: Write source-index tests.**

Required mappings:

```text
Option::None          -> VariantId #None
Option::Some::(_)     -> VariantId #Some(_)
Option::Some(1)       -> selected VariantId #Some(_)
System::print::(_)    -> CallableId
Option::Some::*       -> AssociatedFamilyId
```

For inherited `Derived::build::(_)`, target the defining `CallableId` while `AssociatedResolution` still records lookup owner `Derived`.

Add a nested/chained expression regression test proving a formal associated target is attached only to the associated selector token and not to a nested receiver/argument call.

- [ ] **Step 6: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::integration::source_index
git add phalcom-semantic/src/identity.rs phalcom-semantic/src/source_index phalcom-semantic/src/session.rs phalcom-semantic/tests/semantic/integration/source_index.rs
git commit -m "feat(semantic): index associated family and member targets"
```

---

# 25. Task 24 — Complete Incremental Fingerprints and Dependency Propagation

**Files:**

```text
Modify: phalcom-semantic/src/checker/analysis.rs
Modify: phalcom-semantic/src/db/fingerprint.rs
Modify: phalcom-semantic/src/db/query.rs
Modify: phalcom-semantic/src/db/product.rs
Modify: phalcom-semantic/src/session.rs
Modify: phalcom-semantic/src/snapshot.rs
Test:   phalcom-semantic/tests/semantic/incremental/fingerprints.rs
Test:   phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
Test:   phalcom-semantic/tests/semantic/incremental/checker_dependencies.rs
Test:   phalcom-semantic/tests/semantic/incremental/product_stability.rs
Test:   phalcom-semantic/tests/semantic/incremental/query_ownership.rs
```

**Interfaces:**

- Makes body reuse sensitive to associated semantic inputs without depending on source ranges/runtime representation.

- [ ] **Step 1: Write failing invalidation tests.**

A callable containing `Option::Some(1)` must reanalyze when:

```text
#Some(_) signature changes
variant GADT result changes
construction visibility changes
family membership changes
```

- [ ] **Step 2: Write non-invalidation tests.**

The same callable should remain reusable when:

```text
unrelated variant body implementation changes with unchanged signature
source ranges move but semantic associated fingerprints do not change
```

- [ ] **Step 3: Ensure `AssociatedResolution` itself is not an upstream global query product.**

It is a body result derived from upstream family/signature products. Its deterministic body fingerprint may include semantic target/type IDs as appropriate.

- [ ] **Step 4: Add canonical family-type/resolution hashing.**

Do not hash AST ranges, display strings, hash-map iteration order, runtime pointers, or advisory shape.

- [ ] **Step 5: Run the dependency/reuse suites, then the complete incremental module, and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::incremental::callable_dependencies
cargo test -p phalcom-semantic --test semantic semantic::incremental::checker_dependencies
cargo test -p phalcom-semantic --test semantic semantic::incremental::product_stability
cargo test -p phalcom-semantic --test semantic semantic::incremental::query_ownership
cargo test -p phalcom-semantic --test semantic semantic::incremental

git add phalcom-semantic/src/checker/analysis.rs phalcom-semantic/src/db phalcom-semantic/src/session.rs phalcom-semantic/src/snapshot.rs phalcom-semantic/tests/semantic/incremental
git commit -m "test(semantic): track associated resolution dependencies"
```

---

# 26. Task 25 — Add Diagnostic Presentation for Associated Failures

**Files:**

```text
Modify: phalcom-semantic/src/diagnostic.rs
Modify: phalcom-semantic/src/diagnostic_presenter.rs      # or actual current presenter module
Modify: phalcom-semantic/src/presentation.rs
Test:   phalcom-semantic/tests/semantic/foundations/diagnostics.rs
Test:   phalcom-semantic/tests/semantic/foundations/diagnostic_presentation.rs
Test:   phalcom-semantic/tests/semantic/golden/* appropriate fixture
```

**Interfaces:**

- Consumes: structured associated diagnostics/explanations.
- Produces: stable, concise user-facing diagnostic content.

- [ ] **Step 1: Add golden/foundation cases for every new diagnostic code.**

At minimum:

```text
owner not type form
family missing
exact member missing
member inaccessible
family call shape missing
underconstrained associated generic
GADT owner conflict
```

- [ ] **Step 2: Present requested/available selector shapes precisely.**

Keep selector labels colon-free in semantic identity, but source-facing diagnostics may render normal call labels with `:` where appropriate in prose.

- [ ] **Step 3: Ensure getter/zeroarg distinction is visible.**

For example:

```text
family `None` has getter `#None`, but no zero-argument constructor `#None()`
```

rather than generic “not callable.”

- [ ] **Step 4: Keep explanation evidence protocol-neutral.**

No terminal-color/LSP logic enters the semantic checker.

- [ ] **Step 5: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::diagnostics
cargo test -p phalcom-semantic --test semantic semantic::foundations::diagnostic_presentation
cargo test -p phalcom-semantic --test semantic semantic::golden

git add phalcom-semantic/src phalcom-semantic/tests/semantic
git commit -m "feat(semantic): diagnose associated lookup and invocation"
```

---
# 27. Task 26 — Adapt Advisory Analysis Without Creating a Second Resolver

**Files:**

```text
Modify: phalcom-semantic/src/advisory/analyzer.rs
Modify: phalcom-semantic/src/advisory/shape.rs
Modify: phalcom-semantic/src/advisory/mod.rs
Modify: phalcom-semantic/src/session.rs
Test:   phalcom-semantic/tests/semantic/foundations/advisory_domain.rs
Test:   phalcom-semantic/tests/semantic/integration/advisory_analysis.rs
```

**Interfaces:**

- Consumes: formal `AssociatedResolution` / exact invocation target.
- Produces: advisory runtime-shape projection only.

- [ ] **Step 1: Add a regression test proving advisory associated analysis consumes formal attachment.**

The test should fail if the advisory layer attempts to infer `Option::Some` from receiver shape without a formal associated resolution.

- [ ] **Step 2: Add projection for exact variant values/callables/families only where runtime-shape semantics are already representable.**

If Part 4 has not introduced a trustworthy runtime family representation, use `ValueShape::Unknown` for runtime shape while preserving formal type/denotation elsewhere.

- [ ] **Step 3: Do not reuse legacy `resolve_method_family` as the formal `::` resolver.**

Legacy captured method-family machinery may remain for `>>`/old reflection behavior until revisited.

- [ ] **Step 4: Ensure advisory does not widen exact formal type evidence.**

- [ ] **Step 5: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::advisory_domain
cargo test -p phalcom-semantic --test semantic semantic::integration::advisory::analysis

git add phalcom-semantic/src/advisory phalcom-semantic/src/session.rs phalcom-semantic/tests/semantic
git commit -m "refactor(advisory): consume formal associated resolutions"
```

---

# 28. Task 27 — Clean Part 1 Stale Associated Comments and Preserve Compiler Staging

**Files:**

```text
Modify: phalcom-ast/src/ast.rs
Verify: phalcom-core/src/compiler/lib/expr.rs
Create: phalcom-core/tests/associated_staging.rs
Test:   phalcom-ast/tests/family_selector_syntax.rs
```

**Interfaces:** documentation/architecture hygiene plus an executable guard proving Part 3 did not cross the Part 4 boundary; no parser grammar change and no runtime lowering.

- [ ] **Step 1: Locate stale old-method-reference comments.**

```bash
rg -n 'method reference|bound forms|MakeFamily|receiver::#' phalcom-ast/src phalcom-core/src/compiler
```

- [ ] **Step 2: Update comments around `AssociatedLookupExpr` / `AssociatedInvokeExpr`.**

Describe them as static associated syntax produced by Part 1 and resolved by `phalcom-semantic`; remove claims that they are bound method-reference syntax.

- [ ] **Step 3: Verify the exact current compiler staging guards.**

At the verified Part 1 branch they are:

```rust
Expr::AssociatedLookup(expr) => {
    return Err(CompilerError::AssociatedLookupNotLoweredYet(expr.range));
}
Expr::AssociatedInvoke(expr) => {
    return Err(CompilerError::AssociatedInvokeNotLoweredYet(expr.range));
}
```

If names changed mechanically, use the equivalent explicit staging errors. Do not replace them with `MakeFamily`, `Invoke`, `InvokePack`, or any VM lookup path.

- [ ] **Step 4: Add a dedicated staging regression test.**

In `phalcom-core/tests/associated_staging.rs`, compile one exact lookup and one direct invocation and assert the compiler returns the corresponding not-lowered error. This test must not inspect or expect legacy `Bytecode::MakeFamily`.

Keep legacy ignored tests in `phalcom-core/tests/family_selector_runtime.rs` ignored if they still encode pre-Part-1 `::` semantics; do not unignore them as evidence for Part 3.

- [ ] **Step 5: Run the staging/parser tests.**

```bash
cargo test -p phalcom-ast --test family_selector_syntax
cargo test -p phalcom-core --test associated_staging
```

- [ ] **Step 6: Commit.**

```bash
git add phalcom-ast/src/ast.rs phalcom-core/tests/associated_staging.rs
git commit -m "test(core): preserve associated lowering boundary"
```

---

# 29. Task 28 — Add Full End-to-End Semantic Integration Coverage

**Files:**

```text
Create: phalcom-semantic/tests/semantic/integration/associated_lookup.rs
Modify: phalcom-semantic/tests/semantic/integration/mod.rs
Modify: phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md
```

**Interfaces:** verifies composition across parser → declaration products → body checker → source/explanation products.

- [ ] **Step 1: Add one composite enum fixture.**

Include:

```phalcom
enum Option<T> {
    @variant None
    @variant None()
    @variant Some(_ value: T)
}
```

Test exact singleton, zeroarg constructor, constructor ref, family ref, direct call, call-on-family, expected-type specialization, and source targets in one module. Include at least one behavioral getter/zeroarg distinction and the explicit getter alias `owner::name::`. Setter/operator/subscript exact forms remain covered in the focused Task 12 suite.

- [ ] **Step 2: Add the recorded Decision Gate G1 end-to-end case.**

If G1-A was ratified, prove an escaping unspecialized family is underconstrained while immediate application can still infer. If G1-B was ratified, prove one stored declaration-polymorphic family can be independently instantiated at two distinct types. In either case, assert no Dynamic/Object defaulting.

- [ ] **Step 3: Add one composite GADT fixture.**

Exercise exact result typing and owner contradiction.

- [ ] **Step 4: Add one behavioral inheritance fixture.**

Exercise effective class-side family composition, exact override selection, family reification, and defining target navigation.

- [ ] **Step 5: Add one negative architecture fixture.**

Use a family shape that ordinary message dispatch/dNU would accept through class behavior but static `::` must reject, proving no fallback path exists.

- [ ] **Step 6: Update the semantic coverage ledger with exact capability rows.**

Do not mark Part 4 runtime execution or Part 5 `match` as covered.

- [ ] **Step 7: Run integration tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic semantic::integration::associated_lookup

git add phalcom-semantic/tests/semantic/integration phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md
git commit -m "test(semantic): cover associated lookup end to end"
```

---

# 30. Task 29 — Audit for Semantic Leaks Into Ordinary Dispatch or Legacy Runtime Family Machinery

**Files:** repository-wide audit; only modify files where an actual Part 3 leak exists.

- [ ] **Step 1: Search associated checker for ordinary dispatch calls.**

```bash
rg -n 'resolve_dispatch|DispatchLookup|doesNotUnderstand|dNU|MakeFamily|MethodFamilyResolver|resolve_method_family' \
  phalcom-semantic/src/checker/associated.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/call.rs
```

Expected:

- ordinary message-send branches in `expression.rs` / `call.rs` may still use dispatch;
- `checker/associated.rs` must not use ordinary dispatch as family discovery;
- application after exact target selection may reuse call checking but not dispatch selection.

- [ ] **Step 2: Search compiler lowering.**

```bash
rg -n 'AssociatedLookup|AssociatedInvoke|MakeFamily' phalcom-core/src
```

Verify new associated AST is still not lowered through legacy `MakeFamily`.

- [ ] **Step 3: Search for getter/zeroarg conflation.**

```bash
rg -n 'SelectorKind::Getter|SelectorKind::Method' phalcom-semantic/src/checker/associated.rs phalcom-semantic/src/types/family.rs
```

Manually verify no `Getter => Method([])` normalization exists for family selection.

- [ ] **Step 4: Search for family-as-union shortcuts.**

```bash
rg -n 'TypeData::Union|\.union\(' phalcom-semantic/src/checker/associated.rs phalcom-semantic/src/types/family.rs
```

A union is allowed only for sound dynamic-call result joins, never to represent a family value itself.

- [ ] **Step 5: Search for generic default erasure.**

```bash
rg -n 'Dynamic|Object|Any' phalcom-semantic/src/checker/associated.rs
```

Every hit must correspond to an explicit dynamic boundary or diagnostic, not unresolved generic defaulting.

- [ ] **Step 6: Record audit results in the final implementation report.**

If no code changes are needed, do not create an empty audit commit.

---

# 31. Task 30 — Focused Part 3 Verification

Run the focused suites after all implementation tasks.

- [ ] **Parser/AST prerequisites:**

```bash
cargo test -p phalcom-ast --test family_selector_syntax
cargo test -p phalcom-ast --test enum_syntax
```

- [ ] **Core type machinery:**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::type_model
cargo test -p phalcom-semantic --test semantic semantic::foundations::substitution
cargo test -p phalcom-semantic --test semantic semantic::foundations::kinds
cargo test -p phalcom-semantic --test semantic semantic::foundations::generics_core
cargo test -p phalcom-semantic --test semantic semantic::foundations::generic_inference_proof_integrity
```

- [ ] **Canonical call machinery:**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::canonical_call_application
cargo test -p phalcom-semantic --test semantic semantic::foundations::bidirectional_calls
```

- [ ] **Associated semantics:**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::associated_resolution
cargo test -p phalcom-semantic --test semantic semantic::integration::associated_lookup
```

- [ ] **Source/incremental/advisory:**

```bash
cargo test -p phalcom-semantic --test semantic semantic::integration::source_index
cargo test -p phalcom-semantic --test semantic semantic::incremental
cargo test -p phalcom-semantic --test semantic semantic::foundations::advisory_domain
cargo test -p phalcom-semantic --test semantic semantic::integration::advisory::analysis
```

- [ ] **Diagnostics/explanations:**

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::diagnostics
cargo test -p phalcom-semantic --test semantic semantic::foundations::diagnostic_presentation
cargo test -p phalcom-semantic --test semantic semantic::foundations::explanations_graph
```

Any failure caused by Part 3 must be fixed before continuing. Pre-existing failures recorded in Task 0 may remain only if they are demonstrably unrelated and documented.

---

# 32. Task 31 — Full Workspace Verification

- [ ] **Step 1: Format.**

```bash
cargo fmt --all -- --check
```

If it fails, run:

```bash
cargo fmt --all
cargo fmt --all -- --check
```

- [ ] **Step 2: Build/check the workspace.**

```bash
cargo check --workspace
```

- [ ] **Step 3: Run semantic crate tests.**

```bash
cargo test -p phalcom-semantic
```

- [ ] **Step 4: Run the full workspace tests.**

```bash
cargo test --workspace
```

- [ ] **Step 5: Run Clippy using the repository's normal policy.**

Prefer:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

If the repository has known baseline Clippy failures, record the exact pre-existing diagnostics and still ensure no new Part 3 warnings are introduced.

- [ ] **Step 6: Verify compiler staging with the dedicated regression target.**

```bash
cargo test -p phalcom-core --test associated_staging
```

This is the executable proof that runtime compilation still stops at the explicit Part 4 boundary rather than accidentally executing legacy family behavior.

- [ ] **Step 7: Run the architecture searches from Task 29 again after formatting/fixes.**

---

# 33. Task 32 — Final Semantic Invariant Checklist

Before claiming Part 3 complete, manually verify each invariant against code plus tests.

- [ ] `::` family/member discovery is static and compiler-owned.
- [ ] `.` ordinary dispatch behavior is unchanged.
- [ ] `>>` is untouched except mechanical compile adaptations, if any.
- [ ] no live-provider/monkey-patching semantics were introduced.
- [ ] associated owner derives from exact `TypeForm` denotation.
- [ ] arrow-kinded bare/partial declaration forms can be associated owners.
- [ ] generic type parameters without declaration identity are not speculatively searched.
- [ ] inherited class-side behavior composes statically by exact selector.
- [ ] inherited generic behavior is specialized through generic superclass templates to the defining owner before lookup-owner `Self` specialization.
- [ ] static associated lookup does not enter `Class` instance behavior tail.
- [ ] direct associated variants do not inherit.
- [ ] one family preserves `#None`, `#None()`, and `#None(_)` distinctly.
- [ ] family values use `TypeData::Family`, never callable union encoding.
- [ ] family structural identity excludes selector base name.
- [ ] family denotation retains `AssociatedFamilyId`, lookup-owner form/specialization, and the exact access-filtered captured member bindings; nominal family ID alone is never used to reconstruct a capability.
- [ ] every ordinary family-value call publishes `FamilyApplicationResolution`; a lost nominal denotation is represented by `target: None`, never by associated re-resolution.
- [ ] variant constructor identity is not `CallableId`.
- [ ] singleton lookup returns a value, not a fake getter callable.
- [ ] behavioral getter lookup reifies a callable/member, not its result.
- [ ] exact behavioral/constructor callable denotation retains the owner form against which it was acquired, including inherited lookup-owner binding.
- [ ] family invocation uses Method selector kind only.
- [ ] callable rest lanes remain positional/labeled/complete-aware in canonical signatures/types and the static binder.
- [ ] exact inaccessible member does not fall through to rest.
- [ ] static direct calls can bypass family reification.
- [ ] direct and reified-family calls share member-selection/application logic.
- [ ] canonical call binder/inference remains the only call checking engine.
- [ ] GADT owner contradiction is distinguished from payload mismatch.
- [ ] unresolved generic variables are never default-erased; direct result-relevant unsolved variables follow the concrete-operation underconstraint rule, while reified residual binders follow the explicitly recorded G1 decision.
- [ ] source occurrences use formal associated resolution targets.
- [ ] incremental body dependencies include associated products/hierarchy actually consumed.
- [ ] advisory does not become formal associated authority.
- [ ] `AssociatedResolution` is sufficient for Part 4 lowering.
- [ ] `phalcom-core` still does not lower new associated AST.

---

# 34. Recommended Commit Sequence

A disciplined sequence that keeps reviewable semantic boundaries:

```text
feat(types): add canonical associated family types
feat(types): propagate associated family types
feat(semantic): model associated invocation resolutions
feat(semantic): preserve associated value denotations
feat(semantic): expose tracked associated declaration products
feat(semantic): resolve associated type-form owners
feat(semantic): compose static effective associated families
feat(semantic): specialize associated member templates
feat(semantic): type associated family views
feat(semantic): infer associated owner specializations
feat(semantic): resolve exact variant singleton values
feat(semantic): reify exact associated callable members
feat(semantic): reify static associated family values
refactor(semantic): generalize executable call targets
refactor(semantic): preserve callable rest lanes
feat(semantic): select static associated family members
feat(semantic): type direct associated family invocation
feat(semantic): invoke first-class associated families
feat(semantic): preserve exact associated callable identity
feat(semantic): enforce GADT associated owner compatibility
feat(semantic): freeze dynamic associated family routing
feat(semantic): enforce associated acquisition visibility
feat(semantic): explain associated family resolution
feat(semantic): index associated family and member targets
test(semantic): track associated resolution dependencies
feat(semantic): diagnose associated lookup and invocation
refactor(advisory): consume formal associated resolutions
docs(ast): describe static associated expression semantics
test(semantic): cover associated lookup end to end
```

If Part 2 work remains intentionally uncommitted, do not force this exact commit granularity; preserve the same conceptual review boundaries in the diff/report.

---

# 35. Final Implementation Report Requirements

The implementing agent's final report must include:

1. starting branch and exact starting SHA;
2. whether Part 2 was committed or WIP and the exact structural-name reconciliations made;
3. final SHA if commits were created;
4. files added/modified;
5. family type representation actually implemented;
6. associated owner-resolution rules implemented;
7. inherited behavioral family composition behavior;
8. exact singleton/callable/family reification behavior;
9. `InvocationTargetId`/constructor call integration;
10. generic/GADT specialization behavior, including unresolved escaped generic policy;
11. dynamic-pack semantic status;
12. visibility/capability behavior;
13. source-index targets;
14. incremental dependency/fingerprint changes;
15. explanation/diagnostic changes;
16. exact focused/full test commands and results;
17. `cargo fmt`/`cargo check`/Clippy status;
18. every known baseline failure that remains;
19. deviations from this plan and why;
20. discoveries that should change Part 4 planning;
21. explicit confirmation that no runtime lowering, VM representation, `match`, `>>` redesign, monkey-patching, or live-provider semantics were added.

---

# 36. Completion Boundary

Part 3 is done when semantic analysis can prove and publish the meaning of associated expressions, including first-class families and exact constructors, while runtime execution remains intentionally staged. Full completion also requires Decision Gate G1 to be recorded if bare generic associated values/families can escape without contextual specialization; an executor must not silently choose that language policy.

The final handoff to Part 4 must look like:

```text
Associated AST
    ↓
Part 3 static AssociatedResolution
    ├── exact value
    ├── exact callable
    ├── first-class family
    ├── static direct invocation
    └── frozen-candidate dynamic invocation
    ↓
Part 4 lowering only
```

If Part 4 would need to ask “which family/member did this source expression mean?” **or** “which structural family operation did this ordinary family call select?” then Part 3 is incomplete.
