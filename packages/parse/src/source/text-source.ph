from ..core.pair import Pair
from ..core.source import Source

/// Stable byte-oriented source over an immutable `String`.
///
/// Tokens are UTF-8 bytes as integers (`Token = Int`); offsets and spans
/// are byte offsets, matching `String.slice` bounds. The underlying
/// string is immutable, so a running parser observes a stable sequence.
class TextSource {
  mut _text: String
  mut _view

  @constructor
  new(_ text: String) {
    _text = text
    _view = text.bytes
  }

  /// The full underlying text.
  text -> String {
    _text
  }

  /// Total number of bytes.
  size -> Int {
    _view.size
  }

  /// Byte value at `index`, or `-1` when out of bounds.
  at(_ index: Int) -> Int {
    if index >= 0 and index < _view.size {
      _view.at(index)
    } else {
      -1
    }
  }

  /// Substring by half-open byte range, for span rendering.
  slice(_ start: Int, _ end: Int) -> String {
    _text.slice(start, end)
  }

  /// Human-oriented position for an offset: line and column, both
  /// starting at 1. Columns count bytes since the last line break.
  lineColAt(_ offset: Int) -> Pair<Int, Int> {
    let mut line = 1
    let mut lastBreak = -1
    let mut i = 0
    while i < offset and i < _view.size {
      if _view.at(i) == 10 {
        line = line + 1
        lastBreak = i
      }
      i = i + 1
    }
    let mut column = offset - lastBreak
    if offset > _view.size {
      column = _view.size - lastBreak
    }
    Pair<Int, Int>(first: line, second: column)
  }
}

impl Source for TextSource {
  type Token = Int
}

export TextSource
