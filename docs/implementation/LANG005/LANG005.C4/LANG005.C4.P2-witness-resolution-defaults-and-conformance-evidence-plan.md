---
id: LANG005.C4.P2
category: LANG
program: LANG005
checkpoint: LANG005.C4
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C4.P1
  - LANG005.C3
  - LANG005.C2 applicability-proof-state closure
follows: LANG005.C4.P1
prepared: 2026-09-14
repository: aureat/phalcom-lang
repository_baseline: a7861a5b148715179ef81e2dfe82986a7d0fc499
repository_baseline_commit: "lang005: complete explicit conformance coherence"
planning_mode: live-post-C4.P1
next_plan: LANG005.C4.P3
---

# LANG005.C4.P2 — Witness Resolution, Defaults, Compatibility, and Conformance Evidence

## Luna Patch-Grade Implementation Plan

`LANG005.C4.P1` established the **source/conformance-head half** of explicit trait conformance:

```text
impl TraitRef for Target
    ↓
ImplId
    ↓
ResolvedConformanceHead
    ↓
authorization
    ↓
ConformanceContribution
    ↓
workspace ConformanceIndex
    ↓
coherence-safe exact head matching
    ↓
ConformanceHeadMatch
```

`LANG005.C4.P2` must implement the **requirement-satisfaction half**:

```text
ConformanceHeadMatch
    +
TraitSurface
    +
conformance-local witness declarations
    +
effective target behavior
        ↓
instantiate each TraitRequirementId
        ↓
resolve one compatible satisfaction source
        ↓
select trait defaults only where no concrete witness exists
        ↓
prove conformance completeness
        ↓
source-level ConformanceWitnessPlan
        ↓
exact specialization
        ↓
ConformanceEvidence
```

The plan's central architectural rule is:

> A source conformance is checked once as a generic proof template. Exact target/`TraitRef` applications instantiate that checked template into `ConformanceEvidence`; they do not independently re-run source witness selection.

This rule is required for generic conformances, incremental correctness, stable witness identity, and the P2→P3 execution boundary.

P2 is complete when the semantic system can answer:

> “For this exact target and exact `TraitRef`, which unique source conformance proves the relationship, which concrete/default behavior satisfies each `TraitRequirementId`, and what structured evidence records that proof?”

P2 is **not** complete merely because a unique P1 head exists, and it must not yet make trait-evidenced behavior participate in ordinary call/reference dispatch. That is C4.P3.

---

# 0. Executor contract

This plan is written for a Luna-class implementer operating under constrained architectural authority.

The implementer:

- must begin from the pushed P1 completion baseline or a descendant that preserves its semantic contracts;
- must read the final C4.P1 walkthrough and handoff before editing production code;
- may adapt helper names, exact file placement, and mechanical factoring to repository drift;
- must preserve the identity, ownership, proof-state, surface-separation, and source-vs-exact-evidence boundaries defined here;
- must treat the audited P1/C3 gaps in §6 as P2 entry work, not optional cleanup;
- must not merge conformance-local witness members into target inherent surfaces;
- must not represent successful conformance as a boolean;
- must not allocate one source witness callable per exact generic specialization;
- must not re-typecheck C3 trait defaults once per conformance;
- must not re-run a parallel applicability/type solver when C2/type-relation authority already exists;
- must not implement trait-evidenced ordinary dispatch, bound trait-evidenced references, runtime witness lookup, associated types, trait constraints, or conditional conformance in P2;
- must preserve cold/incremental parity for every new witness/evidence product;
- must keep tests narrow during BUILD mode and broaden only at explicit gates;
- must update the checkpoint/walkthrough/handoff at durable completion gates.

Testing is evidence gathering. During a local task, run the smallest test that can distinguish the current hypothesis. Do not repeatedly run the entire workspace after each semantic helper edit.

The executor may not self-waive any STOP/CONSULT trigger in this document.

---

# 1. Goal

Implement complete behavioral requirement satisfaction for explicit trait conformances by:

1. closing the live post-P1 identity/fingerprint/TraitRef-validation gaps that block sound witness construction;
2. introducing first-class conformance-local callable ownership for witness bodies;
3. publishing conformance-owned witness signatures and body analyses without reclassifying them as inherent members;
4. specializing C3 `TraitSurface` requirements under trait arguments, abstract `Self`, and implementation-local substitutions;
5. defining one canonical witness-compatibility procedure over the existing type relation and visibility authorities;
6. resolving reusable effective inherent witnesses, including inherited and C2 proven conditional/specialized behavior;
7. supporting readable data components as property witnesses without fabricating getter callables;
8. selecting explicit witnesses, concrete target witnesses, and trait defaults with deterministic precedence;
9. rejecting incomplete or semantically incompatible conformances;
10. publishing one source-level witness/completeness plan per conformance declaration;
11. instantiating that plan for exact target + exact `TraitRef` matches into structured `ConformanceEvidence`;
12. preserving explicit proof states for no-conformance, invalid/incomplete, unknown, blocked, dynamic, cancelled, budget-exhausted, and internal-failure outcomes;
13. integrating witness identities, signatures, source locations, dependencies, and evidence into incremental semantic products;
14. leaving P3 with an evidence object that can be consumed without re-solving witness/default selection.

---

# 2. Checkpoint slice acceptance objective

P2 accepts the following semantic flow:

```text
source:
    trait Printable {
        toString -> String
        describe -> String { self.toString }
    }

    class User {
        toString -> String { ... }
    }

    impl Printable for User {}
        ↓
P1 head:
    ConformanceContribution
    ImplId
    target template
    TraitRef template
        ↓
P2 source proof:
    instantiate TraitSurface under source domain
        ↓
    Printable.toString
        → compatible effective inherent User.toString

    Printable.describe
        → trait default Printable.describe
        ↓
ConformanceWitnessPlan
    source ImplId
    requirement selection templates
    completeness = complete
        ↓
exact query:
    User + Printable
        ↓
ConformanceHeadMatch
        ↓
instantiate witness plan
        ↓
ConformanceEvidence
    source_impl = ...
    exact_target = User
    exact_trait_ref = Printable
    requirement selections:
        toString → InherentCallable(User.toString)
        describe → TraitDefault(Printable.describe)
```

The following non-equalities are mandatory:

```text
ConformanceHeadMatch != ConformanceEvidence

TraitRequirementId != witness CallableId

trait default CallableId != default selection relation

conformance-local witness CallableId != target-owned inherent CallableId

source ConformanceWitnessPlan != exact ConformanceEvidence

exact ConformanceEvidence != ordinary trait-evidenced dispatch
```

P2 must stop after the last distinction. P3 owns ordinary availability, ambiguity across multiple traits, callable-reference selection, lowering, and execution.

---

# 3. Repository grounding

## 3.1 Live planning baseline

This plan is grounded against the pushed repository baseline:

```text
repository: aureat/phalcom-lang
revision: a7861a5b148715179ef81e2dfe82986a7d0fc499
commit: lang005: complete explicit conformance coherence
prepared: 2026-09-14
```

Immediate predecessor state:

```text
C3 completion:
    9f6c9b8df8c06671eee51551495aca3a3aa92bdc

C4.P1 completion:
    a7861a5b148715179ef81e2dfe82986a7d0fc499
```

The P1 handoff states `IMPLEMENTED + FOCUSED_TESTED` and explicitly leaves:

```text
witness selection
default selection
completeness
ConformanceEvidence
```

to P2.

## 3.2 Stable P1 inputs

P2 must consume, not replace:

```text
ImplKind::Conformance { trait_ref, for_range }
ImplDef.target

ImplId

TraitRef

ConformanceTarget
    Declaration(DeclarationId)
    ExactEnumCase(VariantId)

ResolvedConformanceHead

ConformanceContribution

ConformanceIndex

ConformanceIndex::query_exact

ConformanceHeadMatch {
    impl_id,
    exact_target,
    exact_trait_ref,
    impl_bindings,
}

ConformanceIndex::overlap_conflicts

trait-or-target ownership

C3 TraitHeaderTable
C3 TraitSurfaceTable
TraitRequirementId
trait-owned default CallableId
abstract Self representation
```

P2 must not rename a head match into evidence.

## 3.3 Verified C3 behavior relevant to P2

The landed C3 implementation establishes:

- traits are distinct declarations and are not nominal inhabitable type forms;
- trait generic parameters are declaration-owned;
- `TraitRef` is a distinct contract reference;
- `TraitRequirementId` remains stable independently of default presence;
- each trait behavior member publishes a canonical semantic signature;
- bodyful trait members are the same requirement plus a trait-owned default callable;
- complete `TraitSurface` publication precedes default body checking;
- trait defaults are checked once under abstract `Self`;
- abstract requirement/default calls can be semantically resolved without executable runtime targets;
- trait surfaces do not enter ordinary inherent dispatch surfaces.

P2 must preserve all of these.

---

# 4. Required reads before implementation

Read these in order before T1.

## 4.1 Normative language authority

1. `docs/specs/objects/traits.md`
2. `docs/specs/objects/impl.md`
3. `docs/specs/objects/enums.md`
4. authoritative callable/member specification
5. authoritative visibility/access-control specification
6. relevant generic-callable and type-relation specifications

## 4.2 LANG005 checkpoint state

7. `docs/implementation/LANG005/LANG005.C3/LANG005.C3-CHECKPOINT.md`
8. `docs/implementation/LANG005/LANG005.C3/LANG005.C3-GUIDANCE.md`
9. C3 final walkthrough/handoff
10. `docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md`
11. `docs/implementation/LANG005/LANG005.C4/LANG005.C4-GUIDANCE.md`
12. `LANG005.C4.P1-explicit-conformance-and-coherence-plan.md`
13. `LANG005.C4.P1-walkthrough.md`
14. `LANG005.C4.P1-handoff.md`
15. this P2 plan

