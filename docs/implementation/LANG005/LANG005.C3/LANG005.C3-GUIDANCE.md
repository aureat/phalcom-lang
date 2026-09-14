---
id: LANG005.C3.guidance
category: LANG
program: LANG005
checkpoint: LANG005.C3
kind: implementer-guidance
status: ACTIVE
baseline_revision: 3d4d1a855337738eb86a056721e8c52f1cf3c18b
plan: LANG005.C3.P1
requirements: LANG005.C3.P1-requirements-analysis.md
---

# Implementer Guidance — LANG005.C3 Trait Declarations and Abstract Trait Surfaces

## 0. Purpose

This document is the operating guide for agents implementing work under:

```text
LANG005.C3
    Trait declarations and abstract trait surfaces
```

It is optimized for Luna-oriented patch-grade execution under the architecture fixed by:

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
LANG005.C3-CHECKPOINT.md
LANG005.C3.P1-requirements-analysis.md
LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md
```

Its job is to keep implementation on the intended architecture, make the predecessor repair boundary explicit, prevent accidental C4–C7 work, bound debugging/testing, and make the checkpoint durable.

The P1 plan is the detailed execution contract. This guidance is the persistent operating model.

---

# 1. Current Checkpoint State

The current planning baseline is:

```text
repository:
    aureat/phalcom-lang

branch:
    main

revision:
    3d4d1a855337738eb86a056721e8c52f1cf3c18b

C2.P1:
    COMPLETE / IMPLEMENTED / focused evidence

C2.P2:
    COMPLETE / IMPLEMENTED / focused evidence

C2.P3:
    COMPLETE / IMPLEMENTED / focused evidence

C3:
    PROPOSED / NOT_STARTED / UNVERIFIED

C3.P1:
    PROPOSED / NOT_STARTED / UNVERIFIED
```

The old rule:

> Do not implement C3 until C2.P3 is complete.

is obsolete because C2.P3 is complete.

The new rule is:

> **P1 may begin now at T0, but trait production source work is forbidden until T1/T2 close the integrated C1/C2 takeover repairs and G1 passes.**

The first P1 action is not trait parsing.

The first P1 sequence is:

```text
T0
    local takeover + checkpoint truth

T1
    C1 exact anonymous-product reification
    + dynamic Record logical identity

T2
    C2 exact-case conditional publication/invalidation

G1
    predecessor foundation restored

then
    T3+ trait work
```

---

# 2. Canonical Repository Location

Use the live C3 directory:

```text
docs/implementation/LANG005/LANG005.C3/
    LANG005.C3-CHECKPOINT.md
    LANG005.C3-GUIDANCE.md
    LANG005.C3.P1-requirements-analysis.md
    LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md
```

Do not create a competing renamed C3 directory.

Do not reorganize C1/C2 while implementing C3.

---

# 3. Required Read Order

At the start of a C3.P1 implementation session, read only what is needed:

```text
1. AGENTS.md
2. docs/workflow/implementation-record-lifecycle-convention.md
3. docs/workflow/luna-patch-grade-plan-schema.md

4. LANG005.C3-CHECKPOINT.md
5. LANG005.C3-GUIDANCE.md
6. LANG005.C3.P1-requirements-analysis.md
7. LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md

8. final C1.P3 walkthrough/handoff
9. final C2.P3 walkthrough/handoff
10. integrated C1/C2 audit material named by the plan

11. exact live source files named by the active task
12. exact relevant test README/conventions named by the plan
```

Read normative specification files when the active task reaches the specification boundary or when a semantic question actually depends on them.

Do not begin with a broad repository scan.

Do not rediscover architecture already established by the checkpoint, requirements analysis, plan, and predecessor handoffs unless the live tree contradicts them.

---

# 4. Authority and Drift Rules

## 4.1 Language semantics

Effective normative specification plus ratified Phalcom decisions own language meaning.

Current implementation defects do not override accepted semantics.

Historical `@protocol class` material is not automatically authoritative merely because it is older or already written. C3 includes a coherent protocol→trait specification migration.

## 4.2 Checkpoint state

`LANG005.C3-CHECKPOINT.md` owns durable execution state:

```text
lifecycle
active plan
landed stable interfaces
invariants
verification evidence
audit findings closed/deferred
consultations/amendments
next action
```

## 4.3 Requirements analysis

`LANG005.C3.P1-requirements-analysis.md` owns the fresh architectural requirements derived from:

```text
final C1/C2 implementation
C1/C2 audit
ratified trait decisions
live repository seams
```

Do not resurrect assumptions from the pre-final-C2 plan when the new requirements analysis supersedes them.

## 4.4 P1 plan

`LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md` owns:

```text
task order
task contracts
verification gates
testing budget
STOP/CONSULT triggers
completion deliverables
```

## 4.5 Live repository

Live source owns mechanical reality.

Mechanical drift examples:

```text
helper renamed
private module split
collection type changed
query method renamed
test module moved
```

Adapt those locally.

Architectural drift examples:

```text
semantic owner changed
identity model changed
runtime type-environment model changed
conditional target ownership changed
trait would need fake nominal type
module linkage requires runtime trait descriptor
```

Stop and consult.

## 4.6 Walkthrough/handoff

Once P1 is implemented, its walkthrough/handoff become the operational truth about what actually landed.

Do not force the implementation to match a stale private helper name from the plan after equivalent architecture has landed.

---

# 5. Session Bootstrap

Before editing:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
```

