# Collection Families

This specification defines the source-visible behavior shared by mutable
hash-keyed collection families and the public contracts of `MutableBag<T>`,
`IndexedDict<K,V>`, and `OrderedSet<T>`. It complements the common collection
and iteration protocols. The VM collection specification owns their physical
representation and tracing requirements.

## 1. Shared rules

These three types are mutable `Collection` and `Iterable` implementations.
Their generic parameters are invariant. They use Telos `hash` and `==` for
membership and key comparison, including the existing numeric SameValueZero
key rule. They do not gain a `Hashable` or `Comparable` requirement. A mutable
collection instance is not admissible as a key or member in a hash-keyed
collection because its observable contents can change while stored.

An operation that compares a candidate key or member invokes user `hash` and
`==` without holding a heap borrow across the callback. A callback error
propagates. A callback may read the collection being inspected; structural
mutation of that same receiver is rejected for the duration of key inspection.
`IndexedDict` value-only overwrite remains permitted during key inspection.
Mutation side effects on other objects are not rolled back when an operation
fails. An operation validates and prepares its receiver update before making a
single coherent publication of that update.

The collection iterators are independent live positional traversals. Each
`next` observes the current source contents at its current position; changes
between calls may therefore affect later results. Iterators retain their source
collection. `MutableBag`'s expanded iterator snapshots the count for one
distinct entry when it enters that entry, emits that many copies, and then
continues with later source positions. It does not materialize the expanded
bag.

For `==`, each operand's logical contents are copied into GC-rooted snapshots
before the first user equality callback. The entire comparison reads those
snapshots. Callback mutations may commit to either live collection, but do not
change the result of the comparison already in progress. Callback errors
propagate.

## 2. `MutableBag<T>`

`MutableBag<T>` records a positive exact integer multiplicity for each distinct
member. Its `size` and `totalCount` are the total number of occurrences;
`distinctCount` is the number of distinct members. Empty counts are zero.
`from<I>(_ source: I)` consumes an `Iterable` of `T` in source order and counts
repeated equal values. It stops at the first iteration, hash, equality, or
count error.

| Selector | Contract |
|---|---|
| `new() -> Self` | Construct an empty bag. |
| `from<I>(_ source: I) -> Self` | Construct by counting each source occurrence. |
| `size`, `totalCount -> Int` | Exact total occurrence count. |
| `distinctCount -> Int` | Number of stored distinct members. |
| `count(_ value: T) -> Int` | Exact multiplicity, or zero when absent. |
| `contains(_ value: T) -> Bool` | Whether multiplicity is positive. |
| `add(_ value: T) -> Self` | Add one occurrence. |
| `add(_ value: T, count n: Int) -> Self` | Add `n`; zero is a no-op; a negative count raises `ArgumentError`. |
| `remove(_ value: T) -> Self` | Remove up to one occurrence; absence is a no-op. |
| `remove(_ value: T, count n: Int) -> Self` | Remove up to `n`; excess saturates at zero; zero is a no-op; negative raises `ArgumentError`. |
| `clear -> Unit` | Remove all members and counts. |
| `iterator -> MutableBagIterator<T>` | Iterate every occurrence in first-admission order under the shared live-iteration rule. |
| `entries -> MutableBagEntriesView<T>` | Live positional view of labeled `(value: T, count: Int)` entries in first-admission order. |
| `==(_ other: Self) -> Bool` | Compare multiplicity functions independent of encounter order. |

Only positive counts are stored. Total count is the exact sum of entry counts.
Count and total arithmetic obey the active VM integer policy; failure leaves
the receiver's content and indices unchanged. Equality is multiplicity-based
and uses key/member equality, not order.

## 3. `IndexedDict<K,V>`

`IndexedDict<K,V>` is an ordered mapping. Key operations and positional
operations are separate: integer keys remain keys when `K` is `Int`. Replacing
an existing key preserves its position and stored key representative;
removing and later reinserting a key appends it unless positional insertion is
requested. `from<I>(_ source: I)` consumes labeled `(key: K, value: V)` entries
in source order and raises `DuplicateKeyError` at the first duplicate without
requesting later entries.

