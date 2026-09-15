---
id: LANG005.C5
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
requires:
  - LANG005.C4 semantic completion
active_plan: null
latest_completed_plan: LANG005.C5.P1
next_plan: LANG005.C5.P2
next_checkpoint: LANG005.C6
planning_baseline_revision: e5152196d716ed70fc47d104a0fd789e602285db
planning_baseline_commit: "docs: record C4 stress and certification evidence"
prepared: 2026-09-15
repository: aureat/phalcom-lang
---

# Checkpoint Record — LANG005.C5 Associated Types, Projection, and Property-Requirement Integration

## 1. Objective and ownership boundary

`LANG005.C5` extends the trait/conformance architecture established by C3–C4 with **associated types as first-class trait-owned requirements whose concrete meanings are conformance-dependent**, then makes those bindings usable through type-level projection and integrates the result through the language/tooling/runtime boundary.

C5 also owns one narrowly scoped enabling language-surface vertical required to make the ratified trait property model executable:

```text
trait property-shaped requirement
    ↓ elaborates to
ordinary getter/setter TraitRequirementId obligations

class/impl direct-field `via` declaration
    ↓ elaborates to
ordinary getter/setter implementations
```

This enabling vertical must not become a second storage/conformance system.

The checkpoint's core semantic sentence is:

> A trait may own associated type declarations and behavioral property requirements, but it never owns or injects representation. A conformance binds associated types and proves behavioral requirements. Associated projections normalize from canonical conformance evidence, while property requirements and `via` delegation converge through ordinary getter/setter behavior.

The full C5 acceptance path is:

```text
trait declaration
    ├─ behavioral requirements/defaults
    ├─ property-shaped requirements
    └─ associated type declarations
            ↓
TraitSurface
    ├─ TraitRequirementId keyed behavioral surface
    └─ AssociatedTypeRequirementId keyed associated surface
            ↓
source conformance
    impl TraitRef for Target
            ↓
ImplId + ConformanceHeadMatch
            ↓
source conformance plan
    ├─ ConformanceWitnessPlan
    └─ associated type binding plan
            ↓
exact target + exact TraitRef specialization
            ↓
ConformanceEvidence
    ├─ behavioral selections
    ├─ exact associated type bindings
    └─ one unified completeness/proof state
            ↓
associated projection formation
            ↓
projection normalization through semantic evidence
            ↓
compiler/lowering/runtime reification only after semantic normalization
            ↓
source index / editor / LSP / Iterable integration / certification
```

C5 owns:

```text
trait associated type declarations
stable trait-owned associated type requirement identity
conformance associated type bindings
source binding plans and exact specialized binding evidence
joint behavioral + associated conformance completeness
trait property-shaped requirement elaboration to getter/setter requirements
direct-field `via` accessor elaboration in class and inherent impl declarations
contextual associated projection, beginning with Self::Item
projection normalization through exact/symbolic conformance evidence
projection ambiguity/cycle/terminal-state preservation
associated-type use in trait/conformance signatures and defaults where proven
incremental/fingerprint/source-index/tooling support for associated declarations/bindings/projections
associated-type-driven Iterable migration/integration
compiler/runtime transport of already-normalized associated type information when required
C5-focused stress and certification
```

C5 explicitly does **not** own:

```text
trait-owned or conformance-introduced storage
StateRequirementId / TraitFieldId / associated-state bindings
field inheritance
arbitrary place delegation
subscript delegation
delegate objects / delegate protocols
computed-property shorthand beyond ratified direct-field `via`
generic T: Trait constraint syntax and proof machinery
conditional conformance depending on trait evidence
nested generic conformance evidence environments required by trait bounds
supertraits / trait inheritance
trait objects / existential representation
public reflection descriptors for traits/conformances/associated types
associated type defaults
GATs
conformance specialization / "most specific wins"
class-header `with` conformance sugar
runtime trait/conformance/projection solving
```

Those boundaries are architectural, not scheduling suggestions.

---

## 2. Current lifecycle state

Planning state:

```yaml
checkpoint: LANG005.C5
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
active_plan: null
next_plan: LANG005.C5.P2
next_checkpoint: LANG005.C6
```

The patch-grade P1 plan is complete against the shared working tree. T0 takeover
was re-verified, T1–T10 were implemented, and the affected crates plus
P1-focused semantic/compiler gates were checked. The two known Universe
capability failures remain inherited baseline evidence; they do not block the
focused P1 result. P2 remains the active successor for projection formation and
normalization.

The predecessor checkpoint, C4, remains recorded as `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED` because broad workspace/release certification is blocked by known pre-existing failures. Its **focused semantic architecture required by C5 is complete and stable**: explicit conformance, exact target/`TraitRef` matching, witness/default selection, `ConformanceEvidence`, trait-evidenced dispatch, semantic lowering, runtime execution, and source/editor projection all have focused evidence.

