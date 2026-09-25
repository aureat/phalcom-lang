from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class Map<K, V> is Collection {
  @class
  native new() -> Map<K, V>
  internal native rawSize -> Int
  internal native rawGet(_ key: Dynamic) -> Option<Dynamic>
  internal native rawPut(_ key: Dynamic, _ value: Dynamic) -> Dynamic
  internal native rawHas(_ key: Dynamic) -> Bool
  internal native rawRemove(_ key: Dynamic) -> Dynamic
  internal native rawKeyAt(_ index: Int) -> Dynamic
  internal native rawValueAt(_ index: Int) -> Dynamic
  size { self.rawSize }

  /// Renders entries in insertion order using each key and value's string form.
  toString {
    let s = "{"
    let i = 0
    while i < self.rawSize {
      if i > 0 {
        s = s + ", "
      }
      s = s + self.rawKeyAt(i).toString + ": " + self.rawValueAt(i).toString
      i = i + 1
    }
    return s + "}"
  }

  /// Returns the value for `k`, or `None` when no association exists.
  get(_ k) { self.rawGet(k) }

  /// Returns the value for `k`, raising when the key is absent.
  [_ k] {
    return self.get(k).match(
      some: |value| { value },
      none: || { KeyError.new("Map key not found").raise() }
    )
  }

  [_ key, default fallback] {
    return self.get(key).match(
      some: |value| { value },
      none: || { fallback }
    )
  }

  get(_ key, orElse block) {
    return self.get(key).match(
      some: |value| { value },
      none: || { block.call(key) }
    )
  }

  get(_ key, orPut block) {
    return self.get(key).match(
      some: |value| { value },
      none: || {
        let value = block.call(key)
        self.insert(value, for: key)
        value
      }
    )
  }

  /// Inserts `value` for `key` and returns the previous association.
  insert(_ value, for key) { self.rawPut(key, value) }

  /// Stores a value for `k` and returns the map.
  at(_ k, put) {
    self.rawPut(k, put)
    return self
  }

  /// Stores a value for `k` through the subscript protocol.
  [_ k]=(_ val) { self.rawPut(k, val) }

  /// Reports whether `k` has an association.
  contains(_ k) { self.rawHas(k) }

  /// Reports whether `k` has an association.
  includes(_ k) { self.contains(k) }

  /// Removes `k` and returns the map.
  remove(_ k) {
    self.rawRemove(k)
    return self
  }

  /// Removes all associations from the map.
  clear {
    while self.size > 0 {
      self.rawRemove(self.rawKeyAt(0))
    }
    return ()
  }

  /// Returns a live view of the map's keys in encounter order.
  keys -> MapKeysView<K, V> { MapKeysView.new(self) }

  /// Returns a live view of the map's values in encounter order.
  values -> MapValuesView<K, V> { MapValuesView.new(self) }

  /// Returns a live view of the map's entries in encounter order.
  entries -> MapEntriesView<K, V> { MapEntriesView.new(self) }

  /// Returns a fresh iterator over the map's entries.
  iterator -> MapIterator<K, V> { MapIterator<K, V>.new(self) }

  /// Builds a mutable map from a record's labels and values.
  @class
  from(_ record) {
    if record is not Record {
      throw ArgumentError.new("Map.from: argument must be a Record")
    }
    let result = Map<Dynamic, Dynamic>.new()
    if record is Record {
      let i = 0
      while i < record.rawSize {
        result.insert(record.rawValueAt(i), for: record.rawLabelAt(i))
        i = i + 1
      }
    }
    return result
  }

  /// Compares maps by key set and corresponding value equality.
  ==(_ other) {
    if other is Map {
      let same = self.size == other.size
      let i = 0
      while same and i < self.size {
        let key = self.rawKeyAt(i)
        same = other.get(key).match(
          some: |value| { self.rawValueAt(i) == value },
          none: || { false }
        )
        i = i + 1
      }
      return same
    }
    return false
  }

  /// Returns the inverse of structural map equality.
  !=(_ other) {
    return not (self == other)
  }
}

class MapIterator<K, V> {
  _source: Map<K, V>
  _index: Int
  _done: Bool

