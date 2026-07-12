// U-ITER-FIX item 1(a) (ADR-0035 §3, iteration.md §3): a `continue` reached
// through a materialized block — here a non-Bool `if` condition (`Truthy`)
// forcing the inliner's deopt fallback — cannot statically jump into the
// enclosing loop's own chunk (it lives in a different compiler function).
// Rather than the silent no-op U-ITER shipped with, this now fails
// **loudly**: an `Error.new().raise()` trap fires on the very first
// iteration and the program halts before anything prints. The common
// `if (Bool) { continue }` path is unaffected (inliner fast path).
class Truthy {
    ifTrue(block) { return block.call() }
}

for (x in List.new().add(1).add(2).add(3)) {
    if (Truthy.new()) { continue }
    System.print(x)
}
System.print("done")
