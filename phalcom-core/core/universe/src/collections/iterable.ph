@native
class Iterable is Object {
  // Generic index-cursor walk over `self.size` (ADR-0048 §1/§3). A subclass whose
  // cursor is not a 0..size index (none in-kernel today) overrides this.
  iterate(_ cursor) {
    const next = (cursor == None).ifTrue(|| { 0 }, ifFalse: || { cursor + 1 })
    return (next < self.size).ifTrue(|| { next }, ifFalse: || { None })
  }

  each(_ f) {
    for x in self {
      f.call(x)
    }
    return ()
  }

  // map/filter/reduce/includes walk `iterate`/`iteratorValue` DIRECTLY, not
  // `self.each`, so generic operations remain protocol-driven and independent
  // of any receiver-specific traversal convenience.
  // Concrete Iterable transforms are eager. Lazy transforms live behind `.iter`,
  // so the receiver makes evaluation timing visible at every call site.
  map(_ f) {
    let result = List.new()
    let c = self.iterate(None)
    while (c != None) {
      result.append(f.call(self.iteratorValue(c)))
      c = self.iterate(c)
    }
    return result
  }

  map(indexed f) {
    let result = List.new()
    let index = 0
    let c = self.iterate(None)
    while (c != None) {
      result.append(f.call(index, self.iteratorValue(c)))
      index = index + 1
      c = self.iterate(c)
    }
    return result
  }

  each(indexed f) {
    let index = 0
    let c = self.iterate(None)
    while (c != None) {
      f.call(index, self.iteratorValue(c))
      index = index + 1
      c = self.iterate(c)
    }
    return ()
  }

  flatMap(_ f) {
    let result = List.new()
    let outer = self.iterate(None)
    while (outer != None) {
      let inner = f.call(self.iteratorValue(outer))
      let ic = inner.iterate(None)
      while (ic != None) {
        result.append(inner.iteratorValue(ic))
        ic = inner.iterate(ic)
      }
      outer = self.iterate(outer)
    }
    return result
  }

  filter(_ pred) {
    let result = List.new()
    let c = self.iterate(None)
    while (c != None) {
      let x = self.iteratorValue(c)
      pred.call(x).ifTrue(|| { result.append(x) }, ifFalse: || { None })
      c = self.iterate(c)
    }
    return result
  }

  contains(_ x) {
    let found = false
    let c = self.iterate(None)
    while (c != None) {
      (self.iteratorValue(c) == x).ifTrue(|| { found = true }, ifFalse: || { None })
      c = self.iterate(c)
    }
    return found
  }

  includes(_ x) { self.contains(x) }

  isEmpty { self.size == 0 }

  indexed { IndexedIterable.new(self) }

  all(where f) {
    for x in self {
      f.call(x).ifFalse || { return false }
    }
    return true
  }

  any(where f) {
    for x in self {
      f.call(x).ifTrue || { return true }
    }
    return false
  }

  none(where f) {
    for x in self {
      f.call(x).ifTrue || { return false }
    }
    return true
  }

  count {
    let n = 0
    for x in self { n = n + 1 }
    return n
  }

  count(where f) {
    let n = 0
    for x in self { f.call(x).ifTrue || { n = n + 1 } }
    return n
  }

  find(where f) {
    for x in self {
      f.call(x).ifTrue || { return Some(x) }
    }
    return None
  }

  index(where f) {
    let index = 0
    for x in self {
      f.call(x).ifTrue || { return Some(index) }
      index = index + 1
    }
    return None
  }

  join { self.join("") }

  join(_ sep) {
    // Note: O(N²) allocation cost due to naive string concatenation. Each `result = result + ...`
    // allocates a new string and copies all prior content. For N elements, total work is ~N²/2.
    // This is acceptable for Phalcom's interpreter domain (collections stay small) but users
    // joining large collections should be aware of this limitation.
    let first = true
    let result = ""
    for x in self {
      first.ifFalse || { result = result + sep }
      first = false
      result = result + x.toString
    }
    return result
  }

