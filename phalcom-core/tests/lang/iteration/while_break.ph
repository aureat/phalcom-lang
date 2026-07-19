// U-ITER-FIX item 2 (spec §3.2): `break` inside a bare `while` body leaves
// the loop, exactly as it does inside `for`. Before this fix, a bare `while`
// pushed no `LoopContext`, so `break` here raised the out-of-loop compile
// error (C-ITER-7). Over i = 0..4: prints 0, 1, breaks at i == 2.
let i = 0
while (i < 4) {
  if (i == 2) { break }
  System.print(i)
  i = i + 1
}
System.print("done")
