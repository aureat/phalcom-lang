// A1-A3 product regression: Record equality and hash ignore encounter order.

let first = {a: 1, b: 2}
let second = {b: 2, a: 1}

System.print(first == second)
System.print(first.hash == second.hash)
