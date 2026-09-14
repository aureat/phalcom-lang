---
id: LANG005.C4.GUIDANCE
category: LANG
program: LANG005
checkpoint: LANG005.C4
kind: implementer-guidance
status: ACTIVE_PLANNING_GUIDANCE
requires:
  - LANG005.C4-CHECKPOINT.md
  - LANG005.C4.P1-explicit-conformance-and-coherence-plan.md
planning_baseline_revision: 20ad3f31b39b0fe1b1fdf028df4ac9579fcfd9ad
---

# Implementer Guidance — LANG005.C4 Explicit Trait Conformance, Witness Evidence, and Trait-Evidenced Dispatch

## 0. Purpose

This document is the persistent architectural and execution guidance for all C4 implementation sessions.

Load it before working on:

```text
C4.P1 explicit conformance/coherence
C4.P2 witness/default/evidence
C4.P3 trait-evidenced dispatch/lowering
```

The checkpoint's core semantic sentence is:

> A conformance is a structured proof that one exact target satisfies one exact trait reference. The source `impl` identifies where the proof comes from; witnesses/defaults explain why it is valid; semantic evidence—not runtime scanning—authorizes trait behavior.

Every patch should preserve that sentence.

---

# 1. Current Checkpoint State

Planning is complete enough to begin implementation **once predecessor entry conditions are actually satisfied**.

```text
planning state: READY
implementation state on planning baseline: NOT READY
reason: planning baseline is pre-C3
```

Required entry conditions:

```text
C1 complete
C2 complete
pre-C4 applicability proof-state closure complete
C3 complete
```

Do not infer those from planning documents. Verify the live repository.

A fresh implementer must begin with the P1 T0 takeover procedure.

---

# 2. Canonical Repository Location

Use:

```text
docs/implementation/LANG005/
  LANG005.C4-explicit-conformance-and-evidence/
```

with at least:

```text
LANG005.C4-CHECKPOINT.md
LANG005.C4-GUIDANCE.md
LANG005.C4.P1-explicit-conformance-and-coherence-plan.md
```

Later P2/P3 plans, walkthroughs, and handoffs belong in the same checkpoint directory.

Do not move or rename predecessor checkpoint directories as part of C4.

---

# 3. Required Read Order

Before any C4 production edit, read in this order.

## 3.1 Normative language authority

1. `docs/specs/data/traits.md`
2. `docs/specs/data/impl.md`
3. `docs/specs/data/enums.md`
4. applicable callable/member specifications
5. applicable visibility/access specifications
6. type-system specifications needed by generic substitutions and compatibility

The normative specs define language behavior. Plans do not override them.

## 3.2 Checkpoint state

7. final `LANG005.C3-CHECKPOINT.md`
8. final `LANG005.C3-GUIDANCE.md`
9. final C3 walkthrough/handoff
10. `LANG005.C4-CHECKPOINT.md`
11. `LANG005.C4-GUIDANCE.md`
12. active C4 plan

## 3.3 Production architecture

Inspect the live equivalents of:

```text
trait identity / TraitRef
TraitSurface / TraitRequirementId
impl parsing/AST
ImplId
TypeParameterOwner::Impl
target-resolution machinery
TypeStore / TypeSubstitution / TypeEnvironment
exact enum-case identity
effective inherent surfaces
conditional/specialized inherent lookup
applicability proof-state result
semantic DB/session publication
module semantic shards/fingerprints
source index/LSP semantic targets
compiler call-selection/lowering products
```

Do not trust filenames from an old plan when the live architecture moved.

---

# 4. Authority and Drift Rules

## 4.1 Language semantics

Authority order:

```text
ratified/current normative specification
    >
checkpoint fixed decisions
    >
active implementation plan
    >
historical plans
    >
support examples
```

If a support fixture conflicts with the normative trait/impl specification, the support fixture is not authoritative.

## 4.2 Live code mechanics

For implementation mechanics:

```text
live post-C3 repository
    >
planning-baseline symbol names
```

Preserve semantic roles while adapting names and file placement.

## 4.3 Checkpoint state

The checkpoint record is the authority for:

```text
what C4 owns
what it defers
which plan is active
which gates are complete
which architecture decisions are fixed
```

Update it at durable gates.

## 4.4 Plan execution

The plan is patch-grade authority for:

```text
task order
test budget
scope
acceptance criteria
STOP/CONSULT triggers
```

Do not silently move P2/P3 behavior into P1 because it appears locally convenient.

## 4.5 Walkthrough/handoff

After a plan lands, its walkthrough describes the implementation that actually exists.

Future sessions should prefer the walkthrough over speculative implementation details in the original plan.

---

# 5. Session Bootstrap

At the start of every C4 implementation session:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -1 --oneline
```

Then answer privately:

```text
Which C4 plan/task/gate am I implementing?
What predecessor revision am I on?
Are there unrelated local edits?
Which semantic product is authoritative for this task?
What is the smallest test that can fail for the reason I care about?
```

Never overwrite unrelated worktree changes.

If implementation is being split across sessions, read the latest walkthrough/handoff before the plan.

---

# 6. The Semantic Model to Keep Loaded

Keep this pipeline mentally visible:

```text
trait declaration
    ↓
