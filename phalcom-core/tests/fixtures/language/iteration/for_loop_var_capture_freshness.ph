// U-ITER-FIX item 3 (spec §3.3): the loop variable is one stack slot rebound
// every iteration. A closure captured over it inside the body must see the
// per-step value it closed over, not whatever the slot held on the loop's
// *final* iteration — matches the inlined-`while` capture behavior
// (blocks_shared_mutation.ph's `let` capture, applied per-iteration here
// instead of shared across calls). Before this fix all three closures shared
// one open upvalue cell and printed [3, 3, 3]; each iteration now closes its
// own cell before rebinding, so calling them afterward prints [0, 1, 2].
let closures = []
for x in [0, 1, 2] {
  closures.append(|| { x })
}
for c in closures {
  System.print(c.call())
}
