// area: runtime-errors
// spec: U-LIST-plan.md §3/§7; ADR-0020; DEFERRED.md #18
// status: NEGATIVE
// Adapted from Wren `test/core/list/insert_index_too_large.wren` /
// `insert_index_too_small.wren`: Phalcom has no `insert(_:_:)` (its `List`
// API is `add`/`at(_:put:)`, not positional insertion), but `at(_:put:)`
// shares the same "index must be in range" law as Wren's `insert` — writing
// past the end is a hard type error, never a silent grow or panic.

const l = List.new()
l.add(1)
l.add(2)
l.add(3)
System.print(l.at(4, put: 9))
