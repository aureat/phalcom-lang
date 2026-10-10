from universe.text.writer import TextWriter

/// A mutable buffer for constructing UTF-8 strings efficiently.
class StringBuilder {
  mut _chunks: List<Dynamic>
  mut _byteCount: Int

  @constructor
  new() {
    _chunks = List<Dynamic>.new()
    _byteCount = 0
  }

  @constructor
  new(capacity: Int) {
    _chunks = List<Dynamic>.new()
    _byteCount = 0
  }

  @class
  call() -> StringBuilder {
    StringBuilder.new()
  }

  @class
  call(capacity: Int) -> StringBuilder {
    StringBuilder.new(capacity)
  }

  /// Appends a string, substring, character, or other text value to the buffer.
  append(_ text) -> StringBuilder {
    if text.is(Char) {
      _chunks.append(text.toString)
      _byteCount = _byteCount + text.utf8Length
      return self
    }
    let s = text.toString
    let len = s.rawByteCount
    if len > 0 {
      _chunks.append(s)
      _byteCount = _byteCount + len
    }
    self
  }

  /// Appends a trailing newline to the buffer.
  appendLine() -> StringBuilder {
    _chunks.append("\n")
    _byteCount = _byteCount + 1
    self
  }

  /// Appends text followed by a trailing newline to the buffer.
  appendLine(_ text) -> StringBuilder {
    self.append(text)
    self.appendLine()
  }

  /// Reserves capacity for at least `additionalBytes` more bytes.
  reserve(_ additionalBytes: Int) {
    self
  }

  /// Indicates whether the buffer contains zero bytes.
  isEmpty -> Bool {
    _byteCount == 0
  }

  /// Returns the current number of bytes accumulated in the buffer.
  byteCount -> Int {
    _byteCount
  }

  /// Clears all accumulated text from the buffer.
  clear() {
    _chunks = List<Dynamic>.new()
    _byteCount = 0
  }

  /// Conforms to TextWriter: writes a string.
  write(_ text: String) -> Unit {
    self.append(text)
    return ()
  }

  /// Conforms to TextWriter: writes a character.
  writeChar(_ char: Char) -> Unit {
    self.append(char)
    return ()
  }

  /// Assembles and returns all accumulated chunks as a single String.
  toString -> String {
    if _byteCount == 0 {
      return ""
    }
    String.rawJoin(_chunks)
  }
}

impl TextWriter for StringBuilder {}

export StringBuilder
