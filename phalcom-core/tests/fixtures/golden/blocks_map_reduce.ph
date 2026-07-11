let numbers = 5

let square = n => n * n
let sum = { a, b => a + b }

System.print(square.call(numbers))
System.print(sum.call(3, 4))
