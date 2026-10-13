from universe.collections.iteration import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class OrderedSet<T> is Collection {
  @class
  native new() -> OrderedSet<T>
  internal native rawSize -> Int
  internal native rawAdd(_ value: Dynamic) -> Dynamic
  internal native rawAt(_ index: Int) -> T
  internal native rawIndexOf(_ value: Dynamic) -> Option<Int>
  internal native rawInsertAt(_ value: Dynamic, _ index: Int) -> Bool
  internal native rawReplaceAt(_ value: Dynamic, _ index: Int) -> Bool
  internal native rawMove(_ sourceIndex: Int, _ destinationIndex: Int) -> Bool
  internal native rawRemoveAt(_ index: Int) -> Bool
  internal native rawClear -> Unit

  size { self.rawSize }

  contains(_ value: T) -> Bool { self.rawIndexOf(value).isSome }
  add(_ value: T) -> Self { self.rawAdd(value) }
  at(_ index: Int) -> Option<T> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized >= self.size { return None }
    return Some(self.rawAt(normalized))
  }
  indexOf(_ value: T) -> Option<Int> { self.rawIndexOf(value) }

  insert(_ value: T, at index: Int) -> Result<Unit, Error> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized > self.size {
      return Result::Error(IndexError.new("OrderedSet insertion index out of range"))
    }
    if self.rawInsertAt(value, normalized) { return Result::Ok(()) }
    return Result::Error(ArgumentError.new("OrderedSet already contains the member"))
  }

  replaceAt(_ index: Int, with value: T) -> Result<T, Error> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized >= self.size {
      return Result::Error(IndexError.new("OrderedSet replacement index out of range"))
    }
    let mut previous = self.rawAt(normalized)
    if self.rawReplaceAt(value, normalized) { return Result::Ok(previous) }
    return Result::Error(ArgumentError.new("OrderedSet already contains the member"))
  }

  move(_ sourceIndex: Int, to index: Int) -> Result<Unit, Error> {
    let mut source = sourceIndex
    let mut destination = index
    if source < 0 { source = self.size + source }
    if destination < 0 { destination = self.size + destination }
    if source < 0 or source >= self.size or destination < 0 or destination >= self.size {
      return Result::Error(IndexError.new("OrderedSet move index out of range"))
    }
    if source == destination { return Result::Ok(()) }
    if self.rawMove(source, destination) { return Result::Ok(()) }
    return Result::Error(IndexError.new("OrderedSet move index out of range"))
  }

  remove(_ value: T) -> OrderedSet<T> {
    match self.rawIndexOf(value) {
      Option::Some(index) => { self.rawRemoveAt(index) }
      Option::None => { false }
    }
    return self
  }

  removeAt(_ index: Int) -> Result<T, Error> {
    let mut normalized = index
    if normalized < 0 { normalized = self.size + normalized }
    if normalized < 0 or normalized >= self.size {
      return Result::Error(IndexError.new("OrderedSet removal index out of range"))
    }
    let mut value = self.rawAt(normalized)
    if not self.rawRemoveAt(normalized) {
      return Result::Error(IndexError.new("OrderedSet removal index out of range"))
    }
    return Result::Ok(value)
  }

  clear -> Unit { self.rawClear }
  iterator -> OrderedSetIterator<T> { OrderedSetIterator<T>.new(self) }

  @class
  from<I>(_ source: I) -> OrderedSet<T>
  where I |= CanonicalIterable, <I as CanonicalIterable>::Item == T
  {
    let mut result = OrderedSet<T>.new()
    let mut it = source.iterator
    let mut step = it.next
    while step.isSome {
      let mut value = step.match(some: |item| { item }, none: || { None })
      if result.rawInsertAt(value, result.size) {
        step = it.next
      } else {
        ArgumentError.new("OrderedSet constructor encountered a duplicate member").raise()
      }
    }
    return result
  }

  ==(_ other) {
    if not (other is OrderedSet) { return false }
    let mut left = List<Dynamic>.new()
    let mut i = 0
    while i < self.size {
      left.append(self.rawAt(i))
      i = i + 1
    }
    let mut right = List<Dynamic>.new()
    i = 0
    while i < other.size {
      right.append(other.rawAt(i))
      i = i + 1
    }
    if left.size != right.size { return false }
    let mut same = true
    i = 0
    while same and i < left.size {
      let mut a = left.at(i)
      let mut b = right.at(i)
      let mut memberEqual = a == b
      if not memberEqual and a is Float and b is Float {
        memberEqual = a.isNaN and b.isNaN
      }
      same = memberEqual
      i = i + 1
    }
    return same
  }
}

class OrderedSetIterator<T> {
  mut _source: OrderedSet<T>
  mut _index: Int

  @constructor
  new(_ source: OrderedSet<T>) {
    _source = source
    _index = 0
  }

  next -> Option<T> {
    let mut value = _source.at(_index)
    if value.isSome { _index = _index + 1 }
    return value
  }

  iterator { self }
}

impl<T> CanonicalIterator for OrderedSetIterator<T> {
  type Item = T
}

impl<T> CanonicalIterable for OrderedSet<T> {
  type Item = T
  type Iterator = OrderedSetIterator<T>
  iterator -> OrderedSetIterator<T> { OrderedSetIterator<T>.new(self) }
}
