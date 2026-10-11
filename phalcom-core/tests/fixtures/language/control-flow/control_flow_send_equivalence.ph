// area: control flow
// spec: control-flow.md §1
// status: PASS
// The keyword control forms and explicit sacred-selector sends have
// observationally compatible runtime behavior, but retain separate semantic
// identities (`if/else` vs. `ifTrue(_:ifFalse:)`, `while` vs. `whileTrue(_)`).

System.print(if (3 > 2) { "yes" } else { "no" })
System.print((3 > 2).ifTrue(|| { "yes" }, ifFalse: || { "no" }))

let mut i = 0
while (i < 2) {
  System.print(i)
  i = i + 1
}

let mut j = 0
|| { j < 2 }.whileTrue || {
  System.print(j)
  j = j + 1
}
