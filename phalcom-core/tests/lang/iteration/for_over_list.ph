// C-ITER-1 (ADR-0035 §2, iteration.md §2): `for` over a `List` visits the
// elements in order, driven by the cursor protocol (never `.each`).
for (x in List.new().add(10).add(20).add(30)) { System.print(x) }
