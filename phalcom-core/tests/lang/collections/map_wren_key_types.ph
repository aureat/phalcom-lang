// area: collections
// spec: map-and-set.md §2; tuple-and-range.md §1; ADR-0039
// status: PASS
// Ported from wren/test/core/map/key_types.wren — a spread of hashable key
// types (Bool, Number, String, Class, and the `None` absence
// singleton in place of Wren's surface `nil`, which Phalcom does not have —
// U6) all round-trip through `at(_, put:)`/`[]`, then get re-keyed with
// independently-constructed-but-equal objects to prove key equality is
// structural/value, not identity, for each type.

const m = Map.new()
m.at(None, put: "none value")
m.at(true, put: "true value")
m.at(false, put: "false value")
m.at(0, put: "zero")
m.at(1.2, put: "1 point 2")
m.at(List, put: "list class")
m.at("null", put: "string value")

System.print(m[None])
System.print(m[true])
System.print(m[false])
System.print(m[0])
System.print(m[1.2])
System.print(m[List])
System.print(m["null"])
System.print(m.size)

// Use the same keys (but sometimes different objects) to prove key equality
// is structural/value, not identity.
m.at(None, put: "new none value")
m.at(not false, put: "new true value")
m.at(not true, put: "new false value")
m.at(2 - 2, put: "new zero")
m.at(1.2, put: "new 1 point 2")
m.at([].class, put: "new list class")
m.at("nu" + "ll", put: "new string value")

System.print(m[None])
System.print(m[true])
System.print(m[false])
System.print(m[0])
System.print(m[1.2])
System.print(m[List])
System.print(m["null"])
System.print(m.size)
