---
id: LANG007.C5.design
category: LANG
program: LANG007
checkpoint: LANG007.C5
kind: technical-specification
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-10-13
---

# Interval algebra exposed through typed matching

## 1. Intent and authority

Make temporal and numeric interval calculations readable, composable, and safe
to branch over. A relationship should return useful values: the occupied nights
in conflict, the boundary at which two stays meet, and the pieces available after
a maintenance interval is removed. Application code should not reconstruct these
results independently after asking an overlap Boolean.

The user selected `match left.relate(right)` with ordinary typed cases. This
document proposes its exact contract; it is not an accepted language rule or a
claim of existing implementation. The [checkpoint](LANG007.C5-CHECKPOINT.md)
owns state; [P1](LANG007.C5.P1-interval-algebra-and-relational-matching-plan.md)
owns execution.

Existing authorities to preserve:

- [specification authority](../../../specs/README.md);
- [runtime closed sums](../../../specs/objects/enums.md),
  [data values](../../../specs/objects/data-objects.md), and
  [traits](../../../specs/objects/traits.md);
- [pattern/type coherence laws](../../../specs/typing/type/type.reflection-pattern.md),
  especially T1063–T1078;
- [structural products](../../../specs/structural-products.md),
  [structural sums](../../../specs/structural-sums.md), and
  [Universe ownership](../../../specs/universe.md);
- [Range](../../../specs/collections/range.md) and
  [collection literal authority](../../../specs/collection-literals.md), whose
  inactive-operator text conflicts with current source and requires G1 resolution.

The historical [TYPE001.C6 matching contract](../../TYPE001-adt-gadt/C6-match-semantics-and-pattern-projection/match-semantics-exhaustiveness-and-gadt-proofs-spec.md)
explains the inherited total-match and semantic-to-executable architecture. It
does not override the effective `docs/specs/` owners.

## 2. Alternatives and selected design

| Approach | Benefit | Cost / hazard | Decision |
| --- | --- | --- | --- |
| Endpoint tests or overlap Booleans | Small library addition | Repeats boundary policy; callers calculate shared/residual pieces again | Useful convenience predicates only |
| Guarded arms such as `x if x.overlaps(y)` | Familiar reading | Guard syntax is absent in live match arms; general guard coverage is conservative; effects can repeat | Outside C5 |
| Direct relational patterns / extractors | Can bind algebraic results inside `match interval` | Requires a matcher protocol, failure/effect order, coverage rules, and disambiguation from structural patterns | Deferred |
| Closed relationship value from `a.relate(b)` | One computation, ordinary payload patterns, exhaustive cases, no new syntax | Explicit projection call; finite payload allocation may occur | Selected by user |

Three operations remain distinct:

```phalcom
// Structural decomposition of one interval's public view:
match window.view {
    IntervalView::Empty => ...
    IntervalView::Span(lower, upper) => ...
}

// Set relationship and algebraic pieces:
match requested.relate(existing) {
    IntervalRelation::Intersects(shared, requestedOnly, existingOnly) => ...
    // seven other cases
}

// Qualitative temporal geometry, where its preconditions hold:
match requested.allenRelation(existing) {
    Some(AllenRelation::During) => ...
    Some(_) => ...
    None => ...
}
```

All snippets here describe proposed APIs. `match window` retains normal matching
semantics; it does not secretly compare with another interval. A structural view
is requested explicitly with `.view`. C5 does not claim to add direct nominal
product patterns. Matching against a numeric range as a scalar-membership pattern
is also distinct and outside this checkpoint.

## 3. Value and domain model

### 3.1 Immutable validated values

Proposed declaration shapes, to be expressed with canonical source `data` and
`@variant` declarations in implementation:

```text
LowerBound<T> = Unbounded | Included(T) | Excluded(T)
UpperBound<T> = Unbounded | Included(T) | Excluded(T)
IntervalView<T> = Empty | Span(LowerBound<T>, UpperBound<T>)

Interval<T,D> private(view: IntervalView<T>, domain: D)
    where D conforms to the internal IntervalDomain<T> contract

IntervalPieces<T,D> = Zero | One(Interval<T,D>)
                         | Two(Interval<T,D>, Interval<T,D>)
```

Separate bound types prevent a positive-infinity lower bound or a
negative-infinity upper bound from being mistaken for a valid interval.
Unboundedness is a tag, never a numeric sentinel. `.view` returns immutable
canonical semantic data, with no storage pointers or live mutable backing.

