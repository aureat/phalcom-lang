# Covering Inherent-Impl Body Coordinate Closure — Corrective Implementation Handoff

## 0. Purpose

This handoff describes the remaining work required to **fully close the covering-inherent-impl body-coordinate defect** in the Telos semantic compiler.

The repository was re-audited at:

```text
03df529c4f543e42c908d9b617938b075ed35a65
fix(semantic): specialize Self in inherent impl bodies
```

The current implementation is substantially correct. Do **not** discard or redesign it.

The original bug was the mismatch between:

```text
source impl coordinate space
    TypeParameterOwner::Impl

canonical declaration coordinate space
    TypeParameterOwner::Declaration
```

when checking bodies of covering inherent implementations such as:

```telos
data Id<Tag>(_ token: Token)

impl<Tag> Id<Tag> {
  @class
  fresh() -> Id<Tag> {
    Id<Tag>(Token())
  }
}
```

The compiler correctly maintains distinct parameter identities:

```text
Declaration(Id)::Tag = D
Impl(the_impl)::Tag   = I
```

and correctly canonicalizes public signatures:

```text
source body signature:
    Id<I>

publication:
    I -> D

canonical surface:
    Id<D>
```

The required body view is the inverse:

```text
canonical:
    Id<D>

body view:
    D -> I

source-body contract:
    Id<I>
```

That architecture is now present and must be preserved.

The current patch also correctly separates **type-level `Self`** from the runtime lowercase `self` receiver. That separation must likewise be preserved.

There are, however, **two remaining semantic completeness defects** plus several mandatory closure tests that are still absent.

Do not mark this incident closed until all requirements in this handoff are satisfied.

---

# 1. Architecture that is already correct and must not be redesigned

## 1.1 Distinct binder identities are intentional

For:

```telos
data Pair<X, Y>(...)

impl<A, B> Pair<B, A> {
  ...
}
```

the declaration parameters and implementation parameters are intentionally different semantic identities.

Conceptually:

```text
Declaration:
    X = D0
    Y = D1

Impl:
    A = I0
    B = I1
```

For the covering target:

```text
Pair<B, A>
```

the mapping is:

```text
I(B) -> D(X)
I(A) -> D(Y)
```

and the inverse is:

```text
D(X) -> I(B)
D(Y) -> I(A)
```

Never:

- compare generic parameters by source spelling;
- collapse `Impl` and `Declaration` ownership;
- make different `TypeParameterId`s equal;
- teach the inference solver to ignore binder ownership;
- introduce name-based generic unification.

`CoveringImplSubstitution` is the correct authority.

---

## 1.2 Public signatures remain declaration-canonical

`EffectiveCallableDefinition.signature` is the authoritative public/canonical callable signature.

For a covering implementation, it must contain declaration-owned coordinates.

Example:

```telos
impl<A, B> Pair<B, A> {
  firstValue() -> B { ... }
}
```

must publish approximately:

```text
Pair<X, Y>::firstValue() -> X
```

It must **not** publish an `ImplId`-owned `B`.

The invariant remains:

```text
accepted unconditional surface signature
    contains no free TypeParameterOwner::Impl(current_impl)
```

Do not solve body checking by changing the published signature into impl coordinates.

---

## 1.3 `InherentBodyContext` is source-only provenance

The current architecture:

```rust
pub struct EffectiveCallableDefinition {
    pub signature: CallableSemanticSignature,
    pub inherent_body_context: Option<InherentBodyContext>,
    ...
}
```

is correct.

The intended split is:

```text
EffectiveCallableDefinition.signature
    = canonical public contract

InherentBodyContext
    = source-coordinate information required to check the original body
```

Do not create two competing canonical signatures.

The body signature is a **view/specialization** of the canonical signature, not a second public semantic authority.

---

## 1.4 Covering body checking uses the inverse substitution

The current body flow:

```text
source signature
     |
     | impl_to_decl
     v
canonical public signature
     |
     | decl_to_impl
     v
source-body view
```

is correct.

The law is:

```text
body_view(canonicalize(source_contract))
    ≡
source_contract
```

for a covering impl, modulo canonical normalization.

This must hold recursively for every supported generic/type structure.

---

## 1.5 Type-level `Self` and runtime lowercase `self` are different concepts

