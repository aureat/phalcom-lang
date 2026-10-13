---
id: LANG007.C5.progression-design
category: LANG
program: LANG007
checkpoint: LANG007.C5
kind: technical-specification
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-10-13
---

# Immutable Progression, iteration, and interval interaction

## 1. Scope and distinction

The user expanded C5 on 2026-10-13 to include an immutable `Progression` that
participates in iteration and interacts with intervals and ranges. This document
proposes that contract alongside the [interval design](interval-relational-matching-spec.md).
It is not a claim of an existing type or accepted source behavior.

| Value | Meaning | Order / traversal |
| --- | --- | --- |
| `Interval<T,D>` | All domain values in one connected region | Membership and algebra; no automatic enumeration |
| Existing `Range` | Numeric bound membership with separate unit-step integer iteration | Existing iteration remains unchanged |
| `Progression<T,D>` | An ordered arithmetic sequence with an explicit start, signed stride, and extent | A fresh independent iterator for each traversal |

An integer progression 1, 4, 7 is not the interval `[1,7]`. The values 2, 3, 5,
and 6 are absent. Its interval hull is a lossy observation and must never decide
progression membership or event conflicts.

Preserve the accepted [iterator contract](../../../specs/iterator.md),
[data immutability](../../../specs/objects/data-objects.md), and canonical
associated types in [traits](../../../specs/objects/traits.md) and
[associated types](../../../specs/associated-types.md). Implementation must use
the live `universe.collections.iterator_laws` interfaces, not an invented
parallel Iterable protocol.

## 2. Representation and arithmetic meaning

Proposed semantic shapes:

```text
ProgressionView<T,S> = Empty | Stepped(start: T, step: S, count: Option<Int>)
Progression<T,D> private(view: ProgressionView<T,D::Step>, domain: D)
ProgressionPieces<T,D> = Zero | One(Progression<T,D>)
                            | Two(Progression<T,D>, Progression<T,D>)
```

`D` uses canonical nominal domain identity, as in Interval. An internal
`ProgressionDomain<T>` capability supplies associated `Step`, validation, exact
`advance(start, step, index)`, and exact calculation of grid indices against
interval cuts. These are source-library methods and ordinary conformance
witnesses. `ProgressionView<T,S>` has an explicit Step parameter so its public
data declaration does not depend on unresolved projection spelling.

For nonempty P:

```text
value(k) = advance(P.start, P.step, k)
admitted indices = 0 <= k < count, or every k >= 0 if count is None
```

The value is an immutable recipe. It contains no iterator position, mutable
collection backing, implicit wall clock, callback, or observable allocation
identity. Iterator objects are separate mutable cursors.

Canonical finite empty extent is `ProgressionView::Empty`; `Some(0)` normalizes
to Empty. Negative count is an error. Zero stride is always an error, including
for zero count; validation precedes empty normalization. A finite singleton
retains the valid supplied step in its view. Equality is normalized recipe
equality, including domain, start, signed step, and count; it is not general
sequence/set extensional equality. Empty values within the same domain have one
normal form. Reversed progressions deliberately have a different traversal.

### Supported step types

| Facade | Domain / element | Step | Required law |
| --- | --- | --- | --- |
| `IntegerProgressions` | IntegerDomain / Int | Nonzero Int | start + k * step, exact |
| `ExactProgressions` | RationalDomain / Rational | Nonzero Rational | Exact rational arithmetic, no Float conversion |
| `DateProgressions` | DateDomain / Date | Period with months = 0 and days != 0 | Calendar day advancement |
| `TimestampProgressions` | TimestampDomain / Timestamp | Duration with nanoseconds != 0 | Exact POSIX nanosecond advancement |

Monthly Periods, recurrence rules, time zones, local wall-clock repetition, and
Float drift are outside the initial contract. Do not pretend adding one month
is an affine, constant-stride progression. Date progression counts calendar days;
it does not equate a day with an elapsed 24-hour duration.

The `(T,D)` interval domain does not change with traversal direction or stride.
The step is progression data, not a type-level extent or an overload key.

## 3. Proposed API

Factories on each facade:

```text
finite(start: T, step: D::Step, count: Int) -> Progression<T,D>
unbounded(start: T, step: D::Step) -> Progression<T,D>
```

No default-argument family is implied. Checked private construction is the only
way to create valid recipes. Public view descriptors are validated again if
accepted by a named factory.

Instance operations:

| Operation | Contract |
| --- | --- |
| `view` | Immutable canonical public descriptor |
| `isEmpty` | True only for canonical Empty |
| `count` | Some(nonnegative count) for finite, None for unbounded |
| `at(index: Int)` | Exact zero-based value; negative indexing only on finite sequences; out-of-range raises ProgressionError |
| `contains(value: T)` | Exact grid membership and admitted index test; hull membership alone is insufficient |
| `take(count: Int)` | Immutable prefix recipe; negative errors, zero empties, finite counts saturate |
| `drop(count: Int)` | Immutable suffix recipe; negative errors, exhausted finite recipe empties |
| `reversed` | Empty or finite sequence only; finite result starts at last value and negates stride; unbounded raises ProgressionError |
| `clip(interval: Interval<T,D>)` | Sequence values lying in the interval, preserving order, stride, and phase |
| `subtract(interval: Interval<T,D>)` | At most two recipes, preserving the receiver's traversal order |
| `relate(interval: Interval<T,D>)` | Closed typed relationship described below |
| `hull` | Empty or smallest member-containing Interval; unbounded side follows traversal direction; explicitly lossy |
| `iterator` | Fresh `ProgressionIterator<T,D>` conforming to canonical Iterator |

`reversed.reversed` restores the original normalized finite recipe. `drop` and
`take` adjust start/count using exact arithmetic and do not enumerate values.
Bounds/counts never use Float arithmetic. Errors are ordinary language errors,
not failed pattern recognition.

No eager `Progression.toList` is required. Materialization remains explicit via
the iterator API; an unbounded caller should first bound traversal with `take`.
No general `size` that attempts to count an infinite sequence is added.

## 4. Iteration and lifetime

Conformance shape, using existing canonical declarations:

```text
impl CanonicalIterable for Progression<T,D>:
    type Item = T
    type Iterator = ProgressionIterator<T,D>
    iterator -> new iterator retaining self

impl CanonicalIterator for ProgressionIterator<T,D>:
    type Item = T
    next -> Option<T>
```

The iterator keeps an immutable progression reference and a mutable Int index /
done flag. `next` checks finite exhaustion before calculating a value, returns
`Some(value(k))`, then advances the index. Once exhausted it continues returning
None. Empty iterators perform no domain advancement. If advancement cannot be
represented, raise ProgressionError; never wrap or silently terminate as None.

Two iterators over the same value start independently at index 0. Nested `for`
loops, early `break`, and abandoned iterators do not alter the progression or
each other. Returning the iterator after the original local goes out of scope
must retain the recipe and domain through ordinary VM tracing. No weak backing,
native pointer, shared cursor, or custom rooting policy is allowed.

The progression algorithm has no user callback or external mutable input. It
introduces no scheduling or cancellation semantics beyond existing iteration.
Large or unbounded traversal remains ordinary potentially long-running code;
`clip`, `take`, `drop`, and `at` must not scan to reach their result.

## 5. Clipping uses grid indices, not interval endpoints

For each interval boundary, solve the index inequality on `start + step*k`:

- positive step: inclusive lower uses ceiling; exclusive lower uses floor + 1;
  inclusive upper uses floor; exclusive upper uses ceiling − 1;
- negative step: reverse inequality direction, then apply the corresponding
  inclusive/exclusive rounding rule;
- intersect the resulting index interval with `[0,count)` or `[0,+∞)`.

The ratios are exact integer/rational coordinate ratios. Date coordinates use
`daysSince`; Timestamp coordinates use `unixNanoseconds`. Floor/ceiling must
handle negative values mathematically, not truncation toward zero. Never
subtract a floating epsilon from a boundary.

The first retained index and finite retained count determine one clipped recipe;
if none exist, return Empty. Finite interval plus nonzero stride always yields a
finite count. An unbounded interval can leave an unbounded result. Descending
progressions retain descending order.

Examples:

| Operation | Exact result |
| --- | --- |
| 1,4,7,10 clipped to `[2,10)` | 4,7 |
| 10,7,4,1 clipped to `[2,10)` | 7,4 |
| 1,4,7 clipped to singleton `[2,2]` | Empty, despite hull intersection |
| 1,4,7 clipped to singleton `[4,4]` | 4 |
| Unbounded 1,4,7,… clipped to `[2,10)` | Finite 4,7 |
| −5,−2,1,4 clipped to `[−3,2)` | −2,1 |
| Tuesdays starting 2026-10-13, step seven days, clipped to `[Oct 18, Oct 23)` | Oct 20; never Oct 18 |

