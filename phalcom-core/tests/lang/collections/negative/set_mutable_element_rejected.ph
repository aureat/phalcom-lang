// area: collections
// spec: map-and-set.md §3; collection-protocol.md law 4; DEC-CT-C
// status: NEGATIVE
// A mutable collection (`List`) is not a valid `Set` element — its identity
// `hash` is inconsistent with structural `==`. `add(_)` must raise a
// catchable Error, never silently identity-key it (the `Set` twin of
// `map_mutable_key_rejected.ph`).

let s = Set.new()
let elem = List.new()
s.add(elem)
