// area: family
// spec: selectors.md §3.1 (base-name index); ADR-0047
// status: PASS
// The base-name index is flattened through inheritance: a subclass's
// `::` reference sees a base name defined only on its superclass.

class Animal {
  speak() { return "..." }
}
class Dog extends Animal {
}
let d = Dog.new()
let f = d::speak
System.print(f())
