---
id: LANG005.C5.GUIDANCE
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: implementer-guidance
status: ACTIVE_PLANNING_GUIDANCE
requires:
  - LANG005.C5-CHECKPOINT.md
  - LANG005.C5.P1-associated-type-declarations-and-binding-foundations-plan.md
planning_baseline_revision: e5152196d716ed70fc47d104a0fd789e602285db
prepared: 2026-09-15
repository: aureat/phalcom-lang
---

# Implementer Guidance — LANG005.C5 Associated Types, Projection, and Property-Requirement Integration

## 0. Purpose

This document is the persistent architectural and execution guidance for all `LANG005.C5` implementation sessions.

Load it before working on:

```text
C5.P1 associated declarations/bindings + property/via foundation
C5.P2 associated projection formation/normalization
C5.P3 vertical integration / Iterable migration / certification
```

The checkpoint's core semantic sentence is:

> Associated types are trait-owned declarations whose concrete meanings come from conformance evidence. Trait property declarations are behavioral getter/setter contracts, not storage requirements. `via` is direct-field accessor elaboration, not runtime delegation. Projection is normalized semantically before compiler/runtime consumption.

Every C5 patch must preserve that sentence.

---

# 1. Current checkpoint state

At initial planning time:

```text
checkpoint: LANG005.C5
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
active plan: LANG005.C5.P1
```

C4 focused semantics are the required predecessor architecture. C4 remains broadly `BASELINE_BLOCKED`; this is not permission to redesign conformance and is not, by itself, a blocker for focused C5 implementation.

A fresh implementer must begin with the active plan's T0 takeover/re-grounding procedure.

Do not infer live repository state from this document alone.

Current durable state after P1:

```text
checkpoint: LANG005.C5
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
completed plan: LANG005.C5.P1 (COMPLETE / IMPLEMENTED / FOCUSED_TESTED)
next plan: LANG005.C5.P2
```

P1's associated declaration, binding-plan, exact-evidence, completeness,
property, and direct-field `via` interfaces are frozen for P2. The inherited
C5-BL-07 Universe Bool capability failures remain classified baseline D.

P2 is now complete as `IMPLEMENTED / FOCUSED_TESTED`. Its canonical
`AssociatedProjection` carrier, contextual `Self::Item` formation, singular
`types::normalize_type` authority, exact evidence/signature integration,
owner-complete incremental products, and source-index target bridge are frozen
for P3. The next plan is responsible for Iterable migration, cross-stack
integration, and broader certification; it must not add generic trait-bound
assumption semantics or a runtime projection solver.

---

# 2. Canonical repository location

Use:

```text
docs/implementation/LANG005/LANG005.C5/
```

with:

```text
LANG005.C5-CHECKPOINT.md
LANG005.C5-GUIDANCE.md
LANG005.C5.P1-associated-type-declarations-and-binding-foundations-plan.md
```

Later P2/P3 plans, walkthroughs, handoffs, and genuine companion technical records belong in the same checkpoint directory.

Do not reorganize older LANG005 checkpoint directories while implementing C5.

---

# 3. Required read order

Before any C5 production edit, read in this order.

## 3.1 Workflow authority

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
```

## 3.2 Normative language authority

Read the live authoritative equivalents of:

```text
docs/specs/objects/traits.md
field/class/object-model specifications
callable / selector specifications
visibility/access specifications
type-expression / generic-signature specifications
impl/conformance specifications if split from trait spec
exact enum-case identity specification
```

The normative specs define language behavior. Plans define implementation work.

## 3.3 Checkpoint/predecessor state

```text
LANG005.C4-CHECKPOINT.md
LANG005.C4-GUIDANCE.md
LANG005.C4.P4-walkthrough.md
LANG005.C4.P4-handoff.md
LANG005.C5-CHECKPOINT.md
LANG005.C5-GUIDANCE.md
active C5 plan
latest completed C5 walkthrough/handoff, if any
```

## 3.4 Production architecture

Inspect the live equivalents of:

```text
TraitRef
TraitRequirementId
TraitSurface
ImplId
ConformanceIndex
ConformanceHeadMatch
ConformanceWitnessPlan
ConformanceEvidence
ConformanceCompleteness
TraitDispatch* products
FieldId
FieldSemanticSignature
class/trait/impl AST member categories
field/accessor parsing and semantic publication
TypeTerm / type annotation formation
TypeEnvironment / TypeSubstitution / generic specialization
semantic DB query/product/fingerprint ownership
source index / editor / LSP targets
semantic-to-core lowering
runtime type environment / RuntimeTypeRecipe
```

Do not trust stale filenames when live code moved. Preserve roles, identity, and ownership.

---

# 4. Authority and drift rules

## 4.1 Semantic authority order

When sources disagree, use:

```text
1. explicit user-ratified C5 decisions recorded by checkpoint/guidance amendments
2. current authoritative specs under docs/specs/
3. accepted C5 checkpoint amendments
4. completed C4 checkpoint/walkthrough/handoff contracts
5. landed production architecture
6. active C5 implementation plan for mechanics/task order
7. historical plans/support fixtures
8. experimental/unstructured documents
```

A design example in an unstructured document does not outrank the checkpoint's accepted semantics.

## 4.2 Live mechanics

For implementation mechanics:

```text
live repository
    >
