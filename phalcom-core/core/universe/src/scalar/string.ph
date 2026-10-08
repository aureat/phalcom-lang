//! UTF-8 encoded string primitives, slicing, search, transformation, and sequence views.
//!
//! Phalcom strings are immutable byte arrays representing valid UTF-8 sequences.
//! This module provides string manipulation, searching, trimming, character iteration,
//! and Unicode scalar decoding.

from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable
from universe.scalar.string_index import StringIndex

/// Immutable UTF-8 text string.
///
/// Strings represent text sequences encoded as UTF-8. Slicing and indexing operations
/// are specified at byte granularity, while codepoint accessors provide decoded
/// Unicode scalar values.
native class String is Object {
  native +(_ other: String) -> String
  native hash -> Int

  @class
  native new() -> String

  @class
  native new(_ value: Dynamic) -> String

  internal native rawByteCount -> Int
  internal native rawByteAt(_ index: Int) -> Option<Int>
  internal native rawSlice(_ start: Int, _ end: Int) -> String
  native isCharBoundary(_ byteOffset: Int) -> Bool
  native decodeCharAt(_ byteOffset: Int) -> Option<Char>
  native lowercase() -> String
  native uppercase() -> String

  /// Returns the string itself as its string representation.
  toString { self }

  /// Length of the UTF-8 encoded string in bytes.
  size { rawByteCount }

  /// Length of the UTF-8 encoded string in bytes (O(1)).
  byteCount -> Int { rawByteCount }

  /// Number of Unicode scalar values in the string (O(n)).
  charCount -> Int {
    let mut count = 0
    let mut it = chars.iterator
    while let Option::Some(_) = it.next {
      count = count + 1
    }
    return count
  }

  /// Logical start position of the string.
  startIndex -> StringIndex { StringIndex(0) }

  /// Logical end position of the string.
  endIndex -> StringIndex { StringIndex(rawByteCount) }

  /// The first Unicode scalar character in the string, or None if empty.
  first -> Option<Char> {
    if rawByteCount == 0 { return None }
    decodeCharAt(0)
  }

  /// The last Unicode scalar character in the string, or None if empty.
  last -> Option<Char> {
    if rawByteCount == 0 { return None }
    let mut i = rawByteCount - 1
    while i > 0 and not isCharBoundary(i) {
      i = i - 1
    }
    decodeCharAt(i)
  }

  /// Advances a string index to the next Unicode scalar boundary.
  index(after index: StringIndex) -> Option<StringIndex> {
    if not index.is(StringIndex) {
      throw ArgumentError.new("index(after:): index must be a StringIndex")
    }
    let offset = index.offset
    if offset < 0 or offset >= rawByteCount {
      return None
    }
    match decodeCharAt(offset) {
      Option::Some(ch) => Some(StringIndex(offset + ch.utf8Length))
      _ => None
    }
  }

  /// Retreats a string index to the previous Unicode scalar boundary.
  index(before index: StringIndex) -> Option<StringIndex> {
    if not index.is(StringIndex) {
      throw ArgumentError.new("index(before:): index must be a StringIndex")
    }
    let offset = index.offset
    if offset <= 0 or offset > rawByteCount {
      return None
    }
    let mut prev = offset - 1
    while prev > 0 and not isCharBoundary(prev) {
      prev = prev - 1
    }
    Some(StringIndex(prev))
  }

  /// Subscript operator for String.
  [_ index] {
    if index.is(StringIndex) {
      let offset = index.offset
      if offset < 0 or offset >= rawByteCount {
        throw IndexError.new("StringIndex out of range")
      }
      let opt = decodeCharAt(offset)
      if opt.isSome {
        return opt.unwrap
      }
      throw IndexError.new("StringIndex is not a valid character boundary")
    } else {
      throw ArgumentError.new("String subscript requires StringIndex or Range")
    }
  }

  /// Reports whether this string contains only ASCII characters.
  isAscii -> Bool {
    let mut i = 0
    let len = rawByteCount
    while i < len {
      match rawByteAt(i) {
        Option::Some(b) => {
          if b >= 128 { return false }
        }
        _ => { return false }
      }
      i = i + 1
    }
    return true
  }

  /// Repeats this string `count` times.
  repeat(_ count: Int) -> String {
    self * count
  }

  /// Pads the start of this string with `with` character up to `width`.
  padStart(_ width: Int) -> String {
    padStart(width, with: ' ')
  }

  padStart(_ width: Int, with fill) -> String {
    let currentLen = charCount
    if currentLen >= width { return self }
    let fillChar = if fill.is(Char) { fill.toString } else { fill.toString }
    let padCount = width - currentLen
    let mut pad = ""
    let mut i = 0
    while i < padCount {
      pad = pad + fillChar
      i = i + 1
    }
    return pad + self
  }

  /// Pads the end of this string with `with` character up to `width`.
  padEnd(_ width: Int) -> String {
    padEnd(width, with: ' ')
  }

  padEnd(_ width: Int, with fill) -> String {
    let currentLen = charCount
    if currentLen >= width { return self }
    let fillChar = if fill.is(Char) { fill.toString } else { fill.toString }
    let padCount = width - currentLen
    let mut pad = ""
    let mut i = 0
    while i < padCount {
      pad = pad + fillChar
      i = i + 1
    }
    return self + pad
  }

  /// Canonical Telos source representation of this string literal.
  repr -> String {
    let mut out = "\""
    let mut i = 0
    let len = rawByteCount
    while i < len {
      match rawByteAt(i) {
        Option::Some(b) => {
          if b == 34 {
            out = out + "\\\""
          } else if b == 92 {
            out = out + "\\\\"
          } else if b == 10 {
            out = out + "\\n"
          } else if b == 13 {
            out = out + "\\r"
          } else if b == 9 {
            out = out + "\\t"
          } else if b == 0 {
            out = out + "\\0"
          } else if isCharBoundary(i) {
            match decodeCharAt(i) {
              Option::Some(ch) => {
                let code = ch.codePoint
                if code < 32 {
                  out = out + "\\u{" + code.toString + "}"
                } else {
                  out = out + ch.toString
                }
              }
              _ => {
                out = out + rawSlice(i, i + 1)
              }
            }
          }
        }
        _ => {}
      }
      match decodeCharAt(i) {
        Option::Some(ch) => { i = i + ch.utf8Length }
        _ => { i = i + 1 }
      }
    }
    out = out + "\""
    return out
  }

  /// Indicates whether the string contains zero bytes.
  isEmpty { rawByteCount == 0 }

  /// Extracts a substring by half-open byte range `[start, end)`.
  ///
  /// @param start Starting byte offset (inclusive).
  /// @param end Ending byte offset (exclusive).
  /// @returns Sliced substring.
  slice(_ start, _ end) { rawSlice(start, end) }

  /// Determines the number of bytes in the UTF-8 code point starting at byte offset `i`.
  ///
  /// Returns `1`, `2`, `3`, or `4` depending on the lead byte range, or `None` if `i` is out of bounds.
  ///
  /// @param i Byte offset into the string.
  /// @returns Number of bytes (1-4) or `None`.
  leadByteLen(_ i) {
    match decodeCharAt(i) {
      Option::Some(c) => c.utf8Length
      _ => None
    }
  }

  /// Decodes and returns the Unicode scalar value starting at byte offset `i`.
  ///
  /// Returns `None` if `i` is out of bounds or points to an invalid/continuation byte.
  ///
  /// @param i Byte offset into the string.
  /// @returns Decoded Unicode codepoint as an `Int`, or `None`.
  codePointAt(_ i) {
    match decodeCharAt(i) {
      Option::Some(c) => Some(c.codePoint)
      _ => None
    }
  }

  /// Finds the first occurrence of a substring, scanning from left to right.
  ///
  /// Returns the starting byte offset of the match, or `-1` if not found.
  ///
  /// @param needle The substring to find.
  /// @returns Byte index of the match or `-1`.
  indexOf(_ needle) -> Int {
    if not needle.is(String) {
      throw ArgumentError.new("indexOf: needle must be a String")
    }
    if needle.isEmpty {
      throw ArgumentError.new("indexOf: needle must be non-empty")
    }

    let mut i = 0
    while i <= rawByteCount - needle.rawByteCount {
      let mut is_match = true
      let mut j = 0
      while j < needle.rawByteCount {
        if rawByteAt(i + j) != needle.rawByteAt(j) {
          is_match = false
        }
        if is_match {
          j = j + 1
        } else {
          j = needle.rawByteCount
        }
      }
      if is_match {
        return i
      }
      i = i + 1
    }
    return -1
  }

  /// Checks whether this string contains the given substring.
  ///
  /// @param needle The substring to search for.
  /// @returns `true` if `needle` is found; otherwise `false`.
  contains(_ needle) -> Bool {
    if not needle.is(String) {
      throw ArgumentError.new("contains: needle must be a String")
    }
    if needle.isEmpty {
      return true
    }
    return indexOf(needle) != -1
  }

  /// Alias for `contains`.
  includes(_ needle) -> Bool { contains(needle) }

  /// Splits the string around occurrences of `delimiter`.
  ///
  /// Returns a list of substrings separated by the delimiter.
  ///
  /// @param delimiter Non-empty string separator.
  /// @returns List of sliced string components.
  split(_ delimiter) -> List<String> {
    if not delimiter.is(String) {
      throw ArgumentError.new("split: delimiter must be a String")
    }
    if delimiter.isEmpty {
      throw ArgumentError.new("split: delimiter must be non-empty")
    }

    let mut result = List<String>.new()
    let mut prev = 0
    let mut i = indexOf(delimiter)
    while i != -1 {
      result.rawPush(rawSlice(prev, i))
      prev = i + delimiter.rawByteCount

      // Search for next occurrence after this delimiter
      let mut rest = rawSlice(prev, rawByteCount)
      let mut nextIdx = rest.indexOf(delimiter)
      if nextIdx == -1 {
        i = -1
      } else {
        i = prev + nextIdx
      }
    }
    result.rawPush(rawSlice(prev, rawByteCount))
    return result
  }

  /// Replaces all occurrences of `needle` with `to`.
  ///
  /// @param needle Non-empty substring to be replaced.
  /// @param to Substring replacement.
  /// @returns Transformed string.
  replace(_ needle, _ to) -> String {
    if not needle.is(String) {
      throw ArgumentError.new("replace: from must be a String")
    }
    if not to.is(String) {
      throw ArgumentError.new("replace: to must be a String")
    }
    if needle.isEmpty {
      throw ArgumentError.new("replace: from must be non-empty")
    }

    let mut result: String = ""
    let mut prev = 0
    let mut i = indexOf(needle)
    while i != -1 {
      result = result + rawSlice(prev, i) + to
      prev = i + needle.rawByteCount
      let mut rest = rawSlice(prev, rawByteCount)
      let mut nextIdx = rest.indexOf(needle)
      if nextIdx == -1 {
        i = -1
      } else {
        i = prev + nextIdx
      }
    }
    result = result + rawSlice(prev, rawByteCount)
    return result
  }

  /// Trims whitespace from both the beginning and end of the string.
  trim() -> String {
    return trim(" \t\n\r")
  }

  /// Trims whitespace from the beginning of the string.
  trimStart() -> String {
    return trimStart(" \t\n\r")
  }

  /// Trims whitespace from the end of the string.
  trimEnd() -> String {
    return trimEnd(" \t\n\r")
  }

  /// Trims custom characters from both the start and end of the string.
  ///
  /// @param chars String containing characters to strip.
  /// @returns Trimmed string.
  trim(_ chars) -> String { trimStart(chars).trimEnd(chars) }

  /// Trims custom characters from the start of the string.
  ///
  /// @param chars String containing characters to strip.
  /// @returns Trimmed string.
  trimStart(_ chars) -> String {
    if not chars.is(String) {
      throw ArgumentError.new("trimStart: chars must be a String")
    }

    let mut i = 0
    let mut stop = false
    while i < rawByteCount and not stop {
      match decodeCharAt(i) {
        Option::Some(c) => {
          if chars.contains(c.toString) {
            i = i + c.utf8Length
          } else {
            stop = true
          }
        }
        _ => {
          stop = true
        }
      }
    }
    return rawSlice(i, rawByteCount)
  }

  /// Trims custom characters from the end of the string.
  ///
  /// @param chars String containing characters to strip.
  /// @returns Trimmed string.
  trimEnd(_ chars) -> String {
    if not chars.is(String) {
      throw ArgumentError.new("trimEnd: chars must be a String")
    }

    let mut i = rawByteCount
    let mut stop = false
    while i > 0 and not stop {
      let mut prev = i - 1
      while prev > 0 and not isCharBoundary(prev) {
        prev = prev - 1
      }
      match decodeCharAt(prev) {
        Option::Some(c) => {
          if chars.contains(c.toString) {
            i = prev
          } else {
            stop = true
          }
        }
        _ => {
          stop = true
        }
      }
    }
    return rawSlice(0, i)
  }

  /// Repeats the string `count` times.
  ///
  /// @param count Non-negative integer repetition count.
  /// @returns Repeated string.
  *(_ count) {
    if not count.is(Number) {
      throw ArgumentError.new("*: count must be a Number")
    }
    if count < 0 {
      throw ArgumentError.new("*: count must be >= 0")
    }
    if count % 1 != 0 {
      throw ArgumentError.new("*: count must be an integer")
    }

    if count == 0 { return "" }
    if count == 1 { return self }

    let mut result = ""
    let mut i = 0
    while i < count {
      result = result + self
      i = i + 1
    }
    return result
  }

  /// Returns a byte-level sequence view of this string.
  bytes { StringByteSequence.new(self) }

  /// Returns a character sequence view yielding `Char` Unicode scalar values.
  chars -> StringChars { StringChars.new(self) }

  /// Returns a Unicode codepoint sequence view of this string.
  codePoints { StringCodePointSequence.new(self) }
}

