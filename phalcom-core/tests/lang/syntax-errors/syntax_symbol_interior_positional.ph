// area: errors
// spec: selectors.md §1 R2, §2
// status: NEGATIVE
// R2 — positionals precede labels. `#move(to,_)` has a positional slot `_`
// interior to a label (`to`), which is illegal and rejected at lex time with
// a precise span (selectors.md §2 "Malformed contents ... are a lex-time
// error").

let a = #move(to,_)
