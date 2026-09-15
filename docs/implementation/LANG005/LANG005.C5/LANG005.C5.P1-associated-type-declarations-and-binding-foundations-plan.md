---
id: LANG005.C5.P1
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C4 semantic completion
follows: LANG005.C4.P4
supersedes: null
prepared: 2026-09-15
repository: aureat/phalcom-lang
repository_baseline: e5152196d716ed70fc47d104a0fd789e602285db
repository_baseline_commit: "docs: record C4 stress and certification evidence"
intended_repository_path: docs/implementation/LANG005/LANG005.C5/LANG005.C5.P1-associated-type-declarations-and-binding-foundations-plan.md
next_plan: LANG005.C5.P2
---

# LANG005.C5.P1 — Associated Type Declarations, Binding Foundations, and Trait Property Delegation

## Luna Patch-Grade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use the repository's Luna implementation workflow together with `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Execute task-by-task. Do not improvise architecture outside the authority explicitly granted by this plan.

**Goal:** Establish first-class associated type declarations and conformance bindings with exact specialized conformance evidence, while implementing the minimum trait-property and direct-field `via` elaboration required by the ratified trait model.

**Architecture:** C5.P1 extends the C4 semantic spine instead of replacing it. Trait property declarations elaborate to ordinary behavioral `TraitRequirementId`s; direct-field `via` declarations elaborate to ordinary getter/setter callable contributions; associated types receive a distinct trait-owned identity and conformance-dependent binding plan; exact associated bindings are materialized into the existing `ConformanceEvidence`. No new storage model, projection solver, generic trait-bound proof engine, or runtime conformance authority is introduced.

**Tech stack:** Rust workspace; `phalcom-ast`; `phalcom-modules`; `phalcom-semantic`; `phalcom-core`; `phalcom-lsp`; canonical semantic DB/query/fingerprint infrastructure; existing C4 conformance/evidence architecture.

**Normative specs / design authority:** `docs/specs/objects/traits.md`, current field/privacy rules under `docs/spec/current/classes.md` and `docs/spec/current/object-model.md`, C4 checkpoint/guidance/walkthrough/handoff, and the user-ratified C5 amendments recorded in this plan.

---

# 0. Executor contract

This plan is written for a Luna-class implementer operating under constrained architectural authority.

The implementer:

- may adapt private helper names, small file decomposition, and mechanically equivalent APIs to the live repository;
- must preserve every `INV-*` invariant below;
- must extend C4 identities/products rather than create replacements;
- must not create a second conformance-completeness engine;
- must not treat trait property declarations as storage requirements;
- must not treat `via` as a runtime delegation protocol;
- must not introduce associated-type projection, `T: Trait`, conditional conformance, GATs, associated defaults, reflection descriptors, trait objects, or runtime associated-type lookup;
- must keep tests selective and tied to the current task/gate;
- must classify unrelated failures and preserve the known C4 baseline blockers;
- must STOP AND CONSULT on every trigger named in Sections 13 and the task-local trigger sections;
- must keep the C5 checkpoint record current once C5 implementation begins;
- must produce the P1 walkthrough and P2 handoff before declaring P1 administratively complete.

Testing is evidence gathering, not ritual. Do not run broad suites merely because a file changed.

---

# 1. Goal

Implement the C5.P1 foundation in four convergent slices:

1. **Trait property requirements**
   - parse and preserve property-shaped trait declarations such as `mut count: Int`;
   - elaborate them into ordinary getter/setter behavioral requirements;
   - retain higher-level source provenance for diagnostics/tooling;
   - never create trait-owned storage or a state-requirement identity.

2. **Direct-field accessor delegation**
   - support `count via _field`, `count=(_) via _field`, and `mut count via _field` in class and `impl` declarations;
   - resolve only directly owned target fields in this first version;
   - derive accessor type from the field's canonical field signature;
   - require field mutability for delegated setters;
   - lower to ordinary accessor behavior without new bytecode/runtime machinery.

3. **Associated type declarations/bindings**
   - parse `trait T { type Item }` and conformance `type Item = X` members;
   - give each associated declaration a stable trait-owned semantic identity distinct from behavioral requirements;
   - publish associated requirements in `TraitSurface` separately from behavioral members;
   - resolve conformance binding LHS names against that exact trait surface;
   - diagnose duplicate, extra, missing, invalid, or inherent-impl bindings.

4. **Exact associated binding evidence**
   - retain one source binding template per source `ImplId`;
   - exact-specialize the binding through the same conformance environment used by C4;
   - extend `ConformanceEvidence` with exact associated bindings;
   - make final conformance completeness jointly depend on behavioral satisfaction and associated binding satisfaction;
   - preserve exact generic and exact enum-case identity;
   - integrate fingerprints, replacement/removal, source index, and minimal tooling.

P1 ends before type-level projection. It intentionally produces the exact binding map that P2 will normalize projections against.

---

# 2. Checkpoint acceptance objective

C5 is a multi-plan checkpoint. P1 is the foundation, not full C5 closure.

The intended checkpoint progression is:

```text
C4
    TraitSurface behavioral requirements
    explicit conformance / ImplId
    ConformanceWitnessPlan
    exact ConformanceEvidence
    trait-evidenced dispatch/lowering/runtime
        ↓
C5.P1
    trait property requirement elaboration
    direct-field `via` accessor elaboration
    associated type declaration identity
    conformance associated binding plan
    exact specialized binding evidence
    combined completeness
        ↓
C5.P2
    type-level associated projection
    contextual Self::Item
    normalization / cycles / ambiguity / proof-state preservation
        ↓
C5.P3
    vertical integration
    Iterable migration
    editor/LSP projection UX
    runtime reification where required
    stress/certification
        ↓
C6
    T: Trait assumptions
    generic trait-proof machinery
    conditional conformance
    nested evidence
```

P1 is accepted when the semantic system can answer both of these questions without projection syntax:

```text
1. Which ordinary getter/setter obligations does this trait property declaration create,
   and which ordinary members satisfy them?

2. For this exact target + exact TraitRef proven by C4,
   what exact type is bound to each trait-owned associated type declaration?
```

P1 is **not** accepted merely because the parser accepts `type Item` or `via`.

---

# 3. Repository grounding

Prepared against remote `main`:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   e5152196d716ed70fc47d104a0fd789e602285db
commit:     docs: record C4 stress and certification evidence
```

Verified planning facts:

1. `TraitDef.members` and `ImplDef.members` are currently `Vec<BehaviorMember>`; `BehaviorMember` is behavior-only.
2. `ClassMember` already distinguishes `Field(FieldDef)` from getter/setter/method/index behavior.
3. `FieldId` is canonical as `(owner DeclarationId, name, DispatchSide)`.
4. `FieldSemanticSignature` canonically publishes field mutability and declared type.
5. fields are private to the declaring class and are not inherited-visible; a subclass using the same field spelling owns a new slot.
6. field access is receiver-local rather than ordinary dynamic dispatch.
7. `TraitRequirementId` is behavioral and keyed by trait owner + selector + dispatch side.
8. `TraitSurface` currently contains only behavioral requirements.
9. C4's stable seams include `ImplId`, `TraitRef`, `ConformanceIndex`, `ConformanceHeadMatch`, `ConformanceWitnessPlan`, `ConformanceEvidence`, `TraitDispatch*`, semantic lowering, and executable runtime conformance plans.
10. runtime does not scan traits/conformances/class dictionaries to prove conformance.
11. `TypeTerm` currently has only canonical, `SelfType`, and inference terms; there is no projection term.
12. generic constraints currently contain subtype/equivalence relations, not trait-conformance assumptions.
13. `Token::TypeKw` already exists, so member-position associated-type parsing should reuse it without disturbing top-level aliases.
14. current mutable field surface is older than the ratified examples: mutable fields are presently unkeyworded, and `mut` is not a lexer token.
15. current visibility predominantly flows through the existing attribute/visibility pipeline; C5.P1 must not reserve or redesign `private`/`protected` syntax solely for this feature.
16. C4 remains recorded `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED` because of broad pre-existing blockers, even though focused C4 semantics are complete.

Implementation sessions must re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

# 4. Required reads before implementation

Read in this order before production edits.

## 4.1 Workflow authority

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
```

## 4.2 LANG005 predecessor state

```text
docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4-GUIDANCE.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P4-walkthrough.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P4-handoff.md
```

If the C5 checkpoint/guidance documents have been created by execution time, read them next and treat accepted amendments there as higher implementation authority than this prospective plan.

## 4.3 Normative language/design authority

```text
docs/specs/objects/traits.md
docs/spec/current/classes.md
docs/spec/current/object-model.md
relevant callable/selector/visibility specifications
```

## 4.4 Production code

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/selector.rs

