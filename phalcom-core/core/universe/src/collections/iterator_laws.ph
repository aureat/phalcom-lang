@!documentation("Canonical Iterator and Iterable trait laws.")

trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }

  map(_ f) { MappedIterator.new(self, f) }
  filter(_ pred) { FilterIterator.new(self, pred) }
  filterMap(_ f) { FilterMapIterator.new(self, f) }
  flatMap(_ f) { FlatMapIterator.new(self, f) }
  flatten { FlattenIterator.new(self) }
  chain(_ other) { ChainIterator.new(self, other) }
  skip(_ n) { SkipIterator.new(self, n) }
  skipWhile(_ pred) { SkipWhileIterator.new(self, pred) }
  take(_ n) { TakeIterator.new(self, n) }
  takeWhile(_ pred) { TakeWhileIterator.new(self, pred) }
  mapWhile(_ f) { MapWhileIterator.new(self, f) }
  inspect(_ f) { InspectIterator.new(self, f) }
  zip(_ other) { StrictZipIterator.new(self, other) }
}

trait Iterable
  where
    Self::Iterator |= Iterator,
    <Self::Iterator as Iterator>::Item == Self::Item
{
  type Item
  type Iterator

  iterator -> Self::Iterator

  iter { self.iterator }

  indexed { IndexedIterator.new(self.iterator) }

  each(_ f) {
    let it = self.iterator
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      f.call(val)
      step = it.next
    }
    return ()
  }

  each(indexed f) {
    let it = self.iterator
    let step = it.next
    let index = 0
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      f.call(index, val)
      index = index + 1
      step = it.next
    }
    return ()
  }

  contains(_ x) {
    let it = self.iterator
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (val == x) { return true }
      step = it.next
    }
    return false
  }

  includes(_ x) { self.contains(x) }

  count {
    let it = self.iterator
    let n = 0
    let step = it.next
    while (step.isSome) {
      n = n + 1
      step = it.next
    }
    return n
  }

  count(where f) {
    let it = self.iterator
    let n = 0
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (f.call(val)) { n = n + 1 }
      step = it.next
    }
    return n
  }

  find(where f) {
    let it = self.iterator
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (f.call(val)) { return Some(val) }
      step = it.next
    }
    return None
  }

  index(where f) {
    let it = self.iterator
    let index = 0
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (f.call(val)) { return Some(index) }
      index = index + 1
      step = it.next
    }
    return None
  }

  any(where f) {
    let it = self.iterator
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (f.call(val)) { return true }
      step = it.next
    }
    return false
  }

  all(where f) {
    let it = self.iterator
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (not f.call(val)) { return false }
      step = it.next
    }
    return true
  }

  none(where f) {
    let it = self.iterator
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (f.call(val)) { return false }
      step = it.next
    }
    return true
  }

  nth(_ n) {
    (n.is(Number) and n >= 0 and ((n % 1) == 0)).ifFalse || {
      throw ArgumentError.new("nth: n must be a non-negative integer")
    }
    let it = self.iterator
    let rem = n
    while (rem > 0) {
      let step = it.next
      if (step.isNone) { return None }
      rem = rem - 1
    }
    return it.next
  }

  last {
    let it = self.iterator
    let step = it.next
    if (step.isNone) { return None }
    let lastVal = step.match(some: |v| { v }, none: || { None })
    step = it.next
    while (step.isSome) {
      lastVal = step.match(some: |v| { v }, none: || { None })
      step = it.next
    }
    return Some(lastVal)
  }

  fold(initial initial, using f) {
    let it = self.iterator
    let acc = initial
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      acc = f.call(acc, val)
      step = it.next
    }
    return acc
  }

  reduce(using f) {
    let it = self.iterator
    let firstStep = it.next
    if (firstStep.isNone) { return None }
    let acc = firstStep.match(some: |v| { v }, none: || { None })
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      acc = f.call(acc, val)
      step = it.next
    }
    return Some(acc)
  }

  join { self.join("") }

  join(_ sep) {
    let it = self.iterator
    let step = it.next
    if (step.isNone) { return "" }
    let first = true
    let result = ""
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (not first) {
        result = result + sep
      }
      first = false
      result = result + val.toString
      step = it.next
    }
    return result
  }

  group(by block) {
    let it = self.iterator
    let result = Map<Dynamic, Dynamic>.new()
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      let key = block.call(val)
      let list = result.get(key).match(
        some: |list| { list },
        none: || {
          let new_list = List<Dynamic>.new()
          result.insert(new_list, for: key)
          new_list
        }
      )
      list.append(val)
      step = it.next
    }
    return result
  }

  partition(where predicate) {
    let it = self.iterator
    let accepted = List<Dynamic>.new()
    let rejected = List<Dynamic>.new()
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      if (predicate.call(val)) {
        accepted.append(val)
      } else {
        rejected.append(val)
      }
      step = it.next
    }
    return (accepted, rejected)
  }

  toSet {
    let it = self.iterator
    let result = Set<Dynamic>.new()
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      result.add(val)
      step = it.next
    }
    return result
  }

  toMap {
    let it = self.iterator
    let result = Map<Dynamic, Dynamic>.new()
    let step = it.next
    while (step.isSome) {
      let entry = step.match(some: |v| { v }, none: || { None })
      let key = entry.key
      if (result.includes(key)) {
        return Result::Error(DuplicateKeyError.new(key))
      }
      result.insert(entry.value, for: key)
      step = it.next
    }
    return Result::Ok(result)
  }

  toMap(merging block) {
    let it = self.iterator
    let result = Map<Dynamic, Dynamic>.new()
    let step = it.next
    while (step.isSome) {
      let entry = step.match(some: |v| { v }, none: || { None })
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
      step = it.next
    }
    return result
  }

  toList {
    let it = self.iterator
    let result = List<Dynamic>.new()
    let step = it.next
    while (step.isSome) {
      let val = step.match(some: |v| { v }, none: || { None })
      result.append(val)
      step = it.next
    }
    return result
  }
}

