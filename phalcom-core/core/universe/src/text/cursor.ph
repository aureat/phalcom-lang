from universe.scalar.string_index import StringIndex
from universe.text.substring import Substring

/// A sequential forward cursor navigating UTF-8 text.
class TextCursor {
  _source: Substring
  mut _offset: Int

  @constructor
  new(_ source) {
    _source = if source.is(Substring) { source } else { source.asSubstring }
    _offset = 0
  }

  @class
  call(_ text) -> TextCursor {
    TextCursor.new(text)
  }

  /// Indicates whether the cursor has reached or passed the end of the text.
  isAtEnd -> Bool {
    _offset >= _source.byteCount
  }

  /// Current byte offset from the beginning of the source text.
  offset -> Int {
    _offset
  }

  /// Current position as a StringIndex.
  index -> StringIndex {
    StringIndex(_offset)
  }

  /// Returns the unconsumed remaining text as a Substring.
  remaining -> Substring {
    _source.slice(_offset, _source.byteCount)
  }

  /// Looks ahead at the next Unicode scalar without advancing the cursor.
  peek() -> Option<Char> {
    if _offset >= _source.byteCount {
      return None
    }
    _source.decodeCharAt(_offset)
  }

  /// Consumes and returns the next Unicode scalar, advancing the cursor.
  next() -> Option<Char> {
    if _offset >= _source.byteCount {
      return None
    }
    let opt = _source.decodeCharAt(_offset)
    if opt.isSome {
      _offset = _offset + opt.unwrap.utf8Length
    }
    opt
  }

  /// Checks if the remaining text starts with `prefix`.
  startsWith(_ prefix) -> Bool {
    self.remaining.startsWith(prefix)
  }

  /// If the remaining text starts with `prefix`, advances past it and returns true.
  consume(_ prefix) -> Bool {
    if self.remaining.startsWith(prefix) {
      let len = if prefix.is(Char) {
        prefix.utf8Length
      } else if prefix.is(String) {
        prefix.rawByteCount
      } else if prefix.is(Substring) {
        prefix.byteCount
      } else {
        prefix.toString.rawByteCount
      }
      _offset = _offset + len
      return true
    }
    return false
  }

  /// Saves the current byte offset as an integer checkpoint.
  checkpoint() -> Int {
    _offset
  }

  /// Restores the cursor position to a previous checkpoint.
  restore(_ cp: Int) {
    _offset = cp
  }
}

export TextCursor
