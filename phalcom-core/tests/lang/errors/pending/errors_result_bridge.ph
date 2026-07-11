// area: errors
// spec: error-handling.md; values-and-absence.md
// status: PENDING

let parsed = { Number.parse("42") }.attempt()
System.print(parsed.map { n => n * 2 }.unwrapOr(0))
