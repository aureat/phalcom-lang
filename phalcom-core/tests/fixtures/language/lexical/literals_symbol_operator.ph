// area: lexical
// spec: selectors.md §2
// status: PASS
// Bare operator symbols (`#+`, `#==`) preserve their spelling without
// inferring selector arity; explicit slots select operator methods.

System.print(#+)
System.print(#==)
System.print(#+ == #+)
