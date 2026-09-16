---
id: LANG005.C6.GUIDANCE
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementer-guidance
status: ACTIVE_PLANNING_GUIDANCE
requires:
  - LANG005.C6-CHECKPOINT.md
  - LANG005.C6.P1-canonical-generic-conformance-evidence-and-runtime-transport-plan.md
planning_baseline_revision: 14352f3824e8dcfc9f5b3f46d9f4a4bb5cf48709
prepared: 2026-09-16
repository: aureat/phalcom-lang
---

# Implementer Guidance — LANG005.C6 Generic Conformance Evidence, Associated Constraint Closure, and Production Iteration Migration

## 0. Purpose

This document is the persistent architectural and execution guidance for all `LANG005.C6` implementation sessions.

Load it before working on:

```text
C6.P1 direct generic conformance/evidence
C6.P2 generic associated projections/refinements
C6.P3 conditional/nested conformance proof
C6.P4 Iterator / Iterable generic laws
C6.P5 production iteration migration
C6.P6 closure / stress / certification
```

The checkpoint's core semantic sentence is:

> Generic constraints are canonical semantic propositions. A generic body is checked under lexical proof evidence; a generic application must discharge that evidence; associated meaning is normalized semantically; compiler/runtime layers transport already-selected evidence; and the VM never becomes a conformance or projection solver.

Every C6 patch must preserve that sentence.

This is **guidance**, not a competing checkpoint status record. Live lifecycle truth belongs in `LANG005.C6-CHECKPOINT.md`. Task-level mechanics, exact files, focused tests, and local acceptance belong in the active patch-grade plan.

---

# 1. Current checkpoint state

At guidance creation:

```text
checkpoint: LANG005.C6
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
active plan: none (P1 closed)
latest completed C6 plan: LANG005.C6.P1
```

C5 is complete as `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` and is the predecessor takeover authority.

P1 is complete as an implemented, focused-tested direct generic-conformance vertical. Its walkthrough and handoff are the continuation authority; do not infer later C6 work from P1 completion.

The current intended plan sequence is:

```text
P1 direct generic conformance/evidence
P2 generic associated projection/equality
P3 conditional/nested conformance proof
P4 Iterator/Iterable generic laws
P5 production stateful iteration migration
P6 remaining closure / stress / certification
```

The exact future numbering can be amended by repository evidence. The dependency order must not be bypassed.

---

# 2. Canonical repository location

Use:

```text
docs/implementation/LANG005/LANG005.C6/
```

with:

```text
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md
LANG005.C6.P1-canonical-generic-conformance-evidence-and-runtime-transport-plan.md
```

Later plans, walkthroughs, handoffs, and genuine companion technical records belong in the same checkpoint directory.

Do not reorganize older LANG005 checkpoint directories during C6 work.

---

# 3. Required read order

Before any C6 production edit, read in this order.

