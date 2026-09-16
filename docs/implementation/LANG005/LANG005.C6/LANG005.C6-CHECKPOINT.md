---
id: LANG005.C6
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
requires:
  - LANG005.C5
active_plan: LANG005.C6.P4
latest_completed_plan: LANG005.C6.P3
next_plan: LANG005.C6.P4
next_checkpoint: null
planning_baseline_revision: 14352f3824e8dcfc9f5b3f46d9f4a4bb5cf48709
prepared: 2026-09-16
repository: aureat/phalcom-lang
---

# Checkpoint Record — LANG005.C6 Generic Conformance Evidence, Associated Constraint Closure, and Production Iteration Migration

## 1. Objective and ownership boundary

`LANG005.C6` extends the trait/conformance architecture established by `LANG005.C3`–`C5` from **exact, concrete source conformances** into a complete **generic proof-bearing constraint and evidence system**. It then uses that system to express and execute the generic relationships required by Phalcom's stateful `Iterator` / `Iterable` model and, only after those relationships are semantically authoritative, migrates production iteration away from the legacy cursor protocol.

The checkpoint's core semantic sentence is:

> Generic constraints are canonical semantic propositions, not syntax-local hints. A generic body is checked under lexical proof assumptions; a generic application must discharge those assumptions from exact or enclosing evidence; associated projections and refinements are normalized through that proof environment; the compiler transports already-selected executable evidence; and the VM executes evidence without performing conformance search or type proof.

The checkpoint's full intended authority flow is:

```text
generic declaration / callable
    ↓
canonical GenericSignature
    ├─ kind facts
    ├─ subtype/equivalence facts
    └─ conformance obligations
            ↓
lexical generic evidence environment
            ↓
proof-bearing semantic analysis
    ├─ direct assumed evidence
    ├─ exact C4/C5 evidence
    ├─ associated projection/equality
    ├─ nested/conditional obligations
    └─ proof terminal states
            ↓
generic application specialization
    ├─ canonical type substitution
    └─ canonical evidence recipe
            ↓
semantic-to-core lowering
    ├─ RuntimeTypeEnvironment recipe
    └─ RuntimeConformanceEnvironment recipe
            ↓
VM execution of already-selected evidence
            ↓
Iterator / Iterable generic laws
            ↓
production for / collection / range / lazy iterator migration
            ↓
legacy cursor and runtime Iterable-superclass retirement
```

C6 owns:

```text
direct generic conformance relation `|=`
same-subject conjunction `T |= A + B`
canonical direct conformance propositions in GenericSignature
lexical generic evidence composition
exact-versus-assumed proof-source distinction
generic-body trait dispatch through assumed evidence
generic-call evidence discharge and forwarding
generic associated projections on constrained subjects
qualified associated projections where required for disambiguation
associated specialization in trait bounds
associated equality constraints
associated-evidence interaction with generic inference
stable public metadata for generic conformance/associated constraints
conditional conformances and nested proof obligations
recursive proof state / cycle handling / memoization
coherence rules for conditional conformances as authorized by normative specs
generic Iterator / Iterable laws and evidence relationships
canonical Universe Iterator / Iterable migration
production for-loop evidence-driven checking/lowering
stateful collection, Map, Range, and lazy iterator migration
legacy cursor protocol retirement
runtime Iterable-superclass retirement
cross-module / incremental / reflection / source-index / LSP integration
C6-focused performance, stress, and certification
```

C6 explicitly does **not** own:

```text
redesign of C4/C5 exact conformance identity
replacement of ConformanceEvidence as exact proof authority
runtime conformance search
runtime associated-projection solving
trait-object / existential representation unless separately ratified
supertrait syntax/semantics unless separately ratified
conformance specialization / "most specific wins" unless separately ratified
ad-hoc compiler special cases for Iterator ⇒ Iterable
trait-owned or conformance-introduced storage
a second generic signature / trait-bound system beside GenericSignature
Rust-style `T: Trait` conformance syntax
broad unrelated collection/runtime rewrites before evidence semantics are ready
```

Those boundaries are architectural, not scheduling suggestions.

---

## 2. Current lifecycle state