## 4.3 Production architecture

Inspect the live equivalents of:

```text
phalcom-semantic/src/identity.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/surface.rs
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/source_index/
phalcom-semantic/src/db/
phalcom-core semantic lowering consumers
```

The compiler/runtime files are audit-only during most of P2. Production runtime behavior belongs to P3 unless a mechanical identity match arm is required to keep the workspace compiling.

## 4.4 Support fixtures

Read as forward-design references:

```text
docs/implementation/LANG005/support/core_trait_impls.ph
docs/implementation/LANG005/support/iterable.ph
```

Do not pull C5 associated-type syntax or C6 trait-conditioned conformance into P2 merely because support files contain them.

---

# 5. Authority and conflict resolution

Use this precedence:

```text
1. explicit user-ratified LANG005 decisions
2. current authoritative docs/specs/ language specifications
3. C4 checkpoint and guidance
4. C4.P1 landed walkthrough/handoff contracts
5. C3 landed walkthrough/handoff contracts
6. live production architecture
7. this plan
8. historical plans/support examples
```

If this plan conflicts with a higher authority, STOP AND CONSULT.

Mechanical drift is not a conflict. Adapt naming and helper placement when the live repository already supplies the same semantic responsibility differently.

---

# 6. Entry audit findings and mandatory corrective closure

The post-P1 audit identified three proven correctness gaps and several required seams. P2 owns their closure because witness/evidence semantics cannot be sound without them.

## P2-A01 — HIGH — conformance-local callable identity is absent

Current callable owner identity supports only:

```text
CallableOwnerId::Declaration(DeclarationId)
CallableOwnerId::Variant(VariantId)
```

A witness body inside:

```phalcom
impl Printable for User {
  toString -> String { ... }
}
```

is neither target-owned inherent behavior nor trait-owned behavior.

P2 must introduce a canonical conformance-local ownership form, expected conceptually as:

```rust
CallableOwnerId::Conformance(ImplId)
```

or the exact repository-equivalent.

One source conformance has one source witness callable identity even when the conformance is generic and applies to many exact targets.

### Required invariant

```text
impl<T> Tagged for Value<T> {
    tag -> String { ... }
}

one ImplId
one conformance-owned tag CallableId

Value<Int> evidence
    → same source CallableId + T := Int

Value<String> evidence
    → same source CallableId + T := String
```

Do not create one callable identity per exact target specialization.

## P2-A02 — HIGH — semantic-shard conformance-member fingerprint asymmetry

The live initial semantic-shard construction still fingerprints all `Statement::Impl` members through the target-owned impl-member helper.

The body-refresh path, by contrast, explicitly fingerprints only `ImplKind::Inherent`.

That creates a cold/incremental asymmetry which was harmless only while P1 intentionally treated conformance bodies as inert.

P2 must replace this with explicit ownership-correct behavior:

```text
inherent impl member
    → target-owned CallableId signature/body fingerprints

conformance witness member
    → Conformance(ImplId)-owned CallableId signature/body fingerprints

conformance head
    → separate structural/head fingerprint
```

Body-only conformance edits must not change head identity.

## P2-A03 — HIGH — P1 conformance TraitRef formation bypasses trait generic constraints

The C3 checked `TraitRef::form` validates trait generic constraints.

The landed P1 conformance-head resolver currently resolves trait arguments, checks arity/kind, then directly constructs `TraitRef::new(...)`.

Therefore P2 must close the gap before evidence can be constructed.

Concrete invalid applications such as:

```phalcom
trait NumericTag<T> where T <: Number {
  tag -> String
}

class Item {}

impl NumericTag<String> for Item {}
```

must never become evidence-eligible.

P2 must use or factor one proof-state-aware checked TraitRef-formation route suitable for conformance heads.

### C4 boundary for symbolic constraints

C4 supports generic source heads but does not support trait-conditioned/conditional conformance.

Therefore:

```phalcom
impl<T> NumericTag<T> for Box<T> {}
```

is acceptable only when the trait application constraints are provable for the entire source conformance domain under already-established non-trait generic facts.

If proof depends on a future `T: SomeTrait` obligation or another C6-only condition, C4 must reject/defer it rather than assume success.

## P2-A04 — HIGH — source index intentionally skips conformance witnesses

P1 source indexing records the trait and target occurrences then stops for conformance bodies.

P2 must publish normal source identities for conformance-local witness callables, parameters, type references, bodies, and local scopes using the new conformance callable owner.

## P2-A05 — HIGH — no canonical instantiated trait requirement view

Raw C3 `TraitSurfaceMember` signatures still contain:

```text
trait declaration parameters
abstract Self
requirement-local callable generics
```

P2 must add a reusable specialization view before witness compatibility.

## P2-A06 — HIGH — no unified effective inherent witness query

P2 must not implement independent lookup logic for:

```text
direct members
inherent impl members
inherited members
exact-case members
C2 conditional/specialized members
```

Factor/consume one canonical inherent-only witness lookup.

## P2-A07 — HIGH — no structured witness compatibility outcome

Compatibility cannot be a boolean.

It must preserve at least:

```text
compatible
incompatible/refuted
unknown
blocked
dynamic
cancelled
budget exceeded
internal failure
```

Only proven compatibility may satisfy a trait requirement.

## P2-A08 — MEDIUM — exact conformance query returns all matches

`ConformanceIndex::query_exact` returns `Vec<ConformanceHeadMatch>` deliberately.

P2 must handle:

```text
0 matches
1 match
2+ matches
```

explicitly and never use `.first()` as implicit precedence.

## P2-A09 — MEDIUM — conformance lookup is still scan-based

P1 exact lookup and overlap checking are correctness-first and not yet optimized for P3 high-frequency ordinary lookup.

This is not an early P2 blocker, but P2 must not bake scan-based lookup into the future dispatch API.

If a bounded family index is straightforward while introducing evidence caching, add it before P2 handoff. Otherwise record it as an explicit P3 entry requirement.

---

# 7. Architecture

## 7.1 Source proof first, exact evidence second

P2 must not perform witness resolution from scratch every time an exact conformance query is issued.

Required architecture:

```text
source ConformanceContribution
    +
source impl generic environment
    +
TraitSurface
    +
source witness declarations
    +
target effective behavior templates
        ↓
ConformanceWitnessPlan
        ↓
ConformanceHeadMatch
    +
exact impl bindings
        ↓
ConformanceEvidence
```

Conceptually:

```rust
pub struct ConformanceWitnessPlan {
    pub impl_id: ImplId,
    pub target_template: TypeId,
    pub trait_ref_template: TraitRef,
    pub requirements:
        BTreeMap<TraitRequirementId, RequirementSelectionTemplate>,
    pub completeness: ConformanceCompleteness,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}
```

Names may differ.

The exact product is conceptually:

```rust
pub struct ConformanceEvidence {
    pub source_impl: ImplId,
    pub exact_target: TypeId,
    pub exact_trait_ref: TraitRef,
    pub impl_environment: TypeEnvironment,
    pub requirements:
        BTreeMap<TraitRequirementId, RequirementSelection>,
}
```

C5 may later extend exact evidence with associated-type bindings.

C6 may later extend it with nested trait-constraint evidence.

P2 must not pre-fill speculative placeholders for those future features.

## 7.2 Why source-level planning is mandatory

Given:

```phalcom
impl<T> Tagged for Value<T> {
  tag -> String { "tag" }
}
```

the body and witness relationship must be analyzed once under the source generic environment.

Exact applications:

```text
Value<Int> + Tagged
Value<String> + Tagged
```

specialize that proof; they do not create two independently body-checked witness declarations.

This provides:

```text
stable source callable identity
bounded semantic work
incremental locality
correct generic semantics
deterministic P3 lowering inputs
```

## 7.3 No inherent-surface contamination

Neither:

```text
ConformanceWitnessPlan
ConformanceEvidence
```

may insert conformance-local witness callables or selected defaults into:

```text
DeclarationSurface
ConditionalInherentMemberSet
target runtime class method table as semantic authority
```

P3 may make selected trait behavior *available* through ordinary syntax. Availability is not ownership.

---

# 8. Conformance-local callable ownership

## 8.1 Required identity extension

Preferred model:

```rust
pub enum CallableOwnerId {
    Declaration(DeclarationId),
    Variant(VariantId),
    Conformance(ImplId),
}
```

The exact representation may differ only if it preserves the same semantic identity.

Update:

```text
Hash
Ord
Eq
module()
debug/presentation
stable semantic fingerprinting
source indexing
query keys
callable tables
```

mechanically.

## 8.2 `declaration_owner()` is no longer total

The repository currently has many call sites where:

```text
callable.declaration_owner()
```

means different things.

After conformance-owned callables exist, audit each use and classify it as one of:

```text
A. true lexical declaration owner
B. receiver/nominal target owner
C. generic owner
D. class/private access context
E. field/storage owner
F. source module owner
G. metadata/presentation owner
```

Do not implement:

```rust
CallableOwnerId::Conformance(impl_id)
    .declaration()
    -> target declaration
```

merely to keep the old total API alive. That would erase the ownership distinction P2 is introducing.

Preferred migration:

```text
callable.module()
    remains total

callable.declaration_owner()
    becomes optional / declaration-only
    or receives an explicitly named compatibility helper

conformance target declaration
    is obtained from conformance semantic context,
    not fabricated as lexical owner
```

## 8.3 Conformance witness body context

For a conformance-local witness:

```phalcom
impl<T> Trait<X> for Target<T> {
  member<U>(_ value: U) -> ...
}
```

body analysis must have explicit fields for:

```text
current callable
    Conformance(ImplId)::member

source/lexical module
    impl declaration module

Self / receiver type
    Target<T> source template

owner generic signature
    TypeParameterOwner::Impl(ImplId)

callable generic signature
    TypeParameterOwner::Callable(conformance-owned CallableId)

trait context
    exact/template TraitRef where needed for signature checking

access-control enclosing class
    NONE unless a separate normative rule explicitly grants it
```

### Security/correctness invariant

A conformance declared by the trait-owning module for a foreign target must not gain target-private access merely because `Self` is the target.

Therefore:

```text
Self target identity
    !=
class private/protected lexical privilege
```

This must have a negative test.

---

# 9. Conformance witness declaration publication

## 9.1 Only behavior members are legal

P2 source witness declarations reuse the shared `BehaviorMember` grammar:

```text
method
getter
setter
index getter
index setter
```

They are not an arbitrary helper namespace.

Every conformance body member must map to one `TraitRequirementId`.

## 9.2 Explicit witness mapping

For every conformance member:

1. derive its selector and dispatch role;
2. look up the corresponding C3 requirement in the trait surface;
3. reject no-match members;
4. reject duplicates targeting the same requirement;
5. require a concrete body for an explicit witness;
6. publish one conformance-owned canonical signature;
7. analyze its body in the context from §8.3;
8. retain source provenance and diagnostics.

A malformed explicit witness must not silently disappear and allow default fallback.

## 9.3 Bodyless conformance member policy

C4 conformance declarations establish concrete evidence.

A bodyless member written inside a conformance does not create a second abstract requirement mechanism.

Required behavior:

```text
bodyless conformance member
    → diagnostic
    → not a witness
    → conformance remains incomplete/invalid
```

Do not interpret it as “repeat the trait requirement.”

## 9.4 Signature construction

Conformance witness signature resolution must have access to:

```text
impl-local generic binders
target-relative Self
member-local generic binders
ordinary imported type names
trait requirement being satisfied
```

Do not resolve the witness signature as if it were target-declaration-owned.

---

# 10. Instantiated trait requirement view

## 10.1 Required specialization

Given:

```phalcom
trait Comparable<Rhs> {
  compare(_ other: Rhs) -> Ordering
  clone -> Self
}
```

and source/exact conformance context:

```text
Target = Meter
TraitRef = Comparable<Meter>
```

P2 needs:

```text
Rhs  := Meter
Self := Meter
```

while preserving requirement-local callable binders.

Introduce a reusable product/view conceptually like:

```rust
pub struct InstantiatedTraitRequirement {
    pub requirement: TraitRequirementId,
    pub source_callable: CallableId,
    pub signature: CallableSemanticSignatureOrSpecializedView,
    pub visibility: MemberVisibility,
    pub default_callable: Option<CallableId>,
}
```

Do not mutate the canonical `TraitSurface`.

## 10.2 Specialization order

Use one deterministic specialization pipeline:

```text
canonical trait member signature
    ↓
bind trait declaration parameters
    from TraitRef arguments
    ↓
specialize abstract Self
    to source/exact conformance target
    ↓
apply impl target bindings where target/trait arguments contain impl parameters
    ↓
preserve member-local generic parameter identity
```

For a source generic witness plan, the result may still contain `TypeParameterOwner::Impl(ImplId)` parameters.

For exact evidence, those become exact target types through `ConformanceHeadMatch.impl_bindings`.

## 10.3 Reuse canonical specialization machinery

Factor/reuse:

```text
TypeEnvironment
TypeSubstitution
TypeView
specialize_self_type
receiver specialization
bounded relation engine
```

Do not create a trait-specific type graph or string substitution layer.

---

# 11. Canonical witness compatibility

## 11.1 Result type

Required conceptual result:

```rust
pub enum WitnessCompatibility {
    Compatible(WitnessCompatibilityProof),

    Incompatible(WitnessMismatch),

    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),

    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(String),
}
```

Names may differ.

Only `Compatible` may satisfy a requirement.

## 11.2 Compatibility inputs

Compatibility compares:

```text
instantiated trait requirement
candidate witness signature/capability
candidate visibility/access
candidate origin
ambient generic/proof environment
```

## 11.3 Selector and role

The candidate must satisfy exact member-role identity:

```text
method != getter
getter != setter
index getter != index setter
instance != class side
```

Selector shape remains Phalcom's canonical selector authority.

Types do not create new selector identity; they decide compatibility after selector match.

## 11.4 Callable parameter/result substitutability

Do not use exact type equality except where the general callable contract requires it.

Conceptually:

```text
required parameter R
witness parameter W

R <: W
```

because callers permitted by the requirement must be accepted by the witness.

For returns:

```text
witness return WR
required return RR

WR <: RR
```

Use the repository's existing callable/type relation authority and proof states.

## 11.5 Generic callable requirements

For requirement-local generics:

```phalcom
trait Mapper<T> {
  map<U>(_ value: U) -> U
}
```

the witness's callable generic contract must be at least as general as the requirement.

Required invariants:

```text
binder spelling does not matter
kind compatibility is semantic
constraint compatibility is semantic
the witness may not strengthen caller obligations
unresolved generic proof is not success
```

If the existing repository lacks one reusable signature-substitutability helper, P2 may factor one over the existing generic/type relation engine.

Do not introduce a parallel inference solver.

## 11.6 `Self`

All `Self` occurrences in the requirement must be specialized to the conformance target before comparison.

A conformance-local witness body itself is checked with target-relative `Self`.

The trait's source/default callable remains abstract and trait-owned.

## 11.7 Visibility/access coverage

Required rule:

```text
access_set(requirement)
    ⊆
access_set(witness)
```

Do not compare `MemberVisibility` by enum ordinal.

If no canonical access-set helper exists, establish one in the visibility/access layer and use it from witness compatibility.

Authorization to declare the conformance does not grant broader witness visibility.

---

# 12. Effective inherent witness lookup

## 12.1 One authority

Introduce or expose one P2-facing query:

```text
resolve_effective_inherent_witness(
    receiver/source target domain,
    selector,
    side,
    proof context
)
```

The exact name may differ.

It must cover existing canonical behavior sources rather than scanning syntax.

## 12.2 Candidate sources

The query must be capable of selecting:

```text
direct target callable
covering inherent impl callable
C2 specialized inherent impl callable
C2 constrained/conditional callable when proven
inherited superclass callable
exact enum-case callable
enum-root behavior where existing enum dispatch semantics permit it
```

## 12.3 Required returned data

A successful candidate should preserve:

```text
CallableId
specialized canonical signature
visibility/access facts
receiver specialization environment
C2 conditional selection/applicability evidence if relevant
declaration/source provenance
proof status
```

P2 compatibility consumes this product.

## 12.4 Inherent-only recursion boundary

This lookup must remain **inherent-only**.

Once P3 adds trait-evidenced ordinary member availability, P2 conformance construction must not recursively use behavior obtained from another trait conformance as if it were inherent evidence.

Required architectural distinction:

```text
witness search in P2
    direct/inherent/inherited/data/default

ordinary member search in P3
    inherent
    + trait-evidenced
```

Keep separate APIs if necessary to make recursion impossible.

---

# 13. C2 conditional/specialized behavior as witness evidence

P2 must consume C2 proof state; it must not re-solve C2 applicability from syntax.

For a source generic conformance:

```phalcom
impl<T> Tagged for Value<T> {}
```

and conditional inherent behavior:

```phalcom
impl<T> Value<T> where T <: Number {
  tag -> String { ... }
}
```

the unconditional generic conformance cannot rely on `tag` unless the C2 applicability condition is proven throughout the conformance's whole source domain.

Since C4 does not yet support trait-conditioned conditional conformance, an unproven conditional witness must not produce source completeness.

For an exact conformance:

```phalcom
impl Tagged for Value<Int> {}
```

the same conditional inherent member may be used when C2 proves it for `Value<Int>`.

Required proof behavior:

```text
Applicable/Proven
    → candidate witness

NotApplicable/Refuted
    → no candidate

Unknown
    → witness selection unknown / conformance not proven

Blocked
    → blocked

Dynamic
    → dynamic boundary, never static conformance proof

Cancelled/BudgetExceeded/InternalFailure
    → preserve terminal state
```

Do not collapse terminal states into “missing member.”

---

# 14. Data-component witnesses

## 14.1 Readable property requirement

For:

```phalcom
trait Named {
  name -> String
}

data Person(name: String)

impl Named for Person {}
```

the evidence may select:

```text
RequirementSelection::DataComponent(Person.name)
```

The component retains `DataComponentId`.

Do not synthesize a getter `CallableId`.

## 14.2 Generic component specialization

For:

```phalcom
data Box<T>(value: T)

trait Valued<T> {
  value -> T
}

impl<T> Valued<T> for Box<T> {}
```

specialize the component's declared type through the source/exact target environment before compatibility.

## 14.3 Mutability

Immutable data component:

```text
may satisfy compatible getter/read requirement
must not satisfy setter/write requirement
```

If future mutable component semantics exist, use their canonical capability product rather than inferring mutability from syntax text.

## 14.4 Collision coherence

P2 should rely on existing C2/data selector-reservation semantics so a data component and concrete callable cannot independently compete for one target selector.

Do not invent a P2 source-order tie-break.

---

# 15. Inherited witnesses

An inherited callable may witness a requirement.

For:

```phalcom
class Base {
  print -> String { ... }
}

class Child is Base {}

impl Printable for Child {}
```

`Base.print` may be selected if ordinary inherent dispatch for `Child` resolves to that callable and compatibility is proven.

If `Child` overrides the selector, normal inherent dispatch decides the effective concrete callable before P2 compatibility checking.

Required invariant:

```text
class inheritance
    determines effective inherent callable

trait conformance
    records witness relationship

the two mechanisms remain independent
```

Conformance itself does not implicitly propagate from `Base` to `Child` in C4 unless a separately ratified rule says so.

---

# 16. Exact enum-case witnesses

