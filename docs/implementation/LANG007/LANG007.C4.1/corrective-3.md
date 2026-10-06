# Audit report — covering inherent-impl body-coordinate closure

**Repository head audited:** `24c25d5c244a9bff6478d729511295c1774060ab`

**Baseline compared:** `03df529c4f543e42c908d9b617938b075ed35a65`

## Verdict

**Not fully closed.**

The implementation successfully fixes the two major semantic-completeness defects from the previous audit:

- callable-local generic constraints now receive source-relative `Self` specialization;
- `Self` specialization is now recursively complete through the relevant `RecordRow` carriers.

The implementation also substantially improves the regression suite, source-body provenance testing, binder-shadowing coverage, `Self<T>` rejection, and incremental/cold comparison.

However, I found **one P1 semantic correctness defect in the final repository state**:

> **class-side inherent-impl bodies currently conflate runtime lowercase `self` with type-level `Self`.**

This directly contradicts one of the central architectural invariants of the corrective work. The new test intended to protect that distinction is a false positive because unresolved ordinary dispatch can remain diagnostic-free.

There is also a **P2 incremental-certification gap**: the new “context-only” test does not actually isolate the `CallableBody → CallableDefinition → InherentBodyContext` dependency because the new impl-header source fingerprint independently invalidates the member.

Accordingly, I would change the current C4.1 status from:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

to approximately:

```text
IMPLEMENTED — CORRECTIVE FOLLOW-UP REQUIRED
```

until the runtime-receiver defect is fixed and directly regression-tested.

---

# 1. Summary of findings

| ID | Severity | Status | Finding |
|---|---|---|---|
| AUD-C4.1-01 | **P1** | **OPEN** | Class-side lowercase `self` is incorrectly overwritten with `source_target_head` |
| AUD-C4.1-02 | **P1 adjacency** | **OPEN** | `impl_domain.head_type` has the same receiver/type-`Self` conflation for class-side conditional inherent impls |
| AUD-C4.1-03 | **P1 test defect** | **OPEN** | `test_class_side_type_self_and_lowercase_receiver_remain_distinct` does not actually prove receiver dispatch and can pass with unresolved dispatch |
| AUD-C4.1-04 | **P2 certification** | **OPEN** | Context-only incremental test is independently dirtied by `impl_header_fingerprint`, so it does not prove the `CallableDefinition` dependency |
| AUD-C4.1-05 | P3 coverage | OPEN | Systematic covering round-trip / deeper nested-shape certification requested by the handoff is still missing |
| — | — | **FIXED** | Callable-local generic constraint `Self` specialization |
| — | — | **FIXED** | Trait-ref argument specialization inside generic constraints |
| — | — | **FIXED** | RecordRow recursive `Self` specialization |
| — | — | **FIXED** | RecordRow covering materialization in type-member projections |
| — | — | **FIXED** | Callable-local binder shadowing |
| — | — | **FIXED** | Unrelated binder negative case |
| — | — | **FIXED** | `Self<T>` rejection |
| — | — | **FIXED** | Exact `(ImplId, source_member_index)` source-body provenance |
| — | — | **PRESERVED** | Declaration-canonical public signatures |

---

# 2. P1 — class-side runtime `self` is currently wrong

This is the blocking defect.

## 2.1 The architecture correctly introduced two separate concepts

`CallableBodyRequest` now exposes:

```rust
pub self_type_override: Option<TypeId>,
pub self_instance_type_override: Option<TypeId>,
```

with the second explicitly documented as:

> Source-relative type for type-level `Self`; it does not alter the runtime lowercase `self` value.

That is exactly the correct architectural separation.

The intended semantics are:

```text
self_instance_type_override
    -> type-level `Self`
    -> Id<I_Tag>, Pair<I_B,I_A>, ...

self_type_override
    -> runtime lowercase `self`
    -> actual receiver value
```

For an instance member:

```text
runtime self = Id<I_Tag>
```

is appropriate.

