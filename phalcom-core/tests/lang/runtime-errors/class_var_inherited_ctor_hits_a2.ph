class Base {
  @class _count = 0

  @constructor
  new() {
    class.bump()
  }

  @class
  bump() {
    _count = _count + 1
  }
}

class Derived extends Base {}

let d = Derived.new()