  @constructor
  new(_ source: Map<K, V>) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<(key: K, value: V)> {
    if _done { return None }
    if _index >= _source.size {
      _done = true
      return None
    }
    let key = _source.rawKeyAt(_index)
    let value = _source.rawValueAt(_index)
    _index = _index + 1
    Some((key: key, value: value))
  }

  iterator { self }
}

impl<K, V> CanonicalIterator for MapIterator<K, V> {
  type Item = (key: K, value: V)
}

impl<K, V> CanonicalIterable for Map<K, V> {
  type Item = (key: K, value: V)
  type Iterator = MapIterator<K, V>
  iterator -> MapIterator<K, V> { MapIterator<K, V>.new(self) }
}

/// Live view of map keys in encounter order.
class MapKeysView<K, V> is Collection {
  _map: Map<K, V>

  @constructor
  new(_ map: Map<K, V>) { _map = map }

  size { _map.size }

  at(_ index) { _map.rawKeyAt(index) }

  iterator -> MapKeysIterator<K, V> { MapKeysIterator<K, V>.new(self) }
}

/// Live view of map values in encounter order.
class MapValuesView<K, V> is Collection {
  _map: Map<K, V>

  @constructor
  new(_ map: Map<K, V>) { _map = map }

  size { _map.size }

  at(_ index) { _map.rawValueAt(index) }

  iterator -> MapValuesIterator<K, V> { MapValuesIterator<K, V>.new(self) }
}

/// Immutable key-value association used by entry views.
class Entry {
  @constructor
  new(_ key, _ value) {
    _key = key
    _value = value
  }

  key { _key }

  value { _value }
}

class MapEntriesView<K, V> is Collection {
  _map: Map<K, V>

  @constructor
  new(_ map: Map<K, V>) { _map = map }

  size { _map.size }

  at(_ index) { (key: _map.rawKeyAt(index), value: _map.rawValueAt(index)) }

  iterator -> MapEntriesIterator<K, V> { MapEntriesIterator<K, V>.new(self) }
}

class MapKeysIterator<K, V> {
  _source: MapKeysView<K, V>
  _index: Int
  _done: Bool

  @constructor
  new(_ source: MapKeysView<K, V>) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<K> {
    if _done { return None }
    if _index >= _source.size {
      _done = true
      return None
    }
    let value = _source.at(_index)
    _index = _index + 1
    Some(value)
  }

  iterator { self }
}

class MapValuesIterator<K, V> {
  _source: MapValuesView<K, V>
  _index: Int
  _done: Bool

  @constructor
  new(_ source: MapValuesView<K, V>) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<V> {
    if _done { return None }
    if _index >= _source.size {
      _done = true
      return None
    }
    let value = _source.at(_index)
    _index = _index + 1
    Some(value)
  }

  iterator { self }
}

class MapEntriesIterator<K, V> {
  _source: MapEntriesView<K, V>
  _index: Int
  _done: Bool

  @constructor
  new(_ source: MapEntriesView<K, V>) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<(key: K, value: V)> {
    if _done { return None }
    if _index >= _source.size {
      _done = true
      return None
    }
    let value = _source.at(_index)
    _index = _index + 1
    Some(value)
  }

  iterator { self }
}

impl<K, V> CanonicalIterator for MapKeysIterator<K, V> {
  type Item = K
}

impl<K, V> CanonicalIterable for MapKeysView<K, V> {
  type Item = K
  type Iterator = MapKeysIterator<K, V>
  iterator -> MapKeysIterator<K, V> { MapKeysIterator<K, V>.new(self) }
}

impl<K, V> CanonicalIterator for MapValuesIterator<K, V> {
  type Item = V
}

impl<K, V> CanonicalIterable for MapValuesView<K, V> {
  type Item = V
  type Iterator = MapValuesIterator<K, V>
  iterator -> MapValuesIterator<K, V> { MapValuesIterator<K, V>.new(self) }
}

impl<K, V> CanonicalIterator for MapEntriesIterator<K, V> {
  type Item = (key: K, value: V)
}

impl<K, V> CanonicalIterable for MapEntriesView<K, V> {
  type Item = (key: K, value: V)
  type Iterator = MapEntriesIterator<K, V>
  iterator -> MapEntriesIterator<K, V> { MapEntriesIterator<K, V>.new(self) }
}