C5 must not reinterpret the inherited C4 broad blockers as missing C5 prerequisites unless a fresh reproducer proves they affect the C5 semantic slice.

---

## 3. Repository planning baseline

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   e5152196d716ed70fc47d104a0fd789e602285db
commit:     docs: record C4 stress and certification evidence
prepared:   2026-09-15
```

Planning verified these repository facts:

1. `TraitDef.members` and `ImplDef.members` are currently behavior-only `Vec<BehaviorMember>` surfaces and must be generalized for associated declarations/bindings.
2. `BehaviorMember` is intentionally callable/accessor/index behavior; associated types must not be encoded as fake callables.
3. `TraitRequirementId` is behavior-specific and must remain behavior-specific.
4. `TraitSurface` currently owns behavioral requirements/defaults and needs a separate associated-type table.
5. `ImplId` is the stable source/provenance identity for one implementation declaration.
6. `ConformanceWitnessPlan` and exact `ConformanceEvidence` are stable C4 seams and must be extended, not replaced.
7. `ConformanceCompleteness` is the canonical conformance proof-state authority and must remain singular.
8. `FieldId` and `FieldSemanticSignature` already provide canonical field owner/name/side, mutability, and declared type for `via` resolution.
9. fields are private to the declaring class and are not inherited-visible; superclass field search is therefore forbidden for P1 `via`.
10. existing field/compiler architecture treats declared fields as representation/layout facts; `via` must not add storage.
11. `TypeTerm` currently has no associated-projection term.
12. generic constraints currently have no trait-conformance-assumption form.
13. semantic DB/query/fingerprint infrastructure already separates source-sensitive input identity from semantic product identity.
14. source index/editor/LSP already project canonical semantic identities and must extend that pattern rather than name-solve associated types independently.
15. runtime consumes proven semantic lowering products and does not scan conformance records or class dictionaries to prove trait behavior.
16. current mutable-field surface predates the newly ratified explicit `mut` examples; C5.P1 must add the required surface without opportunistically breaking legacy field syntax.

Implementation began by re-running:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

and re-reading the live files named by the active plan. Mechanical drift may be adapted. Architectural drift requires consultation. The live checkout is `main` at `e5152196d716ed70fc47d104a0fd789e602285db`; unrelated working-tree changes remain outside C5 scope and are not staged by this work.

---

## 4. Predecessor takeover state

### 4.1 Stable C4 seams

C5 inherits and must preserve:

```text
ImplId
TraitRef
TraitRequirementId
TraitSurface
ConformanceIndex
ConformanceHeadMatch
ConformanceWitnessPlan
ConformanceEvidence
ConformanceCompleteness
TraitDispatchContribution
TraitDispatchIndex
TraitDispatchSelection
TraitDispatchSite
TraitDispatchTerminal
retained trait-dispatch ambiguity candidates
semantic-to-core trait invocation projection
executable conformance plan
RuntimeConformanceEnvironmentId / runtime conformance environment
conformance-local detached witnesses
trait-owned detached defaults
editor/LSP projection through canonical semantic products
DiagnosticCode::TraitDispatchAmbiguous
```

The inherited authority flow remains:

```text
source conformance
    ↓
semantic plan/evidence
    ↓
semantic selection/normalization
    ↓
compiler lowering
    ↓
runtime execution
```

The VM/runtime must never become the place where conformance or associated-type meaning is discovered.

### 4.2 C4 distinctions C5 must preserve

```text
source ImplId != exact applied conformance
TraitRequirementId != CallableId
ConformanceHeadMatch != ConformanceEvidence
trait default declaration != selected default relation
conformance-local witness != inherent target behavior
trait-evidenced behavior != inherent behavior
representation != conformance
semantic authority != runtime execution mechanism
```

C5 adds new distinctions rather than erasing these:

```text
TraitRequirementId != AssociatedTypeRequirementId
source associated binding template != exact associated binding
associated type declaration != associated projection
projection formation != projection normalization
property-shaped requirement source syntax != physical field
`via` source syntax != runtime delegation protocol
```

### P1 T1 durable source-member taxonomy

The AST now keeps the P1 source categories lossless and separate from
`BehaviorMember`:

```text
TraitMember::Behavior(BehaviorMember)
TraitMember::Property(TraitPropertyRequirement)
TraitMember::AssociatedType(AssociatedTypeDeclaration)

ImplMember::Behavior(BehaviorMember)
ImplMember::Delegation(DelegatedAccessorDef)
ImplMember::AssociatedTypeBinding(AssociatedTypeBinding)

