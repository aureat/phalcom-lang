// C-ITER-3 (iteration.md §3.3): the iterable expression is evaluated exactly
// once — bound to a synthetic temporary before the loop — so a side-effecting
// receiver runs a single time.
class Source {
  mut _calls
  @constructor
  new() { _calls = 0 }
  makeList {
    _calls = _calls + 1
    System.print("built")
    return [1, 2]
  }
  calls { _calls }
}
let s = Source.new()
for x in s.makeList { System.print(x) }
System.print(s.calls)