Current checkpoint state at creation:

```yaml
checkpoint: LANG005.C6
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
active_plan: null
latest_completed_plan: LANG005.C6.P1
next_plan: LANG005.C6.P2
```

`LANG005.C6.P1` is implemented and focused-tested. C6 remains partial because later P2–P6 plans own generic associated projections, recursive conditional proof, iterator laws, production migration, and closure certification.

The active P1 objective is narrowly foundational:

> Establish one complete direct generic-conformance evidence vertical: `where Subject |= TraitApplication`, including same-subject `+` conjunction, becomes a canonical generic constraint; those constraints become lexical assumed evidence in generic bodies; generic trait behavior resolves through that evidence; generic applications discharge direct bounds from exact C4/C5 evidence or compatible enclosing assumed evidence; compiler/runtime lowering transports deterministic evidence slots through the existing runtime conformance-environment mechanism; and public/cross-module generic signatures preserve those bounds as stable semantic metadata.

P1 explicitly does **not** own generic associated projection semantics, associated specialization/equality, conditional conformance recursion, the general `Iterator ⇒ Iterable` theorem, or production iteration migration. Those remain later C6 work.

P1 focused gates have run. The broad recomputation target retains one unrelated Universe cold/incremental presentation-parity baseline; it does not invalidate the direct generic-conformance evidence path.

---

## 3. Repository planning baseline

C6.P1 planning was grounded against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   14352f3824e8dcfc9f5b3f46d9f4a4bb5cf48709
prepared:   2026-09-16
```

Relevant recent repository movement at planning time:

```text
14352f3824e8dcfc9f5b3f46d9f4a4bb5cf48709
docs(work): record test relocation and modular audit probes work log

ed9272bddb2dcb043bc24457a142657bab13fe71
refactor(tests): relocate src test modules to tests, organize modular audit probes, and move audit fixtures

8c389d7a59dddf51fff93efb1bebe2732bd0881b
refactor(tests): rename historical checkpoint and stage test files to descriptive names

b6cf027ea5332c7e18a3ec390eef2e107390b773
feat(lang005): complete C5 associated-type vertical integration, iterator substrate, and certification (P3)
```

Planning verified these live facts:

1. `GenericConstraint` currently has only `Subtype` and `Equivalent`.
2. `GenericSignature` is the canonical owner of generic parameters, kinds, variances, and generic constraints.
3. `GenericConstraintSyntax` currently lacks a conformance relation.
4. the lexer has `Token::Pipe` but no dedicated `|=` token.
5. `CheckingContext` currently carries a flat `ambient_constraints` vector rather than a complete lexical generic evidence environment.
6. `TraitRef::form` currently mixes canonical applied-trait identity formation with referenced-trait generic constraint checking.
7. `CallSpecialization` currently carries type specialization but not generic conformance evidence sources.
8. exact trait dispatch products are exact-evidence oriented and must not be repurposed by fabricating `ImplId` / `ConformanceEvidence` for generic assumptions.
9. stable `GenericConstraintRef` currently represents only subtype/equivalence.
10. `RuntimeConformanceEnvironment` already provides an interned positional handle array and is explicitly not a semantic conformance lookup table.
11. call frames already carry separate runtime type and conformance environment IDs.
12. `InvokeTraitRequirement` already executes a preselected trait requirement slot.
13. semantic and core test trees were recently relocated/renamed and must be rediscovered by semantic responsibility rather than historical P3/stage names.
14. the LSP associated-type hover still uses `.` presentation where canonical associated notation requires `::`.
15. no `LANG005.C6/` implementation directory existed at the planning revision.

Every implementation session must re-ground the live revision before relying on these mechanical facts.

---

## 4. Predecessor takeover state

### 4.1 C5 lifecycle state

`LANG005.C5` is closed as:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
latest_completed_plan: LANG005.C5.P3
```

C6 must inherit C5 rather than rediscover or redesign it.

### 4.2 Frozen C4/C5 semantic interfaces

C6 inherits these canonical products and identities:

```text
ImplId
TraitRef
TraitRequirementId
AssociatedTypeRequirementId
TraitSurface
ConformanceIndex
ConformanceHeadMatch
ConformanceWitnessPlan
ConformanceAssociatedTypePlan
ConformanceEvidence
ConformanceCompleteness
TraitDispatchContribution
TraitDispatchIndex
TraitDispatchSelection
TraitDispatchSite
TraitDispatchTerminal
TypeData::AssociatedProjection
types::normalize_type
SemanticTargetId::AssociatedType
RuntimeTypeEnvironment
RuntimeConformanceEnvironment
RuntimeConformanceEnvironmentId
```

Their meanings remain fixed unless an explicit principal-architect amendment says otherwise.

### 4.3 Frozen C5 associated-type architecture

C5 established:

```text
associated type declaration
    ↓
AssociatedTypeRequirementId
    ↓
TraitSurface.associated_types
    ↓
source conformance associated binding plan
    ↓
exact ConformanceEvidence.associated_types
    ↓
TypeData::AssociatedProjection
    ↓
types::normalize_type
    ↓
already-normalized compiler/runtime consumption
```

C6 may extend projection formation/normalization to generic evidence contexts. It must not introduce another recursive projection normalizer.

### 4.4 Frozen exact conformance distinction

C6 must preserve:

```text
ImplId != exact applied conformance
TraitRef != runtime type identity
ConformanceHeadMatch != ConformanceEvidence
source conformance plan != exact proof
generic assumed evidence != ConformanceEvidence
trait default declaration != selected default relation
conformance-local witness != inherent target method
semantic proof != runtime execution
```

### 4.5 Concrete iterator reference model inherited from C5

C5 proved the following concrete/exact reference model for future C6 migration:

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

Verified laws:

```text
iterator and next are getter-shaped
reusable Iterable.iterator creates fresh traversal state
Iterator.iterator returns self
next returns Option<Item>
None means clean exhaustion only
exhaustion is stable
errors propagate normally
Map target Item is (key: K, value: V)
no public Cursor/hasNext/reset iterator model
```

This is a reference model, not yet the production iteration implementation.

### 4.6 Production iteration takeover

Production iteration remains legacy/cursor-based at C6 start.

C6 must not migrate production `for`, collections, Range, Map, lazy adapters, runtime `Iterable`, or cursor protocol until the required generic evidence/associated relationships exist.

---

## 5. Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome / dependency |
|---|---|---|---|---|---|
| `LANG005.C6.P1` | Direct generic conformance propositions, lexical assumed evidence, generic body dispatch, exact/forwarded call discharge, runtime evidence transport, stable metadata/tooling | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Direct-bound vertical is green; one unrelated recomputation baseline retained |
| `LANG005.C6.P2` | Generic/qualified associated projections, associated specialization/equality, projection-driven inference, durable associated constraint metadata | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Tasks T1–T10 complete; generic associated projections, refinements, equality, and inference verified; G1–G5 pass |
| `LANG005.C6.P3` | Conditional conformances, nested recursive obligations, cycle/memoization proof state, conditional coherence | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Tasks T0–T8 complete; recursive proof engine, cycle/memoization, derived runtime evidence, and cross-module/incremental integration verified; Gates G1–G6 pass |
| `LANG005.C6.P4` | Iterator/Iterable generic laws, canonical Universe migration, explicit blanket/derived implication decision | PROPOSED | NOT_STARTED | UNVERIFIED | Depends on recursive proof/equality authority |
| `LANG005.C6.P5` | Production `for`, collections, Map, Range, lazy stateful iterators, cursor/runtime-superclass retirement | PROPOSED | NOT_STARTED | UNVERIFIED | Depends on P4 semantic laws |
| `LANG005.C6.P6` | Remaining generic-constraint closure, higher-kind integration where specified, alias/GADT/tooling/performance/stress/certification | PROPOSED | NOT_STARTED | UNVERIFIED | C6 closure plan; exact scope may be amended |

The precise numbering/content of P2–P6 may be amended by later live repository analysis. The dependency structure is more authoritative than the placeholder plan names.

---

## 6. Established and ratified C6 invariants

These are accepted checkpoint architecture even before implementation.

### 6.0 Method-based language model (no free functions)

