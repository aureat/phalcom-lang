// area: lexical/literals
// spec: lexical-structure.md
// status: PASS
// D2: `_` digit separators work on both sides of the decimal point, but never
// adjacent to it — `1_000.500_5` decodes to `1000.5005`.
System.print(1_000.500_5)
