// area: string
// spec: string-interpolation.md §8
// status: PASS

let mut trace: List<String> = []

class Probe {
  mut _label
  mut _trace
  @constructor
  new(_ label, _ trace) {
    _label = label
    _trace = trace
  }

  value {
    _trace.append("expr:" + _label)
    return self
  }

  toString {
    _trace.append("string:" + _label)
    return _label
  }
}

let mut a = Probe.new("a", trace)
let mut b = Probe.new("b", trace)

let mut result = "\(a.value)-\(b.value)"
System.print(result)
System.print(trace.at(0))
System.print(trace.at(1))
System.print(trace.at(2))
System.print(trace.at(3))
