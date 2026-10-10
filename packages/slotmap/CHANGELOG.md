# Changelog

## 0.1.1

- Use explicit singleton marker declarations such as `data Entities()`.
- Replace manual `while true { match iterator.next ... }` test loops with
  `while let` tuple-pattern iteration.
- Use supported `for (entry in iterable)` syntax with local tuple destructuring
  in examples.
- Rename `contains` to `containsKey` to avoid semantic collision with
  `Iterable.contains`.
- Stop exposing `DenseSlotMap`'s mutable backing key/value lists.
- Stop tests from depending on `SlotKey`'s internal slot/generation values.
- Clarify that keys are opaque typed handles and document the strengthened API
  invariants.
