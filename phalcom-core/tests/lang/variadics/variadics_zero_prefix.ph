// area: variadics
// spec: U9-implementation-spec.md §2, §6; messages-and-selectors.md §4
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