## 3.1 Workflow authority

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
docs/workflow/sol-principal-architect-project-doctrine.md   # if present live
```

## 3.2 Normative language authority

Begin with:

```text
docs/specs/README.md
docs/specs/generic-constraints.md
```

Then read the authoritative live equivalents for:

```text
traits/conformance
associated types
generic signatures/type formation
kinds
subtyping/equivalence
callables/selectors/dispatch
exact enum-case identity
runtime/VM evidence execution where specified
iteration, when the owning C6 plan reaches production migration
```

Normative specs define language semantics. Plans define implementation work.

When an old handoff and the normative generic-constraints specification disagree, the normative specification wins.

## 3.3 Checkpoint/predecessor state

Read:

```text
LANG005.C5-CHECKPOINT.md
LANG005.C5-GUIDANCE.md
LANG005.C5.P3-walkthrough.md
LANG005.C5.P3-handoff.md
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md
active C6 plan
latest completed C6 walkthrough/handoff, if any
```

Prefer completed walkthrough/handoff facts over speculative mechanics in older plans while preserving checkpoint invariants.

## 3.4 Production architecture

Inspect the live equivalents of:

```text
GenericSignature
GenericConstraint
TypeTerm
type annotation formation
parameter kinds / KindId
TraitRef
TraitRequirementId
AssociatedTypeRequirementId
TraitSurface
ImplId
ConformanceIndex
ConformanceHeadMatch
ConformanceWitnessPlan
ConformanceAssociatedTypePlan
ConformanceEvidence
ConformanceCompleteness
TraitDispatch* products
TypeData::AssociatedProjection
types::normalize_type
CallSpecialization
semantic DB query/product/fingerprint ownership
source index / editor / LSP targets
stable generic metadata / GenericConstraintRef
semantic-to-core trait invocation lowering
RuntimeTypeEnvironment
RuntimeConformanceEnvironment
RuntimeConformanceEnvironmentRegistry
InvokeTraitRequirement
call-frame environment propagation
production iterator / for-loop lowering only when relevant
```

Do not trust stale filenames after the test/source relocations. Preserve roles, identity, and authority.

---

# 4. Authority and drift rules

## 4.1 Semantic authority order

When sources disagree:

```text
1. explicit user-ratified C6 decisions / accepted checkpoint amendments
2. current authoritative specs under docs/specs/
3. LANG005.C6-CHECKPOINT.md durable invariants/amendments
4. completed C6 walkthroughs/handoffs
5. completed C5 checkpoint/walkthrough/handoff contracts
6. landed production architecture
7. active C6 patch-grade plan for task mechanics
8. historical plans/support fixtures
9. experimental/unstructured documents
```

A historical plan does not outrank current normative semantics.

## 4.2 Live mechanics

For mechanics:

```text
live repository
    >
planning-time helper/file/test names
```

Mechanical drift may be adapted locally.

Architectural drift involving semantic identity, evidence ownership, proof authority, metadata representation, incremental identity, runtime responsibility, slot identity, projection normalization, or iteration law is a STOP/CONSULT condition.

## 4.3 Checkpoint state authority

`LANG005.C6-CHECKPOINT.md` is the single durable authority for:

```text
what C6 owns/defers
active/completed plan state
established invariants/interfaces
verification gates actually passed
deferred/baseline failures
accepted consultations/amendments
current next action
```

Update it at durable gates, not after every edit.

## 4.4 Guidance authority

This file owns persistent C6 architectural/execution rules that should survive individual plan turnover.

Do not use it as a chronological implementation log.

## 4.5 Plan authority

The active patch-grade plan owns:

```text
task dependency order
specific file/symbol expectations
focused BUILD test budget
verification gates
local STOP/CONSULT triggers
acceptance criteria
walkthrough/handoff requirements
```

Do not move P2/P3/P4/P5 work into P1 because it appears convenient.

---

# 5. Session bootstrap

At the start of every C6 implementation session:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then answer privately:

```text
Which C6 plan/task/gate am I implementing?
What completed walkthrough/handoff am I inheriting?
What revision was the active plan prepared against?
Are there unrelated local edits?
Which semantic product is authoritative for this task?
Am I about to duplicate an identity, proof, normalization, cache, or runtime mechanism?
What is the smallest discriminating test?
What exact STOP/CONSULT trigger would apply if the live architecture differs?
```

Preserve unrelated modified/staged/untracked work.

No reset, clean, broad staging, or opportunistic repository normalization.

Because test names were recently cleaned up, rediscover tests by semantic responsibility:

```sh
cargo test ... -- --list
rg '<semantic concept>' <test tree>
```

Do not resurrect historical `p3`, `stageN`, `plan_b`, or checkpoint-coded test names.

---

# 6. Semantic model to keep loaded

Keep this high-level pipeline visible:

```text
source generic declaration
    ↓
GenericSignature
    ├─ parameter identities
    ├─ parameter kinds
    ├─ variance
    ├─ subtype/equivalence propositions
    └─ conformance / associated propositions
            ↓
lexical generic evidence environment
            ↓
proof / normalization
    ├─ assumed generic evidence
    ├─ exact ConformanceEvidence
    ├─ associated projection evidence
    └─ rich terminal outcomes
            ↓