Record:

```text
starting revision
branch
unrelated modified/staged/untracked files
whether current HEAD is the planned baseline or a descendant
```

Preserve unrelated work.

Do not:

```text
git reset --hard
git clean
restore unrelated files
broad-format unrelated source
stage unrelated work
rewrite adjacent subsystems for cleanup
```

If local HEAD is a semantic descendant of the planning baseline, map drift before editing.

A remote repository baseline is not evidence that the local worktree is clean.

---

# 6. T0 — Replace Stale Takeover State, Do Not Re-Audit Everything

T0 exists to establish local truth and lock the audit disposition.

Verify these four integrated findings against live source:

```text
C1-F01
    exact anonymous-product RuntimeTypeRecipe path incomplete
    OR already fixed equivalently

C1-F02
    dynamic Record logical canonicalization incomplete
    OR already fixed equivalently

C2-F01
    exact-case conditional full-target publication incomplete
    OR already fixed equivalently

C2-F02
    owner-complete exact-case conditional invalidation incomplete
    OR already fixed equivalently
```

Verify the final C2 architecture is still repository-equivalent to:

```text
InherentImplTarget
ConditionalInherentMemberSet
semantic applicability/specialization
receiver-effective conditional lookup
semantic lowering selection
ordinary runtime override probe + fallback
```

Then update the checkpoint:

```text
status -> IN_PROGRESS
completion -> PARTIAL
active_plan -> LANG005.C3.P1
starting revision/worktree truth
final C2 takeover map
audit repair/deferred map
G1 barrier
```

Do not reopen already-resolved C2 questions merely because this is a new session.

---

# 7. G1 Is the Hard Internal Barrier

C3 has no external C2.P3 blocker.

Instead:

```text
C3.P1 starts
    ↓
T1 + T2
    ↓
G1
```

G1 must prove:

```text
C1
    semantic exact anonymous-product type
        -> RuntimeTypeRecipe
        -> actual runtime generic environment
        -> materialized exact runtime type

    optimizer rematerialization preserves exact type

    static/dynamic Record structural logical identity converges

C2
    conditional exact-case contribution
        -> full InherentImplTarget product
        -> canonical publication
        -> owner-complete removal/replacement
        -> cold/incremental parity
```

Until G1 passes:

```text
DO NOT add Statement::Trait
DO NOT rename DeclarationKind::Protocol
DO NOT add TraitRef/TraitSurface
DO NOT implement trait default semantics
```

This prevents C3 from building compatibility adapters around known predecessor defects.

---

# 8. C1 Takeover Repair — Architecture to Preserve

Do not redesign C1.

The correct direction already exists:

```text
semantic exact type
    ↓
RuntimeTypeRecipe
    ↓
RuntimeTypeEnvironment
    ↓
instantiate_type_recipe(...)
    ↓
runtime exact type descriptor
```

The defect is incomplete wiring, not the model.

## 8.1 C1-F01

Required result:

```text
anonymous-product lowering spec
    carries enough runtime type recipe information

generic invocation/frame activation
    carries actual generic environment

BuildStaticTuple / BuildStaticRecord
    instantiate recipe using current runtime environment

runtime anonymous product descriptor
    receives canonical exact_type

optimizer rematerialization
    preserves the same recipe/exact type
```

Forbidden:

```text
infer exact type from runtime payload classes
per-value semantic metadata arrays
new generic runtime class per application
compiler-local generic type inference
dropping recipe in optimized path
```

## 8.2 C1-F02

Dynamic Record construction must use the same canonical structural logical-label order as static Record construction.

Keep separate:

```text
source/presentation order
logical structural order
physical storage order
```

Do not reorder source effects merely to canonicalize logical identity.

## 8.3 Deferred C1 findings

Do not expand T1 into general product cleanup.

Defer unless directly forced by the repair:

```text
C1-F03 data-component u16 operand boundary
C1-F04 ProductStorage/layout pairing hardening
C1-F05 Record lookup/equality performance
C1-F06 ProductLayout validation complexity
```

---

# 9. C2 Takeover Repair — Architecture to Preserve

Do not redesign C2 specialized behavior.

Keep:

```text
shared erased generic runtime class
+
semantic per-expression conditional selection
+
compiled fallback handle
+
ordinary strict-subclass override probe
```

Do not:

```text
create runtime classes per applied generic type
globally install specialized conditional methods
move impl-domain solving into VM
teach LSP a second applicability solver
```

## 9.1 C2-F01

Conditional products must be published by the **full** semantic target:

```text
InherentImplTarget::Declaration(...)
InherentImplTarget::ExactEnumCase(...)
```

Do not normalize exact cases back to declaration-only keys.

Consumers should query one canonical product family.

## 9.2 C2-F02

Publication and invalidation must land together.

Owner replacement/removal must delete every conditional target product owned by the declaration:

```text
ordinary declaration target
exact enum case targets
any other canonical target owned by the declaration
```

Do not publish exact-case products in one patch and defer stale removal.

## 9.3 C2-F03

Applicability proof state currently remains too coarse for C4.

C3 should record:

```text
MUST FIX BEFORE C4 CONFORMANCE/WITNESS WORK
```

Do not opportunistically invent conformance proof architecture in C3.

If T8 unexpectedly requires proof distinctions, stop and consult.

## 9.4 Deferred C2 hardening

Do not pull into C3:

```text
C2-F04 conditional callable preindexing
C2-F05 finer ImplId-domain fingerprints
```

unless evidence shows the active C3 path depends on them.

---

# 10. The Semantic Model to Keep Loaded

```text
class
    opaque nominal object abstraction
    owns object/runtime representation

data
    nominal transparent immutable product
    owns components

enum
    nominal closed sum
    owns variants
    closed root/case contracts remain enum-specific

impl
    inherent-behavior provenance/applicability
    not runtime class reopening

trait
    reusable abstract behavioral/conformance contract
    owns no instance representation
```

Core separations:

```text
TraitSurface
    != DeclarationSurface

TraitSurface
    != C2 conditional inherent surface

TraitSurface
    != EnumBehaviorProduct

TraitRequirementId
    != trait source/default CallableId

TraitRequirementId
    != future concrete witness CallableId

TraitRef
    != ordinary inhabitable TypeId

trait
    != class
```

---

# 11. Identity Policy — Reuse Before Inventing

## 11.1 Trait declaration identity

Use:

```rust
DeclarationId
```

with declaration category:

```text
DeclarationKind::Trait
```

Do not create a parallel mandatory `TraitId` universe.

A category-safe wrapper is allowed only when it prevents invalid API states and remains one-to-one with canonical `DeclarationId`.

## 11.2 Trait requirement identity

This is genuinely new semantic identity:

```rust
TraitRequirementId {
    owner: DeclarationId,
    selector: Selector,
    side: DispatchSide,
}
```

It represents the contract obligation.

It is not a concrete implementation/witness identity.

## 11.3 Trait member/default source identity

Reuse:

```rust
CallableId {
    owner: CallableOwnerId::Declaration(trait_decl),
    selector,
    side,
}
```

A bodyful member has:

```text
TraitRequirementId
    contract identity

trait-owned CallableId
    source/default callable identity
```

C4 may later choose:

```text
target-owned CallableId
    concrete witness identity
```

Do not add `TraitMemberId` / `TraitDefaultId` simply because the feature is new.

## 11.4 Source semantic identity

Reuse:

```text
SemanticTargetId::Declaration(DeclarationId)
SemanticTargetId::Callable(CallableId)
```

Add presentation classification, not a duplicate navigation identity.

---

# 12. TraitRef Policy

Conceptual form:

```rust
TraitRef {
    trait_declaration: DeclarationId,
    arguments: Box<[TypeId]>,
}
```

Formation must verify declaration category `Trait`.

A `TraitRef` is not:

```text
ordinary nominal TypeId
trait object
runtime descriptor
conformance proof
witness table
bool conforms-to result
```

Do not add `TypeData::Trait` merely to reuse ordinary nominal application.

Do not insert traits into nominal declaration type metadata.

Use a dedicated trait-reference formation path over type-like syntax.

---

# 13. Trait Generic Ownership

Trait generic binders remain ordinary stable generic binders:

```text
trait declaration parameter
    TypeParameterOwner::Declaration(trait DeclarationId)

trait member generic parameter
    TypeParameterOwner::Callable(trait-owned CallableId)
```

`TypeLevelBinding` remains lexical binder state.

Do not add:

```text
TypeLevelBinding::Trait
TypeParameterOwner::Trait
```

unless a real semantic dimension absent from the existing identity model is proven.

## 13.1 No fake nominal header

Do not call nominal class/object machinery merely to obtain a generic signature.

In particular, do not use `NominalDeclarationHeader::from_signature` as a fake trait type entry.

Trait header metadata must own its generic signature directly.

## 13.2 Callable body analysis

Current callable body analysis assumes owner generics are retrievable through nominal declaration metadata.

Generalize narrowly.

Preferred direction:

```text
CallableBodyRequest / body-analysis context
    receives owner_generic_signature
    or repository-equivalent explicit owner generic environment
```

Classes/enums/data should continue using their canonical existing source.

Traits should supply their trait header generic signature.

Do not make body checking rediscover owner generics from AST.

---

# 14. TraitSurface Policy

The central C3 product is separate:

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

Do not insert it into:

```text
Declared/Effective DeclarationSurface
conditional inherent member index
EnumBehaviorProduct
runtime class method table
```

The future C4 conformance checker will compose these products; C3 does not flatten them prematurely.

---

# 15. Body Classification

Fixed rule:

```text
bodyless trait behavior
    = requirement

bodyful trait behavior
    = same requirement
      + default implementation
```

A default is not an unrelated helper method.

When selector/side is unchanged, bodyless ↔ bodyful transition preserves:

```text
TraitRequirementId
trait-owned CallableId
```