The current addition of:

```rust
self_type_override
```

versus:

```rust
self_instance_type_override
```

is architecturally correct.

Preserve this distinction.

### Runtime receiver

For:

```telos
@class
fresh() -> Self { ... }
```

lowercase:

```telos
self
```

still denotes the runtime class-side receiver/class object.

It must **not** suddenly have type `Id<Tag>`.

### Type-level `Self`

The type annotation:

```telos
Self
```

inside:

```telos
impl<Tag> Id<Tag>
```

must denote the source-relative instance target:

```text
Id<I_Tag>
```

The correct model is therefore approximately:

```text
SelfRole::ReceiverValue
    !=
SelfRole::InstanceType
```

Do not collapse these roles.

---

# 2. Confirmed remaining defect A — `Self` is not specialized inside callable-local generic constraints

## 2.1 Current implementation

The current body specialization path eventually invokes:

```rust
specialize_signature_instance_self(...)
```

in:

```text
phalcom-semantic/src/impls.rs
```

That helper currently specializes:

- callable parameter declared types;
- declared return;
- inferred return.

Conceptually:

```rust
for parameter in &mut signature.parameters {
    specialize(...);
}

specialize(..., &mut signature.declared_return);

if let Some(inferred) = &mut signature.inferred_return {
    ...
}
```

But it does **not** specialize:

```rust
signature.generics.constraints
```

That is semantically incomplete.

---

## 2.2 Why this is a real correctness bug

Callable-local generic constraints are part of the callable contract.

For example:

```telos
impl<T> Box<T> {
  coerce<U>(_ value: U) -> Self
    where U == Self
  {
    value
  }
}
```

Inside this source body, the constraint must mean:

```text
U == Box<I_T>
```

not:

```text
U == owner-relative canonical Self
```

and not:

```text
U == Box<D_T>
```

after the rest of the body has already been converted to impl coordinates.

`checker/body.rs` later constructs callable-local evidence directly from:

```rust
signature.generics.constraints
```

using:

```rust
GenericEvidenceEnvironment::try_from_constraints(...)
```

Therefore leaving `Self` unspecialized in those constraints means the body environment can contain a mixture of coordinate systems:

```text
parameters / result:
    source impl coordinates

generic constraints:
    canonical owner-relative Self
```

That violates the fundamental body-view invariant.

---

# 3. Required architectural fix for callable generic constraints

Do **not** add another one-off implementation in `impls.rs`.

There is already comparable logic in:

```text
phalcom-semantic/src/checker/context.rs
CheckingContext::specialize_dispatch_signature(...)
```

That path already understands that specialization must recurse through:

```text
GenericConstraint::Subtype
GenericConstraint::Equivalent
GenericConstraint::Conforms
```

and through:

```text
TypeTerm::Canonical(...)
TypeTerm::SelfType(...)
TraitRef.arguments
```

The implementation should consolidate the semantic transformation instead of maintaining two subtly different walkers.

## 3.1 Preferred direction

Extract reusable low-level operations into an appropriate type/substitution module, for example conceptually:

```rust
specialize_self_type_term(
    store,
    declarations,
    receiver,
    term,
) -> TypeTerm
```

and:

```rust
specialize_self_generic_constraint(
    store,
    declarations,
    receiver,
    constraint,
) -> GenericConstraint
```

and possibly:

```rust
specialize_self_generic_signature(
    store,
    declarations,
    receiver,
    signature,
) -> GenericSignature
```

Exact naming is up to the implementation, but there should be **one semantic authority** for this transformation.

The operation must:

### `TypeTerm::Canonical(ty)`

Perform:

```rust
specialize_self_type(...)
```

recursively.

### `TypeTerm::SelfType(term)`

Materialize the canonical `SelfType` and specialize it using the supplied source target.

Do not leave the `SelfType` term untouched in the body-local view.

### `TypeTerm::Infer`

Preserve it unless the existing semantic contract explicitly says otherwise.

### `GenericConstraint::Subtype`

Specialize both operands.

### `GenericConstraint::Equivalent`

Specialize both operands.

### `GenericConstraint::Conforms`

Specialize:

```text
subject
```

and **every**:

```text
trait_ref.arguments[i]
```

