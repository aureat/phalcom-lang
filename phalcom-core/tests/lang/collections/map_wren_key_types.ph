// area: collections
// spec: map-and-set.md §2; tuple-and-range.md §1; ADR-0039
// status: PASS
// Ported from wren/test/core/map/key_types.wren — a spread of hashable key
// types (Bool, Number, String, Class, Range, and the `None` absence
// singleton in place of Wren's surface `nil`, which Phalcom does not have —
// U6) all round-trip through `at(_, put:)`/`at(_)`, then get re-keyed with
// independently-constructed-but-equal objects to prove key equality is
// structural/value, not identity, for each type.

let m = Map.new()
m.at(None, put: "none value")
m.at(true, put: "true value")
m.at(false, put: "false value")
m.at(0, put: "zero")
m.at(1.2, put: "1 point 2")
m.at(List, put: "list class")
m.at("null", put: "string value")
m.at(Range.new(1, 3, true), put: "1 to 3")

System.print(m.at(None))
System.print(m.at(true))
System.print(m.at(false))
System.print(m.at(0))
System.print(m.at(1.2))
System.print(m.at(List))
System.print(m.at("null"))
System.print(m.at(Range.new(1, 3, true)))
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
m.at(Range.new(3 - 2, 1 + 2, true), put: "new 1 to 3")

System.print(m.at(None))
System.print(m.at(true))
System.print(m.at(false))
System.print(m.at(0))
System.print(m.at(1.2))
System.print(m.at(List))
System.print(m.at("null"))
System.print(m.at(Range.new(1, 3, true)))
System.print(m.size)
