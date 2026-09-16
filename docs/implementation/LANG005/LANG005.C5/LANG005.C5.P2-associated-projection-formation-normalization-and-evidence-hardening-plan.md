---
id: LANG005.C5.P2
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C5.P1
follows: LANG005.C5.P1
supersedes: null
prepared: 2026-09-15
repository: aureat/phalcom-lang
repository_baseline: 3a2dcbd49fa7548337b52e9562cab4f8582741e4
repository_baseline_commit: "chore: synchronize accumulated workspace changes"
intended_repository_path: docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-associated-projection-formation-normalization-and-evidence-hardening-plan.md
next_plan: LANG005.C5.P3
---

# LANG005.C5.P2 — Associated Projection Formation, Normalization, and Evidence Hardening

> **For agentic workers:** this plan is deliberately written for GPT-5.6 Luna as the primary implementer under the repository's Luna workflow. Follow `AGENTS.md`, `docs/workflow/luna-implementer-prompt.md`, and `docs/workflow/shared-consultation-escalation-protocol.md`. If the Superpowers workflow is available in the execution environment, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` as appropriate, but this Phalcom plan and checkpoint remain the architecture authority.
>
> **Planning verification note:** this plan was prepared by re-auditing the pushed `main` tree at `3a2dcbd49fa7548337b52e9562cab4f8582741e4` and reconciling it with the C5 checkpoint/guidance and P1 walkthrough/handoff. The planning session did **not** rerun Cargo tests. Test results cited as inherited baseline evidence come from the completed P1 records; T0 requires Luna to establish the live execution baseline.

**Goal:** repair the P1 associated-conformance completeness seam that projection will depend upon, then implement canonical contextual associated-type projection beginning with `Self::Item`, including symbolic formation, source/exact normalization, recursive nested normalization, cycle/terminal-state handling, signature/default/witness integration, and owner-complete incremental/source-tooling support without introducing generic trait-bound proof or runtime projection solving.

**Architecture:** associated projections become canonical type-graph nodes carried by `TypeId`, keyed semantically by subject + trait application/context + `AssociatedTypeRequirementId`. Structural type substitution/materialization rewrites the projection's constituent types but never solves the projection. A dedicated semantic normalizer uses three authority modes—abstract trait context, source conformance binding plan, and exact `ConformanceEvidence`—and preserves non-success proof states instead of collapsing them to `Dynamic`. P1's source `ImplId`, associated binding plan, exact associated map, and singular conformance completeness remain the authoritative spine.

**Tech stack:** Rust workspace; `phalcom-ast`; `phalcom-semantic`; semantic DB/snapshot/query infrastructure; `phalcom-lsp`/source-index projection as required; `phalcom-core` only as a compile-time consumer check unless P2 semantic products force a direct lowering adaptation.

**Primary specification:** `docs/specs/objects/traits.md`

**Checkpoint authority:** `docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md`

---

## 0. Executor contract

This plan is designed for a capable implementation model operating under **constrained architectural authority**.

The implementer may adapt local mechanics to the live repository, but must not redesign the semantic model. In particular:

- verify every file/symbol named below before editing it;
- adapt private helper names, local module factoring, imports, and mechanically drifted signatures without consultation;
- preserve all `INV-*` invariants below;
- do not infer architecture from a failing test when the accepted specification/checkpoint says otherwise;
- do not use compiler/runtime/LSP convenience as justification for moving canonical semantic authority out of `phalcom-semantic`;
- do not broaden P2 into the explicitly deferred visibility/private-authority design lane;
- do not broaden P2 into C6 generic trait assumptions;
- follow the verification budget instead of running broad suites reflexively;
- classify and defer unrelated failures;
- stop and consult when a global or task-local trigger fires;
- keep `LANG005.C5-CHECKPOINT.md` current at meaningful state transitions;
- create `LANG005.C5.P2-walkthrough.md` and `LANG005.C5.P2-handoff.md` before declaring this plan administratively complete.

Testing is evidence gathering. A focused PASS answers the question for which the test was run; it is not a prompt to run a broader suite.

---

# 1. Goal

Implement the C5 associated-projection authority layer so that contextual `Self::Item` is a first-class canonical type form that can remain symbolic in an abstract trait, normalize through a source conformance's associated binding plan while that conformance is being analyzed, normalize through exact conformance evidence for exact subjects, compose recursively inside ordinary types, and participate soundly in trait signatures/defaults/witness compatibility.

Before projection is allowed to consume P1 evidence, close the confirmed P1 completeness defect whereby an unrelated deferred conditional inherent witness can bypass fixed associated-binding incompleteness during exact conformance resolution.

This plan is therefore intentionally:

```text
corrective evidence hardening
        +
canonical projection formation
        +
projection normalization authority
        +
focused semantic integration
```

It is **not** the P3 Iterable migration/certification plan.

---

# 2. Checkpoint acceptance objective

`LANG005.C5` owns associated types as trait-owned declarations whose concrete meanings come from conformance-dependent evidence. P1 established declaration/binding/evidence foundations. P2 must make those foundations usable as types.

P2 is accepted when all of the following bounded conditions hold:

1. fixed source-invalidity—including associated binding failures—cannot be erased merely because some behavioral witness is deferred until exact specialization;
2. `Self::Item` is represented losslessly in type syntax and resolves contextually to one exact `AssociatedTypeRequirementId`;
3. the canonical semantic projection identity contains a canonical subject type, a canonical relevant trait application/context, and the exact trait-owned associated requirement ID;
4. projection identity is not `subject + string name`, is not source-range identity, and is not an expression-level associated member lookup;
5. abstract trait checking can retain a symbolic projection without pretending that a concrete conformance exists;
6. source conformance checking can normalize contextual projections through the source `ConformanceAssociatedTypePlan` without recursively requesting the final evidence it is helping to build;
7. exact projection normalization consumes C4/P1 exact conformance matching/evidence rather than re-reading source syntax;
8. nested projections normalize through ordinary canonical type structure, including applications, tuples, records, callable types, unions, and exact enum-case-containing types where legal;
9. recursive associated bindings are bounded and produce a structured recursive/cyclic terminal outcome rather than stack overflow, infinite recursion, arbitrary `Dynamic`, or source-order behavior;
10. ambiguity, unknown/insufficient evidence, blocked, dynamic-boundary, cancelled, budget-exceeded, and internal-failure states remain semantically distinct;
11. trait signatures/defaults and conformance signatures/witness compatibility consume the correct projection semantics at the correct phase;
12. exact enum-case subject identity survives normalization;
13. projection dependencies are owner-complete and cold/incremental equivalent for the P2 product surface;
14. source navigation for the associated name in `Self::Item` uses `SemanticTargetId::AssociatedType(AssociatedTypeRequirementId)` rather than a separate name resolver;
15. compiler/runtime remain consumers of already-decided semantic types; no runtime projection solver is introduced;
16. P2 stops before generic `T: Trait` assumption/proof, conditional trait conformance, GATs, associated defaults, and the P3 core-library migration.

P2 advances but does not close the full C5 checkpoint. P3 remains responsible for the associated-type-driven core `Iterable` migration, broad cross-stack integration, stress/negative corpus expansion, and checkpoint certification.

---

# 3. Repository grounding

## 3.1 Prepared baseline

```text
repository: aureat/phalcom-lang
branch:     main
revision:   3a2dcbd49fa7548337b52e9562cab4f8582741e4
commit:     chore: synchronize accumulated workspace changes
parent:     e5152196d716ed70fc47d104a0fd789e602285db
prepared:   2026-09-15
```

The pushed `main` revision above was verified during planning. It is one synchronization commit beyond the P1/C5 planning baseline recorded in the older checkpoint metadata.

## 3.2 Repository facts verified during P2 planning

### Workflow and state

- root `AGENTS.md` places canonical language/type/name/proof authority in `phalcom-semantic` and requires compiler/runtime/LSP consumers to consume canonical products rather than reconstruct them;
- `AGENTS.md` requires narrow BUILD testing, one serious correction for a repeated semantic failure, and immediate consultation for architectural failure;
- the checkpoint record is the shared durable state authority;
- the current C5 checkpoint still records the older `e515219...` planning baseline and `active_plan: null`; T0 must re-ground it for P2 without rewriting historical P1 evidence as though it had been collected at the new revision;
- the P1 walkthrough/handoff still describe the pre-push dirty-tree state. Their architecture remains useful, but their Git state is historical rather than the current takeover state;
- `docs/specs/` currently has no root `README.md` despite a generic `AGENTS.md` reference to one. This is a documentation-layout inconsistency, not a P2 blocker. Use the effective named spec `docs/specs/objects/traits.md`.

### P1 associated-type authority

- `AssociatedTypeRequirementId { owner: DeclarationId, index: u32 }` is the trait-owned associated requirement identity;
- `TraitSurface.associated_types` is separate from behavioral `TraitRequirementId` members;
- `AssociatedTypeBindingTemplate` stores an unspecialized binding value under one source `ImplId`;
- `ConformanceAssociatedTypePlan` owns source binding resolution/failures/fingerprint;
- `ConformanceEvidence.associated_types` stores exact specialized bindings keyed by the trait-owned associated requirement;
- `ConformanceFailure::AssociatedType` participates in the one existing `ConformanceCompleteness`;
- `SemanticTargetId::AssociatedType(AssociatedTypeRequirementId)` already exists for declaration/binding source identity.

### Confirmed P1 correctness defect

The live `resolve_conformance_evidence` path currently contains the following semantic shape:

```rust
let has_deferred_selection = plan
    .requirements
    .values()
    .any(|selection| {
        matches!(
            selection,
            RequirementSelectionTemplate::ConditionalInherent { .. }
        )
    });

if let Some(resolution) =
    plan.completeness.clone().into_resolution(head.impl_id.clone())
    && !has_deferred_selection
{
    return resolution;
}
```

The existence of **any** exact-specialization-deferred behavioral selection suppresses early consumption of the whole source completeness value. Because source completeness also contains fixed associated-binding failures, exact specialization can proceed past a source `Incomplete` that it is not semantically capable of repairing.

The later exact path specializes accepted bindings and can construct `ConformanceResolution::Proven(...)`. The existing associated-completeness regression does not specifically combine a missing/invalid associated binding with a deferred `ConditionalInherent` selection. P2 must add that discriminator and close the soundness hole before projection relies on exact evidence.

### Type representation

Live `TypeTerm` is:

```rust
pub enum TypeTerm {
    Canonical(TypeId),
    SelfType(SelfTypeTerm),
    Infer(InferVarId),
}
```

Live canonical `TypeData` contains canonical nested `TypeId` structure:

```text
Never
Unit
ClassObject
Nominal
Applied
ExactCase
Union
Tuple
Record
Callable
Family
Parameter
Lambda
SelfType
```

There is no associated projection node.

Because ordinary nested type nodes store child `TypeId`s, a projection represented only as an outer `TypeTerm` cannot naturally represent:

```phalcom
Option<Self::Item>
(Self::Key, Self::Value)
{ current: Self::Item }
(Self::Item) -> List<Self::Item>
```

without widening the entire type graph from `TypeId` children to `TypeTerm` children. This plan therefore fixes the **carrier** of a projection as a canonical `TypeId`/`TypeData` form. The private Rust factoring may use an interned projection ID if that better matches the live module graph, but P2 must not settle for a `TypeTerm`-only side representation.

### Structural type transformation

`TypeEnvironment` currently maps ordinary type parameters, row parameters, and `Self`. `TypeView::materialize` recursively rewrites canonical `TypeData` shapes and substitutes `Self`/generic parameters. It performs no conformance lookup.

P2 preserves that division:

```text
TypeEnvironment / TypeSubstitution / TypeView
    = structural rewrite only

projection normalization
    = semantic conformance/associated-binding query
```

### Type formation and trait surface

- `TypeAnnotationExpr` has no projection syntax node;
- `TypeFormationSite` currently provides only module + optional `SelfTypeTerm`, which is insufficient to identify which trait owns `Item`;
- `build_trait_surface` currently walks source members in one pass: associated declarations are registered when encountered, while behavioral/property signatures are formed when encountered;
- once signatures can contain `Self::Item`, a lookup into a partially built associated table would make source order semantically observable, so P2 must make associated declaration identity/name availability complete before behavioral signature formation.

### Proof-state and exhaustive-consumer infrastructure

- the live relation/query infrastructure already distinguishes refutation, blocked/unknown-like conditions, dynamic boundary, cancellation, budget exhaustion, and internal failure;
- P2 should reuse/extend this style rather than expose an `Option<TypeId>` normalizer;
- adding a canonical projection variant affects exhaustive consumers in `types/store.rs`, `types/environment.rs`, `types/substitution.rs`, `types/instantiation.rs`, `types/relation.rs`, `export.rs`, `metadata/export.rs`, `presentation.rs`, inference/GADT/coverage/row helpers, and other compiler-enforced matches;
- not every consumer becomes a projection solver. Many should structurally preserve the node, produce an honest unknown/non-exportable result, or require normalization before entry.

## 3.3 Planning evidence boundary

This planning pass was a **static live-tree audit** plus reconciliation against the P1 records. It did not execute Cargo tests. Luna must not cite inherited P1 verification as new P2 execution evidence.

> **Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as a mandatory consultation condition.**

---

# 4. Required reads before implementation

Read in this order. Do not replace these reads with broad repository exploration.

## 4.1 Workflow authority

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
```

## 4.2 C5 state and predecessor

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P1-associated-type-declarations-and-binding-foundations-plan.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P1-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P1-handoff.md
```

Read only the relevant C4 record sections if a C4 interface below differs materially from the live implementation:

```text
docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P2-handoff.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P4-handoff.md
```

Do not re-audit C4 from scratch when the live P1 tree agrees with the handoff.

## 4.3 Normative language authority

```text
docs/specs/objects/traits.md
```

Then read only the effective object/type/callable specs referenced by that file or by a concrete implementation conflict. There is no root `docs/specs/README.md` in the planning baseline.

## 4.4 Production architecture

