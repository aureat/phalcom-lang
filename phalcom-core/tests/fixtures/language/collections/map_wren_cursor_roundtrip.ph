// area: collections
// spec: map-and-set.md §2; iteration.md §1; ADR-0035 §1; ADR-0039; ADR-0048
// status: PASS
// Adapted from wren/test/core/map/{iterate,iterator_value}.wren onto
// Phalcom's cursor protocol: Wren's cursor is a raw int/`null`-terminated
// sequence with implementation-defined values; Phalcom's (post-Route B,
// U-ITERABLE, ADR-0048) is the bare-index/`None`-end-sentinel protocol
// (`iterate(_)`/`iteratorValue(_)`, C-ITER-9) — no per-step `Some` wrapper —
// so this pins the *shape* directly rather than asserting "is a Num and
// increasing" as Wren does. `iteratorValue(_)` yields the KEY (DEC-CT-E:
// both Map and Set yield keys via the cursor; there is no `MapEntry`
// wrapper — Wren's `MapEntry` has no Phalcom analog).

const m = Map.new()
m.at("one", put: 1)

let c = m.iterate(None)
System.print(c)
System.print(m.iteratorValue(c))

// The map mutating afterward does not retroactively change an already-read
// value (there is no MapEntry snapshot object in Phalcom — the value was
// simply read once above).
m.at("one", put: "updated")
System.print(m["one"])

// Full traversal to the end.
const m2 = Map.new()
m2.at(1, put: "a")
m2.at(2, put: "b")
let cursor = m2.iterate(None)
let seen = 0
while (cursor != None) {
  seen = seen + 1
  cursor = m2.iterate(cursor)
}
System.print(seen)
// `cursor` is now `None` (exhausted); `iterate(None)` always means "start
// from the beginning", so this restarts the traversal rather than staying
// past-the-end — pinning that restart-not-latch behavior.
System.print(m2.iterate(cursor))

// Nothing to iterate in an empty map.
System.print(Map.new().iterate(None))