// The canonical Iterator => Iterable theorem is an ordinary guarded
// generic-target conformance. It intentionally remains in this transition
// module while the legacy public Iterable runtime class is retired by P5.
impl<I> Iterable for I
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
  type Iterator = I

  iterator { self }
}

// ============================================================================
// Canonical Lazy Iterator Adapters (one-shot, stateful)
// ============================================================================

class MappedIterator<I, U> {
  _upstream: I
  _mapper
  _done: Bool

  @constructor
  new(_ upstream: I, _ mapper) {
    _upstream = upstream
    _mapper = mapper
    _done = false
  }

  next {
    if (_done) { return None }
    let step = _upstream.next
    if (step.isNone) {
      _done = true
      return None
    }
    let val = step.match(some: |v| { v }, none: || { None })
    Some(_mapper.call(val))
  }

  iterator { self }
}

impl<I, U> Iterator for MappedIterator<I, U> {
  type Item = U
}

class FilterIterator<I> {
  _upstream: I
  _predicate
  _done: Bool

  @constructor
  new(_ upstream: I, _ predicate) {
    _upstream = upstream
    _predicate = predicate
    _done = false
  }

  next {
    if (_done) { return None }
    while (true) {
      let step = _upstream.next
      if (step.isNone) {
        _done = true
        return None
      }
      let val = step.match(some: |v| { v }, none: || { None })
      if (_predicate.call(val)) {
        return Some(val)
      }
    }
  }

  iterator { self }
}

impl<I> Iterator for FilterIterator<I> {
  type Item = <I as Iterator>::Item
}

class FilterMapIterator<I, U> {
  _upstream: I
  _mapper
  _done: Bool

  @constructor
  new(_ upstream: I, _ mapper) {
    _upstream = upstream
    _mapper = mapper
    _done = false
  }

  next {
    if (_done) { return None }
    while (true) {
      let step = _upstream.next
      if (step.isNone) {
        _done = true
        return None
      }
      let val = step.match(some: |v| { v }, none: || { None })
      let mapped = _mapper.call(val)
      if (mapped.isSome) {
        return mapped
      }
    }
  }

  iterator { self }
}

impl<I, U> Iterator for FilterMapIterator<I, U> {
  type Item = U
}