Do not specialize only the conformance subject.

---

# 4. Critical binder constraint during callable-generic specialization

Callable-local binders must remain callable-local.

For:

```telos
impl<T> Box<T> {
  f<T>(_ x: T) -> ...
}
```

there can simultaneously exist:

```text
Declaration(Box)::T
Impl(the_impl)::T
Callable(f)::T
```

All three are different.

The fix must never convert the callable-owned parameter merely because it shares the spelling `T`.

Specialization must affect only semantic terms that refer to:

- the covering declaration coordinates through the existing inverse substitution;
- owner-relative `Self`.

It must **not** rewrite arbitrary callable-owned binders.

Add a hostile shadowing regression for this.

---

# 5. Confirmed remaining defect B — `specialize_self_type()` is not recursively complete through RecordRow carriers

## 5.1 What has already been fixed correctly

The recent patch fixed `materialize_type()` in:

```text
phalcom-semantic/src/types/instantiation.rs
```

so that covering substitution now recurses into:

```rust
ProjectionSubject::RecordRow(...)
```

and:

```rust
TypeMemberArgument::RecordRow(...)
```

using:

```rust
materialize_row_id(...)
```

That repair is correct.

Do not revert it.

---

## 5.2 Remaining asymmetry

The new source-relative `Self` specialization relies on:

```text
phalcom-semantic/src/types/substitution.rs
specialize_self_type(...)
```

That traversal is still incomplete.

Several row-carrying positions remain opaque.

For example, it currently preserves:

```rust
StaticArgument::RecordRow(row) =>
    StaticArgument::RecordRow(*row)
```

instead of recursively specializing the row.

Likewise:

```rust
ProjectionSubject::RecordRow(row_id) =>
    ProjectionSubject::RecordRow(*row_id)
```

and:

```rust
TypeMemberArgument::RecordRow(row_id) =>
    TypeMemberArgument::RecordRow(*row_id)
```

A row can itself contain types involving `Self`.

For example, conceptually:

```text
{
    owner: Self
    payload: Option<Self>
    ...R
}
```

or a row used as a type-member subject/argument can embed such fields.

Those nested `Self` occurrences remain declaration-relative while surrounding body types become source-relative.

That breaks recursive semantic closure.

---

# 6. Required RecordRow `Self` specialization

Introduce or reuse a recursive row-specialization operation, conceptually:

```rust
specialize_self_record_row(
    store,
    declarations,
    receiver,
    row_id,
) -> RecordRowId
```

It must recursively specialize every type-bearing part of a row.

At minimum:

```text
RecordRowField.ty
```

must recurse.

For:

```text
RecordRowTail::Anonymous(Unconstrained { key_type })
```

specialize:

```text
key_type
```

For:

```text
RecordRowTail::Anonymous(Typed {
    key_type,
    value_type
})
```

specialize both.

For:

```text
RecordRowTail::Projection(type)
```

specialize the projection type recursively.

For:

```text
RecordRowTail::Parameter(...)
```

preserve the parameter identity.

For:

```text
RecordRowTail::Closed
```

preserve as closed.

Then use this helper for all row carriers encountered by `specialize_self_type()`:

```text
StaticArgument::RecordRow
ProjectionSubject::RecordRow
TypeMemberArgument::RecordRow
```

The transformation must continue to use canonical TypeStore formation APIs and preserve row invariants.

Do not mutate rows in place or bypass the store's canonical interning/validation.

---

# 7. Avoid duplicated structural walkers

This is an important architectural constraint.

There are now multiple transformations that recursively visit canonical types:

```text
TypeSubstitution
GenericInstantiation/materialize_type
specialize_self_type
receiver specialization
projection normalization
```

Do not blindly merge all of them into one giant visitor, but also do not add yet another incomplete hand-written traversal.

For the immediate patch:

- reuse existing row formation helpers;
- extract reusable `Self` term/constraint helpers where there is already duplicated behavior;
- preserve the semantic distinctions between parameter substitution and `Self` specialization.

The goal is:

```text
different transformations
    may have different semantics

but

each transformation must be structurally total
    over every carrier relevant to its semantics
```

---

# 8. Public-signature invariant must continue to hold

After the patch, verify again that:

```text
EffectiveCallableDefinition.signature
```

remains declaration-canonical.

The new body-local specialization must occur only on the cloned/body view produced by:

```rust
InherentBodyContext::signature_for_body(...)
```

The body specialization must not mutate:

```text
effective surface
callable signature table
metadata/public contract
reflection-visible signature
dispatch signature template
```

into impl coordinates.

The required invariant is:

```text
PUBLICATION

source impl signature
    -- impl_to_decl -->
canonical declaration signature
```

while:

```text
BODY CHECKING

canonical declaration signature
    -- decl_to_impl -->
source body signature
    -- source Self specialization -->
fully source-relative body contract
```

---

# 9. `Self` application must not become a workaround

The original investigation observed that:

```telos
Self<Tag>
```

appeared to work in at least one affected context.

Do **not** preserve that behavior merely because it bypasses this bug.

Under the existing canonical semantic model:

```text
Self :: Type
```

not:

```text
Self :: Type -> Type
```

`TypeStore::self_type(...)` produces a proper type.

Therefore, unless the language design has deliberately changed elsewhere, this should remain invalid:

```telos
Self<Tag>
```

The implementer must explicitly verify current behavior.

### Required closure check

Add or locate a negative regression proving:

```telos
Self<T>
```

is rejected as application of a non-constructor/proper type.

If it is currently accepted, classify and fix that as a separate type-formation defect.

Do **not** redefine `Self` into a constructor to make this incident disappear.

---

# 10. Already-fixed cache architecture that must be preserved

The current patch corrected `InherentBodyContext` fingerprinting.

It now includes:

```text
source_target_head
full GenericSignature semantic structure
covering impl_to_decl
covering decl_to_impl
```

and the callable body now records a dependency on:

```text
QueryKey::CallableDefinition(callable)
```

This is correct.

Do not revert to a hand-rolled partial hash containing only parameter IDs.

Do not make the body independently rediscover its impl context from syntax.

The authoritative dependency is:

```text
CallableBody
    ->
CallableDefinition
    ->
InherentBodyContext
```

---

# 11. Incremental correctness is not yet sufficiently certified

The current test:

```text
test_covering_body_incremental_binder_rename_matches_cold_analysis
```

only compares diagnostics:

```rust
assert_eq!(
    incremental.snapshot.diagnostics,
    cold.snapshot.diagnostics,
);
```

That is insufficient.

A stale semantic product can still have an empty diagnostic set.

Also, renaming:

```text
Tag -> Marker
```

throughout the declaration and body triggers many ordinary syntax/signature invalidations. It does not isolate the newly important body-context dependency.

---

# 12. Required context-only incremental regression

Construct an edit where:

- the public callable signature is unchanged;
- the callable body text is unchanged;
- the covering mapping changes;
- therefore the body context/fingerprint changes.

A useful fixture is:

```telos
data Pair<X, Y>()
```

Revision 1:

```telos
impl<A, B> Pair<A, B> {
  ping() -> Unit { () }
}
```

Revision 2:

```telos
impl<A, B> Pair<B, A> {
  ping() -> Unit { () }
}
```

The public member contract remains:

```text
ping() -> Unit
```

and the body remains:

```telos
()
```

but:

```text
source_target_head
```

changes from:

```text
Pair<I_A, I_B>
```

to:

```text
Pair<I_B, I_A>
```

and the covering substitution changes.

This is precisely the dependency the new `CallableDefinition` fingerprint is supposed to carry.

### Required assertions

Do not assert diagnostics alone.

Prove:

1. the `CallableDefinition` fingerprint/product changes appropriately;
2. the `CallableBody` query is revalidated/recomputed according to the query framework's revision/disposition API;
3. incremental final `CallableAnalysis` equals the corresponding cold analysis semantically;
4. public canonical callable signature remains unchanged;
5. `InherentBodyContext.source_target_head` reflects the new permutation.

Do not compare raw store-local `TypeId` integers between independent cold/incremental stores.

Compare normalized structure, presentation, stable fingerprints, or other canonical semantic products.

---

# 13. Already-fixed exact source provenance must be preserved

The current repair to:

```text
source_body_for_callable(...)
```

is correct.

It now uses the accepted:

```text
EffectiveCallableDefinition
```

