---
id: LANG005.C3.guidance
category: LANG
program: LANG005
checkpoint: LANG005.C3
kind: implementer-guidance
status: ACTIVE
baseline_revision: 986568da050d1bbfa7f1d769c9e1f4d58eb81cf3
---

# Implementer Guidance — LANG005.C3 Trait Declarations and Abstract Trait Surfaces

## 0. Purpose

This document is the operating guide for agents implementing work under:

```text
LANG005.C3
    Trait declarations and abstract trait surfaces
```

It is written for lower-end Luna implementers as well as stronger agents.

It does not replace:

```text
AGENTS.md
docs/spec/README.md
docs/implementation/README.md
docs/workflow/implementation-record-lifecycle-convention.md
LANG005.C3-CHECKPOINT.md
LANG005.C3.P1-requirements-analysis.md
LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md
```

Its job is to make the execution policy explicit: what to read, what architecture is fixed, which live repository seams to reuse, what not to pull forward from later checkpoints, when to stop, and how to test without wasting implementation time.

---

# 1. Current Checkpoint State

At the audited baseline:

```text
repository main:
    986568da050d1bbfa7f1d769c9e1f4d58eb81cf3

C2.P1:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

C2.P2:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

C2.P3:
    PLANNED / NOT_STARTED / UNVERIFIED

C3:
    BLOCKED / NOT_STARTED / UNVERIFIED
```

Therefore:

> **Do not implement C3 until C2.P3 is complete and C3 G0 verifies the final C2 takeover architecture.**

The first C3 action is not trait parsing. The first C3 action is predecessor verification and checkpoint update.

---

# 2. Canonical Repository Location

When C3 records are added to the repository, use the existing LANG005 family style:

```text
docs/implementation/LANG005/
  LANG005.C3-trait-declarations-and-abstract-surfaces/
    LANG005.C3-CHECKPOINT.md
    LANG005.C3-GUIDANCE.md
    LANG005.C3.P1-requirements-analysis.md
    LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md
```

Do not reorganize C1/C2 as part of C3.

---

# 3. Required Read Order

At the start of a C3 implementation session, read:

```text
1. AGENTS.md
2. docs/spec/README.md
3. docs/implementation/README.md
4. docs/workflow/implementation-record-lifecycle-convention.md

5. LANG005.C3-CHECKPOINT.md
6. LANG005.C3-GUIDANCE.md

7. LANG005.C3.P1-requirements-analysis.md
8. LANG005.C3.P1-first-class-traits-and-abstract-surfaces-plan.md

9. LANG005.C2-CHECKPOINT.md
10. LANG005.C2.P3-walkthrough.md
11. LANG005.C2.P3-handoff.md

12. exact live source files named by the current task
```

Also inspect the LANG005 support files listed in §16, but respect their checkpoint ownership.

Do not start with a broad repository scan when the plan/checkpoint already names the owners.

---

# 4. Authority and Drift Rules

Use these rules rather than a simplistic “one document always wins” ordering.

## 4.1 Language semantics

Canonical effective `docs/spec/` rules and ratified Phalcom decisions own language meaning.

If a protocol-era spec conflicts with the ratified LANG005 trait direction, do not silently follow the old protocol rule; C3 includes the required specification reconciliation.

## 4.2 Checkpoint state

`LANG005.C3-CHECKPOINT.md` is the durable source of truth for:

```text
current lifecycle
accepted amendments
landed interfaces
verification evidence
blockers
next action
```

A numbered plan is intent, not implementation truth.

## 4.3 Plan execution

The C3.P1 plan owns task ordering and detailed intended work **except where the checkpoint amendment ledger records a repository-audit correction**.

The currently adopted corrections are `C3-A01` through `C3-A07`.

If the downloadable plan still contains older placeholder identity shapes, apply the checkpoint amendments instead of reproducing stale scaffolding.

## 4.4 Live repository

Live source owns mechanical reality.

Examples:

```text
helper renamed
module split
query enum moved
collection type changed
```

Adapt mechanically and record the mapping.

If live source changes semantic ownership or invalidates an invariant, stop and consult.

