// area: option
// spec: values-and-absence.md §3.3; catalog-delta.md §2.2; U-STD §2.6
// status: PASS
// U-STD: `ifSome(_)` runs `f` for its side effect on a `Some`'s value and
// returns `self` so calls chain; a `None` is passed through, `f` never fires.
// The `Some` case prints the value (7) then chains `map`+`unwrapOr` (-> 8);
// the `None` case prints nothing and falls through to the default (99).

System.print(Some.new(7).ifSome { v => System.print(v) }.map { v => v + 1 }.unwrapOr(0))
System.print(None.ifSome { v => System.print(v) }.unwrapOr(99))