P2 must preserve `VariantId` and the exact case type environment from P1/C2.

For:

```phalcom
impl Trait for Result<Int>::Ok(_) {
  ...
}
```

requirement instantiation and candidate lookup must not collapse the target to `Result<Int>`.

Case-specific/GADT specialization may affect:

```text
Self
member signatures
payload-aware case behavior
inherent exact-case witness applicability
```

Use the existing enum/exact-case semantic products.

Test enum-root and exact-case conformances separately.

---

# 17. Explicit witness precedence and failure policy

For one requirement:

```text
explicit conformance-local declaration?
    ↓ yes
validate as the witness
```

If the explicit declaration is malformed/incompatible, diagnose it and fail that requirement.

Do **not** silently fall through to:

```text
existing inherent member
trait default
```

after an explicit attempted witness fails.

This makes source intent deterministic and avoids hiding errors.

A valid explicit conformance witness supersedes a trait default.

---

# 18. Concrete witness vs trait default precedence

When no explicit conformance-local witness exists:

```text
compatible effective concrete target witness
    ↓
selected
```

Only when no compatible concrete witness exists may P2 select the trait default.

Required order:

```text
explicit conformance-local witness
    otherwise
compatible effective concrete target witness
    otherwise
compatible data/property capability
    otherwise
trait default
    otherwise
missing requirement
```

Data property capability may be integrated into effective target capability lookup; the semantic outcome must remain a `DataComponentId` selection.

A trait default is not copied into the target's inherent surface and does not become target-owned.

---

# 19. Trait default selection

## 19.1 C3 analysis is authoritative

C3 already checks a default body once under:

```text
trait generic parameters
abstract Self
complete TraitSurface
```

P2 must reuse that checked default.

Do not body-check the trait default again for each conformance.

## 19.2 Selection product

A default selection records:

```text
TraitRequirementId
    → trait-owned default CallableId
```

plus whatever specialization environment P3 will need.

It does not create a new source callable.

## 19.3 Invalid C3 default

If the canonical trait default already has semantic body errors, preserve those diagnostics as trait declaration errors.

Do not create successful executable conformance evidence which depends on a default whose semantic contract/body is unavailable or invalid under the repository's callable-validation policy.

The exact propagation rule should follow the existing callable analysis status model.

---

# 20. Requirement selection model

Use one semantic representation for all successful behavioral satisfaction sources.

Conceptually:

```rust
pub enum RequirementSelectionTemplate {
    ConformanceCallable {
        callable: CallableId,
    },

    InherentCallable {
        callable: CallableId,
        specialization: ReceiverSpecializationOrEquivalent,
        applicability: Option<CanonicalApplicabilityEvidence>,
    },

    DataComponent {
        component: DataComponentId,
        specialized_type: TypeIdOrView,
    },

    TraitDefault {
        callable: CallableId,
    },
}
```

The exact specialized `RequirementSelection` may replace symbolic template fields with exact `TypeId`/environment values.

Do not encode source kinds as strings.

Do not convert data components/defaults into fake target-owned callable identities for uniformity.

---

# 21. Conformance completeness

A source conformance is behaviorally complete in C4 when every C3 behavioral requirement has one proven selection.

C5 associated types are not part of this completeness definition yet.

Required conceptual state:

```rust
pub enum ConformanceCompleteness {
    Complete,
    Incomplete {
        failures: Box<[RequirementFailure]>
    },
    Unknown(...),
    Blocked(...),
    Dynamic(...),
    Cancelled,
    BudgetExceeded(...),
    InternalFailure(...),
}
```

Names may differ.

Only `Complete` can later instantiate `ConformanceEvidence`.

Do not publish an evidence object with a `complete: false` flag.

---

# 22. Generic source conformances are universal over their declared domain

This is a hard correctness rule.

For:

```phalcom
impl<T> Tagged for Value<T> {}
```

the source conformance claims the relationship for every exact target in the P1 head domain, not merely for exact applications that happen to find a compatible witness later.

Therefore the `ConformanceWitnessPlan` must be established against the symbolic source domain.

It is invalid to implement:

```text
query Value<Int>
    → witness exists
    → produce evidence

query Value<String>
    → witness missing
    → no evidence
```

for one unconditional source conformance whose P1 head covers both.

Either the source conformance is universally complete over its C4 domain, or it is invalid/incomplete.

Future C6 conditional conformance can narrow that domain using trait/generic proof conditions.

P2 must not simulate C6 by producing exact evidence opportunistically.

---

# 23. TraitRef constraint validation closure

## 23.1 One checked formation path

Factor or expose a checked trait-reference formation operation usable from:

```text
ordinary C3 TraitRef formation
C4 conformance-head formation
future C5/C6 consumers
```

It must validate:

```text
declaration is a trait
arity
kind
canonical argument validity
trait generic constraints
proof-state outcome
```

## 23.2 Concrete refutation

A concrete violated constraint is a diagnostic and lookup/evidence ineligibility.

## 23.3 Symbolic source domain

For source generic parameters, use the ambient impl/declaration generic constraints as assumptions only where the type-system authority can prove the trait's own constraints.

Do not treat parameter name equality or syntactic similarity as proof.

## 23.4 C6 boundary

If validity depends on trait-conformance constraints or conditional conformance not yet modeled in C4, diagnose/defer rather than guessing.

---

# 24. Source-level `ConformanceWitnessPlan`

## 24.1 Product ownership

One source `ImplId` owns at most one source witness plan.

The plan should be snapshot-owned and incrementally replaceable/removable by source module.

## 24.2 Product contents

At minimum retain:

```text
ImplId
source module/provenance
target template
TraitRef template
impl generic signature/environment
per-TraitRequirementId selection template
completeness/proof status
diagnostics
semantic dependencies/fingerprint inputs
```

## 24.3 What it must not contain

Do not store:

```text
runtime function pointers
runtime class-table entries
arbitrary editor-only reconstructed syntax
one exact instance per possible generic application
associated-type bindings
nested trait-bound evidence
```

## 24.4 Diagnostics authority

The plan is the natural owner for conformance-body and requirement-satisfaction diagnostics.

P1 `ConformanceContribution` remains head/provenance authority.

---

# 25. Exact `ConformanceEvidence`

## 25.1 Resolution entry

Expose one semantic resolution operation conceptually:

```text
resolve_conformance_evidence(
    exact_target,
    exact_trait_ref,
    query budget/control
)
```

## 25.2 Resolution sequence

Required order:

```text
1. query P1 ConformanceIndex::query_exact

2. branch:
    0 matches
        → NotDeclared

    1 match
        → continue

    2+ matches
        → CoherenceConflict
        → never choose first

3. load source ConformanceWitnessPlan

4. require source plan Complete

5. construct exact TypeEnvironment from ConformanceHeadMatch.impl_bindings

6. specialize each RequirementSelectionTemplate

7. publish ConformanceEvidence
```

Do not re-run witness selection in step 6.

## 25.3 Evidence contents

Conceptually:

```rust
pub struct ConformanceEvidence {
    pub source_impl: ImplId,
    pub exact_target: TypeId,
    pub exact_trait_ref: TraitRef,
    pub impl_environment: TypeEnvironment,
    pub requirements:
        BTreeMap<TraitRequirementId, RequirementSelection>,
}
```

Optionally retain direct references to the source plan/fingerprint if the repository's query architecture benefits.

## 25.4 Resolution result

Use a structured result, conceptually:

```rust
pub enum ConformanceResolution {
    Proven(Arc<ConformanceEvidence>),

    NotDeclared,
    InvalidSource(ImplId),
    Incomplete(ImplId),
    CoherenceConflict(Box<[ImplId]>),

    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),

    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(String),
}
```

Only `Proven` authorizes later P3 trait-evidenced behavior.

---

# 26. Exact evidence identity and generic specialization

## 26.1 Generic source conformance

For:

```phalcom
impl<T> Tagged for Value<T> {
  tag -> String { "generic" }
}
```

required relationships:

```text
source ImplId:
    same

source witness CallableId:
    same

Value<Int> evidence:
    exact_target = Value<Int>
    impl environment T := Int

Value<String> evidence:
    exact_target = Value<String>
    impl environment T := String
```

## 26.2 Specialized source conformances

For:

```phalcom
impl Tagged for Value<Int> {
  tag -> String { "int" }
}

impl Tagged for Value<String> {
  tag -> String { "string" }
}
```

required relationships:

```text
different ImplId
different conformance-owned witness CallableId
different exact target
same TraitRequirementId
same trait declaration
```

Both variants need explicit tests because they prove different identity dimensions.

---

# 27. Body analysis vs witness compatibility

Do not conflate:

```text
declaration signature compatibility
body semantic correctness
```

A callable may have a source-declared signature suitable for witness compatibility while its body separately has an error.

Follow the repository's existing callable contract/body validation semantics.

Important incremental rule:

```text
body-only edit
    never changes:
        ImplId
        conformance head
        conformance-owned CallableId

body-only edit
    may change:
        CallableAnalysis
        inferred return knowledge
        witness compatibility/evidence
            IF the canonical signature depends on body inference
```

If a fully annotated witness's canonical contract is unchanged, body-only edits should not force head or compatibility recomputation beyond required body validation dependencies.

---

# 28. Incremental semantic architecture

## 28.1 Fingerprint categories

P2 should distinguish at least:

```text
conformance head fingerprint
    trait ref / target / impl generics / supported constraints

conformance witness signature fingerprint
    member selector/generics/annotations/visibility

conformance witness body fingerprint
    executable body

source witness plan fingerprint
    head identity
    TraitSurface contract fingerprint
    effective witness-source dependencies
    witness compatibility results/default availability

exact evidence fingerprint
    source witness-plan fingerprint
    exact ConformanceHeadMatch bindings
```

## 28.2 Required invalidation causes

