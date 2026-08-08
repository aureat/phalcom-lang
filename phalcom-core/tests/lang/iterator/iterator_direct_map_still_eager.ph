// area: iterator
// spec: E.1
// status: PASS
let calls = 0
let result = [1, 2, 3].map |x| { calls = calls + 1; x * 2 }
System.print(calls)
System.print(result.isA(List))
System.print(result.at(2))
