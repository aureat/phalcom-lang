// PENDING (U-ITER follow-on; ADR-0035 §3, iteration.md §3): a `continue`
// reached through a materialized block — here a non-Bool `if` condition
// (`Truthy`) forcing the inliner's deopt fallback — must skip to the next
// cursor step. Today it silently no-ops (falls through) instead. The common
// `if (Bool) { continue }` path is unaffected (inliner fast path). This
// `.expected` pins the INTENDED spec output (continue always taken, so no
// element is printed); it fails until the gap is closed.
class Truthy {
    ifTrue(block) { return block.call() }
}

for (x in List.new().add(1).add(2).add(3)) {
    if (Truthy.new()) { continue }
    System.print(x)
}
System.print("done")
