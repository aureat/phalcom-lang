/// Cell values: the root class and all value types.
/// DEC-VM-1: Every cell value is a user class instance because Number#+
/// raises on non-number arguments and cannot be overridden. This forces
/// error propagation to work: a formula like 1 + errorValue must produce
/// an error, not a crash.

/// Root of the cell value hierarchy. All cell values support arithmetic,
/// comparisons, and rendering.
class CellValue {
  /// Subclasses must override all arithmetic operations.
  plus(_ other) { ErrorVal.typeError }

  minus(_ other) { ErrorVal.typeError }

  times(_ other) { ErrorVal.typeError }

  dividedBy(_ other) { ErrorVal.typeError }

  modulo(_ other) { ErrorVal.typeError }

  lessThan(_ other) { ErrorVal.typeError }

  lessThanOrEqual(_ other) { ErrorVal.typeError }

  greaterThan(_ other) { ErrorVal.typeError }

  greaterThanOrEqual(_ other) { ErrorVal.typeError }

  isError { false }
}

/// A numeric cell value. Stores an f64.
class CellNum is CellValue {
  @constructor
  new(_ n) {
    _n = n
  }

  @class
  of(_ n) { CellNum.new(n) }

  value { _n }

  plus(_ other) {
    if (other.isError) {
      return other
    }

    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }

    CellNum.of(_n + other.value)
  }

  minus(_ other) {
    if (other.isError) {
      return other
    }

    if (other is not CellNum) {
      return ErrorVal.typeError
    }

    CellNum.of(_n - other.value)
  }

  times(_ other) {
    if (other.isError) {
      return other
    }

    if (not (other.is(CellNum))) {
      return ErrorVal.typeError
    }

    CellNum.of(_n * other.value)
  }

  dividedBy(_ other) {
    if (other.isError) {
      return other
    }

    if (not (other.is(CellNum))) {
      return ErrorVal.typeError
    }

    const on = other.value
    if (on == 0) {
      return ErrorVal.divByZero
    }

    CellNum.of(_n / on)
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

    CellNum.of(_n % on)
  }

  lessThan(_ other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }

    _n < other.value
  }

  lessThanOrEqual(_ other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }

    _n <= other.value
  }

  greaterThan(_ other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }

    _n > other.value
  }

  greaterThanOrEqual(_ other) {
    if (not (other is CellNum)) {
      return ErrorVal.typeError
    }

    _n >= other.value
  }

  asNumber -> Float { _n }

  toString {
    _n.toString
  }

  hash {
    _n.hash
  }

  ==(_ other) {
    if (not (other is CellNum)) {
      return false
    }

    _n == other.value
  }
}

/// A text cell value. Stores a string.
class CellText is CellValue {
  @constructor
  new(_ s) {
    _s = s
  }

  @class
  of(_ s) { CellText.new(s) }

  value { _s }

  plus(_ other) {
    if (other.isError) {
      return other
    }

    CellText.of(_s + other.toString)
  }

  minus(_ other) { ErrorVal.typeError }

  times(_ other) { ErrorVal.typeError }

  dividedBy(_ other) { ErrorVal.typeError }

  modulo(_ other) { ErrorVal.typeError }

  lessThan(_ other) { ErrorVal.typeError }

  lessThanOrEqual(_ other) { ErrorVal.typeError }

  greaterThan(_ other) { ErrorVal.typeError }

  greaterThanOrEqual(_ other) { ErrorVal.typeError }

  toString { _s }

  hash { _s.hash }

  ==(_ other) {
    if (not (other is CellText)) {
      return false
    }

    _s == other.value
  }
}

/// A boolean cell value.
class CellBool is CellValue {
  @constructor
  new(_ b) {
    _b = b
  }

  @class
  of(_ b) { CellBool.new(b) }

  value { _b }

  plus(_ other) { ErrorVal.typeError }

  minus(_ other) { ErrorVal.typeError }

  times(_ other) { ErrorVal.typeError }

  dividedBy(_ other) { ErrorVal.typeError }

  modulo(_ other) { ErrorVal.typeError }

  lessThan(_ other) { ErrorVal.typeError }

  lessThanOrEqual(_ other) { ErrorVal.typeError }

  greaterThan(_ other) { ErrorVal.typeError }

  greaterThanOrEqual(_ other) { ErrorVal.typeError }

  toString {
    if (_b) {
      return "true"
    }

    "false"
  }

  hash { _b.hash }

  ==(_ other) {
    if (other == nil) {
      return false
    }

    if (not (other is CellBool)) {
      return false
    }

    _b == other.value
  }
}

/// Empty cell (no value entered).
class CellEmpty is CellValue {
  @constructor
  new() {}

  @class
  of { CellEmpty.new() }

  plus(_ other) {
    if (other.isError) {
      return other
    }

    if (other is CellEmpty) {
      return CellEmpty.of
    }

    other
  }

  minus(_ other) {
    if (other.isError) {
      return other
    }

    if (other is CellEmpty) {
      return CellNum.of(0)
    }

    ErrorVal.typeError
  }

  times(_ other) {
    if (other.isError) {
      return other
    }

    CellNum.of(0)
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

    CellNum.of(0)
  }

  modulo(_ other) {
    if (other.isError) {
      return other
    }

    CellNum.of(0)
  }

  lessThan(_ other) { typeError }

  lessThanOrEqual(_ other) { ErrorVal.typeError }

  greaterThan(_ other) { ErrorVal.typeError }

  greaterThanOrEqual(_ other) { ErrorVal.typeError }

  toString { "" }

  hash { 0 }

  ==(_ other) {
    if (other == nil) {
      return false
    }

    other is CellEmpty
  }
}

let x = CellNum.of(10)
let y = CellNum.of(5)
let z = CellEmpty.new()
let result1 = x.minus(y)
let result2 = z.minus(x)

/// An error cell value. Stores a symbol like #DIV0 and display name.
class ErrorVal is CellValue {
  @constructor
  new(_ sym, _ display) {
    _sym = sym
    _display = display
  }

  @class
  typeError { ErrorVal.new(#TYPE, "#VALUE!") }

  @class
  divByZero { ErrorVal.new(#DIV0, "#DIV/0!") }

  @class
  circRef { ErrorVal.new(#CIRC, "#CIRC!") }

  @class
  nameError { ErrorVal.new(#NAME, "#NAME?") }

  symbol { _sym }

  displayName { _display }

  isError { true }

  plus(_ other) { self }

  minus(_ other) { self }

  times(_ other) { self }

  dividedBy(_ other) { self }

  modulo(_ other) { self }

  lessThan(_ other) { self }

  lessThanOrEqual(_ other) { self }

  greaterThan(_ other) { self }

  greaterThanOrEqual(_ other) { self }

  toString { _display }

  hash { _sym.hash }

  ==(_ other) {
    if (other == nil) {
      return false
    }

    if (not (other is ErrorVal)) {
      return false
    }

    _sym == other.symbol
  }
}

export (CellValue, CellNum, CellText, CellBool, CellEmpty, ErrorVal)