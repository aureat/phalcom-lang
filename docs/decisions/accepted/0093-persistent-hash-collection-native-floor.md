# TDR-0093 — Admit the persistent hash collection native floor

- Ratified: 2026-10-11
- Status: Accepted
- Date: 2026-10-11
- Supersedes: none
- Amends: [TDR-0017](./0017-freeze-vm-blessed-primitive-floor.md)
- Related: [TDR-0028](./0028-collections-representation-and-literals.md), [TDR-0033](./0033-amend-floor-admit-collection-container-primitives.md), [TDR-0063](./0063-admit-bytes-native-octet-buffer.md), [TDR-0092](./0092-ordered-mutable-hash-collection-native-floor.md), and [`collection-families.md`](../../specs/collection-families.md)
- Implementation record: [`COLL006.C1.P7-api-and-floor-consultation.md`](../../implementation/COLL006/COLL006.C1/COLL006.C1.P7-api-and-floor-consultation.md)

This decision is the scoped P7 amendment to TDR-0017 and authorizes only the
exact 14 native method bindings for `Bag<T>` and `Bijection<K,V>` listed below.

## Context

TDR-0017 freezes the VM native primitive floor. TDR-0028 and TDR-0033 establish
that collection storage primitives may be implemented natively while high-level
operations (iteration, algebra, equality, transformations) reside in Telos source.
TDR-0092 admitted 29 bindings for the ordered mutable hash collection families.

P7 introduces persistent immutable hash collections backed by the GC-visible
`PersistentHashNode` substrate. To support `Bag<T>` and `Bijection<K,V>`, a minimal
native floor is required to manage immutable root pointers, exact multiplicity arithmetic,
atomic dual-root invariants, and callback-free key snapshots (under TDR-0063).

## Decision

### 1. Native admission

Admit exactly **14 new native method bindings** for `Bag<T>` and `Bijection<K,V>`:
seven and seven respectively.

#### Bag — 7 bindings

| Side | Selector | Parameters | Return Type | Responsibility |
|---|---|---|---|---|
| Class | `new()` | `()` | `Bag<T>` | Allocate empty `Bag` with `root = None`, `total = 0`, `distinct = 0` |
| Instance | `rawSize` | `()` | `Int` | Return cached total multiplicity |
| Instance | `rawDistinctCount` | `()` | `Int` | Return cached distinct key count |
| Instance | `rawCount(_)` | `(value: T)` | `Int` | Lookup multiplicity in persistent trie |
| Instance | `rawAdding(_,_)` | `(value: T, count: Int)` | `Bag<T>` | Two-phase persistent update returning new Bag |
| Instance | `rawRemoving(_,_)` | `(value: T, count: Int)` | `Bag<T>` | Two-phase persistent removal returning new Bag |
| Instance | `rawKeys` | `()` | `Array<T>` | Callback-free distinct key snapshot |

#### Bijection — 7 bindings

| Side | Selector | Parameters | Return Type | Responsibility |
|---|---|---|---|---|
| Class | `new()` | `()` | `Bijection<K,V>` | Allocate empty `Bijection` with dual empty roots |
| Instance | `rawSize` | `()` | `Int` | Return association count |
| Instance | `rawGet(_)` | `(key: K)` | `Option<V>` | Forward root key lookup |
| Instance | `rawAssociate(_,_)` | `(key: K, value: V)` | `Bijection<K,V>` | Atomic two-phase dual-root association or conflict error |
| Instance | `rawRemoveKey(_)` | `(key: K)` | `Bijection<K,V>` | Atomic two-phase removal from both roots |
| Instance | `rawInverse` | `()` | `Bijection<V,K>` | O(1) swap of forward and reverse roots |
| Instance | `rawKeys` | `()` | `Array<K>` | Callback-free forward key snapshot |

No other P7 native selector is admitted by this amendment.

### 2. Scope of native behavior

- Native methods are restricted to immutable root allocation, trie traversal,
  atomic path-copying persistent updates, and callback-free array snapshots (`rawKeys`).
- Multiset algebra (`+`, `union`, `intersection`, `difference`), equality loops (`==`),
  hashing, lazy iteration, and exception handling are implemented entirely in Telos source code.
- `rawKeys` complies with TDR-0063 bulk snapshot principles: it iterates the HAMT
  without executing any Telos bytecode or user callback.

### 3. Error and conflict policies

- `Bag#rawAdding` and `Bag#rawRemoving` reject negative count arguments with `ArgumentError`.
  Zero count returns the receiver itself.
- `Bijection#rawAssociate` enforces strict partial bijection:
  - If `(k, v)` already matches, returns `self`.
  - If `k` is mapped to another value or `v` is mapped to another key, raises `BijectionConflictError`.
  - No silent overwrite occurs.
- Failure during callbacks leaves all live objects and public roots unmodified.

### 4. Census impact

- Prior verified baseline: 458 declarations in `phalcom-native-surface/src/generated.rs`.
- With these 14 additions, the expected native surface declaration count is **472**.

## Consequences

- P7 can proceed with implementation of `Bag<T>` and `Bijection<K,V>` without any
  speculative native primitives.
- Pure Telos source files (`bag.ph` and `bijection.ph`) compose these 14 primitives
  into complete, idiomatic collection interfaces.
