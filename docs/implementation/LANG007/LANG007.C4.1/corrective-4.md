# Implementation audit — `main` @ `24c25d5c`

I audited the pushed state through the current head:

- `24c25d5c244a9bff6478d729511295c1774060ab` — `fix(semantic): close LANG007 C4.1 corrective gaps`
- `b6b24e259c83a787d6c2bd667a8287be1b1a8748` — `docs: record LANG007 C4.1 corrective closure`

## Executive verdict

There are **two different verdicts**, and separating them is important.

| Scope | Verdict |
|---|---|
| C4.1 covering-impl / `Self` / source-coordinate correction implemented in `24c25d5c` | **Substantially correct** |
| The two architectural match-hardening issues from my immediately preceding audit | **Still open** |
| C4.1 focused regression quality | **Good, with one serious test-oracle problem and several coverage gaps** |
| Incremental fingerprint narrowing | **Architecturally plausible, not sufficiently certified yet** |
| Full release/workspace certification | **Not established** |

So I **would keep `24c25d5c`**. It contains useful and mostly well-architected fixes.

But I **would not close the previous semantic-hardening audit**. The two exact issues I identified remain in the source essentially unchanged.

---

# 1. BLOCKER — match still has its own equality-normalization authority

This was the first remaining issue from the prior audit, and it has **not been implemented**.

Current:

`phalcom-semantic/src/checker/expression.rs`

still contains:

```rust
fn apply_ambient_equalities(
    store: &mut crate::types::store::TypeStore,
    ambient_constraints: &[crate::types::parameter::GenericConstraint],
    ty: TypeId,
) -> TypeId {
    let mut subst = crate::types::substitution::TypeSubstitution::new();

    for c in ambient_constraints {
        if let crate::types::parameter::GenericConstraint::Equivalent {
            left,
            right,
        } = c
        {
            if let (
                crate::types::parameter::TypeTerm::Canonical(left_ty),
                crate::types::parameter::TypeTerm::Canonical(right_ty),
            ) = (left, right)
            {
                if let TypeData::Parameter(p) = store.get(*left_ty) {
                    subst.bind(*p, *right_ty);
                } else if let TypeData::Parameter(p) = store.get(*right_ty) {
                    subst.bind(*p, *left_ty);
                }
            }
        }
    }

    ...
}
```

and match synthesis still does:

```rust
let scrutinee_ty = apply_ambient_equalities(
    ctx.store,
    &ctx.ambient_constraints,
    scrutinee_typed
        .knowledge
        .ty()
        .unwrap_or_else(|| ctx.store.unit()),
);
```

That is exactly the architecture I called out previously:

```text
GenericEvidenceEnvironment
        ↓
flat ambient_constraints copy
        ↓
match-specific equality solver
        ↓
coverage root
```

The repository already has a richer equality authority:

```text
GenericEvidenceEnvironment
    ├─ canonical constraints
    ├─ equivalence graph
    ├─ equivalent_terms(...)
    ├─ refinements
    └─ typed proof outcomes
```

`checker/body.rs` still explicitly derives the old side channel:

```rust
ctx.generic_evidence = generic_evidence;
ctx.ambient_constraints =
    ctx.generic_evidence.constraints().to_vec();
```

So `ambient_constraints` is a derived copy, but `apply_ambient_equalities` then turns that copy back into an independent normalization engine.

### Why the C4.1 work does not fix this

The new `specialize_self_generic_constraint(...)` work is good, but it solves a different problem:

```text
"How do we translate owner-relative Self through a generic constraint?"
```

It does **not** solve:

```text
"Who is authoritative for proving/refining equality when match coverage
consumes the callable's generic evidence?"
```

The match helper still only understands equalities with a directly exposed `TypeData::Parameter` on one side.

Thus this remains possible in principle:

```text
generic evidence proves a richer equivalence/refinement
                  ↓
GenericEvidenceEnvironment knows it
                  ↓
match-local parameter substitution cannot express it
                  ↓
coverage gets a weaker type than ordinary type semantics
```

