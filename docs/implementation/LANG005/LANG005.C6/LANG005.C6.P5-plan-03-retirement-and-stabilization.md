---
id: LANG005.C6.P5.PART-III
parent_plan: LANG005.C6.P5
shared_header: LANG005.C6.P5-plan-00-shared-header.md
previous_part: LANG005.C6.P5-plan-02-builtins-and-iterator-surface.md
requirements_analysis: LANG005.C6.P5-plan-03-retirement-and-stabilization-requirements-analysis.md
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan-part
part: 3-of-3
planning_cycle: restructured-pass-4-live-audit
file_pass: 4-of-4
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-09-18
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: 206e0ab91129e189782c4028de5a40379bcc033c
planning_revision_subject: "feat(lang005): T14 wire intermediary abstract Collection class and retire legacy UniverseKey::Iterable"
executor_profile: GPT-5.6-Luna-High-or-Extra-High
scope_tasks:
  - T12
  - T13
  - T14
  - T15
scope_gates:
  - G5
  - G6
---

# LANG005.C6.P5 Part III — Canonical Retirement, Consumer Closure, and Stabilization

> **For Luna / agentic implementers:** read `LANG005.C6.P5-plan-00-shared-header.md` first, then this file. The shared header owns plan-wide invariants, verification doctrine, baseline policy, consultation protocol, and cross-part architecture. This file is the execution authority for T12–T15 and G5–G6 only.
>
> **Critical takeover fact:** T14 was partially executed before T12/T13 retirement prerequisites were fully closed. Do **not** undo the landed `Collection` hierarchy merely because cursor-dependent paths now fail. The correct recovery is to complete canonical semantic/lowering coverage and remove the remaining cursor consumers.
>
> **Primary live regression:** after the runtime `Iterable` superclass was removed, the focused `language::iteration` group exposed six `does not understand 'iterate(_)'` failures. Current source proves the path: `EntrySelection::Inline` only injects canonical iterator laws + Range conformance into semantic analysis, missing List/Bytes/etc.; missing lane products then trigger the still-live `compile_for_lanes` cursor compatibility lowering. Repair the semantic support closure and delete the compatibility fallback. Never restore `iterate` to `Collection`.

**Goal:** finish P5 with one production iteration architecture by giving positional expansion first-class canonical iteration evidence, ensuring every accepted `for` reaches canonical lowering in every compilation context, migrating the remaining repository/source consumers, completing and verifying the new abstract `Collection` hierarchy/public trait cutover, deleting all live cursor selectors/fallbacks, and stabilizing tooling/docs/tests/records.

**Part-III terminal condition:** after G6 there is no live production path that establishes or executes iterability through `iterate(_)`, `iteratorValue(_)`, runtime selector coincidence, `Iterable` class ancestry, or compiler/VM trait rediscovery. `Collection` may remain as an abstract runtime taxonomy class, but only explicit canonical trait evidence makes a value iterable.

---

# 0. Executor contract

The implementer is operating under constrained architectural authority.

The implementer may:

- adapt helper/type names to current repository style;
- split or colocate private semantic helpers mechanically;
- use a curated inline-Universe support-module set if full Universe analysis would pull unrelated diagnostic debt;
- mechanically rename `ForIterationRequirementSpec` to a shared implicit-iteration name if expansion reuse makes the old name misleading and no semantics change;
- adapt fixture names and exact test filtering to current registration.

The implementer must:

- preserve every fixed invariant below;
- preserve the already-landed `Collection` runtime hierarchy unless an explicit architect amendment changes it;
- fix the semantic/lowering authority hole rather than restoring compatibility methods;
- stop before changing Tuple/Dynamic semantics, arbitrary Dynamic expansion semantics, runtime evidence authority, or REPL semantic architecture without consultation;
- execute sparse discriminating tests in BUILD MODE;
- classify broad failures A/B/C/D before editing;
- perform at most one serious correction for the same nontrivial semantic failure;
- update the checkpoint only at durable interfaces, gates, consultations, and deferred failures;
- produce final P5 walkthrough/handoff records.

Testing is evidence gathering. Do not run the workspace because a local task passed.

---

# 1. Checkpoint acceptance objective

P5 is the production migration plan inside C6. Part III is its destructive closure/stabilization part, not full C6 certification.

Part III advances the checkpoint by proving:

```text
canonical semantic evidence
    -> executable lowering metadata
    -> stateful ordinary iterator objects
```

is the **only** production iteration path for:

- `for`;
- generic positional `*` expansion;
- built-in collection/view traversal;
- repository-defined iterable sources such as Sheetcalc `RefRange`;
- iterator adapters/defaults/terminals already landed in Part II.

P6 retains broader C6 stress/certification and unrelated generic-constraint closure.

---

# 2. Repository grounding and live audit

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   206e0ab91129e189782c4028de5a40379bcc033c
subject:    feat(lang005): T14 wire intermediary abstract Collection class and retire legacy UniverseKey::Iterable
```

Re-read all named paths at local HEAD before editing. Mechanical drift may be adapted. Architectural drift requires consultation.

## 2.1 Already-landed canonical infrastructure

| Concept | Live owner | Part-III treatment |
|---|---|---|
| canonical traits | `core/universe/src/collections/iterator_laws.ph` | preserve identities |
| `for` semantic product | `CallableAnalysis.for_iteration_lanes` | reuse/verify |
| executable `for` lane spec | `ModuleLoweringSemantics.for_iteration_lanes` | reuse/verify |
| canonical single-lane lowering | `compile_canonical_for` | preserve |
| canonical generalized lowering | `compile_canonical_for_lanes` | preserve |
| stateful adapters/defaults | `iterator_laws.ph` | preserve/test |
| boundedness | `compiler/boundedness.rs` | preserve/test |
| List/Set/Map/Bytes/Range stateful iterators | source modules | preserve/remove cursor shims |
| String byte/codepoint stateful views | `scalar/string.ph` | preserve |
| generic missing-runtime-type erasure fallback | `typing/environment.rs::instantiate_type_ref` | inherited; do not reopen casually |

## 2.2 Already-landed T14 state

Current main already contains:

```text
Collection abstract native class
CoreClasses.collection_class
UniverseKey::Collection
concrete List/Map/Set/Tuple/Range/Bytes -> Collection
collections/collection.ph
collections/package.ph exposes .collection + .iterator_laws
collections/iterable.ph deleted
legacy UniverseKey::Iterable removed
```

These are now takeover state. T14 below verifies/completes them; it does not re-perform the same migration.

## 2.3 Confirmed remaining defects

| ID | Finding | Severity | Owning task |
|---|---|---:|---|
| `AUD-P3-01` | Inline semantic support includes only `iterator_laws` + `range` | blocking | T12/T13 |
| `AUD-P3-02` | `compile_for_lanes` still emits cursor selectors | blocking | T13 |
| `AUD-P3-03` | expansion uses `for_iteration_lanes` opportunistically, then dynamic `iterator`/`next` sends | blocking | T12 |
| `AUD-P3-04` | VM cursor-protocol probe already removed; fast lane is product-only | completed/verify | T12/G5 |
| `AUD-P3-05` | List/Map/views/Set/Range/Bytes/Tuple retain `iteratorValue`; Range retains `iterate` | blocking | T13 |
| `AUD-P3-06` | Tuple lacks visible canonical Iterable vertical and still calls deleted `ZippedIterable` | blocking/architectural risk | T13 |
| `AUD-P3-07` | RefRange changed to `Collection` but still only implements cursor traversal | blocking | T13 |
| `AUD-P3-08` | Set/Map canonical integration tests remain ignored under stale baseline reasons | blocking evidence gap | T13/T15 |
| `AUD-P3-09` | AST/LSP/current specs/guides still teach cursor semantics | stabilization gap | T15 |
| `AUD-P3-10` | P5 handoff/checkpoint lifecycle prose is stale relative to current partial T14 state | record gap | T15 |

---

# 3. Required reads before implementation

Always read:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/luna-patch-grade-plan-schema.md

docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md

docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-plan-00-shared-header.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-plan-01-semantic-and-for-cutover.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-plan-02-builtins-and-iterator-surface.md
LANG005.C6.P5-plan-03-retirement-and-stabilization-requirements-analysis.md
LANG005.C6.P5-walkthrough.md
LANG005.C6.P5-handoff.md
```