TraitRef
    ↓
TraitSurface
    ├─ TraitRequirementId
    └─ trait-owned default CallableId

source conformance:
    impl TraitRef for Target
    ↓
ImplId
    ↓
ConformanceContribution
    ↓
authorized workspace candidate
    ↓
coherence-safe exact ConformanceHeadMatch
    ↓
requirement-by-requirement witness/default selection
    ↓
ConformanceEvidence
    ↓
trait-evidenced member lookup
    ↓
call/reference selection
    ↓
lowering/execution
```

The most important non-equalities are:

```text
ImplId != exact conformance relationship

ConformanceHeadMatch != ConformanceEvidence

TraitRequirementId != CallableId

trait default CallableId != selected default relation

conformance-local witness != inherent target member

trait-evidenced availability != inherent surface membership
```

If a patch erases one of these distinctions, stop.

---

# 7. P1 — Conformance Head Semantics

P1 is intentionally narrower than “implement trait conformance.”

P1 answers:

```text
Which conformance declarations exist?
Are they legal where declared?
Which exact target + TraitRef pairs can they match?
Do any source domains overlap?
```

P1 does not answer:

```text
Does the target actually satisfy all requirements?
```

That is P2.

### Correct P1 product

Conceptually:

```text
ConformanceHeadMatch {
    impl_id,
    exact_target,
    exact_trait_ref,
    impl_substitution,
    environment,
}
```

### Incorrect P1 product

Do not build:

```text
bool conforms
```

Do not call a unique head match:

```text
ConformanceEvidence
```

Names shape later architecture. Use names that preserve the phase boundary.

---

# 8. Shared `impl` Syntax, Separate Semantics

Source syntax should be normalized around one implementation declaration family.

Conceptually:

```text
ImplDef {
    generic_parameters,
    kind,
    members,
    source,
}
```

with:

```text
ImplKind::Inherent {
    target
}

