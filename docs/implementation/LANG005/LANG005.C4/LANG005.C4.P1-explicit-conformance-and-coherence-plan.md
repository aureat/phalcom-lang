---
id: LANG005.C4.P1
category: LANG
program: LANG005
checkpoint: LANG005.C4
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C3
  - LANG005.C2 applicability-proof-state closure
follows: LANG005.C3.P1
prepared: 2026-09-14
repository: aureat/phalcom-lang
repository_baseline: 20ad3f31b39b0fe1b1fdf028df4ac9579fcfd9ad
planning_mode: prospective-post-C3
next_plan: LANG005.C4.P2
---

# LANG005.C4.P1 — Explicit Conformance Declarations, Ownership, and Coherence

## Luna Patch-Grade Implementation Plan

This plan begins `LANG005.C4` after the complete delivery of C1, C2, and C3. Its responsibility is intentionally narrower than “implement traits”: C3 owns trait declarations and abstract trait surfaces; P1 owns the source and semantic machinery that says **which explicit conformance declarations exist, whether they are authorized, what exact target/trait domains they describe, and whether those domains are globally coherent**.

P1 must not yet decide how individual trait requirements are satisfied. That work belongs to C4.P2.

The architectural cut is therefore:

```text
C3
    trait declaration
    TraitRef
    TraitRequirementId
    TraitSurface
    abstract Self
    trait-owned defaults
        ↓
C4.P1
    impl TraitRef for Target
    source ImplId provenance
    conformance head resolution
    trait-or-target ownership
    cross-module conformance publication
    exact target + exact TraitRef lookup
    generic-head specialization
    overlap/coherence
        ↓
C4.P2
    witness discovery
    conformance-local witness callables
    data/inherited/conditional witnesses
    default selection
    completeness
    structured ConformanceEvidence
        ↓
C4.P3
    trait-evidenced ordinary dispatch
    ambiguity resolution
    lowering/runtime execution
```

P1 is complete when the semantic system can answer:

> “For this exact target and this exact trait reference, which well-formed, authorized, coherence-safe source conformance declaration is the candidate?”

P1 is **not** complete merely because the parser accepts `impl Trait for Target`, and it must never answer the stronger question:

> “Does this target satisfy the trait?”

That stronger statement requires P2 evidence.

---

## 0. Executor contract

This plan is written for a Luna-class implementer operating under constrained architectural authority.

The implementer:

- may adapt filenames, helper placement, naming, and mechanical factoring to the post-C3 repository;
- must preserve the identities, ownership rules, semantic products, coherence law, and checkpoint boundaries defined here;
- must re-ground against the actual C3 completion commit before editing production code;
- must not treat the planning baseline as post-C3 implementation truth;
- must not merge conformance declarations into inherent surfaces;
- must not introduce a boolean `conforms` cache in place of structured candidate/evidence architecture;
- must not implement witnesses, defaults, associated types, trait constraints, conditional conformance, trait objects, or runtime trait scanning in P1;
- must keep tests narrowly aligned to the task currently being implemented and broaden only at named gates;
- must preserve cold/incremental parity for every new workspace product;
- must update the C4 checkpoint state at durable gates;
- must produce a P1 walkthrough and a P2 handoff before completion.

Testing is evidence gathering. During BUILD mode, use the smallest test that distinguishes the current hypothesis. Do not repeatedly run the whole workspace because a parser or semantic helper changed.

The executor may not self-waive any STOP/CONSULT trigger in this document.

---

## 1. Goal

Implement the first semantic half of explicit trait conformance:

1. extend the shared `impl` declaration syntax so it can represent both inherent implementations and `impl TraitRef for Target` conformances without creating a parallel declaration subsystem;
2. resolve conformance trait references and targets under implementation-local generic scope;
3. preserve `ImplId` as source/provenance identity;
4. represent conformance heads separately from inherent behavior products;
5. enforce the trait-or-target ownership rule using canonical declaration provenance rather than import/re-export provenance;
6. publish conformance contributions into a workspace-level index discoverable from either ownership side;
7. support exact and generic conformance heads, including distinct specialized targets such as `Value<String>` and `Value<Int>`;
8. resolve exact `(target, TraitRef)` candidate queries through canonical generic substitution;
9. reject duplicate and overlapping conformances globally, with no source-order or “most specific” precedence;
10. make add/edit/delete/re-export/module replacement incrementally correct;
11. expose source/tooling relationships sufficient for later P2/P3 projection;
12. leave requirement satisfaction completely unclaimed until P2.

---

## 2. Checkpoint slice acceptance objective

P1 accepts the following semantic flow:

```text
source:
    impl Printable for User { ... }
        ↓
shared ImplDef
    kind = conformance
    trait syntax = Printable
    target syntax = User
    impl-local generics
        ↓
resolve header
    ImplId
    exact/parameterized TraitRef template
    nominal target identity
    target TypeId template
    impl GenericSignature
        ↓
authorize
    source module owns trait OR target
        ↓
ConformanceContribution
    source/provenance only
    no witness proof
        ↓
workspace ConformanceIndex
    coarse candidate bucket
    deterministic owner-complete publication
        ↓
coherence
    pairwise joint target+TraitRef overlap analysis
    no implicit specialization
        ↓
exact query
    exact receiver target
    exact TraitRef
        ↓
ConformanceHeadMatch
    selected ImplId
    impl parameter substitution
    exact target
    exact TraitRef
```

The distinction at the end is mandatory:

```text
ConformanceHeadMatch
    means:
        one source conformance head applies to this exact pair

ConformanceEvidence
    means:
        every trait obligation is actually satisfied

P1 produces the first.
P2 produces the second.
```

A P1 query result therefore must not be named or exposed as `conforms`, `is_conforming`, `Satisfied`, `ProvenConformance`, or any equivalent that implies requirement completeness.

### P1 success examples

The index must distinguish:

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

as two independent conformance domains:

```text
Value<String> + Tagged
Value<Int>    + Tagged
```

The shared declaration identity `Value` must not collapse them.

Likewise, exact generic trait references remain distinct:

```phalcom
trait Converter<T> {
  convert -> T
}

impl Converter<Int> for User { ... }
impl Converter<String> for User { ... }
```

These are distinct candidate domains because `Converter<Int>` and `Converter<String>` are distinct `TraitRef`s.

### P1 rejection example

This pair overlaps and is invalid under C4 coherence:

```phalcom
impl<T> Tagged for Value<T> { ... }
impl Tagged for Value<Int> { ... }
```

For the exact pair `Value<Int> + Tagged`, both heads apply. C4 has no trait-conformance specialization rule, so the exact-looking declaration does not win.

---

## 3. Repository grounding

### 3.1 Planning baseline

Prepared against remote `main`:

```text
repository: aureat/phalcom-lang
revision: 20ad3f31b39b0fe1b1fdf028df4ac9579fcfd9ad
commit: docs: pin lang005 checkpoint revision
prepared: 2026-09-14
```

The baseline includes later LANG005 predecessor repair work after the original C2 completion revision, including generic call-entry reification. It is still pre-C3 for purposes of this plan.

P1 is therefore **prospective-post-C3**. T0 must replace all planning assumptions about C3 implementation details with the landed interfaces from the actual C3 completion revision.

### 3.2 Verified baseline facts relevant to P1

At the planning baseline:

1. `phalcom-semantic/src/identity.rs` defines `ImplId { module, local }`, where the source module is directly available from the implementation provenance identity.
2. `CallableOwnerId` is currently `Declaration(DeclarationId) | Variant(VariantId)`; P1 must not add conformance-owned callables. P2 owns that decision and migration.
3. `phalcom-semantic/src/impls.rs` owns inherent implementation target resolution, generic target domains, conditional applicability, contribution publication, and effective-surface composition.
4. `InherentImplTarget` currently distinguishes `Declaration(DeclarationId)` and `ExactEnumCase(VariantId)`.
5. `ResolvedInherentImplTarget` and `InherentImplContribution` are explicitly inherent products. P1 must not reuse those names/types as conformance products merely because source syntax shares `impl`.
6. `TypeParameterOwner::Impl(ImplId)` already gives implementation-local generic parameters canonical identity.
7. `TypeEnvironment`, `TypeSubstitution`, canonical `TypeId`, generic signatures, and target-unification machinery already exist and should be reused for conformance-head specialization.
8. At this baseline, `ImplApplicabilityResult` is still collapsed to `Applicable | NotApplicable | Blocked`. The earlier C1+C2 audit identified this as insufficient before witness/conformance work. P1 assumes the required pre-C4 proof-state closure has landed by T0.
9. `ModuleSemanticStructureShard` already retains source-local fingerprints and separates callable signature/body invalidation. C4 conformance header/body invalidation must follow the same incremental discipline.
10. Existing impl fingerprint extraction assumes one inherent target field. The post-C3 implementation must be audited and extended so conformance trait reference and target identity participate in structural fingerprints, while witness body text does not perturb conformance-head identity.
11. The canonical trait specification states that conformance is explicit, nominal, exact-trait-reference based, and owned by either the trait module or target module.
12. The canonical impl specification states that inherent and conformance implementations share the `impl` declaration mechanism but produce different semantic products.
13. The trait specification permits declaration-owned nominal targets and exact enum cases; anonymous structural tuple/record/function types are not direct conformance targets.
14. Re-export does not transfer conformance ownership.
15. Coherence is exact `(target, TraitRef)` coherence: at most one applicable conformance may exist, and the language does not select by source order or most-specific appearance.
16. Generic conformance heads are part of C4; associated types are C5; trait-conditioned/conditional conformance is C6.

### 3.3 Required post-C3 verification

T0 must verify, not assume, the landed forms of:

```text
TraitRef
TraitHeader / TraitInfo
TraitSurface
TraitRequirementId
trait declaration lookup API
trait declaration-kind test
trait generic substitution API
source/index targets for traits
post-C3 ModuleSemanticStructureShard fields
post-C3 impl AST
post-C3 semantic DB/session product ownership
C2 applicability proof-state result
```

If C3 renamed or reorganized these concepts, adapt mechanically while preserving this plan's semantic roles.

---

## 4. Required reads before implementation

Read these in order before T1:

### Normative behavior

1. `docs/specs/data/traits.md`
2. `docs/specs/data/impl.md`
3. `docs/specs/data/enums.md` — exact-case target identity and case type environment
4. authoritative callable/member specs relevant to selector/member identity
5. authoritative visibility specification if witness-independent conformance accessibility checks are present post-C3