Normative reads:

```text
docs/specs/iterator.md
docs/specs/callables/arguments.md
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/objects/traits.md
docs/specs/objects/impl.md
```

Task-local live reads:

```text
phalcom-core/src/modules/compile.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/call.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/boundedness.rs
phalcom-core/src/vm/dispatch.rs

phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/collection.ph
phalcom-core/core/universe/src/collections/{list,map,set,tuple,range,bytes}.ph
phalcom-core/core/universe/src/scalar/string.ph
examples/sheetcalc/src/grid/ref-range.ph

phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-native-meta/src/universe.rs
phalcom-modules/src/builtin.rs
```

Testing conventions:

```text
phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
phalcom-core/tests/core/language/iteration.rs
phalcom-core/tests/core/collections/outgoing_packs.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
```

Avoid a broad repository scan except at the explicit T13/T15 census steps.

---

# 4. Normative authority

## 4.1 Canonical iteration law

```text
Iterator:
    type Item
    next -> Option<Item>
    iterator -> self
    stateful / single-pass / sticky exhaustion

Iterable:
    type Item
    type Iterator
    Iterator |= Iterator
    Iterator.Item == Item
    iterator -> fresh traversal for reusable sources

Derived:
    I |= Iterator => I |= Iterable
```

## 4.2 `for`

```text
nominal/evidence-based participation
all sources evaluated once left-to-right
all iterators acquired once left-to-right after source evaluation
next once per lane per classified attempt
None = exhaustion
Some(payload) = yielded item, including payload None
strict multi-lane zip
same-function direct-jump break/continue
```

## 4.3 Positional `*`

`docs/specs/callables/arguments.md` requires:

```text
Unit  -> zero positional values
Tuple -> Tuple positional lane only
Iterable -> yielded items in iteration order
other -> invalid positional expansion source
```

Operand evaluates once; iterator failures/control effects propagate normally; expansion finishes before target activation.

## 4.4 Runtime ancestry

The user-approved current runtime hierarchy is:

```text
Object
 └── Collection (abstract)
      ├── List
      ├── Map
      ├── Set
      ├── Tuple
      ├── Range
      └── Bytes
```

`Collection` is not a substitute for `Iterable` trait evidence.

## 4.5 Historical behavior that is not authority

Do not preserve as production architecture:

```text
iterate(_)
iteratorValue(_)
class Iterable
class Iterator is Iterable
SourceIterator
IndexedIterable
ZippedIterable
cursor tokens
structural “responds to iterator selectors” tests
runtime Iterable ancestry as conformance
```

---

# 5. Takeover stable-interface map

| Fact/interface | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| trait identity | `iterator_laws.ph::{Iterator,Iterable}` | Universe/semantic | unchanged |
| generic evidence | `GenericConformanceEvidence` | semantic | exact/assumed/derived remain distinct |
| `for` semantic site | `CallableAnalysis.for_iteration_lanes` | semantic | every accepted lane gets product |
| `for` executable site | `ModuleLoweringSemantics.for_iteration_lanes` | core lowering | compiler never re-proves |
| invocation specs | `ForIterationRequirementSpec` | lowering/compiler | selected or requirement only |
| Option branch | `JumpIfNone` + `GetVariantPayload(0)` | compiler/VM | payload not sentinel |
| expansion direct probe | `PackTryExpandTuplePositionals` / `ListTryExpandTuplePositionals` | compiler/VM | Unit/Tuple product lane only |
| expansion boundedness | `require_exhaustible` | compiler | Unbounded reject, Unknown allowed |
| runtime taxonomy | `CoreClasses.collection_class` | runtime/native metadata | no conformance meaning |
| stateful iteration | ordinary source-authored iterator classes | Universe + heap tracing | no cursor tokens/native heap arm |

---

# 6. Architecture

## 6.1 Shared implicit-iteration semantic resolver

Part III should leave one proof algorithm used by both `for` and positional expansion:

```text
source expression/type
    ↓
canonical Iterable declaration identity
    ↓
prove Source |= Iterable
    ↓
normalize Iterable::Item + Iterable::Iterator
    ↓
prove projected Iterator |= Iterator
    ↓
resolve Iterable.iterator + Iterator.next requirements
    ↓
retain exact/assumed/derived evidence + dependencies
```

Published site products remain distinct:

```text
ForIterationLaneResolution
PositionalExpansionIterationResolution
```

Do not encode “site kind” by looking in the wrong map and falling back dynamically.

## 6.2 Lowering

```text
semantic for product ----------------> for_iteration_lanes
semantic expansion product ----------> positional_expansions
                                       ↓
                            selected/requirement invocation specs
                                       ↓
                                    compiler
```

If `ForIterationRequirementSpec` is reused by expansion, either keep it as a private historical name or mechanically rename to a neutral `ImplicitIterationRequirementSpec`. Do not fork the executable ABI.

## 6.3 Inline semantic support

`EntrySelection::Inline` has an implicit Universe runtime but currently analyzes only source-authored iterator laws + Range. Part III must make the **semantic** workspace see all source-authored conformance contributions required by the canonical built-ins used in inline programs.

Preferred shape:

```text
build synthetic entry
    ↓
add curated canonical Universe semantic-support modules
    ↓
resolve their import closure
    ↓
analyze workspace
    ↓
runtime_modules remains {synthetic entry only}
```