  // D.1 splits explicit-initial accumulation from no-initial reduction.
  // Labels are selector identity, so neither historical positional form is
  // retained as an alias.
  fold(initial initial, using f) {
    let acc = initial
    for x in self {
      acc = f.call(acc, x)
    }
    return acc
  }

  reduce(using f) {
    let c = self.iterate(None)
    if (c == None) { return None }

    let acc = self.iteratorValue(c)
    c = self.iterate(c)
    while (c != None) {
      acc = f.call(acc, self.iteratorValue(c))
      c = self.iterate(c)
    }
    return Some(acc)
  }

  group(by block) {
    let result = Map.new()
    for x in self {
      let key = block.call(x)
      let list = result.get(key).match(
        some: |list| { list },
        none: || {
          let new_list = List.new()
          result.insert(new_list, for: key)
          new_list
        }
      )
      list.append(x)
    }
    return result
  }

  partition(where predicate) {
    let accepted = List.new()
    let rejected = List.new()
    for x in self {
      predicate.call(x).ifTrue(|| { accepted.append(x) }, ifFalse: || { rejected.append(x) })
    }
    return (accepted, rejected)
  }

  toSet {
    let result = Set.new()
    for x in self {
      result.add(x)
    }
    return result
  }

  toMap {
    let result = Map.new()
    for entry in self {
      let key = entry.key
      if (result.includes(key)) {
        return Err.new(DuplicateKeyError.new(key))
      }
      result.insert(entry.value, for: key)
    }
    return Ok.new(result)
  }

  toMap(merging block) {
    let result = Map.new()
    for entry in self {
      let key = entry.key
      let val = entry.value
      result.get(key).match(
        some: |existingVal| {
          let merged = block.call(existingVal, val)
          result.insert(merged, for: key)
        },
        none: || {
          result.insert(val, for: key)
        }
      )
    }
    return result
  }

  toList {
    let result = List.new()
    for x in self { result.append(x) }
    return result
  }

  iter { SourceIterator.new(self) }
}

// First-class value-level view for ordinal iteration. The source cursor is
// opaque; the view carries it beside its own zero-based ordinal.
class IndexedIterable is Iterable {
  @constructor
  new(_ source) { _source = source }

  size { _source.size }

  iterate(_ cursor) {
    let source_cursor = (cursor == None).ifTrue(|| { None }, ifFalse: || { cursor.at(0) })
    let next = _source.iterate(source_cursor)
    if (next == None) { return None }
    let ordinal = (cursor == None).ifTrue(|| { 0 }, ifFalse: || { cursor.at(1) + 1 })
    (next, ordinal)
  }

  iteratorValue(_ cursor) { (cursor.at(1), _source.iteratorValue(cursor.at(0))) }
}

// Strict lockstep view used by Tuple#zipped. Every source must expose the
// same size; a mismatch raises before a truncated result can escape.
class ZippedIterable is Iterable {
  @constructor
  new(_ sources) { _sources = sources }

  size {
    if (_sources.size == 0) { return 0 }
    let expected = _sources.at(0).size
    let i = 1
    while (i < _sources.size) {
      if (_sources.at(i).size != expected) {
        throw ArgumentError.new("zipped iterables must have equal lengths")
      }
      i = i + 1
    }
    expected
  }

  iterate(_ cursor) {
    let next = List.new()
    let any_live = false
    let any_none = false
    let i = 0
    while (i < _sources.size) {
      let source = _sources.at(i)
      let previous = (cursor == None).ifTrue(|| { None }, ifFalse: || { cursor.at(i) })
      let candidate = source.iterate(previous)
      next.append(candidate)
      if (candidate == None) {
        any_none = true
      } else {
        any_live = true
      }
      i = i + 1
    }
    if (any_none and any_live) {
      throw ArgumentError.new("zipped iterables must end together")
    }
    if (any_none) { return None }
    Tuple._$fromList(next)
  }

  iteratorValue(_ cursor) {
    let values = List.new()
    let i = 0
    while (i < _sources.size) {
      values.append(_sources.at(i).iteratorValue(cursor.at(i)))
      i = i + 1
    }
    Tuple._$fromList(values)
  }
}

