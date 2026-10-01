---
id: LANG005.C5.P3
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C5.P2
follows: LANG005.C5.P2
supersedes: null
prepared: 2026-09-16
repository: aureat/phalcom-lang
planning_revision: e3e3813720c200e277a9414857536f3e2e026054
---

# LANG005.C5.P3 — Associated-Type Vertical Integration, Iterator Substrate, and C5 Certification

> **For Luna / agentic implementers:** Execute this plan task-by-task under the repository's Luna patch-grade workflow. Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` only when the execution environment supports them. Do not reinterpret this plan as permission to perform the C6 iterator migration.

**Goal:** Complete and certify associated types as a cross-stack Phalcom capability, prove the ratified stateful iterator architecture with concrete/exact conformances, close remaining C5-owned compiler/runtime/module/tooling/incremental gaps, and hand the general iterator migration to C6 without introducing generic trait-bound evidence early.

**Architecture:** P3 consumes the frozen P1/P2 semantic authority—trait-owned `AssociatedTypeRequirementId`, source binding plans, exact `ConformanceEvidence`, canonical `TypeData::AssociatedProjection`, and `types::normalize_type`—and tests those facts through real compiler/runtime, multi-module, incremental, metadata, and LSP boundaries. The ratified `Iterator`/`Iterable` architecture is represented and executed only through concrete fixture conformances because the general relationship `Self::Iterator |= Iterator` plus associated equality belongs to C6. Production `for`, the legacy cursor protocol, the runtime `Iterable` superclass, and production collection/lazy-iterator migration remain unchanged in P3.

**Tech stack:** Rust workspace (`phalcom-ast`, `phalcom-semantic`, `phalcom-type-meta`, `phalcom-core`, `phalcom-lsp`, module/session infrastructure), Phalcom source fixtures, Cargo integration tests, semantic DB/incremental infrastructure, stable semantic metadata.

**Primary specification/state:**

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-handoff.md
docs/specs/objects/traits.md
```

**Ratified design amendment for this plan:** C5.P3 no longer owns the definitive production iteration migration. It owns C5 closure plus a C6-ready concrete iterator substrate/reference model. C6 owns generic conformance assumptions, nested evidence, associated equality constraints, the general `Iterator ⇒ Iterable` relationship, and the production `for`/core-collection/cursor-protocol migration.

---

## 0. Executor contract

This plan is designed for Luna-style execution under constrained architectural authority.

The implementer:

- may adapt mechanical helper names and file-local decomposition to the live repository;
- must preserve the semantic ownership and checkpoint boundary in this plan;
- must not improvise generic trait evidence or iteration migration architecture;
- must use the smallest discriminating tests during BUILD mode;
- must classify failures before editing;
- must keep known baselines separate from P3-owned failures;
- must STOP AND CONSULT when a listed trigger fires;
- must keep the C5 checkpoint/guidance synchronized at durable gates;
- must produce the required P3 walkthrough and C6 handoff;
- must not commit/push unless the surrounding implementation workflow explicitly instructs it.

Testing is evidence gathering, not ritual. A test is run because it discriminates an invariant, validates a durable interface, or satisfies a named gate—not because a file changed.

The plan intentionally avoids edit-sized microtasks. Each T-task is a coherent reviewer boundary with its own acceptance and focused evidence.

---

# 1. Goal

Deliver `LANG005.C5.P3` as the closing C5 implementation plan by proving that the associated-type architecture implemented in P1/P2 survives real compiler/runtime execution, stable/multi-module semantic boundaries, incremental replacement/removal, real LSP consumers, exact generic specialization, exact enum-case identity, bound/detached callable lifetime, labeled tuple associated values, and a high-density stress corpus.

At the same time, establish an executable concrete reference model for the ratified future iterator API:

```phalcom
trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }
}

trait Iterable {
  type Item
  type Iterator

  iterator -> Self::Iterator
}
```

without claiming or implementing the generic theorem:

```text
Self::Iterator conforms to Iterator
Self::Iterator::Item = Self::Item
```

and without introducing any future C6 bound syntax.

---

# 2. Checkpoint acceptance objective

This is the **full closure plan for LANG005.C5**, but only after amending the checkpoint to reflect the ratified C5/C6 boundary.

C5 acceptance after P3 means Phalcom has a complete associated-type semantic capability for **declared/source and exact conformance evidence**:

```text
trait associated declaration
    ↓
AssociatedTypeRequirementId
    ↓
source associated binding plan
    ↓
exact ConformanceEvidence
    ↓
canonical AssociatedProjection
    ↓
abstract/source/exact normalization
    ↓
projection-bearing callable contracts
    ↓
semantic lowering / stable metadata as actually required
    ↓
compiler/runtime execution
    ↓
source index / real LSP / incremental lifecycle
```

It does **not** mean generic code can assume arbitrary trait evidence. That is C6.

P3 therefore closes C5 by proving all C5-owned authority and integration while leaving the following migration to C6:

```text
generic conformance constraints (`|=`)
nested generic conformance evidence
associated equality constraints in `where`
`Trait<Assoc = T>` refinement syntax
conditional/derived conformance
general Iterator ⇒ Iterable evidence
Iterable.Iterator ⇒ Iterator evidence
Iterable.Iterator.Item = Iterable.Item evidence
production `for` checker migration
production `for` compiler lowering migration
runtime/core Iterable superclass retirement
cursor protocol retirement
production collection and lazy iterator migration
production Map entry-iteration semantic change
```

The checkpoint may close as `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` even if broad workspace/release baselines remain, provided all C5-owned A/B failures are fixed, C/D failures are truthfully classified, and the focused certification matrix passes.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   e3e3813720c200e277a9414857536f3e2e026054
commit:     feat(semantic): implement associated type projections
relevant predecessor: LANG005.C5.P2 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

Verified planning facts from the pushed tree:

1. P2 is landed and the checkpoint currently marks `active_plan: LANG005.C5.P3` and `latest_completed_plan: LANG005.C5.P2`.
2. `TypeData::AssociatedProjection` is the canonical type carrier; `types::normalize_type` is the sole recursive associated-projection normalization/cycle authority.
3. `ConformanceAssociatedTypePlan` owns source binding normalization; `ConformanceEvidence.associated_types` and exact requirement views own exact normalized meaning.
4. P2 semantic gates passed for AST projection syntax, impl queries, associated incremental lifecycle, semantic editor/source-index integration, and projection normalization; two unrelated Universe `Bool` capability tests remain `C5-BL-07`.
5. The current production iteration implementation is still the normative cursor model: runtime `class Iterable<T>`, `iterate(_)`, `iteratorValue(_)`, stateless cursor pipeline classes, structural `for` semantic analysis, and cursor bytecode lowering.
6. `../../../__unstructured__/spec/current/iteration.md` truthfully describes the current cursor implementation and must not be rewritten to claim the stateful iterator model is already production behavior.
7. `docs/implementation/LANG005/support/iterable.ph` and `core_trait_impls.ph` still encode the now-superseded `Cursor`-associated design and current Map-key iteration assumption; they need revision as future migration support.
8. Production `Map<K,V>` currently inherits `Iterable` and `iteratorValue(_)` returns keys; `.entries` creates separate entry objects. P3 must not change production Map iteration because the definitive iteration migration is deferred to C6.
9. `phalcom-semantic/src/metadata/export.rs` currently rejects `TypeData::AssociatedProjection(_)` as `MetadataExportError::NonExportableForm`.
10. `phalcom-type-meta::TypeNode` has no associated-projection stable node today. This is a verify-first integration seam, not automatic proof that metadata must change.
11. P2 proved semantic editor/source-index convergence through `SemanticTargetId::AssociatedType`, but full `phalcom-lsp` packaging was explicitly deferred.
12. `phalcom-lsp/src/backend.rs` contains target-specific presentation/navigation paths; at least one consumer explicitly matches `SemanticTargetId::Callable`, so associated-target behavior must be tested rather than assumed.
13. `phalcom-core/tests/core/language/traits_p3_closure.rs` is the existing C4/P1 executable trait vertical and is the natural location for small C5 runtime tests unless a dedicated `traits_c5_closure.rs` keeps the module clearer.
14. Current `for` compiler lowering in `phalcom-core/src/compiler/lib/loops.rs` is deeply coupled to cursor behavior, multi-lane patterns, indices, break/continue, and closure-capture semantics. It is intentionally not a P3 migration target.
15. The repository supports labeled tuple type/product shapes. P3 can therefore certify an associated binding shaped `(key: K, value: V)` without inventing a nominal entry type.

Before editing, re-run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then re-read the named live paths. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

# 4. Required reads before implementation

Read in this order. Do not begin production edits from this plan alone.

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
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-associated-projection-formation-normalization-and-evidence-hardening-plan.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-handoff.md
```

## 4.3 Normative language/spec authority

```text
docs/specs/objects/traits.md
docs/spec/current/iteration.md
docs/spec/current/syntax/expressions.md
docs/specs/callables/arguments.md
relevant tuple/product type specification currently authoritative in the tree
relevant exact enum-case identity specification
```

`../../../__unstructured__/spec/current/iteration.md` describes current implemented iteration, not the newly ratified future target. Preserve that distinction.

## 4.4 Ratified future iterator support records

```text
docs/implementation/LANG005/support/iterable.ph
docs/implementation/LANG005/support/core_trait_impls.ph
```

These files are stale at takeover and are expected to change in P3.

## 4.5 Semantic architecture

```text
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/source_index/
phalcom-semantic/src/presentation.rs
```

## 4.6 Stable metadata

```text
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/bundle.rs
phalcom-type-meta/src/header.rs
stable metadata import/materialization path discovered from live references
```

## 4.7 Compiler/runtime trait execution

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/vm/associated.rs
phalcom-core/src/frame.rs
phalcom-core/src/heap/block.rs
phalcom-core/tests/core/language/traits_p3_closure.rs
phalcom-core/tests/README.md
```

Read generic runtime type-environment code actually used by trait invocation before changing it. Do not infer its current file from historical plans.

## 4.8 Incremental/module/editor/LSP

```text
phalcom-semantic/tests/semantic/README.md
phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md
phalcom-semantic/tests/semantic/incremental/associated_types.rs
phalcom-semantic/tests/semantic/integration/editor.rs
phalcom-semantic/tests/semantic/integration/compiler_capabilities.rs
phalcom-lsp/src/backend.rs
phalcom-lsp/tests/impl_navigation.rs
phalcom-lsp/tests/imported_binding_resolution.rs
other focused definition/hover fixture tests discovered from the live tree
```

Avoid broad exploration unless one of the verify-first assumptions fails.

---

# 5. Normative authority

## 5.1 Frozen C5 associated-type behavior

P3 inherits these as normative/frozen:

```text
AssociatedTypeRequirementId is trait-owned stable requirement identity.
Same spelling in different traits is distinct.
Names/ranges are not associated-type identity.
Source associated binding plan is ImplId-owned source authority.
Exact associated values live in ConformanceEvidence.
Behavioral + associated failures share one ConformanceCompleteness authority.
Self::Item is contextual projection syntax in C5.
TypeData::AssociatedProjection is the canonical semantic projection form.
types::normalize_type is the sole recursive normalization/cycle authority.
Abstract/source/exact projection states remain distinct.
Structural substitution rewrites constituents; it does not prove conformance.
Compiler/runtime/LSP do not rediscover associated meaning.
Exact enum-case identity must remain exact through conformance/projection.
```

## 5.2 Ratified iterator target behavior

The target post-C6 iterator architecture is fixed for P3 planning:

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
  iterator { self }
}

trait Iterable {
  type Item
  type Iterator
  iterator -> Self::Iterator
}
```

Ratified semantic laws:

```text
`next` is getter-shaped.
`iterator` is getter-shaped.
Iterator is single-pass stateful traversal.
Iterable is a traversal source.
Collection/source Iterable.iterator creates a fresh independent iterator.
Iterator.iterator returns the same traversal object (`self`).
Every iterator is conceptually iterable; not every iterable is an iterator.
next returns Option<Item> directly.
None means permanent exhaustion; later next reads remain None.
Iteration errors use normal raise/error propagation.
No public cursor, hasNext, reset, clone requirement, or iterator-specific mutability marker.
Map's target Item is a labeled tuple `(key: K, value: V)`.
Tuple target Item may remain Dynamic initially.
Range's target design uses a stateful RangeIterator.
Lazy adapters target stateful iterator composition.
```

## 5.3 Ratified C6 evidence/bound boundary

P3 must not implement any of:

```phalcom
T |= Iterable
T |= Iterable + Debug
T |= Iterator<Item = A>
T::Item = A
```

in generic `where` clauses.

Future semantics:

```text
`|=` is the conformance relation.
`T |= A + B` is sugar for two independent conformance constraints.
Associated refinement/equality is evidence attached to conformance, not a new trait identity.
Generic trait constraints live in `where`, not in generic binder `T: Trait` syntax.
```

These are C6 authority and are documented in the P3 handoff, not implemented here.

## 5.4 Current production iteration authority

Until C6 migration lands, current production behavior remains:

```text
runtime class Iterable<T>
iterate(_)
iteratorValue(_)
structural current `for` semantics
cursor compiler lowering
cursor-based lazy pipeline
Map direct iteration yields keys
```

P3 may add regression evidence but must not change these production semantics.

## 5.5 Authority ordering during P3

When sources disagree:

```text
1. explicit ratified decisions in this plan/checkpoint amendment
2. current C5 trait specification and accepted checkpoint state
3. completed P2 walkthrough/handoff
4. live repository architecture for mechanics
5. current production iteration spec for current behavior only
6. historical LANG005 support fixtures
```

The old cursor-based LANG005 support fixture is superseded by the ratified iterator target and must not overrule it.

---

# 6. Takeover state

## 6.1 Stable interface map

