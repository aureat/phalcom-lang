---
id: LANG005.C3.P1
category: LANG
program: LANG005
checkpoint: LANG005.C3
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C2
follows: LANG005.C2.P4
supersedes: "LANG005.C3.P1 pre-final-C2 plan prepared at 986568da050d1bbfa7f1d769c9e1f4d58eb81cf3"
prepared: 2026-09-14
repository: aureat/phalcom-lang
repository_baseline: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
requirements: LANG005.C3.P1-requirements-analysis.md
---

# LANG005.C3.P1 — First-Class Trait Declarations and Abstract Trait Surfaces

## Revised Luna Patch-Grade Implementation Plan

This plan is derived from the fresh `LANG005.C3.P1-requirements-analysis.md`. It supersedes the pre-final-C2 P1 plan. The final C2 architecture is now a real predecessor, not a prospective dependency; however, the C1+C2 completion audit proved a small set of foundational defects that must be repaired before trait source work builds on them.

The plan therefore has two ordered responsibilities:

```text
TAKEOVER STABILIZATION
    C1 exact anonymous-product reification
    C1 dynamic Record logical canonicalization
    C2 exact-case conditional publication/invalidation
        ↓
TRAIT CHECKPOINT
    syntax + module declaration
    trait header/generic ownership
    TraitRef
    TraitRequirementId
    TraitSurface
    abstract Self/default analysis
    incremental/source/compiler boundaries
    protocol→trait spec migration
```

No production trait implementation begins until the takeover remediation gate passes.

---

## 0. Executor contract

This plan is designed for a Luna-class implementation model working under constrained architectural authority.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture, ownership rules, identities, and invariants below;
- must not improvise material language, semantic, runtime, or incremental architecture;
- must follow the testing budget rather than testing reflexively;
- must classify and defer unrelated failures under the repository A/B/C/D taxonomy;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep `LANG005.C3-CHECKPOINT.md` current at durable gates;
- must produce `LANG005.C3.P1-walkthrough.md` and `LANG005.C3.P1-handoff.md` before completion;
- must not silently convert deferred audit debt into C3 scope;
- must not treat predecessor checkpoint labels as stronger evidence than landed source and focused tests.

Testing is evidence gathering, not a ritual. Do not run a broad suite merely because an edit occurred. Build coherent semantic slices, run the smallest discriminator that answers the current question, and broaden only at named gates or when evidence requires it.

The executor may not self-waive a STOP/CONSULT condition. Mechanical adaptation is allowed; architectural adaptation is not.

---

## 1. Goal

Close `LANG005.C3` by first repairing the predecessor identity/surface invariants that traits require, then implementing first-class non-storage `trait` declarations, generic `TraitRef` contract references, stable trait requirement/source identities, a separate complete `TraitSurface`, one-time abstract-`Self` default checking, incremental/source integration, a compiler/runtime non-class boundary, and normative protocol→trait specification migration.

---

## 2. Checkpoint acceptance objective

This plan is intended as **full checkpoint closure** for `LANG005.C3`, with a bounded predecessor-stabilization prefix.

The checkpoint is accepted when the repository has this semantic flow:

```text
source trait declaration
    ↓
DeclarationId + DeclarationKind::Trait
    ↓
TraitHeader / TraitInfo
    generic signature owned by DeclarationId
    ↓
TraitRef(declaration, canonical generic arguments)
    ↓
complete TraitSurface
    ├─ TraitRequirementId
    ├─ trait-owned CallableId
    ├─ CallableSemanticSignature
    ├─ visibility/source
    └─ optional default availability
            ↓
       default body analysis once
       under abstract owner-relative Self
       using contract-relative lookup
       using canonical callable application
       with no executable InvocationTargetId
```

Before that flow is allowed to become the C3 foundation, G1 must establish:

```text
C1 exact anonymous-product runtime type
    semantic recipe -> lowering -> runtime environment -> materialized exact_type

C1 dynamic Record identity
    source presentation order != canonical logical identity

C2 exact-case conditional behavior
    contribution -> full-target conditional product -> snapshot publication
    -> owner-complete replacement/removal
```

Successful P1 completion means:

- all integrated predecessor remediations are closed and focused-tested;
- C3 requirements are implemented;
- focused C3 acceptance passes;
- checkpoint state is truthful;
- walkthrough and C4 handoff exist.

It does **not** mean:

- LANG005 is release-certified;
- any target conforms to a trait;
- C4 witness/coherence machinery exists;
- C5 associated types exist;
- C6 conformance constraints exist;
- C7 trait reflection/runtime descriptors exist.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
commit: lang005: complete C2 specialized impl applicability
relevant predecessor: LANG005.C2.P3
```

### Verified repository facts

1. `LANG005.C2` is recorded complete and C2.P3's central specialized inherent-impl architecture is landed.
2. `ConditionalDispatchSelection` and the semantic→lowering→runtime conditional selection boundary are real and must be preserved.
3. `phalcom-semantic/src/session.rs` currently publishes conditional members only under `InherentImplTarget::Declaration(...)`; exact-case conditional consumers exist without a corresponding canonical publisher.
4. `phalcom-semantic/src/dispatch.rs::remove_surface` removes only the declaration-target conditional key, so exact-case conditional lifecycle would become stale as soon as exact-case publication is added.
5. `phalcom-core/src/modules/semantic_lowering.rs::AnonymousProductConstructionLoweringSpec` still contains `kind` + `layout` only; it does not carry `RuntimeTypeRecipe`.
6. `phalcom-core/src/typing/environment.rs` already defines `RuntimeTypeRecipe`, `RuntimeTypeEnvironmentId`, the registry, and `instantiate_type_recipe`; existing frame/block machinery already stores and propagates a runtime type environment.
7. `phalcom-core/src/product/mod.rs` dynamic Record construction still uses `RecordProductShape::from_ordered_labels`, making presentation order equal logical order.
8. `phalcom-modules/src/declaration.rs::DeclarationKind` still contains `Protocol` rather than `Trait`.
9. `phalcom-ast/src/ast.rs::Statement` has no `Trait` variant.
10. `phalcom_ast::BehaviorMember` remains the shared behavior-only AST category.
11. `IndexMethodDef.body` remains a bodyful statement vector while methods/getters/setters use `MemberBody`.
12. `CallableId`, `CallableOwnerId::Declaration`, `TypeParameterOwner::{Declaration,Callable,Impl}`, `SelfTypeTerm`, `GenericSignature`, and `SemanticTargetId::{Declaration,Callable}` remain stable takeover seams.
13. `TypeLevelBinding` remains lexical binder infrastructure (`TypeForm` / `RecordRow`), not named declaration identity.
14. `DeclarationTypeTable` remains nominal/class-object type infrastructure and must not become trait generic storage.
15. `phalcom-semantic/src/checker/body.rs` still obtains declaration-owner generic parameters through `DeclarationTypeTable`, so trait defaults require a narrow owner-generic input generalization.
16. `CallableApplicationTarget` already carries optional `CallableId`, optional `InvocationTargetId`, and `CallTargetAuthority`, which is the correct seam for abstract semantic calls.
17. `SourceDeclarationKind` currently lacks a trait category; source navigation already has canonical declaration/callable targets.
18. Protocol-era active documents still assert `@protocol class`, signature-only protocols, structural conformance, and/or first-class Protocol descriptors.
19. The live C3 directory contains checkpoint, guidance, and the old plan but no historical requirements-analysis file; the fresh requirements analysis is therefore the authoritative requirements artifact for this revision.
20. The revised C1+C2 audit identifies C1-F01, C2-F01, and C2-F02 as strict pre-C3 blockers and recommends fixing C1-F02 while product identity work is reopened.

### Working-tree truth

This planning environment verifies remote branch/revision, not the future executor's local uncommitted state. T0 must record:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
```

and preserve unrelated modified/staged/untracked files.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read in this order before editing production source:

```text
1. AGENTS.md
2. docs/workflow/implementation-record-lifecycle-convention.md
3. docs/workflow/luna-patch-grade-plan-schema.md
4. docs/workflow/shared-consultation-escalation-protocol.md

5. docs/implementation/LANG005/LANG005.C3/LANG005.C3-CHECKPOINT.md
6. docs/implementation/LANG005/LANG005.C3/LANG005.C3-GUIDANCE.md
7. docs/implementation/LANG005/LANG005.C3/LANG005.C3.P1-requirements-analysis.md
8. this plan

9. docs/implementation/LANG005/LANG005.C1-data-and-unified-products/LANG005.C1-CHECKPOINT.md
10. LANG005.C1.P3 walkthrough
11. LANG005.C1.P3 handoff

12. docs/implementation/LANG005/LANG005.C2-inherent-impl-and-behavior/LANG005.C2-CHECKPOINT.md
13. LANG005.C2.P3 walkthrough
14. LANG005.C2.P3 handoff
15. C2.P3 incident record if still present/relevant

16. the revised C1+C2 audit supplied with this planning package
17. exact live source/test paths named by the current task
```

Before trait-spec migration, read the active specification authority/index document used by the repository and the protocol/trait conflict cluster named in T11.

For testing conventions, read:

```text
phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
```

Avoid broad repository exploration when the plan names the owners. Expand only when a task's verify-first assumption fails or an unexpected dependency is evidenced.

---

## 5. Normative authority

### 5.1 Normative language behavior

The accepted LANG005 taxonomy is fixed:

```text
anonymous tuple/record
    structural transparent immutable products

data
    nominal transparent immutable products

enum
    nominal closed sums of products

class
    opaque nominal object abstraction

trait
    reusable behavioral/conformance contract
    owns no instance representation
```

The governing separation remains:

```text
semantic category
!= exact type identity
!= behavior identity
!= runtime descriptor identity
!= physical representation
```

### 5.2 Accepted C3 design decisions

The following are normative for this plan:

- trait is a distinct named declaration, not `@protocol class` sugar;
- canonical declaration identity is existing `DeclarationId` plus `DeclarationKind::Trait`;
- only genuinely new semantic identities may be added;
- `TraitRequirementId` is distinct from the trait source/default callable and future target witness;
- trait source/default callable identity reuses `CallableId`;
- trait declaration generics use `TypeParameterOwner::Declaration`;
- member-local generics use `TypeParameterOwner::Callable`;
- `TraitRef` is a contract reference and is not automatically an inhabitable `TypeId`;
- trait generic metadata must not force fake nominal/class-object type metadata;
- `TraitSurface` is separate from `DeclarationSurface`, C2 conditional inherent surfaces, and enum contract products;
- every behavioral trait member defines a requirement;
- bodyful member = same requirement + optional default;
- complete surface publication precedes default-body checking;
- defaults are checked once under existing owner-relative `SelfTypeTerm`;
- abstract `Self` lookup reads `TraitSurface`;
- default calls reuse canonical callable application but have no executable runtime invocation target;
- default-to-default calls remain requirement-relative;
- index requirements use the shared index AST after `MemberBody` normalization;
- C3 owns no concrete conformance, witness selection, associated types, conformance constraints, trait objects, or runtime vtables;
- C3 does not allocate ordinary runtime trait classes or inject trait methods.

### 5.3 Current implementation

Current source determines mechanical APIs and file placement. It may be mechanically adapted without changing the rules above.

### 5.4 Audit authority

The revised C1+C2 audit owns the predecessor completion discrepancy for this plan. The audit does not redefine language semantics; it proves where landed source fails the predecessor invariant and therefore determines T1/T2 remediation ordering.

### 5.5 Historical/future material

Protocol-era typing documents are conflicting historical authority to be migrated, not the language rule to implement. C4–C8 support examples are forward-compatibility references, not C3 scope.

---

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| declaration identity | `phalcom_modules::DeclarationId` | module/declaration shell | trait remains in canonical named declaration identity universe |
| declaration category | `phalcom-modules/src/declaration.rs::DeclarationKind` | module layer | category metadata distinguishes Trait without a second ID universe |
| statement declarations | `phalcom-ast/src/ast.rs::Statement` | AST | C3 adds a true `Trait` statement, not an attribute on Class |
| behavior member syntax | `phalcom_ast::BehaviorMember` | AST | one shared method/getter/setter/index syntax category |
| member body representation | `MemberBody`; `IndexMethodDef.body` still exceptional | AST | T3 normalizes index body shape once for all consumers |
| callable identity | `phalcom_semantic::CallableId` | semantic identity | behavior identity remains owner + selector + side |
| callable declaration owner | `CallableOwnerId::Declaration` | semantic identity | trait source/default member needs no owner variant |
| impl provenance | `phalcom_semantic::ImplId` | C2 semantic impl domain | contribution identity remains distinct from behavior identity |
| conditional target | `InherentImplTarget::{Declaration, ExactEnumCase}` | C2 semantics | full target identity indexes conditional behavior |
| conditional member set | `ConditionalInherentMemberSet` | C2 semantics | immutable semantic applicability product; exact-case publisher must be repaired |
| conditional selection | `ConditionalDispatchSelection` | C2 checker | semantics chooses specialized behavior; runtime does not solve domains |
| effective ordinary surface | `DeclarationSurface` | semantic declaration behavior | trait surface never inserts into it |
| enum requirement identity | `EnumRequirementId` | enum contract semantics | closed enum requirement remains distinct from trait requirement |
| generic owner | `TypeParameterOwner` | semantic type system | Declaration/Callable owners already cover trait/header/member |
| generic signature | `GenericSignature` | semantic type system | canonical binder metadata reused |
| lexical binding | `TypeLevelBinding` | type resolver | remains lexical binder state only |
| Self | `SelfTypeTerm` | semantic type system | trait default uses canonical owner-relative Self |
| call target/application | `CallableApplicationTarget`, `CallTargetAuthority` | checker/call application | abstract call may have callable identity with `target=None` |
| source semantic target | `SemanticTargetId::{Declaration, Callable}` | source index | trait navigation reuses canonical identity |
| anonymous product lowering | `AnonymousProductConstructionLoweringSpec` | semantic→compiler lowering | T1 adds runtime type recipe projection rather than compiler re-inference |
| runtime type recipe/env | `RuntimeTypeRecipe`, `RuntimeTypeEnvironmentId`, registry, `instantiate_type_recipe` | core runtime typing | complete existing design; do not replace it |
| Record shape | `RecordProductShape` / shape registry | core product runtime | canonical logical shape distinct from presentation order |

