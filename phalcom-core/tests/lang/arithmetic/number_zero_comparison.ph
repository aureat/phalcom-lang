// area: arithmetic/operators
// spec: values-and-absence.md; messages-and-selectors.md
// status: PASS
// ported from wren/test/core/number/comparison.wren: 0 and -0 compare equal.
System.print(0 < -0)
System.print(-0 < 0)
System.print(0 > -0)
System.print(-0 > 0)
System.print(0 <= -0)
System.print(-0 <= 0)
System.print(0 >= -0)
System.print(-0 >= 0)