### LANG005 predecessor state

6. final `LANG005.C3-CHECKPOINT.md`
7. final `LANG005.C3-GUIDANCE.md`
8. final C3 walkthrough and handoff
9. final C3 implementation plan only as historical intent; landed code wins for mechanics
10. final C2 P3 walkthrough/handoff for target-domain and applicability architecture
11. the C1+C2/C3 audit record containing the pre-C4 applicability-proof-state requirement

### Production code

12. `phalcom-ast/src/ast.rs`
13. `phalcom-ast/src/parser.rs`
14. `phalcom-modules/src/declaration.rs`
15. module interface/linker ownership APIs
16. `phalcom-semantic/src/identity.rs`
17. post-C3 trait semantic owner, expected `phalcom-semantic/src/traits.rs` or repository equivalent
18. `phalcom-semantic/src/impls.rs`
19. `phalcom-semantic/src/semantic_shard.rs`
20. `phalcom-semantic/src/session.rs`
21. semantic DB/query layer
22. type annotation resolver and `TraitRef` formation API
23. type substitution/unification/constraint proof machinery
24. source index / semantic target projection
25. semantic test README and existing impl/trait test organization

### Support fixtures

26. `docs/implementation/LANG005/support/core_trait_impls.ph`
27. `docs/implementation/LANG005/support/iterable.ph`

The support fixtures are forward-design references, not permission to pull C5/C6 syntax into P1.

---

## 5. Normative authority and conflict resolution

Use this precedence when sources disagree:

```text
1. explicit user-ratified LANG005 decisions in the C4 checkpoint/guidance
2. current authoritative specs under docs/specs/
3. completed C3/C2 checkpoint + walkthrough contracts
4. landed production architecture
5. this implementation plan
6. old LANG005 drafts/support fixtures
7. legacy protocol documents
```

If this plan conflicts with a higher authority, STOP AND CONSULT before changing architecture.

Mechanical drift is not a conflict. Rename/adapt paths and helper shapes when the landed C3 repository already provides the same semantic responsibility under a different interface.

---

## 6. Entry conditions and takeover state

P1 does not include predecessor repair work. Before implementation begins, T0 must establish all of the following:

### EC-01 — C1 complete

Exact runtime/product identity and reification prerequisites required by generic applications are complete and accepted.

### EC-02 — C2 complete

Inherent behavior, exact-case behavior, generic/specialized target applicability, owner-complete publication/removal, and semantic→lowering selection boundaries are complete.

### EC-03 — C2 applicability proof-state closure complete

The collapsed pre-C4 result:

```text
Applicable | NotApplicable | Blocked
```

must no longer be the only canonical proof-state model used by C4-facing semantic consumers. The pre-C4 architecture must distinguish enough states to avoid silently treating unknown/dynamic/blocked proof as success or ordinary disproval.

Expected conceptual distinction:

```text
proved
 disproved
 unknown
 blocked
 dynamic
```

Names may differ.

P1 itself mostly uses decidable head unification rather than conditional constraint proof, but C4 may not begin on an applicability model known to be too weak for P2.

### EC-04 — C3 complete

C3 must expose canonical APIs for:

```text
recognize trait DeclarationId
form TraitRef under generic scope
query TraitSurface
substitute trait generics
identify trait generic owner
represent abstract Self
navigate trait source identity
```

### EC-05 — no hidden C3 conformance implementation

If C3 already landed any provisional conformance parser/index code beyond its stated scope, T0 must inventory it. Do not layer a second competing architecture on top. Either adopt it if semantically equivalent or STOP if it materially contradicts P1.

---

## 7. Architecture

### 7.1 Shared syntax, separate semantic products

The source grammar shares `impl`, but semantics branch immediately after syntactic normalization:

```text
impl Target { ... }
    ↓
Inherent impl
    ↓
InherentImplContribution
    ↓
effective inherent behavior

impl TraitRef for Target { ... }
    ↓
Conformance impl
    ↓
ConformanceContribution
    ↓
ConformanceIndex
```

The second arrow must never flow into `DeclarationSurface`, `ConditionalInherentMemberSet`, or another target-owned inherent behavior product.

### 7.2 AST normalization

Preferred conceptual shape:

```rust
pub struct ImplDef {
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub kind: ImplKind,
    pub where_clause: Option<WhereClauseSyntax>,
    pub members: Vec<BehaviorMember>,
    pub range: SourceRange,
}

pub enum ImplKind {
    Inherent {
        target: TypeAnnotation,
    },
    Conformance {
        trait_ref: TypeAnnotation,
        target: TypeAnnotation,
    },
}
```

Equivalent normalized shapes are acceptable. The fixed requirement is that conformance syntax preserves the trait reference and target separately; do not encode `Trait for Target` into one opaque target expression.

If the post-C3 AST already generalized `ImplDef`, retain it.

### 7.3 Source identity

`ImplId` remains the identity of the source declaration/provenance:

```text
ImplId
    module
    source-local impl ordinal
```

Do not add `ConformanceId` in P1 merely because conformances are semantically important.

A generic source conformance can apply to many exact target/trait pairs, so source declaration identity and exact application identity must remain separate.

### 7.4 Target identity

Conformance targets are nominal:

```text
Declaration(DeclarationId)
ExactEnumCase(VariantId)
```

The implementation should expose a semantically neutral target identity to C4. It must not make conformance APIs depend on a type literally named `InherentImplTarget`.

Mechanically acceptable choices:

1. extract/rename the shared two-variant identity to a neutral `NominalImplTarget` / `ImplTargetId`, with inherent wrappers where useful; or
2. introduce a thin `ConformanceTarget` containing the same canonical declaration/variant identities while leaving C2 internals untouched.

The architectural requirement is semantic neutrality, not a mandatory repository-wide rename.

The resolved conformance also retains the target's type template:

```rust
ConformanceTarget {
    nominal: NominalTargetId,
    head_type: TypeId,
}
```

`head_type` is what distinguishes, for example:

```text
Value<String>
Value<Int>
Value<T>
Pair<T, T>
```

while `nominal` supplies ownership/candidate bucketing.

### 7.5 Trait-reference template

A source generic conformance stores the C3 `TraitRef` under the impl-local generic environment.

Example:

```phalcom
impl<T> Converter<T> for Box<T> { ... }
```

The stored trait reference is semantically a template because its argument refers to `TypeParameterOwner::Impl(impl_id)`.

Do not invent a separate AST-shaped “trait head” if canonical C3 `TraitRef` can already contain parameter forms.

### 7.6 Conformance contribution

Recommended semantic product:

```rust
pub struct ConformanceContribution {
    pub impl_id: ImplId,
    pub source_module: ModuleId,
    pub target: NominalTargetId,
    pub target_head: TypeId,
    pub trait_ref: TraitRef,
    pub generic_signature: Option<GenericSignature>,
    pub source: SemanticSourceSpan,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}
```

Optional mechanical metadata may be added for fingerprinting/query efficiency.

P1 `ConformanceContribution` must not contain:

```text
requirement -> witness map
selected defaults
associated type bindings
nested conformance evidence
conformance-owned CallableId values
runtime witness table
```

Those are P2/P3 products.

### 7.7 Candidate bucketing

A workspace index needs a coarse key that can find potentially applicable source conformances without scanning all implementations.

Recommended bucket identity:

```rust
ConformanceFamilyKey {
    target: NominalTargetId,
    trait_decl: DeclarationId,
}
```

For generic target declarations, the nominal target part is the declaration or exact variant identity, not the full exact `TypeId`.

The contribution retains the target and trait argument templates for subsequent joint matching.

The index may additionally maintain reverse indexes by trait declaration and by target declaration for source tooling and invalidation, but there must be one canonical contribution identity: `ImplId`.

### 7.8 Exact conformance key

At query time, form the exact relationship:

```rust
ExactConformanceKey {
    target: TypeId,
    trait_ref: TraitRef,
}
```

or the repository-equivalent snapshot-guarded type handle if the semantic DB requires store identity.

The exact key is not a new declaration ID. It is a semantic relationship key within a snapshot.

If raw `TypeId` cannot safely cross snapshot boundaries, use the repository's snapshot/store guarded wrapper or recompute from structural inputs. Never persist a naked snapshot-local type handle in a cross-snapshot cache.

### 7.9 Head matching

Candidate lookup specializes one source conformance head to an exact query by solving both sides under the same impl-local substitution:

```text
source target head
    Value<T>
source trait head
    Converter<T>

query target
    Value<Int>
query trait
    Converter<Int>

joint solution
    T := Int
```

The result is not conformance evidence. Recommended shape:

```rust
pub struct ConformanceHeadMatch {
    pub impl_id: ImplId,
    pub exact_target: TypeId,
    pub exact_trait_ref: TraitRef,
    pub bindings: TypeSubstitution,
    pub environment: TypeEnvironment,
}
```

The matching algorithm must use canonical type substitution/unification machinery. It must not compare rendered type strings or generic parameter names.

### 7.10 Query result

Do not return `bool`.

Recommended conceptual result:

```rust
pub enum ConformanceCandidateResult {
    Unique(ConformanceHeadMatch),
    Absent,
    Indeterminate(ConformanceIndeterminacy),
    Conflict(Arc<[ImplId]>),
}
```

In a coherence-clean accepted snapshot, `Conflict` should not occur for a fully exact supported query, but retaining an explicit state is safer for diagnostics and partially-invalid editor snapshots.

Do not name `Unique` as `Proven` conformance. Only the head relation is proven.

### 7.11 Ownership rule

Authorization is:

```text
impl_id.module == trait_ref.declaration.module
    OR
impl_id.module == target.declaration.module
```

For an exact enum case:

```text
target owner module = variant.owner.module
```

Re-export/import paths are irrelevant. Use canonical declaration identities after resolution.

Authorization must run after both trait and target resolve canonically, but before publication as an eligible contribution.

An unauthorized declaration may still produce a diagnostic/source product for tooling, but must not participate as an eligible candidate in conformance lookup or overlap with valid conformances.

### 7.12 Cross-module publication

Unlike inherent impls, legal conformance may live in either owner module:

```text
trait-owner module
or
target-owner module
```

Therefore a target-owned module snapshot cannot be the sole source of conformance candidates.

Use a workspace-level semantic product:

```text
module-local ConformanceContribution[]
        ↓
workspace ConformanceIndex
        ↓
family bucket(target nominal identity, trait declaration)
```

