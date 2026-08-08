// area: option
// spec: values-and-absence.md §3.3; catalog-delta.md §2.2; U-STD §2.6
// status: PASS
// U-STD: `flatMap(_)` binds `f` (which itself returns an `Option`) over a
// `Some`'s value without re-wrapping; a `None` short-circuits the chain.

System.print(Some.new(5).flatMap |v| { Some.new(v * 2) }.unwrapOr(0))
System.print(None.flatMap |v| { Some.new(v * 2) }.unwrapOr(-1))
