// area: list
// spec: D.1 §14; ADR-0020
// status: PASS
// `fold(initial:using:)` folds left-to-right from its explicit initial value.

const l = []
l.append(1)
l.append(2)
l.append(3)
System.print(l.fold(initial: 0, using: |acc, x| { acc + x }))
