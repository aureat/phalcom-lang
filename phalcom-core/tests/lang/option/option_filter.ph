// area: option
// spec: values-and-absence.md §3.3; catalog-delta.md §2.2; U-STD §2.6
// status: PASS
// U-STD: `filter(_)` keeps a `Some(v)` when `pred(v)` holds and collapses it
// to `None` otherwise; a `None` stays `None`. `pred` must yield a real `Bool`.

System.print(Some.new(4).filter |v| { v > 3 }.unwrapOr(-1))
System.print(Some.new(2).filter |v| { v > 3 }.unwrapOr(-1))
System.print(None.filter |v| { v > 3 }.unwrapOr(-1))