## 4.5 Walkthrough/handoff

Once a predecessor plan is complete, its walkthrough/handoff is the operational truth about what actually landed.

Do not reconstruct C2.P3 from its plan after the walkthrough exists.

---

# 5. Session Bootstrap

Before editing:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
```

Record the starting revision in checkpoint state when P1 begins.

Preserve unrelated modified, staged, and untracked files.

Do not:

```text
git reset --hard
git clean
broad-format unrelated files
stage unrelated work
rewrite adjacent subsystems for cleanliness
```

Read directory-specific `AGENTS.md` files if present.

---

# 6. Verify C2 Before Any C3 Source Edit

C3 requires final C2 architecture.

Verify repository-equivalent owners for:

```text
BehaviorMember
ImplId
TypeParameterOwner::Impl
InherentImplContribution
effective DeclarationSurface
EnumBehaviorProduct
EnumRequirementId
exact-case impl target
conditional inherent impl-domain/applicability product
conditional member publication
receiver applicability/selection evidence
conditional dispatch/lowering boundary
incremental conditional-surface fingerprints
source/tooling receiver-effective view where P3 planned it
```

Already audited P1/P2 seams include:

```text
phalcom_ast::BehaviorMember
phalcom_semantic::CallableId
phalcom_semantic::ImplId
phalcom_semantic::EnumRequirementId
phalcom_semantic::checker::enum_behavior::EnumBehaviorProduct
```

Do not guess P3 symbol names before P3 lands.

If P3 is incomplete:

```text
STOP
```

If P3 lands equivalent semantics under different names:

```text
record mapping
continue
```

If P3 changes the behavioral-surface ownership model materially:

```text
PLAN DRIFT
STOP AND CONSULT
```

---

# 7. The Semantic Model to Keep Loaded

Throughout C3:

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
    closed root/case contract remains enum-specific

impl
    inherent-behavior provenance / applicability
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

TraitRequirementId
    != future concrete witness CallableId

EnumRequirementId
    != TraitRequirementId

TraitRef
    != ordinary inhabitable runtime value type

trait
    != class
```

---

# 8. Identity Policy — Reuse Before Inventing

This is a repository-audit correction to the initial P1 identity sketch.

## 8.1 Trait declaration identity

The repository already has:

```rust
DeclarationId {
    module,
    name,
}
```

Use it as the canonical declaration identity.

Category comes from:

```text
DeclarationKind::Trait
```

The audited tree currently has an unused `DeclarationKind::Protocol`. The expected C3 action is to rename/replace that variant with `Trait`, unless G0 finds an external compatibility constraint not visible at this baseline.

Do not create a mandatory parallel `TraitId` identity universe simply because traits are new.

A lightweight category-safe wrapper is allowed only if it prevents invalid API states and does not become a second identity.

## 8.2 Trait requirement identity

This is genuinely new semantic identity.

Use repository-equivalent semantics such as:

```rust
TraitRequirementId {
    owner: DeclarationId,
    selector: Selector,
    side: DispatchSide,
}
```

This is analogous to `EnumRequirementId` but belongs to a reusable trait contract rather than a closed enum.

It must remain distinct from future witness callable identity.

## 8.3 Trait member/default source identity

Reuse existing:

```rust
CallableId {
    owner: CallableOwnerId::Declaration(trait_decl),
    selector,
    side,
}
```

This already supports:

```text
callable-local generics
CallableSemanticSignature
body analysis
source identity
fingerprinting
presentation/navigation
```

Do not add mandatory `TraitMemberId` or `TraitDefaultId` if they merely duplicate this callable identity.

For a bodyful trait member:

```text
TraitRequirementId
    = contract identity

trait-owned CallableId
    = default source implementation / callable analysis identity

future target-owned CallableId
    = possible concrete witness
```

That separation is sufficient unless implementation reveals a genuinely new semantic dimension.

## 8.4 Source semantic targets

Reuse:

```text
SemanticTargetId::Declaration(DeclarationId)
SemanticTargetId::Callable(CallableId)
```

