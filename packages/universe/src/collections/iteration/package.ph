from universe.text.builder import StringBuilder

/// Iterator and iterable collection protocols.
trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }

  map(_ f) { MappedIterator.new(self, f) }
  filter(_ pred) { FilterIterator.new(self, pred) }
  filterMap(_ f) { FilterMapIterator.new(self, f) }
  flatMap<S>(_ f: (Self::Item) -> S) -> FlatMapIterator<Self, S>
    where S |= Iterable
  { FlatMapIterator.new(self, f) }
  flatten -> FlattenIterator<Self>
    where Self::Item |= Iterable
  { FlattenIterator.new(self) }
  chain<J>(_ other: J) -> ChainIterator<Self, J>
    where J |= Iterator, <J as Iterator>::Item == Self::Item
  { ChainIterator.new(self, other) }
  skip(_ n) { SkipIterator.new(self, n) }
  skipWhile(_ pred) { SkipWhileIterator.new(self, pred) }
  take(_ n) { TakeIterator.new(self, n) }
  takeWhile(_ pred) { TakeWhileIterator.new(self, pred) }
  mapWhile(_ f) { MapWhileIterator.new(self, f) }
  inspect(_ f) { InspectIterator.new(self, f) }
  zip<J>(_ other: J) -> StrictZipIterator<Self, J>
    where J |= Iterator
  { StrictZipIterator.new(self, other) }
}

