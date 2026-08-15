// area: family
// spec: docs/spec/callables/reflection.md §2
// status: PASS
// MethodFamily snapshots exact routes plus the ordered compatible rest route.
// BoundMethodFamily keeps exact-before-rest resolution without consulting the
// receiver's class for a new implementation.

class Adder {
  sum(_ a, _ b) { -1 }
  sum(*numbers) {
    let total = 0
    numbers.each(|n| { total = total + n })
    return total
  }
}

const methods = Adder >> #sum(...)
System.print(methods.size)
const bound = methods.bind(Adder.new())
System.print(bound(1, 2))
System.print(bound(1, 2, 3))
