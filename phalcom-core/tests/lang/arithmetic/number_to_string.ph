// area: arithmetic/operators
// spec: values-and-absence.md
// status: PASS
// ported from wren/test/core/number/to_string.wren.
System.print(123.toString == "123")
System.print((-123).toString == "-123")
System.print((-0).toString == "-0")
System.print(12.34.toString == "12.34")
System.print((-0.0001).toString == "-0.0001")