Do not turn support modules into runtime initialization units merely to make proof visible.

The exact support set is verify-first. Expected owners include:

```text
collections.iterator_laws
collections.list
collections.map
collections.set
collections.range
collections.bytes
scalar.string
collections.tuple         once Tuple canonical conformance exists
```

If including these source modules introduces unrelated known Universe diagnostic debt, curate the smallest exact support closure rather than weakening semantic validation.

## 6.4 Compiler end state

Production `for`:

```text
semantic success
    => every lane has ForIterationLoweringSpec
    => canonical lowering only
```

No `compile_for_lanes` cursor compatibility branch survives.

Generic positional expansion:

```text
source once
Unit/Tuple fast probe
else dedicated PositionalExpansionLoweringSpec required
iterator once
next loop
outer Option branch
append Some payload
```

No ordinary dynamic `iterator` or `next` send is a conformance fallback.

## 6.5 Runtime end state

The VM:

- knows how to copy Unit/Tuple positional lanes;
- invokes bytecode-selected methods/environments;
- branches on Option representation;
- owns no iteration protocol discovery;
- owns no conformance search;
- treats Collection only as class ancestry.

---

# 7. Ownership boundaries

## Owns

Part III owns:

- positional-expansion iteration evidence/lowering;
- inline implicit-Universe semantic support required by iteration;
- removal of residual compiler cursor fallback;
- Tuple retirement-readiness closure;
- RefRange migration;
- removal of residual cursor selectors/use sites;
- verification/completion of Collection/native/public-name transition;
- current docs/LSP/source-comment terminology;
- final P5 cursor/name census and implementation records.

## Does not own

Part III does not own:

- redesign of generic evidence;
- changing the meaning of Dynamic;
- trait specialization/precedence;
- new Tuple type theory without consultation;
- new Range semantics;
- making String itself Iterable;
- runtime trait search;
- iterator-specific native heap representation;
- general REPL semantic architecture modernization;
- historical ADR rewriting;
- broad workspace cleanup/certification.

---

# 8. Global Part-III invariants

`INV-III-01` — canonical Iterator/Iterable declaration identities remain unchanged.

`INV-III-02` — Collection ancestry never proves Iterable conformance.

`INV-III-03` — every executable generic iteration site is backed by semantic evidence.

`INV-III-04` — compiler never discovers conformance by selector presence.

`INV-III-05` — VM never proves/searches conformance.

`INV-III-06` — no production `for` cursor compatibility lowering remains after T13.

`INV-III-07` — Unit/Tuple positional expansion stays a direct product-lane special case.

`INV-III-08` — expansion source expression evaluates exactly once.

`INV-III-09` — expansion obtains iterator exactly once and `next` once per attempted item.

`INV-III-10` — outer Option `None` alone is exhaustion; `Some(None)` remains data.

`INV-III-11` — boundedness remains conservative and compiler-owned.

`INV-III-12` — stateful iterator/adaptor state remains in ordinary GC-traced fields.

`INV-III-13` — reusable source freshness and iterator aliasing semantics remain unchanged.

`INV-III-14` — Map direct Item remains labeled `(key: K, value: V)`.

`INV-III-15` — Range keeps the current forward integer +1 subset only.

`INV-III-16` — String itself remains non-Iterable.

`INV-III-17` — direct-jump loop control/captures/fiber suspension remain intact.

`INV-III-18` — public Iterator/Iterable names denote traits only.

`INV-III-19` — no compatibility `Iterable` runtime class is restored.

`INV-III-20` — historical cursor documents may remain historical; current-facing docs may not teach cursor semantics.

---

# 9. Non-goals

Do not expand this part into:

```text
new collection taxonomy beyond approved Collection
Sequence/List hierarchy redesign
general Tuple type-system redesign
new Dynamic capability model
new trait-object/existential representation
new runtime conformance registry
new compiler optimization/fusion architecture
new Range step/descending semantics
new String character abstraction
new iterator adapters beyond the accepted Part-II surface
workspace-wide documentation modernization
workspace formatting/clippy cleanup
release certification
```

---

# 10. Expected impact map

## 10.1 T12

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs / body.rs as publication requires
phalcom-semantic/src/checker/statement.rs or shared iteration helper
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/db/fingerprint.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/scope.rs if invocation helper refactor needed
phalcom-core/src/vm/dispatch.rs             verify/minimal fast-lane changes only
phalcom-core/src/error.rs                   only if NonIterableStarOperand becomes dead
```

Tests:

```text
phalcom-semantic/tests/semantic/capabilities/iteration.rs
new focused semantic expansion test module or existing canonical call/argument location
phalcom-core/tests/core/collections/outgoing_packs.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
phalcom-core/tests/core/language/iteration.rs
```

## 10.2 T13

```text
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/modules/compile.rs          final inline support adjustments

