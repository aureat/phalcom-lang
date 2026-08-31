// area: lexical/operators
// spec: syntax/grammar.md (`unary := ( "-" | "not" ) unary`);
//   syntax/expressions.md §precedence row 9
// status: PASS
// U-ERR-FIX NOT-KEYWORD wired the previously-dead `not` token as a unary
// prefix operator alongside `!`. U-NEG then retired prefix `!` outright, so
// `not` is now the sole boolean-negation prefix (`UnaryOp::Not`); `!=`
// remains its own two-character token, untouched.
System.print(not true)
System.print(not false)
System.print(not not true)