Phalcom has **no source-level free functions**. All generic executable behavior is method-based (instance methods or class-side methods). Generic callable constraints belong to methods and compose with `ReceiverSpecialization` and owner-level generic contracts.

### 6.1 Canonical generic-contract authority

`GenericSignature` remains the single semantic owner of:

```text
generic parameters
parameter kinds
variance
subtype constraints
equivalence constraints
direct conformance constraints
future associated constraints
```

C6 must not create a parallel "trait-bound signature" system.

### 6.2 Direct conformance relation

The canonical semantic shape is:

```text
Conforms(subject, TraitRef)
```

represented in the existing generic-constraint family.

The subject is a general type-level term, not only a bare generic parameter.

### 6.3 Conformance syntax

Canonical direct conformance syntax is:

```phalcom
where T |= Printable
where T |= Printable + Hashable
where T |= Converts<A, B>
```

`+` is same-subject conjunction sugar only.

Phalcom does not adopt Rust-style:

```phalcom
<T: Printable>
```

as conformance syntax.

### 6.4 Kind, conformance, subtype, and equivalence are distinct

```text
:     kind
|=    conformance/evidence
<:    subtype
==    canonical type equivalence
```

No relation is encoded as another for implementation convenience.

### 6.5 Associated syntax

Canonical associated notation uses `::`:

```text
Trait::Item
Self::Item
T::Item
<T as Trait>::Item
```

Behavioral member send uses `.`.

Standalone equality uses:

```phalcom
T::Item == A
```

Single `=` is reserved for associated specialization inside a trait application:

```phalcom
T |= Iterator<Item = A>
```

### 6.6 Generic assumptions are evidence, not exact conformances

A lexical assumption such as:

```phalcom
where T |= Printable
```

must never be represented by a fabricated:

```text
ImplId
ConformanceEvidence
source conformance declaration
exact evidence fingerprint
```

Exact and assumed proof sources remain distinct.

### 6.7 TraitRef identity and proof are separate

`TraitRef` is canonical applied-trait identity:

```text
trait declaration + canonical ordinary type arguments
```

Whether that trait application is well-formed under its own generic constraints belongs to generic proof/evidence semantics, not to identity construction.

### 6.8 Structural method coincidence is not conformance

A type that happens to expose a matching selector does not satisfy `|=`.

### 6.9 Generic evidence composes lexically

Owner and callable generic constraints compose into one lexical evidence environment.

A bounded generic caller may satisfy a bounded generic callee from compatible assumed evidence.

### 6.10 Direct conformance discharge follows ordinary inference

P1 staging is:

```text
ordinary argument / expected-type inference
    ↓
existing subtype/equivalence solving
    ↓
candidate substitution
    ↓
direct conformance obligation instantiation
    ↓
exact or assumed proof
```

Direct conformance is not initially a unification relation.

### 6.11 Dynamic is not static proof

`Dynamic` must never silently satisfy a static conformance obligation.

### 6.12 Semantic proof / runtime execution boundary

Semantic analysis owns:

```text
constraint meaning
proof
ambiguity
associated normalization
trait requirement identity
evidence source
```

Compiler/runtime own:

```text
lowering
method-handle preparation
environment construction
slot transport
execution
```

The VM does not search for conformances.

### 6.13 Runtime type and conformance state remain separate

```text
RuntimeTypeEnvironment
    canonical type substitutions

RuntimeConformanceEnvironment
    executable trait requirement handles
```

Do not merge them.

### 6.14 Generic evidence slot identity is complete

A generic executable evidence slot must be identified by:

```text
canonical conformance obligation
+
TraitRequirementId
```

A `TraitRequirementId` alone is insufficient because the same trait can be assumed for multiple subjects in one lexical body.

### 6.15 Slot layout is semantic and deterministic

Evidence-slot numbering must be independent of:

```text
source constraint order
`+` versus comma spelling
duplicate identical bounds
```

### 6.16 Stable generic metadata uses stable identities only

Stable public constraints may contain:

```text
stable subject type node
stable trait declaration identity
stable trait ordinary type arguments
```

They must not contain snapshot-local semantic IDs, proof objects, inference variables, or runtime environment IDs.

