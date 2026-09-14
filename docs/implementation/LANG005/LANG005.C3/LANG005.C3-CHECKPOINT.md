---
id: LANG005.C3
category: LANG
program: LANG005
checkpoint: LANG005.C3
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
requires:
  - LANG005.C2
blocked_by: []
next_plan: LANG005.C3.P1
baseline_revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
requirements: LANG005.C3.P1-requirements-analysis.md
active_plan: LANG005.C3.P1
---

# Checkpoint Record — LANG005.C3 Trait Declarations and Abstract Trait Surfaces

## 1. Objective and Ownership Boundary

`LANG005.C3` establishes **first-class Phalcom trait declarations and their abstract semantic contract surfaces**.

The checkpoint introduces the language category:

```text
trait
    reusable behavioral / conformance contract
    no owned instance representation
```

The bounded acceptance objective is:

```text
source `trait` declaration
    ↓
canonical declaration identity
    ↓
generic TraitRef identity
    ↓
complete abstract TraitSurface
    ├─ TraitRequirementId entries
    ├─ callable signatures
    └─ optional trait-owned defaults
             ↓
       default bodies checked once
       against abstract owner-relative Self
```

C3 owns:

```text
trait declaration syntax
trait declaration classification
generic trait-reference formation
abstract requirement identity
trait callable signatures
default implementation source identity
abstract Self lookup inside defaults
semantic-only contract calls
trait semantic/incremental products
source/tooling identity
compiler acceptance without class/runtime dispatch semantics
protocol-era specification reconciliation
shared index MemberBody normalization required by bodyless index requirements
```

C3 explicitly does **not** own:

```text
impl Trait for Target
conformance proofs
witness selection
coherence / overlap
default selection for concrete conformances
associated types / associated bindings
projection normalization
generic trait-conformance constraints
conditional conformances
trait objects / existential trait values
runtime witness/vtable dispatch
full trait reflection
final metatype-conformance spelling
derived behavior policy
```

Those remain C4–C8 work.

---

## 2. Current Lifecycle State

```yaml
checkpoint: LANG005.C3
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
next_plan: LANG005.C3.P1
external_blocker: none
internal_first_gate: G1
active_plan: LANG005.C3.P1
next_action: LANG005.C3.P1.T1
```

The previous lifecycle state was stale. It recorded C3 as blocked on `LANG005.C2.P3` at revision `986568da050d1bbfa7f1d769c9e1f4d58eb81cf3`.

That is no longer true.

The current planning baseline is:

```text
repository: aureat/phalcom-lang
branch: main
revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
commit: lang005: complete C2 specialized impl applicability
planning date: 2026-09-14
```

`LANG005.C2.P3` is implemented. Its central architecture is a real predecessor:

```text
specialized/constrained inherent impl domain
    ↓
semantic applicability
    ↓
conditional member selection
    ↓
lowering fallback evidence
    ↓
ordinary runtime dispatch probe + selected fallback
```

C3 therefore has **no external C2.P3 blocker**.

However, the integrated C1/C2 audit proved several predecessor defects whose repair is required before trait source semantics may build on the foundation. Those repairs are internal tasks of `LANG005.C3.P1`; they do not restore a stale checkpoint-level `blocked_by: LANG005.C2.P3`.

The rule is:

```text
P1 may start
    ↓
T0 takeover/state capture
    ↓
T1/T2 predecessor remediation
    ↓
G1 must pass
    ↓
only then may trait production source work begin
```

---

## 3. Repository Baseline and Takeover Truth

