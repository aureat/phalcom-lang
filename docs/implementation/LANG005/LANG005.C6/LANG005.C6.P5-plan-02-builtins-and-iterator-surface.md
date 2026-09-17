---
id: LANG005.C6.P5.PART-II
parent_plan: LANG005.C6.P5
shared_header: LANG005.C6.P5-plan-00-shared-header.md
previous_part: LANG005.C6.P5-plan-01-semantic-and-for-cutover.md
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan-part
part: 2-of-3
planning_cycle: restructured-pass-3
file_pass: 3-of-4
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
next_part: LANG005.C6.P5-plan-03-retirement-and-stabilization.md
executor_profile: GPT-5.6-Luna-High-or-Extra-High
---

# LANG005.C6.P5 Part II — Built-in Stateful Traversal and Production Iterator Surface

## Current execution state — 2026-09-17

Part II is in progress. The canonical stateful iterator vertical is focused-
tested for `Bytes`, `StringByteSequence`, and `StringCodePointSequence`.
Stateful `Set`, `Map`, and Map-view implementations are present, but direct
runtime construction reaches the inherited generic call-entry recipe failure.
Tuple remains blocked because the live semantic model cannot publish
`Dynamic` as `Iterator::Item`; Range remains deferred after its focused path
reproduced the inherited `Option<Number>` call-entry boundary. G3 is therefore
not passed, and T9–T11 are intentionally not started.

> **For Luna / agentic implementers:** read `LANG005.C6.P5-plan-00-shared-header.md` and the completed Part-I implementation record before executing this file. The header owns plan-wide architecture, invariants, scope exclusions, debugging budgets, verification doctrine, baseline policy, consultation rules, and checkpoint-record obligations. Part I owns the semantic/lowering/`for` architecture and must already have passed G2.
>
> **Do not execute Part III retirement work from this file.** Part II deliberately leaves the legacy runtime `Iterable` superclass, the remaining cursor selectors outside already-migrated production consumers, VM positional-expansion probing, and public-name retirement in place until Part III. Part II's job is to remove the *need* for those facilities from canonical built-in traversal and the production API surface, not to delete the compatibility substrate globally.
>
> **Required execution sub-skill:** use the repository's task-by-task implementation workflow (`subagent-driven-development` or `executing-plans`, where available). Each T-task is one reviewer-sized coherent patch unit. Do not convert the task into edit-sized test rituals; follow the shared BUILD/STABILIZE/CERTIFY doctrine.

**Goal:** migrate every P5-owned built-in source and view to real stateful `Iterator` objects, make the canonical `Iterator`/`Iterable` traits production-ergonomic through source-authored defaults, replace cursor-stateless lazy descriptors with one-shot adapter objects, preserve accepted eager collection conveniences without selector ambiguity, and align the existing boundedness analyzer with the canonical surface.

**Part-II terminal condition:** after T11 and the final G4 execution, every core source/view owned by P5 can participate in canonical traversal without cursor state; `Iterator` implementers normally provide only `Item + next`; lazy transformations are one-shot stateful adapters owned by `Iterator`; non-conflicting consumers/terminals are owned by `Iterable`; accepted direct eager built-in transforms survive through thin inherent forwarding; `.iter` is a direct alias for `.iterator`, not `SourceIterator`; and compiler boundedness recognizes the canonical surface without becoming a trait solver.

---

# 0. Part-II authority, scope, and executable ordering

This file is the patch-grade execution authority for T6–T11 and G3–G4 only.

Authority order:

```text
normative language specs + accepted user rulings
    ↓
LANG005.C6.P5 requirements analysis, final pass 3/3
    ↓
LANG005.C6.P5-plan-00-shared-header.md
    ↓
this Part-II file
    ↓
approved consultation/amendment recorded in LANG005.C6-CHECKPOINT.md
```

The shared header's §11.2 block lists the Part-II scope as:

```text
T6, T7, T8, G3, T9, T10, G4, T11
```

That list is a **scope inventory**, not the executable gate order. The header's cross-part rule and gate table both define G4 as the **terminal Part-II gate** and explicitly say G4 proves boundedness is coherent. Therefore this file executes:

```text
T6  Set / Tuple / Bytes / String views
T7  Map labeled-entry migration
T8  Range stateful migration
G3  built-in source migration gate
T9  stateful lazy adapters + Iterator defaults
T10 Iterable terminals + concrete eager forwarding
T11 canonical boundedness update
G4  final Part-II iterator/default/boundedness gate
```

Task/gate numbers remain unchanged. This is a gate-order clarification, not a semantic amendment.

Part II must not reopen the Part-I decisions for:

```text
ForIterationLaneResolution ownership
semantic proof and projection normalization
derived Iterator => Iterable evidence transport
ForIterationLaneLoweringSpec
compiler selected/requirement invocation helpers
Option exhaustion/payload representation
for source/iterator evaluation order
direct-jump break/continue architecture
strict multi-lane mismatch semantics
ZipLengthMismatchError identity
```

If implementing T6–T11 requires changing any of those, STOP AND CONSULT rather than silently changing Part I.

---

# 1. Repository re-grounding performed for this planning pass

The repository was re-audited specifically for built-in traversal, API ownership, adapter lifecycle, test migration, and boundedness. The planning baseline remains:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   06e37b6830cd9114d03f2a794d0b9a973f8cdc82
subject:    feat(lang005): complete P4 iterator iterable law staging
```

The following live facts materially refine the older monolithic plan and are binding unless local implementation HEAD has changed.

## 1.1 The legacy runtime superclass still owns almost the entire generic collection API

`phalcom-core/core/universe/src/collections/iterable.ph` currently contains:

```text
class Iterable<T> is Object
    iterate(_)
    each(_)
    map(_)
    map(indexed:)
    each(indexed:)
    flatMap(_)
    filter(_)
    contains/includes
    isEmpty
    indexed
    all/any/none
    count/count(where:)
    find/index
    join/join(_)
    fold/reduce
    group/partition
    toSet/toMap/toMap(merging:)/toList
    iter -> SourceIterator
```

and then the legacy cursor-shaped classes:

```text
IndexedIterable
ZippedIterable
class Iterator is Iterable
SourceIterator
MapIterator          # mapping adapter, not Map source traversal
FilterIterator
SkipIterator
TakeIterator
TakeWhileIterator
FlatMapIterator
```

Those adapters are explicitly stateless traversal descriptors whose per-traversal state lives in caller-threaded cursors. That architecture is superseded by P5 even though older COLL004 E-spec prose still describes it as intentional.

## 1.2 Historical iterator fixtures encode the architecture P5 is replacing

The current `language-corpus::iterator` fixture directory includes tests with names such as:

```text
iterator_flatmap_order_and_repeatability
iterator_iter_is_lazy_and_idempotent
iterator_filter_map_order
iterator_skip_take_takewhile
iterator_direct_map_still_eager
```

The direct-eager/lazy split remains relevant, but **pipeline repeatability does not**. P5's fixed lifecycle is:

```text
stored adapter object = one-shot stateful Iterator
second terminal on same adapter = still exhausted
restart = reacquire source.iterator/source.iter and rebuild adapters
```

Therefore old repeatability expected files are migration inputs, not acceptance authorities.

## 1.3 Part I's preferred built-in conformance module is the natural Part-II extension point

Part I prefers:

```text
phalcom-core/core/universe/src/collections/iterator_impls.ph
```

for `ListIterator<T>` and the canonical `List<T> |= Iterable` / `ListIterator<T> |= Iterator` conformances. Part II should extend the **actual path landed by Part I**, not create a second conformance registry/module.

At Part-II preflight, record:

```text
actual ListIterator declaration path
actual built-in conformance module path
actual imports used to name Iterator/Iterable
whether package.ph / UNIVERSE_NODES already contain iterator_impls
```

All subsequent tasks use that real path.

## 1.4 Source-owned iterator classes are preferable when native/internal access is module-scoped

Several target sources expose enough public API for an external iterator class:

```text
Set    -> size + at(_)
Tuple  -> size + at(_)
Bytes  -> size + at(_)
```

but three important families need source-local access or source-local semantic logic:

```text
Map    -> _$keyAt / _$valueAt are @internal
Range  -> _$lower / _$upper / _$upperInclusive are @internal and validation timing matters
String -> _$byteCount / _$byteAt plus private UTF-8 stepping logic
```

Therefore Part II must not force all concrete iterator classes into one implementation module merely for aesthetic uniformity.

Preferred placement rule:

```text
public-access index-backed iterator
    -> may live in iterator_impls.ph or the source module

iterator requiring internal/source-private mechanics
    -> declare the concrete class in the owning source module
       and place only canonical trait impl declarations in iterator_impls.ph
```

Do not add public native accessors solely so a central iterator module can reach them.

## 1.5 A separate iterator-adapter module is not the default plan

The monolithic plan suggested a likely new `iterator_adapters.ph`. This pass identified a module-cycle hazard:

```text
iterator_laws.ph
    needs concrete adapter classes in default bodies

adapter classes
    need Iterator constraints / generic next dispatch

iterator_adapters.ph importing iterator_laws.ph
    + iterator_laws.ph importing iterator_adapters.ph
    -> likely source-module cycle
