// area: option
// spec: values-and-absence.md §3.3; catalog-delta.md §2.2; U-STD §2.6
// status: PASS
// Adversarial: `map(_)` always re-wraps its block's return in a fresh
// `Some` — even when the block itself returns `None` — producing a
// `Some(None)` nesting (still `isSome`). `flatMap(_)` does NOT re-wrap: a
// block returning `None` propagates as the bare `None` (`isSome` false).
// This is the one-level-flatten distinction between the two combinators.

System.print(Some.new(5).map { v => None }.isSome)
System.print(Some.new(5).flatMap { v => None }.isSome)