ClassMember::Delegation(DelegatedAccessorDef)
```

`via` accepts exactly one direct field identifier and does not parse an
expression. `mut` is contextual in member position: explicit mutable fields,
mutable trait properties, and read/write delegation are accepted while the
legacy unkeyworded mutable-field spelling remains accepted. No projection,
trait-bound, storage, or runtime delegation semantics are introduced by T1.

---

### P1 durable implementation state

The completed P1 implementation establishes these takeover interfaces:

```text
AssociatedTypeRequirementId { owner: DeclarationId, index: u32 }
TraitSurface.associated_types: BTreeMap<AssociatedTypeRequirementId, TraitAssociatedTypeRequirement>
AssociatedTypeBindingTemplate { requirement, value_template, source_impl, source }
ConformanceAssociatedTypePlan { impl_id, bindings, failures, diagnostics, fingerprint }
ConformanceEvidence.associated_types: BTreeMap<AssociatedTypeRequirementId, ExactAssociatedTypeBinding>
ConformanceFailure::AssociatedType alongside behavioral failures
SourceIndex SemanticTargetId::AssociatedType(AssociatedTypeRequirementId)
```

Trait properties elaborate to ordinary getter/setter requirements. Class and
inherent-impl `via` declarations elaborate to ordinary accessors over one
directly owned field; field type, mutability, privacy, and selector conflicts
remain authoritative in the existing field/member products. No storage,
projection solver, or runtime delegation/conformance authority was added.

---

## 5. Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome / next dependency |
|---|---|---|---|---|---|
| `LANG005.C5.P1` | Associated declarations/bindings, exact binding evidence, trait property elaboration, direct-field `via`, incremental/source foundations | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | T1–T10 complete; T11 records synchronized; P2 consumes the frozen identity/evidence seams |
| `LANG005.C5.P2` | Associated projection formation and normalization | PLANNED | NOT_STARTED | UNVERIFIED | Consumes P1 associated identities/bindings; establishes `Self::Item` and normalization semantics |
| `LANG005.C5.P3` | Vertical integration, Iterable migration, tooling/runtime consequences, stress/certification | PLANNED | NOT_STARTED | UNVERIFIED | Closes C5 and hands generic trait-bound proof work to C6 |

Canonical P1 plan:

```text
docs/implementation/LANG005/LANG005.C5/
  LANG005.C5.P1-associated-type-declarations-and-binding-foundations-plan.md
```

The planned filenames for P2/P3 should be chosen when their patch-grade plans are written; do not create empty placeholder plan files.

---

## 6. Checkpoint acceptance objective by plan

### 6.1 P1 — Associated declarations, bindings, and property/delegation foundation

P1 must establish:

```text
trait `type Item`
    → stable AssociatedTypeRequirementId
    → separate TraitSurface associated requirement

conformance `type Item = T`
    → LHS resolves to exact trait-owned associated requirement
    → source binding template under ImplId
    → exact specialization through C4 conformance environment
    → exact associated binding in ConformanceEvidence

trait `mut count: Int`
    → getter TraitRequirementId
    → setter TraitRequirementId
    → no storage requirement

class/impl `mut count via _count`
    → ordinary getter implementation
    → ordinary setter implementation
    → field type/mutability from canonical FieldSemanticSignature
    → ordinary C4 witness selection