body analysis + generic call specialization
            ↓
explicit evidence recipe
            ↓
semantic-to-core projection
            ↓
RuntimeTypeEnvironment
+
RuntimeConformanceEnvironment
            ↓
VM executes selected requirement handles
```

Important non-equalities:

```text
GenericConstraint != syntax spelling
TraitRef != runtime type
TraitRef identity != TraitRef application proof
generic assumed evidence != ConformanceEvidence
ImplId != exact applied conformance
TraitRequirementId != AssociatedTypeRequirementId
trait requirement != inherent method
structural method coincidence != conformance
associated declaration != associated projection
projection formation != projection normalization
type substitution != conformance evidence
semantic proof != runtime lookup
source order != semantic generic-contract identity
```

If an implementation erases one of these distinctions, stop.

---

# 7. Generic constraint language rules — fixed

## 7.0 Method-based language model (no free functions)

Phalcom has **no source-level free functions**. All generic executable behavior is method-based (instance methods or class-side methods).
Generic constraints on callables belong to methods and compose with `ReceiverSpecialization` and owner-level generic contracts.

## 7.1 Relation spellings

```text
:     kind classification
|=    trait conformance / evidence
<:    subtype relation
==    canonical semantic type equivalence
```

Do not overload one relation as another.

## 7.2 Direct conformance

```phalcom
where T |= Printable
where T |= Converts<A, B>
```

means canonical direct conformance propositions.

The semantic representation must be general:

```text
Conforms(subject, TraitRef)
```

not:

```text
BoundOnGenericParameter(parameter, trait)
```

## 7.3 Same-subject conjunction

```phalcom
where T |= Printable + Hashable
```

means two independent obligations.

No intersection/synthetic trait is created.

Equivalent source order/grouping must have equivalent semantic identity.

## 7.4 Generic parameter syntax

Generic parameter lists introduce parameters:

```phalcom
<T, U, F>
```

They are not the conformance-bound syntax.

Do not introduce Rust-style:

```phalcom
<T: Printable>
```

## 7.5 Kind constraints

Normative where-clause kind syntax is conceptually:

```phalcom
where F: (Type) -> Type
```

P1 should normalize this into existing parameter-kind authority if the live implementation permits an additive path.

Do not create semantic `GenericConstraint::Kind` merely to preserve syntax shape.

Binder-local kind syntax may remain compatibility input until a separate migration removes it.

## 7.6 Associated notation

Behavior:

```text
value.method
```

Associated selection/projection:

```text
Trait::Item
Self::Item
T::Item
<T as Trait>::Item
```

Never use `.` for an associated type.

## 7.7 Associated equality/specialization

General equality:

```phalcom
T::Item == A
```

Associated specialization within a trait application:

```phalcom
T |= Iterator<Item = A>
```

The older standalone spelling `T::Item = A` is superseded.

P1 must not activate these semantics early.

---

# 8. GenericSignature and canonical constraint ownership

`GenericSignature` remains the single canonical generic contract.

It must eventually own, directly or through canonical fields:

```text
parameters
kinds
variance
subtype
equivalence
conformance
associated equality/refinement
```

Do not create:

```text
TraitBoundSignature
ConformanceConstraintSet disconnected from GenericSignature
checker-only generic trait flags
compiler-only bound tables
```

Source provenance may be stored separately for diagnostics/navigation. Semantic identity must be canonical.

For direct conformance:

```text
T |= A + B
T |= B + A
where T |= A, T |= B
```

must normalize to one semantic proposition set.

Duplicate identical obligations are idempotent, not ambiguous.

---

# 9. TraitRef rule — identity versus proof

`TraitRef` is canonical applied-trait identity:

```text
trait declaration
+
canonical ordinary type arguments
```

Identity formation may verify:

```text
trait declaration category
arity
ordinary argument kinds
canonical argument forms
```

It must not be the recursive generic conformance solver.

Trait-application well-formedness belongs to the generic proof/evidence layer.

This separation is required because future trait applications may themselves have conformance obligations.

If a live implementation requires recursive proof inside `TraitRef` identity construction to remain valid, STOP AND CONSULT.

---

# 10. Generic evidence environment rules

C6 must establish and preserve one lexical generic evidence authority.

Conceptually:

```text
GenericEvidenceEnvironment
    ├─ canonical lexical propositions
    ├─ indexed direct conformance assumptions
    ├─ owner/callable provenance
    └─ deterministic executable evidence layout