### Verdict

**OPEN — unchanged from previous audit.**

This is not a regression introduced by `24c25d5c`; the new commit simply does not implement the requested closure.

---

# 2. BLOCKER — constructor opening still conflates `Refuted` with local-proof failure

The second previous issue is also still present.

Current:

`phalcom-semantic/src/checker/coverage/domain.rs`

still declares:

```rust
pub(crate) fn open_variant_case(
    ...
) -> Option<OpenedVariantCase>
```

Canonical GADT proof gets an explicit distinction:

```rust
match solve_gadt_branch_proof(...) {
    GadtProofResult::Reachable {
        proof,
        exact_case,
    } => ...
    GadtProofResult::Refuted => return None,
}
```

but the subsequent local proof still uses `?`:

```rust
let (local_proof_bindings, local_proof_equalities) =
    crate::checker::gadt_proof::solve_local_case_proof_against_local(
        store,
        &proof,
        &subject.local,
        &case_instantiation,
    )?;
```

`solve_local_case_proof_against_local` itself still returns:

```rust
pub(crate) type LocalCaseProof =
    Option<(
        BTreeMap<TypeParameterId, LocalType>,
        Box<[LocalConstraint]>
    )>;
```

Therefore these two facts are still collapsed:

```text
constructor is proven impossible
```

and:

```text
local/canonical constructor proof could not be established
```

into the same:

```text
None
```

The downstream consequence also remains.

For an exact case:

```rust
let Some(opened) = open_variant_case(...) else {
    return DomainDecomposition::Empty;
};
```

For a general enum:

```rust
if let Some(opened) = open_variant_case(...) {
    cases.push(...);
}
```

and if none remain:

```rust
return DomainDecomposition::Empty;
```

That is precisely the semantic shape I objected to.

The coverage engine already knows how to propagate:

```rust
DomainDecomposition::Blocked(...)
    ↓
ExhaustivenessResult::Blocked(...)
    ↓
bootstrap rejection
    ↓
lowering rejection
```

but `open_variant_case` cannot produce that state.

The proper boundary remains conceptually:

```rust
enum VariantOpenOutcome {
    Reachable(OpenedVariantCase),
    Refuted,
    Blocked(BlockReason),
}
```

with local proof machinery returning enough information to distinguish a genuine contradiction from inability to prove.

### Why this matters

A producer bug or unsupported local proof must never accidentally become:

```text
could not prove constructor
        ↓
constructor disappears
        ↓
domain appears empty
        ↓
exhaustiveness becomes Proven
```

The existing fail-closed lowering safeguards cannot protect against this if the semantic layer has already falsely emitted `Proven`.

### Verdict

**OPEN — unchanged from previous audit.**

This is the more important of the two remaining architecture issues because it sits directly on an exhaustiveness-soundness boundary.

---

# 3. The C4.1 source/body coordinate architecture itself is now good

The new work in `24c25d5c` materially improves the covering-inherent-impl implementation.

The resulting architecture is now much cleaner:

```text
published callable signature
declaration-owned coordinates
          │
          │ inverse covering materialization
          ▼
impl-source generic coordinates
          │
          │ owner-relative Self specialization
          ▼
source-body semantic view
          │
          ▼
body checker
```

This is the correct separation.

The important property is:

```text
published API contract ≠ source-body checking coordinate system
```

without creating two independent semantic contracts.

## `specialize_self_type_term`

The new helper in:

`phalcom-semantic/src/types/substitution.rs`

is a good abstraction:

```rust
pub fn specialize_self_type_term(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    receiver: TypeId,
    term: &TypeTerm,
) -> TypeTerm
```

It correctly handles:

```text
Canonical(T)
    → recursively specialize Self inside T

SelfType(Self)
    → construct canonical Self form
    → specialize against receiver

Infer(v)
    → preserve Infer(v)
```

The last point is important. It does **not** casually materialize inference variables or alter binder identity.

That is the right ownership boundary.

---

# 4. Generic-constraint specialization is now centralized correctly

