from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable
from universe.scalar.string_index import StringIndex

/// An immutable, non-allocating contiguous text view over a String.
class Substring {
  _string: String
  _start: Int
  _end: Int

  @constructor
  new(_ string: String, _ start: Int, _ end: Int) {
    if not string.is(String) {
      throw ArgumentError.new("Substring: source must be a String")
    }
    if not start.is(Number) or not end.is(Number) {
      throw ArgumentError.new("Substring: start and end must be integers")
    }
    let total = string.rawByteCount
    if start < 0 or end < start or end > total {
      throw SliceError.new("Substring: slice out of bounds")
    }
    if not string.isCharBoundary(start) or not string.isCharBoundary(end) {
      throw SliceError.new("Substring: boundaries must fall on character boundaries")
    }
    _string = string
    _start = start
    _end = end
  }

  @class
  call(_ string: String, _ start: Int, _ end: Int) -> Substring {
    Substring.new(string, start, end)
  }

  /// The underlying string storage.
  string -> String { _string }

  /// The starting byte offset within the underlying string.
  startOffset -> Int { _start }

  /// The ending byte offset within the underlying string.
  endOffset -> Int { _end }

  /// The byte length of this substring view.
  byteCount -> Int { _end - _start }

  /// Indicates whether the substring contains zero bytes.
  isEmpty -> Bool { _start == _end }

  /// The number of Unicode scalars contained in this substring (O(n)).
  charCount -> Int {
    let mut count = 0
    let mut it = chars.iterator
    while let Option::Some(_) = it.next {
      count = count + 1
    }
    return count
  }

  /// Logical start position of the substring.
  startIndex -> StringIndex { StringIndex(0) }

  /// Logical end position of the substring.
  endIndex -> StringIndex { StringIndex(_end - _start) }

  /// The first Unicode scalar character in the substring, or None if empty.
  first -> Option<Char> {
    if _start == _end { return None }
    _string.decodeCharAt(_start)
  }

  /// The last Unicode scalar character in the substring, or None if empty.
  last -> Option<Char> {
    if _start == _end { return None }
    let mut i = _end - 1
    while i > _start and not _string.isCharBoundary(i) {
      i = i - 1
    }
    _string.decodeCharAt(i)
  }

  /// Advances a relative substring index to the next scalar boundary.
  index(after index: StringIndex) -> Option<StringIndex> {
    if not index.is(StringIndex) {
      throw ArgumentError.new("index(after:): index must be a StringIndex")
    }
    let rel = index.offset
    let abs = _start + rel
    if rel < 0 or abs >= _end {
      return None
    }
    match _string.decodeCharAt(abs) {
      Option::Some(ch) => Some(StringIndex(rel + ch.utf8Length))
      _ => None
    }
  }

  /// Retreats a relative substring index to the previous scalar boundary.
  index(before index: StringIndex) -> Option<StringIndex> {
    if not index.is(StringIndex) {
      throw ArgumentError.new("index(before:): index must be a StringIndex")
    }
    let rel = index.offset
    let abs = _start + rel
    if rel <= 0 or abs > _end {
      return None
    }
    let mut prev = abs - 1
    while prev > _start and not _string.isCharBoundary(prev) {
      prev = prev - 1
    }
    Some(StringIndex(prev - _start))
  }

  /// Reads the raw byte at relative offset `index`.
  rawByteAt(_ index: Int) -> Option<Int> {
    if index < 0 or index >= byteCount {
      return None
    }
    _string.rawByteAt(_start + index)
  }

  /// Decodes and returns the Unicode scalar value at relative byte offset `offset`.
  decodeCharAt(_ offset: Int) -> Option<Char> {
    if offset < 0 or _start + offset >= _end {
      return None
    }
    _string.decodeCharAt(_start + offset)
  }

  /// Determines whether the relative byte offset is a UTF-8 character boundary.
  isCharBoundary(_ offset: Int) -> Bool {
    if offset < 0 or offset > byteCount {
      return false
    }
    _string.isCharBoundary(_start + offset)
  }

  /// Returns this substring view.
  asSubstring -> Substring { self }

  /// Searches for the first occurrence of this substring in `source` from the beginning.
  findIn(_ source: Substring) -> Option<(Int, Int)> {
    self.findIn(source, 0)
  }

  /// Searches for the first occurrence of this substring in `source` at or after byte offset `offset`.
  findIn(_ source: Substring, _ offset: Int) -> Option<(Int, Int)> {
    self.toString.findIn(source, offset)
  }

  /// Searches for the first occurrence of this substring in `source` at or after byte offset `from`.
  findIn(_ source: Substring, from offset: Int) -> Option<(Int, Int)> {
    self.findIn(source, offset)
  }

  /// Subscript operator for Substring.
  [_ index] {
    if index.is(StringIndex) {
      let rel = index.offset
      let abs = _start + rel
      if rel < 0 or abs >= _end {
        throw IndexError.new("Substring index out of bounds")
      }
      let opt = _string.decodeCharAt(abs)
      if opt.isSome {
        return opt.unwrap
      }
      throw IndexError.new("Substring index is not a valid character boundary")
    } else if index.is(Range) {
      return self.sliceByRange(index)
    } else {
      throw ArgumentError.new("Substring subscript requires StringIndex or Range")
    }
  }

