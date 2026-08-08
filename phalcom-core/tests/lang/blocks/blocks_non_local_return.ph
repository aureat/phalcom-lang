// area: blocks
// spec: blocks.md §5; functions.md §2; ADR-0013 (non-local return)
// status: PASS
// A `return` inside a block unwinds to the *enclosing method* activation, not
// just the block's own frame. `findNegative` iterates with `List.each`, which
// is `.ph`-defined and calls `f.call(...)` per element — so the `return n`
// executes inside a re-entrant `block_call`/`run_until`, crossing more than one
// call boundary before it reaches `findNegative`'s frame. This is the
// multi-level unwind U10 exists for: a single-level `{ return x }.call()` would
// never exercise it (U10-implementation-spec.md §2, §5).
class Finder {
  findNegative(_ numbers) {
    numbers.each |n| {
      (n < 0).ifTrue || { return n }
    }
    return None
  }
}
const numbers = List.new()
numbers.add(3)
numbers.add(-5)
numbers.add(8)
System.print(Finder.new().findNegative(numbers))
