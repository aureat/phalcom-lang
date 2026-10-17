from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class Bijection<K, V> is Collection {
  @class
  native new() -> Bijection<K, V>
  internal native rawSize -> Int
  internal native rawGet(_ key: Dynamic) -> Option<V>
  internal native rawAssociate(_ key: Dynamic, _ value: Dynamic) -> Bijection<K, V>
  internal native rawRemoveKey(_ key: Dynamic) -> Bijection<K, V>
  internal native rawInverse -> Bijection<V, K>
  internal native rawKeys -> Array<K>

  size { self.rawSize }
  isEmpty { self.size == 0 }

  get(_ key: K) -> Option<V> { self.rawGet(key) }
  at(_ key: K) -> Option<V> { self.get(key) }
  containsKey(_ key: K) -> Bool { self.get(key).isSome }

  keyFor(_ value: V) -> Option<K> { self.inverse.get(value) }
  containsValue(_ value: V) -> Bool { self.keyFor(value).isSome }

  associating(_ key: K, _ value: V) -> Bijection<K, V> {
    self.rawAssociate(key, value)
  }

  removingKey(_ key: K) -> Bijection<K, V> {
    self.rawRemoveKey(key)
  }

  removingValue(_ value: V) -> Bijection<K, V> {
    let k = self.keyFor(value)
    if k.isSome {
      let actualKey = k.unwrapOr(None)
      return self.removingKey(actualKey)
    }
    return self
  }

  inverse -> Bijection<V, K> {
    self.rawInverse
  }

  entries -> List<(key: K, value: V)> {
    let keys = self.rawKeys
    let mut out = List<(key: K, value: V)>.new()
    let mut i = 0
    while i < keys.size {
      let k = keys.at(i)
      let optVal = self.get(k)
      let v = optVal.unwrapOr(None)
      out.append((key: k, value: v))
      i = i + 1
    }
    return out
  }

  iterator -> BijectionIterator<K, V> {
    BijectionIterator<K, V>.new(self)
  }

  ==(_ other) {
    if not (other is Bijection) { return false }
    if self.size != other.size { return false }
    let keys = self.rawKeys
    let mut i = 0
    while i < keys.size {
      let k = keys.at(i)
      let v1 = self.get(k)
      let v2 = other.get(k)
      if v1 != v2 {
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
      let optV = self.get(k)
      let v = optV.unwrapOr(None)
      acc = acc ^ (k.hash * 31 + v.hash)
      i = i + 1
    }
    return acc
  }
}

class BijectionIterator<K, V> {
  mut _keys: Array<K>
  mut _bijection: Bijection<K, V>
  mut _index: Int

  @constructor
  new(_ bijection: Bijection<K, V>) {
    _bijection = bijection
    _keys = bijection.rawKeys
    _index = 0
  }

  next -> Option<(key: K, value: V)> {
    if _index < _keys.size {
      let k = _keys[_index]
      _index = _index + 1
      let optVal = _bijection.get(k)
      let v = optVal.unwrapOr(None)
      return Some((key: k, value: v))
    }
    return None
  }
}

export Bijection, BijectionIterator
