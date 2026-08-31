// area: control flow
// spec: control-flow.md
// status: PASS
// U5: `while (cond) { body }` desugars to `{ cond }.whileTrue || { body }`
// (control-flow.md §1/§3). U6: the loop counter is reassigned, so it is a
// mutable `let` binding (ADR-0014); a `const` here would be an AssignToImmutable
// compile error.

let i = 0
while (i < 3) {
  System.print(i)
  i = i + 1
}
