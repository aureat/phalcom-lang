// area: string
// spec: object-model.md; values-and-absence.md
// status: PASS
// Wren precedent: test/core/string/equality.wren. Ported 1:1 except the
// 8-bit-clean `"a\0b\0c"` cases, which do not apply (no `\0` escape in the
// lexer — see string_concatenation.ph's note). `==`/`!=` are ordinary sends
// (`Object::==`, control-flow.md §1) that fall back to
// `Value::value_eq`, which compares two strings by content and never equates
// a `String` to a value of another type.
System.print("" == "")
System.print("abcd" == "abcd")
System.print("abcd" == "d")
System.print("e" == "abcd")
System.print("" == "abcd")

// Not equal to other types.
System.print("1" == 1)
System.print("true" == true)

System.print("" != "")
System.print("abcd" != "abcd")
System.print("abcd" != "d")
System.print("e" != "abcd")
System.print("" != "abcd")

// Not equal to other types.
System.print("1" != 1)
System.print("true" != true)

// Non-ASCII: content equality is a Rust `String` (UTF-8) comparison, not a
// byte-length or ASCII-only comparison.
System.print("vålue" == "value")
System.print("vålue" == "vålue")