For a class-side member:

```text
runtime self = class object Id
```

while:

```text
type-level Self = Id<I_Tag>
```

These are deliberately not the same thing.

---

# 3. The session currently assigns both overrides to the instance target

In the direct inherent-body path in `session.rs`, current head does:

```rust
self_type_override:
    body_context.as_ref().map(|context| context.source_target_head),

self_instance_type_override:
    body_context.as_ref().map(|context| context.source_target_head),
```

The downstream revalidation path does exactly the same thing.

This collapses:

```text
runtime receiver
```

and:

```text
type-level source-relative instance Self
```

back into one type.

That defeats the separation introduced in `CallableBodyRequest`.

---

# 4. Why this definitely changes lowercase `self`

`checker/body.rs` consumes the request as:

```rust
if let Some(self_type) = self_type_override {
    ctx.self_override_type = Some(self_type);
}

ctx.self_instance_type_override = self_instance_type_override;
```

Then `checker/expression.rs` handles lowercase `self` with:

```rust
Expr::SelfVar { range } => {
    if let Some(self_ty) = ctx.self_override_type {
        TypedExpression::established(self_ty, ...)
    } else if let Some(class_decl) = ctx.current_class.clone() {
        if ctx.current_side == DispatchSide::Class {
            ...
            info.class_object_type
        } else {
            ...
        }
    }
}
```

The override has priority.

So once:

```rust
ctx.self_override_type = Some(source_target_head)
```

is installed, the normal class-side branch is never reached.

---

# 5. `source_target_head` is an instance type, not the class object

For:

```telos
impl Counter {
    @class
    ...
}
```

the inherent target resolution obtains the ordinary type annotation:

```text
Counter
```

and publishes:

```rust
source_target_head = resolved_target.target_type
```

That is the nominal instance form.

For:

```telos
impl<T> Id<T>
```

it is the applied nominal instance form:

```text
Id<I_T>
```

It is not:

```text
ClassObject(Id)
```

Therefore current class-side body checking gets:

```text
lowercase self : Id<I_T>
```

instead of:

```text
lowercase self : ClassObject(Id)
```

---

# 6. This changes dispatch side

`dispatch_owner_for_lookup()` makes the distinction explicit:

```rust
TypeData::ClassObject { declaration }
    => DispatchSide::Class

TypeData::Nominal { declaration }
    => DispatchSide::Instance

TypeData::Applied { ... }
    => DispatchSide::Instance
```

So consider:

```telos
class Counter {
  @class
  direct() -> Int { 10 }
}

impl Counter {
  @class
  throughReceiver() -> Int {
    self.direct()
  }
}
```

Correct:

```text
self
  -> ClassObject(Counter)
  -> class-side lookup
  -> Counter.direct @class
```

Current body context:

```text
self
  -> Counter instance
  -> instance-side lookup
  -> no instance direct()
```

That is a real semantic mismatch.

---

# 7. Why the new regression nevertheless passes

The new test is:

```rust
#[test]
fn test_class_side_type_self_and_lowercase_receiver_remain_distinct() {
    ...
    let update = session.update(...);
    assert!(!update.snapshot.has_errors(), ...);
}
```

This does **not** verify the property named by the test.

For an unresolved method dispatch, `synthesize_method_call()` eventually reaches:

```rust
ResolvedDispatchResult::Missing { .. } =>
    analyze_unresolved_application(
        ...,
        UnresolvedApplicationReason::DispatchMissing
    )
```

and `analyze_unresolved_application()` maps this to:

```rust
TypeKnowledge::Unknown(
    UnknownReason::DynamicMessageSend
)
```

with an otherwise `Ready` analysis status.

It does **not necessarily emit a source diagnostic**.

Therefore:

```telos
self.direct()
```

can fail to select the class-side callable and the test can still satisfy:

```rust
!snapshot.has_errors()
```

This explains how the repository can report:

```text
impls::bodies — 17 passed
```

while the semantic invariant is still violated.

