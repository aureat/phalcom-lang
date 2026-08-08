// area: dispatch
// spec: messages-and-selectors.md
// status: PENDING

class Summer {
  sum(*numbers) {
    return numbers.reduce(0) |acc, n| { acc + n }
  }
}
System.print(Summer.new().sum(1, 2, 3))
