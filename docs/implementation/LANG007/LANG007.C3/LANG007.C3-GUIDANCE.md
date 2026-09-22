---
id: LANG007.C3.guidance
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: checkpoint-guidance
title: "Type Data and Unified Algebraic Static Domains"
status: PROPOSED
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
---

# LANG007.C3 — Implementation Guidance

## Type Data and Unified Algebraic Static Domains

## 0. Purpose

This document is the compact architectural guidance for all `LANG007.C3` implementation plans.

Use it with:

```text
LANG007.C3-requirements-analysis.md
LANG007.C3-CHECKPOINT.md
the active P1/P2/P3 patch-grade plan
predecessor walkthrough/handoff
live repository
current authoritative specs
```

This guidance is not a replacement for the active numbered plan.

It fixes the architectural direction that later plans must preserve.

---

# 1. C3 in one sentence

Unify runtime products and sums under `data`, add nominal type-system-only `type data` domains, generalize static generic application and associated lookup around those domains, then expose the compiler's existing structural domains through the same public language without duplicating C1/C2 semantics or introducing runtime static evaluation.

---

# 2. Read this first

C3 is **semantic unification**, not physical representation unification.

Correct:

```text
one source declaration model
one semantic algebraic identity model
one static-domain classification model
one associated namespace authority
one generic application model

multiple optimized internal/runtime representations
    are still allowed
```

Incorrect:

```text
"everything is data"
therefore
"everything must use DataObject storage"
```

Incorrect:

```text
"type data looks like values"
therefore
"create runtime objects for it"
```

Incorrect:

```text
"user static domains are new"
therefore
"build a second type/static-term engine"
```

---

# 3. Non-negotiable inherited C2 rules

Do not redesign:

```text
*
    positional pack

**
    labeled pack

***
    complete pack

...
    NOT pack syntax
```

Do not redesign:

```text
TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks
GenericParameterMode
GenericBinderHeader
whole-pack InferVarId(KindId)
checked pack substitution/materialization
schema-v4 scoped pack metadata
```

Keep:

```text
LabeledTypePack != RecordRow
CompleteTypePack != Tuple
Single(TypePack) != PositionalPack(TypePack)
no pack partition solver
no runtime pack solver
```

If C3 appears to require weakening one of these, STOP AND CONSULT.

---

# 4. Primary architecture decision

## 4.1 Do not create a second static-term universe

The live repository already uses `TypeId` for many non-proper static/type-level terms.

Therefore C3 should extend the current canonical term graph.

Target:

```text
TypeId
    canonical handle for non-row kinded static terms

ProperTypeId
    proof that one TypeId specifically denotes a proper Type

RecordRowId
    canonical row-domain handle, retained separately
```

New type-data/const terms may be represented by `TypeId`.

That does **not** make them proper types.

---

## 4.2 User-defined static domains

Add one dynamic classifier, conceptually:

```rust
KindData::StaticDomain(StaticDomainId)
```

Do not add:

```rust
KindData::Shape
KindData::Endian
KindData::ParserState
...
```

User declarations are data in a domain registry, not Rust enum variants.

---

## 4.3 Heterogeneous application

General canonical envelope:

```rust
enum StaticArgument {
    Term(TypeId),
    RecordRow(RecordRowId),
}
```

Why:

```text
TypeId
    already works for Type, packs, type lambdas, projections,
    and can work for const/type-data terms

RecordRow
    deliberately has separate identity and inference machinery
```

Do not solve this by forcing rows into TypeId.

---

## 4.4 Generic parameter classification

Prefer preserving:

```text
TypeParameterData.kind: KindId
GenericBinderHeader.parameter_kinds
GenericSignature.parameter_kinds
InferenceVariable.kind
```

with dynamic nominal static-domain `KindId`s.

Do not maintain both:

```text
kind
and
domain
```

as independent authorities for the same parameter.

If the implementation introduces a `StaticDomainRef` helper, one representation must be derived from the other.

---

# 5. Unified algebraic declaration model

## 5.1 Four independent axes

Every algebraic declaration has:

```text
phase
    Runtime | TypeOnly

shape
    Product | Sum

implementation provenance
    Phalcom | Native | Intrinsic | ...

ownership
    module | associated nominal owner
```