phalcom-core/core/universe/src/collections/{list,map,set,tuple,range,bytes}.ph
phalcom-core/core/universe/src/collections/iterator_laws.ph  only if zipped bridge/helper needed
phalcom-core/core/universe/src/scalar/string.ph              census/only if necessary
examples/sheetcalc/src/grid/ref-range.ph
```

## 10.3 T14

Mostly verify-current plus residual corrections:

```text
phalcom-core/core/universe/src/collections/collection.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-native-meta/src/universe.rs
phalcom-modules/src/builtin.rs
reflection/source-index tests if affected
```

## 10.4 T15

```text
phalcom-ast/src/ast.rs
phalcom-lsp/src/hover.rs
current docs/specs/guides named by census
.agents/skills/language-design/phalcom/overlay.md
Sheetcalc current docs/comments
LANG005.C6-CHECKPOINT.md
LANG005.C6.P5-walkthrough.md
LANG005.C6.P5-handoff.md
```

Unexpected adjacent touches are allowed for mechanical dependencies. Entering a new semantic/runtime authority subsystem is consultation territory.

---

# 11. Implementer decision authority

## 11.1 FIXED

Do not change:

- semantic proof ownership;
- Collection-vs-Iterable separation;
- canonical trait identities;
- no-runtime-proof rule;
- stateful lifecycle/sticky exhaustion;
- Option branch semantics;
- strict zip behavior;
- Map Item shape;
- Range subset;
- String non-Iterable status;
- Unit/Tuple `*` product-lane semantics;
- direct-jump `for` architecture;
- final removal of cursor fallback/selectors.

## 11.2 MECHANICALLY FLEXIBLE

May adapt:

- exact helper names;
- whether shared implicit-iteration resolver lives in `checker/iteration.rs` or an existing checker module;
- exact semantic product map field names;
- exact neutral rename of `ForIterationRequirementSpec`;
- curated inline support helper decomposition;
- test fixture names;
- exact internal compiler error used for missing lowering metadata;
- source placement of `TupleIterator` / `RefRangeIterator` consistent with access rules.

## 11.3 VERIFY-FIRST

| Assumption | Verify | Response |
|---|---|---|
| current HEAD still equals/descends from `206e0ab` | git preflight | adapt commits; architecture drift => stop |
| `has_iteration_protocol` remains absent | `rg` + dispatch | if present, remove in T12 |
| expansion still has dynamic iterator/next fallback | `expr.rs` | if already fixed, verify tests and skip edits |
| inline support still only iterator_laws/range | `compile.rs` | if expanded, test then delete fallback |
| Tuple lacks canonical conformance | `tuple.ph` + semantic snapshot | if already landed, verify instead |
| ZippedIterable has no live definition | source census | if live canonical replacement exists, reuse |
| RefRange remains cursor-only | source | if already migrated, verify |
| Collection native graph is complete | native/core files | patch residual mechanical misses |
| Set/Map ignored tests are stale | run explicitly | pass => unignore; fail => blocking classification |

---

# 12. Global STOP / CONSULT triggers

Use the shared protocol plus these specific triggers:

1. Tuple canonical `Item` still cannot be represented without changing `Dynamic` semantics.
2. arbitrary Dynamic `*source` must remain legal but no precomputed evidence token exists.
3. inline canonical support requires accepting unrelated semantic errors or running the whole Universe under an unreviewed baseline.
4. a proposed fix adds iteration behavior to `Collection`.
5. a proposed fix restores `Iterable` as runtime class/module.
6. a valid `for` cannot publish canonical lane evidence under the current trait system.
7. expansion requires dynamic selector coincidence or VM trait search.
8. removing a cursor selector breaks a current non-iteration API with real normative authority.
9. `Tuple.zipped` has semantics incompatible with the landed strict-zip iterator and the existing specs do not decide the difference.
10. REPL effective trait completion requires a material new semantic service.
11. stale tests conflict with current normative specs in a way that changes user-observable behavior.
12. the same semantic failure survives one serious correction.

A trigger cannot be self-waived.

---

# 13. Debugging budget

## Mechanical failures

Up to three coherent cycles:

```text
inspect -> concrete cause -> one correction -> smallest test
```

## Semantic failures

Before editing, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

One serious correction per underlying semantic problem. If it persists, consult.

## Architectural failures

Zero speculative patches. Consult immediately.

Never rerun an unchanged failing command without a relevant code/state change.

---

# 14. Testing surface and coverage ledger

| Coverage | Invariant(s) | Required discriminator | Owner |
|---|---|---|---|
| `P3-CV-FOR-01` | 01–06 | inline List `for` publishes canonical lane | T12/T13 |
| `P3-CV-FOR-02` | 01–06 | inline Bytes `for` publishes canonical lane | T12/T13 |
| `P3-CV-FOR-03` | 06,17 | parallel List lanes canonical, strict mismatch preserved | T13 |
| `P3-CV-FOR-04` | 06 | disassembly/no selector census for `for` | T13/G5 |
| `P3-CV-EXP-01` | 03–05 | dedicated semantic expansion product | T12 |
| `P3-CV-EXP-02` | 03–05 | exact concrete Iterable expansion | T12 |
| `P3-CV-EXP-03` | 03–05 | generic assumed Iterable expansion | T12 |
| `P3-CV-EXP-04` | 03–05 | Iterator source via derived Iterable | T12 |
| `P3-CV-EXP-05` | 10 | Item containing None not confused with exhaustion | T12 |
| `P3-CV-EXP-06` | 07 | Unit/Tuple direct positional lane unchanged | T12 |
| `P3-CV-EXP-07` | 03–05 | structural iterator/next lookalike rejected | T12 |
| `P3-CV-EXP-08` | 11 | proven unbounded rejected; Unknown allowed | T12 |
| `P3-CV-EXP-09` | 08–09 | source once / iterator once / next once | T12 |
| `P3-CV-SRC-01` | 13 | RefRange row-major fresh/sticky | T13 |
| `P3-CV-SRC-02` | 01–05 | Tuple canonical conformance/iterator | T13 |
| `P3-CV-SRC-03` | 13 | Tuple.zipped no deleted legacy type | T13 |
| `P3-CV-SRC-04` | 14 | Map labeled tuple still intact | T13 |
| `P3-CV-SRC-05` | 15 | Range canonical behavior after cursor removal | T13 |
| `P3-CV-RT-01` | 02,18–19 | Collection ancestry exists, legacy Iterable identity absent | T14 |
| `P3-CV-RT-02` | 02 | Collection lookalike not automatically Iterable | T14 |
| `P3-CV-NAME-01` | 18 | Iterator/Iterable definition identity = traits | T14/T15 |
| `P3-CV-TOOL-01` | 20 | LSP hover/current docs canonical | T15 |
| `P3-CV-TEST-01` | all | Set/Map stale ignores removed or blocking | T13/T15 |
| `P5-CV-58` | 04–06 | no cursor sends in loop/expansion paths | G5/G6 |
| `P5-CV-59` | 04–05 | no runtime trait proof | G5/G6 |
| `P5-CV-64` | 03 | repository custom source migrated | T13 |
| `P5-CV-65` | 06,19 | final production cursor census clean | G6 |
| `P5-CV-66` | 18 | public canonical trait names | G5 |
| `P5-CV-112` | 03–10 | `*` canonical evidence | T12/G5 |
| `P5-CV-113` | 02,19 | runtime legacy Iterable identity absent | T14/G5 |
| `P5-CV-114` | 20 | LSP/AST current terminology | T15 |
| `P5-CV-115` | 20 | only historical cursor text remains | T15/G6 |

---

# 15. Verification execution budget

## BUILD MODE

Mandatory only when relevant:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion
```

Use exact test-name filters while implementing when possible.

## STABILIZE MODE at G5

Add:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-native-meta
```

## STABILIZE MODE at G6

Run G5 plus directly affected LSP/REPL tests if touched, language-corpus iterator/collection slices, final census, and `git diff --check`.

## Explicitly do not run during ordinary BUILD

```text
cargo test --workspace
cargo clippy --workspace
full workspace formatter rewrite
broad unrelated language corpora
all examples
release/perf suites
```

Those do not discriminate the active retirement defects efficiently.

---

# 16. Baseline / unrelated failure policy

Classify:

```text
A — definitely caused by Part III: fix
B — probably caused by Part III: investigate/fix
C — unclear: one bounded classification pass; record/defer if nonblocking
D — clearly unrelated baseline: record and continue
```

Special rule for this plan: a failure containing `iterate(_)` / `iteratorValue(_)` in a current production iteration/expansion path is presumptively **A** until proven otherwise because cursor retirement is the core objective.

Do not add new ignores. Remove stale ignores when their recorded blocker is gone.

---

# 17. Execution graph and recovery order

Although T14 has partially landed, stable task numbering/order remains:

```text
current partial-T14 state
        |
        v
T12  canonical expansion + shared implicit-iteration support
        |
        v