ImplKind::Conformance {
    trait_ref,
    target
}
```

The exact AST shape may differ, but the parser must preserve the two type expressions separately.

Never encode:

```phalcom
impl Printable for User
```

as an exotic inherent target expression.

Never lower conformance syntax directly into `InherentImplContribution`.

The shared syntax exists for language regularity.

The separate semantic products exist for correctness.

---

# 9. Identity Policy

## 9.1 `ImplId`

Reuse `ImplId` for source/provenance identity.

One source declaration:

```phalcom
impl<T> Tagged for Value<T> {}
```

has one `ImplId`.

Do not allocate a new source identity for each:

```text
Value<Int>
Value<String>
Value<Bool>
```

application.

## 9.2 Exact relation

The exact relationship is the pair:

```text
exact target + exact TraitRef
```

The source `ImplId` that proves it is provenance attached to that exact relationship.

## 9.3 Avoid premature `ConformanceId`

Do not introduce a standalone opaque `ConformanceId` merely because “conformance feels like an entity.”

First prove that:

```text
ImplId + exact key + evidence
```

is insufficient for a required durable operation.

Potential later reasons might include public reflection or persistent metadata. Those are not P1 assumptions.

---

# 10. Generic Target Identity Is Central

C4 must preserve exact generic target identity.

This is a primary regression:

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

The system must model:

```text
Value<String> + Tagged → source A
Value<Int>    + Tagged → source B
```

not:

```text
DeclarationId(Value) + Tagged → one collapsed relation
```

This test should exist at:

```text
P1 semantic head lookup
P2 witness evidence
P3 runtime execution
```

with progressively stronger assertions.

At P3:

```phalcom
Assert.equal(Value<String>.new(...).tag, "string")
Assert.equal(Value<Int>.new(...).tag, "int")
```

must prove the complete pipeline.

---

# 11. Exact TraitRef Identity Is Equally Central

For:

```phalcom
trait Converter<T> {
  convert -> T
}
```

these are separate exact contracts:

```text
Converter<Int>
Converter<String>
```

A single target may have conformances to both if the rest of member semantics remains coherent.

Do not index only by trait declaration identity when answering exact conformance queries.

A coarse index may use the trait declaration as a bucket key, but exact matching must retain generic arguments.

---

# 12. Nominal Target Policy

Legal direct conformance targets:

```text
class/data nominal type
enum root
exact enum case
generic application of a nominal target
```

Illegal direct targets:

```text
anonymous Tuple
anonymous Record
function type
other structural type expression
```

Do not broaden this because the general type system can represent those types.

The conformance model is nominal and explicit.

---

# 13. Exact Enum Cases

Exact enum-case conformances preserve exact `VariantId` identity.

Do not normalize:

```text
Result<Int>::Ok(_)
```

into merely:

```text
Result<Int>
```

Ownership comes from the enum declaration module.

Witness lookup in P2 may combine:

```text
case-specific behavior
enum-root/inherited applicable behavior where the enum model permits it
C2 exact-case conditional behavior
```

but conformance identity itself remains exact.

---

# 14. Trait-or-Target Ownership

Authorization rule:

```text
source module owns trait
OR
source module owns target
```

Use canonical declaration/module identity.

Do not use:

```text
current import path
alias provenance
re-exporting module
file proximity
runtime load ownership
```

### Required four-way test

For trait module `traits`, target module `models`, third module `app`:

```text
traits declares conformance  → authorized
models declares conformance  → authorized
app declares conformance     → unauthorized
app re-exports both          → still unauthorized
```

Then add the conformance in both legal owner modules.

Both are authorized.

Together they violate coherence.

That proves authorization and coherence are different checks.

---

# 15. Cross-Module Publication

A conformance can be sourced from either ownership side, so a target-owned-only collection is incomplete.

The workspace-level semantic graph must discover conformances regardless of source side.

Prefer a bounded coarse key:

```text
nominal target family
+
trait declaration
```

then exact-match candidate heads.

Do not query by scanning every module or every conformance on each lookup.

Do not make runtime import order contribute semantic membership.

---

# 16. Generic Conformance Heads

C4 supports generic source heads:

```phalcom
impl<T> Tagged for Value<T> {}
```

Implementation-local parameters belong to `ImplId`.

The same parameter environment resolves:

```text
target head
trait reference
later P2 witness signatures
later C5 associated bindings
```

Alpha-renaming:

```phalcom
impl<T> ...
impl<U> ...
```

must not change semantic domain when the mappings are otherwise equivalent.

Generic heads are C4.

Trait-conditioned generic heads are C6.

Do not conflate them.

---

# 17. Coherence Has No Specialization Rule

This is fixed:

```phalcom
impl<T> Tagged for Value<T> {}
impl Tagged for Value<Int> {}
```

is rejected.

Do not choose the second because it “looks more specific.”

C2's target-specialized inherent behavior is not precedent for conformance specialization.

The trait specification explicitly requires coherence, not best-match selection.

If a future language revision introduces specialization, it will require new normative rules.

---

# 18. Overlap Must Be Joint

Overlap is not:

```text
target domain intersects
```

alone.

It is:

```text
target domain intersects
AND
TraitRef domain intersects to the same exact reference
```

Example:

```phalcom
impl<T> Converter<T> for Value<T> {}
impl Converter<Int> for Value<String> {}
```

At `Value<String>` the first means:

```text
Converter<String>
```

so it does not collide with:

```text
Converter<Int>
```

A target-only overlap detector is a correctness bug.

Reuse canonical type/generic unification machinery. Do not compare strings or syntax text.

---

# 19. Applicability Proof State

Before C4, applicability must distinguish semantically different non-success states.

At minimum keep available:

```text
Proven
Disproven
Unknown
Blocked
Dynamic
```

Names may differ.

Only proven facts may construct successful semantic evidence.

Do not silently map:

```text
Unknown → false
Dynamic → true
Blocked → false
```

when later diagnostics/evidence require those distinctions.

P2 in particular must consume C2 conditional witness applicability evidence rather than re-running a parallel solver.

---

# 20. P2 — Witness Resolution Mental Model

For each `TraitRequirementId`, instantiate the requirement under:

```text
Self := exact target
trait generic substitution
impl generic substitution
```

then select a satisfaction source.

Candidate sources:

```text
explicit conformance-local witness
compatible effective concrete target behavior
compatible data component/property capability
trait default
```

The result is a relation:

```text
TraitRequirementId
    →
RequirementSelection
```

not a mutation of either entity.

---

# 21. Conformance-Local Witnesses Are Not Inherent Members

Given:

```phalcom
impl Printable for User {
  toString -> String {
    "User"
  }
}
```

the body is owned by the conformance source context.

It must not appear as if declared in:

```phalcom
class User {
  toString ...
}
```

Consequences:

```text
no DeclarationSurface injection
no target-owned callable identity
no unrelated extension helpers
no inherent conflict resolution by shadowing
```

If a compatible existing inherent callable can satisfy the requirement, select it.

If an explicit conformance body supplies the requirement, model it as conformance-owned behavior.

---

# 22. Conformance Body Membership Rule

A conformance body is not a general extension namespace.

Every behavioral declaration in the body must correspond to a trait requirement that C4 understands.

Therefore:

```phalcom
impl Printable for User {
  toString -> String { ... }   // potentially valid witness

