---
id: LANG005.C3
category: LANG
program: LANG005
checkpoint: LANG005.C3
kind: checkpoint-record
status: BLOCKED
completion: NOT_STARTED
verification: UNVERIFIED
requires:
  - LANG005.C2
blocked_by:
  - LANG005.C2.P3
next_plan: LANG005.C3.P1
baseline_revision: 986568da050d1bbfa7f1d769c9e1f4d58eb81cf3
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
generic trait reference identity
    ↓
abstract TraitSurface
    ├─ requirements
    └─ optional defaults
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
trait semantic/incremental products
source/tooling identity
compiler acceptance without class/runtime dispatch semantics
normative protocol→trait specification reconciliation
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
status: BLOCKED
completion: NOT_STARTED
verification: UNVERIFIED
next_plan: LANG005.C3.P1
blocker: LANG005.C2.P3
```

C3 is initialized and planned, but source implementation must not start until the complete C2 checkpoint is truthful and C2.P3's takeover interfaces are verified.

### Why C2.P3 is a hard predecessor

C4 will eventually consider multiple possible witness sources:

```text
ordinary declared/inherent behavior
C2 receiver-conditional inherent behavior
trait defaults
conformance-local implementations
```

C3 must therefore be built on the final C2 behavioral-surface architecture, not on the current P1/P2-only state.

---

## 3. Repository Baseline and Audit Anchor

Initial C3 repository audit:

```text
repository: aureat/phalcom-lang
branch: main
revision: 986568da050d1bbfa7f1d769c9e1f4d58eb81cf3
audit date: 2026-09-14
```

Latest audited commit:

```text
feat(semantic,core,ast): complete LANG005.C2.P1 and P2
inherent impl and variants-only enum migration
```

At this revision the canonical C2 checkpoint reports:

```text
LANG005.C2.P1
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

LANG005.C2.P2
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

LANG005.C2.P3
    PLANNED / NOT_STARTED / UNVERIFIED
```

Therefore C3 remains blocked.

### Current repository facts verified during this audit

1. `phalcom-ast::BehaviorMember` exists and is the shared behavior-only AST category for methods/getters/setters/index members.
2. `phalcom-semantic::CallableId` is already the canonical behavioral callable identity:
   ```text
   CallableOwnerId + Selector + DispatchSide
   ```
3. `CallableOwnerId::Declaration(DeclarationId)` is already suitable for a callable declared by a trait declaration; no trait-specific callable owner variant is required merely for category naming.
4. `phalcom-modules::DeclarationKind` currently contains an unused `Protocol` variant. No production `DeclarationKind::Protocol` use was found in the audited tree.
5. `phalcom-semantic::SemanticTargetId` already has:
   ```text
   Declaration(DeclarationId)
   Callable(CallableId)
   ```
   so source tooling does not require new trait-specific target variants merely to identify a trait declaration/member.
6. `EnumRequirementId` demonstrates the repository's established pattern for a semantic requirement identity that is distinct from its implementation callable.
7. `IndexMethodDef` still stores an executable statement body rather than `MemberBody`, so declaration-only index requirements are **not** representable by the current shared AST.
8. `docs/implementation/LANG005/support/indexable.ph` explicitly shows intended bodyless index-getter/index-setter trait requirements.
9. `docs/implementation/LANG005/support/comparable.ph` provides a direct C3-level trait/default example.
10. `iterable.ph`, `core_trait_impls.ph`, and `metatype_conformance_example.ph` contain later-checkpoint syntax such as associated types, conformance impls, and placeholder metatype target spelling; they are future-facing design references, not C3.P1 scope.
11. The protocol-era typing specification still asserts `@protocol class`, signature-only protocols, and structural conformance direction; C3 must reconcile that entire authority cluster, not only one file.
12. The repository's implementation-record lifecycle convention applies to new work and requires YAML metadata plus explicit checkpoint ledgers.

---

## 4. Canonical Checkpoint Location

When these records are added to the repository, use the LANG005 family's existing directory style:

```text
docs/implementation/LANG005/
  LANG005.C3-trait-declarations-and-abstract-surfaces/
    LANG005.C3-CHECKPOINT.md
    LANG005.C3-GUIDANCE.md
    LANG005.C3.P1-requirements-analysis.md
    LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md
