// PENDING (U-ITER follow-on; ADR-0035 §3, iteration.md §3): a `break` reached
// through a block the inliner *materializes* as a real closure — here a
// non-Bool `if` condition (`Truthy`) forcing the sacred inliner's deopt
// fallback — must leave the loop. Today it silently no-ops: the jump lands in
// the closure's own chunk, not the loop's. The common `if (Bool) { break }`
// path is unaffected (inliner fast path). This `.expected` pins the INTENDED
// spec output (break fires at x=1); it fails until the gap is closed.
class Truthy {
    ifTrue(block) { return block.call() }
}

for (x in List.new().add(1).add(2).add(3)) {
    System.print(x)
    if (Truthy.new()) { break }
}
System.print("done")