/// Character sequence view over a UTF-8 string.
class StringChars {
  _string: String

  @constructor
  new(_ s: String) { _string = s }

  rawByteCount -> Int { _string.rawByteCount }

  decodeCharAt(_ byteOffset: Int) -> Option<Char> { _string.decodeCharAt(byteOffset) }

  iterator -> StringCharIterator { StringCharIterator.new(self) }

  each(_ f) {
    let mut it = iterator
    while let Option::Some(ch) = it.next {
      f.call(ch)
    }
  }
}

/// Iterator over decoded Unicode scalar `Char` values of a string.
class StringCharIterator {
  _source: StringChars
  mut _byteOffset: Int
  mut _done: Bool

  @constructor
  new(_ source: StringChars) {
    _source = source
    _byteOffset = 0
    _done = false
  }

  next -> Option<Char> {
    if _done { return None }
    if _byteOffset >= _source.rawByteCount {
      _done = true
      return None
    }
    match _source.decodeCharAt(_byteOffset) {
      Option::Some(ch) => {
        _byteOffset = _byteOffset + ch.utf8Length
        Some(ch)
      }
      _ => {
        _done = true
        None
      }
    }
  }

  iterator { self }
}

impl CanonicalIterator for StringCharIterator {
  type Item = Char
}

impl CanonicalIterable for StringChars {
  type Item = Char
  type Iterator = StringCharIterator
  iterator -> StringCharIterator { StringCharIterator.new(self) }
}

