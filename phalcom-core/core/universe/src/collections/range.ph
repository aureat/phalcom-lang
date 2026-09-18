from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

// Range is a native bounds descriptor. Its lower_/upper_/upperInclusive_
// observations preserve omitted endpoints. Stateful traversal is owned by
// RangeIterator; the legacy cursor methods remain during the staged migration.
native class Range is Collection {
  internal native rawLower -> Option<Number>
  internal native rawUpper -> Option<Number>
  internal native rawUpperInclusive -> Bool
  @class
  new(_ lower, _ upper, _ upperInclusive) {
    if (lower == None) {
      if (upper == None) { return .. }
      return upperInclusive.ifTrue(|| { ..=upper }, ifFalse: || { ..upper })
    }
    if (upper == None) { return lower.. }
    return upperInclusive.ifTrue(|| { lower..=upper }, ifFalse: || { lower..upper })
  }

  private isSliceCoordinate(_ value) {
    // TODO(NUMERIC-TOWER): require Int once the tower is fully landed.
    return value.is(Number) and ((value % 1) == 0)
  }

  private sliceBoundary(_ coordinate, _ size) {
    if (coordinate < 0) {
      if (coordinate < -size) { return 0 }
      return size + coordinate
    }
    if (coordinate > size) { return size }
    return coordinate
  }

  private sliceInclusiveEnd(_ coordinate, _ size) {
    if (coordinate < 0) {
      if (coordinate < -size) { return 0 }
      // Here -size <= coordinate < 0, so adding one cannot overflow and
      // denotes the exclusive boundary after the included element.
      return size + coordinate + 1
    }
    if (coordinate >= size) { return size }
    return coordinate + 1
  }

  // Normalizes this bound descriptor for a finite sequence of `size` elements.
  // Omitted endpoints are distinct from a supplied None, which is malformed.
  internal rawSliceBounds(_ size) {
    let start = 0
    let end = size
    let lower = self.rawLower
    if (lower.isSome) {
      let coordinate = lower.unwrapOr(0)
      if (not self.isSliceCoordinate(coordinate)) {
        return Result::Error(SliceError.new("Range lower bound must be an integer coordinate"))
      }
      start = self.sliceBoundary(coordinate, size)
    }
    let upper = self.rawUpper
    if (upper.isSome) {
      let coordinate = upper.unwrapOr(0)
      if (not self.isSliceCoordinate(coordinate)) {
        return Result::Error(SliceError.new("Range upper bound must be an integer coordinate"))
      }
      if (self.rawUpperInclusive) {
        end = self.sliceInclusiveEnd(coordinate, size)
      } else {
        end = self.sliceBoundary(coordinate, size)
      }
    }
    return Result::Ok((start, end))
  }

  iterator -> RangeIterator { RangeIterator.new(self) }

  // first, last, size, includes (Spec E.2 / Range specs)
  first {
    let lowerOpt = self.rawLower
    lowerOpt.isNone.ifTrue || {
      throw Error.new("Range has no first element because lower bound is absent")
    }
    return lowerOpt.unwrapOr(None)
  }

  last {
    let upperOpt = self.rawUpper
    upperOpt.isNone.ifTrue || {
      throw Error.new("Range has no last element because upper bound is absent")
    }
    let upper = upperOpt.unwrapOr(None)
    return self.rawUpperInclusive.ifTrue(|| { upper }, ifFalse: || { upper - 1 })
  }

  size {
    let lowerOpt = self.rawLower
    let upperOpt = self.rawUpper
    (lowerOpt.isNone or upperOpt.isNone).ifTrue || {
      throw Error.new("Unbounded Range has no size")
    }
    let lower = lowerOpt.unwrapOr(None)
    let upper = upperOpt.unwrapOr(None)
    (lower > upper).ifTrue || {
      return 0
    }
    let diff = upper - lower
    return self.rawUpperInclusive.ifTrue(|| { diff + 1 }, ifFalse: || { diff })
  }

  contains(_ x) {
    let lowerOpt = self.rawLower
    let upperOpt = self.rawUpper

    let lowerOk = lowerOpt.isNone.ifTrue(|| { true }, ifFalse: || { x >= lowerOpt.unwrapOr(None) })
    let upperOk = upperOpt.isNone.ifTrue(|| { true }, ifFalse: || {
      self.rawUpperInclusive.ifTrue(|| { x <= upperOpt.unwrapOr(None) }, ifFalse: || { x < upperOpt.unwrapOr(None) })
    })
    return lowerOk and upperOk
  }

  includes(_ x) { self.contains(x) }

  at(_ i) {
    let lowerOpt = self.rawLower
    lowerOpt.isNone.ifTrue || {
      throw Error.new("Range index out of range (lower bound is absent)")
    }
    let lower = lowerOpt.unwrapOr(None)
    let index = i
    // Support negative indexing
    (index < 0).ifTrue || {
      let sz = self.size
      index = sz + index
    }
    // Check range bounds
    let sz = self.size
    (index < 0 or index >= sz).ifTrue || {
      throw Error.new("Range index out of range")
    }
    return lower + index
  }

  ==(_ other) {
    other.is(Range).ifFalse || { return false }
    return (self.rawLower == other.rawLower) and
           (self.rawUpper == other.rawUpper) and
           (self.rawUpperInclusive == other.rawUpperInclusive)
  }

  toString {
    let lowerOpt = self.rawLower
    let upperOpt = self.rawUpper
    let lowerStr = lowerOpt.isNone.ifTrue(|| { "" }, ifFalse: || { lowerOpt.unwrapOr(0).toString })
    let upperStr = upperOpt.isNone.ifTrue(|| { "" }, ifFalse: || { upperOpt.unwrapOr(0).toString })
    let op = self.rawUpperInclusive.ifTrue(|| { "..=" }, ifFalse: || { ".." })
    return lowerStr + op + upperStr
  }

  hash {
    let h1 = self.rawLower.isSome.ifTrue(|| { self.rawLower.unwrapOr(0).hash }, ifFalse: || { 17 })
    let h2 = self.rawUpper.isSome.ifTrue(|| { self.rawUpper.unwrapOr(0).hash }, ifFalse: || { 31 })
    let h3 = self.rawUpperInclusive.ifTrue(|| { 1 }, ifFalse: || { 0 })
    return h1 + h2 * 37 + h3 * 97
  }
}

