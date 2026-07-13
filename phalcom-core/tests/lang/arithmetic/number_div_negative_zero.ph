// area: arithmetic/operators
// spec: values-and-absence.md
// status: PASS
// ported from wren/test/core/number/divide.wren: dividing by a negative-zero
// literal flips the infinity sign relative to dividing by 0, and 0/-0 is NaN
// (same IEEE-754 rule as 0/0 — arithmetic_div_zero.ph/arithmetic_zero_forms.ph).
System.print(3 / -0)
System.print(-3 / -0)
System.print(0 / -0)
System.print(-0 / -0)