and changes only default availability/body-analysis products.

---

# 16. Shared BehaviorMember — Reuse It

Use the canonical shared syntax category:

```rust
BehaviorMember {
    Method(...),
    Getter(...),
    Setter(...),
    Index(...),
}
```

Do not create a trait-only behavior grammar.

Trait legality is contextual semantic/parser policy layered over the shared member forms.

---

# 17. Shared Index MemberBody Normalization

This is mandatory C3 infrastructure.

Current shape:

```text
IndexMethodDef.body
    executable statement vector
```

Required shared shape:

```text
IndexMethodDef.body
    MemberBody::Declaration
    MemberBody::Block(...)
```

or repository-equivalent.

Rules:

1. no `TraitIndexRequirementDef`;
2. normalize one shared AST;
3. audit every `IndexMethodDef.body` production consumer;
4. preserve bodyful class/index behavior;
5. preserve bodyful inherent-impl index behavior;
6. declaration-only index syntax is legal only in declaration contexts that permit it;
7. include getter and setter requirement forms;
8. update signature/body fingerprints;
9. update compiler/source-index/LSP/optimizer exhaustive matches mechanically;
10. use concrete index types in C3 tests so associated types are not pulled forward.

If equivalent normalization lands before T3, verify/reuse it rather than layering another representation.

---

# 18. Build Complete TraitSurface Before Checking Defaults

Wrong:

```text
read member
check body
publish member
```

Correct:

```text
parse all members
    ↓
build all requirement/callable signatures
    ↓
publish complete TraitSurface
    ↓
check default bodies
```

This is required so:

```text
default calls later-declared requirement
default calls later-declared default
```

work independently of source order.

---

# 19. Abstract Self

Reuse `SelfTypeTerm`.

Do not invent:

```text
TraitSelfType
ProtocolSelfType
FakeConformer
synthetic trait class
```

A default body environment is:

```text
current declaration
    trait DeclarationId

current callable
    trait-owned CallableId

owner generic signature
    trait header product

Self
    owner-relative SelfTypeTerm

available receiver behavior
    complete TraitSurface

fields/storage
    none

superclass
    none

conformance evidence
    none
```

---

# 20. Abstract Contract Calls

A default may call another trait member through `self`.

Resolution is contract-relative.

Expected canonical call target shape:

```text
CallableApplicationTarget
    signature = specialized trait signature
    callable  = Some(trait-owned CallableId)
    target    = None
    authority = abstract trait-contract authority
```

The exact private authority enum name is flexible.

Required behavior:

```text
canonical argument binding
generic callable inference
return checking
diagnostics
effects/contracts where already supported
```

must reuse the ordinary canonical call application machinery.

Forbidden:

```text
new trait-only call checker
fake InvocationTargetId
compile-time binding directly to trait default implementation
runtime dispatch placeholder
future witness selection in C3
```

---

# 21. Defaults Are Checked Once

A default implementation is analyzed once under abstract trait `Self`.

Do not:

```text
copy default body into each conforming type
re-typecheck default per conformance
select witnesses in C3
assume a concrete target
```

C4 later reasons about whether/when a default satisfies a concrete requirement.

---

# 22. No Trait Storage

Trait declarations own no:

```text
fields
stored properties
data components
constructor-managed state
class invariants
ProductLayout
ProductStorage
field slots
superclass state
ordinary ClassId
```

A getter/setter/index requirement expresses a capability.

It does not inject state.

If parser syntax could be mistaken for stored state, diagnose it at the trait declaration boundary.

---

# 23. C3 Does Not Implement Conformance

C3 ends at:

```text
trait declaration
    ↓
TraitSurface
    ├─ TraitRequirementId
    ├─ signatures
    └─ optional trait-owned default CallableId
```

C4 begins at:

```text
Target
+ TraitRef
+ TraitSurface
+ target effective behavior
    ↓
conformance proof
    ↓
witness/default mapping
```

During C3 do not:

```text
parse impl Trait for T
infer structural conformance
select witnesses
construct ConformanceId
build witness maps
perform coherence/overlap
install selected defaults
```

---

# 24. C3 Does Not Implement Associated Types

Associated types are C5.

Do not implement:

```phalcom
type Item
type Cursor
type Output
```

inside traits/conformances during C3.

Do not fake associated types as ordinary generic parameters.

Do not add projection normalization.

Forward-looking support files may contain associated types; treat them as compatibility references only.

---

# 25. C3 Does Not Implement Trait-Conformance Generic Bounds

Trait-conformance constraints are C6.

Do not add new conformance semantics such as:

```text
T: Printable
T conforms Printable
GenericConstraint::Conforms
```

Existing non-trait generic constraint machinery remains available.

Do not reinterpret existing subtype/bound syntax as trait conformance merely because traits now exist.

---

# 26. Class-Side / Metatype Conformance Boundary

Do not settle final metatype conformance syntax in C3.

Do not infer C3 class-side trait semantics from provisional support examples.

If existing `@class` behavior appears inside a trait and no current normative rule has ratified it:

```text
diagnose/defer
or
STOP AND CONSULT if acceptance depends on it
```

Do not pull later metatype conformance design into P1.

