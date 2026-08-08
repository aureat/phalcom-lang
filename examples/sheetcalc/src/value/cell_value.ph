/// Cell values: the root class and all value types.
/// DEC-VM-1: Every cell value is a user class instance because Number#+
/// raises on non-number arguments and cannot be overridden. This forces
/// error propagation to work: a formula like 1 + errorValue must produce
/// an error, not a crash.

/// Root of the cell value hierarchy. All cell values support arithmetic,
/// comparisons, and rendering.
class CellValue {
  /// Subclasses must override all arithmetic operations.
  plus(_ other) {
    return ErrorVal.typeError
  }

  minus(_ other) {
    return ErrorVal.typeError
  }

  times(_ other) {
    return ErrorVal.typeError
  }

  dividedBy(_ other) {
    return ErrorVal.typeError
  }

  modulo(_ other) {
    return ErrorVal.typeError
  }

  lessThan(_ other) {
    return ErrorVal.typeError
  }

  lessThanOrEqual(_ other) {
    return ErrorVal.typeError
  }

  greaterThan(_ other) {
    return ErrorVal.typeError
  }

  greaterThanOrEqual(_ other) {
    return ErrorVal.typeError
  }

  isError {
    return false
  }
}

/// A numeric cell value. Stores an f64.
class CellNum is CellValue {
  @constructor
  new(_ n) {
    _n = n
  }

  @class
  of(_ n) {
    return CellNum.new(n)
  }

  value {
    return _n
  }

  plus(_ other) {
    if (other.isError) {
      return other
    }
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return CellNum.of(_n + other.value)
  }

  minus(_ other) {
    if (other.isError) {
      return other
    }
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return CellNum.of(_n - other.value)
  }

  times(_ other) {
    if (other.isError) {
      return other
    }
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return CellNum.of(_n * other.value)
  }

  dividedBy(_ other) {
    if (other.isError) {
      return other
    }
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    const on = other.value
    if (on == 0) {
      return ErrorVal.divByZero
    }
    return CellNum.of(_n / on)
  }

  modulo(_ other) {
    if (other.isError) {
      return other
    }
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    const on = other.value
    if (on == 0) {
      return ErrorVal.divByZero
    }
    return CellNum.of(_n % on)
  }

  lessThan(_ other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return _n < other.value
  }

  lessThanOrEqual(_ other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return _n <= other.value
  }

  greaterThan(_ other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return _n > other.value
  }

  greaterThanOrEqual(_ other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return _n >= other.value
  }

  asNumber {
    return _n
  }

  toString {
    const s = _n.toString
    return s
  }

  hash {
    return _n.hash
  }

  ==(_ other) {
    if (not (other is CellNum)) {
      return false
    }
    return _n == other.value
  }
}

/// A text cell value. Stores a string.
class CellText is CellValue {
  @constructor
  new(_ s) {
    _s = s
  }

  @class
  of(_ s) {
    return CellText.new(s)
  }

  value {
    return _s
  }

  plus(_ other) {
    if (other.isError) {
      return other
    }
    return CellText.of(_s + other.toString)
  }

  minus(_ other) {
    return ErrorVal.typeError
  }

  times(_ other) {
    return ErrorVal.typeError
  }

  dividedBy(_ other) {
    return ErrorVal.typeError
  }

  modulo(_ other) {
    return ErrorVal.typeError
  }

  lessThan(_ other) {
    return ErrorVal.typeError
  }

  lessThanOrEqual(_ other) {
    return ErrorVal.typeError
  }

  greaterThan(_ other) {
    return ErrorVal.typeError
  }

  greaterThanOrEqual(_ other) {
    return ErrorVal.typeError
  }

  toString {
    return _s
  }

  hash {
    return _s.hash
  }

  ==(_ other) {
    if (not (other is CellText)) {
      return false
    }
    return _s == other.value
  }
}

/// A boolean cell value.
class CellBool is CellValue {
  @constructor
  new(_ b) {
    _b = b
  }

