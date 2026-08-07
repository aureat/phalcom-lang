// area: arithmetic/operators
// spec: values-and-absence.md
// status: PASS
// ported from wren/test/core/number/divide.wren: dividing by a negative-zero
// literal normalizes to integer zero, so it has the same signs as division by
// zero; 0/-0 is NaN (same IEEE-754 rule as 0/0).
System.print(3 / -0)
System.print(-3 / -0)
System.print(0 / -0)
System.print(-0 / -0)