and validates both:

```text
CallableDefinitionOrigin::InherentImpl(ImplId)
```

and:

```text
source_member_index
```

before selecting the AST body.

This is essential.

Never regress to:

```text
CallableId / selector
    -> scan impl blocks
    -> first matching body
```

A selector is not sufficient source identity.

The source-body identity is:

```text
ImplId
+
source_member_index
```

---

# 14. Required provenance regression

Even though the implementation is now architecturally correct, add a regression specifically protecting it.

The cleanest test may be a narrow unit test around the source-body selection helper.

Build a parsed module containing multiple impl members with the same selector or otherwise ambiguous selector-level identity.

Construct an `EffectiveCallableDefinition` pointing to a specific:

```text
ImplId
source_member_index
```

and assert that:

```text
source_body_for_callable(...)
```

returns exactly that member's body.

The test must fail if the implementation reverts to "first selector match".

Do not rely solely on conflict diagnostics to prove this property.

---

# 15. Required semantic regressions

The existing tests should remain.

In particular retain:

```text
test_generic_covering_permutation_body_analysis
test_covering_class_impl_body_uses_impl_generic_coordinates
test_covering_class_impl_body_specializes_bare_self
test_covering_permutation_body_specializes_parameters_and_return
test_covering_body_incremental_binder_rename_matches_cold_analysis
```

Add the following stronger tests.

---

## 15.1 Callable-local generic constraint containing `Self`

Add a source-level regression approximately like:

```telos
data Box<T>(_ value: T)

impl<T> Box<T> {
  identity<U>(_ value: U) -> Self
    where U == Self
  {
    value
  }
}
```

The exact fixture may be adjusted to the language's current proven-equivalence/body return semantics, but it must force the body checker to **consume** the callable-local constraint.

Do not write a test where the constraint is unused.

Expected:

```text
body-local U == Box<I_T>
```

and no declaration-relative `Self` remains in the body-local callable-generic contract.

---

## 15.2 Direct semantic-product assertion for generic constraints

In addition to the source-level test, add a lower-level assertion around:

```rust
InherentBodyContext::signature_for_body(...)
```

Given a canonical callable generic constraint containing `Self`, inspect the resulting body signature and assert:

```text
signature.generics.constraints
```

contains the source target type.

This makes the regression diagnostic-independent.

---

## 15.3 Callable-local generic shadowing

Add:

```telos
impl<T> Box<T> {
  f<T>(_ value: T) -> T {
    value
  }
}
```

or a stronger constrained version.

Assert that:

```text
Declaration(Box)::T
Impl(the_impl)::T
Callable(f)::T
```

remain three distinct identities.

The body-local `T` must refer to the callable-owned binder where shadowing applies.

No name-based capture is permitted.

---

## 15.4 Negative wrong-specialization test

Ensure the fix does not simply make unrelated generic types interchangeable.

For example, conceptually:

```telos
impl<T> Id<T> {
  bad<U>(_ value: Id<U>) -> Id<T> {
    value
  }
}
```

must still fail unless an explicit constraint proves `U == T`.

The rule is:

```text
only explicit covering substitution and actual generic evidence
    may relate distinct binders
```

---

# 16. Required RecordRow / `Self` tests

Do not depend only on high-level syntax for this.

Add direct TypeStore/substitution tests in or near:

```text
phalcom-semantic/src/types/substitution.rs
```

or the relevant type-system test module.

Create a row containing a type-level `Self`, e.g. conceptually:

```text
row:
    owner: Self
    payload: Option<Self>
```

Call:

```rust
specialize_self_type(
    store,
    declarations,
    source_target_head,
    containing_type,
)
```

and assert every nested `Self` becomes the expected source target.

Cover at least these carriers:

### Projection subject

```text
ProjectionSubject::RecordRow(row)
```

### Projection argument

```text
TypeMemberArgument::RecordRow(row)
```

### Static argument

```text
StaticArgument::RecordRow(row)
```

A table-driven test is appropriate.

Also cover:

```text
RecordRowTail::Anonymous::Typed
RecordRowTail::Projection
```

if practical.

The test should inspect the resulting canonical row contents rather than merely asserting that type formation succeeded.

---

# 17. Required covering RecordFields/row-coordinate regression