phalcom-semantic/src/identity.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/source_index/*

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/impl_decl.rs
phalcom-core/src/compiler/attributes.rs only to understand existing synthesis boundaries

phalcom-lsp/src/semantic_tokens.rs
live definition/hover navigation adapters
```

## 4.5 Test conventions

```text
phalcom-semantic/tests/semantic/README.md
phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md
phalcom-core/tests/README.md
phalcom-ast/tests/impl_syntax.rs
phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/incremental/*
phalcom-semantic/tests/semantic/integration/*
phalcom-core/tests/core/language/traits_p3_closure.rs
```

Avoid broad repository exploration after these reads unless a concrete implementation question requires it.

---

# 5. Normative authority and ratified C5 amendments

Use this precedence if sources disagree:

```text
1. explicit user-ratified C5 decisions recorded below
2. current authoritative specs under docs/specs/
3. C5 checkpoint/guidance amendments, once accepted
4. completed C4 checkpoint/walkthrough/handoff contracts
5. landed production architecture
6. this implementation plan for mechanics
7. legacy/current-draft documents under docs/spec/current when superseded by the above
8. old experimental/support documents
```

## 5.1 Ratified associated-type behavior

The following are fixed:

```phalcom
trait Iterable {
  type Item
}

impl<T> Iterable for List<T> {
  type Item = T
}
```

- associated declarations are trait-owned;
- bindings are conformance-dependent;
- associated types are not ordinary trait generic arguments;
- one source `ImplId` may produce different exact binding values for different exact target applications;
- P1 does not implement projection.

## 5.2 Ratified trait property/state-requirement meaning

This source:

```phalcom
trait Counter {
  mut count: Int

  increment {
    count++
  }
}
```

means a property-shaped behavioral contract, equivalent in conformance obligations to:

```phalcom
count -> Int
count=(_: Int) -> ()
```

It does **not** mean the trait owns, injects, or requires a physical field slot.

A read-only trait property:

```phalcom
count: Int
```

creates only the getter obligation.

No `StateRequirementId`, `TraitFieldId`, associated-state binding, target layout contribution, or runtime storage contract may be introduced.

## 5.3 Ratified `via` surface

Direct field delegation is initially:

```phalcom
class Counter {
  mut _count: Int

  count via _count
  count=(_) via _count
  mut count via _count
}
```

The three delegate forms are alternatives for one accessor surface:

```text
count via _count
    -> getter only

count=(_) via _count
    -> setter only

mut count via _count
    -> getter + setter
```

The field must be declared separately and is the type authority. Setter delegation requires writable storage.

`via` is valid in class bodies and `impl` bodies. It is syntactic/semantic accessor elaboration, not runtime delegation.

P1 delegates only to a directly named field. No property chain, subscript, call expression, arbitrary place, delegate object, or delegate protocol is permitted.

## 5.4 Field ownership

Fields are private and non-inherited-visible. Therefore a P1 `via` target is resolved against the exact target declaration's own field table only.

Do not search superclasses for a delegate field.

## 5.5 Member conflicts

A synthesized accessor never silently yields to or overrides a user-declared accessor. Ordinary selector conflict rules apply.

If a custom setter is desired, use getter-only delegation plus an explicit setter. If a custom getter is desired, use setter-only delegation plus an explicit getter.

## 5.6 Parameter binding semantics relevant to P1

Local parameter bindings are not selector identity. For bodyless declarations, local names are semantically inert.

The canonical generated setter requirement may therefore be shown as:

```phalcom
count=(_: Int) -> ()
```

`count=(_) via _count` deliberately carries no source-local binding; the compiler may use an internal synthetic value binding when materializing executable code.

P1 must not expand into a full parameter-grammar redesign unless the live parser cannot represent this already-ratified discard form without architectural changes.

## 5.7 Surface compatibility rule for `mut`

The ratified examples require explicit `mut` in property/delegation syntax and permit explicit `mut` on the field declaration.

Current `main` still treats unkeyworded fields as mutable. P1 must:

- accept the ratified explicit `mut` member spelling;
- preserve legacy unkeyworded mutable-field parsing during P1 unless a newer authoritative spec has already migrated the repository;
- avoid a whole-language compatibility removal in this plan;
- prefer a contextual `mut` parser form over globally reserving a new keyword if that is mechanically sufficient.

If the live authoritative spec has already resolved this migration differently, follow it and record the mechanical drift.

## 5.8 Visibility spelling is not a C5.P1 redesign

Property requirements and delegated accessors carry ordinary member visibility.

Do not reserve a new `private`/`protected` keyword solely for P1. Reuse the live canonical visibility/modifier/attribute pipeline. The semantic examples in design discussion describe visibility behavior, not permission to create a parallel trait-only visibility grammar.

## 5.9 Consultation amendment — conformance-local `via` is deferred

The referenced architectural consultation confirmed that C4 conformance-owned
callables deliberately have no lexical target-class ownership (`current_class =
None`), so they cannot access the target declaration's private fields without
weakening the established ownership model. This supersedes the earlier
conditional conformance-local examples in this plan:

```text
Supported in P1: class-body and inherent-impl `via` declarations.
Deferred/rejected in P1: conformance-local `via` declarations.
```

Shared parsing may still recognize the form so semantic classification can
issue a deterministic diagnostic, but T2 and T9 must not create or execute a
conformance-local delegated witness. Trait properties remain satisfiable by
delegated accessors already published on the target's ordinary inherent
surface.

---

# 6. Takeover state and stable interface map

| Concept | Current symbol/path | Owner | P1 invariant |
|---|---|---|---|
| source implementation provenance | `ImplId` / `phalcom-semantic/src/identity.rs` | semantic | one source identity remains shared by all exact applications |
| trait application | `TraitRef` / `traits.rs` | semantic | associated binding lookup is against exact trait declaration/application |
| behavioral obligation identity | `TraitRequirementId` / `traits.rs` | semantic | remains behavioral only |
| trait contract surface | `TraitSurface` / `traits.rs` | semantic | extended with a separate associated-type table |
| exact conformance candidate | `ConformanceHeadMatch` / `impls.rs` | semantic | reused for exact binding specialization |
| behavioral source plan | `ConformanceWitnessPlan` / `impls.rs` | semantic | retained; associated binding plan is adjacent, not encoded as callable witness selection |
| exact conformance proof | `ConformanceEvidence` / `impls.rs` | semantic | extended with exact associated bindings |
| conformance proof state | `ConformanceCompleteness` / `impls.rs` | semantic | remains single final completeness authority |
| callable/field identity | `CallableId`, `FieldId` | semantic | `via` resolves canonical `FieldId`; synthesized accessors use ordinary callable identity |
| field type/mutability | `FieldSemanticSignature` / `signature.rs` | semantic | canonical source for delegate type and setter legality |
| trait dispatch | `TraitDispatch*` | semantic | does not learn a special property/via path |
| semantic→core boundary | `phalcom-core/src/modules/semantic_lowering.rs` | core projection | receives already-authorized semantic facts only |
| runtime conformance | existing executable conformance plan/environment | core/runtime | no associated-type solving or `via` protocol added |

---

# 7. Architecture

## 7.1 Authority flow

```text
source AST
    ↓
canonical semantic declaration/field/trait/conformance facts
    ↓
behavior/property/delegation elaboration
    ↓
TraitSurface + declaration/inherent/conformance callable products
    ↓
C4 witness/default selection
    +
associated binding source plan
    ↓
exact ConformanceEvidence
    ↓
semantic lowering of already-proven executable behavior
    ↓
compiler
    ↓
ordinary getter/setter bytecode and existing runtime conformance plan
```

The VM must never answer:

```text
Which field does `via` mean?
Which associated declaration does `Item` mean?
Which associated type is bound for this conformance?
Is this conformance complete?
```

Those are semantic questions.

## 7.2 AST/member taxonomy

The current `BehaviorMember` must remain behavior-only. P1 should introduce heterogeneous member categories conceptually equivalent to:

```rust
pub enum TraitMember {
    Behavior(BehaviorMember),
    Property(TraitPropertyRequirement),
    AssociatedType(AssociatedTypeDeclaration),
}

pub enum ImplMember {
    Behavior(BehaviorMember),
    Delegation(DelegatedAccessorDef),
    AssociatedTypeBinding(AssociatedTypeBinding),
}
```

`ClassMember` additionally gains a delegation form or a mechanically equivalent source node so class-body `via` is preserved until semantic/compiler elaboration.

Recommended source structs:

```rust
pub struct TraitPropertyRequirement {
    pub name: String,
    pub name_range: SourceRange,
    pub annotation: TypeAnnotation,
    pub mutable: bool,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}

pub struct AssociatedTypeDeclaration {
    pub name: String,
    pub name_range: SourceRange,
    pub range: SourceRange,
}

pub struct AssociatedTypeBinding {
    pub name: String,
    pub name_range: SourceRange,
    pub value: TypeAnnotation,
    pub range: SourceRange,
}

pub enum DelegatedAccessorKind {
    Getter,
    Setter,
    ReadWrite,
}

pub struct DelegatedAccessorDef {
    pub name: String,
    pub name_range: SourceRange,
    pub kind: DelegatedAccessorKind,
    pub target_field: String,
    pub target_range: SourceRange,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}
```

Exact names are mechanically flexible. The semantic categories are fixed.

## 7.3 Trait property elaboration

For a read-only property:

```phalcom
value: T
```

publish one ordinary requirement:

```text
TraitRequirementId(owner=Trait, selector=value, side=Instance)
CallableSemanticSignature: () -> T
```

For a mutable property:

```phalcom
mut value: T
```

publish:

```text
value getter  : () -> T
value setter  : (_: T) -> Unit
```

Both are ordinary C4 behavioral requirements. Preserve one higher-level source/provenance record for diagnostics/tooling, but do not give the property a conformance identity separate from its generated requirements.

## 7.4 Direct-field `via` elaboration

Resolve the target field through canonical field semantic products:

```text
DelegatedAccessorDef
    ↓ exact target declaration
FieldId { owner=target declaration, name=_field, side=Instance }
    ↓
FieldSemanticSignature
    ├─ declared type
    └─ mutable
```

Then synthesize ordinary callable semantics:

```text
Getter:
    selector: property getter
    return: field declared type
    implementation source: delegate field read

Setter:
    selector: property setter
    parameter: field declared type
    return: Unit
    implementation source: delegate field write
```

No superclass search is allowed.

The delegate field must have a usable explicit declared type. The ratified shorthand intentionally does not repeat the type on the property declaration.

For class/inherent-impl accessors, use the ordinary declaration/inherent callable ownership conventions already established by C2.

Conformance-local `via` is rejected semantically under the T2 consultation
amendment. Do not grant a conformance body target-class private-field access or
publish a delegated conformance witness.

## 7.5 Associated type identity

Add a stable trait-owned identity conceptually equivalent to:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssociatedTypeRequirementId {
    pub owner: DeclarationId,
    pub index: u32,
}
```

The index is source order within the trait's associated-type declaration lane. Same-spelling declarations in different traits are distinct.

Do not use the name as canonical identity. Do not reuse `TraitRequirementId`, `CallableId`, `TypeParameterId`, or a standalone `DeclarationId`.

## 7.6 TraitSurface associated requirements

Extend `TraitSurface` with a separate table:

```rust
pub struct TraitAssociatedTypeRequirement {
    pub requirement: AssociatedTypeRequirementId,
    pub name: Box<str>,
    pub kind: KindId,
    pub source: SemanticSourceSpan,
}

pub struct TraitSurface {
    pub declaration: DeclarationId,
    pub generic_signature: Option<GenericSignature>,
    pub members: BTreeMap<TraitRequirementId, TraitSurfaceMember>,
    pub associated_types: BTreeMap<AssociatedTypeRequirementId, TraitAssociatedTypeRequirement>,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}
```

P1 associated declarations are plain `Type`-kind declarations. Do not add defaults, GAT binders, or trait-bound constraints.

A derived name lookup map is allowed for efficient binding resolution, but identity remains the requirement ID.

## 7.7 Source associated binding plan

Associated type bindings are not callable witnesses. Keep them adjacent to C4 witness planning.

Preferred conceptual product:

```rust
pub struct AssociatedTypeBindingTemplate {
    pub requirement: AssociatedTypeRequirementId,
    pub value_template: TypeId,
    pub source_impl: ImplId,
    pub source: SemanticSourceSpan,
}

pub struct ConformanceAssociatedTypePlan {
    pub impl_id: ImplId,
    pub bindings: BTreeMap<AssociatedTypeRequirementId, AssociatedTypeBindingTemplate>,
    pub failures: Box<[AssociatedTypeBindingFailure]>,
    pub diagnostics: Box<[SemanticDiagnostic]>,
    pub fingerprint: ProductFingerprint,
}
```

The exact container/storage name is flexible. The fixed rules are:

- the plan is source-`ImplId` owned;
- bindings are keyed by canonical associated requirement ID;
- the RHS is formed in the impl-local generic environment;
- missing/duplicate/unknown/invalid bindings are retained as deterministic failures/diagnostics;
- no binding is encoded as `RequirementSelectionTemplate`.

## 7.8 Exact associated binding evidence

Extend exact evidence conceptually with:

```rust
pub struct ExactAssociatedTypeBinding {
    pub requirement: AssociatedTypeRequirementId,
    pub value: TypeId,
    pub source_impl: ImplId,
    pub source: SemanticSourceSpan,
}

pub struct ConformanceEvidence {
    // existing C4 fields remain
    pub associated_types: BTreeMap<AssociatedTypeRequirementId, ExactAssociatedTypeBinding>,
}
```

Materialize each valid source `value_template` through the exact C4 conformance environment. Example:

```phalcom
impl<T> Iterable for List<T> {
  type Item = T
}
```

must yield:

```text
source ImplId: I

Evidence(List<Int>, Iterable)
    source_impl = I
    Item = Int

Evidence(List<String>, Iterable)
    source_impl = I
    Item = String
```

## 7.9 Completeness extension

`ConformanceCompleteness` remains the single final proof state.

The current `Incomplete` payload is behavior-specific because it contains `RequirementFailure`. C5 must generalize that payload rather than invent an independent associated-type completeness result.

Preferred conceptual migration:

```rust
pub enum ConformanceFailure {
    Behavioral(RequirementFailure),
    AssociatedType(AssociatedTypeBindingFailure),
}

pub enum ConformanceCompleteness {
    Complete,
    Incomplete {
        failures: Box<[ConformanceFailure]>,
    },
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}
```

Do not manufacture fake behavioral `TraitRequirementId`s for missing associated bindings. Do not publish `Complete` when the associated plan has any binding failure.

Terminal proof-state precedence must reuse the existing C4 policy. Associated binding formation must not collapse unknown/blocked/dynamic states into ordinary absence.

## 7.10 Incremental authority

Fingerprint layers must separate:

```text
trait declaration shape
    -> TraitSurface fingerprint

conformance associated binding source shape/RHS
    -> ConformanceAssociatedTypePlan fingerprint

exact specialized associated binding values
    -> exact ConformanceEvidence fingerprint

body-only behavior edits
    -> must not spuriously change associated declaration/binding identity
```

Removal/replacement must be owner-complete.

## 7.11 Source tooling

P1 tooling obligations are narrow:

```text
trait `type Item`
    -> canonical definition/source target

conformance `type Item = T`
    -> LHS reference resolves to the trait-owned associated declaration

property/via declaration
    -> preserve source provenance for diagnostics/hover/navigation where existing APIs expose generated callables
```

Do not implement `Self::Item` definition/completion/hover in P1.

## 7.12 Runtime/lowering

`via` compiles as ordinary getter/setter behavior.

There is no:

```text
Via bytecode
Delegate object
Delegate registry
Runtime field-name lookup
Runtime associated-type table lookup
Runtime projection solver
```

Associated type data stays semantic in P1 except where existing executable conformance metadata must carry a fingerprint/identity-safe extension for internal consistency. Do not expose new runtime semantics merely to mirror semantic products.

---

# 8. Ownership boundaries

## 8.1 P1 owns

```text
heterogeneous trait/impl member AST needed for P1
property-shaped trait requirement syntax and elaboration
direct-field `via` syntax and elaboration
explicit `mut` contextual member spelling required by these forms
associated type declaration syntax
associated type binding syntax
AssociatedTypeRequirementId
TraitSurface associated-type table
associated binding source plan
binding validation
combined conformance failure/completeness representation
exact generic associated binding specialization
ConformanceEvidence associated binding map
exact-case preservation
fingerprint/invalidation/removal for new products
source-index identity and binding-LHS navigation
minimal compiler execution support for `via`
focused parser/semantic/core/tooling tests
C5 checkpoint/walkthrough/handoff bookkeeping
```

## 8.2 P1 does not own

```text
Self::Item AST or semantic projection term
projection normalization
projection cycles or ambiguity
T::Item generic-body syntax
<T as Trait>::Item qualification syntax
T: Trait generic constraints
conditional conformance based on trait evidence
nested conformance evidence
associated type defaults
GATs
supertraits
trait inheritance
trait objects/existentials
public trait/conformance reflection
associated-type-based Iterable migration
delegate protocol/delegate objects
subscript delegation
property-chain delegation
arbitrary-place delegation
conformance-introduced storage
trait-owned storage
field inheritance
new runtime trait/associated-type solver
class-header `with Trait` sugar
whole-language visibility syntax migration
removal of legacy unkeyworded mutable-field syntax
```

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| trait property type | trait property AST -> semantic signature | TraitSurface, diagnostics | separate storage type table |
| generated property obligations | `TraitRequirementId` + `TraitSurfaceMember` | C4 conformance | StateRequirementId |
| delegate field identity/type/mutability | `FieldId` + `FieldSemanticSignature` | delegation elaborator | AST rescanning as semantic authority |
| delegated callable identity/signature | ordinary callable/surface products | dispatch, C4 witnesses, compiler | Via-specific dispatch table |
| associated declaration identity | `AssociatedTypeRequirementId` | TraitSurface, bindings, source index | name-only lookup identity |
| source binding template | `ImplId` + associated requirement | exact evidence builder | exact-application-specific source IDs |
| exact associated value | `ConformanceEvidence.associated_types` | future P2 normalization | runtime lookup table / independent cache |
| conformance completeness | `ConformanceCompleteness` | trait dispatch/lowering/tooling | separate associated completeness boolean |
| field privacy/noninheritance | canonical field/object model | delegation resolution | superclass field scan |

---

# 9. Global invariants

`INV-01 — Trait properties are behavioral, never representational.`  
A trait property declaration elaborates only to ordinary getter/setter requirements and never creates storage metadata or target layout.

`INV-02 — TraitRequirementId remains behavioral-only.`  
Associated types receive a distinct semantic identity.

`INV-03 — Associated identity is trait-owned.`  
Same-spelling associated declarations in different traits are never conflated.

`INV-04 — One source conformance remains one ImplId.`  
Exact associated binding specialization never manufactures per-application source identities.

`INV-05 — ConformanceEvidence remains the exact proof authority.`  
Associated exact bindings extend it; no replacement evidence system is introduced.

`INV-06 — Conformance completeness is singular.`  
Behavioral and associated failures converge into the existing completeness proof state.

`INV-07 — `via` is compile-time accessor elaboration.`  
Runtime sees ordinary getter/setter code only.

`INV-08 — Delegate type/mutability comes from canonical field semantics.`  
The delegated property does not repeat or independently infer the field type.

`INV-09 — P1 delegation is own-field only.`  
No inherited field search and no general place/delegate protocol.

`INV-10 — Synthesized members obey ordinary conflict rules.`  
Generated setters/getters never silently override or disappear behind explicit declarations.

`INV-11 — Exact case identity is preserved.`  
Associated binding specialization for an exact enum case retains its `VariantId`/exact target context.

`INV-12 — Runtime is not semantic authority.`  
The VM never resolves `via`, associated binding names, conformance completeness, or projections.

`INV-13 — P1 stops before projection.`  
No `AssociatedTypeProjection` term or `Self::Item` normalization enters this plan.

`INV-14 — P1 stops before generic trait proof.`  
No `T: Trait` generic constraint or nested conformance-evidence machinery enters this plan.

`INV-15 — Incremental replacement is owner-complete.`  
Removing/renaming property, delegate, associated declaration, or binding leaves no stale surface/evidence/source-index product.

`INV-16 — Local parameter names do not affect selector identity.`  
Generated/bodyless setter signatures may use discard syntax without changing callable identity.

`INV-17 — Legacy mutable-field compatibility is not opportunistically broken.`  
P1 adds the needed explicit `mut` surface without using C5 as a whole-language syntax cleanup.

---

# 10. Non-goals

Do not use this plan to implement or redesign:

- projection syntax or projection term identity;
- trait-bound generic constraints;
- associated defaults/GATs;
- conditional conformance;
- trait object representation;
- supertraits;
- reflection descriptors;
- Iterable's final associated-type API;
- field inheritance;
- field/public property unification;
- general delegation protocols;
- arbitrary computed property shorthand;
- new visibility keyword semantics;
- class-header conformance sugar;
- C4 baseline failures unrelated to P1.

---

# 11. Expected impact map

## 11.1 Expected source areas

```text
phalcom-ast/src/ast.rs
    heterogeneous TraitMember/ImplMember and delegation/property/associated nodes

phalcom-ast/src/parser.rs
    contextual mut/via member grammar; trait associated/property members; impl associated/delegation members

phalcom-ast/src/token.rs / lexer.rs
    only if live parser convention requires a token rather than contextual identifiers

phalcom-ast/src/selector.rs
    only if delegation/property nodes need shared selector construction helpers

phalcom-modules/*
    exhaustive AST/interface/fingerprint walkers only; no module-level associated declaration IDs

phalcom-semantic/src/identity.rs
    AssociatedTypeRequirementId

phalcom-semantic/src/traits.rs
    property elaboration; associated requirements; TraitSurface extension

phalcom-semantic/src/signature.rs
    reuse FieldSemanticSignature; no duplicate field-type table

phalcom-semantic/src/impls.rs
    delegation contribution/witness integration; associated plan; exact evidence; completeness generalization

phalcom-semantic/src/trait_dispatch.rs
    mechanical view extension if associated plans are exposed; no dispatch redesign

phalcom-semantic/src/checker/declaration*.rs
    field-backed delegation signature formation and class surface publication

phalcom-semantic/src/types/annotation.rs / environment.rs / substitution.rs
    reuse ordinary RHS type formation and exact materialization; avoid TypeTerm projection edits

phalcom-semantic/src/db/query.rs / fingerprint.rs
    product keys, fingerprints, dependencies, reuse

phalcom-semantic/src/session.rs / snapshot.rs
    publication/removal/lifecycle for associated plans

phalcom-semantic/src/source_index/*
    associated declaration definitions; binding LHS references; generated source provenance

phalcom-core/src/modules/semantic_lowering.rs
    executable projection of semantically validated delegated accessors where required

phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/impl_decl.rs
    compile `via` as ordinary getter/setter behavior or consume equivalent lowered form

phalcom-lsp/*
    only minimal source target/semantic token/definition exhaustiveness required by P1
```

## 11.2 Expected tests

```text
phalcom-ast/tests/trait_syntax.rs                # create if no trait-focused file exists
phalcom-ast/tests/impl_syntax.rs
phalcom-ast/tests/lexer.rs                       # only if tokenization changes

phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/capabilities/associated_types.rs   # preferred new focused module
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/incremental/associated_types.rs    # preferred if module tree supports it
phalcom-semantic/tests/semantic/integration/editor.rs or live equivalent

phalcom-core/tests/core/language/traits_p3_closure.rs
    extend with small P1 executable verticals, or create traits_c5_p1.rs if that keeps ownership clearer
```

## 11.3 Documentation/state

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
this plan
LANG005.C5.P1-walkthrough.md
LANG005.C5.P1-handoff.md
relevant authoritative spec amendments for ratified property/via semantics
```

## 11.4 Unexpected-touch rule

Touch adjacent exhaustive-match consumers when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem that changes identity, ownership, runtime authority, field representation, generic proof semantics, or projection semantics.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

1. `TraitRequirementId` remains behavioral-only.
2. associated types get a distinct trait-owned identity.
3. associated binding identity is source `ImplId` + associated requirement; no independent opaque binding ID is required without a proven consumer.
4. one source generic conformance specializes to multiple exact binding maps under the same `ImplId`.
5. trait property declarations are getter/setter behavioral sugar, not storage requirements.
6. `via` initially targets a directly named own field only.
7. field type/mutability authority is `FieldSemanticSignature` or its canonical live equivalent.
8. delegated setters require writable fields.
9. generated accessors conflict normally with explicit members.
10. fields remain private/non-inherited-visible.
11. `via` works in class and `impl` declarations.
12. `ConformanceEvidence` is extended, not replaced.
13. final completeness combines behavioral + associated failures in one proof state.
14. runtime does not solve delegation or associated types.
15. P1 does not implement projection or generic trait constraints.
16. top-level `type Alias = ...` behavior remains unchanged.
17. P1 accepts the explicit `mut` spelling required by the new member forms without opportunistically removing legacy field syntax.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact AST enum/struct names;
- contextual-keyword helper naming;
- whether `mut`/`via` are parsed contextually or already exist as tokens on the live branch;
- private helper/module decomposition;
- exact associated-plan table owner in snapshot/session;
- exact source-index target enum spelling;
- exact diagnostic code spelling following repository conventions;
- whether source provenance grouping is stored directly on `TraitSurface` or in adjacent semantic/source-index metadata;
- whether core compiles a validated `DelegatedAccessorDef` directly or receives a semantically lowered getter/setter plan, provided semantic authority is preserved;
- test module filenames where the live tree already has a better owner.

## 12.3 VERIFY-FIRST assumptions

| Assumption | Verify at | If false |
|---|---|---|
| fields remain private/non-inherited-visible | `docs/spec/current/classes.md`, field resolver/compiler | STOP if semantics changed; do not add superclass scan casually |
| field semantic signatures expose mutability + declared type | `signature.rs` | adapt mechanically if equivalent canonical product moved |
| conformance-owned bodies can resolve exact target field owner | `checker/body.rs`, C4 conformance body setup | if categorically false for semantic reasons, STOP AND CONSULT before restricting `via` |
| C4 exact conformance environment can materialize impl parameter forms | `impls.rs`, `types/environment.rs` | STOP if a new generic proof mechanism seems required |
| semantic DB has owner-complete module replacement hooks for conformance plans | session/snapshot/query tables | extend mechanically; STOP only if lifecycle authority must move |
| `ConformanceCompleteness` consumers can migrate to a generalized failure enum without semantic redesign | `impls.rs`, dispatch/editor consumers | adapt matches mechanically; STOP if callers rely on behavior-only invariants architecturally |
| P1 tooling can add associated-definition targets without exposing raw `TypeId` as stable identity | source-index/editor APIs | STOP if public identity boundary would need snapshot-local TypeId |

---

# 13. Global STOP / CONSULT triggers

Stop editing and build the repository's Implementation Incident packet if any of these fires:

1. `TraitRequirementId` would need to represent associated types.
2. associated declarations appear to require module-level `DeclarationId`s.
3. correct P1 implementation would require replacing `ConformanceEvidence` rather than extending it.
4. correct P1 implementation would require a second final completeness query/boolean.
5. `Self::Item` or another projection term becomes necessary to implement the minimal P1 source bindings.
6. `T: Trait` proof machinery becomes necessary.
7. exact generic specialization appears to require one `ImplId` per exact application.
8. exact enum-case binding cannot preserve exact-case identity.
9. `via` appears to require changing object layout or field inheritance.
10. `via` appears to require a runtime delegate registry/protocol.
11. core/compiler would need to re-resolve field type/mutability independently from semantic authority.
12. conformance-local `via` cannot access target-owned fields under C4's existing ownership rules and two plausible semantic choices exist.
13. a new visibility model is required to implement property requirements.
14. adding contextual `mut` cannot coexist safely with the live mutable-field grammar without a user-visible compatibility decision.
15. a P1 test can pass only by weakening C4 coherence/witness/default invariants.
16. associated binding RHS formation needs associated projection/default/GAT machinery.
17. a new runtime associated-type lookup path is proposed.
18. an incremental fix would duplicate canonical identity/fingerprint logic in a second layer.
19. the implementation touches a materially unexpected subsystem because ownership was misidentified.
20. current specs, live code, and user-ratified C5 decisions conflict in a way that changes user-visible semantics rather than mechanics.

A trigger cannot be self-waived.

---

# 14. Debugging budget

## Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress:

```text
inspect
identify one concrete cause
make one coherent correction
rerun the smallest discriminating command
```

## Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure. If it persists, STOP AND CONSULT.

## Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless something relevant changed.

---

# 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required evidence |
|---|---|---|---|
| CV-01 | parser/category separation | INV-02, INV-13 | trait associated declaration and top-level alias remain distinct |
| CV-02 | property elaboration | INV-01 | read-only property creates getter only; mutable creates getter+setter |
| CV-03 | delegation success | INV-07, INV-08 | getter/setter/read-write delegate to direct own field |
| CV-04 | delegation mutability | INV-08 | setter/read-write delegation rejects immutable field |
| CV-05 | field ownership boundary | INV-09 | superclass field is not a valid delegate target |
| CV-06 | conflict behavior | INV-10 | generated accessor conflicts with explicit same selector |
| CV-07 | C4 witness convergence | INV-01, INV-07 | property requirement satisfied by explicit and `via` accessors through ordinary witness selection |
| CV-08 | associated identity | INV-02, INV-03 | same `Item` spelling in two traits yields distinct IDs |
| CV-09 | TraitSurface separation | INV-02 | behavioral and associated tables remain separate |
| CV-10 | binding validation | INV-05, INV-06 | duplicate/unknown/missing/inherent binding cases diagnosed |
| CV-11 | generic specialization | INV-04, INV-05 | `List<Int>` and `List<String>` exact evidence differ under one source ImplId |
| CV-12 | exact-case specialization | INV-11 | exact enum-case target stays exact in evidence |
| CV-13 | completeness integration | INV-06 | behavior + associated failures jointly determine final proof state |
| CV-14 | runtime anti-authority | INV-07, INV-12 | executable `via` uses ordinary field accessor bytecode; no runtime resolver |
| CV-15 | source tooling | INV-03 | binding LHS navigates to trait associated declaration |
| CV-16 | incremental declaration lifecycle | INV-15 | add/edit/remove associated declaration removes stale surface identity |
| CV-17 | incremental binding lifecycle | INV-15 | binding RHS/add/remove invalidates exact evidence correctly |
| CV-18 | incremental delegation lifecycle | INV-15 | read-write→read-only removes setter; field type/mutability edits update delegates |
| CV-19 | cold/incremental parity | INV-15 | final semantic products agree after equivalent cold/incremental construction |
| CV-20 | boundary protection | INV-12, INV-13, INV-14 | no projection term, trait-bound constraint, or VM associated solver appears in diff |

---

# 16. Verification execution budget

## BUILD mode

Use only the smallest failing/passing test relevant to the current task.

Preferred ladders:

```text
T0 parser/AST reproducer
    -> owning parser integration target

semantic micro test
    -> one semantic module filter

core executable vertical
    -> one named core test
```

## STABILIZE mode

After coherent semantic slices are complete, run affected module suites and crate checks.

## CERTIFY mode

P1 does not require full workspace/release certification. C4 already records broad baseline blockers. P1 ends at focused-tested unless the user explicitly asks for release certification.

## Verification ladder

```text
V0 — exact new regression / one test
V1 — directly affected module target
V2 — owning crate check / focused integration module
V3 — semantic-core integration vertical
V4 — affected crate suites
V5 — workspace/release (not required by P1)
```

## Explicitly deferred during BUILD

Do not reflexively run:

```text
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
full language corpus
unrelated Universe capability suites
Iterable stress suites
```

Known C4 baseline blockers make these poor debugging discriminators. Broad gates may be sampled at final bookkeeping only for classification, not as a prerequisite to P1 focused acceptance.

---

# 17. Baseline / unrelated failure policy

Use repository workflow classification:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

Known C4 baseline families include:

```text
Universe generic call-entry failures
incremental A7 cold/incremental presentation mismatch
Universe Bool capability failures
Iterable/outgoing-pack generic call-entry failures
pre-existing formatting drift
pre-existing AST Clippy violations
```

A/B are active responsibility. C gets one bounded classification pass. D is recorded and deferred immediately.

Never weaken P1 tests to make a broad baseline gate green.

---

# 18. Tasks

## T0 — Re-ground, establish C5 lifecycle state, and prove C4 takeover

### Purpose

Bind the plan to the actual implementation checkout, preserve unrelated work, establish C5's checkpoint records, and prove the stable C4 seams still exist before source edits.

### Preconditions

- user has selected this P1 plan for implementation;
- repository checkout is available;
- no architectural implementation begins before this task passes.

### Consumes

- live checkout and repository workflow rules;
- C4 checkpoint/handoff state;
- the user-ratified C5.P1 requirements and this plan.

### Produces

```text
starting revision and branch
working-tree inventory
C5 checkpoint/guidance present and plan marked active
verified stable C4 seam map
baseline focused trait/conformance tests recorded
```

### Files and symbols

**Read:** all Section 4 paths relevant to takeover.

**Create/modify if absent:**

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
```

Do not rewrite historical C4 records except to correct an objective factual cross-link if necessary.

### Required implementation shape

This is a takeover/documentation gate, not feature implementation. Establish checkout state, verify stable C4 seams, create/normalize C5 lifecycle records if absent, and record the focused baseline. Do not alter semantic/runtime architecture in T0.

### Steps

- [ ] Run repository hygiene commands and record exact outputs in working notes:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

- [ ] Confirm unrelated modified/staged/untracked work and explicitly exclude it from staging.

- [ ] Read `AGENTS.md`, workflow documents, C4 handoff, live traits spec, field specs, and the live symbols in Section 6.

- [ ] Verify the following code facts directly:

```text
TraitRequirementId remains behavioral
TraitSurface remains behavior-only pre-P1
ConformanceEvidence remains C4 exact proof product
ConformanceCompleteness still owns final proof state
FieldSemanticSignature owns canonical field type/mutability
fields remain own-class/private/non-inherited-visible
```

- [ ] If C5 checkpoint/guidance are absent, create minimal canonical records that state:

```text
C5 objective: associated type declarations/bindings/projection and vertical integration
P1 active: declarations/bindings foundations + ratified property/via enabling semantics
P2 next: projection formation/normalization
P3 later: vertical integration/Iterable/certification
C6 boundary: T: Trait proof and conditional conformance
```

- [ ] Run only the C4-focused takeover checks:

```sh
cargo check -p phalcom-ast -p phalcom-semantic
cargo test -p phalcom-semantic --test semantic capabilities::traits
cargo test -p phalcom-semantic --test semantic impls::queries
cargo test -p phalcom-core --test core language::traits_p3_closure::generic_source_conformance_executes_for_multiple_exact_targets
```

If a filter name drifted, find the live equivalent; do not broaden first.

### Forbidden approaches

- do not repair C4 baseline blockers;
- do not normalize unrelated documentation trees;
- do not stage/rewrite unrelated working-tree changes;
- do not infer live state from historical plans when code/checkpoint records disagree.

### Test changes required

No feature tests are added. Only C5 lifecycle records may change.

### Tests to run now

Run only the focused takeover commands already listed in this task. Do not substitute a broad workspace run.

### Tests explicitly deferred

- all new P1 feature tests;
- workspace tests, Clippy, formatting, and known broad C4 certification blockers.

### Acceptance

- the current checkout and unrelated work are known;
- C5 lifecycle records exist;
- C4 semantic takeover tests pass or failures are classified as known baseline/unrelated;
- no hidden associated-type/property/via implementation contradicts this plan.

### Local STOP / CONSULT triggers

- C4 seam ownership materially differs from Section 6;
- field inheritance/privacy semantics changed;
- C5 checkpoint already contains a conflicting accepted architecture;
- current associated-type implementation exists and materially contradicts this plan.

### Checkpoint update

Record P1 active, starting revision, verified C4 seams, and baseline/unrelated classifications. Do not claim feature implementation progress.

### Commit

Documentation-only C5 lifecycle normalization may be committed separately if repository policy expects it:

```sh
git add docs/implementation/LANG005/LANG005.C5
git commit -m "docs: establish LANG005 C5 checkpoint state"
```

Do not commit unrelated work.

---

## T1 — Normalize P1 member syntax and AST categories

### Purpose

Create lossless source representations for trait property requirements, associated declarations, associated bindings, and direct-field delegation without contaminating `BehaviorMember`.

### Preconditions

- T0 takeover passes;
- C5 checkpoint/guidance identify P1 as active;
- no unresolved parser/member-model architecture conflict remains.

### Consumes

- live parser/member conventions from T0;
- current field syntax and visibility pipeline.

### Produces

- heterogeneous trait/impl member AST;
- class/impl delegation AST;
- contextual `mut`/`via` source support;
- parser-level distinction between top-level aliases and member associated types.

### Files and symbols

**Modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/token.rs / lexer.rs only if contextual parsing is not viable
phalcom-ast/src/selector.rs only for shared selector helpers
all exhaustive AST walkers that must compile
```

**Tests:**

```text
Create: phalcom-ast/tests/trait_syntax.rs   # if no better live owner exists
Modify: phalcom-ast/tests/impl_syntax.rs
Modify: phalcom-ast/tests/parser.rs only for shared recovery regression
Modify: phalcom-ast/tests/lexer.rs only if lexer changes
```

### Required implementation shape

Add mechanically equivalent source categories to Section 7.2.

Grammar obligations:

```phalcom
trait Iterable {
  type Item
}

impl<T> Iterable for List<T> {
  type Item = T
}

trait Counter {
  count: Int
  mut total: Int
}

class Counter {
  mut _count: Int
  count via _count
}

impl Counter {
  count=(_) via _count
}

impl CounterTrait for Counter {
  mut count via _count
}
```

Parse `via` target as one direct field token, not an expression.

Reject parser-level unsupported forms such as:

```phalcom
count via state.count
count via values[index]
count via makeStorage()
trait T { type Item = Int }      // associated defaults are not P1
impl T for X { type Item }       // binding requires RHS
```

Associated binding syntax may parse inside any `impl` into `ImplMember::AssociatedTypeBinding`; T5 gives inherent-impl use a semantic diagnostic. This keeps grammar shared and semantic ownership explicit.

For `mut`, prefer contextual parsing so existing identifiers/legacy fields are not globally disrupted. `mut _field: T` and legacy mutable-field syntax may both parse during P1.

Do not introduce a new visibility grammar.

### Forbidden approaches

- do not encode associated types as fake `BehaviorMember`s/callables;
- do not parse `via` RHS as a general expression;
- do not remove legacy field syntax merely to land P1;
- do not add projection or trait-bound grammar.

### Test changes required

Add exact tests for:

```text
C5P1-AST-01 trait type declaration
C5P1-AST-02 conformance type binding
C5P1-AST-03 top-level type alias remains unchanged
C5P1-AST-04 read-only trait property
C5P1-AST-05 mutable trait property
C5P1-AST-06 getter delegation
C5P1-AST-07 setter delegation
C5P1-AST-08 read/write delegation
C5P1-AST-09 delegation in impl
C5P1-AST-10 non-field delegate syntax rejected
```

### Tests to run now

```sh
cargo test -p phalcom-ast --test trait_syntax
cargo test -p phalcom-ast --test impl_syntax
```

If lexer changes:

```sh
cargo test -p phalcom-ast --test lexer mut
cargo test -p phalcom-ast --test lexer via
```

### Tests explicitly deferred

Do not run semantic/core runtime tests yet. The new AST is intentionally not semantically complete.

### Acceptance

- source nodes preserve every P1 construct and source span;
- `BehaviorMember` stays behavior-only;
- top-level aliases still parse exactly as before;
- unsupported general delegation syntax is rejected;
- crate compiles after exhaustive-match consumers are mechanically migrated.

### Local STOP / CONSULT triggers

- parser cannot support contextual `mut` without changing unrelated expression/name grammar;
- `via` requires expression parsing rather than direct field tokens;
- implementing member categories would require collapsing existing behavior/member semantics.

### Checkpoint update

After G1, record the durable source-member taxonomy and the compatibility decision for contextual `mut`/legacy mutable-field parsing.

### Commit

```sh
git add phalcom-ast phalcom-modules phalcom-semantic phalcom-core phalcom-lsp
git commit -m "lang005: parse associated and delegated trait members"
```

Stage only files actually changed for AST exhaustiveness.

---

## T2 — Establish canonical direct-field `via` semantic elaboration

### Purpose

Resolve each delegation through the target's canonical field signature and publish ordinary getter/setter callable semantics with no runtime delegation model.

### Preconditions

- T1 syntax/AST categories compile and focused parser tests pass;
- `FieldId`/`FieldSignatureTable` remain canonical field authority.

### Consumes

- `DelegatedAccessorDef` or live equivalent from T1;
- `FieldId` / `FieldSignatureTable` / `FieldSemanticSignature`;
- class/inherent/conformance callable ownership rules from C2/C4.

### Produces

- validated delegated getter/setter callable contributions;
- semantic diagnostics for invalid delegate fields/mutability/conflicts;
- a semantic/lowering representation sufficient for T9 execution.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/identity.rs            # only if helper constructors are useful
phalcom-semantic/src/signature.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/surface.rs / dispatch owners as live tree requires
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
```

### Required implementation shape

Create one semantic helper family, conceptually:

```rust
fn resolve_delegated_field(
    field_signatures: &FieldSignatureTable,
    target_owner: &DeclarationId,
    target_name: &str,
    side: DispatchSide,
) -> Result<&FieldSemanticSignature, DelegationError>;

fn delegated_accessor_signatures(
    ...,
    delegation: &DelegatedAccessorDef,
    field: &FieldSemanticSignature,
) -> Result<Vec<CallableSemanticSignature>, DelegationError>;
```

Rules:

1. form the canonical `FieldId` from exact target declaration + delegate field name + instance side;
2. do not query superclass fields;
3. require a usable explicit declared field type for delegation;
4. getter delegation requires readable field existence only;
5. setter/read-write delegation additionally requires `field.mutable == true`;
6. use the delegation's member visibility, not a fictional public field visibility;
7. publish ordinary selectors/signatures;
8. route duplicate selector detection through existing class/inherent/conformance conflict logic;
9. retain source provenance that points back to the `via` declaration;
10. reject conformance-local delegation deterministically; do not publish a
    delegated conformance witness or alter `CallableOwnerId` ownership.

Recommended diagnostics:

```text
delegation.field_not_found
delegation.field_type_required
delegation.setter_requires_mutable_field
delegation.member_conflict
```

Follow live `DiagnosticCode` naming conventions.

Do not analyze a fake synthesized body merely to infer the signature. The field signature already owns type/mutability.

### Forbidden approaches

- no superclass field search;
- no delegate-object/protocol registry;
- no second field-type inference path;
- no runtime `via` product;
- no silent generated-vs-explicit member precedence.

### Test changes required

Add focused semantic cases:

```text
CV-03 own-field getter/setter/read-write success
CV-04 immutable-field setter rejection
CV-05 superclass field not visible to delegation
CV-06 generated/explicit selector collision
read-only delegate + explicit custom setter succeeds
setter-only delegate + explicit custom getter succeeds
class body and inherent impl delegation publish equivalent callable surface
conformance-local delegation is rejected without weakening C4 field/body ownership
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::traits::delegat
cargo test -p phalcom-semantic --test semantic impls::queries::delegat
```

Use the live exact filter names.

### Tests explicitly deferred

Do not execute VM tests yet. T9 owns executable lowering.

### Acceptance

- semantic surfaces contain normal getter/setter callables for every valid delegation;
- invalid field/mutability/conflict cases fail semantically;
- no `Via` runtime product or independent delegate type inference exists.

### Local STOP / CONSULT triggers

- field type would need to be inferred independently from `FieldSemanticSignature`;
- inherited field search appears necessary;
- a class or inherent-impl delegation cannot use the canonical field-signature authority.

### Checkpoint update

After focused delegation tests, record direct-own-field lookup, no inheritance, field mutability/type authority, and ordinary-callable elaboration.

### Commit

```sh
git add phalcom-semantic phalcom-ast/tests phalcom-semantic/tests
git commit -m "lang005: elaborate direct field access delegation"
```

---

## T3 — Elaborate trait property requirements into ordinary C4 obligations

### Purpose

Make property-shaped trait declarations first-class source syntax while preserving C4's behavioral requirement identity and witness engine unchanged.

### Preconditions

- T1 property syntax is available;
- T2 delegated accessors publish ordinary callable semantics;
- C4 behavioral requirement identity remains intact.

### Consumes

- `TraitPropertyRequirement` from T1;
- ordinary signature/selector/visibility builders;
- `TraitSurface` behavioral member publication;
- delegation semantics from T2 for integration tests.

### Produces

- getter and optional setter `TraitRequirementId`s;
- trait-owned callable signatures for those requirements;
- retained property-source grouping/provenance;
- no storage/state requirement identity.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/traits.rs
phalcom-semantic/src/checker/declaration_signature.rs or focused property helper
phalcom-semantic/src/diagnostic.rs if dedicated source-level message family is needed
phalcom-semantic/src/source_index/* only enough to keep source declaration ranges coherent
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/traits.rs
```

### Required implementation shape

Extend `build_trait_surface` or the live equivalent to iterate heterogeneous `TraitMember`.

For property nodes, construct ordinary requirement/callable semantics directly from the property annotation:

```text
`count: Int`
    getter selector: count
    parameters: []
    declared return: Int

`mut count: Int`
    getter as above
    setter selector: count=(_)
    parameters: [Int]
    declared return: Unit
```

The setter's source-level local parameter is irrelevant. Use a synthetic/internal name if the semantic signature struct requires one; it must not enter selector identity or diagnostics as though authored by the user.

Retain one provenance record conceptually like:

```rust
pub struct TraitPropertyRequirementSource {
    pub name: Box<str>,
    pub declared_type: TypeId,
    pub mutable: bool,
    pub getter: TraitRequirementId,
    pub setter: Option<TraitRequirementId>,
    pub source: SemanticSourceSpan,
}
```

This record is presentation/tooling provenance only. It is not a conformance obligation ID.

Conflict examples must reuse `TraitMemberConflict` or a dedicated semantically equivalent diagnostic:

```phalcom
trait T {
  mut value: Int
  value -> String
}
```

must not silently publish two same-selector signatures.

### Forbidden approaches

- no `StateRequirementId`, trait field layout, or storage-binding evidence;
- no property-specific branch in C4 witness selection;
- no target-surface injection;
- source grouping/provenance is not conformance identity.

### Test changes required

Prove:

```text
CV-02 read-only property -> one getter requirement
CV-02 mutable property -> getter + setter requirement
CV-07 explicit getter/setter witnesses satisfy those requirements
CV-07 delegated accessor witnesses satisfy them through ordinary C4 selection
immutable data component may satisfy getter but cannot satisfy mutable property by itself
trait default can call/read/write its own private/property requirement under existing visibility rules
no trait field/storage/instance-layout product appears
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::traits::property
cargo test -p phalcom-semantic --test semantic capabilities::traits::delegat
```

### Tests explicitly deferred

- associated-type declaration/binding tests (T4–T6);
- VM execution of `via` (T9);
- broad workspace suites.

### Acceptance

C4 witness selection sees only ordinary getter/setter requirements and ordinary candidate callables; it requires no property-specific branch.

### Local STOP / CONSULT triggers

- satisfying a property appears to require a new state/storage witness category;
- default body resolution would require injecting members into target surfaces;
- property visibility requires a new visibility model rather than existing member visibility.

### Checkpoint update

Record that trait field-shaped properties elaborate to ordinary behavioral requirements and create no representation/state identity.

### Commit

```sh
git add phalcom-semantic phalcom-semantic/tests
git commit -m "lang005: elaborate trait property requirements"
```

---

## T4 — Add associated type requirement identity and TraitSurface publication

### Purpose

Give `type Item` a stable trait-owned semantic identity and publish it separately from behavioral requirements.

### Preconditions

- T1 associated-type declaration syntax is available;
- TraitSurface publication/fingerprinting is verified from T0.

### Consumes

- `TraitMember::AssociatedType` from T1;
- trait declaration identity/header/generic scope;
- `TraitSurface` lifecycle/fingerprinting.

### Produces

- `AssociatedTypeRequirementId`;
- `TraitAssociatedTypeRequirement`;
- `TraitSurface.associated_types`;
- deterministic duplicate declaration diagnostics;
- source identity suitable for T5 binding resolution and T8 navigation.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/identity.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/lib.rs public re-exports as appropriate
phalcom-semantic/src/db/fingerprint.rs initial TraitSurface hashing
exhaustive presentation/debug helpers
```

**Tests:**

```text
Create/modify phalcom-semantic/tests/semantic/capabilities/associated_types.rs
register module in capabilities/mod.rs and tests/semantic.rs tree as required
```

### Required implementation shape

Implement identity using trait declaration + source-order index. Example:

```rust
AssociatedTypeRequirementId {
    owner: iterable_decl_id,
    index: 0,
}
```

Publish:

```rust
TraitSurface {
    members: ...,
    associated_types: ...,
}
```

Provide deterministic lookup helper by name for source binding resolution, conceptually:

```rust
impl TraitSurface {
    pub fn associated_type_by_name(&self, name: &str)
        -> Option<&TraitAssociatedTypeRequirement>;
}
```

The helper may scan the `BTreeMap` at first; a secondary name index is optional optimization. Name is never identity.

All P1 associated types have `KindId::TYPE` or the live equivalent ordinary type kind.

Reject duplicate associated names in one trait. Do not interpret one as an overload.

### Forbidden approaches

- do not reuse `TraitRequirementId`, `CallableId`, or string name as associated identity;
- do not create a runtime nominal type for the associated declaration;
- do not put associated declarations in the behavioral table;
- do not implement projection normalization.

### Test changes required

```text
CV-08 deterministic associated requirement ID
CV-08 A.Item != B.Item
CV-09 associated requirement not present in behavioral members
CV-09 behavior table unchanged for existing traits
duplicate `type Item` rejected deterministically
TraitSurface fingerprint changes when associated declaration shape changes
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::associated_types::surface
cargo test -p phalcom-semantic --test semantic capabilities::traits
```

### Tests explicitly deferred

- conformance binding/evidence tests (T5–T6);
- source navigation (T8);
- P2 projection.

### Acceptance

`trait T { type Item }` yields one canonical trait-owned associated requirement and no callable/behavior requirement for `Item`.

### Local STOP / CONSULT triggers

- implementation requires a module-level declaration identity per associated type;
- existing TraitSurface consumers assume every requirement is callable in a way that cannot be mechanically extended.

### Checkpoint update

Record `AssociatedTypeRequirementId` and the separate TraitSurface associated-type table as stable P1 seams.

### Commit

```sh
git add phalcom-semantic phalcom-semantic/tests
git commit -m "lang005: publish associated type requirements"
```

---

## T5 — Build source associated-type binding plans for conformances

### Purpose

Resolve `type Item = RHS` against the exact trait surface under impl-local generic scope and retain one source binding plan per `ImplId`.

### Preconditions

- T4 associated requirement identities are published;
- C4 source conformance head and impl-generic scope APIs remain canonical.

### Consumes

- `ImplMember::AssociatedTypeBinding`;
- `ConformanceIndex`/resolved source conformance head;
- `TraitSurface.associated_types`;
- impl generic resolver/type annotation formation;
- `ImplId` provenance.

### Produces

- `AssociatedTypeBindingTemplate`;
- `ConformanceAssociatedTypePlan` or equivalent adjacent product;
- duplicate/unknown/missing/invalid/inherent diagnostics;
- source-plan fingerprint.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/types/annotation.rs only if an existing formation entry point must be exposed/reused
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/trait_dispatch.rs ConformanceSemanticView if the new plan is part of the borrowed semantic view
phalcom-semantic/src/db/query.rs / product keys as required
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/diagnostic.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/associated_types.rs
phalcom-semantic/tests/semantic/impls/queries.rs
```

### Required implementation shape

For each conformance source:

```text
resolved TraitRef template
    ↓ trait declaration
TraitSurface
    ↓ associated name lookup
AssociatedTypeRequirementId
    ↓
resolve RHS TypeAnnotation under impl generic scope
    ↓
canonical TypeId template containing impl parameter forms when generic
    ↓
AssociatedTypeBindingTemplate
```

Example:

```phalcom
impl<T> Iterable for List<T> {
  type Item = T
}
```

must retain `T` as the impl-owned canonical parameter form in the source plan, not eagerly erase it to `Dynamic` or create an exact application.

Validation rules:

```text
duplicate LHS in one conformance -> failure/diagnostic
unknown LHS -> failure/diagnostic naming exact trait
missing required declaration -> failure/diagnostic at conformance head
RHS formation failure -> retain underlying type diagnostic + associated binding failure
associated binding in inherent impl -> semantic error, no plan entry
trait associated default -> unsupported syntax from T1, not interpreted here
```

Do not use target fields/properties when resolving associated binding names. LHS names belong to the trait surface.

### Conformance failure model migration

Introduce `AssociatedTypeBindingFailure` with enough structured identity to render deterministic diagnostics, conceptually:

```rust
pub enum AssociatedTypeBindingFailureKind {
    Missing,
    Duplicate,
    Unknown,
    Invalid,
}

pub struct AssociatedTypeBindingFailure {
    pub requirement: Option<AssociatedTypeRequirementId>,
    pub written_name: Box<str>,
    pub kind: AssociatedTypeBindingFailureKind,
    pub source: SemanticSourceSpan,
}
```

Keep exact details mechanically flexible.

### Forbidden approaches

- do not store bindings as `RequirementSelectionTemplate`;
- do not create an opaque binding ID when `(ImplId, AssociatedTypeRequirementId)` suffices;
- do not resolve LHS by global/target-member name search;
- do not exact-specialize source templates here;
- do not collapse invalid RHS formation to `Dynamic`.

### Test changes required

```text
CV-10 valid binding resolves canonical associated requirement
CV-10 duplicate binding rejected
CV-10 unknown/extra binding rejected
CV-10 missing binding retained as conformance failure
binding in inherent impl rejected
RHS generic parameter resolves to impl-owned canonical type parameter
same binding name in two different traits resolves by exact trait owner, not global name
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::associated_types::binding
cargo test -p phalcom-semantic --test semantic impls::queries::associated
```

### Tests explicitly deferred

- exact generic binding specialization (T6);
- incremental replacement matrix (T7);
- P2 projection.

### Acceptance

Every source conformance has a deterministic associated binding plan/failure set keyed by canonical associated requirement IDs. No exact target application has been materialized yet.

### Local STOP / CONSULT triggers

- resolving a plain RHS like `T` requires projection or trait proof machinery;
- source plan identity appears to require a new opaque binding ID;
- implementation wants to store bindings as callable witness selections.

### Checkpoint update

Record source associated-binding plan ownership/identity and diagnostic families.

### Commit

```sh
git add phalcom-semantic phalcom-semantic/tests
git commit -m "lang005: plan conformance associated type bindings"
```

---

## T6 — Extend exact ConformanceEvidence and unified completeness

### Purpose

Specialize source binding templates under exact conformance environments and make associated binding failures participate in the single canonical completeness state.

### Preconditions

- T5 source associated-binding plans/failures exist;
- C4 exact evidence construction remains canonical.

### Consumes

- C4 `ConformanceHeadMatch` / exact target / exact TraitRef / impl bindings;
- C4 `ConformanceWitnessPlan` and exact evidence builder;
- T5 associated binding source plan.

### Produces

- exact associated binding map in `ConformanceEvidence`;
- generalized conformance failure payload;
- exact generic and exact-case specialization;
- evidence fingerprint including associated values.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/trait_dispatch.rs only for mechanical match/view propagation
phalcom-semantic/src/db/fingerprint.rs
all exhaustive ConformanceCompleteness / RequirementFailure consumers
phalcom-core semantic-lowering structs only if they copy the completeness/failure enum structurally
phalcom-lsp/editor presentation consumers only if exhaustive
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/associated_types.rs
phalcom-semantic/tests/semantic/impls/queries.rs
```

### Required implementation shape

Use the same exact environment already used to instantiate trait requirements/witness signatures. Do not build a new substitution engine.

Conceptually:

```rust
fn specialize_associated_binding(
    store: &mut TypeStore,
    template: &AssociatedTypeBindingTemplate,
    environment: &TypeEnvironment,
) -> ExactAssociatedTypeBinding {
    let value = TypeView::new(template.value_template, environment.clone())
        .materialize(store);
    ...
}
```

Extend evidence:

```rust
ConformanceEvidence {
    source_impl,
    exact_target,
    exact_trait_ref,
    ...existing requirement selections...,
    associated_types,
    completeness,
    ...
}
```

Generalize incomplete failure payload to include both behavior and associated failures. Preserve every non-incomplete terminal state and its existing precedence rules.

The exact binding map may contain only successfully materialized bindings. Missing/invalid ones are represented in the failure set; a conformance with any such failure is not `Complete`.

### Required generic test

Use one source conformance and assert one source `ImplId`:

```phalcom
trait Iterable {
  type Item
}

class List<T> {}

impl<T> Iterable for List<T> {
  type Item = T
}
```

Then build/query exact evidence for both `List<Int>` and `List<String>` and assert:

```text
same source_impl
exact targets differ
exact associated Item values differ: Int vs String
exact TraitRef is preserved
```

### Required exact-case test

Use an exact enum-case conformance with a binding whose RHS is valid in that case environment and assert the evidence target remains exact; never coerce it back to the enum root declaration.

### Forbidden approaches

- no second completeness engine;
- no fake behavioral failure IDs for associated failures;
- no conformance-head re-matching during specialization;
- no exact-case erasure;
- no VM associated-type solver.

### Test changes required

```text
CV-11 generic exact specialization
CV-12 exact-case preservation
CV-13 missing associated binding prevents Complete
CV-13 behavioral failure + associated failure coexist deterministically
existing C4 complete/incomplete/default/witness tests remain semantically unchanged when no associated requirements exist
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::associated_types::exact
cargo test -p phalcom-semantic --test semantic impls::queries::conformance
cargo test -p phalcom-semantic --test semantic capabilities::traits
```

### Tests explicitly deferred

- P2 projection/normalization;
- C7 public reflection;
- broad workspace/release certification.

### Acceptance

Exact `ConformanceEvidence` is the single place where the fully specialized associated binding map and final combined completeness meet.

### Local STOP / CONSULT triggers

- exact specialization needs a second target/trait matching engine;
- `ConformanceCompleteness` cannot be generalized without changing its semantic authority;
- exact case identity would be lost.

### Checkpoint update

After G4, record exact associated binding evidence and the generalized conformance failure/completeness seam.

### Commit

```sh
git add phalcom-semantic phalcom-core phalcom-lsp
git commit -m "lang005: add associated bindings to conformance evidence"
```

Stage only mechanically affected downstream consumers.

---

## T7 — Complete fingerprints, dependencies, replacement, and cold/incremental parity

### Purpose

Make every new P1 semantic product participate correctly in the existing incremental DB/session ownership model.

### Preconditions

- T2–T6 semantic products exist and pass their focused correctness tests;
- their intended dependency ownership is known.

### Consumes

- T2 delegation semantic products;
- T3 trait property behavior expansion;
- T4 associated TraitSurface entries;
- T5 binding plans;
- T6 exact evidence.

### Produces

- source/product fingerprints at correct granularity;
- dependency edges;
- owner-complete module replacement/removal;
- cold/incremental equivalence tests.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
semantic product/dependency key definitions
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
module semantic shard/publication structures if new member AST shapes participate in structural fingerprints
```

**Tests:**

```text
Create or modify phalcom-semantic/tests/semantic/incremental/associated_types.rs
existing incremental DB/session tests as direct owners require
```

### Required implementation shape

Extend the existing semantic DB/query ownership model rather than adding a side cache. Every new source-plan/evidence product must have explicit product/dependency ownership or be deterministically owned by an existing canonical C4 product, with owner-complete removal and cold/incremental semantic parity.

### Forbidden approaches

- no whole-workspace invalidation for local edits;
- no stale-by-name cleanup;
- no irrelevant body contents in TraitSurface fingerprints;
- no incremental-only semantic solver.

### Required fingerprint rules

1. adding/removing/renaming `type Item` changes TraitSurface input/product fingerprint;
2. editing a trait default body without changing contract shape does not spuriously change associated declaration identity;
3. changing `type Item = T` to `type Item = Box<T>` changes associated binding plan and exact evidence fingerprints;
4. binding RHS edits do not invalidate unrelated trait surfaces;
5. changing `_count: Int` to `_count: BigInt` invalidates delegated accessor signatures and dependent conformance evidence;
6. changing delegate field mutability invalidates setter/read-write delegation legality;
7. changing `mut count via _count` to `count via _count` removes only the synthesized setter contribution;
8. module replacement/removal removes stale associated plans/evidence/source targets;
9. exact evidence fingerprint includes exact associated binding values and exact target/TraitRef identity.

### Required incremental scenarios

```text
cold build with associated declaration + binding
incremental edit RHS
incremental remove binding
incremental add binding back
incremental rename associated declaration + binding
incremental delete source conformance
incremental change delegate field type
incremental read-write -> getter-only delegation
cold rebuild of final source
compare canonical products/presentation required by the test harness
```

### Test changes required

Add the incremental scenarios already listed in this task under the live `tests/semantic/incremental/` owner, reusing the existing cold/incremental harness rather than creating a second session fixture.

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic incremental::associated_types
```

If the new tests live in existing DB/session modules, run the exact filters instead.

Then:

```sh
cargo test -p phalcom-semantic --test semantic incremental::db
```

only if the new query/product keys touch DB reuse logic directly.

### Tests explicitly deferred

- editor/LSP projection (T8);
- executable `via` runtime tests (T9);
- broad workspace/release gates.

### Acceptance

No stale associated requirement, binding, delegated accessor, or exact evidence survives owner replacement/removal; cold and incremental final products agree for the P1 scenarios.

### Local STOP / CONSULT triggers

- invalidation requires a second ownership registry parallel to existing session/snapshot authority;
- fixing P1 requires resolving the known unrelated A7 baseline mismatch globally;
- exact evidence reuse cannot distinguish RHS changes without over-invalidating the whole workspace.

### Checkpoint update

After G5, record fingerprint/dependency ownership, owner-complete removal rules, and cold/incremental evidence.

### Commit

```sh
git add phalcom-semantic phalcom-semantic/tests
git commit -m "lang005: invalidate associated type and delegation products"
```

---

## T8 — Add source-index and minimal editor/LSP projection

### Purpose

Make P1 identities navigable without implementing P2 projection UX.

### Preconditions

- T4/T5 canonical associated identities/source spans exist;
- T7 replacement/removal lifecycle is stable enough to prevent stale source targets.

### Consumes

- `AssociatedTypeRequirementId` and requirement source from T4;
- binding resolution from T5;
- property/delegation provenance from T2/T3.

### Produces

- source definition target for associated declaration;
- binding-LHS reference targeting that definition;
- exhaustive semantic-token/presentation support for new source nodes;
- no projection completion/hover semantics.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/source_index/*
phalcom-semantic/src/editor/* live definition/navigation adapter
phalcom-semantic/src/presentation/* only if exhaustive target rendering requires it
phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp definition/hover adapters only as required for associated declaration/binding LHS
```

**Tests:**

```text
phalcom-semantic/tests/semantic/integration/editor.rs or live source-index test owner
phalcom-lsp focused definition/semantic-token tests if existing
```

### Required implementation shape

Create a canonical semantic target form conceptually equivalent to:

```rust
AssociatedType(AssociatedTypeRequirementId)
```

or extend the live target enum in the same identity-preserving manner.

Index:

```phalcom
trait Iterable {
  type Item
}
```

as a definition target.

Index:

```phalcom
impl Iterable for Foo {
  type Item = Int
}
```

`Item` on the binding LHS as a reference to that exact trait-owned target.

Do not expose the RHS `TypeId` as public/stable identity.

For property/delegation nodes, ensure existing generated callable/source occurrence logic does not crash or duplicate navigation entries. Higher-level provenance may be used for hover/diagnostics, but no new public property descriptor is required.

### Forbidden approaches

- no name-only associated definition lookup;
- no independent LSP associated-type resolver;
- no P2 projection UX;
- no C7 reflection work.

### Test changes required

```text
CV-15 associated declaration has source definition
CV-15 binding LHS definition jumps to associated declaration
same spelling in two traits navigates to correct owner
semantic-token traversal handles TraitMember/ImplMember/ClassMember variants
no Self::Item projection completion/definition is added
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic integration::editor associated
```

If LSP tests are touched:

```sh
cargo test -p phalcom-lsp associated
```

### Tests explicitly deferred

- `Self::Item` navigation/normalization (P2);
- public reflection APIs (C7);
- broad editor/LSP suites unrelated to these source targets.

### Acceptance

P1 declaration/binding source identity is usable by editor tooling, and all new AST variants are traversed without tool-specific semantic re-solving.

### Local STOP / CONSULT triggers

- navigation requires a name-only global associated-type search;
- public source target would need to embed snapshot-local `TypeId`;
- tooling attempts to solve conformance independently.

### Checkpoint update

Record the canonical source target/navigation seam for associated declarations and binding LHS references.

### Commit

```sh
git add phalcom-semantic phalcom-lsp
git commit -m "lang005: index associated type declarations and bindings"
```

---

## T9 — Lower and execute `via` as ordinary accessors

### Purpose

Complete the minimal executable delegation vertical while preserving semantic authority and avoiding new runtime machinery.

### Preconditions

- T2 canonical delegated callable semantics exist;
- T7 invalidation is stable;
- ordinary accessor lowering paths are understood.

### Consumes

- validated delegation semantic facts from T2;
- class/inherent/conformance member compilation architecture;
- field load/store bytecodes and existing getter/setter compiler paths.

### Produces

- executable delegated getters/setters in class and impl contexts;
- no new bytecode or VM lookup protocol;
- runtime tests proving trait property requirements can be satisfied by delegated accessors.

### Files and symbols

**Modify as required by live architecture:**

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/impl_decl.rs
shared AST-to-behavior synthesis helper if one canonical helper is needed
```

Do not modify VM dispatch/heap/class layout unless a compile error reveals a purely mechanical exhaustive match; a semantic runtime redesign is forbidden.

**Tests:**

```text
phalcom-core/tests/core/language/traits_p3_closure.rs
or create phalcom-core/tests/core/language/traits_c5_p1.rs and register it in the existing core test module tree
```

### Required implementation shape

The executable semantics are exactly equivalent to handwritten accessors:

```phalcom
count via _count
```

behaves as:

```phalcom
count -> FieldType {
  _count
}
```

and:

```phalcom
count=(_) via _count
```

behaves as:

```phalcom
count=(_ syntheticValue: FieldType) -> () {
  _count = syntheticValue
}
```

`mut count via _count` emits both.

The compiler may materialize synthetic ordinary AST getter/setter nodes or compile a lowered delegation plan directly. The field selection and legality must already be semantically validated; core must not build a competing type/mutability resolver.

No new bytecode is permitted. Reuse ordinary field load/store and method installation.

### Required executable verticals

1. **getter/read-write class delegation**

```phalcom
class Counter {
  mut _count: Int
  mut count via _count

  @constructor
  new(_ value: Int) { _count = value }
}

let c = Counter.new(1)
c.count = 4
let result = c.count
```

assert `result == 4`.

2. **custom setter composition**

```phalcom
class Counter {
  mut _count: Int
  count via _count
}

impl Counter {
  count=(_ value: Int) {
    _count = value + 1
  }
}
```

assert getter is delegated and custom setter executes.

3. **trait property + via witness + default**

```phalcom
trait CounterTrait {
  mut count: Int
  increment { count++ }
}

class Counter {
  mut _count: Int
  mut count via _count
}

impl CounterTrait for Counter {}
```

or, if C4 requires explicit members only for the selected surface shape, use the exact live conformance spelling that lets the inherent delegated members serve as witnesses. Invoke `increment` and assert the field-backed state changed.

4. **conformance-local via rejection**

Conformance-local `via` is intentionally rejected by the T2 consultation
amendment because conformance-owned bodies do not have target-class private
field authority. Test the deterministic semantic diagnostic instead:

```phalcom
impl CounterTrait for Counter {
  mut count via _count
}
```

assert the conformance-owned delegated witness is rejected without changing
the C4 detached-witness ownership model.

### Forbidden approaches

- no `Via` bytecode/opcode or runtime delegation table;
- no runtime field-name lookup;
- no compiler-side delegated-field re-typechecking;
- no member installation outside existing lowering ownership.

### Test changes required

Add focused core regressions for getter-only, setter-only, read/write, custom-accessor composition, and one trait conformance whose selected witness came from `via`. Assert executable behavior and reuse existing runtime-authority checks where cheap.

### Tests to run now

```sh
cargo test -p phalcom-core --test core language::traits_c5_p1
```

or exact named filters in `traits_p3_closure` if extending that file.

Then run one existing C4 regression:

```sh
cargo test -p phalcom-core --test core language::traits_p3_closure::generic_source_conformance_executes_for_multiple_exact_targets
```

### Tests explicitly deferred

- projection execution (P2/P3);
- delegate protocol/subscript delegation;
- broad language-corpus/workspace runtime suites.

### Acceptance

Delegation executes using ordinary accessor/field machinery and C4 trait evidence; runtime contains no `via`/associated-type semantic solver.

### Local STOP / CONSULT triggers

- a new bytecode/runtime delegate concept seems necessary;
- core must infer field type/mutability independently;
- conformance-local delegate execution would require injecting conformance methods into class dictionaries contrary to C4.

### Checkpoint update

After G6, record that `via` executes through ordinary accessor lowering/bytecode with no runtime delegation model.

### Commit

```sh
git add phalcom-core phalcom-semantic phalcom-core/tests
git commit -m "lang005: lower direct field accessor delegation"
```

---

## T10 — Close the integrated P1 diagnostic and interaction matrix

### Purpose

Prove the combined feature behaves coherently at boundaries and that P1 has not disturbed C4 semantics.

### Preconditions

- T2–T9 slices pass their focused gates;
- no unresolved RED consultation remains.

### Consumes

All T1–T9 products.

### Produces

- deterministic negative corpus;
- interaction coverage across property/via/associated binding/C4 evidence;
- proof that no runtime/storage/projection scope leaked into P1.

### Files and symbols

**Tests primarily:**

```text
phalcom-semantic/tests/semantic/capabilities/associated_types.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-core/tests/core/language/traits_c5_p1.rs or traits_p3_closure.rs
semantic diagnostic golden owner if the repository uses one for these codes
```

### Required implementation shape

Close only missing diagnostics/interactions exposed by the matrices in this task. Reuse canonical parser/semantic/conformance products; diagnostic closure must not introduce a second resolver, identity model, or feature architecture.

### Forbidden approaches

- no assertion weakening;
- no diagnostic-only alternate resolver;
- no name-based associated identity shortcut;
- no new syntax/projection behavior during matrix closure.

### Required negative matrix

Add deterministic cases for:

```text
1. duplicate trait associated declaration
2. duplicate conformance associated binding
3. unknown/extra associated binding
4. missing associated binding
5. associated binding in inherent impl
6. invalid RHS type annotation
7. duplicate property-generated getter vs explicit getter
8. duplicate property-generated setter vs explicit setter
9. missing delegate field
10. delegate field has no acceptable explicit type
11. delegated setter targets immutable field
12. delegate tries to use superclass field
13. delegated getter conflicts with explicit getter
14. delegated setter conflicts with explicit setter
15. general expression after `via` rejected by parser
```

### Required interaction matrix

Prove:

```text
property requirement + explicit getter witness
property requirement + data component getter witness
mutable property + explicit getter/setter witness
mutable property + via read/write witness
trait default calls property requirement satisfied by via
trait with both behavior and associated type becomes complete only when both dimensions are satisfied
generic source conformance selects existing C4 behavior witness and exact-specializes Item independently
same target can conform to two traits that both spell Item without identity collision
existing trait with no associated declarations retains identical C4 complete evidence behavior
```

### Test changes required

Represent each negative/interaction matrix row as a focused regression in its existing owning harness. Extend P1 modules rather than creating a broad new test binary.

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic capabilities::associated_types
cargo test -p phalcom-semantic --test semantic capabilities::traits
cargo test -p phalcom-semantic --test semantic impls::queries
cargo test -p phalcom-core --test core language::traits_c5_p1
```

Use live equivalent module/filter if test organization differs.

### Tests explicitly deferred

- P2 projection cases;
- C6 trait-bound/conditional-conformance cases;
- known baseline-blocked broad workspace gates.

### Acceptance

All P1 success and negative interaction cases are deterministic and use canonical diagnostic/semantic products rather than test-only shortcuts.

### Local STOP / CONSULT triggers

- a failing matrix case suggests property/via needs a second witness-selection path;
- associated binding completeness and behavior completeness disagree between queries;
- passing a case requires runtime/type-projection scope expansion.

### Checkpoint update

Record the completed negative/interaction matrix and residual baseline failures; do not mark C5 complete because P2/P3 remain.

### Commit

```sh
git add phalcom-ast phalcom-semantic phalcom-core phalcom-lsp
git commit -m "test: close C5 P1 associated type interaction matrix"
```

---

## T11 — Stabilize affected crates, update specs/state, and produce completion records

### Purpose

Perform bounded stabilization, synchronize ratified specification text, and leave C5 durable state ready for P2.

### Preconditions

- G1–G6 pass or any non-pass is classified as nonblocking;
- all P1 production tasks are implemented;
- no open RED consultation remains.

### Consumes

All P1 implementation and tests.

### Produces

- affected-crate compile/test evidence;
- authoritative spec text aligned with ratified property/via semantics;
- current C5 checkpoint ledger;
- P1 walkthrough;
- P2 handoff.

### Files and symbols

**Modify documentation:**

```text
docs/specs/objects/traits.md
relevant canonical field/property/accessor spec if one exists
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
```

**Create:**

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P1-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P1-handoff.md
```

### Required implementation shape

Perform stabilization and durable record synchronization only. Production edits are limited to A/B defects exposed by the named focused gates; any newly discovered architecture triggers consultation rather than opportunistic expansion.

### Spec synchronization requirements

Update the trait spec to make explicit:

```text
associated type declaration/binding P1 semantics
trait property requirements elaborate to getter/setter behavior
trait properties do not imply storage
`via` is direct-field accessor delegation only in its first version
fields remain separately declared type/mutability authority
setter delegation requires mutable field
no inherited field delegation
ordinary selector conflicts apply
projection remains a later C5 concern
```

Do not add future delegate protocols or unratified projection syntax.

### Test changes required

Only add/adjust a test if final stabilization finds an A/B regression not already covered by T1–T10. Planned P1 coverage should already exist before this task.

### Tests to run now

The final focused stabilization commands are:

Run affected crate checks first:

```sh
cargo check -p phalcom-ast -p phalcom-modules -p phalcom-semantic -p phalcom-core -p phalcom-lsp
```

Run focused suites:

```sh
cargo test -p phalcom-ast --test trait_syntax
cargo test -p phalcom-ast --test impl_syntax
cargo test -p phalcom-semantic --test semantic capabilities::associated_types
cargo test -p phalcom-semantic --test semantic capabilities::traits
cargo test -p phalcom-semantic --test semantic impls::queries
cargo test -p phalcom-semantic --test semantic incremental::associated_types
cargo test -p phalcom-core --test core language::traits_c5_p1
```

Run only touched LSP module filters if LSP production code changed.

Do not automatically run workspace-wide tests/clippy/fmt. If project policy requires one broad comparison at plan end, run it once, classify against C4 baseline, and record the result without repairing unrelated D-class failures.

### Final anti-scope audit

Search the diff for forbidden architecture:

```sh
git diff --stat
git diff -- phalcom-semantic/src/types/parameter.rs phalcom-core/src/vm.rs
```

Then inspect for accidental introduction of concepts equivalent to:

```text
AssociatedTypeProjection
TraitConformance generic constraint
runtime associated type resolver
Via bytecode
StateRequirementId
trait field storage/layout mutation
```

A mechanical exhaustive-match edit is acceptable; semantic additions are not.

### Local STOP / CONSULT triggers

- final stabilization exposes a new semantic-authority/identity defect outside P1;
- truthful completion would require weakening an invariant or absorbing P2/C6 work;
- plan/checkpoint/spec authorities disagree about user-visible P1 semantics.

### Checkpoint update

Record:

```text
P1 status/completion/verification
stable AssociatedTypeRequirementId semantics
TraitSurface associated table
associated source plan + exact evidence extension
unified conformance failure/completeness model
delegation/property elaboration invariants
focused verification gates
known baseline failures left untouched
next plan: C5.P2 projection formation and normalization
```

### Walkthrough requirements

The walkthrough must record:

- final architecture actually landed;
- exact AST member shapes used;
- exact associated plan/evidence structs used;
- how completeness was generalized;
- how `via` is lowered;
- any mechanical drift from this plan;
- tests added and actually run;
- baseline failures observed/deferred;
- consultation decisions;
- final verification classification.

### Handoff requirements

The P2 handoff must explicitly freeze:

```text
AssociatedTypeRequirementId
TraitSurface.associated_types
source binding plan owner + value template representation
ConformanceEvidence exact associated binding map
combined completeness/failure representation
source-index target for associated declarations
property/via semantics as non-projection enabling behavior
```

It must tell P2 **not** to redesign these while adding `Self::Item` formation/normalization.

### Forbidden approaches

- do not redesign P1 during stabilization/docs;
- do not repair unrelated C4/workspace baseline failures;
- do not claim release certification from focused tests;
- do not omit checkpoint/walkthrough/handoff truth updates.

### Tests explicitly deferred

- known baseline-blocked workspace tests/Clippy/formatting unless separately cleared by project policy;
- P2/P3/C6 suites;
- release certification.

### Acceptance

- all required focused gates pass;
- affected crates compile;
- checkpoint records actual implementation truth;
- walkthrough and handoff exist;
- P1 can truthfully be marked `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` if no active P1 failure remains;
- broad baseline blockers do not falsely downgrade a successful focused P1 implementation to implementation failure.

### Commit

```sh
git add docs/specs docs/implementation/LANG005/LANG005.C5 phalcom-ast phalcom-modules phalcom-semantic phalcom-core phalcom-lsp
git commit -m "docs: record C5 P1 associated type foundations"
```

Stage only P1-authorized paths actually changed.

---

# 19. Verification gates

## G1 — Source grammar and member taxonomy gate

Purpose: prove AST categories are lossless and do not break top-level aliases or existing impl/member grammar.

Run:

```sh
cargo test -p phalcom-ast --test trait_syntax
cargo test -p phalcom-ast --test impl_syntax
```

Expected: PASS.

Do not broaden after PASS.

---

## G2 — Property/delegation semantic gate

Purpose: prove property requirements and direct-field delegation converge into ordinary callable/requirement products.

Run:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::traits::property
cargo test -p phalcom-semantic --test semantic capabilities::traits::delegat
cargo test -p phalcom-semantic --test semantic impls::queries::delegat
```

Expected: PASS with no special property/via witness resolver.

---

## G3 — Associated surface/binding gate

Purpose: prove associated identity, TraitSurface publication, binding resolution, and failure diagnostics.

Run:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::associated_types::surface
cargo test -p phalcom-semantic --test semantic capabilities::associated_types::binding
```

Expected: PASS.

---

## G4 — Exact evidence/completeness gate

Purpose: prove one source generic conformance exact-specializes associated bindings and completeness joins both requirement dimensions.

Run:

```sh
cargo test -p phalcom-semantic --test semantic capabilities::associated_types::exact
cargo test -p phalcom-semantic --test semantic impls::queries::conformance
```

Expected: PASS; same source `ImplId`, different exact Item values.

---

## G5 — Incremental lifecycle gate

Purpose: prove owner-complete invalidation/replacement and cold/incremental parity for P1 products.

Run:

```sh
cargo test -p phalcom-semantic --test semantic incremental::associated_types
```

Expected: PASS.

---

## G6 — Executable delegation/C4 convergence gate

Purpose: prove `via` executes as ordinary accessors and can satisfy property requirements through existing C4 evidence.

Run:

```sh
cargo test -p phalcom-core --test core language::traits_c5_p1
cargo test -p phalcom-core --test core language::traits_p3_closure::generic_source_conformance_executes_for_multiple_exact_targets
```

Expected: PASS; no new runtime delegation/conformance solver.

---

## G7 — Final focused stabilization gate

Run:

```sh
cargo check -p phalcom-ast -p phalcom-modules -p phalcom-semantic -p phalcom-core -p phalcom-lsp
cargo test -p phalcom-ast --test trait_syntax
cargo test -p phalcom-ast --test impl_syntax
cargo test -p phalcom-semantic --test semantic capabilities::associated_types
cargo test -p phalcom-semantic --test semantic capabilities::traits
cargo test -p phalcom-semantic --test semantic impls::queries
cargo test -p phalcom-semantic --test semantic incremental::associated_types
cargo test -p phalcom-core --test core language::traits_c5_p1
```

Expected: all P1-focused checks PASS. If an exact module filename/filter differs, use the live equivalent and record it in the walkthrough.

Do not automatically continue to workspace certification after PASS.

---

# 20. Final focused acceptance mapping

At completion, the walkthrough must map evidence like:

| Coverage | Concrete test owner | Required result |
|---|---|---|
| CV-01 | AST trait/impl syntax tests | PASS |
| CV-02 | semantic trait property tests | PASS |
| CV-03 | semantic + core delegation tests | PASS |
| CV-04 | immutable-field negative | PASS |
| CV-05 | inherited-field rejection | PASS |
| CV-06 | conflict negatives | PASS |
| CV-07 | C4 witness/property/via integration | PASS |
| CV-08 | associated identity tests | PASS |
| CV-09 | TraitSurface table separation | PASS |
| CV-10 | binding diagnostics matrix | PASS |
| CV-11 | generic exact evidence | PASS |
| CV-12 | exact-case evidence | PASS |
| CV-13 | combined completeness | PASS |
| CV-14 | executable ordinary lowering | PASS |
| CV-15 | source definition/binding navigation | PASS |
| CV-16 | associated declaration invalidation | PASS |
| CV-17 | binding/evidence invalidation | PASS |
| CV-18 | delegation invalidation | PASS |
| CV-19 | cold/incremental equivalence | PASS |
| CV-20 | anti-scope audit | PASS by diff inspection + affected tests |

---

# 21. Performance/resource evidence

P1 is not a performance checkpoint. Do not add benchmark ceremony.

Nevertheless preserve these complexity constraints:

```text
associated binding LHS lookup may use a bounded trait-surface scan or name index; never workspace-global search
exact specialization reuses existing TypeEnvironment/TypeView machinery
delegation field lookup is direct canonical FieldId lookup; never superclass/workspace scan
no per-instance associated/delegate metadata
no runtime map lookup added to ordinary getter/setter execution
```

If implementation accidentally adds workspace scans or runtime metadata, treat that as an architectural defect rather than something to benchmark away.

---

# 22. Checkpoint bookkeeping

## At plan start

- ensure C5 checkpoint/guidance exist;
- mark P1 `IN_PROGRESS`;
- record starting revision;
- record C4 broad baseline blockers as inherited, not P1 failures.

## During plan

Update checkpoint only for:

- implementation of a durable new identity/interface;
- consultation/amendment;
- meaningful deferred failure;
- coherent gate completion.

Recommended durable updates:

```text
T3: property requirements confirmed behavioral-only
T4: AssociatedTypeRequirementId + TraitSurface table established
T6: exact associated bindings + unified completeness established
T7: incremental ownership/fingerprint closure established
T9: via runtime anti-authority executable closure established
```

## At plan completion

- update P1 ledger row;
- record focused gate results;
- record final stable interfaces/invariants;
- record any baseline blockers observed;
- record consultations/amendments;
- set active/next action to C5.P2.

---

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P1-walkthrough.md
```

Required contents:

1. plan identity and final lifecycle state;
2. objective/outcome;
3. final AST/member architecture;
4. property/via elaboration architecture;
5. associated identity/surface architecture;
6. associated source plan/exact evidence architecture;
7. completeness/failure model migration;
8. incremental/source tooling changes;
9. lowering/runtime behavior;
10. important code paths changed;
11. deviations from the plan;
12. consultations and decisions;
13. tests added;
14. tests actually executed;
15. tests intentionally deferred;
16. residual/baseline failures;
17. final verification classification;
18. P2 prerequisites established.

---

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P1-handoff.md
```

Required contents:

```text
current revision/state
stable AssociatedTypeRequirementId shape/semantics
TraitSurface associated requirement API
associated binding plan API
exact ConformanceEvidence binding map
unified ConformanceFailure/ConformanceCompleteness API
property/via semantics and source AST forms
source-index associated target
fingerprint/invalidation rules
known baseline failures
must-read implementation files
C5.P2 objective: Self::Item projection formation + normalization
first recommended P2 commands/tests
explicit do-not-redesign list
```

P2 must inherit P1 binding/evidence products rather than recompute associated bindings from syntax.

---

# 25. Completion truth table

Do not conflate:

```text
parser accepts syntax
semantic products exist
focused semantic tests pass
runtime via tests pass
incremental parity passes
affected crates compile
P1 complete
C5 checkpoint complete
workspace release certified
```

Expected successful P1 final metadata is normally:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

C5 itself remains `IN_PROGRESS` for P2/P3.

C4/C5 broad baseline blockers do not become `RELEASE_COMPLETE` evidence by omission.

---

# 26. Plan self-review checklist

The architect must verify before handing this plan to an implementer:

- [x] associated type identity has one canonical owner;
- [x] behavioral `TraitRequirementId` is not overloaded;
- [x] property requirements lower to ordinary behavior only;
- [x] direct-field `via` has no runtime protocol;
- [x] field type/mutability authority is canonical semantic field data;
- [x] field inheritance is not introduced;
- [x] generated accessor conflicts are ordinary conflicts;
- [x] associated source plan and exact evidence are separate levels;
- [x] one source `ImplId` survives exact specialization;
- [x] final completeness is singular;
- [x] exact enum-case identity is protected;
- [x] incremental replacement/removal is explicitly planned;
- [x] source identity/navigation is explicitly planned;
- [x] P2 projection work is excluded;
- [x] C6 trait-proof work is excluded;
- [x] runtime associated-type solving is excluded;
- [x] testing is narrow during BUILD and broad only at named gates;
- [x] C4 baseline blockers are classified rather than opportunistically repaired;
- [x] checkpoint/walkthrough/handoff obligations are included;
- [x] no material design fork is left to the implementer.

---

# 27. Dependency order summary

```text
T0 re-ground/lifecycle
    ↓
T1 AST/parser categories
    ↓
T2 direct-field delegation semantics
    ↓
T3 trait property requirement elaboration
    ↓
T4 associated identity + TraitSurface
    ↓
T5 source associated binding plan
    ↓
T6 exact evidence + unified completeness
    ↓
T7 fingerprints/invalidation
    ↓
T8 source index/tooling
    ↓
T9 executable via lowering
    ↓
T10 interaction/diagnostic closure
    ↓
T11 focused stabilization + records
    ↓
C5.P2 projection formation/normalization
```

T2/T3 and T4 are semantically independent after T1, but execute them in the listed order to keep the trait-property enabling slice settled before the associated-type conformance evidence structures broaden C4 APIs.

---

# 28. Final implementation boundary

After P1, the repository should support all of the following as canonical semantic facts:

```phalcom
trait Counter {
  mut count: Int
}

class CounterImpl {
  mut _count: Int
  mut count via _count
}

impl Counter for CounterImpl {}
```

with `count`/`count=(_)` satisfied through ordinary C4 requirements/witnesses, and:

```phalcom
trait Iterable {
  type Item
}

impl<T> Iterable for List<T> {
  type Item = T
}
```

with exact conformance evidence capable of representing:

```text
List<Int>    -> Item = Int
List<String> -> Item = String
```

under the same source `ImplId`.

What P1 must **not** yet make legal is type-level projection such as:

```phalcom
Self::Item
T::Item
```

in arbitrary signatures or generic code. P2 begins exactly from the canonical exact associated binding map established here.