```

P1 must end before general associated projection.

### 6.2 P2 — Projection formation and normalization

P2 must establish a canonical type-level associated projection model beginning with contextual trait-owned forms such as:

```phalcom
trait Iterator {
  type Item
  next() -> Option<Self::Item>
}
```

Projection identity must carry enough information to identify:

```text
subject
exact/symbolic trait application
AssociatedTypeRequirementId
```

and must never be reconstructed as `subject + name` alone.

P2 normalization must preserve semantic terminal states rather than collapsing unresolved projections to `Dynamic` by convenience. Required conceptual outcomes include:

```text
known exact
symbolic / abstract Self
unknown / insufficient evidence
blocked
dynamic boundary
ambiguous
recursive / cyclic
cancelled
budget exceeded
```

P2 owns projection in trait/conformance signatures/defaults where the relevant trait identity/evidence is available. It does not invent `T: Trait` assumptions.

If P2 requires a new user-visible explicit qualification syntax such as `<T as Trait>::Item`, implementation must STOP AND CONSULT; that syntax is not ratified by this checkpoint.

### 6.3 P3 — Vertical integration and certification

P3 must integrate the C5 semantic model through:

```text
associated-type-driven core library forms, especially Iterable
trait defaults and witness compatibility using projection
source index / hover / definition / diagnostics
compiler lowering of already-normalized type information
runtime type reification only where normalized generic information survives lowering
cold/incremental equivalence
negative diagnostic corpus
high-density valid stress programs
cross-checks against C4 exact generic and exact enum-case conformance
```

P3 must end with a truthful C5 verification classification and a C6 handoff.

---

## 7. Fixed checkpoint invariants

The following are accepted C5 design invariants. They are not optional implementation preferences.

### C5-INV-01 — Trait properties are behavioral, never representational

A trait declaration such as:

```phalcom
trait Counter {
  mut count: Int
}
```

creates ordinary getter/setter obligations equivalent to:

```phalcom
count -> Int
count=(_: Int) -> ()
```

It never creates storage metadata, a required field slot, or target layout.

### C5-INV-02 — No associated-state semantic category

C5 must not introduce:

```text
StateRequirementId
TraitFieldId
AssociatedStateBinding
conformance storage contribution
trait-owned instance layout
```

for property requirements.

### C5-INV-03 — `TraitRequirementId` remains behavioral-only

Associated type requirements receive a distinct stable identity.

### C5-INV-04 — Associated identity is trait-owned

`A.Item` and `B.Item` are distinct requirements even when both are spelled `Item`.

### C5-INV-05 — One source `impl` remains one `ImplId`

Generic exact applications do not manufacture new source identities.

### C5-INV-06 — Binding templates and exact bindings are distinct

For:

```phalcom
impl<T> Iterable for List<T> {
  type Item = T
}
```

the source binding is one template under one `ImplId`, while exact evidence may contain:

```text
List<Int>    → Item = Int
List<String> → Item = String
```

### C5-INV-07 — `ConformanceEvidence` remains exact proof authority

Associated exact bindings extend C4 evidence; they do not create a replacement evidence system.

### C5-INV-08 — Conformance completeness remains singular

Behavioral failures and associated-binding failures jointly determine the existing conformance proof state.

### C5-INV-09 — `via` is compile-time accessor elaboration

```phalcom
count via _count
count=(_) via _count
mut count via _count
```

elaborate to ordinary accessors. The VM does not receive a `via` protocol or opcode.

### C5-INV-10 — Delegate target is initially an own field only

P1 resolves a directly named field belonging to the exact target declaration. It does not search superclasses and does not accept property chains, subscripts, calls, arbitrary places, delegate objects, or delegate protocols.

### C5-INV-11 — Delegate field is the type authority

A delegated accessor does not independently repeat/infer another property type. The canonical field signature supplies its value type; delegated setter legality also consumes field mutability.

### C5-INV-12 — Generated accessors obey ordinary conflict law

A derived setter/getter cannot silently override or yield to an explicit user member. Duplicate selector/conflict rules remain authoritative.

### C5-INV-13 — Field privacy/non-inheritance remains intact

Fields remain private representation of the declaring class. Cross-hierarchy behavior is exposed through accessors, not inherited field visibility.

### C5-INV-14 — Exact target identity survives associated binding specialization

Exact generic applications and exact enum cases retain their canonical exact target identity, including `VariantId` where applicable.

### C5-INV-15 — Projection is semantic, not runtime-discovered

P2 normalizes associated projections using canonical semantic evidence. Runtime never scans trait declarations, conformance records, or class dictionaries to solve them.

### C5-INV-16 — Unresolved projection is not automatically `Dynamic`

Unknown, blocked, ambiguous, recursive, dynamic-boundary, cancelled, and budget states remain distinguishable according to existing semantic proof-state conventions.

### C5-INV-17 — C5 stops before generic trait-assumption proof

`T: Trait`, conditional conformance, and nested generic conformance evidence belong to C6.

### C5-INV-18 — Local parameter bindings do not affect selector identity

Bodyless setter requirements may use discard syntax such as `count=(_: Int) -> ()`; generated executable setter internals may use a synthetic local binding without changing source-level identity.

### C5-INV-19 — Incremental replacement is owner-complete

Add/edit/delete/rename of associated declarations, bindings, property requirements, delegated accessors, or delegate fields must not leave stale semantic/source/tooling products.

### C5-INV-20 — C5 does not opportunistically break legacy mutable-field syntax

P1 adds the ratified explicit `mut` surface required by the new property/delegation forms. Whole-language removal of pre-existing field syntax is outside this checkpoint unless a newer authoritative specification has already made that migration mandatory.

---

## 8. Stable interface / takeover map

| Concept | Canonical current owner | C5 treatment |
|---|---|---|
| `ImplId` | `phalcom-semantic/src/identity.rs` | Preserve as source/provenance identity |
| `TraitRef` | trait semantic layer | Preserve exact trait application identity |
| `TraitRequirementId` | trait semantic layer | Preserve as behavior-only identity |
| `TraitSurface` | trait semantic layer | Extend with separate associated-type requirements |
| `ConformanceIndex` | conformance semantic layer | Reuse for exact applicable conformance discovery |
| `ConformanceHeadMatch` | conformance semantic layer | Reuse exact target/trait/impl substitution for associated binding specialization |
| `ConformanceWitnessPlan` | conformance semantic layer | Preserve behavioral source plan; do not stuff type bindings into witness selections |
| `ConformanceEvidence` | conformance semantic layer | Extend with exact associated binding map |
| `ConformanceCompleteness` | conformance semantic layer | Remain one final conformance proof-state authority |
| `CallableId` | semantic identity | Synthesized `via` accessors use ordinary callable identity |
| `FieldId` | semantic identity | Canonical delegate target identity |
| `FieldSemanticSignature` | `signature.rs` | Canonical delegate type/mutability source |
| `TypeEnvironment` / substitutions | type semantic layer | Reuse for exact associated binding specialization |
| `TypeTerm` | type semantic layer | P2 extends as needed for projection; P1 does not |
| semantic DB fingerprints/query products | DB layer | Extend owner-completely for associated/property/delegation products |
| source index/editor | semantic projection layer | Project canonical associated/property/delegate identity |
| semantic→core lowering | core projection | Consume already-authorized/normalized semantic products |
| runtime conformance/type environment | core/runtime | Consume lowered exact information; never solve C5 semantics |

This table must be refreshed when a completed plan materially changes an interface.

---

## 9. State/property requirement reconciliation

The earlier phrase “associated state” is resolved and must not reappear as an unresolved design fork.

The accepted interpretation is:

```phalcom
trait Counter {
  private mut count: Int

