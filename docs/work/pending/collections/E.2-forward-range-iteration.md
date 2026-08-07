# Spec E.2 — Forward Integer Range Iteration

Status: implementation specification. Requires C.2's Range AST/runtime representation and E.1's explicit Iterator wrapper. This phase deliberately implements only the Range iteration subset already required by the ratified collection/boundedness semantics.

## 1. Mission

Activate these Range values as ordinary forward integer iterables:

```phalcom
0..10
0..=10
0..
```

with implicit step:

```text
+1
```

while preserving Range as a bounds structure rather than turning it into a Progression object.

Supported semantics:

```text
lower bound present
integer lower
optional integer upper
forward/non-descending bounds
half-open or upper-inclusive
```

Unsupported/deferred cases must fail cleanly or remain unavailable rather than inheriting obsolete pre-C arithmetic behavior.

Expected primitive-floor delta: **0**.

## 2. Preconditions from C.2

C.2 changes native Range storage to conceptually:

```text
lower: optional Value
upper: optional Value
upperInclusive: Bool
```

with compact private-sentinel representation allowed internally.

Core `.ph` can inspect:

```text
lower_          -> Option<Value>
upper_          -> Option<Value>
upperInclusive_ -> Bool
```

C.2 also removes the obsolete language convention:

```text
..  inclusive
... exclusive
```

E.2 must not revive it.

## 3. Why E must activate `0..`

The boundedness specification uses:

```phalcom
0..
```

as the canonical provably unbounded source.

That source must not be merely a syntactic bound descriptor that cannot participate in iteration; otherwise:

```phalcom
(0..).iter.take(10).toList
```

could not realize the ratified "explicitly introduce a finite bound" model.

At the same time, the language intentionally leaves several broader Range/Progression questions open. E.2 therefore supplies the smallest coherent forward integer iteration subset instead of pretending the entire Range runtime design is settled.

## 4. Keep Range and Progression distinct

Do not add `step` to `RangeObject`.

Do not reinterpret a Range as:

```text
(start, end, step)
```

Do not implement:

```phalcom
range.by(step)
```

in this phase.

The eventual `Progression` remains the stepped-traversal abstraction.

E.2's implicit `+1` is the default traversal behavior of the supported integer Range subset, not a stored Range field.

## 5. Range implements the existing cursor protocol

Range should override the generic index/size-based `Iterable#iterate` behavior.

Use:

```text
Range#iterate(cursor)
Range#iteratorValue(cursor)
```

A live Range cursor is the current yielded numeric value itself.

Therefore:

```text
iteratorValue(cursor) = cursor
```

for every live supported Range cursor.

This avoids allocating a cursor wrapper for each integer and avoids requiring a `size` on an unbounded Range.

## 6. Initial cursor

For:

```text
range.iterate(None)
```

the candidate is the lower bound.

If the lower bound is absent, this E subset cannot choose a starting value.

Lowerless forms:

```phalcom
..b
..=b
..
```

remain valid **Range values** but are not iterable through E.2's forward integer subset.

When iteration is attempted, follow the repository's existing unsupported-operation/argument-error convention and produce a catchable language error. Do not panic and do not silently choose `0` as an implicit start.

Do not add a new public Range-specific error family solely for this temporary semantic boundary unless current HEAD's error taxonomy already requires one.

## 7. Endpoint validation

For the supported iteration subset:

```text
lower -> integer
upper, when present -> integer
```

Use the language's current Int representation.

Until the numeric tower lands, reuse Spec C's compatibility seam:

```text
finite integral Number
```

Do not create an E-specific integer-coercion rule.

No String/Symbol/custom successor protocol is introduced.

An invalid endpoint fails when iteration is initiated, before yielding a fabricated first element.

## 8. Forward-only ordering rule

E.2 supports non-descending forward bounds.

For two-sided ranges:

```text
lower <= upper
```

is supported.

When:

```text
lower > upper
```

do **not** choose between:

```text
empty ascending range
automatic descending traversal
negative implicit step
```

because descending/reversed Range semantics are explicitly deferred.

Instead, iteration of such a Range must fail through the same catchable unsupported-range-iteration path used for other currently unsupported Range traversal forms.

This preserves the future design space.

## 9. Half-open finite iteration

For:

```phalcom
a..b
```

with supported integer bounds and `a <= b`:

```text
first candidate = a
subsequent       = previous + 1
live iff         candidate < b
```

Examples:

```text
0..0  -> empty
0..1  -> 0
1..4  -> 1,2,3
```

No value equal to `b` is yielded.

## 10. Inclusive finite iteration

For:

```phalcom
a..=b
```

with supported integer bounds and `a <= b`:

```text
first candidate = a
subsequent       = previous + 1
live iff         candidate <= b
```

Examples:

```text
0..=0 -> 0
0..=1 -> 0,1
1..=4 -> 1,2,3,4
```

## 11. Lower-bounded unbounded iteration

For:

```phalcom
a..
```

with a supported integer lower bound:

```text
first candidate = a
subsequent       = previous + 1
exhaustion       = never from the Range itself
```

No hidden maximum count is permitted.

No artificial `Number` threshold is used as semantic exhaustion. If the current pre-numeric-tower representation reaches a representation/arithmetic boundary, follow the numeric subsystem's ordinary behavior; do not disguise it as Range exhaustion.

After the Int tower lands, the Range iteration implementation should naturally use the new integer arithmetic without changing Range semantics.

## 12. Conceptual algorithm

`Range#iterate(previous)`:

```text
lower = requireSupportedLowerInteger(self.lower_)
upper = optionalSupportedUpperInteger(self.upper_)

if upper exists and lower > upper:
    raise unsupported-forward-range-iteration

if previous == None:
    candidate = lower
else:
    candidate = previous + 1

if upper absent:
    return candidate

if upperInclusive_:
    return candidate <= upper ? candidate : None

return candidate < upper ? candidate : None
```

Validation may be factored so the lower/upper checks are not needlessly duplicated, but do not introduce mutable iterator state just to cache them.

`Range#iteratorValue(cursor)`:

```text
return cursor
```

The implementation may use Bool methods/control flow consistent with current core.ph style.

## 13. Interaction with `.iter`

E.1's generic:

```phalcom
Iterable#iter => SourceIterator.new(self)
```

is sufficient.

`SourceIterator` delegates:

```text
iterate
iteratorValue
```

to Range.

Do not allocate a special native `RangeIterator`.

Pipeline example:

```phalcom
const firstTen =
    (0..).iter
        .map { x => x * 2 }
        .take(10)
        .toList
```

Expected values:

```text
0,2,4,6,8,10,12,14,16,18
```

The unbounded Range itself remains O(1) storage.

## 14. Direct Range operations remain D-eager

The explicit iterator boundary remains meaningful for Range.

Given D.1's generic eager `Iterable#map`:

```phalcom
(0..10).map { x => x * 2 }
```

is eager and returns a concrete result.

```phalcom
(0..10).iter.map { x => x * 2 }
```

is lazy.

For:

```phalcom
(0..).map { x => x * 2 }
```

successful completion would require exhausting the unbounded Range. E.3 therefore rejects the call as a provably unbounded eager exhaustor.

Do not special-case Range so that direct `map` becomes lazy; that would destroy the receiver-driven rule established by D/E.1.

## 15. `for` over an unbounded Range is legal

A `for` loop is control flow, not automatically a successful full-source materializer.

This is valid:

```phalcom
for (x in 0..) {
    if (x == 10) {
        break
    }
}
```

E.3 MUST NOT reject `for` solely because its source is provably unbounded.

A loop without a terminating `break` may run forever. That is normal program behavior, not a compile-time eager-exhaustor error.

## 16. No `size` requirement

Do not add a fake `size` for:

```phalcom
0..
```

and do not encode infinity as:

```text
-1
Infinity
None
```

under a `size` getter.

Range iteration must work through its override of `iterate`.

If an existing Range `size` method survived C for finite compatibility, audit it so:

- it is never required by generic iteration;
- it does not claim to size a lower-only unbounded Range;
- it does not preserve obsolete `...` semantics.

A final Range size/count API can be ratified separately.

## 17. Boundedness is not a runtime Range flag

Do not add:

```text
range.isBounded
range.boundedness
range.isInfinite
```

as public methods in E.

E.3 classification is compiler metadata derived from syntax/pipeline facts.

Runtime code simply follows Range traversal semantics.

This avoids making the static analysis lattice part of user-visible object identity.

## 18. Unsupported forms remain real Range values

These all still construct successfully after C:

```phalcom
..5
..=5
..
5..2
```

E.2 does not turn their *construction* into an error merely because this iteration subset cannot traverse them.

The error occurs only when an unsupported value is asked to participate in the current forward integer traversal protocol.

This distinction preserves Range's non-iteration uses such as slicing/bound descriptions.

## 19. Progression gate

Do not ship a skeletal `Progression` from E.2.

The following still require explicit design decisions:

- public Progression class/runtime representation;
- `by(step)` validation;
- zero step;
- negative step;
- sign mismatch;
- descending bounds;
- endpoint reachability under arbitrary step;
- equality/hash;
- slicing relationship;
- boundedness propagation through Progression.

E.3 may conservatively classify any future/unrecognized progression-like expression as Unknown until that unit lands.

## 20. Tests

### 20.1 Finite half-open

- `range_iter_half_open_empty.ph`
- `range_iter_half_open_single.ph`
- `range_iter_half_open_many.ph`

Pin:

```text
0..0
0..1
1..4
```

### 20.2 Finite inclusive

- `range_iter_inclusive_single.ph`
- `range_iter_inclusive_many.ph`

Pin:

```text
0..=0
1..=4
```

### 20.3 Pipeline from unbounded Range

- `range_unbounded_take_prefix.ph`
- `range_unbounded_map_take_prefix.ph`
- `range_unbounded_filter_take_prefix.ph`

All must terminate only because `take` introduces an explicit finite limit.

### 20.4 `for` + break

- `range_unbounded_for_break.ph`

Prove an unbounded Range can be used with explicit user short-circuit control flow.

### 20.5 Unsupported traversal

Negative runtime fixtures:

- `range_iter_upper_only_unsupported.ph`
- `range_iter_fully_lowerless_unsupported.ph`
- `range_iter_reversed_unsupported.ph`
- `range_iter_non_integer_lower_unsupported.ph`
- `range_iter_non_integer_upper_unsupported.ph`

Assert a catchable language failure, not a Rust panic.

Do not over-pin a new public error subclass if the error-taxonomy project has not ratified one.

## 21. Completion checklist

E.2 is complete when:

- supported finite Range forms iterate correctly;
- `a..` is a genuine non-exhausting source;
- lowerless Range construction remains valid but traversal is unsupported;
- reversed traversal is not silently assigned semantics;
- Range cursor is the current element;
- generic `.iter` wrapping works;
- direct Range operations remain D-eager;
- no `size` is required for unbounded iteration;
- no hidden cap exists;
- no Progression or `by(step)` was smuggled in;
- floor census remains unchanged.
