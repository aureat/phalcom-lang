# Phalcom semantic architecture re-audit — current `main`

**Repository:** `aureat/phalcom-lang`
**Audited head:** `ee7e8ee725f90d71ba4a96f907f27339abd989a0` — `fix(semantic): address C4.1 audit findings`
**Previous implementation head audited:** `24c25d5c244a9bff6478d729511295c1774060ab`
**Audit scope:** follow-up verification of the previous C4.1 / canonical `Result.transpose` / match-coverage audit, including the latest corrective commit, match proof ownership, generic equality authority, covering inherent-impl `Self`, callable-body incrementality, negative trait-dispatch dependencies, signature-coordinate purity, and certification gaps.
**Audit date:** 2026-10-06

---

# 1. Executive verdict

The latest corrective commit fixes a meaningful subset of the previous findings, but it does **not** close the broader semantic architecture audit.

The most important distinction is that the **original observable `Result.transpose` failure is now repaired** while two underlying proof-architecture problems remain. The current focused semantic regression now proves the outer `Result` match, the equality-constrained inner `Option` match, direct equality, reversed equality, and chained equality cases. It also checks that the `Result::Ok(option)` payload enters the arm as an `Option`-origin type. A natural runtime `Result.transpose()` test remains in the core suite. Therefore the incident should no longer be described as “`Result.transpose` is still broken.”

However, the implementation still contains:

1. a **second, match-local generic equality normalizer** (`apply_ambient_equalities`) instead of one canonical `GenericEvidenceEnvironment`-backed refinement authority;
2. an **`Option`-typed constructor-opening boundary** where genuine refutation and local proof inability can still collapse to the same `None`, after which domain decomposition may interpret the result as `Empty`;
3. a newly sharpened incremental problem: after callable-body fingerprint narrowing, **negative trait-dispatch lookup has no durable dependency product**, so a body that previously observed “no trait candidate” has no exact dependency edge that can be invalidated when a new conformance later makes the candidate exist.

The new head **does correctly fix** the class-side lowercase-`self` / type-level-`Self` conflation, including the conditional inherent-impl path and the previously invalid regression oracle. It also corrects the stale callable-body fingerprint documentation and directly asserts the `CallableBody -> CallableDefinition` dependency.

## 1.1 Current disposition

| Area | Current verdict |
|---|---|
| Original `Result.transpose` nested-match symptom | **FIXED behaviorally** |
| Canonical Universe non-proven-match readiness gate | **PRESERVED / GOOD** |
| Lowering `NonProvenMatch` fail-closed defense | **PRESERVED / GOOD** |
| Class-side lowercase `self` vs type-level `Self` | **FIXED** |
| Conditional class-side inherent-impl receiver override | **FIXED** |
| Previous class-side test oracle defect | **FIXED** |
| `CallableBody -> CallableDefinition` dependency | **NOW DIRECTLY ASSERTED** |
| Stale callable-body fingerprint comment | **FIXED** |
| Match-local `apply_ambient_equalities` authority duplication | **OPEN — architectural closure blocker** |
| `open_variant_case -> Option` refutation/blocking conflation | **OPEN — soundness blocker** |
| Missing trait-dispatch negative dependency after fingerprint narrowing | **OPEN — high-confidence incremental correctness defect** |
| Recursive “no impl-owned binder leaks into public signature” invariant | **OPEN — verification/invariant gap** |
| End-to-end nested covering-shape matrix | **OPEN — verification gap** |
| Current-head broad CI / release certification | **NOT ESTABLISHED** |

My updated overall disposition is therefore:

> **The immediate canonical `Result.transpose` incident is repaired and the C4.1 `Self` follow-up is materially improved, but the semantic proof architecture is not yet closed. Two match-proof seams and one incremental negative-dependency seam should be repaired before this area is treated as architecturally complete.**

---

# 2. What changed since the previous audit

The latest implementation commit is:

```text
ee7e8ee725f90d71ba4a96f907f27339abd989a0
fix(semantic): address C4.1 audit findings
```

It changes five production/test areas.

## 2.1 Dispatch-side-aware inherent body receiver overrides

`phalcom-semantic/src/impls.rs` now gives `InherentBodyContext` two explicit meanings:

```rust
pub fn runtime_receiver_override(
    &self,
    side: DispatchSide,
) -> Option<TypeId> {
    (side == DispatchSide::Instance)
        .then_some(self.source_target_head)
}

/// Returns the source-relative instance type used for type-level `Self`.
pub fn instance_self_override(&self) -> TypeId {
    self.source_target_head
}
```

`phalcom-semantic/src/session.rs` now consumes them separately:

```rust
self_type_override:
    body_context
        .as_ref()
        .and_then(|context|
            context.runtime_receiver_override(callable.side)
        ),

self_instance_type_override:
    body_context
        .as_ref()
        .map(|context|
            context.instance_self_override()
        ),
```

and the same split is used by downstream body revalidation.

This is the correct conceptual model:

```text
instance-side inherent body
    lowercase self  -> source target instance
    type-level Self -> source target instance

class-side inherent body
    lowercase self  -> ordinary class object
    type-level Self -> source target instance
```

## 2.2 Conditional inherent domains no longer overwrite class-side runtime `self`

`phalcom-semantic/src/checker/body.rs` now gates the domain receiver override:

```rust
if callable.side == crate::identity::DispatchSide::Instance
    && let Some(domain) = &impl_domain
{
    ctx.self_override_type = Some(domain.head_type);
}
```

This closes the adjacent path I called out previously: a conditional class-side inherent impl can no longer force its instance target head into lowercase `self`.

## 2.3 The class-side regression is now semantic-product-based

The old test merely asserted that no diagnostics were emitted and had an invalid `throughType() -> Self { Self.direct() }` oracle.

The new test:

```phalcom
class Counter {
  @class
  direct() -> Int { 10 }
}

impl Counter {
  @class
  throughReceiver() -> Int { self.direct() }

  @class
  throughType() -> Int { Self.direct() }
}
```

then inspects the semantic product and requires:

```text
lowercase self
    -> TypeData::ClassObject { declaration: Counter }

self.direct()
    -> exact class-side Counter.direct callable
```

