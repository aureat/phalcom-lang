// area: collections
// spec: D.1 §15.1
// status: NEGATIVE

[1, 2].reduce(0, |acc, x| { acc + x })