---

# 27. Source Index and LSP

Canonical source identities remain:

```text
trait definition
    SemanticTargetId::Declaration(trait DeclarationId)

trait member/default definition
    SemanticTargetId::Callable(trait-owned CallableId)
```

Add `SourceDeclarationKind::Trait` or repository-equivalent presentation.

The LSP may render traits using a dedicated Trait/Interface-like symbol kind depending on client protocol support, but:

```text
LSP does not own trait semantics
LSP does not own trait identity
LSP does not infer conformance
LSP does not re-solve C2 conditional applicability
```

If semantic source-index projection already gives enough data, avoid LSP-specific duplication.

Run focused LSP tests only if LSP-specific code changes.

---

# 28. Incremental Architecture

Prefer separable products:

```text
trait declaration/header product
TraitSurface
default CallableAnalysis
```

## 28.1 Body-only default edit

May change:

```text
default body fingerprint
default CallableAnalysis
body diagnostics
```

Must not change:

```text
DeclarationId
TraitRequirementId
trait-owned CallableId
unchanged signatures
TraitRef identity
unrelated surfaces
```

## 28.2 Signature edit

Must change:

```text
CallableSemanticSignature
TraitSurface fingerprint
dependent semantic lookups
affected default analysis
```

## 28.3 Selector/side edit

Must change:

```text
CallableId
TraitRequirementId
surface member key
dependent references
```

## 28.4 Bodyless ↔ bodyful with same selector/side

Preserve:

```text
TraitRequirementId
trait-owned CallableId
```

Change:

```text
default availability/body product
```

## 28.5 Trait header generic edit

Must invalidate:

```text
trait header generic signature
TraitRef formation/specialization
TraitSurface signatures
dependent defaults
```

## 28.6 Cold/incremental law

For every incremental trait regression:

```text
incremental snapshot result == cold rebuild result
```

This includes the T2 exact-case conditional repair.

---

# 29. Compiler / Runtime Boundary

Preferred C3 behavior:

```text
Statement::Trait
    compile-time / semantic declaration
    no ordinary runtime class allocation
```

Do not create:

```text
ClassId per trait
FinalizeClass
superclass edge
field layout
trait method table installed on targets
runtime trait scan during Invoke
runtime conformance registry
witness/vtable dispatch
```

The compiler consumes semantic facts; it does not reconstruct trait semantics from raw AST.

If module execution/export linkage forces a runtime trait value:

```text
STOP AND CONSULT
```

Do not create a fake class and do not prematurely implement C7 reflection.

---

# 30. Protocol-Era Specification Reconciliation

Audit the active language-feature cluster, not the entire repository vocabulary.

At minimum:

```text
docs/spec/typing/README.md
docs/spec/typing/STATUS.md
docs/spec/typing/01-protocol-foundation.md
docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md
docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md
```

Search active specs for claims about:

```text
@protocol class
Protocol descriptor
signature-only protocol
structural protocol conformance
protocol defaults
class-side protocol requirements
```

Migrate/supersede language-feature authority coherently.

Do **not** replace unrelated uses of English `protocol` such as:

```text
cursor protocol
network protocol
call protocol
API protocol
```

Historical material should remain traceable under repository spec-migration rules.

---

# 31. DeclarationKind::Protocol Migration

The expected action is:

```text
DeclarationKind::Protocol
    ↓
DeclarationKind::Trait
```

Before renaming, verify there is no stable:

```text
serialization schema
generated metadata ABI
persisted cache contract
plugin/API consumer
```

If a real compatibility contract exists:

```text
STOP AND CONSULT
```

Do not keep both semantic declaration categories merely because the stale variant exists.

---

# 32. Fixed Decisions

The implementer must not change these without architectural consultation:

```text
C1 exact product type comes from semantic RuntimeTypeRecipe
dynamic/static Record structural logical identity must converge
C2 conditional products are keyed by full target
C2 conditional invalidation is owner-complete
semantic layer owns conditional applicability

trait is a distinct declaration category
trait is not class sugar
trait owns no storage/layout/superclass
TraitSurface is separate from all inherent/enum surfaces
TraitRequirementId is distinct from future witness identity
trait source/default callable reuses CallableId
TraitRef is not ordinary inhabitable nominal type
trait generic metadata is not fake DeclarationTypeTable metadata
bodyful member = requirement + default
complete TraitSurface exists before default checking
defaults are checked once
Self uses canonical SelfTypeTerm
default calls are contract-relative
abstract contract calls have no runtime target in C3
no conformance/witness work in C3
no associated types in C3
no trait-conformance generic bounds in C3
no final metatype conformance spelling in C3
index requirements use shared AST
runtime ordinary dispatch remains trait-unaware
source identity reuses canonical Declaration/Callable targets
C2-F03 remains hard pre-C4 work
```

---

# 33. Mechanically Flexible Decisions

Adapt locally when semantics remain unchanged:

```text
private helper names
private module/file split
map/set implementation
category-safe wrapper type
exact trait header struct name
exact query key spelling
exact diagnostic code spelling following convention
abstract call authority enum name
test file/module organization
LSP presentation symbol kind
mechanical signature drift
```