There are also dedicated generic-covering and conditional-class-side regressions.

That is a substantial improvement over “no error occurred.”

## 2.4 The callable-definition dependency is now explicitly asserted

The context-only permutation test now checks the dependency graph:

```rust
session
    .db()
    .index()
    .dependencies_of(&body_key)
    .is_some_and(|edges|
        edges.iter().any(|edge|
            edge.dependency == definition_key
        )
    )
```

This proves that the body query really records its accepted `CallableDefinition` dependency rather than relying only on coincident source dirtiness.

## 2.5 The callable-body fingerprint documentation now matches the implementation

The old comment claimed broad workspace / linked-program products remained hashed even though they had been intentionally removed.

It now correctly states:

```rust
/// Computes callable-body input identity from direct body-relative inputs.
///
/// Exact semantic dependencies are recorded separately and are validated by
/// the query layer before reuse. Broad workspace and linked-program products
/// are intentionally not hashed here because they are not direct inputs to a
/// callable body.
```

That documentation defect is closed.

---

# 3. The original `Result.transpose` failure is now behaviorally fixed

This is the largest factual update to the previous report.

The current semantic regression in:

```text
phalcom-semantic/tests/semantic/adts/matching/exhaustiveness.rs
```

contains the original shape:

```phalcom
transpose<U>()
    -> Option<Result<U, E>>
    where T == Option<U>
{
    match self {
        Result::Ok(option) => {
            match option {
                Option::Some(value) =>
                    Option::Some(Result::Ok(value))

                Option::None =>
                    Option::None
            }
        }

        Result::Error(error) =>
            Option::Some(Result::Error(error))
    }
}
```

The same test also contains three direct discriminators:

```phalcom
directConstrainedOption<U>(
    _ option: T
) where T == Option<U> {
    match option {
        Option::Some(_) => 1
        Option::None => 0
    }
}

reverseConstrainedOption<U>(
    _ option: T
) where Option<U> == T {
    match option {
        Option::Some(_) => 1
        Option::None => 0
    }
}

directChainedConstrainedOption<U, V>(
    _ option: T
) where T == V, V == Option<U> {
    match option {
        Option::Some(_) => 1
        Option::None => 0
    }
}
```

The test asserts all of the following are exhaustive:

```text
contextual Result self match
qualified Result self match
transpose outer Result match
direct constrained Option match
reversed constrained Option match
chained constrained Option match
transpose inner Option match
```

It additionally inspects the `Result::Ok(option)` binding and requires its canonical nominal origin to be `Option` before the nested match is analyzed.

Finally, it verifies the nested candidate identities:

```text
arm 0 -> Option::Some
arm 1 -> Option::None
```

This means the original failure mode:

```text
T == Option<U>
    but nested option remains semantically opaque
    -> Some/None leave residual wildcard
    -> Missing([Wildcard])
```

is no longer the observed result.

There is also a core runtime test:

```text
phalcom-core/tests/core/language/algebraic_data/scenarios.rs
::result_transpose_executes_natural_braced_lexical_match_arm
```

which executes a natural `Result<Option<Int>, String>::Ok(...).transpose()` path.

## 3.1 Correct updated interpretation

The previous audit should therefore be amended from:

> “The constrained nested `Option` failure remains.”

to:

> **“The constrained nested `Option` failure is fixed by the current implementation, but the mechanism that fixes it still duplicates generic-equality authority inside match synthesis.”**

That distinction matters. A working reproducer does not automatically prove the architecture is sound for all evidence forms.

---

# 4. Finding A1 — match still owns a second generic-equality normalizer

**Severity:** High architectural correctness / completeness
**Status:** **OPEN**
**Observable original reproducer:** fixed
**Architectural obligation:** not fixed

The current source still contains in:

```text
phalcom-semantic/src/checker/expression.rs
```

the match-private helper:

```rust
fn apply_ambient_equalities(
    store: &mut crate::types::store::TypeStore,
    ambient_constraints: &[
        crate::types::parameter::GenericConstraint
    ],
    ty: TypeId,
) -> TypeId {
    let mut subst =
        crate::types::substitution::TypeSubstitution::new();

    for c in ambient_constraints {
        if let
            crate::types::parameter::GenericConstraint::Equivalent {
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

    if subst.is_empty() {
        return ty;
    }

    let mut normalized = ty;

    for _ in 0..=subst.len() {
        let next = subst.apply(store, normalized);

        if next == normalized {
            return normalized;
        }

        normalized = next;
    }

    ty
}
```

and match synthesis still consumes it directly:

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

Meanwhile `checker/body.rs` does:

```rust
ctx.generic_evidence = generic_evidence;

ctx.ambient_constraints =
    ctx.generic_evidence.constraints().to_vec();
```

So the effective ownership is still:

```text
GenericEvidenceEnvironment
        │
        │ copied as flat constraints
        ▼
ambient_constraints
        │
        │ match-specific re-solving
        ▼
apply_ambient_equalities
        │
        ▼
CoverageSubject.canonical
```

That is a duplicated semantic authority.

## 4.1 Why this remains architecturally wrong even though the current test passes

The current helper understands a narrow family of equations:

```text
Parameter == CanonicalType
CanonicalType == Parameter
```

and can transitively reapply the resulting substitution.

That is why these now work:

```text
T == Option<U>
Option<U> == T
T == V
V == Option<U>
```

But `GenericEvidenceEnvironment` already models a richer semantic space:

```rust
pub struct GenericEvidenceEnvironment {
    constraints: Box<[GenericConstraint]>,
    conformances: BTreeMap<...>,
    equivalences: BTreeMap<TypeTerm, BTreeSet<TypeTerm>>,
    refinements: BTreeMap<TypeTerm, TypeTerm>,
    layout: GenericEvidenceLayout,
}
```

It owns:

```text
equivalence connected components
assumed conformances
projection refinements
generic obligation proof
typed terminal outcomes
```

and generic projection normalization already has:

```rust
ProjectionNormalizationMode::Generic {
    evidence: &GenericEvidenceEnvironment,
    ...
}
```

The match helper cannot represent all of that.

### Example latent discriminator

