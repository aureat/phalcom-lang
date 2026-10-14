from universe.collections.iteration import Iterator as CanonicalIterator, Iterable as CanonicalIterable

/// A read-only view over a fixed range of an owning collection.
native class Slice<T> is Collection {
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

  at(_ index: Int) -> T { self.rawAt(index) }

  get(_ index: Int) -> Option<T> {
    let mut i = index
    if i < 0 { i = self.size + i }
    if i >= 0 and i < self.size { return Some(self.rawAt(i)) }
    None
  }

  [_ index: Int] -> T { self.rawAt(index) }

  iterator -> SliceIterator<T> { SliceIterator<T>.new(self) }

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
    if other is not Slice { return false }
    if self.size != other.size { return false }
    let mut i = 0
    while i < self.size {
      if self.rawAt(i) != other.rawAt(i) { return false }
      i = i + 1
    }
    true
  }

  !=(_ other) -> Bool { not (self == other) }

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

class SliceIterator<T> {
  _values: Slice<T>
  mut _index: Int

  @constructor new(_ values: Slice<T>) {
    _values = values
    _index = 0
  }

  next -> Option<T> {
    if _index < _values.size {
      let value = _values.rawAt(_index)
      _index = _index + 1
      return Some(value)
    }
    None
  }

  iterator { self }
}

impl<T> CanonicalIterator for SliceIterator<T> {
  type Item = T
}

impl<T> CanonicalIterable for Slice<T> {
  type Item = T
  type Iterator = SliceIterator<T>
  iterator -> SliceIterator<T> { SliceIterator<T>.new(self) }
}

export Slice