Publication must be deterministic and independent of:

```text
import order
module initialization order
source discovery order
hash-map order
trait-owner vs target-owner preference
```

### 7.13 Coherence

For any exact pair:

```text
exact target + exact TraitRef
```

there may be at most one applicable source conformance.

Overlap is determined by **joint satisfiability of both the target head and trait-reference head**.

Do not check only the target or only the trait reference.

Example — disjoint:

```phalcom
impl Tagged for Value<String> {}
impl Tagged for Value<Int> {}
```

No exact target satisfies both.

Example — overlap:

```phalcom
impl<T> Tagged for Value<T> {}
impl Tagged for Value<Int> {}
```

`Value<Int> + Tagged` satisfies both.

Example where target alone looks overlapping but pair is disjoint:

```phalcom
impl<T> Converter<T> for Value<T> {}
impl Converter<Int> for Value<String> {}
```

The first declaration at target `Value<String>` requires `Converter<String>`, while the second requires `Converter<Int>`. There is no exact `(target, TraitRef)` pair satisfying both, so the coherence checker must consider both halves jointly.

### 7.14 Generic alpha-renaming

Before pairwise overlap analysis, each source conformance's impl-local type parameters must remain distinct even if source names are equal.

`ImplId`-owned type parameter identity already supplies this distinction. Do not unify parameters by spelling.

### 7.15 No specialization precedence

If two conformance heads overlap, reject them.

Do not implement:

```text
exact beats generic
more constrained beats less constrained
trait-owner wins
target-owner wins
later source wins
first imported wins
```

C4 has no trait-conformance specialization system.

### 7.16 Conditional conformance boundary

C6 owns conditional conformance and trait constraints.

P1 supports generic head formation such as:

```phalcom
impl<T> Sized for List<T> {}
```

because generic parameters occur structurally in the head.

P1 does **not** implement applicability conditioned on a `where` clause such as:

```phalcom
impl<T> Printable for List<T>
where T: Printable
{
}
```

or another condition whose truth varies by exact substitution.

If the shared parser already accepts a `where` clause, P1 must preserve syntax/source information and issue the C4/C6-staging diagnostic required by the checkpoint. It must not silently ignore conditions or publish the conformance as unconditional.

If the final C4 checkpoint has ratified a narrower set of non-trait head constraints for C4, follow that higher authority; otherwise defer all condition-bearing conformance applicability to C6.

### 7.17 Requirement-body boundary

P1 parses and retains conformance body syntax because the complete source form must exist, but P1 does not resolve body members as witnesses.

Crucial negative invariant:

```text
impl Printable for User {
  toString { ... }
}
```

must not cause `User.toString` to appear in:

```text
DeclarationSurface(User)
ConditionalInherentMemberSet(User)
class runtime method table
inherent CallableId namespace
```

P2 will assign conformance-local callable identity and witness semantics.

---

## 8. Ownership boundaries

### 8.1 P1 owns

P1 owns:

- conformance `impl TraitRef for Target` syntax and AST normalization;
- conformance-vs-inherent impl kind distinction;
- impl-local generic scope shared by target and TraitRef;
- trait-reference validation as a trait;
- legal nominal target validation;
- exact enum-case conformance target resolution;
- source conformance provenance via `ImplId`;
- conformance head semantic product;
- trait-or-target authorization;
- cross-module conformance indexing;
- exact candidate lookup;
- conformance-head specialization/substitution;
- generic overlap/coherence;
- duplicate detection;
- incremental publication/replacement/removal;
- source/LSP identity sufficient to navigate the conformance declaration, trait side, and target side;
- P1 diagnostics;
- P1 semantic/incremental testing.

### 8.2 P1 does not own

P1 does not own:

- `TraitRequirementId` creation — C3;
- `TraitSurface` construction — C3;
- abstract trait default body checking — C3;
- witness compatibility — P2;
- conformance-owned callable identity — P2;
- data component witness selection — P2;
- inherited witness selection — P2;
- C2 conditional member reuse as witness — P2;
- default selection — P2;
- conformance completeness — P2;
- `ConformanceEvidence` — P2;
- ordinary trait-evidenced member lookup — P3;
- trait-default runtime execution — P3;
- associated type declaration/binding/projection — C5;
- generic trait bounds, nested conformance proof, conditional conformance — C6;
- trait objects/existentials/vtables — later;
- public reflection descriptors — later;
- supertraits — unratified/later;
- conformance specialization — unratified/forbidden in current model;
- metatype/class-side conformance — reserved;
- trait-qualified dispatch syntax — reserved.

### 8.3 Source-of-truth table

| Concern | Authority after P1 |
|---|---|
| source impl identity | `ImplId` |
| trait identity | C3 trait `DeclarationId` |
| trait application identity | C3 `TraitRef` |
| target nominal owner | declaration/`VariantId` identity |
| target generic head | resolved canonical `TypeId` template |
| conformance source head | `ConformanceContribution` |
| legal declaration ownership | canonical module identity comparison |
| cross-module candidate set | `ConformanceIndex` |
| exact source-head applicability | `ConformanceHeadMatch` |
| exact conformance candidate relationship | exact target + exact `TraitRef` query |
| requirement satisfaction | **not P1; P2 `ConformanceEvidence`** |
| trait-evidenced dispatch | **not P1; P3** |

---

## 9. Global invariants

The implementation and tests must preserve all of these.

### CF-01 — Explicit conformance

Structural member coincidence never creates conformance. Only an explicit applicable conformance declaration enters the index.

### CF-02 — Shared impl syntax, separate product

Inherent and conformance impls may share parser/generic machinery but never share their semantic contribution product.

### CF-03 — Source identity is `ImplId`

Do not invent an opaque conformance declaration identity in P1 when `ImplId` already identifies the source declaration.

### CF-04 — Exact application is not source identity

One generic `ImplId` may match many exact conformance queries.

### CF-05 — Exact TraitRef identity matters

`Converter<Int>` and `Converter<String>` are independent trait references.

### CF-06 — Exact target generic identity matters

`Value<Int>` and `Value<String>` may independently conform to the same trait.

### CF-07 — Nominal target requirement

Structural tuples, records, function types, and unrelated structural expressions are not direct conformance targets.

### CF-08 — Exact enum case identity is preserved

An exact case conformance is keyed to its `VariantId`/case type environment and does not imply enum-root conformance.

### CF-09 — Trait side must actually be a trait

A class/data/enum/type alias used in the trait position is rejected even if syntactically type-like.

### CF-10 — Trait-or-target ownership

A conformance is authorized only when its source module canonically owns the trait or target.

### CF-11 — Re-export does not transfer ownership

Imported/re-exported bindings cannot authorize a third-party conformance.

### CF-12 — Authorization and coherence are separate

Two conformances may each be individually authorized and still conflict globally.

### CF-13 — Global static composition

Conformance availability is determined by the semantic program, not runtime module load order.

### CF-14 — No inherent-surface injection

A conformance contribution never enters a target's inherent `DeclarationSurface` or conditional inherent member set.

### CF-15 — No witness proof in P1

The existence of a unique conformance head is not proof that requirements are satisfied.

### CF-16 — No boolean semantic collapse

C4 candidate/evidence APIs are structured products/results, not a single boolean.

### CF-17 — Joint overlap

Coherence overlap is evaluated over target and TraitRef simultaneously.

### CF-18 — Alpha-renamed generic identity

Generic parameter spelling cannot affect overlap or applicability.

### CF-19 — No conformance specialization

Overlapping applicable heads are invalid; no declaration wins by apparent specificity.

### CF-20 — Publication order independence

Candidate set and diagnostics are deterministic regardless of module/source traversal order.

### CF-21 — Owner-complete removal

Replacing/removing one source module removes all conformance contributions owned by that source module before replacement products become visible.

### CF-22 — Body-only edits do not change head identity

Editing only a conformance witness body does not alter its target, TraitRef, generic head, ownership, or coherence key.

### CF-23 — Header edits do change the right products

Editing trait arguments, target arguments, generic parameters, ownership-relevant declaration reference, or source module invalidates the conformance head/index/coherence results.

### CF-24 — Unsupported conditions never become unconditional

A deferred C6 conditional conformance must not be published as if its condition were absent.

### CF-25 — Snapshot-safe exact keys

Snapshot-local type handles are not reused across semantic snapshots without appropriate store/snapshot guarding.

### CF-26 — Invalid declarations do not pollute lookup

Unresolved, unauthorized, structurally illegal, or unsupported-condition conformances may remain source/diagnostic products but are not eligible exact candidates.

### CF-27 — C3 trait surface remains independent

P1 does not mutate `TraitSurface` based on any conformance.

### CF-28 — C2 inherent behavior remains independent

P1 does not mutate the target's inherent behavior based on any conformance.

### CF-29 — No runtime semantic authority

P1 introduces no runtime trait scan, runtime conformance registration, or class-table mutation.

### CF-30 — Incremental/cold equivalence

For the same source program, cold and incremental analysis produce equivalent conformance contributions, coherence diagnostics, and exact candidate query results.

---

## 10. Non-goals

Do not expand P1 to make attractive demos execute.

Explicitly out of scope:

```text
requirement witness maps
conformance-local CallableOwnerId
witness body type-checking as a trait witness
inherent member compatibility matching
data component witness matching
inherited witness matching
trait default fallback selection
conformance completeness
ordinary member syntax enabled by trait evidence
trait-evidenced callable references
compiler lowering of defaults through requirements
runtime witness/conformance tables
associated type declarations or bindings
Self::Associated projection
generic trait-bound syntax
nested conformance evidence
conditional conformance
supertraits
trait objects / existential values
trait-qualified calls
metatype conformance
conformance specialization
implicit/structural conformance
derived/synthesized conformance
```

If implementation of a P1 API seems to require one of these, STOP AND CONSULT. The likely problem is that the boundary has been crossed too early.

---

## 11. Expected impact map

The exact post-C3 paths must be verified at T0.

### 11.1 AST/parser

