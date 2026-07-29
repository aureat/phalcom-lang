// A class that defines [] but NOT at — xs[i] works, xs.at(i) doesNotUnderstand.
class OnlyIndex {
  @constructor
  new() {}
  [i] { return i * 2 }
}
const obj = OnlyIndex.new()
System.print(obj[3])
