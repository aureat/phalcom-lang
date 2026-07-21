@construct
class Pair {
  _first
  _second
  first => _first
  second => _second
}
let p = Pair.new(first: 1, second: 2)
System.print(p.first)
System.print(p.second)
