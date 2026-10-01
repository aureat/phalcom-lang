---
id: LANG005.C6.P5
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan
planning_pass: 1-of-3
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG005.C6.P4
follows: LANG005.C6.P4
supersedes: null
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: 06e37b6830cd9114d03f2a794d0b9a973f8cdc82
planning_revision_subject: "feat(lang005): complete P4 iterator iterable law staging"
requirements_analysis: LANG005.C6.P5-requirements-analysis.md
executor_profile: GPT-5.6-Luna-High-or-Extra-High
---

# LANG005.C6.P5 — Production Stateful Iteration, Evidence-Driven `for`, Iterator Toolbox, and Cursor Retirement

> **For agentic workers:** execute this plan task-by-task. The preferred execution style is one coherent task per implementation session or fresh implementer, with review at every named gate. Do not run broad test suites unless this plan explicitly names them.
>
> **Required workflow:** read `AGENTS.md`, `docs/workflow/implementation-record-lifecycle-convention.md`, `docs/workflow/shared-consultation-escalation-protocol.md`, the C6 checkpoint/guidance, this plan, and the P4 walkthrough/handoff before editing. Follow repository BUILD / STABILIZE / CERTIFY doctrine exactly.
>
> **Required sub-skill when available:** use the repository plan-execution / subagent-development workflow. Do not invent a second execution process.

**Goal:** Replace Phalcom's production cursor protocol and runtime `Iterable` class hierarchy with the canonical P4 `Iterator` / `Iterable` trait model, make every `for` form consume semantic evidence and stateful `next -> Option<Item>` iterators, provide a production-ready default iterator surface, migrate built-in sources/views, and remove every live cursor dependency without moving proof into the VM.

**Architecture:** Formal iteration meaning remains semantic-layer authority. `CallableAnalysis` publishes one canonical resolution for each `for` lane: the proven `Iterable` conformance, normalized `Item` / `Iterator` types, and semantically selected `iterator` / `next` trait invocations. `ModuleLoweringSemantics` projects those decisions into executable specs using the existing selected/requirement trait-invocation machinery. The compiler emits a direct jump loop that acquires one iterator per lane and calls `next` exactly once per attempted step; the VM only executes already-selected calls. Built-in reusable sources construct ordinary stateful iterator instances, adapters retain ordinary traced object fields, and P4's derived `Iterator => Iterable` law gives custom iterators `for` participation plus the `Iterable` terminal surface from only `Item + next`.

**Tech Stack:** Rust workspace; `phalcom-semantic`; `phalcom-core`; source-authored Phalcom Universe; canonical `GenericConformanceEvidence`, `TraitDispatchSite`, associated projection normalization, `ModuleLoweringSemantics`, `TraitInvocationSpec`, `TraitRequirementInvocationSpec`, `InvokeTraitSelected`, `InvokeTraitRequirement`, existing ADT `Option` representation / `GetVariantPayload`, compiler boundedness metadata, ordinary traced `InstanceObject` fields.

**Spec / analysis:** `LANG005.C6.P5-requirements-analysis.md` (final pass 3/3); `docs/specs/iterator.md`; `docs/specs/generic-constraints.md`; `docs/specs/objects/traits.md`; `docs/specs/objects/impl.md`; `docs/specs/associated-types.md`; C6 checkpoint/guidance; P4 walkthrough/handoff.

## Global constraints

- P4's source trait identities and derived `Iterator => Iterable` rule remain authoritative; do not recreate them under a new declaration identity.
- Semantic analysis owns conformance proof, associated projection meaning, trait requirement identity, ambiguity, and evidence provenance.
- Compiler/lowering consume semantic products; they do not rediscover `Iterable`, `Iterator`, `Item`, `iterator`, or `next` from selector strings.
- VM/runtime execute finite preselected recipes and do no conformance proof or associated-type normalization.
- An ordinary custom iterator implements `type Item` + `next`; broad behavior is derived/defaulted.
- `Iterable` means reusable traversal factory; `Iterator` means stateful one-shot traversal; `Iterator.iterator` is `self` and exhaustion is sticky.
- Direct collection `map` / `filter` / `flatMap` stay eager; `source.iter.map(...)` is lazy. Do not invent trait precedence/specialization to resolve selector overlap.
- Strict zip never truncates. Mixed `Some` / `None` on an attempted step raises `ZipLengthMismatchError`.
- Parallel `for` setup is two-phase: evaluate all source expressions left-to-right, then acquire all iterators left-to-right, then traverse.
- `Map<K,V>::Item` is the labeled tuple `(key: K, value: V)`.
- `Range` remains the existing forward-integer subset; no step/descending/Progression design enters P5.
- `String` itself remains non-Iterable; `bytes` and `codePoints` views become canonical Iterable sources.
- No new native iterator heap arm, no per-adapter native primitive, no runtime cursor compatibility protocol at plan completion.
- P5 is focused implementation/stabilization, not C6 release certification; workspace-wide test/clippy gates are not BUILD requirements.

---

# 0. Executor contract

This plan is written for GPT-5.6 Luna High/Extra High operating under constrained architectural authority.

The implementer:

- may adapt helper names, local decomposition, imports, exact test fixture names, and mechanically drifted private signatures to live HEAD;
- must preserve every `FIXED` decision and every `INV-*` invariant below;
- must not redesign trait identity, semantic ownership, runtime evidence, Option representation, class identity, GC rooting, Range semantics, Map item shape, or eager-vs-lazy API policy;
- must re-ground the local checkout before edits and preserve unrelated modified/staged/untracked files;
- must use sparse discriminating tests in BUILD MODE rather than test after every edit;
- must classify unrelated failures `A/B/C/D` and continue on D immediately;
- must perform one serious corrective attempt for the same nontrivial semantic failure, then STOP AND CONSULT if it persists;
- must not use a cursor bridge as the final implementation even if it makes tests pass;
- must keep `LANG005.C6-CHECKPOINT.md` current only at durable interfaces/gates/consultations/deferred failures;
- must produce `LANG005.C6.P5-walkthrough.md` and `LANG005.C6.P5-handoff.md` at completion.

Testing is evidence gathering. A narrow PASS answers the named question and is not permission to broaden reflexively.

---

# 1. Goal

At P5 completion, ordinary `for` has one explanation:

```text
source expression
    ↓ evaluated once
semantic proof: Source |= Iterable
    ↓
normalize <Source as Iterable>::Item
normalize <Source as Iterable>::Iterator
    ↓
semantically select Iterable.iterator
semantically select Iterator.next
    ↓
ModuleLoweringSemantics executable lane spec
    ↓
compiler stores source
compiler invokes selected iterator once
    ↓
loop attempt: invoke selected next once
    ├─ None       -> normal exhaustion
    └─ Some(item) -> GetVariantPayload(0) -> pattern/index bind -> body
```

For N lanes, the preheader is:

```text
source0
source1
...
sourceN
iterator0
iterator1
...
iteratorN
traversal
```

and each attempted step obtains all N `next` results before deciding all-None / all-Some / mismatch.

---

# 2. Checkpoint acceptance objective

`LANG005.C6.P5` is the production migration plan between P4's completed generic law and P6's remaining closure/stress/certification work.

P5 is complete when:

1. production `for` no longer uses `iterate(_)` / `iteratorValue(_)` semantics or compiler selector sends;
2. built-in iterable sources and views have canonical `Iterable` conformances and create stateful iterator objects;
3. traversal objects/adapters have canonical `Iterator` conformances and sticky `next -> Option<Item>` behavior;
4. the required Iterator/Iterable default toolbox is live, with eager concrete wrappers where semantics require them;
5. Map, Range, String views, strict zip, boundedness, and argument expansion are integrated with the canonical model;
6. the runtime `Iterable` superclass and its `CoreClasses.iterable_class` identity are gone;
7. positional expansion no longer probes cursor selectors;
8. current docs/tooling/comments describe the trait model;
9. a final production census finds no live cursor protocol outside explicitly historical/superseded documents;
10. focused semantic/runtime/incremental/GC/disassembly gates pass or remaining failures are truthfully classified.

P5 is not workspace release certification. P6 owns broader C6 stress/certification unless the user explicitly promotes P5 to CERTIFY MODE.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 06e37b6830cd9114d03f2a794d0b9a973f8cdc82
subject: feat(lang005): complete P4 iterator iterable law staging
predecessor: LANG005.C6.P4
```

Verified planning facts:

1. P4 is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`; its canonical semantic/runtime evidence gates are green. The blocked positive legacy iteration corpus reproduces an existing `Option.unwrapOr` / generic call-entry failure.
2. `phalcom-core/core/universe/src/collections/iterator_laws.ph` owns the canonical trait identities and source derived rule.
3. `CallableAnalysis` currently publishes expression, binding, associated, family, match, flow, dependency, and fingerprint products but has no iteration-lane product.
4. `Statement::For` in `phalcom-semantic/src/checker/statement.rs` still calls `resolve_iteration_element_application`, which probes cursor-era callable shapes rather than proving canonical `Iterable`.
5. `ModuleLoweringSemantics` already owns `TraitInvocationSpec` and `TraitRequirementInvocationSpec` maps keyed by `LoweringSite`; compiler helpers already emit `InvokeTraitSelected` and `InvokeTraitRequirement` from those specs.
6. `compile_for` and `compile_for_lanes` in `phalcom-core/src/compiler/lib/loops.rs` still send `iterate` / `iteratorValue`; the generalized preheader currently interleaves source evaluation and cursor acquisition.
7. `compile_for_lanes` already has the jump skeleton for patterns, ordinals, captures, strict multi-lane termination, and break/continue; P5 should preserve that control-flow architecture while changing traversal state.
8. strict-zip compiler lowering currently raises `ArgumentError`; `ZipLengthMismatchError` is normative but has no production declaration at the planning revision.
9. production `Iterable<T>` / `Iterator` / `SourceIterator` / lazy cursor adapters live in `collections/iterable.ph`; current built-ins inherit the runtime `Iterable` class.
10. `CoreClasses.iterable_class` currently parents `List`, `Map`, `Set`, `Tuple`, `Range`, and `Bytes`; bootstrap, primitive registration, and invariants reference it.
11. current Map direct cursor iteration yields keys; `MapEntriesView` creates nominal `Entry` objects.
12. current Range cursor traversal is forward integer +1 only, supports upper-unbounded sources, and rejects missing lower / descending traversal.
13. current iterator corpus includes laziness/repeatability tests for cursor-era pipelines; tests expecting repeatability of the same pipeline object must change to one-shot semantics.
14. `Object::Instance` GC tracing already visits ordinary instance slots, so upstream iterators/callbacks/current-inner objects can be ordinary fields.
15. `VM::has_iteration_protocol` still tests cursor selectors for generic positional expansion; cursor deletion is unsafe until that dependency is migrated.
16. compiler boundedness already recognizes `.iter`, `map`, `filter`, `flatMap`, `skip`, `take`, `takeWhile` through conservative syntax/source facts and must be updated rather than replaced.
17. `examples/sheetcalc/src/grid/ref-range.ph` is a live repository subclass/use site of the legacy cursor root.
18. `docs/guide/collections.md`, `../../../__unstructured__/spec/current/iteration.md`, some current comments/LSP text, and historical accepted cursor decisions still contain cursor descriptions; authoritative `docs/specs/iterator.md` is the target language rule.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

# 4. Required reads before implementation

Read in this order before T0 completes:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/luna-patch-grade-plan-schema.md

docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-handoff.md

LANG005.C6.P5-requirements-analysis.md

docs/specs/README.md
docs/specs/iterator.md
docs/specs/generic-constraints.md
docs/specs/objects/traits.md
docs/specs/objects/impl.md
docs/specs/associated-types.md

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md

phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/derived_conformance.rs
phalcom-semantic/src/db/fingerprint.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/boundedness.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/heap/trace.rs

phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/iterable.ph
phalcom-core/core/universe/src/collections/list.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/collections/set.ph
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/range.ph
phalcom-core/core/universe/src/collections/bytes.ph
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/core/universe/src/errors/package.ph
phalcom-core/core/universe/src/errors/argument.ph

phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-native-meta/src/universe.rs

phalcom-core/tests/core/language/derived_conformance.rs
phalcom-core/tests/core/language/corpus.rs
phalcom-core/tests/fixtures/language/iteration/
phalcom-core/tests/fixtures/language/iterator/
phalcom-core/tests/fixtures/language/sequence/
phalcom-semantic/tests/semantic/capabilities/flow_loops.rs
phalcom-semantic/tests/semantic/incremental/derived_conformance.rs
```

Do not broadly re-audit unrelated compiler/type-system subsystems unless one of these paths disproves a plan premise.

---

# 5. Normative authority

Authority order:

```text
1. explicit user-ratified decisions / accepted checkpoint amendments
2. current docs/specs normative language rules
3. LANG005.C6-CHECKPOINT.md durable invariants
4. completed P4 walkthrough/handoff
5. landed P1-P4 semantic/runtime architecture
6. this P5 plan
7. older implementation plans / current cursor code
8. historical cursor ADRs/guides/tests where superseded
```

## 5.1 Normative behavior used by P5

```text
Iterator:
    type Item
    next -> Option<Item>
    iterator -> self
    single-pass
    sticky exhaustion

Iterable:
    type Item
    type Iterator
    Iterator |= Iterator
    Iterator.Item == Item
    iterator -> fresh independent traversal for reusable sources

Derived law:
    I |= Iterator  =>  I |= Iterable
    Iterable.Iterator == I
    Iterable.Item == Iterator.Item

for:
    nominal/evidence-based participation
    source expression once
    all sources left-to-right before any iterator acquisition
    iterator acquisition once/lane left-to-right
    next once/lane/attempt
    direct same-function loop control
    pattern mismatch remains a language error
    strict multi-lane zip

Map:
    direct Item = (key: K, value: V)

Range:
    forward integer +1 subset only

Runtime:
    semantic proof is not runtime lookup
