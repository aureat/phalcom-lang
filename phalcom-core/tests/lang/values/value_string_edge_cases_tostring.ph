// area: values
// spec: values-and-absence.md; U-CORE-4 (R-INV-4.1); lexer string-literal scan
// status: PASS
// Adversarial: an empty string, a string with an embedded literal newline
// (the lexer has no `\n` escape — a raw newline inside the quotes is scanned
// as-is), and a string with an escaped backslash (`\\` -> one literal `\`,
// the lexer's only recognized escape besides `\(`-interpolation) all render
// through `String#toString`'s `=> self` unchanged.

System.print("".toString)
System.print("line1\nline2".toString)
System.print("a\\b".toString)