Do not use mechanical flexibility to create duplicate semantic owners.

---

# 34. Verify-First Assumptions

Check live source before acting:

| Assumption | Verify in | If false |
|---|---|---|
| C1 product lowering still lacks recipe attachment | `phalcom-core/src/modules/semantic_lowering.rs`, product compiler/materializer | reuse equivalent fix; consult only if architecture differs |
| runtime type environments still use the planned registry/id model | `phalcom-core/src/typing/environment.rs`, frame/send/dispatch paths | adapt mechanical names; consult on ownership change |
| dynamic Record still uses presentation order as logical order | product runtime constructor | omit repaired slice if already canonical |
| exact-case conditional publication is still declaration-keyed | semantic session/dispatch | reuse equivalent fix; consult if semantic owner changed |
| exact-case removal still lacks owner-complete target cleanup | semantic dispatch/session | reuse equivalent fix; consult if lifecycle model changed |
| `BehaviorMember` remains shared | `phalcom-ast/src/ast.rs` | consult if removed/replaced architecturally |
| `IndexMethodDef.body` still lacks declaration state | AST/parser/consumers | skip normalization if equivalent shared representation landed |
| `DeclarationKind::Protocol` remains migration seam | modules declaration metadata | consult on external compatibility contract |
| `CallableId` remains declaration-owned canonical callable identity | semantic identity | consult if changed |
| `TypeParameterOwner` already covers declaration/callable | semantic generic parameter model | consult if changed |
| trait owner generics cannot yet be supplied directly to body analysis | checker body/request path | adapt to equivalent existing input if available |
| `CallableApplicationTarget` permits no runtime target | checker call path | consult if canonical call architecture changed |
| source targets already have declaration/callable variants | semantic identity/source index | reuse |
| compiler supports compile-time-only declaration patterns | compiler declaration path | verify; consult if module linkage requires runtime trait value |
| spec migration convention remains current | `docs/spec*/README.md` / workflow | adapt mechanically |

---

# 35. STOP / CONSULT Triggers

Stop immediately if:

1. a core C1/C2 audit finding cannot be reconciled with live source.
2. C1-F01 requires replacing the runtime recipe/environment architecture.
3. C2 exact-case behavior requires consumer-specific publication logic.
4. trait requires fake class/nominal type representation.
5. `TraitSurface` must be merged into `DeclarationSurface` or C2 conditional products.
6. abstract calls require runtime behavior before C4+.
7. generic ownership/identity must materially change.
8. `TraitRef` can only be implemented as ordinary inhabitable `TypeId`.
9. default checking requires a concrete witness/conformance.
10. associated types are required to implement the C3 core.
11. trait-conformance generic constraints are required to implement the C3 core.
12. final class-object/metatype conformance semantics must be selected.
13. module linkage requires fake runtime trait class/descriptor.
14. shared index support requires trait-only AST.
15. `DeclarationKind::Protocol` is externally serialized/stable.
16. incremental support requires redesigning the semantic database.
17. C2-F03 unexpectedly blocks C3 and would force partial conformance proof architecture.
18. accepted specs, audit, plan, code, and/or tests materially conflict.
19. two plausible fixes have different ownership/identity/runtime consequences.
20. a second source of canonical inference/resolution/identity would be introduced.
21. the same semantic failure survives one serious corrective attempt.
22. an invariant must be weakened to pass tests.
23. scope becomes materially broader than the plan.
24. an unexpected subsystem becomes architecturally necessary.

A triggered condition is not self-waivable.

---

# 36. Bounded Debugging Protocol

## 36.1 Mechanical failure

A coherent correction cycle is:

```text
inspect exact failure
identify concrete local cause
make one coherent correction
rerun smallest discriminator
```

Allow roughly three coherent cycles while each cycle shows real progress.

Examples:

```text
missing import
renamed helper
ordinary borrow checker cleanup
mechanical match exhaustiveness
test filter typo
```

If the problem stops being mechanical, reclassify it.

## 36.2 Semantic failure

Before a nontrivial corrective edit, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Make at most one serious correction for the same underlying semantic failure.

If substantially the same semantic failure remains:

```text
STOP AND CONSULT
```

## 36.3 Architectural contradiction

Zero speculative architecture fixes.

Stop immediately.

## 36.4 Test reruns

Never rerun an unchanged failing test when nothing relevant to that failure changed.

---

# 37. Failure Classification

Use:

```text
A — definitely caused by current change
B — probably caused by current change
C — unclear
D — clearly unrelated/baseline
```

Policy:

```text
A/B
    active responsibility

C
    one bounded classification pass
    record/defer if still unclear and nonblocking

D
    record and continue
```

Do not weaken assertions, skip tests, or change semantic expectations merely to make broad gates green.

An expected feature gap in an intentionally incomplete intermediate task is planned-red state, not baseline class D.

---

# 38. Testing Doctrine

Design broad coverage, execute narrowly.

Verification ladder:

```text
T0 exact reproducer/new regression
T1 directly affected feature
T2 owning subsystem/module
T3 adjacent cross-layer integration
T4 broad crate/package
T5 workspace/release
```

