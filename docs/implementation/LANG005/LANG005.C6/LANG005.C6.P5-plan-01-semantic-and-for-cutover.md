---
id: LANG005.C6.P5.PART-I
parent_plan: LANG005.C6.P5
shared_header: LANG005.C6.P5-plan-00-shared-header.md
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan-part
part: 1-of-3
planning_cycle: restructured-pass-2
file_pass: 2-of-4
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: 06e37b6830cd9114d03f2a794d0b9a973f8cdc82
planning_revision_subject: "feat(lang005): complete P4 iterator iterable law staging"
requirements_analysis: LANG005.C6.P5-requirements-analysis.md
predecessor: LANG005.C6.P4
next_part: LANG005.C6.P5-plan-02-builtins-and-iterator-surface.md
executor_profile: GPT-5.6-Luna-High-or-Extra-High
---

# LANG005.C6.P5 Part I — Canonical Iteration Semantics and Production `for` Cutover

> **For Luna / agentic implementers:** read `LANG005.C6.P5-plan-00-shared-header.md` before this file. The header owns all plan-wide architecture, invariants, scope exclusions, debugging budgets, verification doctrine, baseline policy, consultation rules, and checkpoint-record obligations. This file specializes that contract for T0–T5 and G1–G2 only.
>
> **Do not execute Part II or Part III from this file.** Part I deliberately leaves legacy cursor collection methods, the runtime `Iterable` superclass, cursor adapters, and non-List built-in sources in place where needed for compatibility. Their retirement is sequenced later.
>
> **Required execution sub-skill:** use the repository's task-by-task implementation workflow (`subagent-driven-development` or `executing-plans`, where available). Treat each T-task below as one reviewer-sized patch unit, not as a collection of edit-sized rituals.

**Goal:** Establish one canonical semantic and executable evidence path for `for`, prove it with a first stateful `ListIterator` vertical, then migrate every currently supported `for` shape from cursor selectors to preselected `Iterable.iterator` / `Iterator.next` execution while preserving direct-jump control flow, ordering, patterns, captures, strict zip, errors, and generic evidence.

**Part-I terminal condition:** after G2, production `for` no longer sends or reasons about `iterate(_)` / `iteratorValue(_)`; a semantically accepted lane is driven exclusively by canonical P4 trait evidence and `next -> Option<Item>`. Legacy cursor APIs may still exist elsewhere until Parts II/III.

---

# 0. Part-I authority and relationship to the shared header

This file is not a second architecture document. It is the patch-grade execution decomposition for the first replacement vertical defined by the shared header.

The authority order for this part is:

```text
normative language specs + accepted user rulings
    ↓
LANG005.C6.P5 requirements analysis, final pass 3/3
    ↓
LANG005.C6.P5 shared header
    ↓
this Part-I file
    ↓
approved consultation/amendment recorded in LANG005.C6-CHECKPOINT.md
```

If current source has mechanically drifted, adapt the mechanics. If a fixed premise below is false, do not improvise around it; invoke the shared STOP/CONSULT protocol.

Part I owns stable task IDs:

```text
T0  local preflight / baseline / migration census
T1  canonical for-lane semantic product
T2  executable lane lowering projection
T3  first production source/iterator vertical: List
G1  semantic + lowering + List vertical gate
T4  single-lane for cutover
T5  generalized/pattern/ordinal/parallel/strict-zip for cutover
G2  canonical production-for gate
```

No later part may renumber these IDs.

---

# 1. Part-I repository re-grounding performed during this planning pass

The remote repository was re-inspected specifically for the semantic/compiler cutover. `main` remains at:

```text
06e37b6830cd9114d03f2a794d0b9a973f8cdc82
feat(lang005): complete P4 iterator iterable law staging
```

The following live facts materially refine the older monolithic plan.

## 1.1 Canonical P4 source law is unchanged

`phalcom-core/core/universe/src/collections/iterator_laws.ph` still defines exactly one canonical pair of traits:

```phalcom
trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }
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

impl<I> Iterable for I
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
  type Iterator = I

  iterator { self }
}
```

Part I consumes these identities. It does not recreate, rename, or special-case the theorem.

## 1.2 Semantic `for` is still a compatibility path

`phalcom-semantic/src/checker/statement.rs` still handles `Statement::For` by:

1. analyzing each lane source expression;
2. building a `CallPremise`;
3. calling `resolve_iteration_element_application(...)` for known source types;
4. binding the loop pattern from that compatibility result;
5. feeding the resulting lane fact into the already-existing loop fixed-point / flow machinery.

That helper is cursor-era semantic authority. T1 replaces its role for production `for` without redesigning the flow solver.

## 1.3 `CallableAnalysis` has no lane product yet

`phalcom-semantic/src/checker/analysis.rs` currently publishes expression products, bindings, associated/member/family/match products, flow graph, exits, diagnostics, dependencies, semantic dependencies, fingerprint, and status. There is no first-class iteration-lane index. T1 adds one and includes it in publication/fingerprinting.

## 1.4 Existing semantic iteration tests are legacy oracles, not an empty test location

`phalcom-semantic/tests/semantic/capabilities/iteration.rs` already exists. It deliberately tests the old shape by defining classes that expose only `iteratorValue(_)`, then observing incomplete/unknown element knowledge.

Therefore T1 must **rewrite/migrate that file's iteration authority**. It must not create a second `capabilities/iteration.rs` and leave the cursor-era oracle active beside it.

Existing loop flow ownership remains in `capabilities/flow_loops.rs`; keep pure flow/fixed-point cases there unless canonical iteration changes their setup.

## 1.5 Ordinary trait dispatch is not a sufficient data model for derived loop evidence

Current `TraitDispatchSite` has these forms:

```rust
pub enum TraitDispatchSite {
    AbstractRequirement { requirement: TraitRequirementId },
    Evidenced(TraitDispatchSelection),
    Assumed(AssumedTraitDispatchSelection),
}
```

There is no `TraitDispatchSite::Derived`.

At the same time, the canonical proof engine already publishes:

```rust
GenericConformanceEvidence::Exact(...)
GenericConformanceEvidence::Assumed(...)
GenericConformanceEvidence::Derived(...)
```

and P3/P4 already transports `DerivedConformanceEvidence` through generic call environments using `RuntimeConformanceSlotRecipe::DerivedConditional`.

This is important for:

```phalcom
run<I>(_ iter: I)
  where I |= Iterator
{
  for x in iter { ... }
}
```

`I |= Iterable` is derived locally from `I |= Iterator`. It must not be fabricated as an exact ordinary `TraitDispatchSelection`, and the compiler must not special-case the theorem by treating `iterator` as an identity operation. T1/T2 therefore use an iteration-specific evidence-backed requirement product described in §4.

## 1.6 Existing lowering/runtime machinery is reusable

`phalcom-core/src/modules/semantic_lowering.rs` already owns:

```rust
TraitInvocationSpec
TraitRequirementInvocationSpec
ModuleLoweringSemantics::trait_invocations
ModuleLoweringSemantics::trait_requirement_invocations
```

and projects ordinary `TraitDispatchSite` products.

`phalcom-core/src/compiler/lib/scope.rs` already emits:

```text
InvokeTraitSelected
InvokeTraitRequirement
```

through helpers that currently discover specs by source range.

`phalcom-core/src/modules/compile.rs` already materializes derived generic evidence/runtime environment recipes. T2 must extract/reuse that machinery rather than designing a loop-only proof engine or VM search path.

## 1.7 List is still cursor-backed in production

`phalcom-core/core/universe/src/collections/list.ph` is still:

```phalcom
@native
class List<T> is Iterable {
    ...
    iteratorValue(_ cursor) { self.at(cursor) }
}
```

where `Iterable` is the legacy runtime superclass. No production `ListIterator<T>` vertical exists. T3 adds canonical conformance in parallel; it does not remove the superclass yet.

## 1.8 Core test registration paths are concrete

The core language integration binary registers files explicitly in:

```text
phalcom-core/tests/core/language/mod.rs
```

and compiler lowering tests already live under:

```text
phalcom-core/tests/core/language/compiler/lowering.rs
phalcom-core/tests/core/language/compiler/lowering_scenarios.rs
```

T2 should extend those established lowering locations unless a dedicated `iteration_lowering.rs` makes the tests substantially clearer; if creating one, register it in `language/mod.rs`.

## 1.9 `ZipLengthMismatchError` fits the ordinary source error package

The Universe has a modular `errors` package. `errors/package.ph` exposes child error modules and `phalcom-modules/src/builtin.rs` enumerates those children. `ZipLengthMismatchError` can therefore be introduced as an ordinary source-authored `Error` subclass, preferably in `errors/iteration.ph`, plus normal package/builtin topology registration. No VM exception ABI is required by the current architecture.

## 1.10 Option payload extraction already has an appropriate bytecode-level shape

The compiler/VM already have `JumpIfNone` and canonical variant payload extraction (`GetVariantPayload(0)` for the single `Some` payload). T4 should use that proven Option branch shape after `next`, not `unwrapOr`, `isSome`, or payload-value inspection.

This directly protects `Iterator<Item = Option<T>>`: `Some(None)` is a yielded item, not exhaustion.

---

# 2. Part-I acceptance objective

Part I is accepted only when all of these statements are true at G2:

1. each semantically accepted `for` lane publishes one canonical semantic resolution keyed by its source expression;
2. lane resolution proves canonical `Iterable`, normalizes canonical `Item` and `Iterator`, proves the projected iterator satisfies canonical `Iterator`, and records executable evidence for `Iterable.iterator` and `Iterator.next`;
3. derived `Iterator => Iterable` evidence is preserved as derived evidence and reaches runtime through existing generic-conformance environment transport—never via a compiler identity special case;
4. `ModuleLoweringSemantics` contains executable lane specifications; compiler loop code does not run trait proof, associated-type normalization, or trait candidate discovery;
5. `List<T>` can be traversed through a fresh ordinary `ListIterator<T>` whose `next -> Option<T>` is stateful and sticky after exhaustion;
6. single-lane and generalized `for` use iterator objects and `next`, not cursor selectors;
7. all lane source expressions run once left-to-right before any iterator acquisition in generalized loops;
8. each attempted strict parallel step that reaches classification has exactly one successful `next` return from each lane, left-to-right; thrown errors propagate immediately;
9. all-`Some` executes the body, all-`None` exits, mixed exhaustion raises `ZipLengthMismatchError` without executing pattern/body work for that attempted step;
10. existing pattern, ordinal, break, continue, return, capture, nesting, and fiber/direct-jump properties survive the traversal migration;
11. Part I has not deleted the global cursor compatibility surface needed by later migrations.

Part I therefore establishes the replacement path and moves `for` onto it. It does **not** claim cursor retirement for the repository as a whole.

---

# 3. Part-I invariant subset

All shared-header invariants apply. These are the ones Luna must keep visibly in mind while implementing T0–T5:

| ID | Part-I interpretation |
|---|---|
| `INV-01` | P4 `Iterator` / `Iterable` identities and theorem remain unchanged. |
| `INV-02` | trait proof and associated `Item`/`Iterator` normalization happen only in semantic analysis. |
| `INV-03` | compiler receives preselected implicit `iterator` / `next` execution facts. |
| `INV-04` | no VM proof/projection/cursor capability search is added. |
| `INV-05` | lane source expression evaluates exactly once left-to-right. |
| `INV-06` | generalized preheader evaluates all sources before any iterator getter, then getters left-to-right once. |
| `INV-07` | one `next` return per lane per attempted step that reaches strict classification; thrown error aborts normally. |
| `INV-08` | `None` is exhaustion; exceptions remain exceptions. |
| `INV-09` | once an iterator returns `None`, subsequent calls remain `None`. |
| `INV-10` | branch on Option tag, not payload; `Some(None)` is not exhaustion. |
| `INV-11` | production `for` stays direct-jump/no materialized loop closure. |
| `INV-12` | captured per-iteration bindings close before rebind/next. |
| `INV-13` | strict parallel iteration is all-Some / all-None / mixed-error. |
| `INV-15` | source iterators are independent; iterator objects are stateful and aliasable. |
| `INV-20` | List iterator state uses ordinary GC-traced fields. |
| `INV-26` | lane evidence is fingerprinted; unrelated body edits do not spuriously change its semantic content. |

Any proposed fix that weakens one of these is an architectural failure, not a test-fix opportunity.

---

# 4. Fixed Part-I interfaces and evidence model

This section resolves the most important ambiguity left in the first monolithic plan.

## 4.1 Semantic product: do not force derived evidence into `TraitDispatchSite`

The semantic product should use an evidence-backed requirement invocation record rather than assuming every implicit loop call can be represented as an existing ordinary expression `TraitDispatchSite`.

The preferred model is:

```rust
pub type ForIterationLaneResolutionIndex =
    BTreeMap<ExpressionId, ForIterationLaneResolution>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForIterationLaneResolution {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator: ForIterationRequirementResolution,
    pub next: ForIterationRequirementResolution,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForIterationRequirementResolution {
    Evidence {
        conformance: GenericConformanceKey,
        evidence: GenericConformanceEvidence,
        requirement: TraitRequirementId,
        signature: CallableSemanticSignature,
    },
    AbstractRequirement {
        requirement: TraitRequirementId,
        signature: CallableSemanticSignature,
    },
}
```

Mechanical type names may adapt, but the information boundary is FIXED.

Why this shape:

- `GenericConformanceEvidence::Exact` retains canonical exact evidence and requirement selections;
- `GenericConformanceEvidence::Assumed` retains the lexical conformance key required to address the active runtime evidence slot;
- `GenericConformanceEvidence::Derived` retains the actual P3/P4 derived proof object and its nested evidence;
- an active trait-default requirement, if encountered, remains a requirement-site invocation rather than being fabricated as evidence;
- lowering can project all cases without rerunning proof.

Do **not** add `TraitDispatchSite::Derived` merely to make loop code convenient unless repository-wide trait-dispatch architecture independently requires it and an adviser approves that broader change. P5 needs a correct loop product, not an expansion of every ordinary dispatch abstraction.

## 4.2 The semantic product must retain requirement identity, not selector strings

The product identifies canonical requirements for:

```text
Iterable.iterator
Iterator.next
```

by `TraitRequirementId` under the canonical P4 trait declarations. Selector text may appear in diagnostics but is not executable authority.

## 4.3 Derived requirement execution is an existing-evidence transport problem

For a derived evidence case, T2 must:

1. take the retained `DerivedConformanceEvidence`;
2. use its canonical requirement selection (`requirements()` / equivalent) for the requested requirement;
3. reuse or extract the existing `modules/compile.rs` logic that turns derived evidence and nested proof bindings into runtime conformance-environment recipes;
4. emit a selected requirement call under that already-materialized environment;
5. perform no proof at runtime.

The implementation may introduce a focused lowering type such as:

```rust
pub enum IterationInvocationSpec {
    Selected(TraitInvocationSpec),
    Requirement(TraitRequirementInvocationSpec),
    Derived(DerivedTraitInvocationSpec),
}

pub struct DerivedTraitInvocationSpec {
    pub requirement: TraitRequirementId,
    pub selection: RequirementSelectionTemplate,
    pub environment: Arc<RuntimeCallEnvironmentRecipe>,
    // plus the exact repo-native method/root metadata required to execute it
}
```

The exact private fields depend on the existing executable-semantics APIs. The architectural requirements are:

- the semantic product owns proof/evidence;
- lowering owns conversion to executable metadata;
- compiler only installs/emits metadata;
- VM only executes it;
- derived environment materialization reuses the P3/P4 path rather than creating a second recipe format.

If existing executable semantics cannot invoke a selected trait target under a supplied derived environment without changing the runtime evidence ABI, T2 must STOP AND CONSULT with the concrete missing capability.

## 4.4 Canonical built-in trait identity

T1 must locate the P4 traits as semantic declarations owned by:

```text
universe.collections.iterator_laws::Iterator
universe.collections.iterator_laws::Iterable
```

Do not resolve these by whatever unqualified `Iterator`/`Iterable` happens to be visible in user source. The legacy runtime class has the same surface name during migration.

Preferred implementation order:

1. use existing Universe `ModuleId`/source identity helpers to identify the canonical `collections.iterator_laws` module;
2. obtain the declaration identity from the semantic module/interface products once per analysis context/workspace;
3. record ordinary trait-header/surface dependencies when consumed;
4. pass the `DeclarationId` into proof/normalization helpers.

Do not hard-code a fake `DeclarationId` or add a second global trait registry.

## 4.5 Successful lane publication only

A lane resolution is executable semantic truth, not a diagnostic-progress record. Publish one only when:

```text
source type known enough
AND Source |= Iterable proven/assumed/derived/abstractly available
AND Item projection normalizes
AND Iterator projection normalizes
AND projected Iterator |= Iterator is available
AND iterator requirement resolves
AND next requirement resolves
```

For invalid, missing, ambiguous, coherence-conflicting, blocked, dynamic, cancelled, budget-exceeded, or internal-failure states:

- preserve the normal semantic terminal state/diagnostic behavior;
- do not mint a fake lowering product;
- do not silently fall back to structural cursor or dynamic `next` lookup.

## 4.6 Fingerprinting and dependencies

`CallableAnalysis.dependency_fingerprint` must include semantic content of the new lane index. Hash:

```text
source ExpressionId / stable lane key
source_type
item_type
iterator_type
canonical conformance key / trait identity
requirement identity
exact evidence fingerprint OR assumed key OR derived rule/evidence semantic fingerprint
selected requirement template identity where concrete
```

Never hash `Arc` addresses.

Use existing `SemanticDependency` variants only. Expected dependencies include as applicable:

```text
TraitHeader
TraitSurface
ConformanceDispatch
AssociatedSurface / declarations used by normalization
nested derived-conformance dependencies already recorded by proof logic
```

Do not add `SemanticDependency::Iteration*` simply to make invalidation easy.

## 4.7 Compiler Option extraction

After an emitted `next` call:

```text
Option<Item>
    ↓
JumpIfNone -> exit/classification
    ↓ Some proven
GetVariantPayload(0)
    ↓
Item
```

Do not route through source-level `unwrapOr`, because:

- it adds irrelevant callable/generic dispatch to the loop hot path;
- the known P4 baseline involves `Option.unwrapOr` / generic call-entry recipe instantiation;
- it is unnecessary once the Option tag has already been tested;
- payload-tag separation is required by `INV-10`.

## 4.8 Generalized-loop ordering

For N lanes, the exact observable order is:

```text
Phase A — source evaluation
source[0]
source[1]
...
source[N-1]

Phase B — iterator acquisition
iterator(source[0])
iterator(source[1])
...
iterator(source[N-1])

Each attempted step
next(iterator[0])
next(iterator[1])
...
next(iterator[N-1])
classify returned Options
```

If a source expression throws, later source expressions and all iterator getters are skipped normally.

If an iterator getter throws, all source expressions have already completed; later iterator getters are skipped normally.

If `next(iterator[k])` throws, the exception propagates normally and later lane `next` calls for that attempt are not forced. `INV-07` applies to an attempted step that reaches strict Option classification.

---

# 5. Part-I change map

This map is intentionally more precise than the monolithic draft. Re-read all named files at local HEAD before editing.

## 5.1 Semantic layer

```text
phalcom-semantic/src/checker/analysis.rs
    add lane-resolution product/index and public semantic shape

phalcom-semantic/src/checker/context.rs
    temporary collection/publication of lane products; dependency recording hooks

phalcom-semantic/src/checker/body.rs
    include lane index in final CallableAnalysis publication if this is where products are finalized

phalcom-semantic/src/checker/statement.rs
    replace cursor-era resolve_iteration_element_application path for Statement::For

phalcom-semantic/src/checker/iteration.rs               [preferred new focused file if useful]
    canonical trait identity lookup/proof/projection/requirement-selection helper

phalcom-semantic/src/db/fingerprint.rs
    fingerprint lane semantic content

phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/types/projection.rs or live equivalent
    READ/REUSE; modify only for a genuinely reusable missing helper, never loop-specific solver logic
```

## 5.2 Semantic tests

```text
phalcom-semantic/tests/semantic/capabilities/iteration.rs
    MODIFY existing legacy oracle into canonical iteration semantic authority tests

phalcom-semantic/tests/semantic/capabilities/flow_loops.rs
    MODIFY only fixtures that need canonical conformance setup; preserve flow ownership

phalcom-semantic/tests/semantic/incremental/iteration.rs   [recommended create]
    lane evidence fingerprint/invalidation tests

phalcom-semantic/tests/semantic.rs or live module registry
    register new incremental module if repository convention requires it

phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md
    update only if normal project policy requires new canonical ownership rows during implementation
```

If `incremental/iteration.rs` is mechanically awkward under current module registration, place the exact cases in `incremental/fingerprints.rs` / `associated_types.rs` rather than inventing a new harness.

## 5.3 Lowering/compiler

```text
phalcom-core/src/modules/semantic_lowering.rs
    project lane resolutions into executable specs

phalcom-core/src/modules/compile.rs
    extract/reuse derived evidence -> runtime recipe construction for implicit lane calls

phalcom-core/src/compiler/lib/scope.rs
    add spec-taking trait invocation emit helpers; keep explicit-call lookup wrappers

phalcom-core/src/compiler/lib/loops.rs
    T4/T5 cursor -> stateful iterator cutover

phalcom-core/src/bytecode.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/chunk.rs / executable-semantics owner
    READ; modify only if a mechanical metadata-plumbing extension is needed and does not change runtime proof authority
```

## 5.4 Universe source

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
    READ; identity/law fixed

phalcom-core/core/universe/src/collections/list.ph
    retain List surface/legacy compatibility during Part I

phalcom-core/core/universe/src/collections/iterator_impls.ph   [recommended create]
    ListIterator + canonical List/ListIterator conformances; intended extension point for Part II built-ins

