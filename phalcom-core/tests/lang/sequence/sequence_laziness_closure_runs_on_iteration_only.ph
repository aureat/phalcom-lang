// area: sequence
// spec: D.1 eager traversal + E.1 explicit iterator pipeline
// status: PASS
// Explicit iterator map is lazy: closure runs during iteration, not stage creation.

let counter = 0
let f = |x| { counter = counter + 1; x * 2 }
let view = [1, 2, 3].iter.map(f)
System.print(counter)
for (x in view) {
}
System.print(counter)
