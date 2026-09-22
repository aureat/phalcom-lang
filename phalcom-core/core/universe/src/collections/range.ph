from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

/// Describes a bounded or unbounded numeric interval.
native class Range is Collection {
  internal native rawLower -> Option<Number>
  internal native rawUpper -> Option<Number>
  internal native rawUpperInclusive -> Bool
  /// Constructs a range from optional lower and upper bounds.
  @class
  new(_ lower, _ upper, _ upperInclusive) {
    if lower == None {
      if upper == None {
        return ..
      }
      if upperInclusive {
        return ..=upper
      }
      return ..upper
    }
    if upper == None {
      return lower..
    }
    if upperInclusive {
      return lower..=upper
    }
    lower..upper
  }

  /// Reports whether a value is an integral numeric slice coordinate.
  private isSliceCoordinate(_ value) {
    return value is Number and value % 1 == 0
  }

  private sliceBoundary(_ coordinate, _ size) {
    if coordinate < 0 {
      if coordinate < -size { return 0 }
      return size + coordinate
    }
    if coordinate > size { return size }
    return coordinate
  }

  private sliceInclusiveEnd(_ coordinate, _ size) {
    if coordinate < 0 {
      if coordinate < -size { return 0 }
      return size + coordinate + 1
    }
    if coordinate >= size { return size }
    return coordinate + 1
  }

  /// Normalizes bounds for a finite sequence of `size` elements.
  internal rawSliceBounds(_ size) {
    let start = 0
    let end = size
    let lower = self.rawLower
    if lower.isSome {
      let coordinate = lower.unwrapOr(0)
      if not self.isSliceCoordinate(coordinate) {
        return Result::Error(SliceError.new("Range lower bound must be an integer coordinate"))
      }
      start = self.sliceBoundary(coordinate, size)
    }
    let upper = self.rawUpper
    if upper.isSome {
      let coordinate = upper.unwrapOr(0)
      if not self.isSliceCoordinate(coordinate) {
        return Result::Error(SliceError.new("Range upper bound must be an integer coordinate"))
      }
      if self.rawUpperInclusive {
        end = self.sliceInclusiveEnd(coordinate, size)
      } else {
        end = self.sliceBoundary(coordinate, size)
      }
    }
    return Result::Ok((start, end))
  }

  /// Returns a fresh iterator over the range.
  iterator -> RangeIterator { RangeIterator.new(self) }

  /// Returns the lower bound, or raises for an unbounded lower side.
  first {
    let lower = self.rawLower
    if lower.isNone {
      throw Error.new("Range has no first element because lower bound is absent")
    }
    return lower.unwrapOr(None)
  }

  /// Returns the last included value, or raises for an unbounded upper side.
  last {
    let upper = self.rawUpper
    if upper.isNone {
      throw Error.new("Range has no last element because upper bound is absent")
    }
    let value = upper.unwrapOr(None)
    if self.rawUpperInclusive {
      return value
    }
    return value - 1
  }

  /// Returns the number of values in a bounded range.
  size {
    let lower = self.rawLower
    let upper = self.rawUpper
    if lower.isNone or upper.isNone {
      throw Error.new("Unbounded Range has no size")
    }
    let first = lower.unwrapOr(None)
    let last = upper.unwrapOr(None)
    if first > last {
      return 0
    }
    let difference = last - first
    if self.rawUpperInclusive {
      return difference + 1
    }
    difference
  }

  /// Reports whether `x` falls within the range bounds.
  contains(_ x) {
    let lower = self.rawLower
    let upper = self.rawUpper
    let lowerOk = true
    let upperOk = true

    if lower.isSome {
      lowerOk = x >= lower.unwrapOr(None)
    }
    if upper.isSome {
      let bound = upper.unwrapOr(None)
      if self.rawUpperInclusive {
        upperOk = x <= bound
      } else {
        upperOk = x < bound
      }
    }
    lowerOk and upperOk
  }

  /// Reports whether `x` falls within the range bounds.
  includes(_ x) { self.contains(x) }

  /// Returns the value at zero-based index `i`, supporting negative indexes.
  at(_ i) {
    let lower = self.rawLower
    if lower.isNone {
      throw Error.new("Range index out of range (lower bound is absent)")
    }
    let index = i
    if index < 0 {
      index = self.size + index
    }
    let count = self.size
    if index < 0 or index >= count {
      throw Error.new("Range index out of range")
    }
    lower.unwrapOr(None) + index
  }

  /// Compares ranges by bounds and upper-bound inclusivity.
  ==(_ other) {
    if other is not Range {
      return false
    }
    self.rawLower == other.rawLower and
      self.rawUpper == other.rawUpper and
      self.rawUpperInclusive == other.rawUpperInclusive
  }

  /// Renders the range using its bound syntax.
  toString {
    let lower = ""
    let upper = ""
    if self.rawLower.isSome {
      lower = self.rawLower.unwrapOr(0).toString
    }
    if self.rawUpper.isSome {
      upper = self.rawUpper.unwrapOr(0).toString
    }
    let operator = ".."
    if self.rawUpperInclusive {
      operator = "..="
    }
    lower + operator + upper
  }

  /// Computes a hash from the range bounds.
  hash {
    let lower = 17
    let upper = 31
    let inclusive = 0
    if self.rawLower.isSome {
      lower = self.rawLower.unwrapOr(0).hash
    }
    if self.rawUpper.isSome {
      upper = self.rawUpper.unwrapOr(0).hash
    }
    if self.rawUpperInclusive {
      inclusive = 1
    }
    lower + upper * 37 + inclusive * 97
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
    if _done { return None }

    if not _started {
      let lowerOpt = _range.rawLower
      if lowerOpt.isNone {
        throw ArgumentError.new("Range iteration unsupported when lower bound is absent")
      }
      let lower = lowerOpt.unwrapOr(0)
      if lower is not Number or lower % 1 != 0 {
        throw ArgumentError.new("Range iteration unsupported: lower bound must be an integer")
      }

      let upperOpt = _range.rawUpper
      if upperOpt.isSome {
        let upper = upperOpt.unwrapOr(0)
        if upper is not Number or upper % 1 != 0 {
          throw ArgumentError.new("Range iteration unsupported: upper bound must be an integer")
        }
        if lower > upper {
          throw ArgumentError.new("Range iteration unsupported: lower bound exceeds upper (descending traversal not supported)")
        }
      }

      _nextValue = lower
      _started = true
    } else {
      _nextValue = _nextValue + 1
    }

    let upperOpt = _range.rawUpper
    if upperOpt.isSome {
      let upper = upperOpt.unwrapOr(0)
      let live = _nextValue < upper
      if _range.rawUpperInclusive {
        live = _nextValue <= upper
      }
      if not live {
        _done = true
        return None
      }
    }
    Some(_nextValue)
  }

  iterator { self }

  /// Returns all remaining values as a list.
  toList {
    let result = List<Dynamic>.new()
    let step = self.next
    while step.isSome {
      let value = step.match(some: |v| { v }, none: || { None })
      result.append(value)
      step = self.next
    }
    result
  }

  /// Returns the first `n` values from this iterator.
  take(_ n) { TakeIterator.new(self, n) }
}

impl CanonicalIterator for RangeIterator {
  type Item = Number
}

impl CanonicalIterable for Range {
  type Item = Number
  type Iterator = RangeIterator
  iterator -> RangeIterator { RangeIterator.new(self) }
}