Do not add trait-specific source target variants merely for presentation.

LSP may render a declaration as an Interface-like symbol, but semantic source identity stays canonical.

---

# 9. TraitRef Policy

A trait reference is a contract reference, conceptually:

```rust
TraitRef {
    trait_declaration: DeclarationId,  // verified DeclarationKind::Trait
    arguments: Box<[TypeId]>,
}
```

It is not:

```text
ordinary nominal TypeId
existential trait object
runtime descriptor
conformance proof
bool conforms_to result
```

Do not add `TypeData::Trait` merely to reuse ordinary generic type application.

Use a dedicated trait-reference resolution/formation path over the existing type-like syntax.

If a category-safe trait declaration wrapper is used internally, it must still map one-to-one to the canonical `DeclarationId`.

---

# 10. TraitSurface Policy

The central C3 product is a separate contract surface.

Conceptually:

```text
TraitSurface
    owner: trait DeclarationId
    generic signature
    requirements
    callable signatures
    visibility
    default availability/source callable
    diagnostics
```

Do not insert trait members into:

```text
DeclaredSurface
Effective DeclarationSurface
conditional inherent member index
runtime class method table
```

Those are different semantic products.

---

# 11. Body Classification Rule

Fixed rule:

```text
bodyless trait behavior
    = requirement

bodyful trait behavior
    = same requirement + default implementation
```

The default is not an unrelated helper method.

Requirement identity survives a bodyless↔bodyful transition when the selector/side is unchanged.

The bodyful default's source/body analysis can use the trait-owned `CallableId`.

---

# 12. Shared BehaviorMember — Reuse It

Current repository fact:

```rust
pub enum BehaviorMember {
    Method(...),
    Getter(...),
    Setter(...),
    Index(...),
}
```

Use this shared syntax category.

Do not create a parallel trait-only method/getter/setter grammar.

However, do **not** assume every `BehaviorMember` currently supports declaration-only bodies. Index members are the important exception; see §13.

---

# 13. Required Index-Member Normalization

This is a mandatory repository-driven C3 correction.

Current audited shape:

```text
BehaviorMember::Index(IndexMethodDef)

IndexMethodDef.body
    = executable Vec<Statement>-style body
```

Current index members therefore cannot represent a bodyless trait requirement.

But LANG005's support surface contains:

```phalcom
trait Indexable {
  type Index
  type Item

  [_ index: Index] -> Item
}

trait MutableIndexable {
  type Index
  type Item

  [_ index: Index] -> Item
  [_ index: Index]=(_ value: Item) -> Unit
}
```

Associated types are C5, but the **bodyless index requirement shape itself is C3 infrastructure**.

C3.P1 must therefore normalize the shared index body representation before claiming index requirements.

Preferred shape:

```text
IndexMethodDef.body
    MemberBody::Declaration
    MemberBody::Block(...)
```

or repository-equivalent.

Implementation rules:

1. Do not create `TraitIndexRequirementDef`.
2. Change the shared representation once.
3. Audit every production consumer of `IndexMethodDef.body`.
4. Preserve existing class/inherent bodyful index behavior.
5. Only declaration contexts that semantically permit abstract members may accept `MemberBody::Declaration`.
6. Add getter and setter requirement regressions.
7. Update fingerprint logic so declaration-only/bodyful index bodies obey the same surface/body separation rules.
8. Update product optimizer/source index/LSP/compiler exhaustive matches mechanically.
9. If P3 or another predecessor already normalizes index bodies, reuse it.

This AST normalization must be reflected in the C3.P1 plan before implementation begins.

---

# 14. Abstract Self Is the Core of Defaults

Reuse the existing owner-relative `SelfTypeTerm`.

Do not invent:

```text
TraitSelfType
ProtocolSelfType
FakeConformer
synthetic trait class
```

A default body is checked under:

```text
current declaration
    = trait DeclarationId

current callable
    = trait-owned CallableId

Self
    = existing owner-relative SelfTypeTerm

available receiver behavior
    = completed TraitSurface

fields/storage
    = none

superclass
    = none

conformance evidence
    = none
```

---