`subtract(I)` splits the admitted index sequence around its clipped contiguous
index block. The returned Two is in traversal order. This differs intentionally
from `IntervalPieces::Two`, whose order is ascending domain order.

## 6. Typed progression/interval relationship

Proposed closed sum:

```text
ProgressionIntervalRelation<T,D>:
    EmptyProgression
    EmptyInterval(original: Progression<T,D>)
    Disjoint(original: Progression<T,D>)
    Contained(inside: Progression<T,D>)
    Clipped(inside: Progression<T,D>, outside: ProgressionPieces<T,D>)
```

Precedence: empty progression first (including both empty); then empty interval;
then no retained grid values; then every admitted grid value retained; otherwise
Clipped. Payload constructors/patterns use positional arguments.

This relation concerns scheduled points, not durations. A recurring inspection
starting at 10:00 and lasting two hours needs an Interval per occurrence for
duration overlap; testing just the inspection-start progression cannot prove
that its entire service duration is clear. The hotel example distinguishes these.

The ordinary match engine proves coverage of these five cases. It does not infer
congruence or arithmetic inequalities about original inputs from freely
constructible relationship values. Domain mismatch is an error at the call
boundary, never Disjoint.

## 7. Interaction with Range and Interval

`Interval.progression(step:)` is available only for the three discrete supported
domains. For positive step it needs a finite lower bound and anchors at the first
member; for negative step it needs a finite upper bound and anchors at the last
member. The opposite side can be unbounded. Empty yields Empty after validating
step. Missing anchor raises ProgressionError. There is no chosen arbitrary
origin for an all-domain interval.

For Rational intervals, use an explicit grid:
`ExactProgressions.unbounded(start:, step:).clip(interval)`. An excluded dense
lower bound has no next Rational value; it must not be “rounded up” to an
invented first member.

`Range.progression(step: Int)` explicitly converts through `toIntegerInterval`
and uses the discrete anchoring rule. Initial version requires actual Int bounds
where present. Existing `Range.iterator` remains ascending unit-step and retains
its existing errors; negative strides are an additive Progression API.

Examples, using the proposed API:

```phalcom
let region = Range.new(1, 10, false)
let walk = region.progression(step: 3)       // 1,4,7
let reverseWalk = region.progression(step: -3) // 9,6,3
let alignedReverse = walk.reversed          // 7,4,1
```

Reversing a recipe preserves its phase; constructing a negative-step recipe
from interval bounds anchors at the last member and can select a different grid.
Do not assert that those operations are equivalent.

Progression `hull` is a named conversion to Interval. Binary progression union,
arbitrary set difference, and progression/progression intersection are outside
initial implementation. The latter needs arithmetic-grid intersection (e.g.
congruence/CRT for integers and commensurate Rational strides), not interval
intersection. This is a future extension with an explicit algorithm/ordering
contract; do not return false positives from overlapping hulls in the meantime.

## 8. Cases, failures, and acceptance invariants

| ID | Contract | Owner / evidence |
| --- | --- | --- |
| PR-01 | Recipe and view are immutable; validation rejects zero step, negative count, and invalid domain/step combinations | Factory/runtime tests |
| PR-02 | Values are exact affine advancement for admitted indices; direction is explicit | Domain arithmetic and negative-coordinate oracle |
| PR-03 | Every iterator is independent, retains its recipe, and stays exhausted | Iterator/nested-loop/GC tests |
| PR-04 | Clip preserves grid phase, sign, order, and exact bound inclusion without scanning | Differential grid oracle |
| PR-05 | Take/drop/reverse/count behavior handles empty, singleton, finite, and unbounded recipes | Boundary and reversal laws |
| PR-06 | Relate partitions points into five cases; hull cannot decide membership | Sparse-gap counterexample and match tests |
| PR-07 | Range adapters retain existing Range behavior and expose stride explicitly | Range/progression regression fixtures |
| PR-08 | `D::Step`, Item, iterator types, and domain reification use canonical semantic identities | Imported generic/incremental tests |

Rejected cases include Period.months != 0, zero Duration, Date + Duration as a
step, mixed domains, invalid Dynamic values, and reversing an unbounded recipe.
No errors become None except ordinary iterator exhaustion and the documented
unbounded `count` observation.

This choice does not preclude geometric progressions, recurrence schedules,
monthly calendar sequences, Float sampling, CRT intersections, or immutable
sequence libraries. They need separate names or contracts when they violate
the affine laws above. C5 does not redesign arbitrary Iterable or turn intervals
into containers by default.