```

The patch-grade default for T9/T10 is therefore:

```text
extend the existing canonical identity module
phalcom-core/core/universe/src/collections/iterator_laws.ph
```

with:

- `Iterator` lazy default bodies;
- canonical adapter classes;
- adapter `impl Iterator` declarations;
- `Iterable` terminal/convenience defaults.

This keeps the P4 trait declaration identity unchanged and avoids introducing a cyclic module graph.

A separate adapter module is mechanically allowed **only if Luna first proves an acyclic dependency shape on the live module system** and does not duplicate trait identities. Do not create a new module and discover the cycle after writing all adapters.

## 1.6 Map's current production behavior confirms the migration is observable

Current `map.ph` has:

```text
Map.iteratorValue(cursor)       -> key only
Map.entries                     -> MapEntriesView
MapEntriesView.iteratorValue    -> Entry.new(key, value)
```

P5 intentionally replaces both direct Map traversal and entries-view traversal with:

```text
(key: K, value: V)
```

using the ordinary labeled Tuple/product representation.

Current `Map.contains` / `includes` are inherent key-membership operations and remain so.

## 1.7 String remains deliberately non-Iterable

Current `String` has two explicit traversal views:

```text
string.bytes      -> StringByteSequence
string.codePoints -> StringCodePointSequence
```

Neither is currently a canonical `Iterable`. `String` itself remains non-Iterable because byte traversal and code-point traversal are intentionally distinct.

Important live detail:

```text
String._$byteAt -> Option<Int>
StringByteSequence.at(_) delegates to _$byteAt
```

Canonical byte iteration must yield `Int`, not `Option<Int>`. A byte iterator therefore must extract the `Some` payload at an in-range index rather than wrapping the public view's current Option-shaped `at` result in another `Some`.

Code-point iteration similarly yields the current integer scalar value; P5 does not invent `Char`.

## 1.8 The boundedness subsystem already exists and is intentionally syntactic

`phalcom-core/src/compiler/boundedness.rs` currently owns:

```rust
enum Boundedness { Bounded, Unbounded, Unknown }
enum IterationMode { Concrete, LazyPipeline, Unknown }
struct SourceFacts { mode, boundedness }
```

It recognizes `.iter`, legacy lazy adapter selectors, Range shapes, and full-source exhaustors. It rejects only provably `Unbounded`; `Unknown` remains legal.

T11 updates this existing compiler-only heuristic. It does **not** move boundedness into semantic trait evidence and does not use selector presence as proof of `Iterable`.

## 1.9 Concrete `isEmpty` needs an explicit preservation decision

The runtime superclass currently supplies:

```phalcom
isEmpty { self.size == 0 }
```

but the final canonical `Iterable` default surface intentionally does **not** put destructive `isEmpty` on all Iterators.

Therefore T10 must preserve `isEmpty` only on reusable concrete source families that accepted it through the legacy superclass. This is easy to miss when moving algorithms to trait defaults.

## 1.10 Part II should not destructively edit the legacy superclass

Part III still owns repository legacy subclasses and global runtime-class retirement. Removing methods from `collections/iterable.ph` during T10 can break those remaining consumers before Part III migrates them.

Part II therefore:

```text
ADDS canonical defaults and built-in forwarding
MIGRATES P5-owned source/view traversal
MIGRATES public iterator fixtures to one-shot semantics
DOES NOT delete the legacy superclass or globally remove its methods
```

Part III performs the destructive class/cursor deletion once all remaining consumers are migrated.

---

# 2. Required preflight before T6

Do not begin source edits until Part I's handoff has been inspected in the local checkout.

Record in `LANG005.C6-CHECKPOINT.md` or the active implementation notes:

```text
local HEAD
G2 result
ForIterationLaneResolution path
ForIterationLaneLoweringSpec path
ListIterator path
built-in conformance module path
ZipLengthMismatchError path
BL-P5-01 classification
any approved Part-I deviations
```

Run the minimum Part-I sanity check only if implementation work is starting in a fresh checkout:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
```

If G2 was already executed in the same verified worktree and no relevant files changed, do not rerun it mechanically.

Then inspect current source ownership:

```sh
rg -n 'class (List|Set|Map|Tuple|Bytes|Range)|String(Byte|CodePoint)Sequence|iteratorValue|iterate\(' \
  phalcom-core/core/universe/src/collections \
  phalcom-core/core/universe/src/scalar/string.ph

rg -n 'SourceIterator|IndexedIterable|ZippedIterable|MapIterator|FilterIterator|FlatMapIterator|SkipIterator|TakeIterator|TakeWhileIterator' \
  phalcom-core/core/universe/src/collections

rg -n 'iterator_flatmap_order_and_repeatability|iterator_iter_is_lazy_and_idempotent|language-corpus.*iterator' \
  phalcom-core/tests
```

This is a verification census, not permission to delete the hits.

---

# 3. Part-II fixed architecture and invariants

The shared header invariants remain in force. The following are the Part-II-specific invariants Luna should keep visible while editing.

## INV-II-01 — reusable source and iterator are different lifecycle categories

```text
reusable source:
    source.iterator -> fresh traversal object every access

Iterator:
    iterator -> self
    next mutates traversal state
    once next returns None, future next calls remain None
```

## INV-II-02 — `Item + next` remains the normal custom Iterator burden

A user iterator should not be forced to implement:

```text
map
filter
count
fold
toList
...
```

The canonical traits supply derivable behavior.

## INV-II-03 — adapter objects hold iterators, never cursor tokens

No canonical adapter stores:

```text
(source, cursor)
(cursor tuple)
caller-threaded offset token
```

It stores upstream iterator objects and adapter-local state.

## INV-II-04 — no hidden move semantics

If:

```phalcom
const it = xs.iterator
const mapped = it.map(f)
it.next
mapped.next
```

both aliases observe one shared upstream traversal. P5 does not make iterator references affine.

## INV-II-05 — sticky exhaustion is stronger than Rust's base Iterator contract

No adapter may restart after returning `None`, even if:

- upstream later mutates;
- an adversarial custom upstream would yield again;
- another alias calls methods;
- a mapper/predicate would later produce values.

Early-stopping adapters carry explicit done state when required.

## INV-II-06 — direct collection transforms and lazy iterator transforms are intentionally different selectors by receiver role

```text
List.map(f)       -> eager accepted collection transform
List.iter.map(f)  -> lazy stateful Iterator adapter
```

Do not solve this with trait precedence/specialization.

## INV-II-07 — no eager map/filter/flatMap defaults on `Iterable`

Those selectors would overlap with lazy `Iterator` defaults because every Iterator derives Iterable.

## INV-II-08 — `.iter` is only an alias

```text
source.iter == semantic behavior of source.iterator
```

For a reusable source it returns that source's concrete fresh iterator. For an Iterator it returns the same traversal through derived Iterable evidence. No `SourceIterator` allocation.

## INV-II-09 — Map direct Item is a labeled tuple

```text
Map<K,V>::Item = (key: K, value: V)
```

No key-only compatibility lane and no iterator-specific pair class.

## INV-II-10 — String itself is not Iterable

Only `StringByteSequence` and `StringCodePointSequence` become canonical sources.

## INV-II-11 — Range semantics do not expand

P5 preserves the existing forward integer `+1` subset. No step, progression, descending, or lowerless semantics are invented.

## INV-II-12 — generic defaults are source-authored and use ordinary evidence

No new VM proof search, native combinator family, hidden runtime iterator registry, or adapter-specific heap variant.

## INV-II-13 — ordinary instance fields are the GC ownership model

Source, callback, upstream iterator, current inner iterator, phase flags, and buffered values live in normal traced fields.

## INV-II-14 — boundedness remains a conservative compiler service

It may recognize canonical syntax shapes, but it does not decide conformance.

---

# 4. Part-II task graph and patch boundaries

Executable graph:

```text
                     ┌───────────────┐
Part I G2 ──────────>│ T6 finite core│
                     └──────┬────────┘
                            v
                     ┌───────────────┐
                     │ T7 Map        │
                     └──────┬────────┘
                            v
                     ┌───────────────┐
                     │ T8 Range      │
                     └──────┬────────┘
                            v
                          G3
                            |
                            v
                     ┌───────────────┐
                     │ T9 adapters   │
                     └──────┬────────┘
                            v
                     ┌───────────────┐
                     │ T10 defaults  │
                     └──────┬────────┘
                            v
                     ┌───────────────┐
                     │ T11 bounded   │
                     └──────┬────────┘
                            v
                          G4
                            |
                            v
                         Part III
```

Recommended review/commit shape:

```text
commit 1  T6 finite core + String views
commit 2  T7 Map entries + views
commit 3  T8 Range
commit 4  G3 checkpoint note if repository workflow records gates separately
commit 5  T9 stateful adapter toolbox
commit 6  T10 Iterable defaults + concrete eager wrappers
commit 7  T11 boundedness
commit 8  G4 checkpoint/walkthrough boundary if workflow records gates separately
```

Do not combine T9 and T10 into one giant patch merely because both touch `iterator_laws.ph`. Their reviewer questions are different:

```text
T9: are lazy state machines correct?
T10: is API ownership/non-overlap correct?
```

---

# 5. Expected source impact map

Paths are architectural anchors; re-resolve after local checkout inspection.

## T6–T8 built-ins

```text
phalcom-core/core/universe/src/collections/iterator_impls.ph   # actual Part-I path
phalcom-core/core/universe/src/collections/set.ph
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/bytes.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/collections/range.ph
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/core/universe/src/collections/package.ph          # only if topology actually changes
phalcom-modules/src/builtin.rs                                 # only if topology actually changes
```