Never infer one axis from another.

Examples:

```phalcom
data Point(_ x: Int, _ y: Int)
```

```text
phase          Runtime
shape          Product
implementation Phalcom
ownership      module
```

```phalcom
intrinsic type data TypePack
```

```text
phase          TypeOnly
shape          compiler-defined algebraic/static domain
implementation Intrinsic
ownership      Universe module
```

---

## 5.2 Target AST

Prefer one canonical AST shape:

```rust
AlgebraicDef {
    phase,
    shape,
    implementation,
    ...
}
```

Final semantic consumers must not branch:

```text
Statement::Data => product semantics
Statement::Enum => sum semantics
```

P1 may retain a compatibility source flag/span for `enum`.

That flag is presentation/diagnostic state only.

---

# 6. Parser guidance

## 6.1 Braced body discrimination

These are different:

```phalcom
data Person {
    name: String
}
```

record product.

```phalcom
data State {
    Ready
}
```

sum.

Use syntactic shape.

Do not use name resolution.

---

## 6.2 Empty forms

Keep:

```phalcom
data Token()
```

zero-component product, one semantic value per exact runtime specialization.

Keep:

```phalcom
data Impossible {}
```

empty sum, zero inhabitants.

Same algebraic distinction for `type data`.

---

## 6.3 Singleton vs zero-arg constructor

Never collapse:

```phalcom
data X {
    A
    B()
}
```

`A` is singleton associated value.

`B()` is constructor family.

Existing runtime exact-case behavior must continue to observe the distinction.

---

## 6.4 Legacy `enum`

Compatibility route, if P1 keeps one:

```text
parse enum
    ↓
produce canonical Runtime + Sum AlgebraicDef
    ↓
emit migration diagnostic
```

Forbidden:

```text
parse enum
    ↓
EnumDef semantic product
    ↓
later convert to Data
```

if that path creates a separate semantic identity.

P3 removes/finalizes compatibility only after the repository corpus has migrated.

---

# 7. Preserve ADT/GADT machinery

The current enum subsystem is not throwaway code.

Reuse:

```text
VariantId
VariantInfo
VariantConstructorSignature
CaseTypeEnvironment
ExactCase TypeData
enum requirement/default semantics
match exhaustiveness
GADT refinement
exact-case impl/conformance
runtime case lowering
reflection/reification
```

Neutralize ownership gradually.

A safe compatibility technique is a neutral case ID/facade backed by the same owner+selector identity.

Do not mass-rewrite exact-case machinery solely to remove the word `Variant`.

New public APIs and stable metadata should prefer `case`.

---

# 8. One algebraic semantic authority

The target is one authoritative product/facade:

```text
AlgebraicDeclarationInfo
    owner
    phase
    generics
    representation
      Product
      Sum
```

Compatibility views may project:

```text
DataInfo
EnumInfo
VariantInfo
```

for current consumers.

The forbidden architecture is:

```text
AST
├─ independently build AlgebraicInfo
├─ independently build DataInfo
└─ independently build EnumInfo
```

That creates divergent truth.

Derive all adapters from one authoritative declaration product.

---

# 9. Type-data canonical identity

## 9.1 Domain

Every `type data` introduces one nominal static domain.

```phalcom
type data Width(_ value: Int)
type data Height(_ value: Int)
```

must remain distinct.

## 9.2 Term

Canonical term identity is:

```text
domain identity
+ constructor/case identity
+ owner specialization if associated
+ canonical arguments
```

No runtime equality.

No object address.

No user hash.

No runtime method dispatch.

---

# 10. Const/static literal guidance

C3 needs at least compile-time `Int` because accepted source includes:

```phalcom
Shape::Vector(3)
Integer<32, Endian::Big>
```

Do not parse these literals as runtime expression evaluation.

Represent supported static constants canonically in the static term system.

Conceptually:

```rust
TypeData::ConstInt(3)
```

classified by the compiler-supported static Int domain.

Do not automatically lift arbitrary runtime `Int` expressions.

---

# 11. ProperTypeId is sacred

Any C3 patch that makes this legal internally:

```text
Shape::Vector(3)
    accepted where API requires ProperTypeId
```

