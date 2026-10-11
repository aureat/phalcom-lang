# Universe Collections

This directory specifies source-visible collection types and identifies
proposed COLL006 contracts separately. It does not prescribe a particular VM
object layout. The canonical iterator and iterable
protocols remain owned by [`../iterator.md`](../iterator.md), and Tuple/Record
structural typing remains owned by
[`../structural-products.md`](../structural-products.md) and
[`../tuple-and-record-typing.md`](../tuple-and-record-typing.md).

## Collection owners

| Type or surface | Owner | Role |
| --- | --- | --- |
| `List<T>` | [`list.md`](list.md) | Mutable, dynamically resizable sequence. |
| `Map<K,V>` | [`map.md`](map.md) | Mutable key/value association with deterministic encounter order. |
| `Set<T>` | [`set.md`](set.md) | Mutable unique-member collection. |
| `Range` | [`range.md`](range.md) | Lazy numeric interval and integer iteration source. |
| `Bytes` | [`bytes.md`](bytes.md) | Mutable byte sequence with explicit byte/text conversions. |
| `Path`, `OpenMode`, resources, and byte streams | [`../io.md`](../io.md) | Resource lifecycle and byte-oriented I/O. |
| `Array<T>` | [`array.md`](array.md) | Fixed-size mutable sequence target. |
| `Vector<T,N>` | [`vector.md`](vector.md) | Fixed-size immutable sequence target with static extent. |
| Bracket, map, tuple, and range literal syntax | [`../collection-literals.md`](../collection-literals.md) | Surface construction rules and their relationship to collection types. |
| `Tuple<...>` and `Record<...>` | [`../structural-products.md`](../structural-products.md) | Immutable heterogeneous structural products. |

`Array<T>` and `Vector<T,N>` record the proposed type-level collection design
direction in [COLL006](../../implementation/COLL006/STATUS.md). Their pages
define proposed type semantics only; their appearance here does not claim that
the design is normative or that its runtime/compiler implementation has been
accepted or completed.
The accepted bracket rule remains that `[a, b]` constructs a fresh mutable
`List`; a future defaulting or contextual selection rule requires an explicit
superseding specification.

## Shared principles

- Mutable collections expose mutation through their own APIs. Binding a
  collection to an immutable local does not make its referenced object
  immutable.
- Collection views and iterators have their documented ownership and
  invalidation behavior; an implementation must not silently replace a live
  view with an independent snapshot or vice versa.
- Encounter order is stable where the type specifies it. Equality rules may be
  independent of iteration order, as for `Map` and `Set`.
- The source contract does not require a particular heap arm, backing buffer,
  hash-table implementation, or allocation strategy. Those belong to VM
  specifications when they are part of the implementation contract.