## T9–T10 canonical surface

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/iterator_impls.ph   # if adapter impls are kept there
phalcom-core/core/universe/src/collections/list.ph
phalcom-core/core/universe/src/collections/set.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/range.ph
phalcom-core/core/universe/src/collections/bytes.ph
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/core/universe/src/collections/iterable.ph         # read; avoid destructive edits
```

## T11 boundedness

```text
phalcom-core/src/compiler/boundedness.rs
```

## Focused tests

Prefer extending the Part-I stateful test module rather than creating many tiny Rust integration binaries:

```text
phalcom-core/tests/core/language/iteration_stateful.rs
phalcom-core/tests/core/language/mod.rs
phalcom-core/tests/fixtures/language/iterator/*
phalcom-core/tests/fixtures/language/sequence/*
phalcom-core/tests/fixtures/language/iteration/*
phalcom-core/tests/fixtures/language/collections/*
phalcom-core/tests/fixtures/language/bytes/*
phalcom-core/tests/fixtures/language/strings/*
phalcom-core/tests/core/collections/contract.rs                 # only if direct collection contract coverage belongs there
```

Do not move semantic proof tests into runtime fixtures. Part I owns semantic iteration evidence; Part II tests source-level execution, concrete associated Item behavior, default dispatch, lifecycle, GC, and boundedness.

---

# 6. Part-II coverage ownership

Part II owns or materially closes 58 final coverage rows. Some rows were partially established by Part I; Part II must not mark them complete until the surface/runtime half is proven.

| Coverage | Primary Part-II task | Required discriminator |
|---|---|---|
| `P5-CV-14` | T7 | indexed encounter ordinal remains independent of Map key/value representation |
| `P5-CV-33` | T6 | Set encounter order |
| `P5-CV-34` | T6 | Tuple Item remains Dynamic traversal |
| `P5-CV-35` | T6 | Bytes yields Int octets |
| `P5-CV-36` | T7 | direct Map yields labeled `(key,value)` tuple |
| `P5-CV-37` | T7 | generic Map K/V Item projection |
| `P5-CV-38` | T7 | Map pair destructuring in `for` |
| `P5-CV-39` | T7 | keys/values/entries views |
| `P5-CV-40` | T8 | Range exclusive bound |
| `P5-CV-41` | T8 | Range inclusive bound |
| `P5-CV-42` | T8 | upper-unbounded Range |
| `P5-CV-43` | T8 | missing-lower failure |
| `P5-CV-44` | T8 | lower > upper current failure behavior |
| `P5-CV-45` | T9 | `source.iter.map` construction is lazy |
| `P5-CV-46` | T9 | filter may consume many upstream items for one output |
| `P5-CV-47` | T9 | skip prefix executes once |
| `P5-CV-48` | T9 | take stops without extra upstream read |
| `P5-CV-49` | T9 | takeWhile sticky stop |
| `P5-CV-50` | T9 | flatMap current-inner retention |
| `P5-CV-51` | T10 | indexed ordinal adapter |
| `P5-CV-52` | T9 | existing zipped view strict mismatch uses canonical stateful iteration |
| `P5-CV-53` | T9 | GC with upstream/callback retained |
| `P5-CV-70` | T9/T10 | custom Iterator implements only Item+next and gains `for` + defaults |
| `P5-CV-71` | T10 | two `.iter` accesses on reusable source are independent |
| `P5-CV-72` | T9/T10 | same stored pipeline stays exhausted |
| `P5-CV-73` | T9/T10 | rebuilding pipeline from source restarts independently |
| `P5-CV-74` | T9 | adapter/upstream aliases share state |
| `P5-CV-75` | T10 | direct built-in map stays eager |
| `P5-CV-76` | T9 | Iterator map callback runs only during consumption |
| `P5-CV-77` | T9 | filterMap skips mapper None |
| `P5-CV-78` | T9 | flatten handles empty/multiple inners |
| `P5-CV-79` | T9 | chain switches once |
| `P5-CV-80` | T9 | skipWhile stops predicate after first retained |
| `P5-CV-81` | T9 | mapWhile sticky after mapper None |
| `P5-CV-82` | T9 | inspect side effect once/yield item unchanged |
| `P5-CV-83` | T10 | nth consumes exactly n+1 or exhausts |
| `P5-CV-84` | T10 | last returns final Some / empty None |
| `P5-CV-85` | T9 | pairwise zip is strict, never shortest |
| `P5-CV-86` | T9 | all adapters intrinsically fused/sticky; no public fuse |
| `P5-CV-87` | T6 | String bytes view works through generic Iterable bound |
| `P5-CV-88` | T6 | codePoints canonical; String itself non-Iterable |
| `P5-CV-89` | T7 | entries Item matches direct Map Item |
| `P5-CV-90` | T7 | nominal Entry use migrated or authority-justified |
| `P5-CV-91` | T8/T9/T11 | `(0..).iter.take(n).toList` terminates without Range size |
| `P5-CV-92` | T8 | no step/descending semantics invented |
| `P5-CV-93` | T9 | rich defaults add no VM proof/native-combinator floor |
| `P5-CV-94` | T9/T10 | one generic algorithm by role; eager wrappers forward |
| `P5-CV-95` | T9 | GC through nested flatMap/flatten current inner |
| `P5-CV-100` | T10 | generic `T |= Iterator` gets `.iter` through derived Iterable without ambiguity |
| `P5-CV-101` | T10 | arbitrary custom Iterable gets `source.iter.map(...).toList` |
| `P5-CV-102` | T10 | `List.map` eager vs `List.iter.map` lazy |
| `P5-CV-103` | T7/T10 | Map direct contains is key membership; iterator contains is entry membership |
| `P5-CV-104` | T10 | `.iter` does not allocate SourceIterator |
| `P5-CV-105` | T7/T10 | `map.entries.toMap` round-trip via tuple labels |
| `P5-CV-106` | T6 | byte sequence Item is Int |
| `P5-CV-107` | T6 | code-point sequence Item is integer |
| `P5-CV-108` | T9 | nested flatMap/flatten GC retains all state |
| `P5-CV-111` | T11 | boundedness recognizes canonical surface conservatively |

Rows not listed remain owned by Part I or Part III. Do not create duplicate fixtures merely to make a coverage number appear in this file.

---

# 7. Verification discipline for Part II

## BUILD mode

After each coherent T-task:

1. run the task's narrow Rust/unit/source-language filters;
2. run `cargo check` only for crates actually touched if compilation evidence requires it;
3. inspect full failure output;
4. classify failures before editing;
5. stop after the named focused checks pass.

Do not run the full workspace after T6, T7, T8, T9, T10, or T11.

## STABILIZE mode

Run G3 after T8 and G4 after T11.

Gates intentionally combine the minimum neighboring surfaces needed to detect integration defects.

## CERTIFY mode

Part III/G6 owns the final broad focused P5 certification. Part II does not steal that work.

## Test-authority rule

A legacy test that encodes cursor/stateless-pipeline semantics is **not** a reason to preserve those semantics. Migrate the fixture when P5's normative behavior changed.

Examples:

```text
old: same pipeline materializes twice identically
new: same pipeline exhausts once; a newly created pipeline repeats traversal

old: SourceIterator class identity is observable
new: source.iter is the source's concrete iterator

old: Tuple.zipped prechecks source.size
new: strict zip is driven by per-attempt next results
```

---

# 8. T6 — Migrate Set, Tuple, Bytes, and String traversal views

## Purpose

Expand the canonical stateful source/iterator model from Part I's `List` pilot to the remaining non-Map finite core families and the two explicit String traversal views.

T6 should establish a repeatable implementation template before Map and Range add family-specific semantics.

## Preconditions

- Part I G2 PASS or explicitly accepted non-blocking baseline classification;
- canonical `ListIterator` vertical works;
- actual built-in conformance module from Part I identified;
- no global legacy retirement has begun.

## Consumes

```text
Set.size / Set.at
Tuple.size / Tuple.at
Bytes.size / Bytes.at
String byte-count / byte-at / codePointAt / leadByteLen behavior
canonical Iterator / Iterable traits
ordinary instance field tracing
Part-I canonical for path
```

## Produces

Conceptual final source/iterator relationships:

```text
Set<T>
    Item = T
    Iterator = SetIterator<T>

Tuple
    Item = Dynamic
    Iterator = TupleIterator

Bytes
    Item = Int
    Iterator = BytesIterator

StringByteSequence
    Item = Int
    Iterator = StringByteIterator

StringCodePointSequence
    Item = Int/current integer numeric authority
    Iterator = StringCodePointIterator

String
    no Iterable conformance
```

Every reusable source/view returns a fresh iterator object.

## T6.1 — Extend the Part-I conformance topology, do not fork it

Use the exact module Part I landed for built-in conformances.

If Part I used `collections/iterator_impls.ph`, extend it. Do not create:

```text
iterator_impls2.ph
collection_iterators.ph
builtin_iterable_conformances.ph
```

for the same authority.

If a concrete iterator class needs source-private/internal access, declare the class in its source module and reference it from the conformance module.

## T6.2 — SetIterator

Preferred state:

```text
_source: Set<T>
_index: Int
_done: Bool
```

Conceptual `next`:

```text
if done -> None
if index >= source.size:
    done = true
    None
else:
    item = source.at(index)
    index += 1
    Some(item)
```

Requirements:

- preserve current Set encounter order;
- do not convert Set to List before traversal;
- do not introduce hash-table bucket iteration semantics different from `Set.at`;
- after first None, later Set mutation must not resurrect the iterator;
- two `set.iterator` values have independent `_index/_done`.

Keep the legacy `iteratorValue` method during transition; canonical `for` must no longer require it.

## T6.3 — TupleIterator

Preferred state:

```text
_source: Tuple
_index: Int
_done: Bool
```

Associated Item remains `Dynamic` in this checkpoint. Do not infer tuple-wide homogeneity from one concrete tuple literal.

The iterator uses the ordinary Tuple element order over both positional and labeled storage values exactly as current `Tuple.at`/`_$at` ordering defines it.

Do not redesign tuple labels or product typing.

## T6.4 — BytesIterator

Preferred state:

```text
_source: Bytes
_index: Int
_done: Bool
```

Associated Item is `Int`.

Use the public `Bytes.at`/native-backed current storage access. Do not allocate an intermediate List and do not use `toList`.

The current Bytes law about callbacks yielding/suspending remains relevant once generic `each` moves to the trait; T6 itself only proves raw iteration.

## T6.5 — StringByteSequence canonical iterator

Place `StringByteIterator` in `scalar/string.ph` unless the live module visibility model proves an external class can use all required String internals without widening visibility.

State should be minimal:

```text
_string or _view
_byteOffset
_done
```

Do **not** blindly write:

```phalcom
Some(_view.at(i))
```

because the live `StringByteSequence.at` delegates to `String._$byteAt`, which is `Option<Int>`.

Canonical `next -> Option<Int>` must perform one of the equivalent safe shapes:

```text
offset known in-range
read _$byteAt(offset)
extract Some(byte)
advance offset
return Some(byte)
```

or use the existing Option match/elimination API. An impossible `None` at an in-range byte offset is an internal invariant issue, not an iterator exhaustion signal to be silently double-wrapped.

Do not change the public `StringByteSequence.at` contract in T6 merely to simplify iterator code unless a current normative String spec separately requires it.

## T6.6 — StringCodePointSequence canonical iterator

State:

```text
_string or _view
_byteOffset
_done
```

Algorithm:

```text
if done -> None
if byteOffset >= byteCount:
    done = true
    return None

read code point at current lead-byte offset
advance by leadByteLen(current offset)
return Some(integer code point)
```

Do not scan from byte zero to recover the next position on every `next`. State is the current byte offset.

Do not invent a `Char` type.

If live String representation permits malformed internal UTF-8 such that `codePointAt` can return None at an in-range lead position, STOP AND CLASSIFY the existing String invariant before deciding whether that is an error or exhaustion. P5 does not define a new malformed-String policy.

## T6.7 — Canonical conformances

Add ordinary explicit conformances with associated Item/Iterator bindings.

Conceptually:

```text
Set<T> |= Iterable
SetIterator<T> |= Iterator

Tuple |= Iterable
TupleIterator |= Iterator

Bytes |= Iterable
BytesIterator |= Iterator

StringByteSequence |= Iterable
StringByteIterator |= Iterator

StringCodePointSequence |= Iterable
StringCodePointIterator |= Iterator
```

Do not write separate `Iterable` impls for iterator classes; they derive through P4.

Do not make `String |= Iterable`.

## T6.8 — Transitional method policy

Keep legacy source cursor methods until Part III if they are still used by compatibility code:

```text
Set.iteratorValue
Tuple.iteratorValue
Bytes.iteratorValue
legacy Iterable.iterate inheritance
```

String views currently have custom `each` and private cursor helpers. Keep them during T6 if T10 has not yet supplied the final default surface. T10/Part III can remove redundant local traversal conveniences when safe.

The canonical conformance is not inferred from the old superclass.

## T6.9 — Required tests

Extend `language::iteration_stateful` with direct tests, not only corpus goldens.

Minimum discriminators:

```text
p5_set_iterator_preserves_encounter_order
p5_set_iterators_are_independent_and_sticky

p5_tuple_iterator_walks_positional_and_labeled_storage_values_in_current_order
p5_tuple_iterator_item_is_accepted_as_dynamic_in_generic_body

p5_bytes_iterator_yields_int_octets
p5_bytes_iterator_sticky_after_source_mutation

p5_string_bytes_view_is_generic_iterable
p5_string_bytes_view_yields_int_not_option
p5_string_codepoints_view_yields_integer_scalars
p5_string_itself_is_not_iterable
p5_string_view_iterators_are_independent
```

For the negative String case, prefer semantic compile rejection or an existing generic-bound fixture. Do not rely on runtime dNU as proof that the type is non-Iterable.

Coverage:

```text
CV-33, CV-34, CV-35, CV-87, CV-88, CV-106, CV-107
```

## T6.10 — Tests to run now

First list the actual stateful module filter if needed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --list
```

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bytes
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
```

Run `language-corpus collections` only if T6 modified a shared collection behavior exercised there. Do not run iterator/sequence yet.

## T6.11 — Acceptance

T6 passes when:

```text
Set/Tuple/Bytes canonical iterators exist and are stateful
String bytes/codePoints are canonical reusable Iterable views
String itself is not Iterable
all canonical iterators are sticky
two iterator acquisitions are independent
no traversal copies the source
no new native iterator representation exists
legacy cursor methods are not used by production for
```

## T6.12 — Local STOP / CONSULT triggers

STOP if:

1. associated Item for String code points requires a new language type decision;
2. String byte/code-point iteration requires changing String representation;
3. an external iterator class cannot access required internals and the proposed fix is a new native/public primitive rather than source-local placement;
4. Tuple Item cannot remain `Dynamic` without a broad tuple-type redesign;
5. Set encounter ordering is discovered to be unspecified by newer authority and current tests conflict materially.

## T6.13 — Checkpoint note

Record only durable facts:

```text
canonical finite built-in iterator inventory
String remains non-Iterable
String views Item types
any source-local iterator placement decisions
```

---

# 9. T7 — Migrate Map to canonical labeled entry tuples and stateful views

## Purpose

Perform the intentional Map Item migration:

```text
old direct Map traversal -> key
old Map.entries          -> nominal Entry

new canonical direct Map traversal and entries
    -> (key: K, value: V)
```

while preserving insertion order, key membership APIs, map storage, equality, hashing, and mutation policy.

## Preconditions

- T6 green;
- canonical generalized `for` from Part I works;
- labeled tuple construction/access/destructuring already works independently;
- `ZipLengthMismatchError` and Part-I loop machinery are not touched.

## T7.1 — Run the real Entry usage census before editing

Search current live code/tests/examples, excluding historical design material first:

```sh
rg -n '\bEntry\b|Entry\.new|\.entries\b|entry\.key|entry\.value' \
  phalcom-core phalcom-semantic phalcom-modules phalcom-lsp examples tests \
  -g '!target/**'
```

Then separately inspect current docs/specs to classify which references are historical versus current authority:

```sh
rg -n '\bEntry\b|\.entries\b' docs/spec docs/decisions docs/implementation \
  -g '!docs/implementation/LANG005/LANG005.C6/**'
```

Do not preserve nominal `Entry` merely because older COLL001/COLL002 implementation plans describe it. P5's final requirements supersede it for iteration unless a newer normative spec requires nominal identity.

Record:

```text
production Entry constructors
public tests relying on Entry.is(...)
reflection/name tests
only property-access uses (.key/.value)
historical-only references
```

## T7.2 — Concrete Map iterator belongs with Map's internal slot access

Preferred declaration location:

```text
phalcom-core/core/universe/src/collections/map.ph
```

because `_$keyAt` / `_$valueAt` are internal.

Conceptual class:

```text
MapIterator<K,V>
    _map
    _index
    _done
```

`next -> Option<(key: K, value: V)>`:

```text
if done -> None
if index >= map.size:
    done = true
    return None

let i = index
index += 1
return Some((key: map._$keyAt(i), value: map._$valueAt(i)))
```

Construct the existing labeled product/Tuple. Do not allocate `Entry`.

After first `None`, mutation/appending another association must not resurrect this iterator.

## T7.3 — Direct Map canonical conformance

Add:

```text
Map<K,V> |= Iterable
    Item = (key: K, value: V)
    Iterator = MapIterator<K,V>

MapIterator<K,V> |= Iterator
    Item = (key: K, value: V)
```

The exact tuple type syntax must use the repository's canonical labeled-tuple type form; do not erase to `Dynamic` merely to make the conformance easy.

This row is an important generic typing discriminator:

```phalcom
fn consume<M>(m: M)
  where M |= Iterable,
        M::Item == (key: String, value: Int)
```

or the repository-native equivalent must be able to prove the K/V shape for `Map<String,Int>`.

## T7.4 — Map keys/values/entries views

Each view is a reusable source retaining the Map and returning a fresh stateful iterator.

Target relationships:

```text
MapKeysView<K,V>
    Item = K
    Iterator = MapKeysIterator<K,V>

MapValuesView<K,V>
    Item = V
    Iterator = MapValuesIterator<K,V>

MapEntriesView<K,V>
    Item = (key: K, value: V)
    Iterator = MapEntriesIterator<K,V>
```

Iterator state:

```text
_map
_index
_done
```

Views stay lightweight retained-source views; do not materialize Lists.

`MapEntriesIterator` may share implementation logic with `MapIterator` only if the sharing is source-level and does not introduce a second pair representation. Duplicating a three-line index state machine is preferable to inventing a new abstraction that obscures Item typing.

## T7.5 — Map direct membership remains representation-specific

Keep:

```phalcom
map.contains(key)
map.includes(key)
```

as key membership.

Later, after T10:

```phalcom
map.iter.contains((key: k, value: v))
```

is generic entry membership because `map.iter` is an Iterator of entry tuples.

Do not change direct Map membership to compare entry tuples.

## T7.6 — Pair patterns and label access

Required observable shapes:

```phalcom
for (key, value) in map { ... }

for entry in map {
  entry.key
  entry.value
}
```

The same shape must come from `map.entries`.

Part I already owns tuple/pattern lowering. T7 only supplies the new Item representation.

## T7.7 — `Entry` deletion/retention decision

Default outcome:

```text
delete Entry from the iteration path
delete the class entirely if the T7 census finds no independent current authority
```

A live use that only needs `.key` / `.value` is not a reason to preserve the class; labeled tuples provide those projections.

Retain nominal Entry only if a current, non-historical authority requires:

```text
Entry type identity
Entry construction API
Entry reflection
Entry-specific methods beyond key/value
```

If that occurs, STOP AND CONSULT because the final P5 requirements explicitly converge `Map.entries` on tuple Item unless newer authority supersedes it.

## T7.8 — `toMap` interoperability

T10's generic `toMap` defaults consume `.key` and `.value`. Ensure T7's labeled tuple shape supports:

```phalcom
map.entries.toMap
```

without converting back through a nominal Entry.

Duplicate-key behavior remains whatever `toMap` currently specifies; T7 does not redesign it.

## T7.9 — Required tests

Add or update:

```text
p5_map_iterator_yields_labeled_entry_tuple
p5_map_generic_item_preserves_key_and_value_types
p5_map_for_pair_destructuring
p5_map_entry_label_access
p5_map_keys_view_stateful
p5_map_values_view_stateful
p5_map_entries_view_matches_direct_map_item
p5_map_iterator_ordinal_independent_of_key_values
p5_map_iterator_sticky_after_insert
```

Later T10 adds membership and `toMap` default integration.

Coverage now:

```text
CV-14
CV-36..39
CV-89
CV-90
```

Stage `CV-103` and `CV-105` for final closure in T10.

## T7.10 — Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful map -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
```

If no `map` substring filter matches the stateful test names, use the exact `--list`-discovered prefix instead.

Do not run iterator/sequence broad lanes yet.

## T7.11 — Acceptance

T7 passes when:

```text
direct Map iterator Item is labeled tuple
generic associated Item preserves K and V
Map/entries agree on Item representation
keys/values views have typed stateful iterators
direct key membership unchanged
Map storage/order/equality/hash unchanged
Entry is removed or explicitly authority-justified
no full Map copy
no iterator-specific pair object
```

## T7.12 — Local STOP / CONSULT triggers

STOP if:

1. labeled tuple construction requires a new heap representation;
2. canonical tuple type syntax cannot preserve K/V without a generic-type-system redesign;
3. a current normative source requires nominal Entry identity;
4. changing direct Map Item breaks a newer authority rather than a legacy fixture;
5. internal-access rules would require a new native Map traversal primitive.

## T7.13 — Checkpoint note

Record the externally observable Item migration and the Entry decision.

---

# 10. T8 — Replace Range cursor traversal with stateful RangeIterator

## Purpose

Move Range traversal state out of the yielded-number cursor and into a real iterator object while preserving exactly the currently ratified forward integer semantics.

## Preconditions

- T7 green;
- Part-I canonical `for` works;
- current Range tests and any BL-P5-01 Range interaction are classified.

## T8.1 — Fixed semantic subset

P5 supports:

```text
a..b       forward +1, upper exclusive
a..=b      forward +1, upper inclusive
a..        forward +1, non-exhausting
```

P5 does not add:

```text
..b / ..=b iteration
..
descending traversal
Range.by(step)
negative step
zero step
Progression
non-integer traversal
```

Current failure behavior for unsupported forms remains catchable.

## T8.2 — Concrete state placement

Declare `RangeIterator` in `range.ph` unless the live internal-access rules prove otherwise.

Preferred state:

```text
_range
_started or _nextValue
_done
```

Do not use `Range.size` to drive iteration.

Keeping the Range object itself is usually simpler than snapshotting all bounds and preserves normal source lifetime/representation semantics. Snapshotting normalized scalar bounds is acceptable only if it is proven semantically equivalent under Range immutability and does not alter failure timing.

## T8.3 — Error timing

Acquiring:

```phalcom
const it = range.iterator
```

should not eagerly raise errors that current iteration raises only when traversal begins unless normative authority explicitly says otherwise.

Validate on first `next`:

```text
lower bound exists
lower is integral Number
upper, when present, is integral Number
lower <= upper when upper present
```

Then advance.

Do not preserve `Option.unwrapOr` merely because the old cursor code uses it. Prefer the ordinary `Option.match`/payload path already used elsewhere when it avoids the recorded generic-call baseline.

## T8.4 — Exact next state machine

Conceptually:

```text
if done:
    return None

on first next:
    validate bounds
    candidate = lower

on later next:
    candidate = previous + 1

if no upper:
    return Some(candidate)

if exclusive and candidate >= upper:
    done = true
    return None

if inclusive and candidate > upper:
    done = true
    return None

return Some(candidate)
```

The lower==upper edge is different for the two operators:

```text
a..a   -> immediate None
a..=a  -> Some(a), then None
```

Once None is returned, later calls remain None.

## T8.5 — Canonical conformance

```text
Range |= Iterable
    Item = Number
    Iterator = RangeIterator

RangeIterator |= Iterator
    Item = Number
```

No compiler special case for Range.

## T8.6 — Current Range APIs remain independent

Do not redesign:

```text
slice bounds
first
last
size
contains/includes
at
equality/hash/toString
```

A future Range iteration redesign should not be smuggled into P5 through cleanup.

## T8.7 — Required tests

Add:

```text
p5_range_iterator_exclusive
p5_range_iterator_inclusive
p5_range_iterator_equal_exclusive_empty
p5_range_iterator_equal_inclusive_one
p5_range_iterator_upper_unbounded_prefix
p5_range_iterator_missing_lower_raises_on_next
p5_range_iterator_non_integer_rejected
p5_range_iterator_descending_remains_unsupported
p5_range_iterator_sticky_after_bounded_exhaustion
```

Coverage:

```text
CV-40..44
CV-92
```

`CV-91` closes after T9/T10/T11 because it requires lazy `take(...).toList`.

## T8.8 — Tests to run now

Use discovered Range/stateful test filters. Minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful range -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
```

Run a dedicated Range corpus lane only if one exists on the local checkout; do not invent a command name.

## T8.9 — Acceptance

```text
Range state is iterator-owned
exclusive/inclusive/equal edges correct
a.. is genuinely unbounded
lowerless/descending/non-integer behavior not invented
sticky exhaustion
no Range.size dependency
no compiler Range special case
```

## T8.10 — Local STOP / CONSULT triggers

Any pressure to decide:

```text
step
descending
Progression
upper-only start semantics
new Range Item type
```

requires consultation.

---

# 11. G3 — Built-in source migration gate

## Purpose

Prove that all P5-owned built-in sources/views have canonical stateful traversal before the shared adapter/default surface is built on top of them.

G3 does **not** require global cursor deletion.

## Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bytes
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings
```

If a named lane is baseline-red for an unrelated pre-existing reason, classify it under the shared baseline policy; do not broaden debugging automatically.

## G3 code census

Inspect canonical source files:

```sh
rg -n 'iteratorValue|iterate\(' \
  phalcom-core/core/universe/src/collections/list.ph \
  phalcom-core/core/universe/src/collections/set.ph \
  phalcom-core/core/universe/src/collections/map.ph \
  phalcom-core/core/universe/src/collections/tuple.ph \
  phalcom-core/core/universe/src/collections/range.ph \
  phalcom-core/core/universe/src/collections/bytes.ph \
  phalcom-core/core/universe/src/scalar/string.ph
```

Hits may remain for transition compatibility. The gate question is not "are names deleted?" but:

```text
Does every P5-owned canonical Iterable conformance return a stateful Iterator whose next path is independent of those cursor methods?
```

## G3 PASS evidence

Record:

```text
List/Set/Tuple/Bytes/Map/Range iterator class paths
Map view iterator paths
String view iterator paths
all associated Item/Iterator bindings
Entry decision
Range non-feature boundary
focused test results
```

Do not proceed to T9 if a core source still requires cursor state for canonical `iterator`.

---

# 12. T9 — Implement the canonical one-shot lazy adapter toolbox on Iterator

## Purpose

Replace the legacy cursor-stateless lazy pipeline architecture with real stateful one-shot Iterator adapters, while preserving the ergonomic implementation rule:

```text
custom Iterator author writes Item + next
canonical Iterator defaults provide lazy transformation breadth
```

This is the most state-machine-sensitive task in Part II. Keep the patch coherent and the tests discriminating.

## Preconditions

- G3 PASS;
- all built-in sources can supply canonical iterators;
- ordinary trait defaults and generic trait methods remain available;
- `ZipLengthMismatchError` exists from Part I.

## T9.1 — Source placement: extend the canonical P4 identity module by default

Default file:

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
```

Add adapter classes and their `Iterator` impls there unless a proven acyclic split exists.

Why this is the default:

```text
Iterator defaults must construct adapter classes
adapter bodies must call generic upstream next
adapter conformances need Iterator constraints/projections
```

Keeping these in the same module avoids a source-module dependency cycle and preserves the P4 trait declaration identity.

Do not rename the module merely because it now contains production defaults.

## T9.2 — Canonical lazy defaults owned by Iterator

Target default selectors:

```text
map<U>
filter
filterMap<U>
flatMap<S/U as exact live syntax permits>
flatten
chain<J>
skip
skipWhile
take
takeWhile
mapWhile<U>
inspect
zip<J>
```

`iterator { self }` remains the primitive default already in P4.

Do **not** add terminal/query methods in T9; T10 owns them.

Do **not** add `iter` on Iterator. `iter` belongs to Iterable in T10, and Iterator receives it through the P4 derived conformance.

## T9.3 — Concrete adapter inventory and state

Use collision-free names. `MapIterator` is reserved for Map source traversal; use `MappedIterator` or equivalent for the mapping adapter.

Recommended semantic shapes:

| Adapter | Required retained state |
|---|---|
| `MappedIterator<I,U>` | upstream `I`, mapper |
| `FilterIterator<I>` | upstream, predicate |
| `FilterMapIterator<I,U>` | upstream, mapper |
| `FlatMapIterator<I,S>` | outer upstream, mapper, current inner iterator or absence |
| `FlattenIterator<I>` | outer upstream, current inner iterator or absence |
| `ChainIterator<I,J>` | first, second, phase/done |
| `SkipIterator<I>` | upstream, remaining prefix, initialized/done if needed |
| `SkipWhileIterator<I>` | upstream, predicate, stillSkipping, done |
| `TakeIterator<I>` | upstream, remaining, done |
| `TakeWhileIterator<I>` | upstream, predicate, done |
| `MapWhileIterator<I,U>` | upstream, mapper, done |
| `InspectIterator<I>` | upstream, callback |
| `StrictZipIterator<I,J>` | left, right, done |
| `IndexedIterator<I>` | upstream, ordinal |

Exact generic parameter counts may include explicit Item type parameters if live Phalcom source typing needs them. That is a mechanical representation choice; do not erase constraints to Dynamic to avoid stating associated projections.

## T9.4 — Generic constraints

The semantic requirements are fixed:

```text
upstream type conforms to Iterator
mapper/predicate signatures are typed from Iterator::Item
filterMap mapper returns Option<U>
flatMap mapper returns S where S |= Iterable
flatten requires current Item |= Iterable
chain second iterator Item == Self::Item
zip second iterator Item may differ if output pair preserves both item types
mapWhile mapper returns Option<U>
```

Use current method-generic syntax:

```text
map<U>(...)
```

and current callable type syntax from live Universe code, e.g. `(T) -> U`.

If one expansion-tier method cannot be expressed because a specific generic/associated constraint syntax is not implemented, isolate that method's blocker. Do not weaken the whole toolbox to Dynamic.

Consult only when fixing the syntax would be a cross-cutting type-system change.

## T9.5 — Map adapter

Laws:

```text
construction -> zero mapper calls
next:
    call upstream.next once
    None -> sticky None
    Some(x) -> mapper(x) once -> Some(y)
```

Do not implement via `upstream.next.map(...)` if doing so obscures sticky state or reintroduces the known generic-call baseline. Direct Option match is acceptable and often clearer.

## T9.6 — Filter adapter

`next` may consume many upstream items:

```text
loop:
    step = upstream.next
    None -> return None
    Some(x):
        predicate(x) exactly once
        true -> return Some(x)
        false -> continue
```

No buffered cursor/item after returning a yielded item.

After upstream None, store done if needed to protect against adversarial non-sticky custom upstreams.

## T9.7 — FilterMap adapter

```text
loop:
    upstream.next
    None -> sticky None
    Some(x):
        mapper(x) once
        mapper None -> continue
        mapper Some(y) -> Some(y)
```

The mapper's None means "skip this item", not "adapter exhausted".

## T9.8 — FlatMap adapter

State:

```text
outer
mapper
currentInner: Option<InnerIterator>
done
```

Algorithm:

```text
if current inner exists:
    inner.next
    Some(y) -> yield
    None -> clear current inner

advance outer once
None -> done, return None
Some(x):
    mapper(x) exactly once -> inner source
    acquire inner source.iterator exactly once
    store inner iterator
    continue inner loop
```

Empty inner iterables are skipped without re-running the mapper.

GC must retain:

```text
outer
mapper closure/environment
current inner iterator
any inner source retained by that iterator
```

## T9.9 — Flatten adapter

Same traversal algorithm as flatMap(identity), but do not implement by materializing or by repeatedly re-acquiring `item.iterator`.

For each outer item:

```text
acquire its iterator once
store it
exhaust it before outer.next
```

Empty inners are skipped.

## T9.10 — Chain adapter

State:

```text
first
second
phase = First | Second | Done
```

Rules:

```text
while First:
    first.next
    Some -> yield
    None -> phase Second exactly once

while Second:
    second.next
    Some -> yield
    None -> Done

Done -> None forever
```

No switching back to first.

## T9.11 — Skip adapter

Validate count using the same current non-negative integer policy as the existing legacy stage.

State:

```text
upstream
remaining
prefixApplied / equivalent
done
```

The prefix is consumed once for the iterator lifetime, not once for each output call.

After prefix consumption, each successful adapter `next` performs one normal upstream `next`.

## T9.12 — SkipWhile adapter

State:

```text
upstream
predicate
stillSkipping = true
done
```

While `stillSkipping`:

```text
consume upstream
predicate(item)
true  -> skip
false -> stillSkipping=false; yield item
```

Once false has occurred, never call the predicate again. Subsequent outputs delegate to upstream.

For a provably unbounded upstream this adapter remains non-finitely-exhausting; T11 may preserve `Unbounded`.

## T9.13 — Take adapter

Validate count on construction.

State:

```text
upstream
remaining
done
```

Critical law:

```text
take(0).next
    -> None
    -> zero upstream next calls
```

Once `remaining == 0`, no future upstream reads.

If upstream ends early, mark done.

## T9.14 — TakeWhile adapter

State:

```text
upstream
predicate
done
```

On first predicate false:

```text
done = true
return None
```

Do not inspect upstream again on later calls.

## T9.15 — MapWhile adapter

State:

```text
upstream
mapper
done
```

Mapper returning None is terminal, unlike filterMap:

```text
upstream Some(x)
mapper(x)
    Some(y) -> yield y
    None    -> done=true; return None
```

Future `next` returns None without upstream reads or mapper calls.

## T9.16 — Inspect adapter

```text
upstream Some(x)
callback(x) once
return Some(x)
```

No item transformation.

If callback raises, propagate the error normally. Do not convert it to None.

## T9.17 — Indexed adapter

`IndexedIterator` is stateful:

```text
upstream
ordinal = 0
```

For each upstream Some(x):

```text
result = (ordinal, x)        # use canonical tuple shape chosen by final API
ordinal += 1
```

The ordinal is encounter order, never a source cursor/key/index identity.

T10's `Iterable.indexed` constructs this adapter from one fresh `self.iterator`.

## T9.18 — Strict pairwise zip adapter

`Iterator.zip(otherIterator)` is strict, not shortest.

Per attempted step:

```text
leftResult  = left.next     exactly once
rightResult = right.next    exactly once

both Some:
    yield pair tuple

both None:
    done = true
    return None

mixed:
    done = true
    raise ZipLengthMismatchError
```

Do not read either lane again to classify the result.

The exact pair tuple labels are a surface detail only if already ratified; preserve current tuple conventions and do not invent a bespoke ZipPair.

## T9.19 — Migrate existing `Tuple.zipped` away from cursor/size semantics

Current `ZippedIterable.size` prechecks every source's `.size` and the cursor protocol drives traversal. That cannot remain the canonical implementation because:

- not every Iterable has `size`;
- strict zip is defined by per-attempt `next` results;
- P5 state lives on iterator objects.

By G4, `Tuple.zipped` must produce a canonical reusable zipped source whose iterator:

1. acquires each lane's iterator once;
2. calls each iterator once per attempted step, left-to-right;
3. gathers all results before classification;
4. all Some -> yield tuple of payloads;
5. all None -> sticky end;
6. mixed -> `ZipLengthMismatchError`.

Exact class placement/name may preserve the existing `ZippedIterable` public identity if that identity is observable. Do not move/rename an externally visible class casually. If preserving identity requires keeping the class in `iterable.ph`, rewriting that one helper class is allowed; do not use that as permission to refactor the legacy root class.

`Tuple.zipped` and pairwise `Iterator.zip` may use separate concrete iterator classes if one is N-lane dynamic and one is statically pairwise. They must share the same strict length law.

## T9.20 — Sticky exhaustion against adversarial upstreams

Create at least one test iterator that violates P5 sticky behavior internally:

```text
next sequence: Some(1), None, Some(2), ...
```

Canonical adapters that have already observed terminal conditions must not resurrect.

At minimum verify:

```text
takeWhile
mapWhile
take
chain after Done
strict zip after all-None or mismatch path
```

Base map/filter cannot make a non-sticky upstream sticky unless they store done after first upstream None; they **must** do so because P5 requires every canonical adapter to be sticky regardless of upstream behavior.

This is why all adapters should generally record `_done` once they observe upstream exhaustion.

## T9.21 — GC tests

Use the existing forced-GC harness.

Required cases:

1. adapter is the only obvious strong reference to upstream source/iterator;
2. mapper/predicate closure captures an object used after GC;
3. flatMap retains current inner iterator across GC;
4. flatten retains current inner iterator across GC;
5. nested flatMap/flatten retains outer + callback + current inner + state.

No new heap trace arm is expected because ordinary `Object::Instance` slots are already traced.

## T9.22 — Migrate old iterator fixtures

Explicitly update/remove expectations that assume reusable descriptor semantics.

Examples:

```text
iterator_flatmap_order_and_repeatability
    -> split into one-shot stored-pipeline test
       + rebuild-from-source repeatability test

iterator_iter_is_lazy_and_idempotent
    -> `.iter` on Iterator is same traversal through derived Iterable
       `.iter` on reusable source returns fresh concrete iterator

iterator_filter_map_order
    -> preserve ordering/laziness but use stateful next consumption

iterator_skip_take_takewhile
    -> preserve value results, add no-extra-read/sticky checks
```

Do not keep a hidden compatibility mode where a stored adapter silently restarts just to keep the golden unchanged.

## T9.23 — Required coverage

Close:

```text
CV-45..50
CV-52
CV-53
CV-72..74
CV-76..82
CV-85
CV-86
CV-93
CV-95
CV-108
```

T10 closes the terminal/default half of CV-70/71/72/73/75/83/84/94/100..105.

## T9.24 — Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful adapter -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
```

Use actual `--list` filters if the substring differs.

Run a forced-GC subset by exact test name if the core binary cannot cheaply filter all adapter tests.

Do not run the full sequence/collections surface yet; T10 changes its ownership.

## T9.25 — Acceptance

```text
all target lazy adapter defaults exist on Iterator
all adapters are stateful one-shot Iterator objects
construction is lazy
aliases share upstream state
sticky exhaustion holds against adversarial upstream
flatMap/flatten retain current inner iterator
strict zip never truncates
Tuple.zipped no longer depends on size/cursors
no SourceIterator is needed for canonical paths
no native adapter primitive / VM proof path added
GC retention passes
```

## T9.26 — Local STOP / CONSULT triggers

STOP if:

1. a required adapter needs conformance-local storage;
2. generic trait method syntax cannot state a core method without a cross-cutting type-system change;
3. ordinary instance slots fail GC retention;
4. trait default bodies cannot construct adapter classes without changing module identity/architecture;
5. preserving `Tuple.zipped` public identity requires a material module/public-name change;
6. strict pairwise zip cannot raise the ordinary `ZipLengthMismatchError` without a new runtime error ABI.

---

# 13. T10 — Move non-conflicting consumers/terminals to Iterable and preserve eager built-in conveniences

## Purpose

Replace the algorithmic role of the legacy runtime superclass with canonical trait defaults, while preserving accepted direct eager built-in transforms through thin inherent wrappers and avoiding any Iterator-vs-Iterable selector overlap.

After T10 the runtime superclass may still physically exist for compatibility, but the **canonical traits are sufficient** for production behavior.

## Preconditions

- T9 adapter tests green;
- P4 derived Iterator=>Iterable dispatch works;
- all P5 built-in sources have canonical Iterable conformance;
- no global retirement has begun.

## T10.1 — Final ownership split

`Iterator` owns only lazy/state-transforming adapters:

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
zip
```

`Iterable` owns non-conflicting conveniences and consumers:

```text
iter
indexed

each
each(indexed:)
contains
includes
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
join
join(_)
group(by:)
partition(where:)
toList
toSet
toMap
toMap(merging:)
```

Concrete reusable collection classes own selector-conflicting eager transforms:

```text
map
map(indexed:)
filter
flatMap
```

Concrete reusable sources that historically inherited `isEmpty` retain an inherent source-specific `isEmpty`; it is not a generic Iterable default.

## T10.2 — `iter` default

Canonical:

```phalcom
iter { self.iterator }
```

No wrapper allocation.

Consequences:

```text
List.iter          -> fresh ListIterator
Map.iter           -> fresh MapIterator
custom source.iter -> whatever its iterator getter returns
iterator.iter      -> same iterator object through derived Iterable
```

Do not add an `Iterator.iter` method with the same selector.

## T10.3 — `indexed`

Canonical:

```text
self.iterator -> IndexedIterator
```

On a reusable source, each `indexed` call starts a fresh traversal.

On an Iterator receiver, derived Iterable's `iterator` is self, so `it.indexed` wraps the current traversal and shares its upstream state.

## T10.4 — Terminal implementation rule: one iterator acquisition

Each terminal/convenience that consumes traversal obtains exactly one iterator:

```text
let it = self.iterator
```

and advances only that object.

Using canonical `for x in self` inside a trait default is semantically legal after Part I, but direct `next` loops are preferred for the core defaults when they make short-circuit/no-overread behavior easier to prove and avoid accidental recursive use of another default.

Use ordinary Option matching/payload extraction; do not reintroduce `unwrapOr` as a generic hot-path dependency merely for convenience.

## T10.5 — Short-circuit consumers

Specify exact over-read behavior.

### contains/includes

Stop at first equal item. Do not continue after match.

On `Map` and `Set`, inherent methods shadow the generic default and preserve key/member representation semantics.

### find(where:)

Stop at first predicate true and return `Some(item)`.

### any(where:)

Stop at first true.

### all(where:)

Stop at first false.

### none(where:)

Stop at first true and return false.

### index(where:)

Return encounter ordinal, not source key/index/cursor identity.

These are important for unbounded sources: they may terminate without exhausting and T11 must not classify them as full-source exhaustors.

## T10.6 — Full consumers

### count

Consume to sticky exhaustion and return total encountered items.

### count(where:)

Consume all and increment only on predicate true.

### fold(initial:using:)

Consume all, threading explicit initial accumulator.

### reduce(using:)

Empty -> None.
Non-empty -> first item as accumulator, then fold rest, return Some(acc).

Do not acquire a second iterator to obtain the first item.

### join / join(_)

Preserve current stringification/separator semantics. P5 does not optimize the known O(N²) concatenation implementation.

### group(by:)

Preserve current Map-of-Lists behavior.

### partition(where:)

Preserve `(accepted, rejected)` List tuple behavior.

### toList / toSet

Materialize from current traversal.

### toMap / toMap(merging:)

Consume entry-like values through `.key` and `.value`. Canonical Map labeled tuples satisfy this.

Do not add a generic builder/collect abstraction.

## T10.7 — `nth`

Validate `n` using the same non-negative integer policy as skip/take unless a current normative API explicitly differs.

Semantics:

```text
n = 0 -> consume one step
n = k -> consume/discard k steps, then return step k
end before n -> None
```

Exactly `n+1` successful/attempted advancement positions as applicable; no restart.

Do not use source random access because this is an Iterable/Iterator consumer.

## T10.8 — `last`

Consume exactly one iterator to exhaustion.

```text
empty -> None
nonempty -> Some(final encountered item)
```

This is intentionally destructive on an Iterator receiver because it is a terminal, not a getter promising non-consumption.

## T10.9 — Concrete eager transformation wrappers

The legacy runtime superclass currently makes `map`, `map(indexed:)`, `filter`, and `flatMap` available to all classes that inherit it. Part III will remove that superclass. Before then, add thin inherent wrappers to the built-in reusable source classes whose accepted public surface currently includes those selectors.

Expected inventory on the current baseline:

```text
List
Set
Map
Tuple
Bytes
Range
```

String byte/code-point views did **not** inherit the legacy Iterable superclass before P5; do not invent direct eager `map/filter/flatMap` on them merely because they now conform to the canonical trait. Their production path is:

```text
string.bytes.iter.map(...)
string.codePoints.iter.filter(...)
```

Each wrapper delegates to one canonical algorithm:

```phalcom
map(_ f) {
  self.iterator.map(f).toList
}

filter(_ p) {
  self.iterator.filter(p).toList
}

flatMap(_ f) {
  self.iterator.flatMap(f).toList
}
```

`map(indexed:)` may delegate through `self.indexed` / `self.iterator` and the canonical lazy map, then materialize.

Do not copy traversal loops into six classes.

### Map note

Because P5 intentionally changes `Map::Item` from key to entry tuple, direct eager Map transformations now consume the canonical entry Item. Preserve eagerness/result-family, not the obsolete key-only cursor payload.

If a newer normative Map API explicitly defines a different transform input, STOP AND CONSULT.

## T10.10 — Preserve concrete `isEmpty` without making Iterator destructive

Add/retain inherent `isEmpty` only on reusable concrete families that historically received it from the legacy superclass.

On current baseline this is expected for:

```text
List
Set
Map
Tuple
Bytes
Range
```

Implementation may remain:

```phalcom
isEmpty { self.size == 0 }
```

where that exactly preserves current behavior.

For Range, preserve current `size`-based behavior, including errors for unbounded/lowerless shapes; do not invent a new non-consuming emptiness theorem in P5.

Do not add `isEmpty` to the `Iterable` trait because every Iterator derives Iterable and a getter-like name would then consume or hang.

## T10.11 — Do not destructively delete legacy algorithms yet

Do **not** remove methods from `collections/iterable.ph` merely because canonical defaults now exist. Part III still needs migration runway for repository-defined legacy subclasses.

T10 proves independence by tests on classes that do not inherit the legacy runtime superclass.

Part III deletes the old class/methods after all remaining consumers are migrated.

## T10.12 — Custom Iterator ergonomics test

Create a source-language iterator that implements only:

```text
class Counter { state...; next -> Option<Int> }
impl Iterator for Counter { type Item = Int }
```

No `map`, `filter`, `count`, `iter`, `toList`, etc.

Prove:

```text
for x in Counter...
counter.map(...)
counter.filter(...)
counter.iter
counter.count
counter.toList
counter.nth
counter.last
```

as appropriate to one-shot state.

This closes the production ergonomic objective, not just method availability on built-ins.

## T10.13 — Arbitrary custom Iterable test

Create a reusable source that is **not** a legacy runtime `Iterable` subclass:

```text
impl Iterable for Source
    Item = Int
    Iterator = SourceIteratorType
```

where the iterator implements only Item+next.

Prove:

```phalcom
source.iter.map(f).toList
source.count
source.find(where: ...)
```

without Source defining those methods.

This test is the primary proof that the runtime superclass is no longer algorithmically necessary.

## T10.14 — Selector ambiguity test

For generic:

```text
T |= Iterator
```

prove:

```text
t.map(...)     -> Iterator lazy default
t.iter         -> Iterable default through derived evidence
t.toList       -> Iterable terminal through derived evidence
```

with no overlapping `Iterable.map`.

This is the architectural discriminator for P5-SURFACE-11.

## T10.15 — Map membership discriminator

Prove both in the same fixture:

```phalcom
map.contains(existingKey) == true

map.iter.contains(
  (key: existingKey, value: existingValue)
) == true
```

and a wrong value makes the iterator-entry membership false while direct key membership remains true.

## T10.16 — No SourceIterator allocation

The old:

```phalcom
iter { SourceIterator.new(self) }
```

must not be used by canonical defaults.

Prove `source.iter` has the same concrete traversal family as `source.iterator`, using the least brittle existing identity/reflection test available:

```text
class/type check
or behavior only possible on concrete iterator
or direct object identity comparison when one iterator getter result is reused
```

Do not add new reflection API solely to test this.

## T10.17 — Required fixture migrations

Update sequence/iterator goldens that depended on superclass or repeatability.

Important expected changes:

```text
stored pipeline materialized twice:
    second is empty/exhausted

fresh source.iter pipeline:
    traverses again

direct List.map:
    eager List result unchanged

List.iter.map:
    adapter construction does not call mapper

generic custom Iterable:
    gets terminals/defaults despite no legacy superclass
```

## T10.18 — Required coverage

Close:

```text
CV-51
CV-70
CV-71
CV-72
CV-73
CV-75
CV-83
CV-84
CV-94
CV-100
CV-101
CV-102
CV-103
CV-104
CV-105
```

and reinforce CV-45..53 / CV-76..86 through terminal integration where relevant.

## T10.19 — Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful default -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
```

Use exact filters from `--list` where needed.

Do not run broad workspace tests.

## T10.20 — Acceptance

```text
Iterator Item+next author gets full target lazy toolbox
Iterator gets Iterable terminals through derived evidence
arbitrary custom Iterable gets non-conflicting defaults
.iter is alias, no SourceIterator
stored adapter is one-shot
fresh source rebuild restarts
direct built-in map/filter/flatMap remain eager
isEmpty preserved concretely, absent generically on Iterator
Map membership distinction preserved
toMap works with labeled tuples
no selector-precedence rule added
legacy superclass no longer required for canonical algorithms
```

## T10.21 — Local STOP / CONSULT triggers

STOP if:

1. ordinary member resolution cannot expose non-overlapping Iterable defaults through derived evidence;
2. the only proposed fix is trait precedence/specialization;
3. preserving direct eager built-in transforms requires duplicating full traversal algorithms;
4. `toMap` cannot consume labeled tuple `.key/.value` without a product-model change;
5. concrete `isEmpty` preservation conflicts with newer normative APIs.

---

# 14. T11 — Align the existing boundedness analyzer with canonical iteration

## Purpose

Update the existing compiler-only boundedness lattice after the canonical surface is stable, without turning it into a conformance checker.

T11 executes **before G4**, because G4's shared-header definition explicitly includes boundedness coherence.

## Preconditions

- T9 lazy selector set is final for P5;
- T10 terminal/eager wrapper set is final;
- direct collection vs iterator receiver behavior is stable.

## T11.1 — Preserve the current lattice and rejection rule

Do not redesign:

```rust
Bounded
Unbounded
Unknown
```

Rule remains:

```text
reject full-source exhaustor only when provably Unbounded
Unknown remains legal
```

No runtime boundedness flag.

## T11.2 — Recognize canonical traversal entry points

Current `.iter` recognition stays but changes meaning from wrapper-descriptor construction to direct iterator acquisition.

Add `.iterator` with the same source fact transition:

```text
source.iter
source.iterator
    -> LazyPipeline(source.boundedness)
```

On an already-lazy Iterator receiver, `.iter`/`.iterator` preserve lazy mode and boundedness.

Do not require the receiver to be semantically proven Iterable here; semantic acceptance has already happened elsewhere.

## T11.3 — Propagation table

Use this minimum sound table.

### Preserve receiver boundedness

```text
map
filter
filterMap
inspect
skip
indexed
```

For an Unbounded source these remain Unbounded: none can cause normal finite source exhaustion.

### `skipWhile`

```text
Bounded   -> Bounded
Unbounded -> Unbounded
Unknown   -> Unknown
```

Reason: with an infinite upstream, either the predicate stays true forever and `next` never returns, or it eventually becomes false and the adapter then has an infinite remainder. It cannot normally become finitely exhausted.

### Finite limit

```text
take -> Bounded
```

even from Unbounded/Unknown receiver, because the adapter performs at most n successful yields/reads after validation and stops itself.

### Predicate/mapper early stop

```text
takeWhile:
    Bounded -> Bounded
    Unbounded/Unknown -> Unknown

mapWhile:
    Bounded -> Bounded
    Unbounded/Unknown -> Unknown
```

A callback may stop them at finite time, so an unbounded input is not proof of unbounded output.

### Nested expansion

```text
flatMap:
    Unbounded outer -> Unbounded
    Bounded/Unknown outer -> Unknown

flatten:
    Unbounded outer -> Unbounded
    Bounded/Unknown outer -> Unknown
```

Do not attempt to prove inner source cardinality in P5.

### chain

If the implementation can cheaply inspect the second argument's syntax facts:

```text
first Unbounded -> Unbounded
first Bounded   -> facts(second)
first Unknown   -> Unknown
```

If the second argument cannot be inspected safely through current AST shape, use `Unknown` except the sound `first Unbounded -> Unbounded` case.

### strict zip

Conservative minimum:

```text
both Bounded   -> Bounded
both Unbounded -> Unbounded
mixed Bounded/Unbounded -> Unknown
any Unknown    -> Unknown
```

Mixed finite/infinite strict zip will normally terminate by mismatch error, but boundedness is not an error-theorem engine; `Unknown` avoids adding a new semantic notion merely for this case.

Do not classify zip as shortest.

## T11.4 — Full-source exhaustor catalog

Ensure terminal calls remain recognized on lazy pipelines:

Getter-like terminals:

```text
toList
toSet
toMap
last        # full source
count       # getter form
join        # getter form if applicable
```

Method terminals:

```text
map(indexed:)     # only direct concrete eager wrapper, not Iterator map
fold(initial:using:)
reduce(using:)
count(where:)
each(_)
each(indexed:)
group(by:)
partition(where:)
toMap(merging:)
join(_)
```

Audit the actual selector encoding rather than copying this list blindly; use current canonical selector names from the source surface.

Do **not** classify short-circuit/prefix queries as mandatory full exhaustion:

```text
contains/includes
find
index
any
all
none
nth
```

They may terminate on an unbounded source.

`last` necessarily exhausts and should be guarded on provably unbounded sources.

## T11.5 — Concrete eager transforms

For a `Concrete` receiver:

```text
map(_)
map(indexed:)
filter(_)
flatMap(_)
```

remain eager and therefore full-source operations.

For `LazyPipeline` receiver:

```text
map(_)
filter(_)
flatMap(_)
```

are lazy adapters and must not themselves trigger exhaustion diagnostics.

The mode distinction is the point of the existing `IterationMode`.

## T11.6 — Range discriminators

Keep:

```text
finite lower+upper Range -> Bounded
lower-only a..           -> Unbounded
lowerless Range          -> Unknown
```

Required acceptance:

```phalcom
(0..).toList
    -> compile-time provably-unbounded exhaustion error

(0..).iter.map(f).toList
    -> same error

(0..).iter.take(10).toList
    -> accepted, terminates

(0..).iter.takeWhile(p).toList
    -> accepted as Unknown

unknownIterable.toList
    -> accepted as Unknown
```

Do not call `Range.size` or add runtime caps.

## T11.7 — Required unit tests

Extend `boundedness.rs` unit tests for:

```text
.iterator enters LazyPipeline
.iter still enters LazyPipeline
map/filter/filterMap/inspect/skip preserve
skipWhile unbounded remains Unbounded
take becomes Bounded
takeWhile/mapWhile unbounded -> Unknown
flatMap/flatten unbounded outer -> Unbounded
chain conservative cases
zip conservative cases
direct concrete map is eager
lazy iterator map is not eager
last on unbounded is diagnosed
nth on unbounded is not rejected solely for boundedness
```

## T11.8 — Required corpus tests

Preserve/update the existing negative collection expansion/exhaustor fixtures rather than creating a second boundedness test universe.

Close:

```text
CV-91
CV-111
```

## T11.9 — Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core boundedness -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections_literals_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
```

The iterator lane is included only because selector-role changes can alter eager/lazy diagnosis. Do not run the full core suite.

## T11.10 — Acceptance

```text
.iterator and .iter recognized
new lazy adapters classified conservatively
take bounds unbounded Range pipelines
Unknown remains legal
direct collection transforms stay eager
Iterator map/filter/flatMap stay lazy
short-circuit queries are not false-positive exhaustors
no semantic trait proof in boundedness
no runtime boundedness metadata/cap
```

## T11.11 — Local STOP / CONSULT triggers

STOP if satisfying a test appears to require:

```text
semantic conformance lookup from boundedness
arbitrary predicate theorem proving
runtime cardinality flags
Range size fabrication
new public boundedness reflection API
```

Those are outside P5.

---

# 15. G4 — Final Part-II iterator/default/boundedness gate

## Purpose

G4 is the terminal Part-II gate. It proves:

```text
built-in sources canonical
adapter toolbox one-shot/sticky
Iterable terminals/defaults available through ordinary evidence
direct eager built-in API preserved
boundedness matches canonical selector roles
no new native/proof floor
```

Part III may begin only after G4 passes or has an explicitly accepted non-blocking baseline classification.

## G4.1 — Focused test commands

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core iteration_stateful -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iterator_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus sequence_negative

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bytes
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus strings

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core boundedness -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus collections_literals_negative
```

If the repository's test binary filters differ, use `cargo test ... -- --list` once and substitute the actual filter. Do not widen to the whole workspace merely because one filter name changed.

## G4.2 — Canonical surface census

Run:

```sh
rg -n 'trait Iterator|trait Iterable|map<|filterMap|flatten|chain|skipWhile|mapWhile|inspect|zip|nth|last|toList|toSet|toMap' \
  phalcom-core/core/universe/src/collections/iterator_laws.ph
```

Confirm the role split rather than just the name count:

```text
Iterator:
    lazy adapters only

Iterable:
    iter/indexed/consumers/terminals
    no overlapping map/filter/flatMap
```

## G4.3 — Legacy cursor-adapter census

Run:

```sh
rg -n 'SourceIterator|class MapIterator is Iterator|class FilterIterator is Iterator|class FlatMapIterator is Iterator|class SkipIterator is Iterator|class TakeIterator is Iterator|class TakeWhileIterator is Iterator' \
  phalcom-core/core/universe/src/collections
```

Legacy definitions may still physically exist in `iterable.ph` for Part III, but canonical current fixtures/code must not instantiate them.

Search live production/fixtures for constructors:

```sh
rg -n 'SourceIterator\.new|MapIterator\.new|FilterIterator\.new|FlatMapIterator\.new|SkipIterator\.new|TakeIterator\.new|TakeWhileIterator\.new' \
  phalcom-core/core/universe/src \
  phalcom-core/tests \
  -g '!phalcom-core/core/universe/src/collections/iterable.ph'
```

Expected: no canonical production/test consumer of the legacy cursor adapters.

## G4.4 — SourceIterator discriminator

Search:

```sh
rg -n 'SourceIterator' phalcom-core/core/universe/src phalcom-core/tests
```

By G4, any hit outside the legacy compatibility file/historical test text is a defect.

## G4.5 — Native floor check

Search changed source for new native iterator/combinator declarations:

```sh
git diff --unified=0 -- \
  phalcom-core/core/universe/src/collections \
  phalcom-core/core/universe/src/scalar/string.ph \
  phalcom-core/src/compiler/boundedness.rs
```

No new `@native` adapter or runtime proof hook should appear.

## G4.6 — Lifecycle acceptance checklist

All must be explicitly evidenced:

```text
[ ] two source.iterator calls independent
[ ] iterator.iterator is self
[ ] source.iter is source.iterator alias, not wrapper
[ ] stored adapter second terminal stays exhausted
[ ] rebuilding pipeline from source traverses again
[ ] upstream alias + adapter share state
[ ] take(0) does zero upstream reads
[ ] takeWhile/mapWhile sticky
[ ] strict zip raises mismatch, never truncates
[ ] Map direct Item is labeled entry tuple
[ ] String itself non-Iterable, views canonical
[ ] Range a.. + take works without size
[ ] custom Item+next Iterator gains defaults
[ ] custom non-legacy Iterable gains terminals
[ ] direct List.map eager / List.iter.map lazy
[ ] Map key membership vs entry membership distinct
[ ] nested adapter GC retention passes
[ ] boundedness Unknown remains legal
```

## G4.7 — Failure classification

A failure at G4 belongs to one of:

```text
A. built-in state machine defect -> return to T6/T7/T8 owner
B. adapter lifecycle defect      -> T9
C. default/member-resolution bug -> T10
D. eager wrapper regression      -> T10
E. boundedness false claim       -> T11
F. Part-I semantic/lowering regression -> verify causality, then consult if architecture must change
G. unrelated baseline failure    -> shared baseline policy
```

Do not make a cross-cutting rewrite before assigning the failure to one owner.

---

# 16. Part-II executor debugging budgets

The shared budgets apply. For the heaviest Part-II task, use these concrete limits.

## T6/T7/T8

One focused failing source/runtime test may receive:

```text
up to 2 direct implementation corrections
then one evidence-gathering inspection pass
then consult if the failure implies a fixed semantic premise is false
```

Do not redesign collection storage to fix iterator code.

## T9

State-machine failures are expected to be local.

For one adapter:

```text
attempt 1: state transition / Option branch correction
attempt 2: typing/generic constraint correction
attempt 3: only if new evidence identifies a distinct local bug
```

After that, stop if the only next move is changing generic trait semantics, GC architecture, or runtime dispatch.

Do not "fix" adapter tests by weakening sticky exhaustion.

## T10

If a default is unavailable:

1. prove whether member resolution sees the applicable Iterable evidence;
2. compare with a known working trait default;
3. inspect selector overlap;
4. if the fix requires precedence/specialization, consult.

Do not copy the method into every class as an emergency workaround.

## T11

If boundedness misclassifies:

1. print/inspect the canonical selector and inferred receiver facts;
2. correct the local syntax catalog;
3. if facts require semantic type/conformance knowledge, stop.

---

# 17. Part-II implementation records

At each task completion, update only durable checkpoint facts.

## T6 record

```text
finite core iterator classes
String view iterator classes
associated Item/Iterator bindings
String non-Iterable proof
```

### Current execution record — 2026-09-17

```text
T6: PARTIAL — Bytes and both String views are stateful and focused-tested;
    Set is source-authored but its generic iterator constructor is blocked by
    the inherited VM call-entry recipe failure; Tuple is deferred because
    Dynamic cannot be published as the associated Item type.
T7: PARTIAL — Map and its key/value/entry view iterators are source-authored;
    canonical Map and entry Items use labeled (key: K, value: V) products;
    focused runtime execution is blocked by the same generic call-entry issue.
T8: NOT STARTED — the Range focused path reaches the inherited Option<Number>
    call-entry boundary, so no unverified Range conformance was retained.
G3: NOT PASSED — the complete P5-owned canonical source inventory is not
     executable yet.
T9-T11: NOT STARTED — held by the explicit G3 precondition.
```

## T7 record

```text
Map Item migration
Entry decision
Map view iterator inventory
```

## T8 record

```text
RangeIterator state
preserved supported/unsupported forms
```

## G3 record

```text
focused gate results
built-in canonical traversal inventory complete
```

## T9 record

```text
adapter class inventory
one-shot/sticky lifecycle
strict zip implementation ownership
Tuple.zipped migration
GC evidence
```

## T10 record

```text
final Iterator vs Iterable default ownership
concrete eager wrapper inventory
concrete isEmpty preservation inventory
SourceIterator no longer canonical
```

## T11 record

```text
boundedness selector propagation table
new/changed exhaustor catalog entries
```

## G4 record

```text
exact commands/results
baseline classifications
remaining legacy compatibility hits intentionally deferred to Part III
next action: Part III T12
```

Do not rewrite the final P5 walkthrough in Part II; Part III/G6 owns completion records.

---

# 18. Part-II handoff contract to Part III

Part III may assume only interfaces actually proven by G4.

Required handoff facts:

```text
actual iterator_impls module path
all core source iterator class paths
all core Iterable associated Item/Iterator bindings
Map labeled tuple Item exact type
Entry final status
RangeIterator path + supported subset
String byte/codepoint iterator paths + Item types
Iterator lazy default selector list
Iterable terminal/default selector list
concrete eager wrapper inventory
concrete isEmpty wrapper inventory
Tuple.zipped canonical implementation path
boundedness propagation table
exact G3/G4 results
remaining legacy cursor/class hits intentionally deferred
any approved deviations/consultation outcomes
```

Part III must **not** need to reopen:

```text
stateful one-shot adapter lifecycle
Map direct Item representation
Range step/descending decision
String Item/Iterable decision
Iterator vs Iterable default ownership
eager-vs-lazy selector policy
boundedness lattice semantics
```

If any are unresolved at G4, Part II is not ready to hand off.

Part III then owns:

```text
T12 positional expansion cursor probe
T13 repository legacy subclasses/live cursor consumers
T14 runtime Iterable/Iterator class + public-name retirement
G5 retirement gate
T15 docs/tooling/final stabilization
G6 completion gate
```

---

# 19. Part-II commit/review checklist

Before each commit:

```text
[ ] diff limited to active task ownership
[ ] no unrelated formatter churn
[ ] no new native iterator primitive
[ ] no cursor-token state added to canonical classes
[ ] no P4 trait identity rename
[ ] no Part-I semantic/lowering architecture change
[ ] focused task tests read in full
[ ] git diff --check
```

Before G3:

```text
[ ] List from Part I still green
[ ] Set/Tuple/Bytes canonical
[ ] String views canonical
[ ] Map + views canonical labeled entries
[ ] Range canonical
```

Before G4:

```text
[ ] G3 passed
[ ] T9 adapters one-shot/sticky
[ ] T10 defaults/wrappers role split exact
[ ] T11 boundedness complete
[ ] old repeatability fixtures migrated
[ ] no canonical SourceIterator use
[ ] no selector overlap requiring precedence
[ ] no broad retirement performed early
```

---

# 20. Part-II self-review performed during planning

This plan was reviewed against the final three-pass requirements analysis, the shared header, Part I's handoff contract, and the live repository baseline.

## 20.1 Architecture review

Checked:

- stateful iterator ownership remains object-local;
- built-in source placement respects internal visibility;
- no second conformance registry/module is introduced;
- adapter module cycle risk is removed by default co-location with the canonical trait module;
- Map uses labeled product representation;
- String stays non-Iterable;
- Range remains the existing forward subset;
- lazy/default ownership avoids selector overlap;
- legacy runtime class deletion remains Part III;
- boundedness stays compiler-local.

## 20.2 Lifecycle review

Checked:

- reusable source vs one-shot iterator distinction;
- alias-sharing;
- fresh traversal reacquisition;
- sticky exhaustion after None;
- mutation cannot resurrect finite iterators after done;
- early-stopping adapter done flags;
- flatMap/flatten current-inner ownership;
- zip classification uses already-read results.

## 20.3 API review

Checked:

- Item+next author experience;
- `.iter` no wrapper;
- direct eager collection transforms preserved through forwarding;
- no generic Iterable map/filter/flatMap overlap;
- concrete `isEmpty` preservation explicitly covered;
- `nth`/`last` included;
- explicit materializers retained; no `collect`.

## 20.4 Test review

Checked:

- 58 Part-II coverage rows assigned;
- old repeatability fixtures identified as migrations;
- direct stateful tests separated from corpus goldens;
- GC cases included;
- Map generic typing included;
- String typed Item cases included;
- Range unbounded+taken case included;
- boundedness false-positive/false-negative cases included;
- G3/G4 are bounded focused gates, not workspace-wide suites.

## 20.5 Cross-part review

Checked:

- Part-I architecture is consumed, not reopened;
- G4 executes after T11 despite the header's inventory listing;
- global cursor/runtime class deletion remains Part III;
- VM positional expansion remains Part III;
- Part-III handoff enumerates exact state it needs.

---

# 21. Part-II completion truth model

Part II is **not** complete because code compiles.

It is complete only if G4 establishes all of:

```text
1. canonical built-in sources/views use real stateful iterators;
2. Iterator lazy defaults are source-authored and one-shot;
3. Iterable consumers/terminals are available without selector conflict;
4. concrete accepted eager transforms survive;
5. concrete source isEmpty behavior survives without becoming an Iterator terminal;
6. stored adapter values never restart;
7. rebuilding from a reusable source does restart independently;
8. Map entries are labeled tuples everywhere canonical;
9. Range remains only the ratified forward subset;
10. String views are canonical while String is not Iterable;
11. GC retains adapter/upstream/callback/current-inner state;
12. boundedness recognizes the final surface conservatively;
13. no new runtime trait proof or native combinator floor was introduced;
14. remaining cursor/runtime-class dependencies are explicitly enumerated for Part III.
```

At that point the implementation should be in the intended transitional state:

```text
canonical traversal + production defaults are complete
legacy cursor/runtime class still physically exists only as migration substrate
Part III can remove cross-cutting remaining consumers and identities
```

---

# 22. Part-II next action after G4

After successful G4:

```text
STOP Part II implementation work.
Do not opportunistically delete Iterable runtime infrastructure.
Open/read:
    LANG005.C6.P5-plan-03-retirement-and-stabilization.md
Begin:
    T12 positional expansion migration
```

If Part III has not yet been generated, preserve the exact G4 handoff state and wait for the architect plan rather than inventing retirement order.
