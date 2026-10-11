from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

/// An immutable homogeneous sequence with an exact static dimension.
native class Vector<T, N: Int> is Collection {
  internal native rawLength -> Int
  internal native rawAt(_ index: Int) -> T
  internal native rawSlice(_ start: Int, _ end: Int) -> Slice<T>

  size -> Int { self.rawLength }

  first -> Option<T> {
    if self.size == 0 { return None }
    Some(self.rawAt(0))
  }

  last -> Option<T> {
    if self.size == 0 { return None }
    Some(self.rawAt(self.size - 1))
  }

  at(_ index: Int) -> Option<T> {
    let mut i = index
    if i < 0 { i = self.size + i }
    if i >= 0 and i < self.size { return Some(self.rawAt(i)) }
    None
  }

  get(_ index: Int) -> Option<T> { self.at(index) }

  [_ index: Int] -> T {
    let mut i = index
    if i < 0 { i = self.size + i }
    if i >= 0 and i < self.size { return self.rawAt(i) }
    throw IndexError.new("Vector index out of range")
  }

  iterator -> VectorIterator<T, N> { VectorIterator<T, N>.new(self) }

  slice(_ range: Range) -> Slice<T> {
    return range.rawSliceBounds(self.size).match(
      ok: |bounds| {
        let mut start = bounds[0]
        let mut end = bounds[1]
        if start > end { end = start }
        self.rawSlice(start, end)
      },
      err: |error| { error.raise() }
    )
  }

  ==(_ other) -> Bool {
    if other is Vector {
      if self.size != other.size { return false }
      let mut i = 0
      while i < self.size {
        if self.rawAt(i) != other.rawAt(i) { return false }
        i = i + 1
      }
      return true
    }
    false
  }

  !=(_ other) -> Bool { not (self == other) }

  hash -> Int {
    let mut acc = 23 + self.size
    let mut i = 0
    while i < self.size {
      acc = (acc * 31 + self.rawAt(i).hash) % 999999937
      i = i + 1
    }
    acc
  }

  toString -> String {
    let mut text = "Vector(["
    let mut i = 0
    while i < self.size {
      if i > 0 { text = text + ", " }
      text = text + self.rawAt(i).toString
      i = i + 1
    }
    text + "])"
  }

  toArray -> Array<T> { <T>[*self] }

  toList -> List<T> {
    let result = List<T>.new()
    let mut i = 0
    while i < self.size {
      result.append(self.rawAt(i))
      i = i + 1
    }
    result
  }
}

class VectorIterator<T, N: Int> {
  _values: Vector<T, N>
  mut _index: Int

  @constructor new(_ values: Vector<T, N>) {
    _values = values
    _index = 0
  }

  next -> Option<T> {
    if _index < _values.size {
      let value = _values[_index]
      _index = _index + 1
      return Some(value)
    }
    None
  }

  iterator { self }
}

impl<T, N: Int> CanonicalIterator for VectorIterator<T, N> {
  type Item = T
}

impl<T, N: Int> CanonicalIterable for Vector<T, N> {
  type Item = T
  type Iterator = VectorIterator<T, N>
  iterator -> VectorIterator<T, N> { VectorIterator<T, N>.new(self) }
}
