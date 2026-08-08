// area: blocks
// spec: blocks.md §5; ADR-0013 (non-local return); values-and-absence.md
// status: PASS
// A bare `return` (no expression) inside a block still performs a non-local
// return, unwinding to the enclosing method — and surfaces the `None` singleton,
// never the private `Nil` sentinel, mirroring `Bytecode::Return`'s bare-return
// handling. The `return` lives inside an `each` block, so it crosses a
// re-entrant `block_call` just like the value-carrying case.
class Finder {
  firstNeg(_ numbers) {
    numbers.each |n| {
      (n < 0).ifTrue { return }
    }
    return 99
  }
}
const numbers = List.new()
numbers.add(1)
numbers.add(-2)
System.print(Finder.new().firstNeg(numbers))