At minimum, inspect the live versions of:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/db/
phalcom-semantic/src/source_index/
phalcom-semantic/src/metadata/
phalcom-semantic/src/export.rs
phalcom-lsp/
```

## 4.5 Test architecture

```text
phalcom-semantic/tests/semantic/README.md
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/incremental/associated_types.rs
phalcom-semantic/tests/semantic/incremental/db.rs
phalcom-semantic/tests/semantic/integration/
phalcom-ast/tests/trait_syntax.rs
phalcom-ast/tests/integration/
phalcom-core/tests/README.md
```

Do not broadly inspect `phalcom-core` runtime tests unless a P2 task actually changes lowering/runtime code.

---

# 5. Normative authority

## 5.1 Accepted behavior

The following are normative for this plan.

1. **Trait-owned associated identity.** `AssociatedTypeRequirementId` is canonical; associated names are presentation/resolution keys only.
2. **Source binding authority.** A binding LHS resolves only through its exact conformance trait surface.
3. **Contextual projection.** The ratified initial source form is `Self::Item` inside a trait/conformance context where the relevant trait is already known.
4. **Projection is semantic.** Runtime/compiler/LSP do not rediscover associated meaning.
5. **One conformance completeness authority.** Behavioral and associated failures jointly determine the existing proof state.
6. **C6 boundary.** Generic `T: Trait` proof, generic `T::Item` through assumed evidence, and conditional trait conformance remain C6.
7. **Qualification boundary.** `<T as Trait>::Item` or any other explicit qualification syntax is not ratified; implementation must consult before adding one.
8. **User deferral during P2 planning.** Visibility/private-authority redesign is deferred. P2 preserves landed P1 behavior around private fields/methods and conformance-local `via`.

The last item means P2 does **not** decide or implement package-scoped inherent implementation authority, target-owned conformance private authority, conformance-local `via` enablement, or visibility syntax redesign.

## 5.2 Current implementation that is not itself normative

The following are current implementation facts, not language rules:

- the projection carrier does not yet exist;
- `TypeFormationSite` currently has only module + `self_term`;
- trait-surface construction is currently one pass;
- `AssociatedTypeBindingFailure.reason` is currently a string;
- exact evidence currently structurally materializes associated binding templates;
- `resolve_conformance_evidence` currently bypasses source completeness when any conditional inherent selection is deferred;
- P1 currently rejects conformance-local `via`.

Where this plan explicitly corrects an implementation defect, the accepted semantic invariant outranks current behavior.

## 5.3 Future/non-normative ideas

Do not implement these in P2:

```text
<T as Trait>::Item
general T::Item
associated type defaults
associated type trait bounds
GATs
trait inheritance/supertraits
trait objects/existentials
reflection descriptors for associated types
generalized delegation
```

---

# 6. Takeover state

P1 is recorded as `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; C5 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

The planning baseline is now a pushed tree at `3a2dcbd...`, so T0 must record a fresh P2 takeover anchor.

## 6.1 Stable interface map

| Concept | Current symbol/path | Canonical owner | P2 invariant |
|---|---|---|---|
| Source implementation identity | `ImplId` | semantic source/conformance layer | One source `impl` remains one `ImplId` |
| Trait application identity | `TraitRef` | trait semantic layer | Projection carries relevant trait application/context |
| Behavioral requirement identity | `TraitRequirementId` | trait semantic layer | Remains behavior-only |
| Associated requirement identity | `AssociatedTypeRequirementId` | trait semantic layer | Projection keys exact requirement ID, never name alone |
| Trait contract | `TraitSurface` | semantic DB/trait layer | Associated declarations remain separate from behavior |
| Source associated binding | `AssociatedTypeBindingTemplate` | source conformance plan | Value is an unspecialized source template |
| Source associated plan | `ConformanceAssociatedTypePlan` | `ImplId`-owned conformance analysis | Source-context projection authority |
| Source witness plan | `ConformanceWitnessPlan` | `ImplId`-owned conformance analysis | Behavioral plan plus one unified completeness state |
| Exact matching | `ConformanceIndex::query_exact`, `ConformanceHeadMatch` | semantic conformance layer | Reuse; no projection-specific conformance scanner |
| Exact proof | `ConformanceEvidence` | semantic conformance layer | Exact-context projection authority |
| Exact binding map | `ConformanceEvidence.associated_types` | exact evidence | Keyed by requirement ID |
| Final proof state | `ConformanceCompleteness` / `ConformanceResolution` | conformance semantics | Fixed failures cannot be erased by deferred behavior |
| Canonical type store | `TypeStore`, `TypeData` | semantic type layer | Projection is canonical `TypeId`-carried node |
| Structural specialization | `TypeEnvironment`, `TypeView`, `TypeSubstitution` | semantic type layer | Rewrites projection constituents; never solves conformance |
| Relation outcomes | `types/relation.rs`, `types/outcome.rs` | semantic type relation | No false refutation/`Dynamic` convenience |
| Source target | `SemanticTargetId::AssociatedType(...)` | source index | Projection token targets trait-owned requirement |
| Runtime | existing lowering/type-recipe/runtime environment | core/runtime | Never becomes projection solver |

## 6.2 Inherited verification evidence

P1 recorded focused evidence including AST 6/6 + 16/16, semantic trait tests 21/23 with two classified baseline failures, `impls::queries` 53/53, associated incremental tests 3/3, DB incremental tests 14/14, direct-field delegation runtime verticals, the exact generic C4 runtime regression, and affected-crate checks.

These are **historical P1 results**. T0 re-establishes what is still true on the pushed baseline.

---

# 7. Architecture

## 7.1 Authority flow

```text
source syntax
    ↓
AST associated projection occurrence
    ↓
contextual semantic formation
    ↓
canonical projection TypeId
    ↓
projection normalization query
    ├─ abstract trait mode
    │      → retain symbolic canonical projection
    ├─ source conformance mode
    │      → consume ConformanceAssociatedTypePlan
    └─ exact mode
           → consume ConformanceIndex / ConformanceEvidence
    ↓
normalized type or structured terminal state
    ↓
trait signatures/defaults/witness compatibility
    ↓
semantic publication / source index
    ↓
compiler/LSP/runtime consumers
```

## 7.2 Corrective entry gate: fixed failure vs deferred specialization

The correct invariant is:

```text
fixed source-invalidity
    cannot be repaired by exact specialization
    must remain terminal

exact-specialization-dependent behavioral uncertainty
    may be discharged for the exact receiver
```

Fixed source-invalidity includes missing/duplicate/unknown/invalid associated binding and other source-invalid conditions proven before exact specialization.

P2 should preserve one final `ConformanceCompleteness`, but factor exact-resolution logic so only legitimately deferred behavioral applicability is deferred. It may retain fixed failures separately inside the existing plan or introduce a narrowly-scoped internal source-plan subresult. It must **not** introduce a second public/final completeness boolean or proof authority.

## 7.3 Canonical projection carrier

Preferred conceptual shape:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AssociatedTypeProjection {
    pub subject: TypeId,
    pub trait_ref: TraitRef,
    pub requirement: AssociatedTypeRequirementId,
}

pub enum TypeData {
    // existing variants...
    AssociatedProjection(AssociatedTypeProjection),
}
```

Equivalent private factoring through an interned projection ID is mechanically acceptable if it still produces one canonical `TypeId` and preserves the same identity.

Fixed identity fields:

```text
subject TypeId
canonical trait application/context
AssociatedTypeRequirementId
```

Not identity:

```text
written name
source range
diagnostic text
conformance source order
```

The projection is a proper type of `KindId::TYPE` in P2 because P1 associated declarations are plain proper types.

## 7.4 Why projection is not a `TypeTerm`-only variant

Nested canonical types store `TypeId` children. P2 must support `Option<Self::Item>`, tuples/records/callables containing projections, and recursively normalized exact values. Do not widen the whole canonical graph to `TypeTerm`; carry projection as a `TypeId`, and use `TypeTerm::Canonical(projection_type_id)` at term boundaries.

## 7.5 Syntax representation

Use a lossless distinct type-annotation node, conceptually:

```rust
pub struct AssociatedTypeProjectionSyntax {
    pub subject: Box<TypeAnnotation>,
    pub name: String,
    pub name_range: SourceRange,
    pub range: SourceRange,
}

pub enum TypeAnnotationExpr {
    // existing forms...
    AssociatedProjection(AssociatedTypeProjectionSyntax),
}
```

The private AST name/factoring is flexible. Initial grammar acceptance is deliberately narrow: `Self::Item` in a type position. Do not reinterpret it as an ordinary nominal/static symbol reference, and do not expand to arbitrary `T::Item` or explicit trait qualification.

## 7.6 Projection formation context

Current `TypeFormationSite { module, self_term }` does not identify the owning trait. Extend type formation with an associated-projection context or an adjacent argument, conceptually:

```rust
pub struct AssociatedProjectionFormationContext<'a> {
    pub trait_ref: TraitRef,
    pub trait_surface: &'a TraitSurface,
    pub mode: AssociatedProjectionContextMode,
}

pub enum AssociatedProjectionContextMode {
    AbstractTrait,
    SourceConformance { impl_id: ImplId },
}
```

Exact factoring is flexible. Formation of `Self::Item` must resolve `Item` only in this trait's associated table/name index and intern a projection with the exact requirement ID. There is no global associated-name search.

## 7.7 Two-phase trait contract construction

Refactor trait-surface construction into at least:

```text
Phase A — trait associated shape
    collect all associated declarations
    assign stable AssociatedTypeRequirementId values
    diagnose duplicate declarations
    establish complete name → ID resolution

Phase B — behavioral contract
    form property/behavior signatures
    allow Self::Item against complete associated shape
    build defaults/requirements
```

Behavior-before-`type Item` and behavior-after-`type Item` must be semantically equivalent.

## 7.8 Three normalization authority modes

### Abstract trait mode

`Self::Item` remains a valid symbolic canonical projection. It is not an error, `Unknown`, or `Dynamic` merely because no concrete conformance exists yet.

### Source conformance mode

Inside `impl<T> Iterator for List<T> { type Item = T; ... }`, contextual `Self::Item` normalizes through the **source associated binding plan**. Do not request final exact evidence while constructing source signatures/witnesses.

### Exact mode

For an exact subject/trait query, use existing exact conformance authority:

```text
ConformanceIndex
    ↓
ConformanceHeadMatch
    ↓
ConformanceEvidence
    ↓
associated_types[requirement]
    ↓
exact type
```

Multiple applicable conformances are ambiguity, not a winner-by-order rule.

## 7.9 Structural materialization vs semantic normalization

Extend structural visitors so a projection is rewritten as:

```text
projection(subject, trait_ref, requirement)
    under TypeEnvironment
        ↓
projection(
    materialize(subject),
    materialize_trait_arguments(trait_ref),
    same requirement
)
```

`TypeEnvironment`, `TypeView`, `TypeSubstitution`, generic materialization, and type-lambda substitution do not query `ConformanceIndex`.

## 7.10 Source associated binding dependency graph

P2 should permit contextual projection in binding RHS types, e.g.:

```phalcom
trait Collection {
  type Element
  type Sequence
}

impl Collection for Buffer {
  type Element = Byte
  type Sequence = List<Self::Element>
}
```

Build source associated plans in order-independent phases:

```text
1. resolve all LHS names to requirement IDs;
2. record complete supplied-binding set;
3. form RHSs with projection syntax preserved symbolically;
4. normalize source-binding dependencies;
5. detect cycles;
6. publish valid normalized templates + structured failures.
```

Do not require textual declaration/binding order for semantic success.

## 7.11 Normalization result algebra

Do not expose normalization as `Option<TypeId>` or `Result<TypeId, String>`.

Conceptual result:

```rust
pub enum ProjectionNormalization {
    Normalized(TypeId),
    Symbolic(TypeId),
    Incomplete { source_impl: ImplId },
    Ambiguous { candidates: Box<[ImplId]> },
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Recursive { cycle: Box<[AssociatedProjectionKey]> },
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}
```

Exact names/payloads may adapt to existing outcome APIs. Distinctions are fixed: unknown != dynamic, blocked != dynamic, ambiguous != unknown, recursive != ordinary unresolved name, and cancellation/budget are not semantic refutations.

## 7.12 Recursive normalization

Use a canonical key equivalent to:

```rust
pub struct AssociatedProjectionKey {
    pub subject: TypeId,
    pub trait_ref: TraitRef,
    pub requirement: AssociatedTypeRequirementId,
}
```

Track active key stack/set and shared query budget/cancellation controls. Detect direct and multi-node cycles deterministically.

## 7.13 Recursive normalization through ordinary type structure

Provide one central semantic helper/query equivalent to:

```rust
normalize_type(
    ty: TypeId,
    context: &ProjectionNormalizationContext,
    budget: &mut QueryBudget,
    cancel: &CancellationToken,
) -> TypeNormalizationOutcome
```

It recursively traverses projection-bearing `AssociatedProjection`, `Applied`, `ExactCase`, `Union`, `Tuple`, `Record`, `Callable`, `Family`, and any legal scoped/lambda form. Reuse the original interned ID when unchanged. Do not duplicate this recursion in witness checking, hover, exact evidence, and defaults.

## 7.14 Exact evidence normalization

Extend exact binding construction:

```text
source binding template
    ↓
TypeView structural materialization
    ↓
projection-aware exact normalization
    ↓
ExactAssociatedTypeBinding.value
```

If exact normalization produces a non-success terminal state, exact conformance resolution must propagate an honest non-proven state rather than publish `Proven` with a falsely exact associated value.

## 7.15 Trait signature/default/witness integration

For `trait Iterator { type Item; next -> Option<Self::Item> }`:

- trait surface stores a symbolic requirement signature;
- source conformance `type Item = T` compares against `Option<T>`;
- exact `List<Int> + Iterator` compares against `Option<Int>`.

Normalize before relation checking where context has sufficient associated evidence. The relation engine must still fail safely if a symbolic projection reaches it: identical canonical projection IDs may be equal; differing unresolved symbolic projections must not become definite nominal mismatch solely because `TypeData` gained a new variant.

## 7.16 Same-name associated declarations in distinct traits

This is not a global name ambiguity. `A.Item` and `B.Item` remain different IDs, and each contextual `Self::Item` resolves against its own trait context. True ambiguity comes from genuinely ambiguous conformance/evidence, not spelling collision.

## 7.17 Exact enum-case preservation

Projection normalization queries using the exact subject. Generic environment construction may inspect the enum root to bind declaration generics, but projection/conformance identity does not erase the exact case.

## 7.18 Incremental query architecture

Required dependency direction:

```text
projection formation
    → TraitSurface associated shape

source normalization
    → ConformanceAssociatedTypePlan
    → impl/trait generic environment

exact normalization
    → applicable conformance contribution/evidence
    → exact associated binding

source projection occurrence
    → AssociatedTypeRequirementId source mapping