T13  for fallback + Tuple/RefRange/source consumer closure
        |
        v
T14  verify/finish Collection/native/public retirement
        |
        v
G5   one-architecture retirement gate
        |
        v
T15  tooling/docs/final census/records
        |
        v
G6   final focused P5 stabilization
```

Do not revert partial T14 in order to make this graph look chronological.

Recommended reviewer-sized patch shape:

```text
patch 1  T12 semantic expansion product + shared resolver
patch 2  T12 inline semantic support + expansion lowering/compiler cutover
patch 3  T13 remove for cursor fallback + canonical lane regressions
patch 4  T13 Tuple decision/implementation + zipped migration
patch 5  T13 RefRange + residual cursor selector removal + stale test ignores
patch 6  T14 runtime/public identity verification corrections
patch 7  T15 tooling/docs/current-source cleanup
patch 8  G6 records/final census
```

Tuple may force a consultation between patches 3 and 4.

---

# 18. T12 — First-class canonical iteration for positional expansion

## Purpose

Make non-Unit/non-Tuple positional `*` expansion consume the same canonical Iterable/Iterator proof architecture as `for`, while publishing its own site product and removing all dynamic selector fallback.

## Preconditions

- canonical P4 traits exist;
- Part-I `for` evidence product/lowering exists;
- VM tuple/Unit product-lane probe is currently product-only;
- boundedness service is live;
- partial T14 hierarchy stays in place.

## Consumes

```text
ForIterationLaneResolution proof semantics
ForIterationRequirementResolution evidence representation
ForIterationRequirementSpec executable forms
runtime trait environment materialization helpers
JumpIfNone / GetVariantPayload
require_exhaustible
PackTryExpandTuplePositionals / ListTryExpandTuplePositionals
```

## Produces

```text
shared internal implicit-iteration resolver
PositionalExpansionIterationResolution index
fingerprint/dependency coverage
PositionalExpansionLoweringSpec map
canonical expansion compiler path with no dynamic iterator/next fallback
expanded inline Universe semantic support sufficient for built-in iteration
```

## Files and symbols

**Read:**

```text
phalcom-semantic/src/checker/{analysis,statement,call,context,body}.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-core/src/modules/{semantic_lowering,compile}.rs
phalcom-core/src/compiler/lib/{expr,scope}.rs
phalcom-core/src/compiler/boundedness.rs
phalcom-core/src/vm/dispatch.rs
```

**Modify expected:** same paths, narrowly.

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/iteration.rs
new/appropriate semantic argument-expansion capability tests
phalcom-core/tests/core/collections/outgoing_packs.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
phalcom-core/tests/core/language/iteration.rs
```

## T12-A — Classify expansion semantics before editing diagnostic ownership

Build a matrix from current specs/tests:

| Source | Expected eligibility |
|---|---|
| Unit | direct valid |
| Tuple | direct valid, positional lane only |
| concrete List/Set/etc | canonical Iterable |
| generic `T where T |= Iterable` | canonical assumed evidence |
| generic `I where I |= Iterator` | derived Iterable evidence |
| structural `iterator`/`next` only | invalid |
| known Int | invalid |
| Dynamic | verify normative behavior; consult if runtime acceptance required |

Do not change runtime error timing merely because one implementation is easier.

### Local STOP/CONSULT

Stop immediately if `Dynamic` is explicitly required to defer iterability validation to runtime and no already-ratified evidence token exists.

## T12-B — Extract shared implicit-iteration proof helper

Refactor Part-I semantic proof so both `Statement::For` and expansion analysis call one internal routine.

Required proof sequence:

```text
canonical trait identities
Source |= Iterable
associated Item normalization
associated Iterator normalization
projected Iterator |= Iterator
iterator requirement resolution
next requirement resolution
exact / assumed / derived evidence retention
```

Do not alter `GenericConformanceEvidence` meanings.

### Required publication

Add conceptual product:

```rust
PositionalExpansionIterationResolutionIndex
    = BTreeMap<ExpressionId, PositionalExpansionIterationResolution>
```

Store only successful executable resolutions, exactly like `for` lanes.

### Fingerprinting

Hash semantic content, not addresses:

```text
source expression identity
source/item/iterator types
conformance key / trait identity
requirement identity
exact/assumed/derived evidence semantic fingerprint
selected requirement template identity
```

Reuse existing `SemanticDependency` categories.

## T12-C — Repair Inline implicit-Universe semantic support

Current code explicitly filters to:

```text
collections.iterator_laws
collections.range
```

Replace the ad-hoc two-module filter with one helper that loads the source-authored semantic support required by implicit canonical iteration.

Requirements:

1. include actual conformance-owner modules needed by supported built-ins;
2. resolve their import closure;
3. keep them semantic-only for Inline runtime materialization;
4. do not pull unrelated full-Universe diagnostic debt if a curated closure suffices;
5. pin List and Bytes lane publication in tests;
6. preserve the existing Range assertion that support modules are not rematerialized into runtime initialization.

The exact curated set is verify-first. Start from current conformance owners, not class names guessed from this plan.

### Forbidden approaches

- no `if source.is(Collection)` semantic shortcut;
- no synthetic fake conformance records;
- no hard-coded `List` element type inference in compiler;
- no suppression of semantic diagnostics from added modules without accepted baseline authority.

## T12-D — Add dedicated expansion lowering projection

Extend `ModuleLoweringSemantics` with a dedicated map, for example:

```rust
pub positional_expansions: BTreeMap<SourceRange, PositionalExpansionLoweringSpec>
```

Projection must reuse the same selected/requirement conversion and runtime environment construction used by `for`.

If the executable enum is renamed to a neutral type, update `for` and expansion together mechanically and keep bytecode/runtime behavior unchanged.

## T12-E — Cut compiler expansion to required canonical metadata

In `compile_positional_expansion`:

Preserve:

```text
boundedness check
source local
iterator local
item local
Unit/Tuple direct probe
outer Option branch
append target abstraction
```

Change:

```text
OLD:
    spec = for_iteration_lanes.get(expr_range)
    if no spec -> dynamic `iterator` send / dynamic `next` send

NEW:
    spec = positional_expansions.get(expr_range)
    missing required spec -> compiler/internal semantic-lowering error
    emit preselected iterator
    emit preselected next
```

Do not use ordinary method dispatch as a conformance fallback.

## T12-F — Verify VM fast lane stays product-only

`try_append_tuple_positionals` should remain roughly:

```text
if source has Unit/Tuple positional lane:
    copy lane -> target
    true
else:
    false
```

No trait/class/selector detection.

Re-audit `NonIterableStarOperand` after semantic ownership is clear. Remove only if genuinely dead.

## T12 tests to add/modify

Required:

1. exact List expansion through evidence;
2. generic assumed Iterable expansion;
3. generic Iterator-derived Iterable expansion;
4. structural lookalike rejected;
5. `Some(None)`-like item data survives;
6. Unit empty lane;
7. Tuple positional lane excludes labels;
8. List-literal spread and outgoing pack use same generic semantics;
9. source side effect once;
10. proven unbounded rejected; Unknown allowed;
11. inline List/Bytes canonical lane publication to close shared support regression.

## Tests to run now

During implementation, use exact filters. At coherent T12 completion:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
```

Do not broaden after PASS.

## T12 acceptance

T12 passes when:

- expansion has a dedicated semantic product;
- expansion has dedicated lowering metadata;
- no `for_iteration_lanes` range hack remains in `compile_positional_expansion`;
- no dynamic `iterator`/`next` fallback remains;
- VM probe is product-only;
- inline List/Bytes evidence publication is working;
- all T12 discriminator tests pass or a documented Dynamic-semantics consultation blocks continuation.

## Checkpoint update

Record the new positional-expansion semantic/lowering product and the inline semantic-support rule as durable interfaces.

---

# 19. T13 — Close all remaining cursor consumers and repository sources

## Purpose

Remove the last live compiler/source dependencies on cursor semantics and make every surviving traversal source canonical before final retirement verification.

## Preconditions

- T12 canonical expansion is complete;
- inline semantic support now sees built-in conformance owners;
- partial T14 Collection hierarchy remains live.

## Produces

```text
no compiler cursor for fallback
Tuple final canonical disposition
Tuple.zipped canonical implementation/no deleted dependency
RefRange canonical stateful iterator
no concrete collection cursor selectors
stale Set/Map iteration ignores resolved
pre-G5 source census clean
```

## T13-A — Delete `compile_for_lanes` compatibility architecture

Once inline support is fixed, rerun the user-supplied failing group first. The six `iterate(_)` failures should disappear through canonical lane publication—not through restored methods.

Then change `compile_for` so:

```text
accepted lane(s) + lowering specs -> canonical lowering
missing expected spec            -> compile/internal projection failure
```

Delete `compile_for_lanes` and any cursor-only scratch helpers/constants that become dead.

### Required negative guard

Add a compiler/lowering test that would fail if a future missing lane silently reintroduces ordinary selector fallback.

### Forbidden

- do not send dynamic `iterator`/`next` as fallback;
- do not use `Collection` ancestry;
- do not compile semantically invalid sources into dNU behavior.

## T13-B — Resolve Tuple canonical iteration

First re-audit current semantic capability for heterogeneous associated Item.

Target if architecture supports it:

```phalcom
class TupleIterator { ... next -> Option<Dynamic> ... }
impl CanonicalIterator for TupleIterator { type Item = Dynamic }
impl CanonicalIterable for Tuple {
  type Item = Dynamic
  type Iterator = TupleIterator
  iterator { TupleIterator.new(self) }
}
```

This shape is illustrative only. Do not force `Dynamic` through static proof machinery if current semantics reject it by design.

### Mandatory consultation seam

If `type Item = Dynamic` still cannot be published without weakening the established `Dynamic is not static proof` rule, STOP AND CONSULT with:

```text
current Tuple semantic type
attempted associated Item declaration
exact checker/proof rejection
why union/erasure/other existing type form does or does not fit
normative Tuple traversal expectation
```

No compiler special case is allowed.

## T13-C — Migrate `Tuple.zipped`

Current source calls deleted `ZippedIterable.new(self)`.

Audit historical/current Tuple.zipped semantics. Prefer a canonical implementation over existing stateful iterator machinery, likely `StrictZipIterator` or a thin source/view that owns canonical iterators.

Acceptance:

- no reference to deleted `ZippedIterable`;
- strict length mismatch semantics preserved where applicable;
- no cursor tuple state;
- stored iterator/view lifecycle follows canonical one-shot/fresh-source rules.

If `Tuple.zipped` semantics are materially different from binary `Iterator.zip`, consult before changing user-visible behavior.

## T13-D — Migrate Sheetcalc RefRange

Implement:

```text
RefRange.iterator -> fresh RefRangeIterator
RefRangeIterator.next -> Option<Ref>
impl Iterator for RefRangeIterator
impl Iterable for RefRange
```

Preserve row-major normalized rectangle traversal.

Delete `iteratorValue` and stale inheritance comments.

Test:

- 1-cell;
- multi-row/multi-column order;
- fresh independent iterators;
- sticky exhaustion;
- `for` over RefRange if its package test harness supports it.

## T13-E — Remove residual cursor methods from core sources

Delete after canonical proof:

```text
List.iteratorValue
Map.iteratorValue
MapKeysView.iteratorValue
MapValuesView.iteratorValue
MapEntriesView.iteratorValue
Set.iteratorValue
Range.iterate
Range.iteratorValue
Bytes.iteratorValue
Tuple.iteratorValue
```

Update stale source comments in the same patch.

Audit `Entry` in `map.ph`; remove only if no independent current consumer exists.

## T13-F — Resolve stale ignored Set/Map tests

Run the ignored tests explicitly by exact test name before changing annotations.

If green:

- remove `#[ignore]`;
- remove obsolete “generic call-entry recipe” baseline reason;
- keep the stronger full `for`/view assertions.

If not green:

- classify as A/B unless proven unrelated;
- one serious correction;
- persistent semantic failure => consult;
- G5 remains blocked.

## T13-G — Pre-G5 production source census

Run focused search similar to:

```sh
rg -n 'iteratorValue\(|iterate\(|compile_for_lanes|SourceIterator|IndexedIterable|ZippedIterable|class Iterator is Iterable|class .* is Iterable' \
  phalcom-core/src \
  phalcom-core/core/universe/src \
  phalcom-semantic/src \
  examples \
  -g '!target/**'
```

