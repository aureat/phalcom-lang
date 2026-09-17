---
id: LANG005.C6.P4.REQUIREMENTS
category: LANG
program: LANG005
checkpoint: LANG005.C6
plan: LANG005.C6.P4
kind: requirements-analysis
status: ANALYZED
completion: NOT_STARTED
verification: REPOSITORY_AND_REFERENCE_RESEARCHED
depends_on:
  - LANG005.C6.P3
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: 2da270877ee34693357784d9e2077f63da721802
planning_revision_subject: "feat(lang): implement conditional conformance proof evidence"
---

# LANG005.C6.P4 Requirements Analysis

## Iterator / Iterable Generic Laws, Guarded Derived Conformance, Trait-Contract Entailment, Canonical Universe Staging, and P3 Closure Integration

---

# 0. Purpose

This document is the architectural and requirements-analysis input for the next artifact:

```text
LANG005.C6.P4 Luna patch-grade implementation plan
```

It is **not** that implementation plan.

Its purpose is to determine, before patch sequencing is written, exactly what P4 must mean, what it must inherit, what P3 gaps must be closed or dispositioned, what first-class language mechanism should express the `Iterator ⇒ Iterable` law, how the law composes with associated projections and recursive proof, how coherence remains sound, how the canonical Universe can begin migration without accidentally doing P5, and what verification surface must exist before P4 may be marked complete.

The analysis is grounded against repository revision:

```text
2da270877ee34693357784d9e2077f63da721802
feat(lang): implement conditional conformance proof evidence
```

and the uploaded/current C6 records:

```text
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md
LANG005.C6.P3-conditional-conformances-recursive-proof-evidence-cycles-and-coherence-plan.md
LANG005.C6.P3-walkthrough.md
LANG005.C6.P3-handoff.md
```

It also incorporates the prior post-implementation P3 closure audit and external design research from:

```text
Rust std / rustc trait solving
Chalk goals, clauses, and coherence
Swift Evolution SE-0143 conditional conformances
Pyrefly recursive protocol performance regressions
Astral ty / Ruff recursive protocol and generic relation implementation
```

The governing principle is:

> P4 must add a first-class, generic, proof-bearing language law. It must not add an Iterator-specific compiler theorem, runtime shortcut, structural-conformance heuristic, or ad-hoc special case.

---

# 1. Executive conclusion

## 1.1 P4 is not “make Iterator work”

P4 has four distinct semantic obligations:

```text
A. Define the contract of Iterable itself.

B. Define the theorem:
       Iterator ⇒ Iterable

C. Make that theorem produce full semantic evidence:
       behavior
       associated bindings
       nested evidence
       cross-module/incremental identity

D. Install the canonical Iterator / Iterable law in the Universe
   without yet migrating production iteration.
```

These are separate requirements.

The central mistake to avoid is collapsing them into one mechanism.

---

## 1.2 The two laws P4 must establish

### Law A — the `Iterable` contract

For any subject `S` satisfying `Iterable`:

```text
<S as Iterable>::Iterator |= Iterator

<<S as Iterable>::Iterator as Iterator>::Item
    ==
<S as Iterable>::Item
```

This is a trait-contract law.

It must hold for:

```text
explicit Iterable conformances
conditional Iterable conformances
derived Iterable conformances
generic assumed `T |= Iterable`
cross-module imported Iterable evidence
```

It is **not** specific to subjects that are themselves iterators.

---

### Law B — the Iterator-derived Iterable theorem

For any subject `I` satisfying `Iterator`:

```text
I |= Iterable
```

with canonical derived evidence:

```text
<I as Iterable>::Item
    ==
<I as Iterator>::Item

<I as Iterable>::Iterator
    ==
I

Iterable.iterator
    returns self
```

This is a conformance implication / derived-conformance law.

It is not produced merely by placing constraints on `Iterable`.

---

## 1.3 Recommended P4 architecture

The recommended semantic architecture is:

```text
ordinary source `impl` spelling
        ↓
guarded generic target rule
        ↓
DerivedConformanceRule
        ↓
index by consequent TraitRef
        ↓
P3 ConformanceProofContext
        ↓
prove antecedent GenericSignature
        ↓
materialize derived behavioral + associated evidence
        ↓
GenericConformanceEvidence::Derived
        ↓
existing semantic-to-core evidence lowering
        ↓
RuntimeTypeEnvironment
+
RuntimeConformanceEnvironment
        ↓
proof-free VM
```

The intended source declaration is equivalent to:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
{
    type Item = <I as Iterator>::Item
    type Iterator = I

    iterator {
        self
    }
}
```

However, **the source spelling and the semantic storage category must not be confused**.

The current C4/P3 `ConformanceTarget` means a nominal declaration or exact enum case. A bare generic subject `I` is neither. Therefore the generic-target declaration above should lower to a **guarded derived-conformance rule**, not be forced into `ConformanceTarget::Declaration`, not fabricate a nominal target, and not mutate exact conformance identity.

This is the principal P4 architectural recommendation.

---

## 1.4 Why not simply add `ConformanceTarget::TypeParameter`?

Because a generic parameter is not an exact target identity.

Current C4/C5 semantics use:

```text
ConformanceTarget
    Declaration(DeclarationId)
    ExactEnumCase(VariantId)
```

for:

```text
ownership
head indexing
coherence
exact source provenance
witness planning
exact evidence construction
```

A naked impl parameter means instead:

```text
for every subject satisfying this rule's antecedent,
derive the consequent conformance
```

That is a logical rule.

It is not an exact target.

Encoding it as another exact target variant would blur:

```text
source rule
exact conformance
applied proof
ownership
coherence
```

and would force C4's nominal head machinery to pretend that an impl-owned type parameter has declaration identity.

P4 should preserve C4.

---

## 1.5 Why not use supertraits?

Current Phalcom specifications explicitly do not define trait inheritance / supertraits as a language mechanism.

Adding:

```text
Iterator : Iterable
```

would require:

```text
trait inheritance syntax
requirement inheritance
associated requirement inheritance/refinement
default interaction
conformance implication
coherence
trait application well-formedness
metadata
reflection
tooling
```

That is a materially broader language feature than P4 needs.

Supertraits remain deferred unless separately ratified.

---

## 1.6 Why not make `Iterator` a compiler-known special trait?

Because the checkpoint explicitly forbids that architecture.

Forbidden forms include:

```text
if trait == Iterator then synthesize Iterable
special VM opcode
runtime "is Iterator" test
selector-based recognition
structural `next` + `iterator` coincidence
hard-coded associated projection rewrite
```

The standard library must exercise a general language mechanism.

---

# 2. Current lifecycle and repository truth

## 2.1 Checkpoint state

The C6 front matter currently says:

```text
checkpoint: LANG005.C6
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED

active_plan: LANG005.C6.P4
latest_completed_plan: LANG005.C6.P3
next_plan: LANG005.C6.P4
```

That is the correct high-level state.

The body contains stale current-action text from before P3 completion and must be corrected during P4 preflight.

---

## 2.2 P4 declared scope

The checkpoint ledger describes P4 as:

```text
Iterator/Iterable generic laws,
canonical Universe migration,
explicit blanket/derived implication decision
```

and P5 as:

```text
production for,
collections,
Map,
Range,
lazy stateful iterators,
cursor/runtime-superclass retirement
```

That division is critical.

P4 establishes the law.

P5 changes production iteration behavior.

---

## 2.3 P3 implementation baseline inherited by P4

The P3 implementation established the following live concepts:

```text
ConformanceProofContext
ProofMemoState
GenericConformanceProof
GenericConformanceEvidence::Exact
GenericConformanceEvidence::Assumed
GenericConformanceEvidence::Derived
DerivedConformanceEvidence
NestedGenericEvidence
RuntimeConformanceSlotRecipe::DerivedConditional
RuntimeConformanceSlot
RuntimeConformanceEnvironment
RuntimeTypeEnvironment
```

The actual P3 cycle model is:

```text
ProofMemoState::Visiting
    ↓