```

Prefer existing `QueryKey`/`SemanticDependency`/fingerprint patterns. Do not solve invalidation through whole-workspace recomputation when owner-indexed products suffice.

## 7.19 Tooling

The `Item` token in `Self::Item` maps to `SemanticTargetId::AssociatedType(requirement)`. Definition goes to the trait `type Item`, not a concrete binding. Hover/presentation may show exact normalized meaning when exact evidence is already available, but must not implement a second LSP conformance solver. P3 owns broader UX polish/certification.

---

# 8. Ownership boundaries

## 8.1 Owns

- `phalcom-ast`: lossless syntax/ranges, not trait proof.
- `phalcom-semantic` trait/conformance layer: requirement identity, source plan, exact evidence, completeness, projection authority.
- semantic type layer: canonical projection node, structural traversal, safe relation/presentation behavior.
- DB/snapshot layer: query cache/dependencies/fingerprints/replacement.
- source-index/editor semantic layer: occurrence → canonical associated target.
- compiler/runtime: execution consequences only after semantic decisions.

## 8.2 Does not own

P2 does not own visibility/private-authority redesign, package implementation ownership, conformance-local `via` enablement, generic trait assumptions, conditional conformance, associated defaults/constraints/GATs, supertraits, trait objects, reflection descriptor design, generalized delegation, full Iterable migration, or runtime projection solving.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Associated declaration identity | `AssociatedTypeRequirementId` / `TraitSurface` | formation, bindings, tooling | name-only ID table |
| Projection canonical identity | semantic type store | signatures, normalizer, presentation | AST range/name identity |
| Relevant trait for `Self::Item` | trait/conformance formation context | annotation resolver | global `Item` search |
| Source binding meaning | `ConformanceAssociatedTypePlan` | source normalizer, exact evidence | syntax reread in checker/LSP |
| Exact associated meaning | `ConformanceEvidence.associated_types` | exact normalizer | runtime/compiler impl scan |
| Final conformance validity | `ConformanceCompleteness` / `ConformanceResolution` | consumers | `associated_complete: bool` |
| Structural substitution | `TypeEnvironment` / `TypeSubstitution` | specialization | conformance lookup inside substitution |
| Projection terminal state | semantic normalizer result | diagnostics/callers | fallback to `Dynamic` |
| Source navigation | source-index target ID | LSP/editor | textual `Item` lookup |
| Runtime type fact | existing lowering/reification products | VM | trait-specific runtime resolver |


---

# 9. Global invariants

Every task and test must preserve these invariants.

### INV-01 — Associated identity remains trait-owned

Projection uses `AssociatedTypeRequirementId`; name is presentation only.

### INV-02 — Projection identity is semantic and complete

Canonical projection identity contains subject + relevant canonical trait application/context + exact associated requirement.

### INV-03 — Source provenance is not type identity

Ranges, written names, and source ordering do not enter canonical projection equality/hash identity.

### INV-04 — One source conformance remains one `ImplId`

Exact projection queries never manufacture per-application source implementation identities.

### INV-05 — Binding template and exact binding remain distinct

Source generic binding templates specialize through exact evidence; they are not overwritten into exact values.

### INV-06 — Conformance completeness remains singular

P2 may factor internal analysis state, but there is one final conformance proof-state authority.

### INV-07 — Fixed source failures are non-deferrable

An unrelated exact-deferred behavioral witness cannot erase a missing/duplicate/unknown/invalid associated binding or other fixed source-invalidity.

### INV-08 — Projection carrier is canonical `TypeId`

Nested associated projections compose through ordinary canonical type structure; do not create a `TypeTerm`-only parallel tree.

### INV-09 — Structural materialization is not semantic normalization

`TypeEnvironment`, substitution, and type materialization rewrite projection constituents but do not query conformance.

### INV-10 — Abstract trait projection is a valid symbolic type

`Self::Item` in a trait may remain symbolic without concrete evidence and without becoming `Unknown`/`Dynamic`.

### INV-11 — Source conformance projection uses source-plan authority

Conformance-local signatures/RHSs do not recursively require final exact `ConformanceEvidence`.

### INV-12 — Exact projection uses exact evidence authority

Exact normalization consumes C4/P1 conformance matching/evidence; it does not rescan syntax.

### INV-13 — Contextual name resolution never globally searches associated names

Same-spelled associated declarations in different traits remain independent.

### INV-14 — Trait member order is semantically irrelevant

Associated declarations are available to all trait signatures regardless of textual ordering.

### INV-15 — Exact enum-case identity survives normalization

Case identity is retained through conformance lookup/projection specialization.

### INV-16 — Terminal states remain distinct

Unknown, blocked, dynamic, ambiguous, recursive, cancelled, budget-exceeded, and internal failure are not collapsed for convenience.

### INV-17 — Recursion is bounded by canonical cycle identity and query controls

Recursive associated bindings cannot overflow or loop indefinitely.

### INV-18 — Exact evidence does not publish falsely exact unresolved values

If an exact binding should normalize and cannot, exact conformance resolution does not return `Proven` with a misleading associated value.

### INV-19 — Relation checking does not false-refute symbolic projections

Unnormalized symbolic projection is handled honestly when it reaches type relation logic.

### INV-20 — Incremental replacement is owner-complete

Add/edit/delete/rename of declarations, bindings, and projection occurrences leaves no stale semantic/source products and converges with cold analysis.

### INV-21 — Tooling consumes semantic identity

LSP/editor/source index never owns an independent associated-type/conformance resolver.

### INV-22 — Compiler/runtime remain non-authoritative

No runtime trait scan, impl scan, class-dictionary inference, or projection solver is introduced.

### INV-23 — P2 stops before C6 generic proof

No one-off `T: Trait`, assumed conformance flag, or generic `T::Item` proof mechanism is introduced.

### INV-24 — P1 property/`via` architecture is not redesigned in P2

Trait properties remain behavioral getter/setter obligations; landed direct-field `via` remains ordinary accessor elaboration.

### INV-25 — Visibility/private authority is deferred

P2 preserves landed private/conformance-local-`via` behavior and does not absorb the separately deferred visibility/ownership design work.

### INV-26 — No new explicit projection qualification syntax without consultation

`<T as Trait>::Item` or any alternative spelling is not silently added.

---

# 10. Non-goals

1. Do not migrate Universe/core `Iterable` to the final associated-type-driven API; that is P3.
2. Do not repair unrelated outgoing-pack/Universe call-entry baselines unless a P2-specific reproducer proves causal coupling.
3. Do not introduce generic trait bounds such as `T: Iterable`.
4. Do not implement generic `T::Item` through assumed evidence.
5. Do not implement conditional trait conformance.
6. Do not add associated defaults, GATs, or trait-bound associated declarations.
7. Do not invent explicit qualification syntax.
8. Do not add supertraits or trait inheritance.
9. Do not add trait objects/existentials.
10. Do not build public reflection descriptors for associated types.
11. Do not change `ImplId` identity.
12. Do not merge `TraitRequirementId` and `AssociatedTypeRequirementId`.
13. Do not add a parallel conformance completeness authority.
14. Do not add runtime associated-type tables as semantic truth.
15. Do not redesign private field/private method access or package implementation authority.
16. Do not enable conformance-local `via` as part of this plan.
17. Do not broaden direct-field delegation.
18. Do not perform unrelated type-store/reflection refactors solely because a new `TypeData` variant makes them visible.
19. Do not opportunistically rewrite old implementation records except where P2 checkpoint state needs a current anchor/amendment.
20. Do not run release/workspace certification during ordinary P2 BUILD.

---

# 11. Expected impact map

## 11.1 Expected source areas

### Syntax

- `phalcom-ast/src/ast.rs` — associated projection type-annotation node.
- `phalcom-ast/src/parser.rs` and type-annotation helpers — parse contextual `Self::Item`.
- AST visitors/range/source helpers that exhaustively match `TypeAnnotationExpr`.

### Canonical type model

- `phalcom-semantic/src/types/store.rs` — projection canonical node, interning, formatting, and kind.
- `phalcom-semantic/src/types/environment.rs` — structural `Self`/parameter materialization through projection constituents.
- `phalcom-semantic/src/types/substitution.rs` — structural substitution.
- `phalcom-semantic/src/types/instantiation.rs` — generic materialization.
- `phalcom-semantic/src/types/relation.rs` — safe relation treatment.
- `phalcom-semantic/src/types/outcome.rs` — reuse/extend terminal vocabulary only when necessary.
- `phalcom-semantic/src/types/annotation.rs` — projection formation context and annotation lowering.
- `phalcom-semantic/src/types/type_lambda.rs` / scoped-type infrastructure — only if legal projection-bearing scoped forms require explicit support.

### Trait/conformance

- `phalcom-semantic/src/traits.rs` — two-phase associated shape + behavioral surface, abstract projection formation.
- `phalcom-semantic/src/impls.rs` — P1 completeness repair, typed associated failures, source/exact projection integration, exact binding evidence.
- `phalcom-semantic/src/snapshot.rs` — exact projection/evidence query facade or memoized query entry.
- `phalcom-semantic/src/checker/context.rs` / `checker/body.rs` — attach correct trait/source projection context to body/signature checking.
- `phalcom-semantic/src/signature.rs` and declaration-signature consumers — projection-bearing declared types where needed.

### Incremental/query

- `phalcom-semantic/src/db/` — projection query/fingerprint/dependency integration as required by live architecture.
- `phalcom-semantic/src/checker/analysis.rs` — new dependency category only if existing trait/conformance dependencies are insufficiently precise.
- `phalcom-semantic/src/session.rs` / snapshot publication — owner-complete product publication/removal if needed.

### Source/tooling

- `phalcom-semantic/src/source_index/` — projection occurrence target.
- `phalcom-semantic/src/presentation.rs` / diagnostic presentation — symbolic/normalized presentation.
- `phalcom-lsp/` — only minimal consumer adaptation if semantic source-index products require it.

### Exhaustive type consumers

Verify and adapt only as required:

```text
phalcom-semantic/src/export.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/coverage/inhabitation.rs
phalcom-semantic/src/advisory/formal.rs
```

Do not turn these files into projection solvers.

## 11.2 Expected tests

- `phalcom-ast/tests/trait_syntax.rs` — `Self::Item` type syntax.
- existing AST integration module if parser behavior crosses it.
- `phalcom-semantic/tests/semantic/capabilities/traits.rs` — formal projection semantics and P1 completeness regression.
- `phalcom-semantic/tests/semantic/impls/queries.rs` — exact conformance/evidence projection behavior.
- `phalcom-semantic/tests/semantic/incremental/associated_types.rs` — projection dependency/replacement/cold parity.
- relevant `phalcom-semantic/tests/semantic/integration/` source-index/presentation files — projection navigation target.
- focused type-foundation tests only where direct canonical structural laws need them.

## 11.3 Expected documentation/state

- `docs/specs/objects/traits.md`
- `docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md`
- `docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md`
- `LANG005.C5.P2-walkthrough.md`
- `LANG005.C5.P2-handoff.md`

## 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary to make the canonical projection type compile and behave coherently.

**STOP AND CONSULT** before entering a materially different subsystem if that entry requires changing identity, proof authority, runtime representation semantics, generic-proof architecture, or the accepted P2/P3 boundary.

---

# 12. Implementer decision authority

## 12.1 FIXED

| Decision | Fixed rule |
|---|---|
| Projection carrier | canonical `TypeId` type-graph representation; not a `TypeTerm`-only parallel tree |
| Projection identity | subject + relevant trait application/context + `AssociatedTypeRequirementId` |
| Associated name | presentation/resolution key within one known trait only; not identity |
| Source provenance | kept outside canonical type identity |
| Abstract trait behavior | projection can remain valid symbolic type |
| Source conformance authority | source `ConformanceAssociatedTypePlan` |
| Exact authority | C4/P1 exact conformance evidence |
| Structural rewrite | never queries conformance |
| Final conformance validity | one `ConformanceCompleteness`/`ConformanceResolution` authority |
| Fixed P1 failures | cannot be waived by deferred behavioral selection |
| Cycle behavior | deterministic structured recursive result; no infinite recursion |
| Terminal states | preserve distinctions; do not fall back to `Dynamic` |
| Trait construction | associated shape available before projection-bearing behavioral signatures |
| Exact case | exact identity preserved |
| Tooling | consumes canonical associated target ID |
| Runtime | no projection/conformance solver |
| C6 boundary | no generic trait assumptions/proofs |
| Syntax boundary | contextual `Self::Item` only; no new explicit qualification |
| Visibility lane | deferred; preserve landed P1 behavior |

## 12.2 MECHANICALLY FLEXIBLE

Luna may adapt without consultation:

- exact private Rust helper/type names;
- whether projection data is inline in `TypeData::AssociatedProjection(...)` or referenced by a separately interned private projection ID;
- local module/file split for a `types/projection.rs` helper if useful;
- private map/iterator choices where deterministic behavior is preserved;
- parser helper names/factoring;
- whether an existing generic result wrapper can host the projection result algebra without losing states;
- exact diagnostic enum names when equivalent existing families are reused;
- exact DB cache/query helper names;
- test fixture helper names;
- mechanical handling of compiler errors caused by exhaustive `TypeData` matching;
- equivalent live paths for tests whose repository module names drifted.

## 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `main` is still `3a2dcbd...` or a mechanically compatible descendant | Git commands | adapt drift; consult if architecture changed |
| `AssociatedTypeRequirementId` remains canonical P1 identity | `traits.rs`, checkpoint | STOP if replaced by name/other identity |
| exact conformance still uses `ConformanceIndex` + `ConformanceEvidence` | `impls.rs`, `snapshot.rs` | adapt mechanics; STOP if proof authority changed |
| conditional inherent deferral still bypasses aggregate source completeness | `impls.rs` | if already fixed, keep regression and skip duplicate fix |
| associated failures remain string-classified | `impls.rs` | use live typed form if already hardened |
| projection can be represented as canonical `TypeData` without architectural cycle | type/trait module graph | factor privately if mechanical; STOP if TypeId carrier becomes impossible |
| `TraitRef` can represent abstract trait application using trait parameters | trait-ref formation | adapt internal constructor; STOP if this requires generic trait proof |
| parser has narrow type-annotation extension point for `Self::Item` | parser | adapt mechanics; STOP if broader unratified syntax is required |
| source associated plan can be LHS-first/RHS-second | `impls.rs` builder | refactor mechanically; STOP if another authority owns binding meaning |
| existing outcomes preserve blocked/dynamic/cancel/budget states | outcome/relation code | reuse/extend; STOP if a required state must be lost |
| query/dependency infrastructure can memoize/bound projection | DB/snapshot | adapt; STOP if only whole-workspace invalidation seems possible |
| source index has a type-annotation occurrence extension point | `source_index/` | extend; STOP if LSP must name-solve independently |
| published metadata/export needs projection-bearing representation | metadata/export | add stable translation if mechanical; STOP if a second associated identity scheme is required |
| P2 needs no runtime solver | compiler/core check | STOP if semantic correctness appears to require runtime solving |
| C5-BL-07 remains unrelated Universe `Bool` failure | T0 focused run | one bounded reclassification pass |

---

# 13. Global STOP / CONSULT triggers

Stop editing and build the repository's implementation incident packet if any of the following occurs:

1. the canonical projection cannot be carried by `TypeId` without widening ordinary nested type representation or creating a second canonical type tree;
2. correct `Self::Item` formation appears to require a global trait search by associated name;
3. correct formation requires a new user-visible explicit trait qualification syntax;
4. correct P2 behavior requires generic `T: Trait` proof/assumption machinery;
5. `AssociatedTypeRequirementId` would need to be replaced or merged with `TraitRequirementId`;
6. `ImplId` source identity would need to change;
7. a second final conformance completeness/proof authority appears necessary;
8. fixed source-invalidity can only be preserved by bypassing the existing completeness architecture;
9. source conformance projection can only be implemented by recursively requesting final evidence being constructed;
10. type substitution/materialization appears to need access to `ConformanceIndex` or workspace semantic state;
11. projection normalization would need to collapse unknown/blocked/ambiguous/recursive state to `Dynamic`;
12. exact enum-case identity cannot be retained with existing C4/P1 matching;
13. projection normalization needs runtime trait/impl scanning or a new VM projection protocol;
14. metadata/export requires a second independent associated identity representation;
15. incremental correctness appears to require whole-workspace invalidation without a clear architectural reason;
16. relation tests can pass only by treating all symbolic projections as unrelated nominal types or by silently accepting them;
17. the same nontrivial semantic failure survives one serious evidence-based correction;
18. two viable normalization designs imply materially different semantic authority or cache identity;
19. implementation must enter a materially unexpected subsystem for semantic reasons;
20. accepted spec, checkpoint, this plan, live code, and/or test oracle materially conflict;
21. passing a test would require weakening any `INV-*`;
22. visibility/private-authority/package ownership changes appear necessary to make projection work;
23. P2 expands materially into P3 Iterable migration/certification;
24. P2 expands materially into C6 generic conformance proof.

A triggered consultation cannot be self-waived.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Allow up to **three coherent correction cycles** while each cycle has a concrete cause and demonstrates progress:

```text
inspect exact compiler/test failure
identify one concrete mechanical cause
make one coherent correction
rerun the smallest discriminating command
```

## 14.2 Semantic failures

Before a nontrivial semantic correction, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow **one serious corrective attempt for the same underlying semantic problem**. If it persists, STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative architecture attempts. Consult immediately.

## 14.4 Test rerun rule

Never rerun an unchanged failing test unless something relevant to its causal path changed.

---

# 15. Testing surface analysis

The plan deliberately designs broad correctness coverage but executes it selectively.

## 15.1 Coverage obligation ledger

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | soundness correction | INV-06, INV-07 | missing associated binding + deferred conditional inherent behavior cannot become `Proven` |
| CV-02 | soundness correction | INV-06, INV-07 | invalid/duplicate associated failure also survives exact-deferred behavioral path |
| CV-03 | diagnostic provenance | INV-01, INV-20 | missing-binding diagnostic uses source-valid conformance provenance and stable requirement identity |
| CV-04 | parser success | INV-02, INV-26 | `Self::Item` parses in type position with exact ranges |
| CV-05 | parser negative | INV-23, INV-26 | unratified broader projection syntax is not silently introduced |
| CV-06 | canonical identity | INV-01, INV-02, INV-03 | repeated same contextual projection canonicalizes identically |
| CV-07 | trait identity | INV-01, INV-13 | same `Item` spelling in A/B remains distinct |
| CV-08 | source order | INV-14 | behavior before/after associated declaration forms equivalent contract |
| CV-09 | abstract formation | INV-10 | trait signature retains symbolic `Self::Item` |
| CV-10 | nested formation | INV-08 | `Option<Self::Item>` contains canonical projection |
| CV-11 | nested composite | INV-08 | tuple/record/callable forms preserve projections structurally |
| CV-12 | source normalization | INV-11 | `type Item = T`; `Self::Item` normalizes to source impl `T` |
| CV-13 | binding RHS dependency | INV-11, INV-14 | `Sequence = List<Self::Element>` normalizes independent of binding order |
| CV-14 | exact normalization | INV-12, INV-18 | exact `List<Int>` projection normalizes to `Int` |
| CV-15 | generic specialization | INV-04, INV-05, INV-12 | one source impl yields distinct exact values for two subjects |
| CV-16 | nested normalization | INV-08, INV-12 | exact projection recursively reduces inside composite types |
| CV-17 | exact enum case | INV-15 | case-specific binding remains case-specific |
| CV-18 | direct cycle | INV-16, INV-17 | `A = Self::A` yields recursive terminal result |
| CV-19 | indirect cycle | INV-16, INV-17 | `A -> B -> C -> A` yields deterministic cycle result |
| CV-20 | ambiguity | INV-13, INV-16 | multiple applicable exact conformances preserve ambiguity |
| CV-21 | unknown/blocked | INV-16 | insufficient evidence remains unknown/blocked under live contract |
| CV-22 | dynamic boundary | INV-16 | only actual dynamic boundary produces dynamic result |
| CV-23 | cancellation/budget | INV-16, INV-17 | bounded query preserves cancellation/budget outcome |
| CV-24 | signature specialization | INV-10, INV-11, INV-12 | required projection specializes before witness relation |
| CV-25 | trait default | INV-10 | default body type-checks with symbolic associated type |
| CV-26 | source witness | INV-11 | conformance witness checks using source binding without final-evidence cycle |
| CV-27 | relation safety | INV-19 | symbolic projection is not false-refuted because it is a new variant |
| CV-28 | incremental RHS edit | INV-20 | binding RHS edit invalidates dependent normalized result only |
| CV-29 | declaration rename/remove | INV-01, INV-20 | stale associated target/projection product disappears |
| CV-30 | cold/incremental parity | INV-20 | mutation sequence converges with cold analysis |
| CV-31 | source navigation | INV-01, INV-21 | `Self::Item` occurrence targets trait `type Item` |
| CV-32 | no LSP resolver | INV-21 | LSP consumes semantic target; does not search names/impls |
| CV-33 | no runtime solver | INV-22 | no VM opcode/projection lookup/conformance scan appears |
| CV-34 | C6 boundary | INV-23 | no generic assumption mechanism added |
| CV-35 | P1 property boundary | INV-24, INV-25 | P2 does not silently alter property/`via` semantics |
| CV-36 | baseline classification | — | C5-BL-07 is rerun/reclassified once and not reflexively repaired |

## 15.2 Dimensions intentionally not required in P2

Unless changed P2 code proves direct relevance, P2 does not require GC/root lifetime, concurrency/liveness, optimizer differential testing, broad VM execution corpus, full reflection API coverage, full Universe Iterable behavior, or workspace release certification.

---

# 16. Verification execution budget

## 16.1 Modes

### BUILD MODE

Default. Sparse discriminating feedback only.

### STABILIZE MODE

Enter at named `G*` gates. Run focused regressions/directly affected modules.

### CERTIFY MODE

Not part of normal P2 work. Enter only if the user explicitly requests it or a checkpoint amendment moves certification into P2.

## 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning semantic/AST module suite
T3 — adjacent source-index/LSP/compiler compile integration
T4 — affected-crate check
T5 — workspace/release verification
```