  helper -> String { ... }     // invalid in C4
}
```

must reject `helper`.

C5 later adds associated-type binding declarations as another legal conformance-body category.

Do not permit arbitrary helper behavior “for convenience.”

---

# 23. Witness Compatibility

Compatibility must use the canonical callable/type system.

Check at least:

```text
selector identity
member category
dispatch side
parameter compatibility
result compatibility
Self substitution
trait generic substitution
impl generic substitution
callable generic compatibility
visibility/access coverage
property mutability
index getter/setter role
```

Do not:

```text
compare source text
compare rendered type strings
match only selector and ignore types
treat getter as setter
treat index getter as index setter
```

The trait requirement is the contract.

The witness must cover it.

---

# 24. Data Components as Witnesses

A data component is not a synthetic getter.

If:

```phalcom
data Person(name: String)
```

satisfies:

```phalcom
trait Named {
  name -> String
}
```

the evidence should retain:

```text
DataComponentId(Person.name)
```

as the witness source.

Do not fabricate a target-owned getter `CallableId`.

For mutable capability:

```text
getter requirement  → immutable component may satisfy
setter requirement  → immutable component cannot satisfy
```

This distinction must be explicit in P2 tests.

---

# 25. Inherited Behavior as Witness

A child conformance may use an inherited effective member as its witness.

Example:

```phalcom
trait Printable {
  print -> String
}

class Base {
  print -> String { "base" }
}

class Child is Base {}

impl Printable for Child {}
```

The canonical class dispatch/effective-surface machinery should determine the effective `print`.

Do not independently scan parent declarations inside C4.

If `Child` overrides `print`, the normal effective lookup determines the new witness candidate.

---

# 26. C2 Conditional/Specialized Behavior as Witness

This is a critical C2→C4 integration seam.

If C2 proves a conditional/specialized inherent member applicable for the exact target, P2 may use that selected callable as a witness.

C4 must consume C2's canonical selection/proof product.

Do not re-solve the C2 impl domain.

Do not accept a witness merely because the runtime value might satisfy the condition.

Static semantic evidence is authoritative.

---

# 27. Default Selection

For one requirement:

```text
explicit conformance-local witness
    >
compatible concrete effective witness
    >
trait default
    >
missing requirement diagnostic
```

A concrete witness wins over the trait default.

A default remains trait-owned.

Never copy a selected default into the target.

Never re-typecheck the trait default independently for each conformance. C3 already checked it once against abstract `Self`.

P2 only specializes/selects it.

---

# 28. Complete Conformance Evidence

P2 must produce a structured proof.

Conceptually:

```text
ConformanceEvidence {
    source_impl,
    exact_target,
    exact_trait_ref,
    impl_environment,
    requirement_selections: {
        requirement → witness/default
    }
}
```

This proof object is the stable bridge to P3.

Do not make P3 reconstruct witnesses from source syntax.

Do not make P3 ask a runtime class whether it “implements” a trait.

---

# 29. Parent Conformance Does Not Implicitly Propagate

Keep two concepts separate:

```text
inherited member can witness child conformance
```

versus:

```text
parent conformance automatically implies child conformance
```

C4 adopts the first and not the second.

So:

```phalcom
impl Printable for Base {}
```

does not by itself prove:

```text
Child conforms Printable
```

unless a later normative rule introduces that propagation.

Write a semantic regression for this.

---

# 30. P3 — Trait-Evidenced Ordinary Lookup

P3 adds a second semantic availability layer beside inherent behavior.

Think:

```text
ordinary receiver behavior
    =
inherent/effective candidates
    +
conformance-evidenced trait behavior
```

but do not physically merge the underlying semantic products.

The lookup result should preserve origin/evidence.

This matters for:

```text
reflection
ambiguity
bound references
lowering
source navigation
```

---

# 31. Multiple Traits and Same Selector

A target can conform to multiple traits containing the same selector.

Do not collapse the requirements.

Example:

```text
Named.name
DisplayNamed.name
```

remain distinct `TraitRequirementId`s.

If both select:

```text
Person.name
```

ordinary lookup can converge on one concrete callable.

If both instead supply different defaults and no concrete target witness resolves them, ordinary unqualified lookup is ambiguous.

Never pick by declaration/import order.

---

# 32. Default → Requirement Calls

This is the single most important P3 execution invariant.

A C3 trait default can call an abstract requirement.

At runtime for a concrete conformance, that abstract call must use the current conformance evidence.

Example:

```phalcom
trait Scalable<T> {
  scale(_ value: T) -> T

  twice(_ value: T) -> T {
    self.scale(self.scale(value))
  }
}
```

For a `Doubler : Scalable<Int>` conformance:

```text
twice default
    ↓
requirement Scalable.scale
    ↓
current ConformanceEvidence
    ↓