GenericConformanceProof::Blocked(
    BlockReason::RecursiveFixpoint
)
```

P4 must use those live interfaces rather than the inaccurate names in the short P3 walkthrough/handoff.

---

# 3. P3 gaps integrated into P4 requirements

P4 inherits P3's architecture, but it must not inherit P3's unresolved verification ambiguity.

The closure gaps fall into three classes.

---

## 3.1 Class A — mandatory P4 preflight closure

These must be resolved before P4's derived-conformance production work begins.

### P4-P3C-01 — lifecycle record synchronization

Correct:

```text
LANG005.C6-CHECKPOINT.md current action
LANG005.C6-GUIDANCE.md current action
```

so both say P4 is active.

Current stale guidance still says P1.

Current stale checkpoint body still says begin P3.

---

### P4-P3C-02 — P3 live-interface correction

Correct P3 continuation records so P4 inherits:

```text
ProofMemoState::Visiting
ConformanceProofContext.memo
Blocked(RecursiveFixpoint)
DerivedConformanceEvidence
Arc<ConformanceEvidence>
```

instead of:

```text
PendingCycle
proof_cache
DerivedGenericEvidence
ExactSourceTemplate
```

This is required before a Luna implementation agent is given P4.

---

### P4-P3C-03 — owner-generic evidence runtime disposition

The ignored runtime regression:

```text
owner_generic_evidence_executes_after_receiver_specialization
```

must be run and classified.

P4 will extend generic evidence composition again.

An unresolved owner-evidence substrate bug cannot remain ambiguous.

---

### P4-P3C-04 — owner + method evidence composition disposition

The ignored regression:

```text
owner_and_method_local_evidence_execute_together
```

must be run and classified.

Derived-conformance rule witnesses and generic callers must compose with existing owner/method evidence without new per-instance evidence arrays.

---

### P4-P3C-05 — exact enum-case generic evidence disposition

The ignored regression:

```text
exact_enum_case_conformance_executes
```

must be run and classified.

P4 does not need enum cases specifically for Iterator law design, but exact-case identity is a frozen C4/C6 invariant and P3 explicitly promised this closure.

Do not widen an exact case to its enum root.

---

### P4-P3C-06 — trait-application well-formedness closure

This is the most directly P4-blocking P3 gap.

P3 coverage `CV-71` required:

```text
trait application nested generic contract uses recursive context
```

The current live `TraitRef` split is incomplete:

```text
TraitRef::form_identity(...)
    identity / arity / kinds

TraitRef::form(...)
    legacy validation
```

and current constraint validation:

```text
Subtype / Equivalent
    substitute ordinary trait parameters

Conforms
    skipped for generic proof layer

SelfType
    ConstraintNotCanonical
```

P4's `Iterable` contract requires subject-aware trait constraints involving:

```text
Self
Self::Iterator
Iterator::Item
```

Therefore P4 must close trait-application contract proving under:

```text
subject + TraitRef + active GenericEvidenceEnvironment
```

without moving proof back into `TraitRef` identity construction.

---

### P4-P3C-07 — memo/resource discriminator

P3 implemented memoization but did not preserve the requested final unique-goal evidence.

P4 adds another recursive proof edge:

```text
Iterator evidence
    → derived Iterable
        → Iterable trait contract
            → Iterator evidence
```

The proof context must demonstrate bounded work.

Add or identify a discriminator where:

```text
repeated subgoal paths
    >
unique canonical goals
```

and prove completed memo reuse.

---

### P4-P3C-08 — runtime environment reuse discriminator

P3's conditional runtime reuse test proves repeated behavior but does not measure conformance-environment registry growth.

P4 must establish:

```text
repeated identical derived Iterator→Iterable evidence
does not allocate permanent environments proportional to call count
```

Use registry cardinality or equivalent structural evidence.

---

### P4-P3C-09 — stronger rooting/lifetime discriminator

P3's GC test evaluates the evidence-dependent operation before forcing GC.

P4 should either cite a stronger existing lifetime test or add:

```text
construct/capture derived evidence
force GC
invoke derived trait behavior after GC
verify nested evidence still executes
```

A post-GC already-computed result is not the strongest discriminator.

---

## 3.2 Class B — mandatory disposition, repair only if P4 depends on it

### P4-P3C-10 — callable-reference Outcome A/B

Record:

```text
Outcome A:
    existing callable-reference object preserves/reconstructs
    type + conformance environments
    → add regression

Outcome B:
    correct support requires new persistent evidence-capture architecture
    → explicit accepted deferral
```

P4 should add a derived `iterator` callable-reference regression only if Outcome A uses existing machinery.

If Outcome B requires a new heap lifetime contract, do not absorb that architecture into P4 without consultation.

---

### P4-P3C-11 — associated declaration bound authority

Correct the historical record confusion between:

```text
AssociatedTypeDeclaration
```

and:

```text
TraitRequirementSyntax
```

The current associated declaration is still structurally plain.

P4 should not activate:

```phalcom
type Iterator |= Iterator
```

merely to express the Iterable law.

The Iterable law also needs cross-associated equality, so associated-declaration bounds are insufficient by themselves anyway.

Prefer the trait-level canonical generic contract.

---

### P4-P3C-12 — P3 CV closure ledger

Build or amend the P3 closure record so all required P2R/CV items are mapped truthfully.

This is record/certification work, not permission to redesign P3.

---

## 3.3 Class C — preserve as inherited baselines unless P4 reproduces them

Examples include the legacy cursor/Range/Option call-entry baselines.

P4 is not production iterator migration.

Do not repair legacy Range or cursor implementation unless:

```text
the new P4 generic law path reproduces the same root defect
```

If it does, reclassify.

Otherwise keep it for P5 / runtime maintenance.

---

# 4. Normative Iterator / Iterable model

## 4.1 Canonical `Iterator`

P4's canonical semantic trait remains:

```phalcom
trait Iterator {
    type Item

    next -> Option<Self::Item>

    iterator {
        self
    }
}
```

Properties:

```text
Item is associated
next is getter-shaped
next returns Option<Item>
None means clean exhaustion
errors propagate normally
iterator returns self
iterator does not restart
iterator does not copy
iterator does not reset
```

---

## 4.2 Canonical `Iterable`

The conceptual surface remains:

```phalcom
trait Iterable {
    type Item
    type Iterator

    iterator -> Self::Iterator
}
```

but P4 must add the **trait contract**:

```text
Self::Iterator |= Iterator

<Self::Iterator as Iterator>::Item
    ==
Self::Item
```

---

## 4.3 Preferred source-level trait contract form

The preferred generic-contract spelling is:

```phalcom
trait Iterable
    where
        Self::Iterator |= Iterator,
        <Self::Iterator as Iterator>::Item == Self::Item
{
    type Item
    type Iterator

    iterator -> Self::Iterator
}
```

This is preferable to new associated-declaration-bound syntax because:

1. it uses `GenericSignature`, the existing generic proposition authority;
2. it expresses both conformance and equality;
3. it keeps `|=`, `==`, and associated projection distinct;
4. it naturally becomes implied evidence from `T |= Iterable`;
5. it maps directly to P1/P2/P3 proof machinery.

---

## 4.4 VERIFY-FIRST formation issue

The AST already allows:

```text
TraitDef.where_clause
```

but P4 must verify that the trait where-clause can resolve:

```text
Self
Self::Iterator
qualified projection of Self::Iterator
```

against the trait's own associated declarations.

The current trait surface builder pre-scans associated declarations before behavioral members, but trait generic-header formation may happen before that surface exists.

Therefore P4 may require **staged trait contract formation**.

---

# 5. Staged trait contract formation requirement

## 5.1 Required semantic phases

A trait such as `Iterable` should be formed conceptually as:

```text
1. declaration identity
2. generic parameter identities + kinds
3. associated declaration headers
       Item
       Iterator
4. trait where-clause GenericSignature
       now allowed to refer to:
           Self
           trait generic parameters
           associated declarations
5. behavioral TraitSurface
6. finalized trait contract publication
```

This does not require a second trait system.

It is staging around existing products.

---

## 5.2 Associated declaration identity must remain C5 identity

Do not change:

```text
AssociatedTypeRequirementId
    = trait declaration identity + stable index
```

Do not resolve `Self::Iterator` by string at proof time.

If an early associated-header view is needed, it must publish the same canonical requirement identity later used by `TraitSurface.associated_types`.

---

## 5.3 `Self` must be part of contract instantiation, not TraitRef identity

`TraitRef` remains:

```text
trait declaration
+
ordinary trait arguments
```

The complete contract application is:

```text
subject
+
TraitRef
```

P4 must not add `Self` to `TraitRef`.

Instead, proving or elaborating a trait contract receives:

```text
subject = S
trait_ref = Iterable
```

and substitutes:

```text
Self → S
```

while preserving the exact associated requirement identity.

---

# 6. Trait contract entailment

This is a foundational P4 requirement separate from derived conformance.

## 6.1 Exact evidence case

If exact evidence proves:

```text
S |= Iterable
```

then that evidence must satisfy the trait's declared GenericSignature:

```text
S::Iterator |= Iterator
S::Iterator::Item == S::Item
```

A source conformance that cannot satisfy the trait contract is invalid/incomplete.

---

## 6.2 Generic assumed case

If a generic body has:

```phalcom
where T |= Iterable
```

then the body may rely on the propositions declared by `Iterable`.

Conceptually:

```text
FromEnv(T |= Iterable)
    ⇒
FromEnv(T::Iterator |= Iterator)

FromEnv(T |= Iterable)
    ⇒