is unsound.

The fact that both are stored behind `TypeId` is an implementation detail.

Always preserve:

```text
kind/domain == Type
    required for ProperTypeId
```

C3 tests must contain negative proper-type boundary cases.

---

# 12. StaticArgument guidance

## 12.1 Canonical representation

```rust
StaticArgument::Term(TypeId)
StaticArgument::RecordRow(RecordRowId)
```

or exact equivalent.

## 12.2 `Applied`

Generalize:

```rust
TypeData::Applied {
    origin,
    arguments: Box<[StaticArgument]>,
}
```

## 12.3 Compatibility API

Retain a convenient type-only helper.

Example:

```rust
apply_type_form(origin, &[TypeId])
```

may internally wrap:

```rust
StaticArgument::Term
```

Use a generalized canonical helper for heterogeneous application.

Do not force every current caller into noisy migration if an adapter is safe.

---

# 13. RecordRow guidance

RecordRow remains:

```text
unordered/keyed row identity
open-tail semantics
row-specific solver
```

It is not:

```text
LabeledTypePack
```

and it is not to be rewritten as a fake TypeId term merely for C3 application storage.

If heterogeneous application needs a row parameter:

- use the existing canonical row representation;
- preserve row-tail parameter/projection identity;
- let `StaticArgument` carry it.

---

# 14. Scoped aliases/type lambdas

When `Applied` becomes heterogeneous, remember that aliases/type lambdas are a separate representation layer.

Audit:

```text
ScopedTypeData::Applied
TypeLambdaData
beta reduction
scoped substitution
stable ScopedTypeNode
metadata export
```

A P2 implementation that changes only global `TypeData::Applied` is incomplete.

Do not wait until metadata tests to discover this.

---

# 15. Generic binder guidance

## 15.1 Domain vs mode

Keep:

```text
P: TypePack
    domain = TypePack
    mode = Single

*Ts
    domain = TypePack
    mode = PositionalPack
```

Likewise:

```text
S: Shape
    domain = Shape
    mode = Single
```

## 15.2 Variance

Do not invent user static-domain variance semantics in C3.

Default/invariant unless an existing rule already applies safely.

---

# 16. Inference guidance

## 16.1 Reuse InferVarId

For user domain:

```text
Shape
```

create ordinary query-local:

```text
InferVarId(kind = StaticDomain(Shape))
```

No new static-var ID family.

## 16.2 First-order structural rules

Same constructor:

```text
Vector(N) = Vector(3)
    => N = 3
```

Different constructor:

```text
Scalar != Vector(N)
```

Different domain:

```text
Shape::Vector(3) != Other::Vector(3)
```

## 16.3 No evaluator

Do not implement:

```text
if static term has method, call method
run user function
execute match
evaluate arbitrary closure
```

C3 terms are closed canonical algebra, not general compile-time programs.

---

# 17. Type members

The existing type-member subsystem already demonstrates heterogeneous domains.

P2 should converge it toward the common static-domain model.

Do not leave:

```text
generic application domain model A
type-member domain model B
```

as independent sources of truth.

Compatibility enums/wrappers are acceptable during migration if they translate to the canonical domain/argument model.

---

# 18. Associated `type data`

## 18.1 Required semantics

```phalcom
class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

One associated namespace:

```text
Parser::State
Parser::State::Start
Parser::Initial
```

## 18.2 Generic owner

```phalcom
class C<T: Type> {
    type data State {
        Empty
        Present(_ value: T)
    }
}
```

Keep:

```text
declaration identity:
    C::State

specialization:
    C<Int>::State
    C<String>::State
```

Do not duplicate the declaration per specialization.

---

# 19. AssociatedSurface is the lookup owner

Current:

```text
AssociatedSurface
AssociatedFamilyInfo
AssociatedFamilyKind
AssociatedMemberId
AssociatedResolution
```

Extend this system.

Do not create a second lookup pipeline solely for static domains.

Namespace collision diagnostics must cover:

```text
case family
data constructor
associated static domain
type/static member
associated callable family
other declaration-owned associated base
```

No source-order winner.

---

# 20. Trait-associated types are different

Keep this distinction explicit in code and docs:

```text
trait associated type
    evidence-bound requirement
    supplied/specialized by conformance