class FlatMapIterator<I, S> {
  _upstream: I
  _mapper
  _currentInner
  _done: Bool

  @constructor
  new(_ upstream: I, _ mapper) {
    _upstream = upstream
    _mapper = mapper
    _currentInner = None
    _done = false
  }

  next {
    if (_done) { return None }
    while (true) {
      if (_currentInner != None) {
        let innerStep = _currentInner.next
        if (innerStep.isSome) {
          return innerStep
        }
        _currentInner = None
      }
      let step = _upstream.next
      if (step.isNone) {
        _done = true
        return None
      }
      let outerVal = step.match(some: |v| { v }, none: || { None })
      let innerSource = _mapper.call(outerVal)
      _currentInner = innerSource.iterator
    }
  }

  iterator { self }
}

impl<I, S> Iterator for FlatMapIterator<I, S> {
  type Item = Dynamic
}

class FlattenIterator<I> {
  _upstream: I
  _currentInner
  _done: Bool

  @constructor
  new(_ upstream: I) {
    _upstream = upstream
    _currentInner = None
    _done = false
  }

  next {
    if (_done) { return None }
    while (true) {
      if (_currentInner != None) {
        let innerStep = _currentInner.next
        if (innerStep.isSome) {
          return innerStep
        }
        _currentInner = None
      }
      let step = _upstream.next
      if (step.isNone) {
        _done = true
        return None
      }
      let innerSource = step.match(some: |v| { v }, none: || { None })
      _currentInner = innerSource.iterator
    }
  }

  iterator { self }
}

impl<I> Iterator for FlattenIterator<I> {
  type Item = Dynamic
}

class ChainIterator<I, J> {
  _first: I
  _second: J
  _phase: Int

  @constructor
  new(_ first: I, _ second: J) {
    _first = first
    _second = second
    _phase = 0
  }

  next {
    if (_phase == 2) { return None }
    if (_phase == 0) {
      let step = _first.next
      if (step.isSome) { return step }
      _phase = 1
    }
    if (_phase == 1) {
      let step = _second.next
      if (step.isSome) { return step }
      _phase = 2
      return None
    }
    None
  }

  iterator { self }
}

impl<I, J> Iterator for ChainIterator<I, J> {
  type Item = <I as Iterator>::Item
}

class SkipIterator<I> {
  _upstream: I
  _remaining: Int
  _done: Bool

  @constructor
  new(_ upstream: I, _ n) {
    (n.is(Number) and n >= 0 and ((n % 1) == 0)).ifFalse || {
      throw ArgumentError.new("skip: n must be a non-negative integer")
    }
    _upstream = upstream
    _remaining = n
    _done = false
  }

  next {
    if (_done) { return None }
    while (_remaining > 0) {
      let step = _upstream.next
      if (step.isNone) {
        _done = true
        return None
      }
      _remaining = _remaining - 1
    }
    let step = _upstream.next
    if (step.isNone) {
      _done = true
      return None
    }
    step
  }

  iterator { self }
}

impl<I> Iterator for SkipIterator<I> {
  type Item = <I as Iterator>::Item
}

class SkipWhileIterator<I> {
  _upstream: I
  _predicate
  _stillSkipping: Bool
  _done: Bool

  @constructor
  new(_ upstream: I, _ predicate) {
    _upstream = upstream
    _predicate = predicate
    _stillSkipping = true
    _done = false
  }

  next {
    if (_done) { return None }
    while (_stillSkipping) {
      let step = _upstream.next
      if (step.isNone) {
        _done = true
        return None
      }
      let val = step.match(some: |v| { v }, none: || { None })
      if (not _predicate.call(val)) {
        _stillSkipping = false
        return step
      }
    }
    let step = _upstream.next
    if (step.isNone) {
      _done = true
      return None
    }
    step
  }

  iterator { self }
}

impl<I> Iterator for SkipWhileIterator<I> {
  type Item = <I as Iterator>::Item
}

class TakeIterator<I> {
  _upstream: I
  _remaining: Int
  _done: Bool

  @constructor
  new(_ upstream: I, _ n) {
    (n.is(Number) and n >= 0 and ((n % 1) == 0)).ifFalse || {
      throw ArgumentError.new("take: n must be a non-negative integer")
    }
    _upstream = upstream
    _remaining = n
    _done = false
  }

