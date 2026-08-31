// area: blocks
// spec: blocks.md §5; functions.md §2; ADR-0013 (non-local return)
// status: PASS
// `return` lives inside TWO nested blocks (an outer `ifTrue` block wrapping
// an inner `ifTrue` block), both reached through `List#each`'s native
// `block_call` re-entry. The non-local return must unwind past both block
// frames AND the native `each` frame to the enclosing method activation.
class Finder {
  findFirstEven(_ numbers) {
    numbers.each |n| {
      (n > 0).ifTrue || {
        (n % 2 == 0).ifTrue || { return n }
      }
    }
    return None
  }
}
const numbers = []
numbers.append(-4)
numbers.append(3)
numbers.append(7)
numbers.append(8)
numbers.append(10)
System.print(Finder.new().findFirstEven(numbers))