# 15. Default Calls Remain Contract-Relative

Example repository support fixture:

```phalcom
trait Comparable<Rhs> {
  compare(_ other: Rhs) -> Ordering

  <(_ other: Rhs) -> Bool {
    self.compare(other) === Ordering::Less
  }
}
```

The call:

```phalcom
self.compare(other)
```

resolves through the trait contract signature.

It must not be hard-bound to a trait default implementation.

Why:

```text
C4 may select a concrete witness
```

A default body is reusable contract-relative behavior.

---

# 16. LANG005 Support Files — Use Them Correctly

Read:

```text
docs/implementation/LANG005/support/
```

but classify each file by checkpoint ownership.

## 16.1 Direct C3 shape references

### `comparable.ph`

Useful for:

```text
generic trait parameter
bodyless requirement
bodyful defaults
operator-shaped default members
default calls through self
```

### `sized.ph`

Useful for:

```text
getter-shaped requirement
```

## 16.2 Forward-compatibility references, not P1 scope

### `iterable.ph`

Contains:

```text
associated types -> C5
bodyless methods -> C3
defaults -> C3
```

Use it to ensure C3's surface can later accept associated-type extensions. Do not implement its `type Item`/`type Cursor` in C3.

### `indexable.ph`

Contains:

```text
associated types -> C5
bodyless index requirements -> C3 AST/surface infrastructure
```

Use it to justify shared index-body normalization, but use concrete types in C3 tests so C5 is not pulled forward.

### `core_trait_impls.ph`

Contains:

```text
impl Trait for Target -> C4
associated bindings -> C5
future witness reuse
```

Do not implement these forms during C3.

### `metatype_conformance_example.ph`

Contains placeholder metatype target spelling:

```phalcom
impl Parser for class Person
```

The file itself states that spelling is provisional.

Do not ratify it in C3.

---

# 17. C3 Does Not Implement Conformance

C3 ends here:

```text
trait declaration
    ↓
TraitSurface
    ├─ TraitRequirementId
    └─ optional trait-owned default CallableId
```

C4 begins here:

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
construct conformance evidence
perform overlap/coherence checks
install selected defaults
```

---

# 18. C3 Does Not Implement Associated Types

Associated types are C5.

Do not implement:

```phalcom
type Item
type Cursor
type Output
```

inside traits or conformances during C3.

Do not fake associated types as generic trait parameters.

Do not add projection normalization.

If parser work encounters future associated-type syntax, preserve/defer/reject it deliberately rather than half-implementing C5.

---

# 19. C3 Does Not Implement Generic Trait Bounds

Trait-conformance constraints are C6.

Do not add:

```text
GenericConstraint::Conforms
T: Printable
T conforms Printable
```

as new conformance semantics during C3.

Existing ordinary generic constraints remain available.

Example:

```phalcom
trait NumericView<T>
where T <: Number
{
  value -> T
}
```

is acceptable only because `<:` is already ordinary generic declaration machinery.

---

# 20. Metatype/Class-Object Trait Semantics Are Deferred

Do not settle final class-object conformance syntax in C3.

The support file's:

```phalcom
impl Parser for class Person
```

is explicitly placeholder syntax.

Do not infer that trait declaration members need `@class` semantics from that example.

C3.P1 remains instance-contract-first.

If an existing class-side marker appears inside a trait, diagnose/defer it unless a newer canonical decision has ratified its meaning.

---

# 21. No Trait Storage

Trait declarations own no instance state.

Reject/defer constructs that imply:

```text
fields
stored properties
data components
constructor-managed state
class invariants
product layout
field slots
superclass storage
```

A property requirement is behavior/capability.

Example:

```phalcom
trait Sized {
  size -> Int
}
```

does not inject a `size` field.

---

# 22. Build the Complete Surface Before Checking Defaults

Source order must not affect contract availability.

Wrong:

```text
read member
check body immediately
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
analyze default bodies
```

This allows a default to call a later-declared requirement/default.

---

# 23. Protocol-Era Specification Reconciliation

Do not treat the protocol migration as a one-file edit.

Audit at minimum:

```text
docs/spec/typing/README.md
docs/spec/typing/STATUS.md
docs/spec/typing/01-protocol-foundation.md
docs/spec/typing/Class-Declaration Attributes — Abstract, Protocol, Mixin.md
docs/spec/typing/phalcom-type-protocol-record-callable-unit-spec.md
```

Then search all active `docs/spec/` for claims about:

```text
@protocol class
Protocol descriptor
structural protocol conformance
signature-only protocol
default implementations
class-side protocol requirements
```

Important distinction:

The English word “protocol” appears all over the repository for things like:

```text
cursor protocol
network/API protocol
measurement protocol
call protocol
```

Do **not** globally replace those.

Only migrate language-feature claims that compete with the new `trait` semantic category.

Historical material should be archived or marked superseded according to `docs/spec/README.md`.

---

# 24. DeclarationKind::Protocol Migration

Audited fact:

```rust
DeclarationKind {
    Class,
    Protocol,
    Adt,
    Data,
    Alias,
}
```

No production use of `DeclarationKind::Protocol` was found during this audit.

Default C3 action:

```text
Protocol
    ↓
