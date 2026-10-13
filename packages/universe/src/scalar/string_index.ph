/// Validated UTF-8 byte offset into a text sequence.
///
/// Distinguishes logical character boundaries and slicing positions
/// from arbitrary integers.
class StringIndex {
  _offset: Int

  @constructor
  new(_ offset: Int) {
    if not offset.is(Number) or offset < 0 {
      throw ArgumentError.new("StringIndex: offset must be a non-negative integer")
    }
    _offset = offset
  }

  @class
  call(_ offset: Int) -> StringIndex {
    StringIndex.new(offset)
  }

  /// The underlying byte offset.
  offset -> Int { _offset }

  compare(_ other: StringIndex) {
    if not other.is(StringIndex) {
      throw ArgumentError.new("StringIndex.compare: argument must be a StringIndex")
    }
    _offset.compare(other.offset)
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(StringIndex) { return false }
    _offset == other.offset
  }

  <(_ other: StringIndex) -> Bool {
    if not other.is(StringIndex) {
      throw ArgumentError.new("<: argument must be a StringIndex")
    }
    _offset < other.offset
  }

  <=(_ other: StringIndex) -> Bool {
    if not other.is(StringIndex) {
      throw ArgumentError.new("<=: argument must be a StringIndex")
    }
    _offset <= other.offset
  }

  >(_ other: StringIndex) -> Bool {
    if not other.is(StringIndex) {
      throw ArgumentError.new(">: argument must be a StringIndex")
    }
    _offset > other.offset
  }

  >=(_ other: StringIndex) -> Bool {
    if not other.is(StringIndex) {
      throw ArgumentError.new(">=: argument must be a StringIndex")
    }
    _offset >= other.offset
  }

  hash -> Int { _offset.hash }

  toString -> String { "StringIndex(\(_offset))" }
}

export StringIndex