### 3.1 Verified planning baseline

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
```

T0 recorded the local execution baseline as:

```text
branch: main
revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
worktree: modified/staged=3, deleted=166, untracked=4
```

The worktree contains broad pre-existing documentation/specification deletions, C3 planning edits, and a new requirements-analysis file. These changes are unrelated to T0's checkpoint-only update and are preserved. No active T1/T2 production source conflict was reported by the scoped status check.

A descendant revision is mechanically acceptable when architecture is unchanged. Material semantic drift is an escalation condition.

### 3.2 Current live facts relevant to C3

The final planning audit revalidated:

1. `phalcom-ast::BehaviorMember` remains the canonical shared behavior-only AST category.
2. `phalcom-ast::Statement` has no first-class `Trait` declaration yet.
3. `IndexMethodDef.body` is still executable `Vec<Statement>`-style state rather than shared `MemberBody`.
4. `phalcom-modules::DeclarationKind` still contains `Protocol`, not `Trait`.
5. `DeclarationId` remains the canonical declaration identity.
6. `CallableId { owner, selector, side }` remains the canonical behavioral source/callable identity.
7. `TypeParameterOwner::{Declaration, Callable, Impl}` already covers required generic ownership.
8. `SelfTypeTerm` remains canonical owner-relative `Self`.
9. `TypeLevelBinding` remains lexical generic binding infrastructure; no trait-specific binding variant is needed.
10. `SemanticTargetId` already supports declaration and callable identities needed for trait navigation.
11. `SourceDeclarationKind` needs a trait presentation category but not a new source-target identity family.
12. `CallableApplicationTarget` already has optional runtime `InvocationTargetId` plus `CallTargetAuthority`, which permits semantic-only abstract contract calls.
13. `DeclarationTypeTable` / `NominalDeclarationHeader` remain nominal type/class-object machinery and must not become fake trait type storage.
14. callable body analysis currently obtains owner generics through nominal declaration metadata and requires a narrow generalization for trait defaults.
15. C2 conditional semantic selection, lowering, runtime fallback, GC rooting, applied class-side specialization, and erased shared generic-class isolation are established architecture and must be preserved.

### 3.3 T0 takeover findings

The four integrated findings were verified against the live tree:

| Finding | Live evidence | Disposition |
|---|---|---|
| `C1-F01` | `AnonymousProductConstructionLoweringSpec` carried only `kind` and `layout`; static tuple/record materializers registered descriptors with `exact_type: None`. | T1 partial closure: metadata recipe and exact attachment are implemented; generic call-entry producer remains open and blocks G1. |
| `C1-F02` | Dynamic `finish_record` formed `RecordProductShape` through `from_ordered_labels`, coupling presentation order to logical order. | T1 closed for dynamic construction; focused regression passes. |
| `C2-F01` | Exact-case conditional consumers and `InherentImplTarget::ExactEnumCase` existed, but session publication registered only `InherentImplTarget::Declaration`. | T2 closed: exact-case sets are published under the full target. |
| `C2-F02` | `SurfaceDispatchResolver::remove_surface` removed only the declaration-target conditional key. | T2 closed: owner-indexed target removal is atomic and owner-complete. |

The final C2 architecture remains repository-equivalent to `InherentImplTarget`, `ConditionalInherentMemberSet`, `ConditionalDispatchSelection`, receiver-effective conditional lookup, semantic lowering projection, and shared VM conditional selection. `C2-F03` remains deferred as a hard pre-C4 condition.

### 3.4 P1.T1 implementation slice

The following T1 changes are now present in the live tree:

1. `AnonymousProductConstructionLoweringSpec` carries a metadata-backed `RuntimeTypeRecipe`.
2. Program compilation exports semantic anonymous-product roots and projects them as `RuntimeTypeRef` values into lowering.
3. Static Tuple/Record materialization instantiates the recipe against the current frame environment and publishes `exact_type: Some(...)`; missing template bindings fail closed with a structured runtime error.
4. Dynamic Record materialization preserves encounter order while storing values in canonical label order through a shared shape permutation primitive.
5. VM ownership now includes the runtime environment registry and instantiation overlay context; GC classifies both as non-roots.

Focused evidence:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core                         PASS
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib product::tests    PASS (6)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib typing::environment::tests PASS (2)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::compiler_lowering::family_application_lowering_projects_static_and_dynamic_records PASS
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e::data_record_construction_and_access PASS
```

The T1 generic-entry requirement remains open. The live call path copies an existing block/frame environment but has no generic callsite producer that derives and interns the actual semantic specialization. Adding one requires a compiler/bytecode/runtime contract decision; payload-type inference is forbidden. G1 therefore remains blocked until that producer is supplied and AR-01..AR-05 are added or satisfied.

### 3.5 P1.T2 implementation slice

Exact-case conditional inherent members are now published under `InherentImplTarget::ExactEnumCase`, including unconditional/covering exact-case contributions represented by an exact-case applicability domain. `SurfaceDispatchResolver` retains an owner-to-target reverse index so declaration removal removes the declaration target and every exact-case target atomically.

Focused evidence:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic PASS
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::impls::targets::test_exact_enum_case_target_success PASS
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::impls::queries::incremental_case_only_member_leaves_root_surface_unchanged PASS
```

### 3.6 Implementation incident — T1 generic call-entry producer

```text
Program: LANG005
Checkpoint: LANG005.C3
Plan/task: LANG005.C3.P1.T1 / G1 precondition
Repository: aureat/phalcom-lang
Branch: main
Starting revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
Current revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b plus uncommitted scoped changes
Trigger: the planned compiler/runtime generic-reification contract is absent in the live call path
Invariant: INV-02/INV-03; exact static products must publish canonical exact_type and missing template bindings must fail closed
Expected architecture: semantic recipe -> shared generic call entry -> interned runtime environment -> frame/block propagation -> materialized exact_type
Observed: CallFrame and BlockObject carry RuntimeTypeEnvironmentId, but ordinary closure activation supplies EMPTY and no generic callsite producer derives actual semantic bindings.
Relevant path: compiler Bytecode::Invoke -> VM dispatch/send -> activate_closure_call -> CallFrame::type_environment -> product materialization.
Local attempts: implemented recipe projection, metadata roots, strict template instantiation, VM environment/context ownership, exact descriptor attachment, and canonical dynamic Record storage; all focused checks passed.
Local authority insufficient because adding the producer changes compiler/bytecode/runtime ABI and generic reification ownership; payload-type inference is forbidden.
Decision requested: choose the canonical call-entry source and ABI for deriving/interning semantic generic bindings, including whether it is a new invocation metadata lane or an existing callable specialization product.
Next action after decision: implement producer, add AR-01..AR-04, then run G1 and begin T3.
```

---

## 4. Canonical Checkpoint Location

The live C3 record family is:

```text
docs/implementation/LANG005/LANG005.C3/
    LANG005.C3-CHECKPOINT.md
    LANG005.C3-GUIDANCE.md
    LANG005.C3.P1-requirements-analysis.md
    LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md
