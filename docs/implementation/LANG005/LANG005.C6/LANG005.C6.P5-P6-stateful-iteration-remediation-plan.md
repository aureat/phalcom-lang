---
id: LANG005.C6.P6
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG005.C6.P4
  - LANG005.C6.P5
follows: LANG005.C6.P5
supersedes: null
remediates:
  - LANG005.C6.P5-AUDIT-2026-09-18
prepared_against_revision: 683e05640a845f0c19ef53af91027139df38d012
---

# LANG005.C6.P6 — P5 Stateful Iteration Remediation and Corrective Closure

> **Plan class:** corrective closure / semantic soundness / stabilization  
> **Primary executor:** GPT-5.6 Luna High/Extra High or equivalent implementation agent  
> **Architectural authority:** this plan plus the accepted C6 specifications and any recorded consultation amendments  
> **Audit basis:** `LANG005.C6.P5-full-implementation-audit.md`  
> **Prepared against:** `aureat/phalcom-lang`, branch `main`, revision `683e05640a845f0c19ef53af91027139df38d012`

---

## 0. Executor contract

This is a patch-grade corrective implementation plan for the reopened `LANG005.C6.P5` stateful-iteration migration.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture and semantic ownership rules in this plan;
- must not improvise material architecture;
- must not preserve stale behavior merely because an active fixture currently expects it;
- must use the testing budget below rather than reflexively running broad suites;
- must classify and defer unrelated failures;
- must update `LANG005.C6-CHECKPOINT.md` at the named lifecycle points;
- must produce the required walkthrough and handoff;
- must STOP AND CONSULT when any global or task-local escalation trigger fires.

Testing is evidence gathering, not a ritual. Run the smallest command that answers the current correctness question.

This plan is corrective work inside the existing `LANG005.C6` checkpoint. It does **not** authorize unrelated C6 cleanup, broad type-system redesign, or workspace-wide certification work.

---

## 1. Goal

Repair the reopened P5 defects so Phalcom has one semantically authoritative stateful-iteration model across `for`, positional `*` expansion, iterator adapters, collection conveniences, active regression fixtures, current-facing documentation, and P5 completion records.

---

## 2. Checkpoint acceptance objective

This plan advances the existing `LANG005.C6` acceptance objective by providing **corrective closure of P5**, not by replacing the broader C6 checkpoint.

At successful completion:

1. `P5-AUD-001` through `P5-AUD-009` and `P5-AUD-012` have concrete repaired/accepted dispositions;
2. `P5-AUD-010`, `P5-AUD-011`, and `P5-AUD-013` have explicit evidence-based dispositions and no longer undermine P5 verification;
3. P5's original G4, G5, and G6 can be re-accepted from actual evidence;
4. current executable corpus and current-facing iteration documentation agree with the canonical stateful model;
5. the corrected P5 walkthrough/handoff and checkpoint record identify one exact final revision;
6. the C6 checkpoint remains truthful: it may still be `IN_PROGRESS / PARTIAL` until a successor plan performs broader C6 closure/certification.

This plan consumes the currently reserved-but-not-materialized `LANG005.C6.P6` slot. The existing checkpoint contains a placeholder P6 scope but no P6 plan file. Corrective work belongs to the checkpoint whose acceptance objective it repairs; therefore this remediation becomes P6. Broader C6 final closure moves to a successor plan, expected to be `LANG005.C6.P7`, unless the live checkpoint is amended differently before completion.

---

## 3. Repository grounding

### 3.1 Prepared revision

```text
repository: aureat/phalcom-lang
branch: main
revision: 683e05640a845f0c19ef53af91027139df38d012
audit revision: b7a73454c742e64973f4729ab577d91a2166238a
audit date: 2026-09-18
```

Repository investigation established:

- current `main` is 18 commits ahead of the audited P5 revision;
- those intervening commits change specification/documentation files only;
- no implementation or test file implicated by the P5 audit changed between the audited revision and this planning revision;
- therefore the audit's source/test findings remain live at the planning revision;
- the current checkpoint still records `active_plan: LANG005.C6.P6`, but no actual P6 plan file exists;
- the checkpoint still names P5 as the latest completed plan even though the audit requires P5 to be treated as reopened/partial;
- P5 walkthrough/handoff remain written against `revision: local-head` and contain claims contradicted by live source.

Planning inspection was source-based. No Rust or Phalcom test suite was executed during planning. The implementation agent owns the baseline/focused commands prescribed below.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

### 3.2 Live defects reconfirmed

The following audit findings were directly reconfirmed at `683e0564...`:

| Audit finding | Live repository fact |
|---|---|
| `P5-AUD-001` | `compiler/lib/expr.rs::compile_positional_expansion` still falls back to ordinary `iterator` / `next` getter sends when lowering metadata is missing. |
| `P5-AUD-002` | `modules/compile.rs` roots runtime trait environments for `analysis.for_iteration_lanes` but has no equivalent positional-expansion root walk. |
| `P5-AUD-003` | `iterator_laws.ph` still erases `FlatMapIterator`, `FlattenIterator`, `StrictZipIterator`, and `IndexedIterator` items to `Dynamic`; `ChainIterator` has no equal-item constraint. |
| `P5-AUD-004` | reusable collection classes still lack the required direct eager `map`, `map(indexed:)`, `filter`, `flatMap`, and concrete `isEmpty` surface. |
| `P5-AUD-005` | semantic iteration publication still requires `Item` normalization to yield `TypeId`, while `Tuple` and `RefRange` publish `Item = Dynamic`. |
| `P5-AUD-006` | single-lane `for` still retains the original `None` after `Dup` + `JumpIfNone`; scope teardown does not pop it. |
| `P5-AUD-007` | cursor-era and repeatable-pipeline fixtures remain active PASS fixtures. |
| `P5-AUD-008` | current docs/comments still include cursor-protocol descriptions. |
| `P5-AUD-009` | `TupleZipIterator` stores `List<Dynamic>` and performs ordinary dynamic `.iterator` / `.next` sends on heterogeneous lanes. |
| `P5-AUD-010` | inline semantic support remains compiler-side/path-category curated rather than a durable semantic-support abstraction. |
| `P5-AUD-011` | runtime type instantiation still falls back to `Dynamic` when an expected generic binding is absent. |
| `P5-AUD-012` | P5 completion records still lack exact SHA and required closure accounting. |
| `P5-AUD-013` | prior P5 closure commit remains broad/review-hostile; this plan must use focused corrective commits. |

### 3.3 Important newly resolved planning fact

The authoritative `docs/specs/iterator.md` explicitly states:

```text
Tuple uses Dynamic as its initial common iteration item type.
```

Therefore P5 remediation **must not** defer Tuple canonical iteration merely because `Dynamic` is not a `TypeId`.

The correct architecture is:

```text
Iterable proof                   remains static/evidence-backed
Iterator projection              remains a concrete semantic TypeId
Iterator proof                   remains static/evidence-backed
Item knowledge                   may be Known(...) or Dynamic(...)
yielded loop binding             carries that TypeKnowledge
compiler/runtime requirement     does not need a fake Dynamic TypeId
```

This is a fixed architectural decision in this plan.

---

## 4. Required reads before implementation

Read these before the first code edit.

### Workflow / execution

```text
AGENTS.md
docs/implementation/README.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/luna-implementer-prompt.md                 # if present in live tree
```

### Checkpoint / predecessor records

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-requirements-analysis.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-plan-00-shared-header.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-plan-01-semantic-and-for-cutover.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-plan-02-builtins-and-iterator-surface.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-plan-03-retirement-and-stabilization.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-handoff.md
LANG005.C6.P5-full-implementation-audit.md
```

### Normative specs

```text
docs/specs/README.md
docs/specs/iterator.md
docs/specs/callables/arguments.md
docs/specs/associated-types.md
docs/specs/generic-constraints.md
docs/specs/objects/traits.md
docs/specs/objects/impl.md
```

### Exact source paths

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/iteration.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/types/evidence.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/impls.rs

phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/compiler/boundedness.rs
phalcom-core/src/typing/environment.rs

phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/list.ph
phalcom-core/core/universe/src/collections/set.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/range.ph
phalcom-core/core/universe/src/collections/bytes.ph
phalcom-core/core/universe/src/scalar/string.ph

examples/sheetcalc/src/grid/ref.ph
examples/sheetcalc/src/grid/ref-range.ph
```

### Exact test/corpus paths

```text
phalcom-semantic/tests/semantic/README.md
phalcom-semantic/tests/semantic/capabilities/iteration.rs
phalcom-semantic/tests/semantic/capabilities/derived_conformance.rs

phalcom-core/tests/README.md
phalcom-core/tests/core/language/iteration.rs
phalcom-core/tests/core/language/derived_conformance.rs
phalcom-core/tests/core/collections/outgoing_packs.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
phalcom-core/tests/core/language/corpus.rs
phalcom-core/tests/fixtures/language/MANIFEST.md

phalcom-core/tests/fixtures/language/collections/range_cursor_protocol_direct.ph
phalcom-core/tests/fixtures/language/collections/map_wren_cursor_roundtrip.ph
phalcom-core/tests/fixtures/language/sequence/sequence_takeview_repeatable.ph
phalcom-core/tests/fixtures/language/iterator/iterator_direct_map_still_eager.ph
```

Avoid broad exploration unless evidence makes it necessary.

---

## 5. Normative authority

### 5.1 Normative behavior

The following rules are fixed.

#### Canonical iterability

A value is not canonically iterable merely because it has members named `iterator` and `next`.

Canonical implicit iteration is established through `Iterable` conformance and the associated `Iterator` proof.

#### Iterator/Iterable law

Conceptually:

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
}

trait Iterable
  where
    Self::Iterator |= Iterator,
    <Self::Iterator as Iterator>::Item == Self::Item
{
  type Item
  type Iterator
  iterator -> Self::Iterator
}
```

The derived law remains:

```text
I |= Iterator  =>  I |= Iterable
```

#### Positional `*`

The positional expansion rules are:

```text
Unit       -> direct empty positional product lane
Tuple      -> direct tuple positional lane only
otherwise  -> canonical Iterable expansion
```

A non-Unit, non-Tuple, non-Iterable value is invalid.

A real canonical iterator's ordinary runtime errors/control effects propagate.

#### Tuple item type

`Tuple` has canonical iteration `Item = Dynamic`.

This does **not** mean the compiler fabricates a static `Dynamic` `TypeId`. It means the yielded value knowledge is a deliberate dynamic boundary while executable conformance proof remains static.

#### Stateful lifecycle

A stored iterator/adapter pipeline is one-shot/stateful.

A reusable source's fresh `.iterator` / `.iter` acquisition creates a fresh traversal.

#### Adapter contracts

P5 Part II fixed these semantic relationships:

```text
map:       T -> U                     => Item = U
filter:    (T -> Bool)                => Item = T
filterMap: T -> Option<U>             => Item = U
flatMap:   T -> S, S |= Iterable      => Item = S::Item
flatten:   T |= Iterable              => Item = T::Item
chain:     I::Item == J::Item         => Item = I::Item
zip:       I::Item + J::Item          => Item = (I::Item, J::Item)
indexed:                               => Item = (Int, I::Item)
mapWhile:  T -> Option<U>             => Item = U
```

Do not erase these contracts to `Dynamic`.

#### Eager/lazy ownership split

```text
collection.map(...)       eager
collection.filter(...)    eager
collection.flatMap(...)   eager
collection.map(indexed:)  eager

