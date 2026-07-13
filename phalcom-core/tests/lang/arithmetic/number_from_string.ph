// area: arithmetic/operators
// spec: values-and-absence.md
// status: PASS
// ported from wren/test/core/number/from_string.wren: `Number.new(_)` is
// Phalcom's coercion constructor (there is no separate `fromString`,
// U-CORE-1's flat Number, ADR-0005), so it plays the role Wren's
// `Num.fromString` plays there.
System.print(Number.new("123") == 123)
System.print(Number.new("-123") == -123)
System.print(Number.new("-0") == -0)
System.print(Number.new("12.34") == 12.34)
System.print(Number.new("-0.0001") == -0.0001)