Expected reads/modifications:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
parser tests
```

Possible lexer/token changes are unlikely if `for` is already tokenized as language syntax, but verify rather than assume.

### 11.2 Modules/declaration shell

Expected reads, with only bounded modifications if required:

```text
phalcom-modules/src/declaration.rs
phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs
```

Conformance impls do not introduce named declarations, but module ownership/provenance and interface fingerprints may need new retained product inputs.

### 11.3 Semantic identity and impl architecture

Expected reads/modifications:

```text
phalcom-semantic/src/identity.rs
phalcom-semantic/src/impls.rs
post-C3 trait semantic module
new phalcom-semantic/src/conformance.rs or equivalent
phalcom-semantic/src/lib.rs
```

Prefer a dedicated conformance semantic owner over continuing to grow `impls.rs` into mixed inherent/conformance logic.

### 11.4 Workspace/incremental products

Expected reads/modifications:

```text
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/session.rs
semantic DB/query modules
semantic snapshot/public query APIs
```

### 11.5 Source/LSP

Expected reads/modifications:

```text
semantic source index
semantic target/occurrence products
phalcom-lsp projection/query adapters
```

Do not build P2 witness navigation yet.

### 11.6 Tests

Expected new or expanded test areas:

```text
phalcom-ast tests for conformance impl syntax
phalcom-semantic/tests/.../conformance/
phalcom-semantic incremental tests
module ownership/re-export fixtures
exact enum-case conformance tests
P1 generic specialization/coherence matrix
```

### 11.7 Documentation/state

Expected:

```text
LANG005.C4-CHECKPOINT.md
LANG005.C4.P1-walkthrough.md
LANG005.C4.P1-handoff.md
```

Normative specs should require no redesign in P1. Correct only demonstrable contradictions discovered during implementation, and record them explicitly.

### Unexpected-touch rule

If implementation requires substantial changes to VM dispatch, bytecode, runtime class tables, garbage collection, or object representation, STOP. P1 should be semantic/index infrastructure only.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer may not change these without consultation:

- explicit conformance syntax is `impl TraitReference for Target`;
- inherent and conformance impls are semantically distinct;
- source conformance identity remains `ImplId` in P1;
- exact relation identity includes both exact target and exact `TraitRef`;
- target application identity must distinguish `Value<Int>` and `Value<String>`;
- generic trait application identity must distinguish `Trait<Int>` and `Trait<String>`;
- legal conformance target categories are nominal declaration targets and exact enum cases;
- ownership is trait-or-target canonical module ownership;
- re-export does not transfer ownership;
- conformance publication is workspace-global/static;
- conformance does not mutate inherent surfaces;
- P1 does not construct witnesses/defaults/completeness evidence;
- overlap is joint target+TraitRef overlap;
- overlapping conformances are rejected;
- there is no most-specific conformance rule;
- conditional conformance remains C6;
- associated types remain C5;
- runtime lookup/registration is not semantic authority.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may choose based on the live repository:

- exact AST enum/field names;
- whether a neutral shared target enum is extracted from `InherentImplTarget` or a conformance wrapper is introduced;
- file/module placement of `ConformanceIndex`;
- exact names of bucket/query/result types;
- map/set container choice, provided deterministic externally visible results are preserved;
- whether coherence is validated during index construction or in a dedicated derived query;
- exact source-index target shape for an impl declaration;
- test file subdivision;
- whether exact candidate queries are memoized.

### 12.3 VERIFY-FIRST

These require repository verification before choosing:

- whether the final C3 AST already changed `ImplDef`;
- whether C3 added a neutral trait-reference syntax node;
- whether `TypeId` is safe as an in-snapshot exact key or needs `SnapshotTypeRef`;
- whether the semantic DB already has a reusable module-contribution aggregation product;
- whether C2 target unification is factored cleanly enough to reuse without coupling conformance to inherent semantics;
- whether `SemanticTargetId` should gain `Impl(ImplId)` now or whether source sites can target trait/target plus a conformance occurrence without adding a new global target category;
- whether the final C4 checkpoint permits any non-trait `where` constraints before C6;
- whether exact enum-case syntax is already generalized for conformance targets after C3.

---

## 13. Global STOP / CONSULT triggers

STOP AND CONSULT if any of these occur:

1. The final C3 implementation lacks a stable way to recognize/form `TraitRef` without nominalizing traits as ordinary types.
2. The pre-C4 proof-state closure did not land.
3. Implementing conformance syntax requires duplicating the entire inherent `impl` parser rather than normalizing a shared form.
4. The only available way to publish a conformance is to merge it into `DeclarationSurface`.
5. Cross-module semantic aggregation cannot discover contributions from both trait-owner and target-owner modules without a broader module architecture change.
6. Canonical target/trait head overlap cannot be decided for the P1 supported head language using existing type machinery.
7. The implementation is tempted to choose a more-specific conformance instead of diagnosing overlap.
8. Authorization appears to depend on import/re-export aliases rather than canonical declaration identity.
9. Exact enum-case target ownership cannot be recovered from `VariantId.owner` or equivalent canonical identity.
10. Source generic conformance would require a new `ConformanceId` solely to work around insufficient `ImplId` provenance; investigate first.
11. A conformance member must be assigned a target-owned `CallableId` for P1 to proceed.
12. A full runtime/VM conformance registry appears necessary.
13. Conditional conformance constraints are necessary to make a required P1 test pass; the test likely belongs to C6.
14. Associated type bindings become necessary; the test likely belongs to C5.
15. Cold and incremental results differ after module add/remove or header edit and the divergence cannot be isolated mechanically.
16. A change outside the expected impact map becomes architectural rather than mechanical.

---

## 14. Debugging budget

### Mechanical failures

Examples:

```text
exhaustive match missing new ImplKind variant
renamed post-C3 helper
source fingerprint field not copied in with_source
new module not exported
parser precedence bug
```

Resolve locally without consultation.

### Semantic failures

Examples:

```text
Value<Int> and Value<String> collapse into one candidate
trait-owner conformance not discovered from target query
re-export accidentally authorizes third party
exact/generic overlap not detected
```

Use one focused reproducer and trace the canonical identities/substitutions. Do not patch around the symptom with source-string comparisons.

### Architectural failures

Examples:

```text
need to inject members into target surface
need runtime registration
need to invent witness callables in P1
need source-order precedence
need a second generic solver
```

STOP AND CONSULT.

---

## 15. Testing surface analysis

P1 testing should be broad in semantic combinations but narrow in execution scope. The realistic executable `Iterable`/`Iterator` stress tests belong primarily to P2/P3 because P1 intentionally does not construct witnesses or dispatch. P1 must nevertheless build the conformance graph those later tests will depend on.

Use coverage IDs below in test names/comments or the P1 evidence table.

### 15.1 Syntax and normalization — SY

- **SY-01** parse empty `impl Printable for User {}`.
- **SY-02** parse generic trait application `impl Converter<Int> for User {}`.
- **SY-03** parse generic impl `impl<T> Tagged for Value<T> {}`.
- **SY-04** parse specialized target `impl Tagged for Value<Int> {}`.
- **SY-05** parse exact enum-case target.
- **SY-06** preserve body members without treating them as inherent members.
- **SY-07** retain source ranges for trait side, `for`, target, generics, body.
- **SY-08** existing inherent impl syntax remains unchanged.
- **SY-09** malformed missing trait/target/`for` diagnostics recover without parser corruption.

### 15.2 Resolution — RS

- **RS-01** trait side resolves to C3 trait declaration.
- **RS-02** non-trait declaration in trait position is rejected.
- **RS-03** unresolved trait reference is diagnosed.
- **RS-04** unresolved target is diagnosed.
- **RS-05** structural target is rejected.
- **RS-06** data/class/enum-root targets accepted.
- **RS-07** exact enum-case target accepted with canonical `VariantId`.
- **RS-08** impl-local generics are `TypeParameterOwner::Impl(ImplId)`.
- **RS-09** one impl generic environment resolves both target and trait arguments.
- **RS-10** parameter names do not affect semantic identity.

### 15.3 Ownership — OW

- **OW-01** conformance in trait owner module accepted.
- **OW-02** conformance in target owner module accepted.
- **OW-03** third-party module rejected.
- **OW-04** re-exporting trait does not confer ownership.
- **OW-05** re-exporting target does not confer ownership.
- **OW-06** exact enum-case target ownership derives from enum declaration module.
- **OW-07** two individually authorized conformances can still conflict.

### 15.4 Exact identity and specialization — GS

This is a first-class P1 category, not an edge case.

- **GS-01** `Value<String> + Tagged` and `Value<Int> + Tagged` are distinct candidate domains.
- **GS-02** exact target application survives nominal declaration sharing.
- **GS-03** unrelated `Value<Bool>` has no candidate.
- **GS-04** `User + Converter<Int>` and `User + Converter<String>` are distinct.
- **GS-05** one generic source conformance can match many exact applications with different substitutions.
- **GS-06** target and TraitRef must be matched using one consistent impl substitution.
- **GS-07** exact enum-case applications remain distinct from root and sibling cases.
- **GS-08** body-only edits do not change specialized head match.
- **GS-09** cold and incremental specialized queries agree.

### 15.5 Coherence — CH

- **CH-01** exact duplicate rejected.
- **CH-02** alpha-equivalent generic duplicate rejected.
- **CH-03** generic covering + exact specialized overlap rejected.
- **CH-04** two disjoint specialized targets accepted.
- **CH-05** same target + distinct exact TraitRefs accepted when domains are disjoint.
- **CH-06** joint target+TraitRef analysis avoids target-only false positive.
- **CH-07** exact enum-case/root/sibling overlap rules are correct.
- **CH-08** source order reversal yields same result.
- **CH-09** module discovery order reversal yields same result.
- **CH-10** trait-owner-vs-target-owner conflict has no precedence winner.
- **CH-11** no “more specific” winner exists.
- **CH-12** indeterminate overlap in supported P1 head language is not silently accepted.

### 15.6 Separation from inherent behavior — SP

- **SP-01** conformance contribution does not alter target `DeclarationSurface`.
- **SP-02** conformance contribution does not alter conditional inherent members.
- **SP-03** conformance body member is not callable through ordinary target lookup after P1 alone.
- **SP-04** conformance body member does not receive a target-owned `CallableId` in P1.
- **SP-05** trait surface is unchanged by adding/removing conformance.
- **SP-06** runtime/class method tables are unchanged by P1 semantic publication.

### 15.7 Incrementality — IN

- **IN-01** add conformance -> candidate appears.
- **IN-02** delete conformance -> candidate disappears.
- **IN-03** edit target type args -> old domain disappears/new domain appears.
- **IN-04** edit trait args -> old domain disappears/new domain appears.
- **IN-05** body-only edit preserves conformance-head/index identity where repository cache semantics permit.
- **IN-06** add overlapping conformance -> coherence diagnostic appears.
- **IN-07** delete overlapping conformance -> diagnostic disappears and unique candidate restores.
- **IN-08** move conformance from trait-owner to target-owner module remains legal.
- **IN-09** move to third-party module becomes unauthorized and disappears from eligible lookup.
- **IN-10** re-export edit does not alter ownership.
- **IN-11** trait declaration generic/header edit invalidates dependent heads.
- **IN-12** target declaration generic/header edit invalidates dependent heads.
- **IN-13** exact case removal/rename removes stale case conformance.
- **IN-14** cold and incremental diagnostics/query results are equivalent.

### 15.8 Source/tooling — LS

- **LS-01** conformance trait reference occurrence navigates to trait declaration.
- **LS-02** target occurrence navigates to target declaration/case.
- **LS-03** source index can identify the `ImplId`/conformance declaration occurrence.
- **LS-04** diagnostics point to the correct side for invalid trait, invalid target, ownership, and overlap.
- **LS-05** overlap diagnostics identify both source conformances deterministically.

### 15.9 Staging/negative boundaries — NG

- **NG-01** associated binding syntax remains unsupported until C5 if encountered.
- **NG-02** conditional conformance constraint remains unsupported until C6.
- **NG-03** trait-object/existential use is not enabled by P1.
- **NG-04** metatype conformance remains rejected/reserved.
- **NG-05** structural implicit conformance does not arise from matching members.
- **NG-06** P1 candidate match is not exposed as complete conformance evidence.

### 15.10 P1 complex semantic fixture

Create one multi-module semantic fixture that deliberately resembles the later P2/P3 stress ecosystem without relying on witness execution.

Recommended shape:

```text
traits module:
    trait Tagged
    trait Converter<T>
    trait Iterable<Item, Cursor>   // C4 testing form, no associated types

