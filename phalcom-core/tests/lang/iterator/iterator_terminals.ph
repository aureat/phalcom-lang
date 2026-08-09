// area: iterator
// spec: E.1
// status: PASS
let pipeline = [1, 2, 3, 4].iter.filter |x| { x > 1 }
System.print(pipeline.count)
System.print(pipeline.fold(initial: 0, using: |acc, x| { acc + x }))
System.print(pipeline.toSet.size)
