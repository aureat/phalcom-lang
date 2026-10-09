from universe.text.substring import Substring

/// A pattern that can search for matches within a Substring.
trait TextPattern {
  /// Searches for the first occurrence of this pattern in `source` at or after byte offset `from`.
  /// Returns `Option<(Int, Int)>` representing `(startByteOffset, endByteOffset)` relative to `source`.
  findIn(_ source: Substring, _ offset: Int) -> Option<(Int, Int)>
  findIn(_ source: Substring, from offset: Int) -> Option<(Int, Int)> {
    self.findIn(source, offset)
  }
}

impl TextPattern for Char {
  findIn(_ source: Substring, _ offset: Int) -> Option<(Int, Int)> {
    self.findIn(source, offset)
  }
}

impl TextPattern for String {
  findIn(_ source: Substring, _ offset: Int) -> Option<(Int, Int)> {
    self.findIn(source, offset)
  }
}

impl TextPattern for Substring {
  findIn(_ source: Substring, _ offset: Int) -> Option<(Int, Int)> {
    self.findIn(source, offset)
  }
}

export TextPattern

