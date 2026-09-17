---
id: LANG005.C6.P5.PLAN-HEADER
parent_plan: LANG005.C6.P5
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan-shared-header
planning_cycle: restructured-pass-2
file_pass: 1-of-4
status: PROPOSED
completion: HEADER_COMPLETE
verification: REMOTE_BASELINE_RECHECKED_AND_STRUCTURALLY_VERIFIED
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: 06e37b6830cd9114d03f2a794d0b9a973f8cdc82
planning_revision_subject: "feat(lang005): complete P4 iterator iterable law staging"
requirements_analysis: LANG005.C6.P5-requirements-analysis.md
supersedes_monolithic_after_plan_set_completion: LANG005.C6.P5-production-stateful-iteration-migration-plan.md
executor_profile: GPT-5.6-Luna-High-or-Extra-High
---

# LANG005.C6.P5 — Shared Plan Header: Production Stateful Iteration Migration

> **For agentic workers:** this file is the mandatory shared header for all three LANG005.C6.P5 implementation-plan parts. Read this header first, then read only the active part and the repository files it names. Do not infer missing architecture from another part or from the superseded monolithic draft.
>
> **Required execution style:** use the repository's Luna patch-grade workflow: coherent reviewer-sized tasks, sparse discriminating tests in BUILD MODE, named stabilization gates, bounded debugging, objective STOP/CONSULT triggers, durable checkpoint updates, and no architectural improvisation.
>
> **Plan-set rule:** the three part files may specialize this header with additional task-local constraints, but they may not weaken or contradict it. If a part appears to conflict with this header, STOP AND CONSULT rather than selecting whichever wording is convenient.

**Goal:** Replace Phalcom's production cursor protocol and runtime `Iterable` class hierarchy with the canonical P4 `Iterator` / `Iterable` trait model, make `for` evidence-driven over stateful `next -> Option<Item>` iterators, provide a production-ready iterator/default surface, migrate all built-in and repository traversal consumers, and retire every live cursor dependency without moving semantic proof into the VM.

**Architecture:** Formal iteration meaning remains semantic-layer authority. Semantic analysis proves canonical `Iterable` conformance, normalizes associated `Item` / `Iterator` projections, and publishes selected `iterator` / `next` evidence. `ModuleLoweringSemantics` projects those decisions into executable trait-invocation specs. The compiler emits direct-jump loops over ordinary stateful iterator objects; the VM executes already-selected calls only. Built-in reusable sources create fresh iterator instances, iterator adapters keep ordinary traced state fields, and P4's derived `Iterator => Iterable` law supplies custom iterators with broad defaults from only `Item + next`.

**Tech stack:** Rust workspace; `phalcom-semantic`; `phalcom-core`; source-authored Phalcom Universe; canonical trait/generic evidence; associated-type normalization; `ModuleLoweringSemantics`; existing `InvokeTraitSelected` / `InvokeTraitRequirement`; canonical `Option` ADT representation; compiler boundedness metadata; ordinary GC-traced `InstanceObject` fields.

**Primary specification / analysis:** `LANG005.C6.P5-requirements-analysis.md` final pass 3/3; `docs/specs/iterator.md`; `docs/specs/generic-constraints.md`; `docs/specs/objects/traits.md`; `docs/specs/objects/impl.md`; `docs/specs/associated-types.md`; C6 checkpoint/guidance; P4 walkthrough/handoff.

---

# 0. Plan-set contract and navigation

The former single-file plan was intentionally too large for reliable one-turn refinement and implementation consumption. P5 is therefore represented as one shared header plus three ordered implementation parts. The split is by architectural responsibility, not by arbitrary line count.

## 0.1 File set

| File | Role | Stable task range | Main outcome |
|---|---|---:|---|
| `LANG005.C6.P5-plan-00-shared-header.md` | shared architecture / policy / invariants | none | one source of plan-wide truth |
| `LANG005.C6.P5-plan-01-semantic-and-for-cutover.md` | Part I — semantic authority and production `for` | T0–T5, G1–G2 | canonical evidence product, lowering projection, List vertical, all `for` forms cut over |
| `LANG005.C6.P5-plan-02-builtins-and-iterator-surface.md` | Part II — built-in traversal and production API | T6–T11, G3–G4 | all core sources/views canonical, iterator toolbox live, boundedness aligned |
| `LANG005.C6.P5-plan-03-retirement-and-stabilization.md` | Part III — integration, retirement, stabilization | T12–T15, G5–G6 | expansion migrated, legacy consumers removed, runtime class/cursors retired, docs/records stabilized |

The task and gate numbers above are **stable across the split**. Do not renumber T0–T15 or G1–G6 in later files; checkpoint records, requirements traceability, and consultation packets may refer to them.

## 0.2 Authority of the split files

The split plan set becomes the execution authority as its files are generated. The monolithic Pass-1 plan remains historical planning input only and must not be used to override a newer split-file decision.

Precedence inside P5 planning artifacts is:

```text
this shared header
    ↓
active part file
    ↓
approved amendment / consultation decision recorded in checkpoint
    ↓
older monolithic P5 planning draft
```

Normative language/spec authorities remain above every implementation plan; see §5.

## 0.3 Cross-part execution rule

Implementation is sequential:

```text
Part I  ──G2──>  Part II  ──G4──>  Part III  ──G6──>  P5 complete
```

A later part may begin only after the preceding part's terminal gate is either:

- PASS; or
- explicitly accepted with a documented baseline/deferred failure that does not invalidate the next part's preconditions.

Do not run ahead by deleting legacy cursor infrastructure in Part I or Part II merely because the replacement path exists. Retirement is deliberately concentrated in Part III after all known consumers have migrated.

## 0.4 What belongs in the header versus part files

This header owns and is the **single plan source of truth** for:

- executor authority;
- checkpoint objective;
- repository baseline;
- normative authorities;
- cross-layer architecture;
- global ownership boundaries;
- global invariants;
- non-goals;
- fixed/flexible/verify-first classifications;
- consultation and debugging policy;
- verification modes and broad-test budget;
- unrelated-failure policy;
- cross-part dependency contracts;
- checkpoint/walkthrough/handoff obligations.

Each part file owns:

- task-local repository re-grounding relevant to that part;
- exact task contracts;
- exact files and symbols;
- required implementation shape;
- task-local forbidden approaches;
- task-local tests and commands;
- task-local acceptance and STOP/CONSULT triggers;
- the exact subset of the final `P5-CV-*` coverage ledger it must prove.

