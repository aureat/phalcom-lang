from universe.collections.iteration import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class Array<T> is Collection {
  internal native rawLength -> Int
  internal native rawAt(_ index: Int) -> Dynamic
  internal native rawSet(_ index: Int, _ value: Dynamic) -> Dynamic
  internal native rawSlice(_ start: Int, _ end: Int) -> Slice<T>
  native toString -> String

  size { self.rawLength }

  first {
    if self.size == 0 { return None }
    return Some(self.at(0))
  }

  last {
    if self.size == 0 { return None }
    return Some(self.at(self.size - 1))
  }

  iterator -> ArrayIterator<T> { ArrayIterator.new(self) }

  slice(_ range: Range) -> Slice<T> {
    match range.rawSliceBounds(self.size) {
      Result::Ok(bounds) => {
        let mut start = bounds[0]
        let mut end = bounds[1]
        if start > end {
          end = start
        }
        self.rawSlice(start, end)
      }
      Result::Error(error) => { error.raise() }
    }
  }

  at(_ i: Int) -> T | None {
    return self.rawAt(i)
  }

  get(_ index: Int) -> Option<T> {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return Some(raw)
    }
    return None
  }

  [_ index: Int] -> T {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    throw IndexError.new("Array index out of range")
  }

  [_ index: Int, default: T] -> T {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    return default
  }

  get(_ index: Int, orElse: (Int) -> T) -> T {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    return orElse.call(index)
  }

  /// Replaces the element at `i` and returns the array.
  at(_ i: Int, put: T) -> Array<T> {
    let mut len = self.size
    let mut norm = i
    if norm < 0 { norm = len + norm }
    if norm < 0 or norm >= len {
      throw IndexError.new("Expected an in-range index, got an out-of-range Number")
    }
    self.rawSet(i, put)
    return self
  }

  /// Assigns an element through the subscript protocol.
  [_ i: Int]=(_ val: T) -> Array<T> {
    return self.at(i, put: val)
  }

  /// Compares arrays by size and corresponding element equality.
  ==(_ other) {
    if other is Array {
      let mut same = self.size == other.size
      let mut i = 0
      while same and i < self.size {
        same = self.at(i) == other.at(i)
        i = i + 1
      }
      return same
    }
    return false
  }

  /// Returns the inverse of structural array equality.
  !=(_ other) {
    return not (self == other)
  }

  toList {
    let mut out = List<T>.new()
    let mut i = 0
    let mut len = self.size
    while i < len {
      out.append(self.at(i))
      i = i + 1
    }
    return out
  }
}

class ArrayIterator<T> {
  mut _values: Array<T>
  mut _index: Int

  @constructor new(_ values: Array<T>) {
    _values = values
    _index = 0
  }

  next -> Option<T> {
    if _index < _values.size {
      let mut value = _values[_index]
      _index = _index + 1
      return Some(value)
    }
    None
  }

  iterator { self }
}

impl<T> CanonicalIterator for ArrayIterator<T> {
  type Item = T
}

impl<T> CanonicalIterable for Array<T> {
  type Item = T
  type Iterator = ArrayIterator<T>
  iterator -> ArrayIterator<T> { ArrayIterator.new(self) }
}
