// area: control flow
// spec: control-flow.md §1; ADR-0017
// status: PASS
// U5 Layer 1: a `return` inside an inlined `if`/`while` block body is a
// non-local return out of the enclosing method, not merely out of the block.
// Here the `return i` fires from inside the inlined `while` + `if` and exits
// `firstMatch` immediately at i == 3, so the trailing `return 0 - 1` is dead.

class Runner {
  @class
  firstMatch {
    let i = 0
    while (i < 10) {
      if (i == 3) { return i }
      i = i + 1
    }
    return 0 - 1
  }
}
System.print(Runner.firstMatch)
