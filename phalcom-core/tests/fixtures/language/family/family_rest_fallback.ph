// area: family
// spec: docs/spec/callables/family.md §3; docs/spec/callables/reflection.md §2
// status: PASS

class Adder {
  sum(_ a, _ b) { -1 }
  sum(*numbers) {
    let total = 0
    numbers.each(|n| { total = total + n })
    return total
  }
}

const family = (Adder >> #sum(...)).bind(Adder.new())
System.print(family(1, 2))
System.print(family(1, 2, 3))
