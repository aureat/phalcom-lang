// area: family
// spec: docs/spec/callables/reflection.md §2
// status: PASS
// Pattern capture walks effective behavior: subclass overrides shadow the base
// exact route while unmatched inherited routes remain in the snapshot.

class Base {
  route(_ value) { "base-positional" }
  route(to) { "base-labeled" }
}
class Child is Base {
  route(_ value) { "child-positional" }
}

const family = Child >> #route(...)
System.print(family.size)
const bound = family.bind(Child.new())
System.print(bound(1))
System.print(bound(to: 2))
