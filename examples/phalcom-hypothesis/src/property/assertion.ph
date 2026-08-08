// Source-aware property assertions.

import FailureOrigin from "core/failure"

class PropertyAssertionError is Error {
  @constructor
  new(message: String, origin: FailureOrigin) {
    super.new(message)
    _failureOrigin = origin
  }

  failureOrigin -> FailureOrigin => _failureOrigin
}

class Assert {
  @class
  equal(expected: Any, actual: Any) -> None {
    self.equalAt(
      expected: expected,
      actual: actual,
      location: SourceLocation.caller(skip: 1),
      label: None
    )
  }

  @class
  equal(expected: Any, actual: Any, because: Symbol) -> None {
    self.equalAt(
      expected: expected,
      actual: actual,
      location: SourceLocation.caller(skip: 1),
      label: Some.new(because)
    )
  }

  @class
  true(condition: Bool) -> None {
    self.trueAt(
      condition: condition,
      location: SourceLocation.caller(skip: 1),
      label: None
    )
  }

  @class
  true(condition: Bool, because: Symbol) -> None {
    self.trueAt(
      condition: condition,
      location: SourceLocation.caller(skip: 1),
      label: Some.new(because)
    )
  }

  @class
  false(condition: Bool) -> None {
    self.falseAt(
      condition: condition,
      location: SourceLocation.caller(skip: 1),
      label: None
    )
  }

  @class
  false(condition: Bool, because: Symbol) -> None {
    self.falseAt(
      condition: condition,
      location: SourceLocation.caller(skip: 1),
      label: Some.new(because)
    )
  }

  @class
  isTrue(condition: Bool) -> None => self.true(condition)

  @class
  isFalse(condition: Bool) -> None => self.false(condition)

  @class
  fail(message: String) -> None {
    self.failAt(
      message: message,
      location: SourceLocation.caller(skip: 1),
      label: None
    )
  }

  @class
  fail(message: String, because: Symbol) -> None {
    self.failAt(
      message: message,
      location: SourceLocation.caller(skip: 1),
      label: Some.new(because)
    )
  }

  @class
  @private
  equalAt(
    expected: Any,
    actual: Any,
    location: SourceLocation,
    label: Option<Symbol>
  ) -> None {
    if expected != actual {
      self.raise(
        message: "expected " + expected.toString +
          " but got " + actual.toString,
        location: location,
        label: label
      )
    }
  }

  @class
  @private
  trueAt(
    condition: Bool,
    location: SourceLocation,
    label: Option<Symbol>
  ) -> None {
    if not condition {
      self.raise(message: "expected true", location: location, label: label)
    }
  }

  @class
  @private
  falseAt(
    condition: Bool,
    location: SourceLocation,
    label: Option<Symbol>
  ) -> None {
    if condition {
      self.raise(message: "expected false", location: location, label: label)
    }
  }

  @class
  @private
  failAt(
    message: String,
    location: SourceLocation,
    label: Option<Symbol>
  ) -> None {
    self.raise(message: message, location: location, label: label)
  }

  @class
  @private
  raise(
    message: String,
    location: SourceLocation,
    label: Option<Symbol>
  ) -> None {
    throw PropertyAssertionError.new(
      message: message,
      origin: FailureOrigin.new(
        errorType: PropertyAssertionError,
        module: location.module,
        selector: location.selector,
        line: location.line,
        column: location.column,
        label: label
      )
    )
  }
}