FromEnv(
    <T::Iterator as Iterator>::Item
    ==
    T::Item
)
```

P4 should implement this as canonical evidence/constraint entailment, not as repeated syntax expansion in every caller.

---

## 6.3 External reference: Chalk implied bounds

Chalk models a trait's own where-clauses as implications from a trait fact in the environment.

That is the right conceptual reference for P4:

```text
trait fact in environment
    ↓
trait-declared constraints become available
```

Phalcom should retain its own evidence model and proof terminal states, but the logical shape is sound.

---

## 6.4 Do not use coinductive success

Chalk may use coinductive well-formedness in recursive trait relationships.

Phalcom P3 explicitly chose:

```text
cycle != proof
```

P4 must preserve that decision.

A trait-contract recursion that re-enters a `Visiting` conformance goal remains:

```text
Blocked(RecursiveFixpoint)
```

unless another already-proven base fact closes the cycle.

---

# 7. The `Iterator ⇒ Iterable` mechanism

## 7.1 Required source theorem

P4 needs a source-level, first-class declaration equivalent to:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
{
    type Item = <I as Iterator>::Item
    type Iterator = I

    iterator {
        self
    }
}
```

This declaration is valuable because it states all of the law in ordinary language concepts:

```text
generic subject
conformance antecedent
associated projection
associated binding
behavioral witness
```

No Iterator-specific semantic primitive is required.

---

# 8. Why this cannot be an ordinary current `ConformanceTarget`

Current `ConformanceTarget` is:

```text
Declaration(DeclarationId)
ExactEnumCase(VariantId)
```

and its meaning includes:

```text
nominal ownership
exact head identity
global coherence
source witness plan identity
```

A bare impl parameter `I` instead means:

```text
all subjects that satisfy this rule's GenericSignature
```

Therefore P4 should not widen `ConformanceTarget` into a union of:

```text
exact nominal target
logical quantified rule
```

unless a later repository proof shows that such a unification can preserve every C4 invariant cleanly.

The safer architecture is a distinct **derived-rule product** consumed by the same proof authority.

---

# 9. Recommended semantic product: guarded derived conformance rule

Names are mechanically flexible.

Conceptually:

```rust
struct DerivedConformanceRule {
    source_impl: ImplId,

    parameters: Box<[TypeParameterId]>,

    subject_template: TypeTerm,

    consequent_trait_ref: TraitRef,

    generic_signature: GenericSignature,

    associated_binding_templates:
        BTreeMap<
            AssociatedTypeRequirementId,
            TypeTermOrTypeIdTemplate
        >,

    requirement_selections:
        BTreeMap<
            TraitRequirementId,
            DerivedRequirementSelectionTemplate
        >,

    fingerprint: ProductFingerprint,

    source: SemanticSourceSpan,
}
```

For P4's initial guarded form:

```text
subject_template
    must be one impl-owned type parameter

generic_signature
    must provide at least one meaningful guard
    for that subject

source module
    must own the consequent trait
```

---

## 9.1 Why “guarded”

P4 should not accidentally authorize:

```phalcom
impl<T> Printable for T {}
```

as an unconditional universal conformance.

The initial capability is intended to express evidence implications such as:

```text
T |= Iterator
    ⇒
T |= Iterable
```

A naked generic-target rule should therefore require an explicit nontrivial generic contract.

---

## 9.2 Why “derived conformance”

The traits specification already leaves room for:

```text
explicit conformance
unless another specification defines derived/synthesized conformance
```

P4 can define exactly that missing category.

Derived conformance remains nominal in trait identity and proof.

It is not structural conformance.

---

# 10. Derived-rule proof integration

## 10.1 Candidate sources for one goal

For a goal:

```text
S |= TraitRef
```

P4 proof conceptually has:

```text
A. lexical assumed evidence

B. exact/conditional nominal source conformance
   through existing ConformanceIndex

C. derived conformance rules
   whose consequent can match TraitRef
```

Candidate discovery must not depend on source order.

---

## 10.2 Rule index

Do not scan every derived rule for every goal.

Maintain an index conceptually keyed by:

```text
consequent trait declaration
+
canonical ordinary consequent trait arguments / pattern class
```

For non-generic `Iterable`:

```text
DerivedRuleIndex[Iterable]
    → Iterator⇒Iterable rule
```

Proof cost should be proportional to relevant rule candidates, not all rules in the workspace.

---

## 10.3 Rule application

For goal:

```text
MyIterator |= Iterable
```

the rule application is:

```text
match subject template:
    I = MyIterator

instantiate rule GenericSignature:
    MyIterator |= Iterator

prove nested antecedent
    in SAME ConformanceProofContext

materialize:
    Iterable::Item
        = <MyIterator as Iterator>::Item

    Iterable::Iterator
        = MyIterator

    Iterable.iterator
        = source rule witness returning self

validate Iterable trait contract

construct derived evidence
```

---

## 10.4 No reverse inference

The derived rule may be used only after the goal subject is sufficiently known.

Do not infer an unknown subject from:

```text
Iterable::Item
Iterable::Iterator
```

or search the world for an Iterator that could make a desired result type fit.

P2's one-way inference rule remains intact.

---

# 11. Coherence requirements for derived rules

This is the hardest P4 design area.

## 11.1 P3 explicit-head coherence remains unchanged

P3's rule remains:

```text
two explicit/conditional nominal conformance heads
do not become legal merely because their where clauses differ
```

P4 must not weaken that rule.

---

## 11.2 Derived rules are a separate coherence category

A derived rule is not an ordinary C4 nominal head.

Therefore its overlap analysis should not be implemented by pretending the subject template `I` is a nominal target.

---

## 11.3 No candidate-ranking semantics

P4 must not define:

```text
explicit beats derived
more specific beats blanket
first declared wins
local beats imported
condition order decides
```

unless explicitly ratified.

The preferred rule is:

> There must be at most one successful conformance proof for a concrete `subject + exact TraitRef`, regardless of whether it originates from an explicit nominal conformance or a derived rule.

---

## 11.4 Explicit-vs-derived conflict

Suppose:

```text
X |= Iterator
```

and source also declares:

```phalcom
impl Iterable for X {
    ...
}
```

The Iterator-derived law already creates:

```text
X |= Iterable
```

so the explicit conformance creates a second proof.

P4 should diagnose this as a coherence conflict unless a future specialization/override mechanism is separately designed.

Do not silently prefer one.

---

## 11.5 Why direct-wins is not safe for Iterator

If explicit `Iterable` simply overrides the derived one, an Iterator could define:

```text
Iterable::Iterator = SomeOtherIterator
```

or a different `Iterable::Item`.

That would violate the canonical theorem:

```text
Iterator subject's Iterable::Iterator = self
Iterator subject's Iterable::Item = Iterator::Item
```

Therefore the derived law is not merely a convenient default implementation.

It is a semantic conformance theorem.

---

## 11.6 Rule-vs-rule conflict

P4 should begin conservatively.

For the initial feature, it is reasonable to reject multiple guarded derived rules whose consequent trait heads can overlap.

P4 does not need:

```text
derived rule specialization
disjoint-condition reasoning
most-specific ranking
negative constraints
```

---

## 11.7 Ownership / orphan-like rule

A naked generic target has no target declaration module.

Therefore P4 should authorize a guarded derived rule only when:

```text
source module owns the consequent trait declaration
```

For:

```text
Iterator ⇒ Iterable
```

the Universe owns `Iterable`, so the law is authorized there.

This prevents arbitrary third-party modules from globally asserting:

```text
for every T satisfying ExternalA,
T satisfies ExternalB
```

when the module owns neither trait.

---

## 11.8 External research: Rust and Chalk

Rust's blanket:

```rust
impl<I: Iterator> IntoIterator for I
```

is the closest standard-library analogue.

Rust can support it because its trait coherence/orphan machinery is designed for blanket impls.

Chalk models an impl as a program clause:

```text
Implemented(consequent)
    :-
where-clause goals
```

That is conceptually the right model for Phalcom's derived rule.

However, Phalcom should not import Rust's specialization, negative impl, auto-trait, or full compatible-world machinery into P4.

---

## 11.9 External research: Swift warning

Swift Evolution SE-0143 is especially relevant.

Swift permits conditional conformances of nominal generic types but deliberately does not generalize the feature to protocol-to-protocol conformance because that sharply increases overlap ambiguity.

It also rejects even apparently disjoint multiple conditional conformances in that proposal.

P4 should take the same lesson:

```text
derived trait implication is powerful
therefore the first design should be intentionally restrictive
and globally coherent
```

Do not implement a broad open-ended “protocol extension conforms to protocol” facility by accident.

---

# 12. Derived evidence architecture

P3's current `DerivedConformanceEvidence` was designed for conditional nominal impls.

Conceptually it contains:

```text
exact_source: Arc<ConformanceEvidence>
impl_bindings
nested evidence
```

