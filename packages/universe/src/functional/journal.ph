//! Persistent append trees for logs, audit trails, and batched output.
from .chain import Chain
from .algebra import Semigroup, Monoid

/// Constant-time concatenation without copying accumulated entries.
/// Entries are materialized using an explicit work stack, not tree recursion.
data Journal<T> {
  Empty
  Entry(_ value: T)
  Concat(_ left: Journal<T>, _ right: Journal<T>)
}

impl<T> Journal<T> {
  isEmpty -> Bool {
    // A user-created Concat may contain only empty subtrees.
    let emptyPending: Chain<Journal<T>> = Chain<Journal<T>>::Empty
    let mut pending: Chain<Journal<T>> = emptyPending.prepend(self)
    while not pending.isEmpty {
      match pending {
        Chain::Empty => {}
        Chain::Link(node, rest) => {
          pending = rest
          match node {
            Journal::Empty => {}
            Journal::Entry(_) => { return false }
            Journal::Concat(left, right) => { pending = pending.prepend(right).prepend(left) }
          }
        }
      }
    }
    true
  }

  /// Appends one entry in O(1), retaining earlier versions unchanged.
  append(_ value: T) -> Journal<T> { self.combine(Journal<T>::Entry(value)) }

  /// Flattens left to right in O(tree nodes + entries), with no recursive calls.
  entries -> Chain<T> {
    let emptyPending: Chain<Journal<T>> = Chain<Journal<T>>::Empty
    let mut pending: Chain<Journal<T>> = emptyPending.prepend(self)
    let mut result: Chain<T> = Chain<T>::Empty
    while not pending.isEmpty {
      match pending {
        Chain::Empty => {}
        Chain::Link(node, rest) => {
          pending = rest
          match node {
            Journal::Empty => {}
            Journal::Entry(value) => { result = result.prepend(value) }
            Journal::Concat(left, right) => { pending = pending.prepend(right).prepend(left) }
          }
        }
      }
    }
    result.reverse
  }

  fold<R>(initial seed: R, using f: (R, T) -> R) -> R {
    let emptyPending: Chain<Journal<T>> = Chain<Journal<T>>::Empty
    let mut pending: Chain<Journal<T>> = emptyPending.prepend(self)
    let mut result = seed
    while not pending.isEmpty {
      match pending {
        Chain::Link(node, rest) => {
          pending = rest
          match node {
            Journal::Empty => {}
            Journal::Entry(value) => { result = f.call(result, value) }
            Journal::Concat(left, right) => { pending = pending.prepend(right).prepend(left) }
          }
        }
        _ => {}
      }
    }
    result
  }

  /// Entry-wise equality makes combination associative under == regardless
  /// of tree grouping. Exact data identity (===) still includes the tree shape.
  ==(_ other) -> Bool {
    if other is not Journal { return false }
    let mut left = self.entries
    let mut right = other.entries
    while not left.isEmpty and not right.isEmpty {
      if left.first.unwrap != right.first.unwrap { return false }
      left = left.tail
      right = right.tail
    }
    left.isEmpty and right.isEmpty
  }
  !=(_ other) -> Bool { not (self == other) }
  hash -> Int {
    self.fold(initial: 17, using: |hash, entry| { (hash * 31 + entry.hash) % 999999937 })
  }
}

impl<T> Semigroup for Journal<T> {
  combine(_ other: Journal<T>) -> Journal<T> {
    match self {
      Journal::Empty => other
      _ => {
        match other {
          Journal::Empty => self
          _ => Journal<T>::Concat(self, other)
        }
      }
    }
  }
}
impl<T> Monoid for Journal<T> { @class empty -> Journal<T> { Journal<T>::Empty } }

export Journal