Trait
```

Do not keep both as separate semantic declaration categories without a ratified language reason.

Before renaming, verify:

```text
no serialization schema
no generated metadata ABI
no plugin/API consumer
no persisted cache contract
```

If such a compatibility dependency exists, stop and record it.

Do not silently preserve two semantic categories because the old enum variant already exists.

---

# 25. Source Index and LSP

Prefer existing semantic targets:

```text
trait definition
    SemanticTargetId::Declaration(trait DeclarationId)

trait member/default definition
    SemanticTargetId::Callable(trait-owned CallableId)
```

A `TraitRequirementId` is contract identity for semantic conformance work; it does not need to become the source-navigation target if `CallableId` already points to the source member.

LSP presentation may use an Interface-like symbol/token if there is no dedicated Trait token.

Do not create LSP-local trait identity.

Do not teach the LSP to infer contract semantics from raw AST.

---

# 26. Incremental Architecture

Prefer separable products:

```text
trait declaration/header product
TraitSurface
default CallableAnalysis
```

## Body-only default edit

Should change:

```text
callable/default body analysis
callable body fingerprint
```

Should not change:

```text
DeclarationId
TraitRequirementId
trait-owned CallableId
unrelated signatures
TraitRef identity
```

## Signature edit

Should change:

```text
CallableSemanticSignature
TraitSurface fingerprint
dependent semantic lookups
```

## Selector edit

Should change:

```text
CallableId
TraitRequirementId
```

## Bodyless ↔ bodyful

With same selector/side:

```text
TraitRequirementId
    unchanged

trait-owned CallableId
    unchanged

default availability/body product
    changes
```

## Index member normalization

Fingerprint code currently has special index handling. After `IndexMethodDef.body` normalization, audit:

```text
semantic shard structural fingerprint
callable signature fingerprint
callable body fingerprint
product optimizer AST traversal
source indexing
compiler body extraction
```

Cold and incremental results must agree.

---

# 27. Compiler / Runtime Boundary

C3 is a semantic declaration checkpoint.

Preferred behavior:

```text
Statement::Trait
    compile-time/type-level declaration
    no ordinary runtime class allocation
