/// Error raised when a byte sequence does not contain valid UTF-8.
class Utf8Error is Error {
  _validUpTo: Int
  _errorLength: Option<Int>

  @constructor
  new(validUpTo: Int, errorLength: Option<Int>) {
    super.new()
    _validUpTo = validUpTo
    _errorLength = errorLength
  }

  @constructor
  new(_ validUpTo: Int, _ errorLength: Option<Int>) {
    super.new()
    _validUpTo = validUpTo
    _errorLength = errorLength
  }

  @class
  call(validUpTo: Int, errorLength: Option<Int>) -> Utf8Error {
    Utf8Error.new(validUpTo, errorLength)
  }

  @class
  call(_ validUpTo: Int, _ errorLength: Option<Int>) -> Utf8Error {
    Utf8Error.new(validUpTo, errorLength)
  }

  /// Byte count of the valid prefix before the first invalid sequence.
  validUpTo -> Int { _validUpTo }
  /// Length of the invalid sequence when known; otherwise `None`.
  errorLength -> Option<Int> { _errorLength }
  message -> String { "invalid UTF-8 sequence" }
}

/// Error raised when parsing an integer from text fails.
class IntParseError is Error {
  _message: String
  _index: Option<Int>

  @constructor
  new(message: String, index: Option<Int>) {
    super.new()
    _message = message
    _index = index
  }

  @constructor
  new(message: String) {
    super.new()
    _message = message
    _index = None
  }

  @constructor
  new(_ message: String) {
    super.new()
    _message = message
    _index = None
  }

  @class
  call(message: String, index: Option<Int>) -> IntParseError {
    IntParseError.new(message, index)
  }

  @class
  call(message: String) -> IntParseError {
    IntParseError.new(message)
  }

  @class
  call(_ message: String) -> IntParseError {
    IntParseError.new(message)
  }

  message -> String { _message }
  index -> Option<Int> { _index }
}

/// Error raised when parsing a float from text fails.
class FloatParseError is Error {
  _message: String
  _index: Option<Int>

  @constructor
  new(message: String, index: Option<Int>) {
    super.new()
    _message = message
    _index = index
  }

  @constructor
  new(message: String) {
    super.new()
    _message = message
    _index = None
  }

  @constructor
  new(_ message: String) {
    super.new()
    _message = message
    _index = None
  }

  @class
  call(message: String, index: Option<Int>) -> FloatParseError {
    FloatParseError.new(message, index)
  }

  @class
  call(message: String) -> FloatParseError {
    FloatParseError.new(message)
  }

  @class
  call(_ message: String) -> FloatParseError {
    FloatParseError.new(message)
  }

  message -> String { _message }
  index -> Option<Int> { _index }
}

/// Error raised when parsing a boolean from text fails.
class BoolParseError is Error {
  _message: String
  _index: Option<Int>

  @constructor
  new(message: String, index: Option<Int>) {
    super.new()
    _message = message
    _index = index
  }

  @constructor
  new(message: String) {
    super.new()
    _message = message
    _index = None
  }

  @constructor
  new(_ message: String) {
    super.new()
    _message = message
    _index = None
  }

  @class
  call(message: String, index: Option<Int>) -> BoolParseError {
    BoolParseError.new(message, index)
  }

  @class
  call(message: String) -> BoolParseError {
    BoolParseError.new(message)
  }

  @class
  call(_ message: String) -> BoolParseError {
    BoolParseError.new(message)
  }

  message -> String { _message }
  index -> Option<Int> { _index }
}

export (
  Utf8Error,
  IntParseError,
  FloatParseError,
  BoolParseError
)
