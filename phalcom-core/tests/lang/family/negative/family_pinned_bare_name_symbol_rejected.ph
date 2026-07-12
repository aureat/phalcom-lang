// area: family/negative
// spec: selectors.md §3 (Pinned form — LOCKED ambiguity rule, U16-Pinned)
//   — a bare name symbol after `::` (`::#name`, no parens) is neither an
//   Open reference (that needs an identifier, not a `#` symbol) nor a valid
//   Pinned reference (Pinned requires the full selector form). Rejected at
//   parse time with a clear diagnostic.
// status: NEGATIVE

class Foo {
  bar() { return 1 }
}
let f = Foo.new()
let g = f::#bar
System.print(g)
