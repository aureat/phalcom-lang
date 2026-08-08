// area: collections
// spec: E.2 §13
// status: PASS
const firstTen = (0..).iter.map |x| { x * 2 }.take(10).toList
System.print(firstTen)