class RangeIterator {
  _range: Range
  _started: Bool
  _done: Bool
  _nextValue: Number

  @constructor
  new(_ range: Range) {
    _range = range
    _started = false
    _done = false
    _nextValue = 0
  }

  next -> Option<Number> {
    if (_done) { return None }

    if (not _started) {
      let lowerOpt = _range.rawLower
      if (lowerOpt.isNone) {
        throw ArgumentError.new("Range iteration unsupported when lower bound is absent")
      }
      let lower = lowerOpt.unwrapOr(0)
      if (not (lower.is(Number) and ((lower % 1) == 0))) {
        throw ArgumentError.new("Range iteration unsupported: lower bound must be an integer")
      }

      let upperOpt = _range.rawUpper
      if (upperOpt.isSome) {
        let upper = upperOpt.unwrapOr(0)
        if (not (upper.is(Number) and ((upper % 1) == 0))) {
          throw ArgumentError.new("Range iteration unsupported: upper bound must be an integer")
        }
        if (lower > upper) {
          throw ArgumentError.new("Range iteration unsupported: lower bound exceeds upper (descending traversal not supported)")
        }
      }

      _nextValue = lower
      _started = true
    } else {
      _nextValue = _nextValue + 1
    }

    let upperOpt = _range.rawUpper
    if (upperOpt.isSome) {
      let upper = upperOpt.unwrapOr(0)
      let live = _range.rawUpperInclusive.ifTrue(|| { _nextValue <= upper }, ifFalse: || { _nextValue < upper })
      if (not live) {
        _done = true
        return None
      }
    }
    Some(_nextValue)
  }

  iterator { self }
}

impl CanonicalIterator for RangeIterator {
  type Item = Number
}

impl CanonicalIterable for Range {
  type Item = Number
  type Iterator = RangeIterator
  iterator -> RangeIterator { RangeIterator.new(self) }
}