Do not automatically climb after a PASS.

## 16.3 Mandatory during BUILD

Across the plan, run only the exact new/failing regression needed for the active task: the P1 soundness reproducer, exact parser tests, exact semantic formation/normalization tests, exact cycle/terminal tests, and exact incremental projection tests.

## 16.4 Mandatory during final STABILIZE

At minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-core \
  -p phalcom-lsp
```

plus the focused AST/semantic/incremental/source-index gates specified below. Run Cargo commands serially.

## 16.5 Only if evidence demands

Use only when touched code or unresolved evidence justifies it:

```text
full phalcom-semantic integration binary
full phalcom-core `core` integration target
specific compiler/runtime regression
metadata/export owning suite
full LSP package tests
```

A new canonical `TypeData` variant may cause `phalcom-core` compile errors through shared APIs; `cargo check -p phalcom-core` is sufficient unless executable behavior changed.

## 16.6 Explicitly deferred / DO NOT RUN during BUILD

Do not run by default:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
```

Also do not run broad language corpora, all core runtime tests, all semantic tests after every task, or P3 Iterable/outgoing-pack suites during BUILD.

---

# 17. Baseline / unrelated failure policy

Classify every unexpected failure:

```text
A — definitely caused by P2
B — probably caused by P2
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: one bounded classification pass; record/defer if still unclear, nonblocking, and outside P2 acceptance.
- D: record and continue immediately.
- Do not weaken/skip assertions to make broad gates green.
- Do not consume adviser compute on D unless it blocks P2 acceptance.

## 17.1 Known inherited baselines

Carry forward the C5 baseline ledger unless live evidence changes it:

```text
C5-BL-01 Universe generic call-entry
C5-BL-02 incremental A7 presentation mismatch
C5-BL-03 Universe Bool capability failures
C5-BL-04 Iterable/outgoing-pack generic call-entry
C5-BL-05 formatting drift
C5-BL-06 AST Clippy violations
C5-BL-07 two trait capability failures involving Universe Bool dependencies/capabilities
```

P2 touches trait signature/default formation, so `C5-BL-07` requires one fresh focused reclassification in T0/G0. Do not assume its old D classification blindly, and do not repair it unless a P2-specific discriminator establishes coupling.

Visibility/private-authority questions are not P2 baseline failures; they are an explicitly deferred design lane.


---

# 18. Tasks

The task order is architectural. Do not move exact-evidence consumers ahead of the canonical projection/normalization authority they depend upon.

## T0 — Take over the pushed P1 state and re-ground C5

### Purpose

Establish the exact live starting revision, verify that P1's frozen products still exist, reproduce/classify the known focused baseline once, and update the shared C5 checkpoint so later P2 sessions do not inherit stale pre-push Git state.

### Preconditions

- P1 is recorded complete.
- The user has said P1 implementation was committed/pushed.
- Planning verified pushed `main` at `3a2dcbd49fa7548337b52e9562cab4f8582741e4`.

### Consumes

- C5 checkpoint/guidance.
- P1 walkthrough/handoff.
- live Git state.
- inherited P1 focused evidence.

### Produces

- current P2 starting revision in checkpoint state;
- `active_plan: LANG005.C5.P2` and plan status `IN_PROGRESS` when implementation actually begins;
- a verified live map of P1 associated interfaces;
- one current classification of C5-BL-07;
- no production semantic changes.

### Files and symbols

**Read:**

```text
AGENTS.md
workflow files named in §4
LANG005.C5-CHECKPOINT.md
LANG005.C5-GUIDANCE.md
P1 walkthrough/handoff
traits.rs
impls.rs
snapshot.rs
types/{store,annotation,environment,relation}.rs
semantic test README
traits/impl/incremental P1 tests
```

**Modify:**

```text
LANG005.C5-CHECKPOINT.md
```

Only current lifecycle/takeover state and any verified baseline reclassification belong in this task.

### Required implementation shape

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Preserve unrelated local changes. Do not reset/clean/restore them.

Verify these P1 products by direct source inspection:

```text
AssociatedTypeRequirementId
TraitSurface.associated_types
AssociatedTypeBindingTemplate
ConformanceAssociatedTypePlan
ConformanceEvidence.associated_types
ConformanceFailure::AssociatedType
ConformanceCompleteness
SemanticTargetId::AssociatedType
```

Verify the `has_deferred_selection` exact-evidence path before T1; if live code has already fixed it, record the drift and retain the planned regression.

Update checkpoint state to distinguish:

```text
P1 historical planning baseline: e515219...
P2 live takeover baseline:       current HEAD
```

Do not rewrite P1's historical walkthrough/handoff to pretend they were produced after the push.

### Forbidden approaches

- no broad repository audit;
- no production refactor in T0;
- no repair of C5-BL-07 without P2-specific causal evidence;
- no workspace test/clippy;
- no visibility/private-authority changes.

### Test changes required

None.

### Tests to run now

First, ensure affected crates compile on the pushed tree:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-core \
  -p phalcom-lsp
```

Then run the focused predecessor selectors serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic capabilities::traits
```

The last command is intentionally permitted to reproduce the two recorded C5-BL-07 failures. Record actual test names/output; do not call the whole module a PASS if it still contains failures.

If a filter selects zero tests, inspect `-- --list` and correct the selector; zero tests are not evidence.

### Tests explicitly deferred

- AST parser suites until syntax changes begin.
- core runtime suites.
- full semantic integration binary.
- workspace/release gates.

### Acceptance

T0 passes when:

1. current HEAD and tree state are recorded;
2. P1 frozen interfaces match or only mechanically drift;
3. no unexplained P1 semantic regression blocks P2 takeover;
4. C5-BL-07 is freshly classified;
5. checkpoint records P2 as active with the live start revision.

### Local STOP / CONSULT triggers

- any P1 identity/evidence authority has materially changed;
- exact conformance evidence no longer uses the expected C4/P1 proof spine;
- C5-BL-07 now reproduces only through a P2/P1 associated-type path rather than its known Universe Bool dependency issue;
- live spec/checkpoint contradicts the P2 architecture materially.

### Checkpoint update

Required. T0 establishes the durable P2 takeover anchor and current baseline classification.

---

## T1 — Repair fixed-failure preservation in exact conformance evidence

### Purpose

Close the confirmed P1 soundness defect before projection consumes exact evidence, and harden associated binding failure representation/provenance sufficiently for P2 normalization.

### Preconditions

- T0 verified the live exact-evidence path.
- The exact conformance source plan still combines behavioral + associated failures into one completeness authority.

### Consumes

- `ConformanceWitnessPlan`.
- `ConformanceAssociatedTypePlan`.
- `ConformanceCompleteness` / `ConformanceFailure`.
- `RequirementSelectionTemplate::ConditionalInherent`.
- `resolve_conformance_evidence` and exact inherent applicability.

### Produces

- exact resolution that can discharge only genuinely deferred behavioral applicability;
- fixed source failures that survive exact specialization;
- typed associated binding failure category where the live design benefits from it;
- deterministic missing-binding source provenance;
- focused soundness regressions.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/snapshot.rs                # only if public exact query propagation changes
phalcom-semantic/src/diagnostic.rs              # verify exact DiagnosticCode location
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
```