```

It composes:

```text
owner generic signature
+
callable generic signature
+
legitimate lexical/domain assumptions
```

It must not be an append-only flat vector scanned by every consumer.

Generic assumptions are abstract proof facts.

Never fabricate:

```text
ImplId
ConformanceEvidence
exact source witness identity
```

for:

```phalcom
where T |= Trait
```

The environment must remain usable by later P2/P3 work rather than being a P1-only dispatch shortcut.

---

# 11. Direct conformance proof rules

The proof API must preserve source and terminal state.

Successful evidence sources:

```text
exact — existing C4/C5 ConformanceEvidence
assumed — lexical GenericEvidenceEnvironment
```

Terminal distinctions must include the repository-native equivalents of:

```text
missing/not declared
ambiguous/coherence conflict
incomplete
unknown/underconstrained
blocked
dynamic boundary
cancelled
budget exceeded
internal failure
```

Do not reduce proof to `bool`.

P1 direct proof order:

```text
normalize/kind-check subject
    ↓
look for compatible lexical assumption
    ↓
otherwise, if exact enough, delegate to exact conformance authority
```

P3 may later add conditional/nested recursion to the same proof framework.

`Dynamic` never means static proof.

---

# 12. Generic-body trait dispatch rules

Under:

```phalcom
where T |= Printable
```

a send:

```phalcom
value.print
```

may resolve through the canonical `Printable` trait surface.

It must retain:

```text
conformance obligation identity
TraitRequirementId
instantiated requirement signature
```

It must not:

```text
inject the trait member into an inherent method table
treat method shape as conformance
pick the first bound by source order
fake an exact TraitDispatchSelection with source ImplId
fall back at runtime to class-dictionary proof
```

If multiple independent assumed traits expose the same selector, preserve ordinary trait ambiguity semantics.

Duplicate identical bounds should already have canonicalized away.

---

# 13. Generic call application rules

Direct conformance proof follows ordinary generic inference:

```text
arguments / expected type
    ↓
existing inference
    ↓
existing subtype/equivalence constraints
    ↓
candidate substitution
    ↓
instantiate direct conformance obligations
    ↓
prove exact or assumed
    ↓
publish evidence source in call specialization
```

Do not make direct conformance a unification relation in P1.

A bounded generic caller may forward evidence:

```phalcom
class Renderer {
  inner<T>(_ value: T) -> String
      where T |= Printable
  {
    value.print
  }