Part files must **reference** global rules by invariant/section ID rather than copy and subtly mutate them.

---

# 1. Executor contract

This plan set is written for GPT-5.6 Luna High/Extra High operating under constrained architectural authority. The principal-architect work has already chosen the important ownership, identity, semantic, runtime, lifecycle, and verification boundaries. Luna's job is to implement them efficiently against live repository state.

The implementer:

- may adapt helper names, private decomposition, imports, exact fixture names, and mechanically drifted private signatures to live HEAD;
- must preserve every `FIXED` decision and every `INV-*` invariant in this header;
- must not redesign trait identity, semantic ownership, runtime evidence, `Option` representation, class identity, GC rooting, Range semantics, Map item shape, or eager-vs-lazy policy;
- must re-ground the local checkout before edits and preserve unrelated modified/staged/untracked files;
- must use sparse discriminating tests in BUILD MODE rather than test after every edit;
- must classify unrelated failures `A/B/C/D` and continue on D immediately;
- must perform at most one serious corrective attempt for the same underlying nontrivial semantic failure, then STOP AND CONSULT if it persists;
- must not use a cursor compatibility bridge as the final production architecture even if it makes tests pass;
- must keep `LANG005.C6-CHECKPOINT.md` current only at durable interfaces, gates, consultations, and deferred failures;
- must produce `LANG005.C6.P5-walkthrough.md` and `LANG005.C6.P5-handoff.md` at completion;
- must not broaden testing merely because a narrow test passed.

Testing is evidence gathering, not ritual. The implementation model should spend most of its time coding the already-decided architecture, not rediscovering it.

---

# 2. Goal and checkpoint acceptance objective

P4 completed the generic proof law and staged canonical trait identities. P5 owns the **production migration** from cursor-based iteration to those canonical traits.

P5 is complete when all of the following are true:

1. production `for` no longer uses `iterate(_)` / `iteratorValue(_)` semantics or compiler selector sends;
2. every accepted `for` lane is backed by canonical semantic `Iterable` evidence and normalized associated projections;
3. built-in reusable sources and views have canonical `Iterable` conformances and create ordinary stateful iterator objects;
4. traversal objects/adapters have canonical `Iterator` conformances and sticky `next -> Option<Item>` behavior;
5. a production-ready default iterator/toolbox surface is live without requiring custom iterator authors to implement more than `Item + next` under ordinary conditions;
6. direct collection eager transforms and explicit iterator lazy transforms retain their accepted distinction;
7. Map direct iteration yields labeled `(key: K, value: V)` tuples; Range and String retain their ratified special semantics;
8. strict parallel iteration never truncates and raises `ZipLengthMismatchError` on mixed exhaustion;
9. positional expansion and every other live runtime/compiler consumer no longer rely on cursor selectors;
10. the runtime `Iterable` superclass identity and its `CoreClasses.iterable_class` plumbing are removed only after all consumers are migrated;
11. current documentation/tooling/comments describe the trait/stateful model rather than the retired cursor protocol;
12. final focused semantic/runtime/incremental/GC/disassembly gates pass, or remaining unrelated failures are truthfully classified and recorded.

P5 is a large migration/stabilization plan, **not** workspace release certification. P6 retains broader C6 closure/stress/certification unless the user explicitly changes checkpoint scope.

---

# 3. Repository grounding

Prepared against the remote default branch:

```text
repository: aureat/phalcom-lang
branch: main
revision: 06e37b6830cd9114d03f2a794d0b9a973f8cdc82
subject: feat(lang005): complete P4 iterator iterable law staging
predecessor: LANG005.C6.P4
```

The remote `main` revision was rechecked while producing this split header and still matches the P4-complete planning baseline. A local checkout was not executed by this planning turn; T0 must re-ground local HEAD/status before implementation.

Planning-grounded facts inherited from the three-pass requirements analysis and the first plan pass:

1. P4 is complete for the canonical `Iterator` / `Iterable` law and derived `Iterator => Iterable` evidence. Do not reimplement that theorem.
2. `phalcom-core/core/universe/src/collections/iterator_laws.ph` owns the canonical trait declaration identities.
3. `Statement::For` in `phalcom-semantic/src/checker/statement.rs` still derives element meaning through transitional cursor-oriented callable resolution rather than canonical `Iterable` proof.
4. `CallableAnalysis` has no dedicated published iteration-lane product yet.
5. `ModuleLoweringSemantics` already supports executable selected/requirement trait invocation specs and is the correct compiler-facing projection boundary.
6. compiler `compile_for` / `compile_for_lanes` still send `iterate` / `iteratorValue`; generalized loop setup currently interleaves source evaluation and cursor acquisition.
7. the existing compiler loop skeleton already correctly owns direct-jump break/continue, per-iteration capture closing, patterns, ordinals, and strict-lane control-flow structure; migrate traversal without redesigning loop control.
8. strict multi-lane mismatch currently raises `ArgumentError`; normative `ZipLengthMismatchError` has no production declaration at this revision.
9. production `collections/iterable.ph` still defines the runtime `Iterable` class, cursor-era `Iterator` class, `SourceIterator`, and cursor-based lazy adapters.
10. `CoreClasses.iterable_class` still parents `List`, `Map`, `Set`, `Tuple`, `Range`, and `Bytes`; bootstrap/invariants/primitives/native metadata reference it.
11. current Map direct cursor iteration yields keys, while the canonical P5 item shape is labeled `(key: K, value: V)`.
12. current Range traversal is forward integer +1, supports upper-unbounded ranges, and rejects missing lower / descending traversal; P5 preserves that subset.
13. existing lazy pipeline fixtures encode repeatability of the same cursor descriptor; canonical stateful iterators are one-shot and those oracles must change where they assert legacy repeatability.
14. ordinary `Object::Instance` fields are GC-traced, so iterator sources, callbacks, current inner iterators, and state can use ordinary fields without a new native heap arm.
15. runtime positional expansion still has a cursor-protocol probe and blocks final cursor deletion until migrated.
16. compiler boundedness metadata already knows existing lazy selector families and must be adapted rather than replaced with runtime proof.
17. repository examples such as Sheetcalc `RefRange` still inherit/use the legacy traversal surface and must migrate before retirement.
18. authoritative `docs/specs/iterator.md` describes the target model while multiple current/historical docs still describe cursor behavior and need final disposition.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

