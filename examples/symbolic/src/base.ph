class TypeError is Error {}

class Expr {
  +(_ other) { BinOp.+(self, other) }

  -(_ other) { BinOp.-(self, other) }

  *(_ other) { BinOp.*(self, other) }

  /(_ other) { BinOp./(self, other) }

  %(_ other) { BinOp.%(self, other) }

  and(_ other) { BinOp.and(self, other) }

  or(_ other) { BinOp.or(self, Expr.coerce(other)) }

  @class
  coerce(_ other) {
    if (value is Expr) {
      value
    } else if (value is Number) {
      Var.number(value)
    } else if (value is Bool) {
      Var.bool(value)
    } else {
      throw TypeError.new("Cannot coerce value of type \(value.type) to Expr")
    }
  }
}

class Var is Expr {
  @get _symbol
  @get _type

  @constructor
  call(_ symbol) {
    _symbol = symbol
  }

  @constructor
  number(_ symbol) {
    _symbol = symbol
    _type = Number
  }

  @constructor
  bool(_ symbol) {
    _symbol = symbol
    _type = Bool
  }

  symbolTrimmed {
    _symbol.toString.trimStart("#")
  }

  toString {
    symbolTrimmed
  }

  toRepr {
    "Var(\(symbolTrimmed) is \(type))"
  }
}

class Const is Expr {
  @get _value
  @get _type

  @constructor
  @requires([Number, Bool].includes(value.class))
  new(_ value) {
    _value = value
    _type = value.class
  }

  toString {
    _value.toString
  }
}

class UnaryOp {
  @get _op
  @get _operand

  @constructor
  new(_ operand, of op) {
    _op = op
    _operand = operand
  }

  @class
  negated(operand) {
    UnaryOp.new(operand, of: "-")
  }

  @class
  logicalNot(operand) {
    UnaryOp.new(operand, of: "not")
  }

  toString {
    "\(op)\(operand.toString)"
  }
}

class BinOp {
  @get _op
  @get _left
  @get _right
  @get _type

  @constructor
  new(_ left, _ right, of op) {
    if (left.type != right.type) {
      throw TypeError.new("Operands must be of the same type for binary operation")
    }
    _op = op
    _left = left
    _right = right
    _type = left.type
  }

  @class
  +(_ left, _ right) {
    if (left.type != Number and right.type != Number) {
      throw TypeError.new("Both operands must be of type Number for addition")
    }
    BinOp.new(left, right, of: "+")
  }

  @class
  -(_ left, _ right) {
    if (left.type != Number and right.type != Number) {
      throw TypeError.new("Both operands must be of type Number for subtraction")
    }
    BinOp.new(left, right, of: "-")
  }

  @class
  *(_ left, _ right) {
    if (left.type != Number and right.type != Number) {
      throw TypeError.new("Both operands must be of type Number for multiplication")
    }
    BinOp.new(left, right, of: "*")
  }

  @class
  /(_ left, _ right) {
    if (left.type != Number and right.type != Number) {
      throw TypeError.new("Both operands must be of type Number for division")
    }
    BinOp.new(left, right, of: "/")
  }

  @class
  %(_ left, _ right) {
    if (left.type != Number and right.type != Number) {
      throw TypeError.new("Both operands must be of type Number for modulo")
    }
    BinOp.new(left, right, of: "%")
  }

  @class
  and(_ left, _ right) {
    if (left.type != Bool and right.type != Bool) {
      throw TypeError.new("Both operands must be of type Bool for logical AND")
    }
    BinOp.new(left, right, of: "and")
  }

  @class
  or(_ left, _ right) {
    if (left.type != Bool and right.type != Bool) {
      throw TypeError.new("Both operands must be of type Bool for logical OR")
    }
    BinOp.new(left, right, of: "or")
  }

  @class
  ==(_ left, _ right) {
    if (left.type != right.type) {
      throw TypeError.new("Operands must be of the same type for equality comparison")
    }
    BinOp.new(left, right, of: "==")
  }

  +(_ other) { BinOp.+(self, other) }

  -(_ other) { BinOp.-(self, other) }

  *(_ other) { BinOp.*(self, other) }

  /(_ other) { BinOp./(self, other) }

  %(_ other) { BinOp.%(self, other) }

  and(_ other) { BinOp.and(self, other) }

  or(_ other) { BinOp.or(self, other) }

  toString {
    const left = if (left is BinOp) { "(\(left.toString))" } else { left.toString }
    const right = if (right is BinOp) { "(\(right.toString))" } else { right.toString }
    "\(left) \(op) \(right)"
  }

  toRepr {
    const left = if (left is BinOp) { "(\(left.toRepr))" } else { left.toRepr }
    const right = if (right is BinOp) { "(\(right.toRepr))" } else { right.toRepr }
    "BinOp(op: \(op), left: \(left), right: \(right))"
  }
}

export Var, BinOp