associated type data
    inherent declaration-owned static domain
    specialized by owner environment
```

Shared syntax/lookup helper does not imply shared evidence ownership.

C3 does not add:

```phalcom
trait X {
    type data State
}
```

as a requirement form.

---

# 21. Trait generic boundary

Do not let storage capability define language semantics.

If a generalized `TypeId` can physically contain a type-data term, that does not prove trait coherence/evidence knows how to reason about it.

Default C3 policy:

```text
reject unsupported user type-data domains on trait generic parameters
```

until explicitly migrated.

If a P2 plan decides to support them:

- audit TraitRef;
- conformance head identity;
- coherence;
- substitution;
- evidence environment;
- metadata;
- inference;
- diagnostics;
- incremental fingerprints.

That becomes a deliberate P2 architecture item, not a side effect.

---

# 22. Metadata guidance

## 22.1 Stable facts

Metadata must encode:

```text
phase
shape
declaration
case
static domain
static term
StaticArgument
associated static domain
owner specialization
generic parameter domain
```

## 22.2 Identity

Never encode identity by:

```text
"Shape::Vector(3)" string
rendered type name
source path text without stable module identity
raw TypeId
raw StaticDomainId
raw InferVarId
```

## 22.3 Schema

Baseline is C2 schema v4.

Every public encoding change is explicitly versioned.

Do not silently add meaning to v4.

Do not rewrite historical fixtures merely to avoid compatibility handling.

---

# 23. Incremental guidance

Fingerprint semantic facts, not syntax accidents.

Should change:

```text
data -> type data
Product -> Sum
case set
payload domain
GADT result
static-domain binder
static argument
associated domain owner/environment
```

Should not change solely for:

```text
equivalent legacy enum -> data migration
source range movement
runtime representation selection
private helper rename
```

Cold and incremental final states must agree.

---

# 24. Runtime guidance

## 24.1 Runtime data

Keep specialized runtime paths.

No requirement to merge:

```text
DataObject
AdtCaseObject
TupleObject
RecordObject
immediates
```

## 24.2 Type data

Never add runtime inhabitants.

Static term formation stops in semantic/compiler metadata.

## 24.3 Intrinsic data

`intrinsic data Tuple<***Es>` means:

```text
source-visible semantic declaration
backed by compiler/runtime-owned specialized representation
```

It does not mean:

```text
construct a generic DataObject tuple
```

---

# 25. TupleEntries guidance

`TupleEntries` is the public static-domain name for the complete two-lane tuple entry shape.

Canonical internal authority stays:

```text
KindId::COMPLETE_TYPE_PACK
TypeData::CompleteTypePack
project_complete
project_positionals
project_labeled
```

Required public equivalences:

```text
T::Entries == ***T
T::Positionals == *T
T::Labeled == **T
```

Do not introduce:

```text
TypeData::TupleEntries
```

if it would be a second representation of the same semantic term.

A source declaration descriptor/facade is fine.

---

# 26. Universe migration guidance

Do not migrate Universe before P2 static-domain semantics are stable.

P3 migration order should be:

```text
1. intrinsic static-domain declarations
2. intrinsic Unit/Tuple/Record declaration facades
3. structural type members
4. core closed sums Option/Result/Ordering -> data
5. remaining repository-owned enum source
6. tooling/presentation terminology
7. remove/finalize enum compatibility parser
```

Every migration preserves canonical declaration/module identity.

---

# 27. Specification migration guidance

The implementation must not end C3 with source behavior contradicting `docs/specs`.

P1 should begin semantic-owner updates.

P3 finishes cleanup.

Do not rewrite historical implementation records.

Primary owners:

```text
docs/specs/objects/data-objects.md
docs/specs/objects/enums.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/associated-types.md
docs/specs/objects/impl.md
docs/specs/universe.md
pattern/GADT owning specs
```

---

# 28. P1 boundaries

P1 owns:

```text
canonical algebraic AST
data Product/Sum
runtime sum migration
type data top-level
StaticDomainId foundation
canonical static-data terms
proper-type rejection
metadata foundation
```

P1 does not own:

```text
general heterogeneous Applied
associated type data
full static generic inference
Universe migration
LSP completion
corpus removal of enum
```

If P1 starts changing `TraitRef` or associated domain specialization, it is drifting into P2.

---

# 29. P2 boundaries

P2 owns:

```text
StaticArgument
heterogeneous Applied
generic domain resolution
static argument grammar
domain inference
checked substitution
static aliases
type-member domain convergence
associated type data
:: lookup
metadata/import/incremental/source-index
```

P2 does not own:

```text
Universe final migration
enum final removal
general static evaluator
trait type-data requirements
```

---

# 30. P3 boundaries

P3 owns:

```text
intrinsic source surface
TupleEntries bridge
Universe migration
core sum migration
enum final compatibility decision
tooling
corpus
spec closure
certification
```

P3 should mostly consume P1/P2 architecture.

If P3 requires a new static-term carrier or new associated-domain identity model, P2 was incomplete: STOP AND CONSULT rather than inventing it in P3.

---

# 31. Recommended P1 planning emphasis

A P1 patch-grade plan should pre-decide:

1. exact AlgebraicDef AST shape;
2. exact compatibility handling for legacy enum;
3. product/sum braced-body parser discrimination;
4. record-payload case representation;
5. neutral case identity migration technique;
6. authoritative semantic product and adapter strategy;
7. exact static-domain store representation;
8. exact type-data term TypeData variants;
9. static Int term formation needed by type-data constructors;
10. exact metadata schema transition;
11. runtime compiler adapter path;
12. focused ADT/GADT preservation gates.

Do not leave these decisions to implementation improvisation.

---

# 32. Recommended P2 planning emphasis

A P2 patch-grade plan should pre-decide:

1. exact `StaticArgument` enum;
2. `Applied` migration API;
3. scoped heterogeneous argument representation;
4. row symbolic argument representation;
5. static argument syntax AST;
6. generic binder-domain resolution;
7. type-data inference term variants;
8. materialization failures;
9. type-member domain migration;
10. associated-domain declaration IDs;
11. owner specialization representation;
12. AssociatedSurface categories;
13. trait unsupported/supported boundary;
14. metadata schema transition;
15. incremental dependency keys;
16. source-index/LSP semantic targets.

---

# 33. Recommended P3 planning emphasis

A P3 patch-grade plan should pre-decide:

1. exact Universe source files to migrate;
2. intrinsic declaration-to-canonical-domain registration;
3. TupleEntries mapping;
4. static type-member source declarations;
5. Option/Result/Ordering migration ordering;
6. enum compatibility removal point;
7. corpus migration scope;
8. LSP required interactions;
9. metadata/interface compatibility gates;
10. performance/resource evidence;
11. runtime-negative audit;
12. C3 closure truth table.

---

# 34. Testing doctrine

Testing is evidence gathering.

During P1/P2 BUILD mode, do not run workspace tests reflexively.

Use:

```text
T0 exact new regression
T1 direct feature module
T2 owning subsystem
T3 adjacent integration
T4 affected crate
T5 workspace/release only at explicit certification
```

P3 may deliberately broaden because it is checkpoint closure.

---

# 35. Coverage dimensions that matter

Prioritize:

```text
canonical identity
negative domain behavior
empty/singleton/zero-arg boundaries
GADT preservation
heterogeneous composition
aliasing
cross-module persistence
cold/incremental equality
source identity
metadata compatibility
runtime-negative behavior
tooling identity
```

Do not add concurrency/GC testing merely because it exists elsewhere unless a C3 change actually affects those mechanisms.

---

# 36. Baseline failures

Inherited:

```text
BASELINE-001
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Do not repair during C3 unless causal evidence shows a C3-owned regression.