| Selector | Contract |
|---|---|
| `new() -> Self` | Construct an empty dictionary. |
| `from<I>(_ source: I) -> Self` | Construct from unique labeled entries in source order. |
| `size -> Int` | Number of entries. |
| `get(_ key: K) -> Option<V>` | Value for a key, or `None`. |
| `[_ key: K] -> V` | Strict keyed lookup; missing key raises `KeyError`. |
| `contains(_ key: K) -> Bool` | Whether a key is present. |
| `insert(_ value: V, for key: K) -> Option<V>` | Append a new key and return `None`, or replace its value in place and return `Some(previousValue)`. |
| `insert(_ entry: (key: K, value: V), at i: Int) -> Result<Unit, Error>` | Strictly insert a new key at a position; duplicate gives `DuplicateKeyError`, invalid position gives `IndexError`. |
| `entryAt(_ i: Int) -> Option<(key: K, value: V)>` | Entry at a position, or `None`. |
| `keyAt(_ i: Int) -> Option<K>` | Key at a position, or `None`. |
| `valueAt(_ i: Int) -> Option<V>` | Value at a position, or `None`. |
| `indexOfKey(_ key: K) -> Option<Int>` | Position of a key, or `None`. |
| `move(_ from: Int, to i: Int) -> Result<Unit, Error>` | Move an entry to final post-move position `i`; same position is a no-op. |
| `removeAt(_ i: Int) -> Result<(key: K, value: V), Error>` | Remove and return the entry at a position. |
| `remove(_ key: K) -> Self` | Remove the key if present; absence is a no-op. |
| `clear -> Unit` | Remove all entries. |
| `iterator -> IndexedDictIterator<K,V>` | Iterate labeled entries in current positional order. |
| `==(_ other: Self) -> Bool` | Compare ordered entry sequences; key positions use key equivalence and values use Telos equality. |

For a collection of length `n`, a negative positional index `i` normalizes to
`n+i`, using exact integer arithmetic. Reads, removal, and move source positions
must normalize into `[0,n)`. Insert positions normalize into `[0,n]`, so `size`
appends. The move destination is the final position and normalizes against the
pre-move length into `[0,n)`; `size` is invalid. Invalid positional operations
return `IndexError` through `Result`. An integer key passed to keyed subscript
or keyed methods is never interpreted as a position.

An overwrite invoked reentrantly during a key callback reads the value current
at its commit point for `Some(previousValue)`, then commits the outer requested
value. The outer commit therefore wins; a failure before that commit does not
roll back an already committed inner value overwrite.

## 4. `OrderedSet<T>`

`OrderedSet<T>` contains unique members in encounter order. `from<I>(_ source: I)`
is strict: the first duplicate raises `ArgumentError` and later input is not
consumed. `add` is idempotent and preserves the existing representative and
position. Positional insertion is strict and rejects an existing member.

| Selector | Contract |
|---|---|
| `new() -> Self` | Construct an empty ordered set. |
| `from<I>(_ source: I) -> Self` | Construct from unique members in source order. |
| `size -> Int` | Number of unique members. |
| `contains(_ value: T) -> Bool` | Whether a member is present. |
| `add(_ value: T) -> Self` | Append when absent; duplicate is a no-op. |
| `insert(_ value: T, at i: Int) -> Result<Unit, Error>` | Strict positional insertion; duplicate gives `ArgumentError`, invalid position gives `IndexError`. |
| `at(_ i: Int) -> Option<T>` | Member at a position, or `None`. |
| `indexOf(_ value: T) -> Option<Int>` | Position of a member, or `None`. |
| `replaceAt(_ i: Int, with value: T) -> Result<T, Error>` | Replace and return the prior value; a duplicate elsewhere gives `ArgumentError`; replacing with an equal member at the same slot is a no-op preserving the representative. |
| `move(_ from: Int, to i: Int) -> Result<Unit, Error>` | Move to final post-move position; same position is a no-op. |
| `remove(_ value: T) -> Self` | Remove a member if present; absence is a no-op. |
| `removeAt(_ i: Int) -> Result<T, Error>` | Remove and return the member at a position. |
| `clear -> Unit` | Remove all members. |
| `iterator -> OrderedSetIterator<T>` | Iterate in current positional order. |
| `==(_ other: Self) -> Bool` | Compare members position by position using key/member equivalence. |

