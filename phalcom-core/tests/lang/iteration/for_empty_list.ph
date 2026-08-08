// C-ITER-2 (iteration.md §3.3): `for` over an empty `List` runs the body zero
// times — the first `iterate(None)` is already `None`.
System.print("before")
for (x in []) { System.print("never") }
System.print("after")