**Verify before changing:**

```text
ConformanceCompleteness::into_resolution
ConformanceFailure
AssociatedTypeBindingFailure
build_conformance_witness_plan (live equivalent)
resolve_conformance_evidence
ConditionalInherent exact specialization
```

### Required implementation shape

#### A. Separate fixed invalidity from exact-deferred applicability

Do not keep this semantic rule:

```text
has any deferred behavioral selection
    ⇒ ignore all source completeness until later
```

Replace it with:

```text
fixed failures
    retained unconditionally

source terminal state that is not exact-resolvable
    retained unconditionally

only exact-specialization-dependent behavioral selections
    resolved later
```

One acceptable internal shape is conceptually:

```rust
struct ConformanceWitnessPlan {
    // existing source products...
    completeness: ConformanceCompleteness,
    fixed_failures: Box<[ConformanceFailure]>, // private/internal if useful
    // or equivalent factored representation
}
```

Do **not** blindly add this exact field if existing completeness internals provide a cleaner path. The required semantic behavior is fixed; private factoring is flexible.

The final exact resolution must recompute/confirm a single final result after conditional selections:

```text
if fixed failures exist
    → Incomplete
else if non-success source terminal state exists
    → preserve it
else resolve exact-deferred selections
    → if any fails/blocks/etc, preserve exact outcome
    → otherwise Proven
```

#### B. Harden associated binding failure classification

The planning baseline uses:

```rust
AssociatedTypeBindingFailure {
    name: Box<str>,
    reason: Box<str>,
    source: SemanticSourceSpan,
    requirement: Option<AssociatedTypeRequirementId>,
}
```

Prefer structured classification, conceptually:

```rust
pub enum AssociatedTypeBindingFailureKind {
    Missing,
    Duplicate,
    Unknown,
    Invalid,
}
```

with human-readable prose rendered from structured state.

The exact variant set may reflect live failure cases (for example a distinct formation/unsupported kind) but consumers must not branch on diagnostic strings.

#### C. Verify missing-binding provenance

A missing binding has no source binding occurrence. Ensure the primary diagnostic's module/range belongs to the conformance/head or another valid source location in that module. If the diagnostic infrastructure already supports secondary labels, the trait `type Item` declaration can be a secondary source. Do not invent a large diagnostic framework solely for this.

### Forbidden approaches

- no second `associated_complete` boolean used by consumers;
- no special-case `if associated_type_plan.failures.is_empty()` that leaves other fixed failures bypassable;
- no suppressing source failure to get a conditional witness test green;
- no string parsing of `reason` to drive normalization logic;
- no projection code yet;
- no unrelated conformance coherence redesign.

### Test changes required

Add regressions covering CV-01–CV-03.

At minimum, create a source conformance that simultaneously has:

```text
one associated binding failure
AND
one behavioral requirement selected through ConditionalInherent
```

Then query an exact target for which the conditional inherent candidate becomes applicable.

Expected:

```text
ConformanceResolution != Proven
fixed associated failure remains observable
```

A second case should use a non-missing associated failure if fixture cost is low, proving the rule is fixed-invalidity rather than a missing-binding special case.

### Tests to run now

Run only the exact new regressions first, for example after naming them:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic \
  associated_failure_survives_deferred_conditional_witness
```

Then the existing associated completeness regression:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic \
  associated_type_binding_validation_is_part_of_conformance_completeness
```

If implementation changes shared exact conformance resolution, run the owning query module once at G1, not after every edit.

### Tests explicitly deferred

- parser/type formation tests;
- projection tests;
- full `capabilities::traits` module until stabilization gate;
- core runtime.

### Acceptance

T1 is complete when:

1. CV-01 passes;
2. a fixed associated failure cannot be waived by unrelated deferred behavioral selection;
3. final proof authority remains singular;
4. associated failure semantics no longer require parsing free-form reason strings if the planned hardening was applicable;
5. missing-binding diagnostic provenance is source-valid;
6. no projection implementation was smuggled into the conformance repair.

### Local STOP / CONSULT triggers

- preserving fixed failures seems to require a second final proof authority;
- conditional inherent resolution actually needs to repair a source failure category previously assumed fixed;
- the current completeness enum cannot represent the required result without changing C4 downstream contracts materially;
- a diagnostic provenance fix would require redesigning cross-module diagnostic infrastructure.

### Checkpoint update

Required after G1. Record the fixed invariant: exact-deferred behavioral resolution can no longer erase fixed associated/source invalidity. Record any new typed failure interface if it is durable for P2/P3.

---

## T2 — Add lossless contextual `Self::Item` type syntax

### Purpose

Represent the ratified initial projection syntax distinctly in the AST without yet solving its semantic meaning.

### Preconditions

- T1 soundness gate is green.
- No projection syntax exists in live `TypeAnnotationExpr`.

### Consumes

- current type-annotation grammar;
- `SelfType` syntax;
- `::` token/parser conventions;
- `TypeAnnotation` source ranges;
- existing AST source walkers.

### Produces

- distinct associated projection type-annotation AST node;
- parser support for `Self::Item` in type positions;
- exact projection/name range preservation;
- deterministic syntax/unsupported-form behavior;
- no semantic normalization yet.

### Files and symbols

**Read:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/token.rs
phalcom-ast/tests/trait_syntax.rs
AST integration tests containing type syntax
```

**Modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
exhaustive AST visitors/helpers revealed by compiler
phalcom-ast/tests/trait_syntax.rs or the nearest type-annotation test module
```

### Required implementation shape

Add a projection syntax node equivalent to:

```rust
AssociatedTypeProjectionSyntax {
    subject: Box<TypeAnnotation>,
    name: String,
    name_range: SourceRange,
    range: SourceRange,
}
```

The P2 parser accepts `Self::Item` as an associated projection type form.

Preserve the ability to distinguish:

```text
whole projection range
subject range
associated name range
```

Do not parse `Self::Item` as a nominal `StaticSymbolRef` path. Expression-level `owner::member` syntax is a separate category and is not the canonical type projection representation.

For broader forms:

- if the existing grammar currently rejects `T::Item`, keep it rejected;
- if it accidentally accepts it as another unrelated type form, add a P2 semantic guard rather than silently redefining that legacy syntax as generic projection;
- do not add `<T as Trait>::Item` or another qualification spelling.

### Forbidden approaches

- no associated-name semantic lookup in parser;
- no trait lookup in parser;
- no expression AST reuse as canonical type syntax;
- no broad `T::Item` support;
- no normalization in AST layer.

### Test changes required

Cover CV-04 and CV-05.

Add at least:

```phalcom
trait Iterator {
  type Item
  next -> Self::Item
}
```

and a nested parse case:

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
}
```

Assert the AST category and name/range where the test framework makes that practical.

Add a negative/unsupported case proving the parser did not accidentally ratify a broader projection grammar.

### Tests to run now

Run only the exact AST tests added/affected. If they live in `trait_syntax`:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-ast --test trait_syntax \
  self_associated_type_projection
```

Correct the filter to the actual non-zero test name.

### Tests explicitly deferred

- semantic resolution;
- full AST integration module until G2;
- all semantic/core suites.

### Acceptance

T2 is complete when `Self::Item` is losslessly represented as its own type syntax form, nested parsing works, and no new user-visible qualification/generic projection semantics have been introduced.

### Local STOP / CONSULT triggers

- the parser cannot distinguish type projection from expression associated lookup without grammar redesign;
- accepting only `Self::Item` would create syntactic ambiguity requiring a language decision;
- correct source syntax appears to require a new explicit trait qualification.

### Checkpoint update

Not required until G2 unless the AST representation itself becomes a durable public interface worth recording.

---

## T3 — Add canonical projection `TypeId` and structural type plumbing

### Purpose

Introduce the canonical projection type carrier and make all structural type consumers coherent without yet giving arbitrary consumers permission to solve projections.

### Preconditions

- T2 supplies a distinct projection AST node.
- P1 associated identity remains stable.

### Consumes

- `TypeStore` interning/kind infrastructure;
- `TraitRef`;
- `AssociatedTypeRequirementId`;
- `TypeEnvironment` / `TypeView`;
- `TypeSubstitution` / generic materialization;
- type formatting/presentation/export conventions;
- current exhaustive `TypeData` consumers.

### Produces

- canonical projection `TypeId` representation satisfying INV-02/INV-08;
- projection constructor/interner;
- structural rewrite through subject + trait arguments;
- stable formatting/debug presentation;
- safe exhaustive handling across the semantic crate;
- no conformance lookup in structural transforms.

### Files and symbols

**Primary read/modify:**

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/annotation.rs     # type-level construction hook, no normalization yet
```

**Verify/adapt exhaustive consumers:**

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/export.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/coverage/inhabitation.rs
phalcom-semantic/src/advisory/formal.rs
phalcom-semantic/src/impls.rs
```

Compiler errors after adding the enum variant are an expected discovery mechanism for exhaustive matches, but do not automatically imply every match needs new semantics.

### Required implementation shape

#### A. Canonical node

Implement the conceptual identity from §7.3. If cross-module type dependencies make direct inline storage awkward, move only the private data carrier; keep one canonical `TypeId` identity.

Provide a constructor roughly equivalent to:

```rust
TypeStore::associated_projection(
    subject: TypeId,
    trait_ref: TraitRef,
    requirement: AssociatedTypeRequirementId,
) -> TypeId
```

and ensure `KindId::TYPE`.

#### B. Formatting

Canonical debug/user presentation should make trait identity visible enough to avoid misleading same-name output. It may display contextual shorthand when a presenter has context, but `TypeStore::format_type` must not imply that name alone identifies the projection.

A mechanically simple debug form such as:

```text
<Self as Iterator>::Item
```

is acceptable internally even though that text is **not** being ratified as source syntax. If using such a format risks user-visible syntax confusion in established diagnostics, use an unambiguous internal representation instead and let semantic presentation render `Self::Item` contextually.

#### C. Structural transformation

Update `TypeView`, substitution, and generic materialization so projection subject and trait arguments are structurally transformed; requirement ID is stable.

Do not normalize the projection in those helpers.

#### D. Safe exhaustive consumers

For each exhaustive match, classify it before editing:

```text
structural traversal       → recurse/preserve projection
presentation               → render projection
relation                   → equality/safe uncertainty; no solving here
inference conversion       → preserve as canonical atom unless architecture has a projection-aware hook
metadata/export            → stable translate or explicitly non-exportable according to existing publication contract
inhabitation/coverage      → unknown when not semantically normalized, unless proven by existing rules
parameter-detection walker → inspect subject/trait args as appropriate
```

Do not add arbitrary wildcard arms merely to compile if that would hide projection semantics.

### Forbidden approaches

- no `TypeTerm::Projection` as the only canonical representation;
- no second projection arena that creates identity outside the canonical `TypeId` store unless it is purely an implementation detail referenced by TypeData;
- no source span/name in canonical hash key;
- no conformance query from `TypeEnvironment`/`TypeSubstitution`;
- no `Dynamic` fallback for unsupported consumers;
- no broad refactor of unrelated type variants.

### Test changes required

Add direct structural tests for CV-06, CV-10, CV-11 as low-level tests where appropriate:

- same projection identity interns equally;
- substituting abstract `Self` in a projection rewrites only the subject/trait arguments;
- nested applied/tuple/callable/record structures retain the projection node correctly;
- same requirement name under different trait IDs is not equal.

### Tests to run now

After the variant and structural visitors compile:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

Then run only the direct type/projection unit or semantic test(s) added for structural identity.

Do **not** run trait normalization tests yet; the semantic normalizer is intentionally absent.

### Tests explicitly deferred

- exact normalization;
- witness/default integration;
- incremental source-index integration;
- core runtime tests.

### Acceptance

T3 is complete when:

1. projection is one canonical `TypeId` form;
2. repeated identical projection canonicalizes;
3. structural substitutions/materializations preserve and rewrite it correctly without semantic solving;
4. exhaustive consumers compile with honest behavior;
5. no relation consumer false-refutes solely because the new variant exists;
6. no runtime/compiler semantic authority was added.

### Local STOP / CONSULT triggers

- only a `TypeTerm`-only representation seems feasible;
- storing `TraitRef`/requirement identity in canonical type data creates a material dependency-cycle/identity problem;
- metadata publication requires inventing an unrelated stable associated identity;
- inference requires turning projection into an inference variable or `Dynamic` merely to compile;
- relation semantics require a new global solver inside `types/relation.rs`.

### Checkpoint update

Required after G2. Record the canonical projection carrier/identity and structural-vs-semantic-normalization invariant.

---

## T4 — Make trait formation contextual and source-order independent

### Purpose

Resolve `Self::Item` inside trait signatures/default contracts to the correct trait-owned associated identity while retaining a symbolic canonical projection and eliminating member-order dependence.

### Preconditions

- T3 canonical projection type exists.
- P1 `TraitSurface.associated_types` remains authoritative.

### Consumes

- trait header/generic signature;
- `TraitSurface` associated requirements;
- type annotation formation;
- abstract `Self` representation;
- callable/property signature formation.

### Produces

