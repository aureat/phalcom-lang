# COLL006.C1.P7 — API Contract and Native Floor Consultation Packet

- Date: 2026-10-11
- Program: `COLL006` (Telos Collections Upgrade)
- Checkpoint: `COLL006.C1` (Collection Core & Representations)
- Plan: `COLL006.C1.P7` (Persistent Hash Substrate, Bag, & Bijection)
- Status: **PROPOSED / RATIFIED**

---

## 1. Executive Summary

This consultation packet establishes the authoritative public API contracts, error handling semantics, and VM native floor bindings for:
1. `Bag<T>` — Persistent immutable multiset with exact integer multiplicities and algebra.
2. `Bijection<K,V>` — Persistent immutable finite partial bijection with dual atomic roots, strict uniqueness, and O(1) inverse.
3. The native primitive floor amendment (TDR-0093) admitting exactly **14 native bindings** (7 for `Bag`, 7 for `Bijection`).
4. Formal disposition of the `R-P7-NUMERIC-HASH-LAW` finding.

---

## 2. Public API Specification

### 2.1 `Bag<T>`

`Bag<T>` is an immutable `Collection` and `Iterable` implementation. It associates each distinct member with a strictly positive exact integer count (`Int`).

#### Method Inventory

| Selector | Contract | Notes |
|---|---|---|
| `@constructor new() -> Bag<T>` | Empty bag. | Identity element for `+`. |
| `@constructor from<I>(_ source: I) -> Bag<T>` | Count occurrences from an `Iterable<T>`. | Stops at first iteration/hash error. |
| `size -> Int` / `totalCount -> Int` | Total occurrences across all distinct members. | Equivalent to sum of multiplicities. |
| `distinctCount -> Int` | Number of unique logical keys stored. | `0 <= distinctCount <= totalCount`. |
| `count(_ value: T) -> Int` | Multiplicity of `value`. | Returns `0` if absent. |
| `contains(_ value: T) -> Bool` | `count(value) > 0`. | Key membership test. |
| `adding(_ value: T) -> Bag<T>` | Returns new Bag with `count(value) + 1`. | Structural sharing with original. |
| `adding(_ value: T, count n: Int) -> Bag<T>` | Returns new Bag with `count(value) + n`. | If `n == 0`, returns `self`. If `n < 0`, raises `ArgumentError`. |
| `removing(_ value: T) -> Bag<T>` | Returns new Bag with `count(value) - 1`. | If `count == 1`, key is removed. If `count == 0`, returns `self`. |
| `removing(_ value: T, count n: Int) -> Bag<T>` | Returns new Bag with `max(0, count(value) - n)`. | Saturates at 0. If `n == 0`, returns `self`. If `n < 0`, raises `ArgumentError`. |
| `+(_ other: Bag<T>) -> Bag<T>` | Additive multiset sum. | For each key, `count = countA + countB`. |
| `union(_ other: Bag<T>) -> Bag<T>` | Maximum multiplicity combination. | For each key, `count = max(countA, countB)`. |
| `intersection(_ other: Bag<T>) -> Bag<T>` | Minimum multiplicity combination. | For each key, `count = min(countA, countB)`. Keys with count 0 omitted. |
| `difference(_ other: Bag<T>) -> Bag<T>` | Truncated subtraction. | For each key, `count = max(0, countA - countB)`. |
| `==(_ other: Dynamic) -> Bool` | Multiplicity equality. | True iff `other is Bag<T>` and for every key in either bag, multiplicities match. |
| `!=(_ other: Dynamic) -> Bool` | Negation of `==`. | |
| `hash -> Int` | Order-independent value hash. | Combines `(hash(key) ^ hash(count))` across all distinct entries. |
| `iterator -> BagIterator<T>` | Lazy iterator over all occurrences. | Yields each distinct key repeated by its multiplicity. |
| `entries -> Array<(value: T, count: Int)>` | Labeled snapshot of distinct entries. | Derived from `rawKeys`. |

---

### 2.2 `Bijection<K,V>`

`Bijection<K,V>` is an immutable one-to-one mapping between keys and values. It maintains dual persistent hash roots (`forward_root` mapping `K -> V` and `reverse_root` mapping `V -> K`) and guarantees `forward[k] == v` if and only if `reverse[v] == k`.

#### Conflict Policy & Error Channel

- **Idempotent Match:** If `k -> v` already exists exactly, `associating(k, v)` returns `self` (no error, no reallocation).
- **Conflict:**
  - If `k` is already associated with a different value `v2 != v`, `BijectionConflictError` is raised.
  - If `v` is already associated with a different key `k2 != k`, `BijectionConflictError` is raised.
- **Error Channel:** Thrown exception `BijectionConflictError is Error` with diagnostic message identifying the conflicting key or value.

#### Method Inventory