  increment {
    count++
  }
}
```

is source grouping for a behavioral property contract. It elaborates to requirements equivalent to:

```phalcom
private count -> Int
private count=(_: Int) -> ()
```

A conforming type may satisfy those through explicit accessors or direct-field delegation:

```phalcom
class CounterImpl {
  mut _count: Int
  mut count via _count
}
```

whose behavior is equivalent to ordinary accessors over `_count`.

The conformance engine therefore sees only ordinary getter/setter witnesses. It must not contain a rule of the form “field X satisfies state requirement Y.”

The higher-level property declaration should remain available as source/provenance grouping for diagnostics/tooling where useful, but that source grouping is not a new canonical conformance identity.

---

## 10. Direct-field `via` initial semantics

The initial accepted forms are:

```phalcom
count via _count
```

Getter only.

```phalcom
count=(_) via _count
```

Setter only.

```phalcom
mut count via _count
```

Getter + setter.

Rules:

1. the delegate field is declared separately;
2. `via` declares no storage;
3. the delegate target is a directly named field owned by the exact target declaration;
4. field lookup does not traverse superclasses;
5. the field's semantic type determines the accessor value type;
6. setter-only/mutable delegation requires a writable field;
7. accessor visibility comes from the accessor declaration's normal member visibility pipeline, not from a new storage-visibility rule;
8. class-body and inherent-impl forms converge to the same effective behavior model;
9. generated accessors participate in ordinary duplicate/conflict/conformance rules;
10. the VM/runtime does not know `via` existed.

Future delegation mechanisms—subscripts, paths, delegate objects, protocol-based delegation—require a later explicit design checkpoint/amendment.

---

## 11. Associated type identity and binding policy

The canonical conceptual identity is trait-owned and stable, for example:

```text
AssociatedTypeRequirementId {
    owner: DeclarationId,
    index: u32,
}
```

The exact Rust representation may adapt to live repository conventions, but these semantic properties are fixed:

```text
name is presentation, not identity
same spelling in different traits is distinct
identity is stable across conformances
identity is not CallableId
identity is not TraitRequirementId
identity is not TypeParameterId
```

A conformance binding:

```phalcom
impl<T> Iterable for List<T> {
  type Item = T
}
```

must resolve the LHS through the exact conformance trait surface:

```text
ImplDef
    ↓
resolved TraitRef
    ↓
trait declaration / TraitSurface
    ↓
name → AssociatedTypeRequirementId
```

The binding RHS is formed under the implementation's ordinary generic/type environment.

There is no name-only global `Item` resolution.

---

## 12. Projection normalization policy

P2 begins from P1's canonical binding map.

The initial contextual form is:

```phalcom
Self::Item
```

inside a trait/conformance context where the owning trait identity is available.

A semantic projection must not be represented only as text. Conceptually it carries:

```text
subject type term
relevant trait application / trait context
AssociatedTypeRequirementId
source/provenance
```

Normalization follows conformance evidence, not runtime behavior lookup.

For exact evidence:

```text
List<Int> + Iterable
    evidence Item = Int
        ↓
projection
    <List<Int> as Iterable>.Item
        ↓
Int
```

For generic/abstract contexts, the result may remain symbolic when proof is incomplete or intentionally abstract.

P2 must diagnose or preserve ambiguity rather than choosing by name/source order.

C5 does not currently ratify an explicit user-visible qualification spelling such as `<T as Iterable>::Item`. If a correct P2 architecture materially requires one, record a consultation incident and obtain an explicit language-design decision.

---

## 13. Incremental and fingerprint requirements

C5 semantic products must preserve cold/incremental equivalence.

Required dependency behavior includes:

```text
associated declaration shape change
    → invalidates TraitSurface associated shape and dependents

associated binding RHS change
    → invalidates binding plan / exact evidence / dependent projections
    → does not invalidate unrelated TraitSurface products

property requirement add/remove/change
    → invalidates elaborated getter/setter requirement surface

via add/remove/change
    → invalidates synthesized accessor contribution/effective behavior

delegate field type/mutability change
    → invalidates delegated accessor signature/legality and dependent evidence

