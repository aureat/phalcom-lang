// area: arithmetic/operators
// spec: messages-and-selectors.md; object-model.md
// status: PASS
// ported from wren/test/core/number/equality.wren: a Number is never == to a
// value of another class, even when the printed forms coincide.
System.print(123 == "123")
System.print(1 == true)
System.print(0 == false)
System.print(123 != "123")
System.print(1 != true)
System.print(0 != false)