Recompute affected source witness plans/evidence when:

```text
trait requirement added/removed/changed
trait default added/removed/changed where selected or candidate
conformance witness added/removed/signature changed
relevant conformance witness body changes inferred contract
target inherent member added/removed/signature changed
target hierarchy changes effective inherited witness
C2 conditional/specialized applicability changes
data component signature/capability changes
visibility/access facts change
conformance head changes
module removed/replaced
```

## 28.3 Non-causes where possible

Do not invalidate unrelated evidence merely because:

```text
unrelated module body changed
unrelated trait changed
unrelated target member changed
unrelated conformance changed outside candidate/witness dependency
```

Follow existing semantic dependency/fingerprint infrastructure rather than adding workspace scans.

## 28.4 Add/edit/delete

Required lifecycle:

```text
add conformance witness
    → publish callable/signature/body/source
    → recompute source plan
    → exact evidence becomes proven if complete

delete witness
    → remove callable products
    → source plan may fall back to inherent/default or become incomplete

delete conformance
    → remove source plan
    → remove/retire exact evidence cache entries

remove module
    → owner-complete removal of every conformance-owned callable/plan/evidence dependency
```

---

# 29. Source indexing and presentation

## 29.1 Conformance witness source targets

After P2:

```phalcom
impl Printable for User {
  toString -> String {
  ^^^^^^^^
  conformance-owned CallableId
  }
}
```

must have normal callable source information.

Use:

```text
SourceOwner::Callable(conformance-owned CallableId)
SemanticTargetId::Callable(...)
```

or repository-equivalent existing source identity.

## 29.2 Parameter/local scopes

Conformance witness bodies must receive ordinary source scopes for:

```text
parameters
member generics
locals
type references
body expressions
```

Do not leave them unindexed merely because they are not inherent members.

## 29.3 Requirement relationship

If the source-index architecture can represent semantic relationship edges cleanly, expose:

```text
explicit witness CallableId
    ↔ TraitRequirementId/source callable
```

through compiler-owned semantic products.

Do not make the LSP infer this relation from selector spelling.

Rich UI/navigation is not a completion blocker if the current public source index lacks a suitable relation type; canonical identities and evidence must exist so later tooling can project it.

---

# 30. Diagnostics

Add precise diagnostics rather than degrading every failure into “method not found.”

Required categories include:

```text
conformance member matches no trait requirement
duplicate explicit witness
bodyless explicit witness
missing trait witness
witness selector/role mismatch
witness parameter incompatibility
witness return incompatibility
witness generic-contract incompatibility
witness visibility/access mismatch
conditional inherent witness not proven
trait generic argument/constraint invalid
data component cannot satisfy mutable/write requirement
witness resolution unknown
witness resolution blocked
witness resolution dynamic
conformance source incomplete
coherence conflict encountered during evidence query
unsupported C5 associated binding syntax if encountered
unsupported C6 conditional trait constraint if encountered
```

Prefer stable diagnostic codes specific enough for tests and LSP.

Primary spans:

```text
invalid explicit witness
    → witness declaration

missing requirement
    → conformance declaration / trait reference
      with requirement source context where supported

invalid TraitRef
    → trait reference syntax

visibility mismatch
    → witness declaration plus requirement context

coherence conflict
    → P1 conformance source spans
```

Avoid duplicate diagnostics from both source-plan construction and every exact evidence query. Source-invalidity diagnostics belong primarily to the source conformance product.

---

# 31. T0 — Entry lock and live-baseline verification

## Objective

Freeze the actual P2 implementation baseline and verify that the P1 handoff contracts still hold.

## Actions

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -1 --oneline
```

Record:

```text
expected baseline:
    a7861a5b148715179ef81e2dfe82986a7d0fc499
    or explicit descendant

unrelated working-tree changes:
    preserve exactly
```

Read P1 walkthrough/handoff.

Verify focused predecessor tests still pass before production edits:

```sh
cargo test -p phalcom-semantic --test semantic impls
cargo test -p phalcom-semantic --test semantic traits
cargo check -p phalcom-semantic
```

Use exact test filters available in the live repository if suite organization differs.

## Required assertions

Confirm:

```text
ConformanceHeadMatch is not evidence
conformance members do not enter inherent surfaces
TraitSurface remains canonical contract source
C2 proof-state closure is present
no hidden P2 implementation has landed since baseline
```

## STOP/CONSULT

Stop if:

- P1 is no longer the repository baseline contract;
- a newer commit already introduces witness/evidence architecture with materially different identity rules;
- C3/P1 tests fail before P2 edits for unrelated reasons;
- the worktree contains conflicting modifications in P2 target files that cannot be safely preserved.

## Gate G0

T0 complete when predecessor state is recorded and focused predecessor tests pass.

---

# 32. T1 — Write failing architecture tests for audited P2 gaps

## Objective

Before broad production edits, create focused regressions proving P2-A01/A02/A03.

## Tests

### T1.1 constrained TraitRef rejection

```phalcom
trait NumericTag<T> where T <: Number {
  tag -> String
}

class Item {}

impl NumericTag<String> for Item {}
```

Assert:

```text
conformance diagnostic exists
no lookup-eligible ConformanceContribution
no future evidence
```

### T1.2 initial/incremental conformance body identity parity

Create:

```phalcom
trait Tagged { tag -> String }
class User {}

impl Tagged for User {
  tag -> String { "a" }
}
```

Inspect semantic-shard fingerprints/products before implementation.

Write expected P2 regression so cold and body-only incremental publication will eventually refer to the same conformance-owned callable identity.

### T1.3 no target-owned witness pollution

Assert that after P2 identity publication:

```text
User DeclarationSurface
    does not contain conformance tag

User target-owned CallableId table
    does not gain conformance tag
```

## Gate

Tests should fail for the intended missing functionality before production implementation.

---

# 33. T2 — Introduce conformance-local callable ownership

## Objective

Establish canonical source identity for conformance witness bodies.

## Production work

1. extend `CallableOwnerId` with conformance ownership;
2. update total `module()` behavior;
3. audit/remove false total `declaration_owner()` assumptions;
4. update hashing/order/debug/source target support;
5. add query/fingerprint support;
6. make compile-only match-arm updates outside semantic code where necessary, without adding P3 behavior.

## Required owner helpers

Prefer semantically named APIs such as:

```text
callable.module()
callable.declaration_owner() -> Option<&DeclarationId>
callable.variant_owner() -> Option<&VariantId>
callable.conformance_owner() -> Option<&ImplId>
```

or equivalent.

Do not map a conformance owner to its target declaration as a fake declaration owner.

## Audit list

Search all uses of:

```text
CallableOwnerId::
declaration_owner()
owner.declaration()
```

Classify before changing behavior.

Especially inspect:

```text
checker/body.rs
checker/context.rs
checker/expression.rs
checker/call.rs
db/query.rs
db/fingerprint.rs
session.rs
semantic_shard.rs
source_index/
presentation/editor
metadata export
phalcom-core semantic lowering
```

## Tests

Add identity-level tests:

```text
conformance callable module == ImplId.module
conformance callable has no lexical declaration owner
two members in same conformance share Conformance(ImplId) owner
same selector in two different conformance ImplIds yields different CallableId
```

## Gate G1

Identity migration complete when semantic crate compiles and focused existing class/variant callable tests remain green.

---

# 34. T3 — Repair semantic-shard/source fingerprint ownership

## Objective

Close P2-A02 and establish stable incremental inputs for witnesses.

## Production work

Refactor impl-member fingerprint extraction into explicit branches:

```text
ImplKind::Inherent
    → existing target-owned helper

ImplKind::Conformance
    → derive ImplId from statement identity
    → Conformance(ImplId) owner
    → collect signature/body fingerprints
```

Ensure `from_source` and body-only `with_source`/refresh use the same identity policy.

Do not let body contents enter conformance-head structural fingerprints.

## Tests

Required:

```text
cold initial shard and incremental refreshed shard
    contain same conformance witness CallableId

body edit
    changes body fingerprint
    preserves signature fingerprint when signature unchanged
    preserves conformance head identity

signature edit
    changes conformance witness signature fingerprint

delete conformance
    removes witness fingerprints
```

## Gate G2

Cold/incremental fingerprint parity for conformance witnesses.

---

# 35. T4 — Close checked TraitRef formation for conformance heads

## Objective

Close P2-A03 before evidence construction.

## Production work

Factor a reusable checked TraitRef formation API from C3/P1 logic.

Requirements:

```text
trait category
arity
kind
canonical argument form
trait generic constraints
proof-state result
```

Conformance head resolution must use the checked path.

Preserve P1 diagnostics/source spans where possible.

## Tests

### Concrete

```text
valid constrained TraitRef
invalid concrete argument
wrong kind
wrong arity
```

### Generic

Test a generic source head whose ambient impl constraints prove the trait's declaration constraint.

Test a generic head whose constraint cannot be proved in C4.

Expected:

```text
provable
    → eligible

unprovable/conditional-on-future-trait-proof
    → no successful source conformance proof
