// Source-aware failure identity and immutable failure payloads.

@data
@immutable
class FailureOrigin {
  const _errorType: Class
  const _module: Symbol
  const _selector: Symbol
  const _line: Int
  const _column: Int
  const _label: Option<Symbol>

  @class
  unknown(errorType: Class) -> FailureOrigin {
    return FailureOrigin.new(
      errorType: errorType,
      module: #unknown,
      selector: #unknown,
      line: 0,
      column: 0,
      label: None
    )
  }

  sameSite(other: FailureOrigin) -> Bool {
    return _errorType == other.errorType and
      _module == other.module and
      _selector == other.selector and
      _line == other.line and
      _column == other.column and
      _label == other.label
  }
}

@data
@immutable
class Failure {
  const _origin: FailureOrigin
  const _error: Error
  const _example: Any
  const _arguments: List<Any>
  const _notes: List<Any>

  @class
  from(error: Error, example: Any, arguments: List<Any>) -> Failure {
    return self.from(
      error: error,
      example: example,
      arguments: arguments,
      notes: const []
    )
  }

  @class
  from(
    error: Error,
    example: Any,
    arguments: List<Any>,
    notes: List<Any>
  ) -> Failure {
    let origin = FailureOrigin.unknown(error.class)
    if error.respondsTo(#failureOrigin) {
      origin = error.failureOrigin
    }

    return Failure.new(
      origin: origin,
      error: error,
      example: example,
      arguments: _FailureCopies.list(arguments),
      notes: _FailureCopies.list(notes)
    )
  }

  sameOrigin(other: Failure) -> Bool { _origin.sameSite(other.origin) }
}

class _FailureCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