The previous patch repaired declaration→impl row materialization.

Add a regression proving it.

Use a covering implementation whose generic domain includes a row/`RecordFields` parameter and whose callable signature or generic constraint places that row inside a type-member projection.

The test must prove:

```text
declaration row binder
    ->
impl row binder
```

recursively through the projection.

This should protect the recent `materialize_type()` fix independently of the `Self` tests.

---

# 18. Required nested type regressions

Keep or add covering-body tests where impl parameters occur recursively inside:

```text
Option<Box<T>>
(T) -> Box<T>
Tuple<T, Box<T>>
record field types
type-member projections
associated projections
type packs where supported
```

The goal is to prove the coordinate transformation is recursive, not top-level-only.

---

# 19. Class-side `Self` test must protect lowercase `self`

The current separation of receiver and instance `Self` should get a direct regression.

For a class-side covering impl, test both concepts if source semantics expose them:

```telos
@class
someMethod() -> Self {
    ...
}
```

The return/type annotation must specialize to the source target instance type.

At the same time, any valid use/inspection of lowercase class-side:

```telos
self
```

must retain class-object receiver semantics.

This prevents a future "simplification" from merging:

```text
self_type_override
```

and:

```text
self_instance_type_override
```

again.

---

# 20. Public canonicalization regressions

Retain and strengthen the existing `impls::signatures` tests.

For a permutation:

```telos
impl<A, B> Pair<B, A>
```

assert the published callable signature contains declaration-owned coordinates only.

Add a recursive helper if one does not already exist:

```rust
assert_no_impl_owned_parameters(...)
```

It should walk:

```text
parameters
return
callable generic constraints
TraitRef arguments
associated projections
type-member projections
records / rows
type packs
callable types
nested generic applications
```

This should protect the publication boundary independently of whether the body happens to compile.

---

# 21. Round-trip property test

This boundary is well suited to a property-style regression.

For a covering substitution `C`:

```text
C:
    ImplParam -> DeclarationParam
```

and inverse:

```text
C^-1:
    DeclarationParam -> ImplParam
```

the invariant is:

```text
body_view(canonicalize(source_type))
    == source_type
```

for source types built from impl-owned parameters.

Generate or table-drive:

```text
1 parameter identity
2 parameter swap
3 parameter rotation
4 parameter arbitrary permutation
```

through nested structures such as:

```text
T
Box<T>
Pair<T,U>
Option<Pair<U,T>>
(T) -> U
(T, Box<U>) -> Pair<U,T>
```

If the project's property-testing infrastructure is not appropriate here, a deterministic permutation table is sufficient.

Do not introduce a new external testing framework solely for this patch.

---

# 22. Conditional/specialized impl non-regression

The patch primarily concerns covering impls.

Do not break existing conditional impl behavior such as:

```telos
impl<T> Box<T>
  where T <: Number
{
  ...
}
```

or specialized targets such as:

```telos
impl Box<Int> {
  ...
}
```

Run and, if needed, add targeted regression coverage demonstrating that:

```text
conditional InherentImplDomain
```

still owns its existing applicability semantics.

Do not route conditional impls through the covering inverse map unless they are semantically covering.

---

# 23. Exact-case / enum non-regression

If `InherentBodyContext` is also attached to accepted exact-enum-case members, verify that the new `Self` specialization does not regress exact-case semantics.

At minimum run existing exact-case/enum inherent-impl suites.

Do not reinterpret exact-case `Self` without repository evidence that its semantics require a change.

---

# 24. Inferred return handling

The current helper already specializes:

```rust
signature.inferred_return
```

through:

```rust
map_type(...)
```

Preserve this.

However, tests should ensure that inferred body results do not become a second signature authority.

The rule remains:

```text
declared canonical contract
    constrains body

body-derived/inferred result
    must not become its own premise
```

Do not introduce a fixpoint or second result owner to solve this incident.

---

# 25. Query/incremental requirements

The final implementation must preserve these dependency laws:

```text
CallableBody
    depends on
CallableDefinition
```

and:

```text
CallableDefinition
    fingerprints
InherentBodyContext
```

where `InherentBodyContext` fingerprint includes semantically relevant:

```text
source_target_head
generic signature
covering mapping
```