| Concept | Current symbol/path | Canonical owner | P3 invariant |
|---|---|---|---|
| Behavioral requirement identity | `TraitRequirementId` | trait semantic surface | Remains behavior-only |
| Associated requirement identity | `AssociatedTypeRequirementId` | trait semantic surface | Never reconstruct from string name |
| Source conformance identity | `ImplId` | conformance source declaration | Not exact applied conformance |
| Exact trait application | `TraitRef` + exact target | semantic conformance engine | Exact target/trait application selects evidence |
| Source associated bindings | `ConformanceAssociatedTypePlan` | impl/conformance semantic plan | Source mode never requests final evidence |
| Exact associated values | `ConformanceEvidence.associated_types` | exact conformance evidence | Runtime consumers never recompute |
| Exact projected requirement views | `ConformanceEvidence.requirement_views` | exact evidence | Witness/default consumers use normalized contracts |
| Projection carrier | `TypeData::AssociatedProjection` | semantic type store | One canonical type graph form |
| Projection normalization | `types::normalize_type` / `ProjectionNormalizationContext` | semantic type subsystem | Sole recursive/cycle authority |
| Source navigation identity | `SemanticTargetId::AssociatedType` | semantic source index | LSP consumes; does not resolve names |
| Runtime trait selection | semantic lowering + executable conformance plan | semantic/compiler boundary | VM executes selected method; no proof search |
| Runtime generic types | `RuntimeTypeRecipe` / runtime type environment | compiler/runtime reification | Receives normalized types, not projections to solve |
| Current `for` lowering | `phalcom-core/src/compiler/lib/loops.rs` | current compiler iteration implementation | Transitional cursor behavior; no P3 migration |
| Current iteration semantic checking | `resolve_iteration_element_application` path | semantic statement checker | Transitional cursor behavior; no P3 migration |
| Stable type metadata | `phalcom-type-meta::TypeNode` + semantic exporter | metadata subsystem | Verify projection transport requirement before extending |

## 6.2 P2 verification inherited

P2 recorded:

```text
affected AST/semantic/core/LSP crates compile
AST trait syntax: 8/8
semantic impls::queries: 55/55
associated incremental: 5/5
semantic integration::editor: 11/11
projection normalization discriminator: 6/6
trait capability slice: 36/38
    only C5-BL-07 two Universe Bool failures remain
```

P3 does not rerun all of this after every task. It reruns only changed seams and final gates.

## 6.3 Confirmed P3 gaps at takeover

```text
no executable core associated-projection runtime vertical recorded
no generic multi-exact-target associated runtime proof
no exact enum-case associated runtime proof
no associated-type bound/detached-callable+GC runtime proof
no full LSP packaging proof for AssociatedType targets
no cross-module associated-type consumer certification
no cross-module cold/incremental associated parity certification
stable metadata exporter rejects raw AssociatedProjection
stale iterator support fixtures encode Cursor model
checkpoint/guidance still describe full Iterable migration as P3-owned
```

These are the primary P3 work items.

---

# 7. Architecture

## 7.1 Authority flow

P3 preserves one direction of authority:

```text
source trait / impl
    ↓
TraitSurface + AssociatedTypeRequirementId
    ↓
source conformance plans
    ↓
exact ConformanceEvidence
    ↓
canonical projection normalization
    ↓
ordinary normalized TypeId contracts
    ↓
semantic lowering / stable publication
    ↓
compiler runtime type recipes + executable conformance method choices
    ↓
VM execution
```

Consumers never reverse this direction.

## 7.2 Runtime associated-type execution

The desired execution path is:

```text
Self::Item
    ↓ semantic normalization
T / Int / String / labeled tuple / exact case / other ordinary TypeId
    ↓ runtime type recipe if reification is needed
RuntimeTypeEnvironment
    ↓
ordinary VM call/contract execution
```

P3 must not create:

```text
RuntimeAssociatedProjection
VM associated-type table lookup
runtime `Item` name lookup
runtime trait scan to normalize a projection
```

If any runtime path appears to need one, STOP AND CONSULT before changing architecture.

## 7.3 Concrete iterator reference model

Because live Universe already owns class names `Iterable` and `Iterator`, executable P3 fixtures must use non-conflicting names such as:

```phalcom
trait C5Iterator {
  type Item
  next -> Option<Self::Item>
  iterator { self }
}

trait C5Iterable {
  type Item
  type Iterator
  iterator -> Self::Iterator
}
```

or equally explicit test-local names.

The *support documentation* should use the canonical future names `Iterator` / `Iterable`; executable fixtures use prefixed names only to avoid colliding with transitional production classes.

P3 provides explicit concrete conformances:

```text
C5List<T> -> C5Iterable
C5ListIterator<T> -> C5Iterator
C5ListIterator<T> -> C5Iterable   // explicit, not derived
```

so the concrete semantics can execute without generic evidence assumptions.

No compiler rule may infer the latter conformance from the former.

## 7.4 Metadata branch

P3 uses a strict verify-first discriminator:

```text
projection-bearing public trait contract
    ↓ normal cross-module compilation/publication path

A. raw symbolic AssociatedProjection never enters durable metadata
   → retain exporter rejection
   → document/test normalized-only boundary

B. durable semantic metadata must represent the symbolic projection
   → extend stable semantic metadata with a canonical AssociatedProjection node
   → exporter/importer round-trip stable subject + trait application + requirement identity
   → runtime consumers still receive normalized forms only
```

The branch is not optional guessing. T4 below defines the discriminator and the exact allowed correction.

If implementing branch B requires inventing a new independent associated-requirement identity universe instead of stable translation of the semantic identity, STOP AND CONSULT.

## 7.5 LSP architecture

```text
semantic session/source index
    ↓ SemanticTargetId::AssociatedType
LSP adapter
    ↓ location / hover presentation
protocol response
```

The LSP must not do:

```text
read token text "Item"
find surrounding trait
scan trait members
scan impls
resolve applicable conformance
normalize projection
```

Any missing functionality is an adapter/presentation gap over canonical semantic products.

## 7.6 Incremental architecture

Cross-module P3 mutations must reuse existing owner/dependency products:

```text
trait surface fingerprint
impl/source associated plan fingerprint
exact evidence dependency
projection occurrence/source-index dependency
consumer semantic query dependency
```

P3 does not add a whole-workspace invalidation fallback.

---

# 8. Ownership boundaries

## 8.1 P3 owns

- C5 checkpoint/guidance amendment reflecting the ratified iterator/C6 boundary;
- revision of LANG005 iterator support fixtures to the target stateful design;
- executable runtime proof for projection-bearing associated contracts;
- generic source conformance specialized to multiple exact targets at runtime;
- exact enum-case associated runtime proof;
- projection-bearing bound/detached callable + forced-GC proof;
- labeled tuple associated binding/exact specialization proof;
- concrete iterator-target semantics: getter-shaped `iterator`/`next`, fresh collection iterators, iterator self-identity, stable exhaustion, ordinary failure propagation;
- cross-module associated-type semantic/compiler/runtime integration;
- stable metadata boundary verification and the bounded correction if raw projection metadata is genuinely required;
- real LSP associated declaration/binding/projection navigation and bounded hover adaptation;
- cross-module incremental replacement/removal/re-add cold parity;
- high-density C5 stress and negative corpus;
- reclassification of baselines that intersect actual P3 verticals;
- final C5 focused certification and C6 migration handoff.

## 8.2 P3 does not own

- `|=` parsing or conformance-bound semantics;
- `T |= A + B` sugar;
- `Trait<Item = A>` associated refinement syntax;
- generic `T::Item = A` constraint semantics;
- nested generic conformance evidence;
- conditional/derived conformance;
- general Iterator ⇒ Iterable proof;
- general Iterable.Iterator ⇒ Iterator proof;
- general equality Iterable.Iterator.Item = Iterable.Item;
- production `for` semantic migration;
- production `for` compiler lowering migration;
- removal of `class Iterable<T>` from the runtime bootstrap;
- production collection inheritance migration;
- production Map iteration change from key to entry tuple;
- production stateful RangeIterator/lazy adapter migration;
- visibility/private/package-authority redesign;
- GATs, associated defaults, supertraits, trait objects, public reflection descriptors;
- general runtime metadata redesign unrelated to C5;
- unrelated baseline cleanup.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| associated requirement identity | `TraitSurface` / `AssociatedTypeRequirementId` | impl plans, projections, source index | string/range identity |
| source binding meaning | `ConformanceAssociatedTypePlan` | source signatures, exact evidence | body checker local table |
| exact associated value | `ConformanceEvidence` | normalizer, exact requirement view | runtime table / LSP lookup |
| recursive normalization | `types::normalize_type` | semantic exact/source consumers | metadata/compiler/LSP normalizer |
| executable trait method selection | semantic lowering | compiler/VM | VM conformance search |
| stable source target | `SemanticTargetId::AssociatedType` | LSP | editor name search |
| target iterator semantics | C5 checkpoint/support + C6 handoff | future C6 migration | production P3 special case |
| current production iteration | `../../../__unstructured__/spec/current/iteration.md` + live core/compiler | existing runtime | support fixture claiming already migrated |
| generic trait evidence | future C6 constraint/evidence subsystem | future generic checking | P3 hard-coded iterator theorem |

---

# 9. Global invariants

`INV-01 — Associated requirement identity remains trait-owned.`  
No consumer identifies an associated type by spelling or range.

`INV-02 — One recursive projection normalizer.`  
`types::normalize_type` remains the only recursive associated-projection walk/cycle detector.

`INV-03 — Source mode does not request final evidence.`  
Source conformance signatures/bindings use source plans only.

`INV-04 — Exact mode consumes exact evidence.`  
Exact projection meaning comes from canonical `ConformanceEvidence`.

`INV-05 — Runtime is not projection authority.`  
The VM never solves associated projection or proves conformance.

`INV-06 — Runtime receives normalized type information.`  
If runtime reification is required, P3 transports ordinary normalized runtime type recipes/environments.

`INV-07 — Exact generic applications specialize one source conformance correctly.`  
One `ImplId` may produce different exact associated values for `Box<Int>` and `Box<String>`.

`INV-08 — Exact enum-case identity is preserved.`  
An exact-case associated conformance never collapses to its enum root in the C5 execution path.

`INV-09 — Tuple labels survive associated normalization.`  
`(key: K, value: V)` remains labeled after exact specialization.

`INV-10 — Bound/detached callable specialization survives lifetime events.`  
A method contract involving an associated value remains correct after binding and forced GC.

`INV-11 — No C6 bound syntax or assumption semantics land in P3.`

`INV-12 — No iterator-specific compiler theorem lands in P3.`  
There is no special rule deriving Iterator/Iterable nested evidence.

`INV-13 — Production cursor iteration remains behaviorally unchanged in P3.`  
`for`, core `Iterable<T>`, Map key iteration, and lazy cursor pipelines remain transitional production behavior.

`INV-14 — Ratified iterator target remains stateful and getter-shaped.`  
Future `iterator` and `next` are getters; no public cursor survives the target model.

`INV-15 — Iterable freshness semantics are explicit.`  
A reusable source's `iterator` creates an independent traversal.

`INV-16 — Iterator self-iteration is identity-preserving.`  
Concrete P3 iterator fixture explicitly conforms to the iterable fixture and returns `self`.

`INV-17 — Exhaustion is fused/stable.`  
After `next` yields `None`, subsequent reads yield `None`.

`INV-18 — Iterator errors are ordinary errors.`  
`None` means exhaustion only.

`INV-19 — Map target Item is a labeled entry product.`  
Future Map associated item is `(key: K, value: V)`, not key-only and not an unlabeled pair.

`INV-20 — LSP consumes canonical associated targets.`  
No editor-owned associated lookup or projection solver is introduced.

`INV-21 — Cross-module identity remains stable.`  
Trait/requirement/impl identities survive normal module linking/import boundaries.

`INV-22 — Cross-module incremental and cold results converge.`

`INV-23 — Metadata representation, if required, is stable semantic translation.`  
No snapshot-local `TypeId` or source-range identity is serialized as durable associated identity.

`INV-24 — No whole-workspace invalidation shortcut.`

`INV-25 — Existing C4 conformance surface separation is preserved.`  
Trait-only defaults/witnesses remain detached/evidenced and are not injected into target inherent method tables.

`INV-26 — C5 closure remains honest about C6.`  
P3 completion never claims generic trait evidence or production stateful iteration is implemented.

`INV-27 — Baseline failures are classified, not laundered.`

`INV-28 — A C5-owned failure cannot be waived because a broad baseline exists.`

---

# 10. Non-goals

Do not expand P3 into any of the following even if adjacent code makes them tempting:

```text
full standard-library iterator rewrite
generic constraint parser redesign
trait existential/object representation
generic bound syntax bikeshedding
conditional conformance architecture
associated type defaults
GAT syntax/semantics
supertrait design
package-scoped inherent/private authority redesign
visibility redesign
Map API redesign beyond the target-support record
Tuple item-union precision
mutation-during-iteration policy
iterator cloning/reset/peek protocol
async iterator/stream design
fiber integration redesign
collection performance rewrite
string-join optimization
metadata schema modernization unrelated to AssociatedProjection
exact-case metadata work not reached by a C5 discriminator
whole-repository formatting cleanup
workspace-wide unrelated test repair
```

---

# 11. Expected impact map