collection.iter.map(...)       lazy/stateful
collection.iter.filter(...)    lazy/stateful
collection.iter.flatMap(...)   lazy/stateful
```

Reusable concrete sources retain non-consuming/source-specific `isEmpty`.

### 5.2 Current implementation is evidence, not authority

Current dynamic fallback, cursor fixtures, missing eager wrappers, and `Dynamic` adapter erasure are implementation defects because they conflict with the accepted model.

### 5.3 Future/non-normative proposals excluded here

This plan does not establish:

- heterogeneous tuple union inference;
- a new `Any`/top-type rule;
- a new generic runtime-erasure model;
- a new prelude/module-loader architecture;
- descending Range traversal;
- a new collection-builder abstraction;
- a new VM iterator opcode family.

---

## 6. Takeover state

Preserve the substantial P5 work that is already correct.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| canonical implicit iteration proof | `checker/iteration.rs::resolve_implicit_iteration` | semantic | compiler must not rediscover iterability |
| `for` lane product | `ForIterationLaneResolution` | semantic | one product per accepted lane |
| positional expansion product | `PositionalExpansionIterationResolution` | semantic | one product per accepted non-direct expansion |
| requirement resolution | `ForIterationRequirementResolution` | semantic | preserves exact/assumed/derived/abstract execution authority |
| lowering requirement spec | `ForIterationRequirementSpec` | semantic lowering | no runtime trait proof search |
| `for` lowering | `compiler/lib/loops.rs` | compiler | selected requirements only |
| positional Tuple/Unit fast lane | `PackTryExpandTuplePositionals`, `ListTryExpandTuplePositionals` | compiler/VM | product-only, not conformance proof |
| stateful iterator library | `collections/iterator_laws.ph` | Universe source | ordinary traced objects |
| runtime taxonomy | `Object -> Collection -> concrete collections` | native/runtime class graph | `Collection` does not imply `Iterable` |
| Map canonical item | `(key: K, value: V)` | Map/iterator conformance | direct Map iteration is entry iteration |
| boundedness | `compiler/boundedness.rs` | compiler heuristic | conservative; not a trait solver |
| associated Dynamic representation | `AssociatedTypeBindingValue::Dynamic` | semantic type system | dynamic boundary, not canonical TypeId proof |

Do not reintroduce the retired runtime `Iterable` superclass or cursor selectors.

---

## 7. Architecture

### 7.1 Authority direction

```text
source
  ↓
semantic conformance / associated projection / proof
  ↓
ForIteration... semantic products
  ↓
semantic-to-core lowering
  ↓
selected trait invocation / requirement-slot recipe
  ↓
compiler
  ↓
ordinary bytecode + frozen runtime environments
  ↓
VM/runtime execution
```

The compiler never answers “is this iterable?” by selector lookup.

The VM never trait-solves.

### 7.2 Dynamic `Item` architecture

Canonical iterable execution needs two different facts:

```text
A. how to execute iterator / next
B. what static knowledge is available for yielded Item
```

They must not be conflated.

Target semantic shape:

```rust
struct ...IterationResolution {
    source_expression: ExpressionId,
    source_range: SourceRange,
    source_type: TypeId,

    item_knowledge: TypeKnowledge, // Known or Dynamic
    iterator_type: TypeId,        // remains concrete/symbolic TypeId

    iterator: ForIterationRequirementResolution,
    next: ForIterationRequirementResolution,
}
```

The exact private struct name may differ, but this separation is fixed.

For `Tuple`:

```text
Iterable proof       established
Iterator projection  TupleIterator
Iterator proof       established
Item knowledge       Dynamic(ExplicitEscape)
```

For `List<Int>`:

```text
Iterable proof       established
Iterator projection  ListIterator<Int>
Iterator proof       established
Item knowledge       Known(Int)
```

The lowering/compiler does not require a fabricated item `TypeId` merely to execute iteration. If live search confirms `item_type` is unused outside publication/fingerprinting, remove it from lowering specs rather than introducing a fake runtime type.

### 7.3 Positional expansion architecture

```text
source expression
   ↓
Unit/Tuple direct product probe
   ├─ yes -> append product positionals
   └─ no
       ↓
required PositionalExpansionLoweringSpec
       ↓
preselected iterator requirement
       ↓