### 6.2 Audit disposition takeover

The executor inherits this fixed disposition:

```text
INTEGRATE / PRECONDITION
    C1-F01 runtime reification closure
    C1-F02 dynamic Record canonicalization
    C2-F01 exact-case conditional publication
    C2-F02 exact-case conditional owner-complete invalidation

DEFER, RECORD
    C1-F03 data-component u32/u16 narrowing
    C1-F04 RecordView cache bypass
    C1-F05 ProductLayout quadratic validation
    C1-F06 ProductStorage/layout pairing hardening
    C2-F04 conditional compiler lookup scan
    C2-F05 coarse impl-domain invalidation

DEFER, HARD PRE-C4 CONDITION
    C2-F03 explicit proof-state granularity

ALREADY CLOSED / PRESERVE
    previous compile regression
    bound fallback GC rooting
    semantic-only editor applicability authority
    conditional selection body fingerprint
    no shared generic runtime-class contamination
    direct/bound shared runtime selection
```

No finding may be silently reclassified during implementation. If live evidence materially contradicts a disposition, STOP AND CONSULT.

---

## 7. Architecture

### 7.1 Overall authority flow

```text
language source
    ↓
AST/module declaration identity
    ↓
canonical semantic products
    ├─ trait header
    ├─ TraitRef
    ├─ TraitSurface
    └─ default CallableAnalysis
    ↓
stable lowering facts where runtime/compiler action is needed
    ↓
compiler
    ↓
bytecode/runtime metadata
    ↓
VM/runtime
```

The compiler, VM, and LSP never become parallel semantic authorities.

### 7.2 Takeover remediation flow — C1

```text
semantic exact anonymous-product TypeId
    ↓ project, do not reconstruct in compiler
RuntimeTypeRecipe
    Closed(exact runtime type)
    or Template(runtime type template)
    ↓
AnonymousProductConstructionLoweringSpec
    kind
    layout
    type_recipe
    ↓
current CallFrame RuntimeTypeEnvironmentId
    ↓
instantiate_type_recipe(...)
    ↓
canonical RuntimeTypeRef
    ↓
RuntimeAnonymousProductDescriptor.exact_type = Some(...)
```

Optimizer rematerialization must carry exactly the same `type_recipe` as eager construction.

Dynamic Record construction must use one shared canonicalization primitive:

```text
encounter/source labels
    ↓
presentation labels preserved
    ↓
canonical logical labels derived
    ↓
source/presentation -> logical permutation
    ↓
values stored in logical coordinates
    ↓
canonical shape registry identity
```

### 7.3 Takeover remediation flow — C2

```text
InherentImplContribution
    target = Declaration | ExactEnumCase
    ↓
conditional domain/member assembly by full InherentImplTarget
    ↓
ConditionalInherentMemberSet per target
    ↓
owner-complete snapshot replacement
    ↓
receiver-effective semantic lookup
    ↓
ConditionalDispatchSelection
    ↓
existing lowering/runtime projection
```

Publication and invalidation are one coherent operation. Do not publish exact-case keys and defer removal cleanup.

### 7.4 Trait declaration flow

```text
Statement::Trait(TraitDef)
    ↓
module interface binding
DeclarationId + DeclarationKind::Trait
    ↓
TraitHeader / TraitInfo
    generic signature
    ↓
identity allocation for all members
    TraitRequirementId
    trait-owned CallableId
    ↓
signature formation using scoped resolver
    ↓
complete TraitSurface
    ↓
bodyful defaults analyzed
    under abstract Self
```

### 7.5 Trait identity model

```text
trait declaration
    DeclarationId + kind Trait

contract requirement
    TraitRequirementId(owner, selector, side)

source/default callable
    CallableId(Declaration owner, selector, side)

future concrete witness
    target-owned CallableId

contract application
    TraitRef(declaration, canonical generic arguments)
```

No mandatory `TraitId`, `TraitMemberId`, or `TraitDefaultId`.

### 7.6 Generic ownership

```text
trait declaration generic T
    TypeParameterOwner::Declaration(trait DeclarationId)

trait member generic U
    TypeParameterOwner::Callable(trait-owned CallableId)

TraitHeader
    owns/references GenericSignature

DeclarationTypeTable
    remains nominal/class-object infrastructure only
```

### 7.7 Surface publication

```text
parse all behavior members
    ↓
allocate stable identities
    ↓
resolve every signature
    ↓
validate duplicate selector/side + legality
    ↓
publish TraitSurface
    ↓
check defaults
```

Default bodies do not participate in deciding whether later requirements exist.

### 7.8 Default analysis and abstract call architecture

```text
trait default body
    ↓
BodyAnalysisContext
    owner declaration = trait
    current callable = trait-owned CallableId
    owner generics = TraitHeader GenericSignature
    Self = SelfTypeTerm(trait owner)
    field/super capabilities = none
    abstract surface = TraitSurface
    ↓
trait-Self member lookup
    ↓
TraitRequirementId + CallableSemanticSignature
    ↓
canonical callable application checker
    ↓
CallableApplicationTarget
    callable = Some(trait-owned CallableId)
    target = None
    authority = TraitContract / AbstractContract equivalent
```

The call is semantically checked but not executable. C4 later supplies witness/default selection.

### 7.9 Incremental product model

At minimum:

```text
TraitHeader(DeclarationId)
TraitSurface(DeclarationId)
CallableAnalysis(trait-owned CallableId)
```

Header/surface fingerprints own structure; callable body fingerprint owns statements. Cold/incremental results must agree.

### 7.10 Compiler/runtime boundary

`Statement::Trait` is a compile-time/type-level declaration in C3. It must not create ordinary class allocation/finalization, storage, method injection, conformance registry, vtable, or runtime trait scan.

If module execution/export cannot represent a compile-time-only trait declaration without a runtime descriptor, STOP AND CONSULT instead of synthesizing a fake class.

---

## 8. Ownership boundaries

### 8.1 Owns

C3.P1 owns:

- integrated predecessor repairs C1-F01, C1-F02, C2-F01, C2-F02;
- trait token/AST/parser declaration shape;
- shared index `MemberBody` normalization required by trait requirements;
- module declaration category `Trait` and trait named binding/import/export;
- trait generic header product;
- `TraitRef` formation;
- `TraitRequirementId`;
- `TraitSurface` and complete signature publication;
- narrow owner-generic body-analysis generalization;
- abstract owner-relative `Self` lookup and contract-call authority;
- default-body checking once;
- incremental query/fingerprint integration;
- source index/LSP projection;
- compiler/runtime no-class boundary;
- protocol-era spec reconciliation;
- focused checkpoint verification and execution records.

### 8.2 Does not own

C3.P1 does not own:

- conformance syntax/proofs/witnesses/coherence;
- associated types/bindings/projection normalization;
- trait-conformance generic constraints/conditional conformances;
- trait objects/existentials/vtables;
- full reflection/runtime trait descriptors;
- final metatype conformance syntax;
- derived behavior;
- C1-F03/F04/F05/F06 remediation;
- C2-F04/F05 remediation;
- C2-F03 proof-state upgrade except as an explicit C4 prerequisite;
- workspace release certification.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| exact anonymous-product semantic type | semantic snapshot/type identity projected as `RuntimeTypeRecipe` | compiler/VM product materialization | compiler reconstruction from payload values |
| runtime generic environment | VM runtime type-environment registry/frame | recipe instantiation, captured blocks | per-product ad hoc generic maps |
| Record logical identity | shared product shape canonicalization | static/dynamic construction, equality/reflection | dynamic encounter-order logical identity |
| conditional exact-case member set | C2 semantic target-indexed conditional product | checker, lowering, tooling | checker/compiler/LSP-specific publisher |
| conditional target ownership/removal | C2 dispatch publication lifecycle | incremental snapshots | full-map cleanup scans as architectural fallback |
| trait declaration kind | module declaration shell | semantics/tooling/compiler no-op | parser-only trait flag |
| trait generic signature | trait header semantic product | signatures/defaults/TraitRef/C4 | fake nominal `DeclarationTypeTable` entry |
| trait requirement identity | semantic trait contract | TraitSurface/C4/tooling metadata | aliasing to source/default/witness `CallableId` |
| trait source/default callable | canonical `CallableId` | signature/body/source index | `TraitMemberId`/`TraitDefaultId` duplicates |
| TraitRef | semantic trait reference formation | future C4/C5/C6 | ordinary nominal `TypeId` surrogate |
| TraitSurface | semantic trait query/product | checker/defaults/C4/source presentation | `DeclarationSurface` mutation or LSP reconstruction |
| abstract trait call | checker canonical application | default diagnostics/analysis | executable `InvocationTargetId` before C4 |
| trait source navigation | `SemanticTargetId::Declaration/Callable` | source index/LSP | LSP-local trait IDs |
| compiler trait boundary | semantic declaration/lowering projection | compiler | compiler inference of contract semantics |

---

## 9. Global invariants

`INV-01` — Semantic category, exact type identity, behavior identity, runtime descriptor identity, and physical representation remain distinct.

`INV-02` — C1 anonymous products with statically exact generic semantic types materialize with the corresponding exact runtime type; shared code/layout never implies loss of exact instantiated type.

`INV-03` — Anonymous-product exact runtime type is projected from semantics as `RuntimeTypeRecipe`; it is never inferred from runtime payload values.

`INV-04` — Product optimizer rematerialization preserves the same runtime type recipe and exact type as eager construction.

`INV-05` — Dynamic and static Records with the same structural labels use the same canonical logical label order; presentation order remains independently preserved.

`INV-06` — C2 conditional behavior is indexed and published by full `InherentImplTarget`, including `ExactEnumCase`.

`INV-07` — Conditional target replacement/removal is owner-complete; incremental snapshots cannot retain exact-case products after owner/variant/domain deletion.

`INV-08` — Conditional applicability remains semantic authority; runtime and LSP do not re-solve impl domains.

`INV-09` — Specialized conditional behavior is never globally installed into the shared erased generic runtime class.

`INV-10` — Trait is a distinct declaration category and is not class sugar.

`INV-11` — Trait declaration identity is canonical `DeclarationId` plus verified `DeclarationKind::Trait`; there is no mandatory parallel `TraitId` universe.

`INV-12` — Trait requirement identity is first-class `TraitRequirementId` and is distinct from both trait source/default callable and future concrete witness callable.

`INV-13` — Trait source/default callable identity reuses canonical `CallableId`; no mandatory `TraitMemberId` or `TraitDefaultId` exists.

`INV-14` — Trait declaration generics use `TypeParameterOwner::Declaration`; member-local generics use `TypeParameterOwner::Callable`.

`INV-15` — Trait generic metadata does not create a fake nominal type/class-object entry in `DeclarationTypeTable`.

`INV-16` — `TypeLevelBinding` remains lexical generic-binder infrastructure; named trait declaration resolution uses canonical declaration lookup.

`INV-17` — `TraitRef` is trait declaration identity plus canonical generic arguments, not an inhabitable `TypeId`, runtime trait object, or conformance proof.

`INV-18` — `TraitSurface` is distinct from ordinary `DeclarationSurface`, C2 conditional inherent surfaces, and enum behavior products.

`INV-19` — Every behavioral trait member defines a requirement; a bodyful member is the same requirement plus default availability.

`INV-20` — Requirement identity and trait source callable identity survive bodyless↔bodyful transition when selector/side is unchanged.

`INV-21` — Shared `IndexMethodDef` supports declaration-only bodies after normalization; no trait-only parallel index AST exists.

`INV-22` — C3 does not redesign index setter result semantics.

`INV-23` — All trait member signatures publish before any default body is analyzed; source order cannot control contract availability.

`INV-24` — Trait defaults are analyzed once under canonical owner-relative `SelfTypeTerm`.

`INV-25` — Trait `Self` lookup reads completed `TraitSurface`, not concrete declaration surfaces or future conformers.

`INV-26` — Calls from trait defaults reuse canonical argument/generic application checking.

`INV-27` — Abstract trait requirement applications have semantic callable identity but no executable `InvocationTargetId`.

`INV-28` — Default-to-default calls remain contract-relative and do not statically bind to the default body.

`INV-29` — Trait owns no fields, stored components, product layout, superclass edge, constructor-managed storage, or ordinary runtime class.

`INV-30` — Enum closed requirements remain `EnumRequirementId` semantics; they are not converted into traits.

`INV-31` — Trait header, TraitSurface, and default-body analysis have separable fingerprints/dependencies; body-only edits do not invalidate structural identity unnecessarily.

`INV-32` — Cold and incremental analysis agree for trait products and for repaired exact-case conditional target lifecycle.

`INV-33` — Trait source tooling reuses canonical semantic declaration/callable targets; LSP creates no parallel semantic identity or solver.

`INV-34` — Compiler/runtime ordinary dispatch remains trait-unaware in C3; no trait class, method injection, conformance registry, witness table, or VM trait scan is introduced.