## 11.1 Expected documentation/state

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
docs/implementation/LANG005/support/iterable.ph
docs/implementation/LANG005/support/core_trait_impls.ph
docs/specs/objects/traits.md                     # only if C5 closure semantics need clarification
LANG005.C5.P3-walkthrough.md                     # create
LANG005.C5.P3-handoff.md                         # create
```

Do not rewrite `../../../__unstructured__/spec/current/iteration.md` to the target model while production remains cursor-based. A small forward-reference/amendment note is permitted only if repository decision-record conventions require it and it remains truthful about current behavior.

## 11.2 Expected semantic source areas

Primarily verify/use; modify only when a P3 reproducer exposes a C5-owned gap:

```text
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/source_index/
phalcom-semantic/src/presentation.rs
```

## 11.3 Expected stable metadata areas — conditional

Only if T4 proves raw symbolic projections must cross durable semantic metadata:

```text
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/header.rs              # schema/feature bump only if live conventions require
phalcom-semantic/src/metadata/export.rs
stable metadata importer/materializer discovered in live code
focused metadata round-trip tests discovered in live tree
```

## 11.4 Expected core/runtime test areas

```text
phalcom-core/tests/core/language/traits_p3_closure.rs
```

Preferred: extend this file with focused C5 executable verticals if it remains comprehensible. If it becomes unwieldy, create:

```text
phalcom-core/tests/core/language/traits_c5_closure.rs
```

and register it through the existing core test module conventions. Do not restructure unrelated tests.

Production core runtime code should change only if a C5-owned runtime reproducer proves a transport/reification defect.

## 11.5 Expected semantic tests

```text
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/incremental/associated_types.rs
phalcom-semantic/tests/semantic/integration/editor.rs
phalcom-semantic/tests/semantic/integration/compiler_capabilities.rs
```

A new focused file under `phalcom-semantic/tests/semantic/associated/` is allowed if it keeps P3 integration tests coherent and the module is registered using existing conventions.

## 11.6 Expected LSP tests

```text
phalcom-lsp/src/backend.rs                    # adapter only if necessary
phalcom-lsp/tests/impl_navigation.rs
phalcom-lsp/tests/imported_binding_resolution.rs
focused hover/definition test file discovered from live conventions
```

## 11.7 Explicit non-target production paths

Do not modify as part of the planned iterator work:

```text
phalcom-core/core/universe/src/collections/iterable.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/src/compiler/lib/loops.rs
semantic `for` iteration checker
runtime Universe Iterable bootstrap/class relation
```

They may only be touched if an independent C5-owned bug is reproduced and the change is unrelated to performing the deferred migration. Such a touch is a STOP/CONSULT condition if it changes iteration semantics.

## 11.8 Unexpected-touch rule

Touching adjacent helper/tests is allowed when mechanically necessary. STOP AND CONSULT before entering a materially different subsystem when the entry would change identity, proof authority, stable representation, runtime semantics, or checkpoint scope.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

1. P1/P2 associated requirement/projection identities or normalizer ownership.
2. The C5/C6 boundary described in §§2/5/8.
3. The fact that P3 implements no `|=` or associated constraint syntax.
4. The fact that P3 does not migrate production `for`.
5. The fact that P3 does not remove the production cursor `Iterable` superclass/protocol.
6. Getter-shaped target `iterator` and `next`.
7. Stateful target iterator model.
8. Fresh source iterator vs iterator-self identity semantics.
9. Stable exhaustion.
10. Map target `Item = (key: K, value: V)` labeled tuple.
11. Tuple target `Item = Dynamic` initial policy.
12. Runtime/compiler remain consumers of normalized semantic types.
13. LSP remains consumer of canonical semantic targets.
14. Exact case identity preservation.
15. C4 detached trait behavior separation.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- names of test-local fixture traits/classes, provided they avoid Universe collisions and clearly map to target `Iterator`/`Iterable`;
- whether C5 runtime tests extend `traits_p3_closure.rs` or live in a new focused sibling test module;
- private helper extraction in test harnesses;
- exact multi-module fixture harness based on current repository test infrastructure;
- exact LSP test file placement based on existing navigation/hover conventions;
- metadata helper names/private structs if branch B is required;
- checkpoint wording that preserves the exact boundary and intent;
- test filter names and commands after validating them with `-- --list` when necessary.

## 12.3 VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| exact projection-bearing calls already lower to ordinary runtime types | existing C4/C5 lowering + new T2 test | fix semantic→runtime transport; do not add runtime projection solver |
| bound callable preserves exact associated specialization | C4 bound-call lowering/runtime env | fix canonical env retention; STOP if method identity model must change |
| exact enum-case projection survives executable lowering | C4 exact-case lowering + T3 | fix C5-owned erasure if local; STOP if global case identity architecture must change |
| labeled tuple associated type is representable and normalizable | tuple type/product grammar + P2 normalizer | adapt syntax mechanically; STOP if labels are not semantic type constituents as assumed |
| raw AssociatedProjection is not needed by durable metadata | T4 cross-module discriminator | if false, implement bounded stable node per T4; STOP if independent identity scheme is needed |
| current metadata schema has an importer/materializer that can round-trip a new node | live metadata consumers | if absent because metadata is export-only for this path, adapt to actual boundary; do not invent unused importer |
| canonical SemanticTargetId is sufficient for LSP definition | backend/navigation tests | extend adapter/presentation only; no name resolver |
| cross-module incremental dependency graph already observes associated plans | T6 | add owner-scoped dependency edge; no global invalidation |
| legacy BL-01/BL-04 does not block C5 runtime vertical | G1/G7 discriminators | reclassify A/B if it does; fix only underlying C5-causal defect |
| C5-BL-07 remains unchanged | final trait capability gate | retain D if exact cause unchanged; otherwise classify before edit |

---

# 13. Global STOP / CONSULT triggers

Stop editing and build a consultation packet if any occurs:

1. A P1/P2 identity or normalization architecture appears insufficient for a P3 vertical.
2. A runtime consumer appears to require solving `AssociatedProjection` itself.
3. A general iterator relationship cannot be tested concretely without adding a generic evidence theorem.
4. Any proposed fix adds `|=`, generic trait assumptions, associated equality constraint semantics, or conditional conformance.
5. Production `for`/cursor iteration must change to make a C5 certification test work.
6. Metadata needs a new independent associated identity rather than stable translation of existing identity.
7. Stable metadata transport would require serializing snapshot-local `TypeId`, source ranges, or plain associated names as identity.
8. LSP navigation cannot work without independently resolving traits/conformances rather than consuming semantic targets.
9. Exact enum-case identity is lost in a way requiring redesign beyond the bounded C5 path.
10. A bound callable requires changing callable/conformance identity rather than retaining an existing runtime type environment.
11. The same nontrivial semantic failure remains after one serious correction.
12. Two plausible fixes have materially different proof/runtime ownership.
13. The live repository contradicts the ratified getter-shaped/stateful iterator design.
14. A test oracle requires generic trait evidence to be considered valid.
15. A broad baseline must be weakened/skipped to make P3 appear green.
16. Cross-module incrementality appears to require whole-workspace invalidation.
17. C5 closure cannot be stated truthfully without performing C6 work.
18. Any task begins rewriting production collection iteration as a workaround.

A triggered consultation cannot be self-waived.

The consultation packet should include:

```text
Invariant at risk:
Observed behavior:
Minimal reproducer:
Relevant call/query path:
What was verified:
One attempted correction, if semantic/mechanical:
Two architecture options, if applicable:
Decision requested:
```

---

# 14. Debugging budget

## 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

```text
inspect
identify concrete mechanical cause
make one coherent correction
rerun smallest discriminator
```

Examples:

```text
wrong test module registration
moved helper name
fixture syntax typo
missing import
metadata fingerprint arm omitted after an approved TypeNode addition
```

## 14.2 Semantic failures

Before editing, record privately:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

## 14.4 Test discipline

Never rerun an unchanged failing test unless a relevant dependency changed.

Do not respond to one test failure by running a broader suite.

---

# 15. Testing surface analysis

The following coverage obligations are mandatory design coverage for P3. The final walkthrough must map each `CV-*` to an actual test/evidence result.

| ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | runtime canonical success | 04–06 | associated type in trait default parameter/result executes |
| CV-02 | generic exact specialization | 04,07 | one source impl, `Box<Int>` and `Box<String>` exact associated values execute distinctly |
| CV-03 | exact witness contract | 04,07 | explicit witness uses normalized `Self::Item` in parameter and return |
| CV-04 | default selection | 04,25 | trait default using `Self::Item` remains detached and executes |
| CV-05 | exact enum case | 08 | exact enum-case associated conformance executes only for the exact case |
| CV-06 | sibling exact case negative | 08 | sibling enum case does not inherit/leak associated conformance |
| CV-07 | bound callable lifetime | 06,10 | projection-bearing bound method survives forced GC |
| CV-08 | runtime no-solver boundary | 05,06 | execution succeeds without runtime projection representation/search |
| CV-09 | labeled tuple binding | 09,19 | exact `(key: K, value: V)` binding preserves labels |
| CV-10 | nested structural projection | 02,09 | `List<Self::Item>` or tuple-nested projection executes/normalizes correctly |
| CV-11 | concrete iterator getter surface | 14 | `iterator`/`next` are used as getters in executable fixture |
| CV-12 | iterable freshness | 15 | two source `.iterator` reads produce independent traversal |
| CV-13 | iterator identity | 16 | iterator `.iterator` returns same traversal object |
| CV-14 | stable exhaustion | 17 | repeated post-exhaustion `next` remains None |
| CV-15 | iterator failure semantics | 18 | raised error is not converted to None |
| CV-16 | explicit concrete dual conformance | 11,12,16 | iterator fixture explicitly implements both traits; no derived magic |
| CV-17 | absence of bound syntax | 11 | no P3 parser/AST/constraint additions for `|=` |
| CV-18 | absence of iterator theorem | 12 | code review/search proves no hard-coded Iterable/Iterator proof rule |
| CV-19 | cross-module trait identity | 01,21 | trait in module A, conformance/type in B, consumer in C |
| CV-20 | cross-module exact normalization | 04,21 | imported exact target resolves correct associated value |
| CV-21 | cross-module runtime | 05,06,21 | imported consumer executes projection-bearing trait call |
| CV-22 | metadata discriminator | 23 | prove raw projection is either legitimately rejected or stably round-tripped |
| CV-23 | metadata fingerprint/round-trip | 23 | branch B only: stable projection node fingerprint and import/export round-trip |
| CV-24 | source navigation declaration | 20 | associated declaration target through real LSP definition |
| CV-25 | source navigation binding | 20 | binding LHS navigates to trait associated declaration |
| CV-26 | source navigation projection | 20 | `Self::Item` occurrence navigates to trait associated declaration |
| CV-27 | LSP cross-module navigation | 20,21 | imported projection/binding target resolves correct URI/range |
| CV-28 | hover/presentation | 20 | associated target presents coherently without LSP solving |
| CV-29 | incremental RHS edit | 22 | `Int -> String` updates exact consumer products |
| CV-30 | incremental remove/readd | 22 | removing/readding associated binding retires/restores downstream meaning |
| CV-31 | incremental trait contract edit | 22 | add/remove projection-bearing trait method updates consumer |
| CV-32 | cold/incremental parity | 22 | equivalent final source graph produces equivalent semantic result |
| CV-33 | no global invalidation | 24 | owner-scoped dependency behavior retained where harness exposes counters/reuse |
| CV-34 | high-density composition | 01–10,25 | one executable C5 stress program combines generic/exact/default/explicit/bound/case/labeled product |
| CV-35 | missing binding negative | 01–04 | incomplete conformance stays incomplete through integrated path |
| CV-36 | duplicate binding negative | 01–04 | duplicate associated binding fails closed |
| CV-37 | recursive binding negative | 02–04 | projection cycle remains deterministic/non-dynamic |
| CV-38 | wrong parameter witness | 04 | normalization reveals and rejects parameter mismatch |
| CV-39 | wrong return witness | 04 | normalization reveals and rejects return mismatch |
| CV-40 | same-name trait isolation | 01 | two traits with `Item` remain distinct in integrated fixture |
| CV-41 | legacy iteration smoke | 13 | current List `for` continues working |
| CV-42 | legacy Map behavior smoke | 13 | current production Map iteration still yields keys during P3 |
| CV-43 | legacy lazy pipeline smoke | 13 | cursor-based `.iter` pipeline remains operational |
| CV-44 | BL-01 reclassification | 27,28 | generic runtime call-entry discriminator against new associated vertical |
| CV-45 | BL-04 reclassification | 27,28 | legacy Iterable/outgoing-pack failure classified by causal overlap, not name |
| CV-46 | BL-07 reclassification | 27,28 | two Universe Bool failures unchanged or reclassified with evidence |
| CV-47 | affected-crate compile | all | AST/semantic/type-meta/core/LSP compile after final changes |
| CV-48 | source diff hygiene | all | `git diff --check`; no accidental production iterator migration |
| CV-49 | C6 handoff completeness | 26 | every deferred iterator/evidence migration obligation explicitly handed off |

### Coverage principles

- A single dense executable test may satisfy multiple runtime coverage IDs if its assertions distinguish them.
- Do not use broad workspace tests as proof for a specific semantic obligation.
- Every negative must assert the dedicated/expected semantic failure rather than simply `is_err()` when the test harness exposes diagnostics.
- Where a coverage item is verified by code review/search rather than runtime behavior (e.g. CV-18), record the exact search/review evidence in the walkthrough.

---

# 16. Verification execution budget

## 16.1 Modes

### BUILD MODE

Run only the exact new regression or closest owning test module after each task.

### STABILIZE MODE

Run directly affected feature suites and cross-layer integration after coherent groups of tasks.

### CERTIFY MODE

Run the final P3 focused matrix and only the broad crate/package gates explicitly listed in G8/G9. Do not automatically run full workspace test/clippy.

## 16.2 Verification ladder

```text
V0 — exact new regression / one discriminator
V1 — focused P3 feature test module
V2 — owning semantic/core/LSP module suite
V3 — cross-layer C5 integration slice
V4 — affected crates compile / selected crate tests
V5 — broad workspace/release (explicitly deferred unless evidence demands)
```

## 16.3 Mandatory during BUILD

- exact T-task regression(s);
- `cargo check` only for the changed crate when compilation feedback is necessary;
- use `-- --list` once before relying on uncertain filters.

## 16.4 Mandatory during STABILIZE

At coherent gates:

```text
semantic impls::queries if exact evidence/normalization changed
semantic associated incremental module if dependency logic changed
core trait closure runtime module after runtime/lowering changes
focused LSP navigation tests after adapter changes
metadata round-trip/export tests if metadata changes
```

## 16.5 Mandatory at final CERTIFY

At minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-type-meta \
  -p phalcom-core \
  -p phalcom-lsp

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic capabilities::traits

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-core --test core language::traits_p3_closure
```