`Interval` has a private raw primary constructor. Published factories validate
and normalize; public `IntervalView::Span` values are observations/descriptors,
not validated intervals. Passing a manually constructed view to a checked
factory must validate it again. There is no unchecked public constructor path.

`D` is a nominal domain type, retained by ordinary generic reification and
declaration identity. Domains are immutable nullary values. Do not introduce a
string-keyed domain registry or a new runtime ID namespace.

### 3.2 Bounded supported domains

| Facade | Value type / domain | Interpretation | Normal form |
| --- | --- | --- | --- |
| `IntegerIntervals` | `Int`, `IntegerDomain` | Arbitrary-precision discrete integers | Inclusive finite lower, exclusive finite upper |
| `ExactIntervals` | `Rational`, `RationalDomain` | Dense exact rational order | Preserve endpoint inclusion |
| `DateIntervals` | `Date`, `DateDomain` | Civil dates at one-day granularity | Inclusive finite lower, exclusive finite upper |
| `TimestampIntervals` | `Timestamp`, `TimestampDomain` | Absolute POSIX nanosecond coordinates | Inclusive finite lower, exclusive finite upper |

Factories include `empty`, `all`, `closedOpen(lower, upper)`, `closed`, `open`,
`openClosed`, and `fromBounds(lower, upper)`. These are named methods, not new
literal syntax. Each facade returns its specific `Interval<T,D>`.

Internally the domain owns comparison, endpoint validation, canonicalization,
and discrete successor. `DateDomain` uses calendar-day advancement, not adding
86,400 seconds. `TimestampDomain` uses exact integer nanoseconds. The Rational
factory accepts Rational endpoints; explicit Int-to-Rational promotion is owned
by the Range bridge or a named factory, never contextual guessing.

Finite Float endpoints, NaN, infinities encoded as endpoint values, Complex,
Period, and bare local DateTime/ZonedDateTime are outside initial support.
Reject unsupported inputs with `IntervalError`; never substitute `None`, empty,
zero, or an arbitrary order. Float order and representable-value topology require
a separately specified domain. A user-defined comparator is not automatically a
total, lawful order. Public custom-domain admission is deferred.

### 3.3 Validation and canonicalization

Construction rules, in this order:

1. Validate endpoint types and domain validity. An unordered comparison or
   illegal endpoint is an error.
2. If both supplied finite coordinates have lower > upper, raise `IntervalError`
   for reversed bounds. Do not silently swap them.
3. Normalize discrete bounds: excluded lower `x` becomes included `successor(x)`;
   included upper `y` becomes excluded `successor(y)`.
4. If normalized lower is beyond upper, or equal with no admitted point, use the
   one canonical empty value. This includes `(1,2)` over Int and `(d,d+1)` over Date.
5. Equal dense bounds produce a singleton only if both include the value;
   otherwise they produce empty. `[d,d]` over Date becomes `[d,d+1)`.
6. Unbounded sides carry no inclusion flag and do not call successor.

Successor must be exact. If a supported implementation cannot represent it,
report an error rather than wrap or reinterpret the bound as unbounded. Current
Int coordinates are unbounded integers; verify Date/Timestamp implementation
arithmetic at T2 before relying on that property.

Equality includes the domain and normalized view. Set-equivalent discrete
spellings, e.g. `[1,3]` and `[1,4)`, become equal Interval values. Existing Range
equality remains its stored-bound equality. C5 does not silently redefine it.

## 4. Set algebra and algorithm

For interval A, `members(A)` is the set of values of its declared domain between
its bounds, respecting inclusion. A and B must have the same `(T,D)`; typed calls
reject domain mismatch statically where provable. At a Dynamic boundary the
public API validates the actual domain before any comparison and raises an
ordinary `IntervalError` on mismatch. This is not a pattern miss.

Compare lower bounds with unbounded first; at equal finite coordinates an
included lower precedes an excluded lower. Compare upper bounds with unbounded
last; at equal coordinates an excluded upper precedes an included upper.

Algorithms:

```text
intersection(A,B):
    empty if either input is empty
    normalize(maxLower(A.lower,B.lower), minUpper(A.upper,B.upper))

subtract(A,B):
    S = intersection(A,B)
    if A empty: Zero
    if S empty: One(A)
    otherwise:
        left  = portion of A strictly before S
        right = portion of A strictly after S
        flip inclusion at each finite cut of S
        normalize; discard empty pieces; return Zero, One, or ordered Two

union(A,B):
    discard empties
    merge only when the actual union is one interval in this domain
    otherwise return ordered Two(A,B)

hull(A,B):
    discard empties; take minLower and maxUpper
    may fill missing values; keep its API distinct from union
```

For subtraction of `[2,3)` from `[1,4)`, the result is `[1,2)` and `[3,4)`.
For dense subtraction of the singleton `[2,2]` from `[1,3]`, the result is `[1,2)`
and `(2,3]`. Neither piece may include the removed point.

The internal computed-bound normalizer returns Empty for disjoint cuts; it does
not reuse the public constructor's reversed-input error. A computed empty
intersection is an ordinary result, not a user-input error.

No algorithm iterates interval members. Intersection and binary relation are a
bounded number of comparisons; subtraction and binary union emit at most two
pieces. `IntervalPieces::Two` contains sorted, nonempty, disjoint pieces with an
actual hole between them. `union` must merge a touching pair when it is connected.
`subtract` must preserve any excluded singleton hole.

## 5. Typed relationship value

The proposed closed sum has exactly these constructor shapes:

```text
IntervalRelation<T,D>:
    BothEmpty
    LeftEmpty(right: Interval<T,D>)
    RightEmpty(left: Interval<T,D>)
    Before(gap: Interval<T,D>)
    After(gap: Interval<T,D>)
    TouchesBefore(boundary: T, gap: Interval<T,D>)
    TouchesAfter(boundary: T, gap: Interval<T,D>)
    Intersects(shared: Interval<T,D>,
               leftOnly: IntervalPieces<T,D>,
               rightOnly: IntervalPieces<T,D>)
```

Use positional constructors/patterns for these fields in the public API. The
descriptive field names above identify values; implementation declarations use
`_` argument labels, as in `Intersects(_ shared, _ leftOnly, _ rightOnly)`.

`A.relate(B)` selects a case in this exact precedence:

1. The three empty cases.
2. Nonempty intersection: `Intersects(S, A−B, B−A)`.
3. Empty intersection with A's finite upper coordinate equal to B's finite
   lower coordinate: `TouchesBefore`.
4. The reverse equal-boundary case: `TouchesAfter`.
5. A wholly before B: `Before`; otherwise `After`.

`gap` is the actual unowned interval between the two sets, with complementary
inclusion at their facing boundaries. Empty gap is permitted only for a touch.
Equal facing coordinates with both included instead produce `Intersects` carrying
a singleton. Both excluded produce a touch with a singleton gap in a dense
domain. One included produces a touch with an empty gap. Discrete domains have
already normalized their cuts, so `[1,2]` and `[3,4]` over Int touch at 3 with no
missing integer. No epsilon or floating tolerance participates.

This order ensures the cases are disjoint and total. In particular, “overlap”
must not ambiguously mean both Allen's crossing relation and any shared values.
The convenience predicate `intersects` means intersection is nonempty;
`touches` means the returned case is either `TouchesBefore` or `TouchesAfter`.
`covers(B)` means `B−A` is empty, including when B is empty. Equal intervals
cover each other. Strict containment is available via proper Allen relations,
not an overloaded meaning of `covers`.

Ordinary library values are not unforgeable formal proofs. A caller can build an
`IntervalRelation::Intersects` value manually. Matching that value establishes
its exact case and payload types; it must not establish inequalities about
unrelated input bindings. Correct payloads are the `relate` API's runtime
contract, tested by the algebra laws. Any later formal producer-based refinement
needs its own canonical semantic specification.

## 6. Allen relations are a separate precise notion

For finite nonempty normalized spans with strict coordinates `a0<a1` and
`b0<b1`, `A.allenRelation(B)` returns `Some` of exactly one relation below.
Otherwise it returns `None`: empty, unbounded, or dense point intervals do not
get coerced into proper Allen spans.

| A relative to B | Coordinate condition | Converse |
| --- | --- | --- |
| Before | a1 < b0 | After |
| Meets | a1 = b0 | MetBy |
| Overlaps | a0 < b0 < a1 < b1 | OverlappedBy |
| Starts | a0 = b0 and a1 < b1 | StartedBy |
| During | b0 < a0 < a1 < b1 | Contains |
| Finishes | b0 < a0 and a1 = b1 | FinishedBy |
| Equal | a0 = b0 and a1 = b1 | Equal |
| FinishedBy | a0 < b0 and a1 = b1 | Finishes |
| Contains | a0 < b0 < b1 < a1 | During |
| StartedBy | a0 = b0 and b1 < a1 | Starts |
| OverlappedBy | b0 < a0 < b1 < a1 | Overlaps |
| MetBy | b1 = a0 | Meets |
| After | b1 < a0 | Before |