preselected next requirement loop
```

Missing lowering metadata after semantic acceptance is an internal compiler/projection defect, not permission for dynamic fallback.

### 7.4 Runtime evidence roots

The root collector must be source-construct neutral.

Both:

```text
analysis.for_iteration_lanes
analysis.positional_expansions
```

must feed one shared helper that roots exact/derived requirement environments.

Assumed evidence uses active requirement slots and does not invent runtime impl bindings.

### 7.5 Adapter architecture

Adapter classes remain source-authored ordinary objects. Their runtime state machines are largely already correct; remediation primarily restores formal generic contracts and associated `Item` precision.

No native heap arms or VM combinator opcodes.

### 7.6 `Tuple.zipped` policy

This plan resolves `P5-AUD-009` as follows:

> `Tuple.zipped` is an **explicit heterogeneous Dynamic boundary**, not an implicit-iterability proof mechanism.

Rationale:

- Tuple's normative common item type is `Dynamic`;
- current type theory does not publish per-element `Iterable` evidence for arbitrary heterogeneous tuple values;
- `Tuple.zipped` is an explicit method call, not `for` or `*`;
- ordinary dynamic sends on `Dynamic` are already a language boundary;
- accepting this boundary avoids inventing heterogeneous tuple conformance theory solely for this API.

Requirements:

- `Tuple.zipped` remains a reusable source;
- its iterator acquires each lane iterator exactly once;
- each attempted step advances every lane exactly once, left-to-right, before classification;
- all Some => yield tuple;
- all None => sticky exhaustion;
- mixed => `ZipLengthMismatchError`;
- its dynamic dispatch must not leak into `for`, `*`, generic `Iterable`, or semantic conformance;
- `docs/specs/iterator.md` must explicitly state this boundary.

If a newer live normative source requires static per-element proof, STOP AND CONSULT before implementing this policy.

---

## 8. Ownership boundaries

### 8.1 Owns

This plan owns:

- P5 audit remediation;
- canonical Dynamic-Item publication for implicit iteration;
- fail-closed positional expansion;
- runtime trait-environment rooting for expansion;
- single-lane `for` stack cleanup;
- iterator adapter type contracts;
- required eager collection wrappers;
- explicit `Tuple.zipped` dynamic-boundary policy;
- active affected corpus migration;
- current-facing iteration documentation/comments;
- P5 re-certification evidence and records.

### 8.2 Does not own

This plan does not own:

- full C6 release certification;
- unrelated generic-constraint closure outside iteration;
- higher-kinded type integration;
- new tuple-union type theory;
- global runtime generic-erasure redesign;
- unrelated semantic diagnostics debt;
- unrelated example/library modernization;
- general module/prelude redesign;
- performance optimization unrelated to iteration correctness.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| whether source canonically conforms to `Iterable` | semantic conformance engine | `for`, `*` semantic resolution | compiler/VM selector probing |
| `Iterable::Item` knowledge | projection normalization + semantic iteration product | loop binding/type flow | compiler fabricated type |
| iterator implementation selected | semantic evidence product | lowering/compiler/runtime recipe | runtime trait search |
| positional direct Tuple/Unit lane | language expansion semantics + VM product probe | compiler expansion | generic conformance shortcut |
| runtime trait environment | semantic evidence + module compiler root export | lowering/runtime dispatch | runtime inference |
| adapter output `Item` | Universe generic declarations/impl constraints | semantic checker | Dynamic erasure |
| collection eager transform ownership | concrete reusable source class | user API | duplicate generic eager algorithm |
| boundedness | compiler boundedness service | exhausting constructs | semantic conformance or VM |
| `Tuple.zipped` hetero lane dynamicity | explicit API/spec boundary | TupleZipIterator | implicit `Iterable` proof |
| completion state | `LANG005.C6-CHECKPOINT.md` | next plans/agents | stale walkthrough claims |

---

## 9. Global invariants

`INV-01` — Canonical implicit iterability is semantic evidence, never selector coincidence.

`INV-02` — Unit/Tuple positional `*` product handling remains a direct product lane and does not require `Iterable`.

`INV-03` — Non-Unit/non-Tuple positional `*` requires a canonical semantic expansion product.

`INV-04` — `Dynamic` associated `Item` is a deliberate yielded-value knowledge boundary, not a fake `TypeId` and not proof of conformance.

`INV-05` — The projected iterator type remains statically representable and proven `Iterator` even when `Item` is Dynamic.

`INV-06` — The compiler consumes semantic requirement products and never performs trait/conformance inference.

`INV-07` — The VM performs no iteration trait search or structural proof.

`INV-08` — Runtime trait environments required by both `for` and positional expansion are rooted from the same evidence rules.

`INV-09` — Unexpected missing runtime generic bindings are never accepted as evidence that the expansion-root path is correct.

`INV-10` — Single-lane `for` is evaluation-stack balanced on natural exhaustion, break, continue, and error paths.

`INV-11` — Multi-lane `for` keeps two-phase acquisition and strict lockstep behavior unchanged.

`INV-12` — Lazy adapters are stateful one-shot iterators; stored aliases share traversal state.

`INV-13` — Adapter associated `Item` contracts match values actually yielded.

`INV-14` — `chain` cannot legally combine unequal item types under one declared Item contract.

`INV-15` — Direct reusable collection transforms remain eager; `.iter` transforms remain lazy.

`INV-16` — `isEmpty` on reusable concrete sources is non-consuming and does not become a generic Iterator terminal.

`INV-17` — Map canonical item remains `(key: K, value: V)`.

`INV-18` — `Tuple` canonical item remains `Dynamic`; no heterogeneous union inference is added.

`INV-19` — `RefRange` publishes its actual `Ref` item type rather than Dynamic if the live type annotation is expressible.

`INV-20` — `Tuple.zipped` dynamicity is confined to that explicit API boundary and does not establish canonical conformance.

`INV-21` — No active PASS fixture teaches or depends on retired `iterate` / `iteratorValue` semantics.

`INV-22` — Historical documents may retain cursor history; current-facing authority must not.

`INV-23` — `Collection` ancestry does not establish `Iterable` conformance.

`INV-24` — Corrective commits remain focused and do not repeat P5's broad unrelated churn.

---

## 10. Non-goals

Do not:

- redesign the type lattice;
- add heterogeneous tuple element-union inference;
- add a new top type merely to represent Tuple iteration;
- restore or emulate the cursor protocol;
- restore the runtime `Iterable` class;
- add native iterator/adaptor heap variants;
- move trait solving into compiler/VM;
- convert `Collection` ancestry into iterability;
- add generic `Iterable.isEmpty`;
- create a generic collection builder/collector abstraction;
- optimize String/Map/Range iteration beyond P5 requirements;
- fix unrelated workspace failures;
- rewrite historical ADRs/plans to erase prior cursor history;
- run full workspace tests during build mode;
- turn the global missing-binding-to-Dynamic behavior into a broad type-system project unless a focused P5 invariant cannot otherwise be proven.

---

## 11. Expected impact map

### 11.1 Expected semantic source areas

- `phalcom-semantic/src/checker/analysis.rs` — iteration product item knowledge.
- `phalcom-semantic/src/checker/iteration.rs` — Dynamic Item normalization and fail-closed shared proof.
- `phalcom-semantic/src/checker/call.rs` — expansion resolution failure publication/diagnostic.
- `phalcom-semantic/src/checker/expression.rs` — list/tuple spread resolution failure publication.
- `phalcom-semantic/src/db/fingerprint.rs` — hash `TypeKnowledge` rather than unconditional item `TypeId`.
- possibly a small diagnostic helper/error enum path — only if current checker conventions require it.

### 11.2 Expected compiler/lowering areas

- `phalcom-core/src/modules/semantic_lowering.rs` — remove/replace unused item `TypeId` from executable iteration specs.
- `phalcom-core/src/modules/compile.rs` — shared runtime trait-root collection for implicit iteration sites.
- `phalcom-core/src/compiler/lib/expr.rs` — delete dynamic expansion fallback.
- `phalcom-core/src/compiler/lib/loops.rs` — natural-exhaustion stack cleanup.
- `phalcom-core/src/compiler/lib/error.rs` — internal missing-metadata error if no suitable existing variant exists.
- `phalcom-core/src/typing/environment.rs` — **read/instrument only by default**; global erasure semantics are not automatically rewritten.

### 11.3 Expected Universe source

- `collections/iterator_laws.ph` — typed adapter signatures/constraints.
- `collections/list.ph`
- `collections/set.ph`
- `collections/map.ph`
- `collections/tuple.ph`
- `collections/range.ph`
- `collections/bytes.ph`
- possibly `scalar/string.ph` only for stale comments or typed surfaces affected by shared defaults.
- `examples/sheetcalc/src/grid/ref-range.ph` — precise Ref item type.

### 11.4 Expected tests

- semantic iteration/derived-conformance tests;
- core language iteration/derived-conformance tests;
- outgoing pack tests;
- active iterator/sequence/collections/list/bytes/strings corpus;
- stack-discipline regression;
- adapter typing regressions.

### 11.5 Expected documentation/state

- `LANG005.C6-CHECKPOINT.md`
- corrected P5 walkthrough/handoff or audit-amended replacements
- new P6 walkthrough/handoff
- current iteration/collection/spread guides and production comments
- `docs/specs/iterator.md` only for the explicit `Tuple.zipped` Dynamic-boundary clarification, not for unrelated redesign.

### 11.6 Unexpected-touch rule

Touch adjacent helpers/tests when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem when that entry changes semantic authority, type representation, runtime lifetime/rooting, module identity, or public API beyond this plan.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. semantic analysis owns canonical iterability;
2. Unit/Tuple positional expansion keeps the direct product lane;
3. compiler dynamic `iterator`/`next` fallback for non-direct `*` must be deleted;
4. Tuple canonical `Item` remains `Dynamic`;
5. Dynamic Item is carried as dynamic knowledge, not fabricated as a TypeId;
6. iterator executable requirements remain statically proven;
7. runtime trait-root collection must cover both `for` and expansion;
8. `for` natural exhaustion must explicitly remove its retained `None`;
9. iterator adapter type relations from Part II must be expressed rather than erased;
10. direct built-in transformations remain eager;
11. `Tuple.zipped` is an explicit heterogeneous Dynamic boundary under the policy in §7.6;
12. active cursor fixtures must be migrated, not used to resurrect cursor APIs;
13. current docs must describe stateful iteration;
14. P5 cannot be re-marked complete until recertification gates pass.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- private helper names;
- exact internal enum/struct names;
- whether `item_knowledge` is stored directly as `TypeKnowledge` or as an equivalent semantic knowledge carrier, provided no fake `TypeId` is introduced;
- local helper factoring for root extraction;
- exact compiler error variant for missing required expansion metadata;
- exact adapter generic parameter counts;
- explicit extra type parameters when current source syntax cannot directly spell a nested projection;
- placement of focused tests within existing test modules;
- minor file-local refactors needed to satisfy Rust ownership/borrowing.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| lowering `item_type` is not consumed by compiler/runtime | search `ForIterationLoweringSpec.item_type`, `PositionalExpansionLoweringSpec.item_type` | if mechanically used only for debug/metadata, adapt; if execution depends on a TypeId, STOP AND CONSULT |
| `RefModule.Ref` or equivalent is legal in `RefRange` type annotations | live parser/semantic syntax + nearby module-qualified type use | use equivalent import/type spelling; if impossible without broad module/type changes, consult |
| method/impl constraints can express adapter associated projections/equalities | current generic syntax + C6 tests/spec | use explicit type parameters mechanically; if core relationship cannot be expressed without type-system change, consult |
| current `Tuple.zipped` behavior has no newer normative static-proof requirement | `docs/specs/iterator.md`, relevant current specs | if contradicted, consult |
| outgoing pack semantic diagnostics can reject invalid `*` before target activation | current semantic checker diagnostic ownership | adapt to existing diagnostic mechanism; do not fall back to runtime dNU |
| runtime root helper can be shared without changing environment identity | `modules/compile.rs` current root structures | if sharing changes key identity/layout, consult |
| current language-corpus target names remain those listed in §19 | `cargo test ... -- --list` if needed | adjust filters only |
| P6 remains unmaterialized at implementation start | checkpoint directory | if a real P6 plan now exists, STOP AND CONSULT about numbering/supersession |
| no source/test changes after `683e0564...` invalidate audit | `git log`, `git diff`, named path rereads | mechanical drift -> adapt; architectural drift -> consult |

---

## 13. Global STOP / CONSULT triggers

STOP AND CONSULT if any of the following occurs:

1. canonical iteration cannot carry `Dynamic` item knowledge without changing the meaning of `Dynamic`;
2. iterator executable selection would require a Dynamic/runtime proof path;
3. a compiler or VM selector probe appears necessary to make `for` or `*` work;
4. removing expansion fallback requires changing ordinary Dynamic message-send semantics;
5. expansion diagnostics cannot be produced without materially redesigning generic call application;
6. runtime trait environment identity/rooting must be redesigned rather than shared mechanically;
7. a required adapter contract cannot be stated without cross-cutting generic/type-system work;
8. a test oracle from a current authoritative spec conflicts with this plan;
9. `Tuple.zipped` is found to have a newer normative static-conformance requirement;
10. `RefRange` cannot publish `Ref` without a material module/type-identity change;
11. the single-lane stack repair changes break/continue/upvalue semantics;
12. an invariant must be weakened to get tests green;
13. the same nontrivial semantic failure persists after one serious correction;
14. two plausible fixes have materially different architecture;
15. a new VM/native iterator representation seems necessary;
16. a second source of canonical conformance/projection truth would be introduced;
17. unexpected GC/lifetime or generic reification behavior becomes correctness-critical;
18. scope expands materially beyond iteration remediation;
19. a real `LANG005.C6.P6` plan appears and numbering/supersession becomes ambiguous.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

A cycle is:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

Examples:

- Rust import/signature drift;
- renamed helper;
- borrow-check cleanup;
- source-generic spelling mismatch with an obvious equivalent;
- test filter selecting zero tests.

### 14.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

### 14.4 Test rerun rule

Never rerun an unchanged failing test unless something relevant to that failure changed.

---

## 15. Testing surface analysis

### 15.1 Coverage obligations

| Coverage ID | Dimension | Invariant(s) | Required case | Audit / legacy coverage |
|---|---|---|---|---|
| `P6-CV-01` | canonical Dynamic item | INV-04/05/18 | `for` over Tuple publishes canonical lane and Dynamic loop item knowledge | AUD-005 / P5-CV-34 |
| `P6-CV-02` | precise external item | INV-19 | RefRange canonical item is `Ref`, generic Iterable consumer works | AUD-005 |
| `P6-CV-03` | exact expansion | INV-01/03 | concrete List/Range expansion succeeds via evidence | AUD-001 |
| `P6-CV-04` | assumed expansion | INV-03/08 | generic `T |= Iterable` expansion executes | AUD-001/002 |
| `P6-CV-05` | derived expansion | INV-03/08 | generic `I |= Iterator` expands through derived Iterable | AUD-001/002 |
| `P6-CV-06` | conditional/derived environment | INV-08/09 | expansion whose selected conformance needs impl bindings executes | AUD-002/011 |
| `P6-CV-07` | structural negative | INV-01 | `iterator`/`next` lookalike with no conformance is rejected semantically | AUD-001 |
| `P6-CV-08` | invalid scalar | INV-01/03 | `*42` fails before receiver target activation/runtime iterator lookup | AUD-001 |
| `P6-CV-09` | valid iterator failure | INV-03 | error thrown by valid canonical iterator propagates | AUD-001 |
| `P6-CV-10` | direct product lane | INV-02 | Unit empty lane + Tuple positional-only lane unchanged | P5 expansion coverage |
| `P6-CV-11` | source ordering | INV-03 | source evaluated once; expansion finishes before later argument | outgoing packs |
| `P6-CV-12` | boundedness composition | INV-03 | open Range rejected; `take(n)` expansion accepted | P5 boundedness |
| `P6-CV-13` | stack repetition | INV-10 | many naturally exhausted inner single-lane `for`s in one activation do not grow stack | AUD-006 |
| `P6-CV-14` | break exit | INV-10 | break path does not execute exhaustion-sentinel Pop | AUD-006 |
| `P6-CV-15` | continue/upvalue | INV-10/11 | continue/capture behavior remains correct | AUD-006 |
| `P6-CV-16` | map typing | INV-13 | `Iterator<Int>.map(String mapper)` yields Item String | AUD-003 |
| `P6-CV-17` | filter typing | INV-13 | filter preserves Item | AUD-003 |
| `P6-CV-18` | filterMap typing | INV-13 | filterMap returns U | AUD-003 |
| `P6-CV-19` | chain positive | INV-13/14 | equal Items accepted | AUD-003 |
| `P6-CV-20` | chain negative | INV-14 | mismatched Items rejected semantically | AUD-003 |
| `P6-CV-21` | flatten contract | INV-13 | non-Iterable inner Item rejected | AUD-003 |
| `P6-CV-22` | flatMap contract | INV-13 | mapper result must conform Iterable | AUD-003 |
| `P6-CV-23` | zip shape | INV-13 | zip preserves both lane Item types | AUD-003 / P5-CV-85 |
| `P6-CV-24` | indexed shape | INV-13 | indexed yields `(Int, T)` | AUD-003 / P5-CV-51 |
| `P6-CV-25` | eager direct map | INV-15 | List direct map executes callback eagerly and returns List | AUD-004 / P5-CV-75/102 |
| `P6-CV-26` | eager wrappers | INV-15 | direct filter/flatMap/map(indexed:) eager | AUD-004 |
| `P6-CV-27` | non-consuming source query | INV-16 | concrete isEmpty does not consume traversal | AUD-004 |
| `P6-CV-28` | one-shot lifecycle | INV-12 | same stored pipeline remains exhausted | AUD-007 / P5-CV-72 |
| `P6-CV-29` | fresh lifecycle | INV-12 | rebuilding from `source.iter` restarts independently | AUD-007 / P5-CV-71/73 |
| `P6-CV-30` | explicit dynamic zip | INV-20 | Tuple.zipped strict state machine works and dynamicity does not authorize `for`/`*` lookalikes | AUD-009 |
| `P6-CV-31` | cursor retirement corpus | INV-21 | no active PASS fixture invokes `iterate`/`iteratorValue` | AUD-007 |
| `P6-CV-32` | docs/current authority | INV-22 | current-facing census contains no cursor-as-current semantics | AUD-008 |
| `P6-CV-33` | runtime taxonomy | INV-23 | `class Fake is Collection` is not canonically Iterable | P5 T14 preservation |
| `P6-CV-34` | record integrity | — | exact final SHA + P5-CV closure matrix + actual commands/results | AUD-012 |
| `P6-CV-35` | source hygiene | INV-24 | final diff contains no unrelated churn; classified residue only | AUD-013 |

### 15.2 Existing P5 coverage obligations

Do not replace the original P5 coverage ledger with only the new rows.

The final walkthrough must also disposition every existing `P5-CV-*` row from the P5 plans. P6 rows are remediation discriminators; the P5 rows remain the original completion contract.

---

## 16. Verification execution budget

### 16.1 BUILD MODE

Goal: implement with sparse, discriminating feedback.

Mandatory during build:

- one exact semantic or runtime reproducer per task when useful;
- focused semantic `capabilities::iteration` after semantic-product/expansion changes;
- exact core test filters for stack/expansion/adapter changes;
- exact corpus lane only when a task edits that lane.

Do **not** run during ordinary build steps:

```text
cargo test --workspace
cargo clippy --workspace
full phalcom-core integration binary without filter
all language-corpus lanes
unrelated examples
unrelated semantic golden suites
```

### 16.2 STABILIZE MODE

Run coherent feature gates after grouped tasks:

- semantic canonical iteration/derived conformance;
- core iteration/derived conformance;
- outgoing packs;
- iterator/sequence/collections affected corpus;
- affected crate checks.

### 16.3 CERTIFY MODE

Run only the P6 re-certification gates in §19.

Full workspace/release certification is explicitly deferred to the successor C6 closure plan unless the user separately authorizes it.

### 16.4 Verification ladder

```text
T0 — exact reproducer / new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — affected crate/package checks
T5 — workspace/release verification (deferred)
```

---

## 17. Baseline / unrelated failure policy

Classify every unexpected failure:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated / baseline
```

