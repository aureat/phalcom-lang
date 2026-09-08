# Maps

> Sources: accepted Map/Set specification; COLL002
> Raw: [collection specification and program snapshot](../raw/collections/2026-09-08-collections-sources.md)
> Updated: 2026-09-08

Map and Set are hash collections with insertion-order iteration stable within a run. Map lookup is total through `Some(v)` or `None`; Set membership is Boolean. Mutable collections use identity hashing while structural equality compares their contents under the collection protocol.

## Surface

Map construction, `at`, `put`, `size`, `includes`, `remove`, keys/values/entries, and one-value iteration form the core surface. Set construction, `add`, `includes`, `size`, `remove`, and one-value iteration form the corresponding set surface. Pair traversal is explicit through `entries`.

## Literal boundary

The map literal `{ a: 1 }` is accepted with symbol keys; `{}` remains an empty block and the empty map is constructed explicitly. Set literal syntax is reserved-inactive. COLL002 is proposed; accepted specification text is not proof of implementation.