`INV-35` — Active normative specification has one effective behavioral-contract model: `trait`; protocol-era language-feature claims are superseded/migrated rather than coexisting as a competing rule.

`INV-36` — C2-F03 proof-state granularity is not silently forgotten: it is recorded as a hard prerequisite before C4 witness/conformance implementation begins.

---

## 10. Non-goals

Do not implement:

```text
C4
    impl Trait for Target
    conformance proofs
    structural/nominal conformance inference
    witness/default selection
    coherence/overlap

C5
    associated types
    associated bindings
    projection normalization

C6
    GenericConstraint::Conforms
    T: Trait / conforms bounds
    conditional conformances

C7
    trait objects/existentials
    runtime trait descriptors as semantic authority
    conformance reflection registry
    witness/vtable runtime dispatch

C8
    derived behavior policy
```

Also do not:

- settle final metatype/class-object conformance syntax;
- introduce class-side trait semantics without newer ratified authority;
- redesign selector/family semantics;
- redesign index setter return semantics;
- fix C1-F03/F04/F05/F06 as incidental cleanup;
- fix C2-F04/F05 as incidental cleanup;
- implement C2-F03 proof-state redesign inside C3 unless a C3 requirement unexpectedly proves it necessary, in which case STOP AND CONSULT;
- run or repair broad release certification as ordinary BUILD work.

---

## 11. Expected impact map

The exact set may mechanically drift, but the following areas are expected.

### 11.1 Audit remediation — C1

- `phalcom-core/src/modules/semantic_lowering.rs` — add canonical runtime type recipe to anonymous-product lowering projection.
- `phalcom-core/src/typing/environment.rs` — consume existing recipe/environment API; extend only as mechanically required.
- `phalcom-core/src/frame.rs` — verify/populate runtime type environment on generic call entry if the missing producer is here.
- `phalcom-core/src/heap/block.rs` — preserve lexical type environment on escaped blocks; normally verification/mechanical adjustment only.
- `phalcom-core/src/vm/send.rs` / `phalcom-core/src/vm/dispatch.rs` — propagate runtime type environments through callable/block activation and instantiate recipe at static product materialization as appropriate.
- `phalcom-core/src/product/mod.rs` — descriptor materialization with exact runtime type; dynamic Record canonicalization.
- `phalcom-core/src/product/shape.rs` — shared logical Record canonicalization helper if needed.
- `phalcom-core/src/compiler/lib/expr.rs` and static product construction paths — carry lowering recipe rather than re-infer.
- `phalcom-core/src/compiler/lib/product_opt.rs` plus tests — preserve recipe during virtual-product rematerialization.
- focused product/runtime/compiler tests.

### 11.2 Audit remediation — C2

- `phalcom-semantic/src/impls.rs` — assemble conditional products by full target or expose the canonical target-grouped product builder.
- `phalcom-semantic/src/session.rs` — publish all target-indexed conditional sets.
- `phalcom-semantic/src/dispatch.rs` — owner-complete target replacement/removal.
- existing checker/associated/tooling consumers — preferably verification-only; no duplicated exact-case logic.
- `phalcom-semantic/tests/semantic/impls/` — specialized exact-case semantic/incremental regressions.
- core runtime/lowering tests only where required to prove direct/bound/fallback coherence.

### 11.3 Trait AST/module surface

- `phalcom-ast/src/token.rs`
- `phalcom-ast/src/lexer.rs`
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- AST parser/range tests, including new `trait_syntax` integration target if repository convention permits.
- `phalcom-modules/src/declaration.rs`
- `phalcom-modules/src/interface.rs`

### 11.4 Trait semantics

- `phalcom-semantic/src/identity.rs` or a trait-specific semantic module for `TraitRequirementId` export.
- create `phalcom-semantic/src/traits.rs` or repository-equivalent module owning trait header/ref/surface products.
- `phalcom-semantic/src/lib.rs` exports.
- `phalcom-semantic/src/checker/declaration_signature.rs`
- `phalcom-semantic/src/checker/body.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/semantic_shard.rs`
- `phalcom-semantic/src/db/` query/fingerprint/dependency code as required.
- `phalcom-semantic/src/source_index/` builder/scope/symbol/occurrence code.
- `phalcom-semantic/tests/semantic/capabilities/traits.rs` or repository-equivalent new trait capability module.
- `phalcom-semantic/tests/semantic/incremental/` focused trait invalidation cases.

### 11.5 Compiler/runtime boundary

- `phalcom-core/src/compiler/` top-level statement dispatch and AST exhaustive matches.
- `phalcom-core/src/modules/semantic_lowering.rs` only if a trait no-op declaration projection is needed.
- `phalcom-core/src/compiler/lib/product_opt.rs` / AST walkers only for new `Statement::Trait` exhaustiveness.
- `phalcom-core/tests/core/language/traits.rs` or repository-equivalent focused source-language test module.

### 11.6 LSP/tooling

- `phalcom-semantic/src/source_index/scope.rs::SourceDeclarationKind`
- semantic source-index symbol mapping.
- `phalcom-lsp/` only if normal semantic source-index projection does not already cover workspace symbols/tokens; no LSP-local trait semantic implementation.

### 11.7 Specification/state

- `docs/implementation/LANG005/LANG005.C3/LANG005.C3-CHECKPOINT.md`
- `docs/implementation/LANG005/LANG005.C3/LANG005.C3.P1-walkthrough.md`
- `docs/implementation/LANG005/LANG005.C3/LANG005.C3.P1-handoff.md`
- `docs/spec/typing/README.md`
- `docs/spec/typing/STATUS.md`
- `docs/spec/typing/01-protocol-foundation.md`
- `docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md`
- `docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md`
- `docs/spec/typing/03-type-parameters-and-generic-signatures.md`
- `docs/spec/next/phalcom-meta-dispatch-and-type-extension-spec.md` if still active/conflicting under spec governance.

### Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary. STOP AND CONSULT before entering a materially different subsystem not anticipated here when that entry changes architecture, ownership, identity, runtime representation, incremental authority, or language behavior.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

- the audit disposition in §6.2 without consultation;
- the existing C1 runtime recipe/environment architecture; T1 completes it rather than replacing it;
- exact type must come from semantic projection, never payload inference;
- logical Record identity is canonical and independent of presentation order;
- C2 conditional products are keyed by full target identity and owned/invalidation-managed semantically;
- trait is not a class and owns no representation;
- declaration identity remains `DeclarationId` + kind Trait;
- no mandatory `TraitId`, `TraitMemberId`, or `TraitDefaultId`;
- requirement identity is distinct from source/default/witness callable identity;
- `TraitRef` is not an ordinary inhabitable `TypeId`;
- trait generic metadata is not stored by fabricating nominal `DeclarationTypeTable` entries;
- `TraitSurface` is not inserted into concrete/inherent/conditional/enum surfaces;
- bodyful member = same requirement + default;
- complete surface precedes default body checking;
- abstract `Self` uses existing `SelfTypeTerm`;
- abstract calls have no executable runtime target;
- default-to-default calls remain contract-relative;
- C3 implements no conformance/witness/associated-type/conformance-bound machinery;
- index declaration support uses shared AST normalization, not trait-only syntax;
- no runtime trait class/method injection/scan/vtable in C3;
- source tooling reuses canonical semantic targets;
- C2-F03 remains a hard C4 precondition if not repaired before handoff.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt without consultation:

- private helper names;
- exact trait semantic module name (`traits.rs`, `trait_semantics.rs`, etc.);
- whether a checked one-to-one trait declaration wrapper improves API safety;
- map/set implementation details;
- internal query-key spelling;
- exact `CallTargetAuthority` variant spelling (`TraitContract`, `AbstractContract`, equivalent);
- private body-analysis context field name for owner generics;
- small file/module decomposition;
- imports and mechanical exhaustive-match updates;
- exact diagnostic codes following repository conventions;
- test file/module organization consistent with existing test READMEs;
- Interface-like LSP presentation kind where protocol does not define Trait.

Mechanical flexibility is not permission to create duplicate semantic owners.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| local checkout is on expected branch/revision and unrelated changes exist or not | T0 git commands | preserve unrelated changes; mechanical descendant okay; material semantic drift -> consult |
| C1-F01 is still open | `AnonymousProductConstructionLoweringSpec`, static materializers, generic frame env producer | if already fixed equivalently, verify tests and mark task slice already closed; if different architecture, consult |
| C1-F02 is still open | `product/mod.rs::finish_record` / shape helpers | if already canonicalized, verify cross-route identity test and skip edit |
| C2-F01/F02 remain open | `session.rs`, `dispatch.rs`, exact-case product producer | if already fixed equivalently, verify lifecycle tests; if publisher ownership moved, map mechanically or consult if semantic ownership changed |
| `DeclarationKind::Protocol` has no stable ABI/serialization contract | repository search + generated metadata/cache code | if stable external compatibility exists, STOP AND CONSULT |
| `BehaviorMember` remains reusable | `phalcom-ast/src/ast.rs` and parser helpers | if trait behavior needs parallel member AST, STOP AND CONSULT |
| `IndexMethodDef.body` still lacks `MemberBody` | AST | if predecessor already normalized safely, omit T3 normalization and verify existing tests |
| trait owner generics still unavailable to body analysis except via `DeclarationTypeTable` | `checker/body.rs`, call sites | if a canonical explicit owner-generic seam already exists, reuse it |
| canonical signature builder still accepts an explicit resolver | `checker/declaration_signature.rs` | adapt exact helper; if trait-specific signature solving seems required, consult |
| `CallableApplicationTarget` still supports `target=None` and authority | `checker/call.rs` | adapt mechanically; if execution identity is mandatory for all applications, consult |
| module named declarations can be compile-time-only at runtime | module interface/compiler type-alias precedent | if runtime export/linkage requires object identity, STOP AND CONSULT |
| source index can add Trait presentation without new `SemanticTargetId` | source-index modules | if navigation requires new identity for semantics, consult |
| protocol-era documents are active according to current spec governance | spec README/index | migrate only effective/conflicting language-feature claims; archive/mark historical according to governance |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing the affected issue and prepare a consultation packet when any of the following occurs:

1. an integrated audit finding cannot be reconciled with current source semantics;
2. the audit oracle appears materially wrong;
3. C1 exact runtime reification appears to require replacing rather than completing the existing recipe/environment design;
4. runtime generic environment ownership/lifetime/GC semantics must change materially;
5. C2 exact-case conditional publication cannot be made target-indexed at the semantic owner;
6. owner-complete invalidation requires a global scan or different incremental ownership architecture rather than a bounded target ownership model;
7. a C1/C2 invariant would need to be weakened to make C3 work;
8. trait requires a fake class, nominal type, class object, or runtime descriptor merely to store semantics;
9. declaration/callable/generic identity ownership must change;
10. a new identity type appears necessary for semantics rather than category-safety convenience;
11. trait generic metadata can only be exposed through `DeclarationTypeTable` nominal machinery;
12. `TraitSurface` appears to require insertion into `DeclarationSurface`, conditional inherent surfaces, or enum behavior products;
13. default checking requires a concrete conformer/witness/conformance proof;
14. abstract calls require an executable runtime `InvocationTargetId` before C4;
15. canonical callable application cannot represent an abstract contract application without a second argument/generic solver;
16. index declaration-only support would require trait-only AST rather than safe shared normalization;
17. associated types or trait-conformance constraints become necessary for P1 core;
18. final metatype/class-object conformance semantics must be chosen;
19. module linkage requires a runtime trait object/descriptor in C3;
20. runtime representation/reflection boundary must change;
21. incremental DB architecture must be redesigned rather than extended with ordinary products/dependencies;
22. LSP requires its own trait semantic reasoning rather than consuming source-index/semantic products;
23. a newer canonical spec contradicts the fixed C3 architecture;
24. two plausible fixes have materially different architecture;
25. an unexpected subsystem becomes architecturally necessary;
26. the same nontrivial semantic failure persists after one serious hypothesis-driven correction;
27. a test passes only after weakening a fixed invariant;
28. scope becomes materially more cross-cutting than this plan predicts.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

One cycle is:

```text
inspect exact compiler/test failure
identify concrete mechanical cause
make one coherent correction
rerun smallest discriminating command
```

Mechanical examples include imports, renamed helpers, borrow/type cleanup, exhaustive matches, test module registration, and incorrect filters.

After roughly three cycles without convergence, reassess whether the issue is actually semantic/architectural.

### Semantic failures

Before a nontrivial semantic correction, record in local notes/consultation packet:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If substantially the same failure persists:

```text
STOP AND CONSULT
```

### Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test merely to see whether it changes.

---

## 15. Testing surface analysis

The requirements analysis defines the complete coverage surface. The implementation may add equivalent test names, but each coverage ID below must have explicit evidence or a documented justified omission.

### 15.1 Audit-remediation coverage

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| `AR-01` | generic runtime reification | INV-02, INV-03 | generic `make<T>` creates anonymous products for distinct concrete `T` with distinct correct exact runtime product types while sharing code/layout where appropriate |
| `AR-02` | nested recipe | INV-02, INV-03 | nested applied/tuple/record/callable/union recipe supported by current runtime instantiates canonically |
| `AR-03` | lexical lifetime | INV-02 | escaped block uses captured runtime type environment to create a correctly typed product after lexical parent returns; include GC stress if existing harness supports it cheaply |
| `AR-04` | optimizer differential | INV-04 | optimized/rematerialized and eager construction expose identical exact type |
| `AR-05` | Record identity | INV-05 | static/dynamic Record permutations share canonical logical shape/equality/hash while preserving presentation order |
| `AR-06` | exact-case publication | INV-06, INV-08 | specialized exact case applies for matching receiver and not wrong args/sibling/root |
| `AR-07` | direct/bound coherence | INV-08, INV-09 | repaired exact-case product drives direct and bound/family access without shared-root leakage |
| `AR-08` | owner-complete invalidation | INV-07, INV-32 | add/edit/delete constraint/variant/owner and compare cold vs incremental exact-case conditional sets |
| `AR-09` | editor semantic authority | INV-08 | tooling lookup/completion follows semantic exact-case applicability; no editor solver |