// Stateless lazy pipeline root. Traversal state is carried only in cursors,
// allowing one pipeline instance to be traversed independently and repeatedly.
class Iterator is Iterable {
  iter { self }
  map(_ f) { MapIterator.new(self, f) }
  filter(_ pred) { FilterIterator.new(self, pred) }
  flatMap(_ f) { FlatMapIterator.new(self, f) }
  skip(_ n) { SkipIterator.new(self, n) }
  take(_ n) { TakeIterator.new(self, n) }
  takeWhile(_ pred) { TakeWhileIterator.new(self, pred) }
}


// cursor protocol; no stage stores traversal state on its instance.
class SourceIterator is Iterator {
  @constructor
  new(_ source) {
    _source = source
  }
  iterate(_ cursor) { _source.iterate(cursor) }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor) }
}

class MapIterator is Iterator {
  @constructor
  new(_ source, _ f) {
    _source = source
    _f = f
  }
  iterate(_ cursor) { _source.iterate(cursor) }
  iteratorValue(_ cursor) { _f.call(_source.iteratorValue(cursor)) }
}

class FilterIterator is Iterator {
  @constructor
  new(_ source, _ pred) {
    _source = source
    _pred = pred
  }
  iterate(_ cursor) {
    let cur = _source.iterate(cursor)
    while (cur != None) {
      if (_pred.call(_source.iteratorValue(cur))) { return cur }
      cur = _source.iterate(cur)
    }
    return None
  }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor) }
}

class SkipIterator is Iterator {
  @constructor
  new(_ source, _ n) {
    (n.is(Number) and n >= 0 and n % 1 == 0).ifFalse || {
      throw ArgumentError.new("skip: n must be a non-negative integer")
    }
    _source = source
    _n = n
  }
  iterate(_ cursor) {
    if (cursor != None) { return _source.iterate(cursor) }
    let cur = _source.iterate(None)
    let rem = _n
    while (cur != None and rem > 0) {
      cur = _source.iterate(cur)
      rem = rem - 1
    }
    return cur
  }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor) }
}

class TakeIterator is Iterator {
  @constructor
  new(_ source, _ n) {
    (n.is(Number) and n >= 0 and n % 1 == 0).ifFalse || {
      throw ArgumentError.new("take: n must be a non-negative integer")
    }
    _source = source
    _n = n
  }
  iterate(_ cursor) {
    if (_n == 0) { return None }
    if (cursor == None) {
      let up = _source.iterate(None)
      if (up == None) { return None }
      return (up, 1)
    }
    let up = cursor.at(0)
    let yielded = cursor.at(1)
    if (yielded >= _n) { return None }
    let next = _source.iterate(up)
    if (next == None) { return None }
    return (next, yielded + 1)
  }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor.at(0)) }
}

class TakeWhileIterator is Iterator {
  @constructor
  new(_ source, _ pred) { _source = source; _pred = pred }
  iterate(_ cursor) {
    let cand = _source.iterate(cursor)
    if (cand == None) { return None }
    if (_pred.call(_source.iteratorValue(cand))) { return cand }
    return None
  }
  iteratorValue(_ cursor) { _source.iteratorValue(cursor) }
}

class FlatMapIterator is Iterator {
  @constructor
  new(_ source, _ f) { _source = source; _f = f }

  @private
  seekFromOuter(_ outerCursor) {
    let oc = outerCursor
    while (oc != None) {
      let inner = _f.call(_source.iteratorValue(oc))
      let ic = inner.iterate(None)
      if (ic != None) { return (oc, inner, ic) }
      oc = _source.iterate(oc)
    }
    return None
  }

  iterate(_ cursor) {
    if (cursor == None) { return self.seekFromOuter(_source.iterate(None)) }
    let outer = cursor.at(0)
    let inner = cursor.at(1)
    let ic = cursor.at(2)
    let nextIc = inner.iterate(ic)
    if (nextIc != None) { return (outer, inner, nextIc) }
    return self.seekFromOuter(_source.iterate(outer))
  }
  iteratorValue(_ cursor) { cursor.at(1).iteratorValue(cursor.at(2)) }
}
