# Phalcom Collections — Spec E: Explicit Lazy Iteration and Boundedness

Status: implementation-spec bundle. This continues Specs A–D and implements the ratified boundary between eager concrete collection operations and explicit lazy iterator pipelines, together with conservative static boundedness analysis for eager exhaustors.

The repository baseline re-checked while producing this plan is `aureat/phalcom-lang` main at commit `5c73279157891ca8e2fc045db5e7dff683c0be5b`. Every implementation session MUST re-inspect actual HEAD before editing. Paths below are architectural anchors, not frozen line numbers.

## Dependencies

Implement after D.1 in the intended sequence. D.2/D.3 may land before or alongside E where their files do not conflict, but E assumes D's public eager/lazy split is the winning collection semantics.

- A supplies Unit and Tuple/product construction used by iterator cursor state.
- B supplies ordered Map/Entry/view behavior; iterator pipelines may wrap those views through the ordinary Iterable protocol.
- C.2 supplies the new Range bound representation and raw `lower_`, `upper_`, `upperInclusive_` observations.
- C.3 supplies slicing but is otherwise independent.
- D.1 removes the old direct-lazy `Iterable#map/where/skip/take` surface and establishes eager direct transformations.
- D.3 supplies materializers such as `toList`, `toSet`, and `toMap`; on an iterator receiver these become eager terminal/exhaustor operations.
- F later consumes E.3's boundedness API when implementing positional expansion (`*`), which is also an eager exhaustor.

## Repository diagnosis

Current HEAD already has a useful low-level iteration substrate:

```text
iterate(cursor)       -> next live cursor or None
iteratorValue(cursor) -> encountered value
```

`for` is compiled against that protocol, and exhaustion is represented by the shared `None` singleton rather than a `Some(cursor)` wrapper. Keep that substrate. E does not replace it with a Rust iterator object or a new VM opcode.

Current HEAD also contains the older U-SEQ lazy view architecture (`MapView`, `WhereView`, `SkipView`, `TakeView`) directly under `Iterable`. D.1 intentionally removes that public direct-collection behavior. E reuses the good implementation idea—stateless cursor-transforming view objects—but moves it behind explicit `.iter`.

No repository boundedness-analysis subsystem was found on the inspected baseline. E.3 therefore introduces a small compiler-only semantic lattice rather than trying to force cardinality into public runtime types.

C.2 intentionally stopped short of full Range iteration because descending, negative-step, Progression, and upper-only iteration semantics remain open. E.2 activates only the subset already required by the ratified boundedness examples:

```text
finite forward integer range with lower bound
lower-bounded unbounded forward integer range, e.g. 0..
```

It does not invent the deferred cases.

## Phase order

### E.1 — Explicit Iterator Pipeline Runtime

Introduce the explicit lazy receiver:

```phalcom
source.iter
```

and a reusable `Iterator` pipeline root over the existing cursor protocol.

On ordinary concrete `Iterable` receivers:

```text
map/filter/flatMap
    -> D eager operations
```

On an `Iterator` receiver:

```text
map/filter/flatMap
    -> lazy stage descriptors
```

Add the lazy limiter stages required by the ratified boundedness model and the pre-existing U-SEQ migration:

```text
skip(n)
take(n)
takeWhile(predicate)
```

The pipeline is a reusable traversal descriptor, not a one-shot consumed cursor object. Construction does not execute mapping/filtering callbacks. Materialization and inherited terminal operations consume the pipeline.

Recommended concrete stage classes are ordinary `.ph` classes and add zero native bindings.

Expected primitive-floor delta: **0**.

Artifact: `E.1-explicit-iterator-pipeline.md`.

### E.2 — Forward Integer Range Iteration Subset

Make C.2 Range values participate in the existing cursor protocol for the semantics already pinned:

```phalcom
0..10
0..=10
0..
```

under forward integer step `+1`.

A live Range cursor is the current Range element itself. Finite upper-bound checks honor half-open versus inclusive syntax. A lower-only Range never reports exhaustion.

Do not implement:

- upper-only iteration;
- fully unbounded iteration without a lower start point;
- reversed/descending Range traversal;
- `Range#by(step)` / Progression;
- zero/negative step behavior;
- non-integer domains.

Those remain explicitly deferred rather than being guessed.

Expected primitive-floor delta: **0** beyond C.2's Range raw observers.

Artifact: `E.2-forward-range-iteration.md`.

### E.3 — Static Boundedness and Eager-Exhaustor Diagnostics

Add a compiler-only three-state classification:

```text
Bounded
Unbounded
Unknown
```

plus enough source-mode information to distinguish a concrete receiver from an explicit lazy pipeline receiver.

The analysis is intentionally conservative:

- bounded literal collections -> Bounded;
- finite two-sided supported Range -> Bounded;
- lower-only forward Range (`a..`) -> Unbounded;
- `.iter` preserves boundedness and enters lazy mode;
- lazy `map`, `filter`, and `skip` preserve boundedness;
- lazy `take` -> Bounded;
- lazy `takeWhile` over an unbounded/unknown source -> Unknown;
- lazy `flatMap` over an unbounded outer source -> Unbounded; otherwise Unknown unless stronger proof is available;
- unrecognized expressions -> Unknown.

Reject an eager exhaustor only when its source is **provably Unbounded**. Unknown sources remain legal exactly as ratified.

E.3 must expose the analysis as a reusable compiler service so F can reject:

```phalcom
foo(*(0..))
```

without implementing a second cardinality analyzer.

Expected primitive-floor delta: **0**.

Artifact: `E.3-boundedness-and-exhaustor-diagnostics.md`.

## Cross-phase invariants

After E.1:

- `.iter` itself performs no traversal;
- direct collection `map/filter/flatMap` remain eager as established by D;
- `collection.iter.map/filter/flatMap` are lazy;
- iterator-stage creation invokes no user callback;
- pipeline traversal preserves source encounter order;
- `map` invokes its callback once for each encountered source element during each traversal;
- `filter` invokes its predicate only while searching for the next accepted value;
- `flatMap` does not recompute the callback merely to advance within the same returned inner iterable;
- `take(0)` can exhaust without touching the source;
- `take(n)` and `skip(n)` validate the count when the stage is constructed;
- traversing the same pipeline value again starts a fresh traversal and re-executes callbacks; stages do not memoize;
- `.iter` on an iterator is idempotent (`iterator.iter === iterator` if identity is observable through the ordinary object model);
- old `lazyMap`/direct `where` are not reintroduced.

After E.2:

- finite ascending integer Range iteration honors `..` versus `..=`;
- `0..` is a real non-exhausting forward source;
- a `for` loop over `0..` is legal because user control flow may `break`;
- no hidden element cap is imposed;
- unsupported lowerless/reversed/non-integer Range iteration does not accidentally execute old arithmetic semantics;
- Range remains O(1) storage.

After E.3:

- provably unbounded eager exhaustion fails at compile time;
- unknown-boundedness eager exhaustion is legal;
- short-circuit prefix operations are not rejected merely because their source is unbounded;
- adding a finite lazy `take` makes an otherwise unbounded pipeline statically bounded;
- `map/filter/skip` do not launder an unbounded pipeline into bounded;
- `takeWhile` does not pretend arbitrary predicate termination can be proven;
- the compiler stores no runtime boundedness flag and performs no hidden truncation;
- F can query the same facts before positional expansion.

## Important semantic distinction: Range versus iterator pipeline

Range is still a bounds value, not the iterator object model itself.

```phalcom
const r = 0..10
```

creates a Range.

```phalcom
const p = r.iter
```

creates an explicit lazy pipeline wrapper.

Range can participate directly in ordinary iteration (`for`) and D's generic eager operations because it implements `iterate`/`iteratorValue`. The `.iter` boundary is what selects the lazy transformation overrides:

```phalcom
r.map { ... }          // eager D transformation
r.iter.map { ... }     // lazy E transformation
```

For an unbounded Range, the former is a provably unbounded eager exhaustor and E.3 rejects it; the latter creates a legal unbounded lazy stage.

## Deliberate exclusions

Spec E does **not** implement or decide:

- Progression object model or `Range#by(step)`;
- descending/reversed Range semantics;
- zero-step/negative-step/sign-mismatch behavior;
- upper-only or fully-lowerless Range iteration domains;
- a complete public iterator protocol hierarchy beyond the minimal explicit pipeline root;
- mutation-during-iteration semantics;
- snapshot versus fail-fast semantics for collection-backed iterators/views;
- async iteration;
- iterator cloning/copying protocol;
- caching/memoization;
- parallel traversal;
- exact eager result-family matrix (D);
- Set/ImmutableSet completion;
- sorting gates left by D.3;
- expansion/argument packs (F), except the reusable boundedness hook F must call;
- advanced theorem proving over arbitrary predicates or callback results;
- public reflection APIs for boundedness metadata.

## Verification gate

Each phase should run the repository's normal build/test/clippy gate and its focused language fixtures.

At the end of E, additionally verify:

1. no new native primitive binding was introduced;
2. D's eager collection tests remain green;
3. U-SEQ tests asserting direct laziness are gone or migrated;
4. the same pipeline can be traversed twice;
5. `(0..).iter.take(10).toList` terminates with ten values;
6. `(0..).toList` is a compile error;
7. `(0..).iter.map { ... }.toList` is a compile error;
8. `(0..).iter.takeWhile { ... }.toList` compiles (Unknown boundedness);
9. an arbitrary parameter/unknown iterator `.toList` compiles;
10. no implementation adds a runtime safety cap to unknown/unbounded iteration.