### 6.17 Incremental semantic identity ignores irrelevant source presentation

Semantically equivalent direct bound reordering/regrouping must not become a different public semantic contract.

Source/input fingerprints may still change because text changed.

### 6.18 C5 normalizer remains singular

Future generic projection support must extend/consume `types::normalize_type`; it must not create a second recursive associated-projection authority.

### 6.19 Iterator migration is evidence-driven

The production iterator migration may begin only after the generic semantic laws needed to express it are implemented.

No compiler-only `Iterator ⇒ Iterable` shortcut is allowed.

---

## 7. Planned P1 takeover interfaces

P1 is expected to establish the following durable seams for later C6 plans.

Names may adapt mechanically; ownership and information content are fixed.

### 7.1 Generic conformance key

Conceptually:

```rust
GenericConformanceKey {
    subject,
    trait_ref,
}
```

### 7.2 Generic evidence environment

One lexical authority, conceptually:

```text
GenericEvidenceEnvironment
    ├─ canonical visible generic constraints
    ├─ indexed direct conformance assumptions
    ├─ evidence provenance
    └─ deterministic behavioral evidence layout
```

### 7.3 Direct conformance proof

One proof API returning rich outcomes, including:

```text
ProvenExact
ProvenAssumed
Missing / NotDeclared
Ambiguous / Coherence conflict
Incomplete
Unknown / underconstrained
Blocked
Dynamic
Cancelled
BudgetExceeded
InternalFailure
```

Do not collapse this to `bool`.

### 7.4 Assumed trait dispatch

Generic body dispatch must retain:

```text
canonical conformance key
TraitRequirementId
instantiated callable signature
```

without exact source-conformance provenance.

### 7.5 Generic call evidence binding

Call specialization must retain whether each callee conformance obligation is:

```text
Exact(existing ConformanceEvidence)
or
Forwarded(caller assumed evidence)
```

### 7.6 Generic evidence layout

The slot key is conceptually:

```text
(GenericConformanceKey, TraitRequirementId)
```

The resulting layout must be deterministic and capacity-checked.

### 7.7 Runtime evidence recipe

Core lowering must project P1 evidence as:

```text
Exact selected target
or
Forwarded caller slot
```

then intern a `RuntimeConformanceEnvironment`.

### 7.8 Stable direct conformance metadata

Stable metadata must add a direct conformance relation containing a stable subject type node plus stable trait application identity.

P2 owns stable generic associated projection/equality representation.

---

## 8. Planned later C6 architecture

### 8.1 P2 — Generic associated projections and equality

P2 is expected to own:

```phalcom
T::Item
<T as Iterator>::Item
T |= Iterator<Item = A>
T::Item == A
```

P2 must use P1's evidence environment and proof model.

Contextual `T::Item` may resolve only when lexical evidence identifies one associated declaration `Item`; otherwise qualified projection is required.

Associated specialization/equality may feed inference and may therefore require a bounded fixed-point/worklist beyond P1's one-way conformance discharge.

### 8.2 P3 — Conditional conformance and nested proof

P3 is expected to own:

```phalcom
impl<T> Printable for List<T>
    where T |= Printable
```

with:

```text
candidate selection
substitution
nested obligations
recursive proof states
cycle handling
memoization
proof budget
conditional coherence
```

P3 must extend the P1 proof API rather than replace it.

### 8.3 Conditional overlap remains unresolved until P3 authority

Example:

```phalcom
impl<T> Trait for Box<T> where T |= A
impl<T> Trait for Box<T> where T |= B
```

Current architecture rejects overlapping heads and has no specialization rule.

Unless an authoritative spec explicitly changes that, the safe default is to preserve overlap rejection. Luna must not invent condition-based specialization.

### 8.4 P4 — Iterator / Iterable generic laws

Generic constraints alone do not automatically prove:

```text
Iterator ⇒ Iterable
```

P4 must explicitly establish the language mechanism: blanket/derived conformance, supertrait implication if separately ratified, or another first-class proof rule.

No special case in the compiler/VM.

### 8.5 P5 — Production iteration migration

Only after P4:

```text
canonical Universe Iterator / Iterable declarations
collection conformances
fresh stateful iterators
Iterator.iterator === self
for-checker Item typing
for lowering via iterator + next + match
Map direct iteration → (key: K, value: V)
RangeIterator
stateful lazy adapters
runtime Iterable superclass retirement
iterate(_) / iteratorValue(_) retirement
iteration spec update
```

### 8.6 P6 — Closure/certification

P6 may own remaining generic constraint surfaces, alias/GADT integration, higher-kind closure where normatively specified, tooling/performance stress, and final C6 certification.

Do not pre-commit to unsupported higher-kinded trait subject syntax.

---

## 9. Source-of-truth and ownership table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| source generic constraint syntax | AST/parser | semantic formation/tooling | checker string parsing |
| generic semantic propositions | `GenericSignature` | checker/metadata/incremental/reflection | separate trait-bound signature |
| direct conformance identity | generic constraint/key | evidence/proof/lowering | parameter-only trait map |
| applied trait identity | `TraitRef` | proof/dispatch/metadata | runtime type identity |
| trait application proof | generic evidence/proof layer | declarations/calls | TraitRef constructor |
| lexical assumptions | generic evidence environment | dispatch/calls/P2/P3 | flat ad-hoc vectors |
| exact source proof | `ConformanceEvidence` | exact dispatch/generic proof | fake generic exact evidence |
| associated projection meaning | `TypeData::AssociatedProjection` + `types::normalize_type` | checker/metadata/lowering | alternate normalizer |
| call evidence discharge | semantic call specialization | core lowering | compiler re-proof |
| runtime type substitution | `RuntimeTypeEnvironment` | VM/runtime typing | conformance env |
| runtime executable evidence | `RuntimeConformanceEnvironment` | trait requirement invoke | semantic lookup table |
| public generic metadata | stable metadata model | importer/reflection | snapshot IDs |
| bound navigation | source index canonical identity | LSP | text matching |
| checkpoint lifecycle state | this file | plans/guidance/handoffs | competing state documents |

---

## 10. Verification ledger

| Gate | Scope | Evidence | Result | What it proves |
|---|---|---|---|---|
| C6 planning baseline | live repository / C5 takeover | C6.P1 requirements/repository audit | COMPLETE | P1 plan is grounded; no implementation claim |
| `C6.P1.G1` | canonical source/generic contract | 11 semantic generic-conformance tests | PASS | `|=` syntax and canonical direct propositions |
| `C6.P1.G2` | semantic evidence/composition | 11 semantic generic-conformance tests | PASS | lexical assumed evidence, body dispatch, exact/forwarded discharge |
| `C6.P1.G3` | compiler/runtime transport | 4 core generic-conformance tests | PASS | selected exact/default/forwarded evidence executes without runtime proof search, including cross-module transport |
| `C6.P1.G4` | metadata/cross-module | metadata export/validation, reflection regression, core runtime gate | PASS | durable direct-bound metadata and relation-aware reflection |
| `C6.P1.G5` | incremental/tooling | 33 fingerprint tests, source-index regression, full LSP package | FOCUSED PASS; BASELINE BLOCKED | canonical identity, bound navigation, `::` hover, and operator token coverage; one unrelated broad parity baseline |
| `C6.P1.G6` | focused P1 stabilization | semantic generic-conformance + constraints + core generic-conformance | PASS | bounded P1 acceptance matrix is green |

Only actual execution evidence may change these rows to PASS/FAIL.

---

## 11. Deferred / baseline issue ledger

### C6-BL-INHERITED-01 — C5-BL-01 / C5-BL-04

Description:

```text
legacy generic call-entry / Range / Option.match failures
```

Policy:

```text
if the new direct generic evidence call path reproduces the same underlying defect:
    reclassify as P1-owned blocker

otherwise:
    retain as inherited baseline for later production iteration/runtime maintenance
```

### C6-BL-INHERITED-02 — C5-BL-07

Description:

```text
two Universe Bool dependency/capability failures
```

Initial classification:

```text
D — unrelated inherited baseline
```

Reclassify only with concrete C6 coupling evidence.

### Design-deferred issues, not baselines

The following are intentionally deferred architecture:

```text
generic associated projection/equality → P2
conditional conformance recursion/coherence → P3
Iterator ⇒ Iterable mechanism → P4
production iteration migration → P5
higher-kinded trait subject closure → later normative/planned work
```

Do not report them as implementation failures before their owning plans.

---

## 12. Consultation / amendment ledger

### 2026-09-16 — LANG005.C6.P1 Mandatory Clarifications and Execution Amendments

- **Plan/Task**: `LANG005.C6.P1` (T1–T8)
- **Trigger**: Authoritative clarification on Phalcom language model (no top-level free functions; all callables are methods on classes/traits) and integration with receiver specialization.
- **Decision**: All generic callable examples and tests must use legal method declarations (`class` / `trait` methods). Generic evidence environment composes owner and callable generic constraints and integrates with `ReceiverSpecialization`. Status vocabulary normalized across checkpoint records.
- **Affected Invariant/Scope**: Clarification across all P1 tasks and test fixtures; no free-function forms added.
- **Follow-up**: Ground all tests and evidence transport in method/receiver dispatch.

Record only durable architectural consultations or accepted amendments here.

### 2026-09-16 — LANG005.C6.P1 adviser consultation: trait-header dependency lane

- **Plan/Task**: `LANG005.C6.P1.T5` semantic replay/currentness
- **Trigger**: Generic callable signature replay fell back to a nominal `DeclarationShell` dependency for a trait bound, but traits intentionally have no nominal declaration shell.
- **Decision**: Add `SemanticDependency::TraitHeader`; map it to `QueryKey::TraitHeader`; record it through a trait-specific resolver path while retaining `DeclarationShell` for ordinary owners and `TraitSurface` for behavioral member use.
- **Follow-up**: Implemented and verified through the semantic generic-conformance gate; no plan amendment required.

### 2026-09-16 — LANG005.C6.P1 compiler recipe projection correction

- **Plan/Task**: `LANG005.C6.P1.T6` runtime conformance recipe projection
- **Trigger**: Semantic call specialization carried concrete witness keys while compiler callee layouts retained generic parameter subjects, leaving runtime recipe slots empty.
- **Decision**: Materialize callee layout conformance subjects and trait arguments with specialization bindings before matching semantic evidence; preserve semantic evidence as the sole witness authority.
- **Follow-up**: Exact/default/forwarded core runtime tests pass; no second proof source or runtime search introduced.

Each entry should include:

```text
date
plan/task
trigger
decision
affected invariant/scope
follow-up
```

Do not record routine debugging chatter.

---

## 13. Performance/resource invariants

C6 must keep these resource properties visible throughout implementation.

### 13.1 Direct assumed evidence lookup

Generic body/member proof should use indexed canonical assumptions, not scan all generic constraints for every send.

### 13.2 Proof reuse

Repeated identical exact/nested obligations should reuse the semantic query/proof context. P1 should exploit existing exact evidence reuse; P3 may add explicit recursive memoization when nested proof arrives.

### 13.3 Runtime environment reuse

`RuntimeConformanceEnvironmentRegistry` interning is the canonical runtime reuse mechanism.

Repeated identical bounded generic calls must not produce permanent environment growth proportional to invocation count.

### 13.4 Runtime rooting

Detached/default/witness handles transported by generic evidence must remain rooted through the existing runtime conformance environment registry.

### 13.5 No premature optimization

Do not design P3's recursive proof cache or P5 iterator performance architecture before the corresponding semantic workload exists and is measured.

---

## 14. Checkpoint acceptance direction

C6 is not accepted merely when `T |= Trait` works.

Checkpoint completion eventually requires coherent evidence for:

```text
direct generic conformance
generic associated projections/refinements
conditional/nested conformance proof
generic Iterator/Iterable laws
production iteration migration
legacy cursor retirement
cross-module metadata/incremental/tooling correctness
resource/performance sanity
focused stress/certification
```

The final C6 completion state must be truthfully distinguished from individual plan completion.

---

## 15. Current next action

The active next action is:

```text
Begin `LANG005.C6.P3` (Conditional Conformance Proofs & Cycles) according to its authoritative plan when scheduled. The P2 walkthrough and handoff contain the completed generic associated projections, refinements, equality, and inference substrate.
```

