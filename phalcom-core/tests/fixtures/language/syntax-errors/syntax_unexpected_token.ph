// area: errors
// spec: lexical-structure.md; implementation-status.md
// status: NEGATIVE
// A stray token where an expression is expected. (Previously this fixture used
// `[1, 2, 3]`, which U-COLL made a valid list literal; migrated to a retired arrow that
// can never begin an expression so the "unexpected token" intent is preserved.)

System.print(=> 3)
