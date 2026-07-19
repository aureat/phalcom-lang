const numbers = 5

const square = n => n * n
const sum = { a, b => a + b }

System.print(square.call(numbers))
System.print(sum.call(3, 4))
