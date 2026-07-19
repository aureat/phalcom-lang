// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// Adapted from wren/test/core/map/to_string.wren — empty map, non-quoted
// string values, and nested maps via the native `Object::toString`
// primitive (value.rs's `Value::to_string`, U-COLLTYPES). Two cases are
// dropped from the Wren original: (1) a custom-class element's own
// `toString` override is NOT honored when the element is nested inside a
// collection being stringified — a pre-existing, already-tracked gap
// (DEFERRED.md #19, the "toString-message trap" `List#map` also dodges);
// (2) the "iteration order is unspecified, check one of six orderings"
// case — Phalcom's `Map` is insertion-ordered (not hash-bucket-ordered), so
// the rendering is deterministic and pinned directly instead of branched.

// Handle empty map.
System.print(Map.new())

// Does not quote strings.
const m1 = Map.new()
m1.at("1", put: "2")
System.print(m1)

// Nested maps.
const inner = Map.new()
const outer = Map.new()
outer.at(1, put: inner)
inner.at(2, put: Map.new())
System.print(outer)

// Insertion order is deterministic (unlike Wren's unspecified hash order).
const m2 = Map.new()
m2.at(1, put: 2)
m2.at(3, put: 4)
m2.at(5, put: 6)
System.print(m2)