/// Byte-level sequence view over a UTF-8 string.
class StringByteSequence {
  mut _string
  @constructor
  new(_ s) { _string = s }

  iterator -> StringByteIterator { StringByteIterator.new(self) }

  size { _string.rawByteCount }

  at(_ i) { _string.rawByteAt(i) }

  internal rawByteAt(_ i) { _string.rawByteAt(i) }

  each(_ f) {
    let mut i = 0
    while i < size {
      f.call(at(i))
      i = i + 1
    }
  }

  // Iterate over byte offsets: cursor steps to next lead byte.
  private nextCursor(_ cursor) {
    let mut next = 0
    if cursor != None {
      next = cursor + 1
    }
    if next < _string.rawByteCount {
      return next
    } else {
      return None
    }
  }
}

/// Iterator over raw byte values of a string.
class StringByteIterator {
  mut _source: StringByteSequence
  mut _offset: Int
  mut _done: Bool

  @constructor
  new(_ source: StringByteSequence) {
    _source = source
    _offset = 0
    _done = false
  }

  next -> Option<Int> {
    if _done { return None }
    if _offset >= _source.size {
      _done = true
      return None
    }
    let mut value = _source.rawByteAt(_offset)
    _offset = _offset + 1
    value
  }

  iterator { self }
}

