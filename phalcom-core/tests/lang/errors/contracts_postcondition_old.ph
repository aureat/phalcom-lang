// area: errors
// spec: contract-annotations.md
// status: PASS
// contract: @ensures's old() captures the pre-state snapshot before the method body mutates it

class Box {
  @constructor
  new(_ init) {
    _val = init
  }

  val => _val

  @ensures(self.val == old(self.val) * 2)
  double() {
    _val = _val * 2
  }
}

const b = Box.new(7)
b.double()
System.print(b.val)