Doubler scale witness
```

Do not hard-bind the default's `self.scale` to a trait declaration method.

Do not perform runtime trait search.

---

# 33. Bound Callable References

Trait-evidenced behavior must work through ordinary callable references where callable shape permits it.

Example:

```phalcom
const f = &object.transform(_)
```

The reference must retain the same selected witness/default/evidence as the corresponding call.

Do not re-resolve later under a different conformance environment.

Test at least:

```text
inherent witness reference
conformance-local witness reference
trait-default reference
```

---

# 34. Runtime Boundary

Semantic authority lives before runtime.

Permitted implementation strategies after semantic proof include:

```text
direct static lowering
specialized call target
internal witness table
cached conformance evidence
other compiler-selected representation
```

Forbidden semantic strategies:

```text
scan runtime class methods to decide conformance
register conformance dynamically at module init
let import order determine conformance
copy trait defaults into target method tables and call that the semantic model
```

The runtime realizes the semantic selection.

It does not decide it.

---

# 35. Source/LSP Model

Tooling should expose semantic relationships where feasible:

```text
conformance source → trait
conformance source → target
requirement → selected witness
witness → satisfied requirement(s)
selected default
ownership/coherence diagnostic origins
```

Use canonical semantic IDs.

Do not reconstruct relationships from printed type names or AST pattern matching in the LSP layer.

P1 only needs enough relationship projection for conformance heads.

P2/P3 can enrich this with witness/evidence relationships.

---

# 36. Incremental Architecture

Treat these as separate invalidation layers:

```text
conformance source structure/header
candidate publication
coherence result
exact head match
witness compatibility
complete ConformanceEvidence
trait-evidenced lookup
call/reference selection
lowered executable target
```

### Body-only witness edit

Should not change:

```text
P1 conformance head identity
ownership
coherence domain
```

It may change:

```text
body analysis
runtime behavior
```

### Witness signature edit

Must invalidate:

```text
witness compatibility
ConformanceEvidence
trait-evidenced lookup if completeness changes
dependent calls/references
```

### Trait requirement signature edit

Must revalidate every dependent conformance.

### Trait default add/remove

May change:

```text
incomplete ↔ complete conformance
selected default
ordinary trait-evidenced availability
```

### Conformance header edit

Must update:

```text
candidate family
ownership
coherence
exact head matches
all downstream evidence
```

### Deletion

Must remove all source contributions and derived products.

No stale candidate/evidence may survive module replacement.

### Cold parity

Every incremental scenario must have an equivalent cold-analysis result.

---

# 37. Testing Doctrine

C4 testing is not primarily “does the parser accept trait impl syntax?”

It is:

> Can the language define a realistic trait contract, establish explicit conformances through multiple witness mechanisms, call defaults that depend on abstract requirements, and execute the result while preserving all semantic identities and coherence rules?

Use four layers:

```text
A micro semantic
B interaction semantic
C vertical executable
D adversarial/incremental
```

P1 uses A/B/D heavily.

P2 uses A/B/D plus evidence inspection.

P3 must add C strongly.

---

# 38. Generic Specialization Test Family

This family is mandatory.

```text
GS-01  Value<String> and Value<Int> independently match Tagged
GS-02  each selects a distinct source conformance
GS-03  Value<Bool> has no candidate when only String/Int are declared
GS-04  same DeclarationId(Value) does not collapse exact targets
GS-05  exact TraitRef generic arguments are preserved
GS-06  C2 specialized inherent behavior can later serve the matching specialized conformance
GS-07  generic covering conformance + exact conformance overlap is rejected
GS-08  editing one specialized conformance does not corrupt the other
GS-09  cold/incremental results agree
GS-10  P3 execution returns the specialized result for each exact target
```

At P3 the canonical executable version is:

```phalcom
impl Tagged for Value<String> {
  tag -> String { "string" }
}

impl Tagged for Value<Int> {
  tag -> String { "int" }
}

Assert.equal(Value<String>.new("x").tag, "string")
Assert.equal(Value<Int>.new(1).tag, "int")
```

---

# 39. C4 `Iterable` Fixture

The repository's intended `Iterable` trait uses associated types:

```text
type Item
type Cursor
```

That final form is C5.

For C4 use:

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

The important test is not just that it compiles.

A `RangeView`/`Countdown` conformance should deliberately mix witness origins.

Preferred mixed case:

```text
iterate        → existing inherent member
iteratorValue  → conformance-local body
each           → trait default
count          → trait default
contains       → trait default
toList         → trait default that calls another default
```

This one program catches a large fraction of C4 integration failures.

---

# 40. C4 `Iterator<Item>` Fixture

Also use a stateful trait:

```phalcom
trait Iterator<Item> {
  next -> Option<Item>

  nextOr(_ fallback: Item) -> Item { ... }
  countRemaining -> Int { ... }
  drain -> List<Item> { ... }
}
```

Implement it for a mutable `CountdownIterator`.

Test:

```text
two independent instances
repeated next calls
default → witness mutation
drain after partial consumption
fallback after exhaustion
```

This catches runtime/frame/self-binding mistakes that cursor-only `Iterable` may not.

---

# 41. Shared Witness Test

Required:

```phalcom
trait Named {
  name -> String
}

trait DisplayNamed {
  name -> String

  displayName -> String {
    "<\(self.name)>"
  }
}

class Person {
  name -> String { ... }
}

impl Named for Person {}
impl DisplayNamed for Person {}
```

Both requirements remain distinct.

Both may select the same concrete `Person.name`.

Ordinary lookup is unambiguous because selection converges.

---

# 42. Competing Default Test

Required invalid case:

```phalcom
trait Pretty {
  render -> String { "pretty" }
}

trait Debuggable {
  render -> String { "debug" }
}

class Item {}

impl Pretty for Item {}
impl Debuggable for Item {}

