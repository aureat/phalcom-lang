/// Cell values: the root class and all value types.
/// DEC-VM-1: Every cell value is a user class instance because Number#+
/// raises on non-number arguments and cannot be overridden. This forces
/// error propagation to work: a formula like 1 + errorValue must produce
/// an error, not a crash.

/// Root of the cell value hierarchy. All cell values support arithmetic,
/// comparisons, and rendering.
class CellValue {
  /// Subclasses must override all arithmetic operations.
  plus(other) {
    return ErrorVal.typeError
  }

  minus(other) {
    return ErrorVal.typeError
  }

  times(other) {
    return ErrorVal.typeError
  }

  dividedBy(other) {
    return ErrorVal.typeError
  }

  modulo(other) {
    return ErrorVal.typeError
  }

  lessThan(other) {
    return ErrorVal.typeError
  }

  lessThanOrEqual(other) {
    return ErrorVal.typeError
  }

  greaterThan(other) {
    return ErrorVal.typeError
  }

  greaterThanOrEqual(other) {
    return ErrorVal.typeError
  }

  isError {
    return false
  }
}

/// A numeric cell value. Stores an f64.
class CellNum extends CellValue {
  construct new(n) {
    _n = n
  }

  static of(n) {
    return CellNum.new(n)
  }

  value {
    return _n
  }

  plus(other) {
    if (other.isError) {
      return other
    }
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return CellNum.of(_n + other.value)
  }

  minus(other) {
    if (other.isError) {
      return other
    }
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return CellNum.of(_n - other.value)
  }

  times(other) {
    if (other.isError) {
      return other
    }
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return CellNum.of(_n * other.value)
  }

  dividedBy(other) {
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

  modulo(other) {
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

  lessThan(other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return _n < other.value
  }

  lessThanOrEqual(other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return _n <= other.value
  }

  greaterThan(other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }
    return _n > other.value
  }

  greaterThanOrEqual(other) {
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

  ==(other) {
    if (not (other is CellNum)) {
      return false
    }
    return _n == other.value
  }
}

/// A text cell value. Stores a string.
class CellText extends CellValue {
  construct new(s) {
    _s = s
  }

  static of(s) {
    return CellText.new(s)
  }

  value {
    return _s
  }

  plus(other) {
    if (other.isError) {
      return other
    }
    return CellText.of(_s + other.toString)
  }

  minus(other) {
    return ErrorVal.typeError
  }

  times(other) {
    return ErrorVal.typeError
  }

  dividedBy(other) {
    return ErrorVal.typeError
  }

  modulo(other) {
    return ErrorVal.typeError
  }

  lessThan(other) {
    return ErrorVal.typeError
  }

  lessThanOrEqual(other) {
    return ErrorVal.typeError
  }

  greaterThan(other) {
    return ErrorVal.typeError
  }

  greaterThanOrEqual(other) {
    return ErrorVal.typeError
  }

  toString {
    return _s
  }

  hash {
    return _s.hash
  }

  ==(other) {
    if (not (other is CellText)) {
      return false
    }
    return _s == other.value
  }
}

/// A boolean cell value.
class CellBool extends CellValue {
  construct new(b) {
    _b = b
  }

  static of(b) {
    return CellBool.new(b)
  }

  value {
    return _b
  }

  plus(other) {
    return ErrorVal.typeError
  }

  minus(other) {
    return ErrorVal.typeError
  }

  times(other) {
    return ErrorVal.typeError
  }

  dividedBy(other) {
    return ErrorVal.typeError
  }

  modulo(other) {
    return ErrorVal.typeError
  }

  lessThan(other) {
    return ErrorVal.typeError
  }

  lessThanOrEqual(other) {
    return ErrorVal.typeError
  }

  greaterThan(other) {
    return ErrorVal.typeError
  }

  greaterThanOrEqual(other) {
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

  ==(other) {
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
class CellEmpty extends CellValue {
  construct new() {
  }

  static of {
    return CellEmpty.new()
  }

  plus(other) {
    if (other.isError) {
      return other
    }
    if (other is CellEmpty) {
      return CellEmpty.of
    }
    return other
  }

  minus(other) {
    if (other.isError) {
      return other
    }
    if (other is CellEmpty) {
      return CellNum.of(0)
    }
    return ErrorVal.typeError
  }

  times(other) {
    if (other.isError) {
      return other
    }
    return CellNum.of(0)
  }

  dividedBy(other) {
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

  modulo(other) {
    if (other.isError) {
      return other
    }
    return CellNum.of(0)
  }

  lessThan(other) {
    return ErrorVal.typeError
  }

  lessThanOrEqual(other) {
    return ErrorVal.typeError
  }

  greaterThan(other) {
    return ErrorVal.typeError
  }

  greaterThanOrEqual(other) {
    return ErrorVal.typeError
  }

  toString {
    return ""
  }

  hash {
    return 0
  }

  ==(other) {
    if (other == nil) {
      return false
    }
    return other is CellEmpty
  }
}

/// An error cell value. Stores a symbol like #DIV0 and display name.
class ErrorVal extends CellValue {
  construct new(sym, display) {
    _sym = sym
    _display = display
  }

  static typeError {
    return ErrorVal.new(#TYPE, "#VALUE!")
  }

  static divByZero {
    return ErrorVal.new(#DIV0, "#DIV/0!")
  }

  static circRef {
    return ErrorVal.new(#CIRC, "#CIRC!")
  }

  static nameError {
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

  plus(other) {
    return self
  }

  minus(other) {
    return self
  }

  times(other) {
    return self
  }

  dividedBy(other) {
    return self
  }

  modulo(other) {
    return self
  }

  lessThan(other) {
    return self
  }

  lessThanOrEqual(other) {
    return self
  }

  greaterThan(other) {
    return self
  }

  greaterThanOrEqual(other) {
    return self
  }

  toString {
    return _display
  }

  hash {
    return _sym.hash
  }

  ==(other) {
    if (other == nil) {
      return false
    }
    if (not (other is ErrorVal)) {
      return false
    }
    return _sym == other.symbol
  }
}