Classify every hit. Production code hits are not allowed into G5.

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
```

Add the narrow Sheetcalc/fixture command matching its actual registration rather than running every example.

## T13 acceptance

- all focused canonical `for` tests pass without cursor fallback;
- Tuple disposition is architecturally resolved and tested;
- `Tuple.zipped` no longer references deleted legacy type;
- RefRange is canonical;
- no core source cursor selectors remain;
- stale Set/Map retirement-related ignores are removed or G5 is explicitly blocked;
- source census contains no live production cursor consumer.

## Checkpoint update

Record Tuple decision/consultation, RefRange migration, and compiler cursor fallback deletion as durable closure facts.

---

# 20. T14 — Complete and verify runtime/public retirement with `Collection`

## Purpose

Treat the already-landed T14 hierarchy work as takeover state, audit it for completeness, repair only residual inconsistencies, and certify that runtime ancestry no longer encodes iterability.

## Preconditions

- T12/T13 consumer migration complete;
- no production source/compiler cursor dependency remains.

## T14-A — Verify native/runtime graph

Audit:

```text
phalcom-native-meta/src/universe.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/vm/bootstrap.rs
```

Require:

```text
Object -> Collection
Collection abstract
List/Map/Set/Tuple/Range/Bytes -> Collection
no iterable_class
no UniverseKey::Iterable
no legacy Iterable relation row
metaclass parallel relation valid
```

Do not reparent collections directly to Object; the user-approved `Collection` intermediary supersedes the older Part-III default.

## T14-B — Verify Universe module/public topology

Require:

```text
collections/collection.ph exists
collections/iterable.ph absent
collections/package.ph exposes .collection + .iterator_laws, not .iterable
builtin node/source provider has collection, not iterable
Iterator/Iterable public definitions come from iterator_laws
```

## T14-C — Verify `Collection` carries no hidden iteration semantics

`collection.ph` should remain effectively an abstract common runtime class, not a compatibility protocol host.

Regression idea:

```phalcom
class Fake is Collection { }
```

must not become canonically Iterable merely because of ancestry.

Use semantic evidence tests rather than runtime dNU as the primary oracle.

## T14-D — Reflection/source-index/public identity

Where current tooling exposes declaration identity, verify:

```text
Iterable -> canonical trait declaration
Iterator -> canonical trait declaration
Collection -> class declaration
```

No same-scope ambiguity or compatibility alias with distinct identity.

## T14-E — Native metadata baseline

The T14 handoff says semantic-diagnostics baseline shifted mechanically for `is Collection`. Verify the current baseline reflects only approved source movement/semantic state; do not blanket-update diagnostics to hide new canonical conformance errors.

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-native-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
```

## T14 acceptance

- current Collection graph is complete and internally consistent;
- legacy native Iterable identity/module/public class is absent;
- no iteration semantics live on Collection;
- public Iterator/Iterable identities are canonical traits;
- no current production code requires the deleted class/module.

## Checkpoint update

Correct any stale wording that still says T14 is pending. Record that the `Collection` hierarchy is the final runtime taxonomy and trait conformance is the final iteration authority.

---

# 21. G5 — Canonical retirement gate

## Purpose

Prove the repository has **one** live production iteration architecture before documentation-only/final stabilization work.

## Required source census

```sh
rg -n 'iteratorValue\(|iterate\(|compile_for_lanes|has_iteration_protocol|SourceIterator|IndexedIterable|ZippedIterable|iterable_class|UniverseKey::Iterable' \
  phalcom-core/src \
  phalcom-core/core/universe/src \
  phalcom-semantic/src \
  phalcom-native-meta/src \
  phalcom-modules/src \
  examples \
  -g '!target/**'
```

Production hits must be zero or specifically unrelated to the retired protocol.

## Required focused commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-native-meta
```

Add exact Tuple/RefRange focused tests if they live outside those filters.

## G5 must prove

1. user-supplied List/Bytes/parallel `iterate(_)` failures are gone because canonical lane products publish;
2. no compiler cursor fallback exists;
3. expansion has dedicated evidence/lowering and no dynamic conformance fallback;
4. VM fast lane remains Unit/Tuple product-only;
5. exact/assumed/derived expansion works;
6. structural lookalikes are rejected;
7. Tuple final decision is implemented/approved;
8. RefRange canonical traversal works;
9. Set/Map stale ignores are resolved;
10. Collection graph is final and legacy Iterable identity absent;
11. no VM/compiler semantic proof search exists.

## On failure

Classify before editing. Do not broaden testing. A failure exposing a missing cursor consumer is Part-III responsibility, not baseline.

Do not proceed to T15 with a failed G5 unless the user/architect explicitly records an accepted blocker and explains why docs stabilization is still safe.

---

# 22. T15 — Current-facing tooling/docs, final census, stabilization, records

## Purpose

Make every current-facing surface describe the architecture that G5 proved, remove stale test/baseline language, perform final production/historical classification, and close P5 records.

## T15-A — AST/compiler/source comments

Update current comments that still say `for` lowers to cursor selectors.

At minimum audit:

```text
phalcom-ast/src/ast.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/expr.rs
core Universe source comments
iteration tests/disassembly comments
```

Production comments should describe stable semantics, not “P5 temporary migration” narration.

## T15-B — LSP current help

Update `phalcom-lsp/src/hover.rs` hard-coded `for` help:

Required meaning:

```text
for consumes canonical Iterable via a stateful Iterator
break/continue remain same-function loop control
```

Do not mention deleted selectors.

If LSP definition/completion tests expose Iterator/Iterable identity, pin trait declarations.

## T15-C — Current docs/guides/specs

Audit/update at least:

```text
docs/spec/current/iteration.md
docs/spec/current/control-flow.md
docs/guide/collections.md
docs/guide/control-flow.md
docs/spec/collections/06-rest-spread-and-pack-operators.md
docs/wiki/collections/traversal.md              if current-facing
examples/sheetcalc current docs
.agents/skills/language-design/phalcom/overlay.md
```

Where `docs/specs/iterator.md` is the newer normative authority, current/guide text should align with it rather than creating a second slightly different model.

Update expansion docs to retain:

```text
* Unit -> empty positional lane
* Tuple -> positional lane
* Iterable -> canonical iterator traversal
```

## T15-D — Historical documents

Do **not** rewrite:

```text
accepted cursor ADR decision bodies
old implementation plans/walkthroughs
historical audits/perf logs
raw archival wiki snapshots
```

Optional narrow supersession notes are okay if repository convention supports them.

## T15-E — REPL audit

Inspect current REPL completion/snapshot architecture.

If it simply walks runtime inheritance, verify that removal of `Iterable` does not falsely promise trait-default completion. Update stale docs that show `List -> Iterable -> Object`.

If current product requirements demand effective trait-default completion and it is now missing, STOP AND CONSULT before designing a REPL-specific trait solver or semantic service expansion.

## T15-F — Final cursor/name census

Run repository-wide current/historical classification searches:

```sh
rg -n 'iterate\(_|iteratorValue\(|class Iterable|class Iterator is Iterable|SourceIterator|IndexedIterable|ZippedIterable|iterable_class|UniverseKey::Iterable|cursor protocol' \
  . \
  -g '!target/**'
```

Classify each remaining hit:

```text
historical accepted
historical superseded
negative regression
current bug/stale docs
unrelated terminology
```

Every current bug/stale-doc hit must be resolved before G6.

## T15-G — Final test-state hygiene

Audit for retirement-related ignores/skips/baseline comments. No stale ignore may remain merely because it predates the generic environment fix.

Do not remove unrelated baseline records.

## T15-H — Durable records

Update `LANG005.C6-CHECKPOINT.md` with:

- T12/T13/T14/T15 completion;
- G5/G6 outcomes;
- Collection final hierarchy;
- expansion product interface;
- Tuple decision;
- RefRange migration;
- deferred unrelated failures;
- consultations;
- next action P6.

Update `LANG005.C6.P5-walkthrough.md` and create/finalize `LANG005.C6.P5-handoff.md`.

## Tests to run during T15

Only affected tooling tests while editing:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp
```