Consider an associated projection equality:

```phalcom
trait HasItem {
    type Item
}

fn classify<I, U>(
    _ value: I::Item
)
where
    I |= HasItem,
    I::Item == Option<U>
{
    match value {
        Option::Some(_) => 1
        Option::None => 0
    }
}
```

The semantic fact is conceptually:

```text
I::Item == Option<U>
```

The current match helper sees neither side as a direct `TypeData::Parameter`, so its private substitution builder has nothing to bind.

The generic evidence / projection subsystem, however, is specifically designed to represent:

```text
associated projection
    +
lexical equivalence
    +
assumed conformance
    ->
generic projection refinement
```

This is exactly the type of case that should not require another match-specific solver.

## 4.2 Root cause

The implementation has two different answers to:

> “What is the canonical type of this body expression under the callable's lexical generic evidence?”

Ordinary generic/projection semantics answer through:

```text
GenericEvidenceEnvironment
ProjectionNormalizationMode::Generic
generic obligation proof
```

Match root formation answers through:

```text
flat GenericConstraint scan
TypeSubstitution
```

A semantic proof system should not have a coverage-local reinterpretation of generic equality.

## 4.3 Correct architecture

Introduce one reusable **lexical generic type refinement** operation owned by the generic-evidence/type-normalization layer.

Conceptually:

```rust
pub enum GenericTypeRefinement {
    Refined(TypeId),
    Unchanged(TypeId),

    Ambiguous,
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Recursive,
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}
```

The exact enum name is secondary. The important properties are:

1. **authority:** the operation consumes `GenericEvidenceEnvironment`;
2. **composition:** it reuses generic projection normalization rather than duplicating it;
3. **equality closure:** it understands the full equivalence component, not only one directly bound parameter;
4. **determinism:** it chooses a refinement only when the evidence establishes one coherent canonical target;
5. **fail-closed outcomes:** ambiguity, recursion, blocked dependencies, cancellation, and budgets do not silently become an arbitrary type;
6. **structural recursion:** refinement applies through normal canonical type formation rather than source spelling;
7. **no declaration mutation:** generic declarations remain generic; this is a view/refinement operation;
8. **shared use:** match, expression typing, pattern payload formation, and any future consumer use the same primitive.

A viable ownership shape is:

```text
GenericEvidenceEnvironment
        │
        ▼
refine_canonical_type_under_evidence(...)
        │
        ├── equivalence closure
        ├── projection normalization
        ├── bounded/cancelled outcomes
        └── canonical TypeStore reconstruction
        │
        ▼
ordinary checker semantic type
        │
        ▼
CoverageSubject::from_expression(...)
```

Then delete:

```text
checker/expression.rs::apply_ambient_equalities
```

rather than moving it elsewhere under another name.

## 4.4 Important design detail: do not abuse `unique_refinement`

Current `GenericEvidenceEnvironment::unique_refinement(...)` is not sufficient as the new general authority.

It is built as:

```text
for each equivalence component:
    if a term has exactly one "other" term
        publish that as a refinement
```

That works for a two-node relation such as:

```text
Projection == ConcreteType
```

but is not a general canonical representative algorithm for:

```text
T == U == Option<V>
```

The new refinement operation should inspect the whole proven equivalence component and select only a semantically unique, structurally informative target.

For example:

```text
{ T, U, Option<V> }
        ->
Option<V>
```

is a coherent refinement for `T` and `U`.

But:

```text
{ T, Foo<U>, Bar<V> }
```

must not arbitrarily choose one concrete-looking term if the environment is inconsistent, ambiguous, or still symbolic.

## 4.5 Required regressions

Keep the current direct/reverse/chained tests, but add at least:

```text
A1-R1 direct parameter equality
A1-R2 reversed equality
A1-R3 chained parameter equality
A1-R4 associated projection equality -> closed Option match
A1-R5 nested projection inside applied type
A1-R6 ambiguous/non-unique equivalence remains blocked/symbolic
A1-R7 cyclic malformed equality does not publish a partial specialization
A1-R8 ordinary checker type and coverage-root type are identical under the same evidence
```

### Acceptance criterion

The audit is not closed until:

```text
grep/use-site inspection finds no match-owned equality solver
```

and a coverage root is formed from the same refined semantic type authority ordinary checking uses.

---

# 5. Finding A2 — constructor opening still conflates refutation and proof inability

**Severity:** **Soundness blocker**
**Status:** **OPEN**

The existing coverage stack already supports fail-closed blocking:

```text
DomainDecomposition::Blocked
    ->
Inhabitation::Blocked
    ->
UsefulnessSearch::Blocked
    ->
ExhaustivenessResult::Blocked
    ->
MatchAnalysisBlocked diagnostic
    ->
canonical bootstrap rejection
    ->
lowering NonProvenMatch defense
```

That part is good.

The problem is that one crucial producer cannot emit `Blocked`.

## 5.1 Current constructor-opening API

`phalcom-semantic/src/checker/coverage/domain.rs` still exposes:

```rust
pub(crate) fn open_variant_case(
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    rigids: &mut RigidArena,
    subject: &CoverageSubject,
    variant_info: &VariantInfo,
) -> Option<OpenedVariantCase>
```

Its canonical proof distinguishes reachability:

```rust
let (mut proof, exact_case) =
    match solve_gadt_branch_proof(...) {
        GadtProofResult::Reachable {
            proof,
            exact_case,
        } => (proof, exact_case),

        GadtProofResult::Refuted =>
            return None,
    };
```

But immediately afterward, local proof solving also returns an `Option`:

```rust
let (
    local_proof_bindings,
    local_proof_equalities,
) =
    solve_local_case_proof_against_local(
        store,
        &proof,
        &subject.local,
        &case_instantiation,
    )?;
```

and in `gadt_proof.rs`:

```rust
pub(crate) type LocalCaseProof =
    Option<(
        BTreeMap<TypeParameterId, LocalType>,
        Box<[LocalConstraint]>
    )>;
```

So this boundary has only two states:

```text
Some(...) -> success
None      -> everything else
```

## 5.2 Why `None` is semantically overloaded

`solve_local_case_proof_against_local(...)` currently does:

```rust
if !unify_local_types(...) {
    return None;
}
```

The recursive `unify_local_types(...) -> bool` can return `false` for different semantic reasons.

Some are genuine proof refutations:

```text
known different variant identities
different tuple arity
different labels
incompatible repeated binding
canonical concrete type mismatch
```

Those should mean:

```text
Refuted
```

But other failures can represent the local proof engine's inability to establish a coherent relation:

```text
opaque parent rigid relation
unsupported local/canonical carrier
proof state that cannot be safely reconciled
an internal local/canonical coherence gap
future local type forms not handled by this unifier
```

Those must not be interpreted as “the constructor is impossible.”

## 5.3 Current unsound collapse in domain decomposition

For an exact case:

```rust
let Some(opened) =
    open_variant_case(...)
else {
    return DomainDecomposition::Empty;
};
```

For a general enum:

```rust
if let Some(opened) =
    open_variant_case(...)
{
    cases.push(...);
}
```

and:

```rust
if cases.is_empty() {
    return DomainDecomposition::Empty;
}
```

Therefore the dangerous semantic path is:

```text
local proof cannot establish case coherently
        │
        ▼
None
        │
        ▼
case omitted
        │
        ▼
possibly zero cases
        │
        ▼
DomainDecomposition::Empty
        │
        ▼
potential ExhaustivenessResult::Proven
```

The bootstrap and lowering guards cannot protect against a false `Proven` product that was manufactured upstream.

## 5.4 Correct architecture

Replace both boolean/`Option` proof boundaries with typed semantic outcomes.

Conceptually:

```rust
pub(crate) enum LocalCaseProofOutcome {
    Reachable {
        bindings:
            BTreeMap<TypeParameterId, LocalType>,
        equalities:
            Box<[LocalConstraint]>,
    },

    Refuted,

    Blocked(BlockReason),
}
```

and:

```rust
pub(crate) enum VariantOpenOutcome {
    Reachable(OpenedVariantCase),
    Refuted,
    Blocked(BlockReason),
}
```

Then:

```rust
match open_variant_case(...) {
    VariantOpenOutcome::Reachable(opened) => {
        ...
    }

    VariantOpenOutcome::Refuted => {
        // proven impossible constructor;
        // omit from finite domain
    }

    VariantOpenOutcome::Blocked(reason) => {
        return DomainDecomposition::Blocked(reason);
    }
}
```

For an exact-case subject:

```rust
Refuted
    -> Empty

Blocked
    -> Blocked
```

These are fundamentally different facts and must remain different.

## 5.5 Refactor `unify_local_types` as a typed relation

Do not merely wrap the current `bool` and call every `false` `Refuted`.

The recursive relation itself needs enough information to classify failure.

For example:

```rust
enum LocalUnification {
    Proven,
    Refuted(LocalProofFailure),
    Blocked(BlockReason),
}
```

A useful policy is:

| Situation | Outcome |
|---|---|
| same terms | `Proven` |
| same constructor, recursively proven args | `Proven` |
| different known exact variants | `Refuted` |
| different known tuple/record/callable shape | `Refuted` |
| conflicting binding for same flexible parameter | `Refuted` |
| canonical non-parameter concrete mismatch | `Refuted` |
| unresolved/opaque local relation that engine cannot classify | `Blocked` |
| recursive/budget/cancelled proof | corresponding blocked terminal |
| impossible internal state | structured internal incident / fail-closed blocked result |

If `BlockReason` lacks a sufficiently precise reason, add one rather than using `Empty` as an error channel. Existing candidates such as:

```text
UnknownType(InferenceBlocked)
RecursiveFixpoint
InferenceDidNotConverge
BudgetExceeded
SuppressedDependency
```

may cover some paths, but a dedicated local-proof/coherence reason is preferable if this becomes a stable diagnostic boundary.

## 5.6 Required tests

At minimum:

```text
A2-R1 definitely impossible GADT case -> Refuted
A2-R2 compatible local rigid case -> Reachable
A2-R3 conflicting repeated local binding -> Refuted
A2-R4 deliberately blocked local proof -> Blocked
A2-R5 exact-case Blocked never becomes Empty
A2-R6 enum case Blocked never gets silently omitted
A2-R7 final coverage for blocked decomposition is ExhaustivenessResult::Blocked
A2-R8 canonical Universe readiness rejects Blocked
A2-R9 lowering rejects any non-Proven match
```

The key negative assertion is:

```text
proof inability
    MUST NOT
be observable as
    Empty / Impossible / Redundant / Proven
```

---

# 6. Finding A3 — negative trait-dispatch lookup has no stable dependency

**Severity:** High incremental correctness
**Status:** **OPEN**
**Confidence:** High from current source ownership; requires an end-to-end discriminator to prove the stale-reuse manifestation

This finding should be upgraded from the previous report's “absence-dependency test gap.”

After callable-body fingerprint narrowing, the broad aggregate conformance view is intentionally no longer part of every callable-body input fingerprint. That is a good optimization direction **only if every semantic lookup records exact positive or negative dependency products**.

Positive trait dispatch does.

Negative trait dispatch currently does not.

## 6.1 Current positive dependency path

In `phalcom-semantic/src/checker/context.rs`, successful evidenced trait dispatch records:

```rust
self.record_semantic_dependency(
    SemanticDependency::ConformanceDispatch(
        selection.source_impl.clone()
    )
);

self.record_semantic_dependency(
    SemanticDependency::TraitSurface(
        selection.requirement.owner.clone()
    )
);
```

Derived evidence also records its nested dependencies.

That is good for an already-existing selected conformance.

## 6.2 Current missing-result path

The same function has:

```rust
TraitDispatchResolution::Missing => None,
```

and the second trait-dispatch path similarly has:

```rust
TraitDispatchResolution::Missing => {}
```

No semantic dependency is recorded for the fact:

```text
"this target family / selector / side currently has no usable trait-dispatch candidate"
```

## 6.3 Why this matters more after fingerprint narrowing

The callable-body formal input fingerprint intentionally no longer hashes the aggregate:

```text
ConformanceSemanticView::fingerprint()
```

or the broad linked/workspace products.