  outer<T>(_ value: T) -> String
      where T |= Printable
  {
    self.inner(value)   // inner also requires Printable
  }
}
```

No exact `impl` for abstract `T` is invented.

Failed bound application must not contaminate later independent analysis.

Exact enum-case identity must not widen to the enum root.

---

# 14. Evidence-slot rules

A runtime evidence slot is not identified by `TraitRequirementId` alone.

Example:

```phalcom
class Combiner {
  both<T, U>(_ left: T, _ right: U)
      where
          T |= Printable,
          U |= Printable
  {
      left.print
      right.print
  }
}
```

The two `Printable.print` requirements may require different evidence values.

Canonical slot identity is therefore:

```text
GenericConformanceKey
+
TraitRequirementId
```

Conceptually:

```text
(Conforms(T, Printable), Printable.print)
(Conforms(U, Printable), Printable.print)
```

Slot layout must be:

```text
deterministic
source-order independent
conjunction-spelling independent
duplicate-insensitive
capacity checked
```

Associated type requirements do not consume executable behavioral slots.

---

# 15. Compiler/runtime boundary rules

The semantic layer must decide:

```text
which conformance obligation applies
which proof source satisfies it
which trait requirement is invoked
which exact witness/default is selected
which caller evidence is forwarded
```

Core/compiler may translate those decisions into executable recipes.

The VM may:

```text
construct/copy evidence slot handles
intern RuntimeConformanceEnvironment
attach environment IDs to call frames
invoke a slot
```

The VM may not:

```text
search conformance declarations
inspect generic where-clause syntax
construct semantic TraitRef identity
normalize associated projections
choose among trait candidates
decide ambiguity
```

Keep runtime type substitution and conformance evidence separate.

Prefer the existing:

```text
RuntimeConformanceEnvironment
InvokeTraitRequirement
```

mechanism.

A new generic-bound opcode or runtime proof table is a STOP/CONSULT condition unless a later accepted plan explicitly ratifies it.

---

# 16. Stable metadata and reflection rules

Public generic signatures must preserve their semantic contracts across modules.

Direct conformance metadata should contain:

```text
stable subject type node
stable trait declaration identity
stable ordinary trait type arguments
```

Never persist:

```text
semantic TypeId
semantic TraitRef
InferVarId
ConformanceEvidence
RuntimeConformanceEnvironmentId
snapshot-local caches
```

Reflection must represent conformance as conformance.

Do not shoehorn:

```text
subject |= Trait
```

into a fake binary type/type left/right relation.

P2 must explicitly solve stable representation for generic associated projection/equality constraints. Do not preempt it by exporting raw C5 executable projection forms incorrectly.

---

# 17. Incremental identity rules

Semantic product identity must follow canonical meaning.

Source presentation changes such as:

```phalcom
where T |= A + B
```

to:

```phalcom
where T |= B + A
```

may alter source/input fingerprints but must not alter the canonical semantic contract or deterministic evidence layout.

Adding/removing/changing a real bound must invalidate:

```text
owning GenericSignature
body evidence consumers
generic applications
public metadata/interface products
source-index/tooling products
```

without broadly invalidating unrelated traits/conformances.

Cold and incremental results must converge.

---

# 18. Associated projection guidance for P2+

C5's authority remains:

```text
TypeData::AssociatedProjection
types::normalize_type
```

P2 extends that authority into generic evidence contexts.

Expected generic forms:

```phalcom
T::Item
<T as Iterator>::Item
T |= Iterator<Item = A>
T::Item == A
```

Contextual `T::Item` may be accepted only if current evidence identifies exactly one associated declaration `Item`.

If multiple bound traits expose `Item`, require qualified projection rather than source-order selection.

Associated specialization/equality may feed type inference, so P2 may need a bounded worklist/fixed-point interaction. P1 must not implement that feedback early.

Never create a second recursive projection normalizer.

---

# 19. Conditional conformance guidance for P3

Conditional conformance example:

```phalcom
impl<T> Printable for List<T>
    where T |= Printable
```

Proof shape:

```text
goal List<User> |= Printable
    ↓
candidate conditional conformance
    ↓
head substitution T = User
    ↓
nested goal User |= Printable
    ↓
proof
```

This requires:

```text
nested obligations
recursive proof state
cycle detection
memoization
proof budgets
conditional candidate coherence
```

It must extend P1's proof model.

Do not activate conditional conformances with a one-level special case.

### Overlap

Until an authoritative conformance/coherence specification says otherwise, preserve current no-overlap semantics.

Do not invent "conditions make overlap legal" or "most specific wins".

---

# 20. Iterator / Iterable guidance for P4–P5

C5's concrete/exact reference model is ratified:

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
  iterator { self }
}

trait Iterable {
  type Item
  type Iterator
  iterator -> Self::Iterator
}
```

Required eventual generic relationships include the equivalent of:

```text
Self::Iterator |= Iterator
Self::Iterator::Item == Self::Item
```

The theorem:

```text
Iterator ⇒ Iterable
```

is not produced merely by writing constraints.

P4 must establish a first-class language mechanism.

Forbidden:

```text
compiler special case
VM special case
selector heuristic
implicit structural theorem
```

Only after the law is semantically authoritative may P5 migrate:

```text
List / Set / Map / Tuple / Bytes / Range
for checking
for lowering
Map entry iteration
RangeIterator
lazy filter/map/take
runtime Iterable superclass
iterate(_)
iteratorValue(_)
```

Production `for` remains legacy until its owning plan lands.

---

# 21. Performance/resource rules

## 21.1 Evidence lookup

Direct assumed evidence should be indexed by canonical conformance key.

Avoid a full generic-constraint scan for every member send.

## 21.2 Proof memoization

Reuse existing exact evidence/query memoization before adding another cache.

P3 may add recursive proof memoization when nested obligations exist.

## 21.3 Runtime environment interning

Use `RuntimeConformanceEnvironmentRegistry::intern`.

Repeated identical evidence invocations must not create persistent environment growth proportional to call count.

## 21.4 Rooting

Evidence-transported method handles must remain rooted through the existing environment registry.

Do not create an unrooted side store.

## 21.5 Optimization discipline

Do not prematurely optimize recursive proof or iterator execution before correctness and actual hot paths are established.

---

# 22. Testing discipline

Testing is evidence gathering.

Use the smallest command that proves the current invariant.

## 22.1 BUILD mode

Default during implementation:

```text
exact new regression
exact affected feature module
no automatic broadening after PASS
```

## 22.2 STABILIZE mode

At coherent gates:

```text
owning semantic feature module
owning core runtime feature module
specific metadata/incremental/tooling modules affected
```

## 22.3 CERTIFY mode

Workspace/release-level commands only when a named checkpoint/release gate requires them.

Do not run during ordinary BUILD:

```text
cargo test --workspace
cargo clippy --workspace
full language corpus
all semantic integration
all core tests
all LSP tests
release builds
```

unless evidence demands escalation.

## 22.4 Test naming

Use descriptive semantic names.

Do not create historical lifecycle-coded names such as:

```text
p3_case
stage4
plan_b
checkpoint_test
```

## 22.5 Filter validity

A command that selects zero tests proves nothing.

Use `-- --list` when necessary.

## 22.6 Failure classification

```text
A — definitely caused by active C6 work
B — probably caused
C — unclear
D — unrelated/inherited baseline
```

A/B are active responsibility.

C receives one bounded classification pass.

D is recorded and not repaired opportunistically.

---

# 23. Inherited baseline handling

### C5-BL-01 / C5-BL-04

Legacy generic call-entry / Range / Option.match failures.

During P1:

```text
new direct bounded-generic call reproduces same root defect
    → P1 blocker

otherwise
    → preserve inherited baseline
```

### C5-BL-07

Universe Bool dependency/capability failures.

Default:

```text
classification D
```

Do not spend C6 implementation budget on them without direct coupling evidence.

---

# 24. STOP / CONSULT rules

Consult rather than improvising when any of the following occurs.

## 24.1 Architecture authority

- another canonical generic constraint/evidence owner already exists live;
- `GenericSignature` would have to be replaced rather than extended;
- assumed evidence appears to require fake exact conformance identity;
- trait identity and generic proof cannot be separated;
- generic associated projection requires a second recursive normalizer;
- compiler/runtime would need semantic proof/search;
- stable metadata would require snapshot-local IDs;
- runtime evidence would require semantic `TraitRef`/`TypeId`;
- evidence slot identity would depend on source order;
- same trait on two subjects cannot be represented without aliasing;
- GC/rooting ownership must change;
- owner-level evidence requires per-object conformance arrays;
- where-kind syntax requires a broad kind-system redesign;
- reflection requires an unplanned breaking public API migration;
- P1 direct bounds require P2 associated-inference feedback;
- P1 direct bounds require P3 conditional recursion;
- passing tests requires weakening C4/C5 exact evidence or ambiguity semantics.

## 24.2 Iterator architecture

- implementing `Iterator ⇒ Iterable` appears to require a compiler special case;
- blanket conformance needs a new target-head model not yet ratified;
- production iteration migration becomes necessary before generic evidence laws exist;
- legacy cursor removal would break still-authoritative production semantics before replacement is complete.