```

Do not create a competing `LANG005.C3-trait-declarations-and-abstract-surfaces/` directory.

Do not reorganize C1/C2 as part of C3.

---

## 5. Plan Ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG005.C3.P1` | C1/C2 takeover remediation; shared index body normalization; first-class trait declarations; `TraitRef`; `TraitRequirementId`; `TraitSurface`; abstract `Self` defaults; incremental/tooling/compiler boundary; protocol-era spec migration | IN_PROGRESS | PARTIAL | FOCUSED_TESTED | T0/T2 complete; T1 product/metadata slice implemented; generic call-entry producer remains before G1 |

P1 is intended to close C3 unless implementation discovers a genuinely separate corrective package required to make the checkpoint acceptance objective truthful.

---

## 6. Predecessor Audit Disposition

The C1/C2 audit is part of C3 takeover authority. No substantive finding may disappear.

### 6.1 Must close before trait source work

| Finding | Disposition | P1 owner | Gate consequence |
|---|---|---|---|
| `C1-F01` — anonymous-product runtime generic/reification closure is not wired end-to-end | TAKEOVER REPAIR | T1 | G1 cannot pass until exact runtime type attachment is proven |
| `C1-F02` — dynamic Record logical order follows presentation order instead of canonical structural order | TAKEOVER REPAIR | T1 | close while product identity work is reopened |
| `C2-F01` — conditional exact-enum-case member sets have no canonical full-target publisher | CLOSED | T2 | focused exact-case publication regression passes |
| `C2-F02` — exact-case conditional invalidation/removal is not owner-complete | CLOSED | T2 | focused owner-complete removal regression passes |

### 6.2 Hard pre-C4 prerequisite

| Finding | Disposition |
|---|---|
| `C2-F03` — applicability proof state collapses distinctions needed by future conformance evidence | DEFER FROM C3; **MUST CLOSE BEFORE C4 CONFORMANCE/WITNESS IMPLEMENTATION** |

C4 must be able to distinguish, at minimum:

```text
proved
disproved
not enough evidence
blocked analysis
dynamic boundary
```

and retain proof/witness information where required.

C3 must not prematurely redesign proof state unless trait implementation unexpectedly depends on it. If that occurs, STOP AND CONSULT.

### 6.3 Deferred C1/C2 hardening

The following are recorded but are not C3 semantic prerequisites:

```text
C1-F03
    unchecked data-component u32 -> u16 narrowing

C1-F04
    ProductStorage accepts same-sized but wrong authoritative layout pairing

C1-F05
    Record label lookup / equality can remain unnecessarily expensive

C1-F06
    ProductLayout overlap validation remains quadratic

C2-F04
    conditional callable compiler lookup can scan impl/member inventory

C2-F05
    ImplId domain invalidation can remain coarser than ideal
```

These must not be pulled into C3 merely for cleanup.

### 6.4 Resolved predecessor concerns to preserve

Do not reopen without new evidence:

```text
specialized behavior does not leak across erased generic runtime classes
bound conditional fallback is GC-rooted through owning runtime objects
class-side specialization uses correct semantic identity
LSP/editor delegates conditional applicability to semantic authority
runtime performs no semantic impl-domain solving
ordinary C2 effective/inherent surfaces remain sound
variants-only enum behavior migration is established
```

---

## 7. Adopted Architecture

### 7.1 Trait is a declaration category, not a class kind

Canonical identity:

```rust
DeclarationId
```

Category:

```text
DeclarationKind::Trait
```

The current unused `DeclarationKind::Protocol` is the expected migration seam. T4 must verify there is no serialized ABI/persisted-schema/plugin compatibility dependency before replacing it.

Do not create a second mandatory `TraitId` identity universe.

A category-safe wrapper is mechanically allowed only if it maps one-to-one to `DeclarationId` and prevents invalid API states without becoming a second identity.

### 7.2 Requirement identity is first-class and distinct

Conceptual shape:

```rust
pub struct TraitRequirementId {
    pub owner: DeclarationId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

A trait requirement is not a future concrete witness.

### 7.3 Trait source/default callables reuse CallableId

For every behavioral trait member:

```rust
CallableId {
    owner: CallableOwnerId::Declaration(trait_declaration),
    selector,
    side,
}
```

The callable identity owns:

```text
callable-local generics
CallableSemanticSignature
source/default body identity
body analysis
fingerprinting
presentation/navigation
```

No mandatory `TraitMemberId` or `TraitDefaultId` is required.

### 7.4 Generic TraitRef is a contract reference

Conceptual shape:

```rust
TraitRef {
    trait_declaration: DeclarationId,
    arguments: Box<[TypeId]>,
}
```

The declaration must be verified `DeclarationKind::Trait`.

`TraitRef` is not automatically:

```text
an ordinary inhabitable TypeId
a class object
an existential trait value
a runtime descriptor
a conformance proof
a witness table
```

### 7.5 Trait generic metadata is not nominal type metadata

Trait declaration generic parameters still use:

```text
TypeParameterOwner::Declaration(trait DeclarationId)
```

Member-local generics still use:

```text
TypeParameterOwner::Callable(trait-owned CallableId)
```

But trait headers/generic signatures must not be inserted into `DeclarationTypeTable` through `NominalDeclarationHeader::from_signature`.

C3 requires a trait-owned header/product supplying generic signature information directly.

### 7.6 TraitSurface is separate from all inherent/enum surfaces

The central semantic product is conceptually:

```text
TraitSurface
    owner
    generic signature
    requirements
    callable signatures
    visibility
    default availability/source
    diagnostics
```

It must remain distinct from:

```text
Declared/Effective DeclarationSurface
C2 conditional inherent member sets
EnumBehaviorProduct / EnumRequirementId
runtime class method tables
```

### 7.7 Body classification

```text
bodyless trait behavior
    = requirement

