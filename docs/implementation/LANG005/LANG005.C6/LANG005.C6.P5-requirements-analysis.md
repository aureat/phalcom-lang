---
id: LANG005.C6.P5.REQUIREMENTS
category: LANG
program: LANG005
checkpoint: LANG005.C6
plan: LANG005.C6.P5
kind: requirements-analysis
analysis_pass: 3-of-3
status: FINAL
completion: COMPLETE
verification: REPOSITORY_INSPECTED_REMOTE_PASS3_LOCAL_EXECUTION_UNAVAILABLE
prepared: 2026-09-17
repository: aureat/phalcom-lang
repository_branch: main
repository_revision: 06e37b6830cd9114d03f2a794d0b9a973f8cdc82
predecessor: LANG005.C6.P4
---

# LANG005.C6.P5 — Iterator / Iterable Stabilization and Production Iteration Migration
## Requirements Analysis — Pass 3 of 3 (Final)

> **Pass status:** This is the third and final cumulative requirements-analysis pass. It preserves the repository-grounded findings from passes 1 and 2, closes the remaining design questions that can be closed from the live repository, and converts unresolved executable-state questions into explicit pre-implementation gates rather than leaving architectural ambiguity. It is **not** the implementation plan; it is the final requirements source from which that plan should be written.
>
> **Final design priority:** A user-defined iterator should normally implement only `type Item` and `next`. The canonical `Iterator` trait is the home for state-transforming lazy adapters. `Iterable` remains the reusable traversal-factory contract (`Item`, `Iterator`, `iterator`) and is also the natural home for non-conflicting traversal consumers/terminals whose semantics are identical for a reusable source and a one-shot iterator. Because every `Iterator` derives `Iterable`, a custom iterator still receives both surfaces from `Item + next`. Eager direct transformations such as collection `map`/`filter` must not be duplicated as `Iterable` defaults because they would collide with lazy `Iterator` selectors; concrete reusable collections preserve those accepted eager transforms through thin inherent forwarding wrappers.
>
> **Repository-grounding limitation:** Remote `main` was rechecked in pass 3 and still points to `06e37b6830cd9114d03f2a794d0b9a973f8cdc82` (`feat(lang005): complete P4 iterator iterable law staging`). A local checkout was not available, so this final analysis does not claim local worktree state or executable test reproduction. Those are mandatory implementation preflight gates, not unresolved language design.

---

# 1. Executive assessment

P5 is the production cutover from a staged canonical iterator law to a single live iteration architecture. P4 already established the hard semantic laws: `Iterator` owns `Item` and `next`; `Iterable` owns `Item`, `Iterator`, and `iterator`; `Iterable::Iterator` conforms to `Iterator`; both Item projections agree; and every `Iterator` derives `Iterable` over itself. P5 must make those laws the only authority used by `for`, collections, lazy pipelines, argument-expansion consumers that still depend on cursor detection, and current public documentation.

The final pass closes five architectural points that were still partially open after pass 2.

First, the exact `for` semantic seam is now identified. `Statement::For` currently calls `resolve_iteration_element_application`, which structurally probes `iteratorValue(_)`, a getter fallback, and `iterate(_)`. `CallableAnalysis` already owns per-expression semantic products including trait dispatch. `ModuleLoweringSemantics` already projects those products into `TraitInvocationSpec` / `TraitRequirementInvocationSpec` keyed by `LoweringSite`. Therefore P5 should add a first-class iteration-lane semantic product to `CallableAnalysis`, keyed by the lane source expression identity, and project it through the existing semantic-lowering layer. The compiler consumes that product; it does not rediscover traits.

Second, the production API ownership is now closed more carefully than in pass 2. The Rust lesson is **not** “put every operation on both source and iterator traits.” It is “keep the primitive iterator requirement tiny and derive a large algorithmic surface once.” In Phalcom that means:

```text
Iterable
    small reusable-source contract
    Item + Iterator + iterator
    optional non-conflicting convenience: iter -> iterator

Iterator
    tiny implementation burden
    Item + next
    rich lazy-adapter default surface

Iterable defaults
    iter / indexed + traversal consumers / terminals
    iterator receivers inherit these through Iterator => Iterable

concrete reusable collections
    representation-specific iterator construction
    accepted eager direct conveniences as thin inherent forwarding wrappers
```

This avoids a real semantic hazard. Because `Iterator => Iterable` is a derived conformance, placing eager `map`/`filter` defaults on `Iterable` and lazy defaults with the same selectors on `Iterator` would force a new trait-precedence/specialization rule. P5 must not invent such a rule. The universal path for an arbitrary reusable iterable is `source.iter.<iterator operation>`; core collections may retain ergonomic eager `source.map(...)` through inherent wrappers that delegate into iterator algorithms and materialize.

Third, parallel `for` setup is no longer ambiguous. `docs/specs/iterator.md` states that all lane source expressions are evaluated exactly once, left-to-right, before traversal begins, and one iterator is then obtained from each lane. The current generalized compiler interleaves `source1 -> iterate1 -> source2 -> iterate2`. P5 must replace this with a two-phase preheader: evaluate/store every source left-to-right, then acquire/store every iterator left-to-right, then begin advancement.

Fourth, cursor retirement has one additional production dependency beyond `for`: `VM::has_iteration_protocol` still checks for `iterate(_)` and `iteratorValue(_)` to decide the generic positional `*` expansion lane. That dependency must be migrated or explicitly coordinated with the argument-expansion owner before cursor selectors are deleted. The replacement must not turn the VM into a trait solver.

Fifth, the stateful object model is compatible with the existing GC. Iterator/adaptor objects can be ordinary instances: `Object::Instance` tracing walks every instance slot, so upstream iterators, callbacks, current inner iterators, source objects, and buffered values held in ordinary fields are already traced. No iterator-specific GC representation is required.

The resulting target is intentionally Rust-like in leverage but Phalcom-native in semantics:

```text
custom Iterator author writes Item + next
        ↓
Iterator defaults provide the broad toolbox
        ↓
derived Iterator => Iterable makes it usable by for
        ↓
Iterable sources create fresh independent iterator objects
        ↓
for and generic algorithms consume preselected evidence
        ↓
VM executes; it never proves conformance or normalizes projections
```

The only remaining non-design uncertainty is executable repository state: the known P4 `Option.unwrapOr`/generic-call-entry baseline block must be reproduced in a real checkout before patches begin.

---

# 2. Live repository baseline

## 2.1 Remote repository state re-verified in pass 3

The connected GitHub repository still reports:

```text
repository: aureat/phalcom-lang
branch:     main
HEAD:       06e37b6830cd9114d03f2a794d0b9a973f8cdc82
subject:    feat(lang005): complete P4 iterator iterable law staging
```

The recent commit list contains no newer `main` revision. Pass 3 therefore analyzes the same P4-complete repository state as passes 1 and 2, not a moving target.

## 2.2 Local-state limitation and mandatory preflight

No local checkout was available. These facts remain intentionally unclaimed:

```text
local branch / detached state
local HEAD divergence
modified / staged / untracked files
local-only commits
current executable test baseline
```

Before implementation, the plan must begin with:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -n 10 --oneline
```

and must reproduce the P4-recorded `language-corpus::iteration` baseline failure before attributing any iteration failure to P5.

## 2.3 Additional pass-3 evidence read

Pass 3 deepened the earlier authority set with the concrete integration seams that matter to implementation planning:

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/impls.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/boundedness.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/heap/trace.rs

phalcom-core/core/universe/src/collections/iterable.ph
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/scalar/string.ph
phalcom-core/core/universe/src/collections/package.ph

docs/implementation/LANG005/support/core_trait_impls.ph
docs/specs/objects/impl.md
```

The analysis therefore has enough repository evidence to name the semantic ownership and lowering contracts for P5. What it does not have is permission to claim runtime verification without a checkout.

---

# 3. P4 takeover state

P4 is complete and implemented, with verification classified `BASELINE_BLOCKED` because the positive legacy iteration corpus retained a pre-existing `Option.unwrapOr` / generic call-entry runtime failure. Its canonical semantic work is green.

P5 inherits the following as fixed architecture:

```text
GenericSignature generic constraints
Conforms(subject, TraitRef)
assumed vs exact generic evidence distinction
associated-type declaration identity
associated projection formation and normalization
conditional / nested conformance proof
DerivedConformanceRuleIndex / guarded generic-target conformance
runtime type-environment recipes
runtime conformance-environment recipes
InvokeTraitRequirement execution
VM prohibition on semantic proof search
```

P5 must not reinterpret the existence of a production migration problem as evidence that P4's theorem should be special-cased in the compiler or VM.

P4's staged Universe law is already source-authored and executable through the general conformance machinery. P5's obligation is to consume it.

---

# 4. Current canonical Iterator / Iterable architecture

The live canonical source in `collections/iterator_laws.ph` is:

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

The canonical theorem is therefore:

```text
I |= Iterator
    => I |= Iterable

<I as Iterable>::Iterator == I

<I as Iterable>::Item
    ==
<I as Iterator>::Item
```

The language specification additionally fixes these lifecycle semantics:

- an `Iterable` that is not itself an iterator produces an independent traversal through `iterator`;
- an `Iterator` is stateful and single-pass;
- `Iterator.iterator` returns the same traversal (`self`), not a reset or clone;
- `next` returns `Some(item)` while producing values and `None` on exhaustion;
- exhaustion is sticky: after `None`, later `next` calls on the same iterator also return `None`;
- failure propagates through ordinary error semantics and is not represented by `None`;
- a type is not iterable merely because it structurally exposes `iterator` or `next`.

These are P5 implementation requirements, not optional library conventions.

---

# 5. P4 staging state and final public-name transition