Before implementation:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then re-read:

```text
LANG005.C6-CHECKPOINT.md
LANG005.C6.P2-walkthrough.md
LANG005.C6.P2-handoff.md
docs/specs/generic-constraints.md
```

Mechanical drift may be adapted. Architectural drift requires consultation.

---

## 16. Lifecycle truth

These states are distinct:

```text
planned
implemented
focused tested
baseline blocked
release complete
checkpoint complete
```

The current checkpoint is:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
```

That state must remain until actual implementation evidence justifies an update.

## C6.P2/T1 consultation — receiver-owner evidence (2026-09-16)

Starting revision: `af2d0aec`; checkout already contained partial T1 work.
User scope: fix independent owner-evidence/layout failures; leave exact variant
conformance, exact-case inference, and enum lowering untouched.

Decision: receiver specialization supplies fixed declaration-owner substitutions.
Do not activate owner inference merely to transport evidence. Finalize one call
specialization after ordinary/method-local application, discharge the composed
owner/method obligations canonically, and preserve semantic runtime slot positions.
Evidence layout construction/composition must propagate failure, never default to
an empty environment. No VM conformance search or persistent capture redesign.

Active work: T1 correction and focused verification; P2 overall remains partial.
Exact-variant work and broader P2 projection implementation are outside this run.

### C6.P2/T1 nominal-owner dependency correction and focused result — 2026-09-16

- **Plan/Task**: `LANG005.C6.P2.T1`
- **Trigger**: Receiver-owner finalization queried generic signature metadata for a trait-owned callable requirement, creating a nominal `DeclarationShell` dependency and causing the semantic workspace fallback to publish an empty callable snapshot.
- **Decision**: Guard owner finalization with nominal declaration-table membership before querying `declaration_generic_signature()`. Trait-owned requirements therefore retain the established `TraitHeader` dependency lane; only nominal owners contribute receiver-owned generic evidence.
- **Affected invariant/scope**: Owner and method evidence compose into one semantic specialization; failed calls remove partial specialization; exact-variant, exact-case, and enum-lowering work remains outside this correction.
- **Follow-up**: Core generic-conformance (9 selected tests) and semantic generic-conformance (15 tests) pass on the corrected tree. Scoped semantic/core compile check passes. Resume P2 at T2; do not treat this partial T1 slice as P2 closure.

### C6.P2 Completion & Verification Record — 2026-09-16

- **Plan**: `LANG005.C6.P2` (Generic Associated Projections, Refinements, Equality, and Inference)
- **Status**: `COMPLETE` / `IMPLEMENTED` / `FOCUSED_TESTED`
- **Accomplishments**:
  - T1: Resolved receiver-owner evidence composition and generic evidence layout ordering.
  - T2: AST & Parser support for associated specialization syntax in trait requirements (`Iterator<Item = T>`), qualified projections (`<I as Iterator>::Item`), and where-clause kind facts.
  - T3: Lowered associated specialization into canonical `GenericConstraint::Conforms` + `GenericConstraint::Equivalent` with canonical orientation (`left <= right`, `INV-13`).
  - T4: Canonical projection formation and generic evidence normalization (`INV-01`, `INV-04`, `INV-13`).
  - T5: Lexical generic evidence environment composition, projection normalizer integration with generic evidence, and contextual projection support.
  - T6: Extended solver term calculus with `InferenceTerm::AssociatedProjection`, bounded 16-iteration fixed-point normalization loop, one-way inference discipline (`INV-27`), and call-site equality discharge (`INV-20`, `INV-29`).
  - T7: Durable stable metadata support with `StableAssociatedTypeRequirementRef`, `TypeNode::AssociatedProjection`, and metadata validation (`INV-33`).
  - T8: Source index mapping for associated projection targets, occurrence resolution, and semantic fingerprint parity (`INV-13`).
  - T9: Purely behavioral runtime boundary closure (`INV-37`) with zero runtime projection bytecodes or associated dictionary machinery.
  - T10: All 5 stabilization gates G1–G5 passed cleanly.