```

## STOP/CONSULT

Stop if satisfying this requirement would require implementing general `T: Trait` proof machinery. That belongs to C6. P2 should instead reject/defer that source domain.

## Gate G3

Every lookup-eligible P1 contribution uses a valid checked `TraitRef`.

---

# 36. T5 — Publish conformance-local witness signatures and source scopes

## Objective

Turn conformance members from inert syntax into canonical conformance-owned callable declarations.

## Production work

For each lookup-eligible conformance source:

1. locate source `ImplDef` via `ImplId`;
2. build impl-local type resolver/environment;
3. derive conformance-owned `CallableId` for each member;
4. map selector/side to one `TraitRequirementId`;
5. reject extra/duplicate/bodyless witness declarations;
6. resolve/publish canonical `CallableSemanticSignature`;
7. index source callable/parameters/type references;
8. retain witness declaration provenance.

Do not yet select these as complete witnesses until T7 compatibility exists.

## Body context plumbing

Add explicit context input for:

```text
Self target template
impl generic signature
source module
access enclosing declaration = none
trait conformance context where required
```

Do not derive these from `callable.declaration_owner()`.

## Tests

Cover:

```text
method
getter
setter
index getter
index setter
member-local generic
Self parameter/result
impl generic in witness signature
same requirement selector in two different conformance ImplIds
extra member rejected
duplicate explicit witness rejected
bodyless witness rejected
source navigation exists
target surface remains clean
```

## Gate G4

Every syntactically valid conformance witness has stable conformance-owned identity/signature/source products.

---

# 37. T6 — Analyze conformance-local witness bodies

## Objective

Check witness bodies once under the source conformance environment.

## Production work

Extend callable-body query plumbing to support conformance-owned callables.

Required body context:

```text
current callable
source module
Self target template
impl generic signature
callable-local generics
ordinary dispatch/effective inherent context
NO target-private lexical privilege
```

### Important access negative

If trait module `traits` defines a legal conformance for foreign target `models.User`, a witness body in `traits` must not gain access to private `User` state/members merely because `Self = User`.

## Tests

Include:

```text
simple witness body complete
witness return mismatch
impl generic visible
member generic visible
Self visible as target type
private target member inaccessible from foreign trait-owner conformance
target-owner conformance follows ordinary source access rules
body edit incremental recomputation
```

## Gate G5

Conformance-local witness bodies are first-class analyzed callables without inherent ownership leakage.

---

# 38. T7 — Implement instantiated requirement specialization

## Objective

Create the reusable P2 contract view required for compatibility.

## Production work

Implement/factor:

```text
instantiate_trait_requirement(
    TraitSurfaceMember,
    target template/exact target,
    TraitRef,
    impl environment
)
```

Preserve:

```text
TraitRequirementId
trait-owned source/default CallableId
requirement-local generic signature
visibility
source provenance
```

Specialize:

```text
trait declaration parameters
abstract Self
impl parameters appearing in target/TraitRef
```

Do not mutate `TraitSurface`.

## Tests

Matrix:

```text
non-generic trait
generic trait argument
Self return
Self parameter
trait arg + Self in same signature
member-local generic
getter
setter
index get/set
exact enum case Self
source generic target containing Impl-owned parameter
exact target specialization
```

## Gate G6

Raw trait surfaces can be projected into source/exact requirement contracts without identity loss.

---

# 39. T8 — Implement canonical witness compatibility

## Objective

Create one proof-state-aware compatibility authority.

## Production work

Implement/factor:

```text
check_witness_compatibility(
    instantiated requirement,
    candidate capability/signature,
    visibility,
    budget/cancellation
)
```

Use existing type relation machinery.

### Required checks

```text
selector
dispatch side
member role
parameter count/labels/rest shape
parameter substitutability
return substitutability
callable generic binder/kind compatibility
callable generic constraint compatibility
visibility/access coverage
terminal proof state
```

### Required proof-state preservation

Never collapse:

```text
Unknown
Blocked
Dynamic
Cancelled
BudgetExceeded
InternalFailure
```

into `Incompatible`.

## Tests

Positive:

```text
exact match
contravariantly broader witness parameter
covariantly narrower witness return
alpha-renamed member generics
public witness for public requirement
```

Negative:

```text
getter vs setter
index get vs set
wrong selector
too-narrow witness parameter
too-wide witness return
witness strengthens generic constraint
private/protected/internal coverage failure
dynamic/blocked relation not accepted
```

## Gate G7

Witness compatibility is one reusable semantic proof relation.

---

# 40. T9 — Implement effective target witness candidate lookup

## Objective

Feed T8 with canonical target behavior without syntax scans.

## Production work

Add/factor inherent-only P2 lookup covering:

```text
direct members
inherent impl members
inherited members
exact-case members
C2 specialized/conditional members
```

Return canonical signature, visibility, specialization, applicability evidence, and provenance.

Add data-component capability lookup for property requirements.

## Required generic-domain behavior

For source generic conformances, candidate applicability must be proven across the source domain.

Do not opportunistically use exact-specialization-only behavior to justify an unconditional generic conformance.

## Tests

Cover:

```text
direct class witness
covering inherent impl witness
inherited witness
child override
exact specialized inherent witness
proven C2 conditional witness
disproven conditional witness
unknown/blocked conditional witness
data component getter witness
immutable data component cannot satisfy setter
exact enum-case witness
```

## Gate G8

All allowed non-explicit witness sources are discoverable through canonical semantic products.

---

# 41. T10 — Build source-level witness selection and completeness

## Objective

Construct one `ConformanceWitnessPlan` per source conformance.

## Algorithm

For each `TraitRequirementId` in the C3 surface:

```text
1. instantiate requirement under source conformance domain

2. explicit conformance member exists?
    yes:
        require concrete body
        run compatibility
        if Compatible:
            select ConformanceCallable
        else:
            record explicit witness failure
        STOP selection for this requirement

3. resolve effective concrete inherent witness
    if exactly one Compatible:
        select InherentCallable
        STOP

    if ambiguity/unknown/blocked/dynamic:
        preserve failure state
        STOP as appropriate

4. resolve compatible data/property capability
    if proven:
        select DataComponent
        STOP

5. trait default exists and is usable?
    yes:
        select TraitDefault
        STOP

6. record MissingRequirement
```

After all requirements:

```text
all proven selections
    → Complete plan

any semantic incompatibility/missing requirement
    → Incomplete/invalid plan

terminal unknown/blocked/dynamic/etc.
    → corresponding non-success source-plan state
```

## Tests

### Precedence

```text
explicit witness > inherent witness > default
inherent witness > default
data component > default for compatible getter
invalid explicit witness does not fall through
```

### Completeness

```text
empty trait → complete
all inherent → complete
all defaults → complete
mixed sources → complete
one missing → incomplete
one incompatible → incomplete
```

### One callable multiple requirements

Across separate traits/conformances, the same inherent callable may be selected independently for distinct `TraitRequirementId`s.

Do not merge requirement identities.

## Gate G9

Every eligible source conformance has an explicit source-level satisfaction result.

---

# 42. T11 — Instantiate exact `ConformanceEvidence`

## Objective

Produce the P2 exit product.

## Production work

Implement exact resolver:

```text
exact target + exact TraitRef
    ↓
ConformanceIndex::query_exact
    ↓
unique ConformanceHeadMatch
    ↓
complete ConformanceWitnessPlan
    ↓
exact impl environment
    ↓
specialize selections
    ↓
ConformanceEvidence
```

Do not re-run witness selection.

## Tests

### Zero/one/multiple candidate

```text
no head
unique complete head
unique incomplete head
multiple P1 matches
```

### Generic source conformance

```phalcom
impl<T> Tagged for Value<T> {
  tag -> String { "generic" }
}
```

Assert:

```text
Value<Int> evidence:
    same source ImplId
    same source witness CallableId
    T := Int

Value<String> evidence:
    same source ImplId
    same source witness CallableId
    T := String
```

### Specialized source conformances

```phalcom
impl Tagged for Value<Int> { ... }
impl Tagged for Value<String> { ... }
```

Assert distinct source/witness identities.

### Exact generic TraitRef

```text
User + Converter<Int>
User + Converter<String>
```

must produce distinct exact evidence when separate coherent source conformances exist.

## Gate G10

`ConformanceEvidence` is produced only for complete proven exact relationships.

---

# 43. T12 — Incremental lifecycle, dependency, and parity closure

## Objective

Make evidence a correct incremental semantic product rather than a cold-analysis feature.

## Mutation matrix

Test cold + incremental parity for:

```text
add conformance
delete conformance

add explicit witness
delete explicit witness
edit witness signature
edit witness body

add target inherent witness
delete target inherent witness
edit target witness signature

change superclass
add/remove child override

add/remove applicable C2 conditional member
change constraint applicability

edit data component type

add trait requirement
remove trait requirement
edit trait requirement signature

add trait default
remove trait default
edit selected default signature/body where relevant

change trait generic argument constraint

remove provider module
replace source module
```

## Required assertions

For each relevant case:

```text
cold result == incremental result semantically
no stale witness callable remains
no stale source plan remains
no stale exact evidence remains
unrelated modules are not spuriously reprocessed where measurable
```

## Evidence cache policy

If exact evidence is memoized, key it by canonical semantic identities and snapshot-safe inputs.

Do not key by source spelling.

## Gate G11

Cold/incremental semantic parity for source plans and exact evidence.

---

# 44. T13 — P2 adversarial semantic acceptance matrix

## Objective

Prove the semantic architecture with dense but still P2-scoped fixtures.

## A. `Iterable` mixed-source evidence fixture

Use the current no-associated-type form:

```phalcom
trait Iterable<Item, Cursor> {
  iterate(_ cursor: Option<Cursor>) -> Option<Cursor>
  iteratorValue(_ cursor: Cursor) -> Item

  each(_ f) { ... }
  count -> Int { ... }
  contains(_ expected: Item) -> Bool { ... }
  toList -> List<Item> { ... }
}
```

Construct a target where:

```text
iterate
    → existing inherent callable

iteratorValue
    → conformance-local witness callable

each
    → trait default

count
    → trait default

contains
    → trait default

toList
    → trait default
```

Assert the exact requirement map in `ConformanceEvidence`.

Do not call these methods through target syntax yet.

## B. Specialized evidence fixture

```phalcom
trait Tagged {
  tag -> String
}

class Value<T> {}

impl Tagged for Value<String> {
  tag -> String { "string" }
}

