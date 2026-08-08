// area: option
// spec: values-and-absence.md §3.3; catalog-delta.md §2.2; U-STD §2.6
// status: PASS
// Adversarial: a `None` receiver short-circuits an entire `map` ->
// `flatMap` -> `filter` chain — every stage's block passes through
// untouched, so none of them ever run. A shared `calls` counter proves
// zero invocations, not just that the final value is `None`.

let calls = 0
const result = None.map |v| { calls = calls + 1; v + 1 }.flatMap |v| { calls = calls + 1; Some.new(v * 2) }.filter |v| { calls = calls + 1; v > 0 }.unwrapOr(-1)
System.print(result)
System.print(calls)
