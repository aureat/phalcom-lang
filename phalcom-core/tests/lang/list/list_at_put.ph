// area: list
// spec: catalog-delta.md §2.4; ADR-0020; U-STD §2.6; DEFERRED.md #18
// status: PASS
// U-STD: `at(_, put:)` is the public wrapper over the `set_(_,_)` floor
// primitive — selector `at(_:put:)`, matching `set_`'s arity — writing
// `put` at index `i` and returning `self` so writes chain.

const l = []
l.append(10)
l.append(20)
l.append(30)
l.at(1, put: 99)
System.print(l.at(1))
System.print(l.at(0))
System.print(l.at(2))
