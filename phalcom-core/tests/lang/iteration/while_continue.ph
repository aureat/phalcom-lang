// U-ITER-FIX item 2 (spec §3.2): `continue` inside a bare `while` body skips
// straight to the next condition retest — there is no separate step label
// for `while`, since re-evaluating the condition block *is* the step. Before
// this fix, a bare `while` pushed no `LoopContext`, so `continue` here raised
// the out-of-loop compile error (C-ITER-7). Over i = 0..4: skip printing at
// i == 2, otherwise print i, still incrementing every iteration.
var i = 0
while (i < 4) {
  i = i + 1
  if (i == 2) { continue }
  System.print(i)
}
System.print("done")
