// area: string
// spec: string-interpolation.md §8
// status: PASS

class CustomString {
  @constructor
  new(_ text) {
    _text = text
  }

  toString {
    return "custom:" + _text
  }
}

let c = CustomString.new("hello")
System.print("\(c)")