bodyful trait behavior
    = same requirement
      + optional default body/source callable
```

A default does not erase the requirement.

Bodyless ↔ bodyful with unchanged selector/side preserves:

```text
TraitRequirementId
trait-owned CallableId
```

while default availability/body products change.

### 7.8 Abstract Self

Trait signatures/defaults reuse canonical owner-relative `SelfTypeTerm`.

```text
owner
    trait DeclarationId

Self
    existing SelfTypeTerm

available receiver members
    complete TraitSurface

storage
    none

superclass
    none

conformance evidence
    none
```

Do not invent `TraitSelfType`, a fake conformer, or a synthetic trait class.

### 7.9 Complete surface before default bodies

Required sequence:

```text
parse trait members
    ↓
form all requirement/callable signatures
    ↓
publish complete TraitSurface
    ↓
analyze bodyful defaults
```

Source order must not control abstract member availability.

### 7.10 Default calls are contract-relative

A default call such as:

```phalcom
self.compare(other)
```

resolves through the trait contract signature.

It must not hard-bind to another trait default implementation because C4 may later select a concrete witness.

### 7.11 Abstract calls are semantic-only in C3

Canonical application machinery is reused.

Expected shape:

```text
CallableApplicationTarget
    signature = trait member signature
    callable  = Some(trait-owned CallableId)
    target    = None
    authority = abstract trait-contract authority
```

The exact authority enum spelling is mechanically flexible.

There is no executable `InvocationTargetId` for an abstract requirement in C3.

### 7.12 No owned representation

Traits own no:

```text
fields
data components
ProductLayout
ProductStorage
superclass edge
allocation path
ordinary runtime ClassId
```

Getter/setter/index requirements describe capabilities, not storage.

### 7.13 Source identity reuses canonical targets

Prefer:

```text
trait declaration
    -> SemanticTargetId::Declaration(DeclarationId)

trait member/default
    -> SemanticTargetId::Callable(CallableId)
```

Add `SourceDeclarationKind::Trait` or repository-equivalent presentation classification.

Do not add `SemanticTargetId::Trait` merely for presentation.

### 7.14 Compiler/runtime boundary

C3 is semantic-first.

A trait declaration must not create:

```text
ordinary class creation
field slots
ProductLayout
superclass installation
method injection into targets
VM trait scan on ordinary sends
runtime conformance registry
witness/vtable construction
```

If module linkage genuinely requires runtime trait reification before C7, STOP AND CONSULT.

---

## 8. Shared Index Member Normalization

`BehaviorMember::Index(IndexMethodDef)` currently cannot represent a bodyless trait requirement because `IndexMethodDef.body` is executable statement state.

C3 therefore owns a shared normalization before trait index requirements can be complete.

Preferred shape:

```text
IndexMethodDef.body
    MemberBody::Declaration
    MemberBody::Block(...)