trait Iterable
  where
    Self::Iterator |= Iterator,
    <Self::Iterator as Iterator>::Item == Self::Item
{
  type Item
  type Iterator

  iterator -> Self::Iterator

  iter -> Self::Iterator { self.iterator }

  indexed -> IndexedIterator<Self::Iterator> { IndexedIterator.new(self.iterator) }

  each(_ f) {
    let mut it = self.iterator
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      f.call(val)
      step = it.next
    }
    return ()
  }

  each(indexed f) {
    let mut it = self.iterator
    let mut step = it.next
    let mut index = 0
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      f.call(index, val)
      index = index + 1
      step = it.next
    }
    return ()
  }

  contains(_ x) {
    let mut it = self.iterator
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      if val == x { return true }
      step = it.next
    }
    return false
  }

  includes(_ x) { self.contains(x) }

  count {
    let mut it = self.iterator
    let mut n = 0
    let mut step = it.next
    while step.isSome {
      n = n + 1
      step = it.next
    }
    return n
  }

  count(where f) {
    let mut it = self.iterator
    let mut n = 0
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      if f.call(val) { n = n + 1 }
      step = it.next
    }
    return n
  }

  find(where f) {
    let mut it = self.iterator
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      if f.call(val) { return Some(val) }
      step = it.next
    }
    return None
  }

  index(where f) {
    let mut it = self.iterator
    let mut index = 0
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      if f.call(val) { return Some(index) }
      index = index + 1
      step = it.next
    }
    return None
  }

  any(where f) {
    let mut it = self.iterator
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      if f.call(val) { return true }
      step = it.next
    }
    return false
  }

  all(where f) {
    let mut it = self.iterator
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      if not f.call(val) { return false }
      step = it.next
    }
    return true
  }

  none(where f) {
    let mut it = self.iterator
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      if f.call(val) { return false }
      step = it.next
    }
    return true
  }

  /// Returns the element at zero-based iterator position `n`.
  nth(_ n) {
    if n is not Number or n < 0 or n % 1 != 0 {
      throw ArgumentError.new("nth: n must be a non-negative integer")
    }
    let mut it = self.iterator
    let mut rem = n
    while rem > 0 {
      let mut step = it.next
      if step.isNone { return None }
      rem = rem - 1
    }
    return it.next
  }

  last {
    let mut it = self.iterator
    let mut step = it.next
    if step.isNone { return None }
    let mut lastVal = step.match(some: |v| { v }, none: || { None })
    step = it.next
    while step.isSome {
      lastVal = step.match(some: |v| { v }, none: || { None })
      step = it.next
    }
    return Some(lastVal)
  }

  fold(initial initial, using f) {
    let mut it = self.iterator
    let mut acc = initial
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      acc = f.call(acc, val)
      step = it.next
    }
    return acc
  }

  reduce(using f) {
    let mut it = self.iterator
    let mut firstStep = it.next
    if firstStep.isNone { return None }
    let mut acc = firstStep.match(some: |v| { v }, none: || { None })
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      acc = f.call(acc, val)
      step = it.next
    }
    return Some(acc)
  }

  join { self.join("") }

  join(_ sep) {
    let mut it = self.iterator
    let mut step = it.next
    if step.isNone { return "" }
    let mut sb = StringBuilder.new()
    let mut first = true
    while step.isSome {
      let val = step.match(some: |v| { v }, none: || { None })
      if not first {
        sb.append(sep)
      }
      first = false
      sb.append(val.toString)
      step = it.next
    }
    return sb.toString
  }

  group(by block) {
    let mut it = self.iterator
    let mut result = Map<Dynamic, Dynamic>.new()
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      let mut key = block.call(val)
      let mut list = result.get(key).match(
        some: |list| { list },
        none: || {
          let mut new_list = List<Dynamic>.new()
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
    let mut accepted = List<Dynamic>.new()
    let mut rejected = List<Dynamic>.new()

    let mut it = self.iterator
    while let Option::Some(val) = it.next {
      if predicate.call(val) {
        accepted.append(val)
      } else {
        rejected.append(val)
      }
    }

    return (accepted, rejected)
  }

  toSet {
    let mut result = Set<Dynamic>.new()

    let mut it = self.iterator
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      result.add(val)
      step = it.next
    }

    return result
  }

  toMap -> Result<Map<Dynamic, Dynamic>, DuplicateKeyError> {
    let mut result = Map<Dynamic, Dynamic>.new()

    let mut it = self.iterator
    let mut step = it.next
    while step.isSome {
      let mut entry = step.match(some: |v| { v }, none: || { None })
      let mut key = entry.key
      if result.includes(key) {
        return Result::Error(DuplicateKeyError.new(key))
      }
      result.insert(entry.value, for: key)
      step = it.next
    }
    return Result::Ok(result)
  }

  toMap(merging block) {
    let mut it = self.iterator
    let mut result = Map<Dynamic, Dynamic>.new()
    let mut step = it.next
    while step.isSome {
      let mut entry = step.match(some: |v| { v }, none: || { None })
      let mut key = entry.key
      let mut val = entry.value
      result.get(key).match(
        some: |existingVal| {
          let mut merged = block.call(existingVal, val)
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
    let mut it = self.iterator
    let mut result = List<Dynamic>.new()
    let mut step = it.next
    while step.isSome {
      let mut val = step.match(some: |v| { v }, none: || { None })
      result.append(val)
      step = it.next
    }
    return result
  }
}

impl<I> Iterable for I
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
  type Iterator = I

  iterator { self }
}

class MappedIterator<I, U> {
  mut _upstream: I
  mut _mapper
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> U)
    where I |= Iterator
  {
    _upstream = upstream
    _mapper = mapper
    _done = false
  }

  next {
    if _done { return None }
    let mut step = _upstream.next
    if step.isNone {
      _done = true
      return None
    }
    let mut val = step.match(some: |v| { v }, none: || { None })
    Some(_mapper.call(val))
  }

  iterator { self }
}

impl<I, U> Iterator for MappedIterator<I, U>
  where I |= Iterator
{
  type Item = U
}

class FilterIterator<I> {
  mut _upstream: I
  mut _predicate
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ predicate: (<I as Iterator>::Item) -> Bool)
    where I |= Iterator
  {
    _upstream = upstream
    _predicate = predicate
    _done = false
  }

  next {
    if _done { return None }
    while true {
      let mut step = _upstream.next
      if step.isNone {
        _done = true
        return None
      }
      let mut val = step.match(some: |v| { v }, none: || { None })
      if _predicate.call(val) {
        return Some(val)
      }
    }
  }

  iterator { self }
}

impl<I> Iterator for FilterIterator<I>
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
}

class FilterMapIterator<I, U> {
  mut _upstream: I
  mut _mapper
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> Option<U>)
    where I |= Iterator
  {
    _upstream = upstream
    _mapper = mapper
    _done = false
  }

  next {
    if _done { return None }
    while true {
      let mut step = _upstream.next
      if step.isNone {
        _done = true
        return None
      }
      let mut val = step.match(some: |v| { v }, none: || { None })
      let mut mapped = _mapper.call(val)
      if mapped.isSome {
        return mapped
      }
    }
  }

  iterator { self }
}

impl<I, U> Iterator for FilterMapIterator<I, U>
  where I |= Iterator
{
  type Item = U
}