If P3 creates `traits_c5_closure.rs`, run its registered filter instead of relying solely on `traits_p3_closure`.

Run the exact LSP integration tests modified by P3; do not substitute `cargo test -p phalcom-lsp` unless final task evidence or adapter breadth justifies it.

## 16.6 Explicitly deferred during BUILD

Do not reflexively run:

```text
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets
repository-wide formatter check
full core language corpus
all LSP integration tests
all module architecture tests
```

Known repository baselines make these poor inner-loop discriminators.

---

# 17. Baseline / unrelated failure policy

Use:

```text
A — definitely caused by P3 / definitely required for P3 acceptance
B — probably caused by P3 / likely causally coupled
C — unclear after one bounded classification pass
D — clearly unrelated inherited baseline
```

A/B are active responsibility. C gets one bounded discriminator; if nonblocking and still unclear, record in checkpoint and continue. D is recorded and immediately deferred.

## 17.1 Known inherited ledger to reclassify only where P3 touches it

### C5-BL-01 — Universe generic call-entry

P3 must test the new exact associated-type runtime vertical. If the same underlying call-entry defect blocks normalized associated contracts, reclassify A/B and fix the causal defect. If the P3 vertical passes, BL-01 remains unrelated.

### C5-BL-02 — A7 cold/incremental presentation mismatch

Do not generalize this failure to excuse C5 incremental divergence. P3 has its own cross-module cold-parity obligations.

### C5-BL-03 — Universe Bool capability failures

Re-run only if affected semantic capability paths change. Otherwise preserve existing classification.

### C5-BL-04 — Iterable/outgoing-pack generic call-entry

Because P3 no longer migrates production Iterable, this issue is not automatically P3-owned. Reclassify only against the new associated-type runtime vertical. If the root generic call-entry defect reproduces there, fix the shared cause; otherwise retain as legacy/baseline.

### C5-BL-05 — formatting drift

Run `git diff --check` and format touched files according to repository policy. Do not absorb unrelated formatting drift.

### C5-BL-06 — AST Clippy baseline

Do not add new warnings in touched code. Broad inherited Clippy debt is not P3 closure work.

### C5-BL-07 — two Universe Bool trait capability failures

Final trait capability gate must reproduce/classify. If exact cause and failure count are unchanged, keep D. If new P3 changes alter cause/count, classify before edit.

---

# 18. Tasks

## T0 — Re-ground the pushed P2 state and amend the C5/P3 boundary before feature edits

### Purpose

Make the repository's durable checkpoint/guidance agree with the ratified architecture **before** Luna starts implementing P3. This prevents a fresh implementation session from following stale wording that still assigns the full production Iterable migration to P3.

### Preconditions

- live branch is the intended continuation branch;
- pushed P2 revision or a mechanically later revision contains the P2 walkthrough/handoff;
- no production code edits have begun for P3.

### Consumes

- current Git state;
- C5 checkpoint/guidance;
- P2 walkthrough/handoff;
- ratified iterator and C6-bound decisions in this plan.

### Produces

- verified takeover revision/state;
- corrected C5 ownership boundary;
- corrected P3 scope in checkpoint/guidance;
- durable record that production iteration migration is C6-owned;
- baseline re-grounding for later P3 classification.

### Files and symbols

**Read:**

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-handoff.md
```

**Modify:**

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
```

### Required implementation shape

#### A. Re-ground first

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Record the actual starting revision in the checkpoint/plan execution notes if it differs mechanically from `e3e3813720c200e277a9414857536f3e2e026054`.

Then run the minimal takeover checks:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-type-meta \
  -p phalcom-core \
  -p phalcom-lsp

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types
```

Do not begin P3 edits if these have new A/B failures that invalidate the P2 takeover.

#### B. Amend checkpoint scope

Replace wording equivalent to:

```text
P3 owns full Iterable migration
```

with the precise split:

```text
P3 owns:
    C5 vertical integration/certification
    concrete/exact ratified iterator reference model
    cross-module/runtime/tooling/incremental associated-type closure

C6 owns:
    generic conformance assumption/equality evidence
    general Iterator/Iterable relationship
    production for/collection/runtime iterator migration
    cursor protocol retirement
```

Do not rewrite historical P1/P2 statements as though they were wrong at the time. Add an accepted P3 planning amendment/current boundary.

#### C. Record the bound syntax boundary

Checkpoint/guidance must explicitly state that P3 adds none of:

```text
T |= Trait
T |= A + B
Trait<Item = A>
T::Item = A generic constraint semantics
```

and that `T: Trait` is not Phalcom's future conformance syntax.

#### D. Record current-vs-target iteration distinction

```text
current production: cursor Iterable class/protocol
target post-C6: stateful Iterator/Iterable traits
```

This distinction must be visible to future implementers.

### Forbidden approaches

- no production feature edits in T0;
- no retroactive rewrite of P2 verification claims;
- no claiming current `iteration.md` is obsolete implementation-wise;
- no moving C6 work back into C5 to preserve old checkpoint wording;
- no broad test run beyond takeover discriminators.

### Test changes required

None.

### Tests to run now

Only the takeover commands above.

### Tests explicitly deferred

- trait capability baseline slice;
- core runtime trait closure;
- LSP tests;
- workspace tests/clippy.

### Acceptance

T0 is complete when:

```text
live takeover is understood
P2 focused seams are still green
checkpoint/guidance describe the revised P3/C6 boundary
active plan remains P3
no production code has changed
```

### Local STOP / CONSULT triggers

- live branch does not contain P2 implementation/records;
- P2 takeover tests have new failures attributable to repository drift;
- checkpoint convention does not allow amendment without superseding plan identity;
- another pushed commit has already performed part of the production iterator migration.

### Checkpoint update

Mandatory. T0 establishes the durable P3 scope amendment and starting revision.

---

## T1 — Replace stale LANG005 iterator support fixtures with the ratified target architecture

### Purpose

Turn the LANG005 support files into reliable C6 migration input while preserving the truth that production iteration remains cursor-based during C5.P3.

### Preconditions

- T0 boundary amendment is recorded.
- current production iteration spec is left intact.

### Consumes

- ratified iterator target from §5.2;
- current stale `support/iterable.ph` and `support/core_trait_impls.ph`;
- P2 associated type syntax (`Self::Item`, `Self::Iterator`).

### Produces

- future-target `Iterator` and `Iterable` support declarations;
- future core conformance sketches using `Item` + `Iterator`, not `Cursor`;
- future Map associated `Item = (key: K, value: V)`;
- explicit comments marking C6-owned evidence/migration relationships;
- removal of misleading support comments that the cursor model is the LANG005 target.

### Files and symbols

**Modify:**

```text
docs/implementation/LANG005/support/iterable.ph
docs/implementation/LANG005/support/core_trait_impls.ph
```

**Read for syntax consistency:**

```text
docs/specs/objects/traits.md
current tuple/product type syntax spec
```

### Required implementation shape

#### A. Target support traits

The support file should communicate this target shape:

```phalcom
// Target architecture after C6 evidence/migration is available.
trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }
}

trait Iterable {
  type Item
  type Iterator

  iterator -> Self::Iterator
}
```

Add a comment, not provisional syntax, for the C6-owned semantic relationship:

```text
C6 must make the following ordinary evidence:
    Self::Iterator conforms to Iterator
    Self::Iterator::Item = Self::Item

C6 must also establish the general Iterator ⇒ Iterable relationship.
```

Do not write an invented trait-level `where` syntax in the support file unless that exact declaration placement is separately ratified later.

#### B. Core target mapping

Revise the future support mapping conceptually to:

```text
List<T>
    Item = T
    Iterator = ListIterator<T>

Set<T>
    Item = T
    Iterator = SetIterator<T>

Map<K,V>
    Item = (key: K, value: V)
    Iterator = MapIterator<K,V>

Tuple
    Item = Dynamic
    Iterator = TupleIterator

Bytes
    Item = Int
    Iterator = BytesIterator

Range
    Item = Number
    Iterator = RangeIterator
```

These are future target sketches, not production declarations landed into Universe by T1.

#### C. Iterator target comments

Record:

```text
collection/source iterator getter creates fresh traversal
iterator.iterator returns self
next returns Option<Item>
None is stable exhaustion
errors propagate normally
no public cursor/hasNext/reset
```

#### D. Map entry target

Use a labeled tuple shape:

```phalcom
type Item = (key: K, value: V)
```

Do not retain the stale comment that current key-yield behavior is the LANG005 target.

#### E. Separate current implementation status

The support comments must say that current production `Iterable`/`for` remains cursor-based until C6 migration. This avoids confusing support design with live semantics.

### Forbidden approaches

- no edits to `phalcom-core/core/universe/src/collections/iterable.ph`;
- no edits to production Map iteration;
- no `Cursor` associated type in the target support API;
- no nullary-call spelling `next()`/`iterator()`;
- no provisional `|=` syntax;
- no `T: Trait` examples;
- no claims that every iterator is automatically iterable in current C5 implementation;
- no tests added solely for comments/support files.

### Test changes required

None. These support files are design/migration artifacts, not production sources.

### Tests to run now

No runtime tests. Use only a lightweight syntax check if repository tooling treats these support files as parse-checked documentation; otherwise no test is required.

### Tests explicitly deferred

All executable iterator semantics to T3.

### Acceptance

The support artifacts contain no stale cursor-target design and unambiguously describe the ratified stateful target plus the C6 evidence dependency.

### Local STOP / CONSULT triggers

- support files are actually consumed by a build path rather than documentation/planning only;
- labeled tuple type syntax in live authoritative grammar differs materially from the plan;
- another accepted record requires a different iterator target.

### Checkpoint update

Not necessary unless the support rewrite exposes a new durable boundary not already recorded by T0.

---

## T2 — Add the first executable associated-type runtime vertical and exact generic specialization

### Purpose

Prove that P2 associated-type normalization reaches actual compiler/runtime execution for projection-bearing trait contracts, without relying on the production iteration stack.

### Preconditions

- T0 takeover green.
- P1/P2 semantic query architecture unchanged.

### Consumes

- `TypeData::AssociatedProjection` / `types::normalize_type`;
- exact `ConformanceEvidence.associated_types` and requirement views;
- C4 semantic trait dispatch/lowering;
- runtime generic type environment/reification;
- existing `traits_p3_closure.rs` VM harness.

### Produces

- executable proof of a trait default with associated parameter/result contracts;
- executable proof of an explicit witness using `Self::Item`;
- one generic source conformance specialized for at least two exact target applications;
- proof that runtime execution needs no projection solver;
- early BL-01/BL-04 causal discriminator.

### Files and symbols

**Test:**

Preferred:

```text
phalcom-core/tests/core/language/traits_p3_closure.rs
```

Allowed if file clarity materially improves:

```text
phalcom-core/tests/core/language/traits_c5_closure.rs
phalcom-core/tests/core/language/mod.rs or equivalent registration
```

**Verify/read; modify only on reproduced C5-owned failure:**

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/vm/associated.rs
runtime type environment / RuntimeTypeRecipe implementation
phalcom-core/src/frame.rs
phalcom-core/src/heap/block.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/types/projection.rs
```

### Required implementation shape

#### A. Test source pattern

Use a trait whose associated contract is directly callable. Representative shape:

```phalcom
trait Cell {
  type Item

  echo(_ value: Self::Item) -> Self::Item {
    value
  }
}

class Box<T> {
  @constructor new(_ value: T) { _value = value }
}

impl<T> Cell for Box<T> {
  type Item = T
}
```

Then execute through exact targets such as:

```phalcom
class Caller {
  intEcho(_ box: Box<Int>) -> Int {
    box.echo(42)
  }

  stringEcho(_ box: Box<String>) -> String {
    box.echo("text")
  }
}
```

The exact syntax may be adapted to current constructor/type inference rules, but keep the key discriminator: **one source `ImplId`, two exact associated values, projection-bearing default call, runtime execution**.

#### B. Add explicit witness case

Add a second trait or second method where the conformance provides a body whose source signature uses `Self::Item`, for example:

```phalcom
impl<T> Transformer for Box<T> {
  type Item = T
  transform(_ value: Self::Item) -> Self::Item { value }
}
```

This proves source-conformance normalization and exact runtime specialization, not only default selection.

#### C. Assert runtime result, not just compile success

Store results in module bindings and assert exact values/types through the existing test harness.

#### D. Assert no inherent injection

Where a trait-only selector is used, retain the C4 invariant by checking the target class method dictionary does not receive the trait-only selector, following existing `high_density_c4_stress_program` style.

#### E. If the test fails at runtime type entry

Trace the path:

```text
exact semantic requirement view
    ↓ lowering
runtime callable/type recipe
    ↓ frame/call environment
runtime parameter/result checks or method body execution
```

Fix the smallest producer/transport defect. Do not create associated-specific runtime proof state.

### Forbidden approaches

- no using production `Iterable` to prove associated runtime behavior;
- no runtime projection lookup;
- no creating per-exact `ImplId`;
- no mutating trait symbolic signature globally to one exact target;
- no weakening the test to untyped/Dynamic calls merely to execute;
- no fixing BL-01/BL-04 unless the new reproducer proves causal overlap.

### Test changes required

Covers CV-01–CV-04, CV-07–CV-08, CV-44–CV-45 discriminator.

Add explicit test names reflecting associated behavior, e.g.:

```text
associated_type_default_executes_for_multiple_exact_generic_targets
associated_type_explicit_witness_executes_with_normalized_parameter_and_return
```

Use actual naming consistent with the test module.

