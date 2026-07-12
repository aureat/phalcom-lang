// area: bindings
// spec: open-questions.md Q7; ADR-0046
// status: PASS
// U14: `let [first, *rest] = list` — the rest sub-pattern binds a fresh
// `List` holding everything from index `elements.len()` onward (U9's `*name`
// spelling reused verbatim, messages-and-selectors.md §5 spread parity).
let [first, *rest] = [1, 2, 3]
System.print(first)
System.print(rest.size)
System.print(rest.at(0))
System.print(rest.at(1))
