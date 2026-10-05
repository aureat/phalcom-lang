// Public structural strategy contract.
//
// Strategies are pure descriptions of how values consume semantic primitive
// choices. They never own randomness, replay cursors, or shrinking policy.

import DrawData from "choices/data"

protocol Strategy<out T> {
  draw(data: DrawData) -> T
  map<U>(transform: [T] -> U) -> Strategy<U>
  filter(predicate: [T] -> Bool) -> Strategy<T>
  flatMap<U>(transform: [T] -> Strategy<U>) -> Strategy<U>
  named(label: Symbol) -> Strategy<T>
  label -> Option<Symbol>
  fingerprint -> String
}
