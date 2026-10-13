from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class MutableBag<T> is Collection {
  @class
  native new() -> MutableBag<T>
  internal native rawSize -> Int
  internal native rawDistinctCount -> Int
  internal native rawCount(_ value: Dynamic) -> Int
  internal native rawAdd(_ value: Dynamic, _ count: Int) -> Dynamic
  internal native rawRemove(_ value: Dynamic, _ count: Int) -> Dynamic
  internal native rawAt(_ index: Int) -> T
  internal native rawCountAt(_ index: Int) -> Int
  internal native rawClear -> Unit

  size { self.rawSize }
  totalCount { self.rawSize }
  distinctCount { self.rawDistinctCount }
  count(_ value: T) -> Int { self.rawCount(value) }
  contains(_ value: T) -> Bool { self.count(value) > 0 }

  add(_ value: T) -> Self { self.rawAdd(value, 1) }
  add(_ value: T, count n: Int) -> Self { self.rawAdd(value, n) }
  remove(_ value: T) -> Self { self.rawRemove(value, 1) }
  remove(_ value: T, count n: Int) -> Self { self.rawRemove(value, n) }
  clear -> Unit { self.rawClear }

  iterator -> MutableBagIterator<T> { MutableBagIterator<T>.new(self) }
  entries -> MutableBagEntriesView<T> { MutableBagEntriesView<T>.new(self) }

  @class
  from<I>(_ source: I) -> MutableBag<T>
  where I |= CanonicalIterable, <I as CanonicalIterable>::Item == T
  {
    let mut result = MutableBag<T>.new()
    let mut it = source.iterator
    let mut step = it.next
    while step.isSome {
      let mut value = step.match(some: |item| { item }, none: || { None })
      result.add(value)
      step = it.next
    }
    return result
  }

  ==(_ other) {
    if not (other is MutableBag) { return false }
    let mut left = List<Dynamic>.new()
    let mut i = 0
    while i < self.distinctCount {
      left.append((value: self.rawAt(i), count: self.rawCountAt(i)))
      i = i + 1
    }
    let mut right = List<Dynamic>.new()
    i = 0
    while i < other.distinctCount {
      right.append((value: other.rawAt(i), count: other.rawCountAt(i)))
      i = i + 1
    }
    if left.size != right.size { return false }
    let mut same = true
    i = 0
    while same and i < left.size {
      let mut entry = left.at(i)
      let mut found = false
      let mut j = 0
      while not found and j < right.size {
        let mut candidate = right.at(j)
        let mut keyEqual = entry.value == candidate.value
        if not keyEqual and entry.value is Float and candidate.value is Float {
          keyEqual = entry.value.isNaN and candidate.value.isNaN
        }
        if keyEqual {
          found = entry.count == candidate.count
        }
        j = j + 1
      }
      same = found
      i = i + 1
    }
    return same
  }
}

class MutableBagIterator<T> {
  mut _source: MutableBag<T>
  mut _index: Int
  mut _remaining: Int
  mut _current: Option<T>

  @constructor
  new(_ source: MutableBag<T>) {
    _source = source
    _index = 0
    _remaining = 0
    _current = None
  }

  next -> Option<T> {
    while _remaining == 0 and _index < _source.distinctCount {
      _remaining = _source.rawCountAt(_index)
      if _remaining == 0 {
        _index = _index + 1
      } else {
        _current = Some(_source.rawAt(_index))
      }
    }
    if _remaining == 0 { return None }
    let mut value = _current.match(some: |item| { item }, none: || { None })
    _remaining = _remaining - 1
    if _remaining == 0 {
      _index = _index + 1
      _current = None
    }
    return Some(value)
  }

  iterator { self }
}

impl<T> CanonicalIterator for MutableBagIterator<T> {
  type Item = T
}

impl<T> CanonicalIterable for MutableBag<T> {
  type Item = T
  type Iterator = MutableBagIterator<T>
  iterator -> MutableBagIterator<T> { MutableBagIterator<T>.new(self) }
}

class MutableBagEntriesView<T> {
  mut _source: MutableBag<T>
  @constructor
  new(_ source: MutableBag<T>) { _source = source }
  size { _source.distinctCount }
  at(_ index: Int) -> Option<(value: T, count: Int)> {
    if index < 0 or index >= _source.distinctCount { return None }
    return Some((value: _source.rawAt(index), count: _source.rawCountAt(index)))
  }
  iterator -> MutableBagEntriesIterator<T> { MutableBagEntriesIterator<T>.new(self) }
}

class MutableBagEntriesIterator<T> {
  mut _source: MutableBagEntriesView<T>
  mut _index: Int
  @constructor
  new(_ source: MutableBagEntriesView<T>) { _source = source; _index = 0 }
  next -> Option<(value: T, count: Int)> {
    let mut value = _source.at(_index)
    if value.isSome { _index = _index + 1 }
    return value
  }
  iterator { self }
}

impl<T> CanonicalIterator for MutableBagEntriesIterator<T> {
  type Item = (value: T, count: Int)
}

impl<T> CanonicalIterable for MutableBagEntriesView<T> {
  type Item = (value: T, count: Int)
  type Iterator = MutableBagEntriesIterator<T>
  iterator -> MutableBagEntriesIterator<T> { MutableBagEntriesIterator<T>.new(self) }
}