Run REPL tests only if REPL source/docs behavior was touched or the audit proves a functional compatibility obligation.

Do not run workspace tests.

## T15 acceptance

- no current-facing cursor description remains;
- historical hits are intentionally classified;
- LSP current help is canonical;
- operational language-design overlay is canonical;
- stale retirement-related ignores are gone;
- checkpoint/walkthrough/handoff truthfully match current source and verification.

---

# 23. G6 — Final focused P5 stabilization

## Purpose

Prove the complete P5 migration is coherent across semantic evidence, lowering, runtime execution, expansion, built-ins, retirement, tooling, and records without claiming workspace/release certification.

## G6 command set

Re-run only if relevant files changed since G5; otherwise reuse G5 evidence and run the deltas.

Core focused baseline:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion
```

Part-II surface/boundedness/fixture slice: run the current registered iterator/collection fixture tests named by the Part-II handoff and any tests changed in T13.

Tooling if changed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp
```

Affected crate checks:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-native-meta
```

Hygiene:

```sh
git diff --check
```

Final production census must be clean.

## G6 expected state

```text
P5 status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

unless a recorded non-P5 baseline blocker prevents truthful promotion. Do not claim release/workspace certified.

---

# 24. Final focused acceptance matrix

| Obligation | Evidence | Required result |
|---|---|---|
| canonical `for` semantic authority | semantic iteration tests | PASS |
| inline List/Bytes canonical lanes | new lane tests | PASS |
| no `for` cursor fallback | source/disassembly census | PASS |
| expansion semantic product | semantic expansion tests | PASS |
| expansion exact/assumed/derived | core/semantic tests | PASS |
| Unit/Tuple expansion lane | outgoing pack tests | PASS |
| invalid structural source | negative semantic test | PASS |
| unbounded expansion | boundedness regression | PASS |
| Map labeled Item | iteration tests | PASS |
| Set stateful full path | unignored test | PASS |
| Map stateful full path | unignored test | PASS |
| Range stateful path | iteration tests | PASS |
| String views | iteration tests | PASS |
| Tuple final path | T13 test | PASS / approved amendment |
| Tuple.zipped replacement | T13 test/census | PASS |
| RefRange canonical path | focused Sheetcalc test | PASS |
| Collection native graph | invariants/checks | PASS |
| no legacy runtime Iterable identity | code census | PASS |
| canonical public names | definition/source-index test | PASS |
| no current cursor docs/help | final census | PASS |
| checkpoint/walkthrough/handoff | record review | COMPLETE |

---

# 25. Performance/resource evidence

Part III is not a performance optimization plan, but retirement must avoid obvious regressions in architecture shape.

Required qualitative evidence:

- no new per-step cursor allocation; canonical iterators already own state;
- no runtime conformance search added to hot paths;
- no native synchronous traversal loop added for expansion;
- no duplicate semantic proof at compiler/runtime;
- inline support modules are semantic-only and not runtime-initialized;
- no adapter-specific native heap arms.

Do not add benchmark ceremony unless a focused regression appears.

---

# 26. Checkpoint bookkeeping

## At Part-III start

Record:

```text
local HEAD/status
partial T14 takeover state
user-supplied post-T14 iteration failure signature
known stale ignored Set/Map tests
known Tuple/RefRange blockers
```

## During Part III

Update checkpoint only for:

- T12 semantic/lowering interface;
- Dynamic expansion consultation if triggered;
- Tuple decision/consultation;
- T13 cursor fallback/source closure;
- G5 result;
- T14 final Collection graph verification;
- deferred failures;
- G6 result.

## At completion

- mark P5 complete only if G6 passes truthfully;
- record exact verification state;
- set next plan/action to P6 or the accepted next checkpoint action;
- remove stale “active T14” wording.

---

# 27. Walkthrough deliverable

Update/create:

```text
LANG005.C6.P5-walkthrough.md
```

Required sections:

1. final P5 outcome;
2. Part-I canonical evidence/`for` architecture;
3. Part-II built-ins/adapters/defaults/boundedness;
4. T14 partial-cutover incident and why `iterate(_)` resurfaced;
5. inline semantic-support repair;
6. positional expansion semantic/lowering product;
7. compiler cursor fallback removal;
8. Tuple final decision;
9. RefRange migration;
10. final Collection/native/public hierarchy;
11. tooling/docs changes;
12. tests actually run;
13. deferred/unrelated failures;
14. final census;
15. residual risks.

Do not write that a test “passed” unless it was actually run in the implementation session or inherited from a specifically identified verified gate with no relevant changes afterward.

---

# 28. Handoff deliverable

Finalize:

```text
LANG005.C6.P5-handoff.md
```

It must contain:

```text
current revision/worktree status
P5 final lifecycle state
canonical trait identities
implicit iteration semantic products for for + expansion
Collection hierarchy
stateful iterator/default surface
Tuple disposition
RefRange disposition
boundedness rule
public-name identity
remaining deferred failures
next P6 objective
must-read files
first recommended commands
things P6 must not redesign
```

Remove obsolete instructions that still say T14 steps are pending when they are already landed.

---

# 29. Completion truth table

Keep these states distinct:

```text
source edited
focused test added
focused test passed
G5 retirement gate passed
T15 current-facing cleanup complete
G6 focused P5 gate passed
P5 implemented
C6 complete
workspace/release certified
```

Only the first seven are candidates for this part. P5 completion does not imply C6 or release completion.

---

# 30. Plan self-review

Before handing this plan to Luna, verify:

- [x] stable T12–T15/G5–G6 IDs preserved;
- [x] partial T14 current state represented instead of replaying stale steps;
- [x] user-supplied six-failure signature has a source-grounded root cause;
- [x] compiler cursor fallback deletion is explicit;
- [x] positional expansion current architecture reflects the already-removed VM protocol probe;
- [x] expansion receives its own semantic/lowering product;
- [x] inline Universe semantic support is an explicit prerequisite;
- [x] Tuple/Dynamic uncertainty is a mandatory consultation seam, not guessed architecture;
- [x] RefRange halfway migration is explicitly repaired;
- [x] stale Set/Map ignores are explicitly resolved;
- [x] Collection is preserved as user-approved runtime taxonomy without becoming conformance authority;
- [x] current docs/LSP/agent overlay are in final cleanup scope;
- [x] historical documents are not rewritten indiscriminately;
- [x] test execution budget is narrow during BUILD and coherent at gates;
- [x] broad workspace certification is deferred;
- [x] no compatibility bridge can become final architecture;
- [x] checkpoint/walkthrough/handoff obligations are concrete.

This plan is execution-ready subject to live local preflight and the explicitly named Tuple/Dynamic and Dynamic-expansion consultation seams.