| Selector | Contract | Notes |
|---|---|---|
| `@constructor new() -> Bijection<K,V>` | Empty bijection. | |
| `@constructor from<I>(_ entries: I) -> Bijection<K,V>` | Construct from labeled `(key: K, value: V)` iterable. | Fails on first duplicate key or duplicate value. |
| `size -> Int` | Number of active associations. | Identical for forward and reverse roots. |
| `get(_ key: K) -> Option<V>` | Lookup value for key. | Returns `None` if key not present. |
| `[_ key: K] -> V` | Strict subscript lookup. | Returns `V` or raises `KeyError` if absent. |
| `keyFor(_ value: V) -> Option<K>` | Reverse lookup key for value. | Derived in O(1) via reverse root. |
| `containsKey(_ key: K) -> Bool` | Key membership test. | `get(key).isSome`. |
| `containsValue(_ value: V) -> Bool` | Value membership test. | `keyFor(value).isSome`. |
| `associating(_ key: K, _ value: V) -> Bijection<K,V>` | Strict persistent association. | Atomic preflight & commit. Throws `BijectionConflictError` on clash. |
| `removingKey(_ key: K) -> Bijection<K,V>` | Remove association by key. | Removes from both roots atomically; returns `self` if absent. |
| `removingValue(_ value: V) -> Bijection<K,V>` | Remove association by value. | Equivalent to `self.inverse.removingKey(value).inverse`. |
| `inverse -> Bijection<V,K>` | Returns inverse bijection. | **O(1) operation**: swaps forward and reverse roots. |
| `==(_ other: Dynamic) -> Bool` | Extensional mapping equality. | True iff `other is Bijection<K,V>`, `size` matches, and every `k -> v` matches. |
| `!=(_ other: Dynamic) -> Bool` | Negation of `==`. | |
| `hash -> Int` | Order-independent directional hash. | Combines `(hash(k) * 31 + hash(v))` across all associations. |
| `iterator -> BijectionIterator<K,V>` | Lazy iterator over labeled entries. | Yields `(key: K, value: V)`. |
| `keys -> Array<K>` | Forward key snapshot. | Uses `rawKeys`. |
| `values -> Array<V>` | Forward value snapshot. | Uses `inverse.keys`. |

---

## 3. Native Primitive Floor (TDR-0093 Candidate)

Admit exactly **14 native primitive bindings** (7 for `Bag`, 7 for `Bijection`):

### 3.1 `Bag` (7 bindings)

| Side | Selector | Parameters | Return Type | Responsibility |
|---|---|---|---|---|
| Class | `new()` | `()` | `Bag<T>` | Allocates empty `Bag` object with `root = None`, `total_count = 0`, `distinct_count = 0`. |
| Instance | `rawSize` | `()` | `Int` | Returns cached total multiplicity `total_count`. |
| Instance | `rawDistinctCount` | `()` | `Int` | Returns cached distinct key count `distinct_count`. |
| Instance | `rawCount(_)` | `(value: T)` | `Int` | Lookups `value` in persistent trie; returns stored count or 0. |
| Instance | `rawAdding(_,_)` | `(value: T, count: Int)` | `Bag<T>` | Two-phase HAMT insertion/update; returns new `Bag` object. |
| Instance | `rawRemoving(_,_)` | `(value: T, count: Int)` | `Bag<T>` | Two-phase HAMT removal/update; returns new `Bag` object. |
| Instance | `rawKeys` | `()` | `Array<T>` | Callback-free traversal producing snapshot `Array` of distinct keys. |

### 3.2 `Bijection` (7 bindings)

| Side | Selector | Parameters | Return Type | Responsibility |
|---|---|---|---|---|
| Class | `new()` | `()` | `Bijection<K,V>` | Allocates empty `Bijection` with `forward_root = None`, `reverse_root = None`, `len = 0`. |
| Instance | `rawSize` | `()` | `Int` | Returns `len`. |
| Instance | `rawGet(_)` | `(key: K)` | `Option<V>` | Lookups `key` in `forward_root`. |
| Instance | `rawAssociate(_,_)` | `(key: K, value: V)` | `Bijection<K,V>` | Two-phase dual-root preflight/commit; throws `BijectionConflictError` on conflict. |
| Instance | `rawRemoveKey(_)` | `(key: K)` | `Bijection<K,V>` | Lookups value for key; if present, removes from both forward and reverse roots. |
| Instance | `rawInverse` | `()` | `Bijection<V,K>` | O(1) constructor swapping `forward_root` and `reverse_root`. |
| Instance | `rawKeys` | `()` | `Array<K>` | Callback-free traversal of `forward_root` producing snapshot `Array` of keys. |

### 3.3 Audit of Bulk Mechanism (`rawKeys`)

Under TDR-0063, callback-free container operations are permissible in native code. `rawKeys` traverses the immutable persistent hash trie without calling any Telos method, returning a rooted Telos `Array`. This snapshot powers all high-level operations (`iterator`, algebra, equality, hashing, `values`) cleanly in Telos source code without requiring complex iterative native state machines.

---

## 4. `R-P7-NUMERIC-HASH-LAW` Disposition

### Finding Summary
In `phalcom-core/src/primitive/number.rs`, `Number#hash` uses `val.to_bits()` for floats $\ge 2^{53}$ and NaNs, while using `DefaultHasher` for `LargeInt`. Under `same_value_zero`, equal representations (e.g. $2^{53}+2$, $2^{64}$) return true, but their hashes diverge.

### Disposition
Per Plan §18 (T1) and checkpoint guidance, `Number#hash` is an established repository-wide contract that must **not** be modified in P7. Persistent collections use the established `ordered_hash.rs` helpers (`hash_key` and `keys_equal`), which preserves the exact behavior of existing `Map`, `Set`, `IndexedDict`, `OrderedSet`, and `MutableBag`. The strict mathematical identity test is documented and pinned in `tests/core/collections/contract.rs`.

---

## 5. Decision & Ratification

The architecture, error models, and 14 native bindings are hereby ratified for implementation in T5 (`Bag<T>`) and T6 (`Bijection<K,V>`).
