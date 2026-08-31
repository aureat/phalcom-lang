// area: iterator
// spec: E.1
// status: PASS
let skipped = [1, 2, 3, 4, 5].iter.skip(3).toList
let taken = [1, 2, 3, 4, 5].iter.take(3).toList
let prefix = [1, 2, 3, 1].iter.takeWhile |x| { x < 3 }.toList
System.print(skipped.at(0))
System.print(skipped.at(1))
System.print(taken.toString)
System.print(prefix.toString)