The test is testing only:

```text
"nothing emitted an error"
```

rather than:

```text
"lowercase self retained class-object semantics and selected the exact class-side method"
```

---

# 8. Required architectural fix

Do not remove the two-override design. It is correct.

Fix the call sites.

For an inherent body, conceptually:

```rust
let instance_self = context.source_target_head;

let receiver_self = match callable.side {
    DispatchSide::Instance => Some(context.source_target_head),
    DispatchSide::Class => None,
};
```

Then:

```rust
self_type_override: receiver_self,
self_instance_type_override: Some(instance_self),
```

Meaning:

### Instance side

```text
lowercase self
    = source target instance

type Self
    = source target instance
```

### Class side

```text
lowercase self
    = no override
    -> ordinary class-object receiver machinery

type Self
    = source target instance
```

That is the required invariant.

I would avoid duplicating the conditional in multiple places. Put it on `InherentBodyContext` or a closely related helper, e.g. conceptually:

```rust
impl InherentBodyContext {
    fn runtime_receiver_override(
        &self,
        side: DispatchSide,
    ) -> Option<TypeId> {
        matches!(side, DispatchSide::Instance)
            .then_some(self.source_target_head)
    }

    fn instance_self_override(&self) -> TypeId {
        self.source_target_head
    }
}
```

Then use it from both:

- normal inherent-body analysis;
- downstream body revalidation.

---

# 9. P1 adjacency — conditional inherent impls contain the same mistake

There is another path in `checker/body.rs`:

```rust
if let Some(domain) = &impl_domain {
    ctx.self_override_type = Some(domain.head_type);
}
```

This is also unconditional with respect to dispatch side.

For:

```telos
impl<T> Box<T>
  where ...
{
  @class
  ...
}
```

a class-side body can again receive the instance head as its runtime lowercase receiver.

So merely correcting `session.rs` is insufficient.

The rule should remain the same:

```text
domain.head_type
    may specialize instance-side runtime self

domain.head_type
    must not replace class-side runtime self
```

Type-level `Self` for such a body should still use the appropriate source/domain instance head.

This should be corrected in the same patch so the same defect is not left alive in the conditional inherent-impl path.

---

# 10. Required replacement test for class-side receiver semantics

The existing test should be upgraded from diagnostic-based to semantic-product-based.

For:

```telos
class Counter {
  @class
  direct() -> Int { 10 }
}

impl Counter {
  @class
  throughReceiver() -> Int {
    self.direct()
  }
}
```

assert at least:

```text
throughReceiver CallableAnalysis
    contains exact callable selection:
    Counter::direct / DispatchSide::Class
```

Preferably also identify the `self` expression and assert its knowledge is:

```rust
TypeData::ClassObject {
    declaration: Counter
}
```

At minimum the method-call expression must have:

```rust
expression.callable == Some(class_side_direct_callable)
```

and must **not** be:

```text
Unknown(DynamicMessageSend)
```

The regression must fail if class-side dispatch falls through to unresolved application.

---

# 11. Add the generic version of the receiver test

Also test the actual covering case:

```telos
data Id<T>()

impl<T> Id<T> {
  @class
  direct() -> Int { 10 }

  @class
  throughReceiver() -> Int {
    self.direct()
  }

  @class
  make() -> Self {
    Id<T>()
  }
}
```

The test should simultaneously prove:

```text
type-level Self
    == Id<I_T>
```

and:

```text
lowercase self
    == class object Id
```

This is the strongest regression for the architectural split.

---

# 12. Callable-local generic `Self` constraints are now correctly implemented

This part of the patch is good.

The new shared:

```rust
map_generic_constraint(...)
```

covers:

```text
Subtype
Equivalent
Conforms
```

and maps:

```text
canonical operand types
Self terms
trait-reference arguments
```

while preserving:

```text
Infer terms
```

That is the right primitive.

Then:

```rust
specialize_self_generic_constraint(...)
```

uses it consistently from the body view.