class FlatMapIterator<I, S> {
  mut _upstream: I
  mut _mapper
  mut _currentInner
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> S)
    where I |= Iterator, S |= Iterable
  {
    _upstream = upstream
    _mapper = mapper
    _currentInner = None
    _done = false
  }

  next {
    if _done { return None }
    while true {
      if _currentInner is not None {
        let mut innerStep = _currentInner.next
        if innerStep.isSome {
          return innerStep
        }
        _currentInner = None
      }
      let mut step = _upstream.next
      if step.isNone {
        _done = true
        return None
      }
      let mut outerVal = step.match(some: |v| { v }, none: || { None })
      let mut innerSource = _mapper.call(outerVal)
      _currentInner = innerSource.iterator
    }
  }

  iterator { self }
}

impl<I, S> Iterator for FlatMapIterator<I, S>
  where I |= Iterator, S |= Iterable
{
  type Item = <S as Iterable>::Item
}

class FlattenIterator<I> {
  mut _upstream: I
  mut _currentInner
  mut _done: Bool

  @constructor
  new(_ upstream: I)
    where I |= Iterator, <I as Iterator>::Item |= Iterable
  {
    _upstream = upstream
    _currentInner = None
    _done = false
  }

  next {
    if _done { return None }
    while true {
      if _currentInner is not None {
        let mut innerStep = _currentInner.next
        if innerStep.isSome {
          return innerStep
        }
        _currentInner = None
      }
      let mut step = _upstream.next
      if step.isNone {
        _done = true
        return None
      }
      let mut innerSource = step.match(some: |v| { v }, none: || { None })
      _currentInner = innerSource.iterator
    }
  }

  iterator { self }
}

impl<I> Iterator for FlattenIterator<I>
  where I |= Iterator, <I as Iterator>::Item |= Iterable
{
  type Item = < <I as Iterator>::Item as Iterable>::Item
}

class ChainIterator<I, J> {
  mut _first: I
  mut _second: J
  mut _phase: Int

  @constructor
  new(_ first: I, _ second: J)
    where I |= Iterator, J |= Iterator, <J as Iterator>::Item == <I as Iterator>::Item
  {
    _first = first
    _second = second
    _phase = 0
  }

  next -> Option<<I as Iterator>::Item> {
    if _phase == 2 { return None }
    if _phase == 0 {
      let mut step = _first.next
      if step.isSome { return step }
      _phase = 1
    }
    if _phase == 1 {
      let mut step = _second.next
      if step.isSome { return step }
      _phase = 2
      return None
    }
    None
  }

  iterator { self }
}

impl<I, J> Iterator for ChainIterator<I, J>
  where I |= Iterator, J |= Iterator, <J as Iterator>::Item == <I as Iterator>::Item
{
  type Item = <I as Iterator>::Item
}

class SkipIterator<I> {
  mut _upstream: I
  mut _remaining: Int
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ n)
    where I |= Iterator
  {
    if n is not Number or n < 0 or n % 1 != 0 {
      throw ArgumentError.new("skip: n must be a non-negative integer")
    }
    _upstream = upstream
    _remaining = n
    _done = false
  }

  next -> Option<<I as Iterator>::Item> {
    if _done { return None }
    while _remaining > 0 {
      let mut step = _upstream.next
      if step.isNone {
        _done = true
        return None
      }
      _remaining = _remaining - 1
    }
    let mut step = _upstream.next
    if step.isNone {
      _done = true
      return None
    }
    step
  }

  iterator { self }
}

impl<I> Iterator for SkipIterator<I>
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
}

class SkipWhileIterator<I> {
  mut _upstream: I
  mut _predicate
  mut _stillSkipping: Bool
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ predicate: (<I as Iterator>::Item) -> Bool)
    where I |= Iterator
  {
    _upstream = upstream
    _predicate = predicate
    _stillSkipping = true
    _done = false
  }

  next -> Option<<I as Iterator>::Item> {
    if _done { return None }
    while _stillSkipping {
      let mut step = _upstream.next
      if step.isNone {
        _done = true
        return None
      }
      let mut val = step.match(some: |v| { v }, none: || { None })
      if not _predicate.call(val) {
        _stillSkipping = false
        return step
      }
    }
    let mut step = _upstream.next
    if step.isNone {
      _done = true
      return None
    }
    step
  }

  iterator { self }
}

impl<I> Iterator for SkipWhileIterator<I>
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
}

class TakeIterator<I> {
  mut _upstream: I
  mut _remaining: Int
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ n)
    where I |= Iterator
  {
    if n is not Number or n < 0 or n % 1 != 0 {
      throw ArgumentError.new("take: n must be a non-negative integer")
    }
    _upstream = upstream
    _remaining = n
    _done = false
  }

  next -> Option<<I as Iterator>::Item> {
    if _done or _remaining <= 0 {
      _done = true
      return None
    }
    let mut step = _upstream.next
    if step.isNone {
      _done = true
      return None
    }
    _remaining = _remaining - 1
    step
  }

  iterator { self }
}

