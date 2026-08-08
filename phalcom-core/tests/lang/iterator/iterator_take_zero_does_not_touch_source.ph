// area: iterator
// spec: E.1
// status: PASS
let calls = 0
let pipeline = [1, 2, 3].iter.map |x| { calls = calls + 1; x }.take(0)
pipeline.toList
System.print(calls)