And `specialize_signature_instance_self()` now applies it to:

```rust
signature.generics.constraints
```

This closes the specific gap found in the preceding audit.

---

# 13. Direct `TypeTerm::SelfType` handling is also improved

The previous version of `specialize_signature_instance_self()` only acted on:

```rust
DeclaredTypeState::Known(
    TypeTerm::Canonical(...)
)
```

Current head accepts:

```rust
DeclaredTypeState::Known(term)
```

and routes it through:

```rust
specialize_self_type_term(...)
```

That correctly handles:

```text
Canonical
SelfType
Infer
```

without treating an inference term as a concrete type.

This is a real improvement.

---

# 14. Callable-generic binder shadowing is now properly tested

The added test creates:

```telos
impl<T> Box<T> {
  identity<T>(_ value: T) -> T { value }
}
```

and explicitly obtains:

```text
Declaration(Box)::T
Impl(the_impl)::T
Callable(identity)::T
```

then checks they are all different IDs despite identical spelling.

This is exactly the regression that was needed.

It protects against any future temptation to perform body-coordinate conversion by source names.

**This portion is closed.**

---

# 15. Negative unrelated-binder behavior is protected

The new test:

```telos
impl<T> Id<T> {
  bad<U>(_ value: Id<U>) -> Id<T> {
    value
  }
}
```

expects a `ReturnMismatch`.

That is important because it proves the repair has not weakened generic rigidity globally.

The architecture remains:

```text
different rigid binders
    remain different
unless
explicit semantic substitution/evidence relates them
```

**Good.**

---

# 16. `Self<T>` is now explicitly rejected

The new test:

```telos
bad(_ value: Self<T>) -> Unit
```

expects:

```text
ApplicationNotConstructor
```

This is aligned with the current language model:

```text
Self :: Type
```

not:

```text
Self :: Type -> Type
```

The added signature-diagnostic propagation in `build_inherent_impl_contribution()` also makes this failure visible even if the invalid parameter is never referenced by the body.

This is the correct outcome.

I found no reason to reinterpret `Self` as a constructor.

---

# 17. RecordRow `Self` recursion is now substantially complete

The new `specialize_self_row()` is architecturally sound.

It recursively handles:

```text
RecordRowField.ty
Anonymous(Unconstrained).key_type
Anonymous(Typed).key_type
Anonymous(Typed).value_type
Projection tail
Parameter tail
Closed tail
```

and reconstitutes through:

```rust
record_row_checked(...)
```

rather than mutating/interpreting row internals unsafely.

That preserves TypeStore canonicalization.

---

# 18. All important row carriers are now traversed

`specialize_self_type()` now recurses through RecordRows embedded as:

```text
Applied static argument
StaticData argument
TypeMemberProjection subject
TypeMemberProjection argument
Record value type
```

The tests explicitly inspect the resulting row structure rather than only asserting successful formation.

This directly closes the prior structural hole.

**I consider the RecordRow `Self` specialization defect fixed.**

---

# 19. Covering row binder materialization is also tested properly

The new materialization test constructs:

```text
declaration-owned RecordRow parameter
    D_R
```

and binds it to:

```text
impl-owned RecordRow parameter
    I_R
```

inside both:

```text
ProjectionSubject::RecordRow
```

and:

```text
TypeMemberArgument::RecordRow
```

then inspects the resulting row tail.

This is substantially stronger than a no-diagnostics test.

**Good.**

---

# 20. Generic constraint trait arguments are directly tested

The test for:

```rust
GenericConstraint::Conforms {
    subject: Self,
    trait_ref: Trait<Self>,
}
```

proves both:

```text
subject Self
```

and:

```text
TraitRef argument Self
```

are specialized.

That addresses another easy-to-miss location.

The dummy callable parameter created in that particular unit test is not doing much, but the separate source-level shadowing test provides the binder-protection coverage, so I do not consider this a correctness problem.

---

# 21. Exact source-body provenance is now properly regression-tested