That means this old accidental safety net is gone:

```text
any conformance set change
    -> body input fingerprint changes
```

The new architecture instead expects:

```text
only semantically consumed products
    -> dependency edges
    -> exact invalidation
```

That is correct—but absence itself must then be a first-class dependency.

## 6.4 Concrete stale-reuse scenario

Consider two modules.

### Revision N

Provider:

```phalcom
trait Printable {
    render() -> String
}

class Item {}
```

Consumer:

```phalcom
fn show(_ item: Item) {
    item.render()
}
```

Suppose ordinary inherent dispatch misses and trait dispatch also has no candidate.

The body observes:

```text
TraitDispatchResolution::Missing
```

and records no:

```text
ConformanceDispatch(...)
```

because no `ImplId` exists yet.

It also does not record a stable candidate-set/bucket dependency.

### Revision N+1

Provider adds:

```phalcom
impl Printable for Item {
    render() -> String {
        "item"
    }
}
```

A new:

```text
ConformanceDispatch(new_impl_id)
```

product now exists.

But the consumer from revision N cannot have a reverse edge to a query key that did not exist.

If no broader exact discovery dependency connects the cached consumer to the changed candidate set, the old body can remain reusable with the stale conclusion:

```text
no trait target
```

even though cold analysis now finds one.

This is the classic negative-cache invalidation problem.

## 6.5 The repository already has the correct stable semantic key shape

`phalcom-semantic/src/trait_dispatch.rs` defines:

```rust
pub struct TraitDispatchBucketKey {
    pub target_family:
        TraitDispatchTargetFamily,

    pub selector:
        Selector,

    pub side:
        DispatchSide,
}
```

and `TraitDispatchIndex` already maintains deterministic bucket contents:

```rust
BTreeMap<
    TraitDispatchBucketKey,
    Box<[TraitDispatchContribution]>
>
```

This is almost exactly the product identity needed for negative lookup dependencies.

The missing piece is that it is not currently a DB query product in `QueryKey`.

## 6.6 Correct architecture: empty-capable discovery product

Introduce a stable query/dependency product conceptually like:

```rust
QueryKey::TraitDispatchBucket(
    TraitDispatchBucketKey
)
```

with a product fingerprint computed from the sorted bucket contributions.

Critically, **empty is a valid published product**.

Then every ordinary trait-dispatch lookup does this before exact proof:

```text
receiver
    -> TraitDispatchTargetFamily

(selector, side)
    -> TraitDispatchBucketKey

record dependency on bucket
    -> even when bucket is empty

then:
    no candidates   -> Missing
    one/many        -> exact conformance proof
```

Positive selection still records:

```text
TraitDispatchBucket
ConformanceDispatch(selected impl)
TraitSurface(selected trait)
CallableSignature(selected default/witness as required)
derived nested proof dependencies
```

Negative selection records at least:

```text
TraitDispatchBucket(empty/current contents)
```

## 6.7 Publication/invalidation contract

The session should authoritatively publish bucket products from the current:

```text
ConformanceIndex
+
TraitSurfaceTable
```

A conformance add/remove/re-target/selector-affecting trait edit changes exactly the affected bucket fingerprint.

Then:

```text
empty bucket
    -> non-empty bucket
```

invalidates prior negative consumers.

Likewise:

```text
one candidate
    -> zero candidates

one candidate
    -> competing candidates
```

invalidates prior positive consumers.

This is much better than reintroducing:

```text
whole ConformanceSemanticView fingerprint
```

into every body, which would restore correctness at the cost of broad invalidation.

## 6.8 Required incremental regressions

Add exact cold-parity tests:

```text
A3-R1 Missing -> add exact conformance
    consumer recomputes
    trait dispatch becomes Found
    incremental == cold

A3-R2 Found -> remove conformance
    consumer recomputes
    trait dispatch becomes Missing
    incremental == cold

A3-R3 add unrelated conformance
    different target/selector bucket
    original consumer reuses product

A3-R4 Found -> add competing candidate
    consumer recomputes to ambiguity/coherence state

A3-R5 edit conditional applicability in same bucket
    affected consumer recomputes

A3-R6 remove/re-add same source impl
    retired query incarnation cannot resurrect stale body product
```

The unrelated-conformance test is important because it proves the repair preserves the intended incremental granularity.

---

# 7. Fixed finding — class-side runtime `self` and type-level `Self` are now separated

**Previous severity:** P1 semantic defect
**Current status:** **FIXED**

The new implementation matches the architecture prescribed in the previous audit.

## 7.1 Correct source-body model

For a generic covering class-side impl:

```phalcom
data Id<Tag>()

impl<Tag> Id<Tag> {
    @class
    direct() -> Int { 10 }

    @class
    throughReceiver() -> Int {
        self.direct()
    }

    @class
    make() -> Self {
        Id<Tag>()
    }
}
```

the checker now maintains:

```text
lowercase self
    -> class object Id

type-level Self
    -> source-relative instance Id<Tag>
```

Those are intentionally different.

## 7.2 Why the implementation is now correct

The direct and downstream paths both use:

```text
runtime_receiver_override(side)
```

which returns the source target only for instance-side methods.

For a class-side callable:

```text
self_type_override = None
```

so the ordinary checker class-object logic remains authoritative.

At the same time:

```text
self_instance_type_override = source_target_head
```

still specializes type annotations containing `Self`.

## 7.3 Conditional inherent impl path also fixed

The separate `impl_domain.head_type` override is now instance-only:

```rust
if callable.side == DispatchSide::Instance
    && let Some(domain) = &impl_domain
{
    ctx.self_override_type = Some(domain.head_type);
}
```

Therefore the same defect is not left behind for:

```phalcom
impl<T> Box<T>
where T <: Int
{
    @class
    ...
}
```

## 7.4 Regression quality is now good

The tests no longer rely only on:

```rust
!snapshot.has_errors()
```

They inspect:

```text
self expression semantic type
exact selected callable
body signature's type-level Self result
```

This closes both:

```text
previous implementation bug
previous false-positive test oracle
```

---

# 8. Fixed finding — callable-body definition dependency is now explicit

**Status:** **FIXED / materially improved**

