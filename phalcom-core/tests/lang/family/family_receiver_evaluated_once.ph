// area: family
// spec: docs/spec/callables/family.md §1 and §5
// status: PASS

class Box {
  value() { 12 }
}
class Factory {
  @class
  make() {
    System.print("created")
    return Box.new()
  }
}
const family = (Box >> #value()).bind(Factory.make())
System.print(family())
System.print(family())
