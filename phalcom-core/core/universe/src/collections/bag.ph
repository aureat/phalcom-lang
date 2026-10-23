from universe.collections.iteration import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class Bag<T> is Collection {
  @class
  native new() -> Bag<T>
  internal native rawSize -> Int
  internal native rawDistinctCount -> Int
  internal native rawCount(_ value: Dynamic) -> Int
  internal native rawAdding(_ value: Dynamic, _ count: Int) -> Bag<T>
  internal native rawRemoving(_ value: Dynamic, _ count: Int) -> Bag<T>
  internal native rawKeys -> Array<T>

  size { self.rawSize }
  totalCount { self.rawSize }
  distinctCount { self.rawDistinctCount }
  count(_ value: T) -> Int { self.rawCount(value) }
  contains(_ value: T) -> Bool { self.count(value) > 0 }

  adding(_ value: T) -> Bag<T> { self.rawAdding(value, 1) }
  adding(_ value: T, count n: Int) -> Bag<T> { self.rawAdding(value, n) }
  removing(_ value: T) -> Bag<T> { self.rawRemoving(value, 1) }
  removing(_ value: T, count n: Int) -> Bag<T> { self.rawRemoving(value, n) }

  entries -> List<(value: T, count: Int)> {
    let keys = self.rawKeys
    let mut out = List<(value: T, count: Int)>.new()
    let mut i = 0
    while i < keys.size {
      let k = keys.at(i)
      out.append((value: k, count: self.rawCount(k)))
      i = i + 1
    }
    return out
  }

  iterator -> BagIterator<T> { BagIterator<T>.new(self) }

  +(_ other: Bag<T>) -> Bag<T> {
    let otherKeys = other.rawKeys
    let mut result = self
    let mut i = 0
    while i < otherKeys.size {
      let k = otherKeys.at(i)
      let c = other.rawCount(k)
      result = result.adding(k, count: c)
      i = i + 1
    }
    return result
  }

  union(_ other: Bag<T>) -> Bag<T> {
    let otherKeys = other.rawKeys
    let mut result = self
    let mut i = 0
    while i < otherKeys.size {
      let k = otherKeys.at(i)
      let cOther = other.rawCount(k)
      let cSelf = self.rawCount(k)
      if cOther > cSelf {
        let diff = cOther - cSelf
        result = result.adding(k, count: diff)
      }
      i = i + 1
    }
    return result
  }

  intersection(_ other: Bag<T>) -> Bag<T> {
    let selfKeys = self.rawKeys
    let mut result = Bag<T>.new()
    let mut i = 0
    while i < selfKeys.size {
      let k = selfKeys.at(i)
      let cSelf = self.rawCount(k)
      let cOther = other.rawCount(k)
      let common = if cSelf < cOther { cSelf } else { cOther }
      if common > 0 {
        result = result.adding(k, count: common)
      }
      i = i + 1
    }
    return result
  }

  difference(_ other: Bag<T>) -> Bag<T> {
    let otherKeys = other.rawKeys
    let mut result = self
    let mut i = 0
    while i < otherKeys.size {
      let k = otherKeys.at(i)
      let cOther = other.rawCount(k)
      result = result.removing(k, count: cOther)
      i = i + 1
    }
    return result
  }

  @class
  from<I>(_ source: I) -> Bag<T>
  where I |= CanonicalIterable, <I as CanonicalIterable>::Item == T
  {
    let mut result = Bag<T>.new()
    let mut it = source.iterator
    let mut step = it.next
    while step.isSome {
      let value = step.match(some: |item| { item }, none: || { None })
      result = result.adding(value)
      step = it.next
    }
    return result
  }

  ==(_ other) {
    if not (other is Bag) { return false }
    if self.distinctCount != other.distinctCount or self.size != other.size {
      return false
    }
    let keys = self.rawKeys
    let mut i = 0
    while i < keys.size {
      let k = keys.at(i)
      if self.rawCount(k) != other.rawCount(k) {
        return false
      }
      i = i + 1
    }
    return true
  }

  hash {
    let keys = self.rawKeys
    let mut acc = 0
    let mut i = 0
    while i < keys.size {
      let k = keys.at(i)
      let c = self.rawCount(k)
      acc = acc ^ (k.hash * 31 + c.hash)
      i = i + 1
    }
    return acc
  }
}

class BagIterator<T> {
  mut _keys: Array<T>
  mut _bag: Bag<T>
  mut _keyIndex: Int
  mut _countRemaining: Int
  mut _currentKey: Option<T>

  @constructor
  new(_ bag: Bag<T>) {
    _bag = bag
    _keys = bag.rawKeys
    _keyIndex = 0
    _countRemaining = 0
    _currentKey = None
  }

  next -> Option<T> {
    while _countRemaining <= 0 and _keyIndex < _keys.size {
      let k = _keys[_keyIndex]
      _keyIndex = _keyIndex + 1
      _currentKey = Some(k)
      _countRemaining = _bag.rawCount(k)
    }
    if _countRemaining > 0 {
      _countRemaining = _countRemaining - 1
      return _currentKey
    }
    return None
  }
}

export Bag, BagIterator
