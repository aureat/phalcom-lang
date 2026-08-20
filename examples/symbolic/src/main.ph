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

@sealed
class Ordering {
  @get _kind

  @class _less
  @class _equal
  @class _greater
  @class _unordered

  @private
  @constructor
  create(_ kind) {
    _kind = kind
  }

  @class
  less {
    if (_less == None) {
      _less = Ordering.create(#less)
    }

    _less
  }

  @class
  equal {
    if (_equal == None) {
      _equal = Ordering.create(#equal)
    }

    _equal
  }

  @class
  greater {
    if (_greater == None) {
      _greater = Ordering.create(#greater)
    }

    _greater
  }

  @class
  unordered {
    if (_unordered == None) {
      _unordered = Ordering.create(#unordered)
    }

    _unordered
  }

  @class
  new() {
    Error.new("Ordering values cannot be constructed directly").raise()
  }

  reverse {
    if (self == Ordering.less) {
      return Ordering.greater
    }

    if (self == Ordering.greater) {
      return Ordering.less
    }

    self
  }

  toString { toRepr }

  toRepr { "Ordering.\(kind.toString.trimStart("#"))" }
}

System.print(Ordering.less) // "Ordering.less"
System.print(Ordering.less == Ordering.less) // true
System.print(Ordering.less is Ordering) // true

const main = |*args| {
  // System.print(expr1.toRepr)
}

main()