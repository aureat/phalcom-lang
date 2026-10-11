# `Map<K,V>`

`Map<K,V>` is a mutable mapping from keys of type `K` to values of type `V`.
`Map.new()` creates an empty instance. A key's hash and the keyed-collection
equivalence relation determine key identity; equivalent keys occupy one
association. Numeric keys use the SameValueZero-style equivalence specified by
[`../numeric-types.md`](../numeric-types.md), which groups NaNs and signed zeroes
for key lookup while ordinary Float equality follows IEEE semantics. Mutating
values for an existing key does not create a second entry.

## Operations and order

- `size`, `contains`/`includes`, and `get(key)` inspect the mapping. `get`
  returns `None` when the key is absent. Direct indexed lookup raises
  `KeyError`; the defaulted and `orElse` forms avoid that failure. `orPut`
  computes and inserts a value only when the key is absent.
- `insert(value, for: key)` stores the association and returns the previous
  value when present. `at(key, put: value)` and indexed assignment store the
  association; `remove(key)` and `clear` mutate the receiver.
- `keys`, `values`, and `entries` are live views. Their order is the Map's
  encounter order. `iterator` is a fresh entry iterator.
- `Map.from(record)` constructs a mutable mapping from a Record's labels and
  values.

Encounter order is deterministic and follows insertion order. Replacing an
existing key's value preserves that key's position; removing and reinserting a
key places the new association according to the collection's insertion order.
Equality is structural: maps compare by key membership and corresponding value
equality, independent of entry iteration order. A mutable Map cannot be a Map
key or Set member.

The map literal `{key: value, ...}` is accepted only for one or more
comma-separated key/value pairs. Bare identifier keys are Symbols; literal and
parenthesized expression keys use their expression values. Empty braces remain
an empty block; use `Map.new()` for an empty map. Elements evaluate left to
right and the literal returns a fresh mutable map. See
[`../collection-literals.md`](../collection-literals.md).

The source contract requires hash/equality consistency and safe re-entry of
user hash/equality behavior. It does not prescribe a particular hash-table
implementation.
