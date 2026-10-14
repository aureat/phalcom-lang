// area: control flow
// spec: if let and while let

// 1. if let with Some / None
let mut opt = Option::Some(42)
if let Option::Some(x) = opt {
  System.print("some: \(x)")
} else {
  System.print("none")
}

class Helper {
  @class makeNone -> Option<Int> { Option::None }
}
let mut optNone = Helper.makeNone
if let Option::Some(y) = optNone {
  System.print("some: \(y)")
} else {
  System.print("none")
}

// 2. if let with Tuple and Record pattern
let mut pair = (10, 20)
if let (a, b) = pair {
  System.print("pair: \(a), \(b)")
}

let mut rec = { name: "Phalcom", version: 1 }
if let { name: n, version: v } = rec {
  System.print("rec: \(n) v\(v)")
}

// 3. while let with iterator/cursor
class Counter {
  @get mut _curr
  @constructor new() { _curr = 0 }

  next -> Option<Int> {
    if (_curr < 3) {
      let mut v = _curr
      _curr = _curr + 1
      return Option::Some(v)
    }
    Option::None
  }
}

let mut c = Counter.new()
while let Option::Some(val) = c.next {
  System.print("count: \(val)")
}

// 4. while let with tuple destructuring
class PairGen {
  @get mut _step
  @constructor new() { _step = 0 }

  next -> Option<(Int, Int)> {
    if (_step < 2) {
      let mut s = _step
      _step = _step + 1
      return Option::Some((s, s * 10))
    }
    Option::None
  }
}

let mut pg = PairGen.new()
while let Option::Some((idx, mult)) = pg.next {
  System.print("pair: \(idx), \(mult)")
}

// 5. while let with explicit type annotation
let mut c2 = Counter.new()
while let Option::Some(v): Option<Int> = c2.next {
  System.print("annotated: \(v)")
}