Item.new().render
```

The final call must be ambiguous.

Then add an inherent concrete `render`.

Both trait requirements can select that callable, and ordinary dispatch becomes unambiguous.

This pair should remain a permanent regression.

---

# 43. Data Component Test

Required:

```phalcom
trait Named {
  name -> String
}

data Person(name: String)

impl Named for Person {}
```

Evidence should select the `DataComponentId`.

Negative companion:

```text
mutable property requirement
+
immutable component
→ incomplete/incompatible conformance
```

No synthetic getter identity.

---

# 44. Inherited Witness Test

Required:

```text
Base.print
Child inherits Base.print
impl Printable for Child
→ inherited callable selected
```

Then override on `Child`.

The selected witness should become the override through normal effective lookup.

Separately assert:

```text
impl Printable for Base
does not automatically establish Child conformance
```

---

# 45. C2 Conditional Witness Test

Required C2→C4 integration:

```text
specialized/conditional inherent member proven for Box<Int>
    → eligible witness

same member disproven for Box<String>
    → not eligible

generic/unknown proof
    → not silently accepted
```

Use the canonical C2 selection/proof object.

This test should fail if C4 attempts to re-solve applicability independently.

---

# 46. Exact Enum-Case Test

Required:

```text
Result<Int>::Ok(_) conforms HasValue<Int>
Result<Int>::Error(_) does not
Result<Int> root does not automatically acquire case conformance
```

Preserve exact `VariantId`.

Where case GADT specialization exists, test the specialized type environment rather than erasing to the enum root.

---

# 47. Negative Conformance Body Test

Required:

```phalcom
impl Printable for User {
  toString -> String { ... }
  helper -> String { ... }
}
```

Reject the unrelated `helper`.

This is a direct guard against accidentally turning conformance syntax into extension methods.

---

# 48. Cross-Module Ownership/Coherence Test

Use at least three modules:

```text
traits   owns Printable
models   owns User
app      owns neither
```

Test:

```text
traits-side conformance valid
models-side conformance valid
app-side conformance invalid
re-export does not authorize app
traits + models both declaring same relation → coherence error
```

This fixture should be used again in incremental add/delete tests.

---

# 49. Incremental Test Matrix

At minimum:

| Edit | Expected semantic change |
|---|---|
| add conformance | candidate appears |
| delete conformance | candidate/evidence disappears |
| change trait argument | old exact key disappears; new key appears |
| add missing inherent witness | P2 incomplete → complete |
| delete inherent witness | complete → default or incomplete |
| change witness signature | evidence recomputed |
| edit witness body only | P1 head/coherence identity stable |
| add trait default | incomplete → complete |
| remove trait default | complete → incomplete |
| change requirement signature | dependent conformances revalidate |
| move source from trait owner to target owner | remains authorized |
| move source to third module | unauthorized |
| add overlapping conformance | coherence failure |
| delete overlapping conformance | coherence restored |
| change inherited override | selected witness changes |
| change C2 conditional domain | affected witness/evidence changes |

Every meaningful row requires cold/incremental parity.

---

# 50. Compiler/Runtime Differential Tests

Where several witness sources can produce the same observable behavior, run the same operation through each path:

```text
direct inherent witness
inherent-impl witness
conformance-local witness
trait default
```

This catches lowering code that supports only one witness representation.

The user-level output may be identical; internal semantic assertions should still verify the selected origin.

---

# 51. Evidence Inspection Tests

Do not rely only on executable output.

For a mixed conformance, assert the evidence map.

Example:

```text
Iterable<Int, Int>.iterate
    → RangeView.iterate
    kind = inherent callable

Iterable<Int, Int>.iteratorValue
    → conformance-owned callable
    kind = conformance witness

Iterable<Int, Int>.count
    → Iterable.count
    kind = trait default