P4 intentionally stages the canonical traits separately from the cursor runtime classes:

```text
collections.iterator_laws
    export trait Iterator
    export trait Iterable

collections.iterable
    legacy @native class Iterable<T>
    legacy class Iterator
    legacy cursor adapters / generic operations

collections/package.ph
    expose .iterable
    expose .iterator_laws
```

Pass 3 closes the desired end state:

1. The public declarations named `Iterator` and `Iterable` are the canonical traits from the P4 law.
2. The legacy classes with those names disappear; they are not retained under aliases as public compatibility types.
3. The canonical trait declaration identity should be preserved if possible. File/module renaming is not itself a semantic goal; P5 should avoid gratuitous declaration-identity churn merely to rename `iterator_laws.ph`.
4. The package exposure set must stop exposing the legacy class-bearing module as an iteration authority once migration is complete.
5. Any temporary internal alias used to stage the cutover must be removed in the same P5 completion sequence.

A safe transition is therefore **consumer-first, name-switch-last**:

```text
migrate semantic/lowering consumers
migrate collection conformances + iterator classes
migrate eager convenience methods off superclass inheritance
migrate views/repository subclasses
prove no runtime class consumer remains
remove legacy class declarations + iterable_class bootstrap identity
publish only canonical trait meanings under Iterator / Iterable
update guides/LSP/current specs
```

`P5-NAME-01` — Final public `Iterator` denotes the canonical trait.

`P5-NAME-02` — Final public `Iterable` denotes the canonical trait.

`P5-NAME-03` — No import scope may contain two live public declarations with the same `Iterator` or `Iterable` name after the cutover.

`P5-NAME-04` — Preserve canonical trait declaration identity unless a module-path change is required by the package system; do not rename declarations merely for cosmetic cleanup.

`P5-NAME-05` — Temporary compatibility names are internal, mechanically bounded, and deleted before P5 completion.

`P5-NAME-06` — Current docs, examples, source-index/LSP descriptions, and tests change in the same cutover so no current-facing surface teaches the cursor classes.

---

# 6. Current production iteration architecture

## 6.1 Semantic analysis is still structural cursor probing

`Statement::For` in `phalcom-semantic/src/checker/statement.rs` analyzes every lane source and calls `resolve_iteration_element_application`. That helper currently attempts legacy protocol members rather than proving the canonical trait law:

```text
iteratorValue(_)
iteratorValue getter fallback
iterate(_)
```

This is a direct P5 replacement surface. The target is nominal/evidence-driven:

```text
prove sourceType |= Iterable
normalize <sourceType as Iterable>::Item
normalize <sourceType as Iterable>::Iterator
prove associated iterator |= Iterator
select Iterable.iterator
select Iterator.next
```

A user type that merely happens to define cursor-shaped methods must stop qualifying for `for`.

## 6.2 The compiler independently owns the old protocol today

`phalcom-core/src/compiler/lib/loops.rs` still lowers ordinary `for` with string-selected sends to `iterate` and `iteratorValue`. The simple path preserves several valuable properties that P5 must keep:

- source evaluated once;
- direct jump-based loop, not `.each { ... }`;
- no materialized block frame on the normal body path, preserving fiber suspension;
- stable local rebinding;
- explicit `CloseUpvalue` before captured loop-variable rebind;
- `continue` targets close/advance; `break` targets exit.

P5 replaces only the iteration authority and dataflow, not these control-flow guarantees.

## 6.3 Generalized lanes contain a confirmed ordering defect relative to the canonical spec

Current `compile_for_lanes` performs:

```text
source1; first cursor acquisition for source1
source2; first cursor acquisition for source2
...
```

The canonical iterator spec instead requires:

```text
source1
source2
...
iterator1
iterator2
...
first next-attempt
```

Therefore P5 must split the generalized preheader into two phases. This is observable whenever source expressions or `iterator` getters have side effects or fail.

## 6.4 Strict zip error identity is also a confirmed migration defect

The compiler currently raises:

```text
ArgumentError("strict zip lanes ended at different times")
```

The canonical specification names `ZipLengthMismatchError`. No production class with that name exists yet. P5 must introduce the ordinary language error declaration needed by the canonical contract and use it consistently for comma-separated `for` lanes and first-class strict zip.

## 6.5 The cursor protocol also leaks into positional expansion

`VM::has_iteration_protocol` still recognizes generic positional expansion by looking up `iterate(_)` and `iteratorValue(_)`. `PackTryExpandTuplePositionals` / `ListTryExpandTuplePositionals` use that probe to choose the generic iterable fallback.

This is not permission to keep cursor selectors after P5. It is a cross-owner retirement dependency. P5 must either migrate the expansion path to canonical iteration evidence through its existing compiler/lowering owner or stop and consult if genuinely dynamic expansion requires a runtime conformance facility that the language has not ratified. The VM must not replace cursor probing with runtime trait proof search.

---

# 7. Complete live `for` syntax inventory — final

The authoritative iterator specification and current AST agree on this conceptual grammar:

```text
for lane ("," lane)* block

lane:
    pattern ["at" ordinalBinding] "in" iterableExpression
```

Pass 1 therefore recognizes these live forms:

| Form | Example | Current lowering path | P5 status |
|---|---|---|---|
| simple single lane | `for x in xs {}` | ordinary fast path | migrate |
| pattern lane | `for (a, b) in xs {}` | generalized lanes | migrate/preserve |
| ordinal lane | `for x at i in xs {}` | generalized lanes | migrate/preserve |
| pattern + ordinal | `for (a, b) at i in xs {}` | generalized lanes | migrate/preserve |
| parallel lanes | `for x in xs, y in ys {}` | generalized lanes | migrate/preserve strict zip |
| parallel + ordinals | multiple lanes each with `at` | generalized lanes | migrate/preserve |
| nested loops | nested statements | ordinary control flow | preserve |
| `break` | inside body | loop-context jump | preserve |
| `continue` | inside body | step-target jump | preserve |
| `return`/throw | inside containing callable | normal callable/error exit | preserve |

Pass 2 closes labeled loops as **unsupported in the live language**. The AST documentation states that `break` carries no operand and no label and is the unlabelled form only; the compiler loop-context stack likewise targets the innermost enclosing loop. P5 must preserve that surface and must not add loop labels as part of iteration migration.

---

# 8. Current specialized `for` semantics

## 8.1 Pattern semantics

`for` lanes bind patterns, not only names. Canonical semantics are irrefutable-binding semantics: a yielded value that does not satisfy the pattern raises ordinary pattern-mismatch failure rather than being skipped.

The compiler's generalized lane path already tests the pattern and raises `for pattern did not match iterable value` on failure. P5 must retain the pattern engine rather than introducing iterator-specific destructuring logic.

## 8.2 `at` semantics

`at` is a zero-based encounter ordinal. It is not:

```text
collection index
Map key
Range value
iterator cursor
```

The current compiler stores a separate ordinal and assigns it to each lane's `at` binding. This is compatible with the canonical model: iterator implementation state must remain private to the iterator.

## 8.3 Parallel semantics

Parallel lanes are strict zip, not shortest-source zip:

```text
all Some -> bind all lanes, execute body
all None -> terminate normally
mixed Some/None -> raise length mismatch
```

Every attempted lockstep step must inspect every lane. If a mismatch is discovered, some iterators may already have advanced. P5 must not attempt rollback.

## 8.4 Evaluation and acquisition ordering — pass-2 closure

The canonical specification says that **all lane source expressions are evaluated exactly once, left-to-right, before traversal begins**, and that one iterator is obtained from each lane. Since `iterator` is an ordinary getter and may construct state or execute user behavior, the safest and most literal production rule is a two-phase preheader:

```text
phase 1 — source evaluation
    evaluate lane 0 source
    evaluate lane 1 source
    ...
    evaluate lane N source

phase 2 — iterator acquisition
    obtain lane 0 .iterator
    obtain lane 1 .iterator
    ...
    obtain lane N .iterator

phase 3 — traversal
    begin lockstep next reads
```

P5 should therefore **change** the current interleaved source/cursor setup to this two-phase ordering. This is observable and must be pinned with side-effect tests. It also keeps source evaluation semantics independent from iterator construction and gives all source expressions the same evaluation status before any traversal object exists.

For one lane the distinction collapses naturally: evaluate source once, obtain iterator once, then begin `next` reads.

---

# 9. Target evidence-driven `for` architecture

Pass 3 can now name the existing ownership seam precisely enough for planning.

## 9.1 Semantic product

`CallableAnalysis` is already the canonical per-callable semantic product and already owns per-expression trait-dispatch facts. P5 should extend it with a dedicated iteration-lane resolution index, conceptually:

```text
ForIterationLaneResolutionIndex
    key: lane source ExpressionId
    value: ForIterationLaneResolution
```

The exact Rust names may follow repository style; the semantic content may not be weakened. Each successful lane resolution must represent at least:

```text
source expression identity/range
proven source TraitRef / evidence for Iterable
normalized <Source as Iterable>::Item
normalized <Source as Iterable>::Iterator
proof that projected Iterator type |= Iterator
proof/equality Iterator::Item == Iterable::Item
selected invocation plan for Iterable.iterator
selected invocation plan for Iterator.next
proof/dependency provenance needed for invalidation/diagnostics
```

Pattern checking continues to consume the normalized Item type; pattern mechanics do not become part of trait proof.

`P5-FOR-23` — Iteration-lane proof is a first-class `CallableAnalysis` semantic product keyed to the lane source expression identity; it is not compiler-local metadata.

`P5-FOR-24` — The lane product participates in callable semantic fingerprinting and ordinary semantic dependency recording.

## 9.2 Projection into lowering