```

Do not create:

```text
ClassId for every trait
superclass edge
field layout
FinalizeClass
trait methods installed on conformers
runtime trait scan during Invoke
runtime conformance registry
witness/vtable dispatch
```

Type aliases provide a useful compile-time-only precedent in the repository, but do not assume the exact compiler branch is reusable without verifying module/export requirements.

If module execution requires a runtime object for trait exports:

```text
STOP AND CONSULT
```

Do not create a fake class.

Do not prematurely implement the C7 reflection descriptor surface.

---

# 28. Reflection Boundary

C7 owns full reflection/reification.

C3 must preserve semantic identities now, but runtime reflection objects cannot become semantic authority.

Do not add manual public descriptor constructors.

Do not let runtime reflection mutate trait membership/conformance.

---

# 29. Task Execution Policy

Implement one coherent P1 task at a time.

Before a task:

```text
read purpose
verify preconditions
inspect named owners
identify expected semantic product
identify forbidden shortcuts
```

During a task:

```text
edit owning layer
reuse canonical helpers
avoid unrelated cleanup
do not implement next-checkpoint features
run smallest discriminating test after a coherent unit
```

After a gate:

```text
update CHECKPOINT.md
record landed symbol map
record verification evidence
record plan drift/amendments
record next action
```

Do not update checkpoint state after every tiny edit.

---

# 30. Fixed Decisions

Do not change these without architectural consultation:

```text
trait is a distinct declaration category
trait is not class sugar
trait owns no storage/layout/superclass
TraitSurface is separate from inherent surfaces
TraitRequirementId is distinct from future witness CallableId
reuse existing declaration/callable identity unless new semantics require otherwise
bodyful member = requirement + default
defaults checked once
abstract Self uses canonical SelfTypeTerm
default calls are contract-relative
no conformance in C3
TraitRef is not automatically an inhabitable runtime value type
no runtime class/method injection
no associated types in C3
no conformance constraints in C3
no final metatype conformance spelling in C3
index requirements use shared AST, not trait-only syntax
```

---

# 31. Mechanically Flexible Decisions

Adapt without consultation when semantics remain unchanged:

```text
private helper names
private module/file split
map/set implementation choice
newtype wrapper convenience
exact query key spelling
exact diagnostic code spelling following repository convention
test file organization
LSP presentation symbol kind
```

Do not turn “mechanically flexible” into permission to create duplicate semantic owners.

---

# 32. Verify-First Decisions

Check live source before acting:

```text
C2.P3 product names
C2.P3 receiver-effective surface API
DeclarationKind::Protocol compatibility
current BehaviorMember helper APIs
current CallableSyntaxRef API
current MemberBody parser helpers
all IndexMethodDef.body consumers
query/fingerprint key names
source-index attachment API
compiler type-only declaration path
module runtime binding requirements
spec archive destination convention
```

Mechanical drift: adapt.

Architectural drift: stop.

---

# 33. STOP / CONSULT Triggers

Stop immediately if:

1. C2.P3 is incomplete.
2. Final C2 conditional-surface ownership materially differs from the plan.
3. `BehaviorMember` cannot be reused.
4. Index declaration-only support would require a trait-only AST instead of a safe shared normalization.
5. Trait declarations can only be represented as classes.
6. TraitRef can only be implemented by making traits ordinary nominal value types.
7. Default checking needs a concrete conformance/witness.
8. Associated types become necessary for the P1 core.
9. Generic conformance constraints become necessary.
10. Metatype/class-object conformance syntax must be chosen.
11. TraitSurface must be copied into DeclarationSurface.
12. Ordinary VM dispatch must scan traits.
13. Module linkage forces a fake trait class/runtime descriptor.
14. `DeclarationKind::Protocol` is part of a stable external schema/ABI.
15. A new source target enum variant seems necessary only for naming/presentation.
16. Incremental support would require redesigning the semantic DB rather than adding normal products/dependencies.
17. A newer canonical spec contradicts the adopted C3 architecture.
18. The same semantic failure persists after one serious hypothesis-driven correction.
19. A test passes only after weakening a fixed invariant.

Do not solve around these triggers.

---

# 34. Bounded Debugging Protocol

## Mechanical failure

```text
observe exact compiler/test failure
form one concrete mechanical hypothesis
repair
rerun smallest relevant test/check
```

Up to about three coherent cycles while evidence improves.

## Semantic failure

```text
identify violated invariant
inspect semantic owner
form one specific hypothesis
predict expected result
make one serious correction
run exact regression
```

If the same semantic failure remains:

```text
STOP AND CONSULT
```

## Architectural contradiction

No speculative patching.

```text
STOP immediately
```

Never rerun an unchanged failing test merely to see whether it changes.

---

# 35. Failure Classification — Use Repository Convention

Do not use an invented failure taxonomy in checkpoint records.

Use the repository lifecycle convention:

```text
A — definitely caused by current change
B — probably caused by current change
C — unclear
D — clearly pre-existing / unrelated
```

Policy:

```text
A/B
    active implementation responsibility

