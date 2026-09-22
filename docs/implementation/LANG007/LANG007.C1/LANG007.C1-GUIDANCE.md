---
id: LANG007.C1.GUIDANCE
category: LANG
program: LANG007
checkpoint: LANG007.C1
kind: implementer-guidance
status: ACTIVE_PLANNING_GUIDANCE
repository: aureat/phalcom-lang
planning_baseline_revision: 5b63ade8073d13d6b528af4774181665c30cbd47
prepared: 2026-09-21
requires:
  - LANG007.C1-CHECKPOINT.md
  - LANG007.C1.P1-patch-grade-implementation-plan.md
---

# Implementer Guidance — LANG007.C1 Type-Level Structural Domains and Intrinsic Type Operations

## 0. Purpose

This is the persistent architectural and execution guidance for all `LANG007.C1` implementation sessions.

Load it before working on:

```text
TypePack / LabeledTypePack
native/compiler-intrinsic type members
typed compile-time type-member arguments
Tuple Positionals / Labeled / At
Tuple/Record/pack/row Map<F>
symbolic projection normalization
pack/row materialization or inference
stable metadata/source identity
Universe type-member source anchors
```

The C1 semantic sentence to keep loaded is:

> Structural type operations are compiler-owned semantic projections over distinct canonical type-level domains; abstract results remain symbolic, concrete invalidity remains invalid, and the VM never becomes a type-member or projection solver.

Every C1 patch must preserve that sentence.

This file is guidance, not checkpoint state authority.

Lifecycle truth belongs in:

```text
LANG007.C1-CHECKPOINT.md
```

Exact task mechanics, file/symbol expectations, focused test commands, and gate criteria belong in the active patch-grade plan.

---

# 1. Current checkpoint state

Current state:

```yaml
checkpoint: LANG007.C1
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
active plan: none
next plan: null
planning baseline: 5b63ade8073d13d6b528af4774181665c30cbd47
```

`LANG007.C1` is complete across P1, P2, and P3. Subsequent work extends the established C1 substrate.

---

# 2. Lifecycle amendment

The P1 plan was originally prepared under a temporary user instruction not to update implementation records during execution.

The later explicit request for canonical lifecycle records supersedes that exception.

Therefore C1 now follows normal lifecycle obligations:

```text
read checkpoint before editing
mark P1 active at start
update checkpoint at durable architecture events
record consultations/amendments
record deferred/baseline failures
record coherent verification gates
finalize P1 state at completion
create P1 walkthrough
create P1 handoff when successor work exists
```

This does **not** reopen normative specification editing.

During P1:

```text
docs/specs/**       read-only unless user separately requests spec work
docs/decisions/**   read-only unless a new decision is explicitly required
docs/implementation/LANG007/** lifecycle records may be maintained
```

Do not opportunistically reorganize unrelated implementation documentation.

---

# 3. Canonical repository location

Use:

```text
docs/implementation/LANG007/
```

with:

```text
PROGRAM.md
STATUS.md
LANG007.C1/
    LANG007.C1-CHECKPOINT.md
    LANG007.C1-GUIDANCE.md
    LANG007.C1.P1-patch-grade-implementation-plan.md
```

Later C1 corrective plans, walkthroughs, handoffs, and genuine C1 companion technical records belong in the same checkpoint directory.

Do not create competing `implementation-state.md` files.

---

# 4. Required read order

Before production edits, read in this order.

## 4.1 Workflow authority

```text
AGENTS.md
implementation-record lifecycle convention
principal architect / implementation planner doctrine
shared consultation/escalation protocol
repository commit/push discipline if present
```

## 4.2 LANG007 semantic authority

Read the live accepted C1 specifications for:

```text
type-level domains
type members
structural Tuple/Record operations
generic/kind syntax
native/intrinsic declaration semantics
```

Then read relevant authoritative specs, including the live equivalents of:

```text
docs/specs/README.md
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/source-declarations.md
docs/specs/objects/impl.md
docs/specs/objects/traits.md
docs/specs/universe.md
docs/specs/vm/products.md
```

Read the accepted Tuple/Record structural-product and modifier/intrinsic decisions.

If the expected accepted LANG007 specs are absent from the implementation checkout, STOP AND CONSULT. Do not reconstruct the language contract from this guidance alone.

## 4.3 C1 implementation state

Read:

```text
PROGRAM.md
STATUS.md
LANG007.C1-CHECKPOINT.md
LANG007.C1-GUIDANCE.md
LANG007.C1.P1-patch-grade-implementation-plan.md
latest completed C1 walkthrough/handoff, when one exists
```

## 4.4 Live production architecture

Inspect live equivalents of:

```text
KindId / KindData / TypeStore / ProperTypeId
TypeData / TupleTypeElement
RecordRowData / RecordRowTail / RecordRowSolver
GenericSignature / TypeParameterData
GenericInstantiation / TypeEnvironment
TypeLevelBinding
TypeStore::apply_type_form
TypeData associated projection / types::projection normalizer
InferenceSession / InferVarId
GenericApplicationSession
RecordRowVarId
stable KindNode / TypeNode
metadata exporter / validator
source-index builder
Tuple/Record Universe source
native source census/runtime member registration
```

Do not trust stale private helper names.

Preserve role, identity, ownership, and lifetime.

---

# 5. Authority and drift rules

Use this authority order:

```text
1. explicit user rulings / accepted LANG007 amendments
2. current authoritative docs/specs semantics
3. LANG007.C1-CHECKPOINT.md accepted invariants/amendments
4. completed C1 walkthroughs/handoffs
5. live production architecture for mechanical facts
6. active C1 patch-grade plan for execution mechanics
7. historical plans/analyses
```

Rules:

- a stale plan cannot override accepted semantics;
- a stale test cannot redefine accepted semantics;
- live source may mechanically drift;
- mechanical drift may be adapted locally;
- material drift in ownership, identity, lifetime, projection representation, inference authority, metadata identity, or runtime responsibility requires STOP/CONSULT;
- checkpoint state is durable current-work authority;
- guidance is persistent cross-plan doctrine, not a chronological log.

---

# 6. Session bootstrap

At the start of every C1 implementation session:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Answer privately:

```text
Which C1 plan/task/gate am I implementing?
What is the actual local revision?
Are there unrelated local edits?
Which accepted LANG007 spec owns the behavior?
Which canonical semantic structure owns the fact I am changing?
Am I about to create a second representation of pack/row/projection/member identity?
Am I accidentally turning a compile-time operation into runtime dispatch?
What is the smallest discriminating test?
Which STOP/CONSULT trigger applies if the live tree contradicts the plan?
```

Preserve unrelated staged/modified/untracked work.

Do not reset, clean, broadly reformat, or broadly stage.

---

# 7. Semantic model to keep loaded

Keep these distinctions explicit:

```text
Type
    proper type domain

Type -> Type
    type-constructor kind

TypePack
    ordered sequence of proper type terms

LabeledTypePack
    ordered sequence of labeled proper type terms

RecordRow
    key-canonical record row domain

Const<Int>
    typed compile-time member argument domain
```

Important non-equalities:

```text
TypePack != Tuple
LabeledTypePack != RecordRow
Int const domain != kind Int
TypeId != ProperTypeId
ProjectionId != runtime method
type-member identity != member name
Stuck != Invalid
symbolic projection != Dynamic
InferVarId != TypeId
RecordRowVarId != RecordRowId
source native type member != runtime native callable
trait associated projection != intrinsic type-member projection
```

If an implementation erases one of these distinctions, stop.

---

# 8. Canonical kinded-form rule

C1 uses the existing canonical kinded-form arena.

Keep:

```text
TypeId
    canonical ID for a type-level form with a KindId

ProperTypeId
    certified TypeId whose kind is Type
```

Therefore it is valid for:

```text
Int                    -> TypeId(kind Type)
Option                 -> TypeId(kind Type -> Type)
explicit TypePack      -> TypeId(kind TypePack)
explicit LabeledPack   -> TypeId(kind LabeledTypePack)
pack parameter P       -> TypeData::Parameter(P), kind TypePack
```

Do not introduce a separate canonical pack arena unless live evidence proves the existing kinded-form arena cannot preserve the accepted semantics. That would be an architectural consultation.

`RecordRow` is the deliberate exception: retain the existing distinct row representation/solver architecture.

---

# 9. TypePack and LabeledTypePack rules

## 9.1 TypePack

A TypePack is:

```text
ordered [T0, T1, ...]
where every Ti is a proper type
```

It is not:

```text
Tuple<T...>
List<Type>
RecordRow
runtime object
```

## 9.2 LabeledTypePack

A LabeledTypePack is:

```text
ordered [(l0,T0), (l1,T1), ...]
```