The introduction of:

```rust
pub fn specialize_self_generic_constraint(...)
```

is a strong improvement.

It delegates structural traversal through:

```rust
map_generic_constraint(...)
```

and covers:

```text
Subtype
Equivalent
Conforms
```

including the `TraitRef.arguments` lane.

Both major consumers now share the same semantic transformation instead of maintaining separate partial walkers.

That eliminates a real class of future bugs where:

```text
parameter / result type has specialized Self
```

but:

```text
where-clause / trait argument still contains old Self
```

The new regression:

```rust
self_specialization_recurses_through_generic_constraint_trait_arguments
```

directly proves:

```text
Conforms {
    subject: Self,
    trait_ref: Marker<Self>,
}
```

becomes:

```text
Conforms {
    subject: Receiver,
    trait_ref: Marker<Receiver>,
}
```

That part is well designed.

**Verdict: PASS.**

---

# 5. Record-row carrier closure is considerably better

The recursive `Self` traversal through row-bearing structures is now much more complete.

`specialize_self_row(...)` owns:

```text
field value types
closed tails
anonymous unconstrained key type
anonymous typed key/value types
stable row parameters
projection tails
```

and this is reused by:

```text
TypeData::Record
StaticArgument::RecordRow
ProjectionSubject::RecordRow
TypeMemberArgument::RecordRow
```

The change that replaces the duplicated `TypeData::Record` traversal with:

```rust
let row_id =
    specialize_self_row(...);

let row =
    store.record_row(row_id).clone();

store.record_row_type_checked(
    row.fields.to_vec(),
    row.tail,
)
```

is also the right refactoring.

It materially reduces the chance that `Record` and row-as-static-argument diverge semantically later.

The added materialization test for row projection carriers is also useful:

```text
declaration row parameter R
        ↓ covering materialization
impl row parameter S
        ↓
ProjectionSubject::RecordRow
TypeMemberArgument::RecordRow
```

and verifies both carriers retain `S`.

**Verdict: PASS.**

---

# 6. Callable-local binder identity is now protected much better

This is one of the strongest new tests:

```rust
test_covering_body_preserves_callable_generic_shadowing
```

The source deliberately creates three `T`s:

```telos
data Box<T> ...

impl<T> Box<T> {
    identity<T>(_ value: T) -> T { value }
}
```

and then verifies:

```text
declaration T
≠ impl T
≠ callable T
```

despite all three having the same spelling.

That is exactly the discriminator needed here. Generic identity must be:

```text
owner + index
```

not:

```text
identifier text
```

It then verifies that the body signature retains the **callable-owned** parameter for both argument and result.

This is strong evidence that the correction is not relying on name-based substitution.

**Verdict: PASS.**

---

# 7. `where U == Self` body specialization is now directly exercised

This new test is also valuable:

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

It exercises three things simultaneously:

```text
callable-local U
owner-relative Self
generic Equivalent constraint
```

under covering inherent body reconstruction.

That directly addresses one of the gaps in the previous implementation.

**Verdict: PASS.**

---

# 8. Negative binder-confusion coverage is good

This regression is correctly designed:

```telos
data Id<T>()

impl<T> Id<T> {
    bad<U>(_ value: Id<U>) -> Id<T> {
        value
    }
}
```

It requires `ReturnMismatch`.

This prevents an overaggressive source-coordinate translation from incorrectly unifying:

```text
U
```

with:

```text
impl T
```

merely because both are available generic parameters.

This is an important negative discriminator.

**Verdict: PASS.**

---

# 9. `Self<T>` diagnostic publication is correctly repaired

The new test:

```telos
impl<T> Id<T> {
    bad(_ value: Self<T>) -> Unit {
        ()
    }
}
```

requires:

```text
ApplicationNotConstructor
```

That is semantically correct: once `Self` denotes a proper instance type, it is not a generic constructor that can be reapplied.

The production change around:

```rust
let diagnostic_start = ctx.diagnostics.len();

let raw_sig =
    semantic_signature_for_syntax_with_resolver(...);

diagnostics.extend(
    ctx.diagnostics[diagnostic_start..]
        .iter()
        .cloned()
);
```

is also reasonable: signature formation diagnostics now survive into the contribution even when the malformed signature prevents normal body checking.

I checked the primary session publication path; it consumes `contribution.diagnostics` rather than separately publishing this temporary checking context's diagnostic vector, so I do **not** see an immediate double-publication defect there.

**Verdict: PASS.**

---

# 10. Exact source-body provenance remains correct

The implementation continues to use the accepted semantic definition to locate the source body.

`source_body_for_callable(...)` requires:

```rust
definition.origin
    == CallableDefinitionOrigin::InherentImpl(expected_impl)
```

and:

```rust
definition.source_member_index
    == source_member_index
```

before accepting the AST member.

That preserves:

```text
semantic acceptance
        ↓
exact ImplId
        ↓
exact source member index
        ↓
source body
```

rather than doing a selector-only search.

This is especially important with duplicate/rejected implementations.

The previously added duplicate-definition regression still verifies that a rejected candidate cannot replace the accepted body's analysis.

**Verdict: PASS.**

---

# 11. Context-only incremental invalidation is now well tested

The new test:

```rust
test_covering_body_context_only_permutation_changes_context_not_public_contract
```

is good.

It changes:

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

It then verifies:

```text
CallableDefinition product     changes
source_target_head             changes

public signature product       unchanged

body query                     refreshed/revalidated
incremental semantic product   == cold semantic product
```

This is exactly the right invariant.

The implementation correctly recognizes that:

```text
body's coordinate system changed
```

does not imply:

```text
public contract changed
```

That is architecturally solid.

**Verdict: PASS.**

---

# 12. The callable-body fingerprint narrowing is plausible, but needs one more correctness gate

`24c25d5c` makes a significant incremental-semantic change in:

`phalcom-semantic/src/db/fingerprint.rs`

The callable-body direct input fingerprint now deliberately does **not** hash the broad:

```text
sources
source_resolution_input
linked_component_product
declared_signature
conformance_semantics aggregate
```

as direct inputs.

Instead, it relies increasingly on explicit DB dependencies.

This is the right long-term direction.

For ordinary/inherent bodies, I verified that body checking records:

```rust
SemanticDependency::CallableSignature(...)
```

and body query publication additionally records:

```rust
QueryKey::CallableDefinition(callable)
```

when an accepted definition exists.

Covering context changes are therefore represented through the exact definition product.

Trait defaults directly consume/hash their trait surface.

Conformance use records:

```text
ConformanceDispatch(impl)
TraitSurface(trait)
```

for positive evidence.

The session also has explicit settled-plan reanalysis:

```text
changed_conformance_plans
    ↓
ConformanceDispatch keys
    ↓
reverse dependency closure
    ↓
affected callable bodies
```

and a `prior_conformance_clients` mechanism.

So I do **not** see evidence that the narrowing is inherently wrong.

## But the hardest case is not certified

The dangerous incremental transition is:

```text
revision N:
    a conformance does not exist
    consumer observes Missing / no candidate

revision N+1:
    a new conformance is introduced
```

There cannot have been a dependency on:

```text
ConformanceDispatch(new_impl)
```

at revision N because that product did not exist.

This is the classic **absence dependency** problem.

The current session has broader structural/trait invalidation machinery that may correctly catch this through trait surfaces and semantic work propagation, but I did not find a focused regression demonstrating the exact contract:

```text
previously absent conformance
    → add conformance
    → cross-module consumer recomputes
    → cold parity
```

Likewise the reverse:

```text
present conformance
    → delete conformance
    → exact former consumer recomputes
```

should be explicit.

Given that this commit removes coarse fingerprint barriers, this discriminator is important enough to require before certifying the optimization.

### Also: the source comment is now stale

The function still says:

```rust
/// Computes callable-body input identity from the conservative A4 workspace inputs.
///
/// Exact semantic dependencies are recorded separately, but broad source and
/// linked-program barriers remain until the next checkpoint proves they can be
/// removed safely.
```

