// area: blocks
// spec: blocks.md §5; iteration.md; ADR-0013 (non-local return)
// status: PASS
// `return` inside a `for` loop's body unwinds all the way to the enclosing
// METHOD activation, not merely out of the loop — the loop's own `break`
// machinery is a jump within the same function and is not what fires here;
// this is a full non-local return crossing the method boundary. `"after"`
// must never print.
class Scanner {
  firstOver(_ numbers, _ limit) {
    for (n in numbers) {
      (n > limit).ifTrue || { return n }
    }
    return None
  }
}
const numbers = []
numbers.append(1)
numbers.append(5)
numbers.append(9)
numbers.append(2)
System.print(Scanner.new().firstOver(numbers, 4))
System.print("after")
