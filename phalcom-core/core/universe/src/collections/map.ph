from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

@native
class Map<K, V> is Collection {
  @class @native new() -> Map<K, V>
  @internal @native _$size -> Int
  @internal @native _$get(_ key: Dynamic) -> Option<Dynamic>
  @internal @native _$put(_ key: Dynamic, _ value: Dynamic) -> Dynamic
  @internal @native _$has(_ key: Dynamic) -> Bool
  @internal @native _$remove(_ key: Dynamic) -> Dynamic
  @internal @native _$keyAt(_ index: Int) -> Dynamic
  @internal @native _$valueAt(_ index: Int) -> Dynamic
  size { self._$size }

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Map` rendering exactly — `{k: v, k2: v2}`, `{}` when empty — so the
  // `.toString` message and the native render agree. Derived over the floor
  // (`size_`/`keyAt_`/`valueAt_`), not a primitive: ADR-0019's default answer to
  // "add a primitive" is no, and this is expressible.
  //
  // Each key and value renders via its OWN `toString` — which the native path
  // cannot do. That is the point of CB-1: once `\(…)` sends `toString`, this is
  // the path it takes.
  toString {
    let s = "{"
    let i = 0
    while (i < self._$size) {
      s = s + (i > 0).ifTrue(|| { ", " }, ifFalse: || { "" })
      s = s + self._$keyAt(i).toString + ": " + self._$valueAt(i).toString
      i = i + 1
    }
    return s + "}"
  }

  // Safe association lookup: Some(value) on hit, None on absence.
  get(_ k) { self._$get(k) }

  // Strict association lookup. Do one lookup so a stored None remains a
  // value rather than being confused with an absent key.
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

  // Explicit insert returns the previous value when replacing an association.
  insert(_ value, for key) { self._$put(key, value) }

  // Legacy mutation spelling retained only while B.3 still lowers association
  // literals into chained sends. `get` and `[]` are the lookup surface.
  at(_ k, put) {
    self._$put(k, put)
    return self
  }

  // `m[k] = v` shares insert's key identity and encounter-order semantics.
  [_ k]=(_ val) { self._$put(k, val) }

  contains(_ k) { self._$has(k) }
  includes(_ k) { self.contains(k) }

  // Removes an association. The raw primitive returns its former value, but
  // the public mutable-collection protocol is chainable.
  remove(_ k) {
    self._$remove(k)
  }

  clear {
    while (self.size > 0) {
      self._$remove(self._$keyAt(0))
    }
    return ()
  }

  // Lightweight live encounter-order views. They retain the Map and read its
  // current slots; they never copy associations into a List.
  keys -> MapKeysView<K, V> { MapKeysView.new(self) }

  values -> MapValuesView<K, V> { MapValuesView.new(self) }

  entries -> MapEntriesView<K, V> { MapEntriesView.new(self) }

  iterator -> MapIterator<K, V> { MapIterator<K, V>.new(self) }

  // Copies a positive Record's Symbol labels and values into a fresh mutable
  // Map. `#{}` canonicalizes to Unit, which represents the empty Record form.
  @class
  from(_ record) {
    if ((not record.is(Record)) and (not record.is(Unit))) {
      throw ArgumentError.new("Map.from: argument must be a Record or Unit")
    }
    let result = Map<Dynamic, Dynamic>.new()
    if (record.is(Record)) {
      let i = 0
      while (i < record._$size) {
        result.insert(record._$valueAt(i), for: record._$labelAt(i))
        i = i + 1
      }
    }
    return result
  }

  // Structural equality: same key set, pairwise-== values (order-independent
  // over keys — `includes`/`get` do the membership + value work, not raw
  // index comparison). Guarded by `isA(Map)` so a non-Map is simply unequal.
  ==(_ other) {
    if (other.is(Map)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        let k = self._$keyAt(i)
        same = other.get(k).match(
          some: |value| { self._$valueAt(i) == value },
          none: || { false }
        )
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  // MUST route through == (the ==/!= decoupling hazard) — Object#!= negates
  // identity, not this structural ==.
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
    if (_done) { return None }
    if (_index >= _source.size) {
      _done = true
      return None
    }
    let key = _source._$keyAt(_index)
    let value = _source._$valueAt(_index)
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


// Map projections are ordinary retained-source Iterable views. They inherit
// the generic cursor walk and deliberately leave active-iteration mutation
// behavior unspecified.
class MapKeysView<K, V> is Collection {
  _map: Map<K, V>

  @constructor
  new(_ map: Map<K, V>) { _map = map }

  size { _map.size }

  at(_ index) { _map._$keyAt(index) }

  iterator -> MapKeysIterator<K, V> { MapKeysIterator<K, V>.new(self) }
}

class MapValuesView<K, V> is Collection {
  _map: Map<K, V>

  @constructor
  new(_ map: Map<K, V>) { _map = map }

  size { _map.size }

  at(_ index) { _map._$valueAt(index) }

  iterator -> MapValuesIterator<K, V> { MapValuesIterator<K, V>.new(self) }
}

// Immutable-by-surface association value for MapEntriesView. No setters or
// value-object equality/hash protocol are part of this phase.
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

  at(_ index) { (key: _map._$keyAt(index), value: _map._$valueAt(index)) }

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