planning-time helper/file names
```

Mechanical drift may be adapted locally.

Architectural drift involving identity, ownership, proof authority, representation, projection semantics, or runtime responsibility is a STOP/CONSULT condition.

## 4.3 Checkpoint state authority

`LANG005.C5-CHECKPOINT.md` is the single durable authority for:

```text
what C5 owns
what C5 defers
which plan is active
which invariants are fixed/implemented
which gates actually passed
which failures are deferred
which consultations/amendments were adopted
```

Update it at durable gates, not after every edit.

## 4.4 Plan authority

The active patch-grade plan owns:

```text
task dependency order
specific file/symbol expectations
focused test budget
gate commands
acceptance criteria
local STOP/CONSULT triggers
walkthrough/handoff requirements
```

Do not move P2 or C6 work into P1 because it appears convenient.

## 4.5 Walkthrough/handoff precedence

After a plan lands, its walkthrough records what actually exists. Subsequent sessions should prefer completed walkthrough/handoff facts over speculative mechanics in the original plan, while preserving checkpoint invariants.

---

# 5. Session bootstrap

At the start of every C5 implementation session:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then answer privately:

```text
Which C5 plan/task/gate am I implementing?
What predecessor revision and walkthrough am I inheriting?
Are there unrelated local edits?
Which semantic product is the canonical authority for this task?
Am I about to duplicate an existing identity/query/completeness mechanism?
What is the smallest discriminating test for the current change?
```

Preserve unrelated modified/staged/untracked work. No reset/clean/broad staging.

---

# 6. Semantic model to keep loaded

Keep this pipeline mentally visible:

```text
trait declaration
    ↓
TraitRef
    ↓
TraitSurface
    ├─ behavioral TraitRequirementId requirements/defaults
    ├─ property-shaped source declarations
    │      ↓ elaborate
    │   getter/setter TraitRequirementId requirements
    └─ AssociatedTypeRequirementId declarations

source target behavior
    ├─ explicit methods/getters/setters
    ├─ data/property-like existing witnesses
    └─ direct-field `via`
           ↓ elaborate
       ordinary getter/setter implementations

source conformance
    impl TraitRef for Target
    ↓
ImplId
    ↓
ConformanceHeadMatch
    ↓
source conformance plans
    ├─ ConformanceWitnessPlan
    └─ associated binding plan
    ↓
exact specialization
    ↓
ConformanceEvidence
    ├─ behavioral witness/default selections
    ├─ exact associated bindings
    └─ unified completeness/proof state
    ↓
associated projection
    ↓
semantic normalization
    ↓
compiler/runtime projection of already-decided facts
```

The important non-equalities are:

```text
TraitRequirementId != AssociatedTypeRequirementId
property source declaration != physical field
`via` declaration != field declaration
`via` declaration != runtime delegate object
source binding template != exact associated binding
associated declaration != projection
projection formation != normalization
ImplId != exact applied conformance
ConformanceHeadMatch != ConformanceEvidence
semantic normalization != runtime lookup
```

If an implementation erases one of these distinctions, stop.

---

# 7. State/property requirement reconciliation — fixed

This is no longer an open design question.

A trait declaration:

```phalcom
trait Counter {
  private mut count: Int

  increment {
    count++
  }
}
```

means a behavioral property contract equivalent to requirements:

```phalcom
private count -> Int
private count=(_: Int) -> ()
```

It does not imply:

```text
a physical `count` slot
a required backing field
trait-injected representation
conformance-added representation
abstract storage identity
```

Therefore C5 must not introduce:

```text
StateRequirementId
TraitFieldId
AssociatedStateRequirement
AssociatedStateBinding
trait layout contribution
conformance storage plan
```

The higher-level property declaration should remain available as source/provenance grouping where useful for diagnostics, hover, documentation, or semantic tokens.

Conformance satisfaction remains ordinary C4 behavioral witness selection.

Correct architecture:

```text
trait property source
    ↓