If the implementation changes the context representation, update the fingerprint correspondingly.

Every new semantic input to body checking must either:

- be included in body formal input fingerprinting; or
- be reached through a recorded authoritative query dependency.

Do not add hidden semantic state that bypasses query invalidation.

---

# 26. No raw TypeId equality across cold sessions

When writing cold-vs-incremental tests, remember:

```text
TypeId
```

is store-local.

Do not compare raw IDs from independently constructed semantic stores.

Compare:

- canonical formatted structure;
- stable product fingerprints;
- declaration/parameter ownership and indices;
- semantic shapes;
- normalized products.

---

# 27. Do not widen the inference solver

This patch must not alter generic constraint solving simply to tolerate:

```text
Id<I> <: Id<D>
```

or similar mismatches.

If such a mismatch reaches the generic solver after this patch, that means the body-coordinate transformation is still wrong.

The solver should continue treating distinct rigid parameter identities as distinct.

Do not weaken it.

---

# 28. Do not publish impl-owned parameters

No fix may cause:

```text
TypeParameterOwner::Impl
```

to leak into public unconditional signatures.

That includes callable-local constraints after publication.

If a body-local signature correctly contains an impl-owned binder, that is source-local and acceptable.

The published canonical contract must not.

---

# 29. Do not re-lower the source signature as the fix

The source syntax should not be independently re-lowered into a competing body signature.

The canonical callable signature is authoritative.

Correct flow:

```text
canonical signature
+
InherentBodyContext
=
body-local view
```

Incorrect flow:

```text
reparse / re-lower body declaration syntax
=
second independent signature
```

The latter risks drift and violates existing architecture.

---

# 30. Do not solve by source spelling

Do not introduce code equivalent to:

```rust
if declaration_param.name == impl_param.name {
    ...
}
```

Generic identity is semantic, not textual.

Tests with:

```telos
impl<A, B> Pair<B, A>
```

must remain first-class certification specifically to prevent this mistake.

---

# 31. Do not change `Self` language semantics casually

Do not redefine:

```text
Self
```

into an unspecialized nominal constructor.

The established semantic model is owner-relative proper type formation with explicit `SelfRole`.

The current incident is about selecting the correct body-relative instance type.

If `Self<T>` remains accepted somewhere, fix the kind/type-formation defect independently instead of changing the language model.

---

# 32. Suggested implementation sequence

Perform the work in this order.

## Step 1 — baseline verification

Before editing, run the current focused suites and record the baseline.

At minimum:

```text
impls::bodies
impls::signatures
relevant substitution/type-instantiation tests
relevant incremental tests
```

Confirm the existing `Id<Tag>`, swapped generic, and bare-`Self` regressions currently pass.

---

## Step 2 — centralize `Self` specialization of generic constraints

Inspect:

```text
phalcom-semantic/src/checker/context.rs
CheckingContext::specialize_dispatch_signature
```

and:

```text
phalcom-semantic/src/impls.rs
specialize_signature_instance_self
```

Extract/reuse the semantic logic required to specialize:

```text
TypeTerm
GenericConstraint
TraitRef arguments
```

Avoid duplicated transformation rules.

---

## Step 3 — make `specialize_signature_instance_self()` signature-complete

It must specialize:

```text
parameters
declared_return
inferred_return
callable-local generic constraints
```

while preserving callable-owned binder identities.

---

## Step 4 — make `specialize_self_type()` row-complete

Implement recursive row specialization and wire it into every RecordRow carrier.

Do not disturb parameter IDs or canonical row invariants.

---

## Step 5 — add direct semantic-product tests

Before relying on source-level tests, prove the transformations themselves.

Test:

```text
generic constraint Self specialization
RecordRow nested Self specialization
covering row materialization
```

at the lowest reasonable semantic layer.

---

## Step 6 — add hostile source-level tests

Add:

```text
method generic where U == Self
method-generic shadowing
negative unrelated binder mismatch
nested generic shape
class-side Self
```

---

## Step 7 — strengthen incremental certification

Add the context-only permutation edit:

```text
Pair<A,B> -> Pair<B,A>
```

with unchanged member body and public signature.

Assert query/product behavior, not just diagnostics.

---

## Step 8 — protect provenance

