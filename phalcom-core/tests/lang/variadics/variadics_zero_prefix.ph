// area: variadics
// spec: U9-implementation-spec.md §2, §6; messages-and-selectors.md §4
// status: PASS
// A zero-prefix variadic (`sum(*numbers)`) collects every positional
// argument into a real `List` bound to the rest parameter — `sum(1,2,3)`
// sums to 6, `sum()` sums to 0 (empty list, no fixed args required).

class Summer {
  sum(*numbers) {
    var total = 0
    numbers.each({ n => total = total + n })
    return total
  }
}
let s = Summer.new()
System.print(s.sum(1, 2, 3))
System.print(s.sum())
