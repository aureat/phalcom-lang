// area: collections
// spec: collection-literals-and-map-spec.md §7; B.3 §6.2
// status: NEGATIVE
// Repeated bare Symbol labels are a compile-time provable duplicate.

let impossible = Map { same: 1, same: 2 }
