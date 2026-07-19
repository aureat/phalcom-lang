// area: collections
// spec: map-and-set.md §2; collection-protocol.md law 4; DEC-CT-C
// status: NEGATIVE
// A mutable collection (`List`) is not a valid `Map` key — its identity
// `hash` is inconsistent with structural `==`. `at(_, put:)` must raise a
// catchable Error, never silently identity-key it.

const m = Map.new()
const key = List.new()
m.at(key, put: 1)