  @class
  of(_ b) {
    return CellBool.new(b)
  }

  value {
    return _b
  }

  plus(_ other) {
    return ErrorVal.typeError
  }

  minus(_ other) {
    return ErrorVal.typeError
  }

  times(_ other) {
    return ErrorVal.typeError
  }

  dividedBy(_ other) {
    return ErrorVal.typeError
  }

  modulo(_ other) {
    return ErrorVal.typeError
  }

  lessThan(_ other) {
    return ErrorVal.typeError
  }

  lessThanOrEqual(_ other) {
    return ErrorVal.typeError
  }

  greaterThan(_ other) {
    return ErrorVal.typeError
  }

  greaterThanOrEqual(_ other) {
    return ErrorVal.typeError
  }

  toString {
    if (_b) {
      return "true"
    }
    return "false"
  }

  hash {
    return _b.hash
  }

  ==(_ other) {
    if (other == nil) {
      return false
    }
    if (not (other is CellBool)) {
      return false
    }
    return _b == other.value
  }
}

/// Empty cell (no value entered).
class CellEmpty is CellValue {
  @constructor
  new() {
  }

  @class
  of {
    return CellEmpty.new()
  }

  plus(_ other) {
    if (other.isError) {
      return other
    }
    if (other is CellEmpty) {
      return CellEmpty.of
    }
    return other
  }

  minus(_ other) {
    if (other.isError) {
      return other
    }
    if (other is CellEmpty) {
      return CellNum.of(0)
    }
    return ErrorVal.typeError
  }

  times(_ other) {
    if (other.isError) {
      return other
    }
    return CellNum.of(0)
  }

  dividedBy(_ other) {
    if (other.isError) {
      return other
    }
    if (other is CellEmpty) {
      return ErrorVal.divByZero
    }
    const on = other.asNumber
    if (on == nil) {
      return ErrorVal.typeError
    }
    if (on == 0) {
      return ErrorVal.divByZero
    }
    return CellNum.of(0)
  }

  modulo(_ other) {
    if (other.isError) {
      return other
    }
    return CellNum.of(0)
  }

  lessThan(_ other) {
    return ErrorVal.typeError
  }

  lessThanOrEqual(_ other) {
    return ErrorVal.typeError
  }

  greaterThan(_ other) {
    return ErrorVal.typeError
  }

  greaterThanOrEqual(_ other) {
    return ErrorVal.typeError
  }

  toString {
    return ""
  }

  hash {
    return 0
  }

  ==(_ other) {
    if (other == nil) {
      return false
    }
    return other is CellEmpty
  }
}

/// An error cell value. Stores a symbol like #DIV0 and display name.
class ErrorVal is CellValue {
  @constructor
  new(_ sym, _ display) {
    _sym = sym
    _display = display
  }

  @class
  typeError {
    return ErrorVal.new(#TYPE, "#VALUE!")
  }

  @class
  divByZero {
    return ErrorVal.new(#DIV0, "#DIV/0!")
  }

  @class
  circRef {
    return ErrorVal.new(#CIRC, "#CIRC!")
  }

  @class
  nameError {
    return ErrorVal.new(#NAME, "#NAME?")
  }

  symbol {
    return _sym
  }

  displayName {
    return _display
  }

  isError {
    return true
  }

  plus(_ other) {
    return self
  }

  minus(_ other) {
    return self
  }

  times(_ other) {
    return self
  }

  dividedBy(_ other) {
    return self
  }

  modulo(_ other) {
    return self
  }

  lessThan(_ other) {
    return self
  }

  lessThanOrEqual(_ other) {
    return self
  }

  greaterThan(_ other) {
    return self
  }

  greaterThanOrEqual(_ other) {
    return self
  }

  toString {
    return _display
  }

  hash {
    return _sym.hash
  }

  ==(_ other) {
    if (other == nil) {
      return false
    }
    if (not (other is ErrorVal)) {
      return false
    }
    return _sym == other.symbol
  }
}
