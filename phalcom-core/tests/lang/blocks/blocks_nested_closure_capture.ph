// area: blocks
// spec: blocks.md §5 (nested closures)
// status: PASS
// A block that RETURNS another block: the inner block captures `base` from
// the outer block's own frame, two capture-levels removed from `main`. The
// inner closure keeps working correctly after the outer block's frame is
// gone (chained open->closed upvalue promotion), and is reusable.
const makeAdder = { base =>
  { n => n + base }
}
const addTen = makeAdder.call(10)
System.print(addTen.call(5))
System.print(addTen.call(32))