impl Tagged for Value<Int> {
  tag -> String { "int" }
}
```

Assert source/witness identity separation.

## C. Generic source evidence fixture

```phalcom
impl<T> Tagged for Value<T> {
  tag -> String { "generic" }
}
```

Assert same source/witness identities under different exact environments.

## D. Inherited/data/default fixture

One trait with:

```text
required method witnessed by superclass
required getter witnessed by data component
defaulted method
```

Prove mixed selection kinds.

## E. Failure fixture

One conformance containing:

```text
extra member
bad explicit witness
missing requirement
visibility mismatch
```

Assert precise diagnostics and no evidence.

## Gate G12

All adversarial semantic evidence fixtures pass.

---

# 45. T14 — Focused verification and P2 completion artifacts

## Required focused commands

Adapt exact test names to the live repository, but execute at least:

```sh
cargo test -p phalcom-semantic --test semantic traits
cargo test -p phalcom-semantic --test semantic impls
cargo check -p phalcom-semantic
cargo fmt --all -- --check
git diff --check
```

Run targeted tests for any separately touched semantic query/source-index modules.

Do **not** require the full compiler/VM runtime suite for P2 completion unless production compiler/runtime behavior was materially changed beyond mechanical enum-match compatibility.

P3 owns executable trait-evidenced dispatch certification.

## Completion documents

Update/create:

```text
LANG005.C4-CHECKPOINT.md
LANG005.C4.P2-walkthrough.md
LANG005.C4.P2-handoff.md
```

The P2 walkthrough must record:

```text
actual callable-owner shape
actual source witness-plan product
actual exact evidence product
actual compatibility authority
actual incremental dependencies
focused test counts
known P3 prerequisites
```

The P3 handoff must explicitly state:

```text
P3 may consume ConformanceEvidence
P3 may not re-run witness/default selection
P3 must preserve inherent-vs-trait-evidenced surface separation
```

## Gate G13

P2 status:

```text
COMPLETE
IMPLEMENTED
FOCUSED_TESTED
```

The source-plan/evidence, proof-state compatibility, terminal applicability,
C2 evidence retention, and incremental parity slices are implemented and
focused-tested. Checkpoint `next_plan` is `LANG005.C4.P3`.

---

# 46. Test inventory

The final P2 suite should cover at least the following IDs.

## Identity and ownership

```text
P2-ID-01 conformance-owned callable identity
P2-ID-02 same source generic witness identity across exact applications
P2-ID-03 different ImplIds produce different witness CallableIds
P2-ID-04 target DeclarationSurface remains uncontaminated
P2-ID-05 trait default remains trait-owned
```

## TraitRef validation

```text
P2-TR-01 concrete valid constrained TraitRef
P2-TR-02 concrete violated constraint rejected
P2-TR-03 wrong arity rejected
P2-TR-04 wrong kind rejected
P2-TR-05 symbolic provable constraint accepted
P2-TR-06 C6-only/unprovable trait-conditioned constraint not assumed
```

## Explicit witnesses

```text
P2-EW-01 method witness
P2-EW-02 getter witness
P2-EW-03 setter witness
P2-EW-04 index-get witness
P2-EW-05 index-set witness
P2-EW-06 member generic witness
P2-EW-07 Self in signature
P2-EW-08 extra member rejected
P2-EW-09 duplicate witness rejected
P2-EW-10 bodyless witness rejected
P2-EW-11 bad explicit witness does not fall through
```

## Inherent/inherited/conditional witnesses

```text
P2-IW-01 direct member
P2-IW-02 inherent impl member
P2-IW-03 inherited member
P2-IW-04 child override
P2-IW-05 specialized inherent member
P2-IW-06 conditional member proven
P2-IW-07 conditional member disproven
P2-IW-08 conditional member unknown
P2-IW-09 conditional member blocked
P2-IW-10 conditional member dynamic
```

## Data components

```text
P2-DC-01 immutable readable component satisfies getter
P2-DC-02 generic component specializes correctly
P2-DC-03 immutable component cannot satisfy setter
```

## Compatibility

```text
P2-CM-01 exact callable match
P2-CM-02 contravariant parameter compatibility
P2-CM-03 covariant return compatibility
P2-CM-04 parameter mismatch
P2-CM-05 return mismatch
P2-CM-06 role mismatch
P2-CM-07 generic alpha-renaming
P2-CM-08 stronger witness generic constraint rejected
P2-CM-09 visibility coverage success
P2-CM-10 visibility coverage failure
P2-CM-11 dynamic/blocked not accepted
```

## Defaults/completeness

```text
P2-DF-01 default selected
P2-DF-02 inherent beats default
P2-DF-03 explicit beats default
P2-DF-04 missing requirement incomplete
P2-DF-05 empty trait complete
P2-DF-06 mixed-source complete
```

## Exact evidence

```text
P2-CE-01 no source head → NotDeclared
P2-CE-02 unique complete head → Proven
P2-CE-03 unique incomplete head → Incomplete
P2-CE-04 multiple matches → CoherenceConflict
P2-CE-05 distinct exact TraitRefs remain distinct
P2-CE-06 exact enum case retains VariantId environment
P2-CE-07 generic source evidence instantiates without reselection
P2-CE-08 specialized source conformances retain distinct provenance
```

## Incremental

```text
P2-IN-01 cold/incremental witness fingerprint parity
P2-IN-02 body edit
P2-IN-03 signature edit
P2-IN-04 add/delete witness
P2-IN-05 add/delete conformance
P2-IN-06 trait requirement edit
P2-IN-07 trait default edit
P2-IN-08 hierarchy edit
P2-IN-09 conditional applicability edit
P2-IN-10 module removal
```

---

# 47. Required invariants

P2 is not complete until all of these are demonstrably true.

1. A unique P1 conformance head is not itself conformance evidence.
2. `ImplId` remains source/provenance identity.
3. One generic source conformance has one source `ImplId`.
4. One conformance-local witness declaration has one source `CallableId`.
5. A generic source witness `CallableId` is reused across exact target specializations.
6. Conformance-local witness ownership is not target inherent ownership.
7. Conformance-local witness ownership is not trait default ownership.
8. `TraitRequirementId` remains independent of witness identity.
9. `TraitSurface` remains the sole trait requirement/default contract authority.
10. Trait defaults remain trait-owned callables.
11. Trait defaults are not re-typechecked per conformance.
12. Raw trait requirement signatures are specialized before witness comparison.
13. `Self` specializes to the conformance target.
14. Trait generic arguments specialize through exact `TraitRef`.
15. Impl generic parameters retain `TypeParameterOwner::Impl`.
16. Callable-local witness generics retain `TypeParameterOwner::Callable`.
17. Selector/member-role compatibility is exact.
18. Getter cannot satisfy setter.
19. Index getter cannot satisfy index setter.
20. Witness parameter compatibility follows canonical callable substitutability.
21. Witness return compatibility follows canonical callable substitutability.
22. Visibility is access-set coverage, not enum ordinal comparison.
23. Only proven compatibility satisfies a requirement.
24. Unknown proof is not disproval.
25. Dynamic proof is not static success.
26. Blocked proof is not ordinary missing-member failure.
27. C2 conditional applicability evidence is consumed, not independently re-solved.
28. A source generic conformance is proven over its whole declared C4 domain.
29. P2 does not opportunistically validate different exact subsets of one unconditional generic conformance.
30. A data component remains identified by `DataComponentId`.
31. P2 does not fabricate getter callables for data components.
32. Explicit conformance witness has precedence over existing/default behavior.
33. An invalid explicit witness does not silently fall through.
34. Compatible concrete target behavior precedes trait default.
35. Trait default is selected only when no concrete satisfaction source exists.
36. Every behavioral requirement must have one proven selection for source completeness.
37. C5 associated-type completeness is not implemented in P2.
38. `ConformanceWitnessPlan` is source-level and body/generic-domain aware.
39. `ConformanceEvidence` is exact-target/exact-`TraitRef` evidence.
40. Exact evidence instantiates a source plan rather than re-running selection.
41. `ConformanceEvidence` exists only for a complete proven relationship.
42. Two or more P1 exact matches never produce arbitrary selection.
43. Exact enum-case evidence retains exact `VariantId`/case environment.
44. Inherited class behavior may witness a child conformance.
45. Conformance itself does not automatically inherit from a superclass in C4.
46. Conformance witnesses do not enter `DeclarationSurface`.
47. Selected defaults do not enter `DeclarationSurface`.
48. P2 witness search is inherent-only and cannot recursively use P3 trait-evidenced behavior.
49. Conformance witness bodies do not gain foreign target-private access merely from `Self`.
50. Cold and incremental witness identity are identical.
51. Body-only edits preserve head and callable identity.
52. Signature/body dependencies invalidate evidence only when semantically relevant.
53. Removing a conformance removes its witness/source-plan products.
54. Removing a provider module leaves no stale evidence.
55. LSP/source tooling consumes canonical conformance witness identity rather than selector reconstruction.
56. P2 introduces no runtime trait scan or runtime conformance registry.
57. P2 introduces no associated-type bindings.
58. P2 introduces no general trait-bound solver.
59. P2 introduces no trait objects.
60. P2 hands P3 structured evidence sufficient to avoid witness/default reselection.

---

# 48. Files likely to change

Exact paths may drift, but expect focused changes around:

```text
phalcom-semantic/src/identity.rs

phalcom-semantic/src/impls.rs
    ConformanceContribution consumers
    witness-plan/evidence products or a new dedicated module

phalcom-semantic/src/traits.rs
    reusable requirement/TraitRef specialization helpers where appropriate

phalcom-semantic/src/signature.rs
    conformance-owned signature support

phalcom-semantic/src/semantic_shard.rs
    conformance witness fingerprints

phalcom-semantic/src/session.rs
    source witness-plan publication
    incremental invalidation

phalcom-semantic/src/snapshot.rs
    witness-plan/evidence roots or canonical query access