```

## 5.2 Current implementation versus future proposal

Cursor selectors, runtime `Iterable` inheritance, `SourceIterator`, cursor-based adapters, cursor-era repeatability tests, and `VM::has_iteration_protocol` are implementation debt to remove. They are not normative counter-authority.

Methods deliberately deferred by the final requirements analysis (`rev`, `cycle`, `collect`, ordering families, `try_*`, chunks/windows, step-by, etc.) remain non-goals.

---

# 6. Takeover state

| Concept | Current symbol/path | Owner | P5 invariant |
|---|---|---|---|
| canonical Iterator trait | `collections/iterator_laws.ph::Iterator` | Universe + semantic trait system | retain declaration identity |
| canonical Iterable trait | `collections/iterator_laws.ph::Iterable` | Universe + semantic trait system | retain declaration identity |
| Iterator=>Iterable theorem | guarded `impl<I> Iterable for I where I |= Iterator` | generic proof engine | no compiler/VM special case |
| generic evidence | `GenericConformanceEvidence` | semantic | exact/assumed/derived remain distinct |
| trait call evidence | `TraitDispatchSite` | semantic expression analysis | compiler consumes selection |
| callable semantic product | `CallableAnalysis` | semantic | add lane product here |
| semantic fingerprint | `callable_body_product_fingerprint` | semantic DB | lane result must affect product |
| lowering boundary | `ModuleLoweringSemantics` | core semantic projection | project, do not re-prove |
| selected trait execution | `TraitInvocationSpec` / `InvokeTraitSelected` | core compiler/VM | reuse |
| requirement execution | `TraitRequirementInvocationSpec` / `InvokeTraitRequirement` | core compiler/VM | reuse |
| loop control skeleton | `compiler/lib/loops.rs` | compiler | preserve jump/fiber/capture semantics |
| Option payload extraction | `GetVariantPayload(slot)` after proven variant | compiler/VM ADT path | use for `Some` payload, not `unwrapOr` |
| runtime Iterable class | `CoreClasses.iterable_class` + `collections/iterable.ph` | legacy runtime | retire late |
| cursor expansion probe | `VM::has_iteration_protocol` | legacy VM expansion path | retire before cursor deletion |
| boundedness | `compiler/boundedness.rs` | compiler static metadata | update narrowly |
| GC roots | ordinary `Object::Instance` slots | heap tracer | no iterator-specific tracing arm |

Do not re-prove P4 laws. P5 starts from them.

---

# 7. Architecture

## 7.1 Authority flow

```text
AST `for` lane source expression
        ↓
semantic expression analysis
        ↓
prove Source |= canonical Iterable
normalize Iterable::Item / Iterable::Iterator
select Iterable.iterator / Iterator.next through existing trait evidence
        ↓
ForIterationLaneResolution in CallableAnalysis
        ↓
callable product fingerprint + semantic dependency set
        ↓
semantic-to-core projection
        ↓
ForIterationLaneLoweringSpec
    iterator invocation: selected or requirement form
    next invocation: selected or requirement form
        ↓
compiler direct-jump loop
        ↓
existing InvokeTraitSelected / InvokeTraitRequirement bytecodes
        ↓
VM execution only
```

## 7.2 Canonical semantic product shape

Use the live repository equivalents of the following shape; exact private names are mechanically flexible, but the information boundary is FIXED:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForIterationLaneResolution {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator_dispatch: TraitDispatchSite,
    pub next_dispatch: TraitDispatchSite,
}

pub type ForIterationLaneResolutionIndex =
    BTreeMap<ExpressionId, ForIterationLaneResolution>;
```

Only a semantically successful lane receives an executable resolution. Invalid/blocked/dynamic cases retain their ordinary typed status/diagnostics and must not publish a fake lowering selection.

`TraitDispatchSite` (or the repository-native equivalent already used for ordinary trait calls) must retain exact/assumed/derived evidence provenance. Do not collapse it to a selector string or method name.

## 7.3 Canonical lowering product shape

