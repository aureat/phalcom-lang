// area: control flow
// spec: control-flow.md §1
// status: PASS
// U5: keyword sugar and the explicit sacred-selector send compile to
// observationally identical behavior — both are the same message send
// (`if/else` === `ifTrue(_:ifFalse:)`, `while` === `whileTrue(_:)`).

System.print(if (3 > 2) { "yes" } else { "no" })
System.print((3 > 2).ifTrue({ "yes" }, ifFalse: { "no" }))

let i = 0
while (i < 2) {
  System.print(i)
  i = i + 1
}

let j = 0
{ j < 2 }.whileTrue {
  System.print(j)
  j = j + 1
}