phalcom-semantic/src/dispatch.rs
    inherent-only candidate projection if needed

phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/call.rs
    conformance witness body context and reusable compatibility machinery

phalcom-semantic/src/types/relation.rs
    only if a reusable bounded compatibility helper is missing

phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/environment.rs
    only low-level specialization helpers if required

phalcom-semantic/src/source_index/
    conformance witness callables/scopes

phalcom-semantic/src/db/
    query keys/fingerprints/products

phalcom-semantic/src/diagnostic.rs

phalcom-semantic/tests/semantic/
    traits/
    impls/
    or repository-equivalent focused suites
```

Potential mechanical compile-only changes:

```text
phalcom-core semantic-lowering match arms
metadata/stable identity match arms
presentation/editor match arms
```

Do not implement P3 execution merely because a new `CallableOwnerId` variant reaches compiler code.

---

# 49. Prohibited shortcuts

Do not:

```text
rename ConformanceHeadMatch to ConformanceEvidence

use bool conforms

store conformance witness as User.member

copy trait default into User surface

synthesize DataComponent getter CallableId

re-typecheck default per conformer

body-check generic witness per exact specialization

resolve generic source conformance opportunistically per exact target

use .first() on multiple conformance matches

treat Unknown as NotApplicable

treat Dynamic as Proven

derive visibility compatibility from enum ordering

grant target private access to foreign conformance witness bodies

scan source AST for target witnesses when semantic surfaces exist

re-run C2 applicability with a new solver

use trait-evidenced P3 behavior as P2 witness source

implement associated types in the witness map

implement T: Trait constraints to make a hard test pass

add runtime trait scanning

mutate runtime class tables as semantic conformance authority

introduce conformance specialization/"most specific wins"
```

---

# 50. STOP / CONSULT triggers

Stop implementation and request architectural review if any of these occurs.

## SC-01 — conformance callable ownership cannot be added without redefining public callable identity

If the live repository has acquired an identity architecture after P1 that materially supersedes `CallableOwnerId`, do not add a parallel conformance callable system.

## SC-02 — witness body needs target-private privilege to satisfy existing normative tests/spec

The current design deliberately separates `Self` from class lexical access. If a ratified visibility specification says conformances receive target-private privilege, this plan must be amended.

## SC-03 — trait requirement compatibility needs a new variance/overload model

P2 must reuse Phalcom's ordinary callable substitutability and selector semantics. Do not invent trait-specific overload rules.

## SC-04 — generic source completeness appears to require per-exact-target witness selection

That would contradict the unconditional generic-conformance model. Re-evaluate source-domain proof or identify a C6 conditional-conformance requirement.

## SC-05 — checked TraitRef constraints require `T: Trait`

Do not implement C6 inside P2.

## SC-06 — associated types are required for a chosen fixture

Use a no-associated-type C4 fixture. C5 owns associated types.

## SC-07 — P2 witness lookup requires ordinary trait-evidenced member lookup

That is likely an accidental cycle into P3. Keep witness lookup inherent-only.

## SC-08 — exact evidence cannot be specialized from source plan without re-running selection

Before accepting that architecture, prove which semantic fact changes across exact applications. The expected C4 model is source-universal satisfaction plus exact substitution.

## SC-09 — conformance evidence requires runtime registration

Semantic evidence must remain compile-time authority. Runtime representation choices are P3/later.

## SC-10 — fixing P1 coherence requires introducing conformance specialization

Forbidden under current C4 rules.

---

# 51. Verification budget policy

Use three levels.

## BUILD

For T1–T12:

```text
one/few targeted semantic tests
cargo check -p phalcom-semantic when identity/query signatures move
```

Do not run the full workspace after every edit.

## GATE

At major gates G3, G7, G10, G12:

```text
focused traits suite
focused impl/conformance suite
source-index tests if touched
cargo check -p phalcom-semantic
```

## COMPLETION

At G13:

```sh
cargo test -p phalcom-semantic --test semantic traits
cargo test -p phalcom-semantic --test semantic impls
cargo check -p phalcom-semantic
cargo fmt --all -- --check
git diff --check
```

If the test binary uses different filtering syntax, run repository-equivalent commands and record them in the walkthrough.

Full VM/runtime acceptance belongs to P3.

---

# 52. P2 vertical acceptance example

Use this as the compact conceptual acceptance target:

```phalcom
trait Comparable<Rhs> {
  compare(_ other: Rhs) -> Ordering

  <(_ other: Rhs) -> Bool {
    self.compare(other) === Ordering::Less
  }
}

class Meter {
  value -> Int
}

impl Comparable<Meter> for Meter {
  compare(_ other: Meter) -> Ordering {
    ...
  }
}
```

P2 must prove:

```text
TraitRequirementId(compare)
    → ConformanceCallable(
        CallableOwnerId::Conformance(impl_id),
        selector compare(_)
      )

TraitRequirementId(<)
    → TraitDefault(
        CallableOwnerId::Declaration(Comparable),
        selector <(_)
      )

ConformanceEvidence {
    source_impl,
    exact_target = Meter,
    exact_trait_ref = Comparable<Meter>,
    requirements = {
        compare => conformance witness,
        <       => trait default,
    }
}
```

P2 does **not** yet need:

```text
a < b
    executes correctly
```

That is P3's vertical execution acceptance.

---

# 53. P2 dense acceptance example — mixed `Iterable` evidence

A stronger semantic fixture should model:

```phalcom
trait Iterable<Item, Cursor> {
  iterate(_ cursor: Option<Cursor>) -> Option<Cursor>
  iteratorValue(_ cursor: Cursor) -> Item

  each(_ f) { ... }
  count -> Int { ... }
  contains(_ expected: Item) -> Bool { ... }
  toList -> List<Item> { ... }
}
```

Target/conformance arrangement:

```text
RangeView.iterate
    existing inherent callable

impl Iterable<Int, Int> for RangeView {
    iteratorValue(_ cursor: Int) -> Int { ... }
}

each/count/contains/toList
    trait defaults
```

P2 completion requires exact evidence:

```text
iterate
    → InherentCallable

iteratorValue
    → ConformanceCallable

each
    → TraitDefault

count
    → TraitDefault

contains
    → TraitDefault

toList
    → TraitDefault
```

This fixture should remain free of C5 associated types.

---

# 54. P2 → P3 handoff contract

P2 must leave P3 these stable capabilities.

## Required P3 inputs

```text
resolve exact ConformanceEvidence
enumerate exact requirement selections
distinguish selection source kind
obtain selected concrete CallableId/DataComponentId/default CallableId
obtain exact specialization environment
obtain source ImplId/provenance
obtain trait requirement identity
```

## P3 must not need to

```text
parse conformance bodies
match witness selectors
re-check witness signature compatibility
re-run C2 applicability
decide whether a default wins
re-check source completeness
```

## P3 responsibility

P3 will add:

```text
trait-evidenced ordinary member availability
shared-witness convergence
competing-default ambiguity
call/reference selection
selected-default invocation semantics
default → abstract requirement → selected witness binding
compiler/runtime evidence transport
executable vertical acceptance
```

P2's evidence architecture is successful if P3 can remain a consumer.

---

# 55. Completion checklist

Before marking P2 complete:

```text
[ ] T0 live baseline recorded
[ ] P1/C3 focused predecessor tests green
[ ] P2-A01 conformance callable owner closed
[ ] false total declaration-owner assumptions audited
[ ] P2-A02 shard fingerprint asymmetry closed
[ ] P2-A03 checked TraitRef constraints closed
[ ] conformance witness source indexing published
[ ] witness signatures canonical
[ ] witness bodies analyzed once
[ ] target-private access leakage negative test passes
[ ] instantiated requirement view implemented
[ ] compatibility is proof-state aware
[ ] callable variance/substitutability covered
[ ] visibility access coverage covered
[ ] inherent/inherited/C2 candidate lookup unified
[ ] data component witness implemented
[ ] explicit witness precedence implemented
[ ] invalid explicit witness no-fallback behavior implemented
[ ] trait default selection implemented
[ ] source-level completeness implemented
[ ] source ConformanceWitnessPlan published
[ ] exact ConformanceEvidence published
[ ] generic source conformance specializes without reselection
[ ] distinct specialized conformances preserve provenance
[ ] exact TraitRef applications remain distinct
[ ] exact enum-case evidence preserved
[ ] cold/incremental parity matrix passes
[ ] target inherent surfaces remain uncontaminated
[ ] no runtime trait scan introduced
[ ] no C5 associated-type semantics introduced
[ ] no C6 trait-bound/conditional-conformance semantics introduced
[ ] focused completion commands pass
[ ] C4 checkpoint updated
[ ] P2 walkthrough written
[ ] P3 handoff written
```

---

# 56. Final acceptance statement

`LANG005.C4.P2` is complete only when the repository can establish this implication:

```text
unique authorized/coherent source conformance head
    +
complete C3 TraitSurface
    +
proven compatible explicit/inherent/data/default satisfaction
        ↓
complete source ConformanceWitnessPlan
        ↓
exact head specialization
        ↓
ConformanceEvidence
```

and reject the inverse shortcuts:

```text
unique head
    ≠ automatically conforms

matching selector
    ≠ automatically compatible witness

structural member coincidence
    ≠ implicit conformance

trait default exists
    ≠ target owns that member

conformance witness body exists
    ≠ target inherent member

unknown/blocked/dynamic relation
    ≠ proven evidence
```

The P2 exit product is not a runtime vtable and not a method-table mutation.

It is a **structured semantic proof**:

```text
this exact target
    satisfies
this exact TraitRef
    because
this source ImplId
    selects
these exact witnesses/defaults
    for
these exact TraitRequirementId values
```

That proof is the sole semantic authority P3 should need to make trait behavior available and executable.
