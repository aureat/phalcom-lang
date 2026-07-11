// area: list
// spec: catalog-delta.md §2.4; ADR-0020; U-STD §2.6; DEFERRED.md #18
// status: PASS
// U-STD: `at(_, put:)` is the public wrapper over the `rawSet(_,_)` floor
// primitive — selector `at(_:put:)`, matching `rawSet`'s arity — writing
// `put` at index `i` and returning `self` so writes chain.

let l = List.new()
l.add(10)
l.add(20)
l.add(30)
l.at(1, put: 99)
System.print(l.at(1))
System.print(l.at(0))
System.print(l.at(2))