Policy:

- A/B: active responsibility;
- C: one bounded classification pass; if nonblocking and still unclear, record and continue;
- D: record and continue immediately;
- do not consume adviser compute on D unless it blocks acceptance;
- do not weaken assertions, disable tests, or update broad diagnostic baselines merely to make gates green.

Record deferred failures in `LANG005.C6-CHECKPOINT.md`.

---

# 18. Tasks

## T0 — Reopen P5 state and establish the corrective baseline

### Purpose

Make checkpoint state truthful before substantial implementation begins.

### Preconditions

- clean understanding of current `git status --short`;
- P6 plan file is the active authorized plan;
- no real conflicting P6 plan has appeared.

### Consumes

- P5 audit;
- current C6 checkpoint;
- P5 walkthrough/handoff.

### Produces

- truthful active-plan/checkpoint state;
- starting revision record;
- explicit audit remediation ledger.

### Files and symbols

**Read:**

```text
LANG005.C6-CHECKPOINT.md
LANG005.C6.P5-walkthrough.md
LANG005.C6.P5-handoff.md
```

**Modify:**

```text
LANG005.C6-CHECKPOINT.md
```

**Create:**

```text
this P6 plan file if not already committed
```

### Required implementation shape

1. Inspect `git status --short`; preserve unrelated work.
2. Confirm current revision descends from the planning revision or classify drift.
3. Amend the plan ledger so P5 is no longer represented as unconditionally accepted after the audit.
4. Mark P6 active/in progress.
5. Record an audit-remediation ledger mapping `P5-AUD-001..013` to P6 tasks.
6. Replace the old placeholder P6 description with this corrective scope.
7. Set the expected successor to P7 for broader C6 closure unless live workflow authority dictates another number.

Do not rewrite P5 walkthrough/handoff yet; they are evidence records to be corrected after implementation.

### Forbidden approaches

- no source edits in T0;
- no pretending P5 was never implemented;
- no deleting old completion records;
- no broad checkpoint history rewrite.

### Tests to run now

> **Do not run runtime/integration tests after this task.** T0 changes execution records only.

### Tests explicitly deferred

All implementation tests.

### Acceptance

Checkpoint state clearly says P5 is under corrective remediation and P6 is active.

### Local STOP / CONSULT triggers

- a real P6 plan now exists with conflicting authorized scope;
- checkpoint lifecycle conventions materially differ from the inspected convention.

### Checkpoint update

This task is itself the start-of-plan checkpoint update.

---

## T1 — Make canonical implicit iteration Dynamic-Item aware

### Purpose

Close the architectural half of `P5-AUD-005`: allow a canonically proven Iterable to yield `Dynamic` items without requiring a fake/static `TypeId`.

### Preconditions

- T0 complete;
- normative Tuple `Item = Dynamic` reconfirmed.

### Consumes

- `AssociatedTypeBindingValue::Dynamic`;
- `ProjectionNormalizationResult::Dynamic`;
- `TypeKnowledge::Dynamic`;
- current `resolve_implicit_iteration`.

### Produces

- semantic iteration products carrying item knowledge;
- Tuple canonical `for` lane publication;
- no loss of static Iterator proof.

### Files and symbols

**Read:**

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/iteration.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/types/{evidence,projection}.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-core/src/modules/semantic_lowering.rs
```

**Modify expected:**

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/iteration.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-core/src/modules/semantic_lowering.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/iteration.rs
phalcom-core/tests/core/language/iteration.rs
```

### Required implementation shape

#### T1.1 Separate Item knowledge from executable iterator type

Change both semantic public products:

```text
ForIterationLaneResolution
PositionalExpansionIterationResolution
```

so yielded item information is not an unconditional `TypeId`.

Preferred:

```rust
pub item_knowledge: TypeKnowledge,
pub iterator_type: TypeId,
```

Remove `item_type: TypeId` unless a verified consumer genuinely requires it.

The internal `ImplicitIterationResolution` should follow the same model.

#### T1.2 Normalize Item projection with explicit Dynamic handling

Do not call `.type_id()` and reject every non-TypeId result.

Required conceptual logic:

```text
normalize Iterable::Item
    Normalized/ Symbolic TypeId
        -> Known item knowledge with inherited Established/Assumed strength

    Dynamic boundary
        -> TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)

    incomplete/blocked/ambiguous/recursive/cancelled/budget/internal
        -> preserve existing non-success semantic outcome
```

Do not collapse unrelated terminal states into Dynamic.

#### T1.3 Keep Iterator projection strict

`Iterable::Iterator` must still normalize to a static/symbolic `TypeId` usable to prove:

```text
projected iterator type |= Iterator
```

If Iterator itself normalizes Dynamic, the implicit iteration proof is not executable and must not publish a lane.

#### T1.4 Publish appropriate expression status

When Item knowledge is Dynamic, use the repository's established Dynamic-boundary status convention rather than pretending it is Known.

The lane can still be executable/valid because conformance and iterator execution are proven.

#### T1.5 Lowering

Verify whether lowering `item_type` is unused outside construction.

Expected repair:

```text
ForIterationLoweringSpec
PositionalExpansionLoweringSpec
```

drop `item_type` because execution needs only:

```text
iterator_type
iterator requirement
next requirement
```

If a consumer needs item knowledge for a legitimate later compiler feature, carry an explicit knowledge descriptor rather than forging a TypeId.

#### T1.6 Fingerprints

Hash `item_knowledge` deterministically in iteration/expansion fingerprints.

Use existing TypeKnowledge/DynamicReason hashing conventions; do not hash addresses.

### Forbidden approaches

- no synthetic nominal `Dynamic` TypeId;
- no compiler Tuple special case;
- no treating Dynamic as static proof of `Iterator`;
- no changing Tuple to a homogeneous union or `Any`;
- no bypassing associated projection normalization.

### Test changes required

Cover:

- `P6-CV-01`;
- existing P5 Tuple Dynamic coverage.

Required semantic assertions:

1. Tuple exact `Iterable` conformance exists;
2. Tuple Item projection normalizes to Dynamic;
3. a `for` lane is still published;
4. the lane's item knowledge is Dynamic;
5. iterator requirement remains selected/proven;
6. structural lookalikes remain unrelated to this change.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration tuple -- --nocapture
```

If the exact core filter selects zero tests, list `language::iteration` and use the narrow matching name.

### Tests explicitly deferred

- outgoing packs;
- adapter suites;
- language corpus;
- workspace tests.

### Acceptance

Tuple canonical iteration publishes through the ordinary semantic path with Dynamic yielded item knowledge, and no execution layer requires a fake Dynamic TypeId.

### Local STOP / CONSULT triggers

- a downstream runtime/compiler consumer materially requires Item as TypeId;
- Dynamic lane publication changes conformance identity or runtime trait-selection rules;
- the semantic status model cannot represent “executable proof + Dynamic yielded value” without a new cross-cutting state.

### Checkpoint update

Record the durable fact:

```text
canonical implicit iteration separates executable Iterator proof from yielded Item knowledge;
Tuple Item is Dynamic without compiler special-casing.
```

---

## T2 — Give RefRange its real item type

### Purpose

Close the RefRange half of `P5-AUD-005` without unnecessarily routing a homogeneous domain type through Dynamic.

### Preconditions

- T1 complete;
- `Ref` class identity verified.

### Consumes

- `examples/sheetcalc/src/grid/ref.ph::Ref`;
- canonical Iterator/Iterable law.

### Produces

```text
RefRangeIterator::Item = Ref
RefRange::Item = Ref
next -> Option<Ref>
```

using the live module-qualified/import spelling.

### Files and symbols

**Read:**

```text
examples/sheetcalc/src/grid/ref.ph
examples/sheetcalc/src/grid/ref-range.ph
```

**Modify:**

```text
examples/sheetcalc/src/grid/ref-range.ph
```

**Tests:**

Use an existing Sheetcalc/example harness if present. If none exists, add the smallest repository-appropriate source fixture that imports the example module without creating a new test framework.

### Required implementation shape

- preserve row-major traversal;
- preserve iterator state and sticky exhaustion;
- replace `Option<Dynamic>` with the real Ref type;
- replace both associated Item bindings with that real Ref type;
- do not change Ref identity/equality/hash.

### Forbidden approaches

- no Dynamic merely because it compiles;
- no nominal duplication of Ref;
- no new core Universe dependency on Sheetcalc.

### Test changes required

`P6-CV-02`:

- 1-cell range;
- multi-row/multi-column row-major order;
- two independent iterators;
- sticky exhaustion;
- generic `T |= Iterable` consumer sees Ref item type if harness permits.

### Tests to run now

Run the exact existing Sheetcalc/example test command discovered from the live repository.

If no dedicated executable test harness exists:

> **Do not run broad workspace tests merely to exercise Sheetcalc.** Add the smallest appropriate focused integration/fixture and run only it.

### Tests explicitly deferred

Core iterator corpus.

### Acceptance

RefRange's canonical associated Item is precise and no longer depends on Dynamic iteration publication.

### Local STOP / CONSULT triggers

- module-qualified Ref type cannot be represented without a broader module/type-language change.

### Checkpoint update

Only if this establishes a reusable example migration rule worth recording.

---

## T3 — Make positional expansion fail closed and remove structural fallback

### Purpose

Close `P5-AUD-001` and restore one definition of canonical implicit iterability.

### Preconditions

- T1 Dynamic-Item publication complete;
- direct Unit/Tuple product lane still green.

### Consumes

- `resolve_positional_expansion`;
- `PositionalExpansionIterationResolution`;
- `PositionalExpansionLoweringSpec`;
- product-only VM fast lane.

### Produces

- semantic invalidation/diagnostic on unresolved non-direct `*`;
- mandatory lowering spec after direct product probe;
- no ordinary `iterator`/`next` compiler fallback.

### Files and symbols

**Read/Modify:**

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/iteration.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/error.rs          # only if needed
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/iteration.rs
phalcom-core/tests/core/collections/outgoing_packs.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
```

