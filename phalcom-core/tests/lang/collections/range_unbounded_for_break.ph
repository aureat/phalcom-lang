// area: collections
// spec: E.2 §15
// status: PASS
let found = None
for (x in 0..) {
  if (x == 10) { found = x; break }
}
System.print(found)