source replacement/removal
    → removes all owned associated/property/delegation/source-index products
```

C5 may extend existing product keys or add bounded new keys where architectural ownership is clear. It must not solve invalidation through broad whole-workspace recomputation when existing owner-indexed publication/removal patterns can represent the dependency.

---

## 14. Source/editor/tooling requirements

C5 tooling must project canonical semantic identities.

At minimum:

```phalcom
trait Iterable {
  type Item
}
```

must expose a source target for the associated declaration, and:

```phalcom
impl Iterable for Foo {
  type Item = Int
}
```

must resolve the LHS `Item` to that trait-owned declaration.

Property/delegation source provenance should allow diagnostics/navigation to describe the higher-level source declaration while conformance remains keyed by ordinary accessors.

P2/P3 must extend navigation/hover/presentation to projection uses from canonical associated identities. Tooling must not implement an independent `Item` resolver.

---

## 15. Compiler/runtime authority

C5 preserves C4's anti-authority boundary:

```text
semantic evidence / normalized type
    ↓
compiler lowering
    ↓
bytecode/runtime metadata
    ↓
VM execution
```

Forbidden runtime responsibilities:

```text
scan trait declarations for `Item`
scan impls for a binding
resolve ambiguous conformance
choose a `via` delegate
decide conformance completeness
normalize associated projection
```

`via` must lower as ordinary getter/setter behavior.

If a normalized associated type survives into runtime generic reification, C5 should use the existing `RuntimeTypeRecipe` / runtime type-environment architecture after semantic normalization rather than add a trait-specific runtime solver.

---

## 16. C6 boundary

C6 begins where associated projection requires **generic trait assumptions/proofs** rather than an already-known trait/conformance context.

C6 owns:

```text
generic T: Trait constraints
canonical trait-conformance GenericConstraint form
generic assumption identity
proof query for assumed conformance
conditional conformance on trait evidence
nested ConformanceEvidence
generic-body trait dispatch through assumptions
associated projection through assumed evidence
cycle/coherence handling for those assumptions
lowering/tooling of generic trait proofs
```

C5 must not add a weak one-off `T: Trait` mechanism merely to make `T::Item` work.

C2 conditional inherent behavior remains a separate concept from C6 conditional trait conformance.

The language still has no “most specific conformance wins” rule.

---

## 17. Verification ledger

P1 implementation and focused stabilization are complete. The live test
organization differs from a few planned filters; equivalent selectors are
recorded below.

| Gate | Scope | Result | Meaning |
|---|---|---|---|
| Planning audit | repository/C4 takeover + C5 requirements | COMPLETE | P1 requirements and architecture are grounded against `e5152196...` |
| P1 T0 takeover | C4 semantic/conformance/compiler seams | PASS with inherited baseline exceptions | predecessor evidence retained; current P1 reruns include `impls::queries` 53/53 and exact generic source conformance runtime 1/1 | C4 ownership and focused takeover behavior remain established |
| P1 G1 | heterogeneous source-member taxonomy + contextual `mut` compatibility | PASS | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test trait_syntax` (6/6); `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration impl_syntax` (16/16); `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core -p phalcom-lsp` | `TraitMember`, `ImplMember`, and direct-field delegation preserve source categories; `BehaviorMember` remains behavior-only; explicit contextual `mut` coexists with legacy unkeyworded fields. The plan's standalone `--test impl_syntax` target does not exist in the live manifest; the integration-module selector is the equivalent. |
| P1 G2 | property/delegation semantic elaboration | PASS | `trait_property_and_class_via_publish_ordinary_accessor_surface`, `conformance_local_via_is_rejected_without_target_private_field_access`, `delegated_accessors_conflict_with_explicit_ordinary_accessors` | property requirements and direct-field `via` converge through ordinary callable products and ordinary conflict diagnostics |
| P1 G3 | associated surface/binding diagnostics | PASS | associated identity/evidence test; negative binding matrix; inherent-impl rejection; trait-local identity test | exact trait-owned associated identity, binding plan, and diagnostics are published |
| P1 G4 | exact associated evidence + joint completeness | PASS | `associated_type_surface_uses_trait_owned_identity_and_exact_binding_evidence`; `associated_type_binding_validation_is_part_of_conformance_completeness`; `impls::queries` 53/53 | generic source bindings specialize per exact target while behavioral and associated failures share one completeness state |
| P1 G5 | incremental lifecycle | PASS | `cargo test -p phalcom-semantic --test semantic incremental::associated_types` (3/3); incremental `db` (14/14) | replacement/removal/readdition and cold parity preserve P1 products |
| P1 G6 | executable ordinary accessor lowering | PASS | `direct_field_delegation_executes_as_ordinary_accessors` (1/1); `delegated_getter_coexists_with_custom_inherent_setter` (1/1); exact generic C4 runtime (1/1) | `via` executes as ordinary field accessors; no runtime delegation/conformance solver exists |
| P1 G7 | final focused stabilization | PASS with inherited baseline exceptions | affected-crate check passed; AST 6/6 + 16/16; semantic traits 21/23; semantic impl queries 53/53; incremental associated types 3/3; core P1 equivalents passed | the two `capabilities::traits` failures are C5-BL-07 Universe Bool dependency/capability baseline failures, classification D |
| P2 gates | pending | NOT RUN | Defined by future P2 patch-grade plan |
| P3/certification gates | pending | NOT RUN | Defined by future P3 patch-grade plan |

