// area: classes
// spec: object-model.md; messages-and-selectors.md
// status: PASS
// Ported from Wren `test/core/class/equality.wren`: a `Class` is itself an
// object and compares by identity — equal to itself, unequal to a different
// class, and unequal to any non-class value.

System.print(Number == Number)
System.print(Number == Bool)

// Not equal to other types.
System.print(Number == 123)
System.print(Number == true)

System.print(Number != Number)
System.print(Number != Bool)

// Not equal to other types.
System.print(Number != 123)
System.print(Number != true)