```

Do not opportunistically reorganize C1/C2 while introducing C3.

---

## 5. Plan Ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG005.C3.P1` | First-class trait declarations, abstract trait surfaces, abstract-Self defaults, incremental/tooling/compiler boundary | PLANNED | NOT_STARTED | UNVERIFIED | blocked on C2.P3 |

C3 currently has one planned implementation package. The intended result is that P1 closes C3 unless implementation discovers a genuinely separate corrective package required to make this acceptance objective truthful.

---

## 6. Adopted Architecture

### 6.1 Trait is a declaration category, not a second class kind

The canonical declaration identity remains the repository's existing:

```rust
DeclarationId
```

Category is carried by the declaration shell:

```text
DeclarationKind::Trait
```

The current unused `DeclarationKind::Protocol` is the expected migration seam. The default implementation action is to rename/replace it with `Trait`, unless takeover inspection finds a real external compatibility contract that was not visible in this audit.

Do **not** create a separate mandatory `TraitId` wrapper merely to duplicate `DeclarationId`.

A category-safe helper/newtype is mechanically permitted if it prevents invalid states and remains cheap, but semantic identity is still the canonical declaration identity plus the fact that its declaration kind is `Trait`.

### 6.2 Requirement identity is distinct and first-class

C3 requires a semantic requirement identity analogous to the existing enum contract pattern.

Recommended shape:

```rust
pub struct TraitRequirementId {
    pub owner: DeclarationId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

Exact repository naming may differ.

This identity is not a future concrete witness `CallableId`.

### 6.3 Reuse CallableId for the trait member/default source callable

For every behavioral trait member, reuse:

```rust
CallableId {
    owner: CallableOwnerId::Declaration(trait_declaration),
    selector,
    side,
}
```

This identity supports:

```text
callable-local generic ownership
CallableSemanticSignature
source definition identity
default body analysis
fingerprints
presentation/navigation
```

A separate mandatory `TraitMemberId` is not required when `CallableId` already carries the same source-callable identity.

A separate mandatory `TraitDefaultId` is also not required. A bodyful trait member's default implementation may use its trait-owned `CallableId`; its distinction from a future concrete witness follows from the different callable owner.

Only introduce extra IDs if they carry semantic information the existing repository identities cannot represent.

### 6.4 Generic trait reference

C3 still requires a durable contract reference conceptually equivalent to:

```rust
TraitRef {
    trait: DeclarationId,     // verified DeclarationKind::Trait
    arguments: Box<[TypeId]>,
}
```

`TraitRef` is not automatically an ordinary inhabitable `TypeId`, runtime trait object, or conformance proof.

A category-safe trait-declaration wrapper may be used internally, but `TraitRef` must not force a parallel declaration identity universe.

### 6.5 TraitSurface is separate from DeclarationSurface

The semantic contract product remains distinct:

```text
TraitSurface
    trait declaration identity
    generic signature
    requirements
    callable signatures
    visibility
    default availability/source
    diagnostics
```

It must not be merged into:

```text
DeclaredSurface
Effective DeclarationSurface
C2 conditional inherent surface
EnumBehaviorProduct
runtime class method tables
```

### 6.6 Body classification

```text
bodyless trait behavior
    = TraitRequirementId + callable signature

bodyful trait behavior
    = same TraitRequirementId
      + trait-owned default CallableId/body
```

The existence of a default does not erase the requirement.

### 6.7 Abstract Self

Trait signatures/defaults reuse the existing owner-relative `SelfTypeTerm`.

```text
owner = trait DeclarationId
receiver = abstract Self
member lookup = completed TraitSurface
storage = none
superclass = none
conformance evidence = none
```

No new trait-specific Self type system is allowed.

### 6.8 Default bodies are checked once

A default implementation is analyzed once under abstract trait `Self`.

It is not copied and re-typechecked for every future conformance.

A call from a default to another trait member is contract-relative:

```text
self.member(...)
    ↓
TraitSurface requirement/signature
```

It must not be hard-bound directly to another default implementation, because C4 may later choose a concrete witness.

### 6.9 Source order does not control surface availability

Required publication sequence:

```text
parse all trait members
    ↓
