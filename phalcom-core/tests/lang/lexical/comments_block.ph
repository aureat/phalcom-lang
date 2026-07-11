// area: lexical/comments
// spec: lexical-structure.md
// status: PASS
// D1: `/* … */` block comments are trivia — an inline one is skipped, and a
// multi-line one does not leak a newline into the token stream.
System.print(/* inline */ 1)
/* line one
   line two */
System.print(2)
