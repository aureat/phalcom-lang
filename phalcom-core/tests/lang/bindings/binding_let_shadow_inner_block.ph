// area: bindings
// spec: values-and-absence.md; open-questions.md; ADR-0014; blocks.md §5
// status: PASS
// An inner block's `const x` shadows the enclosing `const x` for the duration
// of the block's own frame; once the block returns, the OUTER binding's
// original value is unaffected — the shadow never wrote through to the
// outer slot (a fresh local, not an upvalue alias).

const x = 1
const show = {
  const x = 2
  System.print(x)
}
show.call()
System.print(x)
