//! Persistent sequences: constant-time prepend and shared immutable tails.
from universe.collections.iteration import Iterator, Iterable

/// An immutable homogeneous sequence. Link shares its tail without copying.
/// Iteration and the operations below use loops rather than recursive calls.
data Chain<T> {
  Empty
  Link(_ head: T, _ tail: Chain<T>)
}

impl<T> Chain<T> {
  @class empty -> Chain<T> { Chain<T>::Empty }

  @class
  fromIterable<I>(_ values: I) -> Chain<T>
    where I |= Iterable, <I as Iterable>::Item == T
  {
    let mut result: Chain<T> = Chain<T>::Empty
    let iterator = values.iterator
    let mut step: Option<T> = iterator.next
    while step.isSome {
      match step {
        Option::Some(value) => { result = result.prepend(value) }
        Option::None => {}
      }
      step = iterator.next
    }
    result.reverse
  }

  prepend(_ value: T) -> Chain<T> { Chain<T>::Link(value, self) }

  isEmpty -> Bool {
    match self {
      Chain::Empty => true
      Chain::Link(_, _) => false
    }
  }

  first -> Option<T> {
    match self {
      Chain::Empty => Option::None
      Chain::Link(head, _) => Option::Some(head)
    }
  }

  /// Empty has an empty tail; a Link returns its shared tail.
  tail -> Chain<T> {
    match self {
      Chain::Empty => self
      Chain::Link(_, rest) => rest
    }
  }

  size -> Int {
    let mut count = 0
    let mut remaining: Chain<T> = self
    while true {
      match remaining {
        Chain::Empty => { return count }
        Chain::Link(_, rest) => { count = count + 1; remaining = rest }
      }
    }
  }

  reverse -> Chain<T> {
    let mut result: Chain<T> = Chain<T>::Empty
    let mut remaining: Chain<T> = self
    while true {
      match remaining {
        Chain::Empty => { return result }
        Chain::Link(head, rest) => { result = result.prepend(head); remaining = rest }
      }
    }
  }

  /// Copies the left spine and shares the entire right sequence. O(left.size).
  append(_ other: Chain<T>) -> Chain<T> {
    let mut result = other
    let mut remaining: Chain<T> = self.reverse
    while true {
      match remaining {
        Chain::Empty => { return result }
        Chain::Link(head, rest) => { result = result.prepend(head); remaining = rest }
      }
    }
  }

  map<U>(_ f: (T) -> U) -> Chain<U> {
    let mut result: Chain<U> = Chain<U>::Empty
    let mut remaining: Chain<T> = self
    while true {
      match remaining {
        Chain::Empty => { return result.reverse }
        Chain::Link(head, rest) => { result = result.prepend(f.call(head)); remaining = rest }
      }
    }
  }

  /// Calls f once per element, left to right, preserving each returned order.
  flatMap<U>(_ f: (T) -> Chain<U>) -> Chain<U> {
    let mut result: Chain<U> = Chain<U>::Empty
    let mut remaining: Chain<T> = self
    while true {
      match remaining {
        Chain::Empty => { return result.reverse }
        Chain::Link(head, rest) => {
          let mut inner: Chain<U> = f.call(head)
          while not inner.isEmpty {
            match inner {
              Chain::Link(value, tail) => { result = result.prepend(value); inner = tail }
              Chain::Empty => {}
            }
          }
          remaining = rest
        }
      }
    }
  }

  filter(_ predicate: (T) -> Bool) -> Chain<T> {
    let mut result: Chain<T> = Chain<T>::Empty
    let mut remaining: Chain<T> = self
    while true {
      match remaining {
        Chain::Empty => { return result.reverse }
        Chain::Link(head, rest) => {
          if predicate.call(head) { result = result.prepend(head) }
          remaining = rest
        }
      }
    }
  }

  fold<R>(initial seed: R, using f: (R, T) -> R) -> R {
    let mut result = seed
    let mut remaining: Chain<T> = self
    while true {
      match remaining {
        Chain::Empty => { return result }
        Chain::Link(head, rest) => { result = f.call(result, head); remaining = rest }
      }
    }
  }

  /// Visits all values once and retains only present callback results.
  filterMap<U>(_ f: (T) -> Option<U>) -> Chain<U> {
    let mut result: Chain<U> = Chain<U>::Empty
    let mut remaining: Chain<T> = self
    while true {
      match remaining {
        Chain::Empty => { return result.reverse }
        Chain::Link(head, rest) => {
          match f(head) {
            Option::Some(value) => { result = result.prepend(value) }
            Option::None => {}
          }
          remaining = rest
        }
      }
    }
  }

  /// Nonpositive counts produce an empty sequence.
  take(_ count: Int) -> Chain<T> {
    let mut result: Chain<T> = Chain<T>::Empty
    let mut remaining: Chain<T> = self
    let mut left = count
    while left > 0 {
      match remaining {
        Chain::Empty => { return result.reverse }
        Chain::Link(head, rest) => { result = result.prepend(head); remaining = rest }
      }
      left = left - 1
    }
    result.reverse
  }

  /// Returns a shared suffix. Nonpositive counts return self.
  drop(_ count: Int) -> Chain<T> {
    let mut remaining: Chain<T> = self
    let mut left = count
    while left > 0 {
      match remaining {
        Chain::Empty => { return remaining }
        Chain::Link(_, rest) => { remaining = rest }
      }
      left = left - 1
    }
    remaining
  }