phalcom-core/core/universe/src/collections/package.ph
phalcom-modules/src/builtin.rs
    add iterator_impls child only if new module chosen

phalcom-core/core/universe/src/errors/iteration.ph             [recommended create in T5]
phalcom-core/core/universe/src/errors/package.ph
phalcom-modules/src/builtin.rs
    ZipLengthMismatchError module/topology
```

Do not remove `collections/iterable.ph` or `CoreClasses.iterable_class` in Part I.

## 5.5 Core/language tests

```text
phalcom-core/tests/core/language/iteration_stateful.rs          [create]
phalcom-core/tests/core/language/mod.rs                         [register]
    direct ListIterator/state/GC/evidence vertical

phalcom-core/tests/core/language/compiler/lowering.rs
phalcom-core/tests/core/language/compiler/lowering_scenarios.rs
    lane lowering projections

phalcom-core/tests/core/language/derived_conformance.rs
    preserve P4 runtime derived-evidence regressions; add only if a direct loop transport discriminator belongs here

phalcom-core/tests/fixtures/language/iteration/**
phalcom-core/tests/language_corpus.rs or live corpus entry
    T4/T5 production-loop behavior/disassembly/oracle migration
```

---

# 6. Part-I coverage ownership ledger

Part I owns the following final P5 coverage obligations. Some rows are first proven in Part I and then re-exercised with additional built-ins/adapters in later parts.

| Coverage | Part-I owner | Required evidence by G2 |
|---|---|---|
| `P5-CV-01` | T3/T4 | empty List iterator + empty List `for` |
| `P5-CV-02` | T3/T4 | one-item List |
| `P5-CV-03` | T3/T4 | multi-item List |
| `P5-CV-04` | T4 | source expression side effect once |
| `P5-CV-05` | T4 | iterator getter side effect once |
| `P5-CV-06` | T5 | two-lane source evaluation LTR |
| `P5-CV-07` | T5 | strict equal-length zip |
| `P5-CV-08` | T5 | initial mixed exhaustion error |
| `P5-CV-09` | T5 | late mixed exhaustion error |
| `P5-CV-10` | T5 | simultaneous exhaustion exits |
| `P5-CV-11` | T5 | tuple pattern binding from Some payload |
| `P5-CV-12` | T5 | pattern mismatch behavior retained |
| `P5-CV-13` | T5 | `at` ordinal 0,1,2 |
| `P5-CV-14` | T5 + Part II reinforcement | ordinal is traversal ordinal, not representation key |
| `P5-CV-15` | T4/T5 | continue causes exactly one next attempt |
| `P5-CV-16` | T4/T5 | break prevents later next |
| `P5-CV-17` | T4/T5 | nested loop reacquires source iterator |
| `P5-CV-18` | T4/T5 | return exits body/loop correctly |
| `P5-CV-19` | T4/T5 | iterator acquisition error propagates |
| `P5-CV-20` | T3/T4/T5 | next error propagates, not exhaustion |
| `P5-CV-21` | T4/T5 | captured iteration values remain independent |
| `P5-CV-22` | T1/T4 | generic `T |= Iterable` loop |
| `P5-CV-23` | T1/T2/T4 | generic `T |= Iterator` loop via derived Iterable |
| `P5-CV-24` | T1/T2/T4 | conditional Iterable conformance |
| `P5-CV-25` | T1/T2/T4 | cross-module conformance |
| `P5-CV-26` | T1 | associated Item normalization |
| `P5-CV-27` | T1/T4 | exact enum-case source identity preserved |
| `P5-CV-28` | T1/T4 | exact enum-case item identity reaches generic body call |
| `P5-CV-29` | T3 | two independent List iterators |
| `P5-CV-30` | T3 | Iterator.iterator identity through P4 derived law |
| `P5-CV-31` | T3 | sticky None after exhaustion |
| `P5-CV-32` | T3 | GC with live List iterator |
| `P5-CV-54` | T1 | structural lookalike rejected |
| `P5-CV-55` | T1 | invalid Iterable.Iterator contract rejected |
| `P5-CV-56` | T1 | Item equality mismatch rejected |
| `P5-CV-57` | T1 | ambiguity/coherence conflict remains diagnostic |
| `P5-CV-58` | T4/T5/G2 | production loop disassembly/code has no cursor sends |
| `P5-CV-59` | T2/T4/G2 | no runtime trait proof/lookup in loop hot path |
| `P5-CV-60` | T1 | body-only incremental edit does not globally invalidate evidence |
| `P5-CV-61` | T1 | Iterable conformance edit invalidates affected loop |
| `P5-CV-62` | T1 | Item/Iterator binding edit invalidates affected loop |
| `P5-CV-63` | T1/T2 | iterator witness selection edit changes executable semantics |
| `P5-CV-67` | T5 | all sources before any iterator acquisition |
| `P5-CV-68` | T5 | no labeled break/continue feature invented |
| `P5-CV-69` | T5 | `ZipLengthMismatchError` |
| `P5-CV-70` | T1/T2/T4 + Part II defaults | custom Iterator with only Item+next participates in `for`; broad defaults completed Part II |
| `P5-CV-96` | T1 | one canonical lane product per source expression |
| `P5-CV-97` | T2 | lowering contains executable iterator+next specs without compiler lookup |
| `P5-CV-98` | T5 | source 2 failure before iterator getter 1 |
| `P5-CV-99` | T5 | iterator getter 1 failure only after all sources ran |
| `P5-CV-109` | T1/T2 | semantic/lowering fingerprint changes when selected evidence changes |
| `P5-CV-110` | T1 | unrelated body-only change leaves lane semantic content stable |

`P5-CV-14` cannot exercise final Map tuple semantics until Part II, and `P5-CV-70`'s "gains defaults" half belongs to Part II. Part I must still establish the loop/evidence halves; do not mark the complete P5 rows closed until their later halves land.

---

# 7. Execution order

```text
T0
 ↓
T1 ── semantic lane evidence
 ↓
T2 ── executable lane projection
 ↓
T3 ── ListIterator direct vertical
 ↓
G1
 ↓
T4 ── single-lane production for
 ↓
T5 ── generalized production for
 ↓
G2
 ↓
Part II may begin
```

T1 and T3 are conceptually separable, but execute them in the order above so the first runtime vertical can immediately be inspected through the same evidence vocabulary that `for` will consume. Do not cut compiler loops before G1.

---

# 8. T0 — Re-ground local checkout, classify baseline, and freeze Part-I census

## Purpose

Establish a trustworthy local starting state, reproduce the known P4 baseline classification, and capture the exact live semantic/compiler/List/error/test surfaces before edits. T0 exists to prevent Luna from executing a precise plan against a materially different tree.

## Preconditions

- shared header read;
- P4 walkthrough/handoff and C6 checkpoint/guidance read;
- no P5 production edit made in the local checkout.

## Consumes

- P4 completed architecture;
- this Part-I file;
- local worktree state.

## Produces

- exact local branch/revision/status record;
- `BL-P5-01` reproduction/classification;
- Part-I cursor/canonical-trait census;
- confirmation of test filters and live module registration paths;
- checkpoint record marking P5 Part I active.

## Files and symbols

**Read only:**

```text
AGENTS.md
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md
P4 walkthrough/handoff
iterator_laws.ph
checker/statement.rs
checker/analysis.rs
checker/context.rs
trait_dispatch.rs
generic_evidence.rs
db/fingerprint.rs
modules/semantic_lowering.rs
modules/compile.rs
compiler/lib/scope.rs
compiler/lib/loops.rs
collections/list.ph
collections/package.ph
errors/package.ph
phalcom-modules/src/builtin.rs
semantic capabilities/iteration.rs
semantic capabilities/flow_loops.rs
core language/mod.rs
language corpus iteration entry + fixtures
```

**Modify:**

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
```

only for durable start/baseline facts.

## Required implementation procedure

Run exactly this preflight first:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -10 --oneline
```

Do not reset, stash, clean, or overwrite unrelated changes. If planned files have overlapping user edits, preserve them and assess whether the plan can mechanically adapt.

Run the focused migration census:

```sh
rg -n 'resolve_iteration_element_application|iterate\(|iteratorValue\(|\$for_cursor|emit_strict_zip_error' \
  phalcom-semantic/src/checker \
  phalcom-core/src/compiler/lib/loops.rs \
  phalcom-core/tests/fixtures/language/iteration

rg -n 'TraitDispatchSite|GenericConformanceEvidence|DerivedConditional|TraitInvocationSpec|TraitRequirementInvocationSpec' \
  phalcom-semantic/src \
  phalcom-core/src/modules \
  phalcom-core/src/compiler/lib/scope.rs

rg -n 'class List|iteratorValue|iterator_laws|ListIterator|iterator_impls' \
  phalcom-core/core/universe/src/collections \
  phalcom-modules/src/builtin.rs

rg -n 'ZipLengthMismatchError|strict zip lanes|ArgumentError' \
  phalcom-core/core/universe/src/errors \
  phalcom-core/src/compiler/lib/loops.rs \
  phalcom-core/tests
```

Confirm the focused test selectors before relying on them:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration -- --list
```

Then run the P4 law/evidence controls and legacy positive iteration baseline:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration -- --nocapture
```

The last command may reproduce the P4 `BASELINE_BLOCKED` failure involving `Option.unwrapOr` / generic call-entry type recipe instantiation. Record the earliest concrete failure and classify it; do not repair it yet merely because it is red.

## Baseline decision tree

Classify `BL-P5-01` as one of:

```text
A  same known cursor-era/Option generic-call baseline; does not break P4 law controls
   -> record D/baseline for now and proceed

B  canonical P4 derived-conformance tests now fail
   -> STOP AND CONSULT; P5 premise drifted

C  failure now appears on a minimal direct canonical `next -> Option<Item>` call independent of cursor code
   -> record as potential P5 blocker; T3 discriminates it before compiler cutover

D  legacy positive corpus is now green
   -> record baseline resolved externally; do not resurrect old failure assumptions
```

## Forbidden approaches

- no production edits in T0;
- no `cargo test --workspace`;
- no broad clippy;
- no "fixing" the legacy corpus before classification;
- no treating historical cursor-document hits as live code defects;
- no relying on the remote planning revision if local HEAD materially differs.

## Tests to run now

Only the commands above.

## Acceptance

T0 is accepted when:

- local status/branch/HEAD are recorded;
- P4 semantic/core derived-conformance controls are classified;
- `BL-P5-01` is truthfully recorded;
- live symbols/files match this part mechanically or drift is documented;
- no architecture premise has been silently invalidated;
- checkpoint says Part I/T0 active/complete as appropriate.

## Local STOP / CONSULT triggers

- P4 canonical traits/rule are absent or materially changed;
- derived-evidence runtime tests fail for a new reason;
- the local checkout contains overlapping planned edits whose intent cannot be safely preserved;
- canonical trait identity has moved such that the header's authority model is false.

## Checkpoint update

Record only:

```text
P5 Part I started
local baseline revision
BL-P5-01 classification
a material drift note if any
```

Do not paste the full grep census into the checkpoint.

---

# 9. T1 — Publish canonical `for` lane semantic resolution

## Purpose

Replace cursor-call result typing as the semantic authority for `Statement::For` with a first-class product that proves canonical `Iterable`, normalizes associated types, proves the projected iterator contract, and retains exact/assumed/derived evidence for both implicit operations.

## Preconditions

- T0 accepted;
- P4 trait law controls are intact;
- live semantic proof/normalization APIs re-read.

## Consumes

```text
GenericConformanceKey
GenericConformanceProof
GenericConformanceEvidence::{Exact, Assumed, Derived}
DerivedConformanceEvidence
ConformanceProofContext / existing proof entry point
TraitRequirementId / TraitSurfaceTable
associated projection normalization APIs
CheckingContext semantic dependency recording
existing Statement::For flow/pattern fixed-point machinery
```

## Produces

```text
ForIterationLaneResolutionIndex
ForIterationLaneResolution
ForIterationRequirementResolution (or mechanically equivalent evidence-backed type)
canonical for-lane resolver
CallableAnalysis publication field
fingerprint/dependency participation
migrated semantic iteration tests
```

## Files and symbols

**Read:**

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/types/projection.rs or live normalizer owner
phalcom-semantic/src/db/fingerprint.rs
```

**Modify:**

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs              # if final publication constructed here
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/db/fingerprint.rs
```

**Create, preferred if it reduces `statement.rs` complexity:**

```text
phalcom-semantic/src/checker/iteration.rs
```

and register it in `checker/mod.rs` or the live module owner.

**Tests:**

```text
MODIFY phalcom-semantic/tests/semantic/capabilities/iteration.rs
MODIFY flow_loops.rs only where fixture conformance setup is necessary
CREATE phalcom-semantic/tests/semantic/incremental/iteration.rs if normal module registration is straightforward
otherwise extend incremental/fingerprints.rs + associated_types.rs with the same discriminators
```

## Required implementation shape

### Step 1 — canonical trait identities

Add a narrow semantic helper that obtains the canonical P4 `Iterator` and `Iterable` declaration identities from the Universe source/module semantic world.

Required properties:

```text
same DeclarationId used by normal imported trait references
no dependency on user lexical import spelling
no collision with legacy runtime class named Iterable
no second registry
ordinary TraitHeader/TraitSurface dependencies recorded
```

If the live semantic workspace already exposes a canonical declaration lookup by `ModuleId + exported name`, reuse it. Do not create string comparisons inside conformance proof.

### Step 2 — prove source `Iterable`

For each `for_stmt.lanes[*].iter`:

1. analyze the source expression exactly once using existing expression analysis;
2. if the type is known, build canonical conformance goal:

```text
SourceType |= canonical Iterable
```

3. call the existing generic conformance proof engine under the lane's lexical evidence environment;
4. preserve every terminal state distinctly—missing, ambiguous, incomplete, coherence conflict, unknown, blocked, dynamic, cancelled, budget exceeded, internal failure;
5. publish no executable lane product unless proof is usable.

Do not use method-name coincidence as a fallback.

### Step 3 — normalize `Item` and `Iterator`

From the proven/available canonical Iterable evidence, normalize:

```text
<Source as Iterable>::Item
<Source as Iterable>::Iterator
```

to canonical `TypeId`s using the existing associated-type authority.

Then prove/establish:

```text
IteratorType |= canonical Iterator
<IteratorType as Iterator>::Item == ItemType
```

The equality already belongs to the Iterable trait law, but the lane product must not publish inconsistent projections if an invalid source impl/semantic state reaches this boundary.

No loop-local projection algorithm is allowed.

### Step 4 — resolve requirement invocations without faking ordinary dispatch

Resolve canonical requirement IDs/signatures for:

```text
Iterable.iterator
Iterator.next
```

For each, retain the conformance evidence form:

```rust
ForIterationRequirementResolution::Evidence {
    conformance,
    evidence: GenericConformanceEvidence::Exact(..) | Assumed(..) | Derived(..),
    requirement,
    signature,
}
```

or `AbstractRequirement` only when analysis is actually inside the relevant active trait-default requirement context.

For exact/derived evidence, verify the evidence contains a requirement selection for the requested requirement. For assumed evidence, retain the lexical conformance key; runtime selection comes from its evidence slot.

Do not turn `Derived` into `TraitDispatchSite::Evidenced`.

### Step 5 — publish lane product

Add to `CallableAnalysis`:

```rust
pub for_iteration_lanes: Arc<ForIterationLaneResolutionIndex>
```

or the repository-native naming equivalent.

Use the source expression's canonical `ExpressionId` as the key. One source expression yields at most one lane resolution in one callable analysis.

The `Statement::For` flow code should bind the pattern from `resolution.item_type` with appropriate semantic evidence status/origin; do not continue taking item knowledge from cursor callable return shape.

Preserve:

```text
solve_loop_header
loop frame push/pop
break/continue flow joins
pattern binding
causal invalidity
explanation parentage
flow graph behavior
```

### Step 6 — retire the cursor helper from production `for`

After the new resolver is authoritative:

```text
Statement::For must not call resolve_iteration_element_application
```

If `resolve_iteration_element` / the compatibility helper has another live consumer, keep it with a comment naming its remaining owner and Part III retirement. If it has no live consumer, delete it now.

### Step 7 — fingerprint lane semantics

Extend `callable_body_product_fingerprint` with deterministic lane-product hashing.

For `GenericConformanceEvidence::Derived`, hash stable semantic identities/fingerprints from the rule/evidence graph, not `Arc` identity. Follow the patterns already used for expression trait dispatch and derived rule products.

Ordering must be deterministic (`BTreeMap` key order or explicit sort).

### Step 8 — dependency recording

Mirror existing generic-call proof dependency behavior:

- exact evidence -> exact conformance dispatch + trait surface;
- assumed evidence -> lexical trait surface/header and caller evidence dependencies already modeled;
- derived conditional source -> source conformance + trait surface + nested evidence dependencies;
- guarded derived rule -> rule/trait surface/dependencies using existing derived proof infrastructure;
- associated normalization -> existing associated/declaration dependency owner.

Do not create an iteration-specific reverse-edge graph.

## Forbidden approaches

- no cursor selector calls for semantic element typing;
- no `TypeData::Applied` "first generic argument" heuristic;
- no `List`/`Range`/Map branch in semantic `for`;
- no structural `iterator`/`next` acceptance without conformance proof;
- no fake `ImplId` or fake exact evidence for lexical assumptions;
- no new `TraitDispatchSite::Derived` solely for P5 convenience;
- no compiler-side Item/Iterator normalization later;
- no broad rewrite of `TraitDispatchSite` or the generic proof engine unless a concrete reusable missing capability is demonstrated and consulted.

## Required semantic tests

### A. Rewrite legacy `capabilities/iteration.rs`

Remove/replace tests whose authority is "a class with only `iteratorValue(_)` influences `for` element type." The canonical replacement fixtures must import/use the P4 traits and explicit conformances.

At minimum add/convert these cases:

1. **`p5_for_explicit_iterable_publishes_lane_resolution`** (`CV-96`)
   - source class explicitly implements canonical Iterable;
   - iterator class implements canonical Iterator;
   - assert one lane product keyed by the lane source expression;
   - assert `item_type == Int`, `iterator_type == IntIter` (example names may differ).

2. **`p5_generic_iterable_loop_uses_assumed_evidence`** (`CV-22`)
   - `run<S>(_ s: S) where S |= Iterable`;
   - `for item in s` publishes assumed/evidence-backed iterator selection;
   - loop binding uses `<S as Iterable>::Item`, not Dynamic/first generic parameter.

3. **`p5_generic_iterator_loop_derives_iterable`** (`CV-23`, `CV-70` loop half)
   - `run<I>(_ i: I) where I |= Iterator`;
   - prove `I |= Iterable` via P4 guarded rule;
   - assert lane's source-side evidence is `GenericConformanceEvidence::Derived` or equivalent preserved provenance;
   - no special-case identity path.

4. **`p5_associated_item_projection_normalizes`** (`CV-26`)
   - use an Iterable whose `Item` depends on another associated/generic projection;
   - assert normalized item type is the canonical solved type.

5. **`p5_structural_iteration_lookalike_is_not_iterable`** (`CV-54`)
   - class defines methods named `iterator` and an iterator-like `next` but no canonical impl;
   - `for` is rejected/blocked with the established conformance diagnostic semantics;
   - no lane lowering product.

6. **invalid conformance contract discriminators** (`CV-55`, `CV-56`)
   - `Iterable::Iterator` does not conform to Iterator;
   - or iterator Item disagrees with Iterable Item;
   - rejection is at semantic authority, not runtime.

7. **ambiguity/coherence terminal discriminator** (`CV-57`)
   - preserve the existing coherence/ambiguity diagnostic; do not collapse to missing/dynamic.

8. **exact enum-case identity** (`CV-27`, semantic half of `CV-28`)
   - source/item exact-case identity survives associated normalization and pattern binding.

### B. Incremental tests

Create a dedicated incremental iteration module if practical. Required discriminators:

1. `CV-109`: change the selected iterator/next evidence or requirement witness; affected callable fingerprint must change.
2. `CV-110` / `CV-60`: change only a loop-body expression that does not alter lane semantic dependencies; lane resolution semantic content remains equal even though whole callable product may legitimately change for body content.
3. `CV-61`: edit the source Iterable conformance; affected loop recomputes.
4. `CV-62`: edit associated `Item` or `Iterator` binding; affected loop recomputes.
5. `CV-63` semantic half: change the selected iterator witness/default; lane product changes so lowering can invalidate.

Do not write a brittle test that assumes the *entire callable fingerprint* stays unchanged after any body edit; the requirement is that iteration evidence does not cause **global/unrelated** invalidation and the lane product itself remains semantically stable when its dependencies did not change.

## Tests to run now

First list the intended filters if module registration was changed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::iteration -- --list
```

Then run only the focused semantic lanes:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::iteration -- --nocapture
```

If the incremental cases were placed in existing modules, run only their exact test-name prefix instead of an unrelated full incremental directory.

Do **not** run runtime `for` tests yet. The compiler still uses cursors after T1 by design.

## Acceptance

T1 is accepted when:

- valid canonical lanes publish exactly one evidence-backed resolution;
- `Statement::For` no longer derives element semantics from cursor callable result shape;
- `T |= Iterator` produces genuine derived `Iterable` evidence;
- structural lookalikes do not pass;
- invalid associated contracts do not publish executable lanes;
- lane products are deterministically fingerprinted and dependency-aware;
- existing loop flow machinery remains in place;
- focused semantic tests pass or any unrelated baseline is classified under header policy.

## Local STOP / CONSULT triggers

STOP if any of these occurs:

1. canonical `iterator_laws` declaration identity cannot be obtained without a second registry or fake identity;
2. the generic proof engine cannot prove the existing P4 theorem in the context needed by `for` without changing the theorem itself;
3. associated normalization needs a new loop-only solver;
4. derived evidence would have to be erased or mislabeled as exact to publish the lane;
5. fixing `Statement::For` requires redesigning loop fixed-point semantics rather than replacing only lane setup;
6. an accepted conformance diagnostic would need to be weakened to make the test green.

## Checkpoint update

After the focused T1 tests pass, record one durable entry:

```text
P5 canonical for-lane semantic product published;
semantic layer now owns Iterable proof + Item/Iterator normalization + iterator/next evidence;
cursor-call result typing is no longer production for authority.
```

---

# 10. T2 — Project lane evidence into executable lowering specs

## Purpose

Convert T1's semantic lane proof into immutable compiler-facing invocation metadata, including locally derived evidence, without re-solving traits in `phalcom-core` or creating a new VM proof mechanism.

## Preconditions

- T1 accepted;
- the lane product retains exact/assumed/derived provenance;
- existing explicit selected/requirement trait lowering has been re-read at local HEAD.

## Consumes

```text
ForIterationLaneResolution
ForIterationRequirementResolution
TraitInvocationSpec
TraitRequirementInvocationSpec
RequirementSelectionTemplate
RuntimeCallEnvironmentRecipe
RuntimeConformanceSlotRecipe::DerivedConditional
existing executable semantics tables
```

## Produces

```text
ForIterationLaneLoweringSpec map in ModuleLoweringSemantics
IterationInvocationSpec with exact/requirement/derived executable forms
shared derived-evidence recipe projection helper reused by generic-call and loop paths
compiler helpers that emit a provided trait invocation spec directly
focused lowering tests
```

## Files and symbols

**Modify:**

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/compiler/lib/scope.rs
```

**Modify only if mechanically necessary:**

```text
phalcom-core/src/chunk.rs or executable-semantics table owner
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
```

A change to the *meaning* of these runtime evidence types is not mechanically necessary and is a consultation trigger.

**Tests:**

```text
phalcom-core/tests/core/language/compiler/lowering.rs
phalcom-core/tests/core/language/compiler/lowering_scenarios.rs
phalcom-core/tests/core/language/derived_conformance.rs only for a narrow execution discriminator if needed
```

## Required implementation shape

### Step 1 — add a dedicated lane lowering attachment

Add a precise lowering site kind and map, for example:

```rust
LoweringSiteKind::ForIterationLane

pub struct ModuleLoweringSemantics {
    ...
    pub for_iteration_lanes:
        BTreeMap<LoweringSite, ForIterationLaneLoweringSpec>,
}
```

Key the site with the lane source expression's source/range plus the dedicated kind. The compiler must be able to retrieve exactly one spec for each semantically accepted lane.

### Step 2 — use an executable invocation enum that represents all semantic evidence forms

Preferred shape:

```rust
pub struct ForIterationLaneLoweringSpec {
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator: IterationInvocationSpec,
    pub next: IterationInvocationSpec,
}

pub enum IterationInvocationSpec {
    Selected(TraitInvocationSpec),
    Requirement(TraitRequirementInvocationSpec),
    Derived(DerivedTraitInvocationSpec),
}
```

`DerivedTraitInvocationSpec` is not a new proof object. It is executable metadata produced from already-proven `DerivedConformanceEvidence` and an existing `RequirementSelectionTemplate` plus the normal runtime conformance environment recipe.

### Step 3 — exact evidence projection

For `GenericConformanceEvidence::Exact(evidence)`:

- select the requested requirement from `evidence.requirements`;
- use `evidence.requirement_views` for the canonical signature;
- construct the same `TraitDispatchSelection` / `TraitInvocationSpec` information that ordinary exact trait dispatch already uses;
- include existing requirement-slot layout;
- do not query `TraitDispatchIndex` again.

This is projection from evidence, not candidate discovery.

### Step 4 — assumed/abstract projection

For lexical assumed conformance:

```text
GenericEvidenceSlotKey(conformance_key, requirement)
    -> existing GenericEvidenceLayout slot
    -> TraitRequirementInvocationSpec
```

For a genuine active trait-default abstract requirement, use the existing active requirement slot semantics.

Do not allocate a new slot namespace for loops.

### Step 5 — derived evidence projection

Refactor `phalcom-core/src/modules/compile.rs` so the existing logic that materializes a `GenericCallConformanceSource::Derived` into runtime recipe/root metadata is callable from lane projection as well.

The shared helper should conceptually accept:

```text
DerivedConformanceEvidence
exact conformance key / subject / trait ref
requested requirement selection
current callable generic environment/layout context
```

and produce the existing runtime recipe/root structures required by an invocation.

Do **not** copy the nested-evidence traversal into `semantic_lowering.rs` and again into `compile.rs`.

The derived lane spec then carries enough executable metadata to call the already-selected requirement target under that environment.

### Step 6 — refactor compiler emit helpers

Current helpers discover explicit-call specs by source range. Extract the actual emit work into spec-taking helpers, for example:

```rust
pub(crate) fn emit_trait_invocation_spec(
    &mut self,
    spec: &TraitInvocationSpec,
    arity: u8,
    range: SourceRange,
) -> Result<(), CompilerError>

pub(crate) fn emit_trait_requirement_invocation_spec(
    &mut self,
    spec: &TraitRequirementInvocationSpec,
    arity: u8,
    range: SourceRange,
) -> Result<(), CompilerError>
```

and an equivalent helper for the derived executable form if required.

Existing:

```text
emit_trait_invoke_if_present
emit_trait_requirement_if_present
```

should remain thin range-lookup wrappers for explicit source calls and delegate to the spec-taking helpers. This prevents T4/T5 from fabricating an expression range lookup for implicit calls.

### Step 7 — preserve runtime authority

The emitted bytecode/executable metadata may prepare methods and conformance roots exactly as ordinary trait calls do. It may not:

```text
search TraitDispatchIndex at runtime
normalize associated types at runtime
prove Iterator/Iterable at runtime
match selector text to decide conformance at runtime
call VM::has_iteration_protocol
```

## Forbidden approaches

- no `emit_operator_send("iterator", ...)` or `emit_operator_send("next", ...)` as canonical implicit loop authority;
- no selector strings stored as proof;
- no derived theorem identity special case in compiler;
- no second derived-evidence recipe format;
- no runtime solver or trait-search opcode;
- no flattening derived evidence into an exact impl if it loses nested environment requirements;
- no source-range scanning of ordinary expression trait maps as a substitute for a lane lowering map.

## Required tests

Add focused projection tests with at least these three semantic shapes:

### Case A — exact source conformance

```text
ConcreteSource |= Iterable
ConcreteIterator |= Iterator
```

Assert lane lowering contains executable selected specs for `iterator` and `next`; no compiler trait discovery required. (`CV-97`)

### Case B — assumed generic source

```phalcom
run<S>(_ source: S)
  where S |= Iterable
```

Assert source-side `iterator` uses the existing requirement/evidence slot model and the projected iterator `next` uses the proof implied by the normalized iterator contract. No fake exact impl. (`CV-22`, `CV-97`)

### Case C — derived source conformance

```phalcom
run<I>(_ iter: I)
  where I |= Iterator
```

Assert:

```text
Iterable evidence is derived
lowering retains a derived executable invocation/environment
no runtime rule solver metadata is introduced
```

This is the central `CV-23` / `CV-59` discriminator.

### Incremental/lowering discriminator

Change an iterator/next witness/default in a fixture and verify the projected lane spec changes (`CV-63`, `CV-109`). The semantic test owns invalidation; the core test only proves lowering reflects the new selection.

## Tests to run now

Use the existing compiler-lowering integration binary rather than inventing a second harness:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_lowering -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_lowering_scenarios -- --nocapture
```

If those filters are too broad, first list and run only new `p5_iteration_*` test names.

Run the P4 derived-evidence runtime control only if the shared derived recipe helper changed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance -- --nocapture
```

Do not run production `language-corpus iteration` yet.

## Acceptance

T2 is accepted when:

- every successful T1 lane can project `iterator` and `next` into executable metadata;
- exact, assumed/abstract, and derived evidence forms remain semantically distinct where needed;
- derived execution reuses P3/P4 runtime environment recipe machinery;
- compiler has spec-taking invocation helpers for implicit operations;
- no compiler/VM proof or associated normalization exists;
- existing explicit trait-call lowering still passes focused regressions.

## Local STOP / CONSULT triggers

STOP if:

1. derived lane execution requires a new runtime conformance-environment lifetime/ABI rather than reuse of P3/P4 machinery;
2. the only apparent route is to rerun conformance proof in `phalcom-core`;
3. `InvokeTraitSelected`/requirement machinery cannot represent the selected callable under an existing evidence environment and the required extension changes VM semantics;
4. a lane spec would need to depend on mutable semantic snapshot objects at runtime;
5. exact/assumed/derived cases cannot be distinguished without modifying the P4 theorem or generic proof contract.

## Checkpoint update

After focused tests pass, record:

```text
P5 for-lane semantic evidence now projects into immutable executable lowering;
derived Iterator=>Iterable execution reuses existing derived conformance transport;
compiler remains proof-free.
```

---

# 11. T3 — Establish List as the first canonical stateful source/iterator vertical

## Purpose

Prove the canonical protocol independently of `for` by making `List<T>` a reusable `Iterable` source that creates a fresh stateful `ListIterator<T>`, with `ListIterator<T> |= Iterator`, typed `next -> Option<T>`, sticky exhaustion, ordinary-field GC rooting, and correct alias/independence behavior.

## Preconditions

- T1/T2 semantic/executable products exist;
- P4 law identity remains unchanged;
- legacy List cursor superclass remains available during transition.

## Consumes

```text
canonical Iterator/Iterable trait identities
P4 Iterator=>Iterable derived rule
List size/at/native storage surface
ordinary source-authored class fields
canonical Some/None representation
```

## Produces

```text
ListIterator<T>
impl Iterable for List<T>
impl Iterator for ListIterator<T>
fresh List.iterator traversal
sticky ListIterator.next
first end-to-end production source iterator independent of for compiler
```

## File placement decision

Preferred placement is a focused new module:

```text
phalcom-core/core/universe/src/collections/iterator_impls.ph
```

that imports/references the canonical P4 traits and `List`, defines `ListIterator<T>`, and declares the conformances. This avoids putting the canonical trait name `Iterable` into the same source module that still uses the legacy runtime class name during transition and gives Part II a natural place to add further built-in iterator implementations.

If current module dependency rules make that topology cyclic or impossible, Luna may mechanically co-locate the implementation in another collection source module **without** changing trait identity or reintroducing a second iteration abstraction.

If creating `iterator_impls.ph`, update:

```text
collections/package.ph
phalcom-modules/src/builtin.rs
```

with the ordinary Universe child/module topology. Do not expose a second public `Iterator`/`Iterable` declaration from this module.

## Files and symbols

**Read:**

```text
collections/list.ph
collections/iterator_laws.ph
collections/package.ph
phalcom-modules/src/builtin.rs
heap/trace.rs
core language derived_conformance.rs
```

**Modify/create:**

```text
CREATE collections/iterator_impls.ph                 [preferred]
MODIFY collections/package.ph                         [if new child]
MODIFY phalcom-modules/src/builtin.rs                 [if new child]
MODIFY list.ph only if a small direct hook/import is mechanically required
CREATE phalcom-core/tests/core/language/iteration_stateful.rs
MODIFY phalcom-core/tests/core/language/mod.rs
```

## Required implementation shape

The state machine is fixed conceptually:

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

impl<T> Iterator for ListIterator<T> {
  type Item = T
}

impl<T> Iterable for List<T> {
  type Item = T
  type Iterator = ListIterator<T>

  iterator { ListIterator.new(self) }
}
```

Adapt field declarations/constructor syntax to live Phalcom syntax. Do not weaken the state semantics.

### Sticky exhaustion is mandatory

The `_done` flag is semantically important. This sequence must remain exhausted:

```text
it = list.iterator
consume until it.next == None
mutate/append to list
it.next == None
it.next == None
```

Without `_done`, a mutable source can resurrect an exhausted iterator, violating `INV-09`.

### Reusable source vs one-shot iterator

These identities must hold:

```text
list.iterator !== list.iterator              # two independent traversal objects
iterator.iterator === iterator               # via canonical derived Iterable law
```

Advancing one List iterator does not advance another. Aliasing the same iterator object shares state normally.

### GC

`ListIterator` holds the List in a normal instance field. Existing `Object::Instance` tracing must keep the source reachable. Do not add a native heap arm, side table, weak reference, or manual root.

### Transition compatibility

Keep the legacy:

```text
List is runtime Iterable superclass
List.iteratorValue(_)
legacy generic methods
```

for now if later Part-II/III consumers still need them. Canonical conformance and legacy inheritance coexist temporarily. Do not use the legacy superclass as evidence of trait conformance.

## Forbidden approaches

- no `SourceIterator` wrapper merely to implement `.iterator`;
- no cursor token stored as public protocol state;
- no snapshot copy of the List solely for traversal;
- no native `Object::ListIterator` arm;
- no resetting/clone-on-iterator behavior;
- no resurrection after first None;
- no deletion of legacy List cursor methods in Part I.

## Required tests

Create `iteration_stateful.rs` with direct source-language execution tests. Suggested exact discriminator set:

1. **`p5_list_iterator_empty_is_sticky`** — `CV-01`, `CV-31`.
2. **`p5_list_iterator_one_and_many_preserve_order`** — `CV-02`, `CV-03`.
3. **`p5_list_source_returns_independent_iterators`** — `CV-29`.
4. **`p5_iterator_iterator_is_self_via_derived_iterable`** — `CV-30`; prove through normal trait call/evidence, not class method special case.
5. **`p5_exhausted_list_iterator_does_not_resurrect_after_append`** — strong sticky exhaustion discriminator.
6. **`p5_list_iterator_aliases_share_state`** — lifecycle/alias foundation for later adapter tests.
7. **`p5_list_iterator_survives_gc_with_only_iterator_live`** — `CV-32`; drop/avoid another strong source reference where harness permits, force GC, continue traversal.
8. **`p5_list_iterator_next_error_is_not_none`** only if a source/list accessor failure can be induced without fabricating unsupported mutation semantics; otherwise T4 custom iterator covers `CV-20`.

Also keep the P4 canonical derived-conformance tests green.

## Tests to run now

Register the new test module, then list it:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --list
```

Run only the new vertical and P4 law control:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance -- --nocapture
```

If the module is registered as `language::iteration_stateful`, use that exact filter after `--list` confirms it.

Do not run `language-corpus iteration`; production `for` is intentionally still cursor-based until T4.

## Acceptance

T3 is accepted when direct calls prove:

```text
List -> fresh ListIterator
ListIterator.next -> Option<T>
order correct
independent traversals
same-object aliasing shares state
sticky exhaustion
canonical Iterator.iterator identity plus P4 Iterator=>Iterable acceptance
GC safety through ordinary fields
```

and no native iterator representation was introduced.

## Local STOP / CONSULT triggers

STOP if:

1. canonical conformance cannot be declared without changing P4 trait identity;
2. Universe module topology forces a public-name conflict that cannot be solved mechanically without changing package architecture;
3. ordinary instance fields fail to retain the List through GC;
4. direct `next -> Option<T>` reproduces a broad generic-call-entry failure that prevents the minimal canonical vertical from executing;
5. making List conform would require removing `CoreClasses.iterable_class` early.

For trigger 4, collect the minimal direct `next` fixture and stack/error trace; do not widen into a generic runtime rewrite without consultation.

## Checkpoint update

Record the first production canonical stateful vertical and the sticky/GC invariants after focused tests pass.

---

# 12. G1 — Canonical semantic/lowering/List vertical gate

## Purpose

Prove all architecture needed for `for` exists independently before replacing production loop lowering. This gate separates semantic/evidence problems from compiler loop problems.

## Required commands

Use exact filters confirmed during T0/T1/T3. Expected shape:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::iteration -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_lowering -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance -- --nocapture
```

If the new incremental or stateful modules use different registered names, substitute only the confirmed names. Do not silently run zero tests.

## Required evidence

G1 must establish:

```text
semantic lane product exists and is canonical
exact/assumed/derived evidence retained
fingerprints/dependencies discriminated
lowering projects implicit iterator+next without proof
List canonical source+iterator works directly
P4 derived theorem/runtime transport still works
```

## Explicitly not required at G1

```text
production for runtime success
other built-in iterables
iterator adapter toolbox
cursor retirement
workspace tests
```

## Gate failure policy

- one test reveals ordinary type/import/fixture error -> mechanical debugging budget;
- one semantic/evidence assertion is wrong -> one hypothesis-driven correction;
- derived evidence cannot execute without new ABI -> STOP;
- P4 control regresses -> STOP.

Do not begin T4 until G1 is PASS or an adviser explicitly accepts a nonblocking baseline condition.

---

# 13. T4 — Cut over single-lane `for` to canonical stateful iteration

## Purpose

Replace the common single-lane name-binding cursor lowering with the preselected lane spec while preserving direct-jump loop control, captures, generic evidence, errors, return, and fiber suspension.

## Preconditions

- G1 PASS/accepted;
- direct ListIterator vertical works;
- compiler spec-taking invocation helpers exist.

## Consumes

```text
ForIterationLaneLoweringSpec
IterationInvocationSpec::{Selected, Requirement, Derived}
compiler emit_*_spec helpers
existing LoopContext
existing local/scratch APIs
JumpIfNone
GetVariantPayload(0) or exact existing canonical equivalent
CloseUpvalue
```

## Produces

```text
single-lane compile_for path uses iterator object + next
no iterate/iteratorValue in common loop path
canonical Option tag/payload handling
preserved direct-jump control semantics
updated disassembly oracle
```

## Files and symbols

**Modify:**

```text
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/tests/language_corpus.rs                  # comments/helper expectations as needed
phalcom-core/tests/fixtures/language/iteration/**
```

**Modify helper code only if necessary:**

```text
language-corpus disassembly support that currently asserts cursor-era zero-allocation/no-WrapSome properties
```

Do not change VM iteration semantics; there is no iteration opcode.

## Required implementation shape

### Step 1 — source evaluation

Keep the fresh loop scope. Compile the lane source once and store it in a synthetic local:

```text
$for_source = <lane source expression>
```

A source-side effect must occur exactly once (`CV-04`).

### Step 2 — iterator acquisition

Retrieve the lane's `ForIterationLaneLoweringSpec` from `ModuleLoweringSemantics` using the source expression site. Emit its preselected `iterator` invocation exactly once and store:

```text
$for_iter = selected Iterable.iterator($for_source)
```

No dynamic selector lookup fallback is permitted for a semantically valid lane. Missing lowering for an otherwise successful lane is an internal compiler/lowering invariant failure, not a cue to send `iterator` normally.

### Step 3 — loop header calls `next`

At the header:

```text
option = selected Iterator.next($for_iter)
```

Call exactly once per attempted single-lane step.

The value can remain on stack if stack discipline is simple, or be stored in a scratch local if needed for branch/payload extraction. Prefer the smallest shape that is obviously correct under break/continue and source spans.

### Step 4 — branch on Option tag

Use the existing Option-aware control primitive:

```text
JumpIfNone -> exit
```

After the branch has established `Some`, extract:

```text
GetVariantPayload(0)
```

into the loop binding value.

Do not send:

```text
unwrapOr
isSome
match
```

and do not compare the payload to `None`.

### Step 5 — body and capture lifecycle

Reuse the existing body compilation and loop context. Preserve:

```text
break -> exit
continue -> bottom/close then next header
return -> normal function return semantics
fiber yield -> no materialized loop block/native reentrant frame
```

If the binding was captured, `CloseUpvalue(binding_slot)` must happen before the next iteration overwrites/rebinds the slot. `continue` must target that close point, not jump around it.

### Step 6 — no explicit cursor step

There is no post-body `iterate(cursor)` call. Advancement happens when the next header invokes `next`.

Conceptual lowering:

```text
begin loop scope
source = eval expr
iter = invoke lane.iterator(source)
declare binding
push LoopContext

header:
    option = invoke lane.next(iter)
    if None -> exit
    item = payload[0]
    binding = item
    body

continue-target:
    close captured binding if needed
    loop header

exit:
    patch break/continue
end scope
```

## Disassembly oracle migration

The current corpus describes a cursor direct-jump loop and includes cursor-era "zero allocation" / no-`WrapSome` expectations. P5 changes the allocation model intentionally:

```text
one iterator allocation per reusable-source traversal is valid
Some(Item) values from next are valid canonical Option values
```

Preserve the load-bearing oracle:

```text
no Closure / block_call on taken for path
```

Replace cursor-specific disassembly expectations with:

```text
no iterate selector
no iteratorValue selector
implicit calls emitted through trait-selected/requirement execution metadata
no runtime proof/search helper
```

Do not preserve a "no iterator object" assertion after the architecture explicitly requires iterator objects.

## Required tests

Add/update focused source fixtures for:

1. `CV-01..03`: empty/one/many List through canonical loop;
2. `CV-04`: source expression invoked once;
3. `CV-05`: iterator getter invoked once;
4. `CV-15`: `continue` advances exactly once;
5. `CV-16`: `break` causes no later `next`;
6. `CV-17`: nested loop reacquires fresh inner source iterator on each outer execution;
7. `CV-18`: return from body;
8. `CV-19`: error from iterator getter propagates unchanged;
9. `CV-20`: error from `next` propagates unchanged;
10. `CV-21`: closure captures per-iteration binding independently;
11. `CV-22`: generic `T |= Iterable` runtime loop;
12. `CV-23`: generic `T |= Iterator` runtime loop through **derived** Iterable evidence;
13. `CV-24`: conditional Iterable conformance runtime loop;
14. `CV-25`: cross-module conformance runtime loop;
15. `CV-27/28`: exact enum-case preservation where existing test harness supports it;
16. `INV-10` explicit discriminator: iterator with `Item = Option<Int>` yields `Some(None)` as one body item before exhaustion;
17. `CV-54`: a class with matching names but no conformance does not compile as `for`.

For error-order tests, custom source-authored Iterators are preferable to abusing List internals.

## Tests to run now

First run the disassembly/control discriminators:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative -- --nocapture
```

Then run the positive iteration corpus **once** after the coherent single-lane patch is complete:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration -- --nocapture
```

If the corpus includes generalized cases that are expected to remain red until T5, use exact single-lane fixture tests or a temporary focused integration test rather than forcing generalized migration into T4. Record those expected reds as part-state, not baseline regressions.

Run the semantic generic evidence test only if a runtime failure points back to projection:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration -- --nocapture
```

## Acceptance

T4 is accepted when:

- common single-lane `for` executes through the T2 lane spec;
- no common-loop cursor sends remain;
- `Some(None)` is observed as an item, not exhaustion;
- generic direct and derived evidence execute without runtime proof;
- break/continue/return/capture/fiber direct-jump behavior is preserved;
- disassembly oracle has been updated to the stateful architecture;
- any remaining generalized loop failure is explicitly owned by T5.

## Local STOP / CONSULT triggers

STOP if:

1. a derived loop invocation requires reconstructing environment/proof per iteration;
2. safe Some payload extraction cannot be expressed with existing canonical ADT bytecode after `JumpIfNone`;
3. preserving fiber suspension requires introducing a native/reentrant iteration frame;
4. correct capture lifecycle would require changing closure/upvalue architecture rather than preserving existing close-before-rebind;
5. the only way to make a semantically valid loop compile is a dynamic `next` send fallback.

## Checkpoint update

After focused T4 evidence passes, record:

```text
single-lane production for now executes preselected Iterable.iterator + Iterator.next;
Option payload extracted directly after tag guard;
direct-jump/capture/fiber invariants preserved.
```

---

# 14. T5 — Cut over generalized patterns, ordinals, parallel lanes, and strict zip

## Purpose

Migrate `compile_for_lanes` and all non-common `for` forms to the same canonical lane specs, enforce the ratified two-phase preheader, preserve pattern/ordinal/control semantics, and replace the `ArgumentError` strict mismatch with `ZipLengthMismatchError`.

## Preconditions

- T4 single-lane canonical path accepted;
- T2 supports one executable lane spec per source expression;
- existing generalized pattern scratch/binding code has been re-read at local HEAD.

## Consumes

```text
N x ForIterationLaneLoweringSpec
existing generalized pattern test/commit helpers
existing scratch/local APIs
existing LoopContext
canonical Option tag/payload operations
ordinary source error hierarchy
```

## Produces

```text
generalized canonical for implementation
two-phase source/getter evaluation order
strict all-Some/all-None/mixed classification
ZipLengthMismatchError
preserved patterns/at/captures/break/continue
no cursor selectors anywhere in compiler loop lowering
```

## Files and symbols

**Modify:**

```text
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/tests/fixtures/language/iteration/**
phalcom-core/tests/language_corpus.rs and focused helpers/comments as needed
```

**Create/modify ordinary error package:**

```text
CREATE phalcom-core/core/universe/src/errors/iteration.ph
MODIFY phalcom-core/core/universe/src/errors/package.ph
MODIFY phalcom-modules/src/builtin.rs
```

Recommended error declaration:

```phalcom
class ZipLengthMismatchError is Error {}
```

If existing Error construction conventions require only inherited `new(_)`, use that. Do not add native error representation.

## Required implementation shape

### Phase A — evaluate every source before any iterator acquisition

The old generalized loop interleaves source evaluation and cursor acquisition. Replace it.

Exact order:

```text
for lane 0..N:
    source_slots[lane] = evaluate source[lane]

for lane 0..N:
    iterator_slots[lane] = emit selected iterator(source_slots[lane])
```

This is observable and fixed by `INV-05`/`INV-06`.

`CV-98` discriminator:

```text
source0 succeeds
source1 throws
iterator0 getter would record a side effect

expected:
source0 effect
source1 effect/error
NO iterator0 getter effect
```

`CV-99` discriminator:

```text
source0 effect
source1 effect
iterator0 getter throws

expected:
source0 effect
source1 effect
iterator0 getter effect/error
NO iterator1 getter effect
```

### Ordinal setup

Retain one shared zero-based ordinal local:

```text
$for_ordinal = 0
```

It is traversal attempt/body ordinal, not a collection cursor/key. Only increment after one successful all-Some body iteration reaches the step point. `continue` still represents completion of that iteration and advances ordinal once; `break`/return do not create another ordinal.

### Attempt phase — collect Option results once

For each lane left-to-right:

```text
option_slots[lane] = invoke selected next(iterator_slots[lane])
```

If an invocation throws, propagate immediately. Otherwise do not branch on the first `None` before all lane calls have returned; strict classification requires the complete returned Option vector.

Do not call any lane `next` twice to classify.

### Classification

Classify retained Options without rereading iterators:

```text
all Some
    -> extract every payload exactly once
       perform pattern tests/commits
       write optional `at` bindings from shared ordinal
       run body

all None
    -> exit normally

mixed Some/None
    -> raise ZipLengthMismatchError
       no pattern tests/commits for this attempt
       no body
       no rollback of iterator state already advanced by next calls
```

The easiest correct compiler shape may count/branch through Option tags using scratch locals; it does not need an actual runtime count object if jump structure is clearer. Correctness matters more than micro-optimizing the preselected calls in P5.

### Payload extraction

Only after classification proves all lanes are `Some`:

```text
GetVariantPayload(0) for each option slot
```

Then reuse the existing generalized pattern machinery.

### Pattern semantics

Preserve current semantics for:

```text
name pattern
tuple/product patterns
pattern mismatch raise
stable pattern-local slots
matcher scratch cleanup
```

Do not redesign pattern matching in P5.

### Captures

Before the next attempt, close every captured pattern binding that will be rebound. Preserve the current generalized code's name collection/dedup behavior unless live source has a more precise stable binding-ID mechanism.

`continue` must target the capture-close/ordinal-step point.

### Break/return

`break` patches directly to loop exit. `return` retains normal function-return behavior. Neither may trigger another `next` call.

### Strict mismatch error

Replace the current compiler helper that raises:

```text
ArgumentError("strict zip lanes ended at different times")
```

with construction/raise of `ZipLengthMismatchError` using the ordinary language error class.

The exact message may retain the current wording unless the normative spec requires another string. The **class identity** is the important semantic contract.

### No labeled-loop expansion

Do not add labeled break/continue. Preserve the current unlabelled innermost-loop model. If parser tests already reject a labeled form, keep that oracle; if parser never recognizes it, no new semantic feature is required (`CV-68`).

## Forbidden approaches

- no size precheck to decide strict zip;
- no shortest-input truncation;
- no `source0 -> iterator0 -> source1 -> iterator1` interleaving;
- no first-None early exit before remaining `next` calls return for a non-throwing attempted strict step;
- no reread/rollback after mixed exhaustion;
- no per-lane ordinal based on key/cursor/index representation;
- no `ArgumentError` as final mismatch type;
- no cursor compatibility branch inside `compile_for_lanes`;
- no broad pattern-matcher rewrite;
- no labeled loop feature.

## Required tests

### A. Evaluation-order fixtures

Add source/getter event-log fixtures proving `CV-06`, `CV-67`, `CV-98`, `CV-99` exactly.

The fixture should record externally visible markers rather than infer order from final values. A simple mutable list/string log owned outside the source objects is sufficient.

### B. Strict zip fixtures

1. equal lengths (`CV-07`);
2. initial mismatch, e.g. `[]` with `[1]` (`CV-08`);
3. late mismatch, e.g. `[1]` with `[10,20]` (`CV-09`);
4. simultaneous completion (`CV-10`);
5. mixed mismatch catches/observes `ZipLengthMismatchError`, not generic `ArgumentError` (`CV-69`);
6. side-effecting `next` confirms every non-throwing lane was called once before mixed classification.

### C. Pattern/ordinal/control fixtures

Preserve or add:

```text
CV-11 tuple pattern binding
CV-12 pattern mismatch raises
CV-13 at ordinal 0,1,2
CV-15 continue advances once
CV-16 break no later next
CV-17 nested loops
CV-18 return
CV-21 capture independence
```

`CV-14` gets a generic ordinal discriminator now; Part II repeats it with canonical Map entry iteration once Map migrates.

### D. Error propagation

- iterator getter error occurs after all sources and before later getters;
- `next` error remains the original error and is never converted to `None`/zip mismatch;
- strict mismatch is raised only when all called `next`s returned normally and the retained Options are mixed.

### E. Disassembly/cursor census

After T5:

```sh
rg -n 'iterate|iteratorValue|\$for_cursor' phalcom-core/src/compiler/lib/loops.rs
```

must find no production cursor lowering. Historical comments in that file should be rewritten/deleted.

The iteration disassembly test must retain:

```text
no block_call/Closure on direct loop path
```

and assert the canonical trait invocation path rather than cursor selectors.

## Tests to run now

Run the full **iteration feature lane**, not the workspace:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::flow_loops -- --nocapture
```

Run the direct stateful List vertical only if a failure suggests source iterator regression:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --nocapture
```

Do not run collection/sequence/iterator broad suites yet; Part II is about to rewrite those surfaces.

## Acceptance

T5 is accepted when:

- all supported `for` forms use canonical lane specs;
- generalized setup is two-phase and side-effect tests prove it;
- one retained Option result per lane drives classification;
- strict mismatch raises `ZipLengthMismatchError`;
- no cursor selector remains in compiler loop lowering;
- patterns/at/break/continue/return/captures/nesting remain correct;
- direct-jump/fiber-compatible architecture remains intact;
- no global cursor API has been prematurely deleted.

## Local STOP / CONSULT triggers

STOP if:

1. accepted normative ordering in a newer authority conflicts with the two-phase model;
2. strict zip can only be implemented by runtime collection size knowledge;
3. error handling would require calling later `next`s after an earlier thrown exception;
4. generalized pattern correctness requires a material pattern-system redesign;
5. parser has newly landed labeled loop semantics that materially change `break`/`continue` ownership;
6. adding `ZipLengthMismatchError` unexpectedly requires a native/core-class ABI rather than ordinary source declaration;
7. preserving fiber suspension would require a new execution frame.

## Checkpoint update

Record:

```text
all production for lowering is canonical stateful iteration;
generalized preheader is source-all-then-iterator-all;
strict parallel classification is all-Some/all-None/mixed ZipLengthMismatchError;
compiler loops contain no cursor selectors.
```

---

# 15. G2 — Canonical production-`for` gate

## Purpose

Prove the entire `for` feature is independent of the cursor protocol before Part II broadens canonical iteration to every built-in source and the production default surface.

## Mandatory focused commands

First ensure semantic/evidence authority is still green:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::flow_loops -- --nocapture
```

Run the Part-I core/lowering verticals:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_lowering -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance -- --nocapture
```

Run the production loop lanes:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_negative -- --nocapture
```

Run the compiler cursor census:

```sh
rg -n 'resolve_iteration_element_application|iterate\(|iteratorValue\(|\$for_cursor' \
  phalcom-semantic/src/checker/statement.rs \
  phalcom-core/src/compiler/lib/loops.rs
```

Expected final interpretation:

```text
no production Statement::For cursor resolver
no compiler loop cursor send / cursor local
compatibility helper may remain elsewhere only if another live non-for owner is explicitly documented
```

Run diff integrity:

```sh
git diff --check
```

Do not broaden after these pass.

## G2 evidence checklist

The walkthrough/checkpoint record must be able to answer YES to each:

- [ ] canonical trait identity was preserved;
- [ ] semantic lane product is published and fingerprinted;
- [ ] exact/assumed/derived evidence forms are not conflated;
- [ ] derived evidence reaches executable lowering through existing runtime environment machinery;
- [ ] compiler performs no trait proof or associated normalization;
- [ ] List direct stateful iterator vertical is green;
- [ ] single-lane production loop is canonical;
- [ ] generalized production loop is canonical;
- [ ] all-source-before-any-getter ordering is proven;
- [ ] `next` Option tag/payload semantics preserve `Some(None)`;
- [ ] strict mixed exhaustion raises `ZipLengthMismatchError`;
- [ ] direct-jump/no-block-call loop property is preserved;
- [ ] captures close before rebind/next;
- [ ] compiler/semantic production-for path has no cursor dependency;
- [ ] unrelated/baseline failures are classified rather than repaired opportunistically.

## G2 completion classification

Use the shared completion truth table. Appropriate Part-I statuses include:

```text
IMPLEMENTED_FOCUSED_TESTED
IMPLEMENTED_FOCUSED_TESTED_WITH_BASELINE_DEFERRED
BLOCKED_CONSULTATION_REQUIRED
```

Do not call the whole P5 plan complete at G2.

---

# 16. Part-I narrow verification budget

This section specializes the shared BUILD/STABILIZE/CERTIFY doctrine.

## BUILD MODE

Run only:

```text
T0 baseline controls
T1 capabilities::iteration + exact incremental iteration tests
T2 exact compiler-lowering tests
T3 iteration_stateful + P4 derived-conformance control
T4 iteration_disasm/negative + one coherent positive single-lane lane
T5 language-corpus iteration + flow_loops
```

Do not run a suite merely because a file compiled after an import fix.

## G1/G2 stabilization

G1 and G2 commands above are the intended Part-I stabilization breadth.

## Only if evidence demands

```text
full phalcom-semantic test binary
full phalcom-core --test core
full collection corpus
full generic-conformance corpus
```

Use these only when a focused failure suggests adjacent ownership that the narrow tests cannot discriminate.

## Explicitly deferred during Part I

```text
cargo test --workspace
workspace clippy
all docs tests
all LSP tests
broad sequence/iterator collection corpus unrelated to for
runtime-class retirement greps outside the Part-I compiler/semantic path
```

Those either belong to later P5 parts or checkpoint/release certification.

---

# 17. Part-I failure diagnosis packets

When a nontrivial failure survives the permitted local correction, send a compact consultation packet. Do not ask an adviser to rediscover Part I.

## 17.1 Semantic proof/evidence incident

```text
Task: T1
Source fixture:
Canonical source type:
Goal proved/failed:
Expected evidence form: Exact | Assumed | Derived | Abstract
Observed GenericConformanceProof:
Normalized Item:
Normalized Iterator:
Iterator proof:
Requirement ID/signature:
Semantic dependencies recorded:
Why the existing proof API is insufficient:
Proposed alternatives (if >1):
Invariant at risk:
```

## 17.2 Derived executable transport incident

```text
Task: T2
Semantic lane evidence:
DerivedConformanceOrigin:
Requested requirement:
RequirementSelectionTemplate:
Existing RuntimeConformanceSlotRecipe path:
Existing bytecode/executable invocation available:
Exact missing bridge:
Would a VM ABI change be required? why:
Minimal failing lowering/runtime test:
```

## 17.3 Loop codegen/control incident

```text
Task: T4 or T5
Minimal source:
Disassembly around loop:
Expected stack/local shape:
Observed stack/local shape:
LoopContext break/continue targets:
Capture slots closed where:
Option tag/payload operations:
Trait invocation spec emitted:
Runtime result/error:
Invariant at risk:
```

## 17.4 Baseline generic-call incident

```text
Task: T3/T4
Minimal direct next fixture:
Does failure occur without for? yes/no
Does it require unwrapOr? yes/no
Does P4 derived-conformance control pass? yes/no
Earliest error:
Call environment recipe involved:
Why this blocks canonical P5 path:
```

---

# 18. Part-I checkpoint bookkeeping

The shared checkpoint remains the work-state authority.

## At T0

Record:

```text
Part I active
starting revision/status
BL-P5-01 classification
```

## Durable events worth recording during Part I

Only update the checkpoint when one of these happens:

```text
T1 semantic lane product becomes stable
T2 executable lane lowering contract becomes stable
T3 first List stateful vertical passes
G1 result
T4 single-lane for cutover passes
T5 generalized for cutover passes
G2 result
consultation/amendment
deferred blocking/nonblocking baseline
```

Do not append a diary entry for every mechanical edit.

## At G2

Record:

```text
Part I completion classification
new stable semantic/lowering interfaces
production for no longer cursor-dependent
focused test commands/results
remaining legacy cursor owners intentionally deferred to Parts II/III
next action: Part II T6
```

---

# 19. Part-I handoff contract to Part II

Part II may assume only the interfaces actually proven at G2.

Required handoff facts:

```text
ForIterationLaneResolution type/path
lane index field on CallableAnalysis
ForIterationLaneLoweringSpec type/path
IterationInvocationSpec variants actually implemented
spec-taking compiler invocation helpers
ListIterator type/module
canonical List Iterable conformance
canonical ListIterator Iterator conformance
ZipLengthMismatchError path/export
production for compiler path
exact G1/G2 results
BL-P5-01 final Part-I classification
any approved deviations
```

Part II must **not** need to reopen:

```text
who proves Iterable
who normalizes Item/Iterator
how derived Iterator=>Iterable evidence reaches runtime
for evaluation order
strict parallel mismatch semantics
Option payload/exhaustion representation
loop direct-jump control architecture
```

If G2 leaves any of those unresolved, Part I is not ready to hand off.

---

# 20. Part-I implementation commit shape

Commit boundaries are recommendations, not semantic authority, but the following shape keeps reviews coherent:

```text
commit 1  T0 checkpoint baseline only, if repository workflow commits state-start records
commit 2  T1 semantic lane product + semantic/incremental tests
commit 3  T2 lowering projection + derived transport reuse + compiler spec helpers
commit 4  T3 ListIterator vertical + direct runtime/GC tests
commit 5  T4 single-lane for cutover + disassembly oracle migration
commit 6  T5 generalized for + ZipLengthMismatchError + ordering/strict tests
commit 7  G2 checkpoint/walkthrough state update for Part-I boundary, if workflow records gates separately
```

Do not make one commit per mechanical compiler error. Do not combine T1–T5 into one unreviewable migration commit unless repository workflow explicitly requires squashing later.

---

# 21. Part-I self-review checklist

The architect/executor should use this before declaring G2 or before handing Part I to a reviewer.

## Semantic authority

- [ ] no production `for` element type is derived from `iteratorValue` return type;
- [ ] canonical trait declarations are identified by stable semantic identity, not user lexical name;
- [ ] source Iterable proof, associated normalization, projected Iterator proof, and requirement identities are semantic-layer facts;
- [ ] derived Iterator=>Iterable evidence remains `DerivedConformanceEvidence` provenance;
- [ ] structural lookalikes are rejected;
- [ ] invalid associated contracts do not publish executable lanes.

## Lowering/runtime authority

- [ ] exact/assumed/derived lane requirements project without rerunning proof;
- [ ] derived evidence reuses P3/P4 environment recipes;
- [ ] compiler has spec-taking helpers rather than implicit range tricks;
- [ ] VM sees only executable selection/environment metadata;
- [ ] no new iteration-specific runtime proof registry/opcode exists.

## List vertical

- [ ] List source creates fresh iterators;
- [ ] ListIterator is an ordinary source class;
- [ ] next is typed Option<T>;
- [ ] sticky exhaustion survives later List mutation;
- [ ] iterator.iterator is self through the canonical Iterator default and ListIterator is accepted as Iterable through the P4 derived law;
- [ ] GC retains source through ordinary fields.

## Compiler loop

- [ ] source evaluated once;
- [ ] generalized sources all evaluate before any iterator getter;
- [ ] iterator getter once per lane;
- [ ] next once per non-throwing lane before strict classification;
- [ ] errors propagate normally;
- [ ] `JumpIfNone`/Option tag controls exhaustion;
- [ ] Some payload extracted directly;
- [ ] Some(None) is a yielded item;
- [ ] no cursor selectors in production loop compiler;
- [ ] break/continue/return/nesting/capture/fiber behavior preserved;
- [ ] strict mixed exhaustion raises ZipLengthMismatchError;
- [ ] no labeled-loop feature invented.

## Incremental/performance boundary

- [ ] lane product participates in callable fingerprint;
- [ ] selection change changes semantic/lowering product;
- [ ] unrelated body-only edit does not create global iteration invalidation;
- [ ] no trait proof occurs per iteration/item;
- [ ] iterator acquisition is once per traversal;
- [ ] no requirement to preserve cursor-era zero-allocation loop claim remains.

## Scope discipline

- [ ] runtime Iterable superclass still exists until later planned retirement;
- [ ] non-List built-in source migration is not pulled into Part I unless mechanically required for an existing Part-I fixture;
- [ ] iterator adapter/default toolbox is deferred to Part II;
- [ ] VM positional expansion cursor probe is deferred to Part III;
- [ ] workspace certification is not claimed.

---

# 22. End state after Part I

After successful G2, the repository should have two iteration eras temporarily coexisting, but with a clean ownership boundary:

```text
CANONICAL / PRODUCTION FOR PATH

source expression
    ↓ semantic analysis
Source |= Iterable evidence
    ↓
Item + Iterator normalization
    ↓
Iterator |= Iterator proof
    ↓
iterator + next requirement evidence
    ↓ CallableAnalysis lane product
    ↓ ModuleLoweringSemantics executable lane spec
compiler direct-jump for
    ↓
stateful iterator.next -> Option<Item>
    ↓
None exit / Some payload body


TEMPORARY LEGACY COMPATIBILITY OUTSIDE FOR

runtime Iterable superclass
iterate(_)
iteratorValue(_)
legacy collection methods/adapters/other consumers
    ↓
Parts II/III migrate then retire
```

That is the deliberate Part-I stopping point. Do not "finish the cleanup" early: broad built-in migration, default-rich production iterator surface, boundedness integration, positional expansion, repository subclass migration, runtime class removal, public-name cutover, and final cursor census belong to the next two files.