```

or repository-equivalent.

Rules:

1. no trait-only index AST;
2. normalize the shared representation once;
3. audit all `IndexMethodDef.body` consumers;
4. preserve bodyful class and inherent-impl behavior exactly;
5. only declaration contexts that permit abstract members may accept declaration-only form;
6. include both index getter and index setter requirements;
7. update structural/body fingerprints appropriately;
8. update compiler/source-index/LSP/optimizer exhaustive matches mechanically;
9. if live repository drift already provides equivalent normalization, reuse it.

---

## 9. LANG005 Support-File Scope

### Direct C3 shape references

```text
docs/implementation/LANG005/support/comparable.ph
docs/implementation/LANG005/support/sized.ph
```

They demonstrate:

```text
generic trait parameters
bodyless requirements
bodyful defaults
getter-shaped requirements
default calls through self
```

### Forward-compatibility references only

```text
iterable.ph
indexable.ph
core_trait_impls.ph
metatype_conformance_example.ph
```

They include later work:

```text
associated types         -> C5
impl Trait for Target    -> C4
associated bindings      -> C5
generic conformance      -> C6
metatype conformance     -> later explicit design
```

Use these files to avoid painting later checkpoints into a corner, not as permission to implement later features.

For C3 index tests, use concrete types rather than C5 associated types.

---

## 10. Protocol-Era Specification Migration

C3 must reconcile the active/conflicting protocol language-feature authority cluster.

At minimum inspect:

```text
docs/spec/typing/README.md
docs/spec/typing/STATUS.md
docs/spec/typing/01-protocol-foundation.md
docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md
docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md
```

Search active specification material for language-feature claims about:

```text
@protocol class
Protocol descriptor
signature-only protocol
structural protocol conformance
protocol defaults
class-side protocol requirements
```

Do not globally replace the ordinary English word `protocol`.

Historical material must be archived or explicitly marked superseded according to the repository's specification migration rules.

---

## 11. Global Invariant Ledger

These are accepted architecture constraints. They are not claims that source implementation already exists.

`INV-01` — Semantic declaration identity, exact type identity, callable identity, runtime descriptor identity, and physical layout identity remain distinct.

`INV-02` — Anonymous products with statically exact generic semantic types materialize with the corresponding exact runtime type.

`INV-03` — Exact anonymous-product runtime type is projected from semantics as `RuntimeTypeRecipe`; it is not inferred from runtime payload values.

`INV-04` — Product optimizer rematerialization preserves the same runtime type recipe/exact type as eager construction.

`INV-05` — Static and dynamic Records with the same structural labels use one canonical logical label order; presentation/source order remains independent.

`INV-06` — C2 conditional member products are keyed/published by full `InherentImplTarget`, including `ExactEnumCase`.

`INV-07` — Conditional target replacement/removal is owner-complete; incremental snapshots cannot retain stale exact-case products.

`INV-08` — Conditional applicability remains semantic authority; runtime/compiler/LSP do not re-solve impl domains.

`INV-09` — Trait is a distinct semantic declaration category, not class sugar.

`INV-10` — Trait declaration identity participates in the existing `DeclarationId` universe.

`INV-11` — Trait requirements have identity distinct from future concrete witnesses.

`INV-12` — Trait-declared source/default callables reuse canonical `CallableId` unless live evidence proves a missing semantic dimension.

`INV-13` — `TraitRef` is a contract reference, not automatically an inhabitable runtime value type.

`INV-14` — Trait generic signatures are trait metadata, not fake nominal `DeclarationTypeTable` entries.

`INV-15` — `TraitSurface` is distinct from ordinary inherent, conditional inherent, and enum behavior products.

`INV-16` — Every behavioral trait member defines a requirement.

`INV-17` — A bodyful trait member is requirement + default, not a second unrelated method.

`INV-18` — Complete `TraitSurface` publication precedes default body analysis.

`INV-19` — Defaults are checked once under abstract owner-relative `Self`.

`INV-20` — Default calls to trait members are contract-relative.

`INV-21` — Abstract trait-member calls carry no executable runtime target in C3.

`INV-22` — Traits own no fields, product layout, superclass edge, ordinary allocation path, or ordinary runtime class.

`INV-23` — Shared index declaration/body normalization serves trait requirements; no trait-only index grammar exists.

`INV-24` — Source navigation reuses canonical declaration/callable target identity.

`INV-25` — Body-only default edits preserve requirement/callable identity and do not invalidate unrelated trait signatures.

`INV-26` — Signature/header edits invalidate the precise trait surface/dependent queries and cold/incremental analysis agrees.

`INV-27` — Enum closed-contract semantics remain distinct from trait semantics.

`INV-28` — C2 conditional inherent applicability remains distinct from trait conformance.

`INV-29` — C3 creates no target conformance/witness relation.

`INV-30` — Associated types remain C5 and generic trait-conformance constraints remain C6.

`INV-31` — Runtime dispatch remains trait-unaware in C3.

`INV-32` — Protocol-era language-feature authority is reconciled without global vocabulary replacement.

`INV-33` — C2 proof-state granularity is a hard pre-C4 condition; C3 does not normalize away that obligation.

---

## 12. Stable Interface / Takeover Map

### Final C1/C2 interfaces inherited by C3

| Concept | Live owner / repository-equivalent symbol | C3 rule |
|---|---|---|
| declaration identity | `phalcom_modules::DeclarationId` | reuse |
| declaration kind | `phalcom_modules::DeclarationKind` | migrate `Protocol` seam to `Trait` |
| behavior syntax | `phalcom_ast::BehaviorMember` | reuse |
| callable identity | `phalcom_semantic::CallableId` | reuse |
| callable owner | `phalcom_semantic::CallableOwnerId` | trait member owner is declaration |
| impl provenance | `phalcom_semantic::ImplId` | preserve |
| inherent impl target | `phalcom_semantic::impls::InherentImplTarget` | full target must remain canonical |
| conditional member product | `ConditionalInherentMemberSet` or repository-equivalent | publish by full target |
| inherent applicability | `ImplApplicabilityResult` + specialization/evidence products | preserve; proof-state hardening deferred to pre-C4 |
| ordinary effective behavior | `DeclarationSurface` / dispatch resolver | do not merge TraitSurface |
| enum requirement | `EnumRequirementId` | remain enum-specific |
| enum behavior | `EnumBehaviorProduct` | remain enum-specific |
| owner-relative Self | `SelfTypeTerm` | reuse |
| callable application | `CallableApplicationTarget` | reuse with no runtime target for abstract contract call |
| source semantic target | `SemanticTargetId` | reuse declaration/callable |
| anonymous-product runtime recipe machinery | `RuntimeTypeRecipe`, `RuntimeTypeEnvironmentId`, environment registry | complete wiring; do not replace |

### C3 interfaces to establish

Exact private names may adapt mechanically:

```text
DeclarationKind::Trait
Trait declaration AST/header
TraitRequirementId
TraitRef
TraitSurface
trait header/generic-signature product
trait-owned CallableId default source
trait-aware abstract Self lookup adapter
abstract trait-contract call authority
trait surface/default query keys and fingerprints
SourceDeclarationKind::Trait or repository-equivalent
```

---

## 13. P1 Task Ledger

| Task | Scope | State before implementation |
|---|---|---|
| `T0` | final takeover, local-state capture, audit disposition lock | COMPLETE |
| `T1` | C1 exact runtime reification + Record logical identity closure | PARTIAL |
| `T2` | C2 exact-case conditional product publication + owner-complete lifecycle | COMPLETE |
| `T3` | shared index `MemberBody` normalization | NOT_STARTED |
| `T4` | canonical trait syntax, module declaration kind, trait header | NOT_STARTED |
| `T5` | `TraitRef` + requirement/source identity foundations | NOT_STARTED |
| `T6` | complete `TraitSurface` + signature publication | NOT_STARTED |
| `T7` | callable-body owner-generic input generalization | NOT_STARTED |
| `T8` | abstract `Self`, semantic-only contract calls, default analysis | NOT_STARTED |
| `T9` | incremental products, source index, LSP projection | NOT_STARTED |
| `T10` | compiler/runtime trait non-class boundary | NOT_STARTED |
| `T11` | protocol-era specification migration | NOT_STARTED |
| `T12` | focused stabilization, checkpoint closure, walkthrough, C4 handoff | NOT_STARTED |

No production implementation work has been recorded by this checkpoint update.

---

## 14. Verification Gate Sequence

### G1 — Predecessor identity/surface foundation restored

After T1 + T2.

Must prove:

```text
semantic exact anonymous-product type
    -> RuntimeTypeRecipe
    -> actual runtime generic environment
    -> materialized exact descriptor type

