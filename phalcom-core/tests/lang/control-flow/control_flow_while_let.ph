// area: control flow
// spec: control-flow.md
// status: PASS
// U5: `while (cond) { body }` desugars to `{ cond }.whileTrue { body }`
// (control-flow.md §1/§3). Uses `let` (not `var`) since `var` lands in U6.

let i = 0
while (i < 3) {
  System.print(i)
  i = i + 1
}
