// area: iterator
// spec: E.1
// status: PASS
let first = [1, 2, 3, 4].iter.map |x| { x * 2 }.filter |x| { x > 4 }.toList
let second = [1, 2, 3, 4].iter.filter |x| { x > 2 }.map |x| { x * 2 }.toList
System.print(first.at(0))
System.print(first.at(1))
System.print(second.at(0))
System.print(second.at(1))
