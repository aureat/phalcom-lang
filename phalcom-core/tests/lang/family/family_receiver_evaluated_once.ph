// area: family
// spec: docs/spec/callables/family.md §1 and §5
// status: PASS
// The receiver expression is evaluated once when the Family is constructed;
// repeated Family calls reuse the captured receiver.

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
const family = Factory.make()::value()
System.print(family())
System.print(family())