The unit test around `source_body_for_callable()` is good.

It deliberately creates two impls with the same selector:

```telos
impl User {
  ping() -> Unit { 1 }
}

impl User {
  ping() -> Unit { 2 }
}
```

and constructs a definition pointing specifically at:

```text
ImplId(second impl)
source_member_index 0
```

then asserts the selected source range belongs to the second implementation.

That directly protects the desired identity:

```text
body source =
    (ImplId, source_member_index)
```

not:

```text
first selector match
```

This previous issue can now be considered closed.

---

# 22. Public signature ownership remains correct

I found no regression to the central publication model.

The code still follows:

```text
source impl signature
    |
    | impl_to_decl
    v
canonical public signature
```

and only constructs the source body view from a clone of that canonical signature.

The body `Self`/coordinate specialization does not rewrite the effective public surface in place.

Existing permutation signature tests also remain in the focused verification record.

So the most important ownership invariant remains intact.

---

# 23. P2 — the “context-only” incremental test does not isolate the dependency it claims to certify

This is a test-design problem, not currently a demonstrated semantic unsoundness.

The new test changes:

```telos
impl<A, B> Pair<A, B>
```

to:

```telos
impl<A, B> Pair<B, A>
```

while keeping:

```telos
ping() -> Unit { () }
```

unchanged.

That looks like a clean context-only test.

However, the new semantic shard logic now calculates an:

```text
impl_header_fingerprint
```

covering the impl declaration header and folds it into member signature/body source fingerprints.

Therefore changing:

```text
Pair<A,B>
```

to:

```text
Pair<B,A>
```

directly changes the member input fingerprint.

So the body can be recomputed even if this edge were accidentally absent:

```text
CallableBody
    ->
CallableDefinition
```

The test proves:

- final incremental/cold consistency;
- public contract stability;
- body refresh occurs.

It does **not** independently prove:

```text
InherentBodyContext product changed
    -> CallableDefinition changed
    -> dependent CallableBody invalidated
```

because another invalidation mechanism fires simultaneously.

---

# 24. Required direct dependency regression

Add a lower-level query/index test.

After computing the body:

```rust
let edges = db.index()
    .dependencies_of(&QueryKey::CallableBody(callable.clone()))
    .expect(...);
```

assert an edge exists to:

```rust
QueryKey::CallableDefinition(callable.clone())
```

That at least permanently protects the explicit dependency.

For stronger certification, create a DB-level test where:

```text
body source/input fingerprint remains constant
```

but:

```text
CallableDefinition product fingerprint changes
```

because only its `InherentBodyContext` changes.

Then validate that:

```text
CallableBody
```

does not remain valid against the stale observed dependency product.

That would actually certify the intended invalidation law.

---

# 25. The new impl-header fingerprint is conservative, not inherently wrong

I do **not** classify the impl-header fingerprint itself as a correctness defect.

Making each impl member sensitive to:

```text
impl generic binders
target head
where clause
```

is conservative and understandable because those are lexical semantic inputs of the body.

It can cause extra recomputation, but that is preferable to stale semantics.

The problem is only that it prevents the current test from isolating another dependency mechanism.

---

# 26. Callable-body input fingerprint simplification deserves caution, but I found no concrete unsoundness

The last corrective commit substantially simplifies:

```rust
callable_body_input_fingerprint_with_formal_inputs(...)
```

by removing direct dependence on broad:

```text
workspace source resolution fingerprint
linked component product
declared signature fingerprint
aggregate conformance view fingerprint
```

and instead relying more heavily on exact recorded query dependencies.

Architecturally, this is desirable if the dependency graph is complete.

I inspected the surrounding checker infrastructure:

- `TrackingTypeResolver` records name and declaration reads;
- `TrackingTypeHierarchy` records hierarchy reads;
- checker helpers record declaration/surface/callable/conformance dependencies;
- body reuse refreshes formal semantic dependencies before `validate_reuse`;
- callable bodies record `CallableDefinition`;
- body AST and owner generic signature remain direct inputs.