Positional indices, including negative indices, use the same exact
normalization and valid ranges as `IndexedDict`. Insert at `size` appends;
move-to-size is invalid. Equality is order-sensitive. Existing `Set` behavior
remains unchanged and order-insensitive.

## 5. `Bag<T>`

`Bag<T>` is a persistent immutable multiset implementation of `Collection` and `Iterable`.
It records an exact positive integer multiplicity for each distinct member.

| Selector | Contract |
|---|---|
| `new() -> Self` | Construct an empty bag. |
| `from<I>(_ source: I) -> Self` | Construct by counting occurrences from an `Iterable<T>`. |
| `size`, `totalCount -> Int` | Exact total occurrence count across all distinct members. |
| `distinctCount -> Int` | Number of distinct members stored. |
| `count(_ value: T) -> Int` | Exact multiplicity, or zero when absent. |
| `contains(_ value: T) -> Bool` | Whether multiplicity is strictly positive. |
| `adding(_ value: T) -> Self` | Return new Bag with `count(value) + 1`. |
| `adding(_ value: T, count n: Int) -> Self` | Return new Bag with `count(value) + n`; zero is identity; negative raises `ArgumentError`. |
| `removing(_ value: T) -> Self` | Return new Bag with `count(value) - 1`; absence is identity. |
| `removing(_ value: T, count n: Int) -> Self` | Return new Bag with `count(value) - n`; excess saturates at zero; zero is identity; negative raises `ArgumentError`. |
| `+(_ other: Self) -> Self` | Additive multiset sum (`count = countA + countB`). |
| `union(_ other: Self) -> Self` | Maximum multiplicity combination (`count = max(countA, countB)`). |
| `intersection(_ other: Self) -> Self` | Minimum multiplicity combination (`count = min(countA, countB)`). |
| `difference(_ other: Self) -> Self` | Truncated multiset subtraction (`count = max(0, countA - countB)`). |
| `==(_ other: Dynamic) -> Bool` | Multiplicity equality independent of insertion or internal traversal order. |
| `hash -> Int` | Order-independent value hash based on member and count pairs. |
| `iterator -> BagIterator<T>` | Lazy iterator yielding each occurrence in accordance with member multiplicity. |
| `entries -> Array<(value: T, count: Int)>` | Snapshot array of labeled distinct value and count pairs. |

## 6. `Bijection<K,V>`

`Bijection<K,V>` is a persistent immutable finite partial bijection between keys and values.
It maintains dual persistent hash roots and guarantees `forward[k] == v` if and only if
`reverse[v] == k`.

| Selector | Contract |
|---|---|
| `new() -> Self` | Construct an empty bijection. |
| `from<I>(_ source: I) -> Self` | Construct from unique labeled `(key: K, value: V)` entries in source order. |
| `size -> Int` | Number of associations. |
| `get(_ key: K) -> Option<V>` | Value for key, or `None`. |
| `[_ key: K] -> V` | Strict subscript lookup; missing key raises `KeyError`. |
| `keyFor(_ value: V) -> Option<K>` | Reverse lookup key for value in O(1), or `None`. |
| `containsKey(_ key: K) -> Bool` | Whether key is present. |
| `containsValue(_ value: V) -> Bool` | Whether value is present. |
| `associating(_ key: K, _ value: V) -> Self` | Strict persistent association; idempotent if `(k, v)` exists; raises `BijectionConflictError` if `k` or `v` is already mapped to a different counterpart. |
| `removingKey(_ key: K) -> Self` | Return new Bijection with association for `key` removed; absence is identity. |
| `removingValue(_ value: V) -> Self` | Return new Bijection with association for `value` removed; absence is identity. |
| `inverse -> Bijection<V,K>` | O(1) root swap producing inverse bijection. |
| `==(_ other: Dynamic) -> Bool` | Extensional mapping equality: identical size and identical associations. |
| `hash -> Int` | Order-independent directional association hash. |
| `iterator -> BijectionIterator<K,V>` | Lazy iterator over labeled `(key: K, value: V)` tuples. |
| `keys -> Array<K>` | Snapshot array of forward keys. |
| `values -> Array<V>` | Snapshot array of forward values. |

