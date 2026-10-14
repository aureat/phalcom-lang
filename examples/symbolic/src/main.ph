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

data Ordering {
  Less
  Equal
  Greater
  Unordered(_ type1: Object, _ type2: Object)
}

impl Ordering {
  @class less { Ordering::Less }
  @class equal { Ordering::Equal }
  @class greater { Ordering::Greater }
  @class unordered(_ type1: Object, _ type2: Object) { Ordering::Unordered(type1, type2) }

  reverse {
    if (self == Ordering::Less) {
      return Ordering::Greater
    }

    if (self == Ordering::Greater) {
      return Ordering::Less
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