The previous context-only permutation test could prove recomputation, but source dirtiness could independently force work, so it did not prove the exact dependency edge existed.

Current tests now directly inspect the semantic DB graph and require:

```text
CallableBody(callable)
    ->
CallableDefinition(callable)
```

That matters because `CallableDefinition` owns:

```text
accepted implementation provenance
source_member_index
public signature identity
InherentBodyContext
source target head / covering context
```

A body whose source text is unchanged but whose body-coordinate context changes must be invalidated through that semantic product.

This architecture is correct.

The remaining incremental issue in this report is not this edge; it is **negative trait-dispatch discovery**.

---

# 9. Fixed finding — stale fingerprint documentation

**Status:** **FIXED**

The callable-body fingerprint comment now accurately states that:

```text
broad workspace inputs are intentionally not hashed
exact dependencies own reuse validity
```

This aligns documentation with the intended A4-style fine-grained query architecture.

No further action is required on this specific comment.

---

# 10. Preserved good architecture — executable matches remain fail-closed

The defensive boundaries introduced by the earlier remediation remain correct.

## 10.1 Canonical Universe readiness gate

`phalcom-core/src/modules/canonical_universe.rs` still performs:

```rust
ensure_executable_match_coverage(
    &semantic.snapshot
)?;
```

before compiling the analyzed Universe.

The gate rejects both:

```text
ExhaustivenessResult::Missing(...)
ExhaustivenessResult::Blocked(...)
```

and allows only:

```text
ExhaustivenessResult::Proven
```

to proceed as executable match semantics.

## 10.2 Lowering remains defensive

`phalcom-core/src/modules/semantic_lowering.rs` retains:

```rust
if !matches!(
    resolution.exhaustiveness,
    ExhaustivenessResult::Proven
) {
    return Err(
        ProjectionError::NonProvenMatch(
            resolution.expression
        )
    );
}
```

This is the correct defense in depth.

## 10.3 Checker reports blocked analysis distinctly

`checker/expression.rs` still emits:

```text
MatchAnalysisBlocked
```

for:

```text
ExhaustivenessResult::Blocked
```

rather than describing it as a missing-pattern witness.

These boundaries should remain unchanged while A1/A2 are repaired upstream.

---

# 11. Remaining verification gap — recursive public-signature owner purity

**Severity:** Medium
**Status:** **OPEN verification/invariant gap**

The current implementation correctly materializes covering signatures through canonical machinery, and the tests already prove top-level binder ownership and shadowing.

However, the architectural invariant should be stronger:

> **An unconditional published inherent callable signature must contain no `TypeParameterOwner::Impl(current_impl)` anywhere in its durable public type graph.**

That needs to hold recursively through:

```text
Applied arguments
ExactCase enum type
AssociatedProjection subject/trait arguments
TypeMemberProjection subject/arguments
Tuple
RecordRow fields/tails
Callable parameter/result
Family members
TypePack / LabeledTypePack
Pack / tuple-from-packs / callable-from-packs
StaticData arguments
GenericConstraint operands
Conforms TraitRef arguments
```

Current debug/top-level checks do not constitute a complete durable proof of this invariant.

## 11.1 Correct solution

Do **not** add a second public-signature canonicalizer.

Instead, add or reuse one recursive parameter visitor:

```rust
fn collect_type_parameters_deep(
    store: &TypeStore,
    ty: TypeId,
    out: &mut BTreeSet<TypeParameterId>,
)
```

with companion traversal for:

```text
RecordRowId
StaticArgument
TypeTerm
TraitRef
GenericConstraint
CallableSemanticSignature
```

Then define an invariant helper:

```rust
fn signature_contains_parameter_owner(
    store: &TypeStore,
    signature: &CallableSemanticSignature,
    owner: &TypeParameterOwner,
) -> bool
```

Use it:

1. as a production `debug_assert!` after publishing an unconditional covering inherent signature;
2. in focused tests that intentionally place impl binders inside deeply nested carriers.

## 11.2 Required nested invariant examples

Test at least:

```phalcom
Option<Box<T>>
(T) -> Box<T>
Tuple-like nested generic
Record-row-bearing type
associated projection with T
type-member projection with T
constraint subject/trait argument using T
```

The invariant is about **owner identity**, not source spelling.

---

# 12. Remaining verification gap — end-to-end nested covering shapes

**Severity:** Medium/Low
**Status:** **OPEN verification gap**

Low-level tests now cover substantial structural traversal, especially RecordRow carriers. Source-level tests cover covering generic methods, `Self`, equality constraints, binder shadowing, and negative cases.

What is still comparatively thin is the complete source pipeline for deeper nested types:

```text
parse source
    ->
form impl-relative signature
    ->
publish declaration-canonical signature
    ->
materialize body source coordinates
    ->
specialize Self
    ->
bind callable-local generics
    ->
check body
```

A compact source-level matrix should exercise a few representative nested carriers rather than creating a huge Cartesian test suite.

Recommended cases:

```text
nested applied type
callable-valued parameter/result
tuple/record nested Self
associated projection
type-member projection
row-bearing structural carrier
```

The test should inspect the actual semantic products, not merely assert “no diagnostics.”

---

# 13. Current-head verification status

The current lifecycle documentation records focused verification after the latest corrective patch, including:

```text
RUSTFLAGS='' cargo check -p phalcom-semantic

RUSTFLAGS='' cargo test -p phalcom-semantic \
    --test semantic impls::bodies
    -> 18 passed

RUSTFLAGS='' cargo test -p phalcom-semantic \
    --test semantic impls::signatures
    -> 4 passed

RUSTFLAGS='' cargo test -p phalcom-semantic \
    --test semantic foundations::record_rows_field_keys
    -> 14 passed

git diff --check
```

Earlier project lifecycle records also retain focused `Result.transpose` runtime evidence.

However:

```text
GitHub workflow runs for ee7e8ee7: none
```

and this audit did not independently execute the full repository workspace or Hypothesis runtime harness.

Therefore the honest status is:

```text
focused implementation evidence exists
current-head broad/release CI certification is not established
```

That is a verification statement, not evidence that the focused fixes are wrong.

