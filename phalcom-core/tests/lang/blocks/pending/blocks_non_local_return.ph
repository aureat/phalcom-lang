// area: blocks
// spec: blocks.md
// status: PENDING

class Finder {
  findNegative(numbers) {
    numbers.each { n =>
      (n < 0).ifTrue { return n }
    }
    return None
  }
}
let numbers = [3, -5, 8]
System.print(Finder.new().findNegative(numbers))
