from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class IndexedDict<K, V> is Collection {
  @class
  native new() -> IndexedDict<K, V>
  internal native rawSize -> Int
  internal native rawPut(_ key: Dynamic, _ value: Dynamic) -> Option<V>
  internal native rawKeyAt(_ index: Int) -> K
  internal native rawValueAt(_ index: Int) -> V
  internal native rawInsertAt(_ key: Dynamic, _ value: Dynamic, _ index: Int) -> Bool
  internal native rawMove(_ sourceIndex: Int, _ destinationIndex: Int) -> Bool
  internal native rawRemoveAt(_ index: Int) -> Bool
  internal native rawIndexOfKey(_ key: Dynamic) -> Option<Int>
  internal native rawClear -> Unit

  size { self.rawSize }
  get(_ key: K) -> Option<V> {
    match self.rawIndexOfKey(key) {
      Option::Some(index) => Option::Some(self.rawValueAt(index))
      Option::None => Option::None
    }
  }
  [_ key: K] -> V {
    return self.get(key).match(
      some: |value| { value },
      none: || { KeyError.new("IndexedDict key not found").raise() }
    )
  }
  contains(_ key: K) -> Bool { self.rawIndexOfKey(key).isSome }

  insert(_ value: V, for key: K) -> Option<V> { self.rawPut(key, value) }
  remove(_ key: K) -> IndexedDict<K, V> {
    let mut position = self.rawIndexOfKey(key)
    match position {
      Option::Some(index) => { self.rawRemoveAt(index) }
      Option::None => { false }
    }
    return self
  }
  clear -> Unit { self.rawClear }

  entryAt(_ index: Int) -> Option<(key: K, value: V)> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized >= self.size { return None }
    return Some((key: self.rawKeyAt(normalized), value: self.rawValueAt(normalized)))
  }

  keyAt(_ index: Int) -> Option<K> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized >= self.size { return None }
    return Some(self.rawKeyAt(normalized))
  }

  valueAt(_ index: Int) -> Option<V> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized >= self.size { return None }
    return Some(self.rawValueAt(normalized))
  }

  indexOfKey(_ key: K) -> Option<Int> { self.rawIndexOfKey(key) }

  insert(_ entry: (key: K, value: V), at index: Int) -> Result<Unit, Error> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized > self.size {
      return Result::Error(IndexError.new("IndexedDict insertion index out of range"))
    }
    if not self.rawInsertAt(entry.key, entry.value, normalized) {
      let mut duplicateKey: Dynamic = entry.key
      let mut duplicateError: DuplicateKeyError = DuplicateKeyError.new(duplicateKey)
      let mut result: Result<Unit, Error> = Result::Error(duplicateError)
      return result
    }
    return Result::Ok(())
  }

  move(_ sourceIndex: Int, to index: Int) -> Result<Unit, Error> {
    let mut source = sourceIndex
    let mut destination = index
    if source < 0 { source = self.size + source }
    if destination < 0 { destination = self.size + destination }
    if source < 0 or source >= self.size or destination < 0 or destination >= self.size {
      return Result::Error(IndexError.new("IndexedDict move index out of range"))
    }
    if source == destination { return Result::Ok(()) }
    if self.rawMove(source, destination) { return Result::Ok(()) }
    return Result::Error(IndexError.new("IndexedDict move index out of range"))
  }

  removeAt(_ index: Int) -> Result<(key: K, value: V), Error> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized >= self.size {
      return Result::Error(IndexError.new("IndexedDict removal index out of range"))
    }
    let mut key = self.rawKeyAt(normalized)
    let mut value = self.rawValueAt(normalized)
    if not self.rawRemoveAt(normalized) {
      return Result::Error(IndexError.new("IndexedDict removal index out of range"))
    }
    return Result::Ok((key: key, value: value))
  }

  iterator -> IndexedDictIterator<K, V> { IndexedDictIterator<K, V>.new(self) }

  @class
  from<I>(_ source: I) -> IndexedDict<K, V>
  where I |= CanonicalIterable, <I as CanonicalIterable>::Item == (key: K, value: V)
  {
    let mut result = IndexedDict<K, V>.new()
    let mut it = source.iterator
    let mut step = it.next
    while step.isSome {
      let mut entry = step.match(some: |item| { item }, none: || { None })
      if not result.rawInsertAt(entry.key, entry.value, result.size) {
        let mut duplicateKey: Dynamic = entry.key
        DuplicateKeyError.new(duplicateKey).raise()
      }
      step = it.next
    }
    return result
  }

  ==(_ other) {
    if not (other is IndexedDict) { return false }
    let mut left = List<Dynamic>.new()
    let mut i = 0
    while i < self.size {
      left.append((key: self.rawKeyAt(i), value: self.rawValueAt(i)))
      i = i + 1
    }
    let mut right = List<Dynamic>.new()
    i = 0
    while i < other.size {
      right.append((key: other.rawKeyAt(i), value: other.rawValueAt(i)))
      i = i + 1
    }
    if left.size != right.size { return false }
    let mut same = true
    i = 0
    while same and i < left.size {
      let mut a = left.at(i)
      let mut b = right.at(i)
      let mut keyEqual = a.key == b.key
      if not keyEqual and a.key is Float and b.key is Float {
        keyEqual = a.key.isNaN and b.key.isNaN
      }
      same = keyEqual and a.value == b.value
      i = i + 1
    }
    return same
  }
}

class IndexedDictIterator<K, V> {
  mut _source: IndexedDict<K, V>
  mut _index: Int

  @constructor
  new(_ source: IndexedDict<K, V>) {
    _source = source
    _index = 0
  }

  next -> Option<(key: K, value: V)> {
    let mut entry = _source.entryAt(_index)
    if entry.isSome { _index = _index + 1 }
    return entry
  }

  iterator { self }
}

impl<K, V> CanonicalIterator for IndexedDictIterator<K, V> {
  type Item = (key: K, value: V)
}

impl<K, V> CanonicalIterable for IndexedDict<K, V> {
  type Item = (key: K, value: V)
  type Iterator = IndexedDictIterator<K, V>
  iterator -> IndexedDictIterator<K, V> { IndexedDictIterator<K, V>.new(self) }
}
