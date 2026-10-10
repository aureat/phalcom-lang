# Design notes

## Why this is source-level Telos

The implementation needs no special VM capability. It is composed entirely from
ordinary Telos facilities:

- generic data and classes;
- nominal singleton tag types (`data Tag()`);
- algebraic data variants;
- `Option`;
- mutable `List`;
- pattern matching;
- higher-order callables;
- canonical `Iterable` / `Iterator`;
- checked indexing.

The Rust reference implementation needs an occupied/vacant union, manual-drop
management, parity-encoded versions, and unsafe unchecked access because it is
implementing the storage representation inside Rust's ownership model. Telos
does not need to reproduce those mechanisms. Occupancy is modeled directly as
an algebraic data type.

## Sparse representation

```text
Slot<T> = {
  generation: Int,
  state:
    | Occupied(T)
    | Vacant(Option<Int>)
}
```

The map owns:

```text
slots : List<Slot<T>>
free  : Option<Int>
len   : Int
```

A vacant slot participates in the intrusive freelist. No separate free-index
collection is allocated.

## Dense representation

Stable slots contain either a dense index or a freelist link. Compact key/value
lists contain the live entries. A swap-removal repairs exactly one stable-slot
mapping when the last dense entry moves.

## Safety properties

1. Out-of-range handles return `None`.
2. Removed handles return `None`.
3. Slot reuse changes generation.
4. Clear changes generations of every occupied slot.
5. Dense swap-removal does not change user-visible keys.
6. Phantom tags reject cross-domain key use statically.
7. No public operation exposes an unchecked storage access path.


## API hygiene

`SlotKey<Tag>` is intentionally treated as an opaque capability-like handle.
Correctness tests assert liveness and staleness through `get`, `containsKey`,
and `remove` rather than depending on numeric slot/generation details.

`DenseSlotMap` does not expose its mutable `_keys` or `_values` backing lists.
Returning those lists directly would allow external mutation to violate the
stable-slot-to-dense-index bijection.
