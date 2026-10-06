# Revised audit of the C4.1 covering-impl correction

**Current repository head audited:** `ee7e8ee725f90d71ba4a96f907f27339abd989a0`  
**Earlier report target:** `24c25d5c244a9bff6478d729511295c1774060ab`

## Executive verdict

My earlier report was **too conservative in several places**. It correctly identified one real class-side receiver wiring bug at `24c25d5`, but it under-credited a number of mechanisms that were already genuinely implemented. In particular, the inverse covering coordinate system, recursive materialization, recursive `Self` specialization, generic-constraint transformation, row handling, binder preservation, and `CallableBody → CallableDefinition` dependency were actual implementations—not merely partial scaffolding waiting for future completion.

The subsequent `ee7e8ee` commit then fixes the genuine defect I identified: class-side lowercase `self` is now kept separate from source-relative type-level `Self`, including the conditional-impl path, and the regression tests now inspect exact semantic products rather than relying only on “no diagnostics.”

So the revised conclusion is:

| Scope | Revised verdict |
|---|---|
| Covering impl declaration ↔ source-body coordinate conversion | **Implemented correctly** |
| Original `Id<Tag>` body-coordinate failure | **Fixed** |
| Bare `Self` in generic covering impl bodies | **Implemented** |
| Nested type/row/pack/projection materialization | **Implemented recursively** |
| Callable-generic constraints involving `Self` | **Implemented** |
| Declaration/impl/callable binder separation | **Implemented and tested** |
| Class-side runtime `self` vs type-level `Self` | **Fixed at current head** |
| Conditional class-side runtime receiver | **Fixed at current head** |
| Exact body provenance | **Implemented and tested** |
| Incremental `CallableBody → CallableDefinition` dependency | **Implemented; now directly asserted** |
| Telogue package consuming the new semantics | **Not migrated; currently inconsistent** |
| C4.1 lifecycle documentation | **Mostly correct, with a stale test-count record** |

The **compiler-side C4.1 implementation can now be considered substantially closed**. The important remaining issue is repository-level integration: the Telogue package still contains the old `Self<...>` workaround that the compiler now intentionally rejects.

---

# 1. What my earlier analysis under-credited

## 1.1 The inverse body-coordinate mechanism is genuinely implemented

This is the most important correction to my previous framing.

`InherentBodyContext` is not simply storing enough metadata for a future solution. It owns the actual source-body semantic projection:

```rust
pub struct InherentBodyContext {
    pub generic_signature: Option<GenericSignature>,
    pub covering: Option<CoveringImplSubstitution>,
    pub source_target_head: TypeId,
}
```

and:

```rust
pub fn signature_for_body(
    &self,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    canonical_signature: CallableSemanticSignature,
) -> Result<CallableSemanticSignature, TypeMaterializationError> {
    let signature = if let Some(covering) = &self.covering {
        let instantiation = covering.to_body_instantiation(store)?;
        apply_covering_instantiation_to_signature(
            store,
            &instantiation,
            canonical_signature,
        )?
    } else {
        canonical_signature
    };

    Ok(specialize_signature_instance_self(
        store,
        declarations,
        self.source_target_head,
        signature,
    ))
}
```

That realizes the architecture we wanted:

```text
source signature
      │
      │ impl_to_decl
      ▼
canonical public signature
      │
      │ decl_to_impl
      ▼
body-local source signature
```

There is still only **one authoritative published contract**. The source body receives a derived view of that contract. That is exactly right.

My previous report did recognize this, but language such as “body coordinate restoration: PASS” alongside open “round-trip coverage” could make the implementation sound less complete than it is. The implementation itself is real and structurally correct.

---

# 2. The inverse mapping is domain-aware, including RecordRow

`CoveringImplSubstitution` now has genuinely distinct forward and inverse instantiations.

Forward publication:

```rust
pub fn to_instantiation(
    &self,
    store: &mut TypeStore,
) -> Result<GenericInstantiation, TypeMaterializationError>
```

maps:

```text
Impl parameter -> Declaration parameter
```

while:

```rust
pub fn to_body_instantiation(
    &self,
    store: &mut TypeStore,
) -> Result<GenericInstantiation, TypeMaterializationError>
```

maps:

```text
Declaration parameter -> Impl parameter
```

Critically, row binders are not smuggled through `TypeId`:

```rust
if store.type_parameter(impl_parameter).kind == KindId::RECORD_ROW {
    let row = store.record_row_checked(
        Vec::new(),
        RecordRowTail::Parameter(impl_parameter),
    )?;
    instantiation.bind_row(declaration_parameter, row);
}
```

That is the correct domain-aware implementation.

This means my earlier request for “RecordFields/row support” should now be characterized as **implemented**, not pending.

---

# 3. Nested type translation is genuinely recursive

This is another place where I would improve my earlier wording.

I previously listed missing dedicated source-level fixtures for forms such as:

```text
Option<Box<T>>
(T) -> Box<T>
nested projections
packs
rows
```

Those would still be useful tests, but their absence does **not** mean the implementation lacks nested translation.

`apply_covering_instantiation_to_signature()` delegates canonical types to:

```rust
materialize_type(...)
```

rather than performing a shallow parameter replacement.

Likewise `specialize_self_type()` recursively traverses the canonical type graph, including:

```text
Applied
ExactCase
AssociatedProjection
TypeMemberProjection
Union
Tuple
Record
Callable
Family
TypePack
LabeledTypePack
Pack
PackFromPacks
TupleFromPacks
CallableFromPacks
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
StaticData
```

with explicit RecordRow recursion for row-bearing carriers.

So the correct classification is:

> **Nested specialization is implemented. More source-level matrix tests would be certification hardening, not feature completion.**

That distinction matters.

---

# 4. Generic-constraint translation is genuinely implemented

The current design has a shared semantic operation:

```rust
specialize_self_generic_constraint(...)
```

backed by:

```rust
map_generic_constraint(...)
```

It handles all current generic constraint forms:

```text
Subtype
Equivalent
Conforms
```

and importantly transforms both the subject and `TraitRef.arguments`.

`apply_covering_instantiation_to_signature()` likewise materializes canonical operands inside those constraints, including trait arguments.

So callable generics such as:

```telos
retain<T>(_ value: T) -> T
where T <: Self
```

or:

```telos
identity<U>(_ value: U) -> Self
where U == Self
```

are not accidental successes. There is now explicit machinery for them.

The test:

```text
test_covering_body_specializes_callable_generic_self_constraint
```

even inspects the resulting semantic constraint and verifies that:

```text
callable-owned T remains callable-owned T
Self becomes context.source_target_head
```

That is strong evidence, not merely a smoke test.

---

# 5. Binder preservation is genuinely implemented

The shadowing test is particularly good:

```telos
data Box<T>(_ value: T)

impl<T> Box<T> {
  identity<T>(_ value: T) -> T { value }
}
```

It verifies three distinct identities:

```text
Declaration(Box)::T
Impl(the_impl)::T
Callable(identity)::T
```

and explicitly proves that the body parameter/result retain the callable-owned binder.

That means the implementation is doing semantic owner-based substitution, not textual substitution.

The negative test:

```telos
impl<T> Id<T> {
  bad<U>(_ value: Id<U>) -> Id<T> {
    value
  }
}
```

then proves unrelated binders are still rejected.

This portion is properly implemented and properly tested.

---

# 6. RecordRow closure is implemented, not merely partially covered

The shared:

```rust
specialize_self_row(...)
```

recurses through:

```text
field types
anonymous unconstrained key type
anonymous typed key type
anonymous typed value type
projection tails
stable parameter tails
closed tails
```

and canonicalizes again through `record_row_checked`.

That helper is used from:

```text
Record
Applied static row arguments
StaticData row arguments
TypeMemberProjection row subjects
TypeMemberProjection row arguments
```

There is also a direct covering-materialization regression for row projection carriers.

I would therefore upgrade my previous description from “substantially complete” to:

> **The identified RecordRow specialization problem is implemented and closed.**

Future row variants would need corresponding traversal, but that is normal exhaustiveness maintenance, not an existing gap.