targets module:
    class Value<T>
    class Countdown
    enum Result<T> { Ok(T), Error(String) }

legal conformances split across both owner sides:
    Tagged for Value<String>
    Tagged for Value<Int>
    Converter<Int> for Value<String>
    Iterable<Int, Int> for Countdown
    exact-case conformance for Result<Int>::Ok(_)
```

Assert:

```text
candidate lookup identity
ownership
specialization substitution
exact case identity
no inherent-surface pollution
```

Then inject one overlapping generic conformance and verify deterministic coherence failure.

Do not attempt to execute trait behavior in this fixture until P2/P3.

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Run only the exact parser/unit/semantic test being developed plus the smallest predecessor regression filter directly touched.

#### STABILIZE MODE

At task boundaries, run the affected crate's focused conformance/impl/trait test groups.

#### CERTIFY MODE

At P1 gates, run the named focused suites and a bounded set of predecessor regressions. Do not reflexively run expensive unrelated runtime suites.

### 16.2 Verification ladder

Use this order:

```text
1. rustfmt / compilation of touched crate
2. exact new unit test filter
3. exact neighboring regression filter
4. focused crate test module
5. P1 gate suite
6. broader workspace verification only at final certification or if repository policy requires it
```

### 16.3 Mandatory during BUILD

After AST changes:

```text
phalcom-ast compile
exact conformance parser tests
existing inherent impl parser regression
```

After semantic target/head changes:

```text
exact RS/GS tests
one existing C2 impl target regression
one existing C3 TraitRef regression
```

After index/coherence changes:

```text
exact OW/CH tests
one add/remove incremental test
```

### 16.4 Do not repeatedly run

Unless evidence demands it, do not repeatedly run:

```text
full phalcom-core runtime acceptance corpus
full VM tests
GC stress
concurrency suites
unrelated LSP suites
cargo test --workspace after every edit
```

P1 should not touch runtime behavior.

---

## 17. Baseline / unrelated failure policy

Before major work, record the smallest relevant baseline commands and their result.

Classify failures:

```text
A — caused by current patch
B — exposed by current patch but pre-existing
C — pre-existing and unrelated
D — environment/tooling failure
```

Only A blocks the current task automatically.

B requires judgment: repair only if it is a direct prerequisite and remains within checkpoint authority; otherwise record/defer and consult if it prevents verification.

Do not “clean up” unrelated compiler/runtime debt while implementing conformance indexing.

---

# 18. Tasks

## T0 — Post-C3 re-grounding and C4 entry lock

### Purpose

Replace prospective planning assumptions with the actual completed C3 interfaces and prove all C4 entry conditions.

### Preconditions

- C3 implementation and final handoff exist.
- Executor has the actual working tree/revision.

### Consumes

- final C3 checkpoint/guidance/walkthrough/handoff;
- current repository status and commit;
- pre-C4 applicability proof-state repair;
- current specs.

### Produces

A short implementation log entry containing:

```text
actual starting revision
working-tree status
C3 stable interface map
C2 proof-state API
post-C3 impl AST shape
post-C3 shard/session ownership
verified test paths
any drift from this plan
```

### Files and symbols

Read only first. No production edits until the entry gate is satisfied.

### Required implementation shape

Verify each entry condition EC-01..EC-05.

Map the plan concepts to live symbols:

```text
TraitRef                        -> <live symbol>
trait declaration lookup       -> <live API>
TraitSurface                    -> <live symbol>
ImplDef                         -> <live AST>
ImplId                          -> <live symbol>
impl target resolver            -> <live API>
TypeSubstitution/unifier        -> <live API>
semantic module product owner   -> <live API>
```

Search for any provisional conformance implementation already landed in C3 and classify it.

### Forbidden approaches

- coding from the 20ad planning baseline as though C3 were absent;
- reimplementing a C3 API because its final name differs;
- beginning P1 with unresolved C2 proof-state debt.

### Test changes required

None.

### Tests to run now

Only enough to verify predecessor state:

```text
one final C3 trait declaration/TraitRef test
one C2 generic/specialized impl applicability test
one pre-C4 proof-state distinction test
```

Use live test names.

### Tests explicitly deferred

All P1 tests.

### Acceptance

The implementation log proves the real repository satisfies P1 entry conditions and identifies live symbols for every core dependency.

### Local STOP / CONSULT triggers

Any EC failure; material C3 drift; provisional conflicting conformance architecture.

### Checkpoint update

Record P1 start revision and entry condition status.

---

## T1 — Normalize `impl TraitRef for Target` syntax

### Purpose

Represent conformance syntax in the shared impl AST without semantic conflation with inherent impls.

### Preconditions

- T0 accepted.

### Consumes

- live `ImplDef` parser/AST;
- existing generic parameter and `where` syntax;
- `BehaviorMember` body representation.

### Produces

- explicit inherent vs conformance impl AST form;
- separate trait-reference and target syntax fields;
- parser recovery/source ranges;
- unchanged inherent syntax behavior.

### Files and symbols

Expected:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast tests
semantic_shard exhaustive Statement/Impl matches as compilation follow-up only
```

### Required implementation shape

Normalize around the conceptual `ImplKind` described in §7.2.

Parser examples that must work:

```phalcom
impl Printable for User {}
impl Converter<Int> for User {}
impl<T> Tagged for Value<T> {}
impl Tagged for Value<Int> {}
impl Printable for Result<Int>::Ok(_) {}
```

Preserve existing:

```phalcom
impl User {}
impl<T> Value<T> {}
```

The parser must not decide whether the left side is actually a trait; that is semantic resolution.

If `where` syntax is shared, retain it in AST. Do not implement conformance constraint semantics here.

### Forbidden approaches

- second top-level `ConformanceDef` statement unrelated to `ImplDef`;
- encoding `TraitRef for Target` as one synthetic type annotation;
- parser-time trait lookup;
- changing selector/member syntax.

### Test changes required

Implement SY-01..SY-09.

Include an AST assertion proving trait and target type annotations are independently retrievable.

### Tests to run now

Exact AST parser test target plus existing inherent impl parser regression.

### Tests explicitly deferred

Semantic trait/target validation, ownership, coherence.

### Acceptance

The AST losslessly distinguishes inherent and conformance impl source forms, and existing inherent syntax remains green.

### Local STOP / CONSULT triggers

Shared parser architecture cannot represent the two type-like heads without broad grammar redesign.

### Checkpoint update

Record stable conformance AST shape.

---

## T2 — Resolve conformance heads and canonical target identity

### Purpose

Resolve `TraitRef`, target, and impl-local generics into a P1 conformance head without publishing it globally yet.

### Preconditions

- T1 accepted.
- C3 trait lookup/TraitRef APIs verified.

### Consumes

- `ImplId` allocation/source ordinal;
- `TypeParameterOwner::Impl`;
- scoped type resolver;
- C3 `TraitRef` formation;
- nominal/exact-case target resolution;
- C2 generic target substitution machinery.

### Produces

A resolved header product equivalent to:

```rust
ResolvedConformanceHead {
    impl_id,
    source_module,
    trait_ref,
    target,
    target_head,
    generic_signature,
    source,
    diagnostics,
}
```

No witness/body semantics.

### Files and symbols

Expected:

```text
phalcom-semantic/src/impls.rs
new/existing conformance semantic module
phalcom-semantic/src/identity.rs if target identity factoring is needed
post-C3 traits module
semantic diagnostics
```

### Required implementation shape

#### A. Generic scope

Allocate impl-local generic parameters once:

```text
TypeParameterOwner::Impl(impl_id)
```

Use the same resolver scope for both:

```text
trait reference
and
target head
```

Example:

```phalcom
impl<T> Converter<T> for Value<T> {}
```

Both `T` occurrences resolve to the same impl-owned parameter identity.

#### B. Trait validation

Resolve left side using type/trait syntax, then require C3 declaration kind `Trait` and form canonical `TraitRef`.

Do not accept ordinary nominal types as trait references.

#### C. Target validation

Permit declaration-owned class/data/enum root and exact enum-case target categories specified by the trait spec.

Reject structural targets.

#### D. Exact/specialized target heads

Preserve applied target type, including concrete/generic arguments:

```text
Value<Int>
Value<String>
Value<T>
Pair<T, T>
```

Do not reduce to `DeclarationId(Value)`.

#### E. Neutral target identity

Introduce or adapt a neutral canonical nominal target identity as described in §7.4.

Avoid unnecessary C2 churn; the semantic role is fixed, mechanical factoring is flexible.

#### F. Staging of conditions

If the source conformance has a `where` clause outside C4 authority, mark the head ineligible and emit the staged unsupported diagnostic. Never drop the clause and publish an unconditional head.