A generic-target derived rule has no exact nominal consequence-source conformance to wrap.

P4 must extend this carefully.

---

## 12.1 Required distinction

Do not fabricate:

```text
ConformanceTarget::Declaration(fake)
exact ConformanceEvidence for impl parameter I
exact nominal target owner
```

Instead use a derived origin.

Conceptually:

```rust
enum DerivedConformanceOrigin {
    ConditionalSource {
        exact_source: Arc<ConformanceEvidence>,
    },

    GuardedRule {
        rule: Arc<DerivedConformanceRule>,
        materialized_subject: TypeId,
        materialized_trait_ref: TraitRef,
    },
}
```

and:

```rust
struct DerivedConformanceEvidence {
    origin: DerivedConformanceOrigin,

    impl_bindings: TypeSubstitution,

    requirement_views: ...,

    requirements: ...,

    associated_types: ...,

    nested: Box<[NestedGenericEvidence]>,
}
```

Exact spelling is flexible.

Information content is not.

---

## 12.2 Preserve P3 exact semantics

Existing conditional nominal evidence must remain semantically equivalent.

Do not rewrite P3 exact conformance identity merely to make the new rule convenient.

A wrapper/generalized origin is preferable to mutating C4 `ConformanceEvidence`.

---

## 12.3 Evidence must expose associated bindings generically

P4 associated projection cannot assume:

```text
every GenericConformanceEvidence::Derived
has an exact_source whose associated_types map is the answer
```

A guarded rule derives its own associated bindings.

Therefore semantic consumers need an evidence-level operation equivalent to:

```text
associated_binding(
    GenericConformanceEvidence,
    AssociatedTypeRequirementId
)
```

that works for:

```text
Exact
Assumed / symbolic
Derived conditional
Derived guarded rule
```

without runtime lookup.

---

## 12.4 Behavioral requirement selections

Derived `Iterable` evidence must select:

```text
Iterable.iterator
```

to the rule's concrete source witness.

The witness remains semantically owned by the rule/conformance source.

It does not become an inherent method of every Iterator type.

---

# 13. Associated projection behavior under derived evidence

For exact `I |= Iterator`, P4 must prove:

```text
normalize(<I as Iterable>::Item)
    =
normalize(<I as Iterator>::Item)

normalize(<I as Iterable>::Iterator)
    =
I
```

For generic assumed:

```text
T |= Iterator
```

P4 should be able to form symbolic derived Iterable evidence and normalize:

```text
<T as Iterable>::Item
    →
<T as Iterator>::Item

<T as Iterable>::Iterator
    →
T
```

under the same generic evidence environment.

This must reuse:

```text
TypeData::AssociatedProjection
types::normalize_type
ConformanceProofContext
```

No second projection normalizer.

---

# 14. Trait-contract validation under source conformances

The `Iterable` trait contract must reject an invalid explicit conformance.

Example:

```phalcom
class BadIterator {}

class BadIterableIterator {}

impl Iterable for BadIterator {
    type Item = Int
    type Iterator = BadIterableIterator

    iterator -> BadIterableIterator { ... }
}
```

If:

```text
BadIterableIterator !|= Iterator
```

the conformance is invalid.

Likewise:

```text
Iterable::Item = Int
Iterable::Iterator = SomeIter
Iterator(SomeIter)::Item = String
```

must be rejected because:

```text
String != Int
```

This validation belongs to semantic conformance completeness/proof.

Not runtime.

---

# 15. Assumed `Iterable` evidence must entail its contract

A generic method:

```phalcom
class Use {
    @class
    consume<T>(_ value: T)
        where T |= Iterable
    {
        ...
    }
}
```

must be allowed to reason that:

```text
T::Iterator |= Iterator
```

and:

```text
<T::Iterator as Iterator>::Item == T::Item
```

without asking for redundant caller constraints.

This is a major P4 acceptance requirement.

---

# 16. Interaction between the two P4 laws

The Iterator-derived evidence itself must satisfy the Iterable contract.

For `I |= Iterator`:

```text
derived:
    I |= Iterable

derived binding:
    Iterable::Iterator = I

Iterable contract:
    I |= Iterator

derived binding:
    Iterable::Item = Iterator::Item

Iterable contract equality:
    Iterator::Item == Iterable::Item
```

This graph intentionally revisits the original `I |= Iterator` proof.

P3 memoization must make that cheap and terminating.

This is an ideal P4 stress case for:

```text
memo reuse
projection normalization
derived evidence
trait-contract entailment
```

---

# 17. Runtime boundary

P4 must preserve the P3 runtime anti-authority boundary.

## 17.1 Semantic layer decides

```text
derived rule applicability
antecedent proof
associated binding values
requirement witness/default selection
nested evidence sources
trait-contract validity
coherence
```

---

## 17.2 Core/runtime may do

```text
materialize type bindings
materialize nested conformance slots
intern RuntimeTypeEnvironment
intern RuntimeConformanceEnvironment
attach environment ids to frame
invoke preselected trait requirement
```

---

## 17.3 VM may not do

```text
if receiver conforms Iterator then synthesize Iterable
search derived rules
search impl declarations
normalize Iterable::Item
prove trait where clauses
resolve coherence
inspect GenericConstraint
```

---

# 18. Runtime recipe reuse

P4 should first attempt to reuse:

```text
RuntimeConformanceSlotRecipe::DerivedConditional
```

because a guarded derived rule still lowers to:

```text
selected callable
type bindings
nested evidence slot recipes
```

If the current variant name becomes semantically misleading, a mechanical rename/generalization may be considered.

A new proof opcode is not required.

A new runtime rule solver is forbidden.

---

# 19. Runtime environment requirements

A derived `Iterable.iterator` activation may require:

```text
RuntimeTypeEnvironment
    I -> exact subject

RuntimeConformanceEnvironment
    nested Iterator evidence
```

even though the method body is simply:

```phalcom
self
```

The nested environment still matters because:

```text
associated projections
future default bodies
other rule witnesses
generic composition
```

must use the same general architecture.

Do not special-case the trivial body.

---

# 20. Canonical Universe migration

P4's checkpoint scope includes canonical Universe migration.

The current production Universe is not yet the target architecture.

---

## 20.1 Current production state

The current Universe still has:

```phalcom
@native
class Iterable<T> is Object {
    iterate(_ cursor) { ... }
    iteratorValue(_ cursor) { ... }
    ...
}

class Iterator is Iterable {
    iter { self }
}

class SourceIterator is Iterator { ... }
class MapIterator is Iterator { ... }
class FilterIterator is Iterator { ... }
...
```

This is the legacy cursor hierarchy.

Collection native superclasses are also wired to the runtime `Iterable` class.

---

## 20.2 Name collision is real

The target language needs public trait identities:

```text
Iterable
Iterator
```

but those names are currently occupied by legacy classes.

P4 cannot simply add same-named traits beside them.

---

## 20.3 P4 must stage, not finish, migration

P4 owns the canonical law and canonical Universe trait identities.

P5 owns:

```text
List / Set / Map / Tuple / Bytes / Range conformances
stateful collection iterators
for checker
for lowering
Map entry migration
RangeIterator
lazy stateful adapters
legacy superclass retirement
cursor API retirement
```

Therefore P4 needs a compatibility staging strategy.

---

# 21. Universe staging options

## Option U-A — internalize the legacy hierarchy, expose canonical traits

Conceptually:

```text
legacy public class Iterable<T>
    →
internal transition base _LegacyIterable<T>

legacy public class Iterator
    →
internal transition base _LegacyIterator

public names:
    Iterable  -> trait
    Iterator  -> trait
```

Current production collections/adapters may temporarily continue inheriting the internal legacy bases until P5.

Advantages:

```text
P4 can establish final public trait identity
production cursor behavior stays alive
P5 can migrate one consumer family at a time
```

Risks:

```text
native bootstrap key changes
user-visible class references may exist
runtime class identity assumptions may exist
metadata/native registry may expect "Iterable"
legacy tests may inspect class names
```

This option requires a focused repository audit before adoption.

---

## Option U-B — canonical traits in a non-public transition namespace

P4 can install the final law and traits in an internal Universe module/namespace while leaving public legacy names unchanged.

P5 performs the public identity flip.

Advantages:

```text
smaller P4 runtime churn
```

Disadvantages:

```text
P4 would not fully complete "canonical Universe migration"
tests may exercise transition identities rather than final language identity
P5 starts with a public identity migration rather than only behavior migration
```

Use only if the legacy public-name collision makes U-A materially unsafe.

---

## Option U-C — simultaneous P4 production migration

Replace the public legacy classes and immediately migrate all production consumers.

Reject for P4.

That is P5 scope.

---

## 21.1 P4 requirements for Universe staging

Regardless of staging option:

```text
canonical trait contracts must be compiled from real Universe source
Iterator=>Iterable law must be real source semantics
legacy for-loop behavior must remain working
legacy collection iteration must remain working
no cursor selectors are retired in P4
no production collection conformance migration is required in P4
```

---

# 22. P4 versus P5 boundary

## P4 owns

```text
P3 closure preflight
trait-contract subject-aware proof
trait implied constraints
Iterable generic contract
guarded derived-conformance language mechanism
guarded rule coherence/ownership
derived associated evidence
derived behavioral evidence
cross-module rule publication
incremental invalidation
metadata/fingerprints
focused source-index/tooling
canonical Iterator/Iterable Universe declarations
Iterator⇒Iterable Universe rule
transition compatibility required to install those declarations
resource evidence
walkthrough/handoff
```

---

## P4 does not own

```text
production List Iterable conformance
production Set Iterable conformance
production Map Iterable conformance
production Tuple Iterable conformance
production Bytes Iterable conformance
production Range Iterable conformance

production collection iterator objects
production lazy iterator rewrite
production RangeIterator

for-loop evidence-driven checker
for-loop iterator+next lowering

Map direct iteration entry tuple switch

cursor method deletion
runtime Iterable superclass deletion
legacy adapter deletion

full C6 certification
```

---

# 23. Source language scope of guarded derived conformance

P4 should keep the first version intentionally narrow.

## 23.1 Required accepted shape

Equivalent to:

```phalcom
impl<T> Consequent for T
    where T |= Antecedent
{
    ...
}
```

where:

```text
T is an impl-owned Type parameter
consequent trait is owned by source module
rule has a nonempty meaningful GenericSignature
rule body is a complete ordinary conformance body
```

---

## 23.2 Recommended initial generality

Allow canonical conditions already supported by P3:

```text
Conforms
Equivalent
Subtype
associated specialization
```

but do not add:

```text
negative constraints
specialization ranking
disjunction
existential rule heads
higher-kinded subject rules
structural conformance
```

---

## 23.3 Bare target only where intentional

Ordinary nominal generic conformance remains:

```phalcom
impl<T> Printable for Box<T>
```

The new category is only when the target itself is an impl parameter / accepted rule subject template.

Do not reclassify ordinary C4/P3 conformances as rules.

---

# 24. Source authorization

For ordinary conformance:

```text
module owns trait
or
module owns nominal target
```

For guarded bare-target rule:

```text
module must own consequent trait
```

There is no target declaration to own.

This should be explicit in the trait/conformance specification.

---

# 25. Stable rule identity

A rule's semantic identity should be independent of:

```text
generic parameter spelling
where constraint source order
associated binding source order
formatting
source ranges
```

and include canonical meaning equivalent to:

```text
source module / source impl identity
subject template shape
consequent exact TraitRef template
canonical GenericSignature
associated binding templates
behavioral requirement selections
```

Source order is not proof order.

---

# 26. Cross-module rule publication

A downstream module must be able to prove:

```text
DownstreamIter |= Iterable
```

from:

```text
imported Iterator trait
imported Iterable trait
imported Iterator⇒Iterable rule
local/imported DownstreamIter |= Iterator evidence
```

without reparsing the provider module's source.

Therefore a derived rule is a durable semantic product.

---

# 27. Metadata / interface representation

P4 must VERIFY-FIRST whether the current module semantic publication already transports enough conformance source information for a new rule.

If not, add stable representation equivalent to:

```text
StableDerivedConformanceRule
    source/provenance
    canonical generic parameters
    stable subject type template
    stable consequent trait application
    stable GenericSignature
    stable associated binding templates
    stable witness/default references
```

Never persist:

```text
TypeId
TraitRef semantic object
InferVarId
ConformanceProofContext
RuntimeConformanceEnvironmentId
memo state
```

---

# 28. Incremental requirements

P4 must establish precise dependencies.

## 28.1 Rule condition change

Changing:

```text
T |= Iterator
```

to another real condition must invalidate:

```text
derived conformance proofs
derived projection consumers
derived dispatch consumers
runtime environment recipes
public interface/fingerprint consumers
```

---

## 28.2 Canonical no-op change

Reordering equivalent conditions must preserve semantic rule fingerprint.

---

## 28.3 Antecedent evidence change

Adding/removing/changing:

```text
X |= Iterator
```

must recompute consumers relying on derived:

```text
X |= Iterable
```

---

## 28.4 Trait-contract change

Changing the `Iterable` trait GenericSignature must invalidate every evidence product claiming `Iterable`, including derived evidence.

---

## 28.5 Unrelated trait stability

Changing an unrelated conformance must not globally invalidate all derived rules.

---

# 29. Source index and LSP

P4 should provide canonical navigation for new source constructs but not build an editor-owned solver.

At minimum:

```text
derived rule consequent trait occurrence
antecedent trait occurrence
associated projection occurrences
associated binding occurrences
trait-level where-clause occurrences
```

must point to canonical semantic targets.

If the LSP renders a derived conformance relationship, it should identify it as derived/rule-based rather than pretend it is an inherent member or nominal source conformance.

Broader reflection UX may remain P6 if no public API currently requires it.

---

# 30. Diagnostics requirements

P4 diagnostics must distinguish:

```text
no Iterator evidence
derived-rule coherence conflict
invalid rule ownership
invalid unguarded blanket rule
Iterable trait-contract Iterator failure
Iterable Item equality failure
derived associated binding failure
recursive-fixpoint block
unknown / underconstrained
Dynamic boundary
budget exhausted
cancelled
internal failure
```

Do not collapse all failures into:

```text
does not conform
```

when nested provenance is available.

---

# 31. Performance requirements

## 31.1 No global derived-rule scan

Index by consequent trait.

---

## 31.2 Preserve P3 memoization

The proof graph:

```text
I |= Iterable
  → rule condition I |= Iterator
  → validate Iterable contract
      → I |= Iterator
```

must hit memoized Iterator evidence on the second request.

---

## 31.3 No eager transitive explosion

Trait-contract implied constraints should be elaborated:

```text
lazily
or
once per lexical evidence environment
```

not re-expanded for every member send.

---

## 31.4 Rule proof caching remains proof-context-local

Do not create a global cache keyed only by:

```text
TypeId + TraitRef
```

because lexical assumed evidence affects proof.

---

## 31.5 Runtime environments remain interned

Repeated identical derived calls must reuse equal:

```text
RuntimeTypeEnvironment
RuntimeConformanceEnvironment
```

registry entries.

---

## 31.6 No runtime conformance checks on the hot path

Once call specialization exists:

```text
InvokeTraitRequirement
```

or equivalent execution must remain direct.

---

# 32. External implementation research and lessons

## 32.1 Rust standard library — direct semantic analogue

Rust defines the equivalent of:

```rust
impl<I: Iterator> IntoIterator for I
```

with:

```text
Item = I::Item
IntoIter = I
into_iter returns self
```

This is almost exactly Phalcom's target theorem.

Takeaway:

> The law is naturally represented as a generic conformance/impl rule, not an iterator-specific loop rule.

Do not copy Rust syntax.

---

## 32.2 rustc trait solving — candidate + nested obligations

rustc models a trait query as an obligation/goal.

Important candidate sources include:

```text
impl candidates
parameter-environment assumptions
```

and selecting an impl may produce nested obligations from its where-clause.

Takeaway for Phalcom:

```text
P3 ConformanceProofContext
+
DerivedConformanceRule candidate
+
nested GenericSignature proof
```

is the correct architecture.

P4 does not need rustc's full inference/specialization machinery.

---

## 32.3 Chalk — program clause model

Chalk lowers:

```rust
impl<P...> Trait<...> for Type
where WC
```

to a logical form equivalent to:

```text
Implemented(TraitRef)
    :-
WC
```

It also treats associated equality and trait where-clauses as logical goals/implications.

Takeaway:

> A Phalcom guarded derived conformance is best understood as a canonical proof rule / program clause that P3's prover consumes.

This supports keeping it distinct from exact nominal target identity.

---

## 32.4 Chalk coherence — do not ignore blanket overlap

Chalk coherence research emphasizes:

```text
orphan/locality rules
overlap
compatible future worlds
```

P4 does not need to clone Chalk's compatible-world solver, but it must acknowledge that blanket rules affect downstream coherence.

Therefore:

```text
consequent-trait ownership
no rule specialization
no ambiguous overlap
cross-module conflict checks
```

are mandatory.

---

## 32.5 Swift SE-0143 — strong caution on protocol-to-protocol conformance

Swift's conditional-conformance proposal:

```text
supports conditional conformance for nominal generic types
rejects multiple overlapping conditional conformances
does not include protocol extensions that add protocol conformance
```

because protocol-to-protocol implication substantially worsens overlap ambiguity.

Takeaway:

> P4's first derived-rule mechanism should be narrower than a general open-ended trait-implication system.