Add an exact `ImplId + source_member_index` body-selection regression.

---

## Step 9 — verify `Self<T>` behavior

Establish current behavior.

If accepted while `Self` is still kind `Type`, add/fix the negative type-formation regression.

Do not mix this with covering inference semantics.

---

## Step 10 — run broader non-regression suites

At minimum run the relevant semantic areas for:

```text
inherent impls
generic application/inference
type substitution/materialization
RecordRow/type members
incremental queries
conditional impls
associated projections
enum/exact-case impls
```

Run appropriate core/runtime inherent-impl coverage as well if available.

Finally:

```text
git diff --check
```

and targeted formatting only on files modified by this patch.

Do not reformat unrelated repository files.

---

# 33. Minimum acceptance matrix

| Capability | Required result |
|---|---|
| `Id<Tag>` explicit return | PASS |
| covering class-side `-> Self` | PASS |
| swapped `Pair<B,A>` params/result | PASS |
| callable generic `where U == Self` | PASS |
| callable-generic `Self` evidence | source target specialized |
| callable-local binder shadowing | identities remain distinct |
| wrong `Id<U>` returned as `Id<T>` | correctly rejected |
| nested `Option<Box<T>>` | PASS |
| callable `(T) -> Box<T>` | PASS |
| row containing `Self` | fully specialized |
| row projection subject containing `Self` | fully specialized |
| row projection argument containing `Self` | fully specialized |
| static row argument containing `Self` | fully specialized |
| covering RecordFields mapping | declaration→impl row coordinates correct |
| public signature | no impl-owned target parameters |
| context-only impl permutation edit | body invalidated/revalidated |
| incremental vs cold | semantic equivalence |
| exact body provenance | `ImplId + source_member_index` |
| conditional impls | no regression |
| `Self<T>` | rejected unless language design explicitly changed |

---

# 34. Final semantic laws to certify

The incident can be closed only when all of these hold.

## Law 1 — publication

```text
source impl contract
    -- covering impl_to_decl -->
canonical declaration contract
```

---

## Law 2 — body coordinate restoration

```text
canonical declaration contract
    -- covering decl_to_impl -->
impl-relative body contract
```

---

## Law 3 — source-relative `Self`

```text
type-level Self in inherent impl body
    =
source impl target head
```

Example:

```text
impl<T> Id<T>
    => Self == Id<I_T>
```

---

## Law 4 — receiver separation

```text
SelfRole::InstanceType
    !=
SelfRole::ReceiverValue
```

Especially on class side.

---

## Law 5 — full signature closure

Body specialization applies consistently to:

```text
parameters
return
inferred return
callable-local generic constraints
trait-ref arguments
nested canonical types
rows
projections
packs
```

where applicable.

---

## Law 6 — binder identity

```text
same spelling
    does not imply
same TypeParameterId
```

---

## Law 7 — public purity

```text
canonical unconditional surface
    contains no free impl-owned target binders
```

---

## Law 8 — round trip

For covering substitution `C`:

```text
body_view(C(source_contract))
    ≡
source_contract
```

after canonical normalization.

---

## Law 9 — exact source provenance

```text
accepted callable body
    =
body identified by
(ImplId, source_member_index)
```

not selector scanning.

---

## Law 10 — incremental equivalence

For every relevant edit:

```text
incremental final semantic product
    ==
cold final semantic product
```

under stable semantic comparison.

---

# 35. Definition of done

Do **not** close the incident merely because the original:

```telos
Id<Tag>(...)
```

fixture compiles.

The incident is closed only when:

1. callable-local generic constraints are source-`Self` specialized;
2. `specialize_self_type()` is structurally complete through RecordRow carriers;
3. covering parameter and row transformations remain correct recursively;
4. public signatures remain declaration-canonical;
5. lowercase receiver semantics remain separate from type-level `Self`;
6. unrelated binders still conflict correctly;
7. exact accepted source provenance is regression-tested;
8. context-only changes invalidate/revalidate body analysis correctly;
9. cold and incremental semantic products agree;
10. relevant focused and non-regression suites pass.

The existing architecture is now fundamentally correct. The remaining work should be a **narrow completion and certification patch**, not a redesign of covering impls, generic inference, `Self`, or callable identity.