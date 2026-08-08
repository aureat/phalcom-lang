// area: iterator
// spec: E.1
// status: PASS
let calls = 0
let pipeline = [1, 2, 3].iter.map |x| { calls = calls + 1; x * 2 }
System.print(calls)
System.print(pipeline.iter == pipeline)
System.print(pipeline.toList.at(1))
System.print(calls)
