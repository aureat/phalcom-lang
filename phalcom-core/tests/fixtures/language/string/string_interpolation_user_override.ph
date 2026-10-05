// area: string
// spec: string-interpolation.md §8
// status: PASS

class CustomString {
  mut _text
  @constructor
  new(_ text) {
    _text = text
  }

  toString {
    return "custom:" + _text
  }
}

let mut c = CustomString.new("hello")
System.print("\(c)")
