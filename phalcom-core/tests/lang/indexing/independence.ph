// A class that defines [] but NOT at — xs[i] works, xs.at(i) doesNotUnderstand.
class OnlyIndex {
  construct new() {}
  [i] { return i * 2 }
}
let obj = OnlyIndex.new()
System.print(obj[3])