### 15.2 Trait declaration/identity coverage

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| `TR-01` | syntax/ranges | INV-10 | canonical `trait` declaration parses with correct ranges |
| `TR-02` | module identity | INV-10, INV-11 | trait is named/importable/exportable and namespace collision behavior matches other declarations |
| `TR-03` | generic header | INV-14, INV-15 | generic trait binders owned by declaration and no nominal trait table entry |
| `TR-04` | TraitRef | INV-16, INV-17 | valid generic/non-generic refs plus wrong kind/arity/declaration category cases |
| `TR-05` | no nominal trait type | INV-15, INV-17 | ordinary value-type use rejected/deferred; no class-object/nominal form created |
| `TR-06` | requirement identity | INV-12, INV-13 | bodyless member has distinct requirement and source callable identities |
| `TR-07` | default identity | INV-12, INV-13, INV-19 | bodyful member keeps requirement identity and attaches default to source callable |
| `TR-08` | behavior shapes | INV-19, INV-21 | bodyless method/getter/setter/index getter/index setter all publish valid requirement signatures |
| `TR-09` | duplicate selector | INV-18, INV-19 | duplicate selector+side diagnosed independent of annotation differences |
| `TR-10` | visibility/source | INV-18 | TraitSurface retains visibility and source provenance |

### 15.3 Default/Self coverage

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| `DF-01` | Self signature | INV-24 | supported trait signature uses canonical owner-relative Self |
| `DF-02` | generic default | INV-14, INV-15, INV-24 | default resolves trait declaration and member-local generic parameters without nominal trait entry |
| `DF-03` | requirement call | INV-25, INV-26, INV-27 | default calls bodyless requirement; semantic application has no runtime target |
| `DF-04` | default call | INV-28 | default calls bodyful member contract-relatively, not directly to default body |
| `DF-05` | source-order independence | INV-23 | default calls later-declared member successfully |
| `DF-06` | canonical diagnostics | INV-26 | argument/generic/return mismatch uses existing callable diagnostics |
| `DF-07` | no storage | INV-29 | field/stored-component access in default rejected |
| `DF-08` | no super | INV-29 | super dispatch in default rejected |
| `DF-09` | class-side boundary | INV-10, INV-34 | unsupported class-side trait member gets stable deferred/unsupported diagnostic rather than metatype semantics |

### 15.4 Incremental/tooling/compiler/spec coverage

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| `IN-01` | body-only edit | INV-20, INV-31 | body edit preserves requirement/callable/signature identities and unrelated surface products |
| `IN-02` | signature edit | INV-31 | signature edit invalidates TraitSurface/dependents but not unrelated traits |
| `IN-03` | body presence edit | INV-20, INV-31 | bodyless↔bodyful preserves IDs but changes default availability/body product |
| `IN-04` | header generic edit | INV-14, INV-31 | TraitRef/surface/default dependents invalidate correctly |
| `IN-05` | cold parity | INV-32 | representative edit sequences produce identical cold/incremental products/diagnostics |
| `LS-01` | source targets | INV-33 | trait definition -> `SemanticTargetId::Declaration`; member -> `::Callable` |
| `LS-02` | symbol presentation | INV-33 | workspace/source symbol is Trait/interface-like with no local semantic identity |
| `CP-01` | compiler no-op | INV-29, INV-34 | module containing trait compiles without ordinary runtime class/finalization |
| `CP-02` | runtime non-discovery | INV-18, INV-34 | trait members are not ordinary methods on unrelated/by-shape classes in C3 |
| `CP-03` | enum separation | INV-30 | enum requirement tests continue to use enum identity/product path |
| `CP-04` | conditional separation | INV-18 | C2 conditional inherent lookup remains separate from TraitSurface |
| `SP-01` | spec authority | INV-35 | active README/STATUS/effective docs no longer present `@protocol class` signature-only model as canonical |
| `HD-01` | C4 prerequisite | INV-36 | handoff/checkpoint explicitly records C2-F03 proof-state closure requirement before C4 implementation |

### 15.5 Shared index regression coverage

T3 must prove:

```text
bodyful class index getter unchanged
bodyful class/impl index setter unchanged
bodyless trait index getter represented as MemberBody::Declaration
bodyless trait index setter represented likewise
bodyless index rejected in contexts where abstract declaration is illegal
fingerprints/body extraction handle declaration vs block consistently
```

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Default during tasks. Run only exact/new regressions and the smallest compile/test target that answers the current question.

#### STABILIZE MODE

Enter at G1–G5. Run focused regressions plus directly affected owning suites.

#### CERTIFY MODE

Enter only at G6. Run the plan's focused final acceptance and classification checks. Do not automatically escalate to full workspace release gates.

### 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad affected crate
T5 — workspace/release
```

A PASS does not automatically authorize climbing the ladder.

### 16.3 Mandatory during BUILD

- compile the directly edited crate/module only when useful;
- run exact new regression(s) after a coherent implementation slice;
- confirm filtered commands select nonzero tests;
- run Cargo commands serially when concurrent builds would waste resources.

### 16.4 Mandatory at gates

Use the exact commands in §19, adapting only mechanical test-module names after verifying them with `-- --list`.

### 16.5 Mandatory during STABILIZE

At minimum, by checkpoint end:

```text
focused C1 product/reification regressions
focused C2 semantic impl/exact-case regressions
phalcom-ast trait/index syntax target
phalcom-modules trait declaration tests
phalcom-semantic trait capability tests
phalcom-semantic trait incremental tests
focused core trait non-runtime-class tests
focused existing enum/conditional separation regressions
format check classification
```

### 16.6 Only if evidence demands

- broad `cargo test -p phalcom-semantic --test semantic`;
- broad `cargo test -p phalcom-core --test core`;
- full crate builds beyond affected crates;
- LSP-specific test suite if LSP code itself changes rather than normal semantic projection;
- workspace build/test/clippy/Nextest.

### 16.7 Explicitly deferred / DO NOT RUN repeatedly during BUILD

```text
cargo test --workspace --all-targets
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
full language corpus
unrelated REPL suites
unrelated concurrency suites
unrelated optimizer suites outside T1 product rematerialization
```

Reason: the audit already records repository-wide release-gate debt, and these commands do not provide better feedback than focused tests during coherent C3 implementation slices.

---

## 17. Baseline / unrelated failure policy

Classify unexpected failures:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility; diagnose and fix within the task's bounded debugging budget.
- C: one bounded classification pass. If still unclear, nonblocking, and outside active acceptance, record in checkpoint and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks proof of the active checkpoint.
- Do not weaken/skip assertions or change expected semantics to make a broad gate green.
- A planned-red intermediate feature gap is not a baseline failure; track it as an intentionally incomplete task state and rerun when the owning task lands.

Known audit-time broad baseline/release issues include Nextest orchestration/inventory and workspace rustfmt/Clippy failures. Reconfirm before classifying any current failure as D; do not assume an old baseline automatically remains identical.

---

# 18. Tasks

## T0 — Final takeover, local-state capture, and audit disposition lock

### Purpose

Replace the stale pre-final-C2 C3 lifecycle state with truthful current takeover state, verify the audit findings that gate trait work, and establish the exact local execution baseline without changing production semantics.

### Preconditions

- user-authorized implementation session for this plan;
- requirements analysis and this plan available;
- current C1/C2 walkthrough/handoff/audit available.

### Consumes

- final C1/C2 checkpoint records;
- C1.P3 and C2.P3 walkthrough/handoff;
- audit disposition in §6.2;
- live source facts in §3.

### Produces

- recorded local branch/revision/worktree state;
- current C2 takeover symbol map in `LANG005.C3-CHECKPOINT.md`;
- P1 marked active;
- stale pre-final-C2 blocker removed from C3 checkpoint lifecycle;
- audit remediation/deferred ledger recorded;
- objective statement that trait source edits are gated on G1.

### Files and symbols

**Read:**
- `AGENTS.md`
- workflow docs named in §4
- C1/C2 checkpoint/walkthrough/handoff
- `phalcom-core/src/modules/semantic_lowering.rs::AnonymousProductConstructionLoweringSpec`
- `phalcom-core/src/product/mod.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/dispatch.rs`
- `phalcom-semantic/src/impls.rs`

**Modify:**
- `docs/implementation/LANG005/LANG005.C3/LANG005.C3-CHECKPOINT.md`

**Create:** none.

**Tests:** none required unless live drift makes a cheap exact verification necessary.

### Required implementation shape

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
```

Preserve unrelated modified/staged/untracked files.

Verify the four integrated findings against live source:

```text
C1-F01 open or equivalently already fixed
C1-F02 open or equivalently already fixed
C2-F01 open or equivalently already fixed
C2-F02 open or equivalently already fixed
```

Verify final C2 stable products by repository-equivalent names:

```text
ConditionalInherentMemberSet
InherentImplTarget
ConditionalDispatchSelection
receiver-effective conditional lookup
semantic lowering projection
shared VM conditional selection
```

Update checkpoint state to reflect:

```text
C2.P3 is no longer the blocker
P1 is active
G1 predecessor remediation is the internal implementation gate
audit findings have explicit dispositions
C2-F03 is hard pre-C4 debt
```

Do not edit trait production code yet.

### Forbidden approaches

- do not reset/clean/revert unrelated work;
- do not rewrite C1/C2 implementation records wholesale;
- do not silently mark audit blockers “accepted” because checkpoints say complete;
- do not start trait parsing before G1;
- do not broaden into general C1/C2 cleanup.

### Test changes required

None.

### Tests to run now

No runtime/integration tests are required. This task is takeover/state verification.

If a finding appears already fixed, run only the exact existing/new regression needed to prove that fact before skipping its edit task.

### Tests explicitly deferred

All trait tests and broad predecessor suites.

### Acceptance

- local state is recorded;
- live symbol map is known;
- checkpoint no longer claims C3 is blocked by unimplemented C2.P3;
- all audit findings have the plan dispositions in §6.2;
- trait production edits remain gated by G1.

### Local STOP / CONSULT triggers

- current main/checkout materially changes C1/C2 semantic ownership;
- any integrated audit finding cannot be reproduced and no equivalent fix is evident;
- C2 central architecture differs materially from final handoff;
- local checkout contains conflicting user edits in exactly the files T1/T2 must change and safe preservation requires architectural choice.

### Checkpoint update

Mandatory. Record revision, worktree summary without sensitive/unnecessary detail, active plan, takeover map, audit disposition, and next action `T1`.

---

## T1 — C1 exact runtime reification and Record logical identity closure

### Purpose

Complete the C1 anonymous-product exact runtime type path and canonicalize dynamic Record logical shape so C3 inherits a truthful semantic-identity/representation boundary.

### Preconditions

- T0 complete;
- C1-F01/F02 still open or task has equivalent scoped deltas;
- no trait production source edits have begun.

### Consumes

- semantic exact anonymous-product `TypeId` available in snapshot/lowering projection;
- existing `RuntimeTypeRecipe` and runtime type-environment registry;
- current frame/block type-environment fields;
- current anonymous-product shape/layout/materialization architecture;
- static Record logical/presentation/permutation behavior as canonical oracle.

### Produces

- `AnonymousProductConstructionLoweringSpec` carries canonical `RuntimeTypeRecipe` for statically known anonymous products;
- shared generic callable entry creates/selects the correct interned runtime type environment for actual specialization;
- product materialization instantiates recipe and registers descriptor with `exact_type: Some(...)` whenever semantics requires an exact type;
- optimizer rematerialization preserves same recipe;
- dynamic Record construction canonicalizes logical labels/permutation like static construction;
- AR-01..AR-05 focused regressions.

### Files and symbols