Lowering needs an explicit representation for an implicit trait call that has no source `MethodCallExpr`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IterationInvocationSpec {
    Selected(TraitInvocationSpec),
    Requirement(TraitRequirementInvocationSpec),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForIterationLaneLoweringSpec {
    pub item_type: TypeId,       // or existing backend type recipe if needed
    pub iterator_type: TypeId,
    pub iterator: IterationInvocationSpec,
    pub next: IterationInvocationSpec,
}
```

Store the lane specs in `ModuleLoweringSemantics` keyed by the lane source `LoweringSite` / range using a dedicated `LoweringSiteKind::ForIterationLane` or an equally unambiguous repository-native key. Do not overload the ordinary source-call map by fabricating nonexistent AST calls.

Refactor existing compiler trait-call emission so both explicit calls and implicit loop calls share the same code that prepares method handles/environment recipes and emits `InvokeTraitSelected` / `InvokeTraitRequirement`.

## 7.4 Loop lowering shape

Single lane:

```text
evaluate source -> source_slot
emit selected Iterable.iterator -> iterator_slot

loop:
    iterator_slot.next -> option_slot
    if option_slot is None -> exit
    item = GetVariantPayload(0) from option_slot
    bind pattern/index
    body
continue_target:
    close captured loop bindings
    jump loop
exit:
```

Do not call `Option.unwrapOr`, `isSome`, cursor APIs, size, or collection-specific indexing in generic `for` lowering.

Generalized N-lane loop:

```text
phase A: evaluate source[0..N] left-to-right into source slots
phase B: call iterator[0..N] left-to-right into iterator slots

attempt:
    call next once on every iterator -> option[0..N]
    if all None: exit
    if mixed None/Some: raise ZipLengthMismatchError
    if all Some:
        extract payload 0 from every Some
        pattern-bind all lanes
        assign shared ordinal to every `at` binding
        body
        close captured pattern/index bindings
        ordinal += 1
        jump attempt
```

Mismatch detection may occur after live lanes advanced on that failed attempt; that is normative strict-zip behavior.

## 7.5 Source/runtime module architecture

Preserve the declaration identities in `iterator_laws.ph`. Extend those traits with defaults rather than declaring replacement traits.

Recommended source decomposition, adaptable mechanically if Universe dependency rules require it:

```text
collections/iterator_laws.ph
    canonical Iterator / Iterable declarations
    trait default method bodies
    P4 derived rule unchanged in meaning

collections/iterator_adapters.ph        # new, ordinary source classes
    MapIterator / FilterIterator / FilterMapIterator / ...
    generic traversal adapter state only

representation source files
    ListIterator / SetIterator / TupleIterator / BytesIterator
    Map / view iterators
    RangeIterator
    String byte/codepoint iterator classes where visibility is cleanest

collections/iterator_impls.ph           # new if module topology permits
    explicit built-in/source/adapter trait conformances
```

The reason for a central conformance module is transition safety: production still has a class named `Iterable` until late retirement, while the canonical trait is also named `Iterable`. Keep source-class inheritance and trait identity from colliding during migration. If live module/import rules make a central conformance module mechanically awkward, colocate impls using an existing alias/qualified-import mechanism. STOP AND CONSULT only if the live language cannot refer to the canonical trait without changing trait identity or public semantics.

## 7.6 Default API ownership

`Iterator` lazy defaults:

```text
map
filter
filterMap
flatMap
flatten
chain
skip
skipWhile
take
takeWhile
mapWhile
inspect
zip              # strict
```

`Iterable` non-conflicting defaults:

```text
iter
indexed
each
each(indexed:)
contains / includes
count
count(where:)
find(where:)
index(where:)
any(where:)
all(where:)
none(where:)
nth
last
fold(initial:using:)
reduce(using:)
join / join(_)
group(by:)
partition(where:)
toList
toSet
toMap
toMap(merging:)
```

Concrete reusable collections retain eager direct transforms through thin inherent methods:

```text
map
map(indexed:)
filter
flatMap
```

Their bodies should acquire one fresh canonical iterator, delegate to the lazy/default algorithm, and materialize. Do not copy traversal loops into each collection.

---

# 8. Ownership boundaries

## 8.1 Owns

P5 owns:

- canonical `for` lane proof/projection/lowering;
- built-in source and view `Iterable` conformances;
- built-in/adaptor stateful `Iterator` conformances;
- source-authored iterator defaults required by final requirements;
- Map labeled-entry iteration;
- Range forward iteration migration;
- String byte/codepoint view integration;
- strict zip error surface;
- boundedness catalog updates for the new surface;
- positional-expansion removal of cursor-selector dependency;
- cursor selector/class/runtime-superclass/public-name retirement;
- iteration-specific incremental/GC/disassembly verification;
- current iteration docs/tooling/comment cutover.

## 8.2 Does not own

P5 does not own:

- general trait-solver redesign;
- trait specialization/precedence/supertraits;
- generic inference redesign;
- new runtime conformance search;
- new GC representation;
- new Value/DispatchKey optimization program;
- collection-builder / `collect` abstraction;
- ordering/sorting redesign;
- double-ended iterators;
- Range step/descending/Progression;
- generic optimizer/fusion certification;
- broad argument-pack architecture changes beyond removing iteration's cursor dependency.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| source conforms to Iterable | semantic generic proof | `for`, editor products | compiler/VM method-shape check |
| Item/Iterator associated types | semantic projection normalization | loop typing/lowering | compiler guessing generic args |
| iterator/next callable selection | semantic trait dispatch evidence | semantic lowering/compiler | selector-name lookup in loop compiler |
| lane executable recipe | `ModuleLoweringSemantics` | compiler | VM semantic lookup |
| traversal position/exhaustion | iterator object fields | adapters/next | cursor token external protocol |
| Map entry shape | normative Map/Iterator conformance | `for`, `entries`, terminals | `Entry` pair surrogate |
| Range progression | Range/RangeIterator source | `next` | compiler range special case |
| boundedness classification | compiler boundedness metadata | exhaustor diagnostics | runtime behavior oracle |
| GC roots | ordinary object fields + heap tracer | runtime | raw untraced pointer fields |

---

# 9. Global invariants

`INV-01` — P4's canonical `Iterator` and `Iterable` declaration identities remain stable.

`INV-02` — Conformance proof and associated projection normalization occur only in semantic analysis.

`INV-03` — The compiler receives preselected `iterator` / `next` invocation evidence; no `for` path performs trait discovery by selector string.

`INV-04` — The VM performs no conformance proof, associated normalization, or cursor-shape proof.

`INV-05` — Each lane source expression is evaluated exactly once per loop execution, left-to-right.

`INV-06` — All lane source expressions finish before any lane iterator is acquired; iterator acquisition is then left-to-right exactly once/lane.

`INV-07` — Each iterator is advanced exactly once per attempted loop step by the loop itself; adapter-internal extra reads are local adapter semantics.

`INV-08` — `None` from `next` means clean exhaustion only; ordinary errors propagate unchanged.

`INV-09` — Exhaustion is sticky for every iterator/adaptor.

`INV-10` — `Some(item)` payload extraction preserves `Some(None)` / nested Option values; exhaustion is never tested by payload value.

`INV-11` — break/continue/return/fiber behavior remains the direct-jump same-function architecture; no `each`/closure desugaring.

`INV-12` — captured loop bindings are closed before rebinding/next iteration so per-iteration capture identity remains correct.

`INV-13` — strict zip is all-Some / all-None / error-on-mixed; never shortest-source truncation.

`INV-14` — custom iterators normally require only `Item + next` and derive `Iterable` plus defaults.

`INV-15` — reusable sources create independent iterator objects; iterator/adaptor values are one-shot and aliasable.

`INV-16` — direct collection eager transforms and explicit iterator lazy transforms remain observably distinct.

`INV-17` — Map iteration yields labeled `(key: K, value: V)` tuples in encounter order; `Map.contains(key)` stays key membership.

`INV-18` — Range remains forward +1 with current missing-lower/descending failures and supports upper-unbounded lazy traversal.

`INV-19` — String remains non-Iterable; only explicit byte/codepoint views are iterable.

`INV-20` — ordinary object fields root iterator/adaptor sources, callbacks, inner iterators, and buffered state across GC.

`INV-21` — no production cursor selector/token/class/superclass remains at P5 completion.

`INV-22` — runtime `Iterable` class identity is removed only after all semantic/library/compiler/expansion consumers are migrated.

`INV-23` — boundedness remains conservative: reject only proven unbounded full exhaustion; Unknown stays legal.

`INV-24` — cursor removal from positional expansion does not add VM trait proof or structural `next` probing.

`INV-25` — no per-combinator native primitive explosion; source-authored defaults remain the correctness baseline.

`INV-26` — incremental callable fingerprints change when iteration evidence changes and remain stable for unrelated body edits.

---

# 10. Non-goals

Do not add or redesign:

```text
rev / double-ended traversal
cycle / repeatable iterator cloning
size_hint / exact-size trait
sum / product numeric contracts
min/max/sort ordering policy
collect / FromIterator-like builders
try_* short-circuit abstraction
peekable / scan / chunks / windows / intersperse
step_by / Range step semantics
trait specialization or default-precedence rules
runtime trait objects for iteration
iterator fusion/inlining as correctness requirement
workspace-wide formatting cleanup
historical documentation rewriting beyond clear supersession/context
```

---

# 11. Expected impact map

## 11.1 Expected source areas

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/db/fingerprint.rs
# possibly a focused checker/iteration.rs split if statement.rs becomes unwieldy

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/boundedness.rs
phalcom-core/src/vm/dispatch.rs              # expansion cursor probe retirement only

phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/iterable.ph       # ultimately delete/replace
phalcom-core/core/universe/src/collections/list.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/collections/set.ph
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/range.ph
phalcom-core/core/universe/src/collections/bytes.ph
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/core/universe/src/collections/package.ph
# likely new iterator_adapters.ph / iterator_impls.ph

phalcom-core/core/universe/src/errors/*
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-native-meta/src/universe.rs
phalcom-modules/src/builtin.rs                # only for new Universe modules

examples/sheetcalc/src/grid/ref-range.ph
```

## 11.2 Expected tests

```text
phalcom-semantic/tests/semantic/capabilities/iteration.rs             # new
phalcom-semantic/tests/semantic/incremental/iteration.rs              # new
phalcom-semantic/tests/semantic/capabilities/mod.rs
phalcom-semantic/tests/semantic/incremental/mod.rs

phalcom-core/tests/core/language/iteration_stateful.rs                # new
phalcom-core/tests/core/language/mod.rs
phalcom-core/tests/core/memory/* or focused existing GC module

phalcom-core/tests/fixtures/language/iteration/*
phalcom-core/tests/fixtures/language/iterator/*
phalcom-core/tests/fixtures/language/sequence/*
phalcom-core/tests/fixtures/language/collections/*
phalcom-core/tests/fixtures/language/strings/*
phalcom-core/tests/core/language/corpus.rs
```

Use existing test registration conventions rather than creating parallel binaries.

## 11.3 Expected documentation/state

```text
docs/specs/iterator.md                           # only if public default surface needs explicit normative catalog wording
docs/guide/collections.md
docs/wiki/collections/traversal.md
docs/spec/current/iteration.md                   # mark superseded/update current mirror according to specs migration rules
relevant LSP hover/current comments
LANG005.C6-CHECKPOINT.md
LANG005.C6.P5-walkthrough.md
LANG005.C6.P5-handoff.md
```

## 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically required by these paths. STOP AND CONSULT before entering a materially different subsystem if doing so changes architecture/ownership rather than satisfying a local compile dependency.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

- trait declaration identity or P4 derived law;
- semantic ownership of proof/projections;
- two-phase parallel preheader order;
- direct-jump loop architecture;
- stateful one-shot iterator lifecycle;
- sticky exhaustion;
- Map entry tuple shape;
- Range semantic subset;
- eager collection vs lazy iterator distinction;
- default-method ownership split;
- ordinary traced-field GC model;
- final cursor/runtime-class retirement requirement;
- no-runtime-proof boundary.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- helper/type private names if the semantic product remains equivalent;
- whether iteration checker logic stays in `statement.rs` or moves to focused `checker/iteration.rs`;
- whether adapter classes are one source module or a few responsibility-based modules;
- whether built-in conformance impls are centralized or colocated, provided trait/class name collision is avoided and one canonical trait identity is used;
- exact scratch-local names and private compiler helper decomposition;
- exact test fixture filenames;
- minor source signatures required by current generic syntax when semantically equivalent;
- ordinary import/borrow/Rust cleanup.

## 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| local HEAD contains P4-complete law | git + `iterator_laws.ph` | material drift => STOP |
| P4 baseline positive iteration failure still reproduces | `language-corpus iteration` | record changed baseline; if canonical path implicated, classify before edits |
| canonical trait declaration can be resolved by semantic declaration identity without new global solver branch | linked Universe / resolver / trait tables | local mechanical helper okay; new architecture => STOP |
| `TraitDispatchSite` can represent exact/assumed/derived iterator/next calls | `trait_dispatch.rs`, P4 tests | if new proof kind needed => STOP |
| existing selected/requirement lowering can be reused for implicit calls | `semantic_lowering.rs`, `scope.rs` | refactor helper okay; new runtime ABI => STOP |
| `GetVariantPayload(0)` is valid after `JumpIfNone` on canonical Option Some | ADT lowering/runtime tests | if not, use repository-native proven Some extraction; do not use `unwrapOr` |
| ordinary source class fields are GC traced | `heap/trace.rs` | if false/nonlocal rooting needed => STOP |
| source module topology can load adapter/impl modules without trait-ID duplication | `builtin.rs`, package/source loader | adapt module placement; identity change => STOP |
| `ZipLengthMismatchError` can be ordinary Error subclass | errors package patterns | nonlocal ABI => STOP |
| argument expansion has a semantic/compiler seam capable of static iterable evidence | expansion compiler/lowering paths | if dynamic semantics force runtime proof => STOP |
| final nominal `Entry` usage is only migratable compatibility | final `rg` | newer normative identity requirement => STOP |

---

# 13. Global STOP / CONSULT triggers

STOP editing the affected issue and build a consultation packet if any repository-wide trigger fires, or if:

1. local P5 ownership area materially diverges from the P4-complete architecture;
2. canonical `Iterator`/`Iterable` law differs from P4 handoff;
3. correct loop migration appears to require VM conformance proof or runtime projection normalization;
4. `CallableAnalysis` / `ModuleLoweringSemantics` cannot carry lane selections without changing semantic authority;
5. non-conflicting `Iterable` defaults cannot be exposed through derived evidence without adding a broad trait precedence rule;
6. a required high-value default needs unsupported generic semantics whose implementation is larger than a bounded P5-local extension;
7. adapters require trait-owned storage, native heap arms, or untraced pointers;
8. `ZipLengthMismatchError` requires new runtime error ABI rather than source Error subclass;
9. public-name transition requires module/name-resolution redesign;
10. deleting `iterable_class` reveals nonlocal runtime representation/layout identity dependencies;
11. GC correctness cannot be achieved through ordinary traced instance slots;
12. P4's `Option.unwrapOr` failure also breaks the new `next -> Option<Item>` path for reasons requiring broad generic-call runtime redesign;
13. passing tests would weaken exact enum-case identity, nominal conformance, associated equality, or P4 derived evidence;
14. Range requires a new step/descending semantic decision;
15. newer authority contradicts Map labeled tuple Item;
16. positional expansion cannot remove cursor probing without runtime proof/search or a material change to dynamic `*` semantics;
17. a cursor compatibility bridge is becoming the final production protocol;
18. one task expands into a materially unrelated subsystem.

A triggered consultation cannot be self-waived.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows local progress:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

Examples: imports, module registration, borrow errors, source syntax typo, test filter drift.

## 14.2 Semantic/behavioral failures

Before any serious correction record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic problem. If substantially the same failure remains, STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless relevant code/state changed.

---

# 15. Testing surface analysis

The final requirements matrix `P5-CV-01..115` is normative coverage design. This plan groups execution by task/gate rather than running 115 separate tests.

Primary dimensions selected:

| Dimension | Why applicable | Representative invariants |
|---|---|---|
| canonical success | every built-in/custom traversal must work | INV-01..10 |
| empty/one/many boundaries | iteration and adapters are cardinality-sensitive | INV-07..09 |
| negative nominal behavior | method coincidence must not imply Iterable | INV-02..04 |
| lifecycle/repetition | reusable source vs one-shot iterator | INV-14..16 |
| ordering | source/iterator acquisition and Map encounter order | INV-05/06/17 |
| aliasing | adapters retain same upstream object | INV-15 |
| composition | generic/conditional/cross-module + adapters | INV-02/14 |
| exact identity | enum-case/generic evidence must survive | INV-01/02 |
| incremental equivalence | lane selections are semantic products | INV-26 |
| GC/lifetime | source/callback/inner iterator must stay rooted | INV-20 |
| diagnostics | strict zip, invalid conformance, patterns | INV-13 |
| optimizer/runtime boundary | no proof/hot-path selector lookup | INV-03/04/25 |
| fiber/control flow | direct-jump loop remains suspendable | INV-11/12 |

Coverage-task ownership summary:

```text
T0       baseline classification and P4 takeover
T1       CV-22..28, 54..57, 61..63, 96, 100, 109, 110
T2       CV-57..59, 97, 109
T3       CV-01..05, 29..32, 70..74, 104
T4       CV-01..05, 15..21, 22..28, 30..32, 54, 57..59
T5       CV-06..21, 38, 52, 67..69, 98, 99
T6       CV-33..35, 53, 87, 88, 95, 106..108
T7       CV-14, 36..39, 89, 90, 103, 105
T8       CV-40..44, 91, 92
T9       CV-45..53, 70..86, 93..95, 100..104, 108
T10      CV-45..53, 75..86, 94, 101..103
T11      CV-91, 111
T12      CV-112 and expansion-focused regression surface
T13      CV-64, 65, 90, 115
T14      CV-58, 65, 66, 113
T15      CV-57..66, 93, 94, 109..115 + final census
```

Every CV is either directly owned above or exercised as a dependent case at a named gate. The full matrix is reproduced in §27 for exact scenario traceability.

---

# 16. Verification execution budget

## 16.1 Modes

### BUILD MODE

Default for T0–T14. Run only tests that discriminate the current task. Do not broaden after a PASS.

### STABILIZE MODE

Enter only at G1–G6 and T15. Run directly affected suites and classify failures.

### CERTIFY MODE

Not required by this plan unless the user/checkpoint explicitly promotes it. P6 owns broad C6 certification.

## 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad affected crate/package
T5 — workspace/release
```

## 16.3 Mandatory during BUILD

- exact new semantic iteration tests when T1 changes semantic products;
- exact lowering/compiler test when T2/T4/T5 changes executable loop generation;
- one or a few source-language fixtures for the source class/adapters changed in T3/T6–T10;
- exact boundedness test in T11;
- exact expansion regression in T12;
- grep/census commands in retirement tasks.

## 16.4 Mandatory during STABILIZE

At named gates only:

```text
semantic capabilities::iteration
semantic incremental::iteration
core language::iteration_stateful
language-corpus iteration
language-corpus iteration_disasm
language-corpus iteration_negative
language-corpus iterator
language-corpus iterator_negative
language-corpus sequence
language-corpus sequence_negative
language-corpus collections/list/bytes/strings as relevant
affected-crate cargo check
final git diff --check
```

## 16.5 Only if evidence demands

- full `phalcom-semantic --test semantic`;
- full `phalcom-core --test core`;
- full `language-corpus`;
- broader module/REPL/LSP suites if touched behavior actually crosses those surfaces.

## 16.6 Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check as a broad repository gate
full language corpus after every task
unrelated LSP/CLI/concurrency suites
```

Reason: P5 is large and cross-layer; repeated broad gates would turn BUILD into stabilization churn. Run formatting only on touched Rust files / repository-normal mechanisms if available; final `git diff --check` is mandatory.

---

# 17. Baseline / unrelated failure policy

Classify unexpected failures:

```text
A — definitely caused by P5
B — probably caused by P5
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record in checkpoint/walkthrough and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks P5 acceptance.
- Do not weaken/skip assertions to make broad gates green.

Specific baseline `BL-P5-01` at planning time:

```text
language-corpus::iteration
existing failure: Option.unwrapOr / generic call-entry type recipe could not be instantiated
P4 classification: reproduces at clean predecessor; not caused by P4
```

T0 must reproduce or reclassify `BL-P5-01`. The canonical loop implementation must not intentionally route through `unwrapOr`. If the failure disappears because cursor code is removed, record it as retired-baseline collateral, not as a hidden P5 proof. If it reappears on ordinary canonical `next`, diagnose once; broad runtime generic-call repair triggers consultation.

---

# 18. Tasks

## T0 — Re-ground P5 checkout, baseline, and migration census

### Purpose

Establish a trustworthy local starting state before any production edit, reproduce/classify the P4 blocked lane, and record the exact live cursor/public-name dependency census that destructive tasks will later close.

### Preconditions

- P4 handoff read.
- No P5 production edits yet.

### Consumes

- P4 completed revision/contracts.
- final P5 requirements analysis.

### Produces

- local revision/worktree record;
- `BL-P5-01` reproduction/reclassification;
- checked lists of cursor selectors, runtime `iterable_class`, legacy subclasses, current docs/tooling hits;
- checkpoint `active_plan: LANG005.C6.P5`.

### Files and symbols

**Read:** checkpoint/guidance, P4 records, all paths in §4.

**Modify:** `docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md` only for active plan/baseline facts.

**Tests:** existing P4/legacy gates only.

### Required implementation shape

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -10 --oneline

rg -n 'iterate\(|iteratorValue\(|SourceIterator|class Iterator|class Iterable|iterable_class|has_iteration_protocol' \
  phalcom-core phalcom-semantic phalcom-modules phalcom-native-meta examples docs/guide docs/wiki docs/spec/current

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
```

Confirm each Cargo filter selects tests. Record exact baseline trace and revision.

### Forbidden approaches

- no code edits to "see whether tests improve";
- no broad workspace test;
- no cleanup/reset of unrelated work;
- no treating historical docs as production blockers.

### Test changes required

None.

### Tests to run now

Exactly the three commands above, plus `-- --list` if a filter selects zero tests.

### Tests explicitly deferred

All P5 new suites; workspace/broad cargo gates.

### Acceptance

- live revision and worktree state known;
- P4 law tests green or material drift classified;
- `BL-P5-01` recorded as reproducing/changed;
- cursor/runtime-class census stored in working notes/checkpoint summary;
- no production edit made before this evidence exists.

### Local STOP / CONSULT triggers

- canonical P4 traits/rule absent or materially changed;
- P4 semantic tests fail for a new non-baseline reason;
- checkout has overlapping user edits in planned source files that cannot be preserved safely.

### Checkpoint update

Mark P5 active and record baseline revision + BL-P5-01 classification. Do not log every grep hit.

---

## T1 — Publish canonical `for` lane semantic resolution

### Purpose

Replace cursor-call probing as the semantic authority for `Statement::For` with one first-class lane product proving canonical `Iterable`, normalizing associated types, and retaining executable trait selections.

### Preconditions

- T0 accepted.
- P4 law and generic proof engine green.

### Consumes

- `GenericConformanceEvidence` / proof APIs;
- canonical `Iterable` / `Iterator` declaration identities;
- associated projection normalizer;
- `TraitDispatchSite` and requirement identities;
- existing loop fixed-point/pattern flow machinery.

### Produces

- `ForIterationLaneResolution` + index on `CallableAnalysis`;
- canonical resolver used by `Statement::For`;
- no semantic `for` dependence on `iteratorValue` / `iterate`;
- fingerprint/dependency coverage for lane evidence.

### Files and symbols

**Read:**
- `phalcom-semantic/src/checker/{analysis,context,body,statement}.rs`
- `phalcom-semantic/src/{generic_evidence,trait_dispatch,traits,types/projection}.rs`
- `phalcom-semantic/src/db/fingerprint.rs`

**Modify:**
- `checker/analysis.rs`
- `checker/context.rs` and/or `checker/body.rs` publication plumbing
- `checker/statement.rs` or new focused `checker/iteration.rs`
- `db/fingerprint.rs`
- focused dependency-recording helper if required

**Create:**
- `phalcom-semantic/tests/semantic/capabilities/iteration.rs`
- `phalcom-semantic/tests/semantic/incremental/iteration.rs`
- module registrations.

### Required implementation shape

1. Resolve the canonical Universe trait declarations by stable semantic identity. A module/name lookup used once to locate the built-in declaration is acceptable; a trait-name branch inside proof logic is forbidden.
2. For each lane source expression after ordinary expression analysis:
   - require/prove `Source |= Iterable`;
   - normalize `<Source as Iterable>::Item` to canonical `TypeId`;
   - normalize `<Source as Iterable>::Iterator` to canonical `TypeId`;
   - establish that iterator type conforms to canonical `Iterator` through the trait contract / existing proof engine;
   - select `Iterable.iterator` on the source with exact/assumed/derived evidence;
   - select `Iterator.next` for the normalized iterator type with exact/assumed/derived evidence;
   - retain both selections as semantic products.
3. Bind the `for` pattern from `item_type`, preserving the existing loop fixed-point, causal invalidity, explanation, break/continue join, exact-case identity, and flow graph.
4. Publish successful lane products on `CallableAnalysis`; do not invent products for invalid/blocked/dynamic lanes.
5. Add lane product fields to `callable_body_product_fingerprint`; hash semantic identity/evidence selections in the same stable style as existing expression products, never pointer addresses/source-order accidents.
6. Record the semantic dependencies already consumed by trait header/surface/conformance/associated normalization. Do not add a parallel iteration dependency graph.
7. Remove or demote `resolve_iteration_element_application` from production `for`; keep a compatibility helper only if another active consumer still needs it, with an explicit retirement owner.

Representative product:

```rust
pub struct ForIterationLaneResolution {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator_dispatch: TraitDispatchSite,
    pub next_dispatch: TraitDispatchSite,
}
```

### Forbidden approaches

- no first-generic-argument iteration heuristic;
- no structural selector coincidence as conformance;
- no hard-coded `List`/`Range` cases in semantic `for`;
- no fabricated `ImplId` for generic assumptions;
- no compiler-only `Item` inference;
- no P4 theorem special case.

### Test changes required

Add semantic tests covering at minimum:

```text
CV-22 generic T |= Iterable loop publishes Item
CV-23 generic T |= Iterator loop succeeds through derived Iterable
CV-24 conditional Iterable conformance
CV-25 cross-module conformance
CV-26 associated Item projection normalization
CV-27/28 exact enum-case identity through loop item
CV-54 matching method names without conformance rejected
CV-55 invalid Iterable.Iterator contract rejected
CV-56 Item equality mismatch rejected
CV-96 one canonical lane product/source expression
CV-100 generic Iterator receives Iterable.iter without ambiguity (can be staged RED until defaults land)
CV-109 selected evidence change changes fingerprint
CV-110 body-only edit preserves lane semantic product when deps unchanged
```

Keep CV-100 as a staged expectation until T9 if `.iter` is not yet a default; do not fake it in T1.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::iteration
```

### Tests explicitly deferred

Runtime/source-language loop tests until lowering exists.

### Acceptance

- every semantically valid `for` lane has exactly one canonical lane resolution;
- item typing comes from associated projection evidence, not cursor callable result shape;
- nonconforming structural lookalike is rejected;
- lane product participates in fingerprint/dependency behavior;
- current loop flow tests remain semantically intact.

### Local STOP / CONSULT triggers

- canonical built-in trait identity cannot be obtained without adding a second trait registry;
- generic assumed/derived evidence cannot select `iterator` or `next` with existing trait dispatch abstractions;
- associated normalization requires a loop-specific normalizer.

### Checkpoint update

Record the durable new semantic interface (`ForIterationLaneResolution`) and authority boundary after focused tests pass.

---

## T2 — Project lane evidence into executable lowering specs

### Purpose

Carry the T1 lane selections across the semantic-to-core boundary using the existing trait invocation machinery, with no compiler proof reconstruction.

### Preconditions

- T1 semantic product green.

### Consumes

- `ForIterationLaneResolution`;
- `TraitInvocationSpec` / `TraitRequirementInvocationSpec` projection logic;
- existing runtime type/conformance recipe preparation.

### Produces

- `ForIterationLaneLoweringSpec` map in `ModuleLoweringSemantics`;
- shared compiler helpers capable of emitting a provided selected/requirement trait call spec for implicit loop operations.

### Files and symbols

**Modify:**
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/compiler/lib/scope.rs`
- focused executable-semantics/chunk helper only if needed

**Tests:**
- new core semantic-lowering/iteration test under existing core language/compiler test tree.

### Required implementation shape

1. Add `LoweringSiteKind::ForIterationLane` or a precise equivalent.
2. Project each semantic lane's `iterator_dispatch` and `next_dispatch` through the same code paths used for ordinary `TraitDispatchSite::{Evidenced, Requirement...}` products.
3. Represent the result as `IterationInvocationSpec::{Selected, Requirement}` wrapping existing specs. Do not define new runtime evidence recipe kinds merely because the call is implicit.
4. Refactor:

```rust
emit_trait_invoke_if_present(...)
emit_trait_requirement_if_present(...)
```

so their opcode/method-preparation body delegates to helpers that accept a concrete `TraitInvocationSpec` / `TraitRequirementInvocationSpec`. The existing explicit-call path continues to look up by source range; loop lowering can call the spec-taking helper directly.
5. Ensure derived generic evidence still prepares nested runtime type/conformance roots through the existing `modules/compile.rs` machinery. If implicit sites require call-environment roots that are currently discovered only by walking explicit expressions, extend the semantic lowering/root collection from the lane product rather than reconstructing evidence in compiler code.

### Forbidden approaches

- no storing selector strings as executable lane authority;
- no invoking general `emit_operator_send("iterator"/"next")` from canonical loop lowering;
- no new VM trait-search opcode;
- no copying the trait selection algorithm into `phalcom-core`.

### Test changes required

Cover:

```text
CV-57 generic runtime loop evidence forwarded
CV-58 lowering/disasm contains no cursor sends (full proof waits T4)
CV-59 no runtime trait proof/lookup
CV-97 lowering snapshot contains selected/requirement iterator+next specs
CV-109 selection change invalidates lowering product
```

A focused unit/integration test should inspect `ModuleLoweringSemantics` directly for a source loop whose source conformance is exact and another generic/assumed or derived case.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core semantic_lowering -- --nocapture
# If repository filters are different, list and run only the new iteration-lowering tests.
```

### Tests explicitly deferred

`language-corpus::iteration` until T4.

### Acceptance

- compiler-facing semantics contain both implicit calls without trait lookup;
- explicit trait-call lowering still passes focused regressions;
- no new runtime proof mechanism exists.

### Local STOP / CONSULT triggers

- implicit selected calls need a materially new runtime environment lifetime/ABI;
- the only implementation route is compiler-side evidence reconstruction.

### Checkpoint update

Record executable lane lowering contract after focused projection tests pass.

---

## T3 — Establish List as the first canonical stateful source/iterator vertical

### Purpose

Create the smallest complete source/runtime vertical independent of `for`: `List<T> |= Iterable`, fresh `ListIterator<T>`, `ListIterator<T> |= Iterator`, typed `next -> Option<T>`, sticky exhaustion, independent traversals, aliasing, and GC safety.

### Preconditions

- T1/T2 products exist, but `for` need not yet use them.
- transition module can refer to canonical trait identities without redeclaration.

### Consumes

- P4 trait identities/rule;
- native List accessors;
- ordinary source classes/instance fields;
- existing `Some` / `None` surface.

### Produces

- `ListIterator<T>` ordinary class;
- explicit canonical List/iterator conformances;
- one proven `next` runtime vertical before loop compiler cutover.

### Files and symbols

**Modify:**
- `collections/list.ph`
- `collections/iterator_laws.ph` only if defaults/import declarations required, not identity recreation
- `collections/package.ph`
- `phalcom-modules/src/builtin.rs` only if new child modules are created

**Create (recommended):**
- `collections/iterator_impls.ph` for explicit conformances if module topology permits.

**Tests:**
- `phalcom-core/tests/core/language/iteration_stateful.rs`
- optional focused GC case in existing memory module.

### Required implementation shape

Representative source semantics:

```phalcom
class ListIterator<T> {
  @constructor
  new(_ source) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<T> {
    if (_done) { return None }
    if (_index >= _source.size) {
      _done = true
      return None
    }
    let item = _source.at(_index)
    _index = _index + 1
    return Some(item)
  }
}

impl<T> Iterable for List<T> {
  type Item = T
  type Iterator = ListIterator<T>
  iterator { ListIterator.new(self) }
}

impl<T> Iterator for ListIterator<T> {
  type Item = T
}
```

Exact field typing syntax may adapt to current source language. Sticky `_done` is not optional: a later List mutation must not resurrect an iterator once `None` has been returned.

Do not remove the legacy runtime superclass yet. During this vertical it may continue to supply legacy collection methods while canonical conformance exists in parallel.

### Forbidden approaches

- no cursor field exposed publicly;
- no `SourceIterator` wrapper around List;
- no clone/reset/restart behavior on iterator;
- no native ListIterator heap arm;
- no list-copy snapshot solely to iterate.

### Test changes required

Cover:

```text
CV-01..03 empty/one/many
CV-29 two independent List iterators
CV-30 iterator.iterator is same traversal (via P4 derived rule when convenience available)
CV-31 sticky exhaustion
CV-32 GC while List iterator live
CV-70 custom iterator minimal burden can be staged partly here
CV-71 two source iterators independent
CV-72/73 one-shot/reacquire lifecycle when defaults arrive
CV-74 upstream alias state when adapter arrives
CV-104 source.iterator returns concrete iterator, no SourceIterator wrapper
```

At this task, assert direct `next` behavior and conformance before relying on canonical `for`.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful list_iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
```

### Tests explicitly deferred

`language-corpus iteration`, adapter toolbox, other collection families.

### Acceptance

- source `List.iterator` produces fresh distinct stateful objects;
- each iterator yields correct sequence through `next`;
- exhaustion is sticky;
- P4 proves ListIterator=>Iterable without a special case;
- iterator survives GC through ordinary fields.

### Local STOP / CONSULT triggers

- ordinary class fields are not sufficient to root source state;
- built-in conformance cannot be staged without changing canonical trait identity;
- runtime class inheritance must be removed prematurely to make conformance visible.

### Checkpoint update

Record first production stateful source/iterator vertical and GC invariant.

---

## G1 — Semantic/lowering/List vertical gate

Purpose: prove that the canonical law is now usable end-to-end independently of production `for`.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful list_iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
```

Expected: all selected tests pass; no runtime loop migration claimed yet.

Do not broaden after PASS.

---

## T4 — Cut over single-lane `for` to stateful `next -> Option<Item>`

### Purpose

Replace the common single-lane name-binding cursor lowering with the T2 executable lane spec while preserving direct-jump control flow, captures, errors, generic evidence, and fiber suspension.

### Preconditions

- G1 passed.
- List canonical vertical works directly.

### Consumes

- `ForIterationLaneLoweringSpec`;
- compiler spec-taking trait invocation helpers;
- existing loop context / local / close-upvalue machinery;
- existing `JumpIfNone` and `GetVariantPayload(0)`.

### Produces

- canonical single-lane production `for` lowering;
- no `iterate` / `iteratorValue` sends in this path.

### Files and symbols

**Modify:**
- `phalcom-core/src/compiler/lib/loops.rs`
- exact disassembly helpers/tests in language corpus.

**Tests:**
- existing `iteration` fixtures plus new canonical custom/generic cases.

### Required implementation shape

1. Evaluate lane source once and store it.
2. Load the lane lowering spec and emit the preselected `Iterable.iterator` invocation once; store iterator.
3. Loop header emits preselected `Iterator.next` on the iterator and stores/duplicates the resulting Option as needed.
4. `JumpIfNone` exits.
5. On the proven Some path, extract payload slot 0 with `GetVariantPayload(0)` (or the repository's exact proven-variant payload primitive if mechanically named differently).
6. Bind the existing loop variable/pattern value from the payload.
7. Preserve body compilation, same-function break/continue, return, fiber suspendability, and close-before-rebind behavior.
8. `continue` targets the close/bottom of the body then jumps to the next header; there is no separate cursor-step send because `next` itself advances.
9. Do not call `unwrapOr`, `isSome`, collection `size`, or `at` from generic lowering.

### Forbidden approaches

- no fallback to cursor sends on missing lowering spec for semantically valid source code;
- no `coll.each` / closure lowering;
- no ordinary dynamic `next` send selected by method lookup;
- no body-specific List fast path.

### Test changes required

Cover single-lane subsets of:

```text
CV-01..05
CV-15/16 continue/break advancement
CV-17 nested reacquisition
CV-18 return
CV-19 acquisition error
CV-20 next error
CV-21 capture identity
CV-22/23 generic Iterable/Iterator loops
CV-24/25 conditional/cross-module
CV-27/28 exact enum identity
CV-30/31 iterator-self/sticky semantics
CV-54 structural lookalike rejection
CV-57 generic runtime evidence forwarding
CV-58 no cursor sends
CV-59 no runtime proof
```

Update `iteration_disasm` oracle: preserve "no `block_call`/Closure on taken loop path" but retire the cursor-specific "zero allocation" assumption. Stateful iterator construction once per traversal and `Some` values per successful `next` are legal by P5 performance policy.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative
# Run only the new/updated single-lane positive fixture filter if corpus supports per-fixture selection;
# otherwise run `language-corpus iteration` once at the end of T4 and classify BL-P5-01.
```

### Tests explicitly deferred

Parallel lanes/patterns/at mismatch cases to T5.

### Acceptance

- common `for x in list` executes canonical iterator/next path;
- disassembly contains no cursor selector sends;
- control-flow/fiber/capture regressions pass;
- generic evidence path executes through existing trait opcodes;
- no `unwrapOr` extraction is introduced.

### Local STOP / CONSULT triggers

- implicit trait invocation cannot preserve runtime conformance environment across loop iterations without per-item reconstruction;
- `GetVariantPayload` cannot safely extract Some after None guard and no existing equivalent exists;
- fiber suspension requires a new native frame.

### Checkpoint update

Record canonical single-lane `for` cutover and preserved direct-jump invariant.

---

## T5 — Cut over generalized patterns, ordinals, parallel lanes, and strict zip

### Purpose

Migrate `compile_for_lanes` to stateful iterators and make the full `for` surface share T4's canonical authority, including the required two-phase preheader and normative mismatch error.

### Preconditions

- T4 single-lane canonical path green.

### Consumes

- N lane lowering specs;
- current generalized pattern/ordinal/capture jump skeleton.

### Produces

- one canonical generalized loop implementation;
- `ZipLengthMismatchError` source class;
- two-phase evaluation order;
- all-Some/all-None/mixed strict zip semantics.

### Files and symbols

**Modify:**
- `compiler/lib/loops.rs`
- Universe errors package/module registration as needed
- iteration fixtures.

**Create:**
- focused error source file if consistent, e.g. `core/universe/src/errors/iteration.ph` exposing `ZipLengthMismatchError`.

### Required implementation shape

Preheader:

```text
for lane in lanes: compile source -> source_slots[lane]
for lane in lanes: selected iterator(source_slot) -> iterator_slots[lane]
initialize shared ordinal = 0
```

Attempt:

```text
for iterator in lanes:
    selected next(iterator) -> option_slot

none_count / branch structure:
    all None  -> exit
    mixed     -> ZipLengthMismatchError.new(...).raise()
    all Some  -> payload extract each lane
```

Do not short-circuit `next` calls based on an earlier lane's None before all lanes have produced a result for that attempted strict step. The spec permits already-advanced lanes on mismatch and requires inspecting all lanes.

Then reuse existing:

- pattern-match tests / commit bindings;
- shared zero-based `at` ordinal;
- pattern mismatch raise;
- close captured bindings before next attempt;
- break/continue patching.

`ZipLengthMismatchError` should follow existing ordinary Error subclass conventions and require no VM ABI.

### Forbidden approaches

- no size precheck as strict zip authority;
- no shortest-source truncation;
- no interleaving `source0 -> iterator0 -> source1`;
- no per-lane ordinal based on representation cursor;
- no ArgumentError substitute once canonical error is available.

### Test changes required

Cover:

```text
CV-06 source evaluation order
CV-07 equal zip
CV-08 initial mismatch
CV-09 late mismatch
CV-10 all end together
CV-11 tuple pattern
CV-12 pattern mismatch
CV-13 ordinal 0,1,2
CV-14 Map ordinal is encounter ordinal, not key
CV-15/16 continue/break
CV-17 nested loops
CV-18..21 return/errors/captures
CV-38 Map destructuring when T7 lands (stage fixture)
CV-52 zipped strict mismatch when adapter lands
CV-67 all sources before any iterator acquisition
CV-68 labels unsupported remains unchanged
CV-69 ZipLengthMismatchError
CV-98 source2 failure before iterator1
CV-99 iterator1 failure only after all sources ran
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::flow_loops
```

### Tests explicitly deferred

Other collection families/adapters.

### Acceptance

- every current `for` lane shape uses canonical iteration;
- two-phase side-effect tests pass;
- strict mismatch raises `ZipLengthMismatchError`;
- no cursor selector sends remain in loop compiler;
- flow/capture/fiber behavior remains intact.

### Local STOP / CONSULT triggers

- strict zip requires runtime size/collection knowledge;
- all-lane next evaluation conflicts with accepted error ordering in a newer authority;
- source parser unexpectedly supports labeled break/continue and plan assumption is false materially.

### Checkpoint update

Record full production `for` cutover and two-phase/strict-zip invariants.

---

## G2 — Canonical `for` gate

Purpose: prove `for` no longer needs the cursor protocol before broad collection migration.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::flow_loops
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative
```

Then:

```sh
rg -n 'iterate|iteratorValue' phalcom-core/src/compiler/lib/loops.rs
```

Expected: no production cursor send/semantic reference in loop lowering; historical comments removed/updated.

---

## T6 — Migrate Set, Tuple, Bytes, and String traversal views

### Purpose

Expand the canonical stateful model across non-Map core finite sources and the two explicit String traversal views while preserving their element types and current representation semantics.

### Preconditions

- G2 passed.
- List vertical establishes implementation pattern.

### Consumes

- source native accessors;
- canonical traits/conformance module pattern;
- ordinary iterator object fields.

### Produces

```text
Set<T>        -> SetIterator<T>        Item=T
Tuple         -> TupleIterator         Item=Dynamic initially
Bytes         -> BytesIterator         Item=Int
String bytes view      -> stateful byte iterator, integer byte Item
String codePoints view -> stateful codepoint iterator, integer codepoint Item
```

String itself remains non-Iterable.

### Files and symbols

**Modify:**
- `collections/set.ph`
- `collections/tuple.ph`
- `collections/bytes.ph`
- `scalar/string.ph`
- canonical built-in conformance module
- focused corpus fixtures.

### Required implementation shape

For index-backed finite sources use minimal:

```text
source reference
index/ordinal
sticky done flag
```

Read current source size/accessor on each live step unless a normative mutation snapshot rule says otherwise; do not copy full collections merely to iterate.

For String views, retain the view object / source String and use the view's existing size/at decoding contract. Item must be current integer byte/codepoint type; do not invent `Char`.

`Tuple.zipped` may remain a view getter but its implementation must ultimately consume canonical iterators; if its old `ZippedIterable` class is still cursor-based, migrate it in T9 strict zip rather than keep a parallel cursor path.

### Forbidden approaches

- no direct String conformance;
- no character type invention;
- no Set ordering redesign;
- no tuple-homogeneity fiction;
- no cursor fallback in these iterators.

### Test changes required

Cover:

```text
CV-33 Set encounter order
CV-34 Tuple Dynamic traversal
CV-35 Bytes Int octets
CV-53 GC callback/adapter dependent coverage later
CV-87 generic for over String bytes view
CV-88 codePoints Iterable while String non-Iterable
CV-95/108 nested GC dependent later
CV-106 byte Item typing/runtime
CV-107 codepoint Item typing/runtime
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bytes
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful
```

### Tests explicitly deferred

Map, Range, full adapter toolbox.

### Acceptance

- all named sources/views traverse through canonical stateful iterators;
- String itself fails/does not satisfy Iterable;
- no source copies for traversal;
- sticky exhaustion/independent source iterators hold.

### Local STOP / CONSULT triggers

- String view associated Item cannot be stated with current numeric authority without a new semantic decision;
- native internal access visibility forces a new primitive rather than source placement adjustment.

### Checkpoint update

Record expanded built-in iterator inventory only after focused corpus passes.

---

## T7 — Migrate Map to canonical labeled entry tuples and views

### Purpose

Perform Map's semantic migration from key-only direct iteration / nominal Entry view values to one canonical labeled `(key: K, value: V)` Item shape.

### Preconditions

- canonical generalized `for` works;
- tuple/product labeled access is stable.

### Consumes

- native ordered Map key/value accessors;
- labeled tuple representation;
- canonical iterator patterns.

### Produces

```text
Map<K,V> |= Iterable<Item=(key:K,value:V), Iterator=MapIterator<K,V>>
MapIterator<K,V> |= Iterator<Item=(key:K,value:V)>
MapKeysView<K,V> / iterator Item=K
MapValuesView<K,V> / iterator Item=V
MapEntriesView<K,V> / iterator Item=(key:K,value:V)
```

Nominal `Entry` removed if final usage census confirms no independent authority.

### Files and symbols

**Modify:** `collections/map.ph`, conformance module, collection/iterator fixtures.

**Search before deletion:**

```sh
rg -n '\bEntry\b|Entry\.new|\.entries\b|entry\.key|entry\.value' . \
  -g '!target/**' -g '!docs/implementation/**' -g '!docs/decisions/**'
```

### Required implementation shape

`MapIterator.next` stores source + index + done and for each live index constructs the existing labeled tuple/product:

```phalcom
(key: _map._$keyAt(i), value: _map._$valueAt(i))
```

Do not create an iterator-specific pair object. `Map.entries` converges on the same tuple item shape. `toMap` may continue to use `.key` / `.value` because tuple labels preserve those selectors/projections.

Keep `Map.contains` / `includes` inherent key membership exactly as today. Generic entry membership is visible on `map.iter.contains(entry)`.

### Forbidden approaches

- no key-only direct iteration compatibility mode;
- no `Entry` object as canonical Item;
- no Map copy;
- no insertion/equality/hash/mutation redesign.

### Test changes required

Cover:

```text
CV-14 ordinal independent of keys
CV-36 direct labeled tuple
CV-37 generic Map Item typing
CV-38 destructuring
CV-39 keys/values/entries
CV-89 entries agrees direct Item
CV-90 Entry usage migrated/retained by authority
CV-103 Map.contains key vs map.iter.contains entry
CV-105 entries.toMap round-trip via labels
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful map
```

### Tests explicitly deferred

Full sequence/iterator toolbox until T9/T10.

### Acceptance

- direct Map `for` yields labeled tuple entries;
- generic associated Item preserves K/V;
- views have canonical iterator conformances;
- key membership API unchanged;
- `Entry` production identity is deleted or explicitly justified by current authority.

### Local STOP / CONSULT triggers

- newer normative source requires nominal Entry identity;
- labeled tuple construction needs a new runtime representation.

### Checkpoint update

Record Map Item migration because it is externally observable.

---

## T8 — Migrate Range to stateful forward `RangeIterator`

### Purpose

Replace Range's cursor-as-yielded-number protocol with a real stateful iterator while preserving exactly the already-ratified forward integer behavior.

### Preconditions

- canonical loop path works;
- Range current behavior tests identified.

### Consumes

- existing Range bounds accessors and integer-coordinate validation;
- canonical Iterator Option semantics.

### Produces

- `Range |= Iterable<Item=Number, Iterator=RangeIterator>`;
- `RangeIterator |= Iterator<Item=Number>`;
- upper-unbounded traversal compatible with `take` later.

### Files and symbols

**Modify:** `collections/range.ph`, conformance module, range/iteration fixtures.

### Required implementation shape

State can be:

```text
_source or normalized immutable bounds
_started/current candidate
_done
```

Prefer keeping validation in first `next` so acquiring an iterator object does not invent a new error-timing rule absent normative authority. On first `next`:

- missing lower => current catchable iteration failure;
- non-integer lower/upper => current failure;
- lower > bounded upper => current descending-unsupported failure;
- choose lower as first candidate;
- subsequent successful steps add 1;
- bounded exclusive/inclusive check before yielding;
- upper absent => continue indefinitely;
- once None returned, `_done=true` and never restart.

Do not use `Range.size` for iteration. `a..` must remain usable with lazy `take`.

### Forbidden approaches

- no step field;
- no negative/automatic descending;
- no Progression;
- no fake finite size for unbounded ranges;
- no compiler Range special case.

### Test changes required

Cover CV-40..44, CV-91, CV-92.

### Tests to run now

Run the narrow Range/iteration fixture filters discovered in T0 plus:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful range
```

### Tests explicitly deferred

`a...iter.take(n)` until basic adapters in T9.

### Acceptance

- exclusive/inclusive/unbounded cases match current semantics;
- lowerless/descending failures stay catchable;
- no size/cursor dependency;
- sticky exhaustion holds.

### Local STOP / CONSULT triggers

Any pressure to choose step/descending semantics not already ratified.

### Checkpoint update

Record Range semantic preservation and explicit non-feature boundary.

---

## G3 — Built-in source migration gate

Run focused built-in coverage:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bytes
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
```

Expected: built-in sources used by these lanes no longer need cursor traversal for `for`.

---

## T9 — Implement canonical iterator adapter substrate and lazy defaults

### Purpose

Replace cursor-stateless pipeline descriptors with real one-shot stateful iterator adapters and expose the lazy transformation toolbox as `Iterator` defaults while keeping the implementation burden `Item + next`.

### Preconditions

- G3 passed.
- canonical traits and stateful sources work.

### Consumes

- `Iterator` trait declaration in `iterator_laws.ph`;
- ordinary class fields/GC;
- generic trait methods/where constraints/associated projections.

### Produces

Stateful adapters:

```text
MapIteratorAdapter<I,U>          # choose collision-free live names; existing MapIterator currently may mean Map source iterator
FilterIterator<I>
FilterMapIterator<I,U>
FlatMapIterator<I,S>
FlattenIterator<I>
ChainIterator<I,J>
SkipIterator<I>
SkipWhileIterator<I>
TakeIterator<I>
TakeWhileIterator<I>
MapWhileIterator<I,U>
InspectIterator<I>
StrictZipIterator<I,J>
```

and lazy `Iterator` defaults constructing them.

Name collisions are mechanical: if `MapIterator` is used for Map source iteration, call the mapping adapter `MappedIterator` or another consistent source name. Do not overload one class with two meanings.

### Files and symbols

**Modify:** `collections/iterator_laws.ph`.

**Create:** recommended `collections/iterator_adapters.ph` and conformance entries/module registration.

**Delete later:** cursor adapter definitions in `collections/iterable.ph` only after canonical replacements pass.

### Required implementation shape

Every adapter stores the upstream iterator object itself, never a source+cursor pair.

Key laws:

```text
map:
    construction does no callback work
    next = upstream.next.map(mapper) conceptually

filter:
    loop upstream.next until None or predicate true

filterMap:
    loop upstream.next; mapper Some(y)->yield y, None->continue

flatMap:
    retain current inner iterator
    mapper called exactly once per reached outer item
    exhaust current inner before outer.next again

flatten:
    same as flatMap identity where Item |= Iterable

chain:
    phase first/second; switch once

skip:
    consume prefix once, then one upstream per output

skipWhile:
    predicate only while skipping; never again after first retained item

take:
    remaining count; take(0) reads zero upstream items

takeWhile:
    first predicate false sets done permanently

mapWhile:
    first mapper None sets done permanently

inspect:
    callback once per yielded item, return original item

strict zip:
    left.next and right.next once per attempt
    both Some -> labeled/positional pair tuple
    both None -> done
    mixed -> ZipLengthMismatchError and done/error path
```

Every early-stopping adapter needs explicit done state to preserve sticky exhaustion even if an adversarial upstream could later yield again.

Generic constraints must use canonical trait syntax. If an individual expansion-tier method (`flatten`, `chain`, etc.) is blocked by a demonstrated source generic-syntax limitation, STOP only if fixing that limitation would be cross-cutting; otherwise classify the one method for plan amendment rather than weakening the core adapter architecture.

### Forbidden approaches

- no cursor tuples;
- no eager List materialization inside lazy adapters;
- no resetting iterator on second terminal call;
- no hidden move semantics;
- no `fuse` API workaround for non-sticky adapters;
- no native adapter primitives.

### Test changes required

Migrate/update iterator fixtures for one-shot semantics and cover:

```text
CV-45 map laziness
CV-46 filter
CV-47 skip once
CV-48 take no extra read
CV-49 takeWhile sticky
CV-50 flatMap empty/nonempty inners
CV-53 GC callback/upstream
CV-72 same pipeline stays exhausted
CV-73 reacquire restarts
CV-74 alias shares upstream
CV-76 map callback only on consumption
CV-77 filterMap
CV-78 flatten
CV-79 chain
CV-80 skipWhile
CV-81 mapWhile
CV-82 inspect
CV-85 strict zip
CV-86 no fuse/restart
CV-93 no VM/native proof floor
CV-95/108 nested flatMap/flatten GC
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful adapters
```

### Tests explicitly deferred

Terminal/eager forwarding surface to T10; broad collection corpus.

### Acceptance

- pipeline objects are real stateful Iterators;
- all named lazy adapter laws pass;
- same pipeline does not restart;
- upstream aliases share state;
- GC tests prove outer/callback/current-inner retention;
- no cursor storage remains in canonical adapters.

### Local STOP / CONSULT triggers

- source generic constraints needed for adapter types cannot be expressed without general type-system redesign;
- callback/inner state needs special GC rooting;
- ordinary default dispatch on Iterator cannot construct source adapter classes without module/identity redesign.

### Checkpoint update

Record canonical adapter state/lifecycle and one-shot policy after iterator corpus passes.

---

## T10 — Implement Iterable terminal defaults and eager collection forwarding surface

### Purpose

Replace the generic algorithms currently inherited from runtime `Iterable` with trait defaults, then preserve accepted direct eager collection transforms through thin inherent forwarding so runtime superclass deletion does not erase the production API.

### Preconditions

- T9 adapters green.

### Consumes

- `self.iterator` / `.iter` canonical convenience;
- T9 lazy adapters;
- existing collection constructors and Map/Result error surface.

### Produces

`Iterable` defaults:

```text
iter
indexed
each
each(indexed:)
contains/includes
count/count(where:)
find/index
any/all/none
nth/last
fold/reduce
join/join(_)
group/partition
toList/toSet/toMap/toMap(merging:)
```

and concrete eager wrappers for existing direct `map`, `map(indexed:)`, `filter`, `flatMap` APIs where current collection/source semantics require eager Lists.

### Files and symbols

**Modify:**
- `iterator_laws.ph`
- `iterator_adapters.ph` (`IndexedIterator`)
- concrete collection files that currently rely on inherited eager transform selectors
- `iterable.ph` only to remove obsolete algorithms after equivalent defaults/wrappers are live; final deletion waits T14.

### Required implementation shape

A terminal default begins from exactly one `self.iterator` acquisition:

```phalcom
count {
  let iterator = self.iterator
  let count = 0
  while true {
    let step = iterator.next
    if (step == None) { return count }
    let item = <Some payload extraction expressed in source Option API>
    count = count + 1
  }
}
```

Source-language defaults may use `for x in self` once canonical `for` is live; that is acceptable because `for` now bottoms out in semantic Iterable evidence and creates one iterator. Prefer whichever form avoids recursive selector ambiguity and is clearest.

`iter { self.iterator }` lives on `Iterable`. An Iterator gets it through P4 derived conformance and `Iterator.iterator { self }`; do not also add overlapping `Iterator.iter` unless required mechanically and proven unambiguous.

`indexed` constructs `IndexedIterator.new(self.iterator)`.

`nth(n)` consumes exactly n+1 items when available and returns Option; reject/handle invalid n according to the current count/index argument policy used by skip/take—do not invent silent negative semantics.

`last` consumes traversal and returns None for empty / Some(last) otherwise.

`toMap` continues duplicate rejection using labeled entry `.key` / `.value`; `toMap(merging:)` preserves merge behavior.

Concrete eager wrappers conceptually:

```phalcom
List.map(_ f) { self.iterator.map(f).toList }
List.filter(_ p) { self.iterator.filter(p).toList }
List.flatMap(_ f) { self.iterator.flatMap(f).toList }
```

Use one semantic algorithm, not copied loops. Apply wrappers to the actual reusable collection families whose accepted API currently exposes those selectors; do not force them onto Map if Map's direct transform semantics differ or are absent.

### Forbidden approaches

- no eager `map/filter/flatMap` defaults on `Iterable` that collide with lazy `Iterator` defaults;
- no duplicate traversal loop per collection;
- no destructive `isEmpty` default on Iterator;
- no generic `collect` abstraction;
- no restart of exhausted Iterator terminals.

### Test changes required

Cover:

```text
CV-51 indexed ordinal
CV-70 custom Iterator Item+next gains defaults+for
CV-71 source .iter independent
CV-72/73 terminal one-shot/reacquire
CV-75 direct collection map eager
CV-83 nth
CV-84 last
CV-94 defaults use evidence; eager wrappers do not duplicate algorithms
CV-100 generic Iterator can .iter via derived Iterable
CV-101 arbitrary custom Iterable uses source.iter.map(...).toList
CV-102 List.map eager vs List.iter.map lazy
CV-103 Map membership distinction
CV-104 no SourceIterator allocation
CV-105 entries.toMap round trip
```

Migrate cursor-era repeatability expected outputs: a stored pipeline consumed twice stays exhausted; recreating from source yields a fresh traversal.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
```

### Tests explicitly deferred

Final cursor deletion/runtime superclass census.

### Acceptance

- runtime superclass is no longer semantically necessary to supply generic algorithms;
- custom iterator author experience is `Item + next` => adapters + terminals + for;
- direct eager collection transform behavior remains accepted/current;
- no selector ambiguity is introduced.

### Local STOP / CONSULT triggers

- trait-default member resolution through derived Iterable cannot expose non-overlapping defaults without general dispatch redesign;
- eager wrappers require duplicating generic algorithms due missing callable semantics rather than simple forwarding.

### Checkpoint update

Record production default surface and replacement of legacy superclass algorithm ownership.

---

## G4 — Iterator surface gate

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful
```

Expected: one-shot semantics reflected in fixtures; no cursor-era restart expectation remains.

---

## T11 — Update boundedness for canonical `.iterator` / `.iter` and new lazy stages

### Purpose

Keep existing conservative exhaustor diagnostics correct after selector/cursor migration without moving boundedness into the semantic trait solver.

### Preconditions

- canonical surface names stable from T9/T10.

### Consumes

- existing `Boundedness`, `IterationMode`, `SourceFacts`, selector catalog.

### Produces

- canonical lazy-pipeline recognition for `.iterator` and `.iter`;
- conservative propagation for new adapters;
- no dependence on deleted cursor selectors.

### Files and symbols

**Modify:** `phalcom-core/src/compiler/boundedness.rs` and its focused tests.

### Required implementation shape

Preserve:

```text
Bounded / Unbounded / Unknown
Unknown is legal
reject only proven Unbounded full exhaustion
```

Update source recognition:

```text
source.iterator / source.iter -> LazyPipeline(source.boundedness)
```

Safe propagation minimum:

```text
map/filter/filterMap/inspect/skip/indexed  -> preserve receiver boundedness
skipWhile                                 -> preserve boundedness if receiver Bounded; Unbounded may remain Unbounded/Unknown per current conservative rule, choose only what is sound
 take                                     -> Bounded
 takeWhile/mapWhile                       -> Bounded if source Bounded, otherwise Unknown
 flatMap/flatten                          -> Unbounded when proof is sound; otherwise Unknown
 chain/zip                                -> Unknown unless current syntax facts prove enough about both inputs
```

Do not make selector presence a semantic Iterable proof; this module is only termination metadata for already-accepted code.

### Forbidden approaches

- no new language conformance checker in boundedness;
- no aggressive bounded claims for chain/flatten/zip;
- no Range fake size.

### Test changes required

CV-91 and CV-111, plus existing E.3 cases.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core boundedness
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections_literals_negative
```

### Tests explicitly deferred

No broad core suite.

### Acceptance

- `0...iter.take(n).toList` is accepted/terminates;
- proven unbounded full exhaustors remain diagnosed;
- Unknown stays legal;
- new adapter selectors do not create false boundedness certainty.

### Local STOP / CONSULT triggers

Any requirement to move formal conformance/projection proof into this compiler heuristic.

### Checkpoint update

Only if a durable propagation rule is added beyond obvious selector catalog replacement.

---

## T12 — Remove positional-expansion cursor probing through the expansion owner

### Purpose

Eliminate the last known VM structural cursor dependency before cursor selectors are deleted, while preserving tuple/list fast paths and avoiding runtime trait proof.

### Preconditions

- canonical Iterable semantics/lowering live;
- argument-expansion current architecture re-read.

### Consumes

- `PackTryExpandTuplePositionals` / `ListTryExpandTuplePositionals` fast lane;
- compiler generic expansion fallback;
- semantic evidence/lowering products where statically available.

### Produces

- positional expansion generic iterable fallback using canonical iteration semantics;
- `VM::has_iteration_protocol` deleted or no longer cursor-based/needed;
- no runtime trait solver.

### Files and symbols

**Read/Modify as live ownership requires:**
- `phalcom-core/src/compiler/lib/expr.rs`
- `phalcom-core/src/compiler/boundedness.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/bytecode.rs` only if an existing bytecode contract needs a local extension
- relevant COLL005/COLL001 expansion tests, not their historical plan prose.

### Required implementation shape

1. Preserve optimized tuple/Unit positional extraction fast path.
2. For statically typed sources with canonical Iterable evidence, compiler/lowering should drive a stateful iterator using the same preselected semantic evidence pattern as `for` rather than ask the VM whether methods exist.
3. Reuse existing pack/list push bytecodes/builders; do not redesign pack assembly.
4. For `Dynamic` expansion, preserve current normative dynamic-boundary behavior only if an already-existing runtime capability can execute an attached/precomputed evidence recipe without proof search. If the only possible implementation would search conformance or structurally probe `next`, STOP AND CONSULT. Do not hide the cursor probe behind a renamed helper.
5. Remove `has_iteration_protocol` if no other legitimate use remains.

### Forbidden approaches

- no `has_method("next")` replacement;
- no VM trait-name lookup;
- no pack architecture rewrite;
- no regression of tuple fast lane.

### Test changes required

Cover CV-112 plus existing outgoing/list positional expansion cases for tuple, Unit, canonical Iterable, failure, and provably-unbounded rejection.

### Tests to run now

Use exact expansion test filters discovered from live tree, plus:

```sh
rg -n 'has_iteration_protocol|iterate|iteratorValue' phalcom-core/src/vm/dispatch.rs phalcom-core/src/compiler/lib/expr.rs
```

### Tests explicitly deferred

Unrelated labeled expansion and complete pack suites unless touched code crosses them.

### Acceptance

- generic positional expansion no longer depends on cursor selectors;
- tuple/List target fast paths still pass;
- VM contains no iteration protocol structural proof helper;
- no new runtime trait solver.

### Local STOP / CONSULT triggers

Dynamic expansion cannot retain accepted behavior without runtime proof/search or material semantic change.

### Checkpoint update

Record removal of VM cursor dependency because it unblocks final retirement.

---

## T13 — Migrate repository legacy subclasses/use sites and close cursor consumers

### Purpose

Remove remaining production/library/example reliance on inheriting legacy `Iterable` or implementing cursor selectors before deleting the runtime class.

### Preconditions

- T4–T12 canonical consumers live.

### Consumes

- canonical source/iterator authoring pattern;
- final T0 census.

### Produces

- `RefRange` and any other live repository source migrated to explicit Iterable + iterator class;
- no production code requires cursor selectors.

### Files and symbols

**Modify:** at least `examples/sheetcalc/src/grid/ref-range.ph`; any additional live hits from final grep.

### Required implementation shape

For `RefRange`, preserve row-major semantics but split reusable source from traversal state:

```text
RefRange.iterator -> RefRangeIterator.new(self)
RefRangeIterator fields -> source + ordinal + done
next -> Some(Ref) / None
```

Use canonical conformance declarations. Do not encode cell traversal in compiler/runtime.

Run final production search excluding historical docs:

```sh
rg -n 'iterate\(|iteratorValue\(|SourceIterator|is Iterable|class Iterator|class Iterable' \
  phalcom-core phalcom-semantic phalcom-modules phalcom-native-meta examples \
  -g '!target/**'
```

Classify every hit as canonical trait declaration, historical test string intentionally checking absence, or defect. There must be no live cursor consumer before T14.

### Forbidden approaches

- no compatibility subclass of old runtime Iterable;
- no hidden cursor adapter around canonical iterator.

### Test changes required

CV-64, CV-65, CV-90, CV-115 dependency.

### Tests to run now

Run the smallest Sheetcalc/example test/build target that covers RefRange plus final grep. Do not run all examples.

### Tests explicitly deferred

Runtime class deletion tests to T14.

### Acceptance

No live source outside legacy class file itself requires cursor protocol or runtime Iterable inheritance.

### Local STOP / CONSULT triggers

A live external-facing repository API explicitly promises nominal runtime `Iterable` class identity rather than trait conformance.

### Checkpoint update

Record migrated live use sites only if externally meaningful.

---

## T14 — Retire legacy runtime Iterable/Iterator classes and cut public names to traits

### Purpose

Perform the destructive final cutover after every consumer is canonical: remove `collections/iterable.ph` cursor hierarchy, runtime `iterable_class`, collection superclass wiring, and competing public class names so `Iterable` / `Iterator` denote traits only.

### Preconditions

- T13 final consumer census clean.
- default toolbox fully replaces inherited algorithms.
- expansion cursor probe gone.

### Consumes

- canonical trait modules/defaults/adapters/conformances;
- current runtime class bootstrap/invariant tables.

### Produces

- no runtime `Iterable` class identity;
- List/Map/Set/Tuple/Range/Bytes runtime classes parent appropriate ordinary object roots, not Iterable;
- no source class `Iterator` / `SourceIterator` legacy root;
- package public names resolve canonical traits.

### Files and symbols

**Delete or remove exposure:**
- `phalcom-core/core/universe/src/collections/iterable.ph` after any still-useful non-cursor code has migrated to canonical modules.

**Modify:**
- `collections/package.ph`
- `phalcom-modules/src/builtin.rs`
- `phalcom-core/src/universe/core_classes.rs`
- `phalcom-core/src/universe/invariants.rs`
- `phalcom-core/src/universe/primitives.rs`
- `phalcom-core/src/vm/bootstrap.rs`
- `phalcom-native-meta/src/universe.rs`
- any Universe tests/global binding lists that include `iterable_class`.

### Required implementation shape

1. Delete `make_core_class(... "Iterable" ...)` and `CoreClasses.iterable_class`.
2. Reparent runtime native collection classes to `object_class` (or their independently normative superclass if live source says otherwise):

```text
List, Map, Set, Tuple, Range, Bytes
```

3. Remove sealed-class/bootstrap/global-binding/primitive/invariant/native-meta entries for runtime Iterable.
4. Keep canonical trait declaration module path/identity stable. Do not move/redeclare traits merely to obtain pretty public names.
5. Adjust package exposure so `Iterable` and `Iterator` public resolution comes from `iterator_laws` canonical declarations, and helper adapter modules do not accidentally shadow them.
6. Remove legacy source classes `Iterator`, `SourceIterator`, `IndexedIterable`, `ZippedIterable`, and cursor adapters only after their canonical replacements are live.
7. Any native primitive formerly attached to runtime Iterable must already have migrated or be deleted as dead; do not reattach generic traversal as Object methods.

### Forbidden approaches

- no compatibility runtime class named Iterable after completion;
- no Object-wide iterator methods;
- no trait redeclaration under a new module;
- no keeping cursor selectors as undocumented fallback.

### Test changes required

Cover:

```text
CV-58 no cursor sends
CV-65 no production cursor protocol
CV-66 public Iterator/Iterable resolve traits
CV-113 no iterable_class census
plus core class/bootstrap invariant tests
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core

rg -n 'iterable_class|class Iterable|class Iterator|SourceIterator|iterate\(|iteratorValue\(' \
  phalcom-core phalcom-semantic phalcom-modules phalcom-native-meta examples \
  -g '!target/**'
```

### Tests explicitly deferred

Final corpus stabilization to T15.

### Acceptance

- `CoreClasses` has no iterable_class;
- collection runtime superclass graph no longer contains Iterable;
- public source `Iterable` / `Iterator` are traits;
- no live production cursor selectors/classes remain;
- P4 trait identity tests still pass.

### Local STOP / CONSULT triggers

- removing iterable_class changes non-iteration runtime layout/identity in a nonlocal way;
- public trait names cannot be exposed without module/name-resolution redesign;
- a native primitive requires legacy class identity for reasons outside iteration.

### Checkpoint update

Record runtime class/public-name retirement as a durable checkpoint event.

---

## G5 — Retirement gate

Run the T14 focused commands plus:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
```

Expected: no cursor runtime fallback is needed to make these pass.

---

## T15 — Update current docs/tooling, run final census, stabilize, and write records

### Purpose

Reconcile current user/developer documentation and tooling with the canonical production model, execute the focused P5 stabilization ladder, classify residual failures, and leave durable walkthrough/handoff state for P6.

### Preconditions

- G5 passed.

### Consumes

- final source implementation and final T0/T13/T14 censuses.

### Produces

- current docs/tooling no longer describe cursor semantics;
- full P5 coverage disposition;
- P5 walkthrough/handoff;
- checkpoint plan ledger updated.

### Files and symbols

**Modify as live hits require:**
- `docs/specs/iterator.md` only for actual final API/catalog wording, not architecture rewrite
- `docs/guide/collections.md`
- `docs/wiki/collections/traversal.md`
- `../../../__unstructured__/spec/current/iteration.md` under the repository's specs migration/supersession convention
- LSP hover strings/current semantic presentation text mentioning cursor selectors
- AST/compiler comments and corpus comments that claim cursor lowering
- `LANG005.C6-CHECKPOINT.md`

**Create:**
- `LANG005.C6.P5-walkthrough.md`
- `LANG005.C6.P5-handoff.md`

### Required implementation shape

1. Run a final source/docs grep and classify historical hits rather than deleting history blindly:

```sh
rg -n 'iterate\(|iteratorValue\(|cursor protocol|SourceIterator|iterable_class|runtime Iterable|class Iterator|class Iterable' \
  . -g '!target/**'
```

Allowed remaining hits:
- explicitly historical/retired decision/implementation documents whose text is clearly contextualized as historical;
- tests whose purpose is asserting absence (if any).

Current specs/guides/comments/LSP must not teach cursor semantics.

2. Update disassembly comments to the new invariant: direct jump + preselected iterator/next trait invocation, no body closure, no cursor send. Do not retain obsolete "zero allocation loop" as a requirement because stateful iterator allocation is intentional.
3. Run final focused gates in §19/G6 below serially.
4. Build coverage disposition mapping every `P5-CV-01..115` to test/evidence/result. A coverage case may share a test; do not create ceremonial one-test-per-ID files.
5. Classify any remaining failures A/B/C/D. P5 may finish `FOCUSED_TESTED` with documented unrelated D failures; A/B acceptance failures block completion.
6. Write walkthrough/handoff per §§23–24.

### Forbidden approaches

- no rewriting all historical ADRs to pretend cursor design never existed;
- no broad workspace cleanup;
- no marking P5 release certified without CERTIFY gates;
- no hiding unrun tests under PASS language.

### Tests to run now

See G6.

### Tests explicitly deferred

Workspace/release certification unless explicitly promoted.

### Acceptance

- current docs/tooling align with canonical model;
- final production grep clean;
- all required focused gates pass or nonblocking baseline failures classified;
- checkpoint/walkthrough/handoff truthful and synchronized.

### Local STOP / CONSULT triggers

A current authoritative spec conflicts materially with implemented final semantics; do not edit the spec merely to match code without principal-architect ruling.

### Checkpoint update

Complete P5 plan ledger entry, verification state, durable invariants, deferred failures, consultations, next action `LANG005.C6.P6`.

---

# 19. Verification gates

## G1 — Semantic/lowering/List vertical

Defined after T3. Proves first canonical vertical before compiler migration.

## G2 — Production `for` cutover

Defined after T5. Proves every loop form is evidence-driven and cursor-free.

## G3 — Built-in source migration

Defined after T8. Proves core iterable families use stateful objects.

## G4 — Iterator/default surface

Defined after T10. Proves one-shot lazy pipelines + inherited terminals/eager wrappers.

## G5 — Runtime/public retirement

Defined after T14. Proves no hidden legacy class/protocol is needed.

## G6 — Final focused P5 stabilization

Purpose: prove the entire P5 ownership area without escalating to workspace release certification.

Run serially:

```sh
# Semantic authority + incrementality
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance

# Core/runtime/lowering
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration_stateful
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance

# Production source-language surfaces
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bytes
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections_literals_negative

# Affected crates compile
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core

# Hygiene/census
git diff --check
rg -n 'iterable_class|has_iteration_protocol|iterate\(|iteratorValue\(|SourceIterator' \
  phalcom-core phalcom-semantic phalcom-modules phalcom-native-meta examples \
  -g '!target/**'
```

Expected:

- all focused P5 tests pass;
- cursor/runtime-class production grep has no live hits except explicitly justified negative-test text;
- `BL-P5-01` is either gone because its obsolete cursor path disappeared, or remains classified without invalidating canonical tests; if it blocks the canonical path, consultation rules apply;
- no zero-test filters are counted as evidence.

Do not automatically run workspace test/clippy after G6 PASS.

---

# 20. Final focused acceptance

P5 may be marked `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only if:

```text
semantic lane product exists and fingerprints correctly
compiler consumes projected iterator/next evidence
single and generalized for are cursor-free
parallel setup order is two-phase
strict zip error is canonical
List/Set/Tuple/Bytes/Map/Range/String views are migrated
Map direct Item is labeled tuple
Range remains forward-only and unbounded-capable
lazy adapters are stateful one-shot with sticky exhaustion
custom Iterator Item+next gets defaults + for
Iterable terminal surface replaces runtime-superclass algorithms
direct collection eager transforms remain ergonomic
boundedness recognizes canonical surface conservatively
argument expansion no longer probes cursor selectors
repository subclasses/use sites migrated
CoreClasses.iterable_class removed
public Iterator/Iterable are traits
current docs/tooling describe canonical model
final production cursor census is clean
GC + incremental + disassembly evidence passes
```

For the walkthrough, include a table:

| Coverage ID | Test/evidence | Result |
|---|---|---|
| P5-CV-01 | ... | PASS |
| ... | ... | ... |
| P5-CV-115 | ... | PASS / classified |

Do not mark unexecuted cases PASS merely because the implementation looks correct; use `COVERED_BY` / `NOT_SEPARATELY_RUN` language where a case is logically covered by a named shared test.

---

# 21. Performance/resource evidence

P5 has a local hot-path performance contract, not a benchmarking program.

Required evidence:

1. disassembly/code inspection proves no `iterate` / `iteratorValue` sends and no runtime trait lookup in loop lowering;
2. semantic proof/projection occurs before codegen and is not invoked from VM `next` execution;
3. each loop lane acquires iterator once per loop execution;
4. each loop attempted step emits one `next` invocation/lane;
5. runtime evidence environments are prepared/reused through existing executable semantics rather than rebuilt by loop compiler per step;
6. lazy adapters allocate one object/stage and no eager intermediate Lists;
7. GC tests show ordinary field retention is sufficient;
8. Map entry tuple allocation is accepted correctness cost; no new pair heap representation is introduced.

No microbenchmark threshold is required in P5. If a concrete regression appears, collect a focused before/after instruction/allocation profile and either make a bounded correctness-preserving fix or defer optimization to P6/performance work.

---

# 22. Checkpoint bookkeeping

## At plan start

- set `active_plan: LANG005.C6.P5`;
- record live revision/worktree baseline and `BL-P5-01` disposition;
- do not rewrite historical P1-P4 records.

## During plan

Update checkpoint only for durable events:

- T1 semantic lane product established;
- G2 canonical `for` cutover;
- Map externally visible Item migration;
- runtime superclass/public-name retirement;
- consultation/amendment;
- deferred failure;
- named verification gate.

## At plan completion

- mark P5 complete/implemented;
- set truthful verification state;
- record final iteration invariants;
- record deferred/baseline failures;
- record consultations/amendments;
- set next action to P6.

---

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-walkthrough.md
```

It must contain:

- final result;
- exact architecture implemented;
- semantic lane product and lowering interface names as landed;
- Universe source/module layout as landed;
- important code changes by T-task;
- any mechanical deviations from this plan;
- consultation decisions / amendments;
- tests added;
- tests actually run with counts/results;
- tests intentionally deferred;
- `BL-P5-01` final disposition;
- P5-CV-01..115 coverage disposition;
- final cursor/runtime-class census;
- performance/resource evidence;
- residual risks and P6 handoff items.

---

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-handoff.md
```

It must contain:

- completion revision/current worktree state;
- stable semantic/lowering interfaces introduced by P5;
- canonical public Iterator/Iterable module path/identity;
- built-in source/iterator conformance inventory;
- default surface ownership;
- no-cursor/no-runtime-Iterable invariants;
- boundedness/expansion integration state;
- deferred failures;
- P6 next objective;
- must-read files;
- first recommended P6 commands;
- explicit list of things P6 must not redesign.

---

# 25. Completion truth table

Keep these states distinct:

```text
source written
semantic products published
runtime sources migrated
tests added
targeted BUILD tests passed
G1..G6 focused gates passed
P5 checkpoint plan accepted
C6 checkpoint accepted
release certified
```

Expected P5 terminal metadata is normally:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

unless a documented baseline blocker prevents a required P5 acceptance gate. Do not call P5 `CERTIFIED` solely because G6 passes.

---

# 26. Plan self-review checklist

Before executing or handing this plan to Luna, verify:

- every `INV-*` has a named task owner;
- T1 is the sole semantic owner of lane proof/projection meaning;
- T2/compiler never re-proves iteration;
- T4/T5 use preselected trait calls and direct jumps;
- no task reintroduces structural conformance;
- Option payload extraction is value-safe and does not use `unwrapOr`;
- two-phase parallel order is explicit in implementation and tests;
- strict zip error class is ordinary source Error, not VM ABI;
- built-in stateful iterators are ordinary traced objects;
- Map tuple Item and Range subset are fixed;
- default API split avoids Iterator/Iterable selector precedence invention;
- T12 explicitly blocks runtime trait proof for dynamic expansion;
- destructive runtime class deletion is after every consumer migration;
- final grep includes VM expansion and example subclasses, not only compiler loops;
- testing is sparse during BUILD and broad only at named gates;
- all 115 coverage scenarios remain traceable;
- checkpoint/walkthrough/handoff obligations are explicit;
- no placeholders/TODOs remain in the plan;
- later tasks consume interfaces produced by earlier tasks in dependency order.

Plan dependency order:

```text
T0
 ↓
T1 → T2 → T3 → G1
            ↓
           T4 → T5 → G2
                     ↓
              T6 → T7 → T8 → G3
                            ↓
                         T9 → T10 → G4
                                   ↓
                                  T11
                                   ↓
                                  T12
                                   ↓
                                  T13
                                   ↓
                                  T14 → G5
                                         ↓
                                        T15 → G6
```

T11 can begin after the final selector surface from T9/T10 is stable. T12 can be investigated earlier but must not delete the VM cursor probe until canonical expansion support is executable. T13/T14 remain intentionally late.

---

# 27. Requirements traceability and coverage matrix

The final requirements matrix is reproduced below so the implementer never needs to infer whether a semantic edge was intentionally omitted. Test execution remains grouped by tasks/gates above; this is a design ledger, not an instruction to create 115 independent tests.

## 27.1 Requirement-family ownership

| Requirement family | Primary task owner(s) | Gate / closure evidence |
|---|---|---|
| `P5-FOR-01..26` | T1 semantic lane resolution; T2 lowering projection; T4 single-lane lowering; T5 generalized/strict-zip lowering | G1, G2 |
| `P5-COL-*` / `P5-ITER-*` | T3 List vertical; T6 core sources/views; T7 Map; T8 Range; T9/T10 shared surface | G1, G3, G4 |
| `P5-MAP-01..10` | T7 | G3 |
| `P5-RANGE-01..10` | T8; T11 for boundedness interaction | G3, G4 |
| `P5-LAZY-01..18` | T9; GC/lifecycle closure in T15 | G4, G6 |
| `P5-SURFACE-01..20` | T9 lazy adapters; T10 Iterable terminals/eager forwarding; T11 boundedness catalog | G4 |
| `P5-GEN-01..08` | T1/T2 evidence ownership; T3–T10 source-authored conformances/default execution | G1–G4 |
| `P5-RT-01..14` | T2/T4/T5 selected execution; T12 expansion migration; T14 runtime-class retirement | G2, G5 |
| `P5-PERF-01..10` | T2/T4/T5 no re-proof; T9/T10 allocation/laziness shape; T15 disassembly/census | G2, G4, G6 |
| `P5-INC-01..09` | T1 callable product/dependencies/fingerprint; T2 lowering invalidation; T15 incremental stabilization | G1, G6 |
| `P5-RETIRE-01..14` | T12 expansion; T13 repository consumers; T14 runtime/public retirement; T15 final census | G5, G6 |
| `P5-NAME-01..06` | T14 public-name cutover; T15 source/docs census | G5, G6 |
| `P5-DOC-01..06` | T15 | G6 |

This table assigns implementation ownership; it does not weaken the detailed requirement text in the final analysis. If a requirement appears to demand a materially different owner during execution, STOP AND CONSULT rather than moving the authority silently.

## 27.2 Comprehensive coverage matrix imported from the final requirements analysis

The matrix below is requirements-driven. It is not a test execution plan; this Luna plan selects the minimum discriminating test at each patch and reserves broad stabilization for named gates.

| Coverage ID | Scenario | Principal requirements |
|---|---|---|
| P5-CV-01 | empty single-lane List | FOR-05, COL-02/03, ITER lifecycle |
| P5-CV-02 | one-item List | FOR-05, FOR-14 |
| P5-CV-03 | multi-item List | FOR-05, FOR-17 |
| P5-CV-04 | source expression side effect occurs once | FOR-01 |
| P5-CV-05 | iterator getter side effect occurs once | FOR-03/04 |
| P5-CV-06 | two-lane source evaluation order | FOR-02/04 |
| P5-CV-07 | strict zip equal lengths | FOR-19 |
| P5-CV-08 | strict zip initial mismatch | FOR-05/06/20 |
| P5-CV-09 | strict zip late mismatch | FOR-05/06/20 |
| P5-CV-10 | all lanes exhaust together | FOR-19 |
| P5-CV-11 | pattern tuple binding | FOR-09 |
| P5-CV-12 | pattern mismatch raises | FOR-09/12 |
| P5-CV-13 | `at` sequence 0,1,2 | FOR-10 |
| P5-CV-14 | `at` over Map does not expose keys as ordinal | FOR-10, MAP-01 |
| P5-CV-15 | `continue` advances once | FOR-08 |
| P5-CV-16 | `break` prevents later `next` | FOR-07 |
| P5-CV-17 | nested loops reacquire inner iterator per execution | FOR-11 |
| P5-CV-18 | return from loop body | FOR-12/17 |
| P5-CV-19 | error from iterator acquisition | FOR-12 |
| P5-CV-20 | error from `next` | FOR-12, ITER lifecycle |
| P5-CV-21 | closure captures each loop iteration independently | FOR-18 |
| P5-CV-22 | generic `T |= Iterable` loop | GEN-01/02/03 |
| P5-CV-23 | generic `T |= Iterator` loop via derived Iterable | GEN-05 |
| P5-CV-24 | conditional Iterable conformance | GEN-06 |
| P5-CV-25 | cross-module conformance | GEN-07 |
| P5-CV-26 | associated Item projection normalization | GEN-02/08 |
| P5-CV-27 | exact enum-case iterable identity | section 19 |
| P5-CV-28 | exact enum-case item into generic body call | section 19 |
| P5-CV-29 | two independent List iterators | ITER-02 |
| P5-CV-30 | Iterator.iterator returns same traversal | ITER-04/10 |
| P5-CV-31 | repeated next after exhaustion remains None | ITER-09/10 |
| P5-CV-32 | GC while List iterator live | ITER-07, RT-11/12 |
| P5-CV-33 | Set encounter-order traversal | COL-05 |
| P5-CV-34 | Tuple Dynamic item traversal | COL-01/02 |
| P5-CV-35 | Bytes yields Int octets | COL-01/02 |
| P5-CV-36 | Map yields labeled `(key,value)` tuple | MAP-01/05 |
| P5-CV-37 | generic Map<K,V> item typing | MAP-02 |
| P5-CV-38 | Map destructuring loop | MAP-08 |
| P5-CV-39 | Map keys/values/entries views | MAP-06 |
| P5-CV-40 | Range exclusive bound | RANGE-03/05 |
| P5-CV-41 | Range inclusive bound | RANGE-03/05 |
| P5-CV-42 | Range unbounded upper | RANGE-03 |
| P5-CV-43 | Range missing lower failure | RANGE-04 |
| P5-CV-44 | Range lower > upper current behavior | RANGE-03/06 |
| P5-CV-45 | `source.iter.map` laziness | LAZY-01/03/10 |
| P5-CV-46 | filter consumes until match | LAZY-04 |
| P5-CV-47 | skip applies only once | LAZY-05 |
| P5-CV-48 | take stops without extra upstream read | LAZY-06 |
| P5-CV-49 | takeWhile sticky stop | LAZY-07 |
| P5-CV-50 | flatMap crosses empty/non-empty inners | LAZY-08 |
| P5-CV-51 | indexed iterator ordinal | LAZY-09 |
| P5-CV-52 | zipped view strict mismatch | LAZY-09, FOR-19 |
| P5-CV-53 | GC while lazy adapter/callback live | ITER-08 |
| P5-CV-54 | non-Iterable with matching method names rejected | FOR-13 |
| P5-CV-55 | invalid Iterable associated Iterator rejected | GEN-04 |
| P5-CV-56 | Item equality mismatch rejected | GEN-04 |
| P5-CV-57 | ambiguous/coherence conflict remains diagnostic | GEN-06 |
| P5-CV-58 | disassembly/runtime product shows no cursor sends | RETIRE-12, RT-05 |
| P5-CV-59 | no trait proof/runtime lookup in loop hot path | RT-01/05, PERF-01/05 |
| P5-CV-60 | body-only incremental edit does not invalidate iteration evidence globally | INC-01/02 |
| P5-CV-61 | Iterable conformance edit invalidates affected loop | INC-03 |
| P5-CV-62 | Item/Iterator binding edit invalidates affected loop | INC-04 |
| P5-CV-63 | iterator witness edit invalidates executable semantics | INC-05 |
| P5-CV-64 | repo source legacy Iterable subclass migrated | COL-08, RETIRE-09 |
| P5-CV-65 | final repository search has no production cursor protocol | RETIRE-12 |
| P5-CV-66 | final public `Iterable`/`Iterator` resolve to traits | NAME-01/02/03 |
| P5-CV-67 | parallel side effects prove all sources evaluate before any iterator acquisition | FOR-04/21 |
| P5-CV-68 | `break`/`continue` remain unlabelled; labeled form rejected by parser/semantic authority | FOR-22 |
| P5-CV-69 | strict multi-lane mismatch raises `ZipLengthMismatchError` | FOR-20, LAZY-17 |
| P5-CV-70 | custom Iterator implements only Item + next and gains `for` + defaults | SURFACE-01/11 |
| P5-CV-71 | two `.iter` accesses on same reusable source create independent traversals | SURFACE-02/03 |
| P5-CV-72 | same lazy pipeline object is one-shot and stays exhausted | SURFACE-05/06/17 |
| P5-CV-73 | recreating pipeline from reusable source restarts traversal independently | SURFACE-02/05 |
| P5-CV-74 | aliasing upstream iterator and adapter shares traversal state | SURFACE-07/08 |
| P5-CV-75 | Iterable eager `map` uses fresh traversal and returns current concrete eager result | SURFACE-04/09 |
| P5-CV-76 | Iterator `map` is lazy and callback runs only on `next`/consumption | LAZY-03, SURFACE-04 |
| P5-CV-77 | `filterMap` skips None and yields Some payloads | LAZY-11 |
| P5-CV-78 | `flatten` handles empty and multiple inner iterables | LAZY-12 |
| P5-CV-79 | `chain` switches exactly once after first exhaustion | LAZY-13 |
| P5-CV-80 | `skipWhile` predicate stops after first retained item | LAZY-14 |
| P5-CV-81 | `mapWhile` is sticky after first None | LAZY-15, SURFACE-17 |
| P5-CV-82 | `inspect` side effect once per yielded item and item unchanged | LAZY-16 |
| P5-CV-83 | `nth` advances exactly n+1 items or exhausts | SURFACE-13 |
| P5-CV-84 | `last` exhausts and returns final item / None | SURFACE-13 |
| P5-CV-85 | strict pairwise zip never truncates shorter input | SURFACE-14, LAZY-17 |
| P5-CV-86 | no public fuse needed; adversarial adapter cannot restart after None | SURFACE-17 |
| P5-CV-87 | String bytes view works in generic `for<T where T |= Iterable>` | COL-09/10, GEN-01 |
| P5-CV-88 | String codePoints view works as canonical Iterable without making String iterable | COL-09/10 |
| P5-CV-89 | Map.entries item agrees with direct Map labeled tuple shape | MAP-01/09 |
| P5-CV-90 | legacy Entry-dependent repository use is migrated or explicitly retained by authority | MAP-10 |
| P5-CV-91 | unbounded Range `.iter.take(n).toList` terminates without Range size | RANGE-10, LAZY-06 |
| P5-CV-92 | reversed/stepped Range semantics remain unsupported rather than invented | RANGE-09 |
| P5-CV-93 | default-rich iterator implementation introduces no new VM proof search/native combinator floor | SURFACE-19/20, RT-01 |
| P5-CV-94 | generic lazy-adapter / terminal defaults use trait evidence; concrete eager wrappers do not duplicate traversal algorithms | SURFACE-09/11 |
| P5-CV-95 | GC during flatMap/flatten with current inner iterator preserves upstream/callback/inner state | LAZY-08/12/18 |

Coverage cases P5-CV-01..95 preserve the pass-2 matrix; the pass-3 additions below cover only newly closed architectural seams rather than inflating the matrix mechanically.

---

### Pass-3 closure coverage additions

| Coverage ID | Scenario | Principal requirements |
|---|---|---|
| P5-CV-96 | semantic snapshot exposes one canonical lane resolution per `for` source expression | FOR-23/24 |
| P5-CV-97 | lowering snapshot/spec uses selected/requirement iterator + next invocations without compiler trait lookup | FOR-25/26, RT-03/04 |
| P5-CV-98 | source 2 failure occurs before iterator getter 1 in parallel loop | FOR-21 |
| P5-CV-99 | iterator getter 1 failure occurs only after all lane source expressions ran | FOR-21 |
| P5-CV-100 | generic `T |= Iterator` can call `.iter` through derived Iterable evidence without selector ambiguity | SURFACE-03/11, GEN-05 |
| P5-CV-101 | arbitrary custom Iterable uses `source.iter.map(...).toList` without defining map itself | SURFACE-02/03/09 |
| P5-CV-102 | direct `List.map` remains eager while `List.iter.map` is lazy | SURFACE-04/10 |
| P5-CV-103 | direct `Map.contains(key)` keeps key semantics while `map.iter.contains(entry)` uses entry-item semantics | MAP-07, SURFACE-18 |
| P5-CV-104 | no `SourceIterator` allocation occurs merely to implement `.iter`; reusable source returns its concrete iterator | SURFACE-03 |
| P5-CV-105 | Map.entries.toMap round-trip works with labeled tuple `.key/.value` | MAP-09/10 |
| P5-CV-106 | StringByteSequence iterates Int bytes through canonical Iterable conformance | COL-10 |
| P5-CV-107 | StringCodePointSequence iterates integer code points through canonical Iterable conformance | COL-10 |
| P5-CV-108 | GC during nested flatMap/flatten retains outer, callback, current inner iterator, and state | LAZY-08/12/18, RT-11/12 |
| P5-CV-109 | callable semantic fingerprint changes when selected iterator/next evidence changes | INC-05/07/09 |
| P5-CV-110 | body-only loop edit leaves lane evidence product semantically stable when dependencies are unchanged | INC-01/02 |
| P5-CV-111 | boundedness recognizes canonical `.iterator`/`.iter` and new `take`/map/filter/etc. pipeline propagation without false Unbounded claims | P5/P6 boundary, SURFACE-03 |
| P5-CV-112 | positional `*` expansion no longer depends on `iterate`/`iteratorValue` lookup | RT-13/14, RETIRE-13 |
| P5-CV-113 | final CoreClasses/bootstrap/invariants census contains no `iterable_class` | RETIRE-06/11 |
| P5-CV-114 | current LSP hover/AST comments describe iterator/next semantics, not cursor protocol | RETIRE-14, DOC requirements |
| P5-CV-115 | final production grep finds cursor names only in historical/superseded documentation where explicitly contextualized | RETIRE-12/14 |


---

# 28. Pass-1 planning status and next refinement pass

This is **planning pass 1 of 3**, but it is intentionally written as a complete executable patch-grade plan rather than a sketch. Pass 2 must improve this same file in place: re-audit repository mechanics, challenge task boundaries/interfaces/test budget, close any overlooked source/module/expansion/tooling dependencies, and make the plan less error-prone without reopening the ratified P5 architecture. Pass 3 must perform the final coherence/coverage/issue-free audit and produce the planning artifact intended for execution.

No implementation work is authorized by this planning artifact alone unless the user separately asks to execute it.
