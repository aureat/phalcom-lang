// area: errors
// spec: lexical-structure.md
// status: NEGATIVE
// D2: a doubled `__` separator is not flanked by digits, so it is rejected as
// an invalid token rather than silently stripped.
System.print(1__0)