```

The executable test proves it works.

The semantic test proves it works for the correct reason.

Both are required.

---

# 52. Diagnostics Policy

Use distinct diagnostics for distinct semantic failures.

At least cover:

```text
trait reference unresolved
trait position not a trait
illegal/structural target
unauthorized conformance
duplicate/overlapping conformance
missing witness
incompatible witness
visibility/access mismatch
property mutability mismatch
duplicate explicit witness
extra conformance member
ambiguous trait-evidenced selector/default
unsupported associated binding until C5
unsupported trait-conditioned conformance until C6
unsupported metatype conformance
```

Do not degrade precise trait/conformance failures into generic:

```text
method not found
type mismatch
```

when the semantic system knows the stronger cause.

---

# 53. Fixed Decisions

These are fixed unless the user explicitly changes the language design:

1. Traits are not classes.
2. Conformance is explicit and nominal.
3. `ImplId` is source/provenance identity.
4. Exact target + exact TraitRef identifies the applied relation.
5. Exact target generic arguments matter.
6. Exact TraitRef generic arguments matter.
7. Conformance target must be nominal or exact enum case.
8. Trait-or-target ownership is the authorization rule.
9. Re-export does not transfer ownership.
10. Conformance does not mutate inherent surfaces.
11. Conformance does not alter instance representation.
12. No conformance specialization/most-specific rule.
13. Coherence is global and order-independent.
14. P1 unique head match is not satisfaction evidence.
15. P2 uses structured requirement selections.
16. Conformance-local witness bodies are not target-owned inherent overloads.
17. Trait defaults remain trait-owned.
18. Concrete compatible witness wins over default.
19. Data components retain data-component identity as witnesses.
20. C2 proof/selection products are reused, not re-solved.
21. Multiple traits may share one concrete witness.
22. Competing defaults do not get implicit precedence.
23. Parent conformance does not implicitly propagate to child in C4.
24. Semantic analysis, not runtime class scanning, determines conformance.
25. Associated types are C5.
26. Trait constraints/conditional conformance are C6.
27. Metatype conformance remains reserved.
28. Trait objects/public vtables are later work.
29. `Iterable` C4 tests use trait generic parameters rather than associated types.
30. Complex vertical executable tests are required for C4 closure.

---

# 54. Mechanically Flexible Decisions

The implementer may choose repository-appropriate names/placements for:

```text
ConformanceTarget
ConformanceContribution
ConformanceIndex
family bucket key
head-match result
coherence helper decomposition
incremental query names
diagnostic codes
source-index relationship records
internal witness-selection enum names
internal runtime evidence carrier
```

provided the fixed semantic distinctions remain visible and testable.

The implementer may also factor target identity out of `InherentImplTarget` into a neutral shared nominal-target type if that reduces duplication cleanly.

Do not perform broad unrelated refactors merely to make C4 prettier.

---

# 55. Verify-First Decisions

Before choosing an implementation, inspect live code for:

```text
post-C3 TraitRef storage/identity
post-C3 TraitSurface access API
post-C3 callable-owner generalization, if any
current ImplDef representation
current source fingerprinting
current module ownership APIs
current exact enum-case target resolver
current applicability proof result
current semantic query/database ownership model
current source/LSP target model
current compiler call-selection payload
```

Do not add duplicate abstractions because a planning-baseline symbol moved.

---

# 56. STOP / CONSULT Triggers

Stop and obtain a design decision if implementation proves any of these:

1. C3 landed semantics contradict the fixed C4 trait/conformance model.
2. Exact conformance cannot be represented without changing a ratified type-identity invariant.
3. ownership cannot be determined canonically without changing module identity rules.
4. coherence requires a specialization/precedence rule not present in the spec.
5. associated types are required merely to express ordinary C4 witness compatibility.
6. conformance-local witness identity cannot be represented without collapsing it into target ownership.
7. trait-evidenced ordinary lookup would require merging behavior into `DeclarationSurface`.
8. runtime class scanning appears necessary to execute a proven default/witness.
9. a parent-conformance propagation rule becomes necessary for soundness.
10. a required C4 test can only pass by implementing C5/C6 semantics.
11. the live repository already contains a different partial C4 architecture that would be destructive to overwrite.
12. P2 cannot consume P1 without reopening target/ownership/coherence identity.

Do not self-waive these triggers.

---

# 57. Bounded Debugging Protocol

## Mechanical failure

Examples:

```text
compile error
renamed symbol
module path drift
test helper moved
```

Fix locally and continue.

## Semantic failure

Examples:

```text
Value<Int> and Value<String> collide
re-export grants ownership
generic+exact overlap selects one winner
default replaces concrete witness
data component becomes synthetic getter
```

Reproduce with the smallest semantic test.

Trace the authoritative product backward.

Do not patch the symptom at the caller.

## Architectural contradiction

Examples:

```text
only way to publish conformance is inherent surface mutation
only way to execute default is runtime trait lookup
current C3 identity makes exact TraitRef impossible
```

Stop. Record evidence and consult.

---

# 58. Failure Classification

Classify failures before fixing:

```text
REGRESSION
    C4 patch broke existing behavior

NEW_C4_BUG
    intended new C4 behavior is wrong

PREEXISTING
    reproducible at entry revision

BASELINE_INFRA
    unrelated repository/toolchain failure

SCOPE_CONFLICT
    requires changing a fixed checkpoint decision

SPEC_CONFLICT
    normative documents disagree
```

Do not absorb unrelated failures into C4.

Do not label a new C4 failure “baseline” without reproducing it at the entry revision.

---

# 59. Task Execution Policy

For each task:

1. read task purpose and acceptance;
2. inspect only the needed live files/symbols;
3. write the smallest discriminating test first where practical;
4. implement the minimum architectural change;
5. run focused tests;
6. inspect semantic products, not only final output;
7. update checkpoint notes at durable gates;
8. stop scope creep immediately.

Testing priority:

```text
necessary focused tests
    >