### Tests to run now

Run each new test by exact name.

Then run the owning core trait test module/filter only if both pass individually:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-core --test core language::traits_p3_closure
```

or the registered C5-specific module equivalent.

### Tests explicitly deferred

- exact enum-case associated vertical (T3);
- iterator reference semantics (T3);
- cross-module metadata (T4);
- LSP/incremental;
- broad trait capability suite.

### Acceptance

T2 is complete when:

```text
one generic source associated conformance executes for Int and String exact targets
projection-bearing default executes
projection-bearing explicit witness executes
runtime class method tables remain free of trait-only injected methods
no runtime associated solver/state is introduced
```

### Local STOP / CONSULT triggers

- exact runtime call can only execute by carrying symbolic AssociatedProjection into VM;
- runtime needs to rescan conformance evidence;
- one source generic conformance cannot specialize without changing ImplId identity;
- failure is the same semantic defect after one serious correction.

### Checkpoint update

Required if T2 establishes a new durable compiler/runtime transport invariant or reclassifies BL-01/BL-04.

---

## T3 — Certify exact enum cases, labeled tuple bindings, bound callable lifetime, and the concrete iterator reference model

### Purpose

Exercise the remaining high-risk C5 runtime compositions and provide an executable concrete reference for the ratified future iterator API without any generic evidence assumptions.

### Preconditions

- T2 runtime path is green.
- no C6 generic constraint work has landed.

### Consumes

- exact enum-case conformance machinery from C4;
- P2 exact-case projection preservation;
- labeled tuple/product type support;
- bound callable runtime environment/GC behavior;
- getter behavior in trait declarations;
- explicit declared conformances only.

### Produces

- exact-case associated runtime proof;
- labeled tuple associated binding/exact specialization proof;
- projection-bearing bound callable + forced GC proof;
- concrete `C5Iterator`/`C5Iterable` executable semantics for freshness, self-identity, stable exhaustion, error propagation;
- proof that no derived/general Iterator⇒Iterable rule exists in P3.

### Files and symbols

**Tests:**

```text
phalcom-core/tests/core/language/traits_p3_closure.rs
or the T2-created traits_c5_closure.rs
```

**Potential semantic tests if exact tuple labels need direct inspection:**

```text
phalcom-semantic/tests/semantic/capabilities/traits.rs
or focused semantic/associated P3 file
```

### Required implementation shape

#### A. Exact enum-case associated vertical

Use a trait with an associated binding and a callable/default requiring it. Representative:

```phalcom
trait PayloadView {
  type Item
  reveal(_ item: Self::Item) -> Self::Item { item }
}

enum Result<T> {
  Ok(_ value: T)
  Error(_ message: String)
}

impl PayloadView for Result<Int>::Ok(_) {
  type Item = Int
}
```

Execute on `Result<Int>::Ok` and add a compile/semantic negative proving the sibling `Error` case does not receive the conformance.

Do not rely solely on class-side construction to infer the case; the test must exercise the exact target identity path already used by C4 exact-case trait runtime tests.

#### B. Labeled tuple associated binding

Use a map-like fixture, not production Map:

```phalcom
trait EntrySource {
  type Item
  first -> Self::Item
}

class PairSource<K, V> {
  @constructor new(_ key: K, _ value: V) {
    _key = key
    _value = value
  }
}

impl<K, V> EntrySource for PairSource<K, V> {
  type Item = (key: K, value: V)

  first -> Self::Item {
    (key: _key, value: _value)
  }
}
```

Adapt exact tuple expression syntax to live grammar if necessary.

Assert runtime access/presentation distinguishes `.key` and `.value` or otherwise exercises the labeled product according to current tuple semantics. Add a semantic assertion if runtime stringification does not expose labels reliably.

#### C. Bound callable + forced GC

Add a trait method using `Self::Item` in its contract, bind it:

```phalcom
let f = &receiver.method(_)
System.gc
let result = f(value)
```

and assert result after GC.

The test must ensure the callable uses exact associated specialization, not a Dynamic fallback.

#### D. Concrete target iterator traits

Because Universe already owns class names `Iterator` and `Iterable`, define test-local names:

```phalcom
trait C5Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }
}

trait C5Iterable {
  type Item
  type Iterator

  iterator -> Self::Iterator
}
```

Do **not** add constraints between them.

#### E. Concrete collection and iterator fixture

Representative shape:

```phalcom
class C5List<T> {
  @constructor new(_ values) { _values = values }
}

class C5ListIterator<T> {
  @constructor new(_ values) {
    _values = values
    _index = 0
    _done = false
  }
}

impl<T> C5Iterable for C5List<T> {
  type Item = T
  type Iterator = C5ListIterator<T>

  iterator -> Self::Iterator {
    C5ListIterator.new(_values)
  }
}

impl<T> C5Iterator for C5ListIterator<T> {
  type Item = T

  next -> Option<Self::Item> {
    // stateful advance, stable exhaustion
  }
}

impl<T> C5Iterable for C5ListIterator<T> {
  type Item = T
  type Iterator = C5ListIterator<T>