I did **not** find a concrete stale-reuse path from this change during this audit.

So I would not revert this simplification based on the evidence currently available.

But because it broadens the scope beyond the original covering-impl bug, its incremental tests should remain part of the mandatory gate.

---

# 27. The documented test results are useful, but one of them gives false confidence

The repository walkthrough records successful focused runs including:

```text
cargo check -p phalcom-semantic
impls::bodies          17 passed
impls::signatures       4 passed
record_rows_field_keys 14 passed
impls::queries         60 passed
git diff --check
```

Those results are valuable.

However, because the class-side receiver test asserts only absence of diagnostics, its green result does not refute AUD-C4.1-01.

The source control flow explains exactly why it can pass.

So the test result should not be treated as evidence that runtime `self` separation is correct.

---

# 28. Coverage requested previously but still not fully realized

Several requested certification items remain lighter than I would prefer.

## 28.1 Systematic round-trip permutation testing

The law:

```text
body_view(
    canonicalize(source_contract)
)
≡
source_contract
```

deserves a deterministic table/property test over permutations such as:

```text
1 generic
2-way swap
3-way rotation
4-way permutation
```

I did not find such a test in the new closure patch.

The pair-swap fixture is good, but it tests one permutation.

---

## 28.2 Nested type-shape matrix

The implementation recursively handles the relevant type forms, but the requested source-level matrix for shapes such as:

```text
Option<Box<T>>
(T) -> Box<T>
Tuple<T, Box<U>>
nested associated/type-member projections
```

does not appear to have been added specifically for this incident.

This is coverage debt rather than a discovered implementation bug.

Given that the core traversal is shared and row-specific unit testing is now strong, I rate this **P3**, not a closure blocker once the P1 is fixed.

---

## 28.3 Conditional class-side receiver case

Given the `impl_domain.head_type` issue described above, this is now more than optional coverage.

A class-side conditional inherent implementation should get a dedicated regression.

For example, within whatever currently supported constraint syntax is appropriate:

```telos
impl<T> Box<T>
  where ...
{
  @class
  viaSelf() -> Int {
    self.classSideMember()
  }
}
```

Assert exact class-side callable resolution.

---

# 29. Documentation is currently inaccurate

`LANG007.C4.1-handoff.md` states:

> Runtime receiver `self` remains distinct from type-level `Self`.

At current head, that is false for inherent body contexts because both overrides are assigned the same `source_target_head`.

Likewise the walkthrough marks the corrective closure `COMPLETE`.

These documents should not remain in that state after this audit.

After fixing the receiver bug, amend the documentation to record the additional corrective commit and the stronger exact-dispatch regression.

---

# 30. Recommended corrective patch

The remaining code correction should be small and focused.

### A. Centralize body override derivation

Prefer one helper:

```rust
impl InherentBodyContext {
    pub fn runtime_receiver_override(
        &self,
        side: DispatchSide,
    ) -> Option<TypeId> {
        match side {
            DispatchSide::Instance => Some(self.source_target_head),
            DispatchSide::Class => None,
        }
    }

    pub fn instance_self_override(&self) -> TypeId {
        self.source_target_head
    }
}
```

Names can vary.

### B. Use it in direct body analysis

Replace:

```rust
self_type_override:
    body_context.map(|c| c.source_target_head),

self_instance_type_override:
    body_context.map(|c| c.source_target_head),
```

with side-aware receiver handling.

### C. Use exactly the same rule during downstream revalidation

No separate reimplementation.

### D. Correct `impl_domain` receiver override

Change:

```rust
if let Some(domain) = &impl_domain {
    ctx.self_override_type = Some(domain.head_type);
}
```

so it does not overwrite class-side runtime receiver semantics.

### E. Keep type-level `Self` targeting the impl/domain instance head

Do not “fix” this by removing `self_instance_type_override`.

That part is required.

---