  /// Slices a sub-view without allocating new string storage.
  slice(_ start, _ end) -> Substring {
    let s = if start.is(StringIndex) { start.offset } else { start }
    let e = if end.is(StringIndex) { end.offset } else { end }
    if not s.is(Number) or not e.is(Number) {
      throw ArgumentError.new("slice: start and end must be integers or StringIndex")
    }
    let len = byteCount
    if s < 0 or e < s or e > len {
      throw SliceError.new("slice out of bounds")
    }
    Substring.new(_string, _start + s, _start + e)
  }

  private sliceByRange(_ range) -> Substring {
    let lower = range.rawLower
    let upper = range.rawUpper
    let len = byteCount
    let s = if lower.isSome {
      let v = lower.unwrapOr(0)
      if v.is(StringIndex) { v.offset } else { v }
    } else { 0 }
    let e = if upper.isSome {
      let v = upper.unwrapOr(len)
      let base = if v.is(StringIndex) { v.offset } else { v }
      if range.rawUpperInclusive {
        let abs = _start + base
        match _string.decodeCharAt(abs) {
          Option::Some(ch) => base + ch.utf8Length
          _ => base + 1
        }
      } else {
        base
      }
    } else { len }
    slice(s, e)
  }

  /// Materializes the substring into a newly owned String.
  toString -> String {
    _string.rawSlice(_start, _end)
  }

  /// Canonical Telos source representation of this substring.
  repr -> String {
    toString.repr
  }

  /// Returns character sequence view yielding `Char` values.
  chars -> SubstringChars {
    SubstringChars.new(self)
  }

  /// Returns byte sequence view yielding raw integer byte values.
  utf8 -> SubstringBytes {
    SubstringBytes.new(self)
  }

  ==(_ other: Dynamic) -> Bool {
    if other.is(Substring) {
      if self.byteCount != other.byteCount { return false }
      let len = self.byteCount
      let mut i = 0
      while i < len {
        if self.rawByteAt(i) != other.rawByteAt(i) { return false }
        i = i + 1
      }
      return true
    } else if other.is(String) {
      if self.byteCount != other.rawByteCount { return false }
      let len = self.byteCount
      let mut i = 0
      while i < len {
        if self.rawByteAt(i) != other.rawByteAt(i) { return false }
        i = i + 1
      }
      return true
    }
    return false
  }

  compare(_ other) {
    if other.is(Substring) {
      return toString.compare(other.toString)
    } else if other.is(String) {
      return toString.compare(other)
    }
    throw ArgumentError.new("Substring.compare: unsupported comparison type")
  }

  hash -> Int {
    toString.hash
  }
}

/// Character sequence view over a Substring.
class SubstringChars {
  _substring: Substring

  @constructor
  new(_ sub: Substring) { _substring = sub }

  substring -> Substring { _substring }

  iterator -> SubstringCharIterator { SubstringCharIterator.new(self) }
}

class SubstringCharIterator {
  _source: SubstringChars
  mut _relOffset: Int
  mut _done: Bool

  @constructor
  new(_ source: SubstringChars) {
    _source = source
    _relOffset = 0
    _done = false
  }

  next -> Option<Char> {
    if _done { return None }
    let sub = _source.substring
    if _relOffset >= sub.byteCount {
      _done = true
      return None
    }
    let abs = sub.startOffset + _relOffset
    match sub.string.decodeCharAt(abs) {
      Option::Some(ch) => {
        _relOffset = _relOffset + ch.utf8Length
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

impl CanonicalIterator for SubstringCharIterator {
  type Item = Char
}

impl CanonicalIterable for SubstringChars {
  type Item = Char
  type Iterator = SubstringCharIterator
  iterator -> SubstringCharIterator { SubstringCharIterator.new(self) }
}

/// Byte sequence view over a Substring.
class SubstringBytes {
  _substring: Substring

  @constructor
  new(_ sub: Substring) { _substring = sub }

  size -> Int { _substring.byteCount }

  rawByteAt(_ i: Int) -> Option<Int> { _substring.rawByteAt(i) }

  iterator -> SubstringByteIterator { SubstringByteIterator.new(self) }
}

class SubstringByteIterator {
  _source: SubstringBytes
  mut _offset: Int
  mut _done: Bool

  @constructor
  new(_ source: SubstringBytes) {
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
    let value = _source.rawByteAt(_offset)
    _offset = _offset + 1
    value
  }

  iterator { self }
}

impl CanonicalIterator for SubstringByteIterator {
  type Item = Int
}

impl CanonicalIterable for SubstringBytes {
  type Item = Int
  type Iterator = SubstringByteIterator
  iterator -> SubstringByteIterator { SubstringByteIterator.new(self) }
}

export Substring, SubstringChars, SubstringCharIterator, SubstringBytes, SubstringByteIterator