But those inputs are now literally destructured as:

```rust
sources: _sources,
source_resolution_input: _source_resolution_input,
linked_component_product: _linked_component_product,
...
conformance_semantics: _conformance_semantics,
```

and are not hashed.

That comment is false after this commit.

**Verdict: implementation direction GOOD; certification INCOMPLETE; comment MUST be corrected.**

---

# 13. One new test has an internally inconsistent oracle

This is the most concrete defect I found in the newly added test suite.

The test:

```rust
test_class_side_type_self_and_lowercase_receiver_remain_distinct
```

contains:

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

    @class
    throughType() -> Self {
        Self.direct()
    }
}
```

and asserts:

```rust
assert!(!update.snapshot.has_errors(), ...);
```

The second method is not a valid positive example under the semantic model implemented elsewhere in the repository.

`SelfRole::InstanceType` explicitly means the **instance type**, even for class-side formation.

That is also exactly what C4.1's documentation says.

Therefore:

```text
Self
    = Counter instance type

Self.direct()
    = result of Counter's class-side direct()
    = Int
```

so:

```telos
@class
throughType() -> Self {
    Self.direct()   // Int
}
```

should be:

```text
Int → Counter
```

and should produce a return mismatch.

This test therefore does **not** cleanly prove the intended distinction.

Either:

```text
A. the test source is wrong,
```

or, if the repository really reports this test as passing:

```text
B. there is another semantic bug allowing Int to satisfy Self here.
```

The cleaner discriminator is:

```telos
impl Counter {
    @class
    throughReceiver() -> Int {
        self.direct()
    }