## 7. `Bound<T>` and `Interval<T>`

`Bound<T>` and `Interval<T>` define the generic immutable interval domain foundation.
Interval semantics are independent of numeric/progression/slicing `Range`.

### 7.1 `Bound<T>`

`Bound<T>` represents an interval endpoint. It has exactly three semantic states:

```telos
data Bound<T> {
  @variant Unbounded
  @variant Included(value: T)
  @variant Excluded(value: T)
}
```

`Unbounded` has no fixed position independent of endpoint role:
- As a lower bound, `Unbounded` represents $-\infty$.
- As an upper bound, `Unbounded` represents $+\infty$.

#### Endpoint Ordering

Endpoints are ordered via canonical bilateral `<=>` comparison of their underlying finite values, with the following boundary tie-breaking rules:

- **Lower bound ordering** (`compareLower`):
  - `Unbounded < Included(x) < Excluded(x)`
  - For equal finite values $x$: `Included(x) < Excluded(x)`
- **Upper bound ordering** (`compareUpper`):
  - `Excluded(x) < Included(x) < Unbounded`
  - For equal finite values $x$: `Excluded(x) < Included(x)`

Any comparison between finite endpoint values that yields `Ordering::Unordered` is rejected with `InvalidIntervalError`.

### 7.2 `Interval<T>`

`Interval<T>` represents a generic interval over an ordered domain defined by lower and upper bounds:

```telos
data Interval<T>(_ lower: Bound<T>, _ upper: Bound<T>)
```

#### Validity and Emptiness

- **Valid nonempty**: `lower < upper`, or `lower == upper` with both endpoints `Included` (a singleton $[x, x]$).
- **Valid empty**: `lower == upper` with at least one endpoint `Excluded` (e.g. $[x, x)$, $(x, x]$, or $(x, x)$).
- **Invalid**: finite `lower > upper`, or any endpoint comparison resulting in `Ordering::Unordered`.

#### Operations

| Selector | Contract |
|---|---|
| `lower -> Bound<T>` | Lower endpoint bound. |
| `upper -> Bound<T>` | Upper endpoint bound. |
| `isValid -> Bool` | Whether the interval is mathematically valid (not descending or unordered). |
| `isEmpty -> Bool` | Whether the interval contains zero points. |
| `contains(_ point: T) -> Bool` | Point containment: `point` satisfies lower bound ($p \ge x$ if Included, $p > x$ if Excluded, true if Unbounded) AND upper bound ($p \le y$ if Included, $p < y$ if Excluded, true if Unbounded). |
| `overlaps(_ other: Interval<T>) -> Bool` | True iff both intervals are valid, nonempty, and share at least one point. |
| `intersection(_ other: Interval<T>) -> Option<Interval<T>>` | Intersection of two intervals: `max(lowerA, lowerB)` and `min(upperA, upperB)`. Returns `Some(interval)` if valid and nonempty; `None` otherwise. |
| `touchesWithoutGap(_ other: Interval<T>) -> Bool` | Contiguity without overlap: true iff intervals are disjoint and touch at a common boundary point covered by one side's inclusion and the other side's exclusion (e.g. $[a, b)$ and $[b, c]$). |
| `unionIfContiguous(_ other: Interval<T>) -> Option<Interval<T>>` | If `touchesWithoutGap` or `overlaps`, returns `Some(Interval(min(lowerA, lowerB), max(upperA, upperB)))`; otherwise `None`. |
| `==(_ other: Dynamic) -> Bool` | Value equality of bounds. |
| `hash -> Int` | Deterministic hash over endpoint bounds. |

