from universe.collections.iteration import Iterator as CanonicalIterator, Iterable as CanonicalIterable

/// Owned bounded mutable storage used to construct collection values.
native class Buffer<T> is Collection {
  @class
  native new(_ capacity: Int) -> Buffer<T>
  internal native rawLength -> Int
  internal native rawCapacity -> Int
  internal native rawAt(_ index: Int) -> T
  internal native rawSet(_ index: Int, _ value: T) -> Unit
  internal native rawPush(_ value: T) -> Bool
  internal native rawSlice(_ start: Int, _ end: Int) -> Slice<T>

  size { self.rawLength }
  /// Returns the maximum number of elements this buffer can hold.
  capacity { self.rawCapacity }
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
  set(_ index: Int, _ value: T) -> Unit { self.rawSet(index, value) }

  /// Appends one value when capacity remains, or returns a recoverable error.
  push(_ value: T) -> Result<Unit, Error> {
    if self.rawPush(value) {
      return Result::Ok(())
    }
    return Result::Error(Error.new("Buffer capacity exhausted"))
  }

  iterator -> BufferIterator<T> { BufferIterator<T>.new(self) }

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
    if other is not Buffer { return false }
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

class BufferIterator<T> {
  _values: Buffer<T>
  mut _index: Int

  @constructor new(_ values: Buffer<T>) {
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

impl<T> CanonicalIterator for BufferIterator<T> {
  type Item = T
}

impl<T> CanonicalIterable for Buffer<T> {
  type Item = T
  type Iterator = BufferIterator<T>
  iterator -> BufferIterator<T> { BufferIterator<T>.new(self) }
}

export Buffer