Classify:

```text
A caused -> fix
B probably caused -> investigate/fix
C unclear -> one bounded pass, then defer if nonblocking
D unrelated -> record/defer
```

---

# 37. Debugging budget

Mechanical:

```text
up to ~3 coherent correction cycles with progress
```

Semantic:

```text
record:
Observed
Hypothesis
Evidence
Prediction
Discriminating test

allow one serious correction
then STOP AND CONSULT
```

Architecture:

```text
zero speculative redesign attempts
STOP AND CONSULT immediately
```

---

# 38. Global STOP / CONSULT triggers

Stop if any of these occur:

1. new static terms require weakening ProperTypeId;
2. a second general static-term arena is about to be introduced;
3. every user static domain would require a Rust enum variant;
4. RecordRow must be collapsed into TypeId/LabeledTypePack;
5. Data/Enum convergence loses exact-case/GADT evidence;
6. legacy enum/data create different identity;
7. sum behavior requires a second ADT implementation;
8. generic application cannot become heterogeneous without wholesale trait-system redesign;
9. unsupported trait static-domain behavior would be silently accepted;
10. associated type data needs a separate name-resolution authority;
11. associated domain identity requires rendered strings;
12. trait associated-type evidence is being reused for inherent type data;
13. TupleEntries cannot map directly to CompleteTypePack;
14. intrinsic Tuple/Record would require replacing current runtime representation;
15. static type-data correctness requires VM execution;
16. arbitrary runtime `Expr` is being reused as compile-time evaluation;
17. arbitrary static functions/pattern evaluation become necessary;
18. metadata would serialize raw store/session IDs;
19. cold/incremental equality cannot be restored without global invalidation;
20. a C2 pack invariant needs weakening;
21. the same semantic failure survives one serious correction;
22. source/spec/test authorities materially conflict.