  /// Zero-based indexing; negative or out-of-range positions give None.
  at(_ index: Int) -> Option<T> {
    if index < 0 { return Option::None }
    self.drop(index).first
  }

  iterator -> ChainIterator<T> { ChainIterator<T>.new(self) }

  /// Finds the first matching value; no later predicate is evaluated.
  find(_ predicate: (T) -> Bool) -> Option<T> {
    let mut remaining: Chain<T> = self
    while not remaining.isEmpty {
      match remaining {
        Chain::Empty => {}
        Chain::Link(head, rest) => {
          if predicate.call(head) { return Option<T>::Some(head) }
          remaining = rest
        }
      }
    }
    Option<T>::None
  }

  /// Partitions in one pass, retaining each side's original order.
  partition(_ predicate: (T) -> Bool) -> Partition<T> {
    let mut matching: Chain<T> = Chain<T>::Empty
    let mut remaining: Chain<T> = Chain<T>::Empty
    let iterator = self.iterator
    let mut step: Option<T> = iterator.next
    while step.isSome {
      match step {
        Option::Some(value) => {
          if predicate.call(value) { matching = matching.prepend(value) } else { remaining = remaining.prepend(value) }
        }
        Option::None => {}
      }
      step = iterator.next
    }
    Partition<T>(matching: matching.reverse, remaining: remaining.reverse)
  }

  /// Stops when either input is empty; combines corresponding values once.
  zipWith<U, R>(_ other: Chain<U>, combine: (T, U) -> R) -> Chain<R> {
    let mut left: Chain<T> = self
    let mut right: Chain<U> = other
    let mut result: Chain<R> = Chain<R>::Empty
    while not left.isEmpty and not right.isEmpty {
      match left {
        Chain::Empty => {}
        Chain::Link(a, rest) => {
          match right {
            Chain::Empty => {}
            Chain::Link(b, tail) => { result = result.prepend(combine.call(a, b)); right = tail }
          }
          left = rest
        }
      }
    }
    result.reverse
  }

  ==(_ other) -> Bool {
    if other is not Chain { return false }
    let mut left: Chain<T> = self
    let mut right: Chain<T> = other
    while true {
      match left {
        Chain::Empty => { return right.isEmpty }
        Chain::Link(a, rest) => {
          match right {
            Chain::Link(b, tail) => {
              if a != b { return false }
              left = rest
              right = tail
            }
            _ => { return false }
          }
        }
      }
    }
  }
  !=(_ other) -> Bool { not (self == other) }
  hash -> Int {
    self.fold(initial: 17, using: |hash, value| { (hash * 31 + value.hash) % 999999937 })
  }
}

data Partition<T>(matching: Chain<T>, remaining: Chain<T>)

class ChainIterator<T> {
  mut _remaining: Chain<T>
  @constructor new(_ values: Chain<T>) { _remaining = values }
  next -> Option<T> {
    match _remaining {
      Chain::Empty => Option::None
      Chain::Link(head, rest) => { _remaining = rest; Option::Some(head) }
    }
  }
}

impl<T> Iterator for ChainIterator<T> {
  type Item = T
}
impl<T> Iterable for Chain<T> {
  type Item = T
  type Iterator = ChainIterator<T>
  iterator -> ChainIterator<T> { ChainIterator<T>.new(self) }
}

/// A sequence with a statically required first element.
data NonEmpty<T>(head: T, tail: Chain<T>)

impl<T> NonEmpty<T> {
  @class
  fromChain(_ values: Chain<T>) -> Option<NonEmpty<T>> {
    match values {
      Chain::Empty => Option::None
      Chain::Link(head, rest) => Option::Some(NonEmpty<T>(head: head, tail: rest))
    }
  }
  @class one(_ value: T) -> NonEmpty<T> { NonEmpty<T>(head: value, tail: Chain<T>::Empty) }
  toChain -> Chain<T> { self.tail.prepend(self.head) }
  size -> Int { 1 + self.tail.size }
  map<U>(_ f: (T) -> U) -> NonEmpty<U> {
    let head = f.call(self.head)
    NonEmpty<U>(head: head, tail: self.tail.map(f))
  }
  flatMap<U>(_ f: (T) -> NonEmpty<U>) -> NonEmpty<U> {
    let first = f.call(self.head)
    let rest = self.tail.flatMap(|value| { f.call(value).toChain })
    NonEmpty<U>(head: first.head, tail: first.tail.append(rest))
  }
  append(_ other: NonEmpty<T>) -> NonEmpty<T> {
    NonEmpty<T>(head: self.head, tail: self.tail.append(other.toChain))
  }
  /// No identity argument is needed because a first value always exists.
  reduce(_ f: (T, T) -> T) -> T { self.tail.fold(initial: self.head, using: f) }
  /// Filtering may remove every value, so the result deliberately is Chain<U>.
  filterMap<U>(_ f: (T) -> Option<U>) -> Chain<U> { self.toChain.filterMap(f) }
  filter(_ predicate: (T) -> Bool) -> Chain<T> { self.toChain.filter(predicate) }
  iterator -> ChainIterator<T> { self.toChain.iterator }
  ==(_ other) -> Bool {
    if other is not NonEmpty { return false }
    self.head == other.head and self.tail == other.tail
  }
  !=(_ other) -> Bool { not (self == other) }
  hash -> Int { self.toChain.hash }
}

impl<T> Iterable for NonEmpty<T> {
  type Item = T
  type Iterator = ChainIterator<T>
  iterator -> ChainIterator<T> { self.toChain.iterator }
}

export Chain, ChainIterator, NonEmpty, Partition