Rules:

- payloads are proper types;
- labels are unique;
- label order is preserved;
- entry order is semantic;
- it is not normalized by record-row field sorting.

## 9.3 RecordRow contrast

RecordRow remains:

```text
key-identity structure
field presentation order not semantic
open tail may be stable parameter/projection
```

Code sharing may reuse generic algorithms where sound, but representation/identity must remain separate.

---

# 10. Tuple structural rule

Tuple type identity remains structural.

Conceptually:

```text
Tuple(
    positionals: ordered proper types,
    labeled: ordered labeled proper types
)
```

The physical representation may remain a single ordered `TupleTypeElement` array so long as:

```text
all positionals precede all labeled entries
labels are unique
order is preserved
```

Do not introduce a nominal `Tuple<...>` type merely to host type members.

For symbolic pack-based Tuple formation, use the canonical C1 symbolic form planned by P1 and normalize it to the existing concrete Tuple identity when packs become explicit.

---

# 11. Type-member identity rules

A type member is not an ordinary runtime message.

Canonical identity must include semantic owner/domain and selector.

Required domains include the accepted equivalents of:

```text
Tuple
Record
TypePack
LabeledTypePack
RecordRow
```

Required selectors include:

```text
Named(...)
Subscript
```

The following two source forms denote the same canonical Tuple subscript member:

```phalcom
T[I]
T::At<I>
```

Do not create two semantic operations and merely assert their results happen to compare equal.

Same-spelled members on different semantic domains remain different identities:

```text
Tuple::Map
Record::Map
TypePack::Map
LabeledTypePack::Map
RecordRow::Map
```

Source anchors may be declaration-backed for navigation/provenance. Runtime method identity is not the canonical type-member identity.

---

# 12. Type-member parameter rules

Type-member parameters have explicit domains.

Required first-release categories:

```text
Kind(k)
Const(proper type)
```

Examples:

```phalcom
F: Type -> Type
I: Int
```

mean:

```text
F is a type-level form of kind Type -> Type
I is a compile-time value typed Int
```

Do not:

- add `Int` to the kind universe;
- convert integer literals into fake TypeIds;
- use runtime `Value` in semantic projection identity;
- generalize all nominal/trait generic arguments merely to support these members.

Expected-domain semantic elaboration decides what an argument means.

---

# 13. Projection architecture rules

C1 should converge on one generalized semantic projection identity.

A projection carries the semantic equivalents of:

```text
subject
member identity
canonical arguments
result domain
```

Member category may distinguish:

```text
trait-associated requirement
compiler-intrinsic type member
```

but both should use coherent projection normalization/lifetime infrastructure.

Existing associated projection call sites may retain compatibility constructors.

Do not duplicate:

```text
TupleAtProjection
TupleMapProjection
RecordMapProjection
PackMapProjection
```

as unrelated canonical node families.

An abstract but valid projection remains a canonical symbolic projection.

---

# 14. Normalization result rules

Preserve at least these semantic outcomes:

```text
Reduced
Stuck
Invalid
Cancelled
BudgetExceeded
InternalFailure
```

Rules:

```text
abstract input that prevents reduction
    -> Stuck / symbolic

concrete out-of-range index
    -> Invalid

wrong Map constructor kind
    -> Invalid

Dynamic
    != fallback for either case
```

Do not make type relation/subtyping silently normalize through Dynamic to compensate for missing intrinsic handling.

---

# 15. Tuple indexing rules

For Tuple total arity `N` and integer `i`:

```text
j = i       if i >= 0
j = N + i   if i < 0
```

Index valid iff:

```text
0 <= j < N
```

Index order is:

```text
positionals ++ labeled payloads
```

Thus for:

```phalcom
(Int, name: String)
```

index `1` selects `String`.

Concrete Tuple `At` must be O(1) with direct canonical component indexing.

Do not scan labels or use runtime Tuple indexing.

P1 does not own Symbol or Range type-level Tuple indexing.

---

# 16. Map rules

`F` must have kind:

```text
Type -> Type
```

Payload transformation uses the ordinary type-form application authority:

```text
F<T>
```

Do not special-case `Option`.

## 16.1 TypePack

```text
[A, B]::Map<F>
    -> [F<A>, F<B>]
```

Order preserved.

## 16.2 LabeledTypePack

```text
[(name,A), (age,B)]::Map<F>
    -> [(name,F<A>), (age,F<B>)]
```

Labels and label order preserved.

## 16.3 Tuple

```text
(A, B, name: C)::Map<F>
    -> (F<A>, F<B>, name: F<C>)
```

Preserve:

```text
arity
positional/labeled boundary
labels
label order
component order
```

## 16.4 Record

```text
#{name: A, age: B}::Map<F>
    -> #{name: F<A>, age: F<B>}
```

Preserve record key semantics.

## 16.5 Open row

```text
{name: A | R}::Map<F>
```

must map both:

```text
name -> F<A>
R    -> MapRow(R,F)
```

The tail must not remain unchanged.

A stable symbolic row projection is preferable to inventing a query-local row variable as durable identity.

---

# 17. Map/At composition law

For valid concrete `I`:

```text
(T::Map<F>)::At<I>
    ≡
F<T::At<I>>
```

The architecture should make this law emerge naturally.

Do not add a one-off rewrite solely to satisfy one test if the underlying canonical semantics are inconsistent.

---

# 18. Generic inference and materialization rules

The existing architecture already separates:

```text
kinded type inference
row inference
```

Preserve it.

Expected first-release strategy:

```text
P: TypePack
    -> InferVarId(kind TypePack)

L: LabeledTypePack
    -> InferVarId(kind LabeledTypePack)

R: RecordRow
    -> RecordRowVarId
```

Do not create `PackVarId` merely for whole-pack kinded variables.

A separate sequence/pack solver becomes justified only when later pack expansion/splitting inference is actually in scope.

`GenericInstantiation` kinded bindings may carry pack-kinded TypeIds.

Do not assume a field named `type_bindings` means every stored TypeId must be a proper type.

---

# 19. Query-local lifetime rules

Hard boundary:

```text
InferVarId
RecordRowVarId
solver frames
query-local proof state
```

must never enter:

```text
TypeStore canonical nodes
durable semantic snapshots
stable metadata
public cross-module identity
```

Symbolic pack/member/row projections require stable semantic terms, not leaked solver variables.

If the only apparent way to preserve a projection is to serialize/query-persist an inference ID, STOP AND CONSULT.

---

# 20. Existing associated-type rules

Existing trait associated types remain authoritative for trait-owned associated requirements.

Do not weaken:

```text
TraitRef
AssociatedTypeRequirementId
conformance associated binding
generic associated evidence
projection normalization from trait evidence
```

to fit compiler-native type members.

Generalizing projection storage is allowed; changing associated-type proof semantics is not part of C1.

If migration to generalized projection representation changes the meaning of existing `Self::Item` / qualified projection behavior, STOP AND CONSULT.

---

# 21. Stable metadata rules

Durable metadata must represent the new semantic structure without raw store-local IDs.

Expected stable categories:

```text
TypePack kind
LabeledTypePack kind
explicit pack terms
symbolic Tuple-from-pack form
intrinsic type-member identity
projection structure
typed const arguments
stable row projection tail
```

Never serialize:

```text
TypeId
KindId
ProjectionId
InferVarId
RecordRowVarId
solver caches
runtime Value
```

A stable graph may of course contain its own stable node IDs.

Metadata identity must not depend on formatted type strings.

---

# 22. Source identity and Universe rules

Canonical Universe source may contain accepted native/intrinsic type-member declarations for Tuple/Record.

Those declarations provide:

```text
authored source anchor
documentation/navigation provenance
intrinsic binding point
```

They do not create ordinary runtime methods.

`phalcom-core/src/native/source.rs` must not register a type-member declaration as a `NativeMemberKey` or runtime callable merely because it has `native` or `intrinsic` implementation origin.

Intrinsic type-member association is explicit and identity-based. `@intrinsic` is retired as a source attribute.

Do not infer semantic intrinsic operation from presentation spelling after member resolution.

---

# 23. Runtime boundary rules

The VM may execute runtime Tuple/Record behavior.

The VM may not decide:

```text
whether subject is a TypePack
which type-member applies
whether Map<F> is kind-correct
how a structural projection normalizes
whether an index is statically valid
which associated projection proof applies
```

C1 should add no:

```text
type-member bytecode
runtime projection lookup
runtime pack class
runtime type-family evaluator
```

If source integration appears to require one, STOP AND CONSULT.

---

# 24. Zero-product rule — verify first

The planning baseline observed:

```text
TypeStore::tuple([]) -> Unit
```

and accepted prior structural-product decisions were understood to unify zero-product semantics.

However, the live accepted LANG007 structural-products specification is the immediate C1 authority.

Before changing empty Tuple/Record representation:

1. read the live accepted rule;
2. inspect current canonical representation;
3. follow the authority ordering;
4. STOP AND CONSULT if a material unresolved conflict remains.

Do not “clean up” empty Record/Tuple representation opportunistically.

---

# 25. Union rule

C1 does not invent automatic structural-operation distribution over unions.

Do not implement:

```text
(A | B)::Map<F>
    -> A::Map<F> | B::Map<F>
```

unless the live accepted C1 specification explicitly requires it.

Keep union behavior explicit and conservative.

---

# 26. Performance rules

Required asymptotics:

```text
concrete Tuple At         O(1)
explicit pack Map         O(n)
concrete Tuple Map        O(n)
concrete Record Map       O(n)
canonical identity compare O(1)
```

No benchmark suite is required unless evidence suggests a regression.

Code inspection plus focused functional tests are sufficient P1 evidence.

STOP AND CONSULT if concrete `At` is trending toward a repeated linear scan because the canonical Tuple representation already supports direct indexing.

---

# 27. Build / stabilize / certify modes

## BUILD MODE

Default while implementing T1–T7.

Use:

- exact regressions;
- directly affected foundation tests;
- strategic `cargo check`;
- sparse discriminating evidence.

Do not run broad suites after every task.

## STABILIZE MODE

At P1 coherent gates:

- focused type-member suite;
- type annotations;
- row inference;
- generic application;
- existing trait associated types;
- metadata/source index;
- affected crate checks.

## CERTIFY MODE

Not automatic for P1.

Run workspace-wide tests/clippy only if:

- the checkpoint/user explicitly requires it; or
- focused evidence reveals a broad interaction that cannot be classified otherwise.

Most C1 implementation time belongs in BUILD MODE.

---

# 28. Required P1 gate sequence

Do not reorder semantic dependencies.

```text
T0  live authority / takeover
 ↓
T1  AST/parser
 ↓
T2  canonical domains/forms
 ↓
G1  canonical-domain gate
 ↓
T3  type-member identity/source anchors
 ↓
T4  generalized projection/typed arguments
 ↓
G2  member identity/projection gate
 ↓
T5  intrinsic structural algebra
 ↓
G3  structural algebra gate
 ↓
T6  inference/materialization
 ↓
T7  stable metadata/source identity
 ↓
G4  persistence gate
 ↓
T8  Universe/core integration
 ↓
G5  focused plan acceptance
```

Do not implement concrete intrinsic reducers before canonical member/projection identity exists.

Do not design metadata around temporary nodes before canonical semantic representation settles.

---

# 29. Testing doctrine

Test to answer semantic questions.

Required dimensions include:

```text
kind identity
proper-type rejection
pack canonicalization
label ordering
row/pack distinction
source syntax
[] / At canonical identity
Map constructor kind checking
const-domain checking
negative indexes
bounds
labeled Tuple integer order
Positionals
Labeled
Tuple Map
Map/At composition
TypePack Map
LabeledTypePack Map
Record Map
open-row mapped tail
symbolic projection
union non-distribution
query-local lifetime
existing associated types
stable metadata
source target identity
zero-product behavior
runtime negative boundary
TupleFromPacks canonicalization
incremental identity where applicable
```

Do not equate broad test count with stronger evidence.

---

# 30. Failure classification

Classify before editing:

```text
A — definitely caused by current C1 change
B — probably caused by current C1 change
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active responsibility.
- C: one bounded classification pass; defer if nonblocking.
- D: record in checkpoint and continue immediately.
- Do not consume architectural-adviser effort on D unless it blocks C1 evidence.

---

# 31. Debugging budget

## Mechanical failures

Allow a few coherent correction cycles while evidence progresses.

Examples:

```text
exhaustive-match cleanup
imports
renamed helper
borrow/type mismatch
test filter
```

## Semantic failures

Before changing code, write:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

## Architectural failures

Do not trial-patch speculative architectural alternatives.

Consult immediately.

---

# 32. Mandatory STOP / CONSULT triggers

STOP AND CONSULT if any of these occurs.

1. Live accepted LANG007 semantics contradict a fixed C1 premise.
2. `TypePack`/`LabeledTypePack` cannot use the existing kinded-form arena without violating a confirmed invariant.
3. Supporting non-Type projections appears to require treating every TypeId as a proper runtime type.
4. Open-row Map cannot preserve a stable mapped tail without a materially different row architecture.
5. A second canonical type-member/projection registry becomes necessary.
6. `T[I]` and `T::At<I>` appear to require separate semantic identities.
7. Existing trait associated-type semantics would need weakening/redefinition.
8. General nominal const generics become necessary.
9. Generic associated types become necessary.
10. A VM bytecode/runtime type-member dispatch path appears necessary.
11. The same nontrivial semantic failure persists after one serious correction.
12. Two plausible fixes have materially different identity/lifetime/representation consequences.
13. Tests can pass only by weakening an accepted invariant.
14. Stable metadata appears possible only by serializing raw local IDs.
15. Open-row Map is trending toward leaving the tail unmapped.
16. Zero-product authorities materially conflict and cannot be resolved by the accepted authority order.
17. Implementation must enter a materially unrelated subsystem and change its semantic ownership.
18. C1 expands into broad language-wide generic-argument migration.

The implementer cannot self-waive a triggered consultation.

---

# 33. Consultation packet

Collect only the evidence needed for a decision:

```text
exact accepted rule
actual local revision
minimal failing source
canonical subject/result kinds
type-member/projection identity
normalization state
relevant row/pack term
small focused diff
one passing comparator
one failing discriminator
```

Ask a decision question.

Do not ask an adviser to rediscover the repository.

Record the adopted decision in `LANG007.C1-CHECKPOINT.md`.

---

# 34. Checkpoint update rules

Update the checkpoint at:

## P1 start

Record:

```text
actual local revision
P1 IN_PROGRESS
checkpoint IN_PROGRESS / PARTIAL
inherited baseline changes if any
```

## Durable architecture events

Examples:

```text
G1 canonical domains established
G2 member/projection identity established
G3 structural algebra established
G4 stable persistence/inference established
architectural consultation adopted
significant deferred baseline discovered
```

## P1 completion

Record:

```text
P1 final status
completion
verification
actual focused gates
deferred broad gates
residual failures
consultations/amendments
next action
```

Do not update after every edit or test.

---

# 35. Walkthrough and handoff rules

At P1 completion create:

```text
LANG007.C1.P1-walkthrough.md
```

It must state what actually happened:

```text
objective/outcome
architecture implemented
important files/components
durable interfaces/invariants
deviations from plan
consultations
tests added
tests executed
tests deferred
performance evidence
residual failures
verification classification
follow-up
```

Create:

```text
LANG007.C1.P1-handoff.md
```

when successor C1/LANG007 work exists.

It should contain only continuation-critical operational context:

```text
state anchor
completed prerequisites
stable takeover map
inherited architecture
do-not-redesign invariants
repository drift
deferred failures
must-read files
next objective
high-risk areas
first commands/tests
```

---

# 36. Do-not-reexplore / do-not-redesign list

Unless a STOP/CONSULT trigger provides contrary evidence, do not spend implementation time re-deciding:

```text
TypePack is not Tuple
LabeledTypePack is not RecordRow
TypeId may identify non-proper kinded forms
ProperTypeId marks kind Type
Int member index is a typed const, not a kind
[] and At are one subscript identity
Map takes Type -> Type
Map uses ordinary type-form application
open row tails must be mapped
abstract projections remain symbolic
Dynamic is not an unresolved-projection fallback
pack whole-value inference can initially use kinded InferVarId
row inference remains separate
query-local IDs never become canonical
trait associated semantics remain authoritative
type-member normalization stays out of VM
intrinsic is a first-class keyword and ImplementationKind
@intrinsic source attribute is retired
NativeIntrinsicId is internal metadata distinct from source implementation kind
source intrinsic type members are not runtime native callables
```

---

# 37. First recommended commands

At P1 start:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then inspect the actual live C1 specs and primary type-system/parser files named by P1 T0.

Do not begin with workspace tests.

The first test should be the smallest discriminating parser/domain regression relevant to the task currently being implemented.

---

# 38. Completion truth

Keep these states separate:

```text
syntax added
canonical form added
intrinsic reducer added
tests written
focused tests passed
affected crates compile
workspace tests passed
checkpoint complete
release complete
```

Normal successful C1 completion is expected to be:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

unless broader certification is actually executed.

A clean Git tree is not verification.

A generated walkthrough is not verification.

A plan saying a gate should pass is not verification.

Only executed evidence may promote the checkpoint.