**Read:**
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/typing/environment.rs`
- `phalcom-core/src/frame.rs`
- `phalcom-core/src/heap/block.rs`
- `phalcom-core/src/vm/send.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/product/mod.rs`
- `phalcom-core/src/product/shape.rs`
- `phalcom-core/src/compiler/lib/expr.rs`
- `phalcom-core/src/compiler/lib/product_opt.rs`
- C1.P3 handoff/walkthrough and current product tests.

**Modify:** repository-equivalent subset of the above required by the actual call/materialization path.

**Create:** no new subsystem; focused tests may be added in existing core product/language/compiler test modules.

**Tests:** product runtime, runtime typing, product optimizer, source-language generic product regressions.

### Required implementation shape

#### A. Semantic-to-runtime recipe projection

Extend the static anonymous-product lowering product conceptually:

```rust
pub struct AnonymousProductConstructionLoweringSpec {
    pub kind: AnonymousProductConstructionKind,
    pub layout: ProductLayoutSpec,
    pub type_recipe: RuntimeTypeRecipe,
}
```

If a closed exact type is known:

```text
RuntimeTypeRecipe::Closed(canonical_runtime_type)
```

If declaration/callable type parameters remain:

```text
RuntimeTypeRecipe::Template(canonical_runtime_type_template)
```

The exact semantic product type must be the source. Do not derive the recipe from AST syntax or runtime component values in the compiler/VM.

If the semantic→runtime metadata projection helper already exists for Data/enum/generic types, reuse it. Do not create a second type graph encoder.

#### B. Runtime type environment production

Trace the actual generic call-entry path. The existing registry/frame/block fields are infrastructure, not proof of use. At a shared generic callable activation, establish the interned runtime environment corresponding to the actual semantic generic specialization entering the callable.

Nongeneric paths continue to use `RuntimeTypeEnvironmentId::EMPTY` with no unnecessary map allocation.

Captured blocks preserve the environment already expected by `BlockObject`/frame machinery.

If a template requires a parameter that the runtime specialization cannot supply, fail closed with a structured internal/runtime error; do not silently register `exact_type: None` for a statically exact value.

#### C. Materialization

Static Tuple/Record finalization must:

```text
read spec.type_recipe
read current frame type environment
instantiate_type_recipe
obtain canonical RuntimeTypeRef
register/find anonymous product descriptor including exact_type
materialize storage
```

Closed recipes may avoid environment lookup where the existing API permits.

#### D. Optimizer equivalence

Any virtual product record/rematerialization state must retain the same `type_recipe` or the full static construction spec. The optimized path may not rematerialize a product with `exact_type=None` when the eager path would attach exact type.

#### E. Dynamic Record canonicalization

Refactor dynamic Record finalization to use the same logical canonicalization primitive as static Record lowering/materialization:

```text
presentation labels = encounter order
logical labels = canonical structural order
presentation_to_logical/source_to_logical = computed permutation
values stored by logical coordinates
```

Do not sort source evaluation. Labels/values are already evaluated before shape finalization; only structural storage coordinates change.

### Forbidden approaches

- no runtime payload-type inference for exact product type;
- no per-instance generic argument arrays/maps;
- no runtime class per generic application;
- no compiler-only reconstruction of semantic `TypeId` facts;
- no optimizer special case that omits type recipe;
- no change to Record source evaluation or presentation order;
- no unrelated C1-F03/F04/F05/F06 cleanup.

### Test changes required

Add/extend tests for `AR-01` through `AR-05`.

Required exact named scenarios should include repository-equivalent cases such as:

```text
generic_anonymous_product_materializes_distinct_exact_runtime_types
nested_generic_product_recipe_instantiates_recursively
escaped_block_preserves_runtime_type_environment_for_product_materialization
product_optimizer_rematerialization_preserves_exact_runtime_type
dynamic_record_permutations_share_canonical_logical_shape
static_and_dynamic_record_construction_share_logical_identity
```

### Tests to run now

First discover exact current filters:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core -- --list | rg 'product|record|generic|runtime_type'
RUSTFLAGS='' cargo test -p phalcom-core --lib -- --list | rg 'product_opt|typing::environment|product'
```

Run exact new regressions first. Then, if they pass, run the smallest owning focused product/runtime-type and product-optimizer filters selected by the listing.

Do **not** run the full workspace.

### Tests explicitly deferred

- unrelated optimizer suite;
- full `phalcom-core` integration binary;
- workspace build/test/clippy/Nextest;
- C1-F03+ cleanup tests.

### Acceptance

- AR-01..AR-05 pass;
- no static exact generic anonymous product materializes with missing exact type;
- generic frame environment is actually produced, not only propagated;
- escaped lexical block retains correct environment;
- eager/optimized product paths agree;
- dynamic/static Record routes share canonical logical identity without changing presentation/evaluation order.

### Local STOP / CONSULT triggers

- semantic snapshot cannot project an exact/product template without new semantic identity architecture;
- generic call entry lacks enough specialization information and obtaining it would change compiler/runtime call contracts materially;
- runtime environment rooting/lifetime becomes nonlocal or uncertain;
- optimizer would require a new representation semantics rather than carrying existing spec/recipe;
- Record canonicalization changes observable source order/presentation.

### Checkpoint update

After focused acceptance, record closure of C1-F01 and C1-F02 plus the stable recipe/materialization invariant. Do not mark G1 complete yet; T2 remains.

---

## T2 — C2 exact-case conditional product publication and owner-complete lifecycle

### Purpose

Close the missing semantic producer/lifecycle for conditional exact-enum-case behavior without changing the correct C2 applicability/lowering/runtime architecture.

### Preconditions

- T0 complete;
- T1 may be complete or independent worktree sequencing is safe, but G1 awaits both;
- final C2 P3 semantics verified.

### Consumes

- `InherentImplTarget::{Declaration, ExactEnumCase}`;
- `ConditionalInherentMemberSet`;
- inherent impl contributions/domains;
- `SurfaceDispatchResolver` conditional target map;
- existing receiver-effective lookup and `ConditionalDispatchSelection`;
- existing lowering/runtime direct/bound selection.

### Produces

- one canonical conditional member set per full target;
- exact-case conditional target sets published to semantic dispatch snapshot;
- owner-complete replacement/removal for every target key owned by a declaration/module lifecycle unit;
- cold/incremental parity for exact-case conditional changes;
- AR-06..AR-09 regressions.

### Files and symbols

**Read/Modify:**
- `phalcom-semantic/src/impls.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/dispatch.rs`
- exact-case receiver-effective checker/associated lookup consumers as verification points
- semantic shard/query ownership code only if lifecycle ownership is recorded there
- `phalcom-semantic/tests/semantic/impls/` relevant applicability/exact-case/incremental modules

**Verify, avoid semantic duplication:**
- `phalcom-core/src/modules/semantic_lowering.rs`
- conditional compiler/runtime tests
- LSP/source-index conditional lookup path.

### Required implementation shape

#### A. Build products by full target

Do not force exact-case contributions through declaration-root `build_effective_surface` if that product intentionally excludes exact cases. Introduce or reuse a target-grouped conditional assembly boundary:

```text
DeclarationId owner
    ↓ contributions
Map<InherentImplTarget, ConditionalInherentMemberSet>
    Declaration(owner) -> set
    ExactEnumCase(variant) -> set
    ...
```

Unconditional exact-case behavior remains on the existing P2 exact-case path. Conditional exact-case behavior remains outside enum-root `DeclarationSurface`.

#### B. Publish atomically

Session publication must replace all target-indexed conditional products for the owning declaration/revision, not just the declaration-root key.

#### C. Owner-complete removal

Maintain a bounded reverse ownership structure or equivalent:

```text
DeclarationId
    -> Set<InherentImplTarget>
```

so owner replacement/removal can remove old target keys in O(number of owned targets) expected work.

Do not rely on scanning the entire conditional-members map as the architectural solution.

#### D. Preserve consumers

Checker, associated-member resolution, lowering, editor receiver-effective lookup, and runtime already understand the identity. They should consume the newly published canonical product without special-case fixes.

### Forbidden approaches

- no checker/compiler/LSP-only exact-case side table;
- no insertion of conditional exact-case members into enum-root `DeclarationSurface`;
- no global installation into shared runtime behavior class;
- no runtime constraint solving;
- no separate direct/bound selection algorithm;
- no publication-only patch that leaves stale removal for later;
- no whole-map invalidation scan unless consultation explicitly accepts it.

### Test changes required

Add `AR-06` through `AR-09` with exact cases covering:

```text
matching applied exact case -> member available
wrong generic args -> unavailable
sibling exact case -> unavailable
enum root -> no case-only leakage
GADT-refined exact case + conditional domain if supported by existing fixtures
direct call
bound method/family
getter/setter/index selector where current C2 supports them
editor/receiver-effective lookup
lowering is_conditional
add/edit/delete constraint
variant rename/delete
owner/module removal or reload
cold/incremental equality
```

### Tests to run now

Discover exact filters:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic -- --list | rg 'impls|exact_case|conditional'
```

Run exact new semantic regressions first, then the owning `impls` filters that include conditional/exact-case behavior.

Run focused core conditional-runtime tests only if semantic publication regressions pass and AR-07 requires end-to-end execution evidence.

Run focused LSP tests only if LSP-specific source changes were necessary; otherwise semantic receiver-effective source-index tests are enough.

### Tests explicitly deferred

- full semantic integration binary;
- workspace test/clippy/Nextest;
- C2-F03 proof-state redesign tests;
- C2-F04/F05 performance/precision work.

### Acceptance

- full target identity has canonical conditional products;
- exact-case conditional receiver lookup works end-to-end;
- owner removal/replacement leaves no stale target keys;
- cold/incremental results agree;
- ordinary enum root behavior and shared erased generic runtime class remain uncontaminated;
- existing direct/bound runtime selection path is reused.

### Local STOP / CONSULT triggers

- full-target publication cannot be owned by semantic contribution/session lifecycle;
- exact-case conditional set requires mixing into `DeclarationSurface`;
- owner relationship cannot be represented without changing stable `VariantId`/declaration identity;
- invalidation requires semantic DB redesign;
- a consumer needs independent applicability logic after publisher is fixed.

### Checkpoint update

Record closure of C2-F01/F02, exact target ownership interface, and focused evidence. Then proceed to G1.

---

## T3 — Shared index `MemberBody` normalization

### Purpose

Normalize the one behavior-member AST form that cannot represent bodyless requirements, while preserving existing bodyful class/impl index semantics exactly.

### Preconditions

- G1 passes;
- no trait-specific parallel index AST has been introduced.

### Consumes

- shared `BehaviorMember`;
- `MemberBody::{Declaration, Block(...)}` or live equivalent;
- existing index getter/setter signature/parser semantics;
- current `IndexMethodDef.body` consumers.

### Produces

- `IndexMethodDef.body: MemberBody` or exact shared equivalent;
- parser helpers capable of returning declaration-only index body in trait context;
- all body/signature/fingerprint/compiler consumers updated mechanically;
- focused ordinary bodyful index regression coverage;
- foundation for TR-08 trait index requirements.

### Files and symbols

**Read/Modify:**
- `phalcom-ast/src/ast.rs::IndexMethodDef`
- `phalcom-ast/src/parser.rs`
- `phalcom-semantic/src/checker/declaration_signature.rs::CallableSyntaxRef` / `has_body`
- `phalcom-semantic/src/session.rs` callable-body extraction
- `phalcom-semantic/src/semantic_shard.rs`
- semantic DB/fingerprint code that hashes index bodies
- `phalcom-semantic/src/source_index/` index-member visitors
- `phalcom-core/src/compiler/` class/impl index compilation
- `phalcom-core/src/modules/semantic_lowering.rs` exhaustive matches if any
- `phalcom-core/src/compiler/lib/product_opt.rs` / AST walks if any
- native/generated/attribute index member constructors found by search
- affected tests.

### Required implementation shape

Change the shared representation once:

```rust
IndexMethodDef {
    ...
    body: MemberBody,
}
```

or exact equivalent.

Parser body helper must distinguish:

```text
body present -> MemberBody::Block(...)
body absent in trait declaration context -> MemberBody::Declaration
body absent in context that forbids abstract member -> syntax/semantic rejection
```

Update every `IndexMethodDef.body` consumer. Do not leave `.iter()`/length/block assumptions compiling by accident through adapter hacks.

Fingerprint rules should align with method/getter/setter:

```text
structural/surface fingerprint -> signature + body-presence semantic fact
body fingerprint -> statement contents only when Block
```

### Forbidden approaches

- no `TraitIndexRequirementDef` or trait-only index AST;
- no globally permitting bodyless indices in class/impl contexts unless canonical language already does;
- no redesign of setter result semantics;
- no unrelated parser cleanup.

### Test changes required

Add focused AST/semantic/core regressions for the shared index matrix in §15.5.

### Tests to run now

Run the exact parser/AST tests for index members and the smallest existing class/impl index semantic/runtime filter. If a dedicated AST integration target is created, run it directly.

Examples after verifying names:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast -- --list | rg 'index|subscript'
RUSTFLAGS='' cargo test -p phalcom-core --test core -- --list | rg 'index|subscript'
```

### Tests explicitly deferred

Trait syntax tests that depend on T4 may remain planned-red until T4.

### Acceptance

- all production consumers compile with shared `MemberBody` semantics;
- existing bodyful class/impl index behavior passes focused regression;
- declaration-only index body can be represented without trait-specific AST;
- no setter semantic drift.

### Local STOP / CONSULT triggers

- shared normalization would change existing observable index semantics;
- a consumer fundamentally cannot represent declaration-only body without architecture change;
- parser context cannot distinguish legality without introducing a trait-only grammar fork.

### Checkpoint update

Record `IndexMethodDef` shared body invariant only after focused ordinary-index regression passes.

---

## T4 — Canonical trait syntax, module declaration kind, and trait header

### Purpose

Introduce `trait` as a first-class named declaration and publish its generic header without creating nominal/runtime type machinery.

### Preconditions

- G1 passes;
- T3 shared index body representation is available;
- `DeclarationKind::Protocol` compatibility verification completed.

### Consumes

- module declaration shell/namespace collection;
- generic parameter/where-clause parser;
- shared `BehaviorMember` parser;
- `TypeParameterOwner::Declaration` and `GenericSignature`;
- source ranges/attributes conventions.

### Produces

- `trait` token/lexer/parser support;
- `TraitDef` and `Statement::Trait`;
- `DeclarationKind::Trait`;
- trait named module/interface binding/import/export;
- trait header semantic product keyed by `DeclarationId` with generic signature/source metadata;
- no nominal/class-object type entry;
- TR-01..TR-03 foundation.

### Files and symbols