- complete associated-name lookup before behavioral signature formation;
- trait projection formation context;
- symbolic `Self::Item` in trait signatures;
- source-order-independent trait contract;
- same-name-associated-types-across-traits correctness;
- abstract default checking input suitable for later T7 integration.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/traits.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/checker/declaration_signature.rs   # live equivalent if used by trait behavior
phalcom-semantic/src/checker/body.rs                    # context attachment only as required
phalcom-semantic/tests/semantic/capabilities/traits.rs
```

### Required implementation shape

#### A. Trait associated-shape prepass

Split `build_trait_surface` or its live equivalent so all associated declarations and IDs are known before any property/behavior signature type annotations can resolve projections.

Preserve stable ID assignment semantics. If P1 IDs are owner + source-associated ordinal, the prepass must reproduce the same deterministic identity for the same declaration order; P2 is not authorized to redesign identity to name-based IDs.

#### B. Abstract trait application

Construct the relevant symbolic `TraitRef` using the trait declaration and its own generic parameter forms. Example:

```phalcom
trait Converter<Target> {
  type Output
  convert -> Self::Output
}
```

The projection's trait context must retain the symbolic `Converter<Target>` application, not merely the bare declaration.

Do not require a conformance proof for the trait's own abstract context.

#### C. `Self::Item` formation

When resolving the AST projection:

```text
verify subject is the ratified contextual Self form
find Item in current TraitSurface associated-name map
intern projection(subject=abstract Self, trait_ref=current trait app, requirement=id)
```

Unknown associated name is a semantic diagnostic tied to the written name range.

#### D. Same-name traits

Prove that two traits each declaring `type Item` create different projections even if their `Self` representations have similar shape. Trait context is part of identity.

### Forbidden approaches

- no global `Item` search;
- no conformance query in abstract trait formation;
- no source-order lookup into a partially built trait surface;
- no conversion of symbolic projection to `Unknown` just because there is no exact target;
- no associated requirement identity derived from name alone.

### Test changes required

Cover CV-07–CV-10 and the abstract half of CV-25.

Required semantic tests:

1. declaration before use;
2. declaration after use;
3. nested `Option<Self::Item>`;
4. two traits with same `Item` name and distinct projection IDs;
5. unknown `Self::Missing` deterministic diagnostic;
6. generic trait context retains symbolic trait arguments.

Where feasible, compare semantic products rather than only formatted strings.

### Tests to run now

Run the exact new projection-formation tests. Then at G3 run the relevant trait capability slice.

Do not run exact conformance normalization tests yet.

### Tests explicitly deferred

- source conformance RHS dependency normalization;
- exact evidence normalization;
- runtime/compiler vertical;
- full semantic suite.

### Acceptance

T4 is complete when abstract trait projection is a first-class symbolic canonical type, trait member source order does not affect meaning, and same-spelling associated declarations across traits remain distinct without ambiguity.

### Local STOP / CONSULT triggers

- abstract trait context cannot form a canonical `TraitRef` without generic trait-proof machinery;
- P1 ID stability would need to change to make the two-phase builder work;
- trait default signature formation requires exact evidence;
- unknown associated names can only be diagnosed by global lookup.

### Checkpoint update

Required at G3 if the two-phase TraitSurface construction or projection formation context is a durable interface inherited by P3/C6.


---

## T5 — Build source-conformance projection and associated-binding dependency normalization

### Purpose

Make contextual projection usable while a conformance is being analyzed, including associated binding RHSs that depend on sibling associated bindings, without creating a cycle through final `ConformanceEvidence`.

### Preconditions

- T4 can form canonical symbolic projections in a known trait context.
- P1 source associated binding plans remain `ImplId`-owned.

### Consumes

- resolved conformance head/`TraitRef`;
- complete trait associated requirement table;
- impl-owned generic scope;
- P1 associated binding AST/product;
- canonical projection type;
- structural type materialization/substitution.

### Produces

- source-conformance projection context;
- LHS-first/RHS-second associated binding planning;
- source binding dependency graph;
- source normalization of sibling associated projections;
- deterministic source cycles/failures;
- source conformance signatures that can see their own associated bindings without asking for final evidence.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
```

A dedicated private `types/projection.rs` or `associated_projection.rs` module may be created here if it clarifies the source/exact normalizer split, but do not duplicate conformance products.

### Required implementation shape

#### A. Resolve all LHS identities before RHS semantic normalization

Today P1 can form each associated RHS independently after finding its LHS requirement. P2 needs a complete binding identity set before one binding may depend on another.

Conceptual planning shape:

```rust
struct PendingAssociatedBinding {
    requirement: AssociatedTypeRequirementId,
    syntax: TypeAnnotation,
    source: SemanticSourceSpan,
}

// Phase 1: resolve names / duplicates / extras
BTreeMap<AssociatedTypeRequirementId, PendingAssociatedBinding>

// Phase 2: form symbolic RHS templates
BTreeMap<AssociatedTypeRequirementId, TypeId>

// Phase 3: source-normalize projections among those templates
ConformanceAssociatedTypePlan
```

Do not key the dependency graph by source names after LHS resolution. Use requirement IDs.

#### B. Source normalization environment

Source normalization uses:

```text
impl generic parameter environment
trait generic application from the conformance head
target source head / symbolic target type
complete source associated binding map
```

A projection for the current conformance's trait/subject resolves to the sibling binding template.

If the projection's trait context does not match the source conformance's exact trait relationship, do not invent cross-trait proof; preserve/return the appropriate non-normalized state according to the context.

#### C. Source binding cycles

Detect cycle identity through requirement/projection keys, not recursion depth alone.

Required examples:

```phalcom
impl T for X {
  type A = Self::A
}
```

and:

```phalcom
impl T for X {
  type A = Self::B
  type B = Self::C
  type C = Self::A
}
```

The source plan must remain non-proven/incomplete with deterministic diagnostic/proof state.

#### D. Conformance-local signatures

Make the source conformance's own callable signature formation able to consume the source associated plan (or a safely staged read-only view of it) so:

```phalcom
impl<T> Iterator for List<T> {
  type Item = T
  next -> Option<Self::Item> { ... }
}
```

forms the conformance-local declaration contract as `Option<T>` for witness compatibility/body checking.

Do not store a synthetic inherent callable or mutate the trait's symbolic signature.

### Forbidden approaches

- no call to `SemanticSnapshot::resolve_conformance_evidence` while constructing the source plan that evidence consumes;
- no source-order dependency among sibling bindings;
- no requirement-name graph after IDs are resolved;
- no generic trait assumption to resolve a foreign projection;
- no replacing source binding templates with exact target values;
- no recursive call stack without canonical cycle tracking.

### Test changes required

Cover CV-12, CV-13, CV-18, CV-19, and the source half of CV-26.

Required tests:

1. `Item = T` plus conformance-local signature `Self::Item` → source type parameter `T`;
2. sibling RHS `Sequence = List<Self::Element>`;
3. reverse the textual order of `Element`/`Sequence` and prove equivalent plan;
4. direct cycle;
5. indirect cycle;
6. missing sibling binding referenced by a projection reports the correct incomplete/unavailable state rather than `Dynamic`;
7. source projection for a same-spelled associated requirement of another trait is not captured by current name alone.

### Tests to run now

Run exact source-normalization tests by name. After they pass, run only the associated/traits query slice needed for G4.

### Tests explicitly deferred

- exact target normalization;
- source-index projection navigation;
- broad trait capability module;
- core runtime.

### Acceptance

T5 is complete when source conformance signatures and binding RHSs can normalize through their own source associated plan, all LHS identities are established before RHS dependency normalization, cycles are deterministic, and no final-evidence recursion exists.

### Local STOP / CONSULT triggers

- source projection requires final exact evidence to type-check its own conformance;
- the source plan cannot be staged without publishing partially inconsistent canonical products;
- binding cycles cannot be represented through existing completeness/failure architecture without a second proof authority;
- sibling projection semantics appear to require generic trait proof outside the current enclosing conformance.

### Checkpoint update

Required after source normalization becomes a durable P2 interface. Record the three-context authority split: abstract trait, source conformance, exact evidence.

---

## T6 — Implement the canonical projection normalization query and terminal-state algebra

### Purpose

Create the one semantic projection normalizer used by exact evidence, signatures, relations, tooling, and later P3/C6 consumers, with recursive nested normalization and bounded terminal-state behavior.

### Preconditions

- T3 canonical projection type exists.
- T4 abstract formation works.
- T5 source-conformance normalization works without final evidence cycles.
- T1 exact evidence fixed-failure behavior is sound.

### Consumes

- canonical projection key;
- source normalization path;
- `ConformanceIndex::query_exact`;
- `SemanticSnapshot::resolve_conformance_evidence` or its lower-level pure equivalent;
- exact associated binding map;
- `QueryBudget`, `CancellationToken`, `UnknownReason`, `BlockReason`, `DynamicBoundaryObligation`, and existing result patterns;
- canonical type structure/interner.

### Produces

- one canonical projection normalization result algebra;
- one recursive type-normalization entry point;
- abstract/source/exact modes under one authority;
- ambiguity preservation;
- cycle detection;
- cancellation/budget propagation;
- nested canonical rebuilding;
- memoization/query integration where the live DB supports it cleanly.

### Files and symbols

**Primary read/modify:**

```text
phalcom-semantic/src/types/                 # preferred home for pure type-normalization helpers
phalcom-semantic/src/impls.rs               # exact evidence bridge only
phalcom-semantic/src/snapshot.rs            # immutable public exact query facade
phalcom-semantic/src/types/outcome.rs        # reuse/extend state vocabulary
phalcom-semantic/src/types/relation.rs       # safe boundary behavior
phalcom-semantic/src/db/
phalcom-semantic/src/checker/analysis.rs     # dependency category only if needed
```

**Tests:**

```text
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
new focused projection submodule if test organization clearly benefits and semantic README conventions are preserved
```

### Required implementation shape

#### A. Central normalizer

Create a reusable semantic normalization API. A representative shape is:

```rust
pub enum ProjectionNormalizationMode<'a> {
    AbstractTrait {
        trait_ref: &'a TraitRef,
    },
    SourceConformance {
        plan: &'a ConformanceAssociatedTypePlan,
        // source environment/view as needed
    },
    Exact {
        snapshot_or_view: &'a SemanticSnapshot,
    },
}

pub struct ProjectionNormalizationContext<'a> {
    pub mode: ProjectionNormalizationMode<'a>,
    pub budget: &'a mut QueryBudget,
    pub cancel: &'a CancellationToken,
    // active projection stack/set
}
```

Do not force these exact lifetime/field shapes if they fight existing query APIs. The one-authority semantics are fixed.

#### B. Exact projection algorithm

For `AssociatedProjection { subject, trait_ref, requirement }` in exact mode:

```text
1. recursively normalize/materialize subject and trait arguments as appropriate;
2. query exact conformance relationship using existing C4/P1 matching;
3. if no exact proof is available, preserve the precise non-success state;
4. if multiple applicable candidates remain, return Ambiguous with stable candidate identities;
5. if evidence is Proven, lookup requirement in evidence.associated_types;
6. recursively normalize the bound value;
7. detect active-key recursion;
8. return normalized canonical TypeId.
```

Do not query associated bindings by name.

#### C. Abstract algorithm

In abstract trait mode, a canonical projection matching the current trait relationship returns `Symbolic(original_or_structurally_rewritten_projection)`.

Do not turn this into exact conformance lookup.

#### D. Source algorithm

Delegate to or unify with T5's source-plan normalization path. Do not maintain a second source normalizer with subtly different cycle rules.

#### E. Recursive ordinary-type traversal

Normalize nested projections in canonical child positions. Re-intern only changed nodes. Preserve exact case variants and record-row invariants.

Avoid type-graph-wide pre-scans where normal recursive traversal with memoization suffices.

#### F. Result mapping

Map exact conformance results honestly:

```text
ConformanceResolution::Proven        → lookup binding / continue
Incomplete                           → Projection Incomplete
Unknown                              → Unknown
Blocked                              → Blocked
Dynamic                              → Dynamic
Cancelled                            → Cancelled
BudgetExceeded                       → BudgetExceeded
InternalFailure                      → InternalFailure
multiple exact matches/coherence     → Ambiguous
```

Use live enum variants rather than inventing lossy conversions.

#### G. Query/cache ownership

If projection normalization is memoized in DB/snapshot infrastructure, its key must contain stable canonical semantic identity—not source ranges or formatted names.

Do not cache a result without recording the trait surface/conformance/evidence dependencies needed for invalidation.

### Forbidden approaches

- no `Option<TypeId>` API;
- no error string as semantic state;
- no source-name lookup in exact mode;
- no first-match conformance selection;
- no recursion-depth-only cycle handling;
- no `Dynamic` catch-all;
- no runtime fallback;
- no separate normalizer in LSP/compiler;
- no unbounded whole-workspace cache keyed by source text.

### Test changes required

Cover CV-14–CV-23 and CV-27.

Required focused query tests:

- exact success;
- same source generic impl specialized for two targets;
- nested applied/composite success;
- exact enum case preservation;
- direct/indirect cycle;
- ambiguous exact matches preserve ambiguity;
- unknown/blocked and dynamic remain distinct;
- cancellation/budget discriminator using the repository's standard bounded-query test technique;
- symbolic relation safety.

Do not fake inaccessible terminal states by directly constructing impossible internal values if the test framework has a legitimate query-control path.

### Tests to run now

Run exact normalization query tests by name. Use `-- --list` once if module filters are uncertain.

At G4, run the owning `impls::queries` module and focused trait projection tests.

### Tests explicitly deferred

- full source-index/LSP integration;
- P3 Iterable;
- core runtime behavior;
- workspace suite.

### Acceptance

T6 is complete when one semantic normalizer handles abstract/source/exact modes, recursively normalizes nested canonical types, preserves all required terminal states, is cycle/budget/cancel safe, and exact mode consumes only canonical conformance evidence.

### Local STOP / CONSULT triggers

- exact normalization needs a second conformance matching algorithm;
- one of the checkpoint-required terminal states cannot be represented honestly with the existing semantic outcome model;
- cycle detection requires changing canonical projection identity;
- query caching cannot record bounded dependencies without whole-workspace invalidation;
- relation safety would require treating symbolic projection as `Dynamic`.

### Checkpoint update

Required after G4. Record the canonical normalizer interface/result states and dependency owner.

---

## T7 — Integrate normalized projection into exact evidence, signatures, defaults, and witness compatibility

### Purpose

Make projection semantically consequential in the places P2 owns: associated exact bindings, trait requirements/defaults, conformance signatures, and behavioral witness compatibility.

### Preconditions

- T6 normalizer is stable and focused-tested.
- T1 exact resolution cannot erase fixed source failures.

### Consumes

- symbolic trait signatures from T4;
- source conformance plan from T5;
- exact normalizer from T6;
- `TypeView`/conformance environment;
- `InstantiatedTraitRequirement`;
- `check_witness_compatibility*`;
- trait default body checking context;
- exact `ConformanceEvidence` construction.

### Produces

- exact associated binding values that are projection-normalized;
- source/exact projected requirement views;
- trait default checking with abstract associated projections;
- source conformance witness signatures using source associated bindings;
- witness compatibility against normalized parameter/return types;
- honest propagation when normalization is not proven.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
```

### Required implementation shape

#### A. Exact associated binding construction

Replace the P1 conceptual path:

```text
value_template
    → TypeView.materialize
    → ExactAssociatedTypeBinding.value
```

with:

```text
value_template
    → TypeView.materialize structural environment
    → exact projection normalize
    → ExactAssociatedTypeBinding.value
```

If normalization is symbolic in a context that is genuinely exact and exact evidence should resolve it, do not silently publish it. Diagnose/propagate the correct incomplete/blocked/recursive/etc. state.

#### B. Trait requirement instantiation

Do not mutate the abstract `TraitSurface` signature to a concrete value. Produce specialized requirement views:

```text
abstract signature
    + target/trait generic environment
    + associated projection context
    → current requirement view
