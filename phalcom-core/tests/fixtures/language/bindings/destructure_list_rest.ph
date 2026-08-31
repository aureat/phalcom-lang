// area: bindings
// spec: open-questions.md Q7; ADR-0046
// status: PASS
// U14: `const [first, *rest] = list` — the rest sub-pattern binds a fresh
// `List` holding everything from index `elements.len()` onward.
const [first, *rest] = [1, 2, 3]
System.print(first)
System.print(rest.size)
System.print(rest.at(0))
System.print(rest.at(1))
