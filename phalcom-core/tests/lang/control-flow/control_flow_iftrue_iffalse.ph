// area: control flow
// spec: control-flow.md; values-and-absence.md
// status: PASS

System.print((3 > 2).ifTrue || { "yes" }.unwrapOr("no"))