```

Source conformance view uses source-plan binding authority. Exact view uses exact evidence authority.

#### C. Witness compatibility

Before calling bounded subtype relations on a parameter/return position whose relevant projections can be normalized in the current conformance context, normalize them.

Preserve variance direction already implemented by witness compatibility. P2 changes types, not callable compatibility law.

Do not special-case selector identity or local parameter names.

#### D. Trait default body checking

A trait default may type-check against abstract symbolic `Self::Item`. This means ordinary operations requiring a more concrete capability may still remain unavailable unless separately proven by existing type rules; P2 must not pretend every associated type has arbitrary methods.

Default signature contracts can contain symbolic projections. When a default is selected for an exact conformance, its specialized requirement/environment can normalize those projections for exact checking/lowering consumers as appropriate.

#### E. Conformance-local body checking

A conformance body may use `Self::Item` under the source plan. It must not need an exact monomorphic target when checking a generic source conformance.

#### F. Relation fallback

Add only the relation behavior necessary for safety. The normal architecture is normalize before relation where evidence exists. A remaining symbolic projection is an abstract type atom with canonical identity; equality is sound, but unsupported relational claims should return the repository's uncertain/blocked form rather than a false structural mismatch.

### Forbidden approaches

- no concrete mutation of trait-owned symbolic signatures per conformance;
- no final evidence query from source conformance signature construction;
- no bypass of witness variance/generic compatibility rules;
- no arbitrary method capability on an associated symbolic type;
- no dynamic fallback for relation convenience;
- no compiler-side requirement re-specialization.

### Test changes required

Cover CV-18 if exact evidence cycle propagation differs from source cycle, CV-24–CV-27, and CV-17 exact-case integration.

Required semantic scenarios:

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
}

impl<T> Iterator for List<T> {
  type Item = T
  next -> Option<Self::Item> { ... }
}
```

Prove source requirement view uses `T` and exact view uses exact argument.

Add a parameter-position case so P2 is not accidentally return-only.

Add a trait default whose declared/body-local types use `Self::Item` but do not demand unproven methods.

Add a deliberately incompatible witness where normalization reveals the mismatch; prove it is rejected for the normalized type rather than accepted due to symbolic opacity.

### Tests to run now

Run the exact newly added integration tests first.

At G5 run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries
```

and the focused projection/default/witness filters in `capabilities::traits`.

Do not use the whole trait module result as the only evidence because C5-BL-07 may remain.

### Tests explicitly deferred

- broad `capabilities::traits` reclassification until final stabilization;
- core runtime execution unless source compiler behavior was changed;
- P3 Iterable.

### Acceptance

T7 is complete when exact associated evidence contains normalized exact values, projection-bearing trait/conformance signatures specialize in the correct source/exact mode, defaults remain abstractly valid, and witness compatibility sees normalized types without false proof/refutation.

### Local STOP / CONSULT triggers

- exact evidence construction recurses into itself through normalization;
- trait default checking requires a new associated-type capability/bound system;
- witness compatibility would need a projection-specific variance rule;
- exact evidence must publish symbolic projection to avoid blocking;
- relation changes begin to resemble a second projection solver.

### Checkpoint update

Required after G5. Record that projection-bearing signatures/defaults/witnesses are integrated and exact evidence values are normalized before publication.


---

## T8 — Add projection incrementality, source identity, and minimal tooling projection

### Purpose

Make P2 products durable under add/edit/delete/rename and expose projection occurrences through the existing canonical semantic source-index path without creating an editor-specific associated resolver.

### Preconditions

- T6 provides one normalization authority.
- T7 establishes the projection-bearing semantic products that require invalidation.
- P1 source index already publishes associated declaration/binding target identity.

### Consumes

- semantic DB/query ownership;
- trait/conformance fingerprints;
- `SemanticDependency` and `QueryKey` patterns;
- P1 associated incremental tests;
- source-index associated targets;
- semantic presentation/LSP projection conventions.

### Produces

- precise projection normalization dependencies;
- owner-complete invalidation/removal;
- cold/incremental parity tests;
- `Self::Item` source occurrence → canonical associated requirement target;
- minimal presentation/definition integration through existing consumers;
- no LSP-owned resolution logic.

### Files and symbols

**Read/modify as needed:**

```text
phalcom-semantic/src/db/
phalcom-semantic/src/session.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/source_index/
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/diagnostic_presentation.rs
phalcom-lsp/                            # only consumer paths reached by source-index products
phalcom-semantic/tests/semantic/incremental/associated_types.rs
phalcom-semantic/tests/semantic/incremental/db.rs
phalcom-semantic/tests/semantic/integration/
```

### Required implementation shape

#### A. Dependency precision

Record dependencies at the semantic owner actually consumed:

```text
projection formation
    → TraitSurface(trait declaration)

source normalization
    → source associated plan / source conformance product

exact normalization
    → exact conformance/evidence dependencies

projection source target
    → associated requirement source identity
```

If existing dependency categories already conservatively and correctly cover a product, reuse them. Do not add a new category solely to make the architecture look more granular.

If existing categories are too broad but correct, optimization of invalidation breadth is secondary to correctness. However, do not use whole-workspace invalidation when an existing owner-scoped key is available.

#### B. Fingerprint content

Projection/evidence fingerprints must include canonical semantic inputs that affect meaning:

```text
projection identity
associated declaration shape/ID mapping
source associated binding value/template
relevant exact conformance/evidence identity/value
normalization terminal state when published as a product
```

Do not include incidental source formatting/range unless the product is explicitly source-sensitive.

#### C. Owner-complete removal

Deletion/rename must retire:

- stale associated declaration source target;
- binding plan entries owned by removed source;
- exact evidence derived from removed/changed binding;
- dependent normalized projection results;
- projection occurrence source-index entries;
- presentation/navigation products derived from them.

#### D. Projection source index

When walking a projection syntax node, use the semantic formation result or a canonical occurrence-resolution table to attach:

```text
name_range → SemanticTargetId::AssociatedType(requirement)
```

Do not infer the requirement from text in `source_index/builder.rs`.

If the source-index builder currently runs without access to the needed formation product, add a canonical semantic occurrence product at the semantic layer rather than re-running trait lookup in the source-index layer.

#### E. Minimal LSP/editor adaptation

Definition/navigation should flow through the same semantic target machinery used by P1 associated declarations/binding LHSs.

Hover/presentation may show:

```text
Iterator.Item
associated type
```

and, where the current semantic context already supplies an exact normalization result, an exact meaning such as `Int`.

Do not delay P2 on optional hover polish if definition/source identity and semantic presentation are correct; record polish for P3 if necessary.

### Forbidden approaches

- no LSP-only `Item` resolver;
- no text search over impls/traits;
- no full workspace invalidation on every binding edit;
- no snapshot-local `TypeId` serialized as durable associated identity;
- no stale source target retained after rename/remove;
- no P3 UX redesign.

### Test changes required

Cover CV-28–CV-32.

Required incremental scenarios:

1. `type Item = Int` → `String` changes exact normalized projection and leaves unrelated trait surface stable where the test harness exposes reuse;
2. remove/re-add binding and confirm projection/evidence removal/recovery;
3. rename associated declaration and matching binding; old target/occurrence disappears, new identity/product appears according to owner-relative ID rules;
4. add/remove a projection-bearing trait method and confirm dependent semantic products update;
5. cold vs equivalent incremental sequence yields equivalent normalized projection/evidence/diagnostics;
6. source navigation from `Self::Item` resolves to the trait associated declaration.

Where declaration reorder changes owner-relative IDs by design, assert cold/incremental convergence rather than persistence of an ID that the identity model never promised to preserve across reordering.

### Tests to run now

Run each exact new incremental regression by its actual test name after creation. If the filter is uncertain, list `incremental::associated_types` tests once with `-- --list`, then run the intended non-zero test.

Then run the whole focused associated incremental module at G6:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types
```

Run the exact source-index integration test(s) by name.

### Tests explicitly deferred

- full LSP package unless direct consumer changes cause uncertainty;
- workspace semantic suite;
- core language corpus;
- P3 editor polish.

### Acceptance

T8 is complete when projection results invalidate/recompute owner-completely, cold/incremental parity holds for the P2 mutation matrix, and projection occurrences navigate through canonical associated requirement identity without name solving outside semantic formation.

### Local STOP / CONSULT triggers

- only whole-workspace invalidation appears correct;
- source-index construction cannot consume semantic occurrence identity without independently resolving the trait/name;
- stable publication requires serializing snapshot-local projection identity as durable identity;
- cold/incremental parity differs after one serious local correction.

### Checkpoint update

Required after G6. Record projection dependency ownership, source target integration, and actual cold/incremental evidence.

---

## T9 — Run the focused P2 interaction matrix and stabilize affected crates

### Purpose

Prove that the complete P2 architecture behaves coherently across syntax, conformance evidence, projection formation/normalization, signatures/defaults/witnesses, incrementality, and semantic tooling while classifying inherited failures instead of repairing the world.

### Preconditions

- T1–T8 implemented.
- all task-local exact regressions are green or classified through an accepted consultation.

### Consumes

- complete P2 implementation;
- coverage ledger CV-01–CV-36;
- baseline ledger;
- repository testing conventions.

### Produces

- focused P2 verification record;
- coverage obligation mapping;
- classification of any residual failure;
- affected-crate compile proof;
- evidence that no VM/runtime projection solver was introduced.

### Files and symbols

**Tests/review:**

```text
phalcom-ast projection syntax tests
phalcom-semantic traits/projection exact tests
phalcom-semantic impls::queries
phalcom-semantic incremental::associated_types
relevant source-index integration tests
changed metadata/export/type tests
```

**Documentation state:**

```text
LANG005.C5-CHECKPOINT.md verification/deferred ledgers
```

### Required implementation shape

No new architecture should normally be introduced in T9.

Perform a focused interaction review against the coverage ledger. For every coverage ID, identify the test that proves it or explicitly mark why it is structurally covered by another test. Missing semantic coverage is a test gap; do not paper over it with a broad workspace run.

Review the diff for anti-patterns:

```text
name-based associated lookup outside contextual formation
ConformanceIndex use inside TypeEnvironment/substitution
projection-solving logic duplicated in LSP/compiler
Dynamic used as uncertainty fallback
wildcard TypeData arms hiding projection behavior
new per-exact ImplId
second completeness boolean
unbounded recursive normalization
source-order binding semantics
visibility/private-authority drift
P3 Iterable migration accidentally begun
```

Search for newly introduced strings/helpers if useful; code review is part of stabilization evidence.

### Forbidden approaches

- no broad cleanup while stabilizing;
- no changing expected semantics solely because a test is inconvenient;
- no workspace tests as substitute for missing focused tests;
- no fixing C5-BL-01…07 unless reclassified A/B;
- no P3 implementation.

### Test changes required

Only add tests needed to close a specific uncovered `CV-*` obligation or reproduce an A/B failure.

### Tests to run now

Run the named G7 final focused gate defined below.

### Tests explicitly deferred

- workspace all-targets tests;
- workspace clippy;
- release certification;
- full Iterable migration/corpus;
- unrelated baseline suites.

### Acceptance

T9 is complete when every P2 coverage obligation has focused evidence, all A/B failures are fixed, C/D failures are recorded, affected crates compile, and no architectural anti-pattern remains in the scoped diff.

### Local STOP / CONSULT triggers

- focused tests reveal a contradiction among abstract/source/exact normalization authority;
- the only fix for a failing interaction is duplicate solving in a consumer;
- a terminal-state test requires changing the accepted proof-state model;
- patch scope becomes materially larger than the impact map.

### Checkpoint update

Required. Record G7 commands/results, coverage status, residual failures, and actual verification classification.

---

## T10 — Synchronize specification/state and produce P3 handoff

### Purpose

Make the repository records accurately describe the P2 architecture actually implemented and give P3 a compact, reliable takeover package.

### Preconditions

- T9 focused stabilization complete.
- no unresolved mandatory consultation incident.

### Consumes

- final implementation/diff;
- actual test evidence;
- consultation/amendment records;
- residual failure classifications.

### Produces

- updated trait specification for P2 semantics;
- updated C5 checkpoint/guidance;
- `LANG005.C5.P2-walkthrough.md`;
- `LANG005.C5.P2-handoff.md`;
- truthful plan/checkpoint metadata;
- `next_plan: LANG005.C5.P3`.

### Files and symbols

**Modify:**

```text
docs/specs/objects/traits.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
this plan metadata only if actual lifecycle state is recorded in-repo by convention
```

**Create:**

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-handoff.md
```

### Required implementation shape

#### A. Trait specification

Document the implemented P2 rules without over-specifying unimplemented future syntax:

- contextual `Self::Item`;
- projection identity's trait-owned requirement basis;
- abstract symbolic projection;
- source/exact normalization authority;
- recursive/ambiguous/non-success state behavior at the semantic level;
- no generic `T::Item` assumption semantics yet;
- no runtime solver.

Do not add `<T as Trait>::Item` to normative examples unless a separate consultation ratified it.

Do not use this task to amend the deferred visibility/private-authority rules; the user explicitly parked that design lane for later.

#### B. Checkpoint

Record:

```text
P2 plan completion/verification
P2 starting and ending revision
canonical projection identity/carrier
normalization authority/result model
P1 completeness correction
implemented incremental/source-index invariants
actual gate evidence
consultation decisions
residual failures
next action P3
```

If P2 did not achieve every acceptance criterion, do not mark it complete merely because code exists. Create a bounded follow-up plan inside C5 if necessary.

#### C. Guidance

Update only durable guidance P3/C6 must know. Avoid duplicating the walkthrough.

#### D. Walkthrough/handoff

Follow §§23–24 below exactly.

### Forbidden approaches

- no claiming unexecuted gates passed;
- no claiming workspace/release certification;
- no historical rewrite of P1 evidence;
- no future syntax documented as implemented;
- no new feature code except a documentation-discovered trivial correction that is covered by existing focused tests; material changes return to T9/G7.

### Test changes required

None merely for documentation. If the T10 review exposes an uncovered P2 invariant, return to the owning implementation task/T9, add the focused regression there, and rerun the corresponding gate before completing records.

### Tests to run now

No new tests merely because documentation changed.

If a code correction was made in T10, rerun only the smallest relevant G7 discriminator.

### Tests explicitly deferred

All broad certification remains P3/checkpoint closure work.

### Acceptance

T10 is complete when repository documentation/state matches actual implementation/evidence, the walkthrough/handoff exist, and P3 can start without reconstructing P2 from conversation history.

### Local STOP / CONSULT triggers

- documentation reveals an unresolved architecture contradiction;
- P2 acceptance cannot be stated truthfully without unimplemented work;
- P3 would need to redesign a P2 identity/authority instead of consuming it.

### Checkpoint update

Mandatory final P2 update.

---

# 19. Verification gates

## G0 — P2 takeover baseline

**Purpose:** prove the pushed P1 state is usable and classify the known trait baseline once.

Run serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-core \
  -p phalcom-lsp

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic capabilities::traits
```