Allen geometry examines normalized coordinates; dense endpoint inclusion is a
separate set-algebra concern. `[1,2]` and `[2,3]` over Rational have Allen `Meets`
and a shared singleton, so their set relation is `Intersects`. Half-open hotel
stays meeting at checkout have empty intersection. The two queries deliberately
answer different questions.

This table is a concrete endpoint classifier, not the full algebra of relation
sets for uncertain intervals. Boolean combinations in match arms use existing
or-patterns with equal binding sets. Converse can be implemented as ordinary
closed-sum matching. Composition tables, constraint propagation, and fuzzy
endpoints are deferred.

The primary research precedent is
[Allen's interval reasoning paper](https://urresearch.rochester.edu/fileDownloadForInstitutionalItem.action?itemFileId=22361&itemId=10182).
It motivates the thirteen-way qualitative partition and relation reasoning;
C5 deliberately limits itself to concrete interval pairs.

## 7. Boundary and failure matrix

| Input / situation | Required result |
| --- | --- |
| Empty / empty | BothEmpty; intersection empty; union Zero |
| Nonempty / empty | RightEmpty; subtract One(left) |
| Rational `[1,2)` / `[2,3)` | TouchesBefore at 2, empty gap; union One |
| Rational `[1,2]` / `[2,3]` | Intersects with singleton `[2,2]` |
| Rational `[1,2)` / `(2,3]` | TouchesBefore at 2, singleton gap; union Two |
| Rational `[1,2]` / `(2,3]` | TouchesBefore at 2, empty gap; union One |
| Equal singleton `[2,2]` / `[2,2]` | Intersects singleton, both residuals Zero; Allen None |
| Int `[1,2]` / `[3,4]` | Canonical `[1,3)` / `[3,5)`, touching, union One |
| Int `(1,2)` | Canonical Empty |
| Reversed finite bounds | IntervalError at construction |
| All / finite | Intersects; subtraction may have two unbounded pieces |
| Rational `(-∞,2)` / `[2,+∞)` | TouchesBefore, empty gap; union all |
| Date versus Timestamp | Domain mismatch; no implicit conversion |
| Timestamp versus monotonic Instant | Domain mismatch; no interchangeable clock origins |
| NaN, Float, illegal endpoint type | IntervalError; not a non-match or empty result |
| Dynamic receiver / argument with unsupported domain | Runtime validation error before comparisons |
| Error evaluating either operand | Normal language exception propagation; no next-arm retry |
| Malformed source / unknown semantic evidence | Existing diagnostic/blocked rules; never fictitious coverage |

## 8. Range bridge

Range membership is numeric bound membership; integer unit-step iteration is a
separate operation. A Range with integral bounds can contain a fractional Number.
Therefore it cannot silently become an integer set merely because its endpoints
are Int.

Proposed methods:

- `Range.toExactInterval`: Int and Rational endpoints become exact Rational
  endpoints, lower included if present, upper inclusion retained; omitted bounds
  stay unbounded. Other endpoint types raise `IntervalError`.
- `Range.relate(other: Range)`: both receivers convert through
  `toExactInterval`, then use ordinary `Interval.relate`. Its result domain is
  Rational. Operand observation is once, in receiver/argument order.
- `Range.toIntegerInterval`: an explicitly requested integer restriction;
  initial version requires actual Int endpoints, then uses discrete normalization.
  It preserves `contains` only for Int membership; it is not a general Number view.

Examples after implementation:

```phalcom
let a = Range.new(1, 4, false)
let b = Range.new(3, 6, false)
match a.relate(b) {
    IntervalRelation::Intersects(shared, aOnly, bOnly) => ...
    _ => ...
}
```

Do not materialize Range, derive bounds from `first`/`last`, reinterpret a reversed
Range as a valid interval, change its existing hash/equality, or silently clamp
Float endpoints. `Range.toExactInterval` errors for reversed stored bounds even
if a legacy Range membership/size operation currently accepts them. The conversion
boundary is checked and additive; old Range operations retain their contract.

## 9. Semantic and execution ownership

```text
Universe .ph declarations and checked library method bodies
       ↓ canonical declaration / generic / callable identities
phalcom-semantic ordinary calls, case resolution, coverage, payload types
       ↓ MatchLoweringSpec / ExecutablePattern
compiler ordinary sends, variant tests, payload projections
       ↓ existing bytecode and VM data representation
runtime source interval methods and closed-sum values
```

The interval algorithm belongs to `universe.intervals`, implemented in `.ph`.
The semantic layer owns all formal match/type facts but does not duplicate the
runtime algebra in a hidden Rust classifier. Ordinary exhaustive sum coverage
handles the eight set cases and the thirteen Allen cases; it need not solve
endpoint inequalities. Existing snapshots, source indexes, and LSP consumers
expose the same canonical declarations and binding types.

Evaluation order is the ordinary method-call order: receiver expression once,
argument expression once, call once, returned scrutinee stored once, then
top-to-bottom pattern selection. No arm probes rerun `relate`. Or-patterns stage
and commit their usual compatible bindings. Exceptions propagate before branch
selection. Nested matching does not change loop, closure, or return semantics.

C5 introduces no new GC policy: interval/piece payloads are ordinary immutable
data values, rooted and traced through existing generic/data machinery. A GC
stress fixture checks that shared/residual pieces remain alive after their
original operands are dropped. Generic domain parameters must retain canonical
identity across imports, reification, and cold/incremental analysis.

No new scheduler, yielding protocol, reflection exemption, optimizer assumption,
or static-evaluator intrinsic is introduced. Later allocation elimination must
preserve ordinary sends and representation-independent data observations.

## 10. Invariants

| ID | Contract | Owner |
| --- | --- | --- |
| INV-01 | Interval is immutable and factory-validated; view construction alone is not validation | Source factories/data |
| INV-02 | Domain and endpoint inclusion are explicit; no implicit conversion or epsilon | Domain/factory methods |
| INV-03 | Empty has one normal form; discrete normalization preserves member sets | Domain normalization |
| INV-04 | Intersects iff intersection contains at least one domain value; touch is disjoint | Relation implementation |
| INV-05 | Eight relation cases form one disjoint total partition | Relation implementation/tests |
| INV-06 | Shared/residual pieces satisfy intersection/difference laws, ordered with no lost points | Algebra methods |
| INV-07 | Union preserves holes; hull is explicit | Algebra methods |
| INV-08 | Allen classification uses thirteen finite proper-span cases, otherwise None | Allen methods |
| INV-09 | Existing canonical matching owns case coverage and binding types | Semantic layer |
| INV-10 | Operands and relation computation evaluate once in source order | Ordinary call/match lowering |
| INV-11 | No new native interval algorithm, opcode, or duplicate static classifier | Universe/compiler boundary |
| INV-12 | Range bridge is explicit about numeric membership versus discrete iteration | Range adapter |

## 11. Precedent, novelty, and what this choice precludes

[Rust range patterns](https://doc.rust-lang.org/reference/patterns.html#range-patterns)
match scalar values within bounds. That is a useful scalar-membership feature,
but it does not supply two-interval relationships and their residual pieces.

[F# active patterns](https://learn.microsoft.com/en-us/dotnet/fsharp/language-reference/active-patterns)
can customize decomposition. They show that relation-based patterns are possible
elsewhere, with recognizer and partition contracts that a language must define.
C5 avoids adding that protocol by returning ordinary closed data.

[PostgreSQL range operators](https://www.postgresql.org/docs/current/functions-range.html)
already provide overlap, adjacency, intersection, and difference; its
[range model](https://www.postgresql.org/docs/current/rangetypes.html)
also distinguishes dense/discrete domains and canonicalization. Interval algebra
itself is not novel. The intended Phalcom benefit is a coherent standard API that
returns typed algebraic evidence to ordinary exhaustive matching and composes
with existing structural values, instead of application-specific endpoint chains.

Choosing this design precludes silently interpreting constructor patterns as
relational predicates, using arbitrary user matcher calls as coverage proofs,
and assuming allocation-free execution. It does not preclude later explicit
extractors, Float/custom domains, multiranges, scalar range coverage, interval
indexes, or Allen constraint networks. Those extensions must keep structural
matching and explicit relationship computation distinct.

## 12. Adoption requirements

P1.G1 must record adoption of: checked immutable representation, the four
domains, eight set cases, separate Allen query, exact Range conversion limits,
explicit structural `.view`, and ordinary source-only execution. It must also
resolve the existing Range specification conflict with its owning authority.

After adoption, the owning normative pages are proposed to be
`docs/specs/intervals.md` and the updated Range contract, indexed by the specs
README. Creating those accepted pages belongs to implementation T1, not this
planning-only task. Any requested direct product-pattern syntax or general
extractor protocol requires a separate bounded plan rather than a hidden T2 edit.