### Forbidden approaches

- AST-string identity;
- parameter-name matching;
- target `DeclarationId` as the only generic target identity;
- treating a trait as an ordinary target nominal `TypeId` if C3 intentionally kept TraitRef separate;
- witness analysis.

### Test changes required

Implement RS-01..RS-10 and core GS identity tests.

### Tests to run now

Exact conformance-head resolution tests plus one C3 TraitRef regression and one C2 specialized target regression.

### Tests explicitly deferred

Ownership, global publication, overlap, witness/body checking.

### Acceptance

For every supported conformance source head, semantic analysis produces a canonical target template + canonical `TraitRef` template under one impl-local generic environment, or a precise diagnostic.

### Local STOP / CONSULT triggers

Canonical C3 TraitRef cannot contain impl-owned parameter forms; exact target heads cannot be represented without source-string patterns; target neutralization requires broad C2 rewrite.

### Checkpoint update

Record stable resolved conformance-head interface.

---

## T3 — Enforce trait-or-target ownership

### Purpose

Apply the orphan/ownership rule using canonical semantic provenance.

### Preconditions

- T2 canonical trait and target identities exist.

### Consumes

- `impl_id.module`;
- `trait_ref.declaration.module` or live equivalent;
- target declaration/variant owner module.

### Produces

- authorization result;
- precise diagnostic/source ranges;
- eligible/ineligible publication distinction.

### Files and symbols

Expected conformance semantic module, diagnostics, module ownership helpers, semantic tests.

### Required implementation shape

Authorization predicate:

```text
source_module == trait_owner_module
OR
source_module == target_owner_module
```

For exact cases:

```text
target_owner_module = variant.owner.module
```

Compute ownership after symbol resolution so aliases/re-exports cannot spoof it.

A re-exported binding resolves to the original declaration identity and therefore preserves original ownership.

Keep authorization distinct from visibility and coherence.

### Forbidden approaches

- checking lexical import path/source spelling;
- treating re-export module as owner;
- allowing “same package” as ownership unless separately ratified;
- preferring trait owner over target owner when both contain conformances.

### Test changes required

Implement OW-01..OW-07, including multi-module re-export fixtures.

### Tests to run now

Exact ownership test group only.

### Tests explicitly deferred

Global overlap until T6.

### Acceptance

Every resolved conformance head has deterministic authorization based solely on canonical source/trait/target module identity, with re-export behavior proven.

### Local STOP / CONSULT triggers

Canonical declaration ownership is not recoverable after linker resolution; module aliases obscure declaration provenance.

### Checkpoint update

Record ownership predicate and diagnostic codes.

---

## T4 — Publish `ConformanceContribution` and workspace index

### Purpose

Create the static cross-module product that makes legal source conformance declarations discoverable regardless of which ownership side declares them.

### Preconditions

- T2 head resolution accepted.
- T3 authorization accepted.

### Consumes

- module-local resolved conformance heads;
- semantic shard/session product model;
- module replacement/removal lifecycle.

### Produces

- `ConformanceContribution`;
- module-local contribution set keyed by source module/`ImplId`;
- workspace `ConformanceIndex`;
- coarse family buckets by nominal target + trait declaration;
- reverse indexes if required for invalidation/tooling.

### Files and symbols

Expected:

```text
new/existing conformance semantic module
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/session.rs
semantic DB/query layer
phalcom-semantic/src/lib.rs
```

### Required implementation shape

#### A. Contribution construction

Publish only well-formed, authorized, P1-supported heads as eligible contributions.

Retain invalid source diagnostics separately so editor snapshots remain informative.

#### B. Canonical bucket

At minimum:

```text
NominalTargetId + TraitDeclarationId -> [ImplId / contribution]
```

Do not bucket only by target or only by trait.

#### C. Cross-module aggregation

A trait-owner contribution must be found when querying a target declared elsewhere, and vice versa.

#### D. Determinism

Sort or otherwise canonicalize iteration used for externally visible diagnostics/results. Do not let hash-map order determine conflict pair ordering.

#### E. Owner-complete lifecycle

On module replacement/removal:

```text
remove all old contributions whose ImplId.module == module
then publish replacement contribution set
then derive affected coherence/query products
```

Avoid stale entries.

### Forbidden approaches

- target module owns the only index;
- trait module owns the only index;
- runtime registration;
- append-only index without removal;
- storing witness/body-derived data.

### Test changes required

Implement IN-01/IN-02 foundations, trait-owner/target-owner discovery, SP-01/SP-05.

### Tests to run now

Focused index publication/add/remove tests plus one module replacement regression.

### Tests explicitly deferred

Exact specialization query (T5), global coherence (T6).

### Acceptance

Workspace snapshots contain exactly the eligible source conformance contributions from current modules, discoverable through canonical family buckets and removable without staleness.

### Local STOP / CONSULT triggers

Existing semantic product model has no owner-complete workspace aggregation seam and fixing it would require redesign beyond C4.

### Checkpoint update

Record index owner, bucket key, and lifecycle API.

---

## T5 — Exact candidate lookup and generic head specialization

### Purpose

Given an exact target and exact `TraitRef`, determine which indexed source conformance heads apply and produce structured head-match substitution.

### Preconditions

- T4 index available.

### Consumes

- exact target `TypeId` / snapshot-safe type ref;
- exact C3 `TraitRef`;
- family bucket;
- target/trait generic unification;
- `TypeEnvironment` / `TypeSubstitution`.

### Produces

- `ConformanceHeadMatch`;
- `ConformanceCandidateResult` or equivalent structured query;
- exact specialization tests.

### Files and symbols

Conformance semantic owner, type substitution/unification helpers, semantic public query API, tests.

### Required implementation shape

#### A. Form family key

Recover nominal target identity and trait declaration from the exact query, then fetch only relevant candidates.

#### B. Joint matching

For each source contribution, solve one impl-local substitution that simultaneously satisfies:

```text
source target head == exact target
source TraitRef head == exact TraitRef
```

Do not match them independently and merge contradictory substitutions later.

#### C. Return substitution evidence

Retain the bindings/environment. P2 will need the exact source-head instantiation to instantiate requirement signatures and conformance-local bodies.

#### D. Exact target specialization test

Mandatory regression:

```phalcom
impl Tagged for Value<String> { ... }
impl Tagged for Value<Int> { ... }
```

Queries must resolve independently.

#### E. Generic source test

Mandatory regression:

```phalcom
impl<T> Tagged for Value<T> { ... }
```

`Value<Int> + Tagged` and `Value<String> + Tagged` each produce the same `ImplId` with different substitutions.

#### F. Trait-argument identity test

Mandatory regression:

```phalcom
impl Converter<Int> for User { ... }
impl Converter<String> for User { ... }
```

Exact queries select corresponding source heads.

### Forbidden approaches

- lookup keyed only by `DeclarationId`;
- lookup keyed only by trait declaration;
- string comparison of rendered generic args;
- returning true/false;
- claiming trait completeness.

### Test changes required

Implement GS-01..GS-09 except overlap-specific cases, plus NG-06.

### Tests to run now

Exact specialization query group plus existing type-unification regression filters directly reused.

### Tests explicitly deferred

Overlap diagnostics until T6; witness selection until P2.

### Acceptance

Exact query produces a structured source-head match with canonical generic substitution, correctly distinguishing both target and trait generic applications.

### Local STOP / CONSULT triggers

Joint target+TraitRef matching requires a parallel generic solver; supported P1 patterns produce unresolved ambiguity that existing type machinery cannot represent safely.

### Checkpoint update

Record exact query/result API for P2 consumption.

---

## T6 — Global coherence and overlap validation

### Purpose

Reject any pair of eligible source conformance heads for which at least one exact `(target, TraitRef)` pair lies in both domains.

### Preconditions

- T4 contributions available.
- T5 head matching semantics established.

### Consumes

- family buckets;
- canonical type unification/substitution;
- source diagnostics/provenance.

### Produces

- deterministic overlap diagnostics;
- coherence-clean candidate sets;
- no specialization precedence.

### Files and symbols

Conformance semantic owner, diagnostics, DB/index derivation, tests.

### Required implementation shape

#### A. Candidate pairs

Only compare contributions in the same coarse family bucket:

```text
same nominal target identity
same trait declaration
```

This bounds pairwise work.

#### B. Alpha separation

Each impl already owns distinct type parameters through `ImplId`; preserve that identity during pair unification.

#### C. Joint overlap solve

Ask whether there exists a substitution for each impl's parameters such that:

```text
target_head_A == target_head_B
AND
trait_ref_A == trait_ref_B
```

under the same exact relationship.

#### D. No false target-only conflicts

Test a pair where target heads can overlap but trait arguments force disjoint exact references.

#### E. No most-specific rule

Generic+exact overlap is an error.

#### F. Invalid contributions

Unauthorized/unresolved/staged-unsupported contributions do not participate as eligible candidates. Their own diagnostics remain.

#### G. Editor-invalid snapshots

Keep conflict information structured enough that exact candidate lookup can report a conflict in an invalid snapshot rather than arbitrarily selecting one.

### Forbidden approaches

- source-text syntactic overlap only;
- target-only overlap;
- trait-only overlap;
- sorting by specificity and picking first;
- “last declaration wins”;
- silently dropping one conflicting source contribution.

### Test changes required

Implement CH-01..CH-12 and OW-07.

Required table includes at least:

| A | B | Expected |
|---|---|---|
| `Tagged for Value<Int>` | same exact | conflict |
| `Tagged for Value<Int>` | `Tagged for Value<String>` | disjoint |
| `Tagged for Value<T>` | `Tagged for Value<Int>` | conflict |
| `Converter<T> for Value<T>` | `Converter<Int> for Value<String>` | disjoint |
| `Converter<T> for Value<T>` | `Converter<Int> for Value<Int>` | conflict |
| same exact relation declared in trait-owner and target-owner modules | conflict |

### Tests to run now

Focused coherence matrix only, then one incremental add/remove conflict test.

### Tests explicitly deferred

Conditional-domain overlap with trait constraints to C6.

### Acceptance

The workspace accepts exactly coherence-safe conformance heads for the P1 language and produces deterministic paired diagnostics for every overlap, independent of order.

### Local STOP / CONSULT triggers

Overlap of supported heads is undecidable/unknown under current solver; solution would require C6 constraint semantics; implementation pressure toward specialization precedence.

