// area: blocks
// spec: blocks.md; functions.md; catalog-delta.md §2.4
// status: PASS
// Promoted from pending/ once U-STD landed `List.reduce` (DEFERRED.md #25): a
// 2-arity block flows as the trailing argument of `reduce(_:_:)` and is called
// per element to fold the accumulator. Lists are built with `[]` +
// `add(_)` (no list-literal syntax — still deferred, DEFERRED.md #6).
const numbers = []
numbers.append(1)
numbers.append(2)
numbers.append(3)
numbers.append(4)
const sum = numbers.reduce(0) |acc, n| { acc + n }
System.print(sum)
