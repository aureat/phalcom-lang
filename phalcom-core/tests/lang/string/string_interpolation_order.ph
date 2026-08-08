// area: string
// spec: string-interpolation.md §8
// status: PASS

let trace = []

class Probe {
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

let a = Probe.new("a", trace)
let b = Probe.new("b", trace)

let result = "\(a.value)-\(b.value)"
System.print(result)
System.print(trace.at(0))
System.print(trace.at(1))
System.print(trace.at(2))
System.print(trace.at(3))