### Required implementation shape

#### T3.1 Stop discarding expansion failure

Current callers do:

```rust
let (resolution, _) = resolve_positional_expansion(...);
```

Replace that with explicit handling.

For a statically Known source that is not a direct Unit/Tuple product and cannot prove canonical Iterable:

- publish the normal semantic diagnostic/invalid state for an invalid expansion source;
- do not merely omit `positional_expansions`.

Use existing `UnresolvedApplicationReason` -> diagnostic/status mechanisms where possible.

For Dynamic/Unknown source cases, preserve the accepted language semantics from the call-arguments spec and current semantic authority. Do not invent runtime structural acceptance.

#### T3.2 List-literal spread recovery

`checker/expression.rs` may still need a best-effort element type for recovery.

If canonical expansion proof fails:

- emit/retain the invalid expansion semantic fact;
- any `project_applied_argument` fallback may be used only for downstream type recovery;
- it must not turn the expression back into a Ready/valid expansion;
- lowering metadata must not be published.

#### T3.3 Compiler metadata is required

After:

```text
PackTryExpandTuplePositionals / ListTryExpandTuplePositionals
```

returns false, obtain:

```text
lowering.positional_expansions[expr_range]
```

as required metadata.

Delete both:

```rust
emit_getter_send("iterator")
emit_getter_send("next")
```

fallback branches.

If the required spec is absent in a semantically accepted compilation, return an internal compiler/projection error with source range.

#### T3.4 Rewrite structural-positive tests

Replace the current structural `Countdown`/`CountdownIterator` source with real canonical declarations:

```text
impl Iterator for CountdownIterator
impl Iterable for Countdown
```

using actual source syntax.

Rewrite the valid error-propagation test so `Broken` is canonically Iterable and its selected iterator body throws.

#### T3.5 Negative tests

Add:

```text
class LookalikeIterator { next ... }
class Lookalike { iterator ... }
```

without conformances and prove:

```text
Receiver.new().collect(*Lookalike.new())
```

is rejected before runtime structural lookup.

Rewrite the `*42` test: it must no longer expect:

```text
does not understand 'iterator'
```

It should expect the canonical semantic invalid-expansion diagnostic and prove the receiver target/body is not activated.

### Forbidden approaches

- no compiler Collection ancestry check;
- no ordinary dynamic sends as fallback;
- no VM trait search;
- no acceptance of a lookalike because it has matching method names;
- no changing Dynamic ordinary message-send semantics to solve this;
- no deleting direct Unit/Tuple lane.

### Test changes required

Cover:

```text
P6-CV-03
P6-CV-07
P6-CV-08
P6-CV-09
P6-CV-10
P6-CV-11
P6-CV-12
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs positional_spread -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion non_iterable_star_operand -- --nocapture
```

Then, at coherent T3 completion:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion
```

### Tests explicitly deferred

- full core binary;
- iterator/sequence corpus;
- workspace.

### Acceptance

There is no reachable ordinary `iterator`/`next` fallback in `compile_positional_expansion`, and structural lookalikes are rejected semantically.

### Local STOP / CONSULT triggers

- current normative Dynamic `*source` semantics explicitly require runtime acceptance with no canonical token;
- semantic checker cannot diagnose invalid expansion without redesigning canonical call application;
- direct Tuple product lane becomes entangled with Iterable proof.

### Checkpoint update

Record:

```text
non-direct positional * requires canonical semantic evidence;
compiler fallback removed.
```

---

## T4 — Root positional-expansion runtime trait environments

### Purpose

Close `P5-AUD-002` and make exact/derived generic expansion execution independent of accidental Dynamic erasure.

### Preconditions

- T3 fail-closed expansion complete.

### Consumes

- `add_runtime_trait_environment_roots`;
- `ForIterationRequirementResolution`;
- runtime trait environment binding assembly.

### Produces

- one shared implicit-iteration root collector used by both `for` and positional expansion.

### Files and symbols

**Read/Modify:**

```text
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs    # verification/minor factoring only
phalcom-core/src/typing/environment.rs           # read/instrument by default
```

**Tests:**

```text
phalcom-core/tests/core/collections/outgoing_packs.rs
phalcom-core/tests/core/language/derived_conformance.rs
```

### Required implementation shape

Factor the current `for_iteration_lanes` root walk into a helper conceptually like:

```rust
fn collect_implicit_iteration_requirement_roots(
    ...
    range: SourceRange,
    iterator: &ForIterationRequirementResolution,
    next: &ForIterationRequirementResolution,
)
```

or equivalent.

Call it for both:

```text
analysis.for_iteration_lanes.values()
analysis.positional_expansions.values()
```

Preserve exact existing root-key identity rules:

- Exact evidence => source impl environment;
- Derived conditional source => exact source impl environment;
- Derived guarded rule => rule impl bindings;
- Assumed evidence => no extra impl root; active conformance slots own it;
- Abstract requirement => no exact runtime impl root.

Do not duplicate the same evidence match in two separate loops after this patch.

### Missing-binding risk (`P5-AUD-011`)

The global runtime environment currently degrades an absent parameter binding to Dynamic.

Do **not** redesign that global policy in this task unless the focused evidence proves P5 correctness cannot be established otherwise.

Instead:

1. add at least one expansion test whose correct dispatch/result depends on a generic binding;
2. ensure the test fails if the positional-expansion root walk is temporarily omitted;
3. if practical, add a debug/internal assertion at the trait-environment attachment boundary for an expected binding rather than changing global erasure;
4. record the broader explicit-erasure-vs-missing-binding issue as a follow-up risk if still present.

### Forbidden approaches

- no runtime inference from values;
- no “if missing use Dynamic” as the intended expansion path;
- no duplicate root collector;
- no changed stable environment identity without consultation.

### Test changes required

Add:

```text
P6-CV-04
P6-CV-05
P6-CV-06
```

At least one case must use a selected generic/derived conformance with impl bindings that are observed at runtime.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs generic_spread -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance -- --nocapture
```

Run the exact newly added expansion environment test by name.

### Tests explicitly deferred

Broad semantic incremental suite until G1.

### Acceptance

Expansion and `for` root the same categories of runtime trait evidence, and a binding-dependent expansion test proves the path.

### Local STOP / CONSULT triggers

- root identity/keying must change;
- derived evidence cannot be represented by existing runtime recipe structure;
- the only way to make the test pass is relying on missing-binding Dynamic fallback.

### Checkpoint update

Record the shared root-collection invariant.

---

## T5 — Repair single-lane `for` stack discipline

### Purpose

Close `P5-AUD-006` without changing loop semantics.

### Preconditions

- no pending compiler-control-flow consultation from T3/T4.

### Consumes

- existing single-lane canonical `for` lowering;
- direct-jump break/continue machinery;
- `JumpIfNone` stack behavior.

### Produces

- natural exhaustion removes the retained original `None`;
- break does not execute exhaustion-only cleanup.

### Files and symbols

**Read:**

```text
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/scope.rs
relevant jump/break helpers
```

**Modify:**

```text
phalcom-core/src/compiler/lib/loops.rs
```

**Tests:**

```text
phalcom-core/tests/core/language/iteration.rs
iteration disassembly helpers if appropriate
```

### Required implementation shape

Current shape:

```text
next
dup
JumpIfNone exit
extract payload
...
loop
exit
```

Required shape:

```text
loop:
    next
    dup
    JumpIfNone natural_exhaustion

    GetVariantPayload
    ...
    body
    Loop

natural_exhaustion:
    Pop                 # remove retained original None

common_exit:
    ...
```

`break` jumps target `common_exit`, not `natural_exhaustion`.

Continue behavior remains unchanged.

If compiler helpers make a distinct label cumbersome, an equivalent control-flow shape is acceptable only if ownership of the retained sentinel is explicit and break cannot pop a non-existent sentinel.

### Forbidden approaches

- no generic `end_scope` stack popping;
- no cleanup on break that assumes None exists;
- no changing `JumpIfNone` globally to compensate;
- no VM opcode semantic change.

### Test changes required

Add:

```text
P6-CV-13
P6-CV-14
P6-CV-15
```

Preferred stack regression:

- one activation repeatedly enters an inner single-lane `for` that naturally exhausts;
- enough repetitions to expose linear stack growth under the old lowering;
- result remains deterministic.

Also retain/extend break/continue capture tests.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration <new-stack-test-name> -- --nocapture
```

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
```

### Tests explicitly deferred

Outgoing packs and language corpus.

### Acceptance

Single-lane natural exhaustion is stack-balanced and existing break/continue tests remain green.

### Local STOP / CONSULT triggers

- stack cleanup requires changing VM branch semantics;
- captured-local closing semantics differ between natural exhaustion and break in a way not represented by current compiler helpers.

### Checkpoint update

Only record if this changes a durable lowering invariant.

---

## G1 — Canonical iteration authority gate

### Purpose

