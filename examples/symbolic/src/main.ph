from .base import (
  Var,
  BinOp,
)

// const x = Var.number(#x)
// const y = Var.number(#y)
// const a = Var.number(#a)
// const b = Var.number(#b)

// const expr1 = (x + y) * (a - b) * 5 + (x / a) - (y % b)

// const z = Var.bool(#z)
// const w = Var.bool(#w)
// const expr2 = z + w
// System.print(z.toRepr)

@sealed @data
class Ordering {
  @variant Less()
  @variant Equal()
  @variant Greater()
  @variant Unordered(type1:, type2:)

  @private
  @constructor
  create(_ kind) {
    _kind = kind
  }

  @class less { Less.new() }
  @class equal { Equal.new() }
  @class greater { Greater.new() }
  @class unordered(_ type1, _ type2) { Unordered.new(type1: type1, type2: type2) }

  reverse {
    if (self == Ordering.less) {
      return Ordering.greater
    }

    if (self == Ordering.greater) {
      return Ordering.less
    }

    self
  }
}

System.print(Ordering.less)
System.print(Ordering.less == Ordering.less)
System.print(Ordering.unordered(String, Number).type1)
System.print(Ordering.unordered(String, Number).type2)

const main = |*args| {
  // System.print(expr1.toRepr)
}

main()