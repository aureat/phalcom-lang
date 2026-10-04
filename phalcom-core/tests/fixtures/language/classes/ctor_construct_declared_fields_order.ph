@construct
class Pair {
  mut _first
  mut _second
  first { _first }
  second { _second }
}
let mut p = Pair.new(first: 1, second: 2)
System.print(p.first)
System.print(p.second)
