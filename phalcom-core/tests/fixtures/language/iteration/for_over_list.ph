// C-ITER-1 (ADR-0035 §2, iteration.md §2): `for` over a `List` visits the
// elements in order, driven by the cursor protocol (never `.each`).
let values: List<Int> = [10, 20, 30]
for x in values { System.print(x) }