# 31. Required regression suite for the corrective patch

I would require these tests before closure.

1. **Non-generic class-side inherent receiver**
   ```telos
   impl Counter {
     @class throughReceiver() -> Int {
       self.direct()
     }
   }
   ```
   Assert exact class-side `direct` callable selected.

2. **Generic covering class-side inherent receiver**
   ```telos
   impl<T> Id<T> { ... }
   ```
   Assert:
   ```text
   lowercase self -> ClassObject(Id)
   type Self      -> Id<I_T>
   ```

3. **Conditional class-side inherent receiver**
   Assert class-side dispatch remains class-side.

4. **Current bare `Self` return fixture**
   Must continue passing.

5. **Callable-local `where U == Self`**
   Must continue passing.

6. **Shadowing test**
   Must continue proving all three `T` identities distinct.

7. **Row specialization tests**
   Must remain green.

8. **Exact provenance test**
   Must remain green.

9. **Dependency-edge test**
   Explicitly assert:
   ```text
   CallableBody -> CallableDefinition
   ```

10. **Cold/incremental context permutation**
    Keep the existing test; it still provides useful end-to-end parity even though it is not an isolated dependency test.

---

# 32. Closure status by semantic law

### Publication

```text
source impl
    -> impl_to_decl
    -> canonical declaration contract
```

**PASS**

### Body coordinate restoration

```text
canonical declaration contract
    -> decl_to_impl
    -> source body contract
```

**PASS**

### Callable generic constraints

```text
body-local generic constraints
    -> source coordinates + source Self
```

**PASS**

### RecordRow recursion

```text
Self inside row carriers
    -> source target recursively
```

**PASS**

### Binder identity

```text
Declaration T != Impl T != Callable T
```

**PASS**

### Public purity

```text
unconditional public surface does not become impl-local
```

**PASS based on inspected architecture and existing signature tests**

### Source provenance

```text
ImplId + source_member_index
```

**PASS**

### Type-level Self

```text
Self in covering body
    -> source target instance
```

**PASS**

### Runtime receiver separation

```text
class-side lowercase self
    -> class object
```

**FAIL**

### Incremental/cold equivalence

**Good end-to-end coverage**, but the exact `CallableDefinition` dependency remains **not independently certified**.

---

# 33. Final assessment

This corrective series is **substantially better than the state I audited previously**. The difficult parts were mostly solved correctly:

- the shared generic-constraint traversal is the right abstraction;
- callable-local `Self` evidence is now specialized;
- RecordRow specialization is structurally complete over the identified carriers;
- binder identity is protected;
- invalid `Self<T>` is explicitly caught;
- exact body provenance has a strong unit regression;
- incremental/cold testing is materially stronger.

But the implementation accidentally reintroduced the exact conceptual conflation the new `self_instance_type_override` field was designed to prevent.

The problematic state is essentially:

```text
               source_target_head
                  /         \
                 /           \
runtime lowercase self     type-level Self
        ❌                    ✅
```

It must instead be:

```text
class-side body:

runtime lowercase self
        |
        v
ClassObject(Id)

type-level Self
        |
        v
Id<I_T>
```

Because this affects actual dispatch semantics, I consider it **P1 and closure-blocking**.

## Final disposition

```text
Covering body coordinate restoration:      FIXED
Generic constraint closure:                FIXED
RecordRow closure:                         FIXED
Binder/provenance protections:             FIXED
Class-side receiver / type-Self separation: NOT FIXED
Incremental dependency isolation:          INSUFFICIENTLY CERTIFIED

OVERALL:
    NOT CLOSED
```

The good news is that the remaining correctness repair is now narrow. **Do not redesign the inverse-substitution, generic-constraint, RecordRow, or provenance work.** Correct the side-aware runtime receiver override, fix the same rule for conditional inherent domains, replace the false-positive receiver test with exact semantic assertions, and add the direct `CallableBody → CallableDefinition` dependency regression. After those pass, I would perform one final focused closure audit.