Do not convert inherited C4 focused evidence into a C5 PASS entry. C5 must gather its own evidence.

---

## 18. Deferred / baseline issue ledger

C5 inherits these broad baseline blockers from C4 unless a fresh live run proves they have been resolved:

### C5-BL-01 — Universe generic call-entry failure

```text
Classification: inherited baseline
C5 impact: does not block focused semantic P1 work unless reproduced by a C5-specific vertical
Policy: record; do not opportunistically repair inside P1
```

### C5-BL-02 — incremental A7 cold/incremental presentation mismatch

```text
Classification: inherited baseline
C5 impact: C5 still must prove its own newly-added incremental products cold/incremental equivalent
Policy: distinguish new C5 mismatch from the known unrelated presentation mismatch
```

### C5-BL-03 — Universe Bool capability failures

```text
Classification: inherited baseline
C5 impact: unrelated unless a C5 path demonstrably depends on the failing capability
Policy: do not weaken C5 assertions to accommodate them
```

### C5-BL-04 — Iterable/outgoing-pack generic call-entry failures

```text
Classification: inherited baseline
C5 impact: may become relevant in P3 Iterable migration; P1 must not absorb this baseline repair without evidence
Policy: reclassify only when a C5-specific discriminator demonstrates causal coupling
```

### C5-BL-05 — formatting drift

```text
Classification: inherited baseline
Policy: format touched files; do not broad-reformat unrelated tree
```

### C5-BL-06 — AST Clippy violations

```text
Classification: inherited baseline
Policy: repair only new/touched violations caused by C5; do not expand scope to legacy Clippy cleanup
```

### C5-BL-07 — T0 trait capability baseline failures

```text
Classification: inherited baseline (D)
Observed: `capabilities::traits` reports failures in `trait_default_replays_external_nominal_callable_dependencies` and `trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities`; both failures involve unexpected Universe `Bool` declaration dependencies/capabilities.
C5 impact: none established; C4 `impls::queries` and the exact generic runtime takeover regression remain green.
Policy: do not repair or weaken these assertions during P1 unless a C5-specific discriminator demonstrates causal coupling.
```

The ledger must be updated from actual live evidence during implementation. Removed blockers should be marked resolved with the gate/revision that proves it.

---

## 19. Consultation / amendment ledger

### C5-DEC-01 — Associated state / field requirement reconciliation

```text
Decision:
Trait field/property-shaped declarations are source-level behavioral property contracts.
They elaborate to ordinary getter/setter requirements and create no storage identity,
layout contribution, or conformance-added state.

Architecture impact:
No StateRequirementId or associated-state binding product exists in C5.
C4 TraitRequirementId remains the canonical conformance identity for the getter/setter obligations.
```

### C5-DEC-02 — Direct-field `via` ratified

```text
Decision:
Initial delegation syntax is:
  count via _field
  count=(_) via _field
  mut count via _field

The delegate is an already-declared directly owned field.
The field supplies value type; writable delegation requires mutable field.
The forms are valid in class and inherent impl declarations.

Architecture impact:
`via` is accessor elaboration only; no runtime protocol and no general place delegation.
```

### C5-DEC-03 — Fields remain private/non-inherited

```text
Decision:
Current field model is preserved. P1 `via` does not traverse superclass fields.
A parent must expose getter/setter behavior when cross-hierarchy access is intended.
```

### C5-DEC-04 — Generated accessor conflicts are ordinary conflicts

```text
Decision:
A synthesized getter/setter cannot silently override or be replaced by an explicit member.
Use getter-only or setter-only delegation when custom complementary behavior is needed.
```

### C5-INC-01 — Conformance-local `via` deferred after architectural consultation

```text
Plan/task: LANG005.C5.P1 T2/T9
Trigger: C4 conformance-owned bodies lack target-class private-field authority.
Observed: conformance callable body analysis sets current_class = None; bare or
receiver-local field lookup cannot use the target declaration's private FieldId.
Decision: support direct-field via only in class bodies and inherent impls;
reject conformance-local via deterministically. Trait properties may still use
delegated accessors published on the target's ordinary inherent surface.
Architecture impact: preserve C4 CallableOwnerId::Conformance(ImplId), body
ownership, exact-owner FieldId lookup, and no Via runtime product.
Plan amendment: REQUIRED — Level 2. The conditional conformance-local branch
in T2/T9 is superseded; shared parser recognition is retained for diagnosis.
Verification required: class/inherent success, immutable/inherited rejection,
ordinary property witness convergence, and conformance-local diagnostic.
Status: adopted; implementer may resume.
```

