// U-ITER-FIX item 1(a) (ADR-0035 §3, iteration.md §3): a `break` reached
// through a block the inliner *materializes* as a real closure — here a
// non-Bool `if` condition (`Truthy`) forcing the sacred inliner's deopt
// fallback — cannot statically jump into the enclosing loop's own chunk (it
// lives in a different compiler function). Rather than the silent no-op
// U-ITER shipped with, this now fails **loudly**: an `Error.new().raise()`
// trap fires and the program halts. The common `if (Bool) { break }` path is
// unaffected (inliner fast path never takes this deopt twin for a real Bool).
// stdout is only "1" — the trap fires on the first iteration, before "done"
// (and before x=2/x=3) ever print.
class Truthy {
    ifTrue(block) { return block.call() }
}

for (x in List.new().add(1).add(2).add(3)) {
    System.print(x)
    if (Truthy.new()) { break }
}
System.print("done")