---

# 14. Updated finding matrix

| ID | Finding | Severity | Previous status | Current status |
|---|---|---:|---|---|
| OBS-01 | `Result.transpose` nested `Option` coverage fails | Critical observable | Failing incident | **FIXED behaviorally** |
| A1 | Match has private equality-normalization authority | High architecture | Open | **OPEN** |
| A2 | `open_variant_case -> Option` collapses proof states | **Soundness blocker** | Open | **OPEN** |
| A3 | Missing trait dispatch has no negative dependency product after fingerprint narrowing | **High incremental correctness** | Previously framed as test gap | **OPEN / elevated** |
| C1 | Class-side lowercase `self` overwritten by instance target | P1 | Open | **FIXED** |
| C2 | Conditional class-side domain override has same bug | P1 adjacency | Open | **FIXED** |
| T1 | Class-side distinction test had invalid `-> Self` oracle | High test defect | Open | **FIXED** |
| I1 | `CallableBody -> CallableDefinition` edge not directly proven | Medium | Gap | **FIXED / directly asserted** |
| I2 | Fingerprint documentation stale | Low | Open | **FIXED** |
| S1 | Generic constraint `Self` specialization | High | Fixed | **PRESERVED** |
| S2 | `Self` through trait-ref arguments | High | Fixed | **PRESERVED** |
| S3 | RecordRow recursive `Self` specialization | High | Fixed | **PRESERVED** |
| S4 | Binder shadowing / owner identity | High | Fixed | **PRESERVED** |
| S5 | `Self<T>` rejection | Medium | Fixed | **PRESERVED** |
| S6 | Exact `(ImplId, source_member_index)` body provenance | High | Preserved | **PRESERVED** |
| G1 | Recursive no-impl-binder public-signature invariant | Medium | Test gap | **OPEN** |
| G2 | Nested source-level covering matrix | Medium/Low | Test gap | **OPEN** |
| V1 | Current-head broad CI / release evidence | Medium verification | Uncertified | **UNCERTIFIED** |

---

# 15. Recommended implementation order

The remaining work should not be mixed into one large semantic patch. The three important open seams have distinct ownership boundaries.

## Phase 1 — make variant opening fail closed

**Priority:** highest because it is a proof-soundness boundary.

Change:

```text
LocalCaseProof = Option<...>
open_variant_case -> Option<...>
```

into typed outcomes and propagate:

```text
Blocked
    ->
DomainDecomposition::Blocked
    ->
ExhaustivenessResult::Blocked
```

without ever laundering inability into `Empty`.

This should be a compact, auditable patch.

## Phase 2 — establish one generic refinement authority

Add the canonical evidence-backed type refinement operation and migrate match root formation to it.

Delete:

```text
apply_ambient_equalities
```

once all current parameter equality regressions and the new associated-projection discriminator pass.

Do not make coverage itself a generic solver.

## Phase 3 — make trait-dispatch absence a query product

Introduce the stable empty-capable trait-dispatch bucket query/dependency.

Record it for:

```text
Found
Missing
Ambiguous
Incomplete
CoherenceConflict
Unknown
Blocked
Dynamic
Cancelled/Budget terminal paths where lookup had consumed the bucket
```

Then add add/remove/unrelated cold-parity incremental tests.

## Phase 4 — invariant hardening

Add:

```text
recursive public-signature owner-purity visitor
nested source covering matrix
```

These should not invent new semantic machinery.

## Phase 5 — certification

Run current-head:

```text
focused semantic match tests
covering impl tests
incremental trait-dispatch add/remove tests
canonical Universe bootstrap
semantic crate target
core relevant target
Hypothesis runtime harness
workspace / configured CI gates
```

and record exact command/result provenance.

---

# 16. Patch-grade guidance for A1

A practical implementation shape is:

```rust
// generic_evidence.rs or a closely owned canonical type-refinement module

pub enum LexicalTypeRefinement {
    Known(TypeId),
    Ambiguous,
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Recursive,
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

pub fn refine_type_under_generic_evidence(
    store: &mut TypeStore,
    ty: TypeId,
    evidence: &GenericEvidenceEnvironment,
    context: GenericRefinementContext<'_>,
) -> LexicalTypeRefinement
```

The algorithm should conceptually:

```text
1. normalize nested associated/type-member projections using existing
   generic projection normalization;

2. inspect equivalence closure for canonical terms participating in the
   current type;

3. build a deterministic parameter refinement environment only for
   uniquely established replacements;

4. materialize structurally through canonical TypeStore formation;

5. iterate only under an explicit bounded convergence policy;

6. if multiple incompatible refinements remain:
      Ambiguous / Blocked
   not:
      choose whichever term happens to sort first;

7. preserve symbolic unresolved terms when no refinement is justified.
```

Then the checker should form a coverage subject from ordinary semantic state:

```rust
let refined =
    ctx.refine_type_under_lexical_evidence(
        scrutinee_ty
    )?;

let subject =
    CoverageSubject::from_expression(
        refined,
        scrutinee_local_ty,
        ...
    );
```

The exact error plumbing can differ, but the architectural rule is:

> **Coverage receives a semantic type; it does not reconstruct generic equality semantics.**

---

# 17. Patch-grade guidance for A2

The minimum safe API transformation is:

```rust
pub(crate) enum LocalCaseProofOutcome {
    Reachable {
        bindings:
            BTreeMap<TypeParameterId, LocalType>,
        equalities:
            Box<[LocalConstraint]>,
    },

    Refuted,

    Blocked(BlockReason),
}

pub(crate) enum VariantOpenOutcome {
    Reachable(OpenedVariantCase),
    Refuted,
    Blocked(BlockReason),
}
```

`open_variant_case` becomes:

```rust
pub(crate) fn open_variant_case(
    ...
) -> VariantOpenOutcome
```

and must never use `?` on a proof operation whose failure semantics are not “proven impossible.”

The exact-case domain path becomes:

```rust
match open_variant_case(...) {
    VariantOpenOutcome::Reachable(opened) => {
        DomainDecomposition::Closed(
            Box::new([
                constructor_case(opened)
            ])
        )
    }

    VariantOpenOutcome::Refuted => {
        DomainDecomposition::Empty
    }

    VariantOpenOutcome::Blocked(reason) => {
        DomainDecomposition::Blocked(reason)
    }
}
```