# 4. Required reads before implementation

Every implementer session must begin with the smallest sufficient read set. Do not repeatedly reload the entire plan set when working on one task.

## 4.1 Always read

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
LANG005.C6.P5-plan-00-shared-header.md
<active P5 part file>
```

## 4.2 Normative language reads

```text
docs/specs/README.md
docs/specs/iterator.md
docs/specs/generic-constraints.md
docs/specs/objects/traits.md
docs/specs/objects/impl.md
docs/specs/associated-types.md
```

## 4.3 Testing conventions

```text
phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
```

Each part adds only the exact source/tests required for its tasks. Avoid broad repository exploration unless live evidence disproves a premise or a named local dependency requires it.

---

# 5. Normative authority

Authority order for P5:

```text
1. explicit user-ratified decisions / accepted checkpoint amendments
2. current docs/specs normative language rules
3. LANG005.C6-CHECKPOINT.md durable invariants
4. completed P4 walkthrough/handoff
5. landed P1-P4 semantic/runtime architecture
6. this shared P5 header
7. the active P5 part file
8. older P5 monolithic planning draft
9. older implementation plans / cursor-era code
10. historical cursor ADRs/guides/tests where superseded
```

A stale test or current cursor implementation is not normative merely because it exists. Conversely, a plan cannot silently change accepted language behavior.

## 5.1 Normative P5 behavior

```text
Iterator:
    type Item
    next -> Option<Item>
    iterator -> self
    stateful, single-pass
    sticky exhaustion

Iterable:
    type Item
    type Iterator
    Iterator |= Iterator
    Iterator.Item == Item
    iterator -> fresh independent traversal for reusable sources

Derived law:
    I |= Iterator  =>  I |= Iterable
    <I as Iterable>::Iterator == I
    <I as Iterable>::Item == <I as Iterator>::Item

for:
    nominal/evidence-based participation
    source expression evaluated once
    all source expressions left-to-right before any iterator acquisition
    iterator acquisition once/lane left-to-right
    next once/lane/attempt
    direct same-function loop control
    pattern mismatch remains a language error
    parallel lanes use strict zip

Map:
    direct Item = (key: K, value: V)

Range:
    current forward integer +1 subset only

String:
    String itself is not Iterable
    explicit byte/codepoint views are iterable

Runtime:
    semantic proof is not runtime lookup
```

## 5.2 Superseded implementation behavior

The following are migration debt, not counter-authority:

- `iterate(cursor)` / `iteratorValue(cursor)`;
- cursor sentinels and cursor-token tuples;
- runtime `Iterable` inheritance;
- cursor-era `Iterator` descriptor classes;
- same-pipeline repeatability assumptions;
- structural runtime tests for cursor selectors;
- compiler string-selected iteration sends.

## 5.3 Deferred future proposals

P5 does not adopt or decide `rev`, `cycle`, `collect`, ordering families, `try_*`, chunks/windows, step-by, double-ended traversal, exact-size hints, or general iterator fusion. Those remain future design work unless a later accepted checkpoint amendment explicitly moves them into P5.

---

# 6. Takeover state and stable interfaces

| Concept | Current symbol/path | Canonical owner | P5 invariant |
|---|---|---|---|
| canonical Iterator trait | `collections/iterator_laws.ph::Iterator` | Universe + semantic trait system | retain declaration identity |
| canonical Iterable trait | `collections/iterator_laws.ph::Iterable` | Universe + semantic trait system | retain declaration identity |
| Iterator=>Iterable theorem | guarded `impl<I> Iterable for I where I |= Iterator` | generic proof engine | no compiler/VM special case |
| generic conformance evidence | `GenericConformanceEvidence` | semantic | exact/assumed/derived provenance remains meaningful |
| trait dispatch evidence | `TraitDispatchSite` | semantic expression analysis | compiler consumes selection; no selector rediscovery |
| callable semantic product | `CallableAnalysis` | semantic | Part I adds iteration-lane product |
| semantic fingerprint | `callable_body_product_fingerprint` | semantic DB | lane evidence must participate in result identity |
| lowering boundary | `ModuleLoweringSemantics` | core semantic projection | project semantic facts; never re-prove |
| selected trait execution | `TraitInvocationSpec` / `InvokeTraitSelected` | core compiler/VM | reuse |
| requirement execution | `TraitRequirementInvocationSpec` / `InvokeTraitRequirement` | core compiler/VM | reuse |
| loop control skeleton | `compiler/lib/loops.rs` | compiler | preserve direct-jump/fiber/capture semantics |
| Option representation | canonical ADT runtime path | semantic/compiler/VM ADT pipeline | distinguish `None` exhaustion from `Some(payload)` exactly |
| runtime Iterable class | `CoreClasses.iterable_class` + `collections/iterable.ph` | legacy runtime | retire only in Part III |
| cursor expansion probe | runtime/expansion owner | legacy integration path | migrate before cursor deletion |
| boundedness | `compiler/boundedness.rs` | compiler static metadata | update conservatively in Part II |
| GC roots | ordinary `Object::Instance` slots | heap tracer | no iterator-specific native tracing arm |

Do not re-prove P4 laws. P5 starts from them.

---

# 7. Global architecture

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
        ↓
compiler direct-jump loop
        ↓
existing selected/requirement trait invocation machinery
        ↓
VM executes finite preselected calls only
```

The exact private type/helper names may adapt to live repository style. The information boundary may not.

## 7.2 Canonical semantic product target

The intended semantic shape is equivalent to:

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
```

Only a successfully proven lane may publish executable iteration evidence. Invalid, blocked, ambiguous, dynamic, cancelled, or budget-exceeded semantic outcomes must retain their ordinary semantic status; do not publish a fake fallback selection merely to make lowering possible.

The lane product must be part of `CallableAnalysis` publication/fingerprinting/dependency behavior so incremental compilation invalidates exactly when iteration meaning changes.

## 7.3 Canonical lowering product target

Implicit `for` calls have no source method-call AST node, so lowering needs an explicit executable lane spec rather than fabricated ordinary call sites. The intended shape is equivalent to:

```rust
pub enum IterationInvocationSpec {
    Selected(TraitInvocationSpec),
    Requirement(TraitRequirementInvocationSpec),
}