## 8. `RangeMap<K,V>`

`RangeMap<K,V>` is a persistent, immutable mapping from pairwise-disjoint, valid, non-empty `Interval<K>` regions to values `V`.

### 8.1 Invariants

1. **Non-overlapping**: All stored intervals are pairwise non-overlapping.
2. **Point lookup uniqueness**: Any point $k \in K$ belongs to at most one stored interval.
3. **Logarithmic lookup**: Point lookup traverses a balanced persistent tree in $O(\log n)$ time.
4. **Persistence & immutability**: All mutating operations return a new `RangeMap` root, preserving older versions completely intact through structural sharing.
5. **Ordered domain precondition**: Keys used as endpoints must have stable ordering-relevant observable state for the lifetime of any `RangeMap` retaining them.

### 8.2 Selectors and Contracts

| Selector | Contract |
|---|---|
| `new() -> Self` | Construct an empty `RangeMap`. |
| `size -> Int` | Number of stored disjoint interval entries. |
| `isEmpty -> Bool` | Whether the map contains zero entries. |
| `get(_ point: K) -> Option<V>` | Point lookup: finds the unique interval containing `point` and returns `Some(value)`, or `None`. $O(\log n)$ complexity. |
| `containsPoint(_ point: K) -> Bool` | True iff `get(point)` is `Some(_)`. |
| `inserting(_ interval: Interval<K>, _ value: V) -> Self` | Strict insertion: inserts `interval -> value`. Rejects invalid or empty intervals with `InvalidIntervalError`. Rejects any overlap with existing entries with `RangeOverlapError`. Adjoining non-overlapping intervals are permitted. |
| `removing(_ interval: Interval<K>) -> Self` | Coverage subtraction: removes all coverage of `interval`. Discards fully covered entries, truncates partially covered entries, and splits an interior-covered entry into left and right remainders. Empty interval is identity; invalid interval raises `InvalidIntervalError`. |
| `overlaying(_ interval: Interval<K>, _ value: V) -> Self` | Range overlay: replaces mapping over `interval` with `value`. Truncates or splits overlapping existing entries to preserve unaffected remainders outside `interval`. Architected as $O(\log n + k)$ where $k$ is the number of overlapped entries. Empty interval is identity; invalid interval raises `InvalidIntervalError`. |
| `iterator -> RangeMapIterator<K,V>` | In-order traversal by canonical lower bound yielding labeled `(interval: Interval<K>, value: V)` tuples. |
| `entries -> Array<(interval: Interval<K>, value: V)>` | Snapshot array of in-order logical entries. |
| `==(_ other: Dynamic) -> Bool` | Extensional equality: identical normalized logical entries in canonical order, independent of internal tree topology. |
| `hash -> Int` | Deterministic fold over normalized logical entries. |

### 8.3 Normalization and Coalescing

When two adjacent entries have Telos-equal values (`v1 == v2`) and their intervals touch without an unmapped gap (`intervalA.touchesWithoutGap(intervalB)`), they are coalesced into a single entry with interval `intervalA.unionIfContiguous(intervalB)`. Adjacent intervals with a gap between them are never coalesced.

## 9. Errors and representation boundary

The existing `ArgumentError`, `KeyError`, `IndexError`, `DuplicateKeyError`,
`BijectionConflictError`, and `Result` conventions are used as specified above.

The interval and range map family adds:
- `InvalidIntervalError is ArgumentError`: raised when an interval has invalid bounds (e.g. finite `lower > upper`), when an operation encounters `Ordering::Unordered`, or when a non-empty interval is required but an empty interval was provided.
- `RangeOverlapError is ArgumentError`: raised by `RangeMap.inserting` when the requested interval overlaps any existing stored interval.

User callback errors propagate unchanged. A failed validation, duplicate,
conflict, overlap, callback, or permitted integer-policy check leaves that
operation's receiver update unapplied and publishes no partial root.

The VM and library layer preserve the semantics in this document. No public
`Hashable` or `Comparable` constraint is introduced. Existing `Map`, `Set`,
and numeric `Range` contracts are unaffected.