The root-enum path becomes:

```rust
for variant in enum_variants {
    match open_variant_case(...) {
        Reachable(opened) => cases.push(...),
        Refuted => {}
        Blocked(reason) =>
            return DomainDecomposition::Blocked(reason),
    }
}
```

This is the semantic distinction the rest of the coverage engine is already prepared to consume.

---

# 18. Patch-grade guidance for A3

The cleanest incremental model is:

```rust
#[derive(
    Clone, Debug, Eq, Hash,
    Ord, PartialEq, PartialOrd
)]
pub struct TraitDispatchBucketKey {
    pub target_family:
        TraitDispatchTargetFamily,

    pub selector:
        Selector,

    pub side:
        DispatchSide,
}
```

This already exists.

Promote it into DB identity:

```rust
QueryKey::TraitDispatchBucket(
    TraitDispatchBucketKey
)
```

Add a semantic dependency variant if the checker dependency layer requires one:

```rust
SemanticDependency::TraitDispatchBucket(
    TraitDispatchBucketKey
)
```

Publish a deterministic product even for an empty contribution list:

```text
bucket key
    +
sorted TraitDispatchContribution[]
    ->
ProductFingerprint
```

Then trait lookup begins with:

```rust
let bucket_key =
    TraitDispatchBucketKey {
        target_family,
        selector: selector.clone(),
        side,
    };

ctx.record_semantic_dependency(
    SemanticDependency::TraitDispatchBucket(
        bucket_key.clone()
    )
);
```

before consulting candidates.

This establishes:

```text
absence is data
```

rather than:

```text
absence is the lack of a dependency edge
```

That principle is essential in a fine-grained incremental compiler.

---

# 19. Non-solutions to avoid

The remaining work should explicitly reject the following shortcuts.

## 19.1 Do not special-case `Result`, `Option`, or `transpose`

The current reproducer is now green. The goal is to make the semantic authority general, not to preserve this one example with another localized rule.

## 19.2 Do not re-solve generic constraints inside coverage

Coverage may consume:

```text
refined canonical type
local proof state
branch GADT evidence
```

but it should not own callable `where`-clause solving.

## 19.3 Do not map all local unification failures to `Refuted`

That simply renames the A2 soundness problem.

## 19.4 Do not map all local unification failures to `Blocked`

That is sound but loses valid impossible-branch reasoning and would degrade GADT usefulness/exhaustiveness unnecessarily.

The correct distinction is typed.

## 19.5 Do not restore the whole conformance aggregate fingerprint as the permanent A3 fix

That would likely recover correctness by making every conformance edit dirty too many bodies.

Use it only as a temporary emergency guard if needed.

The durable model is an empty-capable, stable discovery bucket.

## 19.6 Do not use source names to validate generic coordinate purity

The existing binder-shadowing tests correctly prove why:

```text
Declaration::T
Impl::T
Callable::T
```

must remain distinct despite identical spelling.

All validation should use stable binder identity.

---

# 20. Closure acceptance matrix

The broader audit can be marked complete only when the following matrix is green.

| ID | Acceptance requirement |
|---|---|
| AC-01 | `Result.transpose` outer and inner matches remain `Proven` |
| AC-02 | direct, reversed, and chained generic equality cases remain `Proven` |
| AC-03 | associated-projection equality can refine a match subject through the canonical evidence authority |
| AC-04 | `apply_ambient_equalities` is removed; match owns no generic-equality solver |
| AC-05 | ordinary semantic type and coverage-root canonical type agree under the same lexical evidence |
| AC-06 | constructor opening has explicit `Reachable / Refuted / Blocked` semantics |
| AC-07 | blocked local proof cannot become `DomainDecomposition::Empty` |
| AC-08 | blocked coverage cannot become `ExhaustivenessResult::Proven` |
| AC-09 | canonical Universe readiness rejects all non-proven executable matches |
| AC-10 | lowering retains `NonProvenMatch` defense |
| AC-11 | class-side lowercase `self` remains a class object in ordinary, covering, and conditional inherent bodies |
| AC-12 | type-level `Self` remains the source-relative instance type in those bodies |
| AC-13 | `CallableBody -> CallableDefinition` dependency remains present |
| AC-14 | prior Missing trait-dispatch lookup is invalidated when a matching conformance is added |
| AC-15 | prior Found trait-dispatch lookup is invalidated when its conformance is removed |
| AC-16 | unrelated conformance bucket edits do not recompute the consumer |
| AC-17 | incremental trait-dispatch mutations match cold semantic products |
| AC-18 | no impl-owned type parameter recursively escapes an unconditional published signature |
| AC-19 | nested covering source cases pass through publication/body-view round trip |
| AC-20 | current-head semantic/core/Hypothesis/workspace certification is recorded |

---

# 21. Final architectural ruling

The current repository is in a much better state than at the original incident.

The immediate chain:

```text
Result.transpose
    -> Result::Ok(option)
    -> T == Option<U>
    -> nested Some/None match
```

now produces the intended executable semantic result.

The C4.1 body-coordinate work also now has the correct `Self` split:

```text
runtime lowercase self
    !=
type-level Self
```

on class-side inherent implementations.

Those fixes should remain.

The remaining architecture should converge on three principles:

```text
1. Generic evidence has one normalization/refinement authority.

2. Proof refutation and proof inability are never represented
   by the same value.

3. In a fine-grained incremental compiler, semantic absence
   is represented by a stable dependency product.
```

Concretely:

```text
lexical generic evidence
    ->
one canonical type refinement authority
    ->
ordinary checker semantic type
    ->
coverage subject

variant opening
    ->
Reachable | Refuted | Blocked
    ->
finite-domain decomposition
    ->
exhaustiveness

trait dispatch discovery
    ->
stable bucket product, including empty
    ->
exact conformance proof
    ->
callable-body dependency graph
```

Once those three seams are closed and the current-head broad verification is recorded, this area can reasonably be treated as architecturally complete rather than merely incident-repaired.
