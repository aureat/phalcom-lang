// area: lexical/statements
// spec: lexical-structure.md
// status: PASS
// D3: `and`/`or` are line-end suppressors too, so a boolean expression can be
// broken after the logical keyword.
let ok = true and
         false or
         true
System.print(ok)
