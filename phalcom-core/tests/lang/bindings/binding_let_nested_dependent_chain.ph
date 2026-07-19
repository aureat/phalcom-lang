// area: bindings
// spec: values-and-absence.md; open-questions.md; ADR-0014
// status: PASS
// Nested `const ... const ... const` chains where each binding depends on the
// prior one, with an inner block shadowing an outer name (`a`) while still
// reading an outer name it does NOT shadow (`c`) — the shadow is scoped to
// the inner block only, and outer `a` survives unchanged after the block.

const a = 2
const b = a * a
const c = b + a
const d = {
  const a = 10
  const b = a + c
  b
}.call()
System.print(c)
System.print(d)
System.print(a)