**Read/Modify:**
- `phalcom-ast/src/token.rs`
- `phalcom-ast/src/lexer.rs`
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-modules/src/declaration.rs`
- `phalcom-modules/src/interface.rs`
- `phalcom-semantic/src/semantic_shard.rs`
- `phalcom-semantic/src/session.rs` or new trait semantic owner
- create `phalcom-semantic/src/traits.rs` or repository-equivalent owner
- `phalcom-semantic/src/lib.rs`
- source declaration extraction exhaustive matches.

### Required implementation shape

Conceptual AST:

```rust
pub struct TraitDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub members: Vec<BehaviorMember>,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}
```

It has no superclass, fields, data components, variants, constructors, or class invariants.

Module integration:

```text
Statement::Trait
    -> named declaration shell
    -> DeclarationId
    -> DeclarationKind::Trait
```

`Protocol` should become `Trait` if compatibility search confirms no stable external contract.

Trait header concept:

```rust
TraitHeader {
    declaration: DeclarationId,
    generic_signature: Option<GenericSignature>,
    source: ...,
}
```

Trait generic parameters use:

```text
TypeParameterOwner::Declaration(trait DeclarationId)
```

Do not call `NominalDeclarationHeader::from_signature` or insert into `DeclarationTypeTable`.

### Forbidden approaches

- no `@protocol class` parser alias as canonical implementation;
- no `TraitId` required identity;
- no class flag/superclass storage;
- no nominal `TypeId`/class-object type solely for trait generics;
- no associated types/conformance syntax;
- no class-side trait semantics.

### Test changes required

Create/extend AST trait syntax tests for:

```text
empty trait
bodyless method/getter/setter/index
bodyful default
generic trait
ordinary where clause
ranges
illegal field/constructor/state syntax
```

Add module tests for namespace collision/import/export and `DeclarationKind::Trait`.

Add semantic header test proving generic parameter owner and absence from nominal declaration type table.

### Tests to run now

After verifying new test target name:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test trait_syntax
RUSTFLAGS='' cargo test -p phalcom-modules trait
```

Then run exact semantic header test filter only.

### Tests explicitly deferred

- TraitRef resolution (T5);
- TraitSurface/defaults (T6–T8);
- full semantic/core suites.

### Acceptance

`trait` is a named declaration with canonical `DeclarationId` and header `GenericSignature`; bodyless index is representable; no nominal/class-object/runtime class semantics have been introduced.

### Local STOP / CONSULT triggers

- `DeclarationKind::Protocol` is serialized/stable ABI and cannot be mechanically migrated;
- module interface requires runtime class semantics for any named declaration;
- trait header generics cannot be stored outside nominal table without broader type-system ownership change;
- parser needs class-only state features to represent trait.

### Checkpoint update

Record stable trait declaration/header interface and `DeclarationKind::Trait` after focused module/header tests pass.

---

## T5 — TraitRef and requirement/source identity foundations

### Purpose

Create canonical generic trait contract references and stable requirement/source callable identities without conformance semantics.

### Preconditions

- T4 trait header exists;
- canonical declaration lookup and generic resolver available.

### Consumes

- `DeclarationId` + `DeclarationKind::Trait`;
- trait header `GenericSignature`;
- canonical type argument resolution/kind/ordinary constraint checking;
- `CallableId`, `CallableOwnerId::Declaration`, `Selector`, `DispatchSide`;
- `TypeParameterOwner::Callable`.

### Produces

- `TraitRef`;
- `TraitRequirementId`;
- deterministic trait member source `CallableId` construction;
- category/arity/kind diagnostics;
- TR-04..TR-07 identity foundation.

### Files and symbols

**Read/Modify:**
- trait semantic module created in T4
- canonical declaration/type-like resolution helpers
- `phalcom-semantic/src/identity.rs` if `TraitRequirementId` belongs there, or re-export from trait module
- `checker/declaration_signature.rs` callable identity helpers as required
- semantic tests.

### Required implementation shape

TraitRef:

```rust
TraitRef {
    declaration: DeclarationId,
    arguments: Box<[TypeId]>,
}
```

Formation:

```text
source declaration reference
 -> DeclarationId
 -> verify kind Trait
 -> trait header GenericSignature
 -> resolve canonical TypeId args
 -> arity/kind/existing ordinary constraint validation
 -> TraitRef
```

Do not route a named trait declaration through `TypeLevelBinding`.

Requirement:

```rust
TraitRequirementId {
    owner: DeclarationId,
    selector: Selector,
    side: DispatchSide,
}
```

Source callable:

```rust
CallableId {
    owner: CallableOwnerId::Declaration(trait_decl),
    selector,
    side,
}
```

For P1 supported members, side is instance unless newer accepted authority explicitly says otherwise.

### Forbidden approaches

- no `TypeData::Trait` just to support TraitRef;
- no conformance boolean/proof;
- no `TraitMemberId`/`TraitDefaultId` duplication;
- no `TypeParameterOwner::Trait`;
- no class-side/metatype semantics.

### Test changes required

`TR-04` through `TR-07`:

```text
non-generic TraitRef
generic TraitRef
canonical equality
wrong declaration category
wrong arity
wrong kind
ordinary constraint reuse
trait as ordinary inhabitable type rejected/deferred
bodyless/bodyful requirement identity stability
source callable identity is distinct from requirement
```

### Tests to run now

Run exact new semantic identity/reference tests from the trait capability module. Use `-- --list` first to verify names.

### Tests explicitly deferred

Surface/default-body tests.

### Acceptance

C4 has a durable contract reference and requirement identity model, but no target conformance relation exists. No duplicate declaration/callable/type-parameter identity universe is introduced.

### Local STOP / CONSULT triggers

- TraitRef formation appears to require making traits ordinary proper types;
- declaration lookup cannot distinguish trait category without new parallel namespace;
- requirement identity needs information beyond owner/selector/side that affects C4 contract semantics;
- source callable identity cannot use declaration-owned `CallableId` without breaking existing identity invariants.

### Checkpoint update

Record `TraitRef`, `TraitRequirementId`, and trait source `CallableId` APIs after identity tests pass.

---

## T6 — Complete TraitSurface and signature publication

### Purpose

Build the complete abstract contract product for a trait before analyzing any default body.

### Preconditions

- T5 identities and TraitRef exist;
- shared behavior member forms available;
- canonical signature builder with explicit resolver available.

### Consumes

- trait header `GenericSignature`;
- all behavior member syntax;
- `TraitRequirementId`;
- trait-owned `CallableId`;
- canonical `semantic_signature_for_syntax_with_resolver(...)` or live equivalent;
- scoped `TypeLevelBinding` for actual generic binders.

### Produces

- immutable/queryable `TraitSurface`;
- all member `CallableSemanticSignature`s;
- visibility/source/default-presence metadata;
- duplicate/illegal member diagnostics;
- complete publication before bodies;
- TR-08..TR-10 and G3 evidence.

### Files and symbols

**Read/Modify:**
- trait semantic module
- `phalcom-semantic/src/checker/declaration_signature.rs`
- semantic session/query wiring
- diagnostics modules as conventional
- tests under semantic trait capabilities.

### Required implementation shape

Build a trait generic resolver from actual trait parameter IDs:

```text
TraitHeader.generic_signature
    -> TypeParameterId values
    -> TypeLevelBinding for each binder
    -> ScopedTypeResolver
    -> canonical semantic signature builder
```

For each member, publish repository-equivalent:

```text
TraitRequirementId
CallableId
CallableSemanticSignature
visibility
source
default_present/default_source
```

The surface is complete before default body analysis is scheduled.

Duplicate exact selector + side is a contract conflict regardless of differing type annotations. Do not invent type-overload semantics.

Trait body legality at this stage rejects structural/state declarations that are not `BehaviorMember`s. Class-side members remain deferred/unsupported in P1 unless ratified otherwise.

### Forbidden approaches

- no body analysis while constructing surface;
- no `DeclarationSurface` insertion;
- no conditional inherent surface insertion;
- no enum requirement reuse;
- no trait-specific type/signature solver;
- no full default statements in TraitSurface structural fingerprint.

### Test changes required

Add/complete:

```text
TR-08 behavior shapes
TR-09 duplicate selector
TR-10 visibility/source
DF-01 Self in supported signatures
surface query before body analysis
later-declared member signature visible before bodies
no DeclarationSurface contamination
no EnumRequirementId reuse
```

### Tests to run now

Run exact semantic trait-surface tests. Then G3 uses a focused trait capability filter.

### Tests explicitly deferred

Default body success/failure beyond signature formation; incremental/body products until T8/T9.

### Acceptance

A complete, body-independent `TraitSurface` can be queried for every valid trait, with stable requirement/source identities and canonical signatures, including bodyless indices.

### Local STOP / CONSULT triggers

- a signature requires body inference to be publishable;
- TraitSurface needs concrete receiver/conformance data;
- existing signature builder cannot accept trait owner generic bindings without duplicating type resolution;
- duplicate/selector semantics conflict with canonical selector model.

### Checkpoint update

Record TraitSurface owner/query/schema and G3-ready invariant.

---

## T7 — Generalize callable body owner-generic inputs

### Purpose

Remove the nominal-only assumption that a declaration-owned callable's owner generics must come from `DeclarationTypeTable`, without changing existing class/data/enum behavior.

### Preconditions

- T6 TraitSurface/signatures available;
- current body analyzer nominal lookup assumption verified.

### Consumes

- existing `CallableBodyRequest` / `BodyAnalysisContext`;
- existing nominal declaration generic lookup;
- trait header `GenericSignature`;
- member-local signature/generic metadata.

### Produces

- narrow explicit owner-generic input to callable body analysis;
- nominal callers retain existing behavior;
- trait default caller can supply trait header generic signature directly;
- DF-02 generic default foundation.

### Files and symbols

**Read/Modify:**
- `phalcom-semantic/src/checker/body.rs`
- body-analysis call sites in session/checker
- generic class/enum/data body tests
- trait default test scaffolding.

### Required implementation shape

Preferred concept:

```rust
CallableBodyRequest {
    ...
    owner_generic_signature: Option<GenericSignatureView>,
}
```

or a context field carrying owner `TypeParameterId`s/bindings.

Rules:

1. Existing nominal caller may continue deriving owner generics from `DeclarationTypeTable` before building request/context.
2. Trait caller passes `TraitHeader.generic_signature` explicitly.
3. Callable-local generics still come from canonical callable signature metadata.
4. The body analyzer itself no longer requires every declaration owner to have a nominal type-table entry.
5. No generic inference/application logic is duplicated.

### Forbidden approaches

- no trait insertion into nominal declaration table;
- no broad redesign of declaration type storage;
- no change to existing generic class/constructor semantics;
- no trait-specific body analyzer.

### Test changes required

- existing generic class method body still resolves owner generic;
- generic trait default resolves trait generic;
- member-local generic composes/shadows according to existing lexical rules;
- no trait nominal type entry is created.

### Tests to run now

Run exact body-analysis regression for existing generic nominal callable plus exact new generic trait-default scaffolding test. Do not run full semantic suite.

### Tests explicitly deferred

Abstract Self lookup/calls until T8.

### Acceptance

Body analysis can receive declaration-owner generic binders independently of nominal type metadata, while existing nominal behavior is unchanged.

### Local STOP / CONSULT triggers

- body analyzer generic ownership is more deeply coupled to nominal `TypeId`/class-object semantics than the audit/old plan established;
- explicit owner generic context would create a second binder source or inconsistent IDs;
- existing generic body tests regress semantically rather than mechanically.

### Checkpoint update

Record the generalized body-analysis input only if it becomes a durable interface C4/C5 will consume.

---

## T8 — Abstract Self lookup, semantic-only contract calls, and default analysis

### Purpose

Check trait defaults exactly once against the completed trait contract, using canonical `Self` and callable application without inventing a concrete witness or executable runtime target.

### Preconditions

- T6 complete TraitSurface;
- T7 owner-generic body context;
- canonical call application target supports `target=None`.

### Consumes

- `SelfTypeTerm`;
- trait header/surface;
- trait-owned source `CallableId`;
- `TraitRequirementId`;
- canonical member selector/signature lookup;
- canonical call argument/generic checker;
- body checker field/super capabilities.

### Produces

- trait-default body analysis query/product;
- abstract trait-Self member lookup adapter;
- abstract call authority in `CallTargetAuthority` or equivalent;
- semantic application with callable identity and no `InvocationTargetId`;
- contract-relative default-to-default behavior;
- DF-01..DF-09 tests.

### Files and symbols

**Read/Modify:**
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/checker/body.rs` integration
- trait semantic lookup/query module
- semantic trait tests.

### Required implementation shape

Trait default context:

```text
current declaration = trait DeclarationId
current callable = trait-owned CallableId
side = Instance
owner generics = TraitHeader signature
member generics = callable signature
Self = SelfTypeTerm { owner: trait, side: Instance, role: InstanceType }
fields = none
superclass = none
conformance evidence = none
abstract contract = completed TraitSurface
```

Abstract lookup:

```text
receiver is current trait abstract Self
    -> TraitSurface lookup(selector, side)
    -> requirement/source callable/signature
