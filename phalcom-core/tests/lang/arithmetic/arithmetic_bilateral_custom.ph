// area: arithmetic
// spec: bilateral operators

class CustomVal {
  @get _val
  @constructor new(_ val) { _val = val }

  +(_ other) {
    if (other is CustomVal) {
      return CustomVal.new(_val + other.val)
    }
    unsupported
  }

  +(from other) {
    if (other is Int) {
      return CustomVal.new(other + _val)
    }
    unsupported
  }

  -(from other) {
    if (other is Int) {
      return CustomVal.new(other - _val)
    }
    unsupported
  }

  /(from other) {
    if (other is Int) {
      return CustomVal.new(other ~/ _val)
    }
    unsupported
  }

  toString { "CustomVal(\(_val))" }
}

class SubVal is CustomVal {
  @constructor new(_ val) { super.new(val) }

  +(from other) {
    CustomVal.new(999)
  }
}

// 1. Direct candidate
let a = CustomVal.new(10)
let b = CustomVal.new(20)
System.print(a + b)

// 2. Reflected candidate
System.print(5 + a)

// 3. Non-commutative reflected: 100 - a (where a.val = 10) => 90
System.print(100 - a)

// 4. Non-commutative reflected division: 100 / a => 10
System.print(100 / a)

// 5. Subtype priority override
let sub = SubVal.new(5)
System.print(1 + sub)