---

# 7. Public contract ownership is genuinely preserved

The publication path still canonicalizes covering members using the forward instantiation:

```rust
let final_sig = if let Some(ref instantiation) = covering_instantiation {
    apply_covering_instantiation_to_signature(
        ctx.store,
        instantiation,
        raw_sig,
    )?
} else {
    raw_sig
};
```

and the body path derives its inverse view later.

This means:

```text
EffectiveCallableDefinition.signature
```

remains declaration-coordinate authority.

The implementation also has a recursive `type_contains_impl_param()` walker used by debug assertions for published parameter/result types.

Its recursive collector covers substantially more than just top-level applications: rows, packs, callables, projections, associated projections, static arguments, etc.

A stronger reusable assertion over generic constraints as well would be nice, but that is **hardening**, not evidence that impl-owned parameters are currently leaking.

---

# 8. My class-side `self` finding was real — and is now fixed

This part of my earlier audit should **not** be retracted.

At `24c25d5`, both fields were populated with `source_target_head`:

```rust
self_type_override: Some(source_target_head),
self_instance_type_override: Some(source_target_head),
```

and lowercase `Expr::SelfVar` gives `self_type_override` precedence.

That really did make a class-side receiver look like an instance receiver.

The new `ee7e8ee` implementation fixes this cleanly by centralizing the distinction:

```rust
pub fn runtime_receiver_override(
    &self,
    side: DispatchSide,
) -> Option<TypeId> {
    (side == DispatchSide::Instance)
        .then_some(self.source_target_head)
}

pub fn instance_self_override(&self) -> TypeId {
    self.source_target_head
}
```

Then both normal analysis and downstream revalidation use:

```rust
self_type_override:
    context.runtime_receiver_override(callable.side),

self_instance_type_override:
    Some(context.instance_self_override()),
```

That is the exact architecture I recommended.

The conditional inherent path is also repaired:

```rust
if callable.side == DispatchSide::Instance
    && let Some(domain) = &impl_domain
{
    ctx.self_override_type = Some(domain.head_type);
}
```

So the current invariant is finally:

```text
Instance-side body:
    self = Id<I_T>
    Self = Id<I_T>

Class-side body:
    self = ClassObject(Id)
    Self = Id<I_T>
```

**Current verdict: fixed.**

---

# 9. The receiver regression is now a real semantic regression

My criticism of the old test was also valid: it only asserted that the snapshot had no errors, while unresolved ordinary dispatch can degrade to `Unknown(DynamicMessageSend)` without a diagnostic.

The replacement test is materially stronger.

It now locates the actual `self` expression and checks:

```rust
TypeData::ClassObject {
    declaration: Counter
}
```

and independently verifies that:

```rust
self.direct()
```

selected the exact class-side `Counter.direct` callable.

The generic covering test similarly verifies:

```text
lowercase self -> ClassObject(Id)
```

while reconstructing the body signature for:

```telos
@class
make() -> Self
```

and asserting its return type is exactly:

```rust
context.source_target_head
```

The conditional class-side test likewise checks the class-object receiver and exact class-side dispatch.

These are now good discriminating tests.

---

# 10. The incremental dependency existed before; the missing piece was direct certification

This is another place where my earlier report could be read too harshly.

At `24c25d5`, the body query already recorded:

```text
CallableBody -> CallableDefinition
```

when an effective definition product existed.

My concern was specifically that the **context permutation test did not isolate that edge**, because the impl-header fingerprint independently caused recomputation.

That was a **test-certification problem**, not an absent dependency implementation.

The current follow-up now directly asserts:

```rust
session
    .db()
    .index()
    .dependencies_of(&body_key)
    .is_some_and(|edges| {
        edges.iter().any(
            |edge| edge.dependency == definition_key
        )
    })
```

So the exact dependency is now permanently guarded.

The existing permutation/cold-parity tests remain useful as the higher-level end-to-end layer.

**Current verdict: implemented and adequately certified for the C4.1 concern.**

---

# 11. The original `Id<Tag>` failure is now genuinely fixed

This is worth stating plainly.

There is a direct source regression:

```telos
data Token()

data Id<Tag>(
  _ token: Token,
)

impl<Tag> Id<Tag> {
  @class
  fresh() -> Id<Tag> {
    Id<Tag>(Token())
  }
}
```

and a stronger permutation regression:

```telos
impl<A, B> Pair<B, A> {
  @class
  duplicate(_ b: B, _ a: A) -> Pair<B, A> {
    Pair<B, A>(b, a)
  }
}
```

These directly exercise the defect that originally produced:

```text
generic constraints conflict
generic argument constraints conflict
generic constraints have no common solution
```

The implementation no longer asks inference to reconcile declaration-owned and impl-owned rigid binders. It translates the canonical contract back into body coordinates first.

That is the semantic fix we wanted.

---

# 12. Bare `Self` is now the correct ergonomic spelling

The repository also directly tests:

```telos
impl<Tag> Id<Tag> {
  @class
  fresh() -> Self {
    Id<Tag>(Token())
  }
}
```

This succeeds.

At the same time:

```telos
Self<T>
```

is explicitly rejected with:

```text
ApplicationNotConstructor
```

That is coherent with the established model:

```text
Self :: Type
```

rather than:

```text
Self :: Type -> Type
```

So both:

```telos
-> Id<Tag>
```

and:

```telos
-> Self
```

are now legitimate source spellings in the covering impl.

That capability is genuinely implemented.

---

# 13. New repository-level integration finding: Telogue still uses the obsolete workaround

This is the significant current issue my previous audit missed because I focused on `phalcom-semantic`.

The package that originally exposed this compiler problem has **not been migrated to the repaired semantics**.

Current head still contains:

`packages/telogue/src/identity-token.ph`

```telos
impl<Tag> Id<Tag> {
  @class
  fresh -> Self<Tag> {
    Id<Tag>(token: IdentityToken.fresh)
  }
}
```

and the problem is broader. A repository search finds live package source using invalid `Self<...>` in:

```text
packages/telogue/src/identity-token.ph
    fresh -> Self<Tag>

packages/telogue/src/variable.ph
    named(...) -> Self<T>

packages/telogue/src/predicate.ph
    named(...) -> Self<*Ts>

packages/telogue/src/base.ph
    of(...) -> Self<T>
```

These are not isolated dead examples.

`packages/telogue/src/id.ph` contains:

```telos
from .identity_token import Id
```

and `main.ph` imports `predicate`, `variable`, and `base`, so these files lie on the actual project dependency graph.

This means the repository currently has the somewhat ironic state:

```text
semantic regression:
    Self<T> must be rejected
        PASS

real integration package:
    still uses Self<T>
        stale
```

This does **not** mean C4.1 failed.

It means:

> **C4.1 is implemented, but its real consumer has not yet been migrated.**

For example, these can now simply become:

```telos
impl<Tag> Id<Tag> {
  @class
  fresh -> Self {
    Id<Tag>(token: IdentityToken.fresh)
  }
}
```

and similarly:

```telos
impl<T> Variable<T> {
  @class
  named(name: Symbol) -> Self {
    ...
  }
}
```

For `Predicate<*Ts>`, bare `Self` is precisely useful because the source target already retains the complete pack specialization.

---

# 14. This exposes a gap in the verification strategy

The C4.1 focused semantic suite is good, but it does not compile the real package that motivated the incident.

That is why the compiler can simultaneously have:

```text
test_covering_body_rejects_self_type_application
    PASS
```

while repository package source still contains four invalid `Self<...>` annotations.

If Telogue/Hypothesis is intended to act as a realistic pressure test for the language, there should be a project-level gate that loads or typechecks the entire package graph.

That would have immediately caught this stale migration.

This is a **repository integration/test coverage issue**, not a type-system correctness issue.

---

# 15. One documentation inconsistency remains

The current C4.1 walkthrough says:

```text
impls::bodies — 18 passed
```

but current `phalcom-semantic/tests/semantic/impls/bodies.rs` contains **19 `#[test]` functions**.

The progression explains the likely mismatch:

```text
24c25d5:
    17 body tests recorded

ee7e8ee:
    + generic covering class-side receiver test
    + conditional class-side receiver test
```

which yields 19 current tests.

So the walkthrough's `18 passed` record is stale or was recorded before the final test set was complete.

That is minor, but lifecycle records should reflect the final commit accurately.

---

# 16. What I would now remove from the “open issue” category

Several things from my earlier report should **not** remain on a closure-blocking list.

A systematic permutation property test would still be useful, but the generic coordinate bijection is already implemented through `impl_to_decl`/`decl_to_impl` and tested with a nontrivial swap. It is an enhancement, not unfinished semantics.

Likewise, a larger nested source matrix would increase confidence, but nested transformation is already implemented through recursive `materialize_type()` and exhaustive `specialize_self_type()` traversal. It should not be described as an unimplemented capability.

A recursive “no impl-owned parameter anywhere” test helper would also be good hardening, but publication already uses recursive materialization and has recursive impl-parameter detection for published parameter/result types. Again: test improvement, not proof of a current leak.

And the context-permutation test still has overlapping invalidation through the impl-header fingerprint, but that no longer matters for certifying the dependency edge because the test suite now directly inspects the DB dependency graph.

---

# 17. One API ergonomics improvement is worth considering

The names:

```rust
self_type_override
self_instance_type_override
```

made this defect unusually easy to introduce.

Their actual meanings are closer to:

```text
runtime receiver-value override
type-level instance-Self override
```

An eventual rename such as:

```rust
receiver_value_override
instance_self_override
```

would make the distinction much harder to misuse.

I would treat this as cleanup, not required corrective work. The new `InherentBodyContext` helpers already encapsulate the dangerous distinction at the main call sites.

---

# Revised closure matrix

| Invariant | Current status |
|---|---|
| Stable declaration/impl binder identities | **PASS** |
| Forward `impl → declaration` publication | **PASS** |
| Inverse `declaration → impl` body view | **PASS** |
| Domain-aware RecordRow inverse mapping | **PASS** |
| Explicit `Id<Tag>` construction in covering body | **PASS** |
| Nontrivial permutation `Pair<B,A>` | **PASS** |
| Bare `Self` in generic covering body | **PASS** |
| Callable-generic `Self` constraints | **PASS** |
| Trait-reference argument `Self` specialization | **PASS** |
| Callable-local shadowing | **PASS** |
| Unrelated binder rejection | **PASS** |
| `Self<T>` rejection | **PASS** |
| Class-side runtime `self` class-object semantics | **PASS at `ee7e8ee`** |
| Class-side type-level `Self` source specialization | **PASS** |
| Conditional class-side runtime receiver | **PASS at `ee7e8ee`** |
| Exact `(ImplId, source_member_index)` provenance | **PASS** |
| `CallableBody → CallableDefinition` dependency | **PASS and directly asserted** |
| Incremental/cold permutation parity | **PASS by recorded focused tests** |
| Telogue migration away from `Self<...>` | **FAIL / not performed** |
| Final walkthrough test count | **STALE** |

# Final revised ruling

My earlier overall verdict of **“not closed”** was appropriate for the exact `24c25d5` head because the class-side runtime receiver wiring defect was real. It is **no longer the correct verdict for the semantic implementation at current head**.

At `ee7e8ee`, I find no remaining C4.1 semantic correctness defect from the issue we were auditing. The implementation now has the architecture originally intended:

```text
source impl contract
        │
        │ canonicalize
        ▼
declaration-owned public contract
        │
        │ inverse materialization
        ▼
impl-owned body contract
        │
        ├── type-level Self = source target instance
        │
        └── runtime self = side-correct receiver
```

I would therefore classify **LANG007.C4.1 semantic closure as implemented and acceptable**.

The repository is **not yet integration-clean**, however. The immediate next action should not be another redesign of the semantic machinery. It should be to migrate the four Telogue `Self<...>` annotations to bare `Self` (or their explicit applied nominal types), then run the entire Telogue project as a compiler/semantic integration fixture. That is the best remaining proof that the original real-world failure—not merely its synthetic regression—has actually been closed end to end.