---

# 39. Consultation packet

Use:

```markdown
# C3 IMPLEMENTATION INCIDENT

## Identity
Plan/task/gate/revision

## Invariants
Exact C3-I / INV-C3 IDs

## Observed
Concrete symptom

## Minimal reproduction
Exact source/test/command

## Relevant path
A -> B -> C

## Evidence
Code/product/test facts only

## Hypothesis
Best causal explanation

## Serious correction already attempted
If any

## Architectural alternatives
Only plausible alternatives

## Decision requested
One narrow question
```

---

# 40. Checkpoint bookkeeping

At P1 start:

```text
C3 status        -> IN_PROGRESS
completion       -> PARTIAL
verification     -> UNVERIFIED
active plan      -> P1
PROGRAM/STATUS   -> rebaseline C3
```

At each coherent plan gate:

- record durable new architecture;
- record actual verification;
- record consultation decisions;
- record deferred failures.

Do not turn CHECKPOINT into an edit log.

At P3 closure:

```text
P1/P2/P3
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

C3
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

only if the checkpoint truth table passes.

---

# 41. Required plan completion artifacts

Every P1/P2/P3 normally produces:

```text
LANG007.C3.P<n>-walkthrough.md
LANG007.C3.P<n>-handoff.md
```

P3 handoff should summarize the frozen C3 substrate and the next accepted LANG007 roadmap action.

---

# 42. Do not re-explore

Unless live evidence contradicts the baseline, do not spend implementation time re-deciding:

```text
whether data should replace enum
whether type data has runtime values
whether type-data equality calls user ==
whether packs use * ** ***
whether TupleEntries should replace CompleteTypePack
whether RecordRow == LabeledTypePack
whether GenericParameterMode == static domain
whether the VM should solve type data
whether associated type data is a trait requirement
whether runtime Tuple must become DataObject
```

These are already decided for C3.

---

# 43. What still requires per-plan verification

Verify against live source before each plan:

```text
exact AST/parser symbols
exact DB query products
exact adapter consumers
exact metadata schema version
exact trait/conformance argument assumptions
exact associated namespace consumers
exact Universe source locations
exact LSP product APIs
exact test module names
```

Mechanical drift is expected.

Architectural drift is not.

---

# 44. Final architectural summary

```text
one algebraic language
    data       -> runtime inhabitants
    type data  -> static inhabitants

one canonical static tower
    KindId classifies
    TypeId stores non-row static terms
    ProperTypeId proves actual Type
    RecordRow keeps dedicated authority

one generic application model
    StaticArgument

one non-row inference identity
    InferVarId(kind)

one associated lookup authority
    AssociatedSurface / ::

one stable persistence model
    declaration/domain/case/argument structure

specialized runtime representations
    remain free beneath the language model

no general compile-time evaluator
no VM static solver
```

If later implementation work remains inside these boundaries, C3 can be delivered without destabilizing C1/C2.