Prove semantic authority, Dynamic Item publication, fail-closed expansion, evidence transport, and stack discipline before API-surface work.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion
```

Expected:

```text
all commands PASS
structural lookalike rejected semantically
*42 no longer fails via 'does not understand iterator'
Tuple for lane publishes with Dynamic Item knowledge
generic/derived expansion runtime bindings execute
single-lane stack regression passes
```

Do not broaden after PASS.

On failure, classify before editing.

Checkpoint update: record G1 as one coherent verification entry.

---

## T6 — Restore sound typed contracts for the iterator adapter toolbox

### Purpose

Close `P5-AUD-003`: make the source-authored adapter state machines tell the same type story as the values they yield.

### Preconditions

- G1 passed;
- canonical associated projection/equality machinery is stable.

### Consumes

- current adapter state machines;
- generic method syntax;
- associated projections/equality constraints.

### Produces

- typed lazy adapter methods;
- typed adapter conformance declarations;
- semantic rejection of invalid compositions.

### Files and symbols

**Read/Modify:**

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
```

**Read for syntax examples:**

```text
phalcom-core/core/universe/src/option/option.ph
phalcom-core/core/universe/src/errors/result.ph
current generic constraint specs/tests
```

**Tests:**

Prefer semantic capability tests for type relationships, plus existing runtime adapter tests for state-machine behavior.

### Required implementation shape

#### T6.1 Upstream constraints

Every generic adapter whose body calls `.next` through a type parameter must have the appropriate static `Iterator` constraint.

Conceptually:

```phalcom
class MappedIterator<I, U>
  where I |= Iterator
{ ... }
```

or equivalent accepted placement on impl/method.

#### T6.2 `map`

Target semantic signature:

```text
map<U>(f: (Self::Item) -> U) -> MappedIterator<Self, U>
```

Adapter Item:

```text
U
```

#### T6.3 `filter`

Predicate:

```text
(Self::Item) -> Bool
```

Adapter Item:

```text
Self::Item
```

#### T6.4 `filterMap`

Mapper:

```text
(Self::Item) -> Option<U>
```

Adapter Item:

```text
U
```

#### T6.5 `flatMap`

Required relationship:

```text
mapper: Self::Item -> S
S |= Iterable
Item = <S as Iterable>::Item
```

The current inner iterator is acquired exactly once per mapped source.

If live syntax cannot express the nested associated projection cleanly, add explicit mechanical type parameters representing `S::Item` / `S::Iterator` with equality constraints. Do not erase to Dynamic.

#### T6.6 `flatten`

Required relationship:

```text
<Self as Iterator>::Item |= Iterable
Item = <<Self as Iterator>::Item as Iterable>::Item
```

Again, explicit bridge type parameters are permitted if needed.

#### T6.7 `chain`

Require:

```text
J |= Iterator
<J as Iterator>::Item == <I as Iterator>::Item
```

Then:

```text
Item = <I as Iterator>::Item
```

A mismatched chain must be a semantic error.

#### T6.8 `zip`

Require both sides Iterator.

Item must be the ordinary tuple/product type:

```text
(<I as Iterator>::Item, <J as Iterator>::Item)
```

Do not use Dynamic.

#### T6.9 `indexed`

Item must be:

```text
(Int, <I as Iterator>::Item)
```

using the live canonical integer type spelling.

#### T6.10 `mapWhile`

Mapper returns `Option<U>` and Item is U.

#### T6.11 Remaining adapters

`skip`, `skipWhile`, `take`, `takeWhile`, `inspect` preserve upstream Item and must explicitly require upstream Iterator where needed.

Do not alter their runtime state-machine laws unless a focused type correction exposes a real behavior bug.

### Forbidden approaches

- no blanket Dynamic;
- no untyped callback merely to avoid generic method syntax;
- no duplicate adapter implementation just for typed variants;
- no VM/native combinator floor;
- no weakening chain equality.

### Test changes required

Add semantic discriminators:

```text
P6-CV-16 .. P6-CV-24
```

At minimum:

```text
map Int -> String
filter preserves Int
filterMap Int -> String
chain Int/Int accepted
chain Int/String rejected
flatten non-Iterable rejected
flatMap mapper non-Iterable rejected
zip Item is tuple of lane items
indexed Item is (Int,T)
```

Retain runtime tests for laziness, sticky exhaustion, GC, and strict zip.

### Tests to run now

Start with exact new semantic test names.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
```

If the live adapter stateful test module exists under the core binary, run its exact filter as well.

### Tests explicitly deferred

Sequence/collections full affected corpus until T7/T9.

### Acceptance

All adapter associated Item relationships are sound and invalid compositions are rejected semantically.

### Local STOP / CONSULT triggers

- one core relationship cannot be stated with current generic/equality syntax without type-system changes;
- callback typing requires changing callable semantics;
- existing associated-projection normalization cannot handle a required nested projection and no explicit-parameter encoding exists.

### Checkpoint update

Record only the durable adapter contract surface, not individual helper details.

---

## T7 — Restore inherent eager collection transforms and concrete `isEmpty`

### Purpose

Close `P5-AUD-004` and restore the accepted user-facing eager/lazy role split before corpus migration.

### Preconditions

- T6 typed adapters complete.

### Consumes

- canonical lazy adapters;
- Iterable materialization terminals;
- concrete collection `size`.

### Produces

On the accepted reusable families:

```text
map
map(indexed:)
filter
flatMap
isEmpty
```

### Files and symbols

**Modify expected:**

```text
phalcom-core/core/universe/src/collections/list.ph
phalcom-core/core/universe/src/collections/set.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/range.ph
phalcom-core/core/universe/src/collections/bytes.ph
```

**Tests:**

```text
phalcom-core/tests/fixtures/language/iterator/iterator_direct_map_still_eager.ph
additional direct eager filter/flatMap/indexed fixtures or focused core tests
```

### Required implementation shape

Use thin forwarding only.

Conceptually:

```phalcom
map<U>(_ f: (Item) -> U) {
  self.iterator.map(f).toList
}

filter(_ p) {
  self.iterator.filter(p).toList
}