ordinary getter/setter requirements

concrete field + `via` source
    ↓
ordinary getter/setter implementations

C4 witness compatibility
    ↓
ConformanceEvidence
```

Incorrect architecture:

```text
trait field requirement
    ↓
match backing storage by name/type
```

Do not implement the latter.

---

# 8. Direct-field `via` — fixed initial model

## 8.1 Accepted syntax

```phalcom
count via _count
```

Getter-only delegation.

```phalcom
count=(_) via _count
```

Setter-only delegation.

```phalcom
mut count via _count
```

Getter + setter delegation.

These are alternative declarations for one accessor surface, not three declarations intended to coexist.

## 8.2 Delegate target

P1 accepts only a directly named field owned by the exact target declaration.

Valid conceptual target:

```phalcom
mut _count: Int
mut count via _count
```

Out of scope:

```phalcom
count via state.count
count via items[index]
count via object.property
count via computeStorage()
```

No superclass search.

Fields are private/non-inherited representation. If a parent needs to expose state to a child, it does so through protected/otherwise accessible behavior, not inherited field slots.

## 8.3 Type authority

The canonical `FieldSemanticSignature` supplies:

```text
field identity
owner
side
value type
mutability
source
```

The delegated accessor must not repeat an independent property type source of truth.

Getter-only delegation requires readable field semantics.

Setter-only or `mut` delegation requires writable/mutable field semantics.

## 8.4 Elaboration

Given:

```phalcom
mut _count: Int
mut count via _count
```

semantic/compiler behavior must converge with ordinary accessors equivalent to:

```phalcom
count -> Int {
  _count
}

count=(_ value: Int) -> () {
  _count = value
}
```

`value` above may be a compiler-internal binding. The source form `count=(_) via _count` deliberately need not create a source-visible local name.

## 8.5 Class and impl declarations

Both are accepted:

```phalcom
class Counter {
  mut _count: Int
  mut count via _count
}
```

and:

```phalcom
class Counter {
  mut _count: Int
}

impl Counter {
  mut count via _count
}
```

Both must contribute ordinary target behavior through the canonical inherent-member machinery.

## 8.6 Conflict law

Generated members do not have special precedence.

This is a conflict:

```phalcom
mut count via _count
count=(_ value: Int) { ... }
```

because the setter exists twice.

For custom setter behavior:

```phalcom
count via _count
count=(_ value: Int) { ... }
```

For custom getter behavior:

```phalcom
count=(_) via _count
count -> Int { ... }
```

Do not implement "explicit beats generated" or "generated silently disappears" rules.

## 8.7 Runtime boundary

No `Via` bytecode, runtime delegate descriptor, dynamic forwarding protocol, or runtime field-name lookup is required.

If a patch starts inventing one, stop.

---

# 9. Parameter-label/local-binding rule relevant to C5

External labels participate in selector identity. Local parameter bindings do not.

Accepted conceptual forms include:

```text
_ value: T      label `_`, local `value`
_: T            label `_`, discarded local
_ _: T          verbose discarded form
from origin: T  label `from`, local `origin`
from: T         label `from`, local `from`
from from: T    verbose same-name form
from _: T       label `from`, discarded local
```

All corresponding external-label/type shapes are callable-signature compatible regardless of local binding spelling.

For bodyless property requirements, use concise/discarded forms naturally:

```phalcom
count=(_: Int) -> ()
```

C5 must not rebuild callable identity around local names.

---

# 10. Associated type declaration identity

Associated declarations are trait-owned semantic requirements.

Example:

```phalcom
trait Iterator {
  type Item
}
```

The associated requirement remains the same requirement across every conformance.

A suitable conceptual identity is:

```rust
AssociatedTypeRequirementId {
    owner: DeclarationId,
    index: u32,
}
```

Equivalent owner-relative stable identity is acceptable if it follows repository conventions.

Fixed properties:

```text
trait-owned
stable across conformances
same spelling in different traits remains distinct
name is presentation, not identity
not a callable identity
not a generic parameter identity
```

Do not extend `TraitRequirementId` with a “kind” variant solely to avoid a new associated-type ID; that would mix behavior and type requirements and destabilize C4 semantics.

---

# 11. AST/member taxonomy

Current trait/impl member lists are behavior-only. C5 must introduce explicit heterogeneous categories rather than encoding associated types as methods.

Conceptually:

```rust
enum TraitMember {
    Behavior(BehaviorMember),
    Property(TraitPropertyRequirement),
    AssociatedType(AssociatedTypeDeclaration),
}

