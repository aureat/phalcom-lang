/// Structured parser-domain diagnostic.
///
/// `parse` reports parser-domain information only: offsets, expectations,
/// and grammar contexts. Mapping offsets to files, lines, and columns is a
/// source/text utility concern (see `TextSource.lineColAt`).
///
/// `ParseError` is a class (not data) because it carries merging and
/// rendering behavior.
class ParseError {
  mut _offset: Int
  mut _expected: List<String>
  mut _contexts: List<String>
  mut _message: Option<String>

  @constructor
  new(
    _ offset: Int,
    _ expected: List<String>,
    _ contexts: List<String>,
    _ message: Option<String>
  ) {
    _offset = offset
    _expected = expected
    _contexts = contexts
    _message = message
  }

  /// Offset where the failure was detected.
  offset -> Int {
    _offset
  }

  /// Ordered expectations accumulated by alternatives.
  expected -> List<String> {
    _expected
  }

  /// Grammar contexts framing the failure, innermost first.
  contexts -> List<String> {
    _contexts
  }

  /// Optional free-form message overriding expectation reporting.
  message -> Option<String> {
    _message
  }

  /// An error expecting a single described token at `offset`.
  @class
  expecting(_ offset: Int, _ expected: String) -> ParseError {
    let expectedList = List<String>.new()
    expectedList.append(expected)
    ParseError.new(offset, expectedList, List<String>.new(), None)
  }

  /// A free-form rejection at `offset`, such as a semantic failure.
  @class
  messageAt(_ offset: Int, _ message: String) -> ParseError {
    ParseError.new(offset, List<String>.new(), List<String>.new(), Some(message))
  }

  /// An internal invariant violation, such as a non-progressing repetition.
  @class
  internalAt(_ offset: Int, _ message: String) -> ParseError {
    ParseError.messageAt(offset, "internal grammar error: " + message)
  }

  /// String-list membership without depending on trait dispatch.
  @class
  listContains(_ list: List<String>, _ value: String) -> Bool {
    let mut i = 0
    while i < list.size {
      if list.at(i) == value {
        return true
      }
      i = i + 1
    }
    false
  }

  /// Furthest-offset merge used by alternatives.
  ///
  /// The more advanced failure wins. Failures at the same offset merge
  /// their expectations (left first, duplicates dropped) and keep the
  /// left-most message when present.
  merge(_ other: ParseError) -> ParseError {
    if other.offset > _offset {
      return other
    }
    if _offset > other.offset {
      return self
    }
    let merged = List<String>.new()
    let left = _expected
    let right = other.expected
    let mut i = 0
    while i < left.size {
      if not ParseError.listContains(merged, left.at(i)) {
        merged.append(left.at(i))
      }
      i = i + 1
    }
    let mut j = 0
    while j < right.size {
      if not ParseError.listContains(merged, right.at(j)) {
        merged.append(right.at(j))
      }
      j = j + 1
    }
    let message = match _message {
      Some(text) => { Some(text) }
      None => { other.message }
    }
    ParseError.new(_offset, merged, _contexts, message)
  }

  /// Pushes a grammar context onto the context stack unless it already
  /// frames the error.
  withContext(_ context: String) -> ParseError {
    if _contexts.size > 0 {
      if _contexts.first == Some(context) {
        return self
      }
    }
    let contexts = List<String>.new()
    contexts.append(context)
    let mut k = 0
    while k < _contexts.size {
      contexts.append(_contexts.at(k))
      k = k + 1
    }
    ParseError.new(_offset, _expected, contexts, _message)
  }

  /// Human-oriented rendering of this error.
  describe -> String {
    let mut text = ""
    match _message {
      Some(messageText) => {
        text = messageText
      }
      None => {
        if _expected.size == 0 {
          text = "parse failed"
        } else if _expected.size == 1 {
          text = "expected " + _expected.at(0)
        } else {
          text = "expected "
          let mut i = 0
          while i < _expected.size {
            if i > 0 {
              if i + 1 == _expected.size {
                text = text + " or "
              } else {
                text = text + ", "
              }
            }
            text = text + _expected.at(i)
            i = i + 1
          }
        }
      }
    }
    text = text + " at offset " + _offset.toString
    if _contexts.size > 0 {
      text = text + "\nwhile parsing:"
      let mut j = 0
      while j < _contexts.size {
        text = text + "\n  " + _contexts.at(j)
        j = j + 1
      }
    }
    return text
  }
}

export ParseError