### Checkpoint update

Record coherence algorithm and diagnostic codes.

---

## T7 — Incremental fingerprints, replacement/removal, and source/LSP projection

### Purpose

Make P1 products stable under edits and expose conformance declaration relationships without rebuilding witness semantics that do not exist yet.

### Preconditions

- T4–T6 semantic model stable.

### Consumes

- post-C3 semantic shard fingerprint design;
- module replacement/removal lifecycle;
- source index/semantic occurrence APIs.

### Produces

- conformance head fingerprint;
- body-independent head invalidation;
- owner-complete index replacement;
- source navigation for impl, trait reference, target;
- cold/incremental parity tests.

### Files and symbols

Expected:

```text
phalcom-semantic/src/semantic_shard.rs
session/DB query layer
source index/semantic targets
phalcom-lsp adapters if required
incremental semantic tests
```

### Required implementation shape

#### A. Fingerprint separation

Conformance head fingerprint includes semantically relevant source structure:

```text
impl generic parameter declarations
trait-reference syntax
for target syntax
supported header-level conditions/staging state
```

It excludes witness body text.

A body-only edit may change body fingerprint/source snapshot but must not force a new conformance domain/coherence result when header is unchanged.

#### B. Replace/remove atomically

When source module changes, ensure no old contribution survives replacement.

#### C. Dependency invalidation

Changes to trait/target declarations that alter generic resolution or declaration identity must invalidate dependent conformance heads even if the impl source text did not change.

#### D. Source index

At minimum make these source relations available:

```text
impl declaration occurrence -> ImplId or snapshot-local conformance source site
trait reference occurrence -> trait DeclarationId
conformance target occurrence -> target DeclarationId/VariantId
```

Do not implement requirement-to-witness navigation yet.

#### E. Diagnostic stability

Overlap diagnostics should consistently choose/display both impl source spans independent of map traversal order.

### Forbidden approaches

- source fingerprint includes full body and unnecessarily rebuilds global coherence for body edits;
- stale append-only contribution retention;
- LSP reconstructs trait/target relationship from syntax instead of semantic products.

### Test changes required

Implement IN-01..IN-14 and LS-01..LS-05.

### Tests to run now

Focused incremental conformance group plus exact source navigation tests.

### Tests explicitly deferred

P2 witness source projection and P3 dispatch navigation.

### Acceptance

All supported add/edit/delete/move/re-export scenarios produce cold-equivalent contributions, diagnostics, and exact candidate lookup, and body-only edits do not perturb the conformance head unnecessarily.

### Local STOP / CONSULT triggers

Cross-module dependency invalidation cannot observe trait/target header changes; source product architecture would require global rebuild only and contradicts current incremental design goals.

### Checkpoint update

Record fingerprint/invalidation keys and source projection API.

---

## T8 — P1 adversarial conformance corpus and separation tests

### Purpose

Exercise the complete P1 semantic path in realistic combinations before handing the graph to witness resolution.

### Preconditions

- T1–T7 complete.

### Consumes

- parser;
- conformance head resolution;
- ownership;
- index;
- exact lookup;
- coherence;
- incrementality.

### Produces

- one positive complex multi-module P1 fixture;
- one negative/adversarial coherence fixture or equivalent test matrix;
- complete coverage evidence for SY/RS/OW/GS/CH/SP/IN/LS/NG IDs.

### Required implementation shape

#### A. Generic specialization centerpiece

Mandatory positive scenario:

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

P1 assertions are semantic, not executable:

```text
query Value<String> + Tagged -> String specialization ImplId
query Value<Int> + Tagged    -> Int specialization ImplId
query Value<Bool> + Tagged   -> absent
DeclarationId(Value) is shared but exact target keys do not collide
body members did not enter inherent surface
```

The string/int runtime return assertions are deliberately deferred to P2/P3.

#### B. Generic overlap mutation

Add:

```phalcom
impl<T> Tagged for Value<T> {
  tag -> String { "generic" }
}
```

and assert overlap against both exact conformances, with no exact-over-generic preference.

#### C. Trait-reference dimension

Add a generic trait scenario proving same target can have disjoint exact TraitRefs.

#### D. Ownership dimension

Split legal conformances across trait-owner and target-owner modules, plus a third-party illegal attempt.

#### E. Exact enum-case dimension

Add one exact case conformance and prove root/sibling independence.

#### F. Iterable-shaped future handoff fixture

Include a C4-only generic-parameter trait declaration:

```phalcom
trait Iterable<Item, Cursor> {
  iterate(_ cursor: Option<Cursor>) -> Option<Cursor>
  iteratorValue(_ cursor: Cursor) -> Item
  // C3 defaults may exist
}
```

and an explicit source conformance head such as:

```phalcom
impl Iterable<Int, Int> for Countdown { ... }
```

P1 asserts only head/index/coherence facts. P2/P3 will inherit the fixture and add witness/default/execution assertions.

### Forbidden approaches

- enabling execution by prematurely implementing witnesses;
- weakening P1 negative boundaries to make the fixture compile/run end-to-end;
- associated types in the C4 Iterable fixture (C5 owns them).

### Tests to run now

The exact P1 complex fixture group plus the focused coherence and incremental groups.

### Tests explicitly deferred

Runtime execution, default-to-requirement calls, bound trait-evidenced references.

### Acceptance

A realistic multi-module generic conformance graph behaves correctly under specialization, ownership, exact TraitRefs, exact cases, overlap mutation, and incremental updates without any inherent-surface pollution.

### Local STOP / CONSULT triggers

Fixture can only be made meaningful by crossing into P2 witness semantics.

### Checkpoint update

Record P1 complex fixture as the named handoff oracle for P2.

---

## T9 — Stabilization, P1 closure, walkthrough, and P2 handoff

### Purpose

Certify the P1 slice and document the exact APIs P2 must consume.

### Preconditions

- T1–T8 accepted.

### Consumes

- all P1 tests/evidence;
- final code diff;
- C4 checkpoint.

### Produces

```text
LANG005.C4.P1-walkthrough.md
LANG005.C4.P1-handoff.md
updated LANG005.C4-CHECKPOINT.md
coverage-to-test evidence table
known-debt list
```

### Required implementation shape

Walk the final architecture from source to exact candidate query:

```text
source impl
→ AST ImplKind::Conformance
→ resolved conformance head
→ ownership
→ module contribution
→ workspace index
→ coherence
→ exact candidate query
→ ConformanceHeadMatch
```

Explicitly demonstrate that no witness/default/completeness evidence exists yet.

The P2 handoff must name the stable interfaces for:

```text
retrieve exact candidate head match
retrieve source ImplId/AST body
retrieve exact target
retrieve exact TraitRef
retrieve impl substitution/environment
query TraitSurface
query effective inherent behavior
```

### Forbidden approaches

- claiming C4 checkpoint complete;
- claiming any target “conforms” based only on P1 candidate match;
- hiding deferred P2 obligations behind green parser/index tests.

### Test changes required

Only stabilization regressions justified by failures found during certification.

### Tests to run now

Run G1–G5 below.

### Acceptance

P1 is fully documented, focused tests are green, cold/incremental behavior is equivalent, and P2 can build witness evidence without reopening conformance identity/ownership/coherence architecture.

### Local STOP / CONSULT triggers

Any invariant still relies on undocumented source order, boolean conformance state, or inherent-surface mutation.

### Checkpoint update

Mark C4.P1 complete only; leave C4 overall in progress.

---

# 19. Verification gates

## G1 — Entry and syntax gate

Requires:

```text
EC-01..EC-05 proven
T1 accepted
SY-01..SY-09 green
existing inherent impl parser regression green
```

Gate statement:

> The repository is genuinely ready for C4, and `impl TraitRef for Target` is represented losslessly without changing inherent semantics.

## G2 — Canonical conformance-head gate

Requires:

```text
T2 + T3 accepted
RS-01..RS-10 green
OW-01..OW-06 green
GS basic exact target/TraitRef identity green
```

Gate statement:

> Conformance source heads resolve canonically under impl-local generics and are authorized solely by canonical trait/target ownership.

## G3 — Publication and exact-query gate

Requires:

```text
T4 + T5 accepted
workspace cross-owner discovery green
GS-01..GS-09 green
SP-01..SP-05 green
```

Gate statement:

> Exact target + exact TraitRef queries find structured source-head matches without collapsing generic target identity or mutating inherent/trait surfaces.

## G4 — Coherence and incrementality gate

Requires:

```text
T6 + T7 accepted
CH-01..CH-12 green
IN-01..IN-14 green
LS-01..LS-05 green
cold/incremental parity established
```

Gate statement:

> The workspace has deterministic global conformance coherence and owner-complete lifecycle behavior independent of order and re-export topology.

## G5 — P1 certification gate

Requires:

```text
T8 complex fixture green
all P1 coverage IDs accounted for
no runtime/VM changes required
walkthrough + handoff written
checkpoint truthful
```

Gate statement:

> C4.P1 supplies a stable, coherence-safe conformance graph and exact head-match API ready for P2 witness construction, while making no false claim of trait satisfaction.

---

## 20. Final focused acceptance

### 20.1 Required positive facts

At final acceptance, demonstrate all of the following in tests or walkthrough evidence:

1. `impl Printable for User {}` parses as a conformance impl, not an inherent impl.
2. Trait and target resolve separately and canonically.
3. Impl generics are owned by `ImplId` and shared across target/TraitRef resolution.
4. Trait-owner and target-owner modules may each legally declare conformances.
5. Third-party modules may not.
6. Re-export does not transfer ownership.
7. Conformance contributions are workspace-discoverable from exact queries.
8. `Value<String>` and `Value<Int>` conformances do not collide merely because they share `DeclarationId(Value)`.
9. `Converter<Int>` and `Converter<String>` remain distinct exact TraitRefs.
10. One generic source head can specialize to several exact queries with different substitutions.
11. Exact enum-case conformance remains case-specific.
12. Generic+exact overlapping conformances are rejected.
13. Joint target+TraitRef overlap avoids target-only false positives.
14. No source/module order selects a winner.
15. Add/delete/header edits update candidate/coherence results without stale contributions.
16. Body-only edits do not alter conformance-head/coherence identity unnecessarily.
17. Conformance body members are absent from inherent surfaces.
18. No `ConformanceEvidence` or boolean “conforms” proof is produced in P1.

### 20.2 Required negative verification

Search/review final diff for these anti-patterns:

```text
DeclarationSurface.add(conformance member)
ConditionalInherentMemberSet <- conformance member
class method table mutation from conformance
runtime conformance registration
is_conforming: bool as semantic authority
ConformanceId introduced without demonstrated need
source-order winner
most-specific conformance winner
import-order winner
re-export ownership
stringified type equality for coherence
where clause silently ignored
associated type binding implementation
witness/default selection in P1
```

Any such finding blocks P1 closure unless explicitly justified by a higher-authority design change.

### 20.3 Final acceptance rule

P1 is accepted only if:

```text
all fixed invariants hold
AND
all G1–G5 gates pass
AND
all P1 coverage IDs are either green or explicitly waived by user-ratified scope change
AND
cold/incremental parity holds
AND
walkthrough/handoff are complete
AND
P2 can consume P1 without reopening identity/ownership/coherence
```

---

## 21. Performance/resource evidence

P1 is primarily semantic infrastructure, but avoid obvious scaling traps.

### 21.1 Candidate lookup

Exact query should not scan every conformance in the workspace.

Expected coarse bound:

```text
lookup family bucket by nominal target + trait declaration
then match only candidates in that bucket
```

### 21.2 Coherence

Pairwise overlap may be O(n²) within one family bucket. That is acceptable initially if family buckets are expected to remain small and evidence is recorded.

Do not perform O(total_conformances²) workspace-wide comparison.

If a family bucket becomes large in measured workloads, record performance debt rather than inventing unsafe specificity heuristics.

### 21.3 Incremental invalidation

A body-only witness edit should not trigger global coherence recomputation for unchanged heads.

A conformance-head edit should invalidate only affected family buckets and dependent exact queries where the DB architecture supports that granularity.

### 21.4 Memory

Do not duplicate full AST bodies in the conformance index. Retain source `ImplId`/semantic source handle and use the existing source shard to retrieve body syntax when P2 needs it.

---

## 22. Checkpoint bookkeeping

### At plan start

Record:

```text
starting revision
working tree state
final C3 completion revision
entry-condition status
actual live symbol mapping
```

### During plan

After each durable gate, update C4 checkpoint with:

```text
landed task/gate
stable API names
new diagnostics
coverage IDs proven
known deviations from plan
unresolved issues
```

### At plan completion

Record:

```text
C4.P1 COMPLETE
C4 checkpoint IN_PROGRESS
final revision
verification commands/results
complex P1 fixture path
walkthrough path
P2 handoff path
remaining C4.P2/P3 scope
```

Do not mark C4 itself complete.

---

## 23. Walkthrough deliverable

`LANG005.C4.P1-walkthrough.md` must explain the landed implementation, not restate this plan.

Required sections:

1. final revision and write set;
2. conformance AST shape;
3. source identity and generic ownership;
4. target/TraitRef resolution;
5. ownership rule implementation;
6. contribution/index architecture;
7. exact candidate query and substitution;
8. coherence algorithm with at least three examples;
9. incremental lifecycle;
10. source/LSP projection;
11. generic specialization example `Value<String>` vs `Value<Int>`;
12. proof that inherent/trait surfaces are unchanged;
13. verification evidence;
14. known debt/non-goals;
15. stable APIs handed to P2.

Include concrete symbol/file references from the landed repository.

---

## 24. Handoff deliverable

`LANG005.C4.P1-handoff.md` should be concise enough for a fresh P2 agent to load quickly but exact enough to prevent architectural regression.

It must state:

### What P1 guarantees

```text
source conformance identity = ImplId
canonical target template
canonical TraitRef template
trait-or-target authorization
workspace candidate publication
exact candidate lookup
impl substitution/head-match evidence
global coherence
incremental lifecycle
```

### What P1 explicitly does not guarantee

```text
requirement satisfaction
witness compatibility
conformance completeness
default selection
trait-evidenced dispatch
runtime execution
```

### P2 stable inputs

Name exact landed APIs for:

```text
exact candidate query
ConformanceContribution lookup by ImplId
ConformanceHeadMatch substitution/environment
TraitSurface query
source ImplDef/body retrieval
target effective inherent lookup
C2 proof-state API
```

### P2 architectural warning

Include this explicitly:

> A unique P1 conformance-head match is not yet conformance evidence. P2 must validate every behavioral requirement and construct structured `ConformanceEvidence`; it must not replace that work with `candidate.is_some()`.

---

## 25. Completion truth table

| Capability | Before P1 | After P1 | Later |
|---|---:|---:|---:|
| parse trait declaration | C3 | yes | — |
| build TraitRef | C3 | yes | — |
| build TraitSurface | C3 | yes | — |
| parse `impl Trait for Target` | no | yes | — |
| resolve exact/generic conformance head | no | yes | — |
| trait-or-target ownership | no | yes | — |
| cross-module conformance index | no | yes | — |
| exact target + TraitRef candidate lookup | no | yes | — |
| generic conformance head specialization | no | yes | — |
| global overlap/coherence | no | yes | — |
| distinguish `Value<Int>`/`Value<String>` conformance | no | yes | — |
| witness selection | no | no | C4.P2 |
| conformance-local witness callable | no | no | C4.P2 |
| data/inherited/conditional witness | no | no | C4.P2 |
| default selection | no | no | C4.P2 |
| complete `ConformanceEvidence` | no | no | C4.P2 |
| ordinary trait-evidenced dispatch | no | no | C4.P3 |
| trait default execution | no | no | C4.P3 |
| associated types | no | no | C5 |
| conditional conformance | no | no | C6 |
| trait objects/vtables | no | no | later |

---

## 26. Plan self-review

Before implementation begins, the planner/implementer should be able to answer “yes” to all of these:

- Does the plan preserve C3 trait identity rather than turning traits into classes?
- Does it preserve C2 inherent behavior as a separate semantic product?
- Does it treat `ImplId` as source provenance rather than exact conformance identity?
- Can one generic source impl match multiple exact applications?
- Can the model distinguish `Value<String>` from `Value<Int>` despite one declaration identity?
- Can it distinguish `Converter<String>` from `Converter<Int>` despite one trait declaration identity?
- Does ownership use canonical declaration modules rather than import aliases?
- Can a legal conformance declared on the trait side be discovered from a target query?
- Can a legal conformance declared on the target side be discovered from a trait query?
- Does coherence jointly analyze target and TraitRef?
- Is generic+exact overlap rejected rather than specialized?
- Can invalid/editor snapshots represent conflict without arbitrarily choosing a winner?
- Are body-only edits isolated from head/coherence identity?
- Does deletion remove stale contributions?
- Is exact enum-case identity preserved?
- Are structural targets rejected?
- Is conditional conformance deferred rather than silently made unconditional?
- Is P1 incapable of claiming requirement satisfaction?
- Does P1 avoid conformance-owned callables so P2 can design them deliberately?
- Does the test plan include the specialized `Value<String>`/`Value<Int>` case as a central regression?
- Is there a clear P2 handoff API that avoids reopening identity/ownership/coherence?

If any answer is “no”, do not begin the affected task until the gap is resolved.

---

# Appendix A — Canonical examples P1 must model

## A.1 Two specialized targets, same trait

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

P1 semantic result:

```text
ImplId A
    target head = Value<String>
    TraitRef = Tagged

ImplId B
    target head = Value<Int>
    TraitRef = Tagged

query(Value<String>, Tagged) -> head match A
query(Value<Int>, Tagged)    -> head match B
query(Value<Bool>, Tagged)   -> absent
```

P1 does **not** yet prove that either `tag` body is a valid witness or execute the returned string. That becomes P2/P3 acceptance.

## A.2 Generic head

```phalcom
impl<T> Tagged for Value<T> {
  tag -> String { "value" }
}
```

P1 can specialize:

```text
Value<Int>    -> T := Int
Value<String> -> T := String
```

The same `ImplId` is provenance for both exact head matches.

## A.3 Forbidden specialization overlap

```phalcom
impl<T> Tagged for Value<T> {}
impl Tagged for Value<Int> {}
```

Reject: exact relation `Value<Int> + Tagged` has two applicable source heads.

## A.4 Distinct trait references

```phalcom
trait Converter<T> {
  convert -> T
}

impl Converter<Int> for User { ... }
impl Converter<String> for User { ... }
```

Distinct exact `TraitRef`s; no duplicate solely because trait declaration is the same.

## A.5 Joint overlap, not target-only overlap

```phalcom
impl<T> Converter<T> for Value<T> {}
impl Converter<Int> for Value<String> {}
```

For the first declaration at `Value<String>`, the trait reference is `Converter<String>`. Therefore there is no exact relation shared with `Value<String> + Converter<Int>`.

A target-only overlap detector would be wrong.

## A.6 Cross-owner conflict

```text
module traits owns Printable
module model owns User

traits module:
    impl Printable for User {}

model module:
    impl Printable for User {}
```

Each is authorized. Together they violate coherence.

## A.7 Re-export is not ownership

```text
module facade re-exports Printable and User

facade:
    impl Printable for User {}
```

Reject because canonical owners remain the original trait/target modules.

---

# Appendix B — Required diagnostics families

Use repository naming conventions, but P1 must have distinct diagnostics for these semantic causes:

```text
conformance trait reference unresolved
conformance trait position is not a trait
conformance target unresolved
conformance target category illegal/structural
conformance exact case invalid
conformance unauthorized by trait-or-target rule
conformance condition unsupported until C6
conformance duplicate/overlap
conformance head generic resolution conflict
conformance query indeterminate in invalid semantic state, if exposed diagnostically
```

Diagnostics should preserve source specificity. For overlap, identify both declarations and the conflicting relationship/domain where practical.

Do not report a P1 overlap as a witness conflict; witnesses have not been analyzed yet.

---

# Appendix C — P2 handoff preview

P2 should begin from this exact conceptual input:

```text
exact target
    +
exact TraitRef
    +
P1 unique ConformanceHeadMatch
        impl_id
        impl substitution/environment
    +
C3 TraitSurface
    +
target effective inherent behavior
        ↓
requirement-by-requirement witness selection
        ↓
ConformanceEvidence
```

P2 must not need to answer any of these again:

```text
Which source conformance heads exist?
Who owns them?
Are they legal to declare here?
Which generic head applies to this exact target/TraitRef?
Do two source conformance domains overlap?
How are conformance additions/removals indexed incrementally?
```

If P2 has to reopen those questions, P1 is not truly complete.
