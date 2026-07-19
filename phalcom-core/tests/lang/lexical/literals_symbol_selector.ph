// area: lexical
// spec: selectors.md §2
// status: PASS
// A selector symbol canonicalizes at compile time: whitespace inside the
// parens is free (stripped), and the no-space + multi-line spellings below
// intern to the same Symbol. A bare name symbol (`#size`) is distinct from
// the zero-arity selector symbol it shares a spelling prefix with (`#size()`)
// — a name symbol identifies a family, not a method.

const a = #move(_,to,duration)
const b = #move(
  _,
  to,
  duration
)
System.print(a)
System.print(a == b)
System.print(#size())
System.print(#size() == #size)