broad confidence runs during BUILD
```

Save broad verification for named gates/certification.

---

# 60. Verification Modes

## BUILD

Smallest relevant tests.

Examples:

```text
parser conformance form
one ownership case
one exact generic target query
one overlap case
one witness source
one default→requirement execution
```

## STABILIZE

Run the affected package/module suites plus direct predecessor regressions.

## CERTIFY

Run the full focused C4 matrix, relevant C1–C3 regressions, cold/incremental parity, complex vertical programs, and repository-level verification required by project convention.

Do not repeatedly run CERTIFY-mode commands while editing one parser branch.

---

# 61. Checkpoint Update Protocol

At each durable gate update:

```text
status/completion
entry revision
landed task/gate
actual symbol/API names
new diagnostics
tests added
coverage IDs proven
known deviations
remaining risks
next task
```

Never mark:

```text
P1 complete
```

when only syntax/ownership landed.

Never mark:

```text
C4 complete
```

at P1 or P2.

---

# 62. Walkthrough Requirement

Every plan requires a walkthrough grounded in the landed code.

P1 walkthrough must cover:

```text
AST/source form
ImplId/generic ownership
target/TraitRef resolution
ownership
publication/index
exact query
coherence
incremental lifecycle
source tooling
Value<String>/Value<Int> specialization
proof of no inherent-surface injection
P2 stable inputs
```

P2 walkthrough must cover:

```text
witness identity
all witness origins
compatibility
default selection
completeness
ConformanceEvidence
diagnostics
incremental evidence invalidation
P3 stable inputs
```

P3 walkthrough must cover:

```text
ordinary trait-evidenced lookup
ambiguity/convergence
bound refs
lowering
default→requirement dispatch
runtime representation
complex executable tests
negative runtime-authority proof
```

---

# 63. Handoff Requirements

## P1 → P2

Must explicitly state:

```text
a unique ConformanceHeadMatch is not ConformanceEvidence
```

and name live APIs for:

```text
candidate lookup
ConformanceContribution by ImplId
head substitution/environment
TraitSurface
source conformance body retrieval
target effective behavior
C2 proof-state API
```

## P2 → P3

Must name live APIs for:

```text
complete ConformanceEvidence
requirement selection
selected concrete callable/data component/default
conformance-local callable
trait/default source identity
ambiguity inputs
```

## P3 → C5/C6

Must document which stable proof/evidence fields C5/C6 may extend without reopening C4.

---

# 64. Scope Discipline for C5/C6 Preparation

C4 should leave extension seams for:

```text
C5 associated bindings
C6 nested conformance evidence
```

but must not pre-implement them.

Good preparation:

```text
evidence structure can later gain associated bindings
candidate/head model retains impl generic environment
TraitRef remains explicit
```

Bad preparation:

```text
implement AssociatedTypeId now
invent GenericConstraint::Conforms now
silently evaluate `where T: Trait` now
```

---

# 65. Fast Mental Checklist Before Every Patch

Ask:

```text
Am I changing source identity, exact relationship identity, or evidence?
Am I accidentally putting conformance behavior into an inherent surface?
Am I preserving exact generic target arguments?
Am I preserving exact TraitRef arguments?
Am I using canonical ownership?
Am I introducing an unstated precedence rule?
Am I re-solving something C2/C3 already owns?
Am I claiming proof before every requirement is checked?
Am I crossing into C5/C6?
What is the smallest test that proves this patch?
```

If the answer to any architectural question is unclear, inspect before editing.

---

# 66. Completion Truth

C4 is complete only when a nontrivial program can demonstrate the entire chain:

```text
trait declaration
→ explicit legal conformance
→ coherence-safe exact relation
→ complete witness/default proof
→ trait-evidenced ordinary behavior
→ default body
→ abstract requirement call
→ selected concrete witness
→ compiler lowering
→ correct VM execution
```

The strongest canonical shape is the generic C4 `Iterable<Item, Cursor>` fixture.

A representative acceptance trace should look like:

```text
RangeView
    conforms Iterable<Int, Int>

iterate requirement
    → inherited/inherent target witness

iteratorValue requirement
    → conformance-local witness

toList
    → trait default

toList calls each
    → trait default

each calls iterate / iteratorValue
    → current ConformanceEvidence
    → selected concrete witnesses

runtime
    → deterministic List<Int> result
```

If the system only passes isolated semantic tests but cannot execute that path, C4 is not complete.

If execution works only because conformance methods/defaults were copied into target method tables and semantic identity was lost, C4 is not complete.

If generic specializations collapse `Value<String>` and `Value<Int>`, C4 is not complete.

If incremental deletion leaves stale conformance/evidence, C4 is not complete.

If competing defaults depend on source order, C4 is not complete.

The final implementation must work **and** preserve the model.

---

# 67. Final Agent Rule

Build C4 around evidence, not convenience.

The stable architecture is:

```text
source declaration
    → candidate relation
    → coherence
    → requirement proof
    → structured evidence
    → dispatch/lowering
```

Never replace it with:

```text
syntax
    → stuff methods onto target
    → hope runtime lookup behaves
```

That shortcut would make simple examples pass while breaking generic identity, coherence, defaults, reflection, incrementality, and future associated types/constraints.

Preserve the proof structure now so C5 and C6 can extend it without rewriting C4.
