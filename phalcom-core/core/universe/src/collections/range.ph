// Range is a native bounds descriptor. Its lower_/upper_/upperInclusive_
// observations preserve omitted endpoints. Progression, equality, hashing,
// and traversal are deliberately deferred.
class Range is Iterable {
  @class
  new(_ lower, _ upper, _ upperInclusive) {
    if (lower == None) {
      if (upper == None) { return .. }
      return upperInclusive.ifTrue(|| { ..=upper }, ifFalse: || { ..upper })
    }
    if (upper == None) { return lower.. }
    return upperInclusive.ifTrue(|| { lower..=upper }, ifFalse: || { lower..upper })
  }

  @private
  isSliceCoordinate(_ value) {
    // TODO(NUMERIC-TOWER): require Int once the tower is fully landed.
    return value.is(Number) and ((value % 1) == 0)
  }

  @private
  sliceBoundary(_ coordinate, _ size) {
    if (coordinate < 0) {
      if (coordinate < -size) { return 0 }
      return size + coordinate
    }
    if (coordinate > size) { return size }
    return coordinate
  }

  @private
  sliceInclusiveEnd(_ coordinate, _ size) {
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
  @internal
  _$sliceBounds(_ size) {
    let start = 0
    let end = size
    let lower = self._$lower
    if (lower.isSome) {
      let coordinate = lower.unwrapOr(None)
      if (not self.isSliceCoordinate(coordinate)) {
        return Err.new(SliceError.new("Range lower bound must be an integer coordinate"))
      }
      start = self.sliceBoundary(coordinate, size)
    }
    let upper = self._$upper
    if (upper.isSome) {
      let coordinate = upper.unwrapOr(None)
      if (not self.isSliceCoordinate(coordinate)) {
        return Err.new(SliceError.new("Range upper bound must be an integer coordinate"))
      }
      if (self._$upperInclusive) {
        end = self.sliceInclusiveEnd(coordinate, size)
      } else {
        end = self.sliceBoundary(coordinate, size)
      }
    }
    return Ok.new((start, end))
  }

  // iterate and iteratorValue for forward integer iteration (E.2).
  // The cursor is the current yielded integer value, not an offset from lower.
  iterate(_ previous) {
    let lowerOpt = self._$lower
    let upperOpt = self._$upper

    // lower is required for iteration
    lowerOpt.isNone.ifTrue || {
      throw ArgumentError.new("Range iteration unsupported when lower bound is absent")
    }
    let lower = lowerOpt.unwrapOr(None)
    (self.isSliceCoordinate(lower)).ifFalse || {
      throw ArgumentError.new("Range iteration unsupported: lower bound must be an integer")
    }

    let hasUpper = upperOpt.isSome
    let upper = hasUpper.ifTrue(|| { upperOpt.unwrapOr(None) }, ifFalse: || { None })
    hasUpper.ifTrue || {
      (self.isSliceCoordinate(upper)).ifFalse || {
        throw ArgumentError.new("Range iteration unsupported: upper bound must be an integer")
      }
      (lower > upper).ifTrue || {
        throw ArgumentError.new("Range iteration unsupported: lower bound exceeds upper (descending traversal not supported)")
      }
    }

    let candidate = (previous == None).ifTrue(|| { lower }, ifFalse: || { previous + 1 })

    hasUpper.ifFalse || {
      return candidate
    }

    let inclusive = self._$upperInclusive
    let live = inclusive.ifTrue(|| { candidate <= upper }, ifFalse: || { candidate < upper })
    return live.ifTrue(|| { candidate }, ifFalse: || { None })
  }

  iteratorValue(_ cursor) { cursor }

  // first, last, size, includes (Spec E.2 / Range specs)
  first {
    let lowerOpt = self._$lower
    lowerOpt.isNone.ifTrue || {
      throw Error.new("Range has no first element because lower bound is absent")
    }
    return lowerOpt.unwrapOr(None)
  }

  last {
    let upperOpt = self._$upper
    upperOpt.isNone.ifTrue || {
      throw Error.new("Range has no last element because upper bound is absent")
    }
    let upper = upperOpt.unwrapOr(None)
    return self._$upperInclusive.ifTrue(|| { upper }, ifFalse: || { upper - 1 })
  }

  size {
    let lowerOpt = self._$lower
    let upperOpt = self._$upper
    (lowerOpt.isNone or upperOpt.isNone).ifTrue || {
      throw Error.new("Unbounded Range has no size")
    }
    let lower = lowerOpt.unwrapOr(None)
    let upper = upperOpt.unwrapOr(None)
    (lower > upper).ifTrue || {
      return 0
    }
    let diff = upper - lower
    return self._$upperInclusive.ifTrue(|| { diff + 1 }, ifFalse: || { diff })
  }

  contains(_ x) {
    let lowerOpt = self._$lower
    let upperOpt = self._$upper

    let lowerOk = lowerOpt.isNone.ifTrue(|| { true }, ifFalse: || { x >= lowerOpt.unwrapOr(None) })
    let upperOk = upperOpt.isNone.ifTrue(|| { true }, ifFalse: || {
      self._$upperInclusive.ifTrue(|| { x <= upperOpt.unwrapOr(None) }, ifFalse: || { x < upperOpt.unwrapOr(None) })
    })
    return lowerOk and upperOk
  }

  includes(_ x) { self.contains(x) }

  at(_ i) {
    let lowerOpt = self._$lower
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
    return (self._$lower == other._$lower) and
           (self._$upper == other._$upper) and
           (self._$upperInclusive == other._$upperInclusive)
  }

  toString {
    let lowerOpt = self._$lower
    let upperOpt = self._$upper
    let lowerStr = lowerOpt.isNone.ifTrue(|| { "" }, ifFalse: || { lowerOpt.unwrapOr(None).toString })
    let upperStr = upperOpt.isNone.ifTrue(|| { "" }, ifFalse: || { upperOpt.unwrapOr(None).toString })
    let op = self._$upperInclusive.ifTrue(|| { "..=" }, ifFalse: || { ".." })
    return lowerStr + op + upperStr
  }

  hash {
    let h1 = self._$lower.isSome.ifTrue(|| { self._$lower.unwrapOr(None).hash }, ifFalse: || { 17 })
    let h2 = self._$upper.isSome.ifTrue(|| { self._$upper.unwrapOr(None).hash }, ifFalse: || { 31 })
    let h3 = self._$upperInclusive.ifTrue(|| { 1 }, ifFalse: || { 0 })
    return h1 + h2 * 37 + h3 * 97
  }
}