impl CanonicalIterator for StringByteIterator {
  type Item = Int
}

impl CanonicalIterable for StringByteSequence {
  type Item = Int
  type Iterator = StringByteIterator
  iterator -> StringByteIterator { StringByteIterator.new(self) }
}

/// Unicode codepoint sequence view over a UTF-8 string.
class StringCodePointSequence {
  mut _string
  @constructor
  new(_ s) { _string = s }

  iterator -> StringCodePointIterator { StringCodePointIterator.new(self) }

  /// Number of Unicode code points in the string.
  size -> Int {
    let mut n = 0
    let mut it = _string.chars.iterator
    while let Option::Some(_) = it.next {
      n = n + 1
    }
    return n
  }

  at(_ ordinal: Int) -> Option<Int> {
    if ordinal < 0 {
      return None
    }
    let mut it = _string.chars.iterator
    let mut idx = 0
    while let Option::Some(ch) = it.next {
      if idx == ordinal {
        return Some(ch.codePoint)
      }
      idx = idx + 1
    }
    return None
  }

  internal byteCountView -> Int { _string.rawByteCount }

  decodeCharAt(_ byteOffset: Int) -> Option<Char> { _string.decodeCharAt(byteOffset) }

  internal rawCodePointAt(_ byteOffset) { _string.codePointAt(byteOffset) }

  internal rawLeadByteLen(_ byteOffset) { _string.leadByteLen(byteOffset) }

  each(_ f) {
    let mut it = _string.chars.iterator
    while let Option::Some(ch) = it.next {
      f.call(ch.codePoint)
    }
  }
}

/// Iterator over decoded Unicode scalar values of a string.
class StringCodePointIterator {
  mut _source: StringCodePointSequence
  mut _offset: Int
  mut _done: Bool

  @constructor
  new(_ source: StringCodePointSequence) {
    _source = source
    _offset = 0
    _done = false
  }

  next -> Option<Int> {
    if _done { return None }
    if _offset >= _source.byteCountView {
      _done = true
      return None
    }
    match _source.decodeCharAt(_offset) {
      Option::Some(ch) => {
        _offset = _offset + ch.utf8Length
        Some(ch.codePoint)
      }
      _ => {
        _done = true
        None
      }
    }
  }

  iterator { self }
}

impl CanonicalIterator for StringCodePointIterator {
  type Item = Int
}

impl CanonicalIterable for StringCodePointSequence {
  type Item = Int
  type Iterator = StringCodePointIterator
  iterator -> StringCodePointIterator { StringCodePointIterator.new(self) }
}
