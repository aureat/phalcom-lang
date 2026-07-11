// area: blocks
// spec: blocks.md §5; ADR-0013 (open/closed upvalues)
// status: PASS
// The counter's `count` local must be promoted from an open (stack-aliasing)
// upvalue to a closed (heap-owned) one when `makeCounter`'s frame returns, so
// the escaped block keeps working and keeps incrementing shared state.
let makeCounter = {
  let count = 0
  { count = count + 1 }
}

let counter = makeCounter.call()
System.print(counter.call())
System.print(counter.call())
System.print(counter.call())
