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

  /// Checks whether this substring contains the given pattern.
  contains(_ pattern) -> Bool {
    let opt = pattern.findIn(self, 0)
    match opt {
      Option::Some(_) => true
      _ => false
    }
  }

  /// Determines whether the substring begins with the given prefix pattern.
  startsWith(_ prefix) -> Bool {
    if prefix.is(Char) {
      if first.isSome {
        return first.unwrap == prefix
      }
      return false
    }
    if prefix.is(String) {
      let pLen = prefix.rawByteCount
      if pLen > byteCount { return false }
      if pLen == 0 { return true }
      let opt = prefix.findIn(self, 0)
      if opt.isSome {
        return opt.unwrap.rawAt(0) == 0
      }
      return false
    }
    if prefix.is(Substring) {
      let pLen = prefix.byteCount
      if pLen > byteCount { return false }
      if pLen == 0 { return true }
      let opt = prefix.findIn(self, 0)
      if opt.isSome {
        return opt.unwrap.rawAt(0) == 0
      }
      return false
    }
    let opt = prefix.findIn(self, 0)
    if opt.isSome {
      return opt.unwrap.rawAt(0) == 0
    }
    return false
  }

  /// Determines whether the substring terminates with the given suffix pattern.
  endsWith(_ suffix) -> Bool {
    if suffix.is(Char) {
      if last.isSome {
        return last.unwrap == suffix
      }
      return false
    }
    if suffix.is(String) {
      let sLen = suffix.rawByteCount
      if sLen > byteCount { return false }
      if sLen == 0 { return true }
      let fromOffset = byteCount - sLen
      let opt = suffix.findIn(self, fromOffset)
      if opt.isSome {
        return opt.unwrap.rawAt(1) == byteCount
      }
      return false
    }
    if suffix.is(Substring) {
      let sLen = suffix.byteCount
      if sLen > byteCount { return false }
      if sLen == 0 { return true }
      let fromOffset = byteCount - sLen
      let opt = suffix.findIn(self, fromOffset)
      if opt.isSome {
        return opt.unwrap.rawAt(1) == byteCount
      }
      return false
    }
    return false
  }

  /// Searches for the first occurrence of `pattern`, returning its start `StringIndex`.
  find(_ pattern) -> Option<StringIndex> {
    let opt = pattern.findIn(self, 0)
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
      return Some(StringIndex(byteCount))
    }
    let optRel = _string.rawRFindSlice(needle, _start, _end)
    if optRel.isSome {
      return Some(StringIndex(optRel.unwrap))
    }
    return None
  }

  /// Searches for the first occurrence of `pattern`, returning the matching byte range.
  findRange(_ pattern) -> Option<Range> {
    let opt = pattern.findIn(self, 0)
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
    let total = byteCount
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
    let mut end = byteCount
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
    let total = byteCount
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

  /// If the substring starts with `prefix`, returns the Substring after the prefix; otherwise None.
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
    Some(slice(len, byteCount))
  }

  /// If the substring ends with `suffix`, returns the Substring before the suffix; otherwise None.
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
    Some(slice(0, byteCount - len))
  }

  /// Replaces the first occurrence of `pattern` with `replacement`.
  replaceFirst(_ pattern, with replacement: String) -> String {
    let opt = pattern.findIn(self, 0)
    if opt.isSome {
      let pair = opt.unwrap
      let s = pair.rawAt(0)
      let e = pair.rawAt(1)
      return slice(0, s).toString + replacement + slice(e, byteCount).toString
    }
    return toString
  }

  /// Replaces all occurrences of `pattern` with `replacement`.
  replaceAll(_ pattern, with replacement: String) -> String {
    let mut out = ""
    let mut cursor = 0
    let total = byteCount
    while cursor <= total {
      let opt = pattern.findIn(self, cursor)
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

  /// Splits this substring around matches of `pattern` into a lazy sequence of Substrings.
  split(_ pattern) -> Split {
    Split.new(self, pattern)
  }

  /// Splits this substring around the first match of `separator` into a pair of Substrings.
  splitOnce(_ separator) -> Option<(Substring, Substring)> {
    let opt = separator.findIn(self, 0)
    if opt.isSome {
      let pair = opt.unwrap
      let s = pair.rawAt(0)
      let e = pair.rawAt(1)
      let left = self.slice(0, s)
      let right = self.slice(e, self.byteCount)
      return Some((left, right))
    }
    return None
  }

  /// Splits this substring into words separated by Unicode whitespace.
  splitWhitespace() -> SplitWhitespace {
    SplitWhitespace.new(self)
  }

  /// Yields a lazy sequence of lines separated by newline terminators.
  lines() -> Lines {
    Lines.new(self, false)
  }

  /// Yields a lazy sequence of lines, optionally retaining terminators.
  lines(keepingTerminators keep: Bool) -> Lines {
    Lines.new(self, keep)
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

/// A lazy sequence that yields Substring components separated by a pattern.
class Split {
  _source: Substring
  _pattern: Dynamic

  @constructor
  new(_ source: Substring, _ pattern: Dynamic) {
    _source = source
    _pattern = pattern
  }

  @class
  call(_ source: Substring, _ pattern: Dynamic) -> Split {
    Split.new(source, pattern)
  }

  source -> Substring { _source }
  pattern -> Dynamic { _pattern }

  iterator -> SplitIterator {
    SplitIterator.new(self)
  }
}

/// An iterator over pattern-separated Substring chunks.
class SplitIterator {
  _source: Substring
  _pattern: Dynamic
  mut _cursor: Int
  mut _done: Bool

  @constructor
  new(_ split: Split) {
    _source = split.source
    _pattern = split.pattern
    _cursor = 0
    _done = false
  }

  next -> Option<Substring> {
    if _done {
      return None
    }
    let total = _source.byteCount
    if _cursor > total {
      _done = true
      return None
    }
    let opt = _pattern.findIn(_source, _cursor)
    if opt.isSome {
      let pair = opt.unwrap
      let mStart = pair.rawAt(0)
      let mEnd = pair.rawAt(1)
      if mStart == mEnd {
        let chunk = _source.slice(_cursor, mStart)
        if _cursor < total {
          let optCh = _source.decodeCharAt(_cursor)
          let step = if optCh.isSome { optCh.unwrap.utf8Length } else { 1 }
          _cursor = _cursor + step
        } else {
          _cursor = total + 1
        }
        return Some(chunk)
      }
      let chunk = _source.slice(_cursor, mStart)
      _cursor = mEnd
      return Some(chunk)
    } else {
      let chunk = _source.slice(_cursor, total)
      _cursor = total + 1
      return Some(chunk)
    }
  }

  iterator -> SplitIterator { self }
}

impl CanonicalIterator for SplitIterator {
  type Item = Substring
}

impl CanonicalIterable for Split {
  type Item = Substring
  type Iterator = SplitIterator
  iterator -> SplitIterator { SplitIterator.new(self) }
}

/// A lazy sequence that yields non-empty words separated by Unicode whitespace.
class SplitWhitespace {
  _source: Substring

  @constructor
  new(_ source: Substring) {
    _source = source
  }

  @class
  call(_ source: Substring) -> SplitWhitespace {
    SplitWhitespace.new(source)
  }

  source -> Substring { _source }

  iterator -> SplitWhitespaceIterator {
    SplitWhitespaceIterator.new(self)
  }
}

/// An iterator yielding non-empty words separated by Unicode whitespace.
class SplitWhitespaceIterator {
  _source: Substring
  mut _cursor: Int

  @constructor
  new(_ split: SplitWhitespace) {
    _source = split.source
    _cursor = 0
  }

  next -> Option<Substring> {
    let total = _source.byteCount
    while _cursor < total {
      let optCh = _source.decodeCharAt(_cursor)
      if optCh.isSome {
        let ch = optCh.unwrap
        if ch.isWhitespace {
          _cursor = _cursor + ch.utf8Length
        } else {
          break
        }
      } else {
        break
      }
    }
    if _cursor >= total {
      return None
    }
    let wordStart = _cursor
    while _cursor < total {
      let optCh = _source.decodeCharAt(_cursor)
      if optCh.isSome {
        let ch = optCh.unwrap
        if ch.isWhitespace {
          break
        }
        _cursor = _cursor + ch.utf8Length
      } else {
        _cursor = _cursor + 1
      }
    }
    let wordEnd = _cursor
    return Some(_source.slice(wordStart, wordEnd))
  }

  iterator -> SplitWhitespaceIterator { self }
}

impl CanonicalIterator for SplitWhitespaceIterator {
  type Item = Substring
}

impl CanonicalIterable for SplitWhitespace {
  type Item = Substring
  type Iterator = SplitWhitespaceIterator
  iterator -> SplitWhitespaceIterator { SplitWhitespaceIterator.new(self) }
}

/// A lazy sequence that yields lines separated by newline terminators (\r\n, \n, or \r).
class Lines {
  _source: Substring
  _keepTerminators: Bool

  @constructor
  new(_ source: Substring, _ keepingTerminators: Bool) {
    _source = source
    _keepTerminators = keepingTerminators
  }

  @class
  call(_ source: Substring) -> Lines {
    Lines.new(source, false)
  }

  @class
  call(_ source: Substring, keepingTerminators keep: Bool) -> Lines {
    Lines.new(source, keep)
  }

  source -> Substring { _source }
  keepingTerminators -> Bool { _keepTerminators }

  iterator -> LinesIterator {
    LinesIterator.new(self)
  }
}

/// An iterator over lines separated by newline terminators.
class LinesIterator {
  _source: Substring
  _keep: Bool
  mut _cursor: Int
  mut _done: Bool

  @constructor
  new(_ lines: Lines) {
    _source = lines.source
    _keep = lines.keepingTerminators
    _cursor = 0
    _done = false
  }

  next -> Option<Substring> {
    if _done {
      return None
    }
    let total = _source.byteCount
    if _cursor >= total {
      _done = true
      return None
    }
    let mut i = _cursor
    while i < total {
      let bOpt = _source.rawByteAt(i)
      if bOpt.isSome {
        let b = bOpt.unwrap
        if b == 10 {
          let lineEnd = if _keep { i + 1 } else { i }
          let chunk = _source.slice(_cursor, lineEnd)
          _cursor = i + 1
          return Some(chunk)
        }
        if b == 13 {
          if i + 1 < total and _source.rawByteAt(i + 1) == Some(10) {
            let lineEnd = if _keep { i + 2 } else { i }
            let chunk = _source.slice(_cursor, lineEnd)
            _cursor = i + 2
            return Some(chunk)
          } else {
            let lineEnd = if _keep { i + 1 } else { i }
            let chunk = _source.slice(_cursor, lineEnd)
            _cursor = i + 1
            return Some(chunk)
          }
        }
      }
      i = i + 1
    }
    let chunk = _source.slice(_cursor, total)
    _cursor = total
    return Some(chunk)
  }

  iterator -> LinesIterator { self }
}

impl CanonicalIterator for LinesIterator {
  type Item = Substring
}

impl CanonicalIterable for Lines {
  type Item = Substring
  type Iterator = LinesIterator
  iterator -> LinesIterator { LinesIterator.new(self) }
}

export (
  Substring,
  SubstringChars,
  SubstringCharIterator,
  SubstringBytes,
  SubstringByteIterator,
  Split,
  SplitIterator,
  SplitWhitespace,
  SplitWhitespaceIterator,
  Lines,
  LinesIterator
)
