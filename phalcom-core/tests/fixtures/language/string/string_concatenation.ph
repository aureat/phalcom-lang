// area: string
// spec: object-model.md; core/core-classes.md
// status: PASS
// Wren precedent: test/core/string/concatenation.wren. The 8-bit-clean
// `"a\0b" + "\0c"` half of the Wren original does not port — Phalcom's
// lexer has no escape-sequence table at all (only `\\` -> `\` and the
// `\(expr)` interpolation escape; see lexer.rs's string-scan loop), so a
// literal `\0` reads as two source characters, not a NUL byte. The
// content-equality half of the same test carries over directly: `+` builds a
// fresh `String` and `==` compares by content (not handle identity).
System.print("a" + "b")
System.print(("a" + "b") == "ab")
System.print(("hello, " + "world") == "hello, world")
