from universe.scalar.string_index import StringIndex
from universe.text.substring import Substring

/// Represents a 1-indexed (line, column) coordinate in source text.
class SourceLocation {
  _line: Int
  _column: Int

  @constructor
  new(_ line: Int, _ column: Int) {
    _line = line
    _column = column
  }

  @class
  call(_ line: Int, _ column: Int) -> SourceLocation {
    SourceLocation.new(line, column)
  }

  @class
  call(line: Int, column: Int) -> SourceLocation {
    SourceLocation.new(line, column)
  }

  line -> Int { _line }
  column -> Int { _column }

  toString -> String {
    _line.toString + ":" + _column.toString
  }
}

/// Precomputed line boundary index providing O(log L) source coordinate mapping.
class LineMap {
  _text: Substring
  _lineStarts: List<Dynamic>

  @constructor
  new(_ source) {
    let sub = if source.is(Substring) { source } else { source.asSubstring }
    _text = sub
    let starts = List<Dynamic>.new()
    starts.append(0)
    let total = sub.byteCount
    let mut i = 0
    while i < total {
      let bOpt = sub.rawByteAt(i)
      if bOpt.isSome {
        let b = bOpt.unwrap
        if b == 10 {
          starts.append(i + 1)
        } else if b == 13 {
          if i + 1 < total and sub.rawByteAt(i + 1) == Some(10) {
            starts.append(i + 2)
            i = i + 1
          } else {
            starts.append(i + 1)
          }
        }
      }
      i = i + 1
    }
    _lineStarts = starts
  }

  @class
  call(_ source) -> LineMap {
    LineMap.new(source)
  }

  /// Translates a StringIndex or byte offset into a 1-indexed SourceLocation.
  location(of index) -> SourceLocation {
    let target = if index.is(StringIndex) { index.offset } else { index }
    let numLines = _lineStarts.size
    let mut low = 0
    let mut high = numLines - 1
    let mut best = 0
    while low <= high {
      let mid = (low + high) ~/ 2
      let startOfLine = _lineStarts.rawAt(mid)
      if startOfLine <= target {
        best = mid
        low = mid + 1
      } else {
        high = mid - 1
      }
    }
    let line = best + 1
    let lineStart = _lineStarts.rawAt(best)
    let mut col = 1
    let mut cur = lineStart
    while cur < target {
      let optCh = _text.decodeCharAt(cur)
      if optCh.isSome {
        cur = cur + optCh.unwrap.utf8Length
      } else {
        cur = cur + 1
      }
      col = col + 1
    }
    SourceLocation.new(line, col)
  }
}

export (
  SourceLocation,
  LineMap
)
