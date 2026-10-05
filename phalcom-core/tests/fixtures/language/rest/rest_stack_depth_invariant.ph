// area: rest
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md §11-14
// status: PASS
// Black-box stack-depth invariant: rest capture must leave the VM value stack
// at its pre-call depth across many calls in one program.

class Summer {
  sum(*numbers) {
    let mut total = 0
    numbers.each(|n| { total = total + n })
    return total
  }
}
let s = Summer.new()
let mut i = 0
let mut last = 0
while (i < 200) {
  last = s.sum(1, 2, 3, 4, 5)
  i = i + 1
}
System.print(last)
System.print(i)
