// area: sequence
// spec: iteration.md §5; ADR-0035; plan.md §3.2 laziness ⊗ effect-timing
// status: PASS
// Lazy map: closure does NOT run at .map call time, only when iterated

var counter = 0
var f = { x => counter = counter + 1; x * 2 }
var view = [1, 2, 3].map(f)
System.print(counter)
for (x in view) {
}
System.print(counter)
