// C-ITER-6 (spec §4, §8.3), mixed-loop-kind case: a `for` nested inside a
// `while` — `continue` inside the `for` body skips only that `for` element;
// the outer `while` is unaffected and keeps counting. Over outer w=[0,1]:
// each pass prints "outer", then the nested `for` over [10,20,30] skips 20
// via `continue` and prints the rest.
let w = 0
while (w < 2) {
  System.print("outer")
  for (n in List.new().add(10).add(20).add(30)) {
    (n == 20).ifTrue || { continue }
    System.print(n)
  }
  w = w + 1
}
