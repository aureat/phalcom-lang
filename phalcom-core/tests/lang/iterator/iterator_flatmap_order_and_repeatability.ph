// area: iterator
// spec: E.1
// status: PASS
let calls = 0
let pipeline = [1, 2].iter.flatMap |x| { calls = calls + 1; [x, x * 10] }
System.print(pipeline.toList.toString)
System.print(calls)
System.print(pipeline.toList.toString)
System.print(calls)