flatMap<S>(_ f) {
  self.iterator.flatMap(f).toList
}
```

Use exact accepted typing/current method syntax per class.

`map(indexed:)` delegates through canonical `indexed` + lazy map + materialization rather than duplicating traversal loops.

`isEmpty` is inherent/source-specific.

For sources with exact size semantics:

```phalcom
isEmpty { self.size == 0 }
```

Range keeps its current `size` behavior, including errors for unsupported/unbounded shapes. Do not invent a consuming emptiness check.

### Result family

Follow the accepted P5 Part-II plan: these thin generic forwarding wrappers materialize the canonical eager collection result (currently List for the defined wrappers) unless a newer normative collection spec explicitly overrides a specific family.

If a newer current spec gives a different result family for one source, STOP AND CONSULT before changing that public API.

### Map note

Map direct transform input follows the canonical Map Item `(key: K, value: V)` under P5. Do not resurrect key-only cursor semantics.

### Bytes note

Current comments saying direct `map/filter` are deliberately absent because they are inherited from Iterable are stale under the accepted P5 ownership split. Update those comments in the same patch.

### Forbidden approaches

- no six copied traversal algorithms;
- no eager methods on Iterator;
- no conflicting eager methods on generic Iterable;
- no consuming generic `isEmpty`;
- no source cursor resurrection.

### Test changes required

Cover:

```text
P6-CV-25
P6-CV-26
P6-CV-27
```

Strengthen eager tests with callback timing:

```text
let calls = 0
let result = source.map { calls += 1; ... }
# calls already complete before result consumption
```

Contrast with:

```text
let pipeline = source.iter.map { calls += 1; ... }
# calls == 0 until consumption
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
```

Run bytes/range-specific exact tests only if those files changed in behavior beyond wrappers.

### Tests explicitly deferred

Sequence lifecycle migration until T9.

### Acceptance

Direct collection transforms are eager and `.iter` transforms remain lazy; concrete `isEmpty` is restored without adding a consuming trait default.

### Local STOP / CONSULT triggers

- current normative API contradicts the P5 wrapper result family;
- selector ambiguity reappears between Iterator/Iterable/concrete sources.

### Checkpoint update

Record the final API ownership split if not already accurately represented.

---

## T8 — Ratify and harden the `Tuple.zipped` Dynamic boundary

### Purpose

Close `P5-AUD-009` without inventing heterogeneous tuple conformance theory.

### Preconditions

- T1 Dynamic Item semantics complete;
- T6 typed ordinary pairwise `Iterator.zip` complete.

### Consumes

- current `TupleZipIterator`;
- ordinary Dynamic message-send semantics;
- strict zip length law.

### Produces

- explicit documented boundary;
- state-machine tests proving strict behavior;
- isolation test proving this boundary does not establish canonical iterability.

### Files and symbols

**Read/Modify as needed:**

```text
phalcom-core/core/universe/src/collections/tuple.ph
docs/specs/iterator.md
```

**Tests:**

Add/strengthen focused Tuple zipped tests.

### Required implementation shape

The existing general structure may remain if it already satisfies:

1. `TupleZipView` is reusable;
2. each `TupleZipIterator` acquisition builds lane iterators once;
3. each attempted output advances each lane once, left-to-right;
4. all lane results are classified only after required advances;
5. all Some => tuple payload;
6. all None => sticky None;
7. mixed => `ZipLengthMismatchError`.

Dynamic calls:

```phalcom
tuple.at(i).iterator
storedIterator.next
```

are allowed here only because each heterogeneous element is a `Dynamic` value crossing an explicit API boundary.

Add a normative paragraph making clear:

```text
Tuple.zipped does not establish or infer Iterable conformance.
Its heterogeneous lane acquisition is dynamic.
The canonical rules for for/* remain evidence-based.
```

### Isolation discriminator

Create a structural lookalike type:

```text
iterator / next methods
no Iterable/Iterator conformance
```

Prove:

- `for x in lookalike` is rejected;
- `Receiver.collect(*lookalike)` is rejected;
- if the value is deliberately placed in a Tuple and consumed by explicit `Tuple.zipped`, ordinary Dynamic dispatch follows the explicit boundary policy.

This test makes the distinction intentional rather than accidental.

### Forbidden approaches

- no per-element tuple proof engine in this plan;
- no claim that dynamic lane success equals canonical conformance;
- no weakening G1 lookalike rejection;
- no shortest-zip behavior.

### Test changes required

`P6-CV-30`.

### Tests to run now

Run the exact Tuple zipped tests plus:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
```

only if the relevant fixture lives there.

### Tests explicitly deferred

Broad collection corpus unless zipped fixtures are registered there.

### Acceptance

Tuple.zipped's dynamicity is explicit, isolated, strict, and documented.

### Local STOP / CONSULT triggers

- a current normative spec requires static per-element proof;
- current Dynamic send semantics cannot support the required error behavior without new runtime mechanics.

### Checkpoint update

Record the explicit dynamic boundary as a deliberate architecture decision.

---

## G2 — Typed iterator/public surface gate

### Purpose

Prove the repaired iterator type contracts, eager/lazy split, lifecycle, and Tuple.zipped policy.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
```

Expected:

```text
typed adapter positive/negative discriminators pass
direct map remains eager
iter.map remains lazy
chain mismatch is rejected
stored pipeline remains one-shot
Tuple.zipped strict behavior passes
```

Do not broaden after PASS.

---

## T9 — Migrate the active affected corpus and preserve coverage

### Purpose

Close `P5-AUD-007`: make current acceptance tests test the current architecture rather than the retired cursor model.

### Preconditions

- G2 passed;
- eager wrappers exist;
- stateful lifecycle stable.

### Consumes

- current fixture registration;
- old cursor/repeatability fixtures;
- canonical iterator APIs.

### Produces

- no active PASS fixture that calls retired cursor selectors;
- one-shot pipeline tests;
- preserved behavioral coverage.

### Files and symbols

**Modify at minimum:**

```text
phalcom-core/tests/fixtures/language/collections/range_cursor_protocol_direct.ph
phalcom-core/tests/fixtures/language/collections/map_wren_cursor_roundtrip.ph
phalcom-core/tests/fixtures/language/sequence/sequence_takeview_repeatable.ph
phalcom-core/tests/fixtures/language/MANIFEST.md
phalcom-core/tests/core/language/corpus.rs comments
```

**Retain/strengthen:**

```text
phalcom-core/tests/fixtures/language/iterator/iterator_direct_map_still_eager.ph
```

### Required implementation shape

#### T9.1 Range cursor fixture

Replace direct calls to:

```text
iterate
iteratorValue
```

with stateful `RangeIterator` behavior.

Preserve coverage of:

- inclusive/exclusive traversal;
- empty range;
- sticky exhaustion;
- fresh independent iterator acquisition;
- forward order.

Rename the fixture if its current name would falsely document cursor semantics.

#### T9.2 Map cursor fixture

Replace cursor roundtrip with canonical Map iterator/entry tests.

Preserve:

- insertion/encounter order as current Map semantics define it;
- direct item shape `(key:..., value:...)`;
- fresh iterator acquisition;
- sticky exhaustion;
- mutation behavior already accepted by current iterator model.

Do not preserve key-only cursor Item.

Rename the fixture.

#### T9.3 Stored pipeline repeatability

Replace:

```text
same view traversed twice -> same output
```

with two tests or one combined fixture proving:

```text
same stored pipeline -> second traversal observes exhausted/current state
fresh source.iter.take(...) -> fresh traversal
```

#### T9.4 Direct eager map

Keep the direct eager fixture and make sure callback timing is asserted.

Do not rewrite it to `.iter.map`.

#### T9.5 Manifest/comments

Update `MANIFEST.md` and corpus comments so they no longer claim cursor protocol or repeatable view law.

### Forbidden approaches

- no deleting coverage solely because it fails;
- no hiding cursor fixtures by excluding them from registration while retaining false PASS status;
- no compatibility aliases for `iterate`/`iteratorValue`;
- no changing P5 one-shot semantics to preserve old golden output.

### Test changes required

Cover:

```text
P6-CV-28
P6-CV-29
P6-CV-31
```

### Tests to run now

Run affected lanes only:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
```

Add negative lanes only if T9 edits their fixtures.

### Tests explicitly deferred

bytes/strings/list unless changed by T7 and not already covered at G2.

### Acceptance

No active PASS fixture encodes retired cursor semantics or repeatable stored-pipeline semantics.

### Local STOP / CONSULT triggers

- a fixture is backed by a newer normative source that conflicts with P5;
- migration exposes a behavior not decided by current specs.

### Checkpoint update

Record corpus migration at gate level, not each fixture edit.

---

## G3 — Active corpus migration gate

Run the current repository targets:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence_negative

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bytes
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
```

Expected:

```text
all directly affected current corpus lanes pass
no PASS fixture uses retired cursor selectors
direct eager collection transform fixture passes
one-shot/fresh lifecycle fixtures pass
```

If a negative-lane phase changes because invalid `*` is now semantic rather than runtime, update only the exact affected phase expectation with evidence. Do not broadly loosen phase checking.

---

## T10 — Synchronize current-facing docs/comments and classify residue

### Purpose

Close `P5-AUD-008` and provide an auditable repository-wide retirement census.

### Preconditions

- G3 passed;
- semantic/API decisions stable.

### Consumes

- final implemented model;
- current specs/guides/comments.

### Produces

- current-facing documentation aligned with canonical iteration;
- classified historical residue.

### Files and symbols

**Modify at minimum if still stale:**

```text
docs/spec/current/iteration.md
docs/guide/collections.md
docs/guide/control-flow.md
docs/spec/collections/06-rest-spread-and-pack-operators.md

phalcom-ast/src/ast.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/dispatch.rs

phalcom-core/tests/core/language/corpus.rs
phalcom-core/tests/fixtures/language/MANIFEST.md
relevant agent/language overlay files
```

**Normative clarification:**

```text
docs/specs/iterator.md    # Tuple.zipped explicit Dynamic boundary only
```

### Required implementation shape

Current-facing text must agree on:

```text
Iterator = stateful traversal object
Iterable = semantic conformance
Collection = runtime/class taxonomy only
for = canonical semantic Iterable evidence
* = Unit/Tuple direct lane, otherwise canonical Iterable evidence
stored lazy pipeline = one-shot/stateful
fresh source.iter = fresh traversal
direct collection transforms = eager where specified
Map item = labeled key/value tuple
Tuple item = Dynamic
Tuple.zipped = explicit heterogeneous Dynamic boundary
```

Remove current claims that:

```text
iterate/iteratorValue is the canonical protocol
selector coincidence establishes iterability
generic * compiles as ordinary cursor sends
iteration allocates no iterator object
stored lazy views are repeatable
```

Historical ADRs/old implementation plans may retain the old model when clearly historical.

### Production terminology

Update stale variable/comment names such as cursor-protocol descriptions around `JumpIfNone` where they now misdescribe generic Option-based iterator exhaustion.

Do not rename unrelated user-visible APIs just for terminology.

### Census

Run a classified search over at least:

```text
phalcom-core/src
phalcom-core/core/universe/src
phalcom-semantic/src
phalcom-native-meta/src
phalcom-modules/src
phalcom-ast/src
phalcom-lsp/src
examples
phalcom-core/tests/fixtures
docs/specs
docs/spec/current
docs/guide
docs/spec/collections
.agents
```

Search for:

```text
iteratorValue
iterate(
cursor protocol
has_iteration_protocol
compile_for_lanes
SourceIterator
IndexedIterable
ZippedIterable
ordinary compiler sends of "iterator"
ordinary compiler sends of "next"
legacy runtime Iterable class identity
repeatable lazy-view wording
key-only Map iteration wording
```

Classify each remaining hit:

```text
CURRENT-MUST-FIX
HISTORICAL-INTENTIONAL
NEGATIVE-TEST
INTERNAL-UNRELATED
FALSE-POSITIVE
```

### `P5-AUD-010` inline support disposition

During this census/review:

- verify the current inline implicit-Universe support rule still works for every canonical built-in source used by P6 tests;
- do not automatically launch a module/prelude refactor;
- if it remains a path-category helper but all correctness tests pass, record `P5-AUD-010` as a nonblocking maintainability follow-up;
- if a P6-required source fails solely because the curated support set omits its conformance owner, replace the path-pattern rule with one canonical module/semantic-owned implicit-support closure rather than adding another ad hoc path.

This makes the audit risk explicit without expanding scope gratuitously.

### `P5-AUD-011` broader erasure disposition

If T4 binding-dependent tests pass independently of missing-binding fallback:

- record the global `env.get(param).unwrap_or(Dynamic)` behavior as a separate architectural hardening follow-up;
- do not claim P6 changed global erasure semantics.

If T4 cannot prove correctness because the fallback masks absence, P6 is blocked until a narrow fail-fast invariant is introduced or consultation resolves the boundary.

### Tests to run now

Documentation/comments do not justify rerunning runtime suites.

Run only:

```sh
git diff --check
```

and the classified census commands.

### Tests explicitly deferred

All broad tests until G4.

### Acceptance

Current-facing docs/comments teach the stateful model and all remaining legacy hits are classified.

### Local STOP / CONSULT triggers

- two current normative specs materially conflict;
- fixing current docs would require a language decision not already made;
- inline-support correctness requires a new module/prelude architecture.

### Checkpoint update

Record classified census and any explicitly deferred architectural risk.

---

## T11 — Rebuild truthful P5/P6 completion records

### Purpose

Close `P5-AUD-012` and operationalize `P5-AUD-013`.

### Preconditions

- G4 (below) passed;
- final code revision committed or otherwise uniquely identifiable.

### Consumes

- actual final diff;
- actual executed commands/results;
- P5 coverage rows;
- P6 coverage rows;
- consultation/deferred-failure ledgers.

### Produces

```text
corrected P5 completion accounting
LANG005.C6.P6-walkthrough.md
LANG005.C6.P6-handoff.md
truthful LANG005.C6-CHECKPOINT.md
```

### Files and symbols

**Modify:**

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-handoff.md
```

**Create:**

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P6-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P6-handoff.md
```

### Required implementation shape

#### Correct P5 records

Do not erase the original history.

Amend P5 walkthrough/handoff so they state:

- original completion claim was invalidated by the 2026-09-18 audit;
- P6 performed corrective closure;
- exact corrected final SHA;
- final G4/G5/G6 disposition after P6;
- original claims that were corrected.

#### P5 coverage matrix

Every existing `P5-CV-*` row receives:

```text
PASS
FAIL
DEFERRED
SUPERSEDED
NOT APPLICABLE
```

plus evidence.

No row disappears.

#### P6 audit closure table

Include:

| Audit | Final disposition | Task/gate | Evidence |
|---|---|---|---|
| P5-AUD-001 | FIXED | T3/G1 | ... |
| ... | ... | ... | ... |

Expected non-code dispositions:

- `P5-AUD-010`: fixed if correctness required support-closure refactor, otherwise explicitly deferred maintainability risk with evidence.
- `P5-AUD-011`: mitigated/proven nonblocking for P5; broader erasure hardening deferred if unchanged.
- `P5-AUD-013`: closed procedurally by focused P6 commit structure and evidence.

#### Exact revision

No `local-head`.

Use the actual committed SHA being certified.

#### Actual commands only

Do not claim any test was run unless it actually completed.

### Forbidden approaches

- no fabricated pass counts;
- no zero-census claim without classification;
- no treating historical hits as current defects;
- no claiming workspace/release certification if not run;
- no combining source-written, tests-added, focused-tested, and checkpoint-accepted states.

### Tests to run now

No new runtime test solely for record creation.

Run:

```sh
git diff --check
git status --short
```

and verify recorded SHA/state.

### Acceptance

The checkpoint, corrected P5 records, and P6 records tell one consistent story tied to one exact revision.

### Local STOP / CONSULT triggers

- final evidence does not actually support P5 G4/G5/G6 acceptance;
- an audit blocker remains unresolved;
- plan completion would require relabeling an unrun broad gate as passed.

### Checkpoint update

This is the final plan-completion checkpoint update.

---

# 19. Verification gates

## G4 — Remediation stabilization gate

### Purpose

Prove all directly affected semantic/compiler/API/corpus behavior before records are finalized.

Run:

```sh
# Semantic authority
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental

# Core canonical iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance

# Positional expansion
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion

# Current affected corpus
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bytes
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings

# Affected crates/tooling
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-native-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp
```

If `phalcom-lsp` package naming has mechanically changed, use the live package name.

Also run:

```sh
git diff --check
```

Do not run `cargo test --workspace` automatically.

### Expected

All commands required by this gate pass or a failure is classified according to §17. Any A/B failure blocks completion.

---

## G5 — Canonical-retirement census gate

### Purpose

Prove that no current production/acceptance surface relies on cursor iteration or structural implicit iterability.

Required assertions:

```text
no compile_for_lanes
no has_iteration_protocol
no runtime Iterable class identity
no production iterate/iteratorValue protocol
no compiler fallback send "iterator"/"next" for positional expansion
no active PASS cursor fixture
structural lookalike rejected by for/*
Tuple.zipped is the only documented heterogeneous Dynamic lane boundary in this scope
```

Run the classified census from T10.

Historical hits are allowed only when classified.

Any unclassified current-facing hit blocks G5.

---

## G6 — P5 corrective re-certification gate

### Purpose

Decide whether the original P5 G4/G5/G6 can now be truthfully accepted.

Required evidence:

1. G1 PASS;
2. G2 PASS;
3. G3 PASS;
4. G4 PASS;
5. G5 PASS;
6. all blocking/high audit findings fixed;
7. `P5-CV-*` matrix complete;
8. exact final SHA available;
9. current docs/corpus aligned;
10. residual medium risks explicitly classified;
11. no audit claim relies on `local-head`.

If all true:

```text
P5 corrective disposition: ACCEPTED AFTER P6 REMEDIATION
P5 G4: PASS
P5 G5: PASS
P5 G6: PASS
```

This does **not** imply:

```text
LANG005.C6 release certified
workspace release gates passed
```

unless those stronger gates were separately executed.

---

## 20. Final focused acceptance

At plan completion, the P6 walkthrough must contain a table like:

| Coverage ID | Test / evidence | Result |
|---|---|---|
| P6-CV-01 | Tuple Dynamic canonical lane semantic/core test | PASS |
| P6-CV-02 | RefRange typed Item test | PASS |
| P6-CV-03 | exact Iterable spread | PASS |
| ... | ... | ... |
| P6-CV-35 | final diff/census hygiene | PASS |

It must also contain the original P5 coverage matrix.

### Audit disposition target

| Finding | Target P6 disposition |
|---|---|
| `P5-AUD-001` | FIXED |
| `P5-AUD-002` | FIXED |
| `P5-AUD-003` | FIXED |
| `P5-AUD-004` | FIXED |
| `P5-AUD-005` | FIXED |
| `P5-AUD-006` | FIXED |
| `P5-AUD-007` | FIXED |
| `P5-AUD-008` | FIXED |
| `P5-AUD-009` | FIXED / RATIFIED EXPLICIT DYNAMIC BOUNDARY |
| `P5-AUD-010` | FIXED if correctness requires; otherwise DEFERRED-NONBLOCKING with evidence |
| `P5-AUD-011` | P5 PATH PROVEN / BROADER ERASURE RISK DEFERRED unless narrow fix required |
| `P5-AUD-012` | FIXED |
| `P5-AUD-013` | CLOSED BY P6 PROCESS/COMMIT DISCIPLINE |

Do not mark P6 complete if any blocker/high finding remains unresolved.

---

## 21. Performance/resource evidence

P6 is not a performance plan.

Do not add benchmark ceremony.

One resource property is correctness-relevant:

```text
single-lane for natural exhaustion must not grow the VM evaluation stack linearly
```

The stress regression in `P6-CV-13` is sufficient unless the repository already exposes a cheap exact stack-depth assertion.

No throughput benchmark is required.

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

T0 must:

- mark P6 active;
- record starting revision;
- record audit reopen;
- map audit findings to tasks.

### 22.2 During plan

Update only for:

- T1 Dynamic-Item architecture established;
- T3 canonical-only expansion established;
- T4 shared evidence-root rule established;
- G1;
- T6/T7 final API ownership if materially durable;
- T8 Tuple.zipped policy;
- G2;
- G3;
- consultation/amendment;
- deferred failure/risk;
- G4/G5/G6.

Do not turn the checkpoint into a chronological test log.

### 22.3 At plan completion

- mark P6 final status/completion/verification truthfully;
- record exact final revision;
- record P5 corrective disposition;
- record final invariants;
- record deferred risks;
- record consultations;
- set next action, normally `LANG005.C6.P7` broader C6 closure/certification.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P6-walkthrough.md
```

It must contain:

1. final result;
2. starting and final revision;
3. audit findings closed;
4. architecture implemented;
5. Dynamic Item decision and implementation;
6. positional expansion authority repair;
7. runtime evidence-root repair;
8. stack-discipline repair;
9. adapter typing changes;
10. eager collection surface changes;
11. Tuple.zipped policy;
12. corpus migration;
13. docs/census result;
14. important files changed;
15. deviations;
16. consultations;
17. tests added;
18. tests actually run;
19. tests deliberately deferred;
20. failure classifications;
21. `P6-CV-*` closure table;
22. full `P5-CV-*` disposition table;
23. P5 G4/G5/G6 final disposition;
24. residual risks;
25. final verification classification.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P6-handoff.md
```

It must contain:

- current exact revision;
- corrected P5 status;
- inherited stable semantic/lowering interfaces;
- final invariants;
- explicit `Tuple.zipped` boundary;
- remaining `P5-AUD-010/011` follow-ups if deferred;
- next objective;
- must-read files;
- deferred failures;
- first recommended commands;
- explicit things not to redesign;
- whether successor is P7 and what broader C6 certification remains.

Keep it compact enough to avoid rediscovery.

---

## 25. Completion truth table

Do not conflate these states:

| State | Meaning |
|---|---|
| source written | edits exist |
| tests added | regressions exist |
| targeted tests passed | T0/T1-level evidence |
| affected suites passed | stabilization evidence |
| P5 audit findings closed | remediation objective met |
| P5 G4/G5/G6 accepted | P5 completion contract restored |
| P6 complete | this corrective plan finished |
| C6 checkpoint accepted | broader C6 objective met |
| release certified | release/workspace gates actually passed |

Expected P6 completion may legitimately be:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

while `LANG005.C6` itself remains:

```yaml
status: IN_PROGRESS
completion: PARTIAL
```

until broader C6 closure.

---

## 26. Suggested corrective commit series

Keep commits focused. One reasonable sequence:

```text
1. docs(lang005): reopen P5 and start P6 remediation
2. fix(iteration): carry Dynamic item knowledge through canonical lanes
3. fix(sheetcalc): type RefRange iteration as Ref
4. fix(expansion): require canonical iterable evidence for positional spread
5. fix(iteration): root expansion trait environments
6. fix(iteration): balance single-lane for exhaustion stack
7. fix(iterators): restore typed adapter contracts
8. fix(collections): restore eager transform and isEmpty surface
9. docs+test(tuple): ratify explicit dynamic Tuple.zipped boundary
10. test(iteration): migrate cursor-era active corpus
11. docs(iteration): synchronize current iteration authorities and comments
12. docs(lang005): recertify P5 and complete P6 records
```

Mechanical coupling may justify merging adjacent commits, but do not recreate a single broad closure commit containing unrelated churn.

---

## 27. Residual-risk policy

### 27.1 Inline Universe semantic support (`P5-AUD-010`)

Default P6 disposition:

```text
verify correctness under all P6 built-in tests
do not broaden to module/prelude redesign if correct
record maintainability follow-up if still path-pattern based
```

Escalate/fix inside P6 only if it causes a P6 correctness failure.

### 27.2 Global missing-binding Dynamic fallback (`P5-AUD-011`)

Default P6 disposition:

```text
prove required expansion environments with discriminating tests
do not claim global erasure semantics were fixed
record broader hardening follow-up
```

If that fallback prevents trustworthy proof of P6 correctness, introduce the narrowest fail-fast invariant or consult.

### 27.3 Broader C6 certification

Not part of P6. Successor plan owns:

- remaining non-P5 C6 obligations;
- broader workspace/release gates;
- final checkpoint acceptance.

---

## 28. Plan self-review

Before delivering/starting this plan, verify:

### Architecture

- [x] one semantic owner for canonical iterability;
- [x] compiler/VM do not reconstruct proof;
- [x] Dynamic Item and iterator proof are separated;
- [x] expansion fallback semantics are explicit;
- [x] runtime evidence-root owner is explicit;
- [x] Tuple.zipped boundary is explicit;
- [x] adapter type contracts are explicit.

### Luna executability

- [x] FIXED / FLEXIBLE / VERIFY-FIRST areas are separated;
- [x] each task names read/modify/test surfaces;
- [x] forbidden shortcuts are explicit;
- [x] task-local escalation conditions exist;
- [x] debugging loops are bounded;
- [x] unrelated failures can be deferred.

### Testing

- [x] architecture-discriminating negative tests exist;
- [x] exact/assumed/derived expansion coverage exists;
- [x] stack repetition coverage exists;
- [x] adapter typing covers positive and negative cases;
- [x] eager/lazy timing is covered;
- [x] lifecycle is covered;
- [x] broad workspace tests are explicitly deferred;
- [x] final affected-corpus gate is explicit.

### Documentation lifecycle

- [x] checkpoint start/update/completion obligations exist;
- [x] corrected P5 records are required;
- [x] P6 walkthrough/handoff are required;
- [x] exact revision accounting is required;
- [x] original P5 coverage rows remain part of completion truth.

### Audit closure

- [x] every `P5-AUD-001..013` has a target disposition;
- [x] no blocker/high finding is silently deferred;
- [x] medium architectural risks are explicitly bounded;
- [x] P5 G4/G5/G6 cannot be restored without evidence.

---

# 29. Final implementation directive

The implementer should treat this as a **corrective closure**, not a redesign.

Preserve the already-correct P5 backbone:

```text
semantic conformance
    ↓
canonical iteration product
    ↓
lowering requirement recipe
    ↓
compiler emission
    ↓
VM execution
```

Repair the seams in this order:

```text
truthful checkpoint state
    ↓
Dynamic Item-capable semantic publication
    ↓
RefRange precision
    ↓
fail-closed positional expansion
    ↓
runtime evidence roots
    ↓
single-lane stack discipline
    ↓
typed adapters
    ↓
eager reusable-collection surface
    ↓
explicit Tuple.zipped Dynamic boundary
    ↓
active corpus migration
    ↓
current docs/census
    ↓
P5/P6 re-certification records
```

The final success criterion is not “the focused tests are green.”

It is:

> **Every implicit iteration path that the language accepts is backed by canonical semantic evidence; Dynamic yielded items are represented truthfully without becoming fake static proof; iterator contracts match the values they yield; reusable collection APIs preserve the intended eager/lazy split; active tests and current documentation describe the same stateful model; and P5's restored completion claim is supported by exact, revision-bound evidence.**