### C5-DEC-05 — Local binding names are signature-irrelevant

```text
Decision:
External labels/selectors and types define callable contract shape; local parameter bindings
are lexical implementation detail. Bodyless requirements may explicitly discard parameters.
```

### C5-DEC-06 — P1 surface compatibility for `mut`

```text
Decision:
C5 needs explicit `mut` for the ratified property/delegation forms. P1 must not opportunistically
turn that into a whole-language breaking migration of legacy mutable-field syntax unless a newer
authoritative spec already requires it.
```

### C5-INC-02 — Generated delegation lowering gate correction

```text
Plan/task: LANG005.C5.P1 T9
Trigger: the first executable direct-field delegation vertical compiled the
generated accessor but the semantic lowering gate discarded the accepted
definition because it had no source-body CallableAnalysis.
Observed: generated delegation has no source expression body by design;
compile_delegated_accessor supplies the ordinary executable body.
Decision: retain an accepted inherent definition when its exact
(ImplId, source_member_index) identifies ImplMember::Delegation, while
body-backed behavior definitions continue to require CallableAnalysis.
Architecture impact: no synthetic analysis product and no second authorization
source; AST classification only locates an already accepted definition.
Consultation: adviser task 01a0a645-6654-7b20-99dc-973a16164e05, decision
PROCEED, no plan amendment.
Status: adopted and verified by the direct-field runtime test.
```

No unresolved C5 design incident is currently open.

---

## 20. Completed plan and next action

Latest completed plan:

```text
LANG005.C5.P1
Associated Type Declarations, Binding Foundations, and Trait Property Delegation
```

Immediate next action for the next implementer:

```text
1. read this checkpoint, the P1 walkthrough, and the P1 handoff;
2. read the P2 plan when it is authored and verify the frozen P1 interfaces;
3. form and normalize `Self::Item` through canonical semantic evidence;
4. do not recompute bindings from syntax or redesign P1 identity, completeness,
   property, or `via` products.
```

Before writing/executing P2, preserve this record and extend it only with new
P2 decisions, gates, failures, and the next durable state:

```text
plan ledger
implemented invariants
stable interface map
verification ledger
deferred issue ledger
consultation/amendment ledger
active/next plan
```

---

## 21. C5 checkpoint completion criteria

C5 may be marked `COMPLETE` only when all of the following are true:

1. P1 associated declarations/bindings and property/delegation foundation are implemented and focused-tested.
2. P2 canonical associated projection and normalization are implemented and focused-tested.
3. P3 integration/certification work is complete or explicitly superseded by an accepted corrective plan within C5.
4. associated declarations have stable trait-owned identity across modules/conformances.
5. generic source bindings specialize correctly for exact target applications under one source `ImplId`.
6. exact enum-case identity is preserved.
7. conformance completeness jointly accounts for behavioral and associated obligations through one proof-state authority.
8. `Self::Item` works in the ratified contextual surfaces owned by C5.
9. projection terminal states are not unsoundly collapsed.
10. trait property requirements and `via` implementations converge through ordinary C4 witness selection without representation injection.
11. source replacement/removal and cold/incremental equivalence are proven for C5 products.
12. source index/editor/LSP project canonical associated identities rather than name-solving independently.
13. compiler/runtime consume already-normalized semantic information and introduce no runtime projection/conformance solver.
14. the associated-type-driven Iterable integration planned for C5 is implemented or an explicit accepted checkpoint amendment records its deferral.
15. residual broad failures are classified truthfully.
16. the checkpoint record, final walkthrough, and C6 handoff reflect actual implementation state.

`COMPLETE / IMPLEMENTED / FOCUSED_TESTED` is a valid final checkpoint state if all C5 acceptance obligations have focused evidence but unrelated workspace/release blockers remain. `RELEASE_COMPLETE` requires the named broad certification gates to actually pass.

---

## 22. Completion truth table

Do not conflate:

```text
P1 source written
P1 tests added
P1 focused tests passed
P1 complete
P2 projection complete
P3 integration complete
C5 implemented
C5 focused-tested
workspace green
release certified
```

The checkpoint metadata must reflect the strongest evidence actually obtained, not desired state.

---

## 23. Durable next-checkpoint handoff boundary

When C5 closes, C6 should inherit these stable facts rather than redesign them:

```text
AssociatedTypeRequirementId and associated TraitSurface requirements
source associated binding plan identity/provenance
exact associated binding map in ConformanceEvidence
canonical projection term/identity
projection normalization query and terminal-state model
property requirement → getter/setter elaboration
via → ordinary accessor elaboration
incremental/source-index ownership for all of the above
runtime anti-authority boundary
```

C6 may then add generic trait assumptions, proof queries, conditional conformance, and nested evidence on top of those products.
