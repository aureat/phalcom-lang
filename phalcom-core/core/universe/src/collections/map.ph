class Map {
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
  [_ k]=(put val) { self._$put(k, val) }

  includes(_ k) { self._$has(k) }

  // Removes an association. The raw primitive returns its former value, but
  // the public mutable-collection protocol is chainable.
  remove(_ k) {
    self._$remove(k)
    return self
  }

  clear {
    while (self.size > 0) {
      self._$remove(self._$keyAt(0))
    }
    return ()
  }

  // Lightweight live encounter-order views. They retain the Map and read its
  // current slots; they never copy associations into a List.
  keys { MapKeysView.new(self) }

  values { MapValuesView.new(self) }

  entries { MapEntriesView.new(self) }

  // Copies a positive Record's Symbol labels and values into a fresh mutable
  // Map. `#{}` canonicalizes to Unit, which represents the empty Record form.
  @class
  from(_ record) {
    if ((not record.is(Record)) and (not record.is(Unit))) {
      throw ArgumentError.new("Map.from: argument must be a Record or Unit")
    }
    let result = Map.new()
    if (record.is(Record)) {
      let i = 0
      while (i < record._$size) {
        result.insert(record._$valueAt(i), for: record._$labelAt(i))
        i = i + 1
      }
    }
    return result
  }

  // DEC-CT-E: the cursor value `iteratorValue` yields is the KEY (both Map and Set yield keys).
  // Pair traversal uses `entries.each`, not a receiver-specific callback arity.
  iteratorValue(_ cursor) { self._$keyAt(cursor) }

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



// Map projections are ordinary retained-source Iterable views. They inherit
// the generic cursor walk and deliberately leave active-iteration mutation
// behavior unspecified.
class MapKeysView is Iterable {
  @constructor
  new(_ map) { _map = map }

  size { _map.size }

  iteratorValue(_ cursor) { _map._$keyAt(cursor) }
}

class MapValuesView is Iterable {
  @constructor
  new(_ map) { _map = map }

  size { _map.size }

  iteratorValue(_ cursor) { _map._$valueAt(cursor) }
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

class MapEntriesView is Iterable {
  @constructor
  new(_ map) { _map = map }

  size { _map.size }

  iteratorValue(_ cursor) { Entry.new(_map._$keyAt(cursor), _map._$valueAt(cursor)) }
}
