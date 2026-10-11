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

## 7. Errors and representation boundary

The existing `ArgumentError`, `KeyError`, `IndexError`, `DuplicateKeyError`,
`BijectionConflictError`, and `Result` conventions are used as specified above.
User callback errors propagate unchanged. A failed validation, duplicate,
conflict, callback, or permitted integer-policy check leaves that operation's
receiver update unapplied.

The VM may use distinct native object arms and internal hash indices, but must
preserve the semantics in this document. No public `Hashable` constraint or
parallel hash/equality authority is introduced. Existing `Map` and `Set`
contracts are unaffected.
