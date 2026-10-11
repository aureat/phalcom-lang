// area: bindings
// spec: open-questions.md Q7; ADR-0046
// status: PASS
// U14: `const [first, *rest] = list` — the rest sub-pattern binds a fresh
// `List` holding everything from index `elements.len()` onward.
let values: List<Int> = [1, 2, 3]
let [first, *rest] = values
System.print(first)
System.print(rest.size)
System.print(rest.at(0))
System.print(rest.at(1))
