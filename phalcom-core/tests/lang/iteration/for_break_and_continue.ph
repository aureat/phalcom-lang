// C-ITER-6 (ADR-0035 §3, iteration.md §3): `continue` skips to the next
// `iterate` (the cursor still advances), `break` leaves the loop. Over
// [1,2,3,4,5]: skip 2, stop at 4 -> prints 1, 3.
for (v in List.new().add(1).add(2).add(3).add(4).add(5)) {
  if (v == 2) { continue }
  if (v == 4) { break }
  System.print(v)
}
System.print("after")
