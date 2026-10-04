//! UTF-8 encoded string primitives, slicing, search, transformation, and sequence views.
//!
//! Phalcom strings are immutable byte arrays representing valid UTF-8 sequences.
//! This module provides string manipulation, searching, trimming, character iteration,
//! and Unicode scalar decoding.

from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

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
  internal native rawByteAt(_ index: Int) -> Int
  internal native rawSlice(_ start: Int, _ end: Int) -> String

  /// Returns the string itself as its string representation.
  toString { self }

  /// Length of the UTF-8 encoded string in bytes.
  size { rawByteCount }

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
    let b = rawByteAt(i)
    if b == None {
      return None
    }

    if b < 128 {
      return 1
    } else if b < 224 {
      return 2
    } else if b < 240 {
      return 3
    } else {
      return 4
    }
  }

  /// Decodes and returns the Unicode scalar value starting at byte offset `i`.
  ///
  /// Returns `None` if `i` is out of bounds or points to an invalid/continuation byte.
  ///
  /// @param i Byte offset into the string.
  /// @returns Decoded Unicode codepoint as an `Int`, or `None`.
  codePointAt(_ i) {
    let b0 = rawByteAt(i)
    if b0 == None {
      return None
    }

    if b0 < 128 {
      // Single-byte ASCII (0xxxxxxx)
      return b0
    } else if b0 < 192 {
      // Continuation byte (10xxxxxx), not a valid UTF-8 start byte
      return None
    } else if b0 < 224 {
      // 2-byte sequence (110xxxxx 10xxxxxx)
      let b1 = rawByteAt(i + 1)
      if b1 == None or b1 < 128 or b1 >= 192 {
        return None
      }
      return ((b0 - 192) * 64) + (b1 - 128)
    } else if b0 < 240 {
      // 3-byte sequence (1110xxxx 10xxxxxx 10xxxxxx)
      let b1 = rawByteAt(i + 1)
      let b2 = rawByteAt(i + 2)
      if b1 == None or b2 == None or b1 < 128 or b1 >= 192 or b2 < 128 or b2 >= 192 {
        return None
      }
      return ((b0 - 224) * 4096) + ((b1 - 128) * 64) + (b2 - 128)
    } else if b0 < 248 {
      // 4-byte sequence (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
      let b1 = rawByteAt(i + 1)
      let b2 = rawByteAt(i + 2)
      let b3 = rawByteAt(i + 3)
      if b1 == None or b2 == None or b3 == None or b1 < 128 or b1 >= 192 or b2 < 128 or b2 >= 192 or b3 < 128 or b3 >= 192 {
        return None
      }
      return ((b0 - 240) * 262144) + ((b1 - 128) * 4096) + ((b2 - 128) * 64) + (b3 - 128)
    } else {
      // Invalid start byte
      return None
    }
  }

  /// Finds the first occurrence of a substring, scanning from left to right.
  ///
  /// Returns the starting byte offset of the match, or `-1` if not found.
  ///
  /// @param needle The substring to find.
  /// @returns Byte index of the match or `-1`.
  indexOf(_ needle) {
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
  contains(_ needle) {
    if not needle.is(String) {
      throw ArgumentError.new("contains: needle must be a String")
    }
    if needle.isEmpty {
      return true
    }
    return indexOf(needle) != -1
  }

  /// Alias for `contains`.
  includes(_ needle) { contains(needle) }

  /// Splits the string around occurrences of `delimiter`.
  ///
  /// Returns a list of substrings separated by the delimiter.
  ///
  /// @param delimiter Non-empty string separator.
  /// @returns List of sliced string components.
  split(_ delimiter) {
    if not delimiter.is(String) {
      throw ArgumentError.new("split: delimiter must be a String")
    }
    if delimiter.isEmpty {
      throw ArgumentError.new("split: delimiter must be non-empty")
    }

    let mut result = List<Dynamic>.new()
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
  replace(_ needle, _ to) {
    if not needle.is(String) {
      throw ArgumentError.new("replace: from must be a String")
    }
    if not to.is(String) {
      throw ArgumentError.new("replace: to must be a String")
    }
    if needle.isEmpty {
      throw ArgumentError.new("replace: from must be non-empty")
    }

    let mut result = ""
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
  trim() {
    return trim(" \t\n\r")
  }

  /// Trims whitespace from the beginning of the string.
  trimStart() {
    return trimStart(" \t\n\r")
  }

  /// Trims whitespace from the end of the string.
  trimEnd() {
    return trimEnd(" \t\n\r")
  }

  /// Trims custom characters from both the start and end of the string.
  ///
  /// @param chars String containing characters to strip.
  /// @returns Trimmed string.
  trim(_ chars) { trimStart(chars).trimEnd(chars) }

  /// Trims custom characters from the start of the string.
  ///
  /// @param chars String containing characters to strip.
  /// @returns Trimmed string.
  trimStart(_ chars) {
    if not chars.is(String) {
      throw ArgumentError.new("trimStart: chars must be a String")
    }

    let mut i = 0
    let mut stop = false
    while i < rawByteCount and not stop {
      let cp = codePointAt(i)
      let mut found = false
      let mut j = 0
      while j < chars.rawByteCount {
        if chars.codePointAt(j) == cp {
          found = true
        }
        let len = chars.leadByteLen(j)
        if len == None {
          j = j + 1
        } else {
          j = j + len
        }
      }
      if found {
        i = i + leadByteLen(i)
      } else {
        stop = true // Exit loop at first non-trimmed character
      }
    }
    return rawSlice(i, rawByteCount)
  }

  /// Trims custom characters from the end of the string.
  ///
  /// @param chars String containing characters to strip.
  /// @returns Trimmed string.
  trimEnd(_ chars) {
    if not chars.is(String) {
      throw ArgumentError.new("trimEnd: chars must be a String")
    }

    let mut i = rawByteCount
    let mut stop = false
    while i > 0 and not stop {
      // Scan backward one byte at a time to find previous lead byte
      i = i - 1
      let mut cp = codePointAt(i)
      if cp != None {
        // Found a lead byte; check if it matches the trim set
        let mut found = false
        let mut j = 0
        while j < chars.rawByteCount {
          if chars.codePointAt(j) == cp {
            found = true
          }
          let len = chars.leadByteLen(j)
          if len == None {
            j = j + 1
          } else {
            j = j + len
          }
        }
        if not found {
          // Not in trim set; keep this character and terminate scanning
          i = i + leadByteLen(i)
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

  /// Returns a Unicode codepoint sequence view of this string.
  codePoints { StringCodePointSequence.new(self) }
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
    Some(value)
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
  size {
    let mut n = 0
    let mut i = nextCursor(None)
    while i != None {
      n = n + 1
      i = nextCursor(i)
    }
    return n
  }

  at(_ byteOffset) { _string.codePointAt(byteOffset) }

  internal byteCountView { _string.rawByteCount }

  internal rawCodePointAt(_ byteOffset) { _string.codePointAt(byteOffset) }

  internal rawLeadByteLen(_ byteOffset) { _string.leadByteLen(byteOffset) }

  each(_ f) {
    let mut i = nextCursor(None)
    while i != None {
      f.call(at(i))
      i = nextCursor(i)
    }
  }

  // Iterate over byte offsets: cursor steps by UTF-8 character boundary.
  private nextCursor(_ cursor) {
    let mut next = 0
    if cursor != None {
      next = cursor + _string.leadByteLen(cursor)
    }
    if next < _string.rawByteCount {
      return next
    } else {
      return None
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
    let mut value = _source.rawCodePointAt(_offset)
    if value == None {
      throw Error.new("String code-point iterator encountered an invalid sequence")
    }
    let mut length = _source.rawLeadByteLen(_offset)
    if length == None {
      throw Error.new("String code-point iterator encountered an invalid lead byte")
    }
    _offset = _offset + length
    Some(value)
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
