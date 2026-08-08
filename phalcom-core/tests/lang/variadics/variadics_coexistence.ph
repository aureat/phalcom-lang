// area: variadics
// spec: U9-implementation-spec.md §2, §6
// status: PASS
// A class defining both an ordinary two-arg `sum(_,_)` and a variadic
// `sum(*numbers)` under the same bare name: these intern as distinct
// selectors (`sum(_:_:)` vs `sum(*)`), so a call with exactly 2 args hits
// the exact-selector fixed method first (the probe never runs on a hit),
// while a call with 3 args misses the fixed selector and falls through to
// the variadic probe.

class Adder {
  sum(_ a, _ b) {
    return -1
  }
  sum(*numbers) {
    let total = 0
    numbers.each(|n| { total = total + n })
    return total
  }
}
const a = Adder.new()
System.print(a.sum(1, 2))
System.print(a.sum(1, 2, 3))