**Expected:** affected crates compile; P1 query/incremental focused suites are green; the trait module either reproduces the exact recorded C5-BL-07 failures or provides new evidence that must be classified.

Do not broaden after G0.

---

## G1 — Fixed-failure conformance soundness

**Purpose:** prove exact conditional witness resolution cannot erase fixed associated failure.

Run the exact T1 regressions and the existing associated-completeness regression. Then, because shared exact conformance resolution changed, run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries
```

**Pass means:** `ConformanceResolution::Proven` is unreachable when fixed associated/source invalidity remains, even if every deferred behavioral selection becomes applicable.

Do not begin projection implementation until G1 passes or an accepted consultation amends the plan.

---

## G2 — Syntax and canonical projection carrier

**Purpose:** prove `Self::Item` has lossless syntax and one canonical structural type representation.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-ast --test trait_syntax

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

Then run the exact canonical projection structural tests added by T3.

If the AST implementation tests live in a different existing integration target, use the verified non-zero equivalent and record it in the walkthrough.

**Pass means:** AST + canonical type carrier are coherent; structural visitors compile; no normalizer is required to make the type graph valid.

---

## G3 — Abstract trait projection formation

**Purpose:** prove trait-owned contextual formation and source-order independence.

Run exact filters covering:

```text
Self::Item abstract formation
associated declaration before/after use equivalence
same-name associated requirements in two traits
generic trait application context
unknown Self::Missing diagnostic
```

Then run the smallest owning trait test slice containing those tests.

**Pass means:** trait context, not global spelling, determines projection identity; abstract projections remain symbolic.

---

## G4 — Source/exact normalization and terminal states

**Purpose:** prove the three normalization modes and bounded recursive query behavior.

Run exact source normalization tests, exact conformance projection queries, cycles, ambiguity, unknown/blocked/dynamic discriminators, and bounded-query tests. Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries
```

**Pass means:** source plans do not depend on final evidence, exact mode uses exact evidence, and terminal states remain distinct.

---

## G5 — Signatures/defaults/witness integration

**Purpose:** prove projection-bearing contracts are usable by real conformance compatibility rather than existing only as standalone query values.

Run focused filters covering:

```text
projection in return type
projection in parameter type
source conformance witness using Self::Item
trait default with abstract Self::Item
exact generic specialization
exact enum-case specialization
incompatible normalized witness rejection
```

Then rerun `impls::queries` only if G4 did not already run after the final T7 changes.

**Pass means:** abstract/source/exact projected types reach witness/default checking at the correct phase and no evidence cycle appears.

---

## G6 — Incremental and semantic tooling

**Purpose:** prove owner-complete lifecycle and canonical source identity.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types
```

plus exact source-index/navigation tests added/affected by T8.

Run `incremental::db` only if T8 changes shared DB invalidation primitives rather than merely adding associated projection dependencies:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::db
```

**Pass means:** binding/declaration/projection changes invalidate the right products, cold parity holds, and source navigation uses canonical associated identity.

---

## G7 — Final focused P2 stabilization

**Purpose:** prove the coherent P2 feature, not the entire repository.

Run serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-core \
  -p phalcom-lsp

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-ast --test trait_syntax

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types
```

Then run the exact focused projection/default/witness/source-index filters needed to cover CV-01–CV-35 if they are not already included by the module commands above.

Finally rerun:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic capabilities::traits
```

**Interpretation:** all P2 tests must pass. If the only failures are the same verified C5-BL-07 failures with the same unrelated cause, record them as D and permit `FOCUSED_TESTED`. Any new failure is classified before editing.

Do not automatically run workspace tests/clippy after G7.

---

# 20. Final focused acceptance

P2 may be recorded `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only when:

- T1's exact-evidence soundness regression passes;
- projection syntax and canonical `TypeId` identity are implemented;
- abstract/source/exact normalization modes are implemented;
- nested normalization and cycle detection are focused-tested;
- required terminal states remain distinct;
- exact associated evidence publishes normalized exact values or propagates non-success honestly;
- projection-bearing signatures/defaults/witness compatibility are integrated;
- exact enum-case preservation has evidence;
- incremental binding/declaration/projection lifecycle has cold-parity evidence;
- source navigation targets canonical associated requirement identity;
- affected crates compile;
- no runtime/LSP duplicate projection solver exists;
- C5-BL-07 and other residual failures are classified truthfully;
- checkpoint/walkthrough/handoff are current.

The walkthrough must reproduce the §15.1 coverage ledger as an evidence table, with every `CV-01` through `CV-36` row mapped to the actual test/evidence and actual result or classification. No coverage row may be omitted or represented by an ellipsis.

P2 does **not** require workspace/release certification to reach `FOCUSED_TESTED`.

---

# 21. Performance / resource evidence

P2 is primarily a correctness plan, not a performance plan. Do not add benchmark ceremony.

The resource requirements are architectural:

1. normalization must be cycle/budget/cancellation bounded;
2. repeated exact projection queries should use existing query/cache infrastructure where semantically safe;
3. recursive nested normalization should reuse original interned `TypeId`s when unchanged;
4. do not scan all traits/conformances by name for each projection;
5. do not trigger whole-workspace invalidation for a local associated binding edit when owner-scoped dependency machinery can represent the change.

Acceptable evidence is code-path/query inspection plus focused tests that prove bounded behavior and invalidation reuse. No microbenchmark is required.

If profiling later identifies projection normalization as hot, optimize under a separate evidence-driven task; do not pre-emptively complicate identity or proof semantics.

---

# 22. Checkpoint bookkeeping

## 22.1 At plan start

Update `LANG005.C5-CHECKPOINT.md` to:

- set `active_plan: LANG005.C5.P2`;
- record P2 starting revision;
- retain P1 as latest completed predecessor;
- retain C5 checkpoint `IN_PROGRESS / PARTIAL`;
- record G0 evidence/classification.

Do not erase the historical P1 planning baseline; distinguish it from the P2 live takeover anchor.

## 22.2 During plan

Update the checkpoint only for durable events:

- T1 soundness invariant landed;
- canonical projection identity/carrier established;
- normalization authority/result model established;
- consultation/amendment;
- new deferred/baseline issue;
- G1–G7 coherent verification result;
- durable source/incremental interface.

Do not turn the checkpoint into a chronological command log.

## 22.3 At plan completion

Record:

```text
plan status/completion/verification
starting and ending revision
implemented invariants/interfaces
actual gate commands/results
consultations/amendments
deferred failures
coverage obligations
next action: LANG005.C5.P3
```

If P2 is incomplete, keep `active_plan`/next action truthful or create a bounded corrective P2 follow-up plan inside C5 rather than falsely closing it.

---

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-walkthrough.md
```

It must contain:

1. final lifecycle classification;
2. start/end revision;
3. architecture actually implemented;
4. exact canonical projection representation;
5. exact normalization result/authority model;
6. T1 P1-soundness correction;
7. important files/symbols changed;
8. durable interfaces inherited by P3/C6;
9. deviations from this plan and why;
10. consultations and resulting decisions;
11. tests added;
12. tests actually run, with command/result;
13. tests deliberately deferred;
14. complete CV-01–CV-36 mapping;
15. baseline/residual failures;
16. performance/resource observations if any;
17. next required work.

Do not claim a gate passed without executed evidence.

---

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-handoff.md
```

It must give the P3 implementer:

```text
current reliable revision/state anchor
P1 + P2 stable identities
canonical projection carrier/key
normalization query/result API
abstract/source/exact authority split
exact associated binding/evidence behavior
projection-bearing signature/default/witness interfaces
incremental/source-index ownership
known baseline failures
consultation decisions
must-read files
first recommended P3 commands
things not to redesign
```

The handoff must explicitly state:

- P3 may migrate `Iterable` using P2 projection semantics;
- P3 must not move generic `T: Trait` proof backward from C6;
- compiler/runtime still do not solve associated projection;
- visibility/private-authority redesign remains outside this P2 handoff unless separately ratified later.

---

# 25. Completion truth table

Do not conflate these states:

| Statement | What it means | Sufficient for P2 COMPLETE? |
|---|---|---|
| Source written | code edits exist | No |
| Tests added | regressions exist | No |
| Exact task tests pass | one slice works | No |
| G1–G6 pass | coherent semantic slices proven | Not alone |
| G7 final focused gate complete | P2 focused behavior proven | Yes, with records current |
| Affected crates check | compile integration works | Required, not sufficient alone |
| `capabilities::traits` still has only classified C5-BL-07 | known baseline remains | Compatible with FOCUSED_TESTED |
| Workspace green | broad unrelated state | Not required for P2 |
| Release certified | checkpoint/release gate | Not claimed by P2 |
| C5 complete | P1+P2+P3 acceptance all done | No; P3 remains |

Use repository metadata truthfully.

---

# 26. Plan self-review

This plan was reviewed against the Luna patch-grade schema and Phalcom principal-architect doctrine.

## 26.1 Architecture review

- one owner exists for associated identity: `AssociatedTypeRequirementId`;
- one canonical projection carrier exists: `TypeId` type graph;
- one source normalization authority exists: `ConformanceAssociatedTypePlan`;
- one exact normalization authority exists: exact conformance evidence;
- one final conformance completeness authority remains;
- structural substitution is separated from semantic solving;
- compiler/runtime/LSP do not reconstruct canonical meaning;
- abstract trait context does not require fictitious evidence;
- exact enum identity is preserved;
- C6 proof boundary is explicit.

## 26.2 Luna executability review

- FIXED/FLEXIBLE/VERIFY-FIRST decisions are explicit;
- each task has preconditions, inputs, outputs, files, forbidden approaches, tests, acceptance, and local escalation triggers;
- the confirmed P1 soundness defect has an exact entry-gate regression;
- likely architecture forks are pre-decided;
- visibility/private-authority work is explicitly deferred instead of left as an accidental fork;
- semantic debugging is bounded to one serious correction;
- architectural failure permits zero speculative fixes.

## 26.3 Testing review

- coverage spans success, negative behavior, identity, ordering, recursion, ambiguity, proof states, exact generic/case specialization, witness integration, incrementality, diagnostics/source tooling, and authority boundaries;
- execution breadth is intentionally smaller than designed coverage;
- broad workspace gates are explicitly deferred;
- C5-BL-07 receives one fresh classification, not reflexive repair;
- P3 runtime/core migration tests are not pulled into P2.

## 26.4 Documentation lifecycle review

- checkpoint start/during/end updates are required;
- walkthrough and handoff are required;
- T/G terminology is consistent;
- P3 next boundary is explicit;
- historical P1 records are not silently rewritten.

## 26.5 Premium-compute review

Likely adviser questions have been pre-solved as fixed architecture. If escalation occurs, Luna should send only the relevant incident packet: invariant, exact failing test, call path, scoped diff, evidence, and decision question. Do not ask an adviser to rediscover the entire repository.

No executable command in this plan depends on a fabricated test name. Where a task instructs Luna to run a newly-created test by name, Luna must use the actual test name and verify a non-zero selection before treating the result as evidence.

---

# 27. Residual risks and explicit deferred decisions

## 27.1 Type-store cross-module dependency risk

The preferred `TypeData::AssociatedProjection` references `TraitRef` and `AssociatedTypeRequirementId`, which currently live outside `types/store.rs`. Rust module references may be mechanically straightforward, but if the dependency graph becomes awkward, Luna may introduce a private interned projection record/ID or move an identity-only data type to an existing neutral semantic identity module **without changing canonical meaning**. If resolving the dependency would change semantic ownership or create a second projection identity, consult.

## 27.2 Metadata/export risk

Projection-bearing signatures may cross snapshot/metadata publication boundaries. The live metadata architecture must be checked. A stable exported representation may need to carry stable subject/trait/requirement identity. This is allowed if it is a translation of the canonical semantic projection, not a new authority. If current metadata cannot represent it without inventing independent associated identity semantics, consult.

## 27.3 Relation/inference risk

The new canonical type form will reach exhaustive relation/inference code. The default architecture is normalization before definitive relation checks and honest symbolic fallback. If inference appears to require decomposing associated projection as an inference equation system, that is more than P2's planned architecture and requires consultation.

## 27.4 Baseline coupling risk

P2 touches trait signatures/defaults, so C5-BL-07 might become causally coupled even though P1 classified it D. G0/G7 deliberately reclassify it. Do not preserve the D label against contrary live evidence.

## 27.5 Visibility/private-authority deferral

The repository still implements the P1 rule that conformance bodies do not gain target-private class authority and conformance-local `via` is rejected. A later design session may change those semantics. The user explicitly deferred that topic while returning to P1/P2. P2 must neither entrench the old rule into projection identity nor attempt to solve the deferred design.

## 27.6 P3 boundary

P2 deliberately stops before migrating core `Iterable`. P3 should be able to perform that migration primarily by consuming the canonical projection/evidence machinery established here. If P3 would have to redesign projection identity, normalization authority, or exact evidence semantics, P2 is not actually complete and should not be closed.

---

# 28. Compact execution map

```text
T0  re-ground pushed P1 state + baseline
    ↓ G0
T1  fix fixed-failure/deferred-evidence soundness
    ↓ G1
T2  add lossless Self::Item syntax
T3  add canonical TypeId projection + structural plumbing
    ↓ G2
T4  two-phase trait formation + abstract symbolic projection
    ↓ G3
T5  source conformance binding graph + source normalization
T6  canonical normalizer + exact mode + terminal states
    ↓ G4
T7  exact evidence + signatures/defaults/witness integration
    ↓ G5
T8  incrementality + source identity + minimal tooling
    ↓ G6
T9  focused interaction stabilization
    ↓ G7
T10 specs/checkpoint/walkthrough/handoff
    ↓
LANG005.C5.P3
```

The critical dependency chain is:

```text
sound evidence
    → canonical projection identity
    → abstract/source formation
    → exact normalization
    → signature/witness integration
    → incremental/tooling publication
```

Do not reorder that chain for convenience.
