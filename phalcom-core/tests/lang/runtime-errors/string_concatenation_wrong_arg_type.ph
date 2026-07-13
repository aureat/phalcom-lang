// area: string
// spec: object-model.md; core/core-classes.md
// status: NEGATIVE
// Wren precedent: test/core/string/concatenation_wrong_arg_type.wren
// (Wren: "Right operand must be a string."). `String::+(_)` (`string_add`,
// primitive/string.rs) calls `expect_string` on both operands; a non-string
// right operand fails the guard with a `RuntimeError::Type`, never silently
// coerces.
System.print("a" + 123)
