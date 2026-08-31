// C-ITER-6 (spec §4, §8.3), mixed-loop-kind case: a `while` nested inside a
// `for` — `break` inside the `while` body leaves only the `while`; the `for`
// keeps iterating its own elements. Over a=[1,2]: each `a` prints, then the
// nested `while` counts up and breaks at count==2, then "outer" prints.
for a in [1, 2] {
  System.print(a)
  let count = 0
  while (count < 5) {
    (count == 2).ifTrue || { break }
    System.print(count)
    count = count + 1
  }
  System.print("outer")
}