resolve/publish all requirement signatures
    ↓
complete TraitSurface
    ↓
analyze bodyful defaults
```

A default may refer to a trait member declared later.

### 6.10 Source tooling reuses existing semantic targets

Prefer:

```text
trait declaration source
    -> SemanticTargetId::Declaration(DeclarationId)

trait behavioral member/default source
    -> SemanticTargetId::Callable(CallableId)
```

Do not add `SemanticTargetId::Trait` or `::TraitMember` unless a concrete semantic/tooling distinction cannot be represented by the existing targets.

Requirement identities may still be attached as semantic contract metadata where C4 needs them; source navigation identity does not need to duplicate them.

### 6.11 No owned representation

Traits own no:

```text
instance fields
data components
ProductLayout
ProductStorage
superclass edge
ordinary allocation path
ordinary runtime ClassId merely to express the contract
```

Getter/setter/index requirements express capabilities, not injected storage.

### 6.12 Compiler/runtime boundary

C3 is semantic-first.

A trait declaration must not cause:

```text
ordinary class creation
field-slot allocation
superclass installation
trait member injection into target classes
VM trait scanning on ordinary sends
runtime conformance registry
witness/vtable construction
```

Where the current compiler can treat a declaration as compile-time-only, prefer that model.

If module linkage genuinely requires a runtime trait descriptor, stop and consult rather than synthesize a fake class or prematurely implement C7 reflection.

---

## 7. Index Requirement Normalization — Required P1 Amendment

The audited repository reveals a concrete mismatch that the initial C3.P1 plan did not make explicit.

Current shared AST:

```text
BehaviorMember::Index(IndexMethodDef)
IndexMethodDef.body
    = executable statement vector
```

Unlike methods/getters/setters, current index members do not have a declaration-only `MemberBody` state.

But the LANG005 support surface explicitly requires forms such as:

```phalcom
trait IntIndexable {
  [_ index: Int] -> Int
}

trait IntMutableIndexable {
  [_ index: Int] -> Int
  [_ index: Int]=(_ value: Int) -> ()
}
```

Therefore C3.P1 must explicitly normalize the shared index-member body representation before claiming complete trait index requirements.

Preferred direction:

```text
IndexMethodDef.body
    Vec<Statement>
        ↓
    MemberBody
        Declaration
        Block(...)
