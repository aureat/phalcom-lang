// area: control flow
// spec: control-flow.md
// status: PASS

let i = 0
|| { i < 3 }.whileTrue || {
  System.print(i)
  i = i + 1
}