`phalcom-core/src/modules/semantic_lowering.rs` already turns semantic trait selections into `TraitInvocationSpec` / `TraitRequirementInvocationSpec` under `LoweringSite` keys. P5 should extend that projection with a dedicated iteration-lane lowering product, conceptually:

```text
LoweringSiteKind::ForIterationLane
ForIterationLaneLoweringSpec {
    iterator_invoke: exact-selected | requirement-selected
    next_invoke:     exact-selected | requirement-selected
    // runtime type recipes only when existing invocation machinery requires them
}
```

The spec should reuse the same selection/evidence/requirement-slot types already used by ordinary trait calls. It must not invent `ForTraitRegistry`, compiler-side conformance lookup, or a string-named trait lookup table.

`P5-FOR-25` — Module semantic lowering projects the semantic lane resolution into existing selected/requirement invocation machinery.

`P5-FOR-26` — Compiler lowering receives executable invocation specs; it never proves `Iterable`, normalizes associated projections, or selects witnesses itself.

## 9.3 Compiler execution shape

Canonical lowering is:

```text
preheader phase A:
    evaluate every lane source exactly once, left-to-right
    store each source

preheader phase B:
    invoke selected Iterable.iterator exactly once per lane, left-to-right
    store each iterator

attempt:
    invoke selected Iterator.next exactly once per lane
    keep each Option result

single lane:
    None -> exit
    Some(item) -> bind -> body

parallel lanes:
    all None -> exit
    all Some -> bind all -> body
    mixed -> ZipLengthMismatchError
```

Mismatch detection must use the already-read `Option` results. No lane is re-read and no rollback is attempted.

## 9.4 Derived/assumed evidence cases are not special compiler paths

The same product must cover:

- exact concrete Iterable conformance;
- conditional conformance;
- generic assumed `T |= Iterable`;
- generic `T |= Iterator` through the P4 derived law;
- cross-module conformance evidence;
- exact enum-case evidence where applicable.

The semantic checker may obtain those proofs differently, but lowering receives one normalized executable contract.

---

# 10. Evaluation-order requirements

The canonical order is now closed by `docs/specs/iterator.md` and must be executable-test-visible.

### Single lane

```text
1. evaluate source expression exactly once
2. invoke its selected `iterator` exactly once
3. repeatedly invoke `next`
4. bind `Some` payload before body
```

### Parallel lanes

```text
phase A: evaluate all source expressions left-to-right
phase B: acquire all iterators left-to-right
phase C: for each attempted step, call next once on every lane left-to-right
phase D: classify all results as all-Some / all-None / mixed
phase E: on all-Some, bind all patterns/ordinals then execute body
```

`P5-FOR-04` is therefore no longer an open question: **all source expressions precede any iterator acquisition**.

Consequences that tests must pin:

- a failure in source expression 2 happens before iterator getter 1 is invoked;
- a failure in iterator getter 1 happens only after every source expression has been evaluated;
- strict-zip mismatch may leave another iterator advanced because all lane `next` calls for the attempted step occur before classification;
- no body or pattern commit occurs for a mixed step;
- `break` performs no future `next` call;
- `continue` advances through the next ordinary attempt;
- per-iteration `CloseUpvalue` behavior remains before the next attempted advance.

---

# 11. Collection migration inventory — final

| Family | Current production relation | Current item behavior | Target `Iterable::Item` | Candidate iterator state | Pass-1 notes |
|---|---|---|---|---|---|
| `List<T>` | `class List<T> is Iterable` legacy class | indexed element | `T` | source + index | simplest pilot; reuse `_$length`/`_$at` |
| `Set<T>` | legacy `Iterable` superclass | `_$at(cursor)` encounter-order element | `T` | source + encounter index or native traversal state | preserve encounter order |
| `Map<K,V>` | legacy `Iterable` superclass | **key only** | `(key: K, value: V)` | map + encounter index/native traversal state | accepted behavior change |
| `Tuple` | legacy `Iterable` superclass | `_$at(cursor)` | `Dynamic` | tuple + index | includes labeled tuple storage shape but iteration item initially Dynamic |
| `Bytes` | legacy `Iterable` superclass | octet by index | `Int` | bytes + index | preserve byte encounter order |
| `Range` | legacy `Iterable` superclass | current cursor value | `Number` | current value + bounds/inclusive/exhausted state | dedicated migration |
| `MapKeysView` | legacy `Iterable` class | key | likely `K` | retained map + index | generic typing needs audit |
| `MapValuesView` | legacy `Iterable` class | value | likely `V` | retained map + index | generic typing needs audit |
| `MapEntriesView` | legacy `Iterable` class | `Entry` object | spec relationship to direct Map entries needs decision | retained map + index | compatibility surface |
| `IndexedIterable` | legacy `Iterable` class | tuple `(ordinal, item)` | typed pair/tuple | upstream iterator + ordinal | should become iterator-composing view |
| `ZippedIterable` | legacy `Iterable` class | tuple of lane values | tuple/dynamic shape | upstream iterators + exhaustion state | strict zip; should not rely on source `.size` |
| `String` | not iterable | N/A | N/A | N/A | remain non-Iterable unless explicit spec change |
| `StringByteSequence` | currently not an `Iterable` subclass; custom `each`/private cursor | bytes | current octet integer type | source string + byte offset | P8 explicitly deferred Iterable wiring; P5 should close it |
| `StringCodePointSequence` | currently not an `Iterable` subclass; custom `each`/private cursor | code points | current code-point numeric type | source string + byte offset | P8 explicitly deferred Iterable wiring; exact numeric type follows live authority |
| repo-defined subclasses such as `RefRange` | source code extends legacy `Iterable` | custom `size`/`iteratorValue` | source-specific | source-specific | must migrate or be explicitly compatibility-handled |

This inventory is intentionally broader than kernel collections. Removing the legacy superclass affects every source declaration that currently inherits from it, including examples and application code in the repository.

Pass 2 deepened two entries in this inventory.

First, the String sequence-view status is no longer conceptually open. The implementation record that introduced `StringByteSequence` and `StringCodePointSequence` explicitly described their standalone `size`/`at`/`each` implementation as a temporary integration state and deferred `Iterable` wiring until the iteration root existed. P5 is now that migration point. `String` itself should remain non-iterable because byte and code-point traversal are intentionally distinct views, while `string.bytes` and `string.codePoints` should become canonical `Iterable` sources. Their exact `Item` annotations must follow the live numeric type authority (`Bytes`-like octet integer for bytes; the current code-point numeric type for code points) rather than inventing a new character type in P5.

Second, repository-defined legacy `Iterable` subclasses are a real source-compatibility concern, not just tests. Examples such as Sheetcalc's `RefRange` inherit the old superclass specifically to gain cursor traversal. P5 must provide a clear source migration recipe: define a dedicated iterator object, conform that iterator to `Iterator`, conform the reusable source to `Iterable`, and move traversal state from cursor values into the iterator object.

Pass 3 must still turn the remote search into a mechanically exhaustive inventory against a local checkout before the implementation plan is emitted.

---

# 12. Stateful iterator design requirements

`P5-ITER-01` — Every canonical production iterator must store traversal state on the iterator object or an iterator-owned runtime representation, not in a caller-threaded cursor token.

`P5-ITER-02` — A reusable collection's `iterator` getter must produce independent traversal state.

`P5-ITER-03` — Iterator classes must conform to the canonical `Iterator` trait with an explicit/derived associated `Item` binding.

`P5-ITER-04` — Iterator classes automatically receive canonical `Iterable` behavior through the P4 derived rule; do not write a second compiler/runtime shortcut.

`P5-ITER-05` — Iterator `next` must be the sole primitive advancement interface exposed to generic loop lowering.

`P5-ITER-06` — State layout should be minimal and should reuse existing collection native access/traversal primitives rather than duplicate storage.

`P5-ITER-07` — Iterator-owned state must keep every required source object alive under GC.

`P5-ITER-08` — Lazy iterators must keep captured callables and upstream iterators alive.

`P5-ITER-09` — Sticky exhaustion must be explicit in iterator behavior where the remaining state alone cannot guarantee it.

`P5-ITER-10` — No iterator may implicitly restart after exhaustion or through `iterator`.

`P5-ITER-11` — Mutation-during-iteration behavior must not be invented. Existing defined behavior must be preserved; undefined behavior should remain unspecified unless migration forces a decision.

`P5-ITER-12` — Stateful iterator implementation must not require conformance-local storage, which Phalcom `impl` semantics forbid. Iterator state belongs in concrete iterator object representations.

---

# 13. Iterator lifecycle requirements

The minimum lifecycle acceptance matrix is:

```text
source.iterator -> traversal A
source.iterator -> traversal B
A and B are independent

A.next -> Some(a)
A.next -> Some(b)
A.next -> None
A.next -> None

A.iterator is A
A.iterator.next continues A; it does not restart
```

Additional requirements:

- breaking a loop leaves the source collection structurally valid;
- constructing/using a second iterator over the same reusable source remains valid after breaking the first;
- nested/re-entrant iteration over the same source uses distinct iterator state;
- GC during an iterator's lifetime cannot reclaim the retained source/upstream/callable evidence needed by later `next` calls;
- an error raised by `next` follows normal failure semantics; P5 need not define restartability after error unless an existing contract already does.

---

# 14. Map migration requirements

Map is an intentional semantic migration, not a mechanical adapter rewrite.

Canonical direct iteration is:

```text
Map<K, V>::Item = (key: K, value: V)
```

The current production `Map.iteratorValue` yields only keys, while `Map.entries` allocates a nominal `Entry` object. The newer canonical iterator work explicitly uses the labeled tuple and rejects substituting the legacy nominal `Entry` merely because it exists in production.

Pass 3 closes the representation direction:

`P5-MAP-01` — Direct Map iteration yields labeled `(key: K, value: V)` tuples in encounter order.

`P5-MAP-02` — Preserve `K` and `V` through the associated Item projection.

`P5-MAP-03` — `MapIterator<K,V>` retains the Map plus minimal position/exhaustion state and reads existing native key/value slots; do not copy the full Map.

`P5-MAP-04` — Do not alter insertion-order, equality, hashing, or mutation policy as part of iteration migration.

`P5-MAP-05` — Construct the canonical labeled tuple using the existing Tuple/product representation; do not add an iterator-specific pair object.

`P5-MAP-06` — `MapKeysView`, `MapValuesView`, and `MapEntriesView` migrate to canonical `Iterable` conformances and stateful iterators.

`P5-MAP-07` — Direct `map.contains(key)` / `map.includes(key)` remain Map key-membership operations; generic Iterator membership applies to `map.iter` entries, not to the Map's representation-specific key API.

`P5-MAP-08` — `for (key, value) in map` and labeled-pattern/property access must observe the canonical entry tuple shape.

`P5-MAP-09` — `Map.entries` converges on the same canonical labeled tuple item shape unless a newer authoritative spec explicitly requires a distinct Entry value type.

`P5-MAP-10` — The legacy `Entry` class is removable from the iteration path. Current repository search found its production construction in `MapEntriesView`; older conversion specs rely on `.key`/`.value`, and labeled tuples already support those labeled accesses. Before deletion, implementation must run a final repository usage search and migrate any real external-facing fixture that still requires nominal `Entry` identity.

This lets `toMap` retain its ergonomic `entry.key` / `entry.value` implementation while consuming the canonical tuple shape.

---

# 15. Range migration requirements

The current Range implementation is a bounds descriptor with cursor-based forward integer iteration:

- lower bound is mandatory for iteration;
- current cursor is the yielded numeric value;
- progression is `+1`;
- upper bound may be omitted;
- upper endpoint may be inclusive/exclusive;
- lower > upper with an upper bound raises an error for iteration;
- descending traversal is explicitly unsupported;
- there is no step parameter/model in the live Range source.

Therefore the initial `RangeIterator` should represent the existing semantics, not an imagined richer range system.

`P5-RANGE-01` — `Range` must conform to canonical `Iterable` with `Item = Number` unless a newer normative type rule supersedes it.

`P5-RANGE-02` — `Range.iterator` must construct a stateful `RangeIterator`.

`P5-RANGE-03` — `RangeIterator.next` must preserve existing inclusive/exclusive and bounded/unbounded behavior.

`P5-RANGE-04` — Missing lower bound must preserve current iteration failure behavior unless explicitly changed by normative authority.

`P5-RANGE-05` — Empty/equal-bound edge cases must be tested independently for exclusive and inclusive forms.

`P5-RANGE-06` — Descending and step behavior must be classified from current authoritative specs; do not add them merely because the handoff asks to audit them.

`P5-RANGE-07` — Range-specific logic must remain inside Range/RangeIterator, not in `for` lowering.

`P5-RANGE-08` — The positive legacy baseline's use of `Option.unwrapOr` inside Range must be re-evaluated after migration; a canonical iterator implementation should avoid retaining obsolete cursor mechanics solely to preserve that call path.

Pass 2 resolves the Range scope from the repository's collection-evolution authority: P5's production migration should support the already-established forward integer subset (`a..b`, `a..=b`, `a..` with implicit `+1`) and **must not add** descending traversal, stored step state, `by(step)`, or `Progression`. Reversed/lowerless unsupported traversal remains a catchable language failure. This is now a P5 non-feature rather than an open design invitation.

`P5-RANGE-09` — P5 must not add Range stepping, automatic descending traversal, or Progression semantics.

`P5-RANGE-10` — `a..` remains a genuine non-exhausting source; iterator adapters such as `take` must be able to bound it lazily without requiring a fake `size`.

---

# 16. Production-ready iteration surface and lazy adapter migration — final design

Pass 3 makes the user-requested ergonomics a concrete production requirement while resolving the selector conflict that a naive “put everything on both traits” design would create.

Current Rust 1.98.1 documentation still presents `Iterator` with one required associated type (`Item`), one required method (`next`), and a large provided/default surface (76 documented methods total on the current page). Phalcom should adopt that **leverage**, not Rust's ownership model or method-count target.

## 16.1 Final role split: two tiny requirements, one broad inherited experience

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
  iterator { self }

  // lazy/state-transforming adapters
}

trait Iterable
  where
    Self::Iterator |= Iterator,
    <Self::Iterator as Iterator>::Item == Self::Item
{
  type Item
  type Iterator
  iterator -> Self::Iterator

  // non-conflicting source conveniences + traversal consumers/terminals
  iter { self.iterator }
}
```

The P4 derived rule remains decisive:

```phalcom
impl<I> Iterable for I
  where I |= Iterator
```

Therefore a custom iterator implementing only `Item + next` receives:

```text
Iterator adapter defaults
        +
derived Iterable conformance
        +
Iterable terminal/convenience defaults
        +
for-loop participation
```

This is the Phalcom equivalent of Rust's “implement the primitive, get the toolbox for free.”

`P5-SURFACE-01` — The normal custom `Iterator` implementation burden is `Item + next`; derivable behavior is provided by defaults/adapters.

`P5-SURFACE-02` — A non-Iterator `Iterable` returns a fresh independent iterator on each `iterator` access.

`P5-SURFACE-03` — `.iter` is a universal ergonomic alias for `.iterator`, not a wrapper protocol. The legacy `SourceIterator` is unnecessary in the canonical model and should disappear unless a real non-iteration use is found.

`P5-SURFACE-04` — Direct collection `map`/`filter`/`flatMap`-style transformations preserve accepted eager semantics; lazy transformation starts explicitly from `.iter` / `.iterator`.

## 16.2 Which defaults belong on which trait

The selector rule is semantic, not stylistic.

### `Iterator` defaults: operations that transform one traversal into another

These are lazy and retain/consume the receiver's current traversal state:

```text
map<U>
filter
filterMap<U>
flatMap<S>
flatten
chain<J>
skip
skipWhile
take
takeWhile
mapWhile<U>
inspect
zip              # strict Phalcom zip
```

### `Iterable` defaults: operations whose meaning is the same for a reusable source and an Iterator

They begin by obtaining `self.iterator`. For a reusable source this is a fresh traversal; for an Iterator the P4 derived witness returns `self`, so the operation consumes that same traversal.

```text
iter
indexed          # construct IndexedIterator from self.iterator

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

`indexed` is intentionally placed here rather than duplicated on `Iterator`: `items.indexed` can always create one `IndexedIterator` from `items.iterator`, and an Iterator receives the same convenience through its derived Iterable conformance.

### Concrete collection inherent methods: selector-conflicting eager transforms and representation-specific operations

Core reusable collections retain accepted direct eager transforms through thin inherent forwarders:

```text
map
map(indexed:)
filter
flatMap
```

Conceptually these can be implemented with one fresh iterator and the canonical iterator algorithms, for example:

```text
collection.map(f)
    -> collection.iterator.map(f).toList
```

Representation-specific selectors stay inherent too: `Map.contains(key)` is key membership, `List` mutation is List behavior, `isEmpty` may use O(1) collection state, and so on.

This arrangement preserves arbitrary custom-Iterable ergonomics for terminals (`source.toList`, `source.find(where:)`, etc.) while avoiding the single problematic overlap: eager collection transforms versus lazy iterator transforms.

`P5-SURFACE-09` — Default algorithms have one semantic implementation by role: lazy transformations on `Iterator`; traversal consumers/terminals on `Iterable`. Do not duplicate the same loop per collection.

`P5-SURFACE-10` — Built-in direct eager transformation selectors may be thin inherent forwarding wrappers; wrapper duplication is acceptable, traversal-algorithm duplication is not.

`P5-SURFACE-11` — Ordinary member resolution must expose non-conflicting `Iterable` defaults through applicable derived `Iterator => Iterable` evidence. P5 must not invent an Iterator-vs-Iterable precedence rule for overlapping selectors; the design avoids such overlap instead.

## 16.3 One-shot iterator lifecycle and aliasing

Canonical lazy pipelines are stateful single-pass iterator objects:

```phalcom
const p = xs.iter.map(f)
p.toList   // consumes p through derived Iterable terminal default
p.toList   // p remains exhausted
```

A fresh traversal is obtained by reconstructing from the reusable source:

```phalcom
xs.iter.map(f).toList
```

Adapters retain the actual upstream iterator object. Because Phalcom references are aliasable rather than Rust-moved, aliases intentionally share advancement:

```phalcom
const it = xs.iterator
const mapped = it.map(f)
it.next
mapped.next   // continues the same upstream traversal
```

`P5-SURFACE-05` — Adapter values are stateful one-shot Iterators.

`P5-SURFACE-06` — Legacy tests expecting the same pipeline object to restart are migrated; restart means reacquire from a reusable source.

`P5-SURFACE-07` — Upstream aliases share traversal state.

`P5-SURFACE-08` — No hidden move/invalidation semantics are introduced.

## 16.4 Required P5 default surface

The surface in §16.2 is the P5 completion target, not merely a wishlist. It contains the useful cursor-era API plus a focused production expansion: `filterMap`, `flatten`, `chain`, `skipWhile`, `mapWhile`, `inspect`, `nth`, `last`, and strict pairwise `zip`.

The implementation plan may stage this over several patches, but P5 must not delete the runtime superclass that currently supplies generic traversal behavior and leave the canonical traits with only `next`/`for`.

`P5-SURFACE-12` — Preserve current useful traversal/query/reduction/materialization behavior through canonical trait defaults.