The Iterator law is a justified first use, not permission to invent specialization.

---

## 32.6 Pyrefly — recursive proof performance

Pyrefly carries a regression showing recursive protocol comparison can expand exponentially when the same recursive goals are recomputed.

Its memoized subset cache collapses repeated `(subject, protocol)` work.

It also has tests warning that provisional/coinductive successes must not leak after an outer branch fails.

Takeaway for Phalcom:

```text
memoization is structural correctness/performance infrastructure
not optional optimization
```

Phalcom's cycle semantics differ:

```text
cycle is blocked, not coinductive success
```

so P4 must not import Pyrefly's coinductive assumptions.

---

## 32.7 Astral `ty` / Ruff — recursion guards and relation specialization

`ty` separates nominal and structural instance handling and uses explicit recursion-detection/materialization machinery for recursive protocol relationships.

Takeaway:

```text
keep recursive state explicit
keep relation identities canonical
avoid uncontrolled recursive materialization
```

Its protocol system is structural and therefore not an authority for Phalcom's nominal conformance semantics.

---

# 33. Required specification amendments

P4 cannot be implemented truthfully while the normative documents still say every conformance target must be nominal/exact only with no derived-rule exception.

At minimum P4 should amend the authoritative equivalents of:

```text
docs/specs/objects/traits.md
docs/specs/generic-constraints.md
docs/specs/iterator.md
```

---

## 33.1 Traits specification

Define:

```text
guarded derived conformance
generic target rule
ownership
evidence identity
coherence
interaction with explicit conformance
```

while preserving ordinary explicit target restrictions.

---

## 33.2 Generic constraints specification

Clarify:

```text
derived rule GenericSignature is ordinary canonical GenericSignature
rule proof uses recursive conformance proof
no reverse open-world inference
trait contract where-clauses are entailed by assumed conformance
Self is substituted from conformance subject during contract proof
```

---

## 33.3 Iterator specification

Replace “evidence syntax specified separately” ambiguity with an explicit reference to the chosen generic mechanism.

The semantic law should remain:

```text
Iterator ⇒ Iterable
```

not the implementation type name.

---

# 34. Proposed P4 foundational requirement map

The patch-grade plan should preserve requirement IDs or an equivalent explicit ledger.

## Foundation / P3 closure

| ID | Requirement |
|---|---|
| `P4-F01` | record actual P4 start revision |
| `P4-F02` | synchronize checkpoint/guidance current action |
| `P4-F03` | correct P3 handoff live interface names |
| `P4-F04` | disposition owner-generic runtime regression |
| `P4-F05` | disposition owner+method evidence regression |
| `P4-F06` | disposition exact enum-case generic regression |
| `P4-F07` | callable-reference Outcome A/B recorded |
| `P4-F08` | P3 CV/P2R closure ledger recorded |
| `P4-F09` | memo unique-goal discriminator established |
| `P4-F10` | runtime environment cardinality discriminator established |
| `P4-F11` | post-GC evidence invocation discriminator established or strong existing proof cited |
| `P4-F12` | associated declaration-bound record corrected |

---

## Trait contract

| ID | Requirement |
|---|---|
| `P4-TC01` | `Iterable` owns one canonical GenericSignature contract |
| `P4-TC02` | trait contract can reference `Self` |
| `P4-TC03` | trait contract can reference own associated projections |
| `P4-TC04` | associated requirement identity remains C5 identity |
| `P4-TC05` | trait contract instantiation substitutes subject for `Self` |
| `P4-TC06` | exact Iterable evidence validates `Iterator` associated conformance |
| `P4-TC07` | exact Iterable evidence validates Item equality |
| `P4-TC08` | assumed `T |= Iterable` entails contract constraints |
| `P4-TC09` | entailment reuses GenericEvidenceEnvironment / P3 proof |
| `P4-TC10` | trait contract cycles remain non-proof |
| `P4-TC11` | `TraitRef` identity remains separate from contract proof |
| `P4-TC12` | no new associated-declaration-bound syntax required for P4 |

---

## Guarded derived conformance

| ID | Requirement |
|---|---|
| `P4-DC01` | ordinary impl syntax can express guarded generic-target rule |
| `P4-DC02` | generic-target rule is semantically distinct from ConformanceTarget |
| `P4-DC03` | rule source has stable ImplId/provenance |
| `P4-DC04` | rule consequent has canonical TraitRef template |
| `P4-DC05` | rule conditions are ordinary GenericSignature |
| `P4-DC06` | naked generic rule requires nontrivial guard |
| `P4-DC07` | source module owns consequent trait |
| `P4-DC08` | rule index is keyed by consequent trait |
| `P4-DC09` | rule application uses P3 proof context |
| `P4-DC10` | rule application preserves rich proof terminals |
| `P4-DC11` | no reverse inference |
| `P4-DC12` | no structural coincidence |
| `P4-DC13` | rule can supply behavioral witnesses |
| `P4-DC14` | rule can supply associated binding templates |
| `P4-DC15` | rule proof can consume exact evidence |
| `P4-DC16` | rule proof can consume assumed evidence |
| `P4-DC17` | rule proof can consume derived conditional evidence |
| `P4-DC18` | rule proof can recursively consume derived-rule evidence |
| `P4-DC19` | derived-rule cycles terminate non-success |
| `P4-DC20` | no Iterator name appears in semantic solver special cases |

---

## Derived evidence

| ID | Requirement |
|---|---|
| `P4-EV01` | existing P3 conditional Derived evidence meaning preserved |
| `P4-EV02` | derived evidence origin can represent guarded rule |
| `P4-EV03` | no fake exact nominal ConformanceEvidence |
| `P4-EV04` | derived evidence materializes exact subject |
| `P4-EV05` | derived evidence materializes exact consequent TraitRef |
| `P4-EV06` | derived evidence exposes associated binding map |
| `P4-EV07` | derived evidence exposes requirement selections |
| `P4-EV08` | nested conformance evidence retained |
| `P4-EV09` | equivalence/subtype proofs do not create runtime slots |
| `P4-EV10` | projections normalize from derived evidence |
| `P4-EV11` | generic assumed Iterator yields symbolic derived Iterable |
| `P4-EV12` | exact Iterator yields exact/materialized derived Iterable |
| `P4-EV13` | failed rule proof publishes no partial evidence |
| `P4-EV14` | runtime recipe consumes finite evidence DAG |
| `P4-EV15` | VM performs zero derived-rule proof |

---

## Coherence

| ID | Requirement |
|---|---|
| `P4-COH01` | P3 nominal conditional no-overlap rule unchanged |
| `P4-COH02` | derived rule has separate coherence category |
| `P4-COH03` | no candidate order precedence |
| `P4-COH04` | no most-specific rule |
| `P4-COH05` | no condition-disjointness solver |
| `P4-COH06` | explicit + successful derived same relation conflicts |
| `P4-COH07` | overlapping derived rules conflict |
| `P4-COH08` | imported rules participate in coherence |
| `P4-COH09` | adding antecedent conformance can reveal conflict and invalidates dependents |
| `P4-COH10` | exact enum-case identity remains exact |
| `P4-COH11` | rule ownership is consequent-trait ownership |
| `P4-COH12` | coherence conflict remains semantic, never runtime |

---

## Iterator law

| ID | Requirement |
|---|---|
| `P4-IT01` | canonical Iterator trait uses associated Item |
| `P4-IT02` | next returns Option<Item> |
| `P4-IT03` | Iterator.iterator returns self |
| `P4-IT04` | every proven Iterator derives Iterable |
| `P4-IT05` | derived Iterable::Item = Iterator::Item |
| `P4-IT06` | derived Iterable::Iterator = subject |
| `P4-IT07` | derived Iterable.iterator = self |
| `P4-IT08` | derived evidence satisfies Iterable trait contract |
| `P4-IT09` | generic `T |= Iterator` can satisfy callee `T |= Iterable` |
| `P4-IT10` | generic associated projection through derived Iterable normalizes |
| `P4-IT11` | custom user Iterator receives theorem |
| `P4-IT12` | cross-module custom Iterator receives theorem |
| `P4-IT13` | structural next/iterator shape alone does not receive theorem |
| `P4-IT14` | Iterator theorem is not encoded as supertrait |
| `P4-IT15` | Iterator theorem has no compiler/VM branch |

---

## Universe staging

| ID | Requirement |
|---|---|
| `P4-U01` | real Universe source defines canonical Iterator trait |
| `P4-U02` | real Universe source defines canonical Iterable trait |
| `P4-U03` | real Universe source defines Iterator⇒Iterable rule |
| `P4-U04` | public-name collision with legacy classes explicitly resolved |
| `P4-U05` | production cursor semantics remain functioning |
| `P4-U06` | production for lowering remains unchanged |
| `P4-U07` | collection migration remains P5 |
| `P4-U08` | lazy adapter migration remains P5 |
| `P4-U09` | legacy superclass retirement remains P5 |
| `P4-U10` | P4 handoff identifies exact P5 transition state |

