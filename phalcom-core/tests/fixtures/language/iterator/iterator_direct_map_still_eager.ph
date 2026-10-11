// area: iterator
// spec: E.1
// status: PASS
let mut calls = 0
let values: List<Int> = [1, 2, 3]
let mut result = values.map |x| { calls = calls + 1; x * 2 }
System.print(calls)
System.print(result.is(List))
System.print(result.at(2))