## 24.3 Debugging

For a nontrivial semantic failure:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same semantic problem.

If it persists, consult.

Mechanical failures may receive up to roughly three coherent correction cycles while each has a concrete cause and makes progress.

Architectural failures receive zero speculative redesign attempts.

---

# 25. Consultation packet

When escalating, collect only what the adviser needs:

```text
checkpoint / plan / task
fixed invariant
expected architecture
live revision
minimal reproducer
relevant symbols / call path
scoped diff
observed result
root-cause hypothesis
alternative hypothesis if material
serious correction attempted
exact STOP/CONSULT trigger
decision requested
```

Ask a decision question.

Do not ask the adviser to rediscover the entire repository.

---

# 26. Plan-transition rules

At the end of each C6 plan:

1. write a walkthrough containing what actually landed and what was actually verified;
2. update `LANG005.C6-CHECKPOINT.md` at durable state boundaries;
3. write a handoff containing only the next plan's necessary takeover context;
4. freeze newly established canonical interfaces in checkpoint/guidance as appropriate;
5. preserve deferred work in its owning later plan;
6. do not start the next plan's implementation while closing the current plan.

Completed walkthrough/handoff facts supersede speculative mechanics from the original plan.

---

# 27. P1-specific guidance summary

Until P1 completes, keep these decisions loaded:

```text
GenericConstraint gains direct Conforms(subject, TraitRef)
|= is the canonical conformance relation
+ expands to independent same-subject obligations
where-kind syntax feeds existing parameter-kind authority
TraitRef identity is separated from application proof
one lexical GenericEvidenceEnvironment owns assumptions
assumed evidence is not ConformanceEvidence
generic trait behavior resolves via TraitRequirementId
generic call discharge occurs after ordinary inference
call specialization records exact vs forwarded evidence
slot key = conformance obligation + TraitRequirementId
RuntimeTypeEnvironment and RuntimeConformanceEnvironment stay separate
InvokeTraitRequirement is reused if semantically compatible
stable metadata gains direct conformance relation
semantic identity ignores bound order/regrouping
LSP associated notation uses ::
```

P1 must not implement:

```text
generic T::Item
associated specialization/equality
conditional conformance
recursive proof
Iterator ⇒ Iterable
production iteration migration
```

---

# 28. Later-plan design guardrails

## P2

Must consume P1 evidence/proof and C5 normalization.

Must not create another evidence environment or projection normalizer.

## P3

Must extend proof outcomes/context with recursion/memoization.

Must not introduce condition-based overlap specialization without normative authority.

## P4

Must choose/implement a first-class generic law for Iterator/Iterable.

Must not use compiler/VM special cases.

## P5

Must migrate production behavior only after P4 laws are executable.

Must remove legacy cursor/runtime-superclass pieces only after replacement parity exists.

## P6

Must certify actual remaining C6 obligations, not become a catch-all for unrelated repository cleanup.

---

# 29. Checkpoint bookkeeping discipline

Update `LANG005.C6-CHECKPOINT.md` only for durable events such as:

```text
plan starts/completes
canonical identity/proof interface established
runtime evidence gate passes
stable metadata contract established
accepted consultation/amendment
meaningful baseline reclassification
named verification gate passes
checkpoint completion state changes
```

Do not update it for each test invocation.

This guidance file should change only when a persistent C6 rule or authority changes.

---

# 30. Completion truthfulness

Never conflate:

```text
plan written
implementation exists
focused tests pass
broad crate tests pass
workspace/release certified
checkpoint complete
```

A normal individual plan can finish:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

while C6 remains `IN_PROGRESS / PARTIAL`.

Only actual final C6 acceptance evidence can close the checkpoint.

---

# 31. Current action

The current action is:

```text
execute LANG005.C6.P1
```

using:

```text
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md
LANG005.C6.P1-canonical-generic-conformance-evidence-and-runtime-transport-plan.md
```

as the durable state, persistent guidance, and patch-grade execution contract respectively.

Re-ground the live repository before the first production edit.
