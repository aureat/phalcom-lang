// C-ITER-6 (spec §4, §8.3): in nested loops `break`/`continue` bind to the
// INNERMOST loop only. The inner `break` at b==20 leaves just the inner loop;
// the outer loop keeps going. The inner `continue` at b==11 skips only b==11.
for (a in List.new().add(1).add(2)) {
  for (b in List.new().add(10).add(11).add(20).add(30)) {
    if (b == 11) { continue }
    if (b == 20) { break }
    System.print(a)
    System.print(b)
  }
  System.print("outer")
}