static/dynamic Record structural logical identity converges

conditional exact-case contribution
    -> full-target semantic product
    -> snapshot/dispatch publication
    -> owner-complete replacement/removal
    -> cold/incremental parity
```

**Hard rule:** do not begin trait production source work before G1 passes.

### G2 — Shared member shape + trait declaration/header identity

After T3 + T4 + T5.

Must prove:

```text
bodyless index is representable through shared MemberBody
ordinary bodyful index behavior remains correct
Statement::Trait is first-class
DeclarationKind::Trait is canonical
DeclarationId remains declaration identity
trait header generics have declaration ownership
TraitRef forms without fake nominal type
TraitRequirementId / trait-owned CallableId separation is correct
```

### G3 — Complete TraitSurface

After T6.

Must prove:

```text
all method/getter/setter/index requirement signatures publish
bodyful members publish default availability without requiring body analysis
later-declared member signatures are visible
duplicates/conflicts are diagnosed
visibility/source metadata survives
TraitSurface is not inserted into inherent/conditional/enum surfaces
```

### G4 — Abstract Self/default semantics

After T7 + T8.

Must prove:

```text
trait owner generics are available without nominal trait type metadata
Self is canonical SelfTypeTerm
defaults may call requirements/defaults including later declarations
abstract contract call uses canonical call checking
runtime target is absent
no conformance/witness exists
stateful/super/class-side deferred boundaries are enforced
```

### G5 — Incremental/source/compiler boundaries

After T9 + T10.

Must prove:

```text
body-only/signature/header invalidation laws hold
cold/incremental parity holds
source targets remain Declaration/Callable
trait presentation category exists
compiler accepts trait without ordinary runtime class creation
no runtime member injection/trait scanning occurs
enum and C2 conditional paths remain distinct
```

### G6 — Focused final certification

After T11 + T12.

Must prove:

```text
focused C1/C2 remediation evidence still passes
focused trait semantics pass
shared index regressions pass
spec migration is coherent
negative architecture searches pass
cargo fmt --all -- --check passes
checkpoint/walkthrough/handoff are truthful
```

G6 is checkpoint certification, not LANG005 release certification.

---

## 15. Verification Ledger

C3 implementation verification is `FOCUSED_TESTED` for the completed T1 product/metadata slice; the plan remains incomplete and G1 is blocked on generic entry plus T2.

Planning/repository evidence:

| Evidence ID | Scope | Result | Meaning |
|---|---|---|---|
| `PLAN-01` | repository baseline | PASS | final C2 revision `3d4d1a8...` verified |
| `PLAN-02` | C2 status | PASS | C2.P3 implemented; stale external blocker removed |
| `PLAN-03` | shared behavior AST | PASS | `BehaviorMember` remains canonical |
| `PLAN-04` | declaration kinds | PASS | `Protocol` seam still present; `Trait` not yet implemented |
| `PLAN-05` | callable identity | PASS | declaration-owned `CallableId` remains canonical |
| `PLAN-06` | source targets | PASS | declaration/callable targets already sufficient |
| `PLAN-07` | index member body | PASS | declaration-only index still requires shared normalization |
| `PLAN-08` | C1 reification audit | PARTIAL | static product recipes and exact descriptor attachment are implemented; generic call-entry specialization remains open |
| `PLAN-09` | dynamic Record audit | PASS | dynamic records now preserve presentation order while using canonical logical/storage coordinates |
| `PLAN-10` | C2 exact-case publication audit | OPEN | conditional exact-case product publication is incomplete |
| `PLAN-11` | C2 exact-case invalidation audit | OPEN | owner-complete removal is incomplete |
| `PLAN-12` | C2 proof-state audit | DEFERRED-PRE-C4 | proof-state granularity must be upgraded before conformance work |

These are planning/takeover facts, not feature test certification.

---

## 16. Deferred / Baseline Issue Ledger

### DEF-001 — C1 data-component operand narrowing

```text
finding: C1-F03
state: deferred
reason: not a C3 semantic prerequisite
```

Do not silently fold into trait work.

### DEF-002 — Product storage/layout hardening

```text
finding: C1-F04
state: deferred
reason: representation API hardening, not C3 dependency
```

### DEF-003 — Product lookup/layout performance

```text
findings:
    C1-F05
    C1-F06
