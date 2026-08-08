// area: collections
// spec: E.2 §13
// status: PASS
const evens = (0..).iter.filter |x| { x % 2 == 0 }.take(5).toList
System.print(evens)
