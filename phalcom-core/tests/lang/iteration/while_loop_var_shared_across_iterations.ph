// U-ITER-FIX item 3 counterpoint (spec §3.3): the per-iteration freshness fix
// applies to `for`'s IMPLICIT loop variable, which the compiler rebinds to a
// fresh slot each step. A bare `while` has no such machinery — its counter is
// one `let` declared ONCE before the loop and mutated in place every
// iteration, exactly like `blocks_shared_mutation.ph`. Closures captured over
// it all alias the SAME open upvalue cell, so calling them after the loop
// prints the counter's FINAL value three times ([3, 3, 3]), not [0, 1, 2].
let closures = List.new()
let i = 0
while (i < 3) {
  closures.add(|| { i })
  i = i + 1
}
for (c in closures) {
  System.print(c.call())
}