```

or the exact repository-equivalent shared representation.

Requirements:

1. Do not invent a trait-only index AST.
2. Audit every `IndexMethodDef.body` consumer across AST, semantic fingerprints, compiler, native/source indexing, product optimizer, LSP, tests, and attributes.
3. Preserve existing bodyful class/impl index behavior exactly.
4. Add declaration-only index parsing only in declaration contexts that permit it.
5. Ensure ordinary class/inherent-impl semantics do not accidentally accept abstract declaration-only index members if they are forbidden there.
6. Add focused regression tests for both index getter and setter requirements.

If repository drift lands this normalization before C3 begins, reuse it rather than repeating it.

---

## 8. LANG005 Support Files — Scope Rules

The repository's `docs/implementation/LANG005/support/` files are useful language-shape fixtures but span multiple future checkpoints.

### Direct C3 reference

```text
comparable.ph
sized.ph
```

These demonstrate:

```text
bodyless requirements
bodyful defaults
generic trait parameters
getter-shaped requirements
default calls through self
```

### Cross-check only; not full C3 scope

```text
iterable.ph
indexable.ph
core_trait_impls.ph
metatype_conformance_example.ph
```

They contain features owned later:

```text
associated types         -> C5
impl Trait for Target    -> C4
associated bindings      -> C5
metatype conformance     -> later explicit design/C4+
conditional conformance  -> C6
```

Implementing agents must use these files to preserve forward compatibility, not as permission to pull all shown syntax into C3.P1.

---

## 9. Protocol-Era Specification Migration Surface

C3 must audit and reconcile the complete active/conflicting protocol authority cluster, including at minimum:

```text
docs/spec/typing/README.md
docs/spec/typing/STATUS.md
docs/spec/typing/01-protocol-foundation.md
docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md
docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md
```

Also search `docs/spec/` for additional active claims about:

```text
@protocol class
Protocol descriptor
signature-only protocol
structural protocol conformance
protocol defaults
protocol class-side requirements
```

Do not mechanically replace every English use of the word “protocol”; many unrelated documents use “protocol” in the ordinary sense of API/procedure/cursor protocol.

The migration goal is one canonical trait rule, not global vocabulary erasure.

Historical content must be archived or explicitly marked superseded rather than deleted.

---

## 10. Required Predecessor Takeover from C2

### Already landed at the audited baseline

C2.P1/P2 provide repository-equivalent forms of:

```text
BehaviorMember
ImplId
TypeParameterOwner::Impl
InherentImplContribution
effective DeclarationSurface
CallableId / callable-definition provenance
variants-only EnumDef
exact-case inherent impl targets
EnumBehaviorProduct
EnumRequirementId
semantic compiler lowering for root/exact-case impl behavior
source/index/incremental impl handling
```

### Still required from C2.P3

Before C3 begins, verify P3 actually lands:

```text
receiver-specialized/constrained impl-domain product
conditional inherent member publication
receiver applicability/selection evidence
conditional dispatch/lowering boundary
incremental fingerprints for conditional applicability
receiver-effective tooling surface where planned
```

Do not hard-code planned P3 type names into C3 before implementation; consume its walkthrough/handoff.

---

## 11. Plan Amendment Ledger

These amendments are adopted by this repository audit and must be applied to C3.P1 execution even if an earlier downloadable plan contains stronger placeholder shapes.

### C3-A01 — Lifecycle normalization

**Trigger:** new C3 checkpoint used the older C2 body-only checkpoint style.

**Decision:** use YAML front matter and the ledgers required by `docs/workflow/implementation-record-lifecycle-convention.md`.

**Consequence:** this checkpoint record is the canonical durable state; plan records remain intent.

### C3-A02 — Minimize new identity types

**Trigger:** live repository already has universal `DeclarationId`, `CallableId`, and `SemanticTargetId`.

**Decision:** require only identity that represents genuinely new semantics.

Canonical minimum:

```text
trait declaration
    DeclarationId + DeclarationKind::Trait

trait source callable/default
    CallableId

trait requirement
    TraitRequirementId

trait reference
    TraitRef(trait declaration + generic arguments)
