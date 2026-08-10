// area: rest
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md §11-15
// status: PASS
// F.3 positional rest captures a canonical Tuple, with Unit for an empty
// residual lane.

class Summer {
  count(*numbers) {
    return numbers.size
  }
  empty(*numbers) { return numbers }
}
const s = Summer.new()
System.print(s.count(1, 2, 3))
System.print(s.empty())