enum ImplMember {
    Behavior(BehaviorMember),
    Delegation(AccessorDelegation),
    AssociatedTypeBinding(AssociatedTypeBinding),
}
```

The exact private AST factoring is mechanically flexible.

Fixed semantic requirements:

```text
trait associated declaration is not BehaviorMember
conformance associated binding is not BehaviorMember
property source remains distinguishable for provenance before/while elaborating to behavior
via source remains distinguishable before/while elaborating to behavior
ordinary inherent/conformance member interpretation remains explicit
```

Do not make `type Item` parse as a top-level type alias accidentally. `Token::TypeKw` already serves top-level aliases; member context must disambiguate intentionally.

---

# 12. TraitSurface policy

C5 extends `TraitSurface` with a separate associated requirement table.

Conceptually:

```rust
TraitSurface {
    declaration,
    generic_signature,
    members: BTreeMap<TraitRequirementId, TraitSurfaceMember>,
    associated_types: BTreeMap<AssociatedTypeRequirementId, TraitAssociatedTypeRequirement>,
    diagnostics,
}
```

The behavior table remains behavior-only.

A secondary name→ID lookup may exist for efficient LHS/projection resolution, but canonical identity is the associated requirement ID.

Trait property declarations elaborate into `members`; they do not enter `associated_types`.

Changing associated declaration shape must participate in the trait-surface fingerprint and source replacement lifecycle.

---

# 13. Source associated binding plan

A source conformance:

```phalcom
impl<T> Iterable for List<T> {
  type Item = T
}
```

contains one source binding template owned/provenanced by the same source `ImplId` as the conformance.

The binding LHS resolves only against the exact trait declaration referenced by the conformance:

```text
ImplDef
  → TraitRef
  → TraitSurface
  → associated name index
  → AssociatedTypeRequirementId
```

Do not globally search all `Item` declarations.

The RHS is formed using ordinary type-annotation/generic-environment machinery.

Associated bindings are not behavioral witness selections. Do not put them inside `RequirementSelectionTemplate` or another callable-only C4 enum simply because that data already lives near conformance code.

Preferred conceptual product:

```rust
AssociatedTypeBindingTemplate {
    requirement: AssociatedTypeRequirementId,
    value: TypeTerm-or-canonical-binding-template,
    source_impl: ImplId,
    source: SemanticSourceSpan,
}

ConformanceAssociatedTypePlan {
    source_impl: ImplId,
    bindings: BTreeMap<AssociatedTypeRequirementId, AssociatedTypeBindingTemplate>,
    diagnostics: Box<[SemanticDiagnostic]>,
}
```

Exact field names/types may adapt to the live type-formation API.

---

# 14. Exact associated binding evidence

The same source conformance may yield different exact associated binding values.

Example:

```phalcom
impl<T> Iterable for List<T> {
  type Item = T
}
```

One source identity:

```text
ImplId X
```

Exact evidence:

```text
List<Int> + Iterable
    source_impl = X
    Item = Int

List<String> + Iterable
    source_impl = X
    Item = String
