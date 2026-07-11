// area: control flow
// spec: control-flow.md
// status: PENDING

var i = 0
{ i < 3 }.whileTrue {
  System.print(i)
  i = i + 1
}
