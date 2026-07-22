// area: string
// spec: string-interpolation.md §8
// status: PASS

let trace = List.new()

class Probe {
  construct new(label, trace) {
    _label = label
    _trace = trace
  }

  value {
    _trace.add("expr:" + _label)
    return self
  }

  toString {
    _trace.add("string:" + _label)
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