pub struct ForIterationLaneLoweringSpec {
    pub item_type: TypeId,       // or repository-native executable type recipe
    pub iterator_type: TypeId,
    pub iterator: IterationInvocationSpec,
    pub next: IterationInvocationSpec,
}
```

Use a dedicated lane-site key/range/kind if required. Reuse existing method preparation, runtime environment, and selected/requirement invocation machinery. Do not invent a loop-only dispatch ABI.

## 7.4 Compiler loop architecture

Single lane:

```text
evaluate source -> source_slot
invoke semantically selected Iterable.iterator -> iterator_slot

loop:
    invoke semantically selected Iterator.next -> option_slot
    if option_slot is None -> exit
    extract Some payload through canonical Option/ADT path
    bind pattern/index
    body
continue_target:
    close captured loop bindings
    jump loop
exit:
```

Do not route generic `for` through `Option.unwrapOr`, `isSome`, collection-specific indexing, `size`, cursor selectors, or `each` closures.

Generalized N-lane loop:

```text
phase A: evaluate source[0..N] left-to-right into source slots
phase B: invoke iterator[0..N] left-to-right into iterator slots

attempt:
    invoke next exactly once on every iterator -> option[0..N]
    if all None: exit
    if mixed None/Some: raise ZipLengthMismatchError
    if all Some:
        extract every payload
        pattern-bind all lanes
        assign shared ordinal to every `at` binding
        body
        close captured pattern/index bindings
        ordinal += 1
        jump attempt
```

Strict mismatch detection is allowed to advance live lanes on the failed attempted step because it must inspect all lanes to determine mismatch.

## 7.5 Source/runtime architecture

Preserve the canonical trait identities in `iterator_laws.ph`. Extend the traits/defaults around those identities rather than declaring replacement traits.

Recommended decomposition, mechanically adaptable to Universe module topology:

```text
collections/iterator_laws.ph
    canonical Iterator / Iterable declarations
    default bodies where dependency ordering permits
    P4 derived law unchanged in meaning

collections/iterator_adapters.ph        # likely new
    ordinary stateful adapter classes

collections/iterator_impls.ph           # optional if useful
    built-in/source/adapter conformances

representation source files
    source-specific iterator classes where locality is clearer
```

Until Part III removes the legacy runtime class named `Iterable`, avoid unqualified class/trait name collisions through module qualification or existing import alias mechanisms. If live module rules make canonical trait identity impossible to reference without declaration duplication, STOP AND CONSULT.

## 7.6 Production API ownership

The final coherent ownership split is FIXED.

`Iterator` owns lazy state-transforming adapters:

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

`Iterable` owns reusable traversal conveniences and non-conflicting consumers/terminals:

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

Concrete reusable collections retain selector-conflicting direct eager transforms as thin inherent wrappers:

```text
map
map(indexed:)
filter
flatMap
```

Those wrappers acquire a fresh canonical iterator and delegate; they do not duplicate traversal algorithms. This avoids requiring a new trait-default precedence/specialization rule while preserving accepted eager collection semantics and lazy explicit iterator semantics.

A custom iterator author should ordinarily implement only:

```phalcom
impl Iterator for MyIterator {
  type Item = T

  next -> Option<T> {
    ...
  }
}
```

and receive `Iterable` participation and the default toolbox through the derived law.

---

# 8. Ownership boundaries

## 8.1 P5 owns

- canonical `for` lane semantic resolution;
- semantic-to-lowering projection for implicit `iterator` / `next` calls;
- production `for` compiler cutover;
- stateful built-in iterator objects and canonical conformances;
- canonical lazy adapter/default surface;
- terminal/default iteration algorithms required by the final requirements analysis;
- Map labeled pair iteration migration;
- Range canonical stateful traversal migration within the existing subset;
- String byte/codepoint view integration;
- strict pairwise/parallel zip behavior and `ZipLengthMismatchError` surface;
- boundedness updates needed for the canonical selector/surface model;
- positional expansion migration away from cursor probing;
- repository legacy traversal consumer migration;
- runtime `Iterable` class/superclass retirement;
- final current-doc/tooling migration and P5 implementation records.

## 8.2 P5 does not own

- redesigning derived conformance or generic proof architecture;
- parser/AST syntax changes for `for`;
- labeled break/continue;
- new Range step/descending/Progression semantics;
- general collection ordering/sorting policy;
- a new runtime trait-object system;
- new VM-level proof/search machinery;
- general optimizer/fusion work;
- broad generic-call/runtime redesign unless proven to be an unavoidable P5 blocker and explicitly approved;
- C6-wide release certification beyond focused P5 stabilization.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| `Source |= Iterable` | semantic generic proof | lane analysis, diagnostics, lowering | compiler/VM selector probing |
| `<Source as Iterable>::Item` | semantic associated normalization | pattern typing, lowering metadata | compiler inference |
| `<Source as Iterable>::Iterator` | semantic associated normalization | dispatch selection, lowering | runtime inspection |
| selected `iterator` / `next` requirement | semantic trait dispatch | lowering/compiler | string lookup in `for` |
| loop evaluation order | language spec + compiler lowering | runtime observation/tests | source-specific workarounds |
| iterator runtime state | ordinary source object fields | adapter `next`, GC | cursor tokens stored by loop |
| strict zip mismatch | language semantics | compiler/adapter/runtime error surface | shortest-source truncation |
| Map iteration item shape | canonical iteration spec | Map iterator/views/toMap/tests | legacy key-only direct iteration |
| boundedness classification | compiler static metadata | diagnostics/eager exhaustors | runtime size/proof search |
| public `Iterator`/`Iterable` identity | canonical trait declarations | source imports/reflection/semantics | replacement declarations |

---

# 9. Global invariants

`INV-01` — P4's canonical `Iterator` and `Iterable` declaration identities remain stable.

`INV-02` — Conformance proof and associated projection normalization occur only in semantic analysis.

`INV-03` — The compiler receives preselected `iterator` / `next` invocation evidence; no `for` path performs trait discovery by selector string.

`INV-04` — The VM performs no conformance proof, associated normalization, or cursor-shape proof.

`INV-05` — Each lane source expression is evaluated exactly once per loop execution, left-to-right.

`INV-06` — All lane source expressions finish before any lane iterator is acquired; iterator acquisition is then left-to-right exactly once per lane.

`INV-07` — Each iterator is advanced exactly once per attempted loop step by the loop itself; adapter-internal extra reads are local adapter semantics.

`INV-08` — `None` from `next` means clean exhaustion only; ordinary errors propagate unchanged.

`INV-09` — Exhaustion is sticky for every iterator/adaptor.

`INV-10` — `Some(item)` payload extraction preserves `Some(None)` and nested Option payloads; exhaustion is never inferred from payload value.

`INV-11` — break/continue/return/fiber behavior remains the direct-jump same-function architecture; no `each`/closure desugaring.

`INV-12` — captured loop bindings are closed before rebinding/next iteration so per-iteration capture identity remains correct.

`INV-13` — strict zip is all-Some / all-None / error-on-mixed; never shortest-source truncation.

`INV-14` — custom iterators normally require only `Item + next` and derive `Iterable` plus defaults.

`INV-15` — reusable sources create independent iterator objects; iterator/adaptor values are one-shot and aliasable.

`INV-16` — direct collection eager transforms and explicit iterator lazy transforms remain observably distinct.

`INV-17` — Map iteration yields labeled `(key: K, value: V)` tuples in encounter order; `Map.contains(key)` remains key membership.

`INV-18` — Range remains forward +1 with current missing-lower/descending failures and supports upper-unbounded lazy traversal.

`INV-19` — String remains non-Iterable; only explicit byte/codepoint views are iterable.

`INV-20` — ordinary object fields root iterator/adaptor sources, callbacks, inner iterators, and buffered state across GC.

`INV-21` — no production cursor selector/token/class/superclass remains at P5 completion.

`INV-22` — runtime `Iterable` class identity is removed only after all semantic/library/compiler/expansion consumers are migrated.

`INV-23` — boundedness remains conservative: reject only proven unbounded full exhaustion; Unknown stays legal.

`INV-24` — cursor removal from positional expansion does not add VM trait proof or structural `next` probing.

`INV-25` — no per-combinator native primitive explosion; source-authored defaults remain the correctness baseline.

`INV-26` — incremental callable fingerprints change when iteration evidence changes and remain stable for unrelated body edits.

Part files must cite these invariant IDs in their task acceptance/test sections.

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
new native iterator heap arms
per-adapter native primitives
workspace-wide formatting cleanup
historical documentation rewriting beyond clear supersession/context
```