C
    one bounded classification pass
    if still unclear but nonblocking, record/defer

D
    record and defer immediately
```

An **expected feature gap during an unfinished task** is not a baseline failure class. Track it separately as a planned-red condition and rerun only when the task expected to close it has landed.

---

# 36. Testing Doctrine

Design broad coverage; execute narrowly.

Use:

```text
T0 exact new regression
T1 directly affected focused filter
T2 owning focused suite
T3 adjacent suite only if changed code creates plausible risk
T4 broad affected crate only when evidence warrants it
T5 workspace/release only when explicitly required
```

Always confirm a filter selected at least one test.

Zero tests is not evidence.

---

# 37. Required Coverage Dimensions

C3 should eventually prove:

```text
trait keyword and ranges
module namespace/export identity
DeclarationKind::Trait
generic trait declaration
TraitRef arity/kinds
trait-vs-value-type category distinction
bodyless method requirement
getter requirement
setter requirement
bodyless index getter requirement
bodyless index setter requirement
bodyful default
duplicate selector/side
visibility retention
Self in signatures
default calling requirement
default calling default
later-declared member availability
illegal storage/constructor
class-side deferred boundary
associated-type deferred boundary
source-order independence
body-only invalidation
signature invalidation
cold/incremental parity
source navigation
LSP presentation
compiler no-runtime-class behavior
no runtime member injection
enum contract remains distinct
conditional inherent behavior remains distinct
protocol-era normative migration
```

Use concrete index types in C3 tests rather than C5 associated types.

Example:

```phalcom
trait IntIndexable {
  [_ index: Int] -> Int
}
```

---

# 38. Tests Not to Run Repeatedly

During BUILD mode, do not repeatedly run:

```sh
cargo test --workspace --all-targets
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Also avoid broad unrelated suites unless a shared edit gives a concrete reason:

```text
REPL
concurrency
optimizer
entire language corpus
entire ADT/GADT suite
```

Index AST normalization may justify one adjacent parser/core compile check because it touches a shared member type; it does not justify full workspace repetition after every fix.

---

# 39. Expected Final Focused Acceptance Shape

Verify actual test names with `-- --list`.

Expected areas:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test trait_syntax

RUSTFLAGS='' cargo test -p phalcom-modules trait

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic traits

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental

RUSTFLAGS='' cargo test -p phalcom-core --test core language::traits

