// area: control flow
// spec: if let and while let

// 1. if let with Some / None
let opt = Some(42)
if let Some(x) = opt {
  System.print("some: \(x)")
} else {
  System.print("none")
}

let optNone = None
if let Some(y) = optNone {
  System.print("some: \(y)")
} else {
  System.print("none")
}

// 2. if let with Tuple and Record pattern
let pair = (10, 20)
if let (a, b) = pair {
  System.print("pair: \(a), \(b)")
}

let rec = #{ name: "Phalcom", version: 1 }
if let #{ name: n, version: v } = rec {
  System.print("rec: \(n) v\(v)")
}

// 3. while let with iterator/cursor
class Counter {
  @get _curr
  @constructor new() { _curr = 0 }

  next {
    if (_curr < 3) {
      let v = _curr
      _curr = _curr + 1
      return Some(v)
    }
    None
  }
}

let c = Counter.new()
while let Some(val) = c.next {
  System.print("count: \(val)")
}

// 4. while let with tuple destructuring
class PairGen {
  @get _step
  @constructor new() { _step = 0 }

  next {
    if (_step < 2) {
      let s = _step
      _step = _step + 1
      return Some((s, s * 10))
    }
    None
  }
}

let pg = PairGen.new()
while let Some((idx, mult)) = pg.next {
  System.print("pair: \(idx), \(mult)")
}