```

Use the same exact conformance environment/substitution that C4 already established for requirement specialization.

Do not create per-application source implementation IDs.

Do not erase exact enum-case identity during specialization.

`ConformanceEvidence` is the canonical exact place for the binding map because it already means “why this exact target satisfies this exact TraitRef.”

---

# 15. Conformance completeness policy

C5 must retain one final conformance completeness/proof-state authority.

After P1, conceptual completeness is:

```text
all behavioral requirements are satisfied
AND
all required associated types are bound and valid
```

Do not create separate final booleans such as:

```text
behavior_complete
associated_complete
```

with competing consumer logic.

It is acceptable to use internal sub-results during analysis, but they must converge into `ConformanceCompleteness` (or its live equivalent) before consumers decide whether evidence is complete.

Missing associated binding must not be encoded as a fake behavioral `RequirementFailure` keyed by a nonexistent `TraitRequirementId`. Generalize the failure representation cleanly or add an adjacent failure category while preserving one final completeness enum.

Existing non-success proof states—unknown, blocked, dynamic, cancelled/budget/internal as applicable—must remain representable.

---

# 16. P1 boundary

P1 owns only the foundation required before projection.

P1 answers:

```text
Which associated type declarations does this trait own?
Which source bindings does this conformance provide?
What exact binding values result for this exact conformance application?
Is the conformance complete when behavioral + associated obligations are combined?
Which getter/setter requirements result from a property declaration?
Which ordinary accessor implementations result from direct-field via?
```

P1 does not answer:

```text
What does Self::Item mean in arbitrary type positions?
How do projections normalize recursively?
How does T::Item work under a generic trait bound?
```

Those belong to P2/C6.

---

# 17. P2 — Projection formation

P2 introduces a distinct type-level projection representation.

Do not reuse expression-level associated member lookup as the canonical type projection.

The semantic projection must retain the associated requirement identity, not just text.

Conceptually:

```rust
AssociatedTypeProjection {
    subject: TypeTerm,
    trait_ref_or_context: TraitRef / symbolic trait application,
    requirement: AssociatedTypeRequirementId,
    source: ...,
}
```

The exact representation may use existing type-store nodes rather than this struct; the identity requirements are fixed.

Initial contextual surface:

```phalcom
Self::Item
```

inside the owning trait/conformance context.

Do not invent a broad `T::Item` rule without proof of which trait owns `Item` and why `T` conforms.

---

# 18. P2 — Projection normalization

Normalization consumes canonical conformance/associated-binding evidence.

Expected conceptual path:

```text
projection subject
    ↓
relevant exact/symbolic trait relationship
    ↓
ConformanceEvidence or abstract trait context
    ↓
AssociatedTypeRequirementId binding
    ↓
substitution / recursive normalization
    ↓
normalized TypeTerm / TypeId / proof-state result
```

Required behaviors:

```text
exact target binding normalization
generic source specialization
exact enum-case preservation
abstract Self projection in trait checking
projection inside trait method signatures/defaults
ambiguity retention/diagnosis
recursive/cyclic projection detection
unknown/blocked/dynamic-boundary preservation
incremental dependency recording
```

Never map “could not prove” directly to `Dynamic` unless the actual type-system rule says the source crossed a Dynamic boundary.

P2's implemented authority is singular: `types::normalize_type` performs the
recursive walk and cycle detection for abstract, source-conformance, and exact
contexts. T5 may stage bindings, schedule dependencies, mutate its plan, and
retain failure provenance, but it must invoke this normalizer rather than
reimplementing source recursion. Source mode preserves a missing sibling as a
symbolic residual until conformance completeness reports `Missing`; exact mode
consumes exact evidence and never uses source-name lookup or runtime lookup.

---

# 19. P2 and explicit projection syntax

Only contextual `Self::Item` is currently ratified enough to plan as C5 core syntax.

Possible future explicit qualification:

```text
<T as Iterable>::Item
```

is not ratified.

If implementation discovers that correct ambiguity handling requires a new explicit user-facing syntax during P2, this is a language-design fork:

```text
STOP
collect examples and semantic need
consult
record accepted decision in checkpoint
amend P2 plan if required
```

Do not silently choose Rust syntax or invent another spelling.

---

# 20. C6 boundary — generic trait proof

The following patterns require generic trait-proof machinery and therefore belong to C6:

```phalcom
foo<T: Iterable>(...)
```

and conditional conformance such as:

```phalcom
impl<T: Hashable> Hashable for List<T> { ... }
```

C6 must introduce a canonical trait-conformance generic constraint / assumption model rather than C5 adding ad hoc proof flags.

C6 will consume C5's:

```text
AssociatedTypeRequirementId
associated binding plan/evidence
projection identity
projection normalization
```

and extend them with assumed/nested conformance evidence.

Do not move C6 work backward into C5 to support `T::Item` prematurely.

---

# 21. Associated type constraints, defaults, and GATs

The trait specification may reserve associated type constraints conceptually, but C5 first-version planning is intentionally conservative.

P1 supports the foundational declaration:

```phalcom
type Item
```

and binding:

```phalcom
type Item = T
```

Do not invent trait-bound associated constraints such as:

```phalcom
type Item: Printable
```

without the C6 trait-proof substrate.

Do not add associated type defaults.

Do not add generic associated types.

If the live authoritative spec already ratified a source form that can be represented entirely with existing subtype/equivalence constraints, verify it and consult before expanding the active plan.

---

# 22. Exact enum-case policy

C4 permits exact enum-case conformance where the target semantics allow it.

C5 must preserve exact case identity through associated binding specialization and projection.

Do not normalize:

```text
Result<Int>::Ok(_)
```

to merely:

```text
Result<Int>
```

when selecting/normalizing associated bindings tied to the exact case.

The exact `VariantId` / case type environment remains part of semantic evidence.

---

# 23. Incremental/database doctrine

C5 adds several semantic products that must obey existing incremental ownership rules.

## 23.1 Trait-surface changes

Changing:

```phalcom
trait T {
  type Item
}
```

by adding/removing/renaming/reordering identity-bearing associated declarations must update the associated TraitSurface shape/fingerprint and invalidate dependents.

## 23.2 Binding RHS changes

Changing:

```phalcom
type Item = T
```

to:

```phalcom
type Item = Box<T>
```

should invalidate the source associated binding plan, exact evidence, dependent projections, and downstream semantic products—not unrelated trait declaration surfaces.

## 23.3 `via` dependencies

Changing delegate field type or mutability must invalidate generated accessor semantics.

Changing `mut count via _count` to `count via _count` must remove only the setter contribution while retaining the getter.

## 23.4 Removal/replacement

Source replacement must owner-completely remove:

```text
associated requirement identities/products
associated binding products
exact evidence dependent on removed binding
property-elaborated requirements
via-generated accessors
source-index/navigation entries
projection products/dependencies
```

No stale ID/product may remain observable after incremental delete/rename.

## 23.5 Cold/incremental equivalence

By C5 closure, cold build and equivalent incremental mutation sequence must converge on the same semantic products/diagnostics for C5 features.

Do not excuse a new C5 mismatch because an unrelated inherited A7 presentation mismatch exists.

---

# 24. Source index / editor / LSP doctrine

Tooling consumes semantic identity.

P1 minimum:

```text
associated declaration has canonical source target
binding LHS resolves/navigates to that declaration
property/delegation declaration preserves useful source provenance
```

P2/P3:

```text
Self::Item projection resolves to canonical associated requirement
hover/presentation can show normalized/exact meaning when available
ambiguity/blocked states remain semantically honest
completion does not invent associated items from name-only searches
```

Do not build a second LSP-only associated-type resolver.

---

# 25. Compiler/lowering doctrine

The compiler is a consumer of semantic decisions.

For `via`:

```text
semantic accessor elaboration
    ↓
