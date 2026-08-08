// area: blocks
// spec: blocks.md; functions.md; catalog-delta.md §2.4
// status: PASS
// Promoted from pending/ once U-STD landed `List.reduce` (DEFERRED.md #25): a
// 2-arity block flows as the trailing argument of `reduce(_:_:)` and is called
// per element to fold the accumulator. Lists are built with `List.new()` +
// `add(_)` (no list-literal syntax — still deferred, DEFERRED.md #6).
const numbers = List.new()
numbers.add(1)
numbers.add(2)
numbers.add(3)
numbers.add(4)
const sum = numbers.reduce(0) |acc, n| { acc + n }
System.print(sum)
