// area: lexical
// spec: selectors.md §2
// status: PASS
// Bare operator symbols (`#+`, `#==`) always lex as a one-argument selector
// symbol, matching every operator method definition's arity.

System.print(#+)
System.print(#==)
System.print(#+ == #+)