```

Do not register TraitSurface with ordinary `SurfaceDispatchResolver`.

Application:

```text
CallableApplicationTarget {
    signature: contract signature,
    callable: Some(trait_owned_callable),
    target: None,
    authority: AbstractContract/TraitContract,
    ...
}
```

The existing call application logic still validates argument lanes, labels, generics, result type, and diagnostics.

Default-to-default call lookup returns the requirement contract even when `default_present=true`.

Illegal field/stored-component and `super` accesses must fail semantically because trait context exposes no such capabilities.

### Forbidden approaches

- no synthetic conformer/fake class;
- no `TraitSelfType` parallel type system;
- no `InvocationTargetId::Behavioral(trait_callable)` for abstract call;
- no direct default-body binding;
- no conformance/witness selection;
- no trait-specific argument/generic solver;
- no compiler/runtime fix for a semantic checker gap.

### Test changes required

Add all `DF-01`..`DF-09`, especially:

```text
default calls bodyless requirement
default calls later requirement
default calls bodyful member contract-relatively
generic trait default
Self return/parameter shape
unknown abstract member
argument/generic mismatch
illegal field
illegal super
abstract target is None and authority is contract-relative
```

### Tests to run now

Run exact new trait-default semantic filters. Then run focused canonical call-application tests only if the common call code was modified beyond adding an authority branch.

### Tests explicitly deferred

Runtime execution of trait defaults; there is no conformance yet.

### Acceptance

Defaults analyze once and successfully call contract members using canonical application; no runtime target/witness exists; later-declared members work; illegal storage/super is rejected.

### Local STOP / CONSULT triggers

- abstract Self cannot be represented by existing `SelfTypeTerm`;
- member lookup needs concrete declaration surface or hierarchy;
- application requires a runtime target to type-check;
- a default must be rechecked per future conformer;
- class-side/metatype behavior becomes necessary.

### Checkpoint update

Record abstract Self/default call contract and authority variant after G4 passes, not after every individual test.

---

## T9 — Incremental products, source index, and LSP projection

### Purpose

Make trait semantics durable across revisions and editor/source consumers without creating parallel semantic authority.

### Preconditions

- T8 semantic trait behavior complete;
- current DB/query/fingerprint conventions understood.

### Consumes

- TraitHeader;
- TraitSurface;
- trait default CallableAnalysis;
- existing semantic dependency/query infrastructure;
- source-index canonical targets.

### Produces

- query/fingerprint/dependency separation for header/surface/body;
- body/signature/header invalidation behavior in §15;
- source index trait declaration/member entries;
- Trait workspace/editor presentation;
- IN-01..IN-05 and LS-01..LS-02 coverage.

### Files and symbols

**Read/Modify:**
- `phalcom-semantic/src/db/` relevant query/product/fingerprint modules
- `phalcom-semantic/src/semantic_shard.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/scope.rs`
- `phalcom-semantic/src/source_index/symbol.rs`
- occurrence/reference modules as required
- `phalcom-lsp/` only if normal source-index presentation requires a small mapping change
- semantic incremental/integration tests.

### Required implementation shape

Fingerprint ownership:

```text
TraitHeader
    declaration/generic header/source structure

TraitSurface
    requirement/source IDs
    selectors/signatures
    visibility
    default-present/source metadata
    NO full default statements

CallableAnalysis/body fingerprint
    default body statements and body-derived facts
```

Invalidation laws are exactly `IN-01`..`IN-05`.

Source index:

```text
trait declaration -> SemanticTargetId::Declaration
trait member/default -> SemanticTargetId::Callable
```

Add `SourceDeclarationKind::Trait` and map to editor Trait/Interface-like presentation.

LSP consumes the source index. Do not reconstruct TraitSurface or perform trait reference/default lookup in LSP code.

### Forbidden approaches

- no full body hash in TraitSurface fingerprint;
- no LSP-local trait identity/solver;
- no broad DB redesign;
- no trait-specific source target enum solely for display;
- no eager invalidation of all semantic products on body edit if existing dependency granularity can express the correct frontier.

### Test changes required

Implement `IN-01`..`IN-05` and `LS-01`..`LS-02`.

At minimum inspect product identity/fingerprint counters or snapshot object identity using existing incremental test idioms rather than only user-visible final diagnostics.

### Tests to run now

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic -- --list | rg 'trait|incremental|source_index'
```

Run exact trait incremental filters and exact source-index integration filters. Run LSP tests only if `phalcom-lsp` source itself changed.

### Tests explicitly deferred

- full semantic suite;
- full LSP suite if LSP source unchanged;
- C2-F05 fine-grained impl-domain fingerprint optimization.

### Acceptance

Cold/incremental trait results agree; body-only edits preserve contract identities/surface signatures; signature/header edits invalidate correct dependents; navigation/workspace symbols use canonical semantic targets.

### Local STOP / CONSULT triggers

- correct invalidation needs semantic DB redesign;
- source index cannot represent trait without new semantic identity;
- LSP requires re-solving trait semantics;
- body/surface separation conflicts with established query ownership.

### Checkpoint update

Record query keys/fingerprint boundaries and focused incremental/source evidence at G5.

---

## T10 — Compiler/runtime trait non-class boundary

### Purpose

Allow trait-containing modules to compile/load while proving that C3 introduces no ordinary runtime class, method injection, or runtime trait dispatch semantics.

### Preconditions

- trait AST/module/semantics complete;
- default bodies are semantic products but have no concrete executable witness target.

### Consumes

- `Statement::Trait`;
- semantic declaration category;
- compiler top-level statement dispatch;
- existing compile-time-only declaration precedent where safe.

### Produces

- exhaustive compiler/AST handling for `Statement::Trait`;
- compile-time no-op/type-level treatment;
- no runtime class/finalization/storage/method installation;
- CP-01..CP-04 focused evidence.

### Files and symbols

**Read/Modify:**
- `phalcom-core/src/compiler/` top-level statement dispatch
- compiler AST/exhaustive visitors
- `phalcom-core/src/modules/semantic_lowering.rs` only if a stable no-op projection is needed
- product optimizer/other AST visitors for new statement exhaustiveness
- `phalcom-core/tests/core/language/traits.rs` or repository-equivalent focused module.

### Required implementation shape

Preferred handling:

```text
Statement::Trait
    semantic/type-level declaration already analyzed
    compiler emits no ordinary runtime class setup
```

No:

```text
ClassId per trait
class-object allocation
superclass edge
field slots/ProductLayout
FinalizeClass
add_method/install default on target
VM trait scan
runtime conformance registry
witness/vtable
```

If the module runtime normally binds every named declaration to a value, verify the type-alias or equivalent compile-time-only precedent. If trait export genuinely requires runtime descriptor identity, stop rather than creating a fake class.

### Forbidden approaches

- no fake class descriptor;
- no compiling default bodies into globally discoverable methods;
- no runtime shape-based conformance discovery;
- no C7 reflection design;
- no compiler reconstruction of TraitSurface.

### Test changes required

Implement `CP-01`..`CP-04`:

```text
module containing unused trait compiles/executes surrounding code
no trait runtime class/finalization side effect
default/requirement not discoverable on unrelated by-shape class
enum requirement path unchanged
conditional inherent behavior path unchanged
```

### Tests to run now

Run exact new core trait-language tests. Run adjacent enum/conditional regression filters only because compiler statement/exhaustiveness code crossed those paths, not the entire core suite.

### Tests explicitly deferred

- runtime execution of selected trait defaults;
- full Universe/reflection trait descriptors;
- workspace tests.

### Acceptance

Trait-containing source compiles and ordinary runtime behavior remains unchanged; no runtime trait class/scan/injection exists.

### Local STOP / CONSULT triggers

- module binding/export requires runtime trait value;
- compiler cannot skip trait without losing required language-visible binding semantics;
- default semantic body must be compiled/executed before conformance exists;
- trait appears in ordinary runtime dispatch tables.

### Checkpoint update

Record compiler/runtime boundary and negative evidence at G5.

---

## T11 — Protocol-era specification migration

### Purpose

Remove competing active language rules and make `trait` the one effective specification model for reusable behavioral contracts.

### Preconditions

- implemented C3 semantics stable through G5;
- current spec governance/index read.

### Consumes

- landed trait behavior;
- canonical language taxonomy;
- active typing spec cluster;
- archive/superseded conventions.

### Produces

- updated effective specification/index/STATUS;
- protocol-era language-feature docs archived/marked superseded/narrowed;
- generic examples migrated to `trait` where normative;
- no accidental claim that C4/C5/C6 features landed;
- SP-01 evidence.

### Files and symbols

**Read/Modify/Archive as governance requires:**

```text
docs/spec/typing/README.md
docs/spec/typing/STATUS.md
docs/spec/typing/01-protocol-foundation.md
docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md
docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md
docs/spec/typing/03-type-parameters-and-generic-signatures.md
docs/spec/next/phalcom-meta-dispatch-and-type-extension-spec.md
```

Also search active spec material for language-feature claims containing:

```text
@protocol class
Protocol descriptor
signature-only protocol
structural protocol conformance
protocol default
class-side protocol requirement
```

### Required implementation shape

Effective rules must match C3:

```text
trait Name<...> { ... }
bodyless behavior = requirement
bodyful behavior = same requirement + default
no owned instance representation
TraitRef is contract reference
no conformance/witness yet
no associated types yet
instance-contract-first C3 scope
```

Mark future features as owned by later checkpoints rather than specifying them as current behavior.

Historical rationale may remain in archive/superseded material. Ordinary English “protocol” terminology remains untouched.

### Forbidden approaches

- no global find/replace of “protocol”;
- no claiming C4 conformance has landed;
- no inventing runtime trait descriptor/reflection surface;
- no ratifying provisional metatype syntax;
- no deleting useful history when archive/superseded marking is the repository convention.

### Test changes required

No code tests for prose-only edits. `SP-01` is verified by targeted searches and effective-spec index review.

### Tests to run now

No compilation/test machinery for prose-only changes.

Run targeted search commands only, e.g. repository-equivalent `rg` scoped to active specs.

### Tests explicitly deferred

All code tests already covered by prior gates.

### Acceptance

Active normative documentation has one coherent trait model and no active competing `@protocol class` signature-only contract rule.

### Local STOP / CONSULT triggers

- spec governance says a conflicting protocol document is still canonical and cannot be superseded by implementation records;
- a newer ratified trait spec materially differs from the plan;
- migration would require deciding C4/C5/C6 semantics.

### Checkpoint update

Record specification migration disposition and canonical effective docs after review.

---

## T12 — Focused stabilization, checkpoint closure, walkthrough, and C4 handoff

### Purpose

Establish focused acceptance evidence, classify remaining failures truthfully, close C3 lifecycle state, and leave C4 a precise takeover package.

### Preconditions

- T1–T11 implemented;
- G1–G5 passed;
- no unresolved consultation trigger.

### Consumes

- all coverage IDs;
- checkpoint evidence;
- current diff;
- known audit/deferred ledger.

### Produces

- G6 focused acceptance;
- completed coverage mapping;
- truthful checkpoint `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` or repository-equivalent lifecycle status;
- `LANG005.C3.P1-walkthrough.md`;
- `LANG005.C3.P1-handoff.md`;
- explicit C2-F03 pre-C4 condition;
- deferred predecessor/release issues ledger.

### Files and symbols

**Modify:**
- `LANG005.C3-CHECKPOINT.md`

**Create:**
- `LANG005.C3.P1-walkthrough.md`
- `LANG005.C3.P1-handoff.md`

**Read:** all changed source/tests/specs and current diff.

### Required implementation shape

1. Run G6 focused acceptance commands.
2. Confirm each command selected nonzero tests.
3. Run negative source searches from §20.
4. Classify any failures A/B/C/D.
5. Do not broaden after PASS unless a changed cross-layer boundary or failure evidence requires it.
6. Run formatting check once; if baseline failures remain in untouched files, classify and record rather than broad-format unrelated work.
7. Update checkpoint with final interfaces/invariants/audit closures/deferred findings/consultations/verification evidence.
8. Write retrospective walkthrough from landed code, not plan prose.
9. Write C4 handoff with actual APIs and first actions.

### Forbidden approaches

- no release-complete claim from focused tests;
- no workspace cleanup to make unrelated lint/format failures green;
- no hiding C2-F03 pre-C4 debt;
- no copying this plan as walkthrough;
- no empty/zero-test filters counted as evidence.

### Test changes required

No new tests unless G6 exposes a missing coverage obligation or regression. If so, add the smallest discriminating regression under the owning layer.

### Tests to run now

Run G6 commands in §19/§20 serially.

### Tests explicitly deferred

Full workspace/release gates unless the user/checkpoint explicitly requests certification.

### Acceptance

- all required coverage obligations have PASS evidence or an explicitly approved justified omission;
- all A/B failures are closed;
- no blocking C remains;
- D failures are recorded;
- C3 checkpoint truth matches implementation;
- walkthrough and handoff exist;
- handoff states C2-F03 must close before C4 conformance/witness implementation.

### Local STOP / CONSULT triggers

- a missing coverage case reveals an architectural gap;
- final negative search finds forbidden architecture in production code;
- focused test oracle conflicts with ratified semantics;
- checkpoint can only be marked complete by weakening an invariant.

### Checkpoint update

Mandatory final update. Set next action to the C4 prerequisite/remediation sequence, not blindly “start C4” if C2-F03 remains open.

---

# 19. Verification gates

### G1 — Predecessor identity/surface foundation restored

**After:** T1 + T2.

**Purpose:** prove C3 can safely build on C1/C2.

Run, after confirming exact filter names with `-- --list`:

```sh
# C1 exact product/reification + Record identity
RUSTFLAGS='' cargo test -p phalcom-core --test core -- --list | rg 'product|record|runtime_type|generic'
RUSTFLAGS='' cargo test -p phalcom-core --lib -- --list | rg 'product_opt|typing::environment|product'
# then exact AR-01..AR-05 filters

# C2 exact-case conditional lifecycle
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic -- --list | rg 'impls|exact_case|conditional'
# then exact AR-06..AR-09 filters
```

If AR-07 includes end-to-end runtime behavior, run only the focused core conditional-dispatch filter discovered from `-- --list`.

**Expected evidence:**

