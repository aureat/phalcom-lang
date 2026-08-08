// area: option
// spec: values-and-absence.md §3.3; catalog-delta.md §2.2; U-STD §2.6
// status: PASS
// Adversarial: `map` -> `flatMap` -> `filter` chained in one expression. The
// first receiver survives the whole chain to a `Some` extracted by
// `unwrapOr`; the second gets collapsed to `None` by `filter`'s predicate
// partway through, so `unwrapOr` falls back to its default.

System.print(Some.new(5).map |v| { v + 1 }.flatMap |v| { Some.new(v * 2) }.filter |v| { v > 10 }.unwrapOr(-1))
System.print(Some.new(1).map |v| { v + 1 }.flatMap |v| { Some.new(v * 2) }.filter |v| { v > 10 }.unwrapOr(-1))