```

`TraitId`, `TraitMemberId`, and `TraitDefaultId` are no longer mandatory architectural requirements.

### C3-A03 — Reuse source target identities

**Decision:** use existing `SemanticTargetId::Declaration` and `::Callable` unless a proven source-tooling distinction requires an extension.

### C3-A04 — Normalize index body representation

**Trigger:** `IndexMethodDef` cannot currently represent bodyless requirements, while LANG005's index support surface requires them.

**Decision:** C3.P1 must explicitly normalize index bodies to `MemberBody` or repository-equivalent shared declaration/body representation, with a full affected-consumer audit.

### C3-A05 — Expand protocol documentation migration audit

**Decision:** reconcile the complete protocol-era authority cluster, including typing README/STATUS and consolidated protocol design records, not just `01-protocol-foundation.md`.

### C3-A06 — Treat LANG005 support files as multi-checkpoint references

**Decision:** use `comparable.ph`/`sized.ph` directly for C3 shape; use associated-type/conformance/metatype examples only for forward-compatibility review.

### C3-A07 — Repository lifecycle failure classification

Implementation evidence must use the repository workflow's failure classes:

```text
A — definitely caused by current change
B — probably caused by current change
C — unclear
D — clearly pre-existing / unrelated
```

Expected temporary feature gaps during an unfinished task are tracked separately as planned-red conditions, not renamed into the A/B/C/D failure taxonomy.

---

## 12. Established Invariants

These are accepted C3 architecture constraints, not claims that C3 source implementation already exists.

`C3-I-001` — Trait is a distinct semantic declaration category, not class sugar.

`C3-I-002` — Trait declaration identity participates in the existing `DeclarationId` universe.

`C3-I-003` — Trait contract requirements have identity distinct from future concrete witnesses.

`C3-I-004` — Existing `CallableId` is reused for trait-declared callable/default source identity unless live implementation proves a missing semantic dimension.

`C3-I-005` — `TraitRef` is a contract reference and is not automatically an inhabitable runtime value type.

`C3-I-006` — `TraitSurface` is distinct from ordinary and conditional inherent surfaces.

`C3-I-007` — Every behavioral trait member defines a requirement.

`C3-I-008` — A bodyful trait member is requirement + default, not a second unrelated method.

`C3-I-009` — Defaults are checked once under abstract owner-relative `Self`.

`C3-I-010` — Default calls to trait members are contract-relative.

`C3-I-011` — Trait owns no storage, product layout, superclass edge, or ordinary runtime class.

`C3-I-012` — Closed enum requirements remain `EnumRequirementId`-based closed-sum semantics, not hidden traits.

`C3-I-013` — C2 conditional inherent applicability remains separate from trait conformance.

`C3-I-014` — C3 creates no conformance/witness relation.

`C3-I-015` — Associated types remain C5; generic conformance constraints remain C6.

`C3-I-016` — Complete TraitSurface publication precedes default body analysis.

`C3-I-017` — Body-only default edits must not change requirement/source-callable identity.

`C3-I-018` — Source tooling reuses canonical semantic declaration/callable targets.

`C3-I-019` — Index requirements use the shared index AST after declaration/body normalization; no trait-only parallel index grammar.

`C3-I-020` — Runtime dispatch remains trait-unaware until later explicit conformance/runtime work.

---

## 13. Stable Interface / Takeover Map

### Current C2 P1/P2 interfaces

| Concept | Audited live owner |
|---|---|
| declaration identity | `phalcom_modules::DeclarationId` |
| declaration kind | `phalcom_modules::DeclarationKind` |
| behavior syntax | `phalcom_ast::BehaviorMember` |
| callable identity | `phalcom_semantic::CallableId` |
| callable owner | `phalcom_semantic::CallableOwnerId` |
| impl provenance | `phalcom_semantic::ImplId` |
| inherent impl model | `phalcom_semantic::impls` |
| enum requirement identity | `phalcom_semantic::EnumRequirementId` |
| enum behavior contract | `phalcom_semantic::checker::enum_behavior::EnumBehaviorProduct` |
| source semantic target | `phalcom_semantic::SemanticTargetId` |
| declaration shard/fingerprints | `phalcom_semantic::semantic_shard` |
| callable signature | `phalcom_semantic::CallableSemanticSignature` |

### C3 interfaces to establish

Names are intent, not mandatory Rust spelling:

```text
DeclarationKind::Trait
TraitRequirementId
TraitRef
TraitSurface
trait default body analysis through existing CallableId
trait-aware abstract Self lookup adapter
trait surface/default query keys and fingerprints
```

### C2.P3 interfaces to import later

Populate this subsection from the P3 walkthrough/handoff after P3 is implemented. Do not guess final symbol names now.

---

## 14. Verification Ledger

C3 implementation verification remains `UNVERIFIED`. The entries below are **planning/audit evidence**, not feature certification.

| Gate | Scope | Evidence | Result | What it establishes |
|---|---|---|---|---|
| AUDIT-0 | repository head | latest commit inspection | `986568d...` | current audit baseline |
| AUDIT-1 | C2 status | `LANG005.C2-CHECKPOINT.md` | P1/P2 complete; P3 planned | C3 remains blocked |
| AUDIT-2 | shared behavior AST | search `pub enum BehaviorMember` | present | C3 should reuse it |
| AUDIT-3 | declaration kinds | `phalcom-modules/src/declaration.rs` | `Protocol` present, no production use found | clean Trait rename seam likely exists |
| AUDIT-4 | callable identity | `phalcom-semantic/src/identity.rs` | Declaration-owned `CallableId` present | trait callable/default can reuse canonical identity |
| AUDIT-5 | source targets | `SemanticTargetId` | Declaration + Callable already present | no trait-specific source target required by default |
| AUDIT-6 | index member body | AST/search | current `IndexMethodDef` lacks declaration-only body state | C3 index normalization required |
| AUDIT-7 | support corpus | `support/indexable.ph` | bodyless index getter/setter shown | confirms intended trait index contract surface |
| AUDIT-8 | workflow | lifecycle convention | YAML + ledgers required for new checkpoints | checkpoint document required correction |

No C3 tests have run because no C3 implementation exists.

---

## 15. Deferred / Baseline Issue Ledger

No C3 implementation baseline failures are recorded yet.

### PRE-001 — C2.P3 predecessor incomplete

**Observed:** C2 checkpoint is still in progress and P3 is planned/not started.

**Classification:** predecessor blocker, not a C3 test failure.

**Why it blocks:** C3 must consume final receiver-conditional inherent surface architecture.

**Revisit:** immediately after C2.P3 walkthrough/handoff is available.

### PRE-002 — Protocol-era specification conflict

**Observed:** active typing documents still describe a different `@protocol class`/structural protocol model.

**Classification:** planned C3 specification migration work.

**Why it does not authorize early edits:** C3 should reconcile this coherently with implementation, not piecemeal before P1 starts.

**Revisit:** C3.P1 specification task.

### PRE-003 — Index declaration-only representation gap

**Observed:** shared `IndexMethodDef` cannot currently express declaration-only index requirements.

**Classification:** planned C3.P1 implementation prerequisite/amendment.

**Revisit:** C3.P1 syntax/AST task after G0.

---

## 16. Consultation / Amendment Ledger

The audit corrections in §11 are the initial adopted amendments.

Any future architectural escalation must add:

```text
Incident
Plan/task
Trigger
Decision
Architectural consequence
Plan amendment
Additional tests
```

Do not paste model conversations into this record.

---

## 17. Required C3.P1 Gate Sequence

After C2.P3 closes:

```text
G0
    verify C2 takeover
    update baseline revision
    import P3 actual symbol map
    verify plan amendments

