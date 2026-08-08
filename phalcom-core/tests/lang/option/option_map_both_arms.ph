// area: option
// spec: values-and-absence.md §3.3; catalog-delta.md §2.2; U-STD §2.6
// status: PASS
// U-STD: `map(_)` lifts `f` over a `Some`'s value and re-wraps; a `None`
// passes through untouched. Extracting with `unwrapOr(_)` proves both arms.

System.print(Some.new(5).map |v| { v + 1 }.unwrapOr(0))
System.print(None.map |v| { v + 1 }.unwrapOr(0))