`P5-SURFACE-13` — Add the high-value derivable methods listed above where current generic syntax can state their constraints; pass 3 found generic trait methods, ordinary `where` constraints, and associated projections/equalities already present in the language architecture.

`P5-SURFACE-14` — `zip` is strict: all Some yields a tuple, all None ends, mixed results raise `ZipLengthMismatchError`.

## 16.5 Generic shapes and state ownership

Conceptually:

```text
MapIterator<I,U>          upstream + mapper
FilterIterator<I>         upstream + predicate
FilterMapIterator<I,U>    upstream + mapper
FlatMapIterator<I,S>      outer + mapper + currentInnerIterator?
FlattenIterator<I>        outer + currentInnerIterator?
ChainIterator<I,J>        first + second + phase
SkipIterator<I>           upstream + remaining/initialized
SkipWhileIterator<I>      upstream + predicate + stillSkipping
TakeIterator<I>           upstream + remaining + done
TakeWhileIterator<I>      upstream + predicate + done
MapWhileIterator<I,U>     upstream + mapper + done
InspectIterator<I>        upstream + callback
IndexedIterator<I>        upstream + ordinal
StrictZipIterator<I,J>    left + right + done
```

Representative constraints:

```text
flatMap<S>: mapper Item -> S where S |= Iterable
flatten:    Self::Item |= Iterable
chain<J>:  J |= Iterator and J::Item == Self::Item
filterMap: mapper Item -> Option<U>
mapWhile:  mapper Item -> Option<U>
```

Exact source signatures belong in implementation planning; the semantic constraints above are fixed.

The GC already traces every ordinary instance slot, so upstreams, callbacks, current inner iterators, and buffered values should be ordinary fields. No new native iterator heap arm is required for rooting.

## 16.6 Adapter laws

`P5-LAZY-01` — Adapters retain upstream `Iterator` objects, never cursor tokens.

`P5-LAZY-02` — Every adapter's `next` returns canonical `Option<Item>`.

`P5-LAZY-03` — `map` performs zero callback work at construction and one mapper call per yielded upstream item.

`P5-LAZY-04` — `filter` invokes the predicate once per inspected upstream item and may consume many upstream items for one output.

`P5-LAZY-05` — `skip` applies its prefix count once per iterator lifetime.

`P5-LAZY-06` — `take(0)` and exhausted `take` do not advance upstream.

`P5-LAZY-07` — `takeWhile` is permanently exhausted after the first rejected item.

`P5-LAZY-08` — `flatMap` stores the current inner iterator so mapper code runs once per reached outer item.

`P5-LAZY-09` — `indexed` exposes a zero-based encounter ordinal independent of upstream representation state.

`P5-LAZY-10` — Adapter composition remains lazy; no intermediate List materialization is a valid implementation shortcut.

`P5-LAZY-11` — `filterMap` skips mapper `None` and yields `Some` payloads.

`P5-LAZY-12` — `flatten` exhausts each inner iterator before advancing outer and skips empty inners.

`P5-LAZY-13` — `chain` switches to the second iterator exactly once after the first exhausts.

`P5-LAZY-14` — `skipWhile` stops invoking its predicate after the first retained item.

`P5-LAZY-15` — `mapWhile` becomes sticky-exhausted at the first mapper `None`.

`P5-LAZY-16` — `inspect` executes exactly once per yielded item and returns the original item.

`P5-LAZY-17` — strict zip invokes each lane once per attempted step and never truncates silently.

`P5-LAZY-18` — all retained state uses ordinary traced object fields/values.

## 16.7 Sticky exhaustion is stronger than Rust's base contract

Phalcom requires repeated `next` after `None` to remain `None`. Therefore every adapter is intrinsically fused. No public `fuse` method is needed, and early-terminating adapters retain a terminal state when necessary.

`P5-SURFACE-17` — Sticky exhaustion is universal and cannot be weakened by a default adapter.

## 16.8 Collection-specific behavior remains collection-specific

A broad default toolbox must not erase receiver-specific meanings:

- `Map.contains(key)` remains key membership, even though `map.iter.contains(entry)` uses generic entry membership.
- `isEmpty` stays on reusable/concrete sources; it is not a destructive Iterator getter.
- random access, mutation, capacity, sorting, optimized accessors, and family-preserving transforms remain concrete collection concerns unless separately ratified.

`P5-SURFACE-18` — Iterator/Iterable breadth does not collapse concrete collection semantics into one generic surface.

## 16.9 Explicitly deferred Rust-like methods

P5 deliberately defers:

```text
by_ref / copied / cloned     Rust ownership/reference concerns
fuse                         redundant under sticky exhaustion
rev / rposition              needs double-ended capability
cycle                         needs repeatable/cloneable traversal contract
size_hint / exact-size APIs  need a dedicated cardinality capability
sum / product                need numeric aggregation contracts
min/max/sorted families      depend on ordering policy
collect                      needs a collection-builder / FromIterator analogue
try_* family                 needs deliberate short-circuit Result abstraction
peekable / scan / chunks / windows / intersperse
                             useful follow-on breadth, not required for P5 closure
step_by                      belongs with Progression/step semantics
```

`P5-SURFACE-15` — Production readiness is semantic breadth with clear ownership, not a method-count contest.

`P5-SURFACE-16` — Keep explicit `toList` / `toSet` / `toMap` until a generic builder/collection-construction trait exists.

## 16.10 Performance posture

Defaults stay source-authored where practical. P5 does not require a VM primitive per adapter. The target cost model is:

```text
proof / projection normalization      compile/semantic time
iterator construction                 once per traversal
adapter object construction           once per pipeline stage
per output item                       already-selected next dispatch + local state/callback work
runtime proof search                  none
cursor-token protocol allocation      none
```

Later inlining/fusion/specialization may optimize the same semantics.

`P5-SURFACE-19` — Rich defaults do not expand the native primitive floor without a measured, separately justified need.

`P5-SURFACE-20` — Adapter fusion/inlining is follow-on optimization; P5 correctness stands without it.
---

# 17. Positive legacy iteration baseline diagnosis

P4 recorded the positive `language-corpus::iteration` lane as baseline-blocked by:

```text
Option.unwrapOr
generic call-entry type recipe could not be instantiated
```

Pass 1 could verify the recorded failure but could not reproduce it locally because no executable checkout was available.

The failure remains a mandatory P5 classification item because production iteration is now P5's responsibility.

Implementation preflight must determine the earliest broken invariant:

```text
A. generic call-entry/runtime environment bug also hit by canonical Iterator.next
B. legacy cursor-only Option usage that disappears in the new path
C. Range-specific Option usage
D. generic Option return/dispatch issue independent of iteration
E. another production iteration defect hidden by the observed symptom
```

A P5-local fix is justified only if the canonical iteration path requires the same broken generic machinery or the bug prevents migration verification. A broad generic-runtime repair remains outside P5.

One additional pass-1 finding is that comments in `compile_for` still describe an older `$cursor.unwrapOr(0)` desugaring while the current code uses `JumpIfNone` and passes the bare cursor to `iteratorValue`. That stale commentary should not be used as diagnostic evidence.

---

# 18. Generic evidence / associated-projection interactions

`P5-GEN-01` — `for<T where T |= Iterable>` must type-check from lexical assumed evidence without requiring an exact nominal conformance at declaration-check time.

`P5-GEN-02` — Loop element type is `<T as Iterable>::Item`, normalized through existing evidence machinery.

`P5-GEN-03` — Loop iterator type is `<T as Iterable>::Iterator`.

`P5-GEN-04` — The trait contract must prove `<T as Iterable>::Iterator |= Iterator` and equality of iterator/item projections; `for` must not restate this as a private special rule.

`P5-GEN-05` — `for<T where T |= Iterator>` must work through the P4 guarded derived conformance to `Iterable`.

`P5-GEN-06` — Conditional conformances and nested evidence must remain usable as loop evidence.

`P5-GEN-07` — Cross-module conformance evidence must survive semantic publication/lowering exactly as ordinary trait calls do.

`P5-GEN-08` — P5 must not widen or erase associated projections merely to simplify compiler lowering.

---

# 19. Exact enum-case preservation

The formerly ignored `exact_enum_case_conformance_executes` regression is active and recorded passing after C6 predecessor work.

P5 must preserve:

```text
exact enum case expression
    -> TypeData::ExactCase
    -> generic inference keeps exact identity
    -> exact/derived conformance proof
    -> runtime evidence
```

Iteration-specific requirements are:

- an exact enum case used as an iterable generic argument must not be widened unless ordinary language typing requires it;
- an exact-case item yielded by an iterator must preserve exact identity through `Option<Item>` extraction and loop binding;
- generic calls in the loop body must receive the same exact evidence they would outside a loop;
- semantic-to-lowering iteration products must carry types/evidence, not reconstruct them from runtime class identities.

---

# 20. Runtime execution requirements

`P5-RT-01` — VM execution remains trait-proof-free.

`P5-RT-02` — Associated projection normalization occurs in semantic analysis, never per `next`.

`P5-RT-03` — Generic assumed evidence uses the existing requirement-invocation machinery.

`P5-RT-04` — Exact trait selections use the existing selected invocation machinery projected by `ModuleLoweringSemantics`.

`P5-RT-05` — `for` performs no runtime string lookup for traits or cursor selectors.

`P5-RT-06` — Any runtime type/conformance environment recipe needed by selected calls is prepared at existing call/setup granularity rather than reconstructed per item.

`P5-RT-07` — `next` branching consumes canonical `Option<Item>` without `unwrapOr` cursor tricks; `Some(None)` remains distinguishable from exhaustion when `Item` itself can be Option/None-like data.

`P5-RT-08` — Direct jump-based loop control and fiber-safe body execution are preserved.

`P5-RT-09` — Existing close-before-rebind loop-capture behavior is preserved.

`P5-RT-10` — Strict zip operates only on iterator `next` results, not size/cursor APIs.

`P5-RT-11` — Iterator/adaptor state is ordinary traced object state. `Object::Instance` tracing already visits every slot, so no raw untracked pointer may replace an upstream/source/callback reference.

`P5-RT-12` — Iterator exhaustion/current-inner state remains correct across GC and arbitrary allocation safepoints.

`P5-RT-13` — Positional `*` expansion's live `VM::has_iteration_protocol` cursor probe is a P5 retirement dependency. It must be migrated through the argument-expansion/compiler owner before `iterate`/`iteratorValue` are deleted.

`P5-RT-14` — Migrating dynamic expansion must not add runtime conformance proof search. If the existing dynamic-expansion contract cannot be represented using precomputed semantic evidence or an already-ratified runtime capability, implementation must STOP/CONSULT rather than reintroduce structural cursor probing under another name.

---

# 21. P5-local performance requirements

P5 performance scope is limited to the production iteration hot path.

## 21.1 Required operation classification

| Operation | Target frequency |
|---|---|
| prove `T |= Iterable` | semantic/compile-time only |
| normalize `Iterable::Item` | semantic/compile-time only |
| normalize `Iterable::Iterator` | semantic/compile-time only |
| prove iterator trait contract | semantic/compile-time only |
| select `Iterable.iterator` witness | semantic/compile-time; encode reusable runtime recipe |
| select `Iterator.next` witness | semantic/compile-time; encode reusable runtime recipe |
| evaluate each source expression | once per loop execution |
| acquire each iterator | once per loop execution / lane |
| instantiate/intern stable evidence environment | setup/call-entry when needed, not per item |
| invoke `next` | once per attempted step per lane, subject to adapter-internal consumption |
| inspect Option | once per `next` result |
| bind item | once per successful yielded step |
| body | once per successful yielded step |

## 21.2 Requirements

`P5-PERF-01` — No semantic conformance proof per item.

`P5-PERF-02` — No associated projection normalization per item.

`P5-PERF-03` — No repeated iterator acquisition per item.

`P5-PERF-04` — No avoidable runtime evidence-environment reconstruction/interning per item.

`P5-PERF-05` — `next` should not perform avoidable hash-map trait lookup when a selected witness/slot can be carried.

`P5-PERF-06` — Stateful iterator allocation is permitted once per traversal; independent-traversal semantics outrank allocation avoidance.

`P5-PERF-07` — Per-item temporary heap allocation should be avoided where representation already permits immediate/stack-managed `Option` and scalar/index state.

`P5-PERF-08` — Map's required entry value may require tuple construction; measure/inspect the real tuple representation before optimizing it away.

`P5-PERF-09` — Lazy adapter composition must not introduce eager intermediate collection allocation.

`P5-PERF-10` — General dispatch/value-layout/DispatchKey optimizations are P6 or another performance project unless a concrete P5 hot-path regression requires a bounded correction.

---

# 22. Incremental dependencies introduced by migration

Pass 3 identifies the concrete product seam: the iteration-lane resolution belongs inside `CallableAnalysis`, alongside existing expression/match/flow products, and its executable projection belongs in `ModuleLoweringSemantics`.

A lane product should depend narrowly on:

```text
source expression semantic identity/type
Iterable trait declaration/surface
applicable exact/assumed/derived Iterable evidence
Iterable::Item binding
Iterable::Iterator binding
Iterator trait declaration/surface
proof for associated iterator
Iterator::Item equality/normalization
selected iterator requirement/default/witness
selected next requirement/default/witness
```

Pattern-flow dependencies remain owned by existing pattern/flow analysis.

`P5-INC-01` — Body-only edits do not force global conformance rediscovery.

`P5-INC-02` — Unrelated declaration/module edits do not invalidate every loop.

`P5-INC-03` — Applicable Iterable conformance/evidence changes invalidate affected lane products.

`P5-INC-04` — Item/Iterator associated-binding changes invalidate affected typing and lowering.

`P5-INC-05` — Selected iterator/next witness/default changes invalidate affected executable semantics.

`P5-INC-06` — No second coarse iteration registry or manual global epoch is introduced.

`P5-INC-07` — `CallableAnalysis` fingerprinting must include the new lane-resolution product in the same provenance-sensitive/range-free style used by neighboring semantic products; omission would permit stale lowering after conformance changes.

`P5-INC-08` — Semantic dependency recording during proof/normalization must reuse existing `SemanticDependencies` categories rather than invent iteration-only invalidation edges.

`P5-INC-09` — `ModuleLoweringSemantics` must change when the projected iterator/next invocation plan changes even if unrelated loop-body semantics do not.

The implementation plan should include a focused incremental test that edits a selected conformance/default and observes only affected consumers becoming stale/recomputed.

---

# 23. Legacy cursor retirement ledger — final checked scope

Pass 3 expands the retirement ledger beyond `for` and library classes.

| Legacy surface | Current owner/use | P5 disposition |
|---|---|---|
| `Iterable.iterate(cursor)` | `collections/iterable.ph` + subclasses | replace with `iterator`/stateful iterator; delete |
| `iteratorValue(cursor)` | collections/views/adapters | replace with `next`; delete |
| `None` cursor sentinel | compiler/library | delete from iteration protocol |
| `$for_cursor*` locals | compiler loops | replace by iterator + Option-result locals |
| semantic cursor-member probing | `checker/statement.rs` | replace by Iterable proof/projections |
| compiler `iterate`/`iteratorValue` sends | `compiler/lib/loops.rs` | replace by lane lowering specs |
| `VM::has_iteration_protocol` | generic positional `*` probe | migrate via argument-expansion owner; delete cursor lookup |
| legacy `@native class Iterable<T>` | runtime superclass + algorithms | split responsibilities; delete class |
| legacy `class Iterator is Iterable` | lazy API root | replace with trait defaults + adapter classes |
| `SourceIterator` | wrapper around cursor Iterable | delete; canonical Iterable already constructs iterator |
| cursor Map/Filter/Skip/Take/TakeWhile/FlatMap adapters | lazy library | redesign as stateful Iterator objects |
| `IndexedIterable` cursor state | value-level indexed traversal | replace with iterator-based implementation |
| `ZippedIterable` cursor tuples/size mismatch | strict zip view | replace with strict iterator advancement |
| `Entry` as MapEntries iteration payload | map view | converge to labeled tuple unless final usage audit finds separate authority |
| old current specs/guides/LSP hover/AST comments | public/tooling text | update to canonical model |
| accepted historical cursor ADRs | history | preserve as history but mark/situate superseding canonical authority where repository convention requires |

`P5-RETIRE-01` — Do not delete a legacy surface before every live production consumer has migrated.

`P5-RETIRE-02` — P5 ends with one production iteration model, not a canonical trait façade over cursor execution.

`P5-RETIRE-03` — Historical documents may remain historical; current specifications/guides/tooling may not teach the cursor protocol as current behavior.

`P5-RETIRE-04` — Any temporary compatibility bridge has an explicit deletion patch/gate.

`P5-RETIRE-10` — Delete the legacy Iterator runtime class only after every adapter and API consumer has moved to trait defaults/stateful iterator classes.

`P5-RETIRE-11` — Remove stale core/global/sealed/native metadata identity for legacy Iterable.

`P5-RETIRE-12` — Final production search must find no `iterate`/`iteratorValue` iteration implementation or lowering use; historical docs may still contain the terms with superseded context.

`P5-RETIRE-13` — Final cursor search includes VM expansion probes and compiler pack/list spread paths, not only collection source files.

`P5-RETIRE-14` — Final search also checks comments/LSP hover/current docs so developer tooling does not describe deleted mechanics.

---

# 24. Legacy runtime `Iterable` superclass retirement

The live VM still allocates a core class named `Iterable` and wires List/Map/Set/Tuple/Range/Bytes through that superclass. `CoreClasses`, bootstrap sealing, primitive/core-class inventories, and universe invariants retain its identity.

P5 retirement is therefore a real runtime identity migration:

```text
1. establish explicit Iterable conformances for reusable built-ins/views
2. establish concrete stateful Iterator classes + Iterator conformances
3. move generic algorithms to Iterator defaults
4. add/retain thin inherent eager collection conveniences as needed
5. migrate repository-defined legacy Iterable subclasses (for example RefRange)
6. migrate compiler/VM consumers of cursor capability
7. restore kernel collection superclass to ordinary Object architecture
8. delete Iterable runtime class + legacy Iterator class
9. delete CoreClasses/bootstrap/sealed/primitives/invariant references
10. verify reflection/tooling no longer reports class inheritance as iterability
```

`P5-RETIRE-05` — Trait evidence, not superclass ancestry, is the semantic source of iterability.

`P5-RETIRE-06` — Audit/remove every `iterable_class` field/use in core class construction, bootstrap, primitives, invariants, globals, reflection metadata, and class-ID assumptions.

`P5-RETIRE-07` — Kernel collection runtime superclass becomes the existing ordinary non-iteration superclass (`Object` unless a newer class-specific authority says otherwise); do not encode traits as superclasses.

`P5-RETIRE-08` — Reflection must expose trait conformance/effective behavior through the trait architecture rather than preserving a fake Iterable superclass explanation.

`P5-RETIRE-09` — Repository source subclasses of legacy Iterable are migrated before deletion.

No replacement `IterableBase`/hidden superclass is introduced merely to keep inherited algorithms. Thin inherent collection wrappers are preferable to a second disguised runtime iteration root.

---

# 25. Canonical public-name transition

The final naming contract is closed:

`P5-NAME-01` — `Iterator` publicly means the canonical trait.

`P5-NAME-02` — `Iterable` publicly means the canonical trait.

`P5-NAME-03` — No same-scope ambiguity between legacy classes and canonical traits survives the cutover.

`P5-NAME-04` — Preserve the P4 trait declaration identities/module provenance when mechanically possible; `iterator_laws.ph` may remain the declaration module if changing its path would create unnecessary identity churn.

`P5-NAME-05` — `collections/package.ph` exposure must stop presenting the legacy class module as a competing public authority once its migration consumers are gone.

`P5-NAME-06` — Current guides, docs, examples, source-index/LSP help, and tests use the final trait meaning in the same completion sequence.

The implementation plan should stage name retirement only after semantic/compiler/library consumers are already able to operate on the traits; name switching is not the first patch.

---

# 26. Library code documentation/style requirements

`P5-DOC-01` — New iterator classes document public abstraction and lifecycle, not LANG005 task history.

`P5-DOC-02` — Comments must describe Phalcom semantics, not Rust implementation details unless those details are part of the public contract.

`P5-DOC-03` — Remove stale cursor-oriented comments from migrated production source.

`P5-DOC-04` — Do not leave comments such as “temporary for P5”, “replaces P4”, task IDs, agent instructions, or planning rationale in library source.

`P5-DOC-05` — Follow neighboring Universe formatting/member-order conventions; do not broadly reformat unrelated files.

`P5-DOC-06` — Iterator docs should explicitly state item type, order, freshness/identity, exhaustion, and relevant edge cases where public.

---

# 27. External reference research and ergonomic benchmark — final

Passes 2 and 3 used current official Rust documentation as an architecture benchmark, not as normative Phalcom authority.

As checked against Rust 1.98.1 documentation (2026-09-01):

```text
Iterator required associated types: Item
Iterator required methods:          next
current documented methods:         76 total
```

The module documentation explicitly describes the broader API as default methods built on `next`. This strongly validates the requested Phalcom goal: **tiny implementation burden, broad derived surface**.

The useful transfers are:

- separate reusable source conversion from iterator advancement;
- put generic traversal algorithms on the iterator abstraction;
- represent lazy transforms as composable state machines;
- make user iterators immediately useful without reimplementing combinators.

The non-transfers are equally important:

| Rust concept | Phalcom decision |
|---|---|
| base Iterator may resume after `None` | Phalcom sticky exhaustion is mandatory |
| ownership/moves often prevent upstream aliases | Phalcom aliases share iterator state |
| `enumerate` | use existing `indexed` / `at` vocabulary |
| `zip` truncates to shortest | Phalcom zip is strict and raises mismatch |
| `fuse` | unnecessary because all Phalcom iterators are fused by contract |
| `by_ref`, `copied`, `cloned` | ownership/reference-specific; do not copy |
| `collect` / `FromIterator` | defer until collection-construction capability is designed |
| `rev` | requires a separate double-ended capability |
| `step_by` | belongs with Progression/step semantics |

The most important pass-3 refinement is also Rust-like: `IntoIterator` does not need to own all `Iterator` algorithms. Likewise Phalcom's `Iterable` should remain a small traversal-factory contract rather than becoming a second giant algorithm trait with selector conflicts.

References consulted:

```text
https://doc.rust-lang.org/std/iter/
https://doc.rust-lang.org/std/iter/trait.Iterator.html
https://doc.rust-lang.org/std/iter/trait.IntoIterator.html
```

Rust is evidence that the architecture scales; Phalcom's own trait/evidence/selector semantics remain authoritative.

---

# 28. Final migration sequence for implementation planning

This is still requirements-level sequencing, not the patch plan. Pass 3 closes the dependency order the plan should respect.

```text
0. PRE-FLIGHT
   Re-ground local worktree; reproduce/classify P4 iteration baseline block.

1. SEMANTIC PRODUCT
   Add ForIterationLaneResolution to CallableAnalysis.
   Prove Iterable, normalize Item/Iterator, select iterator/next evidence.
   Fingerprint and dependency-record it.

2. LOWERING PROJECTION
   Project lane resolutions into ModuleLoweringSemantics using existing
   selected/requirement invocation machinery.

3. LIST VERTICAL
   Add ListIterator stateful class + Iterator conformance.
   Add List Iterable conformance / iterator factory.
   Prove canonical next/Option path independently of for lowering.

4. SINGLE-LANE FOR CUTOVER
   Make semantic for consume Iterable evidence.
   Make simple compiler for consume lane lowering spec.
   Preserve jump/fiber/capture behavior.

5. GENERALIZED FOR CUTOVER
   Patterns + at + parallel lanes.
   Two-phase source/iterator preheader.
   One next per lane per attempt.
   ZipLengthMismatchError.

6. CORE ITERABLE FAMILIES
   Set, Tuple, Bytes, string byte/code-point views, map views.
   Add stateful iterators/conformances; no cursor fallback.

7. MAP VERTICAL
   Direct Map Item -> labeled (key,value).
   MapEntriesView converges to same item shape.
   Migrate Entry-dependent conversions/usages.

8. RANGE VERTICAL
   Stateful RangeIterator, preserving only ratified forward behavior.
   No step/descending invention.

9. ITERATOR DEFAULT TOOLBOX
   Move generic algorithms to canonical Iterator defaults.
   Add required production adapters/terminals.
   Add thin built-in eager forwarding methods where direct collection API requires them.
   Make .iter a canonical convenience over iterator; delete SourceIterator.

10. BOUNDEDNESS INTEGRATION
    Update existing compiler boundedness propagation/catalogs for canonical
    .iterator/.iter and new lazy adapters; preserve reject-only-proven-unbounded posture.

11. ARGUMENT-EXPANSION CURSOR DEPENDENCY
    Migrate Pack/List positional expansion away from VM cursor-selector probing
    through its existing owner. If dynamic conformance semantics are missing, STOP/CONSULT.

12. REPOSITORY SUBCLASSES / USE SITES
    Migrate RefRange and any remaining legacy Iterable subclasses/users.

13. RUNTIME CLASS + PUBLIC NAME RETIREMENT
    Remove legacy Iterable/Iterator classes, iterable_class bootstrap identity,
    old superclass links, competing package exposure.

14. DOCS / TOOLING / CURSOR CENSUS
    Update current specs/guides/LSP/AST comments.
    Final production search for iterate/iteratorValue/iterable_class.

15. STABILIZATION / CERTIFICATION
    Focused semantic/runtime/incremental/performance gates,
    then workflow records/checkpoint/handoff.
```

Destructive retirement is intentionally late. The canonical vertical should become executable before the legacy oracle is removed.

---

# 29. Complete requirement ledger — final pass

This section is the final cumulative ledger. Requirements defined in earlier sections remain normative; the table below adds the pass-3 closure requirements and clarifies changed ownership.

## 29.1 `for` semantic/lowering requirements

The existing `P5-FOR-01..22` requirements remain in force. Pass 3 adds:

| ID | Requirement |
|---|---|
| P5-FOR-23 | Store canonical per-lane iteration resolution in `CallableAnalysis`, keyed by lane source expression identity. |
| P5-FOR-24 | Fingerprint and dependency-record lane resolutions as first-class semantic products. |
| P5-FOR-25 | Project lane semantics through `ModuleLoweringSemantics` into existing exact/requirement trait invocation products. |
| P5-FOR-26 | Compiler lowering consumes those invocation products and performs no conformance/projection discovery. |

`P5-FOR-04/21` are now closed to the two-phase order: all sources left-to-right, then all iterator acquisitions left-to-right.

## 29.2 Collection / iterator / Map / Range / lazy surface

All requirements in sections 11–16 remain incorporated, with pass-3 ownership clarification:

```text
P5-COL-*
P5-ITER-*
P5-MAP-*
P5-RANGE-*
P5-LAZY-*
P5-SURFACE-*
```

The final default-surface rule is role-based: `Iterator` owns lazy adapters; `Iterable` owns non-conflicting traversal consumers/terminals plus source conveniences such as `iter`/`indexed`. `Iterable` does not carry selector-conflicting eager `map`/`filter` transforms. Thin inherent collection forwarders preserve those accepted direct eager transforms.

## 29.3 Generic/evidence/runtime/performance/incremental

All `P5-GEN-*`, `P5-PERF-*`, and existing `P5-RT-*` remain in force. Pass 3 adds:

| ID | Requirement |
|---|---|
| P5-RT-13 | Retire positional-expansion VM cursor probing before deleting cursor selectors. |
| P5-RT-14 | Do not replace that probe with VM trait proof search. |
| P5-INC-07 | New lane semantic product participates in callable product fingerprinting. |
| P5-INC-08 | Lane proof/normalization uses existing semantic dependency recording. |
| P5-INC-09 | Lowering products invalidate when iterator/next selections change. |

## 29.4 Retirement / naming / docs

The final retirement set includes `P5-RETIRE-01..14`, `P5-NAME-01..06`, and `P5-DOC-01..06`.

Newly confirmed final-census targets include:

```text
VM::has_iteration_protocol
PackTryExpandTuplePositionals / ListTryExpandTuplePositionals generic fallback
CoreClasses.iterable_class
bootstrap sealed-class registration
universe primitives/invariants entries
LSP hover text
AST/compiler comments describing cursor lowering
current iteration/control-flow/collection guides/specs
```

## 29.5 Non-negotiable architecture invariants

1. Semantic analysis is the sole owner of conformance proof and associated-type meaning.
2. Lowering consumes semantic evidence; it does not reproduce it.
3. VM executes preselected behavior and never proves traits.
4. Iterator implementation burden remains `Item + next` for ordinary user iterators.
5. Sticky exhaustion is universal.
6. `Iterable` reusable-source semantics and one-shot `Iterator` semantics remain distinct.
7. No trait-specialization/preference rule is invented to preserve eager-vs-lazy selector overlap.
8. Cursor retirement is complete across loops, library code, VM expansion probes, and current documentation.