state: deferred
reason: bounded performance work; no C3 architectural dependency
```

### DEF-004 — Conditional compiler lookup indexing

```text
finding: C2-F04
state: deferred
reason: optimization/lookup quality; canonical semantic model already exists
```

### DEF-005 — Impl-domain fingerprint precision

```text
finding: C2-F05
state: deferred
reason: coarse invalidation may be improved later without redesigning C3
```

### PRE-C4-001 — Applicability proof-state distinction

```text
finding: C2-F03
state: mandatory before C4 conformance/witness machinery
```

Do not begin C4 proof/witness implementation while the semantic result model cannot distinguish the evidence states C4 needs.

---

## 17. Consultation / Amendment Ledger

### Adopted planning amendments

`C3-A01` — Lifecycle normalization: remove stale external C2.P3 blocker and use current repository lifecycle metadata.

`C3-A02` — Final C2 takeover is real architecture, not prospective API design.

`C3-A03` — Integrate C1-F01/F02 and C2-F01/F02 as early P1 takeover repairs.

`C3-A04` — Keep C2-F03 as a hard pre-C4 condition rather than pulling conformance proof design into C3.

`C3-A05` — Minimize new identity types: `DeclarationId`, `CallableId`, and existing source targets remain canonical; add only `TraitRequirementId` and `TraitRef` for genuinely new semantics.

`C3-A06` — Trait generic signatures/header metadata remain outside nominal `DeclarationTypeTable`.

`C3-A07` — Abstract trait calls reuse canonical callable application with no executable runtime target.

`C3-A08` — Shared index body representation is normalized once; no trait-only index AST.

`C3-A09` — Protocol-era specification migration covers the active conflicting language-feature cluster, not global English vocabulary.

`C3-A10` — Runtime trait reification/reflection remains deferred; module-linkage pressure for a runtime descriptor is an escalation condition.

`C3-A11` — Testing follows BUILD/STABILIZE/CERTIFY doctrine and repository A/B/C/D failure classification.

### Future amendment format

Any material future amendment must record:

```text
Incident
Plan/task
Trigger
Observed evidence
Decision
Architectural consequence
Plan amendment
Additional tests
```

Do not paste hidden reasoning/model conversations into the checkpoint.

---

## 18. Testing and Verification Policy

Use the repository ladder:

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

During BUILD mode:

```text
run the smallest test that answers the current correctness question
do not broaden automatically after PASS
do not rerun unchanged failures
do not repeatedly run workspace tests/build/clippy
confirm filters select nonzero tests
```

At named stabilization gates, run focused owning suites as specified by P1.

Broad workspace/release certification is not implied by C3 completion.

Failure classification:

```text
A — definitely caused by current patch
B — probably caused by current patch
C — unclear
D — clearly unrelated/baseline
```

A/B are active responsibility.

C receives one bounded classification pass.

D is recorded and deferred.

Expected not-yet-implemented behavior inside an unfinished task is planned-red state, not a baseline failure.

---

## 19. Required Coverage Families

### Audit remediation

```text
AR-01 generic anonymous-product exact type reaches runtime materialization
AR-02 runtime environment reflects actual generic frame/application arguments
AR-03 optimizer rematerialization preserves exact type
AR-04 static/dynamic Record structural logical identity converges
AR-05 presentation/source order remains independent from logical order
AR-06 exact-case conditional product is published by full target
AR-07 exact-case conditional replacement/removal is owner-complete
AR-08 exact-case constraint/body/variant changes preserve cold/incremental parity
AR-09 ordinary nominal conditional behavior remains unchanged
```

### Trait syntax/identity

```text
TR-01 trait keyword/ranges
TR-02 module declaration/export identity
TR-03 DeclarationKind::Trait
TR-04 generic trait header/binder ownership
TR-05 TraitRef arity/kind/category checks
TR-06 no nominal trait TypeId/ClassObject entry
TR-07 TraitRequirementId vs source CallableId
TR-08 bodyless method/getter/setter/index requirements
TR-09 bodyful requirement + default identity
TR-10 duplicate selector/side diagnostics
```

### Defaults/Self

```text
DF-01 Self in signatures
DF-02 generic default body
DF-03 default calls requirement
DF-04 default calls default
DF-05 later-declared member availability
DF-06 abstract call target has no runtime InvocationTargetId
DF-07 canonical call diagnostics reused
DF-08 illegal storage/super boundaries
DF-09 bodyless↔bodyful preserves requirement/source identity
```

### Incremental/tooling/compiler

```text
IN-01 body-only default edit
IN-02 signature edit
IN-03 selector/side edit
IN-04 trait header generic edit
IN-05 cold/incremental parity

LS-01 source declaration/member targets
LS-02 LSP presentation without LSP-local trait semantics

