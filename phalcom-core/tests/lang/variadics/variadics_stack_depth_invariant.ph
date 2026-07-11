// area: variadics
// spec: U9-implementation-spec.md §2, §6
// status: PASS
// Black-box stack-depth invariant: the call prologue's rest-arg collapse
// (`Vec::split_off` + one `List` push) must leave the VM's value stack at
// exactly its pre-call depth. Run many variadic calls in a loop inside one
// program — a stack leak would show up as a wrong final result or a panic
// well before the loop completes.

class Summer {
  sum(*numbers) {
    var total = 0
    numbers.each({ n => total = total + n })
    return total
  }
}
let s = Summer.new()
var i = 0
var last = 0
while (i < 200) {
  last = s.sum(1, 2, 3, 4, 5)
  i = i + 1
}
System.print(last)
System.print(i)