impl<I> Iterator for TakeIterator<I>
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
}

class TakeWhileIterator<I> {
  mut _upstream: I
  mut _predicate
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ predicate: (<I as Iterator>::Item) -> Bool)
    where I |= Iterator
  {
    _upstream = upstream
    _predicate = predicate
    _done = false
  }

  next -> Option<<I as Iterator>::Item> {
    if _done { return None }
    let mut step = _upstream.next
    if step.isNone {
      _done = true
      return None
    }
    let mut val = step.match(some: |v| { v }, none: || { None })
    if _predicate.call(val) {
      return step
    }
    _done = true
    None
  }

  iterator { self }
}

impl<I> Iterator for TakeWhileIterator<I>
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
}

class MapWhileIterator<I, U> {
  mut _upstream: I
  mut _mapper
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> Option<U>)
    where I |= Iterator
  {
    _upstream = upstream
    _mapper = mapper
    _done = false
  }

  next -> Option<U> {
    if _done { return None }
    let mut step = _upstream.next
    if step.isNone {
      _done = true
      return None
    }
    let mut val = step.match(some: |v| { v }, none: || { None })
    let mut mapped = _mapper.call(val)
    if mapped.isSome {
      return mapped
    }
    _done = true
    None
  }

  iterator { self }
}

impl<I, U> Iterator for MapWhileIterator<I, U>
  where I |= Iterator
{
  type Item = U
}

class InspectIterator<I> {
  mut _upstream: I
  mut _callback
  mut _done: Bool

  @constructor
  new(_ upstream: I, _ callback: (<I as Iterator>::Item) -> Unit)
    where I |= Iterator
  {
    _upstream = upstream
    _callback = callback
    _done = false
  }

  next -> Option<<I as Iterator>::Item> {
    if _done { return None }
    let mut step = _upstream.next
    if step.isNone {
      _done = true
      return None
    }
    let mut val = step.match(some: |v| { v }, none: || { None })
    _callback.call(val)
    step
  }

  iterator { self }
}

impl<I> Iterator for InspectIterator<I>
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
}

class StrictZipIterator<I, J> {
  mut _left: I
  mut _right: J
  mut _done: Bool

  @constructor
  new(_ left: I, _ right: J)
    where I |= Iterator, J |= Iterator
  {
    _left = left
    _right = right
    _done = false
  }

  next -> Option<(<I as Iterator>::Item, <J as Iterator>::Item)> {
    if _done { return None }
    let mut leftStep = _left.next
    let mut rightStep = _right.next
    if leftStep.isSome and rightStep.isSome {
      let mut leftVal = leftStep.match(some: |v| { v }, none: || { None })
      let mut rightVal = rightStep.match(some: |v| { v }, none: || { None })
      return Some((leftVal, rightVal))
    }
    if leftStep.isNone and rightStep.isNone {
      _done = true
      return None
    }
    _done = true
    throw ZipLengthMismatchError.new("strict zip lanes ended at different times")
  }

  iterator { self }
}

impl<I, J> Iterator for StrictZipIterator<I, J>
  where I |= Iterator, J |= Iterator
{
  type Item = (<I as Iterator>::Item, <J as Iterator>::Item)
}

class IndexedIterator<I> {
  mut _upstream: I
  mut _ordinal: Int
  mut _done: Bool

  @constructor
  new(_ upstream: I)
    where I |= Iterator
  {
    _upstream = upstream
    _ordinal = 0
    _done = false
  }

  next -> Option<(Int, <I as Iterator>::Item)> {
    if _done { return None }
    let mut step = _upstream.next
    if step.isNone {
      _done = true
      return None
    }
    let mut val = step.match(some: |v| { v }, none: || { None })
    let mut ord = _ordinal
    _ordinal = _ordinal + 1
    Some((ord, val))
  }

  iterator { self }
}

impl<I> Iterator for IndexedIterator<I>
  where I |= Iterator
{
  type Item = (Int, <I as Iterator>::Item)
}

export Iterator, Iterable, MappedIterator, FilterIterator, FilterMapIterator, FlatMapIterator, FlattenIterator, ChainIterator, SkipIterator, SkipWhileIterator, TakeIterator, TakeWhileIterator, MapWhileIterator, InspectIterator, StrictZipIterator, IndexedIterator