cargo fmt --all -- --check
```

Because C3 now explicitly normalizes index bodies, include the smallest existing focused regression target that proves ordinary bodyful class/impl index accessors still compile/execute correctly.

Do not accept any zero-test command.

Run focused LSP tests only if LSP-specific code changed beyond normal semantic-index consumption.

---

# 40. Negative Verification

Before C3 completion, inspect production code for forbidden shortcuts:

```text
trait -> ClassDef
trait -> ClassId
trait -> superclass
trait -> ProductLayout
trait -> field slots
TraitSurface -> DeclarationSurface insertion
trait default -> target add_method
VM Invoke -> trait scan
TraitRequirementId == concrete target witness CallableId
TraitRef -> ordinary nominal TypeId
trait-only duplicate index AST
```

Also inspect remaining language-feature uses of:

```text
DeclarationKind::Protocol
@protocol class
Protocol descriptor
structural protocol conformance
```

Do not flag unrelated English “protocol” terminology.

Every intentional compatibility/history hit must be explained.

---

# 41. Checkpoint Update Protocol

`CHECKPOINT.md` is mandatory execution memory.

At P1 start:

```text
mark P1 IN_PROGRESS
update baseline revision
record C2.P3 takeover symbol map
record any new plan drift
```

At meaningful gates:

```text
add established invariant
add stable interface
record focused evidence
record consultation/amendment
record deferred issue
```

At completion:

```text
update plan ledger
record final verification
record residual failures
write walkthrough
write C4 handoff
set next action
```

Do not create competing implementation-state files.

Do not put hidden reasoning into checkpoint state.

Record facts, decisions, evidence, and consequences.

---

# 42. Checkpoint Amendment Handling

The current checkpoint already contains adopted corrections:

```text
C3-A01 lifecycle normalization
C3-A02 minimal identity additions
C3-A03 source target reuse
C3-A04 index MemberBody normalization
C3-A05 expanded protocol-spec migration audit
C3-A06 support-file scope classification
C3-A07 repository failure taxonomy
```

Before P1 starts, reconcile the patch-grade plan with these amendments.

If you execute from an older plan copy, treat these checkpoint amendments as mandatory corrections.

If a new architectural amendment is needed:

```text
stop
consult if material
record decision in checkpoint
update plan/tasks if necessary
add tests implied by the amendment
```

---

# 43. Walkthrough Requirement

At P1 completion, write a retrospective walkthrough covering:

```text
trait AST
IndexMethodDef declaration/body normalization
module DeclarationKind::Trait
canonical DeclarationId use
TraitRequirementId
TraitRef
TraitSurface
trait-owned CallableId use
generic ownership
abstract Self
default contract-relative lookup
default body query
incremental dependency graph
source target reuse
LSP presentation
compiler/runtime non-class boundary
protocol-spec migration
verification evidence
plan amendments/deviations
```

The walkthrough describes what actually landed.

It must not merely restate the plan.

---

# 44. C4 Handoff Requirement

The C4 handoff must make these APIs/mechanics explicit:

```text
how to recognize a trait DeclarationId
how to form TraitRef
how to query TraitSurface
how to enumerate TraitRequirementId entries
how to find the trait-owned CallableId for a default
how to substitute trait generics into signatures
how abstract Self is represented
how visibility is retained
how source definitions are targeted
how incremental keys/fingerprints are structured
what compiler/runtime does with trait declarations
how C2 conditional inherent members are queried after P3
```

Reiterate:

```text
conformance != inherent behavior
requirement != witness
default declaration != default selection
```

C4 should consume C3 products rather than reopen C3 identity design.

---

# 45. Scope Discipline for C4/C5/C6 Preparation

Good extension points to leave:

```text
TraitRef formation API
TraitSurface query
TraitRequirementId lookup
trait-owned default CallableId lookup
visibility metadata
generic substitution
receiver-effective inherent surface API from C2.P3
```

Do not implement early:

```text
ConformanceId
witness maps
coherence solver
default selection
impl Trait for Target parser
AssociatedTypeId
Projection
GenericConstraint::Conforms
```

unless the checkpoint is formally amended.

---

# 46. Fast Mental Checklist Before Every Patch

Ask:

```text
Who owns this fact?

AST?
module declaration identity?
trait contract?
generic reference?
call checking?
incremental query?
source identity?
compiler lowering?
```

Then ask:

```text
Am I reusing an existing canonical identity where possible?
Am I creating a second semantic owner?
Am I making trait class-like?
Am I confusing requirement with source callable or future witness?
Am I pulling C4/C5/C6/C7 work forward?
Am I making runtime state semantic authority?
```

If the danger answer is yes, stop and reconsider.

---

# 47. Completion Truth

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

Completion requires:

```text
dedicated trait syntax
DeclarationKind::Trait
canonical declaration identity
TraitRequirementId
TraitRef
TraitSurface
shared declaration-only index support
bodyful defaults
abstract Self
contract-relative default calls
incrementality
source/tooling identity
compiler/runtime non-class boundary
protocol→trait specification reconciliation
focused verification
walkthrough
C4 handoff
```

Only then may the checkpoint become:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

---

# 48. Final Agent Rule

> **Implement the trait contract, not trait conformance. Reuse Phalcom's existing declaration, callable, and source-target identity machinery wherever it already expresses the fact. Add a new ID only for genuinely new semantics—most importantly the trait requirement itself. Normalize shared index-member syntax so bodyless index contracts are real rather than trait-only special cases. Keep defaults abstract-Self checked and contract-relative, and keep all C4–C7 behavior out of C3.**