```text
AR-01..AR-09 PASS
exact generic product descriptors carry correct exact types
optimized/eager type observation agrees
static/dynamic Record logical identity agrees
exact-case conditional products publish by target
owner deletion/edit leaves no stale target product
cold/incremental exact-case results agree
no shared generic class leakage
```

Do not proceed to T3 until G1 passes.

---

### G2 — Shared member shape + trait declaration/header identity

**After:** T3 + T4 + T5.

**Purpose:** prove syntax/module/generic/identity foundation before contract surface/defaults.

Run exact targets after verifying registration:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test trait_syntax
RUSTFLAGS='' cargo test -p phalcom-modules trait
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic -- --list | rg 'traits|trait'
# then exact TR-01..TR-07 filters
```

Also run the smallest existing bodyful index core/semantic regression selected during T3.

**Expected evidence:**

```text
trait parses as Statement::Trait
bodyless index representable via shared MemberBody
ordinary index behavior unchanged
DeclarationKind::Trait + DeclarationId canonical
trait header generic owner correct
no nominal trait table entry
TraitRef and requirement/source identities correct
```

---

### G3 — Complete TraitSurface

**After:** T6.

**Purpose:** prove all contract signatures publish before body analysis.

Run exact semantic trait-surface filters covering TR-08..TR-10 and surface-order tests.

**Expected:**

```text
method/getter/setter/index requirements publish
bodyful members publish default availability without body analysis
later member signature visible
visibility/source retained
duplicate selector diagnosed
no DeclarationSurface/conditional/enum contamination
```

Do not proceed if default analysis is required to construct the surface.

---

### G4 — Abstract Self/default semantics

**After:** T7 + T8.

**Purpose:** prove one-time default checking and semantic-only contract calls.

Run exact `DF-01..DF-09` filters and the one existing generic nominal-body regression touched by T7.

**Expected:**

```text
generic owner bindings work without nominal trait type
Self is canonical SelfTypeTerm
default calls requirement/later/default member
application target callable is semantic, target is None
canonical call diagnostics reused
field/super/class-side boundaries enforced
no witness/conformance exists
```

---

### G5 — Incremental/source/compiler boundaries

**After:** T9 + T10.

**Purpose:** prove product lifetime/tooling/runtime separation.

Run:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic -- --list | rg 'trait|incremental|source_index'
# exact IN-01..IN-05 and LS-01..LS-02 filters

RUSTFLAGS='' cargo test -p phalcom-core --test core -- --list | rg 'trait|enum|conditional'
# exact CP-01..CP-04 filters
```

Run focused LSP tests only if LSP-specific source changed.

**Expected:**

```text
body/signature/header invalidation laws hold
cold/incremental parity holds
source targets remain Declaration/Callable
trait compiles with no ordinary runtime class/method injection
enum and C2 conditional paths remain separate
```

---

### G6 — Focused final certification

**After:** T11 + T12.

**Purpose:** certify the bounded checkpoint, not the release.

Run the consolidated focused targets from G1–G5 without needlessly repeating every exact micro-filter if the owning focused suite already includes them and passed after the last relevant edit.

Then run:

```sh
cargo fmt --all -- --check
```

Treat any unchanged baseline failure according to §17; do not broad-format unrelated source.

Run negative searches over production code for forbidden architecture (see §20.2).

**Expected:** all C3 acceptance evidence passes; any unrelated baseline issue is classified and recorded; no forbidden shortcut is present.

---

## 20. Final focused acceptance

### 20.1 Coverage-to-test evidence table

The walkthrough/checkpoint must contain the actual final mapping:

| Coverage family | Required evidence |
|---|---|
| `AR-01..AR-05` | focused core product/runtime-type/optimizer tests |
| `AR-06..AR-09` | focused semantic exact-case conditional + necessary core/tooling integration |
| `TR-01..TR-03` | AST/module/header tests |
| `TR-04..TR-10` | semantic trait reference/identity/surface tests |
| `DF-01..DF-09` | semantic trait-default/abstract-Self tests |
| `IN-01..IN-05` | semantic incremental trait tests |
| `LS-01..LS-02` | semantic source-index tests; LSP only if source changed |
| `CP-01..CP-04` | focused core compiler/runtime separation tests |
| `SP-01` | spec search/index review evidence |
| `HD-01` | checkpoint/handoff contains C2-F03 pre-C4 requirement |

Every filter used as evidence must select at least one test.

### 20.2 Negative verification

Before completion, inspect production code for forbidden shortcuts:

```text
trait represented as flagged ClassDef
trait -> ordinary ClassId / FinalizeClass
trait -> ProductLayout / field slots / superclass
trait inserted into nominal DeclarationTypeTable
TypeLevelBinding::Trait
TypeParameterOwner::Trait
mandatory duplicate TraitId/TraitMemberId/TraitDefaultId
TraitSurface inserted into DeclarationSurface
TraitSurface inserted into conditional inherent surface
TraitRequirementId aliased to witness CallableId
trait abstract call -> InvocationTargetId::Behavioral
trait default -> runtime add_method / target method installation
VM ordinary Invoke -> trait scan
trait-only duplicate index AST
LSP trait resolver/solver duplicating semantics
exact-case conditional publisher only in checker/compiler/LSP
runtime generic product exact type inferred from component payload
optimized product dropping type recipe
```

Also inspect active spec documents for unresolved language-feature claims:

```text
@protocol class
first-class Protocol descriptor as canonical current contract model
signature-only protocols as canonical current model
structural protocol conformance as already-current C3 semantics
```

Explain intentional history/archive hits.

### 20.3 Final acceptance rule

C3.P1 may be marked focused-tested when:

```text
all integrated audit blockers closed
all required C3 coverage families have passing focused evidence
all A/B failures closed
no blocking C remains
D failures recorded
checkpoint truthful
walkthrough written
handoff written
```

Do not automatically run workspace certification after this PASS.

---

## 21. Performance/resource evidence

This is primarily semantic/correctness work, so do not add benchmark ceremony. Three resource constraints are relevant because the audit remediation touches runtime identity and conditional target lifecycle.

### 21.1 Runtime type environments

Evidence required:

```text
nongeneric calls continue to use RuntimeTypeEnvironmentId::EMPTY
no per-anonymous-product generic argument map/array is allocated
runtime environments are interned/shared according to existing registry design
```

A focused unit assertion or source inspection plus existing registry tests is sufficient; no benchmark required.

### 21.2 Conditional target invalidation

Owner replacement/removal should be O(number of target keys owned by that declaration) expected work, not an O(all conditional targets) scan introduced as the permanent architecture.

Evidence: data structure/API shape and focused lifecycle test; no microbenchmark required.

### 21.3 Trait runtime footprint

C3 trait declarations must add no ordinary per-instance storage, per-conformer runtime method tables, or per-applied-generic runtime classes.

Evidence: negative source inspection + CP-01/CP-02 focused tests.

---

## 22. Checkpoint bookkeeping

The durable shared state is:

```text
docs/implementation/LANG005/LANG005.C3/LANG005.C3-CHECKPOINT.md
```

### At plan start

- record `git status --short`, branch, revision;
- mark P1 active;
- remove stale pre-final-C2 lifecycle blocker;
- import final C2 takeover symbol map;
- record C1/C2 audit disposition;
- state G1 blocks trait production source until remediation passes.

### During plan

Update checkpoint only for durable events:

- T1 establishes completed C1 reification/Record identity invariant;
- T2 establishes target-indexed conditional publication/lifecycle;
- G1 closes predecessor remediation gate;
- T4/T5 establish stable trait header/identity APIs;
- G3 establishes TraitSurface;
- G4 establishes abstract Self/default-call contract;
- G5 establishes query/source/compiler boundaries;
- consultation/amendment;
- meaningful deferred/baseline failure.

Do not log every edit/test rerun.

### At plan completion

Record:

```text
plan completion/verification
final revision
final stable interfaces
final invariant ledger
audit findings closed/deferred
C2-F03 hard pre-C4 condition
focused test evidence
baseline/deferred failures
consultations/amendments
spec migration disposition
walkthrough/handoff locations
next action
```

Do not create a competing implementation-state document.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C3/LANG005.C3.P1-walkthrough.md
```

It must describe what actually landed, including:

- final result/revision;
- C1-F01 reification repair and actual generic runtime environment producer;
- C1-F02 Record canonicalization repair;
- C2-F01/F02 exact-case conditional target publication/removal;
- any audit finding whose planned disposition changed after consultation;
- trait AST/module declaration shape;
- index `MemberBody` normalization and consumer updates;
- declaration/requirement/source-callable identity model;
- `TraitRef` formation;
- trait generic ownership/header storage;
- TraitSurface schema/query/publication order;
- body analyzer owner-generic generalization;
- abstract Self lookup;
- abstract call authority and absence of runtime target;
- default-to-default contract-relative behavior;
- incremental dependency/fingerprint graph;
- source target/LSP presentation;
- compiler/runtime non-class boundary;
- specification migration;
- tests added;
- tests actually run;
- tests deferred;
- A/B/C/D failures;
- plan deviations;
- consultation decisions;
- residual risks.

The walkthrough is retrospective evidence, not a restatement of this plan.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C3/LANG005.C3.P1-handoff.md
```

It must let C4 proceed without rediscovering C3.

Include actual landed APIs for:

```text
trait declaration identification
DeclarationKind::Trait
trait header/generic signature query
TraitRef formation
TraitRequirementId
trait source/default CallableId
TraitSurface query/member entry schema
visibility/source metadata
default body query
owner-generic body-analysis input
Self representation
abstract trait member lookup
abstract call authority / target=None contract
source targets
incremental query keys/fingerprints
C2 receiver-effective conditional lookup
exact-case conditional target publication ownership
compiler/runtime trait boundary
```

Also include:

- current revision/worktree caveat;
- established invariants;
- focused verification evidence;
- deferred predecessor findings;
- **C2-F03 proof-state granularity as a hard prerequisite before C4 conformance/witness implementation**;
- first recommended commands;
- things C4 must not redesign/re-explore.

---

## 25. Completion truth table

Do not conflate implementation stages.

| State | Meaning | Required evidence |
|---|---|---|
| source written | planned code/docs exist | diff only; not acceptance |
| tests added | regressions exist | nonzero test discovery; not acceptance |
| targeted tests passed | exact new regressions pass | T0/T1 evidence |
| owning focused suite passed | coherent subsystem behavior passes | relevant gate evidence |
| predecessor remediation accepted | C1/C2 integrated blockers closed | G1 PASS |
| trait semantics accepted | surface/default/incremental/compiler obligations pass | G2–G5 PASS |
| C3 checkpoint accepted | all required focused coverage + lifecycle docs complete | G6 PASS + checkpoint/walkthrough/handoff |
| LANG005 release certified | broad release gates pass under separate certification | **not implied by this plan** |

Plan completion metadata should be truthful:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only after G6 and required records are complete. Use repository-equivalent enum values if lifecycle convention changed mechanically.

---

## 26. Plan self-review

The architect self-review for this revised plan is:

- [x] Every substantive open audit finding has an explicit disposition.
- [x] Every integrated audit repair has an early task before trait semantics depend on it.
- [x] C1-F01, C1-F02, C2-F01, and C2-F02 have focused coverage obligations.
- [x] C1-F03/F04/F05/F06 and C2-F04/F05 are explicitly deferred rather than silently dropped.
- [x] C2-F03 is explicitly carried as a hard pre-C4 prerequisite.
- [x] Previously resolved audit concerns are preserved as takeover invariants rather than reopened without evidence.
- [x] The stale external C2.P3 blocker is removed; final C2 interfaces are consumed as landed architecture.
- [x] Current repository revision and final C2 state are recorded.
- [x] Local working-tree cleanliness is not fabricated; T0 must record it.
- [x] The absent historical requirements-analysis file is not invented; the new requirements analysis is the planning authority.
- [x] Every important semantic invariant has a canonical owner.
- [x] Every important semantic risk has test coverage IDs.
- [x] No duplicate semantic source of truth is planned.
- [x] No fake nominal trait type/class-object/runtime class is planned.
- [x] No redundant declaration/member/default identity types are required by habit.
- [x] Trait generic signature storage is independent of `DeclarationTypeTable` nominal machinery.
- [x] `TypeLevelBinding` remains lexical binder infrastructure.
- [x] Index requirement architecture matches the live `IndexMethodDef.body` gap and uses shared normalization.
- [x] Callable body generic ownership is grounded in the live `checker/body.rs` nominal lookup assumption.
- [x] Abstract call design is grounded in live `CallableApplicationTarget { callable, target, authority }`.
- [x] Abstract calls cannot become prematurely executable.
- [x] TraitSurface remains separate from ordinary/conditional/enum behavior products.
- [x] Default body analysis is source-order independent.
- [x] Compiler/runtime ordinary dispatch remains trait-unaware in C3.
- [x] Source/LSP tooling reuses canonical semantic identities.
- [x] Testing budget is selective and broad workspace gates are not mandatory after every task.
- [x] STOP/CONSULT conditions are objective and non-self-waivable.
- [x] Debugging loops are bounded.
- [x] Tasks form a dependency order that minimizes broken intermediate semantic states.
- [x] Checkpoint, walkthrough, and handoff obligations are explicit.
- [x] Specification migration covers the live protocol-era conflict cluster without global vocabulary replacement.
- [x] No unresolved planning placeholders remain.
- [x] Symbols and terminology are internally consistent with the live audited repository or marked mechanically flexible.

The execution principle is simple:

```text
repair the semantic foundations that traits depend on
    ↓
publish one canonical trait contract model
    ↓
keep defaults abstract and semantic
    ↓
leave conformance/runtime witness behavior to C4+
```