G1
    canonical trait syntax
    declaration kind / module identity
    shared index MemberBody normalization

G2
    TraitRef
    TraitRequirementId
    TraitSurface
    default/source callable metadata

G3
    abstract Self lookup
    default body analysis

G4
    query/fingerprint separation
    source indexing
    LSP presentation

G5
    compiler/runtime non-class boundary

G6
    focused stabilization
    normative protocol→trait migration closure
    walkthrough
    C4 handoff
```

---

## 18. Verification Expectations

Follow repository testing doctrine:

```text
T0 exact new regression
T1 directly affected focused target/filter
T2 owning focused suite
T3 adjacent suite only when changed code justifies it
T4 broad crate only when evidence warrants it
T5 workspace/release only when explicitly required
```

Every filter must select nonzero tests.

Expected final C3 focused areas:

```text
phalcom-ast trait + declaration-only index syntax
phalcom-modules trait declaration/interface
phalcom-semantic trait declarations/surfaces/defaults
phalcom-semantic incremental trait edits
phalcom-core trait declaration no-runtime-class behavior
focused LSP test only if LSP-specific code changes
cargo fmt --all -- --check
```

Do not repeatedly run workspace-wide test/build/clippy during BUILD mode.

---

## 19. Completion Criteria

C3 is complete only when all of these are true:

```text
dedicated trait syntax exists
DeclarationKind::Trait is canonical
trait declaration identity uses canonical DeclarationId
TraitRequirementId exists
TraitRef exists
TraitSurface exists
trait member/default source callables reuse canonical CallableId or an explicitly justified equivalent
bodyless index requirements are representable through shared AST
defaults are checked once against abstract Self
default calls are contract-relative
trait semantic products are incrementally stable
source indexing uses canonical semantic identities
compiler does not create ordinary runtime class semantics
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

Successful lifecycle:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

`RELEASE_COMPLETE` requires separately named broad evidence and is not implied by C3 completion.

---

## 20. Active Plan and Next Action

### Active plan

None. C3 is blocked before plan start.

### Next planned plan

```text
LANG005.C3.P1
    First-Class Trait Declarations and Abstract Trait Surfaces
```

### Current next action

```text
Implement and close LANG005.C2.P3.
```

After P3 completion:

1. read the P3 walkthrough/handoff;
2. update this checkpoint's baseline revision and takeover map;
3. read `LANG005.C3-GUIDANCE.md`;
4. reconcile the P1 plan with amendments C3-A01…C3-A07;
5. mark P1 `IN_PROGRESS`;
6. begin G0;
7. do not edit trait source until G0 passes.
