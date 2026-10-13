//! UTF-8 encoded string primitives, slicing, search, transformation, and sequence views.
//!
//! Phalcom strings are immutable byte arrays representing valid UTF-8 sequences.
//! This module provides string manipulation, searching, trimming, character iteration,
//! and Unicode scalar decoding.

from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable
from universe.scalar.string_index import StringIndex
from universe.text.errors import Utf8Error
from universe.text.substring import (
  Substring,
  Split,
  SplitWhitespace,
  Lines
)

/// Immutable UTF-8 text string.
///
/// Strings represent text sequences encoded as UTF-8. Slicing and indexing operations
/// are specified at byte granularity, while codepoint accessors provide decoded
/// Unicode scalar values.
native primitive class String is Object {
  native +(_ other: String) -> String
  native hash -> Int

  @class
  native new() -> String

  @class
  native new(_ value: Dynamic) -> String

  @class
  internal native rawJoin(_ list: List<Dynamic>) -> String

  /// Converts valid UTF-8 bytes into a String, returning Error(Utf8Error) if invalid.
  @class
  fromUtf8(_ source) -> Result<String, Utf8Error> {
    let b = if source.is(Bytes) { source } else { Bytes.fromList(source) }
    let s = b.rawUtf8
    if s == None {
      return Result::Error(Utf8Error.new(0, None))
    }
    Result::Ok(s)
  }

  /// Converts UTF-8 bytes lossily into a String, replacing invalid sequences with U+FFFD.
  @class
  fromUtf8Lossy(_ source) -> String {
    let b = if source.is(Bytes) { source } else { Bytes.fromList(source) }
    b.rawUtf8Lossy
  }

  internal native primitive rawByteCount -> Int
  internal native primitive rawByteAt(_ index: Int) -> Option<Int>
  internal native rawSlice(_ start: Int, _ end: Int) -> String
  internal native rawEquals(_ other: Object) -> Bool
  internal native rawFindSlice(_ needle: String, _ start: Int, _ end: Int) -> Option<Int>
  internal native rawRFindSlice(_ needle: String, _ start: Int, _ end: Int) -> Option<Int>
  native isCharBoundary(_ byteOffset: Int) -> Bool
  native decodeCharAt(_ byteOffset: Int) -> Option<Char>
  native lowercase() -> String
  native uppercase() -> String

  /// Equality comparison supporting String and Substring operands.
  ==(_ other: Dynamic) -> Bool {
    if other.is(Substring) {
      return other == self
    }
    rawEquals(other)
  }

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
    } else if index.is(Range) {
      let lower = index.rawLower
      let upper = index.rawUpper
      let len = rawByteCount
      let s = if lower.isSome {
        let v = lower.unwrapOr(0)
        if v.is(StringIndex) { v.offset } else { v }
      } else { 0 }
      let e = if upper.isSome {
        let v = upper.unwrapOr(len)
        let base = if v.is(StringIndex) { v.offset } else { v }
        if index.rawUpperInclusive {
          let opt = decodeCharAt(base)
          if opt.isSome {
            base + opt.unwrap.utf8Length
          } else {
            base + 1
          }
        } else {
          base
        }
      } else { len }
      return Substring.new(self, s, e)
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
  isEmpty -> Bool { rawByteCount == 0 }

  /// Extracts a non-allocating Substring view by half-open byte or index range `[start, end)`.
  slice(_ start, _ end) -> Substring {
    let s = if start.is(StringIndex) { start.offset } else { start }
    let e = if end.is(StringIndex) { end.offset } else { end }
    Substring.new(self, s, e)
  }

  /// Produces a zero-copy Substring view spanning the entire string.
  asSubstring -> Substring {
    Substring.new(self, 0, rawByteCount)
  }

  /// Searches for the first occurrence of this string in `source` from the beginning.
  findIn(_ source: Substring) -> Option<(Int, Int)> {
    self.findIn(source, 0)
  }

  /// Searches for the first occurrence of this string in `source` at or after byte offset `offset`.
  findIn(_ source: Substring, _ offset: Int) -> Option<(Int, Int)> {
    let patLen = rawByteCount
    let srcLen = source.byteCount
    if patLen == 0 {
      if offset >= 0 and offset <= srcLen {
        return Some((offset, offset))
      }
      return None
    }
    if offset < 0 or offset + patLen > srcLen {
      return None
    }
    let optOffset = source.string.rawFindSlice(self, source.startOffset + offset, source.endOffset)
    match optOffset {
      Option::Some(matchRel) => {
        let start = offset + matchRel
        Some((start, start + patLen))
      }
      _ => None
    }
  }

  /// Searches for the first occurrence of this string in `source` at or after byte offset `from`.
  findIn(_ source: Substring, from offset: Int) -> Option<(Int, Int)> {
    self.findIn(source, offset)
  }

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

  /// Checks whether this string contains the given pattern.
  contains(_ pattern) -> Bool {
    if pattern.is(Char) or pattern.is(String) or pattern.is(Substring) {
      return pattern.findIn(asSubstring, 0).isSome
    }
    let opt = pattern.findIn(asSubstring, 0)
    match opt {
      Option::Some(_) => true
      _ => false
    }
  }

  /// Alias for `contains`.
  includes(_ needle) -> Bool { contains(needle) }

  /// Determines whether the string begins with the given prefix pattern.
  startsWith(_ prefix) -> Bool {
    if prefix.is(Char) {
      if first.isSome {
        return first.unwrap == prefix
      }
      return false
    }
    if prefix.is(String) {
      let pLen = prefix.rawByteCount
      if pLen > rawByteCount { return false }
      if pLen == 0 { return true }
      let opt = prefix.findIn(asSubstring, 0)
      if opt.isSome {
        return opt.unwrap.rawAt(0) == 0
      }
      return false
    }
    if prefix.is(Substring) {
      let pLen = prefix.byteCount
      if pLen > rawByteCount { return false }
      if pLen == 0 { return true }
      let opt = prefix.findIn(asSubstring, 0)
      if opt.isSome {
        return opt.unwrap.rawAt(0) == 0
      }
      return false
    }
    let opt = prefix.findIn(asSubstring, 0)
    if opt.isSome {
      return opt.unwrap.rawAt(0) == 0
    }
    return false
  }

  /// Determines whether the string terminates with the given suffix pattern.
  endsWith(_ suffix) -> Bool {
    if suffix.is(Char) {
      if last.isSome {
        return last.unwrap == suffix
      }
      return false
    }
    if suffix.is(String) {
      let sLen = suffix.rawByteCount
      if sLen > rawByteCount { return false }
      if sLen == 0 { return true }
      let fromOffset = rawByteCount - sLen
      let opt = suffix.findIn(asSubstring, fromOffset)
      if opt.isSome {
        return opt.unwrap.rawAt(1) == rawByteCount
      }
      return false
    }
    if suffix.is(Substring) {
      let sLen = suffix.byteCount
      if sLen > rawByteCount { return false }
      if sLen == 0 { return true }
      let fromOffset = rawByteCount - sLen
      let opt = suffix.findIn(asSubstring, fromOffset)
      if opt.isSome {
        return opt.unwrap.rawAt(1) == rawByteCount
      }
      return false
    }
    return false
  }

  /// Searches for the first occurrence of `pattern`, returning its start `StringIndex`.
  find(_ pattern) -> Option<StringIndex> {
    let opt = pattern.findIn(asSubstring, 0)
    if opt.isSome {
      return Some(StringIndex(opt.unwrap.rawAt(0)))
    }
    return None
  }

  /// Searches for the last occurrence of `pattern`, returning its start `StringIndex`.
  findLast(_ pattern) -> Option<StringIndex> {
    let needle = if pattern.is(Char) {
      pattern.toString
    } else if pattern.is(String) {
      pattern
    } else if pattern.is(Substring) {
      pattern.toString
    } else {
      throw ArgumentError.new("findLast: unsupported pattern type")
    }
    if needle.rawByteCount == 0 {
      return Some(StringIndex(rawByteCount))
    }
    let optRel = rawRFindSlice(needle, 0, rawByteCount)
    if optRel.isSome {
      return Some(StringIndex(optRel.unwrap))
    }
    return None
  }

  /// Searches for the first occurrence of `pattern`, returning the matching byte range.
  findRange(_ pattern) -> Option<Range> {
    let opt = pattern.findIn(asSubstring, 0)
    if opt.isSome {
      let pair = opt.unwrap
      let s = pair.rawAt(0)
      let e = pair.rawAt(1)
      return Some(s..e)
    }
    return None
  }

  /// Returns a Substring view with leading Unicode whitespace removed.
  trimmedStart() -> Substring {
    let mut offset = 0
    let total = rawByteCount
    while offset < total {
      let optCh = decodeCharAt(offset)
      match optCh {
        Option::Some(ch) => {
          if not ch.isWhitespace {
            return slice(offset, total)
          }
          offset = offset + ch.utf8Length
        }
        _ => {
          return slice(offset, total)
        }
      }
    }
    slice(total, total)
  }

  /// Returns a Substring view with trailing Unicode whitespace removed.
  trimmedEnd() -> Substring {
    let mut end = rawByteCount
    while end > 0 {
      let mut i = end - 1
      while i > 0 and not isCharBoundary(i) {
        i = i - 1
      }
      let optCh = decodeCharAt(i)
      match optCh {
        Option::Some(ch) => {
          if not ch.isWhitespace {
            return slice(0, end)
          }
          end = i
        }
        _ => {
          return slice(0, end)
        }
      }
    }
    slice(0, 0)
  }

  /// Returns a Substring view with leading and trailing Unicode whitespace removed.
  trimmed() -> Substring {
    let mut s = 0
    let total = rawByteCount
    let mut hasNonWs = false
    while s < total {
      let optCh = decodeCharAt(s)
      match optCh {
        Option::Some(ch) => {
          if not ch.isWhitespace {
            hasNonWs = true
            break
          }
          s = s + ch.utf8Length
        }
        _ => {
          break
        }
      }
    }
    if not hasNonWs {
      return slice(total, total)
    }
    let mut e = total
    while e > s {
      let mut i = e - 1
      while i > s and not isCharBoundary(i) {
        i = i - 1
      }
      let optCh = decodeCharAt(i)
      match optCh {
        Option::Some(ch) => {
          if not ch.isWhitespace {
            break
          }
          e = i
        }
        _ => {
          break
        }
      }
    }
    slice(s, e)
  }

  /// If the string starts with `prefix`, returns the Substring after the prefix; otherwise None.
  stripPrefix(_ prefix) -> Option<Substring> {
    if not startsWith(prefix) {
      return None
    }
    let len = if prefix.is(Char) {
      prefix.utf8Length
    } else if prefix.is(String) {
      prefix.rawByteCount
    } else if prefix.is(Substring) {
      prefix.byteCount
    } else {
      return None
    }
    Some(slice(len, rawByteCount))
  }

  /// If the string ends with `suffix`, returns the Substring before the suffix; otherwise None.
  stripSuffix(_ suffix) -> Option<Substring> {
    if not endsWith(suffix) {
      return None
    }
    let len = if suffix.is(Char) {
      suffix.utf8Length
    } else if suffix.is(String) {
      suffix.rawByteCount
    } else if suffix.is(Substring) {
      suffix.byteCount
    } else {
      return None
    }
    Some(slice(0, rawByteCount - len))
  }

  /// Splits this string around matches of `pattern` into a lazy sequence of Substrings.
  split(_ pattern) -> Split {
    Split.new(asSubstring, pattern)
  }

  /// Splits this string around the first match of `separator` into a pair of Substrings.
  splitOnce(_ separator) -> Option<(Substring, Substring)> {
    let opt = separator.findIn(asSubstring, 0)
    if opt.isSome {
      let pair = opt.unwrap
      let s = pair.rawAt(0)
      let e = pair.rawAt(1)
      let left = slice(0, s)
      let right = slice(e, rawByteCount)
      return Some((left, right))
    }
    return None
  }

  /// Splits this string into words separated by Unicode whitespace.
  splitWhitespace() -> SplitWhitespace {
    SplitWhitespace.new(asSubstring)
  }

  /// Yields a lazy sequence of lines separated by newline terminators.
  lines() -> Lines {
    Lines.new(asSubstring, false)
  }

  /// Yields a lazy sequence of lines, optionally retaining terminators.
  lines(keepingTerminators keep: Bool) -> Lines {
    Lines.new(asSubstring, keep)
  }

  /// Replaces the first occurrence of `pattern` with `replacement`.
  replaceFirst(_ pattern, with replacement: String) -> String {
    let opt = pattern.findIn(asSubstring, 0)
    if opt.isSome {
      let pair = opt.unwrap
      let s = pair.rawAt(0)
      let e = pair.rawAt(1)
      return slice(0, s).toString + replacement + slice(e, rawByteCount).toString
    }
    return self
  }

  /// Replaces all occurrences of `pattern` with `replacement`.
  replaceAll(_ pattern, with replacement: String) -> String {
    let mut out = ""
    let mut cursor = 0
    let total = rawByteCount
    while cursor <= total {
      let opt = pattern.findIn(asSubstring, cursor)
      if opt.isSome {
        let pair = opt.unwrap
        let s = pair.rawAt(0)
        let e = pair.rawAt(1)
        if s == e {
          if cursor < total {
            out = out + slice(cursor, cursor + 1).toString + replacement
            cursor = cursor + 1
          } else {
            out = out + replacement
            break
          }
        } else {
          out = out + slice(cursor, s).toString + replacement
          cursor = e
        }
      } else {
        out = out + slice(cursor, total).toString
        break
      }
    }
    return out
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

  /// Returns an iterable byte-level sequence view of this string as UTF-8 bytes.
  utf8 -> StringByteSequence { StringByteSequence.new(self) }

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

export (
  StringChars,
  StringCharIterator,
  StringByteSequence,
  StringByteIterator,
  StringCodePointSequence,
  StringCodePointIterator
)
