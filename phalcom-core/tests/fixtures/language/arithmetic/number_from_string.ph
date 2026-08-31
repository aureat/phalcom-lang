// area: arithmetic/operators
// spec: values-and-absence.md
// status: PASS
// `Number` is abstract. Concrete constructors own text conversion.
System.print(Int.new("123") == 123)
System.print(Int.new("-123") == -123)
System.print(Int.new("-0") == 0)
System.print(Float.new("12.34") == 12.34)
System.print(Float.new("-0.0001") == -0.0001)
