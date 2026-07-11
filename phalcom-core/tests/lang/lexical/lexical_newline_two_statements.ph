// area: lexical/statements
// spec: lexical-structure.md
// status: PASS
// D3 guard: a newline after a value-ending token is preserved, so two
// consecutive `System.print` statements on separate lines both run.
System.print(1)
System.print(2)