---

## Integration / durability

| ID | Requirement |
|---|---|
| `P4-I01` | derived rule has stable semantic fingerprint |
| `P4-I02` | equivalent condition reorder is fingerprint-stable |
| `P4-I03` | rule condition edits invalidate consumers |
| `P4-I04` | antecedent conformance edits invalidate derived consumers |
| `P4-I05` | trait contract edits invalidate Iterable evidence |
| `P4-I06` | imported rule proves downstream goal |
| `P4-I07` | imported rule runtime behavior executes |
| `P4-I08` | cold/incremental parity |
| `P4-I09` | source index maps rule trait/projection occurrences |
| `P4-I10` | stable metadata contains no snapshot ids |

---

# 35. Proposed P4 coverage matrix

The later patch-grade plan should map every required case to exact tests/audits.

## Source / formation

| CV | Required discriminator |
|---|---|
| `P4-CV-01` | trait where-clause with `Self` forms |
| `P4-CV-02` | trait where-clause with `Self::Iterator` forms |
| `P4-CV-03` | qualified nested Item equality forms |
| `P4-CV-04` | canonical trait-contract order independence |
| `P4-CV-05` | malformed own-associated projection diagnoses |
| `P4-CV-06` | no associated-declaration-bound syntax dependency |

---

## Trait contract proof

| CV | Required discriminator |
|---|---|
| `P4-CV-07` | valid explicit Iterable contract succeeds |
| `P4-CV-08` | non-Iterator associated Iterator rejects |
| `P4-CV-09` | mismatched Iterator::Item rejects |
| `P4-CV-10` | generic `T |= Iterable` implies iterator conformance |
| `P4-CV-11` | generic `T |= Iterable` implies Item equality |
| `P4-CV-12` | imported Iterable assumption implies same contract |
| `P4-CV-13` | Dynamic does not satisfy implied conformance |
| `P4-CV-14` | recursive trait contract terminates non-proof |

---

## Derived rule formation

| CV | Required discriminator |
|---|---|
| `P4-CV-15` | guarded generic target rule forms |
| `P4-CV-16` | unguarded naked target rule rejects |
| `P4-CV-17` | unauthorized external consequent trait rule rejects |
| `P4-CV-18` | generic parameter spelling does not affect identity |
| `P4-CV-19` | where-order does not affect rule fingerprint |
| `P4-CV-20` | rule associated bindings resolve canonical requirements |
| `P4-CV-21` | rule witness remains conformance-owned, not inherent |

---

## Derived proof

| CV | Required discriminator |
|---|---|
| `P4-CV-22` | exact Iterator derives Iterable |
| `P4-CV-23` | non-Iterator does not derive Iterable |
| `P4-CV-24` | assumed generic Iterator derives assumed/derived Iterable |
| `P4-CV-25` | conditional Iterator evidence feeds derived Iterable |
| `P4-CV-26` | nested derived rule chain works |
| `P4-CV-27` | derived rule self-cycle terminates |
| `P4-CV-28` | derived rule mutual cycle terminates |
| `P4-CV-29` | budget exhaustion preserved |
| `P4-CV-30` | cancellation preserved |
| `P4-CV-31` | blocked/unknown provenance preserved |
| `P4-CV-32` | failed rule publishes no partial specialization |

---

## Derived associated evidence

| CV | Required discriminator |
|---|---|
| `P4-CV-33` | derived Iterable::Item normalizes to Iterator::Item |
| `P4-CV-34` | derived Iterable::Iterator normalizes to subject |
| `P4-CV-35` | nested projection inside ordinary type normalizes |
| `P4-CV-36` | generic symbolic derived projection remains canonical |
| `P4-CV-37` | cross-module derived projection normalizes |
| `P4-CV-38` | equality solver consumes derived binding |
| `P4-CV-39` | no runtime associated lookup is introduced |

---

## Derived behavior/runtime

| CV | Required discriminator |
|---|---|
| `P4-CV-40` | Iterator.iterator through derived Iterable returns same identity |
| `P4-CV-41` | generic callee requiring Iterable accepts Iterator evidence |
| `P4-CV-42` | derived witness receives correct type env |
| `P4-CV-43` | derived witness receives correct nested evidence env |
| `P4-CV-44` | repeated derived calls reuse runtime environments |
| `P4-CV-45` | derived evidence callable survives required GC/lifetime path |
| `P4-CV-46` | VM contains zero rule/proof search |

---

## Coherence

| CV | Required discriminator |
|---|---|
| `P4-CV-47` | explicit Iterable + derived Iterable conflict |
| `P4-CV-48` | rule-rule overlap conflict |
| `P4-CV-49` | source declaration order irrelevant |
| `P4-CV-50` | imported rule participates in conflict detection |
| `P4-CV-51` | P3 same-head/different-condition rejection remains green |
| `P4-CV-52` | exact enum-case relation remains exact |

---

## Cross-module / incremental

| CV | Required discriminator |
|---|---|
| `P4-CV-53` | provider rule proves consumer Iterator→Iterable |
| `P4-CV-54` | provider rule runtime executes downstream |
| `P4-CV-55` | antecedent conformance removal invalidates derived consumer |
| `P4-CV-56` | antecedent conformance re-add restores derived consumer |
| `P4-CV-57` | rule condition edit invalidates consumer |
| `P4-CV-58` | rule canonical reorder is semantic no-op |
| `P4-CV-59` | Iterable trait contract edit invalidates evidence |
| `P4-CV-60` | cold/incremental parity |

---

## Performance/resource

| CV | Required discriminator |
|---|---|
| `P4-CV-61` | derived rule lookup uses relevant index, not global scan |
| `P4-CV-62` | Iterator proof reused while validating derived Iterable contract |
| `P4-CV-63` | proof count follows unique goals, not path count |
| `P4-CV-64` | runtime conformance registry growth bounded by distinct envs |
| `P4-CV-65` | repeated invocations do not grow registry linearly |

---

## Universe / P5 boundary

| CV | Required discriminator |
|---|---|
| `P4-CV-66` | canonical Universe Iterator trait loads |
| `P4-CV-67` | canonical Universe Iterable trait loads |
| `P4-CV-68` | Universe Iterator derives Iterable |
| `P4-CV-69` | legacy production `for` smoke remains unchanged |
| `P4-CV-70` | legacy collection cursor smoke remains unchanged |
| `P4-CV-71` | no P5 collection conformance migration included |
| `P4-CV-72` | no cursor selector deletion included |

---

## P3 closure carryover

| CV | Required discriminator |
|---|---|
| `P4-CV-73` | owner-generic evidence runtime disposition |
| `P4-CV-74` | owner+method evidence runtime disposition |
| `P4-CV-75` | exact enum-case generic disposition |
| `P4-CV-76` | callable-reference Outcome A/B |
| `P4-CV-77` | P3 memo/resource evidence repaired |
| `P4-CV-78` | P3 runtime env reuse evidence repaired |
| `P4-CV-79` | P3 rooting evidence repaired/strengthened |
| `P4-CV-80` | records/checkpoint/guidance synchronized |

---

# 36. Verification gate architecture for the later plan

The patch-grade plan should probably use gates equivalent to:

```text
G0 — P3 closure and live-baseline gate

G1 — subject-aware trait-contract / implied-constraint gate

G2 — guarded derived-rule source/coherence gate

G3 — recursive derived proof + associated evidence gate

G4 — runtime derived behavior/evidence gate

G5 — cross-module / incremental / metadata / tooling gate

G6 — canonical Universe law + legacy-production coexistence gate

G7 — final focused P4 stabilization
```

This is only a requirements decomposition.

The next artifact should turn it into exact patch steps/files/tests.

---

# 37. Build / stabilize testing budget requirements

P4 should follow the same Luna discipline.

## BUILD

Run:

```text
one new discriminator
then the owning focused module
```

Do not run workspace-wide tests after every patch.

---

## STABILIZE

At coherent gates run:

```text
new P4 semantic rule/trait-contract module
P3 conditional_conformance semantic regression
P2 generic_associated_constraints regression
P1 generic_conformance regression
focused core derived-rule runtime module
focused Universe loader/iteration smoke
incremental filters actually touched
```

---

## CERTIFY

P4 is still not C6 completion.

Do not automatically run:

```text
cargo test --workspace
cargo clippy --workspace
full corpus
all LSP
release build
```

unless a named gate or actual evidence requires it.

---

# 38. STOP / CONSULT triggers for the P4 plan

The later patch-grade plan should treat the following as hard escalation points.

## Architecture

STOP if:

1. generic-target derived conformance requires replacing C4 `ConformanceIndex`;
2. correct implementation requires treating a type parameter as a nominal `ConformanceTarget`;
3. a fake exact `ConformanceEvidence` must be created for a derived rule;
4. `TraitRef` identity must absorb `Self`;
5. Iterable contract proof requires a second generic constraint system;
6. associated contract formation requires a second associated requirement identity;
7. associated normalization requires a second normalizer;
8. derived rule correctness requires runtime search;
9. derived rule correctness requires reverse open-world inference;
10. cycles must count as proof;
11. explicit-vs-derived conflict requires “most specific wins”;
12. rule overlap requires general specialization;
13. rule ownership cannot be expressed through consequent-trait ownership;
14. stable rule metadata requires snapshot-local IDs.

---

## Lifetime/runtime

STOP if:

15. derived rule witness execution requires a new persistent unrooted side table;
16. callable references require a new heap evidence-capture object;
17. runtime type and conformance environments must be merged;
18. VM must inspect `GenericConstraint`;
19. VM must normalize associated projections;
20. installing Universe traits requires deleting production cursor behavior before replacement exists.

---

## Universe migration

STOP if:

21. public Iterator/Iterable trait installation cannot coexist with legacy behavior without simultaneously performing P5;
22. native bootstrap class identity uses `Iterable` in a way that cannot be transitionally internalized;
23. renaming legacy bases breaks an externally frozen ABI/public contract not already superseded by the current iterator spec;
24. production `for` lowering must change to make P4 tests pass.

---

## Semantic regressions

STOP if:

25. P3 exact/assumed/conditional evidence must be weakened;
26. exact enum-case identity must widen;
27. P2 one-way inference must become reverse search;
28. Dynamic must become static proof;
29. same-head conditional overlap must become legal;
30. P4 requires supertraits to complete the Iterator law.

---

# 39. Specific live-repository questions the P4 plan must VERIFY FIRST

Before first production edit:

```text
1. exact P4 start HEAD / intervening commits
2. three ignored P3 predecessor runtime tests
3. callable-reference evidence lifetime
4. current P3 resource test strength
5. trait header formation ordering
6. whether trait where-clause currently sees Self
7. whether trait where-clause can see own associated declaration identities
8. whether TraitRef::form remains used by paths that should move to subject-aware proof
9. where trait GenericSignature is published/imported
10. where conformance source products are serialized between modules
11. exact ConformanceIndex internal buckets/indexes
12. parser behavior for `impl<I> Trait for I`
13. exact diagnostic currently rejecting generic target
14. current conformance ownership check assumptions
15. derived evidence consumers that directly unwrap exact_source
16. projection normalization paths that assume exact ConformanceEvidence
17. runtime recipe consumers that assume Derived means conditional nominal
18. Universe module/global name collision rules
19. every source/runtime reference to current Iterable class identity
20. every source/runtime reference to current Iterator class identity
21. bootstrap superclass wiring for List/Map/Set/Tuple/Range/Bytes
22. current legacy iterator smoke filters
23. current stable metadata representation for source conformance products
24. incremental dependencies recorded for conformance index queries
```

The later plan should name exact live files after this re-grounding.

---

# 40. Recommended implementation-order shape for the later patch-grade plan

Without writing the patch-grade plan itself, the requirements imply this dependency order:

```text
P4.T0
    close/classify P3 inherited gaps
    synchronize records
        ↓
P4.T1
    subject-aware trait GenericSignature / Self contract proof
        ↓
P4.T2
    trait implied constraints in lexical evidence
        ↓
P4.T3
    guarded derived-conformance source product + ownership/coherence
        ↓
P4.T4
    recursive rule application in P3 proof context
        ↓
P4.T5
    generalized derived associated/behavioral evidence
        ↓
P4.T6
    runtime lowering/reuse/rooting
        ↓
P4.T7
    cross-module/incremental/metadata/tooling
        ↓
P4.T8
    canonical Universe Iterator/Iterable law + transition staging
        ↓
P4.T9
    final focused stabilization / walkthrough / P5 handoff
```

The next response can turn this into Luna-sized patch tasks with exact BUILD tests and STOP/CONSULT boundaries.

---

# 41. P5 takeover contract P4 must produce

P4 should finish with P5 able to assume:

```text
Iterator trait is canonical

Iterable trait is canonical

Iterable contract is enforced:
    Iterable.Iterator |= Iterator
    Iterable.Iterator.Item == Iterable.Item

Iterator evidence automatically derives Iterable evidence

derived Iterable evidence provides:
    Item
    Iterator
    iterator witness

generic code can use the theorem

cross-module imported code can use the theorem

derived evidence is incrementally correct

runtime executes prepared evidence only

canonical Universe contains the law

legacy production cursor path is still operational
and explicitly marked transitional
```

Then P5 can focus on:

```text
collection conformances
stateful iterator objects
for checker/lowering
Map entries
RangeIterator
lazy adapters
legacy retirement
```

without designing trait implication.

---

# 42. Non-goals

P4 must not expand into:

```text
general trait inheritance / supertraits
trait objects / existentials
negative conformances
specialization
most-specific derived rules
condition-domain disjointness solving
generic associated types
higher-kinded conformance rules
reverse associated-output inference
runtime trait solver
runtime projection dictionary
production collection migration
production for lowering
Map iteration semantic flip
Range iterator production rewrite
lazy adapter stateful rewrite
cursor deletion
full C6 certification
```

---

# 43. Final architectural assessment

The repository now has enough generic proof machinery to implement the Iterator law without a runtime hack.

The remaining missing abstraction is not “more recursive proof.”

It is:

```text
a first-class source-level conformance implication
whose subject is universally quantified
and whose result is complete evidence.
```

The best fit with Phalcom's existing architecture is a **guarded derived-conformance rule**:

```text
source:
    ordinary generic impl syntax

semantic category:
    proof rule, not nominal exact target

conditions:
    ordinary GenericSignature

proof:
    existing P3 ConformanceProofContext

associated meaning:
    existing associated projection system

behavior:
    ordinary trait requirement witness relation

runtime:
    existing evidence environments

coherence:
    unique proof, no ranking

ownership:
    consequent trait owner

standard-library first use:
    Iterator ⇒ Iterable
```

At the same time, P4 must finish the subject-aware trait-contract seam so `Iterable` can say what its associated iterator is required to be.

Those two mechanisms together produce the complete law:

```text
Iterable evidence
    entails
        Iterable.Iterator |= Iterator
        Iterable.Iterator.Item == Iterable.Item

Iterator evidence
    derives
        Iterable evidence
        with Iterator=self
             Item=Iterator.Item
             iterator=self
```

That is the semantic foundation P5 needs.

---

# 44. External reference bibliography

The following sources informed the design comparison. They are references, not Phalcom authorities.

## Rust

- Rust standard iterator module, blanket `IntoIterator` for every `Iterator`:
  https://doc.rust-lang.org/std/iter/
- `IntoIterator` associated contract:
  https://doc.rust-lang.org/beta/std/iter/trait.IntoIterator.html
- rustc trait solving overview:
  https://rustc-dev-guide.rust-lang.org/solve/trait-solving.html
- rustc goals and clauses:
  https://rustc-dev-guide.rust-lang.org/traits/goals-and-clauses.html

## Chalk

- Lowering rules:
  https://rust-lang.github.io/chalk/book/clauses/lowering_rules.html
- Coherence:
  https://rust-lang.github.io/chalk/book/clauses/coherence.html

## Swift

- SE-0143 Conditional conformances:
  https://github.com/swiftlang/swift-evolution/blob/main/proposals/0143-conditional-conformances.md

## Pyrefly

- recursive protocol performance / cache soundness regression tests:
  https://github.com/facebook/pyrefly/blob/main/pyrefly/lib/test/perf.rs

## Astral `ty` / Ruff

- current type/protocol implementation lives in the Ruff repository:
  https://github.com/astral-sh/ruff/tree/main/crates/ty_python_semantic

---

# 45. Acceptance statement for this requirements analysis

This analysis is ready to feed the P4 patch-grade plan when the plan:

1. treats `2da270877ee34693357784d9e2077f63da721802` as the current planning baseline unless live HEAD moved;
2. begins with bounded P3 closure rather than silently trusting P3 records;
3. separates the `Iterable` trait contract from `Iterator ⇒ Iterable`;
4. uses subject-aware trait contract proof rather than TraitRef identity hacks;
5. represents generic-target conformance as a guarded derived proof rule rather than fake nominal target identity;
6. preserves global uniqueness without introducing specialization;
7. generalizes P3 derived evidence enough to carry rule-owned associated bindings and behavioral evidence;
8. keeps all proof in semantics and all runtime execution preselected;
9. installs the law in real Universe source while keeping P5 production migration deferred;
10. maps the P4 requirements and coverage surface above into small Luna-executable tasks and verification gates.
