// area: control flow
// spec: control-flow.md
// status: PASS
// short-circuit and proven by a side-effecting counter, not just avoided div-by-zero.
let count = 0
const bump = {
  count = count + 1
  true
}
System.print(false and bump.call())
System.print(count)