ordinary getter/setter lowering
    ↓
ordinary bytecode/runtime execution
```

For associated types:

```text
semantic associated binding/projection normalization
    ↓
concrete/symbolic lowered type facts as required
    ↓
existing runtime type recipe/environment if runtime reification is necessary
```

Do not ask the compiler to rediscover which trait `Item` belongs to.

Do not ask the compiler to scan conformance source declarations to decide a binding.

---

# 26. Runtime doctrine

Runtime is never the semantic authority for C5.

Forbidden runtime behaviors:

```text
name-based associated-type lookup
trait declaration scanning
conformance record scanning
class dictionary scan to infer conformance
projection solver
`via` protocol dispatch
implicit backing-field discovery
```

Permitted runtime behavior:

```text
execute ordinary accessor bytecode
consume already-lowered invocation plans
instantiate already-formed runtime type recipes/environments
carry exact reified type identity when the broader runtime model requires it
```

If correct implementation appears to require runtime semantic solving, STOP AND CONSULT.

---

# 27. Surface compatibility: `mut`

The ratified C5 examples use explicit `mut`:

```phalcom
mut _count: Int
mut count: Int
mut count via _count
```

Current planning baseline predates that syntax for fields.

P1 rule:

```text
accept the explicit `mut` surface required by the new property/delegation model
preserve legacy unkeyworded mutable-field parsing during P1 unless current authoritative spec says otherwise
avoid whole-language compatibility cleanup in C5
```

A contextual parser treatment is acceptable if it preserves lexical compatibility and does not need a globally reserved keyword.

Do not make an unreviewed language-wide `mut` migration merely because C5 needs the spelling.

---

# 28. Visibility compatibility

C5 property requirements and delegated accessors use the language's ordinary visibility model.

Do not create trait-only `private`/`protected` semantics.

If the live repository still represents visibility through attributes while newer authoritative specs use modifier syntax, implement according to the higher authority and treat the migration as verify-first.

The semantic invariant is:

```text
trait default may access the trait's own private requirement according to trait access rules
private requirement does not become public merely because a conformance supplies a witness
delegate field privacy remains representation privacy
accessor visibility is behavior visibility
```

---

# 29. Diagnostics doctrine

Diagnostics must report semantic source concepts without exposing internal elaboration unnecessarily.

Useful property diagnostic:

```text
`CounterImpl` does not satisfy mutable property requirement `count: Int`
```

with notes/labels for getter/setter obligations as needed.

Do not force users to interpret two unrelated synthetic method errors when one property declaration caused both, although canonical conformance evidence remains getter/setter-based internally.

Required P1 associated diagnostic families include:

```text
duplicate associated declaration
duplicate associated binding
unknown/extra binding
missing binding
binding in inherent impl
invalid RHS/type formation
```

Required `via` diagnostic families include:

```text
delegate field not found
delegate target not a directly owned field
setter delegation requires mutable field
duplicate/conflicting generated getter
duplicate/conflicting generated setter
invalid non-field `via` target
```

P2 adds projection ambiguity/cycle/unresolved diagnostics while preserving semantic proof-state information.

Determinism matters: diagnostics must not depend on hash/source iteration order.

---

# 30. Testing doctrine

Testing is designed broadly but executed selectively.

Use the smallest test that answers the current question.

## 30.1 P1 dimensions

P1 must cover, at minimum:

```text
parser/member taxonomy
property getter/setter elaboration
via getter-only/setter-only/mutable forms
field ownership + mutability legality
member conflict behavior
associated identity across traits
TraitSurface separation
binding LHS resolution
missing/duplicate/extra binding diagnostics
generic source specialization
exact enum-case preservation
unified completeness
incremental add/edit/delete/rename
source navigation identity
ordinary runtime execution of via accessors
C4 witness convergence
```

## 30.2 P2 dimensions

P2 must cover:

```text
Self::Item formation
exact normalization
abstract/symbolic Self
projection in signatures/defaults
ambiguous associated names across traits
recursive/cyclic projection
unknown/blocked/dynamic distinction
generic/exact-case substitution
incremental normalization dependencies
source navigation/hover
```

## 30.3 P3 dimensions

P3 must cover:

```text
Iterable migration
cross-module/package associated declarations/bindings
trait defaults/witnesses using projections
bound callable/reference interactions where affected
compiler/runtime generic reification where affected
high-density valid program
negative diagnostic corpus
cold/incremental equivalence
workspace-scale regression slice relevant to C5
```

Do not automatically include every generic semantic suite after every patch.

---

# 31. Verification execution budget

Use the repository's BUILD / STABILIZE / CERTIFY modes.

## 31.1 BUILD

Run only:

```text
exact new/failing parser test
exact semantic test for the product just edited
directly affected module test when the new product compiles
```

Do not run workspace test/clippy after every task.

## 31.2 STABILIZE

At named gates, run:

```text
phalcom-ast focused syntax tests
phalcom-semantic associated/traits/impl/incremental focused modules
phalcom-core focused language verticals for via/C4 convergence
phalcom-lsp/editor focused tests where changed
cargo check for affected crates
```

Use exact commands from the active patch-grade plan.

## 31.3 CERTIFY

Broad crate/workspace gates belong to P3/final checkpoint certification unless evidence requires earlier broadening.

Known baseline failures must be classified before any C5 blame/fix decision.

---

# 32. Baseline/unrelated failure policy

Classify every observed failure:

```text
A — definitely caused by current C5 patch
B — probably caused by current C5 patch
C — unclear
D — clearly unrelated/baseline
```

Policy:

```text
A/B: active responsibility
C: one bounded classification pass; record/defer if nonblocking and still unclear
D: record/defer immediately
```

Inherited C4 broad blockers include, until reverified:

```text
Universe generic call-entry failure
incremental A7 presentation mismatch
Universe Bool capability failures
Iterable/outgoing-pack generic call-entry failures
formatting drift
AST Clippy violations
```

Do not weaken C5 tests to accommodate them.

---

# 33. STOP / CONSULT triggers

In addition to the shared escalation protocol, C5-specific mandatory triggers include:

1. implementing associated types appears to require changing `ImplId` source identity;
2. `TraitRequirementId` must be generalized to mix behavior and associated types to proceed;
3. a separate conformance completeness authority appears necessary;
4. property requirements appear to require physical storage identity or layout contribution;
5. `via` appears to require runtime protocol/bytecode/dynamic field resolution;
6. directly owned private-field assumptions are contradicted by authoritative live semantics;
7. exact generic or exact enum-case identity cannot be preserved through associated specialization with existing C4 seams;
8. P2 requires a new explicit user-visible associated projection qualification syntax;
9. correct `T::Item` semantics require generic trait assumptions during C5;
10. unresolved projections seem to require collapsing to `Dynamic` for implementation convenience;
11. an LSP/compiler/runtime component would need to independently resolve associated names/conformances;
12. incremental correctness appears to require broad whole-workspace invalidation rather than bounded ownership without a clear architectural reason;
13. a material `mut`/visibility language migration beyond C5's feature needs becomes necessary;
14. current specs, checkpoint decisions, and live code materially disagree about any of the above.

A triggered consultation cannot be self-waived.

---

# 34. Debugging budget

## Mechanical failures

Up to three coherent correction cycles while each cycle has a concrete cause and shows progress.

## Semantic failures

Before editing, write:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure. If it persists, STOP AND CONSULT.

## Architectural failures

Zero speculative architecture attempts. Consult immediately.

Never rerun an unchanged failing test unless something relevant changed.

---

# 35. Checkpoint bookkeeping

At plan start:

```text
mark active plan IN_PROGRESS
record starting revision when useful
confirm predecessor handoff
refresh baseline issue state if evidence changed
```

During a plan, update checkpoint only for:

```text
durable new identity/interface
implemented invariant
consultation/amendment
deferred/baseline issue
coherent verification gate
```

At plan completion:

```text
update plan ledger
record completion + verification truthfully
refresh stable interface map
record actual gates/tests
record residual failures
set next plan/action
```

The plan is not administratively complete until checkpoint state is current.

---

# 36. Walkthrough requirements

Every completed C5 numbered plan should produce:

```text
LANG005.C5.P<n>-walkthrough.md
```

The walkthrough must record:

```text
final plan state
actual architecture implemented
important files/symbols changed
durable interfaces/invariants
deviations from plan
consultations/amendments
tests added
tests actually run
tests deliberately deferred
verification classification
baseline/residual failures
next required work
```

Never claim a gate passed without executed evidence.

---

# 37. Handoff requirements

Every non-terminal C5 numbered plan should produce:

```text
LANG005.C5.P<n>-handoff.md
```

It must give the next implementer:

```text
current reliable revision/state anchor
completed prerequisites
stable interface/takeover map
invariants not to redesign
known drift/deferred failures
must-read files
next plan objective
high-risk areas
first recommended commands/tests
explicit do-not-reexplore guidance
```

P3's final handoff should target C6 and clearly distinguish C5 projection semantics from C6 generic trait-proof semantics.

---

# 38. Things not to redesign during C5

Do not re-open without a consultation-triggering contradiction:

```text
C4 explicit conformance identity/coherence
C4 witness/default selection architecture
C4 exact ConformanceEvidence authority
TraitRequirementId behavior identity
runtime anti-authority boundary
private/non-inherited field model
state/property reconciliation
initial direct-field via semantics
generated-accessor conflict law
local parameter names being signature-irrelevant
C6 ownership of generic trait constraints/conditional conformance
```

Do not use C5 as a venue for unrelated syntax cleanup, visibility redesign, trait-object design, reflection design, supertraits, or generalized delegation.

---

# 39. Compact plan map

```text
C5.P1
    syntax/member taxonomy
    property requirement elaboration
    own-field via elaboration
    AssociatedTypeRequirementId
    TraitSurface associated requirements
    conformance binding plan
    exact associated bindings in ConformanceEvidence
    unified completeness
    fingerprints/replacement/source navigation
    via runtime vertical
        ↓
C5.P2
    associated projection type term
    contextual Self::Item
    normalization
    ambiguity/cycles/terminal states
    signatures/defaults/inference integration
    incremental/tooling projection
        ↓
C5.P3
    Iterable migration
    cross-stack integration
    compiler/runtime reification consequences
    editor/LSP UX
    stress/negative corpus
    cold/incremental certification
        ↓
C6
    generic T: Trait assumptions
    conditional conformance
    nested evidence
    generic-body dispatch/projection through proof
```

Keep this map visible when deciding whether a change belongs in the active plan.

---

# 40. Final guidance invariant

The shortest correctness test for any proposed C5 design is:

```text
Does this make associated-type meaning a canonical semantic consequence of
trait identity + conformance evidence, while leaving representation and runtime
discovery out of the proof?
```

If yes, it is likely aligned with C5.

If the design instead makes storage, compiler heuristics, LSP name lookup, or runtime scanning authoritative, it is outside the accepted architecture.