---

# 30. Comprehensive coverage matrix — final pass

The matrix below is requirements-driven. It is not a test execution plan; the eventual Luna plan should select the minimum discriminating test at each patch and reserve broad stabilization for named gates.

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

# 31. P5 / P6 boundary

P5 owns everything required to make canonical iteration the live production model:

```text
for proof/projection/lowering
stateful built-in iterators + Iterable/Iterator conformances
rich Iterator default surface needed to replace superclass-provided generic behavior
thin direct eager core-collection forwarding conveniences
Map labeled-entry migration
Range iterator migration within already-ratified forward behavior
String byte/code-point Iterable integration
lazy adapter migration
boundedness catalog integration needed by the changed surface
cursor retirement across compiler/library/VM expansion dependencies
legacy Iterable/Iterator runtime class retirement
public-name/docs/tooling cutover
iteration-specific incremental/performance correctness
```

P5 does **not** own:

```text
general trait-solver rewrite
general dispatch/preference/specialization feature
general generic-inference redesign
general semantic DB architecture rewrite
general VM dispatch optimization or Value/DispatchKey project
general GC redesign
new collection-builder/FromIterator-like abstraction
new ordering/sorting semantics
new double-ended iterator capability
new Range descending/step semantics or Progression design
general compiler optimization/fusion certification
```

The existing boundedness subsystem is already live compiler infrastructure, so P5 should update it narrowly for the canonical iterator surface rather than move or redesign its ownership. Its conservative rule remains: reject only proven unbounded full exhaustion; Unknown stays legal.

Argument expansion is a cross-project integration dependency. P5 owns removing iteration's cursor dependency, but it must reuse the expansion project's compiler/bytecode architecture rather than redesign packs. If dynamic expansion cannot be expressed without a new runtime trait-proof facility, that is a RED consultation point, not scope creep permission.

---

# 32. STOP / CONSULT conditions for implementation

In addition to the repository-wide escalation protocol, implementation must stop and consult if any of the following becomes true:

1. Local HEAD/worktree materially differs from the analyzed P4-complete baseline in the P5 ownership area.
2. The canonical `Iterator`/`Iterable` law on live HEAD differs materially from P4's completed law.
3. Correct `for` migration appears to require VM trait proof, runtime associated-type normalization, or a second conformance registry.
4. `CallableAnalysis` / `ModuleLoweringSemantics` cannot carry iteration lane selections without changing semantic authority between layers.
5. Generic/derived trait member lookup cannot expose the non-conflicting `Iterable.iter` convenience without a broad dispatch redesign; do not add ad-hoc Iterator-vs-Iterable precedence.
6. A high-value default requires unsupported generic/projection syntax whose implementation would exceed P5; classify/defer that method rather than weakening the core law.
7. Stateful adapters would require conformance-introduced storage or a new native heap arm rather than ordinary classes/fields.
8. Implementing `ZipLengthMismatchError` requires a nonlocal error ABI redesign rather than an ordinary language error declaration.
9. Public-name cutover requires changing module/name-resolution architecture rather than ordinary exposure/declaration cleanup.
10. Removing `iterable_class` reveals a nonlocal runtime layout/identity invariant not represented by current class construction/bootstrap metadata.
11. GC correctness would require new root ownership instead of ordinary traced instance fields.
12. The P4 `Option.unwrapOr` baseline failure reproduces on the canonical `next -> Option<Item>` path and diagnosis points to a broad generic-call-entry/runtime invariant outside bounded P5 repair.
13. Passing tests appears to require weakening exact enum-case identity, associated equality, nominal conformance, or the P4 derived rule.
14. Range migration requires choosing descending, negative-step, step-size, or Progression semantics not already ratified.
15. A newer authoritative contract contradicts Map's labeled tuple Item.
16. Positional expansion cannot retire `VM::has_iteration_protocol` without adding runtime conformance proof/search or changing dynamic `*` semantics materially.
17. A temporary cursor compatibility bridge would remain the real production protocol at P5 completion.
18. Patch scope expands into a materially unrelated compiler/runtime/type-system subsystem rather than a smallest owner-correct prerequisite.

---

# 33. Expected P5 completion state

P5 is complete only when one coherent production path explains ordinary `for`:

```text
source expression
    ↓ exactly once
semantic proof: Source |= Iterable
    ↓
normalize Iterable::Item / Iterable::Iterator
    ↓
preselect Iterable.iterator + Iterator.next invocation evidence
    ↓
compiler receives executable lane lowering spec
    ↓
iterator acquired once
    ↓
stateful next -> Option<Item> per attempted step
    ├─ Some(item) -> pattern/ordinal bind -> body
    └─ None       -> exhaustion classification
```

and the broader system satisfies all of these statements:

```text
all supported for forms use the same semantic authority
parallel preheaders evaluate all sources before any iterator acquisition
strict zip uses ZipLengthMismatchError and never truncates
custom iterators normally implement only Item + next
Iterator owns rich lazy-adapter defaults
Iterable owns reusable traversal plus non-conflicting terminal/convenience defaults
.iter is an ergonomic alias over iterator rather than SourceIterator wrapping
core collection direct eager operations remain ergonomic through thin inherent forwarding
lazy pipelines are real one-shot stateful Iterator objects
sticky exhaustion holds for every iterator and adapter
built-in collections/views expose real Iterable conformances
all concrete traversal objects expose real Iterator conformances
Map direct/items entries use canonical labeled (key,value) tuples
Range uses a real stateful RangeIterator without invented step/descending semantics
String itself remains non-Iterable while bytes/codePoints are canonical Iterable sources
generic/conditional/cross-module/derived evidence works
exact enum-case identity remains intact
semantic proof/projection work is not repeated per item
VM performs no trait proof or projection normalization
positional expansion no longer detects iteration by cursor-selector lookup
no production cursor tokens/selectors remain
legacy runtime Iterable/Iterator classes and iterable_class bootstrap identity are gone
public Iterator / Iterable names denote traits
current specs/guides/LSP/comments describe the canonical model
incremental fingerprints/dependencies track lane evidence narrowly
```

The implementation plan may stage this over many patches, but it may not declare P5 complete while a hidden cursor protocol, runtime Iterable superclass, or duplicated semantic iteration authority remains.

---

# Pass-3 closure record

The pass-2 closure agenda is resolved as follows.

| Pass-2 agenda item | Pass-3 disposition |
|---|---|
| Local checkout / baseline | **PRE-IMPLEMENTATION GATE.** Remote HEAD reverified; local execution unavailable, so baseline reproduction remains mandatory before edits. |
| Exhaust production cursor search | **CLOSED TO PLANNING SCOPE.** Added compiler loops, runtime superclass/library, VM positional-expansion probe, current docs/LSP/comments, repository subclasses to final census. Final implementation still runs mechanical grep. |
| Exact semantic product seam for `Statement::For` | **CLOSED.** Extend `CallableAnalysis` with per-source-expression lane resolution. |
| Exact selected trait invocation seam | **CLOSED.** Project through `ModuleLoweringSemantics` and existing `TraitInvocationSpec` / `TraitRequirementInvocationSpec`. |
| Trait-default discoverability | **CLOSED ARCHITECTURALLY.** Rich overlapping algorithms live only on Iterator; Iterable remains small. Derived evidence must support non-conflicting Iterable conveniences without invented precedence. |
| Adapter generic expressibility | **CLOSED FOR TARGET SET.** Generic trait methods, ordinary where constraints, associated projections/equalities provide the required semantic vocabulary; implementation may defer an individual method only on demonstrated compiler syntax/typing blocker. |
| Map Entry consumers | **CLOSED.** Canonical item is labeled tuple; current Entry construction is isolated to MapEntriesView, and labeled tuple access preserves `.key`/`.value` conversion ergonomics. Final usage grep remains a deletion gate. |
| String view Item types | **CLOSED.** Both byte values and decoded code points are integer-valued on current surface; String itself remains non-Iterable. |
| GC tracing | **CLOSED.** Ordinary instance slots are traced exhaustively; stateful iterator/adaptor fields need no new heap representation. |
| Incremental products/fingerprints | **CLOSED.** Lane product belongs in CallableAnalysis fingerprint/dependency flow and projected lowering semantics. |
| Public-name transition | **CLOSED TO SEQUENCE.** Consumer-first migration; runtime classes/core identity removed late; final public names are traits; avoid unnecessary trait declaration identity churn. |
| Boundedness ownership | **CLOSED.** Existing compiler subsystem remains owner; P5 updates its selector/mode propagation narrowly for canonical iterators and new adapters. |
| Performance-critical defaults | **CLOSED TO REQUIREMENTS.** Source-authored defaults/stateful classes, no proof per item, no native-combinator explosion; fusion is follow-on optimization. |
| Current docs/spec reconciliation | **CLOSED TO CENSUS.** Canonical `docs/specs/iterator.md` wins; current cursor guides/specs/LSP/comments update; historical docs remain historical/superseded. |
| Requirement traceability | **CLOSED.** Pass-3 requirements add discriminating coverage through P5-CV-115 and define the implementation sequence needed for the next planning turn. |

## Final readiness assessment

This requirements analysis is ready to drive the P5 patch-grade implementation plan. The plan should not reopen the central iteration architecture. It should focus on patch decomposition, exact file/symbol edits, narrow tests per patch, and destructive retirement gates.

The only mandatory fact to establish before implementation begins is executable local baseline state. That is a workflow preflight requirement, not an unresolved semantic design question.