    @class
    throughType() -> Int {
        Self.direct()
    }
}
```

and then a *separate* valid test for a `-> Self` return using an expression that actually has type `Counter`.

This needs correction before treating the reported `17/17 impls::bodies` result as evidence for the lowercase-`self` / type-level-`Self` distinction.

**Severity: HIGH test-quality issue; potentially a semantic bug if the current test genuinely passes exactly as recorded.**

---

# 14. Recursive public-signature purity is not directly locked down

The code is significantly better than its tests here.

`apply_covering_instantiation_to_signature(...)` correctly materializes:

```text
parameter canonical types
declared return canonical type
Subtype operands
Equivalent operands
Conforms subject
Conforms TraitRef arguments
```

through the covering body instantiation.

That is good.

But the existing public-purity assertion/test remains relatively shallow.

The implementation has a debug invariant around published parameter/result types, while `impls/signatures.rs::test_permutation_signature_canonicalization` proves top-level returns become declaration-owned parameters.

What I do **not** see is a recursive regression asserting:

```text
published callable signature contains no
TypeParameterOwner::Impl(current_impl)
```

through all relevant positions:

```text
Applied arguments
AssociatedProjection
TypeMemberProjection
Record rows
packs
callables
generic constraints
trait-ref arguments
```

Given that the entire C4.1 incident arose from coordinate leakage, this is worth locking down as one reusable recursive test helper.

This is a **verification gap**, not evidence of a current implementation defect.

---

# 15. Source-level nested coverage is still thinner than the low-level helper coverage

The new unit-level structural coverage is strong for rows.

But I did not find new end-to-end covering-body source regressions for combinations such as:

```text
Option<Box<T>>
(T) -> Box<T>
tuple containing Self
record containing Self
associated projection containing Self
type-member projection containing Self
pack containing Self
```

The low-level `specialize_self_type(...)` implementation does traverse most of these correctly, so I do not see a concrete producer bug there.

The issue is certification: direct helper tests and a handful of source examples do not fully prove that:

```text
parse
→ signature formation
→ canonical publication
→ inverse covering materialization
→ Self specialization
→ body checking
```

preserves all those forms.

Again, this is a coverage gap rather than a reason to revert the implementation.

---

# 16. Recorded verification is focused, not broad certification

The C4.1 lifecycle records report successful focused commands including:

```text
cargo check -p phalcom-semantic
impls::bodies
impls::signatures
foundations::record_rows_field_keys
impls::queries
git diff --check
```

The records report, among other things:

```text
impls::bodies                    17 passed
impls::signatures                 4 passed
record_rows_field_keys           14 passed
impls::queries                   60 passed
```

Those are useful focused results.

I could not independently execute the repository in this chat, and GitHub reports **no workflow runs attached to `24c25d5c`**.

I therefore cannot promote the result to:

```text
workspace/release certified
```

from the available evidence.

This matters particularly because the patch changes incremental fingerprints, not merely local `Self` substitution.

---

# Consolidated findings

| ID | Finding | Severity | Status |
|---|---|---:|---|
| A1 | Match still uses private `apply_ambient_equalities` instead of generic-evidence authority | **BLOCKER** | **OPEN** |
| A2 | `open_variant_case -> Option` still conflates refutation and inability/local-proof failure | **BLOCKER** | **OPEN** |
| C1 | C4.1 source/declaration coordinate separation | High | **FIXED CORRECTLY** |
| C2 | `Self` specialization centralized at `TypeTerm`/constraint level | High | **FIXED CORRECTLY** |
| C3 | `Self` through trait-ref arguments | High | **FIXED CORRECTLY** |
| C4 | Record-row carrier specialization | High | **FIXED CORRECTLY** |
| C5 | Declaration/impl/callable binder shadowing | High | **GOOD REGRESSION** |
| C6 | `Self<T>` misuse publishes proper diagnostic | Medium | **FIXED** |
| C7 | Exact `(ImplId, source_member_index)` body provenance | High | **PRESERVED** |
| C8 | Context-only impl permutation invalidates body but not public contract | High | **GOOD** |
| T1 | Class-side `throughType() -> Self { Self.direct() }` positive test has invalid oracle | **HIGH** | **FIX TEST / INVESTIGATE** |
| I1 | Callable-body coarse fingerprint barriers removed | Medium | **LIKELY SOUND, NEEDS ABSENCE-DEPENDENCY TESTS** |
| I2 | Fingerprint documentation still claims removed barriers remain | Low | **STALE COMMENT** |
| T2 | Recursive “no impl-owned parameter escapes public signature” regression absent | Medium | **TEST GAP** |
| T3 | Nested source-level covering forms remain incompletely exercised | Medium | **TEST GAP** |
| V1 | No current-head CI/workspace/release evidence | Medium | **UNCERTIFIED** |

# Final ruling

**`24c25d5c` is a good patch and should remain.** The C4.1 source-coordinate / covering-body architecture is now substantially more coherent than it was at `677bb585`.

I would classify C4.1 itself as:

> **IMPLEMENTATION ACCEPTABLE / FOCUSED VERIFICATION NEEDS CORRECTION AND COMPLETION**

I would **not**, however, classify the broader incident we discussed in the immediately previous audit as completely resolved. Its two remaining match/coverage soundness obligations were not touched by these commits.

For full closeout, I would require exactly these remaining actions:

1. Replace `apply_ambient_equalities` with a canonical `GenericEvidenceEnvironment`-backed type-refinement/normalization operation; match must become a consumer, not an equality solver.
2. Introduce a typed constructor-opening result (`Reachable | Refuted | Blocked`) and propagate local proof inability into `DomainDecomposition::Blocked`, never `Empty`.
3. Correct `test_class_side_type_self_and_lowercase_receiver_remain_distinct`; if the current invalid source really passes, investigate that as a separate return-type soundness defect.
4. Add a recursive public-signature coordinate-purity assertion and a small nested covering-signature matrix.
5. Add incremental **absent → added conformance** and **present → removed conformance** cold-parity regressions before accepting the new narrow callable-body fingerprint as certified.
6. Update the stale callable-body fingerprint comment and run the broader semantic/core/workspace gates.

After **A1 and A2** are actually landed, the original match/coverage architecture can be considered closed. The current head has not reached that point yet.