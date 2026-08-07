// area: collections
// spec: collection-literals-and-map-spec.md §2.1-§2.2, §6
// status: PASS
// A bare map key is a Symbol, while a computed String key stays a String.
// They compare as distinct keys even when their text matches.

const m = { name: 1, ["name"]: 2 }
System.print(m.size)
System.print(m.at(#name))
System.print(m.at("name"))
