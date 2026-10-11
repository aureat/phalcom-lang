// area: control flow
// spec: control-flow.md
// status: PASS

let mut i = 0
while (i < 6) {
  i = i + 1
  if (i == 2) {
    continue
  }
  if (i == 5) {
    break
  }
  System.print(i)
}
