from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class Set<T> is Collection {
  @class
  native new() -> Set<T>
  internal native rawSize -> Int
  internal native rawAdd(_ value: Dynamic) -> Dynamic
  internal native rawHas(_ value: Dynamic) -> Bool
  internal native rawRemove(_ value: Dynamic) -> Dynamic
  internal native rawAt(_ index: Int) -> Dynamic
  size { self.rawSize }

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Set` rendering exactly — `Set(a, b)`, `Set()` when empty. Derived
  // over `size_`/`at_`; each element renders via its OWN `toString`.
  toString {
    let s = "Set("
    let i = 0
    while (i < self.rawSize) {
      s = s + (i > 0).ifTrue(|| { ", " }, ifFalse: || { "" })
      s = s + self.rawAt(i).toString
      i = i + 1
    }
    return s + ")"
  }

  add(_ v) {
    self.rawAdd(v)
    return self
  }

  contains(_ v) { self.rawHas(v) }
  includes(_ v) { self.contains(v) }

  remove(_ v) {
    self.rawRemove(v)
    return self
  }

  // Positional read in insertion order — not in the map-and-set.md selector
  // table, but a direct, zero-floor-cost derivation over at_ that the
  // U-CORE-5 conformance harness (collection-protocol.md §2) needs, and a
  // natural extension of the sequence protocol every collection instantiates.
  at(_ i) { self.rawAt(i) }

  iterator -> SetIterator<T> { SetIterator<T>.new(self) }

  // Structural equality: same members, order-independent. Same-size plus
  // "every element of self is in other" is sufficient since neither set
  // holds duplicates (add_ is idempotent).
  ==(_ other) {
    if (other.is(Set)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        same = other.includes(self.rawAt(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  !=(_ other) {
    return not (self == other)
  }
}

class SetIterator<T> {
  _source: Set<T>
  _index: Int
  _done: Bool

  @constructor
  new(_ source: Set<T>) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<T> {
    if (_done) { return None }
    if (_index >= _source.size) {
      _done = true
      return None
    }
    let value = _source.at(_index)
    _index = _index + 1
    Some(value)
  }

  iterator { self }
}

impl<T> CanonicalIterator for SetIterator<T> {
  type Item = T
}

impl<T> CanonicalIterable for Set<T> {
  type Item = T
  type Iterator = SetIterator<T>
  iterator -> SetIterator<T> { SetIterator<T>.new(self) }
}
