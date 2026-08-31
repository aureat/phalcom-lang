// area: collections
// spec: collection-literals-and-map-spec.md §7; B.3 §6.2
// status: NEGATIVE
// Repeated bare Symbol labels are a compile-time provable duplicate.

const impossible = { same: 1, same: 2 }