If implementing an in-scope method appears to require one of these, treat that as evidence of an architectural blocker rather than permission to expand scope.

---

# 11. Part decomposition and cross-part contracts

The split must reduce context load **without** splitting a single ownership decision across multiple files. Each part therefore has a concrete input/output contract.

## 11.1 Part I — semantic authority and `for` cutover

File: `LANG005.C6.P5-plan-01-semantic-and-for-cutover.md`

Stable scope:

```text
T0 — re-ground checkout, baseline, migration census
T1 — publish canonical for-lane semantic resolution
T2 — project lane evidence into executable lowering specs
T3 — establish List as first stateful source/iterator vertical
G1 — semantic/lowering/List vertical
T4 — cut over single-lane for
T5 — cut over generalized patterns/ordinals/parallel lanes/strict zip
G2 — canonical for gate
```

Part I must produce the interfaces that later parts consume:

- published per-lane semantic iteration resolution in `CallableAnalysis` or repository-native equivalent;
- fingerprint/dependency participation for that product;
- executable per-lane lowering spec using existing selected/requirement trait invocation infrastructure;
- reusable compiler helper(s) for implicit selected/requirement trait calls;
- one working canonical reusable source (`List`) plus stateful `ListIterator` proving the vertical;
- production single- and multi-lane `for` entirely driven by canonical iterator objects and `next`;
- strict mismatch using `ZipLengthMismatchError` or the approved ordinary source error surface;
- preserved direct-jump control-flow, pattern, ordinal, capture, and fiber semantics.

Part I must **not** delete the legacy runtime `Iterable` class or cursor protocol globally; Parts II/III still need migration runway.

## 11.2 Part II — built-in traversal and production API surface

File: `LANG005.C6.P5-plan-02-builtins-and-iterator-surface.md`

Stable scope:

```text
T6 — Set, Tuple, Bytes, String traversal views
T7 — Map labeled entry tuples and views
T8 — Range stateful forward iterator
G3 — built-in source migration
T9 — canonical lazy iterator adapters/defaults
T10 — Iterable terminals and eager collection forwarding
G4 — iterator surface gate
T11 — canonical boundedness update
```

Part II consumes Part I's semantic/lowering/loop contracts and must produce:

- canonical `Iterable` implementations for all P5 core sources/views;
- canonical `Iterator` implementations for all stateful traversal objects/adapters;
- Map direct traversal over labeled pair tuples and migrated entry conversions;
- Range stateful traversal preserving its ratified subset;
- String byte/codepoint views integrated without making String itself Iterable;
- lazy adapter surface owned by `Iterator`;
- terminal/reusable-source surface owned by `Iterable`;
- thin eager collection wrappers only where selector conflict requires them;
- one-shot aliasing/sticky exhaustion behavior across adapters;
- boundedness metadata updated for canonical `.iterator` / `.iter` and new lazy stages.

Part II still does **not** remove the global cursor runtime/class substrate; it must leave Part III with all known production source consumers already migrated.

## 11.3 Part III — cross-cutting consumers, retirement, and stabilization

File: `LANG005.C6.P5-plan-03-retirement-and-stabilization.md`

Stable scope:

```text
T12 — remove positional-expansion cursor probing through its owning layer
T13 — migrate repository legacy subclasses/use sites and close cursor consumers
T14 — retire runtime Iterable/Iterator classes and cut public names to traits
G5 — retirement gate
T15 — current docs/tooling, final census, stabilization, records
G6 — final focused P5 stabilization
```

Part III consumes the complete canonical traversal surface and must produce:

- no VM/runtime structural cursor proof for positional expansion;
- all live repository source consumers migrated off legacy inheritance/selectors;
- `CoreClasses.iterable_class` and related bootstrap/invariant/primitive/native-metadata plumbing removed;
- canonical public names `Iterator` / `Iterable` resolving to traits, not legacy classes;
- no live production `iterate` / `iteratorValue` protocol remaining;
- current docs/tooling/comments updated or explicitly marked historical/superseded;
- final cursor/name census clean outside intentionally historical material;
- focused semantic/core/runtime/incremental/GC/disassembly stabilization evidence;
- checkpoint, walkthrough, and handoff complete.

## 11.4 Cross-part compatibility rule

A part may refactor a helper produced by an earlier part only when:

1. behavior and ownership are unchanged;
2. downstream interfaces remain mechanically equivalent; and
3. the active part records the mechanical change in its task notes/checkpoint if externally relevant.

If a later part needs to materially change an earlier part's semantic product, public surface, runtime contract, or acceptance semantics, STOP AND CONSULT and amend the plan set rather than silently rewriting history.

---

# 12. Expected impact map

This is a planning map, not permission to touch every path. Each part should touch only the paths its tasks require.

## 12.1 Part I expected source areas

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/db/fingerprint.rs
// optional focused checker/iteration.rs split

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/loops.rs

phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/list.ph
// source file/module for ListIterator if not colocated
phalcom-core/core/universe/src/errors/*            # ZipLengthMismatchError if landed here
```

## 12.2 Part II expected source areas

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/iterator_adapters.ph   # likely new
phalcom-core/core/universe/src/collections/iterator_impls.ph      # optional
phalcom-core/core/universe/src/collections/list.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/collections/set.ph
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/range.ph
phalcom-core/core/universe/src/collections/bytes.ph
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-modules/src/builtin.rs                    # only if new Universe modules are added
phalcom-core/src/compiler/boundedness.rs
```

## 12.3 Part III expected source areas

```text
phalcom-core/src/vm/dispatch.rs or actual positional-expansion owner
phalcom-core/src/compiler / lowering expansion paths as discovered

examples/sheetcalc/src/grid/ref-range.ph
other final live cursor consumers found by census

phalcom-core/core/universe/src/collections/iterable.ph            # delete/retire
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-native-meta/src/universe.rs
phalcom-modules/src/builtin.rs

current docs/guides/wiki/LSP/help text found by final census
LANG005.C6-CHECKPOINT.md
LANG005.C6.P5-walkthrough.md
LANG005.C6.P5-handoff.md
```

## 12.4 Expected tests across the plan set

```text
phalcom-semantic/tests/semantic/capabilities/iteration.rs          # likely new
phalcom-semantic/tests/semantic/incremental/iteration.rs           # likely new
phalcom-semantic/tests/semantic/capabilities/mod.rs
phalcom-semantic/tests/semantic/incremental/mod.rs

phalcom-core/tests/core/language/iteration_stateful.rs             # likely new
phalcom-core/tests/core/language/mod.rs
phalcom-core/tests/core/memory/* or existing focused GC location

phalcom-core/tests/fixtures/language/iteration/*
phalcom-core/tests/fixtures/language/iterator/*
phalcom-core/tests/fixtures/language/sequence/*
phalcom-core/tests/fixtures/language/collections/*
phalcom-core/tests/fixtures/language/strings/*
phalcom-core/tests/core/language/corpus.rs
```

Use existing test registration conventions; do not create parallel integration binaries unnecessarily.

## 12.5 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary. STOP AND CONSULT before entering a materially different subsystem if entry into that subsystem changes architecture, authority, identity, lifetime, runtime representation, or the plan's cross-part contracts.

---

# 13. Implementer decision authority

## 13.1 FIXED

The implementer must not change:

- P4 trait declaration identity or derived law;
- semantic ownership of conformance proof and associated projections;
- semantic publication of `for` lane meaning before compiler lowering;
- reuse of existing selected/requirement trait invocation architecture rather than a loop-only runtime proof system;
- two-phase parallel preheader order;
- direct-jump loop architecture;
- stateful one-shot iterator lifecycle;
- sticky exhaustion;
- exact `Some` payload versus `None` exhaustion distinction;
- Map labeled tuple item shape;
- Range semantic subset;
- String non-Iterable status;
- eager collection versus lazy iterator distinction;
- default-method ownership split;
- ordinary traced-field GC model;
- conservative boundedness rule;
- final cursor/runtime-class retirement requirement;
- no-runtime-proof boundary.

## 13.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- helper/private type names while preserving the same information contract;
- whether checker logic remains in `statement.rs` or moves to a focused module;
- exact adapter/source file decomposition;
- whether built-in conformance impls are centralized or colocated, provided only the canonical trait identity is referenced;
- exact scratch-local names and compiler helper decomposition;
- exact fixture filenames and test helper names;
- equivalent repository APIs caused by mechanical drift;
- ordinary Rust imports/borrow/type cleanup;
- module placement when Universe dependency ordering requires an equivalent arrangement.

## 13.3 VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| local HEAD contains the P4-complete law | git + `iterator_laws.ph` | material drift => STOP |
| legacy positive iteration baseline is unchanged or classifiable | focused corpus baseline | record changed baseline; canonical-path implication requires diagnosis |
| canonical trait declaration can be resolved by stable semantic identity | resolver/trait tables | mechanical helper okay; new identity architecture => STOP |
| `TraitDispatchSite` can represent exact/assumed/derived iterator/next calls | `trait_dispatch.rs`, P4 evidence tests | new proof/evidence kind => STOP |
| existing selected/requirement lowering can serve implicit loop calls | `semantic_lowering.rs`, compiler trait helpers | helper refactor okay; new runtime ABI => STOP |
| canonical Option path supports safe Some payload extraction after None discrimination | ADT lowering/runtime | use existing proven extraction; broad Option redesign => STOP |
| ordinary source object fields are GC traced | heap tracer | nonlocal rooting design => STOP |
| source module topology can reference canonical traits without duplicating declaration identity | builtins/package loader/imports | relocate mechanically; identity change => STOP |
| `ZipLengthMismatchError` can be ordinary source `Error` subclass | errors package conventions | new runtime error ABI => STOP |
| boundedness can be updated as static metadata without runtime proof | boundedness compiler | runtime semantic proof proposal => STOP |
| positional expansion has a semantic/compiler owner that can consume canonical traversal facts | expansion lowering/compiler/runtime path | if dynamic semantics force runtime proof/search => STOP |
| nominal `Entry` has no newer independent normative identity requirement | final consumer/spec census | conflicting authority => STOP |

---

# 14. Global STOP / CONSULT protocol

STOP editing the affected issue and build a compact consultation packet when any of the following occurs:

1. planned architecture appears impossible or materially incorrect;
2. a core plan premise is disproved by live repository state;
3. ownership, identity, lifetime, representation, semantic authority, or cross-layer contract must change;
4. an interface relied upon by a later part must materially change;
5. expected semantic/test behavior appears to conflict with accepted language rules;
6. the same nontrivial semantic failure survives one serious corrective attempt;
7. two plausible fixes have materially different architectural consequences;
8. implementation must enter an unexpected subsystem for more than a mechanical dependency;
9. a workaround/special-case architecture is emerging;
10. a duplicate source of canonical semantic truth/state would be introduced;
11. correctness in GC, generic reification, incremental semantics, VM control semantics, exact enum identity, or fiber/liveness behavior lacks an obvious local answer;
12. spec/plan/code/test authorities materially conflict;
13. tests can pass only by weakening an invariant;
14. scope becomes materially more cross-cutting than this split anticipates;
15. correct loop migration appears to require VM conformance proof or runtime associated-type normalization;
16. non-conflicting `Iterable` defaults cannot work through existing derived evidence without adding broad trait precedence/specialization;
17. adapters require native heap arms, trait-owned storage, or untraced pointers;
18. `ZipLengthMismatchError` requires a new runtime ABI rather than ordinary language error declaration;
19. deleting the runtime `Iterable` identity reveals nonlocal representation/layout semantics;
20. positional expansion cannot remove cursor probing without runtime structural proof or a material change to dynamic expansion semantics;
21. a cursor compatibility bridge is becoming the final protocol rather than temporary migration scaffolding.

A triggered consultation cannot be self-waived.

## 14.1 Consultation packet

Collect cheaply before escalating:

```text
Task / gate:
Observed behavior:
Smallest reproducer:
Expected invariant / requirement:
Verified repository facts:
Relevant call/data-flow:
One attempted correction, if allowed:
Why it failed / what remains ambiguous:
Two plausible options, if applicable:
Decision question for adviser:
```

Ask for a decision, not a fresh repository exploration.

---

# 15. Debugging budget

## 15.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress:

```text
inspect
identify concrete cause
make one coherent correction
rerun the smallest discriminating command
```

Examples: imports, private helper drift, Rust type mismatch, borrow cleanup, wrong test selector, module path adjustment.

## 15.2 Semantic/behavioral failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic problem. If it persists, STOP AND CONSULT.

## 15.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless relevant code/state changed.

---

# 16. Testing-surface design and coverage ownership

The final requirements analysis owns the complete `P5-CV-01..P5-CV-115` design ledger. The split plan files must import their exact relevant rows; this header defines only the plan-wide dimensions and primary ownership.

Applicable correctness dimensions:

| Dimension | Why applicable | Representative invariants |
|---|---|---|
| canonical success | every built-in/custom traversal must work | INV-01..10 |
| empty/one/many boundaries | iteration/adapters are cardinality-sensitive | INV-07..09 |
| nominal negative behavior | method coincidence must not imply conformance | INV-02..04 |
| lifecycle/repetition | reusable source versus one-shot iterator | INV-14..16 |
| ordering | source/iterator acquisition and Map encounter order | INV-05/06/17 |
| aliasing | adapters retain the same upstream iterator object | INV-15 |
| composition | generic/conditional/cross-module evidence + adapters | INV-02/14 |
| exact identity | exact enum/generic evidence must survive lowering | INV-01/02/10 |
| incremental equivalence | lane selections are semantic products | INV-26 |
| GC/lifetime | source/callback/current-inner state survives collection | INV-20 |
| diagnostics | strict zip, invalid conformance, pattern errors | INV-13 |
| runtime boundary | no proof or hot-path structural selector lookup | INV-03/04/25 |
| fiber/control flow | direct-jump loop remains suspendable/correct | INV-11/12 |
| unboundedness | finite lazy prefix remains legal, full proven exhaustion rejected | INV-18/23 |
| retirement | no hidden cursor consumer survives final deletion | INV-21/22/24 |

Primary coverage partition:

| Part | Primary coverage responsibility |
|---|---|
| Part I | semantic/lowering evidence, List vertical, single/generalized `for`, patterns/ordinals/control flow, two-phase ordering, strict parallel zip, incremental lane evidence |
| Part II | all built-in source/view semantics, Map/Range/String specifics, iterator lifecycle/aliasing/laziness, adapter/default toolbox, terminals/conversions, GC retention, boundedness |
| Part III | positional expansion, legacy repository use sites, public-name/runtime-class retirement, final cursor census, documentation/tooling, end-to-end regression and final focused stabilization |

Cross-part repetition is intentional where later gates prove integration; it does not authorize duplicating tests unnecessarily. Each part must map its task tests to exact `P5-CV-*` rows from the requirements analysis.

---

# 17. Verification execution budget

## 17.1 Modes

### BUILD MODE

Default during implementation tasks. Run the smallest tests that discriminate the current task. Do not broaden after PASS merely for reassurance.

### STABILIZE MODE

Use only at named gates G1–G6 and final T15 stabilization. Run focused regressions and directly affected suites; classify failures before editing.

### CERTIFY MODE

Not required by this plan set unless the user/checkpoint explicitly promotes P5. Workspace/release certification remains outside ordinary P5 BUILD/STABILIZE work.

## 17.2 Verification ladder

```text
V0 — exact reproducer/new regression
V1 — directly affected feature tests
V2 — owning subsystem/module suite
V3 — adjacent cross-layer integration
V4 — broad affected crate/package
V5 — workspace/release
```

Use V0/V1 routinely, V2 at coherent boundaries, V3 when a cross-layer gate requires it, V4 only near focused plan completion if warranted, and V5 only under explicit certification scope.

## 17.3 Mandatory during BUILD

Only the active part's discriminating tests, typically:

- exact semantic iteration tests when semantic products change;
- exact lowering/compiler tests when executable lane specs or loop generation change;
- one/few source-language fixtures for a source/adapter changed;
- exact boundedness regression when boundedness changes;
- exact expansion regression when expansion changes;
- exact census commands during retirement tasks.

## 17.4 Mandatory during named stabilization gates

The part files specify the exact commands. Across the full plan set, the expected focused surfaces include:

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
relevant collections/list/bytes/strings lanes
affected-crate cargo check
final git diff --check
final cursor/public-name census
```

Do not automatically run all of them after every part task.

## 17.5 Only if evidence demands

- full `phalcom-semantic --test semantic`;
- full `phalcom-core --test core`;
- full language corpus;
- broader module/REPL/LSP/CLI/concurrency suites when touched behavior truly crosses those surfaces.

## 17.6 Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace
workspace-wide formatting as a release gate
full language corpus after every task
unrelated LSP/CLI/concurrency suites
```

Reason: repeated broad gates would convert a large cross-layer migration into test churn and obscure causal feedback. Focused stabilization occurs at G1–G6; broad certification remains a checkpoint/release concern.

---

# 18. Baseline and unrelated-failure policy

Classify unexpected failures:

```text
A — definitely caused by P5
B — probably caused by P5
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks active acceptance.
- Do not weaken/skip assertions to make broad gates green.

Known planning baseline `BL-P5-01`:

```text
surface: language-corpus::iteration
historical failure: Option.unwrapOr / generic call-entry type recipe could not be instantiated
P4 classification: reproduced at clean predecessor; not caused by P4
```

T0 in Part I must reproduce or reclassify it against the local checkout. Canonical `for` must not intentionally depend on `unwrapOr`. If cursor removal makes the old failure disappear, record it as retired-baseline collateral rather than pretending P5 proved a general generic-call fix. If the same underlying runtime generic-call defect appears on canonical `next`, perform one focused diagnosis; broad generic-call/runtime redesign requires consultation.

---

# 19. Cross-part gates and execution records

The gates are deliberate architectural checkpoints, not ceremonial test bundles.

| Gate | Owned by | Proves | Next-part dependency |
|---|---|---|---|
| G1 | Part I | semantic lane product + lowering projection + List canonical vertical compose | required before compiler-wide `for` cutover continues |
| G2 | Part I | all production `for` forms are canonical/evidence-driven with preserved control flow | required before broad built-in/API migration |
| G3 | Part II | built-in sources/views all use canonical stateful traversal semantics | required before API surface retirement assumptions |
| G4 | Part II | iterator/default surface + boundedness are coherent and one-shot semantics are proven | required before destructive retirement |
| G5 | Part III | no live runtime/public dependency requires legacy Iterable/cursor identities | required before final stabilization |
| G6 | Part III | focused P5 acceptance, final census, records, and residual-risk classification | P5 completion evidence |

At every gate:

1. run only the commands named by the active part;
2. read the full output;
3. classify failures before editing;
4. do not broaden after PASS unless the gate explicitly requires a next rung;
5. update the checkpoint only with durable results.

No part may claim a later gate based on tests from an earlier repository state.

---

# 20. Checkpoint and implementation-record obligations

`LANG005.C6-CHECKPOINT.md` is the checkpoint-wide durable state authority. Do not create a competing implementation-state file.

## 20.1 At P5 start / Part I T0

- mark P5 active;
- record starting revision/local dirty-state policy;
- record baseline classification including `BL-P5-01`;
- record the split-plan file set if useful for later sessions.

## 20.2 During P5

Update the checkpoint only for:

- a durable new semantic/runtime interface;
- a named verification gate;
- an approved consultation/amendment;
- a deferred failure that later sessions must know;
- a material baseline reclassification.

Do not turn the checkpoint into a per-edit diary.

## 20.3 At P5 completion

Record:

- plan completion state;
- final verification classification;
- final stable interfaces/invariants;
- consultations/amendments;
- deferred failures/residual risks;
- next C6 action.

Create:

```text
LANG005.C6.P5-walkthrough.md
LANG005.C6.P5-handoff.md
```

The walkthrough must contain final architecture, important code changes, deviations, consultations, tests added, tests actually run, tests deferred, verification classification, and residual risks.

The handoff must contain current state/revision, inherited stable interfaces, invariants, next objective, must-read files, deferred failures, first recommended commands, and explicit things not to redesign/re-explore.

---

# 21. Completion truth model

Do not conflate:

```text
source written
≠ tests added
≠ targeted tests passed
≠ affected focused suite passed
≠ P5 accepted
≠ C6 checkpoint certified
≠ release certified
```

Truthful metadata examples:

- task implemented but not tested: `IMPLEMENTED_UNVERIFIED`;
- exact task tests passed: `FOCUSED_TESTED`;
- part terminal gate passed: `PART_STABILIZED`;
- G6 passed with final records: `P5_COMPLETE_FOCUSED_VERIFIED`;
- workspace/release certification: only if separately executed and evidenced.

---

# 22. Plan amendments and drift policy

Three levels:

### Mechanical drift

Luna adapts locally and records only if useful for future tasks.

### Architectural clarification

Consultation decision is recorded in checkpoint/walkthrough; update the affected part if necessary. Shared-header change is required only if the clarification changes a global contract/invariant.

### Material architecture or acceptance change

The principal architect amends this header and every affected part, or creates an explicit follow-up plan. Luna must not silently redefine the plan.

If repository `main` changes before a future part is generated, that part must recheck the revision and document whether changes are mechanical or architectural. This header's revision is a planning baseline, not a command to reset newer user work.

---

# 23. Shared self-review requirements for every part

Before delivering each part file, the planner must verify:

### Architecture

- every important fact/state has exactly one canonical owner;
- no compiler/runtime consumer reconstructs semantic proof;
- identities and lifetimes are explicit;
- failure/exhaustion semantics are explicit;
- cross-part interfaces match this header.

### Luna executability

- tasks can be executed without inventing architecture;
- fixed/flexible/verify-first decisions are explicit;
- task-local STOP/CONSULT triggers are objective;
- likely failure loops are bounded;
- no task implicitly requires reading another part for missing details.

### Testing

- every owned requirements/coverage row maps to a task/gate;
- tests are tied to invariants;
- BUILD execution breadth is intentionally narrow;
- broad tests are explicitly deferred where unnecessary;
- baseline failures are classified, not accidentally repaired.

### Documentation lifecycle

- checkpoint update points exist;
- final part owns walkthrough/handoff;
- T/G terminology remains stable;
- no competing checkpoint state file is introduced.

### Artifact quality

- no unresolved placeholder markers or shorthand that delegates missing design to the implementer;
- all named types/helpers are defined or explicitly marked mechanically flexible equivalents;
- Markdown fences/headings are structurally valid;
- task dependency order is valid;
- no requirement is silently dropped because of the split.

---

# 24. Header analysis result and next file

This shared header is the first file of the restructured four-file planning cycle. It deliberately contains no implementation task bodies; those belong to the three independently generated parts so each can receive a fresh repository-focused analysis pass.

The next planning turn should produce:

```text
LANG005.C6.P5-plan-01-semantic-and-for-cutover.md
```

That pass must re-ground the Part-I source/test seams and produce full Luna patch-grade task contracts for T0–T5 and G1–G2 without reopening the global decisions in this header unless live repository evidence contradicts them.
