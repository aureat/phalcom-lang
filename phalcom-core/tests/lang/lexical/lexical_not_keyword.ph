// area: lexical/operators
// spec: syntax/grammar.md (`unary := ( "-" | "!" | "not" ) unary`);
//   syntax/expressions.md §precedence row 9
// status: PASS
// U-ERR-FIX NOT-KEYWORD: `not` was a dead token — it lexed but the parser
// never consumed it as a unary prefix operator, so `not true` was a parse
// error even though the grammar explicitly lists `not` alongside `!` at the
// same unary-prefix precedence. Wired as a synonym for `!` (same
// `UnaryOp::Not`), not removed, since the spec lists it.
System.print(not true)
System.print(not false)
System.print(not not true)