  iterator -> Self::Iterator { self }
}
```

Use live mutability/constructor syntax rather than copying the sketch blindly.

Important: the second conformance is **explicit**. P3 must not derive it automatically from `C5Iterator`.

#### F. Freshness assertions

Create two collection iterators. Advance one twice, then prove the other still yields the first element.

#### G. Iterator identity assertions

Obtain `it.iterator`; prove the returned object is the same traversal according to the repository's identity/equality primitive appropriate for objects. Then advance through one reference and observe advancement through the other.

Do not accidentally use structural `==` if that can be overridden and obscure identity. Use the canonical identity facility available in the live language/runtime fixture.

#### H. Stable exhaustion

After the iterator yields `None`, read `next` at least twice more and assert both are `None`.

#### I. Ordinary failure propagation

Create a small iterator whose second `next` raises a normal error. Assert the error propagates through ordinary runtime handling/test harness and is not converted to `None`. Keep this test minimal; do not design an iterator error protocol.

#### J. Getter syntax discriminator

All iterator fixture source must use:

```phalcom
source.iterator
iterator.next
```

not nullary calls.

### Forbidden approaches

- no canonical production trait names that collide with Universe classes in executable fixtures;
- no compiler-recognized `C5Iterable`/`C5Iterator` special case;
- no `where` trait bounds;
- no conditional conformance;
- no production Map edits;
- no nominal `Entry<K,V>` substitution for the ratified labeled tuple merely because current production Map uses `Entry` objects;
- no using `None` as a hidden bare-cursor sentinel in the target iterator fixture;
- no `next()` or `iterator()` spelling.

### Test changes required

Covers CV-05–CV-19, especially CV-09 and CV-11–CV-18.

### Tests to run now

Run each new exact test individually.

Then run the focused core trait closure module once.

If a semantic tuple-label assertion is added, run only that semantic test by exact name.

### Tests explicitly deferred

- module metadata;
- LSP;
- incremental cross-module;
- broad collection iteration.

### Acceptance

T3 is complete when exact-case associated execution, labeled tuple specialization, bound-callable GC, and all concrete iterator-target semantics pass without any generic conformance assumption or production iterator migration.

### Local STOP / CONSULT triggers

- getter members cannot be effectful/state-mutating under current language semantics;
- object identity needed for iterator-self testing has no sound existing primitive;
- tuple labels disappear before/inside the associated type system contrary to the ratified product model;
- exact case collapses in a subsystem whose correction changes global enum identity architecture;
- explicit dual iterator conformances trigger a coherence rule that implies a design conflict.

### Checkpoint update

Required after G2: record that the target iterator model is executable for explicit concrete evidence only and that general evidence remains C6.

---

## T4 — Build a cross-module associated-type vertical and resolve the stable metadata boundary

### Purpose

Prove associated-type meaning across normal module/import/compiler boundaries and determine—using an executable discriminator—whether symbolic `AssociatedProjection` must be representable in durable semantic metadata.

### Preconditions

- T2/T3 single-module runtime verticals pass.
- T0 boundary amendment is active.

### Consumes

- module/project compilation infrastructure;
- semantic stable identities;
- `MetadataExporter`;
- `phalcom-type-meta::TypeNode`;
- exact associated evidence and normalizer;
- existing multi-module test harnesses.

### Produces

- multi-module trait/conformance/consumer runtime proof;
- stable cross-module associated requirement identity proof;
- one of two explicitly recorded metadata outcomes:
  - normalized-only boundary retained, or
  - stable associated-projection metadata representation implemented and round-tripped;
- exact enum-case cross-module discriminator if the harness naturally permits it.

### Files and symbols

**Tests:** use the repository's current multi-module semantic/core integration harness discovered during T4. Prefer an existing fixture style over inventing a new loader.

Likely areas:

```text
phalcom-semantic/tests/semantic/integration/compiler_capabilities.rs
phalcom-semantic/tests/semantic/integration/*module* or imported-* equivalents
phalcom-core multi-module compiler tests if an existing harness exists
```

**Verify:**

```text
phalcom-semantic/src/metadata/export.rs
phalcom-type-meta/src/type_node.rs
stable metadata consumer/import path
module compiler/session publication path
```

**Conditional modify (branch B only):**

```text
phalcom-type-meta/src/type_node.rs
phalcom-semantic/src/metadata/export.rs
stable metadata import/materialization path
metadata fingerprint/schema tests
phalcom-type-meta/src/header.rs only if version/feature conventions require it
```

### Required implementation shape

#### A. Multi-module fixture

Create a fixture equivalent to:

```text
module A:
    exports trait Cell
        type Item
        echo(_ value: Self::Item) -> Self::Item

module B:
    imports Cell
    exports Box<T>
    impl<T> Cell for Box<T>
        type Item = T

module C:
    imports Cell + Box
    exact Box<Int>/Box<String> calls
```

Use the repository's actual import/export syntax and ownership rules.

The conformance must be declared in a location legal under the **current** implementation; do not use this test to implement the separately ratified future package-ownership redesign.

#### B. Semantic assertions

Assert the consumer resolves the same stable trait/associated requirement identity and exact associated values through the normal linked/module session path.

#### C. Runtime if harness supports compiled multi-module execution

Execute a call in module C and assert exact result. If the semantic integration harness and runtime compiler are separate in the repository, keep two focused tests rather than building a custom test framework.

#### D. Metadata discriminator

Instrument only through tests/public APIs. The question is:

```text
Does a public trait signature containing Self::Item need to export
TypeData::AssociatedProjection into SemanticMetadataBundle in the normal path?
```

##### Branch A — normalized-only/different semantic publication path

If the normal multi-module path succeeds while raw `AssociatedProjection` remains non-exportable:

- keep `MetadataExportError::NonExportableForm` for raw projections;
- add a focused test/documentation assertion that runtime/durable export receives normalized/source-stable forms where required;
- do not add unused metadata schema.

##### Branch B — stable symbolic projection required

If the normal supported module boundary fails specifically because an abstract public trait contract must carry `AssociatedProjection`, implement a stable semantic metadata form.

The stable node must conceptually carry:

```rust
AssociatedProjection {
    subject: TypeNodeId,
    trait_declaration/application: stable trait identity + stable generic args as required,
    requirement: stable trait-owned associated requirement identity,
}
```

Use the live repository's stable identity conventions. A suitable requirement representation is a stable owner declaration plus owner-relative associated requirement index, mirroring semantic identity. If trait applications already have a stable reusable representation, consume it rather than duplicating it.

Required properties:

```text
no source range as identity
no plain associated name as identity
no snapshot-local TypeId
structural fingerprint includes subject + trait identity/application + requirement identity
import reconstructs the canonical semantic AssociatedProjection form
runtime type recipe path still receives normalized types only
```

If the metadata profile distinguishes semantic/public metadata from runtime metadata, keep the projection form in the semantic profile only as appropriate to live conventions.

#### E. Exact case discriminator

If an exact-case associated binding crosses the same metadata boundary, verify it does not collapse to the root enum. If current `TypeNode` exact-case export limitation blocks the actual C5 vertical, fix the smallest stable exact-case representation consistent with existing exact-case identity. If the C5 path never exports exact case here, record the pre-existing limitation without expanding scope.

### Forbidden approaches

- no implementing metadata branch B merely because `export.rs` currently rejects projections; require the real boundary discriminator;
- no serializing `TypeId`;
- no requirement name-only identity;
- no metadata-owned projection normalization;
- no runtime consumption of symbolic projection metadata as proof authority;
- no package ownership redesign;
- no rewriting module architecture.

### Test changes required

Covers CV-19–CV-23 and may contribute to CV-05/CV-08.

If branch B lands, add focused type-meta/metadata tests for:

```text
stable projection node serialization/deserialization
fingerprint distinction for different trait owners with same Item name
round-trip canonical reconstruction
```

### Tests to run now

1. Run the exact multi-module discriminator.
2. If branch A, rerun only its owning module suite.
3. If branch B, run the exact metadata round-trip/fingerprint tests, then the multi-module discriminator again.

Do not broaden to all module tests unless shared metadata/import code changes enough to justify it.

### Tests explicitly deferred

- LSP imported navigation (T5);
- incremental replacement (T6);
- workspace metadata corpus.

### Acceptance

T4 is complete when a normal multi-module consumer obtains and executes/uses the correct exact associated meaning, and the raw projection metadata question has a proven answer with the minimal architecture implemented.

### Local STOP / CONSULT triggers

- branch B requires a new global stable trait identity model;
- metadata import cannot reconstruct projection without performing conformance solving;
- exact case cannot survive without a broad unrelated metadata redesign;
- current module ownership rules make the fixture impossible without implementing the deferred package authority model;
- cross-module semantic identity differs between cold/link paths in a way not explainable mechanically.

### Checkpoint update

Mandatory. Record branch A or B and the resulting stable metadata boundary.

---

## T5 — Complete real LSP navigation/presentation for associated declarations, bindings, and projections

### Purpose

Promote P2's semantic source-index identity from semantic editor tests to real LSP protocol behavior while preserving the semantic layer as sole resolution authority.

### Preconditions

- P2 source index produces `SemanticTargetId::AssociatedType` for declaration/binding/projection occurrences.
- T4 establishes cross-module stable behavior.

### Consumes

- semantic source index occurrences;
- `SemanticTargetId::AssociatedType`;
- existing LSP request-context/navigation/hover infrastructure;
- existing definition/imported-binding tests.

### Produces

- `textDocument/definition` support for associated declaration/binding/projection occurrences;
- cross-module definition location where existing source/link infrastructure supports it;
- coherent hover/presentation for associated target identity;
- no LSP-owned associated resolution.

### Files and symbols

**Read/modify as needed:**

```text
phalcom-lsp/src/backend.rs
LSP presentation/navigation helper modules reached by backend
phalcom-semantic/src/presentation.rs only if semantic presentation lacks an associated target form
```

**Tests:**

```text
phalcom-lsp/tests/impl_navigation.rs
phalcom-lsp/tests/imported_binding_resolution.rs
existing hover test file discovered from live tree
or a new associated_type_navigation.rs if this keeps protocol coverage coherent
```

### Required implementation shape

#### A. Definition target resolution

At a cursor over:

```phalcom
trait Cell {
  type Item
  read -> Self::Item
}
```

`Self::Item` must resolve to the source range of `type Item`.

At a cursor over:

```phalcom
impl Cell for X {
  type Item = Int
}
```

the binding `Item` must resolve to the same trait declaration target, not to itself and not to another trait's same-name associated declaration.

#### B. Canonical target adapter

If `goto_definition` has target-kind-specific matching, add `AssociatedType` to the canonical target→location adapter using semantic declaration/source provenance already carried by semantic products.

Do not re-find the trait/member by token text.

#### C. Cross-module definition

Use the T4 fixture pattern so a projection/binding in another module resolves to the correct source URI/range when the LSP workspace session has that module.

Reuse existing imported target/link resolution infrastructure.

#### D. Hover/presentation

At minimum, hovering an associated declaration/projection should present it as an associated type with owner context, for example conceptually:

```text
Cell.Item
associated type
```

If the semantic presentation API already exposes an exact current binding without extra solving, include it. If not, do not add an LSP-side exact conformance query merely for richer hover. Record rich exact hover as deferred if necessary.

#### E. References only if already generic

If the existing references engine automatically groups equal `SemanticTargetId`s, add a regression proving declaration/binding/projection convergence. If references require a new significant feature path, do not expand P3 beyond definition/navigation; record the limitation.

### Forbidden approaches

- no source text search for `Item`;
- no trait/conformance scan in LSP;
- no LSP call to a separate projection normalizer built for presentation;
- no duplicating stable identity formatting logic if semantic presentation already owns it;
- no unrelated LSP architecture refactor.

### Test changes required

Covers CV-24–CV-28.

Required actual protocol tests:

```text
associated projection definition
associated binding definition
same-name associated trait isolation
cross-module associated definition
associated hover/presentation
```

### Tests to run now

Run each modified/new LSP integration test by exact target/name.

Then run the owning test file(s), e.g.:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test impl_navigation
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test imported_binding_resolution
```

Only run a hover test file if it is changed/created.

### Tests explicitly deferred

- all LSP tests;
- semantic tokens unless associated target kind directly affects them;
- workspace scan performance.

### Acceptance

Real LSP definition/navigation works for declaration, binding, and projection occurrences—including the cross-module case—using canonical semantic target identity, and hover is coherent without independent solving.

### Local STOP / CONSULT triggers

- backend has no target→source location path capable of representing associated declarations without redesign;
- cross-module navigation would require a second module/link resolver;
- hover can only show exact meaning by performing new conformance solving in LSP;
- a new generic source-target identity is proposed outside `SemanticTargetId`.

### Checkpoint update

Required after G4: record actual LSP surface certified and any intentionally deferred presentation/reference polish.

---

## T6 — Extend associated-type incremental coverage across module boundaries and consumer products

### Purpose

Prove owner-complete invalidation and cold/incremental equivalence when an associated binding or projection-bearing trait contract changes in one module and affects a downstream consumer module.

### Preconditions

- T4 multi-module semantic fixture exists.
- P2 local associated incremental tests remain green.

### Consumes

- semantic DB/session mutation APIs;
- existing `incremental::associated_types` fixtures;
- module dependency graph/link/session fingerprints;
- exact evidence and source-index products;
- optional semantic presentation/diagnostic products used by consumer tests.

### Produces

- cross-module RHS replacement test;
- cross-module remove/re-add test;
- projection-bearing trait method add/remove consumer test;
- cold/incremental final-state parity;
- evidence that P3 did not add whole-workspace invalidation.

### Files and symbols

**Test:**

```text
phalcom-semantic/tests/semantic/incremental/associated_types.rs
```

If the file becomes unwieldy, a sibling `associated_types_modules.rs` is allowed and must be registered through existing incremental module conventions.

**Modify only if required by a reproduced gap:**

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/
semantic module/query dependency registration
source-index occurrence dependency bridge
```

### Required implementation shape

#### A. Cross-module RHS edit

Initial graph:

```text
A: trait Cell { type Item; echo(Self::Item) -> Self::Item }
B: impl Cell for Box { type Item = Int }
C: consumer valid for Int
```

Mutation:

```text
B: Item = String
```

Assert consumer C's exact associated binding/requirement view and diagnostics update accordingly.

Do not assert persistence of `TypeId`; assert semantic equivalence/identity products appropriate to the DB.

#### B. Remove/re-add binding

Remove `type Item = ...` from B:

```text
exact evidence becomes incomplete / consumer invalid
stale normalized value disappears
```

Re-add:

```text
proof and consumer recover
```

#### C. Trait projection-bearing method edit

Add or remove a trait method involving `Self::Item` in A and prove B/C products update/retire.

#### D. Cold parity

Construct the final source graph cold and compare the relevant stable semantic outcomes with the incrementally-mutated final graph:

```text
exact associated value
requirement view
consumer diagnostics/status
source target identity where stable
```

Do not compare incidental allocation IDs.

#### E. Reuse/invalidation evidence

If the test harness exposes query counters/fingerprints, assert unrelated trait/module products are reused or not invalidated. If not, code-path/dependency inspection plus absence of a global invalidation change is sufficient; do not build instrumentation solely for this plan.

### Forbidden approaches

- no rebuilding the whole semantic DB in the incremental path and calling it incremental;
- no workspace-global invalidation on associated edit;
- no stale value tolerated because BL-02 exists elsewhere;
- no dependence on source-order-only identity that cold analysis cannot reproduce.

### Test changes required

Covers CV-29–CV-33.

### Tests to run now

Run each new test by exact name.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types
```

Run `incremental::db` only if shared DB invalidation primitives changed.

### Tests explicitly deferred

- broad module architecture incremental suites;
- LSP incremental refresh unless LSP adapter code itself introduces a separate cache dependency;
- workspace benchmarks.

### Acceptance

All cross-module associated mutations retire/recompute the correct downstream products, cold and incremental final states converge, and no global invalidation escape hatch is added.

### Local STOP / CONSULT triggers

- exact associated change cannot invalidate a downstream consumer without global reset;
- module linker/session products lack a bounded dependency hook and adding one would alter module architecture broadly;
- cold and incremental identity semantics disagree after one serious correction;
- fixing the test would require weakening stable identity guarantees.

### Checkpoint update

Required. Record cross-module incremental/cold parity as a durable C5 closure invariant.

---

## T7 — Add high-density C5 stress coverage and integrated negative corpus

### Purpose

Prove the C5 model composes under a substantial program and ensure integrated failures remain precise rather than being hidden by isolated unit coverage.

### Preconditions

- T2–T6 focused verticals pass.
- no unresolved A/B failures from their gates.

### Consumes

- all P1/P2 associated/property/via/C4 trait infrastructure;
- P3 runtime/module/LSP/incremental seams;
- existing C4 high-density stress program pattern;
- diagnostic codes/failure classifications for associated bindings/projections.

### Produces

- one dense executable C5 runtime program;
- focused integrated negative tests;
- code-review evidence against forbidden duplicate solving/special cases;
- coverage closure for CV-34–CV-40 and CV-18.

### Files and symbols

**Tests:**

```text
phalcom-core/tests/core/language/traits_p3_closure.rs or traits_c5_closure.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/impls/queries.rs
or focused semantic/associated P3 module if clearer
```

### Required implementation shape

#### A. Dense executable program

Build one source program combining at least:

```text
associated trait with parameter + return projection
generic source conformance
two exact generic target applications
trait default using Self::Item
explicit witness using Self::Item
nested structural projection (`List<Self::Item>` or labeled tuple containing projection)
labeled tuple associated binding
bound callable/reference + forced GC
exact enum-case associated conformance
ordinary C4 trait-only dispatch separation
runtime assertions for all branches
```

It may include the test-local C5 iterator traits from T3 if doing so keeps the program readable, but do not make the entire stress test depend on iterator semantics.

Assert semantic lowering contains multiple trait invocation sites where useful, following the existing C4 dense test.

#### B. Missing binding integrated negative

Compile a conformance missing an associated requirement while otherwise behaviorally valid. Assert the dedicated associated completeness diagnostic/failure path, not a downstream unrelated type error.

#### C. Duplicate binding integrated negative

Assert duplicate associated binding fails in the source plan/completeness path.

#### D. Recursive binding

Use a cycle already supported by P2 syntax/context and assert the recursive/cyclic associated diagnostic/proof state remains deterministic and not `Dynamic`.

#### E. Wrong parameter witness after normalization

Trait expects `Self::Item`; conformance binds `Item = Int`; witness uses `String`. Assert normalized parameter mismatch.

#### F. Wrong return witness after normalization

Analogous return mismatch.

#### G. Same-name trait isolation

Two traits each define `type Item`; one target conforms to both with different values. Assert projection/evidence never crosses owner identity.

#### H. Architecture review search

Review/search the scoped P3 diff for:

```text
hard-coded "Iterable" / "Iterator" semantic proof rule
new GenericConstraint trait-conformance variant
new parser `|=` handling
AssociatedProjection handling in VM as solver
LSP token/name-based Item resolver
whole-workspace invalidation on associated edit
```

Record the exact review result in the walkthrough.

### Forbidden approaches

- no adding a broad stress program so opaque that a failure cannot be localized; keep named assertions/results;
- no weakening diagnostics to generic errors;
- no special-casing recursive binding to Dynamic;
- no adding future bound syntax just to make the dense test shorter;
- no production iterator migration.

### Test changes required

Covers CV-18, CV-34–CV-40 and reinforces CV-01–CV-10.

### Tests to run now

Run dense program alone, then each negative exact test.

After all pass, run the owning core C5/C4 trait module and semantic impl query module:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries
```

Do not run the whole trait capability module yet unless a negative was added there and a focused filter is insufficient.

### Tests explicitly deferred

- legacy iterator regression/baselines (T8);
- final affected crates (T9);
- workspace certification.

### Acceptance

Dense execution and all integrated negatives pass; code review finds no C6 syntax/evidence, runtime solver, LSP resolver, or iterator-specific theorem introduced by P3.

### Local STOP / CONSULT triggers

- dense program reveals incompatible meanings between source and exact normalization;
- diagnostics require a second associated completeness authority;
- a negative can only be made precise by redesigning P2 terminal-state algebra;
- forbidden architecture appears necessary rather than accidental.

### Checkpoint update

Required if the dense program establishes a new durable integration invariant or reveals/reclassifies a residual issue.

---

## T8 — Preserve transitional production iteration and reclassify P3-relevant baselines

### Purpose

Prove P3 did not accidentally change current production iteration while making an evidence-based final classification of BL-01, BL-04, and BL-07 against the actual P3 verticals.

### Preconditions

- P3 feature work through T7 is stable.
- no intentional production iterator migration occurred.

### Consumes

- existing iteration/core tests;
- current `iteration.md` behavior;
- C5 baseline ledger;
- new T2/T3 runtime verticals.

### Produces

- legacy List `for` smoke proof;
- current Map key-iteration smoke proof;
- current lazy cursor pipeline smoke proof;
- BL-01 classification against associated runtime path;
- BL-04 classification against associated runtime path vs legacy Iterable path;
- BL-07 final focused classification.

### Files and symbols

Prefer existing tests covering current iteration. Add only small discriminators if existing tests do not make the behavior explicit.

Likely read/test areas:

```text
phalcom-core/tests/core/language/*iteration*
phalcom-core/tests/core/language/*collection*
phalcom-core/core/universe/src/collections/iterable.ph
phalcom-core/core/universe/src/collections/map.ph
C5 checkpoint baseline ledger
```

Do not change production implementation merely to create the smoke test.

### Required implementation shape

#### A. List `for` smoke

Prove current `for` still executes over List and retains break/continue semantics if an existing focused test already covers it. Use existing test rather than adding duplicate coverage where possible.

#### B. Map current behavior

Prove current direct Map iteration still yields keys during P3. This is intentionally **not** the target C6 behavior; the test documents transition safety.

#### C. Lazy cursor pipeline

Exercise one representative `.iter.filter(...).map(...).take(...)` or equivalent existing lazy pipeline to ensure P3 did not break current cursor iteration indirectly.

#### D. BL-01 discriminator

Use T2 exact generic associated runtime test as the primary discriminator. If it passes, BL-01 remains unrelated to C5 closure unless another C5-owned path reproduces it.

#### E. BL-04 discriminator

Separate:

```text
legacy Iterable/outgoing-pack failing scenario
vs
new associated runtime generic scenario
```

If only the former fails, classify D for P3/C5 closure and explicitly hand it to the owner of legacy/generic call-entry work. If the same root defect blocks both, classify A/B and repair only the common causal path.

#### F. BL-07 discriminator

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic capabilities::traits
```

Expected based on P2 baseline:

```text
36/38 pass
exact same two Universe Bool dependency/capability failures
```

If count/cause changes, classify before editing.

### Forbidden approaches

- no changing Map to entry iteration in P3;
- no rewriting `for` to target iterator model;
- no “fixing” a legacy iterator baseline by starting the C6 migration;
- no suppressing BL-07 tests;
- no broad collection refactor.

### Test changes required

Covers CV-41–CV-46.

### Tests to run now

Run the smallest existing/new iteration smoke filters.

Run the trait capability slice once for BL-07 classification.

Re-run T2/T3 associated runtime discriminator only if baseline classification depends on direct comparison and the implementation changed since those tests last passed.

### Tests explicitly deferred

- full core collection corpus;
- workspace tests;
- full release certification.

### Acceptance

Current production iteration is unchanged, and BL-01/BL-04/BL-07 are classified from causal evidence rather than inherited labels or names.

### Local STOP / CONSULT triggers

- a P3 semantic/runtime change breaks legacy `for`/Map/lazy iteration and the correction would require production migration;
- BL-04 is causally identical to a P3 blocker but fixing it requires a broad outgoing-pack redesign;
- BL-07 cause changes into associated projection logic.

### Checkpoint update

Mandatory. Record final baseline classifications and current production-iteration smoke evidence.

---

## T9 — Final focused C5 stabilization and certification

### Purpose

Run the intentionally bounded final P3/C5 evidence matrix, close any A/B failures, verify no forbidden scope drift, and establish the truth required to mark C5 complete.

### Preconditions

- T0–T8 complete;
- no unresolved mandatory consultation;
- all focused task-local tests green except accepted C/D baselines.

### Consumes

- all P3 implementations/tests;
- CV-01–CV-49 ledger;
- baseline classifications;
- checkpoint lifecycle conventions.

### Produces

- final focused certification record;
- affected-crate compile proof;
- C5 coverage mapping;
- final scope/code review;
- truthful checkpoint completion classification.

### Files and symbols

No new architecture expected. Only tests/fixes necessary to close an uncovered P3 obligation or a new A/B regression.

### Required implementation shape

#### A. Coverage audit before broad commands

Map every CV-01–CV-49 obligation to an actual test or explicit review evidence. If any item has no evidence, add the smallest focused test before proceeding.

Do not use a broad command as a substitute for a missing discriminator.

#### B. Run final affected-crate check

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-type-meta \
  -p phalcom-core \
  -p phalcom-lsp
```

#### C. Run final semantic exact/evidence gates

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types
```

Run the P3 cross-module semantic module/filter if it is outside these commands.

#### D. Run final core C5 runtime gate

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-core --test core language::traits_p3_closure
```

or the exact registered C5 module if T2 created one.

#### E. Run final LSP focused gate

Run each P3-modified LSP test file. Example:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test impl_navigation
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test imported_binding_resolution
```

plus any dedicated associated hover/navigation test file.

#### F. Run trait capability baseline classifier

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic capabilities::traits
```

Interpret per T8 classification.

#### G. Diff hygiene

```sh
git diff --check
```

Review changed file list and diff for forbidden scope:

```text
production for-loop migration
production Iterable superclass removal
production Map entry behavior change
new |= parser/constraint syntax
new generic conformance assumption type
runtime AssociatedProjection solver
LSP name-based associated resolver
whole-workspace invalidation
```

#### H. Optional broader crate tests

Only if P3 changed shared stable metadata types or central LSP adapter code beyond focused paths, run the owning crate's full unit/integration tests. Do not automatically jump to workspace-wide commands.

### Forbidden approaches

- no feature design during certification;
- no fixing D baselines to make counts prettier;
- no marking C5 complete with uncovered CV obligations;
- no workspace green requirement unless repository policy explicitly mandates it for checkpoint completion;
- no C6 migration slipped in as “cleanup”.

### Test changes required

Only those needed to close an identified CV gap or A/B failure.

### Tests to run now

The final matrix above.

### Tests explicitly deferred

Unless a repository release policy explicitly requires them:

```text
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets
full repository formatter sweep
full core language corpus
```

Record any broad commands that were actually run; never claim unexecuted evidence.

### Acceptance

T9 is complete when:

```text
all C5-owned CV obligations have passing evidence
all P3 A/B failures are fixed
C/D failures are recorded
all affected crates compile
focused semantic/runtime/LSP/incremental gates pass
no forbidden C6/production iterator migration appears in diff
checkpoint can truthfully close C5
```

### Local STOP / CONSULT triggers

- final coverage reveals a requirement that cannot be met without C6 evidence;
- metadata/LSP/runtime fix during certification changes semantic authority;
- a new broad failure is plausibly C5-owned but cannot be classified in one bounded pass;
- C5 completion wording would be misleading.

### Checkpoint update

Mandatory after G8/G9. Record actual final commands/results and completion classification.

---

## T10 — Synchronize specs/state and produce the C6 iterator/evidence migration handoff

### Purpose

Make repository documentation reflect what C5 actually implements, preserve the distinction between current cursor iteration and ratified target iteration, and hand C6 a precise migration contract that requires no reconstruction from chat history.

### Preconditions

- T9 final focused certification complete.
- no unresolved P3 A/B failure.

### Consumes

- final P3 implementation/diff;
- actual verification results;
- metadata branch outcome;
- baseline classifications;
- iterator target support files;
- C5/C6 boundary amendment.

### Produces

- final C5 checkpoint status;
- updated C5 guidance;
- trait spec clarification where appropriate;
- `LANG005.C5.P3-walkthrough.md`;
- `LANG005.C5.P3-handoff.md` targeted at C6;
- explicit production iterator migration contract;
- truthful statement that production iteration is still cursor-based at C5 closure.

### Files and symbols

**Modify:**

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
docs/specs/objects/traits.md                    # only C5 associated semantics / C6 boundary clarification
docs/implementation/LANG005/support/iterable.ph
docs/implementation/LANG005/support/core_trait_impls.ph
```

**Create:**

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P3-walkthrough.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P3-handoff.md
```

### Required implementation shape

#### A. C5 final checkpoint status

If all acceptance criteria are met, set the plan/checkpoint lifecycle truthfully, e.g.:

```text
LANG005.C5.P3: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
LANG005.C5:    COMPLETE / IMPLEMENTED / FOCUSED_TESTED
next checkpoint: LANG005.C6
```

If repository lifecycle convention uses `BASELINE_BLOCKED` at checkpoint level because a mandatory broad gate remains blocked, use that exact truthful state instead. Do not infer the label; follow the lifecycle convention.

#### B. Trait specification

Document only C5 semantics that actually exist:

```text
associated declarations/bindings
contextual projection
normalization/evidence
cross-stack transport boundary
no generic bound assumptions yet
```

Do not document `|=` as implemented syntax.

#### C. Current iteration spec

Do not rewrite `../../../__unstructured__/spec/current/iteration.md` into the future model unless production migration unexpectedly landed through an accepted amendment (which this plan forbids). If needed, add/adjust a non-normative forward reference elsewhere stating that a stateful target is ratified for C6.

#### D. Walkthrough content

Must include:

```text
final lifecycle state
starting/ending revision
P3 scope amendment
associated runtime architecture actually exercised
exact generic runtime results
exact-case results
bound callable/GC result
iterator concrete reference model
metadata branch A/B decision and evidence
multi-module results
LSP results
cross-module incremental results
high-density/negative corpus
legacy iteration smoke results
baseline classifications
complete CV-01–CV-49 evidence table
tests run vs explicitly deferred
residual risks
```

#### E. C6 handoff — mandatory iterator/evidence contract

The handoff must explicitly state that C6 owns and must implement:

```text
conformance relation syntax:
    T |= Trait

conjunction sugar:
    T |= A + B

associated refinement/equality under final C6 design:
    T |= Iterator<Item = A>
    and/or T::Item = A in where

no T: Trait binder syntax
```

It must then specify the semantic relationships the generic evidence system must express:

```text
Iterable:
    Self::Iterator conforms to Iterator
    Self::Iterator::Item = Self::Item

Iterator ⇒ Iterable:
    Iterable.Item = Iterator.Item
    Iterable.Iterator = Self
    iterator = self
```

It must specify the production migration sequence:

```text
1. add generic/nested evidence and associated equality authority
2. make target Iterator/Iterable traits canonical Universe behavior
3. migrate core concrete conformances
4. make collection iterator getters create fresh stateful iterators
5. make iterator.iterator return self
6. migrate semantic for typing to evidence-driven Item
7. migrate compiler for lowering to iterator + next + match semantics
8. migrate Map direct Item to `(key: K, value: V)`
9. migrate Range to RangeIterator
10. migrate lazy adapters to stateful iterator composition
11. remove runtime Iterable superclass/cursor protocol/bootstrap relation
12. retire iterate(_)/iteratorValue(_) production assumptions
13. update current iteration spec only when implementation switches
```

Also hand off the nonfunctional laws:

```text
stable exhaustion
normal error propagation
no hasNext/reset/clone requirement
current mutation-during-iteration policy remains collection-specific
```

#### F. Metadata handoff

If T4 branch A: state raw projection metadata is intentionally non-exportable because the supported compiler/runtime boundary normalizes before durable/runtime export.

If branch B: state the stable metadata projection node/identity contract C6 must preserve.

### Forbidden approaches

- no claiming production iterator migration complete;
- no documenting `|=` as currently accepted parser syntax;
- no removing current cursor normative spec while implementation uses it;
- no rewriting historical P2 records;
- no claiming workspace/release tests that were not run;
- no feature code in T10 except a trivial documentation-discovered correction backed by the final focused discriminator; material code changes return to T9.

### Test changes required

None for documentation alone. Any discovered code gap returns to the owning task and gate.

### Tests to run now

No tests merely because docs changed. Run `git diff --check` after final record edits.

### Tests explicitly deferred

All unexecuted broad repository/release gates remain explicitly listed in the walkthrough/checkpoint.

### Acceptance

C5 records match the implementation and evidence; the C6 handoff fully specifies the generic evidence and production iterator migration obligations; no reader could mistake the concrete P3 iterator fixture for general C5 conformance inference.

### Local STOP / CONSULT triggers

- final docs cannot describe C5 as complete without claiming unimplemented C6 semantics;
- lifecycle convention requires a different closure state due baseline blockers;
- support/spec authority conflicts over whether target iterator design should be normative before migration.

### Checkpoint update

Mandatory final C5 update.

---

# 19. Verification gates

## G0 — Takeover and scope-amendment gate

**Purpose:** prove P2 remains a valid base and durably prevent accidental P3 production iterator migration.

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-type-meta \
  -p phalcom-core \
  -p phalcom-lsp

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types
```

Expected: affected crates compile; P2 exact/incremental suites retain their recorded behavior. Checkpoint/guidance then contain the amended C5/C6 boundary.

Do not proceed with feature edits if a new takeover failure invalidates P2 assumptions.

---

## G1 — Associated runtime execution gate

**Purpose:** prove projection-bearing contracts reach compiler/runtime correctly and establish BL-01/BL-04 discriminator evidence.

Run exact T2 tests by name, then the owning trait runtime module.

Expected:

```text
one generic source impl executes for at least Int and String exact targets
default + explicit witness associated contracts execute
trait-only methods remain detached
no runtime projection solver exists
```

On failure classify before editing.

---

## G2 — Exact-case / labeled-product / iterator-reference gate

**Purpose:** prove high-risk exact identity and the ratified target iterator semantics for explicit concrete evidence.

Run exact T3 tests.

Expected:

```text
exact enum case succeeds; sibling does not leak
(key: K, value: V) labels survive exact associated binding
bound callable survives GC
source iterator getters are fresh
iterator.iterator is same traversal
post-exhaustion next remains None
iterator failure propagates normally
all source uses getter syntax
```

No general `Iterator ⇒ Iterable` inference is expected or permitted.

---

## G3 — Multi-module / metadata boundary gate

**Purpose:** prove associated identity across modules and make a definitive metadata branch decision.

Run the T4 multi-module discriminator and, if branch B, exact metadata round-trip/fingerprint tests.

Expected:

```text
module C observes correct exact associated value from A/B declarations
stable identities remain owner-correct
metadata branch A or B is recorded with evidence
no runtime solving is introduced
```

Do not broaden after PASS.

---

## G4 — Real LSP gate

**Purpose:** prove P2's canonical semantic associated targets are usable through protocol-facing LSP navigation.

Run exact P3 navigation/hover tests and their owning LSP test files.

Expected:

```text
declaration/binding/Self::Item all navigate to canonical associated declaration
cross-module URI/range is correct
hover/presentation is coherent
no LSP-owned trait/projection solver exists
```

---

## G5 — Cross-module incremental gate

**Purpose:** prove downstream owner-complete invalidation and cold parity.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types
```

Expected: local P2 tests plus new cross-module mutation tests all pass.

Run `incremental::db` only if shared DB primitives changed.

---

## G6 — Dense C5 semantic/runtime gate

**Purpose:** prove C5 features compose and integrated negatives fail for the right reason.

Run:

- dense C5 runtime program;
- exact P3 negative tests;
- `impls::queries` after all semantic changes.

Expected: all C5-owned positives pass; negatives remain precise; no second solver/completeness system appears.

---

## G7 — Transitional iteration and baseline gate

**Purpose:** prove P3 leaves current iteration intact and classify inherited failures by causal overlap.

Run focused current iteration smokes plus:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic capabilities::traits
```

Expected: current List/Map/lazy behavior unchanged; BL-07 either exactly reproduces the two baseline failures or is reclassified from fresh evidence.

Record BL-01/BL-04 classification based on T2/T3 runtime behavior.

---

## G8 — Final focused C5 certification

Run serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-type-meta \
  -p phalcom-core \
  -p phalcom-lsp

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic impls::queries

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::associated_types

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-core --test core language::traits_p3_closure

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic capabilities::traits
```

Replace/add the core filter if P3 created a dedicated C5 closure module. Run the exact P3-modified LSP test files and metadata test target(s) separately.

Then:

```sh
git diff --check
```

Expected: every P3-owned test is green; only accepted baselines remain; affected crates compile; diff has no forbidden C6/production iterator migration.

---

## G9 — Checkpoint closure gate

**Purpose:** decide C5 lifecycle truthfully from executed evidence.

Before marking complete, verify:

```text
CV-01–CV-49 all mapped
no A/B failures remain
C/D failures recorded
metadata branch documented
LSP surface documented
current-vs-target iterator distinction explicit
C6 handoff complete
walkthrough records only executed tests
```

No additional runtime test is implied by G9.

---

# 20. Final focused acceptance

P3 may be marked `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only when all of the following hold:

1. P1/P2 associated identities, completeness, and normalization authority remain intact.
2. Projection-bearing default and explicit witness calls execute through the VM for multiple exact generic applications.
3. Exact enum-case associated identity survives executable lowering and does not leak to siblings.
4. A projection-bearing bound callable survives forced GC with correct exact specialization.
5. Labeled tuple associated binding `(key: K, value: V)` preserves labels through exact normalization and execution/semantic observation.
6. Concrete target iterator fixtures prove getter-shaped `iterator`/`next`, fresh source traversal, iterator-self identity, stable exhaustion, and ordinary failure propagation.
7. Those fixtures use explicit conformances only; no general iterator theorem or C6 syntax exists.
8. A multi-module trait/conformance/consumer vertical proves stable associated identity and exact meaning.
9. The metadata boundary has a proven branch A/B outcome and any required stable representation is implemented without runtime solving.
10. Real LSP definition/navigation works for associated declaration, binding, and projection targets; cross-module behavior is covered.
11. Cross-module associated mutations converge cold/incrementally and do not require global invalidation.
12. A dense C5 runtime program and integrated negatives pass.
13. Current production cursor iteration remains operational and intentionally unchanged.
14. BL-01/BL-04/BL-07 are freshly classified where P3 evidence intersects them.
15. Affected crates compile and `git diff --check` passes.
16. No forbidden production iterator migration or generic trait-bound work is present in the diff.
17. Checkpoint/guidance/support records are synchronized.
18. The P3 walkthrough contains a complete CV-01–CV-49 evidence table.
19. The C6 handoff explicitly owns the general evidence and production iterator migration.

A broad workspace-green state is **not** by itself a C5 acceptance requirement unless repository lifecycle policy explicitly mandates it. Conversely, a broad baseline does not excuse any failed P3-owned obligation.

---

# 21. Performance / resource evidence

P3 has no standalone throughput benchmark target. Performance/resource requirements are architectural:

1. projection normalization remains bounded by existing budget/cancellation/cycle mechanisms;
2. runtime does not add projection/conformance scans per call;
3. P3 does not introduce per-instance associated-type maps merely for execution;
4. stable metadata, if extended, hash-conses/deduplicates according to existing `TypeNode` infrastructure;
5. incremental edits remain owner/dependency scoped rather than resetting the workspace;
6. LSP consumes cached/pinned semantic products rather than performing fresh semantic solving for each request.

Evidence may consist of:

```text
code-path inspection
existing query/cache use
focused incremental reuse counters where already available
absence of new runtime solver/state
```

Do not add microbenchmarks or profiling ceremony without an observed performance regression.

If stress tests reveal repeated exact projection resolution as a measurable bottleneck, create a separate evidence-driven optimization task; do not complicate P3 proof identity preemptively.

---

# 22. Checkpoint bookkeeping

## 22.1 At plan start

T0 must record:

```text
active_plan: LANG005.C5.P3
starting revision
revised P3/C6 boundary
P2 as completed predecessor
current production cursor iteration remains transitional
```

## 22.2 During plan

Update the checkpoint only for durable events:

```text
P3 scope amendment accepted
runtime associated transport invariant established
iterator concrete reference model established
metadata branch A/B decision
real LSP surface certified
cross-module incremental parity established
baseline classification changed
consultation/amendment
G1–G8 coherent gate result
```

Do not turn the checkpoint into a command log.

## 22.3 At plan completion

Record:

```text
P3 status/completion/verification
starting and ending revision
implemented P3 integration invariants
metadata boundary decision
iterator target-reference evidence
actual commands/results
baseline/deferred failures
consultations/amendments
C5 checkpoint final lifecycle state
next action: LANG005.C6
```

If any mandatory P3 obligation is incomplete, do not close C5. Create a bounded P3 corrective follow-up rather than moving unresolved C5 work into C6 silently.

---

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P3-walkthrough.md
```

Required sections:

1. final lifecycle classification;
2. starting/ending revision;
3. P3 scope amendment and why production iterator migration moved to C6;
4. inherited P1/P2 interfaces;
5. runtime associated-type architecture actually exercised;
6. generic exact-target execution results;
7. exact enum-case execution result;
8. bound callable/GC result;
9. labeled tuple associated binding result;
10. concrete iterator reference-model implementation and semantics;
11. explicit statement that no generic Iterator⇒Iterable proof exists in C5;
12. cross-module architecture/result;
13. metadata branch A/B evidence and any schema changes;
14. real LSP definition/hover result;
15. cross-module incremental/cold-parity result;
16. dense C5 stress/negative corpus;
17. legacy iteration smoke result;
18. baseline BL-01/02/03/04/05/06/07 disposition relevant to final C5 state;
19. tests added;
20. tests actually executed with command/results;
21. tests explicitly deferred;
22. complete CV-01–CV-49 table;
23. performance/resource observations;
24. deviations/consultations;
25. residual risks;
26. next action C6.

Do not claim broad certification that was not run.

---

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P3-handoff.md
```

This is principally a **C6 handoff**.

It must contain:

## 24.1 Reliable state anchor

```text
ending revision
C5 final checkpoint state
P1/P2/P3 completed interfaces
known baselines
```

## 24.2 Frozen C5 interfaces C6 must consume

```text
AssociatedTypeRequirementId
TraitSurface.associated_types
ConformanceAssociatedTypePlan
ConformanceEvidence.associated_types
ConformanceEvidence.requirement_views
TypeData::AssociatedProjection
ProjectionNormalizationContext/Mode/Result
types::normalize_type
SemanticTargetId::AssociatedType
metadata projection representation/boundary from T4
```

## 24.3 Ratified target iterator API

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
  iterator { self }
}

trait Iterable {
  type Item
  type Iterator
  iterator -> Self::Iterator
}
```

Include:

```text
getter-shaped next / iterator
stateful single-pass Iterator
fresh Iterable.iterator for reusable sources
Iterator.iterator returns self
stable exhaustion
ordinary errors
no cursor/hasNext/reset/clone requirement
Map Item = (key: K, value: V)
Tuple Item = Dynamic initially
Range uses RangeIterator target
lazy adapters become stateful iterator composition
```

## 24.4 C6 generic evidence syntax/semantics to implement

Record the ratified future surface:

```phalcom
where
    T |= Trait,
```

and sugar:

```phalcom
where
    T |= TraitA + TraitB,
```

with the explicit note:

```text
No `T: Trait` conformance syntax.
Colon in generic bounds remains kind-bound territory such as `F: Type -> Type`.
```

Associated refinement/equality examples may be recorded as future C6 target surface only to the extent ratified:

```phalcom
T |= Iterator<Item = A>
T::Item = A
```

Do not claim their exact internal representation is already implemented.

## 24.5 Evidence relationships C6 must make first-class

```text
If X conforms to Iterable:
    X::Iterator conforms to Iterator
    X::Iterator::Item = X::Item

If X conforms to Iterator:
    X also conforms to Iterable
    Iterable.Item = Iterator.Item
    Iterable.Iterator = X
    iterator returns self
```

These must be ordinary general evidence, not `for` compiler special cases.

## 24.6 Production migration sequence

Repeat the exact ordered migration from T10 §E and mark it as C6-owned.

## 24.7 What C6 must not redesign

```text
associated identity
source/exact normalization split
one normalizer
runtime no-solver rule
labeled Map entry target
getter-shaped API
stateful iterator semantics
stable exhaustion
```

## 24.8 First recommended C6 commands

Provide takeover commands for the final C5 exact/query/runtime tests so C6 can detect regressions before introducing generic evidence.

---

# 25. Completion truth table

| Statement | Meaning | Enough to close P3/C5? |
|---|---|---|
| Support iterator fixture rewritten | future target documented | No |
| One associated runtime test passes | one executable path works | No |
| Exact generic targets pass | generic specialization works | No |
| Concrete iterator fixture passes | target semantics representable with explicit evidence | No |
| Cross-module test passes | module integration works | No |
| LSP navigation passes | editor adapter works | No |
| Incremental cold parity passes | lifecycle works | No |
| Dense stress passes | composition works | No |
| Final G8 focused matrix passes | C5 behavior has required focused evidence | Yes, with records/current CV map |
| `capabilities::traits` has only unchanged BL-07 failures | known baseline remains | Compatible with focused closure |
| Workspace all-targets green | broad repository state | Not required unless lifecycle policy says so |
| Production `for` uses stateful Iterator | C6 migration complete | **Must remain false at C5 closure** |
| `|=` generic constraints work | C6 generic evidence implemented | **Must remain false at C5 closure** |
| C5 checkpoint complete | all C5-owned associated capability work complete | Yes, after T10 |

The two bold rows are especially important: if they become true because P3 implemented them opportunistically, scope has drifted into C6 and requires consultation/amendment rather than silent acceptance.

---

# 26. Plan self-review

## 26.1 Architecture coverage

Reviewed against the ratified requirements:

- C5 associated identity/evidence/normalizer ownership is preserved.
- P3 no longer assumes production iterator migration is possible without C6 nested evidence.
- The target iterator API is fully captured without provisional constraint syntax.
- Getter-shaped `iterator`/`next`, fresh-vs-self traversal, stable exhaustion, ordinary failure, labeled Map entry Item, Tuple Dynamic policy, Range/lazy future design are all represented.
- No task requires `|=` or associated equality syntax.
- Production `for`, cursor protocol, runtime Iterable class, production Map behavior, and lazy pipeline are explicitly protected from P3 migration.
- Runtime integration tests use associated traits independent of current iteration.
- Metadata and LSP are treated as consumers of semantic authority.
- Cross-module/cold-incremental closure is explicit.
- C6 receives a full migration contract.

## 26.2 Luna executability

Every task includes:

```text
purpose
preconditions
consumes
produces
files/symbols
required implementation shape
forbidden approaches
test obligations
focused commands
deferred tests
acceptance
local escalation triggers
checkpoint policy
```

Architecture is FIXED where necessary; only metadata transport and mechanical harness placement remain verify-first/flexible.

## 26.3 Testing discipline

The plan intentionally uses:

```text
BUILD: exact new tests
STABILIZE: owning modules
CERTIFY: focused C5 matrix
```

It does not require workspace tests after every task. Broad repository baselines cannot hide C5-owned regressions.

## 26.4 Metadata fork review

The plan does not preemptively add a stable AssociatedProjection type node merely because the exporter rejects it. T4 first proves whether the supported module boundary requires symbolic projection transport. If it does, the permitted stable representation is constrained to canonical stable subject/trait/requirement identity and remains semantic metadata—not runtime proof authority.

## 26.5 Iterator boundary review

The plan deliberately uses prefixed test-local traits for executable target semantics because current Universe owns `Iterable`/`Iterator` class names. This is not a different language design; it is a collision-free C5 certification technique. Canonical future names remain in support records and C6 handoff.

## 26.6 Baseline review

BL-01 and BL-04 are reclassified only against actual P3 runtime evidence. BL-07 receives a final focused discriminator. BL-02 cannot be used to waive C5 incremental parity.

## 26.7 Placeholder review

No task relies on invented function names as mandatory production APIs. Where the live repository must determine an exact harness/importer path, the plan names the owner and defines the allowed architecture. Test names shown as examples are explicitly examples; execution must use actual created names and verify non-zero selection.

No unresolved placeholders, copy-forward shortcuts, or unbounded test instructions remain as executable ambiguity.

## 26.8 Dependency order review

```text
T0 scope + takeover
    ↓
T1 target support records
    ↓
T2 first runtime associated vertical
    ↓
T3 exact-case/labeled/iterator reference vertical
    ↓
T4 cross-module + metadata boundary
    ↓
T5 real LSP adapter
    ↓
T6 cross-module incremental parity
    ↓
T7 dense stress + negatives
    ↓
T8 transitional iteration + baseline classification
    ↓
T9 final focused certification
    ↓
T10 records + C6 handoff
```

No later task is required to retroactively define an identity or proof authority consumed by an earlier task.

---

# 27. Compact execution map

```text
T0  re-ground + amend P3/C6 scope
    ↓ G0
T1  rewrite future iterator support artifacts
T2  executable associated runtime + multi-exact generic specialization
    ↓ G1
T3  exact case + labeled tuple + bound callable + concrete iterator target semantics
    ↓ G2
T4  cross-module integration + metadata discriminator/correction
    ↓ G3
T5  real LSP definition/hover integration
    ↓ G4
T6  cross-module incremental/cold parity
    ↓ G5
T7  dense C5 stress + integrated negatives + architecture review
    ↓ G6
T8  preserve current cursor iteration + baseline reclassification
    ↓ G7
T9  final focused C5 certification
    ↓ G8/G9
T10 checkpoint/spec/support/walkthrough/C6 handoff
    ↓
LANG005.C6
```

The critical semantic dependency chain is:

```text
P2 exact associated evidence
    ↓
runtime execution proof
    ↓
cross-module/stable transport
    ↓
real tooling/incremental consumers
    ↓
C5 certification
    ↓
C6 generic evidence + production iterator migration
```

Do not reorder by attempting the production iterator migration before the C6 evidence relationship exists.
