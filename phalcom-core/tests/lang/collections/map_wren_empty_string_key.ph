// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// Ported from wren/test/core/map/empty_string_key.wren — the empty string is
// a well-formed hashable key like any other `String`.

const m = Map.new()
m.at("", put: "empty string")
System.print(m.at(""))