  next {
    if (_done or (_remaining <= 0)) {
      _done = true
      return None
    }
    let step = _upstream.next
    if (step.isNone) {
      _done = true
      return None
    }
    _remaining = _remaining - 1
    step
  }

  iterator { self }
}

impl<I> Iterator for TakeIterator<I> {
  type Item = <I as Iterator>::Item
}

class TakeWhileIterator<I> {
  _upstream: I
  _predicate
  _done: Bool

  @constructor
  new(_ upstream: I, _ predicate) {
    _upstream = upstream
    _predicate = predicate
    _done = false
  }

  next {
    if (_done) { return None }
    let step = _upstream.next
    if (step.isNone) {
      _done = true
      return None
    }
    let val = step.match(some: |v| { v }, none: || { None })
    if (_predicate.call(val)) {
      return step
    }
    _done = true
    None
  }

  iterator { self }
}

impl<I> Iterator for TakeWhileIterator<I> {
  type Item = <I as Iterator>::Item
}

class MapWhileIterator<I, U> {
  _upstream: I
  _mapper
  _done: Bool

  @constructor
  new(_ upstream: I, _ mapper) {
    _upstream = upstream
    _mapper = mapper
    _done = false
  }

  next {
    if (_done) { return None }
    let step = _upstream.next
    if (step.isNone) {
      _done = true
      return None
    }
    let val = step.match(some: |v| { v }, none: || { None })
    let mapped = _mapper.call(val)
    if (mapped.isSome) {
      return mapped
    }
    _done = true
    None
  }

  iterator { self }
}

impl<I, U> Iterator for MapWhileIterator<I, U> {
  type Item = U
}

class InspectIterator<I> {
  _upstream: I
  _callback
  _done: Bool

  @constructor
  new(_ upstream: I, _ callback) {
    _upstream = upstream
    _callback = callback
    _done = false
  }

  next {
    if (_done) { return None }
    let step = _upstream.next
    if (step.isNone) {
      _done = true
      return None
    }
    let val = step.match(some: |v| { v }, none: || { None })
    _callback.call(val)
    step
  }

  iterator { self }
}

impl<I> Iterator for InspectIterator<I> {
  type Item = <I as Iterator>::Item
}

class StrictZipIterator<I, J> {
  _left: I
  _right: J
  _done: Bool

  @constructor
  new(_ left: I, _ right: J) {
    _left = left
    _right = right
    _done = false
  }

  next {
    if (_done) { return None }
    let leftStep = _left.next
    let rightStep = _right.next
    if (leftStep.isSome and rightStep.isSome) {
      let leftVal = leftStep.match(some: |v| { v }, none: || { None })
      let rightVal = rightStep.match(some: |v| { v }, none: || { None })
      return Some((leftVal, rightVal))
    }
    if (leftStep.isNone and rightStep.isNone) {
      _done = true
      return None
    }
    _done = true
    throw ZipLengthMismatchError.new("strict zip lanes ended at different times")
  }

  iterator { self }
}

impl<I, J> Iterator for StrictZipIterator<I, J> {
  type Item = Dynamic
}

class IndexedIterator<I> {
  _upstream: I
  _ordinal: Int
  _done: Bool

  @constructor
  new(_ upstream: I) {
    _upstream = upstream
    _ordinal = 0
    _done = false
  }

  next {
    if (_done) { return None }
    let step = _upstream.next
    if (step.isNone) {
      _done = true
      return None
    }
    let val = step.match(some: |v| { v }, none: || { None })
    let ord = _ordinal
    _ordinal = _ordinal + 1
    Some((ord, val))
  }

  iterator { self }
}

impl<I> Iterator for IndexedIterator<I> {
  type Item = Dynamic
}

export Iterator, Iterable, MappedIterator, FilterIterator, FilterMapIterator, FlatMapIterator, FlattenIterator, ChainIterator, SkipIterator, SkipWhileIterator, TakeIterator, TakeWhileIterator, MapWhileIterator, InspectIterator, StrictZipIterator, IndexedIterator
