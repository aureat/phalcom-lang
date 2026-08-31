// area: string
// spec: string-interpolation.md §8
// status: NEGATIVE

class BadToString {
  @constructor
  new() {}

  toString {
    return 123
  }
}

let bad = BadToString.new()
System.print("\(bad)")