CP-01 trait compile path creates no ordinary runtime class
CP-02 no runtime member injection/trait scan
CP-03 enum contract remains distinct
CP-04 conditional inherent surface remains distinct
```

### Shared index regressions

```text
IX-01 declaration-only index getter
IX-02 declaration-only index setter
IX-03 ordinary bodyful class index unchanged
IX-04 ordinary bodyful impl index unchanged
IX-05 index surface/body fingerprints separate correctly
```

---

## 20. STOP / CONSULT Triggers

A trigger cannot be self-waived.

Stop and consult if:

1. C1-F01 requires replacing rather than completing the existing `RuntimeTypeRecipe` / runtime-environment architecture.
2. C2-F01/F02 require consumer-specific exact-case logic rather than canonical full-target publication plus owner-complete invalidation.
3. trait requires fake class/nominal type representation.
4. `TraitSurface` would need insertion into `DeclarationSurface` or the C2 conditional surface.
5. a trait requirement needs an executable runtime target before C4+.
6. trait generic ownership requires changing the canonical `TypeParameterOwner` identity model.
7. `TraitRef` can only be implemented by making traits ordinary inhabitable nominal value types.
8. default checking requires a concrete conformance/witness.
9. source tooling appears to need a new identity only for presentation.
10. shared index declaration-only support requires a trait-only AST rather than safe shared normalization.
11. module linkage requires a runtime trait class/descriptor before the reflection checkpoint.
12. incremental support appears to require redesigning the semantic DB rather than adding ordinary canonical products/dependencies.
13. C2 proof-state collapse unexpectedly blocks C3 core semantics; do not silently implement partial C4 proof architecture.
14. accepted spec, audit, plan, live source, and tests materially conflict.
15. two plausible fixes have materially different ownership/identity/runtime consequences.
16. one serious hypothesis-driven correction fails for the same semantic defect.
17. a fixed invariant would need to be weakened to make tests pass.
18. scope becomes materially more cross-cutting than P1 defines.

Architectural contradictions permit zero speculative architecture patches.

---

## 21. Checkpoint Update Protocol

### At P1 start / T0

Record:

```text
local git status / branch / revision
active_plan = LANG005.C3.P1
P1 status = IN_PROGRESS
current descendant baseline
final C2 takeover symbol map
audit finding disposition
G1 as internal barrier to trait production source
```

Update front matter to:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG005.C3.P1
```

unless repository lifecycle convention has mechanically changed.

### During P1

Update only for durable events:

```text
T1 closes C1 reification/Record identity invariant
T2 closes full-target conditional publication/lifecycle
G1 closes predecessor remediation barrier
T4/T5 establish trait declaration/header/identity interfaces
G3 establishes complete TraitSurface
G4 establishes abstract Self/default-call semantics
G5 establishes incremental/source/compiler boundary
consultation/amendment
meaningful deferred/baseline issue
```

Do not turn this record into a chronological test log.

### At P1 completion

Record:

```text
plan completion/verification
final revision
final stable interfaces
final invariant ledger
audit findings closed/deferred
C2-F03 hard pre-C4 condition
focused verification evidence
baseline/deferred failures
consultations/amendments
spec migration disposition
walkthrough path
handoff path
next action
```

Successful C3 lifecycle:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
active_plan: null
```

`RELEASE_COMPLETE` requires separately named broader evidence.

---

## 22. Walkthrough Deliverable

At P1 completion create:

```text
LANG005.C3.P1-walkthrough.md
```

It must report what actually landed:

```text
C1/C2 audit remediations
trait AST/header
shared index MemberBody normalization
DeclarationKind::Trait migration
canonical DeclarationId use
TraitRequirementId
TraitRef
TraitSurface
trait-owned CallableId use
generic ownership
abstract Self
semantic-only contract calls
default body analysis
incremental dependency graph
source target reuse
LSP presentation
compiler/runtime non-class boundary
protocol-spec migration
tests added
tests actually run
tests deferred
plan deviations
consultation decisions
residual risks
```

---

## 23. C4 Handoff Deliverable

At P1 completion create:

```text
LANG005.C3.P1-handoff.md
```

C4 must be able to consume C3 without reopening its architecture.

The handoff must state the actual landed APIs/mechanics for:

```text
recognizing a trait DeclarationId
forming TraitRef
querying TraitSurface
enumerating TraitRequirementId entries
finding a trait-owned default CallableId
substituting trait generics
representing Self
visibility
source targets
incremental keys/fingerprints
C2 conditional inherent lookup after remediation
compiler/runtime trait boundary
```

It must also state:

```text
which C1/C2 audit findings were closed
which remain deferred
that C2-F03 must be resolved before C4 conformance proof/witness machinery
```

Reiterate:

```text
conformance != inherent behavior
requirement != witness
default declaration != default selection
```

---

## 24. Completion Criteria

C3 is complete only when all of these are true:

```text
C1 exact anonymous-product reification repair is verified
dynamic/static Record canonical structural identity is verified
C2 exact-case conditional publication/removal is verified
dedicated trait syntax exists
DeclarationKind::Trait is canonical
trait declaration identity uses DeclarationId
TraitRequirementId exists
TraitRef exists without fake nominal type semantics
TraitSurface exists separately from inherent/enum surfaces
trait generic metadata is owned outside nominal DeclarationTypeTable
bodyless index requirements use shared AST
defaults are checked once against abstract Self
default calls are contract-relative
abstract calls have no executable runtime target
trait semantic products are incrementally stable
source indexing reuses canonical semantic identities
compiler creates no ordinary trait runtime class semantics
protocol-era normative conflict is reconciled
focused verification passes
walkthrough exists
C4 handoff exists
```

and still:

```text
no target conforms yet
no concrete witness has been selected
no associated type projection exists
no generic conformance solver exists
no runtime trait object/vtable exists
```

C3 completion does **not** imply full LANG005 release certification.

---

## 25. Active Plan and Next Action

### Active plan

`LANG005.C3.P1`

### Next plan

```text
LANG005.C3.P1
    First-Class Trait Declarations and Abstract Trait Surfaces
```

### Current next action

```text
Continue LANG005.C3.P1.T1 — complete the generic call-entry producer for C1 exact runtime reification, then execute T2's exact-case publication/removal repair.

T1 and T2 must complete before G1. Do not add Statement::Trait, rename DeclarationKind::Protocol, or implement TraitRef/TraitSurface/default semantics before G1 passes.
```