During BUILD mode:

```text
run smallest discriminating feedback
continue through intentionally incomplete intermediate states
do not automatically broaden after PASS
do not run workspace tests after every edit
do not run workspace clippy repeatedly
do not repair unrelated failures
confirm test filters selected tests
run Cargo commands serially when competing builds would waste resources
```

At G1–G5, enter focused STABILIZE mode for the coherent slice.

At G6, run the named focused checkpoint certification.

Do not treat G6 as release certification.

---

# 39. Required Coverage Families

## 39.1 Audit remediation

```text
AR-01 exact generic anonymous product runtime type
AR-02 real runtime generic environment
AR-03 optimized/eager reification equivalence
AR-04 static/dynamic Record logical identity
AR-05 presentation/source order preservation
AR-06 exact-case conditional publication
AR-07 owner-complete exact-case removal
AR-08 conditional cold/incremental parity
AR-09 nominal conditional behavior non-regression
```

## 39.2 Trait syntax/identity

```text
TR-01 trait keyword/ranges
TR-02 module namespace/export identity
TR-03 DeclarationKind::Trait
TR-04 generic trait declaration ownership
TR-05 TraitRef formation/arity/category
TR-06 no fake nominal trait type
TR-07 TraitRequirementId vs source CallableId
TR-08 bodyless method/getter/setter/index requirements
TR-09 bodyful requirement + default identity
TR-10 duplicate selector/side diagnostics
```

## 39.3 Default/Self

```text
DF-01 Self signatures
DF-02 generic default
DF-03 default calls requirement
DF-04 default calls default
DF-05 later-declared member
DF-06 no runtime target
DF-07 canonical call diagnostics
DF-08 illegal storage/super boundary
DF-09 bodyless↔bodyful identity preservation
```

## 39.4 Incremental/tooling/compiler

```text
IN-01 body-only edit
IN-02 signature edit
IN-03 selector/side edit
IN-04 generic header edit
IN-05 cold/incremental parity

LS-01 source target identity
LS-02 LSP presentation

CP-01 no ordinary runtime trait class
CP-02 no runtime member injection/trait scan
CP-03 enum contract separate
CP-04 conditional inherent surface separate
```

## 39.5 Shared index

```text
IX-01 declaration-only index getter
IX-02 declaration-only index setter
IX-03 class bodyful index non-regression
IX-04 impl bodyful index non-regression
IX-05 index surface/body fingerprint separation
```

---

# 40. Gate Execution Rules

## G1

After T1 + T2.

Run only the exact audit-remediation regressions and directly affected owning suites needed to establish:

```text
C1 exact runtime type
Record canonical identity
C2 exact-case publication
C2 owner-complete lifecycle
cold/incremental parity
```

Do not start T3 until G1 passes.

## G2

After T3 + T4 + T5.

Prove:

```text
shared MemberBody index normalization
ordinary index non-regression
trait syntax/header/module identity
TraitRef
TraitRequirementId/source CallableId separation
no nominal trait type
```

## G3

After T6.

Prove complete TraitSurface publication independent of default body analysis.

## G4

After T7 + T8.

Prove abstract Self/default semantics and no runtime target.

## G5

After T9 + T10.

Prove incremental/source/compiler/runtime boundaries.

## G6

After T11 + T12.

Run consolidated focused certification plus:

```sh
cargo fmt --all -- --check
```

Run negative source inspections for forbidden architecture.

Do not automatically run full workspace certification unless explicitly requested or evidence requires it.

---

# 41. Tests Not to Run Repeatedly

During BUILD mode, do not repeatedly run:

```sh
cargo test --workspace --all-targets
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Also avoid broad unrelated suites absent a concrete shared-boundary reason:

```text
REPL
concurrency
entire language corpus
entire ADT/GADT suite
unrelated optimizer suites
```

T1 may require product optimizer tests because optimizer reification equivalence is in scope.

T3 may require one focused bodyful index regression because shared AST changes are in scope.

That does not justify workspace repetition.

---

# 42. Negative Verification

Before C3 completion inspect production code for forbidden shortcuts:

```text
trait -> ClassDef
trait -> ClassId
trait -> superclass
trait -> ProductLayout
trait -> field slots
TraitSurface -> DeclarationSurface insertion
TraitSurface -> conditional inherent product insertion
trait default -> target add_method
VM Invoke -> trait scan
TraitRequirementId == future witness CallableId
TraitRef -> ordinary nominal TypeId
trait generic signature -> fake NominalDeclarationHeader
trait-only duplicate index AST
abstract contract call -> executable InvocationTargetId
```

Also inspect remaining language-feature uses of:

```text
DeclarationKind::Protocol
@protocol class
Protocol descriptor
structural protocol conformance
```

Do not flag unrelated ordinary English uses of `protocol`.

Every intentional historical/compatibility hit should be explained.

---

# 43. Performance / Resource Discipline

C3 is not a performance checkpoint, but T1/T2 have resource invariants.

## 43.1 Runtime type environment

Do not introduce:

```text
per-value generic argument arrays
new runtime class per generic application
eager cloning of whole type metadata graphs
```

Nongeneric paths should continue to use the repository's empty runtime type environment representation without unnecessary allocation.

## 43.2 Conditional targets

Owner-complete invalidation should be bounded by canonical ownership/indexes, not whole-workspace scans.

Do not solve exact-case lifecycle by making every query scan all impl fragments.

## 43.3 Trait runtime footprint

C3 should add no ordinary per-instance trait runtime state.

Negative evidence is sufficient; do not add benchmark ceremony to semantic trait work.

---

# 44. Checkpoint Update Protocol

`LANG005.C3-CHECKPOINT.md` is the durable execution memory.

## At P1 start

Record:

```text
status: IN_PROGRESS
completion: PARTIAL
active_plan: LANG005.C3.P1
local branch/revision/worktree truth
final C2 takeover map
audit repair/deferred map
G1 barrier
```

## During P1

Update only for durable events:

```text
C1 repair established
C2 repair established
G1 passed
trait header/identity APIs stabilized
TraitSurface stabilized
abstract Self/default contract stabilized
incremental/source/compiler boundary stabilized
consultation/amendment
meaningful deferred failure
```

Do not log every edit/test rerun.

## At completion

Record:

```text
P1 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
C3 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
final revision
stable interfaces
invariants
audit findings closed/deferred
C2-F03 hard pre-C4 condition
focused evidence
residual failures
spec migration result
walkthrough
handoff
next action
```

Do not create competing state documents.

Do not put hidden reasoning into checkpoint state.

Record facts, decisions, evidence, and consequences.

---

# 45. Walkthrough Requirement

At P1 completion write:

```text
LANG005.C3.P1-walkthrough.md
```

It must report actual implementation, including:

```text
C1/C2 audit remediations
trait AST/header
IndexMethodDef declaration/body normalization
DeclarationKind::Trait migration
DeclarationId reuse
TraitRequirementId
TraitRef
TraitSurface
trait-owned CallableId
generic ownership
callable body owner-generic generalization
abstract Self
semantic-only contract calls
default body analysis
incremental products/dependencies
source identity
LSP projection
compiler/runtime non-class boundary
protocol-era spec migration
tests added
tests run
tests deferred
deviations
consultations
residual risks
```

Do not restate the plan as if it were a walkthrough.

---

# 46. C4 Handoff Requirement

At P1 completion write:

```text
LANG005.C3.P1-handoff.md
```

It must explain actual APIs for:

```text
recognizing a trait declaration
forming TraitRef
querying TraitSurface
enumerating TraitRequirementId
finding a default's trait-owned CallableId
substituting trait generics
representing Self
visibility
source targets
incremental keys/fingerprints
C2 conditional inherent lookup after remediation
compiler/runtime trait boundary
```

It must explicitly carry:

```text
C2-F03
    hard pre-C4 requirement:
    preserve proof-state/evidence distinctions needed by conformance
```

Reiterate:

```text
conformance != inherent behavior
requirement != witness
default declaration != default selection
```

C4 consumes C3. It does not reopen C3 identity design.

---

# 47. Fast Mental Checklist Before Every Patch

Ask:

```text
What fact am I changing?

Who owns it?

AST?
module declaration identity?
runtime product typing?
C2 conditional surface?
trait header?
trait contract?
call checking?
incremental query?
source identity?
compiler lowering?
runtime execution?
```

Then ask:

```text
Am I reusing a canonical identity?
Am I creating a second source of truth?
Am I building trait on a known predecessor defect?
Am I making trait class-like?
Am I confusing requirement with source callable or future witness?
Am I pulling C4/C5/C6/C7 work forward?
Am I reconstructing semantics in compiler/LSP/VM?
Am I broad-testing instead of gathering targeted evidence?
```

If the dangerous answer is yes, correct course before patching.

---

# 48. Completion Truth

C3 is not complete because:

```text
trait parses
```

or:

```text
one default type-checks
```

or:

```text
semantic tests compile
```

or:

```text
C2.P3 was already complete
```

Completion requires:

```text
C1 takeover repairs closed
C2 exact-case lifecycle repairs closed
G1 passed
shared index declaration/body normalization
first-class trait syntax
DeclarationKind::Trait
canonical DeclarationId identity
TraitRequirementId
TraitRef
TraitSurface
trait generic header ownership
trait-owned CallableId reuse
abstract Self
semantic-only contract calls
contract-relative defaults
incrementality
source/tooling identity
compiler/runtime non-class boundary
protocol→trait specification reconciliation
focused verification
walkthrough
C4 handoff
```

and still:

```text
no conformance
no witness selection
no associated type projection
no trait-conformance generic solver
no runtime trait object/vtable
```

Only then may the checkpoint become:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

This does not imply `RELEASE_COMPLETE`.

---

# 49. Final Agent Rule

> **Start with the predecessor repairs the audit proved are necessary, pass G1, and only then build the trait contract. Reuse Phalcom's existing declaration, callable, generic-owner, `Self`, call-application, and source-target machinery wherever it already expresses the fact. Keep `TraitSurface` separate, defaults abstract and contract-relative, abstract calls non-executable, runtime dispatch trait-unaware, and C4–C7 semantics out of C3.**
