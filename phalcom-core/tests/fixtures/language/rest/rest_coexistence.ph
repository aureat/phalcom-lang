// area: rest
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md §5, §15-20
// status: PASS
// A class defining both an ordinary two-arg `sum(_,_)` and a positional rest
// `sum(*numbers)` under the same bare name: exact lookup wins for two args,
// while three args miss the fixed selector and use rest-family fallback